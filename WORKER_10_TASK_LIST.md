# Worker 10 Task List

## Squad: Parser/Scanner - TS1109 Focus

## Current Task
- [ ] Audit TS1109 ("expression expected") emission patterns in parser
- [ ] Identify top locations causing false positives

## Queue
- [ ] Fix TS1109 in statement parsing - valid declarations triggering errors
- [ ] Fix TS1109 in expression parsing - await/yield edge cases
- [ ] Fix TS1109 in class member parsing - property declarations
- [ ] Run conformance tests and measure TS1109 reduction
- [ ] Coordinate with Worker 9 to avoid overlap

## Completed
- [x] Branch created from em-team-3
- [x] Reviewed TS1109_ANALYSIS.md for context
- [x] Synced with em-team-3 (no new commits to merge)
- [x] Reconfiguration check: Worker 10 remains on Parser squad (TS1109 focus)

## Recent Merge Status
- **Date**: 2026-01-14
- **Result**: Already in sync (no new commits)
- **Action Taken**: Verified worker-10 branch is fully merged into em-team-3
- **Next**: Continue work on TS1109 expression expected errors
- **Team Update**: Worker 12 re-added to EM-3 (now 4 workers: 9-12)

## Context
TS1109 ("expression expected") has 262 false positive occurrences. These occur when the parser expects an expression but encounters a valid construct it doesn't recognize.

### Key Areas to Investigate

**Area 1 - Statement vs Expression confusion:**
- Declaration statements (class, function, enum) may be treated as expressions
- Check `parseStatement()` for premature TS1109 emission
- Location: `src/compiler/parser.ts`

**Area 2 - Await/Yield handling:**
- Await expressions in non-async contexts may trigger false TS1109
- Yield expressions in generator functions
- Check `parseAwaitExpression()` and `parseYieldExpression()`
- Location: `src/compiler/parser.ts`

**Area 3 - Class property parsing:**
- Property declarations with modifiers may trigger TS1109
- Check `parseClassElement()` for edge cases
- Location: `src/compiler/parser.ts`

**Area 4 - Type assertion edge cases:**
- Angle bracket type assertions: `<Type>expr`
- May be confused with JSX or relational operators
- Location: `src/compiler/parser.ts`

### Reference
- `TS1109_ANALYSIS.md` - Contains existing analysis of this error pattern

### Success Metric
Reduce TS1109 from 262 to <50 total.

### Workflow
1. Create branch: `git checkout -b worker-10-TS1109-fixes`
2. Fix identified patterns
3. Run tests: `npm run test:conformance`
4. Push: `git push origin worker-10-TS1109-fixes`
5. Notify EM-3 for merge
