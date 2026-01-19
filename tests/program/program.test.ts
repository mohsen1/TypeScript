/**
 * Tests for ManagedProgram.
 */

namespace ts {
    // Mock file system for testing
    interface MockFile {
        content: string;
        version?: string;
    }

    function createMockHost(files: Map<string, MockFile>): CompilerHost {
        return {
            getSourceFile: (fileName, languageVersion) => {
                const file = files.get(normalizePath(fileName));
                if (file) {
                    return createSourceFile(fileName, file.content, languageVersion);
                }
                return undefined;
            },
            getDefaultLibFileName: () => "/lib/lib.d.ts",
            writeFile: () => {},
            getCurrentDirectory: () => "/",
            getCanonicalFileName: (f) => f.toLowerCase(),
            useCaseSensitiveFileNames: () => false,
            getNewLine: () => "\n",
            fileExists: (fileName) => files.has(normalizePath(fileName)),
            readFile: (fileName) => files.get(normalizePath(fileName))?.content,
            directoryExists: (dir) => {
                const normalized = normalizePath(dir);
                for (const path of files.keys()) {
                    if (path.startsWith(normalized + "/")) {
                        return true;
                    }
                }
                return false;
            },
            getDirectories: () => [],
        };
    }

    describe("ManagedProgram", () => {
        describe("creation", () => {
            it("should create program with root files", () => {
                const files = new Map<string, MockFile>([
                    ["/src/main.ts", { content: "const x = 1;" }],
                ]);

                const program = createManagedProgram({
                    rootNames: ["/src/main.ts"],
                    options: { target: ScriptTarget.ES2020 },
                    host: createMockHost(files),
                });

                assert.equal(program.getFileCount(), 1);
                assert.isDefined(program.getSourceFile("/src/main.ts"));
            });

            it("should resolve module dependencies", () => {
                const files = new Map<string, MockFile>([
                    ["/src/main.ts", { content: "import { foo } from './foo';" }],
                    ["/src/foo.ts", { content: "export const foo = 1;" }],
                ]);

                const program = createManagedProgram({
                    rootNames: ["/src/main.ts"],
                    options: { target: ScriptTarget.ES2020 },
                    host: createMockHost(files),
                });

                assert.equal(program.getFileCount(), 2);
                assert.isDefined(program.getSourceFile("/src/main.ts"));
                assert.isDefined(program.getSourceFile("/src/foo.ts"));
            });

            it("should handle circular dependencies", () => {
                const files = new Map<string, MockFile>([
                    ["/src/a.ts", { content: "import { b } from './b'; export const a = 1;" }],
                    ["/src/b.ts", { content: "import { a } from './a'; export const b = 2;" }],
                ]);

                const program = createManagedProgram({
                    rootNames: ["/src/a.ts"],
                    options: { target: ScriptTarget.ES2020 },
                    host: createMockHost(files),
                });

                assert.equal(program.getFileCount(), 2);
            });
        });

        describe("source file access", () => {
            it("should return source files in order", () => {
                const files = new Map<string, MockFile>([
                    ["/src/a.ts", { content: "import './b';" }],
                    ["/src/b.ts", { content: "import './c';" }],
                    ["/src/c.ts", { content: "export const c = 1;" }],
                ]);

                const program = createManagedProgram({
                    rootNames: ["/src/a.ts"],
                    options: { target: ScriptTarget.ES2020 },
                    host: createMockHost(files),
                });

                const sourceFiles = program.getSourceFiles();
                assert.equal(sourceFiles.length, 3);
            });

            it("should get source file by name", () => {
                const files = new Map<string, MockFile>([
                    ["/src/main.ts", { content: "const x = 1;" }],
                ]);

                const program = createManagedProgram({
                    rootNames: ["/src/main.ts"],
                    options: { target: ScriptTarget.ES2020 },
                    host: createMockHost(files),
                });

                const sf = program.getSourceFile("/src/main.ts");
                assert.isDefined(sf);
                assert.equal(sf!.fileName, "/src/main.ts");

                const missing = program.getSourceFile("/src/missing.ts");
                assert.isUndefined(missing);
            });

            it("should check if file exists", () => {
                const files = new Map<string, MockFile>([
                    ["/src/main.ts", { content: "const x = 1;" }],
                ]);

                const program = createManagedProgram({
                    rootNames: ["/src/main.ts"],
                    options: { target: ScriptTarget.ES2020 },
                    host: createMockHost(files),
                });

                assert.isTrue(program.hasSourceFile("/src/main.ts"));
                assert.isFalse(program.hasSourceFile("/src/missing.ts"));
            });
        });

        describe("diagnostics", () => {
            it("should get syntactic diagnostics", () => {
                const files = new Map<string, MockFile>([
                    ["/src/main.ts", { content: "const x = ;" }], // Syntax error
                ]);

                const program = createManagedProgram({
                    rootNames: ["/src/main.ts"],
                    options: { target: ScriptTarget.ES2020 },
                    host: createMockHost(files),
                });

                const diagnostics = program.getSyntacticDiagnostics("/src/main.ts");
                assert.isTrue(diagnostics.length > 0);
            });

            it("should get all diagnostics", () => {
                const files = new Map<string, MockFile>([
                    ["/src/main.ts", { content: "const x: number = 'string';" }], // Type error
                ]);

                const program = createManagedProgram({
                    rootNames: ["/src/main.ts"],
                    options: { target: ScriptTarget.ES2020, strict: true },
                    host: createMockHost(files),
                });

                const diagnostics = program.getAllDiagnostics();
                assert.isDefined(diagnostics);
            });
        });

        describe("module resolution", () => {
            it("should resolve relative imports", () => {
                const files = new Map<string, MockFile>([
                    ["/src/main.ts", { content: "import { foo } from './utils/foo';" }],
                    ["/src/utils/foo.ts", { content: "export const foo = 1;" }],
                ]);

                const program = createManagedProgram({
                    rootNames: ["/src/main.ts"],
                    options: { target: ScriptTarget.ES2020 },
                    host: createMockHost(files),
                });

                const resolved = program.getResolvedModules("/src/main.ts");
                assert.isDefined(resolved);
                assert.isDefined(resolved?.get("./utils/foo"));
            });

            it("should resolve with different extensions", () => {
                const files = new Map<string, MockFile>([
                    ["/src/main.ts", { content: "import { foo } from './foo';" }],
                    ["/src/foo.tsx", { content: "export const foo = 1;" }],
                ]);

                const program = createManagedProgram({
                    rootNames: ["/src/main.ts"],
                    options: { target: ScriptTarget.ES2020 },
                    host: createMockHost(files),
                });

                assert.equal(program.getFileCount(), 2);
            });
        });

        describe("incremental updates", () => {
            it("should update a file", () => {
                const files = new Map<string, MockFile>([
                    ["/src/main.ts", { content: "const x = 1;" }],
                ]);

                const program = createManagedProgram({
                    rootNames: ["/src/main.ts"],
                    options: { target: ScriptTarget.ES2020 },
                    host: createMockHost(files),
                });

                const sf1 = program.getSourceFile("/src/main.ts");
                assert.isDefined(sf1);

                program.updateFile("/src/main.ts", "const x = 2;");

                const sf2 = program.getSourceFile("/src/main.ts");
                assert.isDefined(sf2);
                assert.notStrictEqual(sf1, sf2);
            });

            it("should remove a file", () => {
                const files = new Map<string, MockFile>([
                    ["/src/main.ts", { content: "const x = 1;" }],
                    ["/src/other.ts", { content: "const y = 2;" }],
                ]);

                const program = createManagedProgram({
                    rootNames: ["/src/main.ts", "/src/other.ts"],
                    options: { target: ScriptTarget.ES2020 },
                    host: createMockHost(files),
                });

                assert.equal(program.getFileCount(), 2);

                program.removeFile("/src/other.ts");

                assert.equal(program.getFileCount(), 1);
                assert.isFalse(program.hasSourceFile("/src/other.ts"));
            });
        });

        describe("compiler options", () => {
            it("should return compiler options", () => {
                const files = new Map<string, MockFile>([
                    ["/src/main.ts", { content: "const x = 1;" }],
                ]);

                const options: CompilerOptions = {
                    target: ScriptTarget.ES2020,
                    strict: true,
                    module: ModuleKind.ESNext,
                };

                const program = createManagedProgram({
                    rootNames: ["/src/main.ts"],
                    options,
                    host: createMockHost(files),
                });

                const retrievedOptions = program.getCompilerOptions();
                assert.equal(retrievedOptions.target, ScriptTarget.ES2020);
                assert.isTrue(retrievedOptions.strict);
            });

            it("should return root file names", () => {
                const files = new Map<string, MockFile>([
                    ["/src/a.ts", { content: "const a = 1;" }],
                    ["/src/b.ts", { content: "const b = 2;" }],
                ]);

                const program = createManagedProgram({
                    rootNames: ["/src/a.ts", "/src/b.ts"],
                    options: { target: ScriptTarget.ES2020 },
                    host: createMockHost(files),
                });

                const rootNames = program.getRootFileNames();
                assert.equal(rootNames.length, 2);
                assert.include(rootNames, "/src/a.ts");
                assert.include(rootNames, "/src/b.ts");
            });
        });

        describe("common source directory", () => {
            it("should compute common source directory", () => {
                const files = new Map<string, MockFile>([
                    ["/src/a.ts", { content: "const a = 1;" }],
                    ["/src/b.ts", { content: "const b = 2;" }],
                    ["/src/sub/c.ts", { content: "const c = 3;" }],
                ]);

                const program = createManagedProgram({
                    rootNames: ["/src/a.ts", "/src/b.ts", "/src/sub/c.ts"],
                    options: { target: ScriptTarget.ES2020 },
                    host: createMockHost(files),
                });

                const commonDir = program.getCommonSourceDirectory();
                assert.equal(commonDir, "/src");
            });
        });
    });
}
