import QtQuick
import QtQuick.Controls
import QtQuick.Layouts

Item {
    id: batteryRoot

    property int chargeMode: 2 // 0: Maximum (100%), 1: Recommended (70-80%), 2: Custom
    property int startThreshold: 70
    property int stopThreshold: 80

    ColumnLayout {
        anchors.fill: parent
        anchors.margins: 18
        spacing: 14

        // Header matching Reference Image 4
        RowLayout {
            Layout.fillWidth: true
            Text {
                text: "FlexiCharger"
                font.pixelSize: 18
                font.bold: true
                color: "#F0F6FC"
            }
            Item { Layout.fillWidth: true }
            Rectangle {
                width: 86; height: 24; radius: 3; color: "#161C26"; border.color: "#00E1FF"; border.width: 1
                Text { anchors.centerIn: parent; text: "<< Dashboard"; font.pixelSize: 10; color: "#00E1FF" }
                MouseArea { anchors.fill: parent; cursorShape: Qt.PointingHandCursor; onClicked: root.activeTab = 0 }
            }
        }

        Rectangle { Layout.fillWidth: true; height: 1; color: "#202838" }

        // Main FlexiCharger Dual-Panel Layout
        RowLayout {
            Layout.fillWidth: true
            Layout.fillHeight: true
            spacing: 24

            // Left Panel: Big Neon Green Battery Silhouette & Percentage
            Rectangle {
                Layout.preferredWidth: 320
                Layout.fillHeight: true
                radius: 6
                color: "#0C0F16"
                border.color: "#202838"
                border.width: 1

                Column {
                    anchors.centerIn: parent
                    spacing: 16

                    // Glowing Battery Silhouette with Plug Graphic
                    Rectangle {
                        anchors.horizontalCenter: parent.horizontalCenter
                        width: 130
                        height: 220
                        radius: 12
                        color: "#111822"
                        border.color: "#00E676"
                        border.width: 3

                        // Terminal nipple
                        Rectangle {
                            anchors.bottom: parent.top
                            anchors.horizontalCenter: parent.horizontalCenter
                            width: 44
                            height: 10
                            radius: 3
                            color: "#00E676"
                        }

                        // Interior Plug Glyph
                        Text {
                            anchors.centerIn: parent
                            text: "🔌"
                            font.pixelSize: 36
                        }

                        // Internal Charge Level Bar
                        Rectangle {
                            anchors.bottom: parent.bottom
                            anchors.horizontalCenter: parent.horizontalCenter
                            anchors.bottomMargin: 8
                            width: parent.width - 20
                            height: 16
                            radius: 3
                            color: "#0E241E"
                            border.color: "#00E676"
                            border.width: 1

                            Rectangle {
                                width: parent.width * 0.70
                                height: parent.height
                                radius: 2
                                color: "#00E676"
                            }
                        }
                    }

                    // Charge Percentage Label
                    Text {
                        anchors.horizontalCenter: parent.horizontalCenter
                        text: "70 %"
                        font.pixelSize: 28
                        font.bold: true
                        color: "#F0F6FC"
                    }
                }
            }

            // Right Panel: 3 Battery Charge Options
            Rectangle {
                Layout.fillWidth: true
                Layout.fillHeight: true
                radius: 6
                color: "#0C0F16"
                border.color: "#202838"
                border.width: 1

                ColumnLayout {
                    anchors.fill: parent
                    anchors.margins: 20
                    spacing: 14

                    RowLayout {
                        spacing: 8
                        Text { text: "🛡"; font.pixelSize: 14 }
                        Text { text: "Battery Charge Options:"; font.pixelSize: 14; font.bold: true; color: "#F0F6FC" }
                    }

                    Rectangle { Layout.fillWidth: true; height: 1; color: "#1A2230" }

                    // Option 1: Maximum Battery Charge
                    RowLayout {
                        spacing: 10
                        RadioButton {
                            checked: batteryRoot.chargeMode === 0
                            onClicked: batteryRoot.chargeMode = 0
                        }
                        Column {
                            Text { text: "Maximum Battery Charge"; font.pixelSize: 12; font.bold: true; color: "#F0F6FC" }
                            Text { text: "Constantly charge the battery to 100%"; font.pixelSize: 10; color: "#8B949E" }
                        }
                    }

                    // Option 2: Recommended Battery Charge
                    RowLayout {
                        spacing: 10
                        RadioButton {
                            checked: batteryRoot.chargeMode === 1
                            onClicked: batteryRoot.chargeMode = 1
                        }
                        Column {
                            Text { text: "Recommended Battery Charge"; font.pixelSize: 12; font.bold: true; color: "#F0F6FC" }
                            Text { text: "Charge the battery when under 70%, stopping at 80%"; font.pixelSize: 10; color: "#8B949E" }
                        }
                    }

                    // Option 3: Custom Battery Charge (Active)
                    RowLayout {
                        spacing: 10
                        RadioButton {
                            checked: batteryRoot.chargeMode === 2
                            onClicked: batteryRoot.chargeMode = 2
                        }
                        Column {
                            Text { text: "Custom Battery Charge"; font.pixelSize: 12; font.bold: true; color: batteryRoot.chargeMode === 2 ? "#00E1FF" : "#F0F6FC" }
                            Text { text: "Set the start and stop charge threshold"; font.pixelSize: 10; color: "#8B949E" }
                        }
                    }

                    // Threshold Input Boxes
                    ColumnLayout {
                        Layout.leftMargin: 28
                        spacing: 8
                        visible: batteryRoot.chargeMode === 2

                        RowLayout {
                            spacing: 12
                            Text { text: "Start charging when capacity is under:"; font.pixelSize: 11; color: "#F0F6FC" }
                            SpinBox {
                                from: 40; to: 90; value: batteryRoot.startThreshold
                                onValueModified: batteryRoot.startThreshold = value
                            }
                            Text { text: "%"; font.pixelSize: 11; color: "#00E1FF" }
                        }

                        RowLayout {
                            spacing: 12
                            Text { text: "Stop charging when capacity reaches:"; font.pixelSize: 11; color: "#F0F6FC" }
                            SpinBox {
                                from: 50; to: 100; value: batteryRoot.stopThreshold
                                onValueModified: batteryRoot.stopThreshold = value
                            }
                            Text { text: "%"; font.pixelSize: 11; color: "#00E1FF" }
                        }
                    }

                    Item { Layout.fillHeight: true }
                }
            }
        }
    }
}
