//! Binder implementation for TypeScript AST.
//!
//! The binder walks the AST and creates symbols, establishing
//! scope and name resolution.

use serde::Serialize;
use crate::parser::NodeIndex;

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
#[derive(Debug, Default, Serialize)]
pub struct SymbolArena {
    symbols: Vec<Symbol>,
}

impl SymbolArena {
    pub fn new() -> Self {
        SymbolArena {
            symbols: Vec::new(),
        }
    }

    /// Allocate a new symbol and return its ID.
    pub fn alloc(&mut self, flags: u32, name: String) -> SymbolId {
        let id = SymbolId(self.symbols.len() as u32);
        self.symbols.push(Symbol::new(id, flags, name));
        id
    }

    /// Get a symbol by ID.
    pub fn get(&self, id: SymbolId) -> Option<&Symbol> {
        if id.is_none() {
            None
        } else {
            self.symbols.get(id.0 as usize)
        }
    }

    /// Get a mutable symbol by ID.
    pub fn get_mut(&mut self, id: SymbolId) -> Option<&mut Symbol> {
        if id.is_none() {
            None
        } else {
            self.symbols.get_mut(id.0 as usize)
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
// Binder State
// =============================================================================

use crate::parser::{Node, NodeArena};

/// Binder state for walking the AST and creating symbols.
pub struct BinderState {
    /// Arena for allocating symbols
    pub symbols: SymbolArena,
    /// Current container's symbol table (locals)
    pub current_scope: SymbolTable,
    /// Stack of scopes for nested blocks
    scope_stack: Vec<SymbolTable>,
    /// File-level symbol table
    pub file_locals: SymbolTable,
}

impl BinderState {
    pub fn new() -> Self {
        BinderState {
            symbols: SymbolArena::new(),
            current_scope: SymbolTable::new(),
            scope_stack: Vec::new(),
            file_locals: SymbolTable::new(),
        }
    }

    /// Bind a source file, creating symbols for all declarations.
    pub fn bind_source_file(&mut self, arena: &NodeArena, root: NodeIndex) {
        // Start with file scope
        self.current_scope = SymbolTable::new();

        if let Some(node) = arena.get(root) {
            if let Node::SourceFile(sf) = node {
                // Bind each statement
                for &stmt_idx in &sf.statements.nodes {
                    self.bind_node(arena, stmt_idx);
                }
            }
        }

        // Store file locals
        self.file_locals = std::mem::take(&mut self.current_scope);
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

            // Block - creates a new scope
            Node::Block(block) => {
                self.push_scope();
                for &stmt_idx in &block.statements.nodes {
                    self.bind_node(arena, stmt_idx);
                }
                self.pop_scope();
            }

            // Other statements - recurse into children
            Node::IfStatement(if_stmt) => {
                self.bind_node(arena, if_stmt.then_statement);
                if !if_stmt.else_statement.is_none() {
                    self.bind_node(arena, if_stmt.else_statement);
                }
            }
            Node::WhileStatement(while_stmt) => {
                self.bind_node(arena, while_stmt.statement);
            }
            Node::ForStatement(for_stmt) => {
                self.push_scope();
                self.bind_node(arena, for_stmt.initializer);
                self.bind_node(arena, for_stmt.statement);
                self.pop_scope();
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
                // Module block creates a new scope
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
    fn declare_symbol(&mut self, name: String, flags: u32, declaration: NodeIndex) -> SymbolId {
        // Check if symbol already exists
        if let Some(existing_id) = self.current_scope.get(&name) {
            // Symbol already exists - could merge or report error
            // For now, just add the declaration
            if let Some(sym) = self.symbols.get_mut(existing_id) {
                sym.declarations.push(declaration);
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

    fn bind_variable_declaration(
        &mut self,
        arena: &NodeArena,
        decl: &crate::parser::VariableDeclaration,
        decl_idx: NodeIndex,
    ) {
        if let Some(name) = self.get_identifier_name(arena, decl.name) {
            // Determine flags based on parent (let/const vs var)
            // For simplicity, treat all as block-scoped for now
            let flags = symbol_flags::BLOCK_SCOPED_VARIABLE;
            self.declare_symbol(name, flags, decl_idx);
        }
    }

    fn bind_function_declaration(
        &mut self,
        arena: &NodeArena,
        func: &crate::parser::FunctionDeclaration,
        func_idx: NodeIndex,
    ) {
        if let Some(name) = self.get_identifier_name(arena, func.name) {
            self.declare_symbol(name, symbol_flags::FUNCTION, func_idx);
        }

        // Bind function body in new scope
        if !func.body.is_none() {
            self.push_scope();
            // Bind parameters
            for &param_idx in &func.parameters.nodes {
                if let Some(Node::ParameterDeclaration(param)) = arena.get(param_idx) {
                    if let Some(name) = self.get_identifier_name(arena, param.name) {
                        self.declare_symbol(name, symbol_flags::FUNCTION_SCOPED_VARIABLE, param_idx);
                    }
                }
            }
            self.bind_node(arena, func.body);
            self.pop_scope();
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
            self.push_scope();
            self.bind_node(arena, module.body);
            self.pop_scope();
        }
    }
}

impl Default for BinderState {
    fn default() -> Self {
        Self::new()
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
}
