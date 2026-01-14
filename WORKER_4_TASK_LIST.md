# WORKER-4 TASK LIST

## Team Assignment
- **EM:** EM-1
- **Branch:** worker-4
- **Parent:** em-team-1

## Priority: Fix Class Property Initialization (TS2564)

### Mission
Implement the `strictPropertyInitialization` check that verifies class properties are initialized in the constructor. This is the #1 missing error category.

### Current Data
- **Missing TS2564:** 413 errors ("Property 'x' has no initializer and is not definitely assigned in the constructor")
- **Root Cause:** Check is not implemented
- **Target:** <20 missing errors

### Tasks

#### Task 1: Implement Property Initialization Check
**Priority:** TACTICAL
**File:** `wasm/src/checker/thin_checker.rs`

Add `strictPropertyInitialization` validation:
1. Identify all class properties without initializers
2. For each property, verify:
   - Assigned in constructor
   - Assigned in all constructor overloads
   - Assigned via property decorator (if applicable)
3. Emit TS2564 for violations
4. Handle definite assignment analysis (`!` assertion)

**Algorithm:**
```
for each class declaration:
  for each property:
    if property.has_initializer: continue
    if property.has_definite_assignment_assertion: continue
    if not property.assigned_in_all_constructors:
      emit_error(TS2564, property.location)
```

**Edge Cases:**
- Property declarations in base classes
- Protected/private properties
- Static properties
- Abstract classes
- Decorated properties

**Acceptance Criteria:**
- TS2564 emitted for uninitialized properties
- No false positives for definite assignment assertions
- Correct handling of constructor overloads

#### Task 2: Definite Assignment Analysis
**Priority:** HIGH
**File:** `wasm/src/checker/thin_checker.rs`

Implement control flow analysis to determine definite assignment:
1. Track property assignments through constructor body
2. Handle conditional assignments
3. Handle early returns
4. Handle assignment in called functions (limited)

**Acceptance Criteria:**
- Properties assigned in all code paths pass
- Properties with conditional assignments fail
- Early returns don't cause false positives

### Deliverables
1. Implemented `strictPropertyInitialization` check
2. Test suite for property initialization scenarios
3. Conformance test results showing TS2564 reduction

### Success Metric
Reduce missing TS2564 errors from **413 to <20**.

### Notes
- High-ROI task - single check eliminates top missing error category
- Reference TypeScript implementation at `src/compiler/checker.ts`
- Coordinate with EM-1 before merging
