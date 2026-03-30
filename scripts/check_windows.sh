#!/bin/bash
# Lightweight Windows type-check.
# Requires both the Rust target AND the mingw C compiler (for build scripts
# in crates like ring and zstd-sys):
#   rustup target add x86_64-pc-windows-gnu
#   pacman -S mingw-w64-gcc   # Arch / CachyOS
#   apt install gcc-mingw-w64  # Debian / Ubuntu
set -e
cargo check --target x86_64-pc-windows-gnu --workspace
cargo clippy --target x86_64-pc-windows-gnu --workspace -- -D warnings
echo "Windows check passed."
