# Worker 4 Plan - Squad Forge

## Mission
Implement TS2341/TS2445 access modifier enforcement.

Status: Active
Priority: 1

## Current Assignment
Enforce private and protected access modifiers on property/method access.

**Error Codes:**
- TS2341 - "Property 'X' is private and only accessible within class 'Y'"
- TS2445 - "Property 'X' is protected and only accessible within class 'Y' and its subclasses"

**Impact:** 63 conformance tests affected

### Background
TypeScript enforces visibility modifiers:
```typescript
class Foo {
  private secret = 42;
  protected shared = "hello";
}

const f = new Foo();
f.secret;  // TS2341: private
f.shared;  // TS2445: protected
```

The WASM checker has `has_private_modifier()` and `has_protected_modifier()` functions but does NOT emit errors. The infrastructure exists but enforcement is missing.

### Steps
1. **Find existing infrastructure** in `thin_checker.rs`:
   - Search for `has_private_modifier`, `has_protected_modifier` (around line 7081, 7096)
   - Find property access checking code
   - Look for `check_property_access` or similar

2. **Add test cases first**:
   ```typescript
   // Should error: TS2341
   class Foo {
     private x = 1;
   }
   new Foo().x;  // error

   // Should error: TS2445
   class Bar {
     protected y = 2;
   }
   new Bar().y;  // error

   // Should NOT error: protected access in subclass
   class Base {
     protected z = 3;
   }
   class Derived extends Base {
     test() { return this.z; }  // OK
   }

   // Should NOT error: private access within class
   class Baz {
     private w = 4;
     getW() { return this.w; }  // OK
   }
   ```

3. **Implement enforcement**:
   - In property access checking, get the property's containing class
   - Check if property has private/protected modifier
   - Compare access location to property's class:
     - Private: must be same class
     - Protected: must be same class or subclass
   - Emit TS2341/TS2445 on violation

4. **Handle edge cases**:
   - Static members
   - Constructor parameters with modifiers
   - Private fields (`#field` syntax) - different from `private` keyword

5. **Run conformance tests** and report numbers.

### Key Files
- `wasm/src/thin_checker.rs` - `has_private_modifier()`, `has_protected_modifier()`, property access
- `wasm/src/checker/expr.rs` - expression checking

### Success Criteria
- TS2341 emitted for private access violations
- TS2445 emitted for protected access violations
- No errors for valid access within class/subclass

## Task Queue
(empty - single focused task)

## Completed

- [x] Fixed method type parameter scope in check_method_declaration
- [x] Added solver coverage for all major utility type patterns (Partial, Required, Pick, Omit, Record, Exclude, Extract, etc.)
- [x] Added solver coverage for template literal patterns and string intrinsics
- [x] Added solver coverage for recursive conditional types and variadic tuples
- [x] Added solver edge case tests (intersections, unions, generics, conditionals)
- [x] Added checker coverage for class/enum/function namespace merges
- [x] Fixed namespace value/type member access tests (all passing)
- [x] Fixed compilation error in subtype_tests.rs (object_shape_with_index -> object_with_index)
- [x] Added 5 namespace type member access pattern tests
- [x] Added 11 circular constraints in extends clauses tests (F-bounded polymorphism)
- [x] Added 10 additional circular constraint edge cases (polymorphic this, promise, event emitter, fluent interface, recursive JSON, linked list, state machine, visitor, expression tree, repository patterns)
- [x] Added 15 inference from usage pattern tests
- [x] Added 20 context-sensitive typing tests
- [x] Added 25 advanced generic inference tests (mapped types, conditional infer, variadic tuples)
- [x] Added 20 distributive conditional types stress tests
- [x] Added 20 circular constraint edge case tests (5-way cycles, diamond pattern, mutual recursion, index signatures)
- [x] Added 16 mapped type edge case tests (homomorphic modifiers, key remapping, Pick/Omit/Record patterns)
- [x] Added 22 index signature tests (string/number keys, intersection, readonly, value types)
- [x] Added 25 generic constraint tests (extends, keyof constraints)
- [x] Added 20 recursive type tests (self-referential types)
- [x] Added 25 readonly/optional modifier tests
- [x] Added 30 unknown type tests (type guards, narrowing)
- [x] Added 22 class type tests (extends, implements, protected)
- [x] Added 28 interface type tests (extends, merge declarations, excess property checks)
- [x] Added 30 type alias tests (generic, recursive, circular references)
- [x] Added 35 literal type tests (string, number, boolean, template literal)
- [x] Added 28 object type tests (optional properties, excess property checks, type widening)
- [x] Added 30 bigint type tests (literal bigints, arithmetic operations)
- [x] Added 30 typeof type tests (type queries on values, expressions)
- [x] Added 30 infer type tests (conditional type inference)
- [x] Added 30 this type tests (this in classes, fluent interfaces)
- [x] Added 30 readonly property tests (readonly modifiers, Readonly<T>)

## Ready for Merge
No

## Notes
- Implemented access modifier enforcement + tests.
- `./wasm/test.sh` failed due to existing repo errors (BindResult import, TemplateLiteralSpan, object_with_index signature).
- `git push origin worker/forge-4` failed: `Permission denied (publickey)`.
- Be careful with `#private` fields (different mechanism).
- **NEVER edit**: `STRUCTURE.md`, `GOALS.md`, other workers' plan files, or anything in `orchestrator/`
