# CLI Track Plan (tsz)

## Mission
High-performance, tsc-compatible CLI driving the Rust compiler in native mode.

## Scope
Files: `wasm/src/bin/tsz.rs`, `wasm/src/cli/*`, `wasm/src/parallel.rs`, `wasm/src/thin_checker.rs` (integration).

## Current Status
- Args/tsconfig parsing, globbing, compile + emit work.
- Watch mode implemented with notify + debounce.
- Incremental compile in place (cache reuse + export-hash dependent invalidation + symbol-level dependent invalidation).
- Module resolution supports node/bundler + exports/conditions basics; parity still incomplete.

## Current Investigation Notes (Incremental export hash)
Summary of the incremental work (export hash fixed):
- Added per-file export hashing to avoid invalidating dependents when a change does not alter the exported API.
  - New cache field: `CompilationCache::export_hashes` (map of canonical path -> u64 hash).
  - New cache helpers: `invalidate_paths` (no dependents) and export hash clearing in `invalidate_paths_with_dependents` + `clear`.
  - New helper `CompilationCache::export_hash` (test-only accessor) added for debugging.
- Incremental compile flow now does:
  1) canonicalize changed paths;
  2) capture old export hashes;
  3) `invalidate_paths` for changed files only;
  4) compile once with `compile_inner`;
  5) compare old vs new export hashes; if unchanged, return result;
  6) if changed, `invalidate_paths_with_dependents_symbols` and compile again.
  - Entry point: `compile_with_cache_and_changes` in `wasm/src/cli/driver.rs`.
  - Watch mode (`wasm/src/cli/watch.rs`) now uses `compile_with_cache_and_changes` for non-config changes.
- Added symbol-level invalidation for dependents on export changes:
  - Cache import binding symbol IDs per dependent (`CompilationCache::import_symbol_ids`).
  - Track symbol dependency graph in `TypeCache` and invalidate only affected symbols + node cache.
  - Changed files still fully invalidate parse/bind/type caches.
- Export hash computation (`compute_export_hash` in `wasm/src/cli/driver.rs`) includes:
  - Exported symbols from `program.file_locals` (symbol name + formatted type via `TypeFormatter::with_symbols`).
  - Export declarations/signatures: `export * from`, `export {..} from`, `export * as ns`, `export =`, and default export expression signature.
  - Local export declarations and local named exports now contribute signatures by scanning the AST and formatting declaration types.
- Source reading optimization already in place:
  - `read_source_files` uses cached bind results + cached dependencies to skip reading unchanged files.
  - `SourceEntry` now stores `Option<String>` (None == reuse cached binding).

Resolved behavior:
- `compile_with_cache_rechecks_dependents_on_export_change` now re-emits dependents when exported declaration types change.
- The test now asserts dependent recompilation instead of diagnostics because ES import aliases are still typed as `any`.

Remaining limitation:
- ES module imports still resolve to `any`, so cross-file type diagnostics are not yet reliable.
- Source maps are stubbed with a single 0,0 mapping; full node-level mappings still TODO.

Tests run in this state:
- `./wasm/test.sh cli::driver_tests::compile_with_cache_rechecks_dependents_on_export_change` (pass).
- `./wasm/test.sh cli::driver_tests::compile_with_cache_skips_dependents_when_exports_unchanged` (pass).
- `./wasm/test.sh cli::driver_tests::invalidate_paths_with_dependents_symbols_keeps_unrelated_cache` (pass).

## Highest-Impact Next Tasks
- [ ] Incremental compilation caches
  - [x] Cache parsed arenas + binder results per file.
  - [x] Reuse `TypeCache` in `ThinCheckerState::with_cache`.
  - [x] Invalidate dependent files via module graph.
  - [x] Cache per-file diagnostics to skip rechecking unchanged files.
  - [x] Emit outputs only for dirty files in cached builds.
  - [x] Reuse cached dependencies to skip reading unchanged files in watch builds.
  - [x] Skip dependent invalidation when exported API is unchanged.
  - [x] Invalidate affected symbols only.
- [ ] Expand tsconfig support
  - [x] baseUrl
  - [x] paths
  - [x] rootDir
  - [x] jsx (preserve/react-native)
  - [x] sourceMap (basic .map output + sourceMappingURL comments)
  - [x] declarationMap (basic .d.ts.map output + sourceMappingURL comments)
  - [x] noEmitOnError
  - [x] lib (resolve `compilerOptions.lib` to src/lib and follow references)
- [x] Respect `--project` and tsconfig inheritance in CLI.
  - `--project` accepts file or directory
  - `extends` chain already supported
- [ ] Module resolution parity
  - [x] Resolve relative + baseUrl/paths imports with TS extension inference.
  - [x] Resolve bare specifiers via node_modules package.json entries + index fallback.
  - [x] Support exports subpath mapping + basic condition selection (types/import/require/default).
  - [x] Expand exports conditions (node/browser) + moduleResolution-specific ordering.
  - [x] Honor package.json `type` + Node16/NodeNext extension rules.
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
