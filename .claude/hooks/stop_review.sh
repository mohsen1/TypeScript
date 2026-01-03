#!/bin/bash
# Stop hook for Rust migration code review
# Runs when Claude Code finishes responding
# Only triggers ONCE per session to request a code review, then allows stopping

REPO_ROOT="$(cd "$(dirname "${BASH_SOURCE[0]}")/../.." && pwd)"
REVIEW_MARKER="$REPO_ROOT/.claude/.review_requested"

# If we already requested a review this session, always allow stopping
# The marker persists until manually deleted or a new session starts
if [ -f "$REVIEW_MARKER" ]; then
    exit 0
fi

# Get list of changed Rust files (uncommitted)
CHANGED_RUST_FILES=$(git -C "$REPO_ROOT" diff --name-only -- 'wasm/src/*.rs' 'wasm/src/**/*.rs' 2>/dev/null | head -10 || true)
STAGED_RUST_FILES=$(git -C "$REPO_ROOT" diff --cached --name-only -- 'wasm/src/*.rs' 'wasm/src/**/*.rs' 2>/dev/null | head -10 || true)

# Combine and deduplicate
ALL_RUST_FILES=$(printf '%s\n%s' "$CHANGED_RUST_FILES" "$STAGED_RUST_FILES" | sort -u | grep -v '^$' || true)

if [ -z "$ALL_RUST_FILES" ]; then
    # No uncommitted Rust files, allow stopping
    exit 0
fi

# Count files
FILE_COUNT=$(echo "$ALL_RUST_FILES" | wc -l | tr -d ' ')

# Create marker - this persists for the session
touch "$REVIEW_MARKER"

# Format file list for command
FILE_LIST=$(echo "$ALL_RUST_FILES" | tr '\n' ' ')

# Output JSON to block and request review
cat << EOF
{
  "decision": "block",
  "reason": "🦀 Detected $FILE_COUNT uncommitted Rust file(s). Please run code review before completing:\n\nnode scripts/ask-gemini.mjs --review $FILE_LIST\n\nAfter review, address comments, commit changes, then continue or stop."
}
EOF

exit 0
