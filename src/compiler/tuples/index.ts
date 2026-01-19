/**
 * Tuple Types - Module Index
 *
 * Comprehensive implementation of TypeScript's tuple types:
 *
 * - Labeled tuple elements
 * - Optional tuple elements
 * - Rest elements in tuples
 * - Variadic tuple types with spreads
 * - Tuple type inference from array literals
 * - Tuple to array assignability
 * - Readonly tuple types
 * - Tuple length checking
 *
 * @example
 * ```typescript
 * // Labeled tuple
 * type Person = [name: string, age: number];
 *
 * // Optional elements
 * type OptionalTuple = [string, number?];
 *
 * // Rest elements
 * type RestTuple = [string, ...number[]];
 *
 * // Variadic tuples
 * type Concat<A extends unknown[], B extends unknown[]> = [...A, ...B];
 *
 * // Readonly tuples
 * type ReadonlyPair = readonly [string, number];
 * ```
 */

/* @internal */
namespace ts.tuples {
    // Re-export all types
    export type {
        TupleElement,
        TupleTypeInfo,
        SpreadResult,
        TupleOperationOptions,
        TupleContext,
        TupleAssignabilityResult,
        TupleLengthInfo,
        TupleInferenceResult,
        TuplePositionMap,
        CreateTupleOptions,
    };

    // Export TupleElementFlags enum
    export { TupleElementFlags };

    // Export element operations
    export {
        createRequiredElement,
        createOptionalElement,
        createRestElement,
        createVariadicElement,
        isRequiredElement,
        isOptionalElement,
        isRestElement,
        isVariadicElement,
        hasLabel,
        getElementType,
        getRestElementArrayType,
        elementToString,
        cloneElement,
        makeOptional,
        makeRequired,
        removeLabel,
        withLabel,
        elementsEqual,
        getElementLengthContribution,
        validateElementOrder,
        normalizeElements,
        getLabels,
        allLabeled,
        noneLabeled,
        hassMixedLabels,
    };

    // Export variadic operations
    export {
        spreadTupleType,
        isTupleType,
        getTupleTypeInfo,
        concatenateTuples,
        resolveVariadicElements,
        sliceTuple,
        prependToTuple,
        appendToTuple,
        createHomogeneousTuple,
        tupleToArrayType,
    };

    // Export assignability
    export {
        isTupleAssignableTo,
        isTupleAssignableToArray,
        isArrayAssignableToTuple,
        getTupleLengthType,
        analyzeTupleLength,
        isValidTupleIndex,
        getTypeAtIndex,
    };

    // Export inference
    export {
        inferTupleFromArrayLiteral,
        inferTupleWithSpreads,
        inferTupleFromRestParameter,
        createTupleFromParameters,
        inferContextualTupleType,
        widenTupleToArray,
        narrowArrayToTuple,
        inferTupleFromIndexedAccess,
        mapTupleType,
        filterTupleElements,
    };

    /**
     * Create a tuple context from a type checker
     */
    export function createTupleContext(checker: TypeChecker): TupleContext {
        return {
            checker,
            numberType: checker.getNumberType(),
            stringType: checker.getStringType(),
            neverType: checker.getNeverType(),
            unknownType: checker.getUnknownType(),
            anyType: checker.getAnyType(),
            undefinedType: checker.getUndefinedType(),
        };
    }

    /**
     * Create a tuple type from options
     */
    export function createTupleType(
        options: CreateTupleOptions,
        context: TupleContext
    ): TupleTypeInfo {
        const elements: TupleElement[] = [];
        let minLength = 0;
        let hasOptional = false;
        let hasRest = false;
        let hasVariadic = false;

        for (let i = 0; i < options.elementTypes.length; i++) {
            const type = options.elementTypes[i];
            const label = options.labels?.[i];
            const flags = options.elementFlags?.[i] ?? TupleElementFlags.Required;

            elements.push({ type, label, flags });

            if (flags & TupleElementFlags.Rest) {
                hasRest = true;
            } else if (flags & TupleElementFlags.Variadic) {
                hasVariadic = true;
            } else if (flags & TupleElementFlags.Optional) {
                hasOptional = true;
            } else if (!hasRest) {
                minLength++;
            }
        }

        const fixedLength = hasRest || hasVariadic ? undefined : elements.length;
        const combinedType = options.elementTypes.length > 0
            ? context.checker.getUnionType(options.elementTypes)
            : context.neverType;

        return {
            elements,
            readonly: options.readonly ?? false,
            minLength,
            fixedLength,
            hasOptional,
            hasRest,
            hasVariadic,
            combinedType,
        };
    }

    /**
     * Create an empty tuple
     */
    export function createEmptyTuple(readonly: boolean = false): TupleTypeInfo {
        return {
            elements: [],
            readonly,
            minLength: 0,
            fixedLength: 0,
            hasOptional: false,
            hasRest: false,
            hasVariadic: false,
            combinedType: undefined,
        };
    }

    /**
     * Check if a tuple is empty
     */
    export function isEmptyTuple(tuple: TupleTypeInfo): boolean {
        return tuple.elements.length === 0;
    }

    /**
     * Get tuple length as a type
     */
    export function getTupleLength(tuple: TupleTypeInfo): number | undefined {
        return tuple.fixedLength;
    }

    /**
     * Check if tuple has labeled elements
     */
    export function hasLabeledElements(tuple: TupleTypeInfo): boolean {
        return tuple.elements.some(e => e.label !== undefined);
    }

    /**
     * Convert tuple to string representation
     */
    export function tupleToString(
        tuple: TupleTypeInfo,
        typeToString: (type: Type) => string
    ): string {
        const parts: string[] = [];

        for (const element of tuple.elements) {
            parts.push(elementToString(element, typeToString));
        }

        const content = parts.join(", ");
        const prefix = tuple.readonly ? "readonly " : "";

        return `${prefix}[${content}]`;
    }

    /**
     * Create a readonly version of a tuple
     */
    export function toReadonlyTuple(tuple: TupleTypeInfo): TupleTypeInfo {
        return {
            ...tuple,
            readonly: true,
        };
    }

    /**
     * Create a mutable version of a tuple
     */
    export function toMutableTuple(tuple: TupleTypeInfo): TupleTypeInfo {
        return {
            ...tuple,
            readonly: false,
        };
    }

    /**
     * Helpers for common tuple operations
     */
    export const TupleHelpers = {
        /**
         * Create a pair tuple [A, B]
         */
        pair<A extends Type, B extends Type>(
            first: A,
            second: B,
            context: TupleContext
        ): TupleTypeInfo {
            return createTupleType({
                elementTypes: [first, second],
            }, context);
        },

        /**
         * Create a triple tuple [A, B, C]
         */
        triple<A extends Type, B extends Type, C extends Type>(
            first: A,
            second: B,
            third: C,
            context: TupleContext
        ): TupleTypeInfo {
            return createTupleType({
                elementTypes: [first, second, third],
            }, context);
        },

        /**
         * Create a labeled pair [a: A, b: B]
         */
        labeledPair<A extends Type, B extends Type>(
            firstLabel: string,
            first: A,
            secondLabel: string,
            second: B,
            context: TupleContext
        ): TupleTypeInfo {
            return createTupleType({
                elementTypes: [first, second],
                labels: [firstLabel, secondLabel],
            }, context);
        },

        /**
         * Create a head-rest tuple [T, ...T[]]
         */
        headRest<T extends Type>(
            head: T,
            rest: T,
            context: TupleContext
        ): TupleTypeInfo {
            return createTupleType({
                elementTypes: [head, rest],
                elementFlags: [TupleElementFlags.Required, TupleElementFlags.Rest],
            }, context);
        },

        /**
         * Create a rest-tail tuple [...T[], T]
         */
        restTail<T extends Type>(
            rest: T,
            tail: T,
            context: TupleContext
        ): TupleTypeInfo {
            return createTupleType({
                elementTypes: [rest, tail],
                elementFlags: [TupleElementFlags.Rest, TupleElementFlags.Required],
            }, context);
        },

        /**
         * Get the first element type
         */
        first(tuple: TupleTypeInfo): Type | undefined {
            if (tuple.elements.length === 0) return undefined;
            return tuple.elements[0].type;
        },

        /**
         * Get the last element type
         */
        last(tuple: TupleTypeInfo): Type | undefined {
            if (tuple.elements.length === 0) return undefined;
            return tuple.elements[tuple.elements.length - 1].type;
        },

        /**
         * Get all required element types
         */
        requiredTypes(tuple: TupleTypeInfo): Type[] {
            return tuple.elements
                .filter(isRequiredElement)
                .map(e => e.type);
        },

        /**
         * Get all optional element types
         */
        optionalTypes(tuple: TupleTypeInfo): Type[] {
            return tuple.elements
                .filter(isOptionalElement)
                .map(e => e.type);
        },

        /**
         * Get the rest element type if present
         */
        restType(tuple: TupleTypeInfo): Type | undefined {
            const restElement = tuple.elements.find(isRestElement);
            return restElement ? getRestElementArrayType(restElement) ?? restElement.type : undefined;
        },
    };

    /**
     * Validate a tuple type structure
     */
    export function validateTuple(tuple: TupleTypeInfo): { valid: boolean; errors: string[] } {
        const errors: string[] = [];

        // Validate element order
        const orderResult = validateElementOrder(tuple.elements);
        if (!orderResult.valid && orderResult.error) {
            errors.push(orderResult.error);
        }

        // Check for mixed labels
        if (hassMixedLabels(tuple.elements)) {
            errors.push("Tuple elements must either all have labels or none have labels.");
        }

        // Validate min length calculation
        let calculatedMin = 0;
        let seenRest = false;
        for (const element of tuple.elements) {
            if (isRestElement(element)) {
                seenRest = true;
            } else if (isRequiredElement(element) && !seenRest) {
                calculatedMin++;
            }
        }

        if (calculatedMin !== tuple.minLength) {
            errors.push(`Minimum length mismatch: calculated ${calculatedMin}, stored ${tuple.minLength}`);
        }

        return {
            valid: errors.length === 0,
            errors,
        };
    }
}
