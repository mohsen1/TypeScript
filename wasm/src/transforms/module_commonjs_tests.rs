use super::module_commonjs::*;

#[test]
fn test_sanitize_module_name() {
    assert_eq!(sanitize_module_name("./foo"), "foo");
    assert_eq!(sanitize_module_name("./foo/bar"), "foo_bar");
    assert_eq!(sanitize_module_name("../utils"), "utils");
    assert_eq!(sanitize_module_name("@scope/pkg"), "_scope_pkg");
    assert_eq!(sanitize_module_name("../foo-bar.baz/qux"), "foo_bar_baz_qux");
    assert_eq!(
        sanitize_module_name("@scope/foo-bar/baz.qux"),
        "_scope_foo_bar_baz_qux"
    );
    assert_eq!(sanitize_module_name("foo/bar"), "foo_bar");
    assert_eq!(sanitize_module_name("foo-bar"), "foo_bar");
    assert_eq!(sanitize_module_name("foo.bar/baz"), "foo_bar_baz");
    assert_eq!(sanitize_module_name("@scope/pkg/sub"), "_scope_pkg_sub");
}

#[test]
fn test_emit_commonjs_preamble() {
    let mut output = String::new();
    emit_commonjs_preamble(&mut output).unwrap();
    assert!(output.contains("\"use strict\";"));
    assert!(output.contains("Object.defineProperty(exports, \"__esModule\""));
}

#[test]
fn test_emit_exports_init() {
    let mut output = String::new();
    emit_exports_init(&mut output, &["foo".to_string(), "bar".to_string()]).unwrap();
    assert_eq!(output, "exports.foo = exports.bar = void 0;\n");
}

#[test]
fn test_emit_export_assignment() {
    assert_eq!(emit_export_assignment("foo"), "exports.foo = foo;");
}

#[test]
fn test_emit_reexport_property() {
    let result = emit_reexport_property("foo", "module_1", "foo");
    assert!(result.contains("Object.defineProperty"));
    assert!(result.contains("\"foo\""));
    assert!(result.contains("module_1.foo"));
}

#[test]
fn test_collect_export_names_with_parsed_ast() {
    use crate::parser::syntax_kind_ext;
    use crate::scanner::SyntaxKind;
    use crate::thin_parser::ThinParserState;

    let source = "export class C {}";
    let mut parser = ThinParserState::new("test.ts".to_string(), source.to_string());
    let root = parser.parse_source_file();

    let Some(source_file) = parser.arena.get_source_file(parser.arena.get(root).unwrap()) else {
        panic!("Failed to get source file");
    };

    // Debug: print the statements
    eprintln!("Source file has {} statements", source_file.statements.nodes.len());
    for (i, &stmt_idx) in source_file.statements.nodes.iter().enumerate() {
        if let Some(node) = parser.arena.get(stmt_idx) {
            eprintln!(
                "Statement {}: kind = {} (ClassDecl = {})",
                i,
                node.kind,
                syntax_kind_ext::CLASS_DECLARATION
            );

            if node.kind == syntax_kind_ext::CLASS_DECLARATION {
                if let Some(class) = parser.arena.get_class(node) {
                    eprintln!("  Found class, modifiers: {:?}", class.modifiers);
                    if let Some(modifiers) = &class.modifiers {
                        eprintln!("  Modifiers count: {}", modifiers.nodes.len());
                        for &mod_idx in &modifiers.nodes {
                            if let Some(mod_node) = parser.arena.get(mod_idx) {
                                eprintln!(
                                    "    Modifier kind: {} (Export = {})",
                                    mod_node.kind,
                                    SyntaxKind::ExportKeyword as u16
                                );
                            }
                        }
                    }
                    if let Some(name_node) = parser.arena.get(class.name) {
                        if let Some(ident) = parser.arena.get_identifier(name_node) {
                            eprintln!("  Class name: {}", ident.escaped_text);
                        }
                    }
                }
            }
        }
    }

    let export_names = collect_export_names(&parser.arena, &source_file.statements.nodes);

    eprintln!("Collected export names: {:?}", export_names);

    assert!(
        !export_names.is_empty(),
        "Expected to find exported class name"
    );
    assert_eq!(
        export_names,
        vec!["C"],
        "Expected to find class name 'C' in exports"
    );
}

#[test]
fn test_collect_export_names_with_destructuring() {
    use crate::thin_parser::ThinParserState;

    let source = "export const { a, b: c } = obj;";
    let mut parser = ThinParserState::new("test.ts".to_string(), source.to_string());
    let root = parser.parse_source_file();

    let Some(source_file) = parser.arena.get_source_file(parser.arena.get(root).unwrap()) else {
        panic!("Failed to get source file");
    };

    let export_names = collect_export_names(&parser.arena, &source_file.statements.nodes);

    assert_eq!(
        export_names,
        vec!["a", "c"],
        "Expected destructured export names"
    );
}

#[test]
fn test_collect_export_names_with_string_literal_destructuring() {
    use crate::thin_parser::ThinParserState;

    let source = "export const { \"foo-bar\": fooBar, baz } = obj;";
    let mut parser = ThinParserState::new("test.ts".to_string(), source.to_string());
    let root = parser.parse_source_file();

    let Some(source_file) = parser.arena.get_source_file(parser.arena.get(root).unwrap()) else {
        panic!("Failed to get source file");
    };

    let export_names = collect_export_names(&parser.arena, &source_file.statements.nodes);

    assert_eq!(
        export_names,
        vec!["fooBar", "baz"],
        "Expected binding names from string literal destructuring"
    );
}

#[test]
fn test_collect_export_names_with_nested_destructuring() {
    use crate::thin_parser::ThinParserState;

    let source = "export const { a: { b }, c: [d] } = obj;";
    let mut parser = ThinParserState::new("test.ts".to_string(), source.to_string());
    let root = parser.parse_source_file();

    let Some(source_file) = parser.arena.get_source_file(parser.arena.get(root).unwrap()) else {
        panic!("Failed to get source file");
    };

    let export_names = collect_export_names(&parser.arena, &source_file.statements.nodes);

    assert_eq!(
        export_names,
        vec!["b", "d"],
        "Expected binding names from nested destructuring"
    );
}

#[test]
fn test_collect_export_names_with_array_destructuring_rest() {
    use crate::thin_parser::ThinParserState;

    let source = "export const [a, , ...rest] = arr;";
    let mut parser = ThinParserState::new("test.ts".to_string(), source.to_string());
    let root = parser.parse_source_file();

    let Some(source_file) = parser.arena.get_source_file(parser.arena.get(root).unwrap()) else {
        panic!("Failed to get source file");
    };

    let export_names = collect_export_names(&parser.arena, &source_file.statements.nodes);

    assert_eq!(
        export_names,
        vec!["a", "rest"],
        "Expected binding names from array destructuring with rest"
    );
}

#[test]
fn test_collect_export_names_with_object_destructuring_rest() {
    use crate::thin_parser::ThinParserState;

    let source = "export const { a, ...rest } = obj;";
    let mut parser = ThinParserState::new("test.ts".to_string(), source.to_string());
    let root = parser.parse_source_file();

    let Some(source_file) = parser.arena.get_source_file(parser.arena.get(root).unwrap()) else {
        panic!("Failed to get source file");
    };

    let export_names = collect_export_names(&parser.arena, &source_file.statements.nodes);

    assert_eq!(
        export_names,
        vec!["a", "rest"],
        "Expected binding names from object destructuring with rest"
    );
}

#[test]
fn test_collect_export_names_with_destructuring_defaults() {
    use crate::thin_parser::ThinParserState;

    let source = "export const { a = 1, b: c = 2 } = obj;";
    let mut parser = ThinParserState::new("test.ts".to_string(), source.to_string());
    let root = parser.parse_source_file();

    let Some(source_file) = parser.arena.get_source_file(parser.arena.get(root).unwrap()) else {
        panic!("Failed to get source file");
    };

    let export_names = collect_export_names(&parser.arena, &source_file.statements.nodes);

    assert_eq!(
        export_names,
        vec!["a", "c"],
        "Expected binding names from destructuring with defaults"
    );
}

#[test]
fn test_collect_export_names_with_array_destructuring_defaults() {
    use crate::thin_parser::ThinParserState;

    let source = "export const [a = 1, b] = arr;";
    let mut parser = ThinParserState::new("test.ts".to_string(), source.to_string());
    let root = parser.parse_source_file();

    let Some(source_file) = parser.arena.get_source_file(parser.arena.get(root).unwrap()) else {
        panic!("Failed to get source file");
    };

    let export_names = collect_export_names(&parser.arena, &source_file.statements.nodes);

    assert_eq!(
        export_names,
        vec!["a", "b"],
        "Expected binding names from array destructuring with defaults"
    );
}

#[test]
fn test_collect_export_names_with_object_destructuring_alias_and_rest() {
    use crate::thin_parser::ThinParserState;

    let source = "export const { a: b, ...rest } = obj;";
    let mut parser = ThinParserState::new("test.ts".to_string(), source.to_string());
    let root = parser.parse_source_file();

    let Some(source_file) = parser.arena.get_source_file(parser.arena.get(root).unwrap()) else {
        panic!("Failed to get source file");
    };

    let export_names = collect_export_names(&parser.arena, &source_file.statements.nodes);

    assert_eq!(
        export_names,
        vec!["b", "rest"],
        "Expected binding names from alias + rest destructuring"
    );
}

#[test]
fn test_collect_export_names_with_default_export() {
    use crate::thin_parser::ThinParserState;

    let source = "export default function () {}";
    let mut parser = ThinParserState::new("test.ts".to_string(), source.to_string());
    let root = parser.parse_source_file();

    let Some(source_file) = parser.arena.get_source_file(parser.arena.get(root).unwrap()) else {
        panic!("Failed to get source file");
    };

    let export_names = collect_export_names(&parser.arena, &source_file.statements.nodes);

    assert_eq!(export_names, vec!["default"], "Expected default export name");
}

#[test]
fn test_collect_export_names_with_default_export_class_and_named_export() {
    use crate::thin_parser::ThinParserState;

    let source = "export default class Foo {}\nexport const bar = 1;";
    let mut parser = ThinParserState::new("test.ts".to_string(), source.to_string());
    let root = parser.parse_source_file();

    let Some(source_file) = parser.arena.get_source_file(parser.arena.get(root).unwrap()) else {
        panic!("Failed to get source file");
    };

    let export_names = collect_export_names(&parser.arena, &source_file.statements.nodes);

    assert_eq!(
        export_names,
        vec!["default", "bar"],
        "Expected default and named exports"
    );
}

#[test]
fn test_collect_export_names_with_exported_namespace() {
    use crate::thin_parser::ThinParserState;

    let source = "export namespace Foo { export const bar = 1; }";
    let mut parser = ThinParserState::new("test.ts".to_string(), source.to_string());
    let root = parser.parse_source_file();

    let Some(source_file) = parser.arena.get_source_file(parser.arena.get(root).unwrap()) else {
        panic!("Failed to get source file");
    };

    let export_names = collect_export_names(&parser.arena, &source_file.statements.nodes);

    assert_eq!(
        export_names,
        vec!["Foo"],
        "Expected exported namespace name"
    );
}

#[test]
fn test_collect_export_names_with_exported_enum() {
    use crate::thin_parser::ThinParserState;

    let source = "export enum Foo { A, B }";
    let mut parser = ThinParserState::new("test.ts".to_string(), source.to_string());
    let root = parser.parse_source_file();

    let Some(source_file) = parser.arena.get_source_file(parser.arena.get(root).unwrap()) else {
        panic!("Failed to get source file");
    };

    let export_names = collect_export_names(&parser.arena, &source_file.statements.nodes);

    assert_eq!(export_names, vec!["Foo"], "Expected exported enum name");
}

#[test]
fn test_collect_export_names_with_named_exports() {
    use crate::thin_parser::ThinParserState;

    let source = "const foo = 1; export { foo as bar };";
    let mut parser = ThinParserState::new("test.ts".to_string(), source.to_string());
    let root = parser.parse_source_file();

    let Some(source_file) = parser.arena.get_source_file(parser.arena.get(root).unwrap()) else {
        panic!("Failed to get source file");
    };

    let export_names = collect_export_names(&parser.arena, &source_file.statements.nodes);

    assert_eq!(
        export_names,
        vec!["bar"],
        "Expected exported name from named export"
    );
}

#[test]
fn test_collect_export_names_with_string_named_export() {
    use crate::thin_parser::ThinParserState;

    let source = "const foo = 1; export { foo as \"default\" };";
    let mut parser = ThinParserState::new("test.ts".to_string(), source.to_string());
    let root = parser.parse_source_file();

    let Some(source_file) = parser.arena.get_source_file(parser.arena.get(root).unwrap()) else {
        panic!("Failed to get source file");
    };

    let export_names = collect_export_names(&parser.arena, &source_file.statements.nodes);

    assert_eq!(
        export_names,
        vec!["default"],
        "Expected string-named export to be collected"
    );
}

#[test]
fn test_collect_export_names_with_default_named_export() {
    use crate::thin_parser::ThinParserState;

    let source = "const foo = 1; export { foo as default };";
    let mut parser = ThinParserState::new("test.ts".to_string(), source.to_string());
    let root = parser.parse_source_file();

    let Some(source_file) = parser.arena.get_source_file(parser.arena.get(root).unwrap()) else {
        panic!("Failed to get source file");
    };

    let export_names = collect_export_names(&parser.arena, &source_file.statements.nodes);

    assert_eq!(
        export_names,
        vec!["default"],
        "Expected default export name from named export"
    );
}

#[test]
fn test_collect_export_names_with_shorthand_export() {
    use crate::thin_parser::ThinParserState;

    let source = "const foo = 1; export { foo };";
    let mut parser = ThinParserState::new("test.ts".to_string(), source.to_string());
    let root = parser.parse_source_file();

    let Some(source_file) = parser.arena.get_source_file(parser.arena.get(root).unwrap()) else {
        panic!("Failed to get source file");
    };

    let export_names = collect_export_names(&parser.arena, &source_file.statements.nodes);

    assert_eq!(
        export_names,
        vec!["foo"],
        "Expected exported name from shorthand export"
    );
}

#[test]
fn test_collect_export_names_ignores_type_only_specifiers() {
    use crate::thin_parser::ThinParserState;

    let source = "type Foo = number; const foo = 1; export { foo, type Foo };";
    let mut parser = ThinParserState::new("test.ts".to_string(), source.to_string());
    let root = parser.parse_source_file();

    let Some(source_file) = parser.arena.get_source_file(parser.arena.get(root).unwrap()) else {
        panic!("Failed to get source file");
    };

    let export_names = collect_export_names(&parser.arena, &source_file.statements.nodes);

    assert_eq!(
        export_names,
        vec!["foo"],
        "Expected type-only specifiers to be ignored"
    );
}

#[test]
fn test_collect_export_names_with_alias_and_type_only_specifiers() {
    use crate::thin_parser::ThinParserState;

    let source = "type Foo = number; const bar = 1; export { bar as baz, type Foo };";
    let mut parser = ThinParserState::new("test.ts".to_string(), source.to_string());
    let root = parser.parse_source_file();

    let Some(source_file) = parser.arena.get_source_file(parser.arena.get(root).unwrap()) else {
        panic!("Failed to get source file");
    };

    let export_names = collect_export_names(&parser.arena, &source_file.statements.nodes);

    assert_eq!(
        export_names,
        vec!["baz"],
        "Expected alias export with type-only specifiers to be collected"
    );
}

#[test]
fn test_collect_export_names_ignores_only_type_specifiers() {
    use crate::thin_parser::ThinParserState;

    let source = "type Foo = number; export { type Foo };";
    let mut parser = ThinParserState::new("test.ts".to_string(), source.to_string());
    let root = parser.parse_source_file();

    let Some(source_file) = parser.arena.get_source_file(parser.arena.get(root).unwrap()) else {
        panic!("Failed to get source file");
    };

    let export_names = collect_export_names(&parser.arena, &source_file.statements.nodes);

    assert!(
        export_names.is_empty(),
        "Expected only type-only specifiers to be ignored"
    );
}

#[test]
fn test_collect_export_names_with_empty_exports() {
    use crate::thin_parser::ThinParserState;

    let source = "export {};";
    let mut parser = ThinParserState::new("test.ts".to_string(), source.to_string());
    let root = parser.parse_source_file();

    let Some(source_file) = parser.arena.get_source_file(parser.arena.get(root).unwrap()) else {
        panic!("Failed to get source file");
    };

    let export_names = collect_export_names(&parser.arena, &source_file.statements.nodes);

    assert!(
        export_names.is_empty(),
        "Expected empty export clause to produce no exports"
    );
}

#[test]
fn test_collect_export_names_ignores_type_only_alias_specifier() {
    use crate::thin_parser::ThinParserState;

    let source = "type Foo = number; export { type Foo as Bar };";
    let mut parser = ThinParserState::new("test.ts".to_string(), source.to_string());
    let root = parser.parse_source_file();

    let Some(source_file) = parser.arena.get_source_file(parser.arena.get(root).unwrap()) else {
        panic!("Failed to get source file");
    };

    let export_names = collect_export_names(&parser.arena, &source_file.statements.nodes);

    assert!(
        export_names.is_empty(),
        "Expected type-only alias specifier to be ignored"
    );
}

#[test]
fn test_collect_export_names_ignores_type_only_reexport() {
    use crate::thin_parser::ThinParserState;

    let source = "export type { Foo } from \"./foo\";";
    let mut parser = ThinParserState::new("test.ts".to_string(), source.to_string());
    let root = parser.parse_source_file();

    let Some(source_file) = parser.arena.get_source_file(parser.arena.get(root).unwrap()) else {
        panic!("Failed to get source file");
    };

    let export_names = collect_export_names(&parser.arena, &source_file.statements.nodes);

    assert!(
        export_names.is_empty(),
        "Expected no runtime exports for type-only re-export"
    );
}

#[test]
fn test_collect_export_names_ignores_type_only_star_reexport() {
    use crate::thin_parser::ThinParserState;

    let source = "export type * from \"./foo\";";
    let mut parser = ThinParserState::new("test.ts".to_string(), source.to_string());
    let root = parser.parse_source_file();

    let Some(source_file) = parser.arena.get_source_file(parser.arena.get(root).unwrap()) else {
        panic!("Failed to get source file");
    };

    let export_names = collect_export_names(&parser.arena, &source_file.statements.nodes);

    assert!(
        export_names.is_empty(),
        "Expected no runtime exports for type-only star re-export"
    );
}

#[test]
fn test_collect_export_names_ignores_type_only_namespace_reexport() {
    use crate::thin_parser::ThinParserState;

    let source = "export type * as Foo from \"./foo\";";
    let mut parser = ThinParserState::new("test.ts".to_string(), source.to_string());
    let root = parser.parse_source_file();

    let Some(source_file) = parser.arena.get_source_file(parser.arena.get(root).unwrap()) else {
        panic!("Failed to get source file");
    };

    let export_names = collect_export_names(&parser.arena, &source_file.statements.nodes);

    assert!(
        export_names.is_empty(),
        "Expected no runtime exports for type-only namespace re-export"
    );
}

#[test]
fn test_collect_export_names_with_multiple_named_exports() {
    use crate::thin_parser::ThinParserState;

    let source = "const foo = 1; const bar = 2; export { foo, bar as baz };";
    let mut parser = ThinParserState::new("test.ts".to_string(), source.to_string());
    let root = parser.parse_source_file();

    let Some(source_file) = parser.arena.get_source_file(parser.arena.get(root).unwrap()) else {
        panic!("Failed to get source file");
    };

    let export_names = collect_export_names(&parser.arena, &source_file.statements.nodes);

    assert_eq!(
        export_names,
        vec!["foo", "baz"],
        "Expected multiple exported names"
    );
}

#[test]
fn test_collect_export_names_with_string_named_exports() {
    use crate::thin_parser::ThinParserState;

    let source = "const foo = 1; export { foo as \"foo-bar\" };";
    let mut parser = ThinParserState::new("test.ts".to_string(), source.to_string());
    let root = parser.parse_source_file();

    let Some(source_file) = parser.arena.get_source_file(parser.arena.get(root).unwrap()) else {
        panic!("Failed to get source file");
    };

    let export_names = collect_export_names(&parser.arena, &source_file.statements.nodes);

    assert_eq!(
        export_names,
        vec!["foo-bar"],
        "Expected string-named export to be collected"
    );
}

#[test]
fn test_collect_export_names_with_export_import_equals() {
    use crate::thin_parser::ThinParserState;

    let source = "export import Foo = require(\"./bar\");";
    let mut parser = ThinParserState::new("test.ts".to_string(), source.to_string());
    let root = parser.parse_source_file();

    let Some(source_file) = parser.arena.get_source_file(parser.arena.get(root).unwrap()) else {
        panic!("Failed to get source file");
    };

    let export_names = collect_export_names(&parser.arena, &source_file.statements.nodes);

    assert_eq!(
        export_names,
        vec!["Foo"],
        "Expected export name from export import equals"
    );
}

#[test]
fn test_collect_export_names_ignores_type_only_declarations() {
    use crate::thin_parser::ThinParserState;

    let source = "export type Foo = number; export interface Bar { x: number; }";
    let mut parser = ThinParserState::new("test.ts".to_string(), source.to_string());
    let root = parser.parse_source_file();

    let Some(source_file) = parser.arena.get_source_file(parser.arena.get(root).unwrap()) else {
        panic!("Failed to get source file");
    };

    let export_names = collect_export_names(&parser.arena, &source_file.statements.nodes);

    assert!(
        export_names.is_empty(),
        "Expected no runtime exports for type-only declarations"
    );
}

#[test]
fn test_collect_export_names_ignores_declare_exports() {
    use crate::thin_parser::ThinParserState;

    let source = "export declare const foo: number; export declare function bar(): void;";
    let mut parser = ThinParserState::new("test.ts".to_string(), source.to_string());
    let root = parser.parse_source_file();

    let Some(source_file) = parser.arena.get_source_file(parser.arena.get(root).unwrap()) else {
        panic!("Failed to get source file");
    };

    let export_names = collect_export_names(&parser.arena, &source_file.statements.nodes);

    assert!(
        export_names.is_empty(),
        "Expected no runtime exports for declare-only exports"
    );
}

#[test]
fn test_collect_export_names_ignores_declare_namespace() {
    use crate::thin_parser::ThinParserState;

    let source = "export declare namespace Foo { export const bar: number; }";
    let mut parser = ThinParserState::new("test.ts".to_string(), source.to_string());
    let root = parser.parse_source_file();

    let Some(source_file) = parser.arena.get_source_file(parser.arena.get(root).unwrap()) else {
        panic!("Failed to get source file");
    };

    let export_names = collect_export_names(&parser.arena, &source_file.statements.nodes);

    assert!(
        export_names.is_empty(),
        "Expected no runtime exports for declare namespace"
    );
}

#[test]
fn test_collect_export_names_ignores_reexports() {
    use crate::thin_parser::ThinParserState;

    let source = "export * from \"./foo\"; export { bar } from \"./bar\";";
    let mut parser = ThinParserState::new("test.ts".to_string(), source.to_string());
    let root = parser.parse_source_file();

    let Some(source_file) = parser.arena.get_source_file(parser.arena.get(root).unwrap()) else {
        panic!("Failed to get source file");
    };

    let export_names = collect_export_names(&parser.arena, &source_file.statements.nodes);

    assert!(
        export_names.is_empty(),
        "Expected no runtime exports for re-exports"
    );
}

#[test]
fn test_collect_export_names_ignores_reexports_with_type_only_specifiers() {
    use crate::thin_parser::ThinParserState;

    let source = "export { foo, type Bar } from \"./foo\";";
    let mut parser = ThinParserState::new("test.ts".to_string(), source.to_string());
    let root = parser.parse_source_file();

    let Some(source_file) = parser.arena.get_source_file(parser.arena.get(root).unwrap()) else {
        panic!("Failed to get source file");
    };

    let export_names = collect_export_names(&parser.arena, &source_file.statements.nodes);

    assert!(
        export_names.is_empty(),
        "Expected no runtime exports for re-exports with type-only specifiers"
    );
}

#[test]
fn test_collect_export_names_ignores_type_only_reexport_alias() {
    use crate::thin_parser::ThinParserState;

    let source = "export { type Foo as Bar } from \"./foo\";";
    let mut parser = ThinParserState::new("test.ts".to_string(), source.to_string());
    let root = parser.parse_source_file();

    let Some(source_file) = parser.arena.get_source_file(parser.arena.get(root).unwrap()) else {
        panic!("Failed to get source file");
    };

    let export_names = collect_export_names(&parser.arena, &source_file.statements.nodes);

    assert!(
        export_names.is_empty(),
        "Expected no runtime exports for type-only re-export alias"
    );
}

#[test]
fn test_collect_export_names_ignores_reexports_with_aliases() {
    use crate::thin_parser::ThinParserState;

    let source = "export { foo as bar } from \"./foo\";";
    let mut parser = ThinParserState::new("test.ts".to_string(), source.to_string());
    let root = parser.parse_source_file();

    let Some(source_file) = parser.arena.get_source_file(parser.arena.get(root).unwrap()) else {
        panic!("Failed to get source file");
    };

    let export_names = collect_export_names(&parser.arena, &source_file.statements.nodes);

    assert!(
        export_names.is_empty(),
        "Expected no runtime exports for re-exports with aliases"
    );
}

#[test]
fn test_collect_export_names_ignores_default_reexport() {
    use crate::thin_parser::ThinParserState;

    let source = "export { default } from \"./foo\";";
    let mut parser = ThinParserState::new("test.ts".to_string(), source.to_string());
    let root = parser.parse_source_file();

    let Some(source_file) = parser.arena.get_source_file(parser.arena.get(root).unwrap()) else {
        panic!("Failed to get source file");
    };

    let export_names = collect_export_names(&parser.arena, &source_file.statements.nodes);

    assert!(
        export_names.is_empty(),
        "Expected no runtime exports for default re-export"
    );
}

#[test]
fn test_collect_export_names_ignores_default_reexport_alias() {
    use crate::thin_parser::ThinParserState;

    let source = "export { foo as default } from \"./foo\";";
    let mut parser = ThinParserState::new("test.ts".to_string(), source.to_string());
    let root = parser.parse_source_file();

    let Some(source_file) = parser.arena.get_source_file(parser.arena.get(root).unwrap()) else {
        panic!("Failed to get source file");
    };

    let export_names = collect_export_names(&parser.arena, &source_file.statements.nodes);

    assert!(
        export_names.is_empty(),
        "Expected no runtime exports for default re-export alias"
    );
}

#[test]
fn test_collect_export_names_ignores_export_assignment() {
    use crate::thin_parser::ThinParserState;

    let source = "export = foo;";
    let mut parser = ThinParserState::new("test.ts".to_string(), source.to_string());
    let root = parser.parse_source_file();

    let Some(source_file) = parser.arena.get_source_file(parser.arena.get(root).unwrap()) else {
        panic!("Failed to get source file");
    };

    let export_names = collect_export_names(&parser.arena, &source_file.statements.nodes);

    assert!(
        export_names.is_empty(),
        "Expected no runtime exports for export assignment"
    );
}

#[test]
fn test_collect_export_names_ignores_default_reexport_named() {
    use crate::thin_parser::ThinParserState;

    let source = "export { default as Foo } from \"./foo\";";
    let mut parser = ThinParserState::new("test.ts".to_string(), source.to_string());
    let root = parser.parse_source_file();

    let Some(source_file) = parser.arena.get_source_file(parser.arena.get(root).unwrap()) else {
        panic!("Failed to get source file");
    };

    let export_names = collect_export_names(&parser.arena, &source_file.statements.nodes);

    assert!(
        export_names.is_empty(),
        "Expected no runtime exports for default re-export named"
    );
}

#[test]
fn test_collect_export_names_ignores_const_enum() {
    use crate::thin_parser::ThinParserState;

    let source = "export const enum Foo { A }";
    let mut parser = ThinParserState::new("test.ts".to_string(), source.to_string());
    let root = parser.parse_source_file();

    let Some(source_file) = parser.arena.get_source_file(parser.arena.get(root).unwrap()) else {
        panic!("Failed to get source file");
    };

    let export_names = collect_export_names(&parser.arena, &source_file.statements.nodes);

    assert!(
        export_names.is_empty(),
        "Expected no runtime exports for const enums"
    );
}

#[test]
fn test_collect_export_names_ignores_declare_enum() {
    use crate::thin_parser::ThinParserState;

    let source = "export declare enum Foo { A }";
    let mut parser = ThinParserState::new("test.ts".to_string(), source.to_string());
    let root = parser.parse_source_file();

    let Some(source_file) = parser.arena.get_source_file(parser.arena.get(root).unwrap()) else {
        panic!("Failed to get source file");
    };

    let export_names = collect_export_names(&parser.arena, &source_file.statements.nodes);

    assert!(
        export_names.is_empty(),
        "Expected no runtime exports for declare enums"
    );
}
