//! Type lowering: AST nodes → TypeId
//!
//! This module implements the "bridge" that converts raw AST nodes (ThinNode)
//! into the structural type system (TypeId).
//!
//! Lowering is lazy - types are only computed when queried.

use std::sync::Arc;
use crate::parser::thin_node::ThinNodeArena;
use crate::parser::base::NodeIndex;
use crate::parser::NodeList;
use crate::scanner::SyntaxKind;
use crate::parser::syntax_kind_ext;
use crate::solver::types::*;
use crate::solver::intern::TypeInterner;

/// Type lowering context.
/// Converts AST type nodes into interned TypeIds.
pub struct TypeLowering<'a> {
    arena: &'a ThinNodeArena,
    interner: &'a TypeInterner,
    /// Optional symbol resolver - resolves names to SymbolIds.
    /// If provided, this enables correct abstract class detection.
    resolver: Option<&'a dyn Fn(&str) -> Option<u32>>,
}

impl<'a> TypeLowering<'a> {
    pub fn new(arena: &'a ThinNodeArena, interner: &'a TypeInterner) -> Self {
        TypeLowering { arena, interner, resolver: None }
    }

    /// Create a TypeLowering with a symbol resolver.
    /// The resolver converts identifier names to actual SymbolIds from the binder.
    pub fn with_resolver(
        arena: &'a ThinNodeArena,
        interner: &'a TypeInterner,
        resolver: &'a dyn Fn(&str) -> Option<u32>,
    ) -> Self {
        TypeLowering { arena, interner, resolver: Some(resolver) }
    }

    /// Resolve a name to a symbol ID, falling back to hashing if no resolver provided.
    fn resolve_symbol(&self, name: &str) -> u32 {
        if let Some(resolver) = self.resolver {
            if let Some(id) = resolver(name) {
                return id;
            }
        }

        // Fallback to hash
        use std::hash::{Hash, Hasher};
        use std::collections::hash_map::DefaultHasher;
        let mut hasher = DefaultHasher::new();
        name.hash(&mut hasher);
        hasher.finish() as u32
    }

    /// Lower a type node to a TypeId.
    /// This is the main entry point for type synthesis.
    pub fn lower_type(&self, node_idx: NodeIndex) -> TypeId {
        if node_idx == NodeIndex::NONE {
            return TypeId::ANY; // Implicit any for missing type annotations
        }

        let node = match self.arena.get(node_idx) {
            Some(n) => n,
            None => return TypeId::ERROR,
        };

        match node.kind {
            // =========================================================================
            // Keyword types
            // =========================================================================
            k if k == SyntaxKind::AnyKeyword as u16 => TypeId::ANY,
            k if k == SyntaxKind::UnknownKeyword as u16 => TypeId::UNKNOWN,
            k if k == SyntaxKind::NeverKeyword as u16 => TypeId::NEVER,
            k if k == SyntaxKind::VoidKeyword as u16 => TypeId::VOID,
            k if k == SyntaxKind::UndefinedKeyword as u16 => TypeId::UNDEFINED,
            k if k == SyntaxKind::NullKeyword as u16 => TypeId::NULL,
            k if k == SyntaxKind::BooleanKeyword as u16 => TypeId::BOOLEAN,
            k if k == SyntaxKind::NumberKeyword as u16 => TypeId::NUMBER,
            k if k == SyntaxKind::StringKeyword as u16 => TypeId::STRING,
            k if k == SyntaxKind::BigIntKeyword as u16 => TypeId::BIGINT,
            k if k == SyntaxKind::SymbolKeyword as u16 => TypeId::SYMBOL,
            k if k == SyntaxKind::ObjectKeyword as u16 => TypeId::OBJECT,

            // =========================================================================
            // Literal types (true, false)
            // =========================================================================
            k if k == SyntaxKind::TrueKeyword as u16 => {
                self.interner.literal_boolean(true)
            }
            k if k == SyntaxKind::FalseKeyword as u16 => {
                self.interner.literal_boolean(false)
            }

            // =========================================================================
            // Composite types
            // =========================================================================
            k if k == syntax_kind_ext::UNION_TYPE => {
                self.lower_union_type(node_idx)
            }
            k if k == syntax_kind_ext::INTERSECTION_TYPE => {
                self.lower_intersection_type(node_idx)
            }

            // =========================================================================
            // Array and tuple types
            // =========================================================================
            k if k == syntax_kind_ext::ARRAY_TYPE => {
                self.lower_array_type(node_idx)
            }
            k if k == syntax_kind_ext::TUPLE_TYPE => {
                self.lower_tuple_type(node_idx)
            }

            // =========================================================================
            // Function type
            // =========================================================================
            k if k == syntax_kind_ext::FUNCTION_TYPE => {
                self.lower_function_type(node_idx)
            }

            // =========================================================================
            // Type literal (object type)
            // =========================================================================
            k if k == syntax_kind_ext::TYPE_LITERAL => {
                eprintln!("[DEBUG lower_type] Matched TYPE_LITERAL for kind={}", k);
                self.lower_type_literal(node_idx)
            }

            // =========================================================================
            // Conditional type
            // =========================================================================
            k if k == syntax_kind_ext::CONDITIONAL_TYPE => {
                self.lower_conditional_type(node_idx)
            }

            // =========================================================================
            // Mapped type
            // =========================================================================
            k if k == syntax_kind_ext::MAPPED_TYPE => {
                self.lower_mapped_type(node_idx)
            }

            // =========================================================================
            // Indexed access type
            // =========================================================================
            k if k == syntax_kind_ext::INDEXED_ACCESS_TYPE => {
                self.lower_indexed_access_type(node_idx)
            }

            // =========================================================================
            // Literal type (string literal, number literal in type position)
            // =========================================================================
            k if k == syntax_kind_ext::LITERAL_TYPE => {
                self.lower_literal_type(node_idx)
            }

            // =========================================================================
            // Type reference (NamedType or NamedType<Args>)
            // =========================================================================
            k if k == syntax_kind_ext::TYPE_REFERENCE => {
                self.lower_type_reference(node_idx)
            }

            // =========================================================================
            // Identifier (simple type reference without type arguments)
            // =========================================================================
            k if k == SyntaxKind::Identifier as u16 => {
                self.lower_identifier_type(node_idx)
            }

            // =========================================================================
            // This type
            // =========================================================================
            k if k == SyntaxKind::ThisKeyword as u16 => {
                self.interner.intern(TypeKey::ThisType)
            }
            k if k == syntax_kind_ext::THIS_TYPE => {
                self.interner.intern(TypeKey::ThisType)
            }

            // =========================================================================
            // Parenthesized type
            // =========================================================================
            k if k == syntax_kind_ext::PARENTHESIZED_TYPE => {
                self.lower_parenthesized_type(node_idx)
            }

            // =========================================================================
            // Type query (typeof in type position)
            // =========================================================================
            k if k == syntax_kind_ext::TYPE_QUERY => {
                self.lower_type_query(node_idx)
            }

            // =========================================================================
            // Type operator (keyof, readonly, unique)
            // =========================================================================
            k if k == syntax_kind_ext::TYPE_OPERATOR => {
                self.lower_type_operator(node_idx)
            }

            // =========================================================================
            // Infer type (infer R)
            // =========================================================================
            k if k == syntax_kind_ext::INFER_TYPE => {
                self.lower_infer_type(node_idx)
            }

            // =========================================================================
            // Template literal type
            // =========================================================================
            k if k == syntax_kind_ext::TEMPLATE_LITERAL_TYPE => {
                self.lower_template_literal_type(node_idx)
            }

            // =========================================================================
            // Named tuple member
            // =========================================================================
            k if k == syntax_kind_ext::NAMED_TUPLE_MEMBER => {
                self.lower_named_tuple_member(node_idx)
            }

            // =========================================================================
            // Constructor type (new () => T)
            // =========================================================================
            k if k == syntax_kind_ext::CONSTRUCTOR_TYPE => {
                self.lower_constructor_type(node_idx)
            }

            // =========================================================================
            // Optional/Rest types (unwrap)
            // =========================================================================
            k if k == syntax_kind_ext::OPTIONAL_TYPE || k == syntax_kind_ext::REST_TYPE => {
                self.lower_wrapped_type(node_idx)
            }

            // =========================================================================
            // Unknown/unsupported - return ANY for now
            // =========================================================================
            _ => {
                // Debug: print unhandled kind
                eprintln!("[DEBUG lower_type] Unhandled kind: {} (TYPE_LITERAL={})", node.kind, syntax_kind_ext::TYPE_LITERAL);
                TypeId::ANY
            }
        }
    }

    /// Lower a union type (A | B | C)
    fn lower_union_type(&self, node_idx: NodeIndex) -> TypeId {
        let node = match self.arena.get(node_idx) {
            Some(n) => n,
            None => return TypeId::ERROR,
        };

        if let Some(data) = self.arena.get_composite_type(node) {
            let members: Vec<TypeId> = data.types.nodes.iter()
                .map(|&idx| self.lower_type(idx))
                .collect();
            self.interner.union(members)
        } else {
            TypeId::ERROR
        }
    }

    /// Lower an intersection type (A & B & C)
    fn lower_intersection_type(&self, node_idx: NodeIndex) -> TypeId {
        let node = match self.arena.get(node_idx) {
            Some(n) => n,
            None => return TypeId::ERROR,
        };

        if let Some(data) = self.arena.get_composite_type(node) {
            let members: Vec<TypeId> = data.types.nodes.iter()
                .map(|&idx| self.lower_type(idx))
                .collect();
            self.interner.intersection(members)
        } else {
            TypeId::ERROR
        }
    }

    /// Lower an array type (T[])
    fn lower_array_type(&self, node_idx: NodeIndex) -> TypeId {
        let node = match self.arena.get(node_idx) {
            Some(n) => n,
            None => return TypeId::ERROR,
        };

        if let Some(data) = self.arena.get_array_type(node) {
            let element_type = self.lower_type(data.element_type);
            self.interner.array(element_type)
        } else {
            self.interner.array(TypeId::ANY)
        }
    }

    /// Lower a tuple type ([A, B, C])
    fn lower_tuple_type(&self, node_idx: NodeIndex) -> TypeId {
        let node = match self.arena.get(node_idx) {
            Some(n) => n,
            None => return TypeId::ERROR,
        };

        if let Some(data) = self.arena.get_tuple_type(node) {
            let elements: Vec<TupleElement> = data.elements.nodes.iter()
                .map(|&idx| {
                    TupleElement {
                        type_id: self.lower_type(idx),
                        name: None, // TODO: Support named tuple elements
                        optional: false, // TODO: Check for optional marker
                        rest: false, // TODO: Check for rest element
                    }
                })
                .collect();
            self.interner.tuple(elements)
        } else {
            self.interner.tuple(vec![])
        }
    }

    /// Lower type parameters from a NodeList.
    /// Returns a Vec<TypeParamInfo> for use in FunctionShape.
    fn lower_type_parameters(&self, type_params: &Option<NodeList>) -> Vec<TypeParamInfo> {
        match type_params {
            None => vec![],
            Some(list) => {
                list.nodes.iter()
                    .filter_map(|&idx| {
                        let node = self.arena.get(idx)?;
                        let data = self.arena.get_type_parameter(node)?;

                        // Get the name from the identifier node
                        let name = if data.name != NodeIndex::NONE {
                            if let Some(name_node) = self.arena.get(data.name) {
                                if let Some(id_data) = self.arena.get_identifier(name_node) {
                                    self.interner.intern_string(&id_data.escaped_text)
                                } else {
                                    return None;
                                }
                            } else {
                                return None;
                            }
                        } else {
                            return None;
                        };

                        // Lower constraint if present (e.g., T extends SomeType)
                        let constraint = if data.constraint != NodeIndex::NONE {
                            Some(self.lower_type(data.constraint))
                        } else {
                            None
                        };

                        // Lower default if present (e.g., T = DefaultType)
                        let default = if data.default != NodeIndex::NONE {
                            Some(self.lower_type(data.default))
                        } else {
                            None
                        };

                        Some(TypeParamInfo {
                            name,
                            constraint,
                            default,
                        })
                    })
                    .collect()
            }
        }
    }

    /// Lower a function type ((a: T, b: U) => R)
    fn lower_function_type(&self, node_idx: NodeIndex) -> TypeId {
        let node = match self.arena.get(node_idx) {
            Some(n) => n,
            None => return TypeId::ERROR,
        };

        if let Some(data) = self.arena.get_function_type(node) {
            // Lower parameters
            let params: Vec<ParamInfo> = data.parameters.nodes.iter()
                .filter_map(|&idx| {
                    if let Some(param_node) = self.arena.get(idx) {
                        if let Some(param_data) = self.arena.get_parameter(param_node) {
                            return Some(ParamInfo {
                                name: None, // TODO: Extract parameter name
                                type_id: self.lower_type(param_data.type_annotation),
                                optional: param_data.question_token,
                                rest: param_data.dot_dot_dot_token,
                            });
                        }
                    }
                    None
                })
                .collect();

            // Lower return type
            let return_type = self.lower_type(data.type_annotation);

            // Lower type parameters
            let type_params = self.lower_type_parameters(&data.type_parameters);

            let shape = FunctionShape {
                type_params,
                params,
                return_type,
                is_constructor: false,
            };

            self.interner.function(shape)
        } else {
            TypeId::ERROR
        }
    }

    /// Lower a type literal ({ x: T, y: U })
    fn lower_type_literal(&self, node_idx: NodeIndex) -> TypeId {
        let node = match self.arena.get(node_idx) {
            Some(n) => n,
            None => return TypeId::ERROR,
        };

        if let Some(data) = self.arena.get_type_literal(node) {
            let properties: Vec<PropertyInfo> = data.members.nodes.iter()
                .filter_map(|&idx| self.lower_type_element(idx))
                .collect();
            self.interner.object(properties)
        } else {
            self.interner.object(vec![])
        }
    }

    /// Lower a type element (property signature, method signature, etc.)
    fn lower_type_element(&self, node_idx: NodeIndex) -> Option<PropertyInfo> {
        let node = self.arena.get(node_idx)?;

        // Check if it's a property or method signature
        if let Some(sig) = self.arena.get_signature(node) {
            // Get property name as Arc<str>
            let name = if sig.name != NodeIndex::NONE {
                if let Some(name_node) = self.arena.get(sig.name) {
                    if let Some(id_data) = self.arena.get_identifier(name_node) {
                        self.interner.intern_string(&id_data.escaped_text)
                    } else {
                        return None;
                    }
                } else {
                    return None;
                }
            } else {
                return None;
            };

            // Check for readonly modifier
            let readonly = self.has_readonly_modifier(&sig.modifiers);

            Some(PropertyInfo {
                name,
                type_id: self.lower_type(sig.type_annotation),
                optional: sig.question_token,
                readonly,
            })
        } else {
            None
        }
    }

    /// Check if a modifiers list contains a readonly keyword
    fn has_readonly_modifier(&self, modifiers: &Option<NodeList>) -> bool {
        use crate::scanner::SyntaxKind;

        if let Some(mods) = modifiers {
            for &mod_idx in &mods.nodes {
                if let Some(mod_node) = self.arena.get(mod_idx) {
                    if mod_node.kind == SyntaxKind::ReadonlyKeyword as u16 {
                        return true;
                    }
                }
            }
        }
        false
    }

    /// Lower a conditional type (T extends U ? X : Y)
    fn lower_conditional_type(&self, node_idx: NodeIndex) -> TypeId {
        let node = match self.arena.get(node_idx) {
            Some(n) => n,
            None => return TypeId::ERROR,
        };

        if let Some(data) = self.arena.get_conditional_type(node) {
            let cond = ConditionalType {
                check_type: self.lower_type(data.check_type),
                extends_type: self.lower_type(data.extends_type),
                true_type: self.lower_type(data.true_type),
                false_type: self.lower_type(data.false_type),
            };
            self.interner.intern(TypeKey::Conditional(Box::new(cond)))
        } else {
            TypeId::ERROR
        }
    }

    /// Lower a mapped type ({ [K in Keys]: ValueType })
    fn lower_mapped_type(&self, node_idx: NodeIndex) -> TypeId {
        let node = match self.arena.get(node_idx) {
            Some(n) => n,
            None => return TypeId::ERROR,
        };

        if let Some(data) = self.arena.get_mapped_type(node) {
            // For mapped types, we need to extract the type parameter
            // The type_parameter field is the NodeIndex of the type parameter declaration
            let param_name = self.interner.intern_string("K"); // Default name

            let mapped = MappedType {
                type_param: TypeParamInfo {
                    name: param_name,
                    constraint: None, // TODO: Extract constraint from type parameter
                    default: None,
                },
                constraint: TypeId::ANY, // TODO: Extract from name_type
                template: self.lower_type(data.type_node),
                readonly_modifier: None, // TODO: Handle readonly modifier
                optional_modifier: None, // TODO: Handle optional modifier
            };
            self.interner.intern(TypeKey::Mapped(Box::new(mapped)))
        } else {
            TypeId::ERROR
        }
    }

    /// Lower an indexed access type (T[K])
    fn lower_indexed_access_type(&self, node_idx: NodeIndex) -> TypeId {
        let node = match self.arena.get(node_idx) {
            Some(n) => n,
            None => return TypeId::ERROR,
        };

        if let Some(data) = self.arena.get_indexed_access_type(node) {
            let object_type = self.lower_type(data.object_type);
            let index_type = self.lower_type(data.index_type);
            self.interner.intern(TypeKey::IndexAccess(object_type, index_type))
        } else {
            TypeId::ERROR
        }
    }

    /// Lower a literal type ("foo", 42, etc.)
    fn lower_literal_type(&self, node_idx: NodeIndex) -> TypeId {
        let node = match self.arena.get(node_idx) {
            Some(n) => n,
            None => return TypeId::ERROR,
        };

        if let Some(data) = self.arena.get_literal_type(node) {
            // The literal node contains the actual literal value
            if let Some(literal_node) = self.arena.get(data.literal) {
                match literal_node.kind {
                    k if k == SyntaxKind::StringLiteral as u16 => {
                        if let Some(lit_data) = self.arena.get_literal(literal_node) {
                            self.interner.literal_string(&lit_data.text)
                        } else {
                            TypeId::STRING
                        }
                    }
                    k if k == SyntaxKind::NumericLiteral as u16 => {
                        if let Some(lit_data) = self.arena.get_literal(literal_node) {
                            if let Ok(n) = lit_data.text.parse::<f64>() {
                                self.interner.literal_number(n)
                            } else {
                                TypeId::NUMBER
                            }
                        } else {
                            TypeId::NUMBER
                        }
                    }
                    k if k == SyntaxKind::TrueKeyword as u16 => {
                        self.interner.literal_boolean(true)
                    }
                    k if k == SyntaxKind::FalseKeyword as u16 => {
                        self.interner.literal_boolean(false)
                    }
                    _ => TypeId::ANY,
                }
            } else {
                TypeId::ANY
            }
        } else {
            TypeId::ANY
        }
    }

    /// Lower a type reference (NamedType or NamedType<Args>)
    fn lower_type_reference(&self, node_idx: NodeIndex) -> TypeId {
        let node = match self.arena.get(node_idx) {
            Some(n) => n,
            None => return TypeId::ERROR,
        };

        if let Some(data) = self.arena.get_type_ref(node) {
            // For now, just lower the type name as an identifier
            // TODO: Handle type arguments
            self.lower_type(data.type_name)
        } else {
            TypeId::ERROR
        }
    }

    /// Lower an identifier as a type (simple type reference)
    fn lower_identifier_type(&self, node_idx: NodeIndex) -> TypeId {
        let node = match self.arena.get(node_idx) {
            Some(n) => n,
            None => return TypeId::ERROR,
        };

        if let Some(data) = self.arena.get_identifier(node) {
            let name = &data.escaped_text;

            // Check for built-in type names
            match name.as_ref() {
                "any" => return TypeId::ANY,
                "unknown" => return TypeId::UNKNOWN,
                "never" => return TypeId::NEVER,
                "void" => return TypeId::VOID,
                "undefined" => return TypeId::UNDEFINED,
                "null" => return TypeId::NULL,
                "boolean" => return TypeId::BOOLEAN,
                "number" => return TypeId::NUMBER,
                "string" => return TypeId::STRING,
                "bigint" => return TypeId::BIGINT,
                "symbol" => return TypeId::SYMBOL,
                "object" => return TypeId::OBJECT,
                _ => {}
            }

            // Create a reference for named types
            // Use resolver if available, otherwise fall back to hash
            let symbol_id = self.resolve_symbol(name);
            self.interner.reference(SymbolRef(symbol_id))
        } else {
            TypeId::ERROR
        }
    }

    /// Lower a parenthesized type
    fn lower_parenthesized_type(&self, node_idx: NodeIndex) -> TypeId {
        let node = match self.arena.get(node_idx) {
            Some(n) => n,
            None => return TypeId::ERROR,
        };

        // Parenthesized types just wrap another type
        if let Some(data) = self.arena.get_wrapped_type(node) {
            self.lower_type(data.type_node)
        } else {
            TypeId::ERROR
        }
    }

    /// Lower a type query (typeof expr in type position)
    fn lower_type_query(&self, node_idx: NodeIndex) -> TypeId {
        let node = match self.arena.get(node_idx) {
            Some(n) => n,
            None => return TypeId::ERROR,
        };

        if let Some(data) = self.arena.get_type_query(node) {
            // Create a symbol reference from the expression name
            // Use resolver if available for correct symbol ID lookup
            if let Some(expr_node) = self.arena.get(data.expr_name) {
                if let Some(id_data) = self.arena.get_identifier(expr_node) {
                    let symbol_id = self.resolve_symbol(&id_data.escaped_text);
                    return self.interner.intern(TypeKey::TypeQuery(SymbolRef(symbol_id)));
                }
            }
            TypeId::ANY
        } else {
            TypeId::ERROR
        }
    }

    /// Lower a type operator (keyof, readonly, unique)
    fn lower_type_operator(&self, node_idx: NodeIndex) -> TypeId {
        let node = match self.arena.get(node_idx) {
            Some(n) => n,
            None => return TypeId::ERROR,
        };

        if let Some(data) = self.arena.get_type_operator(node) {
            let inner_type = self.lower_type(data.type_node);

            // Check which operator it is
            match data.operator {
                // KeyOfKeyword = 143
                143 => self.interner.intern(TypeKey::KeyOf(inner_type)),
                // ReadonlyKeyword = 148
                148 => self.interner.intern(TypeKey::ReadonlyType(inner_type)),
                // UniqueKeyword = 158 - unique symbol
                158 => {
                    // unique symbol creates a unique symbol type
                    // Use node index as unique identifier
                    self.interner.intern(TypeKey::UniqueSymbol(SymbolRef(node_idx.0)))
                }
                _ => inner_type,
            }
        } else {
            TypeId::ERROR
        }
    }

    /// Lower an infer type (infer R)
    fn lower_infer_type(&self, node_idx: NodeIndex) -> TypeId {
        let node = match self.arena.get(node_idx) {
            Some(n) => n,
            None => return TypeId::ERROR,
        };

        if let Some(data) = self.arena.get_infer_type(node) {
            // Get the type parameter name
            let name = if let Some(tp_node) = self.arena.get(data.type_parameter) {
                // Type parameter node should have an identifier
                if let Some(tp_data) = self.arena.get_type_parameter(tp_node) {
                    if let Some(name_node) = self.arena.get(tp_data.name) {
                        if let Some(id_data) = self.arena.get_identifier(name_node) {
                            self.interner.intern_string(&id_data.escaped_text)
                        } else {
                            self.interner.intern_string("infer")
                        }
                    } else {
                        self.interner.intern_string("infer")
                    }
                } else if let Some(id_data) = self.arena.get_identifier(tp_node) {
                    self.interner.intern_string(&id_data.escaped_text)
                } else {
                    self.interner.intern_string("infer")
                }
            } else {
                self.interner.intern_string("infer")
            };

            self.interner.intern(TypeKey::Infer(TypeParamInfo {
                name,
                constraint: None,
                default: None,
            }))
        } else {
            TypeId::ERROR
        }
    }

    /// Lower a template literal type (`hello${T}world`)
    fn lower_template_literal_type(&self, node_idx: NodeIndex) -> TypeId {
        let node = match self.arena.get(node_idx) {
            Some(n) => n,
            None => return TypeId::ERROR,
        };

        if let Some(data) = self.arena.get_template_literal_type(node) {
            let mut spans: Vec<TemplateSpan> = Vec::new();

            // Add the head text if present
            if let Some(head_node) = self.arena.get(data.head) {
                if let Some(head_lit) = self.arena.get_literal(head_node) {
                    if !head_lit.text.is_empty() {
                        spans.push(TemplateSpan::Text(self.interner.intern_string(&head_lit.text)));
                    }
                }
            }

            // Add template spans (type + text pairs)
            for &span_idx in &data.template_spans.nodes {
                if self.arena.get(span_idx).is_some() {
                    // Template span has a type and literal parts
                    // For simplicity, we'll just add the type reference
                    // TODO: Parse template span structure properly
                    let type_id = self.lower_type(span_idx);
                    if type_id != TypeId::ANY && type_id != TypeId::ERROR {
                        spans.push(TemplateSpan::Type(type_id));
                    }
                }
            }

            self.interner.intern(TypeKey::TemplateLiteral(spans))
        } else {
            TypeId::STRING // Fallback to string
        }
    }

    /// Lower a named tuple member ([name: T])
    fn lower_named_tuple_member(&self, node_idx: NodeIndex) -> TypeId {
        let node = match self.arena.get(node_idx) {
            Some(n) => n,
            None => return TypeId::ERROR,
        };

        if let Some(data) = self.arena.get_named_tuple_member(node) {
            // Lower the type part
            self.lower_type(data.type_node)
        } else {
            TypeId::ERROR
        }
    }

    /// Lower a constructor type (new () => T)
    fn lower_constructor_type(&self, node_idx: NodeIndex) -> TypeId {
        let node = match self.arena.get(node_idx) {
            Some(n) => n,
            None => return TypeId::ERROR,
        };

        // Constructor types use the same data structure as function types
        if let Some(data) = self.arena.get_function_type(node) {
            // Lower parameters
            let params: Vec<ParamInfo> = data.parameters.nodes.iter()
                .filter_map(|&idx| {
                    if let Some(param_node) = self.arena.get(idx) {
                        if let Some(param_data) = self.arena.get_parameter(param_node) {
                            return Some(ParamInfo {
                                name: None,
                                type_id: self.lower_type(param_data.type_annotation),
                                optional: param_data.question_token,
                                rest: param_data.dot_dot_dot_token,
                            });
                        }
                    }
                    None
                })
                .collect();

            // Lower return type
            let return_type = self.lower_type(data.type_annotation);

            // Lower type parameters
            let type_params = self.lower_type_parameters(&data.type_parameters);

            let shape = FunctionShape {
                type_params,
                params,
                return_type,
                is_constructor: true, // Mark as constructor
            };

            self.interner.function(shape)
        } else {
            TypeId::ERROR
        }
    }

    /// Lower a wrapped type (optional or rest type)
    fn lower_wrapped_type(&self, node_idx: NodeIndex) -> TypeId {
        let node = match self.arena.get(node_idx) {
            Some(n) => n,
            None => return TypeId::ERROR,
        };

        if let Some(data) = self.arena.get_wrapped_type(node) {
            // Just unwrap and lower the inner type
            // The optional/rest nature is handled at the tuple level
            self.lower_type(data.type_node)
        } else {
            TypeId::ERROR
        }
    }
}

#[cfg(test)]
#[path = "lower_tests.rs"]
mod tests;
