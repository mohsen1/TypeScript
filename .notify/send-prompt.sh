#!/bin/bash
#
# send-prompt.sh - Send a prompt to a Codex agent in a tmux pane
#
# This script handles the timing requirements for Codex:
# 1. Send the text
# 2. Wait 1 second
# 3. Send Enter key
#
# Usage:
#   .notify/send-prompt.sh <pane> <message>
#   .notify/send-prompt.sh zang-org:forge.0 "Your task: implement feature X"
#   .notify/send-prompt.sh zang-org:director.1 "Check worker status"
#
# Pane format: session:window.pane
#   - zang-org:forge.0    = forge window, pane 0 (worker 1)
#   - zang-org:forge.1    = forge window, pane 1 (worker 2)
#   - zang-org:director.0 = director pane
#   - zang-org:director.1 = EM-Forge pane
#   - zang-org:director.2 = EM-Anvil pane
#

set -e

if [[ $# -lt 2 ]]; then
    echo "Usage: $0 <pane> <message>"
    echo ""
    echo "Examples:"
    echo "  $0 zang-org:forge.0 'Your task: implement X'"
    echo "  $0 zang-org:director.1 'Check worker status'"
    echo ""
    echo "Common panes:"
    echo "  zang-org:director.0  - Director"
    echo "  zang-org:director.1  - EM-Forge"
    echo "  zang-org:director.2  - EM-Anvil"
    echo "  zang-org:forge.0-3   - Forge workers 1-4"
    echo "  zang-org:anvil.0-3   - Anvil workers 1-4"
    exit 1
fi

PANE="$1"
shift
MESSAGE="$*"

# Verify the pane exists
if ! tmux has-session -t "${PANE%%:*}" 2>/dev/null; then
    echo "Error: Session '${PANE%%:*}' does not exist"
    exit 1
fi

# Send the message text
tmux send-keys -t "$PANE" "$MESSAGE"

# Wait 1 second (required for Codex to process)
sleep 1

# Send Enter key
tmux send-keys -t "$PANE" Enter

echo "Sent prompt to $PANE"
