# Project Zang

Project Zang is a performance-first TypeScript compiler in Rust.[^1]
The goal is a correct, fast, drop-in replacement for `tsc`, with both native and WASM targets.

TypeScript is intentionally unsound. Zang keeps a sound core solver and layers a compatibility
engine on top to match TypeScript behavior while preserving correctness where possible.

## Project Direction

> This is a very high level project direction coming from project's manager's boss.

**Current Phase: Phase 8 - Conformance, Convergence, and Hardening**

**Strategic Shift:** We have crossed the threshold of "building the engine." The components (ThinParser, ThinBinder, Solver, ThinEmitter) exist. We are now shifting to **Integration and Correctness**. We are no longer building features in isolation; we are driving the entire pipeline to pass official TypeScript conformance tests.

**Top Priority:** The **Solver** is the bottleneck for correctness. The **Emitter** is the bottleneck for utility. All tracks must prioritize tasks that fix type inference gaps or emission semantics over peripheral features (LSP UI, CLI flags).

### Management Strategy: Autocratic Scheduling
- **The Manager** is the single source of truth for priority.
- **Tracks** are generic workers. If the Solver needs 3 workers, the Manager assigns 3 workers to the Solver, regardless of their previous "track name."
- **Zero-Idle:** If a high-priority task is blocked, swarm it.

### Critical Objectives (Ranked)

1.  **Solver Hardening (The "Brain")**
    *   **Generic Inference:** `solver/infer.rs` is critical. Focus on inference from usage, context-sensitive typing, and handling circular constraints in `extends` clauses.
    *   **Conditional Types:** Stress test `solver/evaluate.rs` with distributive conditional types over unions. This is where most "toy" compilers fail.
    *   **Structural Compatibility:** Verify `subtype.rs` handles variance correctly (covariance for results, contravariance for parameters) in all edge cases.

2.  **Emitter Fidelity (The "Voice")**
    *   **ES5 Downleveling:** Ensure `transforms/class_es5.rs` and `async_es5.rs` produce semantically identical JavaScript to `tsc`. Edge cases: `super()` calls in derived classes with property initializers, and `this` capture in deeply nested arrow/async functions.
    *   **Source Maps:** Verify `source_writer.rs` generates valid maps that debuggers can actually attach to.

3.  **End-to-End Validation**
    *   Stop adding AST nodes. Start compiling real code.
    *   **Metric:** Successfully compile a non-trivial generic library (e.g., `redux` or `lodash` types) without panicking.

### Anti-Priorities (Do Not Work On)
*   New LSP features (Semantic Tokens, Code Actions) unless they expose a Solver bug.
*   CLI argument parsing or fancy terminal output.
*   Performance micro-optimizations (unless we regress significantly).


## Executive Summary (Manager report)
Last updated: 2026-01-07

- Overall: Migration remains active; Rust/WASM compiler is under construction and not production-ready.
- Tracks: CLI added focused binder regression tests for the binder.ts:331 parameter binding gap and re-ran benches (still stops at `binder.ts:432` with `statements` missing); emitter `.d.ts` now covers default re-export specifiers with parser coverage; checker routes tuple literal element access through the solver so optional elements include `undefined`; LSP stabilized nested function-body edit cache timing to preserve prefix symbols and scope reuse; solver expanded TS_UNSOUNDNESS coverage for homomorphic mapped types over primitives and noUnchecked object index signatures.
- Baselines (`tests/cases`, first 100): compiler errors 60/77 (77.9%) pass, JS 40/76 (52.6%) pass; conformance errors 18/90 (20.0%) pass, JS 1/88 (1.1%) pass (no new run).
- Risk: CLI bench blocked by TS2304 in `binder.ts:432`; ES module imports still resolve to `any`; source maps remain minimal; parser/arena child enumeration TODOs remain; conformance baseline pass rates are low.
- Next focus: fix the binder.ts:432 scope wiring and rerun real repo benchmarks, keep pushing assignability/member lookup into solver to end split-brain typing, remove remaining inline emitter transforms in favor of directives, tighten LSP incremental edits that touch suffix scopes, and retire legacy AST with arena memory-hygiene checks.

## Status
This project is not ready for general use yet. The interface and distribution are in progress.

## Planned distribution
- `tsz` CLI (native binaries for major operating systems)
- Rust crate `tsz` (library + CLI)
- WASM bindings
- npm package `@tsz/tsz` (primary)
- compat package `@tsz/tsc` that exposes a `tsc` executable so tooling can swap without noticing
- Playground

## Guiding principles
- Make tsz boringly correct before it is fast. Parity with `tsc` output, errors, and edge cases is the trust anchor.
- Measure everything: benchmark real repos, gate regressions, and only optimize hot paths that move real workloads.
- Determinism wins adoption: stable outputs, stable diagnostics, stable perf, and zero "sometimes" behavior.
- Incrementality is a feature, not a refactor: design caches and query boundaries up front so LSP/CLI stay snappy.
- UX matters as much as speed: error messages, source maps, and CLI flag compatibility are what teams feel every day.
- Keep the architecture clean and enforced. Performance-first is a habit, not a phase.


[^1]: Zang is Persian for rust.
