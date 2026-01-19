/**
 * SourceFile module - Owns source text for zero-copy parsing.
 *
 * This module provides a SourceFile struct that owns the source text,
 * allowing Scanner and Parser to borrow from it without copying.
 */

namespace ts {
    /**
     * Represents the origin of a source file.
     */
    export const enum SourceFileOrigin {
        /** File loaded from disk */
        Disk = 0,
        /** File provided in-memory (e.g., for testing or IDE) */
        InMemory = 1,
        /** Virtual file synthesized during compilation */
        Virtual = 2,
        /** Library file from lib.d.ts */
        Library = 3,
    }

    /**
     * Configuration for creating a SourceFile.
     */
    export interface SourceFileConfig {
        /** The file path (normalized) */
        fileName: string;
        /** The source text content */
        text: string;
        /** Language version for parsing */
        languageVersion: ScriptTarget;
        /** Whether to set parent nodes during parsing */
        setParentNodes?: boolean;
        /** Script kind (TS, TSX, JS, etc.) */
        scriptKind?: ScriptKind;
        /** Origin of the file */
        origin?: SourceFileOrigin;
    }

    /**
     * Metadata about a source file that doesn't change after parsing.
     */
    export interface SourceFileMetadata {
        /** Original file path as provided */
        readonly originalFileName: string;
        /** Normalized file path */
        readonly normalizedFileName: string;
        /** The script kind */
        readonly scriptKind: ScriptKind;
        /** The language version used for parsing */
        readonly languageVersion: ScriptTarget;
        /** Origin of the source */
        readonly origin: SourceFileOrigin;
        /** File length in bytes */
        readonly byteLength: number;
        /** File length in characters */
        readonly charLength: number;
        /** Whether this is a declaration file (.d.ts) */
        readonly isDeclarationFile: boolean;
        /** Whether this has a BOM (byte order mark) */
        readonly hasBom: boolean;
    }

    /**
     * Reference to an imported or required module.
     */
    export interface ModuleReference {
        /** The module specifier string */
        readonly specifier: string;
        /** The position in source where the reference appears */
        readonly pos: number;
        /** The end position */
        readonly end: number;
        /** Whether this is a type-only import */
        readonly isTypeOnly: boolean;
        /** The kind of reference (import, require, etc.) */
        readonly kind: ModuleReferenceKind;
    }

    /**
     * Kind of module reference.
     */
    export const enum ModuleReferenceKind {
        Import = 0,
        Export = 1,
        Require = 2,
        ImportType = 3,
        /// <reference types="..." />
        TripleSlashTypes = 4,
        /// <reference path="..." />
        TripleSlashPath = 5,
        /// <reference lib="..." />
        TripleSlashLib = 6,
    }

    /**
     * Owns the source text and provides a stable reference for parsing.
     *
     * This is designed for zero-copy parsing where the Scanner and Parser
     * can borrow slices of the text without allocation.
     */
    export class OwnedSourceFile {
        /** The owned source text - never changes after construction */
        private readonly _text: string;

        /** Parsed AST node (cached after first parse) */
        private _sourceFile: SourceFile | undefined;

        /** Metadata about this file */
        private readonly _metadata: SourceFileMetadata;

        /** Cached line starts for position calculations */
        private _lineStarts: readonly number[] | undefined;

        /** Module references found in this file */
        private _moduleReferences: ModuleReference[] | undefined;

        constructor(config: SourceFileConfig) {
            // Detect and strip BOM if present
            const hasBom = config.text.charCodeAt(0) === CharacterCodes.byteOrderMark;
            this._text = hasBom ? config.text.slice(1) : config.text;

            const normalizedFileName = normalizePath(config.fileName);
            const scriptKind = config.scriptKind ?? getScriptKindFromFileName(config.fileName);

            this._metadata = {
                originalFileName: config.fileName,
                normalizedFileName,
                scriptKind,
                languageVersion: config.languageVersion,
                origin: config.origin ?? SourceFileOrigin.Disk,
                byteLength: new TextEncoder().encode(this._text).length,
                charLength: this._text.length,
                isDeclarationFile: isDeclarationFileName(normalizedFileName),
                hasBom,
            };
        }

        /**
         * Get the source text. This is the canonical text after BOM stripping.
         */
        public get text(): string {
            return this._text;
        }

        /**
         * Get metadata about this source file.
         */
        public get metadata(): SourceFileMetadata {
            return this._metadata;
        }

        /**
         * Get the file name (normalized path).
         */
        public get fileName(): string {
            return this._metadata.normalizedFileName;
        }

        /**
         * Get the script kind.
         */
        public get scriptKind(): ScriptKind {
            return this._metadata.scriptKind;
        }

        /**
         * Get the language version.
         */
        public get languageVersion(): ScriptTarget {
            return this._metadata.languageVersion;
        }

        /**
         * Check if this is a declaration file.
         */
        public get isDeclarationFile(): boolean {
            return this._metadata.isDeclarationFile;
        }

        /**
         * Get a substring of the source text.
         * This is a zero-copy operation as JS strings are immutable.
         */
        public getSubstring(start: number, end: number): string {
            return this._text.substring(start, end);
        }

        /**
         * Get the character at a position.
         */
        public charCodeAt(position: number): number {
            return this._text.charCodeAt(position);
        }

        /**
         * Get the parsed SourceFile AST.
         * Lazily parses on first access.
         */
        public getSourceFile(setParentNodes?: boolean): SourceFile {
            if (!this._sourceFile) {
                this._sourceFile = createSourceFile(
                    this._metadata.normalizedFileName,
                    this._text,
                    this._metadata.languageVersion,
                    setParentNodes,
                    this._metadata.scriptKind
                );
            }
            return this._sourceFile;
        }

        /**
         * Check if the file has been parsed.
         */
        public isParsed(): boolean {
            return this._sourceFile !== undefined;
        }

        /**
         * Get line starts for position-to-line calculations.
         */
        public getLineStarts(): readonly number[] {
            if (!this._lineStarts) {
                this._lineStarts = computeLineStarts(this._text);
            }
            return this._lineStarts;
        }

        /**
         * Convert a position to line and character.
         */
        public getLineAndCharacterOfPosition(position: number): LineAndCharacter {
            return computeLineAndCharacterOfPosition(this.getLineStarts(), position);
        }

        /**
         * Get the position from line and character.
         */
        public getPositionOfLineAndCharacter(line: number, character: number): number {
            return computePositionOfLineAndCharacter(this.getLineStarts(), line, character, this._text);
        }

        /**
         * Get module references from this file.
         * Collects import/export/require statements.
         */
        public getModuleReferences(): readonly ModuleReference[] {
            if (this._moduleReferences === undefined) {
                this._moduleReferences = this.collectModuleReferences();
            }
            return this._moduleReferences;
        }

        /**
         * Collect all module references from the parsed source file.
         */
        private collectModuleReferences(): ModuleReference[] {
            const sourceFile = this.getSourceFile();
            const references: ModuleReference[] = [];

            // Collect triple-slash references
            if (sourceFile.referencedFiles) {
                for (const ref of sourceFile.referencedFiles) {
                    references.push({
                        specifier: ref.fileName,
                        pos: ref.pos,
                        end: ref.end,
                        isTypeOnly: false,
                        kind: ModuleReferenceKind.TripleSlashPath,
                    });
                }
            }

            if (sourceFile.typeReferenceDirectives) {
                for (const ref of sourceFile.typeReferenceDirectives) {
                    references.push({
                        specifier: ref.fileName,
                        pos: ref.pos,
                        end: ref.end,
                        isTypeOnly: true,
                        kind: ModuleReferenceKind.TripleSlashTypes,
                    });
                }
            }

            if (sourceFile.libReferenceDirectives) {
                for (const ref of sourceFile.libReferenceDirectives) {
                    references.push({
                        specifier: ref.fileName,
                        pos: ref.pos,
                        end: ref.end,
                        isTypeOnly: false,
                        kind: ModuleReferenceKind.TripleSlashLib,
                    });
                }
            }

            // Collect import/export statements
            for (const statement of sourceFile.statements) {
                this.collectModuleReferencesFromNode(statement, references);
            }

            return references;
        }

        /**
         * Recursively collect module references from AST nodes.
         */
        private collectModuleReferencesFromNode(node: Node, references: ModuleReference[]): void {
            switch (node.kind) {
                case SyntaxKind.ImportDeclaration: {
                    const importDecl = node as ImportDeclaration;
                    if (isStringLiteral(importDecl.moduleSpecifier)) {
                        references.push({
                            specifier: importDecl.moduleSpecifier.text,
                            pos: importDecl.moduleSpecifier.pos,
                            end: importDecl.moduleSpecifier.end,
                            isTypeOnly: importDecl.importClause?.isTypeOnly ?? false,
                            kind: ModuleReferenceKind.Import,
                        });
                    }
                    break;
                }

                case SyntaxKind.ExportDeclaration: {
                    const exportDecl = node as ExportDeclaration;
                    if (exportDecl.moduleSpecifier && isStringLiteral(exportDecl.moduleSpecifier)) {
                        references.push({
                            specifier: exportDecl.moduleSpecifier.text,
                            pos: exportDecl.moduleSpecifier.pos,
                            end: exportDecl.moduleSpecifier.end,
                            isTypeOnly: exportDecl.isTypeOnly,
                            kind: ModuleReferenceKind.Export,
                        });
                    }
                    break;
                }

                case SyntaxKind.ImportEqualsDeclaration: {
                    const importEquals = node as ImportEqualsDeclaration;
                    if (importEquals.moduleReference.kind === SyntaxKind.ExternalModuleReference) {
                        const extModRef = importEquals.moduleReference as ExternalModuleReference;
                        if (isStringLiteral(extModRef.expression)) {
                            references.push({
                                specifier: extModRef.expression.text,
                                pos: extModRef.expression.pos,
                                end: extModRef.expression.end,
                                isTypeOnly: importEquals.isTypeOnly,
                                kind: ModuleReferenceKind.Require,
                            });
                        }
                    }
                    break;
                }

                case SyntaxKind.CallExpression: {
                    const call = node as CallExpression;
                    // Handle require('module') and import('module')
                    if (call.arguments.length === 1 && isStringLiteral(call.arguments[0])) {
                        if (call.expression.kind === SyntaxKind.Identifier) {
                            const id = call.expression as Identifier;
                            if (id.escapedText === "require") {
                                const arg = call.arguments[0] as StringLiteral;
                                references.push({
                                    specifier: arg.text,
                                    pos: arg.pos,
                                    end: arg.end,
                                    isTypeOnly: false,
                                    kind: ModuleReferenceKind.Require,
                                });
                            }
                        }
                        else if (call.expression.kind === SyntaxKind.ImportKeyword) {
                            const arg = call.arguments[0] as StringLiteral;
                            references.push({
                                specifier: arg.text,
                                pos: arg.pos,
                                end: arg.end,
                                isTypeOnly: false,
                                kind: ModuleReferenceKind.Import,
                            });
                        }
                    }
                    break;
                }

                case SyntaxKind.ImportType: {
                    const importType = node as ImportTypeNode;
                    if (isLiteralTypeNode(importType.argument) &&
                        isStringLiteral(importType.argument.literal)) {
                        references.push({
                            specifier: importType.argument.literal.text,
                            pos: importType.argument.literal.pos,
                            end: importType.argument.literal.end,
                            isTypeOnly: true,
                            kind: ModuleReferenceKind.ImportType,
                        });
                    }
                    break;
                }
            }

            // Recurse into children for nested call expressions
            forEachChild(node, child => this.collectModuleReferencesFromNode(child, references));
        }

        /**
         * Invalidate the parsed AST (e.g., for incremental updates).
         */
        public invalidate(): void {
            this._sourceFile = undefined;
            this._lineStarts = undefined;
            this._moduleReferences = undefined;
        }
    }

    /**
     * Detect the script kind from a file name.
     */
    function getScriptKindFromFileName(fileName: string): ScriptKind {
        const ext = fileName.substring(fileName.lastIndexOf("."));
        switch (ext.toLowerCase()) {
            case Extension.Js:
                return ScriptKind.JS;
            case Extension.Jsx:
                return ScriptKind.JSX;
            case Extension.Ts:
                return ScriptKind.TS;
            case Extension.Tsx:
                return ScriptKind.TSX;
            case Extension.Json:
                return ScriptKind.JSON;
            default:
                return ScriptKind.Unknown;
        }
    }

    /**
     * Check if a file is a declaration file based on its name.
     */
    function isDeclarationFileName(fileName: string): boolean {
        return fileExtensionIs(fileName, Extension.Dts) ||
               fileExtensionIs(fileName, Extension.Dmts) ||
               fileExtensionIs(fileName, Extension.Dcts);
    }

    /**
     * Create an OwnedSourceFile from a file path and content.
     */
    export function createOwnedSourceFile(
        fileName: string,
        text: string,
        languageVersion: ScriptTarget,
        setParentNodes?: boolean,
        scriptKind?: ScriptKind
    ): OwnedSourceFile {
        return new OwnedSourceFile({
            fileName,
            text,
            languageVersion,
            setParentNodes,
            scriptKind,
        });
    }
}
