
## Project Direction

**Current Phase: Phase 8 - Conformance, Convergence, and Hardening**

### Priority #1: Closing the Unsoundness Gap

The latest conformance report (Jan 2026) indicates a **23.4% Exact Match** rate. More critically, **68.2% of tests have missing errors**. The compiler is currently too permissive, accepting invalid TypeScript code.

We are mobilizing 10 engineers to address the three fundamental pillars of unsoundness identified in the report:

#### 1. Control Flow Analysis (CFA) - The "Flow Graph" Squad
*   **Problem:** The highest missing error counts are **TS2454** (Variable used before assigned: 573 hits) and **TS2564** (Property not initialized: 443 hits).
*   **Architectural Gap:** Our `ThinNode` (SoA) architecture makes tree traversal efficient but lacks the stateful flow tracking of `tsc`.
*   **Directive:** Implement a **Flow Graph Side-Table**.
    *   Do *not* mutate AST nodes.
    *   Build a separate `FlowGraph` vector post-binding.
    *   Implement `check_flow_usage` in the Checker to query this graph.
    *   **Goal:** Reduce TS2454/TS2564 missing errors by 90%.

#### 2. Binding & Scope Resolution - The "Binder" Squad
*   **Problem:** **TS2304** (Cannot find name) is the #1 extra error (702 hits).
*   **Impact:** When the Binder fails to resolve `console`, `Promise`, or `Array`, the Solver defaults the type to `Any` (Error Poisoning). This suppresses downstream errors (causing the high missing error count for TS2322 and TS7006).
*   **Directive:** Fix Global Scope and Lib injection.
    *   Ensure `lib.d.ts` symbols are correctly merged into the root `SymbolTable`.
    *   Fix module augmentation resolution (merging `interface Window` across files).
    *   **Goal:** Reduce TS2304 extra errors to <50.

#### 3. Solver Strictness - The "Semantics" Squad
*   **Problem:** **TS2322** (Type not assignable) is missing in 310 cases, and **TS7006** (Implicit Any) in 357 cases.
*   **Architectural Gap:** The Solver currently favors "bailing out" to `Any` or `True` when encountering complex generics or unknown symbols.
*   **Directive:** Harden the `solve_subtype` logic.
    *   Change the default fallback from `Any` to `Unknown` to force errors on safe failures.
    *   Implement the "Lawyer" layer from `specs/SOLVER.md` to handle `Any` propagation correctly (it should not silence structural mismatches unless explicitly required).
    *   **Goal:** Convert "Missing TS2322" into "Exact Match" or at least "Extra TS2322" (it is better to be too strict than unsound).

### Anti-Priorities (Do Not Work On)
*   **Performance Optimization:** 72.5 tests/sec is sufficient for this phase. Do not optimize hot paths until correctness > 50%.
*   **New Emitter Features:** Downleveling logic is stable enough. Focus on the Checker.
*   **LSP Polish:** No new code actions or renaming features until the semantic model is accurate.

### Resource Allocation (10 Engineers)
*   **CFA Squad (3):** Flow Graph construction, definite assignment analysis.
*   **Binder Squad (3):** Scope resolution, `lib.d.ts` integration, module resolution.
*   **Solver Squad (4):** Subtyping logic, generic inference, error message parity.

### Success Metrics for Next Milestone
1.  **Exact Match:** Increase from 23.4% -> **35%**.
2.  **Missing Errors:** Decrease from 68.2% -> **<40%**.
3.  **TS2304 (Cannot find name):** Eliminate false positives.