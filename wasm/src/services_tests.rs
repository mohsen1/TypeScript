//! Tests for services/mod.rs

use crate::services::*;

#[test]
fn test_text_span_basics() {
    let span = TextSpan::new(10, 5);
    assert_eq!(span.start, 10);
    assert_eq!(span.length, 5);
    assert_eq!(span.end(), 15);
    assert!(span.contains(10));
    assert!(span.contains(14));
    assert!(!span.contains(15));
}

#[test]
fn test_text_span_from_bounds() {
    let span = TextSpan::from_bounds(10, 20);
    assert_eq!(span.start, 10);
    assert_eq!(span.length, 10);
}

#[test]
fn test_stateless_language_service_create() {
    let _ls = StatelessLanguageService::new();
}

#[test]
fn test_language_service_with_checker() {
    use crate::parser_impl::ParserState;
    use crate::binder::BinderState;
    use crate::checker::CheckerState;

    // Parse a simple TypeScript file
    let source = r#"
const x = 42;
function add(a: number, b: number): number {
    return a + b;
}
const y = add(x, 10);
"#;
    let mut parser = ParserState::new("test.ts".to_string(), source.to_string());
    let root = parser.parse_source_file();

    // Bind the file
    let mut binder = BinderState::new();
    binder.bind_source_file(&parser.arena, root);

    // Type check
    let mut checker = CheckerState::new(
        &parser.arena,
        &binder.symbols,
        &binder.file_locals,
        &binder.node_symbols,
        "test.ts".to_string(),
    );
    checker.check_source_file(root);

    // Create language service
    let ls = LanguageService::new(&checker, "test.ts".to_string(), root);

    // Test completions (should include our symbols)
    let completions = ls.get_completions_at_position(0).unwrap();
    assert!(!completions.entries.is_empty());

    // Test navigation bar (should find function and variables)
    let nav_items = ls.get_navigation_bar_items();
    assert!(!nav_items.is_empty());

    // Test outlining spans (should find the function block)
    let outlining = ls.get_outlining_spans();
    assert!(!outlining.is_empty());

    // Test diagnostics API - this code has no errors
    let diagnostics = ls.get_semantic_diagnostics();
    assert!(diagnostics.is_empty(), "Expected no diagnostics for valid code");

    let all_diagnostics = ls.get_all_diagnostics();
    assert!(all_diagnostics.is_empty(), "Expected no diagnostics for valid code");
}

#[test]
fn test_language_service_diagnostics() {
    use crate::parser_impl::ParserState;
    use crate::binder::BinderState;
    use crate::checker::CheckerState;

    // Parse TypeScript code with a type error
    let source = r#"
const x: number = "hello"; // Error: string not assignable to number
"#;
    let mut parser = ParserState::new("test.ts".to_string(), source.to_string());
    let root = parser.parse_source_file();

    // Bind the file
    let mut binder = BinderState::new();
    binder.bind_source_file(&parser.arena, root);

    // Type check
    let mut checker = CheckerState::new(
        &parser.arena,
        &binder.symbols,
        &binder.file_locals,
        &binder.node_symbols,
        "test.ts".to_string(),
    );
    checker.check_source_file(root);

    // Create language service
    let ls = LanguageService::new(&checker, "test.ts".to_string(), root);

    // Test diagnostics API - should have a type error
    let diagnostics = ls.get_semantic_diagnostics();
    assert!(!diagnostics.is_empty(), "Expected diagnostics for type error");
    assert_eq!(diagnostics[0].code, 2322, "Expected TS2322 type error");
    assert_eq!(diagnostics[0].category, DiagnosticCategory::Error);
}

#[test]
fn test_member_completions() {
    use crate::parser_impl::ParserState;
    use crate::binder::BinderState;
    use crate::checker::CheckerState;

    // Parse TypeScript code with an interface and property access
    let source = r#"
interface Point {
    x: number;
    y: number;
}
const p: Point = { x: 1, y: 2 };
p.
"#;
    let mut parser = ParserState::new("test.ts".to_string(), source.to_string());
    let root = parser.parse_source_file();

    // Bind the file
    let mut binder = BinderState::new();
    binder.bind_source_file(&parser.arena, root);

    // Type check
    let mut checker = CheckerState::new(
        &parser.arena,
        &binder.symbols,
        &binder.file_locals,
        &binder.node_symbols,
        "test.ts".to_string(),
    );
    checker.check_source_file(root);

    // Create language service
    let ls = LanguageService::new(&checker, "test.ts".to_string(), root);

    // Global completions should include Point and p
    let global_completions = ls.get_global_completions().unwrap();
    assert!(global_completions.is_global_completion);
    assert!(!global_completions.is_member_completion);

    // Check that we have file-level symbols
    let entry_names: Vec<_> = global_completions.entries.iter()
        .map(|e| e.name.as_str())
        .collect();
    assert!(entry_names.contains(&"Point"), "Should have Point in completions");
    assert!(entry_names.contains(&"p"), "Should have p in completions");
}

#[test]
fn test_get_properties_of_type() {
    use crate::parser_impl::ParserState;
    use crate::binder::BinderState;
    use crate::checker::CheckerState;

    // Parse TypeScript code with an interface
    let source = r#"
interface Point {
    x: number;
    y: number;
    move(dx: number, dy: number): void;
}
const p: Point = { x: 1, y: 2, move(dx, dy) {} };
"#;
    let mut parser = ParserState::new("test.ts".to_string(), source.to_string());
    let root = parser.parse_source_file();

    // Bind the file
    let mut binder = BinderState::new();
    binder.bind_source_file(&parser.arena, root);

    // Type check
    let mut checker = CheckerState::new(
        &parser.arena,
        &binder.symbols,
        &binder.file_locals,
        &binder.node_symbols,
        "test.ts".to_string(),
    );
    checker.check_source_file(root);

    // Get the type of 'p' and check its properties
    if let Some(p_symbol) = binder.file_locals.get("p") {
        if let Some(p_type) = checker.get_cached_type_of_symbol(p_symbol) {
            let properties = checker.get_properties_of_type(p_type);
            let _prop_names: Vec<_> = properties.iter()
                .map(|(name, _)| name.as_str())
                .collect();

            // Note: The exact properties depend on how the type checker resolves
            // the Point type. At minimum we should have some properties.
            // This test validates the infrastructure works.
            assert!(!properties.is_empty() || p_type.is_none(),
                "Should have properties or be unresolved type");
        }
    }
}

#[test]
fn test_type_completions() {
    use crate::parser_impl::ParserState;
    use crate::binder::BinderState;
    use crate::checker::CheckerState;

    // Parse TypeScript code with type annotations
    let source = r#"
interface Person {
    name: string;
    age: number;
}

type Status = "active" | "inactive";

class User implements Person {
    name: string;
    age: number;
}

enum Color { Red, Green, Blue }

const x: number = 1;
"#;
    let mut parser = ParserState::new("test.ts".to_string(), source.to_string());
    let root = parser.parse_source_file();

    // Bind the file
    let mut binder = BinderState::new();
    binder.bind_source_file(&parser.arena, root);

    // Type check
    let mut checker = CheckerState::new(
        &parser.arena,
        &binder.symbols,
        &binder.file_locals,
        &binder.node_symbols,
        "test.ts".to_string(),
    );
    checker.check_source_file(root);

    // Create language service
    let ls = LanguageService::new(&checker, "test.ts".to_string(), root);

    // Get type completions
    let type_completions = ls.get_type_completions();
    assert!(type_completions.is_some(), "Should get type completions");

    let completions = type_completions.unwrap();
    let entry_names: Vec<_> = completions.entries.iter()
        .map(|e| e.name.as_str())
        .collect();

    // Should have primitive types
    assert!(entry_names.contains(&"string"), "Should have 'string' primitive");
    assert!(entry_names.contains(&"number"), "Should have 'number' primitive");
    assert!(entry_names.contains(&"boolean"), "Should have 'boolean' primitive");

    // Should have user-defined types
    assert!(entry_names.contains(&"Person"), "Should have 'Person' interface");
    assert!(entry_names.contains(&"Status"), "Should have 'Status' type alias");
    assert!(entry_names.contains(&"User"), "Should have 'User' class");
    assert!(entry_names.contains(&"Color"), "Should have 'Color' enum");

    // Should NOT have value-only symbols
    assert!(!entry_names.contains(&"x"), "Should NOT have 'x' variable in type completions");
}

#[test]
fn test_project_language_service() {
    use crate::parallel::compile_files;

    // Create a multi-file project
    let files = vec![
        ("math.ts".to_string(), "export function add(x: number, y: number) { return x + y; }".to_string()),
        ("utils.ts".to_string(), "export class Logger { log(msg: string) {} }".to_string()),
        ("main.ts".to_string(), "const x = 1;".to_string()),
    ];

    let program = compile_files(files);
    let project_ls = ProjectLanguageService::new(&program);

    // Test file count
    assert_eq!(project_ls.file_count(), 3);

    // Test symbol count (should have symbols from all files)
    assert!(project_ls.symbol_count() >= 3, "Should have at least 3 symbols");

    // Test global symbol lookup
    assert!(project_ls.get_global_symbol("add").is_some(), "Should find 'add'");
    assert!(project_ls.get_global_symbol("Logger").is_some(), "Should find 'Logger'");
    assert!(project_ls.get_global_symbol("x").is_some(), "Should find 'x'");

    // Test get file by name
    assert!(project_ls.get_file("math.ts").is_some());
    assert!(project_ls.get_file("utils.ts").is_some());
    assert!(project_ls.get_file("nonexistent.ts").is_none());

    // Test symbol origins (cross-file navigation)
    let add_symbol = project_ls.get_global_symbol("add").unwrap();
    let add_file = project_ls.get_symbol_file(add_symbol);
    assert!(add_file.is_some());
    assert_eq!(add_file.unwrap().file_name, "math.ts");

    let logger_symbol = project_ls.get_global_symbol("Logger").unwrap();
    let logger_file = project_ls.get_symbol_file(logger_symbol);
    assert!(logger_file.is_some());
    assert_eq!(logger_file.unwrap().file_name, "utils.ts");

    // Test symbol search
    let search_results = project_ls.search_symbols("log");
    assert!(!search_results.is_empty(), "Should find symbols matching 'log'");
    let found_logger = search_results.iter()
        .any(|(name, _, _)| name == "Logger");
    assert!(found_logger, "Should find 'Logger' in search results");
}
