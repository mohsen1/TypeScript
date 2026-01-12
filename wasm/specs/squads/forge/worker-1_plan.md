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
- The atom comparison `prop.name == prop_atom` in `src/solver/operations.rs:1788` fails
- Occurs specifically in destructuring assignment contexts: `({ x: A.#field, y } = ...)`
- NOT an issue with type construction or `get_property_name`

**Test Results**:
- Running via `cargo run` does NOT emit TS2339 for simple destructuring cases
- Differential test using WASM package DOES emit TS2339
- Suggests there may be a difference between cargo and WASM code paths

### Remaining Work
- Fix atom comparison issue in solver property access resolution
- Debug why `prop.name == prop_atom` fails when both represent `#field`
- Investigate potential difference between cargo and WASM behavior
- Investigate mixin class instance types
- Verify no regressions in existing tests

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
