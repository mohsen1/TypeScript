/**
 * Recursive Types - Type Resolution Cache
 *
 * This module implements caching for recursive type resolutions
 * to avoid redundant computation and prevent infinite loops.
 */

/* @internal */
namespace ts.recursiveTypes {
    /**
     * Create a cache entry
     */
    export function createCacheEntry(
        type: Type,
        isRecursive: boolean = false
    ): TypeCacheEntry {
        return {
            type,
            timestamp: Date.now(),
            accessCount: 0,
            isRecursive,
        };
    }

    /**
     * Type resolution cache with LRU eviction
     */
    export class TypeResolutionCache {
        private cache: Map<string, TypeCacheEntry> = new Map();
        private maxSize: number;
        private stats: RecursiveTypeStats;

        constructor(maxSize: number = 10000) {
            this.maxSize = maxSize;
            this.stats = {
                totalResolutions: 0,
                successfulResolutions: 0,
                cacheHits: 0,
                cyclesDetected: 0,
                maxDepthReached: 0,
                instantiationsPerformed: 0,
            };
        }

        /**
         * Get a cached type
         */
        get(key: string): Type | undefined {
            const entry = this.cache.get(key);
            if (entry) {
                entry.accessCount++;
                this.stats.cacheHits++;
                return entry.type;
            }
            return undefined;
        }

        /**
         * Set a cached type
         */
        set(key: string, type: Type, isRecursive: boolean = false): void {
            // Evict if needed
            if (this.cache.size >= this.maxSize) {
                this.evictLeastRecentlyUsed();
            }

            this.cache.set(key, createCacheEntry(type, isRecursive));
        }

        /**
         * Check if a key exists in the cache
         */
        has(key: string): boolean {
            return this.cache.has(key);
        }

        /**
         * Delete a cached entry
         */
        delete(key: string): boolean {
            return this.cache.delete(key);
        }

        /**
         * Clear the entire cache
         */
        clear(): void {
            this.cache.clear();
        }

        /**
         * Evict least recently used entries
         */
        private evictLeastRecentlyUsed(): void {
            // Find entries with lowest access count
            let minAccess = Infinity;
            let oldestTime = Infinity;
            let toEvict: string | undefined;

            for (const [key, entry] of this.cache) {
                if (entry.accessCount < minAccess ||
                    (entry.accessCount === minAccess && entry.timestamp < oldestTime)) {
                    minAccess = entry.accessCount;
                    oldestTime = entry.timestamp;
                    toEvict = key;
                }
            }

            if (toEvict) {
                this.cache.delete(toEvict);
            }
        }

        /**
         * Get cache statistics
         */
        getStats(): RecursiveTypeStats {
            return { ...this.stats };
        }

        /**
         * Record a resolution attempt
         */
        recordResolution(successful: boolean): void {
            this.stats.totalResolutions++;
            if (successful) {
                this.stats.successfulResolutions++;
            }
        }

        /**
         * Record a cycle detection
         */
        recordCycle(): void {
            this.stats.cyclesDetected++;
        }

        /**
         * Record max depth
         */
        recordDepth(depth: number): void {
            this.stats.maxDepthReached = Math.max(this.stats.maxDepthReached, depth);
        }

        /**
         * Record an instantiation
         */
        recordInstantiation(): void {
            this.stats.instantiationsPerformed++;
        }

        /**
         * Get the number of cached entries
         */
        get size(): number {
            return this.cache.size;
        }

        /**
         * Get all keys for debugging
         */
        keys(): string[] {
            return Array.from(this.cache.keys());
        }

        /**
         * Get entries marked as recursive
         */
        getRecursiveEntries(): [string, Type][] {
            const result: [string, Type][] = [];
            for (const [key, entry] of this.cache) {
                if (entry.isRecursive) {
                    result.push([key, entry.type]);
                }
            }
            return result;
        }
    }

    /**
     * Create a cache key for a type alias with type arguments
     */
    export function createTypeAliasKey(
        symbol: Symbol,
        typeArguments?: readonly Type[]
    ): string {
        const symbolId = (symbol as any).id ?? 0;
        if (!typeArguments || typeArguments.length === 0) {
            return `alias:${symbolId}`;
        }

        const argIds = typeArguments.map(t => getTypeId(t)).join(",");
        return `alias:${symbolId}<${argIds}>`;
    }

    /**
     * Create a cache key for a type reference
     */
    export function createTypeReferenceKey(
        target: Type,
        typeArguments?: readonly Type[]
    ): string {
        const targetId = getTypeId(target);
        if (!typeArguments || typeArguments.length === 0) {
            return `ref:${targetId}`;
        }

        const argIds = typeArguments.map(t => getTypeId(t)).join(",");
        return `ref:${targetId}<${argIds}>`;
    }

    /**
     * Create a cache key for a conditional type
     */
    export function createConditionalTypeKey(
        checkType: Type,
        extendsType: Type,
        trueType: Type,
        falseType: Type
    ): string {
        return `cond:${getTypeId(checkType)}:${getTypeId(extendsType)}:${getTypeId(trueType)}:${getTypeId(falseType)}`;
    }

    /**
     * Create a cache key for a mapped type
     */
    export function createMappedTypeKey(
        type: Type,
        constraintType: Type,
        templateType: Type
    ): string {
        return `mapped:${getTypeId(type)}:${getTypeId(constraintType)}:${getTypeId(templateType)}`;
    }

    /**
     * Get a unique ID for a type
     */
    function getTypeId(type: Type): number {
        return (type as any).id ?? 0;
    }

    /**
     * Global cache instance (optional usage)
     */
    let globalCache: TypeResolutionCache | undefined;

    /**
     * Get or create the global cache
     */
    export function getGlobalCache(): TypeResolutionCache {
        if (!globalCache) {
            globalCache = new TypeResolutionCache();
        }
        return globalCache;
    }

    /**
     * Reset the global cache
     */
    export function resetGlobalCache(): void {
        if (globalCache) {
            globalCache.clear();
        }
        globalCache = undefined;
    }

    /**
     * Scoped cache for a specific resolution context
     */
    export class ScopedCache {
        private localCache: Map<string, Type> = new Map();
        private parentCache?: TypeResolutionCache;

        constructor(parentCache?: TypeResolutionCache) {
            this.parentCache = parentCache;
        }

        /**
         * Get from local or parent cache
         */
        get(key: string): Type | undefined {
            // Check local first
            const local = this.localCache.get(key);
            if (local) {
                return local;
            }

            // Check parent
            return this.parentCache?.get(key);
        }

        /**
         * Set in local cache only
         */
        setLocal(key: string, type: Type): void {
            this.localCache.set(key, type);
        }

        /**
         * Promote local cache to parent
         */
        promoteToParent(): void {
            if (this.parentCache) {
                for (const [key, type] of this.localCache) {
                    this.parentCache.set(key, type);
                }
            }
        }

        /**
         * Clear local cache (on error/rollback)
         */
        clearLocal(): void {
            this.localCache.clear();
        }

        /**
         * Get local cache size
         */
        get localSize(): number {
            return this.localCache.size;
        }
    }

    /**
     * Cached resolver with memoization
     */
    export function createCachedResolver<T>(
        resolver: (key: string) => T,
        cache: Map<string, T> = new Map()
    ): (key: string) => T {
        return (key: string) => {
            const cached = cache.get(key);
            if (cached !== undefined) {
                return cached;
            }

            const result = resolver(key);
            cache.set(key, result);
            return result;
        };
    }
}
