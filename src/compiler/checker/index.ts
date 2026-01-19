/**
 * Thin Checker Module
 *
 * This module provides a lightweight type checker implementation that uses:
 * - Iterative flow analysis (not recursive)
 * - Constraint-based type solving
 * - Modular binder, solver, and evaluator components
 */

/* @internal */
namespace ts.thinChecker {
    // Re-export main types
    export type { ThinTypeChecker, SolverResult } from "../thinChecker";

    // Export factory function
    export { createThinTypeChecker } from "../thinChecker";
}
