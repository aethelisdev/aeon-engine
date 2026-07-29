#!/bin/bash
# ============================================================================
# Aeon Engine - Linux Desktop Entry & Icon Installer
# ============================================================================
# This script installs the Aeon Engine .desktop entry and icon to the user's system.
# This installation is required for Wayland compositors (GNOME, KDE Plasma, Hyprland, Sway, etc.)
# to display the application window icon correctly.
#
# Under Wayland, window icons cannot be set directly on the window as in X11.
# Instead, the compositor matches the app_id against the .desktop entry
# and loads the icon defined in the Icon= field.
# ============================================================================

set -euo pipefail

SCRIPT_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)"
ICON_SOURCE="${SCRIPT_DIR}/../icon/aeicon.png"
DESKTOP_SOURCE="${SCRIPT_DIR}/com.aeengine.Editor.desktop"

ICON_NAME="com.aeengine.Editor"
ICON_DEST_DIR="${HOME}/.local/share/icons/hicolor/256x256/apps"
DESKTOP_DEST_DIR="${HOME}/.local/share/applications"

echo "╔══════════════════════════════════════════════════════════════╗"
echo "║          Aeon Engine - Linux Desktop Installer                ║"
echo "╚══════════════════════════════════════════════════════════════╝"
echo ""

# Validate source files
if [ ! -f "$ICON_SOURCE" ]; then
    echo "❌ Error: Icon file not found: $ICON_SOURCE"
    exit 1
fi

if [ ! -f "$DESKTOP_SOURCE" ]; then
    echo "❌ Error: Desktop file not found: $DESKTOP_SOURCE"
    exit 1
fi

# Create target directories
mkdir -p "$ICON_DEST_DIR"
mkdir -p "$DESKTOP_DEST_DIR"

# Install icon to hicolor theme
echo "📦 Installing icon to: ${ICON_DEST_DIR}/${ICON_NAME}.png"
cp "$ICON_SOURCE" "${ICON_DEST_DIR}/${ICON_NAME}.png"

# Install .desktop file
echo "📦 Installing desktop entry to: ${DESKTOP_DEST_DIR}/com.aeengine.Editor.desktop"
cp "$DESKTOP_SOURCE" "${DESKTOP_DEST_DIR}/com.aeengine.Editor.desktop"

# Update icon cache (if gtk-update-icon-cache is available)
if command -v gtk-update-icon-cache &> /dev/null; then
    echo "🔄 Updating icon cache..."
    gtk-update-icon-cache -f -t "${HOME}/.local/share/icons/hicolor" 2>/dev/null || true
fi

# Update desktop database (if update-desktop-database is available)
if command -v update-desktop-database &> /dev/null; then
    echo "🔄 Updating desktop database..."
    update-desktop-database "${DESKTOP_DEST_DIR}" 2>/dev/null || true
fi

echo ""
echo "✅ Installation complete!"
echo ""
echo "ℹ️  Notes:"
echo "   - The icon should now appear in Wayland compositors (GNOME, KDE, Hyprland, Sway, etc.)"
echo "   - You may need to log out and log back in for changes to take effect."
echo "   - To uninstall, run: ./uninstall_desktop.sh"

