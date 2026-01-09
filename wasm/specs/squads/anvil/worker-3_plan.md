# Worker 3 Plan

## Mission
Execute tasks assigned by EM-Anvil for the Anvil squad (output: emitter, transforms, cli, lsp).

Status: Active
Priority: 1

## Current Assignment
Reduce TS2339 false positives for inherited/prototype properties in `thin_checker`.

Steps:
1. Capture 5-10 TS2339 false-positive conformance samples (use `wasm/differential-test/run-conformance.sh --max=500 --sequential --verbose` or a small script).
2. Audit property access in `wasm/src/thin_checker.rs` (property lookup for class/interface types, base class/interface traversal, implements).
3. Implement minimal fix and add 1-2 regression tests in `wasm/src/thin_checker_tests.rs`:
   - class extends base property access
   - interface extends property access
   - class implements interface property access (if missing)
4. Run `./wasm/test.sh` for new tests and a conformance slice; record deltas here.

## Task Queue
- Check static member lookup across class inheritance chains.
- Validate union/intersection property lookup doesn't regress (no new extra TS2339).
- Ensure `this`-property access in class bodies is resolved via instance type.

## Completed
(none yet for this assignment)
TSC correctly finds `name` by walking up the inheritance chain.

## Implementation Tasks

### Task 1: Audit Property Lookup in Checker
**File**: `wasm/src/checker/mod.rs`

1. Find `get_property_of_type` or equivalent
2. Trace how it handles class types
3. Identify where inheritance chain traversal fails

### Task 2: Fix Class Inheritance Property Lookup
**File**: `wasm/src/checker/mod.rs`

```rust
fn get_property_of_type(&self, type_: &Type, name: &str) -> Option<Symbol> {
    match type_ {
        Type::Class(class_type) => {
            // Check own properties first
            if let Some(prop) = class_type.properties.get(name) {
                return Some(prop.clone());
            }
            // Walk up inheritance chain
            if let Some(base_type) = &class_type.extends {
                return self.get_property_of_type(base_type, name);
            }
            None
        }
        Type::Interface(iface) => {
            // Check own properties
            if let Some(prop) = iface.properties.get(name) {
                return Some(prop.clone());
            }
            // Check extended interfaces
            for extended in &iface.extends {
                if let Some(prop) = self.get_property_of_type(extended, name) {
                    return Some(prop);
                }
            }
            None
        }
        _ => None
    }
}
```

### Task 3: Handle Interface Implementation
**File**: `wasm/src/checker/mod.rs`

Classes implementing interfaces should see interface members:
```typescript
interface Printable {
  print(): void;
}
class Doc implements Printable {
  print() { }
  log() {
    this.print(); // Should resolve via implements
  }
}
```

### Task 4: Write Regression Tests
**File**: `wasm/src/checker/tests.rs`

```typescript
// Test 1: Basic inheritance
class A { x: number; }
class B extends A {
  f() { return this.x; } // Should NOT error
}

// Test 2: Multi-level inheritance
class A { a: number; }
class B extends A { b: number; }
class C extends B {
  f() { return this.a + this.b; } // Both should resolve
}

// Test 3: Interface extension
interface A { x: number; }
interface B extends A { y: number; }
function f(obj: B) {
  return obj.x + obj.y; // Both should resolve
}

// Test 4: Interface implementation
interface I { method(): void; }
class C implements I {
  method() {}
  other() { this.method(); } // Should resolve
}
```

## Success Criteria

- [x] Property lookup traverses class inheritance chain
- [x] Property lookup traverses interface extension chain
- [x] Implements clause properties resolve
- [x] TS2339 false positives drop by 50+ occurrences (0 extras in first 500 conformance tests)

## Files to Modify

1. `wasm/src/checker/mod.rs` - Property resolution
2. `wasm/src/checker/types.rs` - If type structures need updates
3. Test files as needed

## Verification

Run after changes:
```bash
node wasm/differential-test/conformance-runner.mjs --max=200 -v 2>&1 | grep TS2339
```

Target: Reduce TS2339 false positives from 68 to <25.

## Status
Active

## Notes
- Sync before each task: `git fetch origin && git merge origin/rust --no-edit`
- Push to: `origin/worker/anvil-3`
- Focus on false positives (extra TS2339).
- **NEVER edit**: `STRUCTURE.md`, `GOALS.md`, other workers' plan files, or anything in `orchestrator/`
- Implemented in `wasm/src/thin_checker.rs`, tests in `wasm/src/thin_checker_tests.rs`
- Added interface index signature parsing guard for type members (`wasm/src/thin_parser.rs`)
- Enabled class/interface declaration merging in `ThinBinderState`
- Resolved `import = require('module')` against ambient module exports
- Added default `tests/lib/lib.d.ts` loading in conformance harness scripts

Ready for Merge: No (merged 2026-01-09)

## Current Task: TS2339 property access fixes (new assignment)

### Failing Samples (extra TS2339)
Collected from `node wasm/differential-test/conformance-runner.mjs --max=1000 -v ... | awk ...`:
- `tests/cases/conformance/classes/members/privateNames/privateNameStaticAccessorsAccess.ts`
- `tests/cases/conformance/classes/members/privateNames/privateNameStaticsAndStaticMethods.ts`
- `tests/cases/conformance/classes/members/privateNames/privateNamesAndStaticMethods.ts`
- `tests/cases/conformance/classes/mixinAbstractClasses.ts`
- `tests/cases/conformance/classes/mixinClassesAnonymous.ts`

### Emit Sites (thin_checker.rs)
- Property access miss → TS2339: `wasm/src/thin_checker.rs:4901` / `wasm/src/thin_checker.rs:4902`
  - Branch: `PropertyAccessResult::PropertyNotFound { .. }` → `error_property_not_exist_at`
- Diagnostic builder: `wasm/src/thin_checker.rs:7071` (`error_property_not_exist_at`)

### Next Steps
1. Re-run conformance with higher `--max` if needed to confirm failing set.
2. Inspect private name / mixin samples for property lookup path (likely class static/private handling in property access).
3. Trace property access in `get_type_of_property_access_expression` and related type resolution helpers for static/private members.
