import os
import gi
gi.require_version('Gtk', '4.0')
gi.require_version('Adw', '1')
from gi.repository import Gtk, Adw, GLib, Gdk

from hardware import HardwareManager

CSS_STYLE = """
window.acer-window {
    background-color: #0d1117;
    color: #e6edf3;
    font-family: 'Segoe UI', 'Inter', -apple-system, sans-serif;
}

.acer-header {
    background: linear-gradient(90deg, #161b22 0%, #0d1117 100%);
    border-bottom: 2px solid #00d2ff;
    padding: 14px 20px;
}

.brand-title {
    font-size: 19px;
    font-weight: 800;
    letter-spacing: 2px;
    color: #00d2ff;
}

.brand-subtitle {
    font-size: 12px;
    color: #8b949e;
    font-weight: 500;
}

.dashboard-card {
    background-color: #161b22;
    border: 1px solid #30363d;
    border-radius: 12px;
    padding: 18px;
    box-shadow: 0 4px 12px rgba(0, 0, 0, 0.4);
}

.card-title {
    font-size: 15px;
    font-weight: 700;
    color: #c9d1d9;
    letter-spacing: 1px;
}

.temp-badge-cpu {
    font-size: 32px;
    font-weight: 800;
    color: #58a6ff;
}

.temp-badge-gpu {
    font-size: 32px;
    font-weight: 800;
    color: #3fb950;
}

.stat-unit {
    font-size: 15px;
    color: #8b949e;
}

.mode-btn {
    border-radius: 8px;
    font-weight: 700;
    padding: 10px 16px;
    border: 1px solid #30363d;
    background-color: #21262d;
    color: #c9d1d9;
    transition: all 200ms ease;
}

.mode-btn:hover {
    background-color: #30363d;
    border-color: #58a6ff;
    color: #ffffff;
}

.mode-btn.active-quiet {
    background-color: #1f6feb;
    border-color: #58a6ff;
    color: #ffffff;
}

.mode-btn.active-balanced {
    background-color: #238636;
    border-color: #3fb950;
    color: #ffffff;
}

.mode-btn.active-perf {
    background-color: #d29922;
    border-color: #e3b341;
    color: #ffffff;
}

.mode-btn.active-turbo {
    background: linear-gradient(135deg, #f85149 0%, #da3633 100%);
    border-color: #ff7b72;
    color: #ffffff;
}
"""

class AcerControlCenterWindow(Gtk.ApplicationWindow):
    def __init__(self, *args, **kwargs):
        super().__init__(*args, **kwargs)
        self.set_title("Acer Control Center")
        self.set_default_size(840, 560)
        self.add_css_class("acer-window")

        self.hw = HardwareManager()
        self.cpu_mode_auto = True
        self.gpu_mode_auto = True

        self._load_css()
        self._build_ui()
        self._start_monitor_timer()

    def _load_css(self):
        provider = Gtk.CssProvider()
        provider.load_from_data(CSS_STYLE.encode())
        Gtk.StyleContext.add_provider_for_display(
            Gdk.Display.get_default(),
            provider,
            Gtk.STYLE_PROVIDER_PRIORITY_APPLICATION
        )

    def _build_ui(self):
        root_box = Gtk.Box(orientation=Gtk.Orientation.VERTICAL, spacing=0)
        self.set_child(root_box)

        # Header Bar
        header = Gtk.Box(orientation=Gtk.Orientation.HORIZONTAL, spacing=15)
        header.add_css_class("acer-header")
        
        title_box = Gtk.Box(orientation=Gtk.Orientation.VERTICAL, spacing=2)
        title_lbl = Gtk.Label(label="ACER CONTROL CENTER", xalign=0)
        title_lbl.add_css_class("brand-title")
        subtitle_lbl = Gtk.Label(label="Aspire 7 Series • Clevo Linux Edition", xalign=0)
        subtitle_lbl.add_css_class("brand-subtitle")
        title_box.append(title_lbl)
        title_box.append(subtitle_lbl)
        header.append(title_box)

        header_spacer = Gtk.Box(hexpand=True)
        header.append(header_spacer)

        self.status_chip = Gtk.Label(label="DAEMON ACTIVE")
        self.status_chip.add_css_class("mode-btn")
        header.append(self.status_chip)

        root_box.append(header)

        # Main Content Scrollable
        content_box = Gtk.Box(orientation=Gtk.Orientation.VERTICAL, spacing=18)
        content_box.set_margin_top(18)
        content_box.set_margin_bottom(18)
        content_box.set_margin_start(20)
        content_box.set_margin_end(20)
        root_box.append(content_box)

        # Section 1: Power & Performance Modes
        modes_card = Gtk.Box(orientation=Gtk.Orientation.VERTICAL, spacing=12)
        modes_card.add_css_class("dashboard-card")
        
        mode_header = Gtk.Label(label="POWER & THERMAL MODES", xalign=0)
        mode_header.add_css_class("card-title")
        modes_card.append(mode_header)

        btn_row = Gtk.Box(orientation=Gtk.Orientation.HORIZONTAL, spacing=12, homogeneous=True)
        self.btn_quiet = Gtk.Button(label="🌿 Quiet / Eco")
        self.btn_quiet.add_css_class("mode-btn")
        self.btn_quiet.connect("clicked", lambda b: self._set_power_mode("power-saver"))

        self.btn_balanced = Gtk.Button(label="⚖️ Balanced")
        self.btn_balanced.add_css_class("mode-btn")
        self.btn_balanced.connect("clicked", lambda b: self._set_power_mode("balanced"))

        self.btn_perf = Gtk.Button(label="⚡ Performance")
        self.btn_perf.add_css_class("mode-btn")
        self.btn_perf.connect("clicked", lambda b: self._set_power_mode("performance"))

        self.btn_turbo = Gtk.Button(label="🚀 Turbo (100% Fan)")
        self.btn_turbo.add_css_class("mode-btn")
        self.btn_turbo.connect("clicked", lambda b: self._set_turbo_mode())

        btn_row.append(self.btn_quiet)
        btn_row.append(self.btn_balanced)
        btn_row.append(self.btn_perf)
        btn_row.append(self.btn_turbo)
        modes_card.append(btn_row)
        content_box.append(modes_card)

        # Section 2: Dual Fan Speed & Hardware Monitor
        fans_grid = Gtk.Box(orientation=Gtk.Orientation.HORIZONTAL, spacing=18, homogeneous=True)

        # CPU Fan Card
        cpu_card = Gtk.Box(orientation=Gtk.Orientation.VERTICAL, spacing=10)
        cpu_card.add_css_class("dashboard-card")
        cpu_title = Gtk.Label(label="CPU THERMAL & FAN (FAN 0)", xalign=0)
        cpu_title.add_css_class("card-title")
        cpu_card.append(cpu_title)

        temp_row_cpu = Gtk.Box(orientation=Gtk.Orientation.HORIZONTAL, spacing=8)
        self.cpu_temp_lbl = Gtk.Label(label="--")
        self.cpu_temp_lbl.add_css_class("temp-badge-cpu")
        unit_lbl_cpu = Gtk.Label(label="°C")
        unit_lbl_cpu.add_css_class("stat-unit")
        self.cpu_freq_lbl = Gtk.Label(label="• -- GHz", xalign=0)
        self.cpu_freq_lbl.add_css_class("stat-unit")
        self.cpu_freq_lbl.set_margin_start(10)
        temp_row_cpu.append(self.cpu_temp_lbl)
        temp_row_cpu.append(unit_lbl_cpu)
        temp_row_cpu.append(self.cpu_freq_lbl)
        cpu_card.append(temp_row_cpu)

        cpu_switch_row = Gtk.Box(orientation=Gtk.Orientation.HORIZONTAL, spacing=8)
        self.cpu_auto_btn = Gtk.Button(label="Auto Profile")
        self.cpu_auto_btn.add_css_class("mode-btn")
        self.cpu_auto_btn.connect("clicked", lambda b: self._set_cpu_fan_auto())
        cpu_switch_row.append(self.cpu_auto_btn)
        cpu_card.append(cpu_switch_row)

        self.cpu_fan_slider = Gtk.Scale.new_with_range(Gtk.Orientation.HORIZONTAL, 20, 100, 5)
        self.cpu_fan_slider.set_value(50)
        self.cpu_fan_slider.set_hexpand(True)
        self.cpu_fan_slider.connect("value-changed", self._on_cpu_slider_changed)
        cpu_card.append(self.cpu_fan_slider)
        fans_grid.append(cpu_card)

        # GPU Fan Card
        gpu_card = Gtk.Box(orientation=Gtk.Orientation.VERTICAL, spacing=10)
        gpu_card.add_css_class("dashboard-card")
        gpu_title = Gtk.Label(label="GPU THERMAL & FAN (FAN 1)", xalign=0)
        gpu_title.add_css_class("card-title")
        gpu_card.append(gpu_title)

        temp_row_gpu = Gtk.Box(orientation=Gtk.Orientation.HORIZONTAL, spacing=8)
        self.gpu_temp_lbl = Gtk.Label(label="--")
        self.gpu_temp_lbl.add_css_class("temp-badge-gpu")
        unit_lbl_gpu = Gtk.Label(label="°C")
        unit_lbl_gpu.add_css_class("stat-unit")
        self.gpu_status_lbl = Gtk.Label(label="• RTX 3050", xalign=0)
        self.gpu_status_lbl.add_css_class("stat-unit")
        self.gpu_status_lbl.set_margin_start(10)
        temp_row_gpu.append(self.gpu_temp_lbl)
        temp_row_gpu.append(unit_lbl_gpu)
        temp_row_gpu.append(self.gpu_status_lbl)
        gpu_card.append(temp_row_gpu)

        gpu_switch_row = Gtk.Box(orientation=Gtk.Orientation.HORIZONTAL, spacing=8)
        self.gpu_auto_btn = Gtk.Button(label="Auto Profile")
        self.gpu_auto_btn.add_css_class("mode-btn")
        self.gpu_auto_btn.connect("clicked", lambda b: self._set_gpu_fan_auto())
        gpu_switch_row.append(self.gpu_auto_btn)
        gpu_card.append(gpu_switch_row)

        self.gpu_fan_slider = Gtk.Scale.new_with_range(Gtk.Orientation.HORIZONTAL, 20, 100, 5)
        self.gpu_fan_slider.set_value(50)
        self.gpu_fan_slider.set_hexpand(True)
        self.gpu_fan_slider.connect("value-changed", self._on_gpu_slider_changed)
        gpu_card.append(self.gpu_fan_slider)
        fans_grid.append(gpu_card)

        content_box.append(fans_grid)

        # Update initial power profile button visual
        self._refresh_power_buttons(self.hw.get_power_profile())

    def _start_monitor_timer(self):
        GLib.timeout_add_seconds(1, self._on_tick)
        self._on_tick()

    def _on_tick(self):
        cpu_t = self.hw.get_cpu_temp()
        gpu_t = self.hw.get_gpu_temp()
        freq = self.hw.get_cpu_freq_ghz()

        self.cpu_temp_lbl.set_text(str(cpu_t))
        self.gpu_temp_lbl.set_text(str(gpu_t))
        self.cpu_freq_lbl.set_text(f"• {freq} GHz")
        return True

    def _refresh_power_buttons(self, active: str):
        for btn in [self.btn_quiet, self.btn_balanced, self.btn_perf, self.btn_turbo]:
            btn.remove_css_class("active-quiet")
            btn.remove_css_class("active-balanced")
            btn.remove_css_class("active-perf")
            btn.remove_css_class("active-turbo")

        if active == "power-saver":
            self.btn_quiet.add_css_class("active-quiet")
        elif active == "balanced":
            self.btn_balanced.add_css_class("active-balanced")
        elif active == "performance":
            self.btn_perf.add_css_class("active-perf")
        elif active == "turbo":
            self.btn_turbo.add_css_class("active-turbo")

    def _set_power_mode(self, mode: str):
        self.hw.set_power_profile(mode)
        self.hw.set_fan_speed(0, 0)
        self.hw.set_fan_speed(1, 0)
        self._refresh_power_buttons(mode)

    def _set_turbo_mode(self):
        self.hw.set_power_profile("performance")
        self.hw.set_fan_speed(0, 100)
        self.hw.set_fan_speed(1, 100)
        self.cpu_fan_slider.set_value(100)
        self.gpu_fan_slider.set_value(100)
        self._refresh_power_buttons("turbo")

    def _set_cpu_fan_auto(self):
        self.cpu_mode_auto = True
        self.hw.set_fan_speed(0, 0)
        self.cpu_auto_btn.set_label("Auto Profile (Active)")

    def _set_gpu_fan_auto(self):
        self.gpu_mode_auto = True
        self.hw.set_fan_speed(1, 0)
        self.gpu_auto_btn.set_label("Auto Profile (Active)")

    def _on_cpu_slider_changed(self, scale):
        val = int(scale.get_value())
        self.cpu_mode_auto = False
        self.cpu_auto_btn.set_label("Set to Auto")
        self.hw.set_fan_speed(0, val)

    def _on_gpu_slider_changed(self, scale):
        val = int(scale.get_value())
        self.gpu_mode_auto = False
        self.gpu_auto_btn.set_label("Set to Auto")
        self.hw.set_fan_speed(1, val)
