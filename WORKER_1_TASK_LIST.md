# WORKER-1 TASK LIST

## Team Assignment
- **EM:** EM-1
- **Branch:** worker-1
- **Parent:** em-team-1

## Status: Priority #1 COMPLETE ✅

### Previous Task: Parser Noise Elimination - COMPLETE
**Achievement:** Reduced parser noise from 701 to 25 errors (96.4% reduction)
- Merged to rust: commit `2e7f46ff75`
- Target: <40 | Achieved: 25 ✅
- TS1005: 439 → ~20 (95.4% reduction)
- TS1109: 262 → ~5 (98.1% reduction)

---

## Current Task: Push ASI Improvements

### Mission
Push enhanced ASI detection work to rust for final parser refinements.

### Current Work (on worker-1, not yet on rust)
- **Commit:** `ae54d443c2` Complete: Enhanced Expression Statement ASI Detection
- **File:** `TS1005_PATTERN_ANALYSIS.md` - Pattern analysis document
- **Enhancement:** Expanded `is_statement_start()` to include:
  - Expression literals: NumericLiteral, BigIntLiteral, TrueKeyword, FalseKeyword, NullKeyword, ThisKeyword, SuperKeyword
  - Prefix operators: ExclamationToken, TildeToken, PlusToken, MinusToken, PlusPlusToken, MinusMinusToken
  - Keywords: TypeOfKeyword, VoidKeyword, DeleteKeyword
  - Structural: OpenParenToken, OpenBracketToken, LessThanToken

### Action Items
1. **Push current work to origin** - Ensure ASI improvements are available
2. **Build WASM and test** - Verify no regressions from ASI changes
3. **Measure baseline** - Check if TS1005 reduced further with new ASI logic
4. **Report results** - Document impact of enhanced ASI

### Success Metric
- ASI improvements merged to rust
- No regressions in conformance tests
- Further reduction in TS1005 false positives (optional bonus)

---

## Next Priority Assignment (TBD)

With Priority #1 (Parser Noise) complete, Worker-1 will be assigned to:
- **Option A:** Support Priority #2 (Global Scope) - Help with lib.d.ts integration
- **Option B:** Support Priority #3 (Solver Defaults) - Help with ERROR type propagation
- **Option C:** Parser refinements - Continue reducing remaining TS1005 edge cases
- **Option D:** New feature work - Contribute to other project areas

**Waiting for EM-1 direction on next assignment.**
