# Project Zang

Project Zang is a performance-first TypeScript compiler in Rust.[^1]
The goal is a correct, fast, drop-in replacement for `tsc`, with both native and WASM targets.

TypeScript is intentionally unsound. Zang keeps a sound core solver and layers a compatibility
engine on top to match TypeScript behavior while preserving correctness where possible.

## Project Direction

> This is a very high level project direction coming from project's manager's boss.

- Don't get too bugged down with performance. lets ship something first that works. 
- lib.d.ts and similar -- lets handle this.
- I want to see `test/cases` baseline pass rates in manager report
- While you have the mechanisms, tsc has thousands of specific error messages. Your diagnostic engine is generic; matching the user experience of tsc errors requires massive effort.
- tsconfig.json has hundreds of flags. You handle the big ones (target, module, strict), but full compatibility is a long tail.
- Your immediate implementation risk: The complexity of solver/subtype.rs and solver/infer.rs suggests you are deep in the weeds of TypeScript's unsound type system. This is where "compatibility bugs" live—cases where your logic makes sense, but TS does something weird for legacy reasons, breaking compatibility with existing codebases. **you need to manage this well**

## Executive Summary (Manager report)
Last updated: 2026-01-07

- Overall: Migration is active; Rust/WASM compiler is under construction and not production-ready.
- Tracks: CLI now includes symbol-level invalidation with basic source maps; checker added switch/case flow narrowing and tightened dependency scoping; solver added cached property lookups; emitter shares directive payloads; LSP is reshuffling signature-help tests while refining overload JSDoc mapping.
- Risk: LSP signature-help tests are currently failing in-track; symbol-level invalidation touches core cache behavior; source maps remain stubbed to a single 0,0 mapping; ES module imports still resolve to `any`; parser/arena child enumeration TODOs remain; namespace member checking tests are still disabled.
- Next focus: Split `thin_emitter/mod.rs` into smaller modules, fix signature-help failures, validate symbol-level invalidation with tests, build real source maps, type ES imports, close parser/arena TODOs, and keep correctness ahead of perf tweaks.

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
