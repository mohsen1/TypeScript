# Worker 10 Task List (EM-3: Semantics Squad)

**Worker:** worker-10
**EM:** EM-3
**Focus Area:** Type checking strictness, Solver fallback behavior
**Last Updated:** 2026-01-14

---

## CURRENT TASK

### Task 1: Audit and Replace `any` Type Fallback in Checker

**File:** `src/compiler/checker.ts`

**Problem:** The TypeScript checker returns `any` type when it cannot resolve a type, silencing downstream errors.

**Steps:**
1. Search for all instances where `anyType` or `unknownType` is returned as a fallback
2. Identify functions that default to `any` on resolution failure
3. Document each occurrence with:
   - Function name
   - Line number
   - Context (what triggers the fallback)
   - Whether it should return `unknown` instead

**Search Commands:**
```bash
# Search for anyType returns
grep -n "return anyType" src/compiler/checker.ts

# Search for unknownType usage
grep -n "unknownType" src/compiler/checker.ts

# Search for error type fallbacks
grep -n "errorType" src/compiler/checker.ts
```

**Deliverable:** Create a document listing all fallback locations with recommendations.

---

## PENDING TASKS

### Task 2: Replace `any` Fallback with `unknown` in Type Inference

**Priority:** HIGH
**Estimated Impact:** +200-400 extra errors (exposing real bugs)

**Files:**
- `src/compiler/checker.ts`
- `src/compiler/types.ts`

**Changes:**
1. Change `anyType` fallback to `unknownType` in:
   - `getWidenedType()` - type widening
   - `getBaseTypeOfLiteralType()` - literal type base
   - `inferTypeFromAssignment()` - assignment inference
   - `inferTupleTypes()` - tuple inference

2. Update error messages to explicitly state "type is `unknown`" instead of silently accepting

3. Verify `unknown` type propagates correctly:
   - `unknown` assigned to `string` should error (TS2322)
   - `unknown` assigned to `any` should succeed
   - `unknown` method calls should error (TS2339)

**Test:**
```bash
npm run test:conformance
```

**Expected:** Spike in extra errors (correct - we're exposing real bugs)

---

### Task 3: Implement Strict Subtype Checking for Type Assignability

**Priority:** HIGH
**File:** `src/compiler/checker.ts`

**Problem:** Subtype checking is too lenient, missing TS2322 errors.

**Target Functions:**
- `isTypeAssignableTo()`
- `isRelatedTo()`
- `isStructuredTypeAssignableTo()`

**Changes:**
1. Remove "optimistic" subtype checks that return `true` on uncertainty
2. Implement stricter property checking:
   - Excess properties in object literals
   - Missing properties in assignments
   - Optional vs required properties

3. Add specific TypeScript quirks (Lawyer layer):
   - **Function bivariance:** Function parameters are bi-variant in some cases
   - **Void returns:** Functions returning `void` have special assignability
   - **Enum subtyping:** Numeric enums assignable to `number`
   - **Class typing:** Both structural and nominal

**Test Case:**
```typescript
interface Animal { name: string; }
interface Dog extends Animal { bark(): void; }

let animal: Animal = { name: "Buddy" };
let dog: Dog = animal;  // Should error TS2322
```

---

### Task 4: Fix Generic Type Inference

**Priority:** MEDIUM
**File:** `src/compiler/checker.ts`

**Problem:** Generic types often fail to infer, defaulting to `any`.

**Target Functions:**
- `inferTypeArguments()`
- `getInferredType()`
- `inferFromTypes()`

**Changes:**
1. When generic inference fails, default to `unknown` (not `any`)
2. Implement constraint solving:
   - Use argument types to infer type parameters
   - Check `<T extends Constraint>` bounds
   - Handle default type parameters `<T = string>`

3. Fix common patterns:
   ```typescript
   function identity<T>(x: T): T { return x; }
   let result = identity(42);  // Should infer T = number
   ```

---

## COMPLETED TASKS

*None yet*

---

## NOTES

- This is the TypeScript codebase, not Rust
- `checker.ts` is ~80,000 lines - use grep/search strategically
- Focus on type resolution failure paths
- Document every change with comment explaining the "why"

---

## BLOCKERS

*None reported*
