# Migration Plan: TypeScript Compiler → Rust via WebAssembly

## Vision

Incrementally rewrite the TypeScript compiler in Rust, compiled to WebAssembly
for seamless Node.js/browser interop. **Beat TypeScript-Go in performance.**

## Session log

see SESSION_LOG.md -- always amended with each session's work

# ✅ COMPLETED

- **Phase 0-5**: Scanner, Parser, Binder, Solver (~77,000 LOC, 1041 tests)
- **Phase 6**: Emitter (ES5 transforms, source maps, .d.ts) - 31% JS baseline
- **Phase 7**: Language Service (60%) - go-to-def, find refs, completions

## Emitter TODOs (for JS baseline 80%+)
- ⬜ Class inheritance (`__extends` helper for `extends`)
- ⬜ CommonJS exports (`module.exports`, `exports.X`)
- ⬜ Parse error tolerance (13 tests skipped)

## Language Service TODOs (40% remaining)
- ⬜ Formatting engine
- ⬜ Code fixes/refactorings

---

# 🎯 CURRENT FOCUS: Phase 8 - Baseline Compatibility

**Goal**: Match TypeScript's test baselines for `tests/cases/compiler`.

### Current Status
| Baseline | Pass Rate | Notes |
|----------|-----------|-------|
| .errors.txt | **66.2%** (51/77) | Focus area |
| .js emit | **36.8%** (28/76) | Fixed baseline comparison bug |

### Next Steps

#### Priority 0: Multi-thread work

##### Objective: Refactor to Shared Global Type Interner for Thread-Safe Deduplication

We are transitioning the compiler architecture from "Isolated State" (where every thread has its own type registry) to "Shared State" (a single global type universe). This is critical for memory efficiency and enabling O(1) global type equality checks across threads.

###### Architectural Changes

1.  **Hoist the Interner**: The `TypeInterner` must no longer be owned by the short-lived `ThinCheckerState`. It must be owned by the long-lived `MergedProgram` (or a similar global context) and passed by reference.
2.  **Thread Safety**: Ensure `TypeInterner` remains thread-safe (`Sync`). Currently, it uses `std::sync::RwLock`.
    *   *Performance Note:* Please switch imports to `parking_lot::RwLock` if available, or keep `std::sync::RwLock` but verify logic minimizes write-lock duration.

###### Specific Task Instructions

1. Update `src/solver/intern.rs`
- Ensure `TypeInterner` is fully thread-safe (it currently uses `RwLock`, which is correct for this phase).
- Verify that `intern()` operations utilize read-locks for the fast path and only upgrade to write-locks when a new type actually needs to be inserted.

2. Update `src/parallel.rs`
- Modify the `MergedProgram` struct:
  ```rust
  pub struct MergedProgram {
      pub type_interner: TypeInterner, // NEW: The global source of truth
      pub files: Vec<BoundFile>,
      // ... existing fields
  }
  ```
- Update `merge_bind_results` (or the equivalent construction site) to initialize a `TypeInterner::new()` and store it in `MergedProgram`.

3. Update `src/thin_checker.rs`
- Modify `ThinCheckerState<'a>`:
  - Change `pub types: TypeInterner` to `pub types: &'a TypeInterner`.
- Update `ThinCheckerState::new()` signature to accept `interner: &'a TypeInterner` instead of creating one internally.

4. Wire it up in `src/parallel.rs`
- In `check_functions_parallel`, inside the `par_iter()` loop:
  - Do NOT let the checker create a new interner.
  - Pass a reference to `program.type_interner` when initializing `ThinCheckerState`.

5. Fix Call Sites
- You will encounter compilation errors in tests or `thin_parser.rs` where `ThinCheckerState` is instantiated isolated.
- For isolated instances (like `ThinParser::check_source_file`), allow the caller to create a temporary `TypeInterner` and pass its reference, or refactor `ThinParser` to own an interner if it persists.

###### Criteria for Success
- `cargo check` passes.
- `ThinCheckerState` no longer owns `TypeInterner`.
- Multiple threads running `check_functions_parallel` are sharing the **same** underlying `HashMap` in the interner.
- `TypeId` equality works across different files/threads.



**Type Checking (26 failing tests)**
1. ✅ Export assignment validation (2309, 2304)
2. ✅ Setter parameter validation (1052, 1053)
3. ✅ Return type validation (2355) - function must return a value (basic types)
4. ⬜ Abstract class instantiation (2511) - cannot create instance of abstract
5. ⬜ Static member access from instance (2662) - `this.staticProp` in static method
6. ⬜ Abstract property validation (2715, 2729) - abstract in constructor
7. ⬜ Accessor return type inference (7023) - implicit any in getter

**Parser Semantic Errors**

8. ⬜ Declaration expected (1128) - after certain tokens
9. ⬜ Const modifier on class members (1248) - `const` invalid on properties
10. ⬜ Accessor body in ambient context (1183) - no body in declare class
11. ⬜ Accessor in ambient context ES5 (18045) - accessors need ES5+

**Advanced Diagnostics**

12. ⬜ RelatedInformation - point to definition sites for context
13. ⬜ Accessor diagnostic hints (6234) - "did you mean to call it?"

### Blockers Analysis (26 failing tests)
| Category | Codes | Tests | Notes |
|----------|-------|-------|-------|
| Parser errors | 1005, 1068, 1128, 1248 | 6 | Error recovery gaps |
| Type errors | 2339, 2355, 2511, 2662 | 12 | Property access, returns, abstract |
| Accessor errors | 1183, 6234, 18045 | 4 | Ambient context, hints |
| Abstract members | 2715, 2729, 2416, 2540 | 4 | Abstract property handling |

---

# Phase 9: Finishing up all TODOs ⬜

- ⬜ 100% baseline in all aspects
- ⬜ all todos left from previous phases
- ⬜ todos in code
- ⬜ missing unit tests

# Phase 10: Full Rust Mode ⬜

- ⬜ Remove TypeScript fallbacks
- ⬜ Performance benchmarks vs tsc and tsc-go
- ⬜ Memory usage optimization


# Phase 11: Prepare for Release ⬜

## Strategy
- **Track upstream**: Mirror TypeScript releases (5.x → 6.x)
- **Language features**: 100% compatible—no less, no more
- **API/CLI**: Match tsc behavior; extra flags allowed (e.g., `--wasm-threads`)

## TODOs
- ⬜ Upstream sync workflow (track `microsoft/TypeScript` releases)
- ⬜ Compatibility test suite (run against TS test baselines on each release)
- ⬜ Version alignment (match TS version numbers, e.g., `@mohsen1/typescript@5.7.0`)
- ⬜ CLI parity audit (`tsc --help` flags, exit codes, output format)
- ⬜ API compatibility layer (programmatic API matches `typescript` npm)
- ⬜ Packaging for npm (`@aspect/tsc-rust` or similar)
- ⬜ Packaging for cargo (`tsc-rust` crate)
- ⬜ Pre-built WASM binaries for major platforms
- ⬜ CI/CD release pipeline (GitHub Actions)
- ⬜ Documentation (migration guide, API docs, README)
- ⬜ Branding & naming
---

# Quick Reference

```bash
# Tests (Docker)
./wasm/test.sh

# Baseline comparison
node scripts/baseline-test-rust.mjs

# Build WASM
./wasm/build-wasm.sh
```
