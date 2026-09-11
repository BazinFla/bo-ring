use std::process::Command;

/// Dedicated window detection strategy for KDE Plasma (Wayland & X11).
/// Uses `kdotool` as primary Wayland strategy, with `xprop` (+ /proc/PID/comm) fallback.
/// Completely isolated in `kde.rs` to ensure GNOME stability is never affected.
pub fn detect_kde_window() -> Option<String> {
    // Strategy 1: kdotool (Primary KDE Plasma Wayland CLI utility - 1ms execution)
    for kdo_arg in ["getwindowclassname", "getwindowname"] {
        if let Ok(output) = Command::new("kdotool")
            .args(["getactivewindow", kdo_arg])
            .output()
        {
            if output.status.success() {
                let name = String::from_utf8_lossy(&output.stdout).trim().to_string();
                if !name.is_empty() && !is_boring_self(&name) {
                    return Some(format_app_name(&name));
                }
            }
        }
    }

    // Strategy 2: Fast xprop + PID resolution (_NET_ACTIVE_WINDOW -> _NET_WM_PID -> /proc/PID/comm)
    if let Some(xprop_app) = resolve_kde_via_xprop() {
        if !is_boring_self(&xprop_app) {
            return Some(format_app_name(&xprop_app));
        }
    }

    None
}

/// Fast, non-blocking xprop + _NET_WM_PID -> /proc/<pid>/comm resolver
pub fn resolve_kde_via_xprop() -> Option<String> {
    let output = Command::new("xprop")
        .args(["-root", "_NET_ACTIVE_WINDOW"])
        .output()
        .ok()?;

    if !output.status.success() {
        return None;
    }

    let stdout = String::from_utf8_lossy(&output.stdout);
    let win_id = stdout.split_whitespace().last()?;
    if win_id == "0x0" || win_id == "none" {
        return None;
    }

    resolve_x11_window_id(win_id)
}

/// Detailed diagnostic runner specifically for KDE Plasma KWin.
pub fn run_kde_diagnostics() -> String {
    let mut log = String::new();
    log.push_str("🔍 === ISOLATED KDE PLASMA KWIN DIAGNOSTICS ===\n\n");

    log.push_str(&format!("• XDG_CURRENT_DESKTOP: {}\n", std::env::var("XDG_CURRENT_DESKTOP").unwrap_or_else(|_| "undefined".into())));
    log.push_str(&format!("• XDG_SESSION_TYPE: {}\n", std::env::var("XDG_SESSION_TYPE").unwrap_or_else(|_| "undefined".into())));
    log.push_str(&format!("• KDE_SESSION_VERSION: {}\n\n", std::env::var("KDE_SESSION_VERSION").unwrap_or_else(|_| "undefined".into())));

    // Test 1: kdotool (Primary for KDE Wayland)
    log.push_str("1️⃣ [kdotool] getactivewindow (Primary Wayland strategy)\n");
    for arg in ["getwindowclassname", "getwindowname"] {
        log.push_str(&format!("   ▶ Command: kdotool getactivewindow {arg}\n"));
        match Command::new("kdotool").args(["getactivewindow", arg]).output() {
            Ok(out) => {
                log.push_str(&format!("      Exit Code: {}\n", out.status));
                log.push_str(&format!("      Stdout: {}\n", String::from_utf8_lossy(&out.stdout).trim()));
            }
            Err(e) => log.push_str(&format!("      Error: {e} (kdotool not installed)\n")),
        }
    }
    log.push('\n');

    // Test 2: xprop + PID resolution
    log.push_str("2️⃣ [xprop + PID Resolution (Fallback)]\n");
    log.push_str("   ▶ Command: xprop -root _NET_ACTIVE_WINDOW\n");
    match Command::new("xprop").args(["-root", "_NET_ACTIVE_WINDOW"]).output() {
        Ok(out) => {
            log.push_str(&format!("   Exit Code: {}\n", out.status));
            let str_out = String::from_utf8_lossy(&out.stdout).trim().to_string();
            log.push_str(&format!("   Stdout: {}\n", str_out));
            if let Some(win_id) = str_out.split_whitespace().last() {
                if win_id != "0x0" && win_id != "none" {
                    log.push_str(&format!("   ▶ Command: xprop -id {win_id} WM_CLASS\n"));
                    if let Ok(class_out) = Command::new("xprop").args(["-id", win_id, "WM_CLASS"]).output() {
                        log.push_str(&format!("   Stdout WM_CLASS: {}\n", String::from_utf8_lossy(&class_out.stdout).trim()));
                    }

                    log.push_str(&format!("   ▶ Command: xprop -id {win_id} _NET_WM_PID\n"));
                    if let Ok(pid_out) = Command::new("xprop").args(["-id", win_id, "_NET_WM_PID"]).output() {
                        let p_str = String::from_utf8_lossy(&pid_out.stdout);
                        log.push_str(&format!("   Stdout _NET_WM_PID: {}\n", p_str.trim()));
                        if let Some(pid_val) = p_str.split('=').nth(1) {
                            if let Ok(pid) = pid_val.trim().parse::<u32>() {
                                let comm_path = format!("/proc/{}/comm", pid);
                                if let Ok(comm) = std::fs::read_to_string(&comm_path) {
                                    log.push_str(&format!("   Resolved via /proc/{pid}/comm: '{}'\n", comm.trim()));
                                }
                            }
                        }
                    }
                }
            }
        }
        Err(e) => log.push_str(&format!("   Error: {e}\n")),
    }

    log.push_str(&format!("\n🎯 ISOLATED KDE DETECTION RESULT: {:?}\n", detect_kde_window()));
    log
}

fn format_app_name(raw: &str) -> String {
    let clean = raw.trim();
    if clean.contains('.') {
        if let Some(short) = clean.rsplit('.').next() {
            if !short.is_empty() {
                return short.to_string();
            }
        }
    }
    clean.to_string()
}

fn is_boring_self(name: &str) -> bool {
    let lower = name.trim().to_lowercase();
    lower.contains("bo-ring") || lower.contains("boring")
}

fn resolve_x11_window_id(id_str: &str) -> Option<String> {
    let clean_id = id_str.trim().trim_matches('\'').trim_matches('"').trim();
    if clean_id.is_empty() || clean_id == "0" || clean_id == "0x0" {
        return None;
    }

    if clean_id.starts_with("0x") || clean_id.chars().all(|c| c.is_ascii_digit()) {
        // 1. Try WM_CLASS
        if let Ok(class_out) = Command::new("xprop").args(["-id", clean_id, "WM_CLASS"]).output() {
            if class_out.status.success() {
                let c_str = String::from_utf8_lossy(&class_out.stdout);
                if let Some(val) = c_str.split('=').nth(1) {
                    let parts: Vec<&str> = val.split(',').collect();
                    if let Some(last) = parts.last() {
                        let clean = last.trim().trim_matches('"').trim();
                        if !clean.is_empty() && clean != "not found." && !is_boring_self(clean) {
                            return Some(clean.to_string());
                        }
                    }
                }
            }
        }

        // 2. Try _NET_WM_PID -> /proc/<pid>/comm
        if let Ok(pid_out) = Command::new("xprop").args(["-id", clean_id, "_NET_WM_PID"]).output() {
            if pid_out.status.success() {
                let p_str = String::from_utf8_lossy(&pid_out.stdout);
                if let Some(pid_val) = p_str.split('=').nth(1) {
                    let clean_pid = pid_val.trim();
                    if let Ok(pid) = clean_pid.parse::<u32>() {
                        let comm_path = format!("/proc/{}/comm", pid);
                        if let Ok(comm) = std::fs::read_to_string(&comm_path) {
                            let app_name = comm.trim().to_string();
                            if !app_name.is_empty() && !is_boring_self(&app_name) {
                                return Some(app_name);
                            }
                        }
                    }
                }
            }
        }

        // 3. Try _NET_WM_NAME or WM_NAME
        if let Ok(name_out) = Command::new("xprop").args(["-id", clean_id, "_NET_WM_NAME"]).output() {
            if name_out.status.success() {
                let n_str = String::from_utf8_lossy(&name_out.stdout);
                if let Some(val) = n_str.split('=').nth(1) {
                    let clean = val.trim().trim_matches('"').trim();
                    if !clean.is_empty() && clean != "not found." && !is_boring_self(clean) {
                        return Some(clean.to_string());
                    }
                }
            }
        }
    }

    None
}
