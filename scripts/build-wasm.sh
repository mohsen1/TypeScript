#!/bin/bash
# Build WASM bindings for the Zang TypeScript compiler

set -e

# Navigate to workspace root
SCRIPT_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)"
cd "$SCRIPT_DIR/.."

echo "Building WASM module..."

# Check if wasm-pack is installed
if ! command -v wasm-pack &> /dev/null; then
    echo "wasm-pack not found. Installing..."
    cargo install wasm-pack
fi

# Build the WASM module
cd crates/zang_wasm

# Build for bundler (webpack, rollup, etc.)
wasm-pack build --target bundler --out-dir ../../pkg/bundler

# Build for Node.js
wasm-pack build --target nodejs --out-dir ../../pkg/nodejs

# Build for web (browser without bundler)
wasm-pack build --target web --out-dir ../../pkg/web

echo "WASM build complete!"
echo "Output directories:"
echo "  - pkg/bundler  (for webpack/rollup)"
echo "  - pkg/nodejs   (for Node.js)"
echo "  - pkg/web      (for browsers)"
