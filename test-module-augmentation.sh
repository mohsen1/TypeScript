#!/bin/bash
# Test module augmentation (cross-file interface merging)

echo "=== Testing Module Augmentation (Task 3) ==="
echo ""

# Run the WASM compiler on the test files
cd /tmp/orchestrator-workspace/worktrees/worker-1/wasm

echo "Compiling test files..."
cargo run --release -- build \
    ../tests/module-augmentation/file1.ts \
    ../tests/module-augmentation/file2.ts \
    ../tests/module-augmentation/file3.ts \
    ../tests/module-augmentation/usage.ts \
    2>&1 | grep -E "error|Error|TS2304|TS2322" || echo "No errors found - compilation successful!"

echo ""
echo "=== Test Complete ==="
