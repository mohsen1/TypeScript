/**
 * Template Literal Types - Pattern Matching for Type Narrowing
 *
 * Implements pattern matching against template literal types for:
 * - Type narrowing in conditionals
 * - Extracting parts of string literals
 * - Validating string formats
 */

/* @internal */
namespace ts.templateLiterals {
    /**
     * Result of narrowing a type using a template literal pattern
     */
    export interface NarrowingResult {
        /** The narrowed type (or never if no match) */
        narrowedType: Type;
        /** Types extracted from the pattern */
        extractedTypes: Map<string, Type>;
        /** Whether the narrowing was successful */
        success: boolean;
    }

    /**
     * Narrow a string type using a template literal pattern
     */
    export function narrowByTemplateLiteralPattern(
        type: Type,
        pattern: TemplateLiteralType,
        context: TemplateLiteralContext
    ): NarrowingResult {
        const { checker, neverType, stringType } = context;
        const extractedTypes = new Map<string, Type>();

        // Handle string literal types
        if (type.flags & TypeFlags.StringLiteral) {
            const literalType = type as StringLiteralType;
            const patternInfo = parseTemplateLiteralType(pattern);
            const matchResult = matchTemplateLiteralPattern(literalType.value, patternInfo, context);

            if (matchResult.matched) {
                return {
                    narrowedType: type,
                    extractedTypes: matchResult.captures,
                    success: true,
                };
            }

            return {
                narrowedType: neverType,
                extractedTypes,
                success: false,
            };
        }

        // Handle union types - narrow each member
        if (type.flags & TypeFlags.Union) {
            const unionType = type as UnionType;
            const matchingTypes: Type[] = [];
            const combinedExtracts = new Map<string, Type[]>();

            for (const member of unionType.types) {
                const result = narrowByTemplateLiteralPattern(member, pattern, context);
                if (result.success) {
                    matchingTypes.push(result.narrowedType);

                    // Combine extracted types
                    for (const [key, value] of result.extractedTypes) {
                        const existing = combinedExtracts.get(key);
                        if (existing) {
                            existing.push(value);
                        } else {
                            combinedExtracts.set(key, [value]);
                        }
                    }
                }
            }

            if (matchingTypes.length === 0) {
                return {
                    narrowedType: neverType,
                    extractedTypes,
                    success: false,
                };
            }

            // Create union of extracted types
            for (const [key, values] of combinedExtracts) {
                extractedTypes.set(key, checker.getUnionType(values));
            }

            return {
                narrowedType: checker.getUnionType(matchingTypes),
                extractedTypes,
                success: true,
            };
        }

        // Handle generic string type
        if (type.flags & TypeFlags.String) {
            // String matches any template pattern, but we can't extract specific values
            const patternInfo = parseTemplateLiteralType(pattern);

            // Set extracted types to appropriate constraints
            for (const span of patternInfo.spans) {
                if (span.type.flags & TypeFlags.TypeParameter) {
                    const typeParam = span.type as TypeParameter;
                    const name = typeParam.symbol?.name ?? "T";

                    // Extracted type is constrained to string
                    extractedTypes.set(name, stringType);
                }
            }

            return {
                narrowedType: pattern,
                extractedTypes,
                success: true,
            };
        }

        // Handle template literal types
        if (type.flags & TypeFlags.TemplateLiteral) {
            const sourceTemplate = type as TemplateLiteralType;
            const targetPattern = pattern;

            const compatibility = checkTemplateLiteralCompatibility(
                sourceTemplate,
                targetPattern,
                context
            );

            if (compatibility.compatible) {
                return {
                    narrowedType: type,
                    extractedTypes: compatibility.captures,
                    success: true,
                };
            }

            return {
                narrowedType: neverType,
                extractedTypes,
                success: false,
            };
        }

        // Other types don't match template literal patterns
        return {
            narrowedType: neverType,
            extractedTypes,
            success: false,
        };
    }

    /**
     * Check compatibility between two template literal types
     */
    interface CompatibilityResult {
        compatible: boolean;
        captures: Map<string, Type>;
    }

    function checkTemplateLiteralCompatibility(
        source: TemplateLiteralType,
        target: TemplateLiteralType,
        context: TemplateLiteralContext
    ): CompatibilityResult {
        const sourceInfo = parseTemplateLiteralType(source);
        const targetInfo = parseTemplateLiteralType(target);
        const captures = new Map<string, Type>();

        // Simple structural check
        // For full compatibility, we'd need more sophisticated analysis

        // If source is more specific (has literals where target has type params)
        // then source may be assignable to target

        // Check if source head starts with target head
        if (!sourceInfo.head.startsWith(targetInfo.head)) {
            // Target head must be prefix of source head for compatibility
            if (!targetInfo.head.startsWith(sourceInfo.head)) {
                return { compatible: false, captures };
            }
        }

        // Basic compatibility: same structure
        if (sourceInfo.spans.length !== targetInfo.spans.length) {
            // Different number of interpolations
            // Could still be compatible if source is more specific
            return { compatible: true, captures }; // Assume compatible, let checker verify
        }

        // Check each span
        for (let i = 0; i < sourceInfo.spans.length; i++) {
            const sourceSpan = sourceInfo.spans[i];
            const targetSpan = targetInfo.spans[i];

            // Check if trailing text is compatible
            if (sourceSpan.text !== targetSpan.text) {
                return { compatible: false, captures };
            }

            // Check type compatibility
            if (targetSpan.type.flags & TypeFlags.TypeParameter) {
                // Target has a type parameter - capture the source type
                const typeParam = targetSpan.type as TypeParameter;
                const name = typeParam.symbol?.name ?? `T${i}`;
                captures.set(name, sourceSpan.type);
            }
        }

        return { compatible: true, captures };
    }

    /**
     * Extract template literal type parts from a string using a pattern
     *
     * Given pattern `${infer A}-${infer B}` and string "hello-world"
     * Returns { A: "hello", B: "world" }
     */
    export function extractTemplateParts(
        str: string,
        pattern: TemplateLiteralType,
        context: TemplateLiteralContext
    ): Map<string, string> | undefined {
        const patternInfo = parseTemplateLiteralType(pattern);
        const result = new Map<string, string>();

        // Must start with head
        if (!str.startsWith(patternInfo.head)) {
            return undefined;
        }

        let remaining = str.slice(patternInfo.head.length);
        let spanIndex = 0;

        while (spanIndex < patternInfo.spans.length) {
            const span = patternInfo.spans[spanIndex];
            const isLast = spanIndex === patternInfo.spans.length - 1;

            if (isLast) {
                // Last span - remaining must end with the text
                if (!remaining.endsWith(span.text)) {
                    return undefined;
                }

                const captured = remaining.slice(0, remaining.length - span.text.length);

                // Store capture
                if (span.type.flags & TypeFlags.TypeParameter) {
                    const typeParam = span.type as TypeParameter;
                    const name = typeParam.symbol?.name ?? `capture${spanIndex}`;
                    result.set(name, captured);
                }
            } else {
                // Find the next text
                const nextIndex = remaining.indexOf(span.text);
                if (nextIndex === -1) {
                    return undefined;
                }

                const captured = remaining.slice(0, nextIndex);

                // Store capture
                if (span.type.flags & TypeFlags.TypeParameter) {
                    const typeParam = span.type as TypeParameter;
                    const name = typeParam.symbol?.name ?? `capture${spanIndex}`;
                    result.set(name, captured);
                }

                remaining = remaining.slice(nextIndex + span.text.length);
            }

            spanIndex++;
        }

        // If no spans, remaining must be empty
        if (patternInfo.spans.length === 0 && remaining.length > 0) {
            return undefined;
        }

        return result;
    }

    /**
     * Check if a string literal matches a template literal pattern
     */
    export function stringMatchesPattern(
        str: string,
        pattern: TemplateLiteralType,
        context: TemplateLiteralContext
    ): boolean {
        const patternInfo = parseTemplateLiteralType(pattern);
        const matchResult = matchTemplateLiteralPattern(str, patternInfo, context);
        return matchResult.matched;
    }

    /**
     * Get all possible string values from a template literal type
     * Returns undefined if the type contains non-literal interpolations
     */
    export function getTemplateLiteralStringValues(
        type: TemplateLiteralType,
        maxValues: number = 1000
    ): string[] | undefined {
        const info = parseTemplateLiteralType(type);

        // Collect all possible values for each interpolation slot
        const slotValues: string[][] = [];

        for (const span of info.spans) {
            const values = getTypeStringValues(span.type);
            if (values === undefined) {
                return undefined;
            }
            slotValues.push(values);
        }

        // Calculate total combinations
        let totalCombinations = 1;
        for (const values of slotValues) {
            totalCombinations *= values.length;
            if (totalCombinations > maxValues) {
                return undefined;
            }
        }

        // Generate all combinations
        const results: string[] = [];
        const indices = new Array(slotValues.length).fill(0);

        while (true) {
            // Build a string from current indices
            let str = info.head;
            for (let i = 0; i < slotValues.length; i++) {
                str += slotValues[i][indices[i]] + info.spans[i].text;
            }
            results.push(str);

            // Increment indices
            let carry = true;
            for (let i = slotValues.length - 1; i >= 0 && carry; i--) {
                indices[i]++;
                if (indices[i] >= slotValues[i].length) {
                    indices[i] = 0;
                } else {
                    carry = false;
                }
            }

            if (carry) break;
        }

        return results;
    }

    /**
     * Get string values from a type
     */
    function getTypeStringValues(type: Type): string[] | undefined {
        if (type.flags & TypeFlags.StringLiteral) {
            return [(type as StringLiteralType).value];
        }

        if (type.flags & TypeFlags.NumberLiteral) {
            return [String((type as NumberLiteralType).value)];
        }

        if (type.flags & TypeFlags.BigIntLiteral) {
            const bigintType = type as BigIntLiteralType;
            const value = bigintType.value.negative
                ? `-${bigintType.value.base10Value}`
                : bigintType.value.base10Value;
            return [value];
        }

        if (type.flags & TypeFlags.BooleanLiteral) {
            return [(type as any).intrinsicName];
        }

        if (type.flags & TypeFlags.Null) {
            return ["null"];
        }

        if (type.flags & TypeFlags.Undefined) {
            return ["undefined"];
        }

        if (type.flags & TypeFlags.Union) {
            const unionType = type as UnionType;
            const allValues: string[] = [];

            for (const member of unionType.types) {
                const values = getTypeStringValues(member);
                if (values === undefined) {
                    return undefined;
                }
                allValues.push(...values);
            }

            return allValues;
        }

        // Non-literal types can't be enumerated
        return undefined;
    }

    /**
     * Create a regular expression from a template literal pattern
     * Useful for runtime validation
     */
    export function templateLiteralToRegex(
        type: TemplateLiteralType,
        context: TemplateLiteralContext
    ): RegExp | undefined {
        const info = parseTemplateLiteralType(type);

        let pattern = "^" + escapeRegex(info.head);

        for (const span of info.spans) {
            // Get pattern for the type
            const typePattern = getTypeRegexPattern(span.type);
            if (typePattern === undefined) {
                return undefined;
            }

            pattern += `(${typePattern})` + escapeRegex(span.text);
        }

        pattern += "$";

        try {
            return new RegExp(pattern);
        } catch {
            return undefined;
        }
    }

    /**
     * Escape a string for use in a regular expression
     */
    function escapeRegex(str: string): string {
        return str.replace(/[.*+?^${}()|[\]\\]/g, "\\$&");
    }

    /**
     * Get regex pattern for a type
     */
    function getTypeRegexPattern(type: Type): string | undefined {
        if (type.flags & TypeFlags.String) {
            return ".*";
        }

        if (type.flags & TypeFlags.StringLiteral) {
            return escapeRegex((type as StringLiteralType).value);
        }

        if (type.flags & TypeFlags.Number) {
            return "-?\\d+(?:\\.\\d+)?";
        }

        if (type.flags & TypeFlags.NumberLiteral) {
            return escapeRegex(String((type as NumberLiteralType).value));
        }

        if (type.flags & TypeFlags.BigInt) {
            return "-?\\d+";
        }

        if (type.flags & TypeFlags.Boolean || type.flags & TypeFlags.BooleanLiteral) {
            if (type.flags & TypeFlags.BooleanLiteral) {
                return escapeRegex((type as any).intrinsicName);
            }
            return "true|false";
        }

        if (type.flags & TypeFlags.Union) {
            const unionType = type as UnionType;
            const patterns: string[] = [];

            for (const member of unionType.types) {
                const memberPattern = getTypeRegexPattern(member);
                if (memberPattern === undefined) {
                    return undefined;
                }
                patterns.push(memberPattern);
            }

            return patterns.join("|");
        }

        // Type parameter or other - can't create specific pattern
        if (type.flags & TypeFlags.TypeParameter) {
            return ".*"; // Match anything for type parameters
        }

        return undefined;
    }
}
