#!/bin/bash
# Test the notification system with a minimal orchestrator setup
#
# This script:
# 1. Creates a test tmux session with Director, Manager, and Worker panes
# 2. Starts the notification watcher
# 3. Launches agents with cheap/fast models
#
# Usage:
#   ./run-test.sh              # Uses codex with gpt-4o-mini (cheap)
#   ./run-test.sh claude       # Uses claude with haiku
#   ./run-test.sh --dry-run    # Just creates the tmux session, no agents

set -e

SCRIPT_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)"
ROOT_DIR="$(cd "$SCRIPT_DIR/../../../.." && pwd)"
NOTIFY_DIR="$ROOT_DIR/.notify"
SESSION="test-org"
AGENT_TYPE="${1:-codex}"
DRY_RUN=false

if [[ "$1" == "--dry-run" ]]; then
    DRY_RUN=true
fi

# Determine agent command and model
if [[ "$AGENT_TYPE" == "claude" ]]; then
    AGENT_CMD="claude --dangerously-skip-permissions --model haiku"
    echo "Using Claude with Haiku model (fast/cheap)"
else
    # Use gpt-4o-mini for cheap testing
    AGENT_CMD="codex -m gpt-4o-mini --dangerously-bypass-approvals-and-sandbox"
    echo "Using Codex with gpt-4o-mini model (fast/cheap)"
fi

# Clean up old notifications
rm -rf "$NOTIFY_DIR"/*.notify 2>/dev/null || true
mkdir -p "$NOTIFY_DIR"

# Kill existing session if exists
tmux kill-session -t "$SESSION" 2>/dev/null || true

echo "Creating test tmux session: $SESSION"

# Create session with director window
tmux new-session -d -s "$SESSION" -n director -c "$ROOT_DIR"

# Split for manager (pane 1)
tmux split-window -h -t "$SESSION:director.0" -c "$ROOT_DIR"

# Create test squad window with 2 workers
tmux new-window -t "$SESSION" -n test -c "$ROOT_DIR"
tmux split-window -h -t "$SESSION:test.0" -c "$ROOT_DIR"

# Set pane titles
tmux select-pane -t "$SESSION:director.0" -T "director"
tmux select-pane -t "$SESSION:director.1" -T "em-test"
tmux select-pane -t "$SESSION:test.0" -T "test-1"
tmux select-pane -t "$SESSION:test.1" -T "test-2"

# Select director window
tmux select-window -t "$SESSION:director"

echo ""
echo "Test session created!"
echo "  Session: $SESSION"
echo "  Director: $SESSION:director.0"
echo "  Manager:  $SESSION:director.1"
echo "  Worker 1: $SESSION:test.0"
echo "  Worker 2: $SESSION:test.1"
echo ""

if $DRY_RUN; then
    echo "Dry run mode - not starting agents"
    echo "Attach with: tmux attach -t $SESSION"
    exit 0
fi

# Start the notification watcher in background
echo "Starting notification watcher..."
cd "$SCRIPT_DIR/.."
npx tsx src/watcher-cli.ts --session "$SESSION" --verbose &
WATCHER_PID=$!
echo "Watcher PID: $WATCHER_PID"
sleep 2

# Function to start an agent in a pane
start_agent() {
    local PANE="$1"
    local ROLE="$2"
    local SQUAD="$3"
    local WORKER_NUM="$4"
    local AGENT_FILE="$5"

    echo "Starting $ROLE in $PANE..."

    # Set environment variables
    local ENV_CMD=""
    if [[ -n "$SQUAD" ]]; then
        ENV_CMD="export SQUAD_NAME=$SQUAD; "
    fi
    if [[ -n "$WORKER_NUM" ]]; then
        ENV_CMD="${ENV_CMD}export WORKER_NUM=$WORKER_NUM; "
    fi
    ENV_CMD="${ENV_CMD}export NOTIFY_DIR=$NOTIFY_DIR"

    # Build the full command
    local CMD="${ENV_CMD} && $AGENT_CMD"

    tmux send-keys -t "$PANE" "$CMD" Enter
    sleep 3

    # Send the initial prompt
    local PROMPT="Read $AGENT_FILE for your instructions. Then start working."
    tmux send-keys -t "$PANE" "$PROMPT" Enter
}

# Start agents
start_agent "$SESSION:director.0" "director" "" "" "$SCRIPT_DIR/AGENT_DIRECTOR.md"
sleep 2
start_agent "$SESSION:director.1" "manager" "test" "" "$SCRIPT_DIR/AGENT_MANAGER.md"
sleep 2
start_agent "$SESSION:test.0" "worker" "test" "1" "$SCRIPT_DIR/AGENT_WORKER.md"
sleep 2
start_agent "$SESSION:test.1" "worker" "test" "2" "$SCRIPT_DIR/AGENT_WORKER.md"

echo ""
echo "=========================================="
echo "Test environment ready!"
echo "=========================================="
echo ""
echo "Attach: tmux attach -t $SESSION"
echo "Kill:   tmux kill-session -t $SESSION; kill $WATCHER_PID"
echo ""
echo "Watch notifications: tail -f $NOTIFY_DIR/*.notify"
echo ""
echo "The agents should now:"
echo "1. Workers will ask for tasks (via notify.sh)"
echo "2. Manager will receive notifications and assign tasks"
echo "3. Director will receive status updates from manager"
echo ""
