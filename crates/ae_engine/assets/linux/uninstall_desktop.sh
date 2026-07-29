#!/bin/bash
# ============================================================================
# Aeon Engine - Linux Desktop Entry & Icon Uninstaller
# ============================================================================

set -euo pipefail

ICON_NAME="com.aeengine.Editor"
ICON_PATH="${HOME}/.local/share/icons/hicolor/256x256/apps/${ICON_NAME}.png"
DESKTOP_PATH="${HOME}/.local/share/applications/com.aeengine.Editor.desktop"

echo "╔══════════════════════════════════════════════════════════════╗"
echo "║          Aeon Engine - Linux Desktop Uninstaller              ║"
echo "╚══════════════════════════════════════════════════════════════╝"
echo ""

if [ -f "$ICON_PATH" ]; then
    echo "🗑️  Removing icon: $ICON_PATH"
    rm "$ICON_PATH"
else
    echo "ℹ️  Icon not found (already removed?): $ICON_PATH"
fi

if [ -f "$DESKTOP_PATH" ]; then
    echo "🗑️  Removing desktop entry: $DESKTOP_PATH"
    rm "$DESKTOP_PATH"
else
    echo "ℹ️  Desktop entry not found (already removed?): $DESKTOP_PATH"
fi

# Update icon cache
if command -v gtk-update-icon-cache &> /dev/null; then
    echo "🔄 Updating icon cache..."
    gtk-update-icon-cache -f -t "${HOME}/.local/share/icons/hicolor" 2>/dev/null || true
fi

# Update desktop database
if command -v update-desktop-database &> /dev/null; then
    echo "🔄 Updating desktop database..."
    update-desktop-database "${HOME}/.local/share/applications" 2>/dev/null || true
fi

echo ""
echo "✅ Uninstallation complete!"

