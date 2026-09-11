use std::f32::consts::PI;

use crate::config::{
    contrasting_text_color, parse_hex_color, ButtonActionConfig, RingAnimation,
    RingMenuItemConfig, RingTriggerMode, SubMenuItemConfig,
};
use crate::i18n::tr;
use eframe::egui;

use super::types::{ConfigApp, SelectedNodePath, ShortcutTarget};
use super::widgets::{
    render_color_editor, render_color_editor_with_action, render_icon_picker,
    render_node_action_selector, render_type_selector, ElementTypeChoice,
};

impl ConfigApp {
    /// Render Tab 2: Action Ring Customization
    pub(super) fn render_ring_customizer_tab(&mut self, ctx: &egui::Context) {
        self.ensure_active_profile_ring_menu();
        let lang = self.lang();
        let mut node_to_delete: Option<SelectedNodePath> = None;
        let sel_node = self.selected_node.clone();
        let def_norm = self.active_ring_menu().default_color.clone();
        let def_act = self.active_ring_menu().default_active_color.clone();

        egui::SidePanel::right("right_ring_inspector")
            .exact_width(380.0)
            .frame(egui::Frame::none().fill(egui::Color32::from_rgb(20, 20, 22)).inner_margin(16.0))
            .show(ctx, |ui| {
                egui::ScrollArea::vertical().show(ui, |ui| {
                    ui.add_space(10.0);

                    ui.horizontal(|ui| {
                        ui.heading(egui::RichText::new(tr(&lang, "slot_editor_header")).strong().color(egui::Color32::WHITE));
                        ui.with_layout(egui::Layout::right_to_left(egui::Align::Center), |ui| {
                            let menu = self.active_ring_menu();
                            ui.label(
                                egui::RichText::new(format!("{}/{} slots", menu.items.len(), menu.max_slots.max(1)))
                                    .small()
                                    .strong()
                                    .color(egui::Color32::from_gray(140))
                            );
                        });
                    });

                    ui.add_space(15.0);

                    // Section 1: Ring Menu Colors & Settings for active profile
                    ui.group(|ui| {
                        ui.set_width(ui.available_width());
                        let profile_label = if self.active_profile_index == 0 {
                            format!("🌐 {}", tr(&lang, "profile_global_label"))
                        } else {
                            format!("🎨 {}", self.app_profiles.get(self.active_profile_index - 1).map(|p| p.label.as_str()).unwrap_or("App"))
                        };
                        ui.label(egui::RichText::new(format!("{} — {}", tr(&lang, "global_colors_header"), profile_label)).small().strong().color(egui::Color32::from_rgb(0, 215, 175)));
                        ui.add_space(6.0);

                        let def_norm = self.active_ring_menu().default_color.clone();
                        let def_act = self.active_ring_menu().default_active_color.clone();

                        let mut norm_opt = Some(def_norm);
                        render_color_editor(ui, &lang, tr(&lang, "idle_color"), &mut norm_opt, &crate::config::default_ring_color());
                        if let Some(c) = norm_opt {
                            self.active_ring_menu_mut().default_color = c;
                        }

                        ui.add_space(4.0);

                        let mut act_opt = Some(def_act);
                        render_color_editor_with_action(ui, &lang, tr(&lang, "active_hover_color"), &mut act_opt, &crate::config::default_ring_active_color(), |ui| {
                            let os_btn = egui::Button::new(
                                egui::RichText::new("🎯 OS")
                                    .small()
                                    .strong()
                                    .color(egui::Color32::from_rgb(0, 215, 175))
                            )
                            .fill(egui::Color32::from_rgb(26, 34, 38))
                            .stroke(egui::Stroke::new(1.0_f32, egui::Color32::from_rgb(0, 215, 175)))
                            .rounding(egui::Rounding::same(6.0));

                            if ui.add(os_btn).on_hover_text(tr(&lang, "os_color_tooltip")).clicked() {
                                if let Some(os_color) = crate::config::detect_os_accent_color() {
                                    self.active_ring_menu_mut().default_active_color = os_color.clone();
                                    self.status_message = format!("🎨 {}", os_color);
                                } else {
                                    self.status_message = tr(&lang, "status_error_detecting_color").to_string();
                                }
                            }
                        });
                        if let Some(c) = act_opt {
                            self.active_ring_menu_mut().default_active_color = c;
                        }

                        if self.config.general.dev_mode {
                            ui.add_space(8.0);
                            ui.horizontal(|ui| {
                                ui.label(egui::RichText::new(tr(&lang, "max_slots_label")).small().strong());
                                let mut max_s = self.active_ring_menu().max_slots as u32;
                                if ui.add(egui::DragValue::new(&mut max_s).range(4..=32)).changed() {
                                    self.active_ring_menu_mut().max_slots = max_s as usize;
                                }
                            });
                            ui.label(egui::RichText::new(tr(&lang, "max_slots_desc")).small().weak());
                        }

                        ui.add_space(8.0);
                        let anim_label = self.config.ring_menu.animation.label_lang(&lang);
                        ui.horizontal(|ui| {
                            ui.label(egui::RichText::new(tr(&lang, "animation_style")).small().strong());
                            egui::ComboBox::from_id_source("ring_animation_select")
                                .selected_text(anim_label)
                                .width(150.0)
                                .show_ui(ui, |ui| {
                                    for anim in RingAnimation::all() {
                                        let mut current_anim = self.config.ring_menu.animation.clone();
                                        if ui.selectable_value(
                                            &mut current_anim,
                                            anim.clone(),
                                            anim.label_lang(&lang),
                                        ).changed() {
                                            self.config.ring_menu.animation = anim.clone();
                                        }
                                    }
                                });
                        });

                        ui.add_space(8.0);
                        let mut hold_enabled = self.config.ring_menu.trigger_mode != RingTriggerMode::Click;
                        if ui.checkbox(&mut hold_enabled, tr(&lang, "trigger_mode_label")).changed() {
                            self.config.ring_menu.trigger_mode = if hold_enabled {
                                RingTriggerMode::Hybrid
                            } else {
                                RingTriggerMode::Click
                            };
                        }
                        ui.label(egui::RichText::new(tr(&lang, "hold_to_release_desc")).small().weak());

                        ui.add_space(8.0);
                        ui.checkbox(
                            &mut self.config.ring_menu.wheel_navigation,
                            tr(&lang, "wheel_navigation_label"),
                        );
                        ui.label(egui::RichText::new(tr(&lang, "wheel_navigation_desc")).small().weak());

                        ui.add_space(8.0);
                        ui.checkbox(
                            &mut self.config.ring_menu.glow_effect,
                            tr(&lang, "glow_effect_label"),
                        );
                        ui.label(egui::RichText::new(tr(&lang, "glow_effect_desc")).small().weak());

                        ui.add_space(8.0);
                        ui.checkbox(
                            &mut self.config.ring_menu.pulse_effect,
                            tr(&lang, "pulse_effect_label"),
                        );
                        ui.label(egui::RichText::new(tr(&lang, "pulse_effect_desc")).small().weak());
                    });
                    ui.add_space(15.0);

                    // Reusable component to render action selector
                    let current_target = ShortcutTarget::RingNode(self.selected_node.clone());
                    let is_recording_this_node = self.recording_shortcut_for == Some(current_target.clone());

                    let active_profile_idx = self.active_profile_index;
                    let sel_node = self.selected_node.clone();
                    match &sel_node {
                        SelectedNodePath::Slot(s_idx) => {
                            let slot_opt = if active_profile_idx == 0 {
                                self.config.ring_menu.items.get_mut(*s_idx)
                            } else {
                                self.app_profiles.get_mut(active_profile_idx.saturating_sub(1))
                                    .and_then(|p| p.ring_menu.as_mut())
                                    .and_then(|m| m.items.get_mut(*s_idx))
                            };
                            if let Some(slot) = slot_opt {
                                ui.group(|ui| {
                                    ui.set_width(ui.available_width());
                                    ui.horizontal(|ui| {
                                        ui.label(egui::RichText::new(format!("{}{}", tr(&lang, "main_slot_prefix"), slot.slot)).strong().color(egui::Color32::from_rgb(0, 215, 175)));
                                        ui.with_layout(egui::Layout::right_to_left(egui::Align::Center), |ui| {
                                            let del_btn = egui::Button::new(
                                                egui::RichText::new("🗑")
                                                    .small()
                                                    .strong()
                                                    .color(egui::Color32::from_rgb(255, 110, 110))
                                            )
                                            .fill(egui::Color32::from_rgb(42, 22, 26))
                                            .stroke(egui::Stroke::new(1.0_f32, egui::Color32::from_rgb(120, 45, 50)))
                                            .rounding(egui::Rounding::same(6.0));

                                            if ui.add(del_btn).on_hover_text(tr(&lang, "delete_button")).clicked() {
                                                node_to_delete = Some(SelectedNodePath::Slot(*s_idx));
                                            }
                                        });
                                    });
                                    ui.add_space(8.0);

                                    ui.horizontal(|ui| {
                                        ui.label(egui::RichText::new(tr(&lang, "quick_preset_label")).small().strong().color(egui::Color32::from_rgb(0, 215, 175)));
                                        egui::ComboBox::from_id_source(("slot_quick_preset", *s_idx))
                                            .selected_text(tr(&lang, "choose_catalog_placeholder"))
                                            .width(ui.available_width() - 10.0)
                                            .show_ui(ui, |ui| {
                                                let actions = crate::config::actions::default_actions_catalog_for_lang(&lang);
                                                for action in actions {
                                                    let text = format!("{} {}", action.icon, action.label_lang(&lang));
                                                    if ui.button(text).clicked() {
                                                        slot.label = action.label_lang(&lang).to_string();
                                                        slot.icon = action.icon.clone();
                                                        slot.action = Some(action.action.clone());
                                                        slot.items.clear();
                                                    }
                                                }
                                            });
                                    });
                                    ui.add_space(8.0);

                                    ui.horizontal(|ui| {
                                        ui.label(tr(&lang, "label_title"));
                                        ui.add(egui::TextEdit::singleline(&mut slot.label).desired_width(200.0));
                                    });
                                    ui.add_space(8.0);

                                    render_icon_picker(ui, ctx, &lang, &mut self.texture_cache, &mut slot.icon, ("slot_icon", *s_idx));

                                    let is_submenu = !slot.items.is_empty();
                                    match render_type_selector(ui, &lang, is_submenu) {
                                        Some(ElementTypeChoice::DirectAction) => {
                                            slot.items.clear();
                                            if slot.action.is_none() {
                                                slot.action = Some(ButtonActionConfig::Command { cmd: "firefox".to_string() });
                                            }
                                        }
                                        Some(ElementTypeChoice::SubMenu) => {
                                            slot.items.push(SubMenuItemConfig {
                                                label: "Sub-item 1".to_string(),
                                                translations: std::collections::HashMap::new(),
                                                icon: "⚡".to_string(),
                                                action: Some(ButtonActionConfig::Command { cmd: "firefox".to_string() }),
                                                items: vec![],
                                                auto_close: true,
                                                color: None,
                                                active_color: None,
                                            });
                                        }
                                        None => {}
                                    }

                                    ui.add_space(12.0);

                                    if is_submenu {
                                        ui.label(egui::RichText::new(tr(&lang, "contains_sub_items").replace("{}", &slot.items.len().to_string())).color(egui::Color32::from_rgb(0, 215, 175)));
                                    } else {
                                        render_node_action_selector(ui, &lang, &mut slot.action, &mut self.custom_cmd_input, is_recording_this_node, || {
                                            self.recording_shortcut_for = if is_recording_this_node { None } else { Some(current_target.clone()) };
                                        });
                                        ui.add_space(8.0);
                                        ui.checkbox(&mut slot.auto_close, tr(&lang, "auto_close_label"));
                                    }

                                    ui.add_space(12.0);
                                    ui.separator();
                                    ui.add_space(8.0);
                                    ui.label(egui::RichText::new(tr(&lang, "unit_colors_header")).small().strong().color(egui::Color32::from_gray(140)));
                                    render_color_editor(ui, &lang, tr(&lang, "idle_color"), &mut slot.color, &def_norm);
                                    render_color_editor(ui, &lang, tr(&lang, "active_hover_color"), &mut slot.active_color, &def_act);
                                });
                            }
                        }

                        SelectedNodePath::SubItem(s_idx, sub1_idx) => {
                            let sub_opt = if active_profile_idx == 0 {
                                self.config.ring_menu.items.get_mut(*s_idx).and_then(|s| s.items.get_mut(*sub1_idx))
                            } else {
                                self.app_profiles.get_mut(active_profile_idx.saturating_sub(1))
                                    .and_then(|p| p.ring_menu.as_mut())
                                    .and_then(|m| m.items.get_mut(*s_idx))
                                    .and_then(|s| s.items.get_mut(*sub1_idx))
                            };
                            if let Some(sub) = sub_opt {
                                    ui.group(|ui| {
                                        ui.set_width(ui.available_width());
                                        ui.horizontal(|ui| {
                                            ui.label(egui::RichText::new(tr(&lang, "sub_item_level_1").replace("{}", &(sub1_idx + 1).to_string())).strong().color(egui::Color32::from_rgb(0, 215, 175)));
                                            ui.with_layout(egui::Layout::right_to_left(egui::Align::Center), |ui| {
                                                let del_btn = egui::Button::new(
                                                    egui::RichText::new("🗑")
                                                        .small()
                                                        .strong()
                                                        .color(egui::Color32::from_rgb(255, 110, 110))
                                                )
                                                .fill(egui::Color32::from_rgb(42, 22, 26))
                                                .stroke(egui::Stroke::new(1.0_f32, egui::Color32::from_rgb(120, 45, 50)))
                                                .rounding(egui::Rounding::same(6.0));

                                                if ui.add(del_btn).on_hover_text(tr(&lang, "delete_button")).clicked() {
                                                    node_to_delete = Some(SelectedNodePath::SubItem(*s_idx, *sub1_idx));
                                                }
                                            });
                                        });
                                        ui.add_space(8.0);

                                        ui.horizontal(|ui| {
                                            ui.label(tr(&lang, "label_title"));
                                            ui.add(egui::TextEdit::singleline(&mut sub.label).desired_width(200.0));
                                        });
                                        ui.add_space(8.0);

                                        render_icon_picker(ui, ctx, &lang, &mut self.texture_cache, &mut sub.icon, ("sub1_icon", *s_idx, *sub1_idx));

                                        let is_nested_submenu = !sub.items.is_empty();
                                        match render_type_selector(ui, &lang, is_nested_submenu) {
                                            Some(ElementTypeChoice::DirectAction) => {
                                                sub.items.clear();
                                                if sub.action.is_none() {
                                                    sub.action = Some(ButtonActionConfig::Command { cmd: "firefox".to_string() });
                                                }
                                            }
                                            Some(ElementTypeChoice::SubMenu) => {
                                                sub.items.push(SubMenuItemConfig {
                                                    label: "Sub-action 1".to_string(),
                                                    translations: std::collections::HashMap::new(),
                                                    icon: "⚡".to_string(),
                                                    action: Some(ButtonActionConfig::Command { cmd: "firefox".to_string() }),
                                                    items: vec![],
                                                    auto_close: true,
                                                    color: None,
                                                    active_color: None,
                                                });
                                            }
                                            None => {}
                                        }

                                        ui.add_space(12.0);

                                        if is_nested_submenu {
                                            ui.label(egui::RichText::new(tr(&lang, "contains_sub_items").replace("{}", &sub.items.len().to_string())).color(egui::Color32::from_rgb(0, 215, 175)));
                                            ui.add_space(6.0);

                                            let add_sub3_btn = egui::Button::new(
                                                egui::RichText::new(tr(&lang, "add_level_3_btn"))
                                                    .small()
                                                    .strong()
                                                    .color(egui::Color32::from_rgb(0, 215, 175))
                                            )
                                            .fill(egui::Color32::from_rgb(26, 34, 38))
                                            .stroke(egui::Stroke::new(1.0_f32, egui::Color32::from_rgb(0, 215, 175)))
                                            .rounding(egui::Rounding::same(6.0));

                                            if ui.add(add_sub3_btn).clicked() {
                                                let new_num = sub.items.len() + 1;
                                                sub.items.push(SubMenuItemConfig {
                                                    label: format!("Level 3 - {}", new_num),
                                                    translations: std::collections::HashMap::new(),
                                                    icon: "⚡".to_string(),
                                                    action: Some(ButtonActionConfig::Command { cmd: "firefox".to_string() }),
                                                    items: vec![],
                                                    auto_close: true,
                                                    color: None,
                                                    active_color: None,
                                                });
                                            }
                                        } else {
                                            render_node_action_selector(ui, &lang, &mut sub.action, &mut self.custom_cmd_input, is_recording_this_node, || {
                                                self.recording_shortcut_for = if is_recording_this_node { None } else { Some(current_target.clone()) };
                                            });
                                            ui.add_space(8.0);
                                            ui.checkbox(&mut sub.auto_close, tr(&lang, "auto_close_label"));
                                        }

                                        ui.add_space(12.0);
                                        ui.separator();
                                        ui.add_space(8.0);
                                        ui.label(egui::RichText::new(tr(&lang, "unit_colors_header")).small().strong().color(egui::Color32::from_gray(140)));
                                        render_color_editor(ui, &lang, tr(&lang, "idle_color"), &mut sub.color, &def_norm);
                                        render_color_editor(ui, &lang, tr(&lang, "active_hover_color"), &mut sub.active_color, &def_act);
                                    });
                                }
                            }

                        SelectedNodePath::NestedSubItem(s_idx, sub1_idx, sub2_idx) => {
                            let sub2_opt = if active_profile_idx == 0 {
                                self.config.ring_menu.items.get_mut(*s_idx)
                                    .and_then(|s| s.items.get_mut(*sub1_idx))
                                    .and_then(|s1| s1.items.get_mut(*sub2_idx))
                            } else {
                                self.app_profiles.get_mut(active_profile_idx.saturating_sub(1))
                                    .and_then(|p| p.ring_menu.as_mut())
                                    .and_then(|m| m.items.get_mut(*s_idx))
                                    .and_then(|s| s.items.get_mut(*sub1_idx))
                                    .and_then(|s1| s1.items.get_mut(*sub2_idx))
                            };
                            if let Some(sub2) = sub2_opt {
                                ui.group(|ui| {
                                    ui.set_width(ui.available_width());
                                    ui.horizontal(|ui| {
                                        ui.label(egui::RichText::new(tr(&lang, "sub_item_level_2").replace("{}", &(sub2_idx + 1).to_string())).strong().color(egui::Color32::from_rgb(0, 215, 175)));
                                        ui.with_layout(egui::Layout::right_to_left(egui::Align::Center), |ui| {
                                            let del_btn = egui::Button::new(
                                                egui::RichText::new("🗑")
                                                    .small()
                                                    .strong()
                                                    .color(egui::Color32::from_rgb(255, 110, 110))
                                            )
                                            .fill(egui::Color32::from_rgb(42, 22, 26))
                                            .stroke(egui::Stroke::new(1.0_f32, egui::Color32::from_rgb(120, 45, 50)))
                                            .rounding(egui::Rounding::same(6.0));

                                            if ui.add(del_btn).on_hover_text(tr(&lang, "delete_button")).clicked() {
                                                node_to_delete = Some(SelectedNodePath::NestedSubItem(*s_idx, *sub1_idx, *sub2_idx));
                                            }
                                        });
                                    });
                                    ui.add_space(8.0);

                                    ui.horizontal(|ui| {
                                        ui.label(tr(&lang, "label_title"));
                                        ui.add(egui::TextEdit::singleline(&mut sub2.label).desired_width(200.0));
                                    });
                                    ui.add_space(8.0);

                                    render_icon_picker(ui, ctx, &lang, &mut self.texture_cache, &mut sub2.icon, ("sub2_icon", *s_idx, *sub1_idx, *sub2_idx));

                                    ui.add_space(12.0);
                                    render_node_action_selector(ui, &lang, &mut sub2.action, &mut self.custom_cmd_input, is_recording_this_node, || {
                                        self.recording_shortcut_for = if is_recording_this_node { None } else { Some(current_target.clone()) };
                                    });

                                    ui.add_space(8.0);
                                    ui.checkbox(&mut sub2.auto_close, tr(&lang, "auto_close_label"));

                                    ui.add_space(12.0);
                                    ui.separator();
                                    ui.add_space(8.0);
                                    ui.label(egui::RichText::new(tr(&lang, "unit_colors_header")).small().strong().color(egui::Color32::from_gray(140)));
                                    render_color_editor(ui, &lang, tr(&lang, "idle_color"), &mut sub2.color, &def_norm);
                                    render_color_editor(ui, &lang, tr(&lang, "active_hover_color"), &mut sub2.active_color, &def_act);
                                });
                            }
                        }
                    }
                });
            });

        egui::CentralPanel::default().show(ctx, |ui| {
            let rect = ui.max_rect();
            let center = rect.center();
            // Top Right: Per-App Window Profiles Selector (Square Icon Boxes / Logi Options+ Style)
            let total_profiles_count = 1 + self.app_profiles.len() + 1; // Global + App profiles + Add
            let bar_width = (total_profiles_count as f32 * 40.0) + 12.0;
            let profile_bar_rect = egui::Rect::from_min_size(
                egui::pos2(rect.max.x - bar_width - 16.0, rect.min.y + 16.0),
                egui::vec2(bar_width, 38.0),
            );

            let allow_profile_clicks = !self.show_profile_modal && self.prevent_profile_click_frame == 0;
            if self.prevent_profile_click_frame > 0 {
                self.prevent_profile_click_frame -= 1;
            }

            ui.allocate_ui_at_rect(profile_bar_rect, |ui| {
                ui.horizontal(|ui| {
                    ui.spacing_mut().item_spacing = egui::vec2(6.0, 0.0);

                    // 1. Global / Default Profile (🌐 Square Icon)
                    let is_global = self.active_profile_index == 0;
                    let global_btn = egui::Button::new(
                        egui::RichText::new("🌐")
                            .size(15.0)
                            .strong()
                            .color(if is_global { egui::Color32::from_rgb(0, 215, 175) } else { egui::Color32::from_gray(160) })
                    )
                    .fill(if is_global { egui::Color32::from_rgb(32, 42, 52) } else { egui::Color32::from_rgb(22, 24, 30) })
                    .stroke(egui::Stroke::new(if is_global { 1.5_f32 } else { 1.0_f32 }, if is_global { egui::Color32::from_rgb(0, 215, 175) } else { egui::Color32::from_rgb(45, 50, 60) }))
                    .rounding(egui::Rounding::same(8.0))
                    .min_size(egui::vec2(34.0, 34.0));

                    let global_resp = ui.add(global_btn).on_hover_text(tr(&lang, "profile_global_tooltip"));
                    if allow_profile_clicks && global_resp.clicked() {
                        self.active_profile_index = 0;
                        self.status_message = tr(&lang, "profile_global_demo_status").to_string();
                    }

                    // 2. Application Profiles (Square Buttons with pencil icon on hover)
                    let mut open_edit_for: Option<usize> = None;

                    for (i, profile) in self.app_profiles.iter().enumerate() {
                        let profile_idx = i + 1;
                        let is_active = self.active_profile_index == profile_idx;

                        let display_label = profile.label_lang(&lang);
                        let initial = display_label.chars().next().unwrap_or('A').to_uppercase().to_string();

                        let (box_rect, resp) = ui.allocate_exact_size(egui::vec2(34.0, 34.0), egui::Sense::click());

                        let bg_color = if is_active { egui::Color32::from_rgb(32, 42, 52) } else { egui::Color32::from_rgb(22, 24, 30) };
                        let stroke_color = if is_active { egui::Color32::from_rgb(0, 215, 175) } else { egui::Color32::from_rgb(45, 50, 60) };
                        let stroke_width = if is_active { 1.5_f32 } else { 1.0_f32 };
                        let text_color = if is_active { egui::Color32::from_rgb(0, 215, 175) } else { egui::Color32::from_gray(160) };

                        ui.painter().rect_filled(box_rect, 8.0, bg_color);
                        ui.painter().rect_stroke(box_rect, 8.0, egui::Stroke::new(stroke_width, stroke_color));

                        let has_logo = !profile.logo_path.is_empty() && crate::utils::icon_loader::is_image_path(&profile.logo_path);
                        if has_logo {
                            crate::utils::icon_loader::render_icon_or_emoji(
                                ui.painter(),
                                ctx,
                                &mut self.texture_cache,
                                box_rect.center(),
                                12.0,
                                &profile.logo_path,
                                14.0,
                                text_color,
                            );
                        } else {
                            ui.painter().text(
                                box_rect.center(),
                                egui::Align2::CENTER_CENTER,
                                &initial,
                                egui::FontId::proportional(15.0),
                                text_color,
                            );
                        }

                        resp.clone().on_hover_text(format!("{} ({})\n{}", display_label, profile.app_id, tr(&lang, "profile_edit_tooltip")));

                        let is_hovered = resp.hovered();
                        let pencil_rect = egui::Rect::from_min_size(
                            egui::pos2(box_rect.max.x - 14.0, box_rect.min.y),
                            egui::vec2(14.0, 14.0),
                        );

                        // Draw Pencil ✏️ edit icon on hover or active
                        if is_hovered || is_active {
                            ui.painter().circle_filled(pencil_rect.center(), 6.5, egui::Color32::from_rgb(0, 215, 175));
                            ui.painter().text(
                                pencil_rect.center(),
                                egui::Align2::CENTER_CENTER,
                                "✏",
                                egui::FontId::proportional(9.0),
                                egui::Color32::BLACK,
                            );
                        }

                        if allow_profile_clicks && resp.clicked() {
                            let click_pos = ui.input(|i| i.pointer.interact_pos()).unwrap_or(box_rect.center());
                            if pencil_rect.contains(click_pos) || (is_active && is_hovered) {
                                open_edit_for = Some(i);
                            } else {
                                self.active_profile_index = profile_idx;
                                self.status_message = tr(&lang, "status_profile_selected")
                                    .replacen("{}", display_label, 1)
                                    .replacen("{}", &profile.app_id, 1);
                            }
                        }
                    }

                    if allow_profile_clicks {
                        if let Some(edit_idx) = open_edit_for {
                            let p = &self.app_profiles[edit_idx];
                            self.editing_profile_index = Some(edit_idx);
                            self.modal_app_id = p.app_id.clone();
                            self.modal_label = p.label.clone();
                            self.modal_logo_path = p.logo_path.clone();
                            self.confirming_delete = false;
                            self.show_profile_modal = true;
                        }
                    }

                    // 3. '+' Add Profile Square Icon
                    let add_btn = egui::Button::new(
                        egui::RichText::new("➕")
                            .size(13.0)
                            .strong()
                            .color(egui::Color32::from_rgb(0, 215, 175))
                    )
                    .fill(egui::Color32::from_rgb(22, 24, 30))
                    .stroke(egui::Stroke::new(1.0_f32, egui::Color32::from_rgb(0, 215, 175)))
                    .rounding(egui::Rounding::same(8.0))
                    .min_size(egui::vec2(34.0, 34.0));

                    let add_resp = ui.add(add_btn).on_hover_text(tr(&lang, "profile_add_tooltip"));
                    if allow_profile_clicks && add_resp.clicked() {
                        self.editing_profile_index = None;
                        self.modal_app_id = String::new();
                        self.modal_label = String::new();
                        self.modal_logo_path = String::new();
                        self.confirming_delete = false;
                        self.show_profile_modal = true;
                    }
                });
            });

            let painter = ui.painter();
            let pointer_pos = ctx.input(|i| i.pointer.hover_pos()).unwrap_or(center);

            // Central close button ✕
            let is_center_hovered = pointer_pos.distance(center) <= 24.0;
            let mut ring_cfg = self.active_ring_menu().clone();
            ring_cfg.trigger_mode = self.config.ring_menu.trigger_mode.clone();
            ring_cfg.wheel_navigation = self.config.ring_menu.wheel_navigation;
            ring_cfg.glow_effect = self.config.ring_menu.glow_effect;
            ring_cfg.pulse_effect = self.config.ring_menu.pulse_effect;
            ring_cfg.animation = self.config.ring_menu.animation.clone();

            crate::ring_menu::render::render_glowing_circle(
                painter,
                center,
                24.0,
                if is_center_hovered { egui::Color32::from_rgb(220, 50, 60) } else { egui::Color32::from_rgb(180, 180, 190) },
                if is_center_hovered { egui::Color32::from_rgb(255, 70, 80) } else { egui::Color32::WHITE },
                is_center_hovered,
                255,
                ring_cfg.glow_effect,
            );
            painter.text(center, egui::Align2::CENTER_CENTER, "✕", egui::FontId::proportional(15.0), egui::Color32::BLACK);

            let total_slots = ring_cfg.items.len();
            let max_slots = ring_cfg.max_slots.max(1);
            let ring_slot_count = if total_slots < max_slots { total_slots + 1 } else { total_slots }.max(1);
            let main_radius = 130.0 + (ring_slot_count.saturating_sub(6) as f32 * 12.0);
            let raw_active_slot = match sel_node {
                SelectedNodePath::Slot(s) => Some(s),
                SelectedNodePath::SubItem(s, _) => Some(s),
                SelectedNodePath::NestedSubItem(s, _, _) => Some(s),
            };

            let active_slot_idx = if let Some(s) = raw_active_slot {
                if self.closed_category == Some(s) {
                    None
                } else {
                    Some(s)
                }
            } else {
                None
            };

            let default_norm_str = &ring_cfg.default_color;
            let default_act_str = &ring_cfg.default_active_color;

            let is_any_submenu_open = active_slot_idx.is_some() && ring_cfg.items.get(active_slot_idx.unwrap()).map_or(false, |s| !s.items.is_empty());

            // 1. Render main Ring Menu slots
            for (idx, item) in ring_cfg.items.iter().enumerate() {
                let angle = (idx as f32 * (2.0 * PI / ring_slot_count as f32)) - (PI / 2.0);
                let item_center = egui::pos2(
                    center.x + main_radius * angle.cos(),
                    center.y + main_radius * angle.sin(),
                );

                let item_radius = 35.0;
                let is_selected = active_slot_idx == Some(idx);
                let is_hovered = pointer_pos.distance(item_center) <= item_radius;

                let trash_pos = egui::pos2(item_center.x + item_radius * 0.70, item_center.y - item_radius * 0.70);
                let trash_radius = 11.0;
                let is_trash_hovered = (is_selected || is_hovered) && pointer_pos.distance(trash_pos) <= trash_radius;

                let item_norm_hex = item.color.as_deref().unwrap_or(default_norm_str);
                let item_act_hex = item.active_color.as_deref().unwrap_or(default_act_str);

                let norm_color = parse_hex_color(item_norm_hex).unwrap_or(egui::Color32::from_rgb(40, 44, 55));
                let act_color = parse_hex_color(item_act_hex).unwrap_or(egui::Color32::from_rgb(0, 215, 175));

                let fill_color = if is_selected || is_hovered {
                    act_color
                } else {
                    norm_color
                };

                let text_color = contrasting_text_color(fill_color);

                let is_glowing = is_selected || is_hovered;
                let alpha_mult = if is_any_submenu_open && !is_hovered && !is_selected { 110 } else { 255 };

                crate::ring_menu::render::render_glowing_circle(
                    painter,
                    item_center,
                    item_radius,
                    fill_color,
                    act_color,
                    is_glowing,
                    alpha_mult,
                    ring_cfg.glow_effect,
                );

                crate::icon_loader::render_icon_or_emoji(
                    painter,
                    ctx,
                    &mut self.texture_cache,
                    item_center,
                    item_radius,
                    &item.icon,
                    22.0,
                    text_color,
                );

                if !item.items.is_empty() {
                    let badge_pos = egui::pos2(
                        item_center.x + item_radius * 0.75 * angle.cos(),
                        item_center.y + item_radius * 0.75 * angle.sin(),
                    );
                    painter.circle_filled(badge_pos, 5.5, egui::Color32::from_rgb(255, 190, 40));
                }

                // Render Trash icon badge (top-right) and Drag icon badge (top-left) on selected or hovered slots
                if is_selected || is_hovered {
                    let trash_bg = if is_trash_hovered { egui::Color32::from_rgb(240, 60, 60) } else { egui::Color32::from_rgb(180, 40, 50) };
                    painter.circle_filled(trash_pos, trash_radius, trash_bg);
                    painter.text(trash_pos, egui::Align2::CENTER_CENTER, "🗑", egui::FontId::proportional(11.0), egui::Color32::WHITE);

                    let drag_pos = egui::pos2(item_center.x - item_radius * 0.70, item_center.y - item_radius * 0.70);
                    let drag_radius = 11.0;
                    let is_drag_hovered = pointer_pos.distance(drag_pos) <= drag_radius;

                    let drag_bg = if is_drag_hovered || self.dragging_slot == Some(idx) { egui::Color32::from_rgb(0, 215, 175) } else { egui::Color32::from_rgb(45, 55, 70) };
                    painter.circle_filled(drag_pos, drag_radius, drag_bg);
                    painter.circle_stroke(drag_pos, drag_radius, egui::Stroke::new(1.0_f32, egui::Color32::WHITE));
                    painter.text(drag_pos, egui::Align2::CENTER_CENTER, "⠿", egui::FontId::proportional(12.0), egui::Color32::WHITE);

                    if is_drag_hovered {
                        ctx.set_cursor_icon(egui::CursorIcon::Grab);
                        if ctx.input(|i| i.pointer.primary_down()) {
                            self.dragging_slot = Some(idx);
                        }
                    }
                }

                if ctx.input(|i| i.pointer.any_click()) {
                    if is_trash_hovered {
                        node_to_delete = Some(SelectedNodePath::Slot(idx));
                    } else if is_hovered {
                        if !item.items.is_empty() && active_slot_idx == Some(idx) {
                            self.closed_category = Some(idx);
                        } else {
                            self.closed_category = None;
                            self.selected_node = SelectedNodePath::Slot(idx);
                        }
                    }
                }
            }

            // 1.5. Render "➕" Slot on ring if items < max_slots
            if total_slots < max_slots {
                let add_slot_num = total_slots as u8;
                let add_angle = (add_slot_num as f32 * (2.0 * PI / ring_slot_count as f32)) - (PI / 2.0);
                let add_center = egui::pos2(
                    center.x + main_radius * add_angle.cos(),
                    center.y + main_radius * add_angle.sin(),
                );

                let add_radius = 35.0;
                let is_add_hovered = pointer_pos.distance(add_center) <= add_radius;

                let fill_color = if is_add_hovered {
                    egui::Color32::from_rgba_unmultiplied(0, 215, 175, 50)
                } else {
                    egui::Color32::from_rgba_unmultiplied(30, 35, 45, 180)
                };
                let stroke_color = if is_add_hovered {
                    egui::Color32::from_rgb(0, 215, 175)
                } else {
                    egui::Color32::from_rgba_unmultiplied(0, 215, 175, 130)
                };

                painter.circle_filled(add_center, add_radius, fill_color);
                painter.circle_stroke(add_center, add_radius, egui::Stroke::new(2.0f32, stroke_color));

                painter.text(
                    add_center,
                    egui::Align2::CENTER_CENTER,
                    "➕",
                    egui::FontId::proportional(22.0),
                    egui::Color32::from_rgb(0, 215, 175),
                );

                let add_label_pos = egui::pos2(add_center.x, add_center.y + add_radius + 12.0);
                painter.rect_filled(
                    egui::Rect::from_center_size(add_label_pos, egui::vec2(85.0, 18.0)),
                    4.0,
                    egui::Color32::from_black_alpha(180),
                );
                painter.text(
                    add_label_pos,
                    egui::Align2::CENTER_CENTER,
                    tr(&lang, "add_main_slot"),
                    egui::FontId::proportional(11.0),
                    egui::Color32::from_rgb(0, 215, 175),
                );

                if is_add_hovered && ctx.input(|i| i.pointer.any_click()) {
                    let new_slot = self.active_ring_menu().items.len() as u8;
                    let menu = self.active_ring_menu_mut();
                    menu.items.push(RingMenuItemConfig {
                        slot: new_slot,
                        label: format!("Slot {}", new_slot),
                        translations: std::collections::HashMap::new(),
                        icon: "⚡".to_string(),
                        action: Some(ButtonActionConfig::Command { cmd: "firefox".to_string() }),
                        items: vec![],
                        auto_close: true,
                        color: None,
                        active_color: None,
                    });
                    self.selected_node = SelectedNodePath::Slot(menu.items.len() - 1);
                    self.status_message = tr(&lang, "status_slot_added").replace("{}", &new_slot.to_string());
                }
            }

            // 2. Render submenu full circle for active slot
            if let Some(slot_idx) = active_slot_idx {
                if let Some(slot) = ring_cfg.items.get(slot_idx) {
                    if !slot.items.is_empty() {
                        let parent_angle = (slot_idx as f32 * (2.0 * PI / ring_slot_count as f32)) - (PI / 2.0);
                        let parent_center = egui::pos2(
                            center.x + main_radius * parent_angle.cos(),
                            center.y + main_radius * parent_angle.sin(),
                        );

                        let sub_count = slot.items.len();
                        let sub_slot_count = if sub_count < max_slots { sub_count + 1 } else { sub_count }.max(1);
                        let sub_radius = 90.0 + (sub_slot_count.saturating_sub(6) as f32 * 10.0);

                        for (sub_idx, sub_item) in slot.items.iter().enumerate() {
                            let sub_angle = (sub_idx as f32 * (2.0 * PI / sub_slot_count as f32)) - (PI / 2.0);
                            let sub_center = egui::pos2(
                                parent_center.x + sub_radius * sub_angle.cos(),
                                parent_center.y + sub_radius * sub_angle.sin(),
                            );

                            let sub_node_radius = 28.0;

                            let is_sub_selected = self.selected_node == SelectedNodePath::SubItem(slot_idx, sub_idx)
                                || matches!(self.selected_node, SelectedNodePath::NestedSubItem(s, s1, _) if s == slot_idx && s1 == sub_idx);

                            let is_sub_hovered = pointer_pos.distance(sub_center) <= sub_node_radius;

                            let trash_sub_pos = egui::pos2(sub_center.x + sub_node_radius * 0.70, sub_center.y - sub_node_radius * 0.70);
                            let trash_sub_radius = 10.0;
                            let is_sub_trash_hovered = (is_sub_selected || is_sub_hovered) && pointer_pos.distance(trash_sub_pos) <= trash_sub_radius;

                            let sub_norm_hex = sub_item.color.as_deref().unwrap_or(default_norm_str);
                            let sub_act_hex = sub_item.active_color.as_deref().unwrap_or(default_act_str);

                            let sub_norm_color = parse_hex_color(sub_norm_hex).unwrap_or(egui::Color32::from_rgb(50, 55, 70));
                            let sub_act_color = parse_hex_color(sub_act_hex).unwrap_or(egui::Color32::from_rgb(0, 215, 175));

                            let sub_fill = if is_sub_selected || is_sub_hovered {
                                sub_act_color
                            } else {
                                sub_norm_color
                            };

                            let sub_text_color = contrasting_text_color(sub_fill);
                            let is_sub_glowing = is_sub_selected || is_sub_hovered;

                            crate::ring_menu::render::render_glowing_circle(
                                painter,
                                sub_center,
                                sub_node_radius,
                                sub_fill,
                                sub_act_color,
                                is_sub_glowing,
                                255,
                                ring_cfg.glow_effect,
                            );
                            crate::icon_loader::render_icon_or_emoji(
                                painter,
                                ctx,
                                &mut self.texture_cache,
                                sub_center,
                                sub_node_radius,
                                &sub_item.icon,
                                18.0,
                                sub_text_color,
                            );

                            let label_pos = egui::pos2(sub_center.x, sub_center.y + sub_node_radius + 12.0);
                            painter.rect_filled(
                                egui::Rect::from_center_size(label_pos, egui::vec2(85.0, 18.0)),
                                4.0,
                                egui::Color32::from_black_alpha(200),
                            );
                            painter.text(label_pos, egui::Align2::CENTER_CENTER, &sub_item.label, egui::FontId::proportional(11.0), egui::Color32::WHITE);

                            if !sub_item.items.is_empty() {
                                let sub_badge = egui::pos2(sub_center.x + 18.0, sub_center.y - 18.0);
                                painter.circle_filled(sub_badge, 5.0, egui::Color32::from_rgb(255, 190, 40));
                            }

                            // Render Trash badge (top-right) and Drag badge (top-left) for sub-items
                            if is_sub_selected || is_sub_hovered {
                                let trash_bg = if is_sub_trash_hovered { egui::Color32::from_rgb(240, 60, 60) } else { egui::Color32::from_rgb(180, 40, 50) };
                                painter.circle_filled(trash_sub_pos, trash_sub_radius, trash_bg);
                                painter.text(trash_sub_pos, egui::Align2::CENTER_CENTER, "🗑", egui::FontId::proportional(10.0), egui::Color32::WHITE);

                                let drag_sub_pos = egui::pos2(sub_center.x - sub_node_radius * 0.70, sub_center.y - sub_node_radius * 0.70);
                                let drag_sub_radius = 10.0;
                                let is_sub_drag_hovered = pointer_pos.distance(drag_sub_pos) <= drag_sub_radius;

                                let drag_sub_bg = if is_sub_drag_hovered || self.dragging_sub_item == Some((slot_idx, sub_idx)) { egui::Color32::from_rgb(0, 215, 175) } else { egui::Color32::from_rgb(45, 55, 70) };
                                painter.circle_filled(drag_sub_pos, drag_sub_radius, drag_sub_bg);
                                painter.circle_stroke(drag_sub_pos, drag_sub_radius, egui::Stroke::new(1.0_f32, egui::Color32::WHITE));
                                painter.text(drag_sub_pos, egui::Align2::CENTER_CENTER, "⠿", egui::FontId::proportional(11.0), egui::Color32::WHITE);

                                if is_sub_drag_hovered {
                                    ctx.set_cursor_icon(egui::CursorIcon::Grab);
                                    if ctx.input(|i| i.pointer.primary_down()) {
                                        self.dragging_sub_item = Some((slot_idx, sub_idx));
                                    }
                                }
                            }

                            if ctx.input(|i| i.pointer.any_click()) {
                                if is_sub_trash_hovered {
                                    node_to_delete = Some(SelectedNodePath::SubItem(slot_idx, sub_idx));
                                } else if is_sub_hovered {
                                    self.closed_category = None;
                                    self.selected_node = SelectedNodePath::SubItem(slot_idx, sub_idx);
                                }
                            }

                            // 3. Render nested sub-actions (Level 3)
                            if is_sub_selected && !sub_item.items.is_empty() {
                                let nested_radius = 75.0;
                                let nested_count = sub_item.items.len();
                                let nested_step = 0.45;
                                let nested_spread = nested_step * (nested_count.saturating_sub(1) as f32);
                                let nested_start = sub_angle - (nested_spread / 2.0);

                                for (sub2_idx, sub2_item) in sub_item.items.iter().enumerate() {
                                    let n_angle = nested_start + (sub2_idx as f32 * nested_step);
                                    let n_center = egui::pos2(
                                        sub_center.x + nested_radius * n_angle.cos(),
                                        sub_center.y + nested_radius * n_angle.sin(),
                                    );

                                    let n_radius = 22.0;
                                    let is_n_selected = self.selected_node == SelectedNodePath::NestedSubItem(slot_idx, sub_idx, sub2_idx);
                                    let is_n_hovered = pointer_pos.distance(n_center) <= n_radius;

                                    let n_norm_hex = sub2_item.color.as_deref().unwrap_or(default_norm_str);
                                    let n_act_hex = sub2_item.active_color.as_deref().unwrap_or(default_act_str);

                                    let n_norm_color = parse_hex_color(n_norm_hex).unwrap_or(egui::Color32::from_rgb(60, 65, 80));
                                    let n_act_color = parse_hex_color(n_act_hex).unwrap_or(egui::Color32::from_rgb(0, 215, 175));

                                    let n_fill = if is_n_selected || is_n_hovered {
                                        n_act_color
                                    } else {
                                        n_norm_color
                                    };

                                    let n_text_color = contrasting_text_color(n_fill);

                                    painter.circle_filled(n_center, n_radius, n_fill);
                                    painter.circle_stroke(n_center, n_radius, egui::Stroke::new(1.5f32, egui::Color32::WHITE));
                                    crate::icon_loader::render_icon_or_emoji(
                                        painter,
                                        ctx,
                                        &mut self.texture_cache,
                                        n_center,
                                        n_radius,
                                        &sub2_item.icon,
                                        15.0,
                                        n_text_color,
                                    );

                                    let trash_n_pos = egui::pos2(n_center.x + n_radius * 0.70, n_center.y - n_radius * 0.70);
                                    let trash_n_radius = 9.0;
                                    let is_n_trash_hovered = (is_n_selected || is_n_hovered) && pointer_pos.distance(trash_n_pos) <= trash_n_radius;

                                    if is_n_selected || is_n_hovered {
                                        let trash_bg = if is_n_trash_hovered { egui::Color32::from_rgb(240, 60, 60) } else { egui::Color32::from_rgb(180, 40, 50) };
                                        painter.circle_filled(trash_n_pos, trash_n_radius, trash_bg);
                                        painter.text(trash_n_pos, egui::Align2::CENTER_CENTER, "🗑", egui::FontId::proportional(9.0), egui::Color32::WHITE);
                                    }

                                    if ctx.input(|i| i.pointer.any_click()) {
                                        if is_n_trash_hovered {
                                            node_to_delete = Some(SelectedNodePath::NestedSubItem(slot_idx, sub_idx, sub2_idx));
                                        } else if is_n_hovered {
                                            self.closed_category = None;
                                            self.selected_node = SelectedNodePath::NestedSubItem(slot_idx, sub_idx, sub2_idx);
                                        }
                                    }
                                }
                            }
                        }

                        // Render ➕ sub-slot if sub_count < max_slots
                        if sub_count < max_slots {
                            let add_sub_angle = (sub_count as f32 * (2.0 * PI / sub_slot_count as f32)) - (PI / 2.0);
                            let add_sub_center = egui::pos2(
                                parent_center.x + sub_radius * add_sub_angle.cos(),
                                parent_center.y + sub_radius * add_sub_angle.sin(),
                            );
                            let add_sub_node_radius = 28.0;
                            let is_add_sub_hovered = pointer_pos.distance(add_sub_center) <= add_sub_node_radius;

                            let sub_fill_color = if is_add_sub_hovered {
                                egui::Color32::from_rgba_unmultiplied(0, 215, 175, 50)
                            } else {
                                egui::Color32::from_rgba_unmultiplied(30, 35, 45, 180)
                            };
                            let sub_stroke_color = if is_add_sub_hovered {
                                egui::Color32::from_rgb(0, 215, 175)
                            } else {
                                egui::Color32::from_rgba_unmultiplied(0, 215, 175, 130)
                            };

                            painter.circle_filled(add_sub_center, add_sub_node_radius, sub_fill_color);
                            painter.circle_stroke(add_sub_center, add_sub_node_radius, egui::Stroke::new(1.8f32, sub_stroke_color));

                            painter.text(
                                add_sub_center,
                                egui::Align2::CENTER_CENTER,
                                "➕",
                                egui::FontId::proportional(18.0),
                                egui::Color32::from_rgb(0, 215, 175),
                            );

                            if is_add_sub_hovered && ctx.input(|i| i.pointer.any_click()) {
                                let new_num = slot.items.len() + 1;
                                let menu = self.active_ring_menu_mut();
                                if let Some(mut_slot) = menu.items.get_mut(slot_idx) {
                                    mut_slot.items.push(SubMenuItemConfig {
                                        label: format!("Action {}", new_num),
                                        translations: std::collections::HashMap::new(),
                                        icon: "⚡".to_string(),
                                        action: Some(ButtonActionConfig::Command { cmd: "firefox".to_string() }),
                                        items: vec![],
                                        auto_close: true,
                                        color: None,
                                        active_color: None,
                                    });
                                    self.selected_node = SelectedNodePath::SubItem(slot_idx, mut_slot.items.len() - 1);
                                }
                            }
                        }
                    }
                }

                // Handle active Drag & Drop reordering
                if !ctx.input(|i| i.pointer.primary_down()) {
                    self.dragging_slot = None;
                    self.dragging_sub_item = None;
                } else {
                    ctx.set_cursor_icon(egui::CursorIcon::Grabbing);
                    if let Some(dragged_idx) = self.dragging_slot {
                        if dragged_idx < total_slots {
                            let mouse_angle = (pointer_pos.y - center.y).atan2(pointer_pos.x - center.x);
                            let norm_angle = (mouse_angle + (PI / 2.0)).rem_euclid(2.0 * PI);
                            let step = 2.0 * PI / ring_slot_count as f32;
                            let target_idx = ((norm_angle + (step / 2.0)) / step).floor() as usize;

                            if target_idx < total_slots && target_idx != dragged_idx {
                                self.active_ring_menu_mut().items.swap(dragged_idx, target_idx);
                                self.dragging_slot = Some(target_idx);
                                if let SelectedNodePath::Slot(s) = self.selected_node {
                                    if s == dragged_idx {
                                        self.selected_node = SelectedNodePath::Slot(target_idx);
                                    } else if s == target_idx {
                                        self.selected_node = SelectedNodePath::Slot(dragged_idx);
                                    }
                                }
                            }
                        }
                    } else if let Some((s_idx, dragged_sub_idx)) = self.dragging_sub_item {
                        if let Some(slot) = self.active_ring_menu_mut().items.get_mut(s_idx) {
                            let sub_count = slot.items.len();
                            if sub_count > 1 && dragged_sub_idx < sub_count {
                                let parent_angle = (s_idx as f32 * (2.0 * PI / ring_slot_count as f32)) - (PI / 2.0);
                                let parent_center = egui::pos2(
                                    center.x + main_radius * parent_angle.cos(),
                                    center.y + main_radius * parent_angle.sin(),
                                );
                                let mouse_angle = (pointer_pos.y - parent_center.y).atan2(pointer_pos.x - parent_center.x);
                                let norm_angle = (mouse_angle + (PI / 2.0)).rem_euclid(2.0 * PI);
                                let sub_step = 2.0 * PI / sub_count as f32;
                                let target_sub_idx = ((norm_angle + (sub_step / 2.0)) / sub_step).floor() as usize % sub_count;

                                if target_sub_idx < sub_count && target_sub_idx != dragged_sub_idx {
                                    slot.items.swap(dragged_sub_idx, target_sub_idx);
                                    self.dragging_sub_item = Some((s_idx, target_sub_idx));
                                    if let SelectedNodePath::SubItem(s, sub) = self.selected_node {
                                        if s == s_idx {
                                            if sub == dragged_sub_idx {
                                                self.selected_node = SelectedNodePath::SubItem(s_idx, target_sub_idx);
                                            } else if sub == target_sub_idx {
                                                self.selected_node = SelectedNodePath::SubItem(s_idx, dragged_sub_idx);
                                            }
                                        }
                                    }
                                }
                            }
                        }
                    }
                }
            }
        });

        // Handle item deletion (after panel interaction)
        if let Some(to_del) = node_to_delete {
            self.closed_category = None;
            match to_del {
                SelectedNodePath::Slot(s) => {
                    let menu = self.active_ring_menu_mut();
                    if s < menu.items.len() {
                        menu.items.remove(s);
                        for (idx, slot_item) in menu.items.iter_mut().enumerate() {
                            slot_item.slot = idx as u8;
                        }
                        self.selected_node = SelectedNodePath::Slot(0);
                    }
                }
                SelectedNodePath::SubItem(s, sub1) => {
                    if let Some(slot) = self.active_ring_menu_mut().items.get_mut(s) {
                        if sub1 < slot.items.len() {
                            slot.items.remove(sub1);
                            self.selected_node = SelectedNodePath::Slot(s);
                        }
                    }
                }
                SelectedNodePath::NestedSubItem(s, sub1, sub2) => {
                    if let Some(slot) = self.active_ring_menu_mut().items.get_mut(s) {
                        if let Some(sub) = slot.items.get_mut(sub1) {
                            if sub2 < sub.items.len() {
                                sub.items.remove(sub2);
                                self.selected_node = SelectedNodePath::SubItem(s, sub1);
                            }
                        }
                    }
                }
            }
        }
    }
}
