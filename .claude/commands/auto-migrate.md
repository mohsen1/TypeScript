---
description: Autonomous migration - reads MIGRATION_PLAN.md and works on next pending task without user input
allowed-tools: Read, Grep, Glob, Bash, Edit, Write, TodoWrite
---

# Autonomous Migration Mode

You are running in AUTONOMOUS MODE. Work continuously without asking questions.
Make decisions independently based on the migration plan and codebase analysis.

## Prime Directive

**NEVER ASK QUESTIONS. NEVER STOP. ALWAYS MAKE PROGRESS.**

If uncertain, choose the safer/simpler option and document your choice in the commit message.

## Startup Sequence

1. Read `MIGRATION_PLAN.md` to understand current state
2. Find the first unchecked `[ ]` item in the current phase
3. Create a TodoWrite list for the session
4. Begin work immediately

## Decision Framework

When facing choices, use this priority:

1. **Safety first**: If a change might break tests, do the smaller/safer version
2. **Match TypeScript**: When in doubt about behavior, match exactly what TypeScript does
3. **Incremental progress**: Prefer many small commits over one large change
4. **Skip blockers**: If stuck on X, move to Y and note X needs attention

## Work Loop

```
REPEAT:
  1. Pick next unchecked item from MIGRATION_PLAN.md
  2. Implement in Rust (small, testable chunks)
  3. Run: cd wasm && cargo build && cargo test
  4. If tests pass → commit and continue
  5. If tests fail → fix errors (max 3 attempts, then skip with TODO)
  6. Update MIGRATION_PLAN.md progress log
  7. Update TodoWrite with progress
```

## Error Recovery

### Rust compilation error
1. Read the error carefully
2. Apply fix from patterns.md if applicable
3. Common fixes:
   - Borrow error → extract to local variable
   - Missing type → add to imports and parser.rs
   - wasm-bindgen error → add #[wasm_bindgen(skip)]

### Test failure
1. Check which test failed
2. Compare Rust output to expected
3. Fix and re-run (max 3 attempts)
4. If still failing, revert changes and move to next task

### Stuck for >10 minutes on one issue
1. Add TODO comment in code
2. Log issue in MIGRATION_PLAN.md
3. Move to next task
4. Continue making progress

## Commit Pattern

After each successful change:
```bash
git add -A
git commit -m "feat(wasm): <what was done>

<brief description>

🤖 Generated with [Claude Code](https://claude.com/claude-code)

Co-Authored-By: Claude <noreply@anthropic.com>"
```

## Session End

Before context runs out:
1. Commit all passing work
2. Update MIGRATION_PLAN.md with session summary
3. Note where to resume in progress log

## Current Task Selection

1. First, read `.claude/TASK_QUEUE.md` for detailed task breakdown
2. Find the first `[ ]` unchecked task
3. Each task includes:
   - File to modify
   - What to implement
   - Test case to verify
4. Work on it until tests pass or you're blocked
5. Mark as `[x]` done or `[!]` blocked
6. Continue to next task

## Start Now

Read `.claude/TASK_QUEUE.md` and begin working on the first unchecked task.

Additional context from user: $ARGUMENTS
