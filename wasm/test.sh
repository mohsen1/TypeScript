#!/bin/bash
# WASM Rust Test Runner - Docker-based testing with caching
#
# This is the main test runner for the Rust/WASM TypeScript implementation.
# It uses Docker to ensure consistent testing environment and caches builds
# for fast iteration.
#
# Usage:
#   ./wasm/test.sh              # Run all Rust unit tests
#   ./wasm/test.sh test_name    # Run specific test
#   ./wasm/test.sh --rebuild    # Force rebuild Docker image
#   ./wasm/test.sh --clean      # Clean cached volumes
#   ./wasm/test.sh --bench      # Run benchmarks
#
# For TypeScript test suite conformance testing, use:
#   ./wasm/differential-test/run-conformance.sh
#
# Source code is always mounted fresh (not baked into image), so file changes
# are immediately visible without needing to rebuild the Docker image.

set -e

IMAGE_NAME="rust-wasm-base"
SCRIPT_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)"
ROOT_DIR="$(cd "$SCRIPT_DIR/.." && pwd)"

# Parse arguments
REBUILD=false
CLEAN=false
TEST_FILTER=""

for arg in "$@"; do
    case $arg in
        --rebuild)
            REBUILD=true
            ;;
        --clean)
            CLEAN=true
            ;;
        *)
            TEST_FILTER="$arg"
            ;;
    esac
done

# Clean cached volumes if requested
if [ "$CLEAN" = true ]; then
    echo "🧹 Cleaning cached volumes..."
    docker volume rm cargo-registry cargo-git wasm-target-cache 2>/dev/null || true
    echo "✅ Volumes cleaned"
    exit 0
fi

# Build base image if needed or forced (only contains rustc + nextest, not source)
if [ "$REBUILD" = true ] || ! docker image inspect "$IMAGE_NAME" &>/dev/null; then
    echo "🔨 Building base Docker image..."
    docker build -t "$IMAGE_NAME" -f - "$SCRIPT_DIR" << 'EOF'
FROM rust:latest
RUN cargo install cargo-nextest --locked
WORKDIR /app
EOF
fi

# Run tests
echo "🧪 Running tests..."

# We use a workaround: copy source to a writable location inside the container
# This avoids the ro mount conflict with the target cache
if [ -n "$TEST_FILTER" ]; then
    echo "   Filter: $TEST_FILTER"
    docker run --rm --memory="2g" --cpus="2.0" \
        -v "$SCRIPT_DIR:/source:ro" \
        -v "$ROOT_DIR:/repo:ro" \
        -v cargo-registry:/usr/local/cargo/registry \
        -v cargo-git:/usr/local/cargo/git \
        "$IMAGE_NAME" bash -c "rm -rf /app/* && cp -r /source/* /app/ && cp /repo/package.json /package.json && cargo nextest run $TEST_FILTER"
else
    docker run --rm --memory="2g" --cpus="2.0" \
        -v "$SCRIPT_DIR:/source:ro" \
        -v "$ROOT_DIR:/repo:ro" \
        -v cargo-registry:/usr/local/cargo/registry \
        -v cargo-git:/usr/local/cargo/git \
        "$IMAGE_NAME" bash -c "rm -rf /app/* && cp -r /source/* /app/ && cp /repo/package.json /package.json && cargo nextest run"
fi

echo "✅ Tests complete!"
