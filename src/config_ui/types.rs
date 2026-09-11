use crate::config::{AppProfileConfig, Config, RingMenuConfig};
use eframe::egui;
use std::collections::{HashMap, HashSet};
use std::path::PathBuf;
use std::sync::{mpsc, Arc, Mutex};
use std::thread;

use crate::devices::{
    attach_new_mouse_listeners, scan_mouse_device, ButtonCalloutSide, DeviceModelConfig,
    DeviceRegistry, LoadedDeviceProfile, MouseDeviceInfo, ProfileSource, BTN_RING_CODE,
};
use crate::i18n::tr;

#[derive(PartialEq, Clone, Copy, Debug)]
pub enum ConfigTab {
    ButtonConfig,
    RingCustomizer,
    GlobalSettings,
    Debug,
}

#[derive(Clone, PartialEq, Debug)]
pub enum SelectedNodePath {
    Slot(usize),
    SubItem(usize, usize),
    NestedSubItem(usize, usize, usize),
}

#[derive(Clone, PartialEq, Debug)]
pub enum ShortcutTarget {
    Button(u16),
    RingNode(SelectedNodePath),
}

#[derive(Clone, Copy, PartialEq, Eq, Debug)]
pub enum ActionCategory {
    All,
    Navigation,
    Media,
    Windows,
    Shortcuts,
    Apps,
    System,
}

impl ActionCategory {
    pub fn label(&self, lang: &str) -> &'static str {
        match self {
            ActionCategory::All => match lang {
                "fr" => "⭐ Tout",
                "es" => "⭐ Todo",
                _ => "⭐ All",
            },
            ActionCategory::Navigation => match lang {
                "fr" => "🧭 Navigation",
                "es" => "🧭 Navegación",
                _ => "🧭 Navigation",
            },
            ActionCategory::Media => match lang {
                "fr" => "🎵 Médias",
                "es" => "🎵 Medios",
                _ => "🎵 Media",
            },
            ActionCategory::Windows => match lang {
                "fr" => "🪟 Fenêtres",
                "es" => "🪟 Ventanas",
                _ => "🪟 Windows",
            },
            ActionCategory::Shortcuts => match lang {
                "fr" => "⌨️ Raccourci",
                "es" => "⌨️ Atajo",
                _ => "⌨️ Shortcut",
            },
            ActionCategory::Apps => match lang {
                "fr" => "🚀 Applications",
                "es" => "🚀 Aplicaciones",
                _ => "🚀 Apps",
            },
            ActionCategory::System => match lang {
                "fr" => "⚙️ Système",
                "es" => "⚙️ Sistema",
                _ => "⚙️ System",
            },
        }
    }
}

#[derive(Clone, Copy, PartialEq, Eq, Debug)]
pub enum EditorDetectTarget {
    NewButton,
    EditModalButton,
    NewDeviceHardware,
    CurrentDeviceHardware,
}

pub struct ConfigApp {
    pub config: Config,
    pub texture: Option<egui::TextureHandle>,
    pub active_tab: ConfigTab,
    pub selected_button_code: u16,
    pub selected_node: SelectedNodePath,
    pub status_message: String,
    pub toast: Option<(String, std::time::Instant, bool)>,
    pub debug_log_output: String,

    pub action_search_query: String,
    pub selected_action_category: ActionCategory,
    pub custom_cmd_input: String,

    pub event_rx: Option<mpsc::Receiver<crate::devices::MouseInputEvent>>,
    pub event_tx: Option<mpsc::Sender<crate::devices::MouseInputEvent>>,
    pub listened_paths: Arc<Mutex<HashSet<PathBuf>>>,
    pub recording_shortcut_for: Option<ShortcutTarget>,
    pub texture_cache: crate::icon_loader::TextureCache,
    pub mouse_info: Option<MouseDeviceInfo>,
    pub scan_rx: Option<mpsc::Receiver<Option<MouseDeviceInfo>>>,
    pub last_auto_scan: std::time::Instant,
    pub anchor_positions: HashMap<u16, egui::Vec2>,
    pub closed_category: Option<usize>,
    pub dragging_slot: Option<usize>,
    pub dragging_sub_item: Option<(usize, usize)>,
    pub dragging_button: Option<u16>,
    pub app_profiles: Vec<AppProfileConfig>,
    pub active_profile_index: usize,
    pub show_profile_modal: bool,
    pub editing_profile_index: Option<usize>,
    pub modal_app_id: String,
    pub modal_label: String,
    pub modal_logo_path: String,
    pub confirming_delete: bool,
    pub prevent_profile_click_frame: u8,
    pub device_registry: DeviceRegistry,
    pub active_device_profile: DeviceModelConfig,
    pub editor_detecting: Option<EditorDetectTarget>,
    pub show_add_button_modal: bool,
    pub new_btn_code: u16,
    pub new_btn_name: String,
    pub new_btn_icon: String,
    pub new_btn_side: ButtonCalloutSide,
    pub deleting_button_code: Option<u16>,
    pub show_edit_button_modal: bool,
    pub edit_btn_code: u16,
    pub edit_btn_old_code: u16,
    pub edit_btn_name: String,
    pub edit_btn_icon: String,
    pub edit_btn_side: ButtonCalloutSide,
    pub edit_btn_ratio: f32,
    pub show_new_device_modal: bool,
    pub new_device_name: String,
    pub new_device_id: String,
    pub new_device_id_edited: bool,
    pub new_device_duplicate_active: bool,
    pub new_device_match_ids: Vec<String>,
}

impl ConfigApp {
    pub fn new(ctx: &egui::Context, config: Config) -> Self {
        crate::icon_loader::setup_custom_fonts(ctx);

        let device_registry = DeviceRegistry::new();
        let initial_profile = if config.general.device_profile != "auto" {
            device_registry
                .get_profile(&config.general.device_profile)
                .cloned()
                .unwrap_or_else(|| device_registry.get_default_profile().clone())
        } else {
            device_registry.get_default_profile().clone()
        };

        let texture = device_registry.load_texture(ctx, &initial_profile);
        let anchor_positions = initial_profile.config.default_anchor_positions();
        let active_device_profile = initial_profile.config;

        let (tx, rx) = mpsc::channel();
        let listened_paths = Arc::new(Mutex::new(HashSet::new()));

        attach_new_mouse_listeners(ctx.clone(), tx.clone(), Arc::clone(&listened_paths));

        let lang_code = config.general.language.clone();
        let initial_status = tr(&lang_code, "status_physical_listen").to_string();

        let initial_btn_code = active_device_profile
            .buttons
            .first()
            .map(|b| b.code)
            .unwrap_or(BTN_RING_CODE);

        let app_profiles = config.app_profiles.clone();

        let mut app = Self {
            config,
            texture,
            active_tab: ConfigTab::ButtonConfig,
            selected_button_code: initial_btn_code,
            selected_node: SelectedNodePath::Slot(0),
            status_message: initial_status,
            toast: None,
            debug_log_output: String::new(),

            action_search_query: String::new(),
            selected_action_category: ActionCategory::All,
            custom_cmd_input: String::new(),

            event_rx: Some(rx),
            event_tx: Some(tx),
            listened_paths,
            recording_shortcut_for: None,
            texture_cache: crate::icon_loader::TextureCache::new(),
            mouse_info: None,
            scan_rx: None,
            last_auto_scan: std::time::Instant::now(),
            anchor_positions,
            closed_category: None,
            dragging_slot: None,
            dragging_sub_item: None,
            dragging_button: None,
            app_profiles,
            active_profile_index: 0,
            show_profile_modal: false,
            editing_profile_index: None,
            modal_app_id: String::new(),
            modal_label: String::new(),
            modal_logo_path: String::new(),
            confirming_delete: false,
            prevent_profile_click_frame: 0,
            device_registry,
            active_device_profile,
            editor_detecting: None,
            show_add_button_modal: false,
            new_btn_code: 0,
            new_btn_name: String::new(),
            new_btn_icon: "🔘".to_string(),
            new_btn_side: ButtonCalloutSide::Right,
            deleting_button_code: None,
            show_edit_button_modal: false,
            edit_btn_code: 0,
            edit_btn_old_code: 0,
            edit_btn_name: String::new(),
            edit_btn_icon: "🔘".to_string(),
            edit_btn_side: ButtonCalloutSide::Right,
            edit_btn_ratio: 0.5,
            show_new_device_modal: false,
            new_device_name: String::new(),
            new_device_id: String::new(),
            new_device_id_edited: false,
            new_device_duplicate_active: true,
            new_device_match_ids: Vec::new(),
        };

        app.trigger_async_mouse_scan(ctx.clone());
        app
    }

    pub fn select_device_profile(&mut self, ctx: &egui::Context, profile_id: &str) {
        if let Some(p) = self.device_registry.get_profile(profile_id).cloned() {
            self.texture = self.device_registry.load_texture(ctx, &p);
            self.anchor_positions = p.config.default_anchor_positions();
            if !p.config.buttons.iter().any(|b| b.code == self.selected_button_code) {
                if let Some(first) = p.config.buttons.first() {
                    self.selected_button_code = first.code;
                }
            }
            self.active_device_profile = p.config;
        }
    }

    pub fn open_edit_button_modal(&mut self, code: u16) {
        if let Some(btn) = self.active_device_profile.find_button(code).cloned() {
            let lang = self.lang();
            self.edit_btn_old_code = code;
            self.edit_btn_code = code;
            self.edit_btn_name = if btn.default_name.is_empty() {
                btn.display_name(&lang)
            } else {
                btn.default_name
            };
            self.edit_btn_icon = btn.icon;
            self.edit_btn_side = btn.side;
            self.edit_btn_ratio = btn.badge_y_ratio.unwrap_or(0.5);
            self.show_edit_button_modal = true;
            self.editor_detecting = None;
        }
    }

    pub fn update_device_image(&mut self, ctx: &egui::Context, file_path: &std::path::Path) {
        let lang = self.lang();
        let img_bytes = match std::fs::read(file_path) {
            Ok(b) => b,
            Err(e) => {
                eprintln!("Failed to read image file: {}", e);
                self.notify_error(tr(&lang, "editor_image_read_failed"));
                return;
            }
        };

        let decoded = match image::load_from_memory(&img_bytes) {
            Ok(img) => img,
            Err(e) => {
                eprintln!("Failed to decode image file: {}", e);
                self.notify_error(tr(&lang, "editor_image_decode_failed"));
                return;
            }
        };

        let user_images_dir = DeviceRegistry::ensure_user_devices_dir().join("images");
        if let Err(e) = std::fs::create_dir_all(&user_images_dir) {
            eprintln!("⚠️ Could not create user images directory {:?}: {}", user_images_dir, e);
        }

        let ext = file_path
            .extension()
            .and_then(|e| e.to_str())
            .unwrap_or("png")
            .to_lowercase();
        let target_filename = format!("{}.{}", self.active_device_profile.id, ext);
        let dest_path = user_images_dir.join(&target_filename);

        let saved_image_ref = match std::fs::write(&dest_path, &img_bytes) {
            Ok(()) => target_filename,
            Err(e) => {
                eprintln!("⚠️ Failed to save device image to {:?}: {}", dest_path, e);
                file_path.to_string_lossy().to_string()
            }
        };

        self.active_device_profile.image = saved_image_ref;

        let size = [decoded.width() as usize, decoded.height() as usize];
        let buffer = decoded.to_rgba8();
        let pixels = buffer.as_flat_samples();
        let color_image = egui::ColorImage::from_rgba_unmultiplied(size, pixels.as_slice());
        let texture_name = format!(
            "device_{}_{}",
            self.active_device_profile.id,
            std::time::SystemTime::now()
                .duration_since(std::time::UNIX_EPOCH)
                .unwrap_or_default()
                .as_millis()
        );
        self.texture = Some(ctx.load_texture(texture_name, color_image, Default::default()));

        // Save profile JSON to user directory
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
            if let Err(e) = std::fs::write(&dest_file, &json_str) {
                eprintln!("⚠️ Failed to save device profile to {:?}: {}", dest_file, e);
            }
        }

        self.device_registry = DeviceRegistry::new();
        self.notify_success(tr(&lang, "editor_image_updated"));
    }

    pub fn reset_device_image(&mut self, ctx: &egui::Context) {
        let lang = self.lang();
        let default_img = match self.active_device_profile.id.as_str() {
            "mx-master-2s" | "mx_master_2s" => "mx-master-2s.webp",
            "generic" => "device_model.webp",
            _ => "mx-master-4.webp",
        };
        self.active_device_profile.image = default_img.to_string();

        let user_images_dir = DeviceRegistry::user_devices_dir().join("images");
        for ext in &["png", "webp", "jpg", "jpeg"] {
            let p = user_images_dir.join(format!("{}.{}", self.active_device_profile.id, ext));
            if p.exists() {
                let _ = std::fs::remove_file(p);
            }
        }

        let p = LoadedDeviceProfile {
            config: self.active_device_profile.clone(),
            source: ProfileSource::BuiltIn,
            base_dir: None,
        };
        self.texture = self.device_registry.load_texture(ctx, &p);

        // Save profile JSON
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
            if let Err(e) = std::fs::write(&dest_file, &json_str) {
                eprintln!("⚠️ Failed to save device profile to {:?}: {}", dest_file, e);
            }
        }

        self.device_registry = DeviceRegistry::new();
        self.notify_success(tr(&lang, "editor_image_reset"));
    }

    pub fn lang(&self) -> String {
        self.config.general.language.clone()
    }

    pub fn notify_success(&mut self, msg: impl Into<String>) {
        let msg = msg.into();
        self.status_message = msg.clone();
        self.toast = Some((msg, std::time::Instant::now(), true));
    }

    pub fn notify_error(&mut self, msg: impl Into<String>) {
        let msg = msg.into();
        self.status_message = msg.clone();
        self.toast = Some((msg, std::time::Instant::now(), false));
    }

    pub fn active_ring_menu(&self) -> &RingMenuConfig {
        if self.active_profile_index == 0 {
            &self.config.ring_menu
        } else {
            let idx = self.active_profile_index - 1;
            if idx < self.app_profiles.len() {
                if let Some(ref menu) = self.app_profiles[idx].ring_menu {
                    return menu;
                }
            }
            &self.config.ring_menu
        }
    }

    pub fn active_ring_menu_mut(&mut self) -> &mut RingMenuConfig {
        if self.active_profile_index == 0 {
            &mut self.config.ring_menu
        } else {
            let idx = self.active_profile_index - 1;
            let fallback = self.config.ring_menu.clone();
            if idx < self.app_profiles.len() {
                let profile = &mut self.app_profiles[idx];
                if profile.ring_menu.is_none() {
                    profile.ring_menu = Some(fallback);
                }
                return profile.ring_menu.as_mut().unwrap();
            }
            &mut self.config.ring_menu
        }
    }

    pub fn ensure_active_profile_ring_menu(&mut self) {
        if self.active_profile_index > 0 {
            let idx = self.active_profile_index - 1;
            if idx < self.app_profiles.len() {
                if self.app_profiles[idx].ring_menu.is_none() {
                    self.app_profiles[idx].ring_menu = Some(self.config.ring_menu.clone());
                }
            }
        }
    }

    pub fn save_config(&mut self) -> Result<(), Box<dyn std::error::Error>> {
        self.config.app_profiles = self.app_profiles.clone();
        let res = self.config.save_to_file(crate::config::Config::config_path());
        if res.is_ok() {
            crate::platform::ipc::send_command_to_daemon(&crate::platform::ipc::DaemonIpcCommand::ReloadConfig);
        }
        res
    }

    pub fn trigger_async_mouse_scan(&mut self, ctx: egui::Context) {
        let (tx, rx) = mpsc::channel();
        self.scan_rx = Some(rx);

        if let (Some(event_tx), listened_paths) = (&self.event_tx, &self.listened_paths) {
            attach_new_mouse_listeners(ctx.clone(), event_tx.clone(), Arc::clone(listened_paths));
        }

        thread::spawn(move || {
            let info = scan_mouse_device();
            let _ = tx.send(info);
            ctx.request_repaint();
        });
    }
}
