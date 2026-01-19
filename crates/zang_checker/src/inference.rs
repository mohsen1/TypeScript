//! Type Inference
//!
//! Infers types from expressions and declarations.

use std::collections::HashMap;
use zang_core::InternedString;
use zang_parser::{
    Expression, Statement, BindingName, TypeNode, BinaryOperator,
    Identifier, SyntaxKind, KeywordTypeNode,
};
use crate::types::{ResolvedType, TypeId, ObjectType, PropertySignature, FunctionType, ParameterType, LiteralType};

/// Type inference context
pub struct InferenceContext {
    /// Inferred type variables
    type_variables: HashMap<TypeId, ResolvedType>,
    /// Next type variable ID
    next_type_var: TypeId,
    /// Type parameter bindings
    type_parameter_bindings: HashMap<InternedString, ResolvedType>,
}

impl InferenceContext {
    /// Creates a new inference context
    pub fn new() -> Self {
        Self {
            type_variables: HashMap::new(),
            next_type_var: 1,
            type_parameter_bindings: HashMap::new(),
        }
    }

    /// Creates a fresh type variable
    pub fn fresh_type_var(&mut self) -> TypeId {
        let id = self.next_type_var;
        self.next_type_var += 1;
        id
    }

    /// Gets the inferred type for a type variable
    pub fn get_type_var(&self, id: TypeId) -> Option<&ResolvedType> {
        self.type_variables.get(&id)
    }

    /// Sets the inferred type for a type variable
    pub fn set_type_var(&mut self, id: TypeId, ty: ResolvedType) {
        self.type_variables.insert(id, ty);
    }

    /// Binds a type parameter to a type
    pub fn bind_type_parameter(&mut self, name: InternedString, ty: ResolvedType) {
        self.type_parameter_bindings.insert(name, ty);
    }

    /// Gets the binding for a type parameter
    pub fn get_type_parameter(&self, name: &InternedString) -> Option<&ResolvedType> {
        self.type_parameter_bindings.get(name)
    }

    /// Enters a new scope for type parameter bindings
    pub fn push_scope(&mut self) -> InferenceScopeGuard {
        InferenceScopeGuard {
            saved_bindings: self.type_parameter_bindings.clone(),
        }
    }

    /// Restores the previous scope
    pub fn pop_scope(&mut self, guard: InferenceScopeGuard) {
        self.type_parameter_bindings = guard.saved_bindings;
    }
}

impl Default for InferenceContext {
    fn default() -> Self {
        Self::new()
    }
}

/// Scope guard for type parameter bindings
pub struct InferenceScopeGuard {
    saved_bindings: HashMap<InternedString, ResolvedType>,
}

/// Type inferrer for TypeScript expressions
pub struct TypeInferrer<'a> {
    /// The inference context
    context: &'a mut InferenceContext,
    /// Symbol table for looking up declared types
    symbol_types: &'a HashMap<InternedString, ResolvedType>,
}

impl<'a> TypeInferrer<'a> {
    /// Creates a new type inferrer
    pub fn new(
        context: &'a mut InferenceContext,
        symbol_types: &'a HashMap<InternedString, ResolvedType>,
    ) -> Self {
        Self {
            context,
            symbol_types,
        }
    }

    /// Infers the type of an expression
    pub fn infer_expression(&mut self, expr: &Expression) -> ResolvedType {
        match expr {
            Expression::Identifier(ident) => self.infer_identifier(ident),
            Expression::StringLiteral(_lit) => {
                ResolvedType::Literal(LiteralType::String(
                    // For now, we'll return a simple string type
                    // In a full implementation, we'd get the actual string value
                    String::new()
                ))
            }
            Expression::NumericLiteral(lit) => {
                ResolvedType::Literal(LiteralType::Number(lit.value))
            }
            Expression::BooleanLiteral(lit) => {
                ResolvedType::Literal(LiteralType::Boolean(lit.value))
            }
            Expression::NullLiteral(_) => ResolvedType::Null,
            Expression::Array(arr) => self.infer_array_literal(arr),
            Expression::Object(obj) => self.infer_object_literal(obj),
            Expression::Call(call) => self.infer_call_expression(call),
            Expression::Binary(bin) => self.infer_binary_expression(bin),
            Expression::Unary(unary) => self.infer_unary_expression(unary),
            Expression::Conditional(cond) => self.infer_conditional_expression(cond),
            Expression::Arrow(arrow) => self.infer_arrow_function(arrow),
            Expression::Function(func) => self.infer_function_expression(func),
            Expression::PropertyAccess(prop) => self.infer_property_access(prop),
            Expression::ElementAccess(elem) => self.infer_element_access(elem),
            Expression::New(new_expr) => self.infer_new_expression(new_expr),
            Expression::Parenthesized(inner) => self.infer_expression(inner),
            Expression::BigIntLiteral(_) => ResolvedType::BigInt,
            Expression::Class(_) => ResolvedType::Any, // TODO: Implement class expression inference
        }
    }

    /// Infers the type of an identifier
    fn infer_identifier(&self, ident: &Identifier) -> ResolvedType {
        self.symbol_types
            .get(&ident.name)
            .cloned()
            .unwrap_or(ResolvedType::Any)
    }

    /// Infers the type of an array literal
    fn infer_array_literal(&mut self, arr: &zang_parser::ArrayLiteralExpression) -> ResolvedType {
        if arr.elements.is_empty() {
            // Empty array: infer as any[]
            return ResolvedType::Array(Box::new(ResolvedType::Any));
        }

        // Infer types of all elements
        let element_types: Vec<ResolvedType> = arr
            .elements
            .iter()
            .filter_map(|e| e.as_ref().map(|expr| self.infer_expression(expr)))
            .collect();

        if element_types.is_empty() {
            return ResolvedType::Array(Box::new(ResolvedType::Any));
        }

        // Find the best common type
        let element_type = self.find_best_common_type(&element_types);
        ResolvedType::Array(Box::new(element_type))
    }

    /// Infers the type of an object literal
    fn infer_object_literal(&mut self, obj: &zang_parser::ObjectLiteralExpression) -> ResolvedType {
        let mut properties = Vec::new();

        for element in &obj.properties {
            match element {
                zang_parser::ObjectLiteralElement::Property(prop) => {
                    if let Some(name) = self.get_property_name(&prop.name) {
                        let ty = self.infer_expression(&prop.initializer);
                        properties.push(PropertySignature {
                            name,
                            ty: Box::new(ty),
                            optional: false,
                            readonly: false,
                        });
                    }
                }
                zang_parser::ObjectLiteralElement::Shorthand(shorthand) => {
                    let ty = self.infer_identifier(&shorthand.name);
                    properties.push(PropertySignature {
                        name: shorthand.name.name,
                        ty: Box::new(ty),
                        optional: false,
                        readonly: false,
                    });
                }
                _ => {} // TODO: Handle other object literal elements
            }
        }

        ResolvedType::Object(ObjectType {
            properties,
            call_signatures: Vec::new(),
            construct_signatures: Vec::new(),
            index_signatures: Vec::new(),
        })
    }

    /// Gets the property name as an InternedString
    fn get_property_name(&self, name: &zang_parser::PropertyName) -> Option<InternedString> {
        match name {
            zang_parser::PropertyName::Identifier(ident) => Some(ident.name),
            zang_parser::PropertyName::StringLiteral(lit) => Some(lit.value),
            _ => None, // Computed property names not supported yet
        }
    }

    /// Infers the type of a call expression
    fn infer_call_expression(&mut self, call: &zang_parser::CallExpression) -> ResolvedType {
        let callee_type = self.infer_expression(&call.expression);

        match callee_type {
            ResolvedType::Function(func) => {
                // Return the return type of the function
                *func.return_type
            }
            ResolvedType::Object(obj) => {
                // Check for call signatures
                if let Some(sig) = obj.call_signatures.first() {
                    *sig.return_type.clone()
                } else {
                    ResolvedType::Any
                }
            }
            _ => ResolvedType::Any,
        }
    }

    /// Infers the type of a binary expression
    fn infer_binary_expression(&mut self, bin: &zang_parser::BinaryExpression) -> ResolvedType {
        let left_type = self.infer_expression(&bin.left);
        let right_type = self.infer_expression(&bin.right);

        match bin.operator {
            // Arithmetic operators return number
            BinaryOperator::Add => {
                // Special case: string concatenation
                if matches!(left_type, ResolvedType::String | ResolvedType::Literal(LiteralType::String(_)))
                    || matches!(right_type, ResolvedType::String | ResolvedType::Literal(LiteralType::String(_)))
                {
                    ResolvedType::String
                } else {
                    ResolvedType::Number
                }
            }
            BinaryOperator::Subtract
            | BinaryOperator::Multiply
            | BinaryOperator::Divide
            | BinaryOperator::Modulo
            | BinaryOperator::Power
            | BinaryOperator::LeftShift
            | BinaryOperator::RightShift
            | BinaryOperator::UnsignedRightShift
            | BinaryOperator::BitwiseAnd
            | BinaryOperator::BitwiseOr
            | BinaryOperator::BitwiseXor => ResolvedType::Number,

            // Comparison operators return boolean
            BinaryOperator::LessThan
            | BinaryOperator::GreaterThan
            | BinaryOperator::LessThanEquals
            | BinaryOperator::GreaterThanEquals
            | BinaryOperator::Equals
            | BinaryOperator::NotEquals
            | BinaryOperator::StrictEquals
            | BinaryOperator::StrictNotEquals => ResolvedType::Boolean,

            // Logical operators
            BinaryOperator::LogicalAnd => {
                // A && B returns A if A is falsy, otherwise B
                // Type is A | B narrowed by truthiness
                self.find_best_common_type(&[left_type, right_type])
            }
            BinaryOperator::LogicalOr => {
                // A || B returns A if A is truthy, otherwise B
                self.find_best_common_type(&[left_type, right_type])
            }
            BinaryOperator::NullishCoalescing => {
                // A ?? B returns A if A is not null/undefined, otherwise B
                // Remove null/undefined from left type and union with right
                let left_non_null = self.remove_null_undefined(&left_type);
                self.find_best_common_type(&[left_non_null, right_type])
            }

            // Assignment operators return the assigned type
            BinaryOperator::Assign => right_type,
            BinaryOperator::AddAssign
            | BinaryOperator::SubtractAssign
            | BinaryOperator::MultiplyAssign
            | BinaryOperator::DivideAssign
            | BinaryOperator::ModuloAssign
            | BinaryOperator::PowerAssign
            | BinaryOperator::LeftShiftAssign
            | BinaryOperator::RightShiftAssign
            | BinaryOperator::UnsignedRightShiftAssign
            | BinaryOperator::BitwiseAndAssign
            | BinaryOperator::BitwiseOrAssign
            | BinaryOperator::BitwiseXorAssign => ResolvedType::Number,

            BinaryOperator::LogicalAndAssign
            | BinaryOperator::LogicalOrAssign
            | BinaryOperator::NullishCoalescingAssign => right_type,

            // Comma operator returns right operand
            BinaryOperator::Comma => right_type,

            // in and instanceof return boolean
            BinaryOperator::In | BinaryOperator::InstanceOf => ResolvedType::Boolean,
        }
    }

    /// Infers the type of a unary expression
    fn infer_unary_expression(&mut self, unary: &zang_parser::UnaryExpression) -> ResolvedType {
        use zang_parser::UnaryOperator;

        match unary.operator {
            UnaryOperator::Plus | UnaryOperator::Minus => ResolvedType::Number,
            UnaryOperator::BitwiseNot => ResolvedType::Number,
            UnaryOperator::LogicalNot => ResolvedType::Boolean,
            UnaryOperator::TypeOf => ResolvedType::String,
            UnaryOperator::Void => ResolvedType::Undefined,
            UnaryOperator::Delete => ResolvedType::Boolean,
            UnaryOperator::Increment | UnaryOperator::Decrement => ResolvedType::Number,
            UnaryOperator::Await => {
                // Await unwraps a Promise
                let operand_type = self.infer_expression(&unary.operand);
                self.unwrap_promise(&operand_type)
            }
        }
    }

    /// Infers the type of a conditional expression
    fn infer_conditional_expression(&mut self, cond: &zang_parser::ConditionalExpression) -> ResolvedType {
        let true_type = self.infer_expression(&cond.when_true);
        let false_type = self.infer_expression(&cond.when_false);
        self.find_best_common_type(&[true_type, false_type])
    }

    /// Infers the type of an arrow function
    fn infer_arrow_function(&mut self, arrow: &zang_parser::ArrowFunction) -> ResolvedType {
        let mut parameters = Vec::new();

        for param in &arrow.parameters {
            let param_type = if let Some(ref type_ann) = param.type_annotation {
                self.type_from_type_node(type_ann)
            } else {
                ResolvedType::Any
            };

            let name = match &param.name {
                BindingName::Identifier(ident) => ident.name,
                _ => continue, // Skip complex binding patterns for now
            };

            parameters.push(ParameterType {
                name,
                ty: Box::new(param_type),
                optional: param.is_optional,
                rest: param.is_rest,
            });
        }

        let return_type = if let Some(ref ret_type) = arrow.return_type {
            self.type_from_type_node(ret_type)
        } else {
            // Infer return type from body
            match &arrow.body {
                zang_parser::ArrowFunctionBody::Expression(expr) => self.infer_expression(expr),
                zang_parser::ArrowFunctionBody::Block(block) => {
                    self.infer_return_type_from_block(block)
                }
            }
        };

        ResolvedType::Function(FunctionType {
            type_parameters: Vec::new(), // TODO: Handle type parameters
            parameters,
            return_type: Box::new(return_type),
        })
    }

    /// Infers the type of a function expression
    fn infer_function_expression(&mut self, func: &zang_parser::FunctionExpression) -> ResolvedType {
        let mut parameters = Vec::new();

        for param in &func.parameters {
            let param_type = if let Some(ref type_ann) = param.type_annotation {
                self.type_from_type_node(type_ann)
            } else {
                ResolvedType::Any
            };

            let name = match &param.name {
                BindingName::Identifier(ident) => ident.name,
                _ => continue,
            };

            parameters.push(ParameterType {
                name,
                ty: Box::new(param_type),
                optional: param.is_optional,
                rest: param.is_rest,
            });
        }

        let return_type = if let Some(ref ret_type) = func.return_type {
            self.type_from_type_node(ret_type)
        } else {
            self.infer_return_type_from_block(&func.body)
        };

        ResolvedType::Function(FunctionType {
            type_parameters: Vec::new(),
            parameters,
            return_type: Box::new(return_type),
        })
    }

    /// Infers the type of a property access expression
    fn infer_property_access(&mut self, prop: &zang_parser::PropertyAccessExpression) -> ResolvedType {
        let obj_type = self.infer_expression(&prop.expression);
        self.get_property_type(&obj_type, &prop.name.name)
    }

    /// Infers the type of an element access expression
    fn infer_element_access(&mut self, elem: &zang_parser::ElementAccessExpression) -> ResolvedType {
        let obj_type = self.infer_expression(&elem.expression);
        let index_type = self.infer_expression(&elem.argument);

        match obj_type {
            ResolvedType::Array(element_type) => *element_type,
            ResolvedType::Tuple(types) => {
                // If index is a numeric literal, we can get the exact type
                if let ResolvedType::Literal(LiteralType::Number(n)) = index_type {
                    let idx = n as usize;
                    if idx < types.len() {
                        return types[idx].clone();
                    }
                }
                // Otherwise return union of all tuple element types
                self.find_best_common_type(&types)
            }
            ResolvedType::Object(obj) => {
                // Check index signatures
                for sig in &obj.index_signatures {
                    // TODO: Check if index_type is assignable to sig.key_type
                    return *sig.value_type.clone();
                }
                ResolvedType::Any
            }
            _ => ResolvedType::Any,
        }
    }

    /// Infers the type of a new expression
    fn infer_new_expression(&mut self, new_expr: &zang_parser::NewExpression) -> ResolvedType {
        let constructor_type = self.infer_expression(&new_expr.expression);

        match constructor_type {
            ResolvedType::Object(obj) => {
                // Check for construct signatures
                if let Some(sig) = obj.construct_signatures.first() {
                    *sig.return_type.clone()
                } else {
                    ResolvedType::Any
                }
            }
            _ => ResolvedType::Any,
        }
    }

    /// Converts a TypeNode to a ResolvedType
    pub fn type_from_type_node(&self, node: &TypeNode) -> ResolvedType {
        match node {
            TypeNode::Keyword(kw) => self.type_from_keyword(kw),
            TypeNode::Reference(_ref_node) => {
                // TODO: Look up the referenced type
                ResolvedType::Any
            }
            TypeNode::Array(arr) => {
                let element_type = self.type_from_type_node(&arr.element_type);
                ResolvedType::Array(Box::new(element_type))
            }
            TypeNode::Tuple(tuple) => {
                let types: Vec<_> = tuple.elements.iter().map(|e| self.type_from_type_node(e)).collect();
                ResolvedType::Tuple(types)
            }
            TypeNode::Union(union) => {
                let types: Vec<_> = union.types.iter().map(|t| self.type_from_type_node(t)).collect();
                ResolvedType::Union(types)
            }
            TypeNode::Intersection(intersection) => {
                let types: Vec<_> = intersection.types.iter().map(|t| self.type_from_type_node(t)).collect();
                ResolvedType::Intersection(types)
            }
            TypeNode::Function(func) => {
                let parameters: Vec<_> = func
                    .parameters
                    .iter()
                    .map(|p| {
                        let ty = if let Some(ref ann) = p.type_annotation {
                            self.type_from_type_node(ann)
                        } else {
                            ResolvedType::Any
                        };
                        let name = match &p.name {
                            BindingName::Identifier(id) => id.name,
                            _ => return ParameterType {
                                name: zang_core::InternedString::from_raw(0),
                                ty: Box::new(ty),
                                optional: p.is_optional,
                                rest: p.is_rest,
                            },
                        };
                        ParameterType {
                            name,
                            ty: Box::new(ty),
                            optional: p.is_optional,
                            rest: p.is_rest,
                        }
                    })
                    .collect();

                let return_type = self.type_from_type_node(&func.return_type);

                ResolvedType::Function(FunctionType {
                    type_parameters: Vec::new(),
                    parameters,
                    return_type: Box::new(return_type),
                })
            }
            TypeNode::Literal(lit) => {
                match &lit.literal {
                    zang_parser::LiteralExpression::String(_s) => {
                        ResolvedType::Literal(LiteralType::String(String::new())) // TODO: Get actual value
                    }
                    zang_parser::LiteralExpression::Numeric(n) => {
                        ResolvedType::Literal(LiteralType::Number(n.value))
                    }
                    zang_parser::LiteralExpression::Boolean(b) => {
                        ResolvedType::Literal(LiteralType::Boolean(b.value))
                    }
                    _ => ResolvedType::Any,
                }
            }
            TypeNode::Parenthesized(inner) => self.type_from_type_node(inner),
            _ => ResolvedType::Any, // TODO: Handle more type nodes
        }
    }

    /// Converts a keyword type to a ResolvedType
    fn type_from_keyword(&self, kw: &KeywordTypeNode) -> ResolvedType {
        match kw.keyword {
            SyntaxKind::AnyKeyword => ResolvedType::Any,
            SyntaxKind::UnknownKeyword => ResolvedType::Unknown,
            SyntaxKind::NeverKeyword => ResolvedType::Never,
            SyntaxKind::VoidKeyword => ResolvedType::Void,
            SyntaxKind::UndefinedKeyword => ResolvedType::Undefined,
            SyntaxKind::NullKeyword => ResolvedType::Null,
            SyntaxKind::StringKeyword => ResolvedType::String,
            SyntaxKind::NumberKeyword => ResolvedType::Number,
            SyntaxKind::BooleanKeyword => ResolvedType::Boolean,
            SyntaxKind::BigIntKeyword => ResolvedType::BigInt,
            SyntaxKind::SymbolKeyword => ResolvedType::Symbol,
            SyntaxKind::ObjectKeyword => ResolvedType::Object(ObjectType {
                properties: Vec::new(),
                call_signatures: Vec::new(),
                construct_signatures: Vec::new(),
                index_signatures: Vec::new(),
            }),
            _ => ResolvedType::Any,
        }
    }

    /// Gets the type of a property on an object
    fn get_property_type(&self, obj_type: &ResolvedType, name: &InternedString) -> ResolvedType {
        match obj_type {
            ResolvedType::Object(obj) => {
                for prop in &obj.properties {
                    if &prop.name == name {
                        return *prop.ty.clone();
                    }
                }
                ResolvedType::Any
            }
            ResolvedType::Array(_) => {
                // TODO: Return array methods
                ResolvedType::Any
            }
            _ => ResolvedType::Any,
        }
    }

    /// Finds the best common type from a list of types
    fn find_best_common_type(&self, types: &[ResolvedType]) -> ResolvedType {
        if types.is_empty() {
            return ResolvedType::Never;
        }
        if types.len() == 1 {
            return types[0].clone();
        }

        // Check if all types are the same
        let first = &types[0];
        if types.iter().all(|t| self.types_equal(t, first)) {
            return first.clone();
        }

        // Create a union type
        ResolvedType::Union(types.to_vec())
    }

    /// Checks if two types are equal
    fn types_equal(&self, a: &ResolvedType, b: &ResolvedType) -> bool {
        match (a, b) {
            (ResolvedType::Any, ResolvedType::Any) => true,
            (ResolvedType::Unknown, ResolvedType::Unknown) => true,
            (ResolvedType::Never, ResolvedType::Never) => true,
            (ResolvedType::Void, ResolvedType::Void) => true,
            (ResolvedType::Undefined, ResolvedType::Undefined) => true,
            (ResolvedType::Null, ResolvedType::Null) => true,
            (ResolvedType::String, ResolvedType::String) => true,
            (ResolvedType::Number, ResolvedType::Number) => true,
            (ResolvedType::Boolean, ResolvedType::Boolean) => true,
            (ResolvedType::BigInt, ResolvedType::BigInt) => true,
            (ResolvedType::Symbol, ResolvedType::Symbol) => true,
            (ResolvedType::Array(a), ResolvedType::Array(b)) => self.types_equal(a, b),
            _ => false, // TODO: Implement more equality checks
        }
    }

    /// Removes null and undefined from a type
    fn remove_null_undefined(&self, ty: &ResolvedType) -> ResolvedType {
        match ty {
            ResolvedType::Union(types) => {
                let filtered: Vec<_> = types
                    .iter()
                    .filter(|t| !matches!(t, ResolvedType::Null | ResolvedType::Undefined))
                    .cloned()
                    .collect();
                if filtered.len() == 1 {
                    filtered[0].clone()
                } else if filtered.is_empty() {
                    ResolvedType::Never
                } else {
                    ResolvedType::Union(filtered)
                }
            }
            ResolvedType::Null | ResolvedType::Undefined => ResolvedType::Never,
            other => other.clone(),
        }
    }

    /// Unwraps a Promise type
    fn unwrap_promise(&self, ty: &ResolvedType) -> ResolvedType {
        // TODO: Properly handle Promise types
        // For now, just return the type as-is
        ty.clone()
    }

    /// Infers the return type from a block statement
    fn infer_return_type_from_block(&mut self, block: &zang_parser::BlockStatement) -> ResolvedType {
        let mut return_types = Vec::new();

        for stmt in &block.statements {
            if let Statement::Return(ret) = stmt {
                if let Some(ref expr) = ret.expression {
                    return_types.push(self.infer_expression(expr));
                } else {
                    return_types.push(ResolvedType::Undefined);
                }
            }
        }

        if return_types.is_empty() {
            ResolvedType::Void
        } else {
            self.find_best_common_type(&return_types)
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_inference_context_creation() {
        let ctx = InferenceContext::new();
        assert_eq!(ctx.next_type_var, 1);
    }

    #[test]
    fn test_fresh_type_var() {
        let mut ctx = InferenceContext::new();
        let v1 = ctx.fresh_type_var();
        let v2 = ctx.fresh_type_var();
        assert_eq!(v1, 1);
        assert_eq!(v2, 2);
    }

    #[test]
    fn test_type_var_get_set() {
        let mut ctx = InferenceContext::new();
        let id = ctx.fresh_type_var();
        assert!(ctx.get_type_var(id).is_none());
        ctx.set_type_var(id, ResolvedType::String);
        assert!(matches!(ctx.get_type_var(id), Some(ResolvedType::String)));
    }
}
