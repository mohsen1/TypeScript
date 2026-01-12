# Worker 5 Plan - Squad Forge

## Mission
Fix New Expression Type Inference

Status: Active
Priority: P2 (Medium)

## Current Assignment
**Fix New Expression Type Inference (4 failing tests)**

### Background
`new Class()` should infer the proper instance type including inherited properties from base classes. Constructor overload resolution also needs work.

### Failing Tests
1. [x] `test_new_expression_infers_class_instance_type`
2. [x] `test_new_expression_infers_base_class_properties`
3. [x] `test_new_expression_resolves_constructor_overloads`
4. [x] `test_new_expression_resolves_constructor_overloads_with_rest`

### Implementation Steps
1. [x] Read failing tests to understand expected behavior
2. [x] Find `check_new_expression` in `src/thin_checker.rs`
3. [x] Ensure it:
   - Gets the constructor signature(s) from the class type
   - Resolves overloads based on argument types
   - Returns the instance type (not the static/constructor type)
   - Includes inherited properties from base class
4. [x] For overload resolution, check `check_call_expression` as reference
5. [x] Test: `./wasm/test.sh 2>&1 | grep -E "new_expression"`

### Key Code Locations
- `src/thin_checker.rs` - `check_new_expression()`
- `src/solver/operations.rs` - construct signature handling
- `src/binder.rs` - class instance vs static members

### Instance Type vs Constructor Type
```typescript
class Foo { x: number }
// typeof Foo = constructor type (has 'new' signature)
// Foo instance type = { x: number }
// new Foo() should return instance type
```

### Inherited Properties
```typescript
class Base { a: number }
class Derived extends Base { b: string }
const d = new Derived(); // type should have both 'a' and 'b'
```

## Task Queue
- [ ] After new expression: investigate abstract class issues if time

## Completed
- [x] Fix New Expression Type Inference (4 failing tests)
  - Fixed constructor overload detection to count only implementations (with body), not overloads (without body)
  - Fixed definite assignment issues in test cases
  - All 7 new expression tests now passing

## Ready for Merge
Yes

## Notes
- Follow `wasm/specs/WASM_ARCHITECTURE.md`
- Use Docker for Rust tests: `./wasm/test.sh`
- Commit format: `[wasm] checker: Fix new expression instance type inference`
- Sync before each task: `git fetch origin && git merge origin/rust --no-edit`
- Push to: `origin/worker/forge-5`
