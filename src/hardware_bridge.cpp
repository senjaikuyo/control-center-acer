#include "hardware_bridge.h"
#include <QFile>
#include <QTextStream>
#include <QProcess>
#include <QDBusInterface>
#include <QDBusMessage>
#include <QDBusConnection>
#include <QDBusVariant>
#include <QStorageInfo>
#include <cmath>
#include <algorithm>

HardwareBridge::HardwareBridge(QObject *parent) : QObject(parent) {
    // Default 5-node curves
    m_cpuCurve = { {35.0, 20.0}, {45.0, 35.0}, {55.0, 50.0}, {70.0, 75.0}, {85.0, 100.0} };
    m_gpuCurve = { {40.0, 20.0}, {50.0, 40.0}, {60.0, 55.0}, {72.0, 80.0}, {82.0, 100.0} };

    refreshTelemetry();

    connect(&m_pollTimer, &QTimer::timeout, this, &HardwareBridge::refreshTelemetry);
    m_pollTimer.start(1000);
}

void HardwareBridge::refreshTelemetry() {
    pollSysfs();
    pollNvidia();
    pollMemory();
    pollDisk();

    if (m_fanMode == "Custom") {
        processCustomCurve();
    }

    emit telemetryChanged();
}

void HardwareBridge::pollSysfs() {
    // 1. CPU Frequency
    QFile freqFile("/sys/devices/system/cpu/cpu0/cpufreq/scaling_cur_freq");
    if (freqFile.open(QIODevice::ReadOnly | QIODevice::Text)) {
        QTextStream in(&freqFile);
        int khz = in.readLine().trimmed().toInt();
        if (khz > 0) {
            m_cpuClockMhz = khz / 1000;
        }
    }

    // 2. CPU Temperature (hwmon coretemp)
    for (int i = 0; i < 10; ++i) {
        QString path = QString("/sys/class/hwmon/hwmon%1/temp1_input").arg(i);
        QFile tempFile(path);
        if (tempFile.open(QIODevice::ReadOnly | QIODevice::Text)) {
            QTextStream in(&tempFile);
            int milli = in.readLine().trimmed().toInt();
            if (milli > 0) {
                m_cpuTemp = milli / 1000;
                break;
            }
        }
    }

    // 3. CPU Usage from /proc/stat
    QFile statFile("/proc/stat");
    if (statFile.open(QIODevice::ReadOnly | QIODevice::Text)) {
        QTextStream in(&statFile);
        QString line = in.readLine();
        QStringList parts = line.split(" ", Qt::SkipEmptyParts);
        if (parts.size() >= 5 && parts[0] == "cpu") {
            unsigned long long user = parts[1].toULongLong();
            unsigned long long nice = parts[2].toULongLong();
            unsigned long long system = parts[3].toULongLong();
            unsigned long long idle = parts[4].toULongLong();
            unsigned long long total = user + nice + system + idle;

            if (m_prevTotal > 0 && total > m_prevTotal) {
                unsigned long long dTotal = total - m_prevTotal;
                unsigned long long dIdle = idle - m_prevIdle;
                m_cpuUsagePct = static_cast<int>(std::round(100.0 * (1.0 - static_cast<double>(dIdle) / dTotal)));
                m_cpuUsagePct = std::clamp(m_cpuUsagePct, 0, 100);
            }
            m_prevIdle = idle;
            m_prevTotal = total;
        }
    }

    // 4. Fan Speeds (hwmon fan1_input / fan2_input)
    for (int i = 0; i < 10; ++i) {
        QString f1Path = QString("/sys/class/hwmon/hwmon%1/fan1_input").arg(i);
        QFile f1(f1Path);
        if (f1.open(QIODevice::ReadOnly | QIODevice::Text)) {
            QTextStream in(&f1);
            int rpm = in.readLine().trimmed().toInt();
            if (rpm > 0) {
                m_cpuFanRpm = rpm;
                m_cpuFanPct = std::clamp(static_cast<int>(std::round((rpm / 4800.0) * 100.0)), 0, 100);
            }
        }

        QString f2Path = QString("/sys/class/hwmon/hwmon%1/fan2_input").arg(i);
        QFile f2(f2Path);
        if (f2.open(QIODevice::ReadOnly | QIODevice::Text)) {
            QTextStream in(&f2);
            int rpm = in.readLine().trimmed().toInt();
            if (rpm >= 0) {
                m_gpuFanRpm = rpm;
                m_gpuFanPct = std::clamp(static_cast<int>(std::round((rpm / 4500.0) * 100.0)), 0, 100);
            }
        }
    }
}

void HardwareBridge::pollNvidia() {
    QProcess proc;
    QStringList args;
    args << "--query-gpu=clocks.current.graphics,temperature.gpu,utilization.gpu"
         << "--format=csv,noheader,nounits";
    proc.start("nvidia-smi", args);
    if (proc.waitForFinished(400)) {
        QString output = proc.readAllStandardOutput().trimmed();
        QStringList parts = output.split(",");
        if (parts.size() >= 3) {
            m_gpuClockMhz = parts[0].trimmed().toInt();
            m_gpuTemp = parts[1].trimmed().toInt();
            m_gpuUsagePct = parts[2].trimmed().toInt();
        }
    }
}

void HardwareBridge::pollMemory() {
    QFile memFile("/proc/meminfo");
    if (memFile.open(QIODevice::ReadOnly | QIODevice::Text)) {
        QTextStream in(&memFile);
        unsigned long long memTotal = 0, memAvailable = 0;
        while (!in.atEnd()) {
            QString line = in.readLine();
            if (line.startsWith("MemTotal:")) {
                memTotal = line.split(" ", Qt::SkipEmptyParts).value(1).toULongLong();
            } else if (line.startsWith("MemAvailable:")) {
                memAvailable = line.split(" ", Qt::SkipEmptyParts).value(1).toULongLong();
            }
        }
        if (memTotal > 0) {
            m_memTotalGb = std::round((memTotal / (1024.0 * 1024.0)) * 10.0) / 10.0;
            unsigned long long used = memTotal - memAvailable;
            m_memPct = std::clamp(static_cast<int>(std::round((static_cast<double>(used) / memTotal) * 100.0)), 0, 100);
        }
    }
}

void HardwareBridge::pollDisk() {
    QStorageInfo storage = QStorageInfo::root();
    if (storage.isValid() && storage.isReady()) {
        qint64 total = storage.bytesTotal();
        qint64 available = storage.bytesAvailable();
        if (total > 0) {
            m_diskTotalGb = static_cast<int>(total / (1024 * 1024 * 1024));
            qint64 used = total - available;
            m_diskPct = std::clamp(static_cast<int>(std::round((static_cast<double>(used) / total) * 100.0)), 0, 100);
        }
    }
}

void HardwareBridge::setPowerMode(const QString &mode) {
    m_powerMode = mode;
    emit powerModeChanged(mode);

    QString profile = "balanced";
    if (mode == "Performance") profile = "performance";
    else if (mode == "Quiet") profile = "power-saver";

    QDBusInterface iface("net.hadess.PowerProfiles",
                         "/net/hadess/PowerProfiles",
                         "org.freedesktop.DBus.Properties",
                         QDBusConnection::systemBus());
    if (iface.isValid()) {
        iface.call("Set", "net.hadess.PowerProfiles", "ActiveProfile", QVariant::fromValue(QDBusVariant(profile)));
    }
}

void HardwareBridge::setFanMode(const QString &mode) {
    m_fanMode = mode;
    emit fanModeChanged(mode);

    if (mode == "Maximum" || mode == "Turbo") {
        applyFanSpeed(0, 100);
        applyFanSpeed(1, 100);
    } else if (mode == "Automatic") {
        applyFanSpeed(0, 0); // 0 restores automatic firmware control
        applyFanSpeed(1, 0);
    } else if (mode == "Custom") {
        processCustomCurve();
    }
}

void HardwareBridge::applyFanSpeed(int fanIndex, int speedPercent) {
    int pwm = static_cast<int>(std::round((speedPercent / 100.0) * 255.0));

    QDBusInterface iface("com.tux.Tailor",
                         "/com/tux/Tailor",
                         "com.tux.Tailor.Fan",
                         QDBusConnection::systemBus());
    if (iface.isValid()) {
        uchar fIdx = static_cast<uchar>(std::clamp(fanIndex, 0, 1));
        uchar pwmVal = static_cast<uchar>(std::clamp(pwm, 0, 255));
        iface.call("OverrideSpeed", fIdx, pwmVal);
    }
}

int HardwareBridge::interpolateCurve(const std::vector<CurveNode> &nodes, double currentTemp) {
    if (nodes.empty()) return 30;
    if (currentTemp <= nodes.front().temp) return static_cast<int>(nodes.front().speed);
    if (currentTemp >= nodes.back().temp) return static_cast<int>(nodes.back().speed);

    for (size_t i = 0; i < nodes.size() - 1; ++i) {
        if (currentTemp >= nodes[i].temp && currentTemp <= nodes[i + 1].temp) {
            double range = nodes[i + 1].temp - nodes[i].temp;
            if (range <= 0.001) return static_cast<int>(nodes[i].speed);
            double t = (currentTemp - nodes[i].temp) / range;
            return static_cast<int>(std::round(nodes[i].speed + t * (nodes[i + 1].speed - nodes[i].speed)));
        }
    }
    return 30;
}

void HardwareBridge::processCustomCurve() {
    int cpuDuty = interpolateCurve(m_cpuCurve, m_cpuTemp);
    int gpuDuty = interpolateCurve(m_gpuCurve, m_gpuTemp);

    applyFanSpeed(0, cpuDuty);
    applyFanSpeed(1, gpuDuty);
}

void HardwareBridge::applyFanCurve(const QString &targetFan, const QVariantList &nodes) {
    std::vector<CurveNode> parsed;
    for (const auto &item : nodes) {
        QVariantMap map = item.toMap();
        CurveNode node;
        node.temp = map.value("temp", 35.0).toDouble();
        node.speed = map.value("speed", 20.0).toDouble();
        parsed.push_back(node);
    }
    std::sort(parsed.begin(), parsed.end(), [](const CurveNode &a, const CurveNode &b) {
        return a.temp < b.temp;
    });

    if (targetFan == "cpu") {
        m_cpuCurve = parsed;
    } else {
        m_gpuCurve = parsed;
    }

    if (m_fanMode == "Custom") {
        processCustomCurve();
    }
}

void HardwareBridge::setTouchpadEnabled(bool enabled) {
    QDBusInterface iface("org.kde.KWin",
                         "/org/kde/KWin/InputDevice/event14",
                         "org.freedesktop.DBus.Properties",
                         QDBusConnection::sessionBus());
    if (iface.isValid()) {
        iface.call("Set", "org.kde.KWin.InputDevice", "enabled", QVariant::fromValue(QDBusVariant(enabled)));
    }
}

void HardwareBridge::setAirplaneMode(bool enabled) {
    QProcess::execute("rfkill", QStringList() << (enabled ? "block" : "unblock") << "all");
}

void HardwareBridge::setNightLight(bool enabled, qreal level) {
    Q_UNUSED(level);
    QDBusInterface iface("org.kde.KWin",
                         "/ColorCorrect",
                         "org.kde.kwin.ColorCorrect",
                         QDBusConnection::sessionBus());
    if (iface.isValid()) {
        iface.call("setNightLight", enabled);
    }
}

void HardwareBridge::setKeyboardColor(int r, int g, int b) {
    // 1. Direct D-Bus to com.tux.Tailor.Led OverrideColor
    QDBusInterface iface("com.tux.Tailor",
                         "/com/tux/Tailor",
                         "com.tux.Tailor.Led",
                         QDBusConnection::systemBus());
    if (iface.isValid()) {
        QString hexCol = QString("#%1%2%3")
            .arg(r & 0xFF, 2, 16, QChar('0'))
            .arg(g & 0xFF, 2, 16, QChar('0'))
            .arg(b & 0xFF, 2, 16, QChar('0')).toUpper();
        iface.call("OverrideColor", hexCol);
    }

    // 2. Sysfs fallback
    QFile file("/sys/class/leds/rgb:kbd_backlight/multi_intensity");
    if (file.open(QIODevice::WriteOnly | QIODevice::Text)) {
        QTextStream out(&file);
        out << r << " " << g << " " << b << "\n";
    }
}

void HardwareBridge::setKeyboardBrightness(int brightnessPercent) {
    int val = static_cast<int>(std::round((brightnessPercent / 100.0) * 255.0));
    QFile file("/sys/class/leds/rgb:kbd_backlight/brightness");
    if (file.open(QIODevice::WriteOnly | QIODevice::Text)) {
        QTextStream out(&file);
        out << val << "\n";
    }

    QProcess::execute("brightnessctl", QStringList() << "--device=rgb:kbd_backlight" << "set" << QString("%1%").arg(brightnessPercent));
}
