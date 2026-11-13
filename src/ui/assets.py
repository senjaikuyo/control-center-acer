"""
Asset loader and resolver for genuine Acer / Clevo Control Center 3.0 graphics.
"""

import os

PROJECT_ROOT = os.path.abspath(os.path.join(os.path.dirname(__file__), "..", ".."))
ASSETS_DIR = os.path.join(PROJECT_ROOT, "assets", "ui")

CC_IMG_DIR = os.path.join(ASSETS_DIR, "control_center", "image")
FAN_IMG_DIR = os.path.join(ASSETS_DIR, "fan_speed", "image")

class Assets:
    # Main Window & Frame
    BG_MAIN_LAYER1 = os.path.join(CC_IMG_DIR, "bg_main_layer1.png")
    BG_MAIN_LAYER2 = os.path.join(CC_IMG_DIR, "bg_main_layer2.png")
    APP_LOGO = os.path.join(CC_IMG_DIR, "control_center_logo.png")
    TITLE_TEXT = os.path.join(CC_IMG_DIR, "title.png")

    # Window Controls
    IC_CLOSE = os.path.join(CC_IMG_DIR, "ic_close.png")
    IC_CLOSE_HOVER = os.path.join(CC_IMG_DIR, "btn_close_hover.png")
    IC_CLOSE_PRESSED = os.path.join(CC_IMG_DIR, "btn_close_pressed.png")
    IC_MINIMIZE = os.path.join(CC_IMG_DIR, "ic_minimize.png")

    # Navigation Tabs Icons
    IC_TAB_MODES = os.path.join(CC_IMG_DIR, "setting", "ic_basic_setting.png")
    IC_TAB_FAN = os.path.join(CC_IMG_DIR, "ic_fan.png")
    IC_TAB_MONITOR = os.path.join(FAN_IMG_DIR, "main", "ic_system_monitor.png")
    IC_TAB_LED = os.path.join(CC_IMG_DIR, "ic_led.png")

    # 4 Power / Performance Modes (Cards & Badges)
    BTN_QUIET_SEL1 = os.path.join(CC_IMG_DIR, "btn_mode_quiet_layer1_selected_1.png")
    BTN_QUIET_SEL2 = os.path.join(CC_IMG_DIR, "btn_mode_quiet_layer1_selected_2.png")
    IC_QUIET = os.path.join(CC_IMG_DIR, "ic_quite.png")

    BTN_POWER_SAVING_SEL1 = os.path.join(CC_IMG_DIR, "btn_mode_power_saving_layer1_selected_1.png")
    BTN_POWER_SAVING_SEL2 = os.path.join(CC_IMG_DIR, "btn_mode_power_saving_layer1_selected_2.png")
    IC_POWER_SAVING = os.path.join(CC_IMG_DIR, "ic_power_saving.png")

    BTN_ENTERTAINMENT_SEL1 = os.path.join(CC_IMG_DIR, "btn_mode_entertainment_layer1_selected_1.png")
    BTN_ENTERTAINMENT_SEL2 = os.path.join(CC_IMG_DIR, "btn_mode_entertainment_layer1_selected_2.png")
    IC_ENTERTAINMENT = os.path.join(CC_IMG_DIR, "ic_entertainment.png")

    BTN_PERFORMANCE_SEL1 = os.path.join(CC_IMG_DIR, "btn_mode_performance_layer1_selected_1.png")
    BTN_PERFORMANCE_SEL2 = os.path.join(CC_IMG_DIR, "btn_mode_performance_layer1_selected_2.png")
    IC_PERFORMANCE = os.path.join(CC_IMG_DIR, "ic_performance.png")

    # Fan Gauges & Setting Graphics
    FAN_BLADES = os.path.join(FAN_IMG_DIR, "page_systemmonitor", "fan", "system_fan_layer1_1.png")
    FAN_OUTER_RING = os.path.join(FAN_IMG_DIR, "page_systemmonitor", "fan", "system_fan_layer2_1.png")
    FAN_GAUGE_SCALE = os.path.join(FAN_IMG_DIR, "page_systemmonitor", "fan", "system_fan_layer3_1.png")
    FAN_BG_AUTO = os.path.join(FAN_IMG_DIR, "page_systemmonitor", "fan", "bg_system_fan_setting_auto.png")
    FAN_BG_CUSTOM = os.path.join(FAN_IMG_DIR, "page_systemmonitor", "fan", "bg_system_fan_setting_custom.png")
    IC_FAN_SETTING = os.path.join(FAN_IMG_DIR, "page_systemmonitor", "fan", "ic_fan_setting.png")

    # Hardware Icons
    IC_CPU = os.path.join(CC_IMG_DIR, "ic_cpu.png")
    IC_GPU = os.path.join(CC_IMG_DIR, "ic_gpu.png")

    @classmethod
    def verify(cls):
        """Check all essential assets exist"""
        for k, v in cls.__dict__.items():
            if k.isupper() and isinstance(v, str):
                if not os.path.exists(v):
                    print(f"[Warning] Asset not found: {k} -> {v}")
