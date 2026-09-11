use crate::config::{color_to_hex, parse_hex_color, ButtonActionConfig};
use crate::i18n::tr;
use eframe::egui;

pub fn card_frame() -> egui::Frame {
    egui::Frame::none()
        .fill(egui::Color32::from_rgb(22, 24, 30))
        .stroke(egui::Stroke::new(1.0_f32, egui::Color32::from_rgb(38, 42, 54)))
        .rounding(egui::Rounding::same(10.0))
        .inner_margin(egui::Margin::symmetric(14.0, 10.0))
}

pub fn card_frame_active() -> egui::Frame {
    egui::Frame::none()
        .fill(egui::Color32::from_rgb(20, 32, 38))
        .stroke(egui::Stroke::new(1.2_f32, egui::Color32::from_rgb(0, 215, 175)))
        .rounding(egui::Rounding::same(10.0))
        .inner_margin(egui::Margin::symmetric(14.0, 10.0))
}


pub fn render_search_bar(ui: &mut egui::Ui, query: &mut String, hint: &str) -> bool {
    let mut changed = false;
    let frame = egui::Frame::none()
        .fill(egui::Color32::from_rgb(28, 30, 38))
        .stroke(egui::Stroke::new(1.0_f32, egui::Color32::from_rgb(45, 50, 65)))
        .rounding(egui::Rounding::same(8.0))
        .inner_margin(egui::Margin::symmetric(10.0, 6.0));

    frame.show(ui, |ui| {
        ui.horizontal(|ui| {
            ui.label(egui::RichText::new("🔍").size(13.0));
            let response = ui.add(
                egui::TextEdit::singleline(query)
                    .hint_text(hint)
                    .frame(false)
                    .desired_width(ui.available_width() - 25.0),
            );
            if response.changed() {
                changed = true;
            }
            if !query.is_empty() {
                if ui.add(egui::Button::new("🗙").frame(false)).clicked() {
                    query.clear();
                    changed = true;
                }
            }
        });
    });
    changed
}

pub fn render_category_chips(
    ui: &mut egui::Ui,
    current: &mut super::types::ActionCategory,
    lang: &str,
) {
    let categories = [
        super::types::ActionCategory::All,
        super::types::ActionCategory::Navigation,
        super::types::ActionCategory::Media,
        super::types::ActionCategory::Windows,
        super::types::ActionCategory::Shortcuts,
        super::types::ActionCategory::Apps,
        super::types::ActionCategory::System,
    ];

    ui.horizontal_wrapped(|ui| {
        for cat in categories {
            let is_sel = *current == cat;
            let (bg, stroke, text_color) = if is_sel {
                (
                    egui::Color32::from_rgb(0, 170, 140),
                    egui::Stroke::new(1.0_f32, egui::Color32::from_rgb(0, 215, 175)),
                    egui::Color32::WHITE,
                )
            } else {
                (
                    egui::Color32::from_rgb(28, 30, 38),
                    egui::Stroke::new(1.0_f32, egui::Color32::from_rgb(45, 50, 65)),
                    egui::Color32::from_gray(180),
                )
            };

            let btn = egui::Button::new(
                egui::RichText::new(cat.label(lang))
                    .small()
                    .strong()
                    .color(text_color),
            )
            .fill(bg)
            .stroke(stroke)
            .rounding(egui::Rounding::same(12.0))
            .min_size(egui::vec2(0.0, 26.0));

            if ui.add(btn).clicked() {
                *current = cat;
            }
        }
    });
}

pub fn render_action_card(
    ui: &mut egui::Ui,
    icon: &str,
    title: &str,
    desc: &str,
    is_active: bool,
) -> bool {
    let frame = if is_active {
        card_frame_active()
    } else {
        card_frame()
    };

    let mut clicked = false;
    frame.show(ui, |ui| {
        let (rect, response) = ui.allocate_exact_size(
            egui::vec2(ui.available_width(), 36.0),
            egui::Sense::click(),
        );

        if response.clicked() {
            clicked = true;
        }

        let painter = ui.painter();
        let icon_pos = egui::pos2(rect.min.x + 8.0, rect.center().y);
        painter.text(
            icon_pos,
            egui::Align2::CENTER_CENTER,
            icon,
            egui::FontId::proportional(17.0),
            egui::Color32::WHITE,
        );

        let title_pos = egui::pos2(rect.min.x + 28.0, rect.min.y + 8.0);
        painter.text(
            title_pos,
            egui::Align2::LEFT_CENTER,
            title,
            egui::FontId::proportional(12.5),
            if is_active {
                egui::Color32::from_rgb(0, 215, 175)
            } else {
                egui::Color32::WHITE
            },
        );

        if !desc.is_empty() {
            let desc_pos = egui::pos2(rect.min.x + 28.0, rect.min.y + 24.0);
            painter.text(
                desc_pos,
                egui::Align2::LEFT_CENTER,
                desc,
                egui::FontId::proportional(10.5),
                egui::Color32::from_gray(140),
            );
        }

        if is_active {
            let check_pos = egui::pos2(rect.max.x - 10.0, rect.center().y);
            painter.text(
                check_pos,
                egui::Align2::CENTER_CENTER,
                "✓",
                egui::FontId::proportional(15.0),
                egui::Color32::from_rgb(0, 215, 175),
            );
        }
    });

    clicked
}

pub fn render_key_badges(ui: &mut egui::Ui, keys: &[String]) {
    ui.horizontal(|ui| {
        for (i, key) in keys.iter().enumerate() {
            if i > 0 {
                ui.label(egui::RichText::new("+").size(12.0).color(egui::Color32::from_gray(140)));
            }
            let key_frame = egui::Frame::none()
                .fill(egui::Color32::from_rgb(38, 44, 58))
                .stroke(egui::Stroke::new(1.0_f32, egui::Color32::from_rgb(65, 75, 95)))
                .rounding(egui::Rounding::same(5.0))
                .inner_margin(egui::Margin::symmetric(8.0, 4.0));

            key_frame.show(ui, |ui| {
                ui.label(
                    egui::RichText::new(key)
                        .strong()
                        .size(12.0)
                        .color(egui::Color32::from_rgb(0, 215, 175)),
                );
            });
        }
    });
}

pub fn render_color_editor(
    ui: &mut egui::Ui,
    lang: &str,
    label: &str,
    color_opt: &mut Option<String>,
    default_hex: &str,
) {
    render_color_editor_with_action(ui, lang, label, color_opt, default_hex, |_| {});
}

pub fn render_color_editor_with_action(
    ui: &mut egui::Ui,
    lang: &str,
    label: &str,
    color_opt: &mut Option<String>,
    default_hex: &str,
    extra_action: impl FnOnce(&mut egui::Ui),
) {
    let cur_hex = color_opt.as_deref().unwrap_or(default_hex).to_string();
    let color = parse_hex_color(&cur_hex).unwrap_or(egui::Color32::from_rgb(230, 230, 235));
    let mut color_arr = [color.r(), color.g(), color.b(), color.a()];

    ui.horizontal(|ui| {
        ui.label(egui::RichText::new(label).small().strong());
        if ui.color_edit_button_srgba_unmultiplied(&mut color_arr).changed() {
            let new_c = egui::Color32::from_rgba_unmultiplied(
                color_arr[0],
                color_arr[1],
                color_arr[2],
                color_arr[3],
            );
            *color_opt = Some(color_to_hex(new_c));
        }

        if color_opt.is_some() {
            if ui.button(egui::RichText::new("↺").small()).on_hover_text(tr(lang, "reset_default_color_tooltip")).clicked() {
                *color_opt = None;
            }
        }

        extra_action(ui);
    });

    // Swatches Palette
    ui.horizontal(|ui| {
        ui.label(egui::RichText::new(tr(lang, "color_swatches_label")).small().color(egui::Color32::from_gray(120)));
        let swatches = [
            "#00D7AF", "#3B82F6", "#8B5CF6", "#EC4899", "#EF4444",
            "#F59E0B", "#10B981", "#6366F1", "#1E293B", "#334155",
        ];
        for hex in swatches {
            if let Some(c) = parse_hex_color(hex) {
                let btn = egui::Button::new("")
                    .fill(c)
                    .stroke(egui::Stroke::new(1.0_f32, egui::Color32::from_black_alpha(100)))
                    .rounding(egui::Rounding::same(4.0))
                    .min_size(egui::vec2(16.0, 16.0));
                if ui.add(btn).on_hover_text(hex).clicked() {
                    *color_opt = Some(hex.to_string());
                }
            }
        }
    });
}

pub fn key_combos_equal(a: &[String], b: &[String]) -> bool {
    if a.len() != b.len() {
        return false;
    }
    let mut norm_a: Vec<String> = a.iter().map(|s| s.to_uppercase()).collect();
    let mut norm_b: Vec<String> = b.iter().map(|s| s.to_uppercase()).collect();
    norm_a.sort();
    norm_b.sort();
    norm_a == norm_b
}

pub fn key_to_string(key: egui::Key) -> String {
    match key {
        egui::Key::A => "a".to_string(),
        egui::Key::B => "b".to_string(),
        egui::Key::C => "c".to_string(),
        egui::Key::D => "d".to_string(),
        egui::Key::E => "e".to_string(),
        egui::Key::F => "f".to_string(),
        egui::Key::G => "g".to_string(),
        egui::Key::H => "h".to_string(),
        egui::Key::I => "i".to_string(),
        egui::Key::J => "j".to_string(),
        egui::Key::K => "k".to_string(),
        egui::Key::L => "l".to_string(),
        egui::Key::M => "m".to_string(),
        egui::Key::N => "n".to_string(),
        egui::Key::O => "o".to_string(),
        egui::Key::P => "p".to_string(),
        egui::Key::Q => "q".to_string(),
        egui::Key::R => "r".to_string(),
        egui::Key::S => "s".to_string(),
        egui::Key::T => "t".to_string(),
        egui::Key::U => "u".to_string(),
        egui::Key::V => "v".to_string(),
        egui::Key::W => "w".to_string(),
        egui::Key::X => "x".to_string(),
        egui::Key::Y => "y".to_string(),
        egui::Key::Z => "z".to_string(),
        egui::Key::Num0 => "0".to_string(),
        egui::Key::Num1 => "1".to_string(),
        egui::Key::Num2 => "2".to_string(),
        egui::Key::Num3 => "3".to_string(),
        egui::Key::Num4 => "4".to_string(),
        egui::Key::Num5 => "5".to_string(),
        egui::Key::Num6 => "6".to_string(),
        egui::Key::Num7 => "7".to_string(),
        egui::Key::Num8 => "8".to_string(),
        egui::Key::Num9 => "9".to_string(),
        egui::Key::F1 => "F1".to_string(),
        egui::Key::F2 => "F2".to_string(),
        egui::Key::F3 => "F3".to_string(),
        egui::Key::F4 => "F4".to_string(),
        egui::Key::F5 => "F5".to_string(),
        egui::Key::F6 => "F6".to_string(),
        egui::Key::F7 => "F7".to_string(),
        egui::Key::F8 => "F8".to_string(),
        egui::Key::F9 => "F9".to_string(),
        egui::Key::F10 => "F10".to_string(),
        egui::Key::F11 => "F11".to_string(),
        egui::Key::F12 => "F12".to_string(),
        egui::Key::Space => "space".to_string(),
        egui::Key::Tab => "Tab".to_string(),
        egui::Key::Enter => "Return".to_string(),
        egui::Key::Escape => "Escape".to_string(),
        egui::Key::Backspace => "BackSpace".to_string(),
        egui::Key::Delete => "Delete".to_string(),
        egui::Key::Insert => "Insert".to_string(),
        egui::Key::Home => "Home".to_string(),
        egui::Key::End => "End".to_string(),
        egui::Key::PageUp => "Page_Up".to_string(),
        egui::Key::PageDown => "Page_Down".to_string(),
        egui::Key::ArrowLeft => "Left".to_string(),
        egui::Key::ArrowRight => "Right".to_string(),
        egui::Key::ArrowUp => "Up".to_string(),
        egui::Key::ArrowDown => "Down".to_string(),
        _ => format!("{:?}", key),
    }
}

pub fn render_icon_picker(
    ui: &mut egui::Ui,
    ctx: &egui::Context,
    lang: &str,
    texture_cache: &mut crate::utils::icon_loader::TextureCache,
    icon: &mut String,
    id_salt: impl std::hash::Hash,
) {
    ui.horizontal(|ui| {
        ui.label(tr(lang, "icon_title"));

        // Current icon preview
        let (rect, preview_resp) = ui.allocate_exact_size(egui::vec2(28.0, 28.0), egui::Sense::hover());
        ui.painter().rect_filled(rect, 6.0, egui::Color32::from_rgb(32, 35, 45));
        ui.painter().rect_stroke(rect, 6.0, egui::Stroke::new(1.0_f32, egui::Color32::from_rgb(50, 55, 70)));
        crate::utils::icon_loader::render_icon_or_emoji(
            ui.painter(),
            ctx,
            texture_cache,
            rect.center(),
            10.0,
            icon,
            14.0,
            egui::Color32::WHITE,
        );

        ui.add_space(4.0);

        // Icon input box
        ui.add(
            egui::TextEdit::singleline(icon)
                .desired_width(120.0)
                .hint_text(tr(lang, "icon_input_hint")),
        );

        // Emoji quick picker popup
        let picker_id = ui.make_persistent_id(id_salt);
        let btn = egui::Button::new("😀").min_size(egui::vec2(28.0, 24.0));
        if ui.add(btn).on_hover_text(tr(lang, "choose_emoji_tooltip")).clicked() {
            ui.memory_mut(|m| m.toggle_popup(picker_id));
        }

        let browse_btn = egui::Button::new(
            egui::RichText::new("📁").small().color(egui::Color32::from_gray(170)),
        )
        .min_size(egui::vec2(28.0, 24.0));
        if ui.add(browse_btn).on_hover_text(tr(lang, "browse_image_tooltip")).clicked() {
            if let Some(path) = rfd::FileDialog::new()
                .add_filter("Images", &["png", "svg", "jpg", "jpeg", "webp"])
                .pick_file()
            {
                *icon = path.to_string_lossy().to_string();
            }
        }

        egui::popup_below_widget(ui, picker_id, &preview_resp, egui::PopupCloseBehavior::CloseOnClick, |ui| {
            ui.set_max_width(280.0);
            let emojis = [
                "💻", "🌐", "⚡", "📁", "🔀", "✂️", "📋", "💾",
                "🔍", "📸", "🔊", "🔉", "🔇", "⏯️", "⏭️", "⏮️",
                "🔒", "🚪", "⏻", "⚙️", "🎯", "🚀", "💡", "🛠️",
                "🤖", "🎨", "🎮", "📝", "📊", "🔗", "⭐", "🔥",
            ];
            egui::Grid::new("emoji_grid").spacing(egui::vec2(4.0, 4.0)).show(ui, |ui| {
                for (i, &e) in emojis.iter().enumerate() {
                    if ui.button(egui::RichText::new(e).size(16.0)).clicked() {
                        *icon = e.to_string();
                    }
                    if (i + 1) % 8 == 0 {
                        ui.end_row();
                    }
                }
            });
        });
    });
}

#[derive(PartialEq)]
pub enum ElementTypeChoice {
    DirectAction,
    SubMenu,
}

pub fn render_type_selector(ui: &mut egui::Ui, lang: &str, is_submenu: bool) -> Option<ElementTypeChoice> {
    let mut choice = None;
    ui.horizontal(|ui| {
        ui.label(tr(lang, "type_label"));
        if ui.radio(!is_submenu, tr(lang, "type_direct_action")).clicked() && is_submenu {
            choice = Some(ElementTypeChoice::DirectAction);
        }
        if ui.radio(is_submenu, tr(lang, "type_submenu")).clicked() && !is_submenu {
            choice = Some(ElementTypeChoice::SubMenu);
        }
    });
    choice
}

pub fn render_node_action_selector(
    ui: &mut egui::Ui,
    lang: &str,
    action: &mut Option<ButtonActionConfig>,
    _custom_cmd: &mut String,
    is_recording: bool,
    on_toggle_recording: impl FnOnce(),
) {
    let current_action_clone = action.clone();
    let is_keycombo = matches!(current_action_clone, Some(ButtonActionConfig::KeyCombo { .. }));
    let is_folder = matches!(&current_action_clone, Some(ButtonActionConfig::Command { cmd }) if cmd.starts_with("xdg-open ") && !cmd.starts_with("xdg-open http"));
    let is_preset = !is_keycombo && !is_folder;

    // Option A: Catalog Preset
    let lock_label = tr(lang, "quick_lock");
    let term_label = tr(lang, "quick_terminal");
    let brow_label = tr(lang, "quick_browser");
    let presets: [(&str, &str); 3] = [
        ("loginctl lock-session", lock_label),
        ("gnome-terminal", term_label),
        ("xdg-open http://google.com", brow_label),
    ];

    let mut current_preset_idx = presets
        .iter()
        .position(|(cmd, _)| matches!(&current_action_clone, Some(ButtonActionConfig::Command { cmd: c }) if c == cmd))
        .unwrap_or(0);

    if ui.radio(is_preset, tr(lang, "quick_action_option")).clicked() {
        *action = Some(ButtonActionConfig::Command { cmd: presets[0].0.to_string() });
    }

    if is_preset {
        ui.indent("preset_node_indent", |ui| {
            egui::ComboBox::from_id_source("preset_action_node_combo")
                .selected_text(presets[current_preset_idx].1)
                .show_ui(ui, |ui| {
                    for (idx, (cmd, label)) in presets.iter().enumerate() {
                        let target_action = ButtonActionConfig::Command { cmd: cmd.to_string() };
                        if ui.selectable_value(&mut current_preset_idx, idx, *label).clicked() {
                            *action = Some(target_action.clone());
                            current_preset_idx = idx;
                        }
                    }
                });
        });
    }

    ui.add_space(6.0);

    // Option B: Key Combo recording
    let key_text = if is_recording {
        tr(lang, "press_keys_prompt").to_string()
    } else if let Some(ButtonActionConfig::KeyCombo { keys }) = action.as_ref() {
        if is_keycombo {
            format!("{}{}", tr(lang, "recorded_shortcut_prefix"), keys.join(" + "))
        } else {
            tr(lang, "custom_shortcut_option").to_string()
        }
    } else {
        tr(lang, "custom_shortcut_option").to_string()
    };

    let combo_btn_color = if is_recording {
        egui::Color32::from_rgb(255, 70, 70)
    } else if is_keycombo {
        egui::Color32::from_rgb(0, 170, 140)
    } else {
        egui::Color32::from_rgb(45, 45, 55)
    };

    let combo_btn = egui::Button::new(
        egui::RichText::new(key_text).strong().color(egui::Color32::WHITE)
    )
    .fill(combo_btn_color)
    .rounding(egui::Rounding::same(8.0))
    .min_size(egui::vec2(ui.available_width(), 32.0));

    if ui.add(combo_btn).clicked() {
        on_toggle_recording();
    }

    ui.add_space(6.0);

    // Option C: Open Folder
    if ui.radio(is_folder, tr(lang, "folder_path_option")).clicked() {
        *action = Some(ButtonActionConfig::Command { cmd: "xdg-open ~".to_string() });
    }

    if is_folder {
        ui.indent("folder_path_node_indent", |ui| {
            let mut current_path = if let Some(ButtonActionConfig::Command { cmd }) = action.as_ref() {
                if let Some(folder) = cmd.strip_prefix("xdg-open ") {
                    folder.to_string()
                } else {
                    cmd.clone()
                }
            } else {
                "~".to_string()
            };

            ui.horizontal(|ui| {
                ui.label(tr(lang, "folder_path_label"));
                if ui.add(egui::TextEdit::singleline(&mut current_path).desired_width(150.0)).changed() {
                    *action = Some(ButtonActionConfig::Command { cmd: format!("xdg-open {}", current_path) });
                }

                let browse_btn = egui::Button::new(
                    egui::RichText::new(tr(lang, "browse_folder_btn")).small().strong().color(egui::Color32::WHITE)
                )
                .fill(egui::Color32::from_rgb(45, 55, 70))
                .rounding(egui::Rounding::same(6.0));

                if ui.add(browse_btn).clicked() {
                    if let Some(path) = rfd::FileDialog::new().pick_folder() {
                        let path_str = path.to_string_lossy().to_string();
                        *action = Some(ButtonActionConfig::Command { cmd: format!("xdg-open {}", path_str) });
                    }
                }
            });
        });
    }
}
