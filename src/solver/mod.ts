/**
 * Type Solver Module
 *
 * A lock-free type resolution system that avoids recursive RwLock deadlocks.
 *
 * Key design principles:
 * 1. No locks held during recursive type resolution
 * 2. Cycle detection via resolution stack tracking
 * 3. Atomic status transitions for cache entries
 * 4. Snapshot-based lookups (copy-on-read)
 *
 * Usage:
 *   const evaluator = createTypeEvaluator();
 *   const type = evaluator.resolveType(typeNode, checker);
 */

/// <reference path="../compiler/types.ts" />
/// <reference path="types.ts" />
/// <reference path="evaluator.ts" />

namespace ts.solver {
    /**
     * Facade for the type solver system
     * Provides a simplified API for common operations
     */
    export class TypeSolver {
        private readonly evaluator: TypeEvaluator;
        private readonly checker: TypeChecker;

        constructor(checker: TypeChecker, config?: Partial<TypeEvaluatorConfig>) {
            this.checker = checker;
            this.evaluator = createTypeEvaluator(config);
        }

        /**
         * Resolves a type node to a Type
         */
        public resolve(node: TypeNode): Type | undefined {
            return this.evaluator.resolveType(node, this.checker);
        }

        /**
         * Resolves a type node with a custom context
         */
        public resolveWithContext(node: TypeNode, context: ResolutionContext): Type | undefined {
            return this.evaluator.resolveType(node, this.checker, context);
        }

        /**
         * Instantiates a generic type with type arguments
         */
        public instantiate(genericType: Type, typeArguments: readonly Type[]): Type | undefined {
            const context = createResolutionContext();
            return this.evaluator.instantiateType(genericType, typeArguments, this.checker, context);
        }

        /**
         * Performs speculative type resolution (doesn't cache failures)
         */
        public speculativeResolve(node: TypeNode): Type | undefined {
            const context = createResolutionContext();
            context.isSpeculative = true;
            return this.evaluator.resolveType(node, this.checker, context);
        }

        /**
         * Clears all internal caches
         */
        public clearCaches(): void {
            this.evaluator.clearCaches();
        }

        /**
         * Gets cache statistics
         */
        public getStats(): { typeCache: number; instantiationCache: number } {
            return this.evaluator.getCacheStats();
        }

        /**
         * Creates a child solver with shared caches but isolated context
         */
        public createChild(): TypeSolver {
            // For now, create a new solver. In the future, we could share caches.
            return new TypeSolver(this.checker);
        }
    }

    /**
     * Creates a new TypeSolver instance
     */
    export function createTypeSolver(checker: TypeChecker, config?: Partial<TypeEvaluatorConfig>): TypeSolver {
        return new TypeSolver(checker, config);
    }

    /**
     * Batch type resolution helper
     * Resolves multiple type nodes efficiently
     */
    export function resolveTypes(
        nodes: readonly TypeNode[],
        checker: TypeChecker,
        config?: Partial<TypeEvaluatorConfig>
    ): (Type | undefined)[] {
        const evaluator = createTypeEvaluator(config);
        const context = createResolutionContext(config?.maxResolutionDepth);

        return nodes.map(node => evaluator.resolveType(node, checker, context));
    }

    /**
     * Checks if a type resolution would cause a cycle
     * Useful for pre-flight checks before expensive operations
     */
    export function wouldCauseCycle(
        cacheId: TypeCacheId,
        context: ResolutionContext
    ): boolean {
        return context.resolutionStack.has(cacheId);
    }

    /**
     * Safe type resolution with cycle handling
     * Returns a fallback type on cycle detection
     */
    export function safeResolve(
        node: TypeNode,
        checker: TypeChecker,
        fallback: Type
    ): Type {
        const evaluator = createTypeEvaluator();
        const result = evaluator.resolveType(node, checker);
        return result ?? fallback;
    }
}
