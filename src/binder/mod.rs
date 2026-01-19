//! TypeScript Binder Implementation
//!
//! The binder is responsible for:
//! - Walking the AST and creating symbols for declarations
//! - Building scope chains for variable resolution
//! - Creating the control flow graph for type narrowing
//! - Handling hoisting of var declarations and function declarations
//!
//! # Architecture
//!
//! The binder processes the AST in a single pass, maintaining:
//! - A symbol arena for efficient symbol storage
//! - A scope chain for tracking lexical scopes
//! - A flow graph builder for control flow analysis
//! - A container stack for tracking function/block containers

pub mod symbols;
pub mod scope;
pub mod flow;
pub mod container;

pub use symbols::{Symbol, SymbolFlags, SymbolId, SymbolTable, SymbolArena, DeclarationId};
pub use scope::{Scope, ScopeChain, ScopeKind};
pub use flow::{FlowGraphBuilder, FlowNode, FlowNodeId, FlowFlags, NarrowingKind, UNREACHABLE_FLOW, START_FLOW};
pub use container::{ContainerInfo, ContainerStack, ContainerFlags, NodeFlags, HoistingInfo};

/// Represents different kinds of declarations
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum DeclarationKind {
    Variable,
    Function,
    Class,
    Interface,
    TypeAlias,
    Enum,
    EnumMember,
    Module,
    Parameter,
    Property,
    Method,
    GetAccessor,
    SetAccessor,
    TypeParameter,
    ImportAlias,
    ExportAssignment,
}

/// A declaration that the binder processes
#[derive(Debug, Clone)]
pub struct Declaration {
    pub id: DeclarationId,
    pub kind: DeclarationKind,
    pub name: Option<String>,
    pub flags: NodeFlags,
}

/// Diagnostic message from the binder
#[derive(Debug, Clone)]
pub struct BinderDiagnostic {
    pub message: String,
    pub node_id: DeclarationId,
    pub code: u32,
}

/// Diagnostic codes
pub mod diagnostic_codes {
    pub const DUPLICATE_IDENTIFIER: u32 = 2300;
    pub const BLOCK_SCOPED_REDECLARATION: u32 = 2451;
    pub const CANNOT_REDECLARE: u32 = 2451;
    pub const INVALID_USE_OF_THIS: u32 = 2332;
    pub const INVALID_USE_OF_ARGUMENTS: u32 = 2304;
}

/// The main binder struct
#[derive(Debug)]
pub struct Binder {
    /// Storage for all symbols
    pub symbol_arena: SymbolArena,
    /// The scope chain
    pub scope_chain: ScopeChain,
    /// Control flow graph builder
    pub flow_builder: FlowGraphBuilder,
    /// Container stack
    pub container_stack: ContainerStack,
    /// Diagnostics collected during binding
    pub diagnostics: Vec<BinderDiagnostic>,
    /// Whether the file is a module
    pub is_external_module: bool,
    /// Current node being bound
    current_node: Option<DeclarationId>,
    /// Hoisting info per function scope
    hoisting_info_stack: Vec<HoistingInfo>,
}

impl Binder {
    pub fn new() -> Self {
        Binder {
            symbol_arena: SymbolArena::new(),
            scope_chain: ScopeChain::new(),
            flow_builder: FlowGraphBuilder::new(),
            container_stack: ContainerStack::new(),
            diagnostics: Vec::new(),
            is_external_module: false,
            current_node: None,
            hoisting_info_stack: vec![HoistingInfo::new()],
        }
    }

    /// Create a new symbol with the given name and flags
    pub fn create_symbol(&mut self, name: impl Into<String>, flags: SymbolFlags) -> SymbolId {
        self.symbol_arena.create_symbol(name, flags)
    }

    /// Get exclusion flags for a given symbol flags
    fn get_exclusion_flags(flags: SymbolFlags) -> SymbolFlags {
        if flags.contains(SymbolFlags::BLOCK_SCOPED_VARIABLE) {
            SymbolFlags::BLOCK_SCOPED_VARIABLE_EXCLUDES
        } else if flags.contains(SymbolFlags::FUNCTION_SCOPED_VARIABLE) {
            SymbolFlags::FUNCTION_SCOPED_VARIABLE_EXCLUDES
        } else if flags.contains(SymbolFlags::FUNCTION) {
            SymbolFlags::FUNCTION_EXCLUDES
        } else if flags.contains(SymbolFlags::CLASS) {
            SymbolFlags::CLASS_EXCLUDES
        } else if flags.contains(SymbolFlags::INTERFACE) {
            SymbolFlags::INTERFACE_EXCLUDES
        } else if flags.contains(SymbolFlags::REGULAR_ENUM) {
            SymbolFlags::REGULAR_ENUM_EXCLUDES
        } else if flags.contains(SymbolFlags::CONST_ENUM) {
            SymbolFlags::CONST_ENUM_EXCLUDES
        } else if flags.contains(SymbolFlags::TYPE_ALIAS) {
            SymbolFlags::TYPE_ALIAS_EXCLUDES
        } else if flags.contains(SymbolFlags::ALIAS) {
            SymbolFlags::ALIAS_EXCLUDES
        } else {
            SymbolFlags::NONE
        }
    }

    /// Declare a symbol in the current scope
    pub fn declare_symbol(
        &mut self,
        name: &str,
        flags: SymbolFlags,
        declaration_id: DeclarationId,
    ) -> Result<SymbolId, BinderDiagnostic> {
        // Check for existing symbol
        if let Some(existing_id) = self.scope_chain.current_scope().get_local(name) {
            let existing = self.symbol_arena.get(existing_id).unwrap();
            let exclusion_flags = Self::get_exclusion_flags(flags);

            // Check if this is a conflict
            if existing.flags.intersects(exclusion_flags) {
                return Err(BinderDiagnostic {
                    message: format!("Cannot redeclare block-scoped variable '{}'", name),
                    node_id: declaration_id,
                    code: diagnostic_codes::BLOCK_SCOPED_REDECLARATION,
                });
            }

            // Merge the symbol
            let existing = self.symbol_arena.get_mut(existing_id).unwrap();
            existing.flags |= flags;
            existing.add_declaration(declaration_id);
            return Ok(existing_id);
        }

        // Create new symbol
        let symbol_id = self.create_symbol(name, flags);
        let symbol = self.symbol_arena.get_mut(symbol_id).unwrap();
        symbol.add_declaration(declaration_id);

        // Add to appropriate scope
        if flags.contains(SymbolFlags::FUNCTION_SCOPED_VARIABLE) {
            self.scope_chain.add_function_scoped(name.to_string(), symbol_id);
        } else {
            self.scope_chain.add_to_current_scope(name.to_string(), symbol_id);
        }

        Ok(symbol_id)
    }

    /// Bind a variable declaration
    pub fn bind_variable_declaration(
        &mut self,
        name: &str,
        declaration_id: DeclarationId,
        is_const: bool,
        is_let: bool,
    ) -> Result<SymbolId, BinderDiagnostic> {
        let flags = if is_const || is_let {
            SymbolFlags::BLOCK_SCOPED_VARIABLE
        } else {
            SymbolFlags::FUNCTION_SCOPED_VARIABLE
        };

        let result = self.declare_symbol(name, flags, declaration_id);

        // Track for hoisting if it's a var declaration
        if !is_const && !is_let {
            if let Ok(symbol_id) = result {
                if let Some(hoisting) = self.hoisting_info_stack.last_mut() {
                    hoisting.add_hoisted_var(name.to_string(), symbol_id);
                }
            }
        }

        // Create flow assignment
        if result.is_ok() {
            self.flow_builder.create_assignment(declaration_id);
        }

        result
    }

    /// Bind a function declaration
    pub fn bind_function_declaration(
        &mut self,
        name: &str,
        declaration_id: DeclarationId,
    ) -> Result<SymbolId, BinderDiagnostic> {
        let result = self.declare_symbol(name, SymbolFlags::FUNCTION, declaration_id);

        // Track for hoisting
        if let Ok(symbol_id) = result {
            if let Some(hoisting) = self.hoisting_info_stack.last_mut() {
                hoisting.add_hoisted_function(name.to_string(), symbol_id);
            }
        }

        result
    }

    /// Bind a class declaration
    pub fn bind_class_declaration(
        &mut self,
        name: &str,
        declaration_id: DeclarationId,
    ) -> Result<SymbolId, BinderDiagnostic> {
        self.declare_symbol(name, SymbolFlags::CLASS, declaration_id)
    }

    /// Bind an interface declaration
    pub fn bind_interface_declaration(
        &mut self,
        name: &str,
        declaration_id: DeclarationId,
    ) -> Result<SymbolId, BinderDiagnostic> {
        self.declare_symbol(name, SymbolFlags::INTERFACE, declaration_id)
    }

    /// Bind a type alias declaration
    pub fn bind_type_alias_declaration(
        &mut self,
        name: &str,
        declaration_id: DeclarationId,
    ) -> Result<SymbolId, BinderDiagnostic> {
        self.declare_symbol(name, SymbolFlags::TYPE_ALIAS, declaration_id)
    }

    /// Bind an enum declaration
    pub fn bind_enum_declaration(
        &mut self,
        name: &str,
        declaration_id: DeclarationId,
        is_const: bool,
    ) -> Result<SymbolId, BinderDiagnostic> {
        let flags = if is_const {
            SymbolFlags::CONST_ENUM
        } else {
            SymbolFlags::REGULAR_ENUM
        };
        self.declare_symbol(name, flags, declaration_id)
    }

    /// Bind a namespace declaration
    pub fn bind_namespace_declaration(
        &mut self,
        name: &str,
        declaration_id: DeclarationId,
        is_value: bool,
    ) -> Result<SymbolId, BinderDiagnostic> {
        let flags = if is_value {
            SymbolFlags::VALUE_MODULE
        } else {
            SymbolFlags::NAMESPACE_MODULE
        };
        self.declare_symbol(name, flags, declaration_id)
    }

    /// Enter a new scope
    pub fn enter_scope(&mut self, kind: ScopeKind) {
        self.scope_chain.push_scope(kind);

        // Track new hoisting context for function scopes
        if kind.is_function_scope_container() {
            self.hoisting_info_stack.push(HoistingInfo::new());
        }
    }

    /// Exit the current scope
    pub fn exit_scope(&mut self) -> Option<HoistingInfo> {
        let kind = self.scope_chain.current_scope().kind;
        self.scope_chain.pop_scope();

        // Pop hoisting context for function scopes
        if kind.is_function_scope_container() {
            return self.hoisting_info_stack.pop();
        }

        None
    }

    /// Enter a container
    pub fn enter_container(&mut self, node_id: DeclarationId, flags: ContainerFlags) {
        let mut info = ContainerInfo::new(node_id, flags);
        info.parent = self.container_stack.current_container().map(|c| c.node_id);
        info.strict_mode = self.container_stack.in_strict_mode();
        self.container_stack.push_container(info);
    }

    /// Exit the current container
    pub fn exit_container(&mut self) -> Option<ContainerInfo> {
        self.container_stack.pop_container()
    }

    /// Set strict mode
    pub fn set_strict_mode(&mut self, strict: bool) {
        self.scope_chain.set_strict_mode(strict);
        self.container_stack.set_strict_mode(strict);
    }

    /// Look up a symbol by name
    pub fn lookup_symbol(&self, name: &str) -> Option<(SymbolId, u32)> {
        self.scope_chain.lookup(name)
    }

    /// Get a symbol by ID
    pub fn get_symbol(&self, id: SymbolId) -> Option<&Symbol> {
        self.symbol_arena.get(id)
    }

    /// Get the current scope
    pub fn current_scope(&self) -> &Scope {
        self.scope_chain.current_scope()
    }

    /// Check if we're in strict mode
    pub fn in_strict_mode(&self) -> bool {
        self.scope_chain.in_strict_mode()
    }

    /// Bind a control flow branch
    pub fn bind_branch(&mut self, condition_node: DeclarationId) -> (FlowNodeId, FlowNodeId) {
        let true_branch = self.flow_builder.create_condition(condition_node, true);
        let false_branch = self.flow_builder.create_condition(condition_node, false);
        (true_branch, false_branch)
    }

    /// Create a branch label for joining control flow
    pub fn create_branch_label(&mut self) -> FlowNodeId {
        self.flow_builder.create_branch_label()
    }

    /// Create a loop label
    pub fn create_loop_label(&mut self) -> FlowNodeId {
        self.flow_builder.create_loop_label()
    }

    /// Add antecedent to a label
    pub fn add_antecedent(&mut self, label: FlowNodeId, antecedent: FlowNodeId) {
        self.flow_builder.add_antecedent(label, antecedent);
    }

    /// Finish a flow label
    pub fn finish_flow_label(&mut self, label: FlowNodeId) -> FlowNodeId {
        self.flow_builder.finish_flow_label(label)
    }

    /// Get current flow
    pub fn current_flow(&self) -> FlowNodeId {
        self.flow_builder.current_flow()
    }

    /// Set current flow
    pub fn set_current_flow(&mut self, flow: FlowNodeId) {
        self.flow_builder.set_current_flow(flow);
    }

    /// Bind a break statement
    pub fn bind_break(&mut self, label: Option<&str>) {
        self.flow_builder.bind_break(label);
    }

    /// Bind a continue statement
    pub fn bind_continue(&mut self, label: Option<&str>) {
        self.flow_builder.bind_continue(label);
    }

    /// Bind a return statement
    pub fn bind_return(&mut self) {
        self.flow_builder.bind_return();
    }

    /// Bind a throw statement
    pub fn bind_throw(&mut self) {
        self.flow_builder.bind_throw();
    }

    /// Report a diagnostic
    pub fn report_error(&mut self, message: impl Into<String>, node_id: DeclarationId, code: u32) {
        self.diagnostics.push(BinderDiagnostic {
            message: message.into(),
            node_id,
            code,
        });
    }

    /// Check if there were any errors
    pub fn has_errors(&self) -> bool {
        !self.diagnostics.is_empty()
    }

    /// Get all diagnostics
    pub fn get_diagnostics(&self) -> &[BinderDiagnostic] {
        &self.diagnostics
    }
}

impl Default for Binder {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_binder_creation() {
        let binder = Binder::new();
        assert_eq!(binder.current_scope().kind, ScopeKind::Global);
        assert!(!binder.has_errors());
    }

    #[test]
    fn test_bind_variable() {
        let mut binder = Binder::new();

        let result = binder.bind_variable_declaration("x", 1, false, true);
        assert!(result.is_ok());

        let symbol_id = result.unwrap();
        let symbol = binder.get_symbol(symbol_id).unwrap();
        assert_eq!(symbol.name, "x");
        assert!(symbol.flags.contains(SymbolFlags::BLOCK_SCOPED_VARIABLE));
    }

    #[test]
    fn test_bind_var_hoisting() {
        let mut binder = Binder::new();

        // Enter function scope
        binder.enter_scope(ScopeKind::Function);

        // Enter block scope
        binder.enter_scope(ScopeKind::Block);

        // Declare var inside block
        let result = binder.bind_variable_declaration("x", 1, false, false);
        assert!(result.is_ok());

        // var should NOT be in block scope locals
        assert!(binder.current_scope().get_local("x").is_none());

        // Exit block
        binder.exit_scope();

        // var SHOULD be in function scope
        assert!(binder.current_scope().get_local("x").is_some());

        // Exit function and check hoisting info
        let hoisting = binder.exit_scope();
        assert!(hoisting.is_some());
        assert_eq!(hoisting.unwrap().hoisted_vars.len(), 1);
    }

    #[test]
    fn test_bind_function() {
        let mut binder = Binder::new();

        let result = binder.bind_function_declaration("foo", 1);
        assert!(result.is_ok());

        let symbol = binder.get_symbol(result.unwrap()).unwrap();
        assert_eq!(symbol.name, "foo");
        assert!(symbol.flags.contains(SymbolFlags::FUNCTION));
    }

    #[test]
    fn test_duplicate_let_error() {
        let mut binder = Binder::new();

        let result1 = binder.bind_variable_declaration("x", 1, false, true);
        assert!(result1.is_ok());

        let result2 = binder.bind_variable_declaration("x", 2, false, true);
        assert!(result2.is_err());
    }

    #[test]
    fn test_interface_merging() {
        let mut binder = Binder::new();

        let result1 = binder.bind_interface_declaration("Foo", 1);
        assert!(result1.is_ok());

        // Interfaces can merge
        let result2 = binder.bind_interface_declaration("Foo", 2);
        assert!(result2.is_ok());

        // Should be same symbol
        assert_eq!(result1.unwrap(), result2.unwrap());
    }

    #[test]
    fn test_scope_lookup() {
        let mut binder = Binder::new();

        // Declare in global
        binder.bind_variable_declaration("global", 1, false, true).unwrap();

        // Enter function
        binder.enter_scope(ScopeKind::Function);
        binder.bind_variable_declaration("local", 2, false, true).unwrap();

        // Can find both
        assert!(binder.lookup_symbol("global").is_some());
        assert!(binder.lookup_symbol("local").is_some());

        // Exit function
        binder.exit_scope();

        // Can only find global
        assert!(binder.lookup_symbol("global").is_some());
        assert!(binder.lookup_symbol("local").is_none());
    }

    #[test]
    fn test_strict_mode() {
        let mut binder = Binder::new();

        assert!(!binder.in_strict_mode());

        binder.enter_scope(ScopeKind::Module);
        binder.set_strict_mode(true);
        assert!(binder.in_strict_mode());

        binder.enter_scope(ScopeKind::Function);
        // Should inherit strict mode
        assert!(binder.in_strict_mode());
    }

    #[test]
    fn test_flow_graph_assignment() {
        let mut binder = Binder::new();

        binder.bind_variable_declaration("x", 1, false, true).unwrap();

        // Check that flow was updated
        assert_ne!(binder.current_flow(), START_FLOW);
    }

    #[test]
    fn test_bind_class() {
        let mut binder = Binder::new();

        let result = binder.bind_class_declaration("MyClass", 1);
        assert!(result.is_ok());

        let symbol = binder.get_symbol(result.unwrap()).unwrap();
        assert!(symbol.flags.contains(SymbolFlags::CLASS));
    }

    #[test]
    fn test_bind_enum() {
        let mut binder = Binder::new();

        let result = binder.bind_enum_declaration("Colors", 1, false);
        assert!(result.is_ok());

        let symbol = binder.get_symbol(result.unwrap()).unwrap();
        assert!(symbol.flags.contains(SymbolFlags::REGULAR_ENUM));
    }

    #[test]
    fn test_bind_const_enum() {
        let mut binder = Binder::new();

        let result = binder.bind_enum_declaration("Direction", 1, true);
        assert!(result.is_ok());

        let symbol = binder.get_symbol(result.unwrap()).unwrap();
        assert!(symbol.flags.contains(SymbolFlags::CONST_ENUM));
    }

    #[test]
    fn test_container_tracking() {
        let mut binder = Binder::new();

        binder.enter_container(
            1,
            ContainerFlags::IS_CONTAINER | ContainerFlags::IS_FUNCTION_LIKE
        );

        assert!(binder.container_stack.in_function());

        binder.exit_container();
        assert!(!binder.container_stack.in_function());
    }

    #[test]
    fn test_control_flow_branching() {
        let mut binder = Binder::new();

        let (true_branch, false_branch) = binder.bind_branch(100);
        assert_ne!(true_branch, false_branch);

        let join_label = binder.create_branch_label();
        binder.add_antecedent(join_label, true_branch);
        binder.add_antecedent(join_label, false_branch);

        let final_flow = binder.finish_flow_label(join_label);
        assert_eq!(final_flow, join_label);
    }
}
