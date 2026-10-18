import "views"
import "components"
import QtQuick
import QtQuick.Controls
import QtQuick.Layouts

ApplicationWindow {
    id: root
    visible: true
    width: 1040
    height: 680
    minimumWidth: 1000
    minimumHeight: 640
    title: "Control Center"
    color: "#0B0E14"
    property int activeTab: 0

    // Unified Design Tokens
    readonly property color colBg: "#0B0E14"
    readonly property color colCardBg: "#11151E"
    readonly property color colCardBorder: "#202838"
    readonly property color colCyan: "#00E1FF"
    readonly property color colGold: "#FFB300"
    readonly property color colGreen: "#00E676"
    readonly property color colTextWhite: "#F0F6FC"
    readonly property color colTextMuted: "#8B949E"
    readonly property color colTextDark: "#5A6473"

    ColumnLayout {
        anchors.fill: parent
        anchors.margins: 12
        spacing: 10

        // ====================================================================
        // TOP HEADER
        // ====================================================================
        RowLayout {
            Layout.fillWidth: true
            Layout.preferredHeight: 46
            spacing: 8

            // Left: 4 Navigation Tabs using Authentic OEM Icons
            Row {
                spacing: 6
                Repeater {
                    model: [
                        "qrc:/AcerControlCenter/qml/assets/ic_system_monitor.png",
                        "qrc:/AcerControlCenter/qml/assets/ic_led_device.png",
                        "qrc:/AcerControlCenter/qml/assets/ic_battery.png",
                        "qrc:/AcerControlCenter/qml/assets/ic_basic_setting.png"
                    ]
                    Rectangle {
                        width: 34
                        height: 30
                        radius: 4
                        color: root.activeTab === index ? "#122A36" : "transparent"
                        border.color: root.activeTab === index ? root.colCyan : "transparent"
                        border.width: 1

                        Image {
                            anchors.centerIn: parent
                            width: 18; height: 18
                            source: modelData
                            fillMode: Image.PreserveAspectFit
                            opacity: root.activeTab === index ? 1.0 : 0.6
                        }

                        MouseArea {
                            anchors.fill: parent
                            cursorShape: Qt.PointingHandCursor
                            onClicked: root.activeTab = index
                        }
                    }
                }
            }

            Item { Layout.fillWidth: true }

            // Center: Title & Power Profiles
            Column {
                Layout.alignment: Qt.AlignHCenter
                spacing: 4

                Row {
                    anchors.horizontalCenter: parent.horizontalCenter
                    spacing: 8
                    Image {
                        width: 18; height: 18
                        anchors.verticalCenter: parent.verticalCenter
                        source: "qrc:/AcerControlCenter/qml/assets/control_center_logo.png"
                        fillMode: Image.PreserveAspectFit
                    }
                    Text {
                        anchors.verticalCenter: parent.verticalCenter
                        text: "Control Center"
                        font.pixelSize: 15
                        font.bold: true
                        color: root.colTextWhite
                    }
                }

                Row {
                    anchors.horizontalCenter: parent.horizontalCenter
                    spacing: 14

                    Repeater {
                        model: [
                            { name: "Performance", color: root.colGold },
                            { name: "Entertainment", color: root.colCyan },
                            { name: "Power Saving", color: "#2979FF" },
                            { name: "Quiet", color: root.colGreen }
                        ]

                        Item {
                            width: 95
                            height: 22

                            Rectangle {
                                anchors.fill: parent
                                radius: 3
                                color: profileArea.containsMouse ? "#161C26" : "transparent"
                            }

                            Text {
                                anchors.centerIn: parent
                                text: modelData.name
                                font.pixelSize: 12
                                font.bold: hardware.powerMode === modelData.name
                                color: hardware.powerMode === modelData.name ? modelData.color : root.colTextMuted
                            }

                            Rectangle {
                                anchors.bottom: parent.bottom
                                anchors.horizontalCenter: parent.horizontalCenter
                                width: parent.width - 16
                                height: 2
                                radius: 1
                                color: modelData.color
                                visible: hardware.powerMode === modelData.name
                            }

                            MouseArea {
                                id: profileArea
                                anchors.fill: parent
                                hoverEnabled: true
                                cursorShape: Qt.PointingHandCursor
                                onClicked: hardware.setPowerMode(modelData.name)
                            }
                        }
                    }
                }
            }

            Item { Layout.fillWidth: true }

            // Right: Window Controls (Razor-Sharp High-DPI Vector)
            Row {
                spacing: 4
                Rectangle {
                    width: 32; height: 24; radius: 3
                    color: btnMinArea.containsMouse ? "#1C2330" : "transparent"
                    Rectangle {
                        anchors.centerIn: parent
                        width: 12; height: 1.5
                        color: btnMinArea.containsMouse ? "white" : root.colTextMuted
                    }
                    MouseArea { id: btnMinArea; anchors.fill: parent; hoverEnabled: true; onClicked: root.showMinimized() }
                }
                Rectangle {
                    width: 32; height: 24; radius: 3
                    color: btnMaxArea.containsMouse ? "#1C2330" : "transparent"
                    Rectangle {
                        anchors.centerIn: parent
                        width: 10; height: 10
                        color: "transparent"
                        border.color: btnMaxArea.containsMouse ? "white" : root.colTextMuted
                        border.width: 1.5
                        radius: 1
                    }
                    MouseArea { id: btnMaxArea; anchors.fill: parent; hoverEnabled: true; onClicked: root.visibility === Window.Maximized ? root.showNormal() : root.showMaximized() }
                }
                Rectangle {
                    width: 32; height: 24; radius: 3
                    color: btnCloseArea.containsMouse ? "#DC2626" : "transparent"
                    Canvas {
                        anchors.centerIn: parent
                        width: 12; height: 12
                        onPaint: {
                            var ctx = getContext("2d");
                            ctx.reset();
                            ctx.strokeStyle = btnCloseArea.containsMouse ? "white" : "#8B949E";
                            ctx.lineWidth = 1.5;
                            ctx.beginPath();
                            ctx.moveTo(1, 1); ctx.lineTo(11, 11);
                            ctx.moveTo(11, 1); ctx.lineTo(1, 11);
                            ctx.stroke();
                        }
                    }
                    MouseArea { id: btnCloseArea; anchors.fill: parent; hoverEnabled: true; onClicked: root.close() }
                }
            }
        }

        // ====================================================================
        // MULTI-VIEW STACK
        // ====================================================================
        LightingView {
            Layout.fillWidth: true
            Layout.fillHeight: true
            visible: root.activeTab === 1
        }

        BatteryView {
            Layout.fillWidth: true
            Layout.fillHeight: true
            visible: root.activeTab === 2
        }

        SettingsView {
            Layout.fillWidth: true
            Layout.fillHeight: true
            visible: root.activeTab === 3
        }

        GridLayout {
            visible: root.activeTab === 0
            Layout.fillWidth: true
            Layout.fillHeight: true
            columns: 3
            rows: 2
            columnSpacing: 10
            rowSpacing: 10

            // ROW 1 - COL 1: SYSTEM
            Rectangle {
                Layout.fillWidth: true
                Layout.fillHeight: true
                Layout.preferredHeight: 310
                radius: 6
                color: root.colCardBg
                border.color: root.colCardBorder
                border.width: 1

                ColumnLayout {
                    anchors.fill: parent
                    anchors.margins: 14
                    spacing: 6

                    Text {
                        text: "System"
                        font.pixelSize: 14
                        font.bold: true
                        color: root.colTextWhite
                    }

                    Text {
                        text: hardware.cpuName
                        font.pixelSize: 10
                        color: root.colTextMuted
                        elide: Text.ElideRight
                        Layout.fillWidth: true
                    }

                    Text {
                        text: hardware.gpuName
                        font.pixelSize: 10
                        color: root.colTextMuted
                        elide: Text.ElideRight
                        Layout.fillWidth: true
                    }

                    Item { height: 6 }

                    // CPU Clock & Temp
                    RowLayout {
                        Layout.fillWidth: true
                        Column {
                            Layout.fillWidth: true
                            Text { text: hardware.cpuClockMhz + " MHz"; font.pixelSize: 18; font.bold: true; color: root.colTextWhite }
                            Text { text: "CPU clock speed"; font.pixelSize: 10; color: root.colCyan }
                        }
                        Column {
                            Layout.fillWidth: true
                            Text { text: hardware.cpuTemp + " °C"; font.pixelSize: 18; font.bold: true; color: root.colTextWhite }
                            Text { text: "Temperature"; font.pixelSize: 10; color: root.colCyan }
                        }
                    }

                    Item { height: 4 }

                    // GPU Clock & Temp
                    RowLayout {
                        Layout.fillWidth: true
                        Column {
                            Layout.fillWidth: true
                            Text { text: hardware.gpuClockMhz > 0 ? hardware.gpuClockMhz + " MHz" : "OFF MHz"; font.pixelSize: 18; font.bold: true; color: root.colTextWhite }
                            Text { text: "GPU clock speed"; font.pixelSize: 10; color: root.colCyan }
                        }
                        Column {
                            Layout.fillWidth: true
                            Text { text: hardware.gpuTemp > 0 ? hardware.gpuTemp + " °C" : "OFF °C"; font.pixelSize: 18; font.bold: true; color: root.colTextWhite }
                            Text { text: "Temperature"; font.pixelSize: 10; color: root.colCyan }
                        }
                    }

                    Item { height: 6 }

                    // CPU Usage Bar (Segmented OEM Style)
                    RowLayout {
                        Layout.fillWidth: true
                        Text { text: "< CPU usage"; font.pixelSize: 10; color: root.colTextMuted }
                        Item { Layout.fillWidth: true }
                        Text { text: hardware.cpuUsagePct + " %"; font.pixelSize: 10; color: root.colTextWhite }
                    }
                    SegmentedBar {
                        Layout.fillWidth: true
                        value: hardware.cpuUsagePct
                        fillColor: root.colCyan
                    }

                    // GPU Usage Bar (Segmented OEM Style)
                    RowLayout {
                        Layout.fillWidth: true
                        Text { text: "< GPU usage"; font.pixelSize: 10; color: root.colTextMuted }
                        Item { Layout.fillWidth: true }
                        Text { text: hardware.gpuUsagePct + " %"; font.pixelSize: 10; color: root.colTextWhite }
                    }
                    SegmentedBar {
                        Layout.fillWidth: true
                        value: hardware.gpuUsagePct
                        fillColor: root.colCyan
                    }
                }
            }

            // ROW 1 - COL 2: CENTER GAUGE / CUSTOM CURVE
            Rectangle {
                Layout.fillWidth: true
                Layout.fillHeight: true
                radius: 6
                color: root.colCardBg
                border.color: root.colCardBorder
                border.width: 1

                Item {
                    anchors.fill: parent
                    anchors.margins: 14

                    TachometerGauge {
                        anchors.fill: parent
                        visible: hardware.fanMode !== "Custom"
                    }

                    CustomFanCurve {
                        anchors.fill: parent
                        visible: hardware.fanMode === "Custom"
                    }
                }
            }

            // ROW 1 - COL 3: FAN SPEED CONTROL
            Rectangle {
                Layout.fillWidth: true
                Layout.fillHeight: true
                radius: 6
                color: root.colCardBg
                border.color: root.colCardBorder
                border.width: 1

                ColumnLayout {
                    anchors.fill: parent
                    anchors.margins: 14
                    spacing: 4

                    Text {
                        text: "FAN speed control"
                        font.pixelSize: 14
                        font.bold: true
                        color: root.colTextWhite
                    }

                    Item { height: 2 }

                    // Radio Modes
                    Repeater {
                        model: ["Automatic", "Maximum", "Custom", "Turbo"]
                        RowLayout {
                            Layout.fillWidth: true
                            spacing: 8

                            Rectangle {
                                width: 14; height: 14; radius: 7
                                color: "transparent"
                                border.color: hardware.fanMode === modelData ? root.colGold : "#2D3646"
                                border.width: 1.5

                                Rectangle {
                                    anchors.centerIn: parent
                                    width: 7; height: 7; radius: 3.5
                                    color: root.colGold
                                    visible: hardware.fanMode === modelData
                                }

                                MouseArea {
                                    anchors.fill: parent
                                    cursorShape: Qt.PointingHandCursor
                                    onClicked: hardware.setFanMode(modelData)
                                }
                            }

                            Text {
                                text: modelData
                                font.pixelSize: 12
                                color: hardware.fanMode === modelData ? root.colGold : root.colTextMuted
                                MouseArea {
                                    anchors.fill: parent
                                    cursorShape: Qt.PointingHandCursor
                                    onClicked: hardware.setFanMode(modelData)
                                }
                            }

                            Item { Layout.fillWidth: true }

                            // Action button next to Custom
                            Rectangle {
                                visible: modelData === "Custom"
                                width: 85; height: 20; radius: 3
                                color: hardware.fanMode === "Custom" ? "#0E2636" : "#201C0E"
                                border.color: hardware.fanMode === "Custom" ? root.colCyan : root.colGold
                                border.width: 1

                                Text {
                                    anchors.centerIn: parent
                                    text: hardware.fanMode === "Custom" ? "Tutup <<" : "Atur Kurva >>"
                                    font.pixelSize: 10
                                    color: hardware.fanMode === "Custom" ? root.colCyan : root.colGold
                                }

                                MouseArea {
                                    anchors.fill: parent
                                    cursorShape: Qt.PointingHandCursor
                                    onClicked: hardware.setFanMode(hardware.fanMode === "Custom" ? "Automatic" : "Custom")
                                }
                            }
                        }
                    }

                    Rectangle { Layout.fillWidth: true; height: 1; color: "#222A3A"; Layout.topMargin: 4; Layout.bottomMargin: 4 }

                    // Dual RPM Readouts
                    RowLayout {
                        Layout.fillWidth: true
                        Column {
                            Layout.fillWidth: true
                            Text { text: hardware.cpuFanRpm + " RPM"; font.pixelSize: 17; font.bold: true; color: root.colTextWhite }
                            Text { text: "CPU fan"; font.pixelSize: 9; color: root.colCyan }
                        }
                        Column {
                            Layout.fillWidth: true
                            Text { text: hardware.gpuFanRpm + " RPM"; font.pixelSize: 17; font.bold: true; color: root.colTextWhite }
                            Text { text: "GPU fan"; font.pixelSize: 9; color: root.colCyan }
                        }
                    }

                    Item { height: 4 }

                    // CPU Fan Bar (Segmented OEM Style)
                    RowLayout {
                        Layout.fillWidth: true
                        Text { text: "CPU FAN"; font.pixelSize: 10; color: root.colTextMuted }
                        Item { Layout.fillWidth: true }
                        Text { text: hardware.cpuFanPct + " %"; font.pixelSize: 10; color: root.colTextWhite }
                    }
                    SegmentedBar {
                        Layout.fillWidth: true
                        value: hardware.cpuFanPct
                        fillColor: root.colCyan
                    }

                    // GPU Fan Bar (Segmented OEM Style)
                    RowLayout {
                        Layout.fillWidth: true
                        Text { text: "GPU FAN"; font.pixelSize: 10; color: root.colTextMuted }
                        Item { Layout.fillWidth: true }
                        Text { text: hardware.gpuFanPct + " %"; font.pixelSize: 10; color: root.colTextWhite }
                    }
                    SegmentedBar {
                        Layout.fillWidth: true
                        value: hardware.gpuFanPct
                        fillColor: root.colCyan
                    }
                }
            }

            // ROW 2 - COL 1: MEMORY & STORAGE
            Rectangle {
                Layout.fillWidth: true
                Layout.fillHeight: true
                Layout.preferredHeight: 180
                radius: 6
                color: root.colCardBg
                border.color: root.colCardBorder
                border.width: 1

                ColumnLayout {
                    anchors.fill: parent
                    anchors.margins: 14

                    Text { text: "Memory and storage"; font.pixelSize: 13; font.bold: true; color: root.colTextWhite }

                    RowLayout {
                        Layout.fillWidth: true
                        Layout.fillHeight: true

                        // Memory
                        Column {
                            Layout.fillWidth: true
                            Layout.alignment: Qt.AlignHCenter
                            spacing: 4
                            Rectangle {
                                anchors.horizontalCenter: parent.horizontalCenter
                                width: 56; height: 56; radius: 28; color: "#121620"; border.color: root.colCyan; border.width: 2
                                Text { anchors.centerIn: parent; text: hardware.memPct + "%"; font.pixelSize: 11; font.bold: true; color: root.colTextWhite }
                            }
                            Text { anchors.horizontalCenter: parent.horizontalCenter; text: hardware.memTotalGb + "GB"; font.pixelSize: 8; color: root.colTextMuted }
                            Text { anchors.horizontalCenter: parent.horizontalCenter; text: "Memory"; font.pixelSize: 10; color: root.colTextMuted }
                        }

                        // Storage
                        Column {
                            Layout.fillWidth: true
                            Layout.alignment: Qt.AlignHCenter
                            spacing: 4
                            Rectangle {
                                anchors.horizontalCenter: parent.horizontalCenter
                                width: 56; height: 56; radius: 28; color: "#121620"; border.color: root.colCyan; border.width: 2
                                Text { anchors.centerIn: parent; text: hardware.diskPct + "%"; font.pixelSize: 11; font.bold: true; color: root.colTextWhite }
                            }
                            Text { anchors.horizontalCenter: parent.horizontalCenter; text: hardware.diskTotalGb + "GB"; font.pixelSize: 8; color: root.colTextMuted }
                            Text { anchors.horizontalCenter: parent.horizontalCenter; text: "Storage"; font.pixelSize: 10; color: root.colTextMuted }
                        }
                    }
                }
            }

            // ROW 2 - COL 2: PREFERENCE (9 TOGGLES)
            Rectangle {
                Layout.fillWidth: true
                Layout.fillHeight: true
                radius: 6
                color: root.colCardBg
                border.color: root.colCardBorder
                border.width: 1

                ColumnLayout {
                    anchors.fill: parent
                    anchors.margins: 14

                    Text { text: "Preference"; font.pixelSize: 13; font.bold: true; color: root.colTextWhite }

                    PreferenceGrid {
                        Layout.fillWidth: true
                        Layout.fillHeight: true
                    }
                }
            }

            // ROW 2 - COL 3: CONFIGURATION
            Rectangle {
                Layout.fillWidth: true
                Layout.fillHeight: true
                radius: 6
                color: root.colCardBg
                border.color: root.colCardBorder
                border.width: 1

                ColumnLayout {
                    anchors.fill: parent
                    anchors.margins: 14
                    spacing: 8

                    Text { text: "Configuration"; font.pixelSize: 13; font.bold: true; color: root.colTextWhite }

                    RowLayout {
                        spacing: 8
                        CheckBox {
                            id: blueLightCheck
                            text: "Blue light filter"
                            checked: false
                            onCheckedChanged: hardware.setNightLight(checked, 5.0)
                        }
                    }

                    // Segmented slider bar as seen in reference
                    Rectangle {
                        Layout.fillWidth: true
                        height: 12
                        radius: 2
                        color: "#121620"
                        border.color: "#222C3C"
                        border.width: 1

                        Row {
                            anchors.fill: parent
                            anchors.margins: 2
                            spacing: 3
                            Repeater {
                                model: 16
                                Rectangle {
                                    width: (parent.width - 15 * 3) / 16
                                    height: parent.height
                                    radius: 1
                                    color: (blueLightCheck.checked && index < 6) ? root.colCyan : "#1E2634"
                                }
                            }
                        }
                    }

                    Item { height: 2 }

                    // 3 Mode Profile Icons: Contrast/Standard, Brightness/Sun, Sound/Speaker
                    RowLayout {
                        Layout.fillWidth: true
                        spacing: 8

                        Repeater {
                            model: [
                                { label: "Standard", icon: "◐" },
                                { label: "Eye Care", icon: "☼" },
                                { label: "Night", icon: "♫" }
                            ]
                            Rectangle {
                                Layout.preferredWidth: 70
                                Layout.preferredHeight: 28
                                radius: 4
                                color: (blueLightCheck.checked ? (index === 1 ? "#102838" : "#12161E") : (index === 0 ? "#102838" : "#12161E"))
                                border.color: (blueLightCheck.checked ? (index === 1 ? root.colCyan : "#263040") : (index === 0 ? root.colCyan : "#263040"))
                                border.width: 1

                                Row {
                                    anchors.centerIn: parent
                                    spacing: 4
                                    Text {
                                        text: modelData.icon
                                        font.pixelSize: 11
                                        color: (blueLightCheck.checked ? (index === 1 ? root.colCyan : root.colTextMuted) : (index === 0 ? root.colCyan : root.colTextMuted))
                                    }
                                    Text {
                                        text: modelData.label
                                        font.pixelSize: 9
                                        color: (blueLightCheck.checked ? (index === 1 ? "white" : root.colTextMuted) : (index === 0 ? "white" : root.colTextMuted))
                                    }
                                }

                                MouseArea {
                                    anchors.fill: parent
                                    cursorShape: Qt.PointingHandCursor
                                    onClicked: {
                                        blueLightCheck.checked = (index !== 0)
                                        hardware.setNightLight(index !== 0, index === 1 ? 4.0 : 8.0)
                                    }
                                }
                            }
                        }
                    }
                }
            }
        }

        // ====================================================================
        // FOOTER CAROUSEL BAR (EXACT AS IN OEM REFERENCE)
        // ====================================================================
        RowLayout {
            Layout.fillWidth: true
            Layout.preferredHeight: 18
            spacing: 6
            visible: root.activeTab === 0

            Text {
                text: "<"
                font.pixelSize: 12
                font.bold: true
                color: root.colTextMuted
                MouseArea {
                    anchors.fill: parent
                    cursorShape: Qt.PointingHandCursor
                    onClicked: hardware.setFanMode("Automatic")
                }
            }

            Rectangle {
                Layout.fillWidth: true
                height: 4
                radius: 2
                color: "#161C26"

                Rectangle {
                    anchors.verticalCenter: parent.verticalCenter
                    width: parent.width * 0.4
                    height: 4
                    radius: 2
                    color: "#2C384C"
                }
            }

            Text {
                text: ">"
                font.pixelSize: 12
                font.bold: true
                color: root.colTextMuted
                MouseArea {
                    anchors.fill: parent
                    cursorShape: Qt.PointingHandCursor
                    onClicked: hardware.setFanMode("Custom")
                }
            }
        }
    }
}
