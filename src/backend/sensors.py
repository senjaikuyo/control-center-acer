"""
Hardware sensors reader for CPU, NVIDIA GPU, and Battery status.
Uses low-overhead direct C bindings (NVML) and sysfs.
"""

import os
import glob
import ctypes
import subprocess
import logging

logger = logging.getLogger("Sensors")

class SensorManager:
    def __init__(self):
        self._nvml = None
        self._gpu_device = None
        self._init_nvml()

    def _init_nvml(self):
        try:
            self._nvml = ctypes.CDLL("libnvidia-ml.so.1")
            ret = self._nvml.nvmlInit_v2()
            if ret == 0:
                dev = ctypes.c_void_p()
                if self._nvml.nvmlDeviceGetHandleByIndex_v2(0, ctypes.byref(dev)) == 0:
                    self._gpu_device = dev
            else:
                self._nvml = None
        except Exception as e:
            logger.debug(f"NVML init fallback: {e}")
            self._nvml = None

    def get_cpu_temp(self) -> int:
        for p in glob.glob("/sys/class/hwmon/hwmon*"):
            name_file = os.path.join(p, "name")
            if os.path.exists(name_file):
                try:
                    with open(name_file, "r") as f:
                        if "coretemp" in f.read():
                            temp_file = os.path.join(p, "temp1_input")
                            if os.path.exists(temp_file):
                                with open(temp_file, "r") as tf:
                                    return int(tf.read().strip()) // 1000
                except Exception:
                    pass
        return 0

    def get_gpu_temp(self) -> int:
        if self._nvml and self._gpu_device:
            try:
                temp = ctypes.c_uint()
                # 0 = NVML_TEMPERATURE_GPU
                if self._nvml.nvmlDeviceGetTemperature(self._gpu_device, 0, ctypes.byref(temp)) == 0:
                    return int(temp.value)
            except Exception:
                pass
        return 0

    def get_gpu_utilization(self) -> int:
        if self._nvml and self._gpu_device:
            try:
                # nvmlUtilization_t: struct { unsigned int gpu; unsigned int memory; }
                class nvmlUtilization_t(ctypes.Structure):
                    _fields_ = [("gpu", ctypes.c_uint), ("memory", ctypes.c_uint)]
                util = nvmlUtilization_t()
                if self._nvml.nvmlDeviceGetUtilizationRates(self._gpu_device, ctypes.byref(util)) == 0:
                    return int(util.gpu)
            except Exception:
                pass
        return 0

    def get_cpu_freq_ghz(self) -> float:
        try:
            with open("/sys/devices/system/cpu/cpu0/cpufreq/scaling_cur_freq", "r") as f:
                khz = int(f.read().strip())
                return round(khz / 1000000.0, 2)
        except Exception:
            return 0.0

    def get_battery_info(self) -> dict:
        info = {"percent": 0, "status": "Unknown", "present": False}
        for bat in glob.glob("/sys/class/power_supply/BAT*"):
            try:
                cap_file = os.path.join(bat, "capacity")
                stat_file = os.path.join(bat, "status")
                if os.path.exists(cap_file):
                    with open(cap_file, "r") as f:
                        info["percent"] = int(f.read().strip())
                    info["present"] = True
                if os.path.exists(stat_file):
                    with open(stat_file, "r") as f:
                        info["status"] = f.read().strip()
                break
            except Exception:
                pass
        return info
