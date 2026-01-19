//! Async/Await Type Checking
//!
//! This module handles type checking for async functions and await expressions:
//! - Ensuring async function return types wrap in Promise
//! - Handling await expression type unwrapping
//! - Implementing Awaited<T> type resolution
//! - Checking for-await-of loops
//! - Handling async generators and AsyncIterator

use std::collections::HashMap;
use zang_core::{InternedString, Span, StringInterner};
use zang_parser::{
    ArrowFunction, Expression, FunctionDeclaration, FunctionExpression, Statement,
};

use crate::diagnostics::{Diagnostic, DiagnosticKind, DiagnosticSeverity};
use crate::promise::PromiseChecker;
use crate::types::{
    FunctionType, ParameterType, ResolvedType, TypeReference,
};

/// Async function checker
pub struct AsyncChecker<'a> {
    /// String interner
    interner: &'a StringInterner,
    /// Promise checker for Promise-related operations
    promise_checker: PromiseChecker<'a>,
    /// Symbol types from the current context
    symbol_types: &'a HashMap<InternedString, ResolvedType>,
    /// Diagnostics
    diagnostics: Vec<Diagnostic>,
    /// Whether we're inside an async context
    in_async_context: bool,
    /// Current async function's expected return type
    current_return_type: Option<ResolvedType>,
}

impl<'a> AsyncChecker<'a> {
    /// Creates a new async checker
    pub fn new(
        interner: &'a StringInterner,
        symbol_types: &'a HashMap<InternedString, ResolvedType>,
    ) -> Self {
        Self {
            interner,
            promise_checker: PromiseChecker::new(interner),
            symbol_types,
            diagnostics: Vec::new(),
            in_async_context: false,
            current_return_type: None,
        }
    }

    /// Takes the collected diagnostics
    pub fn take_diagnostics(&mut self) -> Vec<Diagnostic> {
        std::mem::take(&mut self.diagnostics)
    }

    /// Returns whether we're in an async context
    pub fn is_in_async_context(&self) -> bool {
        self.in_async_context
    }

    /// Checks an async function declaration
    pub fn check_async_function(
        &mut self,
        func: &FunctionDeclaration,
    ) -> ResolvedType {
        if !func.is_async {
            // Not an async function, return normal function type
            return self.get_function_type(func);
        }

        let old_async_context = self.in_async_context;
        let old_return_type = self.current_return_type.take();

        self.in_async_context = true;

        // Get the declared return type or infer it
        let inner_return_type = self.get_declared_return_type(func);

        // Set the expected return type for checking return statements
        self.current_return_type = Some(inner_return_type.clone());

        // Check the function body
        if let Some(ref body) = func.body {
            self.check_async_body(&body.statements);
        }

        // Restore context
        self.in_async_context = old_async_context;
        self.current_return_type = old_return_type;

        // Async functions always return Promise<T>
        let return_type = self.promise_checker.create_promise_type(inner_return_type);

        // Build the function type
        let params = self.get_parameter_types(func);

        ResolvedType::Function(FunctionType {
            type_parameters: Vec::new(),
            parameters: params,
            return_type: Box::new(return_type),
        })
    }

    /// Checks an async arrow function
    pub fn check_async_arrow(
        &mut self,
        arrow: &ArrowFunction,
    ) -> ResolvedType {
        if !arrow.is_async {
            return self.get_arrow_function_type(arrow);
        }

        let old_async_context = self.in_async_context;
        let old_return_type = self.current_return_type.take();

        self.in_async_context = true;

        // Get the declared return type or infer from body
        let inner_return_type = self.get_arrow_declared_return_type(arrow);
        self.current_return_type = Some(inner_return_type.clone());

        // Check the function body
        match &arrow.body {
            zang_parser::ArrowFunctionBody::Block(block) => {
                self.check_async_body(&block.statements);
            }
            zang_parser::ArrowFunctionBody::Expression(expr) => {
                let expr_type = self.infer_expression_type(expr);
                // Expression body: the expression type should be assignable to return type
                self.check_expression_return_type(&expr_type, arrow.span);
            }
        }

        self.in_async_context = old_async_context;
        self.current_return_type = old_return_type;

        let return_type = self.promise_checker.create_promise_type(inner_return_type);
        let params = self.get_arrow_parameter_types(arrow);

        ResolvedType::Function(FunctionType {
            type_parameters: Vec::new(),
            parameters: params,
            return_type: Box::new(return_type),
        })
    }

    /// Checks an async function expression
    pub fn check_async_function_expression(
        &mut self,
        func: &FunctionExpression,
    ) -> ResolvedType {
        if !func.is_async {
            return self.get_function_expression_type(func);
        }

        let old_async_context = self.in_async_context;
        let old_return_type = self.current_return_type.take();

        self.in_async_context = true;

        let inner_return_type = self.get_function_expr_declared_return_type(func);
        self.current_return_type = Some(inner_return_type.clone());

        self.check_async_body(&func.body.statements);

        self.in_async_context = old_async_context;
        self.current_return_type = old_return_type;

        let return_type = self.promise_checker.create_promise_type(inner_return_type);
        let params = self.get_function_expr_parameter_types(func);

        ResolvedType::Function(FunctionType {
            type_parameters: Vec::new(),
            parameters: params,
            return_type: Box::new(return_type),
        })
    }

    /// Checks an await expression and returns the unwrapped type
    pub fn check_await_expression(
        &mut self,
        operand_type: &ResolvedType,
        span: Span,
    ) -> ResolvedType {
        if !self.in_async_context {
            self.diagnostics.push(Diagnostic {
                kind: DiagnosticKind::Semantic,
                message: "'await' expression is only allowed within an async function".to_string(),
                span,
                severity: DiagnosticSeverity::Error,
                related: Vec::new(),
                code: 1308,
            });
            return ResolvedType::Any;
        }

        // Unwrap the promise/thenable type
        self.promise_checker.get_awaited_type(operand_type)
    }

    /// Checks a for-await-of loop
    pub fn check_for_await_of(
        &mut self,
        iterable_type: &ResolvedType,
        span: Span,
    ) -> ResolvedType {
        if !self.in_async_context {
            self.diagnostics.push(Diagnostic {
                kind: DiagnosticKind::Semantic,
                message: "'for await' is only valid in an async function or async generator".to_string(),
                span,
                severity: DiagnosticSeverity::Error,
                related: Vec::new(),
                code: 1103,
            });
            return ResolvedType::Any;
        }

        // Get the async iterable's element type
        self.get_async_iterable_element_type(iterable_type, span)
    }

    /// Gets the element type from an async iterable
    fn get_async_iterable_element_type(
        &mut self,
        iterable_type: &ResolvedType,
        span: Span,
    ) -> ResolvedType {
        match iterable_type {
            // If it's a type reference to AsyncIterable<T> or AsyncIterableIterator<T>
            ResolvedType::Reference(type_ref) => {
                let name = self.interner.lookup(type_ref.name).unwrap_or_default();
                if name == "AsyncIterable" || name == "AsyncIterableIterator" || name == "AsyncGenerator" {
                    if let Some(first_arg) = type_ref.type_arguments.first() {
                        return first_arg.clone();
                    }
                }
                // Try to get [Symbol.asyncIterator] method
                self.diagnostics.push(Diagnostic {
                    kind: DiagnosticKind::Type,
                    message: format!("Type '{}' must have a '[Symbol.asyncIterator]()' method", name),
                    span,
                    severity: DiagnosticSeverity::Error,
                    related: Vec::new(),
                    code: 2504,
                });
                ResolvedType::Any
            }
            // For object types, check for [Symbol.asyncIterator]
            ResolvedType::Object(obj) => {
                // Look for [Symbol.asyncIterator] method
                // For simplicity, we'll check if there's a method that returns AsyncIterator
                for sig in &obj.call_signatures {
                    if let ResolvedType::Reference(ref_type) = sig.return_type.as_ref() {
                        let name = self.interner.lookup(ref_type.name).unwrap_or_default();
                        if name == "AsyncIterator" || name == "AsyncIterableIterator" {
                            if let Some(elem_type) = ref_type.type_arguments.first() {
                                return elem_type.clone();
                            }
                        }
                    }
                }
                self.diagnostics.push(Diagnostic {
                    kind: DiagnosticKind::Type,
                    message: "Type must have a '[Symbol.asyncIterator]()' method".to_string(),
                    span,
                    severity: DiagnosticSeverity::Error,
                    related: Vec::new(),
                    code: 2504,
                });
                ResolvedType::Any
            }
            // Any type
            ResolvedType::Any => ResolvedType::Any,
            // Other types are not async iterable
            _ => {
                self.diagnostics.push(Diagnostic {
                    kind: DiagnosticKind::Type,
                    message: "Type is not an async iterable".to_string(),
                    span,
                    severity: DiagnosticSeverity::Error,
                    related: Vec::new(),
                    code: 2504,
                });
                ResolvedType::Any
            }
        }
    }

    /// Checks an async generator function
    pub fn check_async_generator(
        &mut self,
        func: &FunctionDeclaration,
    ) -> ResolvedType {
        if !func.is_async || !func.is_generator {
            return self.get_function_type(func);
        }

        let old_async_context = self.in_async_context;
        self.in_async_context = true;

        // Get the yield type and return type
        let yield_type = self.get_declared_return_type(func);
        let return_type = ResolvedType::Void; // Async generators typically return void
        let next_type = ResolvedType::Unknown; // The type passed to next()

        if let Some(ref body) = func.body {
            self.check_async_body(&body.statements);
        }

        self.in_async_context = old_async_context;

        // AsyncGenerator<T, TReturn, TNext>
        self.create_async_generator_type(yield_type, return_type, next_type)
    }

    /// Creates an AsyncGenerator<T, TReturn, TNext> type
    fn create_async_generator_type(
        &self,
        yield_type: ResolvedType,
        return_type: ResolvedType,
        next_type: ResolvedType,
    ) -> ResolvedType {
        let name = self.interner.intern("AsyncGenerator");
        ResolvedType::Reference(TypeReference {
            name,
            type_arguments: vec![yield_type, return_type, next_type],
            target: None,
        })
    }

    /// Checks the body of an async function
    fn check_async_body(&mut self, statements: &[Statement]) {
        for statement in statements {
            self.check_async_statement(statement);
        }
    }

    /// Checks a statement in async context
    fn check_async_statement(&mut self, statement: &Statement) {
        match statement {
            Statement::Return(ret_stmt) => {
                if let Some(ref expr) = ret_stmt.expression {
                    let expr_type = self.infer_expression_type(expr);
                    self.check_expression_return_type(&expr_type, ret_stmt.span);
                }
            }
            Statement::Block(block) => {
                self.check_async_body(&block.statements);
            }
            Statement::If(if_stmt) => {
                self.check_async_statement(&if_stmt.then_statement);
                if let Some(ref else_stmt) = if_stmt.else_statement {
                    self.check_async_statement(else_stmt);
                }
            }
            _ => {}
        }
    }

    /// Checks if an expression type is compatible with the expected return type
    fn check_expression_return_type(&mut self, expr_type: &ResolvedType, span: Span) {
        if let Some(ref expected) = self.current_return_type {
            // In async functions, we should be able to return T or Promise<T>
            // The return type will be awaited
            let awaited_expr = self.promise_checker.get_awaited_type(expr_type);
            let awaited_expected = self.promise_checker.get_awaited_type(expected);

            // Basic compatibility check (simplified)
            if !self.is_assignable(&awaited_expr, &awaited_expected) {
                self.diagnostics.push(Diagnostic {
                    kind: DiagnosticKind::Type,
                    message: format!(
                        "Type is not assignable to the declared return type"
                    ),
                    span,
                    severity: DiagnosticSeverity::Error,
                    related: Vec::new(),
                    code: 2322,
                });
            }
        }
    }

    /// Simple assignability check
    fn is_assignable(&self, source: &ResolvedType, target: &ResolvedType) -> bool {
        match (source, target) {
            (ResolvedType::Any, _) | (_, ResolvedType::Any) => true,
            (ResolvedType::Never, _) => true,
            (_, ResolvedType::Unknown) => true,
            (a, b) if std::mem::discriminant(a) == std::mem::discriminant(b) => true,
            _ => false,
        }
    }

    // Helper methods to get types from AST nodes

    fn get_function_type(&self, func: &FunctionDeclaration) -> ResolvedType {
        let params = self.get_parameter_types(func);
        let return_type = self.get_declared_return_type(func);

        ResolvedType::Function(FunctionType {
            type_parameters: Vec::new(),
            parameters: params,
            return_type: Box::new(return_type),
        })
    }

    fn get_arrow_function_type(&self, arrow: &ArrowFunction) -> ResolvedType {
        let params = self.get_arrow_parameter_types(arrow);
        let return_type = self.get_arrow_declared_return_type(arrow);

        ResolvedType::Function(FunctionType {
            type_parameters: Vec::new(),
            parameters: params,
            return_type: Box::new(return_type),
        })
    }

    fn get_function_expression_type(&self, func: &FunctionExpression) -> ResolvedType {
        let params = self.get_function_expr_parameter_types(func);
        let return_type = self.get_function_expr_declared_return_type(func);

        ResolvedType::Function(FunctionType {
            type_parameters: Vec::new(),
            parameters: params,
            return_type: Box::new(return_type),
        })
    }

    fn get_parameter_types(&self, func: &FunctionDeclaration) -> Vec<ParameterType> {
        func.parameters.iter().map(|p| {
            let name = match &p.name {
                zang_parser::BindingName::Identifier(ident) => ident.name,
                _ => self.interner.intern("_"),
            };
            ParameterType {
                name,
                ty: Box::new(ResolvedType::Any), // Would need type annotation resolution
                optional: p.is_optional,
                rest: p.is_rest,
            }
        }).collect()
    }

    fn get_arrow_parameter_types(&self, arrow: &ArrowFunction) -> Vec<ParameterType> {
        arrow.parameters.iter().map(|p| {
            let name = match &p.name {
                zang_parser::BindingName::Identifier(ident) => ident.name,
                _ => self.interner.intern("_"),
            };
            ParameterType {
                name,
                ty: Box::new(ResolvedType::Any),
                optional: p.is_optional,
                rest: p.is_rest,
            }
        }).collect()
    }

    fn get_function_expr_parameter_types(&self, func: &FunctionExpression) -> Vec<ParameterType> {
        func.parameters.iter().map(|p| {
            let name = match &p.name {
                zang_parser::BindingName::Identifier(ident) => ident.name,
                _ => self.interner.intern("_"),
            };
            ParameterType {
                name,
                ty: Box::new(ResolvedType::Any),
                optional: p.is_optional,
                rest: p.is_rest,
            }
        }).collect()
    }

    fn get_declared_return_type(&self, _func: &FunctionDeclaration) -> ResolvedType {
        // Would resolve the type annotation if present
        ResolvedType::Any
    }

    fn get_arrow_declared_return_type(&self, _arrow: &ArrowFunction) -> ResolvedType {
        ResolvedType::Any
    }

    fn get_function_expr_declared_return_type(&self, _func: &FunctionExpression) -> ResolvedType {
        ResolvedType::Any
    }

    fn infer_expression_type(&self, _expr: &Expression) -> ResolvedType {
        // Would use the inference module
        ResolvedType::Any
    }
}

/// Result of checking an await expression
#[derive(Debug, Clone)]
pub struct AwaitResult {
    /// The unwrapped type after awaiting
    pub unwrapped_type: ResolvedType,
    /// Whether the operand was a thenable
    pub was_thenable: bool,
}

/// Creates an AsyncIterator<T> type
pub fn create_async_iterator_type(interner: &StringInterner, element_type: ResolvedType) -> ResolvedType {
    let name = interner.intern("AsyncIterator");
    ResolvedType::Reference(TypeReference {
        name,
        type_arguments: vec![element_type],
        target: None,
    })
}

/// Creates an AsyncIterable<T> type
pub fn create_async_iterable_type(interner: &StringInterner, element_type: ResolvedType) -> ResolvedType {
    let name = interner.intern("AsyncIterable");
    ResolvedType::Reference(TypeReference {
        name,
        type_arguments: vec![element_type],
        target: None,
    })
}

/// Creates an AsyncIterableIterator<T> type
pub fn create_async_iterable_iterator_type(interner: &StringInterner, element_type: ResolvedType) -> ResolvedType {
    let name = interner.intern("AsyncIterableIterator");
    ResolvedType::Reference(TypeReference {
        name,
        type_arguments: vec![element_type],
        target: None,
    })
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_async_checker_creation() {
        let interner = StringInterner::new();
        let symbol_types = HashMap::new();
        let checker = AsyncChecker::new(&interner, &symbol_types);

        assert!(!checker.is_in_async_context());
    }

    #[test]
    fn test_create_async_iterator_type() {
        let interner = StringInterner::new();
        let elem_type = ResolvedType::String;

        let async_iter = create_async_iterator_type(&interner, elem_type);

        assert!(matches!(async_iter, ResolvedType::Reference(_)));
        if let ResolvedType::Reference(ref_type) = async_iter {
            let name = interner.lookup(ref_type.name).unwrap_or_default();
            assert_eq!(name, "AsyncIterator");
            assert_eq!(ref_type.type_arguments.len(), 1);
        }
    }

    #[test]
    fn test_create_async_iterable_type() {
        let interner = StringInterner::new();
        let elem_type = ResolvedType::Number;

        let async_iterable = create_async_iterable_type(&interner, elem_type);

        assert!(matches!(async_iterable, ResolvedType::Reference(_)));
        if let ResolvedType::Reference(ref_type) = async_iterable {
            let name = interner.lookup(ref_type.name).unwrap_or_default();
            assert_eq!(name, "AsyncIterable");
        }
    }
}
