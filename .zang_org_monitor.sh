SESSION="zang-org"
ROOT_DIR=/Users/mohsenazimi/code/TypeScript
SQUAD_DIR=/Users/mohsenazimi/code/TypeScript/wasm/specs/squads

DIRECTOR_IDLE_SECONDS=180  # Check every 3 minutes (was 10 min)
DIRECTOR_POKE="work"  # Simple command to run director loop
EM_IDLE_SECONDS=120  # Check every 2 minutes (was 1 min, too aggressive)
EM_POKE="Check all 5 worker panes now. For each worker: (1) Capture pane, (2) Categorize as active/merge-ready/blocked. For ANY merge-ready workers: IMMEDIATELY assign new task from GOALS.md backlog, update plan file, commit, push, send worker prompt. Report format: 'W1: [task] - [status] (if was merge-ready: → NOW ASSIGNED [new task])'."
WORKER_IDLE_SECONDS=300  # Check every 5 minutes
WORKER_POKE="Continue with your current task. If you're blocked or done, update your plan file and mark status."
SEND_ENTER_PAUSE=1

STATE_DIR="/tmp/zang-org-monitor-$$"
mkdir -p "$STATE_DIR"
trap "rm -rf '$STATE_DIR'" EXIT

get_idle_threshold() {
  local window="$1"
  local pane="$2"

  if [ "$window" = "director" ] && [ "$pane" = "0" ]; then
    # Director is pane 0 in director window
    echo "$DIRECTOR_IDLE_SECONDS"
  elif [ "$window" = "director" ] && [ "$pane" != "0" ]; then
    # EMs are panes 1 and 2 in director window
    echo "$EM_IDLE_SECONDS"
  elif [ "$window" = "forge" ] || [ "$window" = "anvil" ]; then
    # Workers are in forge/anvil windows
    echo "$WORKER_IDLE_SECONDS"
  else
    echo "$WORKER_IDLE_SECONDS"
  fi
}

get_poke_message() {
  local window="$1"
  local pane="$2"

  if [ "$window" = "director" ] && [ "$pane" = "0" ]; then
    # Director is pane 0 in director window
    echo "$DIRECTOR_POKE"
  elif [ "$window" = "director" ] && [ "$pane" != "0" ]; then
    # EMs are panes 1 and 2 in director window
    echo "$EM_POKE"
  elif [ "$window" = "forge" ] || [ "$window" = "anvil" ]; then
    # Workers are in forge/anvil windows
    echo "$WORKER_POKE"
  else
    echo "$WORKER_POKE"
  fi
}

get_last_hash() {
  local key="$1"
  local file="$STATE_DIR/hash_${key//[^a-zA-Z0-9]/_}"
  [ -f "$file" ] && cat "$file" || echo ""
}

set_last_hash() {
  local key="$1"
  local val="$2"
  local file="$STATE_DIR/hash_${key//[^a-zA-Z0-9]/_}"
  echo "$val" > "$file"
}

get_last_change() {
  local key="$1"
  local file="$STATE_DIR/change_${key//[^a-zA-Z0-9]/_}"
  [ -f "$file" ] && cat "$file" || echo ""
}

set_last_change() {
  local key="$1"
  local val="$2"
  local file="$STATE_DIR/change_${key//[^a-zA-Z0-9]/_}"
  echo "$val" > "$file"
}

is_actively_working() {
  local content="$1"
  local last_lines

  # Get last 10 lines
  last_lines=$(echo "$content" | tail -10)

  # Check for active work indicators
  if echo "$last_lines" | grep -qE "(Running|Compiling|Analyzing|Working|Building|Testing|• Ran|• Read|• Explored|Updating|esc to interrupt|Processing|Executing)"; then
    return 0  # Is working
  fi

  # Check for prompts waiting (but not if there's a spinner/activity)
  if echo "$last_lines" | grep -qE "^›|⌥ \+ ↑ edit" && ! echo "$last_lines" | grep -qE "esc to interrupt"; then
    return 1  # Idle at prompt
  fi

  # Check for recent timestamps (within last minute means active)
  if echo "$last_lines" | grep -qE "[0-9]+m [0-9]+s|Worked for"; then
    # Has timestamp - check if content changed recently
    return 0  # Assume working if has recent timestamp
  fi

  return 1  # Default to idle
}

has_unanswered_prompt() {
  local content="$1"
  local last_lines

  # Get last 5 lines
  last_lines=$(echo "$content" | tail -5)

  # Check if there's already a prompt without activity after it
  if echo "$last_lines" | tail -3 | grep -qE "^› |^  ↳ |New assignment:|Assigned:"; then
    return 0  # Has unanswered prompt
  fi

  return 1  # No unanswered prompt
}

while tmux has-session -t "$SESSION" 2>/dev/null; do
  now=$(date +%s)

  for window in director forge anvil; do
    if ! tmux list-windows -t "$SESSION" -F "#{window_name}" | grep -qx "$window"; then
      continue
    fi

    pane_count=$(tmux list-panes -t "$SESSION:$window" | wc -l | tr -d ' ')

    pane=0
    while [ "$pane" -lt "$pane_count" ]; do
      key="${window}_${pane}"

      content=$(tmux capture-pane -p -t "$SESSION:$window.$pane" -S -200 2>/dev/null || true)
      if [ -z "$content" ]; then
        pane=$((pane + 1))
        continue
      fi

      # Skip if actively working
      if is_actively_working "$content"; then
        set_last_change "$key" "$now"
        pane=$((pane + 1))
        continue
      fi

      # Skip if there's already an unanswered prompt
      if has_unanswered_prompt "$content"; then
        # Don't spam more prompts
        pane=$((pane + 1))
        continue
      fi

      hash=$(printf "%s" "$content" | cksum | awk '{print $1}')
      last_hash=$(get_last_hash "$key")

      if [ "$last_hash" != "$hash" ]; then
        set_last_hash "$key" "$hash"
        set_last_change "$key" "$now"
        pane=$((pane + 1))
        continue
      fi

      last_seen=$(get_last_change "$key")
      [ -z "$last_seen" ] && last_seen="$now"
      idle=$((now - last_seen))
      threshold=$(get_idle_threshold "$window" "$pane")

      if [ "$threshold" -gt 0 ] && [ "$idle" -ge "$threshold" ]; then
        poke=$(get_poke_message "$window" "$pane")
        tmux send-keys -t "$SESSION:$window.$pane" "$poke"; sleep 2  # CRITICAL: Wait for Codex to process
        sleep "$SEND_ENTER_PAUSE"
        tmux send-keys -t "$SESSION:$window.$pane" C-m
        set_last_change "$key" "$now"
      fi

      pane=$((pane + 1))
    done
  done

  sleep 10  # Increased from 5 to reduce check frequency
done
