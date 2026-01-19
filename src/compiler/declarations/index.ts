/**
 * Declaration File Generation Module
 *
 * This module provides infrastructure for generating .d.ts declaration files
 * from TypeScript source files. It handles:
 *
 * - Type export visibility rules
 * - Inferred types that need explicit type annotations
 * - Declaration maps generation
 * - Namespace/module merging for declarations
 * - Bundled declaration file output
 */

/* @internal */
namespace ts.declarations {
    // Re-export from types
    export type {
        DeclarationEmitOptions,
        DeclarationFileResult,
        BundledDeclarationResult,
        ModuleAugmentation,
        GlobalAugmentation,
        SymbolVisibilityInfo,
        CollectedDeclarations,
        ImportInfo,
        ImportedName,
        NamespaceMergeInfo,
        EmitterContext,
    };

    // Export visibility enum
    export { ExportVisibility };

    // Export collector functions
    export { collectDeclarations, collectNamespaceMerges };

    // Export visibility functions
    export {
        getExportVisibility,
        isInternalSymbol,
        needsTypeAnnotation,
        getRequiredSymbols,
        isSymbolAccessible,
        getAccessibilityDiagnostic,
        getRequiredImports,
    };

    // Export emitter
    export { createDeclarationEmitter, DeclarationEmitter };

    /**
     * Emit declarations for a program
     *
     * This is a convenience function that creates an emitter and emits all files.
     */
    export function emitDeclarations(
        program: Program,
        options?: DeclarationEmitOptions
    ): DeclarationFileResult[] {
        const emitter = createDeclarationEmitter(program, options || {});
        return emitter.emitAll();
    }

    /**
     * Emit bundled declarations for a program
     *
     * This creates a single bundled .d.ts file for all source files.
     */
    export function emitBundledDeclarations(
        program: Program,
        options?: DeclarationEmitOptions
    ): BundledDeclarationResult {
        const opts = { ...options, bundledOutput: true };
        const emitter = createDeclarationEmitter(program, opts);
        return emitter.emitBundled();
    }

    /**
     * Get declarations that need to be emitted for a source file
     *
     * This is useful for analyzing what would be emitted without
     * actually generating the output.
     */
    export function getDeclarationsForFile(
        sourceFile: SourceFile,
        program: Program,
        options?: DeclarationEmitOptions
    ): CollectedDeclarations {
        const checker = program.getTypeChecker();
        return collectDeclarations(sourceFile, checker, options || {});
    }

    /**
     * Check if a symbol needs explicit type annotation in declarations
     */
    export function symbolNeedsTypeAnnotation(
        symbol: Symbol,
        checker: TypeChecker
    ): boolean {
        return needsTypeAnnotation(symbol, checker);
    }

    /**
     * Get the visibility of a symbol for declaration emission
     */
    export function getSymbolVisibility(
        symbol: Symbol,
        sourceFile: SourceFile,
        checker: TypeChecker,
        stripInternal: boolean = false
    ): ExportVisibility {
        return getExportVisibility(symbol, checker, sourceFile, stripInternal);
    }
}
