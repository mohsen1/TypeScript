# Worker 6 Task List

## Squad: Binder - Scope Resolution & Module Binding

## Current Task
- [ ] Investigate file_locals population in `src/thin_binder.rs`
- [ ] Debug why imported symbols and module-augmented interfaces fail to resolve

## Queue
- [ ] Fix module augmentation resolution (merging `interface Window` across files)
- [ ] Ensure `lib.d.ts` symbols are correctly merged into the root `SymbolTable`
- [ ] Test lib.dom.d.ts loading and symbol merging
- [ ] Coordinate with Worker 5 on binding fixes

## Completed
- None

## Context
TS2304 has both missing (116) AND extra (343) errors. The extra errors indicate the binder is rejecting valid symbols, often due to:
1. Lib symbols not being merged into global scope
2. Module augmentation not working across files
3. File-level scope not inheriting from library context

### Key Files
- `src/thin_binder.rs` - binding logic, file_locals
- `src/lib_loader.rs` - lib.d.ts loading
- `src/symbol.rs` - SymbolTable implementation

### Goal
Reduce TS2304 extra errors from 343 to <50 by fixing scope resolution and module binding.
