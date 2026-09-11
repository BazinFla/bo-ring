use std::fs;

pub fn detect_os_accent_color() -> Option<String> {
    // 1. Try via XDG Desktop Portal / DBus
    if let Some(color) = detect_accent_color_portal() {
        return Some(color);
    }

    // 2. Try via gsettings (GNOME)
    if let Some(color) = detect_accent_color_gnome() {
        return Some(color);
    }

    // 3. Try via KDE Plasma config (~/.config/kdeglobals)
    if let Some(color) = detect_accent_color_kde() {
        return Some(color);
    }

    None
}

fn get_user_and_bus_env() -> (Option<String>, Option<String>, String) {
    let sudo_user = std::env::var("SUDO_USER").ok();
    let target_user = sudo_user.clone().unwrap_or_else(|| {
        std::env::var("USER").unwrap_or_default()
    });
    
    let user_home = if let Some(ref u) = sudo_user {
        format!("/home/{}", u)
    } else {
        std::env::var("HOME").unwrap_or_default()
    };

    let mut bus_address = std::env::var("DBUS_SESSION_BUS_ADDRESS").ok();
    if bus_address.is_none() && !target_user.is_empty() {
        if let Ok(out) = std::process::Command::new("id").args(["-u", &target_user]).output() {
            if out.status.success() {
                let uid = String::from_utf8_lossy(&out.stdout).trim().to_string();
                bus_address = Some(format!("unix:path=/run/user/{}/bus", uid));
            }
        }
    }

    (sudo_user, bus_address, user_home)
}

fn detect_accent_color_portal() -> Option<String> {
    let (_, bus_address, _) = get_user_and_bus_env();
    let mut cmd = std::process::Command::new("dbus-send");
    cmd.args([
        "--session",
        "--print-reply",
        "--dest=org.freedesktop.portal.Desktop",
        "/org/freedesktop/portal/desktop",
        "org.freedesktop.portal.Settings.Read",
        "string:org.freedesktop.appearance",
        "string:accent-color",
    ]);

    if let Some(bus) = bus_address {
        cmd.env("DBUS_SESSION_BUS_ADDRESS", bus);
    }

    if let Ok(out) = cmd.output() {
        if out.status.success() {
            let output_str = String::from_utf8_lossy(&out.stdout);
            return parse_portal_accent_color(&output_str);
        }
    }
    None
}

fn parse_portal_accent_color(output: &str) -> Option<String> {
    let mut doubles = Vec::new();
    for line in output.lines() {
        let trimmed = line.trim();
        if trimmed.starts_with("double ") {
            if let Ok(val) = trimmed.trim_start_matches("double ").trim().parse::<f64>() {
                doubles.push(val);
            }
        }
    }
    if doubles.len() >= 3 {
        let r = (doubles[0].clamp(0.0, 1.0) * 255.0).round() as u8;
        let g = (doubles[1].clamp(0.0, 1.0) * 255.0).round() as u8;
        let b = (doubles[2].clamp(0.0, 1.0) * 255.0).round() as u8;
        return Some(format!("#{:02X}{:02X}{:02X}", r, g, b));
    }
    None
}

fn detect_accent_color_gnome() -> Option<String> {
    let (sudo_user, bus_address, _) = get_user_and_bus_env();
    let mut cmd = if let Some(ref u) = sudo_user {
        let mut c = std::process::Command::new("sudo");
        c.args(["-u", u, "gsettings", "get", "org.gnome.desktop.interface", "accent-color"]);
        c
    } else {
        let mut c = std::process::Command::new("gsettings");
        c.args(["get", "org.gnome.desktop.interface", "accent-color"]);
        c
    };

    if let Some(bus) = bus_address {
        cmd.env("DBUS_SESSION_BUS_ADDRESS", bus);
    }

    if let Ok(out) = cmd.output() {
        if out.status.success() {
            let val = String::from_utf8_lossy(&out.stdout);
            return gnome_accent_to_hex(&val);
        }
    }
    None
}

fn gnome_accent_to_hex(name: &str) -> Option<String> {
    let clean = name.trim().trim_matches('\'').trim_matches('"');
    match clean {
        "blue" => Some("#3584E4".to_string()),
        "teal" => Some("#129EAF".to_string()),
        "green" => Some("#2EC27E".to_string()),
        "yellow" => Some("#F5C211".to_string()),
        "orange" => Some("#E66100".to_string()),
        "red" => Some("#E01B24".to_string()),
        "pink" => Some("#D1396E".to_string()),
        "purple" => Some("#9141AC".to_string()),
        "slate" => Some("#62A0EA".to_string()),
        s if s.starts_with('#') && (s.len() == 7 || s.len() == 9) => Some(s[..7].to_string()),
        _ => None,
    }
}

fn detect_accent_color_kde() -> Option<String> {
    let (_, _, user_home) = get_user_and_bus_env();
    let kde_config = format!("{}/.config/kdeglobals", user_home);
    if let Ok(content) = fs::read_to_string(&kde_config) {
        let mut in_selection_sec = false;
        for line in content.lines() {
            let trimmed = line.trim();
            if trimmed == "[Colors:Selection]" {
                in_selection_sec = true;
                continue;
            }
            if in_selection_sec && trimmed.starts_with('[') {
                break;
            }
            if in_selection_sec && trimmed.starts_with("Background=") {
                let rgb_str = trimmed.trim_start_matches("Background=");
                let parts: Vec<u8> = rgb_str.split(',').filter_map(|s| s.trim().parse().ok()).collect();
                if parts.len() == 3 {
                    return Some(format!("#{:02X}{:02X}{:02X}", parts[0], parts[1], parts[2]));
                }
            }
        }
    }
    None
}
