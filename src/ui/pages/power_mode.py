"""
Power & Performance Modes page replicating Control Center 3.0 dashboard.
Features Quiet, Power Saving, Entertainment, and Performance profiles.
"""

import gi
gi.require_version('Gtk', '4.0')
from gi.repository import Gtk, GdkPixbuf
from ui.assets import Assets
from backend.tailor import TailorBackend

class PowerModePage(Gtk.Box):
    def __init__(self, backend: TailorBackend, on_profile_changed=None):
        super().__init__(orientation=Gtk.Orientation.VERTICAL, spacing=20)
        self.backend = backend
        self.on_profile_changed = on_profile_changed
        self.cards = {}

        self.set_margin_start(28)
        self.set_margin_end(28)
        self.set_margin_top(20)
        self.set_margin_bottom(20)

        # Header Title
        title_box = Gtk.Box(orientation=Gtk.Orientation.VERTICAL, spacing=4)
        lbl_head = Gtk.Label(label="POWER & PERFORMANCE MODES", xalign=0)
        lbl_head.add_css_class("window-title")
        lbl_sub = Gtk.Label(label="Select system thermal and power management profile", xalign=0)
        lbl_sub.add_css_class("window-subtitle")
        title_box.append(lbl_head)
        title_box.append(lbl_sub)
        self.append(title_box)

        # 2x2 Grid of Mode Cards
        grid = Gtk.Grid()
        grid.set_column_spacing(18)
        grid.set_row_spacing(18)
        grid.set_column_homogeneous(True)
        grid.set_row_homogeneous(True)
        grid.set_hexpand(True)
        grid.set_vexpand(True)

        mode_defs = [
            (
                "quiet",
                "QUIET",
                "Prioritize silent fan acoustics and relaxed thermal profile for quiet environments.",
                Assets.IC_QUIET,
                "active-quiet",
                0, 0
            ),
            (
                "power_saving",
                "POWER SAVING",
                "Maximize battery endurance by scaling down CPU frequencies and background tasks.",
                Assets.IC_POWER_SAVING,
                "active-power-saving",
                1, 0
            ),
            (
                "entertainment",
                "ENTERTAINMENT",
                "Balanced everyday performance for browsing, media playback, and casual gaming.",
                Assets.IC_ENTERTAINMENT,
                "active-entertainment",
                0, 1
            ),
            (
                "performance",
                "PERFORMANCE",
                "Unlock high clock boosts and aggressive thermal cooling for heavy workloads & gaming.",
                Assets.IC_PERFORMANCE,
                "active-performance",
                1, 1
            )
        ]

        for code, title, desc, icon_path, active_css, col, row in mode_defs:
            card = self._create_mode_card(code, title, desc, icon_path, active_css)
            self.cards[code] = (card, active_css)
            grid.attach(card, col, row, 1, 1)

        self.append(grid)

        # Sync initial state
        initial_mode = self.backend.get_performance_profile()
        self.set_active_mode(initial_mode)

    def _create_mode_card(self, code: str, title: str, desc: str, icon_path: str, active_css: str) -> Gtk.Button:
        btn = Gtk.Button()
        btn.add_css_class("mode-card")

        box = Gtk.Box(orientation=Gtk.Orientation.VERTICAL, spacing=12)
        box.set_vexpand(True)

        # Icon + Title row
        head_row = Gtk.Box(orientation=Gtk.Orientation.HORIZONTAL, spacing=14)
        
        try:
            pb = GdkPixbuf.Pixbuf.new_from_file_at_scale(icon_path, 42, 42, True)
            img = Gtk.Image.new_from_pixbuf(pb)
        except Exception:
            img = Gtk.Image.new_from_icon_name("preferences-system-symbolic")

        lbl_title = Gtk.Label(label=title, xalign=0)
        lbl_title.add_css_class("mode-title")

        head_row.append(img)
        head_row.append(lbl_title)
        box.append(head_row)

        # Description
        lbl_desc = Gtk.Label(label=desc, xalign=0)
        lbl_desc.set_wrap(True)
        lbl_desc.add_css_class("mode-desc")
        box.append(lbl_desc)

        btn.set_child(box)
        btn.connect("clicked", lambda b, c=code: self._on_card_clicked(c))
        return btn

    def _on_card_clicked(self, mode: str):
        if self.backend.set_performance_profile(mode):
            self.set_active_mode(mode)
            if self.on_profile_changed:
                self.on_profile_changed(mode)

    def set_active_mode(self, active_mode: str):
        for code, (card, css_class) in self.cards.items():
            card.remove_css_class(css_class)
            if code == active_mode:
                card.add_css_class(css_class)
