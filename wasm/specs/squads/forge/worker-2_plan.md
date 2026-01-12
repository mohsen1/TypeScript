# Worker 2 Plan - Squad Forge

## Mission
TypeScript Error Code Implementation & Fixes

Status: Active
Priority: P1 (HIGH)

## Completed Tasks

### ✅ TS2304: Cannot Find Name (Fixed)
**Commit**: `a9ccc518fc`
**Issue**: 129 extra TS2304 errors for `infer` type parameters in conditional types
**Fix**: Modified `type_ref_is_promise_like()` and added `collect_infer_type_parameters()` to handle `infer` type parameters in conditional types
**Result**: Reduced from 129 extra to 3 extra (97.7% improvement)

### ✅ TS2355: Function Must Return Value (Fixed)
**Commit**: `783349f8b4`
**Issue**: 82 extra TS2355 errors for async functions with `Promise<void>` return types
**Fix**: Modified `type_ref_is_promise_like()` to recursively check `TypeKey::Application` types
**Result**: Fixed async Promise<void> false positives

### ✅ TS2564: Property Has No Initializer (Already Implemented)
**Status**: Fully implemented with 7/7 tests passing
**Features**: Handles destructuring, optional properties, static properties, definite assignment assertions

### ✅ TS7006: Implicit Any Parameter (Already Implemented)
**Status**: Fixes for destructuring and setters already in code
**Features**:
- Lines 17617-17626: Skips TS7006 for destructuring parameters (object/array binding patterns)
- Lines 18385-18396: Skips TS7006 for setter parameters

### ✅ TS2454: Variable Used Before Assignment (Verified)
**Status**: FlowAnalyzer implementation complete and working

### ✅ TS7010: Implicit Any Return Type (Already Implemented)
**Commit**: `2d5165ec51`
**Status**: 5/5 tests passing
**Investigation (Jan 12)**:
- Differential test shows 42 extra, 15 missing TS7010 errors
- Root cause: WASM package (`pkg/wasm.js`) built on Jan 11 12:25, BEFORE TS7010 implementation
- All unit tests pass, including:
  - `test_ts7010_async_function_no_false_positive` - async getters (line 5561-5564)
  - `test_ts7010_class_expression_no_false_positive` - class expressions
  - `test_ts7010_exactly_any_return` - exact 'any' detection
  - `test_ts7010_null_undefined_return` - null/undefined handling
  - `test_ts7010_return_path_analysis` - control flow analysis
- Differential test results are from outdated WASM build and do not reflect current implementation

## Current Status

### Active Task: Namespace Merging (GOALS.md Objective #1)
**Status**: 5/13 tests passing (partial implementation)
**Latest Commit**: `5c2edc248a`

#### Implementation Progress

**Complete (5/13 tests passing)**:
1. ✅ Type exports (interfaces) - enum and function work in both normal and reverse order
2. ✅ Class type exports (normal order) - `namespace_merges_with_class_exports` passes

**Remaining Work (8 tests)**:
1. ❌ Class type exports (reverse order) - `namespace_merges_with_class_exports_reverse_order`
2. ❌ All value access tests - `Foo.value` for class/enum/function
3. ❌ Element access - `Foo["value"]`

#### Key Fixes Implemented

**binder.rs declare_symbol()** (lines 1064-1073):
```rust
// Update value_declaration for merged class/enum/function + namespace symbols
if (flags & symbol_flags::CLASS) != 0
    || (flags & symbol_flags::FUNCTION) != 0
    || (flags & symbol_flags::REGULAR_ENUM) != 0
{
    sym.value_declaration = declaration;
}
```
When a class/enum/function merges with a namespace, `value_declaration` now correctly points to the class/enum/function instead of the namespace.

**thin_checker.rs resolve_qualified_name()** (lines 1934-1963):
Added symbol-based lookup for merged symbols before type-based lookup, allowing type access like `Foo.Bar`.

**thin_checker.rs type_reference_symbol_type()** (lines 975-1008):
For merged class+namespace symbols, returns constructor type (with exports) instead of instance type.

**thin_checker.rs merge_namespace_exports_into_constructor()** (lines 1010-1055):
Now includes type-only exports (interfaces, type aliases) in addition to value exports.

**thin_checker.rs get_type_of_property_access_inner()** (lines 7173-7205):
Added file_locals-based lookup for merged symbols to handle value access attempts.

#### Root Cause of Value Access Failures

The value access failure (`Foo.value` not returning NUMBER) is a complex issue:

1. **Type Computation Order**: When `Foo.value` is accessed, the type of `Foo` might be computed before namespace exports are fully populated
2. **Cache Invalidation**: Simply invalidating caches doesn't solve the problem because the type is computed with incomplete information
3. **Symbol-to-Type Mapping**: The solver doesn't track which symbol a Callable type came from, making it difficult to find exports from the type

#### Potential Solutions for Remaining 8 Tests

1. **Modify Type Lowering**: Track symbol origin in Callable types to enable property lookup from exports
2. **Lazy Type Computation**: Ensure merged symbol types are only computed after all declarations are bound
3. **Solver-Level Integration**: Add merged symbol handling directly in `solver::property_access_type()`
4. **Pre-Compute Exports**: Populate namespace exports in a pre-pass before type checking begins

### Test Results
- **5087+ tests passing** (+5 from namespace merging)
- TS2304: 7/7 tests passing
- TS2355: 7/7 tests passing
- TS2564: 7/7 tests passing
- TS7010: 5/5 tests passing
- Namespace merging: 5/13 tests passing (38% complete)

### Key Implementation Locations
- `src/thin_checker.rs` - Main type checker with all error code implementations
- `src/thin_checker_tests.rs` - Comprehensive test suite
- `src/checker/control_flow.rs` - Flow analysis for definite assignment
- `src/checker/types/diagnostics.rs` - Error code definitions

## Notes
- All sync/merge operations completed successfully
- Changes pushed to `origin/worker/forge-2`
- Namespace merging requires additional work to handle value position access (8 tests remaining)
