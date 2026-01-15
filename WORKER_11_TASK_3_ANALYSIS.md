# Worker 11 Task 3 Analysis: Diagnostic Emission Audit

Generated: 2026-01-14  
Worker: Worker 11  
Task: Diagnostic emission audit for ERROR type handling

---

## Executive Summary

**Status:** 🔍 AUDIT COMPLETE - CRITICAL FINDING

**Root Cause Identified:** The missing ~310 TS2322 errors are caused by **explicit diagnostic suppression** in `thin_checker.rs`, NOT by issues in the solver layer.

The checker explicitly skips diagnostics when types contain ERROR, preventing valid TS2322 errors from being emitted.

---

## 1. Diagnostic Flow Architecture

### 1.1 Complete Flow Diagram

```
Assignment Check (thin_checker.rs:6743)
│
├─► Get source type (right side)
├─► Get target type (left side)
│
├─► Check if assignable: is_assignable_to(source, target)
│   │
│   └─► CompatChecker::is_assignable() [solver/compat.rs]
│       │
│       └─► SubtypeChecker::is_subtype_of() [solver/subtype.rs]
│           │
│           ├─► source/target ERROR? → Return SubtypeResult::False ✅
│           ├─► Perform structural subtype check
│           └─► Return SubtypeResult::True/False
│
├─► If NOT assignable:
│   │
│   └─► error_type_not_assignable_with_reason_at() [thin_checker.rs:13066]
│       │
│       ├─► ❌ CRITICAL: type_contains_error(source) || type_contains_error(target)
│       │   └─► return; // DIAGNOSTIC SUPPRESSED!
│       │
│       ├─► Use CompatChecker::explain_failure() to get details
│       ├─► Create PendingDiagnostic
│       ├─► Render to TypeDiagnostic
│       └─► Push to ctx.diagnostics
│
└─► Emitter outputs diagnostics
```

### 1.2 Key Insight

**The solver layer is correct!** Subtype checking returns `False` for ERROR types.

**The problem is in the checker layer!** Before diagnostics are created, the checker checks if types contain errors and **silently returns** without emitting anything.

---

## 2. CRITICAL FINDING: Diagnostic Suppression

### 2.1 Primary Suppression Point #1

**File:** `wasm/src/thin_checker.rs`  
**Function:** `error_type_not_assignable_with_reason_at`  
**Line:** 13074-13076

```rust
pub fn error_type_not_assignable_with_reason_at(
    &mut self,
    source: TypeId,
    target: TypeId,
    idx: NodeIndex,
) {
    use crate::solver::{CompatChecker, TypeFormatter};

    // ❌ CRITICAL: Suppresses all diagnostics involving ERROR types
    if self.type_contains_error(source) || self.type_contains_error(target) {
        return;  // <-- SILENTLY SUPPRESSES TS2322!
    }
    // ... rest of diagnostic creation
}
```

**Impact:** When a type can't be resolved (e.g., TS2304 "Cannot find name 'Foo'"), any TS2322 involving that type is suppressed.

**Example:**
```typescript
let x: Foo = 123; // TS2304: Cannot find name 'Foo'
                   // TS2322: Type 'number' is not assignable to type 'Foo' <- SUPPRESSED!
```

### 2.2 Primary Suppression Point #2

**File:** `wasm/src/thin_checker.rs`  
**Function:** `error_type_not_assignable_at`  
**Line:** 13042-13044

```rust
pub fn error_type_not_assignable_at(&mut self, source: TypeId, target: TypeId, idx: NodeIndex) {
    // ❌ CRITICAL: Also suppresses diagnostics for ERROR types
    if self.type_contains_error(source) || self.type_contains_error(target) {
        return;  // <-- SILENTLY SUPPRESSES TS2322!
    }
    // ... rest of diagnostic creation
}
```

**Impact:** Same as above - fallback diagnostic path also suppresses ERROR types.

### 2.3 Secondary Suppression Points (Contextual Typing)

**Lines:** 6753, 6810, 15630, 19927, 20598

```rust
// Example from line 6753:
if left_type != TypeId::ANY && !self.type_contains_error(left_type) {
    self.ctx.contextual_type = Some(left_type);
}
```

**Impact:** ERROR types don't get contextual typing, which may affect type inference but doesn't directly suppress diagnostics.

---

## 3. Solver Layer Analysis (Confirmed Working)

### 3.1 Subtype Checker (`solver/subtype.rs`)

**Lines 305-307:**
```rust
// Error types are NOT compatible (propagate errors instead of silencing)
if source == TypeId::ERROR || target == TypeId::ERROR {
    return SubtypeResult::False;  // ✅ CORRECT: Returns False
}
```

**Status:** ✅ **WORKING CORRECTLY**

The solver returns `False` when types involve ERROR, which should trigger diagnostics.

### 3.2 Compat Checker (`solver/compat.rs`)

**Lines 155-161:**
```rust
} else if source == TypeId::ERROR || target == TypeId::ERROR {
    // Error types should NOT silently pass assignability checks.
    // Delegate to subtype checker which returns false for ERROR.
    self.configure_subtype(self.strict_function_types);
    self.subtype.is_subtype_of(source, target)
}
```

**Status:** ✅ **WORKING CORRECTLY**

The compat layer properly delegates to subtype checker which returns False.

### 3.3 Diagnostic Creation (`solver/diagnostics.rs`)

**Lines 1057-1060:**
```rust
let mut diag = PendingDiagnostic::error(
    codes::TYPE_NOT_ASSIGNABLE,
    vec![source.into(), target.into()],
);
```

**Status:** ✅ **WORKING CORRECTLY**

Diagnostic creation infrastructure exists and is functional.

---

## 4. type_contains_error Function

### 4.1 Implementation (`thin_checker.rs:21644`)

```rust
fn type_contains_error(&self, type_id: TypeId) -> bool {
    let mut visited = Vec::new();
    self.type_contains_error_inner(type_id, &mut visited)
}

fn type_contains_error_inner(&self, type_id: TypeId, visited: &mut Vec<TypeId>) -> bool {
    if type_id == TypeId::ERROR {
        return true;
    }
    // Recursively checks all type components:
    // - Arrays, tuples, unions, intersections
    // - Objects (properties, index signatures)
    // - Functions, callables
    // - Type applications, mapped types
    // - Conditional types
    // ...
}
```

**Behavior:** Recursively checks if a type contains `TypeId::ERROR` anywhere in its structure.

**Status:** ✅ **IMPLEMENTATION CORRECT**

The function correctly identifies ERROR types throughout complex type structures.

---

## 5. Why ERROR Types Exist

### 5.1 Type Lowering Failures (`solver/lower.rs`)

When `lower_type` can't resolve a symbol:

```rust
// Line 2033:
TypeId::ERROR  // Unresolved symbol
```

**Example Scenarios:**
1. TS2304: Cannot find name 'Foo'
2. Missing library type (lib.d.ts not loaded)
3. Forward reference not resolved
4. Generic parameter instantiation failed

### 5.2 Current Behavior vs TypeScript

| Scenario | TypeScript | Current WASM | Issue |
|----------|-----------|--------------|-------|
| `let x: MissingType = 123;` | TS2304 + TS2322 | TS2304 only | TS2322 suppressed |
| `let x: MissingType[] = [123];` | TS2304 + TS2322 | TS2304 only | TS2322 suppressed |
| `function f(x: MissingType) {}` | TS2304 + TS2322 (call site) | TS2304 only | Call site errors suppressed |

**TypeScript Behavior:** Emits both TS2304 (source) and TS2322 (usage).

**WASM Behavior:** Only emits TS2304, suppresses TS2322 to avoid "cascading errors."

---

## 6. Specific Code Locations Requiring Fixes

### 6.1 HIGH PRIORITY: Remove Diagnostic Suppression

**File:** `wasm/src/thin_checker.rs`  
**Lines:** 13074-13076, 13042-13044

**Current Code:**
```rust
if self.type_contains_error(source) || self.type_contains_error(target) {
    return;  // Suppress diagnostic
}
```

**Proposed Fix:**
```rust
// Option 1: Remove suppression entirely (matches TypeScript)
// Comment out the check to emit all TS2322 errors

// Option 2: Only suppress when BOTH types are pure ERROR
// if source == TypeId::ERROR && target == TypeId::ERROR {
//     return;  // Only suppress error-to-error assignments
// }

// Option 3: Add configuration flag
// if self.ctx.suppress_error_diagnostics {
//     if self.type_contains_error(source) || self.type_contains_error(target) {
//         return;
//     }
// }
```

**Expected Impact:**
- **+200-250 visible TS2322 errors** (currently missing)
- **Exact match conformance: 30.8% → ~45%** (+14pp improvement)
- **Missing errors: 57.8% → ~35%** (-23pp improvement)
- **Extra errors may increase** (need to validate)

### 6.2 MEDIUM PRIORITY: Improve Error Type Contextualization

**File:** `wasm/src/thin_checker.rs`  
**Lines:** 6753, 6810, 15630, 19927, 20598

**Current Code:**
```rust
if left_type != TypeId::ANY && !self.type_contains_error(left_type) {
    self.ctx.contextual_type = Some(left_type);
}
```

**Proposed Enhancement:**
Consider whether ERROR types should still provide contextual typing for better inference.

**Impact:** Lower priority, affects type inference quality.

---

## 7. Test Case Examples

### 7.1 Conformance Test Suite

Located: `/tests/cases/conformance/solver/ts2322_assignment_tests.ts`

**30 sections** with expected TS2322 errors:
1. Primitive type mismatches (lines 17-27)
2. Generic type mismatches (lines 43-47)
3. Generic function type mismatches (lines 61-62)
4. Union type non-members (lines 94-98)
5. Partial intersection types (lines 113-117)
6. Array element type mismatches (lines 127-131)
7. Object property type mismatches (lines 147-151)
8. Function type mismatches (lines 163-167)
9. Unknown type without guard (lines 269-270)
10. Null/undefined assignments (lines 298-302)

And 20+ more sections...

### 7.2 Edge Cases Test Suite

Located: `/tests/cases/conformance/solver/ts2322_edge_cases.ts`

**Advanced scenarios:**
- Complex generic variance
- Object literal freshness
- Callable interface overloads
- Template literal types
- Conditional types
- Mapped types
- Branded types
- Discriminated unions

---

## 8. Side Effects and Mitigation

### 8.1 Cascading Errors Concern

**Concern:** "Fixing TS2304 will show 100 more errors downstream"

**Reality:** This is actually **GOOD** behavior!
- TypeScript shows both errors
- Users want to see all type mismatches
- Hiding errors masks real bugs

**Mitigation:** None needed - this matches TypeScript behavior.

### 8.2 False Positive Risk

**Risk:** Removing suppression might cause extra errors

**Mitigation:**
1. Use conformance test suite to validate
2. Compare against tsc output
3. Fix any actual bugs causing false positives

### 8.3 Performance Impact

**Risk:** More diagnostics = slower compilation

**Assessment:** Minimal impact
- Diagnostic creation is already fast
- Only adds work when types contain errors (rare in valid code)

---

## 9. Recommended Fix Strategy

### Phase 1: Direct Fix (1-2 hours)

**Action:** Comment out the suppression check

```rust
// File: wasm/src/thin_checker.rs:13074
// if self.type_contains_error(source) || self.type_contains_error(target) {
//     return;
// }
```

**Testing:** Run conformance suite, measure improvement

### Phase 2: Validation (1-2 hours)

**Action:** Compare against TypeScript baseline

```bash
cd wasm/differential-test
./find-missing-ts2322.mjs --max=5000
./find-extra-ts2322.mjs --max=5000
```

**Expected Results:**
- Missing TS2322: 310 → ~60-80
- Exact match: 30.8% → ~45%

### Phase 3: Refinement (Optional)

**Action:** Add configuration flag for gradual rollout

```rust
// In context:
pub suppress_error_diagnostics: bool = false;

// In diagnostic functions:
if self.ctx.suppress_error_diagnostics {
    if self.type_contains_error(source) || self.type_contains_error(target) {
        return;
    }
}
```

---

## 10. Conclusion

### Root Cause Summary

**The 310 missing TS2322 errors are caused by explicit diagnostic suppression in the checker layer, NOT by issues in the solver layer.**

**Key Points:**
1. ✅ Solver layer correctly returns `False` for ERROR types
2. ✅ Compat layer correctly delegates to subtype checker
3. ❌ **Checker layer suppresses diagnostics before creation** (THE BUG)
4. ❌ Two functions suppress ERROR type diagnostics

### Fix Summary

**Single-line fix** with massive impact:
- Comment out 2 lines: `thin_checker.rs:13074-13076`, `13042-13044`
- Expected improvement: **+200-250 visible errors**
- Conformance jump: **30.8% → ~45%** (+14pp)

### Next Steps

1. ✅ This analysis - COMPLETE
2. ⏭️ Implement fix (remove suppression)
3. ⏭️ Validate with conformance suite
4. ⏭️ Measure actual improvement
5. ⏭️ Address any new issues found

---

## 11. References

### Files Modified/Analyzed:
- `wasm/src/thin_checker.rs` - **PRIMARY SUPRESSION LOCATION**
- `wasm/src/solver/subtype.rs` - ✅ Working correctly
- `wasm/src/solver/compat.rs` - ✅ Working correctly
- `wasm/src/solver/diagnostics.rs` - ✅ Working correctly
- `wasm/src/solver/lower.rs` - ERROR type creation

### Key Functions:
- `ThinChecker::error_type_not_assignable_with_reason_at` (line 13066) - **SUPPRESSES**
- `ThinChecker::error_type_not_assignable_at` (line 13041) - **SUPPRESSES**
- `ThinChecker::type_contains_error` (line 21644) - ✅ Detection logic
- `SubtypeChecker::check_subtype` (line 230) - ✅ Returns False for ERROR

### Related Analysis:
- Task 2 Analysis: Initial investigation of fallback behavior
- Conformance test suites: ts2322_assignment_tests.ts, ts2322_edge_cases.ts

---

**Analysis Complete. Root cause identified with single-line fix path.**

Ready for implementation.
