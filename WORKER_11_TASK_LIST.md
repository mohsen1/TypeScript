# Worker 11 Task List

## Squad: Parser/Scanner - TS1005 Focus (Part 3)

**Note:** Reassigned from Binder squad to Parser squad per EM-3 reconfiguration.

## Current Task
- [ ] Audit and fix TS1005 false positives in type parameter parsing
- [ ] Fix TS1005 in return type parsing edge cases

## Queue
- [ ] Fix TS1005 in statement parsing - semicolon insertion edge cases
- [ ] Fix TS1005 in class member parsing - property declarations
- [ ] Fix TS1005 in decorator parsing edge cases
- [ ] Assist with TS1109 class member parsing (secondary focus)
- [ ] Run conformance tests and measure TS1005 reduction
- [ ] Coordinate with Workers 9 & 10 to avoid duplicate work

## Completed
- [x] Branch created from em-team-3
- [x] Reviewed TS1005_REDUCTION_RESULTS.md for patterns already fixed
- [x] **TS2304 global scope binding fix** (Ready for Merge: Yes)
  - Implemented chained lookup in `ThinBinderState::resolve_identifier`
  - Added lib_binders check for resolving console, Array, Object, Promise, etc.
  - All lib_loader tests pass
  - Note: Conformance tests show 4941 crashes (pre-existing, not caused by this fix)
- [x] Synced with em-team-3 (no new commits to merge)
- [x] Reconfiguration check: Worker 11 reassigned to Parser squad (TS1005 patterns 11-15)

## Recent Merge Status
- **Date**: 2026-01-14
- **Result**: Already in sync (no new commits)
- **Action Taken**: Verified worker-11 branch is fully merged into em-team-3
- **Next**: Continue work on TS1005 patterns 11-15 (type params, return types, edge cases)
- **Team Update**: Worker 12 re-added to EM-3 (now 4 workers: 9-12)

## Context
TS1005 ("expected X") is the #1 source of parser false positives (439 occurrences). Workers 1-5 have fixed patterns 1-5. Your focus is on remaining patterns 11-15.

### Patterns to Fix (Your Scope)

**Pattern 11 - Type parameter parsing:**
- `parseTypeParameter()` may emit TS1005 on valid generic syntax
- Check default type parameters: `<T = number>`
- Check constraint handling: `<T extends SomeType>`
- Location: `src/compiler/parser.ts`

**Pattern 12 - Return type arrow confusion:**
- Similar to Pattern 3 but may have other instances
- Arrow functions with return types: `(): number => {}`
- Check for confusion between `=>` and `:` in return types
- Location: `src/compiler/parser.ts`

**Pattern 13 - Statement parsing edge cases:**
- `parseStatement()` may emit TS1005 on valid declarations
- Check `if`, `for`, `while` statement parsing
- Check block statement parsing
- Location: `src/compiler/parser.ts`

**Pattern 14 - Class property declarations:**
- `parseClassElement()` for property declarations
- Check initializer parsing: `prop: type = value;`
- Check optional properties: `prop?: type`
- Location: `src/compiler/parser.ts`

**Pattern 15 - Decorator parsing:**
- Decorators before class/method declarations
- Check `@decorator` syntax edge cases
- Location: `src/compiler/parser.ts`

### Secondary Focus: TS1109 Class Members

**Class member declaration confusion:**
- Property declarations may be treated as expressions
- Check `parseClassElement()` for premature TS1109 emission
- Location: `src/compiler/parser.ts`

### Success Metric
Reduce TS1005 from 439 to <100 total.

### Workflow
1. Create branch: `git checkout -b worker-11-TS1005-patterns`
2. Fix identified patterns 11-15
3. Run tests: `npm run test:conformance`
4. Push: `git push origin worker-11-TS1005-patterns`
5. Notify EM-3 for merge
