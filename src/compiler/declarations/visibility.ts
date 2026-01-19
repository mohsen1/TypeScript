/**
 * Declaration Visibility Rules
 *
 * Implements type export visibility rules for declaration file generation.
 * Determines which symbols need to be included in .d.ts files and what
 * type annotations they need.
 */

/* @internal */
namespace ts.declarations {
    /**
     * Determine the export visibility of a symbol
     */
    export function getExportVisibility(
        symbol: Symbol,
        checker: TypeChecker,
        sourceFile: SourceFile,
        stripInternal: boolean
    ): ExportVisibility {
        // Check if marked as internal
        if (stripInternal && isInternalSymbol(symbol, sourceFile)) {
            return ExportVisibility.Internal;
        }

        // Check if directly exported
        if (isDirectlyExported(symbol, sourceFile)) {
            if (needsTypeAnnotation(symbol, checker)) {
                return ExportVisibility.ExportedNeedsAnnotation;
            }
            return ExportVisibility.Exported;
        }

        // Check if re-exported
        if (isReExported(symbol, checker, sourceFile)) {
            return ExportVisibility.ReExported;
        }

        return ExportVisibility.None;
    }

    /**
     * Check if a symbol is marked as @internal
     */
    export function isInternalSymbol(symbol: Symbol, sourceFile: SourceFile): boolean {
        const declarations = symbol.declarations;
        if (!declarations || declarations.length === 0) {
            return false;
        }

        for (const decl of declarations) {
            if (isInternalDeclaration(decl, sourceFile)) {
                return true;
            }
        }

        return false;
    }

    /**
     * Check if a declaration is marked as @internal
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
     * Check if a symbol is directly exported from the source file
     */
    function isDirectlyExported(symbol: Symbol, sourceFile: SourceFile): boolean {
        // Check the source file's exports
        const fileSymbol = (sourceFile as any).symbol;
        if (!fileSymbol || !fileSymbol.exports) {
            return false;
        }

        return fileSymbol.exports.has(symbol.escapedName);
    }

    /**
     * Check if a symbol is re-exported from another module
     */
    function isReExported(symbol: Symbol, checker: TypeChecker, sourceFile: SourceFile): boolean {
        // Check export declarations like `export { foo } from './bar'`
        for (const statement of sourceFile.statements) {
            if (isExportDeclaration(statement) && statement.exportClause) {
                if (isNamedExports(statement.exportClause)) {
                    for (const element of statement.exportClause.elements) {
                        const exportedSymbol = checker.getSymbolAtLocation(element.name);
                        if (exportedSymbol === symbol) {
                            return !!statement.moduleSpecifier;
                        }
                    }
                }
            }
        }
        return false;
    }

    /**
     * Check if a symbol needs an explicit type annotation
     */
    export function needsTypeAnnotation(symbol: Symbol, checker: TypeChecker): boolean {
        const declarations = symbol.declarations;
        if (!declarations || declarations.length === 0) {
            return false;
        }

        for (const decl of declarations) {
            if (needsTypeAnnotationForDeclaration(decl, checker)) {
                return true;
            }
        }

        return false;
    }

    /**
     * Check if a specific declaration needs a type annotation
     */
    function needsTypeAnnotationForDeclaration(decl: Declaration, checker: TypeChecker): boolean {
        // Variable declarations without type annotation
        if (isVariableDeclaration(decl)) {
            if (!decl.type) {
                // Check if the initializer is a literal or simple expression
                if (decl.initializer) {
                    return !isSimpleLiteralExpression(decl.initializer);
                }
                return true;
            }
            return false;
        }

        // Function declarations without return type
        if (isFunctionDeclaration(decl)) {
            if (!decl.type) {
                return true;
            }
            // Check parameters
            for (const param of decl.parameters) {
                if (!param.type && !param.initializer) {
                    return true;
                }
            }
            return false;
        }

        // Property declarations without type
        if (isPropertyDeclaration(decl) || isPropertySignature(decl)) {
            return !decl.type;
        }

        // Method declarations without return type
        if (isMethodDeclaration(decl) || isMethodSignature(decl)) {
            if (!decl.type) {
                return true;
            }
            for (const param of decl.parameters) {
                if (!param.type && !param.initializer) {
                    return true;
                }
            }
            return false;
        }

        return false;
    }

    /**
     * Check if an expression is a simple literal
     */
    function isSimpleLiteralExpression(expr: Expression): boolean {
        switch (expr.kind) {
            case SyntaxKind.NumericLiteral:
            case SyntaxKind.StringLiteral:
            case SyntaxKind.TrueKeyword:
            case SyntaxKind.FalseKeyword:
            case SyntaxKind.NullKeyword:
                return true;
            case SyntaxKind.ArrayLiteralExpression:
                // Empty array literal
                return (expr as ArrayLiteralExpression).elements.length === 0;
            case SyntaxKind.ObjectLiteralExpression:
                // Empty object literal
                return (expr as ObjectLiteralExpression).properties.length === 0;
            case SyntaxKind.PrefixUnaryExpression:
                // Negative number literal
                const prefix = expr as PrefixUnaryExpression;
                return prefix.operator === SyntaxKind.MinusToken &&
                    prefix.operand.kind === SyntaxKind.NumericLiteral;
            default:
                return false;
        }
    }

    /**
     * Get all symbols that need to be visible for a declaration
     */
    export function getRequiredSymbols(
        symbol: Symbol,
        checker: TypeChecker,
        visited: Set<Symbol> = new Set()
    ): Symbol[] {
        if (visited.has(symbol)) {
            return [];
        }
        visited.add(symbol);

        const result: Symbol[] = [symbol];
        const type = checker.getTypeOfSymbol(symbol);

        // Get symbols from the type
        collectSymbolsFromType(type, checker, visited, result);

        return result;
    }

    /**
     * Collect all symbols referenced by a type
     */
    function collectSymbolsFromType(
        type: Type,
        checker: TypeChecker,
        visited: Set<Symbol>,
        result: Symbol[]
    ): void {
        if (!type) return;

        const symbol = type.symbol;
        if (symbol && !visited.has(symbol)) {
            visited.add(symbol);
            result.push(symbol);
        }

        // Union/intersection types
        if (type.flags & TypeFlags.UnionOrIntersection) {
            const unionOrIntersection = type as UnionOrIntersectionType;
            for (const t of unionOrIntersection.types) {
                collectSymbolsFromType(t, checker, visited, result);
            }
        }

        // Object types with properties
        if (type.flags & TypeFlags.Object) {
            const objectType = type as ObjectType;
            const props = checker.getPropertiesOfType(objectType);
            for (const prop of props) {
                if (!visited.has(prop)) {
                    visited.add(prop);
                    const propType = checker.getTypeOfSymbol(prop);
                    collectSymbolsFromType(propType, checker, visited, result);
                }
            }
        }

        // Type references
        if ((type as TypeReference).typeArguments) {
            for (const typeArg of (type as TypeReference).typeArguments!) {
                collectSymbolsFromType(typeArg, checker, visited, result);
            }
        }
    }

    /**
     * Check if a symbol is accessible from a given location
     */
    export function isSymbolAccessible(
        symbol: Symbol,
        checker: TypeChecker,
        enclosingDeclaration: Node
    ): boolean {
        const result = checker.isSymbolAccessible(
            symbol,
            enclosingDeclaration,
            SymbolFlags.Type | SymbolFlags.Value,
            /*shouldComputeAliasesToMakeVisible*/ false
        );
        return result.accessibility === SymbolAccessibility.Accessible;
    }

    /**
     * Get accessibility diagnostics for a symbol
     */
    export function getAccessibilityDiagnostic(
        symbol: Symbol,
        checker: TypeChecker,
        enclosingDeclaration: Node
    ): SymbolAccessibilityResult {
        return checker.isSymbolAccessible(
            symbol,
            enclosingDeclaration,
            SymbolFlags.Type | SymbolFlags.Value,
            /*shouldComputeAliasesToMakeVisible*/ true
        );
    }

    /**
     * Determine which symbols need to be imported for a declaration
     */
    export function getRequiredImports(
        symbols: Symbol[],
        sourceFile: SourceFile,
        checker: TypeChecker
    ): ImportInfo[] {
        const importsByModule = new Map<string, ImportInfo>();

        for (const symbol of symbols) {
            const declarations = symbol.declarations;
            if (!declarations) continue;

            for (const decl of declarations) {
                const declSourceFile = getSourceFileOfNode(decl);
                if (declSourceFile !== sourceFile) {
                    // This symbol is from another file, we may need to import it
                    const moduleSpecifier = getModuleSpecifierForImport(
                        declSourceFile,
                        sourceFile,
                        checker
                    );

                    if (moduleSpecifier) {
                        let importInfo = importsByModule.get(moduleSpecifier);
                        if (!importInfo) {
                            importInfo = {
                                moduleSpecifier,
                                importedNames: [],
                                isTypeOnly: true,
                            };
                            importsByModule.set(moduleSpecifier, importInfo);
                        }

                        importInfo.importedNames.push({
                            name: symbol.name,
                            isDefault: false,
                            isNamespace: false,
                        });
                    }
                }
            }
        }

        return Array.from(importsByModule.values());
    }

    /**
     * Get the module specifier for importing from one file to another
     */
    function getModuleSpecifierForImport(
        fromFile: SourceFile,
        toFile: SourceFile,
        checker: TypeChecker
    ): string | undefined {
        // For now, use relative path
        // In a full implementation, this would consider path mappings, node_modules, etc.
        const fromPath = fromFile.fileName;
        const toPath = toFile.fileName;

        // Remove extension and make relative
        const relativePath = getRelativePathFromDirectory(
            getDirectoryPath(toPath),
            fromPath,
            /*ignoreCase*/ false
        );

        // Remove .ts/.tsx extension
        return relativePath.replace(/\.(ts|tsx)$/, "");
    }
}
