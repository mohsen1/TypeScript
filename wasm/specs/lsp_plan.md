# LSP Track Plan (Language Service)

## Mission
Fast, correct LSP features for TS/TSX with minimal allocations and stable incremental behavior.

## Scope
Files: `wasm/src/lsp/*`, `wasm/src/thin_binder.rs`, `wasm/src/checker/*`.

## Current Status
- Major features implemented (definitions, references, rename, organize imports, extract variable, signature help).
- Remaining TODOs: JSDoc extraction, operator precedence in extract variable.
- No incremental reparse or type cache reuse between edits.

## Highest-Impact Next Tasks
- [ ] Incremental file updates
  - Add `Project::update_file` and reuse parsed arena + binder where possible.
  - Cache `TypeCache` per file for hover/completions/diagnostics.
- [ ] JSDoc extraction for signature help/hover
  - Parse JSDoc blocks and attach to `SignatureInformation` and `Hover`.
- [ ] Extract variable precedence fix
  - Wrap selected expressions to preserve semantics.
  - Add tests around binary/conditional expressions.
- [ ] Type-aware completions
  - Add member completions using `ThinCheckerState` + `format_type`.
  - Include auto-import suggestions from project export index.
- [ ] Performance instrumentation
  - Measure per-request timing and memoize scope walkers.

## Success Criteria
- Edits <50ms for medium projects.
- Correct docs and signatures for overloaded functions.
- Stable rename/organize imports across files.
