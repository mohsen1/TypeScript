#!/bin/bash
# Worker Stop Hook - Reports worker completion to orchestrator state
#
# This hook runs when a worker Claude Code session finishes.
# It writes the completion status to the shared orchestrator state file.
#
# Shared location: .orchestrator/shared-hooks/

set -euo pipefail

# Find the orchestrator directory (look for parent repo)
# In a worktree, CLAUDE_PROJECT_DIR is the worktree path
# We need to find the parent repository's .orchestrator directory
WORKTREE_DIR="$CLAUDE_PROJECT_DIR"

# Try to find .git/worktrees to determine parent repo
if [[ -f "$WORKTREE_DIR/.git" ]]; then
    GIT_FILE=$(cat "$WORKTREE_DIR/.git" 2>/dev/null || echo "")
    if [[ "$GIT_FILE" =~ gitdir:\ (.*)\.git/worktrees/ ]]; then
        # Extract parent repo path from gitdir
        PARENT_REPO="${BASH_REMATCH[1]}"
        # Remove trailing slash if present
        PARENT_REPO="${PARENT_REPO%/}"
        # Worktrees are siblings to wasm directory
        ORCHESTRATOR_DIR="$PARENT_REPO/wasm/.orchestrator"
    elif [[ -d "$WORKTREE_DIR/.git" ]]; then
        # Regular git repo, not a worktree
        ORCHESTRATOR_DIR="$WORKTREE_DIR/.orchestrator"
    else
        # Fallback: assume orchestrator is in parent directory
        ORCHESTRATOR_DIR="$WORKTREE_DIR/../.orchestrator"
    fi
else
    # Fallback: assume orchestrator is in parent directory
    ORCHESTRATOR_DIR="$WORKTREE_DIR/../.orchestrator"
fi

# Path to shared state file
STATE_FILE="$ORCHESTRATOR_DIR/state.json"
NOTIFY_DIR="$ORCHESTRATOR_DIR/notifications"

# Get current agent identity from directory name or environment
AGENT_NAME="${CLAUDE_AGENT_NAME:-$(basename "$WORKTREE_DIR")}"
TIMESTAMP=$(date -u +"%Y-%m-%dT%H:%M:%SZ")

# Read hook input to get context
INPUT=$(cat)

# Extract transcript path for analysis
TRANSCRIPT_PATH=$(echo "$INPUT" | jq -r '.transcript_path // empty')

# Determine completion status
STATUS="completed"
MESSAGE="Worker session completed"

# Update state file
update_state() {
    local tmp_file="${STATE_FILE}.tmp"

    # Create state file if it doesn't exist
    if [[ ! -f "$STATE_FILE" ]]; then
        mkdir -p "$(dirname "$STATE_FILE")"
        echo '{"version":"1.0","last_updated":null,"agents":{}}' > "$STATE_FILE"
    fi

    # Update agent status
    jq --arg agent "$AGENT_NAME" \
       --arg timestamp "$TIMESTAMP" \
       --arg status "$STATUS" \
       --arg message "$MESSAGE" \
       --arg transcript "$TRANSCRIPT_PATH" \
       '.last_updated = $timestamp |
        .agents[$agent] = {
            "status": $status,
            "message": $message,
            "updated_at": $timestamp,
            "transcript": $transcript
        }' \
       "$STATE_FILE" > "$tmp_file"

    mv "$tmp_file" "$STATE_FILE"
}

# Create notification for manager
notify_manager() {
    mkdir -p "$NOTIFY_DIR"

    # Determine squad from agent name
    if [[ "$AGENT_NAME" =~ anvil ]]; then
        SQUAD="anvil"
    elif [[ "$AGENT_NAME" =~ forge ]]; then
        SQUAD="forge"
    else
        SQUAD="unknown"
    fi

    # Write notification
    cat > "$NOTIFY_DIR/${AGENT_NAME}.json" <<EOF
{
  "timestamp": "$TIMESTAMP",
  "agent": "$AGENT_NAME",
  "squad": "$SQUAD",
  "status": "$STATUS",
  "message": "$MESSAGE"
}
EOF
}

# Main execution
update_state
notify_manager

# Log for debugging (visible in verbose mode)
echo "[Worker Stop Hook] Agent: $AGENT_NAME, Status: $STATUS, Orchestrator: $ORCHESTRATOR_DIR" >&2

exit 0
