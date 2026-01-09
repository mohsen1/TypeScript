# Project Zang

Project Zang is a performance-first TypeScript compiler in Rust.[^1]
The goal is a correct, fast, drop-in replacement for `tsc`, with both native and WASM targets.

TypeScript is intentionally unsound. Zang keeps a sound core solver and layers a compatibility
engine on top to match TypeScript behavior while preserving correctness where possible.

## Project Direction

> This is a very high level project direction coming from project's manager's boss.

**Current Phase: Phase 8 - Conformance, Convergence, and Hardening**

**Strategic Shift:** We have crossed the threshold of "building the engine." The components (ThinParser, ThinBinder, Solver, ThinEmitter) exist. We are now shifting to **Integration and Correctness**. We are no longer building features in isolation; we are driving the entire pipeline to pass official TypeScript conformance tests.

**Top Priority:** The **Solver** is the bottleneck for correctness (estimated 55% complete). The Emitter is 80% complete - **stop adding new Emitter features** and redirect effort to Solver and test coverage.

### ⚠️ Velocity Warning
We are coding faster than we can verify. Current velocity (~70 commits/day) is accumulating "verification debt." Many recent changes show regressions (`current behavior yields never`). **Slow down and gate harder.**

### Tactical Shift: Operation Crucible
- **Anvil Squad (Emitter):** Reduced to 2 workers. Focus ONLY on critical source map bugs and blocking ES5 regressions. **No new transforms.**
- **Forge Squad (Solver):** 5 workers. Primary focus: `solver/evaluate.rs` deferral logic for the Redux blocker.
- **Crucible Tasks (Test Porting):** 3 workers reassigned from Anvil. Their ONLY job:
  - Port conditional type and mapped type tests from official TypeScript repo into `tests/cases/`
  - Target: 50 new solver test cases this week
  - Goal: Give Forge workers failing tests to triangulate correct behavior

### Management Strategy: Autocratic Scheduling
- **The Manager** is the single source of truth for priority.
- **Tracks** are generic workers. If the Solver needs 3 workers, the Manager assigns 3 workers to the Solver, regardless of their previous "track name."
- **Zero-Idle:** If a high-priority task is blocked, swarm it.
- **Bisect-on-Merge:** PRs that regress ANY existing baseline are auto-rejected. No exceptions.

### Critical Objectives (Ranked)

1.  **Solver Hardening (The "Brain")**
    *   **🚨 BLOCKER: Redux/Lodash Generics** - `test_check_redux_lodash_style_generics` must pass. This test combines mapped types, conditional inference, AND cross-file resolution simultaneously. **Swarm this until it's green.**
    *   **ROOT CAUSE:** The bug is eager evaluation of conditional types when `InferenceVar`s are not yet bound. **FIX:** Refactor `solver/evaluate.rs` to introduce a `Deferred` state for `ConditionalResult`. When `check_type` contains an unbound `InferenceVar`, do NOT return `Any` or `Never`. Return a `TypeKey::Conditional` that preserves the constraint. Only evaluate when inference context is finalized.
    *   **Generic Inference:** `solver/infer.rs` is critical. Focus on inference from usage, context-sensitive typing, and handling circular constraints in `extends` clauses.
    *   **Conditional Types:** Stress test `solver/evaluate.rs` with distributive conditional types over unions. This is where most "toy" compilers fail.

2.  **Emitter Fidelity (The "Voice")** - MAINTENANCE MODE
    *   **⛔ NO NEW TRANSFORMS.** The Emitter is 80% complete. Stop feature work.
    *   **Bug fixes only:** Focus on critical source map bugs and blocking ES5 regressions (`super["m"]` in async, nested arrow `this` capture).
    *   **⚠️ Anti-pattern alert:** Do NOT use regex substitutions for code transforms. Always operate on AST.

3.  **Test Coverage (Crucible)**
    *   Port 50 conditional type and mapped type tests from official TypeScript repo.
    *   Every Solver fix must come with a regression test.
    *   **Metric:** Increase solver test coverage by 20% this week.

4.  **Performance Regression Check**
    *   Run `./wasm/bench.sh` regularly. The 500 MB/s throughput goal must not regress.
    *   **Metric:** Successfully compile `redux` or `lodash` types without panicking.

### Anti-Priorities (Do Not Work On)
*   **New Emitter transforms** (ES3, obscure module formats) - we have enough
*   New LSP features (Semantic Tokens, Code Actions) unless they expose a Solver bug
*   CLI argument parsing or fancy terminal output
*   Performance micro-optimizations (unless we regress significantly)


## Executive Summary (Director report)
Last updated: 2026-01-09

### Conformance Metrics (Primary KPI)
| Metric | Value | Target |
|--------|-------|--------|
| Exact Match | 17.7% (86/487) | 50%+ |
| Missing Errors | 70.0% | <30% |
| Extra Errors (False Positives) | 47.2% | <20% |
| Build Status | Passing (4771/4789) | Green |

### Current Squad Structure
- **Squad Forge (5 workers)**: Missing error implementation (TS2454, TS2564, TS7006, TS2792, TS2339, TS2300)
- **Squad Anvil (5 workers)**: False positive elimination (TS2304, TS2322, TS2339, TS2769, TS2355)

### Top Missing Errors (Forge Focus)
- TS2564: 64 occurrences (property initialization)
- TS2454: 43 occurrences (definite assignment) - control flow analysis IMPLEMENTED
- TS7006: 42 occurrences (implicit any)
- TS2705: 37 occurrences
- TS2322: 19 occurrences (type assignability)

### Top False Positives (Anvil Focus)
- TS2304: 136 occurrences (cannot find name)
- TS2355: 82 occurrences (return analysis)
- TS1005: 65 occurrences (parser) - parser fixes IMPLEMENTED
- TS2339: 35 occurrences (property access)
- TS1109: 25 occurrences (parser)

### Recent Progress (Jan 9)
- TS2454 control flow analysis (+315 lines in checker/control_flow.rs)
- TS1005/TS1068 parser fixes (+2568 lines in thin_parser.rs)
- Spread argument expansion fix (+263 lines)
- Inherited property access fix
- DI tests (+674 lines), template literal tests (+388 lines)

### Direction
Conformance-driven development. Both squads working in parallel on orthogonal error codes.
- Forge: Implements missing error checks TSC catches that we don't
- Anvil: Eliminates false positives we report that TSC doesn't
- Director update: merged latest `origin/squad/forge` and `origin/squad/anvil` into `origin/rust`.
- Cross-squad FYI: Forge reported async ES5 fixes in `worker/forge-3` (commit `60da72008ad`) and `worker/forge-5` (commit `ff4c28be4e6`) touching `wasm/src/transforms/async_es5.rs`; coordinate with Anvil if reimplementation or cherry-pick is needed.
- Director update: merged latest Forge updates into `origin/rust` (solver evaluate + thin_checker + parallel tests).
- Tracks: Solver added numeric index name edge-case tests (trailing decimal, leading plus, missing exponent sign, leading zero decimal, hex/binary/octal literals, leading-zero mantissa exponent, leading dot decimal, multiple leading zeros, negative hex/binary/octal literals, uppercase exponent name, uppercase exponent missing sign, uppercase exponent leading zeros, uppercase exponent missing digits, uppercase exponent minus missing digits, uppercase exponent double sign, uppercase exponent double minus, uppercase exponent negative leading zeros, mixed-case exponent, mixed-case exponent with sign, mixed-case exponent missing digits, mixed-case exponent double sign, mixed-case exponent double minus, mixed-case exponent plus minus, mixed-case exponent minus plus, mixed-case exponent trailing sign, mixed-case exponent trailing minus, mixed-case exponent with lowercase e, uppercase exponent missing sign with leading zero, negative exponent leading zeros, positive exponent leading zeros, exponent leading zeros without sign, uppercase exponent leading zeros without sign, exponent double sign, exponent double minus, exponent missing digits, exponent minus missing digits) and conditional-type regressions for object properties, object call-signature infer (current behavior yields never), template literal infer from `string`/`` `${string}` `` (current behavior yields never), template literal prefix infer (current behavior yields never), template literal suffix infer (current behavior yields never), template literal middle infer (current behavior yields never), template literal two infers (current behavior yields never), template literal constrained infer (current behavior yields never), non-distributive template literal infer (current behavior yields never), non-distributive template literal constrained infer (current behavior yields never), non-distributive template literal constrained prefix infer (current behavior yields never), non-distributive template literal constrained middle infer (current behavior yields never), non-distributive template literal middle infer (current behavior yields never), non-distributive template literal two infers (current behavior yields never), non-distributive template literal suffix infer (current behavior yields never), non-distributive template literal prefix infer (current behavior yields never), non-distributive template literal prefix infer with non-matching union branch (current behavior yields never), non-distributive template literal union input (current behavior yields never), non-distributive template literal non-string union branch (current behavior yields never), non-distributive template literal constrained two-infer (current behavior yields never), non-distributive template literal constrained suffix infer (current behavior yields never), optional tuple element inference, non-distributive optional tuple infer TODO, non-distributive optional property infer TODO, optional tuple element array inference (omits undefined), function this-parameter inference TODO, optional property present inference (omits undefined), tuple rest inference (current behavior yields number), union true/false-branch infer preservation, any-check infer preservation, non-distributive function parameter infer (current behavior yields never), function optional-parameter infer (current behavior yields never), function param infer with non-function union branch (current behavior yields never), non-distributive function this-parameter infer (current behavior yields never), non-distributive function return infer (current behavior yields never), function rest parameter infer (current behavior yields never), optional property infer with constraint, and optional tuple element infer with constraint. Subtype unsoundness coverage expanded with primitive boxing (string/symbol), recursion depth limiter provisional subtyping, mapped key remap to `never` yielding empty object, mapped optional modifier add yields optional properties, mapped readonly modifier add yields readonly properties, mapped optional modifier remove yields required properties, mapped optional remove over optional keyof yields required properties, mapped readonly remove over readonly keyof yields mutable properties, mapped optional+readonly add yields optional readonly properties, mapped optional+readonly remove yields mutable required properties, mapped key remap optional add yields optional properties, mapped key remap readonly add yields readonly properties, mapped key remap optional+readonly add yields optional readonly properties, mapped key remap readonly remove yields mutable properties, mapped key remap optional remove retains optional properties, mapped readonly modifier remove yields mutable properties (current behavior allows readonly subtype), `keyof` union optional keys regression, `keyof` union + string index literal narrowing, `keyof` intersection union-of-keys, `keyof` any union regression, apparent string numeric index signature subtyping, apparent string length property subtyping, template literal apparent member subtyping, template literal number index signature subtyping, index signature consistency number vs string index, no-unchecked indexed access tuple/string/union index subtyping, and keyof contravariant object subtyping. Emitter added ES5 regressions for derived-field `super`, ctor-arrow `super`, combined field arrow `super` + `this` capture, class method arrow super + `this` capture, class method nested arrow super + `this` capture, class method arrow super + arguments capture, async class method arrow arguments capture, async class method arrow super + this capture, async class method arrow super + arguments capture, async class method arrow super + this + arguments capture, async class nested arrow super + this capture, async class nested arrow super + this + arguments capture, async class nested arrow super + arguments capture, async class nested arrow computed super (current behavior leaves `super["m"]`), async class nested arrow computed super + this capture (current behavior leaves `super["m"]`), async class nested arrow computed super + this + arguments capture (current behavior leaves `super["m"]`), async class computed super with this + arguments (current behavior yields `void 0["m"]`), async class nested arrow computed super + arguments (current behavior leaves `super["m"]`), async class method returns arrow with super method + arguments, async class method returns arrow with super method + this capture, async class method returns arrow with super method (no args), async class method returns arrow with computed super key + no args (current behavior drops body), async class method returns arrow with computed super + arguments (current behavior leaves `super["m"]`), async class method returns arrow with computed super + this + arguments (current behavior leaves `super["m"]`), async class method returns nested arrow with computed super key + this + arguments (current behavior drops body), async class method arrow super call without args, async class nested arrow super call without args, computed `super["m"]` in field arrows (fix landed), computed super method regression, and async computed super method regression (current behavior leaves `void 0`). Source-map coverage added async ES5 for-of/for-in await RHS plus array/object literal await, nested await call, template literal await, and binary add/multiply/subtract/divide/modulo/less-than/strict-equal/strict-not-equal/equal/not-equal/greater-equal/less-equal/greater-than/bitwise-and/bitwise-or/bitwise-xor/destructuring/shift-left/shift-right/unsigned shift-right/logical-or/logical-and/logical-or complex/logical-and both-awaits/ternary await-condition/try-catch-finally await-in-finally/try-catch-finally catch+finally awaits mapping tests, async assignment await mapping, async compound assignment multiply await mapping, async compound assignment divide await mapping, async compound assignment subtract await mapping, async compound assignment modulo await mapping, async nullish coalescing await mapping, async unary not await mapping, and async unary negative await mapping.
- Baselines (`tests/cases`, first 100): no new run since 2026-01-07; last known compiler errors 60/77 (77.9%) pass, JS 40/76 (52.6%) pass; conformance errors 18/90 (20.0%) pass, JS 1/88 (1.1%) pass.
- Risk: `cli/driver.rs` E0515 fix for package.json parsing is in `rust` and a small ./wasm/test.sh filter now passes, but broader Docker test runs are still pending; complete more filters before resuming heavy work. ES5 computed `super["m"]` lowering fix just landed; async computed `super["m"]` in ES5 currently emits `void 0["m"]` and test asserts current behavior with a TODO (needs async_es5 fix); ensure class_es5.rs edits are clean after regex substitutions; tuple rest conditional infer test expectation updated to match current behavior (number); ES5 `super` lowering now routes through `_super.prototype.*.call` with arrow `this` capture; needs more emitter regressions to guard against call-site changes; full conformance baselines not re-run; parser/arena child enumeration TODOs remain; CLI typesVersions compiler version is hardcoded; conformance baseline pass rates are low.
- Next focus: re-run queued worker tests now that E0515 is resolved, then resume inference bounds edge cases for numeric index names (worker-1), conditional infer edge cases around tuple/template/object/function properties (worker-2), next subtype unsoundness from catalog (worker-3), ES5 emitter regressions around computed super and nested arrow capture (worker-4), and async ES5 source-map coverage for loop/try constructs (worker-5).

## Status
This project is not ready for general use yet. The interface and distribution are in progress.

## Planned distribution
- `tsz` CLI (native binaries for major operating systems)
- Rust crate `tsz` (library + CLI)
- WASM bindings
- npm package `@tsz/tsz` (primary)
- compat package `@tsz/tsc` that exposes a `tsc` executable so tooling can swap without noticing
- Playground

## Guiding principles
- Make tsz boringly correct before it is fast. Parity with `tsc` output, errors, and edge cases is the trust anchor.
- Measure everything: benchmark real repos, gate regressions, and only optimize hot paths that move real workloads.
- Determinism wins adoption: stable outputs, stable diagnostics, stable perf, and zero "sometimes" behavior.
- Incrementality is a feature, not a refactor: design caches and query boundaries up front so LSP/CLI stay snappy.
- UX matters as much as speed: error messages, source maps, and CLI flag compatibility are what teams feel every day.
- Keep the architecture clean and enforced. Performance-first is a habit, not a phase.


[^1]: Zang is Persian for rust.
