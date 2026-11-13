"""
Backend connector for Tuxedo Tailor (tailord) D-Bus interface.
Controls Performance Profiles, Dual Fans, and RGB Keyboard natively on Clevo/Acer ODM hardware.
"""

from gi.repository import Gio, GLib
import json
import logging

logger = logging.getLogger("TailorDBus")

class TailorBackend:
    PROFILES = ["quiet", "power_saving", "entertainment", "performance"]

    def __init__(self):
        self._bus = None
        self._fan_proxy = None
        self._perf_proxy = None
        self._profiles_proxy = None
        self._led_proxy = None
        self.is_connected = False
        self._connect()

    def _connect(self):
        try:
            self._bus = Gio.bus_get_sync(Gio.BusType.SYSTEM, None)
            self._fan_proxy = Gio.DBusProxy.new_sync(
                self._bus,
                Gio.DBusProxyFlags.NONE,
                None,
                "com.tux.Tailor",
                "/com/tux/Tailor",
                "com.tux.Tailor.Fan",
                None
            )
            self._perf_proxy = Gio.DBusProxy.new_sync(
                self._bus,
                Gio.DBusProxyFlags.NONE,
                None,
                "com.tux.Tailor",
                "/com/tux/Tailor",
                "com.tux.Tailor.Performance",
                None
            )
            self._profiles_proxy = Gio.DBusProxy.new_sync(
                self._bus,
                Gio.DBusProxyFlags.NONE,
                None,
                "com.tux.Tailor",
                "/com/tux/Tailor",
                "com.tux.Tailor.Profiles",
                None
            )
            self._led_proxy = Gio.DBusProxy.new_sync(
                self._bus,
                Gio.DBusProxyFlags.NONE,
                None,
                "com.tux.Tailor",
                "/com/tux/Tailor",
                "com.tux.Tailor.Led",
                None
            )
            self.is_connected = True
        except Exception as e:
            logger.error(f"Failed to connect to tailord D-Bus: {e}")
            self.is_connected = False

    def get_number_of_fans(self) -> int:
        if not self.is_connected or not self._profiles_proxy:
            return 2
        try:
            res = self._profiles_proxy.call_sync("GetNumberOfFans", None, Gio.DBusCallFlags.NONE, -1, None)
            return int(res.unpack()[0])
        except Exception:
            return 2

    def get_performance_profile(self) -> str:
        if not self.is_connected or not self._perf_proxy:
            return "entertainment"
        try:
            res = self._perf_proxy.call_sync("GetProfile", None, Gio.DBusCallFlags.NONE, -1, None)
            return str(res.unpack()[0])
        except Exception as e:
            logger.error(f"get_performance_profile error: {e}")
            return "entertainment"

    def set_performance_profile(self, profile: str) -> bool:
        if not self.is_connected or not self._perf_proxy:
            return False
        if profile not in self.PROFILES:
            logger.warning(f"Unknown performance profile: {profile}")
            return False
        try:
            self._perf_proxy.call_sync(
                "SetProfile",
                GLib.Variant("(s)", (profile,)),
                Gio.DBusCallFlags.NONE,
                -1,
                None
            )
            return True
        except Exception as e:
            logger.error(f"set_performance_profile error: {e}")
            return False

    def set_fan_speed(self, fan_index: int, speed_percent: int) -> bool:
        """
        Set fan speed:
        speed_percent = 0 -> Automatic mode (follows active curve)
        speed_percent = 1..100 -> Manual override percentage
        """
        if not self.is_connected or not self._fan_proxy:
            return False
        try:
            spd = max(0, min(100, int(speed_percent)))
            params = GLib.Variant("(yy)", (fan_index, spd))
            self._fan_proxy.call_sync("OverrideSpeed", params, Gio.DBusCallFlags.NONE, -1, None)
            return True
        except Exception as e:
            logger.error(f"set_fan_speed (fan={fan_index}, speed={speed_percent}) error: {e}")
            return False

    def set_all_fans_auto(self) -> bool:
        num = self.get_number_of_fans()
        success = True
        for i in range(num):
            if not self.set_fan_speed(i, 0):
                success = False
        return success

    def set_turbo_fans(self, turbo: bool) -> bool:
        """Turbo spins all fans at 100% full speed, or resets to auto"""
        num = self.get_number_of_fans()
        target_speed = 100 if turbo else 0
        success = True
        for i in range(num):
            if not self.set_fan_speed(i, target_speed):
                success = False
        return success

    def set_keyboard_rgb(self, r: int, g: int, b: int) -> bool:
        if not self.is_connected or not self._led_proxy:
            return False
        try:
            payload = json.dumps({"Single": {"r": int(r), "g": int(g), "b": int(b)}})
            self._led_proxy.call_sync(
                "OverrideColor",
                GLib.Variant("(s)", (payload,)),
                Gio.DBusCallFlags.NONE,
                -1,
                None
            )
            return True
        except Exception as e:
            logger.error(f"set_keyboard_rgb error: {e}")
            return False
