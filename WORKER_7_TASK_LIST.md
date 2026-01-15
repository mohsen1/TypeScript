# Worker-7 Task List

**Squad:** Semantics (Strategic)
**Branch:** `worker-7`
**EM:** EM-2
*Assigned: 2025-01-14*

---

## Priority Mission

Fix Solver "Optimistic Defaults" - **Target: Change UNKNOWN type resolution from returning `ANY` to `UNKNOWN`/`ERROR`**

---

## Background

The current WASM type solver is too "nice." When it encounters:
- Unresolved symbols
- Failed type operations
- Missing type constraints
- Failed generic instantiations

It returns `TypeId::ANY` as a fallback. This **hides errors** instead of exposing them.

**Impact:**
- We're missing 1,841 `TS2322` (Type Mismatch) errors
- We're missing 357 `TS7006` (Implicit Any) errors
- Total: 2,968 missing errors (60% of all missing errors)

**The Fix:**
Change the solver to return `TypeId::UNKNOWN` or `TypeId::ERROR` when resolution fails. This will cause a temporary spike in "extra errors" but will expose the real issues we need to fix.

---

## Assigned Tasks

### 1. Audit Current Type Fallback Points
**Priority:** P0 - Critical
**Files:** `wasm/src/solver/*.rs`

**Action:** Find all locations where the solver returns `TypeId::ANY` as a fallback.

**Search Pattern:** Look for:
- `.with_type(TypeId::ANY)`
- `return TypeId::ANY`
- `self.any_type()`
- Default return values in type resolution functions

**Expected Output:** List of 10-20 locations with file:line references

**Success Criteria:** Complete audit of all fallback points
**Status:** ⏳ PENDING

---

### 2. Implement `TypeId::ERROR` Type
**Priority:** P0 - Critical
**File:** `wasm/src/types/mod.rs`

**Action:** If `TypeId::ERROR` doesn't exist, add it:
- Create the error type variant
- Add display formatting (show as "(error type)")
- Ensure it propagates correctly through type operations
- Make it distinct from `TypeId::UNKNOWN`

**Reason:** We need to distinguish between:
- `UNKNOWN` - "we don't know this type yet (may be resolved later)"
- `ERROR` - "this type resolution failed (permanent error)"

**Success Criteria:** `TypeId::ERROR` exists and is usable
**Status:** ⏳ PENDING

---

### 3. Change Symbol Resolution Fallbacks
**Priority:** P0 - Critical
**File:** `wasm/src/solver/*.rs`

**Action:** Modify symbol resolution to return `TypeId::ERROR` when a symbol cannot be found.

**Key Locations:**
- Symbol table lookup functions
- Import resolution
- Namespace member access
- Property access expressions

**Before:**
```rust
let ty = self.resolve_symbol(name).unwrap_or(self.any_type());
```

**After:**
```rust
let ty = self.resolve_symbol(name).unwrap_or(self.error_type());
```

**Success Criteria:** Unresolved symbols return `TypeId::ERROR`
**Status:** ⏳ PENDING

---

### 4. Change Type Operation Fallbacks
**Priority:** P0 - Critical
**File:** `wasm/src/solver/*.rs`

**Action:** Modify type operations to return `TypeId::ERROR` when operations fail.

**Key Operations:**
- Generic instantiation (missing type arguments)
- Union/intersection types (invalid operands)
- Conditional types (unresolvable conditions)
- Type assertion failures
- Index access types (invalid index)

**Success Criteria:** Failed type operations return `TypeId::ERROR`
**Status:** ⏳ PENDING

---

### 5. Update Error Reporting
**Priority:** P1 - High
**File:** `wasm/src/checker/thin_checker.rs`

**Action:** Ensure the checker properly handles `TypeId::ERROR`:
- Don't emit cascading errors when an expression is `TypeId::ERROR`
- Show "(type error)" in diagnostics instead of "any"
- Track which expressions had resolution failures

**Success Criteria:** Clean error output with minimal cascading
**Status:** ⏳ PENDING

---

### 6. Run Conformance Tests
**Priority:** P1 - High
**Command:** `npm run test:conformance`

**Expected Result:** MASSIVE spike in "extra errors" (1000-2000+)

**Why This Is Good:**
- These errors were already there but hidden by `ANY`
- Now we can see them and fix the root causes
- Each "extra error" is a bug we need to fix

**Success Criteria:** Tests complete, document error spike
**Status:** ⏳ PENDING

---

### 7. Categorize New Errors
**Priority:** P2 - Medium
**Action:** Analyze the new "extra errors" and group them:

| Category | Count | Priority |
|----------|-------|----------|
| Unresolved imports | ? | P0 |
| Missing lib symbols | ? | P0 |
| Generic failures | ? | P1 |
| Property access | ? | P1 |
| Index types | ? | P2 |

**Success Criteria:** Top 5 error categories identified
**Status:** ⏳ PENDING

---

### 8. Document Findings
**Priority:** P2 - Medium
**File:** `WORKER_7_SOLVER_STRICTNESS_ANALYSIS.md`

**Sections:**
1. Changed made (with code snippets)
2. Test results before/after
3. Top 10 error categories with examples
4. Recommended next steps for other squads

**Success Criteria:** Complete analysis document
**Status:** ⏳ PENDING

---

## Success Metrics

| Metric | Before | Target |
|--------|--------|--------|
| Missing TS2322 errors | 1,841 | <200 |
| Missing TS7006 errors | 357 | <50 |
| Extra errors | 3,223 | 4,000-5,000* |
| Exact match | 30.1% | 60%+ |

*We expect a temporary spike in extra errors, which will be addressed by subsequent fixes

---

## Implementation Notes

### What to Change
- **DO:** Change `ANY` → `ERROR` in resolution failures
- **DO:** Update error formatting to show "(error type)"
- **DO:** Document every change location

### What NOT to Change
- **DON'T:** Modify the parser (Worker 5's job)
- **DON'T:** Modify lib loading (Worker 2's job)
- **DON'T:** Add new type system features yet
- **DON'T:** Try to "fix" the extra errors yet (just expose them first)

### Expected Timeline
1. Tasks 1-2: Setup and audit (1-2 hours)
2. Tasks 3-4: Core changes (3-5 hours)
3. Tasks 5-6: Testing (1-2 hours)
4. Tasks 7-8: Analysis (2-3 hours)

**Total:** ~7-12 hours of focused work

---

## Merge Status

**Status:** ⚠️ NO WORK COMPLETED
**Last Updated:** 2026-01-14

**Findings:**
- Worker-7 branch is clean but behind rust
- No commits made on solver defaults work
- Solver defaults work was completed by worker-8 instead
- Worker-7 needs task reassignment or clarification

---

## Notes

- This is a **strategic change** - it will temporarily make things look worse
- The "extra errors" spike is **expected and desired**
- Focus on exposing errors, not fixing them yet
- Work closely with EM-2 to validate approach before merging

---

## Next Steps

After completing these tasks:
1. EM-2 will review and merge to `rust` branch
2. Other squads will use the exposed errors to guide their fixes
3. Worker 7 may be reassigned to fix exposed semantic errors
