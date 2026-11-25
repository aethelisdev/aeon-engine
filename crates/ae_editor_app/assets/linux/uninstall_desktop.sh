#!/bin/bash
# ============================================================================
# Aeon Engine - Linux Desktop Entry & Icon Uninstaller
# ============================================================================

set -euo pipefail

ICON_NAME="com.aeengine.Editor"
XDG_DATA_HOME="${XDG_DATA_HOME:-${HOME}/.local/share}"
ICON_DEST_BASE="${XDG_DATA_HOME}/icons/hicolor"
DESKTOP_PATH="${XDG_DATA_HOME}/applications/com.aeengine.Editor.desktop"

echo "╔══════════════════════════════════════════════════════════════╗"
echo "║    Aeon Engine - Universal Linux Desktop Uninstaller         ║"
echo "╚══════════════════════════════════════════════════════════════╝"
echo ""

# Remove icons from all hicolor sizes
SIZES=("16x16" "22x22" "24x24" "32x32" "48x48" "64x64" "128x128" "256x256" "512x512")
for SIZE in "${SIZES[@]}"; do
    TARGET_FILE="${ICON_DEST_BASE}/${SIZE}/apps/${ICON_NAME}.png"
    if [ -f "$TARGET_FILE" ]; then
        echo "🗑️  Removing icon: $TARGET_FILE"
        rm "$TARGET_FILE"
    fi
done

if [ -f "$DESKTOP_PATH" ]; then
    echo "🗑️  Removing desktop entry: $DESKTOP_PATH"
    rm "$DESKTOP_PATH"
else
    echo "ℹ️  Desktop entry not found (already removed?): $DESKTOP_PATH"
fi

# Refresh FreeDesktop caches
echo "🔄 Updating FreeDesktop icon and desktop databases..."

if command -v xdg-icon-resource &> /dev/null; then
    xdg-icon-resource forceupdate 2>/dev/null || true
fi

if command -v gtk-update-icon-cache &> /dev/null; then
    gtk-update-icon-cache -f -t "${ICON_DEST_BASE}" 2>/dev/null || true
fi

if command -v update-desktop-database &> /dev/null; then
    update-desktop-database "${XDG_DATA_HOME}/applications" 2>/dev/null || true
fi

echo ""
echo "✅ Uninstallation complete!"

