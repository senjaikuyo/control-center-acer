import QtQuick
import QtQuick.Controls
import QtQuick.Layouts

Item {
    id: settingsRoot

    ColumnLayout {
        anchors.fill: parent
        anchors.margins: 20
        spacing: 16

        // Header
        RowLayout {
            Layout.fillWidth: true
            Text {
                text: "Device Settings & Diagnostics"
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

        Text {
            text: "Active Linux Subsystems Status:"
            font.pixelSize: 14
            font.bold: true
            color: "#F0F6FC"
        }

        Repeater {
            model: [
                { title: "Cooling Subsystem", desc: "com.tux.Tailor (tailord.service) · EC PWM Active" },
                { title: "Keyboard Driver", desc: "tuxedo_keyboard (rgb:kbd_backlight sysfs) · Active" },
                { title: "Power Management", desc: "net.hadess.PowerProfiles (powerprofilesctl) · Active" },
                { title: "Motherboard ODM", desc: "Clevo CLV0001:00 WMI · ACPI Persistent Drivers Active" }
            ]

            RowLayout {
                spacing: 12
                Rectangle {
                    width: 8; height: 8; radius: 4; color: "#00E676"
                }
                Text {
                    text: modelData.title + ":"
                    font.pixelSize: 12
                    font.bold: true
                    color: "#F0F6FC"
                }
                Text {
                    text: modelData.desc
                    font.pixelSize: 12
                    color: "#8B949E"
                }
            }
        }

        Item { Layout.fillHeight: true }
    }
}
