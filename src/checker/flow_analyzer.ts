/**
 * Iterative worklist-based flow analyzer for control flow analysis.
 *
 * This module implements an iterative (non-recursive) approach to flow analysis
 * to avoid stack overflow on deeply nested code structures.
 */

namespace ts {
    /**
     * Configuration for flow analysis with safety limits.
     */
    export interface FlowAnalyzerConfig {
        /** Maximum number of iterations before giving up (prevents infinite loops) */
        maxIterations: number;
        /** Maximum worklist size before truncating (memory safety) */
        maxWorklistSize: number;
        /** Enable caching of flow results */
        enableCaching: boolean;
    }

    /**
     * Default configuration with reasonable limits.
     */
    export const DEFAULT_FLOW_ANALYZER_CONFIG: FlowAnalyzerConfig = {
        maxIterations: 100000,
        maxWorklistSize: 50000,
        enableCaching: true,
    };

    /**
     * Represents the state of a flow node being processed.
     */
    export const enum WorkItemState {
        /** Initial state - needs processing */
        Pending = 0,
        /** Currently being processed */
        InProgress = 1,
        /** Processing complete */
        Complete = 2,
    }

    /**
     * A work item in the worklist for iterative processing.
     */
    export interface WorkItem {
        /** The flow node to process */
        flowNode: FlowNode;
        /** Current processing state */
        state: WorkItemState;
        /** Index of parent work item (for branch handling) */
        parentIndex: number;
        /** Computed type result (set when complete) */
        result?: FlowType;
        /** For branch labels, tracks which antecedents have been processed */
        processedAntecedents?: number;
        /** Collected types from antecedents (for branch/loop labels) */
        antecedentTypes?: FlowType[];
    }

    /**
     * Result of flow analysis.
     */
    export interface FlowAnalysisResult {
        /** The computed flow type */
        type: FlowType | undefined;
        /** Whether the analysis hit iteration limits */
        hitLimit: boolean;
        /** Whether the analysis was incomplete (e.g., in a loop) */
        incomplete: boolean;
        /** Number of iterations performed */
        iterations: number;
    }

    /**
     * Callbacks for type operations during flow analysis.
     * These allow the analyzer to interact with the type system without
     * direct dependencies.
     */
    export interface FlowTypeCallbacks {
        /** Get the initial type for a reference */
        getInitialType(): Type;
        /** Get the declared type for a reference */
        getDeclaredType(): Type;
        /** Handle flow assignment */
        getTypeAtAssignment(flow: FlowAssignment): FlowType | undefined;
        /** Handle flow call (assertion) */
        getTypeAtCall(flow: FlowCall, antecedentType: FlowType): FlowType | undefined;
        /** Handle flow condition */
        getTypeAtCondition(flow: FlowCondition, antecedentType: FlowType): FlowType;
        /** Handle switch clause */
        getTypeAtSwitchClause(flow: FlowSwitchClause, antecedentType: FlowType): FlowType;
        /** Handle array mutation */
        getTypeAtArrayMutation(flow: FlowArrayMutation, antecedentType: FlowType): FlowType | undefined;
        /** Create union type from multiple types */
        createUnionType(types: FlowType[], subtypeReduction: boolean): FlowType;
        /** Check if a type is incomplete */
        isIncomplete(type: FlowType): boolean;
        /** Create an incomplete flow type */
        createFlowType(type: Type, incomplete: boolean): FlowType;
        /** Get the actual Type from a FlowType */
        getTypeFromFlowType(flowType: FlowType): Type;
        /** Get base type of literal type */
        getBaseTypeOfLiteralType(type: Type): Type;
        /** Error type for analysis failures */
        getErrorType(): Type;
        /** Convert auto type to any */
        convertAutoToAny(type: Type): Type;
        /** Check if flow node is reachable */
        isReachableFlowNode(flow: FlowNode): boolean;
    }

    /**
     * Iterative worklist-based flow analyzer.
     *
     * Uses an explicit stack/worklist instead of recursion to handle
     * deeply nested control flow without stack overflow.
     */
    export class FlowAnalyzer {
        private readonly config: FlowAnalyzerConfig;
        private readonly callbacks: FlowTypeCallbacks;

        /** Cache for computed flow types (keyed by flow node id) */
        private readonly flowTypeCache: Map<number, FlowType> = new Map();

        /** Worklist of nodes to process */
        private worklist: WorkItem[] = [];

        /** Track which shared flow nodes we've visited in this analysis */
        private sharedFlowNodes: FlowNode[] = [];
        private sharedFlowTypes: FlowType[] = [];
        private sharedFlowCount: number = 0;

        /** Current iteration count */
        private iterations: number = 0;

        /** Flag indicating if we hit limits */
        private hitLimit: boolean = false;

        /** Flow container for nested function handling */
        private flowContainer?: Node;

        /** Reference node being analyzed */
        private reference?: Node;

        constructor(callbacks: FlowTypeCallbacks, config: FlowAnalyzerConfig = DEFAULT_FLOW_ANALYZER_CONFIG) {
            this.callbacks = callbacks;
            this.config = config;
        }

        /**
         * Reset the analyzer state for a new analysis.
         */
        public reset(): void {
            this.flowTypeCache.clear();
            this.worklist = [];
            this.sharedFlowNodes = [];
            this.sharedFlowTypes = [];
            this.sharedFlowCount = 0;
            this.iterations = 0;
            this.hitLimit = false;
            this.flowContainer = undefined;
            this.reference = undefined;
        }

        /**
         * Analyze flow starting from the given flow node.
         * Uses iterative worklist-based processing instead of recursion.
         */
        public analyzeFlow(
            flowNode: FlowNode | undefined,
            reference: Node,
            flowContainer?: Node
        ): FlowAnalysisResult {
            this.reset();

            if (!flowNode) {
                return {
                    type: this.callbacks.getDeclaredType(),
                    hitLimit: false,
                    incomplete: false,
                    iterations: 0,
                };
            }

            this.reference = reference;
            this.flowContainer = flowContainer;

            // Push initial work item
            this.pushWorkItem(flowNode, -1);

            // Process worklist iteratively
            const result = this.processWorklist();

            return {
                type: result,
                hitLimit: this.hitLimit,
                incomplete: false,
                iterations: this.iterations,
            };
        }

        /**
         * Push a new work item onto the worklist.
         */
        private pushWorkItem(flowNode: FlowNode, parentIndex: number): number {
            if (this.worklist.length >= this.config.maxWorklistSize) {
                this.hitLimit = true;
                return -1;
            }

            const index = this.worklist.length;
            this.worklist.push({
                flowNode,
                state: WorkItemState.Pending,
                parentIndex,
            });
            return index;
        }

        /**
         * Main worklist processing loop.
         * Returns the final computed type.
         */
        private processWorklist(): FlowType | undefined {
            while (this.worklist.length > 0) {
                this.iterations++;

                if (this.iterations > this.config.maxIterations) {
                    this.hitLimit = true;
                    return this.callbacks.getErrorType();
                }

                // Get the last item (LIFO order for depth-first processing)
                const currentIndex = this.worklist.length - 1;
                const workItem = this.worklist[currentIndex];

                if (workItem.state === WorkItemState.Complete) {
                    // This item is done, propagate result to parent
                    this.worklist.pop();

                    if (workItem.parentIndex >= 0 && workItem.result !== undefined) {
                        this.propagateResultToParent(workItem.parentIndex, workItem.result);
                    }
                    continue;
                }

                // Process this work item
                const continueProcessing = this.processWorkItem(workItem, currentIndex);

                if (!continueProcessing) {
                    // Work item is complete
                    if (workItem.parentIndex < 0 && workItem.result !== undefined) {
                        // This is the root item, return the result
                        return workItem.result;
                    }
                }
            }

            return undefined;
        }

        /**
         * Process a single work item.
         * Returns true if more processing is needed, false if complete.
         */
        private processWorkItem(workItem: WorkItem, currentIndex: number): boolean {
            const flow = workItem.flowNode;
            const flags = flow.flags;

            // Check for shared flow node cache
            if (flags & FlowFlags.Shared) {
                for (let i = 0; i < this.sharedFlowCount; i++) {
                    if (this.sharedFlowNodes[i] === flow) {
                        workItem.result = this.sharedFlowTypes[i];
                        workItem.state = WorkItemState.Complete;
                        return false;
                    }
                }
            }

            if (workItem.state === WorkItemState.Pending) {
                workItem.state = WorkItemState.InProgress;
            }

            let result: FlowType | undefined;
            let needsMoreProcessing = false;

            if (flags & FlowFlags.Assignment) {
                const assignmentResult = this.processAssignment(flow as FlowAssignment, workItem, currentIndex);
                if (assignmentResult.needsAntecedent) {
                    needsMoreProcessing = true;
                }
                else {
                    result = assignmentResult.result;
                }
            }
            else if (flags & FlowFlags.Call) {
                const callResult = this.processCall(flow as FlowCall, workItem, currentIndex);
                if (callResult.needsAntecedent) {
                    needsMoreProcessing = true;
                }
                else {
                    result = callResult.result;
                }
            }
            else if (flags & FlowFlags.Condition) {
                const condResult = this.processCondition(flow as FlowCondition, workItem, currentIndex);
                if (condResult.needsAntecedent) {
                    needsMoreProcessing = true;
                }
                else {
                    result = condResult.result;
                }
            }
            else if (flags & FlowFlags.SwitchClause) {
                const switchResult = this.processSwitchClause(flow as FlowSwitchClause, workItem, currentIndex);
                if (switchResult.needsAntecedent) {
                    needsMoreProcessing = true;
                }
                else {
                    result = switchResult.result;
                }
            }
            else if (flags & FlowFlags.Label) {
                const labelResult = this.processLabel(flow as FlowLabel, workItem, currentIndex);
                if (labelResult.needsAntecedent) {
                    needsMoreProcessing = true;
                }
                else {
                    result = labelResult.result;
                }
            }
            else if (flags & FlowFlags.ArrayMutation) {
                const mutationResult = this.processArrayMutation(flow as FlowArrayMutation, workItem, currentIndex);
                if (mutationResult.needsAntecedent) {
                    needsMoreProcessing = true;
                }
                else {
                    result = mutationResult.result;
                }
            }
            else if (flags & FlowFlags.ReduceLabel) {
                const reduceResult = this.processReduceLabel(flow as FlowReduceLabel, workItem, currentIndex);
                if (reduceResult.needsAntecedent) {
                    needsMoreProcessing = true;
                }
                else {
                    result = reduceResult.result;
                }
            }
            else if (flags & FlowFlags.Start) {
                result = this.processStart(flow as FlowStart);
            }
            else {
                // Unreachable code
                result = this.callbacks.convertAutoToAny(this.callbacks.getDeclaredType());
            }

            if (!needsMoreProcessing) {
                // Cache result for shared nodes
                if (result !== undefined && (flags & FlowFlags.Shared)) {
                    this.sharedFlowNodes[this.sharedFlowCount] = flow;
                    this.sharedFlowTypes[this.sharedFlowCount] = result;
                    this.sharedFlowCount++;
                }

                workItem.result = result;
                workItem.state = WorkItemState.Complete;
            }

            return needsMoreProcessing;
        }

        /**
         * Propagate a result from a child work item to its parent.
         */
        private propagateResultToParent(parentIndex: number, childResult: FlowType): void {
            const parent = this.worklist[parentIndex];
            if (!parent) return;

            const parentFlow = parent.flowNode;

            // Handle different parent types that need to collect child results
            if (parentFlow.flags & FlowFlags.Label) {
                // Parent is a label, collect antecedent types
                if (!parent.antecedentTypes) {
                    parent.antecedentTypes = [];
                }
                parent.antecedentTypes.push(childResult);
                parent.processedAntecedents = (parent.processedAntecedents || 0) + 1;
            }
            else {
                // For other parents, this is the antecedent result
                parent.antecedentTypes = [childResult];
                parent.processedAntecedents = 1;
            }
        }

        /**
         * Process a flow assignment node.
         */
        private processAssignment(
            flow: FlowAssignment,
            workItem: WorkItem,
            currentIndex: number
        ): { result?: FlowType; needsAntecedent: boolean } {
            const type = this.callbacks.getTypeAtAssignment(flow);

            if (type !== undefined) {
                return { result: type, needsAntecedent: false };
            }

            // Need to process antecedent
            if (!workItem.antecedentTypes) {
                this.pushWorkItem(flow.antecedent, currentIndex);
                return { needsAntecedent: true };
            }

            // Antecedent processed, but no type from assignment means pass through
            return { result: workItem.antecedentTypes[0], needsAntecedent: false };
        }

        /**
         * Process a flow call (assertion) node.
         */
        private processCall(
            flow: FlowCall,
            workItem: WorkItem,
            currentIndex: number
        ): { result?: FlowType; needsAntecedent: boolean } {
            // Need antecedent type first
            if (!workItem.antecedentTypes) {
                this.pushWorkItem(flow.antecedent, currentIndex);
                return { needsAntecedent: true };
            }

            const antecedentType = workItem.antecedentTypes[0];
            const type = this.callbacks.getTypeAtCall(flow, antecedentType);

            if (type !== undefined) {
                return { result: type, needsAntecedent: false };
            }

            return { result: antecedentType, needsAntecedent: false };
        }

        /**
         * Process a flow condition node.
         */
        private processCondition(
            flow: FlowCondition,
            workItem: WorkItem,
            currentIndex: number
        ): { result?: FlowType; needsAntecedent: boolean } {
            // Need antecedent type first
            if (!workItem.antecedentTypes) {
                this.pushWorkItem(flow.antecedent, currentIndex);
                return { needsAntecedent: true };
            }

            const antecedentType = workItem.antecedentTypes[0];
            const result = this.callbacks.getTypeAtCondition(flow, antecedentType);
            return { result, needsAntecedent: false };
        }

        /**
         * Process a switch clause node.
         */
        private processSwitchClause(
            flow: FlowSwitchClause,
            workItem: WorkItem,
            currentIndex: number
        ): { result?: FlowType; needsAntecedent: boolean } {
            // Need antecedent type first
            if (!workItem.antecedentTypes) {
                this.pushWorkItem(flow.antecedent, currentIndex);
                return { needsAntecedent: true };
            }

            const antecedentType = workItem.antecedentTypes[0];
            const result = this.callbacks.getTypeAtSwitchClause(flow, antecedentType);
            return { result, needsAntecedent: false };
        }

        /**
         * Process a label (branch or loop) node.
         */
        private processLabel(
            flow: FlowLabel,
            workItem: WorkItem,
            currentIndex: number
        ): { result?: FlowType; needsAntecedent: boolean } {
            const antecedents = flow.antecedents;

            if (!antecedents || antecedents.length === 0) {
                return { result: this.callbacks.getInitialType(), needsAntecedent: false };
            }

            // Single antecedent - just process it
            if (antecedents.length === 1) {
                if (!workItem.antecedentTypes) {
                    this.pushWorkItem(antecedents[0], currentIndex);
                    return { needsAntecedent: true };
                }
                return { result: workItem.antecedentTypes[0], needsAntecedent: false };
            }

            // Multiple antecedents - need to process all
            const processedCount = workItem.processedAntecedents || 0;

            if (processedCount < antecedents.length) {
                // Push next antecedent to process
                this.pushWorkItem(antecedents[processedCount], currentIndex);
                return { needsAntecedent: true };
            }

            // All antecedents processed, combine results
            const types = workItem.antecedentTypes || [];
            const isBranchLabel = !!(flow.flags & FlowFlags.BranchLabel);
            const result = this.callbacks.createUnionType(types, isBranchLabel);
            return { result, needsAntecedent: false };
        }

        /**
         * Process an array mutation node.
         */
        private processArrayMutation(
            flow: FlowArrayMutation,
            workItem: WorkItem,
            currentIndex: number
        ): { result?: FlowType; needsAntecedent: boolean } {
            // Need antecedent type first
            if (!workItem.antecedentTypes) {
                this.pushWorkItem(flow.antecedent, currentIndex);
                return { needsAntecedent: true };
            }

            const antecedentType = workItem.antecedentTypes[0];
            const type = this.callbacks.getTypeAtArrayMutation(flow, antecedentType);

            if (type !== undefined) {
                return { result: type, needsAntecedent: false };
            }

            return { result: antecedentType, needsAntecedent: false };
        }

        /**
         * Process a reduce label node.
         */
        private processReduceLabel(
            flow: FlowReduceLabel,
            workItem: WorkItem,
            currentIndex: number
        ): { result?: FlowType; needsAntecedent: boolean } {
            // Save and modify the target's antecedents
            const target = flow.target;
            const saveAntecedents = target.antecedents;
            target.antecedents = flow.antecedents;

            // Need antecedent type
            if (!workItem.antecedentTypes) {
                this.pushWorkItem(flow.antecedent, currentIndex);
                return { needsAntecedent: true };
            }

            // Restore antecedents
            target.antecedents = saveAntecedents;

            return { result: workItem.antecedentTypes[0], needsAntecedent: false };
        }

        /**
         * Process a flow start node.
         */
        private processStart(flow: FlowStart): FlowType {
            const container = flow.node;

            // Check if we should continue with containing function's flow
            if (container && container !== this.flowContainer) {
                // For now, just return initial type
                // In full implementation, would need to push container's flowNode
            }

            return this.callbacks.getInitialType();
        }

        /**
         * Get the current iteration count (for debugging/monitoring).
         */
        public getIterationCount(): number {
            return this.iterations;
        }

        /**
         * Check if the analysis hit any limits.
         */
        public didHitLimit(): boolean {
            return this.hitLimit;
        }
    }

    /**
     * Create a flow analyzer with default configuration.
     */
    export function createFlowAnalyzer(callbacks: FlowTypeCallbacks): FlowAnalyzer {
        return new FlowAnalyzer(callbacks);
    }

    /**
     * Create a flow analyzer with custom configuration.
     */
    export function createFlowAnalyzerWithConfig(
        callbacks: FlowTypeCallbacks,
        config: Partial<FlowAnalyzerConfig>
    ): FlowAnalyzer {
        return new FlowAnalyzer(callbacks, { ...DEFAULT_FLOW_ANALYZER_CONFIG, ...config });
    }
}
