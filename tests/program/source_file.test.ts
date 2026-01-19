/**
 * Tests for OwnedSourceFile.
 */

namespace ts {
    describe("OwnedSourceFile", () => {
        describe("construction", () => {
            it("should create from config", () => {
                const owned = createOwnedSourceFile(
                    "/test/file.ts",
                    "const x = 1;",
                    ScriptTarget.ES2020
                );

                assert.equal(owned.fileName, "/test/file.ts");
                assert.equal(owned.text, "const x = 1;");
                assert.equal(owned.languageVersion, ScriptTarget.ES2020);
            });

            it("should detect script kind from file name", () => {
                const tsFile = createOwnedSourceFile("/test/file.ts", "", ScriptTarget.Latest);
                assert.equal(tsFile.scriptKind, ScriptKind.TS);

                const tsxFile = createOwnedSourceFile("/test/file.tsx", "", ScriptTarget.Latest);
                assert.equal(tsxFile.scriptKind, ScriptKind.TSX);

                const jsFile = createOwnedSourceFile("/test/file.js", "", ScriptTarget.Latest);
                assert.equal(jsFile.scriptKind, ScriptKind.JS);

                const jsxFile = createOwnedSourceFile("/test/file.jsx", "", ScriptTarget.Latest);
                assert.equal(jsxFile.scriptKind, ScriptKind.JSX);
            });

            it("should detect declaration files", () => {
                const dtsFile = createOwnedSourceFile("/test/file.d.ts", "", ScriptTarget.Latest);
                assert.isTrue(dtsFile.isDeclarationFile);

                const tsFile = createOwnedSourceFile("/test/file.ts", "", ScriptTarget.Latest);
                assert.isFalse(tsFile.isDeclarationFile);
            });

            it("should strip BOM from text", () => {
                const textWithBom = "\uFEFFconst x = 1;";
                const owned = createOwnedSourceFile("/test/file.ts", textWithBom, ScriptTarget.Latest);

                assert.equal(owned.text, "const x = 1;");
                assert.isTrue(owned.metadata.hasBom);
            });

            it("should normalize file paths", () => {
                const owned = createOwnedSourceFile(
                    "/test/../test/./file.ts",
                    "",
                    ScriptTarget.Latest
                );

                assert.equal(owned.fileName, "/test/file.ts");
            });
        });

        describe("text access", () => {
            it("should provide substring access", () => {
                const owned = createOwnedSourceFile(
                    "/test/file.ts",
                    "const x = 1;",
                    ScriptTarget.Latest
                );

                assert.equal(owned.getSubstring(0, 5), "const");
                assert.equal(owned.getSubstring(6, 7), "x");
            });

            it("should provide character code access", () => {
                const owned = createOwnedSourceFile(
                    "/test/file.ts",
                    "abc",
                    ScriptTarget.Latest
                );

                assert.equal(owned.charCodeAt(0), CharacterCodes.a);
                assert.equal(owned.charCodeAt(1), CharacterCodes.b);
                assert.equal(owned.charCodeAt(2), CharacterCodes.c);
            });
        });

        describe("parsing", () => {
            it("should lazily parse source file", () => {
                const owned = createOwnedSourceFile(
                    "/test/file.ts",
                    "const x = 1;",
                    ScriptTarget.Latest
                );

                assert.isFalse(owned.isParsed());

                const sourceFile = owned.getSourceFile();

                assert.isTrue(owned.isParsed());
                assert.isDefined(sourceFile);
                assert.equal(sourceFile.fileName, "/test/file.ts");
            });

            it("should cache parsed source file", () => {
                const owned = createOwnedSourceFile(
                    "/test/file.ts",
                    "const x = 1;",
                    ScriptTarget.Latest
                );

                const sf1 = owned.getSourceFile();
                const sf2 = owned.getSourceFile();

                assert.strictEqual(sf1, sf2);
            });

            it("should collect module references", () => {
                const owned = createOwnedSourceFile(
                    "/test/file.ts",
                    `import { foo } from './foo';
                     import type { Bar } from './bar';
                     export { baz } from './baz';
                     const dynamic = import('./dynamic');`,
                    ScriptTarget.Latest
                );

                const refs = owned.getModuleReferences();

                assert.isTrue(refs.length >= 3);

                const importRef = refs.find(r => r.specifier === "./foo");
                assert.isDefined(importRef);
                assert.equal(importRef!.kind, ModuleReferenceKind.Import);
                assert.isFalse(importRef!.isTypeOnly);

                const typeImportRef = refs.find(r => r.specifier === "./bar");
                assert.isDefined(typeImportRef);
                assert.isTrue(typeImportRef!.isTypeOnly);

                const exportRef = refs.find(r => r.specifier === "./baz");
                assert.isDefined(exportRef);
                assert.equal(exportRef!.kind, ModuleReferenceKind.Export);
            });
        });

        describe("line mapping", () => {
            it("should compute line starts", () => {
                const owned = createOwnedSourceFile(
                    "/test/file.ts",
                    "line1\nline2\nline3",
                    ScriptTarget.Latest
                );

                const lineStarts = owned.getLineStarts();

                assert.equal(lineStarts.length, 3);
                assert.equal(lineStarts[0], 0);
                assert.equal(lineStarts[1], 6);
                assert.equal(lineStarts[2], 12);
            });

            it("should convert position to line and character", () => {
                const owned = createOwnedSourceFile(
                    "/test/file.ts",
                    "line1\nline2\nline3",
                    ScriptTarget.Latest
                );

                const pos1 = owned.getLineAndCharacterOfPosition(0);
                assert.equal(pos1.line, 0);
                assert.equal(pos1.character, 0);

                const pos2 = owned.getLineAndCharacterOfPosition(6);
                assert.equal(pos2.line, 1);
                assert.equal(pos2.character, 0);

                const pos3 = owned.getLineAndCharacterOfPosition(8);
                assert.equal(pos3.line, 1);
                assert.equal(pos3.character, 2);
            });
        });

        describe("invalidation", () => {
            it("should clear cached data on invalidate", () => {
                const owned = createOwnedSourceFile(
                    "/test/file.ts",
                    "const x = 1;",
                    ScriptTarget.Latest
                );

                // Force parsing
                owned.getSourceFile();
                owned.getLineStarts();
                owned.getModuleReferences();

                assert.isTrue(owned.isParsed());

                owned.invalidate();

                assert.isFalse(owned.isParsed());
            });
        });

        describe("metadata", () => {
            it("should provide correct metadata", () => {
                const owned = createOwnedSourceFile(
                    "/test/file.ts",
                    "const x = 1;",
                    ScriptTarget.ES2020
                );

                const metadata = owned.metadata;

                assert.equal(metadata.normalizedFileName, "/test/file.ts");
                assert.equal(metadata.languageVersion, ScriptTarget.ES2020);
                assert.equal(metadata.scriptKind, ScriptKind.TS);
                assert.isFalse(metadata.isDeclarationFile);
                assert.equal(metadata.charLength, 12);
                assert.equal(metadata.origin, SourceFileOrigin.Disk);
            });
        });
    });
}
