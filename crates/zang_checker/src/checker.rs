//! Type Checker
//!
//! Core type checking logic with lock-free caching.

use std::collections::HashMap;
use zang_core::{Arena, InternedString, Span, StringInterner};
use zang_parser::{
    SourceFile, Statement, Expression, VariableDeclaration, BindingName,
    TypeNode, BinaryOperator, FunctionDeclaration, ClassDeclaration,
};
use crate::context::CheckContext;
use crate::diagnostics::{Diagnostic, DiagnosticSeverity};
use crate::types::{ResolvedType, FunctionType, ParameterType, LiteralType};
use crate::inference::{InferenceContext, TypeInferrer};
use crate::assignability::AssignabilityChecker;
use crate::narrowing::{NarrowingContext, TypeNarrower};

/// Type checker for TypeScript programs
pub struct TypeChecker<'a> {
    /// Arena for type allocation
    arena: &'a Arena,
    /// String interner
    interner: &'a StringInterner,
    /// Check context for cycle detection
    context: CheckContext,
    /// Symbol type map
    symbol_types: HashMap<InternedString, ResolvedType>,
    /// Type inference context
    inference_context: InferenceContext,
    /// Assignability checker
    assignability_checker: AssignabilityChecker,
    /// Narrowing context
    narrowing_context: NarrowingContext,
    /// Collected diagnostics
    diagnostics: Vec<Diagnostic>,
    /// Current function return type (for return statement checking)
    current_return_type: Option<ResolvedType>,
    /// Whether we're in strict mode
    strict_mode: bool,
}

impl<'a> TypeChecker<'a> {
    /// Creates a new type checker
    pub fn new(arena: &'a Arena, interner: &'a StringInterner) -> Self {
        Self {
            arena,
            interner,
            context: CheckContext::new(),
            symbol_types: HashMap::new(),
            inference_context: InferenceContext::new(),
            assignability_checker: AssignabilityChecker::new(),
            narrowing_context: NarrowingContext::new(),
            diagnostics: Vec::new(),
            current_return_type: None,
            strict_mode: true,
        }
    }

    /// Creates a type checker with custom settings
    pub fn with_strict_mode(arena: &'a Arena, interner: &'a StringInterner, strict: bool) -> Self {
        let mut checker = Self::new(arena, interner);
        checker.strict_mode = strict;
        checker
    }

    /// Type checks a source file
    pub fn check(&mut self, source_file: &SourceFile) -> CheckResult {
        // First pass: collect declarations
        self.collect_declarations(source_file);

        // Second pass: type check statements
        for statement in &source_file.statements {
            self.check_statement(statement);
        }

        CheckResult {
            diagnostics: std::mem::take(&mut self.diagnostics),
        }
    }

    /// Collects declarations from the source file
    fn collect_declarations(&mut self, source_file: &SourceFile) {
        for statement in &source_file.statements {
            self.collect_statement_declarations(statement);
        }
    }

    /// Collects declarations from a statement
    fn collect_statement_declarations(&mut self, statement: &Statement) {
        match statement {
            Statement::Variable(var_stmt) => {
                for decl in &var_stmt.declaration_list.declarations {
                    self.collect_variable_declaration(decl);
                }
            }
            Statement::Function(func) => {
                if let Some(ref name) = func.name {
                    let func_type = self.get_function_type(func);
                    self.symbol_types.insert(name.name, func_type);
                }
            }
            Statement::Class(class) => {
                if let Some(ref name) = class.name {
                    // For now, just register as any
                    self.symbol_types.insert(name.name, ResolvedType::Any);
                }
            }
            Statement::Block(block) => {
                for stmt in &block.statements {
                    self.collect_statement_declarations(stmt);
                }
            }
            _ => {}
        }
    }

    /// Collects a variable declaration
    fn collect_variable_declaration(&mut self, decl: &VariableDeclaration) {
        let declared_type = if let Some(ref type_ann) = decl.type_annotation {
            self.resolve_type_node(type_ann)
        } else if let Some(ref init) = decl.initializer {
            self.infer_expression_type(init)
        } else {
            if self.strict_mode {
                if let BindingName::Identifier(ident) = &decl.name {
                    self.error(
                        2304,
                        format!("Variable '{}' implicitly has type 'any'",
                            self.interner.lookup(ident.name).unwrap_or_default()),
                        decl.span,
                    );
                }
            }
            ResolvedType::Any
        };

        if let BindingName::Identifier(ident) = &decl.name {
            self.symbol_types.insert(ident.name, declared_type);
        }
    }

    /// Gets the function type from a function declaration
    fn get_function_type(&mut self, func: &FunctionDeclaration) -> ResolvedType {
        let mut parameters = Vec::new();

        for param in &func.parameters {
            let param_type = if let Some(ref type_ann) = param.type_annotation {
                self.resolve_type_node(type_ann)
            } else {
                ResolvedType::Any
            };

            if let BindingName::Identifier(ident) = &param.name {
                parameters.push(ParameterType {
                    name: ident.name,
                    ty: Box::new(param_type),
                    optional: param.is_optional,
                    rest: param.is_rest,
                });
            }
        }

        let return_type = if let Some(ref ret_type) = func.return_type {
            self.resolve_type_node(ret_type)
        } else {
            ResolvedType::Any
        };

        ResolvedType::Function(FunctionType {
            type_parameters: Vec::new(),
            parameters,
            return_type: Box::new(return_type),
        })
    }

    /// Resolves a type node to a resolved type
    fn resolve_type_node(&self, node: &TypeNode) -> ResolvedType {
        let mut ctx = InferenceContext::new();
        let inferrer = TypeInferrer::new(&mut ctx, &self.symbol_types);
        inferrer.type_from_type_node(node)
    }

    /// Infers the type of an expression
    fn infer_expression_type(&mut self, expr: &Expression) -> ResolvedType {
        let mut inferrer = TypeInferrer::new(&mut self.inference_context, &self.symbol_types);
        inferrer.infer_expression(expr)
    }

    /// Gets the narrowed type for a symbol
    fn get_symbol_type(&self, name: &InternedString) -> ResolvedType {
        if let Some(narrowed) = self.narrowing_context.get_narrowed_type(name) {
            return narrowed.clone();
        }

        self.symbol_types
            .get(name)
            .cloned()
            .unwrap_or(ResolvedType::Any)
    }

    /// Type checks a statement
    fn check_statement(&mut self, statement: &Statement) {
        match statement {
            Statement::Variable(var_stmt) => {
                self.check_variable_statement(var_stmt);
            }
            Statement::Expression(expr_stmt) => {
                self.check_expression(&expr_stmt.expression);
            }
            Statement::If(if_stmt) => {
                self.check_if_statement(if_stmt);
            }
            Statement::Return(ret_stmt) => {
                self.check_return_statement(ret_stmt);
            }
            Statement::Function(func) => {
                self.check_function_declaration(func);
            }
            Statement::Class(class) => {
                self.check_class_declaration(class);
            }
            Statement::Block(block) => {
                for stmt in &block.statements {
                    self.check_statement(stmt);
                }
            }
            Statement::Empty(_) => {}
        }
    }

    /// Type checks a variable statement
    fn check_variable_statement(&mut self, var_stmt: &zang_parser::VariableStatement) {
        for decl in &var_stmt.declaration_list.declarations {
            self.check_variable_declaration(decl);
        }
    }

    /// Type checks a variable declaration
    fn check_variable_declaration(&mut self, decl: &VariableDeclaration) {
        if let Some(ref init) = decl.initializer {
            let init_type = self.infer_expression_type(init);

            if let Some(ref type_ann) = decl.type_annotation {
                let declared_type = self.resolve_type_node(type_ann);

                let result = self.assignability_checker.is_assignable_to(&init_type, &declared_type);

                if !result.is_assignable() {
                    if let BindingName::Identifier(_ident) = &decl.name {
                        self.error(
                            2322,
                            format!(
                                "Type '{}' is not assignable to type '{}'",
                                self.type_to_string(&init_type),
                                self.type_to_string(&declared_type)
                            ),
                            decl.span,
                        );
                    }
                }
            }
        }
    }

    /// Type checks an if statement
    fn check_if_statement(&mut self, if_stmt: &zang_parser::IfStatement) {
        self.check_expression(&if_stmt.condition);

        // Process narrowing for then branch
        self.narrowing_context.save_state();
        {
            let mut narrower = TypeNarrower::new(&mut self.narrowing_context, &self.symbol_types);
            narrower.narrow_by_condition(&if_stmt.condition, true);
        }
        self.check_statement(&if_stmt.then_statement);
        self.narrowing_context.restore_state();

        if let Some(ref else_stmt) = if_stmt.else_statement {
            self.narrowing_context.save_state();
            {
                let mut narrower = TypeNarrower::new(&mut self.narrowing_context, &self.symbol_types);
                narrower.narrow_by_condition(&if_stmt.condition, false);
            }
            self.check_statement(else_stmt);
            self.narrowing_context.restore_state();
        }
    }

    /// Type checks a return statement
    fn check_return_statement(&mut self, ret_stmt: &zang_parser::ReturnStatement) {
        let return_type = if let Some(ref expr) = ret_stmt.expression {
            self.infer_expression_type(expr)
        } else {
            ResolvedType::Undefined
        };

        if let Some(ref expected) = self.current_return_type {
            let result = self.assignability_checker.is_assignable_to(&return_type, expected);
            if !result.is_assignable() {
                self.error(
                    2322,
                    format!(
                        "Type '{}' is not assignable to type '{}'",
                        self.type_to_string(&return_type),
                        self.type_to_string(expected)
                    ),
                    ret_stmt.span,
                );
            }
        }
    }

    /// Type checks a function declaration
    fn check_function_declaration(&mut self, func: &FunctionDeclaration) {
        let prev_return_type = self.current_return_type.take();

        if let Some(ref ret_type) = func.return_type {
            self.current_return_type = Some(self.resolve_type_node(ret_type));
        }

        for param in &func.parameters {
            if let BindingName::Identifier(ident) = &param.name {
                let param_type = if let Some(ref type_ann) = param.type_annotation {
                    self.resolve_type_node(type_ann)
                } else {
                    if self.strict_mode {
                        self.error(
                            7006,
                            format!(
                                "Parameter '{}' implicitly has an 'any' type",
                                self.interner.lookup(ident.name).unwrap_or_default()
                            ),
                            param.span,
                        );
                    }
                    ResolvedType::Any
                };
                self.symbol_types.insert(ident.name, param_type);
            }
        }

        if let Some(ref body) = func.body {
            for stmt in &body.statements {
                self.check_statement(stmt);
            }
        }

        self.current_return_type = prev_return_type;
    }

    /// Type checks a class declaration
    fn check_class_declaration(&mut self, _class: &ClassDeclaration) {
        // TODO: Implement class type checking
    }

    /// Type checks an expression and returns its type
    fn check_expression(&mut self, expr: &Expression) -> ResolvedType {
        match expr {
            Expression::Identifier(ident) => {
                let ty = self.get_symbol_type(&ident.name);
                if matches!(ty, ResolvedType::Any) && self.strict_mode {
                    if !self.symbol_types.contains_key(&ident.name) {
                        self.error(
                            2304,
                            format!(
                                "Cannot find name '{}'",
                                self.interner.lookup(ident.name).unwrap_or_default()
                            ),
                            ident.span,
                        );
                    }
                }
                ty
            }
            Expression::Binary(bin) => {
                self.check_binary_expression(bin)
            }
            Expression::Call(call) => {
                self.check_call_expression(call)
            }
            Expression::PropertyAccess(prop) => {
                self.check_property_access(prop)
            }
            _ => self.infer_expression_type(expr),
        }
    }

    /// Type checks a binary expression
    fn check_binary_expression(&mut self, bin: &zang_parser::BinaryExpression) -> ResolvedType {
        let left_type = self.check_expression(&bin.left);
        let right_type = self.check_expression(&bin.right);

        match bin.operator {
            BinaryOperator::Assign => {
                let result = self.assignability_checker.is_assignable_to(&right_type, &left_type);
                if !result.is_assignable() {
                    self.error(
                        2322,
                        format!(
                            "Type '{}' is not assignable to type '{}'",
                            self.type_to_string(&right_type),
                            self.type_to_string(&left_type)
                        ),
                        bin.span,
                    );
                }

                if let Expression::Identifier(ident) = &*bin.left {
                    self.narrowing_context.remove_narrowing(&ident.name);
                }

                right_type
            }
            BinaryOperator::Add => {
                let is_string_concat = matches!(
                    (&left_type, &right_type),
                    (ResolvedType::String, _) | (_, ResolvedType::String) |
                    (ResolvedType::Literal(LiteralType::String(_)), _) |
                    (_, ResolvedType::Literal(LiteralType::String(_)))
                );

                if is_string_concat {
                    ResolvedType::String
                } else {
                    self.check_numeric_operands(&left_type, &right_type, bin.span);
                    ResolvedType::Number
                }
            }
            BinaryOperator::Subtract
            | BinaryOperator::Multiply
            | BinaryOperator::Divide
            | BinaryOperator::Modulo => {
                self.check_numeric_operands(&left_type, &right_type, bin.span);
                ResolvedType::Number
            }
            BinaryOperator::LessThan
            | BinaryOperator::GreaterThan
            | BinaryOperator::LessThanEquals
            | BinaryOperator::GreaterThanEquals
            | BinaryOperator::Equals
            | BinaryOperator::NotEquals
            | BinaryOperator::StrictEquals
            | BinaryOperator::StrictNotEquals => {
                ResolvedType::Boolean
            }
            BinaryOperator::LogicalAnd | BinaryOperator::LogicalOr => {
                ResolvedType::Union(vec![left_type, right_type])
            }
            _ => self.infer_expression_type(&Expression::Binary(bin.clone())),
        }
    }

    /// Checks that operands are numeric
    fn check_numeric_operands(&mut self, left: &ResolvedType, right: &ResolvedType, span: Span) {
        let left_ok = self.is_numeric_type(left);
        let right_ok = self.is_numeric_type(right);

        if !left_ok {
            self.error(
                2362,
                "The left-hand side of an arithmetic operation must be of type 'any', 'number', 'bigint' or an enum type".to_string(),
                span,
            );
        }

        if !right_ok {
            self.error(
                2363,
                "The right-hand side of an arithmetic operation must be of type 'any', 'number', 'bigint' or an enum type".to_string(),
                span,
            );
        }
    }

    /// Checks if a type is numeric
    fn is_numeric_type(&self, ty: &ResolvedType) -> bool {
        matches!(
            ty,
            ResolvedType::Number
                | ResolvedType::BigInt
                | ResolvedType::Any
                | ResolvedType::Literal(LiteralType::Number(_))
                | ResolvedType::Literal(LiteralType::BigInt(_))
        )
    }

    /// Type checks a call expression
    fn check_call_expression(&mut self, call: &zang_parser::CallExpression) -> ResolvedType {
        let callee_type = self.check_expression(&call.expression);

        match callee_type {
            ResolvedType::Function(func) => {
                let required_params = func.parameters.iter().filter(|p| !p.optional).count();
                let provided_args = call.arguments.len();

                if provided_args < required_params {
                    self.error(
                        2554,
                        format!(
                            "Expected {} arguments, but got {}",
                            required_params,
                            provided_args
                        ),
                        call.span,
                    );
                }

                for (arg, param) in call.arguments.iter().zip(func.parameters.iter()) {
                    let arg_type = self.check_expression(arg);
                    let result = self.assignability_checker.is_assignable_to(&arg_type, &param.ty);

                    if !result.is_assignable() {
                        self.error(
                            2345,
                            format!(
                                "Argument of type '{}' is not assignable to parameter of type '{}'",
                                self.type_to_string(&arg_type),
                                self.type_to_string(&param.ty)
                            ),
                            call.span,
                        );
                    }
                }

                *func.return_type
            }
            ResolvedType::Any => ResolvedType::Any,
            _ => {
                self.error(
                    2349,
                    "This expression is not callable".to_string(),
                    call.span,
                );
                ResolvedType::Any
            }
        }
    }

    /// Type checks a property access expression
    fn check_property_access(&mut self, prop: &zang_parser::PropertyAccessExpression) -> ResolvedType {
        let obj_type = self.check_expression(&prop.expression);

        match obj_type {
            ResolvedType::Object(obj) => {
                for property in &obj.properties {
                    if property.name == prop.name.name {
                        return *property.ty.clone();
                    }
                }

                self.error(
                    2339,
                    format!(
                        "Property '{}' does not exist on type",
                        self.interner.lookup(prop.name.name).unwrap_or_default()
                    ),
                    prop.span,
                );
                ResolvedType::Any
            }
            ResolvedType::Any => ResolvedType::Any,
            _ => {
                self.error(
                    2339,
                    format!(
                        "Property '{}' does not exist on type '{}'",
                        self.interner.lookup(prop.name.name).unwrap_or_default(),
                        self.type_to_string(&obj_type)
                    ),
                    prop.span,
                );
                ResolvedType::Any
            }
        }
    }

    /// Converts a type to a string for error messages
    fn type_to_string(&self, ty: &ResolvedType) -> String {
        match ty {
            ResolvedType::Any => "any".to_string(),
            ResolvedType::Unknown => "unknown".to_string(),
            ResolvedType::Never => "never".to_string(),
            ResolvedType::Void => "void".to_string(),
            ResolvedType::Undefined => "undefined".to_string(),
            ResolvedType::Null => "null".to_string(),
            ResolvedType::String => "string".to_string(),
            ResolvedType::Number => "number".to_string(),
            ResolvedType::Boolean => "boolean".to_string(),
            ResolvedType::BigInt => "bigint".to_string(),
            ResolvedType::Symbol => "symbol".to_string(),
            ResolvedType::Array(elem) => format!("{}[]", self.type_to_string(elem)),
            ResolvedType::Tuple(types) => {
                let inner: Vec<_> = types.iter().map(|t| self.type_to_string(t)).collect();
                format!("[{}]", inner.join(", "))
            }
            ResolvedType::Union(types) => {
                let inner: Vec<_> = types.iter().map(|t| self.type_to_string(t)).collect();
                inner.join(" | ")
            }
            ResolvedType::Intersection(types) => {
                let inner: Vec<_> = types.iter().map(|t| self.type_to_string(t)).collect();
                inner.join(" & ")
            }
            ResolvedType::Function(func) => {
                let params: Vec<_> = func
                    .parameters
                    .iter()
                    .map(|p| {
                        let opt = if p.optional { "?" } else { "" };
                        format!(
                            "{}{}: {}",
                            self.interner.lookup(p.name).unwrap_or_default(),
                            opt,
                            self.type_to_string(&p.ty)
                        )
                    })
                    .collect();
                format!("({}) => {}", params.join(", "), self.type_to_string(&func.return_type))
            }
            ResolvedType::Object(_) => "object".to_string(),
            ResolvedType::Literal(lit) => match lit {
                LiteralType::String(s) => format!("\"{}\"", s),
                LiteralType::Number(n) => format!("{}", n),
                LiteralType::Boolean(b) => format!("{}", b),
                LiteralType::BigInt(s) => format!("{}n", s),
            },
            _ => "unknown".to_string(),
        }
    }

    /// Reports an error diagnostic
    fn error(&mut self, code: u32, message: String, span: Span) {
        self.diagnostics.push(Diagnostic::type_error(code, message, span));
    }

    /// Gets the arena
    pub fn arena(&self) -> &'a Arena {
        self.arena
    }

    /// Gets the string interner
    pub fn interner(&self) -> &'a StringInterner {
        self.interner
    }

    /// Checks if a type is assignable to another
    pub fn is_assignable_to(&mut self, source: &ResolvedType, target: &ResolvedType) -> bool {
        self.assignability_checker.is_assignable_to(source, target).is_assignable()
    }
}

/// Result of type checking
pub struct CheckResult {
    /// Diagnostics produced during type checking
    pub diagnostics: Vec<Diagnostic>,
}

impl CheckResult {
    /// Returns true if there are no errors
    pub fn is_ok(&self) -> bool {
        self.diagnostics.iter().all(|d| d.severity != DiagnosticSeverity::Error)
    }

    /// Returns the number of errors
    pub fn error_count(&self) -> usize {
        self.diagnostics.iter().filter(|d| d.severity == DiagnosticSeverity::Error).count()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_type_checker_creation() {
        let arena = Arena::new();
        let interner = StringInterner::new();
        let checker = TypeChecker::new(&arena, &interner);
        assert!(checker.strict_mode);
    }

    #[test]
    fn test_type_to_string() {
        let arena = Arena::new();
        let interner = StringInterner::new();
        let checker = TypeChecker::new(&arena, &interner);

        assert_eq!(checker.type_to_string(&ResolvedType::String), "string");
        assert_eq!(checker.type_to_string(&ResolvedType::Number), "number");
        assert_eq!(
            checker.type_to_string(&ResolvedType::Array(Box::new(ResolvedType::String))),
            "string[]"
        );
        assert_eq!(
            checker.type_to_string(&ResolvedType::Union(vec![
                ResolvedType::String,
                ResolvedType::Number
            ])),
            "string | number"
        );
    }

    #[test]
    fn test_is_numeric_type() {
        let arena = Arena::new();
        let interner = StringInterner::new();
        let checker = TypeChecker::new(&arena, &interner);

        assert!(checker.is_numeric_type(&ResolvedType::Number));
        assert!(checker.is_numeric_type(&ResolvedType::BigInt));
        assert!(checker.is_numeric_type(&ResolvedType::Any));
        assert!(!checker.is_numeric_type(&ResolvedType::String));
    }
}
