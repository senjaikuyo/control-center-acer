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
// FAN CURVE INTERPOLATION (STEPPED THRESHOLD MODEL)
// Kipas hanya berubah saat menyentuh titik threshold (2 titik)
// Mencegah fan cycling / hunting (tidak naik-turun setiap suhu bergeser 1°C)
// ============================================================================

/// Evaluates targeted fan duty percentage (0.0 to 100.0) using a stepped threshold model.
/// The fan maintains a stable duty until the temperature reaches the next threshold point.
pub fn interpolate_fan_speed(nodes: &[FanCurveNode], temp: f32) -> f32 {
    if temp.is_nan() || nodes.is_empty() {
        return 25.0;
    }

    if nodes.len() == 1 {
        return nodes[0].speed.clamp(0.0, 100.0);
    }

    let mut sorted_nodes = nodes.to_vec();
    sorted_nodes.sort_by(|a, b| a.temp.partial_cmp(&b.temp).unwrap_or(std::cmp::Ordering::Equal));

    // Di bawah ambang batas pertama: kecepatan idle hening
    if temp < sorted_nodes[0].temp {
        let base_speed = (sorted_nodes[0].speed / 2.0).min(25.0);
        return base_speed.clamp(0.0, 100.0);
    }

    // Model berundak (stepped threshold): temukan ambang tertinggi yang sudah tercapai
    for i in (0..sorted_nodes.len()).rev() {
        if temp >= sorted_nodes[i].temp {
            return sorted_nodes[i].speed.clamp(0.0, 100.0);
        }
    }

    sorted_nodes[0].speed.clamp(0.0, 100.0)
}

/// Default 2-point stepped fan curve for CPU.
pub fn default_cpu_curve() -> Vec<FanCurveNode> {
    vec![
        FanCurveNode { temp: 54.0, speed: 60.0 },
        FanCurveNode { temp: 75.0, speed: 85.0 },
    ]
}

/// Default 2-point stepped fan curve for GPU.
pub fn default_gpu_curve() -> Vec<FanCurveNode> {
    vec![
        FanCurveNode { temp: 55.0, speed: 50.0 },
        FanCurveNode { temp: 75.0, speed: 85.0 },
    ]
}

/// Parses CPU and GPU curve node lists from a string configuration.
pub fn parse_curve_nodes_from_str(content: &str) -> Option<(Vec<FanCurveNode>, Vec<FanCurveNode>)> {
    let mut cpu_nodes = Vec::new();
    let mut gpu_nodes = Vec::new();
    let mut current_section = "";

    for line in content.lines() {
        let line = line.trim();
        if line.is_empty() || line.starts_with('#') {
            continue;
        }
        if line.eq_ignore_ascii_case("[cpu]") {
            current_section = "cpu";
            continue;
        } else if line.eq_ignore_ascii_case("[gpu]") {
            current_section = "gpu";
            continue;
        }

        if let Some((temp_str, speed_str)) = line.split_once(',') {
            if let (Ok(temp), Ok(speed)) = (
                temp_str.trim().parse::<f32>(),
                speed_str.trim().parse::<f32>(),
            ) {
                let node = FanCurveNode::new(temp.clamp(20.0, 100.0), speed.clamp(0.0, 100.0));
                if current_section == "cpu" {
                    cpu_nodes.push(node);
                } else if current_section == "gpu" {
                    gpu_nodes.push(node);
                }
            }
        }
    }

    if cpu_nodes.len() >= 2 && gpu_nodes.len() >= 2 {
        cpu_nodes.sort_by(|a, b| a.temp.partial_cmp(&b.temp).unwrap_or(std::cmp::Ordering::Equal));
        gpu_nodes.sort_by(|a, b| a.temp.partial_cmp(&b.temp).unwrap_or(std::cmp::Ordering::Equal));
        Some((cpu_nodes, gpu_nodes))
    } else {
        None
    }
}

/// Serializes CPU and GPU curves into a human-readable configuration string.
pub fn serialize_curve_nodes(cpu: &[FanCurveNode], gpu: &[FanCurveNode]) -> String {
    let mut out = String::from("# Acer Control Center Fan Curves\n[cpu]\n");
    for node in cpu {
        out.push_str(&format!("{:.1},{:.1}\n", node.temp, node.speed));
    }
    out.push_str("\n[gpu]\n");
    for node in gpu {
        out.push_str(&format!("{:.1},{:.1}\n", node.temp, node.speed));
    }
    out
}

/// Formats fan curve nodes into Tailord JSON format: `[{"temp": X, "fan": Y}, ...]`.
/// Nodes are sorted by temperature ascending and clamped to safe bounds (`temp`: 0..=255, `fan`: 0..=100).
pub fn fan_curve_to_json(nodes: &[FanCurveNode]) -> String {
    let mut sorted = nodes.to_vec();
    sorted.sort_by(|a, b| a.temp.partial_cmp(&b.temp).unwrap_or(std::cmp::Ordering::Equal));

    if sorted.len() < 2 {
        let items: Vec<String> = sorted
            .iter()
            .map(|node| {
                let temp = (node.temp.round() as i32).clamp(0, 255) as u8;
                let fan = (node.speed.round() as i32).clamp(0, 100) as u8;
                format!(r#"{{"temp":{},"fan":{}}}"#, temp, fan)
            })
            .collect();
        return format!("[{}]", items.join(","));
    }

    let p1 = &sorted[0];
    let p2 = &sorted[sorted.len() - 1];

    let t1 = (p1.temp.round() as i32).clamp(30, 90) as u8;
    let f1 = (p1.speed.round() as i32).clamp(10, 100) as u8;

    let t2 = (p2.temp.round() as i32).clamp((t1 + 2) as i32, 95) as u8;
    let f2 = (p2.speed.round() as i32).clamp(f1 as i32, 100) as u8;

    let f_base = (f1 / 2).min(25);

    let mut items = Vec::new();
    if t1 > 28 {
        items.push(format!(r#"{{"temp":25,"fan":{}}}"#, f_base));
        if t1 > 30 {
            items.push(format!(r#"{{"temp":{},"fan":{}}}"#, t1 - 1, f_base));
        }
    }

    items.push(format!(r#"{{"temp":{},"fan":{}}}"#, t1, f1));

    if t2 > t1 + 1 {
        items.push(format!(r#"{{"temp":{},"fan":{}}}"#, t2 - 1, f1));
    }

    items.push(format!(r#"{{"temp":{},"fan":{}}}"#, t2, f2));

    if t2 < 100 {
        items.push(format!(r#"{{"temp":100,"fan":100}}"#));
    }

    format!("[{}]", items.join(","))
}

/// Constructs a full Tailord system profile JSON string for `com.tux.Tailor.Profiles AddProfile`.
/// If `template_json` is provided, it replaces the `"fans"` array while preserving other settings (LEDs, etc.).
pub fn build_tailord_profile_json(cpu_prof: &str, gpu_prof: &str, template_json: Option<&str>) -> String {
    let fallback = format!(
        r#"{{"fans":["{}","{}"],"leds":[{{"device_name":"platform:tuxedo_keyboard","function":"kbd_backlight","profile":"default","mode":"Rgb"}}],"performance_profile":"performance"}}"#,
        cpu_prof, gpu_prof
    );

    if let Some(template) = template_json {
        if let Some(fans_pos) = template.find("\"fans\"") {
            if let Some(open_bracket) = template[fans_pos..].find('[') {
                let start_idx = fans_pos + open_bracket;
                if let Some(close_bracket) = template[start_idx..].find(']') {
                    let end_idx = start_idx + close_bracket + 1;
                    let mut res = String::with_capacity(template.len() + 32);
                    res.push_str(&template[..start_idx]);
                    res.push_str(&format!(r#"["{}","{}"]"#, cpu_prof, gpu_prof));
                    res.push_str(&template[end_idx..]);
                    return res;
                }
            }
        }
    }

    fallback
}

/// Sets the active system profile in Tailord via D-Bus (`com.tux.Tailor.Profiles SetActiveProfileName`).
pub fn set_tailord_active_profile(name: &str) -> Result<(), BackendError> {
    let out = Command::new("busctl")
        .args([
            "call",
            "com.tux.Tailor",
            "/com/tux/Tailor",
            "com.tux.Tailor.Profiles",
            "SetActiveProfileName",
            "s",
            name,
        ])
        .output()
        .map_err(|e| BackendError::DbusError(format!("busctl SetActiveProfileName: {}", e)))?;

    if !out.status.success() {
        let err = String::from_utf8_lossy(&out.stderr);
        return Err(BackendError::DbusError(format!("Failed to activate profile: {}", err)));
    }
    Ok(())
}

/// Helper to parse string output returned by `busctl call ...` (format: `s "..."`).
pub fn parse_busctl_string(raw: &str) -> Option<String> {
    let trimmed = raw.trim();
    let prefix = "s \"";
    if trimmed.starts_with(prefix) && trimmed.ends_with('"') && trimmed.len() >= 4 {
        let inner = &trimmed[prefix.len()..trimmed.len() - 1];
        Some(inner.replace(r#"\""#, "\""))
    } else {
        None
    }
}

/// Fetches the raw JSON of the currently active profile from Tailord D-Bus if available.
pub fn get_tailord_active_profile_raw() -> Option<String> {
    let out = Command::new("busctl")
        .args([
            "call",
            "com.tux.Tailor",
            "/com/tux/Tailor",
            "com.tux.Tailor.Profiles",
            "GetActiveProfileName",
        ])
        .output()
        .ok()?;

    if !out.status.success() {
        return None;
    }

    let stdout_name = String::from_utf8_lossy(&out.stdout);
    let active_name = parse_busctl_string(&stdout_name)?;

    let out_p = Command::new("busctl")
        .args([
            "call",
            "com.tux.Tailor",
            "/com/tux/Tailor",
            "com.tux.Tailor.Profiles",
            "GetProfile",
            "s",
            &active_name,
        ])
        .output()
        .ok()?;

    if !out_p.status.success() {
        return None;
    }

    let stdout_p = String::from_utf8_lossy(&out_p.stdout);
    parse_busctl_string(&stdout_p)
}

/// Helper to remove Tailord profiles created during tests or session resets.
pub fn remove_tailord_profile(name: &str) {
    let _ = Command::new("busctl")
        .args(["call", "com.tux.Tailor", "/com/tux/Tailor", "com.tux.Tailor.Profiles", "RemoveProfile", "s", name])
        .output();
    let _ = Command::new("busctl")
        .args(["call", "com.tux.Tailor", "/com/tux/Tailor", "com.tux.Tailor.Fan", "RemoveProfile", "s", name])
        .output();
    let _ = Command::new("busctl")
        .args(["call", "com.tux.Tailor", "/com/tux/Tailor", "com.tux.Tailor.Fan", "RemoveProfile", "s", &format!("{}_cpu", name)])
        .output();
    let _ = Command::new("busctl")
        .args(["call", "com.tux.Tailor", "/com/tux/Tailor", "com.tux.Tailor.Fan", "RemoveProfile", "s", &format!("{}_gpu", name)])
        .output();
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

/// Sets system power profile via direct D-Bus `net.hadess.PowerProfiles ActiveProfile`,
/// with fallback to `/usr/bin/python3 /usr/bin/powerprofilesctl` or PATH `powerprofilesctl`.
pub fn set_power_profile(mode: PowerMode) -> Result<(), BackendError> {
    let profile = mode.as_powerprofiles_str();

    // 1. Direct system D-Bus call to net.hadess.PowerProfiles ActiveProfile (fast, zero python dependency)
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
        .output();

    if let Ok(out) = dbus_res {
        if out.status.success() {
            return Ok(());
        }
    }

    // 2. Fallback: Try python3 /usr/bin/powerprofilesctl set <profile>
    let py_res = Command::new("/usr/bin/python3")
        .args(["/usr/bin/powerprofilesctl", "set", profile])
        .output();

    if let Ok(out) = py_res {
        if out.status.success() {
            return Ok(());
        }
    }

    // 3. Fallback: Try powerprofilesctl directly from PATH
    let direct_res = Command::new("powerprofilesctl")
        .args(["set", profile])
        .output();

    if let Ok(out) = direct_res {
        if out.status.success() {
            return Ok(());
        }
    }

    Ok(())
}

/// Reads the currently active system power profile via D-Bus `net.hadess.PowerProfiles ActiveProfile`.
pub fn get_active_power_profile() -> Option<PowerMode> {
    let out = Command::new("busctl")
        .args([
            "get-property",
            "net.hadess.PowerProfiles",
            "/net/hadess/PowerProfiles",
            "net.hadess.PowerProfiles",
            "ActiveProfile",
        ])
        .output()
        .ok()?;

    if out.status.success() {
        let s = String::from_utf8_lossy(&out.stdout);
        if s.contains("power-saver") {
            Some(PowerMode::Quiet)
        } else if s.contains("performance") {
            Some(PowerMode::Performance)
        } else if s.contains("balanced") {
            Some(PowerMode::Entertainment)
        } else {
            None
        }
    } else {
        None
    }
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
    pub last_dispatched_cpu_duty: Option<u8>,
    pub last_dispatched_gpu_duty: Option<u8>,
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
            last_dispatched_cpu_duty: None,
            last_dispatched_gpu_duty: None,
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
        let prev_fan_mode = self.fan_mode;
        self.fan_mode = fan_mode;

        // When switching back to Automatic, reliably reset fan override to 0
        if prev_fan_mode != fan_mode && fan_mode == FanMode::Automatic {
            let _ = apply_clevo_fan_all(0);
            self.last_dispatched_cpu_duty = Some(0);
            self.last_dispatched_gpu_duty = Some(0);
        }

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
                if prev_fan_mode != fan_mode {
                    let _ = apply_clevo_fan_all(100);
                    self.last_dispatched_cpu_duty = Some(100);
                    self.last_dispatched_gpu_duty = Some(100);
                }
            }
            FanMode::Turbo => {
                self.telemetry.cpu_fan_pct = 100;
                self.telemetry.gpu_fan_pct = 100;
                self.telemetry.cpu_fan_rpm = 5400;
                self.telemetry.gpu_fan_rpm = 5200;
                if prev_fan_mode != fan_mode {
                    let _ = apply_clevo_fan_all(100);
                    self.last_dispatched_cpu_duty = Some(100);
                    self.last_dispatched_gpu_duty = Some(100);
                }
            }
            FanMode::Custom => {
                let cpu_speed = interpolate_fan_speed(cpu_curve_nodes, self.telemetry.cpu_temp as f32);
                let gpu_speed = interpolate_fan_speed(gpu_curve_nodes, self.telemetry.gpu_temp as f32);

                let cpu_pct = (cpu_speed.round() as u32).clamp(0, 100);
                let gpu_pct = (gpu_speed.round() as u32).clamp(0, 100);

                self.telemetry.cpu_fan_pct = cpu_pct;
                self.telemetry.cpu_fan_rpm = (cpu_pct as f32 / 100.0 * 5000.0) as u32;

                self.telemetry.gpu_fan_pct = gpu_pct;
                self.telemetry.gpu_fan_rpm = (gpu_pct as f32 / 100.0 * 4800.0) as u32;
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
                self.last_dispatched_cpu_duty = Some(0);
                self.last_dispatched_gpu_duty = Some(0);
                let _ = set_tailord_active_profile("default");
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
                self.last_dispatched_cpu_duty = Some(0);
                self.last_dispatched_gpu_duty = Some(0);
                let _ = set_tailord_active_profile("default");
                apply_clevo_fan_all(0)?;
            }
            FanMode::Maximum => {
                self.telemetry.cpu_fan_pct = 100;
                self.telemetry.gpu_fan_pct = 100;
                self.telemetry.cpu_fan_rpm = 5100;
                self.telemetry.gpu_fan_rpm = 4900;
                self.last_dispatched_cpu_duty = Some(100);
                self.last_dispatched_gpu_duty = Some(100);
                apply_clevo_fan_all(100)?;
            }
            FanMode::Turbo => {
                self.telemetry.cpu_fan_pct = 100;
                self.telemetry.gpu_fan_pct = 100;
                self.telemetry.cpu_fan_rpm = 5400;
                self.telemetry.gpu_fan_rpm = 5200;
                self.last_dispatched_cpu_duty = Some(100);
                self.last_dispatched_gpu_duty = Some(100);
                apply_clevo_fan_all(100)?;
            }
            FanMode::Custom => {
                let cpu_speed = interpolate_fan_speed(cpu_curve_nodes, self.telemetry.cpu_temp as f32);
                let gpu_speed = interpolate_fan_speed(gpu_curve_nodes, self.telemetry.gpu_temp as f32);

                let cpu_pct = (cpu_speed.round() as u32).clamp(0, 100);
                let gpu_pct = (gpu_speed.round() as u32).clamp(0, 100);

                self.telemetry.cpu_fan_pct = cpu_pct;
                self.telemetry.cpu_fan_rpm = (cpu_pct as f32 / 100.0 * 5000.0) as u32;

                self.telemetry.gpu_fan_pct = gpu_pct;
                self.telemetry.gpu_fan_rpm = (gpu_pct as f32 / 100.0 * 4800.0) as u32;

                let _ = self.sync_curve_to_tailord("acer_custom", cpu_curve_nodes, gpu_curve_nodes);
                let _ = apply_clevo_fan_all(0);
            }
        }
        Ok(())
    }

    /// Evaluates custom fan curves at current temperatures and dispatches to hardware if FanMode::Custom.
    pub fn apply_custom_curve(
        &mut self,
        cpu_curve_nodes: &[FanCurveNode],
        gpu_curve_nodes: &[FanCurveNode],
    ) -> Result<(u32, u32), BackendError> {
        let cpu_speed = interpolate_fan_speed(cpu_curve_nodes, self.telemetry.cpu_temp as f32);
        let gpu_speed = interpolate_fan_speed(gpu_curve_nodes, self.telemetry.gpu_temp as f32);

        let cpu_pct = (cpu_speed.round() as u32).clamp(0, 100);
        let gpu_pct = (gpu_speed.round() as u32).clamp(0, 100);

        self.telemetry.cpu_fan_pct = cpu_pct;
        self.telemetry.cpu_fan_rpm = (cpu_pct as f32 / 100.0 * 5000.0) as u32;

        self.telemetry.gpu_fan_pct = gpu_pct;
        self.telemetry.gpu_fan_rpm = (gpu_pct as f32 / 100.0 * 4800.0) as u32;

        if self.fan_mode == FanMode::Custom {
            let _ = self.sync_curve_to_tailord("acer_custom", cpu_curve_nodes, gpu_curve_nodes);
            let _ = apply_clevo_fan_all(0);
        }

        Ok((cpu_pct, gpu_pct))
    }

    /// Dispatches fan duties to hardware (CPU fan 0, GPU fan 1) if they changed from last dispatched.
    /// Clamps duties to [0, 100] and avoids hardware spam if identical.
    /// Returns `(cpu_dispatched, gpu_dispatched)`.
    pub fn dispatch_fan_duties(&mut self, cpu_duty: u8, gpu_duty: u8) -> (bool, bool) {
        let cpu_clamped = cpu_duty.min(100);
        let gpu_clamped = gpu_duty.min(100);

        let mut cpu_dispatched = false;
        let mut gpu_dispatched = false;

        if self.last_dispatched_cpu_duty != Some(cpu_clamped) {
            let _ = apply_clevo_fan(0, cpu_clamped);
            self.last_dispatched_cpu_duty = Some(cpu_clamped);
            cpu_dispatched = true;
        }

        if self.last_dispatched_gpu_duty != Some(gpu_clamped) {
            let _ = apply_clevo_fan(1, gpu_clamped);
            self.last_dispatched_gpu_duty = Some(gpu_clamped);
            gpu_dispatched = true;
        }

        (cpu_dispatched, gpu_dispatched)
    }

    /// Syncs custom fan curves to the `tailord` daemon via D-Bus:
    /// - Adds Fan profile(s) on `com.tux.Tailor.Fan`
    /// - Registers profile on `com.tux.Tailor.Profiles`
    /// - Sets the profile as active on `com.tux.Tailor.Profiles`
    pub fn sync_curve_to_tailord(
        &self,
        name: &str,
        cpu_nodes: &[FanCurveNode],
        gpu_nodes: &[FanCurveNode],
    ) -> Result<(), BackendError> {
        let cpu_json = fan_curve_to_json(cpu_nodes);
        let gpu_json = fan_curve_to_json(gpu_nodes);

        // 1. Add profile on com.tux.Tailor.Fan
        let out_main = Command::new("busctl")
            .args([
                "call",
                "com.tux.Tailor",
                "/com/tux/Tailor",
                "com.tux.Tailor.Fan",
                "AddProfile",
                "ss",
                name,
                &cpu_json,
            ])
            .output()?;

        if !out_main.status.success() {
            let stderr = String::from_utf8_lossy(&out_main.stderr).to_string();
            return Err(BackendError::CommandFailed {
                command: format!("busctl call com.tux.Tailor.Fan AddProfile ss {}", name),
                message: stderr,
            });
        }

        let (cpu_prof_name, gpu_prof_name) = if cpu_nodes == gpu_nodes {
            (name.to_string(), name.to_string())
        } else {
            let cpu_p = format!("{}_cpu", name);
            let gpu_p = format!("{}_gpu", name);

            let out_cpu = Command::new("busctl")
                .args([
                    "call",
                    "com.tux.Tailor",
                    "/com/tux/Tailor",
                    "com.tux.Tailor.Fan",
                    "AddProfile",
                    "ss",
                    &cpu_p,
                    &cpu_json,
                ])
                .output()?;

            if !out_cpu.status.success() {
                let stderr = String::from_utf8_lossy(&out_cpu.stderr).to_string();
                return Err(BackendError::CommandFailed {
                    command: format!("busctl call com.tux.Tailor.Fan AddProfile ss {}", cpu_p),
                    message: stderr,
                });
            }

            let out_gpu = Command::new("busctl")
                .args([
                    "call",
                    "com.tux.Tailor",
                    "/com/tux/Tailor",
                    "com.tux.Tailor.Fan",
                    "AddProfile",
                    "ss",
                    &gpu_p,
                    &gpu_json,
                ])
                .output()?;

            if !out_gpu.status.success() {
                let stderr = String::from_utf8_lossy(&out_gpu.stderr).to_string();
                return Err(BackendError::CommandFailed {
                    command: format!("busctl call com.tux.Tailor.Fan AddProfile ss {}", gpu_p),
                    message: stderr,
                });
            }

            (cpu_p, gpu_p)
        };

        // 2. Query active profile template if available
        let template_json = get_tailord_active_profile_raw();
        let profile_json = build_tailord_profile_json(&cpu_prof_name, &gpu_prof_name, template_json.as_deref());

        // 3. Add system profile on com.tux.Tailor.Profiles
        let out_prof = Command::new("busctl")
            .args([
                "call",
                "com.tux.Tailor",
                "/com/tux/Tailor",
                "com.tux.Tailor.Profiles",
                "AddProfile",
                "ss",
                name,
                &profile_json,
            ])
            .output()?;

        if !out_prof.status.success() {
            let stderr = String::from_utf8_lossy(&out_prof.stderr).to_string();
            return Err(BackendError::CommandFailed {
                command: format!("busctl call com.tux.Tailor.Profiles AddProfile ss {}", name),
                message: stderr,
            });
        }

        // 4. Update active profile name on com.tux.Tailor.Profiles
        let out_active = Command::new("busctl")
            .args([
                "call",
                "com.tux.Tailor",
                "/com/tux/Tailor",
                "com.tux.Tailor.Profiles",
                "SetActiveProfileName",
                "s",
                name,
            ])
            .output()?;

        if !out_active.status.success() {
            let stderr = String::from_utf8_lossy(&out_active.stderr).to_string();
            return Err(BackendError::CommandFailed {
                command: format!("busctl call com.tux.Tailor.Profiles SetActiveProfileName s {}", name),
                message: stderr,
            });
        }

        Ok(())
    }

    /// Directly overrides fan speed on a specific fan index (0 for CPU, 1 for GPU).
    pub fn override_fan(&self, fan_index: u8, speed: u8) -> Result<(), BackendError> {
        apply_clevo_fan(fan_index, speed)
    }

    /// Sets touchpad state via KWin Wayland D-Bus interface.
    pub fn set_touchpad_enabled(&self, enabled: bool) {
        let _ = Command::new("busctl")
            .args([
                "--user",
                "set-property",
                "org.kde.KWin",
                "/org/kde/KWin/InputDevice/event14",
                "org.kde.KWin.InputDevice",
                "enabled",
                "b",
                if enabled { "true" } else { "false" },
            ])
            .output();
    }

    /// Sets airplane mode via rfkill.
    pub fn set_airplane_mode(&self, enabled: bool) {
        let arg = if enabled { "block" } else { "unblock" };
        let _ = Command::new("rfkill").args([arg, "all"]).output();
    }

    /// Sets night light / blue light filter on KWin Wayland.
    pub fn set_night_light(&self, enabled: bool, level: f32) {
        if enabled {
            // Level 0.0 to 10.0 maps from 6500K down to 3500K warm
            let temp = (6500.0 - (level.clamp(0.0, 10.0) * 300.0)) as u32;
            let _ = Command::new("busctl")
                .args([
                    "--user",
                    "call",
                    "org.kde.KWin.NightLight",
                    "/org/kde/KWin/NightLight",
                    "org.kde.KWin.NightLight",
                    "preview",
                    "u",
                    &temp.to_string(),
                ])
                .output();
        } else {
            let _ = Command::new("busctl")
                .args([
                    "--user",
                    "call",
                    "org.kde.KWin.NightLight",
                    "/org/kde/KWin/NightLight",
                    "org.kde.KWin.NightLight",
                    "stopPreview",
                ])
                .output();
        }
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
        assert_eq!(interpolate_fan_speed(&nodes, 50.0), 25.0);
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

        // Below lowest: base quiet duty
        assert_eq!(interpolate_fan_speed(&nodes, 20.0), 10.0);

        // At lowest threshold
        assert_eq!(interpolate_fan_speed(&nodes, 30.0), 20.0);

        // Above highest
        assert_eq!(interpolate_fan_speed(&nodes, 80.0), 90.0);
        assert_eq!(interpolate_fan_speed(&nodes, 95.0), 90.0);
    }

    #[test]
    fn test_fan_curve_interpolation_stepped() {
        let nodes = vec![
            FanCurveNode::new(40.0, 30.0),
            FanCurveNode::new(60.0, 50.0),
            FanCurveNode::new(80.0, 90.0),
        ];

        // Antara 40 dan 60 -> temp 45, 50, 55 -> kecepatan stabil di 30 (tidak naik turun per derajat)
        assert_eq!(interpolate_fan_speed(&nodes, 45.0), 30.0);
        assert_eq!(interpolate_fan_speed(&nodes, 50.0), 30.0);
        assert_eq!(interpolate_fan_speed(&nodes, 59.9), 30.0);

        // Masuk ambang 60 -> loncat ke 50 dan stabil
        assert_eq!(interpolate_fan_speed(&nodes, 60.0), 50.0);
        assert_eq!(interpolate_fan_speed(&nodes, 70.0), 50.0);

        // Masuk ambang 80 -> loncat ke 90
        assert_eq!(interpolate_fan_speed(&nodes, 80.0), 90.0);
        assert_eq!(interpolate_fan_speed(&nodes, 95.0), 90.0);
    }

    #[test]
    fn test_fan_curve_interpolation_unsorted_input() {
        // Curve nodes supplied out of order should be handled properly
        let nodes = vec![
            FanCurveNode::new(80.0, 100.0),
            FanCurveNode::new(40.0, 20.0),
            FanCurveNode::new(60.0, 60.0),
        ];

        // Temp 50 is between 40 and 60 -> returns 20.0 (stepped)
        let speed = interpolate_fan_speed(&nodes, 50.0);
        assert_eq!(speed, 20.0);
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

    #[test]
    fn test_default_curves_validity() {
        let cpu_nodes = default_cpu_curve();
        let gpu_nodes = default_gpu_curve();

        assert_eq!(cpu_nodes.len(), 2);
        assert_eq!(gpu_nodes.len(), 2);

        for window in cpu_nodes.windows(2) {
            assert!(window[0].temp < window[1].temp);
            assert!(window[0].speed <= window[1].speed);
            assert!(window[0].temp >= 20.0 && window[0].temp <= 100.0);
            assert!(window[0].speed >= 0.0 && window[0].speed <= 100.0);
        }

        for window in gpu_nodes.windows(2) {
            assert!(window[0].temp < window[1].temp);
            assert!(window[0].temp >= 20.0 && window[0].temp <= 100.0);
            assert!(window[0].speed >= 0.0 && window[0].speed <= 100.0);
        }
    }

    #[test]
    fn test_serialize_and_parse_curves() {
        let cpu_original = default_cpu_curve();
        let gpu_original = default_gpu_curve();

        let serialized = serialize_curve_nodes(&cpu_original, &gpu_original);
        assert!(serialized.contains("[cpu]"));
        assert!(serialized.contains("[gpu]"));

        let parsed = parse_curve_nodes_from_str(&serialized);
        assert!(parsed.is_some());
        let (cpu_parsed, gpu_parsed) = parsed.unwrap();

        assert_eq!(cpu_parsed.len(), cpu_original.len());
        assert_eq!(gpu_parsed.len(), gpu_original.len());
        for (orig, parsed) in cpu_original.iter().zip(cpu_parsed.iter()) {
            assert!((orig.temp - parsed.temp).abs() < 0.1);
            assert!((orig.speed - parsed.speed).abs() < 0.1);
        }
    }

    #[test]
    fn test_parse_malformed_curve_config() {
        assert!(parse_curve_nodes_from_str("").is_none());
        assert!(parse_curve_nodes_from_str("garbage content").is_none());
        assert!(parse_curve_nodes_from_str("[cpu]\n10,20\n[gpu]").is_none());
    }

    #[test]
    fn test_controller_apply_custom_curve() {
        let mut controller = HardwareController::new();
        controller.telemetry.cpu_temp = 50;
        controller.telemetry.gpu_temp = 60;
        controller.fan_mode = FanMode::Custom;

        let cpu_nodes = vec![
            FanCurveNode::new(40.0, 30.0),
            FanCurveNode::new(60.0, 50.0),
        ];
        let gpu_nodes = vec![
            FanCurveNode::new(50.0, 40.0),
            FanCurveNode::new(70.0, 80.0),
        ];

        let res = controller.apply_custom_curve(&cpu_nodes, &gpu_nodes);
        assert!(res.is_ok());
        let (cpu_duty, gpu_duty) = res.unwrap();
        // Stepped: CPU 50C >= 40 and < 60 -> 30; GPU 60C >= 50 and < 70 -> 40
        assert_eq!(cpu_duty, 30);
        assert_eq!(gpu_duty, 40);
        assert_eq!(controller.telemetry.cpu_fan_pct, 30);
        assert_eq!(controller.telemetry.gpu_fan_pct, 40);
    }

    #[test]
    fn test_set_fan_mode_telemetry() {
        let mut controller = HardwareController::new();
        controller.telemetry.cpu_temp = 60;
        controller.telemetry.gpu_temp = 55;

        let cpu_nodes = vec![FanCurveNode::new(50.0, 40.0), FanCurveNode::new(70.0, 60.0)];
        let gpu_nodes = vec![FanCurveNode::new(50.0, 30.0), FanCurveNode::new(70.0, 70.0)];

        let _ = controller.set_fan_mode(FanMode::Maximum, &cpu_nodes, &gpu_nodes);
        assert_eq!(controller.telemetry.cpu_fan_pct, 100);
        assert_eq!(controller.telemetry.gpu_fan_pct, 100);

        let _ = controller.set_fan_mode(FanMode::Turbo, &cpu_nodes, &gpu_nodes);
        assert_eq!(controller.telemetry.cpu_fan_pct, 100);
        assert_eq!(controller.telemetry.gpu_fan_pct, 100);

        let _ = controller.set_fan_mode(FanMode::Custom, &cpu_nodes, &gpu_nodes);
        // Stepped: CPU 60C >= 50 and < 70 -> 40; GPU 55C >= 50 and < 70 -> 30
        assert_eq!(controller.telemetry.cpu_fan_pct, 40);
        assert_eq!(controller.telemetry.gpu_fan_pct, 30);
    }

    #[test]
    fn test_dynamic_duty_dispatch_logic() {
        let mut controller = HardwareController::new();
        assert_eq!(controller.last_dispatched_cpu_duty, None);
        assert_eq!(controller.last_dispatched_gpu_duty, None);

        // First dispatch: both should be dispatched
        let (cpu_sent, gpu_sent) = controller.dispatch_fan_duties(45, 55);
        assert!(cpu_sent);
        assert!(gpu_sent);
        assert_eq!(controller.last_dispatched_cpu_duty, Some(45));
        assert_eq!(controller.last_dispatched_gpu_duty, Some(55));

        // Identical dispatch: avoid spamming hardware if identical
        let (cpu_sent, gpu_sent) = controller.dispatch_fan_duties(45, 55);
        assert!(!cpu_sent);
        assert!(!gpu_sent);

        // Only CPU changed: dispatch only CPU
        let (cpu_sent, gpu_sent) = controller.dispatch_fan_duties(50, 55);
        assert!(cpu_sent);
        assert!(!gpu_sent);
        assert_eq!(controller.last_dispatched_cpu_duty, Some(50));
        assert_eq!(controller.last_dispatched_gpu_duty, Some(55));

        // Only GPU changed: dispatch only GPU
        let (cpu_sent, gpu_sent) = controller.dispatch_fan_duties(50, 65);
        assert!(!cpu_sent);
        assert!(gpu_sent);
        assert_eq!(controller.last_dispatched_cpu_duty, Some(50));
        assert_eq!(controller.last_dispatched_gpu_duty, Some(65));

        // Bounds clamping to safe 0..=100 duty
        let (cpu_sent, gpu_sent) = controller.dispatch_fan_duties(150, 200);
        assert!(cpu_sent);
        assert!(gpu_sent);
        assert_eq!(controller.last_dispatched_cpu_duty, Some(100));
        assert_eq!(controller.last_dispatched_gpu_duty, Some(100));

        // Redundant clamped dispatch
        let (cpu_sent, gpu_sent) = controller.dispatch_fan_duties(120, 100);
        assert!(!cpu_sent);
        assert!(!gpu_sent);
    }

    #[test]
    fn test_tailord_json_serialization_and_profiles() {
        // Serialization of 2-point nodes to Tailord stepped JSON schema
        let nodes = vec![
            FanCurveNode::new(54.0, 60.0),
            FanCurveNode::new(75.0, 85.0),
        ];
        let json = fan_curve_to_json(&nodes);
        assert!(json.contains(r#"{"temp":54,"fan":60}"#));
        assert!(json.contains(r#"{"temp":75,"fan":85}"#));
        assert!(json.contains(r#"{"temp":100,"fan":100}"#));

        // Profile JSON generation with default fallback
        let prof_fallback = build_tailord_profile_json("custom_cpu", "custom_gpu", None);
        assert!(prof_fallback.contains(r#""fans":["custom_cpu","custom_gpu"]"#));
        assert!(prof_fallback.contains(r#""performance_profile":"performance""#));
        assert!(prof_fallback.contains(r#""leds""#));

        // Profile JSON generation with template preserving other settings
        let template = r#"{"fans":["old_cpu","old_gpu"],"leds":[{"device":"test"}],"performance_profile":"balanced"}"#;
        let prof_templated = build_tailord_profile_json("my_cpu", "my_gpu", Some(template));
        assert!(prof_templated.contains(r#""fans":["my_cpu","my_gpu"]"#));
        assert!(prof_templated.contains(r#"{"device":"test"}"#));
        assert!(prof_templated.contains(r#""performance_profile":"balanced""#));

        // Busctl output string parser
        let busctl_sample = "s \"default\"\n";
        assert_eq!(parse_busctl_string(busctl_sample), Some("default".to_string()));

        let escaped_json = "s \"{\\\"fans\\\":[\\\"default\\\",\\\"default\\\"]}\"\n";
        assert_eq!(
            parse_busctl_string(escaped_json),
            Some(r#"{"fans":["default","default"]}"#.to_string())
        );

        assert_eq!(parse_busctl_string("invalid"), None);
    }

    #[test]
    fn test_edge_cases_interpolation_and_serialization() {
        // 1. Empty nodes
        let empty_nodes: Vec<FanCurveNode> = vec![];
        assert_eq!(interpolate_fan_speed(&empty_nodes, 40.0), 25.0);
        assert_eq!(fan_curve_to_json(&empty_nodes), "[]");

        // 2. Single node
        let single_node = vec![FanCurveNode::new(55.0, 60.0)];
        assert_eq!(interpolate_fan_speed(&single_node, -10.0), 60.0);
        assert_eq!(interpolate_fan_speed(&single_node, 55.0), 60.0);
        assert_eq!(interpolate_fan_speed(&single_node, 120.0), 60.0);
        assert_eq!(fan_curve_to_json(&single_node), r#"[{"temp":55,"fan":60}]"#);

        // 3. Below minimum temperature
        let nodes = vec![
            FanCurveNode::new(40.0, 30.0),
            FanCurveNode::new(70.0, 75.0),
        ];
        assert_eq!(interpolate_fan_speed(&nodes, -20.0), 15.0);
        assert_eq!(interpolate_fan_speed(&nodes, 0.0), 15.0);
        assert_eq!(interpolate_fan_speed(&nodes, 39.9), 15.0);

        // 4. Above maximum temperature
        assert_eq!(interpolate_fan_speed(&nodes, 70.0), 75.0);
        assert_eq!(interpolate_fan_speed(&nodes, 95.0), 75.0);
        assert_eq!(interpolate_fan_speed(&nodes, 120.0), 75.0);

        // 5. Out of order nodes
        let unsorted_nodes = vec![
            FanCurveNode::new(70.0, 80.0),
            FanCurveNode::new(40.0, 30.0),
        ];
        // Between 40 and 70 -> temp 45 -> 30.0 (stepped)
        let speed_mid = interpolate_fan_speed(&unsorted_nodes, 45.0);
        assert_eq!(speed_mid, 30.0);

        // Serialization of out of order nodes should automatically sort by temp ascending
        let json_sorted = fan_curve_to_json(&unsorted_nodes);
        assert!(json_sorted.contains(r#"{"temp":40,"fan":30}"#));
        assert!(json_sorted.contains(r#"{"temp":70,"fan":80}"#));

        // 6. Out of bound values in serialization (clamped safely)
        let out_of_bounds = vec![
            FanCurveNode::new(-15.0, -5.0),
            FanCurveNode::new(105.0, 150.0),
        ];
        let json_clamped = fan_curve_to_json(&out_of_bounds);
        assert!(!json_clamped.is_empty());

        // 7. Float rounding in serialization
        let float_nodes = vec![
            FanCurveNode::new(45.4, 30.6),
            FanCurveNode::new(69.8, 80.2),
        ];
        let json_float = fan_curve_to_json(&float_nodes);
        assert!(json_float.contains(r#"{"temp":45,"fan":31}"#));
        assert!(json_float.contains(r#"{"temp":70,"fan":80}"#));

        // 8. NaN temperature
        assert_eq!(interpolate_fan_speed(&nodes, f32::NAN), 25.0);
    }

    #[test]
    fn test_fan_mode_switching_and_automatic_reset() {
        let mut controller = HardwareController::new();
        controller.telemetry.cpu_temp = 50;
        controller.telemetry.gpu_temp = 40;

        let cpu_nodes = vec![FanCurveNode::new(40.0, 35.0), FanCurveNode::new(60.0, 55.0)];
        let gpu_nodes = vec![FanCurveNode::new(40.0, 25.0), FanCurveNode::new(60.0, 45.0)];

        // 1. Immediately sets Custom mode
        let res = controller.set_fan_mode(FanMode::Custom, &cpu_nodes, &gpu_nodes);
        assert!(res.is_ok());
        assert_eq!(controller.fan_mode, FanMode::Custom);

        // 2. Switching back to Automatic resets fan mode
        let res_auto = controller.set_fan_mode(FanMode::Automatic, &cpu_nodes, &gpu_nodes);
        assert!(res_auto.is_ok());
        assert_eq!(controller.fan_mode, FanMode::Automatic);
        assert_eq!(controller.last_dispatched_cpu_duty, Some(0));
        assert_eq!(controller.last_dispatched_gpu_duty, Some(0));

        // 3. Switching to Maximum sets dispatched duty to Some(100)
        let res_max = controller.set_fan_mode(FanMode::Maximum, &cpu_nodes, &gpu_nodes);
        assert!(res_max.is_ok());
        assert_eq!(controller.last_dispatched_cpu_duty, Some(100));
        assert_eq!(controller.last_dispatched_gpu_duty, Some(100));

        // 4. Power mode Quiet switches to FanMode::Automatic and resets fan to Some(0)
        let _ = controller.set_power_mode(PowerMode::Quiet);
        assert_eq!(controller.fan_mode, FanMode::Automatic);
        assert_eq!(controller.last_dispatched_cpu_duty, Some(0));
        assert_eq!(controller.last_dispatched_gpu_duty, Some(0));
    }

    #[test]
    fn test_tailord_integration_sync_curve() {
        // If Tailord D-Bus is reachable, execute an end-to-end curve sync and verify
        let tailord_online = Command::new("busctl")
            .args(["introspect", "com.tux.Tailor", "/com/tux/Tailor"])
            .output()
            .map(|o| o.status.success())
            .unwrap_or(false);

        if !tailord_online {
            println!("Skipping live Tailord integration test: Tailord D-Bus service not reachable.");
            return;
        }

        let controller = HardwareController::new();
        let cpu_nodes = vec![
            FanCurveNode::new(35.0, 25.0),
            FanCurveNode::new(55.0, 45.0),
            FanCurveNode::new(75.0, 75.0),
            FanCurveNode::new(90.0, 100.0),
        ];
        let gpu_nodes = vec![
            FanCurveNode::new(40.0, 20.0),
            FanCurveNode::new(60.0, 50.0),
            FanCurveNode::new(80.0, 90.0),
        ];

        let test_profile_name = "test_acc_integration";

        // Query current active profile to restore later
        let prev_active_raw = Command::new("busctl")
            .args(["call", "com.tux.Tailor", "/com/tux/Tailor", "com.tux.Tailor.Profiles", "GetActiveProfileName"])
            .output()
            .ok()
            .and_then(|o| parse_busctl_string(&String::from_utf8_lossy(&o.stdout)))
            .unwrap_or_else(|| "default".to_string());

        // Sync custom curve to Tailord
        let res = controller.sync_curve_to_tailord(test_profile_name, &cpu_nodes, &gpu_nodes);
        assert!(res.is_ok(), "sync_curve_to_tailord failed: {:?}", res.err());

        // Verify active profile was updated in Tailord
        let active_out = Command::new("busctl")
            .args(["call", "com.tux.Tailor", "/com/tux/Tailor", "com.tux.Tailor.Profiles", "GetActiveProfileName"])
            .output()
            .unwrap();
        let active_str = parse_busctl_string(&String::from_utf8_lossy(&active_out.stdout));
        assert_eq!(active_str, Some(test_profile_name.to_string()));

        // Verify CPU fan profile content in Tailord Fan interface
        let fan_out = Command::new("busctl")
            .args(["call", "com.tux.Tailor", "/com/tux/Tailor", "com.tux.Tailor.Fan", "GetProfile", "s", test_profile_name])
            .output()
            .unwrap();
        let fan_str = parse_busctl_string(&String::from_utf8_lossy(&fan_out.stdout));
        assert_eq!(fan_str, Some(fan_curve_to_json(&cpu_nodes)));

        // Clean up: restore original active profile and remove test profiles
        let _ = Command::new("busctl")
            .args(["call", "com.tux.Tailor", "/com/tux/Tailor", "com.tux.Tailor.Profiles", "SetActiveProfileName", "s", &prev_active_raw])
            .output();

        remove_tailord_profile(test_profile_name);
    }
}
