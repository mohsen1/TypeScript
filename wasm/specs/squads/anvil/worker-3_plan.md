# Worker 3 Plan - Squad Anvil

## Mission
Fix ES5 Private Accessors - Part 2: Emission

Status: Complete
Priority: P1 (High)

## Current Assignment
**[COMPLETED] Implement ES5 Private Accessor Transform - Phase 2**

### Background
Continuation of Worker 2's work. This handles the actual emission of private accessor code.

### Implementation Completed

1. [x] Added `PrivateAccessorInfo` struct to track private accessor data
2. [x] Added `collect_private_accessors()` function in `private_fields_es5.rs`
3. [x] Modified `ClassES5Emitter` to include `private_accessors` field
4. [x] Modified `emit_constructor_body` to emit WeakMap.set() calls for accessors
5. [x] Modified `emit_class_epilogue` to emit WeakMap initializations
6. [x] Skip private accessors from being emitted as regular accessors in `emit_methods` and `emit_static_members`

### Test Results
All 7 private accessor parity tests now pass:
- test_parity_es5_private_accessor_getter
- test_parity_es5_private_accessor_setter
- test_parity_es5_private_accessor_pair
- test_parity_es5_private_accessor_static
- test_parity_es5_private_accessor_complex
- test_parity_es5_private_accessor_computed_values
- test_parity_es5_private_accessor_validation

### Key Code Locations
- `src/transforms/class_es5.rs` - class transformation (added 122 lines)
- `src/transforms/private_fields_es5.rs` - `PrivateAccessorInfo` and collection function (added 95 lines)

### The "a" Flag
The `"a"` flag in `__classPrivateFieldGet(obj, map, "a")` tells the helper this is an accessor (call the function) vs a field (return the value directly).

## Task Queue
- [ ] Help with parser error recovery if time

## Completed
- [x] Implement ES5 Private Accessor Transform - Phase 2 (Emission)

## Ready for Merge
Yes

## Notes
- Follow `wasm/specs/WASM_ARCHITECTURE.md`
- Use Docker for Rust tests: `./wasm/test.sh`
- Commit format: `[wasm] transforms: Implement ES5 private accessor emission`
- Sync before each task: `git fetch origin && git merge origin/rust --no-edit`
- Push to: `origin/worker/anvil-3`
