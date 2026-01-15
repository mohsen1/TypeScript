# EM-1 TASK LIST

## Team: em-team-1
## Manager: EM-1
## Base Branch: rust
## Team Branch: em-team-1

---

## Mission Statement

EM-1 is responsible for **parser accuracy**. Your team ensures that:
1. Syntax errors are properly suppressed when TypeScript suppresses them (TS1109, TS1005)
2. Error recovery allows parsing to continue after syntax errors
3. ASI (Automatic Semicolon Insertion) works correctly
4. Parser doesn't emit false syntax errors

**Why this matters:** Parser accuracy is the foundation. If the parser emits false syntax errors, the binder and type checker never run. Users trust TypeScript to parse valid JavaScript.

---

## Team Composition

| Worker | Squad | Focus Area | Status | Throughput |
|--------|-------|------------|--------|------------|
| Worker 3 | Syntax | TS1109 suppression | ❌ **Needs restart** | None |
| Worker 4 | Syntax | TS1005 suppression | ❌ **Did wrong task** | Low |
| Worker 5 | Syntax | Parser error recovery | ✅ Complete | High |

**Leadership:** Worker 5 (exceptional throughput, mentor for Workers 3-4)

**EM Branch:** em-team-1

---

## Completed Work

### Worker 5: Parser Error Recovery ✅
**Commits:**
- 2fac75924a - Statement-level error recovery
- 29ec0035e8 - Control statement error recovery (switch, try-catch, for, while)

**Problem:** Parser would crash or emit cascading errors after encountering syntax errors.

**Fix:** Enhanced error recovery for:
- Switch statements: "case or default expected" error
- Try-catch-finally: Missing catch/finally error
- For/while loops: Error recovery for failed parsing
- Control statements: Comprehensive error recovery

**Impact:** 48 lines of production-quality parser code. Parser now continues after syntax errors instead of crashing.

---

## Team Priorities (Updated 2026-01-15 13:20)

### Priority 1: Restart Workers 3-4 Under Worker 5's Mentorship
**Owner:** EM-1 (with Worker 5 as mentor)
**Status:** 🔴 ACTIVE - Immediate action required
**Started:** 2026-01-15 13:45

**Problem:** Workers 3-4 did not complete their assigned tasks:
- Worker 3: Only documentation, no TS1109 implementation (262 extra errors)
- Worker 4: Did wrong task (solver defaults instead of TS1005, 345 extra errors)

**Mentorship Framework:**

**Week 1: Intensive Onboarding**
- Day 1-2: Worker 5 pairs with Workers 3-4 to explain parser architecture
- Day 3-4: Worker 5 assigns subtasks, reviews code before commits
- Day 5: Checkpoint - assess progress, adjust approach if needed

**Week 2: Scaled Mentorship**
- Worker 5 does daily code reviews (morning sync)
- Workers 3-4 commit daily with specific subtask completion
- Worker 5 available for questions but not pairing full-time

**Week 3: Independence Assessment**
- Workers 3-4 work independently with daily check-ins
- If no substantial progress by end of Week 3, reassign to EM-3 (type checking)

**Specific Tasks Assigned:**

**Worker 3 - TS1109 Suppression (Mentor: Worker 5)**
- Subtask 3.1: Study existing error recovery in Worker 5's commits
- Subtask 3.2: Identify 5 conformance tests where TS1109 should be suppressed
- Subtask 3.3: Implement suppression for expression contexts
- Subtask 3.4: Test and verify error count reduction
- **Commit requirement:** Daily commits with `[wasm] parser: TS1109 - <subtask>`

**Worker 4 - TS1005 Suppression (Mentor: Worker 5)**
- Subtask 4.1: Extend Worker 5's existing token suppression work
- Subtask 4.2: Identify 5 conformance tests where TS1005 should be suppressed
- Subtask 4.3: Implement suppression for additional token types
- Subtask 4.4: Test and verify error count reduction
- **Commit requirement:** Daily commits with `[wasm] parser: TS1005 - <subtask>`

**Success Criteria:**
- Week 1: Workers 3-4 complete at least 2 subtasks each
- Week 2: Substantial code committed (20+ lines per worker)
- Week 3: Error count reduction measurable (TS1109: 262→200, TS1005: 345→250)

**Fallback Plan:**
- If Worker 3 shows no progress by Day 5 → reassign to EM-3 (TS2571/TS2683 tasks)
- If Worker 4 shows no progress by Day 5 → reassign to EM-3 (generic type constraints)
- Worker 5 takes over both tasks (has proven high throughput)

### Priority 2: TS1109 Suppression (Parser Expression Expected)
**Owner:** Worker 3 (restarted) or Worker 5

**Goal:** Reduce TS1109 extra errors from 262 to <40

**Current State:**
- TS1109: "Expression expected"
- 262 extra errors (we emit, tsc doesn't)
- TypeScript often suppresses this error in recovery contexts

**Action Items:**
1. Identify when TypeScript suppresses TS1109
2. Implement suppression logic in parser
3. Add error recovery for expression contexts
4. Test with conformance suite

**Target Metrics:**
| Error Code | Current | Target |
|------------|---------|--------|
| TS1109 extra | ~262 | <40 |

**Key Files:**
- `wasm/src/thin_parser.rs` - expression parsing functions
- `wasm/src/parser_recovery.rs` - error recovery logic

### Priority 3: TS1005 Suppression (Parser Token Expected)
**Owner:** Worker 4 (restarted) or Worker 5

**Goal:** Reduce TS1005 extra errors from 345 to <50

**Current State:**
- TS1005: "X expected" (e.g., "; expected", "} expected")
- 345 extra errors (we emit, tsc doesn't)
- Worker 5 already implemented partial suppression for some tokens
- Need to extend to remaining cases

**Action Items:**
1. Extend Worker 5's work to cover more token types
2. Identify when TypeScript suppresses TS1005
3. Implement suppression logic in parser
4. Add error recovery for token contexts

**Target Metrics:**
| Error Code | Current | Target |
|------------|---------|--------|
| TS1005 extra | ~345 | <50 |

**Key Files:**
- `wasm/src/thin_parser.rs` - token parsing functions
- `wasm/src/parser_recovery.rs` - error recovery logic

### Priority 4: ASI Handling Edge Cases
**Owner:** Worker 3 or Worker 4 (after completing priorities 2-3)

**Goal:** Correct automatic semicolon insertion in edge cases

**Current State:**
- Most ASI cases work correctly
- Edge cases: return statements, postfix ++/--, template literals

**Action Items:**
1. Identify ASI edge cases in conformance tests
2. Implement ASI logic in parser
3. Test with conformance suite

---

## Conformance Test Baseline (2026-01-15)

Current baseline from rust branch (commit 74df9fd30d):

**Top Parser Errors:**
- TS1005: ~345 extra (Parser token expected errors)
- TS1109: ~262 extra (Parser expression expected errors)

**Target:** Reduce parser noise by 80% while maintaining accuracy

---

## Workflow

### For EM-1:
1. **Daily sync**: `git pull origin rust` → merge to em-team-1
2. **Review worker branches**: Check commits, test results
3. **Merge locally**: `git merge worker-X` into em-team-1
4. **Run validation**: `./wasm/differential-test/run-conformance.sh --max=500 --workers=4`
5. **Push to director**: Only when stable and validated

### For Workers:
1. Create branch from em-team-1
2. Work on assigned task ONLY
3. Commit frequently with `[wasm] parser: <description>`
4. Push to worker-X branch
5. Update task list with status
6. Notify EM-1 when ready for merge

---

## Merge Readiness Status (2026-01-15 13:45)

| Worker | Status | Notes |
|--------|--------|-------|
| Worker 3 | 🔵 Restarted | TS1109 suppression - Week 1 mentorship with Worker 5 |
| Worker 4 | 🔵 Restarted | TS1005 suppression - Week 1 mentorship with Worker 5 |
| Worker 5 | 🟢 Mentor | Parser recovery complete, mentoring Workers 3-4 |

---

## Escalation Path

1. Worker commits → worker-X branch
2. EM-1 merges to em-team-1 → validates
3. EM-1 escalates to Director when stable
4. Director reviews → merges to rust

**Do NOT push directly to rust.**

---

## Next Actions for EM-1

1. ✅ Create em-team-1 branch
2. ✅ Create EM_1_TASKS.md
3. ✅ Worker 5 complete - transfer from EM-2
4. ✅ **Workers 3-4 restarted** under Worker 5's mentorship (2026-01-15 13:45)
5. 🔄 **Week 1 Checkpoint** - Assess progress by 2026-01-22
6. 📋 Track parser error counts (TS1109: 262, TS1005: 345)
7. 📅 **Decision point** 2026-02-05 - Reassign or continue based on progress

---

## Notes

- All work must stay in `wasm/` directory
- TypeScript source files (`src/compiler/`) are READ-ONLY
- Run `./wasm/test.sh` for Rust tests (Docker-only)
- Use `./scripts/ask-gemini.mjs` before coding (if applicable)
- Target: 95%+ exact match before production
- **Focus:** Parser accuracy, not binder or type checker work

---

## Team Size and Capacity

**Current Workers:** 3 (Worker 3, Worker 4, Worker 5)
**Capacity:** At maximum (limit is 4)

**Future Considerations:**
- If Worker 3-4 don't progress, may reassign to EM-3 (type checking)
- EM-1 needs strong mentorship from Worker 5
- Focus on parser accuracy tasks only

---

## Success Metrics

| Metric | Current | Target (EM-1) |
|--------|---------|---------------|
| TS1005 extra errors | ~345 | <50 |
| TS1109 extra errors | ~262 | <40 |
| Exact Match Rate | ~30% | 35%+ |

**Overall EM-1 Goal:** Reduce parser noise by 80%, increase exact match rate by 5%
