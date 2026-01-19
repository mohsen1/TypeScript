/**
 * @fileoverview Tests for indexed type definitions
 */

namespace ts.tests {
    describe("IndexAccessFlags", () => {
        it("should have correct flag values", () => {
            expect(IndexAccessFlags.None).toBe(0);
            expect(IndexAccessFlags.IncludeUndefined).toBe(1);
            expect(IndexAccessFlags.NoIndexSignatures).toBe(2);
            expect(IndexAccessFlags.Writing).toBe(4);
        });

        it("should allow combining flags", () => {
            const combined = IndexAccessFlags.IncludeUndefined | IndexAccessFlags.Writing;
            expect(combined & IndexAccessFlags.IncludeUndefined).toBeTruthy();
            expect(combined & IndexAccessFlags.Writing).toBeTruthy();
            expect(combined & IndexAccessFlags.NoIndexSignatures).toBeFalsy();
        });

        it("should have Persistent equal to IncludeUndefined", () => {
            expect(IndexAccessFlags.Persistent).toBe(IndexAccessFlags.IncludeUndefined);
        });
    });

    describe("IndexKeyType", () => {
        it("should have correct enum values", () => {
            expect(IndexKeyType.String).toBe(0);
            expect(IndexKeyType.Number).toBe(1);
            expect(IndexKeyType.Symbol).toBe(2);
        });
    });

    describe("createIndexedAccessCacheKey", () => {
        it("should create unique cache keys", () => {
            const type1 = { id: 1 } as Type;
            const type2 = { id: 2 } as Type;
            const type3 = { id: 3 } as Type;

            const key1 = createIndexedAccessCacheKey(type1, type2, IndexAccessFlags.None);
            const key2 = createIndexedAccessCacheKey(type1, type3, IndexAccessFlags.None);
            const key3 = createIndexedAccessCacheKey(type1, type2, IndexAccessFlags.Writing);

            expect(key1).not.toBe(key2);
            expect(key1).not.toBe(key3);
            expect(key2).not.toBe(key3);
        });

        it("should create same key for same inputs", () => {
            const type1 = { id: 1 } as Type;
            const type2 = { id: 2 } as Type;

            const key1 = createIndexedAccessCacheKey(type1, type2, IndexAccessFlags.None);
            const key2 = createIndexedAccessCacheKey(type1, type2, IndexAccessFlags.None);

            expect(key1).toBe(key2);
        });
    });

    describe("createIndexResolutionState", () => {
        it("should create state with correct defaults", () => {
            const objectType = { id: 1 } as Type;
            const indexType = { id: 2 } as Type;

            const state = createIndexResolutionState(objectType, indexType);

            expect(state.objectType).toBe(objectType);
            expect(state.indexType).toBe(indexType);
            expect(state.accessFlags).toBe(IndexAccessFlags.None);
            expect(state.depth).toBe(0);
            expect(state.isWriting).toBe(false);
            expect(state.cache).toBeInstanceOf(Map);
        });

        it("should set isWriting when Writing flag is set", () => {
            const objectType = { id: 1 } as Type;
            const indexType = { id: 2 } as Type;

            const state = createIndexResolutionState(
                objectType,
                indexType,
                IndexAccessFlags.Writing
            );

            expect(state.isWriting).toBe(true);
        });
    });

    describe("isTypeUsableAsPropertyName", () => {
        it("should return true for string literal type", () => {
            const type = { flags: TypeFlags.StringLiteral } as Type;
            expect(isTypeUsableAsPropertyName(type)).toBe(true);
        });

        it("should return true for number literal type", () => {
            const type = { flags: TypeFlags.NumberLiteral } as Type;
            expect(isTypeUsableAsPropertyName(type)).toBe(true);
        });

        it("should return true for unique symbol type", () => {
            const type = { flags: TypeFlags.UniqueESSymbol } as Type;
            expect(isTypeUsableAsPropertyName(type)).toBe(true);
        });

        it("should return false for string type", () => {
            const type = { flags: TypeFlags.String } as Type;
            expect(isTypeUsableAsPropertyName(type)).toBe(false);
        });

        it("should return false for number type", () => {
            const type = { flags: TypeFlags.Number } as Type;
            expect(isTypeUsableAsPropertyName(type)).toBe(false);
        });
    });

    describe("getPropertyNameFromLiteralType", () => {
        it("should extract name from string literal", () => {
            const type = {
                flags: TypeFlags.StringLiteral,
                value: "foo",
            } as StringLiteralType;

            const name = getPropertyNameFromLiteralType(type);
            expect(unescapeLeadingUnderscores(name!)).toBe("foo");
        });

        it("should extract name from number literal", () => {
            const type = {
                flags: TypeFlags.NumberLiteral,
                value: 42,
            } as NumberLiteralType;

            const name = getPropertyNameFromLiteralType(type);
            expect(unescapeLeadingUnderscores(name!)).toBe("42");
        });

        it("should extract name from unique symbol", () => {
            const type = {
                flags: TypeFlags.UniqueESSymbol,
                escapedName: "__@mySymbol" as __String,
            } as UniqueESSymbolType;

            const name = getPropertyNameFromLiteralType(type);
            expect(name).toBe("__@mySymbol");
        });

        it("should return undefined for non-literal type", () => {
            const type = { flags: TypeFlags.String } as Type;
            const name = getPropertyNameFromLiteralType(type);
            expect(name).toBeUndefined();
        });
    });

    describe("isValidIndexKeyType", () => {
        it("should return true for string type", () => {
            const type = { flags: TypeFlags.String } as Type;
            expect(isValidIndexKeyType(type)).toBe(true);
        });

        it("should return true for number type", () => {
            const type = { flags: TypeFlags.Number } as Type;
            expect(isValidIndexKeyType(type)).toBe(true);
        });

        it("should return true for symbol type", () => {
            const type = { flags: TypeFlags.ESSymbol } as Type;
            expect(isValidIndexKeyType(type)).toBe(true);
        });

        it("should return true for string literal type", () => {
            const type = { flags: TypeFlags.StringLiteral } as Type;
            expect(isValidIndexKeyType(type)).toBe(true);
        });

        it("should return true for template literal type", () => {
            const type = { flags: TypeFlags.TemplateLiteral } as Type;
            expect(isValidIndexKeyType(type)).toBe(true);
        });

        it("should return false for boolean type", () => {
            const type = { flags: TypeFlags.Boolean } as Type;
            expect(isValidIndexKeyType(type)).toBe(false);
        });

        it("should return false for object type", () => {
            const type = { flags: TypeFlags.Object } as Type;
            expect(isValidIndexKeyType(type)).toBe(false);
        });
    });

    describe("getIndexKeyTypeCategory", () => {
        it("should return String for string type", () => {
            const type = { flags: TypeFlags.String } as Type;
            expect(getIndexKeyTypeCategory(type)).toBe(IndexKeyType.String);
        });

        it("should return String for string literal type", () => {
            const type = { flags: TypeFlags.StringLiteral } as Type;
            expect(getIndexKeyTypeCategory(type)).toBe(IndexKeyType.String);
        });

        it("should return Number for number type", () => {
            const type = { flags: TypeFlags.Number } as Type;
            expect(getIndexKeyTypeCategory(type)).toBe(IndexKeyType.Number);
        });

        it("should return Symbol for symbol type", () => {
            const type = { flags: TypeFlags.ESSymbol } as Type;
            expect(getIndexKeyTypeCategory(type)).toBe(IndexKeyType.Symbol);
        });

        it("should return undefined for invalid key type", () => {
            const type = { flags: TypeFlags.Boolean } as Type;
            expect(getIndexKeyTypeCategory(type)).toBeUndefined();
        });
    });

    describe("isKeyTypeApplicableToIndexSignature", () => {
        it("should match same types", () => {
            const keyType = { id: 1, flags: TypeFlags.String } as Type;
            expect(isKeyTypeApplicableToIndexSignature(keyType, keyType)).toBe(true);
        });

        it("should allow number key for string signature", () => {
            const numberKey = { id: 1, flags: TypeFlags.Number } as Type;
            const stringSig = { id: 2, flags: TypeFlags.String } as Type;
            expect(isKeyTypeApplicableToIndexSignature(numberKey, stringSig)).toBe(true);
        });

        it("should allow string key for string signature", () => {
            const stringKey = { id: 1, flags: TypeFlags.String } as Type;
            const stringSig = { id: 2, flags: TypeFlags.String } as Type;
            expect(isKeyTypeApplicableToIndexSignature(stringKey, stringSig)).toBe(true);
        });

        it("should not allow string key for number signature", () => {
            const stringKey = { id: 1, flags: TypeFlags.String } as Type;
            const numberSig = { id: 2, flags: TypeFlags.Number } as Type;
            expect(isKeyTypeApplicableToIndexSignature(stringKey, numberSig)).toBe(false);
        });

        it("should allow number key for number signature", () => {
            const numberKey = { id: 1, flags: TypeFlags.Number } as Type;
            const numberSig = { id: 2, flags: TypeFlags.Number } as Type;
            expect(isKeyTypeApplicableToIndexSignature(numberKey, numberSig)).toBe(true);
        });

        it("should only allow symbol key for symbol signature", () => {
            const symbolKey = { id: 1, flags: TypeFlags.ESSymbol } as Type;
            const symbolSig = { id: 2, flags: TypeFlags.ESSymbol } as Type;
            const stringSig = { id: 3, flags: TypeFlags.String } as Type;

            expect(isKeyTypeApplicableToIndexSignature(symbolKey, symbolSig)).toBe(true);
            expect(isKeyTypeApplicableToIndexSignature(symbolKey, stringSig)).toBe(false);
        });
    });

    describe("isIndexedAccessType", () => {
        it("should return true for indexed access type", () => {
            const type = { flags: TypeFlags.IndexedAccess } as Type;
            expect(isIndexedAccessType(type)).toBe(true);
        });

        it("should return false for other types", () => {
            const type = { flags: TypeFlags.String } as Type;
            expect(isIndexedAccessType(type)).toBe(false);
        });
    });

    describe("isIndexType", () => {
        it("should return true for index type (keyof)", () => {
            const type = { flags: TypeFlags.Index } as Type;
            expect(isIndexType(type)).toBe(true);
        });

        it("should return false for other types", () => {
            const type = { flags: TypeFlags.String } as Type;
            expect(isIndexType(type)).toBe(false);
        });
    });

    describe("IndexedAccessWorkKind", () => {
        it("should have distinct values", () => {
            const values = [
                IndexedAccessWorkKind.PropertyAccess,
                IndexedAccessWorkKind.IndexSignature,
                IndexedAccessWorkKind.UnionDistribute,
                IndexedAccessWorkKind.IntersectionCombine,
                IndexedAccessWorkKind.TupleElement,
                IndexedAccessWorkKind.MappedType,
                IndexedAccessWorkKind.ConditionalType,
                IndexedAccessWorkKind.DeferredAccess,
            ];

            const uniqueValues = new Set(values);
            expect(uniqueValues.size).toBe(values.length);
        });
    });
}
