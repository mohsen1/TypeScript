# TypeScript → Rust/WASM Migration

## Mission
Migrate TypeScript compiler to Rust/WASM. **Beat TypeScript-Go in performance.**

## 🎯 Philosophy: Performance-First Architecture

We have time. No deadlines. Do it right.


## The Plan

There are 3 tracks running at the same time. Find out which track you are on based on git branch name. Each track manages its progress in a plan file in `wasm/specs`

- `emitter-track`: `wasm/specs/emitter_plan.md`
- `cli-track`: `wasm/specs/cli_plan.md`
- `lsp-track`: `wasm/specs/lsp_plan.md`

You must track todo items and progress in the appropriate plan file


## The Architecture

**`specs/WASM_ARCHITECTURE.md`**. This is the guide for how we do things. **Always read this**

## The workflow

Loop:
  1. Read this track's *_plan.md file - understand current state
  2. Pick task with HIGHEST IMPACT (not just next in list)
  3. Implement in wasm/src/*.rs (make sure you add test too)
  4. Test: ./wasm/test.sh
  5. If pass → update *_plan.md, commit
  6. **CRITICAL: Continuously sync with origin/rust**
     - Before starting any task
     - After finishing any task
     - Whenever idle (do not wait for conflicts to pile up)
     - Run: `git add . && git commit -m "[wasm] your changes"`
     - Run: `git push origin rust` (ALL tracks push to shared rust branch)
     - Run: `git fetch origin && git merge origin/rust`
     - Resolve any conflicts if they occur
     - Run: `git push origin rust` (if there were merges)
     - This keeps all tracks in sync and prevents divergence
  7. Repeat
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

# Verify against TypeScript reference
node scripts/verifyScanner.mjs
node scripts/verifyParser.mjs
node scripts/verifyChecker.mjs
```

## 📁 Key Locations

- `wasm/src/` - All Rust code
- `wasm/specs/WASM_ARCHITECTURE.md` - The main architecture. **always read**
- `wasm/specs/*_plan.md` - Plan files
- `wasm/specs/SOLVER.md` - very important guide for solver


## ✅ Commit Format
```
[wasm] <component>: <description>
```

Commit frequently and atomically

## 🚨 Rules

1. **STAY ON YOUR TRACK** - NEVER switch tracks. If your track's work is near perfect, PERFECT IT. Add more tests. Clean up todos. Verify architecture compliance. Polish what you have.
2. **Architecture in mind** - always keep in mind our big picture architecture
3. **Never break the build** - tests must pass
4. **ALWAYS use Docker for Rust** - ./wasm/test.sh only, NEVER raw cargo commands
5. **Separate test files** - `foo.rs` and `foo_tests.rs` or `tests/foo.rs`
6. **Update the plan** - mark tasks complete, add new discoveries
7. **Commit and sync frequently** - after each task, commit and sync with origin/rust

## 🎯 When Your Track Feels "Done"

If checker work feels complete, that means you have MORE work to do:
- **Check out review files you wasm/specs** - maybe something we can address now in our track?
- **Add comprehensive tests** - edge cases, error cases, performance tests
- **Review architecture adherence** - does your code follow `wasm/WASM_ARCHITECTURE.md`?
- **Clean up todos** - remove completed items, update progress
- **Optimize performance** - profile hot paths, reduce allocations
- **Improve error messages** - make diagnostics more helpful
- **Document complex logic** - add comments where non-obvious

**NEVER** say "my track is done, let me help other tracks". Perfect YOUR track.


⚠️ **CRITICAL: Keep syncing `origin/rust` branch`**
1. After each task completion pull in origin/rust
2. Resolve conflicts. Other tracks are making progress too
3. Commit and push
