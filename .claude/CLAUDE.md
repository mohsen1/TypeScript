# TypeScript → Rust/WASM Migration

## Mission
Migrate TypeScript compiler to Rust/WASM. **Beat TypeScript-Go in performance.**

## 🎯 Philosophy: Performance-First Architecture

We have time. No deadlines. Do it right.


## The Plan

There are 3 tracks running at the same time. Find out which track you are on based on git branch name. Each track manages its progress in a plan file in `wasm/`

- `emitter-track`: `wasm/emitter_plan.md`
- `checker-track`: `wasm/checker_plan.md`
- `lsp-track`: `wasm/lsp_plan.md`

You must track todo items and progress in the appropriate plan file


## The Architecture

**`wasm/WASM_ARCHITECTURE.md`**. This is the guide for how we do things. Always read

## The workflow

Loop:
  1. Read this track's *_plan.md file - understand current state
  2. Pick task with HIGHEST IMPACT (not just next in list)
  3. **BEFORE implementing**: Ask Gemini for advice
     - Run: node scripts/ask-gemini.mjs "How should I implement [task]?"
     - Get architectural guidance, edge cases, existing patterns to follow
  4. Implement in wasm/src/*.rs (make sure you add test too)
  5. Test: ./wasm/test.sh
  6. If pass → update *_plan.md, commit
  7. **AFTER implementing**: Ask Gemini to review the implementation
     - Run: node scripts/ask-gemini.mjs --review wasm/src/[modified_file].rs
     - Address any issues found
  8. Final commit with review feedback addressed
  9. **CRITICAL: Sync with origin after EVERY task**
     - Run: `git push origin lsp-track`
     - Run: `git fetch origin && git merge origin/rust`
     - Resolve any conflicts if they occur
     - Run: `git push origin lsp-track` (if there were merges)
     - This keeps all tracks in sync and prevents divergence
  10. Repeat
`

## 🛠️ Commands

```bash
# ⚠️ CRITICAL: ALWAYS use Docker for Rust tests and benchmarks!
# Running cargo test/bench directly on host can use 60GB+ RAM and crash the system

# Rust tests (MUST use Docker wrapper)
./wasm/test.sh                    # All tests (in Docker)
./wasm/test.sh <test_name>        # Specific test (in Docker)

# ❌ NEVER run these directly:
# cargo test          # Will explode host memory
# cargo bench         # Will explode host memory

# Benchmarks (in Docker)
# ./wasm/bench.sh                 # Run benchmarks safely in Docker

# TypeScript integration
npx hereby local                  # Build compiler
npx hereby runtests-parallel      # Full test suite (REQUIRED before merge)

# Gemini reviews
node scripts/ask-gemini.mjs --review                    # Full codebase review
node scripts/ask-gemini.mjs --review wasm/src/parser.rs # Specific file
node scripts/ask-gemini.mjs "How should I structure X?" # Architecture questions

# Verify against TypeScript reference
node scripts/verifyScanner.mjs
node scripts/verifyParser.mjs
node scripts/verifyChecker.mjs
```

## 📁 Key Locations

- `wasm/src/` - All Rust code
- `wasm/WASM_ARCHITECTURE.md` - The main architecture. **always read**
- `wasm/*_plan.md` - Plan files
- `wasm/SOLVER.md` - very important guide for solver


## ✅ Commit Format
```
[wasm] <component>: <description>
```

Commit frequently and atomically

## 🚨 Rules

1. **STAY ON YOUR TRACK** - You are on the **lsp-track**. NEVER switch to checker or emitter work. If your track's work is near perfect, PERFECT IT. Add more tests. Clean up todos. Verify architecture compliance. Polish what you have.
2. **Architecture in mind** - always keep in mind our big picture architecture
3. **Never break the build** - tests must pass
4. **ALWAYS use Docker for Rust** - ./wasm/test.sh only, NEVER raw cargo commands
5. **Separate test files** - `foo.rs` and `foo_tests.rs` or `tests/foo.rs`
6. **Update the plan** - mark tasks complete, add new discoveries
7. **Gemini is your friend** - Gemini can unlock you when things are hard to debug, not sure about path to take. it can guide you how to start a work or review. make sure you use this help

## 🎯 When Your Track Feels "Done"

If lsp work feels complete, that means you have MORE work to do:
- **Add comprehensive tests** - edge cases, error cases, performance tests
- **Review architecture adherence** - does your code follow `wasm/WASM_ARCHITECTURE.md`?
- **Clean up todos** - remove completed items, update progress
- **Optimize performance** - profile hot paths, reduce allocations
- **Improve error messages** - make diagnostics more helpful
- **Document complex logic** - add comments where non-obvious
- **Ask Gemini for review** - get feedback on your best work

**NEVER** say "my track is done, let me help other tracks". Perfect YOUR track.



⚠️ **CRITICAL: For each task, ALWAYS:**
1. **BEFORE**: `node scripts/ask-gemini.mjs "How should I implement [task]?"` - get guidance
2. **IMPLEMENT**: Write code, run tests. add tests
3. **AFTER**: `node scripts/ask-gemini.mjs --review wasm/src/[file].rs` - get review
