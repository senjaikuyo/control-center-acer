#!/usr/bin/env python3
"""
Acer Control Center 3.0 (Linux Edition)
Native Arch Linux & Clevo/Acer laptop implementation.
"""

import sys
import os
import gi

gi.require_version('Gtk', '4.0')
gi.require_version('Adw', '1')
from gi.repository import Gtk, Gdk, Adw

# Add src to python path
src_dir = os.path.dirname(os.path.abspath(__file__))
if src_dir not in sys.path:
    sys.path.insert(0, src_dir)

from ui.style import CSS_STYLES
from ui.window import AcerMainWindow

class AcerControlCenterApp(Adw.Application):
    def __init__(self):
        super().__init__(application_id="com.acer.ControlCenterLinux")

    def do_activate(self):
        self._load_styles()
        win = self.props.active_window
        if not win:
            win = AcerMainWindow(self)
        win.present()

    def _load_styles(self):
        provider = Gtk.CssProvider()
        provider.load_from_data(CSS_STYLES.encode('utf-8'))
        display = Gdk.Display.get_default()
        if display:
            Gtk.StyleContext.add_provider_for_display(
                display,
                provider,
                Gtk.STYLE_PROVIDER_PRIORITY_APPLICATION
            )

def main():
    app = AcerControlCenterApp()
    return app.run(sys.argv)

if __name__ == "__main__":
    sys.exit(main())
