#!/usr/bin/env bash
set -euo pipefail

SESSION_MAIN="zang-hub"
WINDOW_MAIN="hub"
CODEX_CMD="${CODEX_CMD:-codex}"
CODEX_ARGS="${CODEX_ARGS---full-auto}"
CODEX_MANAGER_ARGS="${CODEX_MANAGER_ARGS:-$CODEX_ARGS}"
CODEX_TRACK_ARGS="${CODEX_TRACK_ARGS:-$CODEX_ARGS}"
AUTO_FETCH="${AUTO_FETCH:-1}"
AUTO_RESTART_CODEX="${AUTO_RESTART_CODEX:-1}"
CODEX_RESTART_DELAY="${CODEX_RESTART_DELAY:-2}"
CODEX_AUTO_UPDATE="${CODEX_AUTO_UPDATE:-1}"
CODEX_UPDATE_CMD="${CODEX_UPDATE_CMD:-npm install -g @openai/codex}"
MANAGER_IDLE_SECONDS="${MANAGER_IDLE_SECONDS:-60}"
MANAGER_POKE="${MANAGER_POKE:-Continue managing}"
TRACK_IDLE_SECONDS="${TRACK_IDLE_SECONDS:-60}"
TRACK_POKE="${TRACK_POKE:-continue with your plan.}"
TRACK_START_PROMPT="${TRACK_START_PROMPT:-$TRACK_POKE}"
TRACK_START_PAUSE="${TRACK_START_PAUSE:-15}"
TRACK_LIMIT="${TRACK_LIMIT:-5}"
AUTO_ATTACH="${AUTO_ATTACH:-1}"

if [ "${1:-}" = "--kill" ]; then
  if tmux has-session -t "$SESSION_MAIN" 2>/dev/null; then
    tmux kill-session -t "$SESSION_MAIN"
    tmux set-option -g -u "@zang_auto_monitor" 2>/dev/null || true
    echo "Killed tmux session: $SESSION_MAIN"
  else
    echo "No tmux session found: $SESSION_MAIN"
  fi
  exit 0
fi

BASE_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)"

ROOT_DIR="$BASE_DIR"
WORKTREE_BASE_DEFAULT=""
if [ -d "$BASE_DIR/TypeScript" ]; then
  ROOT_DIR="$BASE_DIR/TypeScript"
  WORKTREE_BASE_DEFAULT="$BASE_DIR"
else
  WORKTREE_BASE_DEFAULT="$(dirname "$BASE_DIR")"
fi
WORKTREE_BASE="${WORKTREE_BASE:-$WORKTREE_BASE_DEFAULT}"

if [ ! -d "$ROOT_DIR" ]; then
  echo "error: TypeScript root not found" >&2
  exit 1
fi

if [ ! -d "$WORKTREE_BASE" ]; then
  echo "error: worktree base not found: $WORKTREE_BASE" >&2
  exit 1
fi

if ! command -v tmux >/dev/null 2>&1; then
  echo "error: tmux not found in PATH" >&2
  exit 1
fi

if ! command -v git >/dev/null 2>&1; then
  echo "error: git not found in PATH" >&2
  exit 1
fi

if ! command -v "$CODEX_CMD" >/dev/null 2>&1; then
  echo "error: $CODEX_CMD not found in PATH" >&2
  exit 1
fi

if [ "$CODEX_AUTO_UPDATE" = "1" ]; then
  if command -v npm >/dev/null 2>&1; then
    echo "Updating Codex..."
    if ! bash -lc "$CODEX_UPDATE_CMD" >/dev/null 2>&1; then
      echo "warning: Codex update failed; continuing" >&2
    fi
  else
    echo "warning: npm not found; skipping Codex update" >&2
  fi
fi

if ! git -C "$ROOT_DIR" rev-parse --is-inside-work-tree >/dev/null 2>&1; then
  echo "error: $ROOT_DIR is not a git repo" >&2
  exit 1
fi

SPEC_DIR="$ROOT_DIR/wasm/specs"
if [ ! -d "$SPEC_DIR" ]; then
  echo "error: $SPEC_DIR not found" >&2
  exit 1
fi

PLAN_FILES=()
while IFS= read -r plan; do
  PLAN_FILES+=("$plan")
done < <(find "$SPEC_DIR" -maxdepth 1 -type f -name "*_plan.md" | sort)
if [ ${#PLAN_FILES[@]} -eq 0 ]; then
  echo "error: no *_plan.md files found in $SPEC_DIR" >&2
  exit 1
fi

plan_is_complete() {
  local plan="$1"
  grep -Eiq '^[[:space:]]*(Status|Track Status|State)[[:space:]]*:[[:space:]]*(Complete|Completed|Done)[[:space:]]*$' "$plan"
}

plan_priority() {
  local plan="$1"
  local value
  value="$(awk -F: 'BEGIN{IGNORECASE=1} /^[[:space:]]*priority[[:space:]]*:/ {print $2; exit}' "$plan" | tr -cd '0-9-')"
  if [ -z "$value" ]; then
    echo 100
  else
    echo "$value"
  fi
}

PLAN_META=()
for plan in "${PLAN_FILES[@]}"; do
  if plan_is_complete "$plan"; then
    continue
  fi
  base="$(basename "$plan")"
  name="${base%_plan.md}"
  priority="$(plan_priority "$plan")"
  PLAN_META+=("${priority}|${name}|${plan}")
done

ACTIVE_META=()
if [ ${#PLAN_META[@]} -gt 0 ]; then
  IFS=$'\n' sorted=($(printf '%s\n' "${PLAN_META[@]}" | sort -t '|' -k1,1n -k2,2))
  unset IFS
  for entry in "${sorted[@]}"; do
    ACTIVE_META+=("$entry")
    if [ ${#ACTIVE_META[@]} -ge "$TRACK_LIMIT" ]; then
      break
    fi
  done
fi

TRACK_NAMES=()
TRACK_DIRS=()
for entry in "${ACTIVE_META[@]}"; do
  rest="${entry#*|}"
  name="${rest%%|*}"
  TRACK_NAMES+=("$name")
  TRACK_DIRS+=("$WORKTREE_BASE/TypeScript-${name}-track")
done

if [ "$AUTO_FETCH" = "1" ]; then
  git -C "$ROOT_DIR" fetch --prune origin rust >/dev/null 2>&1 || true
fi

if ! git -C "$ROOT_DIR" show-ref --verify --quiet refs/heads/rust; then
  if git -C "$ROOT_DIR" show-ref --verify --quiet refs/remotes/origin/rust; then
    git -C "$ROOT_DIR" branch --track rust origin/rust >/dev/null 2>&1 || true
  else
    git -C "$ROOT_DIR" branch rust >/dev/null 2>&1 || true
  fi
fi

is_worktree() {
  local path="$1"
  git -C "$ROOT_DIR" worktree list --porcelain | awk '/^worktree /{print $2}' | grep -Fx "$path" >/dev/null 2>&1
}

for dir in "${TRACK_DIRS[@]}"; do
  if [ -d "$dir" ]; then
    if ! is_worktree "$dir"; then
      echo "warning: $dir exists but is not a git worktree; skipping" >&2
    fi
    continue
  fi
  git -C "$ROOT_DIR" worktree add --force "$dir" rust >/dev/null 2>&1
done

MANAGER_DIR="$BASE_DIR"
if [ "$ROOT_DIR" = "$BASE_DIR" ]; then
  MANAGER_DIR="$(dirname "$ROOT_DIR")"
fi

if [ ! -f "$MANAGER_DIR/MANAGER_AGENT.md" ] && [ -f "$ROOT_DIR/MANAGER_AGENT.md" ]; then
  ln -s "$ROOT_DIR/MANAGER_AGENT.md" "$MANAGER_DIR/MANAGER_AGENT.md"
fi
if [ -f "$MANAGER_DIR/MANAGER_AGENT.md" ] && [ ! -e "$MANAGER_DIR/AGENTS.md" ]; then
  ln -s "MANAGER_AGENT.md" "$MANAGER_DIR/AGENTS.md"
elif [ ! -f "$MANAGER_DIR/MANAGER_AGENT.md" ]; then
  echo "warning: MANAGER_AGENT.md not found in $MANAGER_DIR" >&2
fi

manager_cmd="${CODEX_CMD} ${CODEX_MANAGER_ARGS}"
track_cmd="${CODEX_CMD} ${CODEX_TRACK_ARGS}"
if [ "$AUTO_RESTART_CODEX" = "1" ]; then
  manager_cmd="while true; do ${manager_cmd}; sleep ${CODEX_RESTART_DELAY}; done"
  track_cmd="while true; do ${track_cmd}; sleep ${CODEX_RESTART_DELAY}; done"
fi

if [ "$TRACK_LIMIT" -gt 5 ]; then
  echo "warning: TRACK_LIMIT > 5 may exceed 3x2 layout; consider reducing." >&2
fi

largest_pane_id() {
  tmux list-panes -t "$SESSION_MAIN:$WINDOW_MAIN" -F "#{pane_id} #{pane_width} #{pane_height}" \
    | awk '{print $1, $2*$3}' | sort -k2,2nr | awk 'NR==1{print $1}'
}

if ! tmux has-session -t "$SESSION_MAIN" 2>/dev/null; then
  tmux new-session -d -s "$SESSION_MAIN" -n "$WINDOW_MAIN" -c "$MANAGER_DIR" \
    "bash" "-lc" "$manager_cmd"
  manager_pane_id="$(tmux display-message -p -t "$SESSION_MAIN:$WINDOW_MAIN.0" "#{pane_id}")"
  tmux select-pane -t "$manager_pane_id" -T "manager" 2>/dev/null || true
  tmux set-option -g "@zang_manager_pane" "$manager_pane_id"
  track_count="${#TRACK_DIRS[@]}"
  if [ "$track_count" -eq 5 ]; then
    dir="${TRACK_DIRS[0]}"
    name="${TRACK_NAMES[0]}"
    right_pane_id="$(tmux split-window -h -t "$manager_pane_id" -c "$dir" -P -F "#{pane_id}" \
      "bash" "-lc" "$track_cmd")"
    tmux select-pane -t "$right_pane_id" -T "$name" 2>/dev/null || true
    tmux send-keys -t "$right_pane_id" "$TRACK_START_PROMPT"
    sleep "$TRACK_START_PAUSE"
    tmux send-keys -t "$right_pane_id" C-m

    dir="${TRACK_DIRS[1]}"
    name="${TRACK_NAMES[1]}"
    left_bottom_id="$(tmux split-window -v -t "$manager_pane_id" -c "$dir" -P -F "#{pane_id}" \
      "bash" "-lc" "$track_cmd")"
    tmux select-pane -t "$left_bottom_id" -T "$name" 2>/dev/null || true
    tmux send-keys -t "$left_bottom_id" "$TRACK_START_PROMPT"
    sleep "$TRACK_START_PAUSE"
    tmux send-keys -t "$left_bottom_id" C-m

    dir="${TRACK_DIRS[2]}"
    name="${TRACK_NAMES[2]}"
    left_third_id="$(tmux split-window -v -t "$left_bottom_id" -c "$dir" -P -F "#{pane_id}" \
      "bash" "-lc" "$track_cmd")"
    tmux select-pane -t "$left_third_id" -T "$name" 2>/dev/null || true
    tmux send-keys -t "$left_third_id" "$TRACK_START_PROMPT"
    sleep "$TRACK_START_PAUSE"
    tmux send-keys -t "$left_third_id" C-m

    dir="${TRACK_DIRS[3]}"
    name="${TRACK_NAMES[3]}"
    right_bottom_id="$(tmux split-window -v -t "$right_pane_id" -c "$dir" -P -F "#{pane_id}" \
      "bash" "-lc" "$track_cmd")"
    tmux select-pane -t "$right_bottom_id" -T "$name" 2>/dev/null || true
    tmux send-keys -t "$right_bottom_id" "$TRACK_START_PROMPT"
    sleep "$TRACK_START_PAUSE"
    tmux send-keys -t "$right_bottom_id" C-m

    dir="${TRACK_DIRS[4]}"
    name="${TRACK_NAMES[4]}"
    right_third_id="$(tmux split-window -v -t "$right_bottom_id" -c "$dir" -P -F "#{pane_id}" \
      "bash" "-lc" "$track_cmd")"
    tmux select-pane -t "$right_third_id" -T "$name" 2>/dev/null || true
    tmux send-keys -t "$right_third_id" "$TRACK_START_PROMPT"
    sleep "$TRACK_START_PAUSE"
    tmux send-keys -t "$right_third_id" C-m
  else
    for idx in "${!TRACK_DIRS[@]}"; do
      dir="${TRACK_DIRS[$idx]}"
      name="${TRACK_NAMES[$idx]}"
      target="$(largest_pane_id)"
      if [ -z "$target" ]; then
        echo "warning: no pane available to split for $name" >&2
        continue
      fi
      if ! pane_id="$(tmux split-window -t "$target" -c "$dir" -P -F "#{pane_id}" \
        "bash" "-lc" "$track_cmd")"; then
        echo "warning: tmux could not create pane for $name; increase terminal size or lower TRACK_LIMIT" >&2
        continue
      fi
      tmux select-pane -t "$pane_id" -T "$name" 2>/dev/null || true
      tmux send-keys -t "$pane_id" "$TRACK_START_PROMPT"
      sleep "$TRACK_START_PAUSE"
      tmux send-keys -t "$pane_id" C-m
    done
  fi
  top_left_id="$(tmux list-panes -t "$SESSION_MAIN:$WINDOW_MAIN" -F "#{pane_id} #{pane_top} #{pane_left}" \
    | sort -k2,2n -k3,3n | awk 'NR==1{print $1}')"
  if [ -n "$top_left_id" ] && [ "$top_left_id" != "$manager_pane_id" ]; then
    tmux swap-pane -s "$manager_pane_id" -t "$top_left_id" 2>/dev/null || true
  fi
  tmux select-pane -t "$manager_pane_id" 2>/dev/null || true
fi

escape_for_bash() {
  printf '%q' "$1"
}

MONITOR_CMD=$(cat <<'EOS'
MAIN_SESSION="zang-hub"
WINDOW="hub"
ROOT_DIR=__ROOT_DIR__
SPEC_DIR=__SPEC_DIR__
WORKTREE_BASE=__WORKTREE_BASE__
TRACK_LIMIT=__TRACK_LIMIT__
TRACK_CMD=__TRACK_CMD__
TRACK_START_PROMPT=__TRACK_START_PROMPT__
TRACK_START_PAUSE=__TRACK_START_PAUSE__
MANAGER_IDLE_SECONDS=__MANAGER_IDLE_SECONDS__
MANAGER_POKE=__MANAGER_POKE__
TRACK_IDLE_SECONDS=__TRACK_IDLE_SECONDS__
TRACK_POKE=__TRACK_POKE__

MANAGER_OPTION="@zang_manager_pane"

plan_is_complete() {
  local plan="$1"
  grep -Eiq '^[[:space:]]*(Status|Track Status|State)[[:space:]]*:[[:space:]]*(Complete|Completed|Done)[[:space:]]*$' "$plan"
}

plan_priority() {
  local plan="$1"
  local value
  value="$(awk -F: 'BEGIN{IGNORECASE=1} /^[[:space:]]*priority[[:space:]]*:/ {print $2; exit}' "$plan" | tr -cd '0-9-')"
  if [ -z "$value" ]; then
    echo 100
  else
    echo "$value"
  fi
}

is_worktree() {
  local path="$1"
  git -C "$ROOT_DIR" worktree list --porcelain | awk '/^worktree /{print $2}' | grep -Fx "$path" >/dev/null 2>&1
}

largest_pane_id() {
  tmux list-panes -t "${MAIN_SESSION}:${WINDOW}" -F "#{pane_id} #{pane_width} #{pane_height}" \
    | awk '{print $1, $2*$3}' | sort -k2,2nr | awk 'NR==1{print $1}'
}

declare -A last_hash
declare -A last_change
declare -A update_handled

while tmux has-session -t "$MAIN_SESSION" 2>/dev/null; do
  manager_pane="$(tmux show-option -gqv "$MANAGER_OPTION" 2>/dev/null || true)"
  if [ -z "$manager_pane" ]; then
    manager_pane="$(tmux list-panes -t "${MAIN_SESSION}:${WINDOW}" -F "#{pane_id} #{pane_top} #{pane_left}" \
      | sort -k2,2n -k3,3n | awk 'NR==1{print $1}')"
  fi
  if [ -z "$manager_pane" ]; then
    sleep 5
    continue
  fi

  tmux select-pane -t "$manager_pane" -T "manager" 2>/dev/null || true

  plan_meta=()
  while IFS= read -r plan; do
    [ -n "$plan" ] || continue
    if plan_is_complete "$plan"; then
      continue
    fi
    base="$(basename "$plan")"
    name="${base%_plan.md}"
    priority="$(plan_priority "$plan")"
    plan_meta+=("${priority}|${name}|${plan}")
  done < <(find "$SPEC_DIR" -maxdepth 1 -type f -name "*_plan.md" | sort)

  unset desired
  declare -A desired
  desired_list=()
  if [ ${#plan_meta[@]} -gt 0 ] && [ "$TRACK_LIMIT" -gt 0 ]; then
    IFS=$'\n' sorted=($(printf '%s\n' "${plan_meta[@]}" | sort -t '|' -k1,1n -k2,2))
    unset IFS
    for entry in "${sorted[@]}"; do
      rest="${entry#*|}"
      name="${rest%%|*}"
      desired["$name"]=1
      desired_list+=("$name")
      if [ ${#desired_list[@]} -ge "$TRACK_LIMIT" ]; then
        break
      fi
    done
  fi

  unset running
  declare -A running
  track_panes=0
  layout_changed=0

  while IFS=$'\t' read -r pane_id title path; do
    [ -n "$pane_id" ] || continue
    if [ "$pane_id" = "$manager_pane" ]; then
      continue
    fi
    if [ -z "$title" ]; then
      base="$(basename "$path")"
      if [[ "$base" == TypeScript-*-track ]]; then
        inferred="${base#TypeScript-}"
        inferred="${inferred%-track}"
        title="$inferred"
        tmux select-pane -t "$pane_id" -T "$title" 2>/dev/null || true
      fi
    fi
    if [ -n "$title" ]; then
      running["$title"]="$pane_id"
    fi
    track_panes=$((track_panes + 1))
  done < <(tmux list-panes -t "${MAIN_SESSION}:${WINDOW}" -F "#{pane_id}\t#{pane_title}\t#{pane_current_path}")

  for track in "${!running[@]}"; do
    if [ -z "${desired[$track]-}" ]; then
      tmux kill-pane -t "${running[$track]}" 2>/dev/null || true
      unset running["$track"]
      track_panes=$((track_panes - 1))
      layout_changed=1
    fi
  done

  for track in "${desired_list[@]}"; do
    if [ -n "${running[$track]-}" ]; then
      continue
    fi
    if [ "$track_panes" -ge "$TRACK_LIMIT" ]; then
      break
    fi
    dir="$WORKTREE_BASE/TypeScript-${track}-track"
    if [ -d "$dir" ]; then
      if ! is_worktree "$dir"; then
        echo "warning: $dir exists but is not a git worktree; skipping" >&2
        continue
      fi
    else
      git -C "$ROOT_DIR" worktree add --force "$dir" rust >/dev/null 2>&1 || continue
    fi
    target="$(largest_pane_id)"
    if [ -z "$target" ]; then
      continue
    fi
    if ! pane_id="$(tmux split-window -t "$target" -c "$dir" -P -F "#{pane_id}" \
      "bash" "-lc" "$TRACK_CMD")"; then
      continue
    fi
    tmux select-pane -t "$pane_id" -T "$track" 2>/dev/null || true
    tmux send-keys -t "$pane_id" "$TRACK_START_PROMPT"
    sleep "$TRACK_START_PAUSE"
    tmux send-keys -t "$pane_id" C-m
    running["$track"]="$pane_id"
    track_panes=$((track_panes + 1))
    layout_changed=1
  done

  if [ "$layout_changed" -eq 1 ]; then
    tmux select-layout -t "${MAIN_SESSION}:${WINDOW}" tiled
    pane_count="$(tmux list-panes -t "${MAIN_SESSION}:${WINDOW}" | wc -l | tr -d ' ')"
    if [ "$pane_count" = "6" ]; then
      tmux select-layout -t "${MAIN_SESSION}:${WINDOW}" tiled
    fi
    top_left="$(tmux list-panes -t "${MAIN_SESSION}:${WINDOW}" -F "#{pane_id} #{pane_top} #{pane_left}" \
      | sort -k2,2n -k3,3n | awk 'NR==1{print $1}')"
    if [ -n "$top_left" ] && [ "$top_left" != "$manager_pane" ]; then
      tmux swap-pane -s "$manager_pane" -t "$top_left" 2>/dev/null || true
    fi
  fi

  pane_list=()
  while IFS= read -r pane; do
    [ -n "$pane" ] && pane_list+=("$pane")
  done < <(tmux list-panes -t "${MAIN_SESSION}:${WINDOW}" -F "#{pane_id}")

  now=$(date +%s)
  for pane in "${pane_list[@]}"; do
    content=$(tmux capture-pane -p -t "$pane" -S -200)
    if [ -z "${update_handled[$pane]-}" ]; then
      if printf "%s" "$content" | grep -q "Update available"; then
        tmux send-keys -t "$pane" "1" C-m
        update_handled[$pane]=1
      fi
    fi
    hash=$(printf "%s" "$content" | cksum | awk '{print $1}')
    if [ "${last_hash[$pane]-}" != "$hash" ]; then
      last_hash[$pane]="$hash"
      last_change[$pane]="$now"
      continue
    fi
    last_seen="${last_change[$pane]-$now}"
    idle=$((now - last_seen))
    if [ "$pane" = "$manager_pane" ]; then
      if [ "$MANAGER_IDLE_SECONDS" -gt 0 ] && [ $idle -ge "$MANAGER_IDLE_SECONDS" ]; then
        tmux send-keys -t "$pane" "$MANAGER_POKE" C-m
        last_change[$pane]="$now"
      fi
    else
      if [ "$TRACK_IDLE_SECONDS" -gt 0 ] && [ $idle -ge "$TRACK_IDLE_SECONDS" ]; then
        tmux send-keys -t "$pane" "$TRACK_POKE" C-m
        last_change[$pane]="$now"
      fi
    fi
  done
  sleep 5
done
tmux set-option -g -u "@zang_auto_monitor" 2>/dev/null || true
EOS
)
MONITOR_CMD="${MONITOR_CMD/__ROOT_DIR__/$(escape_for_bash "$ROOT_DIR")}"
MONITOR_CMD="${MONITOR_CMD/__SPEC_DIR__/$(escape_for_bash "$SPEC_DIR")}"
MONITOR_CMD="${MONITOR_CMD/__WORKTREE_BASE__/$(escape_for_bash "$WORKTREE_BASE")}"
MONITOR_CMD="${MONITOR_CMD/__TRACK_LIMIT__/$TRACK_LIMIT}"
MONITOR_CMD="${MONITOR_CMD/__TRACK_CMD__/$(escape_for_bash "$track_cmd")}"
MONITOR_CMD="${MONITOR_CMD/__TRACK_START_PROMPT__/$(escape_for_bash "$TRACK_START_PROMPT")}"
MONITOR_CMD="${MONITOR_CMD/__TRACK_START_PAUSE__/$TRACK_START_PAUSE}"
MONITOR_CMD="${MONITOR_CMD/__MANAGER_IDLE_SECONDS__/$MANAGER_IDLE_SECONDS}"
MONITOR_CMD="${MONITOR_CMD/__MANAGER_POKE__/$(escape_for_bash "$MANAGER_POKE")}"
MONITOR_CMD="${MONITOR_CMD/__TRACK_IDLE_SECONDS__/$TRACK_IDLE_SECONDS}"
MONITOR_CMD="${MONITOR_CMD/__TRACK_POKE__/$(escape_for_bash "$TRACK_POKE")}"

MONITOR_OPTION="@zang_auto_monitor"
MONITOR_FLAG="$(tmux show-option -gqv "$MONITOR_OPTION" 2>/dev/null || true)"
if [ "$MONITOR_FLAG" != "1" ]; then
  tmux set-option -g "$MONITOR_OPTION" "1"
  MONITOR_PATH="$MANAGER_DIR/.zang_monitor.sh"
  printf "%s\n" "$MONITOR_CMD" > "$MONITOR_PATH"
  tmux run-shell -b "bash $(printf '%q' "$MONITOR_PATH") >/dev/null 2>&1"
fi

echo "Hub session (manager top-left): tmux attach -t $SESSION_MAIN"
if [ ${#TRACK_NAMES[@]} -eq 0 ]; then
  echo "Tracks: (none active)"
else
  echo "Tracks: ${TRACK_NAMES[*]}"
fi
echo "Track limit: $TRACK_LIMIT"
echo "Codex auto-update: $CODEX_AUTO_UPDATE"
echo "Worktrees base: $WORKTREE_BASE"

if [ "$AUTO_ATTACH" = "1" ] && [ -z "${TMUX:-}" ]; then
  tmux attach -t "$SESSION_MAIN"
fi
