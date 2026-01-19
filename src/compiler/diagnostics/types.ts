/**
 * Diagnostics System - Type Definitions
 *
 * This module defines the core types for the TypeScript diagnostics system,
 * including diagnostic categories, severity levels, and related information structures.
 */

/* @internal */
namespace ts.diagnostics {
    /**
     * Diagnostic categories matching TypeScript's categories
     */
    export const enum DiagnosticSeverity {
        Error = 0,
        Warning = 1,
        Suggestion = 2,
        Message = 3,
    }

    /**
     * Represents a location in source code
     */
    export interface DiagnosticLocation {
        /** File path */
        file: string;
        /** Starting position (0-based offset) */
        start: number;
        /** Length of the diagnostic span */
        length: number;
        /** Line number (1-based) */
        line: number;
        /** Column number (1-based) */
        column: number;
    }

    /**
     * A related diagnostic location with additional context
     */
    export interface RelatedInformation {
        /** Location of the related information */
        location: DiagnosticLocation;
        /** Message describing the relation */
        message: string;
        /** Diagnostic code (optional) */
        code?: number;
    }

    /**
     * A single diagnostic entry
     */
    export interface Diagnostic {
        /** Unique diagnostic code */
        code: number;
        /** Severity of the diagnostic */
        severity: DiagnosticSeverity;
        /** The diagnostic message (already formatted) */
        message: string;
        /** Location where the diagnostic occurred */
        location?: DiagnosticLocation;
        /** Related information (for chained diagnostics) */
        relatedInformation?: RelatedInformation[];
        /** Category string for display (e.g., "error", "warning") */
        category: string;
        /** Source of the diagnostic (e.g., "TS" for TypeScript) */
        source: string;
    }

    /**
     * A diagnostic message template with placeholders
     */
    export interface DiagnosticTemplate {
        /** Unique diagnostic code */
        code: number;
        /** Severity of the diagnostic */
        severity: DiagnosticSeverity;
        /** Message template with {0}, {1}, etc. placeholders */
        messageTemplate: string;
        /** Category string */
        category: string;
    }

    /**
     * Options for formatting diagnostics
     */
    export interface FormatterOptions {
        /** Enable colored output */
        colors?: boolean;
        /** Show source code context */
        showContext?: boolean;
        /** Number of context lines before/after */
        contextLines?: number;
        /** Show related information */
        showRelated?: boolean;
        /** Pretty print with indentation */
        pretty?: boolean;
        /** Terminal width for line wrapping */
        terminalWidth?: number;
    }

    /**
     * A formatted diagnostic ready for output
     */
    export interface FormattedDiagnostic {
        /** The original diagnostic */
        diagnostic: Diagnostic;
        /** Formatted text output */
        text: string;
        /** ANSI-colored text (if colors enabled) */
        coloredText?: string;
    }

    /**
     * Statistics about collected diagnostics
     */
    export interface DiagnosticStats {
        /** Total number of diagnostics */
        total: number;
        /** Count by severity */
        errors: number;
        warnings: number;
        suggestions: number;
        messages: number;
        /** Diagnostics per file */
        byFile: Map<string, number>;
    }

    /**
     * Callback for diagnostic emission
     */
    export type DiagnosticCallback = (diagnostic: Diagnostic) => void;

    /**
     * Filter predicate for diagnostics
     */
    export type DiagnosticFilter = (diagnostic: Diagnostic) => boolean;
}
