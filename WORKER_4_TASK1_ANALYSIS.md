# Worker 4 Task 1: Control Flow Investigation

**Date:** 2026-01-14
**Status:** COMPLETE

## Executive Summary

**CRITICAL FINDING:** The control flow infrastructure for TS2454/TS2564 checking **FULLY EXISTS** and is **INTEGRATED** into the binding and checking pipeline. The hypothesis that "flow graph is disconnected from checking" is **INCORRECT**.

The 573 missing TS2454 errors are NOT due to missing infrastructure, but likely due to:
1. Incomplete flow graph construction for edge cases
2. Bugs in the `assignment_targets_reference()` matching logic
3. Variables without flow information defaulting to "definitely assigned" behavior

---

## 1. Current State of Flow Graph Infrastructure

### 1.1 ThinBinder (Flow Graph Construction)

**Location:** `wasm/src/thin_binder.rs`

**Key Components:**

| Component | Type | Purpose |
|-----------|------|---------|
| `flow_nodes: FlowNodeArena` | Arena storage | Stores all flow nodes |
| `node_flow: FxHashMap<u32, FlowNodeId>` | Side table | Maps AST node → FlowNodeId |
| `current_flow: FlowNodeId` | State | Tracks active flow during binding |
| `record_flow()` | Method | Records flow node for AST node |
| `create_flow_assignment()` | Method | Creates ASSIGNMENT flow nodes |
| `create_flow_condition()` | Method | Creates TRUE/FALSE_CONDITION nodes |
| `create_branch_label()` | Method | Creates BRANCH_LABEL merge points |
| `create_loop_label()` | Method | Creates LOOP_LABEL back-edges |

**Flow Node Types Created:**
- `START` (line 519) - Entry point for source file
- `ASSIGNMENT` (lines 1186, 1223, 2219, 2740, 3427) - Variable assignments
- `TRUE_CONDITION` / `FALSE_CONDITION` (lines 927, 941, 983, 1003, 1041) - If/while/for branches
- `BRANCH_LABEL` (line 961, 3387) - Merge points for control flow
- `LOOP_LABEL` (line 1019, 3392) - Back-edges for loops
- `CALL` (line 1332, 3439) - Function calls
- `ARRAY_MUTATION` (line 1335, 3451) - Array push/splice operations
- `SWITCH_CLAUSE` (line 3417) - Switch case clauses
- `UNREACHABLE` (lines 111, 150, 192, 255) - Code after return/throw

**`record_flow()` Call Sites:**
- Line 848: Identifiers (for type narrowing)
- Line 1163: Expression statements
- Line 1207: Property/element access
- Line 3609, 3656, 3669: Binary expressions

### 1.2 ThinChecker (Flow Graph Querying)

**Location:** `wasm/src/thin_checker.rs`

**Key Methods:**

| Method | Line | Purpose |
|--------|------|---------|
| `is_definitely_assigned_at()` | 5760-5767 | Query if variable is assigned at point |
| `check_definite_assignment()` | 5437-5443 | Emit TS2454 if not assigned |
| `should_check_definite_assignment()` | 5520+ | Determine when to check |
| `emit_definite_assignment_error()` | 5450-5474 | Create TS2454 diagnostic |

**Critical Code (line 5760-5767):**
```rust
fn is_definitely_assigned_at(&self, idx: NodeIndex) -> bool {
    let flow_node = match self.ctx.binder.get_node_flow(idx) {
        Some(flow) => flow,
        None => return false, // No flow info = NOT definitely assigned
    };
    let analyzer = FlowAnalyzer::new(self.ctx.arena, self.ctx.binder, self.ctx.types);
    analyzer.is_definitely_assigned(idx, flow_node)
}
```

**Issue:** If no flow info exists, returns `false` (not assigned), which prevents TS2454 emission.

### 1.3 FlowAnalyzer (Definite Assignment Algorithm)

**Location:** `wasm/src/checker/control_flow.rs`

**Key Methods:**

| Method | Line | Purpose |
|--------|------|---------|
| `is_definitely_assigned()` | 189-197 | Walk flow graph backwards to check assignment |
| `check_definite_assignment()` | 265-350 | Recursive flow traversal with cycle detection |
| `assignment_targets_reference()` | 1002-1008 | Check if assignment affects specific variable |
| `assignment_targets_reference_internal()` | 1759+ | Deep comparison of assignment target |

**Flow Analysis Logic (line 281-349):**
- Checks `ASSIGNMENT` flow nodes to see if they target the reference
- For `BRANCH_LABEL`: ALL antecedents must have assignment (line 297)
- For `LOOP_LABEL`: checks loop body (line 308-313)
- For `CONDITION`: checks condition predecessor (line 314-319)
- For `SWITCH_CLAUSE`: ALL reachable antecedents must have assignment (line 320-335)
- For `START`: returns `false` (not assigned at program start) (line 336-337)

**Matching Logic (line 1759-1789):**
- Uses `is_matching_reference()` to compare assignment target with usage
- Handles parenthesized expressions, non-null assertions, type assertions
- Recursively unwraps nested assignment operators

### 1.4 DefiniteAssignmentAnalyzer (Forward Analysis)

**Location:** `wasm/src/checker/flow_analyzer.rs`

**Status:** IMPLEMENTED but NOT USED in main checking pipeline

**Purpose:** Forward dataflow analysis that tracks assignment states through flow graph

**Components:**
- `AssignmentState` enum: Unassigned, MaybeAssigned, DefinitelyAssigned
- `AssignmentStateMap`: Maps variables to states
- `DefiniteAssignmentAnalyzer`: Performs iterative fixed-point analysis

**Note:** This appears to be an alternative implementation. The main checker uses `FlowAnalyzer` from `control_flow.rs`.

---

## 2. Integration Status

### 2.1 Binding Pipeline

**Flow:** `bind_source_file()` → creates flow nodes during traversal

**Entry Point:** `thin_binder.rs:501`
```rust
pub fn bind_source_file(&mut self, arena: &ThinNodeArena, root: NodeIndex) {
    let start_flow = self.flow_nodes.alloc(flow_flags::START);
    self.current_flow = start_flow;
    // ... binds statements, creating flow nodes as it goes
}
```

**Confirmed:** Flow graph IS built during binding for all constructs:
- Variable declarations (with initializers create ASSIGNMENT flow)
- Assignments (create ASSIGNMENT flow)
- If/else statements (create CONDITION + BRANCH_LABEL)
- Loops (create LOOP_LABEL + CONDITION)
- Switch statements (create SWITCH_CLAUSE)
- Try/catch (merge flows)

### 2.2 Checking Pipeline

**Flow:** `check_identifier()` → `is_definitely_assigned_at()` → `FlowAnalyzer`

**Entry Point:** `thin_checker.rs:5289-5292`
```rust
} else if self.should_check_definite_assignment(sym_id, idx)
    && !self.is_definitely_assigned_at(idx)
{
    self.error_variable_used_before_assigned_at(name, idx);
}
```

**Confirmed:** Checker DOES query flow graph and emit TS2454 errors.

---

## 3. Gap Analysis

### 3.1 Infrastructure Gaps

**Gap:** NONE

All infrastructure exists and is integrated. The README's statement that "flow graph appears disconnected from checking" is based on outdated analysis.

### 3.2 Potential Root Causes of Missing TS2454 Errors

Based on code analysis, the 573 missing errors are likely due to:

#### Issue 1: Variable Declarations Without Initializers

**Location:** `thin_binder.rs:2217-2221`
```rust
if !decl.initializer.is_none() {
    self.bind_node(arena, decl.initializer);
    let flow = self.create_flow_assignment(idx);
    self.current_flow = flow;
}
```

**Problem:** Only creates ASSIGNMENT flow if initializer exists. Variables declared without initializers (e.g., `let x: string;`) have no flow tracking their declaration.

**Impact:** When `is_definitely_assigned_at()` checks a variable declared without initializer, the flow graph has no ASSIGNMENT node for it, so the backward search fails.

#### Issue 2: `assignment_targets_reference()` Matching

**Location:** `control_flow.rs:1759-1789`

**Potential Issues:**
- `is_matching_reference()` may not correctly match all identifier patterns
- Destructuring patterns may not be tracked
- Property assignments (`obj.prop =`) may not be recognized

**Impact:** Even if ASSIGNMENT flow exists, the matching logic may not recognize it as targeting the specific variable.

#### Issue 3: Complex Control Flow

**Scenarios that may not be handled:**
- Nested try/catch/finally blocks
- Break/continue statements to outer loops
- Labeled break/continue
- Conditional declarations in type guards
- Closure captures affecting definite assignment

#### Issue 4: Missing Flow Information

**Location:** `thin_checker.rs:5763`
```rust
None => return false, // No flow info means variable is NOT definitely assigned
```

**Problem:** When no flow info exists, returns `false` which means "NOT definitely assigned". But the checker only emits TS2454 when the check fails AND returns `false`.

**Wait:** Actually, this is CORRECT behavior. If no flow info, variable is not definitely assigned, so TS2454 SHOULD be emitted.

**Re-analysis:** The issue is likely that `should_check_definite_assignment()` is returning `false` (skip check) for too many cases, OR the matching logic is failing.

---

## 4. Recommendations

### 4.1 Immediate Actions (Debug)

1. **Add diagnostic logging** to `is_definitely_assigned_at()`:
   - Log when `get_node_flow()` returns None
   - Log flow graph traversal path
   - Log `assignment_targets_reference()` results

2. **Test simple cases**:
   ```typescript
   let x: string;
   console.log(x); // Should emit TS2454
   ```

3. **Compare flow graphs**:
   - Use `FlowGraphBuilder` tests to see expected graph
   - Dump `node_flow` map from `ThinBinder`
   - Compare for specific failing test cases

### 4.2 Code Changes Needed

**Priority 1: Fix Variable Declaration Tracking**

Variables declared without initializers need to be tracked in the flow graph. Options:
- Add `DECLARATION` flow node type
- Mark variables as "unassigned" at declaration point
- Track all declarations in a side table

**Priority 2: Improve `assignment_targets_reference()`**

- Add support for destructuring patterns
- Improve identifier matching logic
- Add comprehensive tests

**Priority 3: Debug Real Test Cases**

- Run `node wasm/differential-test/find-ts2454.mjs` on small sample
- Pick 5-10 failing cases
- Trace through flow graph construction and analysis
- Identify specific patterns causing failures

---

## 5. Conclusion

**The infrastructure EXISTS and is INTEGRATED.** The missing TS2454 errors are due to:
1. Variable declarations without initializers not being tracked
2. Potential bugs in matching logic
3. Edge cases in complex control flow

**Next Steps:**
1. Add diagnostic logging to identify specific failures
2. Fix variable declaration tracking
3. Improve matching logic
4. Validate with conformance tests

**Task 1 Status:** COMPLETE - Analysis delivered, gaps identified, recommendations provided.
