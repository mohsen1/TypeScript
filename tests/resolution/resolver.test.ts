/**
 * Tests for ModuleResolver.
 */

namespace ts {
    // Mock file system for testing
    function createMockResolutionHost(files: Set<string>, directories: Set<string>): ModuleResolutionHost {
        return {
            fileExists: (path) => files.has(normalizePath(path)),
            readFile: (path) => files.has(normalizePath(path)) ? "{}" : undefined,
            directoryExists: (path) => directories.has(normalizePath(path)),
            getDirectories: () => [],
        };
    }

    describe("ModuleResolver", () => {
        describe("relative imports", () => {
            it("should resolve relative .ts imports", () => {
                const files = new Set(["/src/foo.ts"]);
                const directories = new Set(["/src"]);
                const host = createMockResolutionHost(files, directories);

                const resolver = createModuleResolver({
                    compilerOptions: { target: ScriptTarget.ES2020 },
                    host,
                });

                const result = resolver.resolve("./foo", "/src/main.ts");

                assert.isDefined(result.resolvedFileName);
                assert.equal(result.resolvedFileName, "/src/foo.ts");
                assert.equal(result.extension, Extension.Ts);
                assert.isFalse(result.isExternalLibrary);
            });

            it("should resolve relative .tsx imports", () => {
                const files = new Set(["/src/component.tsx"]);
                const directories = new Set(["/src"]);
                const host = createMockResolutionHost(files, directories);

                const resolver = createModuleResolver({
                    compilerOptions: { target: ScriptTarget.ES2020 },
                    host,
                });

                const result = resolver.resolve("./component", "/src/main.ts");

                assert.isDefined(result.resolvedFileName);
                assert.equal(result.resolvedFileName, "/src/component.tsx");
                assert.equal(result.extension, Extension.Tsx);
            });

            it("should resolve relative .d.ts imports", () => {
                const files = new Set(["/src/types.d.ts"]);
                const directories = new Set(["/src"]);
                const host = createMockResolutionHost(files, directories);

                const resolver = createModuleResolver({
                    compilerOptions: { target: ScriptTarget.ES2020 },
                    host,
                });

                const result = resolver.resolve("./types", "/src/main.ts");

                assert.isDefined(result.resolvedFileName);
                assert.equal(result.resolvedFileName, "/src/types.d.ts");
                assert.equal(result.extension, Extension.Dts);
            });

            it("should resolve parent directory imports", () => {
                const files = new Set(["/src/utils/helper.ts"]);
                const directories = new Set(["/src", "/src/utils", "/src/components"]);
                const host = createMockResolutionHost(files, directories);

                const resolver = createModuleResolver({
                    compilerOptions: { target: ScriptTarget.ES2020 },
                    host,
                });

                const result = resolver.resolve("../utils/helper", "/src/components/button.ts");

                assert.isDefined(result.resolvedFileName);
                assert.equal(result.resolvedFileName, "/src/utils/helper.ts");
            });
        });

        describe("index file resolution", () => {
            it("should resolve directory imports to index.ts", () => {
                const files = new Set(["/src/utils/index.ts"]);
                const directories = new Set(["/src", "/src/utils"]);
                const host = createMockResolutionHost(files, directories);

                const resolver = createModuleResolver({
                    compilerOptions: { target: ScriptTarget.ES2020 },
                    host,
                });

                const result = resolver.resolve("./utils", "/src/main.ts");

                assert.isDefined(result.resolvedFileName);
                assert.equal(result.resolvedFileName, "/src/utils/index.ts");
            });

            it("should resolve directory imports to index.d.ts", () => {
                const files = new Set(["/src/utils/index.d.ts"]);
                const directories = new Set(["/src", "/src/utils"]);
                const host = createMockResolutionHost(files, directories);

                const resolver = createModuleResolver({
                    compilerOptions: { target: ScriptTarget.ES2020 },
                    host,
                });

                const result = resolver.resolve("./utils", "/src/main.ts");

                assert.isDefined(result.resolvedFileName);
                assert.equal(result.resolvedFileName, "/src/utils/index.d.ts");
            });
        });

        describe("JavaScript resolution", () => {
            it("should not resolve .js without allowJs", () => {
                const files = new Set(["/src/foo.js"]);
                const directories = new Set(["/src"]);
                const host = createMockResolutionHost(files, directories);

                const resolver = createModuleResolver({
                    compilerOptions: { target: ScriptTarget.ES2020, allowJs: false },
                    host,
                });

                const result = resolver.resolve("./foo", "/src/main.ts");

                assert.isUndefined(result.resolvedFileName);
            });

            it("should resolve .js with allowJs", () => {
                const files = new Set(["/src/foo.js"]);
                const directories = new Set(["/src"]);
                const host = createMockResolutionHost(files, directories);

                const resolver = createModuleResolver({
                    compilerOptions: { target: ScriptTarget.ES2020, allowJs: true },
                    host,
                });

                const result = resolver.resolve("./foo", "/src/main.ts");

                assert.isDefined(result.resolvedFileName);
                assert.equal(result.resolvedFileName, "/src/foo.js");
            });

            it("should prefer .ts over .js", () => {
                const files = new Set(["/src/foo.ts", "/src/foo.js"]);
                const directories = new Set(["/src"]);
                const host = createMockResolutionHost(files, directories);

                const resolver = createModuleResolver({
                    compilerOptions: { target: ScriptTarget.ES2020, allowJs: true },
                    host,
                });

                const result = resolver.resolve("./foo", "/src/main.ts");

                assert.isDefined(result.resolvedFileName);
                assert.equal(result.resolvedFileName, "/src/foo.ts");
            });
        });

        describe("path mapping", () => {
            it("should resolve using paths config", () => {
                const files = new Set(["/project/src/utils/helper.ts"]);
                const directories = new Set(["/project", "/project/src", "/project/src/utils"]);
                const host = createMockResolutionHost(files, directories);

                const resolver = createModuleResolver({
                    compilerOptions: {
                        target: ScriptTarget.ES2020,
                        baseUrl: "/project",
                        paths: {
                            "@utils/*": ["src/utils/*"],
                        },
                    },
                    host,
                });

                const result = resolver.resolve("@utils/helper", "/project/src/main.ts");

                assert.isDefined(result.resolvedFileName);
                assert.equal(result.resolvedFileName, "/project/src/utils/helper.ts");
            });

            it("should try multiple path patterns", () => {
                const files = new Set(["/project/fallback/helper.ts"]);
                const directories = new Set(["/project", "/project/fallback"]);
                const host = createMockResolutionHost(files, directories);

                const resolver = createModuleResolver({
                    compilerOptions: {
                        target: ScriptTarget.ES2020,
                        baseUrl: "/project",
                        paths: {
                            "@utils/*": ["src/utils/*", "fallback/*"],
                        },
                    },
                    host,
                });

                const result = resolver.resolve("@utils/helper", "/project/src/main.ts");

                assert.isDefined(result.resolvedFileName);
                assert.equal(result.resolvedFileName, "/project/fallback/helper.ts");
            });

            it("should resolve exact path mappings", () => {
                const files = new Set(["/project/src/config.ts"]);
                const directories = new Set(["/project", "/project/src"]);
                const host = createMockResolutionHost(files, directories);

                const resolver = createModuleResolver({
                    compilerOptions: {
                        target: ScriptTarget.ES2020,
                        baseUrl: "/project",
                        paths: {
                            "config": ["src/config"],
                        },
                    },
                    host,
                });

                const result = resolver.resolve("config", "/project/src/main.ts");

                assert.isDefined(result.resolvedFileName);
                assert.equal(result.resolvedFileName, "/project/src/config.ts");
            });
        });

        describe("baseUrl resolution", () => {
            it("should resolve from baseUrl", () => {
                const files = new Set(["/project/src/utils/helper.ts"]);
                const directories = new Set(["/project", "/project/src", "/project/src/utils"]);
                const host = createMockResolutionHost(files, directories);

                const resolver = createModuleResolver({
                    compilerOptions: {
                        target: ScriptTarget.ES2020,
                        baseUrl: "/project/src",
                    },
                    host,
                });

                const result = resolver.resolve("utils/helper", "/project/src/main.ts");

                assert.isDefined(result.resolvedFileName);
                assert.equal(result.resolvedFileName, "/project/src/utils/helper.ts");
            });
        });

        describe("node_modules resolution", () => {
            it("should resolve from node_modules", () => {
                const files = new Set([
                    "/project/node_modules/lodash/index.d.ts",
                    "/project/node_modules/lodash/package.json",
                ]);
                const directories = new Set([
                    "/project",
                    "/project/node_modules",
                    "/project/node_modules/lodash",
                ]);
                const host = createMockResolutionHost(files, directories);

                const resolver = createModuleResolver({
                    compilerOptions: {
                        target: ScriptTarget.ES2020,
                        moduleResolution: ModuleResolutionKind.NodeJs,
                    },
                    host,
                });

                const result = resolver.resolve("lodash", "/project/src/main.ts");

                assert.isDefined(result.resolvedFileName);
                assert.isTrue(result.isExternalLibrary);
            });

            it("should walk up directories to find node_modules", () => {
                const files = new Set([
                    "/project/node_modules/lodash/index.d.ts",
                    "/project/node_modules/lodash/package.json",
                ]);
                const directories = new Set([
                    "/project",
                    "/project/src",
                    "/project/src/deep",
                    "/project/src/deep/nested",
                    "/project/node_modules",
                    "/project/node_modules/lodash",
                ]);
                const host = createMockResolutionHost(files, directories);

                const resolver = createModuleResolver({
                    compilerOptions: {
                        target: ScriptTarget.ES2020,
                        moduleResolution: ModuleResolutionKind.NodeJs,
                    },
                    host,
                });

                const result = resolver.resolve("lodash", "/project/src/deep/nested/file.ts");

                assert.isDefined(result.resolvedFileName);
                assert.isTrue(result.isExternalLibrary);
            });
        });

        describe("caching", () => {
            it("should cache resolution results", () => {
                let fileExistsCalls = 0;
                const files = new Set(["/src/foo.ts"]);
                const directories = new Set(["/src"]);

                const host: ModuleResolutionHost = {
                    fileExists: (path) => {
                        fileExistsCalls++;
                        return files.has(normalizePath(path));
                    },
                    readFile: (path) => files.has(normalizePath(path)) ? "{}" : undefined,
                    directoryExists: (path) => directories.has(normalizePath(path)),
                };

                const resolver = createModuleResolver({
                    compilerOptions: { target: ScriptTarget.ES2020 },
                    host,
                });

                // First resolution
                const result1 = resolver.resolve("./foo", "/src/main.ts");
                const callsAfterFirst = fileExistsCalls;

                // Second resolution (should be cached)
                const result2 = resolver.resolve("./foo", "/src/main.ts");

                assert.equal(result1.resolvedFileName, result2.resolvedFileName);
                assert.equal(fileExistsCalls, callsAfterFirst); // No additional calls
            });

            it("should clear cache", () => {
                let fileExistsCalls = 0;
                const files = new Set(["/src/foo.ts"]);
                const directories = new Set(["/src"]);

                const host: ModuleResolutionHost = {
                    fileExists: (path) => {
                        fileExistsCalls++;
                        return files.has(normalizePath(path));
                    },
                    readFile: (path) => files.has(normalizePath(path)) ? "{}" : undefined,
                    directoryExists: (path) => directories.has(normalizePath(path)),
                };

                const resolver = createModuleResolver({
                    compilerOptions: { target: ScriptTarget.ES2020 },
                    host,
                });

                resolver.resolve("./foo", "/src/main.ts");
                const callsAfterFirst = fileExistsCalls;

                resolver.clearCache();

                resolver.resolve("./foo", "/src/main.ts");

                assert.isTrue(fileExistsCalls > callsAfterFirst);
            });
        });

        describe("failed lookups", () => {
            it("should track failed lookup locations", () => {
                const files = new Set<string>();
                const directories = new Set(["/src"]);
                const host = createMockResolutionHost(files, directories);

                const resolver = createModuleResolver({
                    compilerOptions: { target: ScriptTarget.ES2020 },
                    host,
                });

                const result = resolver.resolve("./missing", "/src/main.ts");

                assert.isUndefined(result.resolvedFileName);
                assert.isTrue(result.failedLookupLocations.length > 0);
                assert.isTrue(result.failedLookupLocations.some(loc =>
                    loc.includes("missing.ts") || loc.includes("missing.tsx")
                ));
            });
        });
    });
}
