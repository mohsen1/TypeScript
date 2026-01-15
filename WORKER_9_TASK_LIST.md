# Worker 9 Task List (Maintained by EM-3)

## Completed Tasks

### Task 1: Rust Implementation Investigation ✅ COMPLETED
**Completed:** 2025-01-14
**Commit:** 02050ae97

**Deliverables:**
- ✅ RUST_STATUS_REPORT.md created with comprehensive findings
- ✅ 186+ Rust files identified and catalogued
- ✅ Build system verified (Cargo.toml, compiles successfully)
- ✅ Architecture documentation reviewed
- ✅ Clean push to worker-9 branch

**Key Findings:**
- Rust implementation lives in `wasm/` directory (not `rust/`)
- 60.8% conformance test match (target: 95%)
- Critical issues: Parser noise (701 errors), Global scope (343 errors), Optimistic defaults (2961 errors)

---

### Task 2: Fix Parser Noise - ASI Simplification ✅ COMPLETED
**Completed:** 2025-01-14
**Commit:** 0f7a22360

**Root Cause Found:**
The Rust implementation had an "enhanced" ASI that checked `is_statement_start()` when there was a line break, but TypeScript's `canParseSemicolon()` does NOT have this extra check.

**Changes Made:**
- Simplified `can_parse_semicolon()` to match TypeScript exactly
- Removed `is_statement_start()` check and special cases
- ASI now applies whenever `scanner.has_preceding_line_break()` is `true`
- Fixed syntax error in `is_statement_start()` (malformed comment on `LessThanToken`)

**Files Modified:**
- `wasm/src/thin_parser.rs`: Simplified ASI logic (lines 538-568)

---

### Task 3: Fix Global Scope - Resolve TS2304 "Error Poisoning" ✅ COMPLETED
**Completed:** 2025-01-14
**Commit:** 284b8b10d

**Root Cause Found:**
In `bind_source_file_with_libs`, lib symbols were being merged AFTER binding the source file. This meant when the binder encountered global symbols like `console`, `Promise`, `Array`, etc., they didn't exist yet, causing TS2304 errors.

**Fix Applied:**
Swapped the order in `bind_source_file_with_libs`:
1. Merge lib symbols FIRST (via `merge_lib_symbols`)
2. THEN bind the source file

This ensures global symbols from lib.d.ts are available during binding, preventing the "error poisoning" cascade where undefined globals cause downstream type errors to be suppressed.

**Files Modified:**
- `wasm/src/thin_binder.rs`: Fixed order in `bind_source_file_with_libs` (lines 698-702)

**Expected Impact:**
- Extra TS2304 errors reduced from 343 to <10
- Missing errors will increase (exposes hidden bugs!)
- Built-in types available everywhere during binding

---

## Current Task
**Status**: IN PROGRESS

**Task 4**: Invert Solver Defaults - Change TypeId::ANY to TypeId::UNKNOWN

**Priority**: 🟠 STRATEGIC (Issue #3 from PROJECT_DIRECTION.md)

**Context:**
We are missing 2,961 errors (60% of conformance failures), including:
- 184 TS2322 (Type Mismatch)
- 357 TS7006 (Implicit Any)

**Root Cause:**
The solver is "optimistic" - when it encounters an unknown type or a resolution failure, it returns `TypeId::ANY`. This suppresses type errors downstream because:
- `any` is compatible with everything
- Invalid operations on `any` don't emit errors

**Analysis:**
See `PHASE1_ANALYSIS.md` which catalogues ~150 occurrences of `TypeId::ANY`:
- **Category A (Keep):** Test files, explicit `any` keyword, type guards
- **Category B (Change to UNKNOWN):** Optimistic defaults that hide bugs

Priority categories from PHASE1_ANALYSIS:
- **P0 (Critical):** Function return defaults, expression type resolution
- **P1 (High Impact):** Call expression handling, binary operation errors
- **P2 (Medium):** Property access, new expressions
- **P3 (Lower):** Built-in method signatures, spread operators

**Description:**
Change "optimistic defaults" from `TypeId::ANY` to `TypeId::UNKNOWN` (or `TypeId::ERROR`) so that type failures emit errors instead of silently succeeding.

**Action Items:**
1. **Start with P0 (Critical) from PHASE1_ANALYSIS:**
   - Function return defaults (B1): Lines 3276, 3307, 3941, 3945, 3989, 3993
   - Expression type resolution (B2): Lines 4254, 4925, 5303

2. **Continue with P1 (High Impact):**
   - Call expression handling (B3): Lines 7111, 7121, 7143, 7144
   - Binary operation errors (B4): Lines 6896, 6929, 6939, 7020, 7028

3. **Verify TypeId::UNKNOWN exists:**
   - Check if `TypeId::UNKNOWN` is defined in the type system
   - If not, use `TypeId::ERROR` or define it

4. **Test incrementally:**
   - Make one category of changes at a time
   - Run conformance tests after each change
   - Expect "extra errors" to increase (this is good - it exposes hidden bugs!)

**Files to Modify:**
- `wasm/src/thin_checker.rs` - Main solver with TypeId::ANY defaults
- `wasm/src/solver/*.rs` - Solver operations

**Expected Outcome:**
- Missing errors will decrease significantly
- Extra errors will increase initially (this is correct behavior!)
- Conformance may decrease temporarily, but correctness increases
- Type errors are properly reported instead of being hidden behind `any`

**Definition of Done:**
- P0 and P1 categories changed from ANY to UNKNOWN
- Build passes (cargo build)
- Code committed and pushed to worker-9

**Note:** This is a strategic change. Expect a regression in "exact match" percentage, but this is the correct path to correctness.

---

## Pending Tasks
_None yet._
