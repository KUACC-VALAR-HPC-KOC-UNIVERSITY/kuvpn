#!/bin/bash
set -e

NEW_VERSION="$1"

if [ -z "$NEW_VERSION" ]; then
    echo "Usage: $0 <new-version>"
    echo "Example: $0 3.1.0"
    exit 1
fi

SCRIPT_DIR="$(cd "$(dirname "$0")" && pwd)"
ROOT="$SCRIPT_DIR/.."

# Update workspace version in root Cargo.toml
sed -i "s/^version = \".*\"/version = \"$NEW_VERSION\"/" "$ROOT/Cargo.toml"

# Update Windows Inno Setup installer version
sed -i "s/#define MyAppVersion \".*\"/#define MyAppVersion \"$NEW_VERSION\"/" "$ROOT/packaging/windows/kuvpn.iss"

# Update README.md title
sed -i "s/^# KUVPN v.*/# KUVPN v$NEW_VERSION/" "$ROOT/README.md"

# Refresh Cargo.lock
cargo update -w --manifest-path "$ROOT/Cargo.toml"

echo "Bumped to v$NEW_VERSION"
