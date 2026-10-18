#![allow(dead_code)]

use std::ffi::CString;
use std::fmt;
use std::fs;
use std::mem::MaybeUninit;
use std::path::Path;
use std::process::Command;

/// Supported system power profiles.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
pub enum PowerMode {
    Quiet,
    Entertainment,
    #[default]
    Performance,
}

impl PowerMode {
    /// Returns the corresponding powerprofilesctl profile name.
    pub fn as_powerprofiles_str(&self) -> &'static str {
        match self {
            PowerMode::Quiet => "power-saver",
            PowerMode::Entertainment => "balanced",
            PowerMode::Performance => "performance",
        }
    }
}

/// Fan operation modes.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
pub enum FanMode {
    #[default]
    Automatic,
    Maximum,
    Custom,
    Turbo,
}

/// GPU switching modes.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
pub enum GpuSwitchMode {
    Integrated,
    #[default]
    Dynamic,
    Discrete,
}

/// A single node in a custom temperature-to-speed fan curve.
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct FanCurveNode {
    /// Temperature in Celsius (e.g. 20.0 to 100.0).
    pub temp: f32,
    /// Targeted fan duty / speed percentage (0.0 to 100.0).
    pub speed: f32,
}

impl FanCurveNode {
    pub const fn new(temp: f32, speed: f32) -> Self {
        Self { temp, speed }
    }
}

/// Hardware telemetry metrics for CPU, GPU, RAM, Storage, and Fans.
#[derive(Debug, Clone, PartialEq)]
pub struct HardwareTelemetry {
    // CPU
    pub cpu_clock_mhz: u32,
    pub cpu_temp: u32,
    pub cpu_usage_pct: u32,

    // GPU (RTX 3050 Laptop)
    pub gpu_clock_mhz: u32,
    pub gpu_temp: u32,
    pub gpu_usage_pct: u32,

    // Memory
    pub mem_used_gb: f32,
    pub mem_total_gb: f32,
    pub mem_pct: u32,

    // Storage ("/")
    pub disk_used_gb: u32,
    pub disk_total_gb: u32,
    pub disk_pct: u32,

    // Fans
    pub cpu_fan_rpm: u32,
    pub gpu_fan_rpm: u32,
    pub cpu_fan_pct: u32,
    pub gpu_fan_pct: u32,
}

impl Default for HardwareTelemetry {
    fn default() -> Self {
        Self {
            cpu_clock_mhz: 0,
            cpu_temp: 0,
            cpu_usage_pct: 0,
            gpu_clock_mhz: 0,
            gpu_temp: 0,
            gpu_usage_pct: 0,
            mem_used_gb: 0.0,
            mem_total_gb: 0.0,
            mem_pct: 0,
            disk_used_gb: 0,
            disk_total_gb: 0,
            disk_pct: 0,
            cpu_fan_rpm: 0,
            gpu_fan_rpm: 0,
            cpu_fan_pct: 0,
            gpu_fan_pct: 0,
        }
    }
}

/// Error type for hardware backend operations.
#[allow(dead_code)]
#[derive(Debug)]
pub enum BackendError {
    Io(std::io::Error),
    CommandFailed { command: String, message: String },
    DbusError(String),
}

impl fmt::Display for BackendError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            BackendError::Io(err) => write!(f, "I/O error: {}", err),
            BackendError::CommandFailed { command, message } => {
                write!(f, "Command '{}' failed: {}", command, message)
            }
            BackendError::DbusError(msg) => write!(f, "D-Bus error: {}", msg),
        }
    }
}

impl std::error::Error for BackendError {}

impl From<std::io::Error> for BackendError {
    fn from(err: std::io::Error) -> Self {
        BackendError::Io(err)
    }
}

// ============================================================================
// TELEMETRY SAMPLING HELPERS
// ============================================================================

/// Reads CPU temperature in Celsius from `/sys/class/hwmon/hwmon*/temp1_input`.
/// Matches the hwmon device whose `name` file contains "coretemp".
pub fn read_cpu_temp() -> Option<u32> {
    read_cpu_temp_from_dir(Path::new("/sys/class/hwmon"))
}

/// Parameterized directory version of `read_cpu_temp` for testing and modularity.
pub fn read_cpu_temp_from_dir(hwmon_dir: &Path) -> Option<u32> {
    let entries = fs::read_dir(hwmon_dir).ok()?;
    for entry in entries.flatten() {
        let path = entry.path();
        let name_path = path.join("name");
        if let Ok(name) = fs::read_to_string(&name_path) {
            if name.trim().contains("coretemp") {
                let temp_path = path.join("temp1_input");
                if let Ok(temp_str) = fs::read_to_string(temp_path) {
                    if let Ok(milli) = temp_str.trim().parse::<u32>() {
                        return Some(milli / 1000);
                    }
                }
            }
        }
    }
    None
}

/// Reads average CPU frequency in MHz from `/sys/devices/system/cpu/cpu*/cpufreq/scaling_cur_freq`.
pub fn read_cpu_frequency_mhz() -> Option<u32> {
    read_cpu_frequency_from_dir(Path::new("/sys/devices/system/cpu"))
}

/// Parameterized directory version of `read_cpu_frequency_mhz`.
pub fn read_cpu_frequency_from_dir(cpu_dir: &Path) -> Option<u32> {
    let entries = fs::read_dir(cpu_dir).ok()?;
    let mut total_khz: u64 = 0;
    let mut core_count: u64 = 0;

    for entry in entries.flatten() {
        let file_name = entry.file_name();
        let name_str = file_name.to_string_lossy();
        if name_str.starts_with("cpu") && name_str[3..].chars().all(|c| c.is_ascii_digit()) {
            let freq_path = entry.path().join("cpufreq/scaling_cur_freq");
            if let Ok(content) = fs::read_to_string(freq_path) {
                if let Ok(khz) = content.trim().parse::<u64>() {
                    total_khz += khz;
                    core_count += 1;
                }
            }
        }
    }

    if core_count > 0 {
        Some(((total_khz / core_count) / 1000) as u32)
    } else {
        // Fallback: check cpu0 directly
        let fallback_path = cpu_dir.join("cpu0/cpufreq/scaling_cur_freq");
        let content = fs::read_to_string(fallback_path).ok()?;
        let khz = content.trim().parse::<u64>().ok()?;
        Some((khz / 1000) as u32)
    }
}

/// Computes CPU usage percentage from delta jiffies in `/proc/stat`.
///
/// `last_ticks` is updated with `(idle_jiffies, total_jiffies)`.
pub fn read_cpu_usage(last_ticks: &mut (u64, u64)) -> Option<u32> {
    let stat = fs::read_to_string("/proc/stat").ok()?;
    parse_cpu_usage_stat(&stat, last_ticks)
}

/// Parses the first line of `/proc/stat` and calculates the CPU usage percentage.
pub fn parse_cpu_usage_stat(stat_content: &str, last_ticks: &mut (u64, u64)) -> Option<u32> {
    let first_line = stat_content.lines().next()?;
    if !first_line.starts_with("cpu") {
        return None;
    }

    let parts: Vec<u64> = first_line
        .split_whitespace()
        .skip(1)
        .filter_map(|s| s.parse().ok())
        .collect();

    if parts.len() < 4 {
        return None;
    }

    // parts[3] is idle, parts[4] is iowait (if present)
    let idle = parts[3] + parts.get(4).copied().unwrap_or(0);
    let total: u64 = parts.iter().sum();

    let (prev_idle, prev_total) = *last_ticks;
    let mut usage = 0u32;

    if prev_total > 0 && total > prev_total {
        let diff_idle = idle.saturating_sub(prev_idle);
        let diff_total = total.saturating_sub(prev_total);
        if diff_total > 0 {
            let pct = (1.0 - (diff_idle as f64 / diff_total as f64)) * 100.0;
            usage = pct.clamp(0.0, 100.0).round() as u32;
        }
    }

    *last_ticks = (idle, total);
    Some(usage)
}

/// Reads NVIDIA RTX 3050 metrics via `nvidia-smi`:
/// `clocks.gr` (MHz), `utilization.gpu` (%), `temperature.gpu` (°C).
pub fn read_nvidia_gpu_stats() -> Option<(u32, u32, u32)> {
    let output = Command::new("nvidia-smi")
        .args([
            "--query-gpu=clocks.gr,utilization.gpu,temperature.gpu",
            "--format=csv,noheader,nounits",
        ])
        .output()
        .ok()?;

    if !output.status.success() {
        return None;
    }

    let stdout_str = String::from_utf8(output.stdout).ok()?;
    parse_nvidia_smi_output(&stdout_str)
}

/// Parses csv output from `nvidia-smi`: "clocks, utilization, temperature".
pub fn parse_nvidia_smi_output(output_str: &str) -> Option<(u32, u32, u32)> {
    let line = output_str.lines().next()?.trim();
    if line.is_empty() {
        return None;
    }

    let parts: Vec<&str> = line.split(',').map(|s| s.trim()).collect();
    if parts.len() >= 3 {
        let clock = parts[0].parse::<u32>().unwrap_or(0);
        let usage = parts[1].parse::<u32>().unwrap_or(0);
        let temp = parts[2].parse::<u32>().unwrap_or(0);
        Some((clock, usage, temp))
    } else {
        None
    }
}

/// Reads memory statistics from `/proc/meminfo`:
/// returns `(used_gb, total_gb, pct)`.
pub fn read_memory_stats() -> Option<(f32, f32, u32)> {
    let mem = fs::read_to_string("/proc/meminfo").ok()?;
    parse_meminfo(&mem)
}

/// Parses `/proc/meminfo` content for MemTotal and MemAvailable.
pub fn parse_meminfo(content: &str) -> Option<(f32, f32, u32)> {
    let mut total_kb = 0u64;
    let mut avail_kb = 0u64;

    for line in content.lines() {
        if line.starts_with("MemTotal:") {
            total_kb = line
                .split_whitespace()
                .nth(1)
                .and_then(|s| s.parse().ok())
                .unwrap_or(0);
        } else if line.starts_with("MemAvailable:") {
            avail_kb = line
                .split_whitespace()
                .nth(1)
                .and_then(|s| s.parse().ok())
                .unwrap_or(0);
        }
    }

    if total_kb == 0 {
        return None;
    }

    let used_kb = total_kb.saturating_sub(avail_kb);
    let total_gb = ((total_kb as f64 / 1_048_576.0) * 10.0).round() as f32 / 10.0;
    let used_gb = ((used_kb as f64 / 1_048_576.0) * 10.0).round() as f32 / 10.0;
    let pct = ((used_kb as f64 / total_kb as f64) * 100.0).round() as u32;

    Some((used_gb, total_gb, pct))
}

/// Reads storage statistics for the given mount point (typically `"/"`) using `statvfs`:
/// returns `(used_gb, total_gb, pct)`.
pub fn read_storage_stats(mount_point: &str) -> Option<(u32, u32, u32)> {
    let c_path = CString::new(mount_point).ok()?;
    let mut stat = MaybeUninit::<libc::statvfs>::uninit();

    let res = unsafe { libc::statvfs(c_path.as_ptr(), stat.as_mut_ptr()) };
    if res != 0 {
        return None;
    }

    let stat = unsafe { stat.assume_init() };
    let frsize = stat.f_frsize as u64;
    let total_bytes = stat.f_blocks as u64 * frsize;
    let free_bytes = stat.f_bfree as u64 * frsize;
    let used_bytes = total_bytes.saturating_sub(free_bytes);

    let total_gb = (total_bytes / (1024 * 1024 * 1024)) as u32;
    let used_gb = (used_bytes / (1024 * 1024 * 1024)) as u32;
    let pct = if total_bytes > 0 {
        ((used_bytes as f64 / total_bytes as f64) * 100.0).round() as u32
    } else {
        0
    };

    Some((used_gb, total_gb, pct))
}

// ============================================================================
// FAN CURVE INTERPOLATION
// ============================================================================

/// Linearly interpolates targeted fan duty percentage (0.0 to 100.0)
/// given custom curve nodes and current temperature.
pub fn interpolate_fan_speed(nodes: &[FanCurveNode], temp: f32) -> f32 {
    if nodes.is_empty() {
        return 30.0;
    }

    if nodes.len() == 1 {
        return nodes[0].speed.clamp(0.0, 100.0);
    }

    // Ensure sorted copy for interpolation
    let mut sorted_nodes = nodes.to_vec();
    sorted_nodes.sort_by(|a, b| a.temp.partial_cmp(&b.temp).unwrap_or(std::cmp::Ordering::Equal));

    // Below lowest node temp
    if temp <= sorted_nodes[0].temp {
        return sorted_nodes[0].speed.clamp(0.0, 100.0);
    }

    // Above highest node temp
    let last = &sorted_nodes[sorted_nodes.len() - 1];
    if temp >= last.temp {
        return last.speed.clamp(0.0, 100.0);
    }

    // Segment interpolation
    for i in 0..sorted_nodes.len().saturating_sub(1) {
        let n1 = &sorted_nodes[i];
        let n2 = &sorted_nodes[i + 1];

        if temp >= n1.temp && temp <= n2.temp {
            let temp_diff = n2.temp - n1.temp;
            if temp_diff.abs() < 1e-4 {
                return n2.speed.clamp(0.0, 100.0);
            }
            let factor = (temp - n1.temp) / temp_diff;
            let speed = n1.speed + factor * (n2.speed - n1.speed);
            return speed.clamp(0.0, 100.0);
        }
    }

    30.0
}

// ============================================================================
// HARDWARE CONTROLS (D-BUS & SYSFS)
// ============================================================================

/// Controls Clevo Fan speed via D-Bus:
/// `busctl call com.tux.Tailor /com/tux/Tailor com.tux.Tailor.Fan OverrideSpeed yy <fan_index> <speed>`
///
/// - `fan_index`: 0 for CPU fan, 1 for GPU fan.
/// - `speed`: 0 = automatic / normal control, 1..100 = override duty percentage.
pub fn apply_clevo_fan(fan_index: u8, speed: u8) -> Result<(), BackendError> {
    let speed_clamped = speed.min(100);
    let output = Command::new("busctl")
        .args([
            "call",
            "com.tux.Tailor",
            "/com/tux/Tailor",
            "com.tux.Tailor.Fan",
            "OverrideSpeed",
            "yy",
            &fan_index.to_string(),
            &speed_clamped.to_string(),
        ])
        .output()?;

    if !output.status.success() {
        let stderr = String::from_utf8_lossy(&output.stderr).to_string();
        return Err(BackendError::CommandFailed {
            command: "busctl call com.tux.Tailor ...".to_string(),
            message: stderr,
        });
    }

    Ok(())
}

/// Applies fan speed to all available Clevo fans (fans 0 and 1).
pub fn apply_clevo_fan_all(speed: u8) -> Result<(), BackendError> {
    let _ = apply_clevo_fan(0, speed);
    let _ = apply_clevo_fan(1, speed);
    Ok(())
}

/// Sets system power profile via `powerprofilesctl set` (performance, balanced, power-saver),
/// with fallback to D-Bus `net.hadess.PowerProfiles` if needed.
pub fn set_power_profile(mode: PowerMode) -> Result<(), BackendError> {
    let profile = mode.as_powerprofiles_str();

    // 1. Try python3 /usr/bin/powerprofilesctl set <profile>
    let py_res = Command::new("/usr/bin/python3")
        .args(["/usr/bin/powerprofilesctl", "set", profile])
        .output();

    if let Ok(out) = py_res {
        if out.status.success() {
            return Ok(());
        }
    }

    // 2. Try powerprofilesctl directly
    let direct_res = Command::new("powerprofilesctl")
        .args(["set", profile])
        .output();

    if let Ok(out) = direct_res {
        if out.status.success() {
            return Ok(());
        }
    }

    // 3. Fallback: D-Bus call directly to net.hadess.PowerProfiles ActiveProfile
    let dbus_res = Command::new("busctl")
        .args([
            "set-property",
            "net.hadess.PowerProfiles",
            "/net/hadess/PowerProfiles",
            "net.hadess.PowerProfiles",
            "ActiveProfile",
            "s",
            profile,
        ])
        .output()?;

    if !dbus_res.status.success() {
        let err_msg = String::from_utf8_lossy(&dbus_res.stderr).to_string();
        return Err(BackendError::CommandFailed {
            command: "set power profile".to_string(),
            message: err_msg,
        });
    }

    Ok(())
}

// ============================================================================
// HARDWARE CONTROLLER
// ============================================================================

/// Coordinates hardware telemetry polling and hardware controls.
pub struct HardwareController {
    pub telemetry: HardwareTelemetry,
    pub power_mode: PowerMode,
    pub fan_mode: FanMode,
    pub gpu_switch: GpuSwitchMode,
    last_cpu_ticks: (u64, u64),
}

impl Default for HardwareController {
    fn default() -> Self {
        Self::new()
    }
}

impl HardwareController {
    /// Creates a new `HardwareController` instance.
    pub fn new() -> Self {
        Self {
            telemetry: HardwareTelemetry::default(),
            power_mode: PowerMode::Performance,
            fan_mode: FanMode::Automatic,
            gpu_switch: GpuSwitchMode::Dynamic,
            last_cpu_ticks: (0, 0),
        }
    }

    /// Polls all hardware telemetry:
    /// - Intel 13th Gen CPU temp, frequency, and usage.
    /// - NVIDIA RTX 3050 GPU clock, usage, and temp.
    /// - RAM and Storage capacity and utilization.
    /// - Computes fan RPM and duty percentage based on current fan mode and curves.
    pub fn poll_telemetry(
        &mut self,
        fan_mode: FanMode,
        cpu_curve_nodes: &[FanCurveNode],
        gpu_curve_nodes: &[FanCurveNode],
    ) -> &HardwareTelemetry {
        self.fan_mode = fan_mode;

        // 1. CPU Temperature
        if let Some(temp) = read_cpu_temp() {
            self.telemetry.cpu_temp = temp;
        }

        // 2. CPU Frequency
        if let Some(freq) = read_cpu_frequency_mhz() {
            self.telemetry.cpu_clock_mhz = freq;
        }

        // 3. CPU Usage
        if let Some(usage) = read_cpu_usage(&mut self.last_cpu_ticks) {
            self.telemetry.cpu_usage_pct = usage;
        }

        // 4. Memory
        if let Some((used, total, pct)) = read_memory_stats() {
            self.telemetry.mem_used_gb = used;
            self.telemetry.mem_total_gb = total;
            self.telemetry.mem_pct = pct;
        }

        // 5. Storage ("/")
        if let Some((used, total, pct)) = read_storage_stats("/") {
            self.telemetry.disk_used_gb = used;
            self.telemetry.disk_total_gb = total;
            self.telemetry.disk_pct = pct;
        }

        // 6. NVIDIA GPU
        if let Some((clock, usage, temp)) = read_nvidia_gpu_stats() {
            self.telemetry.gpu_clock_mhz = clock;
            self.telemetry.gpu_usage_pct = usage;
            self.telemetry.gpu_temp = temp;
        }

        // 7. Dynamic fan RPM and Duty calculation
        match self.fan_mode {
            FanMode::Automatic => {
                let cpu_pct = (self.telemetry.cpu_temp.saturating_sub(30) * 100 / 60).clamp(20, 100);
                self.telemetry.cpu_fan_pct = cpu_pct;
                self.telemetry.cpu_fan_rpm = (cpu_pct as f32 / 100.0 * 4800.0) as u32;

                if self.telemetry.gpu_temp > 45 {
                    let gpu_pct = (self.telemetry.gpu_temp.saturating_sub(40) * 100 / 50).clamp(20, 100);
                    self.telemetry.gpu_fan_pct = gpu_pct;
                    self.telemetry.gpu_fan_rpm = (gpu_pct as f32 / 100.0 * 4500.0) as u32;
                } else {
                    self.telemetry.gpu_fan_pct = 0;
                    self.telemetry.gpu_fan_rpm = 0;
                }
            }
            FanMode::Maximum => {
                self.telemetry.cpu_fan_pct = 100;
                self.telemetry.gpu_fan_pct = 100;
                self.telemetry.cpu_fan_rpm = 5100;
                self.telemetry.gpu_fan_rpm = 4900;
            }
            FanMode::Turbo => {
                self.telemetry.cpu_fan_pct = 100;
                self.telemetry.gpu_fan_pct = 100;
                self.telemetry.cpu_fan_rpm = 5400;
                self.telemetry.gpu_fan_rpm = 5200;
            }
            FanMode::Custom => {
                let cpu_speed = interpolate_fan_speed(cpu_curve_nodes, self.telemetry.cpu_temp as f32);
                let gpu_speed = interpolate_fan_speed(gpu_curve_nodes, self.telemetry.gpu_temp as f32);

                let cpu_pct = cpu_speed.round() as u32;
                let gpu_pct = gpu_speed.round() as u32;

                self.telemetry.cpu_fan_pct = cpu_pct;
                self.telemetry.cpu_fan_rpm = (cpu_speed / 100.0 * 5000.0) as u32;

                self.telemetry.gpu_fan_pct = gpu_pct;
                self.telemetry.gpu_fan_rpm = (gpu_speed / 100.0 * 4800.0) as u32;
            }
        }

        &self.telemetry
    }

    /// Sets power profile and applies corresponding fan and system parameters.
    pub fn set_power_mode(&mut self, mode: PowerMode) -> Result<(), BackendError> {
        self.power_mode = mode;
        set_power_profile(mode)?;

        match mode {
            PowerMode::Quiet | PowerMode::Entertainment => {
                self.fan_mode = FanMode::Automatic;
                let _ = apply_clevo_fan_all(0);
            }
            PowerMode::Performance => {}
        }

        Ok(())
    }

    /// Applies fan mode hardware control.
    pub fn set_fan_mode(
        &mut self,
        mode: FanMode,
        cpu_curve_nodes: &[FanCurveNode],
        gpu_curve_nodes: &[FanCurveNode],
    ) -> Result<(), BackendError> {
        self.fan_mode = mode;
        match mode {
            FanMode::Automatic => {
                apply_clevo_fan_all(0)?;
            }
            FanMode::Maximum | FanMode::Turbo => {
                apply_clevo_fan_all(100)?;
            }
            FanMode::Custom => {
                let cpu_speed = interpolate_fan_speed(cpu_curve_nodes, self.telemetry.cpu_temp as f32);
                let gpu_speed = interpolate_fan_speed(gpu_curve_nodes, self.telemetry.gpu_temp as f32);

                let _ = apply_clevo_fan(0, cpu_speed.round() as u8);
                let _ = apply_clevo_fan(1, gpu_speed.round() as u8);
            }
        }
        Ok(())
    }

    /// Directly overrides fan speed on a specific fan index (0 for CPU, 1 for GPU).
    pub fn override_fan(&self, fan_index: u8, speed: u8) -> Result<(), BackendError> {
        apply_clevo_fan(fan_index, speed)
    }

    /// Overrides fan speed on all fans.
    pub fn apply_fan_speed(&self, speed: u8) -> Result<(), BackendError> {
        apply_clevo_fan_all(speed)
    }

    /// Sets the GPU switch preference mode.
    pub fn set_gpu_switch(&mut self, mode: GpuSwitchMode) {
        self.gpu_switch = mode;
    }
}

// ============================================================================
// TESTS
// ============================================================================

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_fan_curve_interpolation_empty() {
        let nodes: Vec<FanCurveNode> = vec![];
        assert_eq!(interpolate_fan_speed(&nodes, 50.0), 30.0);
    }

    #[test]
    fn test_fan_curve_interpolation_single_node() {
        let nodes = vec![FanCurveNode::new(50.0, 60.0)];
        assert_eq!(interpolate_fan_speed(&nodes, 30.0), 60.0);
        assert_eq!(interpolate_fan_speed(&nodes, 70.0), 60.0);
    }

    #[test]
    fn test_fan_curve_interpolation_bounds() {
        let nodes = vec![
            FanCurveNode::new(30.0, 20.0),
            FanCurveNode::new(50.0, 40.0),
            FanCurveNode::new(80.0, 90.0),
        ];

        // Below lowest
        assert_eq!(interpolate_fan_speed(&nodes, 20.0), 20.0);
        assert_eq!(interpolate_fan_speed(&nodes, 30.0), 20.0);

        // Above highest
        assert_eq!(interpolate_fan_speed(&nodes, 80.0), 90.0);
        assert_eq!(interpolate_fan_speed(&nodes, 95.0), 90.0);
    }

    #[test]
    fn test_fan_curve_interpolation_linear() {
        let nodes = vec![
            FanCurveNode::new(40.0, 30.0),
            FanCurveNode::new(60.0, 50.0),
            FanCurveNode::new(80.0, 90.0),
        ];

        // Midpoint between 40 and 60 -> temp 50 -> speed 40
        let speed = interpolate_fan_speed(&nodes, 50.0);
        assert!((speed - 40.0).abs() < 1e-4);

        // Quarter between 40 and 60 -> temp 45 -> speed 35
        let speed_quarter = interpolate_fan_speed(&nodes, 45.0);
        assert!((speed_quarter - 35.0).abs() < 1e-4);

        // Midpoint between 60 and 80 -> temp 70 -> speed 70
        let speed2 = interpolate_fan_speed(&nodes, 70.0);
        assert!((speed2 - 70.0).abs() < 1e-4);
    }

    #[test]
    fn test_fan_curve_interpolation_unsorted_input() {
        // Curve nodes supplied out of order should be handled properly
        let nodes = vec![
            FanCurveNode::new(80.0, 100.0),
            FanCurveNode::new(40.0, 20.0),
            FanCurveNode::new(60.0, 60.0),
        ];

        let speed = interpolate_fan_speed(&nodes, 50.0);
        assert!((speed - 40.0).abs() < 1e-4);
    }

    #[test]
    fn test_parse_cpu_usage_stat() {
        let mut ticks = (0, 0);
        let stat_t1 = "cpu  1000 0 500 8500 0 0 0 0\ncpu0 500 0 ...";
        let usage1 = parse_cpu_usage_stat(stat_t1, &mut ticks);
        assert_eq!(usage1, Some(0)); // First sample has no delta yet

        let stat_t2 = "cpu  1100 0 550 8550 0 0 0 0\n";
        // delta: user + system = 150, idle = 50, total delta = 200
        // usage = (1 - 50/200) * 100 = 75%
        let usage2 = parse_cpu_usage_stat(stat_t2, &mut ticks);
        assert_eq!(usage2, Some(75));
    }

    #[test]
    fn test_parse_nvidia_smi() {
        let sample = "1200, 45, 65\n";
        let res = parse_nvidia_smi_output(sample);
        assert_eq!(res, Some((1200, 45, 65)));

        let bad_sample = "NVIDIA-SMI has failed because...";
        assert_eq!(parse_nvidia_smi_output(bad_sample), None);
    }

    #[test]
    fn test_parse_meminfo() {
        let sample = "\
MemTotal:       16384000 kB
MemFree:         4096000 kB
MemAvailable:    8192000 kB
";
        let res = parse_meminfo(sample);
        assert!(res.is_some());
        let (used_gb, total_gb, pct) = res.unwrap();
        assert!((total_gb - 15.6).abs() < 0.2);
        assert!((used_gb - 7.8).abs() < 0.2);
        assert_eq!(pct, 50);
    }

    #[test]
    fn test_read_storage_stats_root() {
        let stats = read_storage_stats("/");
        assert!(stats.is_some());
        let (used, total, pct) = stats.unwrap();
        assert!(total > 0);
        assert!(pct <= 100);
        assert!(used <= total);
    }

    #[test]
    fn test_power_mode_str() {
        assert_eq!(PowerMode::Quiet.as_powerprofiles_str(), "power-saver");
        assert_eq!(PowerMode::Entertainment.as_powerprofiles_str(), "balanced");
        assert_eq!(PowerMode::Performance.as_powerprofiles_str(), "performance");
    }

    #[test]
    fn test_live_telemetry_polling() {
        let mut controller = HardwareController::new();
        let cpu_curve = vec![FanCurveNode::new(40.0, 30.0), FanCurveNode::new(80.0, 90.0)];
        let gpu_curve = vec![FanCurveNode::new(40.0, 30.0), FanCurveNode::new(80.0, 90.0)];
        let telem = controller.poll_telemetry(FanMode::Automatic, &cpu_curve, &gpu_curve);
        println!("Live Telemetry: {:#?}", telem);
        assert!(telem.disk_total_gb > 0);
        assert!(telem.mem_total_gb > 0.0);
    }
}
