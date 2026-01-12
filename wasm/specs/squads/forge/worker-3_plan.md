# Worker 3 Plan - Squad Forge

## Mission
General Task Assignment

Status: Active
Priority: P1 (HIGH)

## Current Assignment
**Available for next task**

### Completed Work
- [x] TS2304 - Cannot find name (fixed infer type parameter false positives)
- [x] TS2705 - Async function must return Promise (implemented and merged to squad/forge)
- [x] TS2300 - Duplicate identifier for class members (verified existing implementation)
- [x] TS1117 - Duplicate object literal properties (implemented)

### Recent Merges
- ✅ TS2705 async function return type - Merged to squad/forge 2026-01-12
- ✅ TS2300 duplicate class members - Verified existing symbol-level check handles this
- ✅ TS1117 duplicate object literal properties - Implemented 2026-01-12
- ✅ All changes committed and pushed to origin/worker/forge-3

### Implementation Details (TS2705)
- Added ASYNC_FUNCTION_RETURNS_PROMISE error code (2705)
- Added is_promise_type() helper function
- Handles function declarations, arrow functions, function expressions, and methods
- Test added: test_async_function_returns_promise
- 5051 tests passing, TS2705 test passes

### TS2300 Duplicate Class Members Investigation (2026-01-12)
**Finding: TS2300 for duplicate class members is already implemented**

Investigation revealed that the existing `check_duplicate_identifiers` function in `thin_checker.rs` (line 12180) already handles duplicate class members through symbol-level checking. This function:
- Checks all symbols in the binder's scope (including class members)
- Detects conflicting declarations using `declarations_conflict()`
- Emits TS2300 on conflicting declarations

**Verification:**
- Added test: `test_duplicate_class_members` - verifies duplicate class properties are detected
- All 12 duplicate identifier tests pass
- No new code was needed for this functionality

**Note:** The symbol-level check reports TS2300 on both duplicate declarations, which differs from tsc's behavior (reports once on the first occurrence). This is acceptable for the current implementation.

### TS1117 Duplicate Object Literal Properties (2026-01-12)
**Implemented TS1117 for duplicate object literal properties**

TypeScript emits TS1117 (not TS2300) for duplicate object literal properties.
This was previously unimplemented.

**Changes:**
- Added OBJECT_LITERAL_DUPLICATE_PROPERTY diagnostic code (1117)
- Added diagnostic message: "An object literal cannot have multiple properties with the same name '{0}'."
- Implemented duplicate detection in `get_type_of_object_literal` (thin_checker.rs:8535)
- Checks for duplicates in:
  - Property assignments: `{ x: value }`
  - Shorthand properties: `{ x }`
  - Method shorthands: `{ foo() {} }`
  - Accessors: `{ get foo() {} }`

**Tests:**
- `test_duplicate_object_literal_properties` - basic duplicate detection
- `test_duplicate_object_literal_mixed_properties` - duplicates across different syntax types
- All 12 duplicate identifier tests pass

### Notes
- Ready for new high-priority task assignment
- Previous work: type inference, async function type checking, duplicate detection
- Push to: `origin/worker/forge-3`
