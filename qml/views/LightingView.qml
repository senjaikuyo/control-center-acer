import QtQuick
import QtQuick.Controls
import QtQuick.Layouts

Item {
    id: lightingRoot

    property color activeColor: "#00E1FF"
    property int brightnessLevel: 2 // OFF, 1, 2, 3, 4
    property bool overrideBootEffect: false

    ColumnLayout {
        anchors.fill: parent
        anchors.margins: 18
        spacing: 12

        // Header matching Reference Image 3
        RowLayout {
            Layout.fillWidth: true
            Text {
                text: "LED Keyboard"
                font.pixelSize: 18
                font.bold: true
                color: "#F0F6FC"
            }

            Item { Layout.fillWidth: true }

            CheckBox {
                id: bootEffectCheck
                text: "Override Boot Effect"
                checked: lightingRoot.overrideBootEffect
                onCheckedChanged: lightingRoot.overrideBootEffect = checked
            }

            Rectangle {
                width: 86; height: 24; radius: 3; color: "#161C26"; border.color: "#00E1FF"; border.width: 1
                Text { anchors.centerIn: parent; text: "<< Dashboard"; font.pixelSize: 10; color: "#00E1FF" }
                MouseArea { anchors.fill: parent; cursorShape: Qt.PointingHandCursor; onClicked: root.activeTab = 0 }
            }
        }

        Rectangle { Layout.fillWidth: true; height: 1; color: "#202838" }

        // Keyboard Wireframe / Chassis Preview Area
        Rectangle {
            Layout.fillWidth: true
            Layout.preferredHeight: 180
            radius: 6
            color: "#0C0F16"
            border.color: "#202838"
            border.width: 1

            // Laptop base outline
            Rectangle {
                anchors.centerIn: parent
                width: parent.width * 0.72
                height: 140
                radius: 8
                color: "#11151E"
                border.color: Qt.rgba(lightingRoot.activeColor.r, lightingRoot.activeColor.g, lightingRoot.activeColor.b, 0.4)
                border.width: 1.5

                Column {
                    anchors.centerIn: parent
                    spacing: 4

                    // Keyboard key deck rows
                    Repeater {
                        model: 5
                        Row {
                            spacing: 3
                            anchors.horizontalCenter: parent.horizontalCenter
                            Repeater {
                                model: index === 4 ? 8 : 14
                                Rectangle {
                                    width: index === 4 ? (parent.parent.width * 0.6) / 8 : 22
                                    height: 14
                                    radius: 2
                                    color: "#161C26"
                                    border.color: Qt.rgba(lightingRoot.activeColor.r, lightingRoot.activeColor.g, lightingRoot.activeColor.b, 0.55)
                                    border.width: 1
                                }
                            }
                        }
                    }

                    // Trackpad wireframe
                    Rectangle {
                        anchors.horizontalCenter: parent.horizontalCenter
                        width: 70
                        height: 30
                        radius: 3
                        color: "transparent"
                        border.color: "#222C3C"
                        border.width: 1
                    }
                }
            }
        }

        // Horizontal Multi-Color Rainbow Spectrum Slider (Exact as in Reference 3)
        ColumnLayout {
            Layout.fillWidth: true
            spacing: 4

            Rectangle {
                Layout.fillWidth: true
                height: 18
                radius: 3
                clip: true

                // Continuous rainbow spectrum
                gradient: Gradient {
                    orientation: Gradient.Horizontal
                    GradientStop { position: 0.00; color: "#00E1FF" } // Cyan
                    GradientStop { position: 0.16; color: "#00E676" } // Green
                    GradientStop { position: 0.33; color: "#FFEB3B" } // Yellow
                    GradientStop { position: 0.50; color: "#FF9800" } // Orange
                    GradientStop { position: 0.66; color: "#F44336" } // Red
                    GradientStop { position: 0.83; color: "#E91E63" } // Pink
                    GradientStop { position: 0.92; color: "#9C27B0" } // Purple
                    GradientStop { position: 1.00; color: "#2979FF" } // Blue
                }

                MouseArea {
                    anchors.fill: parent
                    cursorShape: Qt.PointingHandCursor
                    onClicked: function(mouse) {
                        var frac = Math.max(0, Math.min(1, mouse.x / width));
                        var col = Qt.hsva(frac, 1.0, 1.0, 1.0);
                        lightingRoot.activeColor = col;
                        var r = Math.round(col.r * 255);
                        var g = Math.round(col.g * 255);
                        var b = Math.round(col.b * 255);
                        hardware.setKeyboardColor(r, g, b);
                    }
                }
            }

            // Indicator pointer under spectrum
            Row {
                Layout.alignment: Qt.AlignHCenter
                spacing: 8
                Rectangle {
                    width: 16; height: 16; radius: 3; color: lightingRoot.activeColor; border.color: "white"; border.width: 1
                }
                Text {
                    text: "Selected Color: RGB(" + Math.round(lightingRoot.activeColor.r * 255) + ", " + Math.round(lightingRoot.activeColor.g * 255) + ", " + Math.round(lightingRoot.activeColor.b * 255) + ")"
                    font.pixelSize: 10
                    color: "#8B949E"
                    anchors.verticalCenter: parent.verticalCenter
                }
            }
        }

        // Bottom Settings Grid: Left: Modes (Static / Breathe) | Right: Sleep timer & Stepped Brightness
        RowLayout {
            Layout.fillWidth: true
            spacing: 20

            // Left: Modes
            ColumnLayout {
                Layout.preferredWidth: 260
                spacing: 8

                Text { text: "Lighting Mode"; font.pixelSize: 13; font.bold: true; color: "#F0F6FC" }

                Row {
                    spacing: 12
                    Rectangle {
                        width: 70; height: 38; radius: 4
                        color: "#0E2636"; border.color: "#00E1FF"; border.width: 1
                        Column {
                            anchors.centerIn: parent
                            Text { text: "💡"; font.pixelSize: 12; anchors.horizontalCenter: parent.horizontalCenter }
                            Text { text: "Static"; font.pixelSize: 9; color: "#F0F6FC"; anchors.horizontalCenter: parent.horizontalCenter }
                        }
                    }
                    Rectangle {
                        width: 70; height: 38; radius: 4
                        color: "#121620"; border.color: "#263040"; border.width: 1
                        Column {
                            anchors.centerIn: parent
                            Text { text: "◎"; font.pixelSize: 12; color: "#8B949E"; anchors.horizontalCenter: parent.horizontalCenter }
                            Text { text: "Breathe"; font.pixelSize: 9; color: "#8B949E"; anchors.horizontalCenter: parent.horizontalCenter }
                        }
                    }
                }
            }

            Rectangle { width: 1; Layout.fillHeight: true; color: "#202838" }

            // Right: Sleep timer & Discrete Stepped Brightness (OFF, 1, 2, 3, 4)
            ColumnLayout {
                Layout.fillWidth: true
                spacing: 8

                // Sleep Timer
                RowLayout {
                    spacing: 12
                    Text { text: "⏱ Keyboard Sleep timer:"; font.pixelSize: 11; font.bold: true; color: "#F0F6FC" }
                    Text { text: "30 seconds (Default)"; font.pixelSize: 10; color: "#8B949E" }
                }

                // Brightness: Stepped Levels (OFF, 1, 2, 3, 4)
                RowLayout {
                    spacing: 12
                    Text { text: "Brightness:"; font.pixelSize: 11; font.bold: true; color: "#F0F6FC" }

                    Row {
                        spacing: 6
                        Repeater {
                            model: ["OFF", "1", "2", "3", "4"]
                            Rectangle {
                                width: 44; height: 24; radius: 3
                                color: lightingRoot.brightnessLevel === index ? "#0E2636" : "#121620"
                                border.color: lightingRoot.brightnessLevel === index ? "#00E1FF" : "#222C3C"
                                border.width: 1

                                Text {
                                    anchors.centerIn: parent
                                    text: modelData
                                    font.pixelSize: 10
                                    color: lightingRoot.brightnessLevel === index ? "#00E1FF" : "#8B949E"
                                }

                                MouseArea {
                                    anchors.fill: parent
                                    cursorShape: Qt.PointingHandCursor
                                    onClicked: {
                                        lightingRoot.brightnessLevel = index;
                                        var pct = index * 25; // 0%, 25%, 50%, 75%, 100%
                                        hardware.setKeyboardBrightness(pct);
                                    }
                                }
                            }
                        }
                    }
                }
            }
        }

        Item { Layout.fillHeight: true }
    }
}
