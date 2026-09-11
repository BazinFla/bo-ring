use std::env;
use std::fs;
use std::path::PathBuf;
use std::process::Command;

pub fn get_exe_path() -> PathBuf {
    env::current_exe().unwrap_or_else(|_| PathBuf::from("bo-ring"))
}

pub fn get_home_dir() -> PathBuf {
    env::var("HOME").map(PathBuf::from).unwrap_or_else(|_| PathBuf::from("/tmp"))
}

// ==========================================
// 1. User Systemd Service
// ==========================================

pub fn systemd_service_path() -> PathBuf {
    get_home_dir().join(".config/systemd/user/bo-ring.service")
}

pub fn systemd_system_unit_path() -> PathBuf {
    PathBuf::from("/usr/lib/systemd/user/bo-ring.service")
}

pub fn is_systemd_installed() -> bool {
    systemd_service_path().exists() || systemd_system_unit_path().exists()
}

pub fn is_systemd_enabled() -> bool {
    let output = Command::new("systemctl")
        .args(["--user", "is-enabled", "bo-ring.service"])
        .output();

    if let Ok(out) = output {
        let stdout = String::from_utf8_lossy(&out.stdout).trim().to_string();
        return stdout == "enabled";
    }
    false
}

pub fn is_systemd_active() -> bool {
    let output = Command::new("systemctl")
        .args(["--user", "is-active", "bo-ring.service"])
        .output();

    if let Ok(out) = output {
        let stdout = String::from_utf8_lossy(&out.stdout).trim().to_string();
        if stdout == "active" {
            return true;
        }
    }
    false
}

pub fn enable_systemd_service() -> Result<(), String> {
    let user_path = systemd_service_path();
    let sys_path = systemd_system_unit_path();

    // If no system unit exists, create user unit in ~/.config/systemd/user/
    if !sys_path.exists() {
        let exe = get_exe_path();
        let service_dir = get_home_dir().join(".config/systemd/user");
        fs::create_dir_all(&service_dir).map_err(|e| format!("Failed to create systemd directory: {}", e))?;

        let service_content = format!(
            "[Unit]\n\
             Description=Bo-Ring Daemon\n\
             After=graphical-session.target\n\
             StartLimitIntervalSec=0\n\n\
             [Service]\n\
             ExecStart={} --daemon\n\
             Restart=on-failure\n\
             RestartSec=3\n\n\
             [Install]\n\
             WantedBy=graphical-session.target\n",
            exe.to_string_lossy()
        );

        fs::write(&user_path, service_content).map_err(|e| format!("Failed to write service file: {}", e))?;
    }

    // Reload and enable systemd user service
    let _ = Command::new("systemctl").args(["--user", "daemon-reload"]).output();
    let res = Command::new("systemctl").args(["--user", "enable", "--now", "bo-ring.service"]).output();

    match res {
        Ok(out) if out.status.success() => Ok(()),
        Ok(out) => Err(String::from_utf8_lossy(&out.stderr).trim().to_string()),
        Err(e) => Err(e.to_string()),
    }
}

pub fn disable_systemd_service() -> Result<(), String> {
    let res = Command::new("systemctl").args(["--user", "disable", "--now", "bo-ring.service"]).output();
    let user_path = systemd_service_path();
    if user_path.exists() && systemd_system_unit_path().exists() {
        let _ = fs::remove_file(user_path);
    }
    let _ = Command::new("systemctl").args(["--user", "daemon-reload"]).output();

    match res {
        Ok(out) if out.status.success() => Ok(()),
        Ok(out) => Err(String::from_utf8_lossy(&out.stderr).trim().to_string()),
        Err(e) => Err(e.to_string()),
    }
}

pub fn restart_systemd_service_if_active() {
    if is_systemd_active() {
        println!("🔄 Restarting bo-ring daemon service to apply new configuration...");
        let _ = Command::new("systemctl").args(["--user", "restart", "bo-ring.service"]).output();
    }
}

pub fn ensure_gnome_show_desktop_keybinding() {
    let output = Command::new("gsettings")
        .args(["get", "org.gnome.desktop.wm.keybindings", "show-desktop"])
        .output();
    if let Ok(out) = output {
        let stdout = String::from_utf8_lossy(&out.stdout).trim().to_string();
        if stdout == "@as []" || stdout == "[]" {
            println!("⚙️ Enabling GNOME show-desktop keybinding ('<Super>d', '<Control><Alt>d')...");
            let _ = Command::new("gsettings")
                .args(["set", "org.gnome.desktop.wm.keybindings", "show-desktop", "['<Super>d', '<Control><Alt>d']"])
                .output();
        }
    }
}

// ==========================================
// 2. XDG Autostart Desktop File
// ==========================================

pub fn xdg_autostart_path() -> PathBuf {
    get_home_dir().join(".config/autostart/bo-ring.desktop")
}

pub fn is_xdg_autostart_installed() -> bool {
    xdg_autostart_path().exists()
}

pub fn enable_xdg_autostart() -> Result<(), String> {
    let exe = get_exe_path();
    let autostart_dir = get_home_dir().join(".config/autostart");
    fs::create_dir_all(&autostart_dir).map_err(|e| format!("Failed to create autostart directory: {}", e))?;

    let desktop_content = format!(
        "[Desktop Entry]\n\
         Type=Application\n\
         Name=Bo-Ring Daemon\n\
         Comment=Bo-Ring background daemon\n\
         Exec={} --daemon\n\
         Icon=input-mouse\n\
         Terminal=false\n\
         Categories=Utility;Settings;\n\
         X-GNOME-Autostart-enabled=true\n",
        exe.to_string_lossy()
    );

    let path = xdg_autostart_path();
    fs::write(path, desktop_content).map_err(|e| format!("Failed to write .desktop autostart file: {}", e))?;
    Ok(())
}

pub fn disable_xdg_autostart() -> Result<(), String> {
    let path = xdg_autostart_path();
    if path.exists() {
        fs::remove_file(path).map_err(|e| format!("Failed to delete autostart file: {}", e))?;
    }
    Ok(())
}

// ==========================================
// 3. GNOME Application Menu Shortcut
// ==========================================

pub fn gnome_menu_path() -> PathBuf {
    get_home_dir().join(".local/share/applications/bo-ring.desktop")
}

pub fn gnome_system_menu_path() -> PathBuf {
    PathBuf::from("/usr/share/applications/bo-ring.desktop")
}


pub fn is_gnome_menu_system() -> bool {
    gnome_system_menu_path().exists()
}

pub fn install_gnome_menu() -> Result<(), String> {
    let exe = get_exe_path();
    let apps_dir = get_home_dir().join(".local/share/applications");
    fs::create_dir_all(&apps_dir).map_err(|e| format!("Failed to create applications directory: {}", e))?;

    let desktop_content = format!(
        "[Desktop Entry]\n\
         Type=Application\n\
         Name=Bo-Ring\n\
         Comment=Bo-Ring GUI Configurator and Action Ring\n\
         Exec={}\n\
         Icon=input-mouse\n\
         StartupWMClass=bo-ring\n\
         Terminal=false\n\
         Categories=Settings;HardwareSettings;Utility;\n",
        exe.to_string_lossy()
    );

    let path = gnome_menu_path();
    fs::write(path, desktop_content).map_err(|e| format!("Failed to write GNOME shortcut: {}", e))?;

    // Update desktop database
    let _ = Command::new("update-desktop-database")
        .arg(apps_dir.to_string_lossy().to_string())
        .output();

    Ok(())
}

pub fn remove_gnome_menu() -> Result<(), String> {
    let path = gnome_menu_path();
    if path.exists() {
        fs::remove_file(path).map_err(|e| format!("Failed to delete GNOME shortcut: {}", e))?;
    }
    Ok(())
}

// ==========================================
// 4. Check Permissions / uinput
// ==========================================

pub fn check_uinput_access() -> bool {
    fs::OpenOptions::new()
        .write(true)
        .open("/dev/uinput")
        .is_ok()
}
