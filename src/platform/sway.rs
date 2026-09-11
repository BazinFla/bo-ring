use std::process::Command;

/// Strategy 4: Sway / wlroots (Wayland)
/// Completely isolated in `sway.rs`.
pub fn detect_sway_window() -> Option<String> {
    if let Ok(output) = Command::new("swaymsg").args(["-t", "get_tree"]).output() {
        if output.status.success() {
            let str_out = String::from_utf8_lossy(&output.stdout);
            if let Ok(v) = serde_json::from_str::<serde_json::Value>(&str_out) {
                if let Some(app) = find_sway_focused(&v) {
                    if !app.is_empty() {
                        return Some(app);
                    }
                }
            }
        }
    }
    None
}

fn find_sway_focused(v: &serde_json::Value) -> Option<String> {
    if v.get("focused").and_then(|f| f.as_bool()) == Some(true) {
        if let Some(app) = v.get("app_id").and_then(|a| a.as_str()) {
            if !app.is_empty() { return Some(app.to_string()); }
        }
        if let Some(props) = v.get("window_properties") {
            if let Some(cls) = props.get("class").and_then(|c| c.as_str()) {
                if !cls.is_empty() { return Some(cls.to_string()); }
            }
        }
        if let Some(name) = v.get("name").and_then(|n| n.as_str()) {
            if !name.is_empty() { return Some(name.to_string()); }
        }
    }
    if let Some(nodes) = v.get("nodes").and_then(|n| n.as_array()) {
        for n in nodes {
            if let Some(res) = find_sway_focused(n) { return Some(res); }
        }
    }
    if let Some(fnodes) = v.get("floating_nodes").and_then(|f| f.as_array()) {
        for n in fnodes {
            if let Some(res) = find_sway_focused(n) { return Some(res); }
        }
    }
    None
}
