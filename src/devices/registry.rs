use eframe::egui;
use std::fs;
use std::path::{Path, PathBuf};

use super::model::DeviceModelConfig;

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum ProfileSource {
    BuiltIn,
    System(PathBuf),
    User(PathBuf),
}

#[derive(Debug, Clone)]
pub struct LoadedDeviceProfile {
    pub config: DeviceModelConfig,
    pub source: ProfileSource,
    pub base_dir: Option<PathBuf>,
}

pub struct DeviceRegistry {
    pub profiles: Vec<LoadedDeviceProfile>,
}

impl Default for DeviceRegistry {
    fn default() -> Self {
        Self::new()
    }
}

impl DeviceRegistry {
    pub fn new() -> Self {
        let mut registry = Self {
            profiles: Vec::new(),
        };

        registry.load_builtin_profiles();
        registry.scan_system_profiles();
        registry.scan_user_profiles();

        registry
    }

    fn load_builtin_profiles(&mut self) {
        let mx4_json = include_str!("../../assets/devices/mx_master_4.json");
        if let Ok(config) = serde_json::from_str::<DeviceModelConfig>(mx4_json) {
            self.profiles.push(LoadedDeviceProfile {
                config,
                source: ProfileSource::BuiltIn,
                base_dir: None,
            });
        }

        let mx2s_json = include_str!("../../assets/devices/mx_master_2s.json");
        if let Ok(config) = serde_json::from_str::<DeviceModelConfig>(mx2s_json) {
            self.profiles.push(LoadedDeviceProfile {
                config,
                source: ProfileSource::BuiltIn,
                base_dir: None,
            });
        }

        let generic_json = include_str!("../../assets/devices/generic.json");
        if let Ok(config) = serde_json::from_str::<DeviceModelConfig>(generic_json) {
            self.profiles.push(LoadedDeviceProfile {
                config,
                source: ProfileSource::BuiltIn,
                base_dir: None,
            });
        }
    }

    pub fn user_devices_dir() -> PathBuf {
        let home = std::env::var("HOME").unwrap_or_else(|_| ".".to_string());
        PathBuf::from(home).join(".config").join("bo-ring").join("devices")
    }

    pub fn ensure_user_devices_dir() -> PathBuf {
        let dir = Self::user_devices_dir();
        if !dir.exists() {
            let _ = fs::create_dir_all(&dir);
        }
        dir
    }

    fn scan_directory(&mut self, root: &Path, is_user: bool) {
        if !root.exists() || !root.is_dir() {
            return;
        }

        if let Ok(entries) = fs::read_dir(root) {
            for entry in entries.flatten() {
                let path = entry.path();
                if path.is_dir() {
                    // Check if directory contains device.json or <dirname>.json
                    let candidate_json = path.join("device.json");
                    let dir_name = entry.file_name().to_string_lossy().to_string();
                    let fallback_json = path.join(format!("{}.json", dir_name));

                    let target_json = if candidate_json.exists() {
                        Some(candidate_json)
                    } else if fallback_json.exists() {
                        Some(fallback_json)
                    } else {
                        None
                    };

                    if let Some(json_path) = target_json {
                        if let Ok(content) = fs::read_to_string(&json_path) {
                            if let Ok(config) = serde_json::from_str::<DeviceModelConfig>(&content) {
                                let source = if is_user {
                                    ProfileSource::User(json_path)
                                } else {
                                    ProfileSource::System(json_path)
                                };
                                self.upsert_profile(LoadedDeviceProfile {
                                    config,
                                    source,
                                    base_dir: Some(path.clone()),
                                });
                            }
                        }
                    }
                } else if path.is_file() && path.extension().is_some_and(|ext| ext == "json") {
                    // Direct json file in devices directory
                    if let Ok(content) = fs::read_to_string(&path) {
                        if let Ok(config) = serde_json::from_str::<DeviceModelConfig>(&content) {
                            let source = if is_user {
                                ProfileSource::User(path.clone())
                            } else {
                                ProfileSource::System(path.clone())
                            };
                            let parent = path.parent().map(|p| p.to_path_buf());
                            self.upsert_profile(LoadedDeviceProfile {
                                config,
                                source,
                                base_dir: parent,
                            });
                        }
                    }
                }
            }
        }
    }

    fn scan_system_profiles(&mut self) {
        let home = std::env::var("HOME").unwrap_or_else(|_| ".".to_string());
        let xdg_data = std::env::var("XDG_DATA_HOME")
            .map(PathBuf::from)
            .unwrap_or_else(|_| PathBuf::from(&home).join(".local").join("share"));

        // Local development paths (cargo run in repo)
        self.scan_directory(Path::new("assets/devices"), false);
        self.scan_directory(Path::new("bo-ring/assets/devices"), false);

        self.scan_directory(&xdg_data.join("bo-ring").join("assets").join("devices"), false);
        self.scan_directory(&xdg_data.join("bo-ring").join("devices"), false);
        self.scan_directory(Path::new("/usr/local/share/bo-ring/devices"), false);
        self.scan_directory(Path::new("/usr/share/bo-ring/devices"), false);
    }

    fn scan_user_profiles(&mut self) {
        let user_dir = Self::user_devices_dir();
        self.scan_directory(&user_dir, true);
    }

    fn upsert_profile(&mut self, mut profile: LoadedDeviceProfile) {
        // If a profile with the same id exists, user profile overrides system, system overrides builtin
        if let Some(idx) = self.profiles.iter().position(|p| p.config.id == profile.config.id) {
            if profile.config.match_ids.is_empty() && !self.profiles[idx].config.match_ids.is_empty() {
                profile.config.match_ids = self.profiles[idx].config.match_ids.clone();
            }
            if profile.config.match_names.is_empty() && !self.profiles[idx].config.match_names.is_empty() {
                profile.config.match_names = self.profiles[idx].config.match_names.clone();
            }
            self.profiles[idx] = profile;
        } else {
            self.profiles.push(profile);
        }
    }

    pub fn list_profiles(&self) -> &[LoadedDeviceProfile] {
        &self.profiles
    }

    pub fn get_profile(&self, id: &str) -> Option<&LoadedDeviceProfile> {
        self.profiles.iter().find(|p| p.config.id == id)
    }

    pub fn get_default_profile(&self) -> &LoadedDeviceProfile {
        self.get_profile("mx_master_4")
            .unwrap_or_else(|| &self.profiles[0])
    }

    pub fn save_profile(&self, config: &DeviceModelConfig) -> Result<PathBuf, std::io::Error> {
        let dest_dir = Self::ensure_user_devices_dir();
        let dest_file = dest_dir.join(format!("{}.json", config.id));
        let json_str = serde_json::to_string_pretty(config)
            .map_err(|e| std::io::Error::new(std::io::ErrorKind::InvalidData, e))?;
        fs::write(&dest_file, json_str)?;
        Ok(dest_file)
    }

    pub fn match_device(&self, dev_name: &str, vid_pid: Option<&str>) -> Option<&LoadedDeviceProfile> {
        // 1. First priority: Exact match on hardware VID:PID (match_ids)
        if let Some(vp) = vid_pid {
            let clean_vp = vp.trim().to_lowercase();
            if !clean_vp.is_empty() {
                let mut best_hw_match: Option<(&LoadedDeviceProfile, bool)> = None;
                for p in &self.profiles {
                    let is_user = matches!(p.source, ProfileSource::User(_));
                    if p.config.matches_hardware_id(&clean_vp) {
                        match best_hw_match {
                            None => best_hw_match = Some((p, is_user)),
                            Some((_, false)) if is_user => best_hw_match = Some((p, is_user)),
                            _ => {}
                        }
                    }
                }
                if let Some((p, _)) = best_hw_match {
                    return Some(p);
                }
            }
        }

        // 2. Second priority: Fallback to textual match_names
        self.match_device_name(dev_name)
    }

    pub fn match_device_name(&self, dev_name: &str) -> Option<&LoadedDeviceProfile> {
        let name_lower = dev_name.to_lowercase();
        let mut best_match: Option<(&LoadedDeviceProfile, usize, bool)> = None;

        for p in &self.profiles {
            let is_user = matches!(p.source, ProfileSource::User(_));
            for pattern in &p.config.match_names {
                let pat_lower = pattern.to_lowercase();
                if name_lower.contains(&pat_lower) {
                    let score = pat_lower.len();
                    let better = match best_match {
                        None => true,
                        Some((_, best_score, best_is_user)) => {
                            score > best_score || (score == best_score && is_user && !best_is_user)
                        }
                    };
                    if better {
                        best_match = Some((p, score, is_user));
                    }
                }
            }
        }

        best_match.map(|(p, _, _)| p)
    }

    pub fn load_texture(
        &self,
        ctx: &egui::Context,
        profile: &LoadedDeviceProfile,
    ) -> Option<egui::TextureHandle> {
        let image_ref = &profile.config.image;

        // 1. Built-in textures embedded directly in binary
        if image_ref == "mx-master-4.webp" || image_ref == "device_model.webp" {
            let img_bytes = include_bytes!("../../assets/devices/images/mx-master-4.webp");
            if let Ok(image) = image::load_from_memory(img_bytes) {
                let size = [image.width() as usize, image.height() as usize];
                let image_buffer = image.to_rgba8();
                let pixels = image_buffer.as_flat_samples();
                let color_image = egui::ColorImage::from_rgba_unmultiplied(size, pixels.as_slice());
                return Some(ctx.load_texture(
                    format!("device_{}", profile.config.id),
                    color_image,
                    Default::default(),
                ));
            }
        } else if image_ref == "mx-master-2s.webp" {
            let img_bytes = include_bytes!("../../assets/devices/images/mx-master-2s.webp");
            if let Ok(image) = image::load_from_memory(img_bytes) {
                let size = [image.width() as usize, image.height() as usize];
                let image_buffer = image.to_rgba8();
                let pixels = image_buffer.as_flat_samples();
                let color_image = egui::ColorImage::from_rgba_unmultiplied(size, pixels.as_slice());
                return Some(ctx.load_texture(
                    format!("device_{}", profile.config.id),
                    color_image,
                    Default::default(),
                ));
            }
        }

        // 2. Search in candidate directories (including user, XDG data, system, and local paths)
        let mut candidates = Vec::new();

        if let Some(ref base) = profile.base_dir {
            candidates.push(base.join("images").join(image_ref));
            candidates.push(base.join(image_ref));
        }

        candidates.push(PathBuf::from(image_ref));

        let home = std::env::var("HOME").unwrap_or_else(|_| ".".to_string());
        let xdg_data = std::env::var("XDG_DATA_HOME")
            .map(PathBuf::from)
            .unwrap_or_else(|_| PathBuf::from(&home).join(".local").join("share"));

        // Installed XDG data paths (~/.local/share/bo-ring/...)
        candidates.push(xdg_data.join("bo-ring").join("assets").join("devices").join("images").join(image_ref));
        candidates.push(xdg_data.join("bo-ring").join("assets").join("devices").join(image_ref));
        candidates.push(xdg_data.join("bo-ring").join("assets").join("images").join(image_ref));
        candidates.push(xdg_data.join("bo-ring").join("assets").join(image_ref));
        candidates.push(xdg_data.join("bo-ring").join("devices").join("images").join(image_ref));
        candidates.push(xdg_data.join("bo-ring").join("devices").join(image_ref));

        // User custom config directory (~/.config/bo-ring/devices/...)
        let user_devices_dir = Self::user_devices_dir();
        candidates.push(user_devices_dir.join("images").join(image_ref));
        candidates.push(user_devices_dir.join(image_ref));

        // Relative development paths
        candidates.push(PathBuf::from("assets/devices/images").join(image_ref));
        candidates.push(PathBuf::from("assets/devices").join(image_ref));
        candidates.push(PathBuf::from("assets").join(image_ref));
        candidates.push(PathBuf::from("bo-ring/assets/devices/images").join(image_ref));
        candidates.push(PathBuf::from("bo-ring/assets/devices").join(image_ref));
        candidates.push(PathBuf::from("bo-ring/assets").join(image_ref));

        // System directories (/usr/local/share and /usr/share)
        candidates.push(PathBuf::from("/usr/local/share/bo-ring/assets/devices/images").join(image_ref));
        candidates.push(PathBuf::from("/usr/local/share/bo-ring/devices/images").join(image_ref));
        candidates.push(PathBuf::from("/usr/local/share/bo-ring/devices").join(image_ref));
        candidates.push(PathBuf::from("/usr/share/bo-ring/assets/devices/images").join(image_ref));
        candidates.push(PathBuf::from("/usr/share/bo-ring/devices/images").join(image_ref));
        candidates.push(PathBuf::from("/usr/share/bo-ring/devices").join(image_ref));

        for path in candidates {
            if path.exists() {
                if let Ok(img_bytes) = fs::read(&path) {
                    if let Ok(image) = image::load_from_memory(&img_bytes) {
                        let size = [image.width() as usize, image.height() as usize];
                        let image_buffer = image.to_rgba8();
                        let pixels = image_buffer.as_flat_samples();
                        let color_image =
                            egui::ColorImage::from_rgba_unmultiplied(size, pixels.as_slice());
                        return Some(ctx.load_texture(
                            format!("device_{}", profile.config.id),
                            color_image,
                            Default::default(),
                        ));
                    }
                }
            }
        }

        // Fallback to built-in mx-master-4.webp
        let img_bytes = include_bytes!("../../assets/devices/images/mx-master-4.webp");
        if let Ok(image) = image::load_from_memory(img_bytes) {
            let size = [image.width() as usize, image.height() as usize];
            let image_buffer = image.to_rgba8();
            let pixels = image_buffer.as_flat_samples();
            let color_image = egui::ColorImage::from_rgba_unmultiplied(size, pixels.as_slice());
            Some(ctx.load_texture("device_fallback", color_image, Default::default()))
        } else {
            None
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_device_registry_builtins() {
        let registry = DeviceRegistry::new();
        assert!(!registry.profiles.is_empty());

        let mx4 = registry.get_profile("mx_master_4");
        assert!(mx4.is_some());
        assert_eq!(mx4.unwrap().config.buttons.len(), 6);

        let mx2s = registry.get_profile("mx_master_2s");
        assert!(mx2s.is_some());
        assert_eq!(mx2s.unwrap().config.buttons.len(), 5);
        assert_eq!(mx2s.unwrap().config.image, "mx-master-2s.webp");

        let matched_mx2s = registry.match_device_name("Logitech MX Master 2s");
        assert!(matched_mx2s.is_some());
        assert_eq!(matched_mx2s.unwrap().config.id, "mx_master_2s");

        let generic = registry.get_profile("generic_mouse");
        assert!(generic.is_some());

        let matched = registry.match_device_name("Logitech Wireless Mouse MX Master 4");
        assert!(matched.is_some());
        assert_eq!(matched.unwrap().config.id, "mx_master_4");

        let matched_generic = registry.match_device_name("HP USB Optical Mouse");
        assert!(matched_generic.is_some());
        assert_eq!(matched_generic.unwrap().config.id, "generic_mouse");

        let matched_none = registry.match_device_name("Virtual Keyboard");
        assert!(matched_none.is_none());

        let ctx = egui::Context::default();
        let tex_mx4 = registry.load_texture(&ctx, mx4.unwrap());
        assert!(tex_mx4.is_some());
        let tex_mx2s = registry.load_texture(&ctx, mx2s.unwrap());
        assert!(tex_mx2s.is_some());
    }

    #[test]
    fn test_custom_profile_scan() {
        let temp_dir = std::env::temp_dir().join(format!("boring_test_{}", std::process::id()));
        let model_dir = temp_dir.join("test_g502");
        let _ = fs::create_dir_all(&model_dir);

        let custom_json = r#"{
            "id": "test_g502",
            "name": "Logitech G502 Test",
            "author": "Community Tester",
            "match_names": ["G502 Test Mouse"],
            "image": "image.png",
            "buttons": [
                {
                    "code": 274,
                    "name_key": "btn_middle_wheel",
                    "default_name": "Middle Click",
                    "icon": "🖱️",
                    "side": "right",
                    "anchor": [0.5, 0.3]
                }
            ]
        }"#;

        fs::write(model_dir.join("device.json"), custom_json).expect("Write temp device.json");

        let mut registry = DeviceRegistry::new();
        registry.scan_directory(&temp_dir, true);

        let custom = registry.get_profile("test_g502");
        assert!(custom.is_some());
        assert_eq!(custom.unwrap().config.name, "Logitech G502 Test");
        assert_eq!(custom.unwrap().config.buttons.len(), 1);

        // Write a mock image file to test dynamic texture loading from disk without hardcoding
        let mock_img = image::RgbaImage::new(10, 10);
        let _ = mock_img.save(model_dir.join("image.png"));

        let ctx = egui::Context::default();
        let loaded_tex = registry.load_texture(&ctx, custom.unwrap());
        assert!(loaded_tex.is_some(), "Dynamic texture should be loaded from disk");

        let _ = fs::remove_dir_all(&temp_dir);
    }

    #[test]
    fn test_hardware_id_matching() {
        let registry = DeviceRegistry::new();

        // 1. Hardware VID:PID priority match: "046d:c548" matches mx_master_4 directly, even if name is generic
        let match_by_hw = registry.match_device("Generic Unknown Mouse", Some("046d:c548"));
        assert!(match_by_hw.is_some());
        assert_eq!(match_by_hw.unwrap().config.id, "mx_master_4");

        // "046d:4069" matches mx_master_2s
        let match_2s_hw = registry.match_device("Unknown Device", Some("046d:4069"));
        assert!(match_2s_hw.is_some());
        assert_eq!(match_2s_hw.unwrap().config.id, "mx_master_2s");

        // Case insensitivity and whitespace tolerance
        let match_case = registry.match_device("Unknown Device", Some("  046D:C548  "));
        assert!(match_case.is_some());
        assert_eq!(match_case.unwrap().config.id, "mx_master_4");

        // 2. Fallback to name matching when VID:PID does not match any profile
        let match_fallback = registry.match_device("Logitech MX Master 2s", Some("9999:9999"));
        assert!(match_fallback.is_some());
        assert_eq!(match_fallback.unwrap().config.id, "mx_master_2s");

        let match_fallback_none = registry.match_device("Logitech MX Master 4", None);
        assert!(match_fallback_none.is_some());
        assert_eq!(match_fallback_none.unwrap().config.id, "mx_master_4");
    }
}
