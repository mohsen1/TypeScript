//! Tests for Binder

use crate::binder::*;

#[test]
fn test_symbol_flags() {
    assert_eq!(symbol_flags::NONE, 0);
    assert_eq!(symbol_flags::FUNCTION_SCOPED_VARIABLE, 1);
    assert_eq!(symbol_flags::BLOCK_SCOPED_VARIABLE, 2);
    assert_eq!(symbol_flags::VARIABLE, 3);
}

#[test]
fn test_symbol_id() {
    let id = SymbolId(42);
    assert_eq!(id.0, 42);
    assert!(!id.is_none());
    assert!(SymbolId::NONE.is_none());
}

#[test]
fn test_flow_flags() {
    assert_eq!(flow_flags::UNREACHABLE, 1);
    assert_eq!(flow_flags::START, 2);
    assert_eq!(flow_flags::LABEL, flow_flags::BRANCH_LABEL | flow_flags::LOOP_LABEL);
    assert_eq!(flow_flags::CONDITION, flow_flags::TRUE_CONDITION | flow_flags::FALSE_CONDITION);
}

#[test]
fn test_flow_node_id() {
    let id = FlowNodeId(42);
    assert_eq!(id.0, 42);
    assert!(!id.is_none());
    assert!(FlowNodeId::NONE.is_none());
}

#[test]
fn test_flow_node() {
    let node = FlowNode::new(FlowNodeId(0), flow_flags::START);
    assert!(node.has_flags(flow_flags::START));
    assert!(!node.has_flags(flow_flags::UNREACHABLE));
    assert!(node.antecedent.is_empty());
}

#[test]
fn test_flow_node_arena() {
    let mut arena = FlowNodeArena::new();
    assert!(arena.is_empty());

    let start = arena.alloc(flow_flags::START);
    let branch = arena.alloc(flow_flags::BRANCH_LABEL);

    assert_eq!(arena.len(), 2);
    assert_eq!(start.0, 0);
    assert_eq!(branch.0, 1);

    let start_node = arena.get(start).unwrap();
    assert!(start_node.has_flags(flow_flags::START));

    let branch_node = arena.get(branch).unwrap();
    assert!(branch_node.has_flags(flow_flags::BRANCH_LABEL));
}

#[test]
fn test_symbol() {
    let sym = Symbol::new(
        SymbolId(0),
        symbol_flags::FUNCTION,
        "myFunc".to_string(),
    );
    assert!(sym.has_flags(symbol_flags::FUNCTION));
    assert!(!sym.has_flags(symbol_flags::CLASS));
    assert_eq!(sym.escaped_name, "myFunc");
}

#[test]
fn test_symbol_table() {
    let mut table = SymbolTable::new();
    assert!(table.is_empty());

    table.set("x".to_string(), SymbolId(0));
    table.set("y".to_string(), SymbolId(1));

    assert_eq!(table.len(), 2);
    assert!(table.has("x"));
    assert!(!table.has("z"));
    assert_eq!(table.get("x"), Some(SymbolId(0)));
    assert_eq!(table.get("z"), None);
}

#[test]
fn test_symbol_arena() {
    let mut arena = SymbolArena::new();
    assert!(arena.is_empty());

    let id1 = arena.alloc(symbol_flags::VARIABLE, "x".to_string());
    let id2 = arena.alloc(symbol_flags::FUNCTION, "f".to_string());

    assert_eq!(arena.len(), 2);
    assert_eq!(id1.0, 0);
    assert_eq!(id2.0, 1);

    let sym1 = arena.get(id1).unwrap();
    assert_eq!(sym1.escaped_name, "x");
    assert!(sym1.has_flags(symbol_flags::VARIABLE));

    let sym2 = arena.get(id2).unwrap();
    assert_eq!(sym2.escaped_name, "f");
    assert!(sym2.has_flags(symbol_flags::FUNCTION));
}

#[test]
fn test_bind_variable_declaration() {
    use crate::parser_impl::ParserState;

    let mut parser = ParserState::new(
        "test.ts".to_string(),
        "const x = 42;".to_string(),
    );
    let root = parser.parse_source_file();

    let mut binder = BinderState::new();
    binder.bind_source_file(&parser.arena, root);

    // Should have one symbol 'x'
    assert_eq!(binder.file_locals.len(), 1);
    assert!(binder.file_locals.has("x"));

    let x_id = binder.file_locals.get("x").unwrap();
    let x_sym = binder.symbols.get(x_id).unwrap();
    assert_eq!(x_sym.escaped_name, "x");
    assert!(x_sym.has_flags(symbol_flags::BLOCK_SCOPED_VARIABLE));
}

#[test]
fn test_bind_function_declaration() {
    use crate::parser_impl::ParserState;

    let mut parser = ParserState::new(
        "test.ts".to_string(),
        "function add(a: number, b: number): number { return a + b; }".to_string(),
    );
    let root = parser.parse_source_file();

    let mut binder = BinderState::new();
    binder.bind_source_file(&parser.arena, root);

    // Should have one symbol 'add'
    assert!(binder.file_locals.has("add"));

    let add_id = binder.file_locals.get("add").unwrap();
    let add_sym = binder.symbols.get(add_id).unwrap();
    assert_eq!(add_sym.escaped_name, "add");
    assert!(add_sym.has_flags(symbol_flags::FUNCTION));
}

#[test]
fn test_bind_class_declaration() {
    use crate::parser_impl::ParserState;

    let mut parser = ParserState::new(
        "test.ts".to_string(),
        "class Foo { x: number; bar() {} }".to_string(),
    );
    let root = parser.parse_source_file();

    let mut binder = BinderState::new();
    binder.bind_source_file(&parser.arena, root);

    // Should have one symbol 'Foo'
    assert!(binder.file_locals.has("Foo"));

    let foo_id = binder.file_locals.get("Foo").unwrap();
    let foo_sym = binder.symbols.get(foo_id).unwrap();
    assert_eq!(foo_sym.escaped_name, "Foo");
    assert!(foo_sym.has_flags(symbol_flags::CLASS));
}

#[test]
fn test_bind_multiple_declarations() {
    use crate::parser_impl::ParserState;

    let mut parser = ParserState::new(
        "test.ts".to_string(),
        r#"
            const x = 1;
            function foo() {}
            class Bar {}
            interface IBaz {}
            type MyType = string;
        "#.to_string(),
    );
    let root = parser.parse_source_file();

    let mut binder = BinderState::new();
    binder.bind_source_file(&parser.arena, root);

    // Should have 5 symbols
    assert!(binder.file_locals.has("x"));
    assert!(binder.file_locals.has("foo"));
    assert!(binder.file_locals.has("Bar"));
    assert!(binder.file_locals.has("IBaz"));
    assert!(binder.file_locals.has("MyType"));

    // Verify symbol flags
    let x_id = binder.file_locals.get("x").unwrap();
    assert!(binder.symbols.get(x_id).unwrap().has_flags(symbol_flags::BLOCK_SCOPED_VARIABLE));

    let foo_id = binder.file_locals.get("foo").unwrap();
    assert!(binder.symbols.get(foo_id).unwrap().has_flags(symbol_flags::FUNCTION));

    let bar_id = binder.file_locals.get("Bar").unwrap();
    assert!(binder.symbols.get(bar_id).unwrap().has_flags(symbol_flags::CLASS));

    let ibaz_id = binder.file_locals.get("IBaz").unwrap();
    assert!(binder.symbols.get(ibaz_id).unwrap().has_flags(symbol_flags::INTERFACE));

    let mytype_id = binder.file_locals.get("MyType").unwrap();
    assert!(binder.symbols.get(mytype_id).unwrap().has_flags(symbol_flags::TYPE_ALIAS));
}

#[test]
fn test_bind_namespace_declaration() {
    use crate::parser_impl::ParserState;

    let mut parser = ParserState::new(
        "test.ts".to_string(),
        r#"
            namespace MyNamespace {
                export const x = 1;
                export function foo() {}
            }
        "#.to_string(),
    );
    let root = parser.parse_source_file();

    let mut binder = BinderState::new();
    binder.bind_source_file(&parser.arena, root);

    // Should have namespace symbol 'MyNamespace'
    assert!(binder.file_locals.has("MyNamespace"));

    let ns_id = binder.file_locals.get("MyNamespace").unwrap();
    let ns_sym = binder.symbols.get(ns_id).unwrap();
    assert_eq!(ns_sym.escaped_name, "MyNamespace");
    assert!(ns_sym.has_any_flags(symbol_flags::MODULE));
}

#[test]
fn test_interface_merging() {
    use crate::parser_impl::ParserState;

    let mut parser = ParserState::new(
        "test.ts".to_string(),
        r#"
            interface Foo {
                x: number;
            }
            interface Foo {
                y: string;
            }
        "#.to_string(),
    );
    let root = parser.parse_source_file();

    let mut binder = BinderState::new();
    binder.bind_source_file(&parser.arena, root);

    // Should have one symbol 'Foo' with two declarations
    assert!(binder.file_locals.has("Foo"));

    let foo_id = binder.file_locals.get("Foo").unwrap();
    let foo_sym = binder.symbols.get(foo_id).unwrap();
    assert_eq!(foo_sym.escaped_name, "Foo");
    assert!(foo_sym.has_flags(symbol_flags::INTERFACE));
    assert_eq!(foo_sym.declarations.len(), 2);
}

#[test]
fn test_namespace_merging() {
    use crate::parser_impl::ParserState;

    let mut parser = ParserState::new(
        "test.ts".to_string(),
        r#"
            namespace NS {
                export const a = 1;
            }
            namespace NS {
                export const b = 2;
            }
        "#.to_string(),
    );
    let root = parser.parse_source_file();

    let mut binder = BinderState::new();
    binder.bind_source_file(&parser.arena, root);

    // Should have one symbol 'NS' with two declarations
    assert!(binder.file_locals.has("NS"));

    let ns_id = binder.file_locals.get("NS").unwrap();
    let ns_sym = binder.symbols.get(ns_id).unwrap();
    assert_eq!(ns_sym.escaped_name, "NS");
    assert!(ns_sym.has_any_flags(symbol_flags::MODULE));
    assert_eq!(ns_sym.declarations.len(), 2);
}

#[test]
fn test_class_namespace_merging() {
    use crate::parser_impl::ParserState;

    let mut parser = ParserState::new(
        "test.ts".to_string(),
        r#"
            class Foo {
                x: number;
            }
            namespace Foo {
                export const bar = 1;
            }
        "#.to_string(),
    );
    let root = parser.parse_source_file();

    let mut binder = BinderState::new();
    binder.bind_source_file(&parser.arena, root);

    // Should have one symbol 'Foo' with both CLASS and MODULE flags
    assert!(binder.file_locals.has("Foo"));

    let foo_id = binder.file_locals.get("Foo").unwrap();
    let foo_sym = binder.symbols.get(foo_id).unwrap();
    assert_eq!(foo_sym.escaped_name, "Foo");
    assert!(foo_sym.has_flags(symbol_flags::CLASS));
    assert!(foo_sym.has_any_flags(symbol_flags::MODULE));
    assert_eq!(foo_sym.declarations.len(), 2);
}

#[test]
fn test_flow_nodes_basic() {
    use crate::parser_impl::ParserState;

    let mut parser = ParserState::new(
        "test.ts".to_string(),
        "const x = 1;".to_string(),
    );
    let root = parser.parse_source_file();

    let mut binder = BinderState::new();
    binder.bind_source_file(&parser.arena, root);

    // Should have at least 2 flow nodes: unreachable + start
    assert!(binder.flow_nodes.len() >= 2);

    // Flow node 0 should be UNREACHABLE
    let unreachable = binder.flow_nodes.get(FlowNodeId(0)).unwrap();
    assert!(unreachable.has_flags(flow_flags::UNREACHABLE));

    // Flow node 1 should be START
    let start = binder.flow_nodes.get(FlowNodeId(1)).unwrap();
    assert!(start.has_flags(flow_flags::START));
}

#[test]
fn test_flow_nodes_if_statement() {
    use crate::parser_impl::ParserState;

    let mut parser = ParserState::new(
        "test.ts".to_string(),
        r#"
            let x: number | string;
            if (typeof x === "number") {
                const y = x;
            } else {
                const z = x;
            }
        "#.to_string(),
    );
    let root = parser.parse_source_file();

    let mut binder = BinderState::new();
    binder.bind_source_file(&parser.arena, root);

    // Should have multiple flow nodes for the if statement:
    // - UNREACHABLE (0)
    // - START (1)
    // - TRUE_CONDITION (2) - for the if true branch
    // - BRANCH_LABEL (3) - merge point
    // - FALSE_CONDITION (4) - for the else branch
    assert!(binder.flow_nodes.len() >= 5);

    // Check that we have TRUE_CONDITION and FALSE_CONDITION nodes
    let mut has_true = false;
    let mut has_false = false;
    let mut has_branch_label = false;

    for i in 0..binder.flow_nodes.len() {
        if let Some(flow) = binder.flow_nodes.get(FlowNodeId(i as u32)) {
            if flow.has_flags(flow_flags::TRUE_CONDITION) {
                has_true = true;
            }
            if flow.has_flags(flow_flags::FALSE_CONDITION) {
                has_false = true;
            }
            if flow.has_flags(flow_flags::BRANCH_LABEL) {
                has_branch_label = true;
            }
        }
    }

    assert!(has_true, "Should have TRUE_CONDITION flow node");
    assert!(has_false, "Should have FALSE_CONDITION flow node");
    assert!(has_branch_label, "Should have BRANCH_LABEL flow node");
}

#[test]
fn test_flow_nodes_while_statement() {
    use crate::parser_impl::ParserState;

    let mut parser = ParserState::new(
        "test.ts".to_string(),
        r#"
            let i = 0;
            while (i < 10) {
                i = i + 1;
            }
        "#.to_string(),
    );
    let root = parser.parse_source_file();

    let mut binder = BinderState::new();
    binder.bind_source_file(&parser.arena, root);

    // Should have flow nodes for the while loop:
    // - UNREACHABLE
    // - START
    // - LOOP_LABEL
    // - TRUE_CONDITION
    // - FALSE_CONDITION

    let mut has_loop_label = false;
    let mut has_true = false;
    let mut has_false = false;

    for i in 0..binder.flow_nodes.len() {
        if let Some(flow) = binder.flow_nodes.get(FlowNodeId(i as u32)) {
            if flow.has_flags(flow_flags::LOOP_LABEL) {
                has_loop_label = true;
            }
            if flow.has_flags(flow_flags::TRUE_CONDITION) {
                has_true = true;
            }
            if flow.has_flags(flow_flags::FALSE_CONDITION) {
                has_false = true;
            }
        }
    }

    assert!(has_loop_label, "Should have LOOP_LABEL flow node");
    assert!(has_true, "Should have TRUE_CONDITION flow node");
    assert!(has_false, "Should have FALSE_CONDITION flow node");
}
