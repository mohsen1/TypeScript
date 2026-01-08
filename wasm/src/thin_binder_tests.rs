//! Tests for ThinBinder

use crate::thin_parser::ThinParserState;
use crate::thin_binder::ThinBinderState;

#[test]
fn test_thin_binder_variable_declaration() {
    let mut parser = ThinParserState::new(
        "test.ts".to_string(),
        "let x = 1; const y = 2; var z = 3;".to_string(),
    );
    let root = parser.parse_source_file();

    let mut binder = ThinBinderState::new();
    binder.bind_source_file(parser.get_arena(), root);

    // Check that symbols were created
    assert!(binder.file_locals.has("x"));
    assert!(binder.file_locals.has("y"));
    assert!(binder.file_locals.has("z"));
}

#[test]
fn test_thin_binder_reset_clears_state() {
    let mut parser = ThinParserState::new(
        "test.ts".to_string(),
        "const a = 1;".to_string(),
    );
    let root = parser.parse_source_file();

    let mut binder = ThinBinderState::new();
    binder.bind_source_file(parser.get_arena(), root);

    assert!(binder.file_locals.has("a"));
    assert!(!binder.symbols.is_empty());
    assert!(!binder.node_symbols.is_empty());

    binder.reset();

    assert!(binder.file_locals.is_empty());
    assert!(binder.symbols.is_empty());
    assert!(binder.node_symbols.is_empty());
    assert_eq!(binder.flow_nodes.len(), 1);

    let mut parser = ThinParserState::new(
        "test.ts".to_string(),
        "const b = 2;".to_string(),
    );
    let root = parser.parse_source_file();
    binder.bind_source_file(parser.get_arena(), root);

    assert!(binder.file_locals.has("b"));
    assert!(!binder.file_locals.has("a"));
}

#[test]
fn test_thin_binder_function_declaration() {
    let mut parser = ThinParserState::new(
        "test.ts".to_string(),
        "function foo(a: number, b: string) { return a; }".to_string(),
    );
    let root = parser.parse_source_file();

    let mut binder = ThinBinderState::new();
    binder.bind_source_file(parser.get_arena(), root);

    // Check that function symbol was created
    assert!(binder.file_locals.has("foo"));
}

#[test]
fn test_thin_binder_class_declaration() {
    let mut parser = ThinParserState::new(
        "test.ts".to_string(),
        "class MyClass { x: number; foo() {} }".to_string(),
    );
    let root = parser.parse_source_file();

    let mut binder = ThinBinderState::new();
    binder.bind_source_file(parser.get_arena(), root);

    // Check that class symbol was created
    assert!(binder.file_locals.has("MyClass"));
}

#[test]
fn test_thin_binder_interface_declaration() {
    let mut parser = ThinParserState::new(
        "test.ts".to_string(),
        "interface IFoo { x: number; }".to_string(),
    );
    let root = parser.parse_source_file();

    let mut binder = ThinBinderState::new();
    binder.bind_source_file(parser.get_arena(), root);

    // Check that interface symbol was created
    assert!(binder.file_locals.has("IFoo"));
}

#[test]
fn test_thin_binder_type_alias() {
    let mut parser = ThinParserState::new(
        "test.ts".to_string(),
        "type MyType = string | number;".to_string(),
    );
    let root = parser.parse_source_file();

    let mut binder = ThinBinderState::new();
    binder.bind_source_file(parser.get_arena(), root);

    // Check that type alias symbol was created
    assert!(binder.file_locals.has("MyType"));
}

#[test]
fn test_thin_binder_enum_declaration() {
    let mut parser = ThinParserState::new(
        "test.ts".to_string(),
        "enum Color { Red, Green, Blue }".to_string(),
    );
    let root = parser.parse_source_file();

    let mut binder = ThinBinderState::new();
    binder.bind_source_file(parser.get_arena(), root);

    // Check that enum symbol was created
    assert!(binder.file_locals.has("Color"));
}

#[test]
fn test_thin_binder_import_declaration() {
    let mut parser = ThinParserState::new(
        "test.ts".to_string(),
        r#"import foo from 'module'; import { bar, baz as qux } from 'other';"#.to_string(),
    );
    let root = parser.parse_source_file();

    let mut binder = ThinBinderState::new();
    binder.bind_source_file(parser.get_arena(), root);

    // Default import creates alias symbol
    assert!(binder.file_locals.has("foo"));
    // Named imports create alias symbols
    assert!(binder.file_locals.has("bar"));
    assert!(binder.file_locals.has("qux"));  // aliased from baz
}

#[test]
fn test_thin_binder_export_declaration() {
    let mut parser = ThinParserState::new(
        "test.ts".to_string(),
        r#"const x = 1; export { x, x as y };"#.to_string(),
    );
    let root = parser.parse_source_file();

    let mut binder = ThinBinderState::new();
    binder.bind_source_file(parser.get_arena(), root);

    // Variable should be bound
    assert!(binder.file_locals.has("x"));

    // Export specifiers should have symbols (marked via node_symbols, not file_locals)
    // This ensures the binding runs without errors
    assert!(binder.symbols.len() > 1, "Should have created export symbols");
}

#[test]
fn test_thin_binder_exported_function() {
    let mut parser = ThinParserState::new(
        "test.ts".to_string(),
        r#"export function foo() { return 1; }"#.to_string(),
    );
    let root = parser.parse_source_file();

    let mut binder = ThinBinderState::new();
    binder.bind_source_file(parser.get_arena(), root);

    // Exported function should be bound to file_locals
    assert!(binder.file_locals.has("foo"), "Exported function 'foo' should be in file_locals");
}

#[test]
fn test_thin_binder_exported_class() {
    let mut parser = ThinParserState::new(
        "test.ts".to_string(),
        r#"export class MyClass { x: number; }"#.to_string(),
    );
    let root = parser.parse_source_file();

    let mut binder = ThinBinderState::new();
    binder.bind_source_file(parser.get_arena(), root);

    // Exported class should be bound to file_locals
    assert!(binder.file_locals.has("MyClass"), "Exported class 'MyClass' should be in file_locals");
}

#[test]
fn test_thin_binder_exported_const() {
    let mut parser = ThinParserState::new(
        "test.ts".to_string(),
        r#"export const x = 1, y = 2;"#.to_string(),
    );
    let root = parser.parse_source_file();

    let mut binder = ThinBinderState::new();
    binder.bind_source_file(parser.get_arena(), root);

    // Exported variables should be bound to file_locals
    assert!(binder.file_locals.has("x"), "Exported const 'x' should be in file_locals");
    assert!(binder.file_locals.has("y"), "Exported const 'y' should be in file_locals");
}

fn assert_bound_state_resolves_param_impl(
    source: &str,
    function_name: &str,
    param_name: &str,
    include_scopes: bool,
) {
    use crate::binder::SymbolTable;
    use crate::parallel;
    use crate::parser::{syntax_kind_ext, NodeIndex};

    let program = parallel::compile_files(vec![("test.ts".to_string(), source.to_string())]);
    let file = &program.files[0];

    let mut file_locals = SymbolTable::new();
    for (name, &sym_id) in program.file_locals[0].iter() {
        file_locals.set(name.clone(), sym_id);
    }
    for (name, &sym_id) in program.globals.iter() {
        if !file_locals.has(name) {
            file_locals.set(name.clone(), sym_id);
        }
    }

    let binder = if include_scopes {
        ThinBinderState::from_bound_state_with_scopes(
            program.symbols.clone(),
            file_locals,
            file.node_symbols.clone(),
            file.scopes.clone(),
            file.node_scope_ids.clone(),
        )
    } else {
        ThinBinderState::from_bound_state(
            program.symbols.clone(),
            file_locals,
            file.node_symbols.clone(),
        )
    };

    let arena = &file.arena;
    let mut param_name_idx = NodeIndex::NONE;
    let mut param_symbol = None;
    let mut function_body = NodeIndex::NONE;

    for i in 0..arena.len() {
        let idx = NodeIndex(i as u32);
        let Some(node) = arena.get(idx) else { continue; };
        if node.kind != syntax_kind_ext::FUNCTION_DECLARATION {
            continue;
        }
        let Some(func) = arena.get_function(node) else { continue; };
        let name = arena
            .get(func.name)
            .and_then(|name_node| arena.get_identifier(name_node))
            .map(|ident| ident.escaped_text.as_str());
        if name != Some(function_name) {
            continue;
        }
        for &param_idx in &func.parameters.nodes {
            let Some(param_node) = arena.get(param_idx) else { continue; };
            let Some(param) = arena.get_parameter(param_node) else { continue; };
            let param_text = arena
                .get(param.name)
                .and_then(|param_name_node| arena.get_identifier(param_name_node))
                .map(|ident| ident.escaped_text.as_str());
            if param_text != Some(param_name) {
                continue;
            }
            param_name_idx = param.name;
            param_symbol = binder.get_node_symbol(param.name);
            function_body = func.body;
            break;
        }
        break;
    }

    assert!(
        !param_name_idx.is_none(),
        "Expected to find parameter name for {function_name}"
    );
    assert!(
        param_symbol.is_some(),
        "Expected parameter symbol for {function_name}"
    );
    assert!(
        !function_body.is_none(),
        "Expected function body for {function_name}"
    );

    let mut usage_idx = NodeIndex::NONE;
    for i in 0..arena.len() {
        let idx = NodeIndex(i as u32);
        if idx == param_name_idx {
            continue;
        }
        let Some(node) = arena.get(idx) else { continue; };
        let Some(ident) = arena.get_identifier(node) else { continue; };
        if ident.escaped_text != param_name {
            continue;
        }
        let mut current = idx;
        let mut in_body = false;
        while !current.is_none() {
            if current == function_body {
                in_body = true;
                break;
            }
            let Some(ext) = arena.get_extended(current) else { break; };
            current = ext.parent;
        }
        if in_body {
            usage_idx = idx;
            break;
        }
    }

    assert!(
        !usage_idx.is_none(),
        "Expected a '{param_name}' identifier inside the function body"
    );

    let resolved = binder.resolve_identifier(arena, usage_idx);
    assert_eq!(
        resolved,
        param_symbol,
        "Expected body identifier to resolve to the parameter symbol"
    );
}

fn assert_bound_state_resolves_param(source: &str, function_name: &str, param_name: &str) {
    assert_bound_state_resolves_param_impl(source, function_name, param_name, true);
}

fn assert_bound_state_resolves_param_without_scopes(
    source: &str,
    function_name: &str,
    param_name: &str,
) {
    assert_bound_state_resolves_param_impl(source, function_name, param_name, false);
}

fn node_is_within(arena: &crate::parser::thin_node::ThinNodeArena, node_idx: crate::parser::NodeIndex, container: crate::parser::NodeIndex) -> bool {
    let mut current = node_idx;
    while !current.is_none() {
        if current == container {
            return true;
        }
        let Some(ext) = arena.get_extended(current) else {
            break;
        };
        current = ext.parent;
    }
    false
}

#[test]
fn test_thin_binder_resolves_parameter_from_bound_state() {
    let source = r#"
export function f(node: number) {
    return node;
}
"#;

    assert_bound_state_resolves_param(source, "f", "node");
}

#[test]
fn test_thin_binder_resolves_parameter_from_bound_state_module_instance_state() {
    let source = r#"
export function getModuleInstanceState(node: { body?: { parent?: {} } }) {
    if (node.body && !node.body.parent) {
        return node.body;
    }
    return node.body;
}
"#;

    assert_bound_state_resolves_param(source, "getModuleInstanceState", "node");
}

#[test]
fn test_thin_binder_resolves_parameter_from_bound_state_module_instance_state_with_visited() {
    let source = r#"
export function getModuleInstanceState(
    node: { body?: { parent?: {} } },
    visited?: Map<number, unknown>
) {
    if (node.body && !node.body.parent) {
        return node.body;
    }
    return node.body;
}
"#;

    assert_bound_state_resolves_param(source, "getModuleInstanceState", "node");
}

#[test]
fn test_thin_binder_resolves_parameter_from_bound_state_binder_ts_331() {
    let source = r#"
export function getModuleInstanceState(node: ModuleDeclaration, visited?: Map<number, ModuleInstanceState | undefined>): ModuleInstanceState {
    if (node.body && !node.body.parent) {
        setParent(node.body, node);
        setParentRecursive(node.body, /*incremental*/ false);
    }
    return node.body ? getModuleInstanceStateCached(node.body, visited) : ModuleInstanceState.Instantiated;
}
"#;

    assert_bound_state_resolves_param(source, "getModuleInstanceState", "node");
}

#[test]
fn test_thin_binder_resolves_parameter_from_bound_state_binder_ts_331_without_scopes() {
    let source = r#"
export function getModuleInstanceState(node: ModuleDeclaration, visited?: Map<number, ModuleInstanceState | undefined>): ModuleInstanceState {
    if (node.body && !node.body.parent) {
        setParent(node.body, node);
        setParentRecursive(node.body, /*incremental*/ false);
    }
    return node.body ? getModuleInstanceStateCached(node.body, visited) : ModuleInstanceState.Instantiated;
}
"#;

    assert_bound_state_resolves_param_without_scopes(source, "getModuleInstanceState", "node");
}

#[test]
fn test_thin_binder_resolves_block_local_from_bound_state_binder_ts_432() {
    use crate::binder::SymbolTable;
    use crate::parallel;
    use crate::parser::{syntax_kind_ext, NodeIndex};

    let source = r#"
export function getModuleInstanceStateForAliasTarget(
    specifier: ExportSpecifier,
    visited: Map<number, ModuleInstanceState | undefined>
) {
    const name = specifier.propertyName || specifier.name;
    let p: Node | undefined = specifier.parent;
    while (p) {
        if (isBlock(p) || isModuleBlock(p) || isSourceFile(p)) {
            const statements = p.statements;
            let found: ModuleInstanceState | undefined;
            for (const statement of statements) {
                if (nodeHasName(statement, name)) {
                    return found;
                }
            }
        }
        p = p.parent;
    }
    return ModuleInstanceState.Instantiated;
}
"#;

    let program = parallel::compile_files(vec![("test.ts".to_string(), source.to_string())]);
    let file = &program.files[0];

    let mut file_locals = SymbolTable::new();
    for (name, &sym_id) in program.file_locals[0].iter() {
        file_locals.set(name.clone(), sym_id);
    }
    for (name, &sym_id) in program.globals.iter() {
        if !file_locals.has(name) {
            file_locals.set(name.clone(), sym_id);
        }
    }

    let binder = ThinBinderState::from_bound_state_with_scopes(
        program.symbols.clone(),
        file_locals,
        file.node_symbols.clone(),
        file.scopes.clone(),
        file.node_scope_ids.clone(),
    );

    let arena = &file.arena;
    let mut function_body = NodeIndex::NONE;
    for i in 0..arena.len() {
        let idx = NodeIndex(i as u32);
        let Some(node) = arena.get(idx) else { continue; };
        if node.kind != syntax_kind_ext::FUNCTION_DECLARATION {
            continue;
        }
        let Some(func) = arena.get_function(node) else { continue; };
        let name = arena
            .get(func.name)
            .and_then(|name_node| arena.get_identifier(name_node))
            .map(|ident| ident.escaped_text.as_str());
        if name == Some("getModuleInstanceStateForAliasTarget") {
            function_body = func.body;
            break;
        }
    }

    assert!(
        !function_body.is_none(),
        "Expected function body for getModuleInstanceStateForAliasTarget"
    );

    let mut decl_name_idx = NodeIndex::NONE;
    let mut decl_symbol = None;
    for i in 0..arena.len() {
        let idx = NodeIndex(i as u32);
        let Some(node) = arena.get(idx) else { continue; };
        if node.kind != syntax_kind_ext::VARIABLE_DECLARATION {
            continue;
        }
        let Some(decl) = arena.get_variable_declaration(node) else { continue; };
        let name = arena
            .get(decl.name)
            .and_then(|name_node| arena.get_identifier(name_node))
            .map(|ident| ident.escaped_text.as_str());
        if name != Some("statements") {
            continue;
        }
        if !node_is_within(arena, idx, function_body) {
            continue;
        }
        decl_name_idx = decl.name;
        decl_symbol = binder.get_node_symbol(decl.name);
        break;
    }

    assert!(
        !decl_name_idx.is_none(),
        "Expected declaration for statements"
    );
    assert!(decl_symbol.is_some(), "Expected symbol for statements declaration");

    let mut usage_idx = NodeIndex::NONE;
    for i in 0..arena.len() {
        let idx = NodeIndex(i as u32);
        let Some(node) = arena.get(idx) else { continue; };
        if node.kind != syntax_kind_ext::FOR_OF_STATEMENT {
            continue;
        }
        if !node_is_within(arena, idx, function_body) {
            continue;
        }
        let Some(for_data) = arena.get_for_in_of(node) else { continue; };
        let Some(expr_node) = arena.get(for_data.expression) else { continue; };
        let Some(ident) = arena.get_identifier(expr_node) else { continue; };
        if ident.escaped_text == "statements" {
            usage_idx = for_data.expression;
            break;
        }
    }

    assert!(
        !usage_idx.is_none(),
        "Expected for-of expression to reference statements"
    );

    let resolved = binder.resolve_identifier(arena, usage_idx);
    assert_eq!(
        resolved,
        decl_symbol,
        "Expected for-of expression to resolve to statements declaration"
    );
}
#[test]
fn test_namespace_binding_debug() {
    use crate::thin_parser::ThinParserState;
    use crate::thin_binder::ThinBinderState;

    let source = r#"
namespace foo {
    export class Provide {}
    class NotExported {}
    export function bar() {}
    function baz() {}
}
"#;

    let mut parser = ThinParserState::new("test.ts".to_string(), source.to_string());
    let root = parser.parse_source_file();

    let arena = parser.get_arena();
    let mut binder = ThinBinderState::new();
    binder.bind_source_file(arena, root);

    // Check if 'foo' was bound
    let foo_sym_id = binder.file_locals.get("foo").expect("'foo' should be in file_locals");
    let foo_symbol = binder.get_symbol(foo_sym_id).expect("foo symbol should exist");

    // Check if exports were captured
    assert!(foo_symbol.exports.is_some(), "foo should have exports");

    // Check that ONLY exported members are in exports
    let exports = foo_symbol.exports.as_ref().unwrap();

    // Exported members should be present
    assert!(exports.get("Provide").is_some(), "Provide should be in foo's exports");
    assert!(exports.get("bar").is_some(), "bar should be in foo's exports");

    // Non-exported members should NOT be in exports
    assert!(exports.get("NotExported").is_none(), "NotExported should NOT be in foo's exports");
    assert!(exports.get("baz").is_none(), "baz should NOT be in foo's exports");

    // Should have exactly 2 exports
    assert_eq!(exports.len(), 2, "foo should have exactly 2 exports");
}

#[test]
fn test_import_alias_binding() {
    use crate::thin_parser::ThinParserState;
    use crate::thin_binder::ThinBinderState;
    use crate::binder::symbol_flags;

    let source = r#"
namespace NS {
    export class C {}
}
import Alias = NS.C;
var x: Alias;
"#;

    let mut parser = ThinParserState::new("test.ts".to_string(), source.to_string());
    let root = parser.parse_source_file();

    let arena = parser.get_arena();
    let mut binder = ThinBinderState::new();
    binder.bind_source_file(arena, root);

    // Check that 'Alias' was bound as an ALIAS symbol
    let alias_sym_id = binder.file_locals.get("Alias").expect("'Alias' should be in file_locals");
    let alias_symbol = binder.get_symbol(alias_sym_id).expect("Alias symbol should exist");

    // Verify it has the ALIAS flag
    assert_eq!(alias_symbol.flags & symbol_flags::ALIAS, symbol_flags::ALIAS, "Alias should have ALIAS flag");

    // Verify it has a declaration
    assert!(!alias_symbol.declarations.is_empty(), "Alias should have declarations");
}

#[test]
fn test_namespace_exports_merge_across_decls() {
    use crate::thin_parser::ThinParserState;
    use crate::thin_binder::ThinBinderState;

    let source = r#"
namespace Merge {
    export const a = 1;
    const hidden = 2;
}
namespace Merge {
    export function foo() {}
    export const b = 3;
}
"#;

    let mut parser = ThinParserState::new("test.ts".to_string(), source.to_string());
    let root = parser.parse_source_file();

    let arena = parser.get_arena();
    let mut binder = ThinBinderState::new();
    binder.bind_source_file(arena, root);

    let merge_sym_id = binder.file_locals.get("Merge").expect("'Merge' should be in file_locals");
    let merge_symbol = binder.get_symbol(merge_sym_id).expect("Merge symbol should exist");

    let exports = merge_symbol.exports.as_ref().expect("Merge should have exports");
    assert!(exports.get("a").is_some(), "a should be in Merge exports");
    assert!(exports.get("b").is_some(), "b should be in Merge exports");
    assert!(exports.get("foo").is_some(), "foo should be in Merge exports");
    assert!(exports.get("hidden").is_none(), "hidden should not be in Merge exports");
    assert_eq!(exports.len(), 3, "Merge should have exactly 3 exports");
}

#[test]
fn test_class_namespace_merge_exports() {
    use crate::thin_parser::ThinParserState;
    use crate::thin_binder::ThinBinderState;

    let source = r#"
class Merge {
    constructor() {}
}
namespace Merge {
    export const extra = 1;
    const hidden = 2;
}
"#;

    let mut parser = ThinParserState::new("test.ts".to_string(), source.to_string());
    let root = parser.parse_source_file();

    let arena = parser.get_arena();
    let mut binder = ThinBinderState::new();
    binder.bind_source_file(arena, root);

    let merge_sym_id = binder.file_locals.get("Merge").expect("'Merge' should be in file_locals");
    let merge_symbol = binder.get_symbol(merge_sym_id).expect("Merge symbol should exist");

    let exports = merge_symbol.exports.as_ref().expect("Merge should have exports");
    assert!(exports.get("extra").is_some(), "extra should be in Merge exports");
    assert!(exports.get("hidden").is_none(), "hidden should not be in Merge exports");
    assert_eq!(exports.len(), 1, "Merge should have exactly 1 export");
}

#[test]
fn test_thin_binder_deep_binary_expression() {
    const COUNT: usize = 50000;
    let mut source = String::with_capacity(COUNT * 4);
    for i in 0..COUNT {
        if i > 0 {
            source.push_str(" + ");
        }
        source.push('0');
    }
    source.push(';');

    let mut parser = ThinParserState::new("test.ts".to_string(), source);
    let root = parser.parse_source_file();

    let mut binder = ThinBinderState::new();
    binder.bind_source_file(parser.get_arena(), root);

    assert!(binder.file_locals.is_empty());
}
