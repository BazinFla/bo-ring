use std::process::Command;

/// Strategy 5: Generic X11 / XWayland (xprop)
/// Completely isolated in `x11.rs`.
pub fn detect_x11_window() -> Option<String> {
    if let Ok(output) = Command::new("xprop").args(["-root", "_NET_ACTIVE_WINDOW"]).output() {
        if output.status.success() {
            let str_out = String::from_utf8_lossy(&output.stdout);
            if let Some(win_id) = str_out.split_whitespace().last() {
                if win_id != "0x0" && win_id != "none" {
                    if let Ok(class_out) = Command::new("xprop").args(["-id", win_id, "WM_CLASS"]).output() {
                        if class_out.status.success() {
                            let c_str = String::from_utf8_lossy(&class_out.stdout);
                            if let Some(val) = c_str.split('=').nth(1) {
                                let parts: Vec<&str> = val.split(',').collect();
                                if let Some(last) = parts.last() {
                                    let clean = last.trim().trim_matches('"');
                                    if !clean.is_empty() && !clean.eq_ignore_ascii_case("gnome-shell") {
                                        return Some(clean.to_string());
                                    }
                                }
                            }
                        }
                    }
                    if let Ok(name_out) = Command::new("xprop").args(["-id", win_id, "_NET_WM_NAME"]).output() {
                        if name_out.status.success() {
                            let n_str = String::from_utf8_lossy(&name_out.stdout);
                            if let Some(val) = n_str.split('=').nth(1) {
                                let clean = val.trim().trim_matches('"');
                                if !clean.is_empty() && !clean.eq_ignore_ascii_case("gnome shell") {
                                    return Some(clean.to_string());
                                }
                            }
                        }
                    }
                }
            }
        }
    }
    None
}
