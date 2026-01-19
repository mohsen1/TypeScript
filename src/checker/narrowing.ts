/**
 * Type narrowing module for control flow analysis.
 *
 * This module implements type narrowing from control flow constructs,
 * including truthiness checks and type guards.
 */

namespace ts {
    /**
     * Type facts that can be inferred from control flow.
     * These represent what we know about a type in a particular branch.
     */
    export const enum NarrowingTypeFacts {
        None = 0,
        /** Type is known to be truthy */
        Truthy = 1 << 0,
        /** Type is known to be falsy */
        Falsy = 1 << 1,
        /** Type is not undefined */
        NEUndefined = 1 << 2,
        /** Type is not null */
        NENull = 1 << 3,
        /** Type is not null or undefined */
        NEUndefinedOrNull = NEUndefined | NENull,
        /** Type equals undefined */
        EQUndefined = 1 << 4,
        /** Type equals null */
        EQNull = 1 << 5,
        /** Type equals null or undefined */
        EQUndefinedOrNull = EQUndefined | EQNull,
        /** Type is a string */
        TypeofString = 1 << 6,
        /** Type is a number */
        TypeofNumber = 1 << 7,
        /** Type is a bigint */
        TypeofBigInt = 1 << 8,
        /** Type is a boolean */
        TypeofBoolean = 1 << 9,
        /** Type is a symbol */
        TypeofSymbol = 1 << 10,
        /** Type is an object */
        TypeofObject = 1 << 11,
        /** Type is a function */
        TypeofFunction = 1 << 12,
        /** Type is not a string */
        TypeofNEString = 1 << 13,
        /** Type is not a number */
        TypeofNENumber = 1 << 14,
        /** Type is not a bigint */
        TypeofNEBigInt = 1 << 15,
        /** Type is not a boolean */
        TypeofNEBoolean = 1 << 16,
        /** Type is not a symbol */
        TypeofNESymbol = 1 << 17,
        /** Type is not an object */
        TypeofNEObject = 1 << 18,
        /** Type is not a function */
        TypeofNEFunction = 1 << 19,
    }

    /**
     * Callbacks for type operations during narrowing.
     */
    export interface NarrowingCallbacks {
        /** Get the type of an expression */
        getTypeOfExpression(expr: Expression): Type;
        /** Check if two nodes refer to the same entity */
        isMatchingReference(source: Node, target: Node): boolean;
        /** Filter a type by a predicate */
        filterType(type: Type, predicate: (t: Type) => boolean): Type;
        /** Get the union of multiple types */
        getUnionType(types: Type[]): Type;
        /** Get the intersection of multiple types */
        getIntersectionType(types: Type[]): Type;
        /** Create a type with certain facts applied */
        getTypeWithFacts(type: Type, facts: NarrowingTypeFacts): Type;
        /** Check if a type is assignable to another */
        isTypeAssignableTo(source: Type, target: Type): boolean;
        /** Get the never type */
        getNeverType(): Type;
        /** Get the unknown type */
        getUnknownType(): Type;
        /** Check if expr is an optional chain containing reference */
        optionalChainContainsReference(expr: Expression, reference: Node): boolean;
    }

    /**
     * Configuration for narrowing with recursion limits.
     */
    export interface NarrowingConfig {
        /** Maximum narrowing depth (for nested expressions) */
        maxNarrowingDepth: number;
        /** Enable caching of narrowing results */
        enableCaching: boolean;
    }

    /**
     * Default narrowing configuration.
     */
    export const DEFAULT_NARROWING_CONFIG: NarrowingConfig = {
        maxNarrowingDepth: 500,
        enableCaching: true,
    };

    /**
     * Work item for iterative narrowing.
     */
    interface NarrowingWorkItem {
        /** The expression being narrowed */
        expr: Expression;
        /** The type to narrow */
        type: Type;
        /** Whether we're assuming the condition is true */
        assumeTrue: boolean;
        /** Current narrowing depth */
        depth: number;
        /** Parent work item index */
        parentIndex: number;
        /** Result type (set when complete) */
        result?: Type;
        /** Intermediate results for compound expressions */
        intermediateResults?: Type[];
        /** Processing state */
        state: NarrowingState;
    }

    /**
     * Processing state for narrowing.
     */
    const enum NarrowingState {
        Pending = 0,
        ProcessingLeft = 1,
        ProcessingRight = 2,
        Complete = 3,
    }

    /**
     * Type narrowing engine.
     *
     * Implements iterative (non-recursive) type narrowing to handle
     * deeply nested expressions without stack overflow.
     */
    export class TypeNarrower {
        private readonly callbacks: NarrowingCallbacks;
        private readonly config: NarrowingConfig;

        /** Reference node being analyzed */
        private reference: Node | undefined;

        /** Worklist for iterative processing */
        private worklist: NarrowingWorkItem[] = [];

        /** Current iteration count */
        private iterations: number = 0;

        /** Maximum iterations allowed */
        private readonly maxIterations: number = 100000;

        constructor(
            callbacks: NarrowingCallbacks,
            config: NarrowingConfig = DEFAULT_NARROWING_CONFIG
        ) {
            this.callbacks = callbacks;
            this.config = config;
        }

        /**
         * Narrow a type based on a conditional expression.
         * Uses iterative processing to avoid stack overflow.
         */
        public narrowType(
            type: Type,
            expr: Expression,
            assumeTrue: boolean,
            reference: Node
        ): Type {
            this.reference = reference;
            this.worklist = [];
            this.iterations = 0;

            // Push initial work item
            this.worklist.push({
                expr,
                type,
                assumeTrue,
                depth: 0,
                parentIndex: -1,
                state: NarrowingState.Pending,
            });

            // Process iteratively
            return this.processNarrowingWorklist();
        }

        /**
         * Main worklist processing loop for narrowing.
         */
        private processNarrowingWorklist(): Type {
            while (this.worklist.length > 0) {
                this.iterations++;

                if (this.iterations > this.maxIterations) {
                    // Hit limit, return unchanged type
                    return this.worklist[0].type;
                }

                const currentIndex = this.worklist.length - 1;
                const workItem = this.worklist[currentIndex];

                if (workItem.state === NarrowingState.Complete) {
                    this.worklist.pop();

                    if (workItem.parentIndex >= 0 && workItem.result !== undefined) {
                        this.propagateNarrowingResult(workItem.parentIndex, workItem.result);
                    }
                    else if (workItem.parentIndex < 0) {
                        return workItem.result || workItem.type;
                    }
                    continue;
                }

                // Check depth limit
                if (workItem.depth >= this.config.maxNarrowingDepth) {
                    workItem.result = workItem.type;
                    workItem.state = NarrowingState.Complete;
                    continue;
                }

                this.processNarrowingItem(workItem, currentIndex);
            }

            return this.callbacks.getNeverType();
        }

        /**
         * Process a single narrowing work item.
         */
        private processNarrowingItem(workItem: NarrowingWorkItem, currentIndex: number): void {
            const { expr, type, assumeTrue, depth } = workItem;
            const kind = expr.kind;

            switch (kind) {
                case SyntaxKind.ParenthesizedExpression:
                    this.processParenthesized(workItem, currentIndex);
                    break;

                case SyntaxKind.BinaryExpression:
                    this.processBinaryExpression(workItem, currentIndex);
                    break;

                case SyntaxKind.PrefixUnaryExpression:
                    this.processPrefixUnary(workItem, currentIndex);
                    break;

                case SyntaxKind.TypeOfExpression:
                    workItem.result = this.narrowByTypeof(type, expr, assumeTrue);
                    workItem.state = NarrowingState.Complete;
                    break;

                case SyntaxKind.CallExpression:
                    workItem.result = this.narrowByCall(type, expr, assumeTrue);
                    workItem.state = NarrowingState.Complete;
                    break;

                case SyntaxKind.Identifier:
                case SyntaxKind.PropertyAccessExpression:
                case SyntaxKind.ElementAccessExpression:
                    workItem.result = this.narrowByTruthiness(type, expr, assumeTrue);
                    workItem.state = NarrowingState.Complete;
                    break;

                default:
                    // No narrowing for this expression type
                    workItem.result = type;
                    workItem.state = NarrowingState.Complete;
                    break;
            }
        }

        /**
         * Process parenthesized expression.
         */
        private processParenthesized(workItem: NarrowingWorkItem, currentIndex: number): void {
            const expr = workItem.expr as ParenthesizedExpression;
            const innerExpr = expr.expression;

            if (workItem.state === NarrowingState.Pending) {
                workItem.state = NarrowingState.ProcessingLeft;
                this.worklist.push({
                    expr: innerExpr,
                    type: workItem.type,
                    assumeTrue: workItem.assumeTrue,
                    depth: workItem.depth + 1,
                    parentIndex: currentIndex,
                    state: NarrowingState.Pending,
                });
            }
            else {
                // Result from inner expression
                workItem.result = workItem.intermediateResults?.[0] || workItem.type;
                workItem.state = NarrowingState.Complete;
            }
        }

        /**
         * Process binary expression (&&, ||, comparisons, etc.).
         */
        private processBinaryExpression(workItem: NarrowingWorkItem, currentIndex: number): void {
            const expr = workItem.expr as BinaryExpression;
            const operator = expr.operatorToken.kind;

            switch (operator) {
                case SyntaxKind.AmpersandAmpersandToken:
                    this.processLogicalAnd(workItem, currentIndex, expr);
                    break;

                case SyntaxKind.BarBarToken:
                    this.processLogicalOr(workItem, currentIndex, expr);
                    break;

                case SyntaxKind.EqualsEqualsToken:
                case SyntaxKind.ExclamationEqualsToken:
                case SyntaxKind.EqualsEqualsEqualsToken:
                case SyntaxKind.ExclamationEqualsEqualsToken:
                    workItem.result = this.narrowByEquality(
                        workItem.type,
                        expr,
                        workItem.assumeTrue
                    );
                    workItem.state = NarrowingState.Complete;
                    break;

                case SyntaxKind.InstanceOfKeyword:
                    workItem.result = this.narrowByInstanceOf(
                        workItem.type,
                        expr,
                        workItem.assumeTrue
                    );
                    workItem.state = NarrowingState.Complete;
                    break;

                case SyntaxKind.InKeyword:
                    workItem.result = this.narrowByInOperator(
                        workItem.type,
                        expr,
                        workItem.assumeTrue
                    );
                    workItem.state = NarrowingState.Complete;
                    break;

                default:
                    workItem.result = workItem.type;
                    workItem.state = NarrowingState.Complete;
                    break;
            }
        }

        /**
         * Process logical AND (&&) expressions iteratively.
         */
        private processLogicalAnd(
            workItem: NarrowingWorkItem,
            currentIndex: number,
            expr: BinaryExpression
        ): void {
            const { type, assumeTrue, depth } = workItem;

            if (assumeTrue) {
                // For && with assumeTrue, narrow left then right
                if (workItem.state === NarrowingState.Pending) {
                    workItem.state = NarrowingState.ProcessingLeft;
                    this.worklist.push({
                        expr: expr.left,
                        type,
                        assumeTrue: true,
                        depth: depth + 1,
                        parentIndex: currentIndex,
                        state: NarrowingState.Pending,
                    });
                }
                else if (workItem.state === NarrowingState.ProcessingLeft) {
                    const leftResult = workItem.intermediateResults?.[0] || type;
                    workItem.state = NarrowingState.ProcessingRight;
                    this.worklist.push({
                        expr: expr.right,
                        type: leftResult,
                        assumeTrue: true,
                        depth: depth + 1,
                        parentIndex: currentIndex,
                        state: NarrowingState.Pending,
                    });
                }
                else {
                    workItem.result = workItem.intermediateResults?.[1] || workItem.intermediateResults?.[0] || type;
                    workItem.state = NarrowingState.Complete;
                }
            }
            else {
                // For && with assumeFalse, union of (left false) and (left true, right false)
                if (workItem.state === NarrowingState.Pending) {
                    workItem.state = NarrowingState.ProcessingLeft;
                    workItem.intermediateResults = [];
                    this.worklist.push({
                        expr: expr.left,
                        type,
                        assumeTrue: false,
                        depth: depth + 1,
                        parentIndex: currentIndex,
                        state: NarrowingState.Pending,
                    });
                }
                else if (workItem.state === NarrowingState.ProcessingLeft) {
                    workItem.state = NarrowingState.ProcessingRight;
                    this.worklist.push({
                        expr: expr.right,
                        type,
                        assumeTrue: false,
                        depth: depth + 1,
                        parentIndex: currentIndex,
                        state: NarrowingState.Pending,
                    });
                }
                else {
                    const leftFalse = workItem.intermediateResults?.[0] || type;
                    const rightFalse = workItem.intermediateResults?.[1] || type;
                    workItem.result = this.callbacks.getUnionType([leftFalse, rightFalse]);
                    workItem.state = NarrowingState.Complete;
                }
            }
        }

        /**
         * Process logical OR (||) expressions iteratively.
         */
        private processLogicalOr(
            workItem: NarrowingWorkItem,
            currentIndex: number,
            expr: BinaryExpression
        ): void {
            const { type, assumeTrue, depth } = workItem;

            if (assumeTrue) {
                // For || with assumeTrue, union of (left true) and (left false, right true)
                if (workItem.state === NarrowingState.Pending) {
                    workItem.state = NarrowingState.ProcessingLeft;
                    workItem.intermediateResults = [];
                    this.worklist.push({
                        expr: expr.left,
                        type,
                        assumeTrue: true,
                        depth: depth + 1,
                        parentIndex: currentIndex,
                        state: NarrowingState.Pending,
                    });
                }
                else if (workItem.state === NarrowingState.ProcessingLeft) {
                    workItem.state = NarrowingState.ProcessingRight;
                    this.worklist.push({
                        expr: expr.right,
                        type,
                        assumeTrue: true,
                        depth: depth + 1,
                        parentIndex: currentIndex,
                        state: NarrowingState.Pending,
                    });
                }
                else {
                    const leftTrue = workItem.intermediateResults?.[0] || type;
                    const rightTrue = workItem.intermediateResults?.[1] || type;
                    workItem.result = this.callbacks.getUnionType([leftTrue, rightTrue]);
                    workItem.state = NarrowingState.Complete;
                }
            }
            else {
                // For || with assumeFalse, narrow left then right
                if (workItem.state === NarrowingState.Pending) {
                    workItem.state = NarrowingState.ProcessingLeft;
                    this.worklist.push({
                        expr: expr.left,
                        type,
                        assumeTrue: false,
                        depth: depth + 1,
                        parentIndex: currentIndex,
                        state: NarrowingState.Pending,
                    });
                }
                else if (workItem.state === NarrowingState.ProcessingLeft) {
                    const leftResult = workItem.intermediateResults?.[0] || type;
                    workItem.state = NarrowingState.ProcessingRight;
                    this.worklist.push({
                        expr: expr.right,
                        type: leftResult,
                        assumeTrue: false,
                        depth: depth + 1,
                        parentIndex: currentIndex,
                        state: NarrowingState.Pending,
                    });
                }
                else {
                    workItem.result = workItem.intermediateResults?.[1] || workItem.intermediateResults?.[0] || type;
                    workItem.state = NarrowingState.Complete;
                }
            }
        }

        /**
         * Process prefix unary expression (! operator).
         */
        private processPrefixUnary(workItem: NarrowingWorkItem, currentIndex: number): void {
            const expr = workItem.expr as PrefixUnaryExpression;

            if (expr.operator === SyntaxKind.ExclamationToken) {
                // Negate the assumption
                if (workItem.state === NarrowingState.Pending) {
                    workItem.state = NarrowingState.ProcessingLeft;
                    this.worklist.push({
                        expr: expr.operand,
                        type: workItem.type,
                        assumeTrue: !workItem.assumeTrue,
                        depth: workItem.depth + 1,
                        parentIndex: currentIndex,
                        state: NarrowingState.Pending,
                    });
                }
                else {
                    workItem.result = workItem.intermediateResults?.[0] || workItem.type;
                    workItem.state = NarrowingState.Complete;
                }
            }
            else {
                workItem.result = workItem.type;
                workItem.state = NarrowingState.Complete;
            }
        }

        /**
         * Propagate narrowing result to parent work item.
         */
        private propagateNarrowingResult(parentIndex: number, result: Type): void {
            const parent = this.worklist[parentIndex];
            if (!parent) return;

            if (!parent.intermediateResults) {
                parent.intermediateResults = [];
            }
            parent.intermediateResults.push(result);
        }

        /**
         * Narrow type based on truthiness.
         */
        private narrowByTruthiness(type: Type, expr: Expression, assumeTrue: boolean): Type {
            if (!this.reference) return type;

            if (this.callbacks.isMatchingReference(this.reference, expr)) {
                if (type.flags & TypeFlags.Unknown && assumeTrue) {
                    return this.callbacks.getUnknownType();
                }
                return this.callbacks.getTypeWithFacts(
                    type,
                    assumeTrue ? NarrowingTypeFacts.Truthy : NarrowingTypeFacts.Falsy
                );
            }

            // Handle optional chain containment
            if (assumeTrue && this.callbacks.optionalChainContainsReference(expr, this.reference)) {
                return this.callbacks.getTypeWithFacts(type, NarrowingTypeFacts.NEUndefinedOrNull);
            }

            return type;
        }

        /**
         * Narrow type based on typeof expression.
         */
        private narrowByTypeof(type: Type, expr: Expression, assumeTrue: boolean): Type {
            // In full implementation, would check typeof comparisons
            // For now, return type unchanged
            return type;
        }

        /**
         * Narrow type based on call expression (type guards, assertions).
         */
        private narrowByCall(type: Type, expr: Expression, assumeTrue: boolean): Type {
            // In full implementation, would check for type predicates
            // For now, return type unchanged
            return type;
        }

        /**
         * Narrow type based on equality comparison.
         */
        private narrowByEquality(
            type: Type,
            expr: BinaryExpression,
            assumeTrue: boolean
        ): Type {
            const operator = expr.operatorToken.kind;
            const isStrictEquality =
                operator === SyntaxKind.EqualsEqualsEqualsToken ||
                operator === SyntaxKind.ExclamationEqualsEqualsToken;
            const isNegation =
                operator === SyntaxKind.ExclamationEqualsToken ||
                operator === SyntaxKind.ExclamationEqualsEqualsToken;

            const effectiveAssumeTrue = isNegation ? !assumeTrue : assumeTrue;

            // Check for null/undefined comparisons
            const left = expr.left;
            const right = expr.right;

            if (this.reference && this.callbacks.isMatchingReference(this.reference, left)) {
                const rightType = this.callbacks.getTypeOfExpression(right);
                return this.narrowByEqualityToType(type, rightType, effectiveAssumeTrue, isStrictEquality);
            }

            if (this.reference && this.callbacks.isMatchingReference(this.reference, right)) {
                const leftType = this.callbacks.getTypeOfExpression(left);
                return this.narrowByEqualityToType(type, leftType, effectiveAssumeTrue, isStrictEquality);
            }

            return type;
        }

        /**
         * Narrow type based on equality to another type.
         */
        private narrowByEqualityToType(
            type: Type,
            valueType: Type,
            assumeTrue: boolean,
            strictEquality: boolean
        ): Type {
            const nullableFlags = strictEquality ? TypeFlags.Undefined : (TypeFlags.Null | TypeFlags.Undefined);

            // Check if comparing to null/undefined
            if (valueType.flags & nullableFlags) {
                if (assumeTrue) {
                    // Type is null/undefined
                    return this.callbacks.getTypeWithFacts(type, NarrowingTypeFacts.EQUndefinedOrNull);
                }
                else {
                    // Type is not null/undefined
                    return this.callbacks.getTypeWithFacts(type, NarrowingTypeFacts.NEUndefinedOrNull);
                }
            }

            return type;
        }

        /**
         * Narrow type based on instanceof check.
         */
        private narrowByInstanceOf(
            type: Type,
            expr: BinaryExpression,
            assumeTrue: boolean
        ): Type {
            if (!this.reference) return type;

            const left = expr.left;

            if (!this.callbacks.isMatchingReference(this.reference, left)) {
                return type;
            }

            // Get the constructor type from the right side
            const rightType = this.callbacks.getTypeOfExpression(expr.right);

            // In full implementation, would narrow to instance type
            // For now, return type unchanged
            return type;
        }

        /**
         * Narrow type based on 'in' operator.
         */
        private narrowByInOperator(
            type: Type,
            expr: BinaryExpression,
            assumeTrue: boolean
        ): Type {
            if (!this.reference) return type;

            const right = expr.right;

            if (!this.callbacks.isMatchingReference(this.reference, right)) {
                return type;
            }

            // In full implementation, would narrow based on property presence
            // For now, return type unchanged
            return type;
        }
    }

    /**
     * Create a type narrower with default configuration.
     */
    export function createTypeNarrower(callbacks: NarrowingCallbacks): TypeNarrower {
        return new TypeNarrower(callbacks);
    }

    /**
     * Create a type narrower with custom configuration.
     */
    export function createTypeNarrowerWithConfig(
        callbacks: NarrowingCallbacks,
        config: Partial<NarrowingConfig>
    ): TypeNarrower {
        return new TypeNarrower(callbacks, { ...DEFAULT_NARROWING_CONFIG, ...config });
    }

    /**
     * Get type facts for a given typeof string.
     */
    export function getTypeFactsForTypeof(typeofString: string): NarrowingTypeFacts {
        switch (typeofString) {
            case "string":
                return NarrowingTypeFacts.TypeofString;
            case "number":
                return NarrowingTypeFacts.TypeofNumber;
            case "bigint":
                return NarrowingTypeFacts.TypeofBigInt;
            case "boolean":
                return NarrowingTypeFacts.TypeofBoolean;
            case "symbol":
                return NarrowingTypeFacts.TypeofSymbol;
            case "object":
                return NarrowingTypeFacts.TypeofObject;
            case "function":
                return NarrowingTypeFacts.TypeofFunction;
            case "undefined":
                return NarrowingTypeFacts.EQUndefined;
            default:
                return NarrowingTypeFacts.None;
        }
    }

    /**
     * Get the negation of a type fact.
     */
    export function getNegatedTypeFacts(facts: NarrowingTypeFacts): NarrowingTypeFacts {
        switch (facts) {
            case NarrowingTypeFacts.TypeofString:
                return NarrowingTypeFacts.TypeofNEString;
            case NarrowingTypeFacts.TypeofNumber:
                return NarrowingTypeFacts.TypeofNENumber;
            case NarrowingTypeFacts.TypeofBigInt:
                return NarrowingTypeFacts.TypeofNEBigInt;
            case NarrowingTypeFacts.TypeofBoolean:
                return NarrowingTypeFacts.TypeofNEBoolean;
            case NarrowingTypeFacts.TypeofSymbol:
                return NarrowingTypeFacts.TypeofNESymbol;
            case NarrowingTypeFacts.TypeofObject:
                return NarrowingTypeFacts.TypeofNEObject;
            case NarrowingTypeFacts.TypeofFunction:
                return NarrowingTypeFacts.TypeofNEFunction;
            case NarrowingTypeFacts.EQUndefined:
                return NarrowingTypeFacts.NEUndefined;
            case NarrowingTypeFacts.EQNull:
                return NarrowingTypeFacts.NENull;
            case NarrowingTypeFacts.NEUndefined:
                return NarrowingTypeFacts.EQUndefined;
            case NarrowingTypeFacts.NENull:
                return NarrowingTypeFacts.EQNull;
            case NarrowingTypeFacts.Truthy:
                return NarrowingTypeFacts.Falsy;
            case NarrowingTypeFacts.Falsy:
                return NarrowingTypeFacts.Truthy;
            default:
                return NarrowingTypeFacts.None;
        }
    }
}
