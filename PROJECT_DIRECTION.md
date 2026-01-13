
## Project Direction

**Current Phase: Phase 8 - Conformance, Convergence, and Hardening**

*Last Updated: January 2026*

### The Big Picture

> "We built a Ferrari, but the steering wheel is loose."

The architectural foundation (Data-Oriented Design, ThinNodes, SoA) is proving to be fast and memory-efficient. We have a high-performance compiler infrastructure that runs well in WASM. However, semantically, we are **too permissive**—an "optimistic" compiler that accepts code `tsc` rejects because our solver defaults to `Any` when things get hard.

### Current Status: 30.1% Exact Match

| Metric | Value | Notes |
|--------|-------|-------|
| **Exact Match** | 30.1% | 1488/4939 tests |
| **Missing Errors** | 60.0% | 2961 tests - compiler too permissive |
| **Extra Errors** | 30.9% | 1528 tests - false positives |
| **Performance** | 41.7 tests/sec | ✓ |
| **Stability** | Good | 2 crashes (stack overflow) |

### Component Status Overview

| Component | Status | Notes |
|-----------|--------|-------|
| **Parser/Scanner** | 🟠 **Needs Work** | **701 false positive errors** (TS1005 + TS1109) polluting measurements. |
| **Emitter** | ✅ Done | Fast, Source Maps work, ES6→ES5 downleveling complete. |
| **LSP** | ✅ Working | Go-to-def, Rename, Hover, Semantic Tokens via ScopeWalker. |
| **CFA** | 🟡 Partial | Landed but TS2564 still #1 missing (413), TS2454 still has 225 extra. |
| **Binder** | 🔴 Critical | TS2304 is both missing (116) AND extra (343). Error poisoning. |
| **Solver** | 🟠 In Progress | Switching default fallback from `Any` to `Unknown` to expose bugs. |

---

### Priority #1: Stop the "Any" Poisoning

The biggest enemy is **Error Poisoning**. When the Binder fails to find `Promise` (because `lib.d.ts` didn't load right), the Solver says "Oh, it's `Any`". This silences *all* downstream errors. We think we're compiling correctly, but we're failing silently.

**Immediate Action:** Force the solver to return `Error` or `Unknown` instead of `Any` to expose these bugs.

---

### Squad Directives

#### 0. Parser/Scanner - The "Syntax" Squad 🟠 NEW PRIORITY
*   **Problem:** **701 false positive errors** (TS1005: 439, TS1109: 262) polluting all measurements.
*   **Impact:** These parser errors mask real progress and inflate "Extra Errors" by 14%.
*   **Directive:** Fix parser error emission.
    *   Audit TS1005 ("expected X") emission - likely over-triggering on valid syntax.
    *   Audit TS1109 ("expression expected") - false positives on edge cases.
    *   **Goal:** Reduce parser false positives to <100.

#### 1. Control Flow Analysis (CFA) - The "Flow Graph" Squad 🟡 PARTIAL
*   **Status:** Flow Graph Side-Table implemented and landed.
*   **Reality Check:** TS2564 is still the #1 missing error (413 occurrences). TS2454 has 225 extra errors.
*   **Remaining Work:** The framework is in place, but edge cases need significant work.

#### 2. Binding & Scope Resolution - The "Binder" Squad 🔴 CRITICAL
*   **Problem:** **TS2304** (Cannot find name) remains the #1 source of error poisoning.
*   **Impact:** When the Binder fails to resolve `console`, `Promise`, or `Array`, the Solver defaults to `Any`, suppressing all downstream errors.
*   **Directive:** Fix Global Scope and Lib injection.
    *   Ensure `lib.d.ts` symbols are correctly merged into the root `SymbolTable`.
    *   Fix module augmentation resolution (merging `interface Window` across files).
    *   Debug why basic globals like `console` and `Array` still fail to resolve.
    *   **Goal:** Reduce TS2304 extra errors to <50.

#### 3. Solver Strictness - The "Semantics" Squad 🟠 IN PROGRESS
*   **Problem:** **TS2322** (Type not assignable) and **TS7006** (Implicit Any) still being missed.
*   **Architectural Change:** Switching default fallback from `Any` to `Unknown`.
*   **Directive:** Harden the `solve_subtype` logic.
    *   Implement the "Lawyer" layer from `specs/SOLVER.md` for TypeScript's quirks (function bivariance, void return exceptions).
    *   Stop being nice—the compiler needs to be meaner (stricter) to match `tsc`.
    *   **Goal:** Convert "Missing TS2322" into "Exact Match" or "Extra TS2322" (better to be too strict than unsound).

---

### What's Working Well

1.  **Architecture Validated:** The decision to use `ThinNode` (16-byte structs) and arenas instead of pointer-based ASTs was correct. It makes the LSP and Emitter extremely snappy in WASM.
2.  **Squad Model:** Splitting into CFA, Binder, and Solver squads paid off. The CFA squad delivered a massive compliance jump without touching the AST structure (using side-tables).
3.  **Error Recovery:** The parser is resilient. It doesn't bail on syntax errors, allowing the LSP to function even in broken files.

---

### Anti-Priorities (Do Not Work On)

*   **Performance Optimization:** Current speed is sufficient. Do not optimize hot paths until correctness > 50%.
*   **New Emitter Features:** Downleveling logic is stable. Focus on the Checker.
*   **LSP Polish:** No new code actions until the semantic model is accurate.

---

### Resource Allocation (10 Engineers)

*   **Parser Squad (2):** Fix TS1005/TS1109 false positives. *(new - unblocks accurate measurement)*
*   **CFA Squad (1):** Edge-case polish for TS2564/TS2454. *(reduced - framework done, needs polish)*
*   **Binder Squad (4):** Scope resolution, `lib.d.ts` integration, module resolution. *(critical path)*
*   **Solver Squad (3):** Subtyping logic, generic inference, error message parity.

---

### Success Metrics for Next Milestone

| Metric | Current | Target |
|--------|---------|--------|
| **Exact Match** | 30.1% | **40%** |
| **Missing Errors** | 60.0% | **<50%** |
| **Parser false positives** | 701 | **<100** |
| **TS2304 extra errors** | 343 | **<50** |

---

### Verdict

We are no longer building a "toy" compiler—the skeleton is complete. We are now in the **grind phase**, chasing the long tail of TypeScript's semantic behavior.

**Immediate Focus:** Stop being nice. The compiler needs to become meaner (stricter) to match `tsc`. Fix global scope binding to stop the "Any" poisoning.

---

### Code Hygiene

**Code outside of `wasm/` should be read-only.**

If any changes were made outside of `wasm/`, please double check and revert if it was a mistake. We want to keep the TypeScript codebase pristine. The git remote `git@github.com:microsoft/TypeScript.git` is the source of truth for TypeScript code. 