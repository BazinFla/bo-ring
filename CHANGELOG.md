# Changelog

All notable changes to the **Bo-Ring** project will be documented in this file.

The format is based on [Keep a Changelog](https://keepachangelog.com/en/1.1.0/),
and this project adheres to [Semantic Versioning](https://semver.org/spec/v2.0.0.html).

---

## [0.2.0] - 2026-09-11

### Added
- **Native Logitech HID++ 2.0 Key Diversion (Autonomous Declarative Thumb Gesture Button Support)** :
  - Integrated native Logitech HID++ 2.0 protocol engine directly into Bo-Ring (`src/devices/hidpp.rs`), enabling 100% autonomous capture of the thumb rest gesture button without needing external utilities like Solaar or logiops.
  - **100% Declarative Profile-Driven Diversion** : Added optional `cid` string field on `DeviceButtonConfig` (e.g. `"cid": "0x00C3"`), allowing any mouse profile in `assets/devices/*.json` or user configuration to declare physical hardware Control IDs to divert dynamically without hardcoding values in Rust.
  - Automatic feature discovery on `/dev/hidraw*` querying `REPROG_CONTROLS_V4` (`0x1B04`) and `REPROG_CONTROLS` (`0x1B00`) across direct Bluetooth, USB cable, and wireless receivers.
  - Dynamic button diversion (*CidReporting*): diverts declared Control IDs (neutralizing the hardware-level `Ctrl + Alt + Tab` factory macro) and maps them cleanly to their configured button codes.
  - Dual-mode event dispatch: streams diverted button events directly into both the background daemon's Action Ring loop and the standalone Configuration UI (`attach_hidpp_gui_listener`).
  - Automatic recovery: re-applies hardware diversion transparently upon hotplug re-enumeration and system resume from sleep via D-Bus `PrepareForSleep` listener.
  - Updated `assets/devices/mx_master_2s.json` & `mx_master_4.json` profiles:
    - **MX Master 2S** : Thumb rest button set to `"cid": "0x00C3"`, standard `code: 278` (`btn_thumb_ring`), and SmartShift button set to `"cid": "0x00C4"`, unified `code: 280` (`btn_smartshift`), matching the MX Master series standard.
    - **MX Master 4** : Complete 6-button profile layout matching physical hardware: the haptic thumb rest button is assigned standard `code: 278` (`btn_haptic`), the gesture thumb button in the prolongation of Next/Back is assigned dedicated `code: 281` (`cid: "0x00C3"`, `btn_gestures`), and SmartShift is assigned unified `code: 280` (`cid: "0x00C4"`, `btn_smartshift`), completely resolving keycode collisions and ensuring seamless interoperability between profiles.
    - **New Official Logitech Profile Packs** : Added out-of-the-box profiles with Piper/libratbag vector illustrations for popular Logitech hardware:
      - **MX Master 3 & 3S** (`mx_master_3s.json`, `mx-master-3s.webp`): CIDs `0x00C3` (thumb rest `278`), `0x00C4` (SmartShift `280`), `046d:b023`, `046d:b034`, `046d:4082`.
      - **MX Anywhere 3 & 3S** (`mx_anywhere_3s.json`, `mx-anywhere-3s.webp`): CID `0x00C4` (SmartShift `280`), `046d:b025`, `046d:b02d`, `046d:b037`, `046d:b01a`.
      - **MX Vertical & Lift** (`mx_vertical.json`, `mx-vertical.webp`): Top DPI button `cid: "0x00FD"` (`280`), `046d:b020`, `046d:407b`, `046d:c08a`, `046d:b036`.
      - **M720 Triathlon** (`m720_triathlon.json`, `m720.webp`): Hidden thumb gesture button `cid: "0x00D0"` (`278`), `046d:b015`, `046d:405e`.
      - **MX Ergo Trackball** (`mx_ergo.json`, `mx-ergo.webp`): Precision button `cid: "0x00ED"` (`277`), `046d:b01d`, `046d:406f`.
- **Exact Hardware Matching via VID:PID (`match_ids`) & Hardware Priority** :
  - Extended `DeviceModelConfig` with `match_ids: Vec<String>` enabling exact hardware-level device identification (e.g. `046d:c548`, `046d:b034`).
  - Upgraded `DeviceRegistry::match_device` to prioritize exact VID:PID matches over heuristic string names, eliminating misidentification when multiple devices or wireless receivers are connected.
  - Automatic fallback to textual `match_names` when VID:PID is unknown or omitted.
  - Automatic inheritance of `match_ids` and `match_names` in `upsert_profile` ensuring existing user profiles without `match_ids` inherit standard identifiers from built-in templates without breaking.
  - Updated all built-in device profiles (`mx_master_4.json`, `mx_master_2s.json`, `generic.json`, `template.json`) with standard vendor/product hex IDs.
- **Active Click-Based Mouse Detection (`[ 👂 Detect with a click ]` & `🎯`)** :
  - Real-time hardware identity capture: evdev listeners and IPC daemon propagate physical device vendor/product IDs and device names alongside button events (`MouseInputEvent`).
  - Guarantees detection of the exact physical mouse that was clicked even when multiple mice are plugged in simultaneously.
  - Interactive profile creation modal (`render_new_device_modal`): added a dedicated VID:PID detection button that listens for a mouse click, displays detected hardware badges with 1-click removal (`✖`), and automatically pre-fills the profile name and ID slug from the hardware string.
  - Mode Editor toolbar quick-detect button (`🎯`): 1-click association of the active mouse's VID:PID to the active profile with auto-save, and right-click context menu to view and remove associated IDs.
- **Interactive Mouse Illustration Customization (`🖼️ Change Image`)** :
  - Added a dedicated floating button below `"+ New Button"` in the illustration canvas to select custom mouse artwork via a native file dialog (`rfd::FileDialog`).
  - Automatically copies custom images to `~/.config/bo-ring/devices/images/<profile_id>.<ext>`, updates the profile configuration, and instantly reloads the egui texture.
  - Added a 1-click reset button (`↺`) and right-click context menu option to restore the default factory illustration.
- **Dedicated Model Editor Mode (`mode_editor`) & Interactive Hardware Profiling** :
  - Decoupled button anchor calibration from `dev_mode` into a dedicated Model Editor Mode toggle.
  - Fast 1-click toggle button (`✏️`) in the Button Config tab header (`tab_buttons.rs`), exclusively managed from the Buttons page for streamlined access.
  - Anchor safety lock: anchors are non-draggable and securely locked when editor mode is disabled, preventing accidental layout disruptions during everyday usage.
  - Interactive calibration: drag-and-drop anchors in real-time with floating `[x, y]` coordinates, targeting crosshairs, and a 1-click `📋 Copy JSON` button.
  - **Evdev Physical Button Code Detection** : Capture physical mouse clicks in real time via evdev (`👂 Detect code`) in the Inspector, the Add Button modal, and the Edit Button modal, automatically assigning scancodes without manual entry.
  - **Interactive Button Addition Modal (`➕ New Button`)** : Accessible modal dialog to add new hardware buttons to any profile, configuring evdev code, display name, icon emoji with quick presets, and callout side, with anchors automatically placed at center `[0.5, 0.5]` for dragging.
  - **Comprehensive Button Edition Modal (`✏️ Edit this button`)** : Full edition dialog pre-filled with the button's code, name, icon, callout side, height ratio slider, and anchor coordinates with recenter option; accessible both from the inspector card and via a `✏️` icon directly on the canvas card.
  - **Drag-and-Drop Callout Handle (`✥`)** : Interactive drag handle icon on each callout card on the canvas allowing users to drag buttons up and down to adjust vertical height (`badge_y_ratio`), and smoothly drag across the center to flip between Left and Right columns (`ButtonCalloutSide`).
  - **In-Place Button Deletion (`🗑️`)** : Direct button deletion on each callout card with two-step inline confirmation (`✔️ Yes` / `❌ No`), safely preserving at least one button.
  - **Right Menu Streamlining** : Removed redundant button property inputs from the right inspector panel in Mode Editor, decluttering the view and reserving space for action assignment.
  - **Direct Profile Persistence (`💾 Save Profile`)** : 1-click save writes the modified device profile directly to `~/.config/bo-ring/devices/<id>.json` and automatically refreshes the device registry.
- **Declarative Device Profile System (JSON Packs) & Logitech MX Master 2S** :
  - Introduced open declarative JSON specification (`device.json`) allowing users to easily define custom mouse models with button anchors, callout positions, and custom images without recompiling.
  - Multi-tier zero-code discovery: local development tree (`assets/devices/`), XDG data path (`~/.local/share/bo-ring/assets/devices/`), system packs (`/usr/share/bo-ring/devices/`), and user directory (`~/.config/bo-ring/devices/<id>/`).
  - Built-in out-of-the-box profiles: Logitech MX Master 4 (`mx_master_4.json`), Logitech MX Master 2S (`mx_master_2s.json`), and Generic 5-button mouse (`generic.json`).
  - Hardware device registry (`DeviceRegistry`) with automatic evdev name pattern matching and specificity scoring.
  - Dynamic callout and badge rendering in the button configuration UI based on active profile button definitions.
  - Full-featured device profile selector directly in the Button Config tab header: `⚡ Automatic Detection` option with detected model name, profile source tags (`📦 Built-in`, `🖥️ System`, `👤 User`), inline `📂 Profiles Folder` and `🔄 Refresh` shortcuts, instant texture rendering and auto-save.
- **Developer Action (Secondary Sidebar)** : Added `dev_secondary_sidebar` (`Ctrl+Alt+B`) to developer action catalog with full trilingual support (EN, FR, ES).
- **Preset Optimization** : Updated VS Code profile preset to include the secondary sidebar toggle and optimized slot order.
- **Brand-Agnostic Capability-First Mouse Detection (`BTN_LEFT` & Generic Fallback)** :
  - Eliminated hardcoded vendor filtering (`logitech`, `mx `, `master`, `anywhere`) in daemon attachment (`scan_and_attach_devices` in `src/main.rs`) and UI device scanning (`scan_mouse_device` in `src/devices/mod.rs`).
  - Swapped name-pattern matching for primary evdev capability inspection: any input device exposing `evdev::Key::BTN_LEFT` without keyboard alphanumeric keys (`KEY_A`) is immediately recognized and monitored as a mouse candidate, ensuring universal support for Razer, SteelSeries, Corsair, trackballs, and generic mice.
  - Upgraded `enumerate_input_device_names()` to return `(PathBuf, String, Option<String>, bool)` where the 4th field is an `is_mouse` capability flag computed via `evdev::Device::open()` (with safe generic name fallback for devices locked exclusively by the daemon).
  - Completely removed the legacy fallback `"Logitech MX Master 4"`, delegating profile assignment directly to `DeviceRegistry::match_device()` with seamless fallback to `generic_mouse`.
- **System-Wide Service & Package Detection** :
  - Added detection for system-installed user systemd units (`/usr/lib/systemd/user/bo-ring.service`) alongside user overrides in `~/.config/systemd/user/`.
  - Added detection of system desktop launchers (`/usr/share/applications/bo-ring.desktop`) displaying a `🟢 System Package` badge.
  - Added multilingual localization keys across EN, FR, and ES for inactive service states and system package badges.

### Changed
- **Candidate Mouse Filtering** : Shifted input candidate detection in `main.rs` from restrictive string heuristics to evdev capability verification (`has_mouse_btn && !has_letter_keys`), preventing non-Logitech mice from being skipped before capability checks.
- **Ring Menu Responsiveness & Wayland Stabilization** :
  - Introduced a 200ms stabilization grace period before pointer nudge injection on overlay initialization, allowing GNOME Mutter to fully negotiate XDG toplevel placement, multi-monitor geometry, and maximization before locking the cursor position.
  - Separated `nudge_mouse` relative pulses (`+1` then `-1` separated by 10ms), preventing `libinput` from collapsing them to zero and guaranteeing a genuine `wl_pointer.motion` event across multiple monitors (1080p, 1440p).
  - Anchored click and release grace period (`is_open_cooldown`) to the visual appearance timestamp (`anim_start`), preventing phantom clicks or accidental early dismissals.
  - Reduced `hold_threshold` from `1000ms` to `250ms` for significantly snappier hold-to-release action execution.
- **Service Resilience** : Added `StartLimitIntervalSec=0` to the user `systemd` service unit definition (`bo-ring.service`) to permit continuous automatic restarts upon transient failures.
- **Settings UI State Parity** : Evaluates `systemctl --user is-enabled` alongside `is-active`, ensuring fresh DEB/RPM installs display "⚪ Inactive" with an "⚡ Enable" button instead of reporting the service as uninstalled.
- **Codebase Deduplication & Cleanup** :
  - Renamed `src/config/presets/demo.rs` to `default_config.rs` (`Config::default_config()`) to accurately reflect factory default configuration generation.
  - Removed duplicate `test_expand_path_arg` from `src/ring_menu/mod.rs` (consolidated in `src/utils/path.rs`).
  - Removed unused `pub use devices as device_manager;` re-export from `src/main.rs`.
  - Simplified legacy single-element loops in `uninstall.sh` into direct commands.
- **Settings Tab Consolidation** : Removed the redundant "Mouse Model & Hardware Profiles" card from `tab_settings.rs`. All device management controls (profile selector with auto-detection, open folder, refresh, editor mode toggle) are now exclusively centralized in the Button Config tab header (`tab_buttons.rs`), eliminating UI duplication.
- **Complete i18n Migration** : Migrated all remaining hardcoded UI strings across `tab_settings.rs`, `tab_buttons.rs`, `tab_ring.rs`, `tab_debug.rs`, and `mod.rs` into `locales/en.json`, `fr.json`, and `es.json` (35+ new keys). Zero hardcoded strings remain in the entire configuration UI. Strict parity enforced by `test_locales_key_parity`.
- **Language Detection Performance** : Removed per-frame `detect_system_language()` call from `render.rs`; the `lang` string is now resolved once and passed to `RingMenuApp`, eliminating redundant filesystem access every render cycle.

### Fixed
- **Monotonic Aura Animation Clock (`Instant` vs `SystemTime`)** :
  - Swapped non-monotonic `std::time::SystemTime` for a monotonic `std::time::Instant` reference (`ANIM_BASE_TIME`) in `src/ring_menu/render.rs` (`render_glowing_circle`), ensuring perfectly smooth breathing pulse oscillation immune to NTP adjustments or system clock shifts.
- **Safe Process Termination & POSIX Broadcast Prevention (`safe_kill` & `terminate_ring_process`)** :
  - Eliminated raw `unsafe { libc::kill(...) }` blocks in `src/main.rs`, achieving 100% safe Rust across the daemon core.
  - Introduced `safe_kill(pid, sig)` in `src/platform/overlay.rs` enforcing strictly positive PIDs (`pid > 0`), preventing catastrophic POSIX signal broadcasts to calling process groups (`pid == 0`) or all permitted user processes (`pid == -1`).
  - Added `terminate_ring_process(pid)` combining positive PID validation with `/proc/{pid}/cmdline` inspection (`is_pid_boring`), fully immunizing both `HideRing` IPC and `toggle_ring_menu` against OS PID recycling.
- **Silent File I/O Error Elimination** :
  - Replaced silent `let _ = ...` error discarding with explicit `eprintln!` logging across critical disk writes and directory creations: configuration migration and default generation (`src/config/mod.rs`), overlay PID tracking (`src/ring_menu/mod.rs`), and custom device profile JSON and image saving (`src/config_ui/types.rs`).
- **Strict Localization Parity (`locales/*.json`)** :
  - Centralized all user-facing desktop labels and detection placeholders into `locales/en.json`, `locales/fr.json`, and `locales/es.json` under keys `"desktop"` and `"detecting"`.
  - Replaced hardcoded string literals in `src/ring_menu/state.rs` with `crate::utils::i18n::tr(lang, "...")` calls and added unit tests verifying proper translation across French, English, and Spanish.
- **IPC Infinite Loop Prevention** : Replaced `.lines().flatten()` with `.lines().map_while(Result::ok)` across `main.rs`, `devices/mod.rs`, and `state.rs`, preventing daemon reader threads from spinning indefinitely when the IPC pipe is closed or errors occur.
- **Desktop Detection Over Wallpaper When Windows Are Open** :
  - Fixed an issue where clicking on the empty desktop wallpaper while applications were open in the background would incorrectly load the focused window's profile instead of the Desktop general ring.
  - Decoupled `GetWindowUnderCursor` (primary) from `GetActiveWindow` (fallback) in `detect_gnome_window()`: when `GetWindowUnderCursor` detects the cursor is over the desktop wallpaper (`Global / Desktop`, `Global / Bureau` on French GNOME, `gnome-shell` panel, or `ding`), it immediately returns `"Global / Desktop"` without querying `GetActiveWindow`.
  - Propagates `"Global / Desktop"` directly via `--app` from the daemon to the overlay, correctly displaying `🌐 General Menu (Desktop)` instantly without redundant background polling.
- **GNOME Active Window Detection & `SIGCHLD` `ECHILD` Resolution** :
  - Fixed a critical regression where `std::process::Command::output()` failed across the entire background daemon with `ECHILD (os error 10: No child processes)`. Removed `libc::signal(libc::SIGCHLD, libc::SIG_IGN)` and implemented clean asynchronous thread-based child process reaping (`child.wait()`) on spawned overlays.
  - Restored 100% reliable DBus queries (`gdbus`) to the GNOME Shell extension, ensuring per-application ring profiles (VS Code / Antigravity IDE, Firefox, Brave, etc.) are detected and loaded instead of always falling back to the default general menu.
- **Remapped Button Event Absorption & "Back" Navigation Bug Elimination (`dev.grab` & Passthrough)** :
  - Restored exclusive `dev.grab()` on mouse devices while maintaining native responsiveness: mouse pointer movements (`REL_X`, `REL_Y`), scroll wheels, and primary clicks (`BTN_LEFT`, `BTN_RIGHT`) are immediately forwarded with zero latency via `uinput`.
  - Configured extra buttons (e.g. Button 278 / `BTN_BACK` mapped to `ShowRingMenu` on Logitech MX Master 4) are now fully absorbed by Bo-Ring, completely eliminating unintended OS event bleed-through (such as browser history "Back" navigation upon opening the ring).
  - Unmapped extra buttons continue to be safely passed through to the system.
- **Wayland Multi-Monitor Action Ring Centering** :
  - Fixed an issue where the ring menu would spawn offset or misaligned on secondary or high-resolution monitors (e.g. 1440p alongside 1080p).
- **Accidental Trigger Prevention (Open Cooldown)** : Introduced a `100ms` cooldown grace period immediately after opening the ring menu, preventing phantom mouse clicks or premature button releases from inadvertently activating slots.
- **Stationary Pointer Fallback** : Generalized the stationary pointer detection fallback across all Wayland environments (locks onto raw pointer position after 50ms; 200ms fallback to screen center if events are withheld).
- **Deadzone Precision** : Circle hover detection now strictly enforces central deadzone boundaries, preventing false hover states near the cancel slot ("✕").

### Removed
- Legacy `mxmap` references and compatibility aliases (`mxmap-app`, `mxprofile` file filters) across the configuration UI and file dialogs.
- Unused `demo_*` action aliases, `load_mouse_texture()` helper, `RingAnimation::label()` method, and `is_gnome_menu_installed()` check.
- Developer-only hardcoded constant `"0x400003"` replaced by proper device registry lookups.
- X11 bypass workaround in `main.rs` replaced by unified `active_window::detect_active_window()` abstraction.

---

## [0.1.4] - 2026-08-19

### Added
- **Official "Bo-Ring" Rebranding** : Formally renamed project to **Bo-Ring** ("Button Overlay Ring"), updating DBus interfaces (`org.boring.WindowTracker`), GNOME extension (`bo-ring-window-tracker@flavien`), user systemd service units (`bo-ring.service`), and secure udev rules (`99-bo-ring-uinput.rules`).
- **GNOME Shell Window Focus Under Cursor (`FocusWindowUnderCursor`)** : DBus method integrated into the GNOME Shell tracker extension and automatically dispatched before simulated keyboard events, ensuring the hovered window receives input even if not focused beforehand.
- **Direct Daemon Action Execution (`execute_action_direct`)** : Bypasses redundant Unix domain socket loopbacks when the daemon initiates action dispatching, ensuring direct and instant targeting.
- **Modular Disk Configuration** : Reorganized storage under `~/.config/bo-ring/` into distinct directories (`rings/*.ring.toml` and `actions/*.action.toml`) with automatic migration from unified configuration files.
- **Modular Source Architecture (`src/config/`)** :
  - `src/config/actions/` : Categorized action catalogs (`media.rs`, `system.rs`, `developer.rs`, `ai.rs`).
  - `src/config/presets/` : Built-in factory application profiles (`browser.rs`, `code.rs`, `files.rs`, `blender.rs`, `gimp.rs`, `demo.rs`).
- **Dynamic Version Indicator** : Settings tab version badge dynamically bound to `env!("CARGO_PKG_VERSION")`.

### Changed
- **Rust 2024 Edition Migration** : Modernized the codebase to target the Rust 2024 edition (`edition = "2024"` in `Cargo.toml`) and adapted borrow pattern matching rules.
- **Visual Polish** : Removed blue link line connecting parent categories to child rings for a cleaner visual appearance.

---

## [0.1.3] - 2026-08-13

### Added
- **Precise Cursor Hover & Desktop Tracking** : Native GNOME Shell Mutter/Clutter extension (`bo-ring-window-tracker@flavien`) tracking the exact window under cursor (`GetWindowUnderCursor`) and distinguishing desktop surface backgrounds with 100% reliability.
- **1-Click Extension Management** : Integrated extension installation and updates directly in `install.sh` and through the GUI Settings tab (`tab_settings.rs`).
- **Developer Debug Tab** : Diagnostic tab in the GUI (`tab_debug.rs`) available in Developer Mode for live window detection benchmarking, cursor nudge simulation, and system log copying.
- **Multi-ID Application Matching** : Support for semicolon-separated window classes within a single application profile (e.g., `"firefox; brave; brave-browser; google-chrome; chromium"`).
- **Collision Prevention** : Real-time validation preventing duplicate application IDs during profile creation and editing.
- **KDE Plasma Integration** : Dedicated modular backend (`src/platform/kde.rs`) utilizing `kdotool` (1ms response) with `xprop` fallback, plus automatic distribution package installation in `install.sh`.
- **IPC Unix Socket Delegation** : Action execution from the Ring Menu delegated via Unix domain socket to the persistent background daemon, preserving Wayland window focus and keyboard simulation fidelity.

### Changed
- **Center Slot Simplification** : Replaced the temporary center Home ("🏠") button with a universal Close ("✕") across all menus and profiles.

---

## [0.1.2] - 2026-08-11

### Added
- **Modular Hardware Engine (`src/devices/`)** : Hardware abstraction layer based on `DeviceProfile` and `ButtonSpec` traits, decoupling input device logic (Logitech MX Master 4 and generic fallback mice).
- **Modular Platform Abstraction (`src/platform/`)** : Clean platform layer isolating window detection (`window.rs`), system accent color extraction (`accent_color.rs`), overlay window management (`overlay.rs`), systemd autostart (`autostart.rs`), and `uinput` virtual event synthesis (`input_emitter.rs`).
- **Decoupled Radial Menu Subsystem (`src/ring_menu/`)** : Separated circular slot trigonometry (`geometry.rs`), egui rendering pipeline (`render.rs`), and interaction state management (`state.rs`).

---

## [0.1.1] - 2026-08-10

### Added
- **Hold-to-Release Trigger Mode** : Hold the physical gesture button, hover over the target slot, and release to immediately trigger the action.
- **Scroll Wheel Navigation (Wheel Nav)** : Rotate the mouse wheel to cycle through ring slots, middle-click to select or enter submenus, and right-click to return.
- **Full Configuration Import & Export** : System configuration backup and restore (`.boring-cfg.toml`, `.boring-cfg.json`) via native file dialogs (`rfd`).
- **Contextual Application Profiles** : Create, edit, and export specialized radial menus (`.boring-app.json` / `.boring-app.toml`) with per-slot embedded multilingual translations.
- **Cross-Compositor Window Detection** : Universal active window detection supporting GNOME Shell (DBus), KDE Plasma (KWin DBus / kdotool), Hyprland (`hyprctl`), Sway (`swaymsg`), and X11 (`xprop`).
- **GUI Profile Manager** : Logi Options+ inspired profile selector with live previews and slot editing.

---

## [0.1.0] - 2026-08-09

### Added
- **Kernel Input Daemon & Exclusive Grab (`EVIOCGRAB`)** : Low-level mouse event interception via `evdev` with exclusive `EVIOCGRAB` capturing on configured buttons (preventing raw event leakage to the host desktop) and transparent `uinput` virtual event synthesis for unmapped buttons.
- **Interactive MX Master 4 Diagram & Visual Anchors** : High-definition WebP mouse diagram in Tab 1 with dynamic visual anchor lines targeting all 6 configurable physical inputs:
  - `Action Ring` (Thumb Rest - Code 278)
  - `Gesture Button` (Thumb Grip - Code 277)
  - `Back Button` (Side Back - Code 276)
  - `Forward Button` (Side Forward - Code 275)
  - `Middle Button` (Wheel Click - Code 274)
  - `Thumb Wheel` (Code 999)
- **Hardware Anti-Conflict Safety** : Strict exclusion of Left Click (`272`) and Right Click (`273`) to ensure desktop navigation is never compromised during button reassignment.
- **Interactive Button Sniffer & Live Key Recorder** : Real-time event detection with neon cyan highlighting (`#00d7af`) and an interactive shortcut recorder (`🔴 PRESS YOUR KEYS...`) capturing raw physical key combinations (`Ctrl`, `Alt`, `Shift`, `Super` + alphanumeric keys, F1-F12, navigation).
- **Dynamic Radial Action Ring & Visual Engine** :
  - Smooth, transparent `egui` overlay supporting 8 to 16 dynamic slots with recursive nested category sub-menus (arc bubbles).
  - Configurable upper slot limit (`max_slots` in Developer Mode) and auto-radius scaling to eliminate visual overlap.
  - Side inspector panel with vertical `ScrollArea` for deep nested slot management.
  - Automatic text contrast calculation (dynamic black or white text) based on slot background colors for optimal legibility.
- **Versatile Action Execution** : Shell command execution (`💻 Command / Application`), complex key combinations (`KeyCombo`), preset system actions (Media, Zoom, Navigation, Desktop, Lock), and folder shortcuts.
- **Live Demonstrator & Overlay CLI Flag** : `▶️ TEST ACTION RING IN REAL TIME` button in GUI and `--ring` direct execution CLI flag for immediate transparent viewport testing without physical mouse triggers.
- **Icon Selector & `TextureCache`** : 26 built-in emojis and support for real graphic images (`.png`, `.webp`, `.jpg`, `.jpeg`) decoded and smoothly cached via `TextureCache`.
- **Dynamic Hot-Plug Event Listener (`attach_new_mouse_listeners`)** : Seamless hot-plug switching between Bolt USB receiver and direct Bluetooth Low Energy (`0005:` uhid), auto-attaching new `evdev` reader threads without requiring app or daemon restarts.
- **Hybrid System Startup, Desktop Integration & Diagnostics** :
  - User `systemd` service (`systemctl --user`) with live active/inactive status detection and 1-click controls.
  - Dynamic generation and removal of XDG autostart desktop entry (`~/.config/autostart/bo-ring.desktop`).
  - Desktop application launcher integration (`bo-ring.desktop`, `app_id` & `StartupWMClass`).
  - Visual permission diagnostics inside the Settings tab verifying `/dev/uinput` accessibility and executable binary paths.
- **Multilingual Support (i18n)** : Complete user interface translations for English, French, and Spanish.
- **Cross-Compositor Support** : Native transparent overlay window rendering under Wayland (GNOME Shell, KDE Plasma) and X11 with multi-monitor workspace awareness.
