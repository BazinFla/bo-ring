#!/usr/bin/env bash
# ==============================================================================
# Bo-Ring - Automated Installer for Linux / Wayland
# Supports both pre-built binary (GitHub Release) and from-source compilation.
# ==============================================================================

set -e

GREEN='\033[0;32m'
CYAN='\033[0;36m'
YELLOW='\033[1;33m'
RED='\033[0;31m'
NC='\033[0m' # No Color

SCRIPT_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)"

echo -e "${CYAN}"
echo "  ____   ___        ____  ___ _   _  ____ "
echo " | __ ) / _ \      |  _ \|_ _| \ | |/ ___|"
echo " |  _ \| | | |_____| |_) || ||  \| | |  _ "
echo " | |_) | |_| |_____|  _ < | || |\  | |_| |"
echo " |____/ \___/      |_| \_\___|_| \_|\____|"
echo -e "${NC}"
echo -e "${CYAN}=== Bo-Ring Installer for Linux / Wayland ===${NC}\n"

# ──────────────────────────────────────────────
# 1. Locate or build the binary
# ──────────────────────────────────────────────
BINARY=""

if [ -f "$SCRIPT_DIR/Cargo.toml" ]; then
    # Source tree present — build from source
    if ! command -v cargo &> /dev/null; then
        echo -e "${YELLOW}⚠️  Rust and Cargo are not installed.${NC}"
        echo -e "Installing Rust via rustup..."
        curl --proto '=https' --tlsv1.2 -sSf https://sh.rustup.rs | sh -s -- -y
        source "$HOME/.cargo/env"
    fi
    echo -e "${GREEN}📦 Building / Updating Bo-Ring from source (Release)...${NC}"
    (cd "$SCRIPT_DIR" && cargo build --release)
    BINARY="$SCRIPT_DIR/target/release/bo-ring"
elif [ -f "$SCRIPT_DIR/bo-ring" ] && [ -x "$SCRIPT_DIR/bo-ring" ]; then
    # Pre-built binary found (standalone release archive without Cargo.toml)
    echo -e "${GREEN}📦 Pre-built binary detected.${NC}"
    BINARY="$SCRIPT_DIR/bo-ring"
elif [ -f "$SCRIPT_DIR/target/release/bo-ring" ]; then
    # Binary found
    BINARY="$SCRIPT_DIR/target/release/bo-ring"
else
    echo -e "${RED}❌ No binary found and no source tree available.${NC}"
    echo -e "${RED}   Download a release archive from:${NC}"
    echo -e "${RED}   https://github.com/BazinFla/bo-ring/releases${NC}"
    exit 1
fi

# ──────────────────────────────────────────────
# 2. Install binary to ~/.local/bin & /usr/local/bin
# ──────────────────────────────────────────────
INSTALL_DIR="$HOME/.local/bin"
mkdir -p "$INSTALL_DIR"

if command -v systemctl &> /dev/null; then
    systemctl --user stop bo-ring.service 2>/dev/null || true
fi
killall -9 bo-ring 2>/dev/null || true
sleep 0.2

echo -e "${GREEN}🚀 Installing binary to $INSTALL_DIR/bo-ring...${NC}"
install -m 755 "$BINARY" "$INSTALL_DIR/bo-ring"


# Symlink to /usr/local/bin so command 'bo-ring' works globally out of the box
if sudo -n true 2>/dev/null; then
    echo -e "${GREEN}🔗 Creating global command link /usr/local/bin/bo-ring...${NC}"
    sudo ln -sf "$INSTALL_DIR/bo-ring" /usr/local/bin/bo-ring 2>/dev/null || true
fi

# Install assets to XDG data path (~/.local/share/bo-ring/assets)
DATA_DIR="$HOME/.local/share/bo-ring"
mkdir -p "$DATA_DIR"
if [ -d "$SCRIPT_DIR/assets" ]; then
    cp -r "$SCRIPT_DIR/assets" "$DATA_DIR/"
    echo -e "${GREEN}   🖼️  Application assets deployed to $DATA_DIR/assets${NC}"
fi

# Ensure XDG config directory exists
CONFIG_DIR="$HOME/.config/bo-ring"
mkdir -p "$CONFIG_DIR"
if [ -f "$CONFIG_DIR/config.toml" ]; then
    echo -e "${CYAN}   📄 Existing config preserved at $CONFIG_DIR/config.toml${NC}"
else
    echo -e "${GREEN}   📄 Default localized configuration will be auto-generated at first launch in $CONFIG_DIR/${NC}"
fi

# Ensure ~/.local/bin is in PATH for shell configs if not present
for RC in "$HOME/.bashrc" "$HOME/.zshrc"; do
    if [ -f "$RC" ] && ! grep -q '\.local/bin' "$RC"; then
        echo 'export PATH="$HOME/.local/bin:$PATH"' >> "$RC"
    fi
done

# ──────────────────────────────────────────────
# 3. Configure udev rules for uinput & evdev
# ──────────────────────────────────────────────
echo -e "${GREEN}🔒 Configuring uinput / evdev permissions for Bluetooth & USB mice...${NC}"
TMP_RULE=$(mktemp)
cat << 'EOF' > "$TMP_RULE"
# udev rules for Bo-Ring (secure seat-scoped access to /dev/uinput, evdev and hidraw for Bluetooth & USB mice)
KERNEL=="uinput", SUBSYSTEM=="misc", OPTIONS+="static_node=uinput", TAG+="uaccess"
KERNEL=="event*", SUBSYSTEM=="input", GROUP="input", MODE="0660", TAG+="uaccess"
KERNEL=="hidraw*", SUBSYSTEM=="hidraw", GROUP="input", MODE="0660", TAG+="uaccess"
EOF

if [ -w "/etc/udev/rules.d" ]; then
    cp "$TMP_RULE" /etc/udev/rules.d/99-bo-ring-uinput.rules
    udevadm control --reload-rules && udevadm trigger || true
    echo -e "${GREEN}  ✓ udev rules installed to /etc/udev/rules.d/99-bo-ring-uinput.rules${NC}"
else
    echo -e "${CYAN}🔑 Requesting sudo permission to install udev rules for Bluetooth & USB mice...${NC}"
    if sudo cp "$TMP_RULE" /etc/udev/rules.d/99-bo-ring-uinput.rules 2>/dev/null; then
        sudo udevadm control --reload-rules && sudo udevadm trigger || true
        echo -e "${GREEN}  ✓ udev rules installed to /etc/udev/rules.d/99-bo-ring-uinput.rules${NC}"
    else
        echo -e "${YELLOW}⚠️  Could not install udev rules automatically (sudo password required).${NC}"
        echo -e "${YELLOW}👉 Run manually: sudo cp $TMP_RULE /etc/udev/rules.d/99-bo-ring-uinput.rules${NC}"
    fi
fi
rm -f "$TMP_RULE"

# Ensure user is in input group (fallback for non-logind sessions)
if ! groups | grep -q "\binput\b"; then
    echo -e "${YELLOW}👥 Adding user '$USER' to the 'input' group...${NC}"
    sudo usermod -aG input "$USER" 2>/dev/null || true
fi

# Immediate permission for current session without waiting for relogin
if command -v setfacl &> /dev/null; then
    sudo setfacl -m u:"$USER":rw /dev/uinput 2>/dev/null || true
    for dev in /dev/input/event*; do
        if udevadm info "$dev" 2>/dev/null | grep -E -q "ID_VENDOR_ID=046d|ID_INPUT_MOUSE=1"; then
            sudo setfacl -m u:"$USER":rw "$dev" 2>/dev/null || true
        fi
    done
fi
echo -e "${GREEN}✨ Access to input devices enabled for current session.${NC}"

# ──────────────────────────────────────────────
# 4. Deploy GNOME / XDG application desktop entry
# ──────────────────────────────────────────────
APPS_DIR="$HOME/.local/share/applications"
mkdir -p "$APPS_DIR"
cat << EOF > "$APPS_DIR/bo-ring.desktop"
[Desktop Entry]
Type=Application
Name=Bo-Ring
Comment=Bo-Ring GUI Configurator and Action Ring
Exec=$INSTALL_DIR/bo-ring
Icon=input-mouse
StartupWMClass=bo-ring
Terminal=false
Categories=Settings;HardwareSettings;Utility;
EOF

update-desktop-database "$APPS_DIR" &> /dev/null || true

# ──────────────────────────────────────────────
# 4.5. Deploy GNOME Shell Extension (bo-ring-window-tracker)
# ──────────────────────────────────────────────
if command -v gnome-shell &> /dev/null || [[ "$XDG_CURRENT_DESKTOP" == *"GNOME"* ]]; then
    echo -e "${GREEN}🧩 Deploying GNOME Shell Extension (bo-ring-window-tracker@flavien)...${NC}"
    EXT_DIR="$HOME/.local/share/gnome-shell/extensions/bo-ring-window-tracker@flavien"
    mkdir -p "$EXT_DIR"
    if [ -d "$SCRIPT_DIR/assets/gnome_extension" ]; then
        cp -r "$SCRIPT_DIR/assets/gnome_extension/"* "$EXT_DIR/"
        gnome-extensions enable bo-ring-window-tracker@flavien 2>/dev/null || true
        echo -e "${GREEN}  ✓ GNOME Shell extension deployed and enabled in $EXT_DIR${NC}"
    fi

fi

# ──────────────────────────────────────────────
# 4.6. Check & Install kdotool for KDE Plasma
# ──────────────────────────────────────────────
if [[ "$XDG_CURRENT_DESKTOP" == *"KDE"* ]] || [[ "$XDG_CURRENT_DESKTOP" == *"kde"* ]] || [ -n "$KDE_SESSION_VERSION" ]; then
    echo -e "${GREEN}🟢 KDE Plasma environment detected.${NC}"
    if ! command -v kdotool &> /dev/null; then
        echo -e "${YELLOW}⚠️  'kdotool' is required for instant active window detection under KDE Plasma Wayland.${NC}"
        echo -e "Attempting to install 'kdotool'..."
        if command -v dnf &> /dev/null; then
            sudo dnf install -y kdotool 2>/dev/null || true
        elif command -v apt-get &> /dev/null; then
            sudo apt-get install -y kdotool 2>/dev/null || true
        elif command -v pacman &> /dev/null; then
            sudo pacman -S --noconfirm kdotool 2>/dev/null || true
        elif command -v zypper &> /dev/null; then
            sudo zypper install -y kdotool 2>/dev/null || true
        fi

        if command -v kdotool &> /dev/null; then
            echo -e "${GREEN}  ✓ kdotool installed successfully!${NC}"
        else
            echo -e "${YELLOW}👉 Please install kdotool via your package manager (e.g., sudo dnf install kdotool / sudo apt install kdotool).${NC}"
        fi
    else
        echo -e "${GREEN}  ✓ kdotool is already installed.${NC}"
    fi
fi


# ──────────────────────────────────────────────
# 5. Configure & start systemd background daemon
# ──────────────────────────────────────────────

echo -e "${GREEN}⚙️  Configuring systemd background daemon...${NC}"
SERVICE_DIR="$HOME/.config/systemd/user"
mkdir -p "$SERVICE_DIR"

cat << EOF > "$SERVICE_DIR/bo-ring.service"
[Unit]
Description=Bo-Ring Remapping & Action Ring Daemon
After=graphical-session.target
StartLimitIntervalSec=0

[Service]
ExecStart=$INSTALL_DIR/bo-ring --daemon
Restart=on-failure
RestartSec=3

[Install]
WantedBy=graphical-session.target
EOF

if command -v systemctl &> /dev/null; then
    systemctl --user daemon-reload 2>/dev/null || true
    systemctl --user enable --now bo-ring.service 2>/dev/null || true
    echo -e "${GREEN}✨ Bo-Ring background daemon enabled and started successfully.${NC}"
fi

echo -e "\n${GREEN}✅ Bo-Ring installation completed successfully!${NC}"
echo -e "${CYAN}👉 Run 'bo-ring' to open the GUI configurator.${NC}"
echo -e "${CYAN}👉 Background daemon is active. Press your Action Ring button anytime!${NC}\n"
