/* @internal */
namespace ts {
    /**
     * A lightweight type checker that handles simple type checking scenarios.
     * Uses iterative flow analysis instead of recursive checking.
     */

    /**
     * Result of type inference/solving operations
     */
    export interface SolverResult {
        type: Type;
        isError: boolean;
        narrowedType?: Type;
    }

    /**
     * Represents a type constraint in the constraint solver
     */
    interface TypeConstraint {
        source: Type;
        target: Type;
        node: Node;
        message?: DiagnosticMessage;
    }

    /**
     * Work item for iterative flow analysis
     */
    interface FlowWorkItem {
        node: Node;
        flowNode: FlowNode | undefined;
        phase: FlowAnalysisPhase;
    }

    const enum FlowAnalysisPhase {
        Initial = 0,
        Narrowing = 1,
        Final = 2,
    }

    /**
     * Interface for the thin type checker
     */
    export interface ThinTypeChecker {
        /**
         * Check a source file and return diagnostics
         */
        checkSourceFile(sourceFile: SourceFile): readonly Diagnostic[];

        /**
         * Get the type of a node
         */
        getTypeOfNode(node: Node): Type | undefined;

        /**
         * Check if type assignment is valid
         */
        isTypeAssignableTo(source: Type, target: Type): boolean;
    }

    /**
     * Creates a thin type checker instance
     */
    export function createThinTypeChecker(host: TypeCheckerHost): ThinTypeChecker {
        // Get the full type checker for delegating complex operations
        const fullChecker = createTypeChecker(host, /*produceDiagnostics*/ true);
        const compilerOptions = host.getCompilerOptions();

        // Diagnostic collection
        let diagnostics: Diagnostic[] = [];

        // Type cache for nodes
        const nodeTypeCache = new Map<number, Type>();

        // Flow type cache for iterative analysis
        const flowTypeCache = new Map<number, FlowType>();

        // Constraint solver queue
        const constraintQueue: TypeConstraint[] = [];

        // Primitive types (cached from full checker)
        const anyType = fullChecker.getAnyType();
        const stringType = fullChecker.getStringType();
        const numberType = fullChecker.getNumberType();
        const booleanType = fullChecker.getBooleanType();
        const voidType = fullChecker.getVoidType();
        const undefinedType = fullChecker.getUndefinedType();
        const nullType = fullChecker.getNullType();
        const neverType = fullChecker.getNeverType();
        const unknownType = (fullChecker as any).getUnknownType?.() ?? anyType;

        const thinChecker: ThinTypeChecker = {
            checkSourceFile,
            getTypeOfNode,
            isTypeAssignableTo,
        };

        return thinChecker;

        /**
         * Check a source file using iterative flow analysis
         */
        function checkSourceFile(sourceFile: SourceFile): readonly Diagnostic[] {
            diagnostics = [];
            nodeTypeCache.clear();
            flowTypeCache.clear();
            constraintQueue.length = 0;

            // Phase 1: Bind the source file (uses existing binder)
            bindSourceFile(sourceFile, compilerOptions);

            // Phase 2: Iterative type checking using work queue
            checkSourceFileIteratively(sourceFile);

            // Phase 3: Solve collected constraints
            solveConstraints();

            return diagnostics;
        }

        /**
         * Iteratively check source file using a work queue instead of recursion
         */
        function checkSourceFileIteratively(sourceFile: SourceFile): void {
            const workQueue: FlowWorkItem[] = [];

            // Initialize work queue with all statements
            for (const statement of sourceFile.statements) {
                workQueue.push({
                    node: statement,
                    flowNode: (statement as any).flowNode,
                    phase: FlowAnalysisPhase.Initial,
                });
            }

            // Process work queue iteratively
            while (workQueue.length > 0) {
                const item = workQueue.shift()!;
                processWorkItem(item, workQueue);
            }
        }

        /**
         * Process a single work item in the iterative analysis
         */
        function processWorkItem(item: FlowWorkItem, workQueue: FlowWorkItem[]): void {
            const { node, phase } = item;

            switch (node.kind) {
                case SyntaxKind.VariableStatement:
                    checkVariableStatement(node as VariableStatement, workQueue);
                    break;
                case SyntaxKind.ExpressionStatement:
                    checkExpressionStatement(node as ExpressionStatement, workQueue);
                    break;
                case SyntaxKind.IfStatement:
                    checkIfStatement(node as IfStatement, workQueue);
                    break;
                case SyntaxKind.ReturnStatement:
                    checkReturnStatement(node as ReturnStatement, workQueue);
                    break;
                case SyntaxKind.FunctionDeclaration:
                    checkFunctionDeclaration(node as FunctionDeclaration, workQueue);
                    break;
                case SyntaxKind.ClassDeclaration:
                    checkClassDeclaration(node as ClassDeclaration, workQueue);
                    break;
                case SyntaxKind.Block:
                    checkBlock(node as Block, workQueue);
                    break;
                case SyntaxKind.ForStatement:
                    checkForStatement(node as ForStatement, workQueue);
                    break;
                case SyntaxKind.WhileStatement:
                    checkWhileStatement(node as WhileStatement, workQueue);
                    break;
                case SyntaxKind.TypeAliasDeclaration:
                case SyntaxKind.InterfaceDeclaration:
                    // Type declarations don't need runtime checking
                    break;
                default:
                    // For unsupported nodes, try to check any expressions they contain
                    checkNodeExpressions(node, workQueue);
                    break;
            }
        }

        /**
         * Check variable statement
         */
        function checkVariableStatement(node: VariableStatement, workQueue: FlowWorkItem[]): void {
            for (const declaration of node.declarationList.declarations) {
                checkVariableDeclaration(declaration);
            }
        }

        /**
         * Check a variable declaration
         */
        function checkVariableDeclaration(declaration: VariableDeclaration): void {
            const name = declaration.name;

            if (!isIdentifier(name)) {
                // Handle destructuring patterns in a simplified way
                return;
            }

            const declaredType = declaration.type ? getTypeFromTypeNode(declaration.type) : undefined;
            const initializerType = declaration.initializer ? evaluateExpression(declaration.initializer) : undefined;

            if (declaredType && initializerType) {
                // Add constraint: initializer type must be assignable to declared type
                addConstraint(initializerType, declaredType, declaration, Diagnostics.Type_0_is_not_assignable_to_type_1);
            }

            // Cache the type for this variable
            const resolvedType = declaredType || initializerType || anyType;
            cacheNodeType(declaration, resolvedType);
        }

        /**
         * Check expression statement
         */
        function checkExpressionStatement(node: ExpressionStatement, workQueue: FlowWorkItem[]): void {
            evaluateExpression(node.expression);
        }

        /**
         * Check if statement - add both branches to work queue
         */
        function checkIfStatement(node: IfStatement, workQueue: FlowWorkItem[]): void {
            // Check condition
            const conditionType = evaluateExpression(node.expression);

            // Add then branch to work queue
            workQueue.push({
                node: node.thenStatement,
                flowNode: (node.thenStatement as any).flowNode,
                phase: FlowAnalysisPhase.Narrowing,
            });

            // Add else branch to work queue if present
            if (node.elseStatement) {
                workQueue.push({
                    node: node.elseStatement,
                    flowNode: (node.elseStatement as any).flowNode,
                    phase: FlowAnalysisPhase.Narrowing,
                });
            }
        }

        /**
         * Check return statement
         */
        function checkReturnStatement(node: ReturnStatement, workQueue: FlowWorkItem[]): void {
            if (node.expression) {
                const returnType = evaluateExpression(node.expression);

                // Find containing function and check return type compatibility
                const containingFunction = findAncestor(node, isFunctionLike);
                if (containingFunction && containingFunction.type) {
                    const declaredReturnType = getTypeFromTypeNode(containingFunction.type);
                    addConstraint(returnType, declaredReturnType, node, Diagnostics.Type_0_is_not_assignable_to_type_1);
                }
            }
        }

        /**
         * Check function declaration
         */
        function checkFunctionDeclaration(node: FunctionDeclaration, workQueue: FlowWorkItem[]): void {
            // Check parameters
            for (const param of node.parameters) {
                if (param.type) {
                    cacheNodeType(param, getTypeFromTypeNode(param.type));
                }
                if (param.initializer) {
                    const initType = evaluateExpression(param.initializer);
                    if (param.type) {
                        const paramType = getTypeFromTypeNode(param.type);
                        addConstraint(initType, paramType, param, Diagnostics.Type_0_is_not_assignable_to_type_1);
                    }
                }
            }

            // Add function body to work queue
            if (node.body) {
                workQueue.push({
                    node: node.body,
                    flowNode: (node.body as any).flowNode,
                    phase: FlowAnalysisPhase.Initial,
                });
            }
        }

        /**
         * Check class declaration
         */
        function checkClassDeclaration(node: ClassDeclaration, workQueue: FlowWorkItem[]): void {
            // Check members
            for (const member of node.members) {
                if (isMethodDeclaration(member)) {
                    if (member.body) {
                        workQueue.push({
                            node: member.body,
                            flowNode: (member.body as any).flowNode,
                            phase: FlowAnalysisPhase.Initial,
                        });
                    }
                } else if (isPropertyDeclaration(member)) {
                    if (member.initializer) {
                        const initType = evaluateExpression(member.initializer);
                        if (member.type) {
                            const propType = getTypeFromTypeNode(member.type);
                            addConstraint(initType, propType, member, Diagnostics.Type_0_is_not_assignable_to_type_1);
                        }
                    }
                } else if (isConstructorDeclaration(member)) {
                    if (member.body) {
                        workQueue.push({
                            node: member.body,
                            flowNode: (member.body as any).flowNode,
                            phase: FlowAnalysisPhase.Initial,
                        });
                    }
                }
            }
        }

        /**
         * Check block - add statements to work queue
         */
        function checkBlock(node: Block, workQueue: FlowWorkItem[]): void {
            for (const statement of node.statements) {
                workQueue.push({
                    node: statement,
                    flowNode: (statement as any).flowNode,
                    phase: FlowAnalysisPhase.Initial,
                });
            }
        }

        /**
         * Check for statement
         */
        function checkForStatement(node: ForStatement, workQueue: FlowWorkItem[]): void {
            // Check initializer
            if (node.initializer) {
                if (isVariableDeclarationList(node.initializer)) {
                    for (const decl of node.initializer.declarations) {
                        checkVariableDeclaration(decl);
                    }
                } else {
                    evaluateExpression(node.initializer);
                }
            }

            // Check condition
            if (node.condition) {
                evaluateExpression(node.condition);
            }

            // Check incrementor
            if (node.incrementor) {
                evaluateExpression(node.incrementor);
            }

            // Add body to work queue
            workQueue.push({
                node: node.statement,
                flowNode: (node.statement as any).flowNode,
                phase: FlowAnalysisPhase.Initial,
            });
        }

        /**
         * Check while statement
         */
        function checkWhileStatement(node: WhileStatement, workQueue: FlowWorkItem[]): void {
            // Check condition
            evaluateExpression(node.expression);

            // Add body to work queue
            workQueue.push({
                node: node.statement,
                flowNode: (node.statement as any).flowNode,
                phase: FlowAnalysisPhase.Initial,
            });
        }

        /**
         * Check any expressions in a node
         */
        function checkNodeExpressions(node: Node, workQueue: FlowWorkItem[]): void {
            forEachChild(node, child => {
                if (isExpression(child)) {
                    evaluateExpression(child);
                } else if (isStatement(child)) {
                    workQueue.push({
                        node: child,
                        flowNode: (child as any).flowNode,
                        phase: FlowAnalysisPhase.Initial,
                    });
                }
            });
        }

        /**
         * Evaluate an expression and return its type
         * This is the "evaluator" component
         */
        function evaluateExpression(expr: Expression): Type {
            switch (expr.kind) {
                case SyntaxKind.NumericLiteral:
                    return numberType;
                case SyntaxKind.StringLiteral:
                case SyntaxKind.NoSubstitutionTemplateLiteral:
                    return stringType;
                case SyntaxKind.TrueKeyword:
                case SyntaxKind.FalseKeyword:
                    return booleanType;
                case SyntaxKind.NullKeyword:
                    return nullType;
                case SyntaxKind.Identifier:
                    return evaluateIdentifier(expr as Identifier);
                case SyntaxKind.BinaryExpression:
                    return evaluateBinaryExpression(expr as BinaryExpression);
                case SyntaxKind.CallExpression:
                    return evaluateCallExpression(expr as CallExpression);
                case SyntaxKind.PropertyAccessExpression:
                    return evaluatePropertyAccess(expr as PropertyAccessExpression);
                case SyntaxKind.ElementAccessExpression:
                    return evaluateElementAccess(expr as ElementAccessExpression);
                case SyntaxKind.PrefixUnaryExpression:
                    return evaluatePrefixUnary(expr as PrefixUnaryExpression);
                case SyntaxKind.PostfixUnaryExpression:
                    return evaluatePostfixUnary(expr as PostfixUnaryExpression);
                case SyntaxKind.ConditionalExpression:
                    return evaluateConditional(expr as ConditionalExpression);
                case SyntaxKind.ArrayLiteralExpression:
                    return evaluateArrayLiteral(expr as ArrayLiteralExpression);
                case SyntaxKind.ObjectLiteralExpression:
                    return evaluateObjectLiteral(expr as ObjectLiteralExpression);
                case SyntaxKind.ArrowFunction:
                case SyntaxKind.FunctionExpression:
                    return evaluateFunctionExpression(expr as ArrowFunction | FunctionExpression);
                case SyntaxKind.ParenthesizedExpression:
                    return evaluateExpression((expr as ParenthesizedExpression).expression);
                case SyntaxKind.AsExpression:
                case SyntaxKind.TypeAssertionExpression:
                    return evaluateTypeAssertion(expr as AsExpression | TypeAssertion);
                case SyntaxKind.NewExpression:
                    return evaluateNewExpression(expr as NewExpression);
                default:
                    // Fallback to full checker for complex expressions
                    return getTypeFromFullChecker(expr);
            }
        }

        /**
         * Evaluate identifier
         */
        function evaluateIdentifier(node: Identifier): Type {
            // Check if it's undefined
            if (node.escapedText === "undefined") {
                return undefinedType;
            }

            // Look up symbol
            const symbol = fullChecker.getSymbolAtLocation(node);
            if (symbol) {
                return fullChecker.getTypeOfSymbolAtLocation(symbol, node);
            }

            // Report error for undeclared identifier
            addDiagnostic(node, Diagnostics.Cannot_find_name_0, idText(node));
            return anyType;
        }

        /**
         * Evaluate binary expression
         */
        function evaluateBinaryExpression(node: BinaryExpression): Type {
            const leftType = evaluateExpression(node.left);
            const rightType = evaluateExpression(node.right);
            const operator = node.operatorToken.kind;

            switch (operator) {
                case SyntaxKind.PlusToken:
                    // String concatenation or numeric addition
                    if (isStringType(leftType) || isStringType(rightType)) {
                        return stringType;
                    }
                    return numberType;

                case SyntaxKind.MinusToken:
                case SyntaxKind.AsteriskToken:
                case SyntaxKind.SlashToken:
                case SyntaxKind.PercentToken:
                case SyntaxKind.AsteriskAsteriskToken:
                    return numberType;

                case SyntaxKind.LessThanToken:
                case SyntaxKind.LessThanEqualsToken:
                case SyntaxKind.GreaterThanToken:
                case SyntaxKind.GreaterThanEqualsToken:
                case SyntaxKind.EqualsEqualsToken:
                case SyntaxKind.EqualsEqualsEqualsToken:
                case SyntaxKind.ExclamationEqualsToken:
                case SyntaxKind.ExclamationEqualsEqualsToken:
                    return booleanType;

                case SyntaxKind.AmpersandAmpersandToken:
                    // Returns right type or falsy left
                    return fullChecker.getUnionType([leftType, rightType]);

                case SyntaxKind.BarBarToken:
                    // Returns first truthy or last
                    return fullChecker.getUnionType([leftType, rightType]);

                case SyntaxKind.EqualsToken:
                    // Assignment - check compatibility
                    addConstraint(rightType, leftType, node, Diagnostics.Type_0_is_not_assignable_to_type_1);
                    return rightType;

                case SyntaxKind.PlusEqualsToken:
                case SyntaxKind.MinusEqualsToken:
                case SyntaxKind.AsteriskEqualsToken:
                case SyntaxKind.SlashEqualsToken:
                    return leftType;

                case SyntaxKind.CommaToken:
                    return rightType;

                default:
                    return anyType;
            }
        }

        /**
         * Evaluate call expression
         */
        function evaluateCallExpression(node: CallExpression): Type {
            const calleeType = evaluateExpression(node.expression);

            // Get call signature
            const signatures = fullChecker.getSignaturesOfType(calleeType, SignatureKind.Call);
            if (signatures.length === 0) {
                // Not callable
                if (!isAnyType(calleeType)) {
                    addDiagnostic(node.expression, Diagnostics.This_expression_is_not_callable);
                }
                return anyType;
            }

            // Check arguments against first signature (simplified)
            const signature = signatures[0];
            const parameters = signature.parameters;

            for (let i = 0; i < node.arguments.length; i++) {
                const arg = node.arguments[i];
                const argType = evaluateExpression(arg);

                if (i < parameters.length) {
                    const paramType = fullChecker.getTypeOfSymbolAtLocation(parameters[i], node);
                    addConstraint(argType, paramType, arg, Diagnostics.Argument_of_type_0_is_not_assignable_to_parameter_of_type_1);
                }
            }

            return fullChecker.getReturnTypeOfSignature(signature);
        }

        /**
         * Evaluate property access
         */
        function evaluatePropertyAccess(node: PropertyAccessExpression): Type {
            const objectType = evaluateExpression(node.expression);
            const propertyName = node.name.escapedText as string;

            const property = fullChecker.getPropertyOfType(objectType, propertyName);
            if (property) {
                return fullChecker.getTypeOfSymbolAtLocation(property, node);
            }

            // Check for property existence
            if (!isAnyType(objectType)) {
                addDiagnostic(node.name, Diagnostics.Property_0_does_not_exist_on_type_1,
                    propertyName, fullChecker.typeToString(objectType));
            }
            return anyType;
        }

        /**
         * Evaluate element access
         */
        function evaluateElementAccess(node: ElementAccessExpression): Type {
            const objectType = evaluateExpression(node.expression);
            const indexType = evaluateExpression(node.argumentExpression);

            // Get index signature
            const indexInfos = fullChecker.getIndexInfosOfType(objectType);
            for (const indexInfo of indexInfos) {
                if (fullChecker.isTypeAssignableTo(indexType, indexInfo.keyType)) {
                    return indexInfo.type;
                }
            }

            return anyType;
        }

        /**
         * Evaluate prefix unary expression
         */
        function evaluatePrefixUnary(node: PrefixUnaryExpression): Type {
            const operandType = evaluateExpression(node.operand);

            switch (node.operator) {
                case SyntaxKind.ExclamationToken:
                    return booleanType;
                case SyntaxKind.PlusToken:
                case SyntaxKind.MinusToken:
                case SyntaxKind.TildeToken:
                    return numberType;
                case SyntaxKind.PlusPlusToken:
                case SyntaxKind.MinusMinusToken:
                    return operandType;
                default:
                    return anyType;
            }
        }

        /**
         * Evaluate postfix unary expression
         */
        function evaluatePostfixUnary(node: PostfixUnaryExpression): Type {
            return numberType;
        }

        /**
         * Evaluate conditional expression
         */
        function evaluateConditional(node: ConditionalExpression): Type {
            evaluateExpression(node.condition);
            const whenTrue = evaluateExpression(node.whenTrue);
            const whenFalse = evaluateExpression(node.whenFalse);
            return fullChecker.getUnionType([whenTrue, whenFalse]);
        }

        /**
         * Evaluate array literal
         */
        function evaluateArrayLiteral(node: ArrayLiteralExpression): Type {
            if (node.elements.length === 0) {
                return fullChecker.createArrayType(anyType);
            }

            const elementTypes: Type[] = [];
            for (const element of node.elements) {
                if (isSpreadElement(element)) {
                    const spreadType = evaluateExpression(element.expression);
                    const elementType = fullChecker.getElementTypeOfArrayType(spreadType);
                    if (elementType) {
                        elementTypes.push(elementType);
                    }
                } else {
                    elementTypes.push(evaluateExpression(element));
                }
            }

            const unionType = fullChecker.getUnionType(elementTypes);
            return fullChecker.createArrayType(unionType);
        }

        /**
         * Evaluate object literal
         */
        function evaluateObjectLiteral(node: ObjectLiteralExpression): Type {
            // Delegate to full checker for object literal type
            return getTypeFromFullChecker(node);
        }

        /**
         * Evaluate function expression
         */
        function evaluateFunctionExpression(node: ArrowFunction | FunctionExpression): Type {
            return getTypeFromFullChecker(node);
        }

        /**
         * Evaluate type assertion
         */
        function evaluateTypeAssertion(node: AsExpression | TypeAssertion): Type {
            evaluateExpression(node.expression);
            return getTypeFromTypeNode(node.type);
        }

        /**
         * Evaluate new expression
         */
        function evaluateNewExpression(node: NewExpression): Type {
            const constructorType = evaluateExpression(node.expression);

            const signatures = fullChecker.getSignaturesOfType(constructorType, SignatureKind.Construct);
            if (signatures.length > 0) {
                return fullChecker.getReturnTypeOfSignature(signatures[0]);
            }

            return anyType;
        }

        /**
         * Get type from type node
         */
        function getTypeFromTypeNode(typeNode: TypeNode): Type {
            return fullChecker.getTypeFromTypeNode(typeNode);
        }

        /**
         * Get type from full checker (fallback)
         */
        function getTypeFromFullChecker(node: Node): Type {
            try {
                return fullChecker.getTypeAtLocation(node);
            } catch {
                return anyType;
            }
        }

        /**
         * Check if type is string
         */
        function isStringType(type: Type): boolean {
            return !!(type.flags & TypeFlags.StringLike);
        }

        /**
         * Check if type is any
         */
        function isAnyType(type: Type): boolean {
            return !!(type.flags & TypeFlags.Any);
        }

        /**
         * Add a type constraint to the solver queue
         */
        function addConstraint(source: Type, target: Type, node: Node, message: DiagnosticMessage): void {
            constraintQueue.push({ source, target, node, message });
        }

        /**
         * Solve all collected constraints
         * This is the "solver" component
         */
        function solveConstraints(): void {
            for (const constraint of constraintQueue) {
                if (!isTypeAssignableTo(constraint.source, constraint.target)) {
                    if (constraint.message) {
                        addDiagnostic(
                            constraint.node,
                            constraint.message,
                            fullChecker.typeToString(constraint.source),
                            fullChecker.typeToString(constraint.target)
                        );
                    }
                }
            }
        }

        /**
         * Check if source type is assignable to target type
         */
        function isTypeAssignableTo(source: Type, target: Type): boolean {
            // Use full checker for now
            return fullChecker.isTypeAssignableTo(source, target);
        }

        /**
         * Get the type of a node (public API)
         */
        function getTypeOfNode(node: Node): Type | undefined {
            const nodeId = getNodeId(node);
            if (nodeTypeCache.has(nodeId)) {
                return nodeTypeCache.get(nodeId);
            }

            if (isExpression(node)) {
                return evaluateExpression(node);
            }

            return fullChecker.getTypeAtLocation(node);
        }

        /**
         * Cache type for a node
         */
        function cacheNodeType(node: Node, type: Type): void {
            nodeTypeCache.set(getNodeId(node), type);
        }

        /**
         * Add a diagnostic
         */
        function addDiagnostic(node: Node, message: DiagnosticMessage, ...args: (string | number)[]): void {
            diagnostics.push(createDiagnosticForNode(node, message, ...args));
        }
    }
}
