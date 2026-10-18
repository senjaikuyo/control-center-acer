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
    RESOURCES
        extracted_assets/fanspeed/image/style_1/pointer_temp.png
        extracted_assets/fanspeed/image/style_1/knob_fan_control.png
        extracted_assets/fanspeed/image/style_1/knob_fan_custom_normal.png
        extracted_assets/fanspeed/image/page_systemmonitor/temp/system_temperature_layer1_1.png
        extracted_assets/fanspeed/image/page_systemmonitor/temp/system_temperature_layer2_2.png
        extracted_assets/fanspeed/image/page_systemmonitor/temp/system_temperature_needle_layer1_2.png
        extracted_assets/fanspeed/image/page_systemmonitor/temp/system_temperature_needle_layer2_2.png
)

target_link_libraries(acer-control-center
    PRIVATE
        Qt6::Core
        Qt6::Gui
        Qt6::Quick
        Qt6::DBus
)

install(TARGETS acer-control-center DESTINATION bin)
