# Worker-10 Task List

**Squad:** Syntax Support
**Branch:** `worker-10`
**EM:** EM-3
*Assigned: 2026-01-14*

---

## Priority Mission

Fix parser edge cases to support TS1005/TS1109 reduction efforts. **Target: Reduce edge case parser failures by 50%.**

---

## Assigned Tasks

### 1. Audit ASI (Automatic Semicolon Insertion) Edge Cases
**Priority:** P0 - High
**File:** `wasm/src/parser/`

**Focus Areas:**
1. Line terminators before `++`/`--`
2. `return`, `throw`, `yield` statements without semicolons
3. `break`, `continue` with labels
4. Arrow functions with block-less bodies

### 2. Fix Complex Synchronization Points
**Priority:** P1
**File:** `wasm/src/parser/`

**Tasks:**
1. Improve recovery after unexpected tokens in class bodies
2. Handle `interface` declarations with malformed extends clauses
3. Recover from errors in template literal expressions
4. Handle object destructuring patterns with missing commas

### 3. Support Worker-1/Worker-5 Parser Noise Efforts
**Priority:** P1
**Tasks:**
1. Run conformance tests to identify remaining TS1005/TS1109 patterns
2. Categorize by syntactic context (statement/declaration/expression)
3. Report findings to EM-1 and EM-2 teams
4. Implement fixes for edge cases not covered by main parser work

---

## Success Criteria
- [ ] ASI edge cases identified and documented
- [ ] Parser recovery improved in complex contexts
- [ ] Support EM-1/EM-2 with categorized error patterns
- [ ] Edge case failure rate reduced by 50%

---

## Status
**Status:** 🔄 IN PROGRESS
**Assigned:** 2026-01-14
