#!/usr/bin/env bash
set -e

VERSION="${1:-1.0.0}"
echo ">>> [1/3] Building Posea Audio for Linux..."
cargo build --release

DIST_DIR="dist"
mkdir -p "$DIST_DIR"

# 1. Debian Package (.deb)
echo ">>> [2/3] Constructing Debian Package (.deb)..."
DEB_STAGE="$DIST_DIR/deb_staging"
rm -rf "$DEB_STAGE"

mkdir -p "$DEB_STAGE/DEBIAN"
mkdir -p "$DEB_STAGE/usr/bin"
mkdir -p "$DEB_STAGE/usr/share/applications"
mkdir -p "$DEB_STAGE/usr/share/icons/hicolor/512x512/apps"
mkdir -p "$DEB_STAGE/usr/share/icons/hicolor/scalable/apps"

cp packaging/linux/control "$DEB_STAGE/DEBIAN/control"
sed -i "s/Version: 1.0.0/Version: ${VERSION}/g" "$DEB_STAGE/DEBIAN/control"

cp target/release/posea_fx_mixer "$DEB_STAGE/usr/bin/posea_fx_mixer"
chmod +x "$DEB_STAGE/usr/bin/posea_fx_mixer"

cp packaging/linux/posea-audio.desktop "$DEB_STAGE/usr/share/applications/"
cp assets/icon_512.png "$DEB_STAGE/usr/share/icons/hicolor/512x512/apps/posea-audio.png"
cp assets/icon.svg "$DEB_STAGE/usr/share/icons/hicolor/scalable/apps/posea-audio.svg"

DEB_FILE="$DIST_DIR/posea-audio_${VERSION}_amd64.deb"
dpkg-deb --build "$DEB_STAGE" "$DEB_FILE"
rm -rf "$DEB_STAGE"
echo "    Debian package created: $DEB_FILE"

# 2. Portable Linux Tarball (.tar.gz)
echo ">>> [3/3] Creating Portable Linux Tarball..."
TAR_STAGE="$DIST_DIR/posea-audio-v${VERSION}-linux-x64"
rm -rf "$TAR_STAGE"
mkdir -p "$TAR_STAGE"

cp target/release/posea_fx_mixer "$TAR_STAGE/"
cp packaging/linux/posea-audio.desktop "$TAR_STAGE/"
cp assets/icon_512.png "$TAR_STAGE/icon.png"
cp assets/icon.svg "$TAR_STAGE/icon.svg"
cp README.md "$TAR_STAGE/" 2>/dev/null || true

cat << 'EOF' > "$TAR_STAGE/install.sh"
#!/usr/bin/env bash
set -e
sudo cp posea_fx_mixer /usr/local/bin/
sudo cp posea-audio.desktop /usr/share/applications/
sudo mkdir -p /usr/share/icons/hicolor/512x512/apps/
sudo cp icon.png /usr/share/icons/hicolor/512x512/apps/posea-audio.png
echo "Posea Audio installed successfully!"
EOF
chmod +x "$TAR_STAGE/install.sh"

TAR_FILE="$DIST_DIR/PoseaAudio-v${VERSION}-linux-x64.tar.gz"
tar -czf "$TAR_FILE" -C "$DIST_DIR" "posea-audio-v${VERSION}-linux-x64"
rm -rf "$TAR_STAGE"

echo ">>> Linux packaging completed successfully: $TAR_FILE"
