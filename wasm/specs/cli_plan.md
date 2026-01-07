# CLI Track Plan (stc)

## Mission
High-performance, tsc-compatible CLI driving the Rust compiler in native mode.

## Scope
Files: `wasm/src/bin/stc.rs`, `wasm/src/cli/*`, `wasm/src/parallel.rs`, `wasm/src/thin_checker.rs` (integration).

## Current Status
- Args/tsconfig parsing, globbing, compile + emit work.
- No watch mode or incremental compile.
- Compiler options surface is minimal (target/module/strict/noEmit/paths).

## Highest-Impact Next Tasks
- [ ] Watch mode + debounce
  - Integrate `notify` and change batching.
  - Re-run compile on changed files only.
- [ ] Incremental compilation caches
  - Cache parsed arenas + binder results per file.
  - Reuse `TypeCache` in `ThinCheckerState::with_cache`.
  - Invalidate affected symbols only.
- [ ] Expand tsconfig support
  - `baseUrl`, `paths`, `rootDir`, `jsx`, `sourceMap`, `declarationMap`, `noEmitOnError`, `lib`.
  - Respect `--project` and tsconfig inheritance in CLI.
- [ ] Module resolution parity
  - Support Node16/NodeNext resolution and `.d.ts` lookup.
  - Improve path mapping + extension inference.
- [ ] Benchmark harness
  - Script for `stc` vs `tsc` on large repos with timing + memory stats.

## Success Criteria
- `stc --watch` handles large projects without full reparse.
- CLI builds typical `tsconfig.json` projects with correct output.
- Consistently faster than `tsc` on real-world repos.
