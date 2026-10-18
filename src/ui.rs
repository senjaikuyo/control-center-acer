use crate::backend::{FanCurveNode, FanMode, GpuSwitchMode, PowerMode};
use crate::AcerApp;
use eframe::egui::{self, Color32, FontId, Pos2, Rect, Stroke, StrokeKind, Vec2};
use std::f32::consts::PI;

// ============================================================================
// COLOR PALETTE & DESIGN TOKENS (PIXEL-PERFECT OEM REPLICATION)
// ============================================================================
pub const COLOR_BG: Color32 = Color32::from_rgb(6, 8, 13);
pub const COLOR_CARD_BG: Color32 = Color32::from_rgb(14, 18, 25);
pub const COLOR_CARD_BORDER: Color32 = Color32::from_rgb(28, 35, 48);

pub const COLOR_CYAN: Color32 = Color32::from_rgb(0, 225, 255);
pub const COLOR_CYAN_DIM: Color32 = Color32::from_rgb(12, 36, 50);
pub const COLOR_CYAN_BORDER: Color32 = Color32::from_rgb(0, 200, 235);

pub const COLOR_GOLD: Color32 = Color32::from_rgb(255, 179, 0);
pub const COLOR_GOLD_DIM: Color32 = Color32::from_rgb(46, 34, 10);

pub const COLOR_PURPLE_GLOW: Color32 = Color32::from_rgb(156, 39, 176);

pub const COLOR_TEXT_WHITE: Color32 = Color32::from_rgb(255, 255, 255);
pub const COLOR_TEXT_MUTED: Color32 = Color32::from_rgb(126, 137, 154);
pub const COLOR_TEXT_DARK: Color32 = Color32::from_rgb(75, 85, 100);

pub const COLOR_TEMP_GREEN: Color32 = Color32::from_rgb(0, 230, 118);
pub const COLOR_TEMP_YELLOW: Color32 = Color32::from_rgb(255, 214, 0);
pub const COLOR_TEMP_RED: Color32 = Color32::from_rgb(255, 61, 0);

// ============================================================================
// MAIN FRONTEND ENTRY POINT
// ============================================================================
pub fn render(app: &mut AcerApp, ctx: &egui::Context, ui: &mut egui::Ui) {
    // Fill the window background canvas
    let full_rect = ui.max_rect();
    ui.painter().rect_filled(full_rect, 0.0, COLOR_BG);

    ui.spacing_mut().item_spacing = Vec2::new(10.0, 10.0);

    // 1. TOP WINDOW BAR
    render_top_bar(app, ctx, ui);

    // 2. POWER PROFILES (TOP-CENTER TABS)
    render_power_profiles(app, ui);

    // 3. MAIN MID-DECK (3 COLUMNS)
    render_mid_deck(app, ui);

    // 4. BOTTOM DECK (3 MODULAR FRAMED PANELS)
    render_bottom_deck(app, ui);

    // 5. BOTTOM CAROUSEL / SCROLLBAR TRACK
    render_bottom_scrollbar(app, ui);
}

// ============================================================================
// 1. TOP WINDOW BAR
// ============================================================================
fn render_top_bar(app: &mut AcerApp, ctx: &egui::Context, ui: &mut egui::Ui) {
    ui.horizontal(|ui| {
        ui.add_space(8.0);

        // 4 Category Tabs (⏱ 💡 🔋 ⚙)
        let tabs = [
            ("⏱", "Dashboard / Telemetry"),
            ("💡", "Chassis & Keyboard Lighting"),
            ("🔋", "Battery & Power Management"),
            ("⚙", "Device Settings"),
        ];

        for (idx, (icon, hover_text)) in tabs.iter().enumerate() {
            let is_active = app.active_tab == idx;

            let (rect, resp) = ui.allocate_exact_size(Vec2::new(32.0, 30.0), egui::Sense::click());
            let painter = ui.painter_at(rect);

            let bg = if is_active {
                COLOR_CYAN_DIM
            } else if resp.hovered() {
                Color32::from_rgb(22, 28, 38)
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

            painter.text(
                rect.center(),
                egui::Align2::CENTER_CENTER,
                *icon,
                FontId::proportional(15.0),
                icon_col,
            );

            if resp.clicked() {
                app.active_tab = idx;
            }
            let _ = resp.on_hover_text(*hover_text);

            ui.add_space(4.0);
        }

        // Centered Window Title: "Control Center"
        let avail_w = ui.available_width() - 140.0;
        let title_offset = (avail_w - 140.0) / 2.0;
        if title_offset > 0.0 {
            ui.add_space(title_offset);
        }

        ui.label(
            egui::RichText::new("Control Center")
                .size(16.5)
                .strong()
                .color(COLOR_TEXT_WHITE),
        );

        // Right Window Controls: User Profile, Minimize, Maximize, Close
        ui.with_layout(egui::Layout::right_to_left(egui::Align::Center), |ui| {
            ui.add_space(8.0);

            // Close Button
            let (close_rect, close_resp) =
                ui.allocate_exact_size(Vec2::new(28.0, 24.0), egui::Sense::click());
            let cp = ui.painter_at(close_rect);
            if close_resp.hovered() {
                cp.rect_filled(close_rect, 3.0, Color32::from_rgb(220, 38, 38));
            }
            cp.text(
                close_rect.center(),
                egui::Align2::CENTER_CENTER,
                "✕",
                FontId::proportional(13.0),
                if close_resp.hovered() {
                    Color32::WHITE
                } else {
                    COLOR_TEXT_MUTED
                },
            );
            if close_resp.clicked() {
                ctx.send_viewport_cmd(egui::ViewportCommand::Close);
            }

            ui.add_space(4.0);

            // Maximize / Restore Button
            let (max_rect, max_resp) =
                ui.allocate_exact_size(Vec2::new(28.0, 24.0), egui::Sense::click());
            let mp = ui.painter_at(max_rect);
            if max_resp.hovered() {
                mp.rect_filled(max_rect, 3.0, Color32::from_rgb(28, 35, 48));
            }
            mp.text(
                max_rect.center(),
                egui::Align2::CENTER_CENTER,
                "▢",
                FontId::proportional(13.0),
                if max_resp.hovered() {
                    Color32::WHITE
                } else {
                    COLOR_TEXT_MUTED
                },
            );
            if max_resp.clicked() {
                ctx.send_viewport_cmd(egui::ViewportCommand::Maximized(true));
            }

            ui.add_space(4.0);

            // Minimize Button
            let (min_rect, min_resp) =
                ui.allocate_exact_size(Vec2::new(28.0, 24.0), egui::Sense::click());
            let min_p = ui.painter_at(min_rect);
            if min_resp.hovered() {
                min_p.rect_filled(min_rect, 3.0, Color32::from_rgb(28, 35, 48));
            }
            min_p.text(
                min_rect.center(),
                egui::Align2::CENTER_CENTER,
                "—",
                FontId::proportional(13.0),
                if min_resp.hovered() {
                    Color32::WHITE
                } else {
                    COLOR_TEXT_MUTED
                },
            );
            if min_resp.clicked() {
                ctx.send_viewport_cmd(egui::ViewportCommand::Minimized(true));
            }

            ui.add_space(10.0);

            // Profile / Guide Button
            let (user_rect, user_resp) =
                ui.allocate_exact_size(Vec2::new(24.0, 24.0), egui::Sense::click());
            let up = ui.painter_at(user_rect);
            up.text(
                user_rect.center(),
                egui::Align2::CENTER_CENTER,
                "👤",
                FontId::proportional(13.0),
                if user_resp.hovered() {
                    COLOR_CYAN
                } else {
                    COLOR_TEXT_MUTED
                },
            );
        });
    });
}

// ============================================================================
// 2. POWER PROFILES (TOP-CENTER TABS)
// ============================================================================
fn render_power_profiles(app: &mut AcerApp, ui: &mut egui::Ui) {
    ui.vertical_centered(|ui| {
        ui.horizontal(|ui| {
            let total_w = ui.available_width();
            let bar_w = 340.0;
            if total_w > bar_w {
                ui.add_space((total_w - bar_w) / 2.0);
            }

            let modes = [
                (PowerMode::Performance, "Performance"),
                (PowerMode::Entertainment, "Entertainment"),
                (PowerMode::Quiet, "Quiet"),
            ];

            for (mode, label) in modes {
                let is_active = app.power_mode == mode;
                let text_col = if is_active {
                    COLOR_GOLD
                } else {
                    COLOR_TEXT_MUTED
                };

                let (rect, resp) = ui.allocate_exact_size(Vec2::new(104.0, 24.0), egui::Sense::click());
                let painter = ui.painter_at(rect);

                if resp.hovered() && !is_active {
                    painter.rect_filled(rect, 3.0, Color32::from_rgb(18, 22, 32));
                }

                // Profile Label
                painter.text(
                    Pos2::new(rect.center().x, rect.center().y - 2.0),
                    egui::Align2::CENTER_CENTER,
                    label,
                    FontId::proportional(14.0),
                    text_col,
                );

                // Active Gold Underline Indicator
                if is_active {
                    let underline_rect = Rect::from_min_size(
                        Pos2::new(rect.min.x + 12.0, rect.max.y - 2.5),
                        Vec2::new(rect.width() - 24.0, 2.5),
                    );
                    painter.rect_filled(underline_rect, 1.2, COLOR_GOLD);
                }

                if resp.clicked() {
                    app.set_power_profile(mode);
                }

                ui.add_space(8.0);
            }
        });

        // Down-chevron indicator below power profile bar
        ui.label(
            egui::RichText::new("︾")
                .size(10.0)
                .color(COLOR_TEXT_DARK),
        );
    });
}

// ============================================================================
// 3. MAIN MID-DECK (3 COLUMNS)
// ============================================================================
fn render_mid_deck(app: &mut AcerApp, ui: &mut egui::Ui) {
    ui.columns(3, |cols| {
        // --------------------------------------------------------------------
        // COLUMN A: SYSTEM TELEMETRY
        // --------------------------------------------------------------------
        let col_a = &mut cols[0];
        egui::Frame::new()
            .fill(COLOR_CARD_BG)
            .stroke(Stroke::new(1.0, COLOR_CARD_BORDER))
            .corner_radius(8)
            .inner_margin(14)
            .show(col_a, |ui| {
                ui.label(
                    egui::RichText::new("System")
                        .size(15.0)
                        .strong()
                        .color(COLOR_TEXT_WHITE),
                );

                ui.add_space(2.0);
                ui.label(
                    egui::RichText::new(&app.cpu_name)
                        .size(11.0)
                        .color(COLOR_TEXT_MUTED),
                );
                ui.label(
                    egui::RichText::new(&app.gpu_name)
                        .size(11.0)
                        .color(COLOR_TEXT_MUTED),
                );

                ui.add_space(10.0);

                // CPU Status (Clock & Temp)
                ui.horizontal(|ui| {
                    let half_w = (ui.available_width() - 10.0) / 2.0;
                    ui.allocate_ui(Vec2::new(half_w, 42.0), |ui| {
                        ui.label(
                            egui::RichText::new(format!("{} MHz", app.cpu_clock_mhz))
                                .size(21.0)
                                .strong()
                                .color(COLOR_TEXT_WHITE),
                        );
                        ui.label(
                            egui::RichText::new("CPU clock speed")
                                .size(10.0)
                                .color(COLOR_CYAN),
                        );
                    });

                    ui.allocate_ui(Vec2::new(half_w, 42.0), |ui| {
                        ui.label(
                            egui::RichText::new(format!("{} ℃", app.cpu_temp))
                                .size(21.0)
                                .strong()
                                .color(COLOR_TEXT_WHITE),
                        );
                        ui.label(
                            egui::RichText::new("Temperature")
                                .size(10.0)
                                .color(COLOR_CYAN),
                        );
                    });
                });

                ui.add_space(8.0);

                // GPU Status (Clock & Temp)
                ui.horizontal(|ui| {
                    let half_w = (ui.available_width() - 10.0) / 2.0;
                    ui.allocate_ui(Vec2::new(half_w, 42.0), |ui| {
                        let clk_str = if app.gpu_clock_mhz > 0 {
                            format!("{} MHz", app.gpu_clock_mhz)
                        } else {
                            "OFF MHz".to_string()
                        };
                        ui.label(
                            egui::RichText::new(clk_str)
                                .size(21.0)
                                .strong()
                                .color(COLOR_TEXT_WHITE),
                        );
                        ui.label(
                            egui::RichText::new("GPU clock speed")
                                .size(10.0)
                                .color(COLOR_CYAN),
                        );
                    });

                    ui.allocate_ui(Vec2::new(half_w, 42.0), |ui| {
                        let temp_str = if app.gpu_temp > 0 {
                            format!("{} ℃", app.gpu_temp)
                        } else {
                            "OFF ℃".to_string()
                        };
                        ui.label(
                            egui::RichText::new(temp_str)
                                .size(21.0)
                                .strong()
                                .color(COLOR_TEXT_WHITE),
                        );
                        ui.label(
                            egui::RichText::new("Temperature")
                                .size(10.0)
                                .color(COLOR_CYAN),
                        );
                    });
                });

                ui.add_space(14.0);

                // Usage Progress Bars (< CPU usage and < GPU usage)
                ui.horizontal(|ui| {
                    ui.label(
                        egui::RichText::new("< CPU usage")
                            .size(11.0)
                            .color(COLOR_TEXT_MUTED),
                    );
                    ui.with_layout(egui::Layout::right_to_left(egui::Align::Center), |ui| {
                        ui.label(
                            egui::RichText::new(format!("{} %", app.cpu_usage_pct))
                                .size(11.0)
                                .color(COLOR_TEXT_WHITE),
                        );
                    });
                });
                render_striped_bar(ui, app.cpu_usage_pct, COLOR_CYAN);

                ui.add_space(8.0);

                ui.horizontal(|ui| {
                    ui.label(
                        egui::RichText::new("< GPU usage")
                            .size(11.0)
                            .color(COLOR_TEXT_MUTED),
                    );
                    ui.with_layout(egui::Layout::right_to_left(egui::Align::Center), |ui| {
                        ui.label(
                            egui::RichText::new(format!("{} %", app.gpu_usage_pct))
                                .size(11.0)
                                .color(COLOR_TEXT_WHITE),
                        );
                    });
                });
                render_striped_bar(ui, app.gpu_usage_pct, COLOR_CYAN);
            });

        // --------------------------------------------------------------------
        // COLUMN B: CENTER COLUMN (SWITCHABLE)
        // --------------------------------------------------------------------
        let col_b = &mut cols[1];
        egui::Frame::new()
            .fill(COLOR_CARD_BG)
            .stroke(Stroke::new(1.0, COLOR_CARD_BORDER))
            .corner_radius(8)
            .inner_margin(12)
            .show(col_b, |ui| {
                if app.fan_mode == FanMode::Custom {
                    render_custom_fan_setting_view(app, ui);
                } else {
                    render_speedometer_tachometer_view(app, ui);
                }
            });

        // --------------------------------------------------------------------
        // COLUMN C: FAN SPEED CONTROL
        // --------------------------------------------------------------------
        let col_c = &mut cols[2];
        egui::Frame::new()
            .fill(COLOR_CARD_BG)
            .stroke(Stroke::new(1.0, COLOR_CARD_BORDER))
            .corner_radius(8)
            .inner_margin(14)
            .show(col_c, |ui| {
                ui.horizontal(|ui| {
                    ui.label(
                        egui::RichText::new("🌀 FAN speed control")
                            .size(15.0)
                            .strong()
                            .color(COLOR_TEXT_WHITE),
                    );
                });

                ui.add_space(6.0);

                // Radio buttons (Automatic, Maximum, Custom, Turbo)
                let modes = [
                    (FanMode::Automatic, "Automatic"),
                    (FanMode::Maximum, "Maximum"),
                    (FanMode::Custom, "Custom"),
                    (FanMode::Turbo, "Turbo"),
                ];

                for (m, lbl) in modes {
                    let is_active = app.fan_mode == m;
                    ui.horizontal(|ui| {
                        let (radio_rect, radio_resp) =
                            ui.allocate_exact_size(Vec2::new(16.0, 16.0), egui::Sense::click());
                        let rp = ui.painter_at(radio_rect);
                        let center = radio_rect.center();

                        let stroke_col = if is_active {
                            COLOR_GOLD
                        } else {
                            Color32::from_rgb(45, 54, 70)
                        };
                        rp.circle_stroke(center, 7.0, Stroke::new(1.5, stroke_col));

                        if is_active {
                            rp.circle_filled(center, 4.0, COLOR_GOLD);
                        }

                        let text_col = if is_active {
                            COLOR_GOLD
                        } else {
                            COLOR_TEXT_MUTED
                        };

                        let text_resp = ui.selectable_label(
                            false,
                            egui::RichText::new(lbl).size(13.0).color(text_col),
                        );

                        if radio_resp.clicked() || text_resp.clicked() {
                            app.set_fan_mode(m);
                        }

                        if m == FanMode::Custom {
                            let arrow_lbl = if is_active { "<<" } else { ">>" };
                            if ui.small_button(egui::RichText::new(arrow_lbl).color(COLOR_GOLD)).clicked() {
                                app.set_fan_mode(FanMode::Custom);
                            }
                        }
                    });
                    ui.add_space(2.0);
                }

                ui.add_space(8.0);
                ui.separator();
                ui.add_space(6.0);

                // Dual RPM Meters
                ui.horizontal(|ui| {
                    let half_w = (ui.available_width() - 14.0) / 2.0;

                    ui.allocate_ui(Vec2::new(half_w, 42.0), |ui| {
                        ui.label(
                            egui::RichText::new(format!("{} RPM", app.cpu_fan_rpm))
                                .size(20.0)
                                .strong()
                                .color(COLOR_TEXT_WHITE),
                        );
                        ui.label(
                            egui::RichText::new("CPU fan speed")
                                .size(10.0)
                                .color(COLOR_CYAN),
                        );
                    });

                    ui.separator();

                    ui.allocate_ui(Vec2::new(half_w, 42.0), |ui| {
                        ui.label(
                            egui::RichText::new(format!("{} RPM", app.gpu_fan_rpm))
                                .size(20.0)
                                .strong()
                                .color(COLOR_TEXT_WHITE),
                        );
                        ui.label(
                            egui::RichText::new("GPU fan speed")
                                .size(10.0)
                                .color(COLOR_CYAN),
                        );
                    });
                });

                ui.add_space(14.0);

                // Percentage Duty Bars
                ui.horizontal(|ui| {
                    ui.label(
                        egui::RichText::new("CPU FAN")
                            .size(11.0)
                            .color(COLOR_TEXT_MUTED),
                    );
                    ui.with_layout(egui::Layout::right_to_left(egui::Align::Center), |ui| {
                        ui.label(
                            egui::RichText::new(format!("{} %", app.cpu_fan_pct))
                                .size(11.0)
                                .color(COLOR_TEXT_WHITE),
                        );
                    });
                });
                render_striped_bar(ui, app.cpu_fan_pct, COLOR_CYAN);

                ui.add_space(8.0);

                ui.horizontal(|ui| {
                    ui.label(
                        egui::RichText::new("GPU FAN")
                            .size(11.0)
                            .color(COLOR_TEXT_MUTED),
                    );
                    ui.with_layout(egui::Layout::right_to_left(egui::Align::Center), |ui| {
                        ui.label(
                            egui::RichText::new(format!("{} %", app.gpu_fan_pct))
                                .size(11.0)
                                .color(COLOR_TEXT_WHITE),
                        );
                    });
                });
                render_striped_bar(ui, app.gpu_fan_pct, COLOR_CYAN);
            });
    });
}

// ============================================================================
// CENTER COLUMN: VIEW A - SPEEDOMETER TACHOMETER
// ============================================================================
fn render_speedometer_tachometer_view(app: &mut AcerApp, ui: &mut egui::Ui) {
    ui.vertical_centered(|ui| {
        ui.label(
            egui::RichText::new("Processor Telemetry")
                .size(13.0)
                .color(COLOR_TEXT_MUTED),
        );
    });

    let (rect, _) = ui.allocate_exact_size(Vec2::new(ui.available_width(), 175.0), egui::Sense::hover());
    let painter = ui.painter_at(rect);
    let center = Pos2::new(rect.center().x, rect.center().y + 10.0);
    let radius = 68.0;

    // 1. Electromagnetic Radiating Wave Lines (amber glow horizontal waves)
    for side in [-1.0f32, 1.0f32] {
        for wave_idx in 0..4 {
            let offset_y = (wave_idx as f32 - 1.5) * 11.0;
            let mut points = Vec::new();
            for step in 0..24 {
                let t = step as f32 / 23.0;
                let x = center.x + side * (32.0 + t * 90.0);
                let wave = (t * PI * 2.5).sin() * 5.0;
                let y = center.y + offset_y + wave;
                points.push(Pos2::new(x, y));
            }
            let alpha = (120.0 * (1.0 - (wave_idx as f32 * 0.2))) as u8;
            painter.add(egui::Shape::line(
                points,
                Stroke::new(1.1, Color32::from_rgba_premultiplied(255, 179, 0, alpha)),
            ));
        }
    }

    // 2. Center "CPU" Hub with concentric rings and glowing badge
    // Outer subtle purple aura
    painter.circle_stroke(
        center,
        34.0,
        Stroke::new(1.0, Color32::from_rgba_premultiplied(156, 39, 176, 70)),
    );
    // Purple halo ring
    painter.circle_stroke(center, 28.0, Stroke::new(2.0, COLOR_PURPLE_GLOW));
    // Cyan inner ring
    painter.circle_stroke(center, 23.0, Stroke::new(2.0, COLOR_CYAN));
    // Dark core disc
    painter.circle_filled(center, 21.0, Color32::from_rgb(18, 22, 32));
    // Badge Label
    painter.text(
        center,
        egui::Align2::CENTER_CENTER,
        "CPU",
        FontId::proportional(12.5),
        COLOR_TEXT_WHITE,
    );

    // 3. Left Speedometer Arc (0 to 6 x1000 MHz)
    let left_start = PI * 1.5; // top (12 o'clock)
    let clk_ratio = (app.cpu_clock_mhz as f32 / 6000.0).clamp(0.0, 1.0);
    let needle_left_angle = left_start - (PI * clk_ratio);

    // Arc track and scale tick marks (0, 1, 2, 3, 4, 5, 6)
    for i in 0..15 {
        let a1 = left_start - (PI * (i as f32 / 15.0));
        let a2 = left_start - (PI * ((i + 1) as f32 / 15.0));
        let p1 = Pos2::new(center.x + radius * a1.cos(), center.y + radius * a1.sin());
        let p2 = Pos2::new(center.x + radius * a2.cos(), center.y + radius * a2.sin());
        painter.line_segment([p1, p2], Stroke::new(3.5, Color32::from_rgb(33, 41, 55)));
    }

    // Numbers along left arc
    for val in 0..=6 {
        let frac = val as f32 / 6.0;
        let a = left_start - (PI * frac);
        let tick_p = Pos2::new(
            center.x + (radius + 12.0) * a.cos(),
            center.y + (radius + 12.0) * a.sin(),
        );
        painter.text(
            tick_p,
            egui::Align2::CENTER_CENTER,
            val.to_string(),
            FontId::proportional(9.0),
            COLOR_TEXT_MUTED,
        );
    }

    // Left Needle (Gold)
    let n1_end = Pos2::new(
        center.x + (radius - 4.0) * needle_left_angle.cos(),
        center.y + (radius - 4.0) * needle_left_angle.sin(),
    );
    painter.line_segment([center, n1_end], Stroke::new(2.5, COLOR_GOLD));
    painter.circle_filled(center, 4.0, COLOR_GOLD);

    // 4. Right Temperature Arc (Green to Red Gradient)
    let right_start = PI * 1.5; // top (12 o'clock)
    let temp_ratio = (app.cpu_temp as f32 / 100.0).clamp(0.0, 1.0);
    let needle_right_angle = right_start + (PI * temp_ratio);

    for i in 0..15 {
        let a1 = right_start + (PI * (i as f32 / 15.0));
        let a2 = right_start + (PI * ((i + 1) as f32 / 15.0));
        let p1 = Pos2::new(center.x + radius * a1.cos(), center.y + radius * a1.sin());
        let p2 = Pos2::new(center.x + radius * a2.cos(), center.y + radius * a2.sin());

        let col = if i < 5 {
            COLOR_TEMP_GREEN
        } else if i < 10 {
            COLOR_TEMP_YELLOW
        } else {
            COLOR_TEMP_RED
        };
        painter.line_segment([p1, p2], Stroke::new(4.0, col));
    }

    // Right Needle (Cyan)
    let n2_end = Pos2::new(
        center.x + (radius - 4.0) * needle_right_angle.cos(),
        center.y + (radius - 4.0) * needle_right_angle.sin(),
    );
    painter.line_segment([center, n2_end], Stroke::new(2.5, COLOR_CYAN));
    painter.circle_filled(center, 3.0, COLOR_CYAN);

    // Labels beneath arcs
    painter.text(
        Pos2::new(center.x - radius - 12.0, center.y + 12.0),
        egui::Align2::RIGHT_CENTER,
        "Clock speed\nx1000 MHz",
        FontId::proportional(9.5),
        COLOR_TEXT_MUTED,
    );

    painter.text(
        Pos2::new(center.x + radius + 12.0, center.y + 12.0),
        egui::Align2::LEFT_CENTER,
        format!("{} ℃\nTemperature", app.cpu_temp),
        FontId::proportional(9.5),
        COLOR_TEXT_MUTED,
    );
}

// ============================================================================
// CENTER COLUMN: VIEW B - CUSTOM FAN SETTING CURVE EDITOR
// ============================================================================
fn render_custom_fan_setting_view(app: &mut AcerApp, ui: &mut egui::Ui) {
    // Header & Toggle back
    ui.horizontal(|ui| {
        ui.label(
            egui::RichText::new("Custom fan setting")
                .size(14.0)
                .strong()
                .color(COLOR_TEXT_WHITE),
        );

        ui.with_layout(egui::Layout::right_to_left(egui::Align::Center), |ui| {
            if ui.button(egui::RichText::new(">>").size(14.0).color(COLOR_GOLD)).clicked() {
                app.set_fan_mode(FanMode::Automatic);
            }
        });
    });

    // Subtabs: CPU Fan | GPU Fan
    ui.horizontal(|ui| {
        let is_cpu = app.active_curve_tab == 0;
        let is_gpu = app.active_curve_tab == 1;

        // CPU Fan Tab (Gold)
        let (cpu_rect, cpu_resp) = ui.allocate_exact_size(Vec2::new(76.0, 22.0), egui::Sense::click());
        let cp = ui.painter_at(cpu_rect);
        cp.text(
            Pos2::new(cpu_rect.center().x, cpu_rect.center().y - 2.0),
            egui::Align2::CENTER_CENTER,
            "CPU Fan",
            FontId::proportional(12.5),
            if is_cpu { COLOR_GOLD } else { COLOR_TEXT_MUTED },
        );
        if is_cpu {
            let underline = Rect::from_min_size(
                Pos2::new(cpu_rect.min.x + 8.0, cpu_rect.max.y - 2.0),
                Vec2::new(cpu_rect.width() - 16.0, 2.0),
            );
            cp.rect_filled(underline, 1.0, COLOR_GOLD);
        }
        if cpu_resp.clicked() {
            app.active_curve_tab = 0;
        }

        ui.add_space(8.0);

        // GPU Fan Tab (Cyan)
        let (gpu_rect, gpu_resp) = ui.allocate_exact_size(Vec2::new(76.0, 22.0), egui::Sense::click());
        let gp = ui.painter_at(gpu_rect);
        gp.text(
            Pos2::new(gpu_rect.center().x, gpu_rect.center().y - 2.0),
            egui::Align2::CENTER_CENTER,
            "GPU Fan",
            FontId::proportional(12.5),
            if is_gpu { COLOR_CYAN } else { COLOR_TEXT_MUTED },
        );
        if is_gpu {
            let underline = Rect::from_min_size(
                Pos2::new(gpu_rect.min.x + 8.0, gpu_rect.max.y - 2.0),
                Vec2::new(gpu_rect.width() - 16.0, 2.0),
            );
            gp.rect_filled(underline, 1.0, COLOR_CYAN);
        }
        if gpu_resp.clicked() {
            app.active_curve_tab = 1;
        }
    });

    ui.add_space(4.0);

    // 2D Grid Canvas (0-100% vs 20-100℃)
    let line_col = if app.active_curve_tab == 0 {
        COLOR_GOLD
    } else {
        COLOR_CYAN
    };

    let nodes = if app.active_curve_tab == 0 {
        &mut app.cpu_curve_nodes
    } else {
        &mut app.gpu_curve_nodes
    };

    render_fan_curve_grid(ui, nodes, line_col, &mut app.selected_node_idx);

    // Coordinate Readout Boxes & Action Buttons at Bottom
    let active_nodes = if app.active_curve_tab == 0 {
        &app.cpu_curve_nodes
    } else {
        &app.gpu_curve_nodes
    };

    let idx = app.selected_node_idx.min(active_nodes.len().saturating_sub(1));
    let cur_temp = active_nodes[idx].temp.round() as u32;
    let cur_speed = active_nodes[idx].speed.round() as u32;

    ui.add_space(4.0);
    ui.horizontal(|ui| {
        // Reset / Default icon
        if ui.button(egui::RichText::new("⟲").size(12.0).color(COLOR_TEXT_MUTED)).on_hover_text("Reset to default curve").clicked() {
            if app.active_curve_tab == 0 {
                app.cpu_curve_nodes = vec![
                    FanCurveNode { temp: 25.0, speed: 20.0 },
                    FanCurveNode { temp: 45.0, speed: 35.0 },
                    FanCurveNode { temp: 65.0, speed: 55.0 },
                    FanCurveNode { temp: 80.0, speed: 75.0 },
                    FanCurveNode { temp: 95.0, speed: 100.0 },
                ];
            } else {
                app.gpu_curve_nodes = vec![
                    FanCurveNode { temp: 30.0, speed: 0.0 },
                    FanCurveNode { temp: 50.0, speed: 30.0 },
                    FanCurveNode { temp: 65.0, speed: 50.0 },
                    FanCurveNode { temp: 75.0, speed: 70.0 },
                    FanCurveNode { temp: 85.0, speed: 100.0 },
                ];
            }
        }

        ui.add_space(6.0);

        // Readout Boxes
        ui.horizontal(|ui| {
            ui.label(egui::RichText::new("Fan Speed:").size(10.5).color(COLOR_TEXT_MUTED));
            ui.label(
                egui::RichText::new(format!("{} %", cur_speed))
                    .size(11.0)
                    .strong()
                    .color(line_col),
            );

            ui.add_space(10.0);

            ui.label(egui::RichText::new("Temperature:").size(10.5).color(COLOR_TEXT_MUTED));
            ui.label(
                egui::RichText::new(format!("{} ℃", cur_temp))
                    .size(11.0)
                    .strong()
                    .color(line_col),
            );
        });

        ui.with_layout(egui::Layout::right_to_left(egui::Align::Center), |ui| {
            if ui.button(egui::RichText::new("💾 Apply").size(10.5).color(COLOR_CYAN)).clicked() {
                app.set_fan_mode(FanMode::Custom);
            }
        });
    });
}

// ----------------------------------------------------------------------------
// 2D FAN CURVE INTERACTIVE GRID
// ----------------------------------------------------------------------------
fn render_fan_curve_grid(
    ui: &mut egui::Ui,
    nodes: &mut Vec<FanCurveNode>,
    line_col: Color32,
    selected_idx: &mut usize,
) {
    let (rect, response) = ui.allocate_exact_size(
        Vec2::new(ui.available_width(), 132.0),
        egui::Sense::click_and_drag(),
    );
    let painter = ui.painter_at(rect);

    // Background surface
    painter.rect_filled(rect, 4.0, Color32::from_rgb(11, 14, 20));
    painter.rect_stroke(rect, 4.0, Stroke::new(1.0, Color32::from_rgb(26, 33, 46)), StrokeKind::Inside);

    // Coordinate space inside padding
    let pad_left = 28.0;
    let pad_right = 16.0;
    let pad_top = 10.0;
    let pad_bottom = 20.0;

    let graph_rect = Rect::from_min_max(
        Pos2::new(rect.min.x + pad_left, rect.min.y + pad_top),
        Pos2::new(rect.max.x - pad_right, rect.max.y - pad_bottom),
    );

    let grid_stroke = Stroke::new(0.8, Color32::from_rgb(24, 30, 42));

    // Y Gridlines (0% to 100%)
    for y_pct in [0.0, 0.25, 0.5, 0.75, 1.0] {
        let y = graph_rect.max.y - (graph_rect.height() * y_pct);
        let stroke = if (y_pct - 1.0).abs() < 1e-4 {
            Stroke::new(1.0, Color32::from_rgb(55, 68, 88))
        } else {
            grid_stroke
        };
        painter.line_segment(
            [Pos2::new(graph_rect.min.x, y), Pos2::new(graph_rect.max.x, y)],
            stroke,
        );

        // Y label
        let val_pct = (y_pct * 100.0).round() as u32;
        painter.text(
            Pos2::new(rect.min.x + pad_left - 4.0, y),
            egui::Align2::RIGHT_CENTER,
            format!("{}", val_pct),
            FontId::proportional(8.5),
            COLOR_TEXT_DARK,
        );
    }
    // Y-axis unit
    painter.text(
        Pos2::new(rect.min.x + 8.0, rect.min.y + 4.0),
        egui::Align2::LEFT_TOP,
        "%",
        FontId::proportional(8.5),
        COLOR_TEXT_DARK,
    );

    // X Gridlines (20℃ to 100℃)
    for x_frac in [0.0, 0.25, 0.5, 0.75, 1.0] {
        let x = graph_rect.min.x + (graph_rect.width() * x_frac);
        painter.line_segment(
            [Pos2::new(x, graph_rect.min.y), Pos2::new(x, graph_rect.max.y)],
            grid_stroke,
        );

        // X label
        let temp_val = 20.0 + x_frac * 80.0;
        painter.text(
            Pos2::new(x, graph_rect.max.y + 10.0),
            egui::Align2::CENTER_CENTER,
            format!("{}", temp_val.round() as u32),
            FontId::proportional(8.5),
            COLOR_TEXT_DARK,
        );
    }
    // X-axis unit
    painter.text(
        Pos2::new(rect.max.x - 12.0, graph_rect.max.y + 10.0),
        egui::Align2::RIGHT_CENTER,
        "℃",
        FontId::proportional(8.5),
        COLOR_TEXT_DARK,
    );

    // Helper: Node to Screen Coordinates
    let node_to_screen = |node: &FanCurveNode| -> Pos2 {
        let t_norm = ((node.temp - 20.0) / 80.0).clamp(0.0, 1.0);
        let s_norm = (node.speed / 100.0).clamp(0.0, 1.0);
        let x = graph_rect.min.x + t_norm * graph_rect.width();
        let y = graph_rect.max.y - s_norm * graph_rect.height();
        Pos2::new(x, y)
    };

    // Draw connecting curve lines
    for i in 0..nodes.len().saturating_sub(1) {
        let p1 = node_to_screen(&nodes[i]);
        let p2 = node_to_screen(&nodes[i + 1]);
        painter.line_segment([p1, p2], Stroke::new(2.5, line_col));
    }

    // Draw circular nodes
    for (i, node) in nodes.iter().enumerate() {
        let p = node_to_screen(node);
        let is_sel = *selected_idx == i;

        if is_sel {
            // Outer double-ring halo with crosshair style
            painter.circle_stroke(p, 8.5, Stroke::new(1.2, line_col));
            painter.circle_filled(p, 6.0, COLOR_TEXT_WHITE);
            painter.circle_stroke(p, 6.0, Stroke::new(2.0, line_col));
        } else {
            painter.circle_filled(p, 4.5, COLOR_TEXT_WHITE);
            painter.circle_stroke(p, 4.5, Stroke::new(1.8, line_col));
        }
    }

    // Interactive Dragging & Clicking
    if let Some(mouse_pos) = response.interact_pointer_pos() {
        if response.dragged() {
            let idx = *selected_idx.min(&mut (nodes.len() - 1));
            let new_t = (20.0 + ((mouse_pos.x - graph_rect.min.x) / graph_rect.width()) * 80.0)
                .clamp(20.0, 100.0);
            let new_s = (((graph_rect.max.y - mouse_pos.y) / graph_rect.height()) * 100.0)
                .clamp(0.0, 100.0);

            let min_t = if idx > 0 { nodes[idx - 1].temp + 1.0 } else { 20.0 };
            let max_t = if idx + 1 < nodes.len() { nodes[idx + 1].temp - 1.0 } else { 100.0 };

            nodes[idx].temp = new_t.clamp(min_t, max_t);
            nodes[idx].speed = new_s;
        } else if response.clicked() {
            // Pick closest node
            for (i, node) in nodes.iter().enumerate() {
                let p = node_to_screen(node);
                if (p.x - mouse_pos.x).hypot(p.y - mouse_pos.y) < 14.0 {
                    *selected_idx = i;
                    break;
                }
            }
        }
    }
}

// ============================================================================
// 4. BOTTOM DECK (3 MODULAR FRAMED PANELS)
// ============================================================================
fn render_bottom_deck(app: &mut AcerApp, ui: &mut egui::Ui) {
    ui.columns(3, |cols| {
        // --------------------------------------------------------------------
        // PANEL 1: MEMORY AND STORAGE
        // --------------------------------------------------------------------
        let p1 = &mut cols[0];
        egui::Frame::new()
            .fill(COLOR_CARD_BG)
            .stroke(Stroke::new(1.0, COLOR_CARD_BORDER))
            .corner_radius(8)
            .inner_margin(12)
            .show(p1, |ui| {
                ui.label(
                    egui::RichText::new("Memory and storage")
                        .size(13.0)
                        .strong()
                        .color(COLOR_TEXT_WHITE),
                );
                ui.add_space(6.0);
                ui.horizontal(|ui| {
                    render_circular_dial(
                        ui,
                        "Memory",
                        app.mem_pct,
                        &format!("{:.0}GB", app.mem_total_gb),
                        COLOR_CYAN,
                    );
                    ui.add_space(8.0);
                    render_circular_dial(
                        ui,
                        "Storage",
                        app.disk_pct,
                        &format!("{}GB", app.disk_total_gb),
                        COLOR_CYAN,
                    );
                });
            });

        // --------------------------------------------------------------------
        // PANEL 2: PREFERENCE (8/9 ROUNDED SQUARE TOGGLE TILES)
        // --------------------------------------------------------------------
        let p2 = &mut cols[1];
        egui::Frame::new()
            .fill(COLOR_CARD_BG)
            .stroke(Stroke::new(1.0, COLOR_CARD_BORDER))
            .corner_radius(8)
            .inner_margin(10)
            .show(p2, |ui| {
                ui.label(
                    egui::RichText::new("Preference")
                        .size(13.0)
                        .strong()
                        .color(COLOR_TEXT_WHITE),
                );
                ui.add_space(6.0);

                ui.horizontal(|ui| {
                    render_preference_toggle(ui, "Win key\nlock", &mut app.win_key_lock, COLOR_CYAN);
                    render_preference_toggle(ui, "Swap\nWin/Fn", &mut app.swap_win_fn, COLOR_CYAN);
                    render_preference_toggle(ui, "Fn key\nlock", &mut app.fn_key_lock, COLOR_CYAN);
                    render_preference_toggle(ui, "Touch\nPad", &mut app.touchpad_enabled, COLOR_CYAN);
                    render_preference_toggle(ui, "Camera", &mut app.camera_enabled, COLOR_CYAN);
                    render_preference_toggle(ui, "Airplane", &mut app.airplane_mode, COLOR_CYAN);
                    render_preference_toggle(ui, "Caps\nLock", &mut app.caps_lock, COLOR_CYAN);
                    render_preference_toggle(ui, "Num\nLock", &mut app.num_lock, COLOR_CYAN);
                });
            });

        // --------------------------------------------------------------------
        // PANEL 3: CONFIGURATION & GPU SWITCH
        // --------------------------------------------------------------------
        let p3 = &mut cols[2];
        egui::Frame::new()
            .fill(COLOR_CARD_BG)
            .stroke(Stroke::new(1.0, COLOR_CARD_BORDER))
            .corner_radius(8)
            .inner_margin(12)
            .show(p3, |ui| {
                ui.horizontal(|ui| {
                    // Left Subpanel: Blue Light Filter & Mode Presets
                    let half_w = (ui.available_width() - 14.0) / 2.0;
                    ui.allocate_ui(Vec2::new(half_w, 75.0), |ui| {
                        ui.label(
                            egui::RichText::new("Configuration")
                                .size(12.5)
                                .strong()
                                .color(COLOR_TEXT_WHITE),
                        );
                        ui.horizontal(|ui| {
                            ui.checkbox(&mut app.blue_light_filter, "Blue light filter");
                        });
                        ui.add(
                            egui::Slider::new(&mut app.blue_light_level, 0.0..=10.0)
                                .show_value(false),
                        );

                        // Mode Presets (3 circular mode buttons)
                        ui.horizontal(|ui| {
                            let modes = [
                                (0, COLOR_CYAN, "Standard"),
                                (1, COLOR_GOLD, "Eye Care"),
                                (2, COLOR_PURPLE_GLOW, "Vivid"),
                            ];
                            for (p_idx, col, name) in modes {
                                let (r, resp) = ui.allocate_exact_size(Vec2::new(14.0, 14.0), egui::Sense::click());
                                let p = ui.painter_at(r);
                                let is_sel = app.display_preset == p_idx;
                                p.circle_filled(r.center(), 5.0, col);
                                if is_sel {
                                    p.circle_stroke(r.center(), 7.0, Stroke::new(1.5, COLOR_TEXT_WHITE));
                                }
                                if resp.clicked() {
                                    app.display_preset = p_idx;
                                }
                                let _ = resp.on_hover_text(name);
                                ui.add_space(2.0);
                            }
                        });
                    });

                    ui.separator();

                    // Right Subpanel: GPU Switch Radios (Integrated, Dynamic, Discrete)
                    ui.allocate_ui(Vec2::new(half_w, 75.0), |ui| {
                        ui.label(
                            egui::RichText::new("GPU switch")
                                .size(12.5)
                                .strong()
                                .color(COLOR_TEXT_WHITE),
                        );

                        let g_modes = [
                            (GpuSwitchMode::Integrated, "Integrated"),
                            (GpuSwitchMode::Dynamic, "Dynamic"),
                            (GpuSwitchMode::Discrete, "Discrete"),
                        ];

                        for (g_mode, g_lbl) in g_modes {
                            let is_checked = app.gpu_switch == g_mode;
                            if ui
                                .radio(
                                    is_checked,
                                    egui::RichText::new(g_lbl)
                                        .size(11.0)
                                        .color(if is_checked { COLOR_CYAN } else { COLOR_TEXT_MUTED }),
                                )
                                .clicked()
                            {
                                app.set_gpu_switch(g_mode);
                            }
                        }
                    });
                });
            });
    });
}

// ============================================================================
// 5. BOTTOM CAROUSEL / SCROLLBAR TRACK
// ============================================================================
fn render_bottom_scrollbar(app: &mut AcerApp, ui: &mut egui::Ui) {
    ui.add_space(2.0);
    ui.horizontal(|ui| {
        // Left chevron
        let (left_r, left_resp) = ui.allocate_exact_size(Vec2::new(14.0, 12.0), egui::Sense::click());
        let lp = ui.painter_at(left_r);
        lp.text(
            left_r.center(),
            egui::Align2::CENTER_CENTER,
            "<",
            FontId::proportional(11.0),
            if left_resp.hovered() { COLOR_CYAN } else { COLOR_TEXT_MUTED },
        );
        if left_resp.clicked() {
            app.scroll_pos = (app.scroll_pos - 0.2).clamp(0.0, 1.0);
        }

        // Middle track rail
        let track_w = ui.available_width() - 20.0;
        let (track_rect, track_resp) =
            ui.allocate_exact_size(Vec2::new(track_w, 10.0), egui::Sense::click_and_drag());
        let tp = ui.painter_at(track_rect);

        // Rail background
        tp.rect_filled(track_rect, 3.0, Color32::from_rgb(16, 20, 28));

        // Thumb pill
        let thumb_w = track_rect.width() * 0.42;
        let thumb_x = track_rect.min.x + (track_rect.width() - thumb_w) * app.scroll_pos;
        let thumb_rect = Rect::from_min_size(
            Pos2::new(thumb_x, track_rect.min.y + 1.0),
            Vec2::new(thumb_w, track_rect.height() - 2.0),
        );

        let thumb_col = if track_resp.dragged() {
            Color32::from_rgb(60, 75, 98)
        } else if track_resp.hovered() {
            Color32::from_rgb(48, 60, 80)
        } else {
            Color32::from_rgb(38, 48, 64)
        };
        tp.rect_filled(thumb_rect, 3.0, thumb_col);

        if track_resp.dragged() {
            if let Some(pos) = track_resp.interact_pointer_pos() {
                let frac = (pos.x - track_rect.min.x - thumb_w / 2.0)
                    / (track_rect.width() - thumb_w);
                app.scroll_pos = frac.clamp(0.0, 1.0);
            }
        }

        // Right chevron
        let (right_r, right_resp) = ui.allocate_exact_size(Vec2::new(14.0, 12.0), egui::Sense::click());
        let rp = ui.painter_at(right_r);
        rp.text(
            right_r.center(),
            egui::Align2::CENTER_CENTER,
            ">",
            FontId::proportional(11.0),
            if right_resp.hovered() { COLOR_CYAN } else { COLOR_TEXT_MUTED },
        );
        if right_resp.clicked() {
            app.scroll_pos = (app.scroll_pos + 0.2).clamp(0.0, 1.0);
        }
    });
}

// ============================================================================
// WIDGET RENDER HELPERS
// ============================================================================

/// Renders authentic diagonally segmented/striped usage bars matching OEM UI
pub fn render_striped_bar(ui: &mut egui::Ui, pct: u32, fill_col: Color32) {
    let (rect, _) = ui.allocate_exact_size(Vec2::new(ui.available_width(), 12.0), egui::Sense::hover());
    let painter = ui.painter_at(rect);

    // Track surface
    painter.rect_filled(rect, 2.0, Color32::from_rgb(20, 25, 35));
    painter.rect_stroke(rect, 2.0, Stroke::new(1.0, Color32::from_rgb(32, 40, 54)), StrokeKind::Inside);

    let num_segments = 26;
    let filled_segments = (num_segments as f32 * (pct as f32 / 100.0)).round() as usize;
    let gap = 2.0;
    let seg_w = (rect.width() - (num_segments as f32 - 1.0) * gap) / num_segments as f32;

    for i in 0..num_segments {
        let x = rect.min.x + (i as f32) * (seg_w + gap);
        let seg_rect = Rect::from_min_size(
            Pos2::new(x, rect.min.y + 1.5),
            Vec2::new(seg_w, rect.height() - 3.0),
        );
        let col = if i < filled_segments {
            fill_col
        } else {
            Color32::from_rgb(30, 38, 50)
        };
        painter.rect_filled(seg_rect, 1.0, col);
    }
}

/// Renders circular dial gauge (Memory / Storage) with 270-degree arc, ticks, and cyan needle/arc
pub fn render_circular_dial(ui: &mut egui::Ui, label: &str, pct: u32, subtext: &str, accent: Color32) {
    let (rect, _) = ui.allocate_exact_size(Vec2::new(72.0, 72.0), egui::Sense::hover());
    let painter = ui.painter_at(rect);
    let center = rect.center();
    let radius = 28.0;

    let start_angle = PI * 0.75; // 135 degrees
    let end_angle = PI * 2.25;   // 405 degrees (270 degree sweep)

    // Inactive background arc track
    for i in 0..20 {
        let a1 = start_angle + (end_angle - start_angle) * (i as f32 / 20.0);
        let a2 = start_angle + (end_angle - start_angle) * ((i + 1) as f32 / 20.0);
        let p1 = Pos2::new(center.x + radius * a1.cos(), center.y + radius * a1.sin());
        let p2 = Pos2::new(center.x + radius * a2.cos(), center.y + radius * a2.sin());
        painter.line_segment([p1, p2], Stroke::new(3.0, Color32::from_rgb(28, 35, 48)));
    }

    // Active cyan arc
    let active_segments = (20.0 * (pct as f32 / 100.0)).round() as usize;
    for i in 0..active_segments {
        let a1 = start_angle + (end_angle - start_angle) * (i as f32 / 20.0);
        let a2 = start_angle + (end_angle - start_angle) * ((i + 1) as f32 / 20.0);
        let p1 = Pos2::new(center.x + radius * a1.cos(), center.y + radius * a1.sin());
        let p2 = Pos2::new(center.x + radius * a2.cos(), center.y + radius * a2.sin());
        painter.line_segment([p1, p2], Stroke::new(3.5, accent));
    }

    // Gauge center readouts
    painter.text(
        Pos2::new(center.x, center.y - 6.0),
        egui::Align2::CENTER_CENTER,
        format!("{}%", pct),
        FontId::proportional(12.0),
        COLOR_TEXT_WHITE,
    );
    painter.text(
        Pos2::new(center.x, center.y + 6.0),
        egui::Align2::CENTER_CENTER,
        label,
        FontId::proportional(9.0),
        COLOR_TEXT_MUTED,
    );
    painter.text(
        Pos2::new(center.x, center.y + 16.0),
        egui::Align2::CENTER_CENTER,
        subtext,
        FontId::proportional(8.5),
        accent,
    );
}

/// Renders preference quick toggle button (rounded square tile with glowing cyan border when active)
pub fn render_preference_toggle(ui: &mut egui::Ui, label: &str, state: &mut bool, accent: Color32) {
    let is_on = *state;
    let border = if is_on {
        accent
    } else {
        Color32::from_rgb(38, 48, 64)
    };
    let bg = if is_on {
        Color32::from_rgb(14, 38, 54)
    } else {
        Color32::from_rgb(19, 24, 34)
    };
    let text_col = if is_on {
        Color32::WHITE
    } else {
        COLOR_TEXT_MUTED
    };

    let btn = ui.add(
        egui::Button::new(egui::RichText::new(label).size(8.5).color(text_col))
            .fill(bg)
            .stroke(Stroke::new(if is_on { 1.5 } else { 1.0 }, border))
            .corner_radius(4)
            .min_size(Vec2::new(35.0, 38.0)),
    );

    if btn.clicked() {
        *state = !*state;
    }
}

// ============================================================================
// UI UNIT TESTS
// ============================================================================
#[cfg(test)]
mod tests {
    use super::*;

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
        assert_eq!(app.power_mode, PowerMode::Performance);
        assert_eq!(app.fan_mode, FanMode::Automatic);
        assert_eq!(app.gpu_switch, GpuSwitchMode::Discrete);
        assert!(app.cpu_curve_nodes.len() >= 5);
        assert!(app.gpu_curve_nodes.len() >= 5);
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

        // Ensure temp clamp logic between neighbors
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
}

