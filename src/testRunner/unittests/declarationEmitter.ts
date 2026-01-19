describe("unittests:: Declaration Emitter", () => {
    function createTestProgram(files: { name: string; content: string }[]): ts.Program {
        const documents = files.map(f => new documents.TextDocument(f.name, f.content));
        const host = new fakes.CompilerHost(vfs.createFromFileSystem(
            Harness.IO,
            /*ignoreCase*/ true,
            { documents, cwd: "/" }
        ));

        return ts.createProgram({
            host,
            rootNames: files.map(f => f.name),
            options: {
                declaration: true,
                noLib: true,
                strict: true
            }
        });
    }

    describe("visibility rules", () => {
        it("should identify exported symbols", () => {
            const program = createTestProgram([{
                name: "/test.ts",
                content: `
                    export const foo = 42;
                    const bar = "internal";
                `
            }]);

            const sourceFile = program.getSourceFile("/test.ts")!;
            const checker = program.getTypeChecker();
            const collected = ts.declarations.collectDeclarations(sourceFile, checker, {});

            // Should only have the exported symbol
            const exportedNames = collected.exports.map(e => e.symbol.name);
            assert.include(exportedNames, "foo");
            assert.notInclude(exportedNames, "bar");
        });

        it("should strip internal declarations", () => {
            const program = createTestProgram([{
                name: "/test.ts",
                content: `
                    /** @internal */
                    export const internal = 1;
                    export const public_ = 2;
                `
            }]);

            const sourceFile = program.getSourceFile("/test.ts")!;
            const checker = program.getTypeChecker();
            const collected = ts.declarations.collectDeclarations(sourceFile, checker, {
                stripInternal: true
            });

            const exportedNames = collected.exports.map(e => e.symbol.name);
            assert.notInclude(exportedNames, "internal");
            assert.include(exportedNames, "public_");
        });

        it("should identify re-exported symbols", () => {
            const program = createTestProgram([
                {
                    name: "/lib.ts",
                    content: `export const libFunc = () => {};`
                },
                {
                    name: "/test.ts",
                    content: `export { libFunc } from "./lib";`
                }
            ]);

            const sourceFile = program.getSourceFile("/test.ts")!;
            const checker = program.getTypeChecker();
            const collected = ts.declarations.collectDeclarations(sourceFile, checker, {});

            const reExports = collected.exports.filter(
                e => e.visibility === ts.declarations.ExportVisibility.ReExported
            );
            assert.isAbove(reExports.length, 0);
        });
    });

    describe("type annotation inference", () => {
        it("should detect variables needing type annotation", () => {
            const program = createTestProgram([{
                name: "/test.ts",
                content: `
                    export const typed: number = 5;
                    export const untyped = { foo: 1, bar: "str" };
                `
            }]);

            const sourceFile = program.getSourceFile("/test.ts")!;
            const checker = program.getTypeChecker();
            const collected = ts.declarations.collectDeclarations(sourceFile, checker, {});

            const typed = collected.exports.find(e => e.symbol.name === "typed");
            const untyped = collected.exports.find(e => e.symbol.name === "untyped");

            assert.isFalse(typed?.needsTypeAnnotation);
            assert.isTrue(untyped?.needsTypeAnnotation);
        });

        it("should detect functions needing return type", () => {
            const program = createTestProgram([{
                name: "/test.ts",
                content: `
                    export function typed(): number { return 5; }
                    export function untyped() { return 5; }
                `
            }]);

            const sourceFile = program.getSourceFile("/test.ts")!;
            const checker = program.getTypeChecker();
            const collected = ts.declarations.collectDeclarations(sourceFile, checker, {});

            const typed = collected.exports.find(e => e.symbol.name === "typed");
            const untyped = collected.exports.find(e => e.symbol.name === "untyped");

            assert.isFalse(typed?.needsTypeAnnotation);
            assert.isTrue(untyped?.needsTypeAnnotation);
        });
    });

    describe("declaration emission", () => {
        it("should emit variable declarations", () => {
            const program = createTestProgram([{
                name: "/test.ts",
                content: `export const foo: number = 42;`
            }]);

            const emitter = ts.declarations.createDeclarationEmitter(program, {});
            const result = emitter.emitFile(program.getSourceFile("/test.ts")!);

            assert.include(result.declarationText, "declare const foo: number");
        });

        it("should emit function declarations", () => {
            const program = createTestProgram([{
                name: "/test.ts",
                content: `export function greet(name: string): string { return "Hello " + name; }`
            }]);

            const emitter = ts.declarations.createDeclarationEmitter(program, {});
            const result = emitter.emitFile(program.getSourceFile("/test.ts")!);

            assert.include(result.declarationText, "declare function greet");
            assert.include(result.declarationText, "name: string");
            assert.include(result.declarationText, ": string;");
        });

        it("should emit class declarations", () => {
            const program = createTestProgram([{
                name: "/test.ts",
                content: `
                    export class Person {
                        name: string;
                        constructor(name: string) {
                            this.name = name;
                        }
                        greet(): string {
                            return "Hello " + this.name;
                        }
                    }
                `
            }]);

            const emitter = ts.declarations.createDeclarationEmitter(program, {});
            const result = emitter.emitFile(program.getSourceFile("/test.ts")!);

            assert.include(result.declarationText, "declare class Person");
            assert.include(result.declarationText, "name: string");
            assert.include(result.declarationText, "constructor");
            assert.include(result.declarationText, "greet()");
        });

        it("should emit interface declarations", () => {
            const program = createTestProgram([{
                name: "/test.ts",
                content: `
                    export interface Point {
                        x: number;
                        y: number;
                    }
                `
            }]);

            const emitter = ts.declarations.createDeclarationEmitter(program, {});
            const result = emitter.emitFile(program.getSourceFile("/test.ts")!);

            assert.include(result.declarationText, "interface Point");
            assert.include(result.declarationText, "x: number");
            assert.include(result.declarationText, "y: number");
        });

        it("should emit type alias declarations", () => {
            const program = createTestProgram([{
                name: "/test.ts",
                content: `
                    export type StringOrNumber = string | number;
                `
            }]);

            const emitter = ts.declarations.createDeclarationEmitter(program, {});
            const result = emitter.emitFile(program.getSourceFile("/test.ts")!);

            assert.include(result.declarationText, "type StringOrNumber");
            assert.include(result.declarationText, "string | number");
        });

        it("should emit enum declarations", () => {
            const program = createTestProgram([{
                name: "/test.ts",
                content: `
                    export enum Color {
                        Red = 0,
                        Green = 1,
                        Blue = 2
                    }
                `
            }]);

            const emitter = ts.declarations.createDeclarationEmitter(program, {});
            const result = emitter.emitFile(program.getSourceFile("/test.ts")!);

            assert.include(result.declarationText, "enum Color");
            assert.include(result.declarationText, "Red");
            assert.include(result.declarationText, "Green");
            assert.include(result.declarationText, "Blue");
        });
    });

    describe("namespace merging", () => {
        it("should collect namespace merges across files", () => {
            const program = createTestProgram([
                {
                    name: "/a.ts",
                    content: `
                        export namespace Utils {
                            export function foo() {}
                        }
                    `
                },
                {
                    name: "/b.ts",
                    content: `
                        export namespace Utils {
                            export function bar() {}
                        }
                    `
                }
            ]);

            const checker = program.getTypeChecker();
            const merges = ts.declarations.collectNamespaceMerges(
                program.getSourceFiles(),
                checker
            );

            const utilsMerge = merges.get("Utils");
            assert.isDefined(utilsMerge);
            assert.equal(utilsMerge!.declarations.length, 2);
        });
    });

    describe("declaration maps", () => {
        it("should generate declaration map when requested", () => {
            const program = createTestProgram([{
                name: "/test.ts",
                content: `export const foo = 42;`
            }]);

            const emitter = ts.declarations.createDeclarationEmitter(program, {
                declarationMap: true
            });
            const result = emitter.emitFile(program.getSourceFile("/test.ts")!);

            assert.isDefined(result.declarationMapText);
            assert.isDefined(result.mapPath);

            const map = JSON.parse(result.declarationMapText!);
            assert.equal(map.version, 3);
            assert.include(map.sources, "/test.ts");
        });

        it("should not generate declaration map when not requested", () => {
            const program = createTestProgram([{
                name: "/test.ts",
                content: `export const foo = 42;`
            }]);

            const emitter = ts.declarations.createDeclarationEmitter(program, {
                declarationMap: false
            });
            const result = emitter.emitFile(program.getSourceFile("/test.ts")!);

            assert.isUndefined(result.declarationMapText);
        });
    });

    describe("bundled output", () => {
        it("should bundle declarations from multiple files", () => {
            const program = createTestProgram([
                {
                    name: "/a.ts",
                    content: `export const a = 1;`
                },
                {
                    name: "/b.ts",
                    content: `export const b = 2;`
                }
            ]);

            const emitter = ts.declarations.createDeclarationEmitter(program, {
                bundledOutput: true
            });
            const result = emitter.emitBundled();

            assert.include(result.declarationText, "Bundled declarations");
            assert.include(result.declarationText, "declare const a");
            assert.include(result.declarationText, "declare const b");
        });
    });

    describe("complex patterns", () => {
        it("should handle generic functions", () => {
            const program = createTestProgram([{
                name: "/test.ts",
                content: `
                    export function identity<T>(x: T): T {
                        return x;
                    }
                `
            }]);

            const emitter = ts.declarations.createDeclarationEmitter(program, {});
            const result = emitter.emitFile(program.getSourceFile("/test.ts")!);

            assert.include(result.declarationText, "<T>");
            assert.include(result.declarationText, "x: T");
            assert.include(result.declarationText, "): T;");
        });

        it("should handle class with generics and heritage", () => {
            const program = createTestProgram([{
                name: "/test.ts",
                content: `
                    interface Disposable {
                        dispose(): void;
                    }
                    export class Container<T> implements Disposable {
                        value: T;
                        constructor(value: T) {
                            this.value = value;
                        }
                        dispose(): void {}
                    }
                `
            }]);

            const emitter = ts.declarations.createDeclarationEmitter(program, {});
            const result = emitter.emitFile(program.getSourceFile("/test.ts")!);

            assert.include(result.declarationText, "class Container<T>");
            assert.include(result.declarationText, "implements Disposable");
            assert.include(result.declarationText, "value: T");
        });

        it("should handle const enums", () => {
            const program = createTestProgram([{
                name: "/test.ts",
                content: `
                    export const enum Direction {
                        Up = 1,
                        Down = 2,
                        Left = 3,
                        Right = 4
                    }
                `
            }]);

            const emitter = ts.declarations.createDeclarationEmitter(program, {});
            const result = emitter.emitFile(program.getSourceFile("/test.ts")!);

            assert.include(result.declarationText, "const enum Direction");
        });

        it("should handle optional and readonly properties", () => {
            const program = createTestProgram([{
                name: "/test.ts",
                content: `
                    export interface Config {
                        readonly id: string;
                        name?: string;
                    }
                `
            }]);

            const emitter = ts.declarations.createDeclarationEmitter(program, {});
            const result = emitter.emitFile(program.getSourceFile("/test.ts")!);

            assert.include(result.declarationText, "readonly id");
            assert.include(result.declarationText, "name?");
        });
    });

    describe("helper functions", () => {
        it("emitDeclarations should emit all files", () => {
            const program = createTestProgram([
                { name: "/a.ts", content: `export const a = 1;` },
                { name: "/b.ts", content: `export const b = 2;` }
            ]);

            const results = ts.declarations.emitDeclarations(program);

            assert.equal(results.length, 2);
        });

        it("emitBundledDeclarations should emit bundled output", () => {
            const program = createTestProgram([
                { name: "/a.ts", content: `export const a = 1;` },
                { name: "/b.ts", content: `export const b = 2;` }
            ]);

            const result = ts.declarations.emitBundledDeclarations(program);

            assert.include(result.declarationText, "declare const a");
            assert.include(result.declarationText, "declare const b");
        });

        it("getDeclarationsForFile should analyze without emitting", () => {
            const program = createTestProgram([{
                name: "/test.ts",
                content: `export const foo = 42;`
            }]);

            const sourceFile = program.getSourceFile("/test.ts")!;
            const collected = ts.declarations.getDeclarationsForFile(sourceFile, program);

            assert.isArray(collected.exports);
            assert.isAbove(collected.exports.length, 0);
        });

        it("symbolNeedsTypeAnnotation should check annotation requirements", () => {
            const program = createTestProgram([{
                name: "/test.ts",
                content: `
                    export const typed: number = 5;
                    export const untyped = { x: 1 };
                `
            }]);

            const sourceFile = program.getSourceFile("/test.ts")!;
            const checker = program.getTypeChecker();

            // Find symbols
            for (const statement of sourceFile.statements) {
                if (ts.isVariableStatement(statement)) {
                    for (const decl of statement.declarationList.declarations) {
                        if (ts.isIdentifier(decl.name)) {
                            const symbol = checker.getSymbolAtLocation(decl.name);
                            if (symbol) {
                                const needsAnnotation = ts.declarations.symbolNeedsTypeAnnotation(
                                    symbol,
                                    checker
                                );
                                if (decl.name.text === "typed") {
                                    assert.isFalse(needsAnnotation);
                                } else if (decl.name.text === "untyped") {
                                    assert.isTrue(needsAnnotation);
                                }
                            }
                        }
                    }
                }
            }
        });
    });
});
