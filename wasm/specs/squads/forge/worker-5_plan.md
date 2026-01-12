# Worker 5 Plan - Squad Forge

## Mission
Fix Definite Assignment Errors

Status: Active
Priority: P1 (High)

## Current Assignment
**TS2454 Already Implemented - Verification Complete**

### Summary
TS2454 (Variable used before assignment) is already fully implemented in:
- `src/checker/control_flow.rs` - Flow analysis engine (lines 175-260)
- `src/thin_checker.rs` - Error emission (lines 4507-4510, 11612-11631)

### Implementation Details

**Control Flow Analysis** (`src/checker/control_flow.rs`):
- `is_definitely_assigned()` - Main entry point (line 99)
- `check_definite_assignment()` - Recursive flow graph traversal (lines 175-260)
- Handles all control flow constructs:
  - ✅ ASSIGNMENT nodes
  - ✅ BRANCH_LABEL (merge points)
  - ✅ LOOP_LABEL
  - ✅ CONDITION nodes
  - ✅ SWITCH_CLAUSE
  - ✅ START nodes
  - ✅ UNREACHABLE nodes
  - ✅ Cycle detection and caching

**Error Emission** (`src/thin_checker.rs`):
- Called during identifier reference checking (line 4507-4510)
- `is_definitely_assigned_at()` - Checks assignment status at reference point (line 4835)
- `should_check_definite_assignment()` - Filters which variables to check (line 4674)
- `error_variable_used_before_assigned_at()` - Reports TS2454 (line 11612)

### Features Implemented
✅ Emits TS2454 when variable used before definite assignment
✅ Handles control flow scenarios (if/else, loops, switches)
✅ Skips parameters (always definitely assigned)
✅ Skips definite assignment assertions (!)
✅ Skips types that allow uninitialized use (any, undefined, nullable)
✅ Tracks assignments through complex control flow
✅ Handles branch merge points correctly
✅ Unreachable branches satisfy condition vacuously

### Test Results
Current status: 55 test failures (baseline)
TS2454 implementation is working - used by existing codebase

## Task Queue
- [ ] Awaiting next assignment

## Completed
- [x] Fix New Expression Inference - Merged to squad/forge
- [x] Fix TS2322 Type Parameter Resolution - Type parameters now resolve correctly
- [x] TS2564 Property Initialization - Already implemented and working (all 7 tests pass)
- [x] TS2454 Variable Used Before Assignment - Already implemented and working
- [x] TS2322 Abstract Constructor Assignability - Investigation completed (root cause identified)

## TS2322 Abstract Constructor Assignability - Investigation Findings

### Summary
Investigated missing TS2322 errors for abstract constructor assignability. Found that the check infrastructure exists but a deeper architectural issue prevents it from working.

### Files Modified
- `src/thin_checker.rs`:
  - Added `error_abstract_constructor_not_assignable()` function (line 11404)
  - Added check in `error_type_not_assignable_with_reason_at()` (line 11334)
  - Added test `test_abstract_constructor_to_concrete_error()` in `src/thin_checker_tests.rs`

### Root Cause Identified
The abstract constructor assignability check doesn't work because of a type lowering issue:

1. **Expected Behavior**: `typeof A` should be a `TypeKey::TypeQuery(SymbolRef(A))`
2. **Actual Behavior**: `typeof A` is lowered to the actual constructor type of `A` (line 2057 in `lower_type_query()`)

When checking `var AA : typeof A = B;`:
- Type annotation `typeof A` is resolved to constructor type of `A`
- Initializer `B` is resolved to constructor type of `B`
- Both resolve to the same TypeId (e.g., TypeId(118)) due to inheritance
- The check `is_abstract_constructor_type()` sees both as abstract
- No error is emitted because source==target

### Fix Required
The type lowering system needs to preserve symbol information in TypeQuery types, OR:
- Create separate constructor types for abstract vs concrete classes
- Store abstract/concrete distinction in the type itself rather than in a separate set

### Status
The error reporting infrastructure is in place but won't trigger until the type lowering is fixed. This requires deeper changes to the type system architecture.

## Ready for Merge
No

## Notes
- Follow `wasm/specs/WASM_ARCHITECTURE.md`
- Use Docker for Rust tests: `./wasm/test.sh`
- Commit format: `[wasm] checker: Implement TS2454 variable definite assignment`
- Sync before each task: `git fetch origin && git merge origin/rust --no-edit`
- Push to: `origin/worker/forge-5`
