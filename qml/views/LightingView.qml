import QtQuick
import QtQuick.Controls
import QtQuick.Layouts

Item {
    id: lightingRoot

    property color activeColor: "#00E1FF"
    property int brightness: 80

    ColumnLayout {
        anchors.fill: parent
        anchors.margins: 20
        spacing: 16

        // Header
        RowLayout {
            Layout.fillWidth: true
            Text {
                text: "Chassis & Keyboard Lighting"
                font.pixelSize: 18
                font.bold: true
                color: "#F0F6FC"
            }
            Item { Layout.fillWidth: true }
            Rectangle {
                width: 100; height: 26; radius: 3; color: "#161C26"; border.color: "#00E1FF"; border.width: 1
                Text { anchors.centerIn: parent; text: "<< Dashboard"; font.pixelSize: 11; color: "#00E1FF" }
                MouseArea { anchors.fill: parent; cursorShape: Qt.PointingHandCursor; onClicked: root.activeTab = 0 }
            }
        }

        Rectangle { Layout.fillWidth: true; height: 1; color: "#202838" }

        // Keyboard Visualizer Mock Frame
        Rectangle {
            Layout.fillWidth: true
            Layout.preferredHeight: 140
            radius: 6
            color: "#0C0F16"
            border.color: lightingRoot.activeColor
            border.width: 2

            Column {
                anchors.centerIn: parent
                spacing: 8

                // Key rows
                Repeater {
                    model: 4
                    Row {
                        spacing: 4
                        anchors.horizontalCenter: parent.horizontalCenter
                        Repeater {
                            model: 14
                            Rectangle {
                                width: 28; height: 16; radius: 2
                                color: "#161C26"
                                border.color: Qt.rgba(lightingRoot.activeColor.r, lightingRoot.activeColor.g, lightingRoot.activeColor.b, 0.45)
                                border.width: 1
                            }
                        }
                    }
                }

                Text {
                    anchors.horizontalCenter: parent.horizontalCenter
                    text: "Keyboard RGB LED · Brightness " + lightingRoot.brightness + "%"
                    font.pixelSize: 11
                    color: lightingRoot.activeColor
                }
            }
        }

        // Color Presets
        Text {
            text: "Color Presets"
            font.pixelSize: 13
            font.bold: true
            color: "#F0F6FC"
        }

        Row {
            spacing: 8
            Repeater {
                model: [
                    { name: "Cyan", col: "#00E1FF", r: 0, g: 225, b: 255 },
                    { name: "Blue", col: "#2979FF", r: 41, g: 121, b: 255 },
                    { name: "Purple", col: "#9C27B0", r: 156, g: 39, b: 176 },
                    { name: "Red", col: "#F44336", r: 244, g: 67, b: 54 },
                    { name: "Gold", col: "#FFB300", r: 255, g: 179, b: 0 },
                    { name: "Green", col: "#00E676", r: 0, g: 230, b: 118 },
                    { name: "White", col: "#FFFFFF", r: 255, g: 255, b: 255 },
                    { name: "Off", col: "#121620", r: 0, g: 0, b: 0 }
                ]

                Rectangle {
                    width: 52; height: 32; radius: 4
                    color: modelData.col
                    border.color: lightingRoot.activeColor === modelData.col ? "white" : "#303A4C"
                    border.width: lightingRoot.activeColor === modelData.col ? 2 : 1

                    MouseArea {
                        anchors.fill: parent
                        cursorShape: Qt.PointingHandCursor
                        onClicked: {
                            lightingRoot.activeColor = modelData.col;
                            hardware.setKeyboardColor(modelData.r, modelData.g, modelData.b);
                        }
                    }
                }
            }
        }

        // Brightness Slider
        RowLayout {
            Layout.fillWidth: true
            spacing: 12

            Text { text: "Brightness:"; font.pixelSize: 12; color: "#F0F6FC" }

            Slider {
                Layout.fillWidth: true
                from: 0
                to: 100
                value: lightingRoot.brightness
                onMoved: {
                    lightingRoot.brightness = Math.round(value);
                    hardware.setKeyboardBrightness(lightingRoot.brightness);
                }
            }

            Text { text: lightingRoot.brightness + "%"; font.pixelSize: 12; color: "#00E1FF" }
        }

        Item { Layout.fillHeight: true }
    }
}
