cmake_minimum_required(VERSION 3.16)
project(acer-control-center LANGUAGES CXX)

set(CMAKE_CXX_STANDARD 17)
set(CMAKE_CXX_STANDARD_REQUIRED ON)

find_package(Qt6 6.5 REQUIRED COMPONENTS Core Gui Quick DBus)

qt_standard_project_setup()

qt_add_executable(acer-control-center
    src/main.cpp
    src/hardware_bridge.h
    src/hardware_bridge.cpp
)

qt_add_qml_module(acer-control-center
    URI AcerControlCenter
    VERSION 1.0
    QML_FILES
        qml/Main.qml
        qml/components/TachometerGauge.qml
        qml/components/CustomFanCurve.qml
        qml/components/SegmentedBar.qml
        qml/components/PreferenceGrid.qml
        qml/views/LightingView.qml
        qml/views/BatteryView.qml
        qml/views/SettingsView.qml
    RESOURCES
        qml/assets/ic_system_monitor.png
        qml/assets/ic_led_device.png
        qml/assets/ic_battery.png
        qml/assets/ic_basic_setting.png
        qml/assets/ic_minimize.png
        qml/assets/ic_fullscreen.png
        qml/assets/ic_close.png
        qml/assets/control_center_logo.png
        qml/assets/temp_dial_bg.png
        qml/assets/temp_dial_arc.png
        qml/assets/temp_needle1.png
        qml/assets/temp_needle2.png
        qml/assets/knob_fan_control.png
        qml/assets/fan_bg_custom.png
        qml/assets/fan_bg_auto.png
        qml/assets/card_frame_hud.png
        qml/assets/btn_save_normal.png
        qml/assets/btn_save_hover.png
        qml/assets/btn_default_normal.png
        qml/assets/btn_default_hover.png
        qml/assets/btn_fan_cpu_selected.png
        qml/assets/btn_fan_cpu_normal.png
        qml/assets/btn_fan_gpu_selected.png
        qml/assets/btn_fan_gpu_normal.png
)

target_link_libraries(acer-control-center
    PRIVATE
        Qt6::Core
        Qt6::Gui
        Qt6::Quick
        Qt6::DBus
)

install(TARGETS acer-control-center DESTINATION bin)
