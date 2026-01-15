# Worker 11 Task 2 Analysis: TS2322 Solver Fallback Behavior

Generated: 2026-01-14  
Worker: Worker 11  
Task: Investigate TS2322 solver fallback behavior

---

## Executive Summary

**Status:** 🔍 ANALYSIS COMPLETE  
**Finding:** The codebase is **already quite strict** with ERROR handling. The "310 missing TS2322 errors" problem is NOT primarily caused by `Any` fallback, but rather by a different set of issues.

---

## 1. Current Fallback Strategy

### 1.1 Type Builtin Constants

From `wasm/src/solver/types.rs:14-19`:
```rust
pub const ERROR: TypeId = TypeId(1);
pub const NEVER: TypeId = TypeId(2);
pub const UNKNOWN: TypeId = TypeId(3);
pub const ANY: TypeId = TypeId(4);
```

### 1.2 Lower Fallback (`wasm/src/solver/lower.rs`)

**Fallback to ERROR:**
When lowering fails, the code returns `TypeId::ERROR` in **30+ locations**:
- Missing AST nodes: `None => return TypeId::ERROR` (lines 304, 460, 500, 572, etc.)
- Missing data: `TypeId::ERROR` when `get_*` returns None
- Unhandled syntax kinds: Line 452 `_ => TypeId::ERROR`

**Fallback to UNKNOWN:**
- Line 1650: Generic constraints `constraint.unwrap_or(TypeId::UNKNOWN)` ✅ **ALREADY STRICT**
- Line 2003: Array elements `.unwrap_or(TypeId::UNKNOWN)` ✅ **ALREADY STRICT**  
- Line 2179: Function parameters `unwrap_or(TypeId::UNKNOWN)` ✅ **ALREADY STRICT**
- Line 2180: This parameters `unwrap_or(TypeId::UNKNOWN)` ✅ **ALREADY STRICT**

**Direct ANY returns:**
- Lines 311, 2056: Only when source code contains `any` keyword (correct behavior)

---

## 2. Subtype Checking Rules (`wasm/src/solver/subtype.rs`)

### 2.1 Current Subtype Semantics (lines 262-307)

```rust
// Any is assignable to anything
if source == TypeId::ANY {
    return SubtypeResult::True;
}

// Everything is assignable to any
if target == TypeId::ANY {
    return SubtypeResult::True;
}

// Everything is assignable to unknown
if target == TypeId::UNKNOWN {
    return SubtypeResult::True;
}

// ERROR is NOT compatible (propagates errors)
if source == TypeId::ERROR || target == TypeId::ERROR {
    return SubtypeResult::False;  // ✅ ALREADY STRICT
}
```

### 2.2 Key Findings

1. **ERROR is correctly treated as incompatible** with everything (line 305-307)
2. **Any is still the "universal supertype"** (lines 262-269) - this matches TypeScript
3. **Unknown is used as the strict fallback** for missing type parameters

---

## 3. Root Cause Analysis: Why 310 TS2322 Errors Are Missing

Based on code inspection and test analysis, the missing errors are likely caused by:

### 3.1 Issue 1: Incomplete Type Resolution (not fallback)

When `resolve_type_symbol()` fails (line 2033 in lower.rs):
- Returns `TypeId::ERROR`
- Subtype check returns `False` 
- **BUT** the diagnostic emission might be conditional on successful resolution

### 3.2 Issue 2: Diagnostic Suppression

From `wasm/src/solver/diagnostics.rs:1256-1262`:
```rust
// Error types indicate unresolved types that should trigger TS2322.
PendingDiagnostic::error(
    codes::TYPE_NOT_ASSIGNABLE,
    vec![(*source_type).into(), (*target_type).into()],
```

The diagnostic is emitted when source/target are error types, **BUT**:
1. The checker may skip checking when types are ERROR (early return)
2. The error context might not be properly captured for emission

### 3.3 Issue 3: Solver Bailout in Complex Scenarios

The solver may bail out and return `True` (compatible) in complex type scenarios:
- Deeply nested generics
- Conditional types
- Mapped types
- Recursive types

Look for patterns like:
```rust
// Bailout returning True (hides errors)
return SubtypeResult::True;
```

---

## 4. Concrete Examples from Test Suite

Located: `/tests/cases/conformance/solver/ts2322_assignment_tests.ts`

This test file contains **30 sections** with expected TS2322 errors:

1. **Primitive type mismatches** (lines 17-27)
   - `let numToStr: string = 123;` → TS2322
   - `let strToNum: number = "hello";` → TS2322

2. **Generic type mismatches** (lines 43-47)
   - `let boxWrong: Box<number> = { value: "string" };` → TS2322

3. **Generic function type mismatches** (lines 61-62)
   - `let genericWrong: number = identity<string>("hello");` → TS2322

4. **Union type non-members** (lines 94-98)
   - `let unionWrong: StringOrNumber = true;` → TS2322

5. **Partial intersection types** (lines 113-117)
   - `let intersectionWrong: AB = { a: "test" };` → TS2322

6. **Array element type mismatches** (lines 127-131)
   - `let arrayWrong: number[] = [1, 2, "three"];` → TS2322

7. **Object property type mismatches** (lines 147-151)
   - `let pointWrong: Point = { x: 10, y: "20" };` → TS2322

8. **Function type mismatches** (lines 163-167)
   - `let funcWrong: NumToStr = (x: string) => x;` → TS2322

9. **Unknown type without guard** (lines 269-270)
   - `let unknownWrong: number = unknownValue;` → TS2322

10. **Null/undefined assignments** (lines 298-302)
    - `let nullWrong: number = null;` → TS2322

---

## 5. Patch Plan

### 5.1 Short-term: Diagnostic Emission Audit

**Priority 1:** Verify ERROR types emit diagnostics
- Add logging when `check_subtype` returns `False` for ERROR types
- Ensure `PendingDiagnostic` is created for all False results
- Verify diagnostics reach the emitter

**Files to modify:**
- `wasm/src/solver/subtype.rs` - Add diagnostic tracking
- `wasm/src/solver/diagnostics.rs` - Verify emission logic
- `wasm/src/checker/mod.rs` - Ensure checker doesn't skip ERROR types

**Expected impact:** Convert ~100-200 missing errors to visible errors

### 5.2 Medium-term: Solver Bailout Detection

**Priority 2:** Find and fix silent bailouts
- Search for `return SubtypeResult::True` without proper checks
- Add `TODO_BAILOUT` markers for complex type scenarios
- Implement "best effort" checking instead of bailing

**Files to audit:**
- `wasm/src/solver/subtype.rs` (complex subtype logic)
- `wasm/src/solver/compat.rs` (compatibility layer)
- `wasm/src/solver/operations.rs` (type operations)

**Expected impact:** Convert ~50-100 missing errors to visible errors

### 5.3 Long-term: Type Resolution Completeness

**Priority 3:** Fix symbol resolution and type lowering
- Improve `resolve_type_symbol` to handle more cases
- Add better error recovery in `lower_type`
- Ensure library types (lib.d.ts) are properly loaded

**Files to audit:**
- `wasm/src/solver/lower.rs` - Type lowering
- `wasm/src/solver/db.rs` - Type database
- `wasm/src/lib_loader.rs` - Library loading

**Expected impact:** Convert ~50-100 missing errors to visible errors

---

## 6. Side Effects and Risks

### 6.1 Converting Missing → Extra Errors

**Expected:** Some "missing" errors will become "extra" errors (false positives)

**Mitigation:**
1. Fix the actual bug causing the false positive
2. Improve compatibility layer (lawyer.rs) to match TypeScript quirks
3. Use conformance test suite to validate

**Example:**
- Current: Missing TS2322 for `any` assignment (too permissive)
- After fix: Extra TS2322 for valid TypeScript pattern
- Solution: Add lawyer rule to match TypeScript behavior

### 6.2 Cascading Errors

**Risk:** Fixing one error may expose 10 more downstream

**Mitigation:**
1. Focus on high-impact, low-risk changes first
2. Use gradual rollout (test on subset of conformance suite)
3. Track metrics: exact match % should improve

### 6.3 Performance Impact

**Risk:** More thorough checking = slower compilation

**Mitigation:**
1. Only check when diagnostics are enabled (not in --noEmit)
2. Cache results for repeated checks
3. Profile before/after to measure impact

---

## 7. Recommended Next Steps

### Immediate (Worker 11 can do):
1. ✅ **This analysis** - COMPLETE
2. Search for specific bailout patterns in subtype.rs
3. Add diagnostic logging for ERROR type handling
4. Run conformance tests with diagnostic tracing

### Short-term (requires EM-3 assignment):
1. Implement diagnostic emission audit
2. Fix 5-10 concrete missing error examples
3. Measure impact on conformance metrics

### Medium-term (requires squad coordination):
1. Systematic bailout detection and fixing
2. Type resolution improvements
3. Compatibility layer enhancements

---

## 8. Conclusion

**Key Insight:** The problem is NOT "change Any to Unknown" - that's already done in most places.

**Real Problem:** 
1. Diagnostic emission may be suppressed for ERROR types
2. Solver bailouts in complex scenarios return `True` too early
3. Type resolution failures don't always emit errors

**Solution Focus:** 
- Audit diagnostic emission
- Fix solver bailouts
- Improve type resolution completeness

**Expected Outcome:** 30-40% improvement in missing error count (310 → ~200)

---

## 9. References

### Files Modified/Analyzed:
- `wasm/src/solver/types.rs` - Type definitions
- `wasm/src/solver/lower.rs` - Type lowering (30+ ERROR fallbacks)
- `wasm/src/solver/subtype.rs` - Subtype checking
- `wasm/src/solver/diagnostics.rs` - Diagnostic emission
- `tests/cases/conformance/solver/ts2322_assignment_tests.ts` - Test cases

### Related Scripts:
- `wasm/differential-test/find-missing-ts2322.mjs` - Find missing errors
- `wasm/differential-test/find-ts2322.mjs` - Find false positives

### Conformance Metrics:
- Current missing TS2322: ~310 errors
- Target missing: <100 errors
- Current exact match: 30.8%
- Target exact match: 50%+

---

**Analysis Complete.** Ready for patch implementation.
