use eframe::egui::{self, Color32, FontId, Pos2, Rect, Stroke, Vec2};
use std::fs;
use std::process::Command;
use std::time::{Duration, Instant};

#[derive(Clone, Copy, PartialEq, Eq)]
enum PowerMode {
    Quiet,
    Entertainment,
    Performance,
}

#[derive(Clone, Copy, PartialEq, Eq)]
enum FanMode {
    Automatic,
    Maximum,
    Custom,
    Turbo,
}

#[derive(Clone, Copy, PartialEq, Eq)]
enum GpuSwitchMode {
    Integrated,
    Dynamic,
    Discrete,
}

struct FanCurveNode {
    temp: f32,  // 20.0 to 100.0 C
    speed: f32, // 0.0 to 100.0 %
}

#[allow(dead_code)]
struct AcerApp {
    // Nav tabs
    active_tab: usize, // 0: System, 1: Keyboard, 2: Battery, 3: Tools

    // Power & Fan modes
    power_mode: PowerMode,
    fan_mode: FanMode,
    gpu_switch: GpuSwitchMode,

    // Telemetry
    cpu_clock_mhz: u32,
    cpu_temp: u32,
    cpu_usage_pct: u32,

    gpu_clock_mhz: u32,
    gpu_temp: u32,
    gpu_usage_pct: u32,

    mem_used_gb: f32,
    mem_total_gb: f32,
    mem_pct: u32,

    disk_used_gb: u32,
    disk_total_gb: u32,
    disk_pct: u32,

    cpu_fan_rpm: u32,
    gpu_fan_rpm: u32,
    cpu_fan_pct: u32,
    gpu_fan_pct: u32,

    // Custom fan curve
    active_curve_tab: usize, // 0: CPU, 1: GPU
    cpu_curve_nodes: Vec<FanCurveNode>,
    gpu_curve_nodes: Vec<FanCurveNode>,
    selected_node_idx: usize,

    // Quick Toggles (Preferences)
    win_key_lock: bool,
    swap_win_fn: bool,
    fn_key_lock: bool,
    touchpad_enabled: bool,
    camera_enabled: bool,
    airplane_mode: bool,
    caps_lock: bool,
    num_lock: bool,
    scr_lock: bool,

    // Blue light filter
    blue_light_filter: bool,
    blue_light_level: f32,

    last_update: Instant,
    last_cpu_ticks: (u64, u64),
}

impl Default for AcerApp {
    fn default() -> Self {
        let mut app = Self {
            active_tab: 0,
            power_mode: PowerMode::Performance,
            fan_mode: FanMode::Automatic,
            gpu_switch: GpuSwitchMode::Dynamic,

            cpu_clock_mhz: 2100,
            cpu_temp: 50,
            cpu_usage_pct: 2,

            gpu_clock_mhz: 210,
            gpu_temp: 42,
            gpu_usage_pct: 0,

            mem_used_gb: 8.5,
            mem_total_gb: 16.0,
            mem_pct: 53,

            disk_used_gb: 98,
            disk_total_gb: 163,
            disk_pct: 60,

            cpu_fan_rpm: 2100,
            gpu_fan_rpm: 0,
            cpu_fan_pct: 35,
            gpu_fan_pct: 0,

            active_curve_tab: 0,
            cpu_curve_nodes: vec![
                FanCurveNode { temp: 25.0, speed: 20.0 },
                FanCurveNode { temp: 45.0, speed: 35.0 },
                FanCurveNode { temp: 65.0, speed: 55.0 },
                FanCurveNode { temp: 80.0, speed: 75.0 },
                FanCurveNode { temp: 95.0, speed: 100.0 },
            ],
            gpu_curve_nodes: vec![
                FanCurveNode { temp: 30.0, speed: 0.0 },
                FanCurveNode { temp: 50.0, speed: 30.0 },
                FanCurveNode { temp: 65.0, speed: 50.0 },
                FanCurveNode { temp: 75.0, speed: 70.0 },
                FanCurveNode { temp: 85.0, speed: 100.0 },
            ],
            selected_node_idx: 1,

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

            last_update: Instant::now() - Duration::from_secs(2),
            last_cpu_ticks: (0, 0),
        };
        app.update_hardware();
        app
    }
}

impl AcerApp {
    fn update_hardware(&mut self) {
        // 1. CPU Temp
        if let Ok(entries) = fs::read_dir("/sys/class/hwmon") {
            for entry in entries.flatten() {
                let name_path = entry.path().join("name");
                if let Ok(name) = fs::read_to_string(&name_path) {
                    if name.contains("coretemp") {
                        let temp_path = entry.path().join("temp1_input");
                        if let Ok(temp_str) = fs::read_to_string(temp_path) {
                            if let Ok(milli) = temp_str.trim().parse::<u32>() {
                                self.cpu_temp = milli / 1000;
                            }
                        }
                        break;
                    }
                }
            }
        }

        // 2. CPU Clock Frequency
        if let Ok(freq_str) = fs::read_to_string("/sys/devices/system/cpu/cpu0/cpufreq/scaling_cur_freq") {
            if let Ok(khz) = freq_str.trim().parse::<u32>() {
                self.cpu_clock_mhz = khz / 1000;
            }
        }

        // 3. CPU Usage (%)
        if let Ok(stat) = fs::read_to_string("/proc/stat") {
            if let Some(line) = stat.lines().next() {
                let parts: Vec<u64> = line
                    .split_whitespace()
                    .skip(1)
                    .filter_map(|s| s.parse().ok())
                    .collect();
                if parts.len() >= 4 {
                    let idle = parts[3];
                    let total: u64 = parts.iter().sum();
                    let (prev_idle, prev_total) = self.last_cpu_ticks;
                    if prev_total > 0 && total > prev_total {
                        let diff_idle = idle.saturating_sub(prev_idle);
                        let diff_total = total.saturating_sub(prev_total);
                        let usage = 100 - (diff_idle * 100 / diff_total);
                        self.cpu_usage_pct = usage as u32;
                    }
                    self.last_cpu_ticks = (idle, total);
                }
            }
        }

        // 4. Memory Usage
        if let Ok(mem) = fs::read_to_string("/proc/meminfo") {
            let mut total_kb = 0u64;
            let mut avail_kb = 0u64;
            for line in mem.lines() {
                if line.starts_with("MemTotal:") {
                    total_kb = line.split_whitespace().nth(1).and_then(|s| s.parse().ok()).unwrap_or(0);
                } else if line.starts_with("MemAvailable:") {
                    avail_kb = line.split_whitespace().nth(1).and_then(|s| s.parse().ok()).unwrap_or(0);
                }
            }
            if total_kb > 0 {
                let used_kb = total_kb.saturating_sub(avail_kb);
                self.mem_total_gb = (total_kb as f32 / 1_048_576.0 * 10.0).round() / 10.0;
                self.mem_used_gb = (used_kb as f32 / 1_048_576.0 * 10.0).round() / 10.0;
                self.mem_pct = ((used_kb as f64 / total_kb as f64) * 100.0) as u32;
            }
        }

        // 5. NVIDIA GPU Metrics
        if let Ok(output) = Command::new("nvidia-smi")
            .args(["--query-gpu=clocks.gr,utilization.gpu,temperature.gpu", "--format=csv,noheader,nounits"])
            .output()
        {
            if let Ok(s) = String::from_utf8(output.stdout) {
                let parts: Vec<&str> = s.trim().split(',').map(|p| p.trim()).collect();
                if parts.len() >= 3 {
                    self.gpu_clock_mhz = parts[0].parse().unwrap_or(0);
                    self.gpu_usage_pct = parts[1].parse().unwrap_or(0);
                    self.gpu_temp = parts[2].parse().unwrap_or(0);
                }
            }
        }

        // Calculate dynamic fan RPM / Duty display
        match self.fan_mode {
            FanMode::Automatic => {
                self.cpu_fan_pct = (self.cpu_temp.saturating_sub(30) * 100 / 60).clamp(20, 100);
                self.cpu_fan_rpm = (self.cpu_fan_pct as f32 / 100.0 * 4800.0) as u32;

                if self.gpu_temp > 45 {
                    self.gpu_fan_pct = (self.gpu_temp.saturating_sub(40) * 100 / 50).clamp(20, 100);
                    self.gpu_fan_rpm = (self.gpu_fan_pct as f32 / 100.0 * 4500.0) as u32;
                } else {
                    self.gpu_fan_pct = 0;
                    self.gpu_fan_rpm = 0;
                }
            }
            FanMode::Maximum => {
                self.cpu_fan_pct = 100;
                self.gpu_fan_pct = 100;
                self.cpu_fan_rpm = 5100;
                self.gpu_fan_rpm = 4900;
            }
            FanMode::Turbo => {
                self.cpu_fan_pct = 100;
                self.gpu_fan_pct = 100;
                self.cpu_fan_rpm = 5400;
                self.gpu_fan_rpm = 5200;
            }
            FanMode::Custom => {
                // Interpolate from curve
                let nodes = if self.active_curve_tab == 0 { &self.cpu_curve_nodes } else { &self.gpu_curve_nodes };
                let current_temp = if self.active_curve_tab == 0 { self.cpu_temp as f32 } else { self.gpu_temp as f32 };
                let mut speed = 30.0;
                for i in 0..nodes.len().saturating_sub(1) {
                    if current_temp >= nodes[i].temp && current_temp <= nodes[i+1].temp {
                        let factor = (current_temp - nodes[i].temp) / (nodes[i+1].temp - nodes[i].temp);
                        speed = nodes[i].speed + factor * (nodes[i+1].speed - nodes[i].speed);
                        break;
                    }
                }
                if self.active_curve_tab == 0 {
                    self.cpu_fan_pct = speed as u32;
                    self.cpu_fan_rpm = (speed / 100.0 * 5000.0) as u32;
                } else {
                    self.gpu_fan_pct = speed as u32;
                    self.gpu_fan_rpm = (speed / 100.0 * 4800.0) as u32;
                }
            }
        }
    }

    fn apply_clevo_fan(&self, speed: u8) {
        let _ = Command::new("busctl")
            .args([
                "call",
                "com.tux.Tailor",
                "/com/tux/Tailor",
                "com.tux.Tailor.Fan",
                "OverrideSpeed",
                "yy",
                "0",
                &speed.to_string(),
            ])
            .output();
        let _ = Command::new("busctl")
            .args([
                "call",
                "com.tux.Tailor",
                "/com/tux/Tailor",
                "com.tux.Tailor.Fan",
                "OverrideSpeed",
                "yy",
                "1",
                &speed.to_string(),
            ])
            .output();
    }

    fn set_power_profile(&mut self, mode: PowerMode) {
        self.power_mode = mode;
        match mode {
            PowerMode::Quiet => {
                let _ = Command::new("/usr/bin/python3")
                    .args(["/usr/bin/powerprofilesctl", "set", "power-saver"])
                    .output();
                self.fan_mode = FanMode::Automatic;
                self.apply_clevo_fan(0);
            }
            PowerMode::Entertainment => {
                let _ = Command::new("/usr/bin/python3")
                    .args(["/usr/bin/powerprofilesctl", "set", "balanced"])
                    .output();
                self.fan_mode = FanMode::Automatic;
                self.apply_clevo_fan(0);
            }
            PowerMode::Performance => {
                let _ = Command::new("/usr/bin/python3")
                    .args(["/usr/bin/powerprofilesctl", "set", "performance"])
                    .output();
            }
        }
    }

    fn set_fan_mode(&mut self, mode: FanMode) {
        self.fan_mode = mode;
        match mode {
            FanMode::Automatic => self.apply_clevo_fan(0),
            FanMode::Maximum | FanMode::Turbo => self.apply_clevo_fan(100),
            FanMode::Custom => self.apply_clevo_fan(self.cpu_fan_pct as u8),
        }
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

        let bg_color = Color32::from_rgb(10, 12, 16);
        let card_bg = Color32::from_rgb(17, 20, 27);
        let card_border = Color32::from_rgb(33, 38, 48);
        let cyan = Color32::from_rgb(0, 225, 255);
        let gold = Color32::from_rgb(240, 180, 40);
        let text_muted = Color32::from_rgb(140, 148, 160);

        // Entire window canvas
        let rect = ui.max_rect();
        ui.painter().rect_filled(rect, 0.0, bg_color);

        ui.spacing_mut().item_spacing = Vec2::new(8.0, 8.0);

        // ==========================================
        // 1. TOP WINDOW BAR & MAIN NAVIGATION
        // ==========================================
        ui.horizontal(|ui| {
            ui.add_space(8.0);

            // 4 Left Category Icons
            let tabs = ["⏱", "💡", "🔋", "⚙"];
            for (idx, icon) in tabs.iter().enumerate() {
                let is_active = self.active_tab == idx;
                let btn = ui.add(
                    egui::Button::new(
                        egui::RichText::new(*icon)
                            .size(16.0)
                            .color(if is_active { cyan } else { text_muted }),
                    )
                    .fill(if is_active { Color32::from_rgb(24, 32, 44) } else { Color32::TRANSPARENT })
                    .stroke(Stroke::new(if is_active { 1.5 } else { 0.0 }, if is_active { cyan } else { Color32::TRANSPARENT }))
                    .corner_radius(4),
                );
                if btn.clicked() {
                    self.active_tab = idx;
                }
                ui.add_space(4.0);
            }

            ui.add_space(20.0);

            // Centered Title
            ui.label(
                egui::RichText::new("Control Center")
                    .size(17.0)
                    .strong()
                    .color(Color32::WHITE),
            );

            // Right utility info
            ui.with_layout(egui::Layout::right_to_left(egui::Align::Center), |ui| {
                ui.add_space(8.0);
                ui.label(
                    egui::RichText::new("Clevo Control Center 3.0 • Arch Linux")
                        .size(11.0)
                        .color(text_muted),
                );
            });
        });

        // ==========================================
        // 2. POWER PROFILES (TOP-CENTER TABS)
        // ==========================================
        ui.vertical_centered(|ui| {
            ui.horizontal(|ui| {
                ui.add_space((ui.available_width() - 320.0) / 2.0);
                let modes = [
                    (PowerMode::Performance, "Performance"),
                    (PowerMode::Entertainment, "Entertainment"),
                    (PowerMode::Quiet, "Quiet"),
                ];

                for (mode, label) in modes {
                    let is_active = self.power_mode == mode;
                    let text = egui::RichText::new(label)
                        .size(14.0)
                        .strong()
                        .color(if is_active { gold } else { text_muted });

                    if ui.selectable_label(is_active, text).clicked() {
                        self.set_power_profile(mode);
                    }
                    ui.add_space(14.0);
                }
            });
        });

        ui.add_space(2.0);

        // ==========================================
        // 3. MAIN MID-DECK (3 COLUMNS)
        // ==========================================
        ui.columns(3, |cols| {
            // ------------------------------------------
            // COLUMN A: SYSTEM TELEMETRY
            // ------------------------------------------
            let col_a = &mut cols[0];
            egui::Frame::new()
                .fill(card_bg)
                .stroke(Stroke::new(1.0, card_border))
                .corner_radius(8)
                .inner_margin(12)
                .show(col_a, |ui| {
                    ui.label(egui::RichText::new("System").size(15.0).strong().color(Color32::WHITE));
                    ui.label(egui::RichText::new("13th Gen Intel(R) Core(TM) i5-13420H").size(11.0).color(text_muted));
                    ui.label(egui::RichText::new("NVIDIA GeForce RTX 3050 6GB Laptop GPU").size(11.0).color(text_muted));

                    ui.add_space(10.0);

                    // CPU Clock & Temp
                    ui.horizontal(|ui| {
                        ui.vertical(|ui| {
                            ui.label(egui::RichText::new(format!("{} MHz", self.cpu_clock_mhz)).size(19.0).strong().color(Color32::WHITE));
                            ui.label(egui::RichText::new("CPU clock speed").size(10.0).color(cyan));
                        });
                        ui.add_space(18.0);
                        ui.vertical(|ui| {
                            ui.label(egui::RichText::new(format!("{} ℃", self.cpu_temp)).size(19.0).strong().color(Color32::WHITE));
                            ui.label(egui::RichText::new("Temperature").size(10.0).color(cyan));
                        });
                    });

                    ui.add_space(10.0);

                    // GPU Clock & Temp
                    ui.horizontal(|ui| {
                        ui.vertical(|ui| {
                            let clk_str = if self.gpu_clock_mhz > 0 { format!("{} MHz", self.gpu_clock_mhz) } else { "OFF MHz".to_string() };
                            ui.label(egui::RichText::new(clk_str).size(19.0).strong().color(Color32::WHITE));
                            ui.label(egui::RichText::new("GPU clock speed").size(10.0).color(cyan));
                        });
                        ui.add_space(18.0);
                        ui.vertical(|ui| {
                            let temp_str = if self.gpu_temp > 0 { format!("{} ℃", self.gpu_temp) } else { "OFF ℃".to_string() };
                            ui.label(egui::RichText::new(temp_str).size(19.0).strong().color(Color32::WHITE));
                            ui.label(egui::RichText::new("Temperature").size(10.0).color(cyan));
                        });
                    });

                    ui.add_space(12.0);

                    // Usage Progress Bars
                    ui.horizontal(|ui| {
                        ui.label(egui::RichText::new("CPU usage").size(11.0).color(text_muted));
                        ui.with_layout(egui::Layout::right_to_left(egui::Align::Center), |ui| {
                            ui.label(egui::RichText::new(format!("{} %", self.cpu_usage_pct)).size(11.0).color(Color32::WHITE));
                        });
                    });
                    render_striped_bar(ui, self.cpu_usage_pct, cyan);

                    ui.add_space(6.0);

                    ui.horizontal(|ui| {
                        ui.label(egui::RichText::new("GPU usage").size(11.0).color(text_muted));
                        ui.with_layout(egui::Layout::right_to_left(egui::Align::Center), |ui| {
                            ui.label(egui::RichText::new(format!("{} %", self.gpu_usage_pct)).size(11.0).color(Color32::WHITE));
                        });
                    });
                    render_striped_bar(ui, self.gpu_usage_pct, cyan);
                });

            // ------------------------------------------
            // COLUMN B: CENTER (TACHOMETER OR CUSTOM FAN CURVE)
            // ------------------------------------------
            let col_b = &mut cols[1];
            egui::Frame::new()
                .fill(card_bg)
                .stroke(Stroke::new(1.0, card_border))
                .corner_radius(8)
                .inner_margin(12)
                .show(col_b, |ui| {
                    if self.fan_mode == FanMode::Custom {
                        // RENDER CUSTOM FAN SETTING GRAPH
                        ui.horizontal(|ui| {
                            ui.label(egui::RichText::new("Custom fan setting").size(14.0).strong().color(Color32::WHITE));
                            ui.with_layout(egui::Layout::right_to_left(egui::Align::Center), |ui| {
                                if ui.button(egui::RichText::new(">>").color(gold)).clicked() {
                                    self.fan_mode = FanMode::Automatic;
                                }
                            });
                        });

                        ui.horizontal(|ui| {
                            if ui.selectable_label(self.active_curve_tab == 0, egui::RichText::new("CPU Fan").color(if self.active_curve_tab == 0 { gold } else { text_muted })).clicked() {
                                self.active_curve_tab = 0;
                            }
                            if ui.selectable_label(self.active_curve_tab == 1, egui::RichText::new("GPU Fan").color(if self.active_curve_tab == 1 { cyan } else { text_muted })).clicked() {
                                self.active_curve_tab = 1;
                            }
                        });

                        // Draw Curve Graph Canvas
                        render_fan_curve_graph(
                            ui,
                            if self.active_curve_tab == 0 { &mut self.cpu_curve_nodes } else { &mut self.gpu_curve_nodes },
                            if self.active_curve_tab == 0 { gold } else { cyan },
                            &mut self.selected_node_idx,
                        );

                        // Readout boxes at bottom of graph
                        let nodes = if self.active_curve_tab == 0 { &self.cpu_curve_nodes } else { &self.gpu_curve_nodes };
                        let idx = self.selected_node_idx.min(nodes.len().saturating_sub(1));
                        let cur_temp = nodes[idx].temp as u32;
                        let cur_speed = nodes[idx].speed as u32;

                        ui.add_space(6.0);
                        ui.horizontal(|ui| {
                            ui.label(egui::RichText::new(format!("Fan Speed: {}%", cur_speed)).size(11.0).color(Color32::WHITE));
                            ui.add_space(16.0);
                            ui.label(egui::RichText::new(format!("Temperature: {}℃", cur_temp)).size(11.0).color(Color32::WHITE));
                        });
                    } else {
                        // RENDER AUTHENTIC CLEVO SPEEDOMETER & TEMPERATURE TACHOMETER
                        ui.vertical_centered(|ui| {
                            ui.label(egui::RichText::new("Processor Telemetry").size(13.0).color(text_muted));
                        });
                        render_clevo_tachometer(ui, self.cpu_clock_mhz, self.cpu_temp, gold, cyan);
                    }
                });

            // ------------------------------------------
            // COLUMN C: FAN SPEED CONTROL
            // ------------------------------------------
            let col_c = &mut cols[2];
            egui::Frame::new()
                .fill(card_bg)
                .stroke(Stroke::new(1.0, card_border))
                .corner_radius(8)
                .inner_margin(12)
                .show(col_c, |ui| {
                    ui.horizontal(|ui| {
                        ui.label(egui::RichText::new("🌀 FAN speed control").size(15.0).strong().color(Color32::WHITE));
                    });

                    ui.add_space(6.0);

                    // Radio modes
                    let modes = [
                        (FanMode::Automatic, "Automatic"),
                        (FanMode::Maximum, "Maximum"),
                        (FanMode::Custom, "Custom"),
                        (FanMode::Turbo, "Turbo"),
                    ];

                    for (m, lbl) in modes {
                        let is_active = self.fan_mode == m;
                        ui.horizontal(|ui| {
                            let radio = ui.radio(is_active, egui::RichText::new(lbl).size(13.0).color(if is_active { gold } else { text_muted }));
                            if radio.clicked() {
                                self.set_fan_mode(m);
                            }
                            if m == FanMode::Custom {
                                if ui.small_button(if is_active { "<<" } else { ">>" }).clicked() {
                                    self.set_fan_mode(FanMode::Custom);
                                }
                            }
                        });
                    }

                    ui.add_space(8.0);
                    ui.separator();
                    ui.add_space(6.0);

                    // RPM Readouts
                    ui.horizontal(|ui| {
                        ui.vertical(|ui| {
                            ui.label(egui::RichText::new(format!("{} RPM", self.cpu_fan_rpm)).size(18.0).strong().color(Color32::WHITE));
                            ui.label(egui::RichText::new("CPU fan speed").size(10.0).color(cyan));
                        });
                        ui.add_space(14.0);
                        ui.separator();
                        ui.add_space(14.0);
                        ui.vertical(|ui| {
                            ui.label(egui::RichText::new(format!("{} RPM", self.gpu_fan_rpm)).size(18.0).strong().color(Color32::WHITE));
                            ui.label(egui::RichText::new("GPU fan speed").size(10.0).color(cyan));
                        });
                    });

                    ui.add_space(12.0);

                    // Percentage Bars
                    ui.horizontal(|ui| {
                        ui.label(egui::RichText::new("CPU FAN").size(11.0).color(text_muted));
                        ui.with_layout(egui::Layout::right_to_left(egui::Align::Center), |ui| {
                            ui.label(egui::RichText::new(format!("{} %", self.cpu_fan_pct)).size(11.0).color(Color32::WHITE));
                        });
                    });
                    render_striped_bar(ui, self.cpu_fan_pct, cyan);

                    ui.add_space(6.0);

                    ui.horizontal(|ui| {
                        ui.label(egui::RichText::new("GPU FAN").size(11.0).color(text_muted));
                        ui.with_layout(egui::Layout::right_to_left(egui::Align::Center), |ui| {
                            ui.label(egui::RichText::new(format!("{} %", self.gpu_fan_pct)).size(11.0).color(Color32::WHITE));
                        });
                    });
                    render_striped_bar(ui, self.gpu_fan_pct, cyan);
                });
        });

        ui.add_space(2.0);

        // ==========================================
        // 4. BOTTOM DECK (3 MODULAR PANELS)
        // ==========================================
        ui.columns(3, |cols| {
            // PANEL 1: MEMORY AND STORAGE
            let p1 = &mut cols[0];
            egui::Frame::new()
                .fill(card_bg)
                .stroke(Stroke::new(1.0, card_border))
                .corner_radius(8)
                .inner_margin(12)
                .show(p1, |ui| {
                    ui.label(egui::RichText::new("Memory and storage").size(13.0).strong().color(Color32::WHITE));
                    ui.add_space(6.0);
                    ui.horizontal(|ui| {
                        render_circular_dial(ui, "Memory", self.mem_pct, &format!("{:.1}GB", self.mem_used_gb), cyan);
                        ui.add_space(12.0);
                        render_circular_dial(ui, "Storage", self.disk_pct, &format!("{}GB", self.disk_used_gb), cyan);
                    });
                });

            // PANEL 2: PREFERENCE (QUICK HARDWARE TOGGLES)
            let p2 = &mut cols[1];
            egui::Frame::new()
                .fill(card_bg)
                .stroke(Stroke::new(1.0, card_border))
                .corner_radius(8)
                .inner_margin(10)
                .show(p2, |ui| {
                    ui.label(egui::RichText::new("Preference").size(13.0).strong().color(Color32::WHITE));
                    ui.add_space(6.0);

                    ui.horizontal(|ui| {
                        render_toggle_btn(ui, "Win key\nlock", &mut self.win_key_lock, cyan);
                        render_toggle_btn(ui, "Swap\nWin/Fn", &mut self.swap_win_fn, cyan);
                        render_toggle_btn(ui, "Fn key\nlock", &mut self.fn_key_lock, cyan);
                        render_toggle_btn(ui, "Touch\nPad", &mut self.touchpad_enabled, cyan);
                        render_toggle_btn(ui, "Camera", &mut self.camera_enabled, cyan);
                        render_toggle_btn(ui, "Airplane", &mut self.airplane_mode, cyan);
                        render_toggle_btn(ui, "Caps\nLock", &mut self.caps_lock, cyan);
                        render_toggle_btn(ui, "Num\nLock", &mut self.num_lock, cyan);
                    });
                });

            // PANEL 3: CONFIGURATION & GPU SWITCH
            let p3 = &mut cols[2];
            egui::Frame::new()
                .fill(card_bg)
                .stroke(Stroke::new(1.0, card_border))
                .corner_radius(8)
                .inner_margin(12)
                .show(p3, |ui| {
                    ui.horizontal(|ui| {
                        // Left subpanel: Blue Light
                        ui.vertical(|ui| {
                            ui.label(egui::RichText::new("Configuration").size(12.0).strong().color(Color32::WHITE));
                            ui.horizontal(|ui| {
                                ui.checkbox(&mut self.blue_light_filter, "Blue light filter");
                            });
                            ui.add(egui::Slider::new(&mut self.blue_light_level, 0.0..=10.0).text("Level"));
                        });

                        ui.add_space(10.0);
                        ui.separator();
                        ui.add_space(10.0);

                        // Right subpanel: GPU Switch
                        ui.vertical(|ui| {
                            ui.label(egui::RichText::new("GPU switch").size(12.0).strong().color(Color32::WHITE));
                            if ui.radio(self.gpu_switch == GpuSwitchMode::Integrated, "iGPU only").clicked() {
                                self.gpu_switch = GpuSwitchMode::Integrated;
                            }
                            if ui.radio(self.gpu_switch == GpuSwitchMode::Dynamic, "Dynamic").clicked() {
                                self.gpu_switch = GpuSwitchMode::Dynamic;
                            }
                            if ui.radio(self.gpu_switch == GpuSwitchMode::Discrete, "dGPU only").clicked() {
                                self.gpu_switch = GpuSwitchMode::Discrete;
                            }
                        });
                    });
                });
        });
    }
}

// ----------------------------------------------------------------------------
// CANVAS DRAWING HELPERS
// ----------------------------------------------------------------------------

fn render_striped_bar(ui: &mut egui::Ui, pct: u32, fill_col: Color32) {
    let (rect, _) = ui.allocate_exact_size(Vec2::new(ui.available_width(), 12.0), egui::Sense::hover());
    let painter = ui.painter_at(rect);
    let bg_color = Color32::from_rgb(25, 30, 40);
    painter.rect_filled(rect, 2.0, bg_color);

    let num_segments = 25;
    let filled_segments = (num_segments as f32 * (pct as f32 / 100.0)).round() as usize;
    let seg_w = (rect.width() - (num_segments as f32 * 2.0)) / num_segments as f32;

    for i in 0..num_segments {
        let x = rect.min.x + (i as f32) * (seg_w + 2.0);
        let seg_rect = Rect::from_min_size(Pos2::new(x, rect.min.y + 1.0), Vec2::new(seg_w, rect.height() - 2.0));
        let col = if i < filled_segments { fill_col } else { Color32::from_rgb(35, 42, 54) };
        painter.rect_filled(seg_rect, 1.0, col);
    }
}

fn render_circular_dial(ui: &mut egui::Ui, label: &str, pct: u32, subtext: &str, accent: Color32) {
    let (rect, _) = ui.allocate_exact_size(Vec2::new(65.0, 65.0), egui::Sense::hover());
    let painter = ui.painter_at(rect);
    let center = rect.center();
    let radius = 26.0;

    // Track
    let start_angle = std::f32::consts::PI * 0.75;
    let end_angle = std::f32::consts::PI * 2.25;

    for i in 0..20 {
        let a1 = start_angle + (end_angle - start_angle) * (i as f32 / 20.0);
        let a2 = start_angle + (end_angle - start_angle) * ((i + 1) as f32 / 20.0);
        let p1 = Pos2::new(center.x + radius * a1.cos(), center.y + radius * a1.sin());
        let p2 = Pos2::new(center.x + radius * a2.cos(), center.y + radius * a2.sin());
        painter.line_segment([p1, p2], Stroke::new(3.0, Color32::from_rgb(33, 38, 48)));
    }

    let active_segments = (20.0 * (pct as f32 / 100.0)) as usize;
    for i in 0..active_segments {
        let a1 = start_angle + (end_angle - start_angle) * (i as f32 / 20.0);
        let a2 = start_angle + (end_angle - start_angle) * ((i + 1) as f32 / 20.0);
        let p1 = Pos2::new(center.x + radius * a1.cos(), center.y + radius * a1.sin());
        let p2 = Pos2::new(center.x + radius * a2.cos(), center.y + radius * a2.sin());
        painter.line_segment([p1, p2], Stroke::new(3.5, accent));
    }

    // Text in center
    painter.text(Pos2::new(center.x, center.y - 4.0), egui::Align2::CENTER_CENTER, format!("{}%", pct), FontId::proportional(11.0), Color32::WHITE);
    painter.text(Pos2::new(center.x, center.y + 7.0), egui::Align2::CENTER_CENTER, label, FontId::proportional(9.0), Color32::from_rgb(140, 148, 160));
    painter.text(Pos2::new(center.x, center.y + 18.0), egui::Align2::CENTER_CENTER, subtext, FontId::proportional(8.0), accent);
}

fn render_toggle_btn(ui: &mut egui::Ui, label: &str, state: &mut bool, accent: Color32) {
    let is_on = *state;
    let border = if is_on { accent } else { Color32::from_rgb(45, 52, 65) };
    let bg = if is_on { Color32::from_rgb(16, 36, 48) } else { Color32::from_rgb(22, 26, 34) };
    let text_col = if is_on { Color32::WHITE } else { Color32::from_rgb(140, 148, 160) };

    let btn = ui.add(
        egui::Button::new(egui::RichText::new(label).size(9.0).color(text_col))
            .fill(bg)
            .stroke(Stroke::new(if is_on { 1.5 } else { 1.0 }, border))
            .corner_radius(4)
            .min_size(Vec2::new(34.0, 36.0)),
    );
    if btn.clicked() {
        *state = !*state;
    }
}

fn render_clevo_tachometer(ui: &mut egui::Ui, clock_mhz: u32, temp: u32, gold: Color32, cyan: Color32) {
    let (rect, _) = ui.allocate_exact_size(Vec2::new(ui.available_width(), 170.0), egui::Sense::hover());
    let painter = ui.painter_at(rect);
    let center = rect.center();
    let radius = 68.0;

    // Center circular badge "CPU"
    painter.circle_filled(center, 22.0, Color32::from_rgb(28, 34, 46));
    painter.circle_stroke(center, 22.0, Stroke::new(2.0, cyan));
    painter.circle_stroke(center, 26.0, Stroke::new(1.0, Color32::from_rgb(80, 50, 120)));
    painter.text(center, egui::Align2::CENTER_CENTER, "CPU", FontId::proportional(12.0), Color32::WHITE);

    // Left semi-circle: Speedometer 0 to 6 x1000 MHz
    let start_left = std::f32::consts::PI * 1.5;
    let clk_pct = (clock_mhz as f32 / 6000.0).clamp(0.0, 1.0);
    let needle_left_angle = start_left - (std::f32::consts::PI * clk_pct);

    for i in 0..15 {
        let a1 = start_left - (std::f32::consts::PI * (i as f32 / 15.0));
        let a2 = start_left - (std::f32::consts::PI * ((i + 1) as f32 / 15.0));
        let p1 = Pos2::new(center.x + radius * a1.cos(), center.y + radius * a1.sin());
        let p2 = Pos2::new(center.x + radius * a2.cos(), center.y + radius * a2.sin());
        painter.line_segment([p1, p2], Stroke::new(4.0, Color32::from_rgb(33, 38, 48)));
    }

    // Left Needle
    let n1_end = Pos2::new(center.x + (radius - 5.0) * needle_left_angle.cos(), center.y + (radius - 5.0) * needle_left_angle.sin());
    painter.line_segment([center, n1_end], Stroke::new(2.5, gold));

    // Right semi-circle: Temperature Arc (Green to Red)
    let start_right = std::f32::consts::PI * 1.5;
    let temp_pct = (temp as f32 / 100.0).clamp(0.0, 1.0);
    let needle_right_angle = start_right + (std::f32::consts::PI * temp_pct);

    for i in 0..15 {
        let a1 = start_right + (std::f32::consts::PI * (i as f32 / 15.0));
        let a2 = start_right + (std::f32::consts::PI * ((i + 1) as f32 / 15.0));
        let p1 = Pos2::new(center.x + radius * a1.cos(), center.y + radius * a1.sin());
        let p2 = Pos2::new(center.x + radius * a2.cos(), center.y + radius * a2.sin());
        let col = if i < 5 { Color32::from_rgb(46, 160, 67) } else if i < 10 { Color32::from_rgb(210, 153, 34) } else { Color32::from_rgb(248, 81, 73) };
        painter.line_segment([p1, p2], Stroke::new(4.0, col));
    }

    // Right Needle
    let n2_end = Pos2::new(center.x + (radius - 5.0) * needle_right_angle.cos(), center.y + (radius - 5.0) * needle_right_angle.sin());
    painter.line_segment([center, n2_end], Stroke::new(2.5, cyan));

    // Dial Labels
    painter.text(Pos2::new(center.x - radius - 15.0, center.y), egui::Align2::RIGHT_CENTER, "Clock speed\nx1000 MHz", FontId::proportional(9.0), Color32::from_rgb(140, 148, 160));
    painter.text(Pos2::new(center.x + radius + 15.0, center.y), egui::Align2::LEFT_CENTER, format!("{} ℃\nTemperature", temp), FontId::proportional(9.0), Color32::from_rgb(140, 148, 160));
}

fn render_fan_curve_graph(
    ui: &mut egui::Ui,
    nodes: &mut Vec<FanCurveNode>,
    line_col: Color32,
    selected_idx: &mut usize,
) {
    let (rect, response) = ui.allocate_exact_size(Vec2::new(ui.available_width(), 135.0), egui::Sense::click_and_drag());
    let painter = ui.painter_at(rect);

    // Background Grid
    painter.rect_filled(rect, 4.0, Color32::from_rgb(14, 17, 24));
    let grid_stroke = Stroke::new(0.8, Color32::from_rgb(28, 34, 46));

    // Horizontal lines (0% to 100%)
    for y_pct in [0.25, 0.5, 0.75, 1.0] {
        let y = rect.max.y - (rect.height() * y_pct);
        painter.line_segment([Pos2::new(rect.min.x, y), Pos2::new(rect.max.x, y)], grid_stroke);
    }
    // Vertical lines (Temp: 20C to 100C)
    for x_pct in [0.25, 0.5, 0.75] {
        let x = rect.min.x + (rect.width() * x_pct);
        painter.line_segment([Pos2::new(x, rect.min.y), Pos2::new(x, rect.max.y)], grid_stroke);
    }

    // Convert node to screen pos
    let node_to_screen = |node: &FanCurveNode| -> Pos2 {
        let x = rect.min.x + ((node.temp - 20.0) / 80.0) * rect.width();
        let y = rect.max.y - (node.speed / 100.0) * rect.height();
        Pos2::new(x, y)
    };

    // Draw curve line
    for i in 0..nodes.len().saturating_sub(1) {
        let p1 = node_to_screen(&nodes[i]);
        let p2 = node_to_screen(&nodes[i + 1]);
        painter.line_segment([p1, p2], Stroke::new(2.5, line_col));
    }

    // Draw circular nodes
    for (i, node) in nodes.iter().enumerate() {
        let p = node_to_screen(node);
        let is_sel = *selected_idx == i;
        painter.circle_filled(p, if is_sel { 6.0 } else { 4.0 }, Color32::WHITE);
        painter.circle_stroke(p, if is_sel { 7.0 } else { 5.0 }, Stroke::new(2.0, line_col));
    }

    // Handle drag on nodes
    if let Some(mouse_pos) = response.interact_pointer_pos() {
        if response.dragged() {
            let idx = *selected_idx.min(&mut (nodes.len() - 1));
            let new_temp = (20.0 + ((mouse_pos.x - rect.min.x) / rect.width()) * 80.0).clamp(20.0, 100.0);
            let new_speed = (((rect.max.y - mouse_pos.y) / rect.height()) * 100.0).clamp(0.0, 100.0);

            // Don't cross neighbor temp nodes
            let min_temp = if idx > 0 { nodes[idx - 1].temp + 2.0 } else { 20.0 };
            let max_temp = if idx + 1 < nodes.len() { nodes[idx + 1].temp - 2.0 } else { 100.0 };

            nodes[idx].temp = new_temp.clamp(min_temp, max_temp);
            nodes[idx].speed = new_speed;
        } else if response.clicked() {
            // Find closest node
            for (i, node) in nodes.iter().enumerate() {
                let p = node_to_screen(node);
                if (p.x - mouse_pos.x).abs() < 12.0 && (p.y - mouse_pos.y).abs() < 12.0 {
                    *selected_idx = i;
                    break;
                }
            }
        }
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
