# Worker 5 Task List - Binder Squad

## Current Task
- [ ] **BIND-14: Write Binder integration tests**
  - Create test file: `tests/conformance/binder_integration.ts`
  - Test full binder pipeline: lib loading -> binding -> symbol resolution
  - Verify all global symbols resolve correctly
  - Goal: TS2304 errors < 50

## Queue
- [ ] **BIND-16: Fix module namespace symbol access**
  - Ensure `import * as ns` creates proper namespace object
  - Test: `ns.function()` access patterns
  - Handle namespace member lookup correctly

## Completed
- [x] **BIND-13: Fix import/export symbol resolution**
  - Ensured re-exported symbols are properly bound
  - Handled `export { X } from "module"` correctly
  - Fixed default export/import binding
  - Tested `import X from "module"` resolution
- [x] **BIND-10: Integrate lib loader with Binder**
  - Called `LibLoader` during Binder initialization
  - Merged lib symbols into root SymbolTable
  - Verified global symbols resolve correctly
  - Tested `console.log("hello")` no longer produces TS2304
- [x] **BIND-2: Implement lib.d.ts parsing and loading**
  - Created `wasm/src/lib_loader.rs`
  - Parse `lib.d.ts` into AST using ThinParserState
  - Extract interface/type/variable declarations
  - Build `LibSymbols` struct for global injection
- [x] **BIND-5: Fix module augmentation resolution**
  - Created `wasm/src/binder/interface_merger.rs`
  - Implemented `merge_interface_declarations` for cross-file interfaces
  - Handle `interface Window` merging across multiple files
  - Support `declare global` augmentation in modules
- [x] **BIND-8: Handle ambient module contexts**
  - Created `wasm/src/binder/ambient.rs`
  - Fixed `declare module "node"` resolution
  - Ensured module-scoped symbols don't leak to global
  - Added tests for ambient module import resolution
