FROM ubuntu:20.04

ENV DEBIAN_FRONTEND=noninteractive

# Install dependencies for Windows cross-compilation and Wine (including 32-bit for Inno Setup)
RUN dpkg --add-architecture i386 && \
    apt-get update && apt-get install -y \
    curl \
    git \
    build-essential \
    pkg-config \
    gcc-mingw-w64-x86-64 \
    g++-mingw-w64-x86-64 \
    wine64 \
    wine32 \
    xvfb \
    wget \
    unzip \
    p7zip-full \
    && rm -rf /var/lib/apt/lists/*

# Install Rust
RUN curl --proto '=https' --tlsv1.2 -sSf https://sh.rustup.rs | sh -s -- -y
ENV PATH="/root/.cargo/bin:${PATH}"
RUN rustup target add x86_64-pc-windows-gnu

# Configure cargo for Windows cross-compilation
RUN mkdir -p /root/.cargo && \
    printf '[target.x86_64-pc-windows-gnu]\nlinker = "x86_64-w64-mingw32-gcc"\n' > /root/.cargo/config.toml

WORKDIR /build

# Copy source
COPY . .

# Build
RUN ./scripts/build_windows.sh --no-container