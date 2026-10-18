import os
import glob
import ctypes
import subprocess
from gi.repository import Gio, GLib

class HardwareManager:
    def __init__(self):
        self._bus = None
        self._fan_proxy = None
        self._profile_proxy = None
        self._nvml = None
        self._init_dbus()
        self._init_nvml()

    def _init_dbus(self):
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
            self._profile_proxy = Gio.DBusProxy.new_sync(
                self._bus,
                Gio.DBusProxyFlags.NONE,
                None,
                "com.tux.Tailor",
                "/com/tux/Tailor",
                "com.tux.Tailor.Profiles",
                None
            )
        except Exception as e:
            print(f"[HardwareManager] D-Bus init error: {e}")

    def _init_nvml(self):
        try:
            self._nvml = ctypes.CDLL("libnvidia-ml.so.1")
            self._nvml.nvmlInit_v2()
        except Exception:
            self._nvml = None

    def get_number_of_fans(self) -> int:
        if not self._profile_proxy:
            return 0
        try:
            res = self._profile_proxy.call_sync(
                "GetNumberOfFans", None, Gio.DBusCallFlags.NONE, -1, None
            )
            return int(res.unpack()[0])
        except Exception as e:
            print(f"[HardwareManager] GetNumberOfFans error: {e}")
            return 2  # default fallback

    def set_fan_speed(self, fan_index: int, speed_percent: int):
        """Set fan speed (0 = Auto/Profile, 1-100 = manual percent)"""
        if not self._fan_proxy:
            return False
        try:
            speed = max(0, min(100, int(speed_percent)))
            params = GLib.Variant("(yy)", (fan_index, speed))
            self._fan_proxy.call_sync(
                "OverrideSpeed", params, Gio.DBusCallFlags.NONE, -1, None
            )
            return True
        except Exception as e:
            print(f"[HardwareManager] OverrideSpeed error: {e}")
            return False

    def get_cpu_temp(self) -> int:
        for p in glob.glob("/sys/class/hwmon/hwmon*"):
            name_file = os.path.join(p, "name")
            if os.path.exists(name_file):
                try:
                    with open(name_file) as f:
                        if "coretemp" in f.read():
                            temp_file = os.path.join(p, "temp1_input")
                            if os.path.exists(temp_file):
                                with open(temp_file) as tf:
                                    return int(tf.read().strip()) // 1000
                except Exception:
                    pass
        return 0

    def get_gpu_temp(self) -> int:
        if self._nvml:
            try:
                device = ctypes.c_void_p()
                self._nvml.nvmlDeviceGetHandleByIndex_v2(0, ctypes.byref(device))
                temp = ctypes.c_uint()
                self._nvml.nvmlDeviceGetTemperature(device, 0, ctypes.byref(temp))
                return int(temp.value)
            except Exception:
                pass
        return 0

    def get_cpu_freq_ghz(self) -> float:
        try:
            with open("/sys/devices/system/cpu/cpu0/cpufreq/scaling_cur_freq") as f:
                khz = int(f.read().strip())
                return round(khz / 1000000.0, 2)
        except Exception:
            return 0.0

    def get_power_profile(self) -> str:
        try:
            res = subprocess.run(
                ["/usr/bin/python3", "/usr/bin/powerprofilesctl", "get"],
                capture_output=True, text=True, timeout=2
            )
            return res.stdout.strip()
        except Exception:
            return "balanced"

    def set_power_profile(self, profile: str) -> bool:
        try:
            subprocess.run(
                ["/usr/bin/python3", "/usr/bin/powerprofilesctl", "set", profile],
                check=True, timeout=3
            )
            return True
        except Exception as e:
            print(f"[HardwareManager] set_power_profile error: {e}")
            return False
