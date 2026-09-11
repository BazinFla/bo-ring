use crate::config::{ButtonActionConfig, RingMenuConfig};
use crate::platform::ipc::{daemon_socket_path, DaemonIpcCommand, send_command_to_daemon};
pub use crate::utils::path::expand_path_arg;
use eframe::egui;
use std::process::Command;
use std::time::Instant;

/// Duration of the scale+fade animation in seconds.
pub const ANIM_DURATION_SECS: f32 = 0.20;

/// Calculate next circular slot index given a scroll step and total slot count.
pub fn next_slot_index(current: Option<usize>, step: i32, total_slots: usize) -> usize {
    if total_slots == 0 {
        return 0;
    }
    let curr = current.unwrap_or(0);
    if step > 0 {
        (curr + 1) % total_slots
    } else if step < 0 {
        (curr + total_slots - 1) % total_slots
    } else {
        curr % total_slots
    }
}

pub fn connect_to_daemon_ipc() -> Option<std::sync::mpsc::Receiver<String>> {
    let (tx, rx) = std::sync::mpsc::channel();
    let sock_path = daemon_socket_path();
    if let Ok(stream) = std::os::unix::net::UnixStream::connect(&sock_path) {
        std::thread::spawn(move || {
            use std::io::{BufRead, BufReader};
            let reader = BufReader::new(stream);
            for line in reader.lines().map_while(Result::ok) {
                if tx.send(line).is_err() {
                    break;
                }
            }
        });
        Some(rx)
    } else {
        None
    }
}

#[derive(Clone, Debug)]
pub struct PulseEffect {
    pub center: egui::Pos2,
    pub start_time: Instant,
    pub duration_secs: f32,
    pub max_radius: f32,
    pub color: egui::Color32,
}

impl PulseEffect {
    pub fn new(center: egui::Pos2, color: egui::Color32) -> Self {
        Self {
            center,
            start_time: Instant::now(),
            duration_secs: 0.45,
            max_radius: 110.0,
            color,
        }
    }

    pub fn render(&self, painter: &egui::Painter, alpha_mult: u8) -> bool {
        let elapsed = self.start_time.elapsed().as_secs_f32();
        if elapsed >= self.duration_secs {
            return false;
        }

        let progress = (elapsed / self.duration_secs).min(1.0);
        let ease = 1.0 - (1.0 - progress).powi(3);

        let current_radius = ease * self.max_radius;
        let fade = (1.0 - progress).powi(2);

        let base_alpha = (self.color.a() as f32 * fade) as u8;
        let final_alpha = (base_alpha as u16 * alpha_mult as u16 / 255) as u8;

        if final_alpha > 2 {
            let ring_color = crate::utils::with_alpha(self.color, final_alpha);
            painter.circle_stroke(
                self.center,
                current_radius,
                egui::Stroke::new(3.5 * (1.0 - progress * 0.5), ring_color),
            );

            if progress > 0.15 {
                let echo_progress = (progress - 0.15) / 0.85;
                let echo_radius = (1.0 - (1.0 - echo_progress).powi(3)) * (self.max_radius * 0.75);
                let echo_alpha = ((base_alpha as f32 * 0.5 * (1.0 - echo_progress)) as u16 * alpha_mult as u16 / 255) as u8;
                if echo_alpha > 2 {
                    painter.circle_stroke(
                        self.center,
                        echo_radius,
                        egui::Stroke::new(1.5_f32, crate::utils::with_alpha(self.color, echo_alpha)),
                    );
                }
            }
        }

        true
    }
}

pub struct RingMenuApp {
    pub config: RingMenuConfig,
    pub active_submenu: Option<usize>,
    pub close_requested: bool,
    pub texture_cache: crate::icon_loader::TextureCache,
    /// Ring center position in window-local coordinates.
    /// `None` until a real pointer motion is detected (see Wayland workaround below).
    pub center: Option<egui::Pos2>,
    /// Previous frame's pointer position, used to detect actual mouse movement.
    /// Required because Wayland compositors report stale/incorrect coordinates
    /// in the initial `wl_pointer.enter` event when a new surface appears.
    pub last_pointer: Option<egui::Pos2>,
    /// Timestamp when the open or close animation started.
    pub anim_start: Option<Instant>,
    /// Whether the close animation is currently playing.
    pub closing: bool,
    /// Selected main slot index when using wheel navigation or mouse sector.
    pub selected_main_slot: Option<usize>,
    /// Selected sub-item slot index inside open category.
    pub selected_sub_slot: Option<usize>,
    /// Channel receiver for IPC events from the background daemon.
    pub ipc_rx: Option<std::sync::mpsc::Receiver<String>>,
    /// Flag set when gesture button release event is received via IPC.
    pub button_released: bool,
    /// Whether the gesture button is currently held down (for Hold-to-Release mode).
    pub button_held: bool,
    /// Timestamp when the ring app opened.
    pub opened_at: Instant,
    /// Timestamp when a category submenu was opened (used for cooldown guard).
    pub last_category_opened_at: Option<Instant>,
    /// Timestamp of the last wheel scroll event (used to suppress mouse hover jitter).
    pub last_scroll_at: Option<Instant>,
    /// Active application window name detected when the ring menu opened.
    pub active_app_name: String,


    /// Channel receiver for async background active window detection.
    pub active_app_rx: Option<std::sync::mpsc::Receiver<String>>,
    /// Application profiles list from configuration.
    pub app_profiles: Vec<crate::config::AppProfileConfig>,
    /// Default global ring menu configuration.
    pub default_ring_menu: RingMenuConfig,
    /// Whether an app-specific action ring profile is currently displayed.
    pub is_app_specific: bool,
    /// Active pulse wave animation effects triggered on slot/button interactions.
    pub pulse_effects: Vec<PulseEffect>,
    /// Last hovered main slot index (used to trigger hover pulse).
    pub last_hovered_main: Option<usize>,
    /// Last hovered submenu item index (used to trigger hover pulse).
    pub last_hovered_sub: Option<(usize, usize)>,
    /// Whether the Wayland pointer motion nudge has already been emitted once.
    pub has_nudged: bool,
    /// Resolved active language code for the ring UI.
    pub lang: String,
}


impl RingMenuApp {
    pub fn new(
        default_ring_menu: RingMenuConfig,
        app_profiles: Vec<crate::config::AppProfileConfig>,
        initial_app: Option<String>,
        lang: String,
    ) -> Self {
        let ipc_rx = connect_to_daemon_ipc();

        let (initial_config, is_app_specific, active_app_name, active_app_rx) = if let Some(ref app_name) = initial_app {
            let raw = app_name.trim();
            let custom_ring = app_profiles.iter()
                .find(|p| crate::config::matches_app_id(&p.app_id, raw))
                .and_then(|p| p.ring_menu.clone());

            if let Some(mut ring) = custom_ring {
                ring.trigger_mode = default_ring_menu.trigger_mode.clone();
                ring.wheel_navigation = default_ring_menu.wheel_navigation;
                ring.glow_effect = default_ring_menu.glow_effect;
                ring.pulse_effect = default_ring_menu.pulse_effect;
                ring.animation = default_ring_menu.animation.clone();
                (ring, true, app_name.clone(), None)
            } else {
                (default_ring_menu.clone(), false, app_name.clone(), None)
            }
        } else {
            let (tx, rx) = std::sync::mpsc::channel();
            std::thread::spawn(move || {
                let app = crate::active_window::detect_active_window();
                let _ = tx.send(app);
            });
            (default_ring_menu.clone(), false, "Detecting...".to_string(), Some(rx))
        };

        Self {
            config: initial_config,
            default_ring_menu,
            active_submenu: None,
            close_requested: false,
            texture_cache: crate::icon_loader::TextureCache::new(),
            center: None,
            last_pointer: None,
            anim_start: None,
            closing: false,
            selected_main_slot: None,
            selected_sub_slot: None,
            ipc_rx,
            button_released: false,
            button_held: true, // Assumes the gesture button is held when the ring opens
            opened_at: Instant::now(),
            last_category_opened_at: None,
            last_scroll_at: None,
            active_app_name,
            active_app_rx,
            app_profiles,
            is_app_specific,
            pulse_effects: Vec::new(),
            last_hovered_main: None,
            last_hovered_sub: None,
            has_nudged: false,
            lang,
        }
    }



    pub fn display_app_name(&self) -> String {
        self.display_app_name_lang(&self.lang)
    }

    pub fn display_app_name_lang(&self, lang: &str) -> String {
        let raw = self.active_app_name.trim();
        if raw.is_empty() || crate::utils::i18n::matches_key_any_locale("detecting", raw) {
            return crate::utils::i18n::tr(lang, "detecting").to_string();
        }

        let desktop_candidate = raw.strip_prefix("Global / ").unwrap_or(raw).trim();
        if desktop_candidate.eq_ignore_ascii_case("desktop")
            || crate::utils::i18n::matches_key_any_locale("desktop", desktop_candidate)
        {
            return crate::utils::i18n::tr(lang, "desktop").to_string();
        }

        if let Some(profile) = self.app_profiles.iter().find(|p| crate::config::matches_app_id(&p.app_id, raw)) {
            let label = profile.label_lang(lang).trim();
            if !label.is_empty() {
                return label.to_string();
            }
        }

        raw.to_string()
    }

    /// Begin the close animation smoothly so pulse animations and fade transitions complete.
    pub fn begin_close(&mut self) {
        if self.closing { return; }
        self.closing = true;
        self.anim_start = Some(Instant::now());
    }
}

pub fn send_action_to_daemon(action: &ButtonActionConfig) -> bool {
    let resolved = crate::config::resolve_action(action, &[]);
    match &resolved {
        ButtonActionConfig::ShowRingMenu | ButtonActionConfig::ActionRef { .. } => false,
        ButtonActionConfig::Command { cmd } => {
            send_command_to_daemon(&DaemonIpcCommand::ExecCmd(cmd.clone()))
        }
        ButtonActionConfig::KeyCombo { keys } => {
            send_command_to_daemon(&DaemonIpcCommand::ExecKey(keys.clone()))
        }
    }
}

pub fn execute_action_direct(action: &ButtonActionConfig) {
    let resolved = crate::config::resolve_action(action, &[]);
    match &resolved {
        ButtonActionConfig::ShowRingMenu | ButtonActionConfig::ActionRef { .. } => {
            println!("✨ Action: Opening Ring Menu");
        }
        ButtonActionConfig::Command { cmd } => {
            println!("🚀 Running command: '{}'", cmd);
            let trimmed = cmd.trim();
            if trimmed.contains('|') || trimmed.contains('&') || trimmed.contains(';') || trimmed.contains('>') || trimmed.contains('<') || trimmed.contains('$') {
                let _ = Command::new("sh").args(["-c", trimmed]).spawn();
            } else {
                let mut parts = trimmed.split_whitespace();
                if let Some(program) = parts.next() {
                    let raw_args: Vec<&str> = parts.collect();
                    let expanded_args: Vec<String> = raw_args.iter().map(|a| expand_path_arg(a)).collect();
                    let _ = Command::new(program).args(&expanded_args).spawn();
                }
            }
        }
        ButtonActionConfig::KeyCombo { keys } => {
            let keys_to_send = keys.clone();
            let handle = std::thread::spawn(move || {
                std::thread::sleep(std::time::Duration::from_millis(100));
                crate::keyboard::send_key_combo(&keys_to_send);
            });
            let _ = handle.join();
        }
    }
}

pub fn execute_action(action: &ButtonActionConfig) {
    let resolved = crate::config::resolve_action(action, &[]);
    if send_action_to_daemon(&resolved) {
        println!("📡 Action delegated to background daemon via IPC: {:?}", resolved);
        return;
    }
    execute_action_direct(&resolved);
}
