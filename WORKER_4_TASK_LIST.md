# Worker 4 Task List

Maintained by EM-1

## 🔴 CURRENT TASK: TS2322 Literal Type Narrowing & Type Accuracy

**Last Updated:** 2026-01-15
**Status:** 🔄 ASSIGNED
**Priority:** 🔴 CRITICAL
**Impact:** HIGH - Core type accuracy improvements

### Task Description

Fix type mismatch (TS2322) and implicit any (TS7006) error accuracy by improving literal type narrowing, union type handling, and contextual typing. This is critical for achieving 80%+ exact match conformance.

### Context

From worker-11's Task 6 analysis:
- **Missing TS2322:** 105 errors (primary issue: Abstract Constructor Assignability)
- **Extra TS2322:** 548 errors (~0.5% false positive rate - acceptable)

Key issue categories identified:
1. **Await type resolution** - `unknown` vs `boolean` type mismatches
2. **Literal type narrowing** - Fails in assignments and conditionals
3. **Union type handling** - Incorrectly narrows or fails to narrow
4. **Contextual typing** - Object literal property types not inferred correctly

### Focus Areas

#### 1. Literal Type Narrowing in Assignments
**Problem:**
```typescript
let x: "hello" | "world" = "hello";
if (Math.random() > 0.5) {
    x = "world";
}
// x should be narrowed to "hello" | "world", not string
```

**Files:** `wasm/src/checker/control_flow.rs`, `wasm/src/checker/thin_checker.rs`

#### 2. Union Type Handling with Literals
**Problem:**
```typescript
function f(x: "a" | "b"): void {
    if (x === "a") {
        // x should be narrowed to "a" here
    }
}
```

**Files:** `wasm/src/checker/thin_checker.rs`, `wasm/src/solver/subtype.rs`

#### 3. Contextual Typing for Object Literals
**Problem:**
```typescript
type Foo = { method(x: string): void };
const foo: Foo = {
    method(x) { }  // x should be inferred as string
};
```

**Files:** `wasm/src/checker/thin_checker.rs`

#### 4. Shorthand Method Parameter Types
**Related to worker-12's task:**
```typescript
type Method = {
    method(...args: [type: string, cb: (e: string) => void]): void;
};
const obj: Method = {
    method(type, cb) { }  // Error: Cannot find name 'type', 'cb'
};
```

**Note:** This is primarily worker-12's task, but related work may help here.

### Implementation Steps

1. **Investigation Phase**
   - Run 100-200 sample conformance tests focusing on TS2322 errors
   - Categorize missing vs extra TS2322 by root cause
   - Identify 3-5 high-frequency patterns to fix
   - Document findings in `TS2322_LITERAL_NARROWING_ANALYSIS.md`

2. **Implementation Phase**
   - Fix literal type narrowing in control flow
   - Improve union type handling in subtype checker
   - Enhance contextual typing for object literals
   - Add tests for each fix

3. **Validation Phase**
   - Run conformance tests after each fix
   - Track TS2322 missing/extra counts
   - Ensure no regressions in valid error detection

### Success Criteria

- [ ] Investigation complete with 3-5 patterns identified
- [ ] At least 2 patterns fixed with code changes
- [ ] Missing TS2322 reduced from 105 to <50
- [ ] Extra TS2322 not increased significantly (<600)
- [ ] Conformance tests show improvement
- [ ] No regressions in valid error detection

### Files to Modify

- **Primary:** `wasm/src/checker/control_flow.rs`
- **Primary:** `wasm/src/checker/thin_checker.rs`
- **Maybe:** `wasm/src/solver/subtype.rs` (if union type issues)
- **Tests:** `wasm/src/checker/control_flow_tests.rs` (add new tests)

### Timeline

- **Investigation:** 1 day
- **Implementation:** 2-3 days
- **Validation:** 1 day
- **Total:** 3-5 days

### Dependencies

- None (can start immediately)
- Coordinate with worker-3 on TS2564 Phase 2 (both use control_flow.rs)
- Coordinate with worker-12 on shorthand method typing (related issues)

### Expected Impact

**Baseline:**
- Missing TS2322: 105
- Extra TS2322: 548
- Exact match: ~44%

**Target:**
- Missing TS2322: <50 (52% reduction)
- Extra TS2322: <600 (acceptable)
- Exact match: ~50% (+6pp improvement)

---

## Completed Tasks

### Flow Recording (2026-01-15) ✅
- **Status:** Complete and merged to em-team-1
- **Summary:** Fixed flow recording for statements and identifiers
- **Test Results:** All 54/54 control_flow tests passing 🎉
- **Commits:**
  - a163cbed8c9 [wasm] binder: add flow recording for statements and identifiers
  - 4c2544eb316 [wasm] flow: fix literal type narrowing in assignments

### Application Expansion Tests (2026-01-15) ✅
- **Status:** Complete and merged to em-team-1
- **Summary:** Fixed all failing application expansion tests in the type solver
- **Test Results:** All 34/34 application expansion tests passing 🎉
- **Changes:**
  - Fixed test setup in `evaluate_tests.rs`
  - Added default type parameter support in `instantiate.rs`

---

## Notes

- Work in: /tmp/orchestrator-workspace/worktrees/worker-4
- Push to worker-4 branch when complete
- Coordinate with worker-3 (both working on control flow)
- Coordinate with worker-12 (shorthand method typing)

---

## Previous Task Note

**Recursion Guards** - This task was previously assigned but investigation revealed it's already fully implemented in `wasm/src/solver/subtype.rs` with depth counter (MAX_DEPTH = 100) and TS2589 error emission. No crashes found in testing.
