/**
 * Recursive Types - Type Definitions
 *
 * This module defines types for handling recursive type references,
 * circular type detection, and lazy type evaluation.
 */

/* @internal */
namespace ts.recursiveTypes {
    /**
     * State of a type during resolution
     */
    export const enum TypeResolutionState {
        /** Type has not started resolving */
        Unresolved = 0,
        /** Type is currently being resolved (indicates cycle if encountered) */
        Resolving = 1,
        /** Type resolution completed successfully */
        Resolved = 2,
        /** Type resolution failed or cycle was detected */
        Error = 3,
    }

    /**
     * Represents a type that may be lazily evaluated
     */
    export interface LazyType {
        /** The resolved type, if available */
        resolvedType?: Type;
        /** Current resolution state */
        state: TypeResolutionState;
        /** The computation to resolve this type */
        resolver?: () => Type;
        /** Error message if resolution failed */
        error?: string;
        /** Depth at which this type was encountered */
        depth?: number;
    }

    /**
     * Entry in the type resolution stack
     */
    export interface ResolutionStackEntry {
        /** Symbol or identifier being resolved */
        symbol: Symbol | undefined;
        /** Node being resolved */
        node: Node | undefined;
        /** String key for identification */
        key: string;
        /** Depth in the resolution stack */
        depth: number;
    }

    /**
     * Result of circularity detection
     */
    export interface CircularityCheckResult {
        /** Whether a circularity was detected */
        isCircular: boolean;
        /** Path of the cycle if detected */
        cyclePath?: string[];
        /** Depth at which cycle was detected */
        cycleDepth?: number;
    }

    /**
     * Configuration for recursive type resolution
     */
    export interface RecursiveTypeConfig {
        /** Maximum depth for type expansion */
        maxDepth: number;
        /** Maximum number of type instantiations */
        maxInstantiations: number;
        /** Whether to use lazy evaluation */
        useLazyEvaluation: boolean;
        /** Whether to cache resolved types */
        useCache: boolean;
    }

    /**
     * Default configuration values
     */
    export const DefaultConfig: RecursiveTypeConfig = {
        maxDepth: 50,
        maxInstantiations: 100000,
        useLazyEvaluation: true,
        useCache: true,
    };

    /**
     * Context for recursive type operations
     */
    export interface RecursiveTypeContext {
        /** Type checker for type operations */
        checker: TypeChecker;
        /** Current configuration */
        config: RecursiveTypeConfig;
        /** Resolution stack for cycle detection */
        resolutionStack: ResolutionStackEntry[];
        /** Cache of resolved types */
        cache: Map<string, Type>;
        /** Number of instantiations performed */
        instantiationCount: number;
        /** Circular type marker */
        circularType: Type;
        /** Any type for fallback */
        anyType: Type;
        /** Error type marker */
        errorType: Type;
    }

    /**
     * Result of type resolution
     */
    export interface TypeResolutionResult {
        /** The resolved type */
        type: Type;
        /** Whether resolution was successful */
        success: boolean;
        /** Whether a circular reference was encountered */
        circular: boolean;
        /** Error message if resolution failed */
        error?: string;
        /** Whether result came from cache */
        fromCache: boolean;
    }

    /**
     * Information about a recursive type alias
     */
    export interface RecursiveTypeAliasInfo {
        /** The type alias symbol */
        symbol: Symbol;
        /** Type parameters if generic */
        typeParameters?: TypeParameter[];
        /** Whether the type is directly recursive */
        directlyRecursive: boolean;
        /** Types it mutually refers to */
        mutualReferences?: Symbol[];
        /** The resolved type (may be lazy) */
        type?: LazyType;
    }

    /**
     * Tracking information for type instantiation
     */
    export interface InstantiationTracker {
        /** Target symbol being instantiated */
        targetSymbol: Symbol;
        /** Type arguments used */
        typeArguments: readonly Type[];
        /** Key for deduplication */
        key: string;
        /** Depth of this instantiation */
        depth: number;
    }

    /**
     * Result of infinite instantiation check
     */
    export interface InfiniteInstantiationResult {
        /** Whether infinite instantiation would occur */
        wouldBeInfinite: boolean;
        /** Pattern detected (if any) */
        pattern?: string;
        /** Depth at which pattern was detected */
        detectedAtDepth?: number;
    }

    /**
     * Cache entry for type resolution
     */
    export interface TypeCacheEntry {
        /** The cached type */
        type: Type;
        /** When this entry was created */
        timestamp: number;
        /** How many times this was accessed */
        accessCount: number;
        /** Whether this is a recursive type */
        isRecursive: boolean;
    }

    /**
     * Statistics about recursive type handling
     */
    export interface RecursiveTypeStats {
        /** Total resolutions attempted */
        totalResolutions: number;
        /** Successful resolutions */
        successfulResolutions: number;
        /** Cache hits */
        cacheHits: number;
        /** Cycles detected */
        cyclesDetected: number;
        /** Maximum depth reached */
        maxDepthReached: number;
        /** Instantiations performed */
        instantiationsPerformed: number;
    }
}
