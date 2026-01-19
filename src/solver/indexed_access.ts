/**
 * @fileoverview Indexed access type solver (T[K])
 *
 * This module implements iterative resolution of indexed access types:
 * - Direct property access
 * - Index signature lookup
 * - Tuple element access
 * - Mapped type access
 * - Union/intersection distribution
 * - Bracket notation property access
 */

namespace ts {
    /**
     * Configuration for indexed access solver
     */
    export interface IndexedAccessSolverConfig {
        /** Maximum iterations for worklist processing */
        readonly maxIterations: number;
        /** Maximum worklist size */
        readonly maxWorklistSize: number;
        /** Maximum depth for nested indexed access */
        readonly maxDepth: number;
        /** Whether to include undefined for optional accesses */
        readonly includeUndefinedForOptional: boolean;
    }

    /**
     * Default configuration
     */
    export const defaultIndexedAccessSolverConfig: IndexedAccessSolverConfig = {
        maxIterations: 50000,
        maxWorklistSize: 10000,
        maxDepth: 100,
        includeUndefinedForOptional: true,
    };

    /**
     * Work item for iterative indexed access resolution
     */
    interface WorkItem {
        /** Unique ID for tracking */
        readonly id: number;
        /** Kind of work */
        readonly kind: WorkItemKind;
        /** Object type being indexed */
        readonly objectType: Type;
        /** Index type */
        readonly indexType: Type;
        /** Access flags */
        readonly accessFlags: IndexAccessFlags;
        /** Current depth */
        readonly depth: number;
        /** Parent work item ID */
        readonly parentId: number | null;
        /** Index in parent's results array */
        readonly parentResultIndex: number;
        /** Collected results (for compound items) */
        results: (Type | undefined)[];
        /** Expected result count */
        expectedResults: number;
    }

    /**
     * Types of work items
     */
    const enum WorkItemKind {
        /** Resolve a single indexed access */
        Resolve,
        /** Distribute over union object type */
        UnionObject,
        /** Distribute over union index type */
        UnionIndex,
        /** Combine results from intersection */
        Intersection,
        /** Tuple element access */
        TupleElement,
        /** Mapped type template access */
        MappedType,
        /** Property lookup */
        Property,
        /** Index signature lookup */
        IndexSignature,
    }

    /**
     * IndexedAccessSolver - Iterative resolver for T[K] types
     */
    export class IndexedAccessSolver {
        private readonly config: IndexedAccessSolverConfig;
        private readonly indexSignatureChecker: IndexSignatureChecker;
        private readonly cache: Map<string, Type | undefined> = new Map();
        private nextWorkItemId: number = 0;

        constructor(
            config: Partial<IndexedAccessSolverConfig> = {},
            indexSignatureChecker?: IndexSignatureChecker
        ) {
            this.config = { ...defaultIndexedAccessSolverConfig, ...config };
            this.indexSignatureChecker = indexSignatureChecker || new IndexSignatureChecker();
        }

        /**
         * Resolve an indexed access type T[K]
         */
        public resolve(
            objectType: Type,
            indexType: Type,
            accessFlags: IndexAccessFlags,
            context: IndexedAccessContext
        ): IndexedAccessResult {
            // Check cache
            const cacheKey = createIndexedAccessCacheKey(objectType, indexType, accessFlags);
            const cached = this.cache.get(cacheKey);
            if (cached !== undefined) {
                return {
                    type: cached,
                    isValid: true,
                };
            }

            // Create initial work item
            const worklist: WorkItem[] = [];
            const rootItem = this.createWorkItem(
                WorkItemKind.Resolve,
                objectType,
                indexType,
                accessFlags,
                0,
                null,
                0
            );
            worklist.push(rootItem);

            // Results map: work item ID -> result type
            const results = new Map<number, Type | undefined>();

            let iterations = 0;

            // Process worklist iteratively
            while (worklist.length > 0 && iterations < this.config.maxIterations) {
                iterations++;

                const item = worklist.pop()!;

                // Check depth limit
                if (item.depth > this.config.maxDepth) {
                    results.set(item.id, context.errorType);
                    this.propagateResult(item, context.errorType, results, worklist, context);
                    continue;
                }

                // Process based on kind
                const workResult = this.processWorkItem(item, context, worklist, results);

                if (workResult.completed) {
                    results.set(item.id, workResult.type);
                    this.propagateResult(item, workResult.type, results, worklist, context);
                }
            }

            // Get final result
            const finalResult = results.get(rootItem.id);

            // Cache result
            if (finalResult !== undefined) {
                this.cache.set(cacheKey, finalResult);
            }

            return {
                type: finalResult,
                isValid: finalResult !== undefined && finalResult !== context.errorType,
            };
        }

        /**
         * Create a new work item
         */
        private createWorkItem(
            kind: WorkItemKind,
            objectType: Type,
            indexType: Type,
            accessFlags: IndexAccessFlags,
            depth: number,
            parentId: number | null,
            parentResultIndex: number
        ): WorkItem {
            return {
                id: this.nextWorkItemId++,
                kind,
                objectType,
                indexType,
                accessFlags,
                depth,
                parentId,
                parentResultIndex,
                results: [],
                expectedResults: 0,
            };
        }

        /**
         * Process a single work item
         */
        private processWorkItem(
            item: WorkItem,
            context: IndexedAccessContext,
            worklist: WorkItem[],
            results: Map<number, Type | undefined>
        ): { completed: boolean; type?: Type } {
            switch (item.kind) {
                case WorkItemKind.Resolve:
                    return this.processResolve(item, context, worklist);

                case WorkItemKind.UnionObject:
                    return this.processUnionObject(item, context, worklist, results);

                case WorkItemKind.UnionIndex:
                    return this.processUnionIndex(item, context, worklist, results);

                case WorkItemKind.Intersection:
                    return this.processIntersection(item, context, worklist, results);

                case WorkItemKind.TupleElement:
                    return this.processTupleElement(item, context);

                case WorkItemKind.MappedType:
                    return this.processMappedType(item, context);

                case WorkItemKind.Property:
                    return this.processProperty(item, context);

                case WorkItemKind.IndexSignature:
                    return this.processIndexSignature(item, context);

                default:
                    return { completed: true, type: context.errorType };
            }
        }

        /**
         * Process a resolve work item - determine the appropriate resolution strategy
         */
        private processResolve(
            item: WorkItem,
            context: IndexedAccessContext,
            worklist: WorkItem[]
        ): { completed: boolean; type?: Type } {
            const { objectType, indexType, accessFlags, depth } = item;

            // Handle never type
            if (objectType.flags & TypeFlags.Never) {
                return { completed: true, type: context.neverType };
            }
            if (indexType.flags & TypeFlags.Never) {
                return { completed: true, type: context.neverType };
            }

            // Handle any type
            if (objectType.flags & TypeFlags.Any) {
                return { completed: true, type: context.anyType };
            }

            // Check for union object type - distribute
            if (objectType.flags & TypeFlags.Union) {
                const newItem = this.createWorkItem(
                    WorkItemKind.UnionObject,
                    objectType,
                    indexType,
                    accessFlags,
                    depth,
                    item.id,
                    0
                );
                worklist.push(newItem);
                return { completed: false };
            }

            // Check for intersection object type
            if (objectType.flags & TypeFlags.Intersection) {
                const newItem = this.createWorkItem(
                    WorkItemKind.Intersection,
                    objectType,
                    indexType,
                    accessFlags,
                    depth,
                    item.id,
                    0
                );
                worklist.push(newItem);
                return { completed: false };
            }

            // Check for union index type - distribute
            if (indexType.flags & TypeFlags.Union) {
                const newItem = this.createWorkItem(
                    WorkItemKind.UnionIndex,
                    objectType,
                    indexType,
                    accessFlags,
                    depth,
                    item.id,
                    0
                );
                worklist.push(newItem);
                return { completed: false };
            }

            // Check for tuple type with numeric index
            if (this.isTupleType(objectType, context) && this.isNumericIndexType(indexType)) {
                const newItem = this.createWorkItem(
                    WorkItemKind.TupleElement,
                    objectType,
                    indexType,
                    accessFlags,
                    depth,
                    item.id,
                    0
                );
                worklist.push(newItem);
                return { completed: false };
            }

            // Check for mapped type
            if (this.isMappedType(objectType, context)) {
                const newItem = this.createWorkItem(
                    WorkItemKind.MappedType,
                    objectType,
                    indexType,
                    accessFlags,
                    depth,
                    item.id,
                    0
                );
                worklist.push(newItem);
                return { completed: false };
            }

            // Check for literal index type - try property access
            if (isTypeUsableAsPropertyName(indexType)) {
                const newItem = this.createWorkItem(
                    WorkItemKind.Property,
                    objectType,
                    indexType,
                    accessFlags,
                    depth,
                    item.id,
                    0
                );
                worklist.push(newItem);
                return { completed: false };
            }

            // Fall back to index signature
            const newItem = this.createWorkItem(
                WorkItemKind.IndexSignature,
                objectType,
                indexType,
                accessFlags,
                depth,
                item.id,
                0
            );
            worklist.push(newItem);
            return { completed: false };
        }

        /**
         * Process union object type - T[K] where T is union
         * Result: (A | B)[K] -> A[K] | B[K] (for reading)
         * Result: (A | B)[K] -> A[K] & B[K] (for writing)
         */
        private processUnionObject(
            item: WorkItem,
            context: IndexedAccessContext,
            worklist: WorkItem[],
            results: Map<number, Type | undefined>
        ): { completed: boolean; type?: Type } {
            const unionType = item.objectType as UnionType;
            const isWriting = (item.accessFlags & IndexAccessFlags.Writing) !== 0;

            // Check if all child results are ready
            if (item.results.length < item.expectedResults) {
                // First time - spawn child work items
                if (item.expectedResults === 0) {
                    item.expectedResults = unionType.types.length;
                    item.results = new Array(unionType.types.length);

                    for (let i = 0; i < unionType.types.length; i++) {
                        const childItem = this.createWorkItem(
                            WorkItemKind.Resolve,
                            unionType.types[i],
                            item.indexType,
                            item.accessFlags,
                            item.depth + 1,
                            item.id,
                            i
                        );
                        worklist.push(childItem);
                    }
                }
                return { completed: false };
            }

            // All results ready - combine
            const validResults = item.results.filter((t): t is Type => t !== undefined);

            if (validResults.length === 0) {
                return { completed: true, type: context.errorType };
            }

            // For writing, intersect results; for reading, union results
            const combinedType = isWriting
                ? context.getIntersectionType(validResults)
                : context.getUnionType(validResults);

            return { completed: true, type: combinedType };
        }

        /**
         * Process union index type - T[K] where K is union
         * Result: T[A | B] -> T[A] | T[B] (for reading)
         * Result: T[A | B] -> T[A] & T[B] (for writing)
         */
        private processUnionIndex(
            item: WorkItem,
            context: IndexedAccessContext,
            worklist: WorkItem[],
            results: Map<number, Type | undefined>
        ): { completed: boolean; type?: Type } {
            const unionType = item.indexType as UnionType;
            const isWriting = (item.accessFlags & IndexAccessFlags.Writing) !== 0;

            // Check if all child results are ready
            if (item.results.length < item.expectedResults) {
                if (item.expectedResults === 0) {
                    item.expectedResults = unionType.types.length;
                    item.results = new Array(unionType.types.length);

                    for (let i = 0; i < unionType.types.length; i++) {
                        const childItem = this.createWorkItem(
                            WorkItemKind.Resolve,
                            item.objectType,
                            unionType.types[i],
                            item.accessFlags,
                            item.depth + 1,
                            item.id,
                            i
                        );
                        worklist.push(childItem);
                    }
                }
                return { completed: false };
            }

            // All results ready - combine
            const validResults = item.results.filter((t): t is Type => t !== undefined);

            if (validResults.length === 0) {
                return { completed: true, type: context.errorType };
            }

            const combinedType = isWriting
                ? context.getIntersectionType(validResults)
                : context.getUnionType(validResults);

            return { completed: true, type: combinedType };
        }

        /**
         * Process intersection object type
         * Result: (A & B)[K] -> A[K] & B[K]
         */
        private processIntersection(
            item: WorkItem,
            context: IndexedAccessContext,
            worklist: WorkItem[],
            results: Map<number, Type | undefined>
        ): { completed: boolean; type?: Type } {
            const intersectionType = item.objectType as IntersectionType;

            // Check if all child results are ready
            if (item.results.length < item.expectedResults) {
                if (item.expectedResults === 0) {
                    item.expectedResults = intersectionType.types.length;
                    item.results = new Array(intersectionType.types.length);

                    for (let i = 0; i < intersectionType.types.length; i++) {
                        const childItem = this.createWorkItem(
                            WorkItemKind.Resolve,
                            intersectionType.types[i],
                            item.indexType,
                            item.accessFlags,
                            item.depth + 1,
                            item.id,
                            i
                        );
                        worklist.push(childItem);
                    }
                }
                return { completed: false };
            }

            // All results ready - intersect
            const validResults = item.results.filter((t): t is Type => t !== undefined);

            if (validResults.length === 0) {
                return { completed: true, type: context.errorType };
            }

            const combinedType = context.getIntersectionType(validResults);
            return { completed: true, type: combinedType };
        }

        /**
         * Process tuple element access
         */
        private processTupleElement(
            item: WorkItem,
            context: IndexedAccessContext
        ): { completed: boolean; type?: Type } {
            const tupleType = item.objectType;
            const indexType = item.indexType;

            // Get tuple element types
            const elementTypes = context.getTupleElementTypes(tupleType);
            if (!elementTypes || elementTypes.length === 0) {
                return { completed: true, type: context.undefinedType };
            }

            // Handle numeric literal index
            if (indexType.flags & TypeFlags.NumberLiteral) {
                const index = (indexType as NumberLiteralType).value;

                // Check bounds
                if (index < 0 || index >= elementTypes.length) {
                    // Check for rest element
                    const restType = context.getTupleRestType(tupleType);
                    if (restType) {
                        return { completed: true, type: restType };
                    }

                    // Out of bounds
                    if (!(item.accessFlags & IndexAccessFlags.NoTupleBoundsCheck)) {
                        return { completed: true, type: context.errorType };
                    }
                    return { completed: true, type: context.undefinedType };
                }

                let elementType = elementTypes[index];

                // Check for optional element
                if (context.isTupleElementOptional(tupleType, index)) {
                    if (item.accessFlags & IndexAccessFlags.IncludeUndefined) {
                        elementType = context.getUnionType([elementType, context.undefinedType]);
                    }
                }

                return { completed: true, type: elementType };
            }

            // Handle number type - union of all elements
            if (indexType.flags & TypeFlags.Number) {
                const allTypes = [...elementTypes];
                const restType = context.getTupleRestType(tupleType);
                if (restType) {
                    allTypes.push(restType);
                }

                if (item.accessFlags & IndexAccessFlags.IncludeUndefined) {
                    allTypes.push(context.undefinedType);
                }

                return { completed: true, type: context.getUnionType(allTypes) };
            }

            return { completed: true, type: context.errorType };
        }

        /**
         * Process mapped type access
         */
        private processMappedType(
            item: WorkItem,
            context: IndexedAccessContext
        ): { completed: boolean; type?: Type } {
            const mappedType = item.objectType;
            const indexType = item.indexType;

            // Get mapped type template
            const templateType = context.getMappedTypeTemplateType(mappedType);
            if (!templateType) {
                return { completed: true, type: context.errorType };
            }

            // Get the type parameter and its constraint
            const typeParameter = context.getMappedTypeTypeParameter(mappedType);
            if (!typeParameter) {
                return { completed: true, type: templateType };
            }

            // Substitute index type for type parameter in template
            const substitutedType = context.substituteTypeParameter(
                templateType,
                typeParameter,
                indexType
            );

            // Check for readonly/optional modifiers
            let resultType = substitutedType;

            if (context.isMappedTypeOptional(mappedType)) {
                if (item.accessFlags & IndexAccessFlags.IncludeUndefined) {
                    resultType = context.getUnionType([resultType, context.undefinedType]);
                }
            }

            return { completed: true, type: resultType };
        }

        /**
         * Process direct property access
         */
        private processProperty(
            item: WorkItem,
            context: IndexedAccessContext
        ): { completed: boolean; type?: Type } {
            const propertyName = getPropertyNameFromLiteralType(item.indexType);
            if (!propertyName) {
                return { completed: true, type: context.errorType };
            }

            // Look up property
            const property = context.getPropertyOfType(item.objectType, propertyName);

            if (property) {
                let propertyType = context.getTypeOfSymbol(property);

                // Check for optional property
                if (property.flags & SymbolFlags.Optional) {
                    if (item.accessFlags & IndexAccessFlags.IncludeUndefined) {
                        propertyType = context.getUnionType([propertyType, context.undefinedType]);
                    }
                }

                return {
                    completed: true,
                    type: propertyType,
                };
            }

            // Property not found - try index signature
            return this.processIndexSignature(item, context);
        }

        /**
         * Process index signature lookup
         */
        private processIndexSignature(
            item: WorkItem,
            context: IndexedAccessContext
        ): { completed: boolean; type?: Type } {
            // Skip index signatures if flag set
            if (item.accessFlags & IndexAccessFlags.NoIndexSignatures) {
                return { completed: true, type: context.errorType };
            }

            // Get applicable index info
            const indexInfo = this.indexSignatureChecker.getApplicableIndexInfo(
                item.objectType,
                item.indexType,
                context.getApparentType,
                context.resolveMembers
            );

            if (indexInfo) {
                let resultType = indexInfo.type;

                // Include undefined for index signature access
                if (item.accessFlags & IndexAccessFlags.IncludeUndefined) {
                    resultType = context.getUnionType([resultType, context.undefinedType]);
                }

                return {
                    completed: true,
                    type: resultType,
                };
            }

            // No applicable index signature
            return { completed: true, type: context.errorType };
        }

        /**
         * Propagate a result to parent work item
         */
        private propagateResult(
            item: WorkItem,
            result: Type | undefined,
            results: Map<number, Type | undefined>,
            worklist: WorkItem[],
            context: IndexedAccessContext
        ): void {
            if (item.parentId === null) {
                return;
            }

            // Find parent in worklist or results
            const parent = worklist.find(w => w.id === item.parentId);
            if (parent) {
                parent.results[item.parentResultIndex] = result;

                // Check if parent is ready to complete
                const readyCount = parent.results.filter(r => r !== undefined).length;
                if (readyCount >= parent.expectedResults) {
                    // Re-add parent to worklist to complete
                    worklist.push(parent);
                }
            }
        }

        /**
         * Check if type is a tuple type
         */
        private isTupleType(type: Type, context: IndexedAccessContext): boolean {
            return context.isTupleType(type);
        }

        /**
         * Check if type is a numeric index type
         */
        private isNumericIndexType(type: Type): boolean {
            return !!(
                (type.flags & TypeFlags.Number) ||
                (type.flags & TypeFlags.NumberLiteral)
            );
        }

        /**
         * Check if type is a mapped type
         */
        private isMappedType(type: Type, context: IndexedAccessContext): boolean {
            return context.isMappedType(type);
        }

        /**
         * Check bracket notation property access (element access expression)
         */
        public checkElementAccessExpression(
            objectType: Type,
            indexExpression: Expression,
            context: IndexedAccessContext
        ): IndexedAccessResult {
            // Get the type of the index expression
            const indexType = context.getTypeOfNode(indexExpression);

            // Determine access flags
            let accessFlags = IndexAccessFlags.None;

            // Check if in assignment context
            if (context.isAssignmentTarget(indexExpression)) {
                accessFlags |= IndexAccessFlags.Writing;
            }

            // Check for expression position
            accessFlags |= IndexAccessFlags.ExpressionPosition;

            // Include undefined for noUncheckedIndexedAccess
            if (context.compilerOptions.noUncheckedIndexedAccess) {
                accessFlags |= IndexAccessFlags.IncludeUndefined;
            }

            return this.resolve(objectType, indexType, accessFlags, context);
        }

        /**
         * Get the simplified form of an indexed access type
         */
        public getSimplifiedIndexedAccessType(
            type: Type,
            writing: boolean,
            context: IndexedAccessContext
        ): Type {
            if (!isIndexedAccessType(type)) {
                return type;
            }

            const indexedAccessType = type as unknown as DeferredIndexedAccessType;

            // Check cache
            const cached = writing
                ? indexedAccessType.simplifiedForWriting
                : indexedAccessType.simplifiedForReading;

            if (cached) {
                return cached;
            }

            const accessFlags = writing
                ? IndexAccessFlags.Writing
                : IndexAccessFlags.None;

            const result = this.resolve(
                indexedAccessType.objectType,
                indexedAccessType.indexType,
                accessFlags,
                context
            );

            // Cache result
            if (result.type) {
                if (writing) {
                    indexedAccessType.simplifiedForWriting = result.type;
                } else {
                    indexedAccessType.simplifiedForReading = result.type;
                }
            }

            return result.type || context.errorType;
        }

        /**
         * Get constraint of indexed access type
         */
        public getConstraintOfIndexedAccess(
            objectType: Type,
            indexType: Type,
            context: IndexedAccessContext
        ): Type | undefined {
            // Get constraint of object type
            const objectConstraint = context.getConstraintOfType(objectType);

            // Get constraint of index type
            const indexConstraint = context.getConstraintOfType(indexType);

            if (objectConstraint || indexConstraint) {
                const result = this.resolve(
                    objectConstraint || objectType,
                    indexConstraint || indexType,
                    IndexAccessFlags.None,
                    context
                );
                return result.type;
            }

            return undefined;
        }

        /**
         * Clear the resolution cache
         */
        public clearCache(): void {
            this.cache.clear();
            this.indexSignatureChecker.clearCache();
        }
    }

    /**
     * Context provided to the indexed access solver
     */
    export interface IndexedAccessContext {
        // Built-in types
        readonly anyType: Type;
        readonly errorType: Type;
        readonly neverType: Type;
        readonly undefinedType: Type;
        readonly stringType: Type;
        readonly numberType: Type;

        // Compiler options
        readonly compilerOptions: {
            readonly noUncheckedIndexedAccess?: boolean;
        };

        // Type operations
        getUnionType(types: Type[]): Type;
        getIntersectionType(types: Type[]): Type;
        getApparentType(type: Type): Type;
        resolveMembers(type: Type): void;

        // Property access
        getPropertyOfType(type: Type, name: __String): Symbol | undefined;
        getTypeOfSymbol(symbol: Symbol): Type;

        // Tuple operations
        isTupleType(type: Type): boolean;
        getTupleElementTypes(type: Type): Type[] | undefined;
        getTupleRestType(type: Type): Type | undefined;
        isTupleElementOptional(type: Type, index: number): boolean;

        // Mapped type operations
        isMappedType(type: Type): boolean;
        getMappedTypeTemplateType(type: Type): Type | undefined;
        getMappedTypeTypeParameter(type: Type): Type | undefined;
        isMappedTypeOptional(type: Type): boolean;
        substituteTypeParameter(type: Type, typeParameter: Type, substitute: Type): Type;

        // Expression context
        getTypeOfNode(node: Node): Type;
        isAssignmentTarget(node: Node): boolean;

        // Constraints
        getConstraintOfType(type: Type): Type | undefined;
    }

    /**
     * Create a minimal indexed access context for testing
     */
    export function createMinimalIndexedAccessContext(
        anyType: Type,
        errorType: Type,
        neverType: Type,
        undefinedType: Type,
        stringType: Type,
        numberType: Type
    ): IndexedAccessContext {
        return {
            anyType,
            errorType,
            neverType,
            undefinedType,
            stringType,
            numberType,
            compilerOptions: {},
            getUnionType: (types) => types[0] || neverType,
            getIntersectionType: (types) => types[0] || neverType,
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
    }
}
