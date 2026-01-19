/**
 * Expression Evaluator
 *
 * Evaluates expressions to determine their types.
 * Used by the thin checker for type inference.
 */

/* @internal */
namespace ts.thinChecker {
    /**
     * Result of evaluating an expression
     */
    export interface EvaluationResult {
        type: Type;
        isLiteral: boolean;
        literalValue?: string | number | boolean;
        isConstant: boolean;
    }

    /**
     * Expression evaluator interface
     */
    export interface ExpressionEvaluator {
        /**
         * Evaluate an expression and return its type
         */
        evaluate(expr: Expression): EvaluationResult;

        /**
         * Get cached result for an expression
         */
        getCached(expr: Expression): EvaluationResult | undefined;

        /**
         * Clear the evaluation cache
         */
        clearCache(): void;
    }

    /**
     * Create an expression evaluator
     */
    export function createExpressionEvaluator(
        checker: TypeChecker,
        onError: (node: Node, message: DiagnosticMessage, ...args: (string | number)[]) => void
    ): ExpressionEvaluator {
        // Cache for evaluated expressions
        const cache = new Map<number, EvaluationResult>();

        // Get primitive types from checker
        const anyType = checker.getAnyType();
        const stringType = checker.getStringType();
        const numberType = checker.getNumberType();
        const booleanType = checker.getBooleanType();
        const voidType = checker.getVoidType();
        const undefinedType = checker.getUndefinedType();
        const nullType = checker.getNullType();
        const neverType = checker.getNeverType();

        return {
            evaluate,
            getCached,
            clearCache,
        };

        function evaluate(expr: Expression): EvaluationResult {
            const nodeId = getNodeId(expr);
            const cached = cache.get(nodeId);
            if (cached) {
                return cached;
            }

            const result = evaluateExpression(expr);
            cache.set(nodeId, result);
            return result;
        }

        function getCached(expr: Expression): EvaluationResult | undefined {
            return cache.get(getNodeId(expr));
        }

        function clearCache(): void {
            cache.clear();
        }

        function evaluateExpression(expr: Expression): EvaluationResult {
            switch (expr.kind) {
                case SyntaxKind.NumericLiteral:
                    return evaluateNumericLiteral(expr as NumericLiteral);
                case SyntaxKind.StringLiteral:
                    return evaluateStringLiteral(expr as StringLiteral);
                case SyntaxKind.TrueKeyword:
                    return { type: booleanType, isLiteral: true, literalValue: true, isConstant: true };
                case SyntaxKind.FalseKeyword:
                    return { type: booleanType, isLiteral: true, literalValue: false, isConstant: true };
                case SyntaxKind.NullKeyword:
                    return { type: nullType, isLiteral: true, isConstant: true };
                case SyntaxKind.Identifier:
                    return evaluateIdentifier(expr as Identifier);
                case SyntaxKind.BinaryExpression:
                    return evaluateBinaryExpression(expr as BinaryExpression);
                case SyntaxKind.CallExpression:
                    return evaluateCallExpression(expr as CallExpression);
                case SyntaxKind.PropertyAccessExpression:
                    return evaluatePropertyAccess(expr as PropertyAccessExpression);
                case SyntaxKind.ParenthesizedExpression:
                    return evaluate((expr as ParenthesizedExpression).expression);
                case SyntaxKind.ConditionalExpression:
                    return evaluateConditional(expr as ConditionalExpression);
                default:
                    return evaluateWithFullChecker(expr);
            }
        }

        function evaluateNumericLiteral(node: NumericLiteral): EvaluationResult {
            const value = Number(node.text);
            return {
                type: numberType,
                isLiteral: true,
                literalValue: value,
                isConstant: true,
            };
        }

        function evaluateStringLiteral(node: StringLiteral): EvaluationResult {
            return {
                type: stringType,
                isLiteral: true,
                literalValue: node.text,
                isConstant: true,
            };
        }

        function evaluateIdentifier(node: Identifier): EvaluationResult {
            if (node.escapedText === "undefined") {
                return { type: undefinedType, isLiteral: true, isConstant: true };
            }

            const symbol = checker.getSymbolAtLocation(node);
            if (!symbol) {
                onError(node, Diagnostics.Cannot_find_name_0, idText(node));
                return { type: anyType, isLiteral: false, isConstant: false };
            }

            const type = checker.getTypeOfSymbolAtLocation(symbol, node);
            const isConst = isConstVariable(symbol);

            return {
                type,
                isLiteral: false,
                isConstant: isConst,
            };
        }

        function isConstVariable(symbol: Symbol): boolean {
            const declarations = symbol.declarations;
            if (!declarations || declarations.length === 0) {
                return false;
            }

            for (const decl of declarations) {
                if (isVariableDeclaration(decl)) {
                    const parent = decl.parent;
                    if (isVariableDeclarationList(parent)) {
                        return !!(parent.flags & NodeFlags.Const);
                    }
                }
            }

            return false;
        }

        function evaluateBinaryExpression(node: BinaryExpression): EvaluationResult {
            const left = evaluate(node.left);
            const right = evaluate(node.right);
            const operator = node.operatorToken.kind;

            switch (operator) {
                case SyntaxKind.PlusToken:
                    if (isStringLike(left.type) || isStringLike(right.type)) {
                        return { type: stringType, isLiteral: false, isConstant: left.isConstant && right.isConstant };
                    }
                    return { type: numberType, isLiteral: false, isConstant: left.isConstant && right.isConstant };

                case SyntaxKind.MinusToken:
                case SyntaxKind.AsteriskToken:
                case SyntaxKind.SlashToken:
                case SyntaxKind.PercentToken:
                    return { type: numberType, isLiteral: false, isConstant: left.isConstant && right.isConstant };

                case SyntaxKind.LessThanToken:
                case SyntaxKind.GreaterThanToken:
                case SyntaxKind.LessThanEqualsToken:
                case SyntaxKind.GreaterThanEqualsToken:
                case SyntaxKind.EqualsEqualsToken:
                case SyntaxKind.EqualsEqualsEqualsToken:
                case SyntaxKind.ExclamationEqualsToken:
                case SyntaxKind.ExclamationEqualsEqualsToken:
                    return { type: booleanType, isLiteral: false, isConstant: left.isConstant && right.isConstant };

                case SyntaxKind.EqualsToken:
                    return { type: right.type, isLiteral: false, isConstant: false };

                case SyntaxKind.AmpersandAmpersandToken:
                case SyntaxKind.BarBarToken:
                    return {
                        type: checker.getUnionType([left.type, right.type]),
                        isLiteral: false,
                        isConstant: false,
                    };

                default:
                    return evaluateWithFullChecker(node);
            }
        }

        function isStringLike(type: Type): boolean {
            return !!(type.flags & TypeFlags.StringLike);
        }

        function evaluateCallExpression(node: CallExpression): EvaluationResult {
            const calleeResult = evaluate(node.expression);
            const signatures = checker.getSignaturesOfType(calleeResult.type, SignatureKind.Call);

            if (signatures.length === 0) {
                if (!(calleeResult.type.flags & TypeFlags.Any)) {
                    onError(node.expression, Diagnostics.This_expression_is_not_callable);
                }
                return { type: anyType, isLiteral: false, isConstant: false };
            }

            // Evaluate arguments for side effects and constraint checking
            for (const arg of node.arguments) {
                evaluate(arg);
            }

            const returnType = checker.getReturnTypeOfSignature(signatures[0]);
            return { type: returnType, isLiteral: false, isConstant: false };
        }

        function evaluatePropertyAccess(node: PropertyAccessExpression): EvaluationResult {
            const objectResult = evaluate(node.expression);
            const propertyName = node.name.escapedText as string;

            const property = checker.getPropertyOfType(objectResult.type, propertyName);
            if (!property) {
                if (!(objectResult.type.flags & TypeFlags.Any)) {
                    onError(node.name, Diagnostics.Property_0_does_not_exist_on_type_1,
                        propertyName, checker.typeToString(objectResult.type));
                }
                return { type: anyType, isLiteral: false, isConstant: false };
            }

            const type = checker.getTypeOfSymbolAtLocation(property, node);
            return { type, isLiteral: false, isConstant: objectResult.isConstant };
        }

        function evaluateConditional(node: ConditionalExpression): EvaluationResult {
            evaluate(node.condition);
            const whenTrue = evaluate(node.whenTrue);
            const whenFalse = evaluate(node.whenFalse);

            return {
                type: checker.getUnionType([whenTrue.type, whenFalse.type]),
                isLiteral: false,
                isConstant: whenTrue.isConstant && whenFalse.isConstant,
            };
        }

        function evaluateWithFullChecker(node: Expression): EvaluationResult {
            try {
                const type = checker.getTypeAtLocation(node);
                return { type, isLiteral: false, isConstant: false };
            } catch {
                return { type: anyType, isLiteral: false, isConstant: false };
            }
        }
    }
}
