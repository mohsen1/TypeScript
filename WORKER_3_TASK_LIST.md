# WORKER-3 TASK LIST

## Squad: Syntax Squad
## EM: EM-1
## Branch: worker-3

---

## Primary Task: Fix TS1109 "Expression Expected" Noise

**Priority:** 🔴 CRITICAL (Priority 1 for EM-1)
**Assigned:** 2026-01-15
**Status:** 🔵 STARTING

### Problem

The parser emits **TS1109 "Expression expected"** errors for valid TypeScript syntax or fails to recover gracefully after missing expressions. This creates noise that poisons downstream semantic analysis.

**Current Impact:** ~262 extra errors in conformance tests

### Root Cause

The `error_expression_expected()` function in `thin_parser.rs` is called in situations where:
1. Valid syntax is incorrectly rejected (false positive)
2. Parser doesn't recover after missing expression, causing cascading errors
3. Error recovery sync points are insufficient

### Action Items

#### Phase 1: Investigation

1. **Study Worker 5's TS1005 suppression logic**
   - Review commit history for `ts1005_statement_budget` implementation
   - Understand proximity-based error suppression
   - Apply similar patterns to TS1109

2. **Identify false positive patterns**
   - Find test cases with TS1109 on valid syntax
   - Categorize: ASI-related, statement boundaries, edge cases
   - Document patterns for suppression

#### Phase 2: Implementation

1. **Add `is_at_expression_end()` check before emitting TS1109**
   - Reuse Worker 5's `is_at_expression_end()` helper
   - Suppress TS1109 when parser is at natural expression end
   - Reduces noise for cases like `let x = ;`

2. **Implement statement-level budget**
   - Track TS1109 errors per statement (similar to TS1005)
   - Limit to 2 TS1109 errors per statement
   - Reset budget at statement boundaries

3. **Enhance error recovery**
   - Improve `resync_after_error()` for expression contexts
   - Add synchronization points: semicolons, closing braces, keywords
   - Continue parsing after missing expression

#### Phase 3: Validation

1. **Test with malformed syntax**
   ```typescript
   // Should recover without cascading errors
   let x = ;
   const y = function() { return ; };
   ```

2. **Run conformance tests**
   ```bash
   cd wasm/differential-test
   bash run-conformance.sh --max=500 --workers=4
   ```
   - Track TS1109 count (target: reduce from 262 to <40)
   - Ensure no regression in valid syntax detection

3. **Compare with Worker 5's TS1005 work**
   - Apply same suppression patterns
   - Ensure consistency in error handling

### Files to Work On
- `wasm/src/thin_parser.rs` - `error_expression_expected()` around line 600-700
- `wasm/src/thin_parser.rs` - `is_at_expression_end()` helper (add if missing)
- `wasm/src/thin_parser.rs` - `resync_after_error()` function

### Success Criteria
- **TS1109 extra errors:** Reduce from 262 to <40
- **Parser recovery:** Continues after missing expression
- **No regression:** Valid syntax still accepted

### Testing
1. Create test file with missing expressions
2. Verify parser recovers and continues
3. Run conformance suite before/after
4. Document error count reduction

---

## Reference: Worker 5's TS1005 Work

Worker 5 successfully implemented similar suppression for TS1005:
- **Per-statement budget:** 2 TS1005 errors per statement
- **Proximity suppression:** 80 character threshold
- **Expression end detection:** `is_at_expression_end()` helper

**Apply these patterns to TS1109.**

---

## Instructions

1. Sync with em-team-1: `git pull origin em-team-1`
2. Create feature branch from em-team-1
3. Work on TS1109 suppression ONLY
4. Commit frequently: `[wasm] parser: add TS1109 expression end detection`
5. Push to worker-3 branch
6. Run tests locally
7. Update this task list with status
8. Notify EM-1 when ready for merge

---

## Validation Checklist Before Merge

- [ ] TS1109 errors reduced by target amount (262 → <40)
- [ ] Parser recovers after missing expression
- [ ] No regression in valid syntax detection
- [ ] Conformance tests pass
- [ ] Code follows Worker 5's suppression patterns
- [ ] Minimal repro tests validate fix

---

## Task Completion Report

**Status:** 🟡 Active - Task Assigned

**Date:** 2026-01-15

**Commits:** None yet (task assignment)

**Changes Made:**
- Worker-3 merged into em-team-1
- Task list updated with TS1109 assignment

**Results:**
- Worker-3 has been assigned TS1109 "Expression Expected" suppression task
- Target: Reduce TS1109 errors from 262 to <40
- Task involves implementing error suppression patterns similar to Worker 5's TS1005 work

**Note:** This task differs from original EM-1 plan (TS7006/TS7005). Worker-3 is now in Syntax Squad working on parser accuracy instead of AnyCheck Squad.
