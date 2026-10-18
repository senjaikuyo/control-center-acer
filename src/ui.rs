use crate::backend::{FanCurveNode, FanMode, GpuSwitchMode, PowerMode};
use crate::AcerApp;
use eframe::egui::{self, Color32, Pos2, Rect, Stroke, Vec2};

// ============================================================================
// CLEAN MODERN DESIGN TOKENS (SWISS & MODERN BENTO)
// ============================================================================
pub const BG_CANVAS: Color32 = Color32::from_rgb(13, 17, 23);       // Slate deep canvas
pub const BG_CARD: Color32 = Color32::from_rgb(22, 27, 34);         // Surface card
pub const BG_CARD_SUBTLE: Color32 = Color32::from_rgb(28, 33, 40);  // Subtle card hover/track
pub const BORDER_COLOR: Color32 = Color32::from_rgb(48, 54, 61);    // Clean separator border

pub const ACCENT_BLUE: Color32 = Color32::from_rgb(88, 166, 255);   // Primary brand
pub const ACCENT_GREEN: Color32 = Color32::from_rgb(63, 185, 80);   // Success/Normal
pub const ACCENT_ORANGE: Color32 = Color32::from_rgb(210, 153, 34); // Warning/Boost
pub const ACCENT_RED: Color32 = Color32::from_rgb(248, 81, 73);     // Hot/Turbo

pub const TEXT_PRIMARY: Color32 = Color32::from_rgb(240, 246, 252); // High-contrast text
pub const TEXT_SECONDARY: Color32 = Color32::from_rgb(139, 148, 158); // Muted description
pub const TEXT_DIM: Color32 = Color32::from_rgb(110, 118, 129);      // Micro labels

// ============================================================================
// MAIN RENDER ENTRY POINT
// ============================================================================
pub fn render(app: &mut AcerApp, ctx: &egui::Context, ui: &mut egui::Ui) {
    ctx.request_repaint_after(std::time::Duration::from_millis(500));

    // Paint clean dark slate canvas
    let full_rect = ui.max_rect();
    ui.painter().rect_filled(full_rect, 0.0, BG_CANVAS);

    // Main margin
    egui::Frame::new()
        .inner_margin(egui::Margin::symmetric(24, 20))
        .show(ui, |ui| {
            ui.spacing_mut().item_spacing = Vec2::new(16.0, 16.0);

            // 1. App Header
            render_header(app, ui);

            ui.add_space(4.0);

            // 2. Main Bento Grid (Top Row: KPI Cards)
            render_top_kpis(app, ui);

            // 3. Middle Section: Cooling & Fan Control
            render_cooling_section(app, ui);

            // 4. Bottom Section: System Modes & Preferences
            render_bottom_section(app, ui);
        });
}

// ============================================================================
// 1. APP HEADER
// ============================================================================
fn render_header(app: &mut AcerApp, ui: &mut egui::Ui) {
    ui.horizontal(|ui| {
        ui.vertical(|ui| {
            ui.horizontal(|ui| {
                ui.label(
                    egui::RichText::new("Control Center")
                        .size(20.0)
                        .strong()
                        .color(TEXT_PRIMARY),
                );
                ui.add_space(8.0);
                egui::Frame::new()
                    .fill(Color32::from_rgb(20, 38, 28))
                    .stroke(Stroke::new(1.0, ACCENT_GREEN))
                    .corner_radius(12)
                    .inner_margin(egui::Margin::symmetric(8, 2))
                    .show(ui, |ui| {
                        ui.label(
                            egui::RichText::new("ONLINE")
                                .size(10.0)
                                .strong()
                                .color(ACCENT_GREEN),
                        );
                    });
            });
            ui.label(
                egui::RichText::new("Hardware telemetry and active cooling management")
                    .size(12.0)
                    .color(TEXT_SECONDARY),
            );
        });

        // Right side: Power mode quick switcher
        ui.with_layout(egui::Layout::right_to_left(egui::Align::Center), |ui| {
            let modes = [
                (PowerMode::Performance, "Performance"),
                (PowerMode::Entertainment, "Balanced"),
                (PowerMode::Quiet, "Quiet"),
            ];

            egui::Frame::new()
                .fill(BG_CARD)
                .stroke(Stroke::new(1.0, BORDER_COLOR))
                .corner_radius(8)
                .inner_margin(egui::Margin::symmetric(4, 4))
                .show(ui, |ui| {
                    ui.horizontal(|ui| {
                        for (mode, label) in modes {
                            let is_active = app.power_mode == mode;
                            let btn = ui.add(
                                egui::Button::new(
                                    egui::RichText::new(label)
                                        .size(12.0)
                                        .strong()
                                        .color(if is_active { Color32::WHITE } else { TEXT_SECONDARY }),
                                )
                                .fill(if is_active { ACCENT_BLUE } else { Color32::TRANSPARENT })
                                .corner_radius(6)
                                .min_size(Vec2::new(80.0, 26.0)),
                            );
                            if btn.clicked() {
                                app.set_power_profile(mode);
                            }
                        }
                    });
                });
        });
    });
}

// ============================================================================
// 2. TOP METRIC CARDS (BENTO KPIS)
// ============================================================================
fn render_top_kpis(app: &AcerApp, ui: &mut egui::Ui) {
    ui.columns(4, |cols| {
        // CPU Card
        render_kpi_card(
            &mut cols[0],
            "CPU",
            "Core i5-13420H",
            &format!("{} ℃", app.cpu_temp),
            &format!("{:.2} GHz • {}%", app.cpu_clock_mhz as f32 / 1000.0, app.cpu_usage_pct),
            temp_to_color(app.cpu_temp),
            app.cpu_usage_pct,
        );

        // GPU Card
        let (gpu_temp_str, gpu_subtext) = if app.gpu_clock_mhz > 0 || app.gpu_temp > 0 {
            (format!("{} ℃", app.gpu_temp), format!("{} MHz • {}%", app.gpu_clock_mhz, app.gpu_usage_pct))
        } else {
            ("SLEEP".to_string(), "Optimus Standby".to_string())
        };
        render_kpi_card(
            &mut cols[1],
            "GPU",
            "GeForce RTX 3050",
            &gpu_temp_str,
            &gpu_subtext,
            if app.gpu_temp > 0 { temp_to_color(app.gpu_temp) } else { TEXT_SECONDARY },
            app.gpu_usage_pct,
        );

        // Memory Card
        render_kpi_card(
            &mut cols[2],
            "MEMORY",
            &format!("{:.1} / {:.1} GB", app.mem_used_gb, app.mem_total_gb),
            &format!("{}%", app.mem_pct),
            "System RAM in use",
            ACCENT_BLUE,
            app.mem_pct,
        );

        // Storage Card
        render_kpi_card(
            &mut cols[3],
            "STORAGE",
            &format!("{} / {} GB", app.disk_used_gb, app.disk_total_gb),
            &format!("{}%", app.disk_pct),
            "Root partition usage",
            ACCENT_BLUE,
            app.disk_pct,
        );
    });
}

fn render_kpi_card(
    ui: &mut egui::Ui,
    badge: &str,
    title: &str,
    metric: &str,
    subtext: &str,
    accent: Color32,
    progress_pct: u32,
) {
    egui::Frame::new()
        .fill(BG_CARD)
        .stroke(Stroke::new(1.0, BORDER_COLOR))
        .corner_radius(10)
        .inner_margin(14)
        .show(ui, |ui| {
            ui.horizontal(|ui| {
                ui.label(egui::RichText::new(badge).size(11.0).strong().color(accent));
                ui.with_layout(egui::Layout::right_to_left(egui::Align::Center), |ui| {
                    ui.label(egui::RichText::new(title).size(11.0).color(TEXT_SECONDARY));
                });
            });

            ui.add_space(6.0);
            ui.label(egui::RichText::new(metric).size(24.0).strong().color(TEXT_PRIMARY));
            ui.label(egui::RichText::new(subtext).size(11.0).color(TEXT_SECONDARY));

            ui.add_space(10.0);
            render_clean_progress_bar(ui, progress_pct, accent);
        });
}

// ============================================================================
// 3. COOLING & FAN CONTROL SECTION
// ============================================================================
fn render_cooling_section(app: &mut AcerApp, ui: &mut egui::Ui) {
    egui::Frame::new()
        .fill(BG_CARD)
        .stroke(Stroke::new(1.0, BORDER_COLOR))
        .corner_radius(12)
        .inner_margin(18)
        .show(ui, |ui| {
            ui.horizontal(|ui| {
                ui.vertical(|ui| {
                    ui.label(egui::RichText::new("Fan Cooling & Profiles").size(15.0).strong().color(TEXT_PRIMARY));
                    ui.label(egui::RichText::new("Adjust dynamic curve or select operational presets").size(12.0).color(TEXT_SECONDARY));
                });

                ui.with_layout(egui::Layout::right_to_left(egui::Align::Center), |ui| {
                    let fan_modes = [
                        (FanMode::Turbo, "Turbo (100%)"),
                        (FanMode::Maximum, "Max"),
                        (FanMode::Custom, "Custom Curve"),
                        (FanMode::Automatic, "Auto Curve"),
                    ];

                    for (mode, label) in fan_modes {
                        let is_active = app.fan_mode == mode;
                        let btn = ui.add(
                            egui::Button::new(
                                egui::RichText::new(label)
                                    .size(12.0)
                                    .strong()
                                    .color(if is_active { Color32::WHITE } else { TEXT_SECONDARY }),
                            )
                            .fill(if is_active {
                                if mode == FanMode::Turbo { ACCENT_RED } else { ACCENT_BLUE }
                            } else {
                                BG_CARD_SUBTLE
                            })
                            .stroke(Stroke::new(1.0, if is_active { Color32::TRANSPARENT } else { BORDER_COLOR }))
                            .corner_radius(6)
                            .min_size(Vec2::new(90.0, 28.0)),
                        );
                        if btn.clicked() {
                            app.set_fan_mode(mode);
                        }
                        ui.add_space(4.0);
                    }
                });
            });

            ui.add_space(14.0);

            ui.columns(2, |cols| {
                // Left Column: Live Fan Telemetry
                let col_left = &mut cols[0];
                col_left.vertical(|ui| {
                    ui.label(egui::RichText::new("REAL-TIME RPM").size(11.0).strong().color(TEXT_DIM));
                    ui.add_space(8.0);

                    // CPU Fan Row
                    render_fan_metric_row(
                        ui,
                        "CPU Fan",
                        app.cpu_fan_rpm,
                        app.cpu_fan_pct,
                        ACCENT_BLUE,
                    );

                    ui.add_space(12.0);

                    // GPU Fan Row
                    render_fan_metric_row(
                        ui,
                        "GPU Fan",
                        app.gpu_fan_rpm,
                        app.gpu_fan_pct,
                        ACCENT_GREEN,
                    );
                });

                // Right Column: Interactive Curve Editor
                let col_right = &mut cols[1];
                col_right.vertical(|ui| {
                    ui.horizontal(|ui| {
                        ui.label(egui::RichText::new("CURVE CONFIGURATION").size(11.0).strong().color(TEXT_DIM));
                        ui.with_layout(egui::Layout::right_to_left(egui::Align::Center), |ui| {
                            let is_cpu = app.active_curve_tab == 0;
                            if ui.selectable_label(!is_cpu, "GPU Curve").clicked() {
                                app.active_curve_tab = 1;
                            }
                            if ui.selectable_label(is_cpu, "CPU Curve").clicked() {
                                app.active_curve_tab = 0;
                            }
                        });
                    });

                    ui.add_space(8.0);

                    let is_cpu = app.active_curve_tab == 0;
                    let (cur_temp, cur_speed) = {
                        let nodes = if is_cpu {
                            &mut app.cpu_curve_nodes
                        } else {
                            &mut app.gpu_curve_nodes
                        };

                        render_clean_curve_graph(ui, nodes, if is_cpu { ACCENT_BLUE } else { ACCENT_GREEN }, &mut app.selected_node_idx);
                        let idx = app.selected_node_idx.min(nodes.len().saturating_sub(1));
                        (nodes[idx].temp as u32, nodes[idx].speed as u32)
                    };

                    ui.add_space(6.0);
                    let mut should_reset = false;
                    ui.horizontal(|ui| {
                        ui.label(
                            egui::RichText::new(format!("Selected Point: {} ℃  ➔  {} %", cur_temp, cur_speed))
                                .size(11.0)
                                .color(TEXT_SECONDARY),
                        );
                        ui.with_layout(egui::Layout::right_to_left(egui::Align::Center), |ui| {
                            if ui.small_button("Reset Curve").clicked() {
                                should_reset = true;
                            }
                        });
                    });

                    if should_reset {
                        if is_cpu {
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
                });
            });
        });
}

fn render_fan_metric_row(ui: &mut egui::Ui, name: &str, rpm: u32, pct: u32, accent: Color32) {
    egui::Frame::new()
        .fill(BG_CARD_SUBTLE)
        .stroke(Stroke::new(1.0, BORDER_COLOR))
        .corner_radius(8)
        .inner_margin(12)
        .show(ui, |ui| {
            ui.horizontal(|ui| {
                ui.label(egui::RichText::new(name).size(13.0).strong().color(TEXT_PRIMARY));
                ui.with_layout(egui::Layout::right_to_left(egui::Align::Center), |ui| {
                    ui.label(egui::RichText::new(format!("{} %", pct)).size(13.0).strong().color(accent));
                    ui.add_space(8.0);
                    ui.label(egui::RichText::new(format!("{} RPM", rpm)).size(13.0).color(TEXT_SECONDARY));
                });
            });
            ui.add_space(6.0);
            render_clean_progress_bar(ui, pct, accent);
        });
}

// ============================================================================
// 4. BOTTOM SECTION: QUICK PREFERENCES & SYSTEM INFO
// ============================================================================
fn render_bottom_section(app: &mut AcerApp, ui: &mut egui::Ui) {
    ui.columns(2, |cols| {
        // Left Column: Peripheral & Hardware Toggles
        let col_left = &mut cols[0];
        egui::Frame::new()
            .fill(BG_CARD)
            .stroke(Stroke::new(1.0, BORDER_COLOR))
            .corner_radius(12)
            .inner_margin(16)
            .show(col_left, |ui| {
                ui.label(egui::RichText::new("Hardware Preferences").size(14.0).strong().color(TEXT_PRIMARY));
                ui.label(egui::RichText::new("Quick toggles for keyboard and peripherals").size(11.0).color(TEXT_SECONDARY));

                ui.add_space(12.0);

                ui.horizontal(|ui| {
                    if render_toggle_chip(ui, "Touchpad", &mut app.touchpad_enabled) {
                        app.controller.set_touchpad_enabled(app.touchpad_enabled);
                    }
                    let _ = render_toggle_chip(ui, "Camera", &mut app.camera_enabled);
                    let _ = render_toggle_chip(ui, "Win Lock", &mut app.win_key_lock);
                    let _ = render_toggle_chip(ui, "Fn Lock", &mut app.fn_key_lock);
                    if render_toggle_chip(ui, "Airplane", &mut app.airplane_mode) {
                        app.controller.set_airplane_mode(app.airplane_mode);
                    }
                });
            });

        // Right Column: Display Comfort & GPU Switching
        let col_right = &mut cols[1];
        egui::Frame::new()
            .fill(BG_CARD)
            .stroke(Stroke::new(1.0, BORDER_COLOR))
            .corner_radius(12)
            .inner_margin(16)
            .show(col_right, |ui| {
                ui.label(egui::RichText::new("Display & GPU Switching").size(14.0).strong().color(TEXT_PRIMARY));
                ui.label(egui::RichText::new("Graphics power routing and blue light reduction").size(11.0).color(TEXT_SECONDARY));

                ui.add_space(10.0);

                ui.horizontal(|ui| {
                    let _prev_filter = app.blue_light_filter;
                    let _prev_level = app.blue_light_level;

                    if ui.checkbox(&mut app.blue_light_filter, "Night Filter").changed() {
                        app.controller.set_night_light(app.blue_light_filter, app.blue_light_level);
                    }

                    if app.blue_light_filter {
                        if ui.add(egui::Slider::new(&mut app.blue_light_level, 0.0..=10.0).text("Level")).changed() {
                            app.controller.set_night_light(true, app.blue_light_level);
                        }
                    }

                    ui.with_layout(egui::Layout::right_to_left(egui::Align::Center), |ui| {
                        let gpu_modes = [
                            (GpuSwitchMode::Discrete, "dGPU"),
                            (GpuSwitchMode::Dynamic, "Dynamic"),
                            (GpuSwitchMode::Integrated, "iGPU"),
                        ];
                        for (mode, label) in gpu_modes {
                            if ui.selectable_label(app.gpu_switch == mode, label).clicked() {
                                app.set_gpu_switch(mode);
                            }
                        }
                    });
                });
            });
    });
}

fn render_toggle_chip(ui: &mut egui::Ui, label: &str, state: &mut bool) -> bool {
    let is_on = *state;
    let bg = if is_on { Color32::from_rgb(20, 38, 55) } else { BG_CARD_SUBTLE };
    let border = if is_on { ACCENT_BLUE } else { BORDER_COLOR };
    let text_col = if is_on { ACCENT_BLUE } else { TEXT_SECONDARY };

    let btn = ui.add(
        egui::Button::new(egui::RichText::new(label).size(11.0).strong().color(text_col))
            .fill(bg)
            .stroke(Stroke::new(1.0, border))
            .corner_radius(6)
            .min_size(Vec2::new(70.0, 30.0)),
    );
    if btn.clicked() {
        *state = !*state;
        true
    } else {
        false
    }
}

// ============================================================================
// CANVAS PRIMITIVES
// ============================================================================
fn render_clean_progress_bar(ui: &mut egui::Ui, pct: u32, fill_col: Color32) {
    let (rect, _) = ui.allocate_exact_size(Vec2::new(ui.available_width(), 6.0), egui::Sense::hover());
    let painter = ui.painter_at(rect);

    painter.rect_filled(rect, 3.0, BG_CARD_SUBTLE);

    let clamped_pct = (pct as f32 / 100.0).clamp(0.0, 1.0);
    let fill_width = rect.width() * clamped_pct;
    if fill_width > 0.0 {
        let fill_rect = Rect::from_min_size(rect.min, Vec2::new(fill_width, rect.height()));
        painter.rect_filled(fill_rect, 3.0, fill_col);
    }
}

fn render_clean_curve_graph(
    ui: &mut egui::Ui,
    nodes: &mut Vec<FanCurveNode>,
    accent: Color32,
    selected_idx: &mut usize,
) {
    let (rect, response) = ui.allocate_exact_size(Vec2::new(ui.available_width(), 120.0), egui::Sense::click_and_drag());
    let painter = ui.painter_at(rect);

    painter.rect_filled(rect, 8.0, BG_CARD_SUBTLE);
    painter.rect_stroke(rect, 8.0, Stroke::new(1.0, BORDER_COLOR), egui::StrokeKind::Inside);

    let grid_stroke = Stroke::new(0.5, BORDER_COLOR);
    for y_pct in [0.25, 0.5, 0.75] {
        let y = rect.max.y - (rect.height() * y_pct);
        painter.line_segment([Pos2::new(rect.min.x, y), Pos2::new(rect.max.x, y)], grid_stroke);
    }

    let node_to_screen = |node: &FanCurveNode| -> Pos2 {
        let x = rect.min.x + ((node.temp - 20.0) / 80.0) * rect.width();
        let y = rect.max.y - (node.speed / 100.0) * rect.height();
        Pos2::new(x, y)
    };

    for i in 0..nodes.len().saturating_sub(1) {
        let p1 = node_to_screen(&nodes[i]);
        let p2 = node_to_screen(&nodes[i + 1]);
        painter.line_segment([p1, p2], Stroke::new(2.0, accent));
    }

    for (i, node) in nodes.iter().enumerate() {
        let p = node_to_screen(node);
        let is_sel = *selected_idx == i;
        if is_sel {
            painter.circle_stroke(p, 8.0, Stroke::new(2.0, accent));
        }
        painter.circle_filled(p, 4.5, if is_sel { Color32::WHITE } else { accent });
    }

    if let Some(mouse_pos) = response.interact_pointer_pos() {
        if response.dragged() {
            let idx = *selected_idx.min(&mut (nodes.len() - 1));
            let new_temp = (20.0 + ((mouse_pos.x - rect.min.x) / rect.width()) * 80.0).clamp(20.0, 100.0);
            let new_speed = (((rect.max.y - mouse_pos.y) / rect.height()) * 100.0).clamp(0.0, 100.0);

            let min_temp = if idx > 0 { nodes[idx - 1].temp + 2.0 } else { 20.0 };
            let max_temp = if idx + 1 < nodes.len() { nodes[idx + 1].temp - 2.0 } else { 100.0 };

            nodes[idx].temp = new_temp.clamp(min_temp, max_temp);
            nodes[idx].speed = new_speed;
        } else if response.clicked() {
            for (i, node) in nodes.iter().enumerate() {
                let p = node_to_screen(node);
                if (p.x - mouse_pos.x).abs() < 14.0 && (p.y - mouse_pos.y).abs() < 14.0 {
                    *selected_idx = i;
                    break;
                }
            }
        }
    }
}

fn temp_to_color(temp: u32) -> Color32 {
    if temp < 55 {
        ACCENT_GREEN
    } else if temp < 75 {
        ACCENT_ORANGE
    } else {
        ACCENT_RED
    }
}

// ============================================================================
// UNIT TESTS
// ============================================================================
#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_theme_colors() {
        assert_eq!(BG_CANVAS, Color32::from_rgb(13, 17, 23));
        assert_eq!(BG_CARD, Color32::from_rgb(22, 27, 34));
    }

    #[test]
    fn test_temp_color_thresholds() {
        assert_eq!(temp_to_color(45), ACCENT_GREEN);
        assert_eq!(temp_to_color(65), ACCENT_ORANGE);
        assert_eq!(temp_to_color(85), ACCENT_RED);
    }
}
