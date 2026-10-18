#include <QGuiApplication>
#include <QQmlApplicationEngine>
#include <QQmlContext>
#include <cstdio>
#include "hardware_bridge.h"

int main(int argc, char *argv[]) {
    fprintf(stderr, ">>> Starting Acer Control Center Qt6...\n");
    QGuiApplication app(argc, argv);
    app.setApplicationName("Control Center");
    app.setOrganizationName("Clevo");
    app.setOrganizationDomain("clevo.com");

    HardwareBridge bridge;
    fprintf(stderr, ">>> HardwareBridge initialized\n");

    QQmlApplicationEngine engine;
    engine.rootContext()->setContextProperty("hardware", &bridge);

    QObject::connect(&engine, &QQmlApplicationEngine::warnings, [](const QList<QQmlError> &warnings) {
        for (const auto &w : warnings) {
            fprintf(stderr, ">>> QML Warning: %s\n", qPrintable(w.toString()));
        }
    });

    QObject::connect(
        &engine,
        &QQmlApplicationEngine::objectCreationFailed,
        &app,
        [](const QUrl &url) {
            fprintf(stderr, ">>> CRITICAL: Failed to create object from URL: %s\n", qPrintable(url.toString()));
            QCoreApplication::exit(-1);
        },
        Qt::QueuedConnection);

    const QUrl url(QStringLiteral("qrc:/AcerControlCenter/qml/Main.qml"));
    fprintf(stderr, ">>> Loading URL: %s\n", qPrintable(url.toString()));
    engine.load(url);

    return app.exec();
}
