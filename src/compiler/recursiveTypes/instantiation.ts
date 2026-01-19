/**
 * Recursive Types - Instantiation Tracking
 *
 * This module handles detection of infinite type instantiation patterns
 * and implements depth limits for type expansion.
 */

/* @internal */
namespace ts.recursiveTypes {
    /**
     * Create an instantiation tracker
     */
    export function createInstantiationTracker(
        targetSymbol: Symbol,
        typeArguments: readonly Type[],
        depth: number
    ): InstantiationTracker {
        const key = createInstantiationKey(targetSymbol, typeArguments);
        return {
            targetSymbol,
            typeArguments,
            key,
            depth,
        };
    }

    /**
     * Create a unique key for an instantiation
     */
    function createInstantiationKey(
        symbol: Symbol,
        typeArguments: readonly Type[]
    ): string {
        const symbolId = (symbol as any).id ?? symbol.name ?? "unknown";
        if (typeArguments.length === 0) {
            return `inst:${symbolId}`;
        }

        const argKeys = typeArguments.map(t => getTypeKey(t)).join(",");
        return `inst:${symbolId}<${argKeys}>`;
    }

    /**
     * Get a key for a type (for comparison)
     */
    function getTypeKey(type: Type): string {
        // Use type ID if available
        const id = (type as any).id;
        if (id !== undefined) {
            return String(id);
        }

        // Fallback to flags-based key
        if (type.flags & TypeFlags.StringLiteral) {
            return `str:"${(type as StringLiteralType).value}"`;
        }
        if (type.flags & TypeFlags.NumberLiteral) {
            return `num:${(type as NumberLiteralType).value}`;
        }
        if (type.flags & TypeFlags.TypeParameter) {
            const tp = type as TypeParameter;
            return `tp:${tp.symbol?.name ?? "?"}`;
        }

        return `t:${type.flags}`;
    }

    /**
     * Track instantiations to detect infinite patterns
     */
    export class InstantiationManager {
        private instantiations: Map<string, InstantiationTracker[]> = new Map();
        private totalCount: number = 0;
        private maxDepthSeen: number = 0;

        /**
         * Record a new instantiation
         */
        recordInstantiation(
            symbol: Symbol,
            typeArguments: readonly Type[],
            depth: number
        ): InfiniteInstantiationResult {
            const tracker = createInstantiationTracker(symbol, typeArguments, depth);

            // Update stats
            this.totalCount++;
            this.maxDepthSeen = Math.max(this.maxDepthSeen, depth);

            // Get or create the list for this symbol
            const symbolKey = String((symbol as any).id ?? symbol.name ?? "unknown");
            let list = this.instantiations.get(symbolKey);
            if (!list) {
                list = [];
                this.instantiations.set(symbolKey, list);
            }

            // Check for infinite pattern
            const infiniteCheck = this.checkForInfinitePattern(list, tracker);
            if (infiniteCheck.wouldBeInfinite) {
                return infiniteCheck;
            }

            // Record the instantiation
            list.push(tracker);

            return { wouldBeInfinite: false };
        }

        /**
         * Check if adding a new instantiation would create an infinite pattern
         */
        private checkForInfinitePattern(
            existing: InstantiationTracker[],
            newTracker: InstantiationTracker
        ): InfiniteInstantiationResult {
            // Check for exact duplicate at increasing depth
            for (const prev of existing) {
                if (prev.key === newTracker.key && newTracker.depth > prev.depth) {
                    // Same instantiation at deeper level - likely infinite
                    return {
                        wouldBeInfinite: true,
                        pattern: `Repeated instantiation: ${newTracker.key}`,
                        detectedAtDepth: newTracker.depth,
                    };
                }
            }

            // Check for growing type arguments pattern
            // e.g., T<A>, T<T<A>>, T<T<T<A>>> ...
            if (existing.length >= 3) {
                const recent = existing.slice(-3);
                const growthPattern = this.detectGrowthPattern(recent, newTracker);
                if (growthPattern) {
                    return {
                        wouldBeInfinite: true,
                        pattern: growthPattern,
                        detectedAtDepth: newTracker.depth,
                    };
                }
            }

            return { wouldBeInfinite: false };
        }

        /**
         * Detect if type arguments are growing infinitely
         */
        private detectGrowthPattern(
            recent: InstantiationTracker[],
            newTracker: InstantiationTracker
        ): string | undefined {
            // Check if depth is consistently increasing
            const depths = recent.map(t => t.depth);
            depths.push(newTracker.depth);

            const isIncreasing = depths.every((d, i) =>
                i === 0 || d > depths[i - 1]
            );

            if (!isIncreasing) {
                return undefined;
            }

            // Check if keys are getting longer (sign of nesting)
            const keyLengths = recent.map(t => t.key.length);
            keyLengths.push(newTracker.key.length);

            const keysGrowing = keyLengths.every((len, i) =>
                i === 0 || len >= keyLengths[i - 1]
            );

            if (keysGrowing && keyLengths[keyLengths.length - 1] > keyLengths[0] * 2) {
                return `Type arguments growing exponentially: ${newTracker.key}`;
            }

            return undefined;
        }

        /**
         * Check if we've exceeded the instantiation limit
         */
        isLimitExceeded(limit: number): boolean {
            return this.totalCount > limit;
        }

        /**
         * Get statistics
         */
        getStats(): { total: number; maxDepth: number; uniqueSymbols: number } {
            return {
                total: this.totalCount,
                maxDepth: this.maxDepthSeen,
                uniqueSymbols: this.instantiations.size,
            };
        }

        /**
         * Clear all tracked instantiations
         */
        clear(): void {
            this.instantiations.clear();
            this.totalCount = 0;
            this.maxDepthSeen = 0;
        }
    }

    /**
     * Check if a generic type instantiation would be infinite
     */
    export function wouldInstantiationBeInfinite(
        symbol: Symbol,
        typeArguments: readonly Type[],
        depth: number,
        context: RecursiveTypeContext
    ): boolean {
        // Quick checks
        if (depth > context.config.maxDepth) {
            return true;
        }

        if (context.instantiationCount > context.config.maxInstantiations) {
            return true;
        }

        // Check for self-referential type arguments
        for (const arg of typeArguments) {
            if (containsSymbolReference(arg, symbol)) {
                // Type argument references the type being instantiated
                // Check if it's nested (would cause infinite expansion)
                if (isNestedReference(arg, symbol, 0)) {
                    return true;
                }
            }
        }

        return false;
    }

    /**
     * Check if a type contains a reference to a symbol
     */
    function containsSymbolReference(type: Type, symbol: Symbol): boolean {
        if (type.aliasSymbol === symbol) {
            return true;
        }

        if (type.flags & TypeFlags.Object) {
            const objectType = type as ObjectType;
            if (objectType.objectFlags & ObjectFlags.Reference) {
                const typeRef = type as TypeReference;
                if ((typeRef.target as any)?.symbol === symbol) {
                    return true;
                }
                if (typeRef.typeArguments) {
                    return typeRef.typeArguments.some(t =>
                        containsSymbolReference(t, symbol)
                    );
                }
            }
        }

        if (type.flags & TypeFlags.UnionOrIntersection) {
            const types = (type as UnionOrIntersectionType).types;
            return types.some(t => containsSymbolReference(t, symbol));
        }

        return false;
    }

    /**
     * Check if a reference is nested (would cause infinite expansion)
     */
    function isNestedReference(
        type: Type,
        symbol: Symbol,
        depth: number
    ): boolean {
        if (depth > 5) {
            return true; // Assume nested at this depth
        }

        if (type.flags & TypeFlags.Object) {
            const objectType = type as ObjectType;
            if (objectType.objectFlags & ObjectFlags.Reference) {
                const typeRef = type as TypeReference;
                if ((typeRef.target as any)?.symbol === symbol) {
                    // Found reference - check if it has type args
                    if (typeRef.typeArguments && typeRef.typeArguments.length > 0) {
                        // Check if args also contain the symbol
                        for (const arg of typeRef.typeArguments) {
                            if (containsSymbolReference(arg, symbol)) {
                                return true; // Nested!
                            }
                        }
                    }
                }

                // Check type arguments recursively
                if (typeRef.typeArguments) {
                    return typeRef.typeArguments.some(t =>
                        isNestedReference(t, symbol, depth + 1)
                    );
                }
            }
        }

        if (type.flags & TypeFlags.UnionOrIntersection) {
            const types = (type as UnionOrIntersectionType).types;
            return types.some(t => isNestedReference(t, symbol, depth + 1));
        }

        return false;
    }

    /**
     * Safe type expansion with depth limit
     */
    export function expandTypeWithDepthLimit(
        type: Type,
        expander: (t: Type, depth: number) => Type,
        maxDepth: number,
        currentDepth: number = 0
    ): Type {
        if (currentDepth >= maxDepth) {
            return type; // Stop expansion at limit
        }

        return expander(type, currentDepth);
    }

    /**
     * Memoized type expansion to avoid re-computation
     */
    export function createMemoizedExpander(
        expander: (type: Type) => Type
    ): (type: Type) => Type {
        const cache = new Map<Type, Type>();

        return (type: Type) => {
            const cached = cache.get(type);
            if (cached) {
                return cached;
            }

            const result = expander(type);
            cache.set(type, result);
            return result;
        };
    }
}
