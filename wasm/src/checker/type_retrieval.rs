//! Type retrieval for the type checker.
//!
//! This module contains the core get_type_of_node implementation and
//! type inference logic for all AST node types.

use crate::parser::{Node, NodeIndex};
use crate::scanner::SyntaxKind;
use crate::binder::{SymbolId, SymbolArena, SymbolTable, Symbol, symbol_flags};
use super::types::{
    type_flags, object_flags, signature_flags, diagnostic_codes,
    Type, TypeId, LiteralValue, LiteralType, ObjectType, UnionType, TypeParameter,
    FunctionType, Signature, IndexInfo,
};
use super::arena::TypeArena;
use super::state::{CheckerState, Diagnostic, DiagnosticCategory};

/// Information about the contextual array/tuple element types.
enum ContextualArrayInfo {
    /// Array<T> - single element type for all elements
    Array(TypeId),
    /// Tuple - different type for each position
    Tuple(Vec<TypeId>),
}

/// String mapping kind for intrinsic string manipulation types.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum StringMappingKind {
    Uppercase,
    Lowercase,
    Capitalize,
    Uncapitalize,
}

impl<'a> CheckerState<'a> {
    /// Get the type of a node (with caching).
    pub fn get_type_of_node(&mut self, node: NodeIndex) -> TypeId {
        // Check cache first
        if let Some(&cached) = self.node_types.get(&node) {
            return cached;
        }

        // Track recursion to catch infinite loops (O(1) lookup using HashSet)
        if self.node_resolution_set.contains(&node) {
            // Circular reference - return any_type to break the loop
            return self.types.any_type;
        }
        self.node_resolution_stack.push(node);
        self.node_resolution_set.insert(node);

        let type_id = self.get_type_of_node_worker(node);
        self.node_types.insert(node, type_id);

        self.node_resolution_stack.pop();
        self.node_resolution_set.remove(&node);
        type_id
    }

    /// Get type of node (worker, no caching).
    fn get_type_of_node_worker(&mut self, node: NodeIndex) -> TypeId {
        use crate::parser::Node;
        use crate::scanner::SyntaxKind;

        let Some(n) = self.node_arena.get(node) else {
            return self.types.any_type;
        };

        match n {
            // Literals
            Node::StringLiteral(lit) => {
                self.types.create_string_literal(lit.text.clone())
            }
            Node::NumericLiteral(lit) => {
                let value = lit.text.parse::<f64>().unwrap_or(0.0);
                self.types.create_number_literal(value)
            }
            Node::BigIntLiteral(lit) => {
                // Remove the trailing 'n' from bigint literal (e.g., "123n" -> "123")
                let value = lit.text.trim_end_matches('n').to_string();
                self.types.create_bigint_literal(value)
            }

            // Token nodes - check the kind for keywords
            Node::Token(base) => {
                let kind = base.kind;
                if kind == SyntaxKind::TrueKeyword as u16 {
                    self.types.true_type
                } else if kind == SyntaxKind::FalseKeyword as u16 {
                    self.types.false_type
                } else if kind == SyntaxKind::NullKeyword as u16 {
                    self.types.null_type
                } else if kind == SyntaxKind::StringKeyword as u16 {
                    self.types.string_type
                } else if kind == SyntaxKind::NumberKeyword as u16 {
                    self.types.number_type
                } else if kind == SyntaxKind::BooleanKeyword as u16 {
                    self.types.boolean_type
                } else if kind == SyntaxKind::VoidKeyword as u16 {
                    self.types.void_type
                } else if kind == SyntaxKind::AnyKeyword as u16 {
                    self.types.any_type
                } else if kind == SyntaxKind::NeverKeyword as u16 {
                    self.types.never_type
                } else if kind == SyntaxKind::UndefinedKeyword as u16 {
                    self.types.undefined_type
                } else if kind == SyntaxKind::UnknownKeyword as u16 {
                    self.types.unknown_type
                } else if kind == SyntaxKind::ObjectKeyword as u16 {
                    self.types.object_type
                } else if kind == SyntaxKind::BigIntKeyword as u16 {
                    self.types.big_int_type
                } else if kind == SyntaxKind::SymbolKeyword as u16 {
                    self.types.es_symbol_type
                } else if kind == SyntaxKind::ThisKeyword as u16 {
                    // Get the 'this' type from the enclosing class
                    self.get_this_type()
                } else if kind == SyntaxKind::SuperKeyword as u16 {
                    // Get the superclass type from the enclosing class
                    self.get_super_type()
                } else {
                    self.types.any_type
                }
            }

            // Identifiers - look up in symbol table
            Node::Identifier(id) => {
                let name = id.escaped_text.clone();
                // First check local scopes (function parameters, block-scoped variables)
                if let Some(type_id) = self.lookup_local(&name) {
                    return type_id;
                }
                // Then look up in file-level symbol table
                if let Some(symbol_id) = self.file_locals.get(&name) {
                    self.get_type_of_symbol(symbol_id)
                } else {
                    // Undeclared identifier - report error
                    self.error(
                        node,
                        &format!("Cannot find name '{}'.", name),
                        diagnostic_codes::CANNOT_FIND_NAME
                    );
                    self.types.any_type
                }
            }

            // Union types
            Node::UnionType(ut) => {
                let types: Vec<TypeId> = ut.types.nodes.iter()
                    .map(|&t| self.get_type_of_node(t))
                    .collect();
                self.types.create_union(types)
            }

            // Intersection types
            Node::IntersectionType(it) => {
                let types: Vec<TypeId> = it.types.nodes.iter()
                    .map(|&t| self.get_type_of_node(t))
                    .collect();
                self.types.create_intersection(types)
            }

            // Array types (T[])
            Node::ArrayType(at) => {
                let element_type = self.get_type_of_node(at.element_type);
                self.types.create_array_type(element_type, false)
            }

            // Tuple types ([T, U, V])
            Node::TupleType(tt) => {
                let mut element_types: Vec<TypeId> = Vec::new();
                let mut has_optional_elements = false;
                let mut has_rest_element = false;

                for &elem_idx in &tt.elements.nodes {
                    if let Some(elem_node) = self.node_arena.get(elem_idx) {
                        match elem_node {
                            Node::OptionalType(opt) => {
                                has_optional_elements = true;
                                element_types.push(self.get_type_of_node(opt.type_node));
                            }
                            Node::RestType(rest) => {
                                has_rest_element = true;
                                // For rest element, get the element type of the array
                                let rest_type = self.get_type_of_node(rest.type_node);
                                if let Some(Type::Array(arr)) = self.types.get(rest_type) {
                                    element_types.push(arr.element_type);
                                } else {
                                    // If not an array, just use the type as-is
                                    element_types.push(rest_type);
                                }
                            }
                            _ => {
                                element_types.push(self.get_type_of_node(elem_idx));
                            }
                        }
                    }
                }
                self.types.create_tuple_type(element_types, has_optional_elements, has_rest_element, false)
            }

            // Optional type (T?) - used in tuple elements
            Node::OptionalType(opt) => {
                // For optional types, return the inner type (the optionality is tracked at tuple level)
                self.get_type_of_node(opt.type_node)
            }

            // Rest type (...T) - used in tuple elements
            Node::RestType(rest) => {
                // For rest types, return the element type of the array
                let rest_type = self.get_type_of_node(rest.type_node);
                if let Some(Type::Array(arr)) = self.types.get(rest_type) {
                    arr.element_type
                } else {
                    rest_type
                }
            }

            // Conditional type (T extends U ? X : Y)
            Node::ConditionalType(ct) => {
                self.get_type_of_conditional_type(ct)
            }

            // Template literal type (`hello ${T}`)
            Node::TemplateLiteralType(tlt) => {
                self.get_type_of_template_literal_type(tlt)
            }

            // Mapped type ({ [K in keyof T]: T[K] })
            Node::MappedType(mt) => {
                self.get_type_of_mapped_type(node, mt)
            }

            // Indexed access type (T[K])
            Node::IndexedAccessType(ia) => {
                let object_type = self.get_type_of_node(ia.object_type);
                let index_type = self.get_type_of_node(ia.index_type);
                // Create an IndexedAccess type that will be resolved later during instantiation
                self.types.create_indexed_access_type(object_type, index_type)
            }

            // Infer type (infer T in conditional types)
            Node::InferType(it) => {
                self.get_type_of_infer_type(node, it)
            }

            // Type operators (readonly T, keyof T, etc.)
            Node::TypeOperator(to) => {
                if to.operator == SyntaxKind::ReadonlyKeyword as u16 {
                    // readonly T - make the inner type readonly
                    let inner_type = self.get_type_of_node(to.type_node);
                    self.make_type_readonly(inner_type)
                } else if to.operator == SyntaxKind::KeyOfKeyword as u16 {
                    // keyof T - extract keys from the object type
                    let inner_type = self.get_type_of_node(to.type_node);
                    self.get_keyof_type(inner_type)
                } else {
                    // For other operators (unique), just get the inner type
                    self.get_type_of_node(to.type_node)
                }
            }

            // Type references (generic types like Array<T>, or keywords like number)
            Node::TypeReference(tr) => {
                // Check for type arguments
                if let Some(ref type_args) = tr.type_arguments {
                    if !type_args.nodes.is_empty() {
                        return self.get_type_of_type_reference_with_args(tr.type_name, type_args);
                    }
                }

                // Check if the type_name is a keyword type
                if let Some(Node::Identifier(id)) = self.node_arena.get(tr.type_name) {
                    match id.escaped_text.as_str() {
                        "string" => self.types.string_type,
                        "number" => self.types.number_type,
                        "boolean" => self.types.boolean_type,
                        "void" => self.types.void_type,
                        "any" => self.types.any_type,
                        "never" => self.types.never_type,
                        "undefined" => self.types.undefined_type,
                        "null" => self.types.null_type,
                        "unknown" => self.types.unknown_type,
                        "object" => self.types.object_type,
                        "bigint" => self.types.big_int_type,
                        "symbol" => self.types.es_symbol_type,
                        _ => {
                            // First, check if it's a type parameter in the current scope
                            if let Some(&type_param) = self.type_parameter_scope.get(&id.escaped_text) {
                                type_param
                            }
                            // Otherwise, look up in symbol table
                            else if let Some(symbol_id) = self.file_locals.get(&id.escaped_text) {
                                let base_type = self.get_type_of_symbol(symbol_id);
                                // For classes, a type reference like `let x: MyClass` should resolve
                                // to the instance type, not the constructor type
                                if let Some(Type::Object(obj)) = self.types.get(base_type) {
                                    if obj.has_object_flags(object_flags::CLASS)
                                        && !obj.construct_signatures.is_empty()
                                    {
                                        if let Some(instance_type) =
                                            obj.construct_signatures[0].resolved_return_type
                                        {
                                            return instance_type;
                                        }
                                    }
                                }
                                base_type
                            } else {
                                self.types.object_type
                            }
                        }
                    }
                } else {
                    self.types.object_type
                }
            }

            // Parenthesized types
            Node::ParenthesizedType(pt) => {
                self.get_type_of_node(pt.type_node)
            }

            // TypeQuery - typeof operator in type position (typeof expr)
            Node::TypeQuery(tq) => {
                // Get the type of the expression
                self.get_type_of_node(tq.expr_name)
            }

            // Literal types
            Node::LiteralType(lt) => {
                self.get_type_of_node(lt.literal)
            }

            // Variable declarations - get type from initializer or annotation
            Node::VariableDeclaration(vd) => {
                if !vd.type_annotation.is_none() {
                    self.get_type_of_node(vd.type_annotation)
                } else if !vd.initializer.is_none() {
                    self.get_type_of_node(vd.initializer)
                } else {
                    self.types.any_type
                }
            }

            // Function declarations
            Node::FunctionDeclaration(fd) => {
                self.get_type_of_function_like_with_type_params(
                    node,
                    &fd.parameters,
                    fd.type_annotation,
                    fd.type_parameters.as_ref(),
                )
            }

            // Function expressions
            Node::FunctionExpression(fe) => {
                self.get_type_of_function_like_with_type_params(
                    node,
                    &fe.parameters,
                    fe.type_annotation,
                    fe.type_parameters.as_ref(),
                )
            }

            // Arrow functions
            Node::ArrowFunction(af) => {
                self.get_type_of_function_like_with_type_params(
                    node,
                    &af.parameters,
                    af.type_annotation,
                    af.type_parameters.as_ref(),
                )
            }

            // Method declarations
            Node::MethodDeclaration(md) => {
                self.get_type_of_function_like_with_type_params(
                    node,
                    &md.parameters,
                    md.type_annotation,
                    md.type_parameters.as_ref(),
                )
            }

            // Function type nodes (e.g., type F = (x: number) => string)
            Node::FunctionType(ft) => {
                self.get_type_of_function_like_with_type_params(
                    node,
                    &ft.parameters,
                    ft.type_node,
                    ft.type_parameters.as_ref(),
                )
            }

            // Constructor type nodes
            Node::ConstructorType(ct) => {
                self.get_type_of_function_like_with_type_params(
                    node,
                    &ct.parameters,
                    ct.type_node,
                    ct.type_parameters.as_ref(),
                )
            }

            // Type alias declarations - get the declared type
            Node::TypeAliasDeclaration(ta) => {
                self.get_type_of_type_alias_declaration(ta)
            }

            // Type literals (e.g., { x: number, y: string })
            Node::TypeLiteral(tl) => {
                self.get_type_of_type_literal(&tl.members)
            }

            // Property access expressions (e.g., obj.prop, obj?.prop)
            Node::PropertyAccessExpression(pa) => {
                let prop_type = self.get_type_of_property_access(pa.expression, pa.name);
                // If optional chaining, add undefined to the result type
                if pa.question_dot_token {
                    self.types.create_union(vec![prop_type, self.types.undefined_type])
                } else {
                    prop_type
                }
            }

            // Element access expressions (e.g., obj[key], arr[0], obj?.[key])
            Node::ElementAccessExpression(ea) => {
                let elem_type = self.get_type_of_element_access(ea.expression, ea.argument_expression);
                // If optional chaining, add undefined to the result type
                if ea.question_dot_token {
                    self.types.create_union(vec![elem_type, self.types.undefined_type])
                } else {
                    elem_type
                }
            }

            // Object literals (e.g., { x: 1, y: "hello" })
            Node::ObjectLiteralExpression(ole) => {
                self.get_type_of_object_literal(&ole.properties)
            }

            // Call expressions (e.g., fn(arg1, arg2))
            Node::CallExpression(ce) => {
                self.get_type_of_call_expression(ce.expression, &ce.type_arguments, &ce.arguments)
            }

            // New expressions (e.g., new Foo(arg))
            Node::NewExpression(ne) => {
                self.get_type_of_new_expression(ne.expression, &ne.arguments)
            }

            // Array literal expressions (e.g., [1, 2, 3])
            Node::ArrayLiteralExpression(ale) => {
                self.get_type_of_array_literal(&ale.elements)
            }

            // Parenthesized expressions (e.g., (x))
            Node::ParenthesizedExpression(pe) => {
                self.get_type_of_node(pe.expression)
            }

            // Class declarations
            Node::ClassDeclaration(cd) => {
                self.get_type_of_class_declaration(node, cd)
            }

            // Interface declarations
            Node::InterfaceDeclaration(id) => {
                self.get_type_of_interface_declaration(node, id)
            }

            // Satisfies expression (x satisfies Type)
            // Uses the constraint type as contextual type for the expression
            Node::SatisfiesExpression(se) => {
                let constraint_type = self.get_type_of_node(se.type_node);

                // Set contextual type for proper inference
                let prev_contextual = self.contextual_type;
                self.contextual_type = Some(constraint_type);
                let expression_type = self.get_type_of_node(se.expression);
                self.contextual_type = prev_contextual;

                // Check that expression type is assignable to the constraint
                if !self.is_type_assignable_to(expression_type, constraint_type) {
                    let expr_str = self.type_to_string(expression_type);
                    let constraint_str = self.type_to_string(constraint_type);
                    self.error(
                        node,
                        &format!("Type '{}' does not satisfy the expected type '{}'.", expr_str, constraint_str),
                        diagnostic_codes::TYPE_NOT_ASSIGNABLE_TO_TYPE
                    );
                }

                // Return the original expression type (preserves narrower type)
                expression_type
            }

            // Type assertion (<Type>x)
            Node::TypeAssertion(ta) => {
                let target_type = self.get_type_of_node(ta.type_node);
                target_type
            }

            // As expression (x as Type or x as const)
            Node::AsExpression(ae) => {
                // Check for "as const" - this preserves literal types
                if self.is_const_type_reference(ae.type_node) {
                    // For "as const", return the literal type of the expression
                    // without widening
                    self.get_type_of_node(ae.expression)
                } else {
                    // Regular type assertion - use the target type
                    self.get_type_of_node(ae.type_node)
                }
            }

            // Non-null assertion (x!)
            Node::NonNullExpression(nne) => {
                let base_type = self.get_type_of_node(nne.expression);
                // Remove null and undefined from the type
                self.get_non_nullable_type(base_type)
            }

            // Enum declarations
            Node::EnumDeclaration(ed) => {
                self.get_type_of_enum_declaration(node, ed)
            }

            // Prefix unary expressions (++x, --x, !x, ~x, -x, +x, typeof x, void x, delete x, await x)
            Node::PrefixUnaryExpression(pue) => {
                match pue.operator {
                    // ++x, --x: operand must be numeric, result is number
                    SyntaxKind::PlusPlusToken | SyntaxKind::MinusMinusToken => {
                        let operand_type = self.get_type_of_node(pue.operand);
                        // Check that operand is assignable to number
                        if !self.is_type_assignable_to(operand_type, self.types.number_type) {
                            let operand_str = self.type_to_string(operand_type);
                            self.error(
                                node,
                                &format!("An arithmetic operand must be of type 'any', 'number', 'bigint' or an enum type. Type '{}' is not assignable.", operand_str),
                                diagnostic_codes::TYPE_NOT_ASSIGNABLE_TO_TYPE
                            );
                        }
                        self.types.number_type
                    }
                    // +x, -x: unary plus/minus - result is number
                    SyntaxKind::PlusToken | SyntaxKind::MinusToken => {
                        self.types.number_type
                    }
                    // !x: logical not - result is boolean
                    SyntaxKind::ExclamationToken => {
                        self.types.boolean_type
                    }
                    // ~x: bitwise not - result is number
                    SyntaxKind::TildeToken => {
                        self.types.number_type
                    }
                    // typeof x: result is string (actually a union of literal types)
                    SyntaxKind::TypeOfKeyword => {
                        // TypeScript returns a union of literal string types
                        // For simplicity, we return string type
                        self.types.string_type
                    }
                    // void x: result is undefined
                    SyntaxKind::VoidKeyword => {
                        self.types.undefined_type
                    }
                    // delete x: result is boolean
                    SyntaxKind::DeleteKeyword => {
                        self.types.boolean_type
                    }
                    // await x: unwrap Promise type
                    SyntaxKind::AwaitKeyword => {
                        let operand_type = self.get_type_of_node(pue.operand);
                        // Unwrap Promise<T> to T using get_awaited_type
                        self.get_awaited_type(operand_type)
                    }
                    _ => self.types.any_type
                }
            }

            // Postfix unary expressions (x++, x--)
            Node::PostfixUnaryExpression(pue) => {
                match pue.operator {
                    // x++, x--: operand must be numeric, result is number
                    SyntaxKind::PlusPlusToken | SyntaxKind::MinusMinusToken => {
                        let operand_type = self.get_type_of_node(pue.operand);
                        // Check that operand is assignable to number
                        if !self.is_type_assignable_to(operand_type, self.types.number_type) {
                            let operand_str = self.type_to_string(operand_type);
                            self.error(
                                node,
                                &format!("An arithmetic operand must be of type 'any', 'number', 'bigint' or an enum type. Type '{}' is not assignable.", operand_str),
                                diagnostic_codes::TYPE_NOT_ASSIGNABLE_TO_TYPE
                            );
                        }
                        self.types.number_type
                    }
                    _ => self.types.any_type
                }
            }

            // Binary expressions (a + b, a && b, etc.)
            Node::BinaryExpression(be) => {
                self.get_type_of_binary_expression(be)
            }

            // Conditional expressions (a ? b : c)
            Node::ConditionalExpression(ce) => {
                let when_true_type = self.get_type_of_node(ce.when_true);
                let when_false_type = self.get_type_of_node(ce.when_false);
                // The type is the union of both branches
                self.types.create_union(vec![when_true_type, when_false_type])
            }

            // Template expressions (`hello ${world}`)
            Node::TemplateExpression(_) => {
                // Template expressions always produce string type
                self.types.string_type
            }

            // Tagged template expressions (tag`template`)
            Node::TaggedTemplateExpression(tte) => {
                // The return type is determined by the tag function
                let tag_type = self.get_type_of_node(tte.tag);
                // Try to get the return type of the tag function
                if let Some(crate::checker::types::Type::Function(f)) = self.types.get(tag_type) {
                    f.return_type
                } else {
                    self.types.any_type
                }
            }

            // Yield expressions (yield x, yield* x)
            Node::YieldExpression(ye) => {
                // The type of yield is the yield type from the generator
                // For now, return any if no expression, otherwise the expression type
                if !ye.expression.is_none() {
                    self.get_type_of_node(ye.expression)
                } else {
                    self.types.undefined_type
                }
            }

            // Await expressions (await x)
            Node::AwaitExpression(ae) => {
                let operand_type = self.get_type_of_node(ae.expression);
                // Unwrap Promise<T> to T using get_awaited_type
                self.get_awaited_type(operand_type)
            }

            // Template literal type (TemplateLiteralType is a type node, not expression)
            Node::NoSubstitutionTemplateLiteral(_) => {
                self.types.string_type
            }

            // RegExp literal
            Node::RegularExpressionLiteral(_) => {
                self.types.regexp_type
            }

            // Computed property name: [expression]
            Node::ComputedPropertyName { expression, .. } => {
                // Return the type of the expression (for property key type checking)
                self.get_type_of_node(*expression)
            }

            // Default: return any
            _ => self.types.any_type,
        }
    }

    /// Get the type of an enum declaration.
    fn get_type_of_enum_declaration(
        &mut self,
        _node: NodeIndex,
        enum_decl: &crate::parser::EnumDeclaration
    ) -> TypeId {
        use crate::parser::Node;

        // Get the enum name
        let name = if let Some(Node::Identifier(id)) = self.node_arena.get(enum_decl.name) {
            id.escaped_text.clone()
        } else {
            "unknown".to_string()
        };

        // Collect enum members with their values
        let mut members: Vec<(String, TypeId)> = Vec::new();
        let mut next_value: i64 = 0;

        for &member_idx in &enum_decl.members.nodes {
            if let Some(Node::EnumMember(member)) = self.node_arena.get(member_idx) {
                // Get member name
                let member_name = if let Some(Node::Identifier(id)) = self.node_arena.get(member.name) {
                    id.escaped_text.clone()
                } else {
                    continue;
                };

                // Get or compute member value
                let value_type = if !member.initializer.is_none() {
                    // Has explicit initializer
                    let init_type = self.get_type_of_node(member.initializer);

                    // If it's a numeric literal, update next_value
                    if let Some(Type::Literal(lit)) = self.types.get(init_type) {
                        if let LiteralValue::Number(n) = &lit.value {
                            next_value = *n as i64 + 1;
                        }
                    }

                    init_type
                } else {
                    // Auto-increment numeric value
                    let lit_type = self.types.create_number_literal(next_value as f64);
                    next_value += 1;
                    lit_type
                };

                members.push((member_name, value_type));
            }
        }

        // Create the enum type
        self.types.create_enum_type(name, members)
    }

    /// Get the type of a call expression.
    fn get_type_of_call_expression(
        &mut self,
        expression: NodeIndex,
        type_arguments: &Option<crate::parser::NodeList>,
        arguments: &crate::parser::NodeList
    ) -> TypeId {
        use super::state::MAX_CALL_DEPTH;

        // Check call depth to prevent memory explosion from deeply nested callbacks
        {
            let depth = *self.call_depth.borrow();
            if depth >= MAX_CALL_DEPTH {
                // Return any_type to break potential infinite recursion
                return self.types.any_type;
            }
            *self.call_depth.borrow_mut() = depth + 1;
        }

        // Use a helper to ensure we decrement call_depth on all exit paths
        let result = self.get_type_of_call_expression_inner(expression, type_arguments, arguments);

        // Decrement call depth
        {
            let mut depth = self.call_depth.borrow_mut();
            *depth = depth.saturating_sub(1);
        }

        result
    }

    /// Inner implementation of call expression type resolution.
    fn get_type_of_call_expression_inner(
        &mut self,
        expression: NodeIndex,
        type_arguments: &Option<crate::parser::NodeList>,
        arguments: &crate::parser::NodeList
    ) -> TypeId {
        // Get the type of the function being called
        let func_type = self.get_type_of_node(expression);

        // If it's a function type, handle generics and return type
        if let Some(Type::Function(f)) = self.types.get(func_type) {
            // Extract function information to avoid borrow issues
            let type_parameters = f.type_parameters.clone();
            let parameter_types = f.parameter_types.clone();
            let return_type = f.return_type;
            let min_argument_count = f.min_argument_count;
            let has_rest_parameter = f.has_rest_parameter;

            // Check argument count
            let arg_count = arguments.nodes.len() as u32;
            if arg_count < min_argument_count && !has_rest_parameter {
                self.error(
                    expression,
                    &format!("Expected {} arguments, but got {}.", min_argument_count, arg_count),
                    diagnostic_codes::EXPECTED_ARGUMENTS
                );
            }
            if arg_count > parameter_types.len() as u32 && !has_rest_parameter {
                self.error(
                    expression,
                    &format!("Expected {} arguments, but got {}.", parameter_types.len(), arg_count),
                    diagnostic_codes::EXPECTED_ARGUMENTS
                );
            }

            // If the function has type parameters, we need to infer or use explicit type arguments
            if !type_parameters.is_empty() {
                let inferred_type_args = if let Some(explicit_args) = type_arguments {
                    // Use explicit type arguments: identity<string>("hello")
                    explicit_args.nodes.iter()
                        .map(|&arg| self.get_type_of_node(arg))
                        .collect::<Vec<_>>()
                } else {
                    // Infer type arguments from the argument types
                    self.infer_type_arguments(&type_parameters, &parameter_types, arguments)
                };

                // Instantiate parameter types with the type arguments
                let instantiated_param_types: Vec<TypeId> = parameter_types.iter()
                    .map(|&pt| self.instantiate_type(pt, &inferred_type_args, &type_parameters))
                    .collect();

                // Check argument types against instantiated parameter types
                for (i, &arg_node) in arguments.nodes.iter().enumerate() {
                    if i >= instantiated_param_types.len() {
                        break;
                    }
                    let param_type = instantiated_param_types[i];
                    let arg_type = self.get_type_of_node(arg_node);

                    if !self.is_type_assignable_to(arg_type, param_type) {
                        let arg_str = self.type_to_string(arg_type);
                        let param_str = self.type_to_string(param_type);
                        self.error(
                            arg_node,
                            &format!("Argument of type '{}' is not assignable to parameter of type '{}'.", arg_str, param_str),
                            super::diagnostic_codes::ARGUMENT_NOT_ASSIGNABLE_TO_PARAMETER
                        );
                    }
                }

                // Instantiate the return type with the inferred type arguments
                return self.instantiate_type(return_type, &inferred_type_args, &type_parameters);
            }

            // For non-generic functions, evaluate arguments with contextual types and check type compatibility
            // This enables contextual typing for callbacks like arr.map(x => x + 1)
            for (i, &arg_node) in arguments.nodes.iter().enumerate() {
                if i >= parameter_types.len() {
                    break;
                }
                let param_type = parameter_types[i];
                let prev_contextual_type = self.contextual_type;
                self.contextual_type = Some(param_type);
                let arg_type = self.get_type_of_node(arg_node);
                self.contextual_type = prev_contextual_type;

                // Check if argument type is assignable to parameter type
                if !self.is_type_assignable_to(arg_type, param_type) {
                    let arg_str = self.type_to_string(arg_type);
                    let param_str = self.type_to_string(param_type);
                    self.error(
                        arg_node,
                        &format!("Argument of type '{}' is not assignable to parameter of type '{}'.", arg_str, param_str),
                        super::diagnostic_codes::ARGUMENT_NOT_ASSIGNABLE_TO_PARAMETER
                    );
                }
            }

            return return_type;
        }

        // If it's an object with call signatures, use those (handles overloads)
        if let Some(Type::Object(obj)) = self.types.get(func_type) {
            if !obj.call_signatures.is_empty() {
                let call_sigs = obj.call_signatures.clone();
                return self.resolve_call_with_overloads(expression, &call_sigs, arguments);
            }
        }

        // Default to any for unknown callable types
        self.types.any_type
    }

    /// Resolve a call to a function with multiple overload signatures.
    /// Returns the return type of the first matching overload.
    fn resolve_call_with_overloads(
        &mut self,
        call_node: NodeIndex,
        signatures: &[Signature],
        arguments: &crate::parser::NodeList,
    ) -> TypeId {
        let arg_count = arguments.nodes.len();

        // Collect argument types upfront
        let arg_types: Vec<TypeId> = arguments.nodes.iter()
            .map(|&arg| self.get_type_of_node(arg))
            .collect();

        // Try each overload in order
        for sig in signatures {
            // Check argument count
            let param_count = sig.parameters.len();
            let min_args = sig.min_argument_count as usize;
            let has_rest = (sig.flags & signature_flags::HAS_REST_PARAMETER) != 0;

            // Too few arguments?
            if arg_count < min_args {
                continue;
            }

            // Too many arguments (and no rest)?
            if arg_count > param_count && !has_rest {
                continue;
            }

            // Check each argument against its parameter type
            let mut matches = true;
            for (i, &arg_type) in arg_types.iter().enumerate() {
                if i >= param_count {
                    if !has_rest {
                        matches = false;
                        break;
                    }
                    // Rest parameter - would need to check against rest element type
                    continue;
                }

                let param_symbol = sig.parameters[i];
                let param_type = self.symbol_types.get(&param_symbol)
                    .copied()
                    .unwrap_or(self.types.any_type);

                if !self.is_type_assignable_to(arg_type, param_type) {
                    matches = false;
                    break;
                }
            }

            if matches {
                return sig.resolved_return_type.unwrap_or(self.types.any_type);
            }
        }

        // No matching overload found
        self.error(
            call_node,
            "No overload matches this call.",
            diagnostic_codes::NO_OVERLOAD_MATCHES_CALL
        );

        // Return the first overload's return type as fallback
        signatures.first()
            .and_then(|s| s.resolved_return_type)
            .unwrap_or(self.types.any_type)
    }

    /// Infer type arguments for a generic function call from the provided arguments.
    fn infer_type_arguments(
        &mut self,
        type_parameters: &[TypeId],
        parameter_types: &[TypeId],
        arguments: &crate::parser::NodeList
    ) -> Vec<TypeId> {
        // Create a mapping from type parameter to inferred type
        let mut inferred: std::collections::HashMap<TypeId, TypeId> = std::collections::HashMap::new();

        // For each argument, try to infer type parameters from the corresponding parameter type
        for (i, &arg_node) in arguments.nodes.iter().enumerate() {
            if i >= parameter_types.len() {
                break;
            }

            let param_type = parameter_types[i];

            // Set contextual type for this argument before evaluating it.
            // This enables contextual typing for callback parameters.
            let prev_contextual_type = self.contextual_type;
            self.contextual_type = Some(param_type);

            let arg_type = self.get_type_of_node(arg_node);

            // Restore previous contextual type
            self.contextual_type = prev_contextual_type;

            // If the parameter type is a type parameter, infer it from the argument type
            self.infer_from_types(param_type, arg_type, type_parameters, &mut inferred);
        }

        // Build the result vector in order of type parameters
        type_parameters.iter()
            .map(|&tp| *inferred.get(&tp).unwrap_or(&self.types.any_type))
            .collect()
    }

    /// Recursively infer type arguments by matching a pattern type against an actual type.
    fn infer_from_types(
        &mut self,
        pattern_type: TypeId,
        actual_type: TypeId,
        type_parameters: &[TypeId],
        inferred: &mut std::collections::HashMap<TypeId, TypeId>
    ) {
        // If the pattern is a type parameter, infer it
        if type_parameters.contains(&pattern_type) {
            // If we already inferred this type parameter, we could merge types (union)
            // For now, just use the first inference
            inferred.entry(pattern_type).or_insert(actual_type);
            return;
        }

        // Extract info from pattern type to avoid borrow issues
        enum PatternInfo {
            Function { parameter_types: Vec<TypeId>, return_type: TypeId },
            Union { types: Vec<TypeId> },
            Intersection { types: Vec<TypeId> },
            Other,
        }

        let pattern_info = match self.types.get(pattern_type) {
            Some(Type::Function(f)) => PatternInfo::Function {
                parameter_types: f.parameter_types.clone(),
                return_type: f.return_type,
            },
            Some(Type::Union(u)) => PatternInfo::Union { types: u.types.clone() },
            Some(Type::Intersection(i)) => PatternInfo::Intersection { types: i.types.clone() },
            _ => PatternInfo::Other,
        };

        // Extract info from actual type
        let actual_info = match self.types.get(actual_type) {
            Some(Type::Function(f)) => PatternInfo::Function {
                parameter_types: f.parameter_types.clone(),
                return_type: f.return_type,
            },
            Some(Type::Union(u)) => PatternInfo::Union { types: u.types.clone() },
            Some(Type::Intersection(i)) => PatternInfo::Intersection { types: i.types.clone() },
            _ => PatternInfo::Other,
        };

        // Match function types: (T) => U with (string) => number infers T=string, U=number
        if let (
            PatternInfo::Function { parameter_types: pattern_params, return_type: pattern_return },
            PatternInfo::Function { parameter_types: actual_params, return_type: actual_return }
        ) = (&pattern_info, &actual_info) {
            // Infer from parameter types (contravariant, but for simplicity we use covariant here)
            for (pattern_param, actual_param) in pattern_params.iter().zip(actual_params.iter()) {
                self.infer_from_types(*pattern_param, *actual_param, type_parameters, inferred);
            }
            // Infer from return type
            self.infer_from_types(*pattern_return, *actual_return, type_parameters, inferred);
        }

        // TODO: Handle object types, array types, etc.
    }

    /// Get the type of a new expression.
    fn get_type_of_new_expression(&mut self, expression: NodeIndex, _arguments: &Option<crate::parser::NodeList>) -> TypeId {
        // Get the type of the constructor
        let constructor_type = self.get_type_of_node(expression);

        // If it's a function type, create an instance type
        // For now, just return any - proper class instantiation is complex
        if let Some(Type::Function(_)) = self.types.get(constructor_type) {
            // TODO: Return the instance type
            return self.types.any_type;
        }

        // If it's an object with construct signatures, use those
        if let Some(Type::Object(obj)) = self.types.get(constructor_type) {
            if !obj.construct_signatures.is_empty() {
                if let Some(return_type) = obj.construct_signatures[0].resolved_return_type {
                    return return_type;
                }
            }
        }

        self.types.any_type
    }

    /// Get the type of an array literal.
    fn get_type_of_array_literal(&mut self, elements: &crate::parser::NodeList) -> TypeId {
        use crate::parser::Node;

        // Get contextual element type(s) if available
        let contextual_element_info = self.get_contextual_array_element_type();

        // If contextual type is a tuple with the same number of elements, create a tuple
        if let Some(ContextualArrayInfo::Tuple(tuple_types)) = &contextual_element_info {
            if tuple_types.len() == elements.nodes.len() {
                // Collect element types with contextual typing (preserving order for tuple)
                let mut tuple_element_types = Vec::new();
                for (i, &elem_idx) in elements.nodes.iter().enumerate() {
                    if let Some(elem_node) = self.node_arena.get(elem_idx) {
                        let elem_contextual = tuple_types.get(i).copied();
                        let elem_type = match elem_node {
                            Node::SpreadElement(spread) => {
                                let spread_type = self.get_type_of_node(spread.expression);
                                self.get_element_type_of_spread(spread_type)
                            }
                            _ => {
                                if let Some(ctx_type) = elem_contextual {
                                    let prev_contextual = self.contextual_type;
                                    self.contextual_type = Some(ctx_type);
                                    let t = self.get_type_of_node(elem_idx);
                                    self.contextual_type = prev_contextual;
                                    t
                                } else {
                                    self.get_type_of_node(elem_idx)
                                }
                            }
                        };
                        tuple_element_types.push(elem_type);
                    }
                }
                return self.types.create_tuple_type(tuple_element_types, false, false, false);
            }
        }

        // Collect unique element types (for arrays)
        let mut element_types = Vec::new();
        for (i, &elem_idx) in elements.nodes.iter().enumerate() {
            if let Some(elem_node) = self.node_arena.get(elem_idx) {
                // Determine the contextual type for this element
                let elem_contextual = match &contextual_element_info {
                    Some(ContextualArrayInfo::Array(elem_type)) => Some(*elem_type),
                    Some(ContextualArrayInfo::Tuple(tuple_types)) => {
                        tuple_types.get(i).copied()
                    }
                    None => None,
                };

                let elem_type = match elem_node {
                    Node::SpreadElement(spread) => {
                        // For spread element, get the element type of the spread's array
                        let spread_type = self.get_type_of_node(spread.expression);
                        self.get_element_type_of_spread(spread_type)
                    }
                    _ => {
                        // Apply contextual type when evaluating the element
                        if let Some(ctx_type) = elem_contextual {
                            let prev_contextual = self.contextual_type;
                            self.contextual_type = Some(ctx_type);
                            let t = self.get_type_of_node(elem_idx);
                            self.contextual_type = prev_contextual;
                            t
                        } else {
                            self.get_type_of_node(elem_idx)
                        }
                    }
                };
                if !element_types.contains(&elem_type) {
                    element_types.push(elem_type);
                }
            }
        }

        // Create an Array type from the element types
        // If contextual type is an array, use its element type to widen literals
        let element_type = if element_types.is_empty() {
            // Empty array - use contextual type if available, otherwise never[]
            match &contextual_element_info {
                Some(ContextualArrayInfo::Array(elem_type)) => *elem_type,
                Some(ContextualArrayInfo::Tuple(_)) => self.types.never_type,
                None => self.types.never_type,
            }
        } else if element_types.len() == 1 {
            // Single element type
            // If contextual is Array and element is assignable, use contextual element type for widening
            if let Some(ContextualArrayInfo::Array(ctx_elem)) = &contextual_element_info {
                if self.is_type_assignable_to(element_types[0], *ctx_elem) {
                    *ctx_elem
                } else {
                    element_types[0]
                }
            } else {
                element_types[0]
            }
        } else {
            // Multiple element types
            // Check if all are assignable to contextual element type
            if let Some(ContextualArrayInfo::Array(ctx_elem)) = &contextual_element_info {
                let all_assignable = element_types.iter().all(|&t| self.is_type_assignable_to(t, *ctx_elem));
                if all_assignable {
                    *ctx_elem
                } else {
                    self.types.create_union_type(element_types)
                }
            } else {
                self.types.create_union_type(element_types)
            }
        };

        // Return an Array type, not a union of element types
        self.types.create_array_type(element_type, false)
    }

    /// Get the element type from the contextual type if it's an array or tuple.
    fn get_contextual_array_element_type(&self) -> Option<ContextualArrayInfo> {
        let ctx_type_id = self.contextual_type?;
        let ctx_type = self.types.get(ctx_type_id)?;

        match ctx_type {
            Type::Array(arr) => Some(ContextualArrayInfo::Array(arr.element_type)),
            Type::Tuple(tup) => Some(ContextualArrayInfo::Tuple(tup.element_types.clone())),
            _ => None,
        }
    }

    /// Get the element type when spreading an array/tuple into another array.
    fn get_element_type_of_spread(&mut self, spread_type: TypeId) -> TypeId {
        // Need to extract the element types before creating a union to avoid borrow issues
        let spread_info = if let Some(ty) = self.types.get(spread_type) {
            match ty {
                Type::Array(arr) => Some((vec![arr.element_type], false)),
                Type::Tuple(tup) => Some((tup.element_types.clone(), true)),
                _ => None,
            }
        } else {
            None
        };

        match spread_info {
            Some((types, is_tuple)) => {
                if types.is_empty() {
                    self.types.never_type
                } else if types.len() == 1 {
                    types[0]
                } else if is_tuple {
                    // For tuples, create a union of all element types
                    self.types.create_union_type(types)
                } else {
                    types[0]
                }
            }
            None => {
                // For other types (like any), just return the type
                spread_type
            }
        }
    }

    /// Get the type of a conditional type (T extends U ? X : Y).
    /// Evaluates the condition and returns either the true or false branch type.
    fn get_type_of_conditional_type(&mut self, ct: &crate::parser::ConditionalType) -> TypeId {
        let check_type = self.get_type_of_node(ct.check_type);
        let extends_type = self.get_type_of_node(ct.extends_type);
        let true_type_node_id = ct.true_type;
        let false_type_node_id = ct.false_type;

        // Check if the check type contains unresolved type parameters
        // In that case, we need to defer evaluation (return a conditional type)
        if self.type_contains_type_parameter(check_type) {
            // Evaluate both branch types first
            let true_type = self.get_type_of_node(true_type_node_id);
            let false_type = self.get_type_of_node(false_type_node_id);
            // Create a deferred conditional type
            return self.types.create_conditional_type(
                check_type,
                extends_type,
                true_type,
                false_type,
            );
        }

        // For union types in check position, distribute the conditional
        // (A | B) extends U ? X : Y becomes (A extends U ? X : Y) | (B extends U ? X : Y)
        if let Some(Type::Union(union)) = self.types.get(check_type) {
            let member_types = union.types.clone();
            let mut result_types = Vec::new();
            for member in member_types {
                let result = if self.is_type_assignable_to(member, extends_type) {
                    self.get_type_of_node(true_type_node_id)
                } else {
                    self.get_type_of_node(false_type_node_id)
                };
                if !result_types.contains(&result) {
                    result_types.push(result);
                }
            }
            if result_types.len() == 1 {
                return result_types[0];
            }
            return self.types.create_union_type(result_types);
        }

        // Check if extends_type contains infer types
        if self.type_contains_infer(extends_type) {
            // Try to match the pattern and extract inferred types
            let mut inferences: std::collections::HashMap<TypeId, TypeId> = std::collections::HashMap::new();
            if self.infer_from_type(check_type, extends_type, &mut inferences) {
                // Pattern matched - evaluate true branch with inferences substituted
                let true_type = self.get_type_of_node(true_type_node_id);
                return self.instantiate_type_with_mapper(true_type, &inferences);
            } else {
                // Pattern didn't match - use false branch
                return self.get_type_of_node(false_type_node_id);
            }
        }

        // Simple case: check if check_type is assignable to extends_type
        if self.is_type_assignable_to(check_type, extends_type) {
            self.get_type_of_node(true_type_node_id)
        } else {
            self.get_type_of_node(false_type_node_id)
        }
    }

    /// Check if a type contains infer type parameters.
    fn type_contains_infer(&self, type_id: TypeId) -> bool {
        let Some(ty) = self.types.get(type_id) else {
            return false;
        };

        match ty {
            // Check if this is an infer type (type parameter with special marker)
            Type::TypeParameter(tp) => {
                // Infer types are type parameters created without a symbol
                tp.symbol.is_none()
            }
            Type::TypeReference(tr) => {
                tr.type_arguments.iter().any(|&t| self.type_contains_infer(t))
            }
            Type::Union(u) => u.types.iter().any(|&t| self.type_contains_infer(t)),
            Type::Intersection(i) => i.types.iter().any(|&t| self.type_contains_infer(t)),
            Type::Array(arr) => self.type_contains_infer(arr.element_type),
            Type::Tuple(tup) => tup.element_types.iter().any(|&t| self.type_contains_infer(t)),
            Type::Function(f) => {
                f.parameter_types.iter().any(|&t| self.type_contains_infer(t))
                    || self.type_contains_infer(f.return_type)
            }
            _ => false,
        }
    }

    /// Match a source type against a pattern type, extracting inferred types.
    /// Returns true if the pattern matches, false otherwise.
    /// Inferred bindings are stored in the inferences map (pattern TypeId -> matched TypeId).
    pub(crate) fn infer_from_type(
        &self,
        source: TypeId,
        pattern: TypeId,
        inferences: &mut std::collections::HashMap<TypeId, TypeId>,
    ) -> bool {
        let Some(pattern_type) = self.types.get(pattern) else {
            return false;
        };

        // If pattern is an infer type parameter, bind it to source
        if let Type::TypeParameter(tp) = pattern_type {
            if tp.symbol.is_none() {
                // This is an infer type
                // Check if there's a constraint (from `infer T extends U` syntax)
                let constraint = tp.constraint;
                if !constraint.is_none() {
                    // Validate that source satisfies the constraint
                    if !self.is_type_assignable_to(source, constraint) {
                        // Source doesn't satisfy the constraint - match fails
                        return false;
                    }
                }
                // Bind the infer type to source
                inferences.insert(pattern, source);
                return true;
            }
        }

        // Try to match based on pattern type
        match pattern_type {
            Type::TypeReference(pattern_ref) => {
                // For type references like Promise<infer U>, match the structure
                if let Some(Type::TypeReference(source_ref)) = self.types.get(source) {
                    // Match type arguments
                    let pattern_args = pattern_ref.type_arguments.clone();
                    let source_args = source_ref.type_arguments.clone();

                    if pattern_args.len() != source_args.len() {
                        return false;
                    }

                    for (pattern_arg, source_arg) in pattern_args.iter().zip(source_args.iter()) {
                        if !self.infer_from_type(*source_arg, *pattern_arg, inferences) {
                            return false;
                        }
                    }
                    return true;
                }
                false
            }
            Type::Array(pattern_arr) => {
                if let Some(Type::Array(source_arr)) = self.types.get(source) {
                    let pattern_elem = pattern_arr.element_type;
                    let source_elem = source_arr.element_type;
                    self.infer_from_type(source_elem, pattern_elem, inferences)
                } else {
                    false
                }
            }
            Type::Tuple(pattern_tup) => {
                if let Some(Type::Tuple(source_tup)) = self.types.get(source) {
                    let pattern_elems = pattern_tup.element_types.clone();
                    let source_elems = source_tup.element_types.clone();
                    if pattern_elems.len() != source_elems.len() {
                        return false;
                    }
                    for (p, s) in pattern_elems.iter().zip(source_elems.iter()) {
                        if !self.infer_from_type(*s, *p, inferences) {
                            return false;
                        }
                    }
                    true
                } else {
                    false
                }
            }
            Type::Function(pattern_fn) => {
                if let Some(Type::Function(source_fn)) = self.types.get(source) {
                    // Match return type
                    let pattern_ret = pattern_fn.return_type;
                    let source_ret = source_fn.return_type;
                    self.infer_from_type(source_ret, pattern_ret, inferences)
                } else {
                    false
                }
            }
            _ => {
                // For other types, just check assignability
                self.is_type_assignable_to(source, pattern)
            }
        }
    }

    /// Check if a type contains unresolved type parameters.
    fn type_contains_type_parameter(&self, type_id: TypeId) -> bool {
        if let Some(ty) = self.types.get(type_id) {
            match ty {
                Type::TypeParameter(_) => true,
                Type::Union(u) => u.types.iter().any(|&t| self.type_contains_type_parameter(t)),
                Type::Intersection(i) => i.types.iter().any(|&t| self.type_contains_type_parameter(t)),
                Type::Array(arr) => self.type_contains_type_parameter(arr.element_type),
                Type::Tuple(tup) => tup.element_types.iter().any(|&t| self.type_contains_type_parameter(t)),
                Type::TypeReference(r) => {
                    r.type_arguments.iter().any(|&t| self.type_contains_type_parameter(t))
                }
                Type::Conditional(c) => {
                    self.type_contains_type_parameter(c.check_type)
                        || self.type_contains_type_parameter(c.extends_type)
                        || self.type_contains_type_parameter(c.true_type)
                        || self.type_contains_type_parameter(c.false_type)
                }
                _ => false,
            }
        } else {
            false
        }
    }

    /// Get the type of a template literal type (`hello ${T}`).
    fn get_type_of_template_literal_type(&mut self, tlt: &crate::parser::TemplateLiteralType) -> TypeId {
        use crate::parser::Node;

        let mut texts: Vec<String> = Vec::new();
        let mut types: Vec<TypeId> = Vec::new();

        // Get the head text
        if let Some(Node::NoSubstitutionTemplateLiteral(lit) | Node::TemplateHead(lit)) =
            self.node_arena.get(tlt.head)
        {
            texts.push(lit.text.clone());
        }

        // Get the spans (type, literal pairs)
        for &span_idx in &tlt.template_spans.nodes {
            if let Some(Node::TemplateSpan(span)) = self.node_arena.get(span_idx) {
                // Get the type
                let span_type = self.get_type_of_node(span.expression);
                types.push(span_type);

                // Get the literal text
                if let Some(Node::TemplateMiddle(lit) | Node::TemplateTail(lit)) =
                    self.node_arena.get(span.literal)
                {
                    texts.push(lit.text.clone());
                }
            }
        }

        // If all types are string literals, we can simplify to a single string literal
        if types.iter().all(|&t| {
            self.types.get(t).map_or(false, |ty| {
                matches!(ty, Type::Literal(LiteralType { value: LiteralValue::String(_), .. }))
            })
        }) {
            // Concatenate all parts
            let mut result = String::new();
            for (i, text) in texts.iter().enumerate() {
                result.push_str(text);
                if i < types.len() {
                    if let Some(Type::Literal(LiteralType { value: LiteralValue::String(s), .. })) =
                        self.types.get(types[i])
                    {
                        result.push_str(s);
                    }
                }
            }
            return self.types.create_string_literal(result);
        }

        // Otherwise, create a template literal type
        self.types.create_template_literal_type(texts, types)
    }

    /// Get the type of a mapped type ({ [K in keyof T]: T[K] }).
    fn get_type_of_mapped_type(&mut self, node: NodeIndex, mt: &crate::parser::MappedType) -> TypeId {
        use crate::parser::Node;

        // Track if we added a type parameter - save its name and previous value for restoration
        // This handles shadowing correctly in nested mapped types
        let mut shadowed_param: Option<(String, Option<TypeId>)> = None;

        // Get the type parameter (K in "K in keyof T") and add it to scope
        let type_param_type = if !mt.type_parameter.is_none() {
            let type_id = self.get_type_of_node(mt.type_parameter);
            // Get the name of the type parameter and add to current scope
            if let Some(Node::TypeParameterDeclaration(tp)) = self.node_arena.get(mt.type_parameter) {
                if let Some(Node::Identifier(id)) = self.node_arena.get(tp.name) {
                    let name = id.escaped_text.clone();
                    // Save previous value (if any) before inserting
                    let prev_value = self.type_parameter_scope.insert(name.clone(), type_id);
                    self.type_parameter_names.insert(type_id, name.clone());
                    shadowed_param = Some((name, prev_value));
                }
            }
            type_id
        } else {
            self.types.any_type
        };

        // Get the constraint type (keyof T in "K in keyof T")
        // The type_parameter node should have a constraint
        let constraint_type = if let Some(Node::TypeParameterDeclaration(tp)) = self.node_arena.get(mt.type_parameter) {
            if !tp.constraint.is_none() {
                self.get_type_of_node(tp.constraint)
            } else {
                self.types.any_type
            }
        } else {
            self.types.any_type
        };

        // Get the name type (the "as" clause, if present)
        let name_type = if !mt.name_type.is_none() {
            self.get_type_of_node(mt.name_type)
        } else {
            self.types.any_type
        };

        // Get the template type (the value type, e.g., T[K])
        // K is now in type_parameter_scope, so references to K will resolve correctly
        let template_type = if !mt.type_node.is_none() {
            self.get_type_of_node(mt.type_node)
        } else {
            self.types.any_type
        };

        // Restore the previous type parameter scope entry (handles shadowing correctly)
        if let Some((name, prev_value)) = shadowed_param {
            if let Some(prev_id) = prev_value {
                self.type_parameter_scope.insert(name, prev_id);
            } else {
                self.type_parameter_scope.remove(&name);
            }
        }

        // Create a deferred mapped type (evaluation happens during instantiation)
        self.types.create_mapped_type(
            node,
            type_param_type,
            constraint_type,
            name_type,
            template_type,
        )
    }

    /// Get the type of an infer type (infer T in conditional types).
    /// Creates a special type parameter that can be bound during pattern matching.
    /// Supports `infer T extends U` syntax where the constraint limits valid inferences.
    fn get_type_of_infer_type(
        &mut self,
        _node: NodeIndex,
        it: &crate::parser::InferType,
    ) -> TypeId {
        use crate::parser::Node;
        use crate::binder::SymbolId;

        // Get the type parameter name and constraint from the type parameter declaration
        let (name, constraint_node) = if let Some(Node::TypeParameterDeclaration(tp)) = self.node_arena.get(it.type_parameter) {
            let name = if let Some(Node::Identifier(id)) = self.node_arena.get(tp.name) {
                id.escaped_text.clone()
            } else {
                "T".to_string()
            };
            let constraint = if !tp.constraint.is_none() {
                tp.constraint
            } else {
                NodeIndex::NONE
            };
            (name, constraint)
        } else {
            ("T".to_string(), NodeIndex::NONE)
        };

        // Get the constraint type if one exists (for `infer T extends U` syntax)
        let constraint_type = if !constraint_node.is_none() {
            self.get_type_of_node(constraint_node)
        } else {
            TypeId::NONE
        };

        // Create a type parameter for this infer type
        // The symbol is NONE since infer types don't have symbols in the symbol table
        let type_param = self.types.create_type_parameter(SymbolId::NONE, constraint_type, TypeId::NONE);

        // Cache the name for type_to_string
        self.type_parameter_names.insert(type_param, name.clone());

        // Add to type_parameter_scope so that later references to the name can find it
        // This is important for conditional types where the true branch references the infer type
        self.type_parameter_scope.insert(name, type_param);

        type_param
    }

    /// Get the type of a type alias declaration.
    /// Sets up type parameter scope before evaluating the alias body.
    fn get_type_of_type_alias_declaration(
        &mut self,
        ta: &crate::parser::TypeAliasDeclaration,
    ) -> TypeId {
        use crate::parser::Node;

        // Save current type parameter scope
        let saved_scope = std::mem::take(&mut self.type_parameter_scope);

        // Set up type parameters if present
        if let Some(ref type_params) = ta.type_parameters {
            for &tp_idx in &type_params.nodes {
                if let Some(type_id) = self.create_type_parameter(tp_idx) {
                    // Add to type parameter scope for name lookup
                    if let Some(name) = self.type_parameter_names.get(&type_id) {
                        self.type_parameter_scope.insert(name.clone(), type_id);
                    }
                }
            }
        }

        // Evaluate the type alias body with type parameters in scope
        let result_type = self.get_type_of_node(ta.type_node);

        // Restore previous type parameter scope
        self.type_parameter_scope = saved_scope;

        result_type
    }

    /// Get the keyof type for a given type.
    /// Returns a union of string literal types for the property names.
    fn get_keyof_type(&mut self, type_id: TypeId) -> TypeId {
        // Handle special cases
        if type_id == TypeId::NONE {
            return self.types.never_type;
        }

        let Some(typ) = self.types.get(type_id) else {
            return self.types.never_type;
        };

        match typ {
            // For object types, extract property names as string literals
            Type::Object(obj) => {
                let property_ids = obj.properties.clone();
                let mut key_types = Vec::new();

                for prop_id in property_ids {
                    // Use get_symbol to look up in both binder and local symbol arenas
                    if let Some(sym) = self.get_symbol(prop_id) {
                        // Create a string literal type for the property name
                        let key_type = self.types.create_string_literal(sym.escaped_name.clone());
                        key_types.push(key_type);
                    }
                }

                // Also check members SymbolTable
                let members = self.types.get(type_id)
                    .and_then(|t| if let Type::Object(o) = t { Some(o.members.clone()) } else { None });

                if let Some(members) = members {
                    for (name, _) in members.iter() {
                        let key_type = self.types.create_string_literal(name.clone());
                        if !key_types.contains(&key_type) {
                            key_types.push(key_type);
                        }
                    }
                }

                if key_types.is_empty() {
                    // Empty object has no keys
                    self.types.never_type
                } else if key_types.len() == 1 {
                    key_types[0]
                } else {
                    self.types.create_union_type(key_types)
                }
            }
            // For type parameters, create an index type
            Type::TypeParameter(_) => {
                // keyof T where T is a type parameter - create Index type
                self.types.create_index_type(type_id)
            }
            // For union types, distribute keyof
            Type::Union(u) => {
                // keyof (A | B) = keyof A & keyof B (intersection of keys)
                // For now, just return string as placeholder
                let types = u.types.clone();
                if types.is_empty() {
                    return self.types.never_type;
                }
                // Get keyof for each member and intersect
                let mut key_types: Vec<TypeId> = types.iter()
                    .map(|&t| self.get_keyof_type(t))
                    .collect();
                if key_types.len() == 1 {
                    key_types[0]
                } else {
                    // For simplicity, just return string
                    self.types.string_type
                }
            }
            // For intrinsic types, return appropriate key types
            Type::Intrinsic(i) => {
                match i.intrinsic_name.as_str() {
                    "any" => {
                        // keyof any = string | number | symbol
                        let types = vec![
                            self.types.string_type,
                            self.types.number_type,
                            self.types.es_symbol_type,
                        ];
                        self.types.create_union_type(types)
                    }
                    "unknown" => self.types.never_type,
                    "string" => {
                        // String has methods like length, charAt, etc.
                        // For simplicity, return number | keyof String prototype
                        self.types.number_type
                    }
                    "number" => self.types.never_type,
                    _ => self.types.never_type,
                }
            }
            // Default: return string | number | symbol
            _ => {
                let types = vec![
                    self.types.string_type,
                    self.types.number_type,
                    self.types.es_symbol_type,
                ];
                self.types.create_union_type(types)
            }
        }
    }

    /// Get the type of a class declaration.
    /// Creates an ObjectType with CLASS object flags, containing all class members.
    /// The returned type is the "constructor type" which has a construct signature
    /// that returns the instance type.
    fn get_type_of_class_declaration(
        &mut self,
        node: NodeIndex,
        class: &crate::parser::ClassDeclaration,
    ) -> TypeId {
        use crate::parser::Node;

        // Early caching to handle recursive types (like interfaces)
        // 1. Resolve the symbol for this class
        let class_symbol_id = if let Some(Node::Identifier(id)) = self.node_arena.get(class.name) {
            self.file_locals.get(&id.escaped_text).unwrap_or(SymbolId::NONE)
        } else {
            SymbolId::NONE
        };

        // 2. Check if we already have a cached type for this node
        if let Some(&cached) = self.node_types.get(&node) {
            return cached;
        }

        // 3. Create a placeholder ObjectType and cache it immediately to break recursion cycles
        let placeholder = ObjectType::new(object_flags::CLASS, class_symbol_id);
        let placeholder_type_id = self.types.alloc(Type::Object(Box::new(placeholder)));
        self.node_types.insert(node, placeholder_type_id);
        if !class_symbol_id.is_none() {
            self.symbol_types.insert(class_symbol_id, placeholder_type_id);
        }

        let mut properties = Vec::new();
        let mut constructor_params: Vec<(NodeIndex, SymbolId)> = Vec::new();
        let mut has_constructor = false;

        // First pass: collect properties and methods (for instance type)
        for &member_idx in &class.members.nodes {
            if let Some(member_node) = self.node_arena.get(member_idx) {
                match member_node {
                    // Property declarations
                    Node::PropertyDeclaration(pd) => {
                        // Get property name
                        let name = if let Some(Node::Identifier(id)) = self.node_arena.get(pd.name) {
                            id.escaped_text.clone()
                        } else {
                            continue;
                        };

                        // Get property type
                        let prop_type = if !pd.type_annotation.is_none() {
                            self.get_type_of_node(pd.type_annotation)
                        } else if !pd.initializer.is_none() {
                            self.get_type_of_node(pd.initializer)
                        } else {
                            self.types.any_type
                        };

                        // Determine symbol flags from modifiers (for visibility checking)
                        let mut flags = symbol_flags::PROPERTY;
                        if let Some(ref modifiers) = pd.modifiers {
                            for &mod_idx in &modifiers.nodes {
                                if let Some(node) = self.node_arena.get(mod_idx) {
                                    let kind = node.base().kind;
                                    if kind == SyntaxKind::PrivateKeyword as u16 {
                                        flags |= symbol_flags::PRIVATE;
                                    } else if kind == SyntaxKind::ProtectedKeyword as u16 {
                                        flags |= symbol_flags::PROTECTED;
                                    }
                                    // Note: READONLY and STATIC are in modifier flags, not symbol flags
                                }
                            }
                        }

                        // Create a symbol for this property with visibility flags
                        let symbol_id = self.local_symbols_mut().alloc(flags, name.clone());
                        self.symbol_types.insert(symbol_id, prop_type);
                        properties.push(symbol_id);
                    }

                    // Method declarations
                    Node::MethodDeclaration(md) => {
                        // Get method name
                        let name = if let Some(Node::Identifier(id)) = self.node_arena.get(md.name) {
                            id.escaped_text.clone()
                        } else {
                            continue;
                        };

                        // Get method type
                        let method_type = self.get_type_of_function_like_with_type_params(
                            member_idx,
                            &md.parameters,
                            md.type_annotation,
                            md.type_parameters.as_ref(),
                        );

                        // Create a symbol for this method
                        let symbol_id = self.local_symbols_mut().alloc(symbol_flags::METHOD, name.clone());
                        self.symbol_types.insert(symbol_id, method_type);
                        properties.push(symbol_id);
                    }

                    // Constructor declaration - collect params
                    Node::ConstructorDeclaration(cd) => {
                        has_constructor = true;
                        for &param_idx in &cd.parameters.nodes {
                            if let Some(Node::ParameterDeclaration(param)) = self.node_arena.get(param_idx) {
                                if let Some(Node::Identifier(id)) = self.node_arena.get(param.name) {
                                    let param_symbol = self.local_symbols_mut().alloc(
                                        symbol_flags::FUNCTION_SCOPED_VARIABLE,
                                        id.escaped_text.clone(),
                                    );
                                    constructor_params.push((param_idx, param_symbol));
                                }
                            }
                        }
                    }

                    // Get accessor
                    Node::GetAccessorDeclaration(ga) => {
                        // Get accessor name
                        let name = if let Some(Node::Identifier(id)) = self.node_arena.get(ga.name) {
                            id.escaped_text.clone()
                        } else {
                            continue;
                        };

                        // Get return type
                        let get_type = if !ga.type_annotation.is_none() {
                            self.get_type_of_node(ga.type_annotation)
                        } else {
                            self.types.any_type
                        };

                        // Create a symbol for this accessor
                        let symbol_id = self.local_symbols_mut().alloc(symbol_flags::GET_ACCESSOR, name.clone());
                        self.symbol_types.insert(symbol_id, get_type);
                        properties.push(symbol_id);
                    }

                    // Set accessor
                    Node::SetAccessorDeclaration(sa) => {
                        // Get accessor name
                        let name = if let Some(Node::Identifier(id)) = self.node_arena.get(sa.name) {
                            id.escaped_text.clone()
                        } else {
                            continue;
                        };

                        // Set accessor type is the parameter type
                        let set_type = if !sa.parameters.nodes.is_empty() {
                            if let Some(Node::ParameterDeclaration(param)) =
                                self.node_arena.get(sa.parameters.nodes[0])
                            {
                                if !param.type_annotation.is_none() {
                                    self.get_type_of_node(param.type_annotation)
                                } else {
                                    self.types.any_type
                                }
                            } else {
                                self.types.any_type
                            }
                        } else {
                            self.types.any_type
                        };

                        // Create a symbol for this accessor
                        let symbol_id = self.local_symbols_mut().alloc(symbol_flags::SET_ACCESSOR, name.clone());
                        self.symbol_types.insert(symbol_id, set_type);
                        properties.push(symbol_id);
                    }

                    _ => {}
                }
            }
        }

        // Create the instance type (properties only, no construct signatures)
        // This is the type of instances created with `new`
        let instance_type = self.types.create_class_type(properties.clone(), vec![], vec![]);

        // Create the construct signature - this is what allows `new Foo()`
        let mut construct_signature = Signature::new(node);
        for (_, param_symbol) in constructor_params {
            construct_signature.parameters.push(param_symbol);
        }
        construct_signature.min_argument_count = construct_signature.parameters.len() as u32;
        construct_signature.resolved_return_type = Some(instance_type);

        // If no explicit constructor, create an implicit one
        let construct_signatures = if has_constructor || !construct_signature.parameters.is_empty() {
            vec![construct_signature]
        } else {
            // Implicit constructor with no parameters
            let mut implicit_sig = Signature::new(node);
            implicit_sig.resolved_return_type = Some(instance_type);
            vec![implicit_sig]
        };

        // Update the placeholder type in-place with the resolved members
        // This ensures any recursive references (like `class Node { parent: Node }`)
        // correctly point to the final type, not an empty placeholder.
        if let Some(Type::Object(obj)) = self.types.get_mut(placeholder_type_id) {
            obj.properties = properties;
            obj.construct_signatures = construct_signatures;
        }

        // The placeholder is now the constructor type - return it
        placeholder_type_id
    }

    /// Get the type of an interface declaration.
    /// Creates an ObjectType with INTERFACE object flags.
    fn get_type_of_interface_declaration(
        &mut self,
        node: NodeIndex,
        iface: &crate::parser::InterfaceDeclaration,
    ) -> TypeId {
        use crate::parser::Node;

        // Early caching to handle recursive types
        // 1. Resolve the symbol for this interface
        let interface_symbol_id = if let Some(Node::Identifier(id)) = self.node_arena.get(iface.name) {
            self.file_locals.get(&id.escaped_text).unwrap_or(SymbolId::NONE)
        } else {
            SymbolId::NONE
        };

        // 2. Create a placeholder ObjectType
        let obj = ObjectType::new(object_flags::INTERFACE, interface_symbol_id);
        let type_id = self.types.alloc(Type::Object(Box::new(obj)));

        // 3. Cache it immediately to break recursion cycles
        self.node_types.insert(node, type_id);
        if !interface_symbol_id.is_none() {
            self.symbol_types.insert(interface_symbol_id, type_id);
        }

        let mut properties = Vec::new();
        let mut members_table = SymbolTable::new();
        let mut call_signatures = Vec::new();
        let mut construct_signatures = Vec::new();
        let mut index_infos = Vec::new();

        // Process interface members
        for &member_idx in &iface.members.nodes {
            if let Some(member_node) = self.node_arena.get(member_idx) {
                match member_node {
                    // Property signatures
                    Node::PropertySignature(ps) => {
                        // Get property name
                        let name = if let Some(Node::Identifier(id)) = self.node_arena.get(ps.name) {
                            id.escaped_text.clone()
                        } else {
                            continue;
                        };

                        // Get property type
                        let prop_type = if !ps.type_annotation.is_none() {
                            self.get_type_of_node(ps.type_annotation)
                        } else {
                            self.types.any_type
                        };

                        // Create a symbol for this property
                        let symbol_id = self.local_symbols_mut().alloc(symbol_flags::PROPERTY, name.clone());
                        self.symbol_types.insert(symbol_id, prop_type);
                        members_table.set(name, symbol_id);
                        properties.push(symbol_id);
                    }

                    // Method signatures
                    Node::MethodSignature(ms) => {
                        // Get method name
                        let name = if let Some(Node::Identifier(id)) = self.node_arena.get(ms.name) {
                            id.escaped_text.clone()
                        } else {
                            continue;
                        };

                        // Get method type
                        let method_type = self.get_type_of_function_like_with_type_params(
                            member_idx,
                            &ms.parameters,
                            ms.type_annotation,
                            ms.type_parameters.as_ref(),
                        );

                        // Create a symbol for this method
                        let symbol_id = self.local_symbols_mut().alloc(symbol_flags::METHOD, name.clone());
                        self.symbol_types.insert(symbol_id, method_type);
                        members_table.set(name, symbol_id);
                        properties.push(symbol_id);
                    }

                    // Index signatures: [key: string]: Type
                    Node::IndexSignatureDeclaration(isd) => {
                        // Get the key type from the first parameter
                        let key_type = if !isd.parameters.nodes.is_empty() {
                            let param_idx = isd.parameters.nodes[0];
                            if let Some(Node::ParameterDeclaration(pd)) = self.node_arena.get(param_idx) {
                                if !pd.type_annotation.is_none() {
                                    self.get_type_of_node(pd.type_annotation)
                                } else {
                                    self.types.string_type // default to string
                                }
                            } else {
                                self.types.string_type
                            }
                        } else {
                            self.types.string_type
                        };

                        // Get the value type from the type annotation
                        let value_type = if !isd.type_annotation.is_none() {
                            self.get_type_of_node(isd.type_annotation)
                        } else {
                            self.types.any_type
                        };

                        // Check for readonly modifier
                        let is_readonly = isd.modifiers.as_ref().map_or(false, |mods| {
                            mods.nodes.iter().any(|&mod_idx| {
                                if let Some(Node::Token(base)) = self.node_arena.get(mod_idx) {
                                    base.kind == crate::scanner::SyntaxKind::ReadonlyKeyword as u16
                                } else {
                                    false
                                }
                            })
                        });

                        index_infos.push(IndexInfo {
                            key_type,
                            value_type,
                            is_readonly,
                            declaration: Some(member_idx),
                        });
                    }

                    // TODO: Add CallSignature and ConstructSignature when parser supports them
                    _ => {}
                }
            }
        }

        // Update the placeholder type with the resolved members
        if let Some(Type::Object(obj)) = self.types.get_mut(type_id) {
            obj.properties = properties;
            obj.members = members_table;
            obj.construct_signatures = construct_signatures;
            obj.call_signatures = call_signatures;
            obj.index_infos = index_infos;
        }

        type_id
    }

    /// Get the type of a type literal ({ x: number, y: string }).
    fn get_type_of_type_literal(&mut self, members: &crate::parser::NodeList) -> TypeId {
        use crate::parser::Node;

        let mut properties = Vec::new();
        let mut members_table = SymbolTable::new();
        let mut index_infos = Vec::new();
        let mut call_signatures = Vec::new();
        let mut construct_signatures = Vec::new();

        for &member_idx in &members.nodes {
            if let Some(node) = self.node_arena.get(member_idx) {
                match node {
                    Node::PropertySignature(ps) => {
                        // Get property name
                        let name = if let Some(Node::Identifier(id)) = self.node_arena.get(ps.name) {
                            id.escaped_text.clone()
                        } else {
                            continue;
                        };

                        // Get property type
                        let prop_type = if !ps.type_annotation.is_none() {
                            self.get_type_of_node(ps.type_annotation)
                        } else {
                            self.types.any_type
                        };

                        // Create a symbol for this property
                        let symbol_id = self.local_symbols_mut().alloc(symbol_flags::PROPERTY, name.clone());

                        // Cache the symbol's type
                        self.symbol_types.insert(symbol_id, prop_type);
                        properties.push(symbol_id);
                        members_table.set(name, symbol_id);
                    }
                    Node::MethodSignature(ms) => {
                        // Get method name
                        let name = if let Some(Node::Identifier(id)) = self.node_arena.get(ms.name) {
                            id.escaped_text.clone()
                        } else {
                            continue;
                        };

                        // Get method type
                        let method_type = self.get_type_of_function_like_with_type_params(
                            member_idx,
                            &ms.parameters,
                            ms.type_annotation,
                            ms.type_parameters.as_ref(),
                        );

                        // Create a symbol for this method
                        let symbol_id = self.local_symbols_mut().alloc(symbol_flags::METHOD, name.clone());

                        // Cache the symbol's type
                        self.symbol_types.insert(symbol_id, method_type);
                        properties.push(symbol_id);
                        members_table.set(name, symbol_id);
                    }
                    // Call signatures: (): Type or <T>(): T
                    Node::CallSignature(cs) => {
                        let sig = self.build_signature_from_call_or_construct(
                            member_idx,
                            &cs.parameters,
                            cs.type_annotation,
                            cs.type_parameters.as_ref(),
                        );
                        call_signatures.push(sig);
                    }
                    // Construct signatures: new (): Type or new<T>(): T
                    Node::ConstructSignature(cs) => {
                        let sig = self.build_signature_from_call_or_construct(
                            member_idx,
                            &cs.parameters,
                            cs.type_annotation,
                            cs.type_parameters.as_ref(),
                        );
                        construct_signatures.push(sig);
                    }
                    // Index signatures: [key: string]: Type
                    Node::IndexSignatureDeclaration(isd) => {
                        // Get the key type from the first parameter
                        let key_type = if !isd.parameters.nodes.is_empty() {
                            let param_idx = isd.parameters.nodes[0];
                            if let Some(Node::ParameterDeclaration(pd)) = self.node_arena.get(param_idx) {
                                if !pd.type_annotation.is_none() {
                                    self.get_type_of_node(pd.type_annotation)
                                } else {
                                    self.types.string_type // default to string
                                }
                            } else {
                                self.types.string_type
                            }
                        } else {
                            self.types.string_type
                        };

                        // Get the value type from the type annotation
                        let value_type = if !isd.type_annotation.is_none() {
                            self.get_type_of_node(isd.type_annotation)
                        } else {
                            self.types.any_type
                        };

                        // Check for readonly modifier
                        let is_readonly = isd.modifiers.as_ref().map_or(false, |mods| {
                            mods.nodes.iter().any(|&mod_idx| {
                                if let Some(Node::Token(base)) = self.node_arena.get(mod_idx) {
                                    base.kind == crate::scanner::SyntaxKind::ReadonlyKeyword as u16
                                } else {
                                    false
                                }
                            })
                        });

                        index_infos.push(IndexInfo {
                            key_type,
                            value_type,
                            is_readonly,
                            declaration: Some(member_idx),
                        });
                    }
                    _ => {}
                }
            }
        }

        // Create object type with members table, index infos, and signatures
        let mut obj = ObjectType::new(object_flags::ANONYMOUS, SymbolId::NONE);
        obj.properties = properties;
        obj.members = members_table;
        obj.index_infos = index_infos;
        obj.call_signatures = call_signatures;
        obj.construct_signatures = construct_signatures;
        self.types.alloc(Type::Object(Box::new(obj)))
    }

    /// Build a Signature from call or construct signature parameters.
    fn build_signature_from_call_or_construct(
        &mut self,
        declaration: NodeIndex,
        parameters: &crate::parser::NodeList,
        return_type_node: NodeIndex,
        type_parameters: Option<&crate::parser::NodeList>,
    ) -> Signature {
        use crate::parser::Node;

        let mut sig = Signature::new(declaration);

        // Process type parameters
        if let Some(type_params) = type_parameters {
            for &tp_idx in &type_params.nodes {
                if let Some(Node::TypeParameterDeclaration(tpd)) = self.node_arena.get(tp_idx) {
                    let name = if let Some(Node::Identifier(id)) = self.node_arena.get(tpd.name) {
                        id.escaped_text.clone()
                    } else {
                        "T".to_string()
                    };

                    let constraint = if !tpd.constraint.is_none() {
                        self.get_type_of_node(tpd.constraint)
                    } else {
                        TypeId::NONE
                    };

                    let default = if !tpd.default.is_none() {
                        self.get_type_of_node(tpd.default)
                    } else {
                        TypeId::NONE
                    };

                    let type_param = self.types.create_type_parameter(SymbolId::NONE, constraint, default);
                    self.type_parameter_names.insert(type_param, name.clone());
                    self.type_parameter_scope.insert(name, type_param);
                    sig.type_parameters.push(type_param);
                }
            }
        }

        // Process parameters
        let mut min_args: u32 = 0;
        for &param_idx in &parameters.nodes {
            if let Some(Node::ParameterDeclaration(pd)) = self.node_arena.get(param_idx) {
                let name = if let Some(Node::Identifier(id)) = self.node_arena.get(pd.name) {
                    id.escaped_text.clone()
                } else {
                    "arg".to_string()
                };

                let param_type = if !pd.type_annotation.is_none() {
                    self.get_type_of_node(pd.type_annotation)
                } else {
                    self.types.any_type
                };

                let symbol_id = self.local_symbols_mut().alloc(symbol_flags::FUNCTION_SCOPED_VARIABLE, name);
                self.symbol_types.insert(symbol_id, param_type);
                sig.parameters.push(symbol_id);

                // Count required parameters
                if !pd.question_token && pd.initializer.is_none() && !pd.dot_dot_dot_token {
                    min_args += 1;
                }
            }
        }
        sig.min_argument_count = min_args;

        // Get return type
        if !return_type_node.is_none() {
            sig.resolved_return_type = Some(self.get_type_of_node(return_type_node));
        } else {
            sig.resolved_return_type = Some(self.types.any_type);
        }

        sig
    }

    /// Get the type of a property access expression (obj.prop).
    fn get_type_of_property_access(&mut self, expression: NodeIndex, name: NodeIndex) -> TypeId {
        use crate::parser::Node;

        // Get the type of the expression
        let expr_type = self.get_type_of_node(expression);

        // Get the property name
        let prop_name = if let Some(Node::Identifier(id)) = self.node_arena.get(name) {
            id.escaped_text.clone()
        } else {
            return self.types.any_type;
        };

        // Check for 'any' or 'unknown' type - no need to check properties
        if let Some(typ) = self.types.get(expr_type) {
            if typ.has_flags(type_flags::ANY) {
                return self.types.any_type;
            }
            if typ.has_flags(type_flags::UNKNOWN) {
                self.error(
                    name,
                    "Object is of type 'unknown'.",
                    diagnostic_codes::OBJECT_IS_OF_TYPE_UNKNOWN
                );
                return self.types.any_type;
            }
        }

        // Look up the property on the expression type
        let (prop_type, found) = self.get_property_type_with_check(expr_type, &prop_name);

        // Report error if property not found
        if !found && expr_type != self.types.any_type {
            let type_str = self.type_to_string(expr_type);
            self.error(
                name,
                &format!("Property '{}' does not exist on type '{}'.", prop_name, type_str),
                diagnostic_codes::PROPERTY_DOES_NOT_EXIST_ON_TYPE
            );
        }

        // Check visibility (private/protected) if property was found
        if found {
            if let Some(prop_symbol) = self.get_property_symbol(expr_type, &prop_name) {
                self.check_property_visibility(prop_symbol, &prop_name, name);
            }
        }

        prop_type
    }

    /// Get the type of a property on an object type, with a flag indicating if it was found.
    fn get_property_type_with_check(&mut self, object_type: TypeId, prop_name: &str) -> (TypeId, bool) {
        let Some(typ) = self.types.get(object_type) else {
            return (self.types.any_type, false);
        };

        match typ {
            Type::Object(obj) => {
                // Look up property in object's members
                let members_clone = obj.members.clone();
                if let Some(symbol_id) = members_clone.get(prop_name) {
                    let prop_type = self.symbol_types.get(&symbol_id).copied().unwrap_or(self.types.any_type);
                    return (prop_type, true);
                }
                // Fallback to properties list
                for &prop_id in &obj.properties.clone() {
                    if let Some(sym) = self.get_symbol(prop_id) {
                        if sym.escaped_name == prop_name {
                            let prop_type = self.symbol_types.get(&prop_id).copied().unwrap_or(self.types.any_type);
                            return (prop_type, true);
                        }
                    }
                }
                (self.types.any_type, false)
            }
            Type::Union(union) => {
                // For union types, get the property type from each member and union them
                let member_types = union.types.clone();
                let mut prop_types = Vec::new();
                let mut all_found = true;

                for member in member_types {
                    let (prop_type, found) = self.get_property_type_with_check(member, prop_name);
                    if !found {
                        all_found = false;
                    }
                    if found && !prop_types.contains(&prop_type) {
                        prop_types.push(prop_type);
                    }
                }

                if prop_types.is_empty() {
                    return (self.types.any_type, false);
                }
                if prop_types.len() == 1 {
                    return (prop_types[0], all_found);
                }
                (self.types.create_union(prop_types), all_found)
            }
            Type::Intersection(intersection) => {
                // For intersection types, get the property type from any member that has it
                let member_types = intersection.types.clone();
                for member in member_types {
                    let (prop_type, found) = self.get_property_type_with_check(member, prop_name);
                    if found {
                        return (prop_type, true);
                    }
                }
                (self.types.any_type, false)
            }
            Type::Enum(enum_type) => {
                // For enum types, look up the member by name
                let members = enum_type.members.clone();
                for (member_name, member_type) in members {
                    if member_name == prop_name {
                        return (member_type, true);
                    }
                }
                (self.types.any_type, false)
            }
            _ => (self.types.any_type, false),
        }
    }

    /// Get the type of a property on an object type.
    /// For union types, returns the union of property types from each member.
    fn get_property_type(&mut self, object_type: TypeId, prop_name: &str) -> TypeId {
        // Use the with_check version but ignore the found flag
        let (prop_type, _) = self.get_property_type_with_check(object_type, prop_name);
        prop_type
    }

    /// Get the symbol for a property on an object type (for visibility checks).
    fn get_property_symbol(&self, object_type: TypeId, prop_name: &str) -> Option<SymbolId> {
        let Some(typ) = self.types.get(object_type) else {
            return None;
        };

        match typ {
            Type::Object(obj) => {
                // Look up property in object's members
                if let Some(symbol_id) = obj.members.get(prop_name) {
                    return Some(symbol_id);
                }
                // Fallback to properties list
                for &prop_id in &obj.properties {
                    if let Some(sym) = self.get_symbol(prop_id) {
                        if sym.escaped_name == prop_name {
                            return Some(prop_id);
                        }
                    }
                }
                None
            }
            _ => None,
        }
    }

    /// Check visibility of a property access and report error if not allowed.
    /// Returns true if access is allowed, false otherwise.
    fn check_property_visibility(
        &mut self,
        prop_symbol: SymbolId,
        prop_name: &str,
        error_node: NodeIndex,
    ) -> bool {
        use crate::binder::symbol_flags;

        let Some(sym) = self.get_symbol(prop_symbol) else {
            return true; // If we can't find the symbol, allow access
        };

        let is_private = (sym.flags & symbol_flags::PRIVATE) != 0;
        let is_protected = (sym.flags & symbol_flags::PROTECTED) != 0;

        if !is_private && !is_protected {
            return true; // Public access is always allowed
        }

        // Private: only accessible within the same class
        if is_private {
            if self.enclosing_class.is_none() {
                self.error(
                    error_node,
                    &format!("Property '{}' is private and only accessible within the class.", prop_name),
                    diagnostic_codes::PROPERTY_IS_PRIVATE
                );
                return false;
            }
            // For now, we allow access if we're inside any class
            // A more complete check would verify it's the same class that declares the property
        }

        // Protected: accessible within the class and derived classes
        if is_protected {
            if self.enclosing_class.is_none() {
                self.error(
                    error_node,
                    &format!("Property '{}' is protected and only accessible within the class and its subclasses.", prop_name),
                    diagnostic_codes::PROPERTY_IS_PROTECTED
                );
                return false;
            }
            // For now, we allow access if we're inside any class
            // A more complete check would verify class hierarchy
        }

        true
    }

    /// Get the type of an element access expression (e.g., obj["key"], arr[0]).
    fn get_type_of_element_access(&mut self, expression: NodeIndex, argument: NodeIndex) -> TypeId {
        // Get the type of the expression being indexed
        let expr_type = self.get_type_of_node(expression);

        // Get the type of the index/key
        let index_type = self.get_type_of_node(argument);

        // Try to get the index info from the expression type
        self.get_indexed_access_type(expr_type, index_type)
    }

    /// Get the type resulting from indexing an object type with an index type.
    pub fn get_indexed_access_type(&mut self, object_type: TypeId, index_type: TypeId) -> TypeId {
        let Some(typ) = self.types.get(object_type).cloned() else {
            return self.types.any_type;
        };

        match typ {
            Type::Object(obj) => {
                // First, check if index_type is a string literal - try property lookup
                if let Some(Type::Literal(lit)) = self.types.get(index_type) {
                    if let LiteralValue::String(key_name) = &lit.value {
                        // Look up the property by name in members
                        if let Some(symbol_id) = obj.members.get(key_name) {
                            if let Some(&prop_type) = self.symbol_types.get(&symbol_id) {
                                return prop_type;
                            }
                        }
                    }
                }

                // Check if the object has an applicable index signature
                for index_info in &obj.index_infos {
                    // Check if the index type is assignable to the key type
                    if self.is_type_assignable_to(index_type, index_info.key_type) {
                        return index_info.value_type;
                    }
                }
                // No matching index signature, return any
                self.types.any_type
            }
            Type::Array(arr) => {
                // For arrays, if indexed with number, return element type
                let index_typ = self.types.get(index_type);
                let is_number_index = match index_typ {
                    Some(Type::Intrinsic(intrinsic)) => intrinsic.intrinsic_name == "number",
                    Some(Type::Literal(lit)) => matches!(lit.value, LiteralValue::Number(_)),
                    _ => false,
                };
                if is_number_index {
                    return arr.element_type;
                }
                self.types.any_type
            }
            Type::Tuple(tuple) => {
                // For tuples, check if we have a number literal index
                if let Some(Type::Literal(lit)) = self.types.get(index_type) {
                    if let LiteralValue::Number(n) = &lit.value {
                        let idx = *n as usize;
                        if idx < tuple.element_types.len() {
                            return tuple.element_types[idx];
                        }
                        // Out of bounds - return undefined
                        return self.types.undefined_type;
                    }
                }
                // Check if index is number type (not literal)
                let is_number_type = if let Some(Type::Intrinsic(intrinsic)) = self.types.get(index_type) {
                    intrinsic.intrinsic_name == "number"
                } else {
                    false
                };
                if is_number_type {
                    // Return union of all element types
                    if tuple.element_types.is_empty() {
                        return self.types.never_type;
                    }
                    if tuple.element_types.len() == 1 {
                        return tuple.element_types[0];
                    }
                    return self.types.create_union(tuple.element_types.clone());
                }
                self.types.any_type
            }
            Type::Union(union) => {
                // For union types, get indexed access from each member and union the results
                let member_types = union.types.clone();
                let mut result_types = Vec::new();

                for member in member_types {
                    let member_result = self.get_indexed_access_type(member, index_type);
                    if member_result != self.types.any_type && !result_types.contains(&member_result) {
                        result_types.push(member_result);
                    }
                }

                if result_types.is_empty() {
                    return self.types.any_type;
                }
                if result_types.len() == 1 {
                    return result_types[0];
                }
                self.types.create_union(result_types)
            }
            Type::Enum(enum_info) => {
                // Enum reverse mappings: Color[0] returns "Red", Color["Red"] returns 0
                // Check for string literal index - forward mapping
                if let Some(Type::Literal(lit)) = self.types.get(index_type) {
                    match &lit.value {
                        LiteralValue::String(member_name) => {
                            // Forward mapping: Color["Red"] -> 0
                            for (name, value_type) in &enum_info.members {
                                if name == member_name {
                                    return *value_type;
                                }
                            }
                        }
                        LiteralValue::Number(n) => {
                            // Reverse mapping: Color[0] -> "Red" (only for numeric enums)
                            for (name, value_type) in &enum_info.members {
                                if let Some(Type::Literal(val_lit)) = self.types.get(*value_type) {
                                    if let LiteralValue::Number(val) = &val_lit.value {
                                        if (*val as i64) == (*n as i64) {
                                            // Return the member name as a string literal type
                                            return self.types.create_string_literal(name.clone());
                                        }
                                    }
                                }
                            }
                        }
                        _ => {}
                    }
                }
                // For non-literal number type, return string (union of all member names)
                let is_number_type = if let Some(Type::Intrinsic(intrinsic)) = self.types.get(index_type) {
                    intrinsic.intrinsic_name == "number"
                } else {
                    false
                };
                if is_number_type {
                    // Return union of all member name string literals
                    let name_types: Vec<TypeId> = enum_info.members.iter()
                        .map(|(name, _)| self.types.create_string_literal(name.clone()))
                        .collect();
                    if name_types.is_empty() {
                        return self.types.string_type;
                    }
                    if name_types.len() == 1 {
                        return name_types[0];
                    }
                    return self.types.create_union(name_types);
                }
                self.types.any_type
            }
            _ => self.types.any_type,
        }
    }

    /// Get the type of an object literal ({ x: 1, y: "hello" }).
    /// Returns a fresh object literal type that is subject to excess property checks.
    fn get_type_of_object_literal(&mut self, properties: &crate::parser::NodeList) -> TypeId {
        use crate::parser::Node;

        let mut prop_symbols = Vec::new();
        let mut members_table = SymbolTable::new();

        // Get contextual property types if we have a contextual object type
        let contextual_members = self.get_contextual_object_members();

        for &prop_idx in &properties.nodes {
            if let Some(node) = self.node_arena.get(prop_idx) {
                match node {
                    Node::PropertyAssignment(pa) => {
                        // Get property name (supports identifier, string/number literals, computed)
                        let Some(name) = self.get_property_name_text(pa.name) else {
                            continue;
                        };

                        // Get contextual type for this property (enables callback inference)
                        let contextual_prop_type = contextual_members
                            .as_ref()
                            .and_then(|m| m.get(&name).copied());

                        // Get property type from initializer with contextual type
                        let prop_type = if let Some(ctx_type) = contextual_prop_type {
                            let prev_contextual = self.contextual_type;
                            self.contextual_type = Some(ctx_type);
                            let t = self.get_type_of_node(pa.initializer);
                            self.contextual_type = prev_contextual;
                            t
                        } else {
                            self.get_type_of_node(pa.initializer)
                        };

                        // Create a symbol for this property
                        let symbol_id = self.local_symbols_mut().alloc(symbol_flags::PROPERTY, name.clone());

                        // Cache the symbol's type
                        self.symbol_types.insert(symbol_id, prop_type);
                        prop_symbols.push(symbol_id);
                        members_table.set(name, symbol_id);
                    }
                    Node::ShorthandPropertyAssignment(spa) => {
                        // Get property name (only identifiers valid for shorthand)
                        let Some(name) = self.get_property_name_text(spa.name) else {
                            continue;
                        };

                        // Get property type from the name identifier (which should resolve to a variable)
                        let prop_type = self.get_type_of_node(spa.name);

                        // Create a symbol for this property
                        let symbol_id = self.local_symbols_mut().alloc(symbol_flags::PROPERTY, name.clone());

                        // Cache the symbol's type
                        self.symbol_types.insert(symbol_id, prop_type);
                        prop_symbols.push(symbol_id);
                        members_table.set(name, symbol_id);
                    }
                    Node::MethodDeclaration(md) => {
                        // Handle method declarations in object literals
                        let Some(name) = self.get_property_name_text(md.name) else {
                            continue;
                        };

                        // Get contextual type for this method
                        let contextual_method_type = contextual_members
                            .as_ref()
                            .and_then(|m| m.get(&name).copied());

                        // Get method type with contextual type
                        let method_type = if let Some(ctx_type) = contextual_method_type {
                            let prev_contextual = self.contextual_type;
                            self.contextual_type = Some(ctx_type);
                            let t = self.get_type_of_function_like(prop_idx, &md.parameters, md.type_annotation);
                            self.contextual_type = prev_contextual;
                            t
                        } else {
                            self.get_type_of_function_like(prop_idx, &md.parameters, md.type_annotation)
                        };

                        // Create a symbol for this method
                        let symbol_id = self.local_symbols_mut().alloc(symbol_flags::METHOD, name.clone());

                        // Cache the symbol's type
                        self.symbol_types.insert(symbol_id, method_type);
                        prop_symbols.push(symbol_id);
                        members_table.set(name, symbol_id);
                    }
                    _ => {}
                }
            }
        }

        // Create fresh object literal type (subject to excess property checks)
        self.types.create_fresh_object_literal_type(prop_symbols, members_table)
    }

    /// Get the member types from the contextual type if it's an object type.
    /// Returns a map from property name to property type.
    fn get_contextual_object_members(&self) -> Option<std::collections::HashMap<String, TypeId>> {
        let ctx_type_id = self.contextual_type?;
        let ctx_type = self.types.get(ctx_type_id)?;

        match ctx_type {
            Type::Object(obj) => {
                let mut members = std::collections::HashMap::new();
                // Use both properties list and members table for completeness
                for &sym_id in &obj.properties {
                    if let Some(sym) = self.get_symbol(sym_id) {
                        if let Some(&prop_type) = self.symbol_types.get(&sym_id) {
                            members.insert(sym.escaped_name.clone(), prop_type);
                        }
                    }
                }
                // Also check members table for interface-like objects
                for (name, &sym_id) in obj.members.iter() {
                    if let Some(&prop_type) = self.symbol_types.get(&sym_id) {
                        members.insert(name.clone(), prop_type);
                    }
                }
                Some(members)
            }
            Type::Function(f) => {
                // Function types can also provide contextual typing via their parameter types
                // e.g., for a callback like (x: number) => void passed to object literal property
                None // Function itself doesn't have object members
            }
            _ => None,
        }
    }

    /// Get a mutable reference to the local symbol arena (for creating new symbols during type checking).
    fn local_symbols_mut(&mut self) -> &mut SymbolArena {
        &mut self.local_symbols
    }

    /// Look up a symbol by ID in both binder and local symbols.
    fn get_symbol(&self, id: SymbolId) -> Option<&crate::binder::Symbol> {
        // Check local_symbols first to avoid ID collision with binder symbols
        self.local_symbols.get(id).or_else(|| self.symbol_arena.get(id))
    }

    /// Get the name string from a property name node.
    /// Returns None for computed property names that can't be statically resolved.
    fn get_property_name_text(&self, name_idx: NodeIndex) -> Option<String> {
        use crate::parser::Node;

        let Some(node) = self.node_arena.get(name_idx) else {
            return None;
        };

        match node {
            Node::Identifier(id) => Some(id.escaped_text.clone()),
            Node::StringLiteral(sl) => Some(sl.text.clone()),
            Node::NumericLiteral(nl) => Some(nl.text.clone()),
            Node::ComputedPropertyName { expression, .. } => {
                // For computed property names, check if the expression is a string literal
                if let Some(Node::StringLiteral(sl)) = self.node_arena.get(*expression) {
                    Some(sl.text.clone())
                } else if let Some(Node::NumericLiteral(nl)) = self.node_arena.get(*expression) {
                    Some(nl.text.clone())
                } else {
                    // Dynamic computed property - for now return None
                    // In a full implementation, we'd create an index signature
                    None
                }
            }
            _ => None,
        }
    }

    /// Get the type of a function-like declaration (function, method, arrow, etc.)
    fn get_type_of_function_like(
        &mut self,
        declaration: NodeIndex,
        parameters: &crate::parser::NodeList,
        return_type_annotation: NodeIndex,
    ) -> TypeId {
        self.get_type_of_function_like_with_type_params(
            declaration,
            parameters,
            return_type_annotation,
            None,
        )
    }

    /// Get the type of a function-like declaration with type parameters.
    fn get_type_of_function_like_with_type_params(
        &mut self,
        declaration: NodeIndex,
        parameters: &crate::parser::NodeList,
        return_type_annotation: NodeIndex,
        type_parameters: Option<&crate::parser::NodeList>,
    ) -> TypeId {
        use crate::parser::Node;

        // Track the type parameter names we add and their previous values for restoration.
        // This handles shadowing correctly: `function outer<T>() { function inner<T>() { } }`
        // After inner finishes, we restore outer's T rather than removing it entirely.
        let mut shadowed_params: Vec<(String, Option<TypeId>)> = Vec::new();

        // Create type parameters and add them to the scope
        let type_param_ids: Vec<TypeId> = if let Some(type_params) = type_parameters {
            type_params.nodes.iter()
                .filter_map(|&tp_idx| {
                    let type_id = self.create_type_parameter(tp_idx)?;
                    // Add to type parameter scope for name lookup during signature processing
                    if let Some(name) = self.type_parameter_names.get(&type_id) {
                        // Save the previous value (if any) so we can restore it later
                        let prev_value = self.type_parameter_scope.insert(name.clone(), type_id);
                        shadowed_params.push((name.clone(), prev_value));
                    }
                    Some(type_id)
                })
                .collect()
        } else {
            Vec::new()
        };

        // Collect parameter types and names
        let mut param_types = Vec::new();
        let mut param_names = Vec::new();
        let mut min_arg_count = 0u32;
        let mut has_rest = false;
        let mut this_type: Option<TypeId> = None;

        // Get contextual parameter types if available
        let contextual_param_types: Option<Vec<TypeId>> = self.contextual_type
            .and_then(|ctx| {
                if let Some(Type::Function(f)) = self.types.get(ctx) {
                    Some(f.parameter_types.clone())
                } else {
                    None
                }
            });

        for (param_index, &param_idx) in parameters.nodes.iter().enumerate() {
            if let Some(Node::ParameterDeclaration(param)) = self.node_arena.get(param_idx) {
                // Get parameter name
                let name = if let Some(Node::Identifier(id)) = self.node_arena.get(param.name) {
                    id.escaped_text.clone()
                } else {
                    String::new()
                };

                // Handle `this` parameter: function foo(this: SomeType, ...)
                // The `this` parameter must be the first parameter and is not a regular parameter
                if name == "this" && param_index == 0 {
                    if !param.type_annotation.is_none() {
                        this_type = Some(self.get_type_of_node(param.type_annotation));
                    }
                    // Skip adding to regular parameters - `this` is special
                    continue;
                }

                param_names.push(name);

                // Get parameter type
                // Adjust param_index for contextual types if we skipped a `this` parameter
                let effective_param_index = if this_type.is_some() { param_index - 1 } else { param_index };
                let param_type = if !param.type_annotation.is_none() {
                    // Explicit type annotation takes precedence
                    self.get_type_of_node(param.type_annotation)
                } else if let Some(ref ctx_params) = contextual_param_types {
                    // Use contextual type if available
                    ctx_params.get(effective_param_index).copied().unwrap_or(self.types.any_type)
                } else if !param.initializer.is_none() {
                    // Infer from initializer
                    self.get_type_of_node(param.initializer)
                } else {
                    self.types.any_type
                };
                param_types.push(param_type);

                // Track min argument count and rest parameter
                if param.dot_dot_dot_token {
                    has_rest = true;
                } else if !param.question_token && param.initializer.is_none() {
                    min_arg_count += 1;
                }
            }
        }

        // Get return type
        let return_type = if !return_type_annotation.is_none() {
            self.get_type_of_node(return_type_annotation)
        } else {
            // Return type inference would happen here
            // For now, default to any
            self.types.any_type
        };

        // Restore the previous type parameter scope entries (handles shadowing correctly)
        for (name, prev_value) in shadowed_params {
            if let Some(prev_id) = prev_value {
                // Restore the shadowed outer scope's type parameter
                self.type_parameter_scope.insert(name, prev_id);
            } else {
                // No previous value existed, so remove the entry
                self.type_parameter_scope.remove(&name);
            }
        }

        self.types.create_function_type_with_this(
            declaration,
            param_types,
            param_names,
            return_type,
            type_param_ids,
            min_arg_count,
            has_rest,
            this_type,
        )
    }

    /// Create a TypeParameter from a TypeParameterDeclaration node.
    fn create_type_parameter(&mut self, node: NodeIndex) -> Option<TypeId> {
        use crate::parser::Node;
        use crate::scanner::SyntaxKind;

        let tp = match self.node_arena.get(node)? {
            Node::TypeParameterDeclaration(tp) => tp,
            _ => return None,
        };

        // Get the name of the type parameter
        let name = if let Some(Node::Identifier(id)) = self.node_arena.get(tp.name) {
            id.escaped_text.clone()
        } else {
            return None;
        };

        // Check for 'const' modifier in type parameter
        let is_const = if let Some(ref modifiers) = tp.modifiers {
            modifiers.nodes.iter().any(|&mod_idx| {
                if let Some(Node::Token(token_base)) = self.node_arena.get(mod_idx) {
                    token_base.kind == SyntaxKind::ConstKeyword as u16
                } else {
                    false
                }
            })
        } else {
            false
        };

        // Create a symbol for the type parameter
        let symbol_id = self.local_symbols.alloc(symbol_flags::TYPE_PARAMETER, name.clone());

        // Get constraint type if present
        let constraint = if !tp.constraint.is_none() {
            self.get_type_of_node(tp.constraint)
        } else {
            TypeId::NONE
        };

        // Get default type if present
        let default = if !tp.default.is_none() {
            self.get_type_of_node(tp.default)
        } else {
            TypeId::NONE
        };

        // Create and allocate the type parameter
        let type_param = TypeParameter {
            flags: type_flags::TYPE_PARAMETER,
            symbol: symbol_id,
            constraint,
            default,
            target: TypeId::NONE,
            is_this_type: false,
            is_const,
        };

        // Store the name for type_to_string
        let type_id = self.types.alloc(Type::TypeParameter(Box::new(type_param)));

        // Cache the type parameter name for later lookup
        self.type_parameter_names.insert(type_id, name);

        Some(type_id)
    }

    /// Instantiate a generic type with type arguments.
    /// Replaces type parameters with the provided type arguments.
    pub fn instantiate_type(&mut self, type_id: TypeId, type_arguments: &[TypeId], type_parameters: &[TypeId]) -> TypeId {
        // If no type arguments, return the original type
        if type_arguments.is_empty() || type_parameters.is_empty() {
            return type_id;
        }

        // Create a mapping from type parameters to type arguments
        let mapper: std::collections::HashMap<TypeId, TypeId> = type_parameters.iter()
            .zip(type_arguments.iter())
            .map(|(&param, &arg)| (param, arg))
            .collect();

        self.instantiate_type_with_mapper(type_id, &mapper)
    }

    /// Instantiate a type using a type parameter mapper.
    fn instantiate_type_with_mapper(&mut self, type_id: TypeId, mapper: &std::collections::HashMap<TypeId, TypeId>) -> TypeId {
        use super::state::MAX_INSTANTIATION_DEPTH;

        // Check instantiation depth to prevent infinite recursion
        {
            let depth = *self.instantiation_depth.borrow();
            if depth >= MAX_INSTANTIATION_DEPTH {
                // Type instantiation depth limit reached - return error type
                return self.types.any_type;
            }
            *self.instantiation_depth.borrow_mut() = depth + 1;
        }

        let result = self.instantiate_type_with_mapper_inner(type_id, mapper);

        // Decrement depth counter
        {
            let mut depth = self.instantiation_depth.borrow_mut();
            *depth = depth.saturating_sub(1);
        }

        result
    }

    /// Inner implementation of type instantiation (without depth tracking).
    fn instantiate_type_with_mapper_inner(&mut self, type_id: TypeId, mapper: &std::collections::HashMap<TypeId, TypeId>) -> TypeId {
        // Check if this type parameter is in the mapper
        if let Some(&mapped_type) = mapper.get(&type_id) {
            return mapped_type;
        }

        // Extract data from the type to avoid borrowing issues
        enum TypeInfo {
            TypeParameter,
            Function {
                declaration: NodeIndex,
                parameter_types: Vec<TypeId>,
                parameter_names: Vec<String>,
                return_type: TypeId,
                min_argument_count: u32,
                has_rest_parameter: bool,
            },
            Union {
                types: Vec<TypeId>,
            },
            Intersection {
                types: Vec<TypeId>,
            },
            Mapped {
                type_parameter: TypeId,
                constraint_type: TypeId,
                template_type: TypeId,
            },
            IndexedAccess {
                object_type: TypeId,
                index_type: TypeId,
            },
            Index {
                source_type: TypeId,
            },
            Conditional {
                check_type: TypeId,
                extends_type: TypeId,
                true_type: TypeId,
                false_type: TypeId,
                is_distributive: bool,
            },
            Array {
                element_type: TypeId,
                is_readonly: bool,
            },
            Other,
        }

        let type_info = match self.types.get(type_id) {
            Some(Type::TypeParameter(_)) => TypeInfo::TypeParameter,
            Some(Type::Function(f)) => TypeInfo::Function {
                declaration: f.declaration,
                parameter_types: f.parameter_types.clone(),
                parameter_names: f.parameter_names.clone(),
                return_type: f.return_type,
                min_argument_count: f.min_argument_count,
                has_rest_parameter: f.has_rest_parameter,
            },
            Some(Type::Union(u)) => TypeInfo::Union {
                types: u.types.clone(),
            },
            Some(Type::Intersection(i)) => TypeInfo::Intersection {
                types: i.types.clone(),
            },
            Some(Type::Mapped(m)) => TypeInfo::Mapped {
                type_parameter: m.type_parameter,
                constraint_type: m.constraint_type,
                template_type: m.template_type,
            },
            Some(Type::IndexedAccess(ia)) => TypeInfo::IndexedAccess {
                object_type: ia.object_type,
                index_type: ia.index_type,
            },
            Some(Type::Index(i)) => TypeInfo::Index {
                source_type: i.source_type,
            },
            Some(Type::Conditional(c)) => TypeInfo::Conditional {
                check_type: c.check_type,
                extends_type: c.extends_type,
                true_type: c.true_type,
                false_type: c.false_type,
                is_distributive: c.is_distributive,
            },
            Some(Type::Array(arr)) => TypeInfo::Array {
                element_type: arr.element_type,
                is_readonly: arr.is_readonly,
            },
            Some(_) => TypeInfo::Other,
            None => return type_id,
        };

        match type_info {
            // Type parameter - already checked above
            TypeInfo::TypeParameter => type_id,

            // Function type - instantiate return type and parameter types
            TypeInfo::Function {
                declaration,
                parameter_types,
                parameter_names,
                return_type,
                min_argument_count,
                has_rest_parameter,
            } => {
                let new_param_types: Vec<TypeId> = parameter_types.iter()
                    .map(|&pt| self.instantiate_type_with_mapper(pt, mapper))
                    .collect();
                let new_return_type = self.instantiate_type_with_mapper(return_type, mapper);

                // Check if anything changed
                if new_param_types == parameter_types && new_return_type == return_type {
                    return type_id;
                }

                self.types.create_function_type(
                    declaration,
                    new_param_types,
                    parameter_names,
                    new_return_type,
                    min_argument_count,
                    has_rest_parameter,
                )
            }

            // Union type - instantiate each constituent
            TypeInfo::Union { types } => {
                let new_types: Vec<TypeId> = types.iter()
                    .map(|&t| self.instantiate_type_with_mapper(t, mapper))
                    .collect();

                if new_types == types {
                    return type_id;
                }

                self.types.create_union_type(new_types)
            }

            // Intersection type - instantiate each constituent
            TypeInfo::Intersection { types } => {
                let new_types: Vec<TypeId> = types.iter()
                    .map(|&t| self.instantiate_type_with_mapper(t, mapper))
                    .collect();

                if new_types == types {
                    return type_id;
                }

                self.types.create_intersection(new_types)
            }

            // Mapped type - instantiate to concrete object type
            TypeInfo::Mapped { type_parameter, constraint_type, template_type } => {
                // First, instantiate the constraint type to get the concrete keys
                let instantiated_constraint = self.instantiate_type_with_mapper(constraint_type, mapper);

                // Get the keys from the instantiated constraint
                let keys = self.get_keys_from_type(instantiated_constraint);

                if keys.is_empty() {
                    // If no keys, return empty object type
                    return self.types.create_object_type(Vec::new());
                }

                // For each key, instantiate the template type with K bound to that key
                let mut properties = Vec::new();
                let mut members_table = SymbolTable::new();

                for key_name in keys {
                    // Create a string literal type for this key
                    let key_type = self.types.create_string_literal(key_name.clone());

                    // Create a new mapper with the type parameter bound to this key
                    let mut inner_mapper = mapper.clone();
                    inner_mapper.insert(type_parameter, key_type);

                    // Instantiate the template type with the key bound
                    let property_type = self.instantiate_type_with_mapper(template_type, &inner_mapper);

                    // Create a symbol for this property
                    let symbol_id = self.local_symbols_mut().alloc(symbol_flags::PROPERTY, key_name.clone());
                    self.symbol_types.insert(symbol_id, property_type);
                    properties.push(symbol_id);
                    members_table.set(key_name, symbol_id);
                }

                self.types.create_object_type_with_members(properties, members_table)
            }

            // Indexed access type - instantiate object and index types
            TypeInfo::IndexedAccess { object_type, index_type } => {
                let new_object = self.instantiate_type_with_mapper(object_type, mapper);
                let new_index = self.instantiate_type_with_mapper(index_type, mapper);

                if new_object == object_type && new_index == index_type {
                    return type_id;
                }

                // Try to resolve the indexed access
                self.get_indexed_access_type(new_object, new_index)
            }

            // Index type (keyof) - instantiate the source type
            TypeInfo::Index { source_type } => {
                let new_source = self.instantiate_type_with_mapper(source_type, mapper);

                if new_source == source_type {
                    return type_id;
                }

                // Compute keyof for the new source type
                self.get_keyof_type(new_source)
            }

            // Conditional type - instantiate and evaluate
            TypeInfo::Conditional { check_type, extends_type, true_type, false_type, is_distributive } => {
                let new_check = self.instantiate_type_with_mapper(check_type, mapper);
                let new_extends = self.instantiate_type_with_mapper(extends_type, mapper);
                let new_true = self.instantiate_type_with_mapper(true_type, mapper);
                let new_false = self.instantiate_type_with_mapper(false_type, mapper);

                // Check if the check type still contains type parameters
                if self.type_contains_type_parameter(new_check) {
                    // Still deferred - create new conditional type
                    if new_check == check_type && new_extends == extends_type
                        && new_true == true_type && new_false == false_type {
                        return type_id;
                    }
                    return self.types.create_conditional_type(new_check, new_extends, new_true, new_false);
                }

                // Handle distributive conditional types over unions
                // If the original conditional was distributive and new_check is a union,
                // distribute the conditional over each union member
                if is_distributive {
                    if let Some(Type::Union(union)) = self.types.get(new_check) {
                        let member_types = union.types.clone();
                        let mut result_types = Vec::new();

                        for member in member_types {
                            // Check if extends type contains infer types
                            if self.type_contains_infer(new_extends) {
                                let mut inferences: std::collections::HashMap<TypeId, TypeId> = std::collections::HashMap::new();
                                if self.infer_from_type(member, new_extends, &mut inferences) {
                                    let result = self.instantiate_type_with_mapper(new_true, &inferences);
                                    if !result_types.contains(&result) {
                                        result_types.push(result);
                                    }
                                } else {
                                    if !result_types.contains(&new_false) {
                                        result_types.push(new_false);
                                    }
                                }
                            } else {
                                // Simple assignability check
                                let result = if self.is_type_assignable_to(member, new_extends) {
                                    new_true
                                } else {
                                    new_false
                                };
                                if !result_types.contains(&result) {
                                    result_types.push(result);
                                }
                            }
                        }

                        if result_types.is_empty() {
                            return self.types.never_type;
                        }
                        if result_types.len() == 1 {
                            return result_types[0];
                        }
                        return self.types.create_union(result_types);
                    }
                }

                // Check if extends type contains infer types
                if self.type_contains_infer(new_extends) {
                    let mut inferences: std::collections::HashMap<TypeId, TypeId> = std::collections::HashMap::new();
                    if self.infer_from_type(new_check, new_extends, &mut inferences) {
                        // Pattern matched - substitute inferences in true branch
                        return self.instantiate_type_with_mapper(new_true, &inferences);
                    } else {
                        return new_false;
                    }
                }

                // Evaluate the condition
                if self.is_type_assignable_to(new_check, new_extends) {
                    new_true
                } else {
                    new_false
                }
            }

            // Array type - instantiate element type
            TypeInfo::Array { element_type, is_readonly } => {
                let new_element = self.instantiate_type_with_mapper(element_type, mapper);

                if new_element == element_type {
                    return type_id;
                }

                self.types.create_array_type(new_element, is_readonly)
            }

            // Other types - return as-is for now
            TypeInfo::Other => type_id,
        }
    }

    /// Get property names from a type (for mapped type instantiation).
    fn get_keys_from_type(&mut self, type_id: TypeId) -> Vec<String> {
        let mut keys = Vec::new();

        let Some(typ) = self.types.get(type_id) else {
            return keys;
        };

        match typ {
            // Union of string literals
            Type::Union(u) => {
                let types = u.types.clone();
                for t in types {
                    if let Some(Type::Literal(lit)) = self.types.get(t) {
                        if let LiteralValue::String(s) = &lit.value {
                            keys.push(s.clone());
                        }
                    }
                }
            }
            // Single string literal
            Type::Literal(lit) => {
                if let LiteralValue::String(s) = &lit.value {
                    keys.push(s.clone());
                }
            }
            // Object type - get property names
            Type::Object(obj) => {
                for (name, _) in obj.members.iter() {
                    keys.push(name.clone());
                }
            }
            _ => {}
        }

        keys
    }

    /// Get the type of a type reference with type arguments (e.g., Array<T>, Map<K, V>).
    fn get_type_of_type_reference_with_args(&mut self, type_name: NodeIndex, type_arguments: &crate::parser::NodeList) -> TypeId {
        use crate::parser::Node;

        // Get the base type name
        let name = if let Some(Node::Identifier(id)) = self.node_arena.get(type_name) {
            id.escaped_text.as_str()
        } else {
            return self.types.object_type;
        };

        // Resolve type arguments
        let type_args: Vec<TypeId> = type_arguments.nodes.iter()
            .map(|&arg| self.get_type_of_node(arg))
            .collect();

        // Look up the type in the symbol table
        if let Some(symbol_id) = self.file_locals.get(name) {
            // Check if this is a type alias with type parameters
            if let Some(symbol) = self.symbol_arena.get(symbol_id) {
                if symbol.has_flags(crate::binder::symbol_flags::TYPE_ALIAS) {
                    if let Some(&decl_idx) = symbol.declarations.first() {
                        if let Some(Node::TypeAliasDeclaration(ta)) = self.node_arena.get(decl_idx) {
                            if let Some(ref type_params) = ta.type_parameters {
                                // Collect type parameter TypeIds
                                let mut param_type_ids = Vec::new();

                                // Save current type parameter scope
                                let saved_scope = std::mem::take(&mut self.type_parameter_scope);

                                // Create type parameters and build the scope
                                for &tp_idx in &type_params.nodes {
                                    if let Some(type_id) = self.create_type_parameter(tp_idx) {
                                        param_type_ids.push(type_id);
                                        if let Some(name) = self.type_parameter_names.get(&type_id) {
                                            self.type_parameter_scope.insert(name.clone(), type_id);
                                        }
                                    }
                                }

                                // Get the body type with type parameters in scope
                                let body_type = self.get_type_of_node(ta.type_node);

                                // Restore scope
                                self.type_parameter_scope = saved_scope;

                                // Instantiate with the provided type arguments
                                if !param_type_ids.is_empty() && !type_args.is_empty() {
                                    return self.instantiate_type(body_type, &type_args, &param_type_ids);
                                }

                                return body_type;
                            }
                        }
                    }
                }
            }

            let base_type = self.get_type_of_symbol(symbol_id);

            // Check if it's a generic function type that needs instantiation
            if let Some(Type::Function(f)) = self.types.get(base_type) {
                if !f.type_parameters.is_empty() {
                    return self.instantiate_type(base_type, &type_args, &f.type_parameters.clone());
                }
            }

            // For classes, a type reference like `let x: MyClass` should resolve to the
            // instance type, not the constructor type. The constructor type has construct
            // signatures that return the instance type.
            if let Some(Type::Object(obj)) = self.types.get(base_type) {
                if obj.has_object_flags(object_flags::CLASS) && !obj.construct_signatures.is_empty() {
                    if let Some(instance_type) = obj.construct_signatures[0].resolved_return_type {
                        return instance_type;
                    }
                }
            }

            return base_type;
        }

        // Built-in generic types
        match name {
            "Array" => {
                // Array<T> becomes a mutable array type with element type T
                let element_type = type_args.first().copied().unwrap_or(self.types.any_type);
                self.types.create_array_type(element_type, false)
            }
            "ReadonlyArray" => {
                // ReadonlyArray<T> becomes a readonly array type with element type T
                let element_type = type_args.first().copied().unwrap_or(self.types.any_type);
                self.types.create_array_type(element_type, true)
            }
            "Promise" => {
                self.types.object_type
            }
            "Map" | "Set" | "WeakMap" | "WeakSet" => {
                self.types.object_type
            }
            // String manipulation types (intrinsic)
            "Uppercase" => {
                let arg_type = type_args.first().copied().unwrap_or(self.types.string_type);
                self.apply_string_mapping(arg_type, StringMappingKind::Uppercase)
            }
            "Lowercase" => {
                let arg_type = type_args.first().copied().unwrap_or(self.types.string_type);
                self.apply_string_mapping(arg_type, StringMappingKind::Lowercase)
            }
            "Capitalize" => {
                let arg_type = type_args.first().copied().unwrap_or(self.types.string_type);
                self.apply_string_mapping(arg_type, StringMappingKind::Capitalize)
            }
            "Uncapitalize" => {
                let arg_type = type_args.first().copied().unwrap_or(self.types.string_type);
                self.apply_string_mapping(arg_type, StringMappingKind::Uncapitalize)
            }
            // Awaited<T> - recursively unwraps Promise types
            "Awaited" => {
                let arg_type = type_args.first().copied().unwrap_or(self.types.any_type);
                self.get_awaited_type(arg_type)
            }
            // Partial<T>, Required<T>, Readonly<T> - return object type for now
            "Partial" | "Required" | "Readonly" | "Record" => {
                self.types.object_type
            }
            // Pick, Omit, Exclude, Extract - return object type for now
            "Pick" | "Omit" | "Exclude" | "Extract" => {
                self.types.object_type
            }
            // NonNullable<T> - remove null and undefined
            "NonNullable" => {
                let arg_type = type_args.first().copied().unwrap_or(self.types.any_type);
                self.get_non_nullable_type(arg_type)
            }
            // NoInfer<T> - prevents type inference, returns T unchanged
            "NoInfer" => {
                type_args.first().copied().unwrap_or(self.types.any_type)
            }
            // ReturnType<T>, Parameters<T>, InstanceType<T>, ConstructorParameters<T>
            "ReturnType" | "Parameters" | "InstanceType" | "ConstructorParameters" => {
                self.types.any_type
            }
            // ThisType<T> - marker type for 'this' in object literal methods
            "ThisType" => {
                let constraint = type_args.first().copied().unwrap_or(self.types.any_type);
                self.types.create_this_type(constraint)
            }
            _ => self.types.object_type,
        }
    }

    /// Get the 'this' type for the current context.
    /// Returns the type of the enclosing class if inside a class method,
    /// or 'any' if not inside a class.
    fn get_this_type(&mut self) -> TypeId {
        // Check if we're inside a class
        if let Some(class_idx) = self.enclosing_class {
            // Get the type of the enclosing class declaration
            // We need to get the declared type of the class itself
            if let Some(crate::parser::Node::ClassDeclaration(cd)) = self.node_arena.get(class_idx) {
                // Get the class name
                if let Some(crate::parser::Node::Identifier(id)) = self.node_arena.get(cd.name) {
                    // Look up the class symbol
                    if let Some(symbol_id) = self.file_locals.get(&id.escaped_text) {
                        return self.get_type_of_symbol(symbol_id);
                    }
                }
            }
        }
        // Not inside a class, return any
        self.types.any_type
    }

    /// Get the 'super' type for the current context.
    /// Returns the type of the base class if inside a class that extends another,
    /// or 'any' if not inside a class or the class has no base class.
    fn get_super_type(&mut self) -> TypeId {
        // Check if we're inside a class
        if let Some(class_idx) = self.enclosing_class {
            // Get the class declaration
            if let Some(crate::parser::Node::ClassDeclaration(cd)) = self.node_arena.get(class_idx) {
                // Look for base class in heritage clauses
                if let Some(base_symbol) = self.get_base_class_symbol(&cd.heritage_clauses) {
                    return self.get_type_of_symbol(base_symbol);
                }
            }
        }
        // Not inside a class or no base class, return any
        self.types.any_type
    }

    /// Get the type of a symbol (with caching).
    pub fn get_type_of_symbol(&mut self, symbol_id: SymbolId) -> TypeId {
        // Check cache first
        if let Some(&cached) = self.symbol_types.get(&symbol_id) {
            return cached;
        }

        // Track recursion to catch infinite loops (O(1) lookup using HashSet)
        if self.symbol_resolution_set.contains(&symbol_id) {
            // Circular reference - return any_type to break the loop
            return self.types.any_type;
        }
        self.symbol_resolution_stack.push(symbol_id);
        self.symbol_resolution_set.insert(symbol_id);

        let type_id = self.get_type_of_symbol_worker(symbol_id);
        self.symbol_types.insert(symbol_id, type_id);

        self.symbol_resolution_stack.pop();
        self.symbol_resolution_set.remove(&symbol_id);
        type_id
    }

    /// Get type of symbol (worker, no caching).
    fn get_type_of_symbol_worker(&mut self, symbol_id: SymbolId) -> TypeId {
        use crate::binder::symbol_flags;

        let Some(symbol) = self.symbol_arena.get(symbol_id) else {
            return self.types.any_type;
        };

        // For type aliases, use the first declaration
        if symbol.has_flags(symbol_flags::TYPE_ALIAS) {
            if let Some(&decl) = symbol.declarations.first() {
                return self.get_type_of_node(decl);
            }
        }

        // Get type from value declaration
        if !symbol.value_declaration.is_none() {
            return self.get_type_of_node(symbol.value_declaration);
        }

        // Fallback: try first declaration
        if let Some(&decl) = symbol.declarations.first() {
            return self.get_type_of_node(decl);
        }

        self.types.any_type
    }

    /// Get the type of a binary expression.
    fn get_type_of_binary_expression(&mut self, be: &crate::parser::BinaryExpression) -> TypeId {
        use crate::scanner::SyntaxKind;

        let left_type = self.get_type_of_node(be.left);
        let right_type = self.get_type_of_node(be.right);

        match be.operator_token {
            // Arithmetic operators: +, -, *, /, %, **
            SyntaxKind::PlusToken => {
                // + can be string concatenation or numeric addition
                if self.is_type_assignable_to(left_type, self.types.string_type)
                    || self.is_type_assignable_to(right_type, self.types.string_type)
                {
                    self.types.string_type
                } else {
                    self.types.number_type
                }
            }
            SyntaxKind::MinusToken
            | SyntaxKind::AsteriskToken
            | SyntaxKind::SlashToken
            | SyntaxKind::PercentToken
            | SyntaxKind::AsteriskAsteriskToken => {
                self.types.number_type
            }

            // Bitwise operators: &, |, ^, <<, >>, >>>
            SyntaxKind::AmpersandToken
            | SyntaxKind::BarToken
            | SyntaxKind::CaretToken
            | SyntaxKind::LessThanLessThanToken
            | SyntaxKind::GreaterThanGreaterThanToken
            | SyntaxKind::GreaterThanGreaterThanGreaterThanToken => {
                self.types.number_type
            }

            // Comparison operators: <, >, <=, >=
            SyntaxKind::LessThanToken
            | SyntaxKind::GreaterThanToken
            | SyntaxKind::LessThanEqualsToken
            | SyntaxKind::GreaterThanEqualsToken => {
                self.types.boolean_type
            }

            // Equality operators: ==, !=, ===, !==
            SyntaxKind::EqualsEqualsToken
            | SyntaxKind::ExclamationEqualsToken
            | SyntaxKind::EqualsEqualsEqualsToken
            | SyntaxKind::ExclamationEqualsEqualsToken => {
                self.types.boolean_type
            }

            // Logical operators: &&, ||
            SyntaxKind::AmpersandAmpersandToken => {
                // && returns left if falsy, otherwise right
                // Type is union of narrowed left type and right type
                // Simplified: return union of both types
                self.types.create_union(vec![left_type, right_type])
            }
            SyntaxKind::BarBarToken => {
                // || returns left if truthy, otherwise right
                // Simplified: return union of both types
                self.types.create_union(vec![left_type, right_type])
            }

            // Nullish coalescing: ??
            SyntaxKind::QuestionQuestionToken => {
                // ?? returns left if not null/undefined, otherwise right
                let non_null_left = self.get_non_nullable_type(left_type);
                self.types.create_union(vec![non_null_left, right_type])
            }

            // Assignment operators: =, +=, -=, etc.
            SyntaxKind::EqualsToken => {
                // Assignment returns the right-hand side type
                right_type
            }
            SyntaxKind::PlusEqualsToken => {
                // += can be string concatenation or numeric addition
                if self.is_type_assignable_to(left_type, self.types.string_type)
                    || self.is_type_assignable_to(right_type, self.types.string_type)
                {
                    self.types.string_type
                } else {
                    self.types.number_type
                }
            }
            SyntaxKind::MinusEqualsToken
            | SyntaxKind::AsteriskEqualsToken
            | SyntaxKind::SlashEqualsToken
            | SyntaxKind::PercentEqualsToken
            | SyntaxKind::AsteriskAsteriskEqualsToken => {
                self.types.number_type
            }
            SyntaxKind::AmpersandEqualsToken
            | SyntaxKind::BarEqualsToken
            | SyntaxKind::CaretEqualsToken
            | SyntaxKind::LessThanLessThanEqualsToken
            | SyntaxKind::GreaterThanGreaterThanEqualsToken
            | SyntaxKind::GreaterThanGreaterThanGreaterThanEqualsToken => {
                self.types.number_type
            }
            SyntaxKind::AmpersandAmpersandEqualsToken => {
                self.types.create_union(vec![left_type, right_type])
            }
            SyntaxKind::BarBarEqualsToken => {
                self.types.create_union(vec![left_type, right_type])
            }
            SyntaxKind::QuestionQuestionEqualsToken => {
                let non_null_left = self.get_non_nullable_type(left_type);
                self.types.create_union(vec![non_null_left, right_type])
            }

            // instanceof: returns boolean
            SyntaxKind::InstanceOfKeyword => {
                self.types.boolean_type
            }

            // in: returns boolean
            SyntaxKind::InKeyword => {
                self.types.boolean_type
            }

            // Comma operator: returns right operand type
            SyntaxKind::CommaToken => {
                right_type
            }

            _ => self.types.any_type
        }
    }

    // =========================================================================
    // Visibility Helpers
    // =========================================================================

    /// Check if a modifier list contains a private keyword.
    pub fn has_private_modifier(&self, modifiers: &Option<crate::parser::NodeList>) -> bool {
        use crate::parser::Node;
        modifiers.as_ref().map_or(false, |mods| {
            mods.nodes.iter().any(|&mod_idx| {
                if let Some(Node::Token(base)) = self.node_arena.get(mod_idx) {
                    base.kind == SyntaxKind::PrivateKeyword as u16
                } else {
                    false
                }
            })
        })
    }

    /// Check if a modifier list contains a protected keyword.
    pub fn has_protected_modifier(&self, modifiers: &Option<crate::parser::NodeList>) -> bool {
        use crate::parser::Node;
        modifiers.as_ref().map_or(false, |mods| {
            mods.nodes.iter().any(|&mod_idx| {
                if let Some(Node::Token(base)) = self.node_arena.get(mod_idx) {
                    base.kind == SyntaxKind::ProtectedKeyword as u16
                } else {
                    false
                }
            })
        })
    }

    /// Get visibility flags for a symbol based on modifiers.
    pub fn get_visibility_flags(&self, modifiers: &Option<crate::parser::NodeList>) -> u32 {
        let mut flags = 0u32;
        if self.has_private_modifier(modifiers) {
            flags |= symbol_flags::PRIVATE;
        }
        if self.has_protected_modifier(modifiers) {
            flags |= symbol_flags::PROTECTED;
        }
        flags
    }

    /// Check if a type node is the special "const" type reference.
    /// Used to detect "as const" assertions.
    fn is_const_type_reference(&self, type_node: NodeIndex) -> bool {
        use crate::parser::Node;

        if let Some(Node::TypeReference(tr)) = self.node_arena.get(type_node) {
            if let Some(Node::Identifier(id)) = self.node_arena.get(tr.type_name) {
                return id.escaped_text == "const";
            }
        }
        false
    }

    /// Apply a string mapping transformation to a type.
    /// For string literal types, this transforms the value (e.g., "hello" -> "HELLO" for Uppercase).
    /// For non-string-literal types, returns the base string type.
    fn apply_string_mapping(&mut self, type_id: TypeId, kind: StringMappingKind) -> TypeId {
        match self.types.get(type_id) {
            Some(Type::Literal(lit)) => {
                if let LiteralValue::String(s) = &lit.value {
                    let transformed = match kind {
                        StringMappingKind::Uppercase => s.to_uppercase(),
                        StringMappingKind::Lowercase => s.to_lowercase(),
                        StringMappingKind::Capitalize => {
                            let mut chars = s.chars();
                            match chars.next() {
                                None => String::new(),
                                Some(first) => first.to_uppercase().chain(chars).collect(),
                            }
                        }
                        StringMappingKind::Uncapitalize => {
                            let mut chars = s.chars();
                            match chars.next() {
                                None => String::new(),
                                Some(first) => first.to_lowercase().chain(chars).collect(),
                            }
                        }
                    };
                    self.types.create_string_literal(transformed)
                } else {
                    // Not a string literal, return string type
                    self.types.string_type
                }
            }
            Some(Type::Union(u)) => {
                // Apply to each member of the union
                let types = u.types.clone();
                let mapped_types: Vec<TypeId> = types.iter()
                    .map(|&t| self.apply_string_mapping(t, kind))
                    .collect();
                self.types.create_union_type(mapped_types)
            }
            Some(Type::Intrinsic(i)) if i.flags & type_flags::STRING != 0 => {
                // string -> string (can't transform unknown string)
                self.types.string_type
            }
            _ => {
                // For other types, just return string
                self.types.string_type
            }
        }
    }

}
