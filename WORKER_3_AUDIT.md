# Worker-3 Audit: Invert Solver Defaults (Stop Being "Nice")

## Task: Change Default Return Type

### Summary
Fixed error paths in `thin_checker.rs` where `TypeId::ERROR` was being converted to `TypeId::ANY`, hiding type errors instead of exposing them.

## Changes Made

### File: `wasm/src/thin_checker.rs`

#### 1. Function Call Error Handling (Line ~7111-7133)
**Before:**
```rust
if callee_type == TypeId::ANY || callee_type == TypeId::ERROR {
    return TypeId::ANY;
}
```
**After:**
```rust
if callee_type == TypeId::ANY {
    return TypeId::ANY;
}
if callee_type == TypeId::ERROR {
    return TypeId::ERROR; // Return ERROR instead of ANY to expose type errors
}
```

#### 2. Optional Chaining with Error (Line ~7143-7148)
**Before:**
```rust
if callee_type == TypeId::ANY || callee_type == TypeId::ERROR {
    return TypeId::ANY;
}
```
**After:**
```rust
if callee_type == TypeId::ANY {
    return TypeId::ANY;
}
if callee_type == TypeId::ERROR {
    return TypeId::ERROR; // Return ERROR instead of ANY to expose type errors
}
```

#### 3. Constructor Type Check (Line ~7944-7949)
**Before:**
```rust
if constructor_type == TypeId::ANY || constructor_type == TypeId::ERROR {
    return TypeId::ANY;
}
```
**After:**
```rust
if constructor_type == TypeId::ANY {
    return TypeId::ANY;
}
if constructor_type == TypeId::ERROR {
    return TypeId::ERROR; // Return ERROR instead of ANY to expose type errors
}
```

#### 4. Empty Instance Types (Line ~7983-7984)
**Before:**
```rust
if instance_types.is_empty() {
    return TypeId::ANY;
}
```
**After:**
```rust
if instance_types.is_empty() {
    return TypeId::ERROR; // No construct signatures in intersection - expose error
}
```

#### 5. No Construct Type (Line ~7995-7997)
**Before:**
```rust
let Some(construct_type) = construct_type else {
    return TypeId::ANY;
};
```
**After:**
```rust
let Some(construct_type) = construct_type else {
    return TypeId::ERROR; // Return ERROR instead of ANY to expose type errors
};
```

#### 6. Property Access Object Type Check (Line ~8387-8392)
**Before:**
```rust
if object_type == TypeId::ANY || object_type == TypeId::ERROR {
    return TypeId::ANY;
}
```
**After:**
```rust
if object_type == TypeId::ANY {
    return TypeId::ANY;
}
if object_type == TypeId::ERROR {
    return TypeId::ERROR; // Return ERROR instead of ANY to expose type errors
}
```

#### 7. Resolved Property Access Object Type (Line ~8447-8452)
**Before:**
```rust
if object_type_for_access == TypeId::ANY || object_type_for_access == TypeId::ERROR {
    return TypeId::ANY;
}
```
**After:**
```rust
if object_type_for_access == TypeId::ANY {
    return TypeId::ANY;
}
if object_type_for_access == TypeId::ERROR {
    return TypeId::ERROR; // Return ERROR instead of ANY to expose type errors
}
```

#### 8. Index Signature Property Access (Line ~8715-8723)
**Before:**
```rust
if object_type_for_check == TypeId::ANY
    || object_type_for_check == TypeId::ERROR
    || object_type_for_check == TypeId::UNKNOWN
{
    return TypeId::ANY;
}
```
**After:**
```rust
if object_type_for_check == TypeId::ANY {
    return TypeId::ANY;
}
if object_type_for_check == TypeId::ERROR {
    return TypeId::ERROR; // Return ERROR instead of ANY to expose type errors
}
if object_type_for_check == TypeId::UNKNOWN {
    return TypeId::ANY; // UNKNOWN remains ANY for now (could be stricter)
}
```

#### 9. Element Access Index Signature (Line ~8920-8934)
**Before:**
```rust
if object_type == TypeId::ANY || object_type == TypeId::ERROR {
    return TypeId::ANY;
}
let object_type = self.resolve_type_for_property_access(object_type);
if object_type == TypeId::ANY || object_type == TypeId::ERROR {
    return TypeId::ANY;
}
```
**After:**
```rust
if object_type == TypeId::ANY {
    return TypeId::ANY;
}
if object_type == TypeId::ERROR {
    return TypeId::ERROR; // Return ERROR instead of ANY to expose type errors
}
let object_type = self.resolve_type_for_property_access(object_type);
if object_type == TypeId::ANY {
    return TypeId::ANY;
}
if object_type == TypeId::ERROR {
    return TypeId::ERROR; // Return ERROR instead of ANY to expose type errors
}
```

#### 10. Index Access on ObjectWithIndex (Line ~9141-9159)
**Before:**
```rust
if literal_index.is_some() {
    if let Some(number_index) = shape.number_index.as_ref() {
        return number_index.value_type;
    }
    if let Some(string_index) = shape.string_index.as_ref() {
        return string_index.value_type;
    }
    return TypeId::ANY;
}
if index_type == TypeId::NUMBER {
    if let Some(number_index) = shape.number_index.as_ref() {
        return number_index.value_type;
    }
    if let Some(string_index) = shape.string_index.as_ref() {
        return string_index.value_type;
    }
    return TypeId::ANY;
}
```
**After:**
```rust
if literal_index.is_some() {
    if let Some(number_index) = shape.number_index.as_ref() {
        return number_index.value_type;
    }
    if let Some(string_index) = shape.string_index.as_ref() {
        return string_index.value_type;
    }
    return TypeId::ERROR; // No matching index signature - expose error
}
if index_type == TypeId::NUMBER {
    if let Some(number_index) = shape.number_index.as_ref() {
        return number_index.value_type;
    }
    if let Some(string_index) = shape.string_index.as_ref() {
        return string_index.value_type;
    }
    return TypeId::ERROR; // No matching index signature - expose error
}
```

## Impact

### Expected Results
- **Missing TS2322 errors** (Type 'X' is not assignable to type 'Y'): Should decrease from 184
- **Missing TS7006 errors** (Parameter 'x' implicitly has 'any' type): Should decrease from 357
- **Temporary spike in "Extra Errors":** Expected and GOOD - exposes failing logic instead of hiding it

### Acceptance Criteria Met
✅ No function returns ANY on error paths (in thin_checker.rs)
✅ Error types propagate correctly
✅ Code compiles without errors
✅ Audit document created

## Notes
- This is a strategic change that improves reliability
- Coordinates with worker-2 (global scope) - many missing errors will fix once TS2304 is resolved
- May require coordination with EM-1 for larger merge strategy

## Testing
Run conformance tests to compare before/after error counts:
```bash
# Run tests to verify the changes
cargo test --lib

# Compare error output with TypeScript compiler
# The number of missing TS2322/TS7006 errors should decrease significantly
```
