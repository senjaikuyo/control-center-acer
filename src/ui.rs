use crate::backend::{FanCurveNode, FanMode, PowerMode};
use crate::AcerApp;
use eframe::egui::{self, Color32, FontId, Pos2, Rect, Stroke, StrokeKind, Vec2};
use std::f32::consts::PI;

// ============================================================================
// UNIFIED DESIGN TOKENS (PIXEL-PERFECT OEM CONTROL CENTER 3.0)
// ============================================================================
pub const COLOR_BG: Color32 = Color32::from_rgb(11, 14, 20);            // Canvas background
pub const COLOR_CARD_BG: Color32 = Color32::from_rgb(17, 21, 30);       // Card background
pub const COLOR_CARD_BORDER: Color32 = Color32::from_rgb(32, 40, 56);   // Card border
pub const CARD_CORNER_RADIUS: f32 = 6.0;
pub const CARD_PADDING: f32 = 12.0;
pub const GRID_GAP: f32 = 10.0;

pub const COLOR_CYAN: Color32 = Color32::from_rgb(0, 225, 255);
pub const COLOR_CYAN_DIM: Color32 = Color32::from_rgb(12, 36, 50);
pub const COLOR_GOLD: Color32 = Color32::from_rgb(255, 179, 0);
pub const COLOR_PURPLE_GLOW: Color32 = Color32::from_rgb(156, 39, 176);

pub const COLOR_TEXT_WHITE: Color32 = Color32::from_rgb(240, 246, 252);
pub const COLOR_TEXT_MUTED: Color32 = Color32::from_rgb(139, 148, 158);
pub const COLOR_TEXT_DARK: Color32 = Color32::from_rgb(90, 100, 115);

pub const COLOR_TEMP_GREEN: Color32 = Color32::from_rgb(0, 230, 118);
pub const COLOR_TEMP_YELLOW: Color32 = Color32::from_rgb(255, 214, 0);
pub const COLOR_TEMP_RED: Color32 = Color32::from_rgb(255, 61, 0);

// ============================================================================
// MAIN ENTRY POINT
// ============================================================================
pub fn render(app: &mut AcerApp, ctx: &egui::Context, ui: &mut egui::Ui) {
    let full_rect = ui.max_rect();
    ui.painter().rect_filled(full_rect, 0.0, COLOR_BG);

    // Outer margin
    egui::Frame::new()
        .inner_margin(egui::Margin::symmetric(14, 10))
        .show(ui, |ui| {
            ui.spacing_mut().item_spacing = Vec2::new(GRID_GAP, GRID_GAP);

            // 1. Unified Top Navigation Header
            render_top_header(app, ctx, ui);

            // 2. Main Content Routing
            match app.active_tab {
                1 => render_lighting_tab(app, ui),
                2 => render_battery_tab(app, ui),
                3 => render_settings_tab(app, ui),
                _ => render_dashboard_grid(app, ui),
            }
        });
}

// ============================================================================
// 1. TOP NAVIGATION HEADER
// ============================================================================
fn render_top_header(app: &mut AcerApp, ctx: &egui::Context, ui: &mut egui::Ui) {
    ui.horizontal(|ui| {
        // Left: 4 Category Tabs
        let tabs = [
            (0, "Dashboard / Telemetry"),
            (1, "Chassis & Keyboard Lighting"),
            (2, "Battery & Power Management"),
            (3, "Device Settings"),
        ];

        for (idx, hover_text) in tabs {
            let is_active = app.active_tab == idx;
            let (rect, resp) = ui.allocate_exact_size(Vec2::new(32.0, 28.0), egui::Sense::click());
            let painter = ui.painter_at(rect);

            let bg = if is_active {
                COLOR_CYAN_DIM
            } else if resp.hovered() {
                Color32::from_rgb(24, 30, 42)
            } else {
                Color32::TRANSPARENT
            };
            let stroke = if is_active {
                Stroke::new(1.5, COLOR_CYAN)
            } else {
                Stroke::NONE
            };
            painter.rect(rect, 4.0, bg, stroke, StrokeKind::Inside);

            let icon_col = if is_active {
                COLOR_CYAN
            } else if resp.hovered() {
                COLOR_TEXT_WHITE
            } else {
                COLOR_TEXT_MUTED
            };

            match idx {
                0 => draw_icon_dashboard(&painter, rect, icon_col),
                1 => draw_icon_lighting(&painter, rect, icon_col),
                2 => draw_icon_battery(&painter, rect, icon_col),
                _ => draw_icon_settings(&painter, rect, icon_col),
            }

            if resp.clicked() {
                app.active_tab = idx;
            }
            let _ = resp.on_hover_text(hover_text);
            ui.add_space(2.0);
        }

        // Center: Title & Power Profile Switcher
        let avail_w = ui.available_width() - 110.0;
        let center_offset = (avail_w - 320.0) / 2.0;
        if center_offset > 0.0 {
            ui.add_space(center_offset);
        }

        ui.vertical_centered(|ui| {
            ui.label(
                egui::RichText::new("Control Center")
                    .size(15.5)
                    .strong()
                    .color(COLOR_TEXT_WHITE),
            );

            ui.add_space(2.0);

            // Power profiles bar
            ui.horizontal(|ui| {
                let profiles = [
                    (PowerMode::Performance, "Performance"),
                    (PowerMode::Entertainment, "Entertainment"),
                    (PowerMode::Quiet, "Quiet"),
                ];

                for (mode, label) in profiles {
                    let is_active = app.power_mode == mode;
                    let active_col = match mode {
                        PowerMode::Performance => COLOR_GOLD,
                        PowerMode::Entertainment => COLOR_CYAN,
                        PowerMode::Quiet => COLOR_TEMP_GREEN,
                    };
                    let text_col = if is_active { active_col } else { COLOR_TEXT_MUTED };

                    let (rect, resp) = ui.allocate_exact_size(Vec2::new(96.0, 20.0), egui::Sense::click());
                    let painter = ui.painter_at(rect);

                    if resp.hovered() && !is_active {
                        painter.rect_filled(rect, 3.0, Color32::from_rgb(22, 28, 38));
                    }

                    painter.text(
                        Pos2::new(rect.center().x, rect.center().y - 2.0),
                        egui::Align2::CENTER_CENTER,
                        label,
                        FontId::proportional(12.5),
                        text_col,
                    );

                    if is_active {
                        let underline = Rect::from_min_size(
                            Pos2::new(rect.min.x + 8.0, rect.max.y - 2.0),
                            Vec2::new(rect.width() - 16.0, 2.0),
                        );
                        painter.rect_filled(underline, 1.0, active_col);
                    }

                    if resp.clicked() {
                        app.set_power_profile(mode);
                    }

                    let hover_desc = match mode {
                        PowerMode::Performance => "Performance: Full boost clock & aggressive fan curve",
                        PowerMode::Entertainment => "Entertainment: Balanced daily use and acoustics",
                        PowerMode::Quiet => "Quiet: Energy efficient EPP with whisper-silent cooling",
                    };
                    let _ = resp.on_hover_text(hover_desc);

                    ui.add_space(4.0);
                }
            });
        });

        // Right: Window Control Buttons
        ui.with_layout(egui::Layout::right_to_left(egui::Align::Center), |ui| {
            ui.add_space(4.0);

            // Close button
            let (rect, resp) = ui.allocate_exact_size(Vec2::new(26.0, 22.0), egui::Sense::click());
            let painter = ui.painter_at(rect);
            if resp.hovered() {
                painter.rect_filled(rect, 3.0, Color32::from_rgb(220, 38, 38));
            }
            draw_icon_close(&painter, rect, if resp.hovered() { Color32::WHITE } else { COLOR_TEXT_MUTED });
            if resp.clicked() {
                ctx.send_viewport_cmd(egui::ViewportCommand::Close);
            }

            ui.add_space(2.0);

            // Maximize button
            let (rect, resp) = ui.allocate_exact_size(Vec2::new(26.0, 22.0), egui::Sense::click());
            let painter = ui.painter_at(rect);
            if resp.hovered() {
                painter.rect_filled(rect, 3.0, Color32::from_rgb(28, 35, 48));
            }
            draw_icon_maximize(&painter, rect, if resp.hovered() { Color32::WHITE } else { COLOR_TEXT_MUTED });
            if resp.clicked() {
                ctx.send_viewport_cmd(egui::ViewportCommand::Maximized(true));
            }

            ui.add_space(2.0);

            // Minimize button
            let (rect, resp) = ui.allocate_exact_size(Vec2::new(26.0, 22.0), egui::Sense::click());
            let painter = ui.painter_at(rect);
            if resp.hovered() {
                painter.rect_filled(rect, 3.0, Color32::from_rgb(28, 35, 48));
            }
            draw_icon_minimize(&painter, rect, if resp.hovered() { Color32::WHITE } else { COLOR_TEXT_MUTED });
            if resp.clicked() {
                ctx.send_viewport_cmd(egui::ViewportCommand::Minimized(true));
            }
        });
    });
}

// ============================================================================
// 2. DASHBOARD MAIN 3x2 GRID
// ============================================================================
fn render_dashboard_grid(app: &mut AcerApp, ui: &mut egui::Ui) {
    let avail_w = ui.available_width();
    let col_w = ((avail_w - (GRID_GAP * 2.0)) / 3.0).floor();
    let row1_h = 285.0;
    let row2_h = 185.0;

    // --- ROW 1 (3 EQUAL COLUMNS) ---
    ui.horizontal(|ui| {
        // Col 1: System Telemetry
        ui.allocate_ui(Vec2::new(col_w, row1_h), |ui| {
            render_card_frame(ui, col_w, row1_h, |ui| {
                render_system_panel(app, ui);
            });
        });
        ui.add_space(GRID_GAP);

        // Col 2: Center Gauge OR Custom Fan Curve Editor
        ui.allocate_ui(Vec2::new(col_w, row1_h), |ui| {
            render_card_frame(ui, col_w, row1_h, |ui| {
                if app.fan_mode == FanMode::Custom {
                    render_custom_fan_curve_panel(app, ui);
                } else {
                    render_speedometer_gauge_panel(app, ui);
                }
            });
        });
        ui.add_space(GRID_GAP);

        // Col 3: FAN Speed Control
        ui.allocate_ui(Vec2::new(col_w, row1_h), |ui| {
            render_card_frame(ui, col_w, row1_h, |ui| {
                render_fan_speed_control_panel(app, ui);
            });
        });
    });

    ui.add_space(GRID_GAP);

    // --- ROW 2 (3 EQUAL COLUMNS) ---
    ui.horizontal(|ui| {
        // Col 1: Memory and Storage
        ui.allocate_ui(Vec2::new(col_w, row2_h), |ui| {
            render_card_frame(ui, col_w, row2_h, |ui| {
                render_memory_storage_panel(app, ui);
            });
        });
        ui.add_space(GRID_GAP);

        // Col 2: Preference (8 Toggles)
        ui.allocate_ui(Vec2::new(col_w, row2_h), |ui| {
            render_card_frame(ui, col_w, row2_h, |ui| {
                render_preference_panel(app, ui);
            });
        });
        ui.add_space(GRID_GAP);

        // Col 3: Configuration (Blue Light & Display)
        ui.allocate_ui(Vec2::new(col_w, row2_h), |ui| {
            render_card_frame(ui, col_w, row2_h, |ui| {
                render_configuration_panel(app, ui);
            });
        });
    });
}

fn render_card_frame<R>(ui: &mut egui::Ui, width: f32, height: f32, add_contents: impl FnOnce(&mut egui::Ui) -> R) -> R {
    egui::Frame::new()
        .fill(COLOR_CARD_BG)
        .stroke(Stroke::new(1.0, COLOR_CARD_BORDER))
        .corner_radius(CARD_CORNER_RADIUS)
        .inner_margin(CARD_PADDING)
        .show(ui, |ui| {
            ui.set_width(width - CARD_PADDING * 2.0);
            ui.set_height(height - CARD_PADDING * 2.0);
            add_contents(ui)
        })
        .inner
}

// ============================================================================
// PANEL: SYSTEM TELEMETRY (ROW 1, COL 1)
// ============================================================================
fn render_system_panel(app: &AcerApp, ui: &mut egui::Ui) {
    ui.label(
        egui::RichText::new("System")
            .size(14.0)
            .strong()
            .color(COLOR_TEXT_WHITE),
    );

    ui.add_space(2.0);
    ui.label(
        egui::RichText::new(&app.cpu_name)
            .size(10.5)
            .color(COLOR_TEXT_MUTED),
    );
    ui.label(
        egui::RichText::new(&app.gpu_name)
            .size(10.5)
            .color(COLOR_TEXT_MUTED),
    );

    ui.add_space(8.0);

    // CPU telemetry row
    ui.horizontal(|ui| {
        let half_w = (ui.available_width() - 8.0) / 2.0;
        ui.allocate_ui(Vec2::new(half_w, 38.0), |ui| {
            ui.label(
                egui::RichText::new(format!("{} MHz", app.cpu_clock_mhz))
                    .size(19.0)
                    .strong()
                    .color(COLOR_TEXT_WHITE),
            );
            ui.label(egui::RichText::new("CPU clock speed").size(9.5).color(COLOR_CYAN));
        });

        ui.allocate_ui(Vec2::new(half_w, 38.0), |ui| {
            ui.label(
                egui::RichText::new(format!("{} °C", app.cpu_temp))
                    .size(19.0)
                    .strong()
                    .color(COLOR_TEXT_WHITE),
            );
            ui.label(egui::RichText::new("Temperature").size(9.5).color(COLOR_CYAN));
        });
    });

    ui.add_space(6.0);

    // GPU telemetry row
    ui.horizontal(|ui| {
        let half_w = (ui.available_width() - 8.0) / 2.0;
        ui.allocate_ui(Vec2::new(half_w, 38.0), |ui| {
            let clk_str = if app.gpu_clock_mhz > 0 {
                format!("{} MHz", app.gpu_clock_mhz)
            } else {
                "OFF MHz".to_string()
            };
            ui.label(
                egui::RichText::new(clk_str)
                    .size(19.0)
                    .strong()
                    .color(COLOR_TEXT_WHITE),
            );
            ui.label(egui::RichText::new("GPU clock speed").size(9.5).color(COLOR_CYAN));
        });

        ui.allocate_ui(Vec2::new(half_w, 38.0), |ui| {
            let temp_str = if app.gpu_temp > 0 {
                format!("{} °C", app.gpu_temp)
            } else {
                "OFF °C".to_string()
            };
            ui.label(
                egui::RichText::new(temp_str)
                    .size(19.0)
                    .strong()
                    .color(COLOR_TEXT_WHITE),
            );
            ui.label(egui::RichText::new("Temperature").size(9.5).color(COLOR_CYAN));
        });
    });

    ui.add_space(10.0);

    // CPU Usage bar
    ui.horizontal(|ui| {
        ui.label(egui::RichText::new("< CPU usage").size(10.5).color(COLOR_TEXT_MUTED));
        ui.with_layout(egui::Layout::right_to_left(egui::Align::Center), |ui| {
            ui.label(
                egui::RichText::new(format!("{} %", app.cpu_usage_pct))
                    .size(10.5)
                    .color(COLOR_TEXT_WHITE),
            );
        });
    });
    render_striped_bar(ui, app.cpu_usage_pct, COLOR_CYAN);

    ui.add_space(6.0);

    // GPU Usage bar
    ui.horizontal(|ui| {
        ui.label(egui::RichText::new("< GPU usage").size(10.5).color(COLOR_TEXT_MUTED));
        ui.with_layout(egui::Layout::right_to_left(egui::Align::Center), |ui| {
            ui.label(
                egui::RichText::new(format!("{} %", app.gpu_usage_pct))
                    .size(10.5)
                    .color(COLOR_TEXT_WHITE),
            );
        });
    });
    render_striped_bar(ui, app.gpu_usage_pct, COLOR_CYAN);
}

// ============================================================================
// PANEL: SPEEDOMETER GAUGE (ROW 1, COL 2 - DEFAULT VIEW)
// ============================================================================
fn render_speedometer_gauge_panel(app: &mut AcerApp, ui: &mut egui::Ui) {
    let is_gpu = app.active_curve_tab == 1;
    let mode_str = if is_gpu { "GPU Telemetry" } else { "Processor Telemetry" };

    ui.horizontal(|ui| {
        ui.label(
            egui::RichText::new(mode_str)
                .size(13.5)
                .strong()
                .color(COLOR_TEXT_WHITE),
        );

        ui.with_layout(egui::Layout::right_to_left(egui::Align::Center), |ui| {
            if ui.button(egui::RichText::new(">>").size(13.0).color(COLOR_GOLD)).on_hover_text("Open Custom Fan Curve Editor").clicked() {
                app.set_fan_mode(FanMode::Custom);
            }
        });
    });

    let (rect, resp) = ui.allocate_exact_size(Vec2::new(ui.available_width(), 195.0), egui::Sense::click());
    let painter = ui.painter_at(rect);
    let center = Pos2::new(rect.center().x, rect.center().y + 5.0);
    let radius = 64.0;

    // Toggle CPU/GPU on center hub click
    if resp.clicked() {
        if let Some(pos) = resp.interact_pointer_pos() {
            if pos.distance(center) <= 32.0 {
                app.active_curve_tab = if is_gpu { 0 } else { 1 };
            }
        }
    }

    // 1. Electromagnetic wave lines (amber)
    for side in [-1.0f32, 1.0f32] {
        for wave_idx in 0..4 {
            let offset_y = (wave_idx as f32 - 1.5) * 10.0;
            let mut points = Vec::new();
            for step in 0..20 {
                let t = step as f32 / 19.0;
                let x = center.x + side * (28.0 + t * 76.0);
                let wave = (t * PI * 2.5).sin() * 4.5;
                let y = center.y + offset_y + wave;
                points.push(Pos2::new(x, y));
            }
            let alpha = (110.0 * (1.0 - (wave_idx as f32 * 0.2))) as u8;
            painter.add(egui::Shape::line(
                points,
                Stroke::new(1.0, Color32::from_rgba_premultiplied(255, 179, 0, alpha)),
            ));
        }
    }

    // 2. Center hub
    let hub_label = if is_gpu { "GPU" } else { "CPU" };
    let hub_accent = if is_gpu { COLOR_CYAN } else { COLOR_GOLD };

    painter.circle_stroke(center, 30.0, Stroke::new(1.0, Color32::from_rgba_premultiplied(156, 39, 176, 70)));
    painter.circle_stroke(center, 25.0, Stroke::new(2.0, COLOR_PURPLE_GLOW));
    painter.circle_stroke(center, 20.0, Stroke::new(1.8, hub_accent));
    painter.circle_filled(center, 18.0, Color32::from_rgb(18, 22, 32));
    painter.text(center, egui::Align2::CENTER_CENTER, hub_label, FontId::proportional(11.5), COLOR_TEXT_WHITE);

    // 3. Left Speedometer Arc
    let left_start = PI * 1.5;
    let (clk_val, clk_max, clk_label) = if is_gpu {
        (app.gpu_clock_mhz, 2500.0, "Clock speed\nx100 MHz")
    } else {
        (app.cpu_clock_mhz, 6000.0, "Clock speed\nx1000 MHz")
    };
    let clk_ratio = (clk_val as f32 / clk_max).clamp(0.0, 1.0);
    let needle_left_angle = left_start - (PI * clk_ratio);

    for i in 0..15 {
        let a1 = left_start - (PI * (i as f32 / 15.0));
        let a2 = left_start - (PI * ((i + 1) as f32 / 15.0));
        let p1 = Pos2::new(center.x + radius * a1.cos(), center.y + radius * a1.sin());
        let p2 = Pos2::new(center.x + radius * a2.cos(), center.y + radius * a2.sin());
        painter.line_segment([p1, p2], Stroke::new(3.0, Color32::from_rgb(33, 41, 55)));
    }

    let max_ticks = if is_gpu { 5 } else { 6 };
    for val in 0..=max_ticks {
        let frac = val as f32 / max_ticks as f32;
        let a = left_start - (PI * frac);
        let tick_p = Pos2::new(center.x + (radius + 11.0) * a.cos(), center.y + (radius + 11.0) * a.sin());
        let tick_str = if is_gpu { (val * 5).to_string() } else { val.to_string() };
        painter.text(tick_p, egui::Align2::CENTER_CENTER, tick_str, FontId::proportional(8.5), COLOR_TEXT_MUTED);
    }

    let n1_end = Pos2::new(center.x + (radius - 4.0) * needle_left_angle.cos(), center.y + (radius - 4.0) * needle_left_angle.sin());
    painter.line_segment([center, n1_end], Stroke::new(2.2, hub_accent));
    painter.circle_filled(center, 3.5, hub_accent);

    // 4. Right Temperature Arc
    let right_start = PI * 1.5;
    let temp_val = if is_gpu { app.gpu_temp } else { app.cpu_temp };
    let temp_ratio = (temp_val as f32 / 100.0).clamp(0.0, 1.0);
    let needle_right_angle = right_start + (PI * temp_ratio);

    for i in 0..15 {
        let a1 = right_start + (PI * (i as f32 / 15.0));
        let a2 = right_start + (PI * ((i + 1) as f32 / 15.0));
        let p1 = Pos2::new(center.x + radius * a1.cos(), center.y + radius * a1.sin());
        let p2 = Pos2::new(center.x + radius * a2.cos(), center.y + radius * a2.sin());
        let col = if i < 5 { COLOR_TEMP_GREEN } else if i < 10 { COLOR_TEMP_YELLOW } else { COLOR_TEMP_RED };
        painter.line_segment([p1, p2], Stroke::new(3.5, col));
    }

    let needle_right_col = if is_gpu { COLOR_GOLD } else { COLOR_CYAN };
    let n2_end = Pos2::new(center.x + (radius - 4.0) * needle_right_angle.cos(), center.y + (radius - 4.0) * needle_right_angle.sin());
    painter.line_segment([center, n2_end], Stroke::new(2.2, needle_right_col));
    painter.circle_filled(center, 3.0, needle_right_col);

    // Bottom labels
    painter.text(
        Pos2::new(center.x - radius - 8.0, center.y + 16.0),
        egui::Align2::RIGHT_CENTER,
        clk_label,
        FontId::proportional(9.0),
        COLOR_TEXT_MUTED,
    );

    let temp_label = if is_gpu && temp_val == 0 {
        "SLEEP\nOptimus".to_string()
    } else {
        format!("{} °C\nTemperature", temp_val)
    };

    painter.text(
        Pos2::new(center.x + radius + 8.0, center.y + 16.0),
        egui::Align2::LEFT_CENTER,
        temp_label,
        FontId::proportional(9.0),
        COLOR_TEXT_MUTED,
    );
}

// ============================================================================
// PANEL: CUSTOM FAN SETTING CURVE (ROW 1, COL 2 - CUSTOM MODE VIEW)
// ============================================================================
fn render_custom_fan_curve_panel(app: &mut AcerApp, ui: &mut egui::Ui) {
    ui.horizontal(|ui| {
        ui.label(
            egui::RichText::new("Custom fan setting")
                .size(13.5)
                .strong()
                .color(COLOR_TEXT_WHITE),
        );

        ui.with_layout(egui::Layout::right_to_left(egui::Align::Center), |ui| {
            if ui.button(egui::RichText::new(">>").size(13.0).color(COLOR_GOLD)).on_hover_text("Return to standard gauge").clicked() {
                app.set_fan_mode(FanMode::Automatic);
            }
        });
    });

    ui.add_space(2.0);

    // Subtabs: CPU Fan | GPU Fan
    ui.horizontal(|ui| {
        let is_cpu = app.active_curve_tab == 0;
        let is_gpu = app.active_curve_tab == 1;

        // CPU Fan Tab
        let (cpu_rect, cpu_resp) = ui.allocate_exact_size(Vec2::new(72.0, 20.0), egui::Sense::click());
        let cp = ui.painter_at(cpu_rect);
        cp.text(
            Pos2::new(cpu_rect.center().x, cpu_rect.center().y - 1.0),
            egui::Align2::CENTER_CENTER,
            "CPU Fan",
            FontId::proportional(12.0),
            if is_cpu { COLOR_GOLD } else { COLOR_TEXT_MUTED },
        );
        if is_cpu {
            let underline = Rect::from_min_size(Pos2::new(cpu_rect.min.x + 6.0, cpu_rect.max.y - 2.0), Vec2::new(cpu_rect.width() - 12.0, 2.0));
            cp.rect_filled(underline, 1.0, COLOR_GOLD);
        }
        if cpu_resp.clicked() {
            app.active_curve_tab = 0;
        }

        ui.add_space(6.0);

        // GPU Fan Tab
        let (gpu_rect, gpu_resp) = ui.allocate_exact_size(Vec2::new(72.0, 20.0), egui::Sense::click());
        let gp = ui.painter_at(gpu_rect);
        gp.text(
            Pos2::new(gpu_rect.center().x, gpu_rect.center().y - 1.0),
            egui::Align2::CENTER_CENTER,
            "GPU Fan",
            FontId::proportional(12.0),
            if is_gpu { COLOR_CYAN } else { COLOR_TEXT_MUTED },
        );
        if is_gpu {
            let underline = Rect::from_min_size(Pos2::new(gpu_rect.min.x + 6.0, gpu_rect.max.y - 2.0), Vec2::new(gpu_rect.width() - 12.0, 2.0));
            gp.rect_filled(underline, 1.0, COLOR_CYAN);
        }
        if gpu_resp.clicked() {
            app.active_curve_tab = 1;
        }
    });

    ui.add_space(2.0);

    let line_col = if app.active_curve_tab == 0 { COLOR_GOLD } else { COLOR_CYAN };
    let nodes = if app.active_curve_tab == 0 { &mut app.cpu_curve_nodes } else { &mut app.gpu_curve_nodes };

    let changed_by_canvas = render_fan_curve_grid(ui, nodes, line_col, &mut app.selected_node_idx);

    let active_nodes = if app.active_curve_tab == 0 { &app.cpu_curve_nodes } else { &app.gpu_curve_nodes };
    let idx = app.selected_node_idx.min(active_nodes.len().saturating_sub(1));
    let cur_temp = active_nodes[idx].temp.round() as u32;
    let cur_speed = active_nodes[idx].speed.round() as u32;

    ui.add_space(3.0);

    // Fine-tune controls row
    let mut fine_tune_changed = false;
    let mut temp_val = active_nodes[idx].temp;
    let mut speed_val = active_nodes[idx].speed;

    ui.horizontal(|ui| {
        ui.label(egui::RichText::new(format!("P{} Temp:", idx + 1)).size(10.0).color(COLOR_TEXT_MUTED));
        let min_t = if idx > 0 { active_nodes[idx - 1].temp + 1.0 } else { 20.0 };
        let max_t = if idx + 1 < active_nodes.len() { active_nodes[idx + 1].temp - 1.0 } else { 100.0 };
        if ui.add(egui::Slider::new(&mut temp_val, min_t..=max_t).suffix("°C").show_value(true)).changed() {
            fine_tune_changed = true;
        }

        ui.add_space(4.0);
        ui.label(egui::RichText::new("Duty:").size(10.0).color(COLOR_TEXT_MUTED));
        if ui.add(egui::Slider::new(&mut speed_val, 0.0..=100.0).suffix("%").show_value(true)).changed() {
            fine_tune_changed = true;
        }
    });

    if fine_tune_changed {
        let nodes_mut = if app.active_curve_tab == 0 { &mut app.cpu_curve_nodes } else { &mut app.gpu_curve_nodes };
        nodes_mut[idx].temp = temp_val;
        nodes_mut[idx].speed = speed_val;
    }

    if changed_by_canvas || fine_tune_changed {
        app.curve_has_unsaved_changes = true;
    }

    ui.add_space(3.0);
    ui.horizontal(|ui| {
        if ui.button(egui::RichText::new("⟲").size(11.0).color(COLOR_TEXT_MUTED)).on_hover_text("Reset to default curve").clicked() {
            app.reset_active_curve();
        }

        ui.add_space(4.0);
        ui.label(egui::RichText::new("Node:").size(10.0).color(COLOR_TEXT_MUTED));
        ui.label(
            egui::RichText::new(format!("P{} ({}°C -> {}%)", idx + 1, cur_temp, cur_speed))
                .size(10.5)
                .strong()
                .color(line_col),
        );

        ui.with_layout(egui::Layout::right_to_left(egui::Align::Center), |ui| {
            let has_unsaved = app.curve_has_unsaved_changes;
            let btn_label = if has_unsaved { "● Apply & Save" } else { "💾 Apply" };
            let btn_color = if has_unsaved { COLOR_GOLD } else { COLOR_CYAN };
            if ui.button(egui::RichText::new(btn_label).size(10.5).color(btn_color)).clicked() {
                app.sync_custom_curve_to_tailord();
            }
        });
    });

    if app.curve_has_unsaved_changes {
        ui.label(egui::RichText::new("● Unsaved curve changes — click 'Apply & Save'").size(9.5).color(COLOR_GOLD));
    } else if let Some(ref msg) = app.curve_feedback {
        let fb_col = if msg.contains("✓") { COLOR_TEMP_GREEN } else { COLOR_GOLD };
        ui.label(egui::RichText::new(msg).size(9.5).color(fb_col));
    }
}

fn render_fan_curve_grid(ui: &mut egui::Ui, nodes: &mut Vec<FanCurveNode>, line_col: Color32, selected_idx: &mut usize) -> bool {
    let (rect, response) = ui.allocate_exact_size(Vec2::new(ui.available_width(), 122.0), egui::Sense::click_and_drag());
    let painter = ui.painter_at(rect);

    painter.rect_filled(rect, 3.0, Color32::from_rgb(11, 14, 20));
    painter.rect_stroke(rect, 3.0, Stroke::new(1.0, Color32::from_rgb(26, 33, 46)), StrokeKind::Inside);

    let pad_left = 24.0;
    let pad_right = 14.0;
    let pad_top = 8.0;
    let pad_bottom = 16.0;

    let graph_rect = Rect::from_min_max(
        Pos2::new(rect.min.x + pad_left, rect.min.y + pad_top),
        Pos2::new(rect.max.x - pad_right, rect.max.y - pad_bottom),
    );

    let grid_stroke = Stroke::new(0.8, Color32::from_rgb(22, 28, 38));

    // Y Gridlines (0% to 100%)
    for y_pct in [0.0, 0.25, 0.5, 0.75, 1.0] {
        let y = graph_rect.max.y - (graph_rect.height() * y_pct);
        painter.line_segment([Pos2::new(graph_rect.min.x, y), Pos2::new(graph_rect.max.x, y)], grid_stroke);
        let val_pct = (y_pct * 100.0).round() as u32;
        painter.text(Pos2::new(rect.min.x + pad_left - 3.0, y), egui::Align2::RIGHT_CENTER, format!("{}", val_pct), FontId::proportional(8.0), COLOR_TEXT_DARK);
    }

    // X Gridlines (20°C to 100°C)
    for x_frac in [0.0, 0.25, 0.5, 0.75, 1.0] {
        let x = graph_rect.min.x + (graph_rect.width() * x_frac);
        painter.line_segment([Pos2::new(x, graph_rect.min.y), Pos2::new(x, graph_rect.max.y)], grid_stroke);
        let temp_val = 20.0 + x_frac * 80.0;
        painter.text(Pos2::new(x, graph_rect.max.y + 8.0), egui::Align2::CENTER_CENTER, format!("{}", temp_val.round() as u32), FontId::proportional(8.0), COLOR_TEXT_DARK);
    }

    let node_to_screen = |node: &FanCurveNode| -> Pos2 {
        let t_norm = ((node.temp - 20.0) / 80.0).clamp(0.0, 1.0);
        let s_norm = (node.speed / 100.0).clamp(0.0, 1.0);
        let x = graph_rect.min.x + t_norm * graph_rect.width();
        let y = graph_rect.max.y - s_norm * graph_rect.height();
        Pos2::new(x, y)
    };

    for i in 0..nodes.len().saturating_sub(1) {
        let p1 = node_to_screen(&nodes[i]);
        let p2 = node_to_screen(&nodes[i + 1]);
        painter.line_segment([p1, p2], Stroke::new(2.2, line_col));
    }

    for (i, node) in nodes.iter().enumerate() {
        let p = node_to_screen(node);
        let is_sel = *selected_idx == i;
        if is_sel {
            painter.circle_stroke(p, 7.5, Stroke::new(1.2, line_col));
            painter.circle_filled(p, 5.0, COLOR_TEXT_WHITE);
            painter.circle_stroke(p, 5.0, Stroke::new(1.8, line_col));
        } else {
            painter.circle_filled(p, 4.0, COLOR_TEXT_WHITE);
            painter.circle_stroke(p, 4.0, Stroke::new(1.6, line_col));
        }
    }

    if let Some(mouse_pos) = response.interact_pointer_pos() {
        if response.dragged() {
            let idx = *selected_idx.min(&mut (nodes.len() - 1));
            let new_t = (20.0 + ((mouse_pos.x - graph_rect.min.x) / graph_rect.width()) * 80.0).clamp(20.0, 100.0);
            let new_s = (((graph_rect.max.y - mouse_pos.y) / graph_rect.height()) * 100.0).clamp(0.0, 100.0);

            let min_t = if idx > 0 { nodes[idx - 1].temp + 1.0 } else { 20.0 };
            let max_t = if idx + 1 < nodes.len() { nodes[idx + 1].temp - 1.0 } else { 100.0 };

            nodes[idx].temp = new_t.clamp(min_t, max_t);
            nodes[idx].speed = new_s;
        } else if response.clicked() {
            for (i, node) in nodes.iter().enumerate() {
                let p = node_to_screen(node);
                if (p.x - mouse_pos.x).hypot(p.y - mouse_pos.y) < 14.0 {
                    *selected_idx = i;
                    break;
                }
            }
        }
    }

    response.dragged()
}

// ============================================================================
// PANEL: FAN SPEED CONTROL (ROW 1, COL 3)
// ============================================================================
fn render_fan_speed_control_panel(app: &mut AcerApp, ui: &mut egui::Ui) {
    ui.label(
        egui::RichText::new("FAN speed control")
            .size(14.0)
            .strong()
            .color(COLOR_TEXT_WHITE),
    );

    ui.add_space(4.0);

    // Radio button list
    let modes = [
        (FanMode::Automatic, "Automatic"),
        (FanMode::Maximum, "Maximum"),
        (FanMode::Custom, "Custom"),
        (FanMode::Turbo, "Turbo"),
    ];

    for (m, lbl) in modes {
        let is_active = app.fan_mode == m;
        ui.horizontal(|ui| {
            let (radio_rect, radio_resp) = ui.allocate_exact_size(Vec2::new(14.0, 14.0), egui::Sense::click());
            let rp = ui.painter_at(radio_rect);
            let center = radio_rect.center();

            let stroke_col = if is_active { COLOR_GOLD } else { Color32::from_rgb(45, 54, 70) };
            rp.circle_stroke(center, 6.0, Stroke::new(1.4, stroke_col));
            if is_active {
                rp.circle_filled(center, 3.5, COLOR_GOLD);
            }

            let text_col = if is_active { COLOR_GOLD } else { COLOR_TEXT_MUTED };
            let text_resp = ui.selectable_label(false, egui::RichText::new(lbl).size(12.0).color(text_col));

            if radio_resp.clicked() || text_resp.clicked() {
                app.set_fan_mode(m);
            }

            if m == FanMode::Custom {
                let arrow_lbl = if is_active { "<<" } else { ">>" };
                if ui.small_button(egui::RichText::new(arrow_lbl).size(10.0).color(COLOR_GOLD)).clicked() {
                    app.set_fan_mode(FanMode::Custom);
                }
            }
        });
        ui.add_space(1.0);
    }

    ui.add_space(4.0);
    ui.separator();
    ui.add_space(4.0);

    // Dual RPM Readouts
    ui.horizontal(|ui| {
        let half_w = (ui.available_width() - 8.0) / 2.0;
        ui.allocate_ui(Vec2::new(half_w, 36.0), |ui| {
            ui.label(
                egui::RichText::new(format!("{} RPM", app.cpu_fan_rpm))
                    .size(18.0)
                    .strong()
                    .color(COLOR_TEXT_WHITE),
            );
            ui.label(egui::RichText::new("CPU fan speed").size(9.5).color(COLOR_CYAN));
        });

        ui.allocate_ui(Vec2::new(half_w, 36.0), |ui| {
            ui.label(
                egui::RichText::new(format!("{} RPM", app.gpu_fan_rpm))
                    .size(18.0)
                    .strong()
                    .color(COLOR_TEXT_WHITE),
            );
            ui.label(egui::RichText::new("GPU fan speed").size(9.5).color(COLOR_CYAN));
        });
    });

    ui.add_space(8.0);

    // CPU Fan bar
    ui.horizontal(|ui| {
        ui.label(egui::RichText::new("CPU FAN").size(10.5).color(COLOR_TEXT_MUTED));
        ui.with_layout(egui::Layout::right_to_left(egui::Align::Center), |ui| {
            ui.label(egui::RichText::new(format!("{} %", app.cpu_fan_pct)).size(10.5).color(COLOR_TEXT_WHITE));
        });
    });
    render_striped_bar(ui, app.cpu_fan_pct, COLOR_CYAN);

    ui.add_space(4.0);

    // GPU Fan bar
    ui.horizontal(|ui| {
        ui.label(egui::RichText::new("GPU FAN").size(10.5).color(COLOR_TEXT_MUTED));
        ui.with_layout(egui::Layout::right_to_left(egui::Align::Center), |ui| {
            ui.label(egui::RichText::new(format!("{} %", app.gpu_fan_pct)).size(10.5).color(COLOR_TEXT_WHITE));
        });
    });
    render_striped_bar(ui, app.gpu_fan_pct, COLOR_CYAN);
}

// ============================================================================
// PANEL: MEMORY & STORAGE (ROW 2, COL 1)
// ============================================================================
fn render_memory_storage_panel(app: &AcerApp, ui: &mut egui::Ui) {
    ui.label(
        egui::RichText::new("Memory and storage")
            .size(13.0)
            .strong()
            .color(COLOR_TEXT_WHITE),
    );
    ui.add_space(6.0);
    ui.horizontal(|ui| {
        let dial_w = (ui.available_width() - 8.0) / 2.0;
        ui.allocate_ui(Vec2::new(dial_w, 120.0), |ui| {
            render_circular_dial(ui, "Memory", app.mem_pct, &format!("{:.0}GB", app.mem_total_gb), COLOR_CYAN);
        });
        ui.allocate_ui(Vec2::new(dial_w, 120.0), |ui| {
            render_circular_dial(ui, "Storage", app.disk_pct, &format!("{}GB", app.disk_total_gb), COLOR_CYAN);
        });
    });
}

// ============================================================================
// PANEL: PREFERENCE 8 TOGGLES (ROW 2, COL 2)
// ============================================================================
fn render_preference_panel(app: &mut AcerApp, ui: &mut egui::Ui) {
    ui.label(
        egui::RichText::new("Preference")
            .size(13.0)
            .strong()
            .color(COLOR_TEXT_WHITE),
    );
    ui.add_space(6.0);

    // 4x2 Grid of Quick Toggles to fit inside column width without horizontal overflow
    ui.vertical(|ui| {
        // Row 1 (4 toggles)
        ui.horizontal(|ui| {
            let _ = render_toggle_btn(ui, "Win key\nlock", &mut app.win_key_lock);
            let _ = render_toggle_btn(ui, "Swap\nWin/Fn", &mut app.swap_win_fn);
            let _ = render_toggle_btn(ui, "Fn key\nlock", &mut app.fn_key_lock);
            if render_toggle_btn(ui, "Touch\nPad", &mut app.touchpad_enabled) {
                app.controller.set_touchpad_enabled(app.touchpad_enabled);
            }
        });

        ui.add_space(4.0);

        // Row 2 (4 toggles)
        ui.horizontal(|ui| {
            let _ = render_toggle_btn(ui, "Camera", &mut app.camera_enabled);
            if render_toggle_btn(ui, "Airplane", &mut app.airplane_mode) {
                app.controller.set_airplane_mode(app.airplane_mode);
            }
            let _ = render_toggle_btn(ui, "Caps\nLock", &mut app.caps_lock);
            let _ = render_toggle_btn(ui, "Num\nLock", &mut app.num_lock);
        });
    });
}

fn render_toggle_btn(ui: &mut egui::Ui, label: &str, state: &mut bool) -> bool {
    let is_on = *state;
    let border = if is_on { COLOR_CYAN } else { Color32::from_rgb(38, 48, 64) };
    let bg = if is_on { Color32::from_rgb(14, 38, 54) } else { Color32::from_rgb(19, 24, 34) };
    let text_col = if is_on { Color32::WHITE } else { COLOR_TEXT_MUTED };

    let btn = ui.add(
        egui::Button::new(egui::RichText::new(label).size(8.5).color(text_col))
            .fill(bg)
            .stroke(Stroke::new(if is_on { 1.5 } else { 1.0 }, border))
            .corner_radius(4)
            .min_size(Vec2::new(62.0, 36.0)),
    );

    if btn.clicked() {
        *state = !*state;
        true
    } else {
        false
    }
}

// ============================================================================
// PANEL: CONFIGURATION (ROW 2, COL 3)
// ============================================================================
fn render_configuration_panel(app: &mut AcerApp, ui: &mut egui::Ui) {
    ui.label(
        egui::RichText::new("Configuration")
            .size(13.0)
            .strong()
            .color(COLOR_TEXT_WHITE),
    );
    ui.add_space(6.0);

    ui.horizontal(|ui| {
        if ui.checkbox(&mut app.blue_light_filter, "Blue light filter").changed() {
            app.controller.set_night_light(app.blue_light_filter, app.blue_light_level);
        }
    });

    ui.add_space(4.0);

    if app.blue_light_filter {
        ui.horizontal(|ui| {
            ui.label(egui::RichText::new("Warmth:").size(10.5).color(COLOR_TEXT_MUTED));
            if ui.add(egui::Slider::new(&mut app.blue_light_level, 0.0..=10.0).show_value(false)).changed() {
                app.controller.set_night_light(true, app.blue_light_level);
            }
        });
    }

    ui.add_space(6.0);

    // Preset mode badges
    ui.horizontal(|ui| {
        let presets = [
            (0, "Standard", COLOR_CYAN),
            (1, "Eye Care", COLOR_GOLD),
            (2, "Night", COLOR_TEMP_GREEN),
        ];

        for (idx, name, color) in presets {
            let is_sel = app.display_preset == idx;
            let stroke = if is_sel { Stroke::new(1.2, color) } else { Stroke::new(1.0, Color32::from_rgb(38, 48, 64)) };
            let bg = if is_sel { Color32::from_rgb(16, 32, 44) } else { Color32::from_rgb(18, 22, 30) };

            let btn = ui.add(
                egui::Button::new(egui::RichText::new(name).size(9.5).color(if is_sel { Color32::WHITE } else { COLOR_TEXT_MUTED }))
                    .fill(bg)
                    .stroke(stroke)
                    .corner_radius(4)
                    .min_size(Vec2::new(70.0, 22.0)),
            );

            if btn.clicked() {
                app.display_preset = idx;
                match idx {
                    0 => {
                        app.blue_light_filter = false;
                        app.controller.set_night_light(false, 0.0);
                    }
                    1 => {
                        app.blue_light_filter = true;
                        app.blue_light_level = 4.0;
                        app.controller.set_night_light(true, 4.0);
                    }
                    _ => {
                        app.blue_light_filter = true;
                        app.blue_light_level = 8.0;
                        app.controller.set_night_light(true, 8.0);
                    }
                }
            }
        }
    });
}

// ============================================================================
// LIGHTING VIEW (TAB 1: 💡)
// ============================================================================
fn render_lighting_tab(app: &mut AcerApp, ui: &mut egui::Ui) {
    let avail_w = ui.available_width();
    let card_h = 490.0;
    render_card_frame(ui, avail_w, card_h, |ui| {
        ui.horizontal(|ui| {
            ui.label(egui::RichText::new("Chassis & Keyboard Lighting").size(15.0).strong().color(COLOR_TEXT_WHITE));
            ui.with_layout(egui::Layout::right_to_left(egui::Align::Center), |ui| {
                if ui.button(egui::RichText::new("<< Dashboard").size(12.0).color(COLOR_CYAN)).clicked() {
                    app.active_tab = 0;
                }
            });
        });

        ui.add_space(8.0);
        ui.separator();
        ui.add_space(10.0);

        // Keyboard visual frame
        let (kb_rect, _) = ui.allocate_exact_size(Vec2::new(ui.available_width(), 130.0), egui::Sense::hover());
        let painter = ui.painter_at(kb_rect);
        let glow_col = Color32::from_rgb(app.kbd_color[0], app.kbd_color[1], app.kbd_color[2]);

        painter.rect_filled(kb_rect, 6.0, Color32::from_rgb(12, 15, 22));
        painter.rect_stroke(kb_rect, 6.0, Stroke::new(2.0, glow_col), StrokeKind::Inside);

        let key_w = (kb_rect.width() - 40.0) / 14.0;
        let key_h = 16.0;
        for row in 0..4 {
            let y = kb_rect.min.y + 16.0 + (row as f32 * (key_h + 4.0));
            for col in 0..14 {
                let x = kb_rect.min.x + 20.0 + (col as f32 * (key_w + 2.0));
                let key_rect = Rect::from_min_size(Pos2::new(x, y), Vec2::new(key_w - 2.0, key_h));
                painter.rect_filled(key_rect, 2.0, Color32::from_rgb(22, 28, 38));
                painter.rect_stroke(key_rect, 2.0, Stroke::new(0.8, Color32::from_rgba_premultiplied(app.kbd_color[0], app.kbd_color[1], app.kbd_color[2], 120)), StrokeKind::Inside);
            }
        }

        painter.text(
            Pos2::new(kb_rect.center().x, kb_rect.max.y - 14.0),
            egui::Align2::CENTER_CENTER,
            format!("RGB({}, {}, {}) · Brightness: {}%", app.kbd_color[0], app.kbd_color[1], app.kbd_color[2], app.kbd_brightness),
            FontId::proportional(11.0),
            glow_col,
        );

        ui.add_space(14.0);

        // Palette presets
        ui.label(egui::RichText::new("Color Presets").size(12.5).strong().color(COLOR_TEXT_WHITE));
        ui.add_space(6.0);

        let swatches = [
            ("Acer Cyan", [0, 225, 255]),
            ("Ocean Blue", [41, 121, 255]),
            ("Vivid Purple", [156, 39, 176]),
            ("Crimson Red", [244, 67, 54]),
            ("Amber Gold", [255, 179, 0]),
            ("Neon Green", [0, 230, 118]),
            ("Crisp White", [255, 255, 255]),
            ("Off", [0, 0, 0]),
        ];

        ui.horizontal(|ui| {
            for (name, rgb) in swatches {
                let col = Color32::from_rgb(rgb[0], rgb[1], rgb[2]);
                let is_active = app.kbd_color == rgb;

                let (rect, resp) = ui.allocate_exact_size(Vec2::new(48.0, 30.0), egui::Sense::click());
                let p = ui.painter_at(rect);

                p.rect_filled(rect, 4.0, col);
                let border_stroke = if is_active {
                    Stroke::new(2.0, Color32::WHITE)
                } else if resp.hovered() {
                    Stroke::new(1.5, COLOR_CYAN)
                } else {
                    Stroke::new(1.0, Color32::from_rgb(38, 48, 64))
                };
                p.rect_stroke(rect, 4.0, border_stroke, StrokeKind::Inside);

                if resp.clicked() {
                    app.set_keyboard_color(rgb[0], rgb[1], rgb[2]);
                }
                let _ = resp.on_hover_text(name);
                ui.add_space(4.0);
            }
        });

        ui.add_space(14.0);

        ui.horizontal(|ui| {
            ui.label(egui::RichText::new("Backlight Brightness:").size(12.0).color(COLOR_TEXT_WHITE));
            let mut bri = app.kbd_brightness;
            if ui.add(egui::Slider::new(&mut bri, 0..=100).suffix("%").show_value(true)).changed() {
                app.set_keyboard_brightness(bri);
            }
        });

        if let Some(ref msg) = app.curve_feedback {
            ui.add_space(8.0);
            ui.label(egui::RichText::new(msg).size(11.0).color(COLOR_TEMP_GREEN));
        }
    });
}

// ============================================================================
// BATTERY VIEW (TAB 2: 🔋)
// ============================================================================
fn render_battery_tab(app: &mut AcerApp, ui: &mut egui::Ui) {
    let avail_w = ui.available_width();
    render_card_frame(ui, avail_w, 490.0, |ui| {
        ui.horizontal(|ui| {
            ui.label(egui::RichText::new("Battery & Power Health").size(15.0).strong().color(COLOR_TEXT_WHITE));
            ui.with_layout(egui::Layout::right_to_left(egui::Align::Center), |ui| {
                if ui.button(egui::RichText::new("<< Dashboard").size(12.0).color(COLOR_CYAN)).clicked() {
                    app.active_tab = 0;
                }
            });
        });

        ui.add_space(8.0);
        ui.separator();
        ui.add_space(12.0);

        ui.horizontal(|ui| {
            render_circular_dial(ui, "Battery", 82, "BAT0", COLOR_TEMP_GREEN);
            ui.add_space(20.0);

            ui.vertical(|ui| {
                ui.label(egui::RichText::new("Health & Longevity Protection").size(13.5).strong().color(COLOR_TEXT_WHITE));
                ui.add_space(4.0);
                ui.label(egui::RichText::new("Custom charge limit stops charging at 80% to preserve lithium pack lifespan.").size(11.5).color(COLOR_TEXT_MUTED));
                ui.add_space(6.0);

                ui.horizontal(|ui| {
                    let _ = ui.button(egui::RichText::new("✓ 80% Health Preservation").size(11.5).color(COLOR_CYAN));
                    let _ = ui.button(egui::RichText::new("100% Full Capacity").size(11.5).color(COLOR_TEXT_MUTED));
                });
            });
        });
    });
}

// ============================================================================
// SETTINGS VIEW (TAB 3: ⚙)
// ============================================================================
fn render_settings_tab(app: &mut AcerApp, ui: &mut egui::Ui) {
    let avail_w = ui.available_width();
    render_card_frame(ui, avail_w, 490.0, |ui| {
        ui.horizontal(|ui| {
            ui.label(egui::RichText::new("Device Settings & Diagnostics").size(15.0).strong().color(COLOR_TEXT_WHITE));
            ui.with_layout(egui::Layout::right_to_left(egui::Align::Center), |ui| {
                if ui.button(egui::RichText::new("<< Dashboard").size(12.0).color(COLOR_CYAN)).clicked() {
                    app.active_tab = 0;
                }
            });
        });

        ui.add_space(8.0);
        ui.separator();
        ui.add_space(12.0);

        ui.label(egui::RichText::new("Active Linux Subsystems:").size(13.0).strong().color(COLOR_TEXT_WHITE));
        ui.add_space(4.0);

        let checks = [
            ("Cooling Subsystem", "com.tux.Tailor (tailord.service) · EC PWM Active"),
            ("Keyboard Driver", "tuxedo_keyboard (rgb:kbd_backlight sysfs) · Active"),
            ("Power Manager", "net.hadess.PowerProfiles (powerprofilesctl) · Active"),
            ("Motherboard ODM", "Clevo CLV0001:00 WMI · Persistent drivers loaded"),
        ];

        for (title, desc) in checks {
            ui.horizontal(|ui| {
                ui.label(egui::RichText::new("●").size(10.0).color(COLOR_TEMP_GREEN));
                ui.label(egui::RichText::new(format!("{}:", title)).size(11.5).strong().color(COLOR_TEXT_WHITE));
                ui.label(egui::RichText::new(desc).size(11.5).color(COLOR_TEXT_MUTED));
            });
            ui.add_space(2.0);
        }
    });
}

// ============================================================================
// REUSABLE WIDGET HELPERS
// ============================================================================

/// Renders authentic diagonally segmented/striped usage bars matching OEM UI
pub fn render_striped_bar(ui: &mut egui::Ui, pct: u32, fill_col: Color32) {
    let (rect, _) = ui.allocate_exact_size(Vec2::new(ui.available_width(), 10.0), egui::Sense::hover());
    let painter = ui.painter_at(rect);

    painter.rect_filled(rect, 2.0, Color32::from_rgb(20, 25, 35));
    painter.rect_stroke(rect, 2.0, Stroke::new(1.0, Color32::from_rgb(32, 40, 54)), StrokeKind::Inside);

    let num_segments = 26;
    let filled_segments = (num_segments as f32 * (pct as f32 / 100.0)).round() as usize;
    let gap = 2.0;
    let seg_w = (rect.width() - (num_segments as f32 - 1.0) * gap) / num_segments as f32;

    for i in 0..num_segments {
        let x = rect.min.x + (i as f32) * (seg_w + gap);
        let seg_rect = Rect::from_min_size(Pos2::new(x, rect.min.y + 1.0), Vec2::new(seg_w, rect.height() - 2.0));
        let col = if i < filled_segments { fill_col } else { Color32::from_rgb(30, 38, 50) };
        painter.rect_filled(seg_rect, 1.0, col);
    }
}

/// Renders circular dial gauge (Memory / Storage) with 270-degree arc, ticks, and cyan needle/arc
pub fn render_circular_dial(ui: &mut egui::Ui, label: &str, pct: u32, subtext: &str, accent: Color32) {
    let (rect, _) = ui.allocate_exact_size(Vec2::new(ui.available_width(), 80.0), egui::Sense::hover());
    let painter = ui.painter_at(rect);
    let center = Pos2::new(rect.center().x, rect.center().y - 4.0);
    let radius = 28.0;

    let start_angle = PI * 0.75;
    let end_angle = PI * 2.25;

    for i in 0..20 {
        let a1 = start_angle + (end_angle - start_angle) * (i as f32 / 20.0);
        let a2 = start_angle + (end_angle - start_angle) * ((i + 1) as f32 / 20.0);
        let p1 = Pos2::new(center.x + radius * a1.cos(), center.y + radius * a1.sin());
        let p2 = Pos2::new(center.x + radius * a2.cos(), center.y + radius * a2.sin());
        painter.line_segment([p1, p2], Stroke::new(2.8, Color32::from_rgb(28, 35, 48)));
    }

    let active_segments = (20.0 * (pct as f32 / 100.0)).round() as usize;
    for i in 0..active_segments {
        let a1 = start_angle + (end_angle - start_angle) * (i as f32 / 20.0);
        let a2 = start_angle + (end_angle - start_angle) * ((i + 1) as f32 / 20.0);
        let p1 = Pos2::new(center.x + radius * a1.cos(), center.y + radius * a1.sin());
        let p2 = Pos2::new(center.x + radius * a2.cos(), center.y + radius * a2.sin());
        painter.line_segment([p1, p2], Stroke::new(3.0, accent));
    }

    painter.text(
        Pos2::new(center.x, center.y - 4.0),
        egui::Align2::CENTER_CENTER,
        format!("{}%", pct),
        FontId::proportional(11.5),
        COLOR_TEXT_WHITE,
    );

    painter.text(
        Pos2::new(center.x, center.y + 8.0),
        egui::Align2::CENTER_CENTER,
        subtext,
        FontId::proportional(8.5),
        COLOR_TEXT_MUTED,
    );

    painter.text(
        Pos2::new(center.x, rect.max.y - 4.0),
        egui::Align2::CENTER_CENTER,
        label,
        FontId::proportional(9.5),
        COLOR_TEXT_MUTED,
    );
}

// ============================================================================
// VECTOR PAINTER ICONS (NO UNICODE/FONT DEPENDENCY)
// ============================================================================
fn draw_icon_dashboard(p: &egui::Painter, r: Rect, col: Color32) {
    let c = r.center();
    let rad = 7.0;
    p.circle_stroke(c, rad, Stroke::new(1.3, col));
    let needle_end = Pos2::new(c.x + rad * 0.65 * 0.707, c.y - rad * 0.65 * 0.707);
    p.line_segment([c, needle_end], Stroke::new(1.4, col));
}

fn draw_icon_lighting(p: &egui::Painter, r: Rect, col: Color32) {
    let c = r.center();
    p.circle_stroke(Pos2::new(c.x, c.y - 2.0), 5.5, Stroke::new(1.3, col));
    p.line_segment([Pos2::new(c.x - 3.0, c.y + 5.0), Pos2::new(c.x + 3.0, c.y + 5.0)], Stroke::new(1.3, col));
    p.line_segment([Pos2::new(c.x - 2.0, c.y + 7.5), Pos2::new(c.x + 2.0, c.y + 7.5)], Stroke::new(1.3, col));
}

fn draw_icon_battery(p: &egui::Painter, r: Rect, col: Color32) {
    let c = r.center();
    let body = Rect::from_center_size(Pos2::new(c.x - 1.0, c.y), Vec2::new(14.0, 9.0));
    p.rect_stroke(body, 1.5, Stroke::new(1.2, col), StrokeKind::Inside);
    let term = Rect::from_center_size(Pos2::new(c.x + 7.0, c.y), Vec2::new(2.0, 4.0));
    p.rect_filled(term, 0.5, col);
    let fill = Rect::from_min_size(Pos2::new(body.min.x + 2.0, body.min.y + 2.0), Vec2::new(7.0, 5.0));
    p.rect_filled(fill, 1.0, col);
}

fn draw_icon_settings(p: &egui::Painter, r: Rect, col: Color32) {
    let c = r.center();
    p.circle_stroke(c, 5.5, Stroke::new(1.3, col));
    p.circle_filled(c, 2.0, col);
    for i in 0..4 {
        let a = i as f32 * (PI / 2.0);
        let p1 = Pos2::new(c.x + 5.0 * a.cos(), c.y + 5.0 * a.sin());
        let p2 = Pos2::new(c.x + 7.5 * a.cos(), c.y + 7.5 * a.sin());
        p.line_segment([p1, p2], Stroke::new(1.5, col));
    }
}

fn draw_icon_minimize(p: &egui::Painter, r: Rect, col: Color32) {
    let c = r.center();
    p.line_segment([Pos2::new(c.x - 5.0, c.y), Pos2::new(c.x + 5.0, c.y)], Stroke::new(1.4, col));
}

fn draw_icon_maximize(p: &egui::Painter, r: Rect, col: Color32) {
    let c = r.center();
    let rect = Rect::from_center_size(c, Vec2::new(9.0, 9.0));
    p.rect_stroke(rect, 1.0, Stroke::new(1.3, col), StrokeKind::Inside);
}

fn draw_icon_close(p: &egui::Painter, r: Rect, col: Color32) {
    let c = r.center();
    let d = 4.0;
    p.line_segment([Pos2::new(c.x - d, c.y - d), Pos2::new(c.x + d, c.y + d)], Stroke::new(1.4, col));
    p.line_segment([Pos2::new(c.x + d, c.y - d), Pos2::new(c.x - d, c.y + d)], Stroke::new(1.4, col));
}

// ============================================================================
// UI UNIT TESTS
// ============================================================================
#[cfg(test)]
mod tests {
    use super::*;
    use crate::backend::GpuSwitchMode;

    #[test]
    fn test_color_tokens() {
        assert_eq!(COLOR_CYAN, Color32::from_rgb(0, 225, 255));
        assert_eq!(COLOR_GOLD, Color32::from_rgb(255, 179, 0));
        assert_eq!(COLOR_PURPLE_GLOW, Color32::from_rgb(156, 39, 176));
    }

    #[test]
    fn test_app_ui_state_defaults() {
        let app = AcerApp::default();
        assert_eq!(app.active_tab, 0);
        assert!(matches!(
            app.power_mode,
            PowerMode::Performance | PowerMode::Entertainment | PowerMode::Quiet
        ));
        assert_eq!(app.fan_mode, FanMode::Automatic);
        assert_eq!(app.gpu_switch, GpuSwitchMode::Discrete);
        assert!(app.cpu_curve_nodes.len() >= 2);
        assert!(app.gpu_curve_nodes.len() >= 2);
        assert!(app.touchpad_enabled);
        assert!(app.camera_enabled);
        assert!(app.num_lock);
    }

    #[test]
    fn test_mode_transitions() {
        let mut app = AcerApp::default();

        app.set_power_profile(PowerMode::Quiet);
        assert_eq!(app.power_mode, PowerMode::Quiet);
        assert_eq!(app.fan_mode, FanMode::Automatic);

        app.set_fan_mode(FanMode::Turbo);
        assert_eq!(app.fan_mode, FanMode::Turbo);

        app.set_fan_mode(FanMode::Custom);
        assert_eq!(app.fan_mode, FanMode::Custom);

        app.set_gpu_switch(GpuSwitchMode::Integrated);
        assert_eq!(app.gpu_switch, GpuSwitchMode::Integrated);
    }

    #[test]
    fn test_fan_curve_node_bounds() {
        let mut nodes = vec![
            FanCurveNode { temp: 20.0, speed: 20.0 },
            FanCurveNode { temp: 50.0, speed: 50.0 },
            FanCurveNode { temp: 80.0, speed: 80.0 },
        ];

        let idx = 1;
        let candidate_t = 60.0f32;
        let min_t = nodes[idx - 1].temp + 1.0;
        let max_t = nodes[idx + 1].temp - 1.0;
        nodes[idx].temp = candidate_t.clamp(min_t, max_t);
        assert_eq!(nodes[idx].temp, 60.0);

        let out_of_bounds_t = 90.0f32;
        nodes[idx].temp = out_of_bounds_t.clamp(min_t, max_t);
        assert_eq!(nodes[idx].temp, 79.0);
    }

    #[test]
    fn test_tachometer_telemetry_toggle() {
        let mut app = AcerApp::default();
        assert_eq!(app.active_curve_tab, 0);

        app.active_curve_tab = 1;
        assert_eq!(app.active_curve_tab, 1);

        app.active_curve_tab = 0;
        assert_eq!(app.active_curve_tab, 0);
    }

    #[test]
    fn test_striped_bar_bounds() {
        let num_segments = 26;
        let pct = 50;
        let filled = (num_segments as f32 * (pct as f32 / 100.0)).round() as usize;
        assert_eq!(filled, 13);

        let pct_zero = 0;
        let filled_zero = (num_segments as f32 * (pct_zero as f32 / 100.0)).round() as usize;
        assert_eq!(filled_zero, 0);

        let pct_max = 100;
        let filled_max = (num_segments as f32 * (pct_max as f32 / 100.0)).round() as usize;
        assert_eq!(filled_max, 26);
    }

    #[test]
    fn test_keyboard_color_and_tabs() {
        let mut app = AcerApp::default();
        assert_eq!(app.active_tab, 0);

        app.active_tab = 1;
        assert_eq!(app.active_tab, 1);

        app.set_keyboard_color(0, 225, 255);
        assert_eq!(app.kbd_color, [0, 225, 255]);
        assert!(app.curve_feedback.is_some());

        app.set_keyboard_brightness(80);
        assert_eq!(app.kbd_brightness, 80);
    }
}
