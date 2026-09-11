pub mod geometry;
pub mod render;
pub mod state;

#[allow(unused_imports)]
pub use geometry::get_sector_slot_index;
#[allow(unused_imports)]
pub use state::{execute_action, execute_action_direct, next_slot_index, RingMenuApp};
#[allow(unused_imports)]
pub use crate::platform::overlay::{
    build_ring_overlay_viewport, cleanup_stale_ring_pid, is_pid_boring, ring_pid_file_path,
    safe_kill, terminate_ring_process,
};

use crate::config::RingMenuConfig;

pub fn run_ring_menu_window(
    config: RingMenuConfig,
    app_profiles: Vec<crate::config::AppProfileConfig>,
    initial_app: Option<String>,
    lang: String,
) {
    cleanup_stale_ring_pid();
    let pid_path = ring_pid_file_path();
    let current_pid = std::process::id();
    if let Err(e) = std::fs::write(&pid_path, current_pid.to_string()) {
        eprintln!("⚠️ Failed to write Ring Menu PID to {:?}: {}", pid_path, e);
    }

    struct PidCleanupGuard(std::path::PathBuf);
    impl Drop for PidCleanupGuard {
        fn drop(&mut self) {
            let _ = std::fs::remove_file(&self.0);
        }
    }
    let _guard = PidCleanupGuard(pid_path);

    let builder = build_ring_overlay_viewport();

    let options = eframe::NativeOptions {
        viewport: builder,
        ..Default::default()
    };

    let _ = eframe::run_native(
        "Bo-Ring Actions Ring",
        options,
        Box::new(move |cc| {
            crate::icon_loader::setup_custom_fonts(&cc.egui_ctx);
            crate::keyboard::init_virtual_keyboard();
            Ok(Box::new(RingMenuApp::new(config, app_profiles, initial_app, lang)))
        }),
    );
}


#[cfg(test)]
mod tests {
    use super::*;
    use eframe::egui;

    #[test]
    fn test_next_slot_index() {
        let total = 8;
        assert_eq!(next_slot_index(None, 1, total), 1);
        assert_eq!(next_slot_index(Some(0), 1, total), 1);
        assert_eq!(next_slot_index(Some(7), 1, total), 0); // Wrap forward
        assert_eq!(next_slot_index(Some(0), -1, total), 7); // Wrap backward
        assert_eq!(next_slot_index(Some(4), -1, total), 3);
        assert_eq!(next_slot_index(None, 0, 0), 0);
    }

    #[test]
    fn test_get_sector_slot_index() {
        let center = egui::pos2(500.0, 500.0);
        let total = 8;

        // Inside deadzone (<= 35px)
        assert_eq!(get_sector_slot_index(egui::pos2(510.0, 510.0), center, total), None);

        // Directly Up (Top slot 0)
        assert_eq!(get_sector_slot_index(egui::pos2(500.0, 400.0), center, total), Some(0));

        // Directly Right (Right slot 2)
        assert_eq!(get_sector_slot_index(egui::pos2(600.0, 500.0), center, total), Some(2));

        // Directly Down (Bottom slot 4)
        assert_eq!(get_sector_slot_index(egui::pos2(500.0, 600.0), center, total), Some(4));

        // Directly Left (Left slot 6)
        assert_eq!(get_sector_slot_index(egui::pos2(400.0, 500.0), center, total), Some(6));
    }

    #[test]
    fn test_display_app_name_lang_localization() {
        let mut state = RingMenuApp::new(crate::config::RingMenuConfig::default(), vec![], None, "en".to_string());

        state.active_app_name = "Detecting...".to_string();
        assert_eq!(state.display_app_name_lang("fr"), "Détection...");
        assert_eq!(state.display_app_name_lang("en"), "Detecting...");
        assert_eq!(state.display_app_name_lang("es"), "Detectando...");

        state.active_app_name = "Détection...".to_string();
        assert_eq!(state.display_app_name_lang("en"), "Detecting...");

        state.active_app_name = "Global / Desktop".to_string();
        assert_eq!(state.display_app_name_lang("fr"), "Bureau");
        assert_eq!(state.display_app_name_lang("en"), "Desktop");
        assert_eq!(state.display_app_name_lang("es"), "Escritorio");

        state.active_app_name = "Global / Bureau".to_string();
        assert_eq!(state.display_app_name_lang("en"), "Desktop");
        assert_eq!(state.display_app_name_lang("es"), "Escritorio");

        state.active_app_name = "Escritorio".to_string();
        assert_eq!(state.display_app_name_lang("fr"), "Bureau");
    }
}
