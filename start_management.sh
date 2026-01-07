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
MANAGER_IDLE_SECONDS="${MANAGER_IDLE_SECONDS:-60}"
MANAGER_POKE="${MANAGER_POKE:-Continue managing}"
TRACK_IDLE_SECONDS="${TRACK_IDLE_SECONDS:-60}"
TRACK_POKE="${TRACK_POKE:-continue with your plan.}"
TRACK_START_PROMPT="${TRACK_START_PROMPT:-$TRACK_POKE}"
TRACK_START_PAUSE="${TRACK_START_PAUSE:-0.5}"

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

TRACK_NAMES=()
for plan in "${PLAN_FILES[@]}"; do
  base="$(basename "$plan")"
  TRACK_NAMES+=("${base%_plan.md}")
done

TRACK_DIRS=()
for name in "${TRACK_NAMES[@]}"; do
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
if [ ! -f "$MANAGER_DIR/MANAGER_AGENT.md" ] && [ -f "$ROOT_DIR/MANAGER_AGENT.md" ]; then
  ln -s "$ROOT_DIR/MANAGER_AGENT.md" "$MANAGER_DIR/MANAGER_AGENT.md"
fi
if [ -f "$MANAGER_DIR/MANAGER_AGENT.md" ] && [ ! -e "$MANAGER_DIR/AGENT.md" ]; then
  ln -s "MANAGER_AGENT.md" "$MANAGER_DIR/AGENT.md"
elif [ ! -f "$MANAGER_DIR/MANAGER_AGENT.md" ]; then
  echo "warning: MANAGER_AGENT.md not found in $MANAGER_DIR" >&2
fi

manager_cmd="${CODEX_CMD} ${CODEX_MANAGER_ARGS}"
track_cmd="${CODEX_CMD} ${CODEX_TRACK_ARGS}"
if [ "$AUTO_RESTART_CODEX" = "1" ]; then
  manager_cmd="while true; do ${manager_cmd}; sleep ${CODEX_RESTART_DELAY}; done"
  track_cmd="while true; do ${track_cmd}; sleep ${CODEX_RESTART_DELAY}; done"
fi

if ! tmux has-session -t "$SESSION_MAIN" 2>/dev/null; then
  tmux new-session -d -s "$SESSION_MAIN" -n "$WINDOW_MAIN" -c "$MANAGER_DIR" \
    "bash" "-lc" "$manager_cmd"
  manager_pane_id="$(tmux display-message -p -t "$SESSION_MAIN:$WINDOW_MAIN.0" "#{pane_id}")"
  for dir in "${TRACK_DIRS[@]}"; do
    tmux split-window -t "$SESSION_MAIN:$WINDOW_MAIN" -c "$dir" \
      "bash" "-lc" "$track_cmd"
  done
  tmux select-layout -t "$SESSION_MAIN:$WINDOW_MAIN" tiled
  top_left=$(tmux list-panes -t "$SESSION_MAIN:$WINDOW_MAIN" -F "#{pane_index} #{pane_top} #{pane_left}" \
    | sort -k2,2n -k3,3n | awk 'NR==1{print $1}')
  if [ -n "$top_left" ] && [ "$top_left" != "0" ]; then
    tmux swap-pane -s "$SESSION_MAIN:$WINDOW_MAIN.0" -t "$SESSION_MAIN:$WINDOW_MAIN.$top_left"
  fi
  tmux set-option -g "@zang_manager_pane" "$manager_pane_id"
  sleep 1
  tmux list-panes -t "$SESSION_MAIN:$WINDOW_MAIN" -F "#{pane_id} #{pane_index}" \
    | awk '$2 != 0 {print $1}' \
    | while read -r pane_id; do
        tmux send-keys -t "$pane_id" "$TRACK_START_PROMPT"
        sleep "$TRACK_START_PAUSE"
        tmux send-keys -t "$pane_id" C-m
      done
fi

MONITOR_CMD=$(cat <<'EOS'
MAIN_SESSION="zang-hub"
WINDOW="hub"
MANAGER_OPTION="@zang_manager_pane"
manager_pane="$(tmux show-option -gqv "$MANAGER_OPTION" 2>/dev/null || true)"
if [ -z "$manager_pane" ]; then
  manager_pane="$(tmux list-panes -t "${MAIN_SESSION}:${WINDOW}" -F "#{pane_id} #{pane_top} #{pane_left}" \
    | sort -k2,2n -k3,3n | awk 'NR==1{print $1}')"
fi
if [ -z "$manager_pane" ]; then
  echo "manager pane not found"
  exit 0
fi
declare -A last_hash
declare -A last_change
while tmux has-session -t "$MAIN_SESSION" 2>/dev/null; do
  now=$(date +%s)
  pane_list=()
  while IFS= read -r pane_id; do
    [ -n "$pane_id" ] && pane_list+=("$pane_id")
  done < <(tmux list-panes -t "${MAIN_SESSION}:${WINDOW}" -F "#{pane_id}")

  if [ ${#pane_list[@]} -eq 0 ]; then
    sleep 5
    continue
  fi

  manager_pane="$(tmux show-option -gqv "$MANAGER_OPTION" 2>/dev/null || true)"
  if [ -z "$manager_pane" ]; then
    manager_pane="$(tmux list-panes -t "${MAIN_SESSION}:${WINDOW}" -F "#{pane_id} #{pane_top} #{pane_left}" \
      | sort -k2,2n -k3,3n | awk 'NR==1{print $1}')"
  fi

  for pane in "${pane_list[@]}"; do
    content=$(tmux capture-pane -p -t "$pane" -S -200)
    hash=$(printf "%s" "$content" | cksum | awk '{print $1}')
    if [ "${last_hash[$pane]-}" != "$hash" ]; then
      last_hash[$pane]="$hash"
      last_change[$pane]="$now"
      continue
    fi
    last_seen="${last_change[$pane]-$now}"
    idle=$((now - last_seen))
    if [ "$pane" = "$manager_pane" ]; then
      if [ $idle -ge __MANAGER_IDLE_SECONDS__ ]; then
        tmux send-keys -t "$pane" "__MANAGER_POKE__" C-m
        last_change[$pane]="$now"
      fi
    else
      if [ $idle -ge __TRACK_IDLE_SECONDS__ ]; then
        tmux send-keys -t "$pane" "__TRACK_POKE__" C-m
        last_change[$pane]="$now"
      fi
    fi
  done
  sleep 5
done
tmux set-option -g -u "@zang_auto_monitor" 2>/dev/null || true
EOS
)
MONITOR_CMD="${MONITOR_CMD/__MANAGER_IDLE_SECONDS__/$MANAGER_IDLE_SECONDS}"
MONITOR_CMD="${MONITOR_CMD/__MANAGER_POKE__/$MANAGER_POKE}"
MONITOR_CMD="${MONITOR_CMD/__TRACK_IDLE_SECONDS__/$TRACK_IDLE_SECONDS}"
MONITOR_CMD="${MONITOR_CMD/__TRACK_POKE__/$TRACK_POKE}"

MONITOR_OPTION="@zang_auto_monitor"
MONITOR_FLAG="$(tmux show-option -gqv "$MONITOR_OPTION" 2>/dev/null || true)"
if [ "$MONITOR_FLAG" != "1" ]; then
  tmux set-option -g "$MONITOR_OPTION" "1"
  tmux run-shell -b "bash -lc $(printf '%q' "$MONITOR_CMD")"
fi

echo "Hub session (manager top-left): tmux attach -t $SESSION_MAIN"
echo "Tracks: ${TRACK_NAMES[*]}"
echo "Worktrees base: $WORKTREE_BASE"
