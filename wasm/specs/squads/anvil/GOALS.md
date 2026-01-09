# Squad Anvil Goals

Updated: 2026-01-09

Priority: 1

---
## 🛑 GRACEFUL EXIT - WRAP UP NOW

**Director has called for graceful exit. Wrap up all worker tasks NOW.**

1. Let workers finish their current atomic task (commit what's done)
2. Do NOT assign new tasks
3. Ensure all work is committed and pushed to worker branches
4. Mark workers as complete when done
5. Final merge to squad branch

**Session is ending. No new work assignments.**

---
## ⚠️ TMUX REMINDER - CHECK FOR HANGING PROMPTS

**NEVER forget to pause 1 second before pressing Enter in tmux!**

1. After sending any message, wait 1 second, THEN send Enter (C-m)
2. Check all worker panes for prompts that may be hanging (message sent but no activity)
3. If a prompt is hanging, send Enter again: `sleep 1 && tmux send-keys -t <pane> C-m`

**Do this check NOW and periodically throughout your session.**

---
## 📢 OPERATION CONFORMANCE - NEW DIRECTIVE

**Conformance test results show 18.1% exact match rate. Squad Anvil is reassigned to fix false positives and parser issues.**

### Current State (from conformance runner)
- Extra Errors: 271 tests (38.8%) - WASM reports errors TSC doesn't
- Main culprits: TS2304 (75), TS2339 (68), TS1005 (44), TS2769 (30)

### Root Causes Identified (Anvil Scope)
1. **Namespace Scoping Bug** (TS2304) - 75 false positives - Identifiers can't resolve siblings
2. **Property Resolution** (TS2339) - 68 false positives - "Property does not exist" incorrectly
3. **Parser Issues** (TS1005/TS1068) - 60 false positives - Unexpected token errors
4. **Overload Resolution** (TS2769) - 30 false positives - "No overload matches" incorrectly
5. **Index Signature** (TS7053) - 28 false positives - Implicit any index access

---

## Current Milestone
**Phase 9 - Conformance Parity**: Fix false positives to reach 50%+ conformance.

## Focus Areas
- `wasm/src/thin_binder.rs` - Fix namespace scoping (TS2304)
- `wasm/src/thin_checker.rs` - Fix property resolution (TS2339)
- `wasm/src/parser/` - Fix parser edge cases (TS1005/TS1068)

## Objectives (Ranked by Test Impact)

### 1. **Namespace Scoping Bug** (TS2304) - 75 false positives
   - Problem: Identifiers inside `export namespace {}` can't resolve siblings
   - Root Cause: Binder scope chain not properly linking namespace members
   - Key Files: `thin_binder.rs` - scope resolution
   - Assigned: **Workers 1-2**

### 2. **Property Resolution** (TS2339) - 68 false positives
   - Problem: "Property 'X' does not exist on type 'Y'" when it does exist
   - Root Cause: Type resolution not finding inherited/merged properties
   - Key Files: `thin_checker.rs` - property access checking
   - Assigned: **Worker 3**

### 3. **Parser Edge Cases** (TS1005/TS1068) - 60 false positives
   - Problem: "Expected X" / "Unexpected token" for valid TypeScript
   - Root Cause: Parser not handling certain syntax patterns
   - Key Files: `parser/` - parse functions
   - Assigned: **Worker 4**

### 4. **Overload Resolution** (TS2769) - 30 false positives
   - Problem: "No overload matches this call" when one should match
   - Root Cause: Overload matching too strict or not checking all overloads
   - Key Files: `thin_checker.rs`, `solver/` - call resolution
   - Assigned: **Worker 5**

## Anti-Priorities
- ⛔ New emitter transforms
- ⛔ New ES5 downleveling
- ⛔ Source map enhancements
- ⛔ LSP features

## Cross-Squad Dependencies
- Forge workers implementing missing checks (affects missing errors)
- Share conformance runner: `wasm/differential-test/conformance-runner.mjs`

## Notes to EM
- Run conformance tests: `cd wasm/differential-test && node conformance-runner.mjs --max=500`
- Focus on REDUCING false positives (extra errors)
- Each fix should show reduction in "extra errors" count
- Use Docker for tests: `./wasm/test.sh`

## Management Strategy
**Conformance-Driven Development**:
- PRs must include before/after conformance numbers
- Target: Reduce extra errors from 271 to <100

## Squad Status
- Last EM Report: 2026-01-09 - Operation Conformance initiated
- Conformance Baseline: **271 tests with extra errors (38.8%)**
- Workers Active: 5/5
- Current Focus: Fixing false positives
- Worker Assignments:
  - W1: Namespace scoping (TS2304) - binder scope chain
  - W2: Namespace scoping (TS2304) - symbol resolution
  - W3: Property resolution (TS2339) - inherited properties
  - W4: Parser edge cases (TS1005/TS1068) - syntax patterns
  - W5: Overload resolution (TS2769) - call matching
