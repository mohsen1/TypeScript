# EM_2 Tasks - Parser/Scanner Squad (PRIORITY)

**Branch:** `em-team-2`
**Priority:** 🟠 HIGH
**Assigned Workers:** workers 5-6
**Last Updated:** 2026-01-14

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
| `src/thin_parser.rs` | Main parser logic | Audit TS1005/TS1109 emission |
| `src/scanner.rs` | Lexical analysis | Check tokenization edge cases |
| `src/error_recovery.rs` | Error recovery | Implement/improve resynchronization |

---

## Worker Assignment Strategy

| Worker | Focus Area |
|--------|-----------|
| worker-5 | TS1005 investigation and fixes |
| worker-6 | TS1109 investigation and fixes |
| (Shared) | Error recovery implementation |

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
