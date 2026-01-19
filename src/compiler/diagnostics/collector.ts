/**
 * Diagnostics System - Thread-Safe Collector
 *
 * This module provides a thread-safe diagnostic collector that can be used
 * across multiple compilation phases. It supports filtering, grouping, and
 * statistics collection.
 */

/* @internal */
namespace ts.diagnostics {
    /**
     * Thread-safe diagnostic collector
     *
     * Collects diagnostics from multiple sources and provides
     * filtering, sorting, and statistics capabilities.
     */
    export class DiagnosticCollector {
        /** All collected diagnostics */
        private diagnostics: Diagnostic[] = [];

        /** Callbacks to notify on new diagnostics */
        private listeners: DiagnosticCallback[] = [];

        /** Active filters */
        private filters: DiagnosticFilter[] = [];

        /** Maximum number of diagnostics to collect (0 = unlimited) */
        private maxDiagnostics: number = 0;

        /** Whether to stop collecting after first error */
        private stopOnFirstError: boolean = false;

        /** Whether collection has been stopped */
        private stopped: boolean = false;

        /** Lock state for thread safety simulation */
        private locked: boolean = false;

        constructor(options?: {
            maxDiagnostics?: number;
            stopOnFirstError?: boolean;
            filters?: DiagnosticFilter[];
        }) {
            if (options) {
                this.maxDiagnostics = options.maxDiagnostics ?? 0;
                this.stopOnFirstError = options.stopOnFirstError ?? false;
                if (options.filters) {
                    this.filters = options.filters;
                }
            }
        }

        /**
         * Acquire lock for thread-safe operations
         */
        private acquireLock(): void {
            // In single-threaded JS, this is a simple flag
            // In a real multi-threaded environment, this would use proper synchronization
            while (this.locked) {
                // Spin wait - in real implementation would use proper mutex
            }
            this.locked = true;
        }

        /**
         * Release lock after thread-safe operations
         */
        private releaseLock(): void {
            this.locked = false;
        }

        /**
         * Add a single diagnostic
         */
        add(diagnostic: Diagnostic): boolean {
            if (this.stopped) {
                return false;
            }

            // Apply filters
            if (this.filters.length > 0) {
                for (const filter of this.filters) {
                    if (!filter(diagnostic)) {
                        return false;
                    }
                }
            }

            this.acquireLock();
            try {
                // Check max diagnostics
                if (this.maxDiagnostics > 0 && this.diagnostics.length >= this.maxDiagnostics) {
                    return false;
                }

                this.diagnostics.push(diagnostic);

                // Notify listeners
                for (const listener of this.listeners) {
                    listener(diagnostic);
                }

                // Check stop condition
                if (this.stopOnFirstError && diagnostic.severity === DiagnosticSeverity.Error) {
                    this.stopped = true;
                }

                return true;
            } finally {
                this.releaseLock();
            }
        }

        /**
         * Add multiple diagnostics
         */
        addAll(diagnostics: Diagnostic[]): number {
            let added = 0;
            for (const diagnostic of diagnostics) {
                if (this.add(diagnostic)) {
                    added++;
                }
            }
            return added;
        }

        /**
         * Add a diagnostic from a template
         */
        addFromTemplate(
            template: DiagnosticTemplate,
            args: (string | number)[],
            location?: DiagnosticLocation
        ): boolean {
            const diagnostic = createDiagnosticFromTemplate(template, args, location);
            return this.add(diagnostic);
        }

        /**
         * Get all collected diagnostics
         */
        getAll(): Diagnostic[] {
            return this.diagnostics.slice();
        }

        /**
         * Get diagnostics sorted by location
         */
        getSorted(): Diagnostic[] {
            return sortDiagnostics(this.diagnostics);
        }

        /**
         * Get diagnostics for a specific file
         */
        getForFile(fileName: string): Diagnostic[] {
            return this.diagnostics.filter(d => d.location?.file === fileName);
        }

        /**
         * Get diagnostics by severity
         */
        getBySeverity(severity: DiagnosticSeverity): Diagnostic[] {
            return this.diagnostics.filter(d => d.severity === severity);
        }

        /**
         * Get all errors
         */
        getErrors(): Diagnostic[] {
            return this.getBySeverity(DiagnosticSeverity.Error);
        }

        /**
         * Get all warnings
         */
        getWarnings(): Diagnostic[] {
            return this.getBySeverity(DiagnosticSeverity.Warning);
        }

        /**
         * Get diagnostics by code
         */
        getByCode(code: number): Diagnostic[] {
            return this.diagnostics.filter(d => d.code === code);
        }

        /**
         * Check if there are any errors
         */
        hasErrors(): boolean {
            return this.diagnostics.some(d => d.severity === DiagnosticSeverity.Error);
        }

        /**
         * Check if there are any warnings
         */
        hasWarnings(): boolean {
            return this.diagnostics.some(d => d.severity === DiagnosticSeverity.Warning);
        }

        /**
         * Get count of diagnostics
         */
        get count(): number {
            return this.diagnostics.length;
        }

        /**
         * Get error count
         */
        get errorCount(): number {
            return this.getErrors().length;
        }

        /**
         * Get warning count
         */
        get warningCount(): number {
            return this.getWarnings().length;
        }

        /**
         * Get statistics about collected diagnostics
         */
        getStats(): DiagnosticStats {
            const byFile = new Map<string, number>();

            let errors = 0;
            let warnings = 0;
            let suggestions = 0;
            let messages = 0;

            for (const d of this.diagnostics) {
                switch (d.severity) {
                    case DiagnosticSeverity.Error:
                        errors++;
                        break;
                    case DiagnosticSeverity.Warning:
                        warnings++;
                        break;
                    case DiagnosticSeverity.Suggestion:
                        suggestions++;
                        break;
                    case DiagnosticSeverity.Message:
                        messages++;
                        break;
                }

                if (d.location) {
                    const count = byFile.get(d.location.file) ?? 0;
                    byFile.set(d.location.file, count + 1);
                }
            }

            return {
                total: this.diagnostics.length,
                errors,
                warnings,
                suggestions,
                messages,
                byFile,
            };
        }

        /**
         * Group diagnostics by file
         */
        groupByFile(): Map<string, Diagnostic[]> {
            const groups = new Map<string, Diagnostic[]>();
            const noLocation: Diagnostic[] = [];

            for (const d of this.diagnostics) {
                if (d.location) {
                    const existing = groups.get(d.location.file);
                    if (existing) {
                        existing.push(d);
                    } else {
                        groups.set(d.location.file, [d]);
                    }
                } else {
                    noLocation.push(d);
                }
            }

            if (noLocation.length > 0) {
                groups.set("(no location)", noLocation);
            }

            return groups;
        }

        /**
         * Group diagnostics by code
         */
        groupByCode(): Map<number, Diagnostic[]> {
            const groups = new Map<number, Diagnostic[]>();

            for (const d of this.diagnostics) {
                const existing = groups.get(d.code);
                if (existing) {
                    existing.push(d);
                } else {
                    groups.set(d.code, [d]);
                }
            }

            return groups;
        }

        /**
         * Add a listener for new diagnostics
         */
        addListener(callback: DiagnosticCallback): void {
            this.listeners.push(callback);
        }

        /**
         * Remove a listener
         */
        removeListener(callback: DiagnosticCallback): void {
            const index = this.listeners.indexOf(callback);
            if (index >= 0) {
                this.listeners.splice(index, 1);
            }
        }

        /**
         * Add a filter
         */
        addFilter(filter: DiagnosticFilter): void {
            this.filters.push(filter);
        }

        /**
         * Remove a filter
         */
        removeFilter(filter: DiagnosticFilter): void {
            const index = this.filters.indexOf(filter);
            if (index >= 0) {
                this.filters.splice(index, 1);
            }
        }

        /**
         * Clear all filters
         */
        clearFilters(): void {
            this.filters = [];
        }

        /**
         * Clear all diagnostics
         */
        clear(): void {
            this.acquireLock();
            try {
                this.diagnostics = [];
                this.stopped = false;
            } finally {
                this.releaseLock();
            }
        }

        /**
         * Check if collection is stopped
         */
        isStopped(): boolean {
            return this.stopped;
        }

        /**
         * Resume collection
         */
        resume(): void {
            this.stopped = false;
        }

        /**
         * Stop collection
         */
        stop(): void {
            this.stopped = true;
        }

        /**
         * Create a child collector that reports to this one
         */
        createChild(options?: {
            filters?: DiagnosticFilter[];
        }): DiagnosticCollector {
            const child = new DiagnosticCollector(options);
            child.addListener(d => this.add(d));
            return child;
        }

        /**
         * Merge diagnostics from another collector
         */
        merge(other: DiagnosticCollector): number {
            return this.addAll(other.getAll());
        }
    }

    /**
     * Create common filters
     */
    export const CommonFilters = {
        /**
         * Only errors
         */
        errorsOnly: (d: Diagnostic) => d.severity === DiagnosticSeverity.Error,

        /**
         * No warnings
         */
        noWarnings: (d: Diagnostic) => d.severity !== DiagnosticSeverity.Warning,

        /**
         * No suggestions
         */
        noSuggestions: (d: Diagnostic) => d.severity !== DiagnosticSeverity.Suggestion,

        /**
         * Only for specific file
         */
        forFile: (fileName: string) => (d: Diagnostic) => d.location?.file === fileName,

        /**
         * Exclude specific codes
         */
        excludeCodes: (...codes: number[]) => (d: Diagnostic) => !codes.includes(d.code),

        /**
         * Include only specific codes
         */
        includeCodes: (...codes: number[]) => (d: Diagnostic) => codes.includes(d.code),

        /**
         * Filter by code range
         */
        codeRange: (min: number, max: number) => (d: Diagnostic) =>
            d.code >= min && d.code <= max,

        /**
         * Combine multiple filters (AND)
         */
        combine: (...filters: DiagnosticFilter[]) => (d: Diagnostic) =>
            filters.every(f => f(d)),

        /**
         * Combine multiple filters (OR)
         */
        any: (...filters: DiagnosticFilter[]) => (d: Diagnostic) =>
            filters.some(f => f(d)),
    };

    /**
     * Global diagnostic collector singleton
     */
    let globalCollector: DiagnosticCollector | undefined;

    /**
     * Get or create the global diagnostic collector
     */
    export function getGlobalCollector(): DiagnosticCollector {
        if (!globalCollector) {
            globalCollector = new DiagnosticCollector();
        }
        return globalCollector;
    }

    /**
     * Reset the global collector
     */
    export function resetGlobalCollector(): void {
        if (globalCollector) {
            globalCollector.clear();
        }
    }

    /**
     * Replace the global collector
     */
    export function setGlobalCollector(collector: DiagnosticCollector): void {
        globalCollector = collector;
    }
}
