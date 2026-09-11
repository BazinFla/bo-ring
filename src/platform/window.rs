use std::process::Command;
pub use super::gnome::{
    check_gnome_extension_status, check_gnome_extension_status_lang,
    install_or_update_gnome_extension, install_or_update_gnome_extension_lang,
};

/// Main entry point for detecting the currently active application/window across Linux compositors.
/// Delegates detection to isolated desktop platform submodules (gnome, kde, hyprland, sway, x11).
pub fn detect_active_window() -> String {
    // 1. GNOME Shell DBus Extension (Primary strategy for GNOME 40+)
    if let Some(app) = detect_gnome_window() {
        return app;
    }

    // 2. KDE Plasma kdotool / xprop (Isolated strategy for KDE 5/6)
    if let Some(app) = detect_kde_window() {
        return app;
    }

    // 3. Hyprland (Wayland IPC)
    if let Some(app) = detect_hyprland_window() {
        return app;
    }

    // 4. Sway / wlroots (Wayland IPC)
    if let Some(app) = detect_sway_window() {
        return app;
    }

    // 5. Generic X11 / XWayland (xprop fallback)
    if let Some(app) = detect_x11_window() {
        return app;
    }

    "Global / Desktop".to_string()
}

pub fn focus_window_under_cursor() -> bool {
    if super::gnome::focus_gnome_window_under_cursor() {
        return true;
    }
    false
}

/// Strategy 1: GNOME Shell Extension DBus (Delegated to gnome.rs)
pub fn detect_gnome_window() -> Option<String> {
    super::gnome::detect_gnome_window()
}

/// Strategy 2: KDE Plasma kdotool & xprop (Delegated to kde.rs)
pub fn detect_kde_window() -> Option<String> {
    super::kde::detect_kde_window()
}

/// Strategy 3: Hyprland IPC (Delegated to hyprland.rs)
pub fn detect_hyprland_window() -> Option<String> {
    super::hyprland::detect_hyprland_window()
}

/// Strategy 4: Sway IPC (Delegated to sway.rs)
pub fn detect_sway_window() -> Option<String> {
    super::sway::detect_sway_window()
}

/// Strategy 5: Generic X11 xprop (Delegated to x11.rs)
pub fn detect_x11_window() -> Option<String> {
    super::x11::detect_x11_window()
}

/// Runs diagnostic checks for ALL detection strategies and formats a detailed report.
pub fn run_window_detection_diagnostics() -> String {
    let mut log = String::new();
    log.push_str("🔍 --- COMPLETE WINDOW DETECTION DIAGNOSTICS ---\n\n");

    log.push_str(&format!("• XDG_CURRENT_DESKTOP: {}\n", std::env::var("XDG_CURRENT_DESKTOP").unwrap_or_else(|_| "undefined".into())));
    log.push_str(&format!("• XDG_SESSION_TYPE: {}\n", std::env::var("XDG_SESSION_TYPE").unwrap_or_else(|_| "undefined".into())));
    log.push_str(&format!("• WAYLAND_DISPLAY: {}\n", std::env::var("WAYLAND_DISPLAY").unwrap_or_else(|_| "undefined".into())));
    log.push_str(&format!("• DISPLAY: {}\n\n", std::env::var("DISPLAY").unwrap_or_else(|_| "undefined".into())));

    // 1. GNOME
    log.push_str("--- [STRATEGY 1 : GNOME SHELL DBUS EXTENSION] ---\n");
    for method in ["GetWindowUnderCursor", "GetActiveWindow"] {
        log.push_str(&format!("▶ Command: gdbus call --session --dest org.gnome.Shell --object-path /org/boring/WindowTracker --method org.gnome.Shell.{method}\n"));
        match Command::new("gdbus")
            .args([
                "call",
                "--session",
                "--dest",
                "org.gnome.Shell",
                "--object-path",
                "/org/boring/WindowTracker",
                "--method",
                &format!("org.boring.WindowTracker.{method}"),
            ])
            .output()
        {
            Ok(out) => {
                log.push_str(&format!("   Exit Status: {}\n", out.status));
                log.push_str(&format!("   Stdout: {}\n", String::from_utf8_lossy(&out.stdout).trim()));
                log.push_str(&format!("   Stderr: {}\n", String::from_utf8_lossy(&out.stderr).trim()));
            }
            Err(e) => log.push_str(&format!("   Error: {e}\n")),
        }
    }
    log.push_str(&format!("GNOME Result: {:?}\n\n", detect_gnome_window()));

    // 2. KDE
    log.push_str("--- [STRATEGY 2 : KDE PLASMA KWIN] ---\n");
    log.push_str(&run_kde_diagnostics());
    log.push_str(&format!("KDE Result: {:?}\n\n", detect_kde_window()));

    // 3. Hyprland
    log.push_str("--- [STRATEGY 3 : HYPRLAND (hyprctl)] ---\n");
    match Command::new("hyprctl").args(["activewindow", "-j"]).output() {
        Ok(out) => {
            log.push_str(&format!("   Exit Status: {}\n", out.status));
            log.push_str(&format!("   Stdout: {}\n", String::from_utf8_lossy(&out.stdout).trim()));
        }
        Err(e) => log.push_str(&format!("   Error: {e}\n")),
    }
    log.push_str(&format!("Hyprland Result: {:?}\n\n", detect_hyprland_window()));

    // 4. Sway
    log.push_str("--- [STRATEGY 4 : SWAY (swaymsg)] ---\n");
    match Command::new("swaymsg").args(["-t", "get_tree"]).output() {
        Ok(out) => {
            log.push_str(&format!("   Exit Status: {}\n", out.status));
            log.push_str(&format!("   Stdout length: {} bytes\n", out.stdout.len()));
        }
        Err(e) => log.push_str(&format!("   Error: {e}\n")),
    }
    log.push_str(&format!("Sway Result: {:?}\n\n", detect_sway_window()));

    // 5. X11
    log.push_str("--- [STRATEGY 5 : X11 / XWAYLAND (xprop)] ---\n");
    match Command::new("xprop").args(["-root", "_NET_ACTIVE_WINDOW"]).output() {
        Ok(out) => {
            log.push_str(&format!("   Exit Status: {}\n", out.status));
            log.push_str(&format!("   Stdout: {}\n", String::from_utf8_lossy(&out.stdout).trim()));
        }
        Err(e) => log.push_str(&format!("   Error: {e}\n")),
    }
    log.push_str(&format!("X11 Result: {:?}\n\n", detect_x11_window()));

    log.push_str(&format!("🎯 GLOBAL DETECTED RESULT: '{}'\n", detect_active_window()));
    log
}

/// Runs detailed raw diagnostics specifically for KDE Plasma KWin.
pub fn run_kde_diagnostics() -> String {
    super::kde::run_kde_diagnostics()
}
