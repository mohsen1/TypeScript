/**
 * Tuple Types - Element Operations
 *
 * This module handles tuple element operations including:
 * - Creating and manipulating tuple elements
 * - Labeled tuple elements
 * - Optional elements
 * - Rest elements
 */

/* @internal */
namespace ts.tuples {
    /**
     * Create a required tuple element
     */
    export function createRequiredElement(type: Type, label?: string): TupleElement {
        return {
            type,
            label,
            flags: TupleElementFlags.Required,
        };
    }

    /**
     * Create an optional tuple element
     */
    export function createOptionalElement(type: Type, label?: string): TupleElement {
        return {
            type,
            label,
            flags: TupleElementFlags.Optional,
        };
    }

    /**
     * Create a rest tuple element
     */
    export function createRestElement(type: Type, label?: string): TupleElement {
        return {
            type,
            label,
            flags: TupleElementFlags.Rest,
        };
    }

    /**
     * Create a variadic tuple element (spread of another tuple)
     */
    export function createVariadicElement(type: Type, label?: string): TupleElement {
        return {
            type,
            label,
            flags: TupleElementFlags.Variadic,
        };
    }

    /**
     * Check if element is required
     */
    export function isRequiredElement(element: TupleElement): boolean {
        return (element.flags & TupleElementFlags.Required) !== 0;
    }

    /**
     * Check if element is optional
     */
    export function isOptionalElement(element: TupleElement): boolean {
        return (element.flags & TupleElementFlags.Optional) !== 0;
    }

    /**
     * Check if element is a rest element
     */
    export function isRestElement(element: TupleElement): boolean {
        return (element.flags & TupleElementFlags.Rest) !== 0;
    }

    /**
     * Check if element is variadic
     */
    export function isVariadicElement(element: TupleElement): boolean {
        return (element.flags & TupleElementFlags.Variadic) !== 0;
    }

    /**
     * Check if element has a label
     */
    export function hasLabel(element: TupleElement): boolean {
        return element.label !== undefined && element.label.length > 0;
    }

    /**
     * Get the element type, handling optional wrapping
     */
    export function getElementType(element: TupleElement, context: TupleContext): Type {
        if (isOptionalElement(element)) {
            // Optional elements include undefined in their type
            return context.checker.getUnionType([element.type, context.undefinedType]);
        }
        return element.type;
    }

    /**
     * Get the array element type for a rest element
     */
    export function getRestElementArrayType(element: TupleElement): Type | undefined {
        if (!isRestElement(element) && !isVariadicElement(element)) {
            return undefined;
        }

        // The type should be an array type or tuple type
        const type = element.type;

        if (type.flags & TypeFlags.Object) {
            const objectType = type as ObjectType;
            // Check if it's an array type
            if (objectType.objectFlags & ObjectFlags.Reference) {
                const typeRef = type as TypeReference;
                const target = typeRef.target;
                if (target && (target as any).symbol?.name === "Array") {
                    // Return the type argument
                    return typeRef.typeArguments?.[0];
                }
            }
        }

        return type;
    }

    /**
     * Convert element to its display string
     */
    export function elementToString(element: TupleElement, typeToString: (type: Type) => string): string {
        let result = "";

        // Add label if present
        if (element.label) {
            result += element.label;

            if (isOptionalElement(element)) {
                result += "?";
            }

            result += ": ";
        } else if (isOptionalElement(element)) {
            result += "?: ";
        }

        // Add rest/variadic marker
        if (isRestElement(element) || isVariadicElement(element)) {
            result += "...";
        }

        result += typeToString(element.type);

        return result;
    }

    /**
     * Clone a tuple element
     */
    export function cloneElement(element: TupleElement): TupleElement {
        return {
            type: element.type,
            label: element.label,
            flags: element.flags,
        };
    }

    /**
     * Make an element optional
     */
    export function makeOptional(element: TupleElement): TupleElement {
        return {
            ...element,
            flags: (element.flags & ~TupleElementFlags.Required) | TupleElementFlags.Optional,
        };
    }

    /**
     * Make an element required
     */
    export function makeRequired(element: TupleElement): TupleElement {
        return {
            ...element,
            flags: (element.flags & ~TupleElementFlags.Optional) | TupleElementFlags.Required,
        };
    }

    /**
     * Remove the label from an element
     */
    export function removeLabel(element: TupleElement): TupleElement {
        const clone = cloneElement(element);
        delete clone.label;
        return clone;
    }

    /**
     * Add or change label on an element
     */
    export function withLabel(element: TupleElement, label: string): TupleElement {
        return {
            ...element,
            label,
        };
    }

    /**
     * Check if two elements are structurally equal
     */
    export function elementsEqual(
        a: TupleElement,
        b: TupleElement,
        context: TupleContext
    ): boolean {
        if (a.flags !== b.flags) {
            return false;
        }

        if (a.label !== b.label) {
            return false;
        }

        return context.checker.isTypeIdenticalTo(a.type, b.type);
    }

    /**
     * Get the effective length contribution of an element
     */
    export function getElementLengthContribution(element: TupleElement): {
        min: number;
        max: number | undefined;
    } {
        if (isRestElement(element)) {
            return { min: 0, max: undefined }; // Rest can be 0 to infinite
        }

        if (isVariadicElement(element)) {
            // Variadic depends on the spread type
            return { min: 0, max: undefined };
        }

        if (isOptionalElement(element)) {
            return { min: 0, max: 1 }; // Optional is 0 or 1
        }

        // Required
        return { min: 1, max: 1 };
    }

    /**
     * Validate tuple element constraints
     * - Optional elements cannot precede required elements
     * - Rest element must be last (or variadic elements after it)
     * - At most one rest element
     */
    export function validateElementOrder(elements: TupleElement[]): {
        valid: boolean;
        error?: string;
    } {
        let seenOptional = false;
        let seenRest = false;
        let restIndex = -1;

        for (let i = 0; i < elements.length; i++) {
            const element = elements[i];

            if (isRestElement(element)) {
                if (seenRest) {
                    return {
                        valid: false,
                        error: "A tuple type cannot have multiple rest elements.",
                    };
                }
                seenRest = true;
                restIndex = i;
            } else if (isVariadicElement(element)) {
                // Variadic elements can appear after rest
                continue;
            } else if (isOptionalElement(element)) {
                if (seenRest && i > restIndex) {
                    return {
                        valid: false,
                        error: "Optional elements cannot follow a rest element.",
                    };
                }
                seenOptional = true;
            } else if (isRequiredElement(element)) {
                if (seenOptional) {
                    return {
                        valid: false,
                        error: "A required element cannot follow an optional element.",
                    };
                }
                if (seenRest && i > restIndex) {
                    // Required elements after rest are allowed in variadic contexts
                }
            }
        }

        return { valid: true };
    }

    /**
     * Normalize elements to ensure valid ordering
     * Moves optional elements to the end, before any rest element
     */
    export function normalizeElements(elements: TupleElement[]): TupleElement[] {
        const required: TupleElement[] = [];
        const optional: TupleElement[] = [];
        let rest: TupleElement | undefined;
        const afterRest: TupleElement[] = [];

        let seenRest = false;

        for (const element of elements) {
            if (isRestElement(element)) {
                rest = element;
                seenRest = true;
            } else if (seenRest) {
                afterRest.push(element);
            } else if (isOptionalElement(element)) {
                optional.push(element);
            } else {
                required.push(element);
            }
        }

        const result = [...required, ...optional];
        if (rest) {
            result.push(rest);
        }
        result.push(...afterRest);

        return result;
    }

    /**
     * Get all labels from elements
     */
    export function getLabels(elements: TupleElement[]): (string | undefined)[] {
        return elements.map(e => e.label);
    }

    /**
     * Check if all elements have labels
     */
    export function allLabeled(elements: TupleElement[]): boolean {
        return elements.every(e => e.label !== undefined);
    }

    /**
     * Check if no elements have labels
     */
    export function noneLabeled(elements: TupleElement[]): boolean {
        return elements.every(e => e.label === undefined);
    }

    /**
     * Check if labels are mixed (some labeled, some not)
     */
    export function hassMixedLabels(elements: TupleElement[]): boolean {
        return !allLabeled(elements) && !noneLabeled(elements);
    }
}
