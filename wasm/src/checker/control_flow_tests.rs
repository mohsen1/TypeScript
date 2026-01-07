use super::FlowAnalyzer;
use crate::solver::{PropertyInfo, TypeInterner};
use crate::thin_binder::ThinBinderState;
use crate::thin_parser::ThinParserState;
use crate::parser::thin_node::ThinNodeArena;
use crate::parser::NodeIndex;

fn get_switch_statement(arena: &ThinNodeArena, root: NodeIndex, stmt_index: usize) -> NodeIndex {
    let root_node = arena.get(root).expect("root node");
    let source_file = arena.get_source_file(root_node).expect("source file");
    *source_file
        .statements
        .nodes
        .get(stmt_index)
        .expect("switch statement")
}

fn get_switch_clause_expression(
    arena: &ThinNodeArena,
    switch_idx: NodeIndex,
    clause_index: usize,
) -> NodeIndex {
    let switch_node = arena.get(switch_idx).expect("switch node");
    let switch_data = arena.get_switch(switch_node).expect("switch data");
    let case_block_node = arena.get(switch_data.case_block).expect("case block node");
    let case_block = arena.get_block(case_block_node).expect("case block");
    let clause_idx = *case_block
        .statements
        .nodes
        .get(clause_index)
        .expect("case clause");
    let clause_node = arena.get(clause_idx).expect("clause node");
    let clause = arena.get_case_clause(clause_node).expect("clause data");
    let stmt_idx = *clause
        .statements
        .nodes
        .first()
        .expect("clause statement");
    let stmt_node = arena.get(stmt_idx).expect("statement node");
    let expr_stmt = arena
        .get_expression_statement(stmt_node)
        .expect("expression statement");
    expr_stmt.expression
}

#[test]
fn test_switch_fallthrough_and_default_narrowing() {
    let source = r#"
let x: "a" | "b" | "c";
switch (x) {
  case "a":
    x;
  case "b":
    x;
    break;
  default:
    x;
}
"#;

    let mut parser = ThinParserState::new("test.ts".to_string(), source.to_string());
    let root = parser.parse_source_file();

    let mut binder = ThinBinderState::new();
    binder.bind_source_file(parser.get_arena(), root);

    let arena = parser.get_arena();
    let types = TypeInterner::new();
    let analyzer = FlowAnalyzer::new(arena, &binder, &types);

    let switch_idx = get_switch_statement(arena, root, 1);
    let ident_case_a = get_switch_clause_expression(arena, switch_idx, 0);
    let ident_case_b = get_switch_clause_expression(arena, switch_idx, 1);
    let ident_default = get_switch_clause_expression(arena, switch_idx, 2);

    let lit_a = types.literal_string("a");
    let lit_b = types.literal_string("b");
    let lit_c = types.literal_string("c");
    let union = types.union(vec![lit_a, lit_b, lit_c]);

    let flow_a = binder.get_node_flow(ident_case_a).expect("flow for case a");
    let narrowed_a = analyzer.get_flow_type(ident_case_a, union, flow_a);
    assert_eq!(narrowed_a, lit_a);

    let flow_b = binder.get_node_flow(ident_case_b).expect("flow for case b");
    let narrowed_b = analyzer.get_flow_type(ident_case_b, union, flow_b);
    let expected_b = types.union(vec![lit_a, lit_b]);
    assert_eq!(narrowed_b, expected_b);

    let flow_default = binder.get_node_flow(ident_default).expect("flow for default");
    let narrowed_default = analyzer.get_flow_type(ident_default, union, flow_default);
    assert_eq!(narrowed_default, lit_c);
}

#[test]
fn test_switch_discriminant_narrowing() {
    let source = r#"
let x: { kind: "a" } | { kind: "b" };
switch (x.kind) {
  case "a":
    x;
    break;
  default:
    x;
}
"#;

    let mut parser = ThinParserState::new("test.ts".to_string(), source.to_string());
    let root = parser.parse_source_file();

    let mut binder = ThinBinderState::new();
    binder.bind_source_file(parser.get_arena(), root);

    let arena = parser.get_arena();
    let types = TypeInterner::new();
    let analyzer = FlowAnalyzer::new(arena, &binder, &types);

    let kind_name = types.intern_string("kind");
    let lit_a = types.literal_string("a");
    let lit_b = types.literal_string("b");

    let member_a = types.object(vec![PropertyInfo {
        name: kind_name,
        type_id: lit_a,
        optional: false,
        readonly: false,
        is_method: false,
    }]);
    let member_b = types.object(vec![PropertyInfo {
        name: kind_name,
        type_id: lit_b,
        optional: false,
        readonly: false,
        is_method: false,
    }]);

    let union = types.union(vec![member_a, member_b]);

    let switch_idx = get_switch_statement(arena, root, 1);
    let ident_case_a = get_switch_clause_expression(arena, switch_idx, 0);
    let ident_default = get_switch_clause_expression(arena, switch_idx, 1);

    let flow_case_a = binder.get_node_flow(ident_case_a).expect("flow for case a");
    let narrowed_case_a = analyzer.get_flow_type(ident_case_a, union, flow_case_a);
    assert_eq!(narrowed_case_a, member_a);

    let flow_default = binder.get_node_flow(ident_default).expect("flow for default");
    let narrowed_default = analyzer.get_flow_type(ident_default, union, flow_default);
    assert_eq!(narrowed_default, member_b);
}
