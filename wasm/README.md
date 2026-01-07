# Project Zang

Project Zang is a performance-first TypeScript compiler in Rust.[^1]
The goal is a correct, fast, drop-in replacement for `tsc`, with both native and WASM targets.

TypeScript is intentionally unsound. Zang keeps a sound core solver and layers a compatibility
engine on top to match TypeScript behavior while preserving correctness where possible.

## Project Direction

> This is a very high level project direction coming from project's manager's boss.

**I want to see you using all tracks of development aggressively. no track should sit idle**

The system has a "Ferrari engine" (ThinNode AST + Parallel Binder) but needs a finished transmission (Solver integration).

### 1. Consolidate the "Split Brain" Type System (High Priority)
The project currently has friction between the imperative `ThinChecker` (AST walking) and the declarative `solver/` (structural typing).
*   **Goal:** Move all assignability checks, type relationships, and member lookups into `solver/` behind the `QueryDatabase` trait.
*   **Action:** `ThinChecker` must become a thin traversal layer that pushes constraints into the solver and pulls diagnostics out, rather than doing logic itself.

### 2. Emitter & Compatibility Strategy
To function as a true drop-in replacement for `tsc`, we **will support ES5 down-leveling** (classes to IIFEs, async to generators).
*   **Constraint:** **ES3 support is explicitly out of scope.**
*   **Architecture:** Continue using the **Projection Layer** pattern (`TransformContext`). Do not mutate the AST for transforms; map `NodeIndex` to `TransformDirective` to keep the parallel parser zero-copy and thread-safe.
*   **Focus:** Ensure the `.d.ts` emitter correctly handles symbol visibility and re-exports; this is the "graduation requirement" for library support.

### 3. LSP Incrementality
The current `ThinNodeArena` makes in-place mutation difficult.
*   **Goal:** Sub-10ms response time on keypress.
*   **Action:** Refine `IncrementalParseResult` to ensure small edits inside function bodies do not trigger a full file re-bind or global symbol table invalidation.

### 4. Technical Debt & Cleanup
*   **Deprecate Legacy AST:** Aggressively remove `parser/ast` (the 208-byte fat nodes). The future is `parser/thin_node` (16-byte packed nodes).
*   **Memory Hygiene:** Ensure `ThinNodeArena`s are swapped and dropped correctly during long-running LSP sessions to prevent memory leaks.


## Executive Summary (Manager report)
Last updated: 2026-01-07

- Overall: Migration is active; Rust/WASM compiler is under construction and not production-ready.
- Tracks: CLI typesVersions compiler version is now configurable (flag/env) with fallback tests; emitter emits minimal real source maps for no-transform JS with tests; checker cleared destructuring defaults/aliases and re-enabled namespace value member tests; LSP is still refining IncrementalParseResult to avoid full rebinds on small edits; solver fixed tuple rest index access, optional tuple undefined handling, and negative/fractional tuple indices.
- Baselines (`tests/cases`, first 100): compiler errors 60/77 (77.9%) pass, JS 40/76 (52.6%) pass; conformance errors 18/90 (20.0%) pass, JS 1/88 (1.1%) pass.
- Risk: ES module imports still resolve to `any` (cross-file types unreliable); source maps are only minimal for no-transform JS (transform paths and .d.ts maps still stubbed); parser/arena child enumeration TODOs remain; baseline pass rates are still low on conformance; assignment/flow clearing uses conservative heuristics.
- Next focus: close module-resolution parity gaps (including config-file support for typesVersions override), ship transform/declaration source maps, implement incremental rebind improvements in LSP, finish flow clearing edge cases, type ES imports, improve baseline pass rates, and keep correctness ahead of perf tweaks.

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
