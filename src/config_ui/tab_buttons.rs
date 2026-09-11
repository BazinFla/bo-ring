use crate::config::ButtonActionConfig;
use crate::devices::{
    get_button_name_lang, ButtonCalloutSide, DeviceButtonConfig, DeviceModelConfig, DeviceRegistry,
};
use crate::i18n::tr;
use eframe::egui;

use super::types::{ActionCategory, ConfigApp, EditorDetectTarget, ShortcutTarget};
use super::widgets::{
    card_frame, card_frame_active, key_combos_equal, render_action_card, render_category_chips,
    render_key_badges, render_search_bar,
};

struct ActionItem {
    category: ActionCategory,
    icon: &'static str,
    title_key: &'static str,
    desc_key: &'static str,
    action: ButtonActionConfig,
}

impl ConfigApp {
    /// Render Tab 1: Physical Button Configuration
    pub(super) fn render_button_config_tab(&mut self, ctx: &egui::Context) {
        let lang = self.lang();
        let current_code = self.selected_button_code;
        let btn_name = if let Some(btn) = self.active_device_profile.find_button(current_code) {
            btn.display_name(&lang)
        } else {
            get_button_name_lang(current_code, &lang)
        };

        egui::SidePanel::right("right_button_inspector")
            .exact_width(450.0)
            .frame(
                egui::Frame::none()
                    .fill(egui::Color32::from_rgb(16, 17, 22))
                    .stroke(egui::Stroke::new(1.0_f32, egui::Color32::from_rgb(32, 35, 48)))
                    .inner_margin(egui::Margin::symmetric(18.0, 16.0)),
            )
            .show(ctx, |ui| {
                ui.add_space(4.0);

                // Inspector Header Card
                card_frame().show(ui, |ui| {
                    ui.horizontal(|ui| {
                        let btn_icon = self
                            .active_device_profile
                            .find_button(current_code)
                            .map(|b| b.icon.as_str())
                            .unwrap_or("🔘");
                        ui.label(egui::RichText::new(btn_icon).size(18.0));
                        ui.vertical(|ui| {
                            ui.heading(
                                egui::RichText::new(&btn_name)
                                    .strong()
                                    .size(16.0)
                                    .color(egui::Color32::WHITE),
                            );
                            ui.label(
                                egui::RichText::new(format!("Evdev Code: {}", current_code))
                                    .small()
                                    .color(egui::Color32::from_gray(140)),
                            );
                        });
                        ui.with_layout(egui::Layout::right_to_left(egui::Align::Center), |ui| {
                            let is_assigned = self.config.buttons.contains_key(&current_code);
                            let status_badge = if is_assigned {
                                egui::RichText::new(tr(&lang, "badge_custom")).small().strong().color(egui::Color32::from_rgb(0, 215, 175))
                            } else {
                                egui::RichText::new(tr(&lang, "badge_default")).small().color(egui::Color32::from_gray(150))
                            };
                            ui.label(status_badge);

                            if self.config.general.mode_editor {
                                ui.add_space(6.0);
                                let edit_modal_btn = egui::Button::new(
                                    egui::RichText::new("✏️")
                                        .size(13.0)
                                        .color(egui::Color32::from_rgb(0, 215, 175)),
                                )
                                .fill(egui::Color32::from_rgb(26, 40, 48))
                                .stroke(egui::Stroke::new(1.0_f32, egui::Color32::from_rgb(0, 160, 130)))
                                .rounding(egui::Rounding::same(4.0))
                                .min_size(egui::vec2(26.0, 24.0));

                                if ui
                                    .add(edit_modal_btn)
                                    .on_hover_text(tr(&lang, "editor_edit_button_tip"))
                                    .clicked()
                                {
                                    self.open_edit_button_modal(current_code);
                                }
                            }
                        });
                    });
                });

                ui.add_space(10.0);

                let current_action = self.config.buttons.get(&current_code).cloned();

                // --- SEPARATE CUSTOM KEYBOARD SHORTCUT SECTION ---
                let is_recording = self.recording_shortcut_for == Some(ShortcutTarget::Button(current_code));
                let is_keycombo = matches!(current_action, Some(ButtonActionConfig::KeyCombo { .. }))
                    && !matches!(
                        &current_action,
                        Some(ButtonActionConfig::KeyCombo { keys })
                            if keys.len() == 1 && (keys[0] == "Print" || keys[0] == "Super")
                    );

                let card_style = if is_recording {
                    egui::Frame::none()
                        .fill(egui::Color32::from_rgb(45, 20, 24))
                        .stroke(egui::Stroke::new(1.5_f32, egui::Color32::from_rgb(255, 80, 80)))
                        .rounding(egui::Rounding::same(10.0))
                        .inner_margin(egui::Margin::symmetric(14.0, 10.0))
                } else if is_keycombo {
                    card_frame_active()
                } else {
                    card_frame()
                };

                card_style.show(ui, |ui| {
                    ui.horizontal(|ui| {
                        ui.label(egui::RichText::new("⌨️").size(18.0));
                        ui.vertical(|ui| {
                            ui.label(
                                egui::RichText::new(match lang.as_str() {
                                    "fr" => "Raccourci Clavier personnalisé",
                                    "es" => "Atajo de teclado personalizado",
                                    _ => "Custom Keyboard Shortcut",
                                })
                                .strong()
                                .size(13.0)
                                .color(egui::Color32::WHITE),
                            );

                            if is_recording {
                                ui.label(
                                    egui::RichText::new(tr(&lang, "press_keys_prompt"))
                                        .strong()
                                        .color(egui::Color32::from_rgb(255, 100, 100)),
                                );
                            } else if let Some(ButtonActionConfig::KeyCombo { ref keys }) = current_action {
                                render_key_badges(ui, keys);
                            } else {
                                ui.label(
                                    egui::RichText::new(tr(&lang, "record_shortcut_hint"))
                                        .small()
                                        .color(egui::Color32::from_gray(140)),
                                );
                            }
                        });

                        ui.with_layout(egui::Layout::right_to_left(egui::Align::Center), |ui| {
                            let btn_label = if is_recording {
                                tr(&lang, "cancel")
                            } else if is_keycombo {
                                tr(&lang, "modify_btn")
                            } else {
                                tr(&lang, "record_btn")
                            };

                            let btn = egui::Button::new(
                                egui::RichText::new(btn_label).strong().size(12.0).color(egui::Color32::WHITE)
                            )
                            .fill(if is_recording { egui::Color32::from_rgb(200, 50, 50) } else { egui::Color32::from_rgb(0, 170, 140) })
                            .rounding(egui::Rounding::same(6.0));

                            if ui.add(btn).clicked() {
                                if is_recording {
                                    self.recording_shortcut_for = None;
                                } else {
                                    self.recording_shortcut_for = Some(ShortcutTarget::Button(current_code));
                                }
                            }
                        });
                    });
                });

                ui.add_space(10.0);
                ui.separator();
                ui.add_space(8.0);

                // Search Bar
                render_search_bar(
                    ui,
                    &mut self.action_search_query,
                    match lang.as_str() {
                        "fr" => "Rechercher une action (ex: capture, média, fenêtres...)",
                        "es" => "Buscar una acción...",
                        _ => "Search action (e.g., screenshot, volume, windows...)",
                    },
                );

                ui.add_space(8.0);

                // Category Chips
                render_category_chips(ui, &mut self.selected_action_category, &lang);

                ui.add_space(8.0);

                egui::ScrollArea::vertical()
                    .auto_shrink([false, false])
                    .show(ui, |ui| {
                        ui.spacing_mut().item_spacing.y = 8.0;

                        let search_q = self.action_search_query.to_lowercase();
                        let active_cat = self.selected_action_category;

                        // 1. Action: Native Passthrough (Default)
                        if (active_cat == ActionCategory::All || active_cat == ActionCategory::Navigation)
                            && (search_q.is_empty() || "par defaut default passthrough".contains(&search_q))
                        {
                            let is_active = current_action.is_none();
                            if render_action_card(
                                ui,
                                "↩️",
                                tr(&lang, "action_passthrough_title"),
                                tr(&lang, "action_passthrough_desc"),
                                is_active,
                            ) {
                                self.config.buttons.remove(&current_code);
                                self.notify_success(tr(&lang, "status_action_reset"));
                            }
                        }

                        // 2. Action: Action Ring Menu
                        if (active_cat == ActionCategory::All || active_cat == ActionCategory::Navigation)
                            && (search_q.is_empty() || "action ring menu radial anneau".contains(&search_q))
                        {
                            let is_active = matches!(current_action, Some(ButtonActionConfig::ShowRingMenu));
                            if render_action_card(
                                ui,
                                "⭕",
                                tr(&lang, "action_ring_title"),
                                tr(&lang, "action_ring_desc"),
                                is_active,
                            ) {
                                self.config.buttons.insert(current_code, ButtonActionConfig::ShowRingMenu);
                                self.notify_success(tr(&lang, "status_ring_assigned"));
                            }
                        }

                        // 3. Action Catalog Items
                        let catalog: &[ActionItem] = &[
                            // Navigation
                            ActionItem { category: ActionCategory::Navigation, icon: "⬅️", title_key: "action_back", desc_key: "action_back_desc", action: ButtonActionConfig::KeyCombo { keys: vec!["ALT".to_string(), "Left".to_string()] } },
                            ActionItem { category: ActionCategory::Navigation, icon: "➡️", title_key: "action_next", desc_key: "action_next_desc", action: ButtonActionConfig::KeyCombo { keys: vec!["ALT".to_string(), "Right".to_string()] } },
                            ActionItem { category: ActionCategory::Navigation, icon: "🔀", title_key: "action_switch_app", desc_key: "action_switch_app_desc", action: ButtonActionConfig::KeyCombo { keys: vec!["ALT".to_string(), "Tab".to_string()] } },
                            // Windows & Workspace
                            ActionItem { category: ActionCategory::Windows, icon: "🪟", title_key: "action_overview", desc_key: "action_overview_desc", action: ButtonActionConfig::KeyCombo { keys: vec!["Super".to_string()] } },
                            ActionItem { category: ActionCategory::Windows, icon: "💻", title_key: "action_show_desktop", desc_key: "action_show_desktop_desc", action: ButtonActionConfig::KeyCombo { keys: vec!["CTRL".to_string(), "ALT".to_string(), "d".to_string()] } },
                            ActionItem { category: ActionCategory::Windows, icon: "📸", title_key: "action_screenshot", desc_key: "action_screenshot_desc", action: ButtonActionConfig::KeyCombo { keys: vec!["Print".to_string()] } },
                            // Media
                            ActionItem { category: ActionCategory::Media, icon: "⏯️", title_key: "action_play_pause", desc_key: "action_play_pause_desc", action: ButtonActionConfig::KeyCombo { keys: vec!["PlayPause".to_string()] } },
                            ActionItem { category: ActionCategory::Media, icon: "⏭️", title_key: "action_next_song", desc_key: "action_next_song_desc", action: ButtonActionConfig::KeyCombo { keys: vec!["NextSong".to_string()] } },
                            ActionItem { category: ActionCategory::Media, icon: "⏮️", title_key: "action_prev_song", desc_key: "action_prev_song_desc", action: ButtonActionConfig::KeyCombo { keys: vec!["PreviousSong".to_string()] } },
                            ActionItem { category: ActionCategory::Media, icon: "🔊", title_key: "action_vol_up", desc_key: "action_vol_up_desc", action: ButtonActionConfig::KeyCombo { keys: vec!["VolumeUp".to_string()] } },
                            ActionItem { category: ActionCategory::Media, icon: "🔉", title_key: "action_vol_down", desc_key: "action_vol_down_desc", action: ButtonActionConfig::KeyCombo { keys: vec!["VolumeDown".to_string()] } },
                            ActionItem { category: ActionCategory::Media, icon: "🔇", title_key: "action_mute", desc_key: "action_mute_desc", action: ButtonActionConfig::KeyCombo { keys: vec!["Mute".to_string()] } },
                            // Apps & System
                            ActionItem { category: ActionCategory::Apps, icon: "🖥️", title_key: "action_custom_cmd", desc_key: "action_custom_cmd_desc", action: ButtonActionConfig::Command { cmd: "gnome-terminal || ptyxis || kgx || konsole || alacritty || kitty || x-terminal-emulator || xterm".to_string() } },
                            ActionItem { category: ActionCategory::System, icon: "🔒", title_key: "action_lock", desc_key: "action_lock_desc", action: ButtonActionConfig::Command { cmd: "loginctl lock-session".to_string() } },
                            ActionItem { category: ActionCategory::System, icon: "🚪", title_key: "action_logout", desc_key: "action_logout_desc", action: ButtonActionConfig::Command { cmd: "loginctl terminate-user \"\"".to_string() } },
                            ActionItem { category: ActionCategory::System, icon: "⏻", title_key: "action_poweroff", desc_key: "action_poweroff_desc", action: ButtonActionConfig::Command { cmd: "systemctl poweroff".to_string() } },
                        ];

                        for item in catalog {
                            if active_cat != ActionCategory::All && active_cat != item.category {
                                continue;
                            }

                            let title = tr(&lang, item.title_key);
                            let desc = tr(&lang, item.desc_key);

                            if !search_q.is_empty() {
                                let match_title = title.to_lowercase().contains(&search_q);
                                let match_desc = desc.to_lowercase().contains(&search_q);
                                if !match_title && !match_desc {
                                    continue;
                                }
                            }

                            let is_active = match (&current_action, &item.action) {
                                (Some(ButtonActionConfig::KeyCombo { keys: k1 }), ButtonActionConfig::KeyCombo { keys: k2 }) => key_combos_equal(k1, k2),
                                (Some(ButtonActionConfig::Command { cmd: c1 }), ButtonActionConfig::Command { cmd: c2 }) => c1 == c2,
                                _ => false,
                            };

                            if render_action_card(ui, item.icon, title, desc, is_active) {
                                self.config.buttons.insert(current_code, item.action.clone());
                                self.notify_success(tr(&lang, "action_assigned_toast").replace("{}", title));
                            }
                        }

                        // 5. Action: Open Folder Card
                        if active_cat == ActionCategory::All || active_cat == ActionCategory::Apps {
                            card_frame().show(ui, |ui| {
                                ui.vertical(|ui| {
                                    ui.horizontal(|ui| {
                                        ui.label(egui::RichText::new("📁").size(18.0));
                                        ui.label(egui::RichText::new(tr(&lang, "open_folder_title")).strong().size(13.0).color(egui::Color32::WHITE));
                                    });

                                    let mut folder_path = if let Some(ButtonActionConfig::Command { ref cmd }) = current_action {
                                        if cmd.starts_with("xdg-open ") && !cmd.starts_with("xdg-open http") {
                                            cmd["xdg-open ".len()..].to_string()
                                        } else {
                                            "~".to_string()
                                        }
                                    } else {
                                        "~".to_string()
                                    };

                                    ui.horizontal(|ui| {
                                        if ui.add(egui::TextEdit::singleline(&mut folder_path).desired_width(ui.available_width() - 85.0)).changed() {
                                            self.config.buttons.insert(current_code, ButtonActionConfig::Command { cmd: format!("xdg-open {}", folder_path) });
                                        }

                                        let browse_btn = egui::Button::new(
                                            egui::RichText::new(tr(&lang, "browse_btn")).small().strong().color(egui::Color32::WHITE)
                                        )
                                        .fill(egui::Color32::from_rgb(38, 48, 64))
                                        .stroke(egui::Stroke::new(1.0_f32, egui::Color32::from_rgb(70, 90, 120)))
                                        .rounding(egui::Rounding::same(6.0));

                                        if ui.add(browse_btn).clicked() {
                                            if let Some(path) = rfd::FileDialog::new().pick_folder() {
                                                let path_str = path.to_string_lossy().to_string();
                                                self.config.buttons.insert(current_code, ButtonActionConfig::Command { cmd: format!("xdg-open {}", path_str) });
                                                self.notify_success(tr(&lang, "status_folder_assigned").replace("{}", &path_str));
                                            }
                                        }
                                    });
                                });
                            });
                        }

                        // 6. Action: Custom Command Card
                        if active_cat == ActionCategory::All || active_cat == ActionCategory::Apps || active_cat == ActionCategory::System {
                            card_frame().show(ui, |ui| {
                                ui.vertical(|ui| {
                                    ui.horizontal(|ui| {
                                        ui.label(egui::RichText::new("⚡").size(18.0));
                                        ui.label(egui::RichText::new(tr(&lang, "custom_cmd_title")).strong().size(13.0).color(egui::Color32::WHITE));
                                    });

                                    ui.horizontal(|ui| {
                                        if ui.add(egui::TextEdit::singleline(&mut self.custom_cmd_input).hint_text("ex: firefox https://google.com").desired_width(ui.available_width() - 75.0)).changed() {
                                            self.config.buttons.insert(current_code, ButtonActionConfig::Command { cmd: self.custom_cmd_input.clone() });
                                        }

                                        let test_btn = egui::Button::new(
                                            egui::RichText::new(tr(&lang, "test_btn")).small().strong().color(egui::Color32::from_rgb(0, 215, 175))
                                        )
                                        .fill(egui::Color32::from_rgb(24, 34, 40))
                                        .stroke(egui::Stroke::new(1.0_f32, egui::Color32::from_rgb(0, 215, 175)))
                                        .rounding(egui::Rounding::same(6.0));

                                        if ui.add(test_btn).clicked() && !self.custom_cmd_input.is_empty() {
                                            let cmd = self.custom_cmd_input.clone();
                                            std::thread::spawn(move || {
                                                let _ = std::process::Command::new("sh").args(["-c", &cmd]).spawn();
                                            });
                                            self.notify_success(tr(&lang, "status_cmd_tested"));
                                        }
                                    });
                                });
                            });
                        }
                    });
            });

        // Central Panel: Interactive Mouse View with Modern Badges & Guidance
        egui::CentralPanel::default()
            .frame(egui::Frame::none().fill(egui::Color32::from_rgb(12, 13, 17)))
            .show(ctx, |ui| {
                let rect = ui.max_rect();

                // 1. Top Interactive Guidance Header
                let header_h = 38.0;
                let header_rect = egui::Rect::from_min_size(
                    egui::pos2(rect.min.x + 14.0, rect.min.y + 8.0),
                    egui::vec2(rect.width() - 28.0, header_h),
                );

                ui.painter().rect(
                    header_rect,
                    egui::Rounding::same(9.0),
                    egui::Color32::from_rgb(18, 20, 26),
                    egui::Stroke::new(1.0_f32, egui::Color32::from_rgb(34, 38, 50)),
                );

                let header_ui_rect = header_rect.shrink2(egui::vec2(12.0, 4.0));
                ui.allocate_ui_at_rect(header_ui_rect, |ui| {
                    ui.horizontal_centered(|ui| {
                        ui.label(egui::RichText::new("💡").size(14.0));
                        ui.label(
                            egui::RichText::new(tr(&lang, "select_button_instruction"))
                                .size(12.0)
                                .color(egui::Color32::from_gray(210)),
                        );

                        ui.with_layout(egui::Layout::right_to_left(egui::Align::Center), |ui| {
                            if self.config.general.mode_editor {
                                let copy_all_btn = egui::Button::new(
                                    egui::RichText::new("📋")
                                        .size(13.0),
                                )
                                .min_size(egui::vec2(28.0, 24.0))
                                .fill(egui::Color32::from_rgb(34, 48, 44))
                                .stroke(egui::Stroke::new(1.0_f32, egui::Color32::from_rgb(0, 160, 130)))
                                .rounding(egui::Rounding::same(6.0));

                                if ui
                                    .add(copy_all_btn)
                                    .on_hover_text(tr(&lang, "editor_copy_json_tip"))
                                    .clicked()
                                {
                                    let mut updated = self.active_device_profile.clone();
                                    for btn in &mut updated.buttons {
                                        if let Some(pos) = self.anchor_positions.get(&btn.code) {
                                            btn.anchor = [
                                                (pos.x * 1000.0).round() / 1000.0,
                                                (pos.y * 1000.0).round() / 1000.0,
                                            ];
                                        }
                                    }
                                    if let Ok(json_str) = serde_json::to_string_pretty(&updated) {
                                        ui.output_mut(|o| o.copied_text = json_str);
                                        self.status_message = tr(&lang, "editor_json_copied").to_string();
                                    }
                                }

                                ui.add_space(4.0);
                                let save_profile_btn = egui::Button::new(
                                    egui::RichText::new("💾")
                                        .size(13.0),
                                )
                                .min_size(egui::vec2(28.0, 24.0))
                                .fill(egui::Color32::from_rgb(30, 48, 38))
                                .stroke(egui::Stroke::new(1.0_f32, egui::Color32::from_rgb(0, 180, 140)))
                                .rounding(egui::Rounding::same(6.0));

                                if ui
                                    .add(save_profile_btn)
                                    .on_hover_text(tr(&lang, "editor_save_profile_tip"))
                                    .clicked()
                                {
                                    let mut updated = self.active_device_profile.clone();
                                    for btn in &mut updated.buttons {
                                        if let Some(pos) = self.anchor_positions.get(&btn.code) {
                                            btn.anchor = [
                                                (pos.x * 1000.0).round() / 1000.0,
                                                (pos.y * 1000.0).round() / 1000.0,
                                            ];
                                        }
                                    }
                                    if let Ok(json_str) = serde_json::to_string_pretty(&updated) {
                                        let dest_dir = DeviceRegistry::ensure_user_devices_dir();
                                        let dest_file = dest_dir.join(format!("{}.json", updated.id));
                                        if std::fs::write(&dest_file, &json_str).is_ok() {
                                            self.device_registry = DeviceRegistry::new();
                                            self.active_device_profile = updated;
                                            self.notify_success(tr(&lang, "editor_profile_saved"));
                                        }
                                    }
                                }

                                ui.add_space(4.0);

                                let is_detecting_curr_hw = self.editor_detecting == Some(EditorDetectTarget::CurrentDeviceHardware);
                                let hw_btn = egui::Button::new(
                                    egui::RichText::new(if is_detecting_curr_hw { "⏳" } else { "🎯" })
                                        .size(13.0)
                                        .color(if is_detecting_curr_hw {
                                            egui::Color32::from_rgb(255, 215, 0)
                                        } else {
                                            egui::Color32::from_rgb(0, 215, 175)
                                        }),
                                )
                                .min_size(egui::vec2(28.0, 24.0))
                                .fill(if is_detecting_curr_hw {
                                    egui::Color32::from_rgb(50, 45, 20)
                                } else {
                                    egui::Color32::from_rgb(26, 38, 48)
                                })
                                .stroke(egui::Stroke::new(
                                    1.0_f32,
                                    if is_detecting_curr_hw {
                                        egui::Color32::from_rgb(255, 215, 0)
                                    } else {
                                        egui::Color32::from_rgb(0, 180, 220)
                                    },
                                ))
                                .rounding(egui::Rounding::same(6.0));

                                let hw_tip = if is_detecting_curr_hw {
                                    tr(&lang, "hardware_id_detecting").to_string()
                                } else {
                                    let ids = if self.active_device_profile.match_ids.is_empty() {
                                        tr(&lang, "hardware_id_none").to_string()
                                    } else {
                                        self.active_device_profile.match_ids.join(", ")
                                    };
                                    format!("{}\nVID:PID: {}", tr(&lang, "editor_detect_hw_tip"), ids)
                                };

                                let hw_resp = ui.add(hw_btn).on_hover_text(hw_tip);
                                if hw_resp.clicked() {
                                    if is_detecting_curr_hw {
                                        self.editor_detecting = None;
                                    } else {
                                        self.editor_detecting = Some(EditorDetectTarget::CurrentDeviceHardware);
                                    }
                                }

                                hw_resp.context_menu(|ui| {
                                    ui.label(egui::RichText::new(tr(&lang, "hardware_id_label")).strong());
                                    if self.active_device_profile.match_ids.is_empty() {
                                        ui.label(egui::RichText::new(tr(&lang, "hardware_id_none")).italics().small());
                                    } else {
                                        let mut to_remove = None;
                                        for (idx, id) in self.active_device_profile.match_ids.iter().enumerate() {
                                            ui.horizontal(|ui| {
                                                ui.label(egui::RichText::new(id).monospace());
                                                if ui.small_button("✖").clicked() {
                                                    to_remove = Some(idx);
                                                }
                                            });
                                        }
                                        if let Some(idx) = to_remove {
                                            self.active_device_profile.match_ids.remove(idx);
                                            let _ = self.device_registry.save_profile(&self.active_device_profile);
                                            ui.close_menu();
                                        }
                                    }
                                });

                                ui.add_space(4.0);
                            }

                            // Quick toggle for mode_editor
                            let is_editor = self.config.general.mode_editor;
                            let editor_btn = egui::Button::new(
                                egui::RichText::new("✏️")
                                    .size(13.0),
                            )
                            .min_size(egui::vec2(28.0, 24.0))
                            .fill(if is_editor {
                                egui::Color32::from_rgb(20, 50, 45)
                            } else {
                                egui::Color32::from_rgb(34, 38, 50)
                            })
                            .stroke(egui::Stroke::new(
                                1.0_f32,
                                if is_editor {
                                    egui::Color32::from_rgb(0, 200, 160)
                                } else {
                                    egui::Color32::from_rgb(50, 56, 74)
                                },
                            ))
                            .rounding(egui::Rounding::same(6.0));

                            let editor_tip = if is_editor {
                                format!("{}\n{}", tr(&lang, "editor_mode_active_tip"), tr(&lang, "model_editor_mode_desc"))
                            } else {
                                format!("{}\n{}", tr(&lang, "editor_mode_tip"), tr(&lang, "model_editor_mode_desc"))
                            };

                            if ui.add(editor_btn).on_hover_text(editor_tip).clicked() {
                                self.config.general.mode_editor = !self.config.general.mode_editor;
                                let _ = self.save_config();
                                if self.config.general.mode_editor {
                                    self.status_message = tr(&lang, "editor_status_enabled").to_string();
                                } else {
                                    self.status_message = tr(&lang, "editor_status_disabled").to_string();
                                }
                            }

                            ui.add_space(8.0);

                            // Add new profile button '+'
                            let add_device_btn = egui::Button::new(
                                egui::RichText::new("➕")
                                    .size(13.0)
                                    .color(egui::Color32::from_rgb(0, 215, 175)),
                            )
                            .min_size(egui::vec2(28.0, 24.0))
                            .fill(egui::Color32::from_rgb(34, 38, 50))
                            .stroke(egui::Stroke::new(1.0_f32, egui::Color32::from_rgb(50, 56, 74)))
                            .rounding(egui::Rounding::same(6.0));

                            if ui
                                .add(add_device_btn)
                                .on_hover_text(tr(&lang, "editor_new_profile_tip"))
                                .clicked()
                            {
                                self.show_new_device_modal = true;
                                self.new_device_name = String::new();
                                self.new_device_id = String::new();
                                self.new_device_id_edited = false;
                                self.new_device_duplicate_active = true;
                                self.new_device_match_ids.clear();
                                self.editor_detecting = None;
                            }

                            ui.add_space(4.0);

                            // Quick profile selector right on Tab 1
                            let auto_label = tr(&lang, "device_profile_auto").to_string();
                            let current_display = if self.config.general.device_profile == "auto" {
                                format!("🖲️ {} ({})", auto_label, self.active_device_profile.name)
                            } else {
                                format!("🖲️ {}", self.active_device_profile.name)
                            };

                            let mut tab1_profile_changed = None;
                            let mut refresh_requested = false;
                            egui::ComboBox::from_id_source("tab1_device_profile_combo")
                                .selected_text(
                                    egui::RichText::new(current_display)
                                        .small()
                                        .color(egui::Color32::WHITE),
                                )
                                .show_ui(ui, |ui| {
                                    let is_auto_sel = self.config.general.device_profile == "auto";
                                    if ui.selectable_label(is_auto_sel, &auto_label).clicked() {
                                        tab1_profile_changed = Some("auto".to_string());
                                    }
                                    ui.separator();
                                    for p in self.device_registry.list_profiles() {
                                        let is_sel = self.config.general.device_profile == p.config.id;
                                        let tag = match p.source {
                                            crate::devices::ProfileSource::BuiltIn => format!("📦 {}", tr(&lang, "device_profile_builtin")),
                                            crate::devices::ProfileSource::System(_) => format!("🖥️ {}", tr(&lang, "device_profile_system")),
                                            crate::devices::ProfileSource::User(_) => format!("👤 {}", tr(&lang, "device_profile_user")),
                                        };
                                        let item_label = format!("{}  [{}]", p.config.name, tag);
                                        if ui.selectable_label(is_sel, item_label).clicked() {
                                            tab1_profile_changed = Some(p.config.id.clone());
                                        }
                                    }
                                    ui.separator();
                                    if ui.selectable_label(false, format!("📂 {}", tr(&lang, "open_devices_folder_btn"))).clicked() {
                                        let dir = crate::devices::DeviceRegistry::ensure_user_devices_dir();
                                        let _ = std::process::Command::new("xdg-open").arg(dir).spawn();
                                    }
                                    if ui.selectable_label(false, format!("🔄 {}", tr(&lang, "refresh_profiles_btn"))).clicked() {
                                        refresh_requested = true;
                                    }
                                });
                            if refresh_requested {
                                self.device_registry = crate::devices::DeviceRegistry::new();
                                if self.config.general.device_profile != "auto" {
                                    let id = self.config.general.device_profile.clone();
                                    self.select_device_profile(ctx, &id);
                                }
                                self.status_message = tr(&lang, "save_success").to_string();
                            }
                            if let Some(new_id) = tab1_profile_changed {
                                self.config.general.device_profile = new_id.clone();
                                if new_id != "auto" {
                                    self.select_device_profile(ctx, &new_id);
                                } else {
                                    let detected = self.mouse_info.as_ref().and_then(|i| i.detected_profile_id.clone());
                                    if let Some(ref det_id) = detected {
                                        self.select_device_profile(ctx, det_id);
                                    }
                                }
                                let _ = self.save_config();
                                self.status_message = tr(&lang, "status_device_profile_selected").replace("{}", &self.active_device_profile.name);
                            }
                        });
                });
            });

                // 2. Center Mouse Canvas
                let canvas_rect = egui::Rect::from_min_max(
                    egui::pos2(rect.min.x, rect.min.y + header_h + 12.0),
                    egui::pos2(rect.max.x, rect.max.y - 12.0),
                );
                let mouse_center = egui::pos2(canvas_rect.center().x - 10.0, canvas_rect.center().y);

                if let Some(texture) = &self.texture {
                    let img_size = texture.size_vec2();
                    let scale = (canvas_rect.height() * 0.76) / img_size.y;
                    let target_size = img_size * scale;
                    let img_rect = egui::Rect::from_center_size(mouse_center, target_size);

                    ui.painter().image(
                        texture.id(),
                        img_rect,
                        egui::Rect::from_min_max(egui::pos2(0.0, 0.0), egui::pos2(1.0, 1.0)),
                        egui::Color32::WHITE,
                    );

                    struct CalloutRenderItem {
                        code: u16,
                        icon: String,
                        label: String,
                        side: crate::devices::ButtonCalloutSide,
                        badge_y_ratio: f32,
                    }

                    let mut left_buttons: Vec<&crate::devices::DeviceButtonConfig> = Vec::new();
                    let mut right_buttons: Vec<&crate::devices::DeviceButtonConfig> = Vec::new();

                    for btn in &self.active_device_profile.buttons {
                        match btn.side {
                            crate::devices::ButtonCalloutSide::Left => left_buttons.push(btn),
                            crate::devices::ButtonCalloutSide::Right => right_buttons.push(btn),
                        }
                    }

                    let mut callouts: Vec<CalloutRenderItem> = Vec::new();

                    for (i, btn) in left_buttons.iter().enumerate() {
                        let ratio = btn.badge_y_ratio.unwrap_or_else(|| {
                            if left_buttons.len() <= 1 {
                                0.5
                            } else {
                                0.16 + 0.60 * (i as f32 / (left_buttons.len() - 1) as f32)
                            }
                        });
                        callouts.push(CalloutRenderItem {
                            code: btn.code,
                            icon: btn.icon.clone(),
                            label: btn.display_name(&lang),
                            side: crate::devices::ButtonCalloutSide::Left,
                            badge_y_ratio: ratio,
                        });
                    }

                    for (i, btn) in right_buttons.iter().enumerate() {
                        let ratio = btn.badge_y_ratio.unwrap_or_else(|| {
                            if right_buttons.len() <= 1 {
                                0.5
                            } else {
                                0.20 + 0.55 * (i as f32 / (right_buttons.len() - 1) as f32)
                            }
                        });
                        callouts.push(CalloutRenderItem {
                            code: btn.code,
                            icon: btn.icon.clone(),
                            label: btn.display_name(&lang),
                            side: crate::devices::ButtonCalloutSide::Right,
                            badge_y_ratio: ratio,
                        });
                    }

                    let is_editor_mode = self.config.general.mode_editor;
                    let pointer_down = ui.input(|i| i.pointer.primary_down());
                    let pointer_pos = ui.input(|i| i.pointer.hover_pos().or_else(|| i.pointer.latest_pos()));

                    // Handle cancel if Escape pressed
                    if ui.input(|i| i.key_pressed(egui::Key::Escape)) {
                        self.dragging_button = None;
                    }

                    let mut drop_to_apply: Option<(u16, f32, ButtonCalloutSide)> = None;
                    let mut open_edit_for_button: Option<u16> = None;
                    let mut delete_button_to_apply: Option<u16> = None;

                    for item in callouts {
                        let code = item.code;
                        let btn_icon = &item.icon;
                        let label = &item.label;

                        ui.push_id(code, |ui| {
                        let anchor_rel = self.anchor_positions.get(&code).cloned().unwrap_or(egui::vec2(0.5, 0.5));
                        let mut anchor_pos = egui::pos2(
                            img_rect.min.x + img_rect.width() * anchor_rel.x,
                            img_rect.min.y + img_rect.height() * anchor_rel.y,
                        );

                        let badge_w = 205.0;
                        let badge_h = 48.0;

                        let badge_x = match item.side {
                            crate::devices::ButtonCalloutSide::Left => (img_rect.min.x - 110.0).clamp(canvas_rect.min.x + 110.0, mouse_center.x - 100.0),
                            crate::devices::ButtonCalloutSide::Right => (img_rect.max.x + 110.0).clamp(mouse_center.x + 100.0, canvas_rect.max.x - 110.0),
                        };

                        let badge_y = img_rect.min.y + img_rect.height() * item.badge_y_ratio;

                        // Check if THIS button is currently being dragged
                        let is_this_dragged = is_editor_mode && self.dragging_button == Some(code);
                        let is_other_dragged = is_editor_mode && self.dragging_button.is_some() && self.dragging_button != Some(code);

                        let (badge_pos, current_ratio, current_side) = if is_this_dragged {
                            if let Some(pos) = pointer_pos {
                                let ratio = ((pos.y - img_rect.min.y) / img_rect.height()).clamp(0.05, 0.95);
                                let rounded = (ratio * 100.0).round() / 100.0;
                                let side = if pos.x < mouse_center.x - 20.0 {
                                    ButtonCalloutSide::Left
                                } else if pos.x > mouse_center.x + 20.0 {
                                    ButtonCalloutSide::Right
                                } else {
                                    item.side
                                };
                                let clamped_x = pos.x.clamp(canvas_rect.min.x + 110.0, canvas_rect.max.x - 110.0);
                                let clamped_y = pos.y.clamp(canvas_rect.min.y + 30.0, canvas_rect.max.y - 30.0);
                                (egui::pos2(clamped_x, clamped_y), rounded, side)
                            } else {
                                (egui::pos2(badge_x, badge_y), item.badge_y_ratio, item.side)
                            }
                        } else {
                            (egui::pos2(badge_x, badge_y), item.badge_y_ratio, item.side)
                        };

                        let badge_rect = egui::Rect::from_center_size(badge_pos, egui::vec2(badge_w, badge_h));

                        // Drag handle grip rect on the outer side of the card
                        let grip_rect = match current_side {
                            ButtonCalloutSide::Left => egui::Rect::from_min_size(
                                egui::pos2(badge_rect.min.x - 26.0, badge_rect.min.y + 6.0),
                                egui::vec2(22.0, badge_h - 12.0),
                            ),
                            ButtonCalloutSide::Right => egui::Rect::from_min_size(
                                egui::pos2(badge_rect.max.x + 4.0, badge_rect.min.y + 6.0),
                                egui::vec2(22.0, badge_h - 12.0),
                            ),
                        };

                        let anchor_sense_rect = egui::Rect::from_center_size(anchor_pos, egui::vec2(28.0, 28.0));
                        let anchor_resp = if is_editor_mode && self.dragging_button.is_none() {
                            ui.allocate_rect(anchor_sense_rect, egui::Sense::drag())
                        } else {
                            ui.allocate_rect(anchor_sense_rect, egui::Sense::click())
                        };

                        if !is_editor_mode && anchor_resp.clicked() {
                            self.selected_button_code = code;
                        }

                        if is_editor_mode && self.dragging_button.is_none() && anchor_resp.dragged() {
                            let delta = anchor_resp.drag_delta();
                            let new_rel_x = ((anchor_pos.x + delta.x - img_rect.min.x) / img_rect.width()).clamp(0.0, 1.0);
                            let new_rel_y = ((anchor_pos.y + delta.y - img_rect.min.y) / img_rect.height()).clamp(0.0, 1.0);

                            let rounded_x = (new_rel_x * 1000.0).round() / 1000.0;
                            let rounded_y = (new_rel_y * 1000.0).round() / 1000.0;

                            self.anchor_positions.insert(code, egui::vec2(rounded_x, rounded_y));
                            anchor_pos = egui::pos2(
                                img_rect.min.x + img_rect.width() * rounded_x,
                                img_rect.min.y + img_rect.height() * rounded_y,
                            );

                            self.status_message = format!("📍 {} ({}) : \"anchor\": [{:.3}, {:.3}]", label, code, rounded_x, rounded_y);
                            println!("📍 Button {} ({}) -> \"anchor\": [{:.3}, {:.3}],", code, label, rounded_x, rounded_y);
                        }

                        let is_selected = self.selected_button_code == code;
                        let highlight_color = egui::Color32::from_rgb(0, 215, 175);

                        // Card & Grip Response
                        let card_resp = if is_editor_mode {
                            if is_other_dragged {
                                ui.allocate_rect(badge_rect, egui::Sense::hover())
                            } else {
                                ui.allocate_rect(badge_rect, egui::Sense::click_and_drag())
                            }
                        } else {
                            ui.allocate_rect(badge_rect, egui::Sense::click())
                        };

                        let grip_resp = if is_editor_mode {
                            if is_other_dragged {
                                ui.allocate_rect(grip_rect, egui::Sense::hover())
                            } else {
                                ui.allocate_rect(grip_rect, egui::Sense::drag())
                            }
                        } else {
                            ui.allocate_rect(grip_rect, egui::Sense::hover())
                        };

                        // Check drag initiation
                        if is_editor_mode && self.dragging_button.is_none() && (card_resp.drag_started() || grip_resp.drag_started() || card_resp.dragged() || grip_resp.dragged()) {
                            self.dragging_button = Some(code);
                            self.selected_button_code = code;
                        }

                        let is_being_dragged = is_this_dragged || (is_editor_mode && self.dragging_button == Some(code));

                        if is_being_dragged {
                            self.selected_button_code = code;
                            let side_str = if current_side == ButtonCalloutSide::Left {
                                format!("👈 {}", tr(&lang, "editor_modal_side_left"))
                            } else {
                                format!("👉 {}", tr(&lang, "editor_modal_side_right"))
                            };
                            let height_str = format!("{:.0}", current_ratio * 100.0);
                            self.status_message = tr(&lang, "status_button_drag_position")
                                .replacen("{}", label, 1)
                                .replacen("{}", &height_str, 1)
                                .replacen("{}", &side_str, 1);

                            // If pointer was released while this button was being dragged -> Drop!
                            if !pointer_down {
                                drop_to_apply = Some((code, current_ratio, current_side));
                            }
                        }

                        if card_resp.clicked() && self.dragging_button.is_none() {
                            self.selected_button_code = code;
                            if self.deleting_button_code != Some(code) {
                                self.deleting_button_code = None;
                            }
                        }
                        let is_hovered = card_resp.hovered() || anchor_resp.hovered() || grip_resp.hovered();
                        if is_being_dragged {
                            ui.ctx().set_cursor_icon(egui::CursorIcon::Grabbing);
                        } else if is_hovered {
                            if is_editor_mode && grip_resp.hovered() {
                                ui.ctx().set_cursor_icon(egui::CursorIcon::Grab);
                            } else {
                                ui.ctx().set_cursor_icon(egui::CursorIcon::PointingHand);
                            }
                        }

                        // In editor mode, if being dragged, draw the ghost drop-target at docking position
                        if is_being_dragged {
                            let ghost_x = match current_side {
                                ButtonCalloutSide::Left => (img_rect.min.x - 110.0).clamp(canvas_rect.min.x + 110.0, mouse_center.x - 100.0),
                                ButtonCalloutSide::Right => (img_rect.max.x + 110.0).clamp(mouse_center.x + 100.0, canvas_rect.max.x - 110.0),
                            };
                            let ghost_y = img_rect.min.y + img_rect.height() * current_ratio;
                            let ghost_rect = egui::Rect::from_center_size(egui::pos2(ghost_x, ghost_y), egui::vec2(badge_w, badge_h));

                            ui.painter().rect(
                                ghost_rect,
                                egui::Rounding::same(10.0),
                                egui::Color32::from_rgba_unmultiplied(0, 215, 175, 25),
                                egui::Stroke::new(1.5_f32, egui::Color32::from_rgba_unmultiplied(0, 215, 175, 160)),
                            );
                            let dock_text = if current_side == ButtonCalloutSide::Left {
                                tr(&lang, "editor_dock_left")
                            } else {
                                tr(&lang, "editor_dock_right")
                            };
                            ui.painter().text(
                                ghost_rect.center(),
                                egui::Align2::CENTER_CENTER,
                                dock_text,
                                egui::FontId::proportional(11.0),
                                egui::Color32::from_rgb(0, 215, 175),
                            );
                        }

                        // Connection Line & Glow
                        let line_color = if is_selected {
                            highlight_color
                        } else if is_hovered {
                            egui::Color32::from_rgb(0, 160, 130)
                        } else {
                            egui::Color32::from_rgb(45, 52, 70)
                        };
                        let line_stroke = egui::Stroke::new(if is_selected { 2.0_f32 } else { 1.2_f32 }, line_color);
                        ui.painter().line_segment([anchor_pos, badge_pos], line_stroke);

                        // Anchor Halo & Center
                        if is_selected || is_hovered {
                            ui.painter().circle_filled(
                                anchor_pos,
                                12.0,
                                egui::Color32::from_rgba_unmultiplied(0, 215, 175, 45),
                            );
                        }
                        ui.painter().circle_filled(
                            anchor_pos,
                            7.0,
                            if is_selected || anchor_resp.dragged() { highlight_color } else { egui::Color32::WHITE },
                        );
                        ui.painter().circle_stroke(
                            anchor_pos,
                            7.0,
                            egui::Stroke::new(2.0_f32, egui::Color32::from_black_alpha(200)),
                        );

                        // In editor mode, draw a floating badge with coordinates above the anchor
                        if is_editor_mode {
                            let curr_pos = self.anchor_positions.get(&code).cloned().unwrap_or(egui::vec2(0.5, 0.5));
                            let tag_text = format!("{:.3}, {:.3}", curr_pos.x, curr_pos.y);
                            let tag_pos = egui::pos2(anchor_pos.x, anchor_pos.y - 18.0);
                            let tag_bg = if is_selected || anchor_resp.dragged() {
                                egui::Color32::from_rgb(0, 70, 60)
                            } else {
                                egui::Color32::from_rgba_unmultiplied(20, 24, 30, 220)
                            };
                            let tag_rect = egui::Rect::from_center_size(tag_pos, egui::vec2(76.0, 16.0));
                            ui.painter().rect_filled(tag_rect, egui::Rounding::same(4.0), tag_bg);
                            ui.painter().rect_stroke(
                                tag_rect,
                                egui::Rounding::same(4.0),
                                egui::Stroke::new(1.0_f32, if is_selected { highlight_color } else { egui::Color32::from_gray(70) }),
                            );
                            ui.painter().text(
                                tag_pos,
                                egui::Align2::CENTER_CENTER,
                                tag_text,
                                egui::FontId::monospace(10.0),
                                egui::Color32::WHITE,
                            );
                        }

                        // Badge Card Visuals
                        // Badge Card Visuals
                        let (fill_bg, stroke_color) = if is_being_dragged {
                            (egui::Color32::from_rgba_unmultiplied(22, 48, 56, 245), egui::Color32::from_rgb(0, 255, 210))
                        } else if is_selected {
                            (egui::Color32::from_rgb(18, 36, 42), egui::Color32::from_rgb(0, 215, 175))
                        } else if is_hovered {
                            (egui::Color32::from_rgb(28, 32, 42), egui::Color32::from_rgb(70, 78, 100))
                        } else {
                            (egui::Color32::from_rgb(20, 22, 28), egui::Color32::from_rgb(38, 42, 54))
                        };

                        if is_being_dragged {
                            ui.painter().rect_filled(
                                badge_rect.translate(egui::vec2(0.0, 5.0)),
                                egui::Rounding::same(10.0),
                                egui::Color32::from_black_alpha(130),
                            );
                        }

                        ui.painter().rect(
                            badge_rect,
                            egui::Rounding::same(10.0),
                            fill_bg,
                            egui::Stroke::new(if is_being_dragged { 2.0_f32 } else if is_selected { 1.5_f32 } else { 1.0_f32 }, stroke_color),
                        );

                        // Selected indicator bar on side of card
                        if is_selected {
                            let bar_rect = match current_side {
                                crate::devices::ButtonCalloutSide::Left => egui::Rect::from_min_size(
                                    egui::pos2(badge_rect.max.x - 3.0, badge_rect.min.y + 8.0),
                                    egui::vec2(3.0, badge_h - 16.0),
                                ),
                                crate::devices::ButtonCalloutSide::Right => egui::Rect::from_min_size(
                                    egui::pos2(badge_rect.min.x, badge_rect.min.y + 8.0),
                                    egui::vec2(3.0, badge_h - 16.0),
                                ),
                            };
                            ui.painter().rect_filled(bar_rect, egui::Rounding::same(2.0), highlight_color);
                        }

                        // Icon badge box inside card
                        let icon_box_rect = egui::Rect::from_center_size(
                            egui::pos2(badge_rect.min.x + 22.0, badge_rect.center().y),
                            egui::vec2(28.0, 28.0),
                        );
                        let icon_box_bg = if is_selected {
                            egui::Color32::from_rgb(0, 70, 60)
                        } else {
                            egui::Color32::from_rgb(30, 33, 44)
                        };
                        ui.painter().rect_filled(icon_box_rect, egui::Rounding::same(7.0), icon_box_bg);
                        ui.painter().text(
                            icon_box_rect.center(),
                            egui::Align2::CENTER_CENTER,
                            btn_icon,
                            egui::FontId::proportional(14.0),
                            egui::Color32::WHITE,
                        );

                        // Assigned Action Summary
                        let (assigned_summary, is_custom) = match self.config.buttons.get(&code) {
                            Some(ButtonActionConfig::ShowRingMenu) => ("⭕ Action Ring".to_string(), true),
                            Some(ButtonActionConfig::KeyCombo { keys }) => (format!("⌨ {}", keys.join("+")), true),
                            Some(ButtonActionConfig::Command { cmd }) => {
                                if cmd.starts_with("xdg-open ") {
                                    (tr(&lang, "summary_folder").to_string(), true)
                                } else {
                                    (tr(&lang, "summary_cmd").to_string(), true)
                                }
                            }
                            Some(ButtonActionConfig::ActionRef { id }) => (format!("🚀 {}", id), true),
                            None => (tr(&lang, "summary_default").to_string(), false),
                        };

                        // Render Card Titles & Subtitles
                        let text_left = badge_rect.min.x + 44.0;
                        ui.painter().text(
                            egui::pos2(text_left, badge_rect.min.y + 14.0),
                            egui::Align2::LEFT_CENTER,
                            label,
                            egui::FontId::proportional(12.5),
                            egui::Color32::WHITE,
                        );

                        let sub_color = if is_selected {
                            egui::Color32::from_rgb(0, 225, 185)
                        } else if is_custom {
                            egui::Color32::from_rgb(120, 210, 190)
                        } else {
                            egui::Color32::from_gray(140)
                        };

                        ui.painter().text(
                            egui::pos2(text_left, badge_rect.min.y + 32.0),
                            egui::Align2::LEFT_CENTER,
                            &assigned_summary,
                            egui::FontId::proportional(10.5),
                            sub_color,
                        );

                        // In editor mode, paint the drag grip handle and direct edit icon
                        if is_editor_mode {
                            let is_grip_active = is_being_dragged;
                            let grip_bg = if is_grip_active {
                                egui::Color32::from_rgb(0, 100, 80)
                            } else if grip_resp.hovered() {
                                egui::Color32::from_rgb(32, 46, 58)
                            } else {
                                egui::Color32::from_rgb(20, 24, 32)
                            };
                            let grip_stroke = egui::Stroke::new(
                                1.0_f32,
                                if is_grip_active || grip_resp.hovered() {
                                    egui::Color32::from_rgb(0, 215, 175)
                                } else {
                                    egui::Color32::from_rgb(45, 52, 68)
                                },
                            );

                            ui.painter().rect(grip_rect, egui::Rounding::same(6.0), grip_bg, grip_stroke);
                            ui.painter().text(
                                grip_rect.center(),
                                egui::Align2::CENTER_CENTER,
                                "✥",
                                egui::FontId::proportional(13.0),
                                if is_grip_active || grip_resp.hovered() {
                                    egui::Color32::from_rgb(0, 255, 200)
                                } else {
                                    egui::Color32::from_gray(180)
                                },
                            );

                            // Action buttons on card: Edit and Delete
                            let edit_btn_rect = egui::Rect::from_center_size(
                                egui::pos2(badge_rect.max.x - 35.0, badge_rect.min.y + 13.0),
                                egui::vec2(18.0, 18.0),
                            );
                            let edit_resp = ui.allocate_rect(edit_btn_rect, egui::Sense::click());
                            if edit_resp.clicked() {
                                open_edit_for_button = Some(code);
                                self.deleting_button_code = None;
                            }
                            let edit_bg = if edit_resp.hovered() {
                                egui::Color32::from_rgb(0, 80, 70)
                            } else {
                                egui::Color32::from_rgba_unmultiplied(26, 30, 40, 220)
                            };
                            ui.painter().rect(
                                edit_btn_rect,
                                egui::Rounding::same(4.0),
                                edit_bg,
                                egui::Stroke::new(1.0_f32, if edit_resp.hovered() { egui::Color32::from_rgb(0, 215, 175) } else { egui::Color32::from_rgb(55, 65, 85) }),
                            );
                            ui.painter().text(
                                edit_btn_rect.center(),
                                egui::Align2::CENTER_CENTER,
                                "✏️",
                                egui::FontId::proportional(9.5),
                                egui::Color32::WHITE,
                            );

                            // Delete button on card with two-click confirmation
                            let delete_btn_rect = egui::Rect::from_center_size(
                                egui::pos2(badge_rect.max.x - 14.0, badge_rect.min.y + 13.0),
                                egui::vec2(18.0, 18.0),
                            );
                            let can_delete = self.active_device_profile.buttons.len() > 1;
                            let is_confirming_del = self.deleting_button_code == Some(code);

                            let del_resp = if can_delete {
                                ui.allocate_rect(delete_btn_rect, egui::Sense::click())
                            } else {
                                ui.allocate_rect(delete_btn_rect, egui::Sense::hover())
                            };

                            if !can_delete {
                                del_resp.clone().on_hover_text(tr(&lang, "editor_cannot_delete_last"));
                            } else if is_confirming_del {
                                del_resp.clone().on_hover_text(tr(&lang, "editor_confirm_delete"));
                            } else {
                                del_resp.clone().on_hover_text(tr(&lang, "editor_delete_button_tip"));
                            }

                            if del_resp.clicked() && can_delete {
                                if is_confirming_del {
                                    delete_button_to_apply = Some(code);
                                    self.deleting_button_code = None;
                                } else {
                                    self.deleting_button_code = Some(code);
                                }
                            }

                            let (del_bg, del_stroke, del_icon, del_font_size) = if !can_delete {
                                (
                                    egui::Color32::from_rgba_unmultiplied(26, 26, 30, 180),
                                    egui::Stroke::new(1.0_f32, egui::Color32::from_gray(50)),
                                    "🗑️",
                                    9.5,
                                )
                            } else if is_confirming_del {
                                (
                                    egui::Color32::from_rgb(180, 40, 40),
                                    egui::Stroke::new(1.5_f32, egui::Color32::from_rgb(255, 100, 100)),
                                    "✔️",
                                    11.0,
                                )
                            } else if del_resp.hovered() {
                                (
                                    egui::Color32::from_rgb(80, 25, 30),
                                    egui::Stroke::new(1.0_f32, egui::Color32::from_rgb(255, 100, 100)),
                                    "🗑️",
                                    9.5,
                                )
                            } else {
                                (
                                    egui::Color32::from_rgba_unmultiplied(36, 26, 30, 220),
                                    egui::Stroke::new(1.0_f32, egui::Color32::from_rgb(85, 55, 60)),
                                    "🗑️",
                                    9.5,
                                )
                            };

                            ui.painter().rect(delete_btn_rect, egui::Rounding::same(4.0), del_bg, del_stroke);
                            ui.painter().text(
                                delete_btn_rect.center(),
                                egui::Align2::CENTER_CENTER,
                                del_icon,
                                egui::FontId::proportional(del_font_size),
                                egui::Color32::WHITE,
                            );
                        }
                        });
                    }

                    // Apply any drop commit
                    if let Some((code, new_ratio, new_side)) = drop_to_apply {
                        if let Some(btn) = self.active_device_profile.buttons.iter_mut().find(|b| b.code == code) {
                            btn.badge_y_ratio = Some(new_ratio);
                            btn.side = new_side;
                        }
                        self.selected_button_code = code;
                        self.dragging_button = None;
                    } else if !pointer_down {
                        self.dragging_button = None;
                    }
                    if let Some(code) = open_edit_for_button {
                        self.selected_button_code = code;
                        self.open_edit_button_modal(code);
                    }
                    if let Some(code) = delete_button_to_apply {
                        if self.active_device_profile.buttons.len() > 1 {
                            self.active_device_profile.buttons.retain(|b| b.code != code);
                            self.anchor_positions.remove(&code);
                            self.config.buttons.remove(&code);
                            if self.selected_button_code == code {
                                if let Some(first) = self.active_device_profile.buttons.first() {
                                    self.selected_button_code = first.code;
                                }
                            }
                            let _ = self.save_config();
                            self.notify_success(tr(&lang, "editor_btn_deleted"));
                        }
                    }
                }

                // Floating editor action buttons in bottom right of illustration space
                if self.config.general.mode_editor {
                    let btn_w = 160.0;
                    let btn_h = 32.0;
                    let spacing = 8.0;
                    let margin_x = 24.0;
                    let margin_y = 18.0;

                    let is_custom_image = self.active_device_profile.image != "mx-master-4.webp"
                        && self.active_device_profile.image != "mx-master-2s.webp"
                        && self.active_device_profile.image != "device_model.webp";

                    // 1. "+ New Button" (placed above)
                    let add_btn_y = canvas_rect.max.y - (btn_h * 2.0 + spacing) - margin_y;
                    let add_btn_rect = egui::Rect::from_min_size(
                        egui::pos2(canvas_rect.max.x - btn_w - margin_x, add_btn_y),
                        egui::vec2(btn_w, btn_h),
                    );

                    // Soft drop shadow behind add button
                    ui.painter().rect(
                        add_btn_rect.expand(1.0),
                        egui::Rounding::same(9.0),
                        egui::Color32::from_black_alpha(140),
                        egui::Stroke::NONE,
                    );

                    let add_btn = egui::Button::new(
                        egui::RichText::new(tr(&lang, "editor_add_button"))
                            .size(13.0)
                            .strong()
                            .color(egui::Color32::WHITE),
                    )
                    .fill(egui::Color32::from_rgb(26, 50, 70))
                    .stroke(egui::Stroke::new(1.2_f32, egui::Color32::from_rgb(0, 180, 220)))
                    .rounding(egui::Rounding::same(8.0));

                    ui.allocate_ui_at_rect(add_btn_rect, |ui| {
                        if ui
                            .add_sized(add_btn_rect.size(), add_btn)
                            .on_hover_text(tr(&lang, "editor_add_button_tip"))
                            .clicked()
                        {
                            self.show_add_button_modal = true;
                            let next_code = (275..500)
                                .find(|c| !self.active_device_profile.buttons.iter().any(|b| b.code == *c))
                                .unwrap_or(280);
                            self.new_btn_code = next_code;
                            self.new_btn_name = tr(&lang, "editor_default_button_name").replace("{}", &next_code.to_string());
                            self.new_btn_icon = "🔘".to_string();
                            self.new_btn_side = ButtonCalloutSide::Right;
                            self.editor_detecting = None;
                        }
                    });

                    // 2. Element below: "🖼️ Change Image" (+ optional reset ↺)
                    let change_img_y = canvas_rect.max.y - btn_h - margin_y;
                    let change_row_rect = egui::Rect::from_min_size(
                        egui::pos2(canvas_rect.max.x - btn_w - margin_x, change_img_y),
                        egui::vec2(btn_w, btn_h),
                    );

                    // Soft drop shadow behind change image row
                    ui.painter().rect(
                        change_row_rect.expand(1.0),
                        egui::Rounding::same(9.0),
                        egui::Color32::from_black_alpha(140),
                        egui::Stroke::NONE,
                    );

                    let (pick_w, reset_w, gap) = if is_custom_image {
                        (btn_w - 34.0, 28.0, 6.0)
                    } else {
                        (btn_w, 0.0, 0.0)
                    };

                    let pick_btn_rect = egui::Rect::from_min_size(
                        change_row_rect.min,
                        egui::vec2(pick_w, btn_h),
                    );

                    let change_img_btn = egui::Button::new(
                        egui::RichText::new(tr(&lang, "editor_change_image"))
                            .size(12.5)
                            .strong()
                            .color(egui::Color32::WHITE),
                    )
                    .fill(egui::Color32::from_rgb(24, 38, 48))
                    .stroke(egui::Stroke::new(1.2_f32, egui::Color32::from_rgb(0, 160, 130)))
                    .rounding(egui::Rounding::same(8.0));

                    ui.allocate_ui_at_rect(pick_btn_rect, |ui| {
                        let resp = ui
                            .add_sized(pick_btn_rect.size(), change_img_btn)
                            .on_hover_text(tr(&lang, "editor_change_image_tip"));

                        resp.context_menu(|ui| {
                            if ui.button(tr(&lang, "editor_reset_image")).clicked() {
                                self.reset_device_image(ui.ctx());
                                ui.close_menu();
                            }
                        });

                        if resp.clicked() {
                            if let Some(path) = rfd::FileDialog::new()
                                .set_title(tr(&lang, "editor_pick_image_title"))
                                .add_filter("Images", &["png", "webp", "jpg", "jpeg"])
                                .pick_file()
                            {
                                self.update_device_image(ui.ctx(), &path);
                            }
                        }
                    });

                    if is_custom_image {
                        let reset_btn_rect = egui::Rect::from_min_size(
                            egui::pos2(change_row_rect.min.x + pick_w + gap, change_img_y),
                            egui::vec2(reset_w, btn_h),
                        );

                        let reset_btn = egui::Button::new(
                            egui::RichText::new("↺")
                                .size(13.0)
                                .color(egui::Color32::from_rgb(240, 130, 130)),
                        )
                        .fill(egui::Color32::from_rgb(38, 24, 28))
                        .stroke(egui::Stroke::new(1.0_f32, egui::Color32::from_rgb(120, 50, 60)))
                        .rounding(egui::Rounding::same(8.0));

                        ui.allocate_ui_at_rect(reset_btn_rect, |ui| {
                            if ui
                                .add_sized(reset_btn_rect.size(), reset_btn)
                                .on_hover_text(tr(&lang, "editor_reset_image_tip"))
                                .clicked()
                            {
                                self.reset_device_image(ui.ctx());
                            }
                        });
                    }
                }
            });

        if self.show_add_button_modal {
            self.render_add_button_modal(ctx);
        }
        if self.show_edit_button_modal {
            self.render_edit_button_modal(ctx);
        }
        if self.show_new_device_modal {
            self.render_new_device_modal(ctx);
        }
    }

    fn render_add_button_modal(&mut self, ctx: &egui::Context) {
        let lang = self.lang();
        let modal_title = egui::RichText::new(tr(&lang, "editor_modal_title"))
            .strong()
            .size(15.0)
            .color(egui::Color32::WHITE);

        let mut is_open = self.show_add_button_modal;

        let frame = egui::Frame::none()
            .fill(egui::Color32::from_rgb(22, 24, 30))
            .stroke(egui::Stroke::new(1.5_f32, egui::Color32::from_rgb(0, 215, 175)))
            .rounding(egui::Rounding::same(12.0))
            .shadow(egui::epaint::Shadow {
                offset: egui::vec2(0.0, 4.0),
                blur: 14.0_f32,
                spread: 2.0_f32,
                color: egui::Color32::from_black_alpha(200),
            })
            .inner_margin(egui::Margin::same(20.0));

        let is_detecting = self.editor_detecting == Some(EditorDetectTarget::NewButton);
        let code_already_exists = self.active_device_profile.buttons.iter().any(|b| b.code == self.new_btn_code);
        let is_valid = self.new_btn_code > 0 && !code_already_exists;

        egui::Window::new(modal_title)
            .open(&mut is_open)
            .collapsible(false)
            .resizable(false)
            .anchor(egui::Align2::CENTER_CENTER, egui::vec2(0.0, 0.0))
            .fixed_size(egui::vec2(460.0, 340.0))
            .frame(frame)
            .show(ctx, |ui| {
                ui.add_space(4.0);

                // 1. Evdev Code & Detection
                ui.label(
                    egui::RichText::new(tr(&lang, "editor_modal_code"))
                        .strong()
                        .color(egui::Color32::WHITE),
                );
                ui.add_space(2.0);
                ui.horizontal(|ui| {
                    let mut code_str = if self.new_btn_code == 0 {
                        String::new()
                    } else {
                        self.new_btn_code.to_string()
                    };
                    if ui.add(egui::TextEdit::singleline(&mut code_str).hint_text("ex: 278").desired_width(80.0)).changed() {
                        if let Ok(val) = code_str.trim().parse::<u16>() {
                            self.new_btn_code = val;
                        }
                    }

                    if is_detecting {
                        let detect_active_btn = egui::Button::new(
                            egui::RichText::new(tr(&lang, "editor_detecting_hint"))
                                .small()
                                .strong()
                                .color(egui::Color32::WHITE),
                        )
                        .fill(egui::Color32::from_rgb(180, 40, 40))
                        .rounding(egui::Rounding::same(6.0));

                        if ui.add(detect_active_btn).clicked() {
                            self.editor_detecting = None;
                        }

                        if ui.small_button(tr(&lang, "editor_detecting_cancel")).clicked() {
                            self.editor_detecting = None;
                        }
                    } else {
                        let detect_btn = egui::Button::new(
                            egui::RichText::new(tr(&lang, "editor_detect_by_click"))
                                .small()
                                .strong()
                                .color(egui::Color32::from_rgb(0, 215, 175)),
                        )
                        .fill(egui::Color32::from_rgb(26, 42, 48))
                        .stroke(egui::Stroke::new(1.0_f32, egui::Color32::from_rgb(0, 160, 130)))
                        .rounding(egui::Rounding::same(6.0));

                        if ui.add(detect_btn).on_hover_text(tr(&lang, "detect_button_tooltip")).clicked() {
                            self.editor_detecting = Some(EditorDetectTarget::NewButton);
                        }
                    }
                });

                if code_already_exists {
                    ui.label(
                        egui::RichText::new(tr(&lang, "editor_code_already_used"))
                            .small()
                            .color(egui::Color32::from_rgb(255, 100, 100)),
                    );
                }

                ui.add_space(8.0);

                // 2. Button Display Name
                ui.label(
                    egui::RichText::new(tr(&lang, "editor_modal_name"))
                        .strong()
                        .color(egui::Color32::WHITE),
                );
                ui.add_space(2.0);
                ui.add(
                    egui::TextEdit::singleline(&mut self.new_btn_name)
                        .hint_text(tr(&lang, "editor_name_hint"))
                        .desired_width(ui.available_width()),
                );

                ui.add_space(8.0);

                // 3. Icon Selector & Presets
                ui.label(
                    egui::RichText::new(tr(&lang, "editor_modal_icon"))
                        .strong()
                        .color(egui::Color32::WHITE),
                );
                ui.add_space(2.0);
                ui.horizontal(|ui| {
                    ui.add(egui::TextEdit::singleline(&mut self.new_btn_icon).desired_width(36.0));
                    ui.add_space(4.0);
                    let presets = ["🔘", "🎯", "⚡", "⚙️", "🔄", "🖱️", "🔲", "📐", "🔝", "➕"];
                    for p in presets {
                        let is_active = self.new_btn_icon == p;
                        let btn = egui::Button::new(p).fill(if is_active {
                            egui::Color32::from_rgb(0, 140, 110)
                        } else {
                            egui::Color32::from_rgb(32, 36, 46)
                        });
                        if ui.add(btn).clicked() {
                            self.new_btn_icon = p.to_string();
                        }
                    }
                });

                ui.add_space(8.0);

                // 4. Callout Side
                ui.label(
                    egui::RichText::new(tr(&lang, "editor_modal_side"))
                        .strong()
                        .color(egui::Color32::WHITE),
                );
                ui.add_space(2.0);
                ui.horizontal(|ui| {
                    ui.radio_value(
                        &mut self.new_btn_side,
                        ButtonCalloutSide::Left,
                        tr(&lang, "editor_modal_side_left"),
                    );
                    ui.add_space(10.0);
                    ui.radio_value(
                        &mut self.new_btn_side,
                        ButtonCalloutSide::Right,
                        tr(&lang, "editor_modal_side_right"),
                    );
                });

                ui.add_space(8.0);

                // 5. Hint about anchor
                ui.label(
                    egui::RichText::new(tr(&lang, "editor_modal_hint"))
                        .small()
                        .color(egui::Color32::from_gray(160)),
                );

                ui.add_space(14.0);

                // 6. Action buttons: Cancel and Submit
                ui.horizontal(|ui| {
                    let cancel_btn = egui::Button::new(
                        egui::RichText::new(tr(&lang, "cancel"))
                            .color(egui::Color32::from_gray(180)),
                    )
                    .fill(egui::Color32::from_rgb(34, 34, 40))
                    .rounding(egui::Rounding::same(6.0))
                    .min_size(egui::vec2(90.0, 30.0));

                    if ui.add(cancel_btn).clicked() {
                        self.show_add_button_modal = false;
                        self.editor_detecting = None;
                    }

                    ui.with_layout(egui::Layout::right_to_left(egui::Align::Center), |ui| {
                        let submit_btn = egui::Button::new(
                            egui::RichText::new(tr(&lang, "editor_modal_submit"))
                                .strong()
                                .color(if is_valid { egui::Color32::WHITE } else { egui::Color32::from_gray(140) }),
                        )
                        .fill(if is_valid {
                            egui::Color32::from_rgb(0, 160, 120)
                        } else {
                            egui::Color32::from_rgb(34, 40, 48)
                        })
                        .rounding(egui::Rounding::same(6.0))
                        .min_size(egui::vec2(130.0, 30.0));

                        if ui.add_enabled(is_valid, submit_btn).clicked() {
                            let new_code = self.new_btn_code;
                            let btn_name = if self.new_btn_name.trim().is_empty() {
                                tr(&lang, "editor_default_button_name").replace("{}", &new_code.to_string())
                            } else {
                                self.new_btn_name.trim().to_string()
                            };
                            let btn_icon = if self.new_btn_icon.trim().is_empty() {
                                "🔘".to_string()
                            } else {
                                self.new_btn_icon.trim().to_string()
                            };

                            let new_button = DeviceButtonConfig {
                                code: new_code,
                                cid: None,
                                name_key: None,
                                default_name: btn_name,
                                icon: btn_icon,
                                side: self.new_btn_side,
                                anchor: [0.5, 0.5],
                                badge_y_ratio: None,
                            };

                            self.active_device_profile.buttons.push(new_button);
                            self.anchor_positions.insert(new_code, egui::vec2(0.5, 0.5));
                            self.selected_button_code = new_code;
                            self.show_add_button_modal = false;
                            self.editor_detecting = None;
                            self.notify_success(tr(&lang, "editor_btn_added"));
                        }
                    });
                });
            });

        if !is_open {
            self.show_add_button_modal = false;
            self.editor_detecting = None;
        }
    }

    fn render_edit_button_modal(&mut self, ctx: &egui::Context) {
        let lang = self.lang();
        let modal_title = egui::RichText::new(tr(&lang, "editor_edit_modal_title"))
            .strong()
            .size(15.0)
            .color(egui::Color32::WHITE);

        let mut is_open = self.show_edit_button_modal;

        let frame = egui::Frame::none()
            .fill(egui::Color32::from_rgb(22, 24, 30))
            .stroke(egui::Stroke::new(1.5_f32, egui::Color32::from_rgb(0, 215, 175)))
            .rounding(egui::Rounding::same(12.0))
            .shadow(egui::epaint::Shadow {
                offset: egui::vec2(0.0, 4.0),
                blur: 14.0_f32,
                spread: 2.0_f32,
                color: egui::Color32::from_black_alpha(200),
            })
            .inner_margin(egui::Margin::same(20.0));

        let is_detecting = self.editor_detecting == Some(EditorDetectTarget::EditModalButton);
        let code_collision = self.edit_btn_code != self.edit_btn_old_code
            && self.active_device_profile.buttons.iter().any(|b| b.code == self.edit_btn_code);
        let is_valid = self.edit_btn_code > 0 && !code_collision;

        egui::Window::new(modal_title)
            .open(&mut is_open)
            .collapsible(false)
            .resizable(false)
            .anchor(egui::Align2::CENTER_CENTER, egui::vec2(0.0, 0.0))
            .fixed_size(egui::vec2(480.0, 410.0))
            .frame(frame)
            .show(ctx, |ui| {
                ui.add_space(4.0);

                // 1. Evdev Code & Detection
                ui.label(
                    egui::RichText::new(tr(&lang, "editor_modal_code"))
                        .strong()
                        .color(egui::Color32::WHITE),
                );
                ui.add_space(2.0);
                ui.horizontal(|ui| {
                    let mut code_str = if self.edit_btn_code == 0 {
                        String::new()
                    } else {
                        self.edit_btn_code.to_string()
                    };
                    if ui.add(egui::TextEdit::singleline(&mut code_str).hint_text("ex: 278").desired_width(80.0)).changed() {
                        if let Ok(val) = code_str.trim().parse::<u16>() {
                            self.edit_btn_code = val;
                        }
                    }

                    if is_detecting {
                        let detect_active_btn = egui::Button::new(
                            egui::RichText::new(tr(&lang, "editor_detecting_hint"))
                                .small()
                                .strong()
                                .color(egui::Color32::WHITE),
                        )
                        .fill(egui::Color32::from_rgb(180, 40, 40))
                        .rounding(egui::Rounding::same(6.0));

                        if ui.add(detect_active_btn).clicked() {
                            self.editor_detecting = None;
                        }

                        if ui.small_button(tr(&lang, "editor_detecting_cancel")).clicked() {
                            self.editor_detecting = None;
                        }
                    } else {
                        let detect_btn = egui::Button::new(
                            egui::RichText::new(tr(&lang, "editor_detect_by_click"))
                                .small()
                                .strong()
                                .color(egui::Color32::from_rgb(0, 215, 175)),
                        )
                        .fill(egui::Color32::from_rgb(26, 42, 48))
                        .stroke(egui::Stroke::new(1.0_f32, egui::Color32::from_rgb(0, 160, 130)))
                        .rounding(egui::Rounding::same(6.0));

                        if ui.add(detect_btn).on_hover_text(tr(&lang, "detect_button_tooltip")).clicked() {
                            self.editor_detecting = Some(EditorDetectTarget::EditModalButton);
                        }
                    }
                });

                if code_collision {
                    ui.label(
                        egui::RichText::new(tr(&lang, "editor_code_already_used"))
                            .small()
                            .color(egui::Color32::from_rgb(255, 100, 100)),
                    );
                }

                ui.add_space(8.0);

                // 2. Button Display Name
                ui.label(
                    egui::RichText::new(tr(&lang, "editor_modal_name"))
                        .strong()
                        .color(egui::Color32::WHITE),
                );
                ui.add_space(2.0);
                ui.add(
                    egui::TextEdit::singleline(&mut self.edit_btn_name)
                        .hint_text(tr(&lang, "editor_name_hint"))
                        .desired_width(ui.available_width()),
                );

                ui.add_space(8.0);

                // 3. Icon Selector & Presets
                ui.label(
                    egui::RichText::new(tr(&lang, "editor_modal_icon"))
                        .strong()
                        .color(egui::Color32::WHITE),
                );
                ui.add_space(2.0);
                ui.horizontal(|ui| {
                    ui.add(egui::TextEdit::singleline(&mut self.edit_btn_icon).desired_width(36.0));
                    ui.add_space(4.0);
                    let presets = ["🔘", "🎯", "⚡", "⚙️", "🔄", "🖱️", "🔲", "📐", "🔝", "➕"];
                    for p in presets {
                        let is_active = self.edit_btn_icon == p;
                        let btn = egui::Button::new(p).fill(if is_active {
                            egui::Color32::from_rgb(0, 140, 110)
                        } else {
                            egui::Color32::from_rgb(32, 36, 46)
                        });
                        if ui.add(btn).clicked() {
                            self.edit_btn_icon = p.to_string();
                        }
                    }
                });

                ui.add_space(8.0);

                // 4. Callout Side
                ui.label(
                    egui::RichText::new(tr(&lang, "editor_modal_side"))
                        .strong()
                        .color(egui::Color32::WHITE),
                );
                ui.add_space(2.0);
                ui.horizontal(|ui| {
                    ui.radio_value(
                        &mut self.edit_btn_side,
                        ButtonCalloutSide::Left,
                        tr(&lang, "editor_modal_side_left"),
                    );
                    ui.add_space(10.0);
                    ui.radio_value(
                        &mut self.edit_btn_side,
                        ButtonCalloutSide::Right,
                        tr(&lang, "editor_modal_side_right"),
                    );
                });

                ui.add_space(8.0);

                // 5. Vertical Height Ratio Slider
                ui.label(
                    egui::RichText::new(tr(&lang, "editor_modal_ratio"))
                        .strong()
                        .color(egui::Color32::WHITE),
                );
                ui.add_space(2.0);
                ui.horizontal(|ui| {
                    ui.add_sized(
                        [260.0, 20.0],
                        egui::Slider::new(&mut self.edit_btn_ratio, 0.05..=0.95)
                            .custom_formatter(|n, _| format!("{:.0}%", n * 100.0)),
                    );
                    ui.label(
                        egui::RichText::new(format!("({:.2})", self.edit_btn_ratio))
                            .monospace()
                            .small()
                            .color(egui::Color32::from_gray(140)),
                    );
                });

                ui.add_space(8.0);

                // 6. Anchor info and recenter
                let curr_anchor = self.anchor_positions.get(&self.edit_btn_old_code).cloned().unwrap_or(egui::vec2(0.5, 0.5));
                ui.horizontal(|ui| {
                    ui.label(
                        egui::RichText::new(format!("📍 Ancre : [ {:.3}, {:.3} ]", curr_anchor.x, curr_anchor.y))
                            .monospace()
                            .small()
                            .color(egui::Color32::from_rgb(0, 215, 175)),
                    );
                    if ui.small_button(tr(&lang, "editor_recentre_anchor")).clicked() {
                        self.anchor_positions.insert(self.edit_btn_old_code, egui::vec2(0.5, 0.5));
                    }
                });

                ui.add_space(14.0);

                // 7. Action buttons: Cancel and Submit
                ui.horizontal(|ui| {
                    let cancel_btn = egui::Button::new(
                        egui::RichText::new(tr(&lang, "cancel"))
                            .color(egui::Color32::from_gray(180)),
                    )
                    .fill(egui::Color32::from_rgb(34, 34, 40))
                    .rounding(egui::Rounding::same(6.0))
                    .min_size(egui::vec2(80.0, 30.0));

                    if ui.add(cancel_btn).clicked() {
                        self.show_edit_button_modal = false;
                        self.editor_detecting = None;
                    }

                    let can_delete = self.active_device_profile.buttons.len() > 1;
                    let del_modal_btn = egui::Button::new(
                        egui::RichText::new(format!("🗑️ {}", tr(&lang, "editor_delete_button")))
                            .small()
                            .color(if can_delete { egui::Color32::from_rgb(255, 130, 130) } else { egui::Color32::from_gray(100) }),
                    )
                    .fill(egui::Color32::from_rgb(40, 24, 28))
                    .stroke(egui::Stroke::new(1.0_f32, egui::Color32::from_rgb(110, 45, 50)))
                    .rounding(egui::Rounding::same(6.0))
                    .min_size(egui::vec2(120.0, 30.0));

                    if ui
                        .add_enabled(can_delete, del_modal_btn)
                        .on_hover_text(if can_delete {
                            tr(&lang, "editor_delete_button_tip")
                        } else {
                            tr(&lang, "editor_cannot_delete_last")
                        })
                        .clicked()
                    {
                        let old_code = self.edit_btn_old_code;
                        self.active_device_profile.buttons.retain(|b| b.code != old_code);
                        self.anchor_positions.remove(&old_code);
                        self.config.buttons.remove(&old_code);
                        if self.selected_button_code == old_code {
                            if let Some(first) = self.active_device_profile.buttons.first() {
                                self.selected_button_code = first.code;
                            }
                        }
                        let _ = self.save_config();
                        self.show_edit_button_modal = false;
                        self.editor_detecting = None;
                        self.notify_success(tr(&lang, "editor_btn_deleted"));
                    }

                    ui.with_layout(egui::Layout::right_to_left(egui::Align::Center), |ui| {
                        let submit_btn = egui::Button::new(
                            egui::RichText::new(tr(&lang, "editor_edit_modal_submit"))
                                .strong()
                                .color(if is_valid { egui::Color32::WHITE } else { egui::Color32::from_gray(140) }),
                        )
                        .fill(if is_valid {
                            egui::Color32::from_rgb(0, 160, 120)
                        } else {
                            egui::Color32::from_rgb(34, 40, 48)
                        })
                        .rounding(egui::Rounding::same(6.0))
                        .min_size(egui::vec2(160.0, 30.0));

                        if ui.add_enabled(is_valid, submit_btn).clicked() {
                            let old_code = self.edit_btn_old_code;
                            let new_code = self.edit_btn_code;

                            // Handle code migration if code changed
                            if new_code != old_code {
                                if let Some(anchor) = self.anchor_positions.remove(&old_code) {
                                    self.anchor_positions.insert(new_code, anchor);
                                }
                                if let Some(action) = self.config.buttons.remove(&old_code) {
                                    self.config.buttons.insert(new_code, action);
                                    let _ = self.save_config();
                                }
                                self.selected_button_code = new_code;
                            }

                            let btn_name = if self.edit_btn_name.trim().is_empty() {
                                tr(&lang, "editor_default_button_name").replace("{}", &new_code.to_string())
                            } else {
                                self.edit_btn_name.trim().to_string()
                            };
                            let btn_icon = if self.edit_btn_icon.trim().is_empty() {
                                "🔘".to_string()
                            } else {
                                self.edit_btn_icon.trim().to_string()
                            };

                            if let Some(btn) = self.active_device_profile.buttons.iter_mut().find(|b| b.code == old_code) {
                                btn.code = new_code;
                                btn.name_key = None;
                                btn.default_name = btn_name;
                                btn.icon = btn_icon;
                                btn.side = self.edit_btn_side;
                                btn.badge_y_ratio = Some((self.edit_btn_ratio * 100.0).round() / 100.0);
                            }

                            self.show_edit_button_modal = false;
                            self.editor_detecting = None;
                            self.notify_success(tr(&lang, "editor_btn_updated"));
                        }
                    });
                });
            });

        if !is_open {
            self.show_edit_button_modal = false;
            self.editor_detecting = None;
        }
    }

    fn render_new_device_modal(&mut self, ctx: &egui::Context) {
        let lang = self.lang();
        let modal_title = egui::RichText::new(tr(&lang, "new_device_modal_title"))
            .strong()
            .size(15.0)
            .color(egui::Color32::WHITE);

        let mut is_open = self.show_new_device_modal;

        let frame = egui::Frame::none()
            .fill(egui::Color32::from_rgb(22, 24, 30))
            .stroke(egui::Stroke::new(1.5_f32, egui::Color32::from_rgb(0, 215, 175)))
            .rounding(egui::Rounding::same(12.0))
            .shadow(egui::epaint::Shadow {
                offset: egui::vec2(0.0, 4.0),
                blur: 14.0_f32,
                spread: 2.0_f32,
                color: egui::Color32::from_black_alpha(200),
            })
            .inner_margin(egui::Margin::same(20.0));

        let trimmed_name = self.new_device_name.trim();
        let trimmed_id = self.new_device_id.trim();
        let is_empty = trimmed_name.is_empty() || trimmed_id.is_empty();
        let id_already_exists = !trimmed_id.is_empty()
            && self
                .device_registry
                .list_profiles()
                .iter()
                .any(|p| p.config.id.eq_ignore_ascii_case(trimmed_id));
        let is_valid = !is_empty && !id_already_exists;

        egui::Window::new(modal_title)
            .open(&mut is_open)
            .collapsible(false)
            .resizable(false)
            .anchor(egui::Align2::CENTER_CENTER, egui::vec2(0.0, 0.0))
            .fixed_size(egui::vec2(520.0, 430.0))
            .frame(frame)
            .show(ctx, |ui| {
                ui.add_space(4.0);

                // 1. Model Name
                ui.label(
                    egui::RichText::new(tr(&lang, "new_device_name_label"))
                        .strong()
                        .color(egui::Color32::WHITE),
                );
                ui.add_space(2.0);
                if ui
                    .add(
                        egui::TextEdit::singleline(&mut self.new_device_name)
                            .hint_text(tr(&lang, "new_device_name_hint"))
                            .desired_width(ui.available_width()),
                    )
                    .changed()
                {
                    if !self.new_device_id_edited {
                        self.new_device_id = crate::config::slugify(&self.new_device_name);
                    }
                }

                ui.add_space(8.0);

                // 2. Technical Identifier (JSON filename)
                ui.label(
                    egui::RichText::new(tr(&lang, "new_device_id_label"))
                        .strong()
                        .color(egui::Color32::WHITE),
                );
                ui.add_space(2.0);
                if ui
                    .add(
                        egui::TextEdit::singleline(&mut self.new_device_id)
                            .hint_text("ex: logitech_g502_x")
                            .desired_width(ui.available_width()),
                    )
                    .changed()
                {
                    self.new_device_id_edited = true;
                }

                if id_already_exists {
                    ui.add_space(2.0);
                    ui.label(
                        egui::RichText::new(tr(&lang, "new_device_error_exists"))
                            .small()
                            .color(egui::Color32::from_rgb(255, 100, 100)),
                    );
                }

                ui.add_space(8.0);

                // 3. Hardware Detection (VID:PID)
                ui.label(
                    egui::RichText::new(tr(&lang, "hardware_id_label"))
                        .strong()
                        .color(egui::Color32::WHITE),
                );
                ui.add_space(3.0);

                let is_detecting_hw = self.editor_detecting == Some(EditorDetectTarget::NewDeviceHardware);
                ui.horizontal(|ui| {
                    if is_detecting_hw {
                        let detect_btn = egui::Button::new(
                            egui::RichText::new(tr(&lang, "hardware_id_detecting"))
                                .size(12.0)
                                .color(egui::Color32::from_rgb(255, 215, 0))
                                .strong(),
                        )
                        .fill(egui::Color32::from_rgb(45, 40, 20))
                        .stroke(egui::Stroke::new(1.0_f32, egui::Color32::from_rgb(255, 200, 0)))
                        .rounding(egui::Rounding::same(6.0))
                        .min_size(egui::vec2(280.0, 26.0));

                        ui.add(detect_btn).on_hover_text(tr(&lang, "hardware_id_detect_tip"));

                        let cancel_detect_btn = egui::Button::new(
                            egui::RichText::new("❌")
                                .size(12.0)
                                .color(egui::Color32::from_rgb(255, 120, 120)),
                        )
                        .fill(egui::Color32::from_rgb(40, 26, 26))
                        .rounding(egui::Rounding::same(6.0))
                        .min_size(egui::vec2(28.0, 26.0));

                        if ui.add(cancel_detect_btn).on_hover_text(tr(&lang, "hardware_id_cancel_detect")).clicked() {
                            self.editor_detecting = None;
                        }
                    } else {
                        let detect_btn = egui::Button::new(
                            egui::RichText::new(tr(&lang, "hardware_id_detect_btn"))
                                .size(12.5)
                                .color(egui::Color32::WHITE)
                                .strong(),
                        )
                        .fill(egui::Color32::from_rgb(24, 38, 48))
                        .stroke(egui::Stroke::new(1.0_f32, egui::Color32::from_rgb(0, 180, 220)))
                        .rounding(egui::Rounding::same(6.0))
                        .min_size(egui::vec2(190.0, 26.0));

                        if ui
                            .add(detect_btn)
                            .on_hover_text(tr(&lang, "hardware_id_detect_tip"))
                            .clicked()
                        {
                            self.editor_detecting = Some(EditorDetectTarget::NewDeviceHardware);
                        }
                    }

                    // Display detected match_ids as badges
                    if !self.new_device_match_ids.is_empty() {
                        let mut to_remove = None;
                        for (idx, id) in self.new_device_match_ids.iter().enumerate() {
                            let badge = egui::Frame::none()
                                .fill(egui::Color32::from_rgb(26, 36, 44))
                                .stroke(egui::Stroke::new(1.0_f32, egui::Color32::from_rgb(0, 170, 140)))
                                .rounding(egui::Rounding::same(4.0))
                                .inner_margin(egui::Margin::symmetric(6.0, 3.0));
                            badge.show(ui, |ui| {
                                ui.horizontal(|ui| {
                                    ui.label(egui::RichText::new(id).monospace().size(11.5).color(egui::Color32::WHITE));
                                    if ui.small_button("✖").clicked() {
                                        to_remove = Some(idx);
                                    }
                                });
                            });
                        }
                        if let Some(idx) = to_remove {
                            self.new_device_match_ids.remove(idx);
                        }
                    } else {
                        ui.label(
                            egui::RichText::new(tr(&lang, "hardware_id_none"))
                                .small()
                                .italics()
                                .color(egui::Color32::from_gray(140)),
                        );
                    }
                });

                ui.add_space(8.0);

                // 4. Starting Base
                ui.label(
                    egui::RichText::new(tr(&lang, "new_device_base_label"))
                        .strong()
                        .color(egui::Color32::WHITE),
                );
                ui.add_space(2.0);
                let duplicate_label = tr(&lang, "new_device_base_duplicate")
                    .replace("{}", &self.active_device_profile.name);
                ui.radio_value(&mut self.new_device_duplicate_active, true, duplicate_label);
                ui.add_space(2.0);
                ui.radio_value(
                    &mut self.new_device_duplicate_active,
                    false,
                    tr(&lang, "new_device_base_generic"),
                );

                ui.add_space(16.0);

                // 5. Action buttons: Cancel and Submit
                ui.horizontal(|ui| {
                    let cancel_btn = egui::Button::new(
                        egui::RichText::new(tr(&lang, "cancel"))
                            .color(egui::Color32::from_gray(180)),
                    )
                    .fill(egui::Color32::from_rgb(34, 34, 40))
                    .rounding(egui::Rounding::same(6.0))
                    .min_size(egui::vec2(90.0, 30.0));

                    if ui.add(cancel_btn).clicked() {
                        self.show_new_device_modal = false;
                        if self.editor_detecting == Some(EditorDetectTarget::NewDeviceHardware) {
                            self.editor_detecting = None;
                        }
                    }

                    ui.with_layout(egui::Layout::right_to_left(egui::Align::Center), |ui| {
                        let submit_btn = egui::Button::new(
                            egui::RichText::new(tr(&lang, "new_device_submit"))
                                .strong()
                                .color(if is_valid {
                                    egui::Color32::WHITE
                                } else {
                                    egui::Color32::from_gray(140)
                                }),
                        )
                        .fill(if is_valid {
                            egui::Color32::from_rgb(0, 160, 120)
                        } else {
                            egui::Color32::from_rgb(34, 40, 48)
                        })
                        .rounding(egui::Rounding::same(6.0))
                        .min_size(egui::vec2(160.0, 30.0));

                        if ui.add_enabled(is_valid, submit_btn).clicked() {
                            let id = self.new_device_id.trim().to_string();
                            let name = self.new_device_name.trim().to_string();

                            let new_model = if self.new_device_duplicate_active {
                                let mut base = self.active_device_profile.clone();
                                base.id = id.clone();
                                base.name = name.clone();
                                base.match_names = vec![name.clone()];
                                base.match_ids = self.new_device_match_ids.clone();
                                for btn in &mut base.buttons {
                                    if let Some(pos) = self.anchor_positions.get(&btn.code) {
                                        btn.anchor = [
                                            (pos.x * 1000.0).round() / 1000.0,
                                            (pos.y * 1000.0).round() / 1000.0,
                                        ];
                                    }
                                }
                                base
                            } else {
                                let generic = self
                                    .device_registry
                                    .get_profile("generic")
                                    .map(|p| p.config.clone())
                                    .unwrap_or_else(|| self.active_device_profile.clone());

                                DeviceModelConfig {
                                    id: id.clone(),
                                    name: name.clone(),
                                    author: None,
                                    match_names: vec![name.clone()],
                                    match_ids: self.new_device_match_ids.clone(),
                                    image: generic.image.clone(),
                                    buttons: generic.buttons.clone(),
                                }
                            };

                            if let Ok(json_str) = serde_json::to_string_pretty(&new_model) {
                                let dest_dir = DeviceRegistry::ensure_user_devices_dir();
                                let dest_file = dest_dir.join(format!("{}.json", id));
                                if std::fs::write(&dest_file, &json_str).is_ok() {
                                    self.device_registry = DeviceRegistry::new();
                                    self.select_device_profile(ctx, &id);
                                    self.config.general.mode_editor = true;
                                    let _ = self.save_config();
                                    self.show_new_device_modal = false;
                                    self.editor_detecting = None;
                                    let msg = tr(&lang, "new_device_created").replace("{}", &name);
                                    self.notify_success(msg);
                                }
                            }
                        }
                    });
                });
            });

        if !is_open {
            self.show_new_device_modal = false;
            if self.editor_detecting == Some(EditorDetectTarget::NewDeviceHardware) {
                self.editor_detecting = None;
            }
        }
    }
}

