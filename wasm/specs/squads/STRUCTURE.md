# Squad Structure Guide

This document describes the hierarchical organization of AI agents for Project Zang.

## Organization Chart

```
                    +-----------+
                    |   Human   |
                    | (README)  |
                    +-----+-----+
                          |
                          | Sets "Project Direction"
                          v
                    +-----------+
                    | Director  |
                    | (Window 1)|
                    +-----+-----+
                          |
          +---------------+---------------+
          |                               |
          v                               v
    +------------+                  +------------+
    |  EM-Forge  |                  |  EM-Anvil  |
    | (Window 2) |                  | (Window 3) |
    +-----+------+                  +-----+------+
          |                               |
    +--+--+--+--+--+               +--+--+--+--+--+
    |  |  |  |  |  |               |  |  |  |  |  |
    v  v  v  v  v  v               v  v  v  v  v  v
   W1 W2 W3 W4 W5                 W1 W2 W3 W4 W5
```

## Squad Names

| Squad | Meaning | Focus |
|-------|---------|-------|
| **Forge** | Where metal is shaped by fire | Type system (solver, checker, binder) |
| **Anvil** | Where metal is hammered into form | Output (emitter, transforms, cli, lsp) |

## Directory Layout

```
wasm/specs/squads/
├── STRUCTURE.md          # This file
├── forge/
│   ├── GOALS.md          # Strategic goals from Director
│   ├── worker-1_plan.md  # Worker 1 tasks (EM-Forge assigns)
│   ├── worker-2_plan.md  # Worker 2 tasks
│   ├── worker-3_plan.md  # Worker 3 tasks
│   ├── worker-4_plan.md  # Worker 4 tasks
│   └── worker-5_plan.md  # Worker 5 tasks
└── anvil/
    ├── GOALS.md          # Strategic goals from Director
    ├── worker-1_plan.md  # Worker 1 tasks (EM-Anvil assigns)
    ├── worker-2_plan.md  # Worker 2 tasks
    ├── worker-3_plan.md  # Worker 3 tasks
    ├── worker-4_plan.md  # Worker 4 tasks
    └── worker-5_plan.md  # Worker 5 tasks
```

## Role Responsibilities

### Human (You)
- **Writes:** `wasm/README.md` → "Project Direction" section
- **Reads:** Executive Summary, squad reports
- **Does NOT:** Assign tasks directly to workers

### Director
- **Reads:** `wasm/README.md` (Project Direction)
- **Writes:** `squads/forge/GOALS.md`, `squads/anvil/GOALS.md`
- **Does NOT:** Write code, assign individual worker tasks
- **Focus:** Strategic alignment, cross-squad coordination, risk escalation
- **Can:** Reprioritize squads, reassign focus areas, restructure teams

### Engineering Managers (EMs)
- **Reads:** Their squad's `GOALS.md`
- **Writes:** Worker plan files (`worker-*_plan.md`)
- **Does NOT:** Write code (except plan/doc edits for course correction)
- **Focus:** Task breakdown, worker coordination, merge management
- **Manages:** 5 workers per squad

### Workers
- **Reads:** Their `worker-*_plan.md`
- **Writes:** Code in their assigned crates, their own `worker-*_plan.md` (status updates only)
- **Does NOT:** Self-switch tasks, edit other squads' files, edit management files
- **NEVER edits:** `STRUCTURE.md`, `GOALS.md`, other workers' plans, anything in `orchestrator/`
- **Focus:** Execute assigned tasks, write tests, sync branches

## Squad Ownership (Director can reassign)

### Squad Forge (EM-Forge + 5 Workers)
Default focus areas:
- `wasm/src/solver/` - Type inference, constraint solving
- `wasm/src/checker/` - Type checking logic
- `wasm/src/binder/` - Symbol binding, scope analysis
- `wasm/src/types/` - Type representations

### Squad Anvil (EM-Anvil + 5 Workers)
Default focus areas:
- `wasm/src/thin_emitter/` - JavaScript emission
- `wasm/src/transforms/` - ES5 downleveling, source maps
- `wasm/src/cli/` - Command-line interface
- `wasm/src/lsp/` - Language Server Protocol

## Branch Hierarchy

```
origin/rust                <- Director merges squad branches
    ↑
origin/squad/forge         <- EM-Forge merges worker branches
origin/squad/anvil         <- EM-Anvil merges worker branches
    ↑
origin/worker/<squad>-<N>  <- Workers push here (e.g., worker/forge-1)
```

## Sync Protocol

1. Workers sync from `origin/rust` before each task
2. Workers push to `origin/worker/<squad>-<N>` (e.g., `worker/forge-1`)
3. Workers mark "Ready for merge" in plan
4. **EM merges worker branches into `origin/squad/<squad>`**
5. **Director merges squad branches into `origin/rust`**

This hierarchy ensures:
- Each level only merges from the level below
- Conflicts are resolved at the appropriate level
- `origin/rust` stays clean and always has all work

## Tmux Layout

```
Session: zang-org

Window 1: director
┌─────────────────────────────────────┐
│            Director                 │
└─────────────────────────────────────┘

Window 2: forge (6 panes: 1 EM + 5 Workers)
┌─────────────────┬───────────────────┐
│   EM-Forge      │     Worker 1      │
├─────────────────┼───────────────────┤
│    Worker 2     │     Worker 3      │
├─────────────────┼───────────────────┤
│    Worker 4     │     Worker 5      │
└─────────────────┴───────────────────┘

Window 3: anvil (6 panes: 1 EM + 5 Workers)
┌─────────────────┬───────────────────┐
│   EM-Anvil      │     Worker 1      │
├─────────────────┼───────────────────┤
│    Worker 2     │     Worker 3      │
├─────────────────┼───────────────────┤
│    Worker 4     │     Worker 5      │
└─────────────────┴───────────────────┘
```

## Communication Flow

1. **Human → Director:** Edit `wasm/README.md` Project Direction
2. **Director → EMs:** Write/update `GOALS.md` files
3. **EMs → Workers:** Write/update `worker-*_plan.md` files
4. **Workers → EMs:** Mark "Ready for merge" in plan, push branch
5. **EMs → Director:** Update squad status in GOALS.md
6. **Director → Human:** Update Executive Summary in README.md
