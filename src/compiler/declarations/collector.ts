/**
 * Declaration Collector
 *
 * Collects all declarations that need to be emitted to .d.ts files.
 * Handles visibility analysis and determines which declarations need
 * explicit type annotations.
 */

/* @internal */
namespace ts.declarations {
    /**
     * Collect all declarations from a source file that need to be emitted
     */
    export function collectDeclarations(
        sourceFile: SourceFile,
        checker: TypeChecker,
        options: DeclarationEmitOptions
    ): CollectedDeclarations {
        const exports: SymbolVisibilityInfo[] = [];
        const imports: ImportInfo[] = [];
        const typeReferences = new Set<string>();
        const fileReferences = new Set<string>();
        let hasExportAssignment = false;
        const isExternalModule = isExternalOrCommonJsModule(sourceFile);

        // Collect exports from the source file
        collectExports(sourceFile, checker, options, exports);

        // Determine required imports
        const requiredSymbols = new Set<Symbol>();
        for (const exp of exports) {
            for (const sym of getRequiredSymbols(exp.symbol, checker)) {
                requiredSymbols.add(sym);
            }
        }

        // Calculate imports needed
        const importInfos = getRequiredImports(
            Array.from(requiredSymbols),
            sourceFile,
            checker
        );
        imports.push(...importInfos);

        // Collect type references
        if (sourceFile.typeReferenceDirectives) {
            for (const ref of sourceFile.typeReferenceDirectives) {
                typeReferences.add(ref.fileName);
            }
        }

        // Check for export assignment
        for (const statement of sourceFile.statements) {
            if (isExportAssignment(statement)) {
                hasExportAssignment = true;
                break;
            }
        }

        return {
            exports,
            imports,
            typeReferences: Array.from(typeReferences),
            fileReferences: Array.from(fileReferences),
            hasExportAssignment,
            isExternalModule,
        };
    }

    /**
     * Collect exports from a source file
     */
    function collectExports(
        sourceFile: SourceFile,
        checker: TypeChecker,
        options: DeclarationEmitOptions,
        result: SymbolVisibilityInfo[]
    ): void {
        const stripInternal = options.stripInternal ?? false;

        for (const statement of sourceFile.statements) {
            collectExportsFromStatement(statement, sourceFile, checker, stripInternal, result);
        }
    }

    /**
     * Collect exports from a single statement
     */
    function collectExportsFromStatement(
        statement: Statement,
        sourceFile: SourceFile,
        checker: TypeChecker,
        stripInternal: boolean,
        result: SymbolVisibilityInfo[]
    ): void {
        // Skip internal declarations
        if (stripInternal && isInternalDeclaration(statement, sourceFile)) {
            return;
        }

        // Variable statement
        if (isVariableStatement(statement)) {
            if (hasExportModifier(statement)) {
                for (const decl of statement.declarationList.declarations) {
                    collectVariableDeclaration(decl, sourceFile, checker, stripInternal, result);
                }
            }
            return;
        }

        // Function declaration
        if (isFunctionDeclaration(statement)) {
            if (hasExportModifier(statement) || statement.name) {
                collectFunctionDeclaration(statement, sourceFile, checker, stripInternal, result);
            }
            return;
        }

        // Class declaration
        if (isClassDeclaration(statement)) {
            if (hasExportModifier(statement) || statement.name) {
                collectClassDeclaration(statement, sourceFile, checker, stripInternal, result);
            }
            return;
        }

        // Interface declaration
        if (isInterfaceDeclaration(statement)) {
            if (hasExportModifier(statement)) {
                collectInterfaceDeclaration(statement, sourceFile, checker, stripInternal, result);
            }
            return;
        }

        // Type alias declaration
        if (isTypeAliasDeclaration(statement)) {
            if (hasExportModifier(statement)) {
                collectTypeAliasDeclaration(statement, sourceFile, checker, stripInternal, result);
            }
            return;
        }

        // Enum declaration
        if (isEnumDeclaration(statement)) {
            if (hasExportModifier(statement)) {
                collectEnumDeclaration(statement, sourceFile, checker, stripInternal, result);
            }
            return;
        }

        // Module/Namespace declaration
        if (isModuleDeclaration(statement)) {
            collectModuleDeclaration(statement, sourceFile, checker, stripInternal, result);
            return;
        }

        // Export declaration
        if (isExportDeclaration(statement)) {
            collectExportDeclaration(statement, sourceFile, checker, stripInternal, result);
            return;
        }

        // Export assignment
        if (isExportAssignment(statement)) {
            collectExportAssignment(statement, sourceFile, checker, result);
            return;
        }
    }

    /**
     * Check if a statement has the export modifier
     */
    function hasExportModifier(node: Node): boolean {
        return hasSyntacticModifier(node, ModifierFlags.Export);
    }

    /**
     * Check if a declaration is internal
     */
    function isInternalDeclaration(node: Node, sourceFile: SourceFile): boolean {
        const parseTreeNode = getParseTreeNode(node);
        if (!parseTreeNode) {
            return false;
        }

        const leadingComments = getLeadingCommentRangesOfNode(parseTreeNode, sourceFile);
        if (!leadingComments) {
            return false;
        }

        for (const range of leadingComments) {
            const comment = sourceFile.text.substring(range.pos, range.end);
            if (comment.includes("@internal")) {
                return true;
            }
        }

        return false;
    }

    /**
     * Collect variable declaration
     */
    function collectVariableDeclaration(
        decl: VariableDeclaration,
        sourceFile: SourceFile,
        checker: TypeChecker,
        stripInternal: boolean,
        result: SymbolVisibilityInfo[]
    ): void {
        if (!isIdentifier(decl.name)) {
            // Handle binding patterns
            collectBindingPattern(decl.name, sourceFile, checker, stripInternal, result);
            return;
        }

        const symbol = checker.getSymbolAtLocation(decl.name);
        if (!symbol) return;

        const visibility = getExportVisibility(symbol, checker, sourceFile, stripInternal);
        if (visibility === ExportVisibility.None || visibility === ExportVisibility.Internal) {
            return;
        }

        const needsAnnotation = needsTypeAnnotation(symbol, checker);
        result.push({
            symbol,
            visibility,
            needsTypeAnnotation: needsAnnotation,
            accessibleFrom: [sourceFile],
            inferredType: needsAnnotation ? checker.getTypeOfSymbol(symbol) : undefined,
        });
    }

    /**
     * Collect binding pattern declarations
     */
    function collectBindingPattern(
        pattern: BindingPattern,
        sourceFile: SourceFile,
        checker: TypeChecker,
        stripInternal: boolean,
        result: SymbolVisibilityInfo[]
    ): void {
        for (const element of pattern.elements) {
            if (isOmittedExpression(element)) continue;

            if (isIdentifier(element.name)) {
                const symbol = checker.getSymbolAtLocation(element.name);
                if (symbol) {
                    const visibility = getExportVisibility(symbol, checker, sourceFile, stripInternal);
                    if (visibility !== ExportVisibility.None && visibility !== ExportVisibility.Internal) {
                        result.push({
                            symbol,
                            visibility,
                            needsTypeAnnotation: true, // Binding patterns always need annotation
                            accessibleFrom: [sourceFile],
                            inferredType: checker.getTypeOfSymbol(symbol),
                        });
                    }
                }
            } else {
                collectBindingPattern(element.name, sourceFile, checker, stripInternal, result);
            }
        }
    }

    /**
     * Collect function declaration
     */
    function collectFunctionDeclaration(
        decl: FunctionDeclaration,
        sourceFile: SourceFile,
        checker: TypeChecker,
        stripInternal: boolean,
        result: SymbolVisibilityInfo[]
    ): void {
        if (!decl.name) return;

        const symbol = checker.getSymbolAtLocation(decl.name);
        if (!symbol) return;

        const visibility = getExportVisibility(symbol, checker, sourceFile, stripInternal);
        if (visibility === ExportVisibility.None || visibility === ExportVisibility.Internal) {
            return;
        }

        const needsAnnotation = !decl.type || decl.parameters.some(p => !p.type);
        result.push({
            symbol,
            visibility,
            needsTypeAnnotation: needsAnnotation,
            accessibleFrom: [sourceFile],
            inferredType: needsAnnotation ? checker.getTypeOfSymbol(symbol) : undefined,
        });
    }

    /**
     * Collect class declaration
     */
    function collectClassDeclaration(
        decl: ClassDeclaration,
        sourceFile: SourceFile,
        checker: TypeChecker,
        stripInternal: boolean,
        result: SymbolVisibilityInfo[]
    ): void {
        if (!decl.name) return;

        const symbol = checker.getSymbolAtLocation(decl.name);
        if (!symbol) return;

        const visibility = getExportVisibility(symbol, checker, sourceFile, stripInternal);
        if (visibility === ExportVisibility.None || visibility === ExportVisibility.Internal) {
            return;
        }

        // Check if any members need type annotations
        let needsAnnotation = false;
        for (const member of decl.members) {
            if (isPropertyDeclaration(member) && !member.type) {
                needsAnnotation = true;
                break;
            }
            if (isMethodDeclaration(member) && !member.type) {
                needsAnnotation = true;
                break;
            }
        }

        result.push({
            symbol,
            visibility,
            needsTypeAnnotation: needsAnnotation,
            accessibleFrom: [sourceFile],
        });
    }

    /**
     * Collect interface declaration
     */
    function collectInterfaceDeclaration(
        decl: InterfaceDeclaration,
        sourceFile: SourceFile,
        checker: TypeChecker,
        stripInternal: boolean,
        result: SymbolVisibilityInfo[]
    ): void {
        const symbol = checker.getSymbolAtLocation(decl.name);
        if (!symbol) return;

        const visibility = getExportVisibility(symbol, checker, sourceFile, stripInternal);
        if (visibility === ExportVisibility.None || visibility === ExportVisibility.Internal) {
            return;
        }

        result.push({
            symbol,
            visibility,
            needsTypeAnnotation: false,
            accessibleFrom: [sourceFile],
        });
    }

    /**
     * Collect type alias declaration
     */
    function collectTypeAliasDeclaration(
        decl: TypeAliasDeclaration,
        sourceFile: SourceFile,
        checker: TypeChecker,
        stripInternal: boolean,
        result: SymbolVisibilityInfo[]
    ): void {
        const symbol = checker.getSymbolAtLocation(decl.name);
        if (!symbol) return;

        const visibility = getExportVisibility(symbol, checker, sourceFile, stripInternal);
        if (visibility === ExportVisibility.None || visibility === ExportVisibility.Internal) {
            return;
        }

        result.push({
            symbol,
            visibility,
            needsTypeAnnotation: false,
            accessibleFrom: [sourceFile],
        });
    }

    /**
     * Collect enum declaration
     */
    function collectEnumDeclaration(
        decl: EnumDeclaration,
        sourceFile: SourceFile,
        checker: TypeChecker,
        stripInternal: boolean,
        result: SymbolVisibilityInfo[]
    ): void {
        const symbol = checker.getSymbolAtLocation(decl.name);
        if (!symbol) return;

        const visibility = getExportVisibility(symbol, checker, sourceFile, stripInternal);
        if (visibility === ExportVisibility.None || visibility === ExportVisibility.Internal) {
            return;
        }

        result.push({
            symbol,
            visibility,
            needsTypeAnnotation: false,
            accessibleFrom: [sourceFile],
        });
    }

    /**
     * Collect module/namespace declaration
     */
    function collectModuleDeclaration(
        decl: ModuleDeclaration,
        sourceFile: SourceFile,
        checker: TypeChecker,
        stripInternal: boolean,
        result: SymbolVisibilityInfo[]
    ): void {
        const symbol = checker.getSymbolAtLocation(decl.name);
        if (!symbol) return;

        // Handle both exported namespaces and ambient module declarations
        const visibility = getExportVisibility(symbol, checker, sourceFile, stripInternal);

        // Module declarations should be collected if they're exported or ambient
        if (visibility === ExportVisibility.None && !hasSyntacticModifier(decl, ModifierFlags.Ambient)) {
            return;
        }

        if (visibility === ExportVisibility.Internal) {
            return;
        }

        result.push({
            symbol,
            visibility: visibility || ExportVisibility.Exported,
            needsTypeAnnotation: false,
            accessibleFrom: [sourceFile],
        });
    }

    /**
     * Collect export declaration (re-exports)
     */
    function collectExportDeclaration(
        decl: ExportDeclaration,
        sourceFile: SourceFile,
        checker: TypeChecker,
        stripInternal: boolean,
        result: SymbolVisibilityInfo[]
    ): void {
        if (!decl.exportClause) {
            // export * from 'module' - handled separately
            return;
        }

        if (isNamedExports(decl.exportClause)) {
            for (const element of decl.exportClause.elements) {
                const symbol = checker.getSymbolAtLocation(element.name);
                if (!symbol) continue;

                const visibility = decl.moduleSpecifier
                    ? ExportVisibility.ReExported
                    : ExportVisibility.Exported;

                result.push({
                    symbol,
                    visibility,
                    needsTypeAnnotation: false,
                    accessibleFrom: [sourceFile],
                });
            }
        }
    }

    /**
     * Collect export assignment
     */
    function collectExportAssignment(
        decl: ExportAssignment,
        sourceFile: SourceFile,
        checker: TypeChecker,
        result: SymbolVisibilityInfo[]
    ): void {
        const symbol = checker.getSymbolAtLocation(decl.expression);
        if (!symbol) return;

        result.push({
            symbol,
            visibility: ExportVisibility.Exported,
            needsTypeAnnotation: needsTypeAnnotation(symbol, checker),
            accessibleFrom: [sourceFile],
            inferredType: checker.getTypeOfSymbol(symbol),
        });
    }

    /**
     * Collect declarations for namespace merging
     */
    export function collectNamespaceMerges(
        sourceFiles: readonly SourceFile[],
        checker: TypeChecker
    ): Map<string, NamespaceMergeInfo> {
        const merges = new Map<string, NamespaceMergeInfo>();

        for (const sourceFile of sourceFiles) {
            for (const statement of sourceFile.statements) {
                if (isModuleDeclaration(statement) && isIdentifier(statement.name)) {
                    const name = statement.name.text;
                    const symbol = checker.getSymbolAtLocation(statement.name);
                    if (!symbol) continue;

                    const existing = merges.get(name);
                    if (existing) {
                        existing.declarations.push(statement);
                    } else {
                        merges.set(name, {
                            name,
                            declarations: [statement],
                            mergedType: checker.getDeclaredTypeOfSymbol(symbol),
                        });
                    }
                }
            }
        }

        return merges;
    }
}
