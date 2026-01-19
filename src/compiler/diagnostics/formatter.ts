/**
 * Diagnostics System - Pretty Formatter
 *
 * This module provides formatting capabilities for diagnostics output,
 * including ANSI color support and source code context display.
 */

/* @internal */
namespace ts.diagnostics {
    /**
     * ANSI color codes for terminal output
     */
    export const Colors = {
        // Reset
        reset: "\x1b[0m",

        // Styles
        bold: "\x1b[1m",
        dim: "\x1b[2m",
        italic: "\x1b[3m",
        underline: "\x1b[4m",

        // Foreground colors
        black: "\x1b[30m",
        red: "\x1b[31m",
        green: "\x1b[32m",
        yellow: "\x1b[33m",
        blue: "\x1b[34m",
        magenta: "\x1b[35m",
        cyan: "\x1b[36m",
        white: "\x1b[37m",
        gray: "\x1b[90m",

        // Bright foreground colors
        brightRed: "\x1b[91m",
        brightGreen: "\x1b[92m",
        brightYellow: "\x1b[93m",
        brightBlue: "\x1b[94m",
        brightMagenta: "\x1b[95m",
        brightCyan: "\x1b[96m",
        brightWhite: "\x1b[97m",

        // Background colors
        bgRed: "\x1b[41m",
        bgGreen: "\x1b[42m",
        bgYellow: "\x1b[43m",
        bgBlue: "\x1b[44m",
    };

    /**
     * Color scheme for diagnostics
     */
    export interface ColorScheme {
        error: string;
        warning: string;
        suggestion: string;
        message: string;
        location: string;
        code: string;
        lineNumber: string;
        context: string;
        highlight: string;
        reset: string;
    }

    /**
     * Default color scheme
     */
    export const DefaultColorScheme: ColorScheme = {
        error: Colors.red,
        warning: Colors.yellow,
        suggestion: Colors.cyan,
        message: Colors.blue,
        location: Colors.cyan,
        code: Colors.gray,
        lineNumber: Colors.gray,
        context: Colors.dim,
        highlight: Colors.red + Colors.underline,
        reset: Colors.reset,
    };

    /**
     * No color scheme (for non-color output)
     */
    export const NoColorScheme: ColorScheme = {
        error: "",
        warning: "",
        suggestion: "",
        message: "",
        location: "",
        code: "",
        lineNumber: "",
        context: "",
        highlight: "",
        reset: "",
    };

    /**
     * Diagnostic formatter
     */
    export class DiagnosticFormatter {
        private options: Required<FormatterOptions>;
        private colors: ColorScheme;
        private sourceFileProvider?: (fileName: string) => string | undefined;

        constructor(
            options?: FormatterOptions,
            sourceFileProvider?: (fileName: string) => string | undefined
        ) {
            this.options = {
                colors: options?.colors ?? true,
                showContext: options?.showContext ?? true,
                contextLines: options?.contextLines ?? 2,
                showRelated: options?.showRelated ?? true,
                pretty: options?.pretty ?? true,
                terminalWidth: options?.terminalWidth ?? 80,
            };
            this.colors = this.options.colors ? DefaultColorScheme : NoColorScheme;
            this.sourceFileProvider = sourceFileProvider;
        }

        /**
         * Format a single diagnostic
         */
        format(diagnostic: Diagnostic): FormattedDiagnostic {
            const lines: string[] = [];

            // Format location
            if (diagnostic.location) {
                lines.push(this.formatLocation(diagnostic.location, diagnostic));
            } else {
                lines.push(this.formatHeaderNoLocation(diagnostic));
            }

            // Format message
            lines.push(this.formatMessage(diagnostic));

            // Format source context
            if (this.options.showContext && diagnostic.location) {
                const context = this.formatSourceContext(diagnostic.location);
                if (context) {
                    lines.push(context);
                }
            }

            // Format related information
            if (this.options.showRelated && diagnostic.relatedInformation?.length) {
                lines.push("");
                for (const related of diagnostic.relatedInformation) {
                    lines.push(this.formatRelatedInfo(related));
                }
            }

            // Add spacing if pretty
            if (this.options.pretty) {
                lines.push("");
            }

            const text = lines.join("\n");
            return {
                diagnostic,
                text: this.stripColors(text),
                coloredText: this.options.colors ? text : undefined,
            };
        }

        /**
         * Format multiple diagnostics
         */
        formatAll(diagnostics: Diagnostic[]): string {
            const formatted = diagnostics.map(d => this.format(d));
            const parts = formatted.map(f => f.coloredText ?? f.text);

            // Add summary
            const sorted = sortDiagnostics(diagnostics);
            parts.push(this.formatSummary(sorted));

            return parts.join("\n");
        }

        /**
         * Format diagnostics grouped by file
         */
        formatGroupedByFile(diagnostics: Diagnostic[]): string {
            const collector = new DiagnosticCollector();
            collector.addAll(diagnostics);
            const groups = collector.groupByFile();

            const parts: string[] = [];
            for (const [file, fileDiagnostics] of groups) {
                parts.push(this.formatFileGroup(file, fileDiagnostics));
            }

            parts.push(this.formatSummary(diagnostics));
            return parts.join("\n");
        }

        /**
         * Format location line (file:line:col)
         */
        private formatLocation(location: DiagnosticLocation, diagnostic: Diagnostic): string {
            const { file, line, column } = location;
            const severityColor = this.getSeverityColor(diagnostic.severity);

            return (
                `${this.colors.location}${file}` +
                `${this.colors.reset}:` +
                `${this.colors.lineNumber}${line}` +
                `${this.colors.reset}:` +
                `${this.colors.lineNumber}${column}` +
                `${this.colors.reset} - ` +
                `${severityColor}${diagnostic.category}` +
                ` ${this.colors.code}TS${diagnostic.code}` +
                `${this.colors.reset}: `
            );
        }

        /**
         * Format header without location
         */
        private formatHeaderNoLocation(diagnostic: Diagnostic): string {
            const severityColor = this.getSeverityColor(diagnostic.severity);
            return (
                `${severityColor}${diagnostic.category}` +
                ` ${this.colors.code}TS${diagnostic.code}` +
                `${this.colors.reset}: `
            );
        }

        /**
         * Format the diagnostic message
         */
        private formatMessage(diagnostic: Diagnostic): string {
            if (this.options.pretty && diagnostic.message.length > this.options.terminalWidth) {
                return this.wrapText(diagnostic.message, this.options.terminalWidth, "  ");
            }
            return diagnostic.message;
        }

        /**
         * Format source code context
         */
        private formatSourceContext(location: DiagnosticLocation): string | undefined {
            const sourceText = this.sourceFileProvider?.(location.file);
            if (!sourceText) {
                return undefined;
            }

            const lines = sourceText.split("\n");
            const errorLine = location.line - 1; // Convert to 0-based

            if (errorLine < 0 || errorLine >= lines.length) {
                return undefined;
            }

            const contextLines = this.options.contextLines;
            const startLine = Math.max(0, errorLine - contextLines);
            const endLine = Math.min(lines.length - 1, errorLine + contextLines);

            const parts: string[] = [];
            const gutterWidth = String(endLine + 1).length;

            for (let i = startLine; i <= endLine; i++) {
                const lineNum = String(i + 1).padStart(gutterWidth);
                const line = lines[i];
                const isErrorLine = i === errorLine;

                if (isErrorLine) {
                    // Error line with marker
                    parts.push(
                        `${this.colors.lineNumber}${lineNum}${this.colors.reset} | ${line}`
                    );

                    // Underline the error
                    const startCol = location.column - 1;
                    const length = Math.min(location.length, line.length - startCol);
                    const padding = " ".repeat(gutterWidth + 3 + startCol);
                    const underline = "~".repeat(Math.max(1, length));
                    parts.push(
                        `${this.colors.highlight}${padding}${underline}${this.colors.reset}`
                    );
                } else {
                    // Context line
                    parts.push(
                        `${this.colors.context}${lineNum} | ${line}${this.colors.reset}`
                    );
                }
            }

            return parts.join("\n");
        }

        /**
         * Format related information
         */
        private formatRelatedInfo(related: RelatedInformation): string {
            const { location, message } = related;
            const prefix = `${this.colors.context}  └─ ${this.colors.reset}`;

            if (location) {
                return (
                    `${prefix}${this.colors.location}${location.file}` +
                    `${this.colors.reset}:` +
                    `${this.colors.lineNumber}${location.line}:${location.column}` +
                    `${this.colors.reset}: ${message}`
                );
            }

            return `${prefix}${message}`;
        }

        /**
         * Format file group
         */
        private formatFileGroup(fileName: string, diagnostics: Diagnostic[]): string {
            const parts: string[] = [];

            parts.push(`${Colors.bold}${Colors.underline}${fileName}${Colors.reset}`);
            parts.push("");

            for (const d of sortDiagnostics(diagnostics)) {
                const formatted = this.format(d);
                parts.push(formatted.coloredText ?? formatted.text);
            }

            return parts.join("\n");
        }

        /**
         * Format summary
         */
        private formatSummary(diagnostics: Diagnostic[]): string {
            const stats = new DiagnosticCollector();
            stats.addAll(diagnostics);
            const s = stats.getStats();

            const parts: string[] = [];

            if (s.errors > 0) {
                parts.push(`${this.colors.error}${s.errors} error${s.errors === 1 ? "" : "s"}${this.colors.reset}`);
            }
            if (s.warnings > 0) {
                parts.push(`${this.colors.warning}${s.warnings} warning${s.warnings === 1 ? "" : "s"}${this.colors.reset}`);
            }
            if (s.suggestions > 0) {
                parts.push(`${s.suggestions} suggestion${s.suggestions === 1 ? "" : "s"}`);
            }
            if (s.messages > 0) {
                parts.push(`${s.messages} message${s.messages === 1 ? "" : "s"}`);
            }

            if (parts.length === 0) {
                return `${Colors.green}No errors.${Colors.reset}`;
            }

            return `Found ${parts.join(", ")}.`;
        }

        /**
         * Get color for severity
         */
        private getSeverityColor(severity: DiagnosticSeverity): string {
            switch (severity) {
                case DiagnosticSeverity.Error:
                    return this.colors.error;
                case DiagnosticSeverity.Warning:
                    return this.colors.warning;
                case DiagnosticSeverity.Suggestion:
                    return this.colors.suggestion;
                case DiagnosticSeverity.Message:
                    return this.colors.message;
                default:
                    return this.colors.reset;
            }
        }

        /**
         * Wrap text to specified width
         */
        private wrapText(text: string, width: number, indent: string): string {
            const words = text.split(/\s+/);
            const lines: string[] = [];
            let currentLine = "";

            for (const word of words) {
                if (currentLine.length + word.length + 1 <= width) {
                    currentLine += (currentLine ? " " : "") + word;
                } else {
                    if (currentLine) {
                        lines.push(currentLine);
                    }
                    currentLine = indent + word;
                }
            }

            if (currentLine) {
                lines.push(currentLine);
            }

            return lines.join("\n");
        }

        /**
         * Strip ANSI color codes from text
         */
        private stripColors(text: string): string {
            // eslint-disable-next-line no-control-regex
            return text.replace(/\x1b\[[0-9;]*m/g, "");
        }

        /**
         * Set source file provider
         */
        setSourceFileProvider(provider: (fileName: string) => string | undefined): void {
            this.sourceFileProvider = provider;
        }

        /**
         * Enable/disable colors
         */
        setColors(enabled: boolean): void {
            this.options.colors = enabled;
            this.colors = enabled ? DefaultColorScheme : NoColorScheme;
        }

        /**
         * Set color scheme
         */
        setColorScheme(scheme: ColorScheme): void {
            this.colors = scheme;
        }
    }

    /**
     * Quick format a single diagnostic
     */
    export function formatDiagnostic(
        diagnostic: Diagnostic,
        options?: FormatterOptions
    ): string {
        const formatter = new DiagnosticFormatter(options);
        const result = formatter.format(diagnostic);
        return result.coloredText ?? result.text;
    }

    /**
     * Quick format multiple diagnostics
     */
    export function formatDiagnostics(
        diagnostics: Diagnostic[],
        options?: FormatterOptions
    ): string {
        const formatter = new DiagnosticFormatter(options);
        return formatter.formatAll(diagnostics);
    }

    /**
     * Format diagnostics in TSC style (compact)
     */
    export function formatTscStyle(diagnostics: Diagnostic[]): string {
        return diagnostics
            .map(d => {
                if (d.location) {
                    return `${d.location.file}(${d.location.line},${d.location.column}): ${d.category} TS${d.code}: ${d.message}`;
                }
                return `${d.category} TS${d.code}: ${d.message}`;
            })
            .join("\n");
    }

    /**
     * Format diagnostics as JSON
     */
    export function formatAsJson(diagnostics: Diagnostic[]): string {
        return JSON.stringify(
            diagnostics.map(d => ({
                code: d.code,
                severity: getSeverityName(d.severity),
                message: d.message,
                file: d.location?.file,
                line: d.location?.line,
                column: d.location?.column,
                length: d.location?.length,
                related: d.relatedInformation?.map(r => ({
                    message: r.message,
                    file: r.location.file,
                    line: r.location.line,
                    column: r.location.column,
                })),
            })),
            null,
            2
        );
    }

    /**
     * Create a console-friendly formatter
     */
    export function createConsoleFormatter(): DiagnosticFormatter {
        return new DiagnosticFormatter({
            colors: true,
            showContext: true,
            contextLines: 2,
            showRelated: true,
            pretty: true,
            terminalWidth: process.stdout?.columns ?? 80,
        });
    }

    /**
     * Create a CI/plain formatter (no colors)
     */
    export function createPlainFormatter(): DiagnosticFormatter {
        return new DiagnosticFormatter({
            colors: false,
            showContext: false,
            contextLines: 0,
            showRelated: true,
            pretty: false,
            terminalWidth: 120,
        });
    }
}
