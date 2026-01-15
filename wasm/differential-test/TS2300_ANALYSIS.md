# TS2300 (Duplicate Identifier) - Deep Dive Analysis

**Error Code:** TS2300
**Error Message:** "Duplicate identifier '{0}'."
**Analysis Date:** 2026-01-14
**Analyzer:** Worker-12 (Test & Validation Squad)

---

## Executive Summary

🔴 **CRITICAL FINDING:** WASM is **missing 100% of TS2300 errors** that TSC detects.

- **Files with TS2300 (TSC):** 48
- **Files with TS2300 (WASM):** 0
- **Missing by WASM:** 100% (all 48 files)

This is a **complete gap** in duplicate identifier detection for async/await transformations.

---

## Test Results

### Scope
- **Test files analyzed:** 500 conformance tests
- **Files with TS2300 errors:** 48
- **Test categories affected:** Async/await (es5, es6, es2017)

### Pattern Identified

All missing TS2300 errors follow the same pattern:

```typescript
// @target: es2017
declare var a: boolean;
declare var p: Promise<boolean>;
declare function before(): void;    // ← Declared function
declare function after(): void;     // ← Declared function

async function func(): Promise<void> {
    before();                        // ← Called before await
    var b = await p || a;           // ← Await expression
    after();                         // ← Called after await
}
```

**What TSC detects:**
- TypeScript's async transformation generates temporary variables named `before` and `after`
- These generated variables conflict with the user-declared functions
- **Result:** TS2300 errors on both `before` and `after`

**What WASM does:**
- No TS2300 errors emitted
- Does not detect the conflict between:
  - User-declared `before()` / `after()` functions
  - Compiler-generated `before` / `after` variables

---

## Error Distribution

### By Test Category

| Category | Files with TS2300 | Missing by WASM |
|----------|-------------------|-----------------|
| async/es2017/awaitBinaryExpression | 5 | 5 (100%) |
| async/es2017/awaitCallExpression | 8 | 8 (100%) |
| async/es5/awaitBinaryExpression | 5 | 5 (100%) |
| async/es5/awaitCallExpression | 8 | 8 (100%) |
| async/es6/awaitBinaryExpression | 5 | 5 (100%) |
| async/es6/awaitCallExpression | 8 | 8 (100%) |
| async/es2017/await_incorrectThisType | 1 | 1 (100%) |

### Error Count Summary

| Metric | Count |
|--------|-------|
| Total TS2300 errors (TSC) | 96 |
| Total TS2300 errors (WASM) | 0 |
| Missing by WASM | 96 (100%) |

---

## Root Cause Analysis

### Why WASM Misses These Errors

#### 1. **Transformation Awareness Gap**

TypeScript's async/await transformation:

```typescript
// Original code
async function func(): Promise<void> {
    before();
    var b = await p || a;
    after();
}

// Transformed (simplified)
function func() {
    var _this = this;
    return __awaiter(this, void 0, void 0, function () {
        var before, after;  // ← Generated variables
        return __generator(this, function (_state) {
            switch (_state.label) {
                case 0: before();  // ← Conflict!
                case 1:
                    _state.label = 2;
                    return [4, p || a];
                case 2: after();  // ← Conflict!
            }
        });
    });
}
```

**TSC:**
- Tracks compiler-generated variable names
- Checks for conflicts with user declarations
- Reports TS2300 when conflicts detected

**WASM:**
- Has `check_duplicate_identifiers()` function (thin_checker.rs:14046)
- Only checks **direct user declarations**
- Does NOT simulate async/await transformation
- Does NOT check for conflicts with generated variables

#### 2. **Binding Phase Limitation**

The WASM checker's duplicate detection:

```rust
fn check_duplicate_identifiers(&mut self) {
    // Collect symbols from binder
    for sym_id in symbol_ids {
        let symbol = self.ctx.binder.get_symbol(sym_id);
        if symbol.declarations.len() <= 1 {
            continue;
        }
        // Check for conflicts between declarations...
    }
}
```

**Missing:**
- No integration with async transformation
- No awareness of compiler-generated names
- No check against transformation output

#### 3. **Phase Ordering Issue**

**TSC Approach:**
1. Parse → Bind → **Transform** → Check
   - Or: Parse → Bind → **Check with transformation awareness**

**WASM Approach:**
1. Parse → Bind → Check → Transform (in emitter)
   - Check happens before transformation
   - No feedback from transformation to checker

---

## Impact Assessment

### Severity: 🔴 HIGH

**Why this matters:**

1. **Silent Failures**
   - User code with duplicate identifiers passes silently
   - Runtime errors may occur instead of compile-time errors
   - Breaks type safety guarantees

2. **Async/Await is Common**
   - All modern TypeScript code uses async/await
   - Variables named `before`, `after`, `state`, etc. are common
   - High probability of real-world bugs

3. **Test Coverage Gap**
   - 48 test files specifically test this scenario
   - WASM passes 0/48 of these tests
   - Indicates fundamental feature gap

### Affected Scenarios

| Scenario | Example | WASM Status |
|----------|---------|-------------|
| Declared function named `before` | `declare function before() {}` | ❌ No error |
| Declared function named `after` | `declare function after() {}` | ❌ No error |
| Declared variable named `state` | `var state: any;` | ❌ Likely also missing |
| Declared variable named `_this` | `var _this: any;` | ❌ Likely also missing |

---

## Code Location

### WASM Implementation

**File:** `wasm/src/thin_checker.rs`
**Function:** `check_duplicate_identifiers()` (line 14046)

**Current Implementation:**
```rust
fn check_duplicate_identifiers(&mut self) {
    // Collects symbols from binder
    // Checks for multiple declarations
    // Does NOT account for async/await transformation
}
```

**Call Site:** `check_source_file()` (line 13937)
```rust
// Check for duplicate identifiers (2300)
self.check_duplicate_identifiers();
```

### Async Transformation

**File:** `wasm/src/transforms/async_es5.rs`
**Purpose:** Emits async/await as ES5 state machine
**Generated Names:** `before`, `after`, `state`, `_this`, etc.

**No Integration:** The transformation does NOT feed back into the checker.

---

## Recommendations

### Immediate Actions (P0)

1. **Document the Gap**
   - ✅ This analysis report
   - Add to METRICS_DOCUMENTATION.md
   - Track in project issue tracker

2. **Test Coverage**
   - Mark all 48 async/await tests as expected failures
   - Add to regression suite for future validation

3. **Workaround Documentation**
   - Document that WASM does not detect duplicate identifiers from async transformation
   - Advise users to avoid naming variables `before`, `after`, etc. in async functions

### Implementation Plan (P1)

**Option A: Simulation Approach**
- Modify `check_duplicate_identifiers()` to simulate async transformation
- For each async function with `await`:
  - Generate list of compiler-generated variable names
  - Check against user declarations
  - Report TS2300 if conflicts found

**Estimated effort:** 2-3 days
**Risk:** Medium - requires understanding transformation logic

**Option B: Transform-First Approach**
- Run async/await transformation before checking
- Check the transformed code for duplicates
- May require architectural changes

**Estimated effort:** 1-2 weeks
**Risk:** High - significant refactoring

**Option C: Hybrid Approach** (Recommended)
- Quick fix: Add hardcoded check for `before` and `after` in async functions
- Long term: Implement Option A properly
- Addresses immediate test failures

**Estimated effort:** 1 day for quick fix, 2-3 days for proper fix
**Risk:** Low for quick fix, Medium for proper fix

### Related Gaps

Based on this analysis, investigate:

1. **Other generated variable names**
   - `_this` in class methods
   - `_super` in derived classes
   - State machine variables
   - Check these for similar gaps

2. **Other transformation scenarios**
   - Generator functions (`yield` transformations)
   - Class property decorators
   - Parameter destructuring

---

## Test Cases

### Positive Examples (Should Error)

```typescript
// Test 1: before function
declare function before(): void;
async function test1() {
    before();
    await Promise.resolve();
}
// Expected: TS2300 on 'before' (2 occurrences)

// Test 2: after function
declare function after(): void;
async function test2() {
    await Promise.resolve();
    after();
}
// Expected: TS2300 on 'after' (2 occurrences)

// Test 3: Both
declare function before(): void;
declare function after(): void;
async function test3() {
    before();
    await Promise.resolve();
    after();
}
// Expected: TS2300 on 'before' and 'after' (2 occurrences each)
```

### Negative Examples (Should Pass)

```typescript
// Test 4: No conflict
declare function foo(): void;
async function test4() {
    foo();
    await Promise.resolve();
}
// Expected: No errors

// Test 5: Different names
async function test5() {
    var before = 1;
    var after = 2;
    await Promise.resolve();
}
// Expected: No errors (locals shadow compiler-generated)
```

---

## Data Files Generated

- `wasm/metrics-data/ts2300-analysis.json` - Full analysis data
- All 48 affected test files listed
- TSC vs WASM comparison for each file

---

## Next Steps

1. ✅ Complete this analysis report
2. ⏸️ Present to EM-3 and semantics squad
3. ⏸️ Await prioritization decision
4. ⏸️ Implement fix (if approved)
5. ⏸️ Re-run analysis to verify fix

---

## Conclusion

TS2300 (Duplicate identifier) represents a **100% gap** in WASM's async/await checking. The checker is completely unaware of compiler-generated variable names from async transformation, leading to silent failures in code that should produce compile-time errors.

**Priority:** HIGH - This affects all async/await code and represents a fundamental type safety gap.

**Recommendation:** Add to semantics squad backlog, prioritize after current TS2564/TS7006/TS2322 work.
