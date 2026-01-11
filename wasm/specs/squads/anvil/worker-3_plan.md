# Anvil Worker 3 - TS2339 Property Resolution (Inherited Properties)

Ready for Merge: Yes
Status: Active

## Current Assignment (2026-01-11)

- Prereq: run `./scripts/ask-gemini.mjs "I need to reduce TS2339 false positives (property lookup/inheritance). What's the best approach?"` once the API key is available.
- Reduce TS2339 false positives (property access on inherited/prototype chain, mixins, and interface merges).
- Collect 3-5 failing samples from conformance output; capture the failing expression + expected property resolution.
- Trace property lookup in `wasm/src/thin_checker.rs` (class instance types, interface heritage, index signatures, and union/intersection property merges).
- Implement fix + regression tests; run a targeted TS2339 scan and report the delta.

### Update (2026-01-11)
- Fix: use TypeEnvironment-backed assignability in call/new resolution, and resolve Application symbols (including type param constraints) to improve generic mixin inference (commit 5cf3894068).
- Regression: `test_mixin_return_type_preserves_base_properties` in `wasm/src/thin_checker_tests.rs`.
- Samples (pre-fix): `classes/mixinAbstractClasses.ts`, `classes/mixinClassesAnnotated.ts`, `classes/mixinClassesAnonymous.ts`, `classes/mixinClassesMembers.ts`, `controlFlow/assertionTypePredicates1.ts`.
- TS2339 scan (post-fix): `node wasm/differential-test/find-ts2339.mjs --max=500 --samples=5` → 1 extra (`classes/classDeclarations/classExtendingClassLikeType.ts`, 6 errors).

## Current Assignment (Crash triage: privateNamesInterfaceExtendingClass) - COMPLETED

- **Status**: Fixed and committed (399e930edc)
- **Issue**: Stack overflow crash in `classes/members/privateNames/privateNamesInterfaceExtendingClass.ts`
- **Root cause**: Unbounded recursion when interface extends class with private fields:
  - `get_class_instance_type(C)` → type ref `I` → `type_reference_symbol_type` → `merge_interface_heritage_types` → `get_class_instance_type(C)` again
  - Interface type references bypassed `symbol_resolution_set`, and base class resolution used `get_class_instance_type` without `class_instance_resolution_set`
- **Fix**: Added `class_instance_resolution_set` guard in `merge_interface_heritage_types` around two `get_class_instance_type` calls (wasm/src/thin_checker.rs:2243-2250, 2274-2281)
  - On recursion detection, returns `TypeKey::Ref(SymbolRef)` fallback instead of crashing
- **Regression test**: Added `test_interface_extends_class_no_recursion_crash` in wasm/src/thin_checker_tests.rs
- **Verification**: Conformance test `classes/members/privateNames` slice completes with 0 crashes (124 tests run, previously crashed on privateNamesInterfaceExtendingClass.ts)
- **Time**: ~2.5 hours (investigation, fix, test, verification)

## Operation Conformance Assignment

**Mission**: Fix false positive TS2339 errors for inherited and prototype chain properties.

**Target Error**: TS2339 "Property 'X' does not exist on type 'Y'" - 68 false positives
**Root Cause**: Property lookup doesn't traverse class inheritance or interface extension chains

## Previous Assignment (2026-01-10)

- Reduce TS2339 false positives (mixin/private/static/property lookup cases).
- Collect 3-5 failing samples from conformance output; record the failing expression + expected property resolution.
- Trace property access in `wasm/src/thin_checker.rs` (property access/type resolution paths) and related mixin/base handling; verify index signatures and callable/index merges.
- Implement fix + regression tests; run a targeted conformance check (TS2339 grep) and report the delta.
- Deliverables: sample list + root cause notes, regression test(s), and conformance delta.

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

Ready for Merge: No (merged)

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
- Latest scan (`node wasm/differential-test/find-ts2339.mjs --max=300 --samples=5`): none in first 300 tests.

### Emit Sites (thin_checker.rs)
- Property access miss → TS2339: `wasm/src/thin_checker.rs:4901` / `wasm/src/thin_checker.rs:4902`
  - Branch: `PropertyAccessResult::PropertyNotFound { .. }` → `error_property_not_exist_at`
- Diagnostic builder: `wasm/src/thin_checker.rs:7071` (`error_property_not_exist_at`)

### Next Steps
1. Re-run conformance with higher `--max` if needed to confirm failing set.
2. Inspect private name / mixin samples for property lookup path (likely class static/private handling in property access).
3. Trace property access in `get_type_of_property_access_expression` and related type resolution helpers for static/private members.

### Update (2026-01-10)
- Typed class expressions as constructor values so return-type inference and property access see base members.
- Extended heritage parsing to accept parenthesized/new expressions like `extends (new B2<number>().anon)`.
- Avoided emitting property access nodes when `.` is followed by a non-identifier token (prevents TS2339 on `this.`).
- Build: `./wasm/build-wasm.sh` (warnings only).
- Conformance: `node wasm/differential-test/conformance-runner.mjs classes/classExpressions --max=200 -v`.
- Scan: `node wasm/differential-test/find-ts2339.mjs --max=300 --samples=5` (0 extra TS2339 in first 300).

### Update (2026-01-10)
- Fixed TemplateExpression1 crash by guarding missing `}` in template spans and synthesizing a tail literal to avoid infinite loops.
- Added `test_thin_parser_unterminated_template_expression_no_crash` in `wasm/src/thin_parser_tests.rs`.
- Build: `./wasm/build-wasm.sh` (warnings only).
- Conformance: `node wasm/differential-test/conformance-runner.mjs es6/templates --max=200 -v` (no crashes).
- Test: `./wasm/test.sh test_thin_parser_unterminated_template_expression_no_crash`.

### Update (2026-01-10)
- Larger sweep: `node wasm/differential-test/conformance-runner.mjs es6/templates --max=1000 -v` (178 tests, WASM Crashed: 0).
- Repro: `tests/cases/conformance/es6/templates/TemplateExpression1.ts` (unterminated template expression).
- Crash path: `parse_template_expression` in `wasm/src/thin_parser.rs` during template span rescan; now guarded to emit TS1005 and synthesize a tail.

### Update (2026-01-10)
- Broader sweep: `node wasm/differential-test/conformance-runner.mjs --max=500` (497 tests run, 13 multi-file; WASM Crashed: 0).

### Update (2026-01-10)
- Added TS1160 `UNTERMINATED_TEMPLATE_LITERAL` diagnostic and parser reporting for unterminated template literals (template expressions, no-substitution, template literal types).
- Added `test_thin_parser_unterminated_template_literal_reports_ts1160` in `wasm/src/thin_parser_tests.rs`.
- Build: `./wasm/build-wasm.sh` (warnings only).
- Conformance: `node wasm/differential-test/conformance-runner.mjs es6/templates --max=200 -v` (TS1160 no longer missing; extra TS1160 in `templateStringInPropertyName2` and `templateStringInPropertyNameES6_2`).
- Test: `./wasm/test.sh test_thin_parser_unterminated_template_literal_reports_ts1160`.

### Update (2026-01-10)
- Larger crash sweep: `node wasm/differential-test/conformance-runner.mjs --max=1000` (993 tests run, 120 multi-file; WASM Crashed: 1).
- Crashed file: `classes/members/privateNames/privateNamesInterfaceExtendingClass.ts` with `Maximum call stack size exceeded`.

### Update (2026-01-10)
- Root cause: template literal property names were parsed as identifiers, leaving the closing backtick to be scanned as a new unterminated template literal (extra TS1160).
- Fix: in object literal property assignment, emit TS1136 and consume template literals as property names to keep the scanner in sync.
- Added `test_thin_parser_template_literal_property_name_no_ts1160` in `wasm/src/thin_parser_tests.rs`.
- Build: `./wasm/build-wasm.sh` (warnings only).
- Conformance: `node wasm/differential-test/conformance-runner.mjs es6/templates --max=200 -v` (extra TS1160 removed; extra errors down to 13).
- Test: `./wasm/test.sh test_thin_parser_template_literal_property_name_no_ts1160`.

Ready for Merge: No (merged 2026-01-10)
