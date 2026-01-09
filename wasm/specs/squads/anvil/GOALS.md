# Squad Anvil Goals

Updated: 2026-01-09

Priority: 2

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
## 📢 EM-ANVIL: READ THIS - DIRECTIVE FROM DIRECTOR

**Operation Crucible is NOW IN EFFECT.** Your squad has been restructured:

1. **Workers 1-2**: Bug fixes ONLY. No new ES5 tests, no new transforms.
2. **Workers 3-5**: REASSIGNED to Crucible. They now port solver tests from official TS repo.
3. **Reject any PR** that adds new emitter features.

Please acknowledge by updating Squad Status below.

---

## ⚠️ OPERATION CRUCIBLE - TACTICAL SHIFT

**Emitter is 80% complete. MAINTENANCE MODE activated.**

- **Squad reduced to 2 workers** (workers 1-2)
- **Bug fixes ONLY** - critical source map bugs and blocking ES5 regressions
- **⛔ NO NEW TRANSFORMS** - stop feature work
- Workers 3-5 reassigned to **Crucible Tasks** (test porting)

## Current Milestone
Phase 8 - Conformance, Convergence, and Hardening: Output fidelity across the integrated pipeline, driven by conformance tests.

## Project Direction Alignment
- Emitter is 80% complete - stop adding new features
- Bug fixes only: `super["m"]` in async, nested arrow `this` capture
- ⚠️ Anti-pattern: Do NOT use regex substitutions for code transforms. Always operate on AST.

## Focus Areas (MAINTENANCE ONLY)
- `wasm/src/thin_emitter/` - Critical bug fixes only
- `wasm/src/transforms/` - Blocking ES5 regressions only

## Objectives (Ranked)

1. **Critical Bug Fixes Only**
   - Context: Emitter is in maintenance mode per Operation Crucible
   - Success Criteria: Fix blocking ES5 regressions (`super["m"]` in async, nested arrow `this` capture)
   - Key Files: `transforms/class_es5.rs`, `transforms/async_es5.rs`
   - ⛔ NO NEW FEATURES

2. **Source Map Bug Fixes**
   - Context: Only fix bugs that block debugger attachment
   - Success Criteria: Source maps validate and attach correctly
   - Key Files: `thin_emitter/source_writer.rs`, `thin_emitter/source_map.rs`
   - ⛔ NO NEW MAPPINGS

## Anti-Priorities (ENFORCED)
- ⛔ **New Emitter transforms** - we have enough
- ⛔ **New test coverage** - workers 3-5 reassigned to Crucible
- New LSP features
- CLI argument parsing
- Performance micro-optimizations

## Cross-Squad Dependencies
- Forge squad owns type checking; coordinate on shared conformance regressions

## Notes to EM
- **Only 2 workers active** (workers 1-2)
- Workers 3-5 are now on Crucible Tasks (test porting) - see their plans
- Bug fixes only - reject any PR that adds new transforms
- Read `wasm/specs/WASM_ARCHITECTURE.md` for architecture
- Use Docker for tests: `./wasm/test.sh`

## Management Strategy
Per Project Direction: **Autocratic Scheduling + Bisect-on-Merge**
- PRs that regress ANY existing baseline are auto-rejected
- Zero-Idle: If a high-priority task is blocked, swarm it

## Squad Status
- Last EM Report: 2026-01-08 - Operation Crucible activated
- Workers Active: 2/5 (workers 1-2 on bug fixes)
- Workers Reassigned: 3/5 (workers 3-5 on Crucible test porting)
- Current Focus: Critical bug fixes only
- Direction: MAINTENANCE MODE - no new transforms
- Blockers: None
- Worker Assignments:
  - W1: Bug fixes - blocking ES5 regressions
  - W2: Bug fixes - critical source map issues
  - W3: **CRUCIBLE** - Port conditional type tests from official TS repo
  - W4: **CRUCIBLE** - Port mapped type tests from official TS repo
  - W5: **CRUCIBLE** - Port conditional/mapped type tests from official TS repo
