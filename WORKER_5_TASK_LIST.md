# Worker 5 Task List

## Squad: CFA Squad (TS2454 Focus)

## Current Task
- [x] Identify patterns where our CFA thinks variable is unassigned but tsc accepts it

## Queue
- [ ] Fix flow analysis for loops, try/catch, and conditional assignments
- [ ] Add test cases for patterns that cause false positive TS2454

## Completed
- [x] Review narrowing logic in `wasm/src/solver/narrowing.rs` for over-aggressive unassigned detection
- [x] Analyze TS2454 extra errors (Variable used before being assigned) - 225 false positives
- [x] Identify patterns where our CFA thinks variable is unassigned but tsc accepts it

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

---

## TS2454 FALSE POSITIVE PATTERNS ANALYSIS

### Executive Summary

After thorough analysis of the CFA implementation, I've identified **7 distinct patterns** where our
`check_definite_assignment()` function in `wasm/src/checker/control_flow.rs` (lines 265-350) reports
false positives (says variable is unassigned when tsc accepts the code).

### Pattern Categories

#### Category 1: Exhaustive Control Flow (Requires Type Information)

**Pattern 1.1: Exhaustive Switch Statements**
```typescript
let x: number;
type Status = "success" | "error";
declare const status: Status;
switch (status) {
  case "success": x = 1; break;
  case "error": x = 2; break;
}
console.log(x); // tsc: OK (exhaustive), us: TS2454
```
- **Root Cause**: Our CFA doesn't have access to type information to detect exhaustive coverage
- **Fix Complexity**: HIGH - requires solver integration
- **Impact**: ~30-50 false positives

**Pattern 1.2: Discriminated Unions with Exhaustive Checks**
```typescript
let result: number;
type Action = { type: "add"; value: number } | { type: "sub"; value: number };
function process(action: Action) {
  if (action.type === "add") {
    result = action.value;
  } else {
    result = -action.value;
  }
  console.log(result); // May be flagged incorrectly
}
```

#### Category 2: Never-Returning Functions

**Pattern 2.1: Never-Returning Function Calls**
```typescript
function fail(): never { throw new Error(); }

let x: number;
if (condition) {
  fail();
}
x = 1;
console.log(x); // tsc: OK, us: may flag TS2454
```
- **Root Cause**: Need to verify `flow_flags::CALL` nodes for never-returning functions mark subsequent code as UNREACHABLE
- **Implementation Check**: `flow_graph_builder.rs` should handle this, but may not detect user-defined never functions
- **Fix Complexity**: MEDIUM - need to check return type of call expressions

**Pattern 2.2: Assertion Functions**
```typescript
function assertDefined<T>(val: T | undefined): asserts val is T {
  if (val === undefined) throw new Error();
}

let x: number;
let val: number | undefined;
assertDefined(val);
x = val; // tsc: OK, us: may flag
console.log(x);
```

#### Category 3: Guaranteed Loop Execution

**Pattern 3.1: Do-While Loops**
```typescript
let x: number;
do {
  x = 1;
} while (false);
console.log(x); // tsc: OK (body always executes), us: may flag TS2454
```
- **Root Cause**: `check_definite_assignment` for LOOP_LABEL only checks first antecedent (entry point)
- **Analysis**: Looking at `flow_graph_builder.rs` line 520-524, the body IS processed before condition
- **Verification Needed**: Check if merge label correctly receives the definite assignment from body
- **Fix Complexity**: LOW - may just need to trace through the flow graph correctly

#### Category 4: Early Exit Narrowing

**Pattern 4.1: Early Return**
```typescript
function test(condition: boolean) {
  let x: number;
  if (condition) return;
  x = 1;
  console.log(x); // tsc: OK (only reached if !condition), us: may flag TS2454
}
```
- **Root Cause**: If one branch returns, the merge point should only have one incoming edge
- **Implementation Check**: BRANCH_LABEL handling at lines 292-307 checks all antecedents, but unreachable branches should be skipped (lines 300-303)
- **Verification Needed**: Check if early return properly marks the branch as UNREACHABLE

**Pattern 4.2: Early Throw**
```typescript
function test(condition: boolean) {
  let x: number;
  if (condition) throw new Error();
  x = 1;
  console.log(x); // tsc: OK, us: may flag TS2454
}
```

#### Category 5: Short-Circuit Assignment Patterns

**Pattern 5.1: Logical AND with Assignment**
```typescript
let x: number;
let obj: { value: number } | null = getObj();
if (obj && (x = obj.value) > 0) {
  console.log(x); // x is definitely assigned here
}
```

**Pattern 5.2: Nullish Coalescing with Assignment**
```typescript
let x: number;
x = getValue() ?? (x = 1, 0);
console.log(x); // Always assigned
```

#### Category 6: Cycle Detection Being Too Conservative

**Pattern 6.1: Complex Loop with Multiple Paths**
```typescript
let x: number;
while (condition) {
  x = getValue();
  if (shouldBreak()) break;
  // ... more code that uses x
}
// At merge point after loop, cycle detection may return false prematurely
```
- **Root Cause**: Line 276-278 returns `false` on cycle detection
- **Fix**: May need smarter cycle handling that returns cached partial result

#### Category 7: Finally Block Patterns

**Pattern 7.1: Assignment in Finally Block**
```typescript
let x: number;
try {
  mayThrow();
} finally {
  x = 1;
}
console.log(x); // tsc: OK (finally always runs), us: may flag TS2454
```
- **Root Cause**: Finally blocks require special flow graph handling
- **Implementation Check**: Need to verify `flow_graph_builder.rs` handles finally correctly

### Recommended Fix Priority

| Priority | Pattern | Impact | Complexity |
|----------|---------|--------|------------|
| 1 | Early Return/Throw (4.1, 4.2) | HIGH | LOW |
| 2 | Finally Blocks (7.1) | MEDIUM | MEDIUM |
| 3 | Never-Returning Functions (2.1) | MEDIUM | MEDIUM |
| 4 | Do-While Loops (3.1) | LOW | LOW |
| 5 | Assertion Functions (2.2) | LOW | HIGH |
| 6 | Exhaustive Switch (1.1) | HIGH | VERY HIGH |

### Implementation Notes

The key function is `check_definite_assignment()` at `wasm/src/checker/control_flow.rs:265`:

1. **UNREACHABLE handling** (line 282-283): Currently returns `false`, which is correct
2. **BRANCH_LABEL handling** (lines 292-307): Correctly skips UNREACHABLE antecedents
3. **LOOP_LABEL handling** (lines 308-313): Only checks first antecedent - need to verify do-while case
4. **Cycle detection** (lines 276-278): Returns `false` on cycle - may be too conservative

The flow graph is built in `flow_graph_builder.rs`:
- `build_do_while_statement()` at line 495 - handles do-while loops
- `build_try_statement()` - handles try/catch/finally
- Never-returning calls should set UNREACHABLE on subsequent flow

### Files Analyzed
- `wasm/src/checker/control_flow.rs` - Core CFA logic
- `wasm/src/checker/flow_graph_builder.rs` - Flow graph construction
- `wasm/src/checker/flow_analyzer.rs` - Forward dataflow analysis (alternative approach)
- `wasm/src/thin_checker.rs` - Where TS2454 errors are emitted
- `tests/cases/conformance/controlFlow/definiteAssignmentTS2454EdgeCases.ts` - Test cases
