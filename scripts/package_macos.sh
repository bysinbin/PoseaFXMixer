#!/usr/bin/env bash
set -e

VERSION="${1:-1.0.0}"
echo ">>> [1/3] Building Posea Audio for macOS..."
cargo build --release

DIST_DIR="dist"
APP_DIR="$DIST_DIR/Posea Audio.app"
CONTENTS_DIR="$APP_DIR/Contents"
MACOS_DIR="$CONTENTS_DIR/MacOS"
RESOURCES_DIR="$CONTENTS_DIR/Resources"

echo ">>> [2/3] Constructing macOS App Bundle ($APP_DIR)..."
rm -rf "$APP_DIR"
mkdir -p "$MACOS_DIR"
mkdir -p "$RESOURCES_DIR"

cp target/release/posea_fx_mixer "$MACOS_DIR/posea_fx_mixer"
chmod +x "$MACOS_DIR/posea_fx_mixer"

cp packaging/macos/Info.plist "$CONTENTS_DIR/Info.plist"
cp assets/icon.icns "$RESOURCES_DIR/icon.icns"

echo ">>> [3/3] Generating macOS DMG Disk Image..."
DMG_PATH="$DIST_DIR/PoseaAudio-v${VERSION}-macOS.dmg"
rm -f "$DMG_PATH"

# Create temporary staging directory for DMG
DMG_STAGING="$DIST_DIR/dmg_staging"
rm -rf "$DMG_STAGING"
mkdir -p "$DMG_STAGING"
cp -r "$APP_DIR" "$DMG_STAGING/"
ln -s /Applications "$DMG_STAGING/Applications"

hdiutil create -volname "Posea Audio" -srcfolder "$DMG_STAGING" -ov -format UDZO "$DMG_PATH"
rm -rf "$DMG_STAGING"

echo ">>> macOS DMG successfully generated: $DMG_PATH"
