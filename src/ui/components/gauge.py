"""
Custom Animated Fan Gauge Widget for GTK4.
Renders real Clevo/Acer circular fan tachometer with rotating blades and glowing arc.
"""

import math
import gi
gi.require_version('Gtk', '4.0')
from gi.repository import Gtk, Gdk, GdkPixbuf, GLib
from ui.assets import Assets

class FanGaugeWidget(Gtk.Box):
    def __init__(self, title: str = "CPU FAN", is_gpu: bool = False):
        super().__init__(orientation=Gtk.Orientation.VERTICAL, spacing=10)
        self.title_text = title
        self.is_gpu = is_gpu
        self.speed_percent = 0
        self.is_auto = True
        self.angle_rad = 0.0
        self.rpm = 0

        # Load pixbufs
        try:
            self.pb_outer = GdkPixbuf.Pixbuf.new_from_file_at_scale(Assets.FAN_OUTER_RING, 190, 190, True)
            self.pb_blades = GdkPixbuf.Pixbuf.new_from_file_at_scale(Assets.FAN_BLADES, 90, 90, True)
        except Exception as e:
            print(f"[FanGauge] Error loading pixbufs: {e}")
            self.pb_outer = None
            self.pb_blades = None

        # Drawing area
        self.area = Gtk.DrawingArea()
        self.area.set_content_width(200)
        self.area.set_content_height(200)
        self.area.set_draw_func(self._draw_func)
        self.append(self.area)

        # Labels below
        info_box = Gtk.Box(orientation=Gtk.Orientation.VERTICAL, spacing=2)
        info_box.set_halign(Gtk.Align.CENTER)

        self.lbl_speed = Gtk.Label(label="AUTO")
        self.lbl_speed.add_css_class("speed-value")

        self.lbl_rpm = Gtk.Label(label="-- RPM")
        self.lbl_rpm.add_css_class("speed-unit")

        info_box.append(self.lbl_speed)
        info_box.append(self.lbl_rpm)
        self.append(info_box)

        # Continuous gentle spin animation when active
        GLib.timeout_add(40, self._on_spin_tick)

    def set_speed(self, percent: int, is_auto: bool = False):
        self.speed_percent = max(0, min(100, percent))
        self.is_auto = is_auto
        if self.is_auto:
            self.lbl_speed.set_text("AUTO")
            # Estimated auto baseline RPM
            self.rpm = 2200 if not self.is_gpu else 0
        else:
            self.lbl_speed.set_text(f"{self.speed_percent}%")
            self.rpm = int((self.speed_percent / 100.0) * 5200)
        
        self.lbl_rpm.set_text(f"{self.rpm} RPM" if self.rpm > 0 else "0 RPM")
        self.area.queue_draw()

    def _on_spin_tick(self):
        # Rotate fan blades if spinning
        effective_speed = 30 if self.is_auto else self.speed_percent
        if effective_speed > 0:
            step = 0.04 + (effective_speed / 100.0) * 0.18
            self.angle_rad = (self.angle_rad + step) % (2 * math.pi)
            self.area.queue_draw()
        return True

    def _draw_func(self, area, cr, width, height):
        cx = width / 2.0
        cy = height / 2.0
        radius = 80.0

        # 1. Background Arc (Dark Track)
        start_angle = 135 * (math.pi / 180.0)
        end_angle = 405 * (math.pi / 180.0)

        cr.set_line_width(8.0)
        cr.set_source_rgba(0.13, 0.15, 0.18, 0.8)
        cr.arc(cx, cy, radius, start_angle, end_angle)
        cr.stroke()

        # 2. Glowing Active Progress Arc
        pct = (self.speed_percent / 100.0) if not self.is_auto else 0.45
        sweep = (end_angle - start_angle) * pct
        active_end = start_angle + sweep

        # Arc color gradient effect
        if self.speed_percent >= 90:
            cr.set_source_rgba(0.97, 0.32, 0.29, 0.9)  # Cyber Red / Turbo
        elif self.is_gpu:
            cr.set_source_rgba(0.22, 0.55, 0.99, 0.9)  # NVIDIA Blue
        else:
            cr.set_source_rgba(0.0, 0.82, 1.0, 0.9)    # Neon Cyan

        cr.set_line_width(8.0)
        cr.arc(cx, cy, radius, start_angle, active_end)
        cr.stroke()

        # 3. Draw Outer Ring Pixbuf
        if self.pb_outer:
            ow = self.pb_outer.get_width()
            oh = self.pb_outer.get_height()
            cr.save()
            Gdk.cairo_set_source_pixbuf(cr, self.pb_outer, cx - ow/2, cy - oh/2)
            cr.paint_with_alpha(0.85)
            cr.restore()

        # 4. Draw Rotating Fan Blades in Center
        if self.pb_blades:
            bw = self.pb_blades.get_width()
            bh = self.pb_blades.get_height()
            cr.save()
            cr.translate(cx, cy)
            cr.rotate(self.angle_rad)
            Gdk.cairo_set_source_pixbuf(cr, self.pb_blades, -bw/2, -bh/2)
            cr.paint_with_alpha(0.9)
            cr.restore()
