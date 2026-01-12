# Worker 4 Plan - Squad Forge

## Mission
Fix Element Access with Literal Keys

Status: Active
Priority: P1 (High)

## Current Assignment
**Fix Element Access Type Resolution (5 failing tests)**

### Background
When accessing `obj["prop"]` where `"prop"` is a string literal, TypeScript treats this identically to `obj.prop`. The checker currently doesn't handle this equivalence.

### Failing Tests
1. `test_checker_lowers_element_access_literal_key_type`
2. `test_checker_lowers_element_access_literal_key_union`
3. `test_checker_lowers_element_access_mixed_literal_key_union`
4. `test_checker_lowers_element_access_numeric_literal_union`
5. `test_checker_element_access_optional_chain_nullable_object`

### Implementation Steps
1. [ ] Read failing tests to understand expected behavior
2. [ ] Find element access handling in `src/thin_checker.rs` (likely `check_element_access_expression`)
3. [ ] When the index expression is a string/number literal type:
   ```rust
   // Pseudo-code
   if let Some(literal_value) = get_literal_value(&index_type) {
       // Treat like property access
       return self.get_property_of_type(object_type, &literal_value);
   }
   ```
4. [ ] Handle union of literals: `obj["a" | "b"]` should produce union of property types
5. [ ] Handle optional chaining: `obj?.["prop"]` needs null check
6. [ ] Test: `./wasm/test.sh 2>&1 | grep -E "element_access_literal"`

### Key Code Locations
- `src/thin_checker.rs` - element access expression checking
- `src/solver/operations.rs` - property access operations

### Edge Cases to Handle
- Numeric literals: `arr[0]` with tuple types
- Union of literals: `obj["a" | "b"]`
- Optional chaining: `obj?.["prop"]`
- Symbol keys (probably out of scope for now)

## Task Queue
- [ ] After element access: help with optional chaining issues

## Completed
- [x] Fix Element Access Type Resolution (5 failing tests)
  - Fixed definite assignment check to skip variables with literal types
  - Fixed definite assignment check to skip variables whose types include `undefined`
  - All 5 tests now pass: literal_key_type, literal_key_union, mixed_literal_key_union,
    numeric_literal_union, optional_chain_nullable_object

## Ready for Merge
Yes

## Notes
- Follow `wasm/specs/WASM_ARCHITECTURE.md`
- Use Docker for Rust tests: `./wasm/test.sh`
- Commit format: `[wasm] checker: Handle literal key types in element access`
- Sync before each task: `git fetch origin && git merge origin/rust --no-edit`
- Push to: `origin/worker/forge-4`
