"""
Fan Speed Control page with animated tachometers, auto curves, manual sliders, and turbo blast.
Directly interfaces with Tuxedo Tailor (tailord) on Clevo/Acer hardware.
"""

import gi
gi.require_version('Gtk', '4.0')
from gi.repository import Gtk, GLib
from ui.components.gauge import FanGaugeWidget
from backend.tailor import TailorBackend
from backend.sensors import SensorManager

class FanControlPage(Gtk.Box):
    def __init__(self, backend: TailorBackend, sensors: SensorManager):
        super().__init__(orientation=Gtk.Orientation.VERTICAL, spacing=18)
        self.backend = backend
        self.sensors = sensors

        self.set_margin_start(28)
        self.set_margin_end(28)
        self.set_margin_top(18)
        self.set_margin_bottom(18)

        self.is_auto = True
        self.is_turbo = False

        # Header Title
        title_box = Gtk.Box(orientation=Gtk.Orientation.VERTICAL, spacing=4)
        lbl_head = Gtk.Label(label="DUAL FAN SPEED & THERMAL CONTROL", xalign=0)
        lbl_head.add_css_class("window-title")
        lbl_sub = Gtk.Label(label="Adjust acoustic and cooling performance with real-time tachometers", xalign=0)
        lbl_sub.add_css_class("window-subtitle")
        title_box.append(lbl_head)
        title_box.append(lbl_sub)
        self.append(title_box)

        # Main Twin Gauges & Controls Grid
        main_grid = Gtk.Grid()
        main_grid.set_column_spacing(24)
        main_grid.set_column_homogeneous(True)
        main_grid.set_hexpand(True)

        # 1. CPU Fan Card
        cpu_card = Gtk.Box(orientation=Gtk.Orientation.VERTICAL, spacing=12)
        cpu_card.add_css_class("fan-card")

        cpu_title_box = Gtk.Box(orientation=Gtk.Orientation.HORIZONTAL)
        cpu_title = Gtk.Label(label="CPU THERMAL & FAN", xalign=0)
        cpu_title.add_css_class("fan-title")
        cpu_title.set_hexpand(True)
        self.lbl_cpu_temp = Gtk.Label(label="-- °C")
        self.lbl_cpu_temp.add_css_class("temp-badge")
        cpu_title_box.append(cpu_title)
        cpu_title_box.append(self.lbl_cpu_temp)
        cpu_card.append(cpu_title_box)

        self.cpu_gauge = FanGaugeWidget(title="CPU FAN", is_gpu=False)
        cpu_card.append(self.cpu_gauge)

        # CPU Fan Slider
        self.cpu_slider = Gtk.Scale.new_with_range(Gtk.Orientation.HORIZONTAL, 20, 100, 5)
        self.cpu_slider.set_value(50)
        self.cpu_slider.add_css_class("cyber-slider")
        self.cpu_slider.set_sensitive(False)
        self.cpu_slider.connect("value-changed", self._on_cpu_slider_changed)
        cpu_card.append(self.cpu_slider)

        main_grid.attach(cpu_card, 0, 0, 1, 1)

        # 2. GPU Fan Card
        gpu_card = Gtk.Box(orientation=Gtk.Orientation.VERTICAL, spacing=12)
        gpu_card.add_css_class("fan-card")

        gpu_title_box = Gtk.Box(orientation=Gtk.Orientation.HORIZONTAL)
        gpu_title = Gtk.Label(label="GPU THERMAL & FAN", xalign=0)
        gpu_title.add_css_class("fan-title")
        gpu_title.set_hexpand(True)
        self.lbl_gpu_temp = Gtk.Label(label="-- °C")
        self.lbl_gpu_temp.add_css_class("temp-badge")
        gpu_title_box.append(gpu_title)
        gpu_title_box.append(self.lbl_gpu_temp)
        gpu_card.append(gpu_title_box)

        self.gpu_gauge = FanGaugeWidget(title="GPU FAN", is_gpu=True)
        gpu_card.append(self.gpu_gauge)

        # GPU Fan Slider
        self.gpu_slider = Gtk.Scale.new_with_range(Gtk.Orientation.HORIZONTAL, 20, 100, 5)
        self.gpu_slider.set_value(50)
        self.gpu_slider.add_css_class("cyber-slider")
        self.gpu_slider.set_sensitive(False)
        self.gpu_slider.connect("value-changed", self._on_gpu_slider_changed)
        gpu_card.append(self.gpu_slider)

        main_grid.attach(gpu_card, 1, 0, 1, 1)

        self.append(main_grid)

        # Bottom Control Bar: Mode selection & Turbo
        ctrl_bar = Gtk.Box(orientation=Gtk.Orientation.HORIZONTAL, spacing=14)
        ctrl_bar.set_halign(Gtk.Align.CENTER)
        ctrl_bar.set_margin_top(6)

        self.btn_auto = Gtk.Button(label="Automatic Curve")
        self.btn_auto.add_css_class("action-btn")
        self.btn_auto.add_css_class("active")
        self.btn_auto.connect("clicked", lambda b: self._set_auto_mode())

        self.btn_manual = Gtk.Button(label="Manual Sliders")
        self.btn_manual.add_css_class("action-btn")
        self.btn_manual.connect("clicked", lambda b: self._set_manual_mode())

        self.btn_turbo = Gtk.Button(label="⚡ 1-CLICK TURBO (100%)")
        self.btn_turbo.add_css_class("turbo-btn")
        self.btn_turbo.connect("clicked", lambda b: self._toggle_turbo())

        ctrl_bar.append(self.btn_auto)
        ctrl_bar.append(self.btn_manual)
        ctrl_bar.append(self.btn_turbo)
        self.append(ctrl_bar)

        # Timer for polling sensors
        GLib.timeout_add_seconds(1, self._update_sensors)
        self._update_sensors()

    def _set_auto_mode(self):
        self.is_auto = True
        self.is_turbo = False
        self.btn_auto.add_css_class("active")
        self.btn_manual.remove_css_class("active")
        self.btn_turbo.remove_css_class("active")

        self.cpu_slider.set_sensitive(False)
        self.gpu_slider.set_sensitive(False)

        self.backend.set_all_fans_auto()
        self.cpu_gauge.set_speed(0, is_auto=True)
        self.gpu_gauge.set_speed(0, is_auto=True)

    def _set_manual_mode(self):
        self.is_auto = False
        self.is_turbo = False
        self.btn_manual.add_css_class("active")
        self.btn_auto.remove_css_class("active")
        self.btn_turbo.remove_css_class("active")

        self.cpu_slider.set_sensitive(True)
        self.gpu_slider.set_sensitive(True)

        cpu_val = int(self.cpu_slider.get_value())
        gpu_val = int(self.gpu_slider.get_value())

        self.backend.set_fan_speed(0, cpu_val)
        self.backend.set_fan_speed(1, gpu_val)
        self.cpu_gauge.set_speed(cpu_val, is_auto=False)
        self.gpu_gauge.set_speed(gpu_val, is_auto=False)

    def _toggle_turbo(self):
        self.is_turbo = not self.is_turbo
        if self.is_turbo:
            self.is_auto = False
            self.btn_turbo.add_css_class("active")
            self.btn_auto.remove_css_class("active")
            self.btn_manual.remove_css_class("active")

            self.cpu_slider.set_sensitive(False)
            self.gpu_slider.set_sensitive(False)

            self.backend.set_turbo_fans(True)
            self.cpu_gauge.set_speed(100, is_auto=False)
            self.gpu_gauge.set_speed(100, is_auto=False)
        else:
            self._set_auto_mode()

    def _on_cpu_slider_changed(self, scale):
        if not self.is_auto and not self.is_turbo:
            val = int(scale.get_value())
            self.backend.set_fan_speed(0, val)
            self.cpu_gauge.set_speed(val, is_auto=False)

    def _on_gpu_slider_changed(self, scale):
        if not self.is_auto and not self.is_turbo:
            val = int(scale.get_value())
            self.backend.set_fan_speed(1, val)
            self.gpu_gauge.set_speed(val, is_auto=False)

    def _update_sensors(self):
        cpu_t = self.sensors.get_cpu_temp()
        gpu_t = self.sensors.get_gpu_temp()

        self.lbl_cpu_temp.set_text(f"{cpu_t} °C" if cpu_t > 0 else "-- °C")
        self.lbl_gpu_temp.set_text(f"{gpu_t} °C" if gpu_t > 0 else "-- °C")
        return True
