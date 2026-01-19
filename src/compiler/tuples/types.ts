/**
 * Tuple Types - Type Definitions
 *
 * This module defines types for tuple type operations,
 * including labeled elements, optional elements, rest elements,
 * and variadic tuple types.
 */

/* @internal */
namespace ts.tuples {
    /**
     * Flags for tuple element modifiers
     */
    export const enum TupleElementFlags {
        None = 0,
        /** Element is required (default) */
        Required = 1 << 0,
        /** Element is optional (has ?) */
        Optional = 1 << 1,
        /** Element is a rest element (...) */
        Rest = 1 << 2,
        /** Element is variadic (spread of tuple) */
        Variadic = 1 << 3,
    }

    /**
     * Represents a single element in a tuple type
     */
    export interface TupleElement {
        /** The type of this element */
        type: Type;
        /** Optional label name */
        label?: string;
        /** Element flags */
        flags: TupleElementFlags;
    }

    /**
     * Represents a complete tuple type structure
     */
    export interface TupleTypeInfo {
        /** All elements in the tuple */
        elements: TupleElement[];
        /** Whether the tuple is readonly */
        readonly: boolean;
        /** The minimum length (excluding optional and rest elements) */
        minLength: number;
        /** The fixed length (undefined if has rest element) */
        fixedLength?: number;
        /** Whether the tuple has optional elements */
        hasOptional: boolean;
        /** Whether the tuple has a rest element */
        hasRest: boolean;
        /** Whether the tuple has variadic elements */
        hasVariadic: boolean;
        /** Combined element types for array-like access */
        combinedType?: Type;
    }

    /**
     * Result of spreading a tuple type
     */
    export interface SpreadResult {
        /** The resulting tuple elements */
        elements: TupleElement[];
        /** Whether the spread was successful */
        success: boolean;
        /** Error message if spread failed */
        error?: string;
    }

    /**
     * Options for tuple type operations
     */
    export interface TupleOperationOptions {
        /** Whether to preserve labels during operations */
        preserveLabels?: boolean;
        /** Whether to combine readonly modifiers */
        combineReadonly?: boolean;
        /** Maximum tuple length to expand */
        maxLength?: number;
    }

    /**
     * Context for tuple type operations
     */
    export interface TupleContext {
        /** Type checker for type operations */
        checker: TypeChecker;
        /** Number type */
        numberType: Type;
        /** String type */
        stringType: Type;
        /** Never type */
        neverType: Type;
        /** Unknown type */
        unknownType: Type;
        /** Any type */
        anyType: Type;
        /** Undefined type */
        undefinedType: Type;
    }

    /**
     * Result of tuple assignability check
     */
    export interface TupleAssignabilityResult {
        /** Whether the assignment is valid */
        assignable: boolean;
        /** Error message if not assignable */
        error?: string;
        /** Related diagnostic information */
        related?: Array<{ message: string; index?: number }>;
    }

    /**
     * Result of tuple length analysis
     */
    export interface TupleLengthInfo {
        /** Minimum possible length */
        minLength: number;
        /** Maximum possible length (undefined if unbounded) */
        maxLength?: number;
        /** Whether the length is fixed */
        isFixed: boolean;
        /** The exact length if fixed */
        exactLength?: number;
    }

    /**
     * Result of inferring tuple type from array
     */
    export interface TupleInferenceResult {
        /** The inferred tuple type */
        tupleInfo: TupleTypeInfo;
        /** Whether inference was successful */
        success: boolean;
        /** Error message if inference failed */
        error?: string;
    }

    /**
     * Mapping from element index to type for positional access
     */
    export type TuplePositionMap = Map<number, Type>;

    /**
     * Options for creating tuple types
     */
    export interface CreateTupleOptions {
        /** Element types */
        elementTypes: Type[];
        /** Optional labels for elements */
        labels?: (string | undefined)[];
        /** Flags for each element */
        elementFlags?: TupleElementFlags[];
        /** Whether the tuple is readonly */
        readonly?: boolean;
    }
}
