# Worker 3 Plan - Squad Anvil

## Mission
Fix ES5 Private Accessors - Part 2: Emission

Status: Active
Priority: P1 (High)

## Current Assignment
**Implement ES5 Private Accessor Transform - Phase 2 (3 failing tests)**

### Background
Continuation of Worker 2's work. This handles the actual emission of private accessor code.

### Failing Tests
1. `test_parity_es5_private_accessor_getter`
2. `test_parity_es5_private_accessor_pair`
3. `test_parity_es5_private_accessor_setter`

### Expected Output Pattern
```javascript
// Input:
class Person { get #privateName() { return "value"; } }

// Output:
var _Person_privateName_get;
var Person = (function () {
    function Person() {
        _Person_privateName_get.set(this, function () { return "value"; });
    }
    return Person;
}());
_Person_privateName_get = new WeakMap();
```

### Implementation Steps

1. [ ] Wait for Worker 2 to complete collection phase (or work in parallel on interfaces)
2. [ ] Modify `emit_constructor_body` in `src/transforms/class_es5.rs`:
   ```rust
   // For each private accessor
   for acc in &state.private_accessors {
       if let Some(get_var) = &acc.get_var_name {
           // Emit: _get_var.set(this, function() { ...body... });
           writer.write(get_var);
           writer.write(".set(this, ");
           self.emit_function_expression(acc.getter_body_idx);
           writer.write(");");
       }
       // Same for setter
   }
   ```
3. [ ] Modify `emit_class_epilogue` to emit WeakMap initializations:
   ```rust
   for acc in &state.private_accessors {
       if let Some(get_var) = &acc.get_var_name {
           writer.write(&format!("{} = new WeakMap();", get_var));
       }
       // Same for setter
   }
   ```
4. [ ] Transform property access for private accessors:
   - `this.#name` (read) -> `__classPrivateFieldGet(this, _Class_name_get, "a")`
   - `this.#name = x` (write) -> `__classPrivateFieldSet(this, _Class_name_set, x, "a")`
5. [ ] Test: `./wasm/test.sh 2>&1 | grep -E "private_accessor"`

### Key Code Locations
- `src/transforms/class_es5.rs` - class transformation
- `src/transforms/private_fields_es5.rs` - `__classPrivateFieldGet/Set` usage

### The "a" Flag
The `"a"` flag in `__classPrivateFieldGet(obj, map, "a")` tells the helper this is an accessor (call the function) vs a field (return the value directly).

## Task Queue
- [ ] After emission: help with parser error recovery if time

## Completed
- [x] (Move finished items here)

## Ready for Merge
No

## Notes
- Follow `wasm/specs/WASM_ARCHITECTURE.md`
- Use Docker for Rust tests: `./wasm/test.sh`
- Commit format: `[wasm] transforms: Emit ES5 private accessors with WeakMap`
- Sync before each task: `git fetch origin && git merge origin/rust --no-edit`
- Push to: `origin/worker/anvil-3`
