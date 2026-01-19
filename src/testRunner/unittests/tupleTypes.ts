describe("unittests:: Tuple Types", () => {
    describe("TupleElementFlags", () => {
        it("should define element flag values", () => {
            assert.equal(ts.tuples.TupleElementFlags.None, 0);
            assert.equal(ts.tuples.TupleElementFlags.Required, 1);
            assert.equal(ts.tuples.TupleElementFlags.Optional, 2);
            assert.equal(ts.tuples.TupleElementFlags.Rest, 4);
            assert.equal(ts.tuples.TupleElementFlags.Variadic, 8);
        });
    });

    describe("Element Creation", () => {
        const mockType = { flags: ts.TypeFlags.String } as ts.Type;

        describe("createRequiredElement", () => {
            it("should create a required element", () => {
                const element = ts.tuples.createRequiredElement(mockType);
                assert.equal(element.flags, ts.tuples.TupleElementFlags.Required);
                assert.equal(element.type, mockType);
                assert.isUndefined(element.label);
            });

            it("should create a required element with label", () => {
                const element = ts.tuples.createRequiredElement(mockType, "name");
                assert.equal(element.label, "name");
            });
        });

        describe("createOptionalElement", () => {
            it("should create an optional element", () => {
                const element = ts.tuples.createOptionalElement(mockType);
                assert.equal(element.flags, ts.tuples.TupleElementFlags.Optional);
            });

            it("should create an optional element with label", () => {
                const element = ts.tuples.createOptionalElement(mockType, "age");
                assert.equal(element.label, "age");
            });
        });

        describe("createRestElement", () => {
            it("should create a rest element", () => {
                const element = ts.tuples.createRestElement(mockType);
                assert.equal(element.flags, ts.tuples.TupleElementFlags.Rest);
            });
        });

        describe("createVariadicElement", () => {
            it("should create a variadic element", () => {
                const element = ts.tuples.createVariadicElement(mockType);
                assert.equal(element.flags, ts.tuples.TupleElementFlags.Variadic);
            });
        });
    });

    describe("Element Type Checking", () => {
        const mockType = { flags: ts.TypeFlags.Number } as ts.Type;

        it("isRequiredElement should identify required elements", () => {
            const required = ts.tuples.createRequiredElement(mockType);
            const optional = ts.tuples.createOptionalElement(mockType);

            assert.isTrue(ts.tuples.isRequiredElement(required));
            assert.isFalse(ts.tuples.isRequiredElement(optional));
        });

        it("isOptionalElement should identify optional elements", () => {
            const optional = ts.tuples.createOptionalElement(mockType);
            const required = ts.tuples.createRequiredElement(mockType);

            assert.isTrue(ts.tuples.isOptionalElement(optional));
            assert.isFalse(ts.tuples.isOptionalElement(required));
        });

        it("isRestElement should identify rest elements", () => {
            const rest = ts.tuples.createRestElement(mockType);
            const required = ts.tuples.createRequiredElement(mockType);

            assert.isTrue(ts.tuples.isRestElement(rest));
            assert.isFalse(ts.tuples.isRestElement(required));
        });

        it("isVariadicElement should identify variadic elements", () => {
            const variadic = ts.tuples.createVariadicElement(mockType);
            const rest = ts.tuples.createRestElement(mockType);

            assert.isTrue(ts.tuples.isVariadicElement(variadic));
            assert.isFalse(ts.tuples.isVariadicElement(rest));
        });
    });

    describe("Element Labels", () => {
        const mockType = { flags: ts.TypeFlags.String } as ts.Type;

        it("hasLabel should check for label presence", () => {
            const labeled = ts.tuples.createRequiredElement(mockType, "name");
            const unlabeled = ts.tuples.createRequiredElement(mockType);

            assert.isTrue(ts.tuples.hasLabel(labeled));
            assert.isFalse(ts.tuples.hasLabel(unlabeled));
        });

        it("withLabel should add label to element", () => {
            const element = ts.tuples.createRequiredElement(mockType);
            const labeled = ts.tuples.withLabel(element, "foo");

            assert.equal(labeled.label, "foo");
            assert.isUndefined(element.label); // Original unchanged
        });

        it("removeLabel should remove label from element", () => {
            const labeled = ts.tuples.createRequiredElement(mockType, "bar");
            const unlabeled = ts.tuples.removeLabel(labeled);

            assert.isUndefined(unlabeled.label);
            assert.equal(labeled.label, "bar"); // Original unchanged
        });
    });

    describe("Element Modification", () => {
        const mockType = { flags: ts.TypeFlags.Boolean } as ts.Type;

        it("cloneElement should create a copy", () => {
            const original = ts.tuples.createRequiredElement(mockType, "test");
            const clone = ts.tuples.cloneElement(original);

            assert.deepEqual(clone, original);
            assert.notStrictEqual(clone, original);
        });

        it("makeOptional should convert to optional", () => {
            const required = ts.tuples.createRequiredElement(mockType);
            const optional = ts.tuples.makeOptional(required);

            assert.isTrue(ts.tuples.isOptionalElement(optional));
            assert.isFalse(ts.tuples.isRequiredElement(optional));
        });

        it("makeRequired should convert to required", () => {
            const optional = ts.tuples.createOptionalElement(mockType);
            const required = ts.tuples.makeRequired(optional);

            assert.isTrue(ts.tuples.isRequiredElement(required));
            assert.isFalse(ts.tuples.isOptionalElement(required));
        });
    });

    describe("Element Order Validation", () => {
        const stringType = { flags: ts.TypeFlags.String } as ts.Type;
        const numberType = { flags: ts.TypeFlags.Number } as ts.Type;

        it("should allow valid element order", () => {
            const elements = [
                ts.tuples.createRequiredElement(stringType),
                ts.tuples.createRequiredElement(numberType),
            ];

            const result = ts.tuples.validateElementOrder(elements);
            assert.isTrue(result.valid);
        });

        it("should allow required followed by optional", () => {
            const elements = [
                ts.tuples.createRequiredElement(stringType),
                ts.tuples.createOptionalElement(numberType),
            ];

            const result = ts.tuples.validateElementOrder(elements);
            assert.isTrue(result.valid);
        });

        it("should reject required after optional", () => {
            const elements = [
                ts.tuples.createOptionalElement(stringType),
                ts.tuples.createRequiredElement(numberType),
            ];

            const result = ts.tuples.validateElementOrder(elements);
            assert.isFalse(result.valid);
            assert.include(result.error!, "required");
        });

        it("should reject multiple rest elements", () => {
            const elements = [
                ts.tuples.createRestElement(stringType),
                ts.tuples.createRestElement(numberType),
            ];

            const result = ts.tuples.validateElementOrder(elements);
            assert.isFalse(result.valid);
            assert.include(result.error!, "multiple rest");
        });

        it("should allow required-optional-rest order", () => {
            const elements = [
                ts.tuples.createRequiredElement(stringType),
                ts.tuples.createOptionalElement(numberType),
                ts.tuples.createRestElement(stringType),
            ];

            const result = ts.tuples.validateElementOrder(elements);
            assert.isTrue(result.valid);
        });
    });

    describe("Element Length Contribution", () => {
        const mockType = { flags: ts.TypeFlags.String } as ts.Type;

        it("required element contributes exactly 1", () => {
            const required = ts.tuples.createRequiredElement(mockType);
            const contrib = ts.tuples.getElementLengthContribution(required);

            assert.equal(contrib.min, 1);
            assert.equal(contrib.max, 1);
        });

        it("optional element contributes 0-1", () => {
            const optional = ts.tuples.createOptionalElement(mockType);
            const contrib = ts.tuples.getElementLengthContribution(optional);

            assert.equal(contrib.min, 0);
            assert.equal(contrib.max, 1);
        });

        it("rest element contributes 0-infinity", () => {
            const rest = ts.tuples.createRestElement(mockType);
            const contrib = ts.tuples.getElementLengthContribution(rest);

            assert.equal(contrib.min, 0);
            assert.isUndefined(contrib.max);
        });
    });

    describe("Label Analysis", () => {
        const mockType = { flags: ts.TypeFlags.String } as ts.Type;

        it("allLabeled should check if all elements have labels", () => {
            const elements = [
                ts.tuples.createRequiredElement(mockType, "a"),
                ts.tuples.createRequiredElement(mockType, "b"),
            ];

            assert.isTrue(ts.tuples.allLabeled(elements));
        });

        it("noneLabeled should check if no elements have labels", () => {
            const elements = [
                ts.tuples.createRequiredElement(mockType),
                ts.tuples.createRequiredElement(mockType),
            ];

            assert.isTrue(ts.tuples.noneLabeled(elements));
        });

        it("hassMixedLabels should detect mixed labeling", () => {
            const elements = [
                ts.tuples.createRequiredElement(mockType, "a"),
                ts.tuples.createRequiredElement(mockType),
            ];

            assert.isTrue(ts.tuples.hassMixedLabels(elements));
        });

        it("getLabels should extract all labels", () => {
            const elements = [
                ts.tuples.createRequiredElement(mockType, "first"),
                ts.tuples.createRequiredElement(mockType),
                ts.tuples.createRequiredElement(mockType, "third"),
            ];

            const labels = ts.tuples.getLabels(elements);
            assert.deepEqual(labels, ["first", undefined, "third"]);
        });
    });

    describe("Tuple Type Info", () => {
        it("createEmptyTuple should create empty tuple", () => {
            const tuple = ts.tuples.createEmptyTuple();

            assert.isEmpty(tuple.elements);
            assert.equal(tuple.minLength, 0);
            assert.equal(tuple.fixedLength, 0);
            assert.isFalse(tuple.hasOptional);
            assert.isFalse(tuple.hasRest);
            assert.isFalse(tuple.hasVariadic);
        });

        it("isEmptyTuple should identify empty tuples", () => {
            const empty = ts.tuples.createEmptyTuple();
            assert.isTrue(ts.tuples.isEmptyTuple(empty));

            const nonEmpty: ts.tuples.TupleTypeInfo = {
                elements: [ts.tuples.createRequiredElement({ flags: ts.TypeFlags.String } as ts.Type)],
                readonly: false,
                minLength: 1,
                fixedLength: 1,
                hasOptional: false,
                hasRest: false,
                hasVariadic: false,
            };
            assert.isFalse(ts.tuples.isEmptyTuple(nonEmpty));
        });

        it("toReadonlyTuple should create readonly version", () => {
            const tuple = ts.tuples.createEmptyTuple(false);
            const readonly = ts.tuples.toReadonlyTuple(tuple);

            assert.isFalse(tuple.readonly);
            assert.isTrue(readonly.readonly);
        });

        it("toMutableTuple should create mutable version", () => {
            const tuple = ts.tuples.createEmptyTuple(true);
            const mutable = ts.tuples.toMutableTuple(tuple);

            assert.isTrue(tuple.readonly);
            assert.isFalse(mutable.readonly);
        });
    });

    describe("Tuple Length Analysis", () => {
        it("should analyze fixed-length tuple", () => {
            const tuple: ts.tuples.TupleTypeInfo = {
                elements: [
                    ts.tuples.createRequiredElement({ flags: ts.TypeFlags.String } as ts.Type),
                    ts.tuples.createRequiredElement({ flags: ts.TypeFlags.Number } as ts.Type),
                ],
                readonly: false,
                minLength: 2,
                fixedLength: 2,
                hasOptional: false,
                hasRest: false,
                hasVariadic: false,
            };

            const analysis = ts.tuples.analyzeTupleLength(tuple);
            assert.equal(analysis.minLength, 2);
            assert.equal(analysis.maxLength, 2);
            assert.isTrue(analysis.isFixed);
            assert.equal(analysis.exactLength, 2);
        });

        it("should analyze tuple with optional elements", () => {
            const tuple: ts.tuples.TupleTypeInfo = {
                elements: [
                    ts.tuples.createRequiredElement({ flags: ts.TypeFlags.String } as ts.Type),
                    ts.tuples.createOptionalElement({ flags: ts.TypeFlags.Number } as ts.Type),
                ],
                readonly: false,
                minLength: 1,
                fixedLength: undefined,
                hasOptional: true,
                hasRest: false,
                hasVariadic: false,
            };

            const analysis = ts.tuples.analyzeTupleLength(tuple);
            assert.equal(analysis.minLength, 1);
            assert.equal(analysis.maxLength, 2);
            assert.isFalse(analysis.isFixed);
        });

        it("should analyze tuple with rest element", () => {
            const tuple: ts.tuples.TupleTypeInfo = {
                elements: [
                    ts.tuples.createRequiredElement({ flags: ts.TypeFlags.String } as ts.Type),
                    ts.tuples.createRestElement({ flags: ts.TypeFlags.Number } as ts.Type),
                ],
                readonly: false,
                minLength: 1,
                fixedLength: undefined,
                hasOptional: false,
                hasRest: true,
                hasVariadic: false,
            };

            const analysis = ts.tuples.analyzeTupleLength(tuple);
            assert.equal(analysis.minLength, 1);
            assert.isUndefined(analysis.maxLength);
            assert.isFalse(analysis.isFixed);
        });
    });

    describe("Tuple Index Validation", () => {
        const stringType = { flags: ts.TypeFlags.String } as ts.Type;
        const numberType = { flags: ts.TypeFlags.Number } as ts.Type;

        it("should validate indices for fixed-length tuple", () => {
            const tuple: ts.tuples.TupleTypeInfo = {
                elements: [
                    ts.tuples.createRequiredElement(stringType),
                    ts.tuples.createRequiredElement(numberType),
                ],
                readonly: false,
                minLength: 2,
                fixedLength: 2,
                hasOptional: false,
                hasRest: false,
                hasVariadic: false,
            };

            assert.isTrue(ts.tuples.isValidTupleIndex(tuple, 0));
            assert.isTrue(ts.tuples.isValidTupleIndex(tuple, 1));
            assert.isFalse(ts.tuples.isValidTupleIndex(tuple, 2));
            assert.isFalse(ts.tuples.isValidTupleIndex(tuple, -1));
        });

        it("should allow any non-negative index for variable-length tuple", () => {
            const tuple: ts.tuples.TupleTypeInfo = {
                elements: [
                    ts.tuples.createRequiredElement(stringType),
                    ts.tuples.createRestElement(numberType),
                ],
                readonly: false,
                minLength: 1,
                fixedLength: undefined,
                hasOptional: false,
                hasRest: true,
                hasVariadic: false,
            };

            assert.isTrue(ts.tuples.isValidTupleIndex(tuple, 0));
            assert.isTrue(ts.tuples.isValidTupleIndex(tuple, 100));
            assert.isFalse(ts.tuples.isValidTupleIndex(tuple, -1));
        });
    });

    describe("isTupleType", () => {
        it("should return false for non-object types", () => {
            const stringType = { flags: ts.TypeFlags.String } as ts.Type;
            assert.isFalse(ts.tuples.isTupleType(stringType));
        });

        it("should return false for regular object types", () => {
            const objectType = {
                flags: ts.TypeFlags.Object,
                objectFlags: ts.ObjectFlags.Anonymous,
            } as ts.ObjectType;
            assert.isFalse(ts.tuples.isTupleType(objectType));
        });

        it("should return true for tuple object types", () => {
            const tupleType = {
                flags: ts.TypeFlags.Object,
                objectFlags: ts.ObjectFlags.Tuple,
            } as ts.ObjectType;
            assert.isTrue(ts.tuples.isTupleType(tupleType));
        });
    });

    describe("Homogeneous Tuple Creation", () => {
        const stringType = { flags: ts.TypeFlags.String } as ts.Type;

        it("should create homogeneous tuple of specified length", () => {
            const tuple = ts.tuples.createHomogeneousTuple(stringType, 3, false);

            assert.equal(tuple.elements.length, 3);
            assert.equal(tuple.minLength, 3);
            assert.equal(tuple.fixedLength, 3);
            assert.isFalse(tuple.readonly);

            for (const element of tuple.elements) {
                assert.equal(element.type, stringType);
                assert.isTrue(ts.tuples.isRequiredElement(element));
            }
        });

        it("should create readonly homogeneous tuple", () => {
            const tuple = ts.tuples.createHomogeneousTuple(stringType, 2, true);

            assert.isTrue(tuple.readonly);
            assert.equal(tuple.elements.length, 2);
        });

        it("should create empty tuple for length 0", () => {
            const tuple = ts.tuples.createHomogeneousTuple(stringType, 0, false);

            assert.isEmpty(tuple.elements);
            assert.equal(tuple.minLength, 0);
            assert.equal(tuple.fixedLength, 0);
        });
    });

    describe("TupleHelpers", () => {
        const stringType = { flags: ts.TypeFlags.String } as ts.Type;
        const numberType = { flags: ts.TypeFlags.Number } as ts.Type;

        describe("first and last", () => {
            it("first should return first element type", () => {
                const tuple: ts.tuples.TupleTypeInfo = {
                    elements: [
                        ts.tuples.createRequiredElement(stringType),
                        ts.tuples.createRequiredElement(numberType),
                    ],
                    readonly: false,
                    minLength: 2,
                    fixedLength: 2,
                    hasOptional: false,
                    hasRest: false,
                    hasVariadic: false,
                };

                assert.equal(ts.tuples.TupleHelpers.first(tuple), stringType);
            });

            it("last should return last element type", () => {
                const tuple: ts.tuples.TupleTypeInfo = {
                    elements: [
                        ts.tuples.createRequiredElement(stringType),
                        ts.tuples.createRequiredElement(numberType),
                    ],
                    readonly: false,
                    minLength: 2,
                    fixedLength: 2,
                    hasOptional: false,
                    hasRest: false,
                    hasVariadic: false,
                };

                assert.equal(ts.tuples.TupleHelpers.last(tuple), numberType);
            });

            it("should return undefined for empty tuple", () => {
                const empty = ts.tuples.createEmptyTuple();

                assert.isUndefined(ts.tuples.TupleHelpers.first(empty));
                assert.isUndefined(ts.tuples.TupleHelpers.last(empty));
            });
        });

        describe("requiredTypes and optionalTypes", () => {
            it("should filter required element types", () => {
                const tuple: ts.tuples.TupleTypeInfo = {
                    elements: [
                        ts.tuples.createRequiredElement(stringType),
                        ts.tuples.createOptionalElement(numberType),
                    ],
                    readonly: false,
                    minLength: 1,
                    fixedLength: undefined,
                    hasOptional: true,
                    hasRest: false,
                    hasVariadic: false,
                };

                const required = ts.tuples.TupleHelpers.requiredTypes(tuple);
                assert.deepEqual(required, [stringType]);
            });

            it("should filter optional element types", () => {
                const tuple: ts.tuples.TupleTypeInfo = {
                    elements: [
                        ts.tuples.createRequiredElement(stringType),
                        ts.tuples.createOptionalElement(numberType),
                    ],
                    readonly: false,
                    minLength: 1,
                    fixedLength: undefined,
                    hasOptional: true,
                    hasRest: false,
                    hasVariadic: false,
                };

                const optional = ts.tuples.TupleHelpers.optionalTypes(tuple);
                assert.deepEqual(optional, [numberType]);
            });
        });

        describe("restType", () => {
            it("should return rest element type", () => {
                const boolType = { flags: ts.TypeFlags.Boolean } as ts.Type;
                const tuple: ts.tuples.TupleTypeInfo = {
                    elements: [
                        ts.tuples.createRequiredElement(stringType),
                        ts.tuples.createRestElement(boolType),
                    ],
                    readonly: false,
                    minLength: 1,
                    fixedLength: undefined,
                    hasOptional: false,
                    hasRest: true,
                    hasVariadic: false,
                };

                assert.equal(ts.tuples.TupleHelpers.restType(tuple), boolType);
            });

            it("should return undefined for tuple without rest", () => {
                const tuple: ts.tuples.TupleTypeInfo = {
                    elements: [
                        ts.tuples.createRequiredElement(stringType),
                    ],
                    readonly: false,
                    minLength: 1,
                    fixedLength: 1,
                    hasOptional: false,
                    hasRest: false,
                    hasVariadic: false,
                };

                assert.isUndefined(ts.tuples.TupleHelpers.restType(tuple));
            });
        });
    });

    describe("Tuple Validation", () => {
        const stringType = { flags: ts.TypeFlags.String } as ts.Type;

        it("should validate correctly structured tuple", () => {
            const tuple: ts.tuples.TupleTypeInfo = {
                elements: [
                    ts.tuples.createRequiredElement(stringType),
                ],
                readonly: false,
                minLength: 1,
                fixedLength: 1,
                hasOptional: false,
                hasRest: false,
                hasVariadic: false,
            };

            const result = ts.tuples.validateTuple(tuple);
            assert.isTrue(result.valid);
            assert.isEmpty(result.errors);
        });

        it("should detect incorrect minLength", () => {
            const tuple: ts.tuples.TupleTypeInfo = {
                elements: [
                    ts.tuples.createRequiredElement(stringType),
                    ts.tuples.createRequiredElement(stringType),
                ],
                readonly: false,
                minLength: 1, // Incorrect - should be 2
                fixedLength: 2,
                hasOptional: false,
                hasRest: false,
                hasVariadic: false,
            };

            const result = ts.tuples.validateTuple(tuple);
            assert.isFalse(result.valid);
            assert.isNotEmpty(result.errors);
        });

        it("should detect mixed labels", () => {
            const tuple: ts.tuples.TupleTypeInfo = {
                elements: [
                    ts.tuples.createRequiredElement(stringType, "labeled"),
                    ts.tuples.createRequiredElement(stringType), // Not labeled
                ],
                readonly: false,
                minLength: 2,
                fixedLength: 2,
                hasOptional: false,
                hasRest: false,
                hasVariadic: false,
            };

            const result = ts.tuples.validateTuple(tuple);
            assert.isFalse(result.valid);
            assert.isTrue(result.errors.some(e => e.includes("label")));
        });
    });

    describe("Tuple Slicing", () => {
        const stringType = { flags: ts.TypeFlags.String } as ts.Type;
        const numberType = { flags: ts.TypeFlags.Number } as ts.Type;
        const boolType = { flags: ts.TypeFlags.Boolean } as ts.Type;

        it("should slice from start", () => {
            const tuple: ts.tuples.TupleTypeInfo = {
                elements: [
                    ts.tuples.createRequiredElement(stringType),
                    ts.tuples.createRequiredElement(numberType),
                    ts.tuples.createRequiredElement(boolType),
                ],
                readonly: false,
                minLength: 3,
                fixedLength: 3,
                hasOptional: false,
                hasRest: false,
                hasVariadic: false,
            };

            const sliced = ts.tuples.sliceTuple(tuple, 1);
            assert.equal(sliced.elements.length, 2);
            assert.equal(sliced.elements[0].type, numberType);
            assert.equal(sliced.elements[1].type, boolType);
        });

        it("should slice with end", () => {
            const tuple: ts.tuples.TupleTypeInfo = {
                elements: [
                    ts.tuples.createRequiredElement(stringType),
                    ts.tuples.createRequiredElement(numberType),
                    ts.tuples.createRequiredElement(boolType),
                ],
                readonly: false,
                minLength: 3,
                fixedLength: 3,
                hasOptional: false,
                hasRest: false,
                hasVariadic: false,
            };

            const sliced = ts.tuples.sliceTuple(tuple, 0, 2);
            assert.equal(sliced.elements.length, 2);
            assert.equal(sliced.elements[0].type, stringType);
            assert.equal(sliced.elements[1].type, numberType);
        });

        it("should preserve readonly", () => {
            const tuple: ts.tuples.TupleTypeInfo = {
                elements: [
                    ts.tuples.createRequiredElement(stringType),
                    ts.tuples.createRequiredElement(numberType),
                ],
                readonly: true,
                minLength: 2,
                fixedLength: 2,
                hasOptional: false,
                hasRest: false,
                hasVariadic: false,
            };

            const sliced = ts.tuples.sliceTuple(tuple, 1);
            assert.isTrue(sliced.readonly);
        });
    });
});
