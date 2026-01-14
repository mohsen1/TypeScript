# Worker 12 Task List

**Branch:** worker-12
**Reports to:** EM-3 (Semantics Squad)
**Focus:** Type checking strictness, error messages

---

## CURRENT TASK

### Task 1: Test the spike in errors after `Any`→`Unknown` fallback change
**Status:** BLOCKED - Waiting for worker-9 and worker-10 to complete their audit and replacement tasks

**Dependencies:**
- worker-9: Audit all `any` return statements in checker.ts
- worker-10: Replace `any` fallback with `unknown` in type checker

**When unblocked:**
1. Run conformance tests after the change
2. Measure the spike in errors (expect +200-400 missing errors to become extra/exact)
3. Verify the errors are CORRECT (not false positives)
4. Document findings in a report

---

## PENDING TASKS

### Task 2: Enhance TS2322 Error Messages
**Status:** READY TO START

**File:** `src/compiler/checker.ts`, `src/compiler/diagnosticMessages.json`

**Objective:** Improve "Type 'X' is not assignable to type 'Y'" error messages to match tsc format

**Subtasks:**
- [ ] Find TS2322 error generation in checker.ts
- [ ] Add full type path to error messages (e.g., "Type 'string' is not assignable to type 'number' in property 'age'")
- [ ] Include specific property/field causing the failure
- [ ] Match tsc error message format in 90% of cases

**Reference:**
- Search for error code TS2322 in diagnosticMessages.json
- Find where errors are emitted in checker.ts

---

### Task 3: Add Type Tracing to Errors
**Status:** READY TO START

**File:** `src/compiler/checker.ts`

**Objective:** Show WHERE a type came from when an error occurs

**Subtasks:**
- [ ] Add type origin tracking to type checker
- [ ] Modify error emission to include type origin (e.g., "Type 'number' inferred from argument at line 42")
- [ ] Test with common type error scenarios

**Success Criteria:**
- Users can understand and fix errors without debugging
- Error messages match tsc format in 90% of cases

---

## ANTI-TASKS

**DO NOT work on:**
- Parser work (EM-1's responsibility)
- Binder work (EM-2's responsibility)
- Performance optimization (correctness first)
- New type system features (fix existing ones first)

---

## KEY FILES

- `src/compiler/checker.ts` - Main type checker (3MB+)
- `src/compiler/types.ts` - Type representation
- `src/compiler/diagnosticMessages.json` - Error message definitions
- `src/compiler/utilities.ts` - Type utilities
- `src/factory` - Type factory functions

---

## WORKFLOW

1. Work on current task fully
2. Run tests: `npm test` or relevant test commands
3. Commit: `git add -A && git commit -m "Complete: <task>"`
4. Push: `git push origin worker-12 --force`
5. STOP and wait for EM-3 review

---

## NOTES

This codebase is **TypeScript**, not Rust. The EM_3_TASKS.md document references Rust files but the actual implementation is in TypeScript.

Focus areas for Worker 12:
- Testing/measuring type strictness improvements
- Improving error messages for better DX
- Adding type origin/tracing information
