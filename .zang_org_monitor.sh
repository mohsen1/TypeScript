SESSION="zang-org"
ROOT_DIR=/Users/mohsenazimi/code/TypeScript
SQUAD_DIR=/Users/mohsenazimi/code/TypeScript/wasm/specs/squads

DIRECTOR_IDLE_SECONDS=600
DIRECTOR_POKE="MERGE TIME! Tell both EMs to pause work and push branches. Wait 4 minutes for them to finish, then merge squad/forge and squad/anvil into rust. After merging, tell EMs to sync workers from origin/rust."
EM_IDLE_SECONDS=60
EM_POKE="Check worker panes for stuck workers. If all workers are busy, check for Ready for Merge branches and merge them into squad branch."
WORKER_IDLE_SECONDS=300
WORKER_POKE="How is your task going? If you need help, describe what you are stuck on."
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
        tmux send-keys -t "$SESSION:$window.$pane" "$poke"
        sleep "$SEND_ENTER_PAUSE"
        tmux send-keys -t "$SESSION:$window.$pane" C-m
        set_last_change "$key" "$now"
      fi

      pane=$((pane + 1))
    done
  done

  sleep 5
done
