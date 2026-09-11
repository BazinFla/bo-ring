use eframe::egui;
use std::path::PathBuf;

/// Get the work area dimensions (monitor height minus shell chrome like GNOME top bar).
///
/// Tries multiple strategies in order of accuracy:
/// 1. GNOME Shell Extension DBus (active monitor where cursor is)
/// 2. GNOME Mutter DBus DisplayConfig (primary monitor resolution & scale)
/// 3. Hyprland IPC (`hyprctl monitors -j`)
/// 4. Sway / wlroots IPC (`swaymsg -t get_outputs`)
/// 5. X11 / XWayland (`_NET_WORKAREA` via `xprop` + `xrandr`)
/// 6. Linux DRM sysfs modes (`/sys/class/drm/*/modes`, sorted by resolution)
pub fn get_work_area_size() -> Option<(f32, f32)> {
    // 1. Try GNOME Shell Extension DBus (exact active monitor containing the cursor)
    if let Some(dims) = get_work_area_gnome_extension() {
        return Some(dims);
    }

    // 2. Try GNOME Mutter DBus (primary monitor geometry from compositor)
    if let Some(dims) = get_work_area_mutter() {
        return Some(dims);
    }

    // 3. Try Hyprland IPC
    if let Ok(output) = std::process::Command::new("hyprctl").args(["monitors", "-j"]).output() {
        if output.status.success() {
            if let Ok(val) = serde_json::from_slice::<serde_json::Value>(&output.stdout) {
                if let Some(arr) = val.as_array() {
                    for mon in arr {
                        let is_focused = mon.get("focused").and_then(|v| v.as_bool()).unwrap_or(false);
                        if is_focused || arr.len() == 1 {
                            if let (Some(w), Some(h)) = (mon.get("width").and_then(|v| v.as_f64()), mon.get("height").and_then(|v| v.as_f64())) {
                                let scale = mon.get("scale").and_then(|v| v.as_f64()).unwrap_or(1.0);
                                return Some(((w / scale) as f32, (h / scale) as f32));
                            }
                        }
                    }
                }
            }
        }
    }

    // 4. Try Sway / wlroots IPC
    if let Ok(output) = std::process::Command::new("swaymsg").args(["-t", "get_outputs"]).output() {
        if output.status.success() {
            if let Ok(val) = serde_json::from_slice::<serde_json::Value>(&output.stdout) {
                if let Some(arr) = val.as_array() {
                    for mon in arr {
                        let is_focused = mon.get("focused").and_then(|v| v.as_bool()).unwrap_or(false);
                        if is_focused || arr.len() == 1 {
                            if let Some(rect) = mon.get("rect") {
                                if let (Some(w), Some(h)) = (rect.get("width").and_then(|v| v.as_f64()), rect.get("height").and_then(|v| v.as_f64())) {
                                    return Some((w as f32, h as f32));
                                }
                            }
                        }
                    }
                }
            }
        }
    }

    // 5. Try X11 / XWayland with xrandr + xprop
    if let Some(dims) = get_work_area_x11() {
        return Some(dims);
    }

    // 6. Try Linux DRM sysfs modes (sorted by resolution)
    if let Some(dims) = get_work_area_drm() {
        return Some(dims);
    }

    None
}

fn get_work_area_gnome_extension() -> Option<(f32, f32)> {
    let output = std::process::Command::new("gdbus")
        .args([
            "call",
            "--session",
            "--dest",
            "org.gnome.Shell",
            "--object-path",
            "/org/boring/WindowTracker",
            "--method",
            "org.boring.WindowTracker.GetActiveMonitorWorkArea",
        ])
        .output()
        .ok()?;

    if !output.status.success() {
        return None;
    }

    let stdout = String::from_utf8_lossy(&output.stdout);
    let clean = stdout.trim().trim_matches(|c| c == '(' || c == ')' || c == ' ');
    let nums: Vec<f32> = clean
        .split(',')
        .filter_map(|s| s.trim().parse::<f32>().ok())
        .collect();

    if nums.len() >= 4 {
        let w = nums[2];
        let h = nums[3];
        if w >= 640.0 && h >= 480.0 {
            return Some((w, h));
        }
    }
    None
}

fn get_work_area_mutter() -> Option<(f32, f32)> {
    let output = std::process::Command::new("gdbus")
        .args([
            "call",
            "--session",
            "--dest",
            "org.gnome.Mutter.DisplayConfig",
            "--object-path",
            "/org/gnome/Mutter/DisplayConfig",
            "--method",
            "org.gnome.Mutter.DisplayConfig.GetCurrentState",
        ])
        .output()
        .ok()?;

    if !output.status.success() {
        return None;
    }

    let stdout = String::from_utf8_lossy(&output.stdout);

    // Search for primary logical monitor (contains "true, [('")
    let mut connector = None;
    let mut scale = 1.0_f32;

    let marker = "true, [('";
    if let Some(pos) = stdout.find(marker) {
        let rest = &stdout[pos + marker.len()..];
        if let Some(end) = rest.find('\'') {
            connector = Some(rest[..end].to_string());
        }
        let start = pos.saturating_sub(40);
        let prefix = &stdout[start..pos];
        for p in prefix.split(',') {
            if let Ok(s) = p.trim().parse::<f32>() {
                if (0.5..=4.0).contains(&s) {
                    scale = s;
                }
            }
        }
    }

    // Fallback: search for any logical monitor connector
    if connector.is_none() {
        let marker2 = "[('";
        if let Some(pos2) = stdout.find(marker2) {
            let rest2 = &stdout[pos2 + marker2.len()..];
            if let Some(end2) = rest2.find('\'') {
                connector = Some(rest2[..end2].to_string());
            }
        }
    }

    let conn = connector?;
    let conn_marker = format!("(('{}'", conn);
    let conn_pos = stdout.find(&conn_marker)?;
    let conn_chunk = &stdout[conn_pos..];

    // Find active mode in this connector's block
    let cur_marker = "'is-current': <true>";
    let cur_pos = conn_chunk.find(cur_marker)?;
    let sub = &conn_chunk[..cur_pos];
    let last_paren = sub.rfind('(')?;
    let mode_tuple = &sub[last_paren..];
    let parts: Vec<&str> = mode_tuple.split(',').collect();
    if parts.len() >= 3 {
        let w: f32 = parts[1].trim().parse().ok()?;
        let h: f32 = parts[2].trim().parse().ok()?;
        let logical_w = w / scale;
        let logical_h = (h / scale).max(100.0);
        return Some((logical_w, logical_h));
    }

    None
}

fn get_work_area_drm() -> Option<(f32, f32)> {
    let mut modes = Vec::new();
    if let Ok(entries) = std::fs::read_dir("/sys/class/drm") {
        for entry in entries.flatten() {
            let path = entry.path();
            let status_file = path.join("status");
            if let Ok(status) = std::fs::read_to_string(&status_file) {
                if !status.trim().eq_ignore_ascii_case("connected") {
                    continue;
                }
            }
            let mode_file = path.join("modes");
            if let Ok(content) = std::fs::read_to_string(&mode_file) {
                if let Some(first_line) = content.lines().next() {
                    let parts: Vec<&str> = first_line.trim().split('x').collect();
                    if parts.len() == 2 {
                        if let (Ok(w), Ok(h)) = (parts[0].parse::<f32>(), parts[1].parse::<f32>()) {
                            if w > 0.0 && h > 0.0 {
                                modes.push((w, h));
                            }
                        }
                    }
                }
            }
        }
    }

    modes.sort_by(|a, b| (b.0 * b.1).partial_cmp(&(a.0 * a.1)).unwrap_or(std::cmp::Ordering::Equal));
    if let Some((w, h)) = modes.first() {
        return Some((*w, (*h).max(100.0)));
    }
    None
}


fn get_work_area_x11() -> Option<(f32, f32)> {
    let top_bar_height = std::process::Command::new("xprop")
        .args(["-root", "_NET_WORKAREA"])
        .output()
        .ok()
        .and_then(|output| {
            let stdout = String::from_utf8_lossy(&output.stdout).to_string();
            let values_part = stdout.split('=').nth(1)?;
            let nums: Vec<f32> = values_part
                .split(',')
                .take(4)
                .filter_map(|s| s.trim().parse::<f32>().ok())
                .collect();
            if nums.len() >= 4 { Some(nums[1]) } else { None }
        })
        .unwrap_or(32.0);

    if let Ok(output) = std::process::Command::new("xrandr").arg("--current").output() {
        if output.status.success() {
            let stdout = String::from_utf8_lossy(&output.stdout);
            for line in stdout.lines() {
                if line.contains(" connected") && line.contains("primary") {
                    for word in line.split_whitespace() {
                        if word.contains('x') && word.contains('+') {
                            let parts: Vec<&str> = word.split(|c| c == 'x' || c == '+').collect();
                            if parts.len() >= 2 {
                                let w: f32 = parts[0].parse().ok()?;
                                let h: f32 = parts[1].parse().ok()?;
                                return Some((w, h - top_bar_height));
                            }
                        }
                    }
                }
            }
        }
    }
    None
}

pub fn ring_pid_file_path() -> PathBuf {
    if let Ok(dir) = std::env::var("XDG_RUNTIME_DIR") {
        PathBuf::from(dir).join("bo-ring-ring.pid")
    } else {
        PathBuf::from("/tmp/bo-ring-ring.pid")
    }
}

pub fn is_pid_boring(pid: i32) -> bool {
    let cmdline_path = format!("/proc/{}/cmdline", pid);
    if let Ok(cmdline) = std::fs::read_to_string(&cmdline_path) {
        return cmdline.contains("bo-ring");
    }
    false
}

pub fn cleanup_stale_ring_pid() {
    let pid_path = ring_pid_file_path();
    if pid_path.exists() {
        if let Ok(content) = std::fs::read_to_string(&pid_path) {
            if let Ok(pid) = content.trim().parse::<i32>() {
                if !is_pid_boring(pid) {
                    println!("🧹 Cleaning stale Ring Menu PID file (PID {} dead/reused)...", pid);
                    let _ = std::fs::remove_file(&pid_path);
                }
            } else {
                let _ = std::fs::remove_file(&pid_path);
            }
        } else {
            let _ = std::fs::remove_file(&pid_path);
        }
    }
}

pub fn build_ring_overlay_viewport() -> egui::ViewportBuilder {
    let (work_w, work_h) = get_work_area_size().unwrap_or((2560.0, 1440.0));
    println!("📐 Work area size: {}x{}", work_w, work_h);

    egui::ViewportBuilder::default()
        .with_app_id("bo-ring-overlay")
        .with_maximized(true)
        .with_inner_size([work_w, work_h])
        .with_decorations(false)
        .with_always_on_top()
        .with_transparent(true)
}

/// Safely sends a POSIX signal to a specific process ID.
///
/// Invariants enforced:
/// - `pid > 0` : Strictly positive PID check prevents POSIX broadcast behaviors:
///   - `pid == 0` would target the caller's entire process group.
///   - `pid == -1` would broadcast to all processes the user has permission to signal.
///
/// Returns `true` if the signal was successfully delivered, `false` otherwise.
pub fn safe_kill(pid: i32, sig: i32) -> bool {
    if pid <= 0 {
        return false;
    }
    // Safety: pid is strictly positive (> 0), targeting only a specific process.
    let ret = unsafe { libc::kill(pid, sig) };
    ret == 0
}

/// Safely terminates an active Bo-Ring overlay process after validating its identity.
///
/// Guarantees:
/// 1. `pid > 0` (via `safe_kill`).
/// 2. `/proc/{pid}/cmdline` contains "bo-ring" to protect against OS PID recycling.
/// 3. Sends `SIGTERM` and returns whether delivery succeeded.
pub fn terminate_ring_process(pid: i32) -> bool {
    if !is_pid_boring(pid) {
        return false;
    }
    safe_kill(pid, libc::SIGTERM)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_get_work_area_size() {
        let size = get_work_area_size();
        println!("Detected work area size: {:?}", size);
        if let Some((w, h)) = size {
            assert!(w > 0.0);
            assert!(h > 0.0);
        }
    }

    #[test]
    fn test_safe_kill_invalid_pid() {
        assert!(!safe_kill(0, libc::SIGTERM));
        assert!(!safe_kill(-1, libc::SIGTERM));
        assert!(!safe_kill(-100, libc::SIGTERM));
        assert!(!terminate_ring_process(0));
        assert!(!terminate_ring_process(-1));
    }
}
