# Anvil Worker 3 - TS2339 Property Resolution (Inherited Properties)

## Operation Conformance Assignment

**Mission**: Fix false positive TS2339 errors for inherited and prototype chain properties.

**Target Error**: TS2339 "Property 'X' does not exist on type 'Y'" - 68 false positives
**Root Cause**: Property lookup doesn't traverse class inheritance or interface extension chains

## Problem Analysis

The WASM checker incorrectly reports TS2339 for valid code:

```typescript
class Base {
  name: string;
}
class Derived extends Base {
  greet() {
    console.log(this.name); // FALSE POSITIVE: TS2339
  }
}
```

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
- [ ] TS2339 false positives drop by 50+ occurrences

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
- **NEVER edit**: `STRUCTURE.md`, `GOALS.md`, other workers' plan files, or anything in `orchestrator/`
 - Implemented in `wasm/src/thin_checker.rs`, tests in `wasm/src/thin_checker_tests.rs`
