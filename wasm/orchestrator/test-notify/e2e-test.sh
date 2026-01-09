#!/bin/bash
# E2E test of the notification system with real Codex agents
#
# This test:
# 1. Creates a test tmux session with 1 Manager + 1 Worker
# 2. Starts the notification watcher
# 3. Launches Codex agents with cheap model (gpt-4o-mini)
# 4. Worker should notify manager when it needs a task
# 5. Manager should respond and assign a task
#
# Usage: ./e2e-test.sh

set -e

ROOT_DIR="/Users/mohsenazimi/code/TypeScript"
ORCH_DIR="$ROOT_DIR/wasm/orchestrator"
NOTIFY_DIR="$ROOT_DIR/.notify"
SESSION="e2e-test"
MODEL="gpt-4o-mini"

echo "=========================================="
echo "E2E Notification System Test"
echo "=========================================="
echo "Session: $SESSION"
echo "Model:   $MODEL"
echo "=========================================="
echo ""

# Clean up
echo "[1/6] Cleaning up..."
tmux kill-session -t "$SESSION" 2>/dev/null || true
rm -f "$NOTIFY_DIR"/*.notify 2>/dev/null || true
mkdir -p "$NOTIFY_DIR"
echo "      Done"
echo ""

# Create tmux session
echo "[2/6] Creating tmux session..."
cd "$ROOT_DIR"
tmux new-session -d -s "$SESSION" -n director -c "$ROOT_DIR"
tmux split-window -h -t "$SESSION:director.0" -c "$ROOT_DIR"
tmux new-window -t "$SESSION" -n worker -c "$ROOT_DIR"
tmux select-pane -t "$SESSION:director.0" -T "manager"
tmux select-pane -t "$SESSION:director.1" -T "em-test"
tmux select-pane -t "$SESSION:worker.0" -T "worker-1"
echo "      Created: director window (manager + em) + worker window"
echo ""

# Start watcher
echo "[3/6] Starting notification watcher..."
cd "$ORCH_DIR"
npx tsx src/watcher-cli.ts --session "$SESSION" --poll-interval 500 --verbose > /tmp/watcher.log 2>&1 &
WATCHER_PID=$!
echo "      PID: $WATCHER_PID"
sleep 2
echo ""

# Start manager agent
echo "[4/6] Starting Manager agent in pane director.1..."
MANAGER_PROMPT="You are a test manager. When you receive a notification from a worker asking for a task, respond by sending them a simple task via tmux:
tmux send-keys -t $SESSION:worker.0 'echo Hello from manager! Your task: run ls -la' Enter

After sending the task, send a status notification:
$NOTIFY_DIR/notify.sh status 'Assigned task to worker-1'

Wait for notifications. Do not exit."

tmux send-keys -t "$SESSION:director.1" "export SQUAD_NAME=test && codex -m $MODEL --dangerously-bypass-approvals-and-sandbox" Enter
sleep 5
tmux send-keys -t "$SESSION:director.1" "$MANAGER_PROMPT" Enter
echo "      Manager started"
echo ""

# Start worker agent
echo "[5/6] Starting Worker agent in pane worker.0..."
WORKER_PROMPT="You are a test worker. First, notify your manager that you need a task:
$NOTIFY_DIR/notify.sh task 'Worker ready, need assignment'

Then wait for a task to appear. When you see a task, do it and notify:
$NOTIFY_DIR/notify.sh ready 'Task completed'

Then exit."

tmux send-keys -t "$SESSION:worker.0" "export SQUAD_NAME=test && export WORKER_NUM=1 && codex -m $MODEL --dangerously-bypass-approvals-and-sandbox" Enter
sleep 5
tmux send-keys -t "$SESSION:worker.0" "$WORKER_PROMPT" Enter
echo "      Worker started"
echo ""

# Monitor
echo "[6/6] Monitoring for 60 seconds..."
echo ""
echo "  Watcher log: tail -f /tmp/watcher.log"
echo "  Notifications: cat $NOTIFY_DIR/*.notify"
echo "  Attach: tmux attach -t $SESSION"
echo ""

# Wait and show results
for i in {1..12}; do
    sleep 5
    echo "--- $((i*5))s ---"
    echo "Notifications:"
    cat "$NOTIFY_DIR"/*.notify 2>/dev/null || echo "  (none yet)"
    echo ""
done

echo ""
echo "=========================================="
echo "Test Complete"
echo "=========================================="
echo ""
echo "Final notifications:"
cat "$NOTIFY_DIR"/*.notify 2>/dev/null || echo "  (none)"
echo ""
echo "Watcher log (last 20 lines):"
tail -20 /tmp/watcher.log
echo ""

# Cleanup
echo "Cleaning up..."
kill $WATCHER_PID 2>/dev/null || true
# Don't kill session so user can inspect
echo ""
echo "Session still running. Attach with: tmux attach -t $SESSION"
echo "Kill with: tmux kill-session -t $SESSION"
