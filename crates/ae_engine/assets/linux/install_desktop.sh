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
XDG_DATA_HOME="${XDG_DATA_HOME:-${HOME}/.local/share}"
ICON_DEST_BASE="${XDG_DATA_HOME}/icons/hicolor"
DESKTOP_DEST_DIR="${XDG_DATA_HOME}/applications"

echo "╔══════════════════════════════════════════════════════════════╗"
echo "║      Aeon Engine - Universal Linux Desktop Installer         ║"
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
mkdir -p "$DESKTOP_DEST_DIR"

# Install icon to hicolor theme in standard FreeDesktop resolutions
SIZES=("16x16" "22x22" "24x24" "32x32" "48x48" "64x64" "128x128" "256x256" "512x512")
for SIZE in "${SIZES[@]}"; do
    TARGET_DIR="${ICON_DEST_BASE}/${SIZE}/apps"
    mkdir -p "$TARGET_DIR"
    if command -v magick &> /dev/null; then
        echo "📦 Installing ${SIZE} icon: ${TARGET_DIR}/${ICON_NAME}.png"
        magick "$ICON_SOURCE" -resize "$SIZE" "${TARGET_DIR}/${ICON_NAME}.png"
    elif command -v convert &> /dev/null; then
        echo "📦 Installing ${SIZE} icon: ${TARGET_DIR}/${ICON_NAME}.png"
        convert "$ICON_SOURCE" -resize "$SIZE" "${TARGET_DIR}/${ICON_NAME}.png"
    else
        echo "📦 Installing icon: ${TARGET_DIR}/${ICON_NAME}.png"
        cp "$ICON_SOURCE" "${TARGET_DIR}/${ICON_NAME}.png"
    fi
done

# Install .desktop file
echo "📦 Installing desktop entry: ${DESKTOP_DEST_DIR}/com.aeengine.Editor.desktop"
cp "$DESKTOP_SOURCE" "${DESKTOP_DEST_DIR}/com.aeengine.Editor.desktop"
chmod +x "${DESKTOP_DEST_DIR}/com.aeengine.Editor.desktop"

# Refresh system-wide FreeDesktop / XDG icon and desktop caches
echo "🔄 Updating FreeDesktop icon and desktop databases..."

if command -v xdg-icon-resource &> /dev/null; then
    xdg-icon-resource forceupdate 2>/dev/null || true
fi

if command -v gtk-update-icon-cache &> /dev/null; then
    gtk-update-icon-cache -f -t "${ICON_DEST_BASE}" 2>/dev/null || true
fi

if command -v update-desktop-database &> /dev/null; then
    update-desktop-database "${DESKTOP_DEST_DIR}" 2>/dev/null || true
fi

echo ""
echo "✅ Installation complete!"
echo ""
echo "ℹ️  Universal FreeDesktop integration active across all Linux Desktop Environments."
echo "   (GNOME, KDE Plasma, XFCE, Cinnamon, MATE, LXQt, Hyprland, Sway, COSMIC, etc.)"

