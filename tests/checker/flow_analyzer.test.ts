/**
 * Tests for the iterative worklist-based flow analyzer.
 *
 * These tests verify that the FlowAnalyzer class correctly implements
 * iterative (non-recursive) flow analysis to handle deeply nested code
 * without stack overflow.
 */

namespace ts {
    // Mock types for testing
    interface MockType extends Type {
        name: string;
    }

    function createMockType(name: string, flags: TypeFlags = TypeFlags.Object): MockType {
        return { name, flags } as MockType;
    }

    // Mock flow nodes for testing
    let flowIdCounter = 0;

    function createFlowStart(): FlowStart {
        return {
            flags: FlowFlags.Start,
            id: ++flowIdCounter,
        };
    }

    function createFlowLabel(antecedents: FlowNode[], isLoop = false): FlowLabel {
        return {
            flags: isLoop ? FlowFlags.LoopLabel : FlowFlags.BranchLabel,
            id: ++flowIdCounter,
            antecedents,
        };
    }

    function createFlowCondition(antecedent: FlowNode, assumeTrue: boolean): FlowCondition {
        return {
            flags: assumeTrue ? FlowFlags.TrueCondition : FlowFlags.FalseCondition,
            id: ++flowIdCounter,
            node: {} as any,
            antecedent,
        };
    }

    // Create mock callbacks
    function createMockCallbacks(overrides: Partial<FlowTypeCallbacks> = {}): FlowTypeCallbacks {
        const initialType = createMockType("initial");
        const declaredType = createMockType("declared");
        const errorType = createMockType("error");

        return {
            getInitialType: () => initialType,
            getDeclaredType: () => declaredType,
            getTypeAtAssignment: () => undefined,
            getTypeAtCall: (_, antecedentType) => antecedentType,
            getTypeAtCondition: (_, antecedentType) => antecedentType,
            getTypeAtSwitchClause: (_, antecedentType) => antecedentType,
            getTypeAtArrayMutation: () => undefined,
            createUnionType: (types) => types[0] || initialType,
            isIncomplete: () => false,
            createFlowType: (type) => type,
            getTypeFromFlowType: (flowType) => flowType as Type,
            getBaseTypeOfLiteralType: (type) => type,
            getErrorType: () => errorType,
            convertAutoToAny: (type) => type,
            isReachableFlowNode: () => true,
            ...overrides,
        };
    }

    describe("FlowAnalyzer", () => {
        beforeEach(() => {
            flowIdCounter = 0;
        });

        describe("constructor and configuration", () => {
            it("should create analyzer with default config", () => {
                const callbacks = createMockCallbacks();
                const analyzer = createFlowAnalyzer(callbacks);
                assert.isDefined(analyzer);
            });

            it("should create analyzer with custom config", () => {
                const callbacks = createMockCallbacks();
                const config: Partial<FlowAnalyzerConfig> = {
                    maxIterations: 50000,
                    maxWorklistSize: 25000,
                };
                const analyzer = createFlowAnalyzerWithConfig(callbacks, config);
                assert.isDefined(analyzer);
            });
        });

        describe("analyzeFlow", () => {
            it("should return declared type when flowNode is undefined", () => {
                const callbacks = createMockCallbacks();
                const analyzer = createFlowAnalyzer(callbacks);

                const result = analyzer.analyzeFlow(undefined, {} as any);

                assert.isFalse(result.hitLimit);
                assert.isFalse(result.incomplete);
                assert.equal((result.type as MockType)?.name, "declared");
            });

            it("should handle simple flow start", () => {
                const callbacks = createMockCallbacks();
                const analyzer = createFlowAnalyzer(callbacks);

                const flowStart = createFlowStart();
                const result = analyzer.analyzeFlow(flowStart, {} as any);

                assert.isFalse(result.hitLimit);
                assert.equal((result.type as MockType)?.name, "initial");
            });

            it("should handle single antecedent branch label", () => {
                const callbacks = createMockCallbacks();
                const analyzer = createFlowAnalyzer(callbacks);

                const flowStart = createFlowStart();
                const flowLabel = createFlowLabel([flowStart]);

                const result = analyzer.analyzeFlow(flowLabel, {} as any);

                assert.isFalse(result.hitLimit);
                assert.equal((result.type as MockType)?.name, "initial");
            });

            it("should handle multiple antecedent branch label", () => {
                const unionType = createMockType("union");

                const callbacks = createMockCallbacks({
                    createUnionType: () => unionType,
                });
                const analyzer = createFlowAnalyzer(callbacks);

                const flowStart1 = createFlowStart();
                const flowStart2 = createFlowStart();
                const flowLabel = createFlowLabel([flowStart1, flowStart2]);

                const result = analyzer.analyzeFlow(flowLabel, {} as any);

                assert.isFalse(result.hitLimit);
                assert.equal((result.type as MockType)?.name, "union");
            });

            it("should handle condition flow", () => {
                const narrowedType = createMockType("narrowed");
                const callbacks = createMockCallbacks({
                    getTypeAtCondition: () => narrowedType,
                });
                const analyzer = createFlowAnalyzer(callbacks);

                const flowStart = createFlowStart();
                const flowCondition = createFlowCondition(flowStart, true);

                const result = analyzer.analyzeFlow(flowCondition, {} as any);

                assert.isFalse(result.hitLimit);
                assert.equal((result.type as MockType)?.name, "narrowed");
            });
        });

        describe("deeply nested structures", () => {
            it("should handle deep chain without stack overflow", () => {
                const callbacks = createMockCallbacks();
                const analyzer = createFlowAnalyzer(callbacks);

                // Create a deeply nested chain of flow nodes
                let current: FlowNode = createFlowStart();
                const depth = 1000;

                for (let i = 0; i < depth; i++) {
                    current = createFlowCondition(current, true);
                }

                const result = analyzer.analyzeFlow(current, {} as any);

                assert.isFalse(result.hitLimit);
                assert.isTrue(result.iterations > depth);
            });

            it("should handle wide branch label without stack overflow", () => {
                const callbacks = createMockCallbacks();
                const analyzer = createFlowAnalyzer(callbacks);

                // Create a wide branch with many antecedents
                const antecedents: FlowNode[] = [];
                const width = 500;

                for (let i = 0; i < width; i++) {
                    antecedents.push(createFlowStart());
                }

                const flowLabel = createFlowLabel(antecedents);
                const result = analyzer.analyzeFlow(flowLabel, {} as any);

                assert.isFalse(result.hitLimit);
            });

            it("should hit iteration limit on excessive iterations", () => {
                const callbacks = createMockCallbacks();
                const config: Partial<FlowAnalyzerConfig> = {
                    maxIterations: 100,
                };
                const analyzer = createFlowAnalyzerWithConfig(callbacks, config);

                // Create a chain longer than the iteration limit
                let current: FlowNode = createFlowStart();
                for (let i = 0; i < 200; i++) {
                    current = createFlowCondition(current, true);
                }

                const result = analyzer.analyzeFlow(current, {} as any);

                assert.isTrue(result.hitLimit);
                assert.equal((result.type as MockType)?.name, "error");
            });
        });

        describe("shared flow node caching", () => {
            it("should cache results for shared flow nodes", () => {
                let callCount = 0;
                const callbacks = createMockCallbacks({
                    getTypeAtCondition: (flow, antecedentType) => {
                        callCount++;
                        return antecedentType;
                    },
                });
                const analyzer = createFlowAnalyzer(callbacks);

                // Create a diamond-shaped flow graph
                const flowStart = createFlowStart();
                const condition1 = createFlowCondition(flowStart, true);
                const condition2 = createFlowCondition(flowStart, false);

                // Mark the start as shared
                (flowStart as any).flags |= FlowFlags.Shared;

                const flowLabel = createFlowLabel([condition1, condition2]);

                const result = analyzer.analyzeFlow(flowLabel, {} as any);

                assert.isFalse(result.hitLimit);
                // The shared flow start should be processed, but cached result should be reused
            });
        });

        describe("reset", () => {
            it("should reset analyzer state between analyses", () => {
                const callbacks = createMockCallbacks();
                const analyzer = createFlowAnalyzer(callbacks);

                const flowStart1 = createFlowStart();
                analyzer.analyzeFlow(flowStart1, {} as any);

                // Verify iteration count is reset
                assert.isTrue(analyzer.getIterationCount() > 0);

                analyzer.reset();
                assert.equal(analyzer.getIterationCount(), 0);
                assert.isFalse(analyzer.didHitLimit());
            });
        });
    });

    describe("FlowAnalyzerConfig", () => {
        it("should have sensible defaults", () => {
            assert.equal(DEFAULT_FLOW_ANALYZER_CONFIG.maxIterations, 100000);
            assert.equal(DEFAULT_FLOW_ANALYZER_CONFIG.maxWorklistSize, 50000);
            assert.isTrue(DEFAULT_FLOW_ANALYZER_CONFIG.enableCaching);
        });
    });
}
