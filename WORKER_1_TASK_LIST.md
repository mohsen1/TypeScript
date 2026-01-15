# Worker-1 Task List

## Assignment: Parser Noise (TS1005 & TS1109)
**Priority:** 🔴 CRITICAL
**Owner:** worker-1
**Branch:** worker-1

## Task Description
Fix the "Parser Noise" problem. Our ThinParser is emitting 701 extra errors (TS1005: 439, TS1109: 262) that TypeScript doesn't report. These are false positives from the parser bailing out on valid syntax.

## Problem Analysis
From PROJECT_DIRECTION.md:
- **TS1005:** "Expected X" - parser hit unexpected token
- **TS1109:** "Expected expression" - expression parsing failed
- These errors break downstream semantic analysis because a broken AST produces broken symbols
- Root cause: Missing error resynchronization and ASI (Automatic Semicolon Insertion) issues

## Action Items

### Phase 1: Investigation (Ask Gemini First!)
```bash
# MANDATORY - Run this before writing any code
./scripts/ask-gemini.mjs "I need to implement error resynchronization in the ThinParser to fix TS1005 and TS1109 false positives. What files should I modify and what's the approach?"
```

- [ ] Read `wasm/specs/WASM_ARCHITECTURE.md` parser section
- [ ] Study `wasm/src/parser/` error handling patterns
- [ ] Compare ASI logic with TypeScript's implementation
- [ ] Run conformance tests to get baseline report:
  ```bash
  ./wasm/differential-test/run-conformance.sh --all
  ```

### Phase 2: Implementation
- [ ] Implement error resynchronization in `wasm/src/parser/thin_parser.rs`:
  - On unexpected token, advance to next synchronization point (`;`, `}`, etc.)
  - Continue parsing the rest of the file
- [ ] Audit and fix ASI (Automatic Semicolon Insertion) logic
  - Verify our ASI matches TypeScript's exactly
  - Many TS1005 errors are likely missing semicolons we aren't inferring
- [ ] Add tests for edge cases that previously failed

### Phase 3: Validation
- [ ] Run `./wasm/test.sh` (Docker-only!)
- [ ] Run conformance tests: `./wasm/differential-test/run-conformance.sh --all`
- [ ] Compare to baseline report
- [ ] Verify TS1005 reduced from 439 to <20
- [ ] Verify TS1109 reduced from 262 to <20
- [ ] Check that no new regressions were introduced

## Success Metrics
- **TS1005:** Reduce from 439 to <20
- **TS1109:** Reduce from 262 to <20
- **Combined:** Reduce from ~700 to <40
- **No regressions:** Don't break existing working tests

## Deliverables
1. Code changes in `wasm/src/parser/`
2. Tests for parser error recovery
3. Conformance test report showing improvement
4. Set `Ready for Merge: Yes` in your plan when complete

## Workflow
1. Sync: `git fetch origin && git merge origin/rust --no-edit`
2. **ASK GEMINI FIRST** (see Phase 1)
3. Write code following Gemini's guidance
4. Test: `./wasm/test.sh`
5. Commit: `[wasm] parser: fix error resynchronization for TS1005/TS1109`
6. Push to worker-1 branch
7. Run conformance tests and analyze report
8. Mark `Ready for Merge: Yes` in your plan

## Status
- **Ready for Merge:** No
- **Last Updated:** 2026-01-14
