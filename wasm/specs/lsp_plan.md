# LSP Track Plan (Language Service)

## Mission
Fast, correct LSP features for TS/TSX with minimal allocations and stable incremental behavior.

## Scope
Files: `wasm/src/lsp/*`, `wasm/src/thin_binder.rs`, `wasm/src/checker/*`.

## Current Status
- Major features implemented (definitions, references, rename, organize imports, extract variable, signature help).
- Incremental updates reuse arena/binder; type/scope caches are per-file and reset on edit.
- JSDoc extraction is complete for hover/signature help, including overload matching.

## Highest-Impact Next Tasks
- [x] JSDoc coverage expansion
  - [x] Add overload JSDoc tests for methods/constructors, including `this` and rest params.
  - [x] Add JSDoc proximity tests to guard against unrelated comment chaining.
  - [x] Attach hover JSDoc to single-decl variable statements (use statement start).
- [x] Error-recovery robustness
  - [x] Ensure hover/signature help tolerate missing nodes in incomplete calls/expressions.
  - [x] Add EOF fallback tests for hover and signature help.
  - [x] Recover hover/signature help after incomplete member access/call chains.
  - [x] Advance active parameter on trailing commas without counting comment commas.
- [x] Incremental file updates
  - [x] Add `Project::update_file` that applies LSP edits and re-parses.
  - [x] Reuse parsed arena + binder where possible.
  - [x] Cache `TypeCache` per file for hover/signature help.
  - [x] Extend caches for type-aware completions.
  - [x] Extend caches for diagnostics.
- [x] Incremental binder reuse
  - [x] Preserve prefix symbols/scopes when rebinding suffix statements.
  - [x] Gate incremental binding on `IncrementalParseResult.reparse_start`.
- [x] JSDoc extraction for signature help/hover
  - [x] Parse JSDoc blocks and attach to `SignatureInformation` and `Hover`.
- [x] Extract variable precedence fix
  - [x] Wrap selected expressions to preserve semantics.
  - [x] Add tests around binary/conditional expressions.
- [x] Type-aware completions
  - [x] Add member completions using `ThinCheckerState` + `format_type`.
  - [x] Include auto-import suggestions from project export index.
- [x] Performance instrumentation
  - Measure per-request timing and memoize scope walkers.
  - [x] Reuse scope cache for rename operations.
- [x] Cross-file rename
  - [x] Project-level rename builds workspace edits from multi-file references.
  - [x] Preserve local alias names when renaming exported symbols.
- [x] Diff-based incremental parsing
  - [x] Reparse from the first affected statement and reuse prefix nodes.

## Success Criteria
- Edits <50ms for medium projects.
- Correct docs and signatures for overloaded functions.
- Stable rename/organize imports across files.

Status: Active
