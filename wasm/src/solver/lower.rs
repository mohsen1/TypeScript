//! Type lowering: AST nodes → TypeId
//!
//! This module implements the "bridge" that converts raw AST nodes (ThinNode)
//! into the structural type system (TypeId).
//!
//! Lowering is lazy - types are only computed when queried.

use crate::parser::thin_node::{ThinNodeArena, SignatureData, IndexSignatureData};
use crate::parser::base::NodeIndex;
use crate::parser::NodeList;
use crate::scanner::SyntaxKind;
use crate::parser::syntax_kind_ext;
use crate::solver::types::*;
use crate::solver::TypeDatabase;
use crate::interner::Atom;
use std::cell::RefCell;
use rustc_hash::FxHashMap;

#[cfg(test)]
use crate::solver::TypeInterner;

/// Type lowering context.
/// Converts AST type nodes into interned TypeIds.
pub struct TypeLowering<'a> {
    arena: &'a ThinNodeArena,
    interner: &'a dyn TypeDatabase,
    /// Optional symbol resolver - resolves identifier nodes to SymbolIds.
    /// If provided, this enables correct abstract class detection.
    resolver: Option<&'a dyn Fn(NodeIndex) -> Option<u32>>,
    type_param_scopes: RefCell<Vec<Vec<(Atom, TypeId)>>>,
}

struct InterfaceParts {
    properties: FxHashMap<Atom, PropertyMerge>,
    call_signatures: Vec<CallSignature>,
    construct_signatures: Vec<CallSignature>,
    string_index: Option<IndexSignature>,
    number_index: Option<IndexSignature>,
}

enum PropertyMerge {
    Property(PropertyInfo),
    Method(MethodOverloads),
    Conflict(PropertyInfo),
}

struct MethodOverloads {
    signatures: Vec<CallSignature>,
    optional: bool,
}

impl InterfaceParts {
    fn new() -> Self {
        InterfaceParts {
            properties: FxHashMap::default(),
            call_signatures: Vec::new(),
            construct_signatures: Vec::new(),
            string_index: None,
            number_index: None,
        }
    }

    fn merge_property(&mut self, prop: PropertyInfo) {
        use std::collections::hash_map::Entry;

        match self.properties.entry(prop.name) {
            Entry::Vacant(entry) => {
                entry.insert(PropertyMerge::Property(prop));
            }
            Entry::Occupied(mut entry) => {
                match entry.get_mut() {
                    PropertyMerge::Property(existing) => {
                        if existing.type_id == prop.type_id
                            && existing.optional == prop.optional
                            && existing.readonly == prop.readonly
                            && existing.is_method == prop.is_method
                        {
                            return;
                        }
                        let conflict = PropertyInfo {
                            name: prop.name,
                            type_id: TypeId::ERROR,
                            optional: existing.optional && prop.optional,
                            readonly: existing.readonly && prop.readonly,
                            is_method: false,
                        };
                        entry.insert(PropertyMerge::Conflict(conflict));
                    }
                    PropertyMerge::Method(methods) => {
                        let conflict = PropertyInfo {
                            name: prop.name,
                            type_id: TypeId::ERROR,
                            optional: methods.optional && prop.optional,
                            readonly: false,
                            is_method: false,
                        };
                        entry.insert(PropertyMerge::Conflict(conflict));
                    }
                    PropertyMerge::Conflict(_) => {}
                }
            }
        }
    }

    fn merge_method(&mut self, name: Atom, signature: CallSignature, optional: bool) {
        use std::collections::hash_map::Entry;

        match self.properties.entry(name) {
            Entry::Vacant(entry) => {
                entry.insert(PropertyMerge::Method(MethodOverloads {
                    signatures: vec![signature],
                    optional,
                }));
            }
            Entry::Occupied(mut entry) => {
                match entry.get_mut() {
                    PropertyMerge::Method(methods) => {
                        methods.signatures.push(signature);
                        methods.optional |= optional;
                    }
                    PropertyMerge::Property(prop) => {
                        let conflict = PropertyInfo {
                            name,
                            type_id: TypeId::ERROR,
                            optional: prop.optional && optional,
                            readonly: false,
                            is_method: false,
                        };
                        entry.insert(PropertyMerge::Conflict(conflict));
                    }
                    PropertyMerge::Conflict(_) => {}
                }
            }
        }
    }

    fn merge_index_signature(&mut self, index: IndexSignature) {
        let target = if index.key_type == TypeId::NUMBER {
            &mut self.number_index
        } else {
            &mut self.string_index
        };

        if let Some(existing) = target.as_mut() {
            if existing.value_type != index.value_type || existing.readonly != index.readonly {
                existing.value_type = TypeId::ERROR;
                existing.readonly = false;
            }
        } else {
            *target = Some(index);
        }
    }
}

impl<'a> TypeLowering<'a> {
    pub fn new(arena: &'a ThinNodeArena, interner: &'a dyn TypeDatabase) -> Self {
        TypeLowering {
            arena,
            interner,
            resolver: None,
            type_param_scopes: RefCell::new(Vec::new()),
        }
    }

    /// Create a TypeLowering with a symbol resolver.
    /// The resolver converts identifier names to actual SymbolIds from the binder.
    pub fn with_resolver(
        arena: &'a ThinNodeArena,
        interner: &'a dyn TypeDatabase,
        resolver: &'a dyn Fn(NodeIndex) -> Option<u32>,
    ) -> Self {
        TypeLowering {
            arena,
            interner,
            resolver: Some(resolver),
            type_param_scopes: RefCell::new(Vec::new()),
        }
    }

    /// Resolve a node to a symbol ID if a resolver is provided.
    fn resolve_symbol(&self, node_idx: NodeIndex) -> Option<u32> {
        self.resolver.and_then(|resolver| resolver(node_idx))
    }

    fn push_type_param_scope(&self) {
        self.type_param_scopes.borrow_mut().push(Vec::new());
    }

    fn pop_type_param_scope(&self) {
        let _ = self.type_param_scopes.borrow_mut().pop();
    }

    fn add_type_param_binding(&self, name: Atom, type_id: TypeId) {
        if let Some(scope) = self.type_param_scopes.borrow_mut().last_mut() {
            scope.push((name, type_id));
        }
    }

    fn lookup_type_param(&self, name: &str) -> Option<TypeId> {
        let atom = self.interner.intern_string(name);
        let scopes = self.type_param_scopes.borrow();
        for scope in scopes.iter().rev() {
            for (scope_name, type_id) in scope.iter().rev() {
                if *scope_name == atom {
                    return Some(*type_id);
                }
            }
        }
        None
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
            // Qualified name (A.B)
            // =========================================================================
            k if k == syntax_kind_ext::QUALIFIED_NAME => {
                self.lower_qualified_name_type(node_idx)
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
            _ => TypeId::ANY,
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
                .map(|&idx| self.lower_tuple_element(idx))
                .collect();
            self.interner.tuple(elements)
        } else {
            self.interner.tuple(vec![])
        }
    }

    /// Lower a tuple element, preserving name, optional, and rest metadata.
    fn lower_tuple_element(&self, node_idx: NodeIndex) -> TupleElement {
        let Some(node) = self.arena.get(node_idx) else {
            return TupleElement {
                type_id: TypeId::ERROR,
                name: None,
                optional: false,
                rest: false,
            };
        };

        if node.kind == syntax_kind_ext::NAMED_TUPLE_MEMBER {
            if let Some(data) = self.arena.get_named_tuple_member(node) {
                let name = if let Some(name_node) = self.arena.get(data.name) {
                    if let Some(id_data) = self.arena.get_identifier(name_node) {
                        Some(self.interner.intern_string(&id_data.escaped_text))
                    } else {
                        None
                    }
                } else {
                    None
                };

                return TupleElement {
                    type_id: self.lower_type(data.type_node),
                    name,
                    optional: data.question_token,
                    rest: data.dot_dot_dot_token,
                };
            }
        }

        if node.kind == syntax_kind_ext::REST_TYPE || node.kind == syntax_kind_ext::OPTIONAL_TYPE {
            let wrapped = if let Some(data) = self.arena.get_wrapped_type(node) {
                Some(data.type_node)
            } else {
                self.arena.type_operators.get(node.data_index as usize)
                    .map(|data| data.type_node)
            };

            return TupleElement {
                type_id: wrapped.map_or_else(|| self.lower_type(node_idx), |inner| self.lower_type(inner)),
                name: None,
                optional: node.kind == syntax_kind_ext::OPTIONAL_TYPE,
                rest: node.kind == syntax_kind_ext::REST_TYPE,
            };
        }

        TupleElement {
            type_id: self.lower_type(node_idx),
            name: None,
            optional: false,
            rest: false,
        }
    }

    fn with_type_params<R>(
        &self,
        type_params: &Option<NodeList>,
        f: impl FnOnce() -> R,
    ) -> (Vec<TypeParamInfo>, R) {
        let Some(list) = type_params else {
            return (Vec::new(), f());
        };

        if list.nodes.is_empty() {
            return (Vec::new(), f());
        }

        self.push_type_param_scope();
        let params = self.collect_type_parameters(list);
        let result = f();
        self.pop_type_param_scope();

        (params, result)
    }

    fn collect_type_parameters(&self, list: &NodeList) -> Vec<TypeParamInfo> {
        let mut params = Vec::with_capacity(list.nodes.len());
        for &idx in &list.nodes {
            if let Some(info) = self.lower_type_parameter(idx) {
                let type_id = self.interner.intern(TypeKey::TypeParameter(info.clone()));
                self.add_type_param_binding(info.name, type_id);
                params.push(info);
            }
        }
        params
    }

    fn lower_type_parameter(&self, node_idx: NodeIndex) -> Option<TypeParamInfo> {
        let node = self.arena.get(node_idx)?;
        let data = self.arena.get_type_parameter(node)?;

        let name = self
            .arena
            .get(data.name)
            .and_then(|name_node| self.arena.get_identifier(name_node))
            .map(|id_data| self.interner.intern_string(&id_data.escaped_text))
            .unwrap_or_else(|| self.interner.intern_string("T"));

        let constraint = if data.constraint != NodeIndex::NONE {
            Some(self.lower_type(data.constraint))
        } else {
            None
        };

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
    }

    /// Extract a parameter name if it is an identifier.
    fn lower_parameter_name(&self, node_idx: NodeIndex) -> Option<crate::interner::Atom> {
        let node = self.arena.get(node_idx)?;
        self.arena
            .get_identifier(node)
            .map(|ident| self.interner.intern_string(&ident.escaped_text))
    }

    /// Lower a function type ((a: T, b: U) => R)
    fn lower_function_type(&self, node_idx: NodeIndex) -> TypeId {
        let node = match self.arena.get(node_idx) {
            Some(n) => n,
            None => return TypeId::ERROR,
        };

        if let Some(data) = self.arena.get_function_type(node) {
            let (type_params, (params, return_type)) = self.with_type_params(&data.type_parameters, || {
                let params: Vec<ParamInfo> = data.parameters.nodes.iter()
                    .filter_map(|&idx| {
                        if let Some(param_node) = self.arena.get(idx) {
                            if let Some(param_data) = self.arena.get_parameter(param_node) {
                                return Some(ParamInfo {
                                    name: self.lower_parameter_name(param_data.name),
                                    type_id: self.lower_type(param_data.type_annotation),
                                    optional: param_data.question_token,
                                    rest: param_data.dot_dot_dot_token,
                                });
                            }
                        }
                        None
                    })
                    .collect();

                let return_type = self.lower_type(data.type_annotation);
                (params, return_type)
            });

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
            let mut properties = Vec::new();
            let mut call_signatures = Vec::new();
            let mut construct_signatures = Vec::new();
            let mut string_index = None;
            let mut number_index = None;

            for &idx in &data.members.nodes {
                let Some(member) = self.arena.get(idx) else { continue };

                if let Some(sig) = self.arena.get_signature(member) {
                    match member.kind {
                        k if k == syntax_kind_ext::CALL_SIGNATURE => {
                            call_signatures.push(self.lower_call_signature(sig));
                        }
                        k if k == syntax_kind_ext::CONSTRUCT_SIGNATURE => {
                            construct_signatures.push(self.lower_call_signature(sig));
                        }
                        k if k == syntax_kind_ext::METHOD_SIGNATURE => {
                            if let Some(name) = self.lower_signature_name(sig.name) {
                                let type_id = self.lower_method_signature(sig);
                                properties.push(PropertyInfo {
                                    name,
                                    type_id,
                                    optional: sig.question_token,
                                    readonly: self.has_readonly_modifier(&sig.modifiers),
                                    is_method: true,
                                });
                            }
                        }
                        _ => {
                            if let Some(prop) = self.lower_type_element(idx) {
                                properties.push(prop);
                            }
                        }
                    }
                    continue;
                }

                if let Some(index_sig) = self.arena.get_index_signature(member) {
                    if let Some(index_info) = self.lower_index_signature(index_sig) {
                        if index_info.key_type == TypeId::NUMBER {
                            number_index = Some(index_info);
                        } else {
                            string_index = Some(index_info);
                        }
                    }
                }
            }

            if !call_signatures.is_empty() || !construct_signatures.is_empty() {
                return self.interner.callable(CallableShape {
                    call_signatures,
                    construct_signatures,
                    properties,
                });
            }

            if string_index.is_some() || number_index.is_some() {
                return self.interner.object_with_index(ObjectShape {
                    properties,
                    string_index,
                    number_index,
                });
            }

            self.interner.object(properties)
        } else {
            self.interner.object(vec![])
        }
    }

    pub fn lower_interface_declarations(&self, declarations: &[NodeIndex]) -> TypeId {
        if declarations.is_empty() {
            return TypeId::ERROR;
        }

        let mut parts = InterfaceParts::new();
        let mut type_params: Option<&NodeList> = None;
        let mut found = false;

        for &decl_idx in declarations {
            let Some(node) = self.arena.get(decl_idx) else { continue };
            let Some(interface) = self.arena.get_interface(node) else { continue };
            found = true;
            if type_params.is_none() {
                type_params = interface.type_parameters.as_ref();
            }
        }

        if !found {
            return TypeId::ERROR;
        }

        if let Some(params) = type_params {
            self.push_type_param_scope();
            let _ = self.collect_type_parameters(params);
        }

        for &decl_idx in declarations {
            let Some(node) = self.arena.get(decl_idx) else { continue };
            let Some(interface) = self.arena.get_interface(node) else { continue };
            self.collect_interface_members(&interface.members, &mut parts);
        }

        if type_params.is_some() {
            self.pop_type_param_scope();
        }

        self.finish_interface_parts(parts)
    }

    fn collect_interface_members(&self, members: &NodeList, parts: &mut InterfaceParts) {
        for &idx in &members.nodes {
            let Some(member) = self.arena.get(idx) else { continue };

            if let Some(sig) = self.arena.get_signature(member) {
                match member.kind {
                    k if k == syntax_kind_ext::CALL_SIGNATURE => {
                        parts.call_signatures.push(self.lower_call_signature(sig));
                    }
                    k if k == syntax_kind_ext::CONSTRUCT_SIGNATURE => {
                        parts.construct_signatures.push(self.lower_call_signature(sig));
                    }
                    k if k == syntax_kind_ext::METHOD_SIGNATURE => {
                        if let Some(name) = self.lower_signature_name(sig.name) {
                            let signature = self.lower_call_signature(sig);
                            parts.merge_method(name, signature, sig.question_token);
                        }
                    }
                    _ => {
                        if let Some(prop) = self.lower_type_element(idx) {
                            parts.merge_property(prop);
                        }
                    }
                }
                continue;
            }

            if let Some(index_sig) = self.arena.get_index_signature(member) {
                if let Some(index_info) = self.lower_index_signature(index_sig) {
                    parts.merge_index_signature(index_info);
                }
            }
        }
    }

    fn finish_interface_parts(&self, parts: InterfaceParts) -> TypeId {
        let mut properties = Vec::with_capacity(parts.properties.len());
        for (name, entry) in parts.properties {
            match entry {
                PropertyMerge::Property(prop) => properties.push(prop),
                PropertyMerge::Method(methods) => {
                    let type_id = self.interner.callable(CallableShape {
                        call_signatures: methods.signatures,
                        construct_signatures: Vec::new(),
                        properties: Vec::new(),
                    });
                    properties.push(PropertyInfo {
                        name,
                        type_id,
                        optional: methods.optional,
                        readonly: false,
                        is_method: true,
                    });
                }
                PropertyMerge::Conflict(prop) => properties.push(prop),
            }
        }

        if !parts.call_signatures.is_empty() || !parts.construct_signatures.is_empty() {
            return self.interner.callable(CallableShape {
                call_signatures: parts.call_signatures,
                construct_signatures: parts.construct_signatures,
                properties,
            });
        }

        if parts.string_index.is_some() || parts.number_index.is_some() {
            return self.interner.object_with_index(ObjectShape {
                properties,
                string_index: parts.string_index,
                number_index: parts.number_index,
            });
        }

        self.interner.object(properties)
    }

    fn lower_call_signature(&self, sig: &SignatureData) -> CallSignature {
        let (type_params, (params, return_type)) = self.with_type_params(&sig.type_parameters, || {
            let params = self.lower_signature_params(sig);
            let return_type = self.lower_type(sig.type_annotation);
            (params, return_type)
        });

        CallSignature {
            type_params,
            params,
            return_type,
        }
    }

    fn lower_method_signature(&self, sig: &SignatureData) -> TypeId {
        let (type_params, (params, return_type)) = self.with_type_params(&sig.type_parameters, || {
            let params = self.lower_signature_params(sig);
            let return_type = self.lower_type(sig.type_annotation);
            (params, return_type)
        });

        self.interner.function(FunctionShape {
            type_params,
            params,
            return_type,
            is_constructor: false,
        })
    }

    fn lower_signature_params(&self, sig: &SignatureData) -> Vec<ParamInfo> {
        let Some(params) = &sig.parameters else { return Vec::new() };
        params.nodes.iter().filter_map(|&idx| {
            let param_node = self.arena.get(idx)?;
            let param_data = self.arena.get_parameter(param_node)?;
            Some(ParamInfo {
                name: self.lower_parameter_name(param_data.name),
                type_id: self.lower_type(param_data.type_annotation),
                optional: param_data.question_token,
                rest: param_data.dot_dot_dot_token,
            })
        }).collect()
    }

    fn lower_signature_name(&self, node_idx: NodeIndex) -> Option<Atom> {
        let node = self.arena.get(node_idx)?;
        if let Some(id_data) = self.arena.get_identifier(node) {
            return Some(self.interner.intern_string(&id_data.escaped_text));
        }
        if let Some(lit_data) = self.arena.get_literal(node) {
            if !lit_data.text.is_empty() {
                return Some(self.interner.intern_string(&lit_data.text));
            }
        }
        None
    }

    fn lower_index_signature(&self, sig: &IndexSignatureData) -> Option<IndexSignature> {
        let param_idx = sig.parameters.nodes.first().copied().unwrap_or(NodeIndex::NONE);
        let param_node = self.arena.get(param_idx)?;
        let param_data = self.arena.get_parameter(param_node)?;
        let key_type = self.lower_type(param_data.type_annotation);
        let value_type = self.lower_type(sig.type_annotation);
        let readonly = self.has_readonly_modifier(&sig.modifiers);

        Some(IndexSignature {
            key_type,
            value_type,
            readonly,
        })
    }

    /// Lower a type element (property signature, method signature, etc.)
    fn lower_type_element(&self, node_idx: NodeIndex) -> Option<PropertyInfo> {
        let node = self.arena.get(node_idx)?;

        // Check if it's a property or method signature
        if let Some(sig) = self.arena.get_signature(node) {
            // Get property name as Arc<str>
            let name = self.lower_signature_name(sig.name)?;

            // Check for readonly modifier
            let readonly = self.has_readonly_modifier(&sig.modifiers);

            Some(PropertyInfo {
                name,
                type_id: self.lower_type(sig.type_annotation),
                optional: sig.question_token,
                readonly,
                is_method: false,
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
            let (type_param, constraint) = self.lower_mapped_type_param(data.type_parameter);
            self.push_type_param_scope();
            let type_param_id = self.interner.intern(TypeKey::TypeParameter(type_param.clone()));
            self.add_type_param_binding(type_param.name, type_param_id);
            let template = self.lower_type(data.type_node);
            self.pop_type_param_scope();
            let mapped = MappedType {
                type_param,
                constraint,
                template,
                readonly_modifier: self.lower_mapped_modifier(data.readonly_token, SyntaxKind::ReadonlyKeyword as u16),
                optional_modifier: self.lower_mapped_modifier(data.question_token, SyntaxKind::QuestionToken as u16),
            };
            self.interner.intern(TypeKey::Mapped(Box::new(mapped)))
        } else {
            TypeId::ERROR
        }
    }

    fn lower_mapped_type_param(&self, node_idx: NodeIndex) -> (TypeParamInfo, TypeId) {
        let node = match self.arena.get(node_idx) {
            Some(n) => n,
            None => {
                let name = self.interner.intern_string("K");
                return (TypeParamInfo { name, constraint: None, default: None }, TypeId::ANY);
            }
        };

        if let Some(param_data) = self.arena.get_type_parameter(node) {
            let name = self
                .arena
                .get(param_data.name)
                .and_then(|ident_node| self.arena.get_identifier(ident_node))
                .map(|ident| self.interner.intern_string(&ident.escaped_text))
                .unwrap_or_else(|| self.interner.intern_string("K"));

            let constraint = if param_data.constraint != NodeIndex::NONE {
                Some(self.lower_type(param_data.constraint))
            } else {
                None
            };

            let default = if param_data.default != NodeIndex::NONE {
                Some(self.lower_type(param_data.default))
            } else {
                None
            };

            let constraint_type = constraint.unwrap_or(TypeId::ANY);

            (
                TypeParamInfo {
                    name,
                    constraint,
                    default,
                },
                constraint_type,
            )
        } else {
            let name = self.interner.intern_string("K");
            (TypeParamInfo { name, constraint: None, default: None }, TypeId::ANY)
        }
    }

    fn lower_mapped_modifier(&self, token_idx: NodeIndex, default_kind: u16) -> Option<MappedModifier> {
        use crate::scanner::SyntaxKind;

        if token_idx == NodeIndex::NONE {
            return None;
        }

        let kind = self.arena.get(token_idx).map(|node| node.kind)?;
        if kind == SyntaxKind::PlusToken as u16 || kind == default_kind {
            Some(MappedModifier::Add)
        } else if kind == SyntaxKind::MinusToken as u16 {
            Some(MappedModifier::Remove)
        } else {
            None
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
                    k if k == SyntaxKind::BigIntLiteral as u16 => {
                        if let Some(lit_data) = self.arena.get_literal(literal_node) {
                            let text = lit_data.text.strip_suffix('n').unwrap_or(&lit_data.text);
                            self.interner.literal_bigint(text)
                        } else {
                            TypeId::BIGINT
                        }
                    }
                    k if k == SyntaxKind::TrueKeyword as u16 => {
                        self.interner.literal_boolean(true)
                    }
                    k if k == SyntaxKind::FalseKeyword as u16 => {
                        self.interner.literal_boolean(false)
                    }
                    k if k == syntax_kind_ext::PREFIX_UNARY_EXPRESSION => {
                        if let Some(unary) = self.arena.get_unary_expr(literal_node) {
                            let op = unary.operator;
                            let Some(operand_node) = self.arena.get(unary.operand) else {
                                return TypeId::ANY;
                            };
                            match operand_node.kind {
                                k if k == SyntaxKind::NumericLiteral as u16 => {
                                    if let Some(lit_data) = self.arena.get_literal(operand_node) {
                                        if let Ok(n) = lit_data.text.parse::<f64>() {
                                            let value = if op == SyntaxKind::MinusToken as u16 { -n } else { n };
                                            self.interner.literal_number(value)
                                        } else {
                                            TypeId::NUMBER
                                        }
                                    } else {
                                        TypeId::NUMBER
                                    }
                                }
                                k if k == SyntaxKind::BigIntLiteral as u16 => {
                                    if let Some(lit_data) = self.arena.get_literal(operand_node) {
                                        let text = lit_data.text.strip_suffix('n').unwrap_or(&lit_data.text);
                                        if op == SyntaxKind::MinusToken as u16 {
                                            let mut value = String::with_capacity(text.len() + 1);
                                            value.push('-');
                                            value.push_str(text);
                                            self.interner.literal_bigint(&value)
                                        } else {
                                            self.interner.literal_bigint(text)
                                        }
                                    } else {
                                        TypeId::BIGINT
                                    }
                                }
                                _ => TypeId::ANY,
                            }
                        } else {
                            TypeId::ANY
                        }
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
            let base_type = self.lower_type(data.type_name);
            if let Some(args) = &data.type_arguments {
                if !args.nodes.is_empty() {
                    let type_args: Vec<TypeId> = args.nodes.iter()
                        .map(|&idx| self.lower_type(idx))
                        .collect();
                    return self.interner.application(base_type, type_args);
                }
            }
            base_type
        } else {
            TypeId::ERROR
        }
    }

    /// Lower a qualified name type (A.B).
    fn lower_qualified_name_type(&self, node_idx: NodeIndex) -> TypeId {
        if let Some(symbol_id) = self.resolve_symbol(node_idx) {
            return self.interner.reference(SymbolRef(symbol_id));
        }
        TypeId::ERROR
    }

    /// Lower an identifier as a type (simple type reference)
    fn lower_identifier_type(&self, node_idx: NodeIndex) -> TypeId {
        let node = match self.arena.get(node_idx) {
            Some(n) => n,
            None => return TypeId::ERROR,
        };

        if let Some(data) = self.arena.get_identifier(node) {
            let name = &data.escaped_text;

            if let Some(type_param) = self.lookup_type_param(name) {
                return type_param;
            }

            if let Some(symbol_id) = self.resolve_symbol(node_idx) {
                return self.interner.reference(SymbolRef(symbol_id));
            }

            // Check for built-in type names only if not resolved (shadowing-safe)
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

            TypeId::ERROR
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
            if let Some(symbol_id) = self.resolve_symbol(data.expr_name) {
                return self.interner.intern(TypeKey::TypeQuery(SymbolRef(symbol_id)));
            }
            TypeId::ERROR
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
                if let Some(span_node) = self.arena.get(span_idx) {
                    if span_node.kind == syntax_kind_ext::TEMPLATE_LITERAL_TYPE_SPAN {
                        if let Some(span_data) = self.arena.template_spans.get(span_node.data_index as usize) {
                            let type_id = self.lower_type(span_data.expression);
                            spans.push(TemplateSpan::Type(type_id));

                            if let Some(lit_node) = self.arena.get(span_data.literal) {
                                if let Some(lit_data) = self.arena.get_literal(lit_node) {
                                    if !lit_data.text.is_empty() {
                                        spans.push(TemplateSpan::Text(self.interner.intern_string(&lit_data.text)));
                                    }
                                }
                            }
                        }
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
            let (type_params, (params, return_type)) = self.with_type_params(&data.type_parameters, || {
                let params: Vec<ParamInfo> = data.parameters.nodes.iter()
                    .filter_map(|&idx| {
                        if let Some(param_node) = self.arena.get(idx) {
                            if let Some(param_data) = self.arena.get_parameter(param_node) {
                                return Some(ParamInfo {
                                    name: self.lower_parameter_name(param_data.name),
                                    type_id: self.lower_type(param_data.type_annotation),
                                    optional: param_data.question_token,
                                    rest: param_data.dot_dot_dot_token,
                                });
                            }
                        }
                        None
                    })
                    .collect();

                let return_type = self.lower_type(data.type_annotation);
                (params, return_type)
            });

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
            return self.lower_type(data.type_node);
        }

        if let Some(data) = self.arena.type_operators.get(node.data_index as usize) {
            return self.lower_type(data.type_node);
        }

        TypeId::ERROR
    }
}

#[cfg(test)]
#[path = "lower_tests.rs"]
mod tests;
