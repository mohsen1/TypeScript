# Team 3 Coordination Plan

**Engineering Manager:** Worker 4 (acting for EM-3)
**Team Focus:** Tier 2-3: Type Checker & Symbol Resolution
**Priority:** MEDIUM - Core type checking accuracy
**Created:** 2026-01-16

---

## Team 3 Overview

Team 3 is responsible for fixing semantic type checking errors in the TS2xxx, TS23xx, and TS25xx error code ranges. These are core type system correctness issues that affect the accuracy of type inference, assignability, and symbol resolution.

### Team Members

| Worker # | Focus Area | Error Codes | Priority |
|----------|------------|-------------|----------|
| Worker 10 (EM-3) | TS2322 Assignability | TS2322 | HIGH |
| Worker 11 | This Type Handling | TS2571, TS2683 | HIGH |
| Worker 12 | Callable Expressions | TS2348 | MEDIUM |
| Worker 13 | Constructor Checking | TS2507 | MEDIUM |
| Worker 14 | Symbol Resolution | TS2304 | HIGH |

---

## Task Breakdown and Dependencies

### 1. Worker 10 (EM-3) - TS2322 Assignability Issues

**Task:** Fix type assignability false positives

**Key Files:**
- `wasm/src/solver/subtype.rs` - Subtype checking logic
- `wasm/src/thin_checker.rs` - Type checking integration

**Impact:** Blocks all type correctness validation

**Dependencies:**
- Depends on: Worker 2 (Team 1) - Type expansion fixes
- Blocks: All other type checking work

**Acceptance Criteria:**
- [ ] No TS2322 on valid assignments
- [ ] TS2322 still emitted for actual mismatches
- [ ] Generic type compatibility works correctly
- [ ] Union/intersection type assignability correct
- [ ] Literal type assignability works
- [ ] Discriminated union assignability works

**Validation Script:**
```bash
node wasm/differential-test/find-extra-ts2322.mjs
node wasm/differential-test/find-missing-ts2322.mjs
node wasm/differential-test/analyze-ts2322.mjs
```

---

### 2. Worker 11 - TS2571/TS2683 This Type Fix

**Task:** Fix "'this' implicitly has type 'any'" not being emitted (TS2683), instead incorrectly emitting TS2571

**Key Files:**
- `wasm/src/thin_checker.rs` (around line 629, `current_this_type()` handling)

**Issue:**
When `this` is used inside a regular function (not a method), it should emit TS2683 but instead types as `unknown` and emits TS2571 on property access.

**Example:**
```typescript
function foo() {
    this.x = 1;  // Should: TS2683, Currently: TS2571
}
```

**Impact:** Affects method/function type accuracy

**Dependencies:**
- Depends on: Worker 14 - Symbol resolution for function contexts
- Related to: Worker 10 - Assignability checking

**Acceptance Criteria:**
- [ ] TS2683 emitted for `this` in non-method functions
- [ ] TS2571 no longer incorrectly emitted for `this` access
- [ ] Proper `this` type inference in all function contexts
- [ ] Method `this` types work correctly
- [ ] Arrow function `this` capture works

**Test Cases:**
```typescript
// Regular function - should emit TS2683
function regular() {
    this.x;  // TS2683
}

// Method - should work
class C {
    method() {
        this.x;  // OK
    }
}

// Arrow function - should capture outer this
const arrow = () => {
    this.x;  // Depends on context
};
```

---

### 3. Worker 12 - TS2348 Callable Expression Fix

**Task:** Fix "Cannot invoke expression" (TS2348) being over-reported

**Key Files:**
- `wasm/src/thin_checker.rs` - Call expression checking
- `wasm/src/solver/callable.rs` - Callable type resolution

**Issue:**
TS2348 is emitted for expressions that ARE callable, likely due to issues in callable type resolution.

**Impact:** False errors on valid function calls

**Dependencies:**
- Depends on: Worker 14 - Symbol resolution for function references
- Related to: Worker 10 - Type compatibility for call signatures

**Acceptance Criteria:**
- [ ] TS2348 only emitted for genuinely non-callable expressions
- [ ] Callable type resolution correctly identifies callable types
- [ ] No regressions in call expression type checking
- [ ] Function calls work correctly
- [ ] Method calls work correctly
- [ ] Constructor calls work correctly

**Test Cases:**
```typescript
// Function call - should work
function foo() {}
foo();  // OK

// Method call - should work
class C {
    method() {}
}
new C().method();  // OK

// Non-callable - should emit TS2348
const x = 42;
x();  // TS2348
```

---

### 4. Worker 13 - TS2507 Constructor Checking

**Task:** Non-constructor extends not fully checked

**Key Files:**
- `wasm/src/thin_checker.rs` - Class declaration checking
- `wasm/src/checker/types/diagnostics.rs`

**Issue:**
Constructor inheritance rules need validation. Need to identify gaps and implement proper validation.

**Impact:** Invalid class extends may not be caught

**Dependencies:**
- Depends on: Worker 12 - Callable type resolution for constructors
- Related to: Worker 10 - Type compatibility for inheritance

**Acceptance Criteria:**
- [ ] All invalid extends emit TS2507
- [ ] Valid extends patterns work correctly
- [ ] Error messages match TSC
- [ ] Extending non-constructors detected
- [ ] Extending built-in types validated
- [ ] Extending null/undefined rejected
- [ ] Mixin patterns supported

**Test Cases:**
```typescript
// Invalid - extending non-constructor
class A extends 42 {}  // TS2507

// Invalid - extending null/undefined
class B extends null {}  // TS2507

// Valid - class extends
class C {}
class D extends C {}  // OK

// Valid - mixin pattern
function mixin(Base) {
    return class extends Base {}  // OK
}
```

---

### 5. Worker 14 - TS2304 Symbol Resolution Gaps

**Task:** Fix "Cannot find name" (TS2304) for valid symbols

**Key Files:**
- `wasm/src/binder.rs`
- `wasm/src/thin_binder.rs`
- `wasm/src/thin_checker.rs`

**Issue:**
Valid symbols not being resolved correctly. May involve scope chain traversal issues or global/lib.d.ts symbol merging.

**Impact:** False TS2304 errors block all type checking

**Dependencies:**
- Depends on: Worker 4 (Team 1) - AST traversal fixes
- Blocks: All other Team 3 work (most critical dependency)

**Acceptance Criteria:**
- [ ] All valid symbols in scope are resolvable
- [ ] Global/lib.d.ts symbols accessible where expected
- [ ] No false TS2304 errors for valid identifiers
- [ ] Scope chain traversal works correctly
- [ ] Module imports are resolved
- [ ] Variable declarations are accessible
- [ ] Function/class names are in scope

**Validation Script:**
```bash
node wasm/differential-test/find-ts2304.mjs
node wasm/differential-test/analyze-extra-ts2304.mjs
```

---

## Priority Order

Based on dependencies and impact:

### Phase 1: Foundation (Week 1)
1. **Worker 14** - TS2304 Symbol Resolution (BLOCKS all other work)
2. **Worker 10** - TS2322 Assignability (Core type system)

### Phase 2: This Type & Callables (Week 2)
3. **Worker 11** - TS2571/TS2683 This Type (depends on Worker 14)
4. **Worker 12** - TS2348 Callable Expressions (depends on Worker 14)

### Phase 3: Constructor Validation (Week 2)
5. **Worker 13** - TS2507 Constructor Checking (depends on Worker 12)

---

## Build and Test Commands

### Initial Setup (All Workers)

```bash
# Navigate to wasm directory
cd wasm

# Build WASM package
wasm-pack build --target web --out-dir pkg

# Run unit tests
bash test.sh

# Navigate to differential test directory
cd differential-test
```

### Running Conformance Tests

```bash
# Full conformance test suite
bash run-conformance.sh --max=500 --workers=4

# Quick smoke test
bash run-conformance.sh --max=50 --workers=2
```

### Team 3 Specific Validation

```bash
# TS2322 (Worker 10)
node wasm/differential-test/find-extra-ts2322.mjs
node wasm/differential-test/find-missing-ts2322.mjs
node wasm/differential-test/analyze-ts2322.mjs

# TS2304 (Worker 14)
node wasm/differential-test/find-ts2304.mjs
node wasm/differential-test/analyze-extra-ts2304.mjs

# TS2564 (Related to TS2571)
node wasm/differential-test/find-ts2564.mjs
```

---

## Daily Standup Format

Each team member should report:

1. **Completed Yesterday:**
   - What tasks were completed
   - What tests were run and results
   - Commits made

2. **Planned Today:**
   - What tasks will be worked on
   - What tests will be run
   - Expected deliverables

3. **Blockers:**
   - Any dependencies not met
   - Any technical issues encountered
   - Need for coordination with other teams

4. **Test Results:**
   - Conformance test results (if applicable)
   - Extra/missing error counts
   - Any regressions found

---

## Escalation Path

```
Individual Contributor (Workers 10-14)
    ↓ (blocked for >30 minutes)
Engineering Manager (Worker 4 - EM-3)
    ↓ (cross-team issues)
Director (Worker 1)
```

---

## Coordination with Other Teams

### Team 1 (Quality & Stability) - EM-1 (Worker 2)
- **Dependencies:**
  - Worker 4 (AST traversal) → Worker 14 (Symbol resolution)
  - Worker 2 (Type expansion) → Worker 10 (Assignability)

### Team 2 (Parser Accuracy) - EM-2 (Worker 6)
- **Minimal dependencies** - Team 3 works on semantic analysis after parsing

---

## Success Metrics

| Metric | Current | Target | Notes |
|--------|---------|--------|-------|
| TS2322 Extra | TBD | <50 | Assignability false positives |
| TS2304 Extra | TBD | <100 | Symbol resolution issues |
| TS2571 Extra | TBD | <20 | This type issues |
| TS2348 Extra | TBD | <20 | Callable expression issues |
| TS2507 Missing | TBD | 0 | Constructor validation gaps |

---

## Code Review Guidelines

### Before Submitting for Review:

1. **Build Verification:**
   ```bash
   cd wasm && wasm-pack build --target web --out-dir pkg
   ```

2. **Unit Tests:**
   ```bash
   cd wasm && bash test.sh
   ```

3. **Conformance Tests:**
   ```bash
   cd wasm/differential-test
   bash run-conformance.sh --max=100 --workers=2
   ```

4. **Error Code Validation:**
   - Run appropriate validation scripts for your error code
   - Document extra/missing errors
   - Compare against TypeScript baseline

### Review Checklist:

- [ ] Code builds without errors
- [ ] Unit tests pass
- [ ] No regressions in conformance tests
- [ ] Error messages match TSC
- [ ] Test cases added for fixes
- [ ] Documentation updated (if needed)
- [ ] Commit message follows format

---

## Commit Message Format

```
[Tier 2/3] Brief description of change

- Details of what was changed
- Why it was necessary
- Related issue (error code, test case)
- Test results (extra/missing error counts)

Co-Authored-By: Claude Sonnet 4.5 <noreply@anthropic.com>
```

Example:
```
[Tier 2] Fix TS2683 not emitted for 'this' in regular functions

- Added check for non-method function context
- Modified current_this_type() to emit TS2683
- Removed incorrect TS2571 emission for 'this' access
- Added test cases for this type handling

Fixes: TS2571/TS2683
Extra errors: -15
Missing errors: +8

Co-Authored-By: Claude Sonnet 4.5 <noreply@anthropic.com>
```

---

## Key Reference Documents

- `wasm/README.md` - Project overview
- `wasm/TESTING.md` - Testing guidelines
- `TEAM_STRUCTURE.md` - Overall project organization
- `TS2304_INVESTIGATION.md` - TS2304 specific analysis
- `TS2571_investigation.md` - This type investigation

---

## Next Steps for Workers

1. **Worker 14 (TS2304):** Start immediately - highest priority, blocks others
2. **Worker 10 (TS2322):** Start after Worker 14 shows progress
3. **Workers 11, 12, 13:** Wait for go-ahead from EM after foundation is laid

---

## Status Tracking

- **Created:** 2026-01-16
- **Last Updated:** 2026-01-16
- **Phase:** Planning/Setup
- **Next Milestone:** Worker 14 (TS2304) baseline established

---

*Document created by Worker 4 (EM-3) at 2026-01-16*
