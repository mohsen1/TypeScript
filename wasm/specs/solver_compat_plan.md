# Solver Track B: The Lawyer (Unsoundness & Compat)

## Mission
Implement TypeScript's intentional unsoundness and compatibility rules. This is the "business logic" layer that wraps the core solver.

## Scope
**Files:** `src/solver/compat.rs`, `src/solver/subtype.rs`

**Independence:** HIGH - Wraps core solver but doesn't modify its internals. Focuses on TypeScript-specific edge cases.

## Current Status
🟢 **Ready to Start** - TypeKey refactor is complete, APIs are stable.

## Tasks

### Phase 1: The Compatibility Layer (compat.rs)
- [x] Create `CompatChecker` struct
  - Wraps `SubtypeChecker` with TypeScript-specific rules
  - `is_assignable(source: TypeId, target: TypeId) -> bool`
- [x] Implement `any` escape hatch
  - If source is `any` -> always return true
  - If target is `any` -> always return true (bivariance)
- [x] Implement `unknown` handling
  - `unknown` accepts anything (top type)
  - Only `unknown` and `any` assignable to `unknown`
- [x] Tests for any/unknown
  - Test: `any` assignable to `string`
  - Test: `string` assignable to `any`
  - Test: `unknown` assignable to `any` but not `string`

### Phase 2: Function Bivariance
- [x] Implement function parameter bivariance (unsound but intentional)
  - `(x: string) => void` assignable to `(x: string | number) => void`
  - This is BACKWARDS from sound variance
- [x] Add strict mode flag
  - `strictFunctionTypes: false` -> bivariant (default TS behavior)
  - `strictFunctionTypes: true` -> contravariant (sound)
- [x] Tests for function variance
  - Test: Bivariant mode allows `(Dog) => void` to `(Animal) => void`
  - Test: Strict mode rejects above
  - Test: Return types remain covariant

### Phase 3: Void Return Special Case
- [x] Implement void return compatibility
  - Functions returning `T` assignable to `() => void`
  - Example: `Array.forEach` callback can return anything
- [x] Add contextual typing support
  - When target is `() => void`, don't check return type
- [x] Tests for void returns
  - Test: `() => number` assignable to `() => void`
  - Test: `() => void` NOT assignable to `() => number`

### Phase 4: Index Signature Compatibility
- [x] Implement excess property checking
  - Object literals: strict (reject extra properties)
  - Non-literals: lenient (allow extra properties)
- [x] Implement index signature compatibility
  - `{ [key: string]: number }` accepts `{ a: number, b: number }`
  - But not `{ a: number, b: string }`
- [x] Tests for index signatures
  - Test: Index signature accepts conforming objects
  - Test: Excess property errors on literals
  - Test: Fresh vs non-fresh object types

### Phase 5: Weak Type Detection
- [x] Implement weak type detection (no common properties)
  - Reject assignments to types with only optional properties and no overlap
- [x] Tests for weak types

### Phase 6: Rest Parameter Bivariance
- [x] Accept `(...args: any[] | unknown[]) => ...` as a universal supertype for params
- [x] Tests for rest-parameter bivariance
- [x] Callable/overload rest any/unknown coverage

### Phase 7: Empty Object Assignability
- [x] Treat `{}` as non-nullish top (accept primitives, arrays, functions)
- [x] Tests for `{}` vs `object`/nullish cases

### Phase 8: Integration & Polish
- [x] Wire compat.rs to be the public API
  - ThinChecker assignability now routes through `CompatChecker`
- [x] Add comprehensive error messages
  - Explain WHY assignment failed (which property, which parameter)
- [x] Annotate compat rules with TS issue links
- [x] Performance optimization
  - Cache compat checks (memoization)
  - Short-circuit on `any` early

### Phase 9: Object Keyword Assignability
- [x] Treat `object` as non-primitive top (accept objects, arrays, tuples, functions)
- [x] Tests for `object` keyword accept/reject cases

### Phase 10: Optional Property Widening
- [x] Treat optional properties as `T | undefined` by default (exactOptionalPropertyTypes off)
- [x] Tests for optional property assignability and index signature interactions

### Phase 11: Method Bivariance
- [x] Mark method signatures and apply bivariant parameter checks regardless of strictFunctionTypes
- [x] Tests for method vs function property variance

## Architecture Notes
- This module implements TypeScript's INTENTIONAL unsoundness
- Document why each unsound rule exists (comments with TS issue links)
- Keep core subtype logic pure - all hacks go in compat layer
- Use the Tracer pattern for detailed diagnostics

## Success Criteria
- `cargo test solver::compat` passes all tests
- Handles all TypeScript edge cases from real codebases
- Clear separation between "mathematically correct" (subtype.rs) and "pragmatically useful" (compat.rs)
- Matches TypeScript compiler behavior on compatibility edge cases
