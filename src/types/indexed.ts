/**
 * @fileoverview Index type definitions for TypeScript type system
 *
 * This module defines types and utilities for working with:
 * - Index signatures (string, number, symbol indexers)
 * - Indexed access types (T[K])
 * - keyof operator results
 */

namespace ts {
    /**
     * Flags controlling indexed access behavior
     */
    export const enum IndexAccessFlags {
        None = 0,
        /** Include undefined in result type */
        IncludeUndefined = 1 << 0,
        /** Skip index signatures, only check properties */
        NoIndexSignatures = 1 << 1,
        /** Writing context (affects union/intersection handling) */
        Writing = 1 << 2,
        /** Cache result symbol */
        CacheSymbol = 1 << 3,
        /** Skip tuple bounds checking */
        NoTupleBoundsCheck = 1 << 4,
        /** Expression position (vs type context) */
        ExpressionPosition = 1 << 5,
        /** Report deprecated access */
        ReportDeprecated = 1 << 6,
        /** Suppress noImplicitAny errors */
        SuppressNoImplicitAnyError = 1 << 7,
        /** Contextual typing context */
        Contextual = 1 << 8,
        /** Flags persisted on IndexedAccessType */
        Persistent = IncludeUndefined,
    }

    /**
     * Supported index key types
     */
    export const enum IndexKeyType {
        String = 0,
        Number = 1,
        Symbol = 2,
    }

    /**
     * Information about a single index signature
     */
    export interface IndexSignatureInfo {
        /** The key type (string, number, or symbol type) */
        readonly keyType: Type;
        /** The value type returned by indexing */
        readonly type: Type;
        /** Whether the index signature is readonly */
        readonly isReadonly: boolean;
        /** The declaration node, if available */
        readonly declaration?: IndexSignatureDeclaration;
    }

    /**
     * Represents the result of resolving an indexed access type T[K]
     */
    export interface IndexedAccessResult {
        /** The resolved type, or undefined if resolution failed */
        readonly type: Type | undefined;
        /** Whether the access is valid */
        readonly isValid: boolean;
        /** Error message if access is invalid */
        readonly errorMessage?: string;
        /** The index info used, if any */
        readonly indexInfo?: IndexSignatureInfo;
        /** The property symbol accessed, if any */
        readonly propertySymbol?: Symbol;
    }

    /**
     * Represents a keyof T result
     */
    export interface KeyofResult {
        /** The resulting type (union of literal types) */
        readonly type: Type;
        /** Whether only string keys are included */
        readonly stringsOnly: boolean;
        /** The property names included */
        readonly propertyNames: __String[];
        /** The index key types included */
        readonly indexKeyTypes: Type[];
    }

    /**
     * Configuration for index signature checking
     */
    export interface IndexSignatureCheckConfig {
        /** Maximum number of index signatures per type */
        readonly maxIndexSignatures: number;
        /** Whether to allow implicit any from index signatures */
        readonly allowImplicitAny: boolean;
        /** Whether symbol index signatures are supported */
        readonly supportSymbolIndexSignatures: boolean;
        /** Whether to check index signature compatibility strictly */
        readonly strictIndexSignatureChecking: boolean;
    }

    /**
     * Default configuration for index signature checking
     */
    export const defaultIndexSignatureConfig: IndexSignatureCheckConfig = {
        maxIndexSignatures: 100,
        allowImplicitAny: false,
        supportSymbolIndexSignatures: true,
        strictIndexSignatureChecking: true,
    };

    /**
     * Result of index signature compatibility check
     */
    export interface IndexSignatureCompatibility {
        /** Whether signatures are compatible */
        readonly isCompatible: boolean;
        /** Detailed compatibility results for each signature */
        readonly details: IndexSignatureCompatibilityDetail[];
        /** Overall error message if incompatible */
        readonly errorMessage?: string;
    }

    /**
     * Detail for individual signature compatibility
     */
    export interface IndexSignatureCompatibilityDetail {
        /** The source index signature */
        readonly source: IndexSignatureInfo;
        /** The target index signature (if matched) */
        readonly target?: IndexSignatureInfo;
        /** Whether this specific pair is compatible */
        readonly isCompatible: boolean;
        /** Reason for incompatibility */
        readonly reason?: string;
    }

    /**
     * Cache key for indexed access type lookups
     */
    export interface IndexedAccessCacheKey {
        readonly objectTypeId: number;
        readonly indexTypeId: number;
        readonly accessFlags: IndexAccessFlags;
    }

    /**
     * Creates a cache key for indexed access lookup
     */
    export function createIndexedAccessCacheKey(
        objectType: Type,
        indexType: Type,
        accessFlags: IndexAccessFlags
    ): string {
        return `${objectType.id}:${indexType.id}:${accessFlags}`;
    }

    /**
     * Represents the internal state of index signature resolution
     */
    export interface IndexResolutionState {
        /** Object type being indexed */
        readonly objectType: Type;
        /** Index/key type */
        readonly indexType: Type;
        /** Current access flags */
        readonly accessFlags: IndexAccessFlags;
        /** Depth of nested indexed access */
        readonly depth: number;
        /** Whether we're in a writing context */
        readonly isWriting: boolean;
        /** Cache of resolved types */
        readonly cache: Map<string, Type>;
    }

    /**
     * Creates initial resolution state
     */
    export function createIndexResolutionState(
        objectType: Type,
        indexType: Type,
        accessFlags: IndexAccessFlags = IndexAccessFlags.None
    ): IndexResolutionState {
        return {
            objectType,
            indexType,
            accessFlags,
            depth: 0,
            isWriting: (accessFlags & IndexAccessFlags.Writing) !== 0,
            cache: new Map(),
        };
    }

    /**
     * Check if a type can be used as a property name
     */
    export function isTypeUsableAsPropertyName(type: Type): boolean {
        // String, number, and unique symbol literals can be property names
        return !!(
            (type.flags & TypeFlags.StringLiteral) ||
            (type.flags & TypeFlags.NumberLiteral) ||
            (type.flags & TypeFlags.UniqueESSymbol)
        );
    }

    /**
     * Get the property name from a literal type
     */
    export function getPropertyNameFromLiteralType(type: Type): __String | undefined {
        if (type.flags & TypeFlags.StringLiteral) {
            return escapeLeadingUnderscores((type as StringLiteralType).value);
        }
        if (type.flags & TypeFlags.NumberLiteral) {
            return escapeLeadingUnderscores(String((type as NumberLiteralType).value));
        }
        if (type.flags & TypeFlags.UniqueESSymbol) {
            return (type as UniqueESSymbolType).escapedName;
        }
        return undefined;
    }

    /**
     * Check if a type is a valid index key type
     */
    export function isValidIndexKeyType(type: Type): boolean {
        return !!(
            (type.flags & TypeFlags.String) ||
            (type.flags & TypeFlags.Number) ||
            (type.flags & TypeFlags.ESSymbol) ||
            (type.flags & TypeFlags.StringLiteral) ||
            (type.flags & TypeFlags.NumberLiteral) ||
            (type.flags & TypeFlags.UniqueESSymbol) ||
            (type.flags & TypeFlags.TemplateLiteral)
        );
    }

    /**
     * Get the index key type category
     */
    export function getIndexKeyTypeCategory(type: Type): IndexKeyType | undefined {
        if (type.flags & (TypeFlags.String | TypeFlags.StringLiteral | TypeFlags.TemplateLiteral)) {
            return IndexKeyType.String;
        }
        if (type.flags & (TypeFlags.Number | TypeFlags.NumberLiteral)) {
            return IndexKeyType.Number;
        }
        if (type.flags & (TypeFlags.ESSymbol | TypeFlags.UniqueESSymbol)) {
            return IndexKeyType.Symbol;
        }
        return undefined;
    }

    /**
     * Check if a key type is applicable to an index signature
     */
    export function isKeyTypeApplicableToIndexSignature(
        keyType: Type,
        indexSignatureKeyType: Type
    ): boolean {
        // Exact match
        if (keyType === indexSignatureKeyType) {
            return true;
        }

        const keyCategory = getIndexKeyTypeCategory(keyType);
        const sigCategory = getIndexKeyTypeCategory(indexSignatureKeyType);

        if (keyCategory === undefined || sigCategory === undefined) {
            return false;
        }

        // String signatures apply to both string and number keys (JavaScript coercion)
        if (sigCategory === IndexKeyType.String) {
            return keyCategory === IndexKeyType.String || keyCategory === IndexKeyType.Number;
        }

        // Number signatures only apply to number keys
        if (sigCategory === IndexKeyType.Number) {
            return keyCategory === IndexKeyType.Number;
        }

        // Symbol signatures only apply to symbol keys
        if (sigCategory === IndexKeyType.Symbol) {
            return keyCategory === IndexKeyType.Symbol;
        }

        return false;
    }

    /**
     * Represents an indexed access type T[K] that needs deferred resolution
     */
    export interface DeferredIndexedAccessType {
        /** The object type T */
        readonly objectType: Type;
        /** The index type K */
        readonly indexType: Type;
        /** Access flags */
        readonly accessFlags: IndexAccessFlags;
        /** Cached constraint */
        constraint?: Type;
        /** Simplified form for reading */
        simplifiedForReading?: Type;
        /** Simplified form for writing */
        simplifiedForWriting?: Type;
        /** Alias symbol if type has alias */
        aliasSymbol?: Symbol;
        /** Alias type arguments if type has alias */
        aliasTypeArguments?: readonly Type[];
    }

    /**
     * Checks if a type is an IndexedAccessType
     */
    export function isIndexedAccessType(type: Type): boolean {
        return !!(type.flags & TypeFlags.IndexedAccess);
    }

    /**
     * Checks if a type is an IndexType (keyof T)
     */
    export function isIndexType(type: Type): boolean {
        return !!(type.flags & TypeFlags.Index);
    }

    /**
     * Represents a mapped type index access (mapped type template with keyof)
     */
    export interface MappedTypeIndexAccess {
        /** The mapped type */
        readonly mappedType: Type;
        /** The type parameter of the mapped type */
        readonly typeParameter: Type;
        /** The constraint (keyof T) */
        readonly constraint: Type;
        /** The template type */
        readonly templateType: Type;
    }

    /**
     * Work item for iterative indexed access resolution
     */
    export interface IndexedAccessWorkItem {
        /** Type of work to perform */
        readonly kind: IndexedAccessWorkKind;
        /** Object type being accessed */
        readonly objectType: Type;
        /** Index/key type */
        readonly indexType: Type;
        /** Access flags */
        readonly accessFlags: IndexAccessFlags;
        /** Parent work item (for result propagation) */
        readonly parent?: IndexedAccessWorkItem;
        /** Partial results collected so far */
        results?: Type[];
    }

    /**
     * Types of indexed access work
     */
    export const enum IndexedAccessWorkKind {
        /** Direct property access */
        PropertyAccess,
        /** Index signature lookup */
        IndexSignature,
        /** Union distribution */
        UnionDistribute,
        /** Intersection combine */
        IntersectionCombine,
        /** Tuple element access */
        TupleElement,
        /** Mapped type access */
        MappedType,
        /** Conditional type access */
        ConditionalType,
        /** Generic deferred access */
        DeferredAccess,
    }

    /**
     * Result of a single work item execution
     */
    export interface IndexedAccessWorkResult {
        /** Whether work completed */
        readonly completed: boolean;
        /** Resulting type if completed */
        readonly type?: Type;
        /** New work items to process */
        readonly newWork?: IndexedAccessWorkItem[];
        /** Error if failed */
        readonly error?: string;
    }
}
