import QtQuick
import QtQuick.Controls
import QtQuick.Layouts

Item {
    id: batteryRoot

    property int chargeLimit: 80

    ColumnLayout {
        anchors.fill: parent
        anchors.margins: 20
        spacing: 16

        // Header
        RowLayout {
            Layout.fillWidth: true
            Text {
                text: "Battery & Power Health"
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

        // Battery Level Circular Indicator
        RowLayout {
            Layout.fillWidth: true
            spacing: 24

            Rectangle {
                width: 100; height: 100; radius: 50; color: "#11151E"; border.color: "#00E676"; border.width: 3
                Column {
                    anchors.centerIn: parent
                    Text { anchors.horizontalCenter: parent.horizontalCenter; text: "82%"; font.pixelSize: 22; font.bold: true; color: "#F0F6FC" }
                    Text { anchors.horizontalCenter: parent.horizontalCenter; text: "BAT0"; font.pixelSize: 10; color: "#00E676" }
                }
            }

            ColumnLayout {
                spacing: 6
                Text { text: "Battery Health Preservation"; font.pixelSize: 15; font.bold: true; color: "#F0F6FC" }
                Text {
                    text: "Protect battery lifespan by limiting the charge capacity threshold. Ideal for AC-tethered usage."
                    font.pixelSize: 11
                    color: "#8B949E"
                }

                Row {
                    spacing: 8
                    Layout.topMargin: 8

                    Rectangle {
                        width: 160; height: 28; radius: 4
                        color: batteryRoot.chargeLimit === 80 ? "#0E2636" : "#131822"
                        border.color: batteryRoot.chargeLimit === 80 ? "#00E1FF" : "#263040"
                        border.width: 1
                        Text { anchors.centerIn: parent; text: "✓ 80% Health Preservation"; font.pixelSize: 10; color: batteryRoot.chargeLimit === 80 ? "white" : "#8B949E" }
                        MouseArea { anchors.fill: parent; cursorShape: Qt.PointingHandCursor; onClicked: batteryRoot.chargeLimit = 80 }
                    }

                    Rectangle {
                        width: 140; height: 28; radius: 4
                        color: batteryRoot.chargeLimit === 100 ? "#0E2636" : "#131822"
                        border.color: batteryRoot.chargeLimit === 100 ? "#00E1FF" : "#263040"
                        border.width: 1
                        Text { anchors.centerIn: parent; text: "100% Full Capacity"; font.pixelSize: 10; color: batteryRoot.chargeLimit === 100 ? "white" : "#8B949E" }
                        MouseArea { anchors.fill: parent; cursorShape: Qt.PointingHandCursor; onClicked: batteryRoot.chargeLimit = 100 }
                    }
                }
            }
        }

        Item { Layout.fillHeight: true }
    }
}
