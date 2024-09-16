#!/bin/bash
# Installation script for KUVPN
# This script will download KUVPN and install it in $HOME/.kuvpn/bin
# It will also add $HOME/.kuvpn/bin to PATH

COLOR_PRIMARY="\033[0;34m"
COLOR_WARN="\033[1;33m"
COLOR_SUCCESS="\033[0;32m"
COLOR_FAILURE="\033[0;31m"
COLOR_RESET="\033[0m"

TAG="v0.6.4"

echo ""
printf "${COLOR_PRIMARY}Installing KUVPN${COLOR_RESET}\n\n"
printf "This script will download KUVPN and install it in \$HOME/.kuvpn/bin\n\n"

CLI_DOWNLOAD_URL=""

# Detect OS and architecture
OS="$(uname -s)"
ARCH="$(uname -m)"

case "$OS" in
    Darwin)
        if [ "$ARCH" = "x86_64" ]; then
            CLI_DOWNLOAD_URL="https://github.com/KUACC-VALAR-HPC-KOC-UNIVERSITY/kuvpn/releases/download/${TAG}/kuvpn-x86_64-apple-darwin"
        elif [ "$ARCH" = "arm64" ]; then
            CLI_DOWNLOAD_URL="https://github.com/KUACC-VALAR-HPC-KOC-UNIVERSITY/kuvpn/releases/download/${TAG}/kuvpn-aarch64-apple-darwin"
        else
            printf "${COLOR_FAILURE}Unsupported architecture${COLOR_RESET}\n"
            exit 1
        fi
        ;;
    Linux)
        if [ "$ARCH" = "x86_64" ]; then
            CLI_DOWNLOAD_URL="https://github.com/KUACC-VALAR-HPC-KOC-UNIVERSITY/kuvpn/releases/download/${TAG}/kuvpn-x86_64-unknown-linux-musl"
        else
            printf "${COLOR_FAILURE}Unsupported architecture${COLOR_RESET}\n"
            exit 1
        fi
        ;;
    *)
        printf "${COLOR_FAILURE}Unsupported OS${COLOR_RESET}\n"
        exit 1
        ;;
esac

# Create the directory if it doesn't exist
if [ ! -d "$HOME/.kuvpn/bin" ]; then
    mkdir -p "$HOME/.kuvpn/bin" || {
        printf "${COLOR_FAILURE}Failed to create directory!${COLOR_RESET}\n"
        exit 1
    }
fi

# Download the CLI
printf "${COLOR_PRIMARY}Downloading KUVPN...${COLOR_RESET}\n\n"
curl --proto '=https' --tlsv1.2 -sSfL "$CLI_DOWNLOAD_URL" -o "$HOME/.kuvpn/bin/kuvpn" || {
    printf "${COLOR_FAILURE}Download failed!${COLOR_RESET}\n\n"
    exit 1
}
chmod +x "$HOME/.kuvpn/bin/kuvpn"

# Add to PATH
printf "${COLOR_PRIMARY}Adding KUVPN to PATH...${COLOR_RESET}\n\n"
if echo "$PATH" | grep -qv "$HOME/.kuvpn/bin"; then
    if [ -f "$HOME/.bashrc" ] && [ -w "$HOME/.bashrc" ] && ! grep -q 'export PATH=$PATH:$HOME/.kuvpn/bin' "$HOME/.bashrc"; then
        echo 'export PATH=$PATH:$HOME/.kuvpn/bin' >> "$HOME/.bashrc"
        echo "Run 'source $HOME/.bashrc' or restart your terminal to apply the changes."
    elif [ -f "$HOME/.bash_profile" ] && [ -w "$HOME/.bash_profile" ] && ! grep -q 'export PATH=$PATH:$HOME/.kuvpn/bin' "$HOME/.bash_profile"; then
        echo 'export PATH=$PATH:$HOME/.kuvpn/bin' >> "$HOME/.bash_profile"
        echo "Run 'source $HOME/.bash_profile' or restart your terminal to apply the changes."
    elif [ -f "$HOME/.zshrc" ] && [ -w "$HOME/.zshrc" ] && ! grep -q 'export PATH=$PATH:$HOME/.kuvpn/bin' "$HOME/.zshrc"; then
        echo 'export PATH=$PATH:$HOME/.kuvpn/bin' >> "$HOME/.zshrc"
        echo "Run 'source $HOME/.zshrc' or restart your terminal to apply the changes."
    elif [ -f "$HOME/.config/fish/config.fish" ] && [ -w "$HOME/.config/fish/config.fish" ] && ! grep -q 'set -gx PATH $PATH $HOME/.kuvpn/bin' "$HOME/.config/fish/config.fish"; then
        echo 'set -gx PATH $PATH $HOME/.kuvpn/bin' >> "$HOME/.config/fish/config.fish"
        echo "Run 'source $HOME/.config/fish/config.fish' or restart your terminal to apply the changes."
    else
        printf "${COLOR_WARN}Shell profile not detected or is read-only. You may need to manually add $HOME/.kuvpn/bin to your shell profile.${COLOR_RESET}\n"
    fi
fi

# Auto-source for bash, zsh, and fish
if [ -f "$HOME/.bashrc" ]; then
    source "$HOME/.bashrc"
elif [ -f "$HOME/.zshrc" ]; then
    source "$HOME/.zshrc"
elif [ -f "$HOME/.config/fish/config.fish" ]; then
    fish -c "source $HOME/.config/fish/config.fish"
fi

echo ""
printf "${COLOR_SUCCESS}Installation complete!${COLOR_RESET}\n"
