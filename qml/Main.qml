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

            // Left: 4 Navigation Tabs
            Row {
                spacing: 6
                Repeater {
                    model: ["ic_system_monitor", "ic_led_device", "ic_basic_setting", "ic_extra_setting"]
                    Rectangle {
                        width: 34
                        height: 30
                        radius: 4
                        color: index === 0 ? "#122A36" : "transparent"
                        border.color: index === 0 ? root.colCyan : "transparent"
                        border.width: 1

                        Text {
                            anchors.centerIn: parent
                            text: index === 0 ? "⚡" : (index === 1 ? "💡" : (index === 2 ? "🔋" : "⚙"))
                            font.pixelSize: 13
                            color: index === 0 ? root.colCyan : root.colTextMuted
                        }

                        MouseArea {
                            anchors.fill: parent
                            cursorShape: Qt.PointingHandCursor
                        }
                    }
                }
            }

            Item { Layout.fillWidth: true }

            // Center: Title & Power Profiles
            Column {
                Layout.alignment: Qt.AlignHCenter
                spacing: 4

                Text {
                    anchors.horizontalCenter: parent.horizontalCenter
                    text: "Control Center"
                    font.pixelSize: 15
                    font.bold: true
                    color: root.colTextWhite
                }

                Row {
                    anchors.horizontalCenter: parent.horizontalCenter
                    spacing: 14

                    Repeater {
                        model: [
                            { name: "Performance", color: root.colGold },
                            { name: "Entertainment", color: root.colCyan },
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

            // Right: Window Controls
            Row {
                spacing: 4
                Rectangle {
                    width: 28; height: 24; radius: 3
                    color: btnMinArea.containsMouse ? "#1C2330" : "transparent"
                    Text { anchors.centerIn: parent; text: "—"; color: root.colTextMuted; font.pixelSize: 11 }
                    MouseArea { id: btnMinArea; anchors.fill: parent; hoverEnabled: true; onClicked: root.showMinimized() }
                }
                Rectangle {
                    width: 28; height: 24; radius: 3
                    color: btnMaxArea.containsMouse ? "#1C2330" : "transparent"
                    Text { anchors.centerIn: parent; text: "□"; color: root.colTextMuted; font.pixelSize: 12 }
                    MouseArea { id: btnMaxArea; anchors.fill: parent; hoverEnabled: true; onClicked: root.visibility === Window.Maximized ? root.showNormal() : root.showMaximized() }
                }
                Rectangle {
                    width: 28; height: 24; radius: 3
                    color: btnCloseArea.containsMouse ? "#DC2626" : "transparent"
                    Text { anchors.centerIn: parent; text: "✕"; color: btnCloseArea.containsMouse ? "white" : root.colTextMuted; font.pixelSize: 11 }
                    MouseArea { id: btnCloseArea; anchors.fill: parent; hoverEnabled: true; onClicked: root.close() }
                }
            }
        }

        // ====================================================================
        // DASHBOARD 3x2 GRID
        // ====================================================================
        GridLayout {
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

                    // CPU Usage Bar
                    RowLayout {
                        Layout.fillWidth: true
                        Text { text: "< CPU usage"; font.pixelSize: 10; color: root.colTextMuted }
                        Item { Layout.fillWidth: true }
                        Text { text: hardware.cpuUsagePct + " %"; font.pixelSize: 10; color: root.colTextWhite }
                    }
                    Rectangle {
                        Layout.fillWidth: true
                        height: 9
                        radius: 2
                        color: "#141923"
                        border.color: "#202836"
                        Rectangle {
                            width: parent.width * (hardware.cpuUsagePct / 100.0)
                            height: parent.height
                            radius: 2
                            color: root.colCyan
                        }
                    }

                    // GPU Usage Bar
                    RowLayout {
                        Layout.fillWidth: true
                        Text { text: "< GPU usage"; font.pixelSize: 10; color: root.colTextMuted }
                        Item { Layout.fillWidth: true }
                        Text { text: hardware.gpuUsagePct + " %"; font.pixelSize: 10; color: root.colTextWhite }
                    }
                    Rectangle {
                        Layout.fillWidth: true
                        height: 9
                        radius: 2
                        color: "#141923"
                        border.color: "#202836"
                        Rectangle {
                            width: parent.width * (hardware.gpuUsagePct / 100.0)
                            height: parent.height
                            radius: 2
                            color: root.colCyan
                        }
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

                    // CPU Fan Bar
                    RowLayout {
                        Layout.fillWidth: true
                        Text { text: "CPU FAN"; font.pixelSize: 10; color: root.colTextMuted }
                        Item { Layout.fillWidth: true }
                        Text { text: hardware.cpuFanPct + " %"; font.pixelSize: 10; color: root.colTextWhite }
                    }
                    Rectangle {
                        Layout.fillWidth: true; height: 9; radius: 2; color: "#141923"; border.color: "#202836"
                        Rectangle { width: parent.width * (hardware.cpuFanPct / 100.0); height: parent.height; radius: 2; color: root.colCyan }
                    }

                    // GPU Fan Bar
                    RowLayout {
                        Layout.fillWidth: true
                        Text { text: "GPU FAN"; font.pixelSize: 10; color: root.colTextMuted }
                        Item { Layout.fillWidth: true }
                        Text { text: hardware.gpuFanPct + " %"; font.pixelSize: 10; color: root.colTextWhite }
                    }
                    Rectangle {
                        Layout.fillWidth: true; height: 9; radius: 2; color: "#141923"; border.color: "#202836"
                        Rectangle { width: parent.width * (hardware.gpuFanPct / 100.0); height: parent.height; radius: 2; color: root.colCyan }
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

            // ROW 2 - COL 2: PREFERENCE (8 TOGGLES)
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

                    GridLayout {
                        Layout.fillWidth: true
                        columns: 4
                        columnSpacing: 6
                        rowSpacing: 6

                        property var toggles: [
                            { label: "Win key\nlock", active: false },
                            { label: "Swap\nWin/Fn", active: false },
                            { label: "Fn key\nlock", active: false },
                            { label: "Touch\nPad", active: true },
                            { label: "Camera", active: true },
                            { label: "Airplane", active: false },
                            { label: "Caps\nLock", active: false },
                            { label: "Num\nLock", active: true }
                        ]

                        Repeater {
                            model: parent.toggles
                            Rectangle {
                                Layout.preferredWidth: 64
                                Layout.preferredHeight: 36
                                radius: 4
                                color: modelData.active ? "#0E2636" : "#131822"
                                border.color: modelData.active ? root.colCyan : "#263040"
                                border.width: 1

                                Text {
                                    anchors.centerIn: parent
                                    text: modelData.label
                                    horizontalAlignment: Text.AlignHCenter
                                    font.pixelSize: 8
                                    color: modelData.active ? "white" : root.colTextMuted
                                }

                                MouseArea {
                                    anchors.fill: parent
                                    cursorShape: Qt.PointingHandCursor
                                    onClicked: {
                                        modelData.active = !modelData.active
                                        if (modelData.label.includes("Touch")) hardware.setTouchpadEnabled(modelData.active)
                                        if (modelData.label.includes("Airplane")) hardware.setAirplaneMode(modelData.active)
                                    }
                                }
                            }
                        }
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

                    RowLayout {
                        spacing: 6
                        Repeater {
                            model: ["Standard", "Eye Care", "Night"]
                            Rectangle {
                                Layout.preferredWidth: 70
                                Layout.preferredHeight: 24
                                radius: 4
                                color: index === 0 ? "#10202C" : "#12161E"
                                border.color: index === 0 ? root.colCyan : "#263040"
                                border.width: 1

                                Text {
                                    anchors.centerIn: parent
                                    text: modelData
                                    font.pixelSize: 10
                                    color: index === 0 ? "white" : root.colTextMuted
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
    }
}
