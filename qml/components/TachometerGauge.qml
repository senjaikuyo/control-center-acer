import QtQuick
import QtQuick.Controls
import QtQuick.Layouts

Item {
    id: gaugeRoot
    property bool isGpu: false

    ColumnLayout {
        anchors.fill: parent
        spacing: 4

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

        // Center Dial Gauge Area using Authentic OEM Texture Stack
        Item {
            Layout.fillWidth: true
            Layout.fillHeight: true

            Item {
                id: dialContainer
                width: 240
                height: 240
                anchors.centerIn: parent

                // 1. Electromagnetic Wave Background (Behind Gauge)
                Canvas {
                    anchors.fill: parent
                    antialiasing: true
                    onPaint: {
                        var ctx = getContext("2d");
                        ctx.reset();
                        var cx = width / 2;
                        var cy = height / 2;
                        ctx.lineWidth = 1;
                        for (var side = -1; side <= 1; side += 2) {
                            for (var w = 0; w < 4; ++w) {
                                var offset_y = (w - 1.5) * 11;
                                var alpha = 0.45 * (1.0 - w * 0.2);
                                ctx.strokeStyle = "rgba(255, 179, 0, " + alpha + ")";
                                ctx.beginPath();
                                for (var step = 0; step < 20; ++step) {
                                    var t = step / 19.0;
                                    var x = cx + side * (26 + t * 90);
                                    var wave = Math.sin(t * Math.PI * 2.5) * 5.0;
                                    var y = cy + offset_y + wave;
                                    if (step === 0) ctx.moveTo(x, y);
                                    else ctx.lineTo(x, y);
                                }
                                ctx.stroke();
                            }
                        }
                    }
                }

                // 2. Authentic OEM Scale Dial Base Ring
                Image {
                    id: baseDialImg
                    anchors.fill: parent
                    source: "qrc:/AcerControlCenter/qml/assets/temp_dial_bg.png"
                    fillMode: Image.PreserveAspectFit
                    smooth: true
                    mipmap: true
                }

                // 3. Authentic OEM Gradient Color Arc
                Image {
                    id: arcDialImg
                    anchors.fill: parent
                    source: "qrc:/AcerControlCenter/qml/assets/temp_dial_arc.png"
                    fillMode: Image.PreserveAspectFit
                    smooth: true
                    mipmap: true
                }

                // 4. Authentic Left Needle (Clock Speed)
                Image {
                    id: leftNeedle
                    anchors.fill: parent
                    source: "qrc:/AcerControlCenter/qml/assets/temp_needle1.png"
                    fillMode: Image.PreserveAspectFit
                    smooth: true
                    mipmap: true

                    readonly property real curClock: gaugeRoot.isGpu ? hardware.gpuClockMhz : hardware.cpuClockMhz
                    readonly property real maxClock: gaugeRoot.isGpu ? 2500.0 : 6000.0
                    readonly property real ratio: Math.min(Math.max(curClock / maxClock, 0.0), 1.0)
                    readonly property real targetAngle: -180.0 * ratio

                    transform: Rotation {
                        origin.x: leftNeedle.width / 2
                        origin.y: leftNeedle.height / 2
                        angle: leftNeedle.targetAngle
                        Behavior on angle {
                            NumberAnimation { duration: 400; easing.type: Easing.OutQuad }
                        }
                    }
                }

                // 5. Authentic Right Needle (Temperature)
                Image {
                    id: rightNeedle
                    anchors.fill: parent
                    source: "qrc:/AcerControlCenter/qml/assets/temp_needle2.png"
                    fillMode: Image.PreserveAspectFit
                    smooth: true
                    mipmap: true

                    readonly property real curTemp: gaugeRoot.isGpu ? hardware.gpuTemp : hardware.cpuTemp
                    readonly property real ratio: Math.min(Math.max(curTemp / 100.0, 0.0), 1.0)
                    readonly property real targetAngle: 180.0 * ratio

                    transform: Rotation {
                        origin.x: rightNeedle.width / 2
                        origin.y: rightNeedle.height / 2
                        angle: rightNeedle.targetAngle
                        Behavior on angle {
                            NumberAnimation { duration: 400; easing.type: Easing.OutQuad }
                        }
                    }
                }

                // 6. Authentic Central Knob
                Image {
                    id: centerKnob
                    width: 38
                    height: 38
                    anchors.centerIn: parent
                    source: "qrc:/AcerControlCenter/qml/assets/knob_fan_control.png"
                    fillMode: Image.PreserveAspectFit
                    smooth: true
                }

                // Center Label Badge
                Rectangle {
                    anchors.centerIn: parent
                    width: 28
                    height: 18
                    radius: 3
                    color: "#D00B0E14"
                    border.color: gaugeRoot.isGpu ? "#00E1FF" : "#FFB300"
                    border.width: 1

                    Text {
                        anchors.centerIn: parent
                        text: gaugeRoot.isGpu ? "GPU" : "CPU"
                        font.pixelSize: 9
                        font.bold: true
                        color: "#F0F6FC"
                    }
                }

                // Hub Click Area to toggle CPU/GPU
                MouseArea {
                    anchors.centerIn: parent
                    width: 50
                    height: 50
                    cursorShape: Qt.PointingHandCursor
                    onClicked: gaugeRoot.isGpu = !gaugeRoot.isGpu
                }

                // Bottom Labels (Clock & Temperature)
                Column {
                    anchors.bottom: parent.bottom
                    anchors.left: parent.left
                    anchors.bottomMargin: 8
                    anchors.leftMargin: 12
                    Text { text: "Clock speed"; font.pixelSize: 9; color: "#8B949E"; horizontalAlignment: Text.AlignRight }
                    Text { text: gaugeRoot.isGpu ? "x100 MHz" : "x1000 MHz"; font.pixelSize: 9; color: "#8B949E"; horizontalAlignment: Text.AlignRight }
                }

                Column {
                    anchors.bottom: parent.bottom
                    anchors.right: parent.right
                    anchors.bottomMargin: 8
                    anchors.rightMargin: 12
                    Text {
                        text: (gaugeRoot.isGpu && hardware.gpuTemp === 0) ? "SLEEP" : (hardware.cpuTemp + " °C")
                        font.pixelSize: 10
                        font.bold: true
                        color: "#F0F6FC"
                    }
                    Text { text: "Temperature"; font.pixelSize: 9; color: "#8B949E" }
                }
            }
        }
    }
}
