# Project Zang

Project Zang is a performance-first TypeScript compiler in Rust.[^1]
The goal is a correct, fast, drop-in replacement for `tsc`, with both native and WASM targets.

TypeScript is intentionally unsound. Zang keeps a sound core solver and layers a compatibility
engine on top to match TypeScript behavior while preserving correctness where possible.

## Project Direction

> This is a very high level project direction coming from project's manager's boss.

- Don't get too bogged down with performance. let's ship something first that works.
- lib.d.ts and similar -- let's handle this.
- I saw emitter/mod.rs is a giant file. Smaller files are better.
- I want to see `test/cases` baseline pass rates in manager report
- While you have the mechanisms, tsc has thousands of specific error messages. Your diagnostic engine is generic; matching the user experience of tsc errors requires massive effort.
- tsconfig.json has hundreds of flags. You handle the big ones (target, module, strict), but full compatibility is a long tail.
- Your immediate implementation risk: The complexity of solver/subtype.rs and solver/infer.rs suggests you are deep in the weeds of TypeScript's unsound type system. This is where "compatibility bugs" live—cases where your logic makes sense, but TS does something weird for legacy reasons, breaking compatibility with existing codebases. **you need to manage this well**

## Executive Summary (Manager report)
Last updated: 2026-01-07

- Overall: Migration is active; Rust/WASM compiler is under construction and not production-ready.
- Tracks: CLI now maps re-export bindings for symbol-level invalidation; checker added user-defined type predicate narrowing in flow analysis; solver added SmallVec-backed union/intersection helpers and unknown normalization coverage; emitter continues splitting `thin_emitter/mod.rs` (module wrapper + ES5 template helpers, ES5 bindings in progress); LSP fixed hover JSDoc matching for exported vars and signature-help trailing commas.
- Baselines (`tests/cases`, first 100): compiler errors 60/77 (77.9%) pass, JS 40/76 (52.6%) pass; conformance errors 18/90 (20.0%) pass, JS 1/88 (1.1%) pass.
- Risk: ES module imports still resolve to `any` (cross-file types unreliable); symbol-level invalidation still lacks `export *` and `import = require(...)` mapping; source maps remain stubbed to a single 0,0 mapping; parser/arena child enumeration TODOs remain; baseline pass rates are still low on conformance.
- Next focus: finish splitting `thin_emitter/mod.rs`, extend symbol-level invalidation to import-equals + star re-exports, improve baseline pass rates, validate predicate narrowing with broader tests, and keep correctness ahead of perf tweaks.

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
