/**
 * Template Literal Types - Type Inference
 *
 * Implements template literal type inference including:
 * - Inferring template literal types from string literals
 * - Pattern matching and type extraction
 * - Inference constraints for template patterns
 */

/* @internal */
namespace ts.templateLiterals {
    /** Maximum union size to prevent combinatorial explosion */
    const MAX_TEMPLATE_UNION_SIZE = 100000;

    /**
     * Parse a template literal type into its constituent parts
     */
    export function parseTemplateLiteralType(type: TemplateLiteralType): TemplateLiteralTypeInfo {
        const texts = type.texts;
        const types = type.types;

        const spans: TemplateLiteralSpan[] = [];
        for (let i = 0; i < types.length; i++) {
            spans.push({
                type: types[i],
                text: texts[i + 1],
            });
        }

        const isLiteral = types.every(t =>
            (t.flags & TypeFlags.StringLiteral) !== 0 ||
            (t.flags & TypeFlags.NumberLiteral) !== 0 ||
            (t.flags & TypeFlags.BigIntLiteral) !== 0
        );

        return {
            head: texts[0],
            spans,
            isLiteral,
            texts: [...texts],
            types: [...types],
        };
    }

    /**
     * Check if a string matches a template literal pattern
     */
    export function matchTemplateLiteralPattern(
        str: string,
        template: TemplateLiteralTypeInfo,
        context: TemplateLiteralContext
    ): PatternMatchResult {
        const captures = new Map<string, Type>();

        // Must start with the head
        if (!str.startsWith(template.head)) {
            return { matched: false, captures };
        }

        let remaining = str.slice(template.head.length);

        for (let i = 0; i < template.spans.length; i++) {
            const span = template.spans[i];
            const nextText = span.text;

            if (i === template.spans.length - 1) {
                // Last span - must end with the text
                if (!remaining.endsWith(nextText)) {
                    return { matched: false, captures };
                }

                const capturedValue = remaining.slice(0, remaining.length - nextText.length);

                // Validate and capture
                const matchResult = matchTypeAgainstString(span.type, capturedValue, context);
                if (!matchResult.matched) {
                    return { matched: false, captures };
                }

                for (const [key, value] of matchResult.captures) {
                    captures.set(key, value);
                }
            } else {
                // Find the next literal text
                const nextIndex = remaining.indexOf(nextText);
                if (nextIndex === -1) {
                    return { matched: false, captures };
                }

                const capturedValue = remaining.slice(0, nextIndex);

                // Validate and capture
                const matchResult = matchTypeAgainstString(span.type, capturedValue, context);
                if (!matchResult.matched) {
                    return { matched: false, captures };
                }

                for (const [key, value] of matchResult.captures) {
                    captures.set(key, value);
                }

                remaining = remaining.slice(nextIndex + nextText.length);
            }
        }

        // If no spans, remaining must be empty
        if (template.spans.length === 0 && remaining.length > 0) {
            return { matched: false, captures };
        }

        return { matched: true, captures };
    }

    /**
     * Match a type against a captured string value
     */
    function matchTypeAgainstString(
        type: Type,
        value: string,
        context: TemplateLiteralContext
    ): PatternMatchResult {
        const captures = new Map<string, Type>();

        // String type matches any string
        if (type.flags & TypeFlags.String) {
            return { matched: true, captures };
        }

        // String literal must match exactly
        if (type.flags & TypeFlags.StringLiteral) {
            const literalType = type as StringLiteralType;
            return { matched: literalType.value === value, captures };
        }

        // Number type - value must be a valid number
        if (type.flags & TypeFlags.Number) {
            const num = parseFloat(value);
            return { matched: !isNaN(num) && String(num) === value, captures };
        }

        // Number literal must match
        if (type.flags & TypeFlags.NumberLiteral) {
            const literalType = type as NumberLiteralType;
            return { matched: String(literalType.value) === value, captures };
        }

        // Bigint type - value must be a valid bigint
        if (type.flags & TypeFlags.BigInt) {
            try {
                BigInt(value);
                return { matched: true, captures };
            } catch {
                return { matched: false, captures };
            }
        }

        // Boolean type - must be "true" or "false"
        if (type.flags & TypeFlags.Boolean || type.flags & TypeFlags.BooleanLiteral) {
            return { matched: value === "true" || value === "false", captures };
        }

        // Union type - match any member
        if (type.flags & TypeFlags.Union) {
            const unionType = type as UnionType;
            for (const member of unionType.types) {
                const result = matchTypeAgainstString(member, value, context);
                if (result.matched) {
                    return result;
                }
            }
            return { matched: false, captures };
        }

        // Null/undefined match their string representations
        if (type.flags & TypeFlags.Null) {
            return { matched: value === "null", captures };
        }

        if (type.flags & TypeFlags.Undefined) {
            return { matched: value === "undefined", captures };
        }

        // Template literal type - recursive match
        if (type.flags & TypeFlags.TemplateLiteral) {
            const templateType = type as TemplateLiteralType;
            const info = parseTemplateLiteralType(templateType);
            return matchTemplateLiteralPattern(value, info, context);
        }

        // Type parameter - capture the value
        if (type.flags & TypeFlags.TypeParameter) {
            const typeParam = type as TypeParameter;
            const name = typeParam.symbol?.name ?? "T";
            captures.set(name, context.checker.getStringLiteralType(value));
            return { matched: true, captures };
        }

        // Default: can't match
        return { matched: false, captures };
    }

    /**
     * Infer the template literal type from a string literal
     */
    export function inferTemplateLiteralFromString(
        str: string,
        template: TemplateLiteralType,
        context: TemplateLiteralContext
    ): TemplateLiteralInferenceResult {
        const info = parseTemplateLiteralType(template);
        const matchResult = matchTemplateLiteralPattern(str, info, context);

        if (!matchResult.matched) {
            return {
                type: context.neverType,
                success: false,
                error: `String '${str}' does not match template pattern`,
            };
        }

        return {
            type: context.checker.getStringLiteralType(str),
            success: true,
        };
    }

    /**
     * Create a template literal type from texts and types
     */
    export function createTemplateLiteralType(
        texts: readonly string[],
        types: readonly Type[],
        context: TemplateLiteralContext
    ): Type {
        const { checker, stringType } = context;

        // If no interpolations, return a string literal
        if (types.length === 0) {
            return checker.getStringLiteralType(texts[0]);
        }

        // If all interpolations are string literals, concatenate
        if (types.every(t => t.flags & TypeFlags.StringLiteral)) {
            let result = texts[0];
            for (let i = 0; i < types.length; i++) {
                const literalType = types[i] as StringLiteralType;
                result += literalType.value + texts[i + 1];
            }
            return checker.getStringLiteralType(result);
        }

        // If all interpolations are number literals, concatenate
        if (types.every(t => t.flags & TypeFlags.NumberLiteral)) {
            let result = texts[0];
            for (let i = 0; i < types.length; i++) {
                const literalType = types[i] as NumberLiteralType;
                result += String(literalType.value) + texts[i + 1];
            }
            return checker.getStringLiteralType(result);
        }

        // Check for unions that can be distributed
        const hasUnions = types.some(t => t.flags & TypeFlags.Union);
        if (hasUnions) {
            return distributeTemplateLiteralUnions(texts, types, context);
        }

        // Create the template literal type
        return checker.getTemplateLiteralType(texts, types);
    }

    /**
     * Distribute unions in template literal types
     * `a${X | Y}b` -> `a${X}b` | `a${Y}b`
     */
    function distributeTemplateLiteralUnions(
        texts: readonly string[],
        types: readonly Type[],
        context: TemplateLiteralContext
    ): Type {
        const { checker, neverType } = context;

        // Calculate total combinations
        let totalCombinations = 1;
        const unionSizes: number[] = [];

        for (const type of types) {
            if (type.flags & TypeFlags.Union) {
                const unionType = type as UnionType;
                unionSizes.push(unionType.types.length);
                totalCombinations *= unionType.types.length;
            } else {
                unionSizes.push(1);
            }
        }

        // Prevent combinatorial explosion
        if (totalCombinations > MAX_TEMPLATE_UNION_SIZE) {
            // Return a template literal type without distribution
            return checker.getTemplateLiteralType(texts, types);
        }

        // Generate all combinations
        const results: Type[] = [];
        const indices = new Array(types.length).fill(0);

        while (true) {
            // Create a combination
            const combinedTypes: Type[] = [];
            for (let i = 0; i < types.length; i++) {
                const type = types[i];
                if (type.flags & TypeFlags.Union) {
                    const unionType = type as UnionType;
                    combinedTypes.push(unionType.types[indices[i]]);
                } else {
                    combinedTypes.push(type);
                }
            }

            // Create the template literal for this combination
            const result = createTemplateLiteralType(texts, combinedTypes, context);
            if (!(result.flags & TypeFlags.Never)) {
                results.push(result);
            }

            // Increment indices
            let carry = true;
            for (let i = types.length - 1; i >= 0 && carry; i--) {
                indices[i]++;
                if (indices[i] >= unionSizes[i]) {
                    indices[i] = 0;
                } else {
                    carry = false;
                }
            }

            if (carry) break;
        }

        if (results.length === 0) {
            return neverType;
        }

        if (results.length === 1) {
            return results[0];
        }

        return checker.getUnionType(results);
    }

    /**
     * Check if a type can be interpolated in a template literal
     */
    export function canBeInterpolated(type: Type): boolean {
        const flags = type.flags;

        // Directly interpolatable types
        if (
            flags & TypeFlags.StringLike ||
            flags & TypeFlags.NumberLike ||
            flags & TypeFlags.BigIntLike ||
            flags & TypeFlags.BooleanLike ||
            flags & TypeFlags.Null ||
            flags & TypeFlags.Undefined
        ) {
            return true;
        }

        // Union of interpolatable types
        if (flags & TypeFlags.Union) {
            const unionType = type as UnionType;
            return unionType.types.every(canBeInterpolated);
        }

        // Template literal types can be interpolated
        if (flags & TypeFlags.TemplateLiteral) {
            return true;
        }

        return false;
    }

    /**
     * Get the string representation of a literal type for interpolation
     */
    export function getLiteralStringValue(type: Type): string | undefined {
        if (type.flags & TypeFlags.StringLiteral) {
            return (type as StringLiteralType).value;
        }

        if (type.flags & TypeFlags.NumberLiteral) {
            return String((type as NumberLiteralType).value);
        }

        if (type.flags & TypeFlags.BigIntLiteral) {
            const bigintType = type as BigIntLiteralType;
            return bigintType.value.negative ? `-${bigintType.value.base10Value}` : bigintType.value.base10Value;
        }

        if (type.flags & TypeFlags.BooleanLiteral) {
            // intrinsicName is "true" or "false"
            return (type as any).intrinsicName;
        }

        if (type.flags & TypeFlags.Null) {
            return "null";
        }

        if (type.flags & TypeFlags.Undefined) {
            return "undefined";
        }

        return undefined;
    }

    /**
     * Simplify a template literal type
     * Combines adjacent string literals and removes empty strings
     */
    export function simplifyTemplateLiteralType(
        texts: string[],
        types: Type[],
        context: TemplateLiteralContext
    ): { texts: string[]; types: Type[] } {
        const newTexts: string[] = [texts[0]];
        const newTypes: Type[] = [];

        for (let i = 0; i < types.length; i++) {
            const type = types[i];
            const literalValue = getLiteralStringValue(type);

            if (literalValue !== undefined) {
                // Merge the literal into the preceding text
                newTexts[newTexts.length - 1] += literalValue + texts[i + 1];
            } else {
                newTypes.push(type);
                newTexts.push(texts[i + 1]);
            }
        }

        return { texts: newTexts, types: newTypes };
    }

    /**
     * Check if two template literal types are equivalent
     */
    export function areTemplateLiteralTypesEquivalent(
        a: TemplateLiteralType,
        b: TemplateLiteralType,
        context: TemplateLiteralContext
    ): boolean {
        const aInfo = parseTemplateLiteralType(a);
        const bInfo = parseTemplateLiteralType(b);

        // Check texts
        if (aInfo.texts.length !== bInfo.texts.length) {
            return false;
        }

        for (let i = 0; i < aInfo.texts.length; i++) {
            if (aInfo.texts[i] !== bInfo.texts[i]) {
                return false;
            }
        }

        // Check types
        if (aInfo.types.length !== bInfo.types.length) {
            return false;
        }

        const { checker } = context;
        for (let i = 0; i < aInfo.types.length; i++) {
            if (!checker.isTypeIdenticalTo(aInfo.types[i], bInfo.types[i])) {
                return false;
            }
        }

        return true;
    }
}
