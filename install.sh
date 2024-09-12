#!/bin/bash
# Installation script for KUVPN
# This script will download the KUVPN and install them in $HOME/.kuvpn/bin
# It will also add $HOME/.kuvpn/bin to PATH
# Usage:
# curl --proto '=https' --tlsv1.2 -sSfL URL_TO_SCRIPT_HERE | sh

COLOR_PRIMARY="\033[0;34m"
COLOR_WARN="\033[1;33m"
COLOR_SUCCESS="\033[0;32m"
COLOR_FAILURE="\033[0;31m"
COLOR_RESET="\033[0m"

TAG="v0.6.3"

echo ""
echo "=================================="
echo ""
echo -e "${COLOR_PRIMARY}Installing KUVPN${COLOR_RESET}"
echo ""
echo ""
echo -e "This script will download KUVPN and install it in \$HOME/.kuvpn/bin"
echo ""
echo "=================================="
echo ""


CLI_DOWNLOAD_URL=""

# detect os
if [[ "$OSTYPE" = "darwin"* ]]; then
    # detect arch
    if [[ "$HOSTTYPE" = "x86_64" ]]; then
        # install macos cli
        CLI_DOWNLOAD_URL="https://github.com/KUACC-VALAR-HPC-KOC-UNIVERSITY/kuvpn/releases/download/${TAG}/kuvpn-x86_64-apple-darwin"
    elif [[ "$HOSTTYPE" = "arm64" ]]; then
        # install macos cli
        CLI_DOWNLOAD_URL="https://github.com/KUACC-VALAR-HPC-KOC-UNIVERSITY/kuvpn/releases/download/${TAG}/kuvpn-aarch64-apple-darwin"
    else
        echo -e "${COLOR_FAILURE}unsupported architecture${COLOR_RESET}"
        exit 1
    fi

elif [[ "$OSTYPE" = "linux"* ]]; then
    if [[ "$HOSTTYPE" = "x86_64" ]]; then
        CLI_DOWNLOAD_URL="https://github.com/KUACC-VALAR-HPC-KOC-UNIVERSITY/kuvpn/releases/download/${TAG}/kuvpn-x86_64-unknown-linux-musl"
    else
        echo -e "${COLOR_FAILURE}unsupported architecture${COLOR_RESET}"
        exit 1
    fi

else
    echo -e "${COLOR_FAILURE}unsupported OS${COLOR_RESET}"
    exit 1
fi

# check if .kuvpn/bin folder exists under home directory
if [[ ! -d "$HOME/.kuvpn/bin" ]]; then
    mkdir -p "$HOME/.kuvpn/bin" || {
        echo -e "${COLOR_FAILURE}Failed to create directory!${COLOR_RESET}"
        exit 1
    }
fi

# download cli
echo -e "${COLOR_PRIMARY}Downloading CLI${COLOR_RESET}"
echo ""
curl --proto '=https' --tlsv1.2 -sSfL "$CLI_DOWNLOAD_URL" -o "$HOME/.kuvpn/bin/kuvpn" || {
    echo -e "${COLOR_FAILURE}Download failed!${COLOR_RESET}"
    exit 1
}
chmod +x "$HOME/.kuvpn/bin/kuvpn"
echo ""
echo "=================================="
echo ""

# add .kuvpn/bin to PATH
echo -e "${COLOR_PRIMARY}Adding .kuvpn/bin to PATH${COLOR_RESET}"
echo ""
if [[ ":$PATH:" != *":$HOME/.kuvpn/bin:"* ]]; then

    echo "Adding .kuvpn/bin to PATH"

    # check if .bashrc or .bash_profile exists
    if [[ -f "$HOME/.bashrc" ]]; then
        if ! grep -q 'export PATH=$PATH:$HOME/.kuvpn/bin' "$HOME/.bashrc"; then
            echo 'export PATH=$PATH:$HOME/.kuvpn/bin' >> "$HOME/.bashrc"
            echo "Run source $HOME/.bashrc to apply changes"
        fi
    elif [[ -f "$HOME/.bash_profile" ]]; then
        if ! grep -q 'export PATH=$PATH:$HOME/.kuvpn/bin' "$HOME/.bash_profile"; then
            echo 'export PATH=$PATH:$HOME/.kuvpn/bin' >> "$HOME/.bash_profile"
            echo "Run source $HOME/.bash_profile to apply changes"
        fi
    fi

    # check if .zshrc exists
    if [[ -f "$HOME/.zshrc" ]]; then
        if ! grep -q 'export PATH=$PATH:$HOME/.kuvpn/bin' "$HOME/.zshrc"; then
            echo 'export PATH=$PATH:$HOME/.kuvpn/bin' >> "$HOME/.zshrc"
            echo "Run source $HOME/.zshrc to apply changes"
        fi
    fi

fi
echo "If you are using a shell other than bash or zsh, please add the following line to your shell profile:"
echo 'export PATH=$PATH:$HOME/.kuvpn/bin'

echo ""
echo "=================================="
echo ""

echo -e "${COLOR_SUCCESS}Installation complete!${COLOR_RESET}"