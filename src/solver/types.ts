/**
 * Type Solver Types
 *
 * Core type definitions for the type solver module.
 * Uses lock-free patterns to avoid RwLock deadlocks during type resolution.
 */

/// <reference path="../compiler/types.ts" />

namespace ts.solver {
    /**
     * Unique identifier for cached types
     */
    export type TypeCacheId = number;

    /**
     * Status of type resolution - used to detect and prevent cycles
     */
    export const enum ResolutionStatus {
        /** Type has not started resolving */
        Pending = 0,
        /** Type is currently being resolved (in-progress) */
        Resolving = 1,
        /** Type resolution completed successfully */
        Resolved = 2,
        /** Type resolution failed or hit a cycle */
        Failed = 3,
    }

    /**
     * Entry in the type cache with resolution status
     * Uses a lock-free state machine pattern to prevent deadlocks
     */
    export interface TypeCacheEntry {
        /** Unique identifier for this cache entry */
        readonly id: TypeCacheId;
        /** Current resolution status - updated atomically */
        status: ResolutionStatus;
        /** The resolved type, if available */
        resolvedType?: Type;
        /** Error message if resolution failed */
        error?: string;
        /** Timestamp for cache invalidation */
        readonly timestamp: number;
    }

    /**
     * Resolution context passed during type evaluation
     * Tracks the resolution stack to detect cycles without locks
     */
    export interface ResolutionContext {
        /** Stack of type IDs currently being resolved - for cycle detection */
        readonly resolutionStack: Set<TypeCacheId>;
        /** Maximum resolution depth to prevent stack overflow */
        readonly maxDepth: number;
        /** Current resolution depth */
        depth: number;
        /** Flag indicating if we're in a speculative check */
        isSpeculative: boolean;
    }

    /**
     * Result of a type lookup operation
     * Lock-free pattern: returns a snapshot rather than holding a lock
     */
    export interface TypeLookupResult {
        /** Whether the lookup was successful */
        readonly found: boolean;
        /** The cached type entry, if found */
        readonly entry?: TypeCacheEntry;
        /** Whether a cycle was detected */
        readonly cycleDetected: boolean;
    }

    /**
     * Type instantiation request
     */
    export interface InstantiationRequest {
        /** The generic type to instantiate */
        readonly genericType: Type;
        /** Type arguments for instantiation */
        readonly typeArguments: readonly Type[];
        /** Cache key for the instantiation */
        readonly cacheKey: string;
    }

    /**
     * Result of type instantiation
     */
    export interface InstantiationResult {
        /** Whether instantiation succeeded */
        readonly success: boolean;
        /** The instantiated type, if successful */
        readonly type?: Type;
        /** Error message if instantiation failed */
        readonly error?: string;
    }

    /**
     * Configuration for the type evaluator
     */
    export interface TypeEvaluatorConfig {
        /** Maximum cache size before eviction */
        readonly maxCacheSize: number;
        /** Maximum resolution depth */
        readonly maxResolutionDepth: number;
        /** Enable strict null checks */
        readonly strictNullChecks: boolean;
        /** Cache timeout in milliseconds */
        readonly cacheTimeoutMs: number;
    }

    /**
     * Default configuration values
     */
    export const DEFAULT_EVALUATOR_CONFIG: TypeEvaluatorConfig = {
        maxCacheSize: 10000,
        maxResolutionDepth: 100,
        strictNullChecks: true,
        cacheTimeoutMs: 60000,
    };

    /**
     * Type resolution callback - allows pluggable resolution strategies
     */
    export type TypeResolver = (
        node: TypeNode | undefined,
        context: ResolutionContext
    ) => Type | undefined;

    /**
     * Type instantiation callback
     */
    export type TypeInstantiator = (
        request: InstantiationRequest,
        context: ResolutionContext
    ) => InstantiationResult;

    /**
     * Creates a new resolution context
     */
    export function createResolutionContext(maxDepth: number = 100): ResolutionContext {
        return {
            resolutionStack: new Set(),
            maxDepth,
            depth: 0,
            isSpeculative: false,
        };
    }

    /**
     * Creates a new type cache entry
     */
    export function createTypeCacheEntry(id: TypeCacheId): TypeCacheEntry {
        return {
            id,
            status: ResolutionStatus.Pending,
            resolvedType: undefined,
            error: undefined,
            timestamp: Date.now(),
        };
    }

    /**
     * Checks if we can proceed with resolution (no cycle, not too deep)
     */
    export function canResolve(id: TypeCacheId, context: ResolutionContext): boolean {
        // Check for cycles - this is the key lock-free pattern
        if (context.resolutionStack.has(id)) {
            return false;
        }
        // Check depth limit
        if (context.depth >= context.maxDepth) {
            return false;
        }
        return true;
    }

    /**
     * Enters a resolution scope - adds to stack and increments depth
     */
    export function enterResolution(id: TypeCacheId, context: ResolutionContext): void {
        context.resolutionStack.add(id);
        context.depth++;
    }

    /**
     * Exits a resolution scope - removes from stack and decrements depth
     */
    export function exitResolution(id: TypeCacheId, context: ResolutionContext): void {
        context.resolutionStack.delete(id);
        context.depth--;
    }
}
