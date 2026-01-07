# CLI Track: The Native Interface

<<<<<<< HEAD
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
- [ ] **Scaffold Binary**
  - Add `[[bin]]` entry in `Cargo.toml` for `stc` (Speedy TypeScript Compiler).
  - Add dependencies: `clap` (derive), `anyhow`, `serde`, `serde_json` (with preserve_order).
- [ ] **Implement Argument Parsing**
  - Replicate common `tsc` flags: `--target`, `--module`, `--outDir`, `--strict`, `--noEmit`.
  - Implement `--help` and `--version`.

### Phase 2: Project Configuration (tsconfig)
- [ ] **JSONC Parsing**
  - Implement `tsconfig.json` parser that handles comments (JSONC).
  - Support `extends` inheritance (recursive loading).
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
=======
### Phase 5: Structural Constraints
- [x] Constrain Application args when bases match
- [x] Constrain object properties and index signatures
- [x] Constrain tuple elements
- [x] Constrain union members to target
- [x] Constrain optional union targets
- [x] Infer type params from union targets with a single placeholder member
- [x] Add inference coverage for union targets with placeholder and nullish members
- [x] Use Atom for discriminant property names in narrowing
- [x] Compare property access names via Atom to avoid resolve_atom churn
- [x] Store subtype failure property names as Atom for diagnostics
- [x] Avoid allocation when constraining non-nullish union targets
- [x] Respect type parameter shadowing in instantiation scopes
- [x] Use FxHashMap for substitution and inference maps
- [x] Infer generics for callable overload signatures
- [x] Validate rest parameter argument types in call resolution
- [x] Treat rest parameters as optional for min argument count
- [x] Validate generic call argument count and non-generic params
- [x] Validate generic calls after inference for defaulted params
- [x] Use FxHashSet in inference visited/dedup
- [x] Handle tuple rest parameters in call resolution
- [x] Constrain tuple rest elements during inference
- [x] Add tuple rest inference coverage for rest params and rest arguments
- [x] Infer variadic tuple type params from rest arguments
- [x] Recheck argument counts after inference for rest tuple defaults/constraints
- [x] Add defaulted rest tuple call coverage for count and optional behavior
- [x] Infer rest tuple type params inside tuple parameter types
- [x] Infer tuple rest placeholders from rest tuple arguments
- [x] Add tuple rest expansion subtyping coverage
- [x] Handle tuple rest expansion for tuple-to-array subtyping
- [x] Validate tuple-to-array bounds during inference resolution
- [x] Allow union upper bounds during inference resolution
- [x] Validate object and function bounds during inference resolution
- [x] Validate generic application bounds during inference resolution
- [x] Validate callable bounds during inference resolution
- [x] Allow object keyword bounds during inference resolution
- [x] Validate index signature bounds during inference resolution
- [x] Add negative index signature bounds coverage
- [x] Refine number index property checks during bounds validation
- [x] Match numeric literal name checks to TypeScript canonical `Number.toString` behavior
- [x] Add numeric literal name bounds coverage for exponent and non-canonical forms
- [x] Honor readonly properties and index signatures during bounds validation
- [x] Allow bivariant method parameter checks for bounds validation
- [x] Add readonly and method variance bounds coverage
- [x] Use assignability checks for bounds validation in inference resolution
- [x] Add assignability-based bounds coverage for bivariant function params
- [x] Infer index signature type params from object literal properties
- [x] Infer property type params from source index signatures
- [x] Allow index signatures to satisfy named properties in subtype checks
- [x] Use canonical numeric literal checks for index signature properties in subtyping
- [x] Add subtype tests for index signatures satisfying named properties
- [x] Cover non-canonical numeric names in index signature subtyping tests
- [x] Add inference coverage for non-canonical numeric properties in number index inference
- [x] Infer mixed string/number index signatures from object literal properties
- [x] Cover numeric literal special cases in index signature inference
- [x] Add NaN numeric literal inference coverage for number index signatures
- [x] Add -Infinity numeric literal inference coverage for number index signatures
- [x] Add negative zero numeric literal inference coverage for number index signatures
- [x] Add exponent-form numeric literal inference coverage for number index signatures
- [x] Add optional object property inference coverage
- [x] Add optional property inference coverage for explicit undefined values
- [x] Include optional properties in index/property access evaluation
- [x] Add optional property access coverage for indexed objects
- [x] Add optional property inference coverage for index signature inference
- [x] Add optional property inference coverage for number index signature inference
- [x] Add optional property inference coverage for mixed index signatures
- [x] Add optional non-canonical numeric property coverage for mixed index signatures
- [x] Add optional argument mismatch coverage for required properties
- [x] Add missing required property coverage for generic inference
- [x] Add readonly property mismatch coverage for generic inference
- [x] Add readonly property mismatch coverage for indexed objects
- [x] Add readonly index signature mismatch coverage for generic inference
- [x] Add readonly number index signature mismatch coverage for generic inference
- [x] Add method property bivariant parameter coverage for generic inference
- [x] Add function property contravariant parameter coverage for generic inference
- [x] Add method property bivariant optional parameter coverage for generic inference
- [x] Add missing property inference via index signature coverage
- [x] Add missing numeric property inference via number index signature coverage
- [x] Infer readonly wrapper type params during constraint collection
- [x] Infer type params from callable parameter signatures
- [x] Infer type params from callable arguments with single signatures
- [x] Infer type params from overloaded callable arguments by selecting compatible signatures
- [x] Infer type params from callable parameters with callable arguments
- [x] Infer type params from function/callable this-type positions
- [x] Infer type params from constructor call signatures
- [x] Infer type params from KeyOf wrapper positions
- [x] Infer type params from IndexAccess wrapper positions
- [x] Infer type params from index access on concrete object properties
- [x] Infer type params from template literal wrapper positions
- [x] Infer type params from conditional types with concrete check types
- [x] Infer type params from mapped types with concrete key sets
- [x] Infer array element type params from tuple arguments
- [x] Enforce index signature consistency during bounds validation

## Architecture Notes
- Use `ena` crate for union-find data structure
- All types are TypeId (u32 index), no String allocation
- Inference is demand-driven: only instantiate when needed
- Cache results in TypeInterner to deduplicate

## Success Criteria
- `cargo test solver::infer` passes all tests
- Can infer type arguments for generic functions
- Can resolve conditional types (future, but foundation ready)
- Zero TypeScript-specific business logic in this module
>>>>>>> solver-infer-track
