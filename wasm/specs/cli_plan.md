# CLI Track Plan (tsz)

## Mission
High-performance, tsc-compatible CLI driving the Rust compiler in native mode.

## Scope
Files: `wasm/src/bin/tsz.rs`, `wasm/src/cli/*`, `wasm/src/parallel.rs`, `wasm/src/thin_checker.rs` (integration).

## Current Status
- Args/tsconfig parsing, globbing, compile + emit work.
- Watch mode implemented with notify + debounce.
- Incremental compile in place (cache reuse + export-hash dependent invalidation + symbol-level dependent invalidation).
- Module resolution supports node/bundler + exports/conditions basics + typesVersions + package.json `imports` mappings (default TS version 6.0.0, override via flag/env/tsconfig; precedence CLI > env > config > default; env var `TSZ_TYPES_VERSIONS_COMPILER_VERSION`).
- tsconfig `types`/`typeRoots` packages included in the root file set.
- package.json `imports` condition selection coverage (require vs import).
- Active: benchmarks vs tsc on large repos.
- Benchmark harness script added for tsz vs tsc comparisons.
- File discovery can follow symlinks when `TSZ_FOLLOW_SYMLINKS=1` (bench helper).
- Bench attempt on `src/compiler/tsconfig.json` failed in `tsz` (unsupported syntax + lib parsing errors). Next: address remaining parse gaps or pick a compatible large repo / bench-specific tsconfig that avoids libs.
- Optional chaining parse now accepts optional calls with type arguments (`obj?.<T>(...)`) to unblock compiler sources.
- Call argument lists now accept spread elements (`foo(...args)`), which previously caused parser sync loss in compiler sources.
- As/satisfies expressions now bind before `||`/`&&` so `(... as T) || fallback` parses correctly.
- Expression parsing now treats keywords as identifier names when used in value positions (e.g., `const set = ...; set.add(...)`).
- Arrow function detection now accepts keyword identifiers (`symbol => symbol`) as single-parameter arrows.
- Type predicate parsing now accepts keyword identifiers (`symbol is Symbol`) as predicate parameters.
- Statement parsing now treats `namespace`/`module` as expressions when not followed by a declaration name (e.g., `namespace = x`).
- Statement parsing now treats `type` as an identifier in expression statements (e.g., `type.prop = value`).
- Bench helper: suppress speculative type-argument diagnostics so `index < (chain.length - 1)` parses as relational (checker.ts:8505:110).
- Bench helper: arrow lookahead parses return types after `)` to avoid false `=>` expectation in conditional branches (checker.ts:15885).
- Focused parser coverage for checker.ts:15885 (`everyType` arrow with optional chaining + ternary/comma) via `test_thin_parser_checker_every_type_arrow_optional_chain` and `test_thin_parser_checker_every_type_arrow_optional_chain_line`.
- Bound file merge now preserves persistent scopes for stateless checking; CLI binder reconstruction uses them to resolve locals.
- Focused checker regression: `test_thin_checker_resolves_function_parameter_from_bound_state`.
- Focused binder regressions: `test_thin_binder_resolves_parameter_from_bound_state` (simple bound-state param), `test_thin_binder_resolves_parameter_from_bound_state_module_instance_state` (binder.ts:331:9 `node` param), `test_thin_binder_resolves_parameter_from_bound_state_module_instance_state_with_visited` (binder.ts:331:9 with optional `visited` param), `test_thin_binder_resolves_parameter_from_bound_state_binder_ts_331` (binder.ts:331:9 exact signature), and `test_thin_binder_resolves_parameter_from_bound_state_binder_ts_331_without_scopes` (fallback when scopes missing).
- Bound-state binder now falls back to parameter lookup when persistent scopes are missing, so identifier resolution still works for function parameters.
- For-in/of nodes now set parent pointers so bound-state scope lookup can walk to block locals; added `test_thin_binder_resolves_block_local_from_bound_state_binder_ts_432` (resolves binder.ts:432:37 `statements` lookup).
- Latest bench gap from rerun after binder.ts:432 fix: `src/compiler/binder.ts:575:33` (TS2693: `Set` only refers to a type, but is being used as a value here).
- Latest attempt: `npm install --no-save --no-package-lock typescript @types/node`, `cargo build --release --bin tsz`, `./wasm/bench_cli.sh --repo . --tsconfig src/compiler/tsconfig.json --runs 3 --warmup 1` → tsz failed before timing; tsc not run.
- Bench harness update: fixed BSD `/usr/bin/time -l` parsing in `wasm/bench_cli.sh` so elapsed time is read from the `real` token.
- Synthetic benchmark (1000-file project in `/tmp/tsz_bench_large` with minimal `globals.d.ts`): `./wasm/bench_cli.sh --repo /tmp/tsz_bench_large --tsconfig tsconfig.json --runs 3 --warmup 1 --tsz <repo>/wasm/target/release/tsz --tsc <repo>/node_modules/.bin/tsc` → tsz avg 0.170s best 0.170s max_rss 19.6 MiB; tsc avg 0.200s best 0.200s max_rss 141.3 MiB. Next: run on real repo once optional chaining + lib parsing land.
- Bench-specific config added: `bench/tsconfig.bench.json` with `bench/globals.d.ts` (minimal libs/types). Generated 1000-file synthetic project with `python3 bench/generate_synth_project.py --count 1000` (local `bench/synth/`) and ran `./wasm/bench_cli.sh --repo . --tsconfig bench/tsconfig.bench.json --runs 3 --warmup 1 --tsz <repo>/wasm/target/release/tsz --tsc <repo>/node_modules/.bin/tsc` → tsz avg 0.180s best 0.180s max_rss 19.7 MiB; tsc avg 0.200s best 0.200s max_rss 143.2 MiB.
- Note: without `bench/globals.d.ts` or a lib, `tsc` fails with missing global type errors (Array/Boolean/etc.), so the bench config keeps libs/types empty and supplies minimal globals.
- Symlinked synth attempt: generated `/tmp/tsz_bench_large` and symlinked `bench/synth -> /tmp/tsz_bench_large/src`, then ran `./wasm/bench_cli.sh --repo . --tsconfig bench/tsconfig.bench.json --runs 3 --warmup 1 --tsz <repo>/wasm/target/release/tsz --tsc <repo>/node_modules/.bin/tsc` → tsz avg 0.003s best 0.000s max_rss 8.3 MiB; tsc avg 0.297s best 0.270s max_rss 154.5 MiB. Note: tsz file discovery uses `WalkDir` without following symlinks, so `bench/synth` was skipped and tsz effectively compiled only `bench/globals.d.ts` (results not representative).
- TSC lib error repro (no globals + lib []): `/Users/claude/code/TypeScript-cli-track/node_modules/.bin/tsc --project /tmp/tsz_bench_large/tsconfig.noglobals.json --pretty false --noEmit` with a synthetic file using `Promise` yields `error TS2583: Cannot find name 'Promise'` (twice).

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
- `./wasm/test.sh cli::driver_tests::invalidate_paths_with_dependents_symbols_handles_reexports` (pass).
- `./wasm/test.sh cli::driver_tests::invalidate_paths_with_dependents_symbols_handles_import_equals` (pass).
- `./wasm/test.sh cli::driver_tests::invalidate_paths_with_dependents_symbols_handles_namespace_reexports` (pass).
- `./wasm/test.sh cli::driver_tests::invalidate_paths_with_dependents_symbols_handles_star_reexports` (pass).
- `./wasm/test.sh cli::driver_tests::compile_resolves_node_modules_types_versions` (pass).
- `./wasm/test.sh cli::driver_tests::compile_resolves_node_modules_types_versions_best_match` (pass).
- `./wasm/test.sh cli::driver_tests::compile_resolves_node_modules_types_versions_prefers_specific_range` (pass).
- `./wasm/test.sh cli::driver_tests::compile_resolves_node_modules_types_versions_falls_back_to_wildcard` (pass).
- `./wasm/test.sh cli::driver_tests::compile_resolves_node_modules_types_versions_respects_cli_version_override` (pass).
- `./wasm/test.sh cli::driver_tests::compile_resolves_node_modules_types_versions_invalid_override_falls_back` (pass).
- `./wasm/test.sh cli::driver_tests::compile_resolves_node_modules_types_versions_respects_env_version_override` (pass).
- `./wasm/test.sh cli::driver_tests::compile_resolves_node_modules_types_versions_invalid_env_falls_back` (pass).
- `./wasm/test.sh cli::driver_tests::compile_resolves_node_modules_types_versions_respects_tsconfig_version_override` (pass).
- `./wasm/test.sh cli::driver_tests::compile_resolves_node_modules_types_versions_tsconfig_extends_inherits_override` (pass).
- `./wasm/test.sh cli::driver_tests::compile_resolves_node_modules_types_versions_env_overrides_tsconfig` (pass).
- `./wasm/test.sh cli::driver_tests::compile_resolves_node_modules_types_versions_cli_overrides_env_and_tsconfig` (pass).
- `./wasm/test.sh cli::driver_tests::compile_resolves_node_modules_types_versions_invalid_tsconfig_falls_back` (pass).
- `./wasm/test.sh cli::driver_tests::compile_resolves_node_modules_types_versions_empty_env_uses_tsconfig` (pass).
- `./wasm/test.sh cli::driver_tests::compile_resolves_package_imports_wildcard` (pass).
- `./wasm/test.sh cli::driver_tests::compile_resolves_package_imports_prefers_types_condition` (pass).
- `./wasm/test.sh cli::driver_tests::compile_resolves_package_imports_prefers_require_condition_for_commonjs` (pass).
- `./wasm/test.sh cli::driver_tests::compile_resolves_package_imports_prefers_import_condition_for_esm` (pass).
- `./wasm/test.sh cli::driver_tests::compile_resolves_tsconfig_types_includes_selected_packages` (pass).
- `./wasm/test.sh cli::driver_tests::compile_resolves_tsconfig_type_roots_includes_packages` (pass).
- `./wasm/bench_cli.sh --repo . --tsconfig src/compiler/tsconfig.json --runs 3 --warmup 1` (failed: tsz diagnostics on optional chaining + lib .d.ts parsing).
- `./wasm/bench_cli.sh --repo . --tsconfig src/compiler/tsconfig.json --runs 3 --warmup 1` (failed again: tsz exits with diagnostics; no timings).
- `./wasm/bench_cli.sh --repo . --tsconfig src/compiler/tsconfig.json --runs 1 --warmup 1` (failed: tsz diagnostics; parse gap in spread call arguments).
- `./wasm/bench_cli.sh --repo . --tsconfig src/compiler/tsconfig.json --runs 1 --warmup 1` (failed: tsz diagnostics; first parse error at `src/compiler/checker.ts:8505` (TS1005 `)` expected)).
- `./wasm/bench_cli.sh --repo . --tsconfig src/compiler/tsconfig.json --runs 1 --warmup 1` (rerun: failed; first parse error still `src/compiler/checker.ts:8505` (TS1005 `)` expected)).
- `./wasm/bench_cli.sh --repo . --tsconfig src/compiler/tsconfig.json --runs 1 --warmup 1` (failed: tsz diagnostics; remaining parse gaps now in checker.ts, starting around `WriteTypeParametersInQualifiedName` and later keyword identifier statements).
- `./wasm/bench_cli.sh --repo /tmp/tsz_bench_large --tsconfig tsconfig.json --runs 3 --warmup 1 --tsz <repo>/wasm/target/release/tsz --tsc <repo>/node_modules/.bin/tsc` (tsz avg 0.170s best 0.170s max_rss 19.6 MiB; tsc avg 0.200s best 0.200s max_rss 141.3 MiB).
- `python3 bench/generate_synth_project.py --count 1000` (generated local `bench/synth`).
- `./wasm/bench_cli.sh --repo . --tsconfig bench/tsconfig.bench.json --runs 3 --warmup 1 --tsz <repo>/wasm/target/release/tsz --tsc <repo>/node_modules/.bin/tsc` (tsz avg 0.180s best 0.180s max_rss 19.7 MiB; tsc avg 0.200s best 0.200s max_rss 143.2 MiB).
- `./wasm/bench_cli.sh --repo . --tsconfig bench/tsconfig.bench.json --runs 3 --warmup 1 --tsz <repo>/wasm/target/release/tsz --tsc <repo>/node_modules/.bin/tsc` (symlinked `bench/synth` → `/tmp/tsz_bench_large/src`: tsz avg 0.003s best 0.000s max_rss 8.3 MiB; tsc avg 0.297s best 0.270s max_rss 154.5 MiB; tsz skipped symlinked files).
- `./wasm/test.sh cli::fs_tests::discover_files_follow_links_when_enabled` (pass).
- `./wasm/test.sh thin_parser_tests::test_thin_parser_optional_chain_call_with_type_arguments` (pass).
- `./wasm/test.sh thin_parser_tests::test_thin_parser_spread_in_call_arguments` (pass).
- `./wasm/test.sh thin_parser_tests::test_thin_parser_as_expression_followed_by_logical_or` (pass).
- `./wasm/test.sh thin_parser_tests::test_thin_parser_keyword_identifier_in_expression` (pass).
- `./wasm/test.sh thin_parser_tests::test_thin_parser_arrow_param_keyword_identifier` (pass).
- `./wasm/test.sh thin_parser_tests::test_thin_parser_type_predicate_keyword_param` (pass).
- `./wasm/test.sh thin_parser_tests::test_thin_parser_namespace_identifier_assignment_statement` (pass).
- `./wasm/test.sh thin_parser_tests::test_thin_parser_type_identifier_assignment_statement` (pass).
- `./wasm/test.sh thin_parser_tests::test_thin_parser_relational_with_parenthesized_rhs` (pass).
- `./wasm/test.sh thin_parser_tests::test_thin_parser_every_type_arrow_conditional_comma` (pass).
- `./wasm/test.sh thin_parser_tests::test_thin_parser_every_type_arrow_conditional_comma_expression` (pass).
- `./wasm/test.sh thin_parser_tests::test_thin_parser_arrow_optional_chain_with_ternary_comma` (pass).
- `./wasm/test.sh thin_parser_tests::test_thin_parser_checker_every_type_arrow_optional_chain` (pass).
- `./wasm/test.sh thin_parser_tests::test_thin_parser_checker_every_type_arrow_optional_chain_line` (pass).
- `./wasm/test.sh thin_checker_tests::test_thin_checker_resolves_function_parameter_from_bound_state` (pass).
- `./wasm/test.sh thin_binder_tests::test_thin_binder_resolves_parameter_from_bound_state` (pass).
- `./wasm/test.sh thin_binder_tests::test_thin_binder_resolves_parameter_from_bound_state_module_instance_state` (pass).
- `./wasm/test.sh thin_binder_tests::test_thin_binder_resolves_parameter_from_bound_state_module_instance_state_with_visited` (pass).
- `./wasm/test.sh thin_binder_tests::test_thin_binder_resolves_parameter_from_bound_state_binder_ts_331` (pass).
- `./wasm/test.sh thin_binder_tests::test_thin_binder_resolves_parameter_from_bound_state_binder_ts_331_without_scopes` (pass).
- `./wasm/test.sh thin_binder_tests::test_thin_binder_resolves_block_local_from_bound_state_binder_ts_432` (pass).
- `./wasm/bench_cli.sh --repo . --tsconfig src/compiler/tsconfig.json --runs 1 --warmup 1` (rerun after binder.ts:432 fix; tsz still exits with diagnostics, first error `src/compiler/binder.ts:575:33` (TS2693 `Set` only refers to a type, but is being used as a value here), captured via `./wasm/target/release/tsz --project src/compiler/tsconfig.json --noEmit 2>&1 | rg -m1 'TS[0-9]+'`).

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
- [x] Expand tsconfig support
  - [x] baseUrl
  - [x] paths
  - [x] rootDir
  - [x] jsx (preserve/react-native)
  - [x] sourceMap (basic .map output + sourceMappingURL comments)
  - [x] declarationMap (basic .d.ts.map output + sourceMappingURL comments)
  - [x] noEmitOnError
  - [x] lib (resolve `compilerOptions.lib` to src/lib and follow references)
  - [x] types (include @types packages specified in compilerOptions.types)
  - [x] typeRoots (include type packages from compilerOptions.typeRoots)
- [x] Respect `--project` and tsconfig inheritance in CLI.
  - `--project` accepts file or directory
  - `extends` chain already supported
- [x] Module resolution parity
  - [x] Resolve relative + baseUrl/paths imports with TS extension inference.
  - [x] Resolve bare specifiers via node_modules package.json entries + index fallback.
  - [x] Support exports subpath mapping + basic condition selection (types/import/require/default).
  - [x] Expand exports conditions (node/browser) + moduleResolution-specific ordering.
  - [x] Honor package.json `type` + Node16/NodeNext extension rules.
  - [x] Apply `typesVersions` mappings for package subpaths.
  - [x] `imports` condition selection coverage (require vs import).
- [x] typesVersions range selection/fallback + fixed version doc.
- [x] typesVersions compiler version override (flag/env) + fallback tests.
- [x] typesVersions compiler version override via tsconfig + docs.
- [x] Module resolution parity: support package.json `imports` (# specifiers) with conditions.
- [x] Benchmark harness
  - Script: `wasm/bench_cli.sh` (tsz vs tsc timing + memory stats).
- [x] Bench helper: follow symlinks in file discovery (env `TSZ_FOLLOW_SYMLINKS=1`).
- [x] Bench helper: parse optional call chains with type arguments (`obj?.<T>(...)`).
- [x] Bench helper: parse spread call arguments (`foo(...args)`).
- [x] Bench helper: parse `as`/`satisfies` before logical operators (`expr as T || fallback`).
- [x] Bench helper: allow keyword identifiers (`set`, `get`) in expression context.
- [x] Bench helper: allow keyword identifiers as arrow params (`symbol => symbol`).
- [x] Bench helper: allow keyword identifiers in type predicates (`symbol is Symbol`).
- [x] Bench helper: allow `namespace`/`module` identifiers in expression statements.
- [x] Bench helper: allow `type` identifier in expression statements.

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
