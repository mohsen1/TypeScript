# EM_2 Tasks - Parser/Scanner Squad (PRIORITY)

**Branch:** `em-team-2`
**Priority:** 🟠 HIGH
**Assigned Workers:** workers 5-6 (incoming: workers 1-3 from EM_1 after validation)
**Last Updated:** 2026-01-14 (Director update - awaiting transfer)

---

## Squad Mission

Fix the **701 false positive parser errors** that are polluting conformance measurements. When the parser over-reports errors, it masks real progress and inflates "Extra Errors" by 14%.

---

## Problem Statement

**Error Codes:** TS1005, TS1109

**Current State:**
- **TS1005 ("expected X"):** 439 false positives
- **TS1109 ("expression expected"):** 262 false positives
- **Total:** 701 false positives

**Root Cause:**
The parser is likely:
1. Too strict on valid TypeScript syntax edge cases
2. Not implementing proper error recovery (resynchronization)
3. Emitting errors on syntax that `tsc` accepts

**Impact:**
- Inflates "Extra Errors" metric by 14%
- Masks actual semantic progress
- Incomplete AST leads to missing symbols (cascading into Binder failures)

---

## Immediate Goals

1. **Audit TS1005 Emission**
   - Find where "expected X" is emitted
   - Compare with `tsc` behavior on same test cases
   - Identify over-triggering conditions

2. **Audit TS1109 Emission**
   - Find "expression expected" emission points
   - Check for false positives on valid edge cases
   - Verify lookahead/parser state isn't bailing early

3. **Improve Error Recovery**
   - Implement resynchronization after syntax errors
   - Keep parsing to complete the AST even with errors
   - Don't let one error poison the rest of the file

4. **Target Metric:** Reduce parser false positives to **<100**

---

## Key Files to Investigate

| File | Purpose | Action |
|------|---------|--------|
| `src/compiler/parser.ts` | Main parser logic | Audit TS1005/TS1109 emission |
| `src/scanner.ts` (if exists) | Lexical analysis | Check tokenization edge cases |
| `src/error_recovery.ts` (if exists) | Error recovery | Implement/improve resynchronization |

**Note:** Workers 1-3 have already implemented fixes in `src/compiler/parser.ts`:
- Worker 1: TS1005 patterns 1-5 (semicolon handling, return type arrow functions)
- Worker 2: TS1109 definite assignment assertions
- Worker 3: Cascading error tracking with `last_error_pos`

---

## Worker Assignment Strategy

| Worker | Focus Area | Status |
|--------|-----------|--------|
| worker-1 | TS1005 patterns 1-5 (DONE, needs validation) | Incoming from EM_1 |
| worker-2 | TS1109 definite assignment (DONE, needs validation) | Incoming from EM_1 |
| worker-3 | Cascading error tracking (DONE, needs validation) | Incoming from EM_1 |
| worker-5 | TS1005 investigation and fixes | Current |
| worker-6 | TS1109 investigation and fixes | Current |
| (Shared) | Error recovery implementation | All workers |

**Next Steps for EM_2:**
1. Await EM-1 validation cycle completion
2. Accept transfer of workers 1-3
3. Validate their fixes (run conformance tests)
4. Continue with remaining TS1005/TS1109 patterns

---

## Escalation Triggers

Escalate to Director if:
- Parser false positives < 100 (mission complete)
- Need architectural changes to parser design
- Team size exceeds 4 (need team split)

---

## Success Criteria

- [ ] TS1005 false positives < 50
- [ ] TS1109 false positives < 50
- [ ] Total parser false positives < 100
- [ ] Error recovery keeps parsing after syntax errors
- [ ] No regression in valid syntax parsing

---

## Notes

- **Do NOT** modify worker task lists directly
- Workers should create their own task breakdown
- Coordinate with EM_1 (Binder) as parser fixes may affect symbol binding
- Existing analysis docs: `TS1005_REDUCTION_RESULTS.md`, `TS1109_ANALYSIS.md`
