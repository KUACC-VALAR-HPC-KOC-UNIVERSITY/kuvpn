#!/bin/bash
export SELF=$(readlink -f "$0")
export HERE="${SELF%/*}"

ARCH=$(uname -m)
if [ "$ARCH" = "x86_64" ]; then
    LIB_ARCH="x86_64-linux-gnu"
elif [ "$ARCH" = "aarch64" ] || [ "$ARCH" = "arm64" ]; then
    LIB_ARCH="aarch64-linux-gnu"
fi

# Host system libs FIRST so the Vulkan loader and GPU drivers (which must come
# from the host) are found before any bundled copies. Bundled AppImage libs are
# appended so app-specific deps still resolve from the bundle.
export LD_LIBRARY_PATH="/usr/lib:/usr/lib64:/usr/lib/$LIB_ARCH:${HERE}/usr/lib:${HERE}/usr/lib/$LIB_ARCH:${HERE}/lib/$LIB_ARCH:${HERE}/lib:${LD_LIBRARY_PATH}"
export XDG_DATA_DIRS="${HERE}/usr/share:${XDG_DATA_DIRS:-/usr/local/share:/usr/share}"
export GIO_MODULE_DIR="${HERE}/usr/lib/gio/modules"
export GSETTINGS_SCHEMA_DIR="${HERE}/usr/share/glib-2.0/schemas"

# Disable host GTK modules and paths to avoid loading incompatible host-system plugins
export GTK_MODULES=""
export GTK_PATH=""

# Preload only appindicator/dbusmenu libs from the bundle to avoid crashes.
# We intentionally do NOT preload glib, libstdc++, or other core libs — doing
# so would shadow the host's versions and break the Vulkan GPU driver loading.
PRELOAD_LIBS=""
for lib in libdbusmenu-gtk3.so.4 libdbusmenu-glib.so.4 libayatana-appindicator3.so.1 libayatana-ido3-0.4.so.0 libayatana-indicator3.so.7; do
    FOUND=""
    for dir in "${HERE}/usr/lib" "${HERE}/usr/lib/$LIB_ARCH" "${HERE}/lib" "${HERE}/lib/$LIB_ARCH"; do
        if [ -f "$dir/$lib" ]; then
            FOUND="$dir/$lib"
            break
        fi
    done

    if [ -n "$FOUND" ]; then
        if [ -z "$PRELOAD_LIBS" ]; then
            PRELOAD_LIBS="$FOUND"
        else
            PRELOAD_LIBS="$PRELOAD_LIBS:$FOUND"
        fi
    fi
done

if [ -n "$PRELOAD_LIBS" ]; then
    export LD_PRELOAD="$PRELOAD_LIBS${LD_PRELOAD:+:$LD_PRELOAD}"
fi

# Vulkan ICD discovery: scan standard host paths for Vulkan ICD JSON files
# so wgpu can find the host's GPU drivers (AMD, NVIDIA, Intel, etc.)
if [ -z "$VK_ICD_FILENAMES" ]; then
    VK_ICD_FILES=""
    for icd_dir in /usr/share/vulkan/icd.d /etc/vulkan/icd.d /usr/local/share/vulkan/icd.d; do
        if [ -d "$icd_dir" ]; then
            for f in "$icd_dir"/*.json; do
                [ -f "$f" ] && VK_ICD_FILES="${VK_ICD_FILES:+$VK_ICD_FILES:}$f"
            done
        fi
    done
    [ -n "$VK_ICD_FILES" ] && export VK_ICD_FILENAMES="$VK_ICD_FILES"
fi

# Chrome path for full version
export KUVPN_CHROME_PATH="${HERE}/usr/lib/chromium/chrome"
if [ ! -f "$KUVPN_CHROME_PATH" ]; then
    unset KUVPN_CHROME_PATH
fi

# Run the app
exec "${HERE}/usr/bin/kuvpn-gui" "$@"
