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

---

### Task 4: Invert Solver Defaults - Change TypeId::ANY to TypeId::UNKNOWN ✅ COMPLETED
**Completed:** 2025-01-14
**Commits:** 0fed63f73 (P0), ae04f7619 (P1)

**Root Cause:**
The solver was "optimistic" - when it encountered an unknown type or a resolution failure, it returned `TypeId::ANY`. This suppressed type errors downstream because:
- `any` is compatible with everything
- Invalid operations on `any` don't emit errors

**P0 (Critical) Changes Applied:**
1. Call signature return default: Changed `(TypeId::ANY, None)` to `(TypeId::UNKNOWN, None)`
2. Construct signature return default: Changed `(TypeId::ANY, None)` to `(TypeId::UNKNOWN, None)`
3. Type predicate missing annotation: Changed `TypeId::ANY` to `TypeId::UNKNOWN`
4. Type predicate missing node: Changed `TypeId::ANY` to `TypeId::UNKNOWN`

**P1 (High Impact) Changes Applied:**
1. Binary operand type fallback: `TypeId::ANY` → `TypeId::UNKNOWN`
2. Missing node in binary expression: `TypeId::ANY` → `TypeId::UNKNOWN`
3. Type stack unwrap_or defaults: `TypeId::ANY` → `TypeId::UNKNOWN`

**Files Modified:**
- `wasm/src/thin_checker.rs`: P0 and P1 defaults

**Expected Impact:**
- Missing errors will decrease significantly
- Extra errors will increase initially (this is correct behavior!)
- Conformance may decrease temporarily, but correctness increases
- Type errors are properly reported instead of being hidden behind `any`

---

### Task 5: Continue Inverting Solver Defaults - P2/P3 Categories ✅ COMPLETED
**Completed:** 2025-01-14
**Commits:** f5343b322e6 (P2)

**Root Cause:**
Continuing Task 4, P2 (Medium) and P3 (Lower) categories remained with TypeId::ANY defaults.

**P2 (Medium) Changes Applied:**
1. Parameter without type annotation: `TypeId::ANY` → `TypeId::UNKNOWN`
2. Index signature key/value types: `TypeId::ANY` → `TypeId::UNKNOWN`
3. Property type defaults (2 occurrences): `TypeId::ANY` → `TypeId::UNKNOWN`
4. Class expression fallback: `TypeId::ANY` → `TypeId::UNKNOWN`
5. Await expression fallback: `TypeId::ANY` → `TypeId::UNKNOWN`
6. Parenthesized expression fallback: `TypeId::ANY` → `TypeId::UNKNOWN`

**Files Modified:**
- `wasm/src/thin_checker.rs`: P2 parameter, index signature, property, and expression defaults

**Note:** P3 (Lower) categories remain for future work if needed.

---

## Current Task
_None assigned._ Awaiting EM-3 directive.

---

## Pending Tasks
_None yet._
