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
(previous work cleared - fresh start for Operation Conformance)

## Ready for Merge
No

## Notes
- Functions exist but don't emit errors - wire them up
- Be careful with `#private` fields (different mechanism)
- Run `./wasm/test.sh` before pushing
- Commit format: `[wasm] checker: implement TS2341/TS2445 access modifier enforcement`
- Push to: `origin/worker/forge-4`
- **NEVER edit**: `STRUCTURE.md`, `GOALS.md`, other workers' plan files, or anything in `orchestrator/`
