# Worker 5 Task List - Binder Squad

## Current Task
- [ ] **BIND-2: Implement lib.d.ts parsing and loading**
  - Create `src/binder/lib_loader.rs`
  - Parse `lib.d.ts` into AST
  - Extract interface/type/variable declarations
  - Build `LibSymbols` struct for global injection

## Queue
- [ ] **BIND-5: Fix module augmentation resolution**
  - Implement `merge_interface_declarations` for cross-file interfaces
  - Handle `interface Window` merging across multiple files
  - Support `declare global` augmentation in modules
  - Test: File1: `interface Window { custom: string }` File2: `window.custom` should work
- [ ] **BIND-8: Handle ambient module contexts**
  - Fix `declare module "node"` resolution
  - Ensure module-scoped symbols don't leak to global
  - Test import resolution for ambient modules

## Completed
(none yet)
