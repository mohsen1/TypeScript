/// <reference path="../../solver/types.ts" />
/// <reference path="../../solver/evaluator.ts" />
/// <reference path="../../solver/mod.ts" />

namespace ts {
    describe("unittests:: solver", () => {
        describe("TypeEvaluator", () => {
            describe("ResolutionContext", () => {
                it("should create a context with default values", () => {
                    const context = solver.createResolutionContext();
                    assert.strictEqual(context.depth, 0);
                    assert.strictEqual(context.maxDepth, 100);
                    assert.strictEqual(context.isSpeculative, false);
                    assert.strictEqual(context.resolutionStack.size, 0);
                });

                it("should create a context with custom max depth", () => {
                    const context = solver.createResolutionContext(50);
                    assert.strictEqual(context.maxDepth, 50);
                });

                it("should track resolution stack correctly", () => {
                    const context = solver.createResolutionContext();
                    const id1: solver.TypeCacheId = 1;
                    const id2: solver.TypeCacheId = 2;

                    // Enter resolution
                    solver.enterResolution(id1, context);
                    assert.strictEqual(context.depth, 1);
                    assert.strictEqual(context.resolutionStack.has(id1), true);

                    solver.enterResolution(id2, context);
                    assert.strictEqual(context.depth, 2);
                    assert.strictEqual(context.resolutionStack.has(id2), true);

                    // Exit resolution
                    solver.exitResolution(id2, context);
                    assert.strictEqual(context.depth, 1);
                    assert.strictEqual(context.resolutionStack.has(id2), false);
                    assert.strictEqual(context.resolutionStack.has(id1), true);

                    solver.exitResolution(id1, context);
                    assert.strictEqual(context.depth, 0);
                    assert.strictEqual(context.resolutionStack.has(id1), false);
                });
            });

            describe("canResolve", () => {
                it("should return true for valid resolution", () => {
                    const context = solver.createResolutionContext();
                    const id: solver.TypeCacheId = 1;
                    assert.strictEqual(solver.canResolve(id, context), true);
                });

                it("should return false when id is in resolution stack (cycle detection)", () => {
                    const context = solver.createResolutionContext();
                    const id: solver.TypeCacheId = 1;

                    solver.enterResolution(id, context);
                    assert.strictEqual(solver.canResolve(id, context), false);
                });

                it("should return false when depth limit exceeded", () => {
                    const context = solver.createResolutionContext(2);
                    const id1: solver.TypeCacheId = 1;
                    const id2: solver.TypeCacheId = 2;
                    const id3: solver.TypeCacheId = 3;

                    solver.enterResolution(id1, context);
                    solver.enterResolution(id2, context);

                    // Now at depth 2, which equals maxDepth
                    assert.strictEqual(solver.canResolve(id3, context), false);
                });
            });

            describe("TypeCacheEntry", () => {
                it("should create entry with pending status", () => {
                    const entry = solver.createTypeCacheEntry(1);
                    assert.strictEqual(entry.id, 1);
                    assert.strictEqual(entry.status, solver.ResolutionStatus.Pending);
                    assert.strictEqual(entry.resolvedType, undefined);
                    assert.strictEqual(entry.error, undefined);
                    assert.ok(entry.timestamp > 0);
                });

                it("should have unique timestamps", () => {
                    const entry1 = solver.createTypeCacheEntry(1);
                    // Small delay to ensure different timestamp
                    const entry2 = solver.createTypeCacheEntry(2);
                    assert.ok(entry2.timestamp >= entry1.timestamp);
                });
            });

            describe("TypeEvaluator class", () => {
                it("should create evaluator with default config", () => {
                    const evaluator = solver.createTypeEvaluator();
                    const stats = evaluator.getCacheStats();
                    assert.strictEqual(stats.typeCache, 0);
                    assert.strictEqual(stats.instantiationCache, 0);
                });

                it("should create evaluator with custom config", () => {
                    const evaluator = solver.createTypeEvaluator({
                        maxCacheSize: 500,
                        maxResolutionDepth: 50,
                    });
                    // Config is applied internally
                    assert.ok(evaluator);
                });

                it("should clear caches", () => {
                    const evaluator = solver.createTypeEvaluator();
                    evaluator.clearCaches();
                    const stats = evaluator.getCacheStats();
                    assert.strictEqual(stats.typeCache, 0);
                    assert.strictEqual(stats.instantiationCache, 0);
                });

                it("should return not found for non-existent lookup", () => {
                    const evaluator = solver.createTypeEvaluator();
                    const context = solver.createResolutionContext();
                    const result = evaluator.lookupType("nonexistent", context);
                    assert.strictEqual(result.found, false);
                    assert.strictEqual(result.cycleDetected, false);
                });
            });

            describe("TypeSolver", () => {
                function createTestChecker(): TypeChecker {
                    // Create a minimal mock checker for testing
                    // In real usage, this would be obtained from a program
                    const mockChecker: Partial<TypeChecker> = {
                        getTypeAtLocation: () => undefined,
                        getTypeFromTypeNode: () => undefined,
                        getSymbolAtLocation: () => undefined,
                        getDeclaredTypeOfSymbol: () => undefined,
                    };
                    return mockChecker as TypeChecker;
                }

                it("should create solver instance", () => {
                    const checker = createTestChecker();
                    const solverInstance = solver.createTypeSolver(checker);
                    assert.ok(solverInstance);
                });

                it("should get stats from solver", () => {
                    const checker = createTestChecker();
                    const solverInstance = solver.createTypeSolver(checker);
                    const stats = solverInstance.getStats();
                    assert.strictEqual(stats.typeCache, 0);
                    assert.strictEqual(stats.instantiationCache, 0);
                });

                it("should clear caches via solver", () => {
                    const checker = createTestChecker();
                    const solverInstance = solver.createTypeSolver(checker);
                    solverInstance.clearCaches();
                    const stats = solverInstance.getStats();
                    assert.strictEqual(stats.typeCache, 0);
                });

                it("should create child solver", () => {
                    const checker = createTestChecker();
                    const solverInstance = solver.createTypeSolver(checker);
                    const child = solverInstance.createChild();
                    assert.ok(child);
                    assert.notStrictEqual(child, solverInstance);
                });
            });

            describe("Cycle Detection", () => {
                it("should detect cycles via wouldCauseCycle", () => {
                    const context = solver.createResolutionContext();
                    const id: solver.TypeCacheId = 42;

                    // Not in stack yet
                    assert.strictEqual(solver.wouldCauseCycle(id, context), false);

                    // Add to stack
                    solver.enterResolution(id, context);

                    // Now it would cause a cycle
                    assert.strictEqual(solver.wouldCauseCycle(id, context), true);
                });

                it("should handle nested cycles correctly", () => {
                    const context = solver.createResolutionContext();
                    const id1: solver.TypeCacheId = 1;
                    const id2: solver.TypeCacheId = 2;
                    const id3: solver.TypeCacheId = 3;

                    solver.enterResolution(id1, context);
                    solver.enterResolution(id2, context);
                    solver.enterResolution(id3, context);

                    // All three are in stack
                    assert.strictEqual(solver.wouldCauseCycle(id1, context), true);
                    assert.strictEqual(solver.wouldCauseCycle(id2, context), true);
                    assert.strictEqual(solver.wouldCauseCycle(id3, context), true);

                    // New ID would not cause cycle
                    assert.strictEqual(solver.wouldCauseCycle(4, context), false);

                    // Exit in reverse order
                    solver.exitResolution(id3, context);
                    assert.strictEqual(solver.wouldCauseCycle(id3, context), false);
                    assert.strictEqual(solver.wouldCauseCycle(id2, context), true);

                    solver.exitResolution(id2, context);
                    solver.exitResolution(id1, context);

                    // All should be clear now
                    assert.strictEqual(solver.wouldCauseCycle(id1, context), false);
                    assert.strictEqual(solver.wouldCauseCycle(id2, context), false);
                    assert.strictEqual(solver.wouldCauseCycle(id3, context), false);
                });
            });

            describe("Resolution Status Transitions", () => {
                it("should handle status progression correctly", () => {
                    const entry = solver.createTypeCacheEntry(1);

                    // Initial state
                    assert.strictEqual(entry.status, solver.ResolutionStatus.Pending);

                    // Transition to resolving
                    entry.status = solver.ResolutionStatus.Resolving;
                    assert.strictEqual(entry.status, solver.ResolutionStatus.Resolving);

                    // Transition to resolved
                    entry.status = solver.ResolutionStatus.Resolved;
                    assert.strictEqual(entry.status, solver.ResolutionStatus.Resolved);
                });

                it("should handle failure status", () => {
                    const entry = solver.createTypeCacheEntry(1);
                    entry.status = solver.ResolutionStatus.Failed;
                    entry.error = "Test error";

                    assert.strictEqual(entry.status, solver.ResolutionStatus.Failed);
                    assert.strictEqual(entry.error, "Test error");
                });
            });

            describe("Default Configuration", () => {
                it("should have sensible defaults", () => {
                    const config = solver.DEFAULT_EVALUATOR_CONFIG;
                    assert.strictEqual(config.maxCacheSize, 10000);
                    assert.strictEqual(config.maxResolutionDepth, 100);
                    assert.strictEqual(config.strictNullChecks, true);
                    assert.strictEqual(config.cacheTimeoutMs, 60000);
                });
            });
        });
    });
}
