# Worker 2 Plan - Squad Anvil

## Mission
Fix ES5 Private Accessors - Part 1: Data structures and collection

Status: Collection Complete, Awaiting Worker 3 Emission
Priority: P1 (High)

## Current Assignment
**[COMPLETE] Implement ES5 Private Accessor Transform - Phase 1 (3 failing tests)**

### Background
Private getters/setters (`get #name()`, `set #name()`) need WeakMap-based emission like private fields but with accessor-specific logic.

### Failing Tests
1. `test_parity_es5_private_accessor_getter`
2. `test_parity_es5_private_accessor_pair`
3. `test_parity_es5_private_accessor_setter`

### Your Focus: Collection Phase
Worker 2 handles the data collection. Worker 3 handles emission.

### Implementation Steps

1. [x] Read failing tests in `src/emitter_parity_tests.rs`
2. [x] Add `PrivateAccessorInfo` struct to `src/transforms/class_es5.rs`:
   ```rust
   struct PrivateAccessorInfo {
       name: String,          // Without '#' prefix
       is_static: bool,
       get_var_name: Option<String>,  // e.g., "_Person_name_get"
       set_var_name: Option<String>,  // e.g., "_Person_name_set"
       getter_body_idx: Option<NodeIndex>,
       setter_body_idx: Option<NodeIndex>,
   }
   ```
3. [x] Add `private_accessors: Vec<PrivateAccessorInfo>` to `ClassES5Emitter`
4. [x] In `emit_class_internal`, scan members for private accessors:
   - Check for `GetAccessor`/`SetAccessor` with `PrivateIdentifier` name
   - Create `PrivateAccessorInfo` entries
   - Generate unique variable names: `_{ClassName}_{fieldName}_{get|set}`
5. [x] Coordinate with Worker 3 on emission interface

### Helper Function
```rust
fn get_private_accessor_var_name(class_name: &str, member_name: &str, is_getter: bool) -> String {
    let clean_name = member_name.trim_start_matches('#');
    let suffix = if is_getter { "get" } else { "set" };
    format!("_{}_{}{}", class_name, clean_name, suffix)
}
```

### Key Code Locations
- `src/transforms/class_es5.rs` - class transformation
- `src/transforms/private_fields_es5.rs` - private field patterns (reference)

## Task Queue
- [ ] After Worker 3 implements emission: verify tests pass

## Completed
- [x] Add PrivateAccessorInfo struct to class_es5.rs
- [x] Add collect_private_accessors method that scans GetAccessor/SetAccessor nodes
- [x] Add private_accessors field to ClassES5Emitter
- [x] Call collect_private_accessors in emit_class_internal
- [x] Generate variable names using pattern: _ClassName_fieldName_get/set
- [x] Use HashMap to merge getter/setter pairs into single entries

## Ready for Merge
Yes

## Notes
- Follow `wasm/specs/WASM_ARCHITECTURE.md`
- Use Docker for Rust tests: `./wasm/test.sh`
- Commit format: `[wasm] transforms: Add private accessor collection in class_es5`
- Sync before each task: `git fetch origin && git merge origin/rust --no-edit`
- Push to: `origin/worker/anvil-2`
- Collection phase complete. Worker 3 needs to implement emission for tests to pass.
