#!/bin/bash
# Comprehensive type-check for all supported platforms.
set -e

SCRIPT_DIR="$(cd "$(dirname "$0")" && pwd)"
FLAGS=$@

echo "======================================"
echo "   KUVPN MULTI-PLATFORM CHECK"
echo "======================================"

echo ""
echo ">>> [1/3] Checking Linux (Native)"
cargo check --workspace
cargo clippy --workspace -- -D warnings
echo "    ✓ Linux passed."

echo ""
echo ">>> [2/3] Checking Windows (x86_64-pc-windows-gnu)"
"$SCRIPT_DIR/check_windows.sh" $FLAGS
echo "    ✓ Windows passed."

echo ""
echo ">>> [3/3] Checking macOS (Intel & Apple Silicon)"
"$SCRIPT_DIR/check_macos.sh" $FLAGS
echo "    ✓ macOS passed."

echo ""
echo "======================================"
echo "   All platforms passed successfully!"
echo "======================================"
