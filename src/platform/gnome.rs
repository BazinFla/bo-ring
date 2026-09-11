use std::process::Command;
use std::path::PathBuf;

/// Strategy 1: GNOME Shell Extension DBus (org.boring.WindowTracker)
/// Completely isolated to protect GNOME stability.
pub fn detect_gnome_window() -> Option<String> {
    // 1. Primary strategy: Inspect window directly under the mouse cursor
    if let Some(app) = call_tracker_method("GetWindowUnderCursor") {
        return Some(app);
    }

    // 2. Fallback strategy: If cursor query failed on DBus, fallback to active focused window
    if let Some(app) = call_tracker_method("GetActiveWindow") {
        return Some(app);
    }

    None
}

fn call_tracker_method(method: &str) -> Option<String> {
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
        Ok(output) => {
            if !output.status.success() {
                let stderr = String::from_utf8_lossy(&output.stderr);
                eprintln!("⚠️ detect_gnome_window [{}]: gdbus error (exit {:?}): {}", method, output.status.code(), stderr.trim());
                return None;
            }
            let stdout = String::from_utf8_lossy(&output.stdout);
            if !stdout.is_empty() && stdout.contains('(') {
                let clean = stdout
                    .trim()
                    .trim_matches(|c| c == '(' || c == ')' || c == '\'' || c == ' ');
                let parts: Vec<&str> = clean.split(',').collect();
                if !parts.is_empty() {
                    let app = parts[0].trim().trim_matches('\'');
                    if !app.is_empty()
                        && !app.eq_ignore_ascii_case("bo-ring")
                        && !app.eq_ignore_ascii_case("bo-ring-overlay")
                    {
                        let desktop_candidate = app.strip_prefix("Global / ").unwrap_or(app).trim();
                        if desktop_candidate.eq_ignore_ascii_case("desktop")
                            || crate::utils::i18n::matches_key_any_locale("desktop", desktop_candidate)
                            || app.eq_ignore_ascii_case("gnome-shell")
                            || app.eq_ignore_ascii_case("ding")
                        {
                            println!("✅ detect_gnome_window [{}]: Desktop identified", method);
                            return Some("Global / Desktop".to_string());
                        }
                        println!("✅ detect_gnome_window [{}]: found app '{}'", method, app);
                        return Some(app.to_string());
                    }
                }
            }
            None
        }
        Err(e) => {
            eprintln!("⚠️ detect_gnome_window [{}]: failed to spawn gdbus: {}", method, e);
            None
        }
    }
}



pub fn focus_gnome_window_under_cursor() -> bool {
    if let Ok(output) = Command::new("gdbus")
        .args([
            "call",
            "--session",
            "--dest",
            "org.gnome.Shell",
            "--object-path",
            "/org/boring/WindowTracker",
            "--method",
            "org.boring.WindowTracker.FocusWindowUnderCursor",
        ])
        .output()
    {
        if output.status.success() {
            let str_out = String::from_utf8_lossy(&output.stdout);
            if str_out.contains("true") {
                return true;
            }
        }
    }
    false
}

/// Checks if the GNOME Shell extension DBus service is active or installed (with language localization).
pub fn check_gnome_extension_status_lang(lang: &str) -> (bool, String) {
    if let Ok(output) = Command::new("gdbus")
        .args([
            "call",
            "--session",
            "--dest",
            "org.gnome.Shell",
            "--object-path",
            "/org/boring/WindowTracker",
            "--method",
            "org.boring.WindowTracker.GetActiveWindow",
        ])
        .output()
    {
        if output.status.success() {
            return (true, crate::utils::i18n::tr(lang, "gnome_ext_status_active").to_string());
        }
    }

    let home = std::env::var("HOME").unwrap_or_default();
    let ext_dir = PathBuf::from(&home).join(".local/share/gnome-shell/extensions/bo-ring-window-tracker@flavien");

    if ext_dir.exists() {
        (false, crate::utils::i18n::tr(lang, "gnome_ext_status_restart_req").to_string())
    } else {
        (false, crate::utils::i18n::tr(lang, "gnome_ext_status_not_installed").to_string())
    }
}

/// Checks if the GNOME Shell extension DBus service is active or installed.
pub fn check_gnome_extension_status() -> (bool, String) {
    check_gnome_extension_status_lang("en")
}

/// Installs or updates the GNOME Shell extension directly into the user's home directory (with language localization).
pub fn install_or_update_gnome_extension_lang(lang: &str) -> Result<String, String> {
    let home = std::env::var("HOME").map_err(|_| "Unable to locate $HOME directory".to_string())?;
    let target_dir = PathBuf::from(home).join(".local/share/gnome-shell/extensions/bo-ring-window-tracker@flavien");

    std::fs::create_dir_all(&target_dir)
        .map_err(|e| format!("Error creating extension directory: {e}"))?;

    let metadata_content = include_str!("../../assets/gnome_extension/metadata.json");
    let extension_content = include_str!("../../assets/gnome_extension/extension.js");

    std::fs::write(target_dir.join("metadata.json"), metadata_content)
        .map_err(|e| format!("Error writing metadata.json: {e}"))?;
    std::fs::write(target_dir.join("extension.js"), extension_content)
        .map_err(|e| format!("Error writing extension.js: {e}"))?;

    let _ = Command::new("gnome-extensions")
        .args(["enable", "bo-ring-window-tracker@flavien"])
        .output();

    Ok(crate::utils::i18n::tr(lang, "gnome_ext_deploy_success").to_string())
}

/// Installs or updates the GNOME Shell extension directly into the user's home directory.
pub fn install_or_update_gnome_extension() -> Result<String, String> {
    install_or_update_gnome_extension_lang("en")
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_detect_gnome_window() {
        // In headless CI or non-GNOME desktop environments, detect_gnome_window()
        // gracefully returns None instead of panicking.
        let app = detect_gnome_window();
        println!("DETECTED GNOME WINDOW: {:?}", app);
        if let Some(ref name) = app {
            assert!(!name.is_empty());
        }
    }
}

