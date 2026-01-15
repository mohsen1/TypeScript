# Worker-3 Task List

## 🔴 CURRENT TASK: Global Scope Fix (TS2304 - Error Poisoning)
**Priority:** 🔴 CRITICAL
**Owner:** worker-3
**Branch:** worker-3
**Status:** 🟡 IN PROGRESS
**Assigned:** 2026-01-15

---

## Task Description

**Problem:** TS2304 appears in both Extra (343) and Missing (116) lists. This is the root of "Error Poisoning."

### Why This Matters

**Extra TS2304:** We aren't loading `lib.d.ts` correctly in the test runner, so `console`, `Promise`, `Array`, and other global types are undefined. This causes false "Cannot find name" errors.

**Missing Errors:** When `Promise` is undefined, the Solver treats it as `Any`. This suppresses TS2322 (Type Mismatch) errors downstream, hiding real bugs.

**Impact:** This single issue is poisoning both our error counts AND hiding other type errors from being detected.

---

## Investigation Required

### Phase 1: Understand Current Behavior (DO THIS FIRST)

**Before making any changes:**

1. **Find the test runner setup:**
   ```bash
   cd /tmp/orchestrator-workspace/worktrees/worker-3/wasm
   grep -rn "lib.d.ts" .
   grep -rn "conformance-test" .
   grep -rn "SymbolTable" .
   ```

2. **Understand global symbol loading:**
   - Read `wasm/specs/BINDER.md` for symbol binding architecture
   - Find where `SymbolTable` is created for tests
   - Find where `lib.d.ts` should be loaded
   - Find how global declarations (like `console`, `Promise`) are registered

3. **Run baseline conformance tests:**
   ```bash
   cd /tmp/orchestrator-workspace/worktrees/worker-3/wasm
   ./differential-test/run-conformance.sh --max=100
   ```
   Record current TS2304 counts (both Extra and Missing).

4. **Find examples of broken global resolution:**
   ```bash
   cd /tmp/orchestrator-workspace/worktrees/worker-3/wasm
   ./differential-test/find-extra-ts2304.mjs  # if exists, or create it
   ```

5. **Study how tsc handles globals:**
   - How does TypeScript merge `lib.d.ts` with user code?
   - How do global interfaces (`Window`, `Array`) get merged?
   - What's the difference between `declare var` and `interface` at global scope?

---

## Implementation Plan

### Phase 2: Fix Lib Injection

**Goal:** Ensure `lib.d.ts` is correctly merged into the root `SymbolTable` for every test.

**Key Files to Investigate:**

1. **Test Setup** (likely in `wasm/differential-test/` or `wasm/tests/`)
   - Find the conformance test runner
   - Find where the `SymbolTable` is created
   - Find where TypeScript source files are loaded

2. **Symbol Binding** (`wasm/src/binder/`)
   - `mod.rs` or `symbol_table.rs` - main symbol table implementation
   - Look for global scope handling
   - Look for "ambient" or "declare" handling

3. **Integration Layer** (`wasm/src/integration/`)
   - May handle lib.d.ts loading
   - May handle symbol table initialization

**Pattern to Find and Fix:**

```typescript
// BEFORE (broken - no lib.d.ts):
function run_test(test_file: string) {
    let symbol_table = new SymbolTable();
    load_file(test_file, symbol_table);  // Missing globals!
}

// AFTER (fixed - lib.d.ts loaded):
function run_test(test_file: string) {
    let symbol_table = new SymbolTable();
    load_lib_dts(symbol_table);  // Load console, Promise, Array, etc.
    load_file(test_file, symbol_table);  // Now has access to globals
}
```

### Phase 3: Fix Global Merging

**Goal:** Ensure `interface Window` (and similar globals) merge correctly across files.

**Key Concepts:**

1. **Declaration Merging:** TypeScript allows multiple `interface Window` declarations to merge into one
2. **Global Scope:** All `lib.d.ts` declarations are at global scope
3. **Augmentation:** User code can augment global types (e.g., `interface Window { myCustomProp: string; }`)

**Implementation:**

```rust
// In binder/symbol_table.rs or similar:

// Handle declaration merging for global interfaces
fn merge_global_interface(&mut self, name: &str, new_interface: &Interface) {
    if let Some(existing) = self.global_symbols.get(name) {
        // Merge the new interface members into the existing one
        existing.members.extend(new_interface.members);
    } else {
        // First time seeing this interface - add it
        self.global_symbols.insert(name.to_string(), new_interface);
    }
}
```

### Phase 4: Validate the Fix

**Expected Result:**

1. **TS2304 Extra errors should drop dramatically:**
   - From 343 to <10 (per success metrics)
   - `console`, `Promise`, `Array` should be found
   - False "Cannot find name" errors eliminated

2. **TS2322 Missing errors should increase:**
   - We should see MORE type mismatch errors (good!)
   - These were previously hidden by "undefined = Any" logic
   - This means our type checker is now working correctly

3. **Overall Exact Match should increase:**
   - From current 44.2% toward 80% target
   - More accurate error detection

**Validation Steps:**

1. Run conformance tests after each major change
2. Check TS2304 counts (Extra should drop, Missing should stabilize)
3. Check TS2322 counts (Missing should drop as we fix poisoning)
4. Verify no regressions in previously passing tests
5. Document which errors are "expected" vs "real bugs"

---

## Success Criteria

- [ ] `lib.d.ts` is loaded into root `SymbolTable` for all tests
- [ ] Global types (`console`, `Promise`, `Array`, etc.) resolve correctly
- [ ] TS2304 Extra errors reduced from 343 to <10
- [ ] TS2322 Missing errors decrease (previously hidden by poisoning)
- [ ] Global interface merging works correctly (e.g., `interface Window`)
- [ ] No regressions in tests that were previously passing
- [ ] Conformance test exact match increases significantly
- [ ] Code comments added explaining global symbol loading

---

## Workflow

1. **Sync with latest rust:**
   ```bash
   git fetch origin
   git rebase origin/rust
   ```

2. **Investigation Phase:**
   - Find the test runner and symbol table initialization
   - Understand current lib.d.ts loading (or lack thereof)
   - Run baseline conformance tests
   - Find examples of broken global resolution
   - Document current behavior

3. **Implementation Phase:**
   - Implement lib.d.ts loading in test setup
   - Fix global scope merging
   - Fix interface declaration merging
   - Run tests after each change
   - Document error count changes

4. **Validation:**
   - Run full conformance test suite
   - Verify TS2304 Extra errors dropped to <10
   - Verify TS2322 Missing errors decreased
   - Check for unexpected regressions
   - Document findings

5. **Commit and Push:**
   ```bash
   git add -A
   git commit -m "feat(binder): fix global scope and lib.d.ts loading"
   git push origin worker-3 --force
   ```

6. **STOP** - Wait for EM-1 review

---

## Deliverables

1. lib.d.ts correctly loaded in all tests
2. TS2304 Extra errors reduced from 343 to <10
3. Global interface merging working correctly
4. Baseline test results showing error reductions
5. Documentation of global symbol loading
6. Updated task list with "Complete" status
7. Conformance test report showing the improvement

---

## Known Risks

1. **Test Runner Changes:** May require significant refactoring of test setup
   - **Mitigation:** Start with minimal changes, add lib.d.ts loading first

2. **Declaration Merging Complexity:** Global interface merging can be tricky
   - **Mitigation:** Study TypeScript's behavior carefully, test edge cases

3. **Performance Impact:** Loading lib.d.ts for every test may slow things down
   - **Mitigation:** Cache the parsed lib.d.ts symbols, reuse across tests

4. **Unexpected Regressions:** Fixing poisoning may expose other bugs
   - **Mitigation:** Run tests incrementally, document each change

---

## Previous Tasks: ✅ COMPLETE

### Recursion Guards (Stack Overflow Prevention) ✅
**Status:** ✅ Complete
**Results:** Verified working, zero crashes in all test scenarios

### Invert Solver Defaults (Stop being "Nice") ✅
**Status:** ✅ Complete
**Results:**
- Changed TypeId::ANY defaults to TypeId::UNKNOWN
- TS7006 (Implicit Any): 11 extra errors - catching previously hidden
- TS2322 (Type Mismatch): 4 extra errors - catching previously hidden
- Exact Match: 44.2% (up from ~30% baseline)

### Parser Noise Fix (TS1005 & TS1109) ✅
**Status:** ✅ Complete
**Results:**
- TS1005: 24 extra errors (down from 439) - 95% reduction
- TS1109: 0 extra errors (down from 262) - 100% reduction
- Combined: 24 extra errors (down from 701) - 97% reduction

### Class Property Initialization (TS2564) ✅
**Status:** ✅ Complete
**Implementation:** strictPropertyInitialization check in `wasm/src/checker/declarations.rs`
**Tests:** 4 comprehensive unit tests - all passing

---

## Status

- **Current Task:** Global Scope Fix (TS2304 - Error Poisoning)
- **Phase:** Investigation (Phase 1)
- **Last Updated:** 2026-01-15
- **Ready to Start:** ✅ YES
- **MERGED TO em-team-1:** 2026-01-15 (commit 5441ae2127d)
- **Pushed to Origin:** ✅ YES

---

## Worker-3 Merge Summary

### Completed Tasks Merged:
1. **Invert Solver Defaults** ✅
   - Changed TypeId::ANY defaults to TypeId::UNKNOWN
   - TS7006 (Implicit Any): 11 extra errors - catching previously hidden
   - TS2322 (Type Mismatch): 4 extra errors - catching previously hidden
   - Exact Match: 44.2% (up from ~30% baseline)

2. **Parser Noise Fix (TS1005 & TS1109)** ✅
   - TS1005: 24 extra errors (down from 439) - 95% reduction
   - TS1109: 0 extra errors (down from 262) - 100% reduction
   - Combined: 24 extra errors (down from 701) - 97% reduction

3. **Recursion Guards Investigation** ✅
   - Verified existing recursion guards working correctly
   - Zero crashes in all test scenarios

### Task Assignments Added:
- Global Scope Fix (TS2304 - Error Poisoning) - NOW IN PROGRESS

### Files Modified:
- wasm/src/binder.rs
- wasm/src/checker/expr.rs
- wasm/src/checker/types/diagnostics.rs
- wasm/src/cli/driver.rs
- wasm/src/parallel.rs
- wasm/src/thin_binder.rs
- wasm/src/thin_checker.rs
- wasm/src/thin_parser.rs
- Plus documentation and task list updates
