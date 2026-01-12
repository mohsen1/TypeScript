/**
 * Flow Graph for Control Flow Analysis (CFA) - Definite Assignment Tracking
 *
 * This module provides a side-table based flow graph structure that tracks variable
 * states through the control flow to detect use-before-definite-assignment errors (TS2454).
 *
 * The flow graph is constructed post-binding and does not mutate AST nodes.
 * It operates as a separate data structure that can be queried by the Checker.
 */

import {
    Node,
    Block,
    Statement,
    IfStatement,
    ForStatement,
    ForOfStatement,
    ForInStatement,
    WhileStatement,
    DoStatement,
    TryStatement,
    SwitchStatement,
    CaseClause,
    DefaultClause,
    VariableDeclaration,
    Identifier,
    BinaryExpression,
    Expression,
    SyntaxKind,
} from "./types";

/**
 * Variable state for definite assignment analysis
 */
export const enum VariableState {
    /** Variable is definitely assigned at this point */
    DefinitelyAssigned = 0,
    /** Variable may be assigned at this point (some paths assign, some don't) */
    MaybeAssigned = 1,
    /** Variable is unassigned at this point */
    Unassigned = 2,
}

/**
 * Types of flow nodes in the control flow graph
 */
export const enum FlowNodeKind {
    /** Entry point of a basic block */
    BlockEntry = "BlockEntry",
    /** Exit point of a basic block */
    BlockExit = "BlockExit",
    /** Variable assignment */
    Assignment = "Assignment",
    /** Conditional branch point */
    Condition = "Condition",
    /** Branch (true/false path of a condition) */
    Branch = "Branch",
    /** Loop header */
    LoopHeader = "LoopHeader",
    /** Join point (where multiple control flows merge) */
    Join = "Join",
}

/**
 * Base interface for all flow graph nodes
 */
export interface FlowGraphNode {
    /** Unique identifier for this node */
    id: number;
    /** Kind of flow node */
    kind: FlowNodeKind;
    /** AST node this flow node represents (if any) */
    node: Node | undefined;
    /** Outgoing edges from this node */
    outgoingEdges: FlowEdge[];
}

/**
 * Entry point of a basic block
 */
export interface BlockEntryNode extends FlowGraphNode {
    kind: FlowNodeKind.BlockEntry;
    /** The block this node represents */
    node: Block;
}

/**
 * Exit point of a basic block
 */
export interface BlockExitNode extends FlowGraphNode {
    kind: FlowNodeKind.BlockExit;
    /** The block this node represents */
    node: Block;
}

/**
 * Variable assignment node
 */
export interface AssignmentNode extends FlowGraphNode {
    kind: FlowNodeKind.Assignment;
    /** The assignment statement or variable declaration */
    node: VariableDeclaration | BinaryExpression;
    /** The variable being assigned */
    target: Identifier;
}

/**
 * Conditional branch point (if, while, etc.)
 */
export interface ConditionNode extends FlowGraphNode {
    kind: FlowNodeKind.Condition;
    /** The condition expression */
    node: Expression;
    /** References to variables used in the condition */
    referencedVars: Identifier[];
}

/**
 * Branch edge (true or false path from a condition)
 */
export interface BranchNode extends FlowGraphNode {
    kind: FlowNodeKind.Branch;
    /** The condition node this branches from */
    node: Expression;
    /** Whether this is the true or false branch */
    condition: boolean;
}

/**
 * Loop header node
 */
export interface LoopHeaderNode extends FlowGraphNode {
    kind: FlowNodeKind.LoopHeader;
    /** The loop statement */
    node: ForStatement | ForOfStatement | ForInStatement | WhileStatement | DoStatement;
}

/**
 * Join point (where multiple control flows merge)
 */
export interface JoinNode extends FlowGraphNode {
    kind: FlowNodeKind.Join;
    /** Optional AST node for debugging */
    node: Node | undefined;
    /** Incoming paths that merge at this point */
    incomingPaths: number;
}

/**
 * Flow edge with optional condition
 */
export interface FlowEdge {
    /** Source node ID */
    from: number;
    /** Target node ID */
    to: number;
    /** Optional condition flag (undefined for unconditional edges) */
    condition: boolean | undefined;
}

/**
 * The main flow graph structure
 *
 * Stores all nodes and edges for a function's control flow.
 * Acts as a side-table that can be queried for variable states at any point.
 */
export class FlowGraph {
    /** Map of node ID to flow node */
    private nodes = new Map<number, FlowGraphNode>();
    /** All edges in the graph */
    private edges: FlowEdge[] = [];
    /** Counter for generating unique node IDs */
    private nextId = 0;
    /** Entry node of the graph */
    entryNode: BlockEntryNode | undefined;

    /** Add a node to the graph */
    addNode(node: FlowGraphNode): void {
        node.id = this.nextId++;
        this.nodes.set(node.id, node);
    }

    /** Add an edge to the graph */
    addEdge(from: number, to: number, condition?: boolean): void {
        this.edges.push({ from, to, condition });
        const fromNode = this.nodes.get(from);
        if (fromNode) {
            fromNode.outgoingEdges.push({ from, to, condition });
        }
    }

    /** Get a node by ID */
    getNode(id: number): FlowGraphNode | undefined {
        return this.nodes.get(id);
    }

    /** Get all nodes */
    getAllNodes(): IterableIterator<FlowGraphNode> {
        return this.nodes.values();
    }

    /** Get all edges */
    getAllEdges(): readonly FlowEdge[] {
        return this.edges;
    }

    /** Get the entry node */
    getEntryNode(): BlockEntryNode | undefined {
        return this.entryNode;
    }

    /** Set the entry node */
    setEntryNode(node: BlockEntryNode): void {
        this.entryNode = node;
    }

    /** Get outgoing edges from a node */
    getOutgoingEdges(nodeId: number): FlowEdge[] {
        const node = this.nodes.get(nodeId);
        return node?.outgoingEdges ?? [];
    }

    /** Get predecessors of a node */
    getPredecessors(nodeId: number): FlowGraphNode[] {
        const predecessors: FlowGraphNode[] = [];
        for (const edge of this.edges) {
            if (edge.to === nodeId) {
                const fromNode = this.nodes.get(edge.from);
                if (fromNode) {
                    predecessors.push(fromNode);
                }
            }
        }
        return predecessors;
    }

    /** Get successors of a node */
    getSuccessors(nodeId: number): FlowGraphNode[] {
        const successors: FlowGraphNode[] = [];
        const outgoingEdges = this.getOutgoingEdges(nodeId);
        for (const edge of outgoingEdges) {
            const toNode = this.nodes.get(edge.to);
            if (toNode) {
                successors.push(toNode);
            }
        }
        return successors;
    }
}

/**
 * Builder for constructing flow graphs from AST nodes
 */
export class FlowGraphBuilder {
    private graph: FlowGraph;

    constructor() {
        this.graph = new FlowGraph();
    }

    /** Get the constructed graph */
    getGraph(): FlowGraph {
        return this.graph;
    }

    /** Build a flow graph from a block of statements */
    buildFromBlock(block: Block): FlowGraph {
        const entryNode: BlockEntryNode = {
            id: -1, // Will be assigned by addNode
            kind: FlowNodeKind.BlockEntry,
            node: block,
            outgoingEdges: [],
        };
        this.graph.addNode(entryNode);
        this.graph.setEntryNode(entryNode);

        let currentNodeId = entryNode.id;
        for (const statement of block.statements) {
            currentNodeId = this.processStatement(statement, currentNodeId);
        }

        // Create exit node
        const exitNode: BlockExitNode = {
            id: -1,
            kind: FlowNodeKind.BlockExit,
            node: block,
            outgoingEdges: [],
        };
        this.graph.addNode(exitNode);
        this.graph.addEdge(currentNodeId, exitNode.id);

        return this.graph;
    }

    /** Process a statement and return the ID of the last node created */
    private processStatement(statement: Statement, fromNodeId: number): number {
        switch (statement.kind) {
            case SyntaxKind.ExpressionStatement:
                return this.processExpressionStatement(statement, fromNodeId);

            case SyntaxKind.IfStatement:
                return this.processIfStatement(statement as IfStatement, fromNodeId);

            case SyntaxKind.ForStatement:
                return this.processForStatement(statement as ForStatement, fromNodeId);

            case SyntaxKind.ForOfStatement:
                return this.processForOfStatement(statement as ForOfStatement, fromNodeId);

            case SyntaxKind.ForInStatement:
                return this.processForInStatement(statement as ForInStatement, fromNodeId);

            case SyntaxKind.WhileStatement:
                return this.processWhileStatement(statement as WhileStatement, fromNodeId);

            case SyntaxKind.DoStatement:
                return this.processDoStatement(statement as DoStatement, fromNodeId);

            case SyntaxKind.TryStatement:
                return this.processTryStatement(statement as TryStatement, fromNodeId);

            case SyntaxKind.SwitchStatement:
                return this.processSwitchStatement(statement as SwitchStatement, fromNodeId);

            case SyntaxKind.Block:
                return this.processBlock(statement as Block, fromNodeId);

            case SyntaxKind.VariableStatement:
                return this.processVariableStatement(statement, fromNodeId);

            case SyntaxKind.ReturnStatement:
            case SyntaxKind.BreakStatement:
            case SyntaxKind.ContinueStatement:
            case SyntaxKind.ThrowStatement:
            case SyntaxKind.EmptyStatement:
            case SyntaxKind.DebuggerStatement:
            case SyntaxKind.ImportDeclaration:
            case SyntaxKind.ExportDeclaration:
            case SyntaxKind.ImportEqualsDeclaration:
                // These statements don't affect definite assignment for local variables
                return fromNodeId;

            default:
                return fromNodeId;
        }
    }

    private processExpressionStatement(statement: Statement, fromNodeId: number): number {
        // Check if this is an assignment expression
        const expr = (statement as any).expression;
        if (expr && expr.kind === SyntaxKind.BinaryExpression) {
            const binaryExpr = expr as BinaryExpression;
            const opToken = binaryExpr.operatorToken.kind;
            if (this.isAssignmentOperator(opToken)) {
                const assignmentNode: AssignmentNode = {
                    id: -1,
                    kind: FlowNodeKind.Assignment,
                    node: binaryExpr,
                    target: this.extractTarget(binaryExpr.left),
                    outgoingEdges: [],
                };
                this.graph.addNode(assignmentNode);
                this.graph.addEdge(fromNodeId, assignmentNode.id);
                return assignmentNode.id;
            }
        }
        return fromNodeId;
    }

    private processVariableStatement(statement: Statement, fromNodeId: number): number {
        const varStatement = statement as any;
        if (varStatement.declarationList && varStatement.declarationList.declarations) {
            for (const declaration of varStatement.declarationList.declarations) {
                if (declaration.initializer) {
                    const assignmentNode: AssignmentNode = {
                        id: -1,
                        kind: FlowNodeKind.Assignment,
                        node: declaration,
                        target: declaration.name,
                        outgoingEdges: [],
                    };
                    this.graph.addNode(assignmentNode);
                    this.graph.addEdge(fromNodeId, assignmentNode.id);
                    return assignmentNode.id;
                }
            }
        }
        return fromNodeId;
    }

    private processIfStatement(statement: IfStatement, fromNodeId: number): number {
        const conditionNode: ConditionNode = {
            id: -1,
            kind: FlowNodeKind.Condition,
            node: statement.expression,
            referencedVars: this.extractReferencedVariables(statement.expression),
            outgoingEdges: [],
        };
        this.graph.addNode(conditionNode);
        this.graph.addEdge(fromNodeId, conditionNode.id);

        // True branch
        const trueBranchNode: BranchNode = {
            id: -1,
            kind: FlowNodeKind.Branch,
            node: statement.expression,
            condition: true,
            outgoingEdges: [],
        };
        this.graph.addNode(trueBranchNode);
        this.graph.addEdge(conditionNode.id, trueBranchNode.id);

        const trueBranchEndId = this.processStatement(statement.thenStatement, trueBranchNode.id);

        // False branch
        let falseBranchEndId = conditionNode.id;
        if (statement.elseStatement) {
            const falseBranchNode: BranchNode = {
                id: -1,
                kind: FlowNodeKind.Branch,
                node: statement.expression,
                condition: false,
                outgoingEdges: [],
            };
            this.graph.addNode(falseBranchNode);
            this.graph.addEdge(conditionNode.id, falseBranchNode.id);
            falseBranchEndId = this.processStatement(statement.elseStatement, falseBranchNode.id);
        }

        // Create join node
        const joinNode: JoinNode = {
            id: -1,
            kind: FlowNodeKind.Join,
            node: statement,
            incomingPaths: statement.elseStatement ? 2 : 1,
            outgoingEdges: [],
        };
        this.graph.addNode(joinNode);
        this.graph.addEdge(trueBranchEndId, joinNode.id);
        this.graph.addEdge(falseBranchEndId, joinNode.id);

        return joinNode.id;
    }

    private processForStatement(statement: ForStatement, fromNodeId: number): number {
        // Process initializer - can be VariableDeclarationList or Expression
        let currentId = fromNodeId;
        if (statement.initializer) {
            currentId = this.processForInitializer(statement.initializer, currentId);
        }

        // Create loop header
        const loopHeader: LoopHeaderNode = {
            id: -1,
            kind: FlowNodeKind.LoopHeader,
            node: statement,
            outgoingEdges: [],
        };
        this.graph.addNode(loopHeader);
        this.graph.addEdge(currentId, loopHeader.id);

        // Condition check
        let conditionId = loopHeader.id;
        if (statement.condition) {
            const conditionNode: ConditionNode = {
                id: -1,
                kind: FlowNodeKind.Condition,
                node: statement.condition,
                referencedVars: this.extractReferencedVariables(statement.condition),
                outgoingEdges: [],
            };
            this.graph.addNode(conditionNode);
            this.graph.addEdge(loopHeader.id, conditionNode.id);
            conditionId = conditionNode.id;
        }

        // Loop body
        const bodyEndId = this.processStatement(statement.statement, conditionId);

        // Incrementor - if it's an assignment expression, track it
        let incrementorId = bodyEndId;
        if (statement.incrementor) {
            incrementorId = this.processExpressionAsAssignment(statement.incrementor, bodyEndId);
        }

        // Back edge to loop header
        this.graph.addEdge(incrementorId, loopHeader.id);

        // Exit from loop
        const joinNode: JoinNode = {
            id: -1,
            kind: FlowNodeKind.Join,
            node: statement,
            incomingPaths: 2,
            outgoingEdges: [],
        };
        this.graph.addNode(joinNode);
        this.graph.addEdge(conditionId, joinNode.id); // Exit when condition is false
        this.graph.addEdge(incrementorId, joinNode.id); // Exit after loop (for break)

        return joinNode.id;
    }

    /** Process a ForInitializer (VariableDeclarationList or Expression) */
    private processForInitializer(initializer: Node, fromNodeId: number): number {
        // Check if it's a VariableDeclarationList
        if ("declarations" in initializer && Array.isArray((initializer as any).declarations)) {
            const varDeclList = initializer as any;
            let currentId = fromNodeId;
            for (const declaration of varDeclList.declarations) {
                if (declaration.initializer) {
                    const assignmentNode: AssignmentNode = {
                        id: -1,
                        kind: FlowNodeKind.Assignment,
                        node: declaration,
                        target: declaration.name,
                        outgoingEdges: [],
                    };
                    this.graph.addNode(assignmentNode);
                    this.graph.addEdge(currentId, assignmentNode.id);
                    currentId = assignmentNode.id;
                }
            }
            return currentId;
        }
        // Otherwise it's an expression - check if it's an assignment
        return this.processExpressionAsAssignment(initializer, fromNodeId);
    }

    /** Process an expression as an assignment if it is one */
    private processExpressionAsAssignment(expr: Node, fromNodeId: number): number {
        if (expr.kind === SyntaxKind.BinaryExpression) {
            const binaryExpr = expr as BinaryExpression;
            const opToken = binaryExpr.operatorToken.kind;
            if (this.isAssignmentOperator(opToken)) {
                const assignmentNode: AssignmentNode = {
                    id: -1,
                    kind: FlowNodeKind.Assignment,
                    node: binaryExpr,
                    target: this.extractTarget(binaryExpr.left),
                    outgoingEdges: [],
                };
                this.graph.addNode(assignmentNode);
                this.graph.addEdge(fromNodeId, assignmentNode.id);
                return assignmentNode.id;
            }
        }
        return fromNodeId;
    }

    private processForOfStatement(statement: ForOfStatement, fromNodeId: number): number {
        // For-of loops assign the variable on each iteration
        // The initializer is a ForInitializer (VariableDeclarationList or Expression)
        let currentId = fromNodeId;
        currentId = this.processForInitializer(statement.initializer, currentId);

        const loopHeader: LoopHeaderNode = {
            id: -1,
            kind: FlowNodeKind.LoopHeader,
            node: statement,
            outgoingEdges: [],
        };
        this.graph.addNode(loopHeader);
        this.graph.addEdge(currentId, loopHeader.id);

        const bodyEndId = this.processStatement(statement.statement, loopHeader.id);

        // Back edge - the initializer runs again on each iteration
        const newCurrentId = this.processForInitializer(statement.initializer, bodyEndId);
        this.graph.addEdge(newCurrentId, loopHeader.id);

        const joinNode: JoinNode = {
            id: -1,
            kind: FlowNodeKind.Join,
            node: statement,
            incomingPaths: 1,
            outgoingEdges: [],
        };
        this.graph.addNode(joinNode);
        this.graph.addEdge(loopHeader.id, joinNode.id);

        return joinNode.id;
    }

    private processForInStatement(statement: ForInStatement, fromNodeId: number): number {
        // For-in loops assign the variable on each iteration
        // The initializer is a ForInitializer (VariableDeclarationList or Expression)
        let currentId = fromNodeId;
        currentId = this.processForInitializer(statement.initializer, currentId);

        const loopHeader: LoopHeaderNode = {
            id: -1,
            kind: FlowNodeKind.LoopHeader,
            node: statement,
            outgoingEdges: [],
        };
        this.graph.addNode(loopHeader);
        this.graph.addEdge(currentId, loopHeader.id);

        const bodyEndId = this.processStatement(statement.statement, loopHeader.id);

        // Back edge - the initializer runs again on each iteration
        const newCurrentId = this.processForInitializer(statement.initializer, bodyEndId);
        this.graph.addEdge(newCurrentId, loopHeader.id);

        const joinNode: JoinNode = {
            id: -1,
            kind: FlowNodeKind.Join,
            node: statement,
            incomingPaths: 1,
            outgoingEdges: [],
        };
        this.graph.addNode(joinNode);
        this.graph.addEdge(loopHeader.id, joinNode.id);

        return joinNode.id;
    }

    private processWhileStatement(statement: WhileStatement, fromNodeId: number): number {
        const conditionNode: ConditionNode = {
            id: -1,
            kind: FlowNodeKind.Condition,
            node: statement.expression,
            referencedVars: this.extractReferencedVariables(statement.expression),
            outgoingEdges: [],
        };
        this.graph.addNode(conditionNode);
        this.graph.addEdge(fromNodeId, conditionNode.id);

        const bodyEndId = this.processStatement(statement.statement, conditionNode.id);
        this.graph.addEdge(bodyEndId, conditionNode.id); // Back edge

        const joinNode: JoinNode = {
            id: -1,
            kind: FlowNodeKind.Join,
            node: statement,
            incomingPaths: 1,
            outgoingEdges: [],
        };
        this.graph.addNode(joinNode);
        this.graph.addEdge(conditionNode.id, joinNode.id);

        return joinNode.id;
    }

    private processDoStatement(statement: DoStatement, fromNodeId: number): number {
        const bodyEndId = this.processStatement(statement.statement, fromNodeId);

        const conditionNode: ConditionNode = {
            id: -1,
            kind: FlowNodeKind.Condition,
            node: statement.expression,
            referencedVars: this.extractReferencedVariables(statement.expression),
            outgoingEdges: [],
        };
        this.graph.addNode(conditionNode);
        this.graph.addEdge(bodyEndId, conditionNode.id);
        this.graph.addEdge(conditionNode.id, fromNodeId); // Back edge

        const joinNode: JoinNode = {
            id: -1,
            kind: FlowNodeKind.Join,
            node: statement,
            incomingPaths: 1,
            outgoingEdges: [],
        };
        this.graph.addNode(joinNode);
        this.graph.addEdge(conditionNode.id, joinNode.id);

        return joinNode.id;
    }

    private processTryStatement(statement: TryStatement, fromNodeId: number): number {
        // Process try block
        const tryBlockEndId = this.processBlock(statement.tryBlock, fromNodeId);

        // Process catch block if present
        let currentId = tryBlockEndId;
        if (statement.catchClause) {
            const catchBlockEndId = this.processBlock(statement.catchClause.block, fromNodeId);
            const joinNode: JoinNode = {
                id: -1,
                kind: FlowNodeKind.Join,
                node: statement,
                incomingPaths: 2,
                outgoingEdges: [],
            };
            this.graph.addNode(joinNode);
            this.graph.addEdge(tryBlockEndId, joinNode.id);
            this.graph.addEdge(catchBlockEndId, joinNode.id);
            currentId = joinNode.id;
        }

        // Process finally block if present
        if (statement.finallyBlock) {
            currentId = this.processBlock(statement.finallyBlock, currentId);
        }

        return currentId;
    }

    private processSwitchStatement(statement: SwitchStatement, fromNodeId: number): number {
        const conditionNode: ConditionNode = {
            id: -1,
            kind: FlowNodeKind.Condition,
            node: statement.expression,
            referencedVars: this.extractReferencedVariables(statement.expression),
            outgoingEdges: [],
        };
        this.graph.addNode(conditionNode);
        this.graph.addEdge(fromNodeId, conditionNode.id);

        const caseEndIds: number[] = [];

        for (const clause of statement.caseBlock.clauses) {
            const clauseStartId = conditionNode.id;

            // Process case clause statements
            let clauseEndId = clauseStartId;
            for (const stmt of (clause as CaseClause | DefaultClause).statements) {
                clauseEndId = this.processStatement(stmt, clauseEndId);
            }

            caseEndIds.push(clauseEndId);
        }

        // Join all case paths
        const joinNode: JoinNode = {
            id: -1,
            kind: FlowNodeKind.Join,
            node: statement,
            incomingPaths: caseEndIds.length,
            outgoingEdges: [],
        };
        this.graph.addNode(joinNode);
        for (const endId of caseEndIds) {
            this.graph.addEdge(endId, joinNode.id);
        }

        return joinNode.id;
    }

    private processBlock(block: Block, fromNodeId: number): number {
        const blockEntry: BlockEntryNode = {
            id: -1,
            kind: FlowNodeKind.BlockEntry,
            node: block,
            outgoingEdges: [],
        };
        this.graph.addNode(blockEntry);
        this.graph.addEdge(fromNodeId, blockEntry.id);

        let currentId = blockEntry.id;
        for (const statement of block.statements) {
            currentId = this.processStatement(statement, currentId);
        }

        const blockExit: BlockExitNode = {
            id: -1,
            kind: FlowNodeKind.BlockExit,
            node: block,
            outgoingEdges: [],
        };
        this.graph.addNode(blockExit);
        this.graph.addEdge(currentId, blockExit.id);

        return blockExit.id;
    }

    /** Helper: Check if an operator is an assignment operator */
    private isAssignmentOperator(kind: number): boolean {
        return kind === SyntaxKind.EqualsToken ||
            kind === SyntaxKind.PlusEqualsToken ||
            kind === SyntaxKind.MinusEqualsToken ||
            kind === SyntaxKind.AsteriskEqualsToken ||
            kind === SyntaxKind.SlashEqualsToken ||
            kind === SyntaxKind.PercentEqualsToken ||
            kind === SyntaxKind.AmpersandEqualsToken ||
            kind === SyntaxKind.BarEqualsToken ||
            kind === SyntaxKind.CaretEqualsToken ||
            kind === SyntaxKind.LessThanLessThanEqualsToken ||
            kind === SyntaxKind.GreaterThanGreaterThanEqualsToken ||
            kind === SyntaxKind.GreaterThanGreaterThanGreaterThanEqualsToken;
    }

    /** Helper: Extract target identifier from an assignment expression */
    private extractTarget(node: Node): Identifier {
        if (node.kind === SyntaxKind.Identifier) {
            return node as Identifier;
        }
        // For property access (a.b = ...), we'd need more complex handling
        return node as Identifier;
    }

    /** Helper: Extract referenced variables from an expression */
    private extractReferencedVariables(expr: Expression): Identifier[] {
        const vars: Identifier[] = [];
        // Simple implementation - traverse and collect identifiers
        // A full implementation would need to handle all expression types
        if (expr.kind === SyntaxKind.Identifier) {
            vars.push(expr as Identifier);
        }
        return vars;
    }
}
