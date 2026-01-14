# Worker 5 Task List

## Squad: Binder Squad (CRITICAL)
**EM:** EM-2 | **Team:** worker-5, worker-6
**Branch:** worker-5
**Target:** Reduce TS2304 extra errors from 343 to <50

---

## Task 1: Debug why `console.log`, `Promise`, `Array` fail to resolve [IN_PROGRESS]

### Investigation Steps
1. Add debug logging to trace symbol resolution flow
2. Verify lib.d.ts is loaded correctly
3. Check if lib contexts are passed to binder/checker
4. Identify where resolution fails

### Files to Investigate
- `src/lib_loader.rs` - Library loading and symbol merging
- `src/thin_binder.rs` - Symbol table and file_locals
- `src/thin_checker.rs` - Name resolution (resolve_identifier_symbol)
- `src/lib.rs` - Integration point

### Debug Approach
- Set BIND_DEBUG=1 to enable existing debug logs
- Add targeted logging in merge_lib_symbols()
- Verify file_locals population after lib injection
- Test with simple file: `console.log("test");`

---

## Task 2: Verify `lib_loader.rs` correctly merges `lib.d.ts` symbols into root `SymbolTable` [PENDING]

### Acceptance Criteria
- All lib.d.ts globals appear in file_locals after binding
- merge_lib_symbols() copies all symbols correctly
- No symbols are dropped due to scope issues

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
