# TypeScript → Rust/WASM Migration

## Mission
Migrate TypeScript compiler to Rust/WASM. **Beat TypeScript-Go in performance.**

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
     - Phase 0 tasks (ThinNode, zero-alloc scanner, parallelism) are highest priority
     - Architectural improvements > feature completion
  3. Before major work: Ask "Will this 10x something, or just 1.1x?"
  4. Implement in wasm/src/*.rs
  5. Test: ./wasm/test.sh
  6. If pass → update migration_plan.md, commit
  7. At milestones → Gemini review: `node scripts/ask-gemini.mjs --review`
  8. Address review findings, commit, continue
```

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
# Rust tests (use Docker to prevent RAM explosion)
./wasm/test.sh                    # All tests
./wasm/test.sh <test_name>        # Specific test

# Benchmarks (TODO: add criterion benchmarks)
# cargo bench --bench parser_bench

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



## Reference: typescript-go Submodule

The `typescript-go/` directory contains a submodule with a native Go port of the TypeScript compiler and language server. **Feel free to explore and reference this codebase** for understanding alternative implementations or comparing approaches, but **do not modify any files within the `typescript-go/` directory**.

This submodule is read-only reference material for:
- Understanding how compiler features are implemented in Go
- Comparing type checking strategies
- Seeing alternative approaches to parsing and binding

When working on TypeScript compiler features, you may find it helpful to look at the corresponding Go implementation in `typescript-go/internal/` for inspiration or clarification.

## ✅ Commit Format
```
[wasm] <component>: <description>
```

## 🚨 Rules

1. **Architecture before features** - Phase 0 (ThinNode, parallelism) before Phase 6-8
2. **Never break the build** - tests must pass
3. **Measure impact** - add benchmarks for perf claims
4. **Update the plan** - mark tasks complete, add new discoveries
5. **Separate test files** - `foo.rs` and `foo_tests.rs` or `tests/foo.rs`
6. **Gemini at milestones** - not every commit, but every major component

