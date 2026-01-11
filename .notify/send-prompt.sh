#!/bin/bash
#
# send-prompt.sh - Send a prompt to a Codex agent in a tmux pane
#
# This script handles the timing requirements for Codex:
# 1. Send the text
# 2. Wait 2 seconds (CRITICAL: Codex needs time to process)
# 3. Send Enter key
#
# Usage:
#   .notify/send-prompt.sh <pane> <message>
#

set -e

if [[ $# -lt 2 ]]; then
    echo "Usage: $0 <pane> <message>"
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

# CRITICAL: Wait 2 seconds for Codex to process
# This is the core fix - 1 second was too short
sleep 2

# Send Enter key
tmux send-keys -t "$PANE" Enter

echo "Sent prompt to $PANE"
