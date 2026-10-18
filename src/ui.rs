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

    // Make main window content scrollable cleanly so it never clips or overflows on small window sizes / tile snaps
    egui::ScrollArea::vertical()
        .auto_shrink([false, false])
        .show(ui, |ui| {
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
    let avail_w = ui.available_width();
    let num_cols = if avail_w < 600.0 {
        2
    } else {
        4
    };

    if num_cols == 2 {
        ui.columns(2, |cols| {
            render_kpi_card(
                &mut cols[0],
                "CPU",
                "Core i5-13420H",
                &format!("{} ℃", app.cpu_temp),
                &format!("{:.2} GHz • {}%", app.cpu_clock_mhz as f32 / 1000.0, app.cpu_usage_pct),
                temp_to_color(app.cpu_temp),
                app.cpu_usage_pct,
            );

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
        });

        ui.add_space(8.0);

        ui.columns(2, |cols| {
            render_kpi_card(
                &mut cols[0],
                "MEMORY",
                &format!("{:.1} / {:.1} GB", app.mem_used_gb, app.mem_total_gb),
                &format!("{}%", app.mem_pct),
                "System RAM in use",
                ACCENT_BLUE,
                app.mem_pct,
            );

            render_kpi_card(
                &mut cols[1],
                "STORAGE",
                &format!("{} / {} GB", app.disk_used_gb, app.disk_total_gb),
                &format!("{}%", app.disk_pct),
                "Root partition usage",
                ACCENT_BLUE,
                app.disk_pct,
            );
        });
    } else {
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
            // Header with Title and Mode Switcher
            ui.horizontal(|ui| {
                ui.vertical(|ui| {
                    ui.label(
                        egui::RichText::new("Fan Cooling & Profiles")
                            .size(15.0)
                            .strong()
                            .color(TEXT_PRIMARY),
                    );
                    ui.label(
                        egui::RichText::new("Dynamic fan response curves & instant hardware override")
                            .size(12.0)
                            .color(TEXT_SECONDARY),
                    );
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
                                    .size(11.0)
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
                            .min_size(Vec2::new(76.0, 26.0)),
                        );
                        if btn.clicked() {
                            app.set_fan_mode(mode);
                        }
                        ui.add_space(4.0);
                    }
                });
            });

            ui.add_space(12.0);

            // Active Curve & Cooling Hardware Status Banner
            render_fan_status_banner(app, ui);

            ui.add_space(14.0);

            let avail_w = ui.available_width();
            if avail_w < 680.0 {
                // Stack vertically on narrow windows
                ui.vertical(|ui| {
                    render_fan_telemetry_column(app, ui);
                    ui.add_space(16.0);
                    ui.separator();
                    ui.add_space(12.0);
                    render_curve_editor(app, ui);
                });
            } else {
                ui.columns(2, |cols| {
                    render_fan_telemetry_column(app, &mut cols[0]);
                    render_curve_editor(app, &mut cols[1]);
                });
            }
        });
}

fn render_fan_status_banner(app: &AcerApp, ui: &mut egui::Ui) {
    let (cpu_target, gpu_target) = app.calculate_curve_targets();
    let is_custom = app.fan_mode == FanMode::Custom;

    let banner_bg = if is_custom {
        Color32::from_rgb(18, 38, 26)
    } else {
        BG_CARD_SUBTLE
    };
    let banner_stroke = if is_custom {
        Stroke::new(1.0, ACCENT_GREEN)
    } else {
        Stroke::new(1.0, BORDER_COLOR)
    };

    egui::Frame::new()
        .fill(banner_bg)
        .stroke(banner_stroke)
        .corner_radius(8)
        .inner_margin(egui::Margin::symmetric(14, 9))
        .show(ui, |ui| {
            ui.horizontal(|ui| {
                let (badge_text, badge_color) = match app.fan_mode {
                    FanMode::Custom => ("● ACTIVE CUSTOM CONTROL", ACCENT_GREEN),
                    FanMode::Automatic => ("○ AUTO FIRMWARE (EC)", ACCENT_BLUE),
                    FanMode::Maximum => ("▲ MAX OVERRIDE (100%)", ACCENT_ORANGE),
                    FanMode::Turbo => ("⚡ TURBO OVERRIDE (100%)", ACCENT_RED),
                };

                ui.label(
                    egui::RichText::new(badge_text)
                        .size(11.0)
                        .strong()
                        .color(badge_color),
                );

                ui.add_space(8.0);
                ui.label(egui::RichText::new("|").size(11.0).color(BORDER_COLOR));
                ui.add_space(8.0);

                let status_detail = if is_custom {
                    format!(
                        "Tailord actively controlling fans · Applied Target: CPU {}% ({}°C)  ·  GPU {}% ({}°C)",
                        app.cpu_fan_pct, app.cpu_temp, app.gpu_fan_pct, app.gpu_temp
                    )
                } else {
                    format!(
                        "Curve on standby · Evaluated Target: CPU {}% ({}°C)  ·  GPU {}% ({}°C)",
                        cpu_target, app.cpu_temp, gpu_target, app.gpu_temp
                    )
                };

                ui.label(
                    egui::RichText::new(status_detail)
                        .size(11.0)
                        .color(if is_custom { TEXT_PRIMARY } else { TEXT_SECONDARY }),
                );
            });
        });
}

fn render_fan_telemetry_column(app: &AcerApp, ui: &mut egui::Ui) {
    ui.vertical(|ui| {
        ui.horizontal(|ui| {
            ui.label(
                egui::RichText::new("REAL-TIME FAN TELEMETRY")
                    .size(11.0)
                    .strong()
                    .color(TEXT_DIM),
            );
            ui.with_layout(egui::Layout::right_to_left(egui::Align::Center), |ui| {
                if app.fan_mode == FanMode::Custom {
                    ui.label(
                        egui::RichText::new("HARDWARE DRIVEN")
                            .size(10.0)
                            .strong()
                            .color(ACCENT_GREEN),
                    );
                }
            });
        });

        ui.add_space(8.0);

        let (cpu_target, gpu_target) = app.calculate_curve_targets();

        render_fan_metric_row(
            ui,
            "CPU Fan",
            app.cpu_fan_rpm,
            app.cpu_fan_pct,
            ACCENT_BLUE,
            app.fan_mode,
            cpu_target,
        );

        ui.add_space(12.0);

        render_fan_metric_row(
            ui,
            "GPU Fan",
            app.gpu_fan_rpm,
            app.gpu_fan_pct,
            ACCENT_GREEN,
            app.fan_mode,
            gpu_target,
        );
    });
}

fn render_fan_metric_row(
    ui: &mut egui::Ui,
    name: &str,
    rpm: u32,
    pct: u32,
    accent: Color32,
    fan_mode: FanMode,
    target_pct: u32,
) {
    egui::Frame::new()
        .fill(BG_CARD_SUBTLE)
        .stroke(Stroke::new(1.0, BORDER_COLOR))
        .corner_radius(8)
        .inner_margin(12)
        .show(ui, |ui| {
            ui.horizontal(|ui| {
                ui.label(egui::RichText::new(name).size(13.0).strong().color(TEXT_PRIMARY));
                ui.with_layout(egui::Layout::right_to_left(egui::Align::Center), |ui| {
                    ui.label(
                        egui::RichText::new(format!("{} %", pct))
                            .size(13.0)
                            .strong()
                            .color(accent),
                    );
                    ui.add_space(8.0);
                    ui.label(
                        egui::RichText::new(format!("{} RPM", rpm))
                            .size(13.0)
                            .color(TEXT_SECONDARY),
                    );
                    ui.add_space(8.0);
                    let mode_badge = match fan_mode {
                        FanMode::Custom => format!("Target: {}% (Curve)", target_pct),
                        FanMode::Automatic => "EC Auto".to_string(),
                        FanMode::Maximum => "Max (100%)".to_string(),
                        FanMode::Turbo => "Turbo (100%)".to_string(),
                    };
                    ui.label(egui::RichText::new(mode_badge).size(10.0).color(TEXT_DIM));
                });
            });
            ui.add_space(6.0);
            render_clean_progress_bar(ui, pct, accent);
        });
}

fn render_curve_editor(app: &mut AcerApp, ui: &mut egui::Ui) {
    // Ensure both curves have exactly 2 points
    if app.cpu_curve_nodes.len() != 2 {
        app.cpu_curve_nodes = crate::backend::default_cpu_curve();
    }
    if app.gpu_curve_nodes.len() != 2 {
        app.gpu_curve_nodes = crate::backend::default_gpu_curve();
    }
    app.selected_node_idx = app.selected_node_idx.min(1);

    ui.vertical(|ui| {
        // Tab switcher
        ui.horizontal(|ui| {
            ui.label(
                egui::RichText::new("CURVE CONFIGURATION (2 THRESHOLD POINTS)")
                    .size(11.0)
                    .strong()
                    .color(TEXT_DIM),
            );
            ui.with_layout(egui::Layout::right_to_left(egui::Align::Center), |ui| {
                let is_cpu = app.active_curve_tab == 0;
                if ui.selectable_label(!is_cpu, "GPU Curve").clicked() {
                    app.active_curve_tab = 1;
                    app.selected_node_idx = 0;
                }
                if ui.selectable_label(is_cpu, "CPU Curve").clicked() {
                    app.active_curve_tab = 0;
                    app.selected_node_idx = 0;
                }
            });
        });

        ui.add_space(8.0);

        let is_cpu = app.active_curve_tab == 0;
        let accent = if is_cpu { ACCENT_BLUE } else { ACCENT_GREEN };
        let live_temp = if is_cpu { app.cpu_temp } else { app.gpu_temp };
        let live_duty = if is_cpu { app.cpu_fan_pct } else { app.gpu_fan_pct };

        // Render Graph Canvas
        let (_cur_temp, _cur_speed, changed_by_canvas) = {
            let nodes = if is_cpu {
                &mut app.cpu_curve_nodes
            } else {
                &mut app.gpu_curve_nodes
            };

            let changed = render_clean_curve_graph(
                ui,
                nodes,
                accent,
                &mut app.selected_node_idx,
                live_temp,
                live_duty,
            );
            let idx = app.selected_node_idx.min(1);
            (nodes[idx].temp as u32, nodes[idx].speed as u32, changed)
        };

        ui.add_space(8.0);

        // Status row below graph: 2 Points overview + Current active stage
        let (cpu_target, gpu_target) = app.calculate_curve_targets();
        let active_applied = if is_cpu { cpu_target } else { gpu_target };
        let is_custom_active = app.fan_mode == FanMode::Custom;

        let (t1, s1, t2, s2) = {
            let nodes = if is_cpu { &app.cpu_curve_nodes } else { &app.gpu_curve_nodes };
            (nodes[0].temp as u32, nodes[0].speed as u32, nodes[1].temp as u32, nodes[1].speed as u32)
        };

        ui.horizontal(|ui| {
            // Point selector chips
            if ui.selectable_label(app.selected_node_idx == 0, format!("Point 1: {} ℃ ➔ {} %", t1, s1)).clicked() {
                app.selected_node_idx = 0;
            }
            ui.add_space(4.0);
            if ui.selectable_label(app.selected_node_idx == 1, format!("Point 2: {} ℃ ➔ {} %", t2, s2)).clicked() {
                app.selected_node_idx = 1;
            }

            ui.add_space(8.0);
            ui.label(egui::RichText::new("·").color(BORDER_COLOR));
            ui.add_space(8.0);

            // Active stage badge
            let stage_desc = if live_temp < t1 {
                format!("Stage: Idle / Quiet (< {}℃)", t1)
            } else if live_temp < t2 {
                format!("Stage: Threshold 1 Active ({}%)", s1)
            } else {
                format!("Stage: Threshold 2 Active ({}%)", s2)
            };

            let badge_color = if is_custom_active { ACCENT_GREEN } else { TEXT_SECONDARY };
            egui::Frame::new()
                .fill(if is_custom_active { Color32::from_rgb(18, 38, 26) } else { BG_CARD_SUBTLE })
                .stroke(Stroke::new(1.0, badge_color))
                .corner_radius(4)
                .inner_margin(egui::Margin::symmetric(8, 3))
                .show(ui, |ui| {
                    ui.label(
                        egui::RichText::new(format!("{} · Applied: {}%", stage_desc, active_applied))
                            .size(10.5)
                            .strong()
                            .color(badge_color),
                    );
                });
        });

        ui.add_space(6.0);

        // Fine-tune sliders for selected node
        let mut fine_tune_changed = false;
        {
            let nodes = if is_cpu {
                &mut app.cpu_curve_nodes
            } else {
                &mut app.gpu_curve_nodes
            };
            let idx = app.selected_node_idx.min(1);

            ui.horizontal(|ui| {
                ui.label(
                    egui::RichText::new(format!("Tuning Point {}:", idx + 1))
                        .size(11.0)
                        .strong()
                        .color(TEXT_PRIMARY),
                );

                let (min_t, max_t) = if idx == 0 {
                    (35.0, nodes[1].temp - 2.0)
                } else {
                    (nodes[0].temp + 2.0, 95.0)
                };

                let (min_s, max_s) = if idx == 0 {
                    (10.0, nodes[1].speed)
                } else {
                    (nodes[0].speed, 100.0)
                };

                ui.label(egui::RichText::new("Temp").size(11.0).color(TEXT_SECONDARY));
                let mut temp_val = nodes[idx].temp;
                let t_slider = ui.add(
                    egui::Slider::new(&mut temp_val, min_t..=max_t)
                        .suffix(" °C")
                        .show_value(true),
                );
                if t_slider.changed() {
                    nodes[idx].temp = temp_val.round();
                    fine_tune_changed = true;
                }

                ui.add_space(8.0);
                ui.label(egui::RichText::new("Fan Speed").size(11.0).color(TEXT_SECONDARY));
                let mut speed_val = nodes[idx].speed;
                let s_slider = ui.add(
                    egui::Slider::new(&mut speed_val, min_s..=max_s)
                        .suffix(" %")
                        .show_value(true),
                );
                if s_slider.changed() {
                    nodes[idx].speed = speed_val.round();
                    fine_tune_changed = true;
                }
            });
        }

        ui.add_space(8.0);

        // Action Buttons Row: "Sync to Tailord / Save" + "Reset Curve"
        ui.horizontal(|ui| {
            let sync_btn = ui.add(
                egui::Button::new(
                    egui::RichText::new("Sync to Tailord / Save")
                        .size(11.5)
                        .strong()
                        .color(Color32::WHITE),
                )
                .fill(ACCENT_BLUE)
                .corner_radius(6)
                .min_size(Vec2::new(144.0, 26.0)),
            );
            if sync_btn.clicked() {
                app.sync_custom_curve_to_tailord();
            }

            ui.add_space(6.0);

            let reset_btn = ui.add(
                egui::Button::new(
                    egui::RichText::new("Reset Default (2 Points)")
                        .size(11.5)
                        .color(TEXT_SECONDARY),
                )
                .fill(BG_CARD_SUBTLE)
                .stroke(Stroke::new(1.0, BORDER_COLOR))
                .corner_radius(6)
                .min_size(Vec2::new(140.0, 26.0)),
            );
            if reset_btn.clicked() {
                app.reset_active_curve();
            }
        });

        // Reactive hardware evaluation if canvas or slider changed
        if changed_by_canvas || fine_tune_changed {
            app.apply_custom_curve();
        }

        // Feedback notification message banner
        if let Some(ref msg) = app.curve_feedback {
            let show = if let Some(time) = app.curve_feedback_time {
                time.elapsed().as_secs() < 8
            } else {
                true
            };

            if show {
                ui.add_space(6.0);
                egui::Frame::new()
                    .fill(Color32::from_rgb(16, 32, 22))
                    .stroke(Stroke::new(1.0, ACCENT_GREEN))
                    .corner_radius(6)
                    .inner_margin(egui::Margin::symmetric(10, 5))
                    .show(ui, |ui| {
                        ui.horizontal(|ui| {
                            ui.label(
                                egui::RichText::new(msg)
                                    .size(11.0)
                                    .color(ACCENT_GREEN),
                            );
                        });
                    });
            }
        }
    });
}

// ============================================================================
// 4. BOTTOM SECTION: QUICK PREFERENCES & SYSTEM INFO
// ============================================================================
fn render_bottom_section(app: &mut AcerApp, ui: &mut egui::Ui) {
    let avail_w = ui.available_width();
    if avail_w < 680.0 {
        ui.vertical(|ui| {
            render_preferences_card(app, ui);
            ui.add_space(8.0);
            render_display_card(app, ui);
        });
    } else {
        ui.columns(2, |cols| {
            render_preferences_card(app, &mut cols[0]);
            render_display_card(app, &mut cols[1]);
        });
    }
}

fn render_preferences_card(app: &mut AcerApp, ui: &mut egui::Ui) {
    egui::Frame::new()
        .fill(BG_CARD)
        .stroke(Stroke::new(1.0, BORDER_COLOR))
        .corner_radius(12)
        .inner_margin(16)
        .show(ui, |ui| {
            ui.label(egui::RichText::new("Hardware Preferences").size(14.0).strong().color(TEXT_PRIMARY));
            ui.label(egui::RichText::new("Quick toggles for keyboard and peripherals").size(11.0).color(TEXT_SECONDARY));

            ui.add_space(12.0);

            ui.horizontal_wrapped(|ui| {
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
}

fn render_display_card(app: &mut AcerApp, ui: &mut egui::Ui) {
    egui::Frame::new()
        .fill(BG_CARD)
        .stroke(Stroke::new(1.0, BORDER_COLOR))
        .corner_radius(12)
        .inner_margin(16)
        .show(ui, |ui| {
            ui.label(egui::RichText::new("Display & GPU Switching").size(14.0).strong().color(TEXT_PRIMARY));
            ui.label(egui::RichText::new("Graphics power routing and blue light reduction").size(11.0).color(TEXT_SECONDARY));

            ui.add_space(10.0);

            ui.horizontal(|ui| {
                let _prev_filter = app.blue_light_filter;
                let _prev_level = app.blue_light_level;

                if ui.checkbox(&mut app.blue_light_filter, "Night Filter").changed() {
                    app.controller.set_night_light(app.blue_light_filter, app.blue_light_level);
                }

                if app.blue_light_filter
                    && ui.add(egui::Slider::new(&mut app.blue_light_level, 0.0..=10.0).text("Level")).changed() {
                        app.controller.set_night_light(true, app.blue_light_level);
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
    nodes: &mut [FanCurveNode],
    accent: Color32,
    selected_idx: &mut usize,
    live_temp: u32,
    live_duty: u32,
) -> bool {
    let (rect, response) = ui.allocate_exact_size(
        Vec2::new(ui.available_width(), 130.0),
        egui::Sense::click_and_drag(),
    );
    let painter = ui.painter_at(rect);

    painter.rect_filled(rect, 8.0, BG_CARD_SUBTLE);
    painter.rect_stroke(rect, 8.0, Stroke::new(1.0, BORDER_COLOR), egui::StrokeKind::Inside);

    // Horizontal grid lines (25%, 50%, 75%)
    let grid_stroke = Stroke::new(0.5, Color32::from_rgba_unmultiplied(48, 54, 61, 160));
    for y_pct in [0.25, 0.5, 0.75] {
        let y = rect.max.y - (rect.height() * y_pct);
        painter.line_segment([Pos2::new(rect.min.x, y), Pos2::new(rect.max.x, y)], grid_stroke);
    }

    // Vertical temperature guides (40°C, 60°C, 80°C)
    for t in [40.0, 60.0, 80.0] {
        let x = rect.min.x + ((t - 20.0) / 80.0) * rect.width();
        painter.line_segment([Pos2::new(x, rect.min.y), Pos2::new(x, rect.max.y)], grid_stroke);
    }

    let temp_to_x = |temp: f32| -> f32 {
        rect.min.x + ((temp - 20.0) / 80.0) * rect.width()
    };
    let speed_to_y = |speed: f32| -> f32 {
        rect.max.y - (speed / 100.0) * rect.height()
    };

    if nodes.len() >= 2 {
        let t1 = nodes[0].temp;
        let s1 = nodes[0].speed;
        let t2 = nodes[1].temp;
        let s2 = nodes[1].speed;
        let base_s = (s1 / 2.0).min(25.0);

        let p_start = Pos2::new(rect.min.x, speed_to_y(base_s));
        let p_t1_base = Pos2::new(temp_to_x(t1), speed_to_y(base_s));
        let p_t1_top = Pos2::new(temp_to_x(t1), speed_to_y(s1));
        let p_t2_mid = Pos2::new(temp_to_x(t2), speed_to_y(s1));
        let p_t2_top = Pos2::new(temp_to_x(t2), speed_to_y(s2));
        let p_end = Pos2::new(rect.max.x, speed_to_y(s2));

        // Shaded polygon below stepped staircase
        let area_points = vec![
            Pos2::new(rect.min.x, rect.max.y),
            p_start,
            p_t1_base,
            p_t1_top,
            p_t2_mid,
            p_t2_top,
            p_end,
            Pos2::new(rect.max.x, rect.max.y),
        ];
        let fill_color = Color32::from_rgba_unmultiplied(accent.r(), accent.g(), accent.b(), 22);
        painter.add(egui::Shape::convex_polygon(area_points, fill_color, Stroke::NONE));

        // Stepped line segments (tidak naik turun per derajat, hanya berubah pas di titik)
        let line_stroke = Stroke::new(2.5, accent);
        painter.line_segment([p_start, p_t1_base], line_stroke);
        painter.line_segment([p_t1_base, p_t1_top], line_stroke);
        painter.line_segment([p_t1_top, p_t2_mid], line_stroke);
        painter.line_segment([p_t2_mid, p_t2_top], line_stroke);
        painter.line_segment([p_t2_top, p_end], line_stroke);

        // Labels for thresholds
        painter.text(
            Pos2::new(p_t1_top.x, p_t1_top.y - 12.0),
            egui::Align2::CENTER_CENTER,
            format!("Point 1: {}°C", t1 as u32),
            egui::FontId::proportional(10.0),
            TEXT_SECONDARY,
        );
        painter.text(
            Pos2::new(p_t2_top.x, p_t2_top.y - 12.0),
            egui::Align2::CENTER_CENTER,
            format!("Point 2: {}°C", t2 as u32),
            egui::FontId::proportional(10.0),
            TEXT_SECONDARY,
        );
    }

    // Draw live operating point indicator if within range
    if (20..=100).contains(&live_temp) {
        let live_x = temp_to_x(live_temp as f32);
        let live_y = speed_to_y(live_duty.min(100) as f32);

        // Vertical indicator line
        painter.line_segment(
            [Pos2::new(live_x, rect.min.y), Pos2::new(live_x, rect.max.y)],
            Stroke::new(1.0, Color32::from_rgba_unmultiplied(210, 153, 34, 120)),
        );

        // Operating point marker
        painter.circle_stroke(
            Pos2::new(live_x, live_y),
            7.0,
            Stroke::new(1.5, ACCENT_ORANGE),
        );
        painter.circle_filled(Pos2::new(live_x, live_y), 3.5, Color32::WHITE);
    }

    // Draw node control points (only 2 points)
    for (i, node) in nodes.iter().enumerate() {
        let p = Pos2::new(temp_to_x(node.temp), speed_to_y(node.speed));
        let is_sel = *selected_idx == i;
        if is_sel {
            painter.circle_stroke(p, 9.0, Stroke::new(2.0, accent));
        }
        painter.circle_filled(
            p,
            if is_sel { 5.5 } else { 4.0 },
            if is_sel { Color32::WHITE } else { accent },
        );
    }

    let mut modified = false;

    if let Some(mouse_pos) = response.interact_pointer_pos() {
        // When drag starts or on click, select the closest node
        if response.drag_started() || (response.clicked() && !response.dragged()) {
            let mut closest_idx = *selected_idx;
            let mut min_dist = f32::MAX;
            for (i, node) in nodes.iter().enumerate() {
                let p = Pos2::new(temp_to_x(node.temp), speed_to_y(node.speed));
                let dist = (p.x - mouse_pos.x).hypot(p.y - mouse_pos.y);
                if dist < min_dist {
                    min_dist = dist;
                    closest_idx = i;
                }
            }
            if min_dist < 40.0 || response.clicked() {
                *selected_idx = closest_idx;
            }
        }

        if response.dragged() && nodes.len() >= 2 {
            let idx = (*selected_idx).min(1);
            let new_temp = (20.0 + ((mouse_pos.x - rect.min.x) / rect.width()) * 80.0).clamp(20.0, 100.0);
            let new_speed = (((rect.max.y - mouse_pos.y) / rect.height()) * 100.0).clamp(0.0, 100.0);

            let (min_temp, max_temp) = if idx == 0 {
                (35.0, nodes[1].temp - 2.0)
            } else {
                (nodes[0].temp + 2.0, 95.0)
            };

            let (min_speed, max_speed) = if idx == 0 {
                (10.0, nodes[1].speed)
            } else {
                (nodes[0].speed, 100.0)
            };

            let rounded_temp = new_temp.clamp(min_temp, max_temp).round();
            let rounded_speed = new_speed.clamp(min_speed, max_speed).round();

            if (nodes[idx].temp - rounded_temp).abs() > 0.01 || (nodes[idx].speed - rounded_speed).abs() > 0.01 {
                nodes[idx].temp = rounded_temp;
                nodes[idx].speed = rounded_speed;
                modified = true;
            }
        }

        if response.drag_stopped() {
            modified = true;
        }
    }

    modified
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

    #[test]
    fn test_app_ui_state_defaults() {
        let app = AcerApp::default();
        assert_eq!(app.active_tab, 0);
        assert_eq!(app.power_mode, PowerMode::Performance);
        assert_eq!(app.fan_mode, FanMode::Automatic);
        assert_eq!(app.gpu_switch, GpuSwitchMode::Discrete);
        assert_eq!(app.cpu_curve_nodes.len(), 2);
        assert_eq!(app.gpu_curve_nodes.len(), 2);
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
    fn test_boundary_clamping_first_middle_last() {
        let nodes = vec![
            FanCurveNode::new(25.0, 20.0),
            FanCurveNode::new(50.0, 40.0),
            FanCurveNode::new(75.0, 70.0),
            FanCurveNode::new(95.0, 100.0),
        ];

        // 1. Node 0 clamping
        let idx = 0;
        let min_t = 20.0;
        let max_t = nodes[idx + 1].temp - 1.0;
        assert_eq!(10.0f32.clamp(min_t, max_t), 20.0);
        assert_eq!(60.0f32.clamp(min_t, max_t), 49.0);

        // 2. Middle node clamping
        let idx = 2;
        let min_t = nodes[idx - 1].temp + 1.0;
        let max_t = nodes[idx + 1].temp - 1.0;
        assert_eq!(40.0f32.clamp(min_t, max_t), 51.0);
        assert_eq!(98.0f32.clamp(min_t, max_t), 94.0);

        // 3. Last node clamping
        let idx = nodes.len() - 1;
        let min_t = nodes[idx - 1].temp + 1.0;
        let max_t = 100.0;
        assert_eq!(70.0f32.clamp(min_t, max_t), 76.0);
        assert_eq!(110.0f32.clamp(min_t, max_t), 100.0);
    }

    #[test]
    fn test_sync_custom_curve_to_tailord() {
        let prev_active = std::process::Command::new("busctl")
            .args(["call", "com.tux.Tailor", "/com/tux/Tailor", "com.tux.Tailor.Profiles", "GetActiveProfileName"])
            .output()
            .ok()
            .and_then(|o| crate::backend::parse_busctl_string(&String::from_utf8_lossy(&o.stdout)))
            .unwrap_or_else(|| "default".to_string());

        let mut app = AcerApp::default();
        app.fan_mode = FanMode::Automatic;
        app.cpu_temp = 50;
        app.gpu_temp = 45;

        app.sync_custom_curve_to_tailord();

        assert_eq!(app.fan_mode, FanMode::Custom);
        assert!(app.curve_feedback.is_some());
        let fb = app.curve_feedback.as_ref().unwrap();
        assert!(fb.contains("Locked in & synced to tailord") || fb.contains("Saved to disk"));
        assert!(app.cpu_fan_pct > 0);
        assert_eq!(app.cpu_fan_pct, app.last_applied_cpu_pct);

        let _ = std::process::Command::new("busctl")
            .args(["call", "com.tux.Tailor", "/com/tux/Tailor", "com.tux.Tailor.Profiles", "SetActiveProfileName", "s", &prev_active])
            .output();
    }

    #[test]
    fn test_apply_custom_curve_live() {
        let mut app = AcerApp::default();
        app.fan_mode = FanMode::Custom;
        app.cpu_temp = 75;
        app.gpu_temp = 60;

        // Point 2 at 75C set to 90%
        app.cpu_curve_nodes[1].speed = 90.0;

        app.apply_custom_curve();
        assert_eq!(app.cpu_fan_pct, 90);
        assert!(app.curve_feedback.is_some());
        let fb = app.curve_feedback.as_ref().unwrap();
        assert!(fb.contains("Live applied"));
    }

    #[test]
    fn test_reset_active_curve() {
        let mut app = AcerApp::default();
        app.active_curve_tab = 0;
        app.cpu_curve_nodes[0].speed = 99.0;

        app.reset_active_curve();
        assert_eq!(app.cpu_curve_nodes[0].speed, 60.0);
        assert_eq!(app.cpu_curve_nodes.len(), 2);
        assert!(app.curve_feedback.is_some());

        // Test GPU curve reset
        app.active_curve_tab = 1;
        app.gpu_curve_nodes[0].speed = 88.0;
        app.reset_active_curve();
        assert_eq!(app.gpu_curve_nodes[0].speed, 50.0);
        assert_eq!(app.gpu_curve_nodes.len(), 2);
    }

    #[test]
    fn test_fan_mode_feedback_messages() {
        let mut app = AcerApp::default();

        app.set_fan_mode(FanMode::Automatic);
        assert!(app.curve_feedback.as_ref().unwrap().contains("Auto Curve"));

        app.set_fan_mode(FanMode::Maximum);
        assert!(app.curve_feedback.as_ref().unwrap().contains("Max fan mode"));

        app.set_fan_mode(FanMode::Turbo);
        assert!(app.curve_feedback.as_ref().unwrap().contains("Turbo fan mode"));

        app.set_fan_mode(FanMode::Custom);
        assert!(app.curve_feedback.as_ref().unwrap().contains("Custom Curve"));
    }

    #[test]
    fn test_telemetry_temp_shift_updates_curve_targets() {
        let mut app = AcerApp::default();
        app.fan_mode = FanMode::Custom;

        app.cpu_temp = 30;
        app.apply_custom_curve();
        let cool_pct = app.cpu_fan_pct;

        app.cpu_temp = 85;
        app.apply_custom_curve();
        let hot_pct = app.cpu_fan_pct;

        assert!(hot_pct > cool_pct);
    }
}
