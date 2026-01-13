# Worker 5 Task List

## Squad: CFA Squad (TS2454 Focus)

## Current Task
- [ ] Identify patterns where our CFA thinks variable is unassigned but tsc accepts it

## Queue
- [ ] Fix flow analysis for loops, try/catch, and conditional assignments
- [ ] Add test cases for patterns that cause false positive TS2454

## Completed
- [x] Review narrowing logic in `wasm/src/solver/narrowing.rs` for over-aggressive unassigned detection
- [x] Analyze TS2454 extra errors (Variable used before being assigned) - 225 false positives

## Code Review Findings

### File: `wasm/src/solver/narrowing.rs`
**Status:** ✅ NOT the source of TS2454 false positives

This file handles **type narrowing** for discriminated unions and type guards (e.g., `typeof x === "string"`).
It does NOT handle definite assignment checking. The TS2454 error logic is elsewhere.

### File: `wasm/src/checker/control_flow.rs` - `check_definite_assignment()`
**Status:** ⚠️ Primary logic for TS2454 - needs investigation

Key function: `check_definite_assignment()` (lines 265-350)

#### Current Behavior Analysis:
1. **ASSIGNMENT nodes:** Correctly detects direct assignments to the reference variable
2. **BRANCH_LABEL (merge points):** Requires ALL antecedents to have variable assigned (correct)
3. **LOOP_LABEL:** Only checks first antecedent (loop entry) - correct for conservative analysis
4. **UNREACHABLE paths:** Properly skipped (return true for vacuous satisfaction)

#### Potential Issues Found:

1. **Cycle detection returns `false`** (line 276-278):
   ```rust
   if visited.contains(&flow_id) {
       return false;
   }
   ```
   When a cycle is detected, it returns `false` (unassigned). This is conservative but may cause false positives in complex loop scenarios where the assignment was already found in a different path.

2. **Missing flow node returns `true`** (line 343-344):
   ```rust
   } else {
       true
   }
   ```
   When a flow node doesn't exist, it returns `true` (definitely assigned). This seems backwards - should probably return `false` for safety.

3. **Exhaustive switch without default:** The current logic doesn't detect when a switch statement over a union type covers all cases without an explicit `default` clause.

### File: `wasm/src/thin_checker.rs` - `should_check_definite_assignment()`
**Status:** 📋 Gating logic for when TS2454 checks apply

Already handles several skip conditions:
- Parameters (line 5371-5373)
- Definite assignment assertions `!` (line 5375-5377)
- For-in/for-of loop targets (line 5379-5381)
- Variables with initializers (line 5384-5386)
- Ambient context variables (line 5389-5391)
- Types that allow uninitialized use (undefined, literal types) (line 5397-5399)

### Patterns Likely Causing False Positives:

1. **do-while loops:** Body always executes once, so `do { x = 1; } while(cond); console.log(x);` should be valid

2. **Exhaustive switch on union types:**
   ```typescript
   let x: number;
   type T = "a" | "b";
   declare const t: T;
   switch (t) {
     case "a": x = 1; break;
     case "b": x = 2; break;
   }
   console.log(x); // tsc accepts, we might flag
   ```

3. **Early return narrowing:**
   ```typescript
   let x: number;
   if (cond) return;
   x = 1;
   console.log(x); // x is definitely assigned here
   ```

4. **Never-returning functions:**
   ```typescript
   function fail(): never { throw new Error(); }
   let x: number;
   if (cond) { fail(); }
   x = 1;
   console.log(x); // x is definitely assigned
   ```

## Context
- **Goal:** Reduce TS2454 extra errors (225 false positives)
- **Key files:** `wasm/src/checker/control_flow.rs`, `wasm/src/solver/narrowing.rs`, `wasm/src/checker/reachability_analyzer.rs`
- **Status:** Code review complete - identified patterns needing investigation
