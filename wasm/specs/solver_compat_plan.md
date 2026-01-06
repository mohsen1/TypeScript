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
- [ ] Create `CompatChecker` struct
  - Wraps `SubtypeChecker` with TypeScript-specific rules
  - `is_assignable(source: TypeId, target: TypeId) -> bool`
- [ ] Implement `any` escape hatch
  - If source is `any` -> always return true
  - If target is `any` -> always return true (bivariance)
- [ ] Implement `unknown` handling
  - `unknown` accepts anything (top type)
  - Only `unknown` and `any` assignable to `unknown`
- [ ] Tests for any/unknown
  - Test: `any` assignable to `string`
  - Test: `string` assignable to `any`
  - Test: `unknown` assignable to `any` but not `string`

### Phase 2: Function Bivariance
- [ ] Implement function parameter bivariance (unsound but intentional)
  - `(x: string) => void` assignable to `(x: string | number) => void`
  - This is BACKWARDS from sound variance
- [ ] Add strict mode flag
  - `strictFunctionTypes: false` -> bivariant (default TS behavior)
  - `strictFunctionTypes: true` -> contravariant (sound)
- [ ] Tests for function variance
  - Test: Bivariant mode allows `(Dog) => void` to `(Animal) => void`
  - Test: Strict mode rejects above
  - Test: Return types remain covariant

### Phase 3: Void Return Special Case
- [ ] Implement void return compatibility
  - Functions returning `T` assignable to `() => void`
  - Example: `Array.forEach` callback can return anything
- [ ] Add contextual typing support
  - When target is `() => void`, don't check return type
- [ ] Tests for void returns
  - Test: `() => number` assignable to `() => void`
  - Test: `() => void` NOT assignable to `() => number`

### Phase 4: Index Signature Compatibility
- [ ] Implement excess property checking
  - Object literals: strict (reject extra properties)
  - Non-literals: lenient (allow extra properties)
- [ ] Implement index signature compatibility
  - `{ [key: string]: number }` accepts `{ a: number, b: number }`
  - But not `{ a: number, b: string }`
- [ ] Tests for index signatures
  - Test: Index signature accepts conforming objects
  - Test: Excess property errors on literals
  - Test: Fresh vs non-fresh object types

### Phase 5: Integration & Polish
- [ ] Wire compat.rs to be the public API
  - Other modules call `is_assignable()` not `is_subtype()`
- [ ] Add comprehensive error messages
  - Explain WHY assignment failed (which property, which parameter)
- [ ] Performance optimization
  - Cache compat checks (memoization)
  - Short-circuit on `any` early

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
