/**
 * Program module - Manages all source files in a compilation.
 *
 * The Program is the main entry point for compilation. It holds all source files
 * and provides methods for type checking, emitting, and querying the program.
 */

namespace ts {
    /**
     * Options for creating a program.
     */
    export interface ProgramOptions {
        /** Root file names to include in the program */
        rootNames: readonly string[];
        /** Compiler options */
        options: CompilerOptions;
        /** Host for file system operations */
        host?: CompilerHost;
        /** Old program for incremental compilation */
        oldProgram?: ManagedProgram;
        /** Project references */
        projectReferences?: readonly ProjectReference[];
        /** Config file parsing diagnostics */
        configFileParsingDiagnostics?: readonly Diagnostic[];
    }

    /**
     * Result of resolving a module.
     */
    export interface ResolvedModuleInfo {
        /** The resolved file path (undefined if not found) */
        resolvedFileName: string | undefined;
        /** Whether this is an external library import */
        isExternalLibraryImport: boolean;
        /** The package ID if from node_modules */
        packageId?: PackageId;
        /** Extension of the resolved file */
        extension?: Extension;
        /** Failed lookup locations for diagnostics */
        failedLookupLocations: string[];
    }

    /**
     * File entry in the program with resolution info.
     */
    export interface ProgramFile {
        /** The owned source file */
        ownedFile: OwnedSourceFile;
        /** Resolved modules from this file */
        resolvedModules: Map<string, ResolvedModuleInfo>;
        /** Whether this file has been processed */
        processed: boolean;
        /** The version of this file (for incremental) */
        version: string;
    }

    /**
     * A managed program that holds all source files and provides compilation services.
     */
    export class ManagedProgram {
        /** Compiler options */
        private readonly _options: CompilerOptions;

        /** Compiler host for file operations */
        private readonly _host: CompilerHost;

        /** Map of file path to program file */
        private readonly _files: Map<string, ProgramFile> = new Map();

        /** Root file names */
        private readonly _rootNames: readonly string[];

        /** Project references */
        private readonly _projectReferences: readonly ProjectReference[] | undefined;

        /** Cached module resolution results */
        private readonly _resolvedModuleCache: Map<string, Map<string, ResolvedModuleInfo>> = new Map();

        /** Type checker instance (lazily created) */
        private _typeChecker: TypeChecker | undefined;

        /** Diagnostics from config file parsing */
        private readonly _configFileDiagnostics: readonly Diagnostic[];

        /** Semantic diagnostics cache */
        private _semanticDiagnosticsCache: Map<string, readonly Diagnostic[]> | undefined;

        /** Whether the program structure has changed */
        private _structureChanged: boolean = true;

        /** Files in order they were added */
        private _fileOrder: string[] = [];

        /** Common source directory */
        private _commonSourceDirectory: string | undefined;

        constructor(options: ProgramOptions) {
            this._options = options.options;
            this._rootNames = options.rootNames;
            this._projectReferences = options.projectReferences;
            this._configFileDiagnostics = options.configFileParsingDiagnostics ?? [];

            // Create or use provided host
            this._host = options.host ?? createCompilerHost(this._options);

            // If we have an old program, try to reuse files
            if (options.oldProgram) {
                this.reuseFilesFromOldProgram(options.oldProgram);
            }

            // Process root files
            this.processRootFiles();
        }

        /**
         * Reuse files from an old program that haven't changed.
         */
        private reuseFilesFromOldProgram(oldProgram: ManagedProgram): void {
            for (const [fileName, oldFile] of oldProgram._files) {
                // Check if file still exists and hasn't changed
                const newVersion = this.getFileVersion(fileName);
                if (newVersion && newVersion === oldFile.version) {
                    // Reuse the file
                    this._files.set(fileName, {
                        ...oldFile,
                        processed: false, // Will be reprocessed if needed
                    });
                }
            }
        }

        /**
         * Get file version (modification time or hash).
         */
        private getFileVersion(fileName: string): string | undefined {
            if (this._host.getModifiedTime) {
                const mtime = this._host.getModifiedTime(fileName);
                return mtime?.getTime().toString();
            }
            // Fallback: read file and hash
            const text = this._host.readFile(fileName);
            if (text === undefined) return undefined;
            return generateDjb2Hash(text);
        }

        /**
         * Process root files and their dependencies.
         */
        private processRootFiles(): void {
            const worklist: string[] = [...this._rootNames];
            const processed = new Set<string>();

            while (worklist.length > 0) {
                const fileName = worklist.pop()!;
                const normalizedFileName = normalizePath(fileName);

                if (processed.has(normalizedFileName)) {
                    continue;
                }
                processed.add(normalizedFileName);

                // Add or update file
                const programFile = this.addOrUpdateFile(normalizedFileName);
                if (!programFile) {
                    continue; // File not found
                }

                // Resolve and add dependencies
                const ownedFile = programFile.ownedFile;
                const moduleReferences = ownedFile.getModuleReferences();

                for (const ref of moduleReferences) {
                    const resolution = this.resolveModule(ref.specifier, normalizedFileName);
                    programFile.resolvedModules.set(ref.specifier, resolution);

                    if (resolution.resolvedFileName) {
                        worklist.push(resolution.resolvedFileName);
                    }
                }

                programFile.processed = true;
            }
        }

        /**
         * Add or update a file in the program.
         */
        private addOrUpdateFile(fileName: string): ProgramFile | undefined {
            const existing = this._files.get(fileName);
            const currentVersion = this.getFileVersion(fileName);

            if (existing && existing.version === currentVersion) {
                return existing;
            }

            // Read the file
            const text = this._host.readFile(fileName);
            if (text === undefined) {
                return undefined;
            }

            // Create owned source file
            const ownedFile = createOwnedSourceFile(
                fileName,
                text,
                this._options.target ?? ScriptTarget.Latest,
                /* setParentNodes */ true
            );

            const programFile: ProgramFile = {
                ownedFile,
                resolvedModules: new Map(),
                processed: false,
                version: currentVersion ?? "",
            };

            this._files.set(fileName, programFile);
            this._fileOrder.push(fileName);
            this._structureChanged = true;

            return programFile;
        }

        /**
         * Resolve a module specifier.
         */
        private resolveModule(specifier: string, containingFile: string): ResolvedModuleInfo {
            // Check cache
            let fileCache = this._resolvedModuleCache.get(containingFile);
            if (fileCache) {
                const cached = fileCache.get(specifier);
                if (cached) {
                    return cached;
                }
            } else {
                fileCache = new Map();
                this._resolvedModuleCache.set(containingFile, fileCache);
            }

            // Resolve using the module resolver
            const result = resolveModuleName(
                specifier,
                containingFile,
                this._options,
                this._host
            );

            const info: ResolvedModuleInfo = {
                resolvedFileName: result.resolvedModule?.resolvedFileName,
                isExternalLibraryImport: result.resolvedModule?.isExternalLibraryImport ?? false,
                packageId: result.resolvedModule?.packageId,
                extension: result.resolvedModule?.extension,
                failedLookupLocations: result.failedLookupLocations ?? [],
            };

            fileCache.set(specifier, info);
            return info;
        }

        /**
         * Get all source files in the program.
         */
        public getSourceFiles(): readonly SourceFile[] {
            return this._fileOrder
                .map(fileName => this._files.get(fileName)?.ownedFile.getSourceFile())
                .filter((sf): sf is SourceFile => sf !== undefined);
        }

        /**
         * Get a specific source file by path.
         */
        public getSourceFile(fileName: string): SourceFile | undefined {
            const normalized = normalizePath(fileName);
            return this._files.get(normalized)?.ownedFile.getSourceFile();
        }

        /**
         * Get the owned source file.
         */
        public getOwnedSourceFile(fileName: string): OwnedSourceFile | undefined {
            const normalized = normalizePath(fileName);
            return this._files.get(normalized)?.ownedFile;
        }

        /**
         * Get compiler options.
         */
        public getCompilerOptions(): CompilerOptions {
            return this._options;
        }

        /**
         * Get root file names.
         */
        public getRootFileNames(): readonly string[] {
            return this._rootNames;
        }

        /**
         * Get project references.
         */
        public getProjectReferences(): readonly ProjectReference[] | undefined {
            return this._projectReferences;
        }

        /**
         * Get the type checker for this program.
         */
        public getTypeChecker(): TypeChecker {
            if (!this._typeChecker || this._structureChanged) {
                // Create a Program interface compatible object for the type checker
                const programInterface = this.createProgramInterface();
                this._typeChecker = createTypeChecker(programInterface, /* produceDiagnostics */ true);
                this._structureChanged = false;
            }
            return this._typeChecker;
        }

        /**
         * Create an interface matching the Program type expected by the type checker.
         */
        private createProgramInterface(): Program {
            // This is a compatibility layer for the existing TypeChecker
            return {
                getSourceFiles: () => this.getSourceFiles(),
                getSourceFile: (fileName: string) => this.getSourceFile(fileName),
                getCompilerOptions: () => this._options,
                getRootFileNames: () => this._rootNames,
                getTypeChecker: () => this.getTypeChecker(),
                getProjectReferences: () => this._projectReferences,
                // ... other methods would be implemented as needed
            } as Program;
        }

        /**
         * Get syntactic diagnostics for a file.
         */
        public getSyntacticDiagnostics(fileName?: string): readonly Diagnostic[] {
            if (fileName) {
                const sf = this.getSourceFile(fileName);
                return sf ? sf.parseDiagnostics : [];
            }

            const diagnostics: Diagnostic[] = [];
            for (const file of this.getSourceFiles()) {
                diagnostics.push(...file.parseDiagnostics);
            }
            return diagnostics;
        }

        /**
         * Get semantic diagnostics for a file.
         */
        public getSemanticDiagnostics(fileName?: string): readonly Diagnostic[] {
            const typeChecker = this.getTypeChecker();

            if (fileName) {
                const sf = this.getSourceFile(fileName);
                if (!sf) return [];

                // Check cache
                if (!this._semanticDiagnosticsCache) {
                    this._semanticDiagnosticsCache = new Map();
                }

                const cached = this._semanticDiagnosticsCache.get(fileName);
                if (cached) {
                    return cached;
                }

                const diagnostics = typeChecker.getDiagnostics(sf);
                this._semanticDiagnosticsCache.set(fileName, diagnostics);
                return diagnostics;
            }

            const diagnostics: Diagnostic[] = [];
            for (const file of this.getSourceFiles()) {
                diagnostics.push(...this.getSemanticDiagnostics(file.fileName));
            }
            return diagnostics;
        }

        /**
         * Get all diagnostics.
         */
        public getAllDiagnostics(): readonly Diagnostic[] {
            return [
                ...this._configFileDiagnostics,
                ...this.getSyntacticDiagnostics(),
                ...this.getSemanticDiagnostics(),
            ];
        }

        /**
         * Get the common source directory.
         */
        public getCommonSourceDirectory(): string {
            if (this._commonSourceDirectory === undefined) {
                const fileNames = this._fileOrder.filter(f =>
                    !this._files.get(f)?.ownedFile.isDeclarationFile
                );
                this._commonSourceDirectory = computeCommonSourceDirectoryOfFilenames(
                    fileNames,
                    this._host.getCurrentDirectory(),
                    (f) => this._host.getCanonicalFileName(f)
                );
            }
            return this._commonSourceDirectory;
        }

        /**
         * Check if a file is a source file in the program.
         */
        public hasSourceFile(fileName: string): boolean {
            return this._files.has(normalizePath(fileName));
        }

        /**
         * Get the number of files in the program.
         */
        public getFileCount(): number {
            return this._files.size;
        }

        /**
         * Get resolved modules for a file.
         */
        public getResolvedModules(fileName: string): Map<string, ResolvedModuleInfo> | undefined {
            return this._files.get(normalizePath(fileName))?.resolvedModules;
        }

        /**
         * Update a file in the program (for incremental).
         */
        public updateFile(fileName: string, newText: string): void {
            const normalized = normalizePath(fileName);
            const existing = this._files.get(normalized);

            if (existing) {
                existing.ownedFile.invalidate();
            }

            // Create new owned source file
            const ownedFile = createOwnedSourceFile(
                normalized,
                newText,
                this._options.target ?? ScriptTarget.Latest,
                /* setParentNodes */ true
            );

            const programFile: ProgramFile = {
                ownedFile,
                resolvedModules: existing?.resolvedModules ?? new Map(),
                processed: false,
                version: generateDjb2Hash(newText),
            };

            this._files.set(normalized, programFile);
            this._structureChanged = true;
            this._semanticDiagnosticsCache?.delete(normalized);

            // Re-process dependencies if needed
            const moduleReferences = ownedFile.getModuleReferences();
            for (const ref of moduleReferences) {
                if (!programFile.resolvedModules.has(ref.specifier)) {
                    const resolution = this.resolveModule(ref.specifier, normalized);
                    programFile.resolvedModules.set(ref.specifier, resolution);
                }
            }
        }

        /**
         * Remove a file from the program.
         */
        public removeFile(fileName: string): void {
            const normalized = normalizePath(fileName);
            this._files.delete(normalized);
            this._fileOrder = this._fileOrder.filter(f => f !== normalized);
            this._structureChanged = true;
            this._semanticDiagnosticsCache?.delete(normalized);
            this._resolvedModuleCache.delete(normalized);
        }

        /**
         * Emit output for the program.
         */
        public emit(
            targetSourceFile?: SourceFile,
            writeFile?: WriteFileCallback,
            cancellationToken?: CancellationToken,
            emitOnlyDtsFiles?: boolean,
            customTransformers?: CustomTransformers
        ): EmitResult {
            // Create emit resolver from type checker
            const typeChecker = this.getTypeChecker();
            const emitResolver = typeChecker.getEmitResolver(
                targetSourceFile,
                cancellationToken
            );

            const writer = writeFile ?? this._host.writeFile;
            const sourceFiles = targetSourceFile
                ? [targetSourceFile]
                : this.getSourceFiles();

            const diagnostics: Diagnostic[] = [];
            let emittedFiles: string[] = [];
            let emitSkipped = false;

            for (const sourceFile of sourceFiles) {
                if (sourceFile.isDeclarationFile) {
                    continue; // Don't emit from .d.ts files
                }

                // Check for pre-emit diagnostics
                if (!emitOnlyDtsFiles) {
                    const preEmitDiagnostics = [
                        ...this.getSyntacticDiagnostics(sourceFile.fileName),
                        ...this.getSemanticDiagnostics(sourceFile.fileName),
                    ];

                    if (preEmitDiagnostics.some(d => d.category === DiagnosticCategory.Error)) {
                        emitSkipped = true;
                        diagnostics.push(...preEmitDiagnostics);
                        continue;
                    }
                }

                // Emit would happen here using the emitter
                // For now, we just track what would be emitted
                const outputPath = getOutputPathForSourceFile(
                    this._options,
                    sourceFile.fileName,
                    this.getCommonSourceDirectory()
                );

                if (outputPath) {
                    emittedFiles.push(outputPath);
                }
            }

            return {
                emitSkipped,
                diagnostics,
                emittedFiles,
            };
        }
    }

    /**
     * Get output path for a source file based on compiler options.
     */
    function getOutputPathForSourceFile(
        options: CompilerOptions,
        sourceFileName: string,
        commonSourceDirectory: string
    ): string | undefined {
        if (options.outFile || options.out) {
            return options.outFile || options.out;
        }

        const relativePath = getRelativePathFromDirectory(
            commonSourceDirectory,
            sourceFileName,
            /* ignoreCase */ false
        );

        const outputDir = options.outDir || getDirectoryPath(sourceFileName);
        const baseName = removeFileExtension(relativePath) + ".js";

        return combinePaths(outputDir, baseName);
    }

    /**
     * Create a managed program.
     */
    export function createManagedProgram(options: ProgramOptions): ManagedProgram {
        return new ManagedProgram(options);
    }

    /**
     * Simple DJB2 hash for file versioning.
     */
    function generateDjb2Hash(text: string): string {
        let hash = 5381;
        for (let i = 0; i < text.length; i++) {
            hash = ((hash << 5) + hash) + text.charCodeAt(i);
        }
        return hash.toString(36);
    }
}
