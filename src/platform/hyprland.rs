use std::process::Command;

/// Strategy 3: Hyprland (Wayland)
/// Completely isolated in `hyprland.rs`.
pub fn detect_hyprland_window() -> Option<String> {
    if let Ok(output) = Command::new("hyprctl").args(["activewindow", "-j"]).output() {
        if output.status.success() {
            let str_out = String::from_utf8_lossy(&output.stdout);
            if let Ok(v) = serde_json::from_str::<serde_json::Value>(&str_out) {
                if let Some(cls) = v.get("class").and_then(|c| c.as_str()) {
                    if !cls.is_empty() {
                        return Some(cls.to_string());
                    }
                }
                if let Some(title) = v.get("title").and_then(|t| t.as_str()) {
                    if !title.is_empty() {
                        return Some(title.to_string());
                    }
                }
            }
        }
    }
    None
}
