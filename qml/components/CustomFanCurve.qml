import QtQuick
import QtQuick.Controls
import QtQuick.Layouts

Item {
    id: curveRoot

    property int activeTab: 0 // 0: CPU Fan, 1: GPU Fan
    property int selectedIndex: 0
    property bool hasUnsavedChanges: false
    property string feedbackMessage: ""

    // 5 default control points for CPU and GPU
    property var cpuNodes: [
        { temp: 35, speed: 20 },
        { temp: 45, speed: 35 },
        { temp: 55, speed: 50 },
        { temp: 70, speed: 75 },
        { temp: 85, speed: 100 }
    ]

    property var gpuNodes: [
        { temp: 40, speed: 20 },
        { temp: 50, speed: 40 },
        { temp: 60, speed: 55 },
        { temp: 72, speed: 80 },
        { temp: 82, speed: 100 }
    ]

    function currentNodes() {
        return activeTab === 0 ? cpuNodes : gpuNodes;
    }

    function setNode(idx, t, s) {
        var list = activeTab === 0 ? cpuNodes : gpuNodes;
        if (idx >= 0 && idx < list.length) {
            list[idx].temp = Math.max(20, Math.min(100, Math.round(t)));
            list[idx].speed = Math.max(0, Math.min(100, Math.round(s)));
            hasUnsavedChanges = true;
            feedbackMessage = "";
            curveCanvas.requestPaint();
        }
    }

    function resetCurve() {
        if (activeTab === 0) {
            cpuNodes = [
                { temp: 35, speed: 20 },
                { temp: 45, speed: 35 },
                { temp: 55, speed: 50 },
                { temp: 70, speed: 75 },
                { temp: 85, speed: 100 }
            ];
        } else {
            gpuNodes = [
                { temp: 40, speed: 20 },
                { temp: 50, speed: 40 },
                { temp: 60, speed: 55 },
                { temp: 72, speed: 80 },
                { temp: 82, speed: 100 }
            ];
        }
        hasUnsavedChanges = true;
        feedbackMessage = "Kurva di-reset ke nilai default";
        curveCanvas.requestPaint();
    }

    function applyCurve() {
        hardware.applyFanCurve(activeTab === 0 ? "cpu" : "gpu", currentNodes());
        hasUnsavedChanges = false;
        feedbackMessage = "✓ Kurva berhasil diterapkan ke hardware";
        curveCanvas.requestPaint();
    }

    ColumnLayout {
        anchors.fill: parent
        spacing: 4

        // Top Row: Title & Back Button
        RowLayout {
            Layout.fillWidth: true

            Text {
                text: "Custom fan setting"
                font.pixelSize: 14
                font.bold: true
                color: "#F0F6FC"
            }

            Item { Layout.fillWidth: true }

            Rectangle {
                width: 80
                height: 22
                radius: 3
                color: "#201C0E"
                border.color: "#FFB300"
                border.width: 1

                Text {
                    anchors.centerIn: parent
                    text: "<< Kembali"
                    font.pixelSize: 10
                    color: "#FFB300"
                }

                MouseArea {
                    anchors.fill: parent
                    cursorShape: Qt.PointingHandCursor
                    onClicked: hardware.setFanMode("Automatic")
                }
            }
        }

        // Subtabs: CPU Fan | GPU Fan
        Row {
            spacing: 8
            Layout.topMargin: 2
            Layout.bottomMargin: 2

            // CPU Fan Tab
            Rectangle {
                width: 76
                height: 22
                color: "transparent"

                Text {
                    anchors.centerIn: parent
                    text: "CPU Fan"
                    font.pixelSize: 12
                    font.bold: curveRoot.activeTab === 0
                    color: curveRoot.activeTab === 0 ? "#FFB300" : "#8B949E"
                }

                Rectangle {
                    anchors.bottom: parent.bottom
                    anchors.horizontalCenter: parent.horizontalCenter
                    width: parent.width - 12
                    height: 2
                    color: "#FFB300"
                    visible: curveRoot.activeTab === 0
                }

                MouseArea {
                    anchors.fill: parent
                    cursorShape: Qt.PointingHandCursor
                    onClicked: {
                        curveRoot.activeTab = 0;
                        curveRoot.selectedIndex = 0;
                        curveCanvas.requestPaint();
                    }
                }
            }

            // GPU Fan Tab
            Rectangle {
                width: 76
                height: 22
                color: "transparent"

                Text {
                    anchors.centerIn: parent
                    text: "GPU Fan"
                    font.pixelSize: 12
                    font.bold: curveRoot.activeTab === 1
                    color: curveRoot.activeTab === 1 ? "#00E1FF" : "#8B949E"
                }

                Rectangle {
                    anchors.bottom: parent.bottom
                    anchors.horizontalCenter: parent.horizontalCenter
                    width: parent.width - 12
                    height: 2
                    color: "#00E1FF"
                    visible: curveRoot.activeTab === 1
                }

                MouseArea {
                    anchors.fill: parent
                    cursorShape: Qt.PointingHandCursor
                    onClicked: {
                        curveRoot.activeTab = 1;
                        curveRoot.selectedIndex = 0;
                        curveCanvas.requestPaint();
                    }
                }
            }
        }

        // 2D Curve Canvas Grid
        Item {
            Layout.fillWidth: true
            Layout.preferredHeight: 125

            Rectangle {
                anchors.fill: parent
                radius: 3
                color: "#0B0E14"
                border.color: "#1A212E"
                border.width: 1
            }

            Canvas {
                id: curveCanvas
                anchors.fill: parent
                antialiasing: true

                readonly property int padLeft: 24
                readonly property int padRight: 14
                readonly property int padTop: 8
                readonly property int padBottom: 16

                onPaint: {
                    var ctx = getContext("2d");
                    ctx.reset();

                    var gw = width - padLeft - padRight;
                    var gh = height - padTop - padBottom;
                    var colorAccent = curveRoot.activeTab === 0 ? "#FFB300" : "#00E1FF";

                    // 1. Grid Lines (Y: 0%, 25%, 50%, 75%, 100%)
                    ctx.lineWidth = 0.8;
                    ctx.strokeStyle = "#161C26";
                    ctx.fillStyle = "#5A6473";
                    ctx.font = "8px sans-serif";
                    ctx.textAlign = "right";
                    ctx.textBaseline = "middle";

                    for (var yStep = 0; yStep <= 4; ++yStep) {
                        var yPct = yStep / 4.0;
                        var y = padTop + gh * (1.0 - yPct);
                        ctx.beginPath();
                        ctx.moveTo(padLeft, y);
                        ctx.lineTo(padLeft + gw, y);
                        ctx.stroke();

                        ctx.fillText(Math.round(yPct * 100), padLeft - 4, y);
                    }

                    // 2. Grid Lines (X: 20°C, 40°C, 60°C, 80°C, 100°C)
                    ctx.textAlign = "center";
                    for (var xStep = 0; xStep <= 4; ++xStep) {
                        var xPct = xStep / 4.0;
                        var x = padLeft + gw * xPct;
                        ctx.beginPath();
                        ctx.moveTo(x, padTop);
                        ctx.lineTo(x, padTop + gh);
                        ctx.stroke();

                        var tempVal = 20 + Math.round(xPct * 80);
                        ctx.fillText(tempVal, x, padTop + gh + 9);
                    }

                    // 3. Connect Curve Nodes with Lines
                    var nodes = curveRoot.currentNodes();
                    if (nodes.length > 1) {
                        ctx.lineWidth = 2.2;
                        ctx.strokeStyle = colorAccent;
                        ctx.beginPath();

                        for (var i = 0; i < nodes.length; ++i) {
                            var nx = padLeft + ((nodes[i].temp - 20) / 80.0) * gw;
                            var ny = padTop + (1.0 - (nodes[i].speed / 100.0)) * gh;
                            if (i === 0) ctx.moveTo(nx, ny);
                            else ctx.lineTo(nx, ny);
                        }
                        ctx.stroke();
                    }

                    // 4. Draw Circle Nodes
                    for (var j = 0; j < nodes.length; ++j) {
                        var cx = padLeft + ((nodes[j].temp - 20) / 80.0) * gw;
                        var cy = padTop + (1.0 - (nodes[j].speed / 100.0)) * gh;
                        var isSel = (j === curveRoot.selectedIndex);

                        if (isSel) {
                            ctx.beginPath();
                            ctx.arc(cx, cy, 7, 0, Math.PI * 2);
                            ctx.strokeStyle = colorAccent;
                            ctx.lineWidth = 1.2;
                            ctx.stroke();

                            ctx.beginPath();
                            ctx.arc(cx, cy, 4.5, 0, Math.PI * 2);
                            ctx.fillStyle = "white";
                            ctx.fill();
                            ctx.strokeStyle = colorAccent;
                            ctx.lineWidth = 1.8;
                            ctx.stroke();
                        } else {
                            ctx.beginPath();
                            ctx.arc(cx, cy, 4, 0, Math.PI * 2);
                            ctx.fillStyle = "white";
                            ctx.fill();
                            ctx.strokeStyle = colorAccent;
                            ctx.lineWidth = 1.5;
                            ctx.stroke();
                        }
                    }
                }
            }

            // Mouse interaction on canvas
            MouseArea {
                anchors.fill: parent
                hoverEnabled: true

                function getCoords(mx, my) {
                    var gw = width - curveCanvas.padLeft - curveCanvas.padRight;
                    var gh = height - curveCanvas.padTop - curveCanvas.padBottom;
                    var t = 20 + ((mx - curveCanvas.padLeft) / gw) * 80;
                    var s = (1.0 - (my - curveCanvas.padTop) / gh) * 100;
                    return { temp: t, speed: s };
                }

                onPressed: function(mouse) {
                    var gw = width - curveCanvas.padLeft - curveCanvas.padRight;
                    var gh = height - curveCanvas.padTop - curveCanvas.padBottom;
                    var nodes = curveRoot.currentNodes();

                    for (var i = 0; i < nodes.length; ++i) {
                        var nx = curveCanvas.padLeft + ((nodes[i].temp - 20) / 80.0) * gw;
                        var ny = curveCanvas.padTop + (1.0 - (nodes[i].speed / 100.0)) * gh;
                        var dist = Math.hypot(mouse.x - nx, mouse.y - ny);
                        if (dist < 14) {
                            curveRoot.selectedIndex = i;
                            curveCanvas.requestPaint();
                            return;
                        }
                    }
                }

                onPositionChanged: function(mouse) {
                    if (pressed) {
                        var c = getCoords(mouse.x, mouse.y);
                        curveRoot.setNode(curveRoot.selectedIndex, c.temp, c.speed);
                    }
                }
            }
        }

        // Fine-tune Controls (Temperature & Duty Spinbox/Sliders)
        RowLayout {
            Layout.fillWidth: true
            spacing: 6

            Text {
                text: "P" + (curveRoot.selectedIndex + 1) + " Suhu:"
                font.pixelSize: 10
                color: "#8B949E"
            }

            SpinBox {
                from: 20
                to: 100
                value: curveRoot.currentNodes()[curveRoot.selectedIndex].temp
                stepSize: 1
                onValueModified: curveRoot.setNode(curveRoot.selectedIndex, value, curveRoot.currentNodes()[curveRoot.selectedIndex].speed)
            }

            Text {
                text: "Duty:"
                font.pixelSize: 10
                color: "#8B949E"
            }

            SpinBox {
                from: 0
                to: 100
                value: curveRoot.currentNodes()[curveRoot.selectedIndex].speed
                stepSize: 5
                onValueModified: curveRoot.setNode(curveRoot.selectedIndex, curveRoot.currentNodes()[curveRoot.selectedIndex].temp, value)
            }

            Text {
                text: curveRoot.currentNodes()[curveRoot.selectedIndex].temp + "°C / " + curveRoot.currentNodes()[curveRoot.selectedIndex].speed + "%"
                font.pixelSize: 10
                font.bold: true
                color: curveRoot.activeTab === 0 ? "#FFB300" : "#00E1FF"
            }
        }

        // Action Buttons: Reset & Apply
        RowLayout {
            Layout.fillWidth: true
            spacing: 8
            Layout.topMargin: 2

            Rectangle {
                width: 72
                height: 24
                radius: 3
                color: "#161C26"
                border.color: "#303A4A"
                border.width: 1

                Text {
                    anchors.centerIn: parent
                    text: "⟲ Reset"
                    font.pixelSize: 10
                    color: "#8B949E"
                }

                MouseArea {
                    anchors.fill: parent
                    cursorShape: Qt.PointingHandCursor
                    onClicked: curveRoot.resetCurve()
                }
            }

            Rectangle {
                width: 110
                height: 24
                radius: 3
                color: curveRoot.hasUnsavedChanges ? "#A0640A" : "#0E2D3C"
                border.color: curveRoot.hasUnsavedChanges ? "#FFB300" : "#00E1FF"
                border.width: 1

                Text {
                    anchors.centerIn: parent
                    text: curveRoot.hasUnsavedChanges ? "● Apply & Save" : "✓ Saved"
                    font.pixelSize: 11
                    font.bold: true
                    color: "white"
                }

                MouseArea {
                    anchors.fill: parent
                    cursorShape: Qt.PointingHandCursor
                    onClicked: curveRoot.applyCurve()
                }
            }

            Item { Layout.fillWidth: true }

            Text {
                text: curveRoot.feedbackMessage !== "" ? curveRoot.feedbackMessage : (curveRoot.hasUnsavedChanges ? "● Unsaved curve changes" : "")
                font.pixelSize: 9
                color: curveRoot.feedbackMessage.includes("✓") ? "#00E676" : "#FFB300"
            }
        }
    }
}
