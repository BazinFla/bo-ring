use crate::config::{contrasting_text_color, parse_hex_color, RingAnimation};
use crate::utils::color::with_alpha;
use eframe::egui;
use std::f32::consts::PI;
use std::sync::OnceLock;
use std::time::Instant;

use super::geometry::get_sector_slot_index;
use super::state::{execute_action, next_slot_index, RingMenuApp, ANIM_DURATION_SECS};

/// Render a slot or badge with a multi-layered glowing aura when active/hovered,
/// or a glassmorphism subtle border when idle.
pub fn render_glowing_circle(
    painter: &egui::Painter,
    center: egui::Pos2,
    radius: f32,
    fill_color: egui::Color32,
    glow_color: egui::Color32,
    is_active_or_hovered: bool,
    alpha_mult: u8,
    glow_enabled: bool,
) {
    let fill = with_alpha(fill_color, alpha_mult);
    painter.circle_filled(center, radius, fill);

    if is_active_or_hovered {
        if glow_enabled {
            // Monotonic breathing pulse oscillation for active slot
            static ANIM_BASE_TIME: OnceLock<Instant> = OnceLock::new();
            let time = ANIM_BASE_TIME.get_or_init(Instant::now).elapsed().as_secs_f32();
            let pulse_offset = (time * 5.0).sin() * 1.5;

            // Outer aura 1 (wide, soft glow)
            let aura1_color = with_alpha(glow_color, (alpha_mult as u16 * 65 / 255) as u8);
            painter.circle_stroke(center, radius + 7.0 + pulse_offset, egui::Stroke::new(7.0_f32, aura1_color));

            // Outer aura 2 (mid halo)
            let aura2_color = with_alpha(glow_color, (alpha_mult as u16 * 130 / 255) as u8);
            painter.circle_stroke(center, radius + 3.5 + pulse_offset * 0.5, egui::Stroke::new(3.5_f32, aura2_color));

            // Core stroke (bright crisp border)
            let stroke_color = with_alpha(egui::Color32::WHITE, alpha_mult);
            painter.circle_stroke(center, radius, egui::Stroke::new(2.2_f32, stroke_color));
        } else {
            // Crisp border without glowing aura
            let stroke_color = with_alpha(egui::Color32::WHITE, alpha_mult);
            painter.circle_stroke(center, radius, egui::Stroke::new(2.2_f32, stroke_color));
        }
    } else {
        // Glassmorphism soft border
        let glass_stroke = with_alpha(egui::Color32::from_rgba_unmultiplied(255, 255, 255, 65), alpha_mult);
        painter.circle_stroke(center, radius, egui::Stroke::new(1.8_f32, glass_stroke));
    }
}

impl eframe::App for RingMenuApp {
    fn clear_color(&self, _visuals: &egui::Visuals) -> [f32; 4] {
        [0.0, 0.0, 0.0, 0.0] // Transparent
    }

    fn update(&mut self, ctx: &egui::Context, _frame: &mut eframe::Frame) {
        if self.close_requested {
            ctx.send_viewport_cmd(egui::ViewportCommand::Close);
            return;
        }

        // Non-blocking poll for background active window detection result
        if let Some(rx) = &self.active_app_rx {
            if let Ok(name) = rx.try_recv() {
                let raw = name.trim();
                let custom_ring = self.app_profiles.iter()
                    .find(|p| crate::config::matches_app_id(&p.app_id, raw))
                    .and_then(|p| p.ring_menu.clone());

                if let Some(mut ring) = custom_ring {
                    // Inherit global behavior & visual options from default_ring_menu
                    ring.trigger_mode = self.default_ring_menu.trigger_mode.clone();
                    ring.wheel_navigation = self.default_ring_menu.wheel_navigation;
                    ring.glow_effect = self.default_ring_menu.glow_effect;
                    ring.pulse_effect = self.default_ring_menu.pulse_effect;
                    ring.animation = self.default_ring_menu.animation.clone();

                    self.config = ring;
                    self.is_app_specific = true;
                } else {
                    self.config = self.default_ring_menu.clone();
                    self.is_app_specific = false;
                }

                self.active_app_name = name;
                self.active_app_rx = None;
            }
        }

        // IPC Non-blocking event check (Must run at top of update before center check)
        if let Some(ref rx) = self.ipc_rx {
            while let Ok(line) = rx.try_recv() {
                if line.starts_with("STATE:HELD") || line.starts_with("P:") {
                    self.button_held = true;
                } else if line.starts_with("STATE:RELEASED") || line.starts_with("R:") {
                    self.button_released = true;
                    self.button_held = false;
                    crate::keyboard::nudge_mouse();
                    ctx.request_repaint();
                }
            }
        }

        let mut visuals = egui::Visuals::dark();
        visuals.window_fill = egui::Color32::TRANSPARENT;
        visuals.panel_fill = egui::Color32::TRANSPARENT;
        ctx.set_visuals(visuals);

        egui::CentralPanel::default()
            .frame(egui::Frame::none().fill(egui::Color32::TRANSPARENT))
            .show(ctx, |ui| {
                let rect = ui.max_rect();
                let painter = ui.painter();
                let raw_pointer = ctx.input(|i| i.pointer.hover_pos());

                // === WAYLAND CURSOR POSITIONING WORKAROUND ===
                //
                // Problem: On Wayland (GNOME/Mutter, KDE/KWin, wlroots/Sway, Hyprland),
                // when a new window appears, the compositor sends a `wl_pointer.enter`
                // event with the cursor coordinates. However, these initial coordinates
                // are often stale or incorrect — they don't reflect the true cursor
                // position until a real `wl_pointer.motion` event is received.
                //
                // Solution: We detect actual mouse movement by comparing pointer positions
                // across consecutive frames. The daemon injects a net-zero mouse nudge via
                // uinput after spawning the ring window (see keyboard::nudge_mouse()),
                // which triggers a real `wl_pointer.motion` event from the compositor.
                //
                // Until the center is locked, the window stays fully transparent (nothing
                // is rendered), so the user never sees the ring at a wrong position.
                if self.center.is_none() {
                    let mut lock_pos = None;
                    let elapsed = self.opened_at.elapsed().as_millis();

                    if let Some(pos) = raw_pointer {
                        if !self.has_nudged {
                            // Wait for the compositor to fully map, place, and maximize the window before nudging
                            if elapsed < 200 {
                                ctx.request_repaint();
                                return;
                            }
                            self.has_nudged = true;
                            self.last_pointer = Some(pos);
                            crate::keyboard::nudge_mouse();
                            ctx.request_repaint();
                            return; // Wait for true motion event
                        }

                        // On the frame immediately following the nudge, the pointer motion event
                        // has been processed by the compositor, so `pos` is accurate and up-to-date.
                        lock_pos = Some(pos);
                        self.last_pointer = Some(pos);
                    } else if elapsed > 800 {
                        // Emergency fallback if pointer event withheld after 800ms
                        lock_pos = Some(egui::pos2(rect.width() / 2.0, rect.height() / 2.0));
                    }


                    if let Some(pos) = lock_pos {
                        println!("🎯 Center locked: ({}, {})", pos.x, pos.y);
                        self.center = Some(pos);
                        if self.config.animation != RingAnimation::None {
                            self.anim_start = Some(Instant::now());
                        }
                    } else {
                        ctx.request_repaint();
                        return; // Stay fully transparent until center is locked
                    }
                }






                // Compute animation progress (0.0 → 1.0)
                let anim_t = match self.config.animation {
                    RingAnimation::None => 1.0,
                    RingAnimation::ScaleFade => {
                        if let Some(start) = self.anim_start {
                            let elapsed = start.elapsed().as_secs_f32();
                            let raw_t = (elapsed / ANIM_DURATION_SECS).min(1.0);
                            // Ease-out: 1 - (1 - t)^3
                            let t = 1.0 - (1.0 - raw_t).powi(3);
                            if self.closing {
                                1.0 - t // Reverse for close animation
                            } else {
                                t
                            }
                        } else {
                            1.0
                        }
                    }
                };

                // Close animation completed → actually close
                if self.closing && anim_t <= 0.01 {
                    self.close_requested = true;
                }

                // Keep repainting during animation
                if anim_t < 1.0 || self.closing {
                    ctx.request_repaint();
                }

                // Alpha multiplier for all colors during animation
                let alpha = (anim_t * 255.0) as u8;

                if self.button_released {
                    ctx.request_repaint();
                }

                let pointer_pos = raw_pointer.unwrap_or(self.center.unwrap_or(rect.center()));
                let center = self.center.unwrap();
                let total_main_slots = self.config.items.len();
                let base_main_radius = 125.0 + (total_main_slots.saturating_sub(6) as f32 * 12.0);
                let main_radius = base_main_radius * anim_t; // Scale ring radius dynamically to prevent overlap

                // Render and advance active pulse wave effects
                let mut active_pulses = Vec::new();
                for pulse in self.pulse_effects.drain(..) {
                    if pulse.render(painter, alpha) {
                        active_pulses.push(pulse);
                    }
                }
                self.pulse_effects = active_pulses;

                if !self.pulse_effects.is_empty() {
                    ctx.request_repaint();
                }

                let is_esc = ctx.input(|i| i.key_pressed(egui::Key::Escape));
                if is_esc {
                    if self.active_submenu.is_some() {
                        self.active_submenu = None;
                        self.selected_sub_slot = None;
                    } else {
                        self.begin_close();
                    }
                    return;
                }

                // Grace period: ignore raw click and release triggers during the initial 100ms of ring appearance
                let is_open_cooldown = self.anim_start.map_or(true, |t| t.elapsed().as_millis() < 100);

                // Detect specific mouse button clicks
                let is_left_clicked = !is_open_cooldown && ctx.input(|i| i.pointer.button_clicked(egui::PointerButton::Primary)) && !self.closing;
                let is_middle_clicked = !is_open_cooldown && ctx.input(|i| i.pointer.button_clicked(egui::PointerButton::Middle)) && !self.closing;
                let is_right_clicked = !is_open_cooldown && ctx.input(|i| i.pointer.button_clicked(egui::PointerButton::Secondary)) && !self.closing;
                let is_key_confirm = !is_open_cooldown && ctx.input(|i| i.key_pressed(egui::Key::Enter) || i.key_pressed(egui::Key::Space));
                let mut item_interacted = false;

                let pointer_moved = if let (Some(pos), Some(last)) = (raw_pointer, self.last_pointer) {
                    (pos.x - last.x).abs() > 2.0 || (pos.y - last.y).abs() > 2.0
                } else {
                    false
                };
                // Update last_pointer each frame for accurate per-frame movement detection
                if let Some(pos) = raw_pointer {
                    self.last_pointer = Some(pos);
                }

                let is_hold_threshold_passed = self.opened_at.elapsed().as_millis() >= 250;
                let is_release_event = self.button_released
                    && self.config.trigger_mode != crate::config::RingTriggerMode::Click
                    && !is_open_cooldown
                    && is_hold_threshold_passed;

                if self.button_released {
                    self.button_released = false;
                }

                let is_hold_mode = self.config.trigger_mode == crate::config::RingTriggerMode::HoldToRelease
                    || self.config.trigger_mode == crate::config::RingTriggerMode::Hybrid;
                let is_holding = is_hold_mode && self.button_held;

                // Wheel Navigation: right-click closes active category
                if self.config.wheel_navigation {
                    if is_right_clicked && self.active_submenu.is_some() {
                        self.active_submenu = None;
                        self.selected_sub_slot = None;
                        self.last_category_opened_at = None;
                        item_interacted = true;
                    }
                }

                let total_main_slots = self.config.items.len();
                let sector_main_slot = get_sector_slot_index(pointer_pos, center, total_main_slots);
                let is_in_deadzone = pointer_pos.distance(center) <= 35.0;

                // Compute whether the pointer is within the sub-ring area (margin zone)
                let pointer_in_sub_ring = if let Some(active_idx) = self.active_submenu {
                    let total_slots_f = total_main_slots.max(1) as f32;
                    let parent_angle = (active_idx as f32 * (2.0 * PI / total_slots_f)) - (PI / 2.0);
                    let parent_center = egui::pos2(
                        center.x + main_radius * parent_angle.cos(),
                        center.y + main_radius * parent_angle.sin(),
                    );
                    let active_sub_count = self.config.items.get(active_idx).map_or(0, |item| item.items.len());
                    let active_sub_radius = 90.0 + (active_sub_count.saturating_sub(6) as f32 * 10.0);
                    let dist_to_parent = pointer_pos.distance(parent_center);
                    dist_to_parent <= active_sub_radius + 26.0 + 10.0
                } else {
                    false
                };



                // Wheel Navigation activity window (1.2s exception window for wheel clicks)
                let scroll_active = self.last_scroll_at
                    .map_or(false, |t| t.elapsed().as_millis() < 1200);
                if self.config.wheel_navigation {
                    let scroll = ctx.input(|i| i.raw_scroll_delta);
                    if scroll.y.abs() > 0.5 || scroll.x.abs() > 0.5 {
                        self.last_scroll_at = Some(Instant::now());
                        let step = if scroll.y < -0.5 || scroll.x > 0.5 { 1 } else { -1 };
                        if let Some(sub_idx) = self.active_submenu {
                            if let Some(main_item) = self.config.items.get(sub_idx) {
                                let sub_count = main_item.items.len();
                                if sub_count > 0 {
                                    self.selected_sub_slot = Some(next_slot_index(self.selected_sub_slot, step, sub_count));
                                }
                            }
                        } else {
                            let total_slots = self.config.items.len();
                            if total_slots > 0 {
                                self.selected_main_slot = Some(next_slot_index(self.selected_main_slot, step, total_slots));
                            }
                        }
                    }
                }

                // Hold-to-Release: hover on a category slot auto-opens its submenu
                // hover on a non-category main slot auto-closes any open submenu
                // Skip when pointer is within the sub-ring area (margin zone)
                if is_holding && pointer_moved && !is_in_deadzone && !pointer_in_sub_ring {
                    if let Some(sector_idx) = sector_main_slot {
                        if let Some(item) = self.config.items.get(sector_idx) {
                            if !item.items.is_empty() {
                                // Hovering a category slot while holding → open it
                                if self.active_submenu != Some(sector_idx) {
                                    self.active_submenu = Some(sector_idx);
                                    self.selected_sub_slot = None;
                                    self.last_category_opened_at = Some(Instant::now());
                                }
                            } else {
                                // Hovering a non-category main slot while holding → close any open submenu
                                if self.active_submenu.is_some() {
                                    self.active_submenu = None;
                                    self.selected_sub_slot = None;
                                    self.last_category_opened_at = None;
                                }
                            }
                        }
                    }
                }

                let mut is_pointer_over_sub_item = false;

                if let Some(cat_idx) = self.active_submenu {
                    if let Some(main_item) = self.config.items.get(cat_idx) {
                        let total_sub_slots = main_item.items.len();
                        let parent_angle = (cat_idx as f32 * (2.0 * PI / total_main_slots.max(1) as f32)) - (PI / 2.0);
                        let parent_center = egui::pos2(
                            center.x + main_radius * parent_angle.cos(),
                            center.y + main_radius * parent_angle.sin(),
                        );
                        let sector_sub_slot = get_sector_slot_index(pointer_pos, parent_center, total_sub_slots);

                        let sub_count = total_sub_slots.max(1);
                        let sub_radius = 90.0 + (sub_count.saturating_sub(6) as f32 * 10.0);
                        let sub_item_radius = 26.0;

                        // Spatial collision check with each sub-item circle
                        for (sub_idx, _) in main_item.items.iter().enumerate() {
                            let offset_angle = (sub_idx as f32 * (2.0 * PI / sub_count as f32)) - (PI / 2.0);
                            let sub_center = egui::pos2(
                                parent_center.x + sub_radius * offset_angle.cos(),
                                parent_center.y + sub_radius * offset_angle.sin(),
                            );
                            if pointer_pos.distance(sub_center) <= sub_item_radius + 8.0 {
                                is_pointer_over_sub_item = true;
                                break;
                            }
                        }

                        // Also check if sector_sub_slot is active and pointer is outside parent slot center
                        if !is_pointer_over_sub_item && sector_sub_slot.is_some() {
                            let dist_to_parent = pointer_pos.distance(parent_center);
                            if dist_to_parent > 35.0 && dist_to_parent <= sub_radius + 35.0 {
                                is_pointer_over_sub_item = true;
                            }
                        }
                    }
                }

                // 1. Central button (Close "✕")
                let center_radius = 22.0 * anim_t;
                let is_center_hovered = pointer_pos.distance(center) <= center_radius;

                let center_color = if is_center_hovered {
                    egui::Color32::from_rgb(220, 50, 60)
                } else {
                    egui::Color32::from_rgb(180, 180, 190)
                };
                let center_icon = "✕";

                render_glowing_circle(
                    painter,
                    center,
                    center_radius,
                    center_color,
                    if is_center_hovered { egui::Color32::from_rgb(255, 70, 80) } else { egui::Color32::WHITE },
                    is_center_hovered,
                    alpha,
                    self.config.glow_effect,
                );

                painter.text(
                    center,
                    egui::Align2::CENTER_CENTER,
                    center_icon,
                    egui::FontId::proportional(15.0 * anim_t),
                    with_alpha(egui::Color32::from_rgb(40, 40, 40), alpha),
                );

                let mut trigger_close = false;
                let mut category_just_opened = false;

                if is_center_hovered {
                    item_interacted = true;
                    if is_left_clicked {
                        trigger_close = true;
                        if self.config.pulse_effect {
                            self.pulse_effects.push(super::state::PulseEffect::new(
                                center,
                                egui::Color32::from_rgb(230, 50, 60),
                            ));
                        }
                    }
                }


                let mut newly_hovered_main = None;
                let mut _newly_hovered_sub = None;

                let default_norm_str = self.config.default_color.clone();
                let default_act_str = self.config.default_active_color.clone();

                let total_slots = self.config.items.len().max(1);
                let is_any_submenu_open = self.active_submenu.is_some();

                // 2. Render Main Ring Menu (dynamic slot angle distribution)
                for (idx, item) in self.config.items.iter().enumerate() {
                    let angle = (idx as f32 * (2.0 * PI / total_slots as f32)) - (PI / 2.0);
                    let item_center = egui::pos2(
                        center.x + main_radius * angle.cos(),
                        center.y + main_radius * angle.sin(),
                    );

                    let item_radius = 32.0;
                    let dist_to_item = pointer_pos.distance(item_center);
                    let is_circle_hovered = !is_in_deadzone && dist_to_item <= item_radius + 6.0;

                    let is_wheel_active = scroll_active && self.selected_main_slot == Some(idx);
                    let is_selected = if scroll_active {
                        is_circle_hovered || is_wheel_active
                    } else {
                        is_circle_hovered
                    };

                    let is_active_slot = is_selected && self.active_submenu.is_none();
                    let is_submenu_open = self.active_submenu == Some(idx);

                    if is_circle_hovered {
                        self.selected_main_slot = Some(idx);
                        newly_hovered_main = Some(idx);
                        item_interacted = true;
                    } else if is_wheel_active {
                        newly_hovered_main = Some(idx);
                        item_interacted = true;
                    }

                    // Calculate effective color
                    let item_norm_hex = item.color.as_deref().unwrap_or(&default_norm_str);
                    let item_act_hex = item.active_color.as_deref().unwrap_or(&default_act_str);

                    let norm_color = parse_hex_color(item_norm_hex).unwrap_or(egui::Color32::from_rgb(40, 44, 55));
                    let act_color = parse_hex_color(item_act_hex).unwrap_or(egui::Color32::from_rgb(0, 215, 175));

                    if newly_hovered_main == Some(idx) && self.last_hovered_main != Some(idx) {
                        if self.config.pulse_effect {
                            self.pulse_effects.push(super::state::PulseEffect::new(item_center, act_color));
                        }
                    }

                    let (mut fill_color, mut text_color) = if is_submenu_open || is_active_slot {
                        (act_color, contrasting_text_color(act_color))
                    } else {
                        (norm_color, contrasting_text_color(norm_color))
                    };

                    let is_glowing = is_submenu_open || is_active_slot;
                    let slot_alpha_mult = if is_any_submenu_open && !is_active_slot && !is_submenu_open {
                        (alpha as u16 * 100 / 255) as u8
                    } else {
                        alpha
                    };

                    if is_any_submenu_open && !is_active_slot && !is_submenu_open {
                        fill_color = with_alpha(fill_color, slot_alpha_mult);
                        text_color = with_alpha(text_color, slot_alpha_mult);
                    }

                    render_glowing_circle(
                        painter,
                        item_center,
                        item_radius,
                        fill_color,
                        act_color,
                        is_glowing,
                        slot_alpha_mult,
                        self.config.glow_effect,
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

                    // Submenu indicator dot
                    if !item.items.is_empty() {
                        let badge_pos = egui::pos2(
                            item_center.x + item_radius * 0.75 * angle.cos(),
                            item_center.y + item_radius * 0.75 * angle.sin(),
                        );
                        painter.circle_filled(
                            badge_pos,
                            5.0,
                            if is_submenu_open { egui::Color32::WHITE } else { egui::Color32::BLACK },
                        );
                    }

                    // Interaction on main slot
                    if is_submenu_open && is_circle_hovered && !is_pointer_over_sub_item {
                        item_interacted = true;
                        if is_left_clicked {
                            self.active_submenu = None;
                            self.selected_sub_slot = None;
                            self.last_category_opened_at = None;
                        }
                    } else if !is_pointer_over_sub_item {
                        let should_trigger_slot = if is_selected && !is_open_cooldown {
                            if is_left_clicked || is_key_confirm {
                                is_circle_hovered || scroll_active
                            } else if is_middle_clicked && (is_circle_hovered || scroll_active) {
                                true // Middle click on category/slot → open or trigger (during wheel nav or physical hover)
                            } else if is_release_event && item.items.is_empty() {
                                // Release on a non-category slot: REQUIRE physical circle hover during hold-to-release!
                                is_circle_hovered
                            } else {
                                false
                            }
                        } else {
                            false
                        };

                        if should_trigger_slot {
                            item_interacted = true;
                            if self.config.pulse_effect {
                                self.pulse_effects.push(super::state::PulseEffect::new(item_center, act_color));
                            }
                            if !item.items.is_empty() {
                                // Open category (not already open)
                                if self.active_submenu != Some(idx) {
                                    self.active_submenu = Some(idx);
                                    self.selected_sub_slot = None;
                                    self.last_category_opened_at = Some(Instant::now());
                                    category_just_opened = true;
                                }
                            } else if let Some(action) = &item.action {
                                execute_action(action);
                                if item.auto_close {
                                    self.close_requested = true;
                                }
                            }
                        }
                    }
                }

                self.last_hovered_main = newly_hovered_main;

                // 3. Render Outer Submenu (full 360° circle)
                let release_closes_category = if is_release_event && is_hold_threshold_passed {
                    if let Some(active_idx) = self.active_submenu {
                        sector_main_slot == Some(active_idx)
                    } else {
                        false
                    }
                } else {
                    false
                };

                if release_closes_category {
                    self.active_submenu = None;
                    self.selected_sub_slot = None;
                    self.last_category_opened_at = None;
                } else if let Some(active_idx) = self.active_submenu {
                    if let Some(main_item) = self.config.items.get(active_idx) {
                        let total_slots = self.config.items.len().max(1);
                        let main_angle = (active_idx as f32 * (2.0 * PI / total_slots as f32)) - (PI / 2.0);
                        let main_item_center = egui::pos2(
                            center.x + main_radius * main_angle.cos(),
                            center.y + main_radius * main_angle.sin(),
                        );

                        let sub_count = main_item.items.len().max(1);
                        let sub_radius = 90.0 + (sub_count.saturating_sub(6) as f32 * 10.0);
                        let sub_item_radius = 26.0;

                        let is_category_cooldown = self.last_category_opened_at
                            .map_or(false, |t| t.elapsed().as_millis() < 250);



                        for (sub_idx, sub_item) in main_item.items.iter().enumerate() {
                            let offset_angle = (sub_idx as f32 * (2.0 * PI / sub_count as f32)) - (PI / 2.0);

                            let sub_center = egui::pos2(
                                main_item_center.x + sub_radius * offset_angle.cos(),
                                main_item_center.y + sub_radius * offset_angle.sin(),
                            );

                            let dist_to_sub = pointer_pos.distance(sub_center);
                            let is_sub_circle_hovered = dist_to_sub <= sub_item_radius + 6.0;
                            let is_sub_wheel_active = scroll_active && self.selected_sub_slot == Some(sub_idx);

                            let is_sub_active = if scroll_active {
                                is_sub_circle_hovered || is_sub_wheel_active
                            } else {
                                is_sub_circle_hovered
                            };

                            if is_sub_circle_hovered {
                                self.selected_sub_slot = Some(sub_idx);
                                _newly_hovered_sub = Some((active_idx, sub_idx));
                                item_interacted = true;
                            } else if is_sub_wheel_active {
                                _newly_hovered_sub = Some((active_idx, sub_idx));
                                item_interacted = true;
                            }

                            // Submenu item colors
                            let sub_norm_hex = sub_item.color.as_deref().unwrap_or(&default_norm_str);
                            let sub_act_hex = sub_item.active_color.as_deref().unwrap_or(&default_act_str);

                            let sub_norm_color = parse_hex_color(sub_norm_hex).unwrap_or(egui::Color32::from_rgb(50, 55, 70));
                            let sub_act_color = parse_hex_color(sub_act_hex).unwrap_or(egui::Color32::from_rgb(0, 215, 175));

                            if _newly_hovered_sub == Some((active_idx, sub_idx)) && self.last_hovered_sub != Some((active_idx, sub_idx)) {
                                if self.config.pulse_effect {
                                    self.pulse_effects.push(super::state::PulseEffect::new(sub_center, sub_act_color));
                                }
                            }

                            let (sub_fill, sub_text) = if is_sub_active {
                                (sub_act_color, contrasting_text_color(sub_act_color))
                            } else {
                                (sub_norm_color, contrasting_text_color(sub_norm_color))
                            };

                            render_glowing_circle(
                                painter,
                                sub_center,
                                sub_item_radius,
                                sub_fill,
                                sub_act_color,
                                is_sub_active,
                                alpha,
                                self.config.glow_effect,
                            );

                            crate::icon_loader::render_icon_or_emoji(
                                painter,
                                ctx,
                                &mut self.texture_cache,
                                sub_center,
                                sub_item_radius,
                                &sub_item.icon,
                                19.0,
                                sub_text,
                            );

                            // Tooltip on hovering/selecting submenu item
                            if is_sub_active {
                                let lang = &self.lang;
                                let badge_text = sub_item.label_lang(lang);
                                let font_id = egui::FontId::proportional(14.0);
                                let text_size = ui.fonts(|f| f.layout_no_wrap(badge_text.to_string(), font_id.clone(), egui::Color32::BLACK)).rect.size();

                                let badge_center = egui::pos2(
                                    sub_center.x + (sub_item_radius + text_size.x / 2.0 + 14.0) * offset_angle.cos(),
                                    sub_center.y + (sub_item_radius + 12.0) * offset_angle.sin(),
                                );

                                let badge_rect = egui::Rect::from_center_size(
                                    badge_center,
                                    egui::vec2(text_size.x + 24.0, 32.0),
                                );

                                painter.rect_filled(badge_rect, 16.0, egui::Color32::WHITE);
                                painter.rect_stroke(badge_rect, 16.0, egui::Stroke::new(1.5f32, egui::Color32::from_gray(200)));

                                painter.text(
                                    badge_center,
                                    egui::Align2::CENTER_CENTER,
                                    badge_text,
                                    font_id,
                                    egui::Color32::BLACK,
                                );
                            }

                            // Trigger interaction on submenu item
                            let should_trigger_sub = if is_sub_active && !category_just_opened && !is_category_cooldown && !is_open_cooldown {
                                if is_left_clicked || is_middle_clicked || is_key_confirm {
                                    true
                                } else if is_release_event {
                                    is_sub_circle_hovered && is_hold_threshold_passed
                                } else {
                                    false
                                }
                            } else {
                                false
                            };

                            if should_trigger_sub {
                                if self.config.pulse_effect {
                                    self.pulse_effects.push(super::state::PulseEffect::new(sub_center, sub_act_color));
                                }
                                if let Some(action) = &sub_item.action {
                                    execute_action(action);
                                }
                                if sub_item.auto_close {
                                    self.close_requested = true;
                                }
                            }
                        }
                    }
                    self.last_hovered_sub = _newly_hovered_sub;
                }

                // Close handling
                if is_left_clicked && !item_interacted {
                    trigger_close = true;
                }
                if is_release_event && !item_interacted {
                    if is_hold_mode && is_hold_threshold_passed && self.active_submenu.is_none() && !self.config.wheel_navigation {
                        trigger_close = true;
                    }
                }

                if trigger_close {
                    self.begin_close();
                }

                // 4. Tooltip on hovering center button or main slot
                let lang = &self.lang;
                if is_center_hovered && newly_hovered_main.is_none() {
                    let tooltip_text = crate::utils::i18n::tr(lang, "close_tooltip").to_string();

                    let label_pos = egui::pos2(center.x, center.y + main_radius + 60.0);
                    let font_id = egui::FontId::proportional(14.0);
                    let text_size = ui.fonts(|f| f.layout_no_wrap(tooltip_text.clone(), font_id.clone(), egui::Color32::WHITE)).rect.size();

                    let label_rect = egui::Rect::from_center_size(
                        label_pos,
                        egui::vec2(text_size.x + 20.0, 30.0),
                    );

                    painter.rect_filled(label_rect, 15.0, egui::Color32::from_black_alpha(220));
                    painter.text(
                        label_pos,
                        egui::Align2::CENTER_CENTER,
                        &tooltip_text,
                        font_id,
                        egui::Color32::WHITE,
                    );
                } else if let Some(hovered_idx) = newly_hovered_main {
                    if self.active_submenu.is_none() || self.active_submenu != Some(hovered_idx) {
                        if let Some(item) = self.config.items.get(hovered_idx) {
                            let item_label = item.label_lang(lang);
                            let label_pos = egui::pos2(center.x, center.y + main_radius + 60.0);
                            let font_id = egui::FontId::proportional(14.0);
                            let text_size = ui.fonts(|f| f.layout_no_wrap(item_label.to_string(), font_id.clone(), egui::Color32::WHITE)).rect.size();

                            let label_rect = egui::Rect::from_center_size(
                                label_pos,
                                egui::vec2(text_size.x + 20.0, 30.0),
                            );

                            painter.rect_filled(label_rect, 15.0, egui::Color32::from_black_alpha(220));
                            painter.text(
                                label_pos,
                                egui::Align2::CENTER_CENTER,
                                item_label,
                                font_id,
                                egui::Color32::WHITE,
                            );
                        }
                    }
                }

                // 5. Active Application Name Badge below the ring
                let is_any_tooltip_shown = newly_hovered_main.is_some() || is_center_hovered;
                let app_badge_y = if is_any_tooltip_shown {
                    center.y + main_radius + 95.0
                } else {
                    center.y + main_radius + 60.0
                };
                let badge_pos = egui::pos2(center.x, app_badge_y);
                let badge_text = if self.is_app_specific {
                    format!("🎯 {}", self.display_app_name())
                } else {
                    crate::utils::i18n::tr(lang, "general_menu_badge").replace("{}", &self.display_app_name())
                };
                let font_id = egui::FontId::proportional(12.0 * anim_t);
                let text_size = ui.fonts(|f| f.layout_no_wrap(badge_text.clone(), font_id.clone(), egui::Color32::WHITE)).rect.size();

                let badge_rect = egui::Rect::from_center_size(
                    badge_pos,
                    egui::vec2(text_size.x + 20.0, 24.0 * anim_t),
                );

                painter.rect_filled(
                    badge_rect,
                    12.0 * anim_t,
                    with_alpha(egui::Color32::from_black_alpha(200), (alpha as u16 * 200 / 255) as u8),
                );
                painter.rect_stroke(
                    badge_rect,
                    12.0 * anim_t,
                    egui::Stroke::new(1.0_f32, with_alpha(egui::Color32::from_rgb(0, 215, 175), alpha)),
                );

                painter.text(
                    badge_pos,
                    egui::Align2::CENTER_CENTER,
                    &badge_text,
                    font_id,
                    with_alpha(egui::Color32::from_rgb(0, 215, 175), alpha),
                );
            });
    }
}
