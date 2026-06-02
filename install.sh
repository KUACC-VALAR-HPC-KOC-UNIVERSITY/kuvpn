#!/bin/bash
# KUVPN Installer

set -e

# --- Configuration ---
REPO="ealtun21/kuvpn-actions"
INSTALL_DIR="$HOME/.local/bin"
BINARY_NAME="kuvpn"
GUI_NAME="KUVPN"
VERSION="${VERSION:-latest}"
INSTALL_CLI=0
INSTALL_GUI=0
MUSL_LINUX=0

# --- Colors ---
if [ -t 1 ]; then
    COLOR_PRIMARY="\033[0;34m"
    COLOR_WARN="\033[1;33m"
    COLOR_SUCCESS="\033[0;32m"
    COLOR_FAILURE="\033[0;31m"
    COLOR_RESET="\033[0m"
else
    COLOR_PRIMARY=""
    COLOR_WARN=""
    COLOR_SUCCESS=""
    COLOR_FAILURE=""
    COLOR_RESET=""
fi

log_info()    { printf "${COLOR_PRIMARY}[INFO]${COLOR_RESET} %s\n" "$1"; }
log_warn()    { printf "${COLOR_WARN}[WARN]${COLOR_RESET} %s\n" "$1"; }
log_success() { printf "${COLOR_SUCCESS}[OK]${COLOR_RESET} %s\n" "$1"; }
log_fail()    { printf "${COLOR_FAILURE}[FAIL]${COLOR_RESET} %s\n" "$1"; exit 1; }

# --- Download helper ---
# Prefers wget (pre-installed on Ubuntu/Debian). Falls back to curl (pre-installed on macOS).
download_file() {
    local url="$1"
    local dest="$2"
    if command -v wget >/dev/null 2>&1; then
        wget -q -O "$dest" "$url"
    elif command -v curl >/dev/null 2>&1; then
        curl --proto '=https' --tlsv1.2 -sSfL "$url" -o "$dest"
    else
        log_fail "Neither wget nor curl is available. Please install wget and try again."
    fi
}

# --- Prompt helper ---
prompt_yn() {
    # Usage: prompt_yn "Question?" [default: y|n]
    local question="$1"
    local default="${2:-y}"
    local prompt_str
    if [ "$default" = "y" ]; then
        prompt_str="[Y/n]"
    else
        prompt_str="[y/N]"
    fi
    while true; do
        printf "${COLOR_PRIMARY}?${COLOR_RESET} %s %s: " "$question" "$prompt_str"
        read -r answer </dev/tty
        answer="${answer:-$default}"
        case "$answer" in
            [Yy]*) return 0 ;;
            [Nn]*) return 1 ;;
            *) echo "  Please answer yes or no." ;;
        esac
    done
}

# --- Architecture Detection ---
detect_platform() {
    OS="$(uname -s)"
    ARCH="$(uname -m)"

    case "$OS" in
        Darwin)
            if [ "$ARCH" = "x86_64" ]; then
                GUI_PLATFORM="macOS-x86_64"
                CLI_ARCH="x86_64"
            elif [ "$ARCH" = "arm64" ]; then
                GUI_PLATFORM="macOS-aarch64"
                CLI_ARCH="aarch64"
            else
                log_fail "Unsupported macOS architecture: $ARCH"
            fi
            ;;
        Linux)
            if [ "$ARCH" = "x86_64" ]; then
                GUI_PLATFORM="linux-x86_64"
                CLI_ARCH="x86_64"
            elif [ "$ARCH" = "aarch64" ] || [ "$ARCH" = "arm64" ]; then
                GUI_PLATFORM="linux-aarch64"
                CLI_ARCH="aarch64"
            else
                log_fail "Unsupported Linux architecture: $ARCH"
            fi
            ;;
        *)
            log_fail "Unsupported OS: $OS"
            ;;
    esac
}

# --- Version Resolution ---
resolve_version() {
    if [ "$VERSION" = "latest" ]; then
        log_info "Resolving latest version..."
        local latest_url="https://github.com/$REPO/releases/latest"
        if command -v wget >/dev/null 2>&1; then
            TAG=$(wget -qO /dev/null --server-response "$latest_url" 2>&1 \
                | grep -i 'Location:' | tail -1 | tr -d '[:space:]' | rev | cut -d/ -f1 | rev)
        elif command -v curl >/dev/null 2>&1; then
            TAG=$(curl -sL -o /dev/null -w '%{url_effective}' "$latest_url" | rev | cut -d/ -f1 | rev)
        else
            log_fail "Neither wget nor curl is available. Please install wget and try again."
        fi
    else
        TAG="$VERSION"
    fi

    if [ -z "$TAG" ]; then
        log_fail "Unable to resolve version."
    fi

    log_info "Selected version: $TAG"
}

# --- OpenConnect Detection (mirrors source code paths) ---
OPENCONNECT_SEARCH_PATHS=(
    "/sbin/openconnect"
    "/usr/sbin/openconnect"
    "/usr/local/sbin/openconnect"
    "/usr/local/bin/openconnect"
    "/opt/homebrew/bin/openconnect"
)

detect_openconnect() {
    # First check $PATH
    if command -v openconnect >/dev/null 2>&1; then
        command -v openconnect
        return 0
    fi
    # Then check the same fallback paths the app uses
    for p in "${OPENCONNECT_SEARCH_PATHS[@]}"; do
        if [ -f "$p" ] && [ -x "$p" ]; then
            echo "$p"
            return 0
        fi
    done
    return 1
}

# --- Package Manager Detection ---
detect_package_manager() {
    if command -v apt-get >/dev/null 2>&1; then echo "apt"
    elif command -v dnf >/dev/null 2>&1; then echo "dnf"
    elif command -v yum >/dev/null 2>&1; then echo "yum"
    elif command -v pacman >/dev/null 2>&1; then echo "pacman"
    elif command -v zypper >/dev/null 2>&1; then echo "zypper"
    elif command -v brew >/dev/null 2>&1; then echo "brew"
    else echo "none"
    fi
}

# --- Musl Detection ---
detect_musl() {
    if [ "$(uname -s)" = "Linux" ]; then
        if ldd --version 2>&1 | grep -qi musl; then
            MUSL_LINUX=1
        fi
    fi
}

# --- OpenConnect Installation ---
install_openconnect_linux() {
    local pkg_mgr
    pkg_mgr=$(detect_package_manager)
    case "$pkg_mgr" in
        apt)
            log_warn "Sudo required: installing a system package (openconnect) via apt."
            log_info "You may be prompted for your password."
            sudo apt-get update -qq && sudo apt-get install -y openconnect
            ;;
        dnf)
            log_warn "Sudo required: installing a system package (openconnect) via dnf."
            log_info "You may be prompted for your password."
            sudo dnf install -y openconnect
            ;;
        yum)
            log_warn "Sudo required: installing a system package (openconnect) via yum."
            log_info "You may be prompted for your password."
            sudo yum install -y openconnect
            ;;
        pacman)
            log_warn "Sudo required: installing a system package (openconnect) via pacman."
            log_info "You may be prompted for your password."
            sudo pacman -Sy --noconfirm openconnect
            ;;
        zypper)
            log_warn "Sudo required: installing a system package (openconnect) via zypper."
            log_info "You may be prompted for your password."
            sudo zypper install -y openconnect
            ;;
        none)
            log_warn "Could not detect a supported package manager."
            echo "  Please install openconnect manually. KUVPN looks for it in:"
            echo "  \$PATH, /sbin, /usr/sbin, /usr/local/sbin, /usr/local/bin"
            echo "  Once installed, you can configure the path in the app's settings."
            ;;
        *)
            log_warn "Package manager '$pkg_mgr' not supported for automatic openconnect install."
            echo "  Please install openconnect manually."
            ;;
    esac
}

install_openconnect_macos() {
    # Check for Homebrew
    if command -v brew >/dev/null 2>&1; then
        if prompt_yn "Install openconnect via Homebrew?"; then
            log_info "Installing openconnect via Homebrew..."
            brew install openconnect
            log_success "openconnect installed via Homebrew."
        fi
    else
        log_warn "Homebrew is not installed."
        if prompt_yn "Install Homebrew first (then openconnect)?"; then
            log_info "Installing Homebrew..."
            /bin/bash -c "$(curl -fsSL https://raw.githubusercontent.com/Homebrew/install/HEAD/install.sh)"
            log_success "Homebrew installed."
            log_info "Installing openconnect via Homebrew..."
            brew install openconnect
            log_success "openconnect installed."
        else
            log_warn "Skipping openconnect installation."
            echo "  KUVPN looks for openconnect in: \$PATH, /usr/local/bin, /opt/homebrew/bin"
            echo "  If you install it later, configure the path in the app's settings."
        fi
    fi
}

check_and_prompt_openconnect() {
    local oc_path
    if oc_path=$(detect_openconnect 2>/dev/null); then
        log_success "openconnect found at: $oc_path"
        return 0
    fi

    log_warn "openconnect was not found on this system."
    echo "  openconnect is required to establish VPN connections."
    echo ""

    if [ "$OS" = "Darwin" ]; then
        if prompt_yn "Would you like to install openconnect now?"; then
            install_openconnect_macos
        else
            echo "  You can install it later with: brew install openconnect"
        fi
    elif [ "$OS" = "Linux" ]; then
        local pkg_mgr
        pkg_mgr=$(detect_package_manager)
        if [ "$pkg_mgr" != "none" ]; then
            if prompt_yn "Would you like to install openconnect via $pkg_mgr?"; then
                install_openconnect_linux
            else
                echo "  Install openconnect using your package manager, e.g.:"
                case "$pkg_mgr" in
                    apt)     echo "    sudo apt-get install openconnect" ;;
                    dnf|yum) echo "    sudo $pkg_mgr install openconnect" ;;
                    pacman)  echo "    sudo pacman -S openconnect" ;;
                    zypper)  echo "    sudo zypper install openconnect" ;;
                esac
                echo "  KUVPN will look for it in \$PATH and common sbin directories."
            fi
        else
            install_openconnect_linux  # will print the manual install message
        fi
    fi
}

# --- FUSE Detection and Installation (Linux AppImage requirement) ---
detect_fuse() {
    # AppImages require libfuse2 (FUSE 2 library) specifically.
    # fuse3/fusermount3 on Ubuntu 22.04+ does NOT satisfy this requirement.
    # Check the shared library via ldconfig, then fall back to known paths.
    if ldconfig -p 2>/dev/null | grep -q 'libfuse\.so\.2'; then
        return 0
    fi
    for f in \
        /usr/lib/x86_64-linux-gnu/libfuse.so.2 \
        /usr/lib/aarch64-linux-gnu/libfuse.so.2 \
        /usr/lib/libfuse.so.2 \
        /usr/lib64/libfuse.so.2; do
        [ -f "$f" ] && return 0
    done
    return 1
}

install_fuse_linux() {
    local pkg_mgr
    pkg_mgr=$(detect_package_manager)
    case "$pkg_mgr" in
        apt)
            log_warn "Sudo required: installing FUSE (required for AppImage) via apt."
            log_info "You may be prompted for your password."
            # Ubuntu 22.04+ ships fuse3 by default; AppImages need libfuse2 specifically.
            local ver_id
            ver_id=$(. /etc/os-release 2>/dev/null && echo "${VERSION_ID:-0}")
            local ver_num
            ver_num=$(echo "$ver_id" | tr -d '.')
            if [ "${ver_num:-0}" -ge 2204 ] 2>/dev/null; then
                sudo apt-get update -qq && sudo apt-get install -y libfuse2
            else
                sudo apt-get update -qq && sudo apt-get install -y fuse
            fi
            ;;
        dnf)
            log_warn "Sudo required: installing FUSE via dnf."
            log_info "You may be prompted for your password."
            sudo dnf install -y fuse fuse-libs
            ;;
        yum)
            log_warn "Sudo required: installing FUSE via yum."
            log_info "You may be prompted for your password."
            sudo yum install -y fuse fuse-libs
            ;;
        pacman)
            log_warn "Sudo required: installing FUSE via pacman."
            log_info "You may be prompted for your password."
            sudo pacman -Sy --noconfirm fuse2
            ;;
        zypper)
            log_warn "Sudo required: installing FUSE via zypper."
            log_info "You may be prompted for your password."
            sudo zypper install -y fuse libfuse2
            ;;
        none|*)
            log_warn "Could not detect a supported package manager."
            echo "  Please install libfuse2 manually:"
            echo "    Ubuntu 22.04+:    sudo apt-get install libfuse2"
            echo "    Ubuntu <22.04:    sudo apt-get install fuse"
            echo "    Fedora/RHEL:      sudo dnf install fuse fuse-libs"
            echo "    Arch Linux:       sudo pacman -S fuse2"
            echo "    openSUSE:         sudo zypper install fuse libfuse2"
            ;;
    esac
}

check_and_prompt_fuse() {
    if detect_fuse; then
        log_success "FUSE is available (required for AppImage)."
        return 0
    fi

    log_warn "libfuse2 is not installed. The KUVPN AppImage requires libfuse2 (FUSE 2) to run."
    echo "  Note: fuse3 (Ubuntu 22.04+ default) does NOT work — AppImages need libfuse2 specifically."
    echo "  Without it, the GUI will not start."
    echo ""

    local pkg_mgr
    pkg_mgr=$(detect_package_manager)
    if [ "$pkg_mgr" != "none" ]; then
        if prompt_yn "Would you like to install FUSE now?"; then
            install_fuse_linux
            if detect_fuse; then
                log_success "FUSE installed successfully."
            else
                log_warn "FUSE installation may not be complete. You may need to log out and back in."
            fi
        else
            echo "  You can install libfuse2 later:"
            case "$pkg_mgr" in
                apt)     echo "    sudo apt-get install libfuse2  (use 'fuse' on Ubuntu older than 22.04)" ;;
                dnf|yum) echo "    sudo $pkg_mgr install fuse fuse-libs" ;;
                pacman)  echo "    sudo pacman -S fuse2" ;;
                zypper)  echo "    sudo zypper install fuse" ;;
            esac
        fi
    else
        install_fuse_linux
    fi
}

# --- Download Logic ---
install_cli_binary() {
    local download_url
    if [ "$OS" = "Darwin" ]; then
        download_url="https://github.com/$REPO/releases/download/${TAG}/${BINARY_NAME}-macos-${CLI_ARCH}"
    else
        download_url="https://github.com/$REPO/releases/download/${TAG}/${BINARY_NAME}-linux-${CLI_ARCH}"
    fi

    local tmp_dir
    tmp_dir=$(mktemp -d)
    local tmp_file="$tmp_dir/$BINARY_NAME"

    log_info "Downloading CLI from: $download_url"

    if ! download_file "$download_url" "$tmp_file"; then
        rm -rf "$tmp_dir"
        log_fail "Download failed. Check your internet connection and try again."
    fi

    mkdir -p "$INSTALL_DIR"
    mv "$tmp_file" "$INSTALL_DIR/$BINARY_NAME"
    chmod +x "$INSTALL_DIR/$BINARY_NAME"
    rm -rf "$tmp_dir"

    log_success "CLI installed at $INSTALL_DIR/$BINARY_NAME"
}

install_gui_binary() {
    if [ "$OS" = "Darwin" ]; then
        local dmg_url="https://github.com/$REPO/releases/download/${TAG}/${GUI_NAME}-${GUI_PLATFORM}.dmg"
        local tmp_dir
        tmp_dir=$(mktemp -d)
        local dmg_path="$tmp_dir/${GUI_NAME}.dmg"

        log_info "Downloading macOS GUI (DMG) from: $dmg_url"
        if ! download_file "$dmg_url" "$dmg_path"; then
            rm -rf "$tmp_dir"
            log_fail "Download failed. Check your internet connection and try again."
        fi

        log_info "Mounting DMG..."
        local mount_point
        mount_point=$(mktemp -d)
        hdiutil attach "$dmg_path" -mountpoint "$mount_point" -nobrowse -quiet

        log_info "Copying KUVPN.app to /Applications..."
        # Remove any existing installation first
        if [ -d "/Applications/KUVPN.app" ]; then
            log_warn "Removing existing /Applications/KUVPN.app..."
            rm -rf "/Applications/KUVPN.app" 2>/dev/null || sudo rm -rf "/Applications/KUVPN.app"
        fi
        # Try without sudo first (works for most admin users); fall back if permission denied
        if ! cp -R "$mount_point/KUVPN.app" "/Applications/KUVPN.app" 2>/dev/null; then
            log_warn "Sudo required: writing to /Applications needs administrator access."
            log_info "You may be prompted for your password."
            sudo cp -R "$mount_point/KUVPN.app" "/Applications/KUVPN.app"
        fi

        log_info "Unmounting DMG..."
        hdiutil detach "$mount_point" -quiet
        rm -rf "$tmp_dir" "$mount_point"

        # macOS blocks apps that weren't installed via the App Store or a notarized installer.
        # The quarantine attribute is what triggers the "app is damaged" / "unidentified developer"
        # error. Removing it with xattr allows the app to open normally.
        #
        # xattr -r (recursive) is supported on newer macOS versions. We test for support first,
        # then use the appropriate command without showing stderr if the flag isn't supported.
        echo ""
        log_warn "Sudo required: removing macOS quarantine attribute from KUVPN.app."
        log_info "macOS blocks unsigned apps by default. This one command allows KUVPN to run."
        log_info "You may be prompted for your password."
        if xattr -h 2>&1 | grep -q -- '-r'; then
            sudo xattr -r -d com.apple.quarantine "/Applications/KUVPN.app" 2>/dev/null || true
        else
            sudo xattr -d com.apple.quarantine "/Applications/KUVPN.app" 2>/dev/null || true
        fi

        log_success "KUVPN.app installed to /Applications."
        echo "  Open KUVPN from your Applications folder or Launchpad."
    else
        local appimage_url="https://github.com/$REPO/releases/download/${TAG}/${GUI_NAME}-${GUI_PLATFORM}.AppImage"
        local dest="$INSTALL_DIR/${GUI_NAME}.AppImage"
        log_info "Downloading Linux GUI (AppImage) from: $appimage_url"
        mkdir -p "$INSTALL_DIR"
        if ! download_file "$appimage_url" "$dest"; then
            log_fail "Download failed. Check your internet connection and try again."
        fi
        chmod +x "$dest"
        log_success "GUI installed at $dest"

        # Ensure FUSE is available (AppImage requirement)
        check_and_prompt_fuse

        # Install icon to XDG icon directory
        local icon_dir="$HOME/.local/share/icons/hicolor/256x256/apps"
        local icon_file="$icon_dir/kuvpn.png"
        mkdir -p "$icon_dir"
        if download_file \
            "https://raw.githubusercontent.com/$REPO/main/crates/kuvpn-gui/assets/icon-256.png" \
            "$icon_file"; then
            log_success "Installed icon at $icon_file"
        else
            log_warn "Could not download KUVPN icon; desktop entry will use a fallback icon."
        fi

        # Create desktop entry
        local desktop_file="$HOME/.local/share/applications/kuvpn.desktop"
        mkdir -p "$(dirname "$desktop_file")"
        cat > "$desktop_file" <<EOF
[Desktop Entry]
Name=KUVPN
Exec=$dest
Icon=kuvpn
Type=Application
Categories=Network;
Comment=Connect to Koç University VPN
EOF
        log_success "Created desktop entry at $desktop_file"
    fi
}

# --- Shell Configuration ---
update_shell_config() {
    # Check if ~/.local/bin is already in PATH
    if echo "$PATH" | grep -q "$HOME/.local/bin"; then
        return 0
    fi

    log_warn "~/.local/bin is not in your PATH. Adding to shell configuration..."

    local path_str="\$HOME/.local/bin"
    local sh_cmd="export PATH=\"$path_str:\$PATH\""
    local fish_cmd="set -gx PATH $path_str \$PATH"
    local marker="# Added by kuvpn installer"
    local updated=0

    if [ "$OS" = "Darwin" ]; then
        # macOS Terminal.app opens login shells, so ~/.zprofile is the right target.
        # We update any existing config files and always ensure ~/.zprofile is written
        # (creating it if needed) so the PATH is set on a fresh install with no prior configs.
        local wrote_to_any=0
        local macos_files=("$HOME/.zprofile" "$HOME/.zshrc" "$HOME/.bash_profile" "$HOME/.bashrc" "$HOME/.profile")
        for config_file in "${macos_files[@]}"; do
            if [ -f "$config_file" ]; then
                if ! grep -qF "$marker" "$config_file" && ! grep -qF ".local/bin" "$config_file"; then
                    { echo ""; echo "$marker"; echo "$sh_cmd"; } >> "$config_file"
                    log_success "Added to PATH in $config_file"
                    updated=1
                    wrote_to_any=1
                elif grep -qF ".local/bin" "$config_file"; then
                    wrote_to_any=1
                fi
            fi
        done
        # If no existing config had .local/bin and ~/.zprofile wasn't already processed,
        # create ~/.zprofile — this covers fresh macOS installs with no shell configs.
        if [ "$wrote_to_any" -eq 0 ]; then
            { echo "$marker"; echo "$sh_cmd"; } >> "$HOME/.zprofile"
            log_success "Created $HOME/.zprofile and added ~/.local/bin to PATH"
            updated=1
        fi
    else
        local files=("$HOME/.bashrc" "$HOME/.bash_profile" "$HOME/.zshrc" "$HOME/.profile")
        for config_file in "${files[@]}"; do
            if [ -f "$config_file" ]; then
                if ! grep -qF "$marker" "$config_file" && ! grep -qF 'export PATH="$HOME/.local/bin:$PATH"' "$config_file"; then
                    { echo ""; echo "$marker"; echo "$sh_cmd"; } >> "$config_file"
                    log_success "Added to PATH in $config_file"
                    updated=1
                fi
            fi
        done
    fi

    local fish_config="${XDG_CONFIG_HOME:-$HOME/.config}/fish/config.fish"
    if [ -d "$(dirname "$fish_config")" ]; then
        touch "$fish_config"
        if ! grep -qF "$marker" "$fish_config" && ! grep -qF 'set -gx PATH $HOME/.local/bin $PATH' "$fish_config"; then
            { echo ""; echo "$marker"; echo "$fish_cmd"; } >> "$fish_config"
            log_success "Added to PATH in $fish_config"
            updated=1
        fi
    fi

    if [ "$updated" -eq 1 ]; then
        log_warn "Please restart your terminal or source your shell config for PATH changes to take effect."
    fi
}

# --- Interactive Selection ---
interactive_select() {
    echo ""
    if [ "$MUSL_LINUX" -eq 1 ]; then
        log_warn "GUI is not supported on musl-based Linux systems."
        echo "  The GUI (AppImage) requires glibc and cannot run on musl libc."
        echo "  Only the CLI (statically linked musl binary) is available."
        echo ""
        INSTALL_CLI=1
        INSTALL_GUI=0
        return
    fi
    echo "What would you like to install?"
    echo "  1) CLI only  (command-line, run with: kuvpn)"
    echo "  2) GUI only  (graphical app with system tray)"
    echo "  3) Both CLI and GUI"
    echo ""
    while true; do
        printf "${COLOR_PRIMARY}?${COLOR_RESET} Enter choice [1/2/3]: "
        read -r choice </dev/tty
        case "$choice" in
            1) INSTALL_CLI=1; INSTALL_GUI=0; break ;;
            2) INSTALL_CLI=0; INSTALL_GUI=1; break ;;
            3) INSTALL_CLI=1; INSTALL_GUI=1; break ;;
            *) echo "  Please enter 1, 2, or 3." ;;
        esac
    done
}

# --- Cleanup Logic ---
cleanup_old_installation() {
    log_info "Cleaning up old KUVPN installations and session data..."

    # 1. Remove session data (Wipe)
    local session_paths=(
        "$HOME/.kuvpn"
        "$HOME/.local/share/kuvpn"
        "$HOME/Library/Application Support/kuvpn"
    )

    for p in "${session_paths[@]}"; do
        if [ -d "$p" ]; then
            log_warn "Removing old session data directory: $p"
            rm -rf "$p"
        elif [ -f "$p" ]; then
            log_warn "Removing old session data file: $p"
            rm -f "$p"
        fi
    done

    # 2. Remove old binaries from system-wide and local paths
    local old_binaries=(
        "/usr/local/bin/kuvpn"
        "/usr/bin/kuvpn"
        "/bin/kuvpn"
        "/usr/local/bin/KUVPN.AppImage"
        "/usr/bin/KUVPN.AppImage"
        "$INSTALL_DIR/$BINARY_NAME"
        "$INSTALL_DIR/${GUI_NAME}.AppImage"
    )

    for b in "${old_binaries[@]}"; do
        if [ -e "$b" ]; then
            log_warn "Removing old installation: $b"
            if [ -w "$b" ] || [ -w "$(dirname "$b")" ]; then
                rm -rf "$b"
            else
                log_info "Sudo required to remove $b"
                sudo rm -rf "$b" 2>/dev/null || rm -rf "$b"
            fi
        fi
    done

    # macOS specific: ensure /Applications/KUVPN.app is gone if we're doing a fresh start
    if [ "$OS" = "Darwin" ]; then
        if [ -d "/Applications/KUVPN.app" ]; then
            log_warn "Removing old /Applications/KUVPN.app"
            sudo rm -rf "/Applications/KUVPN.app" 2>/dev/null || rm -rf "/Applications/KUVPN.app"
        fi
    fi

    log_success "Cleanup completed."
}

# --- Main Execution ---

echo ""
log_info "KUVPN Installer"
echo ""

detect_musl

# Parse arguments
if [ $# -eq 0 ]; then
    interactive_select
else
    for arg in "$@"; do
        case $arg in
            --cli)         INSTALL_CLI=1 ;;
            --gui)
                if [ "$MUSL_LINUX" -eq 1 ]; then
                    log_warn "GUI is not supported on musl-based Linux. Ignoring --gui."
                    echo "  The GUI (AppImage) requires glibc. Only the CLI is available on musl."
                else
                    INSTALL_GUI=1
                fi
                ;;
            --all)
                INSTALL_CLI=1
                if [ "$MUSL_LINUX" -eq 1 ]; then
                    log_warn "GUI is not supported on musl-based Linux. Installing CLI only."
                    echo "  The GUI (AppImage) requires glibc. Only the CLI is available on musl."
                else
                    INSTALL_GUI=1
                fi
                ;;
            --version=*)   VERSION="${arg#*=}" ;;
            -y|--yes|--force) ;; # legacy flag, no-op now
        esac
    done
    # Fallback: no install target specified, default to CLI
    if [ "$INSTALL_CLI" -eq 0 ] && [ "$INSTALL_GUI" -eq 0 ]; then
        INSTALL_CLI=1
    fi
fi

detect_platform
resolve_version

# Clean up before installing
cleanup_old_installation

if [ "$INSTALL_CLI" -eq 1 ]; then
    install_cli_binary
fi

if [ "$INSTALL_GUI" -eq 1 ]; then
    install_gui_binary
fi

if [ "$INSTALL_CLI" -eq 1 ] || [ "$INSTALL_GUI" -eq 1 ]; then
    update_shell_config
fi

echo ""
check_and_prompt_openconnect

echo ""
log_success "Done!"
if [ "$INSTALL_CLI" -eq 1 ]; then
    echo "  Run 'kuvpn' to use the CLI."
fi
if [ "$INSTALL_GUI" -eq 1 ] && [ "$OS" = "Linux" ]; then
    echo "  You can find KUVPN in your application menu, or run the AppImage directly."
fi
echo ""
