#!/usr/bin/env python3
import sys
import gi
gi.require_version('Gtk', '4.0')
gi.require_version('Adw', '1')
from gi.repository import Gtk, Adw

from ui import AcerControlCenterWindow

class AcerControlCenterApp(Adw.Application):
    def __init__(self):
        super().__init__(application_id="com.acer.ControlCenterLinux")

    def do_activate(self):
        win = self.props.active_window
        if not win:
            win = AcerControlCenterWindow(application=self)
        win.present()

def main():
    app = AcerControlCenterApp()
    return app.run(sys.argv)

if __name__ == "__main__":
    sys.exit(main())
