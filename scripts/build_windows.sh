#!/bin/bash
set -e

# Configuration
APP_NAME="KUVPN"
SCRIPT_DIR="$(cd "$(dirname "$0")" && pwd)"
VERSION=$(grep '^version = ' "$SCRIPT_DIR/../Cargo.toml" | head -1 | sed 's/version = "\(.*\)"/\1/')
OPENCONNECT_URL="https://github.com/openconnect/openconnect-gui/releases/download/v1.5.3/openconnect-gui-1.5.3-win32.exe"
INNO_SETUP_URL="https://files.jrsoftware.org/is/6/innosetup-6.2.2.exe"

# Podman detection
if [ "$1" != "--no-container" ] && [ ! -f /.containerenv ]; then
    if command -v podman >/dev/null 2>&1; then
        echo "Using Podman to build full Windows Installer..."
        podman build -t kuvpn-windows-builder -f packaging/windows/Dockerfile .
        
        CONTAINER_ID=$(podman create kuvpn-windows-builder)
        mkdir -p dist/windows
        podman cp "$CONTAINER_ID":/build/packaging/windows/KUVPN-Setup.exe dist/windows/
        podman cp "$CONTAINER_ID":/build/target/x86_64-pc-windows-gnu/release/kuvpn-gui.exe dist/windows/
        podman rm "$CONTAINER_ID"
        
        echo "Successfully built Windows installer in dist/windows/KUVPN-Setup.exe"
        exit 0
    fi
fi

if [ "$1" == "--no-container" ]; then
    shift
fi

echo "Building $APP_NAME for Windows..."

# 1. Cross-compile Rust binaries
rustup target add x86_64-pc-windows-gnu
cargo build -p kuvpn-gui --release --target x86_64-pc-windows-gnu
cargo build -p kuvpn-win-helper --release --target x86_64-pc-windows-gnu

# 2. Download and prepare OpenConnect
echo "Preparing OpenConnect binaries..."
OC_DIR="packaging/windows/openconnect"
mkdir -p "$OC_DIR"

if [ ! -f "$OC_DIR/openconnect.exe" ]; then
    echo "Downloading OpenConnect bundle..."
    wget -q -O /tmp/oc_setup.exe "$OPENCONNECT_URL"
    if command -v 7z >/dev/null 2>&1; then
        echo "Extracting OpenConnect files..."
        rm -rf /tmp/oc_extract
        mkdir -p /tmp/oc_extract
        7z x -y /tmp/oc_setup.exe -o/tmp/oc_extract > /dev/null || true
        
        find /tmp/oc_extract -name "openconnect.exe" -exec cp {} "$OC_DIR/" \;
        find /tmp/oc_extract -name "*.dll" -exec cp {} "$OC_DIR/" \;
        find /tmp/oc_extract -name "vpnc-script.js" -exec cp {} "$OC_DIR/" \;
        
        TAP_EXE=$(find /tmp/oc_extract -name "tap-windows*.exe" | head -n 1)
        if [ -n "$TAP_EXE" ]; then
            cp "$TAP_EXE" "$OC_DIR/tap-setup.exe"
            echo "Found and bundled TAP installer: $TAP_EXE"
        else
            echo "Warning: TAP installer not found in OpenConnect bundle."
        fi
        rm -rf /tmp/oc_extract
    else
        echo "Warning: 7z not found. Skipping OpenConnect bundling."
    fi
fi

# 3. Download and bundle Wintun driver
if [ ! -f "$OC_DIR/wintun.dll" ]; then
    echo "Preparing Wintun driver..."
    WINTUN_URL="https://www.wintun.net/builds/wintun-0.14.1.zip"
    wget -q -O /tmp/wintun.zip "$WINTUN_URL"
    if command -v unzip >/dev/null 2>&1; then
        rm -rf /tmp/wintun_extract
        mkdir -p /tmp/wintun_extract
        unzip -q /tmp/wintun.zip -d /tmp/wintun_extract
        cp /tmp/wintun_extract/wintun/bin/amd64/wintun.dll "$OC_DIR/"
        echo "Bundled wintun.dll"
        rm -rf /tmp/wintun_extract
    else
        echo "Warning: unzip not found. Skipping Wintun bundling."
    fi
fi

# 4. Generate Installer with Inno Setup (requires Wine)
if command -v wine >/dev/null 2>&1; then
    echo "Generating Windows Installer with Inno Setup..."
    
    ISCC_PATH="/root/.wine/drive_c/Program Files (x86)/Inno Setup 6/ISCC.exe"
    if [ ! -f "$ISCC_PATH" ]; then
        echo "Inno Setup not found. Installing via Wine (with Xvfb)..."
        wget -q -O /tmp/is_setup.exe "$INNO_SETUP_URL"
        xvfb-run -a env WINEDEBUG=-all wine /tmp/is_setup.exe /VERYSILENT /SUPPRESSMSGBOXES /ALLUSERS /NOICONS || true
        for i in {1..15}; do
            [ -f "$ISCC_PATH" ] && break
            sleep 2
        done
    fi

    if [ -f "$ISCC_PATH" ] || command -v iscc >/dev/null 2>&1; then
        ISCC_CMD="iscc"
        [ -f "$ISCC_PATH" ] && ISCC_CMD="WINEDEBUG=-all wine \"$ISCC_PATH\""
        eval $ISCC_CMD packaging/windows/kuvpn.iss
    else
        echo "Warning: ISCC not found. Skipping installer generation."
    fi
else
    echo "Warning: Wine not found. Skipping installer generation."
fi

echo "Done!"
