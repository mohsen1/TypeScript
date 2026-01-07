# Project Zang

Project Zang is a performance-first TypeScript compiler in Rust.[^1]
The goal is a correct, fast, drop-in replacement for `tsc`, with both native and WASM targets.

TypeScript is intentionally unsound. Zang keeps a sound core solver and layers a compatibility
engine on top to match TypeScript behavior while preserving correctness where possible.

## Executive Summary (Manager report)
Last updated: 2026-01-07

- Overall: Migration is active; Rust/WASM compiler is under construction and not production-ready.
- Tracks: CLI now has export-hash invalidation with basic source map output; LSP maps JSDoc to overload signatures; checker expanded control-flow narrowing (identifier flows, logical conditions); solver reduced atom-resolve allocations; emitter refactored ES5 emit helpers.
- Risk: Source maps are stubbed to a single 0,0 mapping; ES module imports still resolve to `any`; parser/arena child enumeration TODOs remain; namespace member checking tests are still disabled.
- Next focus: Build real source maps, add symbol-level invalidation, type ES imports, re-enable namespace member checking, close parser/arena TODOs, and keep trimming emitter hot paths.

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
