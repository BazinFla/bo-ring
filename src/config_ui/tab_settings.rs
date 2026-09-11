use crate::autostart;
use crate::i18n::tr;
use eframe::egui;

use super::types::ConfigApp;

impl ConfigApp {
    /// Render Tab 3: Global Settings, Language, Autostart Systemd & Desktop Menu
    pub(super) fn render_global_settings_tab(&mut self, ctx: &egui::Context) {
        let lang = self.lang();

        egui::CentralPanel::default()
            .frame(egui::Frame::none().fill(egui::Color32::from_rgb(18, 18, 22)).inner_margin(20.0))
            .show(ctx, |ui| {
                egui::ScrollArea::vertical().show(ui, |ui| {
                    // Title & Description Header
                    ui.heading(
                        egui::RichText::new(tr(&lang, "global_settings_header"))
                            .strong()
                            .size(20.0)
                            .color(egui::Color32::WHITE)
                    );
                    ui.label(egui::RichText::new(tr(&lang, "global_settings_desc")).size(12.0).color(egui::Color32::from_gray(160)));
                    ui.add_space(14.0);

                    let card_frame = || {
                        egui::Frame::none()
                            .fill(egui::Color32::from_rgb(26, 28, 36))
                            .stroke(egui::Stroke::new(1.0_f32, egui::Color32::from_rgb(45, 48, 62)))
                            .rounding(egui::Rounding::same(12.0))
                            .inner_margin(egui::Margin::same(16.0))
                    };

                    // ==========================================
                    // CARD 1: HERO CARD (ABOUT BO-RING)
                    // ==========================================
                    egui::Frame::none()
                        .fill(egui::Color32::from_rgb(24, 26, 36))
                        .stroke(egui::Stroke::new(1.0_f32, egui::Color32::from_rgb(0, 215, 175)))
                        .rounding(egui::Rounding::same(12.0))
                        .inner_margin(egui::Margin::same(16.0))
                        .show(ui, |ui| {
                            ui.set_width(ui.available_width());
                            ui.horizontal(|ui| {
                                ui.vertical(|ui| {
                                    ui.horizontal(|ui| {
                                        ui.heading(
                                            egui::RichText::new("🎛️ Bo-Ring")
                                                .strong()
                                                .size(18.0)
                                                .color(egui::Color32::WHITE)
                                        );
                                        ui.add_space(6.0);

                                        let version_badge = egui::Button::new(
                                            egui::RichText::new(format!("v{}", env!("CARGO_PKG_VERSION")))
                                                .small()
                                                .strong()
                                                .color(egui::Color32::from_rgb(0, 215, 175))
                                        )
                                        .fill(egui::Color32::from_rgb(18, 40, 36))
                                        .stroke(egui::Stroke::new(1.0_f32, egui::Color32::from_rgb(0, 215, 175)))
                                        .rounding(egui::Rounding::same(10.0));
                                        ui.add(version_badge);

                                        let mit_badge = egui::Button::new(
                                            egui::RichText::new(tr(&lang, "license_label"))
                                                .small()
                                                .color(egui::Color32::from_gray(180))
                                        )
                                        .fill(egui::Color32::from_rgb(34, 38, 50))
                                        .stroke(egui::Stroke::new(1.0_f32, egui::Color32::from_rgb(60, 68, 88)))
                                        .rounding(egui::Rounding::same(10.0));
                                        ui.add(mit_badge);
                                    });
                                    ui.add_space(2.0);
                                    ui.label(
                                        egui::RichText::new(tr(&lang, "about_card_desc"))
                                            .size(12.0)
                                            .color(egui::Color32::from_gray(160))
                                    );
                                });

                                ui.with_layout(egui::Layout::right_to_left(egui::Align::Center), |ui| {
                                    let gh_btn = egui::Button::new(
                                        egui::RichText::new(format!("⭐ {}", tr(&lang, "github_repo_btn")))
                                            .strong()
                                            .color(egui::Color32::WHITE)
                                    )
                                    .fill(egui::Color32::from_rgb(38, 44, 58))
                                    .stroke(egui::Stroke::new(1.0_f32, egui::Color32::from_rgb(80, 95, 125)))
                                    .rounding(egui::Rounding::same(8.0))
                                    .min_size(egui::vec2(140.0, 30.0));

                                    if ui.add(gh_btn).on_hover_text(tr(&lang, "github_repo_tooltip")).clicked() {
                                        ctx.open_url(egui::OpenUrl::same_tab("https://github.com/BazinFla/bo-ring"));
                                    }
                                });
                            });
                        });

                    ui.add_space(14.0);

                    // ==========================================
                    // CARD 2: SERVICES & AUTOSTART INTEGRATION
                    // ==========================================
                    card_frame().show(ui, |ui| {
                        ui.set_width(ui.available_width());
                        ui.heading(
                            egui::RichText::new(format!("⚙️ {}", tr(&lang, "systemd_service_header")))
                                .strong()
                                .size(15.0)
                                .color(egui::Color32::WHITE)
                        );
                        ui.add_space(10.0);

                        // --- ITEM 1: SYSTEMD DAEMON ---
                        ui.horizontal(|ui| {
                            ui.vertical(|ui| {
                                ui.label(egui::RichText::new(tr(&lang, "systemd_daemon_title")).strong().color(egui::Color32::WHITE));
                                ui.label(egui::RichText::new(tr(&lang, "systemd_desc")).size(11.5).color(egui::Color32::from_gray(150)));
                            });

                            ui.with_layout(egui::Layout::right_to_left(egui::Align::Center), |ui| {
                                let is_active = autostart::is_systemd_active();
                                let is_enabled = autostart::is_systemd_enabled();
                                let is_installed = autostart::is_systemd_installed();

                                if is_active || is_enabled {
                                    // Single alternating DISABLE button
                                    let disable_btn = egui::Button::new(
                                        egui::RichText::new(format!("⏹️ {}", tr(&lang, "disable_systemd_btn")))
                                            .strong()
                                            .color(egui::Color32::from_rgb(255, 110, 110))
                                    )
                                    .fill(egui::Color32::from_rgb(45, 24, 28))
                                    .stroke(egui::Stroke::new(1.0_f32, egui::Color32::from_rgb(130, 48, 54)))
                                    .rounding(egui::Rounding::same(8.0))
                                    .min_size(egui::vec2(130.0, 30.0));

                                    if ui.add(disable_btn).clicked() {
                                        match autostart::disable_systemd_service() {
                                            Ok(_) => {
                                                self.config.general.autostart_daemon = false;
                                                self.status_message = tr(&lang, "save_success").to_string();
                                            }
                                            Err(e) => {
                                                self.status_message = format!("❌ Systemd Error: {}", e);
                                            }
                                        }
                                    }

                                    ui.add_space(8.0);
                                    if is_active {
                                        let status_pill = egui::Button::new(
                                            egui::RichText::new(format!("🟢 {}", tr(&lang, "daemon_active"))).small().strong().color(egui::Color32::from_rgb(0, 215, 175))
                                        )
                                        .fill(egui::Color32::from_rgb(20, 42, 38))
                                        .stroke(egui::Stroke::new(1.0_f32, egui::Color32::from_rgb(0, 215, 175)))
                                        .rounding(egui::Rounding::same(10.0));
                                        ui.add(status_pill);
                                    } else {
                                        let status_pill = egui::Button::new(
                                            egui::RichText::new(format!("🟡 {}", tr(&lang, "service_installed_inactive"))).small().color(egui::Color32::from_rgb(255, 190, 40))
                                        )
                                        .fill(egui::Color32::from_rgb(45, 38, 20))
                                        .stroke(egui::Stroke::new(1.0_f32, egui::Color32::from_rgb(200, 150, 30)))
                                        .rounding(egui::Rounding::same(10.0));
                                        ui.add(status_pill);
                                    }
                                } else {
                                    // Single alternating ENABLE button
                                    let enable_btn = egui::Button::new(
                                        egui::RichText::new(format!("⚡ {}", tr(&lang, "enable_systemd_btn")))
                                            .strong()
                                            .color(egui::Color32::WHITE)
                                    )
                                    .fill(egui::Color32::from_rgb(0, 160, 130))
                                    .stroke(egui::Stroke::new(1.0_f32, egui::Color32::from_rgb(0, 215, 175)))
                                    .rounding(egui::Rounding::same(8.0))
                                    .min_size(egui::vec2(130.0, 30.0));

                                    if ui.add(enable_btn).clicked() {
                                        match autostart::enable_systemd_service() {
                                            Ok(_) => {
                                                self.config.general.autostart_daemon = true;
                                                self.status_message = tr(&lang, "save_success").to_string();
                                            }
                                            Err(e) => {
                                                self.status_message = format!("❌ Systemd Error: {}", e);
                                            }
                                        }
                                    }

                                    ui.add_space(8.0);
                                    let not_inst_pill = if is_installed {
                                        egui::Button::new(
                                            egui::RichText::new(format!("⚪ {}", tr(&lang, "service_inactive"))).small().color(egui::Color32::from_gray(160))
                                        )
                                        .fill(egui::Color32::from_rgb(34, 34, 42))
                                        .stroke(egui::Stroke::new(1.0_f32, egui::Color32::from_gray(70)))
                                        .rounding(egui::Rounding::same(10.0))
                                    } else {
                                        egui::Button::new(
                                            egui::RichText::new(format!("⚪ {}", tr(&lang, "not_installed"))).small().color(egui::Color32::from_gray(160))
                                        )
                                        .fill(egui::Color32::from_rgb(34, 34, 42))
                                        .stroke(egui::Stroke::new(1.0_f32, egui::Color32::from_gray(70)))
                                        .rounding(egui::Rounding::same(10.0))
                                    };
                                    ui.add(not_inst_pill);
                                }
                            });
                        });

                        ui.add_space(8.0);
                        ui.separator();
                        ui.add_space(8.0);

                        // --- ITEM 2: XDG DESKTOP AUTOSTART ---
                        ui.horizontal(|ui| {
                            ui.vertical(|ui| {
                                ui.label(egui::RichText::new(tr(&lang, "xdg_autostart_header")).strong().color(egui::Color32::WHITE));
                                ui.label(egui::RichText::new(tr(&lang, "xdg_autostart_desc")).size(11.5).color(egui::Color32::from_gray(150)));
                            });

                            ui.with_layout(egui::Layout::right_to_left(egui::Align::Center), |ui| {
                                let is_autostart = autostart::is_xdg_autostart_installed();

                                if is_autostart {
                                    // Single DISABLE button
                                    let disable_btn = egui::Button::new(
                                        egui::RichText::new(format!("⏹️ {}", tr(&lang, "disable_xdg_btn")))
                                            .strong()
                                            .color(egui::Color32::from_rgb(255, 110, 110))
                                    )
                                    .fill(egui::Color32::from_rgb(45, 24, 28))
                                    .stroke(egui::Stroke::new(1.0_f32, egui::Color32::from_rgb(130, 48, 54)))
                                    .rounding(egui::Rounding::same(8.0))
                                    .min_size(egui::vec2(130.0, 30.0));

                                    if ui.add(disable_btn).clicked() {
                                        match autostart::disable_xdg_autostart() {
                                            Ok(_) => self.status_message = tr(&lang, "save_success").to_string(),
                                            Err(e) => self.status_message = format!("❌ XDG Autostart Error: {}", e),
                                        }
                                    }

                                    ui.add_space(8.0);
                                    let active_pill = egui::Button::new(
                                        egui::RichText::new(tr(&lang, "status_active")).small().strong().color(egui::Color32::from_rgb(0, 215, 175))
                                    )
                                    .fill(egui::Color32::from_rgb(20, 42, 38))
                                    .stroke(egui::Stroke::new(1.0_f32, egui::Color32::from_rgb(0, 215, 175)))
                                    .rounding(egui::Rounding::same(10.0));
                                    ui.add(active_pill);
                                } else {
                                    // Single ENABLE button
                                    let enable_btn = egui::Button::new(
                                        egui::RichText::new(format!("⚡ {}", tr(&lang, "enable_xdg_btn")))
                                            .strong()
                                            .color(egui::Color32::WHITE)
                                    )
                                    .fill(egui::Color32::from_rgb(0, 160, 130))
                                    .stroke(egui::Stroke::new(1.0_f32, egui::Color32::from_rgb(0, 215, 175)))
                                    .rounding(egui::Rounding::same(8.0))
                                    .min_size(egui::vec2(130.0, 30.0));

                                    if ui.add(enable_btn).clicked() {
                                        match autostart::enable_xdg_autostart() {
                                            Ok(_) => self.status_message = tr(&lang, "save_success").to_string(),
                                            Err(e) => self.status_message = format!("❌ XDG Autostart Error: {}", e),
                                        }
                                    }

                                    ui.add_space(8.0);
                                    let inactive_pill = egui::Button::new(
                                        egui::RichText::new(tr(&lang, "status_inactive")).small().color(egui::Color32::from_gray(160))
                                    )
                                    .fill(egui::Color32::from_rgb(34, 34, 42))
                                    .stroke(egui::Stroke::new(1.0_f32, egui::Color32::from_gray(70)))
                                    .rounding(egui::Rounding::same(10.0));
                                    ui.add(inactive_pill);
                                }
                            });
                        });

                        ui.add_space(8.0);
                        ui.separator();
                        ui.add_space(8.0);

                        // --- ITEM 3: APPLICATION LAUNCHER (.DESKTOP) ---
                        ui.horizontal(|ui| {
                            ui.vertical(|ui| {
                                ui.label(egui::RichText::new(tr(&lang, "desktop_launcher_header")).strong().color(egui::Color32::WHITE));
                                ui.label(egui::RichText::new(tr(&lang, "desktop_launcher_desc")).size(11.5).color(egui::Color32::from_gray(150)));
                            });

                            ui.with_layout(egui::Layout::right_to_left(egui::Align::Center), |ui| {
                                let is_system = autostart::is_gnome_menu_system();
                                let is_user = autostart::gnome_menu_path().exists();

                                if is_system {
                                    let sys_pill = egui::Button::new(
                                        egui::RichText::new(format!("🟢 {}", tr(&lang, "system_package_installed"))).small().strong().color(egui::Color32::from_rgb(0, 215, 175))
                                    )
                                    .fill(egui::Color32::from_rgb(20, 42, 38))
                                    .stroke(egui::Stroke::new(1.0_f32, egui::Color32::from_rgb(0, 215, 175)))
                                    .rounding(egui::Rounding::same(10.0));
                                    ui.add(sys_pill).on_hover_text(tr(&lang, "system_package_installed_tooltip"));
                                } else if is_user {
                                    // Single REMOVE button
                                    let remove_btn = egui::Button::new(
                                        egui::RichText::new(format!("🗑️ {}", tr(&lang, "remove_launcher_btn")))
                                            .strong()
                                            .color(egui::Color32::from_rgb(255, 110, 110))
                                    )
                                    .fill(egui::Color32::from_rgb(45, 24, 28))
                                    .stroke(egui::Stroke::new(1.0_f32, egui::Color32::from_rgb(130, 48, 54)))
                                    .rounding(egui::Rounding::same(8.0))
                                    .min_size(egui::vec2(130.0, 30.0));

                                    if ui.add(remove_btn).clicked() {
                                        match autostart::remove_gnome_menu() {
                                            Ok(_) => self.status_message = tr(&lang, "save_success").to_string(),
                                            Err(e) => self.status_message = format!("❌ Shortcut Error: {}", e),
                                        }
                                    }

                                    ui.add_space(8.0);
                                    let installed_pill = egui::Button::new(
                                        egui::RichText::new(tr(&lang, "status_installed")).small().strong().color(egui::Color32::from_rgb(0, 215, 175))
                                    )
                                    .fill(egui::Color32::from_rgb(20, 42, 38))
                                    .stroke(egui::Stroke::new(1.0_f32, egui::Color32::from_rgb(0, 215, 175)))
                                    .rounding(egui::Rounding::same(10.0));
                                    ui.add(installed_pill);
                                } else {
                                    // Single INSTALL button
                                    let install_btn = egui::Button::new(
                                        egui::RichText::new(format!("📌 {}", tr(&lang, "install_launcher_btn")))
                                            .strong()
                                            .color(egui::Color32::WHITE)
                                    )
                                    .fill(egui::Color32::from_rgb(0, 160, 130))
                                    .stroke(egui::Stroke::new(1.0_f32, egui::Color32::from_rgb(0, 215, 175)))
                                    .rounding(egui::Rounding::same(8.0))
                                    .min_size(egui::vec2(130.0, 30.0));

                                    if ui.add(install_btn).clicked() {
                                        match autostart::install_gnome_menu() {
                                            Ok(_) => self.status_message = tr(&lang, "save_success").to_string(),
                                            Err(e) => self.status_message = format!("❌ Shortcut Error: {}", e),
                                        }
                                    }

                                    ui.add_space(8.0);
                                    let not_inst_pill = egui::Button::new(
                                        egui::RichText::new(format!("⚪ {}", tr(&lang, "not_installed"))).small().color(egui::Color32::from_gray(160))
                                    )
                                    .fill(egui::Color32::from_rgb(34, 34, 42))
                                    .stroke(egui::Stroke::new(1.0_f32, egui::Color32::from_gray(70)))
                                    .rounding(egui::Rounding::same(10.0));
                                    ui.add(not_inst_pill);
                                }
                            });
                        });

                        ui.add_space(8.0);
                        ui.separator();
                        ui.add_space(8.0);

                        // --- ITEM 4: GNOME SHELL EXTENSION (HOVER & DESKTOP) ---
                        ui.horizontal(|ui| {
                            ui.vertical(|ui| {
                                ui.label(egui::RichText::new(tr(&lang, "gnome_extension_header")).strong().color(egui::Color32::WHITE));
                                ui.label(egui::RichText::new(tr(&lang, "gnome_extension_desc")).size(11.5).color(egui::Color32::from_gray(150)));
                            });

                            ui.with_layout(egui::Layout::right_to_left(egui::Align::Center), |ui| {
                                let (is_active, status_str) = crate::platform::window::check_gnome_extension_status_lang(&lang);

                                let install_btn = egui::Button::new(
                                    egui::RichText::new(format!("🧩 {}", tr(&lang, "gnome_extension_install_btn")))
                                        .strong()
                                        .color(egui::Color32::WHITE)
                                )
                                .fill(egui::Color32::from_rgb(0, 160, 130))
                                .stroke(egui::Stroke::new(1.0_f32, egui::Color32::from_rgb(0, 215, 175)))
                                .rounding(egui::Rounding::same(8.0))
                                .min_size(egui::vec2(150.0, 30.0));

                                if ui.add(install_btn).clicked() {
                                    match crate::platform::window::install_or_update_gnome_extension_lang(&lang) {
                                        Ok(msg) => self.status_message = msg,
                                        Err(e) => self.status_message = format!("❌ Extension Error: {}", e),
                                    }
                                }

                                ui.add_space(8.0);
                                if is_active {
                                    let status_pill = egui::Button::new(
                                        egui::RichText::new(format!("🟢 {status_str}")).small().strong().color(egui::Color32::from_rgb(0, 215, 175))
                                    )
                                    .fill(egui::Color32::from_rgb(20, 42, 38))
                                    .stroke(egui::Stroke::new(1.0_f32, egui::Color32::from_rgb(0, 215, 175)))
                                    .rounding(egui::Rounding::same(10.0));
                                    ui.add(status_pill);
                                } else {
                                    let status_pill = egui::Button::new(
                                        egui::RichText::new(format!("⚪ {status_str}")).small().color(egui::Color32::from_gray(160))
                                    )
                                    .fill(egui::Color32::from_rgb(34, 34, 42))
                                    .stroke(egui::Stroke::new(1.0_f32, egui::Color32::from_gray(70)))
                                    .rounding(egui::Rounding::same(10.0));
                                    ui.add(status_pill);
                                }
                            });
                        });
                    });

                    ui.add_space(14.0);


                    // ==========================================
                    // CARD 3: LANGUAGE & CONFIG MANAGEMENT
                    // ==========================================
                    card_frame().show(ui, |ui| {
                        ui.set_width(ui.available_width());
                        ui.heading(
                            egui::RichText::new(format!("🌐 {}", tr(&lang, "language_section_header")))
                                .strong()
                                .size(15.0)
                                .color(egui::Color32::WHITE)
                        );
                        ui.add_space(10.0);

                        // --- LANGUAGE SELECTION ---
                        ui.horizontal(|ui| {
                            ui.label(egui::RichText::new(tr(&lang, "select_language")).color(egui::Color32::from_gray(200)));
                            ui.add_space(12.0);

                            let system_lang_code = crate::i18n::detect_system_language();
                            let system_lang_name = crate::i18n::available_languages()
                                .iter()
                                .find(|l| l.code == system_lang_code)
                                .map(|l| l.display_name.as_str())
                                .unwrap_or("English");

                            let sys_label = format!("{} ({})", tr(&lang, "system_default"), system_lang_name);

                            let current_display = if self.config.general.language == "system" || self.config.general.language == "default" {
                                sys_label.clone()
                            } else {
                                crate::i18n::available_languages()
                                    .iter()
                                    .find(|l| l.code == self.config.general.language)
                                    .map(|l| l.display_name.clone())
                                    .unwrap_or_else(|| self.config.general.language.clone())
                            };

                            // Styled ComboBox for seamless dark UI design
                            let style_mut = ui.style_mut();
                            style_mut.visuals.widgets.inactive.bg_fill = egui::Color32::from_rgb(34, 38, 50);
                            style_mut.visuals.widgets.hovered.bg_fill = egui::Color32::from_rgb(46, 52, 68);
                            style_mut.visuals.widgets.active.bg_fill = egui::Color32::from_rgb(0, 180, 140);
                            style_mut.visuals.widgets.inactive.rounding = egui::Rounding::same(6.0);

                            egui::ComboBox::from_id_source("language_selector")
                                .selected_text(egui::RichText::new(current_display).color(egui::Color32::WHITE))
                                .show_ui(ui, |ui| {
                                    if ui.selectable_value(&mut self.config.general.language, "system".to_string(), sys_label).changed() {
                                        self.status_message = tr(&lang, "status_lang_system").replace("{}", system_lang_name);
                                    }

                                    ui.separator();

                                    for language_info in crate::i18n::available_languages() {
                                        if ui.selectable_value(&mut self.config.general.language, language_info.code.clone(), &language_info.display_name).changed() {
                                            self.status_message = tr(&lang, "status_lang_set").replace("{}", &language_info.display_name);
                                        }
                                    }
                                });
                        });

                        ui.add_space(10.0);
                        ui.separator();
                        ui.add_space(10.0);

                        // --- GLOBAL CONFIG IMPORT / EXPORT ---
                        ui.horizontal(|ui| {
                            ui.vertical(|ui| {
                                ui.label(egui::RichText::new(tr(&lang, "global_config_io_header")).strong().color(egui::Color32::WHITE));
                                ui.label(egui::RichText::new(tr(&lang, "global_config_io_desc")).size(11.5).color(egui::Color32::from_gray(150)));
                            });

                            ui.with_layout(egui::Layout::right_to_left(egui::Align::Center), |ui| {
                                let import_global_btn = egui::Button::new(
                                    egui::RichText::new(format!("📤 {}", tr(&lang, "import_global_config_btn")))
                                        .strong()
                                        .color(egui::Color32::WHITE)
                                )
                                .fill(egui::Color32::from_rgb(38, 72, 58))
                                .stroke(egui::Stroke::new(1.0_f32, egui::Color32::from_rgb(0, 180, 140)))
                                .rounding(egui::Rounding::same(8.0))
                                .min_size(egui::vec2(130.0, 30.0));

                                if ui.add(import_global_btn).on_hover_text(tr(&lang, "import_global_config_tooltip")).clicked() {
                                    if let Some(file_path) = rfd::FileDialog::new()
                                        .set_title(tr(&lang, "dialog_import_global"))
                                        .add_filter("Bo-Ring Global Config (*.boring-cfg.toml, *.boring-cfg.json)", &["toml", "json", "boring-cfg"])
                                        .pick_file()
                                    {
                                        if let Ok(content) = std::fs::read_to_string(&file_path) {
                                            let imported_opt: Option<crate::config::Config> = toml::from_str(&content)
                                                .ok()
                                                .or_else(|| serde_json::from_str(&content).ok());

                                            if let Some(imported) = imported_opt {
                                                self.config = imported;
                                                self.app_profiles = self.config.app_profiles.clone();
                                                let _ = self.save_config();
                                                self.status_message = tr(&lang, "status_config_imported").replace("{}", &file_path.display().to_string());
                                            } else {
                                                self.status_message = tr(&lang, "status_invalid_config_format").to_string();
                                            }
                                        }
                                    }
                                }

                                ui.add_space(8.0);
                                let export_global_btn = egui::Button::new(
                                    egui::RichText::new(format!("📥 {}", tr(&lang, "export_global_config_btn")))
                                        .strong()
                                        .color(egui::Color32::WHITE)
                                )
                                .fill(egui::Color32::from_rgb(42, 58, 85))
                                .stroke(egui::Stroke::new(1.0_f32, egui::Color32::from_rgb(70, 110, 160)))
                                .rounding(egui::Rounding::same(8.0))
                                .min_size(egui::vec2(130.0, 30.0));

                                if ui.add(export_global_btn).on_hover_text(tr(&lang, "export_global_config_tooltip")).clicked() {
                                    self.config.app_profiles = self.app_profiles.clone();
                                    if let Some(save_path) = rfd::FileDialog::new()
                                        .set_title(tr(&lang, "dialog_export_global"))
                                        .set_file_name("config.boring-cfg.toml")
                                        .add_filter("Bo-Ring Global Config (*.boring-cfg.toml, *.boring-cfg.json)", &["toml", "json", "boring-cfg"])
                                        .save_file()
                                    {
                                        let path_str = save_path.to_string_lossy().to_lowercase();
                                        let ser_result = if path_str.ends_with(".json") {
                                             serde_json::to_string_pretty(&self.config).map_err(|e| e.to_string())
                                        } else {
                                            toml::to_string_pretty(&self.config).map_err(|e| e.to_string())
                                        };

                                        match ser_result {
                                            Ok(data) => {
                                                if let Err(e) = std::fs::write(&save_path, data) {
                                                    self.status_message = tr(&lang, "status_export_error").replace("{}", &e.to_string());
                                                } else {
                                                    self.status_message = tr(&lang, "status_config_exported").replace("{}", &save_path.display().to_string());
                                                }
                                            }
                                            Err(e) => {
                                                self.status_message = tr(&lang, "status_export_error").replace("{}", &e.to_string());
                                            }
                                        }
                                    }
                                }
                            });
                        });
                    });

                    ui.add_space(14.0);

                    // ==========================================
                    // CARD 4: SYSTEM HEALTH & PERMISSIONS
                    // ==========================================
                    card_frame().show(ui, |ui| {
                        ui.set_width(ui.available_width());
                        ui.heading(
                            egui::RichText::new(format!("🩺 {}", tr(&lang, "health_diagnostics_header")))
                                .strong()
                                .size(15.0)
                                .color(egui::Color32::WHITE)
                        );
                        ui.add_space(10.0);

                        ui.horizontal(|ui| {
                            let uinput_ok = autostart::check_uinput_access();
                            ui.label(egui::RichText::new(tr(&lang, "uinput_permission_label")).color(egui::Color32::from_gray(200)));
                            ui.add_space(6.0);
                            if uinput_ok {
                                let ok_pill = egui::Button::new(
                                    egui::RichText::new(format!("🟢 {}", tr(&lang, "access_granted"))).small().strong().color(egui::Color32::from_rgb(0, 215, 175))
                                )
                                .fill(egui::Color32::from_rgb(20, 42, 38))
                                .stroke(egui::Stroke::new(1.0_f32, egui::Color32::from_rgb(0, 215, 175)))
                                .rounding(egui::Rounding::same(10.0));
                                ui.add(ok_pill);
                            } else {
                                let warn_pill = egui::Button::new(
                                    egui::RichText::new(format!("⚠️ {}", tr(&lang, "insufficient_rights"))).small().strong().color(egui::Color32::from_rgb(255, 190, 40))
                                )
                                .fill(egui::Color32::from_rgb(45, 38, 20))
                                .stroke(egui::Stroke::new(1.0_f32, egui::Color32::from_rgb(200, 150, 30)))
                                .rounding(egui::Rounding::same(10.0));
                                ui.add(warn_pill);
                            }
                        });

                        ui.add_space(6.0);
                        ui.horizontal(|ui| {
                            ui.label(egui::RichText::new(tr(&lang, "executable_binary")).color(egui::Color32::from_gray(200)));
                            ui.label(egui::RichText::new(autostart::get_exe_path().to_string_lossy().to_string()).monospace().size(11.0).color(egui::Color32::from_rgb(0, 215, 175)));
                        });

                        ui.add_space(6.0);
                        ui.horizontal(|ui| {
                            ui.label(egui::RichText::new(tr(&lang, "target_device")).color(egui::Color32::from_gray(200)));
                            ui.add(
                                egui::TextEdit::singleline(&mut self.config.general.device_name)
                                    .desired_width(180.0)
                            );
                        });

                        ui.add_space(8.0);
                        ui.separator();
                        ui.add_space(8.0);

                        if ui.checkbox(
                            &mut self.config.general.dev_mode,
                            egui::RichText::new(tr(&lang, "dev_mode_checkbox")).color(egui::Color32::from_gray(200))
                        ).changed() {
                            let _ = self.save_config();
                        }
                        ui.add_space(2.0);
                        ui.label(
                            egui::RichText::new(tr(&lang, "dev_mode_desc"))
                                .small()
                                .color(egui::Color32::from_gray(140))
                        );
                    });
                });
            });
    }

    pub(super) fn render_profile_modal(&mut self, ctx: &egui::Context) {
        if !self.show_profile_modal {
            return;
        }

        let lang = self.lang();
        let is_editing = self.editing_profile_index.is_some();
        let modal_title = if is_editing {
            format!("✏️ {}", tr(&lang, "profile_modal_edit_title"))
        } else {
            format!("✨ {}", tr(&lang, "profile_modal_create_title"))
        };

        // Check duplicate app_id across multiple semicolon-separated IDs
        let input_ids: Vec<String> = self.modal_app_id
            .split(';')
            .map(|s| s.trim().to_lowercase())
            .filter(|s| !s.is_empty())
            .collect();

        let mut duplicate_info: Option<(String, String)> = None; // (colliding_id, colliding_profile_label)

        for (idx, p) in self.app_profiles.iter().enumerate() {
            if let Some(edit_idx) = self.editing_profile_index {
                if edit_idx == idx {
                    continue;
                }
            }
            let p_ids: Vec<String> = p.app_id
                .split(';')
                .map(|s| s.trim().to_lowercase())
                .filter(|s| !s.is_empty())
                .collect();

            for id in &input_ids {
                if p_ids.contains(id) {
                    duplicate_info = Some((id.clone(), p.label.clone()));
                    break;
                }
            }
            if duplicate_info.is_some() {
                break;
            }
        }

        let is_duplicate = duplicate_info.is_some();

        let mut is_open = self.show_profile_modal;

        let frame = egui::Frame::none()
            .fill(egui::Color32::from_rgb(24, 24, 28))
            .stroke(egui::Stroke::new(1.5_f32, egui::Color32::from_rgb(0, 215, 175)))
            .rounding(egui::Rounding::same(12.0))
            .shadow(egui::epaint::Shadow { offset: egui::vec2(0.0, 4.0), blur: 12.0_f32, spread: 2.0_f32, color: egui::Color32::from_black_alpha(180) })
            .inner_margin(egui::Margin::same(20.0));

        egui::Window::new(modal_title)
            .open(&mut is_open)
            .collapsible(false)
            .resizable(false)
            .anchor(egui::Align2::CENTER_CENTER, egui::vec2(0.0, 0.0))
            .fixed_size(egui::vec2(520.0, 280.0))
            .frame(frame)
            .show(ctx, |ui| {
                ui.add_space(4.0);

                if self.confirming_delete {
                    ui.add_space(8.0);
                    egui::Frame::none()
                        .fill(egui::Color32::from_rgb(36, 22, 24))
                        .stroke(egui::Stroke::new(1.2_f32, egui::Color32::from_rgb(220, 60, 70)))
                        .rounding(egui::Rounding::same(8.0))
                        .inner_margin(egui::Margin::same(16.0))
                        .show(ui, |ui| {
                            ui.vertical_centered(|ui| {
                                ui.label(egui::RichText::new(tr(&lang, "profile_modal_delete_confirm_msg")).strong().color(egui::Color32::WHITE));
                                ui.add_space(4.0);
                                ui.label(
                                    egui::RichText::new(format!("\"{}\" ({})", self.modal_label, self.modal_app_id))
                                        .italics()
                                        .color(egui::Color32::from_gray(180))
                                );
                            });
                        });

                    ui.add_space(20.0);
                    ui.horizontal(|ui| {
                        if let Some(edit_idx) = self.editing_profile_index {
                            let confirm_btn = egui::Button::new(
                                egui::RichText::new(tr(&lang, "profile_modal_delete_yes"))
                                    .strong()
                                    .color(egui::Color32::WHITE)
                            )
                            .fill(egui::Color32::from_rgb(210, 40, 50))
                            .rounding(egui::Rounding::same(6.0))
                            .min_size(egui::vec2(130.0, 30.0));

                            if ui.add(confirm_btn).clicked() {
                                if edit_idx < self.app_profiles.len() {
                                    self.app_profiles.remove(edit_idx);
                                    self.active_profile_index = 0;
                                    let _ = self.save_config();
                                }
                                self.editing_profile_index = None;
                                self.modal_app_id.clear();
                                self.modal_label.clear();
                                self.modal_logo_path.clear();
                                self.confirming_delete = false;
                                self.show_profile_modal = false;
                                self.prevent_profile_click_frame = 2;
                            }
                        }

                        ui.with_layout(egui::Layout::right_to_left(egui::Align::Center), |ui| {
                            let cancel_btn = egui::Button::new(
                                egui::RichText::new(tr(&lang, "profile_modal_cancel"))
                                    .color(egui::Color32::from_gray(180))
                            )
                            .fill(egui::Color32::from_rgb(34, 34, 40))
                            .rounding(egui::Rounding::same(6.0))
                            .min_size(egui::vec2(90.0, 30.0));

                            if ui.add(cancel_btn).clicked() {
                                self.confirming_delete = false;
                            }
                        });
                    });
                } else {
                    egui::Grid::new("profile_modal_grid")
                        .num_columns(2)
                        .spacing(egui::vec2(14.0, 12.0))
                        .show(ui, |ui| {
                            ui.label(egui::RichText::new(tr(&lang, "profile_modal_app_id")).strong().color(egui::Color32::WHITE));
                            ui.vertical(|ui| {
                                ui.add(
                                    egui::TextEdit::singleline(&mut self.modal_app_id)
                                        .hint_text("ex: gimp, code, firefox")
                                        .desired_width(230.0)
                                );
                                if is_duplicate {
                                    ui.add_space(2.0);
                                    ui.label(egui::RichText::new(tr(&lang, "profile_modal_duplicate_error")).size(11.0).color(egui::Color32::from_rgb(255, 90, 90)));
                                }
                            });
                            ui.end_row();

                            ui.label(egui::RichText::new(tr(&lang, "profile_modal_label")).strong().color(egui::Color32::WHITE));
                            ui.add(
                                egui::TextEdit::singleline(&mut self.modal_label)
                                    .hint_text("ex: GIMP, VS Code, Firefox")
                                    .desired_width(230.0)
                            );
                            ui.end_row();

                            ui.label(egui::RichText::new(tr(&lang, "profile_modal_logo")).strong().color(egui::Color32::WHITE));
                            ui.horizontal(|ui| {
                                ui.add(
                                    egui::TextEdit::singleline(&mut self.modal_logo_path)
                                        .desired_width(185.0)
                                        .hint_text("/path/to/logo.png")
                                );
                                if ui.button("📁").on_hover_text(tr(&lang, "browse_folder_btn")).clicked() {
                                    let mut dialog = rfd::FileDialog::new()
                                        .add_filter("Images", &["png", "webp", "jpg", "jpeg", "svg"]);

                                    let default_dir = crate::utils::icon_loader::expand_path("assets/apps");
                                    if default_dir.is_dir() {
                                        dialog = dialog.set_directory(default_dir);
                                    }

                                    if let Some(path) = dialog.pick_file() {
                                        self.modal_logo_path = path.display().to_string();
                                    }
                                }
                            });
                            ui.end_row();
                        });

                    ui.add_space(20.0);
                    ui.horizontal(|ui| {
                        let can_save = !self.modal_app_id.trim().is_empty() && !self.modal_label.trim().is_empty() && !is_duplicate;
                        let save_btn = egui::Button::new(
                            egui::RichText::new(format!("💾 {}", tr(&lang, "profile_modal_save")))
                                .strong()
                                .color(if can_save { egui::Color32::BLACK } else { egui::Color32::from_gray(120) })
                        )
                        .fill(if can_save { egui::Color32::from_rgb(0, 215, 175) } else { egui::Color32::from_rgb(35, 45, 45) })
                        .rounding(egui::Rounding::same(6.0))
                        .min_size(egui::vec2(85.0, 30.0));

                        if ui.add_enabled(can_save, save_btn).clicked() {
                            let profile = crate::config::AppProfileConfig {
                                app_id: self.modal_app_id.trim().to_string(),
                                label: self.modal_label.trim().to_string(),
                                translations: if let Some(edit_idx) = self.editing_profile_index {
                                    self.app_profiles.get(edit_idx).map(|p| p.translations.clone()).unwrap_or_default()
                                } else {
                                    std::collections::HashMap::new()
                                },
                                logo_path: self.modal_logo_path.trim().to_string(),
                                ring_menu: if let Some(edit_idx) = self.editing_profile_index {
                                    self.app_profiles.get(edit_idx).and_then(|p| p.ring_menu.clone())
                                } else {
                                    None
                                },
                            };
                            if let Some(edit_idx) = self.editing_profile_index {
                                if edit_idx < self.app_profiles.len() {
                                    self.app_profiles[edit_idx] = profile;
                                }
                                self.show_profile_modal = false;
                            } else {
                                self.app_profiles.push(profile);
                                let new_idx = self.app_profiles.len() - 1;
                                self.active_profile_index = new_idx + 1;
                                self.editing_profile_index = Some(new_idx);
                                self.modal_app_id = self.app_profiles[new_idx].app_id.clone();
                                self.modal_label = self.app_profiles[new_idx].label.clone();
                                self.modal_logo_path = self.app_profiles[new_idx].logo_path.clone();
                                self.show_profile_modal = true;
                            }
                            let _ = self.save_config();
                        }

                        if let Some(edit_idx) = self.editing_profile_index {
                            // EDIT MODE: Export & Import Profile
                            ui.add_space(4.0);
                            let export_btn = egui::Button::new(
                                egui::RichText::new(format!("📥 {}", tr(&lang, "profile_modal_export")))
                                    .strong()
                                    .color(egui::Color32::WHITE)
                            )
                            .fill(egui::Color32::from_rgb(45, 60, 85))
                            .stroke(egui::Stroke::new(1.0_f32, egui::Color32::from_rgb(70, 110, 160)))
                            .rounding(egui::Rounding::same(6.0))
                            .min_size(egui::vec2(80.0, 30.0));

                            if ui.add(export_btn).on_hover_text(tr(&lang, "profile_export_tooltip")).clicked() {
                                if edit_idx < self.app_profiles.len() {
                                    let mut export_profile = self.app_profiles[edit_idx].clone();
                                    export_profile.app_id = self.modal_app_id.trim().to_string();
                                    export_profile.label = self.modal_label.trim().to_string();
                                    export_profile.logo_path = self.modal_logo_path.trim().to_string();

                                    let default_filename = format!("{}.boring-app.json", if export_profile.app_id.is_empty() { "app_profile" } else { &export_profile.app_id });
                                    if let Some(save_path) = rfd::FileDialog::new()
                                        .set_title(tr(&lang, "dialog_export_profile"))
                                        .set_file_name(&default_filename)
                                        .add_filter("Bo-Ring App Profile (*.boring-app.json, *.boring-app.toml)", &["json", "toml", "boring-app"])
                                        .save_file()
                                    {
                                        let path_str = save_path.to_string_lossy().to_lowercase();
                                        let ser_result = if path_str.ends_with(".toml") {
                                            toml::to_string_pretty(&export_profile).map_err(|e| e.to_string())
                                        } else {
                                            serde_json::to_string_pretty(&export_profile).map_err(|e| e.to_string())
                                        };

                                        match ser_result {
                                            Ok(data) => {
                                                if let Err(e) = std::fs::write(&save_path, data) {
                                                    self.status_message = tr(&lang, "status_export_error").replace("{}", &e.to_string());
                                                } else {
                                                    self.status_message = tr(&lang, "status_profile_exported").replace("{}", &save_path.display().to_string());
                                                }
                                            }
                                            Err(e) => {
                                                self.status_message = tr(&lang, "status_export_error").replace("{}", &e.to_string());
                                            }
                                        }
                                    }
                                }
                            }

                            ui.add_space(4.0);
                            let import_btn = egui::Button::new(
                                egui::RichText::new(format!("📤 {}", tr(&lang, "profile_modal_import")))
                                    .strong()
                                    .color(egui::Color32::WHITE)
                            )
                            .fill(egui::Color32::from_rgb(40, 75, 60))
                            .stroke(egui::Stroke::new(1.0_f32, egui::Color32::from_rgb(0, 180, 140)))
                            .rounding(egui::Rounding::same(6.0))
                            .min_size(egui::vec2(80.0, 30.0));

                            if ui.add(import_btn).on_hover_text(tr(&lang, "profile_import_replace_tooltip")).clicked() {
                                if let Some(file_path) = rfd::FileDialog::new()
                                    .set_title(tr(&lang, "dialog_import_profile"))
                                    .add_filter("Bo-Ring App Profile (*.boring-app.json, *.boring-app.toml)", &["json", "toml", "boring-app"])
                                    .pick_file()
                                {
                                    if let Ok(content) = std::fs::read_to_string(&file_path) {
                                        let imported_opt: Option<crate::config::AppProfileConfig> = serde_json::from_str(&content)
                                            .ok()
                                            .or_else(|| toml::from_str(&content).ok());

                                        if let Some(imported) = imported_opt {
                                            if edit_idx < self.app_profiles.len() {
                                                self.app_profiles[edit_idx] = imported;
                                                self.modal_app_id = self.app_profiles[edit_idx].app_id.clone();
                                                self.modal_label = self.app_profiles[edit_idx].label.clone();
                                                self.modal_logo_path = self.app_profiles[edit_idx].logo_path.clone();
                                                let _ = self.save_config();
                                                self.status_message = tr(&lang, "status_profile_imported").replace("{}", &file_path.display().to_string());
                                            }
                                        } else {
                                            self.status_message = tr(&lang, "status_invalid_profile_format").to_string();
                                        }
                                    }
                                }
                            }

                            ui.add_space(8.0);
                            let delete_btn = egui::Button::new(
                                egui::RichText::new(tr(&lang, "profile_modal_delete"))
                                    .strong()
                                    .color(egui::Color32::from_rgb(255, 110, 110))
                            )
                            .fill(egui::Color32::from_rgb(42, 22, 26))
                            .stroke(egui::Stroke::new(1.0_f32, egui::Color32::from_rgb(120, 45, 50)))
                            .rounding(egui::Rounding::same(6.0))
                            .min_size(egui::vec2(80.0, 30.0));

                            if ui.add(delete_btn).clicked() {
                                self.confirming_delete = true;
                            }
                        } else {
                            // CREATE MODE: Import Profile directly as new profile
                            ui.add_space(4.0);
                            let import_create_btn = egui::Button::new(
                                egui::RichText::new(format!("📤 {}", tr(&lang, "profile_modal_import")))
                                    .strong()
                                    .color(egui::Color32::WHITE)
                            )
                            .fill(egui::Color32::from_rgb(40, 75, 60))
                            .stroke(egui::Stroke::new(1.0_f32, egui::Color32::from_rgb(0, 180, 140)))
                            .rounding(egui::Rounding::same(6.0))
                            .min_size(egui::vec2(120.0, 30.0));

                            if ui.add(import_create_btn).on_hover_text(tr(&lang, "profile_import_new_tooltip")).clicked() {
                                if let Some(file_path) = rfd::FileDialog::new()
                                    .set_title(tr(&lang, "dialog_import_app_config"))
                                    .add_filter("Bo-Ring App Profile (*.boring-app.json, *.boring-app.toml)", &["json", "toml", "boring-app"])
                                    .pick_file()
                                {
                                    if let Ok(content) = std::fs::read_to_string(&file_path) {
                                        let imported_opt: Option<crate::config::AppProfileConfig> = serde_json::from_str(&content)
                                            .ok()
                                            .or_else(|| toml::from_str(&content).ok());

                                        if let Some(mut imported) = imported_opt {
                                            if imported.app_id.trim().is_empty() {
                                                imported.app_id = "custom_app".to_string();
                                            }
                                            let base_id = imported.app_id.clone();
                                            let mut counter = 1;
                                            while self.app_profiles.iter().any(|p| p.app_id.trim().eq_ignore_ascii_case(imported.app_id.trim())) {
                                                imported.app_id = format!("{}_{}", base_id, counter);
                                                counter += 1;
                                            }

                                            self.app_profiles.push(imported.clone());
                                            let new_idx = self.app_profiles.len() - 1;
                                            self.active_profile_index = new_idx + 1;
                                            self.editing_profile_index = Some(new_idx);
                                            self.modal_app_id = imported.app_id.clone();
                                            self.modal_label = imported.label.clone();
                                            self.modal_logo_path = imported.logo_path.clone();
                                            self.show_profile_modal = true;
                                            let _ = self.save_config();
                                            self.status_message = tr(&lang, "status_profile_imported").replace("{}", &file_path.display().to_string());
                                        } else {
                                            self.status_message = tr(&lang, "status_invalid_profile_format").to_string();
                                        }
                                    }
                                }
                            }
                        }

                        ui.with_layout(egui::Layout::right_to_left(egui::Align::Center), |ui| {
                            let cancel_btn = egui::Button::new(
                                egui::RichText::new(tr(&lang, "profile_modal_cancel"))
                                    .color(egui::Color32::from_gray(180))
                            )
                            .fill(egui::Color32::from_rgb(34, 34, 40))
                            .rounding(egui::Rounding::same(6.0))
                            .min_size(egui::vec2(75.0, 30.0));

                            if ui.add(cancel_btn).clicked() {
                                self.show_profile_modal = false;
                            }
                        });
                    });
                }
            });

        if !is_open || !self.show_profile_modal {
            self.prevent_profile_click_frame = 2;
            self.show_profile_modal = false;
        } else {
            self.show_profile_modal = true;
        }
    }
}
