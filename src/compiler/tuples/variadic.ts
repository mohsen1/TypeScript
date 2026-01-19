/**
 * Tuple Types - Variadic Tuple Operations
 *
 * This module implements variadic tuple types including:
 * - Spreading tuples into other tuples
 * - Concatenating tuples
 * - Handling variadic type parameters
 */

/* @internal */
namespace ts.tuples {
    /**
     * Spread a tuple type into individual elements
     *
     * For [string, ...number[], boolean], returns:
     * - string (required)
     * - number (rest)
     * - boolean (required, after rest)
     */
    export function spreadTupleType(
        tupleType: Type,
        context: TupleContext
    ): SpreadResult {
        const elements: TupleElement[] = [];

        // Check if it's actually a tuple type
        if (!(tupleType.flags & TypeFlags.Object)) {
            return {
                elements: [createRestElement(tupleType)],
                success: true,
            };
        }

        const objectType = tupleType as ObjectType;

        // Check for tuple target
        if (objectType.objectFlags & ObjectFlags.Reference) {
            const typeRef = tupleType as TypeReference;
            const target = typeRef.target as TupleType;

            if (target && isTupleType(target)) {
                const tupleInfo = getTupleTypeInfo(typeRef, context);
                if (tupleInfo) {
                    return {
                        elements: tupleInfo.elements,
                        success: true,
                    };
                }
            }

            // Check if it's an array type
            if (target && (target as any).symbol?.name === "Array") {
                const elementType = typeRef.typeArguments?.[0] ?? context.unknownType;
                return {
                    elements: [createRestElement(elementType)],
                    success: true,
                };
            }
        }

        // Fallback: treat as rest element of the whole type
        return {
            elements: [createRestElement(tupleType)],
            success: true,
        };
    }

    /**
     * Check if a type is a tuple type
     */
    export function isTupleType(type: Type): boolean {
        if (!(type.flags & TypeFlags.Object)) {
            return false;
        }

        const objectType = type as ObjectType;

        if (objectType.objectFlags & ObjectFlags.Reference) {
            const target = (type as TypeReference).target as TupleType;
            // Check for TupleType marker
            return !!(target?.objectFlags & ObjectFlags.Tuple);
        }

        return !!(objectType.objectFlags & ObjectFlags.Tuple);
    }

    /**
     * Get tuple type information from a type reference
     */
    export function getTupleTypeInfo(
        type: TypeReference,
        context: TupleContext
    ): TupleTypeInfo | undefined {
        const target = type.target as TupleType;
        if (!target || !(target.objectFlags & ObjectFlags.Tuple)) {
            return undefined;
        }

        const typeArguments = type.typeArguments || [];
        const elementFlags = target.elementFlags || [];
        const labeledElementDeclarations = target.labeledElementDeclarations;

        const elements: TupleElement[] = [];
        let minLength = 0;
        let hasOptional = false;
        let hasRest = false;
        let hasVariadic = false;

        for (let i = 0; i < typeArguments.length; i++) {
            const elementType = typeArguments[i];
            const flags = elementFlags[i] ?? ElementFlags.Required;
            const label = labeledElementDeclarations?.[i]?.name
                ? (labeledElementDeclarations[i].name as Identifier).text
                : undefined;

            let tupleFlags: TupleElementFlags;

            if (flags & ElementFlags.Rest) {
                tupleFlags = TupleElementFlags.Rest;
                hasRest = true;
            } else if (flags & ElementFlags.Variadic) {
                tupleFlags = TupleElementFlags.Variadic;
                hasVariadic = true;
            } else if (flags & ElementFlags.Optional) {
                tupleFlags = TupleElementFlags.Optional;
                hasOptional = true;
            } else {
                tupleFlags = TupleElementFlags.Required;
                minLength++;
            }

            elements.push({
                type: elementType,
                label,
                flags: tupleFlags,
            });
        }

        // Calculate fixed length
        const fixedLength = hasRest || hasVariadic ? undefined : elements.length;

        // Get combined type
        const allTypes = elements.map(e => e.type);
        const combinedType = allTypes.length > 0
            ? context.checker.getUnionType(allTypes)
            : context.neverType;

        return {
            elements,
            readonly: target.readonly ?? false,
            minLength,
            fixedLength,
            hasOptional,
            hasRest,
            hasVariadic,
            combinedType,
        };
    }

    /**
     * Concatenate multiple tuples
     *
     * [...A, ...B] where A = [string, number] and B = [boolean]
     * results in [string, number, boolean]
     */
    export function concatenateTuples(
        tuples: TupleTypeInfo[],
        context: TupleContext
    ): TupleTypeInfo {
        const elements: TupleElement[] = [];
        let minLength = 0;
        let hasOptional = false;
        let hasRest = false;
        let hasVariadic = false;
        let isReadonly = false;

        for (let tupleIndex = 0; tupleIndex < tuples.length; tupleIndex++) {
            const tuple = tuples[tupleIndex];
            const isLast = tupleIndex === tuples.length - 1;

            // Handle readonly - if any is readonly, result is readonly
            if (tuple.readonly) {
                isReadonly = true;
            }

            for (const element of tuple.elements) {
                // If there's already a rest element and we're adding more,
                // we need to handle it specially
                if (hasRest && !isRestElement(element) && !isVariadicElement(element)) {
                    // Elements after rest become trailing required/optional
                    elements.push(element);
                } else {
                    elements.push(cloneElement(element));
                }

                if (isRestElement(element)) {
                    hasRest = true;
                } else if (isVariadicElement(element)) {
                    hasVariadic = true;
                } else if (isOptionalElement(element)) {
                    hasOptional = true;
                } else if (isRequiredElement(element) && !hasRest) {
                    minLength++;
                }
            }
        }

        // Calculate fixed length
        const fixedLength = hasRest || hasVariadic ? undefined : elements.length;

        // Get combined type
        const allTypes = elements.map(e => e.type);
        const combinedType = allTypes.length > 0
            ? context.checker.getUnionType(allTypes)
            : context.neverType;

        return {
            elements,
            readonly: isReadonly,
            minLength,
            fixedLength,
            hasOptional,
            hasRest,
            hasVariadic,
            combinedType,
        };
    }

    /**
     * Spread variadic elements into the tuple
     *
     * Resolves [...T] where T is known to be a specific tuple type
     */
    export function resolveVariadicElements(
        info: TupleTypeInfo,
        typeResolver: (type: Type) => TupleTypeInfo | undefined,
        context: TupleContext
    ): TupleTypeInfo {
        const resolvedElements: TupleElement[] = [];
        let minLength = 0;
        let hasOptional = false;
        let hasRest = false;
        let hasVariadic = false;

        for (const element of info.elements) {
            if (isVariadicElement(element)) {
                // Try to resolve the variadic type
                const resolved = typeResolver(element.type);

                if (resolved) {
                    // Inline the resolved tuple
                    for (const resolvedElement of resolved.elements) {
                        resolvedElements.push(resolvedElement);

                        if (isRestElement(resolvedElement)) {
                            hasRest = true;
                        } else if (isVariadicElement(resolvedElement)) {
                            hasVariadic = true;
                        } else if (isOptionalElement(resolvedElement)) {
                            hasOptional = true;
                        } else if (isRequiredElement(resolvedElement) && !hasRest) {
                            minLength++;
                        }
                    }
                } else {
                    // Keep as variadic if can't resolve
                    resolvedElements.push(element);
                    hasVariadic = true;
                }
            } else {
                resolvedElements.push(element);

                if (isRestElement(element)) {
                    hasRest = true;
                } else if (isOptionalElement(element)) {
                    hasOptional = true;
                } else if (isRequiredElement(element) && !hasRest) {
                    minLength++;
                }
            }
        }

        const fixedLength = hasRest || hasVariadic ? undefined : resolvedElements.length;
        const allTypes = resolvedElements.map(e => e.type);
        const combinedType = allTypes.length > 0
            ? context.checker.getUnionType(allTypes)
            : context.neverType;

        return {
            elements: resolvedElements,
            readonly: info.readonly,
            minLength,
            fixedLength,
            hasOptional,
            hasRest,
            hasVariadic,
            combinedType,
        };
    }

    /**
     * Slice a tuple type
     * Similar to Array.prototype.slice
     */
    export function sliceTuple(
        info: TupleTypeInfo,
        start: number,
        end?: number,
        context?: TupleContext
    ): TupleTypeInfo {
        const actualEnd = end ?? info.elements.length;
        const slicedElements = info.elements.slice(start, actualEnd);

        // Recalculate properties
        let minLength = 0;
        let hasOptional = false;
        let hasRest = false;
        let hasVariadic = false;

        for (const element of slicedElements) {
            if (isRestElement(element)) {
                hasRest = true;
            } else if (isVariadicElement(element)) {
                hasVariadic = true;
            } else if (isOptionalElement(element)) {
                hasOptional = true;
            } else if (isRequiredElement(element) && !hasRest) {
                minLength++;
            }
        }

        const fixedLength = hasRest || hasVariadic ? undefined : slicedElements.length;

        return {
            elements: slicedElements,
            readonly: info.readonly,
            minLength,
            fixedLength,
            hasOptional,
            hasRest,
            hasVariadic,
            combinedType: info.combinedType,
        };
    }

    /**
     * Prepend elements to a tuple
     */
    export function prependToTuple(
        info: TupleTypeInfo,
        elements: TupleElement[],
        context: TupleContext
    ): TupleTypeInfo {
        const newElements = [...elements, ...info.elements];
        return recalculateTupleInfo(newElements, info.readonly, context);
    }

    /**
     * Append elements to a tuple
     */
    export function appendToTuple(
        info: TupleTypeInfo,
        elements: TupleElement[],
        context: TupleContext
    ): TupleTypeInfo {
        // If the tuple has a rest element, we need to be careful
        if (info.hasRest) {
            // Find the rest element position
            const restIndex = info.elements.findIndex(isRestElement);
            if (restIndex >= 0) {
                // Insert after rest but before trailing elements
                const before = info.elements.slice(0, restIndex + 1);
                const after = info.elements.slice(restIndex + 1);
                const newElements = [...before, ...elements, ...after];
                return recalculateTupleInfo(newElements, info.readonly, context);
            }
        }

        const newElements = [...info.elements, ...elements];
        return recalculateTupleInfo(newElements, info.readonly, context);
    }

    /**
     * Recalculate tuple info from elements
     */
    function recalculateTupleInfo(
        elements: TupleElement[],
        readonly: boolean,
        context: TupleContext
    ): TupleTypeInfo {
        let minLength = 0;
        let hasOptional = false;
        let hasRest = false;
        let hasVariadic = false;

        for (const element of elements) {
            if (isRestElement(element)) {
                hasRest = true;
            } else if (isVariadicElement(element)) {
                hasVariadic = true;
            } else if (isOptionalElement(element)) {
                hasOptional = true;
            } else if (isRequiredElement(element) && !hasRest) {
                minLength++;
            }
        }

        const fixedLength = hasRest || hasVariadic ? undefined : elements.length;
        const allTypes = elements.map(e => e.type);
        const combinedType = allTypes.length > 0
            ? context.checker.getUnionType(allTypes)
            : context.neverType;

        return {
            elements,
            readonly,
            minLength,
            fixedLength,
            hasOptional,
            hasRest,
            hasVariadic,
            combinedType,
        };
    }

    /**
     * Create a homogeneous tuple (all same type)
     */
    export function createHomogeneousTuple(
        elementType: Type,
        length: number,
        readonly: boolean = false
    ): TupleTypeInfo {
        const elements: TupleElement[] = [];
        for (let i = 0; i < length; i++) {
            elements.push(createRequiredElement(elementType));
        }

        return {
            elements,
            readonly,
            minLength: length,
            fixedLength: length,
            hasOptional: false,
            hasRest: false,
            hasVariadic: false,
            combinedType: elementType,
        };
    }

    /**
     * Convert tuple to array type
     */
    export function tupleToArrayType(
        info: TupleTypeInfo,
        context: TupleContext
    ): Type {
        if (!info.combinedType) {
            return context.checker.createArrayType(context.unknownType);
        }

        const arrayType = context.checker.createArrayType(info.combinedType);

        if (info.readonly) {
            return context.checker.createTypeFromGenericGlobalType(
                context.checker.getGlobalReadonlyArrayType(),
                [info.combinedType]
            );
        }

        return arrayType;
    }
}
