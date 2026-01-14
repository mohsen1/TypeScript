# Worker 12 Task List

## Squad: Solver Strictness - "Any" to "Unknown" Migration

## Current Task
- [ ] Find all locations where `TypeId::ANY` is returned as default/fallback
- [ ] Change default return from `Any` to `Unknown` or `Error`

## Queue
- [ ] Update `lower_type()` to return `Error` instead of `Any` on failure
- [ ] Update `solve_subtype()` to be stricter - fail fast on unresolved types
- [ ] Implement "Lawyer" layer for TypeScript quirks (function bivariance, void return)
- [ ] Run conformance tests - expect spike in errors (this is intentional)
- [ ] Convert "Missing TS2322" into "Exact Match" or "Extra TS2322"

## Completed
- [ ] Branch created from em-team-3
- [ ] Reviewed checker.ts structure for type resolution

## Context
The compiler is "too permissive" - when it encounters something it doesn't understand, it defaults to `Any`. This silences type checking because `Any` disables all type checks in TypeScript.

### The "Any" Poisoning Problem

```typescript
// Example: When binder can't find "Promise"
const p: Promise<number> = Promise.resolve(42);
// If Promise resolves to Any, this becomes:
const p: any = any;  // No errors! Everything silenced!
```

### Strategic Change: Switch Default Fallback

**Current behavior:**
```typescript
// When type resolution fails
return TypeId::ANY;  // Too permissive
```

**Target behavior:**
```typescript
// When type resolution fails
return TypeId::ERROR;  // or TypeId::UNKNOWN
```

### Areas to Update

**Area 1 - Type lowering:**
- `lower_type()` function in checker.ts
- When lowering generic types or complex types, failure should return `Error`
- Search for patterns like: `if (failed) return anyType`

**Area 2 - Subtype checking:**
- `solve_subtype()` - main subtyping logic
- When comparing types fails, should return error not assume compatibility
- Look for "fallback to any" patterns

**Area 3 - Generic inference:**
- When generic parameter can't be inferred, don't default to `Any`
- Return `Unknown` or error instead

**Area 4 - Property access:**
- `getPropertyOfType()` - when property not found
- Currently may return `Any` - should return `Error`

### TypeScript Quirks to Implement ("Lawyer Layer")

1. **Function bivariance:** Function parameters are bidirectionally covariant (unsound)
2. **Void returns:** Functions returning `void` can be assigned to functions returning any type
3. **Enum merging:** Enums merge across declarations
4. **Namespace merging:** Similar to enums

### Expected Outcome
- Conformance scores will **temporarily drop** (more errors = more visible)
- "Missing errors" will convert to "Extra errors" or "Exact match"
- This is GOOD - exposes real bugs instead of hiding them

### Success Metric
- Convert "Missing TS2322" to visible errors
- Better to be too strict (Extra TS2322) than too permissive (Missing TS2322)
- Long-term: Exact Match increases from 30.1% to 40%

### Key Files
- `src/compiler/checker.ts` (3.1M lines) - Main type checking logic
- `src/compiler/types.ts` (487K lines) - Type definitions
- `specs/SOLVER.md` - Reference for subtyping rules (if exists)

### Workflow
1. Create branch: `git checkout -b worker-12-solver-strictness`
2. Find and replace `Any` fallbacks with `Error`/`Unknown`
3. Run tests: `npm run test:conformance`
4. Analyze error spike - this is expected and correct
5. Push: `git push origin worker-12-solver-strictness`
6. Notify EM-3 for merge

### Warning
This change WILL temporarily break metrics. This is intentional and correct. We need to see the errors to fix them.
