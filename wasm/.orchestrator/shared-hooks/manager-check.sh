#!/bin/bash
# Manager Check Hook - Checks for worker updates on each prompt
#
# This hook runs when a manager (EM or Director) receives a prompt.
# It checks the orchestrator state for updates and injects context if there's work.
#
# Shared location: .orchestrator/shared-hooks/

set -euo pipefail

# Find the orchestrator directory (look for parent repo)
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

STATE_FILE="$ORCHESTRATOR_DIR/state.json"
NOTIFY_DIR="$ORCHESTRATOR_DIR/notifications"
AGENT_NAME="${CLAUDE_AGENT_NAME:-$(basename "$WORKTREE_DIR")}"

# Read hook input to get the user's prompt
INPUT=$(cat)
USER_PROMPT=$(echo "$INPUT" | jq -r '.prompt // empty')

# Check if state file exists
if [[ ! -f "$STATE_FILE" ]]; then
    exit 0  # No state yet, nothing to do
fi

# Determine which squad this manager oversees
if [[ "$AGENT_NAME" =~ anvil ]]; then
    SQUAD="anvil"
elif [[ "$AGENT_NAME" =~ forge ]]; then
    SQUAD="forge"
elif [[ "$AGENT_NAME" =~ director ]] || [[ "$AGENT_NAME" =~ em ]]; then
    SQUAD="all"
else
    exit 0  # Unknown manager type, skip
fi

# Find agents needing attention
find_updates() {
    local updates=""

    # Check for notification files
    if [[ -d "$NOTIFY_DIR" ]]; then
        for notify_file in "$NOTIFY_DIR"/*.json; do
            if [[ -f "$notify_file" ]]; then
                local agent=$(jq -r '.agent // empty' "$notify_file" 2>/dev/null || true)
                local squad=$(jq -r '.squad // empty' "$notify_file" 2>/dev/null || true)
                local status=$(jq -r '.status // empty' "$notify_file" 2>/dev/null || true)
                local message=$(jq -r '.message // empty' "$notify_file" 2>/dev/null || true)
                local timestamp=$(jq -r '.timestamp // empty' "$notify_file" 2>/dev/null || true)

                # Check if this notification is relevant to this manager
                if [[ "$SQUAD" == "all" ]] || [[ "$squad" == "$SQUAD" ]]; then
                    updates="${updates}- [${timestamp}] ${agent}: ${status} - ${message}"$'\n'

                    # Mark as processed by moving to processed directory
                    mkdir -p "$NOTIFY_DIR/processed"
                    mv "$notify_file" "$NOTIFY_DIR/processed/${timestamp}_${agent}.json" 2>/dev/null || true
                fi
            fi
        done
    fi

    echo "$updates"
}

UPDATES=$(find_updates)

if [[ -n "$UPDATES" ]]; then
    # Output context that will be added to the conversation
    # Plain text stdout is added as context for UserPromptSubmit
    printf "ORCHESTRATOR UPDATES:\n%s\n" "$UPDATES"

    # Also log for debugging
    echo "[Manager Check Hook] Found ${SQUAD} updates for ${AGENT_NAME}, Orchestrator: $ORCHESTRATOR_DIR" >&2
fi

exit 0
