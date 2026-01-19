/**
 * Tuple Types - Assignability Checking
 *
 * This module handles tuple type assignability including:
 * - Tuple to tuple assignability
 * - Tuple to array assignability
 * - Array to tuple assignability
 * - Length checking
 */

/* @internal */
namespace ts.tuples {
    /**
     * Check if source tuple is assignable to target tuple
     */
    export function isTupleAssignableTo(
        source: TupleTypeInfo,
        target: TupleTypeInfo,
        context: TupleContext,
        checkCallback?: (source: Type, target: Type) => boolean
    ): TupleAssignabilityResult {
        const isAssignable = checkCallback || ((s: Type, t: Type) =>
            context.checker.isTypeAssignableTo(s, t)
        );

        // Check readonly compatibility
        if (target.readonly && !source.readonly) {
            // A non-readonly tuple can be assigned to readonly
        } else if (source.readonly && !target.readonly) {
            return {
                assignable: false,
                error: "Cannot assign a readonly tuple to a mutable tuple.",
            };
        }

        // If target has fixed length, source must provide at least that many elements
        if (target.fixedLength !== undefined) {
            if (source.fixedLength !== undefined && source.fixedLength < target.minLength) {
                return {
                    assignable: false,
                    error: `Source tuple has ${source.fixedLength} elements, but target requires at least ${target.minLength}.`,
                };
            }
        }

        // Check element-by-element
        return checkElementAssignability(source, target, isAssignable, context);
    }

    /**
     * Check element-by-element assignability
     */
    function checkElementAssignability(
        source: TupleTypeInfo,
        target: TupleTypeInfo,
        isAssignable: (source: Type, target: Type) => boolean,
        context: TupleContext
    ): TupleAssignabilityResult {
        const sourceElements = source.elements;
        const targetElements = target.elements;
        const related: Array<{ message: string; index?: number }> = [];

        // Handle empty tuples
        if (targetElements.length === 0) {
            if (sourceElements.length === 0 || source.fixedLength === 0) {
                return { assignable: true };
            }
            if (source.minLength > 0) {
                return {
                    assignable: false,
                    error: "Source tuple has required elements, but target is empty.",
                };
            }
            return { assignable: true };
        }

        if (sourceElements.length === 0 && target.minLength > 0) {
            return {
                assignable: false,
                error: `Target requires at least ${target.minLength} elements, but source is empty.`,
            };
        }

        // Find rest element positions
        const sourceRestIndex = sourceElements.findIndex(isRestElement);
        const targetRestIndex = targetElements.findIndex(isRestElement);

        // Simple case: no rest elements
        if (sourceRestIndex < 0 && targetRestIndex < 0) {
            return checkFixedTupleAssignability(source, target, isAssignable, context);
        }

        // Complex case: handle rest elements
        return checkRestTupleAssignability(
            source, target, sourceRestIndex, targetRestIndex, isAssignable, context
        );
    }

    /**
     * Check assignability for fixed-length tuples
     */
    function checkFixedTupleAssignability(
        source: TupleTypeInfo,
        target: TupleTypeInfo,
        isAssignable: (source: Type, target: Type) => boolean,
        context: TupleContext
    ): TupleAssignabilityResult {
        const sourceElements = source.elements;
        const targetElements = target.elements;
        const related: Array<{ message: string; index?: number }> = [];

        // Check that source has enough required elements
        let sourceIndex = 0;

        for (let targetIndex = 0; targetIndex < targetElements.length; targetIndex++) {
            const targetElement = targetElements[targetIndex];

            if (isOptionalElement(targetElement)) {
                // Optional target can be satisfied by nothing or matching source
                if (sourceIndex < sourceElements.length) {
                    const sourceElement = sourceElements[sourceIndex];
                    if (!isAssignable(sourceElement.type, targetElement.type)) {
                        related.push({
                            message: `Element at index ${targetIndex} is not assignable.`,
                            index: targetIndex,
                        });
                    }
                    sourceIndex++;
                }
                // Otherwise, optional element is just not provided, which is fine
            } else if (isRequiredElement(targetElement)) {
                // Required target must have a matching source
                if (sourceIndex >= sourceElements.length) {
                    return {
                        assignable: false,
                        error: `Target requires element at index ${targetIndex}, but source doesn't have enough elements.`,
                        related,
                    };
                }

                const sourceElement = sourceElements[sourceIndex];
                if (!isAssignable(sourceElement.type, targetElement.type)) {
                    return {
                        assignable: false,
                        error: `Element at index ${targetIndex} is not assignable.`,
                        related: [{
                            message: `Type '${typeToString(sourceElement.type)}' is not assignable to type '${typeToString(targetElement.type)}'.`,
                            index: targetIndex,
                        }],
                    };
                }
                sourceIndex++;
            }
        }

        // Check for extra source elements
        if (sourceIndex < sourceElements.length && !target.hasRest) {
            // Extra elements in source - might be an error depending on context
            // For structural compatibility, extra elements are usually allowed
        }

        if (related.length > 0) {
            return {
                assignable: false,
                error: "Tuple elements are not assignable.",
                related,
            };
        }

        return { assignable: true };
    }

    /**
     * Check assignability with rest elements
     */
    function checkRestTupleAssignability(
        source: TupleTypeInfo,
        target: TupleTypeInfo,
        sourceRestIndex: number,
        targetRestIndex: number,
        isAssignable: (source: Type, target: Type) => boolean,
        context: TupleContext
    ): TupleAssignabilityResult {
        const sourceElements = source.elements;
        const targetElements = target.elements;

        // Case 1: Target has rest, source doesn't
        if (targetRestIndex >= 0 && sourceRestIndex < 0) {
            // Check elements before rest
            for (let i = 0; i < targetRestIndex; i++) {
                const targetElement = targetElements[i];
                if (i >= sourceElements.length) {
                    if (isRequiredElement(targetElement)) {
                        return {
                            assignable: false,
                            error: `Missing required element at index ${i}.`,
                        };
                    }
                    continue;
                }

                const sourceElement = sourceElements[i];
                if (!isAssignable(sourceElement.type, targetElement.type)) {
                    return {
                        assignable: false,
                        error: `Element at index ${i} is not assignable.`,
                    };
                }
            }

            // Rest target elements must accept remaining source elements
            const targetRestElement = targetElements[targetRestIndex];
            const restType = getRestElementArrayType(targetRestElement) ?? targetRestElement.type;

            for (let i = targetRestIndex; i < sourceElements.length; i++) {
                const sourceElement = sourceElements[i];
                if (!isAssignable(sourceElement.type, restType)) {
                    return {
                        assignable: false,
                        error: `Element at index ${i} is not assignable to rest element type.`,
                    };
                }
            }

            // Check elements after rest in target
            const afterRest = targetElements.slice(targetRestIndex + 1);
            // For trailing elements, we need to align from the end
            const sourceTrailingStart = sourceElements.length - afterRest.length;

            for (let i = 0; i < afterRest.length; i++) {
                const targetElement = afterRest[i];
                const sourceIdx = sourceTrailingStart + i;

                if (sourceIdx < targetRestIndex || sourceIdx < 0) {
                    if (isRequiredElement(targetElement)) {
                        return {
                            assignable: false,
                            error: `Missing required trailing element.`,
                        };
                    }
                    continue;
                }

                if (sourceIdx < sourceElements.length) {
                    const sourceElement = sourceElements[sourceIdx];
                    if (!isAssignable(sourceElement.type, targetElement.type)) {
                        return {
                            assignable: false,
                            error: `Trailing element is not assignable.`,
                        };
                    }
                }
            }

            return { assignable: true };
        }

        // Case 2: Source has rest, target doesn't
        if (sourceRestIndex >= 0 && targetRestIndex < 0) {
            // Source with rest can't be assigned to fixed-length target
            // unless target is longer than source's minimum
            if (target.fixedLength !== undefined && source.minLength > target.fixedLength) {
                return {
                    assignable: false,
                    error: `Source has minimum ${source.minLength} elements, but target only accepts ${target.fixedLength}.`,
                };
            }

            // Check that source's required elements match target
            for (let i = 0; i < sourceRestIndex; i++) {
                if (i >= targetElements.length) break;

                const sourceElement = sourceElements[i];
                const targetElement = targetElements[i];

                if (!isAssignable(sourceElement.type, targetElement.type)) {
                    return {
                        assignable: false,
                        error: `Element at index ${i} is not assignable.`,
                    };
                }
            }

            // For remaining target elements, they must be compatible with rest
            const sourceRestElement = sourceElements[sourceRestIndex];
            const restType = getRestElementArrayType(sourceRestElement) ?? sourceRestElement.type;

            for (let i = sourceRestIndex; i < targetElements.length; i++) {
                const targetElement = targetElements[i];
                if (!isAssignable(restType, targetElement.type)) {
                    if (isRequiredElement(targetElement)) {
                        return {
                            assignable: false,
                            error: `Rest element type is not assignable to required element at index ${i}.`,
                        };
                    }
                }
            }

            return { assignable: true };
        }

        // Case 3: Both have rest
        if (sourceRestIndex >= 0 && targetRestIndex >= 0) {
            // Check elements before rest match
            const minPrefix = Math.min(sourceRestIndex, targetRestIndex);

            for (let i = 0; i < minPrefix; i++) {
                const sourceElement = sourceElements[i];
                const targetElement = targetElements[i];

                if (!isAssignable(sourceElement.type, targetElement.type)) {
                    return {
                        assignable: false,
                        error: `Element at index ${i} is not assignable.`,
                    };
                }
            }

            // Check rest types are compatible
            const sourceRestElement = sourceElements[sourceRestIndex];
            const targetRestElement = targetElements[targetRestIndex];
            const sourceRestType = getRestElementArrayType(sourceRestElement) ?? sourceRestElement.type;
            const targetRestType = getRestElementArrayType(targetRestElement) ?? targetRestElement.type;

            if (!isAssignable(sourceRestType, targetRestType)) {
                return {
                    assignable: false,
                    error: "Rest element types are not assignable.",
                };
            }

            return { assignable: true };
        }

        return { assignable: true };
    }

    /**
     * Simple type to string for error messages
     */
    function typeToString(type: Type): string {
        // Use the type's toString or a placeholder
        if ((type as any).intrinsicName) {
            return (type as any).intrinsicName;
        }
        if (type.flags & TypeFlags.StringLiteral) {
            return `"${(type as StringLiteralType).value}"`;
        }
        if (type.flags & TypeFlags.NumberLiteral) {
            return String((type as NumberLiteralType).value);
        }
        return "Type";
    }

    /**
     * Check if a tuple is assignable to an array
     */
    export function isTupleAssignableToArray(
        tuple: TupleTypeInfo,
        arrayElementType: Type,
        context: TupleContext,
        checkCallback?: (source: Type, target: Type) => boolean
    ): TupleAssignabilityResult {
        const isAssignable = checkCallback || ((s: Type, t: Type) =>
            context.checker.isTypeAssignableTo(s, t)
        );

        // All tuple elements must be assignable to array element type
        for (let i = 0; i < tuple.elements.length; i++) {
            const element = tuple.elements[i];
            let elementType = element.type;

            // For rest elements, get the array element type
            if (isRestElement(element)) {
                const restArrayType = getRestElementArrayType(element);
                if (restArrayType) {
                    elementType = restArrayType;
                }
            }

            if (!isAssignable(elementType, arrayElementType)) {
                return {
                    assignable: false,
                    error: `Element at index ${i} of type '${typeToString(elementType)}' is not assignable to array element type.`,
                };
            }
        }

        return { assignable: true };
    }

    /**
     * Check if an array is assignable to a tuple
     */
    export function isArrayAssignableToTuple(
        arrayElementType: Type,
        tuple: TupleTypeInfo,
        context: TupleContext,
        checkCallback?: (source: Type, target: Type) => boolean
    ): TupleAssignabilityResult {
        const isAssignable = checkCallback || ((s: Type, t: Type) =>
            context.checker.isTypeAssignableTo(s, t)
        );

        // Arrays can only be assigned to tuples with rest elements
        // or tuples where all elements are compatible with the array element type
        if (!tuple.hasRest && tuple.fixedLength !== undefined) {
            // Fixed tuple - array can't be assigned as length is unknown
            return {
                assignable: false,
                error: "Array type cannot be assigned to a fixed-length tuple.",
            };
        }

        // Check that array element type is assignable to each tuple element
        for (const element of tuple.elements) {
            if (isRestElement(element)) {
                const restArrayType = getRestElementArrayType(element);
                if (restArrayType && !isAssignable(arrayElementType, restArrayType)) {
                    return {
                        assignable: false,
                        error: "Array element type is not assignable to tuple rest element.",
                    };
                }
            } else if (isRequiredElement(element)) {
                // Required elements must be provided - but array length is unknown
                return {
                    assignable: false,
                    error: "Array cannot satisfy required tuple elements.",
                };
            }
        }

        return { assignable: true };
    }

    /**
     * Get the length type of a tuple
     */
    export function getTupleLengthType(
        tuple: TupleTypeInfo,
        context: TupleContext
    ): Type {
        if (tuple.fixedLength !== undefined) {
            return context.checker.getNumberLiteralType(tuple.fixedLength);
        }

        // For tuples with rest/optional, length is number
        return context.numberType;
    }

    /**
     * Analyze tuple length constraints
     */
    export function analyzeTupleLength(tuple: TupleTypeInfo): TupleLengthInfo {
        if (tuple.fixedLength !== undefined) {
            return {
                minLength: tuple.minLength,
                maxLength: tuple.fixedLength,
                isFixed: true,
                exactLength: tuple.fixedLength,
            };
        }

        if (tuple.hasRest) {
            return {
                minLength: tuple.minLength,
                maxLength: undefined,
                isFixed: false,
            };
        }

        // Has optional elements
        return {
            minLength: tuple.minLength,
            maxLength: tuple.elements.length,
            isFixed: false,
        };
    }

    /**
     * Check if a numeric index is valid for a tuple
     */
    export function isValidTupleIndex(
        tuple: TupleTypeInfo,
        index: number
    ): boolean {
        if (index < 0) {
            return false;
        }

        if (tuple.fixedLength !== undefined) {
            return index < tuple.fixedLength;
        }

        // For variable length tuples, any non-negative index is potentially valid
        return true;
    }

    /**
     * Get the type at a specific tuple index
     */
    export function getTypeAtIndex(
        tuple: TupleTypeInfo,
        index: number,
        context: TupleContext
    ): Type {
        const elements = tuple.elements;

        // Direct element access
        if (index < elements.length) {
            const element = elements[index];

            // Check if this is a rest element
            if (isRestElement(element)) {
                return getRestElementArrayType(element) ?? element.type;
            }

            // For optional elements, include undefined
            if (isOptionalElement(element)) {
                return context.checker.getUnionType([element.type, context.undefinedType]);
            }

            return element.type;
        }

        // Beyond fixed elements - check for rest
        const restElement = elements.find(isRestElement);
        if (restElement) {
            return getRestElementArrayType(restElement) ?? restElement.type;
        }

        // Out of bounds for fixed tuple
        if (tuple.fixedLength !== undefined && index >= tuple.fixedLength) {
            return context.undefinedType;
        }

        return context.undefinedType;
    }
}
