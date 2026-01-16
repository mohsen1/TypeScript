# Project Zang - Team Structure and Work Distribution

## Overview

This document outlines the work distribution plan for 14 workers on Project Zang, a TypeScript compiler rewrite in Rust/WASM. Tasks are organized by priority tier following the project's established guidelines.

---

## Team Organization

### Engineering Managers

| EM | Team Focus | Workers |
|----|------------|---------|
| EM-1 | Tier 0: Quality & Stability | Workers 2, 3, 4 |
| EM-2 | Tier 1-2: Parser & Checker | Workers 5, 6, 7, 8 |
| EM-3 | Tier 3-4: Symbol Resolution & Implicit Any | Workers 9, 10, 11 |
| EM-4 | Tier 5: Async/Await & Infrastructure | Workers 12, 13, 14 |

---

## Worker Assignments

### Team 1: Quality & Stability Foundations (Tier 0)

**Engineering Manager:** EM-1
**Priority:** Highest - These issues block correctness across all tiers

#### Worker 2: Application Type Expansion
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

### Team 2: Parser & Type Checker Accuracy (Tiers 1-2)

**Engineering Manager:** EM-2
**Priority:** High - Parser errors poison downstream analysis

#### Worker 5: TS1109 Parser Fix (Expression Expected)
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

#### Worker 6: TS1005 Parser Fix (X Expected)
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

#### Worker 7: TS2571/TS2683 This Type Fix
**Task:** Fix "'this' implicitly has type 'any'" not being emitted (TS2683), instead incorrectly emitting TS2571

**Key Files:**
- `wasm/src/thin_checker.rs` (around line 629, `current_this_type()` handling)

**Details:**
When `this` is used inside a regular function (not a method), it should emit TS2683 but instead types as `unknown` and emits TS2571 on property access.

**Example:**
```typescript
function foo() {
    this.x = 1;  // Should: TS2683, Currently: TS2571
}
```

**Acceptance Criteria:**
- [ ] TS2683 emitted for `this` in non-method functions
- [ ] TS2571 no longer incorrectly emitted for `this` access
- [ ] Proper `this` type inference in all function contexts

---

#### Worker 8: TS2348 Callable Expression Fix
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

### Team 3: Symbol Resolution & Implicit Any (Tiers 3-4)

**Engineering Manager:** EM-3
**Priority:** Medium - Required for accurate type checking

#### Worker 9: TS2304 Symbol Resolution Gaps
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

#### Worker 10: TS2524 Module Member Resolution
**Task:** Fix module member resolution failures (TS2524)

**Key Files:**
- `wasm/src/binder/`
- `wasm/src/thin_checker.rs`

**Details:**
- Module exports not being properly resolved
- May involve namespace/module merging issues
- Check re-export handling

**Acceptance Criteria:**
- [ ] Module members correctly resolved
- [ ] Re-exports work correctly
- [ ] Namespace merging functions properly

---

#### Worker 11: Implicit Any Over-reporting (TS7006/TS7005)
**Task:** Fix implicit any errors being over-reported

**Key Files:**
- `wasm/src/thin_checker.rs` (implicit any checking functions)

**Details:**
Skip implicit any errors when:
- Parameter has default value (`param.initializer.is_some()`)
- Property has initializer (`prop.initializer.is_some()`)
- Type can be inferred from usage

**Acceptance Criteria:**
- [ ] Parameters with default values don't trigger TS7006
- [ ] Properties with initializers don't trigger TS7005
- [ ] Contextually typed parameters don't trigger TS7006

---

### Team 4: Async/Await & Infrastructure (Tier 5)

**Engineering Manager:** EM-4
**Priority:** Lower - Focus after core accuracy

#### Worker 12: Async Function Return Types (TS2705)
**Task:** Fix async function return type checking gaps

**Key Files:**
- `wasm/src/thin_checker.rs` (async-related functions)
- `wasm/src/solver/evaluate.rs`

**Details:**
- Async function return types need to be wrapped in Promise
- Generator return types need special handling
- `AsyncGenerator` vs `Promise` return type disambiguation

**Acceptance Criteria:**
- [ ] Async functions correctly typed with Promise wrapper
- [ ] Return type checking works for async functions
- [ ] No false positives on valid async returns

---

#### Worker 13: Await Reserved Word Detection (TS1359)
**Task:** Implement 'await' reserved word detection

**Key Files:**
- `wasm/src/thin_parser.rs`
- `wasm/src/thin_checker.rs`

**Details:**
- `await` is a reserved word in certain contexts
- Must detect invalid use of `await` as identifier
- Context-sensitive based on module type and function context

**Acceptance Criteria:**
- [ ] TS1359 emitted for invalid `await` usage
- [ ] No false positives in valid async contexts
- [ ] Module/function context correctly considered

---

#### Worker 14: Solver Test Coverage Restoration
**Task:** Re-enable solver tests that are commented out due to API drift

**Key Files:**
- `wasm/src/solver/infer.rs` (tests)
- `wasm/src/solver/subtype.rs` (tests)
- `wasm/src/solver/evaluate.rs` (tests)

**Details:**
- Tests are commented out because APIs have changed
- Update test code to match current APIs
- Ensure tests pass and provide coverage

**Acceptance Criteria:**
- [ ] All commented-out solver tests re-enabled
- [ ] Tests updated to match current API
- [ ] All solver tests pass

---

## Priority Order Summary

```
1. Workers 2-4 (Tier 0): Quality & Stability Foundations
   └─ Blocks: All downstream accuracy

2. Workers 5-6 (Tier 1): Parser Accuracy
   └─ Blocks: Accurate AST for type checking

3. Workers 9-10 (Tier 3): Symbol Resolution
   └─ Blocks: Accurate type references

4. Workers 7-8, 11 (Tier 2, 4): Type Checker & Implicit Any
   └─ Core type checking accuracy

5. Workers 12-14 (Tier 5): Async/Await & Infrastructure
   └─ Feature completeness
```

---

## Coordination Guidelines

### Before Starting Work
1. Run `wasm-pack build --target web --out-dir pkg` to ensure WASM builds
2. Run conformance tests to establish baseline: `bash run-conformance.sh --max=200 --workers=4`
3. Read relevant spec documents in `wasm/specs/`

### During Development
1. Make incremental commits with clear messages
2. Run unit tests after each change: `cd wasm && bash test.sh`
3. Do not modify files outside your task scope
4. Document any API changes that affect other workers

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

- **Escalations:** Contact EM if blocked for >30 minutes
- **Dependencies:** Coordinate with other workers if your changes affect shared files
- **Merges:** All changes merge to `rust` branch after review

---

*Generated by Director (Worker 1) at 2026-01-16*
