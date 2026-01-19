/**
 * Template Literal Types - Module Index
 *
 * Comprehensive implementation of TypeScript's template literal types:
 *
 * - Template literal type parsing and representation
 * - Intrinsic string manipulation types (Uppercase, Lowercase, Capitalize, Uncapitalize)
 * - Template literal type inference from string literals
 * - Pattern matching for type narrowing
 * - Union distribution in template literal types
 *
 * @example
 * ```typescript
 * // Template literal type creation
 * type Greeting = `Hello, ${string}!`;
 *
 * // String manipulation
 * type Upper = Uppercase<"hello">; // "HELLO"
 * type Lower = Lowercase<"WORLD">; // "world"
 * type Cap = Capitalize<"foo">;    // "Foo"
 * type Uncap = Uncapitalize<"Bar">; // "bar"
 *
 * // Pattern matching
 * type GetName<T> = T extends `get${infer Name}` ? Name : never;
 * type N = GetName<"getName">; // "Name"
 * ```
 */

/* @internal */
namespace ts.templateLiterals {
    // Re-export all types
    export type {
        TemplateLiteralSpan,
        TemplateLiteralTypeInfo,
        PatternMatchResult,
        TemplateLiteralOptions,
        TemplateLiteralInferenceResult,
        TemplateLiteralContext,
        StringLiteralConstraint,
        NarrowingResult,
    };

    // Export IntrinsicTypeKind enum
    export { IntrinsicTypeKind };

    // Export string manipulation functions
    export {
        applyUppercase,
        applyLowercase,
        applyCapitalize,
        applyUncapitalize,
        applyStringTransformation,
        isIntrinsicStringType,
        getIntrinsicTypeKind,
        applyIntrinsicToStringLiteral,
        createIntrinsicTypeResolver,
        createUppercaseType,
        createLowercaseType,
        createCapitalizeType,
        createUncapitalizeType,
        intrinsicTypeMap,
        applyIntrinsicType,
    };

    // Export inference functions
    export {
        parseTemplateLiteralType,
        matchTemplateLiteralPattern,
        inferTemplateLiteralFromString,
        createTemplateLiteralType,
        canBeInterpolated,
        getLiteralStringValue,
        simplifyTemplateLiteralType,
        areTemplateLiteralTypesEquivalent,
    };

    // Export pattern matching functions
    export {
        narrowByTemplateLiteralPattern,
        extractTemplateParts,
        stringMatchesPattern,
        getTemplateLiteralStringValues,
        templateLiteralToRegex,
    };

    /**
     * Create a template literal context from a type checker
     */
    export function createTemplateLiteralContext(checker: TypeChecker): TemplateLiteralContext {
        return {
            checker,
            stringType: checker.getStringType(),
            numberType: checker.getNumberType(),
            bigintType: checker.getBigIntType(),
            booleanType: checker.getBooleanType(),
            nullType: checker.getNullType(),
            undefinedType: checker.getUndefinedType(),
            neverType: checker.getNeverType(),
            unknownType: checker.getUnknownType(),
        };
    }

    /**
     * Process a template literal type node and return the resulting type
     */
    export function processTemplateLiteralTypeNode(
        node: TemplateLiteralTypeNode,
        getTypeFromNode: (node: TypeNode) => Type,
        context: TemplateLiteralContext
    ): Type {
        const texts: string[] = [node.head.text];
        const types: Type[] = [];

        for (const span of node.templateSpans) {
            types.push(getTypeFromNode(span.type));
            texts.push(span.literal.text);
        }

        return createTemplateLiteralType(texts, types, context);
    }

    /**
     * Check if a type is a template literal type
     */
    export function isTemplateLiteralType(type: Type): type is TemplateLiteralType {
        return (type.flags & TypeFlags.TemplateLiteral) !== 0;
    }

    /**
     * Check if a type is a string literal type
     */
    export function isStringLiteralType(type: Type): type is StringLiteralType {
        return (type.flags & TypeFlags.StringLiteral) !== 0;
    }

    /**
     * Get the string value of a type for template literal purposes
     */
    export function getStringValueForTemplate(type: Type): string | undefined {
        return getLiteralStringValue(type);
    }

    /**
     * Evaluate a template literal type to a string literal if possible
     */
    export function evaluateTemplateLiteralType(
        type: TemplateLiteralType,
        context: TemplateLiteralContext
    ): Type {
        const info = parseTemplateLiteralType(type);

        // Check if all interpolations are literals
        if (info.isLiteral) {
            let result = info.head;
            for (let i = 0; i < info.types.length; i++) {
                const value = getLiteralStringValue(info.types[i]);
                if (value === undefined) {
                    return type; // Can't evaluate
                }
                result += value + info.texts[i + 1];
            }
            return context.checker.getStringLiteralType(result);
        }

        // Try to simplify
        const simplified = simplifyTemplateLiteralType(
            info.texts,
            info.types,
            context
        );

        if (simplified.types.length === 0) {
            return context.checker.getStringLiteralType(simplified.texts[0]);
        }

        if (
            simplified.texts.length !== info.texts.length ||
            simplified.types.length !== info.types.length
        ) {
            return context.checker.getTemplateLiteralType(simplified.texts, simplified.types);
        }

        return type;
    }

    /**
     * Check if a template literal type contains only string types
     */
    export function isWideTemplateLiteralType(type: TemplateLiteralType): boolean {
        const info = parseTemplateLiteralType(type);
        return info.types.every(t => t.flags & TypeFlags.String);
    }

    /**
     * Check if a template literal type can be narrowed to a specific string
     */
    export function canNarrowToString(
        template: TemplateLiteralType,
        str: string,
        context: TemplateLiteralContext
    ): boolean {
        const info = parseTemplateLiteralType(template);
        const result = matchTemplateLiteralPattern(str, info, context);
        return result.matched;
    }

    /**
     * Get the constraint for a template literal type
     * Returns the most general type that the template literal can match
     */
    export function getTemplateLiteralConstraint(
        type: TemplateLiteralType,
        context: TemplateLiteralContext
    ): Type {
        // If all parts are string literals, the constraint is the literal itself
        const evaluated = evaluateTemplateLiteralType(type, context);
        if (evaluated.flags & TypeFlags.StringLiteral) {
            return evaluated;
        }

        // Otherwise, constraint is string
        return context.stringType;
    }

    /**
     * Combine multiple template literal types
     */
    export function combineTemplateLiteralTypes(
        types: TemplateLiteralType[],
        context: TemplateLiteralContext
    ): Type {
        if (types.length === 0) {
            return context.neverType;
        }

        if (types.length === 1) {
            return types[0];
        }

        return context.checker.getUnionType(types);
    }

    /**
     * Create helpers for common template literal operations
     */
    export const TemplateHelpers = {
        /**
         * Create a prefix pattern type
         * `${prefix}${string}`
         */
        prefixPattern(prefix: string, context: TemplateLiteralContext): Type {
            return createTemplateLiteralType(
                [prefix, ""],
                [context.stringType],
                context
            );
        },

        /**
         * Create a suffix pattern type
         * `${string}${suffix}`
         */
        suffixPattern(suffix: string, context: TemplateLiteralContext): Type {
            return createTemplateLiteralType(
                ["", suffix],
                [context.stringType],
                context
            );
        },

        /**
         * Create a contains pattern type
         * `${string}${contains}${string}`
         */
        containsPattern(contains: string, context: TemplateLiteralContext): Type {
            return createTemplateLiteralType(
                ["", contains, ""],
                [context.stringType, context.stringType],
                context
            );
        },

        /**
         * Create a pattern with placeholders
         * e.g., createPattern(["GET /", "/"], [string, number])
         */
        createPattern(texts: string[], types: Type[], context: TemplateLiteralContext): Type {
            return createTemplateLiteralType(texts, types, context);
        },
    };
}
