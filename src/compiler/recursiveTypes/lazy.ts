/**
 * Recursive Types - Lazy Type Evaluation
 *
 * This module implements lazy type evaluation to handle recursive types
 * without causing stack overflow or infinite loops.
 */

/* @internal */
namespace ts.recursiveTypes {
    /**
     * Create a lazy type wrapper
     */
    export function createLazyType(resolver: () => Type): LazyType {
        return {
            state: TypeResolutionState.Unresolved,
            resolver,
        };
    }

    /**
     * Check if a lazy type has been resolved
     */
    export function isLazyTypeResolved(lazy: LazyType): boolean {
        return lazy.state === TypeResolutionState.Resolved;
    }

    /**
     * Check if a lazy type is currently resolving (cycle detection)
     */
    export function isLazyTypeResolving(lazy: LazyType): boolean {
        return lazy.state === TypeResolutionState.Resolving;
    }

    /**
     * Check if a lazy type has an error
     */
    export function isLazyTypeError(lazy: LazyType): boolean {
        return lazy.state === TypeResolutionState.Error;
    }

    /**
     * Force evaluation of a lazy type
     */
    export function forceLazyType(
        lazy: LazyType,
        context: RecursiveTypeContext
    ): Type {
        // Already resolved
        if (lazy.state === TypeResolutionState.Resolved && lazy.resolvedType) {
            return lazy.resolvedType;
        }

        // Currently resolving - this is a cycle
        if (lazy.state === TypeResolutionState.Resolving) {
            lazy.state = TypeResolutionState.Error;
            lazy.error = "Circular reference detected during lazy evaluation";
            return context.circularType;
        }

        // Error state
        if (lazy.state === TypeResolutionState.Error) {
            return context.errorType;
        }

        // No resolver
        if (!lazy.resolver) {
            lazy.state = TypeResolutionState.Error;
            lazy.error = "No resolver provided for lazy type";
            return context.errorType;
        }

        // Start resolution
        lazy.state = TypeResolutionState.Resolving;
        lazy.depth = getCurrentDepth(context);

        try {
            const resolvedType = lazy.resolver();
            lazy.resolvedType = resolvedType;
            lazy.state = TypeResolutionState.Resolved;
            return resolvedType;
        } catch (e) {
            lazy.state = TypeResolutionState.Error;
            lazy.error = e instanceof Error ? e.message : String(e);
            return context.errorType;
        }
    }

    /**
     * Create a deferred type reference
     * The type is only evaluated when accessed
     */
    export function createDeferredType(
        resolver: () => Type,
        context: RecursiveTypeContext
    ): Type {
        const lazy = createLazyType(resolver);

        // Create a proxy-like object that forces evaluation on access
        const deferred: Type = {
            flags: TypeFlags.Object,
            get symbol() {
                const resolved = forceLazyType(lazy, context);
                return resolved.symbol;
            },
        } as Type;

        // Store the lazy reference for later forcing
        (deferred as any).__lazy = lazy;

        return deferred;
    }

    /**
     * Check if a type is a deferred type
     */
    export function isDeferredType(type: Type): boolean {
        return !!(type as any).__lazy;
    }

    /**
     * Get the lazy wrapper from a deferred type
     */
    export function getLazyWrapper(type: Type): LazyType | undefined {
        return (type as any).__lazy;
    }

    /**
     * Force all deferred types in a type to resolve
     */
    export function forceAllDeferredTypes(
        type: Type,
        context: RecursiveTypeContext,
        visited: Set<Type> = new Set()
    ): Type {
        if (visited.has(type)) {
            return type;
        }
        visited.add(type);

        // Check if this is a deferred type
        const lazy = getLazyWrapper(type);
        if (lazy) {
            return forceLazyType(lazy, context);
        }

        // Handle unions/intersections
        if (type.flags & TypeFlags.UnionOrIntersection) {
            const unionOrIntersection = type as UnionOrIntersectionType;
            for (let i = 0; i < unionOrIntersection.types.length; i++) {
                const member = unionOrIntersection.types[i];
                if (isDeferredType(member)) {
                    const lazy = getLazyWrapper(member)!;
                    (unionOrIntersection.types as Type[])[i] = forceLazyType(lazy, context);
                }
            }
        }

        // Handle type references
        if (type.flags & TypeFlags.Object) {
            const objectType = type as ObjectType;
            if (objectType.objectFlags & ObjectFlags.Reference) {
                const typeRef = type as TypeReference;
                if (typeRef.typeArguments) {
                    for (let i = 0; i < typeRef.typeArguments.length; i++) {
                        const arg = typeRef.typeArguments[i];
                        if (isDeferredType(arg)) {
                            const lazy = getLazyWrapper(arg)!;
                            (typeRef.typeArguments as Type[])[i] = forceLazyType(lazy, context);
                        }
                    }
                }
            }
        }

        return type;
    }

    /**
     * Create a lazy type alias that defers evaluation
     */
    export function createLazyTypeAlias(
        symbol: Symbol,
        typeParameters: TypeParameter[] | undefined,
        createType: () => Type,
        context: RecursiveTypeContext
    ): LazyType {
        const lazy = createLazyType(() => {
            // Check for circularity
            if (isSymbolBeingResolved(symbol, context)) {
                return context.circularType;
            }

            // Push resolution
            pushResolution(symbol, undefined, context);
            try {
                return createType();
            } finally {
                popResolution(context);
            }
        });

        return lazy;
    }

    /**
     * Resolve a type alias with lazy evaluation
     */
    export function resolveLazyTypeAlias(
        info: RecursiveTypeAliasInfo,
        typeArguments: readonly Type[] | undefined,
        context: RecursiveTypeContext
    ): Type {
        // Check cache first
        const cacheKey = createTypeAliasCacheKey(info.symbol, typeArguments);
        const cached = context.cache.get(cacheKey);
        if (cached) {
            return cached;
        }

        // Force the lazy type
        if (info.type) {
            const resolved = forceLazyType(info.type, context);

            // Apply type arguments if needed
            if (typeArguments && typeArguments.length > 0 && info.typeParameters) {
                const instantiated = instantiateTypeWithArguments(
                    resolved,
                    info.typeParameters,
                    typeArguments,
                    context
                );

                // Cache the result
                if (context.config.useCache) {
                    context.cache.set(cacheKey, instantiated);
                }

                return instantiated;
            }

            // Cache and return
            if (context.config.useCache) {
                context.cache.set(cacheKey, resolved);
            }

            return resolved;
        }

        return context.errorType;
    }

    /**
     * Create a cache key for a type alias instantiation
     */
    function createTypeAliasCacheKey(
        symbol: Symbol,
        typeArguments: readonly Type[] | undefined
    ): string {
        const symbolId = (symbol as any).id ?? 0;
        if (!typeArguments || typeArguments.length === 0) {
            return `alias:${symbolId}`;
        }

        const argIds = typeArguments.map(t => (t as any).id ?? 0).join(",");
        return `alias:${symbolId}<${argIds}>`;
    }

    /**
     * Instantiate a type with type arguments
     */
    function instantiateTypeWithArguments(
        type: Type,
        typeParameters: TypeParameter[],
        typeArguments: readonly Type[],
        context: RecursiveTypeContext
    ): Type {
        // Track instantiation count
        context.instantiationCount++;
        if (context.instantiationCount > context.config.maxInstantiations) {
            return context.errorType;
        }

        // Create a type mapper
        const mapper = createTypeMapper(typeParameters, typeArguments);

        // Instantiate the type
        return instantiateType(type, mapper, context);
    }

    /**
     * Create a type mapper function
     */
    function createTypeMapper(
        typeParameters: TypeParameter[],
        typeArguments: readonly Type[]
    ): (type: Type) => Type {
        const map = new Map<TypeParameter, Type>();
        for (let i = 0; i < typeParameters.length && i < typeArguments.length; i++) {
            map.set(typeParameters[i], typeArguments[i]);
        }

        return (type: Type) => {
            if (type.flags & TypeFlags.TypeParameter) {
                return map.get(type as TypeParameter) ?? type;
            }
            return type;
        };
    }

    /**
     * Instantiate a type using a mapper
     */
    function instantiateType(
        type: Type,
        mapper: (type: Type) => Type,
        context: RecursiveTypeContext
    ): Type {
        // Check for type parameter
        if (type.flags & TypeFlags.TypeParameter) {
            return mapper(type);
        }

        // Handle unions/intersections
        if (type.flags & TypeFlags.UnionOrIntersection) {
            const unionOrIntersection = type as UnionOrIntersectionType;
            const mappedTypes = unionOrIntersection.types.map(t =>
                instantiateType(t, mapper, context)
            );

            if (type.flags & TypeFlags.Union) {
                return context.checker.getUnionType(mappedTypes);
            } else {
                return context.checker.getIntersectionType(mappedTypes);
            }
        }

        // Handle type references
        if (type.flags & TypeFlags.Object) {
            const objectType = type as ObjectType;
            if (objectType.objectFlags & ObjectFlags.Reference) {
                const typeRef = type as TypeReference;
                if (typeRef.typeArguments) {
                    const mappedArgs = typeRef.typeArguments.map(t =>
                        instantiateType(t, mapper, context)
                    );

                    // Check if any args changed
                    const anyChanged = mappedArgs.some((arg, i) =>
                        arg !== typeRef.typeArguments![i]
                    );

                    if (anyChanged) {
                        return context.checker.createTypeReference(
                            typeRef.target,
                            mappedArgs
                        );
                    }
                }
            }
        }

        return type;
    }

    /**
     * Lazy evaluation context manager
     */
    export class LazyEvaluationContext {
        private pendingEvaluations: Map<string, LazyType> = new Map();
        private evaluationOrder: string[] = [];

        /**
         * Register a lazy type for later evaluation
         */
        register(key: string, lazy: LazyType): void {
            this.pendingEvaluations.set(key, lazy);
            this.evaluationOrder.push(key);
        }

        /**
         * Evaluate all pending lazy types in order
         */
        evaluateAll(context: RecursiveTypeContext): Map<string, Type> {
            const results = new Map<string, Type>();

            for (const key of this.evaluationOrder) {
                const lazy = this.pendingEvaluations.get(key);
                if (lazy) {
                    const type = forceLazyType(lazy, context);
                    results.set(key, type);
                }
            }

            return results;
        }

        /**
         * Clear all pending evaluations
         */
        clear(): void {
            this.pendingEvaluations.clear();
            this.evaluationOrder.length = 0;
        }

        /**
         * Get count of pending evaluations
         */
        get pendingCount(): number {
            return this.pendingEvaluations.size;
        }
    }
}
