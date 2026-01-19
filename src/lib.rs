//! TypeScript Parser Library
//!
//! A lightweight TypeScript parser implemented in Rust.
//! Features:
//! - Zero-copy parsing using &str references
//! - Iterative algorithms to avoid stack overflow
//! - Proper error recovery without budget resets
//! - Enum and namespace type checking
//! - JavaScript transform/emit
//! - Intersection types and type compatibility
//! - Union type discrimination and exhaustiveness checking

pub mod binder;
pub mod checker;
pub mod parser;
pub mod solver;
pub mod thin_parser;
pub mod transforms;
pub mod types;

pub use binder::{EnumBinder, EnumBindingError, EnumMemberSymbol, EnumSymbol};
pub use checker::{
    CompatibilityChecker, CompatibilityError, CompatibilityOptions, CompatibilityResult,
    EnumChecker, EnumDeclaration, EnumError, EnumMember, EnumMemberValue, EnumType,
    ExhaustivenessChecker, ExhaustivenessResult, ExhaustivenessWarning, SwitchCase,
    ExportVisibility, NamespaceChecker, NamespaceDeclaration, NamespaceError,
    NamespaceMember, NamespaceMemberKind, ResolvedNamespace,
    UnionChecker, DiscriminantProperty, DiscriminatedUnionInfo, NarrowingResult,
    is_subtype, is_assignable_relaxed, are_equivalent, assert_never_check,
};
pub use parser::error_recovery::{ErrorRecovery, RecoveryStrategy, SyncPoint};
pub use solver::{
    IntersectionSolver, intersect_types, intersect_all, is_empty_intersection,
    UnionSolver, UnionSolverOptions, union_types, union_all, normalize_union,
};
pub use thin_parser::{AstNode, ParseError, Scanner, ThinParser, Token, TokenKind};
pub use transforms::{EnumTransformOptions, EnumTransformer, JsOutput};
pub use types::{
    Type, ObjectType, Property, FunctionType, TypeParameter, Parameter,
    create_intersection, flatten_intersection, merge_object_types,
};

/// Parse TypeScript source code and return the AST
///
/// # Example
/// ```
/// use ts_parser::parse;
///
/// let source = "let x: number = 5;";
/// let result = parse(source);
/// assert!(result.is_ok());
/// ```
pub fn parse(source: &str) -> Result<AstNode<'_>, Vec<ParseError>> {
    let mut parser = ThinParser::new(source);
    parser.parse()
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_simple_variable() {
        let result = parse("let x = 5;");
        assert!(result.is_ok());
    }

    #[test]
    fn test_typed_variable() {
        let result = parse("const name: string = 'hello';");
        assert!(result.is_ok());
    }

    #[test]
    fn test_function_with_types() {
        let result = parse("function add(a: number, b: number): number { return a + b; }");
        assert!(result.is_ok());
    }

    #[test]
    fn test_interface() {
        let result = parse("interface User { name: string; age: number; }");
        assert!(result.is_ok());
    }

    #[test]
    fn test_class() {
        let result = parse("class Animal { }");
        assert!(result.is_ok());
    }

    #[test]
    fn test_import() {
        let result = parse(r#"import { Component } from "@angular/core";"#);
        assert!(result.is_ok());
    }

    #[test]
    fn test_export_function() {
        let result = parse("export function greet(name: string): void { console.log(name); }");
        assert!(result.is_ok());
    }

    #[test]
    fn test_arrow_function() {
        let result = parse("const add = (a: number, b: number) => a + b;");
        assert!(result.is_ok());
    }

    #[test]
    fn test_type_alias() {
        let result = parse("type ID = string | number;");
        assert!(result.is_ok());
    }

    #[test]
    fn test_error_recovery() {
        // Missing expression after =
        let mut parser = ThinParser::new("let x = ; let y = 5;");
        let _ = parser.parse();
        // Should have errors but not panic
        assert!(!parser.errors().is_empty());
    }

    #[test]
    fn test_complex_expression() {
        let result = parse("let x = 1 + 2 * 3 - 4 / 5;");
        assert!(result.is_ok());
    }

    #[test]
    fn test_if_else() {
        let result = parse("if (x > 0) { return x; } else { return -x; }");
        assert!(result.is_ok());
    }

    #[test]
    fn test_for_loop() {
        let result = parse("for (let i = 0; i < 10; i++) { console.log(i); }");
        assert!(result.is_ok());
    }

    #[test]
    fn test_while_loop() {
        let result = parse("while (x > 0) { x--; }");
        assert!(result.is_ok());
    }

    #[test]
    fn test_object_literal() {
        let result = parse("const obj = { name: 'test', value: 42 };");
        assert!(result.is_ok());
    }

    #[test]
    fn test_array_literal() {
        let result = parse("const arr = [1, 2, 3, 4, 5];");
        assert!(result.is_ok());
    }

    #[test]
    fn test_method_call() {
        let result = parse("console.log('hello');");
        assert!(result.is_ok());
    }

    #[test]
    fn test_chained_method_calls() {
        let result = parse("arr.map(x => x * 2).filter(x => x > 5);");
        assert!(result.is_ok());
    }

    #[test]
    fn test_new_expression() {
        let result = parse("const date = new Date();");
        assert!(result.is_ok());
    }

    #[test]
    fn test_ternary() {
        let result = parse("const result = x > 0 ? 'positive' : 'negative';");
        assert!(result.is_ok());
    }

    #[test]
    fn test_generic_type() {
        let result = parse("const arr: Array<number> = [];");
        assert!(result.is_ok());
    }

    #[test]
    fn test_union_type() {
        let result = parse("let value: string | number;");
        assert!(result.is_ok());
    }

    #[test]
    fn test_multiline() {
        let source = r#"
            function greet(name: string): string {
                return 'Hello, ' + name;
            }

            const message = greet('World');
            console.log(message);
        "#;
        let result = parse(source);
        assert!(result.is_ok());
    }

    #[test]
    fn test_comments() {
        let source = r#"
            // This is a comment
            let x = 5; // inline comment
            /* Multi-line
               comment */
            let y = 10;
        "#;
        let result = parse(source);
        assert!(result.is_ok());
    }
}
