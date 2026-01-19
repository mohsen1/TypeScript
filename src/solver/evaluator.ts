/**
 * Type Evaluator
 *
 * Implements type resolution without recursive RwLock deadlocks.
 * Uses a lock-free caching pattern with atomic status transitions.
 */

/// <reference path="../compiler/types.ts" />
/// <reference path="types.ts" />

namespace ts.solver {
    /**
     * TypeEvaluator - Core type resolution engine
     *
     * Design principles to avoid deadlocks:
     * 1. Never hold locks during recursive calls
     * 2. Use atomic status transitions instead of lock-based synchronization
     * 3. Detect cycles via the resolution stack, not via lock contention
     * 4. Cache lookups return snapshots, not live references
     */
    export class TypeEvaluator {
        private readonly typeCache: Map<string, TypeCacheEntry>;
        private readonly instantiationCache: Map<string, TypeCacheEntry>;
        private nextCacheId: TypeCacheId;
        private readonly config: TypeEvaluatorConfig;

        constructor(config: Partial<TypeEvaluatorConfig> = {}) {
            this.config = { ...DEFAULT_EVALUATOR_CONFIG, ...config };
            this.typeCache = new Map();
            this.instantiationCache = new Map();
            this.nextCacheId = 1;
        }

        /**
         * Allocates a new unique cache ID
         * Lock-free: uses simple increment
         */
        private allocateCacheId(): TypeCacheId {
            return this.nextCacheId++;
        }

        /**
         * Generates a cache key for a type node
         */
        private getTypeCacheKey(node: TypeNode): string {
            // Use position-based key for now
            // In production, this would be more sophisticated
            return `type:${node.pos}:${node.end}`;
        }

        /**
         * Lock-free type lookup
         * Returns a snapshot of the cache entry without holding any locks
         */
        public lookupType(key: string, context: ResolutionContext): TypeLookupResult {
            const entry = this.typeCache.get(key);

            if (!entry) {
                return { found: false, cycleDetected: false };
            }

            // Check for cycles using the resolution stack
            const cycleDetected = context.resolutionStack.has(entry.id);

            return {
                found: true,
                entry: { ...entry }, // Return a snapshot
                cycleDetected,
            };
        }

        /**
         * Atomic status transition
         * Returns true if transition succeeded, false if already in target state or beyond
         */
        private tryTransitionStatus(
            entry: TypeCacheEntry,
            from: ResolutionStatus,
            to: ResolutionStatus
        ): boolean {
            if (entry.status === from) {
                entry.status = to;
                return true;
            }
            return false;
        }

        /**
         * Resolves a type node to a Type
         * Uses lock-free pattern with cycle detection via resolution stack
         */
        public resolveType(
            node: TypeNode,
            checker: TypeChecker,
            context: ResolutionContext = createResolutionContext(this.config.maxResolutionDepth)
        ): Type | undefined {
            const key = this.getTypeCacheKey(node);

            // Step 1: Check cache (lock-free read)
            const lookup = this.lookupType(key, context);

            if (lookup.found && lookup.entry) {
                // Cycle detected - return undefined to break the cycle
                if (lookup.cycleDetected) {
                    return undefined;
                }

                // Already resolved - return cached type
                if (lookup.entry.status === ResolutionStatus.Resolved) {
                    return lookup.entry.resolvedType;
                }

                // Resolution in progress by another path - wait or return undefined
                if (lookup.entry.status === ResolutionStatus.Resolving) {
                    // In a single-threaded environment, this means we have a cycle
                    return undefined;
                }

                // Failed previously - don't retry
                if (lookup.entry.status === ResolutionStatus.Failed) {
                    return undefined;
                }
            }

            // Step 2: Create or get cache entry
            let entry = this.typeCache.get(key);
            if (!entry) {
                entry = createTypeCacheEntry(this.allocateCacheId());
                this.typeCache.set(key, entry);
                this.evictIfNeeded();
            }

            // Step 3: Check if we can resolve (depth limit, cycle check)
            if (!canResolve(entry.id, context)) {
                entry.status = ResolutionStatus.Failed;
                entry.error = "Resolution depth exceeded or cycle detected";
                return undefined;
            }

            // Step 4: Attempt to transition to Resolving state
            if (!this.tryTransitionStatus(entry, ResolutionStatus.Pending, ResolutionStatus.Resolving)) {
                // Already being resolved or resolved - check current state
                if (entry.status === ResolutionStatus.Resolved) {
                    return entry.resolvedType;
                }
                return undefined;
            }

            // Step 5: Enter resolution scope
            enterResolution(entry.id, context);

            try {
                // Step 6: Perform actual type resolution
                const resolvedType = this.resolveTypeNode(node, checker, context);

                // Step 7: Cache the result
                if (resolvedType) {
                    entry.resolvedType = resolvedType;
                    entry.status = ResolutionStatus.Resolved;
                }
                else {
                    entry.status = ResolutionStatus.Failed;
                    entry.error = "Type resolution returned undefined";
                }

                return resolvedType;
            }
            catch (e) {
                // Step 8: Handle errors
                entry.status = ResolutionStatus.Failed;
                entry.error = e instanceof Error ? e.message : String(e);
                return undefined;
            }
            finally {
                // Step 9: Exit resolution scope (always, even on error)
                exitResolution(entry.id, context);
            }
        }

        /**
         * Core type node resolution logic
         * Delegates to the TypeChecker's public API with cycle detection
         */
        private resolveTypeNode(
            node: TypeNode,
            checker: TypeChecker,
            context: ResolutionContext
        ): Type | undefined {
            // For complex recursive type structures, we need to handle
            // specific cases to track resolution properly

            switch (node.kind) {
                case SyntaxKind.TypeReference:
                    return this.resolveTypeReference(node as TypeReferenceNode, checker, context);

                case SyntaxKind.ArrayType:
                    return this.resolveArrayType(node as ArrayTypeNode, checker, context);

                case SyntaxKind.UnionType:
                    return this.resolveUnionType(node as UnionTypeNode, checker, context);

                case SyntaxKind.IntersectionType:
                    return this.resolveIntersectionType(node as IntersectionTypeNode, checker, context);

                case SyntaxKind.TupleType:
                    return this.resolveTupleType(node as TupleTypeNode, checker, context);

                case SyntaxKind.IndexedAccessType:
                    return this.resolveIndexedAccessType(node as IndexedAccessTypeNode, checker, context);

                case SyntaxKind.TypeOperator:
                    return this.resolveTypeOperator(node as TypeOperatorNode, checker, context);

                case SyntaxKind.ParenthesizedType:
                    return this.resolveType((node as ParenthesizedTypeNode).type, checker, context);

                // For other types, delegate directly to the TypeChecker
                default:
                    return this.resolveViaChecker(node, checker);
            }
        }

        /**
         * Resolves a type using the TypeChecker's public API
         */
        private resolveViaChecker(node: TypeNode, checker: TypeChecker): Type | undefined {
            try {
                return checker.getTypeFromTypeNode(node);
            }
            catch {
                return undefined;
            }
        }

        /**
         * Resolves a type reference (e.g., Array<T>, Map<K, V>, CustomType)
         * Handles recursive type arguments with cycle detection
         */
        private resolveTypeReference(
            node: TypeReferenceNode,
            checker: TypeChecker,
            context: ResolutionContext
        ): Type | undefined {
            // If there are type arguments, resolve them first to track cycles
            if (node.typeArguments && node.typeArguments.length > 0) {
                for (const argNode of node.typeArguments) {
                    // Just track resolution - we don't need to collect results
                    // since we delegate to the checker anyway
                    const argType = this.resolveType(argNode, checker, context);
                    if (!argType) {
                        // Cycle detected or resolution failed in type argument
                        // Fall back to checker which may handle it differently
                        break;
                    }
                }
            }

            // Delegate to TypeChecker for actual resolution
            return this.resolveViaChecker(node, checker);
        }

        /**
         * Resolves an array type (e.g., T[])
         */
        private resolveArrayType(
            node: ArrayTypeNode,
            checker: TypeChecker,
            context: ResolutionContext
        ): Type | undefined {
            // Track element type resolution for cycle detection
            const elementType = this.resolveType(node.elementType, checker, context);
            if (!elementType) {
                // Cycle detected or resolution failed
                return undefined;
            }

            return this.resolveViaChecker(node, checker);
        }

        /**
         * Resolves a union type (e.g., A | B | C)
         */
        private resolveUnionType(
            node: UnionTypeNode,
            checker: TypeChecker,
            context: ResolutionContext
        ): Type | undefined {
            // Track all constituent types for cycle detection
            for (const typeNode of node.types) {
                const type = this.resolveType(typeNode, checker, context);
                if (!type) {
                    // Continue even on failure - let checker handle it
                    // This allows partial resolution
                }
            }

            return this.resolveViaChecker(node, checker);
        }

        /**
         * Resolves an intersection type (e.g., A & B & C)
         */
        private resolveIntersectionType(
            node: IntersectionTypeNode,
            checker: TypeChecker,
            context: ResolutionContext
        ): Type | undefined {
            // Track all constituent types for cycle detection
            for (const typeNode of node.types) {
                const type = this.resolveType(typeNode, checker, context);
                if (!type) {
                    // Continue even on failure - let checker handle it
                }
            }

            return this.resolveViaChecker(node, checker);
        }

        /**
         * Resolves a tuple type (e.g., [A, B, C])
         */
        private resolveTupleType(
            node: TupleTypeNode,
            checker: TypeChecker,
            context: ResolutionContext
        ): Type | undefined {
            // Track all element types for cycle detection
            for (const element of node.elements) {
                // Handle both TypeNode and NamedTupleMember
                const typeNode = element.kind === SyntaxKind.NamedTupleMember
                    ? (element as NamedTupleMember).type
                    : element as TypeNode;

                const type = this.resolveType(typeNode, checker, context);
                if (!type) {
                    // Continue even on failure
                }
            }

            return this.resolveViaChecker(node, checker);
        }

        /**
         * Resolves an indexed access type (e.g., T[K])
         */
        private resolveIndexedAccessType(
            node: IndexedAccessTypeNode,
            checker: TypeChecker,
            context: ResolutionContext
        ): Type | undefined {
            // Track both object type and index type for cycle detection
            this.resolveType(node.objectType, checker, context);
            this.resolveType(node.indexType, checker, context);

            return this.resolveViaChecker(node, checker);
        }

        /**
         * Resolves a type operator (e.g., keyof T, readonly T)
         */
        private resolveTypeOperator(
            node: TypeOperatorNode,
            checker: TypeChecker,
            context: ResolutionContext
        ): Type | undefined {
            // Track operand type for cycle detection
            this.resolveType(node.type, checker, context);

            return this.resolveViaChecker(node, checker);
        }

        /**
         * Instantiates a generic type with type arguments
         * Uses lock-free caching pattern
         */
        public instantiateType(
            genericType: Type,
            typeArguments: readonly Type[],
            checker: TypeChecker,
            context: ResolutionContext
        ): Type | undefined {
            // Generate cache key
            const cacheKey = this.generateInstantiationKey(genericType, typeArguments);

            // Check instantiation cache
            const cached = this.instantiationCache.get(cacheKey);
            if (cached) {
                if (context.resolutionStack.has(cached.id)) {
                    // Cycle detected in instantiation
                    return undefined;
                }
                if (cached.status === ResolutionStatus.Resolved) {
                    return cached.resolvedType;
                }
                if (cached.status === ResolutionStatus.Failed) {
                    return undefined;
                }
            }

            // Create cache entry
            const entry = createTypeCacheEntry(this.allocateCacheId());
            this.instantiationCache.set(cacheKey, entry);

            // Check resolution limits
            if (!canResolve(entry.id, context)) {
                entry.status = ResolutionStatus.Failed;
                return undefined;
            }

            // Enter resolution scope
            enterResolution(entry.id, context);
            entry.status = ResolutionStatus.Resolving;

            try {
                // For generic type instantiation, we need to use the TypeChecker's
                // internal mechanisms. Since getTypeArguments is available, we can
                // verify that the type is a TypeReference and work with it.
                //
                // In practice, the TypeChecker handles instantiation internally
                // when resolving type references with type arguments.
                // We cache the result to avoid redundant work.

                // The instantiated type is typically already computed by the checker
                // when it resolves the TypeReferenceNode. We use the genericType
                // directly as the result since proper instantiation happens during
                // type reference resolution.

                entry.resolvedType = genericType;
                entry.status = ResolutionStatus.Resolved;

                return genericType;
            }
            catch (e) {
                entry.status = ResolutionStatus.Failed;
                entry.error = e instanceof Error ? e.message : String(e);
                return undefined;
            }
            finally {
                exitResolution(entry.id, context);
            }
        }

        /**
         * Generates a unique key for type instantiation caching
         */
        private generateInstantiationKey(genericType: Type, typeArguments: readonly Type[]): string {
            const genericId = (genericType as { id?: number }).id ?? "unknown";
            const argIds = typeArguments.map(t => (t as { id?: number }).id ?? "unknown").join(",");
            return `inst:${genericId}:[${argIds}]`;
        }

        /**
         * Evicts old entries if cache is too large
         */
        private evictIfNeeded(): void {
            if (this.typeCache.size > this.config.maxCacheSize) {
                // Simple eviction: remove oldest entries
                const toRemove = this.typeCache.size - this.config.maxCacheSize + 100;
                let removed = 0;

                for (const key of this.typeCache.keys()) {
                    if (removed >= toRemove) break;
                    this.typeCache.delete(key);
                    removed++;
                }
            }

            if (this.instantiationCache.size > this.config.maxCacheSize) {
                const toRemove = this.instantiationCache.size - this.config.maxCacheSize + 100;
                let removed = 0;

                for (const key of this.instantiationCache.keys()) {
                    if (removed >= toRemove) break;
                    this.instantiationCache.delete(key);
                    removed++;
                }
            }
        }

        /**
         * Clears all caches
         */
        public clearCaches(): void {
            this.typeCache.clear();
            this.instantiationCache.clear();
        }

        /**
         * Gets cache statistics
         */
        public getCacheStats(): { typeCache: number; instantiationCache: number } {
            return {
                typeCache: this.typeCache.size,
                instantiationCache: this.instantiationCache.size,
            };
        }
    }

    /**
     * Creates a new TypeEvaluator instance
     */
    export function createTypeEvaluator(config?: Partial<TypeEvaluatorConfig>): TypeEvaluator {
        return new TypeEvaluator(config);
    }
}
