/**
 * Iterative Flow Analysis
 *
 * Performs control flow analysis using an iterative work-list algorithm
 * instead of recursive descent. This provides better stack safety for
 * deeply nested control flows.
 */

/* @internal */
namespace ts.thinChecker {
    /**
     * State of flow analysis for a particular flow node
     */
    export const enum FlowState {
        Unvisited = 0,
        InProgress = 1,
        Complete = 2,
    }

    /**
     * Work item for the flow analysis queue
     */
    export interface FlowWorkItem {
        flowNode: FlowNode;
        reference: Node;
        declaredType: Type;
        state: FlowState;
    }

    /**
     * Result of flow analysis
     */
    export interface FlowAnalysisResult {
        type: Type;
        isReachable: boolean;
        narrowedFrom?: Type;
    }

    /**
     * Flow analyzer interface
     */
    export interface FlowAnalyzer {
        /**
         * Analyze flow to determine type at a reference
         */
        analyzeFlow(reference: Node, flowNode: FlowNode, declaredType: Type): FlowAnalysisResult;

        /**
         * Check if a flow node is reachable
         */
        isReachable(flowNode: FlowNode): boolean;

        /**
         * Clear the analysis cache
         */
        clearCache(): void;
    }

    /**
     * Create a flow analyzer
     */
    export function createFlowAnalyzer(checker: TypeChecker): FlowAnalyzer {
        // Cache for flow analysis results
        const flowTypeCache = new Map<string, FlowAnalysisResult>();

        // Cache for reachability
        const reachableCache = new Map<number, boolean>();

        return {
            analyzeFlow,
            isReachable,
            clearCache,
        };

        /**
         * Analyze flow iteratively to determine the type at a reference
         */
        function analyzeFlow(
            reference: Node,
            flowNode: FlowNode,
            declaredType: Type
        ): FlowAnalysisResult {
            const cacheKey = `${getNodeId(reference)}-${getFlowNodeId(flowNode)}`;
            const cached = flowTypeCache.get(cacheKey);
            if (cached) {
                return cached;
            }

            // Use iterative work-list algorithm
            const result = analyzeFlowIteratively(reference, flowNode, declaredType);
            flowTypeCache.set(cacheKey, result);
            return result;
        }

        /**
         * Iterative flow analysis using a work queue
         */
        function analyzeFlowIteratively(
            reference: Node,
            startFlow: FlowNode,
            declaredType: Type
        ): FlowAnalysisResult {
            // Work queue for flow nodes to process
            const workQueue: FlowNode[] = [startFlow];
            // Map from flow node id to computed type
            const typeAtFlow = new Map<number, Type>();
            // Set of visited nodes
            const visited = new Set<number>();
            // Current result
            let resultType = declaredType;

            while (workQueue.length > 0) {
                const flow = workQueue.shift()!;
                const flowId = getFlowNodeId(flow);

                if (visited.has(flowId)) {
                    continue;
                }
                visited.add(flowId);

                const flowResult = processFlowNode(flow, reference, declaredType, workQueue, typeAtFlow);
                if (flowResult) {
                    resultType = flowResult;
                    typeAtFlow.set(flowId, flowResult);
                }
            }

            return {
                type: resultType,
                isReachable: isReachable(startFlow),
            };
        }

        /**
         * Process a single flow node
         */
        function processFlowNode(
            flow: FlowNode,
            reference: Node,
            declaredType: Type,
            workQueue: FlowNode[],
            typeAtFlow: Map<number, Type>
        ): Type | undefined {
            const flags = flow.flags;

            // Start flow - return declared type
            if (flags & FlowFlags.Start) {
                return declaredType;
            }

            // Assignment flow
            if (flags & FlowFlags.Assignment) {
                const assignment = flow as FlowAssignment;
                // Add antecedent to work queue
                if (assignment.antecedent) {
                    workQueue.push(assignment.antecedent);
                }

                // Check if this assignment affects the reference
                if (isMatchingReference(reference, assignment.node)) {
                    // Get type from assignment
                    return getAssignmentType(assignment.node);
                }

                // Return type from antecedent
                const antecedentType = typeAtFlow.get(getFlowNodeId(assignment.antecedent));
                return antecedentType || declaredType;
            }

            // Condition flow
            if (flags & FlowFlags.TrueCondition || flags & FlowFlags.FalseCondition) {
                const condition = flow as FlowCondition;
                // Add antecedent to work queue
                if (condition.antecedent) {
                    workQueue.push(condition.antecedent);
                }

                // Get type from antecedent and narrow based on condition
                const antecedentType = typeAtFlow.get(getFlowNodeId(condition.antecedent)) || declaredType;
                return narrowType(antecedentType, condition.node, !!(flags & FlowFlags.TrueCondition));
            }

            // Branch flow (if/switch)
            if (flags & FlowFlags.BranchLabel) {
                const branch = flow as FlowLabel;
                if (branch.antecedents) {
                    // Add all antecedents to work queue
                    for (const antecedent of branch.antecedents) {
                        workQueue.push(antecedent);
                    }

                    // Collect types from all antecedents and create union
                    const types: Type[] = [];
                    for (const antecedent of branch.antecedents) {
                        const type = typeAtFlow.get(getFlowNodeId(antecedent));
                        if (type) {
                            types.push(type);
                        }
                    }

                    if (types.length > 0) {
                        return checker.getUnionType(types);
                    }
                }
                return declaredType;
            }

            // Loop flow
            if (flags & FlowFlags.LoopLabel) {
                const loop = flow as FlowLabel;
                if (loop.antecedents) {
                    for (const antecedent of loop.antecedents) {
                        workQueue.push(antecedent);
                    }
                }
                return declaredType;
            }

            // Unreachable
            if (flags & FlowFlags.Unreachable) {
                return checker.getNeverType();
            }

            return undefined;
        }

        /**
         * Check if a reference matches a node
         */
        function isMatchingReference(reference: Node, node: Node): boolean {
            if (isIdentifier(reference) && isIdentifier(node)) {
                return reference.escapedText === node.escapedText;
            }
            // For more complex references, would need symbol resolution
            return false;
        }

        /**
         * Get the type from an assignment
         */
        function getAssignmentType(node: Node): Type {
            if (isBinaryExpression(node) && node.operatorToken.kind === SyntaxKind.EqualsToken) {
                return checker.getTypeAtLocation(node.right);
            }
            if (isVariableDeclaration(node) && node.initializer) {
                return checker.getTypeAtLocation(node.initializer);
            }
            return checker.getTypeAtLocation(node);
        }

        /**
         * Narrow type based on condition
         */
        function narrowType(type: Type, condition: Expression, assumeTrue: boolean): Type {
            // Simplified narrowing - full implementation would handle
            // typeof, instanceof, type predicates, etc.
            return type;
        }

        /**
         * Check if a flow node is reachable
         */
        function isReachable(flowNode: FlowNode): boolean {
            const flowId = getFlowNodeId(flowNode);
            const cached = reachableCache.get(flowId);
            if (cached !== undefined) {
                return cached;
            }

            const result = computeReachability(flowNode);
            reachableCache.set(flowId, result);
            return result;
        }

        /**
         * Compute reachability iteratively
         */
        function computeReachability(startFlow: FlowNode): boolean {
            const workQueue: FlowNode[] = [startFlow];
            const visited = new Set<number>();

            while (workQueue.length > 0) {
                const flow = workQueue.shift()!;
                const flowId = getFlowNodeId(flow);

                if (visited.has(flowId)) {
                    continue;
                }
                visited.add(flowId);

                const flags = flow.flags;

                if (flags & FlowFlags.Unreachable) {
                    return false;
                }

                if (flags & FlowFlags.Start) {
                    return true;
                }

                // Add antecedents to check
                if ((flow as FlowLabel).antecedents) {
                    for (const antecedent of (flow as FlowLabel).antecedents!) {
                        workQueue.push(antecedent);
                    }
                } else if ((flow as FlowCondition).antecedent) {
                    workQueue.push((flow as FlowCondition).antecedent);
                }
            }

            return true;
        }

        /**
         * Get flow node id
         */
        function getFlowNodeId(flow: FlowNode): number {
            if (!flow.id) {
                flow.id = nextFlowId++;
            }
            return flow.id;
        }

        let nextFlowId = 1;

        function clearCache(): void {
            flowTypeCache.clear();
            reachableCache.clear();
        }
    }
}
