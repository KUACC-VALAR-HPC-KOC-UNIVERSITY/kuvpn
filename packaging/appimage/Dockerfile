FROM ubuntu:22.04

ENV DEBIAN_FRONTEND=noninteractive

# Install dependencies
RUN apt-get update && apt-get install -y \
    curl \
    git \
    build-essential \
    pkg-config \
    libssl-dev \
    libgtk-3-dev \
    libayatana-appindicator3-dev \
    libayatana-ido3-dev \
    libayatana-indicator3-dev \
    libdbusmenu-glib-dev \
    libdbusmenu-gtk3-dev \
    libdbusmenu-gtk-dev \
    libindicator3-dev \
    librsvg2-dev \
    librsvg2-bin \
    libasound2-dev \
    libdbus-1-dev \
    libxdo-dev \
    libxkbcommon-dev \
    libxkbcommon-x11-dev \
    libxcb-xkb-dev \
    libxcb-render0-dev \
    libxcb-shm0-dev \
    libxcb-util-dev \
    libxcb-image0-dev \
    libxcb1-dev \
    libvulkan-dev \
    libfuse2 \
    file \
    wget \
    unzip \
    && rm -rf /var/lib/apt/lists/*

# Install Rust and appropriate target
RUN curl --proto '=https' --tlsv1.2 -sSf https://sh.rustup.rs | sh -s -- -y
ENV PATH="/root/.cargo/bin:${PATH}"
RUN rustup target add $(uname -m)-unknown-linux-musl

WORKDIR /build

# Copy source
COPY . .

# Build
ARG BUILD_ARGS=""
RUN ./scripts/build_appimage.sh --no-container $BUILD_ARGS
