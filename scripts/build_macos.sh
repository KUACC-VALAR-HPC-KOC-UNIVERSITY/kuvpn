#!/bin/bash
set -e

APP_NAME="KUVPN"
SCRIPT_DIR="$(cd "$(dirname "$0")" && pwd)"
VERSION=$(grep '^version = ' "$SCRIPT_DIR/../Cargo.toml" | head -1 | sed 's/version = "\(.*\)"/\1/')
ARCH="x86_64"

# Podman/Docker detection
if [ "$1" != "--no-container" ] && [ ! -f /.containerenv ] && [ ! -f /proc/1/cgroup ] || [ "$(grep -q docker /proc/1/cgroup)" ]; then
    if command -v podman >/dev/null 2>&1 || command -v docker >/dev/null 2>&1; then
        BUILDER="podman"
        command -v docker >/dev/null 2>&1 && BUILDER="docker"
        
        echo "Using $BUILDER to build macOS App Bundles..."
        $BUILDER build -t kuvpn-macos-builder -f packaging/macos/Dockerfile .
        
        CONTAINER_ID=$($BUILDER create kuvpn-macos-builder)
        mkdir -p dist/macos
        $BUILDER cp "$CONTAINER_ID":/build/KUVPN-x86_64.app dist/macos/
        $BUILDER cp "$CONTAINER_ID":/build/KUVPN-aarch64.app dist/macos/
        # Copy DMG/PKG if generated
        $BUILDER cp "$CONTAINER_ID":/build/KUVPN-x86_64.dmg dist/macos/ 2>/dev/null || true
        $BUILDER cp "$CONTAINER_ID":/build/KUVPN-aarch64.dmg dist/macos/ 2>/dev/null || true
        $BUILDER rm "$CONTAINER_ID"
        
        echo "Successfully built macOS artifacts in dist/macos/"
        exit 0
    fi
fi

if [ "$1" == "--no-container" ]; then
    shift
fi

while [[ "$#" -gt 0 ]]; do
    case $1 in
        --arch) ARCH="$2"; shift ;;
    esac
    shift
done

TARGET="${ARCH}-apple-darwin"

echo "Packaging $APP_NAME for macOS ($ARCH)..."

# Build the binaries (if not already built by Dockerfile)
if [ ! -f "target/${TARGET}/release/kuvpn-gui" ]; then
    echo "Building ${TARGET}..."
    cargo build -p kuvpn-cli -p kuvpn-gui --release --target "${TARGET}"
fi

# Setup App Bundle
APP_BUNDLE="${APP_NAME}-${ARCH}.app"
rm -rf "$APP_BUNDLE"
mkdir -p "$APP_BUNDLE/Contents/MacOS"
mkdir -p "$APP_BUNDLE/Contents/Resources"

cp "target/${TARGET}/release/kuvpn-gui" "$APP_BUNDLE/Contents/MacOS/"
cp crates/kuvpn-gui/assets/kuvpn.icns "$APP_BUNDLE/Contents/Resources/"

# Create Info.plist
# CFBundleIconFile: reference without extension — macOS locates kuvpn.icns automatically.
cat > "$APP_BUNDLE/Contents/Info.plist" <<EOF
<?xml version="1.0" encoding="UTF-8"?>
<!DOCTYPE plist PUBLIC "-//Apple//DTD PLIST 1.0//EN" "http://www.apple.com/DTDs/PropertyList-1.0.dtd">
<plist version="1.0">
<dict>
    <key>CFBundleExecutable</key>
    <string>kuvpn-gui</string>
    <key>CFBundleIdentifier</key>
    <string>tr.edu.ku.kuvpn</string>
    <key>CFBundleName</key>
    <string>KUVPN</string>
    <key>CFBundleVersion</key>
    <string>$VERSION</string>
    <key>CFBundleShortVersionString</key>
    <string>$VERSION</string>
    <key>CFBundlePackageType</key>
    <string>APPL</string>
    <key>CFBundleIconFile</key>
    <string>kuvpn</string>
    <key>LSMinimumSystemVersion</key>
    <string>10.15</string>
</dict>
</plist>
EOF

if [[ "$OSTYPE" == "darwin"* ]]; then
    echo "Creating DMG (Native)..."
    hdiutil create -volname "${APP_NAME}" -srcfolder "${APP_BUNDLE}" -ov -format UDZO "${APP_NAME}-${ARCH}.dmg"
    
    echo "Creating PKG (Native)..."
    pkgbuild --component "${APP_BUNDLE}" --install-location /Applications "${APP_NAME}-${ARCH}.pkg"
    
    echo "Done! ${APP_NAME}-${ARCH}.dmg and ${APP_NAME}-${ARCH}.pkg created."
elif command -v genisoimage >/dev/null 2>&1; then
    echo "Creating DMG using genisoimage..."
    genisoimage -V "${APP_NAME}" -D -R -apple -no-pad -o "${APP_NAME}-${ARCH}.dmg" "${APP_BUNDLE}"
    echo "Done! ${APP_NAME}-${ARCH}.dmg created (Basic ISO-based DMG)."
else
    echo "No DMG creation tool found, skipping DMG/PKG creation. App bundle is at ${APP_BUNDLE}"
fi
