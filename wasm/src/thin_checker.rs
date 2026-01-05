//! ThinChecker - Type checker using ThinNodeArena and Solver
//!
//! This checker uses the ThinNode architecture for cache-optimized AST access
//! and the Solver's type system for structural type interning.
//!
//! # Architecture
//!
//! - Uses ThinNodeArena for AST access (16-byte cache-optimized nodes)
//! - Uses ThinBinderState for symbol information
//! - Uses Solver's TypeInterner for structural type equality (O(1) comparison)
//! - Uses solver::lower::TypeLower for AST-to-type conversion
//!
//! # Status
//!
//! Phase 7.5 integration - using solver type system for type checking.

use std::collections::{HashMap, HashSet};
use std::cell::RefCell;
use rustc_hash::FxHashMap;

use crate::parser::NodeIndex;
use crate::parser::thin_node::ThinNodeArena;
use crate::parser::syntax_kind_ext;
use crate::scanner::SyntaxKind;
use crate::binder::{SymbolId, symbol_flags};
use crate::thin_binder::ThinBinderState;
use crate::solver::{TypeId, TypeInterner};
use crate::checker::state::Diagnostic;

// =============================================================================
// ThinCheckerState
// =============================================================================

/// Type checker state using ThinNodeArena and Solver type system.
///
/// This is a performance-optimized checker that works directly with the
/// cache-friendly ThinNode architecture and uses the solver's TypeInterner
/// for structural type equality.
pub struct ThinCheckerState<'a> {
    /// The ThinNodeArena containing the AST.
    pub arena: &'a ThinNodeArena,

    /// The binder state with symbols.
    pub binder: &'a ThinBinderState,

    /// Type interner for structural type interning.
    /// Uses solver's TypeInterner for O(1) type equality.
    pub types: TypeInterner,

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
            types: TypeInterner::new(),
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
            return TypeId::ANY;
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
            return TypeId::ANY;
        };

        match node.kind {
            // Identifiers
            k if k == SyntaxKind::Identifier as u16 => {
                self.get_type_of_identifier(idx)
            }

            // Literals - use compile-time constant TypeIds
            k if k == SyntaxKind::NumericLiteral as u16 => TypeId::NUMBER,
            k if k == SyntaxKind::StringLiteral as u16 => TypeId::STRING,
            k if k == SyntaxKind::TrueKeyword as u16 => self.types.literal_boolean(true),
            k if k == SyntaxKind::FalseKeyword as u16 => self.types.literal_boolean(false),
            k if k == SyntaxKind::NullKeyword as u16 => TypeId::NULL,

            // Binary expressions
            k if k == syntax_kind_ext::BINARY_EXPRESSION => {
                self.get_type_of_binary_expression(idx)
            }

            // Call expressions
            k if k == syntax_kind_ext::CALL_EXPRESSION => {
                self.get_type_of_call_expression(idx)
            }

            // New expressions
            k if k == syntax_kind_ext::NEW_EXPRESSION => {
                self.get_type_of_new_expression(idx)
            }

            // Property access
            k if k == syntax_kind_ext::PROPERTY_ACCESS_EXPRESSION => {
                self.get_type_of_property_access(idx)
            }

            // Element access
            k if k == syntax_kind_ext::ELEMENT_ACCESS_EXPRESSION => {
                self.get_type_of_element_access(idx)
            }

            // Conditional expression (ternary)
            k if k == syntax_kind_ext::CONDITIONAL_EXPRESSION => {
                self.get_type_of_conditional_expression(idx)
            }

            // Variable declaration
            k if k == syntax_kind_ext::VARIABLE_DECLARATION => {
                self.get_type_of_variable_declaration(idx)
            }

            // Function declaration
            k if k == syntax_kind_ext::FUNCTION_DECLARATION => {
                self.get_type_of_function(idx)
            }

            // Function expression
            k if k == syntax_kind_ext::FUNCTION_EXPRESSION => {
                self.get_type_of_function(idx)
            }

            // Arrow function
            k if k == syntax_kind_ext::ARROW_FUNCTION => {
                self.get_type_of_function(idx)
            }

            // Array literal
            k if k == syntax_kind_ext::ARRAY_LITERAL_EXPRESSION => {
                self.get_type_of_array_literal(idx)
            }

            // Object literal
            k if k == syntax_kind_ext::OBJECT_LITERAL_EXPRESSION => {
                self.get_type_of_object_literal(idx)
            }

            // Prefix unary expression
            k if k == syntax_kind_ext::PREFIX_UNARY_EXPRESSION => {
                self.get_type_of_prefix_unary(idx)
            }

            // Postfix unary expression - ++ and -- always return number
            k if k == syntax_kind_ext::POSTFIX_UNARY_EXPRESSION => TypeId::NUMBER,

            // typeof expression
            k if k == syntax_kind_ext::TYPE_OF_EXPRESSION => TypeId::STRING,

            // void expression
            k if k == syntax_kind_ext::VOID_EXPRESSION => TypeId::UNDEFINED,

            // =========================================================================
            // Type Nodes
            // =========================================================================

            // Type reference (e.g., "number", "string", "MyType")
            k if k == syntax_kind_ext::TYPE_REFERENCE => {
                self.get_type_from_type_reference(idx)
            }

            // Keyword types
            k if k == SyntaxKind::NumberKeyword as u16 => TypeId::NUMBER,
            k if k == SyntaxKind::StringKeyword as u16 => TypeId::STRING,
            k if k == SyntaxKind::BooleanKeyword as u16 => TypeId::BOOLEAN,
            k if k == SyntaxKind::VoidKeyword as u16 => TypeId::VOID,
            k if k == SyntaxKind::AnyKeyword as u16 => TypeId::ANY,
            k if k == SyntaxKind::NeverKeyword as u16 => TypeId::NEVER,
            k if k == SyntaxKind::UnknownKeyword as u16 => TypeId::UNKNOWN,
            k if k == SyntaxKind::UndefinedKeyword as u16 => TypeId::UNDEFINED,
            k if k == SyntaxKind::NullKeyword as u16 => TypeId::NULL,
            k if k == SyntaxKind::ObjectKeyword as u16 => TypeId::OBJECT,
            k if k == SyntaxKind::BigIntKeyword as u16 => TypeId::BIGINT,
            k if k == SyntaxKind::SymbolKeyword as u16 => TypeId::SYMBOL,

            // Union type (A | B)
            k if k == syntax_kind_ext::UNION_TYPE => {
                self.get_type_from_union_type(idx)
            }

            // Array type (T[])
            k if k == syntax_kind_ext::ARRAY_TYPE => {
                self.get_type_from_array_type(idx)
            }

            // Default case
            _ => TypeId::ANY,
        }
    }

    /// Get type from a type reference node (e.g., "number", "string", "MyType").
    fn get_type_from_type_reference(&mut self, idx: NodeIndex) -> TypeId {
        let Some(node) = self.arena.get(idx) else {
            return TypeId::ANY;
        };

        // Get the TypeRefData from the arena
        let Some(type_ref) = self.arena.get_type_ref(node) else {
            return TypeId::ANY;
        };

        let type_name_idx = type_ref.type_name;

        // Get the identifier for the type name
        if let Some(name_node) = self.arena.get(type_name_idx) {
            if let Some(ident) = self.arena.get_identifier(name_node) {
                // Check for built-in types
                match ident.escaped_text.as_str() {
                    "number" => return TypeId::NUMBER,
                    "string" => return TypeId::STRING,
                    "boolean" => return TypeId::BOOLEAN,
                    "void" => return TypeId::VOID,
                    "any" => return TypeId::ANY,
                    "never" => return TypeId::NEVER,
                    "unknown" => return TypeId::UNKNOWN,
                    "undefined" => return TypeId::UNDEFINED,
                    "null" => return TypeId::NULL,
                    "object" => return TypeId::OBJECT,
                    "bigint" => return TypeId::BIGINT,
                    "symbol" => return TypeId::SYMBOL,
                    "Array" => {
                        // Array<T> - get type argument
                        // TODO: Handle generic Array type
                        return self.types.array(TypeId::ANY);
                    }
                    _ => {
                        // TODO: Look up user-defined types from symbol table
                        return TypeId::ANY;
                    }
                }
            }
        }

        TypeId::ANY
    }

    /// Get type from a union type node (A | B).
    fn get_type_from_union_type(&mut self, idx: NodeIndex) -> TypeId {
        let Some(node) = self.arena.get(idx) else {
            return TypeId::ANY;
        };

        // UnionType uses CompositeTypeData which has a types list
        if let Some(composite) = self.arena.get_composite_type(node) {
            let mut member_types = Vec::new();
            for &type_idx in &composite.types.nodes {
                member_types.push(self.get_type_of_node(type_idx));
            }

            if member_types.is_empty() {
                return TypeId::NEVER;
            }
            if member_types.len() == 1 {
                return member_types[0];
            }

            return self.types.union(member_types);
        }

        TypeId::ANY
    }

    /// Get type from an array type node (T[]).
    fn get_type_from_array_type(&mut self, idx: NodeIndex) -> TypeId {
        let Some(node) = self.arena.get(idx) else {
            return TypeId::ANY;
        };

        // ArrayType uses TypeOperatorData which has the element type
        if let Some(type_op) = self.arena.get_type_operator(node) {
            let elem_type = self.get_type_of_node(type_op.type_node);
            return self.types.array(elem_type);
        }

        self.types.array(TypeId::ANY)
    }

    // =========================================================================
    // Type Resolution - Specific Node Types
    // =========================================================================

    /// Get type of identifier.
    fn get_type_of_identifier(&mut self, idx: NodeIndex) -> TypeId {
        let Some(node) = self.arena.get(idx) else {
            return TypeId::ANY;
        };

        let Some(ident) = self.arena.get_identifier(node) else {
            return TypeId::ANY;
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

        // Intrinsic names - use constant TypeIds
        match name.as_str() {
            "undefined" => TypeId::UNDEFINED,
            "NaN" | "Infinity" => TypeId::NUMBER,
            _ => TypeId::ANY,
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
            return TypeId::ANY;
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
    ///
    /// Uses TypeLowering to bridge symbol declarations to solver types.
    fn compute_type_of_symbol(&mut self, sym_id: SymbolId) -> TypeId {
        use crate::solver::TypeLowering;

        let Some(symbol) = self.binder.get_symbol(sym_id) else {
            return TypeId::ANY;
        };

        let flags = symbol.flags;
        let value_decl = symbol.value_declaration;

        // Function - build function type from declaration
        if flags & symbol_flags::FUNCTION != 0 {
            if !value_decl.is_none() {
                return self.get_type_of_function(value_decl);
            }
            return TypeId::ANY;
        }

        // Class - return class constructor type
        if flags & symbol_flags::CLASS != 0 {
            // TODO: Build class constructor type
            return TypeId::ANY;
        }

        // Interface - return interface type from TypeLowering
        if flags & symbol_flags::INTERFACE != 0 {
            // For interfaces, we need to lower the interface body
            // TODO: Implement interface type lowering
            return TypeId::ANY;
        }

        // Type alias - resolve using TypeLowering
        if flags & symbol_flags::TYPE_ALIAS != 0 {
            // Get the type node from the type alias declaration
            if !value_decl.is_none() {
                if let Some(node) = self.arena.get(value_decl) {
                    if let Some(type_alias) = self.arena.get_type_alias(node) {
                        // Lower the aliased type
                        let lowering = TypeLowering::new(self.arena, &self.types);
                        return lowering.lower_type(type_alias.type_node);
                    }
                }
            }
            return TypeId::ANY;
        }

        // Variable - get type from annotation or infer from initializer
        if flags & (symbol_flags::FUNCTION_SCOPED_VARIABLE | symbol_flags::BLOCK_SCOPED_VARIABLE) != 0 {
            if !value_decl.is_none() {
                if let Some(node) = self.arena.get(value_decl) {
                    if let Some(var_decl) = self.arena.get_variable_declaration(node) {
                        // First try type annotation
                        if !var_decl.type_annotation.is_none() {
                            let lowering = TypeLowering::new(self.arena, &self.types);
                            return lowering.lower_type(var_decl.type_annotation);
                        }
                        // Fall back to inferring from initializer
                        if !var_decl.initializer.is_none() {
                            return self.get_type_of_node(var_decl.initializer);
                        }
                    }
                }
            }
            return TypeId::ANY;
        }

        // Parameter - get type from annotation
        if flags & symbol_flags::FUNCTION_SCOPED_VARIABLE != 0 {
            if !value_decl.is_none() {
                if let Some(node) = self.arena.get(value_decl) {
                    if let Some(param) = self.arena.get_parameter(node) {
                        if !param.type_annotation.is_none() {
                            let lowering = TypeLowering::new(self.arena, &self.types);
                            return lowering.lower_type(param.type_annotation);
                        }
                    }
                }
            }
            return TypeId::ANY;
        }

        TypeId::ANY
    }

    /// Get type of binary expression.
    fn get_type_of_binary_expression(&mut self, idx: NodeIndex) -> TypeId {
        let Some(node) = self.arena.get(idx) else {
            return TypeId::ANY;
        };

        let Some(binary) = self.arena.get_binary_expr(node) else {
            return TypeId::ANY;
        };

        let op_kind = binary.operator_token;

        match op_kind {
            // Arithmetic operators return number (simplified)
            k if k == SyntaxKind::PlusToken as u16 => {
                // + can be addition or concatenation
                // Simplified: always return any for now
                TypeId::ANY
            }
            k if k == SyntaxKind::MinusToken as u16
                || k == SyntaxKind::AsteriskToken as u16
                || k == SyntaxKind::SlashToken as u16
                || k == SyntaxKind::PercentToken as u16
                || k == SyntaxKind::AsteriskAsteriskToken as u16 => TypeId::NUMBER,

            // Comparison operators return boolean
            k if k == SyntaxKind::LessThanToken as u16
                || k == SyntaxKind::GreaterThanToken as u16
                || k == SyntaxKind::LessThanEqualsToken as u16
                || k == SyntaxKind::GreaterThanEqualsToken as u16
                || k == SyntaxKind::EqualsEqualsToken as u16
                || k == SyntaxKind::ExclamationEqualsToken as u16
                || k == SyntaxKind::EqualsEqualsEqualsToken as u16
                || k == SyntaxKind::ExclamationEqualsEqualsToken as u16 => TypeId::BOOLEAN,

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
                || k == SyntaxKind::GreaterThanGreaterThanGreaterThanToken as u16 => TypeId::NUMBER,

            _ => TypeId::ANY,
        }
    }

    /// Get type of variable declaration.
    fn get_type_of_variable_declaration(&mut self, idx: NodeIndex) -> TypeId {
        let Some(node) = self.arena.get(idx) else {
            return TypeId::ANY;
        };

        let Some(var_decl) = self.arena.get_variable_declaration(node) else {
            return TypeId::ANY;
        };

        // Infer from initializer
        if !var_decl.initializer.is_none() {
            return self.get_type_of_node(var_decl.initializer);
        }

        // No initializer - implicit any
        TypeId::ANY
    }

    /// Get type of call expression.
    fn get_type_of_call_expression(&mut self, idx: NodeIndex) -> TypeId {
        let Some(node) = self.arena.get(idx) else {
            return TypeId::ANY;
        };

        let Some(call) = self.arena.get_call_expr(node) else {
            return TypeId::ANY;
        };

        // Get the type of the callee
        let _callee_type = self.get_type_of_node(call.expression);

        // For now, return any for function calls
        // TODO: Extract return type from function type using TypeInterner lookup
        TypeId::ANY
    }

    /// Get type of new expression.
    fn get_type_of_new_expression(&mut self, idx: NodeIndex) -> TypeId {
        let Some(node) = self.arena.get(idx) else {
            return TypeId::ANY;
        };

        let Some(new_expr) = self.arena.get_call_expr(node) else {
            return TypeId::ANY;
        };

        // Get the type of the constructor
        let _constructor_type = self.get_type_of_node(new_expr.expression);

        // For now, return any for new expressions
        // TODO: Extract instance type from constructor
        TypeId::ANY
    }

    /// Get type of property access expression.
    fn get_type_of_property_access(&mut self, idx: NodeIndex) -> TypeId {
        let Some(node) = self.arena.get(idx) else {
            return TypeId::ANY;
        };

        let Some(access) = self.arena.get_access_expr(node) else {
            return TypeId::ANY;
        };

        // Get the type of the object
        let object_type = self.get_type_of_node(access.expression);

        // Get the property name
        let Some(name_node) = self.arena.get(access.name_or_argument) else {
            return TypeId::ANY;
        };

        // If it's an identifier, look up the property
        if let Some(ident) = self.arena.get_identifier(name_node) {
            let property_name = &ident.escaped_text;

            // Check for built-in properties on known types
            if let Some(prop_type) = self.get_property_of_type(object_type, property_name) {
                return prop_type;
            }
        }

        TypeId::ANY
    }

    /// Get type of element access expression (e.g., arr[0], obj["prop"]).
    fn get_type_of_element_access(&mut self, idx: NodeIndex) -> TypeId {
        let Some(node) = self.arena.get(idx) else {
            return TypeId::ANY;
        };

        let Some(access) = self.arena.get_access_expr(node) else {
            return TypeId::ANY;
        };

        // Get the type of the object
        let _object_type = self.get_type_of_node(access.expression);

        // Get the index type
        let _index_type = self.get_type_of_node(access.name_or_argument);

        // For now, return any for element access
        // TODO: Extract element type from array/tuple types using TypeInterner lookup
        TypeId::ANY
    }

    /// Get type of conditional expression (ternary: a ? b : c).
    fn get_type_of_conditional_expression(&mut self, idx: NodeIndex) -> TypeId {
        let Some(node) = self.arena.get(idx) else {
            return TypeId::ANY;
        };

        let Some(cond) = self.arena.get_conditional_expr(node) else {
            return TypeId::ANY;
        };

        let when_true = self.get_type_of_node(cond.when_true);
        let when_false = self.get_type_of_node(cond.when_false);

        if when_true == when_false {
            when_true
        } else {
            // Use TypeInterner's union method for automatic normalization
            self.types.union(vec![when_true, when_false])
        }
    }

    /// Get type of function declaration/expression/arrow.
    fn get_type_of_function(&mut self, idx: NodeIndex) -> TypeId {
        use crate::solver::{FunctionShape, ParamInfo};
        use std::sync::Arc;

        let Some(node) = self.arena.get(idx) else {
            return TypeId::ANY;
        };

        let Some(func) = self.arena.get_function(node) else {
            return TypeId::ANY;
        };

        // Collect parameter info using solver's ParamInfo struct
        let mut params = Vec::new();

        for &param_idx in &func.parameters.nodes {
            if let Some(param_node) = self.arena.get(param_idx) {
                if let Some(param) = self.arena.get_parameter(param_node) {
                    // Get parameter name
                    let name: Option<Arc<str>> = if let Some(name_node) = self.arena.get(param.name) {
                        if let Some(name_data) = self.arena.get_identifier(name_node) {
                            Some(Arc::from(name_data.escaped_text.as_str()))
                        } else {
                            None
                        }
                    } else {
                        None
                    };

                    // Use type annotation if present, otherwise any
                    let type_id = if !param.type_annotation.is_none() {
                        self.get_type_from_type_node(param.type_annotation)
                    } else {
                        TypeId::ANY
                    };

                    // Check if optional or has initializer
                    let optional = param.question_token || !param.initializer.is_none();
                    let rest = param.dot_dot_dot_token;

                    params.push(ParamInfo {
                        name,
                        type_id,
                        optional,
                        rest,
                    });
                }
            }
        }

        // Get return type from annotation or infer
        let return_type = if !func.type_annotation.is_none() {
            self.get_type_from_type_node(func.type_annotation)
        } else {
            // TODO: Infer return type from body
            TypeId::ANY
        };

        // Create function type using TypeInterner
        let shape = FunctionShape {
            type_params: Vec::new(), // TODO: Handle type parameters
            params,
            return_type,
            is_constructor: false,
        };

        self.types.function(shape)
    }

    /// Get type of array literal.
    fn get_type_of_array_literal(&mut self, idx: NodeIndex) -> TypeId {
        let Some(node) = self.arena.get(idx) else {
            return TypeId::ANY;
        };

        let Some(array) = self.arena.get_literal_expr(node) else {
            return TypeId::ANY;
        };

        if array.elements.nodes.is_empty() {
            // Empty array literal: infer from context or use never[]
            if let Some(contextual) = self.contextual_type {
                return contextual;
            }
            return self.types.array(TypeId::NEVER);
        }

        // Get types of all elements
        let mut element_types = Vec::new();
        for &elem_idx in &array.elements.nodes {
            if !elem_idx.is_none() {
                element_types.push(self.get_type_of_node(elem_idx));
            }
        }

        // Create union of element types using TypeInterner
        let element_type = if element_types.len() == 1 {
            element_types[0]
        } else if element_types.is_empty() {
            TypeId::NEVER
        } else {
            self.types.union(element_types)
        };

        self.types.array(element_type)
    }

    /// Get type of object literal.
    fn get_type_of_object_literal(&mut self, idx: NodeIndex) -> TypeId {
        let Some(node) = self.arena.get(idx) else {
            return TypeId::ANY;
        };

        let Some(_obj) = self.arena.get_literal_expr(node) else {
            return TypeId::ANY;
        };

        // TODO: Create PropertyInfo for object literal properties
        // For now, return an empty anonymous object type
        self.types.object(Vec::new())
    }

    /// Get type of prefix unary expression.
    fn get_type_of_prefix_unary(&mut self, idx: NodeIndex) -> TypeId {
        let Some(node) = self.arena.get(idx) else {
            return TypeId::ANY;
        };

        let Some(unary) = self.arena.get_unary_expr(node) else {
            return TypeId::ANY;
        };

        match unary.operator {
            // ! returns boolean
            k if k == SyntaxKind::ExclamationToken as u16 => TypeId::BOOLEAN,
            // Unary + and - return number
            k if k == SyntaxKind::PlusToken as u16 || k == SyntaxKind::MinusToken as u16 => TypeId::NUMBER,
            // ~ returns number
            k if k == SyntaxKind::TildeToken as u16 => TypeId::NUMBER,
            // ++ and -- return number
            k if k == SyntaxKind::PlusPlusToken as u16 || k == SyntaxKind::MinusMinusToken as u16 => TypeId::NUMBER,
            _ => TypeId::ANY,
        }
    }

    /// Get a property type from an object type.
    fn get_property_of_type(&self, _type_id: TypeId, _property_name: &str) -> Option<TypeId> {
        // TODO: Implement property lookup on types
        // For now, return None to indicate property not found
        None
    }

    // =========================================================================
    // Type Relations (uses solver::SubtypeChecker)
    // =========================================================================

    /// Check if `source` type is assignable to `target` type.
    ///
    /// Uses the solver's SubtypeChecker with coinductive cycle detection.
    /// Note: Does not resolve Ref types (use `is_assignable_to_with_resolution` for that).
    pub fn is_assignable_to(&self, source: TypeId, target: TypeId) -> bool {
        use crate::solver::SubtypeChecker;
        let mut checker = SubtypeChecker::new(&self.types);
        checker.is_assignable_to(source, target)
    }

    /// Check if `source` type is assignable to `target` type, resolving Ref types.
    ///
    /// Uses the provided TypeEnvironment to resolve type references.
    pub fn is_assignable_to_with_env(
        &self,
        source: TypeId,
        target: TypeId,
        env: &crate::solver::TypeEnvironment,
    ) -> bool {
        use crate::solver::SubtypeChecker;
        let mut checker = SubtypeChecker::with_resolver(&self.types, env);
        checker.is_assignable_to(source, target)
    }

    /// Check if `source` type is a subtype of `target` type.
    ///
    /// Stricter than assignability. Uses coinductive semantics for recursive types.
    pub fn is_subtype_of(&self, source: TypeId, target: TypeId) -> bool {
        use crate::solver::SubtypeChecker;
        let mut checker = SubtypeChecker::new(&self.types);
        checker.is_subtype_of(source, target)
    }

    /// Check if `source` type is a subtype of `target` type, resolving Ref types.
    ///
    /// Uses the provided TypeEnvironment to resolve type references.
    pub fn is_subtype_of_with_env(
        &self,
        source: TypeId,
        target: TypeId,
        env: &crate::solver::TypeEnvironment,
    ) -> bool {
        use crate::solver::SubtypeChecker;
        let mut checker = SubtypeChecker::with_resolver(&self.types, env);
        checker.is_subtype_of(source, target)
    }

    /// Check if two types are identical.
    ///
    /// O(1) operation - just compare TypeId values (structural interning).
    pub fn are_types_identical(&self, type1: TypeId, type2: TypeId) -> bool {
        type1 == type2
    }

    /// Check if a type is assignable to a union of types.
    pub fn is_assignable_to_union(&self, source: TypeId, targets: &[TypeId]) -> bool {
        use crate::solver::SubtypeChecker;
        let mut checker = SubtypeChecker::new(&self.types);
        for &target in targets {
            if checker.is_assignable_to(source, target) {
                return true;
            }
        }
        false
    }

    /// Create a TypeEnvironment populated with resolved symbol types.
    ///
    /// This can be passed to `is_assignable_to_with_env` for type checking
    /// that needs to resolve type references.
    pub fn build_type_environment(&mut self) -> crate::solver::TypeEnvironment {
        use crate::solver::{TypeEnvironment, SymbolRef};

        let mut env = TypeEnvironment::new();

        // Collect all unique symbols from node_symbols map
        let symbols: Vec<SymbolId> = self.binder.node_symbols
            .values()
            .copied()
            .collect::<std::collections::HashSet<_>>()
            .into_iter()
            .collect();

        // Resolve each symbol and add to the environment
        for sym_id in symbols {
            // Get the type for this symbol
            let type_id = self.get_type_of_symbol(sym_id);
            if type_id != TypeId::ANY && type_id != TypeId::ERROR {
                // Use symbol's raw ID as the SymbolRef
                env.insert(SymbolRef(sym_id.0), type_id);
            }
        }

        env
    }

    /// Create a union type from multiple types.
    ///
    /// Automatically normalizes: flattens nested unions, deduplicates, sorts.
    pub fn get_union_type(&self, types: Vec<TypeId>) -> TypeId {
        self.types.union(types)
    }

    /// Create an intersection type from multiple types.
    ///
    /// Automatically normalizes: flattens nested intersections, deduplicates, sorts.
    pub fn get_intersection_type(&self, types: Vec<TypeId>) -> TypeId {
        self.types.intersection(types)
    }

    // =========================================================================
    // Type Narrowing (uses solver::NarrowingContext)
    // =========================================================================

    /// Narrow a type by a typeof guard.
    ///
    /// Example: `typeof x === "string"` narrows `string | number` to `string`.
    pub fn narrow_by_typeof(&self, source: TypeId, typeof_result: &str) -> TypeId {
        use crate::solver::NarrowingContext;
        let ctx = NarrowingContext::new(&self.types);
        ctx.narrow_by_typeof(source, typeof_result)
    }

    /// Narrow a type by excluding a typeof guard.
    ///
    /// Example: `typeof x !== "string"` narrows `string | number` to `number`.
    pub fn narrow_by_typeof_negation(&self, source: TypeId, typeof_result: &str) -> TypeId {
        use crate::solver::NarrowingContext;
        let ctx = NarrowingContext::new(&self.types);

        // Get the target type for this typeof result
        let target = match typeof_result {
            "string" => TypeId::STRING,
            "number" => TypeId::NUMBER,
            "boolean" => TypeId::BOOLEAN,
            "bigint" => TypeId::BIGINT,
            "symbol" => TypeId::SYMBOL,
            "undefined" => TypeId::UNDEFINED,
            "object" => TypeId::OBJECT,
            _ => return source,
        };

        ctx.narrow_excluding_type(source, target)
    }

    /// Narrow a discriminated union by a discriminant property check.
    ///
    /// Example: `action.type === "add"` narrows `{ type: "add" } | { type: "remove" }`
    /// to `{ type: "add" }`.
    pub fn narrow_by_discriminant(&self, union_type: TypeId, property_name: &str, literal_value: TypeId) -> TypeId {
        use crate::solver::NarrowingContext;
        let ctx = NarrowingContext::new(&self.types);
        ctx.narrow_by_discriminant(union_type, property_name, literal_value)
    }

    /// Narrow a discriminated union by excluding a discriminant value.
    ///
    /// Example: `action.type !== "add"` narrows the union to exclude the "add" variant.
    pub fn narrow_by_excluding_discriminant(&self, union_type: TypeId, property_name: &str, excluded_value: TypeId) -> TypeId {
        use crate::solver::NarrowingContext;
        let ctx = NarrowingContext::new(&self.types);
        ctx.narrow_by_excluding_discriminant(union_type, property_name, excluded_value)
    }

    /// Find discriminant properties in a union type.
    ///
    /// Returns information about properties that uniquely identify each union variant.
    pub fn find_discriminants(&self, union_type: TypeId) -> Vec<crate::solver::DiscriminantInfo> {
        use crate::solver::NarrowingContext;
        let ctx = NarrowingContext::new(&self.types);
        ctx.find_discriminants(union_type)
    }

    /// Narrow a type to include only members assignable to target.
    pub fn narrow_to_type(&self, source: TypeId, target: TypeId) -> TypeId {
        use crate::solver::NarrowingContext;
        let ctx = NarrowingContext::new(&self.types);
        ctx.narrow_to_type(source, target)
    }

    /// Narrow a type to exclude members assignable to target.
    pub fn narrow_excluding_type(&self, source: TypeId, excluded: TypeId) -> TypeId {
        use crate::solver::NarrowingContext;
        let ctx = NarrowingContext::new(&self.types);
        ctx.narrow_excluding_type(source, excluded)
    }

    // =========================================================================
    // Type Node Resolution
    // =========================================================================

    /// Get type from a type node.
    ///
    /// Uses compile-time constant TypeIds for intrinsic types (O(1) lookup).
    /// Delegates to TypeLowering for complex types (union, intersection, array, etc.).
    pub fn get_type_from_type_node(&mut self, idx: NodeIndex) -> TypeId {
        use crate::solver::TypeLowering;

        // Use TypeLowering which handles all type nodes
        let lowering = TypeLowering::new(self.arena, &self.types);
        lowering.lower_type(idx)
    }

    // =========================================================================
    // Source Location Tracking & Solver Diagnostics
    // =========================================================================

    /// Get a source location for a node.
    pub fn get_source_location(&self, idx: NodeIndex) -> Option<crate::solver::SourceLocation> {
        let node = self.arena.get(idx)?;
        Some(crate::solver::SourceLocation::new(
            self.file_name.as_str(),
            node.pos,
            node.end,
        ))
    }

    /// Report a type not assignable error using solver diagnostics with source tracking.
    pub fn error_type_not_assignable_at(
        &mut self,
        source: TypeId,
        target: TypeId,
        idx: NodeIndex,
    ) {
        if let Some(loc) = self.get_source_location(idx) {
            let mut builder = crate::solver::SpannedDiagnosticBuilder::new(
                &self.types,
                self.file_name.as_str(),
            );
            let diag = builder.type_not_assignable(source, target, loc.start, loc.length());
            self.diagnostics.push(diag.to_checker_diagnostic(&self.file_name));
        }
    }

    /// Report a property missing error using solver diagnostics with source tracking.
    pub fn error_property_missing_at(
        &mut self,
        prop_name: &str,
        source: TypeId,
        target: TypeId,
        idx: NodeIndex,
    ) {
        if let Some(loc) = self.get_source_location(idx) {
            let mut builder = crate::solver::SpannedDiagnosticBuilder::new(
                &self.types,
                self.file_name.as_str(),
            );
            let diag = builder.property_missing(prop_name, source, target, loc.start, loc.length());
            self.diagnostics.push(diag.to_checker_diagnostic(&self.file_name));
        }
    }

    /// Report a property not exist error using solver diagnostics with source tracking.
    pub fn error_property_not_exist_at(
        &mut self,
        prop_name: &str,
        type_id: TypeId,
        idx: NodeIndex,
    ) {
        if let Some(loc) = self.get_source_location(idx) {
            let mut builder = crate::solver::SpannedDiagnosticBuilder::new(
                &self.types,
                self.file_name.as_str(),
            );
            let diag = builder.property_not_exist(prop_name, type_id, loc.start, loc.length());
            self.diagnostics.push(diag.to_checker_diagnostic(&self.file_name));
        }
    }

    /// Report an argument not assignable error using solver diagnostics with source tracking.
    pub fn error_argument_not_assignable_at(
        &mut self,
        arg_type: TypeId,
        param_type: TypeId,
        idx: NodeIndex,
    ) {
        if let Some(loc) = self.get_source_location(idx) {
            let mut builder = crate::solver::SpannedDiagnosticBuilder::new(
                &self.types,
                self.file_name.as_str(),
            );
            let diag = builder.argument_not_assignable(arg_type, param_type, loc.start, loc.length());
            self.diagnostics.push(diag.to_checker_diagnostic(&self.file_name));
        }
    }

    /// Report a cannot find name error using solver diagnostics with source tracking.
    pub fn error_cannot_find_name_at(&mut self, name: &str, idx: NodeIndex) {
        if let Some(loc) = self.get_source_location(idx) {
            let mut builder = crate::solver::SpannedDiagnosticBuilder::new(
                &self.types,
                self.file_name.as_str(),
            );
            let diag = builder.cannot_find_name(name, loc.start, loc.length());
            self.diagnostics.push(diag.to_checker_diagnostic(&self.file_name));
        }
    }

    /// Report an argument count mismatch error using solver diagnostics with source tracking.
    pub fn error_argument_count_mismatch_at(
        &mut self,
        expected: usize,
        got: usize,
        idx: NodeIndex,
    ) {
        if let Some(loc) = self.get_source_location(idx) {
            let mut builder = crate::solver::SpannedDiagnosticBuilder::new(
                &self.types,
                self.file_name.as_str(),
            );
            let diag = builder.argument_count_mismatch(expected, got, loc.start, loc.length());
            self.diagnostics.push(diag.to_checker_diagnostic(&self.file_name));
        }
    }

    /// Create a diagnostic collector for batch error reporting.
    pub fn create_diagnostic_collector(&self) -> crate::solver::DiagnosticCollector {
        crate::solver::DiagnosticCollector::new(&self.types, self.file_name.as_str())
    }

    /// Merge diagnostics from a collector into the checker's diagnostics.
    pub fn merge_diagnostics(&mut self, collector: &crate::solver::DiagnosticCollector) {
        for diag in collector.to_checker_diagnostics() {
            self.diagnostics.push(diag);
        }
    }

    /// Format a type as a human-readable string using solver's TypeFormatter.
    pub fn format_type(&self, type_id: TypeId) -> String {
        let mut formatter = crate::solver::TypeFormatter::new(&self.types);
        formatter.format(type_id)
    }

    // =========================================================================
    // Source File Checking (Full Traversal)
    // =========================================================================

    /// Check a source file and populate diagnostics.
    /// This is the entry point for type checking a parsed and bound file.
    pub fn check_source_file(&mut self, root_idx: NodeIndex) {
        let Some(node) = self.arena.get(root_idx) else {
            return;
        };

        if let Some(sf) = self.arena.get_source_file(node) {
            // Type check each top-level statement
            for &stmt_idx in &sf.statements.nodes {
                self.check_statement(stmt_idx);
            }
        }
    }

    /// Check a statement and produce type errors.
    fn check_statement(&mut self, stmt_idx: NodeIndex) {
        let Some(node) = self.arena.get(stmt_idx) else {
            return;
        };

        match node.kind {
            syntax_kind_ext::VARIABLE_STATEMENT => {
                self.check_variable_statement(stmt_idx);
            }
            syntax_kind_ext::EXPRESSION_STATEMENT => {
                // Type-check the expression - get it from the expression pool
                // ExpressionStatement stores single expression at data_index
                self.get_type_of_node(stmt_idx);
            }
            syntax_kind_ext::IF_STATEMENT => {
                if let Some(if_data) = self.arena.get_if_statement(node) {
                    // Check condition
                    self.get_type_of_node(if_data.expression);
                    // Check then branch
                    self.check_statement(if_data.then_statement);
                    // Check else branch if present
                    if !if_data.else_statement.is_none() {
                        self.check_statement(if_data.else_statement);
                    }
                }
            }
            syntax_kind_ext::RETURN_STATEMENT => {
                // Return statement expression - just type check the node
                self.get_type_of_node(stmt_idx);
            }
            syntax_kind_ext::BLOCK => {
                if let Some(block) = self.arena.get_block(node) {
                    for &inner_stmt in &block.statements.nodes {
                        self.check_statement(inner_stmt);
                    }
                }
            }
            syntax_kind_ext::FUNCTION_DECLARATION => {
                if let Some(func) = self.arena.get_function(node) {
                    // Check function body if present
                    if !func.body.is_none() {
                        self.push_local_scope();

                        // Add parameters to local scope
                        for &param_idx in &func.parameters.nodes {
                            if let Some(param_node) = self.arena.get(param_idx) {
                                if let Some(param) = self.arena.get_parameter(param_node) {
                                    // Get parameter name
                                    if let Some(name_node) = self.arena.get(param.name) {
                                        if let Some(ident) = self.arena.get_identifier(name_node) {
                                            let param_type = if !param.type_annotation.is_none() {
                                                self.get_type_of_node(param.type_annotation)
                                            } else {
                                                TypeId::ANY
                                            };
                                            self.add_local(ident.escaped_text.clone(), param_type);
                                        }
                                    }
                                }
                            }
                        }

                        self.check_statement(func.body);
                        self.pop_local_scope();
                    }
                }
            }
            syntax_kind_ext::WHILE_STATEMENT | syntax_kind_ext::DO_STATEMENT => {
                if let Some(loop_data) = self.arena.get_loop(node) {
                    self.get_type_of_node(loop_data.condition);
                    self.check_statement(loop_data.statement);
                }
            }
            syntax_kind_ext::FOR_STATEMENT => {
                if let Some(loop_data) = self.arena.get_loop(node) {
                    if !loop_data.initializer.is_none() {
                        self.get_type_of_node(loop_data.initializer);
                    }
                    if !loop_data.condition.is_none() {
                        self.get_type_of_node(loop_data.condition);
                    }
                    if !loop_data.incrementor.is_none() {
                        self.get_type_of_node(loop_data.incrementor);
                    }
                    self.check_statement(loop_data.statement);
                }
            }
            syntax_kind_ext::FOR_IN_STATEMENT | syntax_kind_ext::FOR_OF_STATEMENT => {
                if let Some(for_data) = self.arena.get_for_in_of(node) {
                    self.get_type_of_node(for_data.initializer);
                    self.get_type_of_node(for_data.expression);
                    self.check_statement(for_data.statement);
                }
            }
            syntax_kind_ext::TRY_STATEMENT => {
                if let Some(try_data) = self.arena.get_try(node) {
                    self.check_statement(try_data.try_block);
                    if !try_data.catch_clause.is_none() {
                        if let Some(catch_node) = self.arena.get(try_data.catch_clause) {
                            if let Some(catch) = self.arena.get_catch_clause(catch_node) {
                                self.check_statement(catch.block);
                            }
                        }
                    }
                    if !try_data.finally_block.is_none() {
                        self.check_statement(try_data.finally_block);
                    }
                }
            }
            // Type declarations - just register them, no expression checking needed
            syntax_kind_ext::INTERFACE_DECLARATION |
            syntax_kind_ext::TYPE_ALIAS_DECLARATION |
            syntax_kind_ext::ENUM_DECLARATION |
            syntax_kind_ext::MODULE_DECLARATION |
            syntax_kind_ext::IMPORT_DECLARATION |
            syntax_kind_ext::EXPORT_DECLARATION |
            syntax_kind_ext::EMPTY_STATEMENT |
            syntax_kind_ext::DEBUGGER_STATEMENT |
            syntax_kind_ext::BREAK_STATEMENT |
            syntax_kind_ext::CONTINUE_STATEMENT => {
                // No action needed
            }
            syntax_kind_ext::CLASS_DECLARATION => {
                // TODO: Check class members
                self.get_type_of_node(stmt_idx);
            }
            _ => {
                // Catch-all for other statement types
                self.get_type_of_node(stmt_idx);
            }
        }
    }

    /// Check a variable statement (var/let/const declarations).
    fn check_variable_statement(&mut self, stmt_idx: NodeIndex) {
        let Some(node) = self.arena.get(stmt_idx) else {
            return;
        };

        if let Some(var) = self.arena.get_variable(node) {
            // VariableStatement.declarations contains VariableDeclarationList nodes
            for &list_idx in &var.declarations.nodes {
                self.check_variable_declaration_list(list_idx);
            }
        }
    }

    /// Check a variable declaration list (var/let/const x, y, z).
    fn check_variable_declaration_list(&mut self, list_idx: NodeIndex) {
        let Some(node) = self.arena.get(list_idx) else {
            return;
        };

        // VariableDeclarationList uses the same VariableData structure
        if let Some(var_list) = self.arena.get_variable(node) {
            // Now these are actual VariableDeclaration nodes
            for &decl_idx in &var_list.declarations.nodes {
                self.check_variable_declaration(decl_idx);
            }
        }
    }

    /// Check a single variable declaration.
    fn check_variable_declaration(&mut self, decl_idx: NodeIndex) {
        let Some(node) = self.arena.get(decl_idx) else {
            return;
        };

        let Some(var_decl) = self.arena.get_variable_declaration(node) else {
            return;
        };

        // Get declared type from type annotation
        let declared_type = if !var_decl.type_annotation.is_none() {
            self.get_type_of_node(var_decl.type_annotation)
        } else {
            TypeId::ANY
        };

        // Get inferred type from initializer
        if !var_decl.initializer.is_none() {
            let init_type = self.get_type_of_node(var_decl.initializer);

            // If there's a type annotation, check that initializer is assignable
            if !var_decl.type_annotation.is_none() && declared_type != TypeId::ANY {
                if !self.is_assignable_to(init_type, declared_type) {
                    // Report type error
                    self.error_type_not_assignable_at(init_type, declared_type, var_decl.initializer);
                }
            }
        }
    }
}
