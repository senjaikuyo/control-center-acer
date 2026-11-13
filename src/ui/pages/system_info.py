"""
System Hardware & Battery Monitoring Page.
"""

import gi
gi.require_version('Gtk', '4.0')
from gi.repository import Gtk, GLib
from backend.sensors import SensorManager

class SystemInfoPage(Gtk.Box):
    def __init__(self, sensors: SensorManager):
        super().__init__(orientation=Gtk.Orientation.VERTICAL, spacing=18)
        self.sensors = sensors

        self.set_margin_start(28)
        self.set_margin_end(28)
        self.set_margin_top(18)
        self.set_margin_bottom(18)

        # Header Title
        title_box = Gtk.Box(orientation=Gtk.Orientation.VERTICAL, spacing=4)
        lbl_head = Gtk.Label(label="SYSTEM HARDWARE MONITOR", xalign=0)
        lbl_head.add_css_class("window-title")
        lbl_sub = Gtk.Label(label="Real-time processor, dedicated graphics, and battery diagnostic status", xalign=0)
        lbl_sub.add_css_class("window-subtitle")
        title_box.append(lbl_head)
        title_box.append(lbl_sub)
        self.append(title_box)

        # Info Cards Grid
        grid = Gtk.Grid()
        grid.set_column_spacing(20)
        grid.set_row_spacing(20)
        grid.set_column_homogeneous(True)
        grid.set_hexpand(True)

        # 1. CPU Card
        cpu_box = Gtk.Box(orientation=Gtk.Orientation.VERTICAL, spacing=10)
        cpu_box.add_css_class("fan-card")
        c_title = Gtk.Label(label="INTEL CORE PROCESSOR", xalign=0)
        c_title.add_css_class("fan-title")
        self.lbl_cpu_freq = Gtk.Label(label="Frequency: -- GHz", xalign=0)
        self.lbl_cpu_t = Gtk.Label(label="Temperature: -- °C", xalign=0)
        cpu_box.append(c_title)
        cpu_box.append(self.lbl_cpu_freq)
        cpu_box.append(self.lbl_cpu_t)
        grid.attach(cpu_box, 0, 0, 1, 1)

        # 2. GPU Card
        gpu_box = Gtk.Box(orientation=Gtk.Orientation.VERTICAL, spacing=10)
        gpu_box.add_css_class("fan-card")
        g_title = Gtk.Label(label="NVIDIA GEFORCE DEDICATED GPU", xalign=0)
        g_title.add_css_class("fan-title")
        self.lbl_gpu_load = Gtk.Label(label="Utilization: -- %", xalign=0)
        self.lbl_gpu_t = Gtk.Label(label="Temperature: -- °C", xalign=0)
        gpu_box.append(g_title)
        gpu_box.append(self.lbl_gpu_load)
        gpu_box.append(self.lbl_gpu_t)
        grid.attach(gpu_box, 1, 0, 1, 1)

        # 3. Battery Card
        bat_box = Gtk.Box(orientation=Gtk.Orientation.VERTICAL, spacing=10)
        bat_box.add_css_class("fan-card")
        b_title = Gtk.Label(label="BATTERY PACK HEALTH & STATUS", xalign=0)
        b_title.add_css_class("fan-title")
        self.lbl_bat_cap = Gtk.Label(label="Charge Level: -- %", xalign=0)
        self.lbl_bat_stat = Gtk.Label(label="Power Source: --", xalign=0)
        bat_box.append(b_title)
        bat_box.append(self.lbl_bat_cap)
        bat_box.append(self.lbl_bat_stat)
        grid.attach(bat_box, 0, 1, 2, 1)

        self.append(grid)

        GLib.timeout_add_seconds(1, self._update_stats)
        self._update_stats()

    def _update_stats(self):
        freq = self.sensors.get_cpu_freq_ghz()
        cpu_t = self.sensors.get_cpu_temp()
        gpu_t = self.sensors.get_gpu_temp()
        gpu_util = self.sensors.get_gpu_utilization()
        bat = self.sensors.get_battery_info()

        self.lbl_cpu_freq.set_text(f"Frequency: {freq} GHz")
        self.lbl_cpu_t.set_text(f"Temperature: {cpu_t} °C")
        self.lbl_gpu_load.set_text(f"Utilization: {gpu_util} %")
        self.lbl_gpu_t.set_text(f"Temperature: {gpu_t} °C")

        if bat["present"]:
            self.lbl_bat_cap.set_text(f"Charge Level: {bat['percent']} %")
            self.lbl_bat_stat.set_text(f"Status: {bat['status']}")
        else:
            self.lbl_bat_cap.set_text("Charge Level: N/A")
            self.lbl_bat_stat.set_text("Status: AC Connected")
        return True
