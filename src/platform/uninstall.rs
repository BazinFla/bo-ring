use std::env;
use std::fs;
use std::io::{self, Write};
use std::path::PathBuf;
use std::process::Command;

/// Interactive uninstallation routine for Bo-Ring.
pub fn run_uninstall() {
    let home = env::var("HOME").unwrap_or_else(|_| "/tmp".to_string());
    let home = PathBuf::from(home);

    let files_to_remove = [
        home.join(".local/bin/bo-ring"),
        home.join(".config/systemd/user/bo-ring.service"),
        home.join(".config/autostart/bo-ring.desktop"),
        home.join(".local/share/applications/bo-ring.desktop"),
    ];

    let dirs_to_remove = [
        home.join(".config/bo-ring"),
        home.join(".local/share/bo-ring"),
        home.join(".local/share/gnome-shell/extensions/bo-ring-window-tracker@flavien"),
    ];

    let udev_rules = [
        PathBuf::from("/etc/udev/rules.d/99-bo-ring-uinput.rules"),
    ];

    println!("\n🗑️  Bo-Ring Uninstaller\n");
    println!("The following files will be removed:\n");
    for f in &files_to_remove {
        if f.exists() {
            println!("  • {}", f.display());
        }
    }
    for d in &dirs_to_remove {
        if d.exists() {
            println!("  • {}/ (directory)", d.display());
        }
    }
    for u in &udev_rules {
        if u.exists() {
            println!("  • {} (requires sudo)", u.display());
        }
    }
    println!();

    print!("\x1b[31mProceed with uninstallation? [y/N] \x1b[0m");
    let _ = io::stdout().flush();

    let mut input = String::new();
    if io::stdin().read_line(&mut input).is_err() {
        return;
    }
    if !input.trim().eq_ignore_ascii_case("y") {
        println!("Uninstallation cancelled.");
        return;
    }

    // Disable GNOME Shell extensions
    let _ = Command::new("gnome-extensions")
        .args(["disable", "bo-ring-window-tracker@flavien"])
        .output();

    // Stop and disable systemd services
    let _ = Command::new("systemctl")
        .args(["--user", "stop", "bo-ring.service"])
        .output();
    let _ = Command::new("systemctl")
        .args(["--user", "disable", "bo-ring.service"])
        .output();

    // Remove files
    for f in &files_to_remove {
        if f.exists() {
            if let Err(e) = fs::remove_file(f) {
                eprintln!("  ⚠️  Failed to remove {}: {}", f.display(), e);
            } else {
                println!("  ✗ Removed {}", f.display());
            }
        }
    }

    let symlink = "/usr/local/bin/bo-ring";
    let global_symlink = PathBuf::from(symlink);
    if global_symlink.exists() {
        let _ = Command::new("sudo").args(["rm", "-f", symlink]).output();
        if !global_symlink.exists() {
            println!("  ✗ Removed {}", symlink);
        }
    }

    // Remove directories
    for d in &dirs_to_remove {
        if d.exists() {
            if let Err(e) = fs::remove_dir_all(d) {
                eprintln!("  ⚠️  Failed to remove {}: {}", d.display(), e);
            } else {
                println!("  ✗ Removed {}/", d.display());
            }
        }
    }

    // Reload systemd after removing the service file
    let _ = Command::new("systemctl")
        .args(["--user", "daemon-reload"])
        .output();

    // Refresh desktop database
    let apps_dir = home.join(".local/share/applications");
    let _ = Command::new("update-desktop-database")
        .arg(&apps_dir)
        .output();

    // Remove udev rules (requires sudo)
    for u in &udev_rules {
        if u.exists() {
            println!("\n🔒 Removing udev rules {} (requires sudo)...", u.display());
            let status = Command::new("sudo")
                .args(["rm", "-f"])
                .arg(u)
                .status();
            match status {
                Ok(s) if s.success() => {
                    println!("  ✗ Removed {}", u.display());
                }
                _ => eprintln!("  ⚠️  Failed to remove {} (try manually with sudo)", u.display()),
            }
        }
    }
    let _ = Command::new("sudo")
        .args(["udevadm", "control", "--reload-rules"])
        .output();
    let _ = Command::new("sudo")
        .args(["udevadm", "trigger"])
        .output();

    println!("\n✅ Bo-Ring has been completely uninstalled.\n");
}
