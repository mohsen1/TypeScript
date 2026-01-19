/**
 * Tuple Types - Inference
 *
 * This module handles tuple type inference from:
 * - Array literals
 * - Spread expressions
 * - Function rest parameters
 */

/* @internal */
namespace ts.tuples {
    /**
     * Infer tuple type from an array literal expression
     */
    export function inferTupleFromArrayLiteral(
        elementTypes: Type[],
        elementFlags: TupleElementFlags[],
        context: TupleContext,
        options: TupleOperationOptions = {}
    ): TupleInferenceResult {
        const elements: TupleElement[] = [];
        let minLength = 0;
        let hasOptional = false;
        let hasRest = false;
        let hasVariadic = false;

        for (let i = 0; i < elementTypes.length; i++) {
            const type = elementTypes[i];
            const flags = elementFlags[i] ?? TupleElementFlags.Required;

            elements.push({
                type,
                flags,
            });

            if (flags & TupleElementFlags.Rest) {
                hasRest = true;
            } else if (flags & TupleElementFlags.Variadic) {
                hasVariadic = true;
            } else if (flags & TupleElementFlags.Optional) {
                hasOptional = true;
            } else {
                minLength++;
            }
        }

        const fixedLength = hasRest || hasVariadic ? undefined : elements.length;
        const combinedType = elementTypes.length > 0
            ? context.checker.getUnionType(elementTypes)
            : context.neverType;

        return {
            tupleInfo: {
                elements,
                readonly: false,
                minLength,
                fixedLength,
                hasOptional,
                hasRest,
                hasVariadic,
                combinedType,
            },
            success: true,
        };
    }

    /**
     * Infer tuple type from spread elements in array
     *
     * [a, ...arr, b] -> infers proper tuple structure
     */
    export function inferTupleWithSpreads(
        parts: Array<{ type: Type; spread: boolean }>,
        context: TupleContext
    ): TupleInferenceResult {
        const elements: TupleElement[] = [];
        let minLength = 0;
        let hasOptional = false;
        let hasRest = false;
        let hasVariadic = false;

        for (const part of parts) {
            if (part.spread) {
                // This is a spread - check what type it spreads
                const spreadResult = spreadTupleType(part.type, context);

                if (spreadResult.success) {
                    for (const spreadElement of spreadResult.elements) {
                        elements.push(spreadElement);

                        if (isRestElement(spreadElement)) {
                            hasRest = true;
                        } else if (isVariadicElement(spreadElement)) {
                            hasVariadic = true;
                        } else if (isOptionalElement(spreadElement)) {
                            hasOptional = true;
                        } else if (!hasRest) {
                            minLength++;
                        }
                    }
                } else {
                    // Fallback: treat as variadic
                    elements.push(createVariadicElement(part.type));
                    hasVariadic = true;
                }
            } else {
                // Regular element
                elements.push(createRequiredElement(part.type));
                if (!hasRest) {
                    minLength++;
                }
            }
        }

        const fixedLength = hasRest || hasVariadic ? undefined : elements.length;
        const allTypes = elements.map(e => e.type);
        const combinedType = allTypes.length > 0
            ? context.checker.getUnionType(allTypes)
            : context.neverType;

        return {
            tupleInfo: {
                elements,
                readonly: false,
                minLength,
                fixedLength,
                hasOptional,
                hasRest,
                hasVariadic,
                combinedType,
            },
            success: true,
        };
    }

    /**
     * Infer tuple type from function rest parameter
     *
     * function foo(...args: [string, number, ...boolean[]]) {}
     */
    export function inferTupleFromRestParameter(
        restType: Type,
        context: TupleContext
    ): TupleInferenceResult {
        // Check if rest type is already a tuple
        if (isTupleType(restType)) {
            const typeRef = restType as TypeReference;
            const info = getTupleTypeInfo(typeRef, context);

            if (info) {
                return {
                    tupleInfo: info,
                    success: true,
                };
            }
        }

        // Check if it's an array type
        if (restType.flags & TypeFlags.Object) {
            const objectType = restType as ObjectType;

            if (objectType.objectFlags & ObjectFlags.Reference) {
                const typeRef = restType as TypeReference;
                const target = typeRef.target;

                if (target && (target as any).symbol?.name === "Array") {
                    const elementType = typeRef.typeArguments?.[0] ?? context.unknownType;

                    return {
                        tupleInfo: {
                            elements: [createRestElement(elementType)],
                            readonly: false,
                            minLength: 0,
                            fixedLength: undefined,
                            hasOptional: false,
                            hasRest: true,
                            hasVariadic: false,
                            combinedType: elementType,
                        },
                        success: true,
                    };
                }
            }
        }

        // Fallback: single rest element
        return {
            tupleInfo: {
                elements: [createRestElement(restType)],
                readonly: false,
                minLength: 0,
                fixedLength: undefined,
                hasOptional: false,
                hasRest: true,
                hasVariadic: false,
                combinedType: restType,
            },
            success: true,
        };
    }

    /**
     * Create a tuple type from a function parameter list
     */
    export function createTupleFromParameters(
        parameters: Array<{
            type: Type;
            optional: boolean;
            rest: boolean;
            name?: string;
        }>,
        context: TupleContext
    ): TupleTypeInfo {
        const elements: TupleElement[] = [];
        let minLength = 0;
        let hasOptional = false;
        let hasRest = false;

        for (const param of parameters) {
            let flags: TupleElementFlags;

            if (param.rest) {
                flags = TupleElementFlags.Rest;
                hasRest = true;
            } else if (param.optional) {
                flags = TupleElementFlags.Optional;
                hasOptional = true;
            } else {
                flags = TupleElementFlags.Required;
                if (!hasRest) {
                    minLength++;
                }
            }

            elements.push({
                type: param.type,
                label: param.name,
                flags,
            });
        }

        const fixedLength = hasRest ? undefined : elements.length;
        const allTypes = elements.map(e => e.type);
        const combinedType = allTypes.length > 0
            ? context.checker.getUnionType(allTypes)
            : context.neverType;

        return {
            elements,
            readonly: false,
            minLength,
            fixedLength,
            hasOptional,
            hasRest,
            hasVariadic: false,
            combinedType,
        };
    }

    /**
     * Infer contextual tuple type from target type
     */
    export function inferContextualTupleType(
        sourceElements: Type[],
        targetTuple: TupleTypeInfo,
        context: TupleContext
    ): TupleInferenceResult {
        const elements: TupleElement[] = [];
        let minLength = 0;
        let hasOptional = false;
        let hasRest = false;

        const targetElements = targetTuple.elements;
        let sourceIndex = 0;

        for (let targetIndex = 0; targetIndex < targetElements.length; targetIndex++) {
            const targetElement = targetElements[targetIndex];

            if (isRestElement(targetElement)) {
                // Consume remaining source elements as rest
                const restTypes: Type[] = [];
                while (sourceIndex < sourceElements.length) {
                    restTypes.push(sourceElements[sourceIndex]);
                    sourceIndex++;
                }

                const restArrayType = restTypes.length > 0
                    ? context.checker.getUnionType(restTypes)
                    : getRestElementArrayType(targetElement) ?? targetElement.type;

                elements.push(createRestElement(restArrayType, targetElement.label));
                hasRest = true;
            } else if (sourceIndex < sourceElements.length) {
                // Match source to target
                const sourceType = sourceElements[sourceIndex];
                sourceIndex++;

                if (isOptionalElement(targetElement)) {
                    elements.push(createOptionalElement(sourceType, targetElement.label));
                    hasOptional = true;
                } else {
                    elements.push(createRequiredElement(sourceType, targetElement.label));
                    minLength++;
                }
            } else if (isOptionalElement(targetElement)) {
                // Target is optional and no more source elements
                elements.push(cloneElement(targetElement));
                hasOptional = true;
            }
            // Required target with no source - skip (will error in type checking)
        }

        // Handle excess source elements
        while (sourceIndex < sourceElements.length) {
            if (targetTuple.hasRest) {
                // Already handled
                break;
            }
            // Extra elements
            elements.push(createRequiredElement(sourceElements[sourceIndex]));
            minLength++;
            sourceIndex++;
        }

        const fixedLength = hasRest ? undefined : elements.length;
        const allTypes = elements.map(e => e.type);
        const combinedType = allTypes.length > 0
            ? context.checker.getUnionType(allTypes)
            : context.neverType;

        return {
            tupleInfo: {
                elements,
                readonly: targetTuple.readonly,
                minLength,
                fixedLength,
                hasOptional,
                hasRest,
                hasVariadic: false,
                combinedType,
            },
            success: true,
        };
    }

    /**
     * Widen a tuple type to an array type
     */
    export function widenTupleToArray(
        tuple: TupleTypeInfo,
        context: TupleContext
    ): Type {
        if (!tuple.combinedType) {
            return context.checker.createArrayType(context.unknownType);
        }

        return context.checker.createArrayType(tuple.combinedType);
    }

    /**
     * Narrow an array type to a tuple type with specific length
     */
    export function narrowArrayToTuple(
        arrayElementType: Type,
        length: number,
        context: TupleContext
    ): TupleTypeInfo {
        const elements: TupleElement[] = [];

        for (let i = 0; i < length; i++) {
            elements.push(createRequiredElement(arrayElementType));
        }

        return {
            elements,
            readonly: false,
            minLength: length,
            fixedLength: length,
            hasOptional: false,
            hasRest: false,
            hasVariadic: false,
            combinedType: arrayElementType,
        };
    }

    /**
     * Infer tuple element types from indexed access patterns
     */
    export function inferTupleFromIndexedAccess(
        baseType: Type,
        indexType: Type,
        context: TupleContext
    ): Type | undefined {
        if (!isTupleType(baseType)) {
            return undefined;
        }

        const typeRef = baseType as TypeReference;
        const tupleInfo = getTupleTypeInfo(typeRef, context);

        if (!tupleInfo) {
            return undefined;
        }

        // Check for numeric literal index
        if (indexType.flags & TypeFlags.NumberLiteral) {
            const index = (indexType as NumberLiteralType).value;
            return getTypeAtIndex(tupleInfo, index, context);
        }

        // Check for string literal index (labeled tuple access)
        if (indexType.flags & TypeFlags.StringLiteral) {
            const label = (indexType as StringLiteralType).value;
            const element = tupleInfo.elements.find(e => e.label === label);

            if (element) {
                if (isOptionalElement(element)) {
                    return context.checker.getUnionType([element.type, context.undefinedType]);
                }
                return element.type;
            }
        }

        // For number type, return union of all element types
        if (indexType.flags & TypeFlags.Number) {
            return tupleInfo.combinedType;
        }

        return undefined;
    }

    /**
     * Create a mapped tuple type
     * Maps over each element, transforming types
     */
    export function mapTupleType(
        tuple: TupleTypeInfo,
        mapper: (elementType: Type, index: number, element: TupleElement) => Type,
        context: TupleContext
    ): TupleTypeInfo {
        const mappedElements: TupleElement[] = [];

        for (let i = 0; i < tuple.elements.length; i++) {
            const element = tuple.elements[i];
            const mappedType = mapper(element.type, i, element);

            mappedElements.push({
                ...element,
                type: mappedType,
            });
        }

        const allTypes = mappedElements.map(e => e.type);
        const combinedType = allTypes.length > 0
            ? context.checker.getUnionType(allTypes)
            : context.neverType;

        return {
            ...tuple,
            elements: mappedElements,
            combinedType,
        };
    }

    /**
     * Filter tuple elements by predicate
     */
    export function filterTupleElements(
        tuple: TupleTypeInfo,
        predicate: (element: TupleElement, index: number) => boolean,
        context: TupleContext
    ): TupleTypeInfo {
        const filteredElements = tuple.elements.filter(predicate);
        let minLength = 0;
        let hasOptional = false;
        let hasRest = false;
        let hasVariadic = false;

        for (const element of filteredElements) {
            if (isRestElement(element)) {
                hasRest = true;
            } else if (isVariadicElement(element)) {
                hasVariadic = true;
            } else if (isOptionalElement(element)) {
                hasOptional = true;
            } else if (!hasRest) {
                minLength++;
            }
        }

        const fixedLength = hasRest || hasVariadic ? undefined : filteredElements.length;
        const allTypes = filteredElements.map(e => e.type);
        const combinedType = allTypes.length > 0
            ? context.checker.getUnionType(allTypes)
            : context.neverType;

        return {
            elements: filteredElements,
            readonly: tuple.readonly,
            minLength,
            fixedLength,
            hasOptional,
            hasRest,
            hasVariadic,
            combinedType,
        };
    }
}
