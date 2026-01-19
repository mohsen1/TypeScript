/**
 * Module Resolver - Implements TypeScript module resolution algorithm.
 *
 * This module provides a clean implementation of module resolution that
 * matches TypeScript's behavior for Node.js and classic resolution modes.
 */

namespace ts {
    /**
     * Supported file extensions for module resolution in priority order.
     */
    export const MODULE_EXTENSIONS = {
        typescript: [Extension.Ts, Extension.Tsx, Extension.Dts] as const,
        javascript: [Extension.Js, Extension.Jsx] as const,
        declaration: [Extension.Dts, Extension.Dmts, Extension.Dcts] as const,
        all: [
            Extension.Ts, Extension.Tsx, Extension.Dts,
            Extension.Js, Extension.Jsx,
            Extension.Dmts, Extension.Dcts, Extension.Mts, Extension.Cts,
            Extension.Mjs, Extension.Cjs
        ] as const,
    };

    /**
     * Options for the module resolver.
     */
    export interface ModuleResolverOptions {
        /** The compiler options */
        compilerOptions: CompilerOptions;
        /** Host for file system operations */
        host: ModuleResolutionHost;
        /** Whether to trace resolution for debugging */
        traceEnabled?: boolean;
    }

    /**
     * Result of module resolution.
     */
    export interface ModuleResolutionResult {
        /** The resolved file path (undefined if not found) */
        resolvedFileName: string | undefined;
        /** The extension of the resolved file */
        extension: Extension | undefined;
        /** Whether this is from node_modules */
        isExternalLibrary: boolean;
        /** Package ID for npm packages */
        packageId: PackageId | undefined;
        /** Locations that were checked but didn't exist */
        failedLookupLocations: string[];
        /** The original path before symlink resolution */
        originalPath?: string;
    }

    /**
     * State maintained during resolution.
     */
    interface ResolutionState {
        compilerOptions: CompilerOptions;
        host: ModuleResolutionHost;
        traceEnabled: boolean;
        failedLookupLocations: string[];
        resultFromCache?: ResolvedModuleWithFailedLookupLocations;
    }

    /**
     * Module resolver that implements TypeScript's resolution algorithm.
     */
    export class ModuleResolver {
        private readonly options: CompilerOptions;
        private readonly host: ModuleResolutionHost;
        private readonly traceEnabled: boolean;

        /** Cache for resolved modules */
        private readonly cache: Map<string, Map<string, ModuleResolutionResult>> = new Map();

        /** Cache for package.json contents */
        private readonly packageJsonCache: Map<string, PackageJson | undefined> = new Map();

        constructor(resolverOptions: ModuleResolverOptions) {
            this.options = resolverOptions.compilerOptions;
            this.host = resolverOptions.host;
            this.traceEnabled = resolverOptions.traceEnabled ?? false;
        }

        /**
         * Resolve a module specifier from a containing file.
         */
        public resolve(moduleName: string, containingFile: string): ModuleResolutionResult {
            const containingDirectory = getDirectoryPath(containingFile);

            // Check cache
            let fileCache = this.cache.get(containingFile);
            if (fileCache) {
                const cached = fileCache.get(moduleName);
                if (cached) {
                    return cached;
                }
            } else {
                fileCache = new Map();
                this.cache.set(containingFile, fileCache);
            }

            // Perform resolution
            const result = this.resolveModuleName(moduleName, containingDirectory);

            // Cache and return
            fileCache.set(moduleName, result);
            return result;
        }

        /**
         * Resolve a module name from a directory.
         */
        private resolveModuleName(moduleName: string, containingDirectory: string): ModuleResolutionResult {
            const failedLookupLocations: string[] = [];

            // Determine resolution strategy based on module kind
            const moduleResolution = this.options.moduleResolution ?? ModuleResolutionKind.NodeJs;

            let result: ModuleResolutionResult | undefined;

            if (isExternalModuleNameRelative(moduleName)) {
                // Relative import: ./foo, ../bar
                result = this.resolveRelativeModule(moduleName, containingDirectory, failedLookupLocations);
            } else {
                // Non-relative import: foo, @scope/bar

                // First, check path mappings
                if (this.options.paths) {
                    result = this.resolveWithPathMappings(moduleName, containingDirectory, failedLookupLocations);
                }

                // Then, check baseUrl
                if (!result && this.options.baseUrl) {
                    result = this.resolveFromBaseUrl(moduleName, failedLookupLocations);
                }

                // Finally, resolve from node_modules
                if (!result) {
                    if (moduleResolution === ModuleResolutionKind.NodeJs ||
                        moduleResolution === ModuleResolutionKind.Node16 ||
                        moduleResolution === ModuleResolutionKind.NodeNext) {
                        result = this.resolveFromNodeModules(moduleName, containingDirectory, failedLookupLocations);
                    } else {
                        // Classic resolution
                        result = this.resolveClassic(moduleName, containingDirectory, failedLookupLocations);
                    }
                }
            }

            return result ?? {
                resolvedFileName: undefined,
                extension: undefined,
                isExternalLibrary: false,
                packageId: undefined,
                failedLookupLocations,
            };
        }

        /**
         * Resolve a relative module path.
         */
        private resolveRelativeModule(
            moduleName: string,
            containingDirectory: string,
            failedLookupLocations: string[]
        ): ModuleResolutionResult | undefined {
            const resolved = combinePaths(containingDirectory, moduleName);
            return this.tryResolveFromPath(resolved, failedLookupLocations, /* isExternalLibrary */ false);
        }

        /**
         * Resolve using path mappings from tsconfig.
         */
        private resolveWithPathMappings(
            moduleName: string,
            containingDirectory: string,
            failedLookupLocations: string[]
        ): ModuleResolutionResult | undefined {
            const paths = this.options.paths!;
            const baseUrl = this.options.baseUrl ?? getDirectoryPath(this.options.configFilePath ?? "");

            for (const pattern of Object.keys(paths)) {
                const matchedStar = matchPatternOrExact(pattern, moduleName);
                if (matchedStar !== undefined) {
                    const substitutions = paths[pattern];
                    for (const substitution of substitutions) {
                        const candidate = substitution.replace("*", matchedStar);
                        const resolved = combinePaths(baseUrl, candidate);

                        const result = this.tryResolveFromPath(resolved, failedLookupLocations, /* isExternalLibrary */ false);
                        if (result) {
                            return result;
                        }
                    }
                }
            }

            return undefined;
        }

        /**
         * Resolve from baseUrl.
         */
        private resolveFromBaseUrl(
            moduleName: string,
            failedLookupLocations: string[]
        ): ModuleResolutionResult | undefined {
            const baseUrl = this.options.baseUrl!;
            const resolved = combinePaths(baseUrl, moduleName);
            return this.tryResolveFromPath(resolved, failedLookupLocations, /* isExternalLibrary */ false);
        }

        /**
         * Resolve from node_modules directories.
         */
        private resolveFromNodeModules(
            moduleName: string,
            containingDirectory: string,
            failedLookupLocations: string[]
        ): ModuleResolutionResult | undefined {
            // Walk up the directory tree looking for node_modules
            let directory = containingDirectory;

            while (true) {
                const nodeModulesPath = combinePaths(directory, "node_modules");
                const modulePath = combinePaths(nodeModulesPath, moduleName);

                if (this.directoryExists(nodeModulesPath)) {
                    // Check for package.json main/types/exports
                    const packageResult = this.resolveFromPackage(modulePath, failedLookupLocations);
                    if (packageResult) {
                        return { ...packageResult, isExternalLibrary: true };
                    }

                    // Try direct file resolution
                    const fileResult = this.tryResolveFromPath(modulePath, failedLookupLocations, /* isExternalLibrary */ true);
                    if (fileResult) {
                        return fileResult;
                    }
                }

                // Move up one directory
                const parent = getDirectoryPath(directory);
                if (parent === directory) {
                    break; // Reached root
                }
                directory = parent;
            }

            return undefined;
        }

        /**
         * Resolve using classic (non-Node) resolution.
         */
        private resolveClassic(
            moduleName: string,
            containingDirectory: string,
            failedLookupLocations: string[]
        ): ModuleResolutionResult | undefined {
            // Walk up directories looking for the module
            let directory = containingDirectory;

            while (true) {
                const resolved = combinePaths(directory, moduleName);
                const result = this.tryResolveFromPath(resolved, failedLookupLocations, /* isExternalLibrary */ false);
                if (result) {
                    return result;
                }

                const parent = getDirectoryPath(directory);
                if (parent === directory) {
                    break;
                }
                directory = parent;
            }

            return undefined;
        }

        /**
         * Resolve a module from a package.json.
         */
        private resolveFromPackage(
            packagePath: string,
            failedLookupLocations: string[]
        ): ModuleResolutionResult | undefined {
            const packageJsonPath = combinePaths(packagePath, "package.json");

            if (!this.fileExists(packageJsonPath)) {
                failedLookupLocations.push(packageJsonPath);
                return undefined;
            }

            const packageJson = this.readPackageJson(packageJsonPath);
            if (!packageJson) {
                return undefined;
            }

            // Check exports field (Node 12+)
            if (packageJson.exports) {
                const exportResult = this.resolvePackageExports(packagePath, packageJson, failedLookupLocations);
                if (exportResult) {
                    return this.addPackageId(exportResult, packagePath, packageJson);
                }
            }

            // Check types/typings field
            const typesField = packageJson.types ?? packageJson.typings;
            if (typesField) {
                const typesPath = combinePaths(packagePath, typesField);
                if (this.fileExists(typesPath)) {
                    return this.addPackageId({
                        resolvedFileName: typesPath,
                        extension: this.getExtension(typesPath),
                        isExternalLibrary: true,
                        packageId: undefined,
                        failedLookupLocations,
                    }, packagePath, packageJson);
                }
                failedLookupLocations.push(typesPath);
            }

            // Check main field
            if (packageJson.main) {
                const mainPath = combinePaths(packagePath, packageJson.main);
                const result = this.tryResolveFromPath(mainPath, failedLookupLocations, /* isExternalLibrary */ true);
                if (result) {
                    return this.addPackageId(result, packagePath, packageJson);
                }
            }

            // Try index files
            return this.tryResolveFromPath(combinePaths(packagePath, "index"), failedLookupLocations, /* isExternalLibrary */ true);
        }

        /**
         * Resolve package exports field.
         */
        private resolvePackageExports(
            packagePath: string,
            packageJson: PackageJson,
            failedLookupLocations: string[]
        ): ModuleResolutionResult | undefined {
            const exports = packageJson.exports;
            if (!exports) return undefined;

            // Handle simple string export
            if (typeof exports === "string") {
                const resolved = combinePaths(packagePath, exports);
                return this.tryResolveFromPath(resolved, failedLookupLocations, /* isExternalLibrary */ true);
            }

            // Handle conditional exports
            if (typeof exports === "object") {
                // Check "." entry for root import
                const rootExport = (exports as Record<string, unknown>)["."];
                if (rootExport) {
                    return this.resolveConditionalExport(packagePath, rootExport, failedLookupLocations);
                }

                // Check for types/import/require conditions at top level
                const resolved = this.resolveConditionalExport(packagePath, exports, failedLookupLocations);
                if (resolved) {
                    return resolved;
                }
            }

            return undefined;
        }

        /**
         * Resolve a conditional export object.
         */
        private resolveConditionalExport(
            packagePath: string,
            exportValue: unknown,
            failedLookupLocations: string[]
        ): ModuleResolutionResult | undefined {
            if (typeof exportValue === "string") {
                const resolved = combinePaths(packagePath, exportValue);
                return this.tryResolveFromPath(resolved, failedLookupLocations, /* isExternalLibrary */ true);
            }

            if (typeof exportValue === "object" && exportValue !== null) {
                const conditions = exportValue as Record<string, unknown>;

                // Priority: types > import > require > default
                const conditionOrder = ["types", "import", "require", "default"];

                for (const condition of conditionOrder) {
                    if (condition in conditions) {
                        const result = this.resolveConditionalExport(packagePath, conditions[condition], failedLookupLocations);
                        if (result) {
                            return result;
                        }
                    }
                }
            }

            return undefined;
        }

        /**
         * Try to resolve a file from a path (with or without extension).
         */
        private tryResolveFromPath(
            path: string,
            failedLookupLocations: string[],
            isExternalLibrary: boolean
        ): ModuleResolutionResult | undefined {
            // Try with TypeScript extensions first
            for (const ext of MODULE_EXTENSIONS.typescript) {
                const candidate = path + ext;
                if (this.fileExists(candidate)) {
                    return {
                        resolvedFileName: candidate,
                        extension: ext,
                        isExternalLibrary,
                        packageId: undefined,
                        failedLookupLocations,
                    };
                }
                failedLookupLocations.push(candidate);
            }

            // Check if it's already a valid file
            if (this.fileExists(path)) {
                return {
                    resolvedFileName: path,
                    extension: this.getExtension(path),
                    isExternalLibrary,
                    packageId: undefined,
                    failedLookupLocations,
                };
            }
            failedLookupLocations.push(path);

            // Try JavaScript extensions if allowJs
            if (this.options.allowJs) {
                for (const ext of MODULE_EXTENSIONS.javascript) {
                    const candidate = path + ext;
                    if (this.fileExists(candidate)) {
                        return {
                            resolvedFileName: candidate,
                            extension: ext,
                            isExternalLibrary,
                            packageId: undefined,
                            failedLookupLocations,
                        };
                    }
                    failedLookupLocations.push(candidate);
                }
            }

            // Try as directory with index file
            return this.tryResolveAsDirectory(path, failedLookupLocations, isExternalLibrary);
        }

        /**
         * Try to resolve a path as a directory with an index file.
         */
        private tryResolveAsDirectory(
            path: string,
            failedLookupLocations: string[],
            isExternalLibrary: boolean
        ): ModuleResolutionResult | undefined {
            // Check for package.json in directory
            const packageJsonPath = combinePaths(path, "package.json");
            if (this.fileExists(packageJsonPath)) {
                const packageJson = this.readPackageJson(packageJsonPath);
                if (packageJson) {
                    // Check types field
                    const typesField = packageJson.types ?? packageJson.typings;
                    if (typesField) {
                        const typesPath = combinePaths(path, typesField);
                        if (this.fileExists(typesPath)) {
                            return {
                                resolvedFileName: typesPath,
                                extension: this.getExtension(typesPath),
                                isExternalLibrary,
                                packageId: undefined,
                                failedLookupLocations,
                            };
                        }
                    }

                    // Check main field
                    if (packageJson.main) {
                        const mainPath = combinePaths(path, packageJson.main);
                        const result = this.tryResolveFromPath(mainPath, failedLookupLocations, isExternalLibrary);
                        if (result) {
                            return result;
                        }
                    }
                }
            }

            // Try index files
            for (const ext of MODULE_EXTENSIONS.typescript) {
                const indexPath = combinePaths(path, "index" + ext);
                if (this.fileExists(indexPath)) {
                    return {
                        resolvedFileName: indexPath,
                        extension: ext,
                        isExternalLibrary,
                        packageId: undefined,
                        failedLookupLocations,
                    };
                }
                failedLookupLocations.push(indexPath);
            }

            if (this.options.allowJs) {
                for (const ext of MODULE_EXTENSIONS.javascript) {
                    const indexPath = combinePaths(path, "index" + ext);
                    if (this.fileExists(indexPath)) {
                        return {
                            resolvedFileName: indexPath,
                            extension: ext,
                            isExternalLibrary,
                            packageId: undefined,
                            failedLookupLocations,
                        };
                    }
                    failedLookupLocations.push(indexPath);
                }
            }

            return undefined;
        }

        /**
         * Add package ID to a resolution result.
         */
        private addPackageId(
            result: ModuleResolutionResult,
            packagePath: string,
            packageJson: PackageJson
        ): ModuleResolutionResult {
            if (packageJson.name && packageJson.version) {
                const subModuleName = result.resolvedFileName
                    ? result.resolvedFileName.slice(packagePath.length + 1)
                    : "";

                return {
                    ...result,
                    packageId: {
                        name: packageJson.name,
                        version: packageJson.version,
                        subModuleName,
                    },
                };
            }
            return result;
        }

        /**
         * Read and cache package.json.
         */
        private readPackageJson(path: string): PackageJson | undefined {
            const cached = this.packageJsonCache.get(path);
            if (cached !== undefined) {
                return cached;
            }

            try {
                const content = this.host.readFile(path);
                if (content) {
                    const json = JSON.parse(content) as PackageJson;
                    this.packageJsonCache.set(path, json);
                    return json;
                }
            } catch {
                // Invalid JSON
            }

            this.packageJsonCache.set(path, undefined);
            return undefined;
        }

        /**
         * Check if a file exists.
         */
        private fileExists(path: string): boolean {
            return this.host.fileExists(path);
        }

        /**
         * Check if a directory exists.
         */
        private directoryExists(path: string): boolean {
            return this.host.directoryExists?.(path) ?? false;
        }

        /**
         * Get the extension from a file path.
         */
        private getExtension(path: string): Extension {
            if (path.endsWith(Extension.Dts)) return Extension.Dts;
            if (path.endsWith(Extension.Dmts)) return Extension.Dmts;
            if (path.endsWith(Extension.Dcts)) return Extension.Dcts;
            if (path.endsWith(Extension.Ts)) return Extension.Ts;
            if (path.endsWith(Extension.Tsx)) return Extension.Tsx;
            if (path.endsWith(Extension.Mts)) return Extension.Mts;
            if (path.endsWith(Extension.Cts)) return Extension.Cts;
            if (path.endsWith(Extension.Js)) return Extension.Js;
            if (path.endsWith(Extension.Jsx)) return Extension.Jsx;
            if (path.endsWith(Extension.Mjs)) return Extension.Mjs;
            if (path.endsWith(Extension.Cjs)) return Extension.Cjs;
            if (path.endsWith(Extension.Json)) return Extension.Json;
            return Extension.Js; // Default
        }

        /**
         * Clear the resolution cache.
         */
        public clearCache(): void {
            this.cache.clear();
            this.packageJsonCache.clear();
        }
    }

    /**
     * Package.json structure (subset of fields we care about).
     */
    interface PackageJson {
        name?: string;
        version?: string;
        main?: string;
        types?: string;
        typings?: string;
        exports?: string | Record<string, unknown>;
    }

    /**
     * Match a pattern like "foo/*" against a module name.
     * Returns the matched star portion or empty string for exact match.
     */
    function matchPatternOrExact(pattern: string, moduleName: string): string | undefined {
        const starIndex = pattern.indexOf("*");

        if (starIndex === -1) {
            // Exact match
            return pattern === moduleName ? "" : undefined;
        }

        const prefix = pattern.substring(0, starIndex);
        const suffix = pattern.substring(starIndex + 1);

        if (moduleName.startsWith(prefix) && moduleName.endsWith(suffix)) {
            const matchedPart = moduleName.substring(prefix.length, moduleName.length - suffix.length);
            return matchedPart;
        }

        return undefined;
    }

    /**
     * Create a module resolver.
     */
    export function createModuleResolver(options: ModuleResolverOptions): ModuleResolver {
        return new ModuleResolver(options);
    }
}
