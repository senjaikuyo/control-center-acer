use eframe::egui::{self, Color32, Pos2, Stroke, Vec2};
use std::fs;
use std::process::Command;
use std::time::{Duration, Instant};

#[derive(Clone, Copy, PartialEq, Eq)]
enum PowerMode {
    Quiet,
    Entertainment,
    Performance,
    Turbo,
}

enum FanAction {
    SetAuto,
    SetManual(u8),
}

struct AcerApp {
    power_mode: PowerMode,
    cpu_temp: u32,
    cpu_freq_ghz: f32,
    gpu_temp: u32,

    cpu_fan_auto: bool,
    cpu_fan_speed: u8,

    gpu_fan_auto: bool,
    gpu_fan_speed: u8,

    last_update: Instant,
}

impl Default for AcerApp {
    fn default() -> Self {
        let mut app = Self {
            power_mode: PowerMode::Entertainment,
            cpu_temp: 45,
            cpu_freq_ghz: 2.1,
            gpu_temp: 42,
            cpu_fan_auto: true,
            cpu_fan_speed: 50,
            gpu_fan_auto: true,
            gpu_fan_speed: 50,
            last_update: Instant::now() - Duration::from_secs(2),
        };
        app.update_hardware_stats();
        app
    }
}

impl AcerApp {
    fn update_hardware_stats(&mut self) {
        // 1. Read CPU Temp
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

        // 2. Read CPU Frequency
        if let Ok(freq_str) = fs::read_to_string("/sys/devices/system/cpu/cpu0/cpufreq/scaling_cur_freq") {
            if let Ok(khz) = freq_str.trim().parse::<f32>() {
                self.cpu_freq_ghz = (khz / 1_000_000.0 * 100.0).round() / 100.0;
            }
        }

        // 3. Read GPU Temp
        if let Ok(output) = Command::new("nvidia-smi")
            .args(["--query-gpu=temperature.gpu", "--format=csv,noheader"])
            .output()
        {
            if let Ok(s) = String::from_utf8(output.stdout) {
                if let Ok(t) = s.trim().parse::<u32>() {
                    self.gpu_temp = t;
                }
            }
        }
    }

    fn apply_fan_speed(&self, fan_id: u8, speed: u8) {
        let _ = Command::new("busctl")
            .args([
                "call",
                "com.tux.Tailor",
                "/com/tux/Tailor",
                "com.tux.Tailor.Fan",
                "OverrideSpeed",
                "yy",
                &fan_id.to_string(),
                &speed.to_string(),
            ])
            .output();
    }

    fn set_power_mode(&mut self, mode: PowerMode) {
        self.power_mode = mode;
        match mode {
            PowerMode::Quiet => {
                let _ = Command::new("/usr/bin/python3")
                    .args(["/usr/bin/powerprofilesctl", "set", "power-saver"])
                    .output();
                self.cpu_fan_auto = true;
                self.gpu_fan_auto = true;
                self.apply_fan_speed(0, 0);
                self.apply_fan_speed(1, 0);
            }
            PowerMode::Entertainment => {
                let _ = Command::new("/usr/bin/python3")
                    .args(["/usr/bin/powerprofilesctl", "set", "balanced"])
                    .output();
                self.cpu_fan_auto = true;
                self.gpu_fan_auto = true;
                self.apply_fan_speed(0, 0);
                self.apply_fan_speed(1, 0);
            }
            PowerMode::Performance => {
                let _ = Command::new("/usr/bin/python3")
                    .args(["/usr/bin/powerprofilesctl", "set", "performance"])
                    .output();
                self.cpu_fan_auto = true;
                self.gpu_fan_auto = true;
                self.apply_fan_speed(0, 0);
                self.apply_fan_speed(1, 0);
            }
            PowerMode::Turbo => {
                let _ = Command::new("/usr/bin/python3")
                    .args(["/usr/bin/powerprofilesctl", "set", "performance"])
                    .output();
                self.cpu_fan_auto = false;
                self.gpu_fan_auto = false;
                self.cpu_fan_speed = 100;
                self.gpu_fan_speed = 100;
                self.apply_fan_speed(0, 100);
                self.apply_fan_speed(1, 100);
            }
        }
    }
}

impl eframe::App for AcerApp {
    fn ui(&mut self, ui: &mut egui::Ui, _frame: &mut eframe::Frame) {
        // Periodic hardware update (every 1 second)
        if self.last_update.elapsed() >= Duration::from_secs(1) {
            self.update_hardware_stats();
            self.last_update = Instant::now();
        }
        ui.ctx().request_repaint_after(Duration::from_millis(500));

        let cyan = Color32::from_rgb(0, 210, 255);
        let dark_bg = Color32::from_rgb(14, 17, 23);
        let card_bg = Color32::from_rgb(22, 27, 34);
        let border_col = Color32::from_rgb(48, 54, 61);

        // Fill background
        let rect = ui.max_rect();
        ui.painter().rect_filled(rect, 0.0, dark_bg);

        ui.spacing_mut().item_spacing = Vec2::new(14.0, 14.0);

        // --- HEADER BAR ---
        ui.horizontal(|ui| {
            ui.add_space(10.0);
            ui.vertical(|ui| {
                ui.label(
                    egui::RichText::new("ACER CONTROL CENTER")
                                .size(22.0)
                                .strong()
                                .color(cyan),
                );
                ui.label(
                    egui::RichText::new("Aspire 7 Series • Clevo Linux Edition (Rust Native)")
                        .size(12.0)
                        .color(Color32::from_rgb(139, 148, 158)),
                );
            });
            ui.with_layout(egui::Layout::right_to_left(egui::Align::Center), |ui| {
                ui.add_space(10.0);
                let pill_bg = Color32::from_rgb(35, 134, 54);
                egui::Frame::new()
                    .fill(pill_bg)
                    .corner_radius(6)
                    .inner_margin(egui::Margin::symmetric(10, 5))
                    .show(ui, |ui| {
                        ui.label(
                            egui::RichText::new("ONLINE • TAILORD")
                                .size(11.0)
                                .strong()
                                .color(Color32::WHITE),
                        );
                    });
            });
        });

        ui.separator();

        // --- SECTION 1: POWER MODES (CARDS) ---
        ui.label(
            egui::RichText::new("POWER & THERMAL MODES")
                .size(13.0)
                .strong()
                .color(Color32::from_rgb(201, 209, 217)),
        );

        ui.columns(4, |cols| {
            let modes = [
                (PowerMode::Quiet, "Quiet", "Silent acoustics\nPower saving", Color32::from_rgb(46, 160, 67)),
                (PowerMode::Entertainment, "Entertainment", "Balanced power\nMedia & everyday", Color32::from_rgb(31, 111, 235)),
                (PowerMode::Performance, "Performance", "High CPU boost\nIntense workloads", Color32::from_rgb(210, 153, 34)),
                (PowerMode::Turbo, "Turbo (Max Fan)", "100% full fan blast\nHeavy gaming", Color32::from_rgb(248, 81, 73)),
            ];

            for (i, (mode, title, desc, accent)) in modes.into_iter().enumerate() {
                let is_active = self.power_mode == mode;
                let frame = egui::Frame::new()
                    .fill(if is_active { Color32::from_rgb(33, 38, 45) } else { card_bg })
                    .corner_radius(10)
                    .stroke(Stroke::new(
                        if is_active { 2.0 } else { 1.0 },
                        if is_active { accent } else { border_col },
                    ))
                    .inner_margin(12);

                frame.show(&mut cols[i], |ui| {
                    ui.vertical_centered(|ui| {
                        ui.label(
                            egui::RichText::new(title)
                                .size(15.0)
                                .strong()
                                .color(if is_active { accent } else { Color32::WHITE }),
                        );
                        ui.add_space(4.0);
                        ui.label(
                            egui::RichText::new(desc)
                                .size(11.0)
                                .color(Color32::from_rgb(139, 148, 158)),
                        );
                        ui.add_space(8.0);
                        let btn_text = if is_active { "Active" } else { "Select" };
                        if ui.button(btn_text).clicked() {
                            self.set_power_mode(mode);
                        }
                    });
                });
            }
        });

        ui.add_space(8.0);

        // --- SECTION 2: DUAL FAN SPEEDOMETERS & GAUGES ---
        let mut cpu_action = None;
        let mut gpu_action = None;

        ui.columns(2, |cols| {
            // CPU FAN GAUGE
            cpu_action = render_fan_card(
                &mut cols[0],
                "CPU THERMAL & FAN (FAN 0)",
                self.cpu_temp,
                &format!("{:.2} GHz", self.cpu_freq_ghz),
                self.cpu_fan_auto,
                self.cpu_fan_speed,
                Color32::from_rgb(88, 166, 255),
            );

            // GPU FAN GAUGE
            gpu_action = render_fan_card(
                &mut cols[1],
                "GPU THERMAL & FAN (FAN 1)",
                self.gpu_temp,
                "RTX 3050 6GB",
                self.gpu_fan_auto,
                self.gpu_fan_speed,
                Color32::from_rgb(63, 185, 80),
            );
        });

        if let Some(action) = cpu_action {
            match action {
                FanAction::SetAuto => {
                    self.cpu_fan_auto = true;
                    self.apply_fan_speed(0, 0);
                }
                FanAction::SetManual(s) => {
                    self.cpu_fan_auto = false;
                    self.cpu_fan_speed = s;
                    self.apply_fan_speed(0, s);
                }
            }
        }

        if let Some(action) = gpu_action {
            match action {
                FanAction::SetAuto => {
                    self.gpu_fan_auto = true;
                    self.apply_fan_speed(1, 0);
                }
                FanAction::SetManual(s) => {
                    self.gpu_fan_auto = false;
                    self.gpu_fan_speed = s;
                    self.apply_fan_speed(1, s);
                }
            }
        }
    }
}

fn render_fan_card(
    ui: &mut egui::Ui,
    title: &str,
    temp: u32,
    subtext: &str,
    is_auto: bool,
    mut speed: u8,
    accent: Color32,
) -> Option<FanAction> {
    let card_bg = Color32::from_rgb(22, 27, 34);
    let border_col = Color32::from_rgb(48, 54, 61);
    let mut result_action = None;

    egui::Frame::new()
        .fill(card_bg)
        .corner_radius(10)
        .stroke(Stroke::new(1.0, border_col))
        .inner_margin(16)
        .show(ui, |ui| {
            ui.label(
                egui::RichText::new(title)
                    .size(13.0)
                    .strong()
                    .color(Color32::from_rgb(201, 209, 217)),
            );

            ui.add_space(6.0);

            // Tachometer / Circular dial visual
            let (rect, _) = ui.allocate_exact_size(Vec2::new(ui.available_width(), 130.0), egui::Sense::hover());
            let painter = ui.painter_at(rect);
            let center = Pos2::new(rect.center().x, rect.center().y + 15.0);
            let radius = 60.0;

            // Background Arc (Speedometer rail)
            let start_angle = std::f32::consts::PI * 0.8;
            let end_angle = std::f32::consts::PI * 2.2;
            let active_pct = if is_auto { (temp as f32 / 100.0).clamp(0.2, 1.0) } else { (speed as f32 / 100.0).clamp(0.0, 1.0) };
            let needle_angle = start_angle + (end_angle - start_angle) * active_pct;

            // Draw track
            for i in 0..30 {
                let a1 = start_angle + (end_angle - start_angle) * (i as f32 / 30.0);
                let a2 = start_angle + (end_angle - start_angle) * ((i + 1) as f32 / 30.0);
                let p1 = Pos2::new(center.x + radius * a1.cos(), center.y + radius * a1.sin());
                let p2 = Pos2::new(center.x + radius * a2.cos(), center.y + radius * a2.sin());
                painter.line_segment([p1, p2], Stroke::new(4.0, Color32::from_rgb(33, 38, 45)));
            }

            // Draw active colored track
            let active_segments = (30.0 * active_pct) as usize;
            for i in 0..active_segments {
                let a1 = start_angle + (end_angle - start_angle) * (i as f32 / 30.0);
                let a2 = start_angle + (end_angle - start_angle) * ((i + 1) as f32 / 30.0);
                let p1 = Pos2::new(center.x + radius * a1.cos(), center.y + radius * a1.sin());
                let p2 = Pos2::new(center.x + radius * a2.cos(), center.y + radius * a2.sin());
                painter.line_segment([p1, p2], Stroke::new(5.0, accent));
            }

            // Draw Needle
            let needle_len = radius - 8.0;
            let needle_end = Pos2::new(center.x + needle_len * needle_angle.cos(), center.y + needle_len * needle_angle.sin());
            painter.line_segment([center, needle_end], Stroke::new(2.5, Color32::WHITE));
            painter.circle_filled(center, 5.0, accent);

            // Center Text (Temperature Badge)
            painter.text(
                Pos2::new(center.x, center.y - 12.0),
                egui::Align2::CENTER_CENTER,
                format!("{}°C", temp),
                egui::FontId::proportional(22.0),
                accent,
            );
            painter.text(
                Pos2::new(center.x, center.y + 35.0),
                egui::Align2::CENTER_CENTER,
                subtext,
                egui::FontId::proportional(11.0),
                Color32::from_rgb(139, 148, 158),
            );

            ui.add_space(10.0);

            // Controls
            ui.horizontal(|ui| {
                if ui.selectable_label(is_auto, "Auto Curve").clicked() {
                    result_action = Some(FanAction::SetAuto);
                }
                if ui.selectable_label(!is_auto, "Manual").clicked() {
                    result_action = Some(FanAction::SetManual(speed));
                }
                if !is_auto {
                    ui.label(format!("{}%", speed));
                }
            });

            if !is_auto {
                let mut val = speed as f32;
                if ui.add(egui::Slider::new(&mut val, 20.0..=100.0).text("% Fan Speed")).changed() {
                    speed = val as u8;
                    result_action = Some(FanAction::SetManual(speed));
                }
            }
        });

    result_action
}

fn main() -> eframe::Result<()> {
    let native_options = eframe::NativeOptions {
        viewport: egui::ViewportBuilder::default()
            .with_inner_size([920.0, 580.0])
            .with_min_inner_size([800.0, 520.0])
            .with_title("Acer Control Center 3.0 (Linux Edition)"),
        ..Default::default()
    };

    eframe::run_native(
        "com.acer.ControlCenterLinux",
        native_options,
        Box::new(|_cc| Ok(Box::new(AcerApp::default()))),
    )
}
