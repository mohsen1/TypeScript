
# Migration Plan: TypeScript Compiler → Rust via WebAssembly (Checker)

## Vision

Incrementally rewrite the TypeScript compiler in Rust, compiled to WebAssembly
for seamless Node.js/browser interop. **Beat TypeScript-Go in performance.**

# Files

src/solver/, src/thin_checker.rs, src/checker/ (legacy removal).

# Goal

Pass tests/cases/compiler.
## Tasks

Our focus is to make wasm checker complete

- ✅ Fix the Solver Stack Overflow Risk (COMPLETED)
    Added depth tracking to SubtypeChecker:
    - Added `depth: u32` field to struct
    - Initialize to 0 in both constructors
    - Check `depth > 100` after fast paths, return Provisional
    - Increment before recursion, decrement after
    - All tests pass (593/593)
- 🔄 Move expression type computation to solver/operations.rs (incremental)
- 🔄 Use NodeView API instead of raw arena lookups (incremental)
- ⬜ Deprecate checker/types in favor of solver/types
- ✅ Symbol type checking (errors 2403, 2554) (COMPLETED)
    - Synthesized Symbol constructor type with call signature: `Symbol(description?: string | number): symbol`
    - Implemented variable redeclaration checking (TS2403) for same-scope var declarations
    - Added Symbol property access handling (description, toString, valueOf)
    - Added 7 comprehensive test cases
    - All 614 tests pass
- ✅ Property access from index signature (error 4111) (COMPLETED)
    - Modified PropertyAccessResult to track whether property was resolved via index signature
    - Updated all property resolution paths to propagate from_index_signature flag
    - Implemented error check in get_type_of_property_access for dot notation access
    - Flag is contagious across union members (TypeScript strict behavior)
    - Added 4 comprehensive test cases (1 active solver-level test, 3 integration tests documented but disabled until interface type lowering is implemented)
    - All 614 tests pass
- ✅ Ambient module patterns (errors 5061, 2819) (COMPLETED)
    - Implemented TS5061: Ambient module declaration cannot specify relative module name
    - Implemented TS2819: Private identifiers not allowed in ambient classes
    - Fixed parser bug: parse_ambient_declaration now creates parse_declare_module with declare modifier
    - Added has_modifier helper to CheckerContext for checking modifier presence
    - Added comprehensive checks in DeclarationChecker for module declarations
    - Added checks in ThinCheckerState for private identifiers in ambient classes
    - Added 5 comprehensive test cases
    - All 623 tests pass
    - Note: TS2305 (Module has no exported member) not yet implemented - requires module resolution system
- ✅ Fix critical bug: Missing top-level scope (CRITICAL - COMPLETED)
    - Fixed: check_source_file now pushes/pops a file-level scope
    - Enables top-level variable redeclaration checking (TS2403)
    - Enables type tracking for top-level variables in flow analysis
    - Added 2 test cases for var redeclaration at file level
    - All 625 tests: 623 passed (2 pre-existing failures unrelated to this fix)
    - Addresses critical issue identified by Gemini code review
- ⬜ Various missing error codes (see test failures)
- ✅ Fix tuple subtyping logic (CRITICAL - COMPLETED)
    - Fixed: Now properly rejects `[number, string]` as subtype of `[number]`
    - Checks if target has rest element before allowing extra source elements
    - Handles rest element matching correctly (rest to rest, fixed to rest)
    - Added 5 comprehensive test cases for edge cases
    - All 598 tests pass
- ✅ Fix function parameter variance (MAJOR - COMPLETED)
    - Implemented contravariant parameter checking (strict/sound behavior)
    - Added `strict_function_types: bool` field (default: true)
    - Created `are_parameters_compatible` helper method
    - Updated all function/call signature checking methods
    - Strict mode: target <: source (contravariant, sound)
    - Legacy mode: target <: source OR source <: target (bivariant, unsound)
    - Added comprehensive test for both modes
    - All 607 tests pass (no existing tests broken!)
- ✅ Remove unused ref_cache field (MINOR - COMPLETED)
    - Removed unused field from SubtypeChecker struct
    - Removed from both constructors (new and with_resolver)
    - All 606 tests still pass
- ✅ Fix tuple to array subtyping for rest elements (BLOCKER - COMPLETED)
    - Fixed: Rest elements now properly unwrapped before comparison
    - `[string, ...string[]]` is now assignable to `string[]`
    - Uses get_array_element_type to unwrap rest array type
    - Added 4 comprehensive test cases
    - All 602 tests pass
- ✅ Fix number index signature check (CRITICAL - COMPLETED)
    - Fixed: check_object_to_indexed now validates numeric properties
    - Parses property names with `parse::<f64>()` to detect numeric keys
    - Numeric properties validated against `number_index` signature
    - All properties still validated against `string_index` (TypeScript semantics)
    - Added 4 comprehensive test cases
    - All 606 tests pass
- ... add more tasks (Ask Gemini when needed)



### Current Status (12,408 tests)
| Baseline | Compiler (100 sample) | Conformance | Crash Rate |
|----------|----------------------|-------------|------------|
| .errors.txt | **81.8%** (63/77 subset) | 33.8% (1,741/5,157) | 0.05% |
| .js emit | **60.5%** (46/76 subset) | ~3% | 0.05% |


## Quick Reference

```bash
# Tests (Docker)
./wasm/test.sh

# Baseline comparison
node scripts/baseline-test-rust.mjs

# Build WASM
./wasm/build-wasm.sh
```
