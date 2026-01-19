/**
 * Tests for the type narrowing module.
 *
 * These tests verify that the TypeNarrower class correctly implements
 * iterative (non-recursive) type narrowing to handle deeply nested
 * expressions without stack overflow.
 */

namespace ts {
    // Mock types for testing
    interface MockType extends Type {
        name: string;
    }

    function createMockType(name: string, flags: TypeFlags = TypeFlags.Object): MockType {
        return { name, flags } as MockType;
    }

    // Mock expression nodes
    function createMockIdentifier(name: string): Expression {
        return {
            kind: SyntaxKind.Identifier,
            text: name,
        } as any;
    }

    function createMockBinaryExpression(
        left: Expression,
        operator: SyntaxKind,
        right: Expression
    ): BinaryExpression {
        return {
            kind: SyntaxKind.BinaryExpression,
            left,
            operatorToken: { kind: operator },
            right,
        } as any;
    }

    function createMockPrefixUnary(operator: SyntaxKind, operand: Expression): PrefixUnaryExpression {
        return {
            kind: SyntaxKind.PrefixUnaryExpression,
            operator,
            operand,
        } as any;
    }

    function createMockParenthesized(expr: Expression): ParenthesizedExpression {
        return {
            kind: SyntaxKind.ParenthesizedExpression,
            expression: expr,
        } as any;
    }

    // Create mock callbacks
    function createMockNarrowingCallbacks(overrides: Partial<NarrowingCallbacks> = {}): NarrowingCallbacks {
        const neverType = createMockType("never", TypeFlags.Never);
        const unknownType = createMockType("unknown", TypeFlags.Unknown);

        return {
            getTypeOfExpression: () => createMockType("expression"),
            isMatchingReference: (source, target) => source === target,
            filterType: (type) => type,
            getUnionType: (types) => types[0] || neverType,
            getIntersectionType: (types) => types[0] || neverType,
            getTypeWithFacts: (type, facts) => {
                const mockType = type as MockType;
                return createMockType(`${mockType.name}_narrowed_${facts}`);
            },
            isTypeAssignableTo: () => true,
            getNeverType: () => neverType,
            getUnknownType: () => unknownType,
            optionalChainContainsReference: () => false,
            ...overrides,
        };
    }

    describe("TypeNarrower", () => {
        describe("constructor and configuration", () => {
            it("should create narrower with default config", () => {
                const callbacks = createMockNarrowingCallbacks();
                const narrower = createTypeNarrower(callbacks);
                assert.isDefined(narrower);
            });

            it("should create narrower with custom config", () => {
                const callbacks = createMockNarrowingCallbacks();
                const config: Partial<NarrowingConfig> = {
                    maxNarrowingDepth: 250,
                };
                const narrower = createTypeNarrowerWithConfig(callbacks, config);
                assert.isDefined(narrower);
            });
        });

        describe("narrowType", () => {
            it("should narrow matching reference by truthiness (assumeTrue)", () => {
                const callbacks = createMockNarrowingCallbacks({
                    isMatchingReference: () => true,
                });
                const narrower = createTypeNarrower(callbacks);

                const type = createMockType("input");
                const expr = createMockIdentifier("x");
                const reference = expr;

                const result = narrower.narrowType(type, expr, true, reference);

                assert.include((result as MockType).name, "narrowed");
                assert.include((result as MockType).name, String(NarrowingTypeFacts.Truthy));
            });

            it("should narrow matching reference by truthiness (assumeFalse)", () => {
                const callbacks = createMockNarrowingCallbacks({
                    isMatchingReference: () => true,
                });
                const narrower = createTypeNarrower(callbacks);

                const type = createMockType("input");
                const expr = createMockIdentifier("x");
                const reference = expr;

                const result = narrower.narrowType(type, expr, false, reference);

                assert.include((result as MockType).name, "narrowed");
                assert.include((result as MockType).name, String(NarrowingTypeFacts.Falsy));
            });

            it("should not narrow non-matching reference", () => {
                const callbacks = createMockNarrowingCallbacks({
                    isMatchingReference: () => false,
                });
                const narrower = createTypeNarrower(callbacks);

                const type = createMockType("input");
                const expr = createMockIdentifier("x");
                const reference = createMockIdentifier("y");

                const result = narrower.narrowType(type, expr, true, reference);

                assert.equal((result as MockType).name, "input");
            });
        });

        describe("logical operators", () => {
            it("should narrow && expression with assumeTrue", () => {
                const callbacks = createMockNarrowingCallbacks({
                    isMatchingReference: () => true,
                });
                const narrower = createTypeNarrower(callbacks);

                const type = createMockType("input");
                const left = createMockIdentifier("x");
                const right = createMockIdentifier("y");
                const expr = createMockBinaryExpression(
                    left,
                    SyntaxKind.AmpersandAmpersandToken,
                    right
                );
                const reference = left;

                const result = narrower.narrowType(type, expr, true, reference);

                // Should be narrowed by both left and right
                assert.include((result as MockType).name, "narrowed");
            });

            it("should narrow || expression with assumeFalse", () => {
                const callbacks = createMockNarrowingCallbacks({
                    isMatchingReference: () => true,
                });
                const narrower = createTypeNarrower(callbacks);

                const type = createMockType("input");
                const left = createMockIdentifier("x");
                const right = createMockIdentifier("y");
                const expr = createMockBinaryExpression(
                    left,
                    SyntaxKind.BarBarToken,
                    right
                );
                const reference = left;

                const result = narrower.narrowType(type, expr, false, reference);

                assert.include((result as MockType).name, "narrowed");
            });

            it("should handle negation (!) operator", () => {
                const callbacks = createMockNarrowingCallbacks({
                    isMatchingReference: () => true,
                });
                const narrower = createTypeNarrower(callbacks);

                const type = createMockType("input");
                const inner = createMockIdentifier("x");
                const expr = createMockPrefixUnary(SyntaxKind.ExclamationToken, inner);
                const reference = inner;

                // assumeTrue on !x means assumeFalse on x
                const result = narrower.narrowType(type, expr, true, reference);

                assert.include((result as MockType).name, String(NarrowingTypeFacts.Falsy));
            });
        });

        describe("parenthesized expressions", () => {
            it("should handle parenthesized expressions", () => {
                const callbacks = createMockNarrowingCallbacks({
                    isMatchingReference: () => true,
                });
                const narrower = createTypeNarrower(callbacks);

                const type = createMockType("input");
                const inner = createMockIdentifier("x");
                const expr = createMockParenthesized(inner);
                const reference = inner;

                const result = narrower.narrowType(type, expr, true, reference);

                assert.include((result as MockType).name, "narrowed");
            });
        });

        describe("deeply nested expressions", () => {
            it("should handle deeply nested && without stack overflow", () => {
                const callbacks = createMockNarrowingCallbacks({
                    isMatchingReference: () => true,
                });
                const narrower = createTypeNarrower(callbacks);

                const type = createMockType("input");
                const depth = 200;

                // Build deeply nested && expression: ((((x && x) && x) && x) ...)
                let expr: Expression = createMockIdentifier("x");
                for (let i = 0; i < depth; i++) {
                    expr = createMockBinaryExpression(
                        expr,
                        SyntaxKind.AmpersandAmpersandToken,
                        createMockIdentifier("x")
                    );
                }

                const reference = createMockIdentifier("x");

                // This should complete without stack overflow
                const result = narrower.narrowType(type, expr, true, reference);

                assert.isDefined(result);
            });

            it("should handle deeply nested || without stack overflow", () => {
                const callbacks = createMockNarrowingCallbacks({
                    isMatchingReference: () => true,
                });
                const narrower = createTypeNarrower(callbacks);

                const type = createMockType("input");
                const depth = 200;

                // Build deeply nested || expression
                let expr: Expression = createMockIdentifier("x");
                for (let i = 0; i < depth; i++) {
                    expr = createMockBinaryExpression(
                        expr,
                        SyntaxKind.BarBarToken,
                        createMockIdentifier("x")
                    );
                }

                const reference = createMockIdentifier("x");

                const result = narrower.narrowType(type, expr, false, reference);

                assert.isDefined(result);
            });

            it("should hit depth limit on excessive nesting", () => {
                const callbacks = createMockNarrowingCallbacks({
                    isMatchingReference: () => true,
                });
                const config: Partial<NarrowingConfig> = {
                    maxNarrowingDepth: 50,
                };
                const narrower = createTypeNarrowerWithConfig(callbacks, config);

                const type = createMockType("input");

                // Build expression deeper than the limit
                let expr: Expression = createMockIdentifier("x");
                for (let i = 0; i < 100; i++) {
                    expr = createMockParenthesized(expr);
                }

                const reference = createMockIdentifier("x");

                const result = narrower.narrowType(type, expr, true, reference);

                // Should return something without crashing
                assert.isDefined(result);
            });
        });

        describe("equality narrowing", () => {
            it("should narrow by === comparison", () => {
                const nullType = createMockType("null", TypeFlags.Null);
                const callbacks = createMockNarrowingCallbacks({
                    isMatchingReference: (source, target) => source === target,
                    getTypeOfExpression: () => nullType,
                });
                const narrower = createTypeNarrower(callbacks);

                const type = createMockType("input");
                const left = createMockIdentifier("x");
                const right = createMockIdentifier("null");
                const expr = createMockBinaryExpression(
                    left,
                    SyntaxKind.EqualsEqualsEqualsToken,
                    right
                );

                const result = narrower.narrowType(type, expr, true, left);

                assert.isDefined(result);
            });

            it("should narrow by !== comparison", () => {
                const callbacks = createMockNarrowingCallbacks({
                    isMatchingReference: (source, target) => source === target,
                });
                const narrower = createTypeNarrower(callbacks);

                const type = createMockType("input");
                const left = createMockIdentifier("x");
                const right = createMockIdentifier("null");
                const expr = createMockBinaryExpression(
                    left,
                    SyntaxKind.ExclamationEqualsEqualsToken,
                    right
                );

                const result = narrower.narrowType(type, expr, true, left);

                assert.isDefined(result);
            });
        });
    });

    describe("NarrowingTypeFacts utilities", () => {
        describe("getTypeFactsForTypeof", () => {
            it("should return TypeofString for 'string'", () => {
                assert.equal(getTypeFactsForTypeof("string"), NarrowingTypeFacts.TypeofString);
            });

            it("should return TypeofNumber for 'number'", () => {
                assert.equal(getTypeFactsForTypeof("number"), NarrowingTypeFacts.TypeofNumber);
            });

            it("should return TypeofBigInt for 'bigint'", () => {
                assert.equal(getTypeFactsForTypeof("bigint"), NarrowingTypeFacts.TypeofBigInt);
            });

            it("should return TypeofBoolean for 'boolean'", () => {
                assert.equal(getTypeFactsForTypeof("boolean"), NarrowingTypeFacts.TypeofBoolean);
            });

            it("should return TypeofSymbol for 'symbol'", () => {
                assert.equal(getTypeFactsForTypeof("symbol"), NarrowingTypeFacts.TypeofSymbol);
            });

            it("should return TypeofObject for 'object'", () => {
                assert.equal(getTypeFactsForTypeof("object"), NarrowingTypeFacts.TypeofObject);
            });

            it("should return TypeofFunction for 'function'", () => {
                assert.equal(getTypeFactsForTypeof("function"), NarrowingTypeFacts.TypeofFunction);
            });

            it("should return EQUndefined for 'undefined'", () => {
                assert.equal(getTypeFactsForTypeof("undefined"), NarrowingTypeFacts.EQUndefined);
            });

            it("should return None for unknown typeof string", () => {
                assert.equal(getTypeFactsForTypeof("unknown"), NarrowingTypeFacts.None);
            });
        });

        describe("getNegatedTypeFacts", () => {
            it("should negate TypeofString to TypeofNEString", () => {
                assert.equal(getNegatedTypeFacts(NarrowingTypeFacts.TypeofString), NarrowingTypeFacts.TypeofNEString);
            });

            it("should negate Truthy to Falsy", () => {
                assert.equal(getNegatedTypeFacts(NarrowingTypeFacts.Truthy), NarrowingTypeFacts.Falsy);
            });

            it("should negate Falsy to Truthy", () => {
                assert.equal(getNegatedTypeFacts(NarrowingTypeFacts.Falsy), NarrowingTypeFacts.Truthy);
            });

            it("should negate NEUndefined to EQUndefined", () => {
                assert.equal(getNegatedTypeFacts(NarrowingTypeFacts.NEUndefined), NarrowingTypeFacts.EQUndefined);
            });

            it("should negate EQNull to NENull", () => {
                assert.equal(getNegatedTypeFacts(NarrowingTypeFacts.EQNull), NarrowingTypeFacts.NENull);
            });

            it("should return None for None", () => {
                assert.equal(getNegatedTypeFacts(NarrowingTypeFacts.None), NarrowingTypeFacts.None);
            });
        });
    });

    describe("NarrowingConfig", () => {
        it("should have sensible defaults", () => {
            assert.equal(DEFAULT_NARROWING_CONFIG.maxNarrowingDepth, 500);
            assert.isTrue(DEFAULT_NARROWING_CONFIG.enableCaching);
        });
    });
}
