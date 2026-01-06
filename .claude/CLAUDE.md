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

**Avoid:**
- Micro-optimizations that save nanoseconds (iterator tricks, inline hints)
- Porting TypeScript patterns that don't fit Rust's ownership model
- Adding features before the architecture is solid

## 📋 The Plan
**`specs/migration_plan.md`** is the single source of truth. Phase 0 (Performance) comes BEFORE completing remaining phases.

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
```

⚠️ **IMPORTANT**: Always consult Gemini BEFORE and AFTER each significant task. This catches design issues early and ensures code quality.

## 📊 When to Use Gemini Reviews

**DO review with Gemini:**
- After completing a major component (parser, checker, emitter)
- Before starting architectural changes (ThinNode migration)
- When stuck on a design decision
- After fixing a batch of issues

**DON'T review with Gemini:**
- Every single commit (too noisy)
- Simple bug fixes
- Documentation updates

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
- `specs/migration_plan.md` - THE PLAN
- `specs/gemini_review_*.md` - Review findings to address
- `specs/SOLVER.md` - very important guide for solver

## 🏗️ Checker-Solver Architecture

**"Check Fast, Explain Slow"**: The checker (`thin_checker.rs`) traverses AST and calls solver. The solver (`wasm/src/solver/`) owns all type logic. When errors occur, use `solver.explain_failure()` to get detailed diagnostics instead of re-inspecting types in the checker.



## Reference: typescript-go Submodule

The `typescript-go/` directory contains a submodule with a native Go port of the TypeScript compiler and language server. **Feel free to explore and reference this codebase** for understanding alternative implementations or comparing approaches, but **do not modify any files within the `typescript-go/` directory**.

This submodule is read-only reference material for:
- Understanding how compiler features are implemented in Go
- Comparing type checking strategies
- Seeing alternative approaches to parsing and binding

When working on TypeScript compiler features, you may find it helpful to look at the corresponding Go implementation in `typescript-go/internal/` for inspiration or clarification.

You can take a look at `specs/TYPESCRIPT_GO_ARCHITECTURE.md` for an overview
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
7. **Gemini at milestones** - not every commit, but every major component. or when it is a big task and makes sense to get some help



⚠️ **CRITICAL: For each task, ALWAYS:**
1. **BEFORE**: `node scripts/ask-gemini.mjs "How should I implement [task]?"` - get guidance
2. **IMPLEMENT**: Write code, run tests. add tests
3. **AFTER**: `node scripts/ask-gemini.mjs --review wasm/src/[file].rs` - get review
