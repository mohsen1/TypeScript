describe("unittests:: Template Literal Types", () => {
    describe("String Manipulation - Pure Functions", () => {
        describe("applyUppercase", () => {
            it("should convert to uppercase", () => {
                assert.equal(ts.templateLiterals.applyUppercase("hello"), "HELLO");
                assert.equal(ts.templateLiterals.applyUppercase("World"), "WORLD");
                assert.equal(ts.templateLiterals.applyUppercase(""), "");
                assert.equal(ts.templateLiterals.applyUppercase("123abc"), "123ABC");
            });
        });

        describe("applyLowercase", () => {
            it("should convert to lowercase", () => {
                assert.equal(ts.templateLiterals.applyLowercase("HELLO"), "hello");
                assert.equal(ts.templateLiterals.applyLowercase("World"), "world");
                assert.equal(ts.templateLiterals.applyLowercase(""), "");
                assert.equal(ts.templateLiterals.applyLowercase("123ABC"), "123abc");
            });
        });

        describe("applyCapitalize", () => {
            it("should capitalize first character", () => {
                assert.equal(ts.templateLiterals.applyCapitalize("hello"), "Hello");
                assert.equal(ts.templateLiterals.applyCapitalize("world"), "World");
                assert.equal(ts.templateLiterals.applyCapitalize(""), "");
                assert.equal(ts.templateLiterals.applyCapitalize("a"), "A");
                assert.equal(ts.templateLiterals.applyCapitalize("ABC"), "ABC");
            });

            it("should preserve rest of string", () => {
                assert.equal(ts.templateLiterals.applyCapitalize("helloWORLD"), "HelloWORLD");
                assert.equal(ts.templateLiterals.applyCapitalize("123abc"), "123abc");
            });
        });

        describe("applyUncapitalize", () => {
            it("should uncapitalize first character", () => {
                assert.equal(ts.templateLiterals.applyUncapitalize("Hello"), "hello");
                assert.equal(ts.templateLiterals.applyUncapitalize("World"), "world");
                assert.equal(ts.templateLiterals.applyUncapitalize(""), "");
                assert.equal(ts.templateLiterals.applyUncapitalize("A"), "a");
                assert.equal(ts.templateLiterals.applyUncapitalize("abc"), "abc");
            });

            it("should preserve rest of string", () => {
                assert.equal(ts.templateLiterals.applyUncapitalize("HelloWORLD"), "helloWORLD");
                assert.equal(ts.templateLiterals.applyUncapitalize("ABC"), "aBC");
            });
        });

        describe("applyStringTransformation", () => {
            it("should apply correct transformation by kind", () => {
                const { IntrinsicTypeKind } = ts.templateLiterals;

                assert.equal(
                    ts.templateLiterals.applyStringTransformation("hello", IntrinsicTypeKind.Uppercase),
                    "HELLO"
                );
                assert.equal(
                    ts.templateLiterals.applyStringTransformation("HELLO", IntrinsicTypeKind.Lowercase),
                    "hello"
                );
                assert.equal(
                    ts.templateLiterals.applyStringTransformation("hello", IntrinsicTypeKind.Capitalize),
                    "Hello"
                );
                assert.equal(
                    ts.templateLiterals.applyStringTransformation("Hello", IntrinsicTypeKind.Uncapitalize),
                    "hello"
                );
            });
        });
    });

    describe("Intrinsic Type Detection", () => {
        describe("isIntrinsicStringType", () => {
            it("should recognize intrinsic type names", () => {
                assert.isTrue(ts.templateLiterals.isIntrinsicStringType("Uppercase"));
                assert.isTrue(ts.templateLiterals.isIntrinsicStringType("Lowercase"));
                assert.isTrue(ts.templateLiterals.isIntrinsicStringType("Capitalize"));
                assert.isTrue(ts.templateLiterals.isIntrinsicStringType("Uncapitalize"));
            });

            it("should reject non-intrinsic names", () => {
                assert.isFalse(ts.templateLiterals.isIntrinsicStringType("String"));
                assert.isFalse(ts.templateLiterals.isIntrinsicStringType("number"));
                assert.isFalse(ts.templateLiterals.isIntrinsicStringType("uppercase")); // case sensitive
                assert.isFalse(ts.templateLiterals.isIntrinsicStringType("UPPERCASE"));
            });
        });

        describe("getIntrinsicTypeKind", () => {
            it("should return correct kind for intrinsic types", () => {
                const { IntrinsicTypeKind } = ts.templateLiterals;

                assert.equal(
                    ts.templateLiterals.getIntrinsicTypeKind("Uppercase"),
                    IntrinsicTypeKind.Uppercase
                );
                assert.equal(
                    ts.templateLiterals.getIntrinsicTypeKind("Lowercase"),
                    IntrinsicTypeKind.Lowercase
                );
                assert.equal(
                    ts.templateLiterals.getIntrinsicTypeKind("Capitalize"),
                    IntrinsicTypeKind.Capitalize
                );
                assert.equal(
                    ts.templateLiterals.getIntrinsicTypeKind("Uncapitalize"),
                    IntrinsicTypeKind.Uncapitalize
                );
            });

            it("should return undefined for non-intrinsic names", () => {
                assert.isUndefined(ts.templateLiterals.getIntrinsicTypeKind("String"));
                assert.isUndefined(ts.templateLiterals.getIntrinsicTypeKind("foo"));
            });
        });
    });

    describe("Template Literal Type Info", () => {
        // These tests need a TypeChecker context
        // We'll test the pure utility functions that don't need the checker

        describe("canBeInterpolated", () => {
            it("should return true for interpolatable flag combinations", () => {
                // Create mock types with specific flags
                const stringLiteralMock = { flags: ts.TypeFlags.StringLiteral };
                const numberLiteralMock = { flags: ts.TypeFlags.NumberLiteral };
                const bigintLiteralMock = { flags: ts.TypeFlags.BigIntLiteral };
                const booleanLiteralMock = { flags: ts.TypeFlags.BooleanLiteral };
                const nullMock = { flags: ts.TypeFlags.Null };
                const undefinedMock = { flags: ts.TypeFlags.Undefined };
                const stringMock = { flags: ts.TypeFlags.String };
                const numberMock = { flags: ts.TypeFlags.Number };

                assert.isTrue(ts.templateLiterals.canBeInterpolated(stringLiteralMock as ts.Type));
                assert.isTrue(ts.templateLiterals.canBeInterpolated(numberLiteralMock as ts.Type));
                assert.isTrue(ts.templateLiterals.canBeInterpolated(bigintLiteralMock as ts.Type));
                assert.isTrue(ts.templateLiterals.canBeInterpolated(booleanLiteralMock as ts.Type));
                assert.isTrue(ts.templateLiterals.canBeInterpolated(nullMock as ts.Type));
                assert.isTrue(ts.templateLiterals.canBeInterpolated(undefinedMock as ts.Type));
                assert.isTrue(ts.templateLiterals.canBeInterpolated(stringMock as ts.Type));
                assert.isTrue(ts.templateLiterals.canBeInterpolated(numberMock as ts.Type));
            });

            it("should return false for non-interpolatable types", () => {
                const objectMock = { flags: ts.TypeFlags.Object };
                const anyMock = { flags: ts.TypeFlags.Any };

                assert.isFalse(ts.templateLiterals.canBeInterpolated(objectMock as ts.Type));
                assert.isFalse(ts.templateLiterals.canBeInterpolated(anyMock as ts.Type));
            });

            it("should handle union types", () => {
                const unionOfStrings = {
                    flags: ts.TypeFlags.Union,
                    types: [
                        { flags: ts.TypeFlags.StringLiteral },
                        { flags: ts.TypeFlags.StringLiteral },
                    ],
                };

                const unionWithObject = {
                    flags: ts.TypeFlags.Union,
                    types: [
                        { flags: ts.TypeFlags.StringLiteral },
                        { flags: ts.TypeFlags.Object },
                    ],
                };

                assert.isTrue(ts.templateLiterals.canBeInterpolated(unionOfStrings as unknown as ts.Type));
                assert.isFalse(ts.templateLiterals.canBeInterpolated(unionWithObject as unknown as ts.Type));
            });
        });

        describe("getLiteralStringValue", () => {
            it("should extract string literal values", () => {
                const stringLiteral = {
                    flags: ts.TypeFlags.StringLiteral,
                    value: "hello",
                };

                assert.equal(
                    ts.templateLiterals.getLiteralStringValue(stringLiteral as ts.Type),
                    "hello"
                );
            });

            it("should convert number literals to strings", () => {
                const numberLiteral = {
                    flags: ts.TypeFlags.NumberLiteral,
                    value: 42,
                };

                assert.equal(
                    ts.templateLiterals.getLiteralStringValue(numberLiteral as ts.Type),
                    "42"
                );
            });

            it("should handle bigint literals", () => {
                const bigintLiteral = {
                    flags: ts.TypeFlags.BigIntLiteral,
                    value: { negative: false, base10Value: "123" },
                };

                assert.equal(
                    ts.templateLiterals.getLiteralStringValue(bigintLiteral as ts.Type),
                    "123"
                );

                const negativeBigint = {
                    flags: ts.TypeFlags.BigIntLiteral,
                    value: { negative: true, base10Value: "456" },
                };

                assert.equal(
                    ts.templateLiterals.getLiteralStringValue(negativeBigint as ts.Type),
                    "-456"
                );
            });

            it("should handle null and undefined", () => {
                const nullType = { flags: ts.TypeFlags.Null };
                const undefinedType = { flags: ts.TypeFlags.Undefined };

                assert.equal(
                    ts.templateLiterals.getLiteralStringValue(nullType as ts.Type),
                    "null"
                );
                assert.equal(
                    ts.templateLiterals.getLiteralStringValue(undefinedType as ts.Type),
                    "undefined"
                );
            });

            it("should return undefined for non-literal types", () => {
                const stringType = { flags: ts.TypeFlags.String };
                const objectType = { flags: ts.TypeFlags.Object };

                assert.isUndefined(ts.templateLiterals.getLiteralStringValue(stringType as ts.Type));
                assert.isUndefined(ts.templateLiterals.getLiteralStringValue(objectType as ts.Type));
            });
        });
    });

    describe("Template Helpers", () => {
        // Test that helpers exist and are properly typed
        it("should have helper methods", () => {
            assert.isDefined(ts.templateLiterals.TemplateHelpers);
            assert.isFunction(ts.templateLiterals.TemplateHelpers.prefixPattern);
            assert.isFunction(ts.templateLiterals.TemplateHelpers.suffixPattern);
            assert.isFunction(ts.templateLiterals.TemplateHelpers.containsPattern);
            assert.isFunction(ts.templateLiterals.TemplateHelpers.createPattern);
        });
    });

    describe("Type Checks", () => {
        it("isTemplateLiteralType should check flag", () => {
            const templateType = { flags: ts.TypeFlags.TemplateLiteral };
            const stringType = { flags: ts.TypeFlags.String };

            assert.isTrue(ts.templateLiterals.isTemplateLiteralType(templateType as ts.Type));
            assert.isFalse(ts.templateLiterals.isTemplateLiteralType(stringType as ts.Type));
        });

        it("isStringLiteralType should check flag", () => {
            const stringLiteralType = { flags: ts.TypeFlags.StringLiteral };
            const stringType = { flags: ts.TypeFlags.String };

            assert.isTrue(ts.templateLiterals.isStringLiteralType(stringLiteralType as ts.Type));
            assert.isFalse(ts.templateLiterals.isStringLiteralType(stringType as ts.Type));
        });
    });

    describe("IntrinsicTypeKind enum", () => {
        it("should have expected values", () => {
            const { IntrinsicTypeKind } = ts.templateLiterals;

            assert.equal(IntrinsicTypeKind.Uppercase, "Uppercase");
            assert.equal(IntrinsicTypeKind.Lowercase, "Lowercase");
            assert.equal(IntrinsicTypeKind.Capitalize, "Capitalize");
            assert.equal(IntrinsicTypeKind.Uncapitalize, "Uncapitalize");
        });
    });

    describe("intrinsicTypeMap", () => {
        it("should have all intrinsic types mapped", () => {
            const { intrinsicTypeMap, IntrinsicTypeKind } = ts.templateLiterals;

            assert.isDefined(intrinsicTypeMap[IntrinsicTypeKind.Uppercase]);
            assert.isDefined(intrinsicTypeMap[IntrinsicTypeKind.Lowercase]);
            assert.isDefined(intrinsicTypeMap[IntrinsicTypeKind.Capitalize]);
            assert.isDefined(intrinsicTypeMap[IntrinsicTypeKind.Uncapitalize]);

            assert.isFunction(intrinsicTypeMap[IntrinsicTypeKind.Uppercase]);
            assert.isFunction(intrinsicTypeMap[IntrinsicTypeKind.Lowercase]);
            assert.isFunction(intrinsicTypeMap[IntrinsicTypeKind.Capitalize]);
            assert.isFunction(intrinsicTypeMap[IntrinsicTypeKind.Uncapitalize]);
        });
    });

    describe("Edge Cases", () => {
        describe("Unicode handling", () => {
            it("should handle unicode characters", () => {
                assert.equal(ts.templateLiterals.applyUppercase("héllo"), "HÉLLO");
                assert.equal(ts.templateLiterals.applyLowercase("HÉLLO"), "héllo");
                assert.equal(ts.templateLiterals.applyCapitalize("éllo"), "Éllo");
            });

            it("should handle emojis", () => {
                // Emojis don't have case, so they should pass through
                assert.equal(ts.templateLiterals.applyUppercase("hello 👋"), "HELLO 👋");
                assert.equal(ts.templateLiterals.applyLowercase("HELLO 👋"), "hello 👋");
            });
        });

        describe("Special characters", () => {
            it("should handle numbers and symbols", () => {
                assert.equal(ts.templateLiterals.applyUppercase("abc123!@#"), "ABC123!@#");
                assert.equal(ts.templateLiterals.applyLowercase("ABC123!@#"), "abc123!@#");
            });

            it("should handle whitespace", () => {
                assert.equal(ts.templateLiterals.applyUppercase("hello world"), "HELLO WORLD");
                assert.equal(ts.templateLiterals.applyCapitalize("  hello"), "  hello"); // leading space, so 'h' is not first char
            });
        });

        describe("Empty and single character strings", () => {
            it("should handle empty strings", () => {
                assert.equal(ts.templateLiterals.applyUppercase(""), "");
                assert.equal(ts.templateLiterals.applyLowercase(""), "");
                assert.equal(ts.templateLiterals.applyCapitalize(""), "");
                assert.equal(ts.templateLiterals.applyUncapitalize(""), "");
            });

            it("should handle single character strings", () => {
                assert.equal(ts.templateLiterals.applyUppercase("a"), "A");
                assert.equal(ts.templateLiterals.applyLowercase("A"), "a");
                assert.equal(ts.templateLiterals.applyCapitalize("a"), "A");
                assert.equal(ts.templateLiterals.applyUncapitalize("A"), "a");
            });
        });
    });

    describe("Idempotency", () => {
        it("uppercase should be idempotent", () => {
            const str = "Hello World";
            const once = ts.templateLiterals.applyUppercase(str);
            const twice = ts.templateLiterals.applyUppercase(once);
            assert.equal(once, twice);
        });

        it("lowercase should be idempotent", () => {
            const str = "Hello World";
            const once = ts.templateLiterals.applyLowercase(str);
            const twice = ts.templateLiterals.applyLowercase(once);
            assert.equal(once, twice);
        });

        it("capitalize on already capitalized should be idempotent", () => {
            const str = "Hello";
            const once = ts.templateLiterals.applyCapitalize(str);
            const twice = ts.templateLiterals.applyCapitalize(once);
            assert.equal(once, twice);
        });

        it("uncapitalize on already uncapitalized should be idempotent", () => {
            const str = "hello";
            const once = ts.templateLiterals.applyUncapitalize(str);
            const twice = ts.templateLiterals.applyUncapitalize(once);
            assert.equal(once, twice);
        });
    });

    describe("Composition", () => {
        it("uppercase followed by lowercase should give lowercase", () => {
            const str = "Hello World";
            const result = ts.templateLiterals.applyLowercase(
                ts.templateLiterals.applyUppercase(str)
            );
            assert.equal(result, "hello world");
        });

        it("capitalize followed by uncapitalize should restore first char case", () => {
            const str = "hello World";
            const capitalized = ts.templateLiterals.applyCapitalize(str);
            const uncapitalized = ts.templateLiterals.applyUncapitalize(capitalized);
            assert.equal(uncapitalized, "hello World");
        });
    });
});
