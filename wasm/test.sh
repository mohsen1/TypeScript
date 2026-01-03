#!/bin/bash
# Fast Rust test runner with Docker caching
#
# Usage:
#   ./test.sh              # Run all tests
#   ./test.sh test_name    # Run specific test
#   ./test.sh --rebuild    # Force rebuild image
#   ./test.sh --clean      # Clean cached volumes

set -e

IMAGE_NAME="rust-wasm-tests"
SCRIPT_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)"

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
    docker volume rm cargo-registry cargo-git 2>/dev/null || true
    echo "✅ Volumes cleaned"
    exit 0
fi

# Build image if needed or forced
if [ "$REBUILD" = true ] || ! docker image inspect "$IMAGE_NAME" &>/dev/null; then
    echo "🔨 Building Docker image with BuildKit caching..."
    DOCKER_BUILDKIT=1 docker build -t "$IMAGE_NAME" "$SCRIPT_DIR"
fi

# Run tests
echo "🧪 Running tests..."
if [ -n "$TEST_FILTER" ]; then
    echo "   Filter: $TEST_FILTER"
    docker run --rm --memory="1g" --cpus="2.0" \
        -v cargo-registry:/usr/local/cargo/registry \
        -v cargo-git:/usr/local/cargo/git \
        "$IMAGE_NAME" cargo nextest run "$TEST_FILTER"
else
    docker run --rm --memory="1g" --cpus="2.0" \
        -v cargo-registry:/usr/local/cargo/registry \
        -v cargo-git:/usr/local/cargo/git \
        "$IMAGE_NAME"
fi

echo "✅ Tests complete!"
