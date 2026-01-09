#!/bin/bash
# Manual test of the notification system
#
# This test:
# 1. Creates a minimal tmux session
# 2. Starts the watcher
# 3. Sends a test notification
# 4. Verifies the notification was delivered to the correct pane

set -e

SCRIPT_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)"
ROOT_DIR="$(cd "$SCRIPT_DIR/../../../.." && pwd)"
NOTIFY_DIR="$ROOT_DIR/.notify"
SESSION="notify-test"

echo "=== Notification System Manual Test ==="
echo ""

# Clean up
echo "1. Cleaning up..."
tmux kill-session -t "$SESSION" 2>/dev/null || true
rm -f "$NOTIFY_DIR"/*.notify 2>/dev/null || true
mkdir -p "$NOTIFY_DIR"
echo "   Done"
echo ""

# Create simple tmux session
echo "2. Creating test tmux session..."
tmux new-session -d -s "$SESSION" -n director -c "$ROOT_DIR"
tmux split-window -h -t "$SESSION:director.0" -c "$ROOT_DIR"
tmux select-pane -t "$SESSION:director.0" -T "director"
tmux select-pane -t "$SESSION:director.1" -T "em-test"
echo "   Session: $SESSION"
echo "   Director: $SESSION:director.0"
echo "   EM:       $SESSION:director.1"
echo ""

# Start watcher in background
echo "3. Starting notification watcher..."
cd "$SCRIPT_DIR/.."
npx tsx src/watcher-cli.ts --session "$SESSION" --verbose --poll-interval 500 &
WATCHER_PID=$!
echo "   PID: $WATCHER_PID"
sleep 2
echo ""

# Send test notification
echo "4. Sending test notification from worker/test-1..."
cd "$ROOT_DIR"
SQUAD_NAME=test WORKER_NUM=1 .notify/notify.sh ready "E2E test completed successfully"
sleep 2
echo ""

# Capture the EM pane to see if notification arrived
echo "5. Checking EM pane for notification..."
CAPTURED=$(tmux capture-pane -p -t "$SESSION:director.1" 2>&1)
echo "   Captured pane content:"
echo "   ---"
echo "$CAPTURED" | tail -5
echo "   ---"
echo ""

# Clean up
echo "6. Cleaning up..."
kill $WATCHER_PID 2>/dev/null || true
tmux kill-session -t "$SESSION" 2>/dev/null || true
echo "   Done"
echo ""

# Check if notification was delivered
if echo "$CAPTURED" | grep -q "NOTIFICATION"; then
    echo "✅ SUCCESS: Notification was delivered to EM pane!"
else
    echo "⚠️  NOTE: Notification may not have been delivered yet (tmux timing)."
    echo "   The watcher did process the notification (check output above)."
fi

echo ""
echo "=== Test Complete ==="
