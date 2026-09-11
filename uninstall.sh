#!/usr/bin/env bash
# ==============================================================================
# Bo-Ring - Uninstaller
# Removes all files installed by install.sh
# ==============================================================================

set -e

GREEN='\033[0;32m'
CYAN='\033[0;36m'
YELLOW='\033[1;33m'
RED='\033[0;31m'
NC='\033[0m'

echo -e "${CYAN}"
echo "  ____   ___        ____  ___ _   _  ____ "
echo " | __ ) / _ \      |  _ \|_ _| \ | |/ ___|"
echo " |  _ \| | | |_____| |_) || ||  \| | |  _ "
echo " | |_) | |_| |_____|  _ < | || |\  | |_| |"
echo " |____/ \___/      |_| \_\___|_| \_|\____|"
echo -e "${NC}"
echo -e "${RED}=== Bo-Ring Uninstaller ===${NC}\n"

# ──────────────────────────────────────────────
# Confirmation prompt
# ──────────────────────────────────────────────
echo -e "${YELLOW}This will remove Bo-Ring and all its configuration files.${NC}"
echo -e "${YELLOW}The following files and directories will be deleted:${NC}\n"
echo "  • ~/.local/bin/bo-ring"
echo "  • ~/.config/bo-ring/"
echo "  • ~/.local/share/bo-ring/"
echo "  • ~/.local/share/gnome-shell/extensions/bo-ring-window-tracker@flavien/"
echo "  • ~/.config/systemd/user/bo-ring.service"
echo "  • ~/.config/autostart/bo-ring.desktop"
echo "  • ~/.local/share/applications/bo-ring.desktop"
echo "  • /etc/udev/rules.d/99-bo-ring-uinput.rules"
echo ""

read -r -p "$(echo -e "${RED}Proceed with uninstallation? [y/N] ${NC}")" confirm
if [[ ! "$confirm" =~ ^[Yy]$ ]]; then
    echo -e "${CYAN}Uninstallation cancelled.${NC}"
    exit 0
fi

# ──────────────────────────────────────────────
# 1. Stop and disable systemd service
# ──────────────────────────────────────────────
if systemctl --user is-active bo-ring.service &> /dev/null; then
    echo -e "${GREEN}⏹️  Stopping bo-ring.service daemon...${NC}"
    systemctl --user stop bo-ring.service 2>/dev/null || true
fi

if systemctl --user is-enabled bo-ring.service &> /dev/null; then
    echo -e "${GREEN}🔌 Disabling bo-ring.service...${NC}"
    systemctl --user disable bo-ring.service 2>/dev/null || true
fi

# ──────────────────────────────────────────────
# 2. Remove installed files
# ──────────────────────────────────────────────
remove_file() {
    if [ -f "$1" ]; then
        rm -f "$1"
        echo -e "${GREEN}   ✗ Removed $1${NC}"
    fi
}

remove_dir() {
    if [ -d "$1" ]; then
        rm -rf "$1"
        echo -e "${GREEN}   ✗ Removed $1/${NC}"
    fi
}

echo -e "${GREEN}🗑️  Removing Bo-Ring files...${NC}"

# Binaries
remove_file "$HOME/.local/bin/bo-ring"
symlink="/usr/local/bin/bo-ring"
if [ -f "$symlink" ] || [ -L "$symlink" ]; then
    sudo rm -f "$symlink" 2>/dev/null || rm -f "$symlink" 2>/dev/null || true
    echo -e "${GREEN}   ✗ Removed $symlink${NC}"
fi

# Configuration & Data directories
remove_dir "$HOME/.config/bo-ring"
remove_dir "$HOME/.local/share/bo-ring"

# GNOME Shell extension
if command -v gnome-extensions &> /dev/null; then
    gnome-extensions disable bo-ring-window-tracker@flavien 2>/dev/null || true
fi
remove_dir "$HOME/.local/share/gnome-shell/extensions/bo-ring-window-tracker@flavien"

# Systemd service files
remove_file "$HOME/.config/systemd/user/bo-ring.service"
systemctl --user daemon-reload 2>/dev/null || true

# XDG autostart
remove_file "$HOME/.config/autostart/bo-ring.desktop"

# GNOME application menu entries
remove_file "$HOME/.local/share/applications/bo-ring.desktop"
update-desktop-database "$HOME/.local/share/applications" &> /dev/null || true

# ──────────────────────────────────────────────
# 3. Remove udev rules (requires sudo)
# ──────────────────────────────────────────────
UDEV_RULE="/etc/udev/rules.d/99-bo-ring-uinput.rules"
if [ -f "$UDEV_RULE" ]; then
    echo -e "${YELLOW}🔒 Removing udev rules $UDEV_RULE (requires sudo)...${NC}"
    sudo rm -f "$UDEV_RULE" && echo -e "${GREEN}   ✗ Removed $UDEV_RULE${NC}"
fi
sudo udevadm control --reload-rules && sudo udevadm trigger 2>/dev/null || true

echo -e "\n${GREEN}✅ Bo-Ring has been completely uninstalled.${NC}\n"
