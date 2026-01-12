# Worker 5 Task List - Binder Squad

## Current Task
- [ ] **BIND-8: Handle ambient module contexts**
  - Fix `declare module "node"` resolution
  - Ensure module-scoped symbols don't leak to global
  - Test import resolution for ambient modules

## Queue
- [ ] **BIND-10: Integrate lib loader with Binder**
  - Call `LibLoader` during Binder initialization
  - Merge lib symbols into root SymbolTable
  - Verify global symbols resolve correctly
- [ ] **BIND-11: Test cross-file symbol resolution**
  - Create multi-file test cases for module augmentation
  - Verify `interface Window` merging works across files
  - Test `declare global` in module contexts

## Completed
- [x] **BIND-2: Implement lib.d.ts parsing and loading**
  - Created `wasm/src/binder/lib_loader.rs`
  - Parse `lib.d.ts` into AST using ThinParserState
  - Extract interface/type/variable declarations
  - Build `LibSymbols` struct for global injection
- [x] **BIND-5: Fix module augmentation resolution**
  - Created `wasm/src/binder/interface_merger.rs`
  - Implemented `merge_interface_declarations` for cross-file interfaces
  - Handle `interface Window` merging across multiple files
  - Support `declare global` augmentation in modules
