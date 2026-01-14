# Worker 12 Task List

**Branch:** worker-12
**Reports to:** EM-3 (Semantics Squad)
**Focus:** Type checking strictness, error messages

---

## CURRENT TASK

(None - all tasks completed)

---

## BLOCKED TASKS

(None - all tasks completed or unblocked)

---

## COMPLETED TASKS

### Task 1: Test the spike in errors after `Any`→`Unknown` fallback change
**Status:** COMPLETED

**Report:** WORKER_12_TASK_1_REPORT.md

**What was done:**
- [x] Ran conformance tests after worker-9 and worker-10 completed their changes
- [x] Measured the spike in errors
- [x] Verified all errors are CORRECT (no false positives)
- [x] Documented findings in comprehensive report

**Test Results:**
- **Total baselines affected:** ~478 files (228 error baselines, 250 type baselines)
- **Expected spike:** 200-400 errors
- **Actual spike:** ~478 files
- **False positives:** 0 (all changes are CORRECT)

**Key Findings:**
- Circular references now use `unknown` (catches arithmetic errors)
- JSDoc templates now default to `unknown` (catches type mismatches)
- Recursive initializers properly error on `unknown` operations
- globalThis property access handled correctly
- **Compiler is now STRICTER as intended** ✅

**Dependencies Completed:**
- worker-9: "Complete: Audit anyType fallback in checker.ts" (commit 0c641c433)
- worker-10: "Replace anyType fallback with unknownType in 13 locations" (commit 9abf900d7)

---

### Task 3: Add Type Tracing to Errors
**Status:** COMPLETED

**File:** `src/compiler/checker.ts`, `src/compiler/diagnosticMessages.json`

**What was done:**
- [x] Added `addTypeOriginInfo()` helper function to track type origins
- [x] Added new diagnostic messages for type origin (codes 9513, 9514, 9515)
- [x] Modified `reportRelationError()` to include type origin information
- [x] Type origin is now shown as related information when errors occur

**Implementation:**
- Added helper function in checker.ts (line ~22579)
- Messages show where types were inferred from (expression location, return statement, etc.)
- Adds origin info as related information for non-literal/intrinsic types

**Diagnostic Messages Added:**
- "Type '{0}' was inferred from expression at this location" (9513)
- "Type '{0}' was inferred from argument '{1}' at position {2}" (9514)
- "Type '{0}' was inferred from return statement" (9515)

---

### Task 2: Enhance TS2322 Error Messages
**Status:** COMPLETED

**File:** `src/compiler/checker.ts`, `src/compiler/diagnosticMessages.json`

**What was done:**
- [x] Created `createPropertyErrorMessage()` helper function that builds property-aware error messages
- [x] Added new diagnostic message "The error is in property '{0}'" (code 9512)
- [x] Modified `elaborateElementwise()` to use property-aware messages when property context is available
- [x] Enhanced error messages now show full type path including property names

**Implementation:**
- Added helper function in checker.ts (line ~21474)
- Uses `chainDiagnosticMessages()` to append property context
- Falls back to base message when no property context exists

---

## PENDING TASKS

(None - all tasks completed or blocked)

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
