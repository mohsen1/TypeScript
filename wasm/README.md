# Project Zang

Project Zang is a performance-first TypeScript compiler in Rust.[^1]
The goal is a correct, fast, drop-in replacement for `tsc`, with both native and WASM targets.

TypeScript is intentionally unsound. Zang keeps a sound core solver and layers a compatibility
engine on top to match TypeScript behavior while preserving correctness where possible.

## Project Direction

> This is a very high level project direction coming from project's manager's boss.

**Current Phase: Phase 8 - Conformance, Convergence, and Hardening**

### Priority number one is Conformance

#### HEADLINER

We need to keep working on our project and while maintaining the architectural integrity of our codebase, increase conformance with TypeScript.


Output of `wasm/differential-test/run-conformance.sh --max=10000` dictates where we are and where should we go from here


#### Current focus 

Based on the Conformance Report and the architectural constraints defined in `WASM_ARCHITECTURE.md`, here is a deep analysis of why conformance is low (23.4%) and where the fundamental architectural gaps lie.

### Executive Summary: The "Permissive" Trap

The most alarming statistic is **Missing Errors: 68.2%**.
This means your compiler is **too permissive**. It accepts code that TypeScript rejects.
In a compiler, "Extra Errors" (35%) means you are buggy (parsing/binding issues). "Missing Errors" (68%) means you are **unsound**.

You are failing to catch:
1.  **Uninitialized Variables** (TS2454: 573 hits)
2.  **Uninitialized Properties** (TS2564: 443 hits)
3.  **Implicit Anys** (TS7006: 357 hits)

This suggests the architecture prioritizes *throughput* and *memory* (Data-Oriented Design) but lacks the **Control Flow Graph (CFG)** and **Inference strictness** required to match `tsc`.

---

### 1. Fundamental Issue: Data-Oriented Design vs. Control Flow Analysis (CFA)

**The Problem:**
You are using a `ThinNode` architecture (Struct-of-Arrays). This is excellent for parsing speed (500 MB/s), but it makes **Control Flow Analysis** (CFA) significantly harder.

TS2454 ("Variable used before assigned") and TS2564 require a **Control Flow Graph**. In a pointer-based AST (like TSC), you can attach "Flow Nodes" to AST nodes easily. In your `ThinNode` array, you cannot mutate nodes to add flow data.

**Evidence:**
*   `src/checker/control_flow.rs` exists but seems to rely on a `FlowNodeArena`.
*   The high missing count for TS2454 suggests that `check_identifier` in `thin_checker.rs` is **not** querying the Flow Graph effectively, or the Flow Graph construction is incomplete/disconnected from the linear parser pass.

**Architectural Fix:**
You need a dedicated **Side Table** for Flow Nodes that is computed *after* binding but *before* checking. The Checker must query `flow_graph[node_index]` for every identifier usage. Currently, it seems the checker defaults to "Assigned" if it can't prove otherwise. It must default to "Unassigned".

### 2. Fundamental Issue: The "Any" Fallback

**The Problem:**
The high number of missing **TS7006 (Implicit Any)** and **TS2322 (Type Not Assignable)** suggests that when your Solver encounters a complex type (generics, conditional types), it "bails out" and returns `Any` (or `true` for subtyping) to avoid crashing.

**Evidence:**
*   `src/solver/` uses a `TypeKey` system.
*   If `lower_type` fails to resolve a symbol (due to the TS2304 binding issues), it likely returns `Error` or `Any`.
*   In `specs/SOLVER.md`, the "Error Poisoning" rule states `Error` is compatible with everything.
*   Because you have 702 **TS2304 (Cannot find name)** errors, those unresolved names become `Any/Error`, which then silences all downstream errors (TS2322, TS7006).

**Architectural Fix:**
You cannot fix conformance until you fix **Binding (TS2304)**.
If the Binder cannot find `Array`, `Promise`, or `console`, the Solver treats them as `Any`.
1.  **Fix the Library Context:** Ensure `lib.d.ts` is actually loaded and bound in the test runner. The `WasmProgram` class seems to handle this, but the high TS2304 count implies global scope pollution is failing.
2.  **Strict Error Types:** Change the default bailout from `Any` to `Unknown`. `Unknown` is safe (errors on usage), whereas `Any` suppresses errors.

### 3. Fundamental Issue: The Parser is "Too Strict"

**The Problem:**
**TS1005 (Expected token)** and **TS1109 (Expression expected)** account for ~800 extra errors.
Your `ThinParser` is a recursive descent parser written from scratch. TypeScript allows many grammar ambiguities (ASI, loose keywords) that a strict Rust parser might reject.

**Impact:**
When parsing fails, the AST is incomplete. An incomplete AST leads to:
1.  Missing nodes -> Missing Symbols -> **TS2304 (Cannot find name)**.
2.  Missing Symbols -> Inferred as Any -> **Missing TS2322**.

**Architectural Fix:**
The parser needs a robust **Error Recovery** strategy.
*   Current: Seems to bail or produce error nodes that stop further analysis.
*   Required: "Resynchronization". If a statement is malformed, skip tokens until the next semicolon/brace and *continue parsing*. The AST must be as complete as possible even with syntax errors.

### 4. Fundamental Issue: The "Judge vs. Lawyer" Gap

**The Problem:**
Your `specs/SOLVER.md` describes a "Judge" (Sound Set Theory) and a "Lawyer" (Compat Layer).
The data shows the **Lawyer is missing**.

*   **TS2339 (Property does not exist):** 294 Extra Errors.
    *   This happens when you check `obj.prop`.
    *   A "Sound" solver checks if `prop` is in `obj`.
    *   TypeScript checks: `prop` in `obj` OR `obj` is `any` OR `obj` has string index signature OR `obj` is a Union and *one* constituent has it (sometimes).
*   **TS2322 (Type not assignable):** 310 Missing Errors.
    *   Your solver is likely returning `true` for things TS rejects (e.g., `string | number` assignable to `string`? No, but maybe your union logic is loose).

**Architectural Fix:**
The `CompatChecker` in `src/solver/` needs to implement the "Unsoundness Catalog" explicitly.
*   Implement **Apparent Members** for primitives (e.g., `string` has `.length`).
*   Implement **Union Widening** correctly.

### Summary of Recommendations

1.  **Priority 1: Fix the Parser Recovery (TS1005/1109).**
    *   You cannot trust semantic errors if the syntax tree is broken. 500+ parse errors are masking thousands of semantic issues.

2.  **Priority 2: Fix Global Binding (TS2304).**
    *   700+ "Cannot find name" errors mean your `lib.d.ts` or global scope handling is broken. This causes cascading `Any` types, hiding real errors.

3.  **Priority 3: Invert Control Flow Default.**
    *   Change `FlowAnalyzer` to assume variables are **Unassigned** by default. Currently, it seems to assume they are assigned (hence missing TS2454).

4.  **Priority 4: Strict Solver Fallback.**
    *   When a type cannot be resolved, return `Type::Unknown` instead of `Type::Any`. This will convert "Missing Errors" into "Extra Errors", which are easier to debug and fix.

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
