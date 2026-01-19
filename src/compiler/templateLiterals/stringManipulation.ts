/**
 * Template Literal Types - Intrinsic String Manipulation
 *
 * Implements the intrinsic string manipulation types:
 * - Uppercase<S>: Convert string literal type to uppercase
 * - Lowercase<S>: Convert string literal type to lowercase
 * - Capitalize<S>: Capitalize the first character
 * - Uncapitalize<S>: Uncapitalize the first character
 */

/* @internal */
namespace ts.templateLiterals {
    /**
     * Apply uppercase transformation to a string
     */
    export function applyUppercase(str: string): string {
        return str.toUpperCase();
    }

    /**
     * Apply lowercase transformation to a string
     */
    export function applyLowercase(str: string): string {
        return str.toLowerCase();
    }

    /**
     * Apply capitalize transformation to a string
     * Capitalizes the first character, leaves rest unchanged
     */
    export function applyCapitalize(str: string): string {
        if (str.length === 0) return str;
        return str.charAt(0).toUpperCase() + str.slice(1);
    }

    /**
     * Apply uncapitalize transformation to a string
     * Lowercases the first character, leaves rest unchanged
     */
    export function applyUncapitalize(str: string): string {
        if (str.length === 0) return str;
        return str.charAt(0).toLowerCase() + str.slice(1);
    }

    /**
     * Apply an intrinsic string transformation to a string
     */
    export function applyStringTransformation(
        str: string,
        kind: IntrinsicTypeKind
    ): string {
        switch (kind) {
            case IntrinsicTypeKind.Uppercase:
                return applyUppercase(str);
            case IntrinsicTypeKind.Lowercase:
                return applyLowercase(str);
            case IntrinsicTypeKind.Capitalize:
                return applyCapitalize(str);
            case IntrinsicTypeKind.Uncapitalize:
                return applyUncapitalize(str);
            default:
                return str;
        }
    }

    /**
     * Check if a type is an intrinsic string manipulation type
     */
    export function isIntrinsicStringType(typeName: string): typeName is IntrinsicTypeKind {
        return (
            typeName === IntrinsicTypeKind.Uppercase ||
            typeName === IntrinsicTypeKind.Lowercase ||
            typeName === IntrinsicTypeKind.Capitalize ||
            typeName === IntrinsicTypeKind.Uncapitalize
        );
    }

    /**
     * Get the intrinsic type kind from a type alias name
     */
    export function getIntrinsicTypeKind(name: string): IntrinsicTypeKind | undefined {
        if (isIntrinsicStringType(name)) {
            return name;
        }
        return undefined;
    }

    /**
     * Apply intrinsic string manipulation to a string literal type
     *
     * Returns the transformed string or undefined if the type is not a string literal
     */
    export function applyIntrinsicToStringLiteral(
        value: string,
        intrinsic: IntrinsicTypeKind
    ): string {
        return applyStringTransformation(value, intrinsic);
    }

    /**
     * Create an intrinsic type resolver
     */
    export function createIntrinsicTypeResolver(
        context: TemplateLiteralContext
    ): (type: Type, intrinsic: IntrinsicTypeKind) => Type {
        const { checker, stringType, neverType } = context;

        return function resolveIntrinsicType(type: Type, intrinsic: IntrinsicTypeKind): Type {
            // Handle string literal types
            if (type.flags & TypeFlags.StringLiteral) {
                const stringLiteralType = type as StringLiteralType;
                const transformed = applyStringTransformation(stringLiteralType.value, intrinsic);
                return checker.getStringLiteralType(transformed);
            }

            // Handle union types - distribute over union
            if (type.flags & TypeFlags.Union) {
                const unionType = type as UnionType;
                const transformedTypes = unionType.types.map(t =>
                    resolveIntrinsicType(t, intrinsic)
                );
                return checker.getUnionType(transformedTypes);
            }

            // Handle template literal types
            if (type.flags & TypeFlags.TemplateLiteral) {
                return resolveIntrinsicForTemplateLiteral(type as TemplateLiteralType, intrinsic, context);
            }

            // Handle string type - returns string (no transformation possible)
            if (type.flags & TypeFlags.String) {
                return stringType;
            }

            // Handle never type
            if (type.flags & TypeFlags.Never) {
                return neverType;
            }

            // For other types, return the original type
            // (intrinsic transformations only apply to strings)
            return type;
        };
    }

    /**
     * Resolve intrinsic type for a template literal type
     */
    function resolveIntrinsicForTemplateLiteral(
        type: TemplateLiteralType,
        intrinsic: IntrinsicTypeKind,
        context: TemplateLiteralContext
    ): Type {
        const { checker } = context;

        // If all parts are string literals, we can compute the result
        const texts = type.texts;
        const types = type.types;

        // Check if all interpolated types are string literals
        const allLiterals = types.every(t => t.flags & TypeFlags.StringLiteral);

        if (allLiterals) {
            // Build the full string and transform it
            let fullString = texts[0];
            for (let i = 0; i < types.length; i++) {
                const literalType = types[i] as StringLiteralType;
                fullString += literalType.value + texts[i + 1];
            }

            const transformed = applyStringTransformation(fullString, intrinsic);
            return checker.getStringLiteralType(transformed);
        }

        // For non-literal template parts, transform what we can
        switch (intrinsic) {
            case IntrinsicTypeKind.Uppercase:
            case IntrinsicTypeKind.Lowercase: {
                // Transform the literal texts
                const newTexts = texts.map(t => applyStringTransformation(t, intrinsic));

                // The types also need to be wrapped in the intrinsic
                // For now, return a template literal with transformed texts
                return checker.getTemplateLiteralType(newTexts, types);
            }

            case IntrinsicTypeKind.Capitalize:
            case IntrinsicTypeKind.Uncapitalize: {
                // These only affect the first character
                if (texts[0].length > 0) {
                    // Transform the first text
                    const newTexts = [...texts];
                    newTexts[0] = applyStringTransformation(texts[0], intrinsic);
                    return checker.getTemplateLiteralType(newTexts, types);
                } else if (types.length > 0) {
                    // First part is a type - apply intrinsic to it
                    const resolver = createIntrinsicTypeResolver(context);
                    const firstTransformed = resolver(types[0], intrinsic);
                    const newTypes = [firstTransformed, ...types.slice(1)];
                    return checker.getTemplateLiteralType(texts, newTypes);
                }

                return type;
            }

            default:
                return type;
        }
    }

    /**
     * Create the Uppercase<T> type application
     */
    export function createUppercaseType(
        type: Type,
        context: TemplateLiteralContext
    ): Type {
        const resolver = createIntrinsicTypeResolver(context);
        return resolver(type, IntrinsicTypeKind.Uppercase);
    }

    /**
     * Create the Lowercase<T> type application
     */
    export function createLowercaseType(
        type: Type,
        context: TemplateLiteralContext
    ): Type {
        const resolver = createIntrinsicTypeResolver(context);
        return resolver(type, IntrinsicTypeKind.Lowercase);
    }

    /**
     * Create the Capitalize<T> type application
     */
    export function createCapitalizeType(
        type: Type,
        context: TemplateLiteralContext
    ): Type {
        const resolver = createIntrinsicTypeResolver(context);
        return resolver(type, IntrinsicTypeKind.Capitalize);
    }

    /**
     * Create the Uncapitalize<T> type application
     */
    export function createUncapitalizeType(
        type: Type,
        context: TemplateLiteralContext
    ): Type {
        const resolver = createIntrinsicTypeResolver(context);
        return resolver(type, IntrinsicTypeKind.Uncapitalize);
    }

    /**
     * Map of intrinsic type names to their implementations
     */
    export const intrinsicTypeMap: Record<
        IntrinsicTypeKind,
        (type: Type, context: TemplateLiteralContext) => Type
    > = {
        [IntrinsicTypeKind.Uppercase]: createUppercaseType,
        [IntrinsicTypeKind.Lowercase]: createLowercaseType,
        [IntrinsicTypeKind.Capitalize]: createCapitalizeType,
        [IntrinsicTypeKind.Uncapitalize]: createUncapitalizeType,
    };

    /**
     * Apply an intrinsic type by name
     */
    export function applyIntrinsicType(
        name: string,
        type: Type,
        context: TemplateLiteralContext
    ): Type | undefined {
        const kind = getIntrinsicTypeKind(name);
        if (kind) {
            return intrinsicTypeMap[kind](type, context);
        }
        return undefined;
    }
}
