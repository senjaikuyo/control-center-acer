import QtQuick
import QtQuick.Controls
import QtQuick.Layouts

Item {
    id: gaugeRoot
    property bool isGpu: false

    ColumnLayout {
        anchors.fill: parent
        spacing: 6

        // Header inside card
        RowLayout {
            Layout.fillWidth: true

            Text {
                text: gaugeRoot.isGpu ? "GPU Telemetry" : "Processor Telemetry"
                font.pixelSize: 14
                font.bold: true
                color: "#F0F6FC"
            }

            Item { Layout.fillWidth: true }

            Rectangle {
                width: 98
                height: 22
                radius: 3
                color: "#201C0E"
                border.color: "#FFB300"
                border.width: 1

                Text {
                    anchors.centerIn: parent
                    text: "⚙ Atur Kurva >>"
                    font.pixelSize: 10
                    font.bold: true
                    color: "#FFB300"
                }

                MouseArea {
                    anchors.fill: parent
                    cursorShape: Qt.PointingHandCursor
                    onClicked: hardware.setFanMode("Custom")
                }
            }
        }

        // Center Dial Gauge Area
        Item {
            Layout.fillWidth: true
            Layout.fillHeight: true

            Canvas {
                id: gaugeCanvas
                anchors.fill: parent
                antialiasing: true

                property real curClock: gaugeRoot.isGpu ? hardware.gpuClockMhz : hardware.cpuClockMhz
                property real maxClock: gaugeRoot.isGpu ? 2500.0 : 6000.0
                property real curTemp: gaugeRoot.isGpu ? hardware.gpuTemp : hardware.cpuTemp

                onCurClockChanged: requestPaint()
                onCurTempChanged: requestPaint()
                onWidthChanged: requestPaint()
                onHeightChanged: requestPaint()

                onPaint: {
                    var ctx = getContext("2d");
                    ctx.reset();

                    var cx = width / 2;
                    var cy = height / 2 + 8;
                    var radius = 64;

                    // 1. Amber electromagnetic wave lines behind hub
                    ctx.lineWidth = 1;
                    for (var side = -1; side <= 1; side += 2) {
                        for (var w = 0; w < 4; ++w) {
                            var offset_y = (w - 1.5) * 10;
                            var alpha = 0.45 * (1.0 - w * 0.2);
                            ctx.strokeStyle = "rgba(255, 179, 0, " + alpha + ")";
                            ctx.beginPath();
                            for (var step = 0; step < 20; ++step) {
                                var t = step / 19.0;
                                var x = cx + side * (26 + t * 76);
                                var wave = Math.sin(t * Math.PI * 2.5) * 4.5;
                                var y = cy + offset_y + wave;
                                if (step === 0) ctx.moveTo(x, y);
                                else ctx.lineTo(x, y);
                            }
                            ctx.stroke();
                        }
                    }

                    // 2. Left Arc Track (Clock Speed: Top-to-Bottom on left)
                    ctx.lineWidth = 3.5;
                    ctx.strokeStyle = "#212937";
                    ctx.beginPath();
                    ctx.arc(cx, cy, radius, Math.PI * 1.5, Math.PI * 0.5, true);
                    ctx.stroke();

                    // Active Clock Arc Fill (Cyan)
                    var clkRatio = Math.min(Math.max(curClock / maxClock, 0.0), 1.0);
                    ctx.strokeStyle = "#00E1FF";
                    ctx.beginPath();
                    ctx.arc(cx, cy, radius, Math.PI * 1.5, Math.PI * 1.5 - (Math.PI * clkRatio), true);
                    ctx.stroke();

                    // Left Ticks
                    ctx.fillStyle = "#8B949E";
                    ctx.font = "8px sans-serif";
                    ctx.textAlign = "center";
                    ctx.textBaseline = "middle";
                    var maxTicks = gaugeRoot.isGpu ? 5 : 6;
                    for (var i = 0; i <= maxTicks; ++i) {
                        var a = Math.PI * 1.5 - (Math.PI * (i / maxTicks));
                        var tx = cx + (radius + 11) * Math.cos(a);
                        var ty = cy + (radius + 11) * Math.sin(a);
                        var lbl = gaugeRoot.isGpu ? (i * 5) : i;
                        ctx.fillText(lbl, tx, ty);
                    }

                    // 3. Right Arc Track (Temperature: Top-to-Bottom on right)
                    // Green segment (0 - 45°C)
                    ctx.lineWidth = 3.5;
                    ctx.strokeStyle = "#00E676";
                    ctx.beginPath();
                    ctx.arc(cx, cy, radius, Math.PI * 1.5, Math.PI * 1.5 + (Math.PI * 0.35), false);
                    ctx.stroke();

                    // Yellow segment (45 - 75°C)
                    ctx.strokeStyle = "#FFD600";
                    ctx.beginPath();
                    ctx.arc(cx, cy, radius, Math.PI * 1.5 + (Math.PI * 0.35), Math.PI * 1.5 + (Math.PI * 0.75), false);
                    ctx.stroke();

                    // Red segment (75 - 100°C)
                    ctx.strokeStyle = "#FF3D00";
                    ctx.beginPath();
                    ctx.arc(cx, cy, radius, Math.PI * 1.5 + (Math.PI * 0.75), Math.PI * 2.5, false);
                    ctx.stroke();

                    // 4. Needles
                    // Left Needle (Clock)
                    var needleLeftAngle = Math.PI * 1.5 - (Math.PI * clkRatio);
                    var n1x = cx + (radius - 4) * Math.cos(needleLeftAngle);
                    var n1y = cy + (radius - 4) * Math.sin(needleLeftAngle);
                    ctx.lineWidth = 2.2;
                    ctx.strokeStyle = gaugeRoot.isGpu ? "#00E1FF" : "#FFB300";
                    ctx.beginPath();
                    ctx.moveTo(cx, cy);
                    ctx.lineTo(n1x, n1y);
                    ctx.stroke();

                    // Right Needle (Temperature)
                    var tempRatio = Math.min(Math.max(curTemp / 100.0, 0.0), 1.0);
                    var needleRightAngle = Math.PI * 1.5 + (Math.PI * tempRatio);
                    var n2x = cx + (radius - 4) * Math.cos(needleRightAngle);
                    var n2y = cy + (radius - 4) * Math.sin(needleRightAngle);
                    ctx.lineWidth = 2.2;
                    ctx.strokeStyle = gaugeRoot.isGpu ? "#FFB300" : "#00E1FF";
                    ctx.beginPath();
                    ctx.moveTo(cx, cy);
                    ctx.lineTo(n2x, n2y);
                    ctx.stroke();

                    // 5. Center Hub (Double ring with purple aura)
                    ctx.lineWidth = 2.0;
                    ctx.strokeStyle = "#9C27B0";
                    ctx.beginPath();
                    ctx.arc(cx, cy, 25, 0, Math.PI * 2);
                    ctx.stroke();

                    ctx.lineWidth = 1.8;
                    ctx.strokeStyle = gaugeRoot.isGpu ? "#00E1FF" : "#FFB300";
                    ctx.fillStyle = "#121620";
                    ctx.beginPath();
                    ctx.arc(cx, cy, 20, 0, Math.PI * 2);
                    ctx.fill();
                    ctx.stroke();

                    ctx.fillStyle = "#F0F6FC";
                    ctx.font = "bold 11px sans-serif";
                    ctx.fillText(gaugeRoot.isGpu ? "GPU" : "CPU", cx, cy);

                    // 6. Labels below arcs (2 explicit lines)
                    ctx.font = "9px sans-serif";
                    ctx.fillStyle = "#8B949E";

                    // Left labels
                    ctx.textAlign = "right";
                    ctx.fillText("Clock speed", cx - radius - 8, cy + 14);
                    ctx.fillText(gaugeRoot.isGpu ? "x100 MHz" : "x1000 MHz", cx - radius - 8, cy + 26);

                    // Right labels
                    ctx.textAlign = "left";
                    if (gaugeRoot.isGpu && curTemp === 0) {
                        ctx.fillText("SLEEP", cx + radius + 8, cy + 14);
                        ctx.fillText("Optimus", cx + radius + 8, cy + 26);
                    } else {
                        ctx.fillText(curTemp + " °C", cx + radius + 8, cy + 14);
                        ctx.fillText("Temperature", cx + radius + 8, cy + 26);
                    }
                }
            }

            // Clickable Center Hub Area to toggle CPU/GPU
            MouseArea {
                anchors.centerIn: parent
                width: 50
                height: 50
                cursorShape: Qt.PointingHandCursor
                onClicked: {
                    gaugeRoot.isGpu = !gaugeRoot.isGpu;
                    gaugeCanvas.requestPaint();
                }
            }
        }
    }
}
