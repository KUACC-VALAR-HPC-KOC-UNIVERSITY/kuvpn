#!/bin/bash
# Comprehensive macOS type-check.
# Uses a container with host-path caching for speed.
set -e

# Podman/Docker detection for cross-compilation
if [ "$1" != "--no-container" ] && [ ! -f /.containerenv ]; then
    if command -v podman >/dev/null 2>&1 || command -v docker >/dev/null 2>&1; then
        BUILDER="podman"
        command -v docker >/dev/null 2>&1 && BUILDER="docker"
        
        echo "Using $BUILDER container for macOS check (with full caching)..."
        
        # Ensure cargo directories exist on host to avoid root-owned auto-creation
        mkdir -p "$HOME/.cargo/registry" "$HOME/.cargo/git"

        $BUILDER run --rm \
            -v "$(pwd)":/build:z \
            -v "$HOME/.cargo/registry":/root/.cargo/registry:z \
            -v "$HOME/.cargo/git":/root/.cargo/git:z \
            -w /build \
            kuvpn-macos-builder /bin/bash -c "
            set -e
            export PATH=\"/usr/local/osxcross/target/bin:\${PATH}\"
            export CARGO_TARGET_DIR=\"/build/target\"
            
            echo \"Updating toolchain...\"
            rustup update stable 2>&1 | tail -1
            rustup component add clippy --toolchain stable 2>&1 | tail -1

            echo \"Checking x86_64-apple-darwin...\"
            export CC_x86_64_apple_darwin=x86_64-apple-darwin22.4-clang
            export CXX_x86_64_apple_darwin=x86_64-apple-darwin22.4-clang++
            export LIBZ_SYS_STATIC=1
            cargo +stable check --target x86_64-apple-darwin --workspace
            cargo +stable clippy --target x86_64-apple-darwin --workspace -- -D warnings

            echo \"Checking aarch64-apple-darwin...\"
            export CC_aarch64_apple_darwin=aarch64-apple-darwin22.4-clang
            export CXX_aarch64_apple_darwin=aarch64-apple-darwin22.4-clang++
            export LIBZ_SYS_STATIC=1
            cargo +stable check --target aarch64-apple-darwin --workspace
            cargo +stable clippy --target aarch64-apple-darwin --workspace -- -D warnings
        "
        echo "macOS checks passed (via container)."
        exit 0
    fi
fi

if [ "$1" == "--no-container" ]; then
    shift
fi

echo "Checking x86_64-apple-darwin (Native)..."
cargo check --target x86_64-apple-darwin --workspace
cargo clippy --target x86_64-apple-darwin --workspace -- -D warnings

echo "Checking aarch64-apple-darwin (Native)..."
cargo check --target aarch64-apple-darwin --workspace
cargo clippy --target aarch64-apple-darwin --workspace -- -D warnings

echo "macOS checks passed."
