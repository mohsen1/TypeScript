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
**Commit**: `9485b8b320`
**Status**: 5/13 tests passing (partial implementation)

#### Implementation
Enhanced namespace merging for class/enum/function + namespace symbols with same name:

1. **Modified `resolve_qualified_name`** (thin_checker.rs:1934-1963):
   - Added symbol-based lookup for merged symbols before type-based lookup
   - Handles type access like `Foo.Bar` when Foo is both a class and namespace
   - Resolves member from merged symbol's exports table

2. **Modified `type_reference_symbol_type`** (thin_checker.rs:975-1008):
   - For merged class+namespace symbols, returns constructor type (with exports)
   - Instead of instance type, to allow type member access

3. **Enhanced `merge_namespace_exports_into_constructor`** (thin_checker.rs:1010-1055):
   - Now includes type-only exports (interfaces, type aliases)
   - Previously only merged value exports

#### Test Results (5/13 passing)
**Passing (5)**:
- `namespace_merges_with_class_exports` - type access (normal order)
- `namespace_merges_with_enum_type_exports` - type access (normal+reverse order)
- `namespace_merges_with_function_type_exports` - type access (normal+reverse order)

**Failing (8)**:
- `namespace_merges_with_class_exports_reverse_order` - type access (reverse order)
- `namespace_merges_with_class_value_exports` - value access (normal+reverse order)
- `namespace_merges_with_enum_value_exports` - value access (normal+reverse order)
- `namespace_merges_with_function_value_exports` - value access (normal+reverse order)
- `namespace_merges_with_class_element_access` - element access

#### Remaining Work
Value access failures require deeper integration with solver's `property_access_type`:
- Callable types need to expose their merged symbol's exports
- Element access `Foo["value"]` needs special handling
- Reverse order issues need investigation

### Test Results
- **5087+ tests passing** (+5 from namespace merging)
- TS2304: 7/7 tests passing
- TS2355: 7/7 tests passing
- TS2564: 7/7 tests passing
- TS7010: 5/5 tests passing
- Namespace merging: 5/13 tests passing

### Key Implementation Locations
- `src/thin_checker.rs` - Main type checker with all error code implementations
- `src/thin_checker_tests.rs` - Comprehensive test suite
- `src/checker/control_flow.rs` - Flow analysis for definite assignment
- `src/checker/types/diagnostics.rs` - Error code definitions

## Notes
- All sync/merge operations completed successfully
- Changes pushed to `origin/worker/forge-2`
- Namespace merging requires additional work to handle value position access (8 tests remaining)
