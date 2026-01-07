# TypeScript → Rust/WASM Migration

## Mission
Migrate TypeScript compiler to Rust/WASM. **Beat TypeScript-Go in performance.**

## 🎯 Philosophy: Performance-First Architecture

We have time. No deadlines. Do it right.

## Worker Plans

There are 5 generic worker tracks. Each worker manages its progress in a plan file in `wasm/specs` that matches its worktree name.

- `worker-1`: `wasm/specs/worker-1_plan.md`
- `worker-2`: `wasm/specs/worker-2_plan.md`
- `worker-3`: `wasm/specs/worker-3_plan.md`
- `worker-4`: `wasm/specs/worker-4_plan.md`
- `worker-5`: `wasm/specs/worker-5_plan.md`

You must track todo items and progress in your worker plan file. The manager assigns tasks and priorities there.


## Must Read

- `wasm/specs/WASM_ARCHITECTURE.md`
- `wasm/specs/SOLVER.md` (when working on solver-related tasks)

## Workflow (loop)

1. Read your worker *_plan.md and manager assignment.
2. Execute the highest-impact assigned task (no self-switching).
3. Add tests and run `./wasm/test.sh` (Docker only).
4. Update your plan, commit, and sync `origin/rust`.
5. Repeat.


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
5. **Commit and sync** with `origin/rust` frequently.

## 🎯 If Blocked

- Ask the manager for the next assignment and propose high-impact tasks.
- Add tests, tighten architecture compliance, and polish performance or diagnostics.
