# EM-2 Worker 2 Task List

## Squad: Parser (TS1109 - new.target & Templates Focus)
**Branch:** worker-2
**Status:** Transferred from EM-1 after successful validation ✅
**Last Updated:** 2026-01-14

---

## EM-1 Accomplishments
- ✅ Fixed TS1109 definite assignment assertions: -64 errors (-24%)
- ✅ Exact Match improvement: +0.4%
- ✅ All builds passing, no regressions

---

## EM-2 Assignment

### Current Task: new.target Context Validation

**Priority Target:** ~52 cases (HIGH IMPACT)

#### Task 1: new.target Context Validation (~52 cases)
- [ ] Investigate parser's new.target handling
- [ ] Parser emits TS1109 when `new.target` used outside constructor
- [ ] Implement better context tracking for new.target
- [ ] Add tests for new.target in valid contexts (static methods, class fields)
- [ ] Run conformance to measure impact

### Queue (After new.target complete)

#### Task 2: Template Strings in Type Positions (~14 cases)
- [ ] Investigate template string parsing in type annotations
- [ ] Fix edge cases with complex ${} nesting

#### Task 3: Private Names in `in` Expressions (~9 cases)
- [ ] Fix private name syntax in `in` expressions

#### Task 4: Remaining Destructuring Edge Cases (~4 cases)
- [ ] Address any remaining destructuring patterns

---

## Success Criteria
- Reduce TS1109 from 122 to <100 (need 22 more reductions)
- No regressions in TS1005 or other error codes
- Build passes all tests
- Conformance shows measurable improvement

---

## Handoff Notes from EM-1
**Patterns Already Fixed (EM-1):**
1. Variable declarations: `var t!: T;`
2. Object shorthand properties: `{ a! }`
3. Computed property type parameters
4. Array destructuring with annotations

**Key Learning:** Worker 3's cascading error fix will help with ~35 of our remaining cases by eliminating false positive cascades.

---

## Next Immediate Actions
1. Run conformance to establish EM-2 baseline (TS1109 should be ~122)
2. Start with new.target context validation (highest impact)
3. Coordinate with Worker 3 on testing with cascading error suppression
