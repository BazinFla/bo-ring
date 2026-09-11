# 🤝 Contributing to Bo-Ring / Guide de Contribution

Welcome to the **Bo-Ring** project! Whether you are a **human developer** or an **AI assistant/agent** (Gemini, Claude, ChatGPT, Antigravity, etc.), this document outlines the architecture, design principles, coding standards, and step-by-step guidelines for modifying and extending the codebase.

---

## 📑 Table of Contents

- [🎯 Mission & Core Principles](#-mission--core-principles)
- [🗺️ Project Architecture & Structure](#️-project-architecture--structure)
- [🤖 Guidelines for AI Assistants & Autonomous Agents](#-guidelines-for-ai-assistants--autonomous-agents)
- [👥 Workflow for Human Contributors](#-workflow-for-human-contributors)
- [🎨 Coding Standards & Conventions](#-coding-standards--conventions)
- [🧪 Testing & Verification](#-testing--verification)
- [🛠️ Utility Scripts & Backups](#️-utility-scripts--backups)

---

## 🎯 Mission & Core Principles

**Bo-Ring** (Button Overlay Ring) is a high-performance, ultra-lightweight Action Ring radial overlay and hardware button remapper for mice and productivity input devices on Linux (Wayland & X11).

Key technical principles:
1. **Primary Target & Reference Platform**: Developed and tested primarily under **Fedora GNOME (Wayland)**.
2. **Strict Desktop & Distro Isolation**: Code, workarounds, or hacks specific to alternative environments (**KDE Plasma**, **Sway**, **Hyprland**, etc.) or distributions (**Debian**, **Arch Linux**, etc.) **MUST BE ISOLATED** in distinct modules (e.g. in `src/platform/` as modular backend adapters) to prevent cross-pollution or breaking other environments.
3. **Ultra-Low Overhead**: Idle memory usage under ~10 MB RAM, 0% CPU idle footprint, sub-millisecond input reaction.
4. **Rootless Operation**: Runs entirely in user space. System-level permissions (`/dev/uinput`, `/dev/input/event*`) are configured once via `udev` rules.
5. **Single Source of Truth (SSoT)**: All configuration settings reside cleanly in `Config` (`src/config.rs`) and serialize to `~/.config/bo-ring/config.toml`.
6. **Fluid Native UX**: Modern dark theme built with `egui`, with smooth scale/fade radial animations and automatic OS accent color synchronization.

---

## 🗺️ Project Architecture & Structure

The codebase is written in **Rust (2024 Edition)**. The directory layout and responsibility of each module are detailed below:

```
bo-ring/
├── Cargo.toml               # Package manifest and dependencies (evdev, egui, eframe, serde, toml)
├── Cargo.lock               # Exact dependency tree lockfile
├── README.md                 # User documentation and features overview
├── CHANGELOG.md              # Project changelog and release history
├── CONTRIBUTING.md           # Developer & AI guidelines (this file)
├── backup.sh                # Compressed source backup script
├── install.sh               # System installer (udev rules, binary, desktop entry)
├── uninstall.sh             # System uninstaller script
├── assets/                  # Application icons, graphics, and static resources
├── locales/                 # Internationalization JSON dictionaries
│   ├── en.json              # English localization
│   ├── fr.json              # French localization
│   └── es.json              # Spanish localization
├── devel/                   # Developer documentation and architectural design notes
│   ├── Roadmap.md           # Future milestones, backlog & distro validation matrix
│   └── architecture.md      # Architectural choices, udev security, R&D & dev notes
└── src/                     # Core Rust source code
    ├── main.rs              # Application entry point, CLI parser, daemon loop & GUI runner
    ├── config/              # SSoT configuration models, rings, actions, and presets
    │   ├── mod.rs           # Config loader/serializer, path resolver, migration
    │   ├── rings.rs         # AppProfileConfig, RingMenuConfig, slot models
    │   ├── actions/         # Modular named action catalog (media, system, dev, AI)
    │   └── presets/         # Built-in portable app presets (Blender, GIMP, Code, etc.)
    ├── devices/             # Device abstraction layer
    │   ├── mod.rs           # Device manager, battery scanner, event router
    │   ├── codes.rs         # Mouse button codes and Bluetooth aliases
    │   ├── profile.rs       # Button mapping and device profile trait
    │   ├── mx_master_4.rs   # Hardware-specific mapping for Logitech MX Master 4
    │   └── generic.rs       # Generic fallback device support
    ├── platform/            # Operating system and windowing platform integrations
    │   ├── mod.rs           # Platform module exports
    │   ├── overlay.rs       # Window overlay viewport setup
    │   ├── window.rs        # Active window detection (GNOME DBus / KDE / Hyprland / Sway / X11)
    │   ├── input_emitter.rs # uinput virtual keyboard/mouse event synthesis
    │   ├── accent_color.rs  # OS accent color sync (GNOME, KDE Plasma, XDG Portal)
    │   ├── autostart.rs     # Systemd user service & desktop autostart configuration
    │   ├── ipc.rs           # Daemon Unix domain socket IPC protocol
    │   └── uninstall.rs     # Interactive CLI uninstaller
    ├── ring_menu/           # Radial Action Ring overlay subsystem
    │   ├── mod.rs           # Ring menu subsystem interface
    │   ├── geometry.rs      # Circular slot trigonometric calculations (8 to 16 sectors)
    │   ├── state.rs         # Radial menu state machine (open/closed, levels, trigger modes)
    │   └── render.rs        # egui rendering pipeline (scale/fade, icons, glowing aura)
    ├── config_ui/           # egui Control Panel / Configurator GUI
    │   ├── mod.rs           # Main GUI window layout, tab management, and state sync
    │   ├── types.rs         # UI-specific messages, tab enums, and component state
    │   ├── widgets.rs       # Custom egui widgets (keybind recorder, color picker, drag handles)
    │   ├── tab_buttons.rs   # Hardware button remapper tab & live interactive button detector
    │   ├── tab_ring.rs      # Radial ring menu editor, hierarchy tree & live canvas preview
    │   ├── tab_settings.rs  # Application settings, autostart toggle, theme & accent sync
    │   └── tab_debug.rs     # Live input log & debug inspector
    └── utils/               # Common helper utilities
        ├── mod.rs           # Utility exports
        ├── color.rs         # Color conversions, luminance & auto-contrast math
        ├── i18n.rs          # Runtime internationalization dictionary manager
        ├── icon_loader.rs   # Icon loading, image processing (PNG/WebP), and emoji caching
        └── path.rs          # Home path expansion and asset resolution
```

---

## 🤖 Guidelines for AI Assistants & Autonomous Agents

If you are an AI assistant (such as Gemini, Claude, ChatGPT, or Antigravity) working on this codebase, **you MUST adhere to the following rules**:

### 1. 📦 Run Backup Before Major Edits
Before performing multi-file refactoring, structural redesigns, or high-risk modifications, **always run the backup script**:
```bash
../backup.sh
```
This generates a timestamped `.tar.gz` snapshot in `backups/`.

### 2. 🛡️ Maintain Single Source of Truth (SSoT)
- Do not introduce redundant or isolated state structs.
- All configuration attributes **must** originate in `src/config/`.
- If you add or modify a setting, update:
  1. `src/config/` (struct definition, default implementation, TOML serde attributes).
  2. The GUI tab (`src/config_ui/tab_*.rs`) exposing the setting.
  3. Localization files (`locales/*.json`).

### 3. 🧪 Always Verify Changes
Never conclude a turn or mark a task as done without running:
```bash
cargo check
cargo test
```
If compilation errors or test failures occur, inspect the full log output and resolve the root cause before completing your turn.

### 4. 🐧 Respect Wayland & Linux Constraints
- Do not rely on X11-only binaries (`xdotool`, `xprop`, `xwininfo`) without providing pure Linux / DBus / Wayland fallbacks.
- Keep `uinput` virtual key code mappings in `src/platform/input_emitter.rs` aligned with Linux kernel headers (`linux/input-event-codes.h`).

### 5. 🌐 Localize All User-Facing Text
- Never hardcode raw UI strings inside `src/config_ui/` components.
- Register new translation keys in `locales/en.json`, `locales/fr.json`, and `locales/es.json`.
- Access translations using `i18n::t("your_key")` or `i18n::t_fmt(...)`.

### 6. ⚡ Zero-Regression Performance Rule
- Avoid heap allocations inside hot input event loops (e.g., inside `evdev` event handling or overlay render frames).
- Keep animations light and reactive using frame deltas rather than heavy background polling loops.

### 7. 🧩 Desktop & Environment Isolation Rule
- The primary development target is **Fedora GNOME (Wayland)**.
- Any features, workarounds, or hacks specific to alternative environments (**KDE Plasma**, **Sway**, **Hyprland**) or distros (**Debian**, **Arch Linux**) **must be implemented in isolated, separate modules** (e.g., under `src/platform/`).
- Never inject environment-specific hacks into core files or standard Fedora GNOME modules, preventing cross-contamination and breaking other setups.

---

## 👥 Workflow for Human Contributors

### Prerequisites

Ensure you have the following installed on your system:
- **Rust Toolchain** (1.75+ recommended, 2021 edition support): `rustup default stable`
- **System Libraries**: `libevdev-dev`, `libudev-dev`, `pkg-config`, `wayland-client` (on Debian/Ubuntu/Fedora)
- **Linux Kernel Modules**: `uinput` enabled

## 🎨 Coding Standards & Conventions

### Rust Idioms & Safety
- **No Unhandled Unwraps**: Avoid `unwrap()` in non-test production code. Use pattern matching (`if let`, `match`), `unwrap_or_default()`, or context-aware `expect("reason")`.
- **Explicit Error Context**: Propagate errors gracefully using `Result<T, E>`.
- **Documentation**: Provide concise docstrings (`///`) on public structs, enums, and methods.

### UI & Layout Guidelines (`egui`)
- Follow `egui` best practices: frame containers, cohesive padding, responsive widget sizing.
- Respect system accent color when enabled (`src/platform/accent_color.rs`).
- Ensure contrast calculation utilities (`src/utils/color.rs`) are used whenever rendering custom color text on dynamic backgrounds.

### Desktop & Distro Architecture Isolation
- **Primary Reference**: Fedora GNOME on Wayland is the primary target environment.
- **Isolated Backends**: Specific hacks, IPC protocols, DBus calls, or platform workarounds required for alternative environments (**KDE Plasma**, **Sway**, **Hyprland**) or distros (**Debian**, **Arch Linux**) must be implemented in **separate, dedicated backend files** (e.g. `src/platform/kde.rs`, `src/platform/hyprland.rs`, `src/platform/sway.rs`).
- **Encapsulated Traits**: Abstract environment-specific behaviors behind clean Rust traits so that adding or fixing support for one compositor/desktop never pollutes core code or breaks Fedora GNOME support.

### Commit Conventions
We follow **Conventional Commits**:
- `feat(ring): add support for custom radial icons`
- `fix(uinput): correct key code mapping for media play/pause`
- `docs(readme): update installation instructions`
- `refactor(config): simplify profile serialization`
- `test(devices): add unit tests for mouse event parsing`

---

## 🧪 Testing & Verification

### Running Automated Unit Tests
```bash
cargo test
```
Tests are embedded in module files (`src/config.rs`, `src/ring_menu/geometry.rs`, `src/platform/input_emitter.rs`) inside `#[cfg(test)]` sub-modules. When creating new logic, always write matching unit tests.

### Testing the Application Locally
To test the GUI configurator:
```bash
cargo run
```

To test the background daemon directly:
```bash
cargo run -- --daemon
```

---

## 🛠️ Utility Scripts & Backups
 
- **`../backup.sh`**: Creates a full source archive (`backups/bo-ring_backup_YYYYMMDD_HHMMSS.tar.gz`) excluding `target/`, `.git/`, and temporary build artifacts.
- **`./install.sh`**: Installs udev rules, builds production release binaries, and sets up systemd user services.
- **`./uninstall.sh`**: Cleans up installed binaries, services, and udev rules.

---

*Thank you for contributing to Bo-Ring! Let's make mouse remapping and contextual radial menus on Linux smooth, fast, and delightful.* 🖱️⚡
