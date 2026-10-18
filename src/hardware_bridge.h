#pragma once

#include <QObject>
#include <QString>
#include <QVariantList>
#include <QTimer>
#include <vector>

struct CurveNode {
    double temp{35.0};
    double speed{20.0};
};

class HardwareBridge : public QObject {
    Q_OBJECT

    // Telemetry Properties
    Q_PROPERTY(QString cpuName READ cpuName NOTIFY telemetryChanged)
    Q_PROPERTY(QString gpuName READ gpuName NOTIFY telemetryChanged)
    Q_PROPERTY(int cpuClockMhz READ cpuClockMhz NOTIFY telemetryChanged)
    Q_PROPERTY(int cpuTemp READ cpuTemp NOTIFY telemetryChanged)
    Q_PROPERTY(int cpuUsagePct READ cpuUsagePct NOTIFY telemetryChanged)
    Q_PROPERTY(int gpuClockMhz READ gpuClockMhz NOTIFY telemetryChanged)
    Q_PROPERTY(int gpuTemp READ gpuTemp NOTIFY telemetryChanged)
    Q_PROPERTY(int gpuUsagePct READ gpuUsagePct NOTIFY telemetryChanged)

    // Fan Properties
    Q_PROPERTY(int cpuFanRpm READ cpuFanRpm NOTIFY telemetryChanged)
    Q_PROPERTY(int gpuFanRpm READ gpuFanRpm NOTIFY telemetryChanged)
    Q_PROPERTY(int cpuFanPct READ cpuFanPct NOTIFY telemetryChanged)
    Q_PROPERTY(int gpuFanPct READ gpuFanPct NOTIFY telemetryChanged)
    Q_PROPERTY(QString fanMode READ fanMode NOTIFY fanModeChanged)

    // Power Mode ("Performance", "Entertainment", "Quiet")
    Q_PROPERTY(QString powerMode READ powerMode NOTIFY powerModeChanged)

    // Memory & Storage
    Q_PROPERTY(int memPct READ memPct NOTIFY telemetryChanged)
    Q_PROPERTY(double memTotalGb READ memTotalGb NOTIFY telemetryChanged)
    Q_PROPERTY(int diskPct READ diskPct NOTIFY telemetryChanged)
    Q_PROPERTY(int diskTotalGb READ diskTotalGb NOTIFY telemetryChanged)

public:
    explicit HardwareBridge(QObject *parent = nullptr);

    QString cpuName() const { return m_cpuName; }
    QString gpuName() const { return m_gpuName; }
    int cpuClockMhz() const { return m_cpuClockMhz; }
    int cpuTemp() const { return m_cpuTemp; }
    int cpuUsagePct() const { return m_cpuUsagePct; }
    int gpuClockMhz() const { return m_gpuClockMhz; }
    int gpuTemp() const { return m_gpuTemp; }
    int gpuUsagePct() const { return m_gpuUsagePct; }

    int cpuFanRpm() const { return m_cpuFanRpm; }
    int gpuFanRpm() const { return m_gpuFanRpm; }
    int cpuFanPct() const { return m_cpuFanPct; }
    int gpuFanPct() const { return m_gpuFanPct; }
    QString fanMode() const { return m_fanMode; }
    QString powerMode() const { return m_powerMode; }

    int memPct() const { return m_memPct; }
    double memTotalGb() const { return m_memTotalGb; }
    int diskPct() const { return m_diskPct; }
    int diskTotalGb() const { return m_diskTotalGb; }

    Q_INVOKABLE void setPowerMode(const QString &mode);
    Q_INVOKABLE void setFanMode(const QString &mode);
    Q_INVOKABLE void applyFanSpeed(int fanIndex, int speedPercent);
    Q_INVOKABLE void applyFanCurve(const QString &targetFan, const QVariantList &nodes);
    Q_INVOKABLE void setTouchpadEnabled(bool enabled);
    Q_INVOKABLE void setAirplaneMode(bool enabled);
    Q_INVOKABLE void setNightLight(bool enabled, qreal level);
    Q_INVOKABLE void setKeyboardColor(int r, int g, int b);
    Q_INVOKABLE void setKeyboardBrightness(int brightnessPercent);
    Q_INVOKABLE void refreshTelemetry();

signals:
    void telemetryChanged();
    void fanModeChanged(const QString &mode);
    void powerModeChanged(const QString &mode);

private:
    void pollSysfs();
    void pollNvidia();
    void pollMemory();
    void pollDisk();
    void processCustomCurve();
    int interpolateCurve(const std::vector<CurveNode> &nodes, double currentTemp);

    QTimer m_pollTimer;

    QString m_cpuName{"13th Gen Intel(R) Core(TM) i5-13420H"};
    QString m_gpuName{"NVIDIA GeForce RTX 3050 6GB Laptop GPU"};
    int m_cpuClockMhz{2100};
    int m_cpuTemp{50};
    int m_cpuUsagePct{5};
    int m_gpuClockMhz{210};
    int m_gpuTemp{45};
    int m_gpuUsagePct{0};

    int m_cpuFanRpm{1900};
    int m_gpuFanRpm{900};
    int m_cpuFanPct{38};
    int m_gpuFanPct{20};
    QString m_fanMode{"Automatic"};
    QString m_powerMode{"Quiet"};

    int m_memPct{54};
    double m_memTotalGb{15.3};
    int m_diskPct{64};
    int m_diskTotalGb{162};

    unsigned long long m_prevIdle{0};
    unsigned long long m_prevTotal{0};

    std::vector<CurveNode> m_cpuCurve;
    std::vector<CurveNode> m_gpuCurve;
};
