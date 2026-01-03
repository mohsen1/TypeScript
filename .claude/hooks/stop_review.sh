#!/bin/bash
# Stop hook for Rust migration code review
# Runs when Claude Code finishes responding
# Blocks stopping and asks Claude to run code review if Rust files were changed

REPO_ROOT="$(cd "$(dirname "${BASH_SOURCE[0]}")/../.." && pwd)"

# Check if we already ran a review this session (prevent infinite loop)
REVIEW_MARKER="$REPO_ROOT/.claude/.review_done"
if [ -f "$REVIEW_MARKER" ]; then
    # Already reviewed, allow stopping
    rm -f "$REVIEW_MARKER"
    exit 0
fi

# Get list of changed Rust files in wasm/
CHANGED_RUST_FILES=$(git -C "$REPO_ROOT" diff --name-only HEAD -- 'wasm/*.rs' 'wasm/**/*.rs' 2>/dev/null || echo "")

# Also check staged files
STAGED_RUST_FILES=$(git -C "$REPO_ROOT" diff --cached --name-only -- 'wasm/*.rs' 'wasm/**/*.rs' 2>/dev/null || echo "")

# Combine and deduplicate
ALL_RUST_FILES=$(echo -e "${CHANGED_RUST_FILES}\n${STAGED_RUST_FILES}" | sort -u | grep -v '^$' || true)

if [ -z "$ALL_RUST_FILES" ]; then
    # No Rust files changed, exit normally
    exit 0
fi

# Count files
FILE_COUNT=$(echo "$ALL_RUST_FILES" | wc -l | tr -d ' ')

# Create marker to prevent infinite loop
touch "$REVIEW_MARKER"

# Output JSON to stdout to block and instruct Claude
# The "decision": "block" prevents Claude from stopping
# The "reason" tells Claude what to do next
cat << EOF
{
  "decision": "block",
  "reason": "🦀 Detected $FILE_COUNT changed Rust file(s). Please run a code review before completing:\n\nRun: ./scripts/ask-gemini.mjs --review ${ALL_RUST_FILES//$'\n'/ }\n\nAfter reviewing, summarize the key findings for the user."
}
EOF

exit 0
