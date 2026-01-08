# TypeScript → Rust/WASM Migration

> **This file is for WORKERS only.** If you are an EM (Engineering Manager), read `SQUAD_LEAD_AGENT.md` instead. If you are the Director, read `DIRECTOR_AGENT.md`.

## Mission
Migrate TypeScript compiler to Rust/WASM. **Beat TypeScript-Go in performance.**

## 🎯 Philosophy: Performance-First Architecture

We have time. No deadlines. Do it right.

## Your Identity

Check your environment variables to know who you are:
- `SQUAD_NAME`: Your squad (`forge` or `anvil`)
- `WORKER_NUM`: Your worker number (1-5)

Your plan file is at: `wasm/specs/squads/$SQUAD_NAME/worker-${WORKER_NUM}_plan.md`
Your branch is: `worker/$SQUAD_NAME-$WORKER_NUM` (e.g., `worker/forge-1`)

## Squad Structure

| Squad | Focus Areas |
|-------|-------------|
| **Forge** | Type system: `solver/`, `checker/`, `binder/`, `types/` |
| **Anvil** | Output: `thin_emitter/`, `transforms/`, `cli/`, `lsp/` |

Each squad has 1 EM (Engineering Manager) + 5 Workers.

## Worker Plans

Your plan file location: `wasm/specs/squads/<squad>/worker-<N>_plan.md`

You must track todo items and progress in your worker plan file. The EM assigns tasks and priorities there.

## Branching (required)

Each worker uses its own branch. Naming: `worker/<squad>-<N>` (e.g., `worker/forge-1`, `worker/anvil-3`).
- Create once: `git switch -c worker/$SQUAD_NAME-$WORKER_NUM`
- Push: `git push -u origin worker/$SQUAD_NAME-$WORKER_NUM`

### Sync Protocol (CRITICAL - do this before AND after each task)

**Before starting a task:**
```bash
git fetch origin
git merge origin/rust --no-edit
# Resolve any conflicts, then:
git push origin worker/$SQUAD_NAME-$WORKER_NUM
```

**After completing a task:**
```bash
git add -A && git commit -m "[wasm] <component>: <description>"
git push origin worker/$SQUAD_NAME-$WORKER_NUM
# Then mark "Ready for Merge: Yes" in your plan file
```

This ensures:
1. You start with the latest code (fewer conflicts)
2. Your work gets merged into `rust` promptly
3. Other workers see your changes sooner

## Must Read

- `wasm/specs/WASM_ARCHITECTURE.md`
- `wasm/specs/SOLVER.md` (when working on solver-related tasks)

## Workflow (loop)

1. **Sync first**: `git fetch origin && git merge origin/rust --no-edit`
2. Read your plan file.
3. Write code, add tests, run `./wasm/test.sh`.
4. Commit and push to your worker branch.
5. Mark "Ready for Merge: Yes" in your plan.
6. Repeat.


## ✅ Commit Format
```
[wasm] <component>: <description>
```

Commit frequently and atomically

## 🚨 Rules

1. **Stay on your assignment**; do not self-switch tasks.
2. **Docker-only Rust tests**: `./wasm/test.sh` (never `cargo test/bench`).
3. **Separate test files**: `foo.rs` and `foo_tests.rs` or `tests/foo.rs`.
4. **Update your plan** after each task; keep it accurate.
5. **Commit and push** to your worker branch frequently; do not push to `origin/rust` or squad branches.
6. **Sync before EVERY task**: `git fetch origin && git merge origin/rust --no-edit` - this is mandatory, not optional.
7. **Signal readiness**: After pushing, set `Ready for Merge: Yes` in your plan so EM knows to merge.
8. **NEVER edit management files**: Do not touch `DIRECTOR_AGENT.md`, `SQUAD_LEAD_AGENT.md`, `MANAGER_AGENT.md`, `AGENTS.md`, or `start_*.sh` scripts. These are human-owned.

## 🎯 If Blocked

**You are almost never truly blocked. Do not ask permission. Act.**

- Dirty worktree? `git stash` and continue.
- No assignment in plan? Pick the first item from Task Queue.
- Task Queue empty? Add a test for existing code.
- Build error? Fix it.
- Merge conflict? Resolve it.
- Wrong branch? Switch to the right one.

**Stop asking questions. Start writing code.**
