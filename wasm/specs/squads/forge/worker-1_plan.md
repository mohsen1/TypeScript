# Worker 1 Plan - Squad Forge

## Mission
Fix TS2339: Property Does Not Exist

Status: Active
Priority: P1 (HIGH)

## Current Assignment
**Fix TS2339 False Positives: Property does not exist (35 extra errors)**

### Background
TypeScript is emitting TS2339 "Property does not exist" errors too aggressively. Investigation revealed two main issues:

1. **Private static members** - After WASM rebuild, private static members ARE being added to class constructor types (type correctly shows `#field: number`), but property lookup fails in destructuring contexts.

2. **Mixin classes** - Instance types missing base class properties (not yet investigated).

### Investigation Findings

#### Private Static Members
- Private static members ARE collected in `get_class_constructor_type()` (lines 4136-4176)
- The type correctly shows the property: `{ new (): { ... }; #field: number; ... }`
- **Root Issue**: Property access resolution fails even though property exists in type
- The atom comparison `prop.name == prop_atom` in solver fails
- Occurs in destructuring assignment contexts: `({ x: A.#field, y } = ...)`

#### Key Code Locations
- `src/thin_checker.rs:4038-4470` - `get_class_constructor_type()` collects static members
- `src/thin_checker.rs:8458-8504` - `get_property_name()` handles private identifiers
- `src/solver/operations.rs:1784-1803` - Callable property access lookup
- `src/solver/db.rs:248-254` - `property_access_type()` entry point

### Implementation Steps
1. [x] Sync from origin/rust
2. [x] Search for TS2339 emission code in `src/thin_checker.rs`
3. [x] Investigate type narrowing logic for property access
4. [x] Identify root cause: property lookup fails despite property existing in type
5. [ ] Fix atom comparison issue in property access resolution
6. [ ] Test with various property access patterns
7. [ ] Run conformance to verify reduction in extra errors

### Investigation Results

**Confirmed**: Private static members ARE being added to class constructor types correctly.

Evidence from differential test after WASM rebuild:
```
Property '#field' does not exist on type '{ new (): { ... }; #field: number; ... }'
```
The type display shows `#field: number` is present, but property lookup still fails.

**Root Cause**: Property access resolution fails despite property existing in type.

**Specific Issue Location**:
- File: `src/thin_checker.rs`
- Function: `get_type_of_private_property_access` (line 7086-7215)
- Issue: At line 7124, `property_access_type` is called but returns `PropertyNotFound` even though property exists in type
- The type passed to `property_access_type` includes `#field: number` in its properties list
- But the solver's property lookup in `src/solver/operations.rs:1788` fails: `if prop.name == prop_atom`

**Code Flow**:
1. When accessing `A.#field`, code detects private identifier (line 6920-6922)
2. Calls `get_type_of_private_property_access`
3. Calls `resolve_private_identifier_symbols` which may return empty vector
4. When empty, calls `property_access_type(object_type_for_check, &property_name)` at line 7124
5. Solver looks up property but fails to find it despite it being in the type
6. Returns `PropertyNotFound`, causing TS2339 error at line 7131

**Hypothesis**: The atom comparison `prop.name == prop_atom` fails because:
- Different strings are being interned during type construction vs property access
- Or the properties list in the callable shape is different from what's displayed
- Or there's a type resolution issue causing a different type to be checked

**Test Results**:
- `cargo run`: Does NOT emit TS2339 for simple cases
- WASM package: DOES emit TS2339 in destructuring contexts
- Suggests there may be a code path difference between cargo and WASM

### Remaining Work
- Fix atom comparison issue in solver property access resolution
- Debug why `prop.name == prop_atom` fails when both represent `#field`
- Investigate potential difference between cargo and WASM behavior
- Investigate mixin class instance types
- Verify no regressions in existing tests

### Fix Strategy

**Option 1: Debug the atom mismatch**
1. Add logging to `get_class_constructor_type` to see exact strings being interned
2. Add logging to `property_access_type` to see exact strings being looked up
3. Compare the two to find the difference
4. Fix the root cause of the mismatch

**Option 2: Workaround in `get_type_of_private_property_access`**
If the atom comparison cannot be easily fixed, add a fallback:
- When `property_access_type` returns `PropertyNotFound` but the type shows the property exists
- Return `Success` with the property type from the callable shape
- This would require iterating through the properties list manually

**Option 3: Fix `resolve_private_identifier_symbols`**
Ensure that static private members are found in the class scope even when accessed from outside:
- Check if the property name exists in static members of the class
- Return the symbol if found
- This would prevent falling through to `property_access_type`

### Files to Modify
- `src/thin_checker.rs` - `get_type_of_private_property_access` function
- `src/solver/operations.rs` - `resolve_property_access_inner` function
- `src/solver/db.rs` - `property_access_type` function

### Key Code Locations
- `src/thin_checker.rs` - property access checking, TS2339 emission
- `src/checker/control_flow.rs` - type narrowing
- `src/checker/types/diagnostics.rs` - TS2339 error code

### Test Cases
```typescript
// Should NOT emit TS2339 (type narrowing)
interface Foo { x: number; }
interface Bar { y: string; }

function foo(obj: Foo | Bar) {
  if ('x' in obj) {
    console.log(obj.x); // OK - narrowed to Foo
  }
}

// Should emit TS2339
const obj = { x: 1 };
console.log(obj.y); // Error: Property 'y' does not exist
```

## Completed
- [x] Fix Method Bivariance - Merged to squad/forge
- [x] TS7010/TS7011 - Implemented and pushed to origin/worker/forge-1

## Ready for Merge
No

## Notes
- Follow `wasm/specs/WASM_ARCHITECTURE.md`
- Use Docker for Rust tests: `./wasm/test.sh`
- Commit format: `[wasm] checker: Fix TS2339 false positives in property access`
- Sync before each task: `git fetch origin && git merge origin/rust --no-edit`
- Push to: `origin/worker/forge-1`
