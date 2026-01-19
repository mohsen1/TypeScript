/**
 * @fileoverview Index signature checking for TypeScript type system
 *
 * This module handles:
 * - Checking index signature declarations
 * - Number and string index signature validation
 * - Symbol index signature support
 * - Index signature compatibility checking
 * - keyof operator implementation
 */

namespace ts {
    /**
     * Configuration for index signature checker
     */
    export interface IndexSignatureCheckerConfig {
        /** Maximum depth for recursive type checking */
        readonly maxDepth: number;
        /** Maximum number of index signatures per type */
        readonly maxIndexSignatures: number;
        /** Whether to allow symbol index signatures */
        readonly allowSymbolIndexSignatures: boolean;
        /** Whether to check index signature/property compatibility strictly */
        readonly strictChecking: boolean;
    }

    /**
     * Default configuration
     */
    export const defaultIndexSignatureCheckerConfig: IndexSignatureCheckerConfig = {
        maxDepth: 100,
        maxIndexSignatures: 50,
        allowSymbolIndexSignatures: true,
        strictChecking: true,
    };

    /**
     * Result of checking an index signature declaration
     */
    export interface IndexSignatureCheckResult {
        /** Whether the declaration is valid */
        readonly isValid: boolean;
        /** The index info created from the declaration */
        readonly indexInfo?: IndexSignatureInfo;
        /** Diagnostics produced */
        readonly diagnostics: IndexSignatureDiagnostic[];
    }

    /**
     * Diagnostic from index signature checking
     */
    export interface IndexSignatureDiagnostic {
        /** Diagnostic category */
        readonly category: DiagnosticCategory;
        /** Error code */
        readonly code: number;
        /** Error message */
        readonly message: string;
        /** Related node */
        readonly node?: Node;
    }

    /**
     * IndexSignatureChecker - Validates index signature declarations and compatibility
     */
    export class IndexSignatureChecker {
        private readonly config: IndexSignatureCheckerConfig;
        private readonly indexInfoCache: Map<string, IndexSignatureInfo[]> = new Map();
        private currentDepth: number = 0;

        constructor(config: Partial<IndexSignatureCheckerConfig> = {}) {
            this.config = { ...defaultIndexSignatureCheckerConfig, ...config };
        }

        /**
         * Check an index signature declaration
         */
        public checkIndexSignatureDeclaration(
            declaration: IndexSignatureDeclaration,
            getTypeOfNode: (node: Node) => Type
        ): IndexSignatureCheckResult {
            const diagnostics: IndexSignatureDiagnostic[] = [];

            // Validate that declaration has exactly one parameter
            const parameters = declaration.parameters;
            if (parameters.length !== 1) {
                diagnostics.push({
                    category: DiagnosticCategory.Error,
                    code: 1096,
                    message: "An index signature must have exactly one parameter.",
                    node: declaration,
                });
                return { isValid: false, diagnostics };
            }

            const parameter = parameters[0];

            // Check parameter has a type annotation
            if (!parameter.type) {
                diagnostics.push({
                    category: DiagnosticCategory.Error,
                    code: 1022,
                    message: "An index signature parameter must have a type annotation.",
                    node: parameter,
                });
                return { isValid: false, diagnostics };
            }

            // Get the key type
            const keyType = getTypeOfNode(parameter.type);

            // Validate key type
            const keyTypeValidation = this.validateIndexSignatureKeyType(keyType, parameter.type);
            if (!keyTypeValidation.isValid) {
                diagnostics.push(...keyTypeValidation.diagnostics);
                return { isValid: false, diagnostics };
            }

            // Check return type exists
            if (!declaration.type) {
                diagnostics.push({
                    category: DiagnosticCategory.Error,
                    code: 1021,
                    message: "An index signature must have a type annotation.",
                    node: declaration,
                });
                return { isValid: false, diagnostics };
            }

            // Get the value type
            const valueType = getTypeOfNode(declaration.type);

            // Check for readonly modifier
            const isReadonly = hasEffectiveModifier(declaration, ModifierFlags.Readonly);

            // Create the index info
            const indexInfo: IndexSignatureInfo = {
                keyType,
                type: valueType,
                isReadonly,
                declaration,
            };

            return {
                isValid: true,
                indexInfo,
                diagnostics,
            };
        }

        /**
         * Validate that a type is valid as an index signature key
         */
        public validateIndexSignatureKeyType(
            type: Type,
            node?: Node
        ): { isValid: boolean; diagnostics: IndexSignatureDiagnostic[] } {
            const diagnostics: IndexSignatureDiagnostic[] = [];

            // Valid key types: string, number, symbol, template literal, or union thereof
            if (this.isValidIndexKeyType(type)) {
                return { isValid: true, diagnostics };
            }

            // Check for union of valid types
            if (type.flags & TypeFlags.Union) {
                const unionType = type as UnionType;
                const allValid = unionType.types.every(t => this.isValidIndexKeyType(t));
                if (allValid) {
                    return { isValid: true, diagnostics };
                }
            }

            // Check for symbol (if enabled)
            if (type.flags & TypeFlags.ESSymbol) {
                if (!this.config.allowSymbolIndexSignatures) {
                    diagnostics.push({
                        category: DiagnosticCategory.Error,
                        code: 1023,
                        message: "Symbol index signatures are not enabled.",
                        node,
                    });
                    return { isValid: false, diagnostics };
                }
                return { isValid: true, diagnostics };
            }

            diagnostics.push({
                category: DiagnosticCategory.Error,
                code: 1023,
                message: "An index signature parameter type must be 'string', 'number', 'symbol', or a template literal type.",
                node,
            });

            return { isValid: false, diagnostics };
        }

        /**
         * Check if type is a valid index key type
         */
        private isValidIndexKeyType(type: Type): boolean {
            return !!(
                (type.flags & TypeFlags.String) ||
                (type.flags & TypeFlags.Number) ||
                (type.flags & TypeFlags.StringLiteral) ||
                (type.flags & TypeFlags.NumberLiteral) ||
                (type.flags & TypeFlags.TemplateLiteral) ||
                (type.flags & TypeFlags.ESSymbol) ||
                (type.flags & TypeFlags.UniqueESSymbol)
            );
        }

        /**
         * Get all index signatures from a type
         */
        public getIndexInfosOfType(
            type: Type,
            getApparentType: (type: Type) => Type,
            resolveMembers: (type: Type) => void
        ): IndexSignatureInfo[] {
            // Check cache
            const cacheKey = String(type.id);
            const cached = this.indexInfoCache.get(cacheKey);
            if (cached !== undefined) {
                return cached;
            }

            const result: IndexSignatureInfo[] = [];

            // Get apparent type for primitives
            const apparentType = getApparentType(type);

            // Ensure members are resolved
            resolveMembers(apparentType);

            // Get index infos from object type
            if (apparentType.flags & TypeFlags.Object) {
                const objectType = apparentType as ObjectType;
                if (objectType.indexInfos) {
                    for (const info of objectType.indexInfos) {
                        result.push({
                            keyType: info.keyType,
                            type: info.type,
                            isReadonly: info.isReadonly,
                            declaration: info.declaration,
                        });
                    }
                }
            }

            // Handle union types - intersect index signatures
            if (type.flags & TypeFlags.Union) {
                const unionResult = this.getIndexInfosOfUnionType(
                    type as UnionType,
                    getApparentType,
                    resolveMembers
                );
                result.push(...unionResult);
            }

            // Handle intersection types - union index signatures
            if (type.flags & TypeFlags.Intersection) {
                const intersectionResult = this.getIndexInfosOfIntersectionType(
                    type as IntersectionType,
                    getApparentType,
                    resolveMembers
                );
                result.push(...intersectionResult);
            }

            // Cache and return
            this.indexInfoCache.set(cacheKey, result);
            return result;
        }

        /**
         * Get index infos from a union type (requires all members to have the signature)
         */
        private getIndexInfosOfUnionType(
            unionType: UnionType,
            getApparentType: (type: Type) => Type,
            resolveMembers: (type: Type) => void
        ): IndexSignatureInfo[] {
            if (unionType.types.length === 0) {
                return [];
            }

            // Get index infos from first type
            const firstInfos = this.getIndexInfosOfType(
                unionType.types[0],
                getApparentType,
                resolveMembers
            );

            if (firstInfos.length === 0) {
                return [];
            }

            // For each index info in first type, check all other types have compatible signature
            const result: IndexSignatureInfo[] = [];

            for (const info of firstInfos) {
                let isCommon = true;
                const valueTypes: Type[] = [info.type];
                let isReadonly = info.isReadonly;

                for (let i = 1; i < unionType.types.length; i++) {
                    const otherInfos = this.getIndexInfosOfType(
                        unionType.types[i],
                        getApparentType,
                        resolveMembers
                    );

                    const matchingInfo = this.findIndexInfoWithKeyType(otherInfos, info.keyType);
                    if (!matchingInfo) {
                        isCommon = false;
                        break;
                    }

                    valueTypes.push(matchingInfo.type);
                    // Union of readonly and non-readonly is non-readonly
                    isReadonly = isReadonly && matchingInfo.isReadonly;
                }

                if (isCommon) {
                    // Create union of value types
                    result.push({
                        keyType: info.keyType,
                        type: this.createUnionType(valueTypes),
                        isReadonly,
                        declaration: info.declaration,
                    });
                }
            }

            return result;
        }

        /**
         * Get index infos from an intersection type (combines all signatures)
         */
        private getIndexInfosOfIntersectionType(
            intersectionType: IntersectionType,
            getApparentType: (type: Type) => Type,
            resolveMembers: (type: Type) => void
        ): IndexSignatureInfo[] {
            const result: IndexSignatureInfo[] = [];
            const seenKeyTypes = new Set<string>();

            for (const memberType of intersectionType.types) {
                const memberInfos = this.getIndexInfosOfType(
                    memberType,
                    getApparentType,
                    resolveMembers
                );

                for (const info of memberInfos) {
                    const keyTypeKey = String(info.keyType.id);

                    if (!seenKeyTypes.has(keyTypeKey)) {
                        seenKeyTypes.add(keyTypeKey);

                        // Collect all value types for this key type across intersection
                        const valueTypes: Type[] = [];
                        let isReadonly = false;

                        for (const otherMemberType of intersectionType.types) {
                            const otherInfos = this.getIndexInfosOfType(
                                otherMemberType,
                                getApparentType,
                                resolveMembers
                            );
                            const matchingInfo = this.findIndexInfoWithKeyType(otherInfos, info.keyType);
                            if (matchingInfo) {
                                valueTypes.push(matchingInfo.type);
                                // Intersection of readonly and non-readonly is readonly
                                isReadonly = isReadonly || matchingInfo.isReadonly;
                            }
                        }

                        if (valueTypes.length > 0) {
                            result.push({
                                keyType: info.keyType,
                                type: this.createIntersectionType(valueTypes),
                                isReadonly,
                                declaration: info.declaration,
                            });
                        }
                    }
                }
            }

            return result;
        }

        /**
         * Find an index info with a matching key type
         */
        private findIndexInfoWithKeyType(
            infos: IndexSignatureInfo[],
            keyType: Type
        ): IndexSignatureInfo | undefined {
            return infos.find(info => this.areKeyTypesEquivalent(info.keyType, keyType));
        }

        /**
         * Check if two key types are equivalent for index signature matching
         */
        private areKeyTypesEquivalent(a: Type, b: Type): boolean {
            if (a === b) return true;
            if (a.id === b.id) return true;

            // String and number key types
            if ((a.flags & TypeFlags.String) && (b.flags & TypeFlags.String)) return true;
            if ((a.flags & TypeFlags.Number) && (b.flags & TypeFlags.Number)) return true;
            if ((a.flags & TypeFlags.ESSymbol) && (b.flags & TypeFlags.ESSymbol)) return true;

            return false;
        }

        /**
         * Get a specific index info by key type
         */
        public getIndexInfoOfType(
            type: Type,
            keyType: Type,
            getApparentType: (type: Type) => Type,
            resolveMembers: (type: Type) => void
        ): IndexSignatureInfo | undefined {
            const infos = this.getIndexInfosOfType(type, getApparentType, resolveMembers);
            return this.findIndexInfoWithKeyType(infos, keyType);
        }

        /**
         * Find applicable index info for a given key type
         */
        public getApplicableIndexInfo(
            type: Type,
            indexType: Type,
            getApparentType: (type: Type) => Type,
            resolveMembers: (type: Type) => void
        ): IndexSignatureInfo | undefined {
            const infos = this.getIndexInfosOfType(type, getApparentType, resolveMembers);

            // First try exact match
            const exactMatch = infos.find(info =>
                this.areKeyTypesEquivalent(info.keyType, indexType)
            );
            if (exactMatch) {
                return exactMatch;
            }

            // Check applicable matches
            const applicableInfos: IndexSignatureInfo[] = [];
            for (const info of infos) {
                if (isKeyTypeApplicableToIndexSignature(indexType, info.keyType)) {
                    applicableInfos.push(info);
                }
            }

            if (applicableInfos.length === 0) {
                return undefined;
            }

            if (applicableInfos.length === 1) {
                return applicableInfos[0];
            }

            // Multiple applicable - intersect their value types
            const intersectedType = this.createIntersectionType(
                applicableInfos.map(info => info.type)
            );

            return {
                keyType: indexType,
                type: intersectedType,
                isReadonly: applicableInfos.some(info => info.isReadonly),
                declaration: applicableInfos[0].declaration,
            };
        }

        /**
         * Check compatibility between source and target index signatures
         */
        public checkIndexSignatureCompatibility(
            source: Type,
            target: Type,
            getApparentType: (type: Type) => Type,
            resolveMembers: (type: Type) => void,
            isTypeAssignableTo: (source: Type, target: Type) => boolean
        ): IndexSignatureCompatibility {
            const sourceInfos = this.getIndexInfosOfType(source, getApparentType, resolveMembers);
            const targetInfos = this.getIndexInfosOfType(target, getApparentType, resolveMembers);

            const details: IndexSignatureCompatibilityDetail[] = [];
            let isCompatible = true;

            // For each target index signature, source must have a compatible one
            for (const targetInfo of targetInfos) {
                const sourceInfo = this.findIndexInfoWithKeyType(sourceInfos, targetInfo.keyType);

                if (!sourceInfo) {
                    // Check if source has an applicable index signature
                    const applicableInfo = sourceInfos.find(info =>
                        isKeyTypeApplicableToIndexSignature(targetInfo.keyType, info.keyType)
                    );

                    if (!applicableInfo) {
                        details.push({
                            source: targetInfo,
                            isCompatible: false,
                            reason: `Source type is missing an index signature for '${this.getKeyTypeDescription(targetInfo.keyType)}'.`,
                        });
                        isCompatible = false;
                        continue;
                    }
                }

                const actualSourceInfo = sourceInfo || sourceInfos.find(info =>
                    isKeyTypeApplicableToIndexSignature(targetInfo.keyType, info.keyType)
                )!;

                // Check value type compatibility
                if (!isTypeAssignableTo(actualSourceInfo.type, targetInfo.type)) {
                    details.push({
                        source: actualSourceInfo,
                        target: targetInfo,
                        isCompatible: false,
                        reason: `Index signature value types are incompatible.`,
                    });
                    isCompatible = false;
                    continue;
                }

                // Check readonly compatibility (readonly target requires readonly source in strict mode)
                if (this.config.strictChecking && targetInfo.isReadonly && !actualSourceInfo.isReadonly) {
                    // This is actually fine - non-readonly can be assigned to readonly
                }

                // Check the opposite: can't assign readonly to non-readonly for mutation
                if (!targetInfo.isReadonly && actualSourceInfo.isReadonly) {
                    details.push({
                        source: actualSourceInfo,
                        target: targetInfo,
                        isCompatible: false,
                        reason: `Cannot assign readonly index signature to non-readonly target.`,
                    });
                    isCompatible = false;
                    continue;
                }

                details.push({
                    source: actualSourceInfo,
                    target: targetInfo,
                    isCompatible: true,
                });
            }

            return {
                isCompatible,
                details,
                errorMessage: isCompatible ? undefined : "Index signatures are incompatible.",
            };
        }

        /**
         * Check that all properties are compatible with index signatures
         */
        public checkPropertiesAgainstIndexSignatures(
            type: Type,
            properties: Symbol[],
            getApparentType: (type: Type) => Type,
            resolveMembers: (type: Type) => void,
            getTypeOfSymbol: (symbol: Symbol) => Type,
            isTypeAssignableTo: (source: Type, target: Type) => boolean
        ): IndexSignatureDiagnostic[] {
            const diagnostics: IndexSignatureDiagnostic[] = [];
            const indexInfos = this.getIndexInfosOfType(type, getApparentType, resolveMembers);

            if (indexInfos.length === 0) {
                return diagnostics;
            }

            for (const prop of properties) {
                const propType = getTypeOfSymbol(prop);
                const propName = symbolName(prop);

                // Find applicable index signatures for this property
                for (const indexInfo of indexInfos) {
                    const isApplicable = this.isPropertyApplicableToIndexSignature(
                        propName,
                        indexInfo.keyType
                    );

                    if (isApplicable) {
                        // Property type must be assignable to index signature value type
                        if (!isTypeAssignableTo(propType, indexInfo.type)) {
                            diagnostics.push({
                                category: DiagnosticCategory.Error,
                                code: 2411,
                                message: `Property '${propName}' of type '${this.typeToString(propType)}' is not assignable to '${this.getKeyTypeDescription(indexInfo.keyType)}' index type '${this.typeToString(indexInfo.type)}'.`,
                            });
                        }
                    }
                }
            }

            return diagnostics;
        }

        /**
         * Check if a property name applies to an index signature
         */
        private isPropertyApplicableToIndexSignature(
            propName: string,
            indexKeyType: Type
        ): boolean {
            // String index signature applies to all properties
            if (indexKeyType.flags & TypeFlags.String) {
                return true;
            }

            // Number index signature applies to numeric property names
            if (indexKeyType.flags & TypeFlags.Number) {
                return /^[0-9]+$/.test(propName);
            }

            // Symbol index signature doesn't apply to string property names
            if (indexKeyType.flags & TypeFlags.ESSymbol) {
                return false;
            }

            // String literal key type
            if (indexKeyType.flags & TypeFlags.StringLiteral) {
                return propName === (indexKeyType as StringLiteralType).value;
            }

            return false;
        }

        /**
         * Implement keyof operator
         */
        public getIndexType(
            type: Type,
            stringsOnly: boolean,
            noIndexSignatures: boolean,
            getApparentType: (type: Type) => Type,
            resolveMembers: (type: Type) => void,
            getPropertiesOfType: (type: Type) => Symbol[],
            createLiteralType: (value: string | number) => Type,
            getUnionType: (types: Type[]) => Type,
            stringType: Type,
            numberType: Type,
            symbolType: Type,
            neverType: Type
        ): KeyofResult {
            const propertyNames: __String[] = [];
            const indexKeyTypes: Type[] = [];
            const resultTypes: Type[] = [];

            // Handle union types - distribute keyof
            if (type.flags & TypeFlags.Union) {
                const unionType = type as UnionType;
                const constituentResults = unionType.types.map(t =>
                    this.getIndexType(
                        t, stringsOnly, noIndexSignatures,
                        getApparentType, resolveMembers, getPropertiesOfType,
                        createLiteralType, getUnionType, stringType, numberType, symbolType, neverType
                    )
                );

                // keyof (A | B) = (keyof A) & (keyof B)
                // Find common keys
                const firstNames = new Set(constituentResults[0].propertyNames);
                const commonNames = constituentResults[0].propertyNames.filter(name =>
                    constituentResults.every(r => r.propertyNames.some(n => n === name))
                );

                for (const name of commonNames) {
                    propertyNames.push(name);
                    resultTypes.push(createLiteralType(unescapeLeadingUnderscores(name)));
                }

                return {
                    type: resultTypes.length > 0 ? getUnionType(resultTypes) : neverType,
                    stringsOnly,
                    propertyNames,
                    indexKeyTypes,
                };
            }

            // Handle intersection types
            if (type.flags & TypeFlags.Intersection) {
                const intersectionType = type as IntersectionType;
                const constituentResults = intersectionType.types.map(t =>
                    this.getIndexType(
                        t, stringsOnly, noIndexSignatures,
                        getApparentType, resolveMembers, getPropertiesOfType,
                        createLiteralType, getUnionType, stringType, numberType, symbolType, neverType
                    )
                );

                // keyof (A & B) = (keyof A) | (keyof B)
                const allNames = new Set<__String>();
                for (const r of constituentResults) {
                    for (const name of r.propertyNames) {
                        allNames.add(name);
                    }
                }

                for (const name of allNames) {
                    propertyNames.push(name);
                    resultTypes.push(createLiteralType(unescapeLeadingUnderscores(name)));
                }

                // Include index key types
                for (const r of constituentResults) {
                    for (const keyType of r.indexKeyTypes) {
                        if (!indexKeyTypes.some(k => k.id === keyType.id)) {
                            indexKeyTypes.push(keyType);
                            resultTypes.push(keyType);
                        }
                    }
                }

                return {
                    type: resultTypes.length > 0 ? getUnionType(resultTypes) : neverType,
                    stringsOnly,
                    propertyNames,
                    indexKeyTypes,
                };
            }

            // Get apparent type
            const apparentType = getApparentType(type);
            resolveMembers(apparentType);

            // Get properties
            const properties = getPropertiesOfType(apparentType);
            for (const prop of properties) {
                const name = prop.escapedName;
                propertyNames.push(name);

                // Create literal type for property name
                const nameStr = unescapeLeadingUnderscores(name);
                const isNumeric = /^[0-9]+$/.test(nameStr);

                if (isNumeric && !stringsOnly) {
                    resultTypes.push(createLiteralType(Number(nameStr)));
                } else {
                    resultTypes.push(createLiteralType(nameStr));
                }
            }

            // Get index signatures (unless disabled)
            if (!noIndexSignatures) {
                const indexInfos = this.getIndexInfosOfType(
                    apparentType,
                    getApparentType,
                    resolveMembers
                );

                for (const info of indexInfos) {
                    // Skip non-string types if stringsOnly
                    if (stringsOnly && !(info.keyType.flags & (TypeFlags.String | TypeFlags.StringLiteral))) {
                        continue;
                    }

                    indexKeyTypes.push(info.keyType);

                    // Add the key type itself to results
                    if (info.keyType.flags & TypeFlags.String) {
                        resultTypes.push(stringType);
                    } else if (info.keyType.flags & TypeFlags.Number) {
                        resultTypes.push(numberType);
                    } else if (info.keyType.flags & TypeFlags.ESSymbol) {
                        if (!stringsOnly) {
                            resultTypes.push(symbolType);
                        }
                    } else {
                        resultTypes.push(info.keyType);
                    }
                }
            }

            return {
                type: resultTypes.length > 0 ? getUnionType(resultTypes) : neverType,
                stringsOnly,
                propertyNames,
                indexKeyTypes,
            };
        }

        /**
         * Clear the index info cache
         */
        public clearCache(): void {
            this.indexInfoCache.clear();
        }

        /**
         * Get description of a key type for error messages
         */
        private getKeyTypeDescription(type: Type): string {
            if (type.flags & TypeFlags.String) return "string";
            if (type.flags & TypeFlags.Number) return "number";
            if (type.flags & TypeFlags.ESSymbol) return "symbol";
            if (type.flags & TypeFlags.StringLiteral) {
                return `"${(type as StringLiteralType).value}"`;
            }
            if (type.flags & TypeFlags.NumberLiteral) {
                return String((type as NumberLiteralType).value);
            }
            return "unknown";
        }

        /**
         * Simple type to string (for error messages)
         */
        private typeToString(type: Type): string {
            if (type.flags & TypeFlags.String) return "string";
            if (type.flags & TypeFlags.Number) return "number";
            if (type.flags & TypeFlags.Boolean) return "boolean";
            if (type.flags & TypeFlags.StringLiteral) {
                return `"${(type as StringLiteralType).value}"`;
            }
            if (type.flags & TypeFlags.NumberLiteral) {
                return String((type as NumberLiteralType).value);
            }
            if (type.flags & TypeFlags.Any) return "any";
            if (type.flags & TypeFlags.Unknown) return "unknown";
            if (type.flags & TypeFlags.Never) return "never";
            if (type.flags & TypeFlags.Void) return "void";
            if (type.flags & TypeFlags.Null) return "null";
            if (type.flags & TypeFlags.Undefined) return "undefined";
            return "object";
        }

        /**
         * Create a union type (placeholder - should use actual type checker)
         */
        private createUnionType(types: Type[]): Type {
            if (types.length === 0) {
                // Return a placeholder - actual implementation should use neverType
                return types[0];
            }
            if (types.length === 1) {
                return types[0];
            }
            // Return first type as placeholder - actual implementation should create union
            return types[0];
        }

        /**
         * Create an intersection type (placeholder - should use actual type checker)
         */
        private createIntersectionType(types: Type[]): Type {
            if (types.length === 0) {
                return types[0];
            }
            if (types.length === 1) {
                return types[0];
            }
            // Return first type as placeholder - actual implementation should create intersection
            return types[0];
        }
    }

    /**
     * Helper to check for effective modifier
     */
    function hasEffectiveModifier(node: Node, flags: ModifierFlags): boolean {
        // Check modifiers array
        const modifiers = canHaveModifiers(node) ? node.modifiers : undefined;
        if (modifiers) {
            for (const modifier of modifiers) {
                if (modifier.kind === SyntaxKind.ReadonlyKeyword && (flags & ModifierFlags.Readonly)) {
                    return true;
                }
            }
        }
        return false;
    }
}
