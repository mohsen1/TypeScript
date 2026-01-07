# CLI Track Plan (tsz)

## Mission
High-performance, tsc-compatible CLI driving the Rust compiler in native mode.

## Scope
Files: `wasm/src/bin/tsz.rs`, `wasm/src/cli/*`, `wasm/src/parallel.rs`, `wasm/src/thin_checker.rs` (integration).

## Current Status
- Args/tsconfig parsing, globbing, compile + emit work.
- Watch mode implemented with notify + debounce.
- No incremental compile or module-resolution parity yet.

## Highest-Impact Next Tasks
- [ ] Incremental compilation caches
  - Cache parsed arenas + binder results per file.
  - Reuse `TypeCache` in `ThinCheckerState::with_cache`.
  - Invalidate affected symbols only.
- [ ] Expand tsconfig support
  - [x] baseUrl
  - [x] paths
  - [x] rootDir
  - [x] jsx (preserve/react-native)
  - [ ] sourceMap
  - [ ] declarationMap
  - [x] noEmitOnError
  - [x] lib (resolve `compilerOptions.lib` to src/lib and follow references)
- [x] Respect `--project` and tsconfig inheritance in CLI.
  - `--project` accepts file or directory
  - `extends` chain already supported
- [ ] Module resolution parity
  - [x] Resolve relative + baseUrl/paths imports with TS extension inference.
  - [x] Resolve bare specifiers via node_modules package.json entries + index fallback.
  - [ ] Support Node16/NodeNext exports conditions + subpath exports.
- [ ] Benchmark harness
  - Script for `tsz` vs `tsc` on large repos with timing + memory stats.

## Task Ledger (legacy checklist)

### Phase 1: Foundation & Arguments
- [x] **Scaffold Binary**
  - Add `[[bin]]` entry in `Cargo.toml` for `tsz` (Codename Zang CLI).
  - Add dependencies: `clap` (derive), `anyhow`, `serde`, `serde_json` (with preserve_order).
- [x] **Implement Argument Parsing**
  - Replicate common `tsc` flags: `--target`, `--module`, `--outDir`, `--strict`, `--noEmit`.
  - Implement `--help` and `--version`.

### Phase 2: Project Configuration (tsconfig)
- [x] **JSONC Parsing**
  - [x] Implement `tsconfig.json` parser that handles comments (JSONC).
  - [x] Support `extends` inheritance (recursive loading).
- [x] **Option Mapping**
  - [x] Map `tsconfig` "compilerOptions" to internal `PrinterOptions` (Emitter) and `CheckerOptions`.
  - [x] Handle `include`, `exclude`, and `files` globs.

### Phase 3: The Driver (Orchestration)
- [x] **File Discovery**
  - [x] Implement efficient globbing to find all `.ts` files based on config.
- [x] **Pipeline Connection**
  - [x] Wire up `parallel::compile_files` (Parser/Binder) to the discovered files.
  - [x] Wire up `thin_checker::check_source_file` for type checking.
  - [x] Wire up `thin_emitter::emit` for output generation.
- [x] **Output Writer**
  - [x] Implement parallel file writing for emitted `.js` and `.d.ts` files to `outDir`.
  - [x] Ensure directory structures are created.

### Phase 4: Diagnostics & Reporting
- [x] **Diagnostic Formatter**
  - [x] Implement a reporter that looks like `tsc` (file.ts:line:col - error TS1234: Message).
  - [x] Add color support (`colored` crate).
  - [x] Integrate `solver::diagnostics` output into the CLI reporter.
  - [x] Surface parse diagnostics with source offsets for accurate locations.
- [x] **Exit Codes**
  - [x] Return proper exit codes (0 for success, 1 for errors) based on diagnostic severity.

### Phase 5: Watch Mode (The Speed Demon)
- [x] **File Watching**
  - Integrate `notify` crate for filesystem events.
  - Implement debounce logic.
- [ ] **Incremental Re-compilation**
  - Invalidate specific entries in `ThinParserState` / `Binder` when files change.
  - Re-trigger check/emit only for affected files (and dependents).

### Phase 6: Documentation & Polish
- [ ] **Documentation Website**
  - Set up Docusaurus structure.
  - Auto-generate CLI flag documentation from Clap structs.
  - Write "Migration from tsc" guide.
- [ ] **Benchmarks vs tsc**
  - Create a script to run `tsz` vs `tsc` on large open source repos (e.g., Three.js, React).

## Success Criteria
- `tsz --watch` handles large projects without full reparse.
- CLI builds typical `tsconfig.json` projects with correct output.
- Consistently faster than `tsc` on real-world repos.
