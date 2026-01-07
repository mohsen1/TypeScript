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

use crate::parser::NodeIndex;
use crate::parser::thin_node::ThinNodeArena;
use crate::parser::syntax_kind_ext;
use crate::scanner::SyntaxKind;
use crate::binder::{SymbolId, symbol_flags};
use crate::thin_binder::ThinBinderState;
use crate::solver::{TypeId, TypeInterner, ContextualTypeContext};
use crate::checker::types::diagnostics::{Diagnostic, DiagnosticCategory};
use crate::checker::{CheckerContext, EnclosingClassInfo, FlowAnalyzer};
use crate::interner::Atom;

// =============================================================================
// ThinCheckerState
// =============================================================================

/// Type checker state using ThinNodeArena and Solver type system.
///
/// This is a performance-optimized checker that works directly with the
/// cache-friendly ThinNode architecture and uses the solver's TypeInterner
/// for structural type equality.
///
/// The state is stored in a `CheckerContext` which can be shared with
/// specialized checker modules (expressions, statements, declarations).
pub struct ThinCheckerState<'a> {
    /// Shared checker context containing all state.
    pub ctx: CheckerContext<'a>,
}

/// Maximum depth for recursive type instantiation.
pub const MAX_INSTANTIATION_DEPTH: u32 = 50;

/// Maximum depth for call expression resolution.
pub const MAX_CALL_DEPTH: u32 = 20;

impl<'a> ThinCheckerState<'a> {
    /// Create a new ThinCheckerState.
    ///
    /// # Arguments
    /// * `arena` - The AST node arena
    /// * `binder` - The binder state with symbols
    /// * `types` - The shared type interner (for thread-safe type deduplication)
    /// * `file_name` - The source file name
    pub fn new(
        arena: &'a ThinNodeArena,
        binder: &'a ThinBinderState,
        types: &'a TypeInterner,
        file_name: String,
    ) -> Self {
        ThinCheckerState {
            ctx: CheckerContext::new(arena, binder, types, file_name),
        }
    }

    /// Create a new ThinCheckerState with a persistent cache.
    /// This allows reusing type checking results from previous queries.
    ///
    /// # Arguments
    /// * `arena` - The AST node arena
    /// * `binder` - The binder state with symbols
    /// * `types` - The shared type interner
    /// * `file_name` - The source file name
    /// * `cache` - The persistent type cache from previous queries
    pub fn with_cache(
        arena: &'a ThinNodeArena,
        binder: &'a ThinBinderState,
        types: &'a TypeInterner,
        file_name: String,
        cache: crate::checker::TypeCache,
    ) -> Self {
        ThinCheckerState {
            ctx: CheckerContext::with_cache(arena, binder, types, file_name, cache),
        }
    }

    /// Extract the persistent cache from this checker.
    /// This allows saving type checking results for future queries.
    pub fn extract_cache(self) -> crate::checker::TypeCache {
        self.ctx.extract_cache()
    }

    // =========================================================================
    // Scope Management (delegated to CheckerContext)
    // =========================================================================

    /// Push a new local scope onto the stack.
    pub fn push_local_scope(&mut self) {
        self.ctx.push_local_scope();
    }

    /// Pop the current local scope from the stack.
    pub fn pop_local_scope(&mut self) {
        self.ctx.pop_local_scope();
    }

    /// Add a local variable to the current scope.
    pub fn add_local(&mut self, name: String, type_id: TypeId) {
        self.ctx.add_local(name, type_id);
    }

    /// Look up a local variable in all scopes.
    pub fn lookup_local(&self, name: &str) -> Option<TypeId> {
        self.ctx.lookup_local(name)
    }

    /// Push an expected return type onto the stack (when entering a function).
    pub fn push_return_type(&mut self, return_type: TypeId) {
        self.ctx.push_return_type(return_type);
    }

    /// Pop an expected return type from the stack (when exiting a function).
    pub fn pop_return_type(&mut self) {
        self.ctx.pop_return_type();
    }

    /// Get the current expected return type (if in a function).
    pub fn current_return_type(&self) -> Option<TypeId> {
        self.ctx.current_return_type()
    }

    // =========================================================================
    // Diagnostics (delegated to CheckerContext)
    // =========================================================================

    /// Add an error diagnostic.
    pub fn error(&mut self, start: u32, length: u32, message: String, code: u32) {
        self.ctx.error(start, length, message, code);
    }

    /// Get node span (pos, end) from index.
    pub fn get_node_span(&self, idx: NodeIndex) -> Option<(u32, u32)> {
        self.ctx.get_node_span(idx)
    }

    // =========================================================================
    // Symbol Resolution
    // =========================================================================

    /// Get the symbol for a node index.
    pub fn get_symbol_at_node(&self, idx: NodeIndex) -> Option<SymbolId> {
        self.ctx.binder.get_node_symbol(idx)
    }

    /// Get the symbol by name from file locals.
    pub fn get_symbol_by_name(&self, name: &str) -> Option<SymbolId> {
        self.ctx.binder.file_locals.get(name)
    }

    // =========================================================================
    // Type Resolution - Core Methods
    // =========================================================================

    /// Get the type of a node.
    pub fn get_type_of_node(&mut self, idx: NodeIndex) -> TypeId {
        // Check cache first
        if let Some(&cached) = self.ctx.node_types.get(&idx.0) {
            return cached;
        }

        // Check for circular reference
        if self.ctx.node_resolution_set.contains(&idx) {
            return TypeId::ANY;
        }

        // Push onto resolution stack
        self.ctx.node_resolution_stack.push(idx);
        self.ctx.node_resolution_set.insert(idx);

        let result = self.compute_type_of_node(idx);

        // Pop from resolution stack
        self.ctx.node_resolution_stack.pop();
        self.ctx.node_resolution_set.remove(&idx);

        // Cache result
        self.ctx.node_types.insert(idx.0, result);

        result
    }

    /// Compute the type of a node (internal, not cached).
    fn compute_type_of_node(&mut self, idx: NodeIndex) -> TypeId {
        let Some(node) = self.ctx.arena.get(idx) else {
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
            k if k == SyntaxKind::TrueKeyword as u16 => self.ctx.types.literal_boolean(true),
            k if k == SyntaxKind::FalseKeyword as u16 => self.ctx.types.literal_boolean(false),
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

            // Parenthesized expression - just pass through to inner expression
            k if k == syntax_kind_ext::PARENTHESIZED_EXPRESSION => {
                if let Some(paren) = self.ctx.arena.get_parenthesized(node) {
                    self.get_type_of_node(paren.expression)
                } else {
                    TypeId::ANY
                }
            }

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

            // Function type (e.g., () => number, (x: string) => void)
            k if k == syntax_kind_ext::FUNCTION_TYPE => {
                self.get_type_from_function_type(idx)
            }

            // Type query (typeof X) - returns the type of X
            k if k == syntax_kind_ext::TYPE_QUERY => {
                self.get_type_from_type_query(idx)
            }

            // Qualified name (A.B.C) - resolve namespace member access
            k if k == syntax_kind_ext::QUALIFIED_NAME => {
                self.resolve_qualified_name(idx)
            }

            // Default case
            _ => TypeId::ANY,
        }
    }

    /// Get type from a type reference node (e.g., "number", "string", "MyType").
    fn get_type_from_type_reference(&mut self, idx: NodeIndex) -> TypeId {
        let Some(node) = self.ctx.arena.get(idx) else {
            return TypeId::ANY;
        };

        // Get the TypeRefData from the arena
        let Some(type_ref) = self.ctx.arena.get_type_ref(node) else {
            return TypeId::ANY;
        };

        let type_name_idx = type_ref.type_name;
        let has_type_args = type_ref.type_arguments
            .as_ref()
            .map_or(false, |args| !args.nodes.is_empty());

        // Check if type_name is a qualified name (A.B)
        if let Some(name_node) = self.ctx.arena.get(type_name_idx) {
            if name_node.kind == syntax_kind_ext::QUALIFIED_NAME {
                if has_type_args {
                    if self.resolve_qualified_symbol(type_name_idx).is_none() {
                        let _ = self.resolve_qualified_name(type_name_idx);
                        return TypeId::ERROR;
                    }
                    let type_resolver = |node_idx: NodeIndex| self.resolve_type_symbol_for_lowering(node_idx);
                    let value_resolver = |node_idx: NodeIndex| self.resolve_value_symbol_for_lowering(node_idx);
                    let lowering = crate::solver::TypeLowering::with_resolvers(
                        self.ctx.arena,
                        self.ctx.types,
                        &type_resolver,
                        &value_resolver,
                    );
                    return lowering.lower_type(idx);
                }
                return self.resolve_qualified_name(type_name_idx);
            }
        }

        // Get the identifier for the type name
        if let Some(name_node) = self.ctx.arena.get(type_name_idx) {
            if let Some(ident) = self.ctx.arena.get_identifier(name_node) {
                let name = ident.escaped_text.as_str();

                if has_type_args {
                    let is_builtin_array = name == "Array" || name == "ReadonlyArray";
                    if !is_builtin_array
                        && self.lookup_local(name).is_none()
                        && !self.has_named_type_symbol(name)
                    {
                        self.error_cannot_find_name_at(name, type_name_idx);
                        return TypeId::ERROR;
                    }
                    let type_resolver = |node_idx: NodeIndex| self.resolve_type_symbol_for_lowering(node_idx);
                    let value_resolver = |node_idx: NodeIndex| self.resolve_value_symbol_for_lowering(node_idx);
                    let lowering = crate::solver::TypeLowering::with_resolvers(
                        self.ctx.arena,
                        self.ctx.types,
                        &type_resolver,
                        &value_resolver,
                    );
                    return lowering.lower_type(idx);
                }

                if name == "Array" || name == "ReadonlyArray" {
                    if let Some(type_id) = self.resolve_named_type_reference(name) {
                        return type_id;
                    }
                    let elem_type = type_ref.type_arguments
                        .as_ref()
                        .and_then(|args| args.nodes.first().copied())
                        .map(|idx| self.get_type_from_type_node(idx))
                        .unwrap_or(TypeId::ANY);
                    let array_type = self.ctx.types.array(elem_type);
                    if name == "ReadonlyArray" {
                        return self.ctx.types.intern(crate::solver::TypeKey::ReadonlyType(array_type));
                    }
                    return array_type;
                }

                // Check for built-in types
                match name {
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
                    _ => {}
                }

                if let Some(type_id) = self.resolve_named_type_reference(name) {
                    return type_id;
                }
                self.error_cannot_find_name_at(name, type_name_idx);
                return TypeId::ERROR;
            }
        }

        TypeId::ANY
    }

    fn resolve_named_type_reference(&mut self, name: &str) -> Option<TypeId> {
        // Check local scopes first (includes type parameters)
        if let Some(type_id) = self.lookup_local(name) {
            return Some(type_id);
        }
        // Check file locals
        if let Some(sym_id) = self.ctx.binder.file_locals.get(name) {
            return Some(self.get_type_of_symbol(sym_id));
        }
        // Check all symbols (excluding class members)
        if let Some(sym_id) = self.ctx.binder.get_symbols().find_by_name(name) {
            if let Some(symbol) = self.ctx.binder.get_symbol(sym_id) {
                use crate::binder::symbol_flags;
                let is_class_member = (symbol.flags & symbol_flags::PROPERTY) != 0
                    || (symbol.flags & symbol_flags::METHOD) != 0;
                if !is_class_member {
                    return Some(self.get_type_of_symbol(sym_id));
                }
            }
        }
        None
    }

    fn has_named_type_symbol(&self, name: &str) -> bool {
        if self.ctx.binder.file_locals.get(name).is_some() {
            return true;
        }
        if let Some(sym_id) = self.ctx.binder.get_symbols().find_by_name(name) {
            if let Some(symbol) = self.ctx.binder.get_symbol(sym_id) {
                let is_class_member = (symbol.flags & symbol_flags::PROPERTY) != 0
                    || (symbol.flags & symbol_flags::METHOD) != 0;
                return !is_class_member;
            }
        }
        false
    }

    /// Resolve a qualified name or identifier to a symbol ID.
    fn resolve_qualified_symbol(&self, idx: NodeIndex) -> Option<SymbolId> {
        let node = self.ctx.arena.get(idx)?;

        if node.kind == SyntaxKind::Identifier as u16 {
            return self.ctx.binder.resolve_identifier(self.ctx.arena, idx);
        }

        if node.kind != syntax_kind_ext::QUALIFIED_NAME {
            return None;
        }

        let qn = self.ctx.arena.get_qualified_name(node)?;
        let left_sym = self.resolve_qualified_symbol(qn.left)?;
        let right_name = self.ctx.arena.get(qn.right)
            .and_then(|node| self.ctx.arena.get_identifier(node))
            .map(|ident| ident.escaped_text.as_str())?;

        let left_symbol = self.ctx.binder.get_symbol(left_sym)?;
        let exports = left_symbol.exports.as_ref()?;
        exports.get(right_name)
    }

    fn entity_name_text(&self, idx: NodeIndex) -> Option<String> {
        let node = self.ctx.arena.get(idx)?;
        if node.kind == SyntaxKind::Identifier as u16 {
            return self.ctx.arena.get_identifier(node).map(|ident| ident.escaped_text.clone());
        }
        if node.kind == syntax_kind_ext::QUALIFIED_NAME {
            let qn = self.ctx.arena.get_qualified_name(node)?;
            let left = self.entity_name_text(qn.left)?;
            let right = self.entity_name_text(qn.right)?;
            let mut combined = String::with_capacity(left.len() + 1 + right.len());
            combined.push_str(&left);
            combined.push('.');
            combined.push_str(&right);
            return Some(combined);
        }
        None
    }

    fn resolve_type_symbol_for_lowering(&self, idx: NodeIndex) -> Option<u32> {
        let sym_id = self.resolve_qualified_symbol(idx)?;
        let symbol = self.ctx.binder.get_symbol(sym_id)?;
        if (symbol.flags & symbol_flags::TYPE) != 0 {
            Some(sym_id.0)
        } else {
            None
        }
    }

    fn resolve_value_symbol_for_lowering(&self, idx: NodeIndex) -> Option<u32> {
        let sym_id = self.resolve_qualified_symbol(idx)?;
        let symbol = self.ctx.binder.get_symbol(sym_id)?;
        if (symbol.flags & (symbol_flags::VALUE | symbol_flags::ALIAS)) != 0 {
            Some(sym_id.0)
        } else {
            None
        }
    }

    /// Resolve a qualified name (A.B) to a type.
    /// Returns the type of the rightmost member, or reports TS2694 if not found.
    fn resolve_qualified_name(&mut self, idx: NodeIndex) -> TypeId {
        let Some(node) = self.ctx.arena.get(idx) else {
            return TypeId::ANY;
        };

        let Some(qn) = self.ctx.arena.get_qualified_name(node) else {
            return TypeId::ANY;
        };

        // Resolve the left side (could be Identifier or another QualifiedName)
        let left_type = if let Some(left_node) = self.ctx.arena.get(qn.left) {
            if left_node.kind == syntax_kind_ext::QUALIFIED_NAME {
                self.resolve_qualified_name(qn.left)
            } else if left_node.kind == SyntaxKind::Identifier as u16 {
                // Resolve identifier as a type reference
                self.get_type_from_type_reference_by_name(qn.left)
            } else {
                TypeId::ANY
            }
        } else {
            TypeId::ANY
        };

        if left_type == TypeId::ANY || left_type == TypeId::ERROR {
            return TypeId::ANY;
        }

        // Get the right side name (B in A.B)
        let right_name = if let Some(right_node) = self.ctx.arena.get(qn.right) {
            if let Some(id) = self.ctx.arena.get_identifier(right_node) {
                id.escaped_text.clone()
            } else {
                return TypeId::ANY;
            }
        } else {
            return TypeId::ANY;
        };

        // Look up the member in the left side's exports
        if let Some(crate::solver::TypeKey::Ref(crate::solver::SymbolRef(sym_id))) = self.ctx.types.lookup(left_type) {
            if let Some(symbol) = self.ctx.binder.get_symbol(crate::binder::SymbolId(sym_id)) {
                // Check exports table
                if let Some(ref exports) = symbol.exports {
                    if let Some(member_sym_id) = exports.get(&right_name) {
                        return self.get_type_of_symbol(member_sym_id);
                    }
                }

                // Not found - report TS2694
                self.error_namespace_no_export(&symbol.escaped_name, &right_name, qn.right);
                return TypeId::ERROR;
            }
        }

        // Left side wasn't a reference to a namespace/module
        TypeId::ANY
    }

    /// Helper to resolve an identifier as a type reference (for qualified name left sides).
    fn get_type_from_type_reference_by_name(&mut self, idx: NodeIndex) -> TypeId {
        let Some(node) = self.ctx.arena.get(idx) else {
            return TypeId::ANY;
        };

        if let Some(ident) = self.ctx.arena.get_identifier(node) {
            let name = &ident.escaped_text;

            // Check local scopes
            if let Some(type_id) = self.lookup_local(name) {
                return type_id;
            }
            // Check file locals
            if let Some(sym_id) = self.ctx.binder.file_locals.get(name) {
                return self.get_type_of_symbol(sym_id);
            }
            // Check all symbols
            if let Some(sym_id) = self.ctx.binder.get_symbols().find_by_name(name) {
                return self.get_type_of_symbol(sym_id);
            }

            // Not found
            self.error_cannot_find_name_at(name, idx);
            return TypeId::ERROR;
        }

        TypeId::ANY
    }

    /// Get type from a union type node (A | B).
    fn get_type_from_union_type(&mut self, idx: NodeIndex) -> TypeId {
        let Some(node) = self.ctx.arena.get(idx) else {
            return TypeId::ANY;
        };

        // UnionType uses CompositeTypeData which has a types list
        if let Some(composite) = self.ctx.arena.get_composite_type(node) {
            let mut member_types = Vec::new();
            for &type_idx in &composite.types.nodes {
                // Use get_type_from_type_node to properly resolve typeof expressions via binder
                member_types.push(self.get_type_from_type_node(type_idx));
            }

            if member_types.is_empty() {
                return TypeId::NEVER;
            }
            if member_types.len() == 1 {
                return member_types[0];
            }

            return self.ctx.types.union(member_types);
        }

        TypeId::ANY
    }

    /// Get type from a type query node (typeof X).
    /// Creates a TypeQuery type with the actual SymbolId from the binder.
    fn get_type_from_type_query(&mut self, idx: NodeIndex) -> TypeId {
        use crate::solver::{TypeKey, SymbolRef};

        let Some(node) = self.ctx.arena.get(idx) else {
            return TypeId::ANY;
        };

        let Some(type_query) = self.ctx.arena.get_type_query(node) else {
            return TypeId::ANY;
        };

        let base = if let Some(sym_id) = self.resolve_value_symbol_for_lowering(type_query.expr_name) {
            self.ctx.types.intern(TypeKey::TypeQuery(SymbolRef(sym_id)))
        } else if let Some(name) = self.entity_name_text(type_query.expr_name) {
            // Not found - fall back to hash (for forward compatibility)
            use std::hash::{Hash, Hasher};
            use std::collections::hash_map::DefaultHasher;
            let mut hasher = DefaultHasher::new();
            name.hash(&mut hasher);
            let symbol_id = hasher.finish() as u32;
            self.ctx.types.intern(TypeKey::TypeQuery(SymbolRef(symbol_id)))
        } else {
            return TypeId::ANY;
        };

        if let Some(args) = &type_query.type_arguments {
            if !args.nodes.is_empty() {
                let type_args = args.nodes.iter()
                    .map(|&idx| self.get_type_from_type_node(idx))
                    .collect();
                return self.ctx.types.application(base, type_args);
            }
        }

        base
    }

    /// Get type from an array type node (T[]).
    fn get_type_from_array_type(&mut self, idx: NodeIndex) -> TypeId {
        let Some(node) = self.ctx.arena.get(idx) else {
            return TypeId::ANY;
        };

        // ArrayType uses TypeOperatorData which has the element type
        if let Some(type_op) = self.ctx.arena.get_type_operator(node) {
            let elem_type = self.get_type_of_node(type_op.type_node);
            return self.ctx.types.array(elem_type);
        }

        self.ctx.types.array(TypeId::ANY)
    }

    /// Get type from a function type node (e.g., () => number, (x: string) => void).
    fn get_type_from_function_type(&mut self, idx: NodeIndex) -> TypeId {
        use crate::solver::TypeLowering;

        let type_resolver = |node_idx: NodeIndex| self.resolve_type_symbol_for_lowering(node_idx);
        let value_resolver = |node_idx: NodeIndex| self.resolve_value_symbol_for_lowering(node_idx);
        let lowering = TypeLowering::with_resolvers(
            self.ctx.arena,
            self.ctx.types,
            &type_resolver,
            &value_resolver,
        );

        lowering.lower_type(idx)
    }

    /// Get type of an interface declaration.
    /// This extracts call signatures, construct signatures, and properties
    /// to build a callable type if the interface has call signatures.
    fn get_type_of_interface(&mut self, idx: NodeIndex) -> TypeId {
        use crate::solver::{CallSignature as SolverCallSignature, CallableShape, PropertyInfo, TypeKey};
        use crate::parser::syntax_kind_ext::{CALL_SIGNATURE, CONSTRUCT_SIGNATURE, PROPERTY_SIGNATURE, METHOD_SIGNATURE, HERITAGE_CLAUSE, EXPRESSION_WITH_TYPE_ARGUMENTS};
        use std::sync::Arc;

        let Some(node) = self.ctx.arena.get(idx) else {
            return TypeId::ANY;
        };

        let Some(interface) = self.ctx.arena.get_interface(node) else {
            return TypeId::ANY;
        };

        let mut call_signatures: Vec<SolverCallSignature> = Vec::new();
        let mut construct_signatures: Vec<SolverCallSignature> = Vec::new();
        let mut properties: Vec<PropertyInfo> = Vec::new();

        // First, collect signatures from base interfaces (heritage clauses)
        if let Some(ref heritage_clauses) = interface.heritage_clauses {
            for &clause_idx in &heritage_clauses.nodes {
                let Some(clause_node) = self.ctx.arena.get(clause_idx) else {
                    continue;
                };

                // Heritage clause contains a list of types (expression with type args)
                if clause_node.kind == HERITAGE_CLAUSE {
                    if let Some(heritage_data) = self.ctx.arena.get_heritage_clause(clause_node) {
                        for &type_idx in &heritage_data.types.nodes {
                            // Each type is an ExpressionWithTypeArguments
                            let base_type = self.get_type_of_node(type_idx);

                            // If the base type is callable, merge its signatures
                            if let Some(TypeKey::Callable(base_shape)) = self.ctx.types.lookup(base_type) {
                                call_signatures.extend(base_shape.call_signatures.iter().cloned());
                                construct_signatures.extend(base_shape.construct_signatures.iter().cloned());
                                properties.extend(base_shape.properties.iter().cloned());
                            }
                        }
                    }
                }
            }
        }

        // Iterate over this interface's own members
        for &member_idx in &interface.members.nodes {
            let Some(member_node) = self.ctx.arena.get(member_idx) else {
                continue;
            };

            if member_node.kind == CALL_SIGNATURE {
                // Extract call signature
                if let Some(sig) = self.ctx.arena.get_signature(member_node) {
                    let (params, this_type) = self.extract_params_from_signature(sig);
                    let return_type = if !sig.type_annotation.is_none() {
                        self.get_type_of_node(sig.type_annotation)
                    } else {
                        TypeId::ANY
                    };

                    call_signatures.push(SolverCallSignature {
                        type_params: Vec::new(), // TODO: Handle type parameters
                        params,
                        this_type,
                        return_type,
                        type_predicate: None,
                    });
                }
            } else if member_node.kind == CONSTRUCT_SIGNATURE {
                // Extract construct signature
                if let Some(sig) = self.ctx.arena.get_signature(member_node) {
                    let (params, this_type) = self.extract_params_from_signature(sig);
                    let return_type = if !sig.type_annotation.is_none() {
                        self.get_type_of_node(sig.type_annotation)
                    } else {
                        TypeId::ANY
                    };

                    construct_signatures.push(SolverCallSignature {
                        type_params: Vec::new(),
                        params,
                        this_type,
                        return_type,
                        type_predicate: None,
                    });
                }
            } else if member_node.kind == PROPERTY_SIGNATURE || member_node.kind == METHOD_SIGNATURE {
                // Extract property
                if let Some(sig) = self.ctx.arena.get_signature(member_node) {
                    if let Some(name_node) = self.ctx.arena.get(sig.name) {
                        if let Some(id_data) = self.ctx.arena.get_identifier(name_node) {
                            let type_id = if !sig.type_annotation.is_none() {
                                self.get_type_of_node(sig.type_annotation)
                            } else {
                                TypeId::ANY
                            };

                            properties.push(PropertyInfo {
                                name: self.ctx.types.intern_string(&id_data.escaped_text),
                                type_id,
                                optional: sig.question_token,
                                readonly: false, // TODO: Check for readonly modifier,
                                is_method: member_node.kind == METHOD_SIGNATURE,
                            });
                        }
                    }
                }
            }
        }

        // If we have call signatures, build a callable type
        if !call_signatures.is_empty() || !construct_signatures.is_empty() {
            let shape = CallableShape {
                call_signatures,
                construct_signatures,
                properties,
            };
            return self.ctx.types.callable(shape);
        }

        // Otherwise, just return an object type with the properties
        if !properties.is_empty() {
            return self.ctx.types.object(properties);
        }

        TypeId::ANY
    }

    /// Helper to extract parameters from a SignatureData.
    fn extract_params_from_signature(
        &mut self,
        sig: &crate::parser::thin_node::SignatureData,
    ) -> (Vec<crate::solver::ParamInfo>, Option<TypeId>) {
        use crate::solver::ParamInfo;
        use std::sync::Arc;

        let Some(ref params_list) = sig.parameters else {
            return (Vec::new(), None);
        };

        let mut params = Vec::new();
        let mut this_type = None;
        let this_atom = self.ctx.types.intern_string("this");

        for &param_idx in &params_list.nodes {
            let Some(param_node) = self.ctx.arena.get(param_idx) else { continue };
            let Some(param) = self.ctx.arena.get_parameter(param_node) else { continue };

            let name: Option<Atom> = if let Some(name_node) = self.ctx.arena.get(param.name) {
                if let Some(name_data) = self.ctx.arena.get_identifier(name_node) {
                    Some(self.ctx.types.intern_string(&name_data.escaped_text))
                } else {
                    None
                }
            } else {
                None
            };

            let type_id = if !param.type_annotation.is_none() {
                self.get_type_of_node(param.type_annotation)
            } else {
                TypeId::ANY
            };

            let optional = param.question_token || !param.initializer.is_none();
            let rest = param.dot_dot_dot_token;

            if let Some(name_atom) = name {
                if name_atom == this_atom {
                    if this_type.is_none() {
                        this_type = Some(type_id);
                    }
                    continue;
                }
            }

            params.push(ParamInfo { name, type_id, optional, rest });
        }

        (params, this_type)
    }

    // =========================================================================
    // Type Resolution - Specific Node Types
    // =========================================================================

    /// Get type of identifier, with control flow analysis for narrowing.
    fn get_type_of_identifier(&mut self, idx: NodeIndex) -> TypeId {
        let Some(node) = self.ctx.arena.get(idx) else {
            return TypeId::ANY;
        };

        let Some(ident) = self.ctx.arena.get_identifier(node) else {
            return TypeId::ANY;
        };

        // LEGACY FALLBACK: Try old methods for backward compatibility during transition
        let name = &ident.escaped_text;

        // NEW STATELESS APPROACH: Query binder's persistent scope system
        // This enables lazy checking without traversal-order dependency
        if let Some(sym_id) = self.ctx.binder.resolve_identifier(self.ctx.arena, idx) {
            // Get the declared type of the symbol
            let declared_type = self.get_type_of_symbol(sym_id);
            if declared_type != TypeId::ANY {
                return self.apply_flow_narrowing(idx, declared_type);
            }
            if let Some(local_type) = self.lookup_local(name) {
                return self.apply_flow_narrowing(idx, local_type);
            }
            return declared_type;
        }

        // Check local scopes first to get the declared type
        if let Some(declared_type) = self.lookup_local(name) {
            // Apply control flow analysis to narrow the type
            return self.apply_flow_narrowing(idx, declared_type);
        }

        // Check file locals
        if let Some(sym_id) = self.ctx.binder.file_locals.get(name) {
            return self.get_type_of_symbol(sym_id);
        }

        // Check all symbols by name (handles nested scopes like classes in IIFEs)
        // But skip class members (PROPERTY/METHOD) - they need "this." prefix
        if let Some(sym_id) = self.ctx.binder.get_symbols().find_by_name(name) {
            if let Some(symbol) = self.ctx.binder.get_symbol(sym_id) {
                use crate::binder::symbol_flags;
                // Only use this fallback for classes, functions, variables - not class members
                let is_class_member = (symbol.flags & symbol_flags::PROPERTY) != 0
                    || (symbol.flags & symbol_flags::METHOD) != 0;
                if !is_class_member {
                    return self.get_type_of_symbol(sym_id);
                }
            }
        }

        // Intrinsic names - use constant TypeIds
        match name.as_str() {
            "undefined" => TypeId::UNDEFINED,
            "NaN" | "Infinity" => TypeId::NUMBER,
            // Symbol constructor - synthesize proper type for call signature validation
            "Symbol" => self.get_symbol_constructor_type(),
            // Global objects that are always available
            "console" | "Math" | "JSON" | "Object" | "Array" | "String"
            | "Number" | "Boolean" | "Date" | "RegExp" | "Error" | "Promise"
            | "Map" | "Set" | "WeakMap" | "WeakSet" | "WeakRef" | "Proxy"
            | "Reflect" | "globalThis" | "window" | "document"
            | "FinalizationRegistry" | "BigInt" | "ArrayBuffer" | "SharedArrayBuffer"
            | "DataView" | "Int8Array" | "Uint8Array" | "Uint8ClampedArray"
            | "Int16Array" | "Uint16Array" | "Int32Array" | "Uint32Array"
            | "Float32Array" | "Float64Array" | "BigInt64Array" | "BigUint64Array"
            | "Intl" | "Atomics" | "WebAssembly" | "Iterator" | "AsyncIterator"
            | "Generator" | "AsyncGenerator" | "URL" | "URLSearchParams"
            | "Headers" | "Request" | "Response" | "FormData" | "Blob" | "File"
            | "ReadableStream" | "WritableStream" | "TransformStream"
            | "TextEncoder" | "TextDecoder" | "AbortController" | "AbortSignal"
            | "fetch" | "setTimeout" | "setInterval" | "clearTimeout" | "clearInterval"
            | "queueMicrotask" | "structuredClone" | "atob" | "btoa"
            | "performance" | "crypto" | "navigator" | "location" | "history" => TypeId::ANY,
            _ => {
                // Check if we're inside a class and the name matches a static member (error 2662)
                // Clone values to avoid borrow issues
                if let Some(ref class_info) = self.ctx.enclosing_class.clone() {
                    if self.is_static_member(&class_info.member_nodes, name) {
                        self.error_cannot_find_name_static_member_at(
                            name,
                            &class_info.name,
                            idx,
                        );
                        return TypeId::ERROR;
                    }
                }
                // Report "cannot find name" error
                self.error_cannot_find_name_at(name, idx);
                TypeId::ERROR
            }
        }
    }

    /// Synthesize the Symbol constructor type.
    ///
    /// Returns a callable type with signature: `Symbol(description?: string | number): symbol`
    /// Note: Symbol cannot be constructed with `new`, so no construct signatures.
    fn get_symbol_constructor_type(&self) -> TypeId {
        use crate::solver::{CallSignature, CallableShape, ParamInfo};

        // Parameter: description?: string | number
        let description_param_type = self.ctx.types.union(vec![TypeId::STRING, TypeId::NUMBER]);
        let description_param = ParamInfo {
            name: Some(self.ctx.types.intern_string("description")),
            type_id: description_param_type,
            optional: true,
            rest: false,
        };

        // Call signature: (description?: string | number) => symbol
        let call_sig = CallSignature {
            type_params: vec![],
            params: vec![description_param],
            this_type: None,
            return_type: TypeId::SYMBOL,
            type_predicate: None,
        };

        // Callable shape (no construct signatures - can't use `new Symbol()`)
        let shape = CallableShape {
            call_signatures: vec![call_sig],
            construct_signatures: vec![],
            properties: vec![], // Could add Symbol.for, Symbol.keyFor, etc. later
        };

        self.ctx.types.callable(shape)
    }

    /// Apply control flow narrowing to a type at a specific identifier usage.
    ///
    /// This walks backwards through the control flow graph to determine what
    /// type guards (typeof, null checks, etc.) have been applied.
    fn apply_flow_narrowing(&self, idx: NodeIndex, declared_type: TypeId) -> TypeId {
        // Get the flow node for this identifier usage
        let flow_node = match self.ctx.binder.get_node_flow(idx) {
            Some(flow) => flow,
            None => return declared_type, // No flow info - use declared type
        };

        // Skip narrowing for non-union types (nothing to narrow)
        // Also skip for primitives that can't be narrowed further
        if !self.is_narrowable_type(declared_type) {
            return declared_type;
        }

        // Create a flow analyzer and apply narrowing
        let analyzer = FlowAnalyzer::new(
            self.ctx.arena,
            self.ctx.binder,
            self.ctx.types,
        );

        analyzer.get_flow_type(idx, declared_type, flow_node)
    }

    /// Check if a type can be narrowed (unions, nullable types, etc.)
    fn is_narrowable_type(&self, type_id: TypeId) -> bool {
        use crate::solver::TypeKey;

        // Check if it's a union type
        if let Some(TypeKey::Union(_)) = self.ctx.types.lookup(type_id) {
            return true;
        }

        // Could also check for types that include null/undefined
        // For now, only narrow unions
        false
    }

    /// Get type of a symbol.
    pub fn get_type_of_symbol(&mut self, sym_id: SymbolId) -> TypeId {
        // Check cache first
        if let Some(&cached) = self.ctx.symbol_types.get(&sym_id) {
            return cached;
        }

        // Check for circular reference
        if self.ctx.symbol_resolution_set.contains(&sym_id) {
            return TypeId::ANY;
        }

        // Push onto resolution stack
        self.ctx.symbol_resolution_stack.push(sym_id);
        self.ctx.symbol_resolution_set.insert(sym_id);

        let result = self.compute_type_of_symbol(sym_id);

        // Pop from resolution stack
        self.ctx.symbol_resolution_stack.pop();
        self.ctx.symbol_resolution_set.remove(&sym_id);

        // Cache result
        self.ctx.symbol_types.insert(sym_id, result);

        result
    }

    /// Compute type of a symbol (internal, not cached).
    ///
    /// Uses TypeLowering to bridge symbol declarations to solver types.
    fn compute_type_of_symbol(&mut self, sym_id: SymbolId) -> TypeId {
        use crate::solver::{TypeLowering, TypeKey, SymbolRef};

        let Some(symbol) = self.ctx.binder.get_symbol(sym_id) else {
            return TypeId::ANY;
        };

        let flags = symbol.flags;
        let value_decl = symbol.value_declaration;

        // Namespace / Module
        // Return a Ref type so resolve_qualified_name can access the symbol's exports
        if flags & (symbol_flags::NAMESPACE_MODULE | symbol_flags::VALUE_MODULE) != 0 {
            // Note: We use the symbol ID directly.
            // For merged declarations, this ID points to the unified symbol in the binder.
            return self.ctx.types.intern(TypeKey::Ref(SymbolRef(sym_id.0)));
        }

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

        // Interface - return interface type with call signatures
        if flags & symbol_flags::INTERFACE != 0 {
            if !symbol.declarations.is_empty() {
                let type_resolver = |node_idx: NodeIndex| self.resolve_type_symbol_for_lowering(node_idx);
                let value_resolver = |node_idx: NodeIndex| self.resolve_value_symbol_for_lowering(node_idx);
                let lowering = TypeLowering::with_resolvers(
                    self.ctx.arena,
                    self.ctx.types,
                    &type_resolver,
                    &value_resolver,
                );
                return lowering.lower_interface_declarations(&symbol.declarations);
            }
            if !value_decl.is_none() {
                return self.get_type_of_interface(value_decl);
            }
            return TypeId::ANY;
        }

        // Type alias - resolve using checker's get_type_from_type_node to properly resolve symbols
        if flags & symbol_flags::TYPE_ALIAS != 0 {
            // Get the type node from the type alias declaration
            let decl_idx = if !value_decl.is_none() {
                value_decl
            } else {
                symbol.declarations.first().copied().unwrap_or(NodeIndex::NONE)
            };
            if !decl_idx.is_none() {
                if let Some(node) = self.ctx.arena.get(decl_idx) {
                    if let Some(type_alias) = self.ctx.arena.get_type_alias(node) {
                        // Use checker's type resolution which can resolve type references through binder
                        return self.get_type_from_type_node(type_alias.type_node);
                    }
                }
            }
            return TypeId::ANY;
        }

        // Variable - get type from annotation or infer from initializer
        if flags & (symbol_flags::FUNCTION_SCOPED_VARIABLE | symbol_flags::BLOCK_SCOPED_VARIABLE) != 0 {
            if !value_decl.is_none() {
                if let Some(node) = self.ctx.arena.get(value_decl) {
                    if let Some(var_decl) = self.ctx.arena.get_variable_declaration(node) {
                        // First try type annotation - use get_type_of_node to resolve type references
                        // through the binder (for interfaces, classes, etc.)
                        if !var_decl.type_annotation.is_none() {
                            return self.get_type_of_node(var_decl.type_annotation);
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
                if let Some(node) = self.ctx.arena.get(value_decl) {
                        if let Some(param) = self.ctx.arena.get_parameter(node) {
                        if !param.type_annotation.is_none() {
                            let type_resolver = |node_idx: NodeIndex| self.resolve_type_symbol_for_lowering(node_idx);
                            let value_resolver = |node_idx: NodeIndex| self.resolve_value_symbol_for_lowering(node_idx);
                            let lowering = TypeLowering::with_resolvers(
                                self.ctx.arena,
                                self.ctx.types,
                                &type_resolver,
                                &value_resolver,
                            );
                            return lowering.lower_type(param.type_annotation);
                        }
                    }
                }
            }
            return TypeId::ANY;
        }

        // Alias - resolve the aliased type (import x = ns.member or ES6 imports)
        if flags & symbol_flags::ALIAS != 0 {
            if !value_decl.is_none() {
                if let Some(node) = self.ctx.arena.get(value_decl) {
                    // Handle Import Equals Declaration (import x = ns.member)
                    if node.kind == syntax_kind_ext::IMPORT_EQUALS_DECLARATION {
                        if let Some(import) = self.ctx.arena.get_import_decl(node) {
                            // module_specifier holds the reference (e.g., 'ns.member' or require("..."))
                            // Resolve it to get the aliased type
                            return self.get_type_of_node(import.module_specifier);
                        }
                    }
                    // Handle ES6 named imports - these are already handled by IMPORT_DECLARATION
                    // but fall through to ANY for now as they need module resolution
                }
            }
            return TypeId::ANY;
        }

        TypeId::ANY
    }

    /// Get type of binary expression.
    fn get_type_of_binary_expression(&mut self, idx: NodeIndex) -> TypeId {
        use crate::solver::{BinaryOpEvaluator, BinaryOpResult};

        let Some(node) = self.ctx.arena.get(idx) else {
            return TypeId::ANY;
        };

        let Some(binary) = self.ctx.arena.get_binary_expr(node) else {
            return TypeId::ANY;
        };

        let op_kind = binary.operator_token;

        // Special case: Assignment operator (not a type operation)
        if op_kind == SyntaxKind::EqualsToken as u16 {
            // Check for readonly property assignment
            self.check_readonly_assignment(binary.left, idx);
            return self.get_type_of_node(binary.right);
        }

        // Get operand types
        let left_type = self.get_type_of_node(binary.left);
        let right_type = self.get_type_of_node(binary.right);

        // Map SyntaxKind to operation string
        let op_str = match op_kind {
            k if k == SyntaxKind::PlusToken as u16 => "+",
            k if k == SyntaxKind::MinusToken as u16 => "-",
            k if k == SyntaxKind::AsteriskToken as u16 => "*",
            k if k == SyntaxKind::SlashToken as u16 => "/",
            k if k == SyntaxKind::PercentToken as u16 => "%",
            k if k == SyntaxKind::LessThanToken as u16 => "<",
            k if k == SyntaxKind::GreaterThanToken as u16 => ">",
            k if k == SyntaxKind::LessThanEqualsToken as u16 => "<=",
            k if k == SyntaxKind::GreaterThanEqualsToken as u16 => ">=",
            k if k == SyntaxKind::EqualsEqualsToken as u16 => "==",
            k if k == SyntaxKind::ExclamationEqualsToken as u16 => "!=",
            k if k == SyntaxKind::EqualsEqualsEqualsToken as u16 => "===",
            k if k == SyntaxKind::ExclamationEqualsEqualsToken as u16 => "!==",
            k if k == SyntaxKind::AmpersandAmpersandToken as u16 => "&&",
            k if k == SyntaxKind::BarBarToken as u16 => "||",
            // Bitwise operators - for now, return number directly
            k if k == SyntaxKind::AmpersandToken as u16
                || k == SyntaxKind::BarToken as u16
                || k == SyntaxKind::CaretToken as u16
                || k == SyntaxKind::LessThanLessThanToken as u16
                || k == SyntaxKind::GreaterThanGreaterThanToken as u16
                || k == SyntaxKind::GreaterThanGreaterThanGreaterThanToken as u16 => return TypeId::NUMBER,
            _ => return TypeId::ANY,
        };

        // Use BinaryOpEvaluator to resolve the operation
        let evaluator = BinaryOpEvaluator::new(self.ctx.types);
        let result = evaluator.evaluate(left_type, right_type, op_str);

        match result {
            BinaryOpResult::Success(result_type) => result_type,

            BinaryOpResult::TypeError { .. } => {
                // For now, return any instead of error for binary op mismatches
                // TypeScript is lenient with many binary operations
                TypeId::ANY
            }
        }
    }

    /// Get type of variable declaration.
    fn get_type_of_variable_declaration(&mut self, idx: NodeIndex) -> TypeId {
        let Some(node) = self.ctx.arena.get(idx) else {
            return TypeId::ANY;
        };

        let Some(var_decl) = self.ctx.arena.get_variable_declaration(node) else {
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
        use crate::solver::{CallEvaluator, CallResult, CompatChecker};

        let Some(node) = self.ctx.arena.get(idx) else {
            return TypeId::ANY;
        };

        let Some(call) = self.ctx.arena.get_call_expr(node) else {
            return TypeId::ANY;
        };

        // Get the type of the callee
        let callee_type = self.get_type_of_node(call.expression);

        // Check if callee is any/error (don't report for those)
        if callee_type == TypeId::ANY || callee_type == TypeId::ERROR {
            return TypeId::ANY;
        }

        // Get arguments list (may be None for calls without arguments)
        let args = call.arguments.as_ref().map(|a| &a.nodes).map(|n| n.as_slice()).unwrap_or(&[]);

        // Prepare for contextual typing of arguments
        let mut arg_types = Vec::with_capacity(args.len());

        // Create contextual context from callee type
        let ctx_helper = ContextualTypeContext::with_expected(self.ctx.types, callee_type);

        let arg_count = args.len();
        for (i, &arg_idx) in args.iter().enumerate() {
            // Determine expected type for this argument
            let expected_type = ctx_helper.get_parameter_type_for_call(i, arg_count);

            // Set contextual type for the argument
            let prev_context = self.ctx.contextual_type;
            self.ctx.contextual_type = expected_type;

            // Check the argument with context
            let arg_type = self.get_type_of_node(arg_idx);
            arg_types.push(arg_type);

            if let Some(expected) = expected_type {
                if expected != TypeId::ANY && expected != TypeId::UNKNOWN {
                    if let Some(arg_node) = self.ctx.arena.get(arg_idx) {
                        if arg_node.kind == syntax_kind_ext::OBJECT_LITERAL_EXPRESSION {
                            self.check_object_literal_excess_properties(arg_type, expected, arg_idx);
                        }
                    }
                }
            }

            // Restore previous context
            self.ctx.contextual_type = prev_context;
        }

        // Use CallEvaluator to resolve the call
        let mut checker = CompatChecker::new(self.ctx.types);
        let mut evaluator = CallEvaluator::new(self.ctx.types, &mut checker);
        let result = evaluator.resolve_call(callee_type, &arg_types);

        match result {
            CallResult::Success(return_type) => return_type,

            CallResult::NotCallable { .. } => {
                self.error_not_callable_at(callee_type, call.expression);
                TypeId::ERROR
            }

            CallResult::ArgumentCountMismatch { expected_min, expected_max, actual } => {
                let expected = expected_max.unwrap_or(expected_min);
                self.error_argument_count_mismatch_at(expected, actual, idx);
                TypeId::ERROR
            }

            CallResult::ArgumentTypeMismatch { index, expected, actual } => {
                // Report error at the specific argument
                if index < args.len() {
                    self.error_argument_not_assignable_at(actual, expected, args[index]);
                }
                TypeId::ERROR
            }

            CallResult::NoOverloadMatch { .. } => {
                // For now, just report a generic error
                // TODO: Enhance with specific overload failure details
                self.error_not_callable_at(callee_type, call.expression);
                TypeId::ERROR
            }
        }
    }

    /// Get type of new expression.
    fn get_type_of_new_expression(&mut self, idx: NodeIndex) -> TypeId {
        use crate::checker::types::diagnostics::diagnostic_codes;
        let Some(node) = self.ctx.arena.get(idx) else {
            return TypeId::ANY;
        };

        let Some(new_expr) = self.ctx.arena.get_call_expr(node) else {
            return TypeId::ANY;
        };

        // Check if trying to instantiate an abstract class
        // The expression is typically an identifier referencing the class
        if let Some(expr_node) = self.ctx.arena.get(new_expr.expression) {
            // If it's a direct identifier (e.g., `new MyClass()`)
            if let Some(ident) = self.ctx.arena.get_identifier(expr_node) {
                let class_name = &ident.escaped_text;

                // Try multiple ways to find the symbol:
                // 1. Check if the identifier node has a direct symbol binding
                // 2. Look up in file_locals
                // 3. Search all symbols by name (handles local scopes like classes inside functions)

                let symbol_opt = self.ctx.binder.get_node_symbol(new_expr.expression)
                    .or_else(|| self.ctx.binder.file_locals.get(class_name))
                    .or_else(|| self.ctx.binder.get_symbols().find_by_name(class_name));

                if let Some(sym_id) = symbol_opt {
                    if let Some(symbol) = self.ctx.binder.get_symbol(sym_id) {
                        // Check if it has the ABSTRACT flag
                        if symbol.flags & symbol_flags::ABSTRACT != 0 {
                            self.error_at_node(
                                idx,
                                "Cannot create an instance of an abstract class.",
                                diagnostic_codes::CANNOT_CREATE_INSTANCE_OF_ABSTRACT_CLASS,
                            );
                            return TypeId::ERROR;
                        }
                    }
                }
            }
        }

        // Get the type of the constructor expression
        let constructor_type = self.get_type_of_node(new_expr.expression);

        // Check if the constructor type contains any abstract classes (for union types)
        // e.g., `new cls()` where `cls: typeof AbstractA | typeof AbstractB`
        if self.type_contains_abstract_class(constructor_type) {
            self.error_at_node(
                idx,
                "Cannot create an instance of an abstract class.",
                diagnostic_codes::CANNOT_CREATE_INSTANCE_OF_ABSTRACT_CLASS,
            );
            return TypeId::ERROR;
        }

        // For now, return any for new expressions
        // TODO: Extract instance type from constructor
        TypeId::ANY
    }

    /// Check if a type contains any abstract class constructors.
    /// This handles union types like `typeof AbstractA | typeof ConcreteB`.
    fn type_contains_abstract_class(&self, type_id: TypeId) -> bool {
        use crate::solver::{TypeKey, SymbolRef};
        use crate::binder::SymbolId;

        let Some(type_key) = self.ctx.types.lookup(type_id) else {
            return false;
        };

        match type_key {
            // TypeQuery is `typeof ClassName` - check if the symbol is abstract
            // Since get_type_from_type_query now uses real SymbolIds, we can directly look up
            TypeKey::TypeQuery(SymbolRef(sym_id)) => {
                if let Some(symbol) = self.ctx.binder.get_symbol(SymbolId(sym_id)) {
                    if symbol.flags & symbol_flags::ABSTRACT != 0 {
                        return true;
                    }
                }
                false
            }
            // Union type - check if ANY constituent is abstract
            TypeKey::Union(members) => {
                members.iter().any(|&member| self.type_contains_abstract_class(member))
            }
            // Intersection type - check if ANY constituent is abstract
            TypeKey::Intersection(members) => {
                members.iter().any(|&member| self.type_contains_abstract_class(member))
            }
            _ => false,
        }
    }

    /// Get type of property access expression.
    fn get_type_of_property_access(&mut self, idx: NodeIndex) -> TypeId {
        use crate::solver::{PropertyAccessEvaluator, PropertyAccessResult};

        let Some(node) = self.ctx.arena.get(idx) else {
            return TypeId::ANY;
        };

        let Some(access) = self.ctx.arena.get_access_expr(node) else {
            return TypeId::ANY;
        };

        // Get the property name first (needed for abstract property check regardless of object type)
        let Some(name_node) = self.ctx.arena.get(access.name_or_argument) else {
            return TypeId::ANY;
        };

        // Check for abstract property access in constructor BEFORE evaluating types (error 2715)
        // This must happen even when `this` has type ANY
        if let Some(ident) = self.ctx.arena.get_identifier(name_node) {
            let property_name = &ident.escaped_text;

            if self.is_this_expression(access.expression) {
                if let Some(ref class_info) = self.ctx.enclosing_class.clone() {
                    if class_info.in_constructor && self.is_abstract_member(&class_info.member_nodes, property_name) {
                        self.error_abstract_property_in_constructor(
                            property_name,
                            &class_info.name,
                            access.name_or_argument,
                        );
                    }
                }
            }
        }

        // Get the type of the object
        let object_type = self.get_type_of_node(access.expression);

        // Don't report errors for any/error types
        if object_type == TypeId::ANY || object_type == TypeId::ERROR {
            return TypeId::ANY;
        }

        // If it's an identifier, look up the property
        if let Some(ident) = self.ctx.arena.get_identifier(name_node) {
            let property_name = &ident.escaped_text;

            // Use PropertyAccessEvaluator to resolve the property access
            let evaluator = PropertyAccessEvaluator::new(self.ctx.types);
            let result = evaluator.resolve_property_access(object_type, property_name);

            match result {
                PropertyAccessResult::Success { type_id: prop_type, from_index_signature } => {
                    // Check for error 4111: property access from index signature
                    if from_index_signature {
                        use crate::checker::types::diagnostics::diagnostic_codes;
                        self.error_at_node(
                            access.name_or_argument,
                            &format!(
                                "Property '{}' comes from an index signature, so it must be accessed with ['{}'].",
                                property_name, property_name
                            ),
                            diagnostic_codes::PROPERTY_ACCESS_FROM_INDEX_SIGNATURE,
                        );
                    }
                    prop_type
                }

                PropertyAccessResult::PropertyNotFound { .. } => {
                    self.error_property_not_exist_at(property_name, object_type, idx);
                    TypeId::ERROR
                }

                PropertyAccessResult::PossiblyNullOrUndefined { property_type, cause } => {
                    // Check for optional chaining (?.)
                    if access.question_dot_token {
                        // Suppress error, return (property_type | undefined)
                        let base_type = property_type.unwrap_or(TypeId::ANY);
                        return self.ctx.types.union(vec![base_type, TypeId::UNDEFINED]);
                    }

                    // Report error based on the cause
                    use crate::checker::types::diagnostics::diagnostic_codes;

                    let (code, message) = if cause == TypeId::NULL {
                        (diagnostic_codes::OBJECT_IS_POSSIBLY_NULL, "Object is possibly 'null'.")
                    } else if cause == TypeId::UNDEFINED {
                        (diagnostic_codes::OBJECT_IS_POSSIBLY_UNDEFINED, "Object is possibly 'undefined'.")
                    } else {
                        (diagnostic_codes::OBJECT_IS_POSSIBLY_NULL_OR_UNDEFINED, "Object is possibly 'null' or 'undefined'.")
                    };

                    // Report the error on the expression part
                    self.error_at_node(access.expression, message, code);

                    // Error recovery: return the property type found in valid members
                    property_type.unwrap_or(TypeId::ERROR)
                }

                PropertyAccessResult::IsUnknown => {
                    // Accessing property on unknown type returns any
                    TypeId::ANY
                }
            }
        } else {
            TypeId::ANY
        }
    }

    /// Get type of element access expression (e.g., arr[0], obj["prop"]).
    fn get_type_of_element_access(&mut self, idx: NodeIndex) -> TypeId {
        let Some(node) = self.ctx.arena.get(idx) else {
            return TypeId::ANY;
        };

        let Some(access) = self.ctx.arena.get_access_expr(node) else {
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
        let Some(node) = self.ctx.arena.get(idx) else {
            return TypeId::ANY;
        };

        let Some(cond) = self.ctx.arena.get_conditional_expr(node) else {
            return TypeId::ANY;
        };

        let when_true = self.get_type_of_node(cond.when_true);
        let when_false = self.get_type_of_node(cond.when_false);

        if when_true == when_false {
            when_true
        } else {
            // Use TypeInterner's union method for automatic normalization
            self.ctx.types.union(vec![when_true, when_false])
        }
    }

    /// Get type of function declaration/expression/arrow.
    fn get_type_of_function(&mut self, idx: NodeIndex) -> TypeId {
        use crate::solver::{FunctionShape, ParamInfo};
        use std::sync::Arc;

        let Some(node) = self.ctx.arena.get(idx) else {
            return TypeId::ANY;
        };

        let Some(func) = self.ctx.arena.get_function(node) else {
            return TypeId::ANY;
        };

        // Collect parameter info using solver's ParamInfo struct
        let mut params = Vec::new();
        let mut param_types: Vec<Option<TypeId>> = Vec::new();
        let mut this_type = None;
        let this_atom = self.ctx.types.intern_string("this");

        // Setup contextual typing context if available
        let ctx_helper = if let Some(ctx_type) = self.ctx.contextual_type {
            Some(ContextualTypeContext::with_expected(self.ctx.types, ctx_type))
        } else {
            None
        };

        let mut contextual_index = 0;
        for &param_idx in func.parameters.nodes.iter() {
            if let Some(param_node) = self.ctx.arena.get(param_idx) {
                if let Some(param) = self.ctx.arena.get_parameter(param_node) {
                    // Get parameter name
                    let name = if let Some(name_node) = self.ctx.arena.get(param.name) {
                        if let Some(name_data) = self.ctx.arena.get_identifier(name_node) {
                            Some(self.ctx.types.intern_string(&name_data.escaped_text))
                        } else {
                            None
                        }
                    } else {
                        None
                    };

                    let is_this_param = name == Some(this_atom);

                    // Use type annotation if present, otherwise infer from context
                    let type_id = if !param.type_annotation.is_none() {
                        // Check parameter type for parameter properties in function types
                        self.check_type_for_parameter_properties(param.type_annotation);
                        self.get_type_from_type_node(param.type_annotation)
                    } else if is_this_param {
                        if let Some(ref helper) = ctx_helper {
                            helper.get_this_type().unwrap_or(TypeId::ANY)
                        } else {
                            TypeId::ANY
                        }
                    } else {
                        // Infer from contextual type
                        if let Some(ref helper) = ctx_helper {
                            helper.get_parameter_type(contextual_index).unwrap_or(TypeId::ANY)
                        } else {
                            TypeId::ANY
                        }
                    };

                    if is_this_param {
                        if this_type.is_none() {
                            this_type = Some(type_id);
                        }
                        param_types.push(None);
                        continue;
                    }

                    // Check if optional or has initializer
                    let optional = param.question_token || !param.initializer.is_none();
                    let rest = param.dot_dot_dot_token;

                    params.push(ParamInfo {
                        name,
                        type_id,
                        optional,
                        rest,
                    });
                    param_types.push(Some(type_id));
                    contextual_index += 1;
                }
            }
        }

        // Check for parameter properties (error 2369)
        // Parameter properties are only allowed in constructors, not in regular functions
        self.check_parameter_properties(&func.parameters.nodes);

        // Get return type from annotation or infer
        let return_type = if !func.type_annotation.is_none() {
            // Check return type for parameter properties in function types
            self.check_type_for_parameter_properties(func.type_annotation);
            self.get_type_from_type_node(func.type_annotation)
        } else {
            // TODO: Infer return type from body
            TypeId::ANY
        };

        // Check the function body (for type errors within the body)
        if !func.body.is_none() {
            self.push_local_scope();

            // Add parameters to local scope using the already-computed types (including contextual)
            for (i, &param_idx) in func.parameters.nodes.iter().enumerate() {
                if let Some(param_node) = self.ctx.arena.get(param_idx) {
                    if let Some(param) = self.ctx.arena.get_parameter(param_node) {
                        if let Some(name_node) = self.ctx.arena.get(param.name) {
                            if let Some(name_data) = self.ctx.arena.get_identifier(name_node) {
                                // Use type from params which already includes contextual typing
                                if let Some(param_type) = param_types.get(i).and_then(|t| *t) {
                                    self.add_local(name_data.escaped_text.clone(), param_type);
                                }
                            }
                        }
                    }
                }
            }

            self.push_return_type(return_type);
            self.check_statement(func.body);
            self.pop_return_type();
            self.pop_local_scope();
        }

        // Create function type using TypeInterner
        let shape = FunctionShape {
            type_params: Vec::new(), // TODO: Handle type parameters
            params,
            this_type,
            return_type,
            type_predicate: None,
            is_constructor: false,
        };

        self.ctx.types.function(shape)
    }

    /// Get type of array literal.
    fn get_type_of_array_literal(&mut self, idx: NodeIndex) -> TypeId {
        let Some(node) = self.ctx.arena.get(idx) else {
            return TypeId::ANY;
        };

        let Some(array) = self.ctx.arena.get_literal_expr(node) else {
            return TypeId::ANY;
        };

        if array.elements.nodes.is_empty() {
            // Empty array literal: infer from context or use never[]
            if let Some(contextual) = self.ctx.contextual_type {
                return contextual;
            }
            return self.ctx.types.array(TypeId::NEVER);
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
            self.ctx.types.union(element_types)
        };

        self.ctx.types.array(element_type)
    }

    /// Get type of object literal.
    fn get_type_of_object_literal(&mut self, idx: NodeIndex) -> TypeId {
        use crate::solver::PropertyInfo;
        use std::sync::Arc;

        let Some(node) = self.ctx.arena.get(idx) else {
            return TypeId::ANY;
        };

        let Some(obj) = self.ctx.arena.get_literal_expr(node) else {
            return TypeId::ANY;
        };

        // Collect properties from the object literal
        let mut properties: Vec<PropertyInfo> = Vec::new();

        // Setup contextual typing context
        let ctx_helper = if let Some(ctx_type) = self.ctx.contextual_type {
            Some(ContextualTypeContext::with_expected(self.ctx.types, ctx_type))
        } else {
            None
        };

        for &elem_idx in &obj.elements.nodes {
            let Some(elem_node) = self.ctx.arena.get(elem_idx) else {
                continue;
            };

            // Property assignment: { x: value }
            if let Some(prop) = self.ctx.arena.get_property_assignment(elem_node) {
                if let Some(name) = self.get_property_name(prop.name) {
                    // Set contextual type for property value
                    let prev_context = self.ctx.contextual_type;
                    if let Some(ref helper) = ctx_helper {
                        self.ctx.contextual_type = helper.get_property_type(&name);
                    }

                    let value_type = self.get_type_of_node(prop.initializer);

                    // Restore context
                    self.ctx.contextual_type = prev_context;

                    properties.push(PropertyInfo {
                        name: self.ctx.types.intern_string(&name),
                        type_id: value_type,
                        optional: false,
                        readonly: false,
                        is_method: false,
                    });
                }
            }
            // Shorthand property: { x } - identifier is both name and value
            else if elem_node.kind == syntax_kind_ext::SHORTHAND_PROPERTY_ASSIGNMENT {
                if let Some(ident) = self.ctx.arena.get_identifier(elem_node) {
                    let value_type = self.get_type_of_node(elem_idx);
                    properties.push(PropertyInfo {
                        name: self.ctx.types.intern_string(&ident.escaped_text),
                        type_id: value_type,
                        optional: false,
                        readonly: false,
                        is_method: false,
                    });
                }
            }
            // Method shorthand: { foo() {} }
            else if let Some(method) = self.ctx.arena.get_method_decl(elem_node) {
                if let Some(name) = self.get_property_name(method.name) {
                    // Set contextual type for method
                    let prev_context = self.ctx.contextual_type;
                    if let Some(ref helper) = ctx_helper {
                        self.ctx.contextual_type = helper.get_property_type(&name);
                    }

                    let method_type = self.get_type_of_function(elem_idx);

                    // Restore context
                    self.ctx.contextual_type = prev_context;

                    properties.push(PropertyInfo {
                        name: self.ctx.types.intern_string(&name),
                        type_id: method_type,
                        optional: false,
                        readonly: false,
                        is_method: false,
                    });
                }
            }
            // Accessor: { get foo() {} } or { set foo(v) {} }
            else if let Some(accessor) = self.ctx.arena.get_accessor(elem_node) {
                // Check for missing body - error 1005 at end of accessor
                if accessor.body.is_none() {
                    use crate::checker::types::diagnostics::diagnostic_codes;
                    // Report at accessor.end - 1 (pointing to the closing paren)
                    let end_pos = elem_node.end.saturating_sub(1);
                    self.error_at_position(
                        end_pos,
                        1,
                        "'{' expected.",
                        diagnostic_codes::TOKEN_EXPECTED,
                    );
                }
                if let Some(name) = self.get_property_name(accessor.name) {
                    // For getter, infer return type; for setter, it's void
                    let accessor_type = if elem_node.kind == syntax_kind_ext::GET_ACCESSOR {
                        self.get_type_of_function(elem_idx)
                    } else {
                        TypeId::VOID
                    };
                    properties.push(PropertyInfo {
                        name: self.ctx.types.intern_string(&name),
                        type_id: accessor_type,
                        optional: false,
                        readonly: false,
                        is_method: false,
                    });
                }
            }
            // Skip spread elements and computed properties for now
        }

        self.ctx.types.object(properties)
    }

    /// Get property name as string from a property name node (identifier, string literal, etc.)
    fn get_property_name(&self, name_idx: NodeIndex) -> Option<String> {
        let name_node = self.ctx.arena.get(name_idx)?;

        // Identifier
        if let Some(ident) = self.ctx.arena.get_identifier(name_node) {
            return Some(ident.escaped_text.clone());
        }

        // For string/numeric literals, we'd need access to the token text
        // For now, just handle identifiers which are the common case
        None
    }

    /// Get type of prefix unary expression.
    fn get_type_of_prefix_unary(&mut self, idx: NodeIndex) -> TypeId {
        let Some(node) = self.ctx.arena.get(idx) else {
            return TypeId::ANY;
        };

        let Some(unary) = self.ctx.arena.get_unary_expr(node) else {
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


    // =========================================================================
    // Type Relations (uses solver::CompatChecker for assignability)
    // =========================================================================

    /// Check if `source` type is assignable to `target` type.
    ///
    /// Uses the solver's SubtypeChecker with coinductive cycle detection.
    /// Note: Does not resolve Ref types (use `is_assignable_to_with_resolution` for that).
    pub fn is_assignable_to(&self, source: TypeId, target: TypeId) -> bool {
        use crate::solver::CompatChecker;
        let mut checker = CompatChecker::new(self.ctx.types);
        checker.is_assignable(source, target)
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
        use crate::solver::CompatChecker;
        let mut checker = CompatChecker::with_resolver(self.ctx.types, env);
        checker.is_assignable(source, target)
    }

    /// Check if `source` type is a subtype of `target` type.
    ///
    /// Stricter than assignability. Uses coinductive semantics for recursive types.
    pub fn is_subtype_of(&self, source: TypeId, target: TypeId) -> bool {
        use crate::solver::SubtypeChecker;
        let mut checker = SubtypeChecker::new(self.ctx.types);
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
        let mut checker = SubtypeChecker::with_resolver(self.ctx.types, env);
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
        use crate::solver::CompatChecker;
        let mut checker = CompatChecker::new(self.ctx.types);
        for &target in targets {
            if checker.is_assignable(source, target) {
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
        let symbols: Vec<SymbolId> = self.ctx.binder.node_symbols
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
        self.ctx.types.union(types)
    }

    /// Create an intersection type from multiple types.
    ///
    /// Automatically normalizes: flattens nested intersections, deduplicates, sorts.
    pub fn get_intersection_type(&self, types: Vec<TypeId>) -> TypeId {
        self.ctx.types.intersection(types)
    }

    // =========================================================================
    // Type Narrowing (uses solver::NarrowingContext)
    // =========================================================================

    /// Narrow a type by a typeof guard.
    ///
    /// Example: `typeof x === "string"` narrows `string | number` to `string`.
    pub fn narrow_by_typeof(&self, source: TypeId, typeof_result: &str) -> TypeId {
        use crate::solver::NarrowingContext;
        let ctx = NarrowingContext::new(self.ctx.types);
        ctx.narrow_by_typeof(source, typeof_result)
    }

    /// Narrow a type by excluding a typeof guard.
    ///
    /// Example: `typeof x !== "string"` narrows `string | number` to `number`.
    pub fn narrow_by_typeof_negation(&self, source: TypeId, typeof_result: &str) -> TypeId {
        use crate::solver::NarrowingContext;
        let ctx = NarrowingContext::new(self.ctx.types);

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
    pub fn narrow_by_discriminant(&self, union_type: TypeId, property_name: Atom, literal_value: TypeId) -> TypeId {
        use crate::solver::NarrowingContext;
        let ctx = NarrowingContext::new(self.ctx.types);
        ctx.narrow_by_discriminant(union_type, property_name, literal_value)
    }

    /// Narrow a discriminated union by excluding a discriminant value.
    ///
    /// Example: `action.type !== "add"` narrows the union to exclude the "add" variant.
    pub fn narrow_by_excluding_discriminant(&self, union_type: TypeId, property_name: Atom, excluded_value: TypeId) -> TypeId {
        use crate::solver::NarrowingContext;
        let ctx = NarrowingContext::new(self.ctx.types);
        ctx.narrow_by_excluding_discriminant(union_type, property_name, excluded_value)
    }
    /// Find discriminant properties in a union type.
    ///
    /// Returns information about properties that uniquely identify each union variant.
    pub fn find_discriminants(&self, union_type: TypeId) -> Vec<crate::solver::DiscriminantInfo> {
        use crate::solver::NarrowingContext;
        let ctx = NarrowingContext::new(self.ctx.types);
        ctx.find_discriminants(union_type)
    }

    /// Narrow a type to include only members assignable to target.
    pub fn narrow_to_type(&self, source: TypeId, target: TypeId) -> TypeId {
        use crate::solver::NarrowingContext;
        let ctx = NarrowingContext::new(self.ctx.types);
        ctx.narrow_to_type(source, target)
    }

    /// Narrow a type to exclude members assignable to target.
    pub fn narrow_excluding_type(&self, source: TypeId, excluded: TypeId) -> TypeId {
        use crate::solver::NarrowingContext;
        let ctx = NarrowingContext::new(self.ctx.types);
        ctx.narrow_excluding_type(source, excluded)
    }

    // =========================================================================
    // Type Node Resolution
    // =========================================================================

    /// Get type from a type node.
    ///
    /// Uses compile-time constant TypeIds for intrinsic types (O(1) lookup).
    /// Delegates to TypeLowering for complex types (union, intersection, array, etc.).
    /// Also validates that type references exist and reports errors.
    pub fn get_type_from_type_node(&mut self, idx: NodeIndex) -> TypeId {
        use crate::solver::TypeLowering;

        // First check if this is a type that needs special handling with binder resolution
        if let Some(node) = self.ctx.arena.get(idx) {
            if node.kind == syntax_kind_ext::TYPE_REFERENCE {
                // Validate the type reference exists before lowering
                return self.get_type_from_type_reference(idx);
            }
            if node.kind == syntax_kind_ext::TYPE_QUERY {
                // Handle typeof X - need to resolve symbol properly via binder
                return self.get_type_from_type_query(idx);
            }
            if node.kind == syntax_kind_ext::UNION_TYPE {
                // Handle union types specially to ensure nested typeof expressions
                // are resolved via binder (for abstract class detection)
                return self.get_type_from_union_type(idx);
            }
        }

        // Use TypeLowering which handles all type nodes
        let type_resolver = |node_idx: NodeIndex| self.resolve_type_symbol_for_lowering(node_idx);
        let value_resolver = |node_idx: NodeIndex| self.resolve_value_symbol_for_lowering(node_idx);
        let lowering = TypeLowering::with_resolvers(
            self.ctx.arena,
            self.ctx.types,
            &type_resolver,
            &value_resolver,
        );
        lowering.lower_type(idx)
    }

    // =========================================================================
    // Source Location Tracking & Solver Diagnostics
    // =========================================================================

    /// Get a source location for a node.
    pub fn get_source_location(&self, idx: NodeIndex) -> Option<crate::solver::SourceLocation> {
        let node = self.ctx.arena.get(idx)?;
        Some(crate::solver::SourceLocation::new(
            self.ctx.file_name.as_str(),
            node.pos,
            node.end,
        ))
    }

    /// Report a type not assignable error using solver diagnostics with source tracking.
    ///
    /// This is the basic error that just says "Type X is not assignable to Y".
    /// For detailed errors with elaboration (e.g., "property 'x' is missing"),
    /// use `error_type_not_assignable_with_reason_at` instead.
    pub fn error_type_not_assignable_at(
        &mut self,
        source: TypeId,
        target: TypeId,
        idx: NodeIndex,
    ) {
        if let Some(loc) = self.get_source_location(idx) {
            let mut builder = crate::solver::SpannedDiagnosticBuilder::new(
                self.ctx.types,
                self.ctx.file_name.as_str(),
            );
            let diag = builder.type_not_assignable(source, target, loc.start, loc.length());
            self.ctx.diagnostics.push(diag.to_checker_diagnostic(&self.ctx.file_name));
        }
    }

    /// Report a type not assignable error with detailed elaboration.
    ///
    /// This method uses the solver's "explain" API to determine WHY the types
    /// are incompatible (e.g., missing property, incompatible property types,
    /// etc.) and produces a richer diagnostic with that information.
    ///
    /// **Architecture Note**: This follows the "Check Fast, Explain Slow" pattern.
    /// The `is_assignable_to` check is fast (boolean). This explain call is slower
    /// but produces better error messages. Only call this after a failed check.
    pub fn error_type_not_assignable_with_reason_at(
        &mut self,
        source: TypeId,
        target: TypeId,
        idx: NodeIndex,
    ) {
        use crate::solver::{CompatChecker, TypeFormatter};

        let Some(loc) = self.get_source_location(idx) else {
            return;
        };

        // Use the solver's explain API to get the detailed reason
        let mut checker = CompatChecker::new(self.ctx.types);
        let reason = checker.explain_failure(source, target);

        match reason {
            Some(failure_reason) => {
                // Convert the reason to a PendingDiagnostic with elaboration
                let pending = failure_reason.to_diagnostic(source, target)
                    .with_span(crate::solver::SourceSpan::new(
                        self.ctx.file_name.as_str(),
                        loc.start,
                        loc.length(),
                    ));

                // Render the pending diagnostic to a TypeDiagnostic
                let mut formatter = TypeFormatter::new(self.ctx.types);
                let type_diag = formatter.render(&pending);

                // Convert to checker diagnostic and add
                self.ctx.diagnostics.push(type_diag.to_checker_diagnostic(&self.ctx.file_name));
            }
            None => {
                // Fallback: shouldn't happen if called after a failed check,
                // but just use the basic error message
                self.error_type_not_assignable_at(source, target, idx);
            }
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
                self.ctx.types,
                self.ctx.file_name.as_str(),
            );
            let diag = builder.property_missing(prop_name, source, target, loc.start, loc.length());
            self.ctx.diagnostics.push(diag.to_checker_diagnostic(&self.ctx.file_name));
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
                self.ctx.types,
                self.ctx.file_name.as_str(),
            );
            let diag = builder.property_not_exist(prop_name, type_id, loc.start, loc.length());
            self.ctx.diagnostics.push(diag.to_checker_diagnostic(&self.ctx.file_name));
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
                self.ctx.types,
                self.ctx.file_name.as_str(),
            );
            let diag = builder.argument_not_assignable(arg_type, param_type, loc.start, loc.length());
            self.ctx.diagnostics.push(diag.to_checker_diagnostic(&self.ctx.file_name));
        }
    }

    /// Report a cannot find name error using solver diagnostics with source tracking.
    pub fn error_cannot_find_name_at(&mut self, name: &str, idx: NodeIndex) {
        if let Some(loc) = self.get_source_location(idx) {
            let mut builder = crate::solver::SpannedDiagnosticBuilder::new(
                self.ctx.types,
                self.ctx.file_name.as_str(),
            );
            let diag = builder.cannot_find_name(name, loc.start, loc.length());
            self.ctx.diagnostics.push(diag.to_checker_diagnostic(&self.ctx.file_name));
        }
    }

    /// Report error 2662: Cannot find name 'X'. Did you mean the static member 'C.X'?
    pub fn error_cannot_find_name_static_member_at(
        &mut self,
        name: &str,
        class_name: &str,
        idx: NodeIndex,
    ) {
        use crate::checker::types::diagnostics::diagnostic_codes;

        if let Some(loc) = self.get_source_location(idx) {
            let message = format!(
                "Cannot find name '{}'. Did you mean the static member '{}.{}'?",
                name, class_name, name
            );
            self.ctx.diagnostics.push(Diagnostic {
                code: diagnostic_codes::CANNOT_FIND_NAME_DID_YOU_MEAN_STATIC,
                category: DiagnosticCategory::Error,
                message_text: message,
                file: self.ctx.file_name.clone(),
                start: loc.start,
                length: loc.length(),
                related_information: Vec::new(),
            });
        }
    }

    /// Report error 2403: Subsequent variable declarations must have the same type.
    pub fn error_subsequent_variable_declaration(
        &mut self,
        name: &str,
        prev_type: TypeId,
        current_type: TypeId,
        idx: NodeIndex,
    ) {
        use crate::checker::types::diagnostics::diagnostic_codes;

        if let Some(loc) = self.get_source_location(idx) {
            let prev_type_str = self.format_type(prev_type);
            let current_type_str = self.format_type(current_type);
            let message = format!(
                "Subsequent variable declarations must have the same type. Variable '{}' must be of type '{}', but here has type '{}'.",
                name, prev_type_str, current_type_str
            );
            self.ctx.diagnostics.push(Diagnostic {
                code: diagnostic_codes::SUBSEQUENT_VARIABLE_DECLARATIONS_MUST_HAVE_SAME_TYPE,
                category: DiagnosticCategory::Error,
                message_text: message,
                file: self.ctx.file_name.clone(),
                start: loc.start,
                length: loc.length(),
                related_information: Vec::new(),
            });
        }
    }

    /// Report error 2715: Abstract property 'X' in class 'C' cannot be accessed in the constructor.
    pub fn error_abstract_property_in_constructor(
        &mut self,
        prop_name: &str,
        class_name: &str,
        idx: NodeIndex,
    ) {
        use crate::checker::types::diagnostics::diagnostic_codes;

        if let Some(loc) = self.get_source_location(idx) {
            let message = format!(
                "Abstract property '{}' in class '{}' cannot be accessed in the constructor.",
                prop_name, class_name
            );
            self.ctx.diagnostics.push(Diagnostic {
                code: diagnostic_codes::ABSTRACT_PROPERTY_IN_CONSTRUCTOR,
                category: DiagnosticCategory::Error,
                message_text: message,
                file: self.ctx.file_name.clone(),
                start: loc.start,
                length: loc.length(),
                related_information: Vec::new(),
            });
        }
    }

    /// Check if a node is a `this` expression.
    fn is_this_expression(&self, idx: NodeIndex) -> bool {
        use crate::scanner::SyntaxKind;
        if let Some(node) = self.ctx.arena.get(idx) {
            node.kind == SyntaxKind::ThisKeyword as u16
        } else {
            false
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
                self.ctx.types,
                self.ctx.file_name.as_str(),
            );
            let diag = builder.argument_count_mismatch(expected, got, loc.start, loc.length());
            self.ctx.diagnostics.push(diag.to_checker_diagnostic(&self.ctx.file_name));
        }
    }

    /// Report a "type is not callable" error using solver diagnostics with source tracking.
    pub fn error_not_callable_at(&mut self, type_id: TypeId, idx: NodeIndex) {
        if let Some(loc) = self.get_source_location(idx) {
            let mut builder = crate::solver::SpannedDiagnosticBuilder::new(
                self.ctx.types,
                self.ctx.file_name.as_str(),
            );
            let diag = builder.not_callable(type_id, loc.start, loc.length());
            self.ctx.diagnostics.push(diag.to_checker_diagnostic(&self.ctx.file_name));
        }
    }

    /// Report an excess property error using solver diagnostics with source tracking.
    pub fn error_excess_property_at(&mut self, prop_name: &str, target: TypeId, idx: NodeIndex) {
        if let Some(loc) = self.get_source_location(idx) {
            let mut builder = crate::solver::SpannedDiagnosticBuilder::new(
                self.ctx.types,
                self.ctx.file_name.as_str(),
            );
            let diag = builder.excess_property(prop_name, target, loc.start, loc.length());
            self.ctx.diagnostics.push(diag.to_checker_diagnostic(&self.ctx.file_name));
        }
    }

    /// Report a "Cannot assign to readonly property" error using solver diagnostics with source tracking.
    pub fn error_readonly_property_at(&mut self, prop_name: &str, idx: NodeIndex) {
        if let Some(loc) = self.get_source_location(idx) {
            let mut builder = crate::solver::SpannedDiagnosticBuilder::new(
                self.ctx.types,
                self.ctx.file_name.as_str(),
            );
            let diag = builder.readonly_property(prop_name, loc.start, loc.length());
            self.ctx.diagnostics.push(diag.to_checker_diagnostic(&self.ctx.file_name));
        }
    }

    /// Report TS2694: Namespace has no exported member.
    pub fn error_namespace_no_export(&mut self, namespace_name: &str, member_name: &str, idx: NodeIndex) {
        if let Some(loc) = self.get_source_location(idx) {
            let message = format!("Namespace '{}' has no exported member '{}'.", namespace_name, member_name);
            self.ctx.diagnostics.push(Diagnostic {
                code: 2694,
                category: DiagnosticCategory::Error,
                message_text: message,
                start: loc.start,
                length: loc.length(),
                file: self.ctx.file_name.clone(),
                related_information: Vec::new(),
            });
        }
    }

    /// Create a diagnostic collector for batch error reporting.
    pub fn create_diagnostic_collector(&self) -> crate::solver::DiagnosticCollector<'_> {
        crate::solver::DiagnosticCollector::new(self.ctx.types, self.ctx.file_name.as_str())
    }

    /// Merge diagnostics from a collector into the checker's diagnostics.
    pub fn merge_diagnostics(&mut self, collector: &crate::solver::DiagnosticCollector) {
        for diag in collector.to_checker_diagnostics() {
            self.ctx.diagnostics.push(diag);
        }
    }

    /// Format a type as a human-readable string using solver's TypeFormatter.
    pub fn format_type(&self, type_id: TypeId) -> String {
        let mut formatter = crate::solver::TypeFormatter::new(self.ctx.types);
        formatter.format(type_id)
    }

    // =========================================================================
    // Source File Checking (Full Traversal)
    // =========================================================================

    /// Check a source file and populate diagnostics.
    /// This is the entry point for type checking a parsed and bound file.
    pub fn check_source_file(&mut self, root_idx: NodeIndex) {
        let Some(node) = self.ctx.arena.get(root_idx) else {
            return;
        };

        if let Some(sf) = self.ctx.arena.get_source_file(node) {
            // Push file-level scope for top-level variables
            // This enables variable redeclaration checking (TS2403) and type tracking
            self.push_local_scope();

            // Type check each top-level statement
            for &stmt_idx in &sf.statements.nodes {
                self.check_statement(stmt_idx);
            }

            // Check for function overload implementations (2389, 2391)
            self.check_function_implementations(&sf.statements.nodes);

            // Check for export assignment with other exports (2309)
            self.check_export_assignment(&sf.statements.nodes);

            // Pop file-level scope
            self.pop_local_scope();
        }
    }

    /// Check a statement and produce type errors.
    fn check_statement(&mut self, stmt_idx: NodeIndex) {
        let Some(node) = self.ctx.arena.get(stmt_idx) else {
            return;
        };

        match node.kind {
            syntax_kind_ext::VARIABLE_STATEMENT => {
                self.check_variable_statement(stmt_idx);
            }
            syntax_kind_ext::EXPRESSION_STATEMENT => {
                // ExpressionStatement stores expression index in data_index
                if let Some(expr_stmt) = self.ctx.arena.get_expression_statement(node) {
                    self.get_type_of_node(expr_stmt.expression);
                }
            }
            syntax_kind_ext::IF_STATEMENT => {
                if let Some(if_data) = self.ctx.arena.get_if_statement(node) {
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
                self.check_return_statement(stmt_idx);
            }
            syntax_kind_ext::BLOCK => {
                if let Some(block) = self.ctx.arena.get_block(node) {
                    // Push a new scope for block-scoped variables (let/const)
                    self.push_local_scope();
                    for &inner_stmt in &block.statements.nodes {
                        self.check_statement(inner_stmt);
                    }
                    // Check for function overload implementations in blocks
                    self.check_function_implementations(&block.statements.nodes);
                    self.pop_local_scope();
                }
            }
            syntax_kind_ext::FUNCTION_DECLARATION => {
                if let Some(func) = self.ctx.arena.get_function(node) {
                    // Check for parameter properties (error 2369)
                    // Parameter properties are only allowed in constructors
                    self.check_parameter_properties(&func.parameters.nodes);

                    // Check return type annotation for parameter properties in function types
                    if !func.type_annotation.is_none() {
                        self.check_type_for_parameter_properties(func.type_annotation);
                    }

                    // Check parameter type annotations for parameter properties
                    for &param_idx in &func.parameters.nodes {
                        if let Some(param_node) = self.ctx.arena.get(param_idx) {
                            if let Some(param) = self.ctx.arena.get_parameter(param_node) {
                                if !param.type_annotation.is_none() {
                                    self.check_type_for_parameter_properties(param.type_annotation);
                                }
                            }
                        }
                    }

                    // Check function body if present
                    if !func.body.is_none() {
                        self.push_local_scope();

                        // Get declared return type
                        let return_type = if !func.type_annotation.is_none() {
                            self.get_type_of_node(func.type_annotation)
                        } else {
                            TypeId::ANY
                        };
                        self.push_return_type(return_type);

                        // Add parameters to local scope
                        for &param_idx in &func.parameters.nodes {
                            if let Some(param_node) = self.ctx.arena.get(param_idx) {
                                if let Some(param) = self.ctx.arena.get_parameter(param_node) {
                                    // Get parameter name
                                    if let Some(name_node) = self.ctx.arena.get(param.name) {
                                        if let Some(ident) = self.ctx.arena.get_identifier(name_node) {
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

                        // Check for error 2355: function with return type must return a value
                        // Only check if there's an explicit return type annotation
                        let has_type_annotation = !func.type_annotation.is_none();
                        let requires_return = self.requires_return_value(return_type);
                        let has_return = self.body_has_return_with_value(func.body);

                        if has_type_annotation && requires_return && !has_return {
                            use crate::checker::types::diagnostics::diagnostic_codes;
                            self.error_at_node(
                                func.type_annotation,
                                "A function whose declared type is neither 'undefined', 'void', nor 'any' must return a value.",
                                diagnostic_codes::FUNCTION_LACKS_RETURN_TYPE,
                            );
                        }

                        self.pop_return_type();
                        self.pop_local_scope();
                    }
                }
            }
            syntax_kind_ext::WHILE_STATEMENT | syntax_kind_ext::DO_STATEMENT => {
                if let Some(loop_data) = self.ctx.arena.get_loop(node) {
                    self.get_type_of_node(loop_data.condition);
                    self.check_statement(loop_data.statement);
                }
            }
            syntax_kind_ext::FOR_STATEMENT => {
                if let Some(loop_data) = self.ctx.arena.get_loop(node) {
                    // Push scope for loop variables (e.g., for (let i = 0; ...))
                    self.push_local_scope();
                    if !loop_data.initializer.is_none() {
                        // Check if initializer is a variable declaration list
                        if let Some(init_node) = self.ctx.arena.get(loop_data.initializer) {
                            if init_node.kind == syntax_kind_ext::VARIABLE_DECLARATION_LIST {
                                self.check_variable_declaration_list(loop_data.initializer);
                            } else {
                                self.get_type_of_node(loop_data.initializer);
                            }
                        }
                    }
                    if !loop_data.condition.is_none() {
                        self.get_type_of_node(loop_data.condition);
                    }
                    if !loop_data.incrementor.is_none() {
                        self.get_type_of_node(loop_data.incrementor);
                    }
                    self.check_statement(loop_data.statement);
                    self.pop_local_scope();
                }
            }
            syntax_kind_ext::FOR_IN_STATEMENT | syntax_kind_ext::FOR_OF_STATEMENT => {
                if let Some(for_data) = self.ctx.arena.get_for_in_of(node) {
                    // Push scope for loop variable
                    self.push_local_scope();
                    // Check if initializer is a variable declaration
                    if let Some(init_node) = self.ctx.arena.get(for_data.initializer) {
                        if init_node.kind == syntax_kind_ext::VARIABLE_DECLARATION_LIST {
                            self.check_variable_declaration_list(for_data.initializer);
                        } else {
                            self.get_type_of_node(for_data.initializer);
                        }
                    }
                    self.get_type_of_node(for_data.expression);
                    self.check_statement(for_data.statement);
                    self.pop_local_scope();
                }
            }
            syntax_kind_ext::TRY_STATEMENT => {
                if let Some(try_data) = self.ctx.arena.get_try(node) {
                    self.check_statement(try_data.try_block);
                    if !try_data.catch_clause.is_none() {
                        if let Some(catch_node) = self.ctx.arena.get(try_data.catch_clause) {
                            if let Some(catch) = self.ctx.arena.get_catch_clause(catch_node) {
                                self.check_statement(catch.block);
                            }
                        }
                    }
                    if !try_data.finally_block.is_none() {
                        self.check_statement(try_data.finally_block);
                    }
                }
            }
            // Interface declarations need parameter property checks
            syntax_kind_ext::INTERFACE_DECLARATION => {
                self.check_interface_declaration(stmt_idx);
            }
            // Export declarations - descend into the wrapped declaration
            syntax_kind_ext::EXPORT_DECLARATION => {
                if let Some(export_decl) = self.ctx.arena.get_export_decl(node) {
                    // Check the wrapped declaration (function, class, variable, etc.)
                    if !export_decl.export_clause.is_none() {
                        self.check_statement(export_decl.export_clause);
                    }
                }
            }
            // Type alias declarations - check the type for accessor body and parameter property errors
            syntax_kind_ext::TYPE_ALIAS_DECLARATION => {
                if let Some(type_alias) = self.ctx.arena.get_type_alias(node) {
                    // Check the type for accessor bodies in ambient context and parameter properties
                    self.check_type_for_parameter_properties(type_alias.type_node);
                }
            }
            // Other type declarations - just register them, no expression checking needed
            syntax_kind_ext::ENUM_DECLARATION |
            syntax_kind_ext::IMPORT_DECLARATION |
            syntax_kind_ext::EMPTY_STATEMENT |
            syntax_kind_ext::DEBUGGER_STATEMENT |
            syntax_kind_ext::BREAK_STATEMENT |
            syntax_kind_ext::CONTINUE_STATEMENT => {
                // No action needed
            }
            syntax_kind_ext::MODULE_DECLARATION => {
                // Check module declaration (errors 5061, 2819, etc.)
                let mut checker = crate::checker::declarations::DeclarationChecker::new(&mut self.ctx);
                checker.check_module_declaration(stmt_idx);

                // Check module body for function overload implementations
                if let Some(module) = self.ctx.arena.get_module(node) {
                    if !module.body.is_none() {
                        self.check_module_body(module.body);
                    }
                }
            }
            syntax_kind_ext::CLASS_DECLARATION => {
                self.check_class_declaration(stmt_idx);
            }
            _ => {
                // Catch-all for other statement types
                self.get_type_of_node(stmt_idx);
            }
        }
    }

    /// Check a variable statement (var/let/const declarations).
    fn check_variable_statement(&mut self, stmt_idx: NodeIndex) {
        let Some(node) = self.ctx.arena.get(stmt_idx) else {
            return;
        };

        if let Some(var) = self.ctx.arena.get_variable(node) {
            // VariableStatement.declarations contains VariableDeclarationList nodes
            for &list_idx in &var.declarations.nodes {
                self.check_variable_declaration_list(list_idx);
            }
        }
    }

    /// Check a variable declaration list (var/let/const x, y, z).
    fn check_variable_declaration_list(&mut self, list_idx: NodeIndex) {
        let Some(node) = self.ctx.arena.get(list_idx) else {
            return;
        };

        // VariableDeclarationList uses the same VariableData structure
        if let Some(var_list) = self.ctx.arena.get_variable(node) {
            // Now these are actual VariableDeclaration nodes
            for &decl_idx in &var_list.declarations.nodes {
                self.check_variable_declaration(decl_idx);
            }
        }
    }

    /// Check a single variable declaration.
    fn check_variable_declaration(&mut self, decl_idx: NodeIndex) {
        let Some(node) = self.ctx.arena.get(decl_idx) else {
            return;
        };

        let Some(var_decl) = self.ctx.arena.get_variable_declaration(node) else {
            return;
        };

        // Get the variable name for adding to local scope
        let var_name = if let Some(name_node) = self.ctx.arena.get(var_decl.name) {
            if let Some(ident) = self.ctx.arena.get_identifier(name_node) {
                Some(ident.escaped_text.clone())
            } else {
                None
            }
        } else {
            None
        };

        // Get declared type from type annotation
        let declared_type = if !var_decl.type_annotation.is_none() {
            self.get_type_from_type_node(var_decl.type_annotation)
        } else {
            TypeId::ANY
        };

        // Determine final type (declared or inferred from initializer)
        let final_type = if !var_decl.initializer.is_none() {
            let prev_context = self.ctx.contextual_type;
            if declared_type != TypeId::ANY {
                self.ctx.contextual_type = Some(declared_type);
            }
            let init_type = self.get_type_of_node(var_decl.initializer);
            self.ctx.contextual_type = prev_context;

            // If there's a type annotation, check that initializer is assignable
            if !var_decl.type_annotation.is_none() && declared_type != TypeId::ANY {
                if !self.is_assignable_to(init_type, declared_type) {
                    // Report type error with elaboration (e.g., "property 'x' is missing")
                    self.error_type_not_assignable_with_reason_at(init_type, declared_type, var_decl.initializer);
                }

                // For object literals, also check for excess properties
                // (missing properties are already handled by error_type_not_assignable_with_reason_at)
                if let Some(init_node) = self.ctx.arena.get(var_decl.initializer) {
                    if init_node.kind == syntax_kind_ext::OBJECT_LITERAL_EXPRESSION {
                        self.check_object_literal_excess_properties(init_type, declared_type, var_decl.initializer);
                    }
                }
                declared_type
            } else {
                // No type annotation - use inferred type from initializer
                init_type
            }
        } else {
            declared_type
        };

        // Check for variable redeclaration in the current scope (TS2403)
        // Note: This applies specifically to 'var' merging where types must match.
        // let/const duplicates are caught earlier by the binder (TS2451).
        if let Some(ref name) = var_name {
            if let Some(prev_type) = self.ctx.lookup_local_in_current_scope(name) {
                // Types must be identical for subsequent declarations
                if !self.are_types_identical(final_type, prev_type) {
                    self.error_subsequent_variable_declaration(name, prev_type, final_type, decl_idx);
                }
            }
        }

        // Add variable to local scope (if we're inside a function/method)
        if let Some(name) = var_name {
            self.add_local(name, final_type);
        }
    }

    /// Check object literal assignment for excess properties.
    ///
    /// **Note**: This check is specific to object literals and is NOT part of general
    /// structural subtyping. Excess properties in object literals are errors, but
    /// when assigning from a variable with extra properties, it's allowed.
    /// See https://github.com/microsoft/TypeScript/issues/13813,
    /// https://github.com/microsoft/TypeScript/issues/18075,
    /// https://github.com/microsoft/TypeScript/issues/28616.
    ///
    /// Missing property errors are handled by the solver's `explain_failure` API
    /// via `error_type_not_assignable_with_reason_at`, so we only check excess
    /// properties here to avoid duplication.
    fn check_object_literal_excess_properties(&mut self, source: TypeId, target: TypeId, idx: NodeIndex) {
        use crate::solver::TypeKey;

        // Get the properties of both types
        let source_props = match self.ctx.types.lookup(source) {
            Some(TypeKey::Object(props)) => props,
            _ => return,
        };

        let target_props = match self.ctx.types.lookup(target) {
            Some(TypeKey::Object(props)) => props,
            _ => return,
        };

        // Check for excess properties in source that don't exist in target
        // This is the "freshness" or "strict object literal" check
        for source_prop in &source_props {
            let exists_in_target = target_props.iter().any(|p| p.name == source_prop.name);
            if !exists_in_target {
                let prop_name = self.ctx.types.resolve_atom(source_prop.name);
                self.error_excess_property_at(&prop_name, target, idx);
            }
        }
        // Note: Missing property checks are handled by solver's explain_failure
    }

    /// Check if an assignment target is a readonly property.
    /// Reports error TS2540 if trying to assign to a readonly property.
    fn check_readonly_assignment(&mut self, target_idx: NodeIndex, expr_idx: NodeIndex) {
        let Some(target_node) = self.ctx.arena.get(target_idx) else {
            return;
        };

        // Only check property access expressions
        if target_node.kind != syntax_kind_ext::PROPERTY_ACCESS_EXPRESSION {
            return;
        }

        let Some(access) = self.ctx.arena.get_access_expr(target_node) else {
            return;
        };

        // Get the property name
        let Some(name_node) = self.ctx.arena.get(access.name_or_argument) else {
            return;
        };

        let Some(ident) = self.ctx.arena.get_identifier(name_node) else {
            return;
        };

        let prop_name = ident.escaped_text.clone();

        // Get the type of the object being accessed
        let obj_type = self.get_type_of_node(access.expression);

        // Check if the property is readonly in the object type (solver types)
        if self.is_property_readonly(obj_type, &prop_name) {
            self.error_readonly_property_at(&prop_name, target_idx);
            return;
        }

        // Also check AST-level readonly on class properties
        // Get the class name from the object expression (for `c.ro`, get the type of `c`)
        if let Some(class_name) = self.get_class_name_from_expression(access.expression) {
            if self.is_class_property_readonly(&class_name, &prop_name) {
                self.error_readonly_property_at(&prop_name, target_idx);
            }
        }
    }

    /// Get the class name from an expression, if it's a class instance.
    fn get_class_name_from_expression(&self, expr_idx: NodeIndex) -> Option<String> {
        let Some(node) = self.ctx.arena.get(expr_idx) else {
            return None;
        };

        // If it's a simple identifier, look up its type from the binder
        if let Some(ident) = self.ctx.arena.get_identifier(node) {
            let var_name = &ident.escaped_text;

            // Look up the variable in local scopes
            if let Some(type_id) = self.lookup_local(var_name) {
                // Check if this type is a class instance - the type would be stored
                // We need to trace back to the class name
                if let Some(class_name) = self.get_class_name_from_type(type_id) {
                    return Some(class_name);
                }
                // If get_class_name_from_type returns None, fall through to check file_locals
            }

            // Check file_locals for the variable binding
            if let Some(sym_id) = self.ctx.binder.file_locals.get(var_name) {
                if let Some(symbol) = self.ctx.binder.get_symbol(sym_id) {
                    // Get the value declaration and check if it's a variable with new Class()
                    if !symbol.value_declaration.is_none() {
                        return self.get_class_name_from_var_decl(symbol.value_declaration);
                    }
                }
            }
        }

        None
    }

    /// Get the class name from a variable declaration that initializes to `new ClassName()`.
    fn get_class_name_from_var_decl(&self, decl_idx: NodeIndex) -> Option<String> {
        let Some(node) = self.ctx.arena.get(decl_idx) else {
            return None;
        };

        let Some(var_decl) = self.ctx.arena.get_variable_declaration(node) else {
            return None;
        };

        if var_decl.initializer.is_none() {
            return None;
        }

        let Some(init_node) = self.ctx.arena.get(var_decl.initializer) else {
            return None;
        };

        // Check if initializer is `new ClassName()`
        if init_node.kind != syntax_kind_ext::NEW_EXPRESSION {
            return None;
        }

        // Call and new expressions share CallExprData
        let Some(new_expr) = self.ctx.arena.get_call_expr(init_node) else {
            return None;
        };

        // Get the class name from the new expression
        let Some(expr_node) = self.ctx.arena.get(new_expr.expression) else {
            return None;
        };

        if let Some(ident) = self.ctx.arena.get_identifier(expr_node) {
            return Some(ident.escaped_text.clone());
        }

        None
    }

    /// Get the class name from a TypeId if it represents a class instance.
    fn get_class_name_from_type(&self, _type_id: TypeId) -> Option<String> {
        // For now, we don't have class types in the solver, so return None
        // This will be implemented when we add proper class types to the solver
        None
    }

    /// Check if a property is readonly in a class declaration (by looking at AST).
    fn is_class_property_readonly(&self, class_name: &str, prop_name: &str) -> bool {
        use crate::scanner::SyntaxKind;

        // Find the class declaration by name
        if let Some(sym_id) = self.ctx.binder.file_locals.get(class_name) {
            if let Some(symbol) = self.ctx.binder.get_symbol(sym_id) {
                let decl_idx = if !symbol.value_declaration.is_none() {
                    symbol.value_declaration
                } else if let Some(&idx) = symbol.declarations.first() {
                    idx
                } else {
                    return false;
                };

                let Some(node) = self.ctx.arena.get(decl_idx) else {
                    return false;
                };

                let Some(class) = self.ctx.arena.get_class(node) else {
                    return false;
                };

                // Find the property in the class members
                for &member_idx in &class.members.nodes {
                    let Some(member_node) = self.ctx.arena.get(member_idx) else {
                        continue;
                    };

                    if member_node.kind == syntax_kind_ext::PROPERTY_DECLARATION {
                        if let Some(prop) = self.ctx.arena.get_property_decl(member_node) {
                            // Get the property name
                            if let Some(pname) = self.get_property_name(prop.name) {
                                if pname == prop_name {
                                    // Check if this property has readonly modifier
                                    return self.has_readonly_modifier(&prop.modifiers);
                                }
                            }
                        }
                    }
                }
            }
        }

        false
    }

    /// Check if modifiers include the 'readonly' keyword.
    fn has_readonly_modifier(&self, modifiers: &Option<crate::parser::NodeList>) -> bool {
        use crate::scanner::SyntaxKind;
        if let Some(mods) = modifiers {
            for &mod_idx in &mods.nodes {
                if let Some(mod_node) = self.ctx.arena.get(mod_idx) {
                    if mod_node.kind == SyntaxKind::ReadonlyKeyword as u16 {
                        return true;
                    }
                }
            }
        }
        false
    }

    /// Check if a property is marked readonly in a type.
    fn is_property_readonly(&self, type_id: TypeId, prop_name: &str) -> bool {
        use crate::solver::TypeKey;

        match self.ctx.types.lookup(type_id) {
            Some(TypeKey::Object(props)) => {
                for prop in props.iter() {
                    if self.ctx.types.resolve_atom(prop.name) == prop_name {
                        return prop.readonly;
                    }
                }
                false
            }
            Some(TypeKey::ObjectWithIndex(shape)) => {
                for prop in shape.properties.iter() {
                    if self.ctx.types.resolve_atom(prop.name) == prop_name {
                        return prop.readonly;
                    }
                }
                // Check index signatures for readonly
                if let Some(ref idx) = shape.string_index {
                    if idx.readonly {
                        return true;
                    }
                }
                if let Some(ref idx) = shape.number_index {
                    if idx.readonly {
                        return true;
                    }
                }
                false
            }
            Some(TypeKey::Union(types)) => {
                // Property is readonly if readonly in all union members
                types.iter().all(|t| self.is_property_readonly(*t, prop_name))
            }
            Some(TypeKey::Intersection(types)) => {
                // Property is readonly if readonly in any intersection member
                types.iter().any(|t| self.is_property_readonly(*t, prop_name))
            }
            _ => false,
        }
    }

    /// Check a return statement.
    fn check_return_statement(&mut self, stmt_idx: NodeIndex) {
        let Some(node) = self.ctx.arena.get(stmt_idx) else {
            return;
        };

        let Some(return_data) = self.ctx.arena.get_return_statement(node) else {
            return;
        };

        // Get the expected return type from the function context
        let expected_type = self.current_return_type().unwrap_or(TypeId::ANY);

        // Get the type of the return expression (if any)
        let return_type = if !return_data.expression.is_none() {
            self.get_type_of_node(return_data.expression)
        } else {
            // `return;` without expression returns undefined
            TypeId::UNDEFINED
        };

        // Check if the return type is assignable to the expected type
        if expected_type != TypeId::ANY && !self.is_assignable_to(return_type, expected_type) {
            // Report error at the return expression (or at return keyword if no expression)
            let error_node = if !return_data.expression.is_none() {
                return_data.expression
            } else {
                stmt_idx
            };
            self.error_type_not_assignable_with_reason_at(return_type, expected_type, error_node);
        }

        if expected_type != TypeId::ANY && expected_type != TypeId::UNKNOWN && !return_data.expression.is_none() {
            if let Some(expr_node) = self.ctx.arena.get(return_data.expression) {
                if expr_node.kind == syntax_kind_ext::OBJECT_LITERAL_EXPRESSION {
                    self.check_object_literal_excess_properties(return_type, expected_type, return_data.expression);
                }
            }
        }
    }

    /// Check a class declaration.
    fn check_class_declaration(&mut self, stmt_idx: NodeIndex) {
        use crate::checker::types::diagnostics::diagnostic_codes;

        let Some(node) = self.ctx.arena.get(stmt_idx) else {
            return;
        };

        let Some(class) = self.ctx.arena.get_class(node) else {
            return;
        };

        // Check for reserved class names (error 2414)
        if !class.name.is_none() {
            if let Some(name_node) = self.ctx.arena.get(class.name) {
                if let Some(ident) = self.ctx.arena.get_identifier(name_node) {
                    if ident.escaped_text == "any" {
                        self.error_at_node(
                            class.name,
                            "Class name cannot be 'any'.",
                            diagnostic_codes::CLASS_NAME_CANNOT_BE_ANY,
                        );
                    }
                }
            }
        }

        // Check if this is a declared class (ambient declaration)
        let is_declared = self.has_declare_modifier(&class.modifiers);

        // Check if this class is abstract
        let is_abstract_class = self.has_abstract_modifier(&class.modifiers);

        // Check for abstract members in non-abstract class (error 1253)
        // and private identifiers in ambient classes (error 2819)
        for &member_idx in &class.members.nodes {
            if let Some(member_node) = self.ctx.arena.get(member_idx) {
                // TS2819: Check for private identifiers in ambient classes
                if is_declared {
                    let member_name_idx = match member_node.kind {
                        syntax_kind_ext::PROPERTY_DECLARATION => {
                            self.ctx.arena.get_property_decl(member_node).map(|p| p.name)
                        }
                        syntax_kind_ext::METHOD_DECLARATION => {
                            self.ctx.arena.get_method_decl(member_node).map(|m| m.name)
                        }
                        syntax_kind_ext::GET_ACCESSOR | syntax_kind_ext::SET_ACCESSOR => {
                            self.ctx.arena.get_accessor(member_node).map(|a| a.name)
                        }
                        _ => None,
                    };

                    if let Some(name_idx) = member_name_idx {
                        if !name_idx.is_none() {
                            if let Some(name_node) = self.ctx.arena.get(name_idx) {
                                if name_node.kind == crate::scanner::SyntaxKind::PrivateIdentifier as u16 {
                                    use crate::checker::types::diagnostics::diagnostic_messages;
                                    self.error_at_node(
                                        name_idx,
                                        diagnostic_messages::PRIVATE_IDENTIFIER_IN_AMBIENT_CONTEXT,
                                        diagnostic_codes::PRIVATE_IDENTIFIER_IN_AMBIENT_CONTEXT,
                                    );
                                }
                            }
                        }
                    }
                }

                // Check for abstract members in non-abstract class
                if !is_abstract_class {
                    let member_has_abstract = match member_node.kind {
                        syntax_kind_ext::PROPERTY_DECLARATION => {
                            if let Some(prop) = self.ctx.arena.get_property_decl(member_node) {
                                self.has_abstract_modifier(&prop.modifiers)
                            } else {
                                false
                            }
                        }
                        syntax_kind_ext::METHOD_DECLARATION => {
                            if let Some(method) = self.ctx.arena.get_method_decl(member_node) {
                                self.has_abstract_modifier(&method.modifiers)
                            } else {
                                false
                            }
                        }
                        syntax_kind_ext::GET_ACCESSOR | syntax_kind_ext::SET_ACCESSOR => {
                            if let Some(accessor) = self.ctx.arena.get_accessor(member_node) {
                                self.has_abstract_modifier(&accessor.modifiers)
                            } else {
                                false
                            }
                        }
                        _ => false,
                    };

                    if member_has_abstract {
                        // Report on the 'abstract' keyword
                        self.error_at_node(
                            member_idx,
                            "Abstract properties can only appear within an abstract class.",
                            diagnostic_codes::ABSTRACT_ONLY_IN_ABSTRACT_CLASS,
                        );
                    }
                }
            }
        }

        // Push a scope for type parameters
        self.push_local_scope();

        // Add type parameters to scope
        if let Some(ref type_params) = class.type_parameters {
            for &tp_idx in &type_params.nodes {
                if let Some(tp_node) = self.ctx.arena.get(tp_idx) {
                    if let Some(tp) = self.ctx.arena.get_type_parameter(tp_node) {
                        if let Some(name_node) = self.ctx.arena.get(tp.name) {
                            if let Some(ident) = self.ctx.arena.get_identifier(name_node) {
                                // Add type parameter as a type (use ANY as placeholder)
                                self.add_local(ident.escaped_text.clone(), TypeId::ANY);
                            }
                        }
                    }
                }
            }
        }

        // Collect class name and static members for error 2662 suggestions
        let class_name = if !class.name.is_none() {
            if let Some(name_node) = self.ctx.arena.get(class.name) {
                if let Some(ident) = self.ctx.arena.get_identifier(name_node) {
                    Some(ident.escaped_text.clone())
                } else {
                    None
                }
            } else {
                None
            }
        } else {
            None
        };

        // Save previous enclosing class and set current
        let prev_enclosing_class = self.ctx.enclosing_class.take();
        if let Some(name) = class_name {
            self.ctx.enclosing_class = Some(EnclosingClassInfo {
                name,
                member_nodes: class.members.nodes.clone(),
                in_constructor: false,
                is_declared,
            });
        }

        // Check each class member
        for &member_idx in &class.members.nodes {
            self.check_class_member(member_idx);
        }

        // Check for missing method/constructor implementations (2389, 2390, 2391)
        // Skip for declared classes (ambient declarations don't need implementations)
        if !is_declared {
            self.check_class_member_implementations(&class.members.nodes);
        }

        // Check for accessor abstract consistency (error 2676)
        // Getter and setter must both be abstract or both non-abstract
        self.check_accessor_abstract_consistency(&class.members.nodes);

        // Check for getter/setter type compatibility (error 2322)
        // Getter return type must be assignable to setter parameter type
        self.check_accessor_type_compatibility(&class.members.nodes);

        // Check for property type compatibility with base class (error 2416)
        // Property type in derived class must be assignable to same property in base class
        self.check_property_inheritance_compatibility(stmt_idx, &class);

        // Check that non-abstract class implements all abstract members from base class (error 2654)
        self.check_abstract_member_implementations(stmt_idx, &class);

        // Restore previous enclosing class
        self.ctx.enclosing_class = prev_enclosing_class;

        self.pop_local_scope();
    }

    /// Check an interface declaration.
    fn check_interface_declaration(&mut self, stmt_idx: NodeIndex) {
        use crate::checker::types::diagnostics::diagnostic_codes;

        let Some(node) = self.ctx.arena.get(stmt_idx) else {
            return;
        };

        let Some(iface) = self.ctx.arena.get_interface(node) else {
            return;
        };

        // Check for reserved interface names (error 2427)
        if !iface.name.is_none() {
            if let Some(name_node) = self.ctx.arena.get(iface.name) {
                if let Some(ident) = self.ctx.arena.get_identifier(name_node) {
                    // Reserved type names that can't be used as interface names
                    match ident.escaped_text.as_str() {
                        "string" | "number" | "boolean" | "symbol" | "void" | "object" => {
                            self.error_at_node(
                                iface.name,
                                &format!("Interface name cannot be '{}'.", ident.escaped_text),
                                diagnostic_codes::INTERFACE_NAME_CANNOT_BE,
                            );
                        }
                        _ => {}
                    }
                }
            }
        }

        // Push a scope for type parameters
        self.push_local_scope();

        // Add type parameters to scope
        if let Some(ref type_params) = iface.type_parameters {
            for &tp_idx in &type_params.nodes {
                if let Some(tp_node) = self.ctx.arena.get(tp_idx) {
                    if let Some(tp) = self.ctx.arena.get_type_parameter(tp_node) {
                        if let Some(name_node) = self.ctx.arena.get(tp.name) {
                            if let Some(ident) = self.ctx.arena.get_identifier(name_node) {
                                // Add type parameter as a type (use ANY as placeholder)
                                self.add_local(ident.escaped_text.clone(), TypeId::ANY);
                            }
                        }
                    }
                }
            }
        }

        // Check each interface member for parameter properties
        for &member_idx in &iface.members.nodes {
            self.check_type_member_for_parameter_properties(member_idx);
        }

        // Check that interface correctly extends base interfaces (error 2430)
        self.check_interface_extension_compatibility(stmt_idx, &iface);

        self.pop_local_scope();
    }

    /// Check if a node has the `declare` modifier.
    fn has_declare_modifier(&self, modifiers: &Option<crate::parser::NodeList>) -> bool {
        use crate::scanner::SyntaxKind;
        if let Some(mods) = modifiers {
            for &mod_idx in &mods.nodes {
                if let Some(mod_node) = self.ctx.arena.get(mod_idx) {
                    if mod_node.kind == SyntaxKind::DeclareKeyword as u16 {
                        return true;
                    }
                }
            }
        }
        false
    }

    /// Check if a node has the `abstract` modifier.
    fn has_abstract_modifier(&self, modifiers: &Option<crate::parser::NodeList>) -> bool {
        use crate::scanner::SyntaxKind;
        if let Some(mods) = modifiers {
            for &mod_idx in &mods.nodes {
                if let Some(mod_node) = self.ctx.arena.get(mod_idx) {
                    if mod_node.kind == SyntaxKind::AbstractKeyword as u16 {
                        return true;
                    }
                }
            }
        }
        false
    }

    /// Check if modifiers include the 'static' keyword.
    fn has_static_modifier(&self, modifiers: &Option<crate::parser::NodeList>) -> bool {
        use crate::scanner::SyntaxKind;
        if let Some(mods) = modifiers {
            for &mod_idx in &mods.nodes {
                if let Some(mod_node) = self.ctx.arena.get(mod_idx) {
                    if mod_node.kind == SyntaxKind::StaticKeyword as u16 {
                        return true;
                    }
                }
            }
        }
        false
    }

    /// Get the const modifier node from a list of modifiers, if present.
    /// Returns the NodeIndex of the const modifier for error reporting.
    fn get_const_modifier(&self, modifiers: &Option<crate::parser::NodeList>) -> Option<NodeIndex> {
        use crate::scanner::SyntaxKind;
        if let Some(mods) = modifiers {
            for &mod_idx in &mods.nodes {
                if let Some(mod_node) = self.ctx.arena.get(mod_idx) {
                    if mod_node.kind == SyntaxKind::ConstKeyword as u16 {
                        return Some(mod_idx);
                    }
                }
            }
        }
        None
    }

    /// Check if a member with the given name is static by looking up its symbol flags.
    /// Uses the binder's symbol information for efficient O(1) flag checks.
    fn is_static_member(&self, member_nodes: &[NodeIndex], name: &str) -> bool {
        use crate::binder::symbol_flags;

        for &member_idx in member_nodes {
            // Get symbol for this member
            if let Some(sym_id) = self.ctx.binder.get_node_symbol(member_idx) {
                if let Some(symbol) = self.ctx.binder.get_symbol(sym_id) {
                    // Check if name matches and symbol has STATIC flag
                    if symbol.escaped_name == name && (symbol.flags & symbol_flags::STATIC != 0) {
                        return true;
                    }
                }
            }
        }
        false
    }

    /// Check if a member with the given name is an abstract property by looking up its symbol flags.
    /// Only checks properties (not methods) because accessing this.abstractMethod() in constructor is allowed.
    fn is_abstract_member(&self, member_nodes: &[NodeIndex], name: &str) -> bool {
        use crate::binder::symbol_flags;

        for &member_idx in member_nodes {
            // Get symbol for this member
            if let Some(sym_id) = self.ctx.binder.get_node_symbol(member_idx) {
                if let Some(symbol) = self.ctx.binder.get_symbol(sym_id) {
                    // Check if name matches and symbol has ABSTRACT flag (property only)
                    if symbol.escaped_name == name
                        && (symbol.flags & symbol_flags::ABSTRACT != 0)
                        && (symbol.flags & symbol_flags::PROPERTY != 0)
                    {
                        return true;
                    }
                }
            }
        }
        false
    }

    /// Recursively check a type node for parameter properties in function types.
    /// Function types (like `(x: T) => R` or `new (x: T) => R`) cannot have parameter properties.
    fn check_type_for_parameter_properties(&mut self, type_idx: NodeIndex) {
        let Some(node) = self.ctx.arena.get(type_idx) else {
            return;
        };

        // Check if this is a function type or constructor type
        if node.kind == syntax_kind_ext::FUNCTION_TYPE ||
           node.kind == syntax_kind_ext::CONSTRUCTOR_TYPE {
            if let Some(func_type) = self.ctx.arena.get_function_type(node) {
                // Check each parameter for parameter property modifiers
                self.check_parameter_properties(&func_type.parameters.nodes);
                // Recursively check the return type
                self.check_type_for_parameter_properties(func_type.type_annotation);
            }
        }
        // Check type literals (object types) for call/construct signatures
        else if node.kind == syntax_kind_ext::TYPE_LITERAL {
            if let Some(type_lit) = self.ctx.arena.get_type_literal(node) {
                for &member_idx in &type_lit.members.nodes {
                    self.check_type_member_for_parameter_properties(member_idx);
                }
            }
        }
        // Recursively check array types, union types, intersection types, etc.
        else if node.kind == syntax_kind_ext::ARRAY_TYPE {
            if let Some(arr) = self.ctx.arena.get_array_type(node) {
                self.check_type_for_parameter_properties(arr.element_type);
            }
        }
        else if node.kind == syntax_kind_ext::UNION_TYPE ||
                node.kind == syntax_kind_ext::INTERSECTION_TYPE {
            if let Some(composite) = self.ctx.arena.get_composite_type(node) {
                for &type_idx in &composite.types.nodes {
                    self.check_type_for_parameter_properties(type_idx);
                }
            }
        }
        else if node.kind == syntax_kind_ext::PARENTHESIZED_TYPE {
            if let Some(paren) = self.ctx.arena.get_wrapped_type(node) {
                self.check_type_for_parameter_properties(paren.type_node);
            }
        }
    }

    /// Check a type literal member for parameter properties (call/construct signatures).
    fn check_type_member_for_parameter_properties(&mut self, member_idx: NodeIndex) {
        let Some(node) = self.ctx.arena.get(member_idx) else {
            return;
        };

        // Check call signatures and construct signatures for parameter properties
        if node.kind == syntax_kind_ext::CALL_SIGNATURE ||
           node.kind == syntax_kind_ext::CONSTRUCT_SIGNATURE {
            if let Some(sig) = self.ctx.arena.get_signature(node) {
                if let Some(params) = &sig.parameters {
                    self.check_parameter_properties(&params.nodes);
                }
                // Recursively check the return type
                self.check_type_for_parameter_properties(sig.type_annotation);
            }
        }
        // Check method signatures in type literals
        else if node.kind == syntax_kind_ext::METHOD_SIGNATURE {
            if let Some(sig) = self.ctx.arena.get_signature(node) {
                if let Some(params) = &sig.parameters {
                    self.check_parameter_properties(&params.nodes);
                }
                self.check_type_for_parameter_properties(sig.type_annotation);
            }
        }
        // Check accessors in type literals/interfaces - cannot have body (error 1183)
        else if node.kind == syntax_kind_ext::GET_ACCESSOR || node.kind == syntax_kind_ext::SET_ACCESSOR {
            if let Some(accessor) = self.ctx.arena.get_accessor(node) {
                // Accessors in type literals and interfaces cannot have implementations
                if !accessor.body.is_none() {
                    use crate::checker::types::diagnostics::diagnostic_codes;
                    // Report error on the body
                    self.error_at_node(
                        accessor.body,
                        "An implementation cannot be declared in ambient contexts.",
                        diagnostic_codes::IMPLEMENTATION_CANNOT_BE_IN_AMBIENT_CONTEXT,
                    );
                }
            }
        }
    }

    /// Check that all method/constructor overload signatures have implementations.
    /// Reports errors 2389, 2390, 2391.
    fn check_class_member_implementations(&mut self, members: &[NodeIndex]) {
        use crate::checker::types::diagnostics::diagnostic_codes;

        let mut i = 0;
        while i < members.len() {
            let member_idx = members[i];
            let Some(node) = self.ctx.arena.get(member_idx) else {
                i += 1;
                continue;
            };

            match node.kind {
                syntax_kind_ext::CONSTRUCTOR => {
                    if let Some(ctor) = self.ctx.arena.get_constructor(node) {
                        if ctor.body.is_none() {
                            // Constructor overload signature - check for implementation
                            let has_impl = self.find_constructor_impl(members, i + 1);
                            if !has_impl {
                                self.error_at_node(
                                    member_idx,
                                    "Constructor implementation is missing.",
                                    diagnostic_codes::CONSTRUCTOR_IMPLEMENTATION_MISSING
                                );
                            }
                        }
                    }
                }
                syntax_kind_ext::METHOD_DECLARATION => {
                    if let Some(method) = self.ctx.arena.get_method_decl(node) {
                        // Abstract methods don't need implementations (they're meant for derived classes)
                        let is_abstract = self.has_abstract_modifier(&method.modifiers);
                        if method.body.is_none() && !is_abstract {
                            // Method overload signature - check for implementation
                            let method_name = self.get_method_name_from_node(member_idx);
                            if let Some(name) = method_name {
                                let (has_impl, impl_name) = self.find_method_impl(members, i + 1, &name);
                                if !has_impl {
                                    self.error_at_node(
                                        member_idx,
                                        "Function implementation is missing or not immediately following the declaration.",
                                        diagnostic_codes::FUNCTION_IMPLEMENTATION_MISSING
                                    );
                                } else if let Some(actual_name) = impl_name {
                                    if actual_name != name {
                                        // Implementation has wrong name
                                        self.error_at_node(
                                            members[i + 1],
                                            &format!("Function implementation name must be '{}'.", name),
                                            diagnostic_codes::FUNCTION_IMPLEMENTATION_NAME_MUST_BE
                                        );
                                    }
                                }
                            }
                        }
                    }
                }
                _ => {}
            }
            i += 1;
        }
    }

    /// Check if there's a constructor implementation after position `start`.
    fn find_constructor_impl(&self, members: &[NodeIndex], start: usize) -> bool {
        for i in start..members.len() {
            let member_idx = members[i];
            let Some(node) = self.ctx.arena.get(member_idx) else {
                continue;
            };
            if node.kind == syntax_kind_ext::CONSTRUCTOR {
                if let Some(ctor) = self.ctx.arena.get_constructor(node) {
                    if !ctor.body.is_none() {
                        return true;
                    }
                    // Another constructor overload - keep looking
                }
            } else {
                // Non-constructor member - no implementation found
                return false;
            }
        }
        false
    }

    /// Check if there's a method implementation with the given name after position `start`.
    fn find_method_impl(&self, members: &[NodeIndex], start: usize, name: &str) -> (bool, Option<String>) {
        if start >= members.len() {
            return (false, None);
        }

        let member_idx = members[start];
        let Some(node) = self.ctx.arena.get(member_idx) else {
            return (false, None);
        };

        if node.kind == syntax_kind_ext::METHOD_DECLARATION {
            if let Some(method) = self.ctx.arena.get_method_decl(node) {
                if !method.body.is_none() {
                    // This is an implementation - check if name matches
                    let impl_name = self.get_method_name_from_node(member_idx);
                    if let Some(ref impl_name_str) = impl_name {
                        return (true, impl_name);
                    }
                }
            }
        }
        (false, None)
    }

    /// Check that accessor pairs (get/set) have consistent abstract modifiers.
    /// Reports error TS2676 if one is abstract and the other is not.
    fn check_accessor_abstract_consistency(&mut self, members: &[NodeIndex]) {
        use crate::checker::types::diagnostics::diagnostic_codes;
        use std::collections::HashMap;

        // Collect getters and setters by name
        #[derive(Default)]
        struct AccessorPair {
            getter: Option<(NodeIndex, bool)>,  // (node_idx, is_abstract)
            setter: Option<(NodeIndex, bool)>,
        }

        let mut accessors: HashMap<String, AccessorPair> = HashMap::new();

        for &member_idx in members {
            let Some(node) = self.ctx.arena.get(member_idx) else {
                continue;
            };

            if node.kind == syntax_kind_ext::GET_ACCESSOR || node.kind == syntax_kind_ext::SET_ACCESSOR {
                if let Some(accessor) = self.ctx.arena.get_accessor(node) {
                    let is_abstract = self.has_abstract_modifier(&accessor.modifiers);

                    // Get accessor name
                    if let Some(name) = self.get_property_name(accessor.name) {
                        let pair = accessors.entry(name).or_default();
                        if node.kind == syntax_kind_ext::GET_ACCESSOR {
                            pair.getter = Some((member_idx, is_abstract));
                        } else {
                            pair.setter = Some((member_idx, is_abstract));
                        }
                    }
                }
            }
        }

        // Check for abstract mismatch
        for (_, pair) in accessors {
            if let (Some((getter_idx, getter_abstract)), Some((setter_idx, setter_abstract))) =
                (pair.getter, pair.setter)
            {
                if getter_abstract != setter_abstract {
                    // Report error on both accessors
                    self.error_at_node(
                        getter_idx,
                        "Accessors must both be abstract or non-abstract.",
                        diagnostic_codes::ACCESSORS_MUST_BOTH_BE_ABSTRACT_OR_NOT,
                    );
                    self.error_at_node(
                        setter_idx,
                        "Accessors must both be abstract or non-abstract.",
                        diagnostic_codes::ACCESSORS_MUST_BOTH_BE_ABSTRACT_OR_NOT,
                    );
                }
            }
        }
    }

    /// Check that accessor pairs (get/set) have compatible types.
    /// The getter return type must be assignable to the setter parameter type.
    /// Reports error TS2322 on the return statement of the getter if types mismatch.
    /// Note: Abstract accessors are skipped - they don't need type compatibility checks.
    fn check_accessor_type_compatibility(&mut self, members: &[NodeIndex]) {
        use crate::checker::types::diagnostics::diagnostic_codes;
        use std::collections::HashMap;

        // Collect getter return types and setter parameter types
        struct AccessorTypeInfo {
            getter: Option<(NodeIndex, TypeId, NodeIndex, bool)>,  // (accessor_idx, return_type, body_or_return_pos, is_abstract)
            setter: Option<(NodeIndex, TypeId, bool)>,  // (accessor_idx, param_type, is_abstract)
        }

        let mut accessors: HashMap<String, AccessorTypeInfo> = HashMap::new();

        for &member_idx in members {
            let Some(node) = self.ctx.arena.get(member_idx) else {
                continue;
            };

            if node.kind == syntax_kind_ext::GET_ACCESSOR {
                if let Some(accessor) = self.ctx.arena.get_accessor(node) {
                    if let Some(name) = self.get_property_name(accessor.name) {
                        // Check if this accessor is abstract
                        let is_abstract = self.has_abstract_modifier(&accessor.modifiers);

                        // Get the return type - check explicit annotation first
                        let return_type = if !accessor.type_annotation.is_none() {
                            self.get_type_of_node(accessor.type_annotation)
                        } else {
                            // Infer from return statements in body
                            self.infer_getter_return_type(accessor.body)
                        };

                        // Find the position of the return statement for error reporting
                        let error_pos = self.find_return_statement_pos(accessor.body)
                            .unwrap_or(member_idx);

                        let info = accessors.entry(name).or_insert_with(|| AccessorTypeInfo {
                            getter: None,
                            setter: None,
                        });
                        info.getter = Some((member_idx, return_type, error_pos, is_abstract));
                    }
                }
            } else if node.kind == syntax_kind_ext::SET_ACCESSOR {
                if let Some(accessor) = self.ctx.arena.get_accessor(node) {
                    if let Some(name) = self.get_property_name(accessor.name) {
                        // Check if this accessor is abstract
                        let is_abstract = self.has_abstract_modifier(&accessor.modifiers);

                        // Get the parameter type from the setter's first parameter
                        let param_type = if let Some(&first_param_idx) = accessor.parameters.nodes.first() {
                            if let Some(param_node) = self.ctx.arena.get(first_param_idx) {
                                if let Some(param) = self.ctx.arena.get_parameter(param_node) {
                                    if !param.type_annotation.is_none() {
                                        self.get_type_of_node(param.type_annotation)
                                    } else {
                                        TypeId::ANY
                                    }
                                } else {
                                    TypeId::ANY
                                }
                            } else {
                                TypeId::ANY
                            }
                        } else {
                            TypeId::ANY
                        };

                        let info = accessors.entry(name).or_insert_with(|| AccessorTypeInfo {
                            getter: None,
                            setter: None,
                        });
                        info.setter = Some((member_idx, param_type, is_abstract));
                    }
                }
            }
        }

        // Check type compatibility for each accessor pair
        for (_, info) in accessors {
            if let (Some((_getter_idx, getter_type, error_pos, getter_abstract)), Some((_setter_idx, setter_type, setter_abstract))) =
                (info.getter, info.setter)
            {
                // Skip if either accessor is abstract - abstract accessors don't need type compatibility checks
                if getter_abstract || setter_abstract {
                    continue;
                }

                // Skip if either type is ANY (no meaningful check)
                if getter_type == TypeId::ANY || setter_type == TypeId::ANY {
                    continue;
                }

                // Check if getter return type is assignable to setter param type
                if !self.is_assignable_to(getter_type, setter_type) {
                    // Get type strings for error message
                    let getter_type_str = self.format_type(getter_type);
                    let setter_type_str = self.format_type(setter_type);

                    self.error_at_node(
                        error_pos,
                        &format!("Type '{}' is not assignable to type '{}'.", getter_type_str, setter_type_str),
                        diagnostic_codes::TYPE_NOT_ASSIGNABLE_TO_TYPE,
                    );
                }
            }
        }
    }

    /// Infer the return type of a getter from its body.
    fn infer_getter_return_type(&mut self, body_idx: NodeIndex) -> TypeId {
        if body_idx.is_none() {
            return TypeId::ANY;
        }

        let Some(body_node) = self.ctx.arena.get(body_idx) else {
            return TypeId::ANY;
        };

        // If it's a block, look for return statements
        if body_node.kind == syntax_kind_ext::BLOCK {
            if let Some(block) = self.ctx.arena.get_block(body_node) {
                for &stmt_idx in &block.statements.nodes {
                    if let Some(stmt_node) = self.ctx.arena.get(stmt_idx) {
                        if stmt_node.kind == syntax_kind_ext::RETURN_STATEMENT {
                            if let Some(ret) = self.ctx.arena.get_return_statement(stmt_node) {
                                if !ret.expression.is_none() {
                                    return self.get_type_of_node(ret.expression);
                                }
                            }
                        }
                    }
                }
            }
        }

        TypeId::ANY
    }

    /// Find the position of the first return statement's expression in a body.
    fn find_return_statement_pos(&self, body_idx: NodeIndex) -> Option<NodeIndex> {
        if body_idx.is_none() {
            return None;
        }

        let body_node = self.ctx.arena.get(body_idx)?;

        if body_node.kind == syntax_kind_ext::BLOCK {
            if let Some(block) = self.ctx.arena.get_block(body_node) {
                for &stmt_idx in &block.statements.nodes {
                    if let Some(stmt_node) = self.ctx.arena.get(stmt_idx) {
                        if stmt_node.kind == syntax_kind_ext::RETURN_STATEMENT {
                            if let Some(ret) = self.ctx.arena.get_return_statement(stmt_node) {
                                if !ret.expression.is_none() {
                                    return Some(ret.expression);
                                }
                            }
                        }
                    }
                }
            }
        }

        None
    }

    /// Check that property types in derived class are compatible with base class (error 2416).
    /// For each property/accessor in the derived class, checks if there's a corresponding
    /// member in the base class with incompatible type.
    fn check_property_inheritance_compatibility(
        &mut self,
        _class_idx: NodeIndex,
        class_data: &crate::parser::thin_node::ClassData,
    ) {
        use crate::checker::types::diagnostics::diagnostic_codes;
        use crate::scanner::SyntaxKind;

        // Find base class from heritage clauses (extends, not implements)
        let Some(ref heritage_clauses) = class_data.heritage_clauses else {
            return;
        };

        let mut base_class_idx: Option<NodeIndex> = None;
        let mut base_class_name = String::new();

        for &clause_idx in &heritage_clauses.nodes {
            let Some(clause_node) = self.ctx.arena.get(clause_idx) else {
                continue;
            };

            let Some(heritage) = self.ctx.arena.get_heritage_clause(clause_node) else {
                continue;
            };

            // Only check extends clauses (token = ExtendsKeyword = 96)
            if heritage.token != SyntaxKind::ExtendsKeyword as u16 {
                continue;
            }

            // Get the first type in the extends clause (the base class)
            if let Some(&type_idx) = heritage.types.nodes.first() {
                if let Some(type_node) = self.ctx.arena.get(type_idx) {
                    // Handle both cases:
                    // 1. ExpressionWithTypeArguments (e.g., Base<T>)
                    // 2. Simple Identifier (e.g., Base)
                    let expr_idx = if let Some(expr_type_args) = self.ctx.arena.get_expr_type_args(type_node) {
                        expr_type_args.expression
                    } else {
                        // For simple identifiers without type arguments, the type_node itself is the identifier
                        type_idx
                    };

                    // Get the class name from the expression (identifier)
                    if let Some(expr_node) = self.ctx.arena.get(expr_idx) {
                        if let Some(ident) = self.ctx.arena.get_identifier(expr_node) {
                            base_class_name = ident.escaped_text.clone();

                            // Find the base class declaration via symbol lookup
                            if let Some(sym_id) = self.ctx.binder.file_locals.get(&base_class_name) {
                                if let Some(symbol) = self.ctx.binder.get_symbol(sym_id) {
                                    // Try value_declaration first, then declarations
                                    if !symbol.value_declaration.is_none() {
                                        base_class_idx = Some(symbol.value_declaration);
                                    } else if let Some(&decl_idx) = symbol.declarations.first() {
                                        base_class_idx = Some(decl_idx);
                                    }
                                }
                            }
                        }
                    }
                }
            }
            break; // Only one extends clause is valid
        }

        // If no base class found, nothing to check
        let Some(base_idx) = base_class_idx else {
            return;
        };

        // Get the base class data
        let Some(base_node) = self.ctx.arena.get(base_idx) else {
            return;
        };

        let Some(base_class) = self.ctx.arena.get_class(base_node) else {
            return;
        };

        // Get the derived class name for the error message
        let derived_class_name = if !class_data.name.is_none() {
            if let Some(name_node) = self.ctx.arena.get(class_data.name) {
                if let Some(ident) = self.ctx.arena.get_identifier(name_node) {
                    ident.escaped_text.clone()
                } else {
                    String::from("<anonymous>")
                }
            } else {
                String::from("<anonymous>")
            }
        } else {
            String::from("<anonymous>")
        };

        // Check each member in the derived class
        for &member_idx in &class_data.members.nodes {
            let Some(member_node) = self.ctx.arena.get(member_idx) else {
                continue;
            };

            // Get the member name and type
            let (member_name, member_type, member_name_idx) = match member_node.kind {
                k if k == syntax_kind_ext::PROPERTY_DECLARATION => {
                    let Some(prop) = self.ctx.arena.get_property_decl(member_node) else {
                        continue;
                    };
                    let Some(name) = self.get_property_name(prop.name) else {
                        continue;
                    };

                    // Skip static properties
                    if self.has_static_modifier(&prop.modifiers) {
                        continue;
                    }

                    // Get the type: either from annotation or inferred from initializer
                    let prop_type = if !prop.type_annotation.is_none() {
                        self.get_type_from_type_node(prop.type_annotation)
                    } else if !prop.initializer.is_none() {
                        self.get_type_of_node(prop.initializer)
                    } else {
                        TypeId::ANY
                    };

                    (name, prop_type, prop.name)
                }
                k if k == syntax_kind_ext::GET_ACCESSOR => {
                    let Some(accessor) = self.ctx.arena.get_accessor(member_node) else {
                        continue;
                    };
                    let Some(name) = self.get_property_name(accessor.name) else {
                        continue;
                    };

                    // Skip static accessors
                    if self.has_static_modifier(&accessor.modifiers) {
                        continue;
                    }

                    // Get the return type
                    let accessor_type = if !accessor.type_annotation.is_none() {
                        self.get_type_from_type_node(accessor.type_annotation)
                    } else {
                        self.infer_getter_return_type(accessor.body)
                    };

                    (name, accessor_type, accessor.name)
                }
                _ => continue,
            };

            // Skip if type is ANY (no meaningful check)
            if member_type == TypeId::ANY {
                continue;
            }

            // Look for a matching member in the base class
            for &base_member_idx in &base_class.members.nodes {
                let Some(base_member_node) = self.ctx.arena.get(base_member_idx) else {
                    continue;
                };

                let (base_name, base_type) = match base_member_node.kind {
                    k if k == syntax_kind_ext::PROPERTY_DECLARATION => {
                        let Some(base_prop) = self.ctx.arena.get_property_decl(base_member_node) else {
                            continue;
                        };
                        let Some(name) = self.get_property_name(base_prop.name) else {
                            continue;
                        };

                        // Skip static properties
                        if self.has_static_modifier(&base_prop.modifiers) {
                            continue;
                        }

                        let prop_type = if !base_prop.type_annotation.is_none() {
                            self.get_type_from_type_node(base_prop.type_annotation)
                        } else if !base_prop.initializer.is_none() {
                            self.get_type_of_node(base_prop.initializer)
                        } else {
                            TypeId::ANY
                        };

                        (name, prop_type)
                    }
                    k if k == syntax_kind_ext::GET_ACCESSOR => {
                        let Some(base_accessor) = self.ctx.arena.get_accessor(base_member_node) else {
                            continue;
                        };
                        let Some(name) = self.get_property_name(base_accessor.name) else {
                            continue;
                        };

                        // Skip static accessors
                        if self.has_static_modifier(&base_accessor.modifiers) {
                            continue;
                        }

                        let accessor_type = if !base_accessor.type_annotation.is_none() {
                            self.get_type_from_type_node(base_accessor.type_annotation)
                        } else {
                            self.infer_getter_return_type(base_accessor.body)
                        };

                        (name, accessor_type)
                    }
                    _ => continue,
                };

                // Skip if base type is ANY
                if base_type == TypeId::ANY {
                    continue;
                }

                // Check if names match
                if member_name != base_name {
                    continue;
                }

                // Check type compatibility - derived type must be assignable to base type
                if !self.is_assignable_to(member_type, base_type) {
                    // Format type strings for error message
                    let member_type_str = self.format_type(member_type);
                    let base_type_str = self.format_type(base_type);

                    // Report error 2416 on the member name
                    self.error_at_node(
                        member_name_idx,
                        &format!(
                            "Property '{}' in type '{}' is not assignable to the same property in base type '{}'.",
                            member_name, derived_class_name, base_class_name
                        ),
                        diagnostic_codes::PROPERTY_NOT_ASSIGNABLE_TO_SAME_IN_BASE,
                    );

                    // Add secondary error with type details
                    if let Some((pos, end)) = self.get_node_span(member_name_idx) {
                        self.error(
                            pos,
                            end - pos,
                            format!("Type '{}' is not assignable to type '{}'.", member_type_str, base_type_str),
                            diagnostic_codes::PROPERTY_NOT_ASSIGNABLE_TO_SAME_IN_BASE,
                        );
                    }
                }

                break; // Found matching base member, no need to continue
            }
        }
    }

    /// Check that interface correctly extends its base interfaces (error 2430).
    /// For each member in the derived interface, checks if the same member in a base interface
    /// has an incompatible type.
    fn check_interface_extension_compatibility(
        &mut self,
        _iface_idx: NodeIndex,
        iface_data: &crate::parser::thin_node::InterfaceData,
    ) {
        use crate::checker::types::diagnostics::diagnostic_codes;
        use crate::parser::syntax_kind_ext::{METHOD_SIGNATURE, PROPERTY_SIGNATURE};
        use crate::scanner::SyntaxKind;

        // Get heritage clauses (extends)
        let Some(ref heritage_clauses) = iface_data.heritage_clauses else {
            return;
        };

        // Get the derived interface name for the error message
        let derived_name = if !iface_data.name.is_none() {
            if let Some(name_node) = self.ctx.arena.get(iface_data.name) {
                if let Some(ident) = self.ctx.arena.get_identifier(name_node) {
                    ident.escaped_text.clone()
                } else {
                    String::from("<anonymous>")
                }
            } else {
                String::from("<anonymous>")
            }
        } else {
            String::from("<anonymous>")
        };

        // Process each heritage clause (extends)
        for &clause_idx in &heritage_clauses.nodes {
            let Some(clause_node) = self.ctx.arena.get(clause_idx) else {
                continue;
            };

            let Some(heritage) = self.ctx.arena.get_heritage_clause(clause_node) else {
                continue;
            };

            // Only check extends clauses
            if heritage.token != SyntaxKind::ExtendsKeyword as u16 {
                continue;
            }

            // Process each extended interface
            for &type_idx in &heritage.types.nodes {
                let Some(type_node) = self.ctx.arena.get(type_idx) else {
                    continue;
                };

                // Get the base interface name and declaration
                let expr_idx = if let Some(expr_type_args) = self.ctx.arena.get_expr_type_args(type_node) {
                    expr_type_args.expression
                } else {
                    type_idx
                };

                let Some(expr_node) = self.ctx.arena.get(expr_idx) else {
                    continue;
                };

                let Some(ident) = self.ctx.arena.get_identifier(expr_node) else {
                    continue;
                };

                let base_name = ident.escaped_text.clone();

                // Find the base interface declaration via symbol lookup
                let base_iface_idx = if let Some(sym_id) = self.ctx.binder.file_locals.get(&base_name) {
                    if let Some(symbol) = self.ctx.binder.get_symbol(sym_id) {
                        if !symbol.value_declaration.is_none() {
                            Some(symbol.value_declaration)
                        } else if let Some(&decl_idx) = symbol.declarations.first() {
                            Some(decl_idx)
                        } else {
                            None
                        }
                    } else {
                        None
                    }
                } else {
                    None
                };

                let Some(base_idx) = base_iface_idx else {
                    continue;
                };

                let Some(base_node) = self.ctx.arena.get(base_idx) else {
                    continue;
                };

                let Some(base_iface) = self.ctx.arena.get_interface(base_node) else {
                    continue;
                };

                // Check each member in the derived interface against base members
                for &member_idx in &iface_data.members.nodes {
                    let Some(member_node) = self.ctx.arena.get(member_idx) else {
                        continue;
                    };

                    // Get member name and type
                    let (member_name, member_type) = if member_node.kind == METHOD_SIGNATURE || member_node.kind == PROPERTY_SIGNATURE {
                        if let Some(sig) = self.ctx.arena.get_signature(member_node) {
                            if let Some(name_node) = self.ctx.arena.get(sig.name) {
                                if let Some(id_data) = self.ctx.arena.get_identifier(name_node) {
                                    let name = id_data.escaped_text.clone();
                                    // Get the type of this member
                                    let type_id = self.get_type_of_interface_member(member_idx);
                                    (name, type_id)
                                } else {
                                    continue;
                                }
                            } else {
                                continue;
                            }
                        } else {
                            continue;
                        }
                    } else {
                        continue;
                    };

                    // Look for matching member in base interface
                    for &base_member_idx in &base_iface.members.nodes {
                        let Some(base_member_node) = self.ctx.arena.get(base_member_idx) else {
                            continue;
                        };

                        let (base_member_name, base_type) = if base_member_node.kind == METHOD_SIGNATURE || base_member_node.kind == PROPERTY_SIGNATURE {
                            if let Some(sig) = self.ctx.arena.get_signature(base_member_node) {
                                if let Some(name_node) = self.ctx.arena.get(sig.name) {
                                    if let Some(id_data) = self.ctx.arena.get_identifier(name_node) {
                                        let name = id_data.escaped_text.clone();
                                        let type_id = self.get_type_of_interface_member(base_member_idx);
                                        (name, type_id)
                                    } else {
                                        continue;
                                    }
                                } else {
                                    continue;
                                }
                            } else {
                                continue;
                            }
                        } else {
                            continue;
                        };

                        // Check if names match
                        if member_name != base_member_name {
                            continue;
                        }

                        // Check type compatibility - derived type must be assignable to base type
                        if !self.is_assignable_to(member_type, base_type) {
                            // Report error 2430 on the interface name (not the member)
                            let member_type_str = self.format_type(member_type);
                            let base_type_str = self.format_type(base_type);

                            self.error_at_node(
                                iface_data.name,
                                &format!(
                                    "Interface '{}' incorrectly extends interface '{}'.",
                                    derived_name, base_name
                                ),
                                diagnostic_codes::INTERFACE_INCORRECTLY_EXTENDS_INTERFACE,
                            );

                            // Add sub-error for property incompatibility
                            if let Some((pos, end)) = self.get_node_span(iface_data.name) {
                                self.error(
                                    pos,
                                    end - pos,
                                    format!("Types of property '{}' are incompatible.", member_name),
                                    diagnostic_codes::INTERFACE_INCORRECTLY_EXTENDS_INTERFACE,
                                );
                                self.error(
                                    pos,
                                    end - pos,
                                    format!(
                                        "Type '{}' is not assignable to type '{}'.",
                                        member_type_str, base_type_str
                                    ),
                                    diagnostic_codes::INTERFACE_INCORRECTLY_EXTENDS_INTERFACE,
                                );
                            }

                            // Only report first incompatibility per base interface
                            return;
                        }

                        break; // Found matching member, no need to continue
                    }
                }
            }
        }
    }

    /// Get the type of an interface member (method signature or property signature).
    fn get_type_of_interface_member(&mut self, member_idx: NodeIndex) -> TypeId {
        use crate::parser::syntax_kind_ext::{METHOD_SIGNATURE, PROPERTY_SIGNATURE};
        use crate::solver::FunctionShape;

        let Some(member_node) = self.ctx.arena.get(member_idx) else {
            return TypeId::ANY;
        };

        if member_node.kind == METHOD_SIGNATURE {
            // For method signatures, build a function type
            if let Some(sig) = self.ctx.arena.get_signature(member_node) {
                let (params, this_type) = self.extract_params_from_signature(sig);
                let return_type = if !sig.type_annotation.is_none() {
                    self.get_type_from_type_node(sig.type_annotation)
                } else {
                    TypeId::ANY
                };

                let shape = FunctionShape {
                    type_params: Vec::new(),
                    params,
                    this_type,
                    return_type,
                    type_predicate: None,
                    is_constructor: false,
                };

                return self.ctx.types.function(shape);
            }
        } else if member_node.kind == PROPERTY_SIGNATURE {
            // For property signatures, get the type annotation
            if let Some(sig) = self.ctx.arena.get_signature(member_node) {
                if !sig.type_annotation.is_none() {
                    return self.get_type_from_type_node(sig.type_annotation);
                }
            }
        }

        TypeId::ANY
    }

    /// Check that non-abstract class implements all abstract members from base class (error 2654).
    /// Reports "Non-abstract class 'X' is missing implementations for the following members of 'Y': {members}."
    fn check_abstract_member_implementations(
        &mut self,
        class_idx: NodeIndex,
        class_data: &crate::parser::thin_node::ClassData,
    ) {
        use crate::checker::types::diagnostics::diagnostic_codes;
        use crate::scanner::SyntaxKind;

        // Only check non-abstract classes
        if self.has_abstract_modifier(&class_data.modifiers) {
            return;
        }

        // Find base class from heritage clauses
        let Some(ref heritage_clauses) = class_data.heritage_clauses else {
            return;
        };

        let mut base_class_idx: Option<NodeIndex> = None;
        let mut base_class_name = String::new();

        for &clause_idx in &heritage_clauses.nodes {
            let Some(clause_node) = self.ctx.arena.get(clause_idx) else {
                continue;
            };

            let Some(heritage) = self.ctx.arena.get_heritage_clause(clause_node) else {
                continue;
            };

            // Only check extends clauses
            if heritage.token != SyntaxKind::ExtendsKeyword as u16 {
                continue;
            }

            // Get the base class
            if let Some(&type_idx) = heritage.types.nodes.first() {
                if let Some(type_node) = self.ctx.arena.get(type_idx) {
                    let expr_idx = if let Some(expr_type_args) = self.ctx.arena.get_expr_type_args(type_node) {
                        expr_type_args.expression
                    } else {
                        type_idx
                    };

                    if let Some(expr_node) = self.ctx.arena.get(expr_idx) {
                        if let Some(ident) = self.ctx.arena.get_identifier(expr_node) {
                            base_class_name = ident.escaped_text.clone();

                            if let Some(sym_id) = self.ctx.binder.file_locals.get(&base_class_name) {
                                if let Some(symbol) = self.ctx.binder.get_symbol(sym_id) {
                                    if !symbol.value_declaration.is_none() {
                                        base_class_idx = Some(symbol.value_declaration);
                                    } else if let Some(&decl_idx) = symbol.declarations.first() {
                                        base_class_idx = Some(decl_idx);
                                    }
                                }
                            }
                        }
                    }
                }
            }
            break;
        }

        let Some(base_idx) = base_class_idx else {
            return;
        };

        let Some(base_node) = self.ctx.arena.get(base_idx) else {
            return;
        };

        let Some(base_class) = self.ctx.arena.get_class(base_node) else {
            return;
        };

        // Collect implemented members from derived class
        let mut implemented_members = std::collections::HashSet::new();
        for &member_idx in &class_data.members.nodes {
            if let Some(name) = self.get_member_name(member_idx) {
                // Check if this member is not abstract (i.e., it's an implementation)
                if !self.member_is_abstract(member_idx) {
                    implemented_members.insert(name);
                }
            }
        }

        // Collect abstract members from base class that are not implemented
        let mut missing_members: Vec<String> = Vec::new();
        for &member_idx in &base_class.members.nodes {
            if self.member_is_abstract(member_idx) {
                if let Some(name) = self.get_member_name(member_idx) {
                    if !implemented_members.contains(&name) {
                        missing_members.push(name);
                    }
                }
            }
        }

        // Report error if there are missing implementations
        if !missing_members.is_empty() {
            let derived_class_name = if !class_data.name.is_none() {
                if let Some(name_node) = self.ctx.arena.get(class_data.name) {
                    if let Some(ident) = self.ctx.arena.get_identifier(name_node) {
                        ident.escaped_text.clone()
                    } else {
                        String::from("<anonymous>")
                    }
                } else {
                    String::from("<anonymous>")
                }
            } else {
                String::from("<anonymous>")
            };

            // Format: "Non-abstract class 'C' is missing implementations for the following members of 'B': 'prop', 'readonlyProp', 'm', 'mismatch'."
            let missing_list = missing_members
                .iter()
                .map(|s| format!("'{}'", s))
                .collect::<Vec<_>>()
                .join(", ");

            self.error_at_node(
                class_idx,
                &format!(
                    "Non-abstract class '{}' is missing implementations for the following members of '{}': {}.",
                    derived_class_name, base_class_name, missing_list
                ),
                diagnostic_codes::NON_ABSTRACT_CLASS_MISSING_IMPLEMENTATIONS,
            );
        }
    }

    /// Check if a class member has the abstract modifier.
    fn member_is_abstract(&self, member_idx: NodeIndex) -> bool {
        let Some(node) = self.ctx.arena.get(member_idx) else {
            return false;
        };

        match node.kind {
            k if k == syntax_kind_ext::PROPERTY_DECLARATION => {
                if let Some(prop) = self.ctx.arena.get_property_decl(node) {
                    self.has_abstract_modifier(&prop.modifiers)
                } else {
                    false
                }
            }
            k if k == syntax_kind_ext::METHOD_DECLARATION => {
                if let Some(method) = self.ctx.arena.get_method_decl(node) {
                    self.has_abstract_modifier(&method.modifiers)
                } else {
                    false
                }
            }
            k if k == syntax_kind_ext::GET_ACCESSOR || k == syntax_kind_ext::SET_ACCESSOR => {
                if let Some(accessor) = self.ctx.arena.get_accessor(node) {
                    self.has_abstract_modifier(&accessor.modifiers)
                } else {
                    false
                }
            }
            _ => false,
        }
    }

    /// Get the name of a class member (property, method, or accessor).
    fn get_member_name(&self, member_idx: NodeIndex) -> Option<String> {
        let Some(node) = self.ctx.arena.get(member_idx) else {
            return None;
        };

        let name_idx = match node.kind {
            k if k == syntax_kind_ext::PROPERTY_DECLARATION => {
                self.ctx.arena.get_property_decl(node).map(|p| p.name)
            }
            k if k == syntax_kind_ext::METHOD_DECLARATION => {
                self.ctx.arena.get_method_decl(node).map(|m| m.name)
            }
            k if k == syntax_kind_ext::GET_ACCESSOR || k == syntax_kind_ext::SET_ACCESSOR => {
                self.ctx.arena.get_accessor(node).map(|a| a.name)
            }
            _ => None,
        }?;

        self.get_property_name(name_idx)
    }

    /// Get the name of a method declaration.
    /// Handles both identifier names and numeric literal names.
    fn get_method_name_from_node(&self, member_idx: NodeIndex) -> Option<String> {
        let Some(node) = self.ctx.arena.get(member_idx) else {
            return None;
        };

        if let Some(method) = self.ctx.arena.get_method_decl(node) {
            let Some(name_node) = self.ctx.arena.get(method.name) else {
                return None;
            };
            // Try identifier first
            if let Some(id) = self.ctx.arena.get_identifier(name_node) {
                return Some(id.escaped_text.clone());
            }
            // Try numeric literal (for methods like 0(), 1(), etc.)
            if let Some(lit) = self.ctx.arena.get_literal(name_node) {
                return Some(lit.text.clone());
            }
        }
        None
    }

    /// Check that all top-level function overload signatures have implementations.
    /// Reports errors 2389, 2391.
    fn check_function_implementations(&mut self, statements: &[NodeIndex]) {
        use crate::checker::types::diagnostics::diagnostic_codes;

        let mut i = 0;
        while i < statements.len() {
            let stmt_idx = statements[i];
            let Some(node) = self.ctx.arena.get(stmt_idx) else {
                i += 1;
                continue;
            };

            if node.kind == syntax_kind_ext::FUNCTION_DECLARATION {
                if let Some(func) = self.ctx.arena.get_function(node) {
                    if func.body.is_none() {
                        // Function overload signature - check for implementation
                        let func_name = self.get_function_name_from_node(stmt_idx);
                        if let Some(name) = func_name {
                            let (has_impl, impl_name) = self.find_function_impl(statements, i + 1, &name);
                            if !has_impl {
                                self.error_at_node(
                                    stmt_idx,
                                    "Function implementation is missing or not immediately following the declaration.",
                                    diagnostic_codes::FUNCTION_IMPLEMENTATION_MISSING
                                );
                            } else if let Some(actual_name) = impl_name {
                                if actual_name != name {
                                    // Implementation has wrong name
                                    self.error_at_node(
                                        statements[i + 1],
                                        &format!("Function implementation name must be '{}'.", name),
                                        diagnostic_codes::FUNCTION_IMPLEMENTATION_NAME_MUST_BE
                                    );
                                }
                            }
                        }
                    }
                }
            }
            i += 1;
        }
    }

    /// Check if there's a function implementation with the given name after position `start`.
    fn find_function_impl(&self, statements: &[NodeIndex], start: usize, name: &str) -> (bool, Option<String>) {
        if start >= statements.len() {
            return (false, None);
        }

        let stmt_idx = statements[start];
        let Some(node) = self.ctx.arena.get(stmt_idx) else {
            return (false, None);
        };

        if node.kind == syntax_kind_ext::FUNCTION_DECLARATION {
            if let Some(func) = self.ctx.arena.get_function(node) {
                // Check if this is an implementation (has body)
                if !func.body.is_none() {
                    // This is an implementation - check if name matches
                    let impl_name = self.get_function_name_from_node(stmt_idx);
                    return (true, impl_name);
                } else {
                    // Another overload signature without body - need to look further
                    // but we should check if this is the same function name
                    let overload_name = self.get_function_name_from_node(stmt_idx);
                    if overload_name.as_ref() == Some(&name.to_string()) {
                        // Same function, continue looking for implementation
                        return self.find_function_impl(statements, start + 1, name);
                    }
                }
            }
        }

        (false, None)
    }

    /// Get the name of a function declaration.
    fn get_function_name_from_node(&self, stmt_idx: NodeIndex) -> Option<String> {
        let Some(node) = self.ctx.arena.get(stmt_idx) else {
            return None;
        };

        if let Some(func) = self.ctx.arena.get_function(node) {
            if !func.name.is_none() {
                let Some(name_node) = self.ctx.arena.get(func.name) else {
                    return None;
                };
                if let Some(id) = self.ctx.arena.get_identifier(name_node) {
                    return Some(id.escaped_text.clone());
                }
            }
        }
        None
    }

    /// Check a module body for function overload implementations.
    fn check_module_body(&mut self, body_idx: NodeIndex) {
        let Some(body_node) = self.ctx.arena.get(body_idx) else {
            return;
        };

        // Module body can be a MODULE_BLOCK or another MODULE_DECLARATION (for nested namespaces)
        if body_node.kind == syntax_kind_ext::MODULE_BLOCK {
            if let Some(block) = self.ctx.arena.get_module_block(body_node) {
                if let Some(ref statements) = block.statements {
                    // Check statements
                    for &stmt_idx in &statements.nodes {
                        self.check_statement(stmt_idx);
                    }
                    // Check for function overload implementations
                    self.check_function_implementations(&statements.nodes);
                }
            }
        } else if body_node.kind == syntax_kind_ext::MODULE_DECLARATION {
            // Nested namespace - recurse
            self.check_statement(body_idx);
        }
    }

    /// Check for export assignment with other exported elements (error 2309).
    /// `export = X` cannot be used when there are also `export class/function/etc.`
    /// Also checks that the exported expression exists (error 2304).
    fn check_export_assignment(&mut self, statements: &[NodeIndex]) {
        use crate::checker::types::diagnostics::diagnostic_codes;

        let mut export_assignment_idx: Option<NodeIndex> = None;
        let mut has_other_exports = false;

        for &stmt_idx in statements {
            let Some(node) = self.ctx.arena.get(stmt_idx) else {
                continue;
            };

            match node.kind {
                syntax_kind_ext::EXPORT_ASSIGNMENT => {
                    export_assignment_idx = Some(stmt_idx);

                    // Check that the exported expression exists
                    if let Some(export_data) = self.ctx.arena.get_export_assignment(node) {
                        // Get the type of the expression (this will report 2304 if not found)
                        self.get_type_of_node(export_data.expression);
                    }
                }
                syntax_kind_ext::EXPORT_DECLARATION => {
                    // export { ... } or export * from '...'
                    has_other_exports = true;
                }
                _ => {
                    // Check for export modifiers on declarations
                    // (export class X, export function f, export const x, etc.)
                    if self.has_export_modifier(stmt_idx) {
                        has_other_exports = true;
                    }
                }
            }
        }

        // Report error 2309 if there's an export assignment AND other exports
        if let Some(export_idx) = export_assignment_idx {
            if has_other_exports {
                self.error_at_node(
                    export_idx,
                    "An export assignment cannot be used in a module with other exported elements.",
                    diagnostic_codes::EXPORT_ASSIGNMENT_WITH_OTHER_EXPORTS,
                );
            }
        }
    }

    /// Check if a statement has an export modifier.
    fn has_export_modifier(&self, stmt_idx: NodeIndex) -> bool {
        use crate::scanner::SyntaxKind;

        let Some(node) = self.ctx.arena.get(stmt_idx) else {
            return false;
        };

        // Check different declaration types for export modifier
        let modifiers = match node.kind {
            syntax_kind_ext::FUNCTION_DECLARATION => {
                self.ctx.arena.get_function(node).and_then(|f| f.modifiers.as_ref())
            }
            syntax_kind_ext::CLASS_DECLARATION => {
                self.ctx.arena.get_class(node).and_then(|c| c.modifiers.as_ref())
            }
            syntax_kind_ext::VARIABLE_STATEMENT => {
                self.ctx.arena.get_variable(node).and_then(|v| v.modifiers.as_ref())
            }
            syntax_kind_ext::INTERFACE_DECLARATION => {
                self.ctx.arena.get_interface(node).and_then(|i| i.modifiers.as_ref())
            }
            syntax_kind_ext::TYPE_ALIAS_DECLARATION => {
                self.ctx.arena.get_type_alias(node).and_then(|t| t.modifiers.as_ref())
            }
            syntax_kind_ext::ENUM_DECLARATION => {
                self.ctx.arena.get_enum(node).and_then(|e| e.modifiers.as_ref())
            }
            _ => None,
        };

        if let Some(mods) = modifiers {
            for &mod_idx in &mods.nodes {
                if let Some(mod_node) = self.ctx.arena.get(mod_idx) {
                    if mod_node.kind == SyntaxKind::ExportKeyword as u16 {
                        return true;
                    }
                }
            }
        }

        false
    }

    /// Check that parameters don't have property modifiers (error 2369).
    /// Parameter properties (public/private/protected/readonly on params) are only
    /// allowed in constructor implementations.
    fn check_parameter_properties(&mut self, parameters: &[NodeIndex]) {
        use crate::checker::types::diagnostics::diagnostic_codes;

        for &param_idx in parameters {
            let Some(param_node) = self.ctx.arena.get(param_idx) else {
                continue;
            };
            let Some(param) = self.ctx.arena.get_parameter(param_node) else {
                continue;
            };

            // If the parameter has modifiers, it's a parameter property
            // which is only allowed in constructors
            if param.modifiers.is_some() {
                self.error_at_node(
                    param_idx,
                    "A parameter property is only allowed in a constructor implementation.",
                    diagnostic_codes::PARAMETER_PROPERTY_NOT_ALLOWED,
                );
            }
        }
    }

    /// Report an error at a specific node.
    fn error_at_node(&mut self, node_idx: NodeIndex, message: &str, code: u32) {
        if let Some((start, end)) = self.get_node_span(node_idx) {
            let length = end.saturating_sub(start);
            self.ctx.diagnostics.push(Diagnostic {
                file: self.ctx.file_name.clone(),
                start,
                length,
                message_text: message.to_string(),
                category: DiagnosticCategory::Error,
                code,
                related_information: Vec::new(),
            });
        }
    }

    /// Report an error at a specific position.
    fn error_at_position(&mut self, start: u32, length: u32, message: &str, code: u32) {
        self.ctx.diagnostics.push(Diagnostic {
            file: self.ctx.file_name.clone(),
            start,
            length,
            message_text: message.to_string(),
            category: DiagnosticCategory::Error,
            code,
            related_information: Vec::new(),
        });
    }

    /// Check a class member (property, method, constructor, accessor).
    fn check_class_member(&mut self, member_idx: NodeIndex) {
        let Some(node) = self.ctx.arena.get(member_idx) else {
            return;
        };

        match node.kind {
            syntax_kind_ext::PROPERTY_DECLARATION => {
                self.check_property_declaration(member_idx);
            }
            syntax_kind_ext::METHOD_DECLARATION => {
                self.check_method_declaration(member_idx);
            }
            syntax_kind_ext::CONSTRUCTOR => {
                self.check_constructor_declaration(member_idx);
            }
            syntax_kind_ext::GET_ACCESSOR | syntax_kind_ext::SET_ACCESSOR => {
                self.check_accessor_declaration(member_idx);
            }
            _ => {
                // Other class member types (static blocks, index signatures, etc.)
                self.get_type_of_node(member_idx);
            }
        }
    }

    /// Check for TS2729: Property is used before its initialization.
    /// This checks if a property initializer references another property via `this.X`
    /// where X is declared after the current property.
    fn check_property_initialization_order(&mut self, current_prop_idx: NodeIndex, initializer_idx: NodeIndex) {
        use crate::checker::types::diagnostics::diagnostic_codes;

        // Get class info to access member order
        let Some(class_info) = self.ctx.enclosing_class.clone() else {
            return;
        };

        // Find the position of the current property in the member list
        let Some(current_pos) = class_info.member_nodes.iter().position(|&idx| idx == current_prop_idx) else {
            return;
        };

        // Collect all `this.X` property accesses in the initializer
        let accesses = self.collect_this_property_accesses(initializer_idx);

        for (name, access_node_idx) in accesses {
            // Find if this name refers to another property in the class
            for (target_pos, &target_idx) in class_info.member_nodes.iter().enumerate() {
                if let Some(member_name) = self.get_member_name(target_idx) {
                    if member_name == name {
                        // Check if target is an instance property (not static, not a method)
                        if self.is_instance_property(target_idx) {
                            // Report 2729 if:
                            // 1. Target is declared after current property, OR
                            // 2. Target is an abstract property (no initializer in this class)
                            let should_error = target_pos > current_pos || self.is_abstract_property(target_idx);
                            if should_error {
                                self.error_at_node(
                                    access_node_idx,
                                    &format!("Property '{}' is used before its initialization.", name),
                                    diagnostic_codes::PROPERTY_USED_BEFORE_INITIALIZATION,
                                );
                            }
                        }
                        break;
                    }
                }
            }
        }
    }

    /// Check if a property declaration is abstract (has abstract modifier).
    fn is_abstract_property(&self, member_idx: NodeIndex) -> bool {
        let Some(node) = self.ctx.arena.get(member_idx) else {
            return false;
        };

        if node.kind != syntax_kind_ext::PROPERTY_DECLARATION {
            return false;
        }

        if let Some(prop) = self.ctx.arena.get_property_decl(node) {
            return self.has_abstract_modifier(&prop.modifiers);
        }

        false
    }

    /// Collect all `this.propertyName` accesses in an expression.
    /// Stops at function boundaries where `this` context changes.
    fn collect_this_property_accesses(&self, node_idx: NodeIndex) -> Vec<(String, NodeIndex)> {
        let mut accesses = Vec::new();
        self.collect_this_accesses_recursive(node_idx, &mut accesses);
        accesses
    }

    /// Recursive helper to collect this.X accesses.
    /// Uses the BinaryExprData structure which is used for property access in our arena.
    fn collect_this_accesses_recursive(&self, node_idx: NodeIndex, accesses: &mut Vec<(String, NodeIndex)>) {
        let Some(node) = self.ctx.arena.get(node_idx) else { return };

        // Stop at function boundaries where `this` context changes
        // (but not arrow functions, which preserve `this`)
        if node.kind == syntax_kind_ext::FUNCTION_EXPRESSION ||
           node.kind == syntax_kind_ext::FUNCTION_DECLARATION ||
           node.kind == syntax_kind_ext::CLASS_EXPRESSION ||
           node.kind == syntax_kind_ext::CLASS_DECLARATION {
            return;
        }

        // Property access uses AccessExprData with expression and name_or_argument
        if node.kind == syntax_kind_ext::PROPERTY_ACCESS_EXPRESSION {
            if let Some(access) = self.ctx.arena.get_access_expr(node) {
                // Check if the expression is `this`
                if let Some(expr_node) = self.ctx.arena.get(access.expression) {
                    if expr_node.kind == SyntaxKind::ThisKeyword as u16 {
                        // Get the property name
                        if let Some(name_node) = self.ctx.arena.get(access.name_or_argument) {
                            if let Some(ident) = self.ctx.arena.get_identifier(name_node) {
                                accesses.push((ident.escaped_text.clone(), node_idx));
                            }
                        }
                    } else {
                        // Recurse into the expression part
                        self.collect_this_accesses_recursive(access.expression, accesses);
                    }
                }
            }
            return;
        }

        // For other nodes, recurse into children based on node type
        match node.kind {
            k if k == syntax_kind_ext::BINARY_EXPRESSION => {
                if let Some(binary) = self.ctx.arena.get_binary_expr(node) {
                    self.collect_this_accesses_recursive(binary.left, accesses);
                    self.collect_this_accesses_recursive(binary.right, accesses);
                }
            }
            k if k == syntax_kind_ext::CALL_EXPRESSION => {
                if let Some(call) = self.ctx.arena.get_call_expr(node) {
                    self.collect_this_accesses_recursive(call.expression, accesses);
                    if let Some(ref args) = call.arguments {
                        for &arg in &args.nodes {
                            self.collect_this_accesses_recursive(arg, accesses);
                        }
                    }
                }
            }
            k if k == syntax_kind_ext::PARENTHESIZED_EXPRESSION => {
                if let Some(paren) = self.ctx.arena.get_parenthesized(node) {
                    self.collect_this_accesses_recursive(paren.expression, accesses);
                }
            }
            k if k == syntax_kind_ext::CONDITIONAL_EXPRESSION => {
                if let Some(cond) = self.ctx.arena.get_conditional_expr(node) {
                    self.collect_this_accesses_recursive(cond.condition, accesses);
                    self.collect_this_accesses_recursive(cond.when_true, accesses);
                    self.collect_this_accesses_recursive(cond.when_false, accesses);
                }
            }
            k if k == syntax_kind_ext::ARROW_FUNCTION => {
                // Arrow functions: while they preserve `this` context, property access
                // inside is deferred until the function is called. So we don't recurse
                // because the access doesn't happen during initialization.
                // (This matches TypeScript's behavior for error 2729)
            }
            _ => {
                // For other expressions, we don't recurse further to keep it simple
            }
        }
    }

    /// Check if a class member is an instance property (not static, not a method/accessor).
    fn is_instance_property(&self, member_idx: NodeIndex) -> bool {
        let Some(node) = self.ctx.arena.get(member_idx) else {
            return false;
        };

        if node.kind != syntax_kind_ext::PROPERTY_DECLARATION {
            return false;
        }

        if let Some(prop) = self.ctx.arena.get_property_decl(node) {
            // Check if it has a static modifier
            return !self.has_static_modifier(&prop.modifiers);
        }

        false
    }

    /// Check a property declaration.
    fn check_property_declaration(&mut self, member_idx: NodeIndex) {
        use crate::checker::types::diagnostics::diagnostic_codes;

        let Some(node) = self.ctx.arena.get(member_idx) else {
            return;
        };

        let Some(prop) = self.ctx.arena.get_property_decl(node) else {
            return;
        };

        // Error 1248: A class member cannot have the 'const' keyword
        if let Some(const_mod) = self.get_const_modifier(&prop.modifiers) {
            self.error_at_node(
                const_mod,
                "A class member cannot have the 'const' keyword.",
                diagnostic_codes::CONST_MODIFIER_CANNOT_APPEAR_ON_A_CLASS_ELEMENT,
            );
        }

        // If property has type annotation and initializer, check type compatibility
        if !prop.type_annotation.is_none() && !prop.initializer.is_none() {
            let declared_type = self.get_type_from_type_node(prop.type_annotation);
            let init_type = self.get_type_of_node(prop.initializer);

            if declared_type != TypeId::ANY && !self.is_assignable_to(init_type, declared_type) {
                self.error_type_not_assignable_with_reason_at(init_type, declared_type, prop.initializer);
            }
        } else if !prop.initializer.is_none() {
            // Just check the initializer to catch errors within it
            self.get_type_of_node(prop.initializer);
        }

        // Error 2729: Property is used before its initialization
        // Check if initializer references properties declared after this one
        if !prop.initializer.is_none() && !self.has_static_modifier(&prop.modifiers) {
            self.check_property_initialization_order(member_idx, prop.initializer);
        }
    }

    /// Check a method declaration.
    fn check_method_declaration(&mut self, member_idx: NodeIndex) {
        use crate::checker::types::diagnostics::diagnostic_codes;

        let Some(node) = self.ctx.arena.get(member_idx) else {
            return;
        };

        let Some(method) = self.ctx.arena.get_method_decl(node) else {
            return;
        };

        // Error 1248: A class member cannot have the 'const' keyword
        if let Some(const_mod) = self.get_const_modifier(&method.modifiers) {
            self.error_at_node(
                const_mod,
                "A class member cannot have the 'const' keyword.",
                diagnostic_codes::CONST_MODIFIER_CANNOT_APPEAR_ON_A_CLASS_ELEMENT,
            );
        }

        // Error 1183: An implementation cannot be declared in ambient contexts
        // Check if we're in a declared class and the method has a body
        if !method.body.is_none() {
            if let Some(ref class_info) = self.ctx.enclosing_class {
                if class_info.is_declared {
                    self.error_at_node(
                        member_idx,
                        "An implementation cannot be declared in ambient contexts.",
                        diagnostic_codes::IMPLEMENTATION_CANNOT_BE_IN_AMBIENT_CONTEXT,
                    );
                }
            }
        }

        // Enter a new local scope for the method body
        self.push_local_scope();

        // Get declared return type
        let return_type = if !method.type_annotation.is_none() {
            self.get_type_from_type_node(method.type_annotation)
        } else {
            TypeId::ANY
        };
        self.push_return_type(return_type);

        // Add parameters to local scope
        for &param_idx in &method.parameters.nodes {
            if let Some(param_node) = self.ctx.arena.get(param_idx) {
                if let Some(param) = self.ctx.arena.get_parameter(param_node) {
                    if let Some(name_node) = self.ctx.arena.get(param.name) {
                        if let Some(name_data) = self.ctx.arena.get_identifier(name_node) {
                            let param_type = if !param.type_annotation.is_none() {
                                self.get_type_from_type_node(param.type_annotation)
                            } else {
                                TypeId::ANY
                            };
                            self.add_local(name_data.escaped_text.clone(), param_type);
                        }
                    }
                }
            }
        }

        // Check for parameter properties (error 2369)
        // Parameter properties are only allowed in constructors, not in methods
        self.check_parameter_properties(&method.parameters.nodes);

        // Check parameter type annotations for parameter properties in function types
        for &param_idx in &method.parameters.nodes {
            if let Some(param_node) = self.ctx.arena.get(param_idx) {
                if let Some(param) = self.ctx.arena.get_parameter(param_node) {
                    if !param.type_annotation.is_none() {
                        self.check_type_for_parameter_properties(param.type_annotation);
                    }
                }
            }
        }

        // Check return type annotation for parameter properties in function types
        if !method.type_annotation.is_none() {
            self.check_type_for_parameter_properties(method.type_annotation);
        }

        // Check method body
        if !method.body.is_none() {
            self.check_statement(method.body);
        }

        self.pop_return_type();
        self.pop_local_scope();
    }

    /// Check a constructor declaration.
    fn check_constructor_declaration(&mut self, member_idx: NodeIndex) {
        use crate::checker::types::diagnostics::diagnostic_codes;

        let Some(node) = self.ctx.arena.get(member_idx) else {
            return;
        };

        let Some(ctor) = self.ctx.arena.get_constructor(node) else {
            return;
        };

        // Error 1183: An implementation cannot be declared in ambient contexts
        // Check if we're in a declared class and the constructor has a body
        if !ctor.body.is_none() {
            if let Some(ref class_info) = self.ctx.enclosing_class {
                if class_info.is_declared {
                    self.error_at_node(
                        member_idx,
                        "An implementation cannot be declared in ambient contexts.",
                        diagnostic_codes::IMPLEMENTATION_CANNOT_BE_IN_AMBIENT_CONTEXT,
                    );
                }
            }
        }

        // Check for parameter properties in constructor overload signatures (error 2369)
        // Parameter properties are only allowed in constructor implementations (with body)
        if ctor.body.is_none() {
            self.check_parameter_properties(&ctor.parameters.nodes);
        }

        // Check parameter type annotations for parameter properties in function types
        for &param_idx in &ctor.parameters.nodes {
            if let Some(param_node) = self.ctx.arena.get(param_idx) {
                if let Some(param) = self.ctx.arena.get_parameter(param_node) {
                    if !param.type_annotation.is_none() {
                        self.check_type_for_parameter_properties(param.type_annotation);
                    }
                }
            }
        }

        // Enter a new local scope for the constructor body
        self.push_local_scope();

        // Constructors don't have explicit return types

        // Add parameters to local scope
        for &param_idx in &ctor.parameters.nodes {
            if let Some(param_node) = self.ctx.arena.get(param_idx) {
                if let Some(param) = self.ctx.arena.get_parameter(param_node) {
                    if let Some(name_node) = self.ctx.arena.get(param.name) {
                        if let Some(name_data) = self.ctx.arena.get_identifier(name_node) {
                            let param_type = if !param.type_annotation.is_none() {
                                self.get_type_from_type_node(param.type_annotation)
                            } else {
                                TypeId::ANY
                            };
                            self.add_local(name_data.escaped_text.clone(), param_type);
                        }
                    }
                }
            }
        }

        // Set in_constructor flag for abstract property checks (error 2715)
        if let Some(ref mut class_info) = self.ctx.enclosing_class {
            class_info.in_constructor = true;
        }

        // Check constructor body
        if !ctor.body.is_none() {
            self.check_statement(ctor.body);
        }

        // Reset in_constructor flag
        if let Some(ref mut class_info) = self.ctx.enclosing_class {
            class_info.in_constructor = false;
        }

        self.pop_local_scope();
    }

    /// Check an accessor declaration (getter/setter).
    fn check_accessor_declaration(&mut self, member_idx: NodeIndex) {
        use crate::checker::types::diagnostics::diagnostic_codes;

        let Some(node) = self.ctx.arena.get(member_idx) else {
            return;
        };

        let Some(accessor) = self.ctx.arena.get_accessor(node) else {
            return;
        };

        // Error 1183: An implementation cannot be declared in ambient contexts
        // Check if we're in a declared class and the accessor has a body
        if !accessor.body.is_none() {
            if let Some(ref class_info) = self.ctx.enclosing_class {
                if class_info.is_declared {
                    self.error_at_node(
                        member_idx,
                        "An implementation cannot be declared in ambient contexts.",
                        diagnostic_codes::IMPLEMENTATION_CANNOT_BE_IN_AMBIENT_CONTEXT,
                    );
                }
            }
        }

        // Enter a new local scope for the accessor body
        self.push_local_scope();

        // For getters, get return type from annotation
        // For setters, return type is void
        let return_type = if node.kind == syntax_kind_ext::GET_ACCESSOR {
            if !accessor.type_annotation.is_none() {
                self.get_type_from_type_node(accessor.type_annotation)
            } else {
                TypeId::ANY
            }
        } else {
            TypeId::VOID
        };
        self.push_return_type(return_type);

        // Add parameters to local scope (for setters)
        for &param_idx in &accessor.parameters.nodes {
            if let Some(param_node) = self.ctx.arena.get(param_idx) {
                if let Some(param) = self.ctx.arena.get_parameter(param_node) {
                    if let Some(name_node) = self.ctx.arena.get(param.name) {
                        if let Some(name_data) = self.ctx.arena.get_identifier(name_node) {
                            let param_type = if !param.type_annotation.is_none() {
                                self.get_type_from_type_node(param.type_annotation)
                            } else {
                                TypeId::ANY
                            };
                            self.add_local(name_data.escaped_text.clone(), param_type);
                        }
                    }
                }
            }
        }

        // Check for parameter properties (error 2369)
        // Parameter properties are only allowed in constructors, not in accessors
        self.check_parameter_properties(&accessor.parameters.nodes);

        // For setters, check parameter constraints (1052, 1053)
        if node.kind == syntax_kind_ext::SET_ACCESSOR {
            self.check_setter_parameter(&accessor.parameters.nodes);
        }

        // Check accessor body
        if !accessor.body.is_none() {
            self.check_statement(accessor.body);
        }

        self.pop_return_type();
        self.pop_local_scope();
    }

    /// Check setter parameter constraints (1052, 1053).
    /// - A 'set' accessor parameter cannot have an initializer
    /// - A 'set' accessor cannot have rest parameter
    fn check_setter_parameter(&mut self, parameters: &[NodeIndex]) {
        use crate::checker::types::diagnostics::diagnostic_codes;

        for &param_idx in parameters {
            let Some(param_node) = self.ctx.arena.get(param_idx) else {
                continue;
            };
            let Some(param) = self.ctx.arena.get_parameter(param_node) else {
                continue;
            };

            // Check for initializer (error 1052)
            if !param.initializer.is_none() {
                self.error_at_node(
                    param.name,
                    "A 'set' accessor parameter cannot have an initializer.",
                    diagnostic_codes::SETTER_PARAMETER_CANNOT_HAVE_INITIALIZER,
                );
            }

            // Check for rest parameter (error 1053)
            if param.dot_dot_dot_token {
                self.error_at_node(
                    param_idx,
                    "A 'set' accessor cannot have rest parameter.",
                    diagnostic_codes::SETTER_CANNOT_HAVE_REST_PARAMETER,
                );
            }
        }
    }

    /// Check if a return type requires a return value.
    /// Returns false for void, undefined, any, and never.
    fn requires_return_value(&self, return_type: TypeId) -> bool {
        use crate::solver::TypeKey;

        // void, undefined, any, never don't require a return value
        if return_type == TypeId::VOID
            || return_type == TypeId::UNDEFINED
            || return_type == TypeId::ANY
            || return_type == TypeId::NEVER
        {
            return false;
        }

        // Check for union types that include void/undefined
        if let Some(TypeKey::Union(members)) = self.ctx.types.lookup(return_type) {
            for &member in &members {
                if member == TypeId::VOID || member == TypeId::UNDEFINED {
                    return false;
                }
            }
        }

        true
    }

    /// Check if a function body has at least one return statement with a value.
    /// This is a simplified check - doesn't do full control flow analysis.
    fn body_has_return_with_value(&self, body_idx: NodeIndex) -> bool {
        let Some(node) = self.ctx.arena.get(body_idx) else {
            return false;
        };

        // For block bodies, check all statements
        if node.kind == syntax_kind_ext::BLOCK {
            if let Some(block) = self.ctx.arena.get_block(node) {
                return self.statements_have_return_with_value(&block.statements.nodes);
            }
        }

        false
    }

    /// Check if any statement in the list contains a return with a value.
    fn statements_have_return_with_value(&self, statements: &[NodeIndex]) -> bool {
        for &stmt_idx in statements {
            if self.statement_has_return_with_value(stmt_idx) {
                return true;
            }
        }
        false
    }

    /// Check if a statement contains a return with a value.
    fn statement_has_return_with_value(&self, stmt_idx: NodeIndex) -> bool {
        let Some(node) = self.ctx.arena.get(stmt_idx) else {
            return false;
        };

        match node.kind {
            syntax_kind_ext::RETURN_STATEMENT => {
                if let Some(return_data) = self.ctx.arena.get_return_statement(node) {
                    // Return with expression
                    return !return_data.expression.is_none();
                }
                false
            }
            syntax_kind_ext::BLOCK => {
                if let Some(block) = self.ctx.arena.get_block(node) {
                    return self.statements_have_return_with_value(&block.statements.nodes);
                }
                false
            }
            syntax_kind_ext::IF_STATEMENT => {
                if let Some(if_data) = self.ctx.arena.get_if_statement(node) {
                    // Check both then and else branches
                    let then_has = self.statement_has_return_with_value(if_data.then_statement);
                    let else_has = if !if_data.else_statement.is_none() {
                        self.statement_has_return_with_value(if_data.else_statement)
                    } else {
                        false
                    };
                    return then_has || else_has;
                }
                false
            }
            syntax_kind_ext::SWITCH_STATEMENT => {
                if let Some(switch_data) = self.ctx.arena.get_switch(node) {
                    if let Some(case_block_node) = self.ctx.arena.get(switch_data.case_block) {
                        // Case block is stored as a Block containing case clauses
                        if let Some(case_block) = self.ctx.arena.get_block(case_block_node) {
                            for &clause_idx in &case_block.statements.nodes {
                                if let Some(clause_node) = self.ctx.arena.get(clause_idx) {
                                    if let Some(clause) = self.ctx.arena.get_case_clause(clause_node) {
                                        if self.statements_have_return_with_value(&clause.statements.nodes) {
                                            return true;
                                        }
                                    }
                                }
                            }
                        }
                    }
                }
                false
            }
            syntax_kind_ext::TRY_STATEMENT => {
                if let Some(try_data) = self.ctx.arena.get_try(node) {
                    let try_has = self.statement_has_return_with_value(try_data.try_block);
                    let catch_has = if !try_data.catch_clause.is_none() {
                        self.statement_has_return_with_value(try_data.catch_clause)
                    } else {
                        false
                    };
                    let finally_has = if !try_data.finally_block.is_none() {
                        self.statement_has_return_with_value(try_data.finally_block)
                    } else {
                        false
                    };
                    return try_has || catch_has || finally_has;
                }
                false
            }
            syntax_kind_ext::CATCH_CLAUSE => {
                if let Some(catch_data) = self.ctx.arena.get_catch_clause(node) {
                    return self.statement_has_return_with_value(catch_data.block);
                }
                false
            }
            syntax_kind_ext::WHILE_STATEMENT | syntax_kind_ext::DO_STATEMENT | syntax_kind_ext::FOR_STATEMENT | syntax_kind_ext::FOR_IN_STATEMENT | syntax_kind_ext::FOR_OF_STATEMENT => {
                if let Some(loop_data) = self.ctx.arena.get_loop(node) {
                    return self.statement_has_return_with_value(loop_data.statement);
                }
                false
            }
            _ => false,
        }
    }
}
