#!/bin/bash
# Simple notification script for agents
#
# Usage:
#   ./notify.sh ready [message]     - Signal work is ready for review
#   ./notify.sh task [message]      - Request a new task
#   ./notify.sh blocked [message]   - Signal you're blocked
#   ./notify.sh merge [message]     - Signal branch is ready to merge
#   ./notify.sh status [message]    - Send a status update
#
# Environment:
#   SQUAD_NAME  - Your squad (forge, anvil)
#   WORKER_NUM  - Your worker number (1-5)
#
# The script auto-detects the sender from environment variables.

NOTIFY_DIR="${NOTIFY_DIR:-$(dirname "$0")}"

# Determine sender
if [[ -n "$WORKER_NUM" && -n "$SQUAD_NAME" ]]; then
    SENDER="worker/${SQUAD_NAME}-${WORKER_NUM}"
elif [[ -n "$SQUAD_NAME" ]]; then
    SENDER="em/${SQUAD_NAME}"
else
    SENDER="unknown"
fi

# Parse type
TYPE="$1"
shift
MESSAGE="$*"

# Map short names to full types
case "$TYPE" in
    ready|ready_for_review|review)
        FULL_TYPE="ready_for_review"
        ;;
    task|need_task)
        FULL_TYPE="need_task"
        ;;
    blocked|block)
        FULL_TYPE="blocked"
        ;;
    merge|merge_ready)
        FULL_TYPE="merge_ready"
        ;;
    status|update|status_update)
        FULL_TYPE="status_update"
        ;;
    *)
        echo "Usage: notify.sh <type> [message]"
        echo "Types: ready, task, blocked, merge, status"
        exit 1
        ;;
esac

# Create notification file
SAFE_SENDER=$(echo "$SENDER" | tr '/' '-')
NOTIFY_FILE="${NOTIFY_DIR}/${SAFE_SENDER}.notify"

# Generate JSON notification
TIMESTAMP=$(date -u +"%Y-%m-%dT%H:%M:%SZ")

if [[ -n "$MESSAGE" ]]; then
    JSON="{\"timestamp\":\"${TIMESTAMP}\",\"sender\":\"${SENDER}\",\"type\":\"${FULL_TYPE}\",\"message\":\"${MESSAGE}\"}"
else
    JSON="{\"timestamp\":\"${TIMESTAMP}\",\"sender\":\"${SENDER}\",\"type\":\"${FULL_TYPE}\"}"
fi

# Append to file
echo "$JSON" >> "$NOTIFY_FILE"

echo "Notification sent: $FULL_TYPE from $SENDER"
