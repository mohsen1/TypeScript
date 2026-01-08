#!/usr/bin/env bash
set -euo pipefail

# =============================================================================
# Zang Organization Orchestrator
# =============================================================================
# Creates a hierarchical AI agent organization:
#   Window 1 (director): Director agent
#   Window 2 (forge):    EM-Forge + 5 Workers (type system)
#   Window 3 (anvil):    EM-Anvil + 5 Workers (output)
# =============================================================================

SESSION="zang-org"

# Agent CLI selection (claude is default, use --codex for OpenAI Codex)
USE_CODEX=0
for arg in "$@"; do
  if [ "$arg" = "--codex" ]; then
    USE_CODEX=1
  fi
done

if [ "$USE_CODEX" = "1" ]; then
  AGENT_CMD="${AGENT_CMD:-codex}"
  AGENT_ARGS="${AGENT_ARGS:---dangerously-bypass-approvals-and-sandbox}"
  AGENT_UPDATE_CMD="${AGENT_UPDATE_CMD:-npm install -g @openai/codex}"
  echo "Using OpenAI Codex CLI"
else
  AGENT_CMD="${AGENT_CMD:-claude}"
  AGENT_ARGS="${AGENT_ARGS:---dangerously-skip-permissions}"
  AGENT_UPDATE_CMD="${AGENT_UPDATE_CMD:-npm install -g @anthropic-ai/claude-code}"
  echo "Using Claude Code CLI (default)"
fi

AUTO_FETCH="${AUTO_FETCH:-1}"
AUTO_RESTART_AGENT="${AUTO_RESTART_AGENT:-1}"
AGENT_RESTART_DELAY="${AGENT_RESTART_DELAY:-2}"
AGENT_AUTO_UPDATE="${AGENT_AUTO_UPDATE:-1}"

# Idle monitoring (Director should be mostly idle - hands-off management)
DIRECTOR_IDLE_SECONDS="${DIRECTOR_IDLE_SECONDS:-600}"
DIRECTOR_POKE="${DIRECTOR_POKE:-MERGE TIME! Tell both EMs to pause work and push branches. Wait 4 minutes for them to finish, then merge squad/forge and squad/anvil into rust. After merging, tell EMs to sync workers from origin/rust.}"
EM_IDLE_SECONDS="${EM_IDLE_SECONDS:-60}"
EM_POKE="${EM_POKE:-Check worker panes for stuck workers. If all workers are busy, check for Ready for Merge branches and merge them into squad branch.}"
WORKER_IDLE_SECONDS="${WORKER_IDLE_SECONDS:-300}"
WORKER_POKE="${WORKER_POKE:-How is your task going? If you need help, describe what you are stuck on.}"

# Merge coordination - Director signals EMs to pause for coordinated merge
MERGE_PAUSE_SECONDS="${MERGE_PAUSE_SECONDS:-240}"  # 4 minutes for EMs to finish and push

# Startup timing (agent boots in ~5s)
WORKER_START_PROMPT="${WORKER_START_PROMPT:-You are worker \$WORKER_NUM in squad \$SQUAD_NAME. Read .role/AGENTS.md then your plan at wasm/specs/squads/\$SQUAD_NAME/worker-\${WORKER_NUM}_plan.md. Switch to branch worker/\$SQUAD_NAME-\$WORKER_NUM and work on your current assignment.}"
EM_START_PROMPT="${EM_START_PROMPT:-You are EM for squad \$SQUAD_NAME. Read .role/AGENTS.md for your instructions. You have your own worktree on branch em/\$SQUAD_NAME. FIRST: run ./wasm/test.sh 2>&1 | head -50 to check build. If build fails, FIX IT YOURSELF before assigning worker tasks.}"
DIRECTOR_START_PROMPT="${DIRECTOR_START_PROMPT:-Read .role/AGENTS.md for your instructions. Merge EM branches (em/forge, em/anvil) into rust when they have blocker fixes. Be hands-off otherwise.}"
START_PAUSE="${START_PAUSE:-10}"
SEND_ENTER_PAUSE="${SEND_ENTER_PAUSE:-1}"
STAGGER_PAUSE="${STAGGER_PAUSE:-2}"

AUTO_MONITOR="${AUTO_MONITOR:-1}"
AUTO_ATTACH="${AUTO_ATTACH:-1}"

# =============================================================================
# Session state file for resume
# =============================================================================
STATE_FILE="${STATE_FILE:-$HOME/.zang-org-state.json}"

save_session_state() {
  echo "Saving session state to $STATE_FILE..."
  local state="{\"timestamp\": \"$(date -u +%Y-%m-%dT%H:%M:%SZ)\", \"panes\": {"
  local first=1

  for window in director forge anvil; do
    if ! tmux list-windows -t "$SESSION" -F "#{window_name}" 2>/dev/null | grep -qx "$window"; then
      continue
    fi

    pane_count=$(tmux list-panes -t "$SESSION:$window" 2>/dev/null | wc -l | tr -d ' ')
    pane=0
    while [ "$pane" -lt "$pane_count" ]; do
      # Capture last 500 lines of each pane for context
      content=$(tmux capture-pane -p -t "$SESSION:$window.$pane" -S -500 2>/dev/null | tail -200 || true)
      # Escape for JSON
      content_escaped=$(printf '%s' "$content" | python3 -c 'import json,sys; print(json.dumps(sys.stdin.read()))' 2>/dev/null || echo '""')

      # Get current directory
      pane_path=$(tmux display-message -p -t "$SESSION:$window.$pane" '#{pane_current_path}' 2>/dev/null || echo "")

      if [ "$first" = "1" ]; then
        first=0
      else
        state="$state,"
      fi
      state="$state \"${window}_${pane}\": {\"path\": \"$pane_path\", \"context\": $content_escaped}"

      pane=$((pane + 1))
    done
  done

  state="$state }}"
  echo "$state" > "$STATE_FILE"
  echo "Session state saved."
}

# =============================================================================
# Kill mode (with state save)
# =============================================================================
if [ "${1:-}" = "--kill" ]; then
  if tmux has-session -t "$SESSION" 2>/dev/null; then
    save_session_state
    tmux kill-session -t "$SESSION"
    echo "Killed tmux session: $SESSION"
    echo "State saved to: $STATE_FILE"
    echo "Use '$0 --resume' to continue where you left off"
  else
    echo "No tmux session found: $SESSION"
  fi
  exit 0
fi

# =============================================================================
# Resume mode - restore from saved state
# =============================================================================
RESUME_MODE=0
if [ "${1:-}" = "--resume" ]; then
  if [ -f "$STATE_FILE" ]; then
    RESUME_MODE=1
    echo "Resume mode: will restore from $STATE_FILE"
  else
    echo "No state file found at $STATE_FILE"
    echo "Starting fresh session instead..."
  fi
fi

# =============================================================================
# Fresh mode - reset all branches to origin/rust
# =============================================================================
FRESH_MODE=0
if [ "${1:-}" = "--fresh" ]; then
  FRESH_MODE=1
  echo "Fresh mode: will reset all branches to origin/rust"
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

for cmd in tmux git "$AGENT_CMD"; do
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
# Auto-update Agent CLI
# =============================================================================
if [ "$AGENT_AUTO_UPDATE" = "1" ]; then
  if command -v npm >/dev/null 2>&1; then
    echo "Updating $AGENT_CMD..."
    if ! bash -lc "$AGENT_UPDATE_CMD" >/dev/null 2>&1; then
      echo "warning: $AGENT_CMD update failed; continuing" >&2
    fi
  fi
fi

# =============================================================================
# Ensure rust branch exists
# =============================================================================
if [ "$AUTO_FETCH" = "1" ]; then
  git -C "$ROOT_DIR" fetch --prune origin >/dev/null 2>&1 || true
fi

if ! git -C "$ROOT_DIR" show-ref --verify --quiet refs/heads/rust; then
  if git -C "$ROOT_DIR" show-ref --verify --quiet refs/remotes/origin/rust; then
    git -C "$ROOT_DIR" branch --track rust origin/rust >/dev/null 2>&1 || true
  else
    git -C "$ROOT_DIR" branch rust >/dev/null 2>&1 || true
  fi
fi

# =============================================================================
# Fresh mode: Reset all branches to origin/rust
# =============================================================================
if [ "$FRESH_MODE" = "1" ]; then
  echo "Resetting all branches to origin/rust..."

  # Reset squad branches
  for squad in forge anvil; do
    branch="squad/$squad"
    if git -C "$ROOT_DIR" show-ref --verify --quiet "refs/heads/$branch"; then
      git -C "$ROOT_DIR" branch -D "$branch" >/dev/null 2>&1 || true
    fi
    git -C "$ROOT_DIR" branch "$branch" origin/rust >/dev/null 2>&1 || true
    echo "  Reset $branch -> origin/rust"
  done

  # Reset worker branches
  for squad in forge anvil; do
    for n in 1 2 3 4 5; do
      branch="worker/${squad}-${n}"
      if git -C "$ROOT_DIR" show-ref --verify --quiet "refs/heads/$branch"; then
        git -C "$ROOT_DIR" branch -D "$branch" >/dev/null 2>&1 || true
      fi
      git -C "$ROOT_DIR" branch "$branch" origin/rust >/dev/null 2>&1 || true
      echo "  Reset $branch -> origin/rust"
    done
  done

  echo "All branches reset to origin/rust"
fi

# =============================================================================
# Create squad spec directories if needed
# =============================================================================
SPEC_DIR="$ROOT_DIR/wasm/specs"
SQUAD_DIR="$SPEC_DIR/squads"
mkdir -p "$SQUAD_DIR/forge" "$SQUAD_DIR/anvil"

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

create_goals_if_missing "forge"
create_goals_if_missing "anvil"

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

for squad in forge anvil; do
  for n in 1 2 3 4 5; do
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
  local branch="worker/$name"

  if [ -d "$dir" ]; then
    if ! is_worktree "$dir"; then
      echo "warning: $dir exists but is not a git worktree; skipping" >&2
      return 1
    fi
    # Fresh mode: reset worktree to origin/rust
    if [ "$FRESH_MODE" = "1" ]; then
      git -C "$dir" fetch origin >/dev/null 2>&1 || true
      git -C "$dir" reset --hard origin/rust >/dev/null 2>&1 || true
      git -C "$dir" clean -fd >/dev/null 2>&1 || true
      git -C "$dir" checkout -B "$branch" origin/rust >/dev/null 2>&1 || true
      echo "  Reset worktree $name -> origin/rust" >&2
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
# Ensure EM worktrees (EMs get their own worktrees for blocker fixes)
# =============================================================================
ensure_em_worktree() {
  local squad="$1"
  local dir="$WORKTREE_BASE/TypeScript-em-${squad}"
  local branch="em/$squad"

  if [ -d "$dir" ]; then
    if ! is_worktree "$dir"; then
      echo "warning: $dir exists but is not a git worktree; skipping" >&2
      return 1
    fi
    # Fresh mode: reset worktree to origin/rust
    if [ "$FRESH_MODE" = "1" ]; then
      git -C "$dir" fetch origin >/dev/null 2>&1 || true
      git -C "$dir" reset --hard origin/rust >/dev/null 2>&1 || true
      git -C "$dir" clean -fd >/dev/null 2>&1 || true
      git -C "$dir" checkout -B "$branch" origin/rust >/dev/null 2>&1 || true
      echo "  Reset EM worktree $squad -> origin/rust" >&2
    fi
  else
    git -C "$ROOT_DIR" worktree add --force "$dir" rust >/dev/null 2>&1 || {
      echo "warning: could not create EM worktree for $squad" >&2
      return 1
    }
    # Create EM branch
    git -C "$dir" checkout -B "$branch" origin/rust >/dev/null 2>&1 || true
  fi
  echo "$dir"
}

# Create EM worktrees
WORKTREE_em_forge=""
WORKTREE_em_anvil=""

for squad in forge anvil; do
  if dir="$(ensure_em_worktree "$squad")"; then
    eval "WORKTREE_em_${squad}=\"$dir\""
    echo "EM-${squad} worktree: $dir"
  fi
done

# =============================================================================
# Ensure worktrees for all workers (using simple variables instead of assoc array)
# =============================================================================
WORKTREE_forge_1=""
WORKTREE_forge_2=""
WORKTREE_forge_3=""
WORKTREE_forge_4=""
WORKTREE_forge_5=""
WORKTREE_anvil_1=""
WORKTREE_anvil_2=""
WORKTREE_anvil_3=""
WORKTREE_anvil_4=""
WORKTREE_anvil_5=""

for squad in forge anvil; do
  for n in 1 2 3 4 5; do
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

# Helper to get EM worktree dir
get_em_worktree_dir() {
  local squad="$1"
  eval "echo \"\$WORKTREE_em_${squad}\""
}

# =============================================================================
# Setup role-specific AGENTS.md for each worktree
# =============================================================================
# Note: Git worktrees share the index, so we use .gitignore'd copies or symlinks
# to untracked locations. We put the role file in a .role/ directory.
setup_role_agents() {
  echo "Setting up role-specific agent instructions..."

  # EM worktrees: create .role/AGENTS.md pointing to SQUAD_LEAD_AGENT.md
  for squad in forge anvil; do
    local em_dir
    em_dir="$(get_em_worktree_dir "$squad")"
    if [ -n "$em_dir" ] && [ -d "$em_dir" ]; then
      mkdir -p "$em_dir/.role"
      cp "$ROOT_DIR/SQUAD_LEAD_AGENT.md" "$em_dir/.role/AGENTS.md" 2>/dev/null || true
      echo "  EM-${squad}: .role/AGENTS.md = SQUAD_LEAD_AGENT.md"
    fi
  done

  # Worker worktrees: create .role/AGENTS.md pointing to worker AGENTS.md
  for squad in forge anvil; do
    for n in 1 2 3 4 5; do
      local worker_dir
      worker_dir="$(get_worktree_dir "$squad" "$n")"
      if [ -n "$worker_dir" ] && [ -d "$worker_dir" ]; then
        mkdir -p "$worker_dir/.role"
        cp "$ROOT_DIR/AGENTS.md" "$worker_dir/.role/AGENTS.md" 2>/dev/null || true
      fi
    done
  done
  echo "  Workers: .role/AGENTS.md = AGENTS.md (worker instructions)"

  # Director: create .role/AGENTS.md in ROOT_DIR
  mkdir -p "$ROOT_DIR/.role"
  cp "$ROOT_DIR/DIRECTOR_AGENT.md" "$ROOT_DIR/.role/AGENTS.md" 2>/dev/null || true
  echo "  Director: .role/AGENTS.md = DIRECTOR_AGENT.md"
}

setup_role_agents

# =============================================================================
# Build commands
# =============================================================================
build_cmd() {
  local base_cmd="$1"
  if [ "$AUTO_RESTART_AGENT" = "1" ]; then
    echo "while true; do ${base_cmd}; sleep ${AGENT_RESTART_DELAY}; done"
  else
    echo "$base_cmd"
  fi
}

director_cmd="$(build_cmd "$AGENT_CMD $AGENT_ARGS")"
em_cmd="$(build_cmd "$AGENT_CMD $AGENT_ARGS")"
worker_cmd="$(build_cmd "$AGENT_CMD $AGENT_ARGS")"

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
# Window 1: Director + EMs (Director left, EMs stacked on right)
# =============================================================================
echo "Setting up Director window with EMs..."

# Pane 0: Director (left side)
tmux send-keys -t "$SESSION:director.0" "bash -lc '$director_cmd'" C-m
tmux select-pane -t "$SESSION:director.0" -T "director" 2>/dev/null || true

# Split right for EM-Forge (pane 1) - uses EM worktree
EM_FORGE_DIR="$(get_em_worktree_dir forge)"
[ -z "$EM_FORGE_DIR" ] && EM_FORGE_DIR="$ROOT_DIR"
tmux split-window -h -t "$SESSION:director.0" -c "$EM_FORGE_DIR"
tmux send-keys -t "$SESSION:director.1" "cd '$EM_FORGE_DIR' && export SQUAD_NAME=forge EM_WORKTREE='$EM_FORGE_DIR' && bash -lc '$em_cmd'" C-m
tmux select-pane -t "$SESSION:director.1" -T "em-forge" 2>/dev/null || true

# Split below EM-Forge for EM-Anvil (pane 2) - uses EM worktree
EM_ANVIL_DIR="$(get_em_worktree_dir anvil)"
[ -z "$EM_ANVIL_DIR" ] && EM_ANVIL_DIR="$ROOT_DIR"
tmux split-window -v -t "$SESSION:director.1" -c "$EM_ANVIL_DIR"
tmux send-keys -t "$SESSION:director.2" "cd '$EM_ANVIL_DIR' && export SQUAD_NAME=anvil EM_WORKTREE='$EM_ANVIL_DIR' && bash -lc '$em_cmd'" C-m
tmux select-pane -t "$SESSION:director.2" -T "em-anvil" 2>/dev/null || true

# Wait for director and EMs to boot
sleep "$START_PAUSE"

# Send start prompts to Director and EMs
tmux send-keys -t "$SESSION:director.0" "$DIRECTOR_START_PROMPT"
sleep "$SEND_ENTER_PAUSE"
tmux send-keys -t "$SESSION:director.0" C-m

tmux send-keys -t "$SESSION:director.1" "$EM_START_PROMPT"
sleep "$SEND_ENTER_PAUSE"
tmux send-keys -t "$SESSION:director.1" C-m

tmux send-keys -t "$SESSION:director.2" "$EM_START_PROMPT"
sleep "$SEND_ENTER_PAUSE"
tmux send-keys -t "$SESSION:director.2" C-m

# =============================================================================
# Helper: Setup squad window (5 workers only, EM is in director window)
# =============================================================================
setup_squad_window() {
  local squad="$1"
  local window="$squad"

  echo "Setting up $squad squad window (workers only)..."

  # Create window for squad workers
  tmux new-window -t "$SESSION" -n "$window" -c "$ROOT_DIR"

  # Create 5 worker panes
  local worker_dir

  # First worker in pane 0
  worker_dir="$(get_worktree_dir "$squad" 1)"
  [ -z "$worker_dir" ] && worker_dir="$ROOT_DIR"
  tmux send-keys -t "$SESSION:$window.0" "cd '$worker_dir'" C-m

  # Workers 2-5 via splits
  for n in 2 3 4 5; do
    worker_dir="$(get_worktree_dir "$squad" "$n")"
    [ -z "$worker_dir" ] && worker_dir="$ROOT_DIR"
    tmux split-window -t "$SESSION:$window" -c "$worker_dir"
    tmux select-layout -t "$SESSION:$window" tiled
  done

  # Final layout balance
  tmux select-layout -t "$SESSION:$window" tiled

  # Rename panes (panes 0-4 are workers 1-5)
  for n in 0 1 2 3 4; do
    local worker_num=$((n + 1))
    tmux select-pane -t "$SESSION:$window.$n" -T "${squad}-${worker_num}" 2>/dev/null || true
  done

  # Start codex in each worker pane (staggered)
  for n in 0 1 2 3 4; do
    local worker_num=$((n + 1))
    local worker_dir
    worker_dir="$(get_worktree_dir "$squad" "$worker_num")"
    [ -z "$worker_dir" ] && worker_dir="$ROOT_DIR"
    tmux send-keys -t "$SESSION:$window.$n" "cd '$worker_dir' && export SQUAD_NAME=$squad WORKER_NUM=$worker_num && bash -lc '$worker_cmd'" C-m
    sleep 2  # Small delay between starting each codex
  done

  # Wait for codex instances to boot
  echo "  Waiting ${START_PAUSE}s for codex to boot in $squad squad..."
  sleep "$START_PAUSE"

  # Send start prompts to workers (staggered)
  for pane in 0 1 2 3 4; do
    sleep "$STAGGER_PAUSE"
    tmux send-keys -t "$SESSION:$window.$pane" "$WORKER_START_PROMPT"
    sleep "$SEND_ENTER_PAUSE"
    tmux send-keys -t "$SESSION:$window.$pane" C-m
  done
}

# =============================================================================
# Window 2: Forge Squad (Type System - solver, checker, binder, types)
# =============================================================================
setup_squad_window "forge"

# =============================================================================
# Window 3: Anvil Squad (Output - emitter, transforms, cli, lsp)
# =============================================================================
setup_squad_window "anvil"

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
echo "  1. director  - Director (pane 0) + EM-Forge (pane 1) + EM-Anvil (pane 2)"
echo "  2. forge     - 5 Workers (type system, panes 0-4)"
echo "  3. anvil     - 5 Workers (output, panes 0-4)"
echo ""
echo "EM Worktrees (for blocker fixes):"
for squad in forge anvil; do
  dir="$(get_em_worktree_dir "$squad")"
  [ -n "$dir" ] && echo "  em-${squad}: $dir (branch: em/$squad)"
done
echo ""
echo "Worker Worktrees:"
for squad in forge anvil; do
  for n in 1 2 3 4 5; do
    dir="$(get_worktree_dir "$squad" "$n")"
    [ -n "$dir" ] && echo "  ${squad}-${n}: $dir"
  done
done
echo ""
echo "Agent CLI: $AGENT_CMD $AGENT_ARGS"
echo ""
echo "Quick navigation:"
echo "  tmux select-window -t $SESSION:director"
echo "  tmux select-window -t $SESSION:forge"
echo "  tmux select-window -t $SESSION:anvil"
echo ""
echo "Attach: tmux attach -t $SESSION"
echo "Kill:   $0 --kill"
echo "Fresh:  $0 --fresh  (reset all branches to origin/rust)"
echo "Codex:  $0 --codex  (use OpenAI Codex instead of Claude)"
echo "=============================================="

# =============================================================================
# Auto-attach
# =============================================================================
if [ "$AUTO_ATTACH" = "1" ] && [ -z "${TMUX:-}" ]; then
  tmux attach -t "$SESSION"
fi
