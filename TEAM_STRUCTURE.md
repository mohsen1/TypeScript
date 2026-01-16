# Project Zang - Team Structure and Work Distribution

## Overview

This document outlines the work distribution plan for 14 workers on Project Zang, a TypeScript compiler rewrite in Rust/WASM. Tasks are organized by priority tier following the project's established guidelines.

---

## Team Organization

### Leadership Structure

**Director:** Worker 1
- Created work distribution plan
- Coordinates across all Engineering Managers
- Handles escalations and cross-team dependencies

### Engineering Managers & Teams

| EM | Worker # | Team Focus | Team Members | Priority |
|----|----------|------------|--------------|----------|
| EM-1 | Worker 2 | Tier 0: Quality & Stability | Workers 2, 3, 4, 5 | HIGHEST |
| EM-2 | Worker 6 | Tier 1: Parser Accuracy | Workers 6, 7, 8, 9 | HIGH |
| EM-3 | Worker 10 | Tier 2-3: Type Checker & Symbol Resolution | Workers 10, 11, 12, 13, 14 | MEDIUM |

**Note:** EMs are individual contributors who also manage their team's work.

### Worker Status Table

| Worker | Squad | Status | Current Task |
|--------|-------|--------|--------------|
| Worker 5 | Syntax Squad | Active | Statement-Level Error Recovery Enhancement |
| Worker 6 | Binder Squad | Active | TS2304 Global Scope / Lib Injection (ongoing) |
| Worker 7 | Semantics Squad | Approved | Module Symbol Resolution (TS7005, TS7008, TS2792) |
| Worker 8 | LSP Squad | Approved | LSP TypeScript Config Integration |

---

## Worker Assignments

### Team 1: Quality & Stability Foundations (Tier 0)

**Engineering Manager:** Worker 2 (EM-1)
**Priority:** HIGHEST - These issues block correctness across all tiers
**Tier:** 0 - Quality & Stability

#### Worker 2 (EM-1): Application Type Expansion
**Task:** Fix `TypeKey::Application` not being expanded, causing incorrect diagnostics/assignability

**Key Files:**
- `wasm/src/solver/evaluate.rs`
- `wasm/src/solver/instantiate.rs`
- `wasm/src/solver/intern.rs`

**Details:**
- `TypeKey::Application` wraps a generic type with type arguments but is not properly expanded during type checking
- Reuse existing instantiation logic in `wasm/src/solver/instantiate.rs`
- Re-enable solver tests once updated

**Acceptance Criteria:**
- [ ] Generic type applications are properly expanded
- [ ] Type assignability works correctly with generic types
- [ ] Solver tests pass

---

#### Worker 3: Readonly Types Implementation
**Task:** Implement readonly arrays/tuples (currently treated as mutable)

**Key Files:**
- `wasm/src/solver/subtype.rs`
- `wasm/src/solver/types.rs`
- `wasm/src/thin_checker.rs`

**Details:**
- `readonly T[]` and `readonly [T, U]` should not be assignable to mutable versions
- Need distinct type representation for readonly variants
- Add assignability checks for write operations on readonly types

**Acceptance Criteria:**
- [ ] Readonly arrays/tuples have distinct type representation
- [ ] Assignability correctly rejects mutable-to-readonly assignments
- [ ] Write operations on readonly types emit appropriate errors

---

#### Worker 4: AST Child Enumeration Fix
**Task:** Fix `get_children` returning empty in parser arenas, breaking traversal-based features

**Key Files:**
- `wasm/src/parser/arena.rs`
- `wasm/src/parser/thin_node.rs`
- `wasm/src/thin_parser.rs`

**Details:**
- AST children should be enumerated per-node-kind for deterministic traversals
- Do not allocate large temporary trees
- This blocks features that rely on AST traversal

**Acceptance Criteria:**
- [ ] `get_children` returns correct child nodes for all node types
- [ ] No large temporary allocations during traversal
- [ ] Traversal-based features work correctly

---

#### Worker 5: Solver Test Coverage Restoration
**Task:** Re-enable commented out solver tests (infer/subtype/evaluate) due to API drift

**Key Files:**
- `wasm/src/solver/tests.rs`
- `wasm/src/solver/infer.rs`
- `wasm/src/solver/subtype.rs`
- `wasm/src/solver/evaluate.rs`

**Details:**
- Solver tests are commented out due to API drift from recent changes
- Tests cover critical type inference, subtyping, and evaluation logic
- Need to update test code to match current solver API
- These tests are essential for validating solver correctness

**Tasks:**
1. Identify all commented-out solver tests
2. Update test code to use current solver API
3. Fix any failing assertions
4. Ensure tests cover edge cases
5. Add to CI pipeline

**Acceptance Criteria:**
- [ ] All solver tests uncommented and passing
- [ ] Test coverage for infer/subtype/evaluate modules
- [ ] Tests added to CI pipeline
- [ ] No test regressions in future changes

---

### Team 2: Parser Accuracy (Tier 1)

**Engineering Manager:** Worker 6 (EM-2)
**Priority:** HIGH - Parser errors poison downstream analysis
**Tier:** 1 - Parser Accuracy

#### Worker 6 (EM-2): ASI Handling & EM Coordination
**Management Responsibilities:**
- Coordinate Team 2 parser work
- Run parser-specific conformance tests
- Focus on TS1xxx error codes
- Handle ASI issues as primary individual contributor work

**Individual Task:** ASI Specialist
**Issue:** Automatic Semicolon Insertion edge cases

**Key Files:**
- `wasm/src/thin_parser.rs`

**Tasks:**
1. Study TypeScript ASI rules
2. Identify current ASI failures
3. Implement proper ASI detection
4. Handle edge cases (return statements, postfix operators, etc.)
5. Add comprehensive ASI tests

**ASI Edge Cases:**
- `return\n{}` vs `return {};`
- Postfix `++`/`--`
- Anonymous function expressions
- `break`/`continue` without labels

**Acceptance Criteria:**
- [ ] ASI matches TypeScript behavior
- [ ] No extra TS1005 errors due to missing ASI
- [ ] Team coordination complete

---

#### Worker 7: TS1109 Parser Fix (Expression Expected)
**Task:** Fix parser emitting "Expression expected" for valid syntax

**Key Files:**
- `wasm/src/thin_parser.rs`

**Details:**
- Parser incorrectly emits TS1109 for valid TypeScript constructs
- Common cases: arrow functions, object destructuring, spread operators
- Run conformance tests and check "Extra Errors" for TS1109 codes

**Validation:**
```bash
node wasm/differential-test/find-ts1109.mjs
```

**Acceptance Criteria:**
- [ ] TS1109 only emitted for genuinely invalid expressions
- [ ] Valid syntax no longer triggers false TS1109 errors
- [ ] No regressions in other parser diagnostics

---

#### Worker 8: TS1005 Parser Fix (X Expected)
**Task:** Fix parser emitting "X expected" for valid constructs

**Key Files:**
- `wasm/src/thin_parser.rs`

**Details:**
- Parser incorrectly expects tokens that aren't required
- Common in optional chaining, nullish coalescing, and decorators

**Validation:**
```bash
node wasm/differential-test/find-ts1005.mjs
```

**Acceptance Criteria:**
- [ ] TS1005 only emitted for genuinely missing tokens
- [ ] Valid syntax no longer triggers false TS1005 errors
- [ ] No regressions in other parser diagnostics

---

#### Worker 9: Parser Conformance Testing & Analysis
**Task:** Analyze parser errors and create comprehensive test cases

**Management Support Tasks:**
1. Run conformance tests filtered for TS1xxx errors
2. Categorize parser errors by type and frequency
3. Create parser error priority list for EM
4. Validate fixes from Workers 7 and 8
5. Document parser behavior edge cases

**Key Files:**
- `wasm/src/thin_parser.rs`
- `wasm/differential-test/run-conformance.sh`

**Acceptance Criteria:**
- [ ] Complete parser error categorization
- [ ] Priority list created for team
- [ ] Test cases documented for all parser issues
- [ ] Validation framework established

---

### Team 3: Type Checker & Symbol Resolution (Tiers 2-3)

**Engineering Manager:** Worker 10 (EM-3)
**Priority:** MEDIUM - Core type checking accuracy
**Tier:** 2-3 - Type Checker & Symbol Resolution

#### Worker 10 (EM-3): Type Checker Coordination & TS2322
**Management Responsibilities:**
- Coordinate Tier 2 and Tier 3 work across 5 team members
- Manage dependencies between type checker and symbol resolution
- Run semantic error conformance tests
- Focus on TS2xxx and TS23xx/TS25xx error codes

**Individual Task:** Assignability Specialist
**Issue:** TS2322 accuracy - Type assignability false positives

**Key Files:**
- `wasm/src/solver/subtype.rs`
- `wasm/src/thin_checker.rs`

**Tasks:**
1. Study assignability logic in solver
2. Identify false positive TS2322 cases
3. Fix assignability checking
4. Handle edge cases (any, unknown, never, generics)
5. Verify subtype relationships

**Assignability Edge Cases:**
- Generic type compatibility
- Union/intersection types
- Structural vs nominal typing
- Literal types
- Discriminated unions

**Acceptance Criteria:**
- [ ] No TS2322 on valid assignments
- [ ] TS2322 still emitted for actual mismatches
- [ ] Team coordination complete

---

#### Worker 11: TS2683 This Type Handling
**Task:** Fix "'this' implicitly has type 'any'" not being emitted (TS2683)

**Key Files:**
- `wasm/src/thin_checker.rs` (around line 629, `current_this_type()` handling)

**Details:**
When `this` is used inside a regular function (not a method), it should emit TS2683 ("'this' implicitly has type 'any'") but currently types as `unknown` and emits TS2571 on property access.

**Example:**
```typescript
function foo() {
    this.x = 1;  // Should: TS2683, Currently: TS2571
}
```

**Tasks:**
1. Modify `current_this_type()` to return `any` instead of `unknown` for non-method functions
2. Emit TS2683 when `this` is used in contexts where it's implicitly typed as `any`
3. Ensure `this` type inference is correct in all function contexts
4. Coordinate with Worker 10 on assignability checks

**Acceptance Criteria:**
- [ ] TS2683 emitted for `this` in non-method functions
- [ ] Proper `this` type inference in all function contexts
- [ ] `this` behavior matches TypeScript compiler

---

#### Worker 12: TS2348 Callable Expression Fix
**Task:** Fix "Cannot invoke expression" (TS2348) being over-reported

**Key Files:**
- `wasm/src/thin_checker.rs`
- `wasm/src/solver/callable.rs`

**Details:**
- TS2348 is emitted for expressions that ARE callable
- Root cause likely in callable type resolution
- Need to properly check call signatures before emitting

**Acceptance Criteria:**
- [ ] TS2348 only emitted for genuinely non-callable expressions
- [ ] Callable type resolution correctly identifies callable types
- [ ] No regressions in call expression type checking

---

#### Worker 13: TS2507 Constructor Checking
**Task:** Non-constructor extends not fully checked

**Key Files:**
- `wasm/src/thin_checker.rs` - class declaration checking
- `wasm/src/checker/types/diagnostics.rs`

**Details:**
- Constructor inheritance rules need validation
- Identify gaps in non-constructor extends checking
- Implement proper constructor validation

**Test Cases:**
- Extending non-constructor values
- Extending built-in types
- Extending null/undefined
- Mixin patterns

**Acceptance Criteria:**
- [ ] All invalid extends emit TS2507
- [ ] Valid extends patterns work correctly
- [ ] Error messages match TSC

---

#### Worker 14: TS2304 Symbol Resolution Gaps
**Task:** Fix "Cannot find name" (TS2304) for valid symbols

**Key Files:**
- `wasm/src/binder/`
- `wasm/src/thin_binder.rs`
- `wasm/src/thin_checker.rs`

**Details:**
- Valid symbols not being resolved correctly
- May involve scope chain traversal issues
- Global symbols from lib.d.ts may not be merged correctly

**Acceptance Criteria:**
- [ ] All valid symbols in scope are resolvable
- [ ] Global/lib.d.ts symbols accessible where expected
- [ ] No false TS2304 errors for valid identifiers

---

## Deferred Work (Post-Tier 0/1/2/3)

These tasks are explicitly deferred until Tier 0, 1, 2, and 3 stabilize:

### Tier 4: Implicit Any Checks (Deferred)
- TS7006 extra - Parameter implicit any over-reported
- TS7005 extra - Variable implicit any over-reported

**Deferred Until:** Tier 0, 1, 2, and 3 are stable

### Tier 5: Async/Await (Deferred)
- TS2705 gaps - Async function return type checking
- TS1359 missing - 'await' reserved word detection
- Async generators - AsyncGenerator vs Promise return types

**Deferred Until:** All other tiers are stable

### LSP Strictness from tsconfig (Deferred)
- Hover information accuracy
- Completion suggestions
- Signature help
- Diagnostics filtering

**Deferred Until:** Tier 0 and Tier 1 are stable

---

## Priority Order Summary

```
1. Workers 2-5 (Tier 0): Quality & Stability Foundations
   └─ Blocks: All downstream accuracy
   └─ Team 1 (EM-1: Worker 2)

2. Workers 6-9 (Tier 1): Parser Accuracy
   └─ Blocks: Accurate AST for type checking
   └─ Team 2 (EM-2: Worker 6)

3. Workers 10-14 (Tier 2-3): Type Checker & Symbol Resolution
   └─ Core type checking and symbol accuracy
   └─ Team 3 (EM-3: Worker 10)

Deferred: Tier 4 (Implicit Any), Tier 5 (Async), LSP features
```

---

## Coordination Guidelines

### Engineering Manager Responsibilities
Each EM (Workers 2, 6, 10) should:
1. Run conformance tests to establish team baseline
2. Review all code changes from team members
3. Handle escalations and coordinate cross-team dependencies
4. Report team progress to Director (Worker 1)
5. Ensure no build breaks or test regressions

### Before Starting Work
1. Run `wasm-pack build --target web --out-dir pkg` to ensure WASM builds
2. Run conformance tests to establish baseline: `bash run-conformance.sh --max=200 --workers=4`
3. Read relevant spec documents in `wasm/specs/`
4. (EMs only) Review team task assignments and dependencies

### During Development
1. Make incremental commits with clear messages
2. Run unit tests after each change: `cd wasm && bash test.sh`
3. Do not modify files outside your task scope
4. Document any API changes that affect other workers
5. (EMs only) Conduct daily standup with team members

### Debugging Workflow
1. Find failing test case from conformance output
2. Create minimal reproduction in `/tmp/test.mjs`
3. Compare TSC vs WASM diagnostics
4. Trace through relevant code with logging if needed

### Build Commands
```bash
# Build WASM
cd wasm && wasm-pack build --target web --out-dir pkg

# Run unit tests
cd wasm && bash test.sh

# Run conformance tests
cd wasm/differential-test && bash run-conformance.sh --max=200 --workers=4
```

---

## Success Metrics

| Metric | Current | Target |
|--------|---------|--------|
| Exact Match | ~30% | 50%+ |
| Missing Errors | ~58% | <30% |
| Extra Errors | ~29% | <15% |
| Crashed | N/A | 0% |

---

## Communication Protocol

### Escalation Path
```
Individual Contributor (Workers 3-5, 7-9, 11-14)
    ↓ (blocked for >30 minutes)
Engineering Manager (Workers 2, 6, 10)
    ↓ (cross-team issues)
Director (Worker 1)
```

### Commit Message Format
```
[Tier X] Brief description of change

- Details of what was changed
- Why it was necessary
- Related issue number (if applicable)

Co-Authored-By: Claude Sonnet 4.5 <noreply@anthropic.com>
```

### Dependencies
- Coordinate with other workers if your changes affect shared files
- EMs should manage cross-team dependencies
- All changes merge to `rust` branch after review

### Daily Standup Format (EMs)
Each team member should report:
1. Tasks completed yesterday
2. Tasks planned for today
3. Blockers or dependencies
4. Conformance test results (if applicable)

---

*Generated by Director (Worker 1) at 2026-01-16*
*Mode: hierarchical - 3 Engineering Managers managing 11 Engineers*
