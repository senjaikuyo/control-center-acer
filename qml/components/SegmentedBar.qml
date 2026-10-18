import QtQuick

Item {
    id: barRoot

    property int value: 0 // 0 to 100 percent
    property color fillColor: "#00E1FF"
    property int numSegments: 26

    implicitWidth: 200
    implicitHeight: 10

    Canvas {
        id: barCanvas
        anchors.fill: parent
        antialiasing: true

        onPaint: {
            var ctx = getContext("2d");
            ctx.reset();

            var w = width;
            var h = height;
            var gap = 2.0;
            var slant = 3.0; // pixels of diagonal shear
            var n = barRoot.numSegments;
            var segW = (w - (n - 1) * gap) / n;
            var filledCount = Math.round(n * (Math.min(Math.max(barRoot.value, 0), 100) / 100.0));

            for (var i = 0; i < n; ++i) {
                var x = i * (segW + gap);
                var isFilled = (i < filledCount);
                ctx.fillStyle = isFilled ? barRoot.fillColor : "#1E2634";

                ctx.beginPath();
                ctx.moveTo(x + slant, 0);
                ctx.lineTo(x + segW + slant, 0);
                ctx.lineTo(x + segW, h);
                ctx.lineTo(x, h);
                ctx.closePath();
                ctx.fill();
            }
        }
    }

    onValueChanged: barCanvas.requestPaint()
    onFillColorChanged: barCanvas.requestPaint()
    onWidthChanged: barCanvas.requestPaint()
    onHeightChanged: barCanvas.requestPaint()
}
