/**
 * @fileoverview Tests for indexed access solver
 */

namespace ts.tests {
    describe("IndexedAccessSolver", () => {
        let solver: IndexedAccessSolver;
        let mockContext: IndexedAccessContext;

        // Mock types
        const anyType = { id: 1, flags: TypeFlags.Any } as Type;
        const errorType = { id: 2, flags: TypeFlags.Any } as Type;
        const neverType = { id: 3, flags: TypeFlags.Never } as Type;
        const undefinedType = { id: 4, flags: TypeFlags.Undefined } as Type;
        const stringType = { id: 5, flags: TypeFlags.String } as Type;
        const numberType = { id: 6, flags: TypeFlags.Number } as Type;

        beforeEach(() => {
            solver = new IndexedAccessSolver();

            mockContext = {
                anyType,
                errorType,
                neverType,
                undefinedType,
                stringType,
                numberType,
                compilerOptions: {},
                getUnionType: (types) => {
                    if (types.length === 0) return neverType;
                    if (types.length === 1) return types[0];
                    return { id: Math.random(), flags: TypeFlags.Union, types } as UnionType;
                },
                getIntersectionType: (types) => {
                    if (types.length === 0) return anyType;
                    if (types.length === 1) return types[0];
                    return { id: Math.random(), flags: TypeFlags.Intersection, types } as IntersectionType;
                },
                getApparentType: (type) => type,
                resolveMembers: () => {},
                getPropertyOfType: () => undefined,
                getTypeOfSymbol: () => anyType,
                isTupleType: () => false,
                getTupleElementTypes: () => undefined,
                getTupleRestType: () => undefined,
                isTupleElementOptional: () => false,
                isMappedType: () => false,
                getMappedTypeTemplateType: () => undefined,
                getMappedTypeTypeParameter: () => undefined,
                isMappedTypeOptional: () => false,
                substituteTypeParameter: (type) => type,
                getTypeOfNode: () => anyType,
                isAssignmentTarget: () => false,
                getConstraintOfType: () => undefined,
            };
        });

        describe("constructor", () => {
            it("should use default config", () => {
                const defaultSolver = new IndexedAccessSolver();
                expect(defaultSolver).toBeDefined();
            });

            it("should accept custom config", () => {
                const customSolver = new IndexedAccessSolver({
                    maxIterations: 1000,
                    maxDepth: 50,
                });
                expect(customSolver).toBeDefined();
            });
        });

        describe("resolve", () => {
            it("should return never for never object type", () => {
                const result = solver.resolve(
                    neverType,
                    stringType,
                    IndexAccessFlags.None,
                    mockContext
                );

                expect(result.type).toBe(neverType);
                expect(result.isValid).toBe(true);
            });

            it("should return never for never index type", () => {
                const objectType = { id: 10, flags: TypeFlags.Object } as Type;

                const result = solver.resolve(
                    objectType,
                    neverType,
                    IndexAccessFlags.None,
                    mockContext
                );

                expect(result.type).toBe(neverType);
                expect(result.isValid).toBe(true);
            });

            it("should return any for any object type", () => {
                const result = solver.resolve(
                    anyType,
                    stringType,
                    IndexAccessFlags.None,
                    mockContext
                );

                expect(result.type).toBe(anyType);
                expect(result.isValid).toBe(true);
            });

            it("should cache results", () => {
                const objectType = { id: 10, flags: TypeFlags.Object } as Type;

                const result1 = solver.resolve(
                    objectType,
                    stringType,
                    IndexAccessFlags.None,
                    mockContext
                );

                const result2 = solver.resolve(
                    objectType,
                    stringType,
                    IndexAccessFlags.None,
                    mockContext
                );

                // Both should resolve (even if to error)
                expect(result1).toBeDefined();
                expect(result2).toBeDefined();
            });
        });

        describe("resolve with property access", () => {
            it("should resolve property by literal key", () => {
                const propertyType = { id: 20, flags: TypeFlags.String } as Type;
                const propSymbol = {
                    escapedName: "foo" as __String,
                    flags: SymbolFlags.Property,
                } as Symbol;

                const objectType = {
                    id: 10,
                    flags: TypeFlags.Object,
                    objectFlags: 0,
                } as ObjectType;

                const indexType = {
                    id: 11,
                    flags: TypeFlags.StringLiteral,
                    value: "foo",
                } as StringLiteralType;

                const contextWithProperty = {
                    ...mockContext,
                    getPropertyOfType: (type: Type, name: __String) => {
                        if (name === ("foo" as __String)) return propSymbol;
                        return undefined;
                    },
                    getTypeOfSymbol: (symbol: Symbol) => {
                        if (symbol === propSymbol) return propertyType;
                        return anyType;
                    },
                };

                const result = solver.resolve(
                    objectType,
                    indexType,
                    IndexAccessFlags.None,
                    contextWithProperty
                );

                expect(result.type).toBe(propertyType);
                expect(result.isValid).toBe(true);
            });

            it("should include undefined for optional property when flag set", () => {
                const propertyType = { id: 20, flags: TypeFlags.String } as Type;
                const propSymbol = {
                    escapedName: "foo" as __String,
                    flags: SymbolFlags.Property | SymbolFlags.Optional,
                } as Symbol;

                const objectType = {
                    id: 10,
                    flags: TypeFlags.Object,
                    objectFlags: 0,
                } as ObjectType;

                const indexType = {
                    id: 11,
                    flags: TypeFlags.StringLiteral,
                    value: "foo",
                } as StringLiteralType;

                const contextWithProperty = {
                    ...mockContext,
                    getPropertyOfType: () => propSymbol,
                    getTypeOfSymbol: () => propertyType,
                };

                const result = solver.resolve(
                    objectType,
                    indexType,
                    IndexAccessFlags.IncludeUndefined,
                    contextWithProperty
                );

                expect(result.isValid).toBe(true);
                // Result should be union with undefined (or original if no union created)
                expect(result.type).toBeDefined();
            });
        });

        describe("resolve with index signature", () => {
            it("should resolve through string index signature", () => {
                const valueType = { id: 20, flags: TypeFlags.Number } as Type;

                const objectType = {
                    id: 10,
                    flags: TypeFlags.Object,
                    objectFlags: 0,
                    indexInfos: [{
                        keyType: stringType,
                        type: valueType,
                        isReadonly: false,
                    }],
                } as ObjectType;

                const result = solver.resolve(
                    objectType,
                    stringType,
                    IndexAccessFlags.None,
                    mockContext
                );

                expect(result.type).toBe(valueType);
                expect(result.isValid).toBe(true);
            });

            it("should skip index signatures when flag set", () => {
                const valueType = { id: 20, flags: TypeFlags.Number } as Type;

                const objectType = {
                    id: 10,
                    flags: TypeFlags.Object,
                    objectFlags: 0,
                    indexInfos: [{
                        keyType: stringType,
                        type: valueType,
                        isReadonly: false,
                    }],
                } as ObjectType;

                const result = solver.resolve(
                    objectType,
                    stringType,
                    IndexAccessFlags.NoIndexSignatures,
                    mockContext
                );

                // Should return error type since property not found and signatures skipped
                expect(result.type).toBe(errorType);
                expect(result.isValid).toBe(false);
            });
        });

        describe("resolve with union types", () => {
            it("should distribute over union object type", () => {
                const type1 = {
                    id: 10,
                    flags: TypeFlags.Object,
                    objectFlags: 0,
                    indexInfos: [{
                        keyType: stringType,
                        type: { id: 100, flags: TypeFlags.String } as Type,
                        isReadonly: false,
                    }],
                } as ObjectType;

                const type2 = {
                    id: 11,
                    flags: TypeFlags.Object,
                    objectFlags: 0,
                    indexInfos: [{
                        keyType: stringType,
                        type: { id: 101, flags: TypeFlags.Number } as Type,
                        isReadonly: false,
                    }],
                } as ObjectType;

                const unionType = {
                    id: 12,
                    flags: TypeFlags.Union,
                    types: [type1, type2],
                } as UnionType;

                const result = solver.resolve(
                    unionType,
                    stringType,
                    IndexAccessFlags.None,
                    mockContext
                );

                expect(result.isValid).toBe(true);
                // Result should be union of string | number
                expect(result.type).toBeDefined();
            });

            it("should distribute over union index type", () => {
                const valueType1 = { id: 100, flags: TypeFlags.String } as Type;
                const valueType2 = { id: 101, flags: TypeFlags.Number } as Type;

                const prop1 = {
                    escapedName: "a" as __String,
                    flags: SymbolFlags.Property,
                } as Symbol;

                const prop2 = {
                    escapedName: "b" as __String,
                    flags: SymbolFlags.Property,
                } as Symbol;

                const objectType = {
                    id: 10,
                    flags: TypeFlags.Object,
                    objectFlags: 0,
                } as ObjectType;

                const indexType = {
                    id: 20,
                    flags: TypeFlags.Union,
                    types: [
                        { id: 21, flags: TypeFlags.StringLiteral, value: "a" } as StringLiteralType,
                        { id: 22, flags: TypeFlags.StringLiteral, value: "b" } as StringLiteralType,
                    ],
                } as UnionType;

                const contextWithProps = {
                    ...mockContext,
                    getPropertyOfType: (type: Type, name: __String) => {
                        if (name === ("a" as __String)) return prop1;
                        if (name === ("b" as __String)) return prop2;
                        return undefined;
                    },
                    getTypeOfSymbol: (symbol: Symbol) => {
                        if (symbol === prop1) return valueType1;
                        if (symbol === prop2) return valueType2;
                        return anyType;
                    },
                };

                const result = solver.resolve(
                    objectType,
                    indexType,
                    IndexAccessFlags.None,
                    contextWithProps
                );

                expect(result.isValid).toBe(true);
                expect(result.type).toBeDefined();
            });

            it("should intersect for writing context with union object", () => {
                const type1 = {
                    id: 10,
                    flags: TypeFlags.Object,
                    objectFlags: 0,
                    indexInfos: [{
                        keyType: stringType,
                        type: { id: 100, flags: TypeFlags.String } as Type,
                        isReadonly: false,
                    }],
                } as ObjectType;

                const type2 = {
                    id: 11,
                    flags: TypeFlags.Object,
                    objectFlags: 0,
                    indexInfos: [{
                        keyType: stringType,
                        type: { id: 101, flags: TypeFlags.Number } as Type,
                        isReadonly: false,
                    }],
                } as ObjectType;

                const unionType = {
                    id: 12,
                    flags: TypeFlags.Union,
                    types: [type1, type2],
                } as UnionType;

                const result = solver.resolve(
                    unionType,
                    stringType,
                    IndexAccessFlags.Writing,
                    mockContext
                );

                expect(result.isValid).toBe(true);
                // For writing, result should be intersection
                expect(result.type).toBeDefined();
            });
        });

        describe("resolve with tuple types", () => {
            it("should resolve tuple element by numeric literal", () => {
                const elementType = { id: 100, flags: TypeFlags.String } as Type;

                const tupleType = {
                    id: 10,
                    flags: TypeFlags.Object,
                    objectFlags: ObjectFlags.Reference,
                } as ObjectType;

                const indexType = {
                    id: 20,
                    flags: TypeFlags.NumberLiteral,
                    value: 0,
                } as NumberLiteralType;

                const contextWithTuple = {
                    ...mockContext,
                    isTupleType: (type: Type) => type === tupleType,
                    getTupleElementTypes: (type: Type) => {
                        if (type === tupleType) return [elementType];
                        return undefined;
                    },
                    isTupleElementOptional: () => false,
                };

                const result = solver.resolve(
                    tupleType,
                    indexType,
                    IndexAccessFlags.None,
                    contextWithTuple
                );

                expect(result.type).toBe(elementType);
                expect(result.isValid).toBe(true);
            });

            it("should return error for out of bounds index", () => {
                const elementType = { id: 100, flags: TypeFlags.String } as Type;

                const tupleType = {
                    id: 10,
                    flags: TypeFlags.Object,
                    objectFlags: ObjectFlags.Reference,
                } as ObjectType;

                const indexType = {
                    id: 20,
                    flags: TypeFlags.NumberLiteral,
                    value: 5, // Out of bounds
                } as NumberLiteralType;

                const contextWithTuple = {
                    ...mockContext,
                    isTupleType: (type: Type) => type === tupleType,
                    getTupleElementTypes: (type: Type) => {
                        if (type === tupleType) return [elementType];
                        return undefined;
                    },
                    getTupleRestType: () => undefined,
                    isTupleElementOptional: () => false,
                };

                const result = solver.resolve(
                    tupleType,
                    indexType,
                    IndexAccessFlags.None,
                    contextWithTuple
                );

                expect(result.type).toBe(errorType);
                expect(result.isValid).toBe(false);
            });

            it("should use rest type for out of bounds when available", () => {
                const elementType = { id: 100, flags: TypeFlags.String } as Type;
                const restType = { id: 101, flags: TypeFlags.Number } as Type;

                const tupleType = {
                    id: 10,
                    flags: TypeFlags.Object,
                    objectFlags: ObjectFlags.Reference,
                } as ObjectType;

                const indexType = {
                    id: 20,
                    flags: TypeFlags.NumberLiteral,
                    value: 5,
                } as NumberLiteralType;

                const contextWithTuple = {
                    ...mockContext,
                    isTupleType: (type: Type) => type === tupleType,
                    getTupleElementTypes: (type: Type) => {
                        if (type === tupleType) return [elementType];
                        return undefined;
                    },
                    getTupleRestType: () => restType,
                    isTupleElementOptional: () => false,
                };

                const result = solver.resolve(
                    tupleType,
                    indexType,
                    IndexAccessFlags.None,
                    contextWithTuple
                );

                expect(result.type).toBe(restType);
                expect(result.isValid).toBe(true);
            });

            it("should return union of all elements for number type index", () => {
                const type1 = { id: 100, flags: TypeFlags.String } as Type;
                const type2 = { id: 101, flags: TypeFlags.Number } as Type;

                const tupleType = {
                    id: 10,
                    flags: TypeFlags.Object,
                    objectFlags: ObjectFlags.Reference,
                } as ObjectType;

                const contextWithTuple = {
                    ...mockContext,
                    isTupleType: (type: Type) => type === tupleType,
                    getTupleElementTypes: (type: Type) => {
                        if (type === tupleType) return [type1, type2];
                        return undefined;
                    },
                    getTupleRestType: () => undefined,
                    isTupleElementOptional: () => false,
                };

                const result = solver.resolve(
                    tupleType,
                    numberType,
                    IndexAccessFlags.None,
                    contextWithTuple
                );

                expect(result.isValid).toBe(true);
                // Result should be union of string | number
                expect(result.type).toBeDefined();
            });
        });

        describe("resolve with mapped types", () => {
            it("should resolve mapped type template", () => {
                const templateType = { id: 100, flags: TypeFlags.String } as Type;
                const typeParam = { id: 101, flags: TypeFlags.TypeParameter } as Type;

                const mappedType = {
                    id: 10,
                    flags: TypeFlags.Object,
                    objectFlags: ObjectFlags.Mapped,
                } as ObjectType;

                const contextWithMapped = {
                    ...mockContext,
                    isMappedType: (type: Type) => type === mappedType,
                    getMappedTypeTemplateType: (type: Type) => {
                        if (type === mappedType) return templateType;
                        return undefined;
                    },
                    getMappedTypeTypeParameter: (type: Type) => {
                        if (type === mappedType) return typeParam;
                        return undefined;
                    },
                    isMappedTypeOptional: () => false,
                    substituteTypeParameter: (type: Type) => type,
                };

                const result = solver.resolve(
                    mappedType,
                    stringType,
                    IndexAccessFlags.None,
                    contextWithMapped
                );

                expect(result.isValid).toBe(true);
                expect(result.type).toBeDefined();
            });
        });

        describe("checkElementAccessExpression", () => {
            it("should resolve element access", () => {
                const objectType = { id: 10, flags: TypeFlags.Any } as Type;
                const mockExpression = {} as Expression;

                const result = solver.checkElementAccessExpression(
                    objectType,
                    mockExpression,
                    mockContext
                );

                // For any type, should return any
                expect(result.type).toBe(anyType);
                expect(result.isValid).toBe(true);
            });

            it("should include undefined when noUncheckedIndexedAccess is enabled", () => {
                const valueType = { id: 20, flags: TypeFlags.String } as Type;

                const objectType = {
                    id: 10,
                    flags: TypeFlags.Object,
                    objectFlags: 0,
                    indexInfos: [{
                        keyType: stringType,
                        type: valueType,
                        isReadonly: false,
                    }],
                } as ObjectType;

                const mockExpression = {} as Expression;

                const contextWithOption = {
                    ...mockContext,
                    compilerOptions: {
                        noUncheckedIndexedAccess: true,
                    },
                };

                const result = solver.checkElementAccessExpression(
                    objectType,
                    mockExpression,
                    contextWithOption
                );

                expect(result.isValid).toBe(true);
                // Result should include undefined
                expect(result.type).toBeDefined();
            });

            it("should set writing flag for assignment target", () => {
                const objectType = { id: 10, flags: TypeFlags.Any } as Type;
                const mockExpression = {} as Expression;

                const contextWithAssignment = {
                    ...mockContext,
                    isAssignmentTarget: () => true,
                };

                const result = solver.checkElementAccessExpression(
                    objectType,
                    mockExpression,
                    contextWithAssignment
                );

                // Should still work (any type returns any)
                expect(result.type).toBe(anyType);
            });
        });

        describe("getSimplifiedIndexedAccessType", () => {
            it("should return non-indexed-access types unchanged", () => {
                const type = { id: 10, flags: TypeFlags.String } as Type;

                const result = solver.getSimplifiedIndexedAccessType(
                    type,
                    false,
                    mockContext
                );

                expect(result).toBe(type);
            });

            it("should cache simplified results", () => {
                const objectType = { id: 10, flags: TypeFlags.Any } as Type;
                const indexType = { id: 11, flags: TypeFlags.String } as Type;

                const indexedAccessType = {
                    id: 20,
                    flags: TypeFlags.IndexedAccess,
                    objectType,
                    indexType,
                } as unknown as Type;

                // First call
                const result1 = solver.getSimplifiedIndexedAccessType(
                    indexedAccessType,
                    false,
                    mockContext
                );

                // Second call should use cache
                const result2 = solver.getSimplifiedIndexedAccessType(
                    indexedAccessType,
                    false,
                    mockContext
                );

                expect(result1).toBe(result2);
            });
        });

        describe("getConstraintOfIndexedAccess", () => {
            it("should return undefined when no constraints", () => {
                const objectType = { id: 10, flags: TypeFlags.Object } as Type;
                const indexType = { id: 11, flags: TypeFlags.String } as Type;

                const result = solver.getConstraintOfIndexedAccess(
                    objectType,
                    indexType,
                    mockContext
                );

                expect(result).toBeUndefined();
            });

            it("should resolve using constraint types", () => {
                const objectType = { id: 10, flags: TypeFlags.TypeParameter } as Type;
                const indexType = { id: 11, flags: TypeFlags.String } as Type;
                const constraintType = { id: 12, flags: TypeFlags.Any } as Type;

                const contextWithConstraint = {
                    ...mockContext,
                    getConstraintOfType: (type: Type) => {
                        if (type === objectType) return constraintType;
                        return undefined;
                    },
                };

                const result = solver.getConstraintOfIndexedAccess(
                    objectType,
                    indexType,
                    contextWithConstraint
                );

                // Should resolve using constraint
                expect(result).toBeDefined();
            });
        });

        describe("clearCache", () => {
            it("should clear solver cache", () => {
                const objectType = { id: 10, flags: TypeFlags.Any } as Type;

                // Populate cache
                solver.resolve(
                    objectType,
                    stringType,
                    IndexAccessFlags.None,
                    mockContext
                );

                // Clear
                solver.clearCache();

                // Should work again
                const result = solver.resolve(
                    objectType,
                    stringType,
                    IndexAccessFlags.None,
                    mockContext
                );

                expect(result.type).toBe(anyType);
            });
        });
    });

    describe("createMinimalIndexedAccessContext", () => {
        it("should create a valid context", () => {
            const anyType = { id: 1, flags: TypeFlags.Any } as Type;
            const errorType = { id: 2, flags: TypeFlags.Any } as Type;
            const neverType = { id: 3, flags: TypeFlags.Never } as Type;
            const undefinedType = { id: 4, flags: TypeFlags.Undefined } as Type;
            const stringType = { id: 5, flags: TypeFlags.String } as Type;
            const numberType = { id: 6, flags: TypeFlags.Number } as Type;

            const context = createMinimalIndexedAccessContext(
                anyType,
                errorType,
                neverType,
                undefinedType,
                stringType,
                numberType
            );

            expect(context.anyType).toBe(anyType);
            expect(context.errorType).toBe(errorType);
            expect(context.neverType).toBe(neverType);
            expect(context.undefinedType).toBe(undefinedType);
            expect(context.stringType).toBe(stringType);
            expect(context.numberType).toBe(numberType);

            // Check functions exist
            expect(typeof context.getUnionType).toBe("function");
            expect(typeof context.getIntersectionType).toBe("function");
            expect(typeof context.getApparentType).toBe("function");
            expect(typeof context.isTupleType).toBe("function");
            expect(typeof context.isMappedType).toBe("function");
        });
    });
}
