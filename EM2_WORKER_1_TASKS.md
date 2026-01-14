# EM-2 Worker 1 Task List

## Squad: Parser (TS1005 - Comma & Statement Focus)
**Branch:** worker-1
**Status:** Transferred from EM-1 after successful validation ✅
**Last Updated:** 2026-01-14

---

## EM-1 Accomplishments
- ✅ Fixed TS1005 patterns 1-5: -127 errors (-29%)
- ✅ Exact Match improvement: +1.1%
- ✅ All builds passing, no regressions

---

## EM-2 Assignment

### Current Task: Comma Inference in Object/Array Literals

**Priority Target:** ~85 cases to eliminate

#### Task 1: Object Literal Comma Inference (~50 cases)
- [ ] Investigate how parser handles missing commas in object literals
- [ ] `{a: 1 b: 2}` should recover on missing comma instead of emitting TS1005
- [ ] Improve ASI detection in object literal parsing
- [ ] Add tests for comma recovery scenarios
- [ ] Run conformance to measure impact

#### Task 2: Array Literal Comma Inference (~35 cases)
- [ ] Investigate array literal parsing with missing elements
- [ ] `[1, 2 3]` should recover gracefully
- [ ] Coordinate with Worker 3's cascading error fix

#### Task 3: Statement Termination Edge Cases (~38 cases)
- [ ] Class methods, getters/setters semicolon handling
- [ ] Semicolon vs ASI confusion in specific contexts
- [ ] Return type parsing edge cases

### Queue (After comma inference complete)
- [ ] Fix type parameter parsing edge cases
- [ ] Coordinate with Worker 3 on bracket recovery
- [ ] Address remaining TS1005 edge cases

---

## Success Criteria
- Reduce TS1005 from 267 to <100 (need 167 more reductions)
- No regressions in TS1109 or other error codes
- Build passes all tests
- Conformance shows measurable improvement

---

## Handoff Notes from EM-1
**Patterns Already Fixed (EM-1):**
1. Property semicolon handling (parseSemicolonAfterPropertyName)
2. Return type arrow function confusion (shouldParseReturnType)
3. Object literal element error recovery
4. Array literal element parsing edge cases

**Key Learning:** Worker 3's cascading error fix amplified our results by 45 additional reductions. Always test with Worker 3's fix enabled.

---

## Next Immediate Actions
1. Run conformance to establish EM-2 baseline (TS1005 should be ~267)
2. Start with object literal comma inference (highest impact)
3. Coordinate with Worker 3 on testing with cascading error suppression
