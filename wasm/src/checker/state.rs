//! Checker state and core infrastructure.
//!
//! This module contains the CheckerState struct, diagnostic types,
//! and type guard/relation definitions.

use std::collections::HashMap;
use rustc_hash::FxHashMap;
use serde::Serialize;
use crate::binder::{SymbolId, SymbolArena, SymbolTable};
use crate::parser::NodeIndex;
use super::arena::TypeArena;
use super::types::TypeId;

// =============================================================================
// Diagnostic
// =============================================================================

/// A type-checking diagnostic message.
#[derive(Clone, Debug, Serialize)]
pub struct Diagnostic {
    pub file: String,
    pub start: u32,
    pub length: u32,
    pub message_text: String,
    pub category: DiagnosticCategory,
    pub code: u32,
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

    /// Stack of nodes being resolved (to detect circular references).
    pub(crate) node_resolution_stack: Vec<NodeIndex>,

    /// Current file name.
    pub file_name: String,

    /// Contextual type for expression being checked.
    pub(crate) contextual_type: Option<TypeId>,

    /// Cache for type relation results (source, target, relation) -> result.
    /// Key: (source TypeId, target TypeId, relation as u8)
    /// Value: true if related, false otherwise
    pub(crate) relation_cache: FxHashMap<(TypeId, TypeId, u8), bool>,
}

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
            node_resolution_stack: Vec::new(),
            file_name,
            contextual_type: None,
            relation_cache: FxHashMap::default(),
        }
    }

    /// Resolve a name to a symbol.
    pub fn resolve_name(&self, name: &str) -> Option<SymbolId> {
        self.file_locals.get(name)
    }

    /// Report a diagnostic error.
    pub fn error(&mut self, node: NodeIndex, message: &str, code: u32) {
        if let Some(n) = self.node_arena.get(node) {
            let base = n.base();
            self.diagnostics.push(Diagnostic {
                file: self.file_name.clone(),
                start: base.pos,
                length: base.end - base.pos,
                message_text: message.to_string(),
                category: DiagnosticCategory::Error,
                code,
            });
        }
    }

    /// Get the diagnostics as JSON.
    pub fn get_diagnostics_json(&self) -> String {
        serde_json::to_string(&self.diagnostics).unwrap_or_else(|_| "[]".to_string())
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
                if obj.members.is_empty() && obj.properties.is_empty() {
                    "object".to_string()
                } else {
                    let mut parts = Vec::new();
                    for (name, &symbol_id) in obj.members.iter() {
                        if let Some(&prop_type) = self.symbol_types.get(&symbol_id) {
                            parts.push(format!("{}: {}", name, self.type_to_string(prop_type)));
                        } else {
                            parts.push(format!("{}: any", name));
                        }
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
                    .map(|&t| self.type_to_string(t))
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
        }
    }
}
