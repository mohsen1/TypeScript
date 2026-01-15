# WORKER-10 TASK LIST

## Squad: em-team-3
## EM: EM-3
## Branch: worker-10

---

## Primary Task: Global Scope & Lib Injection (TS2304)

**Priority:** @ CRITICAL (Phase 2 for em-team-3)

### Problem
- 343 extra TS2304 errors ("Cannot find name 'X'")
- `lib.d.ts` symbols are not being injected into the global scope
- Global symbol merging from multiple files is broken
- This poisons downstream type checking with false "undefined" errors

### Context
TS2304 means "Cannot find name 'X'". This happens when:
1. Built-in globals (Array, Object, Promise) are missing from lib.d.ts injection
2. Global symbol merging across files is not working
3. The Binder cannot resolve global declarations

### Action Items

#### 1. Investigate lib.d.ts Injection
- **File:** `wasm/src/binder/thin_binder.rs`
- Check how `lib.d.ts` is loaded and processed
- Verify global symbols are extracted and added to the SymbolGraph
- Compare with how TypeScript handles `lib.d.ts` injection

#### 2. Fix Global Symbol Merging
- **File:** `wasm/src/binder/thin_binder.rs`
- When multiple files declare global symbols (e.g., `declare global { ... }`), they must be merged
- The SymbolGraph needs a "global scope" that persists across files
- Ensure global declarations from all files contribute to the same global ScopeId

#### 3. Verify Symbol Resolution for Global Identifiers
- **File:** `wasm/src/binder/thin_binder.rs`
- When resolving an identifier, check the global scope first
- Built-in types (Array, string, number) should be found without explicit imports
- Test: `const x: Array<number> = []` should NOT produce TS2304 for 'Array'

#### 4. Test with Real-World Scenarios
- Test file with only `const x: string = "hello"` - should resolve 'string' from lib.d.ts
- Test file with `declare global { interface Foo {} }` - should merge Foo into global scope
- Test file using `Promise.all()` - should resolve 'Promise' from lib.d.ts

### Files to Work On
- `wasm/src/binder/thin_binder.rs` (Primary)
- `wasm/src/binder/symbol_graph.rs` (if it exists, or create it)
- `wasm/src/parser/scanner.rs` (if lib loading issues)

### Success Criteria
- Reduce extra TS2304 from 343 to <10
- All lib.d.ts built-in types (Array, Object, Promise, string, number, etc.) resolve correctly
- Global symbol merging works across multiple files
- Conformance tests show significant improvement in TS2304 accuracy

### Testing
- Run: `./wasm/differential-test/run-conformance.sh --all`
- Analyze the report, focus on TS2304 errors
- Compare with tsc output to verify we're not removing valid errors

---

## Instructions

1. **Wait for worker-9 (Parser) to make progress** - Global scope fixes depend on a clean AST
2. Create branch from `em-team-3` (or `rust` if em-team-3 doesn't exist yet)
3. Focus ONLY on TS2304 (global scope). Do not work on other issues.
4. Run conformance tests frequently to track progress
5. Push to `worker-10` branch when ready for review
6. Mark "Ready for Merge: Yes" in your plan when done
7. EM-3 will merge and validate before escalating

---

## Task Completion Report

### Before (Baseline)
- TS2304 Extra: 343
- TS2304 Missing: 116

### After (Your Results)
- TS2304 Extra: ___
- TS2304 Missing: ___

### Notes
- (Document what you fixed, what challenges you encountered)
