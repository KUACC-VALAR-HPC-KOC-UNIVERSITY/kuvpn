# Use a specialized cross-compilation image for Rust to macOS
FROM docker.io/joseluisq/rust-linux-darwin-builder:latest AS builder

# Install additional packaging tools
USER root
RUN apt-get update && apt-get install -y \
    genisoimage \
    && rm -rf /var/lib/apt/lists/*

WORKDIR /build
COPY . .

# Ensure the build script is executable
RUN chmod +x packaging/macos/cross_build.sh

# Install latest stable rust toolchain to support iced 0.14
RUN rustup toolchain install stable --profile minimal

# Run the build
RUN ./packaging/macos/cross_build.sh

# Packaging stage
RUN ./scripts/build_macos.sh --no-container --arch x86_64
RUN ./scripts/build_macos.sh --no-container --arch aarch64
