use eframe::egui;
use super::types::ConfigApp;
use crate::platform::window::{run_window_detection_diagnostics, run_kde_diagnostics};
use crate::utils::i18n::tr;

pub fn render_tab_debug(app: &mut ConfigApp, ui: &mut egui::Ui) {
    let lang = app.config.general.language.clone();
    ui.add_space(10.0);

    ui.horizontal(|ui| {
        ui.heading(
            egui::RichText::new(tr(&lang, "debug_lab_title"))
                .strong()
                .size(18.0)
                .color(egui::Color32::from_rgb(0, 215, 175)),
        );
        ui.add_space(10.0);
        ui.label(
            egui::RichText::new(tr(&lang, "dev_mode_active_badge"))
                .small()
                .color(egui::Color32::from_gray(140)),
        );
    });

    ui.add_space(12.0);

    // Environment info card
    egui::Frame::none()
        .fill(egui::Color32::from_rgb(22, 26, 32))
        .stroke(egui::Stroke::new(1.0_f32, egui::Color32::from_rgb(38, 44, 54)))
        .rounding(egui::Rounding::same(8.0))
        .inner_margin(12.0)
        .show(ui, |ui| {
            ui.label(egui::RichText::new(tr(&lang, "graphical_env_header")).strong().color(egui::Color32::WHITE));
            ui.add_space(6.0);
            let desktop = std::env::var("XDG_CURRENT_DESKTOP").unwrap_or_else(|_| "Unknown".into());
            let session = std::env::var("XDG_SESSION_TYPE").unwrap_or_else(|_| "Unknown".into());
            let display = std::env::var("WAYLAND_DISPLAY").or_else(|_| std::env::var("DISPLAY")).unwrap_or_else(|_| "Unknown".into());

            ui.horizontal(|ui| {
                ui.label(format!("{} {}", tr(&lang, "desktop_label"), desktop));
                ui.add_space(15.0);
                ui.label(format!("{} {}", tr(&lang, "session_label"), session));
                ui.add_space(15.0);
                ui.label(format!("{} {}", tr(&lang, "server_label"), display));
            });
        });

    ui.add_space(15.0);

    // Test control buttons
    ui.horizontal(|ui| {
        let btn_all = egui::Button::new(
            egui::RichText::new(tr(&lang, "run_full_diagnostics_btn"))
                .strong()
                .color(egui::Color32::WHITE),
        )
        .fill(egui::Color32::from_rgb(0, 150, 120))
        .min_size(egui::vec2(160.0, 32.0));

        if ui.add(btn_all).clicked() {
            app.debug_log_output = run_window_detection_diagnostics();
            ui.ctx().request_repaint();
        }

        ui.add_space(10.0);

        let btn_kde = egui::Button::new(
            egui::RichText::new(tr(&lang, "test_kde_btn"))
                .strong()
                .color(egui::Color32::WHITE),
        )
        .fill(egui::Color32::from_rgb(40, 90, 160))
        .min_size(egui::vec2(150.0, 32.0));

        if ui.add(btn_kde).clicked() {
            let mut log = String::from("🔍 --- TARGETED KDE PLASMA KWIN TEST ---\n\n");
            log.push_str(&run_kde_diagnostics());
            app.debug_log_output = log;
            ui.ctx().request_repaint();
        }

        ui.add_space(10.0);

        let btn_nudge = egui::Button::new(
            egui::RichText::new(tr(&lang, "test_nudge_btn"))
                .strong()
                .color(egui::Color32::WHITE),
        )
        .fill(egui::Color32::from_rgb(140, 80, 40))
        .min_size(egui::vec2(140.0, 32.0));

        if ui.add(btn_nudge).clicked() {
            crate::platform::input_emitter::nudge_mouse();
            app.debug_log_output.push_str("\n🐭 Nudge impulse (1px) emitted via uinput!\n");
            ui.ctx().request_repaint();
        }


        ui.with_layout(egui::Layout::right_to_left(egui::Align::Center), |ui| {
            if !app.debug_log_output.is_empty() {
                if ui.button(tr(&lang, "copy_log_btn")).clicked() {
                    ui.output_mut(|o| o.copied_text = app.debug_log_output.clone());
                    app.status_message = tr(&lang, "log_copied_status").to_string();
                }
                if ui.button(tr(&lang, "clear_log_btn")).clicked() {
                    app.debug_log_output.clear();
                }
            }
        });
    });

    ui.add_space(12.0);

    // Log output container
    ui.label(egui::RichText::new(tr(&lang, "results_logs_header")).strong().color(egui::Color32::WHITE));
    ui.add_space(6.0);

    egui::Frame::none()
        .fill(egui::Color32::from_rgb(14, 16, 20))
        .stroke(egui::Stroke::new(1.0_f32, egui::Color32::from_rgb(32, 38, 48)))
        .rounding(egui::Rounding::same(6.0))
        .inner_margin(10.0)
        .show(ui, |ui| {
            egui::ScrollArea::vertical()
                .max_height(400.0)
                .auto_shrink([false; 2])
                .show(ui, |ui| {
                    if app.debug_log_output.is_empty() {
                        ui.label(
                            egui::RichText::new(tr(&lang, "debug_empty_prompt"))
                                .italics()
                                .color(egui::Color32::from_gray(120)),
                        );
                    } else {
                        ui.add(
                            egui::TextEdit::multiline(&mut app.debug_log_output.as_str())
                                .font(egui::TextStyle::Monospace)
                                .desired_width(f32::INFINITY)
                                .desired_rows(20)
                                .interactive(false),
                        );
                    }
                });
        });
}
