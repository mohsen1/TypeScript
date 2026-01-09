#!/bin/bash
#
# heartbeat.sh - Periodic poke to ensure the orchestration system never stops
#
# This script runs in the background and periodically sends a heartbeat
# to the director to ensure they're checking on EMs and workers.
#
# Usage:
#   .notify/heartbeat.sh [--interval <seconds>] [--session <name>]
#   .notify/heartbeat.sh --interval 300 --session zang-org  # Every 5 minutes
#
# Default: Every 5 minutes, poke director in zang-org session
#

SCRIPT_DIR="$(cd "$(dirname "$0")" && pwd)"
INTERVAL=300  # 5 minutes default
SESSION="zang-org"
DIRECTOR_PANE="director.0"

while [[ $# -gt 0 ]]; do
    case $1 in
        --interval)
            INTERVAL="$2"
            shift 2
            ;;
        --session)
            SESSION="$2"
            shift 2
            ;;
        --help)
            echo "Usage: $0 [--interval <seconds>] [--session <name>]"
            echo ""
            echo "Options:"
            echo "  --interval  Seconds between heartbeats (default: 300 = 5 min)"
            echo "  --session   Tmux session name (default: zang-org)"
            exit 0
            ;;
        *)
            echo "Unknown option: $1"
            exit 1
            ;;
    esac
done

echo "==========================================="
echo "Heartbeat Monitor Starting"
echo "==========================================="
echo "Session:      $SESSION"
echo "Interval:     ${INTERVAL}s"
echo "Director:     $SESSION:$DIRECTOR_PANE"
echo "==========================================="
echo ""

# Heartbeat messages - rotate through these
MESSAGES=(
    "HEARTBEAT: Check on your EMs. Are they making progress? Do workers need tasks?"
    "HEARTBEAT: Status check. Verify all agents are active and working on the right things."
    "HEARTBEAT: Coordination check. Any blockers? Any duplicate work happening?"
    "HEARTBEAT: Progress review. Check conformance test status and squad goals."
    "HEARTBEAT: System health check. Verify workers are not idle and EMs are coordinating."
)

MSG_INDEX=0

while true; do
    # Check if session still exists
    if ! tmux has-session -t "$SESSION" 2>/dev/null; then
        echo "[$(date -Iseconds)] Session $SESSION not found. Waiting..."
        sleep 60
        continue
    fi

    # Get current message
    MESSAGE="${MESSAGES[$MSG_INDEX]}"
    MSG_INDEX=$(( (MSG_INDEX + 1) % ${#MESSAGES[@]} ))

    # Send heartbeat to director
    echo "[$(date -Iseconds)] Sending heartbeat to $SESSION:$DIRECTOR_PANE"

    "$SCRIPT_DIR/send-prompt.sh" "$SESSION:$DIRECTOR_PANE" "$MESSAGE" 2>/dev/null || {
        echo "[$(date -Iseconds)] Failed to send heartbeat"
    }

    # Wait for next interval
    sleep "$INTERVAL"
done
