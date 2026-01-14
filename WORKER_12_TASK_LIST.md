# Worker 12 Task List

**Branch:** worker-12
**Reports to:** EM-3 (Semantics Squad)
**Focus:** Type checking strictness, error messages, TypeScript compiler correctness

---

## CURRENT TASK

### Task 4: Reduce TS2322 "Missing Errors" by Improving Type Inference Tracking
**Status:** READY TO START

**Priority:** HIGH
**Expected Impact:** +50-100 exact matches (convert "missing errors" to "exact matches")

**Objective:** Currently, some TS2322 ("Type X is not assignable to type Y") errors are not being emitted when they should be. This reduces our "Exact Match" score. Track down and fix the inference logic that's causing these errors to be missed.

**Subtasks:**
- [ ] Analyze conformance test failures - find cases where TS2322 should emit but doesn't
- [ ] Search for `getBaseConstraintOfType` usages that might return `any` incorrectly
- [ ] Check `checkTypeRelatedTo` for early returns that skip error emission
- [ ] Verify `createDiagnosticForNode` is called in all type mismatch paths
- [ ] Add tests for fixed cases

**Key Files:**
- `src/compiler/checker.ts` - Type checking logic (lines ~21000-23000)
- `tests/cases/compiler` - Conformance test cases

**Success Criteria:**
- Increase Exact Match score by at least 2 percentage points
- No regression in Extra Errors
- Conformance tests pass

---

## PENDING TASKS

### Task 5: Improve TS7006 "Implicit Any" Error Messages
**Status:** PENDING

**Priority:** MEDIUM
**Expected Impact:** Better developer experience

**Objective:** TS7006 errors currently say "Parameter X implicitly has an 'any' type". Enhance this to show WHERE the type was inferred from (similar to Task 3's type tracing).

**Subtasks:**
- [ ] Find TS7006 emission in checker.ts
- [ ] Add contextual information about where the 'any' came from
- [ ] Include suggestion: "Add type annotation for X"
- [ ] Test with common scenarios

**Key Files:**
- `src/compiler/checker.ts`
- `src/compiler/diagnosticMessages.json`

---

### Task 6: Fix "Excess Property Checking" Edge Cases
**Status:** PENDING

**Priority:** MEDIUM
**Expected Impact:** Reduce false positives

**Objective:** Fresh object literals with excess properties sometimes error incorrectly. Fix the logic to match tsc behavior in edge cases involving intersection types, generic constraints, and index signatures.

**Subtasks:**
- [ ] Find `getFreshType` and related freshness checking logic
- [ ] Identify test cases where excess property errors are wrong
- [ ] Fix the checking logic for complex object literal scenarios
- [ ] Add regression tests

**Key Files:**
- `src/compiler/checker.ts` (freshness logic around lines 18000-19000)
- `src/compiler/types.ts` (object literal types)

---

### Task 7: Enhance Generic Type Error Messages
**Status:** PENDING

**Priority:** LOW
**Expected Impact:** Better error messages for complex generics

**Objective:** When generic type instantiation fails, show better information about WHICH type argument caused the failure.

**Subtasks:**
- [ ] Find generic instantiation error reporting
- [ ] Add context showing which type parameter failed
- [ ] Show the constraint that was violated
- [ ] Example: "Type 'string' does not satisfy constraint 'extends number' for type parameter 'T'"

**Key Files:**
- `src/compiler/checker.ts` (generic type checking)
- `src/compiler/diagnosticMessages.json`

---

## COMPLETED TASKS

### Task 1: Test the spike in errors after `Any`→`Unknown` fallback change
**Status:** COMPLETED

**Report:** WORKER_12_TASK_1_REPORT.md

**Results:**
- ~478 baselines affected (228 error, 250 type)
- Expected: 200-400 | Actual: ~478 (within acceptable range)
- All changes CORRECT - 0 false positives
- Compiler is now STRICTER as intended

---

### Task 2: Enhance TS2322 Error Messages
**Status:** COMPLETED

**What was done:**
- Added `createPropertyErrorMessage()` helper function
- Added diagnostic message "The error is in property '{0}'" (code 9512)
- Modified `elaborateElementwise()` to use property-aware messages

---

### Task 3: Add Type Tracing to Errors
**Status:** COMPLETED

**What was done:**
- Added `addTypeOriginInfo()` helper function
- Added diagnostic messages for type origin (codes 9513, 9514, 9515)
- Modified `reportRelationError()` to include type origin information

---

## KEY FILES

- `src/compiler/checker.ts` - Main type checker (3MB+)
  - Lines 21000-23000: Type checking and subtyping
  - Lines 18000-19000: Freshness and excess property checking
  - Lines 22000-22700: Error reporting

- `src/compiler/types.ts` - Type representation
- `src/compiler/diagnosticMessages.json` - Error message definitions
- `tests/cases/compiler` - Conformance test suite

---

## ANTI-TASKS

**DO NOT work on:**
- Parser work (EM-1's responsibility)
- Binder/symbol table work (EM-2's responsibility)
- Performance optimization (correctness first)
- Rust/WASM solver in `wasm/src/solver/` (separate project)

---

## WORKFLOW

1. Work on current task fully
2. Run tests: `npm run build:compiler` and `npm test`
3. Commit: `git add -A && git commit -m "Complete: <task>"`
4. Push: `git push origin worker-12 --force`
5. STOP and wait for EM-3 review

---

## NOTES

**Worker 12 Scope:** TypeScript compiler in `src/compiler/` (TypeScript implementation)
**NOT:** Rust solver in `wasm/src/solver/` (separate WASM implementation)

EM_3_TASKS.md path references to `src/solver/` are incorrect - actual paths are:
- TypeScript: `src/compiler/checker.ts`
- Rust: `wasm/src/solver/subtype.rs`
