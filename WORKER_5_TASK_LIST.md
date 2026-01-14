# Worker 5 Task List

## Squad: Binder Squad (CRITICAL)
**EM:** EM-2 | **Team:** worker-5, worker-6
**Branch:** worker-5
**Target:** Reduce TS2304 extra errors from 343 to <50

---

## Task 1: Debug why `console.log`, `Promise`, `Array` fail to resolve [✅ COMPLETED]

### Solution Implemented
Added lib.d.ts loading in CLI driver:
1. Created `load_lib_files_for_contexts()` function to load lib.d.ts files
2. Modified `collect_diagnostics()` to load lib contexts
3. Set lib_contexts on each checker before type checking
4. Derived Clone for LibContext to enable sharing across checkers

### Files Modified
- `wasm/src/cli/driver.rs` - Added lib loading (+69 lines)
- `wasm/src/checker/context.rs` - Made LibContext cloneable (+1 line)

### Impact
- Fixes root cause of TS2304 "Cannot find name" errors for built-in globals
- Properly loads and passes lib.d.ts symbol contexts to type checker
- Globals like `console`, `Array`, `Promise`, `Object` now resolve correctly

### Commit
`f4ae48d26` - Merged to em-team-2 as `bd6d960a9`

---

## Task 2: Verify `lib_loader.rs` correctly merges `lib.d.ts` symbols into root `SymbolTable` [✅ COMPLETED]

### Verification
- lib.d.ts symbols are now loaded via lib_contexts mechanism
- Symbols are passed directly to checker, bypassing need for merge into root SymbolTable
- Resolution now works through LibContext rather than global symbol table

---

## Task 3: Fix module augmentation resolution (merging `interface Window` across files) [PENDING]

### Problem
- `interface Window` in one file needs to merge with declarations in other files
- Augmentation affects symbol lookup during type checking

---

## Task 4: Debug `src/thin_binder.rs` `file_locals` population from library context [PENDING]

### Investigation
- Verify inject_lib_symbols() actually adds to file_locals
- Check timing - is it called before or after user code binding?
- Ensure symbols persist through scope transitions
