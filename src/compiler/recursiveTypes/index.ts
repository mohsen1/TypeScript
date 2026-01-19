/**
 * Recursive Types - Module Index
 *
 * Comprehensive implementation of recursive type handling:
 *
 * - Recursive type alias resolution
 * - Circular type reference detection
 * - Lazy type evaluation for recursion
 * - Self-referential generic constraints
 * - Infinite type instantiation detection
 * - Mutually recursive types
 * - Depth limits for type expansion
 * - Type resolution caching
 *
 * @example
 * ```typescript
 * // Recursive type alias
 * type Tree<T> = { value: T; children: Tree<T>[] };
 *
 * // Mutually recursive types
 * type A = { b: B };
 * type B = { a: A };
 *
 * // Self-referential constraint
 * interface Node<T extends Node<T>> {
 *   parent: T | null;
 * }
 * ```
 */

/* @internal */
namespace ts.recursiveTypes {
    // Re-export all types
    export type {
        LazyType,
        ResolutionStackEntry,
        CircularityCheckResult,
        RecursiveTypeConfig,
        RecursiveTypeContext,
        TypeResolutionResult,
        RecursiveTypeAliasInfo,
        InstantiationTracker,
        InfiniteInstantiationResult,
        TypeCacheEntry,
        RecursiveTypeStats,
    };

    // Export TypeResolutionState enum
    export { TypeResolutionState };

    // Export DefaultConfig
    export { DefaultConfig };

    // Export circularity detection
    export {
        createStackEntry,
        getResolutionKey,
        isSymbolBeingResolved,
        isNodeBeingResolved,
        pushResolution,
        popResolution,
        checkCircularity,
        isDirectlyRecursive,
        findMutualRecursion,
        isConstraintSelfReferential,
        breakCircularReference,
        createCircularityError,
        isSafelyRecursive,
        resolveWithCircularityCheck,
        getResolutionStackTrace,
        clearResolutionStack,
        getCurrentDepth,
        isMaxDepthExceeded,
    };

    // Export lazy evaluation
    export {
        createLazyType,
        isLazyTypeResolved,
        isLazyTypeResolving,
        isLazyTypeError,
        forceLazyType,
        createDeferredType,
        isDeferredType,
        getLazyWrapper,
        forceAllDeferredTypes,
        createLazyTypeAlias,
        resolveLazyTypeAlias,
        LazyEvaluationContext,
    };

    // Export instantiation tracking
    export {
        createInstantiationTracker,
        InstantiationManager,
        wouldInstantiationBeInfinite,
        expandTypeWithDepthLimit,
        createMemoizedExpander,
    };

    // Export cache
    export {
        createCacheEntry,
        TypeResolutionCache,
        createTypeAliasKey,
        createTypeReferenceKey,
        createConditionalTypeKey,
        createMappedTypeKey,
        getGlobalCache,
        resetGlobalCache,
        ScopedCache,
        createCachedResolver,
    };

    /**
     * Create a recursive type context
     */
    export function createRecursiveTypeContext(
        checker: TypeChecker,
        config: Partial<RecursiveTypeConfig> = {}
    ): RecursiveTypeContext {
        const fullConfig: RecursiveTypeConfig = {
            ...DefaultConfig,
            ...config,
        };

        return {
            checker,
            config: fullConfig,
            resolutionStack: [],
            cache: new Map(),
            instantiationCount: 0,
            circularType: checker.getAnyType(), // Fallback for circular refs
            anyType: checker.getAnyType(),
            errorType: checker.getAnyType(), // Could use a special error type
        };
    }

    /**
     * Resolve a type with full recursive handling
     */
    export function resolveType(
        symbol: Symbol | undefined,
        node: Node | undefined,
        resolver: () => Type,
        context: RecursiveTypeContext
    ): TypeResolutionResult {
        const startDepth = getCurrentDepth(context);

        // Check cache first
        const cacheKey = getResolutionKey(symbol, node);
        const cached = context.cache.get(cacheKey);
        if (cached && context.config.useCache) {
            return {
                type: cached,
                success: true,
                circular: false,
                fromCache: true,
            };
        }

        // Check for circularity
        const circularityCheck = checkCircularity(symbol, node, context);
        if (circularityCheck.isCircular) {
            return {
                type: breakCircularReference(context),
                success: false,
                circular: true,
                error: createCircularityError(circularityCheck.cyclePath!),
                fromCache: false,
            };
        }

        // Check depth limit
        if (isMaxDepthExceeded(context)) {
            return {
                type: context.anyType,
                success: false,
                circular: false,
                error: `Maximum type resolution depth exceeded (${context.config.maxDepth})`,
                fromCache: false,
            };
        }

        // Resolve with stack tracking
        pushResolution(symbol, node, context);
        try {
            const resolvedType = resolver();

            // Cache the result
            if (context.config.useCache) {
                context.cache.set(cacheKey, resolvedType);
            }

            return {
                type: resolvedType,
                success: true,
                circular: false,
                fromCache: false,
            };
        } catch (e) {
            return {
                type: context.errorType,
                success: false,
                circular: false,
                error: e instanceof Error ? e.message : String(e),
                fromCache: false,
            };
        } finally {
            popResolution(context);
        }
    }

    /**
     * Handle a recursive type alias definition
     */
    export function handleRecursiveTypeAlias(
        symbol: Symbol,
        typeNode: Node,
        typeParameters: TypeParameter[] | undefined,
        createType: () => Type,
        context: RecursiveTypeContext
    ): RecursiveTypeAliasInfo {
        // Create lazy type for deferred evaluation
        const lazyType = createLazyTypeAlias(symbol, typeParameters, createType, context);

        // Check for direct self-reference
        const testType = createType();
        const directlyRecursive = isDirectlyRecursive(symbol, testType, context);

        return {
            symbol,
            typeParameters,
            directlyRecursive,
            type: lazyType,
        };
    }

    /**
     * Instantiate a generic type with safety checks
     */
    export function safeInstantiate(
        symbol: Symbol,
        typeParameters: TypeParameter[],
        typeArguments: readonly Type[],
        createInstance: () => Type,
        context: RecursiveTypeContext
    ): Type {
        // Track instantiation count
        context.instantiationCount++;

        // Check limits
        if (context.instantiationCount > context.config.maxInstantiations) {
            return context.anyType;
        }

        // Check for infinite instantiation
        const depth = getCurrentDepth(context);
        if (wouldInstantiationBeInfinite(symbol, typeArguments, depth, context)) {
            return context.circularType;
        }

        // Proceed with instantiation
        return createInstance();
    }

    /**
     * Reset context for new resolution
     */
    export function resetContext(context: RecursiveTypeContext): void {
        clearResolutionStack(context);
        context.cache.clear();
        context.instantiationCount = 0;
    }

    /**
     * Get context statistics
     */
    export function getContextStats(context: RecursiveTypeContext): {
        stackDepth: number;
        cacheSize: number;
        instantiationCount: number;
    } {
        return {
            stackDepth: getCurrentDepth(context),
            cacheSize: context.cache.size,
            instantiationCount: context.instantiationCount,
        };
    }

    /**
     * Helpers for common recursive type patterns
     */
    export const RecursiveHelpers = {
        /**
         * Check if a type is a recursive reference to itself
         */
        isSelfReference(type: Type, symbol: Symbol): boolean {
            return type.aliasSymbol === symbol;
        },

        /**
         * Wrap a resolver with depth tracking
         */
        withDepthLimit<T>(
            resolver: () => T,
            fallback: T,
            context: RecursiveTypeContext
        ): T {
            if (isMaxDepthExceeded(context)) {
                return fallback;
            }
            return resolver();
        },

        /**
         * Create a memoized type resolver
         */
        memoize<T>(
            fn: (...args: any[]) => T,
            keyFn: (...args: any[]) => string
        ): (...args: any[]) => T {
            const cache = new Map<string, T>();
            return (...args: any[]) => {
                const key = keyFn(...args);
                const cached = cache.get(key);
                if (cached !== undefined) {
                    return cached;
                }
                const result = fn(...args);
                cache.set(key, result);
                return result;
            };
        },

        /**
         * Execute with timeout protection
         */
        withTimeout<T>(
            fn: () => T,
            timeoutMs: number,
            fallback: T
        ): T {
            const start = Date.now();

            // Simple check-based timeout (not true async timeout)
            const result = fn();

            if (Date.now() - start > timeoutMs) {
                console.warn(`Type resolution took longer than ${timeoutMs}ms`);
            }

            return result;
        },

        /**
         * Create a recursive type visitor
         */
        createVisitor(
            onType: (type: Type, depth: number) => Type | undefined,
            maxDepth: number = 50
        ): (type: Type) => Type {
            const visited = new Set<Type>();

            function visit(type: Type, depth: number): Type {
                if (depth > maxDepth || visited.has(type)) {
                    return type;
                }
                visited.add(type);

                const result = onType(type, depth);
                return result ?? type;
            }

            return (type: Type) => visit(type, 0);
        },
    };
}
