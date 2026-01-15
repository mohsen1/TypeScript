# WORKER-12 TASK LIST

## Squad: CFA (Control Flow Analysis) Squad
## EM: EM-3
## Branch: worker-12

---

## Primary Task: Implement Class Property Initialization Checks (TS2564)

**Priority:** @ TACTICAL (Priority 4 for EM-3)
**Dependency:** Can start in parallel with other workers

### Problem
- TS2564 is the #1 missing error (413 occurrences)
- "Property 'x' has no initializer and is not definitely assigned in the constructor"
- This means we are simply NOT running the `strictPropertyInitialization` check
- This is a high-ROI task that will knock out the top missing error category

### Action Items

1. **Implement the Check**
   - Add `strictPropertyInitialization` validation to `thin_checker.rs`
   - For each class property:
     - If no initializer AND not marked definite assignment assertion (`!`)
     - AND not assigned in all constructor paths
     - THEN emit TS2564
   - Key file: `wasm/src/checker/thin_checker.rs`

2. **Control Flow Analysis for Constructors**
   - Need to track which properties are assigned in constructor
   - Must handle all code paths (return statements, throws, etc.)
   - Must check all constructor overloads
   - Consider reuse of existing CFA infrastructure

3. **Configuration**
   - Respect `strictPropertyInitialization` compiler option
   - Only emit errors when option is enabled
   - Check how TypeScript's config parsing works

### Files to Work On
- `wasm/src/checker/thin_checker.rs` - Main checker implementation
- `wasm/src/checker/cfa.rs` - Control flow analysis (if exists)
- `wasm/src/binder/class.rs` - Class property binding

### Success Criteria
- Reduce Missing TS2564 from 413 to <20
- Check should respect `strictPropertyInitialization` option
- No false positives on correctly initialized properties

### Edge Cases to Handle
- Properties with definite assignment assertion (`property!: type`)
- Properties assigned in all constructor code paths
- Properties declared in parent class
- Abstract classes
- Properties with type annotations that allow undefined

### Testing
- Run `./wasm/differential-test/run-conformance.sh --all` after each change
- Focus on tests that should produce TS2564 errors
- Verify strictPropertyInitialization option is respected
- Check for false positives on valid code

---

## Instructions
1. Can start in parallel with other workers (no hard dependency)
2. Create branch `worker-12` from `rust` (sync with latest rust first)
3. Focus ONLY on strictPropertyInitialization check
4. Push to `worker-12` branch when ready for review
5. EM-3 will merge and validate before escalating to Director

---

## Task Completion Report

### Actual Work Completed
**Status:** @ PENDING

### Changes Made
- Awaiting implementation

### Results
- Pending

---

## Notes from EM-3
- This is Phase 4 of the EM-3 strategy
- High-ROI task: 413 errors with one check
- This can be developed in parallel with other workers
- Reference TypeScript's implementation for the exact semantics
- Make sure to handle all the edge cases properly
