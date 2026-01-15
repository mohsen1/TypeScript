# WORKER-1 TASK LIST

## Team Assignment
- **EM:** EM-1
- **Branch:** worker-1
- **Parent:** em-team-1

## Status: Task Assignment Updated

### Previous Task: Completed ✅
**Task 1: Error Resynchronization** - COMPLETE
- Commit: `e4ca40664` Complete: Worker-1 Task Completion Notice
- Commit: `a8dcb4e2a` Complete: Error Resynchronization - Target Achieved
- Baseline: 25 errors (Target: <40) ✅ PASS

---

## Current Task: Reduce Remaining TS1005 Errors

### Mission
Further reduce TS1005 ("';' expected") parser errors from current baseline to approach the <40 target across the full conformance test suite.

### Current Data
- **Baseline measurement:** 25 errors ✅ (target achieved)
- **Conformance tests (5000 files):** 331 TS1005 extra errors
- **Target:** <40 extra errors in full conformance suite

### Analysis
The baseline measurement achieved <40 errors, but the full conformance test suite still shows 331 TS1005 errors. These remaining errors are likely from:
1. Edge cases in complex syntax patterns
2. Specific grammar constructs not covered by initial resync
3. Interaction between multiple parsing rules

### Task: Target Top TS1005 Error Patterns

#### Subtask 1: Analyze Top TS1005 Patterns
**Priority:** HIGH
**File:** Create analysis document

Identify the most common TS1005 error patterns:
1. Run conformance tests and collect TS1005 error samples
2. Categorize by syntactic context (expressions, statements, declarations)
3. Identify patterns affecting >5 files each
4. Create targeted fix list

**Deliverable:** `TS1005_PATTERN_ANALYSIS.md` with:
- Top 10 error patterns by frequency
- Example code for each pattern
- Proposed fix approach

#### Subtask 2: Fix Expression Statement Semicolon Insertion
**Priority:** HIGH
**File:** `wasm/src/thin_parser.rs`

Many TS1005 errors occur when semicolons are missing in expression statements.
The resync was added at `parse_expression_statement` level but may need refinement.

**Actions:**
1. Review ASI rules for expression statements
2. Check if newline-based semicolon insertion is working correctly
3. Add better handling for restricted productions
4. Test with specific edge cases

**Acceptance Criteria:**
- Expression statements without semicolons handled correctly
- No false positives for valid syntax
- Test cases added for edge cases

#### Subtask 3: Improve Statement Boundary Detection
**Priority:** MEDIUM
**File:** `wasm/src/thin_parser.rs`

Enhance the synchronization logic to better detect statement boundaries:
1. Add more synchronization points (e.g., after keywords)
2. Improve detection of nested structures
3. Handle edge cases in braces/parentheses

**Acceptance Criteria:**
- Parser recovers faster from errors in nested structures
- Fewer cascading errors
- No crashes or panics

### Success Metric
Reduce conformance test TS1005 errors from **331 to <100**.

### Notes
- Reference TypeScript parser at `src/compiler/parser.ts`
- Run conformance tests after each change
- Work incrementally; test frequently
- Coordinate with EM-1 before merging

---

## Recent Updates
### 2026-01-14 - Task Assignment Updated
- Previous error resynchronization task marked complete
- New task assigned: Reduce remaining TS1005 errors from 331 to <100
- Focus on top error patterns and expression statement handling
