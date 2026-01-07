# CLI Track: The Native Interface

## Mission
Implement a high-performance, `tsc`-compatible command-line interface. This drives the core Rust compiler logic (Parser -> Binder -> Solver -> Emitter) in a native environment, handling file I/O, configuration parsing, and error reporting.

## Scope
**Files:** 
- `wasm/src/bin/stc.rs` (Entry point)
- `wasm/src/cli/` (New module)
  - `args.rs` (Clap definitions)
  - `config.rs` (tsconfig.json parsing)
  - `driver.rs` (Pipeline orchestration)
  - `fs.rs` (File system abstractions)
  - `reporter.rs` (Diagnostic formatting)

**Independence:** **High**. Consumes the `wasm` library (Parser/Solver/Emitter) but does not modify core logic.

## Current Status
**Starting** - Core compiler features (Parallel Parsing, Binding, Emitter) are ready to be wired up to a CLI driver.

## Tasks

### Phase 1: Foundation & Arguments
- [x] **Scaffold Binary**
  - Add `[[bin]]` entry in `Cargo.toml` for `stc` (Speedy TypeScript Compiler).
  - Add dependencies: `clap` (derive), `anyhow`, `serde`, `serde_json` (with preserve_order).
- [x] **Implement Argument Parsing**
  - Replicate common `tsc` flags: `--target`, `--module`, `--outDir`, `--strict`, `--noEmit`.
  - Implement `--help` and `--version`.

### Phase 2: Project Configuration (tsconfig)
- [x] **JSONC Parsing**
  - [x] Implement `tsconfig.json` parser that handles comments (JSONC).
  - [x] Support `extends` inheritance (recursive loading).
- [ ] **Option Mapping**
  - Map `tsconfig` "compilerOptions" to internal `PrinterOptions` (Emitter) and `CheckerOptions`.
  - Handle `include`, `exclude`, and `files` globs.

### Phase 3: The Driver (Orchestration)
- [ ] **File Discovery**
  - Implement efficient globbing to find all `.ts` files based on config.
- [ ] **Pipeline Connection**
  - Wire up `parallel::compile_files` (Parser/Binder) to the discovered files.
  - Wire up `thin_checker::check_source_file` for type checking.
  - Wire up `thin_emitter::emit` for output generation.
- [ ] **Output Writer**
  - Implement parallel file writing for emitted `.js` and `.d.ts` files to `outDir`.
  - Ensure directory structures are created.

### Phase 4: Diagnostics & Reporting
- [ ] **Diagnostic Formatter**
  - Implement a reporter that looks like `tsc` (file.ts:line:col - error TS1234: Message).
  - Add color support (`colored` crate).
  - Integrate `solver::diagnostics` output into the CLI reporter.
- [ ] **Exit Codes**
  - Return proper exit codes (0 for success, 1 for errors) based on diagnostic severity.

### Phase 5: Watch Mode (The Speed Demon)
- [ ] **File Watching**
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
  - Create a script to run `stc` vs `tsc` on large open source repos (e.g., Three.js, React).

## Success Criteria
- [ ] **Compatibility:** Can compile a standard `tsconfig.json` project correctly.
- [ ] **Performance:** Significantly faster than `tsc` (Goal: 10x startup/emit speed).
- [ ] **UX:** Error messages are clear, colorful, and point to the correct source location.
- [ ] **Extensibility:** Infrastructure allows adding custom Rust transforms/lints later.
