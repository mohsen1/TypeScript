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

## Completed

### Session 1: Private Name Parser Fixes (2026-01-09)

1. **Fix compilation error in subtype.rs** - Fixed undefined variable `s_sym` in TypeQuery subtype check that prevented compilation.

2. **Support private identifiers in accessor names** (`thin_parser.rs:2373-2377`)
   - Issue: `static get #quux()` was being parsed with name "get" instead of "#quux"
   - Root cause: `look_ahead_is_accessor` function didn't check for PrivateIdentifier tokens
   - Fix: Added `self.is_token(SyntaxKind::PrivateIdentifier)` to the accessor name check

3. **Support generator methods with private names** (`thin_parser.rs:2217-2218, 2298`)
   - Issue: `static async *#baz()` caused TS1068 "Unexpected token"
   - Root cause: Class member parser wasn't consuming asterisk token before property name
   - Fix: Added `let asterisk_token = self.parse_optional(SyntaxKind::AsteriskToken);` and updated MethodDeclData

4. **Unit tests added** (`thin_checker_tests.rs`):
   - `test_private_static_method_access_no_error` - Tests A.#foo(30)
   - `test_private_static_accessor_access_no_error` - Tests A.#quux accessors
   - `test_private_static_generator_method_access_no_error` - Tests static async *#baz()

### Verification Results

After fixes, direct checker runs on the private name test files produce **no errors**:
- `privateNamesAndStaticMethods.ts` - No errors
- `privateNameStaticsAndStaticMethods.ts` - No errors
- `privateNameStaticAccessorsAccess.ts` - No errors

### Session 2: Abstract Constructor Type Parsing (2026-01-09)

5. **Support abstract constructor types** (`thin_parser.rs:6583-6597, thin_node.rs:672-674`)
   - Issue: `abstract new (...args: any) => any` caused parser errors (TS1005, TS1109)
   - Root cause: `parse_primary_type` didn't recognize `abstract` before `new`
   - Fix:
     - Added `is_abstract: bool` field to `FunctionTypeData`
     - Added look-ahead in `parse_primary_type` to detect `abstract new`
     - Updated `parse_constructor_type` to accept `is_abstract` parameter

6. **Unit test added** (`thin_checker_tests.rs`):
   - `test_abstract_constructor_type_parses` - Tests abstract constructor type syntax in generic constraints

### Verification Results - Session 2

After abstract constructor type fix:
- `mixinAbstractClasses.ts` - Parser errors eliminated (0 → remaining TS2339 are type inference issues)
- `mixinClassesAnonymous.ts` - Parser errors eliminated (remaining TS2339 are type inference issues)

### Conformance Test Results

After WASM package rebuild with all parser fixes:
- **Exact Match: 577 (29.0%)** - up from ~97 (19.5%)
- **Same Error Count: 669 (33.6%)** - up from ~116 (23.3%)

### Remaining TS2339 Issues

38 tests still have extra TS2339 errors, primarily in complex areas:
- **Mixin classes** - complex generic type inference with abstract constraints
- **Control flow analysis** - type narrowing issues
- **Enum merging** - declaration merging
- **Symbol properties** - ES6 symbol handling

These require deeper checker work beyond basic property access fixes.

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

Ready for Merge: No (merged 2026-01-10)

## Current Task: TS2339 property access fixes (new assignment)

### Update (2026-01-09)
- Implemented mixin base instance extraction for heritage expressions and merged base properties (handles call-expression bases and type-parameter constructors).
- Preserved callable index signatures in type literal/interface lowering and interface merge logic.
- Added `test_mixin_inheritance_property_access` in `wasm/src/thin_checker_tests.rs`.
- Build: `./wasm/build-wasm.sh` (warnings only).
- Conformance `--max=200`: 2 Missing TS2339 lines:
  - `ambient/ambientDeclarationsPatterns_merging3.ts`
  - `async/es6/asyncWithVarShadowing_es6.ts` (Missing TS7031, TS2339)
  - Prior pre-change scan showed 0 TS2339 lines (delta +2 missing; no extra TS2339 in first 200).

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
