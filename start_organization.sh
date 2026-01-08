#!/usr/bin/env bash
set -euo pipefail

# =============================================================================
# Zang Organization Orchestrator
# =============================================================================
# Creates a hierarchical AI agent organization:
#   Window 1 (director): Director agent
#   Window 2 (solver):   EM-Solver + 3 Workers
#   Window 3 (tools):    EM-Tools + 3 Workers
# =============================================================================

SESSION="zang-org"
CODEX_CMD="${CODEX_CMD:-codex}"
CODEX_ARGS="${CODEX_ARGS:---dangerously-bypass-approvals-and-sandbox}"
AUTO_FETCH="${AUTO_FETCH:-1}"
AUTO_RESTART_CODEX="${AUTO_RESTART_CODEX:-1}"
CODEX_RESTART_DELAY="${CODEX_RESTART_DELAY:-2}"
CODEX_AUTO_UPDATE="${CODEX_AUTO_UPDATE:-1}"
CODEX_UPDATE_CMD="${CODEX_UPDATE_CMD:-npm install -g @openai/codex}"

# Idle monitoring
DIRECTOR_IDLE_SECONDS="${DIRECTOR_IDLE_SECONDS:-120}"
DIRECTOR_POKE="${DIRECTOR_POKE:-continue}"
EM_IDLE_SECONDS="${EM_IDLE_SECONDS:-90}"
EM_POKE="${EM_POKE:-continue}"
WORKER_IDLE_SECONDS="${WORKER_IDLE_SECONDS:-180}"
WORKER_POKE="${WORKER_POKE:-continue with your plan.}"

# Startup
WORKER_START_PROMPT="${WORKER_START_PROMPT:-continue with your plan.}"
EM_START_PROMPT="${EM_START_PROMPT:-continue}"
DIRECTOR_START_PROMPT="${DIRECTOR_START_PROMPT:-continue}"
START_PAUSE="${START_PAUSE:-30}"
SEND_ENTER_PAUSE="${SEND_ENTER_PAUSE:-2}"

AUTO_MONITOR="${AUTO_MONITOR:-0}"
AUTO_ATTACH="${AUTO_ATTACH:-1}"

# =============================================================================
# Kill mode
# =============================================================================
if [ "${1:-}" = "--kill" ]; then
  if tmux has-session -t "$SESSION" 2>/dev/null; then
    tmux kill-session -t "$SESSION"
    echo "Killed tmux session: $SESSION"
  else
    echo "No tmux session found: $SESSION"
  fi
  exit 0
fi

# =============================================================================
# Path setup
# =============================================================================
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

# =============================================================================
# Validation
# =============================================================================
if [ ! -d "$ROOT_DIR" ]; then
  echo "error: TypeScript root not found" >&2
  exit 1
fi

if [ ! -d "$WORKTREE_BASE" ]; then
  echo "error: worktree base not found: $WORKTREE_BASE" >&2
  exit 1
fi

for cmd in tmux git "$CODEX_CMD"; do
  if ! command -v "$cmd" >/dev/null 2>&1; then
    echo "error: $cmd not found in PATH" >&2
    exit 1
  fi
done

if ! git -C "$ROOT_DIR" rev-parse --is-inside-work-tree >/dev/null 2>&1; then
  echo "error: $ROOT_DIR is not a git repo" >&2
  exit 1
fi

# =============================================================================
# Auto-update Codex
# =============================================================================
if [ "$CODEX_AUTO_UPDATE" = "1" ]; then
  if command -v npm >/dev/null 2>&1; then
    echo "Updating Codex..."
    if ! bash -lc "$CODEX_UPDATE_CMD" >/dev/null 2>&1; then
      echo "warning: Codex update failed; continuing" >&2
    fi
  fi
fi

# =============================================================================
# Ensure rust branch exists
# =============================================================================
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

# =============================================================================
# Create squad spec directories if needed
# =============================================================================
SPEC_DIR="$ROOT_DIR/wasm/specs"
SQUAD_DIR="$SPEC_DIR/squads"
mkdir -p "$SQUAD_DIR/solver" "$SQUAD_DIR/tools"

# =============================================================================
# Create initial GOALS.md files if they don't exist
# =============================================================================
create_goals_if_missing() {
  local squad="$1"
  local goals_file="$SQUAD_DIR/$squad/GOALS.md"
  local squad_cap
  squad_cap="$(echo "$squad" | sed 's/./\U&/')"

  if [ ! -f "$goals_file" ]; then
    cat > "$goals_file" << EOF
# Squad $squad_cap Goals

Updated: $(date +%Y-%m-%d)

## Current Milestone
[Director: Set the current milestone based on Project Direction]

## Objectives (Ranked)
1. **[Objective 1]**
   - Context: [Why this matters]
   - Success Criteria: [Measurable outcome]
   - Key Files: [Paths]
   - Estimated Complexity: [Low/Medium/High]

## Anti-Priorities
- [Director: List things NOT to work on]

## Cross-Squad Dependencies
- None currently

## Notes to EM
- Read wasm/specs/WASM_ARCHITECTURE.md
- Use Docker for tests: ./wasm/test.sh

## Squad Status
- Last EM Report: Not yet started
- Workers Active: 0/3
- Branches Pending Merge: None
- Current Focus: Awaiting Director assignment
- Blockers: None
EOF
    echo "Created $goals_file"
  fi
}

create_goals_if_missing "solver"
create_goals_if_missing "tools"

# =============================================================================
# Create initial worker plan files if they don't exist
# =============================================================================
create_worker_plan_if_missing() {
  local squad="$1"
  local worker_num="$2"
  local plan_file="$SQUAD_DIR/$squad/worker-${worker_num}_plan.md"
  local squad_cap
  squad_cap="$(echo "$squad" | sed 's/./\U&/')"

  if [ ! -f "$plan_file" ]; then
    cat > "$plan_file" << EOF
# Worker ${worker_num} Plan

## Mission
Execute tasks assigned by EM-$squad_cap for the $squad_cap squad.

Status: Active
Priority: ${worker_num}

## Current Assignment
- [EM: Assign initial task]

## Task Queue
- [ ] [EM: Add queued tasks]

## Completed
- [x] (Move finished items here with brief notes and tests run)

## Ready for Merge
No

## Notes
- Follow \`wasm/specs/WASM_ARCHITECTURE.md\`
- Use Docker for Rust tests: \`./wasm/test.sh\`
- Commit format: \`[wasm] <component>: <description>\`
- Sync before each task: \`git fetch origin && git merge origin/rust --no-edit\`
- Push to: \`origin/worker/${squad}-${worker_num}\`
EOF
    echo "Created $plan_file"
  fi
}

for squad in solver tools; do
  for n in 1 2 3 4; do
    create_worker_plan_if_missing "$squad" "$n"
  done
done

# =============================================================================
# Worktree helpers
# =============================================================================
is_worktree() {
  local path="$1"
  git -C "$ROOT_DIR" worktree list --porcelain | awk '/^worktree /{print $2}' | grep -Fx "$path" >/dev/null 2>&1
}

ensure_worktree() {
  local name="$1"
  local dir="$WORKTREE_BASE/TypeScript-${name}-track"

  if [ -d "$dir" ]; then
    if ! is_worktree "$dir"; then
      echo "warning: $dir exists but is not a git worktree; skipping" >&2
      return 1
    fi
  else
    git -C "$ROOT_DIR" worktree add --force "$dir" rust >/dev/null 2>&1 || {
      echo "warning: could not create worktree for $name" >&2
      return 1
    }
  fi
  echo "$dir"
}

# =============================================================================
# Ensure worktrees for all workers (using simple variables instead of assoc array)
# =============================================================================
WORKTREE_solver_1=""
WORKTREE_solver_2=""
WORKTREE_solver_3=""
WORKTREE_solver_4=""
WORKTREE_tools_1=""
WORKTREE_tools_2=""
WORKTREE_tools_3=""
WORKTREE_tools_4=""

for squad in solver tools; do
  for n in 1 2 3 4; do
    name="${squad}-${n}"
    if dir="$(ensure_worktree "$name")"; then
      eval "WORKTREE_${squad}_${n}=\"$dir\""
    fi
  done
done

# Helper to get worktree dir
get_worktree_dir() {
  local squad="$1"
  local n="$2"
  eval "echo \"\$WORKTREE_${squad}_${n}\""
}

# =============================================================================
# Build commands
# =============================================================================
build_cmd() {
  local base_cmd="$1"
  if [ "$AUTO_RESTART_CODEX" = "1" ]; then
    echo "while true; do ${base_cmd}; sleep ${CODEX_RESTART_DELAY}; done"
  else
    echo "$base_cmd"
  fi
}

director_cmd="$(build_cmd "$CODEX_CMD $CODEX_ARGS")"
em_cmd="$(build_cmd "$CODEX_CMD $CODEX_ARGS")"
worker_cmd="$(build_cmd "$CODEX_CMD $CODEX_ARGS")"

# =============================================================================
# Create tmux session if not exists
# =============================================================================
if tmux has-session -t "$SESSION" 2>/dev/null; then
  echo "Session $SESSION already exists. Use --kill to restart."
  exit 0
fi

# Create session with director window
tmux new-session -d -s "$SESSION" -n "director" -c "$ROOT_DIR"

# =============================================================================
# Window 1: Director
# =============================================================================
echo "Setting up Director window..."
tmux send-keys -t "$SESSION:director" "bash -lc '$director_cmd'" C-m
tmux select-pane -t "$SESSION:director.0" -T "director" 2>/dev/null || true
sleep "$START_PAUSE"
tmux send-keys -t "$SESSION:director.0" "$DIRECTOR_START_PROMPT"
sleep "$SEND_ENTER_PAUSE"
tmux send-keys -t "$SESSION:director.0" C-m

# =============================================================================
# Helper: Setup squad window (EM + 4 workers)
# =============================================================================
setup_squad_window() {
  local squad="$1"
  local window="$squad"

  echo "Setting up $squad squad window..."

  # Create window for squad
  tmux new-window -t "$SESSION" -n "$window" -c "$ROOT_DIR"

  # Pane 0: EM (top-left)
  # Export SQUAD_NAME so EM knows which squad it manages
  tmux send-keys -t "$SESSION:$window.0" "export SQUAD_NAME=$squad && bash -lc '$em_cmd'" C-m
  tmux select-pane -t "$SESSION:$window.0" -T "em-$squad" 2>/dev/null || true

  # Create layout: EM + 4 workers
  # Split horizontally: creates pane 1 (right)
  local worker1_dir
  worker1_dir="$(get_worktree_dir "$squad" 1)"
  [ -z "$worker1_dir" ] && worker1_dir="$ROOT_DIR"
  tmux split-window -h -t "$SESSION:$window.0" -c "$worker1_dir"
  tmux send-keys -t "$SESSION:$window.1" "bash -lc '$worker_cmd'" C-m
  tmux select-pane -t "$SESSION:$window.1" -T "${squad}-1" 2>/dev/null || true

  # Split pane 0 vertically: creates pane 2 (bottom-left)
  local worker2_dir
  worker2_dir="$(get_worktree_dir "$squad" 2)"
  [ -z "$worker2_dir" ] && worker2_dir="$ROOT_DIR"
  tmux split-window -v -t "$SESSION:$window.0" -c "$worker2_dir"
  tmux send-keys -t "$SESSION:$window.2" "bash -lc '$worker_cmd'" C-m
  tmux select-pane -t "$SESSION:$window.2" -T "${squad}-2" 2>/dev/null || true

  # Split pane 1 vertically: creates pane 3 (bottom-right top)
  local worker3_dir
  worker3_dir="$(get_worktree_dir "$squad" 3)"
  [ -z "$worker3_dir" ] && worker3_dir="$ROOT_DIR"
  tmux split-window -v -t "$SESSION:$window.1" -c "$worker3_dir"
  tmux send-keys -t "$SESSION:$window.3" "bash -lc '$worker_cmd'" C-m
  tmux select-pane -t "$SESSION:$window.3" -T "${squad}-3" 2>/dev/null || true

  # Split pane 3 vertically: creates pane 4 (bottom-right bottom)
  local worker4_dir
  worker4_dir="$(get_worktree_dir "$squad" 4)"
  [ -z "$worker4_dir" ] && worker4_dir="$ROOT_DIR"
  tmux split-window -v -t "$SESSION:$window.3" -c "$worker4_dir"
  tmux send-keys -t "$SESSION:$window.4" "bash -lc '$worker_cmd'" C-m
  tmux select-pane -t "$SESSION:$window.4" -T "${squad}-4" 2>/dev/null || true

  # Balance the layout
  tmux select-layout -t "$SESSION:$window" tiled

  # Send start prompts to EM and workers
  sleep "$START_PAUSE"

  # EM start
  tmux send-keys -t "$SESSION:$window.0" "$EM_START_PROMPT"
  sleep "$SEND_ENTER_PAUSE"
  tmux send-keys -t "$SESSION:$window.0" C-m

  # Worker starts (staggered slightly)
  for pane in 1 2 3 4; do
    sleep 3
    tmux send-keys -t "$SESSION:$window.$pane" "$WORKER_START_PROMPT"
    sleep "$SEND_ENTER_PAUSE"
    tmux send-keys -t "$SESSION:$window.$pane" C-m
  done
}

# =============================================================================
# Window 2: Solver Squad
# =============================================================================
setup_squad_window "solver"

# =============================================================================
# Window 3: Tools Squad
# =============================================================================
setup_squad_window "tools"

# =============================================================================
# Select director window
# =============================================================================
tmux select-window -t "$SESSION:director"

# =============================================================================
# Monitor script (optional) - uses file-based state instead of assoc arrays
# =============================================================================
escape_for_bash() {
  printf '%q' "$1"
}

MONITOR_CMD=$(cat <<'EOS'
SESSION="zang-org"
ROOT_DIR=__ROOT_DIR__
SQUAD_DIR=__SQUAD_DIR__

DIRECTOR_IDLE_SECONDS=__DIRECTOR_IDLE_SECONDS__
DIRECTOR_POKE=__DIRECTOR_POKE__
EM_IDLE_SECONDS=__EM_IDLE_SECONDS__
EM_POKE=__EM_POKE__
WORKER_IDLE_SECONDS=__WORKER_IDLE_SECONDS__
WORKER_POKE=__WORKER_POKE__
SEND_ENTER_PAUSE=__SEND_ENTER_PAUSE__

STATE_DIR="/tmp/zang-org-monitor-$$"
mkdir -p "$STATE_DIR"
trap "rm -rf '$STATE_DIR'" EXIT

get_idle_threshold() {
  local window="$1"
  local pane="$2"

  if [ "$window" = "director" ]; then
    echo "$DIRECTOR_IDLE_SECONDS"
  elif [ "$pane" = "0" ]; then
    echo "$EM_IDLE_SECONDS"
  else
    echo "$WORKER_IDLE_SECONDS"
  fi
}

get_poke_message() {
  local window="$1"
  local pane="$2"

  if [ "$window" = "director" ]; then
    echo "$DIRECTOR_POKE"
  elif [ "$pane" = "0" ]; then
    echo "$EM_POKE"
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

  for window in director solver tools; do
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
EOS
)

MONITOR_CMD="${MONITOR_CMD//__ROOT_DIR__/$(escape_for_bash "$ROOT_DIR")}"
MONITOR_CMD="${MONITOR_CMD//__SQUAD_DIR__/$(escape_for_bash "$SQUAD_DIR")}"
MONITOR_CMD="${MONITOR_CMD//__DIRECTOR_IDLE_SECONDS__/$DIRECTOR_IDLE_SECONDS}"
MONITOR_CMD="${MONITOR_CMD//__DIRECTOR_POKE__/$(escape_for_bash "$DIRECTOR_POKE")}"
MONITOR_CMD="${MONITOR_CMD//__EM_IDLE_SECONDS__/$EM_IDLE_SECONDS}"
MONITOR_CMD="${MONITOR_CMD//__EM_POKE__/$(escape_for_bash "$EM_POKE")}"
MONITOR_CMD="${MONITOR_CMD//__WORKER_IDLE_SECONDS__/$WORKER_IDLE_SECONDS}"
MONITOR_CMD="${MONITOR_CMD//__WORKER_POKE__/$(escape_for_bash "$WORKER_POKE")}"
MONITOR_CMD="${MONITOR_CMD//__SEND_ENTER_PAUSE__/$SEND_ENTER_PAUSE}"

if [ "$AUTO_MONITOR" = "1" ]; then
  MONITOR_PATH="$ROOT_DIR/.zang_org_monitor.sh"
  printf "%s\n" "$MONITOR_CMD" > "$MONITOR_PATH"
  tmux run-shell -b "bash $(printf '%q' "$MONITOR_PATH") >/dev/null 2>&1"
  echo "Monitor started"
fi

# =============================================================================
# Summary
# =============================================================================
echo ""
echo "=============================================="
echo "Zang Organization Started"
echo "=============================================="
echo "Session: $SESSION"
echo ""
echo "Windows:"
echo "  1. director  - Director agent"
echo "  2. solver    - EM-Solver + 4 Workers"
echo "  3. tools     - EM-Tools + 4 Workers"
echo ""
echo "Worktrees:"
for squad in solver tools; do
  for n in 1 2 3 4; do
    dir="$(get_worktree_dir "$squad" "$n")"
    [ -n "$dir" ] && echo "  ${squad}-${n}: $dir"
  done
done
echo ""
echo "Quick navigation:"
echo "  tmux select-window -t $SESSION:director"
echo "  tmux select-window -t $SESSION:solver"
echo "  tmux select-window -t $SESSION:tools"
echo ""
echo "Attach: tmux attach -t $SESSION"
echo "Kill:   $0 --kill"
echo "=============================================="

# =============================================================================
# Auto-attach
# =============================================================================
if [ "$AUTO_ATTACH" = "1" ] && [ -z "${TMUX:-}" ]; then
  tmux attach -t "$SESSION"
fi
