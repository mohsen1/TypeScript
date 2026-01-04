//! Checker state and core infrastructure.
//!
//! This module contains the CheckerState struct, diagnostic types,
//! and type guard/relation definitions.

use std::collections::{HashMap, HashSet};
use std::cell::RefCell;
use rustc_hash::FxHashMap;
use serde::Serialize;
use crate::binder::{SymbolId, SymbolArena, SymbolTable};
use crate::parser::NodeIndex;
use super::arena::TypeArena;
use super::types::TypeId;

// =============================================================================
// Diagnostic
// =============================================================================

/// Related information for a diagnostic (e.g., "see also" locations).
#[derive(Clone, Debug, Serialize)]
pub struct DiagnosticRelatedInformation {
    pub file: String,
    pub start: u32,
    pub length: u32,
    pub message_text: String,
    pub category: DiagnosticCategory,
    pub code: u32,
}

/// A type-checking diagnostic message with optional related information.
#[derive(Clone, Debug, Serialize)]
pub struct Diagnostic {
    pub file: String,
    pub start: u32,
    pub length: u32,
    pub message_text: String,
    pub category: DiagnosticCategory,
    pub code: u32,
    /// Related information spans (e.g., where a type was declared)
    #[serde(skip_serializing_if = "Vec::is_empty")]
    pub related_information: Vec<DiagnosticRelatedInformation>,
}

impl Diagnostic {
    /// Create a new error diagnostic.
    pub fn error(file: String, start: u32, length: u32, message: String, code: u32) -> Self {
        Diagnostic {
            file,
            start,
            length,
            message_text: message,
            category: DiagnosticCategory::Error,
            code,
            related_information: Vec::new(),
        }
    }

    /// Add related information to this diagnostic.
    pub fn with_related(mut self, file: String, start: u32, length: u32, message: String) -> Self {
        self.related_information.push(DiagnosticRelatedInformation {
            file,
            start,
            length,
            message_text: message,
            category: DiagnosticCategory::Message,
            code: 0,
        });
        self
    }
}

/// Diagnostic category.
#[derive(Clone, Copy, Debug, PartialEq, Eq, Serialize)]
pub enum DiagnosticCategory {
    Warning = 0,
    Error = 1,
    Suggestion = 2,
    Message = 3,
}

// =============================================================================
// Type Guard
// =============================================================================

/// Represents a type guard extracted from a condition expression.
#[derive(Debug, Clone)]
pub enum TypeGuard {
    /// typeof x === "string" style guard
    Typeof {
        target: NodeIndex,
        typeof_result: String,
        is_equality: bool,
    },
    /// x instanceof Foo style guard
    Instanceof {
        target: NodeIndex,
        constructor_type: TypeId,
        is_positive: bool,
    },
    /// x !== null / x !== undefined / x (truthiness) style guard
    Truthiness {
        target: NodeIndex,
        is_truthy: bool,
    },
    /// x.kind === "literal" style guard (discriminated unions)
    Discriminant {
        target: NodeIndex,
        property_name: String,
        discriminant_value: String,
        is_equality: bool,
    },
    /// "prop" in x style guard
    In {
        target: NodeIndex,
        property_name: String,
        is_positive: bool,
    },
}

// =============================================================================
// Type Relation
// =============================================================================

/// The type relationship being checked.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum TypeRelation {
    /// Assignment compatibility (most common)
    Assignable,
    /// Strict subtype relationship
    Subtype,
    /// Type identity (exact structural match)
    Identity,
    /// Comparable (for switch statements, equality checks)
    Comparable,
}

// =============================================================================
// Checker State
// =============================================================================

/// The type checker state.
/// Performs type inference and type checking on AST nodes.
pub struct CheckerState<'a> {
    /// The node arena containing the AST.
    pub node_arena: &'a crate::parser::NodeArena,

    /// The symbol arena containing bound symbols.
    pub symbol_arena: &'a SymbolArena,

    /// Symbol table for file-local name lookup.
    pub file_locals: &'a SymbolTable,

    /// The type arena for allocating types.
    pub types: TypeArena,

    /// Local symbol arena for checker-created symbols.
    pub(crate) local_symbols: SymbolArena,

    /// Cached types for symbols (FxHashMap for fast integer key hashing).
    pub(crate) symbol_types: FxHashMap<SymbolId, TypeId>,

    /// Cached types for nodes (FxHashMap for fast integer key hashing).
    pub(crate) node_types: FxHashMap<NodeIndex, TypeId>,

    /// Type parameter names for type_to_string.
    pub(crate) type_parameter_names: FxHashMap<TypeId, String>,

    /// Current type parameter scope (name -> TypeId).
    pub(crate) type_parameter_scope: HashMap<String, TypeId>,

    /// Diagnostics produced during type checking.
    pub diagnostics: Vec<Diagnostic>,

    /// Stack of symbols being resolved (to detect circular references).
    pub(crate) symbol_resolution_stack: Vec<SymbolId>,
    /// O(1) lookup set for symbol resolution stack.
    pub(crate) symbol_resolution_set: HashSet<SymbolId>,

    /// Stack of nodes being resolved (to detect circular references).
    pub(crate) node_resolution_stack: Vec<NodeIndex>,
    /// O(1) lookup set for node resolution stack.
    pub(crate) node_resolution_set: HashSet<NodeIndex>,

    /// Current file name.
    pub file_name: String,

    /// Contextual type for expression being checked.
    pub(crate) contextual_type: Option<TypeId>,

    /// Cache for type relation results (source, target, relation) -> result.
    /// Key: (source TypeId, target TypeId, relation as u8)
    /// Value: true if related, false otherwise
    /// Uses RefCell for interior mutability (allows caching from &self methods)
    pub(crate) relation_cache: RefCell<FxHashMap<(TypeId, TypeId, u8), bool>>,

    /// The enclosing class for visibility checks (private/protected).
    pub(crate) enclosing_class: Option<NodeIndex>,

    /// Current depth of recursive type instantiation (for depth limits).
    pub(crate) instantiation_depth: RefCell<u32>,

    /// Cache for awaited type results: TypeId → awaited TypeId.
    /// Avoids recomputing Promise unwrapping for the same type.
    pub(crate) awaited_type_cache: RefCell<FxHashMap<TypeId, TypeId>>,

    /// Cache for widened type results: TypeId → widened TypeId.
    /// For literal types, this is their base type (e.g., "hello" → string).
    /// For union of literals with same base, this is the base type.
    pub(crate) widened_type_cache: RefCell<FxHashMap<TypeId, TypeId>>,

    /// Cache for apparent type results: TypeId → apparent TypeId.
    /// The apparent type is the type that a value appears to have when used.
    /// For primitives, this is their wrapper object type (e.g., string → String).
    /// For type parameters, this is the constraint (or its apparent type).
    pub(crate) apparent_type_cache: RefCell<FxHashMap<TypeId, TypeId>>,

    /// Current depth of call expression resolution (for depth limits).
    pub(crate) call_depth: RefCell<u32>,

    /// Stack of local scopes for function parameters and block-scoped variables.
    /// Each entry maps variable names to their types.
    pub(crate) local_scope_stack: Vec<FxHashMap<String, TypeId>>,
}

/// Maximum depth for recursive type instantiation (conditional types, etc.).
/// TypeScript uses a depth limit of 50 by default.
pub const MAX_INSTANTIATION_DEPTH: u32 = 50;

/// Maximum depth for call expression resolution.
/// Prevents memory explosion from deeply nested callback type inference.
pub const MAX_CALL_DEPTH: u32 = 20;

impl<'a> CheckerState<'a> {
    /// Create a new checker state.
    pub fn new(
        node_arena: &'a crate::parser::NodeArena,
        symbol_arena: &'a SymbolArena,
        file_locals: &'a SymbolTable,
        file_name: String,
    ) -> Self {
        CheckerState {
            node_arena,
            symbol_arena,
            file_locals,
            types: TypeArena::new(),
            local_symbols: SymbolArena::new_with_base(SymbolArena::CHECKER_SYMBOL_BASE),
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
            enclosing_class: None,
            instantiation_depth: RefCell::new(0),
            awaited_type_cache: RefCell::new(FxHashMap::default()),
            widened_type_cache: RefCell::new(FxHashMap::default()),
            apparent_type_cache: RefCell::new(FxHashMap::default()),
            call_depth: RefCell::new(0),
            local_scope_stack: Vec::new(),
        }
    }

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

    /// Look up a local variable in the scope stack.
    pub fn lookup_local(&self, name: &str) -> Option<TypeId> {
        // Search from innermost to outermost scope
        for scope in self.local_scope_stack.iter().rev() {
            if let Some(&type_id) = scope.get(name) {
                return Some(type_id);
            }
        }
        None
    }

    /// Resolve a name to a symbol.
    pub fn resolve_name(&self, name: &str) -> Option<SymbolId> {
        self.file_locals.get(name)
    }

    /// Report a diagnostic error.
    pub fn error(&mut self, node: NodeIndex, message: &str, code: u32) {
        if let Some(n) = self.node_arena.get(node) {
            let base = n.base();
            self.diagnostics.push(Diagnostic::error(
                self.file_name.clone(),
                base.pos,
                base.end - base.pos,
                message.to_string(),
                code,
            ));
        }
    }

    /// Report a diagnostic error with related information.
    pub fn error_with_related(
        &mut self,
        node: NodeIndex,
        message: &str,
        code: u32,
        related_node: NodeIndex,
        related_message: &str,
    ) {
        if let Some(n) = self.node_arena.get(node) {
            let base = n.base();
            let mut diag = Diagnostic::error(
                self.file_name.clone(),
                base.pos,
                base.end - base.pos,
                message.to_string(),
                code,
            );

            // Add related information if the related node exists
            if let Some(related) = self.node_arena.get(related_node) {
                let related_base = related.base();
                diag = diag.with_related(
                    self.file_name.clone(),
                    related_base.pos,
                    related_base.end - related_base.pos,
                    related_message.to_string(),
                );
            }

            self.diagnostics.push(diag);
        }
    }

    /// Report a type assignability error with proper formatting.
    pub fn error_type_not_assignable(
        &mut self,
        node: NodeIndex,
        source_type: TypeId,
        target_type: TypeId,
    ) {
        use super::types::{diagnostic_codes, diagnostic_messages, format_message};

        let source_str = self.type_to_string(source_type);
        let target_str = self.type_to_string(target_type);
        let message = format_message(
            diagnostic_messages::TYPE_NOT_ASSIGNABLE,
            &[&source_str, &target_str],
        );
        self.error(node, &message, diagnostic_codes::TYPE_NOT_ASSIGNABLE_TO_TYPE);
    }

    /// Report a "cannot find name" error.
    pub fn error_cannot_find_name(&mut self, node: NodeIndex, name: &str) {
        use super::types::{diagnostic_codes, diagnostic_messages, format_message};

        let message = format_message(diagnostic_messages::CANNOT_FIND_NAME, &[name]);
        self.error(node, &message, diagnostic_codes::CANNOT_FIND_NAME);
    }

    /// Report a "property does not exist" error.
    pub fn error_property_not_found(&mut self, node: NodeIndex, prop_name: &str, type_id: TypeId) {
        use super::types::{diagnostic_codes, diagnostic_messages, format_message};

        let type_str = self.type_to_string(type_id);
        let message = format_message(
            diagnostic_messages::PROPERTY_DOES_NOT_EXIST,
            &[prop_name, &type_str],
        );
        self.error(node, &message, diagnostic_codes::PROPERTY_DOES_NOT_EXIST_ON_TYPE);
    }

    /// Report an argument count mismatch error.
    pub fn error_argument_count(&mut self, node: NodeIndex, expected: usize, actual: usize) {
        use super::types::{diagnostic_codes, diagnostic_messages, format_message};

        let message = format_message(
            diagnostic_messages::EXPECTED_ARGUMENTS,
            &[&expected.to_string(), &actual.to_string()],
        );
        self.error(node, &message, diagnostic_codes::EXPECTED_ARGUMENTS);
    }

    // =========================================================================
    // Modifier helpers
    // =========================================================================

    /// Check if a modifier list contains a specific modifier keyword.
    pub fn has_modifier(
        &self,
        modifiers: &Option<crate::parser::NodeList>,
        kind: crate::scanner::SyntaxKind,
    ) -> bool {
        use crate::parser::Node;

        if let Some(mods) = modifiers {
            for &mod_idx in &mods.nodes {
                if let Some(Node::Token(base)) = self.node_arena.get(mod_idx) {
                    if base.kind == kind as u16 {
                        return true;
                    }
                }
            }
        }
        false
    }

    /// Check if a class member has the abstract modifier.
    pub fn is_abstract_member(&self, member_idx: NodeIndex) -> bool {
        use crate::parser::Node;

        match self.node_arena.get(member_idx) {
            Some(Node::MethodDeclaration(md)) => {
                self.has_modifier(&md.modifiers, crate::scanner::SyntaxKind::AbstractKeyword)
            }
            Some(Node::PropertyDeclaration(pd)) => {
                self.has_modifier(&pd.modifiers, crate::scanner::SyntaxKind::AbstractKeyword)
            }
            Some(Node::GetAccessorDeclaration(ga)) => {
                self.has_modifier(&ga.modifiers, crate::scanner::SyntaxKind::AbstractKeyword)
            }
            Some(Node::SetAccessorDeclaration(sa)) => {
                self.has_modifier(&sa.modifiers, crate::scanner::SyntaxKind::AbstractKeyword)
            }
            _ => false,
        }
    }

    /// Check if a class member has the override modifier.
    pub fn has_override_modifier(&self, member_idx: NodeIndex) -> bool {
        use crate::parser::Node;

        match self.node_arena.get(member_idx) {
            Some(Node::MethodDeclaration(md)) => {
                self.has_modifier(&md.modifiers, crate::scanner::SyntaxKind::OverrideKeyword)
            }
            Some(Node::PropertyDeclaration(pd)) => {
                self.has_modifier(&pd.modifiers, crate::scanner::SyntaxKind::OverrideKeyword)
            }
            Some(Node::GetAccessorDeclaration(ga)) => {
                self.has_modifier(&ga.modifiers, crate::scanner::SyntaxKind::OverrideKeyword)
            }
            Some(Node::SetAccessorDeclaration(sa)) => {
                self.has_modifier(&sa.modifiers, crate::scanner::SyntaxKind::OverrideKeyword)
            }
            _ => false,
        }
    }

    /// Get the name of a class member.
    pub fn get_member_name(&self, member_idx: NodeIndex) -> Option<String> {
        use crate::parser::Node;

        match self.node_arena.get(member_idx) {
            Some(Node::MethodDeclaration(md)) => {
                self.get_identifier_text(md.name)
            }
            Some(Node::PropertyDeclaration(pd)) => {
                self.get_identifier_text(pd.name)
            }
            Some(Node::GetAccessorDeclaration(ga)) => {
                self.get_identifier_text(ga.name)
            }
            Some(Node::SetAccessorDeclaration(sa)) => {
                self.get_identifier_text(sa.name)
            }
            _ => None,
        }
    }

    /// Get the text of an identifier node.
    fn get_identifier_text(&self, node_idx: NodeIndex) -> Option<String> {
        use crate::parser::Node;

        match self.node_arena.get(node_idx) {
            Some(Node::Identifier(id)) => Some(id.escaped_text.clone()),
            _ => None,
        }
    }

    // =========================================================================
    // Variance helpers
    // =========================================================================

    /// Get variance modifier from a type parameter.
    /// Returns (is_in, is_out) tuple indicating contravariant and covariant positions.
    pub fn get_type_parameter_variance(&self, type_param_idx: NodeIndex) -> (bool, bool) {
        use crate::parser::Node;
        use crate::scanner::SyntaxKind;

        if let Some(Node::TypeParameterDeclaration(tp)) = self.node_arena.get(type_param_idx) {
            if let Some(ref mods) = tp.modifiers {
                let mut is_in = false;
                let mut is_out = false;
                for &mod_idx in &mods.nodes {
                    if let Some(Node::Token(base)) = self.node_arena.get(mod_idx) {
                        if base.kind == SyntaxKind::InKeyword as u16 {
                            is_in = true;
                        } else if base.kind == SyntaxKind::OutKeyword as u16 {
                            is_out = true;
                        }
                    }
                }
                return (is_in, is_out);
            }
        }
        (false, false)
    }

    // =========================================================================
    // Base class helpers
    // =========================================================================

    /// Get the base class symbol from a class declaration's heritage clauses.
    /// Returns None if there's no extends clause.
    pub fn get_base_class_symbol(&self, heritage_clauses: &Option<crate::parser::NodeList>) -> Option<crate::binder::SymbolId> {
        use crate::parser::Node;
        use crate::scanner::SyntaxKind;

        let clauses = heritage_clauses.as_ref()?;

        for &clause_idx in &clauses.nodes {
            if let Some(Node::HeritageClause(hc)) = self.node_arena.get(clause_idx) {
                // Check if this is an extends clause
                if hc.token == SyntaxKind::ExtendsKeyword as u16 {
                    // Get the first type (the base class)
                    if let Some(&type_idx) = hc.types.nodes.first() {
                        // Try ExpressionWithTypeArguments first
                        if let Some(Node::ExpressionWithTypeArguments(ewta)) = self.node_arena.get(type_idx) {
                            // Get the identifier from the expression
                            if let Some(Node::Identifier(id)) = self.node_arena.get(ewta.expression) {
                                // Look up the symbol
                                if let Some(symbol_id) = self.file_locals.get(&id.escaped_text) {
                                    return Some(symbol_id);
                                }
                            }
                        }
                        // Fallback: handle direct Identifier node
                        // (Parser may not always wrap in ExpressionWithTypeArguments)
                        else if let Some(Node::Identifier(id)) = self.node_arena.get(type_idx) {
                            if let Some(symbol_id) = self.file_locals.get(&id.escaped_text) {
                                return Some(symbol_id);
                            }
                        }
                    }
                }
            }
        }
        None
    }

    /// Check if a base class has a member with the given name.
    pub fn base_class_has_member(&self, base_symbol: crate::binder::SymbolId, member_name: &str) -> bool {
        use crate::parser::Node;

        // Get the base class symbol and its declaration
        if let Some(symbol) = self.symbol_arena.get(base_symbol) {
            if let Some(&decl_idx) = symbol.declarations.first() {
                if let Some(Node::ClassDeclaration(cd)) = self.node_arena.get(decl_idx) {
                    // Check all members of the base class
                    for &member_idx in &cd.members.nodes {
                        if let Some(name) = self.get_member_name(member_idx) {
                            if name == member_name {
                                return true;
                            }
                        }
                    }

                    // Recursively check the base class's base class
                    if let Some(grand_base_symbol) = self.get_base_class_symbol(&cd.heritage_clauses) {
                        if self.base_class_has_member(grand_base_symbol, member_name) {
                            return true;
                        }
                    }
                }
            }
        }
        false
    }

    /// Get the diagnostics as JSON.
    pub fn get_diagnostics_json(&self) -> String {
        serde_json::to_string(&self.diagnostics).unwrap_or_else(|_| "[]".to_string())
    }

    // =========================================================================
    // Language Service Support Methods
    // =========================================================================

    /// Get the symbol at a given node position.
    /// Used by go-to-definition, find-references, rename, etc.
    pub fn get_symbol_at_location(&self, node_idx: NodeIndex) -> Option<SymbolId> {
        use crate::parser::Node;

        match self.node_arena.get(node_idx) {
            Some(Node::Identifier(id)) => {
                // Look up in file locals (top-level declarations)
                self.file_locals.get(&id.escaped_text)
            }
            Some(Node::PropertyAccessExpression(pae)) => {
                // For property access, get the symbol of the property name
                self.get_symbol_at_location(pae.name)
            }
            Some(Node::TypeReference(tr)) => {
                // For type references, get the symbol of the type name
                self.get_symbol_at_location(tr.type_name)
            }
            Some(Node::QualifiedName { right, .. }) => {
                // For qualified names (A.B), get the symbol of the right side
                self.get_symbol_at_location(*right)
            }
            Some(Node::VariableDeclaration(vd)) => {
                // For variable declarations, get the symbol from the name
                self.get_symbol_at_location(vd.name)
            }
            Some(Node::FunctionDeclaration(fd)) => {
                if !fd.name.is_none() {
                    self.get_symbol_at_location(fd.name)
                } else {
                    None
                }
            }
            Some(Node::ClassDeclaration(cd)) => {
                if !cd.name.is_none() {
                    self.get_symbol_at_location(cd.name)
                } else {
                    None
                }
            }
            Some(Node::InterfaceDeclaration(id)) => {
                self.get_symbol_at_location(id.name)
            }
            Some(Node::TypeAliasDeclaration(tad)) => {
                self.get_symbol_at_location(tad.name)
            }
            Some(Node::EnumDeclaration(ed)) => {
                self.get_symbol_at_location(ed.name)
            }
            Some(Node::ParameterDeclaration(pd)) => {
                self.get_symbol_at_location(pd.name)
            }
            Some(Node::PropertyDeclaration(pd)) => {
                self.get_symbol_at_location(pd.name)
            }
            Some(Node::MethodDeclaration(md)) => {
                self.get_symbol_at_location(md.name)
            }
            _ => None,
        }
    }

    /// Get the type of a symbol (if cached).
    /// Returns the declared type for the symbol from the cache.
    pub fn get_cached_type_of_symbol(&self, symbol_id: SymbolId) -> Option<TypeId> {
        self.symbol_types.get(&symbol_id).copied()
    }

    /// Get all declarations for a symbol.
    /// Used for go-to-definition.
    pub fn get_symbol_declarations(&self, symbol_id: SymbolId) -> Vec<NodeIndex> {
        if let Some(symbol) = self.symbol_arena.get(symbol_id) {
            symbol.declarations.clone()
        } else {
            Vec::new()
        }
    }

    /// Get the name of a symbol.
    pub fn get_symbol_name(&self, symbol_id: SymbolId) -> Option<String> {
        self.symbol_arena.get(symbol_id).map(|s| s.escaped_name.clone())
    }

    /// Get the flags of a symbol.
    pub fn get_symbol_flags(&self, symbol_id: SymbolId) -> u32 {
        self.symbol_arena.get(symbol_id)
            .map(|s| s.flags)
            .unwrap_or(0)
    }

    /// Get all symbols in the file's local scope.
    /// Used for completions at global/module level.
    pub fn get_file_symbols(&self) -> impl Iterator<Item = (&String, &SymbolId)> {
        self.file_locals.iter()
    }

    /// Get the position (start, end) of a node.
    pub fn get_node_span(&self, node_idx: NodeIndex) -> Option<(u32, u32)> {
        self.node_arena.get(node_idx).map(|n| {
            let base = n.base();
            (base.pos, base.end)
        })
    }

    /// Get the kind of a node.
    pub fn get_node_kind(&self, node_idx: NodeIndex) -> Option<crate::scanner::SyntaxKind> {
        self.node_arena.get(node_idx).map(|n| {
            // Convert the u16 kind to SyntaxKind
            unsafe { std::mem::transmute(n.base().kind) }
        })
    }

    /// Find the innermost node at a given position.
    /// Used by language service to find what the user is hovering over.
    /// The root_idx should be the source file node.
    pub fn get_node_at_position(&self, root_idx: NodeIndex, position: u32) -> Option<NodeIndex> {
        self.find_node_at_position_recursive(root_idx, position)
    }

    fn find_node_at_position_recursive(&self, node_idx: NodeIndex, position: u32) -> Option<NodeIndex> {
        let node = self.node_arena.get(node_idx)?;
        let base = node.base();

        // Check if position is within this node
        if position < base.pos || position >= base.end {
            return None;
        }

        // Try to find a more specific child node
        for child_idx in self.get_node_children(node_idx) {
            if let Some(found) = self.find_node_at_position_recursive(child_idx, position) {
                return Some(found);
            }
        }

        // No child contains this position, return this node
        Some(node_idx)
    }

    /// Get the children of a node.
    fn get_node_children(&self, node_idx: NodeIndex) -> Vec<NodeIndex> {
        use crate::parser::Node;
        let mut children = Vec::new();

        if let Some(node) = self.node_arena.get(node_idx) {
            match node {
                Node::SourceFile(sf) => {
                    children.extend(sf.statements.nodes.iter().copied());
                }
                Node::Block(b) => {
                    children.extend(b.statements.nodes.iter().copied());
                }
                Node::VariableStatement(vs) => {
                    children.push(vs.declaration_list);
                }
                Node::VariableDeclarationList(vdl) => {
                    children.extend(vdl.declarations.nodes.iter().copied());
                }
                Node::VariableDeclaration(vd) => {
                    children.push(vd.name);
                    if !vd.type_annotation.is_none() {
                        children.push(vd.type_annotation);
                    }
                    if !vd.initializer.is_none() {
                        children.push(vd.initializer);
                    }
                }
                Node::FunctionDeclaration(fd) => {
                    if !fd.name.is_none() {
                        children.push(fd.name);
                    }
                    // parameters is a NodeList, not Option
                    children.extend(fd.parameters.nodes.iter().copied());
                    if !fd.type_annotation.is_none() {
                        children.push(fd.type_annotation);
                    }
                    if !fd.body.is_none() {
                        children.push(fd.body);
                    }
                }
                Node::ClassDeclaration(cd) => {
                    if !cd.name.is_none() {
                        children.push(cd.name);
                    }
                    children.extend(cd.members.nodes.iter().copied());
                }
                Node::InterfaceDeclaration(id) => {
                    children.push(id.name);
                    children.extend(id.members.nodes.iter().copied());
                }
                Node::CallExpression(ce) => {
                    children.push(ce.expression);
                    children.extend(ce.arguments.nodes.iter().copied());
                }
                Node::PropertyAccessExpression(pae) => {
                    children.push(pae.expression);
                    children.push(pae.name);
                }
                Node::BinaryExpression(be) => {
                    children.push(be.left);
                    children.push(be.right);
                }
                // Add more node types as needed
                _ => {}
            }
        }

        children
    }

    /// Get the number of types allocated.
    pub fn get_type_count(&self) -> usize {
        self.types.len()
    }

    /// Type to string for debugging.
    pub fn type_to_string(&self, type_id: TypeId) -> String {
        use super::types::{Type, LiteralValue};

        let Some(typ) = self.types.get(type_id) else {
            return "unknown".to_string();
        };

        match typ {
            Type::Intrinsic(i) => i.intrinsic_name.clone(),
            Type::Literal(lit) => match &lit.value {
                LiteralValue::String(s) => format!("\"{}\"", s),
                LiteralValue::Number(n) => n.to_string(),
                LiteralValue::BigInt(b) => format!("{}n", b),
                LiteralValue::Boolean(b) => b.to_string(),
            },
            Type::Union(u) => {
                let parts: Vec<String> = u.types.iter()
                    .map(|&t| self.type_to_string(t))
                    .collect();
                parts.join(" | ")
            }
            Type::Intersection(i) => {
                let parts: Vec<String> = i.types.iter()
                    .map(|&t| self.type_to_string(t))
                    .collect();
                parts.join(" & ")
            }
            Type::Object(obj) => {
                // For named types (interfaces, classes), print the name instead of expanding
                // to avoid infinite recursion for recursive types
                if !obj.symbol.is_none() {
                    if let Some(sym) = self.symbol_arena.get(obj.symbol) {
                        return sym.escaped_name.clone();
                    }
                }
                // Anonymous object type - print structure
                if obj.members.is_empty() && obj.properties.is_empty() {
                    "object".to_string()
                } else {
                    let mut parts = Vec::new();
                    // Limit depth to prevent stack overflow on circular types
                    for (name, &symbol_id) in obj.members.iter().take(10) {
                        if let Some(&prop_type) = self.symbol_types.get(&symbol_id) {
                            // Don't recursively expand object types
                            let prop_str = if let Some(Type::Object(inner_obj)) = self.types.get(prop_type) {
                                if !inner_obj.symbol.is_none() {
                                    if let Some(sym) = self.symbol_arena.get(inner_obj.symbol) {
                                        sym.escaped_name.clone()
                                    } else {
                                        "object".to_string()
                                    }
                                } else {
                                    "object".to_string()
                                }
                            } else {
                                self.type_to_string(prop_type)
                            };
                            parts.push(format!("{}: {}", name, prop_str));
                        } else {
                            parts.push(format!("{}: any", name));
                        }
                    }
                    if obj.members.len() > 10 {
                        parts.push("...".to_string());
                    }
                    format!("{{ {} }}", parts.join("; "))
                }
            }
            Type::TypeReference(_) => "TypeReference".to_string(),
            Type::TypeParameter(_) => {
                if let Some(name) = self.type_parameter_names.get(&type_id) {
                    name.clone()
                } else if let Type::TypeParameter(tp) = typ {
                    if let Some(sym) = self.symbol_arena.get(tp.symbol) {
                        sym.escaped_name.clone()
                    } else {
                        "T".to_string()
                    }
                } else {
                    "T".to_string()
                }
            }
            Type::Conditional(c) => {
                format!(
                    "{} extends {} ? {} : {}",
                    self.type_to_string(c.check_type),
                    self.type_to_string(c.extends_type),
                    self.type_to_string(c.true_type),
                    self.type_to_string(c.false_type),
                )
            }
            Type::Mapped(m) => {
                let constraint_str = self.type_to_string(m.constraint_type);
                let template_str = self.type_to_string(m.template_type);
                format!("{{ [K in {}]: {} }}", constraint_str, template_str)
            }
            Type::IndexedAccess(_) => "IndexedAccessType".to_string(),
            Type::Index(_) => "IndexType".to_string(),
            Type::TemplateLiteral(tl) => {
                let mut result = String::from("`");
                for (i, text) in tl.texts.iter().enumerate() {
                    result.push_str(text);
                    if i < tl.types.len() {
                        result.push_str("${");
                        result.push_str(&self.type_to_string(tl.types[i]));
                        result.push('}');
                    }
                }
                result.push('`');
                result
            }
            Type::Function(f) => {
                let type_params_str = if !f.type_parameters.is_empty() {
                    let tp_strs: Vec<String> = f.type_parameters.iter()
                        .map(|&tp| self.type_to_string(tp))
                        .collect();
                    format!("<{}>", tp_strs.join(", "))
                } else {
                    String::new()
                };
                let params: Vec<String> = f.parameter_names.iter()
                    .zip(f.parameter_types.iter())
                    .map(|(name, &typ)| {
                        if name.is_empty() {
                            self.type_to_string(typ)
                        } else {
                            format!("{}: {}", name, self.type_to_string(typ))
                        }
                    })
                    .collect();
                let return_str = self.type_to_string(f.return_type);
                format!("{}({}) => {}", type_params_str, params.join(", "), return_str)
            }
            Type::Array(arr) => {
                let elem_str = self.type_to_string(arr.element_type);
                if arr.is_readonly {
                    format!("readonly {}[]", elem_str)
                } else {
                    format!("{}[]", elem_str)
                }
            }
            Type::Tuple(tup) => {
                let elements: Vec<String> = tup.element_types.iter()
                    .enumerate()
                    .map(|(i, &t)| {
                        let type_str = self.type_to_string(t);
                        // Check if this element has a name
                        if let Some(ref names) = tup.element_names {
                            if let Some(Some(name)) = names.get(i) {
                                return format!("{}: {}", name, type_str);
                            }
                        }
                        type_str
                    })
                    .collect();
                if tup.is_readonly {
                    format!("readonly [{}]", elements.join(", "))
                } else {
                    format!("[{}]", elements.join(", "))
                }
            }
            Type::Enum(e) => {
                format!("typeof {}", e.name)
            }
            Type::ThisType(t) => {
                let constraint_str = self.type_to_string(t.constraint);
                format!("ThisType<{}>", constraint_str)
            }
            Type::UniqueSymbol(s) => {
                format!("typeof {}", s.name)
            }
        }
    }

    /// Check a source file and populate diagnostics.
    /// This is the entry point for type checking a parsed and bound file.
    pub fn check_source_file(&mut self, root_idx: crate::parser::NodeIndex) {
        use crate::parser::Node;

        let Some(node) = self.node_arena.get(root_idx) else {
            return;
        };

        if let Node::SourceFile(sf) = node {
            // Type check each top-level statement
            for &stmt_idx in &sf.statements.nodes {
                self.check_statement(stmt_idx);
            }
        }
    }

    /// Check a statement and produce type errors.
    fn check_statement(&mut self, stmt_idx: crate::parser::NodeIndex) {
        use crate::parser::Node;

        let Some(node) = self.node_arena.get(stmt_idx) else {
            return;
        };

        match node {
            Node::VariableStatement(vs) => {
                self.check_variable_statement(stmt_idx, vs);
            }
            Node::ExpressionStatement(es) => {
                // Type-check the expression (may produce diagnostics)
                self.get_type_of_node(es.expression);
            }
            Node::IfStatement(ifs) => {
                // Check condition
                self.get_type_of_node(ifs.expression);
                // Check then branch
                self.check_statement(ifs.then_statement);
                // Check else branch if present
                if !ifs.else_statement.is_none() {
                    self.check_statement(ifs.else_statement);
                }
            }
            Node::ReturnStatement(rs) => {
                if !rs.expression.is_none() {
                    self.get_type_of_node(rs.expression);
                }
            }
            Node::Block(block) => {
                for &inner_stmt in &block.statements.nodes {
                    self.check_statement(inner_stmt);
                }
            }
            Node::FunctionDeclaration(fd) => {
                // Check function body if present
                if !fd.body.is_none() {
                    // Push a new scope for function parameters
                    self.push_local_scope();

                    // Add parameters to local scope
                    for &param_idx in &fd.parameters.nodes {
                        if let Some(crate::parser::Node::ParameterDeclaration(param)) = self.node_arena.get(param_idx) {
                            if let Some(crate::parser::Node::Identifier(id)) = self.node_arena.get(param.name) {
                                let param_type = if !param.type_annotation.is_none() {
                                    self.get_type_of_node(param.type_annotation)
                                } else {
                                    self.types.any_type
                                };
                                self.add_local(id.escaped_text.clone(), param_type);
                            }
                        }
                    }

                    self.check_statement(fd.body);
                    self.pop_local_scope();
                }
            }
            Node::ClassDeclaration(cd) => {
                // Save and set enclosing class for visibility checks
                let prev_enclosing_class = self.enclosing_class;
                self.enclosing_class = Some(stmt_idx);

                // Check if class is abstract
                let is_abstract_class = self.has_modifier(&cd.modifiers, crate::scanner::SyntaxKind::AbstractKeyword);

                // Get base class symbol for override checking
                let base_class_symbol = self.get_base_class_symbol(&cd.heritage_clauses);

                // Check class members
                for &member_idx in &cd.members.nodes {
                    self.check_class_member(member_idx);

                    // Verify abstract member is not in non-abstract class
                    if !is_abstract_class {
                        if self.is_abstract_member(member_idx) {
                            self.error(
                                member_idx,
                                "Abstract methods can only appear within an abstract class.",
                                super::diagnostic_codes::ABSTRACT_MEMBER_IN_NON_ABSTRACT_CLASS
                            );
                        }
                    }

                    // Check override modifier
                    if self.has_override_modifier(member_idx) {
                        if let Some(member_name) = self.get_member_name(member_idx) {
                            // If member has override modifier, there must be a base class with that member
                            if let Some(base_symbol) = base_class_symbol {
                                if !self.base_class_has_member(base_symbol, &member_name) {
                                    self.error(
                                        member_idx,
                                        &format!("This member cannot have an 'override' modifier because it is not declared in the base class."),
                                        super::diagnostic_codes::OVERRIDE_MEMBER_NOT_IN_BASE
                                    );
                                }
                            } else {
                                // No base class, so override is invalid
                                self.error(
                                    member_idx,
                                    "This member cannot have an 'override' modifier because the class does not extend any class.",
                                    super::diagnostic_codes::OVERRIDE_MEMBER_NOT_IN_BASE
                                );
                            }
                        }
                    }
                }

                // Restore enclosing class (handles nested classes)
                self.enclosing_class = prev_enclosing_class;
            }
            Node::WhileStatement(ws) => {
                self.get_type_of_node(ws.expression);
                self.check_statement(ws.statement);
            }
            Node::DoStatement(ds) => {
                self.check_statement(ds.statement);
                self.get_type_of_node(ds.expression);
            }
            Node::ForStatement(fs) => {
                if !fs.initializer.is_none() {
                    self.get_type_of_node(fs.initializer);
                }
                if !fs.condition.is_none() {
                    self.get_type_of_node(fs.condition);
                }
                if !fs.incrementor.is_none() {
                    self.get_type_of_node(fs.incrementor);
                }
                self.check_statement(fs.statement);
            }
            Node::ForInStatement(fis) => {
                self.get_type_of_node(fis.initializer);
                self.get_type_of_node(fis.expression);
                self.check_statement(fis.statement);
            }
            Node::ForOfStatement(fos) => {
                self.get_type_of_node(fos.initializer);
                self.get_type_of_node(fos.expression);
                self.check_statement(fos.statement);
            }
            Node::SwitchStatement(ss) => {
                self.get_type_of_node(ss.expression);
                // Get the CaseBlock node to access its clauses
                if let Some(Node::CaseBlock(cb)) = self.node_arena.get(ss.case_block) {
                    for &clause_idx in &cb.clauses.nodes {
                        if let Some(Node::CaseClause(cc)) = self.node_arena.get(clause_idx) {
                            self.get_type_of_node(cc.expression);
                            for &stmt in &cc.statements.nodes {
                                self.check_statement(stmt);
                            }
                        } else if let Some(Node::DefaultClause(dc)) = self.node_arena.get(clause_idx) {
                            for &stmt in &dc.statements.nodes {
                                self.check_statement(stmt);
                            }
                        }
                    }
                }
            }
            Node::TryStatement(ts) => {
                self.check_statement(ts.try_block);
                if !ts.catch_clause.is_none() {
                    if let Some(Node::CatchClause(cc)) = self.node_arena.get(ts.catch_clause) {
                        self.check_statement(cc.block);
                    }
                }
                if !ts.finally_block.is_none() {
                    self.check_statement(ts.finally_block);
                }
            }
            Node::ThrowStatement(ts) => {
                self.get_type_of_node(ts.expression);
            }
            // Type declarations - just register them, no expression checking needed
            Node::InterfaceDeclaration(_) |
            Node::TypeAliasDeclaration(_) |
            Node::EnumDeclaration(_) |
            Node::ImportDeclaration(_) |
            Node::ExportDeclaration(_) |
            Node::ExportAssignment(_) |
            Node::ModuleDeclaration(_) => {
                // Type declarations don't need expression checking
            }
            _ => {
                // For other nodes, try to get their type (might produce diagnostics)
                self.get_type_of_node(stmt_idx);
            }
        }
    }

    /// Check a variable statement.
    fn check_variable_statement(&mut self, _stmt_idx: crate::parser::NodeIndex, vs: &crate::parser::VariableStatement) {
        // Get the VariableDeclarationList node
        let Some(crate::parser::Node::VariableDeclarationList(vdl)) = self.node_arena.get(vs.declaration_list) else {
            return;
        };

        // Check each variable declaration
        for &decl_idx in &vdl.declarations.nodes {
            if let Some(crate::parser::Node::VariableDeclaration(vd)) = self.node_arena.get(decl_idx) {
                // If there's an initializer, check type compatibility
                if !vd.initializer.is_none() {
                    // If there's a type annotation, use it as contextual type for the initializer
                    let (init_type, declared_type_opt) = if !vd.type_annotation.is_none() {
                        let declared_type = self.get_type_of_node(vd.type_annotation);
                        // Set contextual type to enable proper inference (e.g., array literals)
                        let prev_contextual = self.contextual_type;
                        self.contextual_type = Some(declared_type);
                        let init_type = self.get_type_of_node(vd.initializer);
                        self.contextual_type = prev_contextual;
                        (init_type, Some(declared_type))
                    } else {
                        (self.get_type_of_node(vd.initializer), None)
                    };

                    // If there's a type annotation, check assignability
                    if let Some(declared_type) = declared_type_opt {
                        if !self.is_type_assignable_to(init_type, declared_type) {
                            let init_str = self.type_to_string(init_type);
                            let decl_str = self.type_to_string(declared_type);
                            self.error(
                                vd.initializer,
                                &format!("Type '{}' is not assignable to type '{}'.", init_str, decl_str),
                                2322,
                            );
                        }
                    }
                }
            }
        }
    }

    /// Check a class member.
    fn check_class_member(&mut self, member_idx: crate::parser::NodeIndex) {
        use crate::parser::Node;

        let Some(node) = self.node_arena.get(member_idx) else {
            return;
        };

        match node {
            Node::MethodDeclaration(md) => {
                if !md.body.is_none() {
                    // Push a new scope for method parameters
                    self.push_local_scope();

                    // Add parameters to local scope
                    for &param_idx in &md.parameters.nodes {
                        if let Some(crate::parser::Node::ParameterDeclaration(param)) = self.node_arena.get(param_idx) {
                            if let Some(crate::parser::Node::Identifier(id)) = self.node_arena.get(param.name) {
                                let param_type = if !param.type_annotation.is_none() {
                                    self.get_type_of_node(param.type_annotation)
                                } else {
                                    self.types.any_type
                                };
                                self.add_local(id.escaped_text.clone(), param_type);
                            }
                        }
                    }

                    self.check_statement(md.body);
                    self.pop_local_scope();
                }
            }
            Node::PropertyDeclaration(pd) => {
                if !pd.initializer.is_none() {
                    self.get_type_of_node(pd.initializer);
                }
            }
            Node::ConstructorDeclaration(cd) => {
                if !cd.body.is_none() {
                    // Push a new scope for constructor parameters
                    self.push_local_scope();

                    // Add parameters to local scope
                    for &param_idx in &cd.parameters.nodes {
                        if let Some(crate::parser::Node::ParameterDeclaration(param)) = self.node_arena.get(param_idx) {
                            if let Some(crate::parser::Node::Identifier(id)) = self.node_arena.get(param.name) {
                                let param_type = if !param.type_annotation.is_none() {
                                    self.get_type_of_node(param.type_annotation)
                                } else {
                                    self.types.any_type
                                };
                                self.add_local(id.escaped_text.clone(), param_type);
                            }
                        }
                    }
                    self.check_statement(cd.body);
                    self.pop_local_scope();
                }
            }
            Node::GetAccessorDeclaration(gd) => {
                if !gd.body.is_none() {
                    self.check_statement(gd.body);
                }
            }
            Node::SetAccessorDeclaration(sd) => {
                if !sd.body.is_none() {
                    // Push a new scope for setter parameter
                    self.push_local_scope();

                    // Add the setter parameter to local scope
                    for &param_idx in &sd.parameters.nodes {
                        if let Some(crate::parser::Node::ParameterDeclaration(param)) = self.node_arena.get(param_idx) {
                            if let Some(crate::parser::Node::Identifier(id)) = self.node_arena.get(param.name) {
                                let param_type = if !param.type_annotation.is_none() {
                                    self.get_type_of_node(param.type_annotation)
                                } else {
                                    self.types.any_type
                                };
                                self.add_local(id.escaped_text.clone(), param_type);
                            }
                        }
                    }

                    self.check_statement(sd.body);
                    self.pop_local_scope();
                }
            }
            _ => {}
        }
    }
}
