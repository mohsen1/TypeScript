/**
 * Template Literal Types - Type Definitions
 *
 * This module defines types for template literal type operations,
 * string manipulation, and pattern matching.
 */

/* @internal */
namespace ts.templateLiterals {
    /**
     * Represents a span in a template literal type
     * Each span has a type that gets interpolated and text following it
     */
    export interface TemplateLiteralSpan {
        /** The type to interpolate */
        type: Type;
        /** The literal text following the interpolation */
        text: string;
    }

    /**
     * Represents a template literal type like `hello ${string} world`
     */
    export interface TemplateLiteralTypeInfo {
        /** The leading text before any interpolation */
        head: string;
        /** The spans with types and following text */
        spans: TemplateLiteralSpan[];
        /** Whether this template literal contains only literal types */
        isLiteral: boolean;
        /** The texts array for compatibility with TS API */
        texts: string[];
        /** The types array for compatibility with TS API */
        types: Type[];
    }

    /**
     * Intrinsic string manipulation type kinds
     */
    export const enum IntrinsicTypeKind {
        Uppercase = "Uppercase",
        Lowercase = "Lowercase",
        Capitalize = "Capitalize",
        Uncapitalize = "Uncapitalize",
    }

    /**
     * Result of pattern matching a string against a template literal type
     */
    export interface PatternMatchResult {
        /** Whether the match succeeded */
        matched: boolean;
        /** Captured type arguments from the match */
        captures: Map<string, Type>;
        /** Remaining string after partial match (if any) */
        remainder?: string;
    }

    /**
     * Options for template literal type operations
     */
    export interface TemplateLiteralOptions {
        /** Maximum number of union members to generate */
        maxUnionSize?: number;
        /** Whether to distribute over unions */
        distribute?: boolean;
        /** Whether to simplify the result */
        simplify?: boolean;
    }

    /**
     * Result of inferring a template literal type
     */
    export interface TemplateLiteralInferenceResult {
        /** The inferred type */
        type: Type;
        /** Whether inference was successful */
        success: boolean;
        /** Error message if inference failed */
        error?: string;
    }

    /**
     * Context for template literal type operations
     */
    export interface TemplateLiteralContext {
        /** Type checker for type operations */
        checker: TypeChecker;
        /** String type */
        stringType: Type;
        /** Number type */
        numberType: Type;
        /** Bigint type */
        bigintType: Type;
        /** Boolean type */
        booleanType: Type;
        /** Null type */
        nullType: Type;
        /** Undefined type */
        undefinedType: Type;
        /** Never type */
        neverType: Type;
        /** Unknown type */
        unknownType: Type;
    }

    /**
     * String literal type constraints
     */
    export interface StringLiteralConstraint {
        /** Pattern that strings must match */
        pattern?: RegExp;
        /** Prefix that strings must start with */
        prefix?: string;
        /** Suffix that strings must end with */
        suffix?: string;
        /** Exact values allowed */
        values?: string[];
    }

    /**
     * Template literal type node (for AST representation)
     */
    export interface TemplateLiteralTypeNode {
        kind: SyntaxKind.TemplateLiteralType;
        head: TemplateHead;
        templateSpans: readonly TemplateLiteralTypeSpan[];
    }

    /**
     * Template literal type span node
     */
    export interface TemplateLiteralTypeSpan {
        kind: SyntaxKind.TemplateLiteralTypeSpan;
        type: TypeNode;
        literal: TemplateMiddle | TemplateTail;
    }
}
