pub mod tab_buttons;
pub mod tab_debug;
pub mod tab_ring;
pub mod tab_settings;
pub mod types;
pub mod widgets;

use std::time::Duration;

use crate::autostart;
use crate::config::{ButtonActionConfig, Config};
use crate::devices::{
    get_button_name_lang, is_supported_button_code, BTN_LEFT_CODE, BTN_RIGHT_CODE,
    ConnectionType,
};
use crate::i18n::tr;
use eframe::egui;

pub use types::{ConfigApp, ConfigTab, EditorDetectTarget, SelectedNodePath, ShortcutTarget};
use widgets::key_to_string;

impl eframe::App for ConfigApp {
    fn update(&mut self, ctx: &egui::Context, _frame: &mut eframe::Frame) {
        let lang = self.lang();
        ctx.request_repaint_after(Duration::from_millis(50));

        if let Some(ref rx) = self.scan_rx {
            if let Ok(info) = rx.try_recv() {
                if let Some(ref dev_info) = info {
                    if self.config.general.device_profile == "auto" {
                        if let Some(ref detected_id) = dev_info.detected_profile_id {
                            if detected_id != &self.active_device_profile.id {
                                self.select_device_profile(ctx, detected_id);
                            }
                        }
                    }
                }
                self.mouse_info = info;
                self.status_message = tr(&lang, "status_info_updated").to_string();
            }
        }

        if self.last_auto_scan.elapsed() > Duration::from_secs(2) {
            self.last_auto_scan = std::time::Instant::now();
            self.trigger_async_mouse_scan(ctx.clone());
        }

        // 1. Process keyboard shortcut recording if active
        if let Some(target) = self.recording_shortcut_for.clone() {
            let mut detected_combo = None;

            ctx.input(|i| {
                let modifiers = &i.modifiers;
                for key in &i.keys_down {
                    let key_str = key_to_string(*key);
                    // Ignore if pressed key is a modifier key alone
                    if key_str != "Ctrl"
                        && key_str != "Alt"
                        && key_str != "Shift"
                        && key_str != "Super"
                        && key_str != "Escape"
                    {
                        let mut keys = Vec::new();
                        if modifiers.ctrl {
                            keys.push("CTRL".to_string());
                        }
                        if modifiers.alt {
                            keys.push("ALT".to_string());
                        }
                        if modifiers.shift {
                            keys.push("SHIFT".to_string());
                        }
                        if modifiers.command || modifiers.mac_cmd {
                            keys.push("Super".to_string());
                        }
                        keys.push(key_str);
                        detected_combo = Some(keys);
                        break;
                    } else if key_str == "Escape" {
                        // Cancel on Escape
                        detected_combo = Some(vec![]);
                        break;
                    }
                }
            });

            if let Some(keys) = detected_combo {
                if !keys.is_empty() {
                    let combo_action = ButtonActionConfig::KeyCombo {
                        keys: keys.clone(),
                    };
                    match target {
                        ShortcutTarget::Button(code) => {
                            if code != BTN_LEFT_CODE && code != BTN_RIGHT_CODE {
                                self.config.buttons.insert(code, combo_action);
                            }
                        }
                        ShortcutTarget::RingNode(SelectedNodePath::Slot(s)) => {
                            if let Some(slot) = self.active_ring_menu_mut().items.get_mut(s) {
                                slot.action = Some(combo_action);
                            }
                        }
                        ShortcutTarget::RingNode(SelectedNodePath::SubItem(s, s1)) => {
                            if let Some(slot) = self.active_ring_menu_mut().items.get_mut(s) {
                                if let Some(sub) = slot.items.get_mut(s1) {
                                    sub.action = Some(combo_action);
                                }
                            }
                        }
                        ShortcutTarget::RingNode(SelectedNodePath::NestedSubItem(s, s1, s2)) => {
                            if let Some(slot) = self.active_ring_menu_mut().items.get_mut(s) {
                                if let Some(sub1) = slot.items.get_mut(s1) {
                                    if let Some(sub2) = sub1.items.get_mut(s2) {
                                        sub2.action = Some(combo_action);
                                    }
                                }
                            }
                        }
                    }
                    self.notify_success(
                        tr(&lang, "status_shortcut_saved").replace("{}", &keys.join(" + ")),
                    );
                } else {
                    self.status_message = tr(&lang, "status_shortcut_canceled").to_string();
                }
                self.recording_shortcut_for = None;
            }
        }

        // 2. Listen to physical mouse button presses via evdev
        let mut incoming_events = Vec::new();
        if let Some(rx) = &self.event_rx {
            while let Ok(event) = rx.try_recv() {
                incoming_events.push(event);
            }
        }

        for ev in incoming_events {
            let code = ev.code;
            if let Some(target) = self.editor_detecting {
                match target {
                    EditorDetectTarget::NewButton => {
                        if code != BTN_LEFT_CODE && code != BTN_RIGHT_CODE {
                            self.new_btn_code = code;
                            self.editor_detecting = None;
                            self.notify_success(
                                tr(&lang, "status_code_detected").replace("{}", &code.to_string()),
                            );
                        }
                    }
                    EditorDetectTarget::EditModalButton => {
                        if code != BTN_LEFT_CODE && code != BTN_RIGHT_CODE {
                            self.edit_btn_code = code;
                            self.editor_detecting = None;
                            self.notify_success(
                                tr(&lang, "status_code_detected").replace("{}", &code.to_string()),
                            );
                        }
                    }
                    EditorDetectTarget::NewDeviceHardware => {
                        if let Some(vp) = ev.vid_pid.clone() {
                            if !self.new_device_match_ids.contains(&vp) {
                                self.new_device_match_ids.push(vp.clone());
                            }
                            if let Some(name) = ev.dev_name.clone() {
                                if self.new_device_name.trim().is_empty() {
                                    self.new_device_name = name.clone();
                                }
                                if !self.new_device_id_edited && self.new_device_id.trim().is_empty() {
                                    self.new_device_id = crate::config::slugify(&name);
                                }
                            }
                            self.editor_detecting = None;
                            self.notify_success(
                                tr(&lang, "status_hardware_detected").replace("{}", &vp),
                            );
                        } else {
                            self.notify_error(tr(&lang, "status_hardware_detect_failed"));
                        }
                    }
                    EditorDetectTarget::CurrentDeviceHardware => {
                        if let Some(vp) = ev.vid_pid.clone() {
                            if !self.active_device_profile.match_ids.contains(&vp) {
                                self.active_device_profile.match_ids.push(vp.clone());
                                if let Err(e) = self.device_registry.save_profile(&self.active_device_profile) {
                                    eprintln!("Failed to save profile: {}", e);
                                }
                                self.notify_success(
                                    tr(&lang, "status_hardware_detected").replace("{}", &vp),
                                );
                            } else {
                                self.notify_success(
                                    tr(&lang, "status_hardware_already_added").replace("{}", &vp),
                                );
                            }
                            self.editor_detecting = None;
                        } else {
                            self.notify_error(tr(&lang, "status_hardware_detect_failed"));
                        }
                    }
                }
            } else if is_supported_button_code(code) {
                self.selected_button_code = code;
                self.status_message = tr(&lang, "status_button_detected").replace(
                    "{}",
                    &format!("{} (Code {})", get_button_name_lang(code, &lang), code),
                );
            }
        }

        let mut visuals = egui::Visuals::dark();
        visuals.panel_fill = egui::Color32::from_rgb(18, 19, 24);
        visuals.window_fill = egui::Color32::from_rgb(18, 19, 24);
        visuals.widgets.noninteractive.bg_fill = egui::Color32::from_rgb(26, 28, 36);
        visuals.selection.bg_fill = egui::Color32::from_rgb(0, 170, 140);
        visuals.selection.stroke = egui::Stroke::new(1.0_f32, egui::Color32::from_rgb(0, 215, 175));
        ctx.set_visuals(visuals);

        egui::TopBottomPanel::top("top_header")
            .frame(
                egui::Frame::none()
                    .fill(egui::Color32::from_rgb(14, 15, 20))
                    .stroke(egui::Stroke::new(1.0_f32, egui::Color32::from_rgb(32, 35, 48)))
                    .inner_margin(egui::Margin::symmetric(16.0, 10.0)),
            )
            .show(ctx, |ui| {
                ui.with_layout(egui::Layout::left_to_right(egui::Align::Center), |ui| {
                    // Logo & Brand
                    ui.horizontal(|ui| {
                        let (logo_rect, _) = ui.allocate_exact_size(egui::vec2(28.0, 28.0), egui::Sense::hover());
                        ui.painter().circle_filled(logo_rect.center(), 14.0, egui::Color32::from_rgb(24, 38, 44));
                        ui.painter().circle_stroke(logo_rect.center(), 14.0, egui::Stroke::new(1.2_f32, egui::Color32::from_rgb(0, 215, 175)));
                        ui.painter().text(
                            logo_rect.center(),
                            egui::Align2::CENTER_CENTER,
                            "🖱️",
                            egui::FontId::proportional(14.0),
                            egui::Color32::WHITE,
                        );

                        ui.add_space(4.0);
                        ui.label(
                            egui::RichText::new("Bo-Ring")
                                .strong()
                                .size(17.0)
                                .color(egui::Color32::WHITE),
                        );
                    });

                    ui.add_space(10.0);

                    // Separator
                    let (sep_rect, _) = ui.allocate_exact_size(egui::vec2(1.0, 22.0), egui::Sense::hover());
                    ui.painter().vline(
                        sep_rect.center().x,
                        sep_rect.top()..=sep_rect.bottom(),
                        egui::Stroke::new(1.0_f32, egui::Color32::from_rgb(45, 48, 60)),
                    );

                    ui.add_space(10.0);

                    let lang = self.lang();

                    // Modern Header navigation tabs
                    let mut tabs = vec![
                        (ConfigTab::ButtonConfig, tr(&lang, "tab_button_config")),
                        (ConfigTab::RingCustomizer, tr(&lang, "tab_ring_customizer")),
                        (ConfigTab::GlobalSettings, tr(&lang, "tab_settings")),
                    ];

                    if self.config.general.dev_mode {
                        tabs.push((ConfigTab::Debug, tr(&lang, "tab_debug")));
                    }

                    for (tab, label) in tabs {
                        let is_active = self.active_tab == tab;
                        let (bg, text_color, stroke) = if is_active {
                            (
                                egui::Color32::from_rgb(0, 170, 140),
                                egui::Color32::WHITE,
                                egui::Stroke::new(1.2_f32, egui::Color32::from_rgb(0, 215, 175)),
                            )
                        } else {
                            (
                                egui::Color32::from_rgb(24, 26, 34),
                                egui::Color32::from_gray(170),
                                egui::Stroke::new(1.0_f32, egui::Color32::from_rgb(40, 44, 58)),
                            )
                        };

                        let tab_btn = egui::Button::new(
                            egui::RichText::new(label)
                                .strong()
                                .size(12.5)
                                .color(text_color),
                        )
                        .fill(bg)
                        .stroke(stroke)
                        .rounding(egui::Rounding::same(8.0))
                        .min_size(egui::vec2(100.0, 32.0));

                        if ui.add(tab_btn).clicked() {
                            self.active_tab = tab;
                        }
                        ui.add_space(4.0);
                    }

                    ui.with_layout(egui::Layout::right_to_left(egui::Align::Center), |ui| {
                        // 💾 Save & Apply Button
                        let save_btn = egui::Button::new(
                            egui::RichText::new(format!("💾 {}", tr(&lang, "save_and_apply")))
                                .strong()
                                .size(13.0)
                                .color(egui::Color32::WHITE),
                        )
                        .fill(egui::Color32::from_rgb(0, 170, 140))
                        .stroke(egui::Stroke::new(1.0_f32, egui::Color32::from_rgb(0, 215, 175)))
                        .rounding(egui::Rounding::same(8.0))
                        .min_size(egui::vec2(120.0, 32.0));

                        if ui.add(save_btn).clicked() {
                            if let Err(e) = self.save_config() {
                                self.notify_error(tr(&lang, "status_save_error").replace("{}", &e.to_string()));
                            } else {
                                autostart::restart_systemd_service_if_active();
                                self.notify_success(tr(&lang, "save_success").to_string());
                            }
                        }

                        ui.add_space(10.0);

                        // Mouse status pill

                        // Mouse status pill
                        if let Some(info) = &self.mouse_info {
                            let (conn_icon, conn_text) = match info.connection_type {
                                ConnectionType::UsbReceiver => ("📡", tr(&lang, "conn_usb_receiver")),
                                ConnectionType::UsbCable => ("🔌", tr(&lang, "conn_usb_cable")),
                                ConnectionType::Bluetooth => ("📶", tr(&lang, "conn_bluetooth")),
                            };

                            let (bat_icon, text_color) = if info.is_charging {
                                ("⚡ 🔋", egui::Color32::from_rgb(255, 215, 0))
                            } else if info.battery_level >= 50 {
                                ("🔋", egui::Color32::from_rgb(0, 215, 175))
                            } else if info.battery_level >= 20 {
                                ("🪫", egui::Color32::from_rgb(255, 190, 40))
                            } else {
                                ("🪫", egui::Color32::from_rgb(255, 90, 90))
                            };

                            let charging_str = tr(&lang, "charging_status");
                            let discharging_str = tr(&lang, "discharging_status");

                            let label_text = if info.is_charging {
                                format!(
                                    "🟢 {} | {} {} ({} {}% {})",
                                    info.name, conn_icon, conn_text, bat_icon, info.battery_level, charging_str
                                )
                            } else {
                                format!(
                                    "🟢 {} | {} {} ({} {}%)",
                                    info.name, conn_icon, conn_text, bat_icon, info.battery_level
                                )
                            };

                            let status_btn = egui::Button::new(
                                egui::RichText::new(label_text)
                                    .small()
                                    .strong()
                                    .color(text_color),
                            )
                            .fill(egui::Color32::from_rgb(26, 28, 36))
                            .stroke(egui::Stroke::new(1.0_f32, egui::Color32::from_rgb(45, 50, 65)))
                            .rounding(egui::Rounding::same(14.0))
                            .min_size(egui::vec2(0.0, 28.0));

                            let current_status_str = if info.is_charging {
                                format!("⚡ {}", charging_str)
                            } else {
                                discharging_str.to_string()
                            };

                            let response = ui.add(status_btn).on_hover_text(format!(
                                "{}: {}\n{}: {} {}\n{}: {}%\n{}: {}\n{}: {}\n{}",
                                tr(&lang, "tooltip_device"),
                                info.name,
                                tr(&lang, "tooltip_connection"),
                                conn_icon,
                                conn_text,
                                tr(&lang, "tooltip_battery"),
                                info.battery_level,
                                tr(&lang, "tooltip_status"),
                                current_status_str,
                                tr(&lang, "tooltip_evdev_channels"),
                                info.channel_count,
                                tr(&lang, "tooltip_click_instruction")
                            ));

                            if response.clicked() {
                                self.trigger_async_mouse_scan(ctx.clone());
                                self.status_message = tr(&lang, "status_analyzing_background").to_string();
                            }

                            if response.secondary_clicked() {
                                if let Some(ref mut info) = self.mouse_info {
                                    info.is_charging = !info.is_charging;
                                }
                            }
                        } else {
                            let search_btn = ui.add(
                                egui::Button::new(
                                    egui::RichText::new(tr(&lang, "scan_mouse_btn"))
                                        .small()
                                        .strong()
                                        .color(egui::Color32::WHITE),
                                )
                                .fill(egui::Color32::from_rgb(180, 50, 50))
                                .stroke(egui::Stroke::new(1.0_f32, egui::Color32::from_rgb(220, 70, 70)))
                                .rounding(egui::Rounding::same(14.0))
                                .min_size(egui::vec2(0.0, 28.0)),
                            ).on_hover_text(tr(&lang, "search_mouse_tooltip"));

                            if search_btn.clicked() {
                                self.trigger_async_mouse_scan(ctx.clone());
                                self.status_message = tr(&lang, "status_searching_background").to_string();
                            }
                        }
                    });
                });
            });

        // Bottom Status Bar / Toast
        egui::TopBottomPanel::bottom("status_bar")
            .frame(
                egui::Frame::none()
                    .fill(egui::Color32::from_rgb(14, 15, 20))
                    .stroke(egui::Stroke::new(1.0_f32, egui::Color32::from_rgb(30, 33, 44)))
                    .inner_margin(egui::Margin::symmetric(16.0, 6.0)),
            )
            .show(ctx, |ui| {
                ui.horizontal(|ui| {
                    if let Some((msg, time, is_success)) = &self.toast {
                        if time.elapsed() < Duration::from_secs(4) {
                            let color = if *is_success {
                                egui::Color32::from_rgb(0, 215, 175)
                            } else {
                                egui::Color32::from_rgb(255, 90, 90)
                            };
                            let icon = if *is_success { "✅" } else { "⚠️" };
                            ui.label(egui::RichText::new(format!("{} {}", icon, msg)).strong().color(color));
                        } else {
                            ui.label(egui::RichText::new(&self.status_message).small().color(egui::Color32::from_gray(160)));
                        }
                    } else {
                        ui.label(egui::RichText::new(&self.status_message).small().color(egui::Color32::from_gray(160)));
                    }

                    ui.with_layout(egui::Layout::right_to_left(egui::Align::Center), |ui| {
                        ui.label(
                            egui::RichText::new(format!("Bo-Ring v{} • Linux Wayland & X11", env!("CARGO_PKG_VERSION")))
                                .small()
                                .color(egui::Color32::from_gray(100)),
                        );
                    });
                });
            });

        match self.active_tab {
            ConfigTab::ButtonConfig => self.render_button_config_tab(ctx),
            ConfigTab::RingCustomizer => self.render_ring_customizer_tab(ctx),
            ConfigTab::GlobalSettings => self.render_global_settings_tab(ctx),
            ConfigTab::Debug => {
                egui::CentralPanel::default().show(ctx, |ui| {
                    tab_debug::render_tab_debug(self, ui);
                });
            }
        }

        self.render_profile_modal(ctx);
    }
}

pub fn run_config_gui(config: Config) {
    let options = eframe::NativeOptions {
        viewport: egui::ViewportBuilder::default()
            .with_app_id("bo-ring")
            .with_inner_size([1140.0, 700.0])
            .with_min_inner_size([980.0, 600.0])
            .with_resizable(true)
            .with_title("Bo-Ring — Control Center"),
        ..Default::default()
    };

    let _ = eframe::run_native(
        "Bo-Ring",
        options,
        Box::new(|cc| Ok(Box::new(ConfigApp::new(&cc.egui_ctx, config)))),
    );
}


