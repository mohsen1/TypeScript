//! Binder implementation for TypeScript AST.
//!
//! The binder walks the AST and creates symbols, establishing
//! scope and name resolution.

use serde::Serialize;
use crate::parser::NodeIndex;
use crate::parser::node_flags;

// =============================================================================
// Symbol Flags
// =============================================================================

/// Flags that describe the kind and properties of a symbol.
/// Matches TypeScript's SymbolFlags enum in src/compiler/types.ts
pub mod symbol_flags {
    pub const NONE: u32 = 0;
    pub const FUNCTION_SCOPED_VARIABLE: u32 = 1 << 0;   // Variable (var) or parameter
    pub const BLOCK_SCOPED_VARIABLE: u32 = 1 << 1;      // Block-scoped variable (let or const)
    pub const PROPERTY: u32 = 1 << 2;                   // Property or enum member
    pub const ENUM_MEMBER: u32 = 1 << 3;                // Enum member
    pub const FUNCTION: u32 = 1 << 4;                   // Function
    pub const CLASS: u32 = 1 << 5;                      // Class
    pub const INTERFACE: u32 = 1 << 6;                  // Interface
    pub const CONST_ENUM: u32 = 1 << 7;                 // Const enum
    pub const REGULAR_ENUM: u32 = 1 << 8;               // Enum
    pub const VALUE_MODULE: u32 = 1 << 9;               // Instantiated module
    pub const NAMESPACE_MODULE: u32 = 1 << 10;          // Uninstantiated module
    pub const TYPE_LITERAL: u32 = 1 << 11;              // Type Literal or mapped type
    pub const OBJECT_LITERAL: u32 = 1 << 12;            // Object Literal
    pub const METHOD: u32 = 1 << 13;                    // Method
    pub const CONSTRUCTOR: u32 = 1 << 14;               // Constructor
    pub const GET_ACCESSOR: u32 = 1 << 15;              // Get accessor
    pub const SET_ACCESSOR: u32 = 1 << 16;              // Set accessor
    pub const SIGNATURE: u32 = 1 << 17;                 // Call, construct, or index signature
    pub const TYPE_PARAMETER: u32 = 1 << 18;            // Type parameter
    pub const TYPE_ALIAS: u32 = 1 << 19;                // Type alias
    pub const EXPORT_VALUE: u32 = 1 << 20;              // Exported value marker
    pub const ALIAS: u32 = 1 << 21;                     // Alias for another symbol
    pub const PROTOTYPE: u32 = 1 << 22;                 // Prototype property
    pub const EXPORT_STAR: u32 = 1 << 23;               // Export * declaration
    pub const OPTIONAL: u32 = 1 << 24;                  // Optional property
    pub const TRANSIENT: u32 = 1 << 25;                 // Transient symbol
    pub const ASSIGNMENT: u32 = 1 << 26;                // Assignment treated as declaration
    pub const MODULE_EXPORTS: u32 = 1 << 27;            // CommonJS module.exports

    // Composite flags
    pub const ENUM: u32 = REGULAR_ENUM | CONST_ENUM;
    pub const VARIABLE: u32 = FUNCTION_SCOPED_VARIABLE | BLOCK_SCOPED_VARIABLE;
    pub const VALUE: u32 = VARIABLE | PROPERTY | ENUM_MEMBER | OBJECT_LITERAL |
                           FUNCTION | CLASS | ENUM | VALUE_MODULE | METHOD |
                           GET_ACCESSOR | SET_ACCESSOR;
    pub const TYPE: u32 = CLASS | INTERFACE | ENUM | ENUM_MEMBER | TYPE_LITERAL |
                          TYPE_PARAMETER | TYPE_ALIAS;
    pub const NAMESPACE: u32 = VALUE_MODULE | NAMESPACE_MODULE | ENUM;
    pub const MODULE: u32 = VALUE_MODULE | NAMESPACE_MODULE;
    pub const ACCESSOR: u32 = GET_ACCESSOR | SET_ACCESSOR;

    // Exclusion flags for redeclaration checks
    pub const FUNCTION_SCOPED_VARIABLE_EXCLUDES: u32 = VALUE & !FUNCTION_SCOPED_VARIABLE;
    pub const BLOCK_SCOPED_VARIABLE_EXCLUDES: u32 = VALUE;
    pub const PARAMETER_EXCLUDES: u32 = VALUE;
    pub const PROPERTY_EXCLUDES: u32 = NONE;
    pub const ENUM_MEMBER_EXCLUDES: u32 = VALUE | TYPE;
    pub const FUNCTION_EXCLUDES: u32 = VALUE & !FUNCTION;
    pub const CLASS_EXCLUDES: u32 = VALUE | TYPE & !VALUE_MODULE & !INTERFACE & !FUNCTION;
    pub const INTERFACE_EXCLUDES: u32 = TYPE & !INTERFACE & !CLASS;
    pub const REGULAR_ENUM_EXCLUDES: u32 = VALUE | TYPE & !REGULAR_ENUM;
    pub const CONST_ENUM_EXCLUDES: u32 = VALUE | TYPE & !CONST_ENUM;
    pub const VALUE_MODULE_EXCLUDES: u32 = VALUE & !FUNCTION & !CLASS & !REGULAR_ENUM & !VALUE_MODULE;
    pub const NAMESPACE_MODULE_EXCLUDES: u32 = NONE;
    pub const METHOD_EXCLUDES: u32 = VALUE & !METHOD;
    pub const GET_ACCESSOR_EXCLUDES: u32 = VALUE & !SET_ACCESSOR;
    pub const SET_ACCESSOR_EXCLUDES: u32 = VALUE & !GET_ACCESSOR;
    pub const TYPE_PARAMETER_EXCLUDES: u32 = TYPE & !TYPE_PARAMETER;
    pub const TYPE_ALIAS_EXCLUDES: u32 = TYPE;
    pub const ALIAS_EXCLUDES: u32 = ALIAS;
}

// =============================================================================
// Symbol
// =============================================================================

/// Unique identifier for a symbol in the symbol table.
#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash, Serialize)]
pub struct SymbolId(pub u32);

impl SymbolId {
    pub const NONE: SymbolId = SymbolId(u32::MAX);

    pub fn is_none(&self) -> bool {
        self.0 == u32::MAX
    }
}

/// A symbol represents a named entity in the program.
/// Symbols are created during binding and used during type checking.
#[derive(Clone, Debug, Serialize)]
pub struct Symbol {
    /// Symbol flags describing kind and properties
    pub flags: u32,
    /// Escaped name of the symbol
    pub escaped_name: String,
    /// Declarations associated with this symbol
    pub declarations: Vec<NodeIndex>,
    /// First value declaration of the symbol
    pub value_declaration: NodeIndex,
    /// Parent symbol (for nested symbols)
    pub parent: SymbolId,
    /// Unique ID for this symbol
    pub id: SymbolId,
}

impl Symbol {
    /// Create a new symbol with the given flags and name.
    pub fn new(id: SymbolId, flags: u32, name: String) -> Self {
        Symbol {
            flags,
            escaped_name: name,
            declarations: Vec::new(),
            value_declaration: NodeIndex::NONE,
            parent: SymbolId::NONE,
            id,
        }
    }

    /// Check if symbol has all specified flags.
    pub fn has_flags(&self, flags: u32) -> bool {
        (self.flags & flags) == flags
    }

    /// Check if symbol has any of specified flags.
    pub fn has_any_flags(&self, flags: u32) -> bool {
        (self.flags & flags) != 0
    }
}

// =============================================================================
// Symbol Table
// =============================================================================

/// A symbol table maps names to symbols.
/// Used for scope management and name resolution.
#[derive(Clone, Debug, Default, Serialize)]
pub struct SymbolTable {
    /// Symbols indexed by their escaped name
    symbols: std::collections::HashMap<String, SymbolId>,
}

impl SymbolTable {
    pub fn new() -> Self {
        SymbolTable {
            symbols: std::collections::HashMap::new(),
        }
    }

    /// Get a symbol by name.
    pub fn get(&self, name: &str) -> Option<SymbolId> {
        self.symbols.get(name).copied()
    }

    /// Set a symbol by name.
    pub fn set(&mut self, name: String, symbol: SymbolId) {
        self.symbols.insert(name, symbol);
    }

    /// Check if a name exists in the table.
    pub fn has(&self, name: &str) -> bool {
        self.symbols.contains_key(name)
    }

    /// Get number of symbols.
    pub fn len(&self) -> usize {
        self.symbols.len()
    }

    /// Check if empty.
    pub fn is_empty(&self) -> bool {
        self.symbols.is_empty()
    }

    /// Iterate over symbols.
    pub fn iter(&self) -> impl Iterator<Item = (&String, &SymbolId)> {
        self.symbols.iter()
    }
}

// =============================================================================
// Symbol Arena
// =============================================================================

/// Arena allocator for symbols.
#[derive(Debug, Serialize)]
pub struct SymbolArena {
    symbols: Vec<Symbol>,
    /// Base offset for symbol IDs (0 for binder, high value for checker-local symbols)
    base_offset: u32,
}

impl Default for SymbolArena {
    fn default() -> Self {
        SymbolArena {
            symbols: Vec::new(),
            base_offset: 0,
        }
    }
}

impl SymbolArena {
    /// Base offset for checker-local symbols to avoid ID collisions.
    pub const CHECKER_SYMBOL_BASE: u32 = 0x10000000;

    pub fn new() -> Self {
        SymbolArena {
            symbols: Vec::new(),
            base_offset: 0,
        }
    }

    /// Create a new symbol arena with a base offset for symbol IDs.
    /// Used for checker-local symbols to avoid collisions with binder symbols.
    pub fn new_with_base(base: u32) -> Self {
        SymbolArena {
            symbols: Vec::new(),
            base_offset: base,
        }
    }

    /// Allocate a new symbol and return its ID.
    pub fn alloc(&mut self, flags: u32, name: String) -> SymbolId {
        let id = SymbolId(self.base_offset + self.symbols.len() as u32);
        self.symbols.push(Symbol::new(id, flags, name));
        id
    }

    /// Get a symbol by ID.
    pub fn get(&self, id: SymbolId) -> Option<&Symbol> {
        if id.is_none() {
            None
        } else if id.0 < self.base_offset {
            // ID is from a different arena (e.g., binder vs checker)
            None
        } else {
            self.symbols.get((id.0 - self.base_offset) as usize)
        }
    }

    /// Get a mutable symbol by ID.
    pub fn get_mut(&mut self, id: SymbolId) -> Option<&mut Symbol> {
        if id.is_none() {
            None
        } else if id.0 < self.base_offset {
            // ID is from a different arena
            None
        } else {
            self.symbols.get_mut((id.0 - self.base_offset) as usize)
        }
    }

    /// Get the number of symbols.
    pub fn len(&self) -> usize {
        self.symbols.len()
    }

    /// Check if empty.
    pub fn is_empty(&self) -> bool {
        self.symbols.is_empty()
    }
}

// =============================================================================
// Control Flow Graph
// =============================================================================

/// Flags for flow nodes describing their type and properties.
/// Matches TypeScript's FlowFlags in src/compiler/types.ts
pub mod flow_flags {
    pub const UNREACHABLE: u32 = 1 << 0;      // Unreachable code
    pub const START: u32 = 1 << 1;             // Start of flow graph
    pub const BRANCH_LABEL: u32 = 1 << 2;      // Branch label
    pub const LOOP_LABEL: u32 = 1 << 3;        // Loop label
    pub const ASSIGNMENT: u32 = 1 << 4;        // Assignment
    pub const TRUE_CONDITION: u32 = 1 << 5;    // True condition
    pub const FALSE_CONDITION: u32 = 1 << 6;   // False condition
    pub const SWITCH_CLAUSE: u32 = 1 << 7;     // Switch clause
    pub const ARRAY_MUTATION: u32 = 1 << 8;    // Array mutation
    pub const CALL: u32 = 1 << 9;              // Call expression
    pub const REDUCE_LABEL: u32 = 1 << 10;     // Reduce label
    pub const REFERENCED: u32 = 1 << 11;       // Referenced

    // Composite flags
    pub const LABEL: u32 = BRANCH_LABEL | LOOP_LABEL;
    pub const CONDITION: u32 = TRUE_CONDITION | FALSE_CONDITION;
}

/// Unique identifier for a flow node.
#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash, Serialize)]
pub struct FlowNodeId(pub u32);

impl FlowNodeId {
    pub const NONE: FlowNodeId = FlowNodeId(u32::MAX);

    pub fn is_none(&self) -> bool {
        self.0 == u32::MAX
    }
}

/// A node in the control flow graph.
#[derive(Clone, Debug, Serialize)]
pub struct FlowNode {
    /// Flow node flags
    pub flags: u32,
    /// Flow node ID
    pub id: FlowNodeId,
    /// Antecedent flow node(s) - predecessors in the control flow
    pub antecedent: Vec<FlowNodeId>,
    /// Associated AST node (for assignments, conditions, etc.)
    pub node: NodeIndex,
}

impl FlowNode {
    pub fn new(id: FlowNodeId, flags: u32) -> Self {
        FlowNode {
            flags,
            id,
            antecedent: Vec::new(),
            node: NodeIndex::NONE,
        }
    }

    pub fn has_flags(&self, flags: u32) -> bool {
        (self.flags & flags) == flags
    }

    pub fn has_any_flags(&self, flags: u32) -> bool {
        (self.flags & flags) != 0
    }
}

/// Arena for flow nodes.
#[derive(Debug, Default, Serialize)]
pub struct FlowNodeArena {
    nodes: Vec<FlowNode>,
}

impl FlowNodeArena {
    pub fn new() -> Self {
        FlowNodeArena { nodes: Vec::new() }
    }

    /// Allocate a new flow node.
    pub fn alloc(&mut self, flags: u32) -> FlowNodeId {
        let id = FlowNodeId(self.nodes.len() as u32);
        self.nodes.push(FlowNode::new(id, flags));
        id
    }

    /// Get a flow node by ID.
    pub fn get(&self, id: FlowNodeId) -> Option<&FlowNode> {
        if id.is_none() {
            None
        } else {
            self.nodes.get(id.0 as usize)
        }
    }

    /// Get a mutable flow node by ID.
    pub fn get_mut(&mut self, id: FlowNodeId) -> Option<&mut FlowNode> {
        if id.is_none() {
            None
        } else {
            self.nodes.get_mut(id.0 as usize)
        }
    }

    pub fn len(&self) -> usize {
        self.nodes.len()
    }

    pub fn is_empty(&self) -> bool {
        self.nodes.is_empty()
    }
}

// =============================================================================
// Binder State
// =============================================================================

use wasm_bindgen::prelude::*;
use crate::parser::{Node, NodeArena};

/// Container kind - tracks what kind of scope we're in
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum ContainerKind {
    /// Source file (global scope)
    SourceFile,
    /// Function/method body (creates function scope)
    Function,
    /// Module/namespace body
    Module,
    /// Class body
    Class,
    /// Block (if, while, for, etc.) - only creates block scope
    Block,
}

/// Scope context - tracks scope chain and hoisting
#[derive(Clone, Debug)]
pub struct ScopeContext {
    /// The symbol table for this scope
    pub locals: SymbolTable,
    /// Parent scope (for scope chain lookup)
    pub parent_idx: Option<usize>,
    /// The kind of container this scope belongs to
    pub container_kind: ContainerKind,
    /// Node index of the container
    pub container_node: NodeIndex,
    /// Hoisted var declarations (for function scope)
    pub hoisted_vars: Vec<(String, NodeIndex)>,
    /// Hoisted function declarations (for function scope)
    pub hoisted_functions: Vec<(String, NodeIndex)>,
}

impl ScopeContext {
    pub fn new(kind: ContainerKind, node: NodeIndex, parent: Option<usize>) -> Self {
        ScopeContext {
            locals: SymbolTable::new(),
            parent_idx: parent,
            container_kind: kind,
            container_node: node,
            hoisted_vars: Vec::new(),
            hoisted_functions: Vec::new(),
        }
    }

    /// Check if this scope is a function scope (where var hoisting happens)
    pub fn is_function_scope(&self) -> bool {
        matches!(self.container_kind, ContainerKind::SourceFile | ContainerKind::Function | ContainerKind::Module)
    }
}

/// Binder state for walking the AST and creating symbols.
#[wasm_bindgen]
pub struct BinderState {
    /// Arena for allocating symbols
    #[wasm_bindgen(skip)]
    pub symbols: SymbolArena,
    /// Current container's symbol table (locals)
    #[wasm_bindgen(skip)]
    pub current_scope: SymbolTable,
    /// Stack of scopes for nested blocks
    scope_stack: Vec<SymbolTable>,
    /// File-level symbol table
    #[wasm_bindgen(skip)]
    pub file_locals: SymbolTable,
    /// Flow node arena for control flow analysis
    #[wasm_bindgen(skip)]
    pub flow_nodes: FlowNodeArena,
    /// Current flow node
    current_flow: FlowNodeId,
    /// Unreachable flow node (for never-returning code)
    unreachable_flow: FlowNodeId,
    /// Scope chain - stack of scope contexts
    scope_chain: Vec<ScopeContext>,
    /// Current scope index in scope_chain
    current_scope_idx: usize,
}

impl BinderState {
    pub fn new() -> Self {
        let mut flow_nodes = FlowNodeArena::new();
        // Create the unreachable flow node
        let unreachable_flow = flow_nodes.alloc(flow_flags::UNREACHABLE);

        BinderState {
            symbols: SymbolArena::new(),
            current_scope: SymbolTable::new(),
            scope_stack: Vec::new(),
            file_locals: SymbolTable::new(),
            flow_nodes,
            current_flow: FlowNodeId::NONE,
            unreachable_flow,
            scope_chain: Vec::new(),
            current_scope_idx: 0,
        }
    }

    /// Bind a source file, creating symbols for all declarations.
    pub fn bind_source_file(&mut self, arena: &NodeArena, root: NodeIndex) {
        // Initialize scope chain with source file scope
        self.scope_chain.clear();
        self.scope_chain.push(ScopeContext::new(ContainerKind::SourceFile, root, None));
        self.current_scope_idx = 0;
        self.current_scope = SymbolTable::new();

        // Create START flow node for the file
        let start_flow = self.flow_nodes.alloc(flow_flags::START);
        self.current_flow = start_flow;

        if let Some(node) = arena.get(root) {
            if let Node::SourceFile(sf) = node {
                // First pass: collect hoisted declarations
                self.collect_hoisted_declarations(arena, &sf.statements.nodes);

                // Process hoisted function declarations first
                self.process_hoisted_functions(arena);

                // Second pass: bind each statement
                for &stmt_idx in &sf.statements.nodes {
                    self.bind_node(arena, stmt_idx);
                }
            }
        }

        // Store file locals
        self.file_locals = std::mem::take(&mut self.current_scope);
    }

    /// Collect hoisted declarations from statements.
    /// var declarations are hoisted to the containing function scope.
    /// function declarations are hoisted to the top of the containing function scope.
    fn collect_hoisted_declarations(&mut self, arena: &NodeArena, statements: &[NodeIndex]) {
        for &stmt_idx in statements {
            if let Some(node) = arena.get(stmt_idx) {
                match node {
                    // var declarations are hoisted
                    Node::VariableStatement(stmt) => {
                        if let Some(Node::VariableDeclarationList(list)) = arena.get(stmt.declaration_list) {
                            // Check if this is a var declaration (not let/const)
                            // Use proper flags instead of magic numbers
                            let is_var = (list.base.flags & (node_flags::LET | node_flags::CONST)) == 0;
                            if is_var {
                                for &decl_idx in &list.declarations.nodes {
                                    if let Some(Node::VariableDeclaration(decl)) = arena.get(decl_idx) {
                                        if let Some(name) = self.get_identifier_name(arena, decl.name) {
                                            self.add_hoisted_var(name, decl_idx);
                                        }
                                    }
                                }
                            }
                        }
                    }
                    // function declarations are hoisted
                    Node::FunctionDeclaration(func) => {
                        if let Some(name) = self.get_identifier_name(arena, func.name) {
                            self.add_hoisted_function(name, stmt_idx);
                        }
                    }
                    // Recurse into blocks for var hoisting (but not let/const)
                    Node::Block(block) => {
                        self.collect_hoisted_declarations(arena, &block.statements.nodes);
                    }
                    Node::IfStatement(if_stmt) => {
                        if let Some(Node::Block(block)) = arena.get(if_stmt.then_statement) {
                            self.collect_hoisted_declarations(arena, &block.statements.nodes);
                        }
                        if !if_stmt.else_statement.is_none() {
                            if let Some(Node::Block(block)) = arena.get(if_stmt.else_statement) {
                                self.collect_hoisted_declarations(arena, &block.statements.nodes);
                            }
                        }
                    }
                    Node::WhileStatement(while_stmt) => {
                        if let Some(Node::Block(block)) = arena.get(while_stmt.statement) {
                            self.collect_hoisted_declarations(arena, &block.statements.nodes);
                        }
                    }
                    Node::ForStatement(for_stmt) => {
                        if let Some(Node::Block(block)) = arena.get(for_stmt.statement) {
                            self.collect_hoisted_declarations(arena, &block.statements.nodes);
                        }
                    }
                    _ => {}
                }
            }
        }
    }

    /// Add a hoisted var declaration to the current function scope
    fn add_hoisted_var(&mut self, name: String, decl_idx: NodeIndex) {
        if let Some(scope) = self.scope_chain.get_mut(self.current_scope_idx) {
            scope.hoisted_vars.push((name, decl_idx));
        }
    }

    /// Add a hoisted function declaration to the current function scope
    fn add_hoisted_function(&mut self, name: String, decl_idx: NodeIndex) {
        if let Some(scope) = self.scope_chain.get_mut(self.current_scope_idx) {
            scope.hoisted_functions.push((name, decl_idx));
        }
    }

    /// Process hoisted function declarations (declare them before other code)
    fn process_hoisted_functions(&mut self, arena: &NodeArena) {
        // Get hoisted functions from current scope
        let hoisted = if let Some(scope) = self.scope_chain.get(self.current_scope_idx) {
            scope.hoisted_functions.clone()
        } else {
            return;
        };

        // Declare each hoisted function
        for (name, decl_idx) in hoisted {
            if let Some(Node::FunctionDeclaration(_)) = arena.get(decl_idx) {
                self.declare_symbol(name, symbol_flags::FUNCTION, decl_idx);
            }
        }
    }

    /// Enter a new scope (function, block, etc.)
    fn enter_scope(&mut self, kind: ContainerKind, node: NodeIndex) {
        let parent = Some(self.current_scope_idx);
        let new_idx = self.scope_chain.len();
        self.scope_chain.push(ScopeContext::new(kind, node, parent));
        self.current_scope_idx = new_idx;

        // Also push to the legacy scope stack for compatibility
        let old_scope = std::mem::take(&mut self.current_scope);
        self.scope_stack.push(old_scope);
        self.current_scope = SymbolTable::new();
    }

    /// Exit the current scope
    fn exit_scope(&mut self) {
        // Pop from scope chain
        if let Some(scope) = self.scope_chain.get(self.current_scope_idx) {
            if let Some(parent_idx) = scope.parent_idx {
                self.current_scope_idx = parent_idx;
            }
        }

        // Pop from legacy scope stack
        if let Some(parent_scope) = self.scope_stack.pop() {
            self.current_scope = parent_scope;
        }
    }

    /// Find the enclosing function scope for var hoisting
    fn find_function_scope_idx(&self) -> usize {
        let mut idx = self.current_scope_idx;
        while let Some(scope) = self.scope_chain.get(idx) {
            if scope.is_function_scope() {
                return idx;
            }
            if let Some(parent_idx) = scope.parent_idx {
                idx = parent_idx;
            } else {
                break;
            }
        }
        0 // Fall back to source file scope
    }

    /// Look up a symbol in the scope chain
    pub fn lookup_symbol(&self, name: &str) -> Option<SymbolId> {
        // First check current scope
        if let Some(id) = self.current_scope.get(name) {
            return Some(id);
        }

        // Walk up the scope chain
        let mut idx = self.current_scope_idx;
        while let Some(scope) = self.scope_chain.get(idx) {
            if let Some(id) = scope.locals.get(name) {
                return Some(id);
            }
            if let Some(parent_idx) = scope.parent_idx {
                idx = parent_idx;
            } else {
                break;
            }
        }

        // Finally check file locals
        self.file_locals.get(name)
    }

    /// Create a new flow node and set it as current.
    fn create_flow_node(&mut self, flags: u32, antecedent: FlowNodeId, node: NodeIndex) -> FlowNodeId {
        let flow_id = self.flow_nodes.alloc(flags);
        if let Some(flow) = self.flow_nodes.get_mut(flow_id) {
            if !antecedent.is_none() {
                flow.antecedent.push(antecedent);
            }
            flow.node = node;
        }
        flow_id
    }

    /// Create a branch label flow node (for merging control flow).
    fn create_branch_label(&mut self) -> FlowNodeId {
        self.flow_nodes.alloc(flow_flags::BRANCH_LABEL)
    }

    /// Add an antecedent to a branch label.
    fn add_antecedent(&mut self, label: FlowNodeId, antecedent: FlowNodeId) {
        if !antecedent.is_none() && antecedent != self.unreachable_flow {
            if let Some(flow) = self.flow_nodes.get_mut(label) {
                if !flow.antecedent.contains(&antecedent) {
                    flow.antecedent.push(antecedent);
                }
            }
        }
    }

    /// Check if current flow is reachable.
    fn is_reachable(&self) -> bool {
        !self.current_flow.is_none() && self.current_flow != self.unreachable_flow
    }

    /// Bind a single node, creating symbols as needed.
    fn bind_node(&mut self, arena: &NodeArena, idx: NodeIndex) {
        if idx.is_none() {
            return;
        }

        let node = match arena.get(idx) {
            Some(n) => n,
            None => return,
        };

        match node {
            // Variable declarations
            Node::VariableStatement(stmt) => {
                self.bind_node(arena, stmt.declaration_list);
            }
            Node::VariableDeclarationList(list) => {
                for &decl_idx in &list.declarations.nodes {
                    self.bind_node(arena, decl_idx);
                }
            }
            Node::VariableDeclaration(decl) => {
                self.bind_variable_declaration(arena, decl, idx);
            }

            // Function declarations
            Node::FunctionDeclaration(func) => {
                self.bind_function_declaration(arena, func, idx);
            }

            // Class declarations
            Node::ClassDeclaration(class) => {
                self.bind_class_declaration(arena, class, idx);
            }

            // Interface declarations
            Node::InterfaceDeclaration(iface) => {
                self.bind_interface_declaration(arena, iface, idx);
            }

            // Type alias declarations
            Node::TypeAliasDeclaration(alias) => {
                self.bind_type_alias_declaration(arena, alias, idx);
            }

            // Enum declarations
            Node::EnumDeclaration(enum_decl) => {
                self.bind_enum_declaration(arena, enum_decl, idx);
            }

            // Block - creates a new block scope (for let/const)
            Node::Block(block) => {
                self.enter_scope(ContainerKind::Block, idx);
                for &stmt_idx in &block.statements.nodes {
                    self.bind_node(arena, stmt_idx);
                }
                self.exit_scope();
            }

            // Other statements - recurse into children with flow analysis
            Node::IfStatement(if_stmt) => {
                self.bind_if_statement(arena, if_stmt, idx);
            }
            Node::WhileStatement(while_stmt) => {
                self.bind_while_statement(arena, while_stmt, idx);
            }
            Node::ForStatement(for_stmt) => {
                // For statement creates its own block scope for the initializer
                self.enter_scope(ContainerKind::Block, idx);
                self.bind_node(arena, for_stmt.initializer);
                self.bind_node(arena, for_stmt.statement);
                self.exit_scope();
            }
            Node::ForInStatement(for_in) => {
                self.enter_scope(ContainerKind::Block, idx);
                self.bind_node(arena, for_in.initializer);
                self.bind_node(arena, for_in.statement);
                self.exit_scope();
            }
            Node::ForOfStatement(for_of) => {
                self.enter_scope(ContainerKind::Block, idx);
                self.bind_node(arena, for_of.initializer);
                self.bind_node(arena, for_of.statement);
                self.exit_scope();
            }
            Node::SwitchStatement(switch_stmt) => {
                self.bind_switch_statement(arena, switch_stmt, idx);
            }
            Node::TryStatement(try_stmt) => {
                self.bind_try_statement(arena, try_stmt, idx);
            }

            // Import declarations
            Node::ImportDeclaration(import) => {
                self.bind_import_declaration(arena, import, idx);
            }

            // Export declarations
            Node::ExportDeclaration(_export) => {
                // Export declarations don't create new symbols,
                // they reference existing ones
            }

            // Module/namespace declarations
            Node::ModuleDeclaration(module) => {
                self.bind_module_declaration(arena, module, idx);
            }
            Node::ModuleBlock(block) => {
                // Module block - the scope is already created by module declaration
                for &stmt_idx in &block.statements.nodes {
                    self.bind_node(arena, stmt_idx);
                }
            }

            _ => {
                // For other node types, no symbols to create
            }
        }
    }

    /// Push a new scope onto the scope stack.
    fn push_scope(&mut self) {
        let old_scope = std::mem::take(&mut self.current_scope);
        self.scope_stack.push(old_scope);
        self.current_scope = SymbolTable::new();
    }

    /// Pop a scope from the stack.
    fn pop_scope(&mut self) {
        if let Some(parent_scope) = self.scope_stack.pop() {
            self.current_scope = parent_scope;
        }
    }

    /// Declare a symbol in the current scope.
    /// Handles declaration merging for interfaces, namespaces, and functions.
    fn declare_symbol(&mut self, name: String, flags: u32, declaration: NodeIndex) -> SymbolId {
        // Check if symbol already exists
        if let Some(existing_id) = self.current_scope.get(&name) {
            // Get existing flags first to avoid borrow issues
            let existing_flags = self.symbols.get(existing_id).map(|s| s.flags).unwrap_or(0);
            let can_merge = Self::can_merge_flags(existing_flags, flags);

            if let Some(sym) = self.symbols.get_mut(existing_id) {
                if can_merge {
                    // Merge the flags and add the declaration
                    sym.flags |= flags;
                    sym.declarations.push(declaration);
                    if sym.value_declaration.is_none() && (flags & symbol_flags::VALUE) != 0 {
                        sym.value_declaration = declaration;
                    }
                } else {
                    // Conflicting declaration - still add but could report error
                    sym.declarations.push(declaration);
                }
            }
            return existing_id;
        }

        // Create new symbol
        let id = self.symbols.alloc(flags, name.clone());
        if let Some(sym) = self.symbols.get_mut(id) {
            sym.declarations.push(declaration);
            if sym.value_declaration.is_none() && (flags & symbol_flags::VALUE) != 0 {
                sym.value_declaration = declaration;
            }
        }
        self.current_scope.set(name, id);
        id
    }

    /// Check if two symbol flag sets can be merged.
    /// TypeScript allows merging:
    /// - Interface + Interface
    /// - Namespace + Namespace
    /// - Namespace + Class/Function/Enum
    /// - Function + Function (overloads)
    fn can_merge_flags(existing_flags: u32, new_flags: u32) -> bool {
        // Interface can merge with interface
        if (existing_flags & symbol_flags::INTERFACE) != 0
            && (new_flags & symbol_flags::INTERFACE) != 0
        {
            return true;
        }

        // Namespace/module can merge with namespace/module
        if (existing_flags & symbol_flags::MODULE) != 0
            && (new_flags & symbol_flags::MODULE) != 0
        {
            return true;
        }

        // Namespace can merge with class, function, or enum
        if (existing_flags & symbol_flags::MODULE) != 0 {
            if (new_flags & (symbol_flags::CLASS | symbol_flags::FUNCTION | symbol_flags::ENUM)) != 0 {
                return true;
            }
        }
        if (new_flags & symbol_flags::MODULE) != 0 {
            if (existing_flags & (symbol_flags::CLASS | symbol_flags::FUNCTION | symbol_flags::ENUM)) != 0 {
                return true;
            }
        }

        // Function overloads
        if (existing_flags & symbol_flags::FUNCTION) != 0
            && (new_flags & symbol_flags::FUNCTION) != 0
        {
            return true;
        }

        false
    }

    /// Get the name from an identifier node.
    fn get_identifier_name(&self, arena: &NodeArena, idx: NodeIndex) -> Option<String> {
        if let Some(Node::Identifier(id)) = arena.get(idx) {
            Some(id.escaped_text.clone())
        } else {
            None
        }
    }

    // =========================================================================
    // Declaration Binding
    // =========================================================================

    /// Check if this is a var declaration (not let/const) by looking at parent list
    fn is_var_declaration(&self, arena: &NodeArena, decl_idx: NodeIndex) -> bool {
        // We need to look at the parent VariableDeclarationList to determine this
        // For now, use a simple heuristic based on the declaration flags
        // In a full implementation, we'd track this during parsing
        if let Some(Node::VariableDeclaration(decl)) = arena.get(decl_idx) {
            // Check the base node flags - if neither Let nor Const, it's var
            let flags = decl.base.flags;
            (flags & (node_flags::LET | node_flags::CONST)) == 0
        } else {
            false
        }
    }

    fn bind_variable_declaration(
        &mut self,
        arena: &NodeArena,
        decl: &crate::parser::VariableDeclaration,
        decl_idx: NodeIndex,
    ) {
        if let Some(name) = self.get_identifier_name(arena, decl.name) {
            // Determine if this is var (function-scoped) or let/const (block-scoped)
            let is_var = self.is_var_declaration(arena, decl_idx);

            if is_var {
                // var: function-scoped, declares in nearest function scope
                // Note: hoisting already declared it, but we need to handle the
                // actual binding point for flow analysis
                let flags = symbol_flags::FUNCTION_SCOPED_VARIABLE;

                // For var, check if already declared (from hoisting)
                if self.current_scope.has(&name) || self.lookup_symbol(&name).is_some() {
                    // Already declared via hoisting, just bind the initializer
                    // The symbol was already created during hoisting
                } else {
                    // Declare in function scope
                    self.declare_symbol(name.clone(), flags, decl_idx);
                }
            } else {
                // let/const: block-scoped, declares in current block
                let flags = symbol_flags::BLOCK_SCOPED_VARIABLE;
                self.declare_symbol(name, flags, decl_idx);
            }
        }
    }

    fn bind_function_declaration(
        &mut self,
        arena: &NodeArena,
        func: &crate::parser::FunctionDeclaration,
        func_idx: NodeIndex,
    ) {
        // Function declarations are hoisted, so the symbol may already exist
        if let Some(name) = self.get_identifier_name(arena, func.name) {
            // Check if already declared via hoisting
            if !self.current_scope.has(&name) {
                self.declare_symbol(name, symbol_flags::FUNCTION, func_idx);
            }
        }

        // Bind function body in new function scope
        if !func.body.is_none() {
            self.enter_scope(ContainerKind::Function, func_idx);

            // Collect hoisted declarations within function
            if let Some(Node::Block(block)) = arena.get(func.body) {
                self.collect_hoisted_declarations(arena, &block.statements.nodes);
                self.process_hoisted_functions(arena);
            }

            // Bind parameters first (they're in function scope)
            for &param_idx in &func.parameters.nodes {
                if let Some(Node::ParameterDeclaration(param)) = arena.get(param_idx) {
                    if let Some(name) = self.get_identifier_name(arena, param.name) {
                        self.declare_symbol(name, symbol_flags::FUNCTION_SCOPED_VARIABLE, param_idx);
                    }
                }
            }

            // Bind the function body
            self.bind_node(arena, func.body);
            self.exit_scope();
        }
    }

    fn bind_class_declaration(
        &mut self,
        arena: &NodeArena,
        class: &crate::parser::ClassDeclaration,
        class_idx: NodeIndex,
    ) {
        if let Some(name) = self.get_identifier_name(arena, class.name) {
            self.declare_symbol(name, symbol_flags::CLASS, class_idx);
        }

        // Bind class members in a new scope
        self.push_scope();
        for &member_idx in &class.members.nodes {
            self.bind_class_member(arena, member_idx);
        }
        self.pop_scope();
    }

    fn bind_class_member(&mut self, arena: &NodeArena, idx: NodeIndex) {
        if let Some(node) = arena.get(idx) {
            match node {
                Node::MethodDeclaration(method) => {
                    if let Some(name) = self.get_identifier_name(arena, method.name) {
                        self.declare_symbol(name, symbol_flags::METHOD, idx);
                    }
                }
                Node::PropertyDeclaration(prop) => {
                    if let Some(name) = self.get_identifier_name(arena, prop.name) {
                        self.declare_symbol(name, symbol_flags::PROPERTY, idx);
                    }
                }
                Node::ConstructorDeclaration(_) => {
                    self.declare_symbol("constructor".to_string(), symbol_flags::CONSTRUCTOR, idx);
                }
                _ => {}
            }
        }
    }

    /// Bind an if statement with flow analysis.
    fn bind_if_statement(
        &mut self,
        arena: &NodeArena,
        if_stmt: &crate::parser::IfStatement,
        _node_idx: NodeIndex,
    ) {
        // Save the current flow before the condition
        let pre_condition_flow = self.current_flow;

        // Create flow node for the true branch (condition is true)
        let true_flow = self.create_flow_node(
            flow_flags::TRUE_CONDITION,
            pre_condition_flow,
            if_stmt.expression,
        );

        // Bind the then statement with true flow
        self.current_flow = true_flow;
        self.bind_node(arena, if_stmt.then_statement);
        let post_then_flow = self.current_flow;

        // Create a branch label for merging after the if statement
        let merge_label = self.create_branch_label();

        if !if_stmt.else_statement.is_none() {
            // Create flow node for the false branch (condition is false)
            let false_flow = self.create_flow_node(
                flow_flags::FALSE_CONDITION,
                pre_condition_flow,
                if_stmt.expression,
            );

            // Bind the else statement with false flow
            self.current_flow = false_flow;
            self.bind_node(arena, if_stmt.else_statement);
            let post_else_flow = self.current_flow;

            // Add both branches to the merge label
            self.add_antecedent(merge_label, post_then_flow);
            self.add_antecedent(merge_label, post_else_flow);
        } else {
            // No else branch: false path goes directly to merge
            let false_flow = self.create_flow_node(
                flow_flags::FALSE_CONDITION,
                pre_condition_flow,
                if_stmt.expression,
            );

            self.add_antecedent(merge_label, post_then_flow);
            self.add_antecedent(merge_label, false_flow);
        }

        // Set current flow to the merge label
        self.current_flow = merge_label;
    }

    /// Bind a while statement with flow analysis.
    fn bind_while_statement(
        &mut self,
        arena: &NodeArena,
        while_stmt: &crate::parser::WhileStatement,
        _node_idx: NodeIndex,
    ) {
        // Create a loop label for the loop entry
        let loop_label = self.flow_nodes.alloc(flow_flags::LOOP_LABEL);
        if let Some(flow) = self.flow_nodes.get_mut(loop_label) {
            if !self.current_flow.is_none() {
                flow.antecedent.push(self.current_flow);
            }
        }

        self.current_flow = loop_label;

        // Create flow node for the true condition (entering loop body)
        let true_flow = self.create_flow_node(
            flow_flags::TRUE_CONDITION,
            loop_label,
            while_stmt.expression,
        );

        // Bind the loop body
        self.current_flow = true_flow;
        self.bind_node(arena, while_stmt.statement);

        // Loop back to the loop label
        self.add_antecedent(loop_label, self.current_flow);

        // Create flow node for the false condition (exiting loop)
        let false_flow = self.create_flow_node(
            flow_flags::FALSE_CONDITION,
            loop_label,
            while_stmt.expression,
        );

        self.current_flow = false_flow;
    }

    fn bind_interface_declaration(
        &mut self,
        arena: &NodeArena,
        iface: &crate::parser::InterfaceDeclaration,
        iface_idx: NodeIndex,
    ) {
        if let Some(name) = self.get_identifier_name(arena, iface.name) {
            self.declare_symbol(name, symbol_flags::INTERFACE, iface_idx);
        }
    }

    fn bind_type_alias_declaration(
        &mut self,
        arena: &NodeArena,
        alias: &crate::parser::TypeAliasDeclaration,
        alias_idx: NodeIndex,
    ) {
        if let Some(name) = self.get_identifier_name(arena, alias.name) {
            self.declare_symbol(name, symbol_flags::TYPE_ALIAS, alias_idx);
        }
    }

    fn bind_enum_declaration(
        &mut self,
        arena: &NodeArena,
        enum_decl: &crate::parser::EnumDeclaration,
        enum_idx: NodeIndex,
    ) {
        if let Some(name) = self.get_identifier_name(arena, enum_decl.name) {
            self.declare_symbol(name, symbol_flags::REGULAR_ENUM, enum_idx);
        }

        // Bind enum members
        for &member_idx in &enum_decl.members.nodes {
            if let Some(Node::EnumMember(member)) = arena.get(member_idx) {
                if let Some(name) = self.get_identifier_name(arena, member.name) {
                    self.declare_symbol(name, symbol_flags::ENUM_MEMBER, member_idx);
                }
            }
        }
    }

    fn bind_import_declaration(
        &mut self,
        arena: &NodeArena,
        import: &crate::parser::ImportDeclaration,
        _import_idx: NodeIndex,
    ) {
        if let Some(Node::ImportClause(clause)) = arena.get(import.import_clause) {
            // Default import
            if !clause.name.is_none() {
                if let Some(name) = self.get_identifier_name(arena, clause.name) {
                    self.declare_symbol(name, symbol_flags::ALIAS, clause.name);
                }
            }

            // Named imports
            if let Some(Node::NamedImports(named)) = arena.get(clause.named_bindings) {
                for &spec_idx in &named.elements.nodes {
                    if let Some(Node::ImportSpecifier(spec)) = arena.get(spec_idx) {
                        if let Some(name) = self.get_identifier_name(arena, spec.name) {
                            self.declare_symbol(name, symbol_flags::ALIAS, spec_idx);
                        }
                    }
                }
            }
        }
    }

    fn bind_module_declaration(
        &mut self,
        arena: &NodeArena,
        module: &crate::parser::ModuleDeclaration,
        module_idx: NodeIndex,
    ) {
        // Get module name
        if let Some(name) = self.get_identifier_name(arena, module.name) {
            // Determine if this is a namespace (value) or module (ambient)
            // For simplicity, treat as namespace module (can contain values)
            let flags = symbol_flags::NAMESPACE_MODULE | symbol_flags::VALUE_MODULE;
            self.declare_symbol(name, flags, module_idx);
        }

        // Bind module body in new scope
        if !module.body.is_none() {
            self.enter_scope(ContainerKind::Module, module_idx);
            self.bind_node(arena, module.body);
            self.exit_scope();
        }
    }

    /// Bind a switch statement.
    fn bind_switch_statement(
        &mut self,
        arena: &NodeArena,
        switch_stmt: &crate::parser::SwitchStatement,
        node_idx: NodeIndex,
    ) {
        // Save flow before switch
        let pre_switch_flow = self.current_flow;

        // Create a branch label for the end of switch (for break statements)
        let end_label = self.create_branch_label();

        // Bind the case block
        if let Some(Node::CaseBlock(case_block)) = arena.get(switch_stmt.case_block) {
            for &clause_idx in &case_block.clauses.nodes {
                if let Some(node) = arena.get(clause_idx) {
                    match node {
                        Node::CaseClause(clause) => {
                            // Create switch clause flow node
                            let clause_flow = self.create_flow_node(
                                flow_flags::SWITCH_CLAUSE,
                                pre_switch_flow,
                                clause.expression,
                            );
                            self.current_flow = clause_flow;

                            // Bind statements in the clause (new block scope for let/const)
                            self.enter_scope(ContainerKind::Block, clause_idx);
                            for &stmt_idx in &clause.statements.nodes {
                                self.bind_node(arena, stmt_idx);
                            }
                            self.exit_scope();

                            // Add to end label
                            self.add_antecedent(end_label, self.current_flow);
                        }
                        Node::DefaultClause(clause) => {
                            // Default clause is always reachable
                            let clause_flow = self.create_flow_node(
                                flow_flags::SWITCH_CLAUSE,
                                pre_switch_flow,
                                node_idx,
                            );
                            self.current_flow = clause_flow;

                            self.enter_scope(ContainerKind::Block, clause_idx);
                            for &stmt_idx in &clause.statements.nodes {
                                self.bind_node(arena, stmt_idx);
                            }
                            self.exit_scope();

                            self.add_antecedent(end_label, self.current_flow);
                        }
                        _ => {}
                    }
                }
            }
        }

        self.current_flow = end_label;
    }

    /// Bind a try statement.
    fn bind_try_statement(
        &mut self,
        arena: &NodeArena,
        try_stmt: &crate::parser::TryStatement,
        _node_idx: NodeIndex,
    ) {
        let pre_try_flow = self.current_flow;

        // Create merge label for after try/catch/finally
        let end_label = self.create_branch_label();

        // Bind try block
        self.bind_node(arena, try_stmt.try_block);
        let post_try_flow = self.current_flow;

        // Bind catch clause if present
        if !try_stmt.catch_clause.is_none() {
            if let Some(Node::CatchClause(catch)) = arena.get(try_stmt.catch_clause) {
                // Catch clause has its own scope
                self.enter_scope(ContainerKind::Block, try_stmt.catch_clause);

                // Bind catch variable if present
                if !catch.variable_declaration.is_none() {
                    if let Some(Node::VariableDeclaration(decl)) = arena.get(catch.variable_declaration) {
                        if let Some(name) = self.get_identifier_name(arena, decl.name) {
                            self.declare_symbol(name, symbol_flags::BLOCK_SCOPED_VARIABLE, catch.variable_declaration);
                        }
                    }
                }

                // Reset flow - catch can be entered from any point in try
                self.current_flow = pre_try_flow;
                self.bind_node(arena, catch.block);
                self.add_antecedent(end_label, self.current_flow);

                self.exit_scope();
            }
        }

        // Add post-try flow to end label
        self.add_antecedent(end_label, post_try_flow);

        // Bind finally block if present
        if !try_stmt.finally_block.is_none() {
            // Finally is always executed
            self.current_flow = end_label;
            self.bind_node(arena, try_stmt.finally_block);
        } else {
            self.current_flow = end_label;
        }
    }
}

impl Default for BinderState {
    fn default() -> Self {
        Self::new()
    }
}

// =============================================================================
// WASM Methods
// =============================================================================

#[wasm_bindgen]
impl BinderState {
    /// Create a new binder state.
    #[wasm_bindgen(constructor)]
    pub fn create() -> Self {
        Self::new()
    }

    /// Get the number of symbols created.
    #[wasm_bindgen(js_name = getSymbolCount)]
    pub fn get_symbol_count(&self) -> u32 {
        self.symbols.len() as u32
    }

    /// Get file locals as JSON string.
    #[wasm_bindgen(js_name = getFileLocalsJson)]
    pub fn get_file_locals_json(&self) -> String {
        serde_json::to_string(&self.file_locals).unwrap_or_else(|_| "{}".to_string())
    }

    /// Get all symbols as JSON string.
    #[wasm_bindgen(js_name = getSymbolsJson)]
    pub fn get_symbols_json(&self) -> String {
        serde_json::to_string(&self.symbols).unwrap_or_else(|_| "[]".to_string())
    }

    /// Get a symbol by name from file locals.
    #[wasm_bindgen(js_name = getSymbolByName)]
    pub fn get_symbol_by_name(&self, name: &str) -> Option<String> {
        if let Some(id) = self.file_locals.get(name) {
            if let Some(sym) = self.symbols.get(id) {
                return serde_json::to_string(sym).ok();
            }
        }
        None
    }

    /// Check if a name exists in file locals.
    #[wasm_bindgen(js_name = hasSymbol)]
    pub fn has_symbol(&self, name: &str) -> bool {
        self.file_locals.has(name)
    }
}

// =============================================================================
// Tests
// =============================================================================

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_symbol_flags() {
        assert_eq!(symbol_flags::NONE, 0);
        assert_eq!(symbol_flags::FUNCTION_SCOPED_VARIABLE, 1);
        assert_eq!(symbol_flags::BLOCK_SCOPED_VARIABLE, 2);
        assert_eq!(symbol_flags::VARIABLE, 3);
    }

    #[test]
    fn test_symbol_id() {
        let id = SymbolId(42);
        assert_eq!(id.0, 42);
        assert!(!id.is_none());
        assert!(SymbolId::NONE.is_none());
    }

    #[test]
    fn test_flow_flags() {
        assert_eq!(flow_flags::UNREACHABLE, 1);
        assert_eq!(flow_flags::START, 2);
        assert_eq!(flow_flags::LABEL, flow_flags::BRANCH_LABEL | flow_flags::LOOP_LABEL);
        assert_eq!(flow_flags::CONDITION, flow_flags::TRUE_CONDITION | flow_flags::FALSE_CONDITION);
    }

    #[test]
    fn test_flow_node_id() {
        let id = FlowNodeId(42);
        assert_eq!(id.0, 42);
        assert!(!id.is_none());
        assert!(FlowNodeId::NONE.is_none());
    }

    #[test]
    fn test_flow_node() {
        let node = FlowNode::new(FlowNodeId(0), flow_flags::START);
        assert!(node.has_flags(flow_flags::START));
        assert!(!node.has_flags(flow_flags::UNREACHABLE));
        assert!(node.antecedent.is_empty());
    }

    #[test]
    fn test_flow_node_arena() {
        let mut arena = FlowNodeArena::new();
        assert!(arena.is_empty());

        let start = arena.alloc(flow_flags::START);
        let branch = arena.alloc(flow_flags::BRANCH_LABEL);

        assert_eq!(arena.len(), 2);
        assert_eq!(start.0, 0);
        assert_eq!(branch.0, 1);

        let start_node = arena.get(start).unwrap();
        assert!(start_node.has_flags(flow_flags::START));

        let branch_node = arena.get(branch).unwrap();
        assert!(branch_node.has_flags(flow_flags::BRANCH_LABEL));
    }

    #[test]
    fn test_symbol() {
        let sym = Symbol::new(
            SymbolId(0),
            symbol_flags::FUNCTION,
            "myFunc".to_string(),
        );
        assert!(sym.has_flags(symbol_flags::FUNCTION));
        assert!(!sym.has_flags(symbol_flags::CLASS));
        assert_eq!(sym.escaped_name, "myFunc");
    }

    #[test]
    fn test_symbol_table() {
        let mut table = SymbolTable::new();
        assert!(table.is_empty());

        table.set("x".to_string(), SymbolId(0));
        table.set("y".to_string(), SymbolId(1));

        assert_eq!(table.len(), 2);
        assert!(table.has("x"));
        assert!(!table.has("z"));
        assert_eq!(table.get("x"), Some(SymbolId(0)));
        assert_eq!(table.get("z"), None);
    }

    #[test]
    fn test_symbol_arena() {
        let mut arena = SymbolArena::new();
        assert!(arena.is_empty());

        let id1 = arena.alloc(symbol_flags::VARIABLE, "x".to_string());
        let id2 = arena.alloc(symbol_flags::FUNCTION, "f".to_string());

        assert_eq!(arena.len(), 2);
        assert_eq!(id1.0, 0);
        assert_eq!(id2.0, 1);

        let sym1 = arena.get(id1).unwrap();
        assert_eq!(sym1.escaped_name, "x");
        assert!(sym1.has_flags(symbol_flags::VARIABLE));

        let sym2 = arena.get(id2).unwrap();
        assert_eq!(sym2.escaped_name, "f");
        assert!(sym2.has_flags(symbol_flags::FUNCTION));
    }

    #[test]
    fn test_bind_variable_declaration() {
        use crate::parser_impl::ParserState;

        let mut parser = ParserState::new(
            "test.ts".to_string(),
            "const x = 42;".to_string(),
        );
        let root = parser.parse_source_file();

        let mut binder = BinderState::new();
        binder.bind_source_file(&parser.arena, root);

        // Should have one symbol 'x'
        assert_eq!(binder.file_locals.len(), 1);
        assert!(binder.file_locals.has("x"));

        let x_id = binder.file_locals.get("x").unwrap();
        let x_sym = binder.symbols.get(x_id).unwrap();
        assert_eq!(x_sym.escaped_name, "x");
        assert!(x_sym.has_flags(symbol_flags::BLOCK_SCOPED_VARIABLE));
    }

    #[test]
    fn test_bind_function_declaration() {
        use crate::parser_impl::ParserState;

        let mut parser = ParserState::new(
            "test.ts".to_string(),
            "function add(a: number, b: number): number { return a + b; }".to_string(),
        );
        let root = parser.parse_source_file();

        let mut binder = BinderState::new();
        binder.bind_source_file(&parser.arena, root);

        // Should have one symbol 'add'
        assert!(binder.file_locals.has("add"));

        let add_id = binder.file_locals.get("add").unwrap();
        let add_sym = binder.symbols.get(add_id).unwrap();
        assert_eq!(add_sym.escaped_name, "add");
        assert!(add_sym.has_flags(symbol_flags::FUNCTION));
    }

    #[test]
    fn test_bind_class_declaration() {
        use crate::parser_impl::ParserState;

        let mut parser = ParserState::new(
            "test.ts".to_string(),
            "class Foo { x: number; bar() {} }".to_string(),
        );
        let root = parser.parse_source_file();

        let mut binder = BinderState::new();
        binder.bind_source_file(&parser.arena, root);

        // Should have one symbol 'Foo'
        assert!(binder.file_locals.has("Foo"));

        let foo_id = binder.file_locals.get("Foo").unwrap();
        let foo_sym = binder.symbols.get(foo_id).unwrap();
        assert_eq!(foo_sym.escaped_name, "Foo");
        assert!(foo_sym.has_flags(symbol_flags::CLASS));
    }

    #[test]
    fn test_bind_multiple_declarations() {
        use crate::parser_impl::ParserState;

        let mut parser = ParserState::new(
            "test.ts".to_string(),
            r#"
                const x = 1;
                function foo() {}
                class Bar {}
                interface IBaz {}
                type MyType = string;
            "#.to_string(),
        );
        let root = parser.parse_source_file();

        let mut binder = BinderState::new();
        binder.bind_source_file(&parser.arena, root);

        // Should have 5 symbols
        assert!(binder.file_locals.has("x"));
        assert!(binder.file_locals.has("foo"));
        assert!(binder.file_locals.has("Bar"));
        assert!(binder.file_locals.has("IBaz"));
        assert!(binder.file_locals.has("MyType"));

        // Verify symbol flags
        let x_id = binder.file_locals.get("x").unwrap();
        assert!(binder.symbols.get(x_id).unwrap().has_flags(symbol_flags::BLOCK_SCOPED_VARIABLE));

        let foo_id = binder.file_locals.get("foo").unwrap();
        assert!(binder.symbols.get(foo_id).unwrap().has_flags(symbol_flags::FUNCTION));

        let bar_id = binder.file_locals.get("Bar").unwrap();
        assert!(binder.symbols.get(bar_id).unwrap().has_flags(symbol_flags::CLASS));

        let ibaz_id = binder.file_locals.get("IBaz").unwrap();
        assert!(binder.symbols.get(ibaz_id).unwrap().has_flags(symbol_flags::INTERFACE));

        let mytype_id = binder.file_locals.get("MyType").unwrap();
        assert!(binder.symbols.get(mytype_id).unwrap().has_flags(symbol_flags::TYPE_ALIAS));
    }

    #[test]
    fn test_bind_namespace_declaration() {
        use crate::parser_impl::ParserState;

        let mut parser = ParserState::new(
            "test.ts".to_string(),
            r#"
                namespace MyNamespace {
                    export const x = 1;
                    export function foo() {}
                }
            "#.to_string(),
        );
        let root = parser.parse_source_file();

        let mut binder = BinderState::new();
        binder.bind_source_file(&parser.arena, root);

        // Should have namespace symbol 'MyNamespace'
        assert!(binder.file_locals.has("MyNamespace"));

        let ns_id = binder.file_locals.get("MyNamespace").unwrap();
        let ns_sym = binder.symbols.get(ns_id).unwrap();
        assert_eq!(ns_sym.escaped_name, "MyNamespace");
        assert!(ns_sym.has_any_flags(symbol_flags::MODULE));
    }

    #[test]
    fn test_interface_merging() {
        use crate::parser_impl::ParserState;

        let mut parser = ParserState::new(
            "test.ts".to_string(),
            r#"
                interface Foo {
                    x: number;
                }
                interface Foo {
                    y: string;
                }
            "#.to_string(),
        );
        let root = parser.parse_source_file();

        let mut binder = BinderState::new();
        binder.bind_source_file(&parser.arena, root);

        // Should have one symbol 'Foo' with two declarations
        assert!(binder.file_locals.has("Foo"));

        let foo_id = binder.file_locals.get("Foo").unwrap();
        let foo_sym = binder.symbols.get(foo_id).unwrap();
        assert_eq!(foo_sym.escaped_name, "Foo");
        assert!(foo_sym.has_flags(symbol_flags::INTERFACE));
        assert_eq!(foo_sym.declarations.len(), 2);
    }

    #[test]
    fn test_namespace_merging() {
        use crate::parser_impl::ParserState;

        let mut parser = ParserState::new(
            "test.ts".to_string(),
            r#"
                namespace NS {
                    export const a = 1;
                }
                namespace NS {
                    export const b = 2;
                }
            "#.to_string(),
        );
        let root = parser.parse_source_file();

        let mut binder = BinderState::new();
        binder.bind_source_file(&parser.arena, root);

        // Should have one symbol 'NS' with two declarations
        assert!(binder.file_locals.has("NS"));

        let ns_id = binder.file_locals.get("NS").unwrap();
        let ns_sym = binder.symbols.get(ns_id).unwrap();
        assert_eq!(ns_sym.escaped_name, "NS");
        assert!(ns_sym.has_any_flags(symbol_flags::MODULE));
        assert_eq!(ns_sym.declarations.len(), 2);
    }

    #[test]
    fn test_class_namespace_merging() {
        use crate::parser_impl::ParserState;

        let mut parser = ParserState::new(
            "test.ts".to_string(),
            r#"
                class Foo {
                    x: number;
                }
                namespace Foo {
                    export const bar = 1;
                }
            "#.to_string(),
        );
        let root = parser.parse_source_file();

        let mut binder = BinderState::new();
        binder.bind_source_file(&parser.arena, root);

        // Should have one symbol 'Foo' with both CLASS and MODULE flags
        assert!(binder.file_locals.has("Foo"));

        let foo_id = binder.file_locals.get("Foo").unwrap();
        let foo_sym = binder.symbols.get(foo_id).unwrap();
        assert_eq!(foo_sym.escaped_name, "Foo");
        assert!(foo_sym.has_flags(symbol_flags::CLASS));
        assert!(foo_sym.has_any_flags(symbol_flags::MODULE));
        assert_eq!(foo_sym.declarations.len(), 2);
    }

    #[test]
    fn test_flow_nodes_basic() {
        use crate::parser_impl::ParserState;

        let mut parser = ParserState::new(
            "test.ts".to_string(),
            "const x = 1;".to_string(),
        );
        let root = parser.parse_source_file();

        let mut binder = BinderState::new();
        binder.bind_source_file(&parser.arena, root);

        // Should have at least 2 flow nodes: unreachable + start
        assert!(binder.flow_nodes.len() >= 2);

        // Flow node 0 should be UNREACHABLE
        let unreachable = binder.flow_nodes.get(FlowNodeId(0)).unwrap();
        assert!(unreachable.has_flags(flow_flags::UNREACHABLE));

        // Flow node 1 should be START
        let start = binder.flow_nodes.get(FlowNodeId(1)).unwrap();
        assert!(start.has_flags(flow_flags::START));
    }

    #[test]
    fn test_flow_nodes_if_statement() {
        use crate::parser_impl::ParserState;

        let mut parser = ParserState::new(
            "test.ts".to_string(),
            r#"
                let x: number | string;
                if (typeof x === "number") {
                    const y = x;
                } else {
                    const z = x;
                }
            "#.to_string(),
        );
        let root = parser.parse_source_file();

        let mut binder = BinderState::new();
        binder.bind_source_file(&parser.arena, root);

        // Should have multiple flow nodes for the if statement:
        // - UNREACHABLE (0)
        // - START (1)
        // - TRUE_CONDITION (2) - for the if true branch
        // - BRANCH_LABEL (3) - merge point
        // - FALSE_CONDITION (4) - for the else branch
        assert!(binder.flow_nodes.len() >= 5);

        // Check that we have TRUE_CONDITION and FALSE_CONDITION nodes
        let mut has_true = false;
        let mut has_false = false;
        let mut has_branch_label = false;

        for i in 0..binder.flow_nodes.len() {
            if let Some(flow) = binder.flow_nodes.get(FlowNodeId(i as u32)) {
                if flow.has_flags(flow_flags::TRUE_CONDITION) {
                    has_true = true;
                }
                if flow.has_flags(flow_flags::FALSE_CONDITION) {
                    has_false = true;
                }
                if flow.has_flags(flow_flags::BRANCH_LABEL) {
                    has_branch_label = true;
                }
            }
        }

        assert!(has_true, "Should have TRUE_CONDITION flow node");
        assert!(has_false, "Should have FALSE_CONDITION flow node");
        assert!(has_branch_label, "Should have BRANCH_LABEL flow node");
    }

    #[test]
    fn test_flow_nodes_while_statement() {
        use crate::parser_impl::ParserState;

        let mut parser = ParserState::new(
            "test.ts".to_string(),
            r#"
                let i = 0;
                while (i < 10) {
                    i = i + 1;
                }
            "#.to_string(),
        );
        let root = parser.parse_source_file();

        let mut binder = BinderState::new();
        binder.bind_source_file(&parser.arena, root);

        // Should have flow nodes for the while loop:
        // - UNREACHABLE
        // - START
        // - LOOP_LABEL
        // - TRUE_CONDITION
        // - FALSE_CONDITION

        let mut has_loop_label = false;
        let mut has_true = false;
        let mut has_false = false;

        for i in 0..binder.flow_nodes.len() {
            if let Some(flow) = binder.flow_nodes.get(FlowNodeId(i as u32)) {
                if flow.has_flags(flow_flags::LOOP_LABEL) {
                    has_loop_label = true;
                }
                if flow.has_flags(flow_flags::TRUE_CONDITION) {
                    has_true = true;
                }
                if flow.has_flags(flow_flags::FALSE_CONDITION) {
                    has_false = true;
                }
            }
        }

        assert!(has_loop_label, "Should have LOOP_LABEL flow node");
        assert!(has_true, "Should have TRUE_CONDITION flow node");
        assert!(has_false, "Should have FALSE_CONDITION flow node");
    }
}
