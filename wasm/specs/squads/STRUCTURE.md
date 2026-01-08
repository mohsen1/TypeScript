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
    | EM-Solver  |                  | EM-Tools   |
    | (Window 2) |                  | (Window 3) |
    +-----+------+                  +-----+------+
          |                               |
    +-----+-----+-----+             +-----+-----+-----+
    |     |     |     |             |     |     |     |
    v     v     v     v             v     v     v     v
   W1    W2    W3    W4            W1    W2    W3    W4
```

## Directory Layout

```
wasm/specs/squads/
├── STRUCTURE.md          # This file
├── solver/
│   ├── GOALS.md          # Strategic goals from Director
│   ├── worker-1_plan.md  # Worker 1 tasks (EM-Solver assigns)
│   ├── worker-2_plan.md  # Worker 2 tasks
│   ├── worker-3_plan.md  # Worker 3 tasks
│   └── worker-4_plan.md  # Worker 4 tasks
└── tools/
    ├── GOALS.md          # Strategic goals from Director
    ├── worker-1_plan.md  # Worker 1 tasks (EM-Tools assigns)
    ├── worker-2_plan.md  # Worker 2 tasks
    ├── worker-3_plan.md  # Worker 3 tasks
    └── worker-4_plan.md  # Worker 4 tasks
```

## Role Responsibilities

### Human (You)
- **Writes:** `wasm/README.md` → "Project Direction" section
- **Reads:** Executive Summary, squad reports
- **Does NOT:** Assign tasks directly to workers

### Director
- **Reads:** `wasm/README.md` (Project Direction)
- **Writes:** `squads/solver/GOALS.md`, `squads/tools/GOALS.md`
- **Does NOT:** Write code, assign individual worker tasks
- **Focus:** Strategic alignment, cross-squad coordination, risk escalation

### Engineering Managers (EMs)
- **Reads:** Their squad's `GOALS.md`
- **Writes:** Worker plan files (`worker-*_plan.md`)
- **Does NOT:** Write code (except plan/doc edits for course correction)
- **Focus:** Task breakdown, worker coordination, merge management
- **Manages:** 4 workers per squad

### Workers
- **Reads:** Their `worker-*_plan.md`
- **Writes:** Code in their assigned crates
- **Does NOT:** Self-switch tasks, edit other squads' files
- **Focus:** Execute assigned tasks, write tests, sync branches

## Squad Ownership

### Squad Solver (EM-Solver + 4 Workers)
Owns these crates/directories:
- `wasm/src/solver/` - Type inference, constraint solving
- `wasm/src/checker/` - Type checking logic
- `wasm/src/binder/` - Symbol binding, scope analysis
- `wasm/src/types/` - Type representations

### Squad Tools (EM-Tools + 4 Workers)
Owns these crates/directories:
- `wasm/src/lsp/` - Language Server Protocol
- `wasm/src/cli/` - Command-line interface
- `wasm/src/thin_emitter/` - JavaScript emission
- `wasm/src/transforms/` - ES5 downleveling, source maps

## File Naming Conventions

### GOALS.md
Written by Director. Format:
```markdown
# Squad [Name] Goals

Updated: YYYY-MM-DD

## Current Milestone
[One-line description]

## Objectives (Ranked)
1. **[Objective Name]**
   - Context: [why this matters]
   - Success Criteria: [measurable outcome]
   - Estimated Complexity: [Low/Medium/High]

2. ...

## Anti-Priorities
- [Things NOT to work on]

## Notes to EM
- [Any context or constraints]
```

### worker-*_plan.md
Written by EM. Format:
```markdown
# Worker [N] Plan

## Mission
[Squad-level mission statement]

Status: Active
Priority: [1-3]

## Current Assignment
- [Specific task with file paths]

## Task Queue
- [ ] [Next task]
- [ ] [Future task]

## Completed
- [x] [Done task with notes]

## Notes
- [Any relevant context]
```

## Branch Hierarchy

```
origin/rust                <- Director merges squad branches here
    ↑
origin/squad/solver        <- EM-Solver merges worker branches here
origin/squad/tools         <- EM-Tools merges worker branches here
    ↑
origin/worker/<squad>-<N>  <- Workers push here (e.g., worker/solver-1)
```

## Sync Protocol

1. Workers sync from `origin/rust` before each task
2. Workers push to `origin/worker/<squad>-<N>` (e.g., `worker/solver-1`)
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

Window 2: solver (5 panes: 1 EM + 4 Workers)
┌─────────────────┬───────────────────┐
│   EM-Solver     │     Worker 1      │
├─────────────────┼───────────────────┤
│    Worker 2     │     Worker 3      │
├─────────────────┼───────────────────┤
│                 │     Worker 4      │
└─────────────────┴───────────────────┘

Window 3: tools (5 panes: 1 EM + 4 Workers)
┌─────────────────┬───────────────────┐
│   EM-Tools      │     Worker 1      │
├─────────────────┼───────────────────┤
│    Worker 2     │     Worker 3      │
├─────────────────┼───────────────────┤
│                 │     Worker 4      │
└─────────────────┴───────────────────┘
```

## Communication Flow

1. **Human → Director:** Edit `wasm/README.md` Project Direction
2. **Director → EMs:** Write/update `GOALS.md` files
3. **EMs → Workers:** Write/update `worker-*_plan.md` files
4. **Workers → EMs:** Mark "Ready for merge" in plan, push branch
5. **EMs → Director:** Update squad status in GOALS.md
6. **Director → Human:** Update Executive Summary in README.md
