//! TypeScript Binder Implementation
//!
//! The binder is responsible for creating the symbol table, which maps names
//! to their declarations. It also establishes the scope chain and handles
//! TypeScript-specific binding concerns like:
//! - Module and namespace merging
//! - Class member binding
//! - Export/import binding
//! - Block scoping for let/const

use crate::arena::Arena;
use crate::scope::{ScopeChain, ScopeKind};
use crate::symbols::{
    internal_symbol_names, NodeId, Symbol, SymbolFlags, SymbolId, SymbolTable,
};
use crate::syntax_kind::SyntaxKind;

/// Represents a node in the AST for binding purposes
/// This is a simplified representation; in practice this would come from the parser
#[derive(Debug, Clone)]
pub struct Node {
    /// Unique identifier for this node
    pub id: NodeId,
    /// The kind of syntax this node represents
    pub kind: SyntaxKind,
    /// The name of this node (for declarations)
    pub name: Option<String>,
    /// Child nodes
    pub children: Vec<NodeId>,
    /// Parent node
    pub parent: Option<NodeId>,
    /// Modifier flags (export, const, etc.)
    pub modifiers: ModifierFlags,
}

/// Modifier flags for declarations
#[derive(Debug, Clone, Copy, Default)]
pub struct ModifierFlags(u32);

impl ModifierFlags {
    pub const NONE: ModifierFlags = ModifierFlags(0);
    pub const EXPORT: ModifierFlags = ModifierFlags(1 << 0);
    pub const AMBIENT: ModifierFlags = ModifierFlags(1 << 1);
    pub const PUBLIC: ModifierFlags = ModifierFlags(1 << 2);
    pub const PRIVATE: ModifierFlags = ModifierFlags(1 << 3);
    pub const PROTECTED: ModifierFlags = ModifierFlags(1 << 4);
    pub const STATIC: ModifierFlags = ModifierFlags(1 << 5);
    pub const READONLY: ModifierFlags = ModifierFlags(1 << 6);
    pub const ABSTRACT: ModifierFlags = ModifierFlags(1 << 7);
    pub const ASYNC: ModifierFlags = ModifierFlags(1 << 8);
    pub const DEFAULT: ModifierFlags = ModifierFlags(1 << 9);
    pub const CONST: ModifierFlags = ModifierFlags(1 << 10);

    pub fn contains(&self, other: ModifierFlags) -> bool {
        (self.0 & other.0) == other.0
    }

    pub fn intersects(&self, other: ModifierFlags) -> bool {
        (self.0 & other.0) != 0
    }
}

impl std::ops::BitOr for ModifierFlags {
    type Output = Self;
    fn bitor(self, rhs: Self) -> Self::Output {
        ModifierFlags(self.0 | rhs.0)
    }
}

/// Result of binding, containing diagnostics and symbol information
#[derive(Debug)]
pub struct BindingResult {
    /// Any diagnostics produced during binding
    pub diagnostics: Vec<BindingDiagnostic>,
    /// The root symbol table (globals/exports)
    pub globals: SymbolTable,
    /// Number of symbols created
    pub symbol_count: usize,
}

/// A diagnostic produced during binding
#[derive(Debug)]
pub struct BindingDiagnostic {
    /// The node ID where the error occurred
    pub node_id: NodeId,
    /// The diagnostic message
    pub message: String,
    /// The diagnostic code
    pub code: u32,
}

/// Diagnostic codes
pub mod diagnostic_codes {
    pub const DUPLICATE_IDENTIFIER: u32 = 2300;
    pub const CANNOT_REDECLARE_BLOCK_SCOPED: u32 = 2451;
    pub const ENUM_MERGE_ERROR: u32 = 2567;
    pub const MULTIPLE_DEFAULT_EXPORTS: u32 = 2528;
}

/// Represents which symbol table to use for a declaration
#[derive(Clone, Copy)]
enum SymbolTableLocation {
    Exports,
    BlockScope,
    CurrentScope,
}

/// The binder creates symbol tables by walking the AST
pub struct Binder {
    /// Arena for symbol storage
    symbol_arena: Arena<Symbol>,
    /// The scope chain
    scopes: ScopeChain,
    /// All nodes (would come from parser in practice)
    nodes: Vec<Node>,
    /// Map from node ID to symbol ID
    node_symbols: Vec<Option<SymbolId>>,
    /// Diagnostics produced during binding
    diagnostics: Vec<BindingDiagnostic>,
    /// Whether we're in strict mode
    in_strict_mode: bool,
    /// Whether the file is an external module
    is_external_module: bool,
    /// Classifiable names (for semantic classification)
    classifiable_names: Vec<String>,
    /// Current container symbol (for declaring members)
    container_symbol: Option<SymbolId>,
    /// Current module exports
    exports: SymbolTable,
}

impl Binder {
    /// Create a new binder
    pub fn new() -> Self {
        Binder {
            symbol_arena: Arena::new(),
            scopes: ScopeChain::new(),
            nodes: Vec::new(),
            node_symbols: Vec::new(),
            diagnostics: Vec::new(),
            in_strict_mode: false,
            is_external_module: false,
            classifiable_names: Vec::new(),
            container_symbol: None,
            exports: SymbolTable::new(),
        }
    }

    /// Bind a source file (the main entry point)
    pub fn bind_source_file(&mut self, root: &Node, nodes: &[Node]) -> BindingResult {
        // Store all nodes
        self.nodes = nodes.to_vec();
        self.node_symbols = vec![None; nodes.len()];

        // Determine if this is an external module
        self.is_external_module = self.has_external_module_indicator(root);

        // External modules are automatically in strict mode
        if self.is_external_module {
            self.in_strict_mode = true;
        }

        // Bind the source file node (this will create the global scope)
        self.bind_source_file_node(root);

        // Collect results - get globals from the first scope (which is the source file scope)
        let globals = self.scopes.get_scope(0)
            .map(|s| s.locals.clone())
            .unwrap_or_default();

        BindingResult {
            diagnostics: std::mem::take(&mut self.diagnostics),
            globals,
            symbol_count: self.symbol_arena.len(),
        }
    }

    /// Bind the source file node specifically (doesn't pop the scope)
    fn bind_source_file_node(&mut self, node: &Node) {
        // Push the global/module scope for the source file
        let scope_kind = if self.is_external_module {
            ScopeKind::Module
        } else {
            ScopeKind::Global
        };
        self.scopes.push_scope(scope_kind);

        if self.in_strict_mode {
            self.scopes.set_strict_mode(true);
        }

        // Bind all children
        self.bind_children(node);

        // Note: We don't pop the scope here so we can extract globals
    }

    /// Check if a source file has an external module indicator
    fn has_external_module_indicator(&self, root: &Node) -> bool {
        // In a real implementation, we'd check for import/export statements
        for &child_id in &root.children {
            if let Some(child) = self.get_node(child_id) {
                match child.kind {
                    SyntaxKind::ImportDeclaration
                    | SyntaxKind::ImportEqualsDeclaration
                    | SyntaxKind::ExportDeclaration
                    | SyntaxKind::ExportAssignment => return true,
                    _ => {}
                }
            }
        }
        false
    }

    /// Get a node by ID
    fn get_node(&self, id: NodeId) -> Option<&Node> {
        self.nodes.get(id as usize)
    }

    /// Bind a single node and its children
    fn bind_node(&mut self, node_id: NodeId) {
        let node = match self.get_node(node_id) {
            Some(n) => n.clone(),
            None => return,
        };

        // Check for "use strict" directive
        if node.kind == SyntaxKind::ExpressionStatement {
            // In real implementation, check for string literal "use strict"
        }

        // Get the container flags for this node
        let container_flags = self.get_container_flags(&node);

        // If this is a container, we need to manage the scope
        if container_flags.is_container {
            self.bind_container(&node, container_flags);
        } else if container_flags.is_block_scoped_container {
            self.bind_block_scoped_container(&node);
        } else {
            self.bind_children(&node);
        }

        // Bind the declaration if applicable
        self.bind_declaration(&node);
    }

    /// Get container flags for a node
    fn get_container_flags(&self, node: &Node) -> ContainerFlags {
        match node.kind {
            // Full containers (with locals)
            SyntaxKind::SourceFile => ContainerFlags {
                is_container: true,
                is_block_scoped_container: true,
                has_locals: true,
                ..Default::default()
            },
            SyntaxKind::ModuleDeclaration | SyntaxKind::ModuleBlock => ContainerFlags {
                is_container: true,
                is_block_scoped_container: true,
                has_locals: true,
                ..Default::default()
            },
            SyntaxKind::ClassDeclaration | SyntaxKind::ClassExpression => ContainerFlags {
                is_container: true,
                is_block_scoped_container: true,
                has_locals: true,
                ..Default::default()
            },
            SyntaxKind::InterfaceDeclaration => ContainerFlags {
                is_container: true,
                is_interface: true,
                ..Default::default()
            },
            SyntaxKind::TypeLiteral => ContainerFlags {
                is_container: true,
                ..Default::default()
            },
            SyntaxKind::ObjectLiteralExpression => ContainerFlags {
                is_container: true,
                ..Default::default()
            },
            SyntaxKind::EnumDeclaration => ContainerFlags {
                is_container: true,
                is_block_scoped_container: true,
                has_locals: true,
                ..Default::default()
            },

            // Function-like containers
            SyntaxKind::FunctionDeclaration
            | SyntaxKind::FunctionExpression
            | SyntaxKind::ArrowFunction => ContainerFlags {
                is_container: true,
                is_block_scoped_container: true,
                is_control_flow_container: true,
                is_function_like: true,
                has_locals: true,
                ..Default::default()
            },
            SyntaxKind::MethodDeclaration => ContainerFlags {
                is_container: true,
                is_block_scoped_container: true,
                is_control_flow_container: true,
                is_function_like: true,
                has_locals: true,
                is_object_literal_or_class_expression_method: true,
                ..Default::default()
            },
            SyntaxKind::Constructor
            | SyntaxKind::GetAccessor
            | SyntaxKind::SetAccessor => ContainerFlags {
                is_container: true,
                is_block_scoped_container: true,
                is_control_flow_container: true,
                is_function_like: true,
                has_locals: true,
                ..Default::default()
            },

            // Block-scoped containers only
            SyntaxKind::Block => ContainerFlags {
                is_block_scoped_container: true,
                ..Default::default()
            },
            SyntaxKind::CatchClause => ContainerFlags {
                is_block_scoped_container: true,
                has_locals: true,
                ..Default::default()
            },
            SyntaxKind::ForStatement
            | SyntaxKind::ForInStatement
            | SyntaxKind::ForOfStatement => ContainerFlags {
                is_block_scoped_container: true,
                is_control_flow_container: false,
                ..Default::default()
            },
            SyntaxKind::CaseBlock => ContainerFlags {
                is_block_scoped_container: true,
                ..Default::default()
            },

            _ => ContainerFlags::default(),
        }
    }

    /// Bind a container node
    fn bind_container(&mut self, node: &Node, _flags: ContainerFlags) {
        let scope_kind = self.get_scope_kind_for_node(node);
        let _scope_id = self.scopes.push_scope(scope_kind);

        // Save the previous container symbol
        let saved_container = self.container_symbol.take();

        // Bind children
        self.bind_children(node);

        // Restore
        self.container_symbol = saved_container;
        self.scopes.pop_scope();
    }

    /// Bind a block-scoped container
    fn bind_block_scoped_container(&mut self, node: &Node) {
        let scope_kind = self.get_scope_kind_for_node(node);
        self.scopes.push_scope(scope_kind);
        self.bind_children(node);
        self.scopes.pop_scope();
    }

    /// Get the scope kind for a node
    fn get_scope_kind_for_node(&self, node: &Node) -> ScopeKind {
        match node.kind {
            SyntaxKind::SourceFile => {
                if self.is_external_module {
                    ScopeKind::Module
                } else {
                    ScopeKind::Global
                }
            }
            SyntaxKind::ModuleDeclaration | SyntaxKind::ModuleBlock => ScopeKind::Namespace,
            SyntaxKind::FunctionDeclaration
            | SyntaxKind::FunctionExpression
            | SyntaxKind::ArrowFunction
            | SyntaxKind::MethodDeclaration
            | SyntaxKind::Constructor
            | SyntaxKind::GetAccessor
            | SyntaxKind::SetAccessor => ScopeKind::Function,
            SyntaxKind::ClassDeclaration | SyntaxKind::ClassExpression => ScopeKind::Class,
            SyntaxKind::InterfaceDeclaration => ScopeKind::Interface,
            SyntaxKind::EnumDeclaration => ScopeKind::Enum,
            SyntaxKind::TypeLiteral => ScopeKind::TypeLiteral,
            SyntaxKind::ObjectLiteralExpression => ScopeKind::ObjectLiteral,
            SyntaxKind::Block | SyntaxKind::CaseBlock => ScopeKind::Block,
            SyntaxKind::CatchClause => ScopeKind::Catch,
            SyntaxKind::ForStatement
            | SyntaxKind::ForInStatement
            | SyntaxKind::ForOfStatement => ScopeKind::ForLoop,
            _ => ScopeKind::Block,
        }
    }

    /// Bind all children of a node
    fn bind_children(&mut self, node: &Node) {
        for &child_id in &node.children {
            self.bind_node(child_id);
        }
    }

    /// Bind a declaration node
    fn bind_declaration(&mut self, node: &Node) {
        if !node.kind.is_declaration() {
            return;
        }

        let (symbol_flags, symbol_excludes) = self.get_symbol_flags_for_node(node);

        if symbol_flags == SymbolFlags::NONE {
            return;
        }

        // Get the declaration name
        let name = match &node.name {
            Some(n) => n.clone(),
            None => self.get_declaration_name(node),
        };

        // Declare the symbol
        self.declare_symbol(node, &name, symbol_flags, symbol_excludes);
    }

    /// Get symbol flags for a declaration node
    fn get_symbol_flags_for_node(&self, node: &Node) -> (SymbolFlags, SymbolFlags) {
        match node.kind {
            SyntaxKind::VariableDeclaration => {
                // Check if it's a const or let declaration
                if node.modifiers.contains(ModifierFlags::CONST) {
                    (
                        SymbolFlags::BLOCK_SCOPED_VARIABLE,
                        SymbolFlags::BLOCK_SCOPED_VARIABLE_EXCLUDES,
                    )
                } else {
                    // Assume let for now (would need to check parent VariableDeclarationList)
                    (
                        SymbolFlags::BLOCK_SCOPED_VARIABLE,
                        SymbolFlags::BLOCK_SCOPED_VARIABLE_EXCLUDES,
                    )
                }
            }
            SyntaxKind::Parameter => (SymbolFlags::FUNCTION_SCOPED_VARIABLE, SymbolFlags::PARAMETER_EXCLUDES),
            SyntaxKind::FunctionDeclaration => (SymbolFlags::FUNCTION, SymbolFlags::FUNCTION_EXCLUDES),
            SyntaxKind::ClassDeclaration => (SymbolFlags::CLASS, SymbolFlags::CLASS_EXCLUDES),
            SyntaxKind::InterfaceDeclaration => (SymbolFlags::INTERFACE, SymbolFlags::INTERFACE_EXCLUDES),
            SyntaxKind::TypeAliasDeclaration => (SymbolFlags::TYPE_ALIAS, SymbolFlags::TYPE_ALIAS_EXCLUDES),
            SyntaxKind::EnumDeclaration => {
                if node.modifiers.contains(ModifierFlags::CONST) {
                    (SymbolFlags::CONST_ENUM, SymbolFlags::CONST_ENUM_EXCLUDES)
                } else {
                    (SymbolFlags::REGULAR_ENUM, SymbolFlags::REGULAR_ENUM_EXCLUDES)
                }
            }
            SyntaxKind::ModuleDeclaration => {
                // Determine if it's a value module or namespace module
                (SymbolFlags::VALUE_MODULE, SymbolFlags::VALUE_MODULE_EXCLUDES)
            }
            SyntaxKind::ImportEqualsDeclaration | SyntaxKind::ImportSpecifier => {
                (SymbolFlags::ALIAS, SymbolFlags::ALIAS_EXCLUDES)
            }
            SyntaxKind::ExportSpecifier => (SymbolFlags::ALIAS, SymbolFlags::ALIAS_EXCLUDES),
            SyntaxKind::NamespaceImport => (SymbolFlags::ALIAS, SymbolFlags::ALIAS_EXCLUDES),
            SyntaxKind::PropertyDeclaration => (SymbolFlags::PROPERTY, SymbolFlags::PROPERTY_EXCLUDES),
            SyntaxKind::MethodDeclaration => (SymbolFlags::METHOD, SymbolFlags::METHOD_EXCLUDES),
            SyntaxKind::Constructor => (SymbolFlags::CONSTRUCTOR, SymbolFlags::METHOD_EXCLUDES),
            SyntaxKind::GetAccessor => (SymbolFlags::GET_ACCESSOR, SymbolFlags::GET_ACCESSOR_EXCLUDES),
            SyntaxKind::SetAccessor => (SymbolFlags::SET_ACCESSOR, SymbolFlags::SET_ACCESSOR_EXCLUDES),
            SyntaxKind::TypeParameter => (SymbolFlags::TYPE_PARAMETER, SymbolFlags::TYPE_PARAMETER_EXCLUDES),
            SyntaxKind::EnumMember => (SymbolFlags::ENUM_MEMBER, SymbolFlags::ENUM_MEMBER_EXCLUDES),
            SyntaxKind::BindingElement => (SymbolFlags::BLOCK_SCOPED_VARIABLE, SymbolFlags::BLOCK_SCOPED_VARIABLE_EXCLUDES),
            _ => (SymbolFlags::NONE, SymbolFlags::NONE),
        }
    }

    /// Get the declaration name for a node
    fn get_declaration_name(&self, node: &Node) -> String {
        match node.kind {
            SyntaxKind::Constructor => internal_symbol_names::CONSTRUCTOR.to_string(),
            SyntaxKind::GetAccessor | SyntaxKind::SetAccessor => {
                // Would need to get the property name
                node.name.clone().unwrap_or_else(|| "__accessor".to_string())
            }
            SyntaxKind::CallSignature => internal_symbol_names::CALL.to_string(),
            SyntaxKind::ConstructSignature => internal_symbol_names::NEW.to_string(),
            SyntaxKind::IndexSignature => internal_symbol_names::INDEX.to_string(),
            SyntaxKind::ExportDeclaration => internal_symbol_names::EXPORT_STAR.to_string(),
            _ => node.name.clone().unwrap_or_else(|| internal_symbol_names::MISSING.to_string()),
        }
    }

    /// Declare a symbol in the appropriate symbol table
    fn declare_symbol(
        &mut self,
        node: &Node,
        name: &str,
        includes: SymbolFlags,
        excludes: SymbolFlags,
    ) -> SymbolId {
        // Determine which symbol table location to use
        let table_location = self.get_symbol_table_location(node);

        // Check for existing symbol
        let existing = self.get_existing_symbol(&table_location, name);

        if let Some(existing_id) = existing {
            // Check for conflicts
            let has_conflict = if let Some(existing_sym) = self.symbol_arena.get(existing_id) {
                existing_sym.flags.intersects(excludes)
            } else {
                false
            };

            if has_conflict {
                // Get the existing flags for the error message
                let is_block_scoped = self.symbol_arena.get(existing_id)
                    .map(|s| s.flags.contains(SymbolFlags::BLOCK_SCOPED_VARIABLE))
                    .unwrap_or(false);

                // Report duplicate identifier error
                let message = if is_block_scoped {
                    format!("Cannot redeclare block-scoped variable '{}'", name)
                } else {
                    format!("Duplicate identifier '{}'", name)
                };
                self.diagnostics.push(BindingDiagnostic {
                    node_id: node.id,
                    message,
                    code: diagnostic_codes::DUPLICATE_IDENTIFIER,
                });

                // Create a new symbol anyway to continue binding
                return self.create_symbol_in_table(node, name, includes, table_location);
            }

            // Merge with existing symbol
            self.merge_symbol(existing_id, node, includes);
            return existing_id;
        }

        // Create new symbol
        let symbol_id = self.create_symbol_in_table(node, name, includes, table_location);

        // Track classifiable names
        if includes.intersects(SymbolFlags::CLASSIFIABLE) {
            self.classifiable_names.push(name.to_string());
        }

        symbol_id
    }

    /// Get the symbol table location for declaring a symbol
    fn get_symbol_table_location(&self, node: &Node) -> SymbolTableLocation {
        let has_export = node.modifiers.contains(ModifierFlags::EXPORT);

        // For exported members in a module, use exports
        if has_export && self.is_external_module {
            return SymbolTableLocation::Exports;
        }

        // For block-scoped variables, use the current block scope
        if matches!(
            node.kind,
            SyntaxKind::VariableDeclaration | SyntaxKind::BindingElement
        ) && node.modifiers.intersects(ModifierFlags::CONST)
        {
            return SymbolTableLocation::BlockScope;
        }

        // Default to current scope's locals
        SymbolTableLocation::CurrentScope
    }

    /// Get an existing symbol from the appropriate table
    fn get_existing_symbol(&self, location: &SymbolTableLocation, name: &str) -> Option<SymbolId> {
        match location {
            SymbolTableLocation::Exports => self.exports.get(name),
            SymbolTableLocation::BlockScope => {
                self.scopes.current_block_scope()
                    .and_then(|s| s.locals.get(name))
            }
            SymbolTableLocation::CurrentScope => {
                self.scopes.current_scope()
                    .and_then(|s| s.locals.get(name))
            }
        }
    }

    /// Create a new symbol and add it to the appropriate symbol table
    fn create_symbol_in_table(
        &mut self,
        node: &Node,
        name: &str,
        flags: SymbolFlags,
        location: SymbolTableLocation,
    ) -> SymbolId {
        let mut symbol = Symbol::new(flags, name);
        symbol.add_declaration(node.id);

        // Set value declaration if applicable
        if flags.intersects(SymbolFlags::VALUE) {
            symbol.set_value_declaration(node.id);
        }

        // Initialize members/exports if needed
        if flags.intersects(SymbolFlags::CLASS | SymbolFlags::ENUM | SymbolFlags::MODULE | SymbolFlags::VARIABLE) {
            symbol.exports = Some(SymbolTable::new());
        }
        if flags.intersects(SymbolFlags::CLASS | SymbolFlags::INTERFACE | SymbolFlags::TYPE_LITERAL | SymbolFlags::OBJECT_LITERAL) {
            symbol.members = Some(SymbolTable::new());
        }

        let symbol_id = self.symbol_arena.alloc(symbol);

        // Store the symbol reference
        if (node.id as usize) < self.node_symbols.len() {
            self.node_symbols[node.id as usize] = Some(symbol_id);
        }

        // Add to appropriate symbol table
        let name_owned = name.to_string();
        match location {
            SymbolTableLocation::Exports => {
                self.exports.set(name_owned, symbol_id);
            }
            SymbolTableLocation::BlockScope => {
                if let Some(scope) = self.scopes.current_block_scope_mut() {
                    scope.locals.set(name_owned, symbol_id);
                }
            }
            SymbolTableLocation::CurrentScope => {
                if let Some(scope) = self.scopes.current_scope_mut() {
                    scope.locals.set(name_owned, symbol_id);
                }
            }
        }

        symbol_id
    }

    /// Merge a declaration into an existing symbol
    fn merge_symbol(&mut self, symbol_id: SymbolId, node: &Node, flags: SymbolFlags) {
        if let Some(mut symbol) = self.symbol_arena.get_mut(symbol_id) {
            symbol.flags |= flags;
            symbol.add_declaration(node.id);

            if flags.intersects(SymbolFlags::VALUE) && symbol.value_declaration.is_none() {
                symbol.set_value_declaration(node.id);
            }
        }

        // Store the symbol reference
        if (node.id as usize) < self.node_symbols.len() {
            self.node_symbols[node.id as usize] = Some(symbol_id);
        }
    }

    /// Get the symbol for a node
    pub fn get_symbol_for_node(&self, node_id: NodeId) -> Option<SymbolId> {
        self.node_symbols.get(node_id as usize).copied().flatten()
    }

    /// Get a symbol by ID
    pub fn get_symbol(&self, id: SymbolId) -> Option<std::cell::Ref<'_, Symbol>> {
        self.symbol_arena.get(id)
    }

    /// Get the scope chain
    pub fn scopes(&self) -> &ScopeChain {
        &self.scopes
    }

    /// Get the symbol arena
    pub fn symbol_arena(&self) -> &Arena<Symbol> {
        &self.symbol_arena
    }
}

impl Default for Binder {
    fn default() -> Self {
        Self::new()
    }
}

/// Container flags for determining how to bind a node
#[derive(Debug, Default)]
#[allow(dead_code)]
struct ContainerFlags {
    is_container: bool,
    is_block_scoped_container: bool,
    is_control_flow_container: bool,
    is_function_like: bool,
    is_function_expression: bool,
    has_locals: bool,
    is_interface: bool,
    is_object_literal_or_class_expression_method: bool,
}

#[cfg(test)]
mod tests {
    use super::*;

    fn create_source_file(children: Vec<Node>) -> (Node, Vec<Node>) {
        let mut all_nodes = vec![];
        let mut child_ids = vec![];

        // Add children first
        for mut child in children {
            child.id = all_nodes.len() as NodeId;
            child.parent = Some(0); // Source file will be at index 0
            child_ids.push(child.id);
            all_nodes.push(child);
        }

        // Create source file
        let source_file = Node {
            id: all_nodes.len() as NodeId,
            kind: SyntaxKind::SourceFile,
            name: None,
            children: child_ids,
            parent: None,
            modifiers: ModifierFlags::NONE,
        };

        // Update child parent IDs
        let source_id = source_file.id;
        for node in &mut all_nodes {
            node.parent = Some(source_id);
        }

        all_nodes.push(source_file.clone());

        // Swap so source file is first
        let len = all_nodes.len();
        all_nodes.swap(0, len - 1);
        for node in &mut all_nodes {
            if node.id == source_id {
                node.id = 0;
            } else {
                node.id += 1;
            }
            if let Some(ref mut parent) = node.parent {
                if *parent == source_id {
                    *parent = 0;
                }
            }
        }
        all_nodes[0].children = (1..len as NodeId).collect();

        (all_nodes[0].clone(), all_nodes)
    }

    #[test]
    fn test_bind_empty_file() {
        let mut binder = Binder::new();
        let (root, nodes) = create_source_file(vec![]);

        let result = binder.bind_source_file(&root, &nodes);

        assert!(result.diagnostics.is_empty());
        assert_eq!(result.symbol_count, 0);
    }

    #[test]
    fn test_bind_function_declaration() {
        let mut binder = Binder::new();

        let func_node = Node {
            id: 0, // Will be reassigned
            kind: SyntaxKind::FunctionDeclaration,
            name: Some("myFunction".to_string()),
            children: vec![],
            parent: None,
            modifiers: ModifierFlags::NONE,
        };

        let (root, nodes) = create_source_file(vec![func_node]);

        let result = binder.bind_source_file(&root, &nodes);

        assert!(result.diagnostics.is_empty());
        assert_eq!(result.symbol_count, 1);
        assert!(result.globals.has("myFunction"));
    }

    #[test]
    fn test_bind_class_declaration() {
        let mut binder = Binder::new();

        let class_node = Node {
            id: 0,
            kind: SyntaxKind::ClassDeclaration,
            name: Some("MyClass".to_string()),
            children: vec![],
            parent: None,
            modifiers: ModifierFlags::NONE,
        };

        let (root, nodes) = create_source_file(vec![class_node]);

        let result = binder.bind_source_file(&root, &nodes);

        assert!(result.diagnostics.is_empty());
        assert_eq!(result.symbol_count, 1);
        assert!(result.globals.has("MyClass"));
    }

    #[test]
    fn test_bind_duplicate_identifier() {
        let mut binder = Binder::new();

        let func1 = Node {
            id: 0,
            kind: SyntaxKind::ClassDeclaration,
            name: Some("Foo".to_string()),
            children: vec![],
            parent: None,
            modifiers: ModifierFlags::NONE,
        };

        let func2 = Node {
            id: 0,
            kind: SyntaxKind::ClassDeclaration,
            name: Some("Foo".to_string()),
            children: vec![],
            parent: None,
            modifiers: ModifierFlags::NONE,
        };

        let (root, nodes) = create_source_file(vec![func1, func2]);

        let result = binder.bind_source_file(&root, &nodes);

        // Should have a duplicate identifier diagnostic
        assert_eq!(result.diagnostics.len(), 1);
        assert!(result.diagnostics[0].message.contains("Duplicate identifier"));
    }

    #[test]
    fn test_bind_interface_declaration() {
        let mut binder = Binder::new();

        let interface_node = Node {
            id: 0,
            kind: SyntaxKind::InterfaceDeclaration,
            name: Some("MyInterface".to_string()),
            children: vec![],
            parent: None,
            modifiers: ModifierFlags::NONE,
        };

        let (root, nodes) = create_source_file(vec![interface_node]);

        let result = binder.bind_source_file(&root, &nodes);

        assert!(result.diagnostics.is_empty());
        assert!(result.globals.has("MyInterface"));

        // Check the symbol has correct flags
        if let Some(symbol_id) = result.globals.get("MyInterface") {
            let symbol = binder.get_symbol(symbol_id).unwrap();
            assert!(symbol.flags.contains(SymbolFlags::INTERFACE));
        }
    }

    #[test]
    fn test_bind_enum_declaration() {
        let mut binder = Binder::new();

        let enum_node = Node {
            id: 0,
            kind: SyntaxKind::EnumDeclaration,
            name: Some("MyEnum".to_string()),
            children: vec![],
            parent: None,
            modifiers: ModifierFlags::NONE,
        };

        let (root, nodes) = create_source_file(vec![enum_node]);

        let result = binder.bind_source_file(&root, &nodes);

        assert!(result.diagnostics.is_empty());
        assert!(result.globals.has("MyEnum"));

        if let Some(symbol_id) = result.globals.get("MyEnum") {
            let symbol = binder.get_symbol(symbol_id).unwrap();
            assert!(symbol.flags.contains(SymbolFlags::REGULAR_ENUM));
        }
    }

    #[test]
    fn test_bind_const_enum() {
        let mut binder = Binder::new();

        let enum_node = Node {
            id: 0,
            kind: SyntaxKind::EnumDeclaration,
            name: Some("ConstEnum".to_string()),
            children: vec![],
            parent: None,
            modifiers: ModifierFlags::CONST,
        };

        let (root, nodes) = create_source_file(vec![enum_node]);

        let result = binder.bind_source_file(&root, &nodes);

        assert!(result.diagnostics.is_empty());

        if let Some(symbol_id) = result.globals.get("ConstEnum") {
            let symbol = binder.get_symbol(symbol_id).unwrap();
            assert!(symbol.flags.contains(SymbolFlags::CONST_ENUM));
        }
    }

    #[test]
    fn test_bind_type_alias() {
        let mut binder = Binder::new();

        let type_alias = Node {
            id: 0,
            kind: SyntaxKind::TypeAliasDeclaration,
            name: Some("MyType".to_string()),
            children: vec![],
            parent: None,
            modifiers: ModifierFlags::NONE,
        };

        let (root, nodes) = create_source_file(vec![type_alias]);

        let result = binder.bind_source_file(&root, &nodes);

        assert!(result.diagnostics.is_empty());
        assert!(result.globals.has("MyType"));

        if let Some(symbol_id) = result.globals.get("MyType") {
            let symbol = binder.get_symbol(symbol_id).unwrap();
            assert!(symbol.flags.contains(SymbolFlags::TYPE_ALIAS));
        }
    }

    #[test]
    fn test_bind_namespace() {
        let mut binder = Binder::new();

        let namespace = Node {
            id: 0,
            kind: SyntaxKind::ModuleDeclaration,
            name: Some("MyNamespace".to_string()),
            children: vec![],
            parent: None,
            modifiers: ModifierFlags::NONE,
        };

        let (root, nodes) = create_source_file(vec![namespace]);

        let result = binder.bind_source_file(&root, &nodes);

        assert!(result.diagnostics.is_empty());
        assert!(result.globals.has("MyNamespace"));

        if let Some(symbol_id) = result.globals.get("MyNamespace") {
            let symbol = binder.get_symbol(symbol_id).unwrap();
            assert!(symbol.flags.contains(SymbolFlags::VALUE_MODULE));
        }
    }

    #[test]
    fn test_interface_merging() {
        let mut binder = Binder::new();

        // Two interfaces with the same name should merge
        let interface1 = Node {
            id: 0,
            kind: SyntaxKind::InterfaceDeclaration,
            name: Some("Mergeable".to_string()),
            children: vec![],
            parent: None,
            modifiers: ModifierFlags::NONE,
        };

        let interface2 = Node {
            id: 0,
            kind: SyntaxKind::InterfaceDeclaration,
            name: Some("Mergeable".to_string()),
            children: vec![],
            parent: None,
            modifiers: ModifierFlags::NONE,
        };

        let (root, nodes) = create_source_file(vec![interface1, interface2]);

        let result = binder.bind_source_file(&root, &nodes);

        // Should merge without errors
        assert!(result.diagnostics.is_empty());
        assert!(result.globals.has("Mergeable"));

        // Should only have one symbol
        assert_eq!(result.symbol_count, 1);

        // The symbol should have both declarations
        if let Some(symbol_id) = result.globals.get("Mergeable") {
            let symbol = binder.get_symbol(symbol_id).unwrap();
            assert_eq!(symbol.declarations.len(), 2);
        }
    }
}
