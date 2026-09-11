<p align="center">
  🇬🇧 <strong>English</strong> | 🇫🇷 <a href="docs/README.fr.md">Français</a>
</p>

# Bo-Ring

A radial menu overlay and mouse button remapper for Linux (Wayland & X11).

<p align="center">
  <a href="https://www.rust-lang.org/"><img src="https://img.shields.io/badge/Language-Rust_2024-orange.svg?style=flat-square&logo=rust" alt="Rust 2024" /></a>
  <a href="Cargo.toml"><img src="https://img.shields.io/badge/Version-0.2.0-blue.svg?style=flat-square" alt="Version 0.2.0" /></a>
  <a href="https://kernel.org"><img src="https://img.shields.io/badge/Platform-Linux%20Wayland%20%7C%20X11-blue.svg?style=flat-square&logo=linux" alt="Linux" /></a>
  <a href="https://github.com/emilk/egui"><img src="https://img.shields.io/badge/GUI-egui%200.28-purple.svg?style=flat-square" alt="egui" /></a>
  <a href="#internationalization"><img src="https://img.shields.io/badge/i18n-EN%20%7C%20FR%20%7C%20ES-brightgreen.svg?style=flat-square" alt="Languages" /></a>
  <a href="LICENSE"><img src="https://img.shields.io/badge/License-MIT-green.svg?style=flat-square" alt="License MIT" /></a>
</p>

<p align="center">
  <img src="assets/img/buttons.png" alt="Bo-Ring Button Remapper" width="49%" />
  <img src="assets/img/ring.png" alt="Bo-Ring Radial Action Ring" width="49%" />
</p>

---

## Overview

**Bo-Ring** is a Linux utility that provides customizable radial shortcut menus (**Action Rings**) and mouse button remapping.

Pressing a configured mouse button displays a radial overlay centered around your cursor. The menu automatically detects the active window to display contextual shortcuts (for web browsers, code editors, 3D software, or file managers).

Bo-Ring includes built-in hardware profiles for popular Logitech devices (**MX Master 4**, **MX Master 3 / 3S**, **MX Master 2S**, **MX Anywhere 3 / 3S**, **MX Vertical / Lift**, **M720 Triathlon**, **MX Ergo**, and **Generic 5-Button mice**). An integrated visual editor also allows you to calibrate and configure any other mouse supported by the Linux kernel (`evdev`).

The application runs in user space using standard Linux input subsystems (`evdev` and `uinput`) and is built with Rust and `egui`.

---

## Features

- **Contextual Radial Menus**: Configurable slot count (up to 16 by default) with support for nested submenus, custom icons or emojis, and desktop accent color matching.
- **Application Profiles**: Automatically switches menus based on the focused window (Firefox, VS Code, GIMP, Blender, etc.).
- **Mouse Button Remapping**: Map buttons detected via `evdev` (side buttons, thumb rest, DPI buttons) to radial menus, keyboard shortcuts, shell commands, or media actions.
- **Visual Model Editor**: Visually position button anchors on mouse illustrations, capture button scancodes by clicking, and add or adjust buttons directly in the GUI.
- **Custom Device Profiles**: Mouse definitions are stored as JSON files (`~/.config/bo-ring/devices/` or `/usr/share/bo-ring/devices/`) with custom artwork support.
- **Hardware Identification**: Identifies connected devices by USB or Bluetooth vendor and product IDs (VID:PID).
- **Trigger Modes**:
  - **`Hybrid` (Default)**: A quick click keeps the menu open; holding the button triggers the selected action upon release.
  - **`Click`**: Click to open, click a slot to trigger, click the center to dismiss.
  - **`HoldToRelease`**: Hold the button, hover over an action, and release to execute.
- **Scroll Wheel Navigation**: Scroll to cycle through slots, middle-click to select, and right-click to return from submenus.
- **Live Configuration Reload**: Settings modified in the GUI apply immediately to the running background service without restarting.
- **Keyboard Layout Support**: Supports AZERTY and QWERTY layouts for key sequence simulation.
- **Battery & Status Reporting**: Displays battery level and connection status for supported Logitech (HID++) and Bluetooth Low Energy devices.
- **Rootless Operation**: Runs entirely in user space through standard seat-scoped `udev` rules (`uaccess`).
- **Multilingual**: Available in English, French, and Spanish with automatic system locale detection.

---

## Compatibility

### Desktop Environments

Bo-Ring detects the active window across Wayland and X11 compositors:

| Environment | Mechanism | Notes |
| :--- | :--- | :--- |
| **GNOME Shell (Wayland & X11)** | D-Bus extension | Window-under-cursor tracking via `bo-ring-window-tracker@flavien` |
| **KDE Plasma 5 / 6 (Wayland & X11)** | `kdotool` / `xprop` | `kdotool` CLI with `xprop` fallback |
| **Hyprland** | Wayland IPC | `hyprctl activewindow` |
| **Sway / wlroots** | Wayland IPC | `swaymsg -t get_tree` |
| **X11 (Generic)** | EWMH (`_NET_ACTIVE_WINDOW`) | XFCE, MATE, i3, bspwm, etc. |

### Supported Hardware

Bo-Ring works with any mouse supported by the Linux kernel via `evdev`:

- **Device Support**: Any USB or Bluetooth mouse with extra buttons (forward, back, thumb buttons, DPI switches) can be recognized and remapped.
- **Built-in Profiles**:
  - **Logitech MX Master 4**: Haptic thumb button (`278`), Gesture thumb button (`281` / CID `0x00C3`), SmartShift (`280` / CID `0x00C4`), Side Forward/Back (`276`/`275`), Middle click (`274`).
  - **Logitech MX Master 3 & 3S**: Thumb rest gesture (`278` / CID `0x00C3`), SmartShift (`280` / CID `0x00C4`), Side Forward/Back (`276`/`275`), Middle click (`274`).
  - **Logitech MX Master 2S**: Thumb rest gesture (`278` / CID `0x00C3`), SmartShift (`280` / CID `0x00C4`), Side Forward/Back behind thumb wheel (`276`/`275`), Middle click (`274`).
  - **Logitech MX Anywhere 3 & 3S**: SmartShift mode button (`280` / CID `0x00C4`), Side Forward/Back (`276`/`275`), Middle click (`274`).
  - **Logitech MX Vertical & Lift**: Top DPI button (`280` / CID `0x00FD`), Side Forward/Back (`276`/`275`), Middle click (`274`).
  - **Logitech M720 Triathlon**: Hidden thumb gesture button (`278` / CID `0x00D0`), Side Forward/Back (`276`/`275`), Middle click (`274`).
  - **Logitech MX Ergo Trackball**: Precision mode button (`277` / CID `0x00ED`), Side Forward/Back (`276`/`275`), Middle click (`274`).
  - **Generic 5-Button Mouse**: Standard side forward (`276`), side back (`275`), and middle click (`274`).
- **Custom Devices**: Other mice (such as gaming, trackball, or vertical ergonomic models) can be configured and calibrated using the integrated visual model editor.

| Button / Scancode | Event Code | Default Mapping | Custom Remapping |
| :--- | :---: | :--- | :--- |
| **Thumb Rest (Gesture)** | `278` | Action Ring Overlay | Yes |
| **Thumb Grip Button** | `277` | Terminal (`gnome-terminal`) | Yes |
| **Side Forward (Next)** | `276` | Browser Forward | Yes |
| **Side Back (Previous)** | `275` | Browser Back | Yes |
| **Middle Click** | `274` | Middle Click | Yes |
| **Left & Right Click** | `272` / `273` | Primary / Secondary Click | Protected (cannot be unbound) |

---

## Installation

### Automated Install (Recommended)

The installation script supports both building from source and installing pre-built binaries, sets up `udev` permissions for rootless operation, and starts the user service:

```bash
git clone https://github.com/BazinFla/bo-ring.git
cd bo-ring
./install.sh
```

The script will:
1. Compile the release binary using `cargo` (or use pre-built binary if available).
2. Install udev rules (`/etc/udev/rules.d/99-bo-ring-uinput.rules`) allowing unprivileged user access to `/dev/uinput` and mouse devices.
3. Enable and start the user `systemd` service (`bo-ring.service`).
4. Install desktop entries for the application launcher.
5. Deploy the GNOME Shell window tracking extension (on GNOME) or verify `kdotool` (on KDE Plasma).

### Pre-Built Packages (.deb / .rpm / tar.gz)

Pre-compiled packages for Debian/Ubuntu, Fedora/RHEL, and standalone tarballs are available on the [GitHub Releases](https://github.com/BazinFla/bo-ring/releases) page.

- **Debian / Ubuntu**:
  ```bash
  sudo dpkg -i bo-ring_0.2.0_amd64.deb
  systemctl --user enable --now bo-ring.service
  ```
- **Fedora / RHEL**:
  ```bash
  sudo rpm -i bo-ring-0.2.0-1.x86_64.rpm
  systemctl --user enable --now bo-ring.service
  ```

### Manual Build

```bash
git clone https://github.com/BazinFla/bo-ring.git
cd bo-ring
cargo build --release
./target/release/bo-ring
```

### Note for GNOME Shell (Wayland)

On GNOME running Wayland, in-place reloading of extensions is not supported by the compositor. After installation:
1. **Restart your user session** (log out and log back in) so GNOME Shell discovers the extension.
2. **Verify that the extension is active**:
   ```bash
   gnome-extensions list --enabled | grep bo-ring
   ```
   If needed, enable it manually:
   ```bash
   gnome-extensions enable bo-ring-window-tracker@flavien
   ```
*(On GNOME X11, press `Alt`+`F2`, type `r`, and press Enter to reload the shell).*

---

## Usage

### Configuration GUI

Open the configuration panel from your desktop application launcher or run:

```bash
bo-ring
```

From the GUI, you can:
- Remap physical buttons and customize trigger modes.
- Calibrate button positions and define custom buttons for your mouse.
- Create and edit application-specific Action Rings.
- Adjust themes, accent colors, and animation styles.

### CLI Reference

| Command | Description |
| :--- | :--- |
| `bo-ring` | Launch the configuration GUI |
| `bo-ring --daemon` | Run the background event listener and remapping daemon |
| `bo-ring --ring` | Open the radial menu directly (useful for testing or custom keybindings) |
| `bo-ring --uninstall` | Launch the uninstaller |

### Managing the Background Service

The daemon runs as a standard user `systemd` service:

```bash
# Check service status
systemctl --user status bo-ring.service

# View daemon logs
journalctl --user -u bo-ring.service -f

# Restart daemon
systemctl --user restart bo-ring.service
```

---

## Configuration

Settings and custom profiles are stored in `~/.config/bo-ring/`:

```
~/.config/bo-ring/
├── config.toml           # Main settings (mappings, trigger mode, appearance)
├── rings/                # Per-application radial menu definitions (*.ring.toml)
├── actions/              # Reusable custom actions (*.action.toml)
└── devices/              # Custom mouse profiles (*.json) and images
```

### Example `config.toml`

```toml
[general]
device_name = "Logitech MX Master 4"
theme = "dark"
language = "system" # "system", "en", "fr", "es"
autostart_daemon = true

[buttons]
278 = { type = "ShowRingMenu" }                           # Thumb gesture button -> Action Ring
277 = { type = "Command", cmd = "gnome-terminal" }        # Thumb grip button -> Terminal
276 = { type = "KeyCombo", keys = ["CTRL", "ALT", "T"] } # Side forward button -> Custom shortcut

[ring_menu]
default_color = "#282C37"
default_active_color = "#00D7AF"
animation = "ScaleFade" # "ScaleFade" or "None"
trigger_mode = "Hybrid" # "Hybrid", "Click", or "HoldToRelease"
wheel_navigation = true
max_slots = 16
glow_effect = true
pulse_effect = true
```

---

## Internationalization
 
 Available translations:
 
 - 🇬🇧 **English** (`en`)
 - 🇫🇷 **Français** (`fr`)
 - 🇪🇸 **Español** (`es`)
 
 The language is detected automatically from your system environment (`LANG`) and can also be selected manually in the settings.

---

## Uninstallation

To remove Bo-Ring, systemd services, udev rules, and configuration files:

```bash
bo-ring --uninstall
# or from the repository directory:
./uninstall.sh
```

---

## Project Context & Feedback

Bo-Ring is a personal project maintained primarily for my own daily workflow. Testing is conducted with the hardware I have on hand (notably Logitech MX Master series mice) and mainly on **Fedora GNOME (latest release)**, with occasional testing on **Debian**.

While the application is designed to support other Linux distributions, desktop environments, and mouse models, behavior may vary depending on your specific setup. If you give it a try, any feedback, bug reports, or profiles for other mouse models are welcome via GitHub issues or pull requests.

---

## License

Bo-Ring is released under the [MIT License](LICENSE).

---

## Third-Party Trademarks

All product and company names (such as *Logitech*, *Firefox*, *Blender*, *GIMP*, *Visual Studio Code*) are trademarks or registered trademarks of their respective holders. Their use here is solely for identification and compatibility purposes and does not imply affiliation or endorsement.
