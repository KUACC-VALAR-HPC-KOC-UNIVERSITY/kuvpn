#!/bin/bash
set -e

# Configuration
APP_NAME="KUVPN"
SCRIPT_DIR="$(cd "$(dirname "$0")" && pwd)"
VERSION=$(grep '^version = ' "$SCRIPT_DIR/../Cargo.toml" | head -1 | sed 's/version = "\(.*\)"/\1/')
ARCH=$(uname -m)

# Podman/Docker detection and wrapper
if [ "$1" != "--no-container" ] && [ ! -f /.containerenv ] && [ ! -f /run/.containerenv ]; then
    BUILD_ARGS=""
    if [ "$1" == "--full" ]; then
        BUILD_ARGS="--full"
    fi

    if command -v podman >/dev/null 2>&1; then
        echo "Using Podman to build AppImage for maximum compatibility..."
        podman build --build-arg BUILD_ARGS="$BUILD_ARGS" -t kuvpn-builder -f packaging/appimage/Dockerfile .
        
        CONTAINER_ID=$(podman create kuvpn-builder)
        mkdir -p dist
        podman cp "$CONTAINER_ID":/build/dist/${APP_NAME}-minimal-${ARCH}.AppImage dist/
        if [ "$1" == "--full" ]; then
             podman cp "$CONTAINER_ID":/build/dist/${APP_NAME}-full-${ARCH}.AppImage dist/
        fi
        podman rm "$CONTAINER_ID"
        
        echo "Successfully built and extracted AppImage(s) to dist/."
        exit 0
    elif command -v docker >/dev/null 2>&1; then
        echo "Using Docker to build AppImage for maximum compatibility ($ARCH)..."
        docker build --build-arg BUILD_ARGS="$BUILD_ARGS" -t kuvpn-builder -f packaging/appimage/Dockerfile .
        
        CONTAINER_ID=$(docker create kuvpn-builder)
        mkdir -p dist
        docker cp "$CONTAINER_ID":/build/dist/${APP_NAME}-minimal-${ARCH}.AppImage dist/
        if [ "$1" == "--full" ]; then
             docker cp "$CONTAINER_ID":/build/dist/${APP_NAME}-full-${ARCH}.AppImage dist/
        fi
        docker rm "$CONTAINER_ID"
        
        echo "Successfully built and extracted AppImage(s) to dist/."
        exit 0
    else
        echo "Neither Podman nor Docker found. Building on host (may have compatibility issues)..."
    fi
fi

if [ "$1" == "--no-container" ]; then
    shift
fi

echo "Building $APP_NAME AppImage v$VERSION for $ARCH..."

# Build the GUI binary
cargo build -p kuvpn-gui --release

# Setup AppDir
APPDIR="packaging/appimage/AppDir"
rm -rf "$APPDIR"
mkdir -p "$APPDIR/usr/bin"
mkdir -p "$APPDIR/usr/share/applications"
mkdir -p "$APPDIR/usr/share/icons/hicolor/256x256/apps"

cp target/release/kuvpn-gui "$APPDIR/usr/bin/"

# Create desktop file
cat > "$APPDIR/usr/share/applications/kuvpn.desktop" <<EOF
[Desktop Entry]
Type=Application
Name=KUVPN
Exec=kuvpn-gui
Icon=kuvpn
Categories=Network;
Comment=Connect to Koç University Network
Terminal=false
EOF

# Use the pre-rendered icon from the GUI assets
cp "crates/kuvpn-gui/assets/icon-256.png" packaging/appimage/kuvpn.png

# Download linuxdeploy if needed
if [ ! -f "packaging/appimage/linuxdeploy" ]; then
    curl -L -o packaging/appimage/linuxdeploy https://github.com/linuxdeploy/linuxdeploy/releases/download/continuous/linuxdeploy-${ARCH}.AppImage
    chmod +x packaging/appimage/linuxdeploy
fi

# Download linuxdeploy-plugin-gtk if needed
if [ ! -f "packaging/appimage/linuxdeploy-plugin-gtk.sh" ]; then
    curl -L -o packaging/appimage/linuxdeploy-plugin-gtk.sh https://raw.githubusercontent.com/linuxdeploy/linuxdeploy-plugin-gtk/master/linuxdeploy-plugin-gtk.sh
    chmod +x packaging/appimage/linuxdeploy-plugin-gtk.sh
fi

# Download appimagetool if needed
if [ ! -f "packaging/appimage/appimagetool" ]; then
    curl -L -o packaging/appimage/appimagetool https://github.com/AppImage/AppImageKit/releases/download/continuous/appimagetool-${ARCH}.AppImage
    chmod +x packaging/appimage/appimagetool
fi

# Build Minimal AppImage
export NO_STRIP=1
export DEPLOY_GTK_VERSION=3

# Make sure linuxdeploy can find the gtk plugin
cp packaging/appimage/linuxdeploy-plugin-gtk.sh ./linuxdeploy-plugin-gtk.sh

# List of libraries to bundle explicitly to avoid host mismatches
# We use find to get full paths in the container environment
# We are including almost everything except glibc/libstdc++ to ensure maximum compatibility
LIBS_TO_BUNDLE_LIST="
    libayatana-appindicator3.so.1 
    libayatana-ido3-0.4.so.0 
    libayatana-indicator3.so.7 
    libdbusmenu-glib.so.4 
    libdbusmenu-gtk3.so.4 
    libindicator3.so.7 
    libpangoft2-1.0.so.0 
    libnss3.so 
    libnssutil3.so 
    libsmime3.so 
    libnspr4.so 
    libatk-1.0.so.0 
    libatk-bridge-2.0.so.0 
    libcups.so.2 
    libgbm.so.1 
    libdrm.so.2
    libxdo.so.3 
    libglib-2.0.so.0 
    libgio-2.0.so.0 
    libgobject-2.0.so.0 
    libgmodule-2.0.so.0
    libdbus-1.so.3
    libproxy.so.1
    libfreetype.so.6
    libfontconfig.so.1
    libharfbuzz.so.0
    libfribidi.so.0
    libgraphite2.so.3
    libexpat.so.1
    libz.so.1
    libpng16.so.16
    libjpeg.so.8
    libwayland-client.so.0
    libwayland-cursor.so.0
    libwayland-egl.so.1
    libxkbcommon.so.0
    liblzma.so.5
    liblz4.so.1
    libgcrypt.so.20
    libgpg-error.so.0
    libblkid.so.1
    libmount.so.1
    libselinux.so.1
    libffi.so.8
    libffi.so.7
    libpcre.so.3
    libpcre2-8.so.0
    libuuid.so.1
    libssl.so.3
    libcrypto.so.3
    libssl.so.1.1
    libcrypto.so.1.1
    libasound.so.2
    libpulse.so.0
    libpulse-mainloop-glib.so.0
    libsqlite3.so.0
    libxml2.so.2
    libstdc++.so.6
    libgcc_s.so.1
    libxkbcommon-x11.so.0
    libxcb-xkb.so.1
    libX11-xcb.so.1
    libxcb-render.so.0
    libxcb-shm.so.0
    libxcb-util.so.1
"

LIBS_TO_BUNDLE=""
# Determine multiarch lib path
if [ "$ARCH" == "x86_64" ]; then
    LIB_ARCH="x86_64-linux-gnu"
else
    LIB_ARCH="aarch64-linux-gnu"
fi

for lib in $LIBS_TO_BUNDLE_LIST; do
    LIB_PATH=$(find /usr/lib/$LIB_ARCH -name "$lib" | head -n 1)
    if [ -n "$LIB_PATH" ]; then
        LIBS_TO_BUNDLE="$LIBS_TO_BUNDLE --library $LIB_PATH"
    else
        # Try generic /usr/lib
        LIB_PATH=$(find /usr/lib -name "$lib" | head -n 1)
        if [ -n "$LIB_PATH" ]; then
            LIBS_TO_BUNDLE="$LIBS_TO_BUNDLE --library $LIB_PATH"
        else
            # Try /lib
            LIB_PATH=$(find /lib -name "$lib" | head -n 1)
            if [ -n "$LIB_PATH" ]; then
                LIBS_TO_BUNDLE="$LIBS_TO_BUNDLE --library $LIB_PATH"
            fi
        fi
    fi
done

./packaging/appimage/linuxdeploy --appimage-extract-and-run --appdir "$APPDIR" \
    --executable "$APPDIR/usr/bin/kuvpn-gui" \
    --desktop-file "$APPDIR/usr/share/applications/kuvpn.desktop" \
    --icon-file packaging/appimage/kuvpn.png \
    $LIBS_TO_BUNDLE \
    --plugin gtk \
    --custom-apprun scripts/AppRun.sh
 
    # Flatten libraries to ensure LD_PRELOAD and LD_LIBRARY_PATH find them easily
    if [ -d "$APPDIR/usr/lib/$LIB_ARCH" ]; then
        cp -rn "$APPDIR"/usr/lib/$LIB_ARCH/* "$APPDIR/usr/lib/" || true
    fi

    # Bundle GIO modules from the build environment
    mkdir -p "$APPDIR/usr/lib/gio/modules"
    cp -L /usr/lib/$LIB_ARCH/gio/modules/*.so "$APPDIR/usr/lib/gio/modules/" || true

    # Bundle and compile GSettings schemas
    mkdir -p "$APPDIR/usr/share/glib-2.0/schemas"
    cp -L /usr/share/glib-2.0/schemas/*.gschema.xml "$APPDIR/usr/share/glib-2.0/schemas/" || true
    glib-compile-schemas "$APPDIR/usr/share/glib-2.0/schemas/" || true

rm ./linuxdeploy-plugin-gtk.sh

mkdir -p dist
ARCH=$ARCH ./packaging/appimage/appimagetool --appimage-extract-and-run "$APPDIR" "dist/${APP_NAME}-minimal-${ARCH}.AppImage"

# Build Full AppImage (Optional)
if [ "$1" == "--full" ]; then
    echo "Downloading Chromium for full AppImage..."
    CHROMIUM_ARCH="Linux_x64"
    if [ "$ARCH" == "aarch64" ]; then
        CHROMIUM_ARCH="Linux_Arm64"
    fi
    REVISION=$(curl -s "https://www.googleapis.com/download/storage/v1/b/chromium-browser-snapshots/o/${CHROMIUM_ARCH}%2FLAST_CHANGE?alt=media")
    curl -L -o packaging/chromium.zip "https://www.googleapis.com/download/storage/v1/b/chromium-browser-snapshots/o/${CHROMIUM_ARCH}%2F${REVISION}%2Fchrome-linux.zip?alt=media"
    mkdir -p "$APPDIR/usr/lib/chromium"
    unzip -q packaging/chromium.zip -d packaging/appimage/AppDir/usr/lib/
    mv "$APPDIR/usr/lib/chrome-linux/"* "$APPDIR/usr/lib/chromium/"
    rm -rf "$APPDIR/usr/lib/chrome-linux" packaging/chromium.zip
    
    ARCH=$ARCH ./packaging/appimage/appimagetool --appimage-extract-and-run "$APPDIR" "dist/${APP_NAME}-full-${ARCH}.AppImage"
fi

echo "Done!"
