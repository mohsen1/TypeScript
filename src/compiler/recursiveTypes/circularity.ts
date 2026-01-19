/**
 * Recursive Types - Circularity Detection
 *
 * This module implements detection and handling of circular type references:
 * - Direct circularity (type A = A)
 * - Indirect circularity (type A = B, type B = A)
 * - Circular constraints (T extends T)
 */

/* @internal */
namespace ts.recursiveTypes {
    /**
     * Create a resolution stack entry
     */
    export function createStackEntry(
        symbol: Symbol | undefined,
        node: Node | undefined,
        depth: number
    ): ResolutionStackEntry {
        const key = getResolutionKey(symbol, node);
        return { symbol, node, key, depth };
    }

    /**
     * Get a unique key for a resolution target
     */
    export function getResolutionKey(
        symbol: Symbol | undefined,
        node: Node | undefined
    ): string {
        if (symbol) {
            const symbolId = getSymbolId(symbol);
            return `sym:${symbolId}`;
        }
        if (node) {
            const nodeId = getNodeId(node);
            return `node:${nodeId}`;
        }
        return `unknown:${Date.now()}`;
    }

    /**
     * Get a numeric ID for a symbol (uses internal ID if available)
     */
    function getSymbolId(symbol: Symbol): number {
        return (symbol as any).id ?? 0;
    }

    /**
     * Get a numeric ID for a node (uses internal ID if available)
     */
    function getNodeId(node: Node): number {
        return (node as any).id ?? 0;
    }

    /**
     * Check if a symbol is currently being resolved (circular reference)
     */
    export function isSymbolBeingResolved(
        symbol: Symbol,
        context: RecursiveTypeContext
    ): boolean {
        const key = getResolutionKey(symbol, undefined);
        return context.resolutionStack.some(entry => entry.key === key);
    }

    /**
     * Check if a node is currently being resolved
     */
    export function isNodeBeingResolved(
        node: Node,
        context: RecursiveTypeContext
    ): boolean {
        const key = getResolutionKey(undefined, node);
        return context.resolutionStack.some(entry => entry.key === key);
    }

    /**
     * Push a new entry onto the resolution stack
     */
    export function pushResolution(
        symbol: Symbol | undefined,
        node: Node | undefined,
        context: RecursiveTypeContext
    ): ResolutionStackEntry {
        const depth = context.resolutionStack.length;
        const entry = createStackEntry(symbol, node, depth);
        context.resolutionStack.push(entry);
        return entry;
    }

    /**
     * Pop an entry from the resolution stack
     */
    export function popResolution(
        context: RecursiveTypeContext
    ): ResolutionStackEntry | undefined {
        return context.resolutionStack.pop();
    }

    /**
     * Check for circularity before resolving a type
     */
    export function checkCircularity(
        symbol: Symbol | undefined,
        node: Node | undefined,
        context: RecursiveTypeContext
    ): CircularityCheckResult {
        const key = getResolutionKey(symbol, node);

        // Check if we're already resolving this
        const existingIndex = context.resolutionStack.findIndex(
            entry => entry.key === key
        );

        if (existingIndex >= 0) {
            // Found a cycle - build the path
            const cyclePath = context.resolutionStack
                .slice(existingIndex)
                .map(entry => entry.key);
            cyclePath.push(key); // Close the cycle

            return {
                isCircular: true,
                cyclePath,
                cycleDepth: existingIndex,
            };
        }

        return { isCircular: false };
    }

    /**
     * Detect if a type alias is directly self-referential
     *
     * type A = A; // Direct
     * type A = { x: A }; // Not direct (through object type)
     */
    export function isDirectlyRecursive(
        aliasSymbol: Symbol,
        aliasType: Type,
        context: RecursiveTypeContext
    ): boolean {
        // Check if the aliased type directly refers to the same symbol
        if (aliasType.aliasSymbol === aliasSymbol) {
            return true;
        }

        // Check type references
        if (aliasType.flags & TypeFlags.Object) {
            const objectType = aliasType as ObjectType;
            if (objectType.objectFlags & ObjectFlags.Reference) {
                const typeRef = aliasType as TypeReference;
                if (typeRef.target && (typeRef.target as any).symbol === aliasSymbol) {
                    return true;
                }
            }
        }

        return false;
    }

    /**
     * Find mutually recursive type aliases
     *
     * type A = B;
     * type B = A;
     */
    export function findMutualRecursion(
        startSymbol: Symbol,
        getReferencedSymbols: (symbol: Symbol) => Symbol[],
        maxDepth: number = 10
    ): Symbol[] {
        const visited = new Set<Symbol>();
        const path: Symbol[] = [];

        function dfs(current: Symbol, depth: number): Symbol[] | undefined {
            if (depth > maxDepth) {
                return undefined;
            }

            if (visited.has(current)) {
                // Check if we found a cycle back to start
                if (current === startSymbol) {
                    return [...path];
                }
                return undefined;
            }

            visited.add(current);
            path.push(current);

            const references = getReferencedSymbols(current);
            for (const ref of references) {
                const result = dfs(ref, depth + 1);
                if (result) {
                    return result;
                }
            }

            path.pop();
            return undefined;
        }

        const result = dfs(startSymbol, 0);
        return result ?? [];
    }

    /**
     * Check if a constraint is self-referential
     *
     * interface A<T extends T> // Self-referential constraint
     * interface A<T extends A<T>> // Recursive but valid
     */
    export function isConstraintSelfReferential(
        typeParameter: TypeParameter,
        constraint: Type | undefined
    ): boolean {
        if (!constraint) {
            return false;
        }

        // Direct self-reference
        if (constraint === typeParameter) {
            return true;
        }

        // Check in union/intersection
        if (constraint.flags & TypeFlags.UnionOrIntersection) {
            const types = (constraint as UnionOrIntersectionType).types;
            return types.some(t => t === typeParameter);
        }

        return false;
    }

    /**
     * Break a circular reference by returning a marker type
     */
    export function breakCircularReference(
        context: RecursiveTypeContext,
        errorMessage?: string
    ): Type {
        // Return circular type marker
        return context.circularType;
    }

    /**
     * Create a circular type diagnostic
     */
    export function createCircularityError(
        cyclePath: string[],
        node?: Node
    ): string {
        if (cyclePath.length === 1) {
            return `Type alias '${cyclePath[0]}' circularly references itself.`;
        }

        const pathString = cyclePath.join(" -> ");
        return `Type alias contains circular reference: ${pathString}`;
    }

    /**
     * Check if a type safely contains recursive references
     * (wrapped in objects, functions, arrays - not direct)
     */
    export function isSafelyRecursive(
        checkType: Type,
        recursiveSymbol: Symbol,
        depth: number = 0,
        maxDepth: number = 5
    ): boolean {
        if (depth > maxDepth) {
            return true; // Assume safe at depth limit
        }

        // Check if this is the recursive symbol directly
        if (checkType.aliasSymbol === recursiveSymbol) {
            return false; // Not safe - direct reference
        }

        if (checkType.flags & TypeFlags.Object) {
            const objectType = checkType as ObjectType;

            // Object types are safe wrappers
            if (objectType.objectFlags & ObjectFlags.Anonymous) {
                // Check members
                const members = objectType.symbol?.members;
                if (members) {
                    for (const member of members.values()) {
                        const memberType = (member as any).type;
                        if (memberType && memberType.aliasSymbol === recursiveSymbol) {
                            return true; // Safe - wrapped in object property
                        }
                    }
                }
                return true;
            }

            // Array/tuple types are safe wrappers
            if (objectType.objectFlags & ObjectFlags.Reference) {
                const typeRef = checkType as TypeReference;
                const target = typeRef.target;
                if (target && ((target as any).symbol?.name === "Array" ||
                    (target.objectFlags & ObjectFlags.Tuple))) {
                    return true;
                }
            }
        }

        // Union/intersection - check each constituent
        if (checkType.flags & TypeFlags.UnionOrIntersection) {
            const types = (checkType as UnionOrIntersectionType).types;
            return types.every(t =>
                isSafelyRecursive(t, recursiveSymbol, depth + 1, maxDepth)
            );
        }

        return true; // Default to safe
    }

    /**
     * Resolve type with circularity protection
     */
    export function resolveWithCircularityCheck<T>(
        symbol: Symbol | undefined,
        node: Node | undefined,
        resolver: () => T,
        onCircular: () => T,
        context: RecursiveTypeContext
    ): T {
        // Check for existing circularity
        const circularityResult = checkCircularity(symbol, node, context);
        if (circularityResult.isCircular) {
            return onCircular();
        }

        // Check depth limit
        if (context.resolutionStack.length >= context.config.maxDepth) {
            return onCircular();
        }

        // Push onto stack and resolve
        pushResolution(symbol, node, context);
        try {
            return resolver();
        } finally {
            popResolution(context);
        }
    }

    /**
     * Track resolution for debugging
     */
    export function getResolutionStackTrace(
        context: RecursiveTypeContext
    ): string[] {
        return context.resolutionStack.map(entry => {
            const symbolName = entry.symbol?.name ?? "(unknown)";
            return `[${entry.depth}] ${entry.key} (${symbolName})`;
        });
    }

    /**
     * Clear the resolution stack (for error recovery)
     */
    export function clearResolutionStack(context: RecursiveTypeContext): void {
        context.resolutionStack.length = 0;
    }

    /**
     * Get current resolution depth
     */
    export function getCurrentDepth(context: RecursiveTypeContext): number {
        return context.resolutionStack.length;
    }

    /**
     * Check if we've exceeded the maximum depth
     */
    export function isMaxDepthExceeded(context: RecursiveTypeContext): boolean {
        return context.resolutionStack.length >= context.config.maxDepth;
    }
}
