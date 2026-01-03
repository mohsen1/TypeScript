#!/bin/bash
# Stop hook for autonomous Rust migration
# Keeps Claude working on migration tasks until explicitly stopped by user
#
# Workflow:
# 1. If uncommitted Rust changes → request Gemini review first
# 2. After review/commit → pick next tasks from migration plan
# 3. Loop indefinitely until user intervenes

REPO_ROOT="$(cd "$(dirname "${BASH_SOURCE[0]}")/../.." && pwd)"
REVIEW_MARKER="$REPO_ROOT/.claude/.review_requested"
TASK_MARKER="$REPO_ROOT/.claude/.tasks_assigned"

# Check for uncommitted Rust source files
CHANGED_RUST_FILES=$(git -C "$REPO_ROOT" diff --name-only -- 'wasm/src/*.rs' 'wasm/src/**/*.rs' 2>/dev/null | head -10 || true)
STAGED_RUST_FILES=$(git -C "$REPO_ROOT" diff --cached --name-only -- 'wasm/src/*.rs' 'wasm/src/**/*.rs' 2>/dev/null | head -10 || true)
ALL_RUST_FILES=$(printf '%s\n%s' "$CHANGED_RUST_FILES" "$STAGED_RUST_FILES" | sort -u | grep -v '^$' || true)

# PHASE 1: If uncommitted Rust files exist and not yet reviewed
if [ -n "$ALL_RUST_FILES" ] && [ ! -f "$REVIEW_MARKER" ]; then
    FILE_COUNT=$(echo "$ALL_RUST_FILES" | wc -l | tr -d ' ')
    FILE_LIST=$(echo "$ALL_RUST_FILES" | tr '\n' ' ')
    touch "$REVIEW_MARKER"

    cat << EOF
{
  "decision": "block",
  "reason": "🦀 REVIEW NEEDED: $FILE_COUNT uncommitted Rust file(s) detected.\n\n1. Run code review:\n   node scripts/ask-gemini.mjs --review $FILE_LIST\n\n2. Address any CRITICAL/MAJOR issues from the review\n\n3. Commit and push changes:\n   git add -A && git commit -m '[wasm] ...' && git push origin rust\n\n4. Then continue with next migration tasks."
}
EOF
    exit 0
fi

# PHASE 2: After review complete (or no changes), assign next tasks
# Reset markers for next cycle
rm -f "$REVIEW_MARKER" "$TASK_MARKER" 2>/dev/null

cat << 'EOF'
{
  "decision": "block",
  "reason": "🚀 AUTONOMOUS MIGRATION MODE\n\nYou've completed the current task. Continue the TypeScript→Rust migration:\n\n1. Read @rust_migration_plan.md to find the next unchecked items in '🚧 In Progress / Next Up' or '📋 Remaining Type Checker Features'\n\n2. Pick 3-5 related tasks and implement them:\n   - Write Rust code in wasm/src/checker/\n   - Add tests in wasm/src/checker/tests.rs\n   - Run: ./wasm/test.sh to verify\n\n3. After implementing, commit your changes:\n   git add -A && git commit -m '[wasm] checker: <description>'\n\n4. Update @rust_migration_plan.md to mark completed items with [x]\n\n5. Push to remote: git push origin rust\n\nKeep going until the user says stop! 🦀"
}
EOF

exit 0
