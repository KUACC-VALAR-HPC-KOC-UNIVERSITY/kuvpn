#!/bin/bash
set -e

# Add osxcross to path
export PATH="/usr/local/osxcross/target/bin:${PATH}"

# Add target architectures
rustup target add x86_64-apple-darwin --toolchain stable
rustup target add aarch64-apple-darwin --toolchain stable

echo "Building for x86_64 macOS..."
export CC_x86_64_apple_darwin=x86_64-apple-darwin22.4-clang
export CXX_x86_64_apple_darwin=x86_64-apple-darwin22.4-clang++
export LIBZ_SYS_STATIC=1 
cargo +stable build --release --target x86_64-apple-darwin -p kuvpn-gui -p kuvpn-cli

echo "Building for aarch64 macOS..."
export CC_aarch64_apple_darwin=aarch64-apple-darwin22.4-clang
export CXX_aarch64_apple_darwin=aarch64-apple-darwin22.4-clang++
export LIBZ_SYS_STATIC=1 
cargo +stable build --release --target aarch64-apple-darwin -p kuvpn-gui -p kuvpn-cli