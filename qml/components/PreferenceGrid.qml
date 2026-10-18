import QtQuick
import QtQuick.Layouts

Item {
    id: prefRoot
    implicitHeight: 120

    GridLayout {
        anchors.fill: parent
        columns: 5
        columnSpacing: 6
        rowSpacing: 6

        // 9 OEM Shortcut Toggles exactly as in Reference Image 1 & 2
        property var toggleList: [
            { id: "winLock", label: "Win key lock", icon: "田", active: false },
            { id: "swapFn", label: "Swap Win-Fn", icon: "⇄", active: false },
            { id: "fnLock", label: "Fn-key Lock", icon: "⚿", active: false },
            { id: "touchpad", label: "Touch Pad", icon: "▱", active: true },
            { id: "camera", label: "Camera", icon: "📷", active: true },
            { id: "airplane", label: "Airplane", icon: "✈", active: false },
            { id: "capsLock", label: "Caps Lock", icon: "⇪", active: false },
            { id: "numLock", label: "Num Lock", icon: "1", active: true },
            { id: "scrLock", label: "Scr Lock", icon: "⇳", active: false }
        ]

        Repeater {
            model: parent.toggleList
            Rectangle {
                Layout.preferredWidth: 54
                Layout.preferredHeight: 52
                radius: 4
                color: modelData.active ? "#0B2636" : "#121620"
                border.color: modelData.active ? "#00E1FF" : "#222C3C"
                border.width: 1

                Column {
                    anchors.centerIn: parent
                    spacing: 3

                    // Icon / Glyph
                    Text {
                        anchors.horizontalCenter: parent.horizontalCenter
                        text: modelData.icon
                        font.pixelSize: 14
                        color: modelData.active ? "#00E1FF" : "#8B949E"
                    }

                    // Label
                    Text {
                        anchors.horizontalCenter: parent.horizontalCenter
                        text: modelData.label
                        font.pixelSize: 8
                        color: modelData.active ? "#F0F6FC" : "#6E7681"
                        horizontalAlignment: Text.AlignHCenter
                    }
                }

                MouseArea {
                    anchors.fill: parent
                    cursorShape: Qt.PointingHandCursor
                    onClicked: {
                        modelData.active = !modelData.active
                        if (modelData.id === "touchpad") hardware.setTouchpadEnabled(modelData.active)
                        if (modelData.id === "airplane") hardware.setAirplaneMode(modelData.active)
                    }
                }
            }
        }
    }
}
