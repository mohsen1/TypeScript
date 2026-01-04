//! ThinChecker - Type checker using ThinNodeArena
//!
//! This checker uses the ThinNode architecture for cache-optimized AST access.
//! It works directly with ThinNodeArena instead of the old Node enum.
//!
//! # Architecture
//!
//! - Uses ThinNodeArena for AST access
//! - Uses ThinBinderState for symbol information
//! - Reuses TypeArena from existing checker (types are already well-optimized)
//!
//! # Status
//!
//! This is a minimal implementation to establish the structure.
//! Type inference methods will be added incrementally.

use std::collections::{HashMap, HashSet};
use std::cell::RefCell;
use rustc_hash::FxHashMap;

use crate::parser::NodeIndex;
use crate::parser::thin_node::ThinNodeArena;
use crate::parser::syntax_kind_ext;
use crate::scanner::SyntaxKind;
use crate::binder::{SymbolId, symbol_flags};
use crate::thin_binder::ThinBinderState;
use crate::checker::arena::TypeArena;
use crate::checker::types::TypeId;
use crate::checker::state::{Diagnostic, DiagnosticCategory};

// =============================================================================
// ThinCheckerState
// =============================================================================

/// Type checker state using ThinNodeArena.
///
/// This is a performance-optimized checker that works directly with the
/// cache-friendly ThinNode architecture.
pub struct ThinCheckerState<'a> {
    /// The ThinNodeArena containing the AST.
    pub arena: &'a ThinNodeArena,

    /// The binder state with symbols.
    pub binder: &'a ThinBinderState,

    /// Type arena for allocating types.
    pub types: TypeArena,

    /// Cached types for symbols.
    symbol_types: FxHashMap<SymbolId, TypeId>,

    /// Cached types for nodes.
    node_types: FxHashMap<u32, TypeId>,

    /// Type parameter names for type_to_string.
    type_parameter_names: FxHashMap<TypeId, String>,

    /// Current type parameter scope (name -> TypeId).
    type_parameter_scope: HashMap<String, TypeId>,

    /// Diagnostics produced during type checking.
    pub diagnostics: Vec<Diagnostic>,

    /// Stack of symbols being resolved (to detect circular references).
    symbol_resolution_stack: Vec<SymbolId>,
    /// O(1) lookup set for symbol resolution stack.
    symbol_resolution_set: HashSet<SymbolId>,

    /// Stack of nodes being resolved (to detect circular references).
    node_resolution_stack: Vec<NodeIndex>,
    /// O(1) lookup set for node resolution stack.
    node_resolution_set: HashSet<NodeIndex>,

    /// Current file name.
    pub file_name: String,

    /// Contextual type for expression being checked.
    contextual_type: Option<TypeId>,

    /// Cache for type relation results.
    relation_cache: RefCell<FxHashMap<(TypeId, TypeId, u8), bool>>,

    /// Current depth of recursive type instantiation.
    instantiation_depth: RefCell<u32>,

    /// Current depth of call expression resolution.
    call_depth: RefCell<u32>,

    /// Stack of local scopes for function parameters and block-scoped variables.
    local_scope_stack: Vec<FxHashMap<String, TypeId>>,
}

/// Maximum depth for recursive type instantiation.
pub const MAX_INSTANTIATION_DEPTH: u32 = 50;

/// Maximum depth for call expression resolution.
pub const MAX_CALL_DEPTH: u32 = 20;

impl<'a> ThinCheckerState<'a> {
    /// Create a new ThinCheckerState.
    pub fn new(
        arena: &'a ThinNodeArena,
        binder: &'a ThinBinderState,
        file_name: String,
    ) -> Self {
        ThinCheckerState {
            arena,
            binder,
            types: TypeArena::new(),
            symbol_types: FxHashMap::default(),
            node_types: FxHashMap::default(),
            type_parameter_names: FxHashMap::default(),
            type_parameter_scope: HashMap::new(),
            diagnostics: Vec::new(),
            symbol_resolution_stack: Vec::new(),
            symbol_resolution_set: HashSet::new(),
            node_resolution_stack: Vec::new(),
            node_resolution_set: HashSet::new(),
            file_name,
            contextual_type: None,
            relation_cache: RefCell::new(FxHashMap::default()),
            instantiation_depth: RefCell::new(0),
            call_depth: RefCell::new(0),
            local_scope_stack: Vec::new(),
        }
    }

    // =========================================================================
    // Scope Management
    // =========================================================================

    /// Push a new local scope onto the stack.
    pub fn push_local_scope(&mut self) {
        self.local_scope_stack.push(FxHashMap::default());
    }

    /// Pop the current local scope from the stack.
    pub fn pop_local_scope(&mut self) {
        self.local_scope_stack.pop();
    }

    /// Add a local variable to the current scope.
    pub fn add_local(&mut self, name: String, type_id: TypeId) {
        if let Some(scope) = self.local_scope_stack.last_mut() {
            scope.insert(name, type_id);
        }
    }

    /// Look up a local variable in all scopes.
    pub fn lookup_local(&self, name: &str) -> Option<TypeId> {
        for scope in self.local_scope_stack.iter().rev() {
            if let Some(&type_id) = scope.get(name) {
                return Some(type_id);
            }
        }
        None
    }

    // =========================================================================
    // Diagnostics
    // =========================================================================

    /// Add an error diagnostic.
    pub fn error(&mut self, start: u32, length: u32, message: String, code: u32) {
        self.diagnostics.push(Diagnostic::error(
            self.file_name.clone(),
            start,
            length,
            message,
            code,
        ));
    }

    /// Get node span (pos, end) from index.
    pub fn get_node_span(&self, idx: NodeIndex) -> Option<(u32, u32)> {
        let node = self.arena.get(idx)?;
        Some((node.pos, node.end))
    }

    // =========================================================================
    // Symbol Resolution
    // =========================================================================

    /// Get the symbol for a node index.
    pub fn get_symbol_at_node(&self, idx: NodeIndex) -> Option<SymbolId> {
        self.binder.get_node_symbol(idx)
    }

    /// Get the symbol by name from file locals.
    pub fn get_symbol_by_name(&self, name: &str) -> Option<SymbolId> {
        self.binder.file_locals.get(name)
    }

    // =========================================================================
    // Type Resolution - Core Methods
    // =========================================================================

    /// Get the type of a node.
    pub fn get_type_of_node(&mut self, idx: NodeIndex) -> TypeId {
        // Check cache first
        if let Some(&cached) = self.node_types.get(&idx.0) {
            return cached;
        }

        // Check for circular reference
        if self.node_resolution_set.contains(&idx) {
            return self.types.any_type;
        }

        // Push onto resolution stack
        self.node_resolution_stack.push(idx);
        self.node_resolution_set.insert(idx);

        let result = self.compute_type_of_node(idx);

        // Pop from resolution stack
        self.node_resolution_stack.pop();
        self.node_resolution_set.remove(&idx);

        // Cache result
        self.node_types.insert(idx.0, result);

        result
    }

    /// Compute the type of a node (internal, not cached).
    fn compute_type_of_node(&mut self, idx: NodeIndex) -> TypeId {
        let Some(node) = self.arena.get(idx) else {
            return self.types.any_type;
        };

        match node.kind {
            // Identifiers
            k if k == SyntaxKind::Identifier as u16 => {
                self.get_type_of_identifier(idx)
            }

            // Literals
            k if k == SyntaxKind::NumericLiteral as u16 => {
                self.types.number_type
            }
            k if k == SyntaxKind::StringLiteral as u16 => {
                self.types.string_type
            }
            k if k == SyntaxKind::TrueKeyword as u16 || k == SyntaxKind::FalseKeyword as u16 => {
                self.types.boolean_type
            }
            k if k == SyntaxKind::NullKeyword as u16 => {
                self.types.null_type
            }

            // Binary expressions
            k if k == syntax_kind_ext::BINARY_EXPRESSION => {
                self.get_type_of_binary_expression(idx)
            }

            // Call expressions
            k if k == syntax_kind_ext::CALL_EXPRESSION => {
                // TODO: Implement properly
                self.types.any_type
            }

            // Property access
            k if k == syntax_kind_ext::PROPERTY_ACCESS_EXPRESSION => {
                // TODO: Implement properly
                self.types.any_type
            }

            // Variable declaration
            k if k == syntax_kind_ext::VARIABLE_DECLARATION => {
                self.get_type_of_variable_declaration(idx)
            }

            // Function declaration
            k if k == syntax_kind_ext::FUNCTION_DECLARATION => {
                // TODO: Build function type
                self.types.any_type
            }

            // Arrow function
            k if k == syntax_kind_ext::ARROW_FUNCTION => {
                // TODO: Build function type
                self.types.any_type
            }

            // Array literal
            k if k == syntax_kind_ext::ARRAY_LITERAL_EXPRESSION => {
                // TODO: Implement properly
                self.types.any_type
            }

            // Object literal
            k if k == syntax_kind_ext::OBJECT_LITERAL_EXPRESSION => {
                // TODO: Implement properly
                self.types.any_type
            }

            // Prefix unary expression
            k if k == syntax_kind_ext::PREFIX_UNARY_EXPRESSION => {
                // TODO: Implement properly based on operator
                self.types.any_type
            }

            // Postfix unary expression
            k if k == syntax_kind_ext::POSTFIX_UNARY_EXPRESSION => {
                // ++ and -- always return number
                self.types.number_type
            }

            // typeof expression
            k if k == syntax_kind_ext::TYPE_OF_EXPRESSION => {
                self.types.string_type
            }

            // void expression
            k if k == syntax_kind_ext::VOID_EXPRESSION => {
                self.types.undefined_type
            }

            // Default case
            _ => self.types.any_type,
        }
    }

    // =========================================================================
    // Type Resolution - Specific Node Types
    // =========================================================================

    /// Get type of identifier.
    fn get_type_of_identifier(&mut self, idx: NodeIndex) -> TypeId {
        let Some(node) = self.arena.get(idx) else {
            return self.types.any_type;
        };

        let Some(ident) = self.arena.get_identifier(node) else {
            return self.types.any_type;
        };

        let name = &ident.escaped_text;

        // Check local scopes first
        if let Some(type_id) = self.lookup_local(name) {
            return type_id;
        }

        // Check file locals
        if let Some(sym_id) = self.binder.file_locals.get(name) {
            return self.get_type_of_symbol(sym_id);
        }

        // Intrinsic names
        match name.as_str() {
            "undefined" => self.types.undefined_type,
            "NaN" | "Infinity" => self.types.number_type,
            _ => self.types.any_type,
        }
    }

    /// Get type of a symbol.
    pub fn get_type_of_symbol(&mut self, sym_id: SymbolId) -> TypeId {
        // Check cache first
        if let Some(&cached) = self.symbol_types.get(&sym_id) {
            return cached;
        }

        // Check for circular reference
        if self.symbol_resolution_set.contains(&sym_id) {
            return self.types.any_type;
        }

        // Push onto resolution stack
        self.symbol_resolution_stack.push(sym_id);
        self.symbol_resolution_set.insert(sym_id);

        let result = self.compute_type_of_symbol(sym_id);

        // Pop from resolution stack
        self.symbol_resolution_stack.pop();
        self.symbol_resolution_set.remove(&sym_id);

        // Cache result
        self.symbol_types.insert(sym_id, result);

        result
    }

    /// Compute type of a symbol (internal, not cached).
    fn compute_type_of_symbol(&mut self, sym_id: SymbolId) -> TypeId {
        let Some(symbol) = self.binder.get_symbol(sym_id) else {
            return self.types.any_type;
        };

        let flags = symbol.flags;

        // Function
        if flags & symbol_flags::FUNCTION != 0 {
            // TODO: Build function type from declaration
            return self.types.any_type;
        }

        // Class
        if flags & symbol_flags::CLASS != 0 {
            // TODO: Build class type from declaration
            return self.types.any_type;
        }

        // Interface
        if flags & symbol_flags::INTERFACE != 0 {
            // TODO: Build interface type from declaration
            return self.types.any_type;
        }

        // Type alias
        if flags & symbol_flags::TYPE_ALIAS != 0 {
            // TODO: Resolve type alias
            return self.types.any_type;
        }

        // Variable
        if flags & (symbol_flags::FUNCTION_SCOPED_VARIABLE | symbol_flags::BLOCK_SCOPED_VARIABLE) != 0 {
            // TODO: Get type from declaration or infer from initializer
            return self.types.any_type;
        }

        self.types.any_type
    }

    /// Get type of binary expression.
    fn get_type_of_binary_expression(&mut self, idx: NodeIndex) -> TypeId {
        let Some(node) = self.arena.get(idx) else {
            return self.types.any_type;
        };

        let Some(binary) = self.arena.get_binary_expr(node) else {
            return self.types.any_type;
        };

        let op_kind = binary.operator_token;

        match op_kind {
            // Arithmetic operators return number (simplified)
            k if k == SyntaxKind::PlusToken as u16 => {
                // + can be addition or concatenation
                // Simplified: always return any for now
                self.types.any_type
            }
            k if k == SyntaxKind::MinusToken as u16
                || k == SyntaxKind::AsteriskToken as u16
                || k == SyntaxKind::SlashToken as u16
                || k == SyntaxKind::PercentToken as u16
                || k == SyntaxKind::AsteriskAsteriskToken as u16 => {
                self.types.number_type
            }

            // Comparison operators return boolean
            k if k == SyntaxKind::LessThanToken as u16
                || k == SyntaxKind::GreaterThanToken as u16
                || k == SyntaxKind::LessThanEqualsToken as u16
                || k == SyntaxKind::GreaterThanEqualsToken as u16
                || k == SyntaxKind::EqualsEqualsToken as u16
                || k == SyntaxKind::ExclamationEqualsToken as u16
                || k == SyntaxKind::EqualsEqualsEqualsToken as u16
                || k == SyntaxKind::ExclamationEqualsEqualsToken as u16 => {
                self.types.boolean_type
            }

            // Assignment returns the assigned value's type
            k if k == SyntaxKind::EqualsToken as u16 => {
                self.get_type_of_node(binary.right)
            }

            // Bitwise operators return number
            k if k == SyntaxKind::AmpersandToken as u16
                || k == SyntaxKind::BarToken as u16
                || k == SyntaxKind::CaretToken as u16
                || k == SyntaxKind::LessThanLessThanToken as u16
                || k == SyntaxKind::GreaterThanGreaterThanToken as u16
                || k == SyntaxKind::GreaterThanGreaterThanGreaterThanToken as u16 => {
                self.types.number_type
            }

            _ => self.types.any_type,
        }
    }

    /// Get type of variable declaration.
    fn get_type_of_variable_declaration(&mut self, idx: NodeIndex) -> TypeId {
        let Some(node) = self.arena.get(idx) else {
            return self.types.any_type;
        };

        let Some(var_decl) = self.arena.get_variable_declaration(node) else {
            return self.types.any_type;
        };

        // Infer from initializer
        if !var_decl.initializer.is_none() {
            return self.get_type_of_node(var_decl.initializer);
        }

        // No initializer - implicit any
        self.types.any_type
    }

    // =========================================================================
    // Type Node Resolution
    // =========================================================================

    /// Get type from a type node.
    pub fn get_type_from_type_node(&mut self, idx: NodeIndex) -> TypeId {
        let Some(node) = self.arena.get(idx) else {
            return self.types.any_type;
        };

        match node.kind {
            k if k == SyntaxKind::NumberKeyword as u16 => self.types.number_type,
            k if k == SyntaxKind::StringKeyword as u16 => self.types.string_type,
            k if k == SyntaxKind::BooleanKeyword as u16 => self.types.boolean_type,
            k if k == SyntaxKind::VoidKeyword as u16 => self.types.void_type,
            k if k == SyntaxKind::NullKeyword as u16 => self.types.null_type,
            k if k == SyntaxKind::UndefinedKeyword as u16 => self.types.undefined_type,
            k if k == SyntaxKind::NeverKeyword as u16 => self.types.never_type,
            k if k == SyntaxKind::AnyKeyword as u16 => self.types.any_type,
            k if k == SyntaxKind::UnknownKeyword as u16 => self.types.unknown_type,
            k if k == SyntaxKind::ObjectKeyword as u16 => self.types.object_type,
            k if k == SyntaxKind::SymbolKeyword as u16 => self.types.es_symbol_type,
            k if k == SyntaxKind::BigIntKeyword as u16 => self.types.big_int_type,

            // TODO: Add more type node handling
            _ => self.types.any_type,
        }
    }
}

// =============================================================================
// Tests
// =============================================================================

#[cfg(test)]
mod tests {
    use super::*;
    use crate::parser::thin_node::ThinNodeArena;

    #[test]
    fn test_thin_checker_creation() {
        let arena = ThinNodeArena::new();
        let binder = ThinBinderState::new();
        let checker = ThinCheckerState::new(&arena, &binder, "test.ts".to_string());

        // Basic sanity check
        assert!(checker.diagnostics.is_empty());
    }

    #[test]
    fn test_thin_checker_basic_types() {
        let arena = ThinNodeArena::new();
        let binder = ThinBinderState::new();
        let mut checker = ThinCheckerState::new(&arena, &binder, "test.ts".to_string());

        // Check that basic types exist
        assert!(checker.types.number_type.0 > 0);
        assert!(checker.types.string_type.0 > 0);
        assert!(checker.types.boolean_type.0 > 0);
    }
}
