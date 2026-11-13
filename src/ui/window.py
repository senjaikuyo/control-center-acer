"""
Main Application Window for Acer Control Center 3.0 (Linux Edition).
Composes genuine background art, header bar, navigation sidebar, and views.
"""

import gi
gi.require_version('Gtk', '4.0')
from gi.repository import Gtk, GdkPixbuf
from ui.assets import Assets
from ui.pages.power_mode import PowerModePage
from ui.pages.fan_control import FanControlPage
from ui.pages.system_info import SystemInfoPage
from backend.tailor import TailorBackend
from backend.sensors import SensorManager

class AcerMainWindow(Gtk.ApplicationWindow):
    def __init__(self, app):
        super().__init__(application=app, title="Acer Control Center 3.0")
        self.set_default_size(1020, 660)
        self.set_resizable(False)
        self.add_css_class("acer-window")

        # Initialize backends
        self.tailor = TailorBackend()
        self.sensors = SensorManager()

        # Root Box
        root_box = Gtk.Box(orientation=Gtk.Orientation.VERTICAL)
        self.set_child(root_box)

        # 1. Custom Header Bar
        header = self._build_header()
        root_box.append(header)

        # 2. Main Content (Sidebar + Stack)
        body = Gtk.Box(orientation=Gtk.Orientation.HORIZONTAL)
        body.set_vexpand(True)
        root_box.append(body)

        # Navigation Sidebar
        sidebar = self._build_sidebar()
        body.append(sidebar)

        # Content Stack
        self.stack = Gtk.Stack()
        self.stack.set_transition_type(Gtk.StackTransitionType.CROSSFADE)
        self.stack.set_transition_duration(200)
        self.stack.set_hexpand(True)
        self.stack.set_vexpand(True)

        self.page_power = PowerModePage(self.tailor)
        self.page_fan = FanControlPage(self.tailor, self.sensors)
        self.page_info = SystemInfoPage(self.sensors)

        self.stack.add_named(self.page_power, "power")
        self.stack.add_named(self.page_fan, "fan")
        self.stack.add_named(self.page_info, "info")

        body.append(self.stack)

        # Select initial tab
        self._select_tab("power")

    def _build_header(self) -> Gtk.Box:
        header = Gtk.Box(orientation=Gtk.Orientation.HORIZONTAL, spacing=12)
        header.add_css_class("acer-header")

        # App Logo & Title
        try:
            pb_logo = GdkPixbuf.Pixbuf.new_from_file_at_scale(Assets.APP_LOGO, 28, 28, True)
            logo_img = Gtk.Image.new_from_pixbuf(pb_logo)
            header.append(logo_img)
        except Exception:
            pass

        title_vbox = Gtk.Box(orientation=Gtk.Orientation.VERTICAL, spacing=2)
        lbl_title = Gtk.Label(label="CONTROL CENTER 3.0", xalign=0)
        lbl_title.add_css_class("window-title")
        lbl_sub = Gtk.Label(label="ACER ASPIRE 7 / CLEVO HARDWARE EDITION", xalign=0)
        lbl_sub.add_css_class("window-subtitle")
        title_vbox.append(lbl_title)
        title_vbox.append(lbl_sub)
        header.append(title_vbox)

        # Spacer
        spacer = Gtk.Box()
        spacer.set_hexpand(True)
        header.append(spacer)

        # Window Controls (Minimize & Close)
        btn_min = Gtk.Button()
        btn_min.add_css_class("win-ctrl-btn")
        try:
            pb_min = GdkPixbuf.Pixbuf.new_from_file_at_scale(Assets.IC_MINIMIZE, 16, 16, True)
            btn_min.set_child(Gtk.Image.new_from_pixbuf(pb_min))
        except Exception:
            btn_min.set_child(Gtk.Image.new_from_icon_name("window-minimize-symbolic"))
        btn_min.connect("clicked", lambda b: self.minimize())
        header.append(btn_min)

        btn_close = Gtk.Button()
        btn_close.add_css_class("win-ctrl-btn")
        btn_close.add_css_class("win-close-btn")
        try:
            pb_close = GdkPixbuf.Pixbuf.new_from_file_at_scale(Assets.IC_CLOSE, 16, 16, True)
            btn_close.set_child(Gtk.Image.new_from_pixbuf(pb_close))
        except Exception:
            btn_close.set_child(Gtk.Image.new_from_icon_name("window-close-symbolic"))
        btn_close.connect("clicked", lambda b: self.close())
        header.append(btn_close)

        return header

    def _build_sidebar(self) -> Gtk.Box:
        box = Gtk.Box(orientation=Gtk.Orientation.VERTICAL, spacing=8)
        box.add_css_class("nav-tab-box")
        box.set_size_request(210, -1)

        self.tab_buttons = {}

        tabs = [
            ("power", "Power Modes", Assets.IC_TAB_MODES),
            ("fan", "Fan Speed", Assets.IC_TAB_FAN),
            ("info", "Hardware Monitor", Assets.IC_TAB_MONITOR)
        ]

        for tab_id, label_text, icon_path in tabs:
            btn = Gtk.Button()
            btn.add_css_class("nav-tab-btn")

            btn_box = Gtk.Box(orientation=Gtk.Orientation.HORIZONTAL, spacing=12)
            try:
                pb = GdkPixbuf.Pixbuf.new_from_file_at_scale(icon_path, 22, 22, True)
                btn_box.append(Gtk.Image.new_from_pixbuf(pb))
            except Exception:
                pass
            
            lbl = Gtk.Label(label=label_text, xalign=0)
            btn_box.append(lbl)

            btn.set_child(btn_box)
            btn.connect("clicked", lambda b, tid=tab_id: self._select_tab(tid))
            self.tab_buttons[tab_id] = btn
            box.append(btn)

        # Stretch bottom
        bottom_spacer = Gtk.Box()
        bottom_spacer.set_vexpand(True)
        box.append(bottom_spacer)

        return box

    def _select_tab(self, tab_id: str):
        for tid, btn in self.tab_buttons.items():
            btn.remove_css_class("active")
            if tid == tab_id:
                btn.add_css_class("active")
        self.stack.set_visible_child_name(tab_id)
