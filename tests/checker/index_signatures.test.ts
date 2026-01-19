/**
 * @fileoverview Tests for index signature checker
 */

namespace ts.tests {
    describe("IndexSignatureChecker", () => {
        let checker: IndexSignatureChecker;

        beforeEach(() => {
            checker = new IndexSignatureChecker();
        });

        describe("constructor", () => {
            it("should use default config", () => {
                const defaultChecker = new IndexSignatureChecker();
                expect(defaultChecker).toBeDefined();
            });

            it("should accept custom config", () => {
                const customChecker = new IndexSignatureChecker({
                    maxDepth: 50,
                    allowSymbolIndexSignatures: false,
                });
                expect(customChecker).toBeDefined();
            });
        });

        describe("validateIndexSignatureKeyType", () => {
            it("should accept string type", () => {
                const stringType = { flags: TypeFlags.String } as Type;
                const result = checker.validateIndexSignatureKeyType(stringType);
                expect(result.isValid).toBe(true);
                expect(result.diagnostics).toHaveLength(0);
            });

            it("should accept number type", () => {
                const numberType = { flags: TypeFlags.Number } as Type;
                const result = checker.validateIndexSignatureKeyType(numberType);
                expect(result.isValid).toBe(true);
            });

            it("should accept symbol type when enabled", () => {
                const symbolType = { flags: TypeFlags.ESSymbol } as Type;
                const result = checker.validateIndexSignatureKeyType(symbolType);
                expect(result.isValid).toBe(true);
            });

            it("should reject symbol type when disabled", () => {
                const disabledChecker = new IndexSignatureChecker({
                    allowSymbolIndexSignatures: false,
                });
                const symbolType = { flags: TypeFlags.ESSymbol } as Type;
                const result = disabledChecker.validateIndexSignatureKeyType(symbolType);
                expect(result.isValid).toBe(false);
                expect(result.diagnostics).toHaveLength(1);
            });

            it("should accept string literal type", () => {
                const literalType = { flags: TypeFlags.StringLiteral } as Type;
                const result = checker.validateIndexSignatureKeyType(literalType);
                expect(result.isValid).toBe(true);
            });

            it("should accept template literal type", () => {
                const templateType = { flags: TypeFlags.TemplateLiteral } as Type;
                const result = checker.validateIndexSignatureKeyType(templateType);
                expect(result.isValid).toBe(true);
            });

            it("should reject boolean type", () => {
                const boolType = { flags: TypeFlags.Boolean } as Type;
                const result = checker.validateIndexSignatureKeyType(boolType);
                expect(result.isValid).toBe(false);
                expect(result.diagnostics.length).toBeGreaterThan(0);
            });

            it("should reject object type", () => {
                const objType = { flags: TypeFlags.Object } as Type;
                const result = checker.validateIndexSignatureKeyType(objType);
                expect(result.isValid).toBe(false);
            });

            it("should accept union of valid types", () => {
                const unionType = {
                    flags: TypeFlags.Union,
                    types: [
                        { flags: TypeFlags.String } as Type,
                        { flags: TypeFlags.Number } as Type,
                    ],
                } as UnionType;
                const result = checker.validateIndexSignatureKeyType(unionType);
                expect(result.isValid).toBe(true);
            });
        });

        describe("getIndexInfosOfType", () => {
            const mockGetApparentType = (type: Type) => type;
            const mockResolveMembers = () => {};

            it("should return empty array for type without index signatures", () => {
                const type = {
                    id: 1,
                    flags: TypeFlags.Object,
                    objectFlags: 0,
                } as ObjectType;

                const result = checker.getIndexInfosOfType(
                    type,
                    mockGetApparentType,
                    mockResolveMembers
                );

                expect(result).toHaveLength(0);
            });

            it("should return index signatures from object type", () => {
                const stringType = { id: 10, flags: TypeFlags.String } as Type;
                const valueType = { id: 11, flags: TypeFlags.Any } as Type;

                const indexInfo: IndexInfo = {
                    keyType: stringType,
                    type: valueType,
                    isReadonly: false,
                };

                const type = {
                    id: 1,
                    flags: TypeFlags.Object,
                    objectFlags: 0,
                    indexInfos: [indexInfo],
                } as ObjectType;

                const result = checker.getIndexInfosOfType(
                    type,
                    mockGetApparentType,
                    mockResolveMembers
                );

                expect(result).toHaveLength(1);
                expect(result[0].keyType).toBe(stringType);
                expect(result[0].type).toBe(valueType);
            });

            it("should cache results", () => {
                const type = {
                    id: 1,
                    flags: TypeFlags.Object,
                    objectFlags: 0,
                    indexInfos: [],
                } as ObjectType;

                const result1 = checker.getIndexInfosOfType(
                    type,
                    mockGetApparentType,
                    mockResolveMembers
                );
                const result2 = checker.getIndexInfosOfType(
                    type,
                    mockGetApparentType,
                    mockResolveMembers
                );

                expect(result1).toBe(result2);
            });
        });

        describe("getApplicableIndexInfo", () => {
            const mockGetApparentType = (type: Type) => type;
            const mockResolveMembers = () => {};

            it("should find exact matching index signature", () => {
                const stringType = { id: 10, flags: TypeFlags.String } as Type;
                const valueType = { id: 11, flags: TypeFlags.Any } as Type;

                const type = {
                    id: 1,
                    flags: TypeFlags.Object,
                    objectFlags: 0,
                    indexInfos: [{
                        keyType: stringType,
                        type: valueType,
                        isReadonly: false,
                    }],
                } as ObjectType;

                const result = checker.getApplicableIndexInfo(
                    type,
                    stringType,
                    mockGetApparentType,
                    mockResolveMembers
                );

                expect(result).toBeDefined();
                expect(result!.type).toBe(valueType);
            });

            it("should find applicable string signature for number key", () => {
                const stringType = { id: 10, flags: TypeFlags.String } as Type;
                const numberType = { id: 11, flags: TypeFlags.Number } as Type;
                const valueType = { id: 12, flags: TypeFlags.Any } as Type;

                const type = {
                    id: 1,
                    flags: TypeFlags.Object,
                    objectFlags: 0,
                    indexInfos: [{
                        keyType: stringType,
                        type: valueType,
                        isReadonly: false,
                    }],
                } as ObjectType;

                const result = checker.getApplicableIndexInfo(
                    type,
                    numberType,
                    mockGetApparentType,
                    mockResolveMembers
                );

                expect(result).toBeDefined();
                expect(result!.type).toBe(valueType);
            });

            it("should return undefined when no applicable signature", () => {
                const stringType = { id: 10, flags: TypeFlags.String } as Type;
                const symbolType = { id: 11, flags: TypeFlags.ESSymbol } as Type;
                const valueType = { id: 12, flags: TypeFlags.Any } as Type;

                const type = {
                    id: 1,
                    flags: TypeFlags.Object,
                    objectFlags: 0,
                    indexInfos: [{
                        keyType: stringType,
                        type: valueType,
                        isReadonly: false,
                    }],
                } as ObjectType;

                const result = checker.getApplicableIndexInfo(
                    type,
                    symbolType,
                    mockGetApparentType,
                    mockResolveMembers
                );

                expect(result).toBeUndefined();
            });
        });

        describe("checkIndexSignatureCompatibility", () => {
            const mockGetApparentType = (type: Type) => type;
            const mockResolveMembers = () => {};
            const mockIsAssignable = () => true;

            it("should report compatible when source has matching signatures", () => {
                const stringType = { id: 10, flags: TypeFlags.String } as Type;
                const valueType = { id: 11, flags: TypeFlags.Any } as Type;

                const indexInfo: IndexInfo = {
                    keyType: stringType,
                    type: valueType,
                    isReadonly: false,
                };

                const source = {
                    id: 1,
                    flags: TypeFlags.Object,
                    objectFlags: 0,
                    indexInfos: [indexInfo],
                } as ObjectType;

                const target = {
                    id: 2,
                    flags: TypeFlags.Object,
                    objectFlags: 0,
                    indexInfos: [indexInfo],
                } as ObjectType;

                const result = checker.checkIndexSignatureCompatibility(
                    source,
                    target,
                    mockGetApparentType,
                    mockResolveMembers,
                    mockIsAssignable
                );

                expect(result.isCompatible).toBe(true);
            });

            it("should report incompatible when source missing signature", () => {
                const stringType = { id: 10, flags: TypeFlags.String } as Type;
                const valueType = { id: 11, flags: TypeFlags.Any } as Type;

                const source = {
                    id: 1,
                    flags: TypeFlags.Object,
                    objectFlags: 0,
                    indexInfos: [],
                } as ObjectType;

                const target = {
                    id: 2,
                    flags: TypeFlags.Object,
                    objectFlags: 0,
                    indexInfos: [{
                        keyType: stringType,
                        type: valueType,
                        isReadonly: false,
                    }],
                } as ObjectType;

                const result = checker.checkIndexSignatureCompatibility(
                    source,
                    target,
                    mockGetApparentType,
                    mockResolveMembers,
                    mockIsAssignable
                );

                expect(result.isCompatible).toBe(false);
            });
        });

        describe("getIndexType (keyof)", () => {
            const mockGetApparentType = (type: Type) => type;
            const mockResolveMembers = () => {};
            const mockGetProperties = (type: Type) => {
                if ((type as any).properties) {
                    return (type as any).properties;
                }
                return [];
            };
            const mockCreateLiteral = (value: string | number) => ({
                id: Math.random(),
                flags: typeof value === "string" ? TypeFlags.StringLiteral : TypeFlags.NumberLiteral,
                value,
            } as Type);
            const mockGetUnion = (types: Type[]) => {
                if (types.length === 0) return neverType;
                if (types.length === 1) return types[0];
                return { id: Math.random(), flags: TypeFlags.Union, types } as UnionType;
            };

            const stringType = { id: 1, flags: TypeFlags.String } as Type;
            const numberType = { id: 2, flags: TypeFlags.Number } as Type;
            const symbolType = { id: 3, flags: TypeFlags.ESSymbol } as Type;
            const neverType = { id: 4, flags: TypeFlags.Never } as Type;

            it("should return property names as literal types", () => {
                const prop1 = { escapedName: "foo" as __String, flags: 0 } as Symbol;
                const prop2 = { escapedName: "bar" as __String, flags: 0 } as Symbol;

                const type = {
                    id: 10,
                    flags: TypeFlags.Object,
                    objectFlags: 0,
                    properties: [prop1, prop2],
                    indexInfos: [],
                } as any;

                const result = checker.getIndexType(
                    type,
                    false,
                    true,
                    mockGetApparentType,
                    mockResolveMembers,
                    mockGetProperties,
                    mockCreateLiteral,
                    mockGetUnion,
                    stringType,
                    numberType,
                    symbolType,
                    neverType
                );

                expect(result.propertyNames).toContain("foo" as __String);
                expect(result.propertyNames).toContain("bar" as __String);
            });

            it("should include index key types when not disabled", () => {
                const type = {
                    id: 10,
                    flags: TypeFlags.Object,
                    objectFlags: 0,
                    indexInfos: [{
                        keyType: stringType,
                        type: { id: 20, flags: TypeFlags.Any } as Type,
                        isReadonly: false,
                    }],
                } as ObjectType;

                const result = checker.getIndexType(
                    type,
                    false,
                    false,
                    mockGetApparentType,
                    mockResolveMembers,
                    () => [],
                    mockCreateLiteral,
                    mockGetUnion,
                    stringType,
                    numberType,
                    symbolType,
                    neverType
                );

                expect(result.indexKeyTypes).toContain(stringType);
            });

            it("should exclude index signatures when noIndexSignatures is true", () => {
                const type = {
                    id: 10,
                    flags: TypeFlags.Object,
                    objectFlags: 0,
                    indexInfos: [{
                        keyType: stringType,
                        type: { id: 20, flags: TypeFlags.Any } as Type,
                        isReadonly: false,
                    }],
                } as ObjectType;

                const result = checker.getIndexType(
                    type,
                    false,
                    true, // noIndexSignatures
                    mockGetApparentType,
                    mockResolveMembers,
                    () => [],
                    mockCreateLiteral,
                    mockGetUnion,
                    stringType,
                    numberType,
                    symbolType,
                    neverType
                );

                expect(result.indexKeyTypes).toHaveLength(0);
            });

            it("should filter non-string keys when stringsOnly is true", () => {
                const type = {
                    id: 10,
                    flags: TypeFlags.Object,
                    objectFlags: 0,
                    indexInfos: [
                        {
                            keyType: stringType,
                            type: { id: 20, flags: TypeFlags.Any } as Type,
                            isReadonly: false,
                        },
                        {
                            keyType: symbolType,
                            type: { id: 21, flags: TypeFlags.Any } as Type,
                            isReadonly: false,
                        },
                    ],
                } as ObjectType;

                const result = checker.getIndexType(
                    type,
                    true, // stringsOnly
                    false,
                    mockGetApparentType,
                    mockResolveMembers,
                    () => [],
                    mockCreateLiteral,
                    mockGetUnion,
                    stringType,
                    numberType,
                    symbolType,
                    neverType
                );

                expect(result.stringsOnly).toBe(true);
                expect(result.indexKeyTypes).not.toContain(symbolType);
            });
        });

        describe("clearCache", () => {
            it("should clear cached index infos", () => {
                const mockGetApparentType = (type: Type) => type;
                const mockResolveMembers = () => {};

                const type = {
                    id: 1,
                    flags: TypeFlags.Object,
                    objectFlags: 0,
                    indexInfos: [],
                } as ObjectType;

                // Populate cache
                checker.getIndexInfosOfType(type, mockGetApparentType, mockResolveMembers);

                // Clear cache
                checker.clearCache();

                // Should work again (won't throw)
                const result = checker.getIndexInfosOfType(
                    type,
                    mockGetApparentType,
                    mockResolveMembers
                );
                expect(result).toHaveLength(0);
            });
        });
    });
}
