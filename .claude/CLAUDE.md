# TypeScript → Rust/WASM Migration

## Mission
Migrate TypeScript compiler to Rust/WASM. **Beat TypeScript-Go in performance.**

### Eventual Goal
Rust port can run every single test case in test/cases faster than go port. Then we will release the port to the world. It should match TypeScript Go in terms of TS language feature (TS version). Later we can add new language features.

## 🎯 Philosophy: Performance-First Architecture

We have time. No deadlines. Do it right.

**Priority Order:**
1. **Architectural wins** (ThinNode, arena allocation, parallelism) > line-by-line porting
2. **Measure before optimizing** - run benchmarks, not guesses
3. **Review at milestones** - use Gemini for architecture decisions, not just bug hunting


## The Plan
**`specs/migration_plan.md`** is the single source of truth. Phase 0 (Performance) comes BEFORE completing remaining phases.

## The Architecture

**`specs/WASM_ARCHITECTURE.md`**. This is the guide for how we do things

## 🔁 WORK LOOP

```
LOOP:
  1. Read specs/migration_plan.md - understand current state
  2. Pick task with HIGHEST IMPACT (not just next in list)
  3. **BEFORE implementing**: Ask Gemini for advice
     - Run: node scripts/ask-gemini.mjs "How should I implement [task]?"
     - Get architectural guidance, edge cases, existing patterns to follow
  4. Implement in wasm/src/*.rs
  5. Test: ./wasm/test.sh
  6. If pass → update migration_plan.md, commit
  7. **AFTER implementing**: Ask Gemini to review the implementation
     - Run: node scripts/ask-gemini.mjs --review wasm/src/[modified_file].rs
     - Address any issues found
  8. Final commit with review feedback addressed
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
- `specs/WASM_ARCHITECTURE.md` - The main architecture. **always read**
- `specs/migration_plan.md` - THE PLAN
- `specs/SOLVER.md` - very important guide for solver


## ✅ Commit Format
```
[wasm] <component>: <description>
```

Commit frequently and atomically

## 🚨 Rules

1. **Architecture before features** - Phase 0 (ThinNode, parallelism) before Phase 6-8
2. **Never break the build** - tests must pass
3. **ALWAYS use Docker for Rust** - ./wasm/test.sh only, NEVER raw cargo commands
4. **Measure impact** - add benchmarks for perf claims
5. **Update the plan** - mark tasks complete, add new discoveries
6. IMPORTANT: **Separate test files** - `foo.rs` and `foo_tests.rs` or `tests/foo.rs` even in Rust files. if you see a file that has source and test in the same file move tests to separate file as a top priority
7. **Gemini is your friend** - Gemini can unlock you when things are hard to debug, not sure about path to take. it can guide you how to start a work or review. make sure you use this help



⚠️ **CRITICAL: For each task, ALWAYS:**
1. **BEFORE**: `node scripts/ask-gemini.mjs "How should I implement [task]?"` - get guidance
2. **IMPLEMENT**: Write code, run tests. add tests
3. **AFTER**: `node scripts/ask-gemini.mjs --review wasm/src/[file].rs` - get review
