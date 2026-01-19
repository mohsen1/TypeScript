/**
 * Declaration Emitter
 *
 * Generates .d.ts files from collected declarations.
 * Handles type annotations, imports, and declaration maps.
 */

/* @internal */
namespace ts.declarations {
    /**
     * Create a declaration emitter
     */
    export function createDeclarationEmitter(
        program: Program,
        options: DeclarationEmitOptions
    ): DeclarationEmitter {
        return new DeclarationEmitterImpl(program, options);
    }

    /**
     * Declaration emitter interface
     */
    export interface DeclarationEmitter {
        /**
         * Emit declaration files for all source files
         */
        emitAll(): DeclarationFileResult[];

        /**
         * Emit declaration file for a single source file
         */
        emitFile(sourceFile: SourceFile): DeclarationFileResult;

        /**
         * Emit bundled declaration file
         */
        emitBundled(): BundledDeclarationResult;
    }

    /**
     * Declaration emitter implementation
     */
    class DeclarationEmitterImpl implements DeclarationEmitter {
        private checker: TypeChecker;
        private host: CompilerHost;
        private nodeFactory: NodeFactory;

        constructor(
            private program: Program,
            private options: DeclarationEmitOptions
        ) {
            this.checker = program.getTypeChecker();
            this.host = program.getCompilerHost()!;
            this.nodeFactory = factory;
        }

        emitAll(): DeclarationFileResult[] {
            const results: DeclarationFileResult[] = [];
            const sourceFiles = this.program.getSourceFiles();

            for (const sourceFile of sourceFiles) {
                if (sourceFile.isDeclarationFile) continue;
                if (this.shouldSkipFile(sourceFile)) continue;

                results.push(this.emitFile(sourceFile));
            }

            return results;
        }

        emitFile(sourceFile: SourceFile): DeclarationFileResult {
            const diagnostics: Diagnostic[] = [];

            // Collect declarations
            const collected = collectDeclarations(sourceFile, this.checker, this.options);

            // Generate declaration text
            const declarationText = this.generateDeclarationText(sourceFile, collected, diagnostics);

            // Calculate output path
            const outputPath = this.getDeclarationOutputPath(sourceFile);

            // Generate declaration map if requested
            let declarationMapText: string | undefined;
            let mapPath: string | undefined;
            if (this.options.declarationMap) {
                const mapResult = this.generateDeclarationMap(sourceFile, declarationText);
                declarationMapText = mapResult.text;
                mapPath = mapResult.path;
            }

            return {
                declarationText,
                declarationMapText,
                outputPath,
                mapPath,
                diagnostics,
            };
        }

        emitBundled(): BundledDeclarationResult {
            const diagnostics: Diagnostic[] = [];
            const moduleAugmentations: ModuleAugmentation[] = [];
            const globalAugmentations: GlobalAugmentation[] = [];
            const allDeclarations: string[] = [];

            const sourceFiles = this.program.getSourceFiles();

            // Collect all namespace merges
            const namespaceMerges = collectNamespaceMerges(sourceFiles, this.checker);

            // Process each source file
            for (const sourceFile of sourceFiles) {
                if (sourceFile.isDeclarationFile) continue;
                if (this.shouldSkipFile(sourceFile)) continue;

                const collected = collectDeclarations(sourceFile, this.checker, this.options);
                const fileDeclarations = this.generateDeclarationText(sourceFile, collected, diagnostics);

                allDeclarations.push(`// From: ${sourceFile.fileName}`);
                allDeclarations.push(fileDeclarations);
            }

            // Generate bundled output
            const declarationText = this.bundleDeclarations(allDeclarations, namespaceMerges);

            // Generate declaration map if requested
            let declarationMapText: string | undefined;
            if (this.options.declarationMap) {
                declarationMapText = this.generateBundledDeclarationMap(sourceFiles);
            }

            return {
                declarationText,
                declarationMapText,
                moduleAugmentations,
                globalAugmentations,
                diagnostics,
            };
        }

        /**
         * Generate declaration text for a source file
         */
        private generateDeclarationText(
            sourceFile: SourceFile,
            collected: CollectedDeclarations,
            diagnostics: Diagnostic[]
        ): string {
            const writer = new DeclarationWriter(this.options.removeComments ?? false);

            // Write file header
            writer.writeLine("// Generated declaration file");
            writer.writeLine(`// Source: ${sourceFile.fileName}`);
            writer.writeLine("");

            // Write type reference directives
            for (const ref of collected.typeReferences) {
                writer.writeLine(`/// <reference types="${ref}" />`);
            }

            // Write file reference directives
            for (const ref of collected.fileReferences) {
                writer.writeLine(`/// <reference path="${ref}" />`);
            }

            if (collected.typeReferences.length > 0 || collected.fileReferences.length > 0) {
                writer.writeLine("");
            }

            // Write imports
            for (const importInfo of collected.imports) {
                this.writeImport(writer, importInfo);
            }

            if (collected.imports.length > 0) {
                writer.writeLine("");
            }

            // Write declarations
            for (const exp of collected.exports) {
                this.writeDeclaration(writer, exp, sourceFile, diagnostics);
            }

            return writer.toString();
        }

        /**
         * Write an import statement
         */
        private writeImport(writer: DeclarationWriter, importInfo: ImportInfo): void {
            const typeOnly = importInfo.isTypeOnly ? "type " : "";
            const names: string[] = [];

            for (const imported of importInfo.importedNames) {
                if (imported.isDefault) {
                    names.push(imported.alias || imported.name);
                } else if (imported.isNamespace) {
                    names.push(`* as ${imported.alias || imported.name}`);
                } else {
                    const alias = imported.alias ? ` as ${imported.alias}` : "";
                    names.push(`${imported.name}${alias}`);
                }
            }

            writer.writeLine(`import ${typeOnly}{ ${names.join(", ")} } from "${importInfo.moduleSpecifier}";`);
        }

        /**
         * Write a declaration
         */
        private writeDeclaration(
            writer: DeclarationWriter,
            info: SymbolVisibilityInfo,
            sourceFile: SourceFile,
            diagnostics: Diagnostic[]
        ): void {
            const symbol = info.symbol;
            const declarations = symbol.declarations;
            if (!declarations || declarations.length === 0) return;

            for (const decl of declarations) {
                if (getSourceFileOfNode(decl) !== sourceFile) continue;

                this.writeDeclarationNode(writer, decl, info, diagnostics);
            }
        }

        /**
         * Write a declaration node
         */
        private writeDeclarationNode(
            writer: DeclarationWriter,
            decl: Declaration,
            info: SymbolVisibilityInfo,
            diagnostics: Diagnostic[]
        ): void {
            // Determine if we need to add export keyword
            const isExported = info.visibility === ExportVisibility.Exported ||
                               info.visibility === ExportVisibility.ExportedNeedsAnnotation;

            if (isVariableDeclaration(decl)) {
                this.writeVariableDeclaration(writer, decl, info, isExported, diagnostics);
            } else if (isFunctionDeclaration(decl)) {
                this.writeFunctionDeclaration(writer, decl, info, isExported, diagnostics);
            } else if (isClassDeclaration(decl)) {
                this.writeClassDeclaration(writer, decl, isExported, diagnostics);
            } else if (isInterfaceDeclaration(decl)) {
                this.writeInterfaceDeclaration(writer, decl, isExported);
            } else if (isTypeAliasDeclaration(decl)) {
                this.writeTypeAliasDeclaration(writer, decl, isExported);
            } else if (isEnumDeclaration(decl)) {
                this.writeEnumDeclaration(writer, decl, isExported);
            } else if (isModuleDeclaration(decl)) {
                this.writeModuleDeclaration(writer, decl, isExported);
            }
        }

        /**
         * Write variable declaration
         */
        private writeVariableDeclaration(
            writer: DeclarationWriter,
            decl: VariableDeclaration,
            info: SymbolVisibilityInfo,
            isExported: boolean,
            diagnostics: Diagnostic[]
        ): void {
            const exportKeyword = isExported ? "export " : "";
            const constOrLet = this.getVariableDeclarationKind(decl);
            const name = isIdentifier(decl.name) ? decl.name.text : "/* binding pattern */";

            let typeString: string;
            if (decl.type) {
                typeString = this.typeNodeToString(decl.type);
            } else if (info.inferredType) {
                typeString = this.checker.typeToString(info.inferredType);
            } else {
                typeString = "any";
                diagnostics.push(createDiagnosticForNode(
                    decl,
                    Diagnostics.Variable_0_implicitly_has_an_1_type,
                    name,
                    "any"
                ));
            }

            writer.writeLine(`${exportKeyword}declare ${constOrLet} ${name}: ${typeString};`);
        }

        /**
         * Get the variable declaration kind (const, let, var)
         */
        private getVariableDeclarationKind(decl: VariableDeclaration): string {
            const parent = decl.parent;
            if (isVariableDeclarationList(parent)) {
                if (parent.flags & NodeFlags.Const) return "const";
                if (parent.flags & NodeFlags.Let) return "let";
            }
            return "var";
        }

        /**
         * Write function declaration
         */
        private writeFunctionDeclaration(
            writer: DeclarationWriter,
            decl: FunctionDeclaration,
            info: SymbolVisibilityInfo,
            isExported: boolean,
            diagnostics: Diagnostic[]
        ): void {
            if (!decl.name) return;

            const exportKeyword = isExported ? "export " : "";
            const name = decl.name.text;

            // Type parameters
            const typeParams = this.writeTypeParameters(decl.typeParameters);

            // Parameters
            const params = this.writeParameters(decl.parameters, diagnostics);

            // Return type
            let returnType: string;
            if (decl.type) {
                returnType = this.typeNodeToString(decl.type);
            } else if (info.inferredType) {
                const signature = this.checker.getSignaturesOfType(info.inferredType, SignatureKind.Call)[0];
                if (signature) {
                    returnType = this.checker.typeToString(this.checker.getReturnTypeOfSignature(signature));
                } else {
                    returnType = "void";
                }
            } else {
                returnType = "void";
            }

            writer.writeLine(`${exportKeyword}declare function ${name}${typeParams}(${params}): ${returnType};`);
        }

        /**
         * Write class declaration
         */
        private writeClassDeclaration(
            writer: DeclarationWriter,
            decl: ClassDeclaration,
            isExported: boolean,
            diagnostics: Diagnostic[]
        ): void {
            if (!decl.name) return;

            const exportKeyword = isExported ? "export " : "";
            const name = decl.name.text;
            const typeParams = this.writeTypeParameters(decl.typeParameters);

            // Extends clause
            let extendsClause = "";
            if (decl.heritageClauses) {
                for (const clause of decl.heritageClauses) {
                    if (clause.token === SyntaxKind.ExtendsKeyword) {
                        extendsClause = ` extends ${clause.types.map(t => this.typeNodeToString(t)).join(", ")}`;
                    }
                }
            }

            // Implements clause
            let implementsClause = "";
            if (decl.heritageClauses) {
                for (const clause of decl.heritageClauses) {
                    if (clause.token === SyntaxKind.ImplementsKeyword) {
                        implementsClause = ` implements ${clause.types.map(t => this.typeNodeToString(t)).join(", ")}`;
                    }
                }
            }

            writer.writeLine(`${exportKeyword}declare class ${name}${typeParams}${extendsClause}${implementsClause} {`);
            writer.indent();

            // Write members
            for (const member of decl.members) {
                this.writeClassMember(writer, member, diagnostics);
            }

            writer.dedent();
            writer.writeLine("}");
        }

        /**
         * Write class member
         */
        private writeClassMember(
            writer: DeclarationWriter,
            member: ClassElement,
            diagnostics: Diagnostic[]
        ): void {
            if (isConstructorDeclaration(member)) {
                const params = this.writeParameters(member.parameters, diagnostics);
                writer.writeLine(`constructor(${params});`);
            } else if (isPropertyDeclaration(member)) {
                const isStatic = hasSyntacticModifier(member, ModifierFlags.Static) ? "static " : "";
                const isReadonly = hasSyntacticModifier(member, ModifierFlags.Readonly) ? "readonly " : "";
                const name = isIdentifier(member.name) ? member.name.text : "/* computed */";
                const optional = member.questionToken ? "?" : "";
                const type = member.type
                    ? this.typeNodeToString(member.type)
                    : this.checker.typeToString(this.checker.getTypeAtLocation(member));
                writer.writeLine(`${isStatic}${isReadonly}${name}${optional}: ${type};`);
            } else if (isMethodDeclaration(member)) {
                const isStatic = hasSyntacticModifier(member, ModifierFlags.Static) ? "static " : "";
                const name = isIdentifier(member.name) ? member.name.text : "/* computed */";
                const typeParams = this.writeTypeParameters(member.typeParameters);
                const params = this.writeParameters(member.parameters, diagnostics);
                const returnType = member.type
                    ? this.typeNodeToString(member.type)
                    : "void";
                writer.writeLine(`${isStatic}${name}${typeParams}(${params}): ${returnType};`);
            } else if (isGetAccessorDeclaration(member)) {
                const isStatic = hasSyntacticModifier(member, ModifierFlags.Static) ? "static " : "";
                const name = isIdentifier(member.name) ? member.name.text : "/* computed */";
                const returnType = member.type
                    ? this.typeNodeToString(member.type)
                    : this.checker.typeToString(this.checker.getTypeAtLocation(member));
                writer.writeLine(`${isStatic}get ${name}(): ${returnType};`);
            } else if (isSetAccessorDeclaration(member)) {
                const isStatic = hasSyntacticModifier(member, ModifierFlags.Static) ? "static " : "";
                const name = isIdentifier(member.name) ? member.name.text : "/* computed */";
                const params = this.writeParameters(member.parameters, diagnostics);
                writer.writeLine(`${isStatic}set ${name}(${params});`);
            }
        }

        /**
         * Write interface declaration
         */
        private writeInterfaceDeclaration(
            writer: DeclarationWriter,
            decl: InterfaceDeclaration,
            isExported: boolean
        ): void {
            const exportKeyword = isExported ? "export " : "";
            const name = decl.name.text;
            const typeParams = this.writeTypeParameters(decl.typeParameters);

            // Extends clause
            let extendsClause = "";
            if (decl.heritageClauses) {
                for (const clause of decl.heritageClauses) {
                    extendsClause = ` extends ${clause.types.map(t => this.typeNodeToString(t)).join(", ")}`;
                }
            }

            writer.writeLine(`${exportKeyword}interface ${name}${typeParams}${extendsClause} {`);
            writer.indent();

            for (const member of decl.members) {
                this.writeInterfaceMember(writer, member);
            }

            writer.dedent();
            writer.writeLine("}");
        }

        /**
         * Write interface member
         */
        private writeInterfaceMember(writer: DeclarationWriter, member: TypeElement): void {
            if (isPropertySignature(member)) {
                const name = isIdentifier(member.name) ? member.name.text : "/* computed */";
                const optional = member.questionToken ? "?" : "";
                const readonly = member.modifiers?.some(m => m.kind === SyntaxKind.ReadonlyKeyword) ? "readonly " : "";
                const type = member.type ? this.typeNodeToString(member.type) : "any";
                writer.writeLine(`${readonly}${name}${optional}: ${type};`);
            } else if (isMethodSignature(member)) {
                const name = isIdentifier(member.name) ? member.name.text : "/* computed */";
                const typeParams = this.writeTypeParameters(member.typeParameters);
                const params = member.parameters.map(p => this.parameterToString(p)).join(", ");
                const returnType = member.type ? this.typeNodeToString(member.type) : "any";
                writer.writeLine(`${name}${typeParams}(${params}): ${returnType};`);
            } else if (isIndexSignatureDeclaration(member)) {
                const params = member.parameters.map(p => this.parameterToString(p)).join(", ");
                const type = member.type ? this.typeNodeToString(member.type) : "any";
                writer.writeLine(`[${params}]: ${type};`);
            } else if (isCallSignatureDeclaration(member)) {
                const typeParams = this.writeTypeParameters(member.typeParameters);
                const params = member.parameters.map(p => this.parameterToString(p)).join(", ");
                const returnType = member.type ? this.typeNodeToString(member.type) : "any";
                writer.writeLine(`${typeParams}(${params}): ${returnType};`);
            }
        }

        /**
         * Write type alias declaration
         */
        private writeTypeAliasDeclaration(
            writer: DeclarationWriter,
            decl: TypeAliasDeclaration,
            isExported: boolean
        ): void {
            const exportKeyword = isExported ? "export " : "";
            const name = decl.name.text;
            const typeParams = this.writeTypeParameters(decl.typeParameters);
            const type = this.typeNodeToString(decl.type);

            writer.writeLine(`${exportKeyword}type ${name}${typeParams} = ${type};`);
        }

        /**
         * Write enum declaration
         */
        private writeEnumDeclaration(
            writer: DeclarationWriter,
            decl: EnumDeclaration,
            isExported: boolean
        ): void {
            const exportKeyword = isExported ? "export " : "";
            const constModifier = hasSyntacticModifier(decl, ModifierFlags.Const) ? "const " : "";
            const name = decl.name.text;

            writer.writeLine(`${exportKeyword}declare ${constModifier}enum ${name} {`);
            writer.indent();

            for (let i = 0; i < decl.members.length; i++) {
                const member = decl.members[i];
                const memberName = isIdentifier(member.name) ? member.name.text : "/* computed */";
                const initializer = member.initializer
                    ? ` = ${this.expressionToString(member.initializer)}`
                    : "";
                const comma = i < decl.members.length - 1 ? "," : "";
                writer.writeLine(`${memberName}${initializer}${comma}`);
            }

            writer.dedent();
            writer.writeLine("}");
        }

        /**
         * Write module/namespace declaration
         */
        private writeModuleDeclaration(
            writer: DeclarationWriter,
            decl: ModuleDeclaration,
            isExported: boolean
        ): void {
            const exportKeyword = isExported ? "export " : "";
            const declareKeyword = hasSyntacticModifier(decl, ModifierFlags.Ambient) ? "declare " : "";
            const name = isIdentifier(decl.name) ? decl.name.text : `"${(decl.name as StringLiteral).text}"`;

            writer.writeLine(`${exportKeyword}${declareKeyword}namespace ${name} {`);
            writer.indent();

            if (decl.body && isModuleBlock(decl.body)) {
                for (const statement of decl.body.statements) {
                    // Recursively write module contents
                    // For simplicity, we just write the text representation
                    writer.writeLine(statement.getText());
                }
            }

            writer.dedent();
            writer.writeLine("}");
        }

        /**
         * Write type parameters
         */
        private writeTypeParameters(typeParams: NodeArray<TypeParameterDeclaration> | undefined): string {
            if (!typeParams || typeParams.length === 0) {
                return "";
            }

            const params = typeParams.map(tp => {
                let result = tp.name.text;
                if (tp.constraint) {
                    result += ` extends ${this.typeNodeToString(tp.constraint)}`;
                }
                if (tp.default) {
                    result += ` = ${this.typeNodeToString(tp.default)}`;
                }
                return result;
            });

            return `<${params.join(", ")}>`;
        }

        /**
         * Write parameters
         */
        private writeParameters(
            params: NodeArray<ParameterDeclaration>,
            diagnostics: Diagnostic[]
        ): string {
            return params.map(p => this.parameterToString(p)).join(", ");
        }

        /**
         * Convert parameter to string
         */
        private parameterToString(param: ParameterDeclaration): string {
            const name = isIdentifier(param.name) ? param.name.text : "/* pattern */";
            const optional = param.questionToken ? "?" : "";
            const dotDotDot = param.dotDotDotToken ? "..." : "";

            let type: string;
            if (param.type) {
                type = this.typeNodeToString(param.type);
            } else {
                type = this.checker.typeToString(this.checker.getTypeAtLocation(param));
            }

            return `${dotDotDot}${name}${optional}: ${type}`;
        }

        /**
         * Convert type node to string
         */
        private typeNodeToString(typeNode: TypeNode): string {
            // Simple text representation - in full implementation would use printer
            const sourceFile = getSourceFileOfNode(typeNode);
            if (sourceFile) {
                return typeNode.getText(sourceFile);
            }
            return "any";
        }

        /**
         * Convert expression to string (for enum initializers)
         */
        private expressionToString(expr: Expression): string {
            const sourceFile = getSourceFileOfNode(expr);
            if (sourceFile) {
                return expr.getText(sourceFile);
            }
            return "/* expression */";
        }

        /**
         * Get declaration output path
         */
        private getDeclarationOutputPath(sourceFile: SourceFile): string {
            const fileName = sourceFile.fileName;
            const baseName = fileName.replace(/\.(ts|tsx)$/, ".d.ts");

            if (this.options.outDir) {
                const relativePath = getRelativePathFromDirectory(
                    this.program.getCommonSourceDirectory(),
                    baseName,
                    /*ignoreCase*/ false
                );
                return combinePaths(this.options.outDir, relativePath);
            }

            return baseName;
        }

        /**
         * Generate declaration map
         */
        private generateDeclarationMap(
            sourceFile: SourceFile,
            declarationText: string
        ): { text: string; path: string } {
            const outputPath = this.getDeclarationOutputPath(sourceFile);
            const mapPath = outputPath + ".map";

            // Simple source map - full implementation would track actual mappings
            const sourceMap = {
                version: 3,
                file: getBaseFileName(outputPath),
                sourceRoot: "",
                sources: [sourceFile.fileName],
                names: [],
                mappings: "",
            };

            return {
                text: JSON.stringify(sourceMap),
                path: mapPath,
            };
        }

        /**
         * Generate bundled declaration map
         */
        private generateBundledDeclarationMap(sourceFiles: readonly SourceFile[]): string {
            const sources = sourceFiles
                .filter(sf => !sf.isDeclarationFile)
                .map(sf => sf.fileName);

            const sourceMap = {
                version: 3,
                file: this.options.outFile || "bundle.d.ts",
                sourceRoot: "",
                sources,
                names: [],
                mappings: "",
            };

            return JSON.stringify(sourceMap);
        }

        /**
         * Bundle declarations into a single output
         */
        private bundleDeclarations(
            declarations: string[],
            namespaceMerges: Map<string, NamespaceMergeInfo>
        ): string {
            const writer = new DeclarationWriter(this.options.removeComments ?? false);

            writer.writeLine("// Bundled declarations");
            writer.writeLine("");

            for (const decl of declarations) {
                writer.writeLine(decl);
                writer.writeLine("");
            }

            return writer.toString();
        }

        /**
         * Check if a file should be skipped
         */
        private shouldSkipFile(sourceFile: SourceFile): boolean {
            // Skip declaration files
            if (sourceFile.isDeclarationFile) return true;

            // Skip JSON files
            if (sourceFile.fileName.endsWith(".json")) return true;

            return false;
        }
    }

    /**
     * Helper class for writing declarations
     */
    class DeclarationWriter {
        private lines: string[] = [];
        private indentLevel = 0;
        private indentString = "    ";

        constructor(private removeComments: boolean) {}

        writeLine(text: string): void {
            const indent = this.indentString.repeat(this.indentLevel);
            this.lines.push(indent + text);
        }

        indent(): void {
            this.indentLevel++;
        }

        dedent(): void {
            if (this.indentLevel > 0) {
                this.indentLevel--;
            }
        }

        toString(): string {
            return this.lines.join("\n");
        }
    }
}
