pub mod backend;
pub mod ui;

use backend::{FanCurveNode, FanMode, GpuSwitchMode, HardwareController, PowerMode};
use eframe::egui;
use std::fs;
use std::process::Command;
use std::time::{Duration, Instant};

pub struct AcerApp {
    // Nav tabs
    pub active_tab: usize, // 0: System, 1: Keyboard, 2: Battery, 3: Tools

    // Power & Fan modes
    pub power_mode: PowerMode,
    pub fan_mode: FanMode,
    pub gpu_switch: GpuSwitchMode,

    // Hardware specifications
    pub cpu_name: String,
    pub gpu_name: String,

    // Telemetry
    pub cpu_clock_mhz: u32,
    pub cpu_temp: u32,
    pub cpu_usage_pct: u32,

    pub gpu_clock_mhz: u32,
    pub gpu_temp: u32,
    pub gpu_usage_pct: u32,

    pub mem_used_gb: f32,
    pub mem_total_gb: f32,
    pub mem_pct: u32,

    pub disk_used_gb: u32,
    pub disk_total_gb: u32,
    pub disk_pct: u32,

    pub cpu_fan_rpm: u32,
    pub gpu_fan_rpm: u32,
    pub cpu_fan_pct: u32,
    pub gpu_fan_pct: u32,

    // Custom fan curve
    pub active_curve_tab: usize, // 0: CPU, 1: GPU
    pub cpu_curve_nodes: Vec<FanCurveNode>,
    pub gpu_curve_nodes: Vec<FanCurveNode>,
    pub selected_node_idx: usize,

    // Quick Toggles (Preferences)
    pub win_key_lock: bool,
    pub swap_win_fn: bool,
    pub fn_key_lock: bool,
    pub touchpad_enabled: bool,
    pub camera_enabled: bool,
    pub airplane_mode: bool,
    pub caps_lock: bool,
    pub num_lock: bool,
    pub scr_lock: bool,

    // Blue light filter & display presets
    pub blue_light_filter: bool,
    pub blue_light_level: f32,
    pub display_preset: usize,

    // Bottom carousel / scrollbar
    pub scroll_pos: f32,

    pub controller: HardwareController,
    pub last_update: Instant,
}

impl Default for AcerApp {
    fn default() -> Self {
        let mut app = Self {
            active_tab: 0,
            power_mode: PowerMode::Performance,
            fan_mode: FanMode::Automatic,
            gpu_switch: GpuSwitchMode::Discrete,

            cpu_name: "13th Gen Intel(R) Core(TM) i5-13420H".to_string(),
            gpu_name: "NVIDIA GeForce RTX 3050 6GB Laptop GPU".to_string(),

            cpu_clock_mhz: 2100,
            cpu_temp: 50,
            cpu_usage_pct: 2,

            gpu_clock_mhz: 210,
            gpu_temp: 42,
            gpu_usage_pct: 0,

            mem_used_gb: 8.5,
            mem_total_gb: 16.0,
            mem_pct: 53,

            disk_used_gb: 101,
            disk_total_gb: 163,
            disk_pct: 66,

            cpu_fan_rpm: 2100,
            gpu_fan_rpm: 0,
            cpu_fan_pct: 35,
            gpu_fan_pct: 0,

            active_curve_tab: 0,
            cpu_curve_nodes: vec![
                FanCurveNode { temp: 22.0, speed: 20.0 },
                FanCurveNode { temp: 38.0, speed: 35.0 },
                FanCurveNode { temp: 50.0, speed: 45.0 },
                FanCurveNode { temp: 65.0, speed: 55.0 },
                FanCurveNode { temp: 75.0, speed: 70.0 },
                FanCurveNode { temp: 85.0, speed: 85.0 },
                FanCurveNode { temp: 95.0, speed: 100.0 },
            ],
            gpu_curve_nodes: vec![
                FanCurveNode { temp: 30.0, speed: 0.0 },
                FanCurveNode { temp: 45.0, speed: 30.0 },
                FanCurveNode { temp: 60.0, speed: 50.0 },
                FanCurveNode { temp: 72.0, speed: 65.0 },
                FanCurveNode { temp: 80.0, speed: 80.0 },
                FanCurveNode { temp: 90.0, speed: 100.0 },
            ],
            selected_node_idx: 2,

            win_key_lock: false,
            swap_win_fn: false,
            fn_key_lock: false,
            touchpad_enabled: true,
            camera_enabled: true,
            airplane_mode: false,
            caps_lock: false,
            num_lock: true,
            scr_lock: false,

            blue_light_filter: false,
            blue_light_level: 0.0,
            display_preset: 0,

            scroll_pos: 0.0,

            controller: HardwareController::new(),
            last_update: Instant::now() - Duration::from_secs(2),
        };

        app.detect_hardware_names();
        app.update_hardware();
        app
    }
}

impl AcerApp {
    pub fn detect_hardware_names(&mut self) {
        if let Ok(info) = fs::read_to_string("/proc/cpuinfo") {
            for line in info.lines() {
                if line.starts_with("model name") {
                    if let Some(name) = line.split(':').nth(1) {
                        let trimmed = name.trim();
                        if !trimmed.is_empty() {
                            self.cpu_name = trimmed.to_string();
                        }
                        break;
                    }
                }
            }
        }

        if let Ok(out) = Command::new("nvidia-smi")
            .args(["--query-gpu=name", "--format=csv,noheader"])
            .output()
        {
            if let Ok(name) = String::from_utf8(out.stdout) {
                let trimmed = name.trim();
                if !trimmed.is_empty() {
                    self.gpu_name = trimmed.to_string();
                }
            }
        }
    }

    pub fn update_hardware(&mut self) {
        let telem = self.controller.poll_telemetry(
            self.fan_mode,
            &self.cpu_curve_nodes,
            &self.gpu_curve_nodes,
        );

        self.cpu_clock_mhz = telem.cpu_clock_mhz;
        self.cpu_temp = telem.cpu_temp;
        self.cpu_usage_pct = telem.cpu_usage_pct;

        self.gpu_clock_mhz = telem.gpu_clock_mhz;
        self.gpu_temp = telem.gpu_temp;
        self.gpu_usage_pct = telem.gpu_usage_pct;

        self.mem_used_gb = telem.mem_used_gb;
        self.mem_total_gb = telem.mem_total_gb;
        self.mem_pct = telem.mem_pct;

        self.disk_used_gb = telem.disk_used_gb;
        self.disk_total_gb = telem.disk_total_gb;
        self.disk_pct = telem.disk_pct;

        self.cpu_fan_rpm = telem.cpu_fan_rpm;
        self.gpu_fan_rpm = telem.gpu_fan_rpm;
        self.cpu_fan_pct = telem.cpu_fan_pct;
        self.gpu_fan_pct = telem.gpu_fan_pct;
    }

    #[allow(dead_code)]
    pub fn apply_clevo_fan(&self, speed: u8) {
        let _ = self.controller.override_fan(0, speed);
        let _ = self.controller.override_fan(1, speed);
    }

    pub fn set_power_profile(&mut self, mode: PowerMode) {
        self.power_mode = mode;
        let _ = self.controller.set_power_mode(mode);
        if matches!(mode, PowerMode::Quiet | PowerMode::Entertainment) {
            self.fan_mode = FanMode::Automatic;
        }
    }

    pub fn set_fan_mode(&mut self, mode: FanMode) {
        self.fan_mode = mode;
        let _ = self.controller.set_fan_mode(
            mode,
            &self.cpu_curve_nodes,
            &self.gpu_curve_nodes,
        );
    }

    pub fn set_gpu_switch(&mut self, mode: GpuSwitchMode) {
        self.gpu_switch = mode;
    }
}

impl eframe::App for AcerApp {
    fn ui(&mut self, ui: &mut egui::Ui, _frame: &mut eframe::Frame) {
        // Hardware tick
        if self.last_update.elapsed() >= Duration::from_secs(1) {
            self.update_hardware();
            self.last_update = Instant::now();
        }
        ui.ctx().request_repaint_after(Duration::from_millis(500));

        let ctx = ui.ctx().clone();
        ui::render(self, &ctx, ui);
    }
}

fn main() -> eframe::Result<()> {
    let native_options = eframe::NativeOptions {
        viewport: egui::ViewportBuilder::default()
            .with_inner_size([1040.0, 680.0])
            .with_min_inner_size([960.0, 600.0])
            .with_title("Control Center"),
        ..Default::default()
    };

    eframe::run_native(
        "com.clevo.ControlCenter",
        native_options,
        Box::new(|_cc| Ok(Box::new(AcerApp::default()))),
    )
}
