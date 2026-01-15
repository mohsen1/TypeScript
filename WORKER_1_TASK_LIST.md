# Worker 1 Task List

## Current Task Status
- **Active Task**: Task 1 (In Progress)
- **Completed Tasks**: None
- **Status**: Working on P0 optimistic defaults

---

## Task Queue

### Task 1: Fix Function Return Defaults in checker/expr.rs (P0 - CRITICAL)
**Status:** IN_PROGRESS
**File:** `wasm/src/checker/expr.rs`
**Priority:** P0

**Description:**
Change `TypeId::ANY` defaults to `TypeId::UNKNOWN` in checker expression handling. These are optimistic defaults that hide type errors.

**Specific Lines to Change:**
- Line 47: `return TypeId::ANY;`
- Line 72: `TypeId::ANY` (in array)
- Line 77: `_ => TypeId::ANY,` (default case)

**Definition of Done:**
- [x] Identify all optimistic defaults in checker/expr.rs
- [ ] Change defaults to TypeId::UNKNOWN
- [ ] Code compiles without errors
- [ ] Conformance test run completed
- [ ] Results documented

---

### Task 2: Fix Apparent Type Defaults in solver/apparent.rs (P0 - CRITICAL)
**Status:** PENDING
**File:** `wasm/src/solver/apparent.rs`
**Priority:** P0

**Description:**
Change `TypeId::ANY` defaults in apparent member kind returns to `TypeId::UNKNOWN`.

**Specific Lines:**
- Line 89: `return Some(ApparentMemberKind::Value(TypeId::ANY));`
- Line 95: `return Some(ApparentMemberKind::Method(TypeId::ANY));`
- Line 104: `return Some(ApparentMemberKind::Method(TypeId::ANY));`
- Line 115, 123, 140, 175, 261: Various member kinds with TypeId::ANY

**Definition of Done:**
- [ ] All apparent type defaults changed to TypeId::UNKNOWN
- [ ] Code compiles
- [ ] Tests run

---

### Task 3: Fix Evaluation Defaults in solver/evaluate.rs (P1 - HIGH)
**Status:** PENDING
**File:** `wasm/src/solver/evaluate.rs`
**Priority:** P1

**Specific Lines:**
- Line 532: check_type == TypeId::ANY (check condition)
- Line 563: check_type == TypeId::ANY
- Line 1169: evaluated_object == TypeId::ANY || evaluated_index == TypeId::ANY
- Line 1503: self.apparent_method_type(TypeId::ANY)
- Line 1516: return Some(ApparentMemberKind::Method(TypeId::ANY))
- Line 2194: self.interner.array(TypeId::ANY)

---

### Task 4: Add Recursion Guards (P5 - STABILITY)
**Status:** PENDING
**Files:** Multiple solver files
**Priority:** P5

**Description:**
Add recursion depth counters to prevent stack overflow crashes.

---

## Notes
- Work on tasks sequentially (Task 1 → Task 2 → Task 3 → Task 4)
- After each task: run tests, commit with descriptive message, push
- Keep test file changes as-is (they test the ANY type intentionally)
- Keep comparisons like `if type == TypeId::ANY` (they're checks, not defaults)
- Keep AnyKeyword handling (when user explicitly types `any`)

## Reporting Format
After completing each task:
- Task completed
- Changes made (file:line_number format)
- Test results (before/after comparison)
- Any issues encountered
