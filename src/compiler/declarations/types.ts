/**
 * Declaration File Generation Types
 *
 * Defines types and interfaces for the declaration file generator infrastructure.
 */

/* @internal */
namespace ts.declarations {
    /**
     * Options for declaration file generation
     */
    export interface DeclarationEmitOptions {
        /** Whether to include declaration maps */
        declarationMap?: boolean;
        /** Whether to remove internal declarations */
        stripInternal?: boolean;
        /** Whether to generate bundled output */
        bundledOutput?: boolean;
        /** Root directory for output */
        outDir?: string;
        /** Output file for bundled declarations */
        outFile?: string;
        /** Whether to remove comments */
        removeComments?: boolean;
    }

    /**
     * Result of declaration emission for a single file
     */
    export interface DeclarationFileResult {
        /** The generated declaration file content */
        declarationText: string;
        /** The declaration map content (if generated) */
        declarationMapText?: string;
        /** The output file path */
        outputPath: string;
        /** The declaration map output path */
        mapPath?: string;
        /** Any diagnostics generated during emission */
        diagnostics: Diagnostic[];
    }

    /**
     * Result of bundled declaration emission
     */
    export interface BundledDeclarationResult {
        /** The bundled declaration file content */
        declarationText: string;
        /** The declaration map content (if generated) */
        declarationMapText?: string;
        /** Module augmentations found */
        moduleAugmentations: ModuleAugmentation[];
        /** Global augmentations found */
        globalAugmentations: GlobalAugmentation[];
        /** Diagnostics from emission */
        diagnostics: Diagnostic[];
    }

    /**
     * Represents a module augmentation in declarations
     */
    export interface ModuleAugmentation {
        moduleName: string;
        declarations: string;
    }

    /**
     * Represents a global augmentation
     */
    export interface GlobalAugmentation {
        declarations: string;
    }

    /**
     * Visibility of an exported symbol
     */
    export const enum ExportVisibility {
        /** Not exported */
        None = 0,
        /** Exported from the module */
        Exported = 1,
        /** Exported but needs type annotation */
        ExportedNeedsAnnotation = 2,
        /** Re-exported from another module */
        ReExported = 3,
        /** Internal (stripped with stripInternal) */
        Internal = 4,
    }

    /**
     * Information about a symbol's visibility for declarations
     */
    export interface SymbolVisibilityInfo {
        symbol: Symbol;
        visibility: ExportVisibility;
        needsTypeAnnotation: boolean;
        accessibleFrom: SourceFile[];
        inferredType?: Type;
    }

    /**
     * Declaration collector result for a source file
     */
    export interface CollectedDeclarations {
        /** Exported declarations */
        exports: SymbolVisibilityInfo[];
        /** Import declarations needed */
        imports: ImportInfo[];
        /** Type reference directives needed */
        typeReferences: string[];
        /** File reference directives needed */
        fileReferences: string[];
        /** Whether the file has export= */
        hasExportAssignment: boolean;
        /** Whether the file is an external module */
        isExternalModule: boolean;
    }

    /**
     * Import information for declarations
     */
    export interface ImportInfo {
        moduleSpecifier: string;
        importedNames: ImportedName[];
        isTypeOnly: boolean;
    }

    /**
     * Imported name information
     */
    export interface ImportedName {
        name: string;
        alias?: string;
        isDefault: boolean;
        isNamespace: boolean;
    }

    /**
     * Namespace merging information
     */
    export interface NamespaceMergeInfo {
        name: string;
        declarations: Declaration[];
        mergedType: Type;
    }

    /**
     * Declaration emitter context
     */
    export interface EmitterContext {
        sourceFile: SourceFile;
        checker: TypeChecker;
        host: EmitHost;
        options: DeclarationEmitOptions;
        resolver: EmitResolver;
        factory: NodeFactory;
    }
}
