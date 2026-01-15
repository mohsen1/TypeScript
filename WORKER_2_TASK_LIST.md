# Worker 2 Task List

Maintained by EM-1

## Current Tasks

### [ACTIVE] Task 2: Investigate and Fix TS2571 Over-Reporting

**Priority:** 🔴 CRITICAL (Tier 2 - Type Checker Accuracy)

**Status:** 🔵 STARTING

**Assigned:** 2026-01-15

### Problem

The type checker emits **TS2571 "Object is of type 'unknown'"** errors in situations where it should not. This is related to the TS2683 fix (implicit `this` in functions) and may be a side effect of how `this` typing is handled.

**Current Impact:** High occurrence in conformance tests (exact count to be determined)

**Root Cause Investigation:**

When `this` is used inside a regular function (not a method), the TS2683 fix correctly emits "this implicitly has type 'any'". However, the property access on `this` may incorrectly emit TS2571 instead:

```typescript
function foo() {
    this.x = 1;  // Should emit: TS2683 only
                // Currently may emit: TS2571 (Object is of type 'unknown')
}
```

**Expected Behavior:**
- When `this` has no type in a non-method function → emit TS2683
- Do NOT emit additional TS2571 for property access on untyped `this`

**Action Items:**

1. **Locate TS2571 emission points** in `wasm/src/thin_checker.rs`
   - Search for `TS2571` or diagnostic_codes::TS2571
   - Find where "Object is of type 'unknown'" is emitted

2. **Analyze the interaction with TS2683 fix**
   - Review `current_this_type()` handling (around line 629)
   - Check if property access adds TS2571 on top of TS2683

3. **Implement suppression logic**
   - When TS2683 is emitted for implicit `this`, suppress TS2571 for property access
   - OR: Ensure `this` gets a more specific type when TS2683 context is detected

4. **Test cases to verify:**
   ```typescript
   // Should emit ONLY TS2683, not TS2571
   function bar() {
       this.prop = 1;
   }

   // Should still emit TS2571 when truly unknown
   function baz(x: unknown) {
       x.prop;  // TS2571 expected
   }
   ```

5. **Run conformance tests:**
   ```bash
   cd wasm/differential-test
   bash run-conformance.sh --max=200 --workers=4
   ```
   - Track TS2571 count before/after
   - Track TS2683 count (should not decrease)
   - Ensure TS2683 fix is not regressed

**Target Metrics:**
| Error Code | Current | Target |
|------------|---------|--------|
| TS2571 extra | TBD | Reduce by 50%+ |
| TS2683 missing | 0 | Maintain at 0 |

**Key Files:**
- `wasm/src/thin_checker.rs` - TS2571 emission points, `current_this_type()`
- `wasm/src/checker/types/diagnostics.rs` - error code definitions

**Reference:** See `PROJECT_DIRECTION.md` Tier 2 section for TS2571/TS2683 interaction.

---

## Completed Tasks

### [COMPLETED] Task 1: Add Special Handling for super() Calls in ThinCheckerState

**Priority:** High (Tier 2 - Type Checker Accuracy)

**Status:** Complete

**Commit:** dc7519914 - "fix: add special handling for super() calls in ThinCheckerState"

**Description:**
Fixed the type checker to properly handle `super()` calls in constructor methods. This ensures correct type checking when calling parent class constructors.

**Example Fixed:**
```typescript
class Parent {
    constructor(x: number) {}
}

class Child extends Parent {
    constructor(y: string) {
        super(42);  // Should type-check: number argument matches Parent constructor
    }
}
```

**Changes Made:**
- Modified `wasm/src/thin_checker.rs` - added special handling for `super()` calls
- Improved type resolution for constructor inheritance
- Ensured proper type checking for super() arguments

**Validation:**
- Code committed to worker-2 branch
- Merged into em-team-1
- Merged into rust branch (dc7519914)

---

## Notes

- **Worktree:** `/tmp/orchestrator-workspace/worktrees/worker-2`
- **Branch:** `worker-2`
- **Target branch:** `em-team-1`
- **Squad:** Type Squad
- **Focus:** `super()` call handling

## Next Assignment

*Awaiting next task from EM-1*

---

## Previous Activity Log

### 2026-01-15
- Completed super() handling fix (dc7519914)
- Merged to em-team-1
