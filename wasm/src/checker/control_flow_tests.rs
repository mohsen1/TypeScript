use super::FlowAnalyzer;
use crate::solver::{PropertyInfo, TypeId, TypeInterner};
use crate::thin_binder::ThinBinderState;
use crate::thin_checker::ThinCheckerState;
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

fn get_if_branch_expression(
    arena: &ThinNodeArena,
    root: NodeIndex,
    stmt_index: usize,
    is_then: bool,
) -> NodeIndex {
    let root_node = arena.get(root).expect("root node");
    let source_file = arena.get_source_file(root_node).expect("source file");
    let if_idx = *source_file
        .statements
        .nodes
        .get(stmt_index)
        .expect("if statement");
    let if_node = arena.get(if_idx).expect("if node");
    let if_data = arena.get_if_statement(if_node).expect("if data");
    let branch_idx = if is_then {
        if_data.then_statement
    } else {
        if_data.else_statement
    };
    assert!(!branch_idx.is_none(), "missing branch statement");
    extract_expression_from_statement(arena, branch_idx)
}

fn extract_expression_from_statement(arena: &ThinNodeArena, stmt_idx: NodeIndex) -> NodeIndex {
    let stmt_node = arena.get(stmt_idx).expect("statement node");
    if let Some(block) = arena.get_block(stmt_node) {
        let inner_idx = *block
            .statements
            .nodes
            .first()
            .expect("block statement");
        return extract_expression_from_statement(arena, inner_idx);
    }

    let expr_stmt = arena
        .get_expression_statement(stmt_node)
        .expect("expression statement");
    expr_stmt.expression
}

fn get_block_expression(
    arena: &ThinNodeArena,
    block_idx: NodeIndex,
    stmt_index: usize,
) -> NodeIndex {
    let block_node = arena.get(block_idx).expect("block node");
    let block = arena.get_block(block_node).expect("block");
    let stmt_idx = *block
        .statements
        .nodes
        .get(stmt_index)
        .expect("block statement");
    extract_expression_from_statement(arena, stmt_idx)
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

#[test]
fn test_instanceof_narrows_to_object_union_members() {
    let source = r#"
let x: string | { a: number };
if (x instanceof Foo) {
  x;
} else {
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

    let prop_a = types.intern_string("a");
    let obj_type = types.object(vec![PropertyInfo {
        name: prop_a,
        type_id: TypeId::NUMBER,
        optional: false,
        readonly: false,
        is_method: false,
    }]);
    let union = types.union(vec![TypeId::STRING, obj_type]);

    let ident_then = get_if_branch_expression(arena, root, 1, true);
    let ident_else = get_if_branch_expression(arena, root, 1, false);

    let flow_then = binder.get_node_flow(ident_then).expect("flow then");
    let flow_else = binder.get_node_flow(ident_else).expect("flow else");

    let narrowed_then = analyzer.get_flow_type(ident_then, union, flow_then);
    assert_eq!(narrowed_then, obj_type);

    let narrowed_else = analyzer.get_flow_type(ident_else, union, flow_else);
    assert_eq!(narrowed_else, union);
}

#[test]
fn test_in_operator_narrows_required_property() {
    let source = r#"
let x: { a: number } | { b: string };
if ("a" in x) {
  x;
} else {
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

    let prop_a = types.intern_string("a");
    let prop_b = types.intern_string("b");

    let type_a = types.object(vec![PropertyInfo {
        name: prop_a,
        type_id: TypeId::NUMBER,
        optional: false,
        readonly: false,
        is_method: false,
    }]);
    let type_b = types.object(vec![PropertyInfo {
        name: prop_b,
        type_id: TypeId::STRING,
        optional: false,
        readonly: false,
        is_method: false,
    }]);
    let union = types.union(vec![type_a, type_b]);

    let ident_then = get_if_branch_expression(arena, root, 1, true);
    let ident_else = get_if_branch_expression(arena, root, 1, false);

    let flow_then = binder.get_node_flow(ident_then).expect("flow then");
    let flow_else = binder.get_node_flow(ident_else).expect("flow else");

    let narrowed_then = analyzer.get_flow_type(ident_then, union, flow_then);
    assert_eq!(narrowed_then, type_a);

    let narrowed_else = analyzer.get_flow_type(ident_else, union, flow_else);
    assert_eq!(narrowed_else, type_b);
}

#[test]
fn test_in_operator_optional_property_keeps_false_branch_union() {
    let source = r#"
let x: { a?: number } | { b: string };
if ("a" in x) {
  x;
} else {
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

    let prop_a = types.intern_string("a");
    let prop_b = types.intern_string("b");

    let type_a = types.object(vec![PropertyInfo {
        name: prop_a,
        type_id: TypeId::NUMBER,
        optional: true,
        readonly: false,
        is_method: false,
    }]);
    let type_b = types.object(vec![PropertyInfo {
        name: prop_b,
        type_id: TypeId::STRING,
        optional: false,
        readonly: false,
        is_method: false,
    }]);
    let union = types.union(vec![type_a, type_b]);

    let ident_then = get_if_branch_expression(arena, root, 1, true);
    let ident_else = get_if_branch_expression(arena, root, 1, false);

    let flow_then = binder.get_node_flow(ident_then).expect("flow then");
    let flow_else = binder.get_node_flow(ident_else).expect("flow else");

    let narrowed_then = analyzer.get_flow_type(ident_then, union, flow_then);
    assert_eq!(narrowed_then, type_a);

    let narrowed_else = analyzer.get_flow_type(ident_else, union, flow_else);
    assert_eq!(narrowed_else, union);
}

#[test]
fn test_user_defined_type_predicate_narrows_branches() {
    let source = r#"
function isString(x: string | number): x is string {
  return typeof x === "string";
}
let x: string | number;
if (isString(x)) {
  x;
} else {
  x;
}
"#;

    let mut parser = ThinParserState::new("test.ts".to_string(), source.to_string());
    let root = parser.parse_source_file();

    let mut binder = ThinBinderState::new();
    binder.bind_source_file(parser.get_arena(), root);

    let arena = parser.get_arena();
    let types = TypeInterner::new();
    let mut checker = ThinCheckerState::new(arena, &binder, &types, "test.ts".to_string());
    checker.check_source_file(root);

    let analyzer = FlowAnalyzer::with_node_types(arena, &binder, &types, &checker.ctx.node_types);

    let ident_then = get_if_branch_expression(arena, root, 2, true);
    let ident_else = get_if_branch_expression(arena, root, 2, false);

    let union = types.union(vec![TypeId::STRING, TypeId::NUMBER]);

    let flow_then = binder.get_node_flow(ident_then).expect("flow then");
    let flow_else = binder.get_node_flow(ident_else).expect("flow else");

    let narrowed_then = analyzer.get_flow_type(ident_then, union, flow_then);
    assert_eq!(narrowed_then, TypeId::STRING);

    let narrowed_else = analyzer.get_flow_type(ident_else, union, flow_else);
    assert_eq!(narrowed_else, TypeId::NUMBER);
}

#[test]
fn test_user_defined_type_predicate_alias_narrows() {
    let source = r#"
function isString(x: string | number): x is string {
  return typeof x === "string";
}
const guard = isString;
let x: string | number;
if (guard(x)) {
  x;
} else {
  x;
}
"#;

    let mut parser = ThinParserState::new("test.ts".to_string(), source.to_string());
    let root = parser.parse_source_file();

    let mut binder = ThinBinderState::new();
    binder.bind_source_file(parser.get_arena(), root);

    let arena = parser.get_arena();
    let types = TypeInterner::new();
    let mut checker = ThinCheckerState::new(arena, &binder, &types, "test.ts".to_string());
    checker.check_source_file(root);

    let analyzer = FlowAnalyzer::with_node_types(arena, &binder, &types, &checker.ctx.node_types);

    let ident_then = get_if_branch_expression(arena, root, 3, true);
    let ident_else = get_if_branch_expression(arena, root, 3, false);

    let union = types.union(vec![TypeId::STRING, TypeId::NUMBER]);

    let flow_then = binder.get_node_flow(ident_then).expect("flow then");
    let flow_else = binder.get_node_flow(ident_else).expect("flow else");

    let narrowed_then = analyzer.get_flow_type(ident_then, union, flow_then);
    assert_eq!(narrowed_then, TypeId::STRING);

    let narrowed_else = analyzer.get_flow_type(ident_else, union, flow_else);
    assert_eq!(narrowed_else, TypeId::NUMBER);
}

#[test]
fn test_asserts_type_predicate_narrows_true_branch() {
    let source = r#"
function assertString(x: string | number): asserts x is string {
  if (typeof x !== "string") throw new Error("nope");
}
let x: string | number;
if (assertString(x)) {
  x;
} else {
  x;
}
"#;

    let mut parser = ThinParserState::new("test.ts".to_string(), source.to_string());
    let root = parser.parse_source_file();

    let mut binder = ThinBinderState::new();
    binder.bind_source_file(parser.get_arena(), root);

    let arena = parser.get_arena();
    let types = TypeInterner::new();
    let mut checker = ThinCheckerState::new(arena, &binder, &types, "test.ts".to_string());
    checker.check_source_file(root);

    let analyzer = FlowAnalyzer::with_node_types(arena, &binder, &types, &checker.ctx.node_types);

    let ident_then = get_if_branch_expression(arena, root, 2, true);
    let ident_else = get_if_branch_expression(arena, root, 2, false);

    let union = types.union(vec![TypeId::STRING, TypeId::NUMBER]);

    let flow_then = binder.get_node_flow(ident_then).expect("flow then");
    let flow_else = binder.get_node_flow(ident_else).expect("flow else");

    let narrowed_then = analyzer.get_flow_type(ident_then, union, flow_then);
    assert_eq!(narrowed_then, TypeId::STRING);

    let narrowed_else = analyzer.get_flow_type(ident_else, union, flow_else);
    assert_eq!(narrowed_else, union);
}

#[test]
fn test_assignment_clears_narrowing_in_branch() {
    let source = r#"
let x: string | number;
if (typeof x === "string") {
  x;
  x = 1;
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

    let root_node = arena.get(root).expect("root node");
    let source_file = arena.get_source_file(root_node).expect("source file");
    let if_idx = *source_file.statements.nodes.get(1).expect("if statement");
    let if_node = arena.get(if_idx).expect("if node");
    let if_data = arena.get_if_statement(if_node).expect("if data");
    let then_block = if_data.then_statement;

    let ident_before = get_block_expression(arena, then_block, 0);
    let ident_after = get_block_expression(arena, then_block, 2);

    let union = types.union(vec![TypeId::STRING, TypeId::NUMBER]);

    let flow_before = binder.get_node_flow(ident_before).expect("flow before");
    let narrowed_before = analyzer.get_flow_type(ident_before, union, flow_before);
    assert_eq!(narrowed_before, TypeId::STRING);

    let flow_after = binder.get_node_flow(ident_after).expect("flow after");
    let narrowed_after = analyzer.get_flow_type(ident_after, union, flow_after);
    assert_eq!(narrowed_after, union);
}

#[test]
fn test_array_mutation_clears_predicate_narrowing() {
    let source = r#"
function isStringArray(x: string[] | number[]): x is string[] {
  return true;
}
let x: string[] | number[];
if (isStringArray(x)) {
  x;
  x.push("a");
  x;
}
"#;

    let mut parser = ThinParserState::new("test.ts".to_string(), source.to_string());
    let root = parser.parse_source_file();

    let mut binder = ThinBinderState::new();
    binder.bind_source_file(parser.get_arena(), root);

    let arena = parser.get_arena();
    let types = TypeInterner::new();
    let mut checker = ThinCheckerState::new(arena, &binder, &types, "test.ts".to_string());
    checker.check_source_file(root);

    let analyzer = FlowAnalyzer::with_node_types(arena, &binder, &types, &checker.ctx.node_types);

    let root_node = arena.get(root).expect("root node");
    let source_file = arena.get_source_file(root_node).expect("source file");
    let if_idx = *source_file.statements.nodes.get(2).expect("if statement");
    let if_node = arena.get(if_idx).expect("if node");
    let if_data = arena.get_if_statement(if_node).expect("if data");
    let then_block = if_data.then_statement;

    let ident_before = get_block_expression(arena, then_block, 0);
    let ident_after = get_block_expression(arena, then_block, 2);

    let string_array = types.array(TypeId::STRING);
    let number_array = types.array(TypeId::NUMBER);
    let union = types.union(vec![string_array, number_array]);

    let flow_before = binder.get_node_flow(ident_before).expect("flow before");
    let narrowed_before = analyzer.get_flow_type(ident_before, union, flow_before);
    assert_eq!(narrowed_before, string_array);

    let flow_after = binder.get_node_flow(ident_after).expect("flow after");
    let narrowed_after = analyzer.get_flow_type(ident_after, union, flow_after);
    assert_eq!(narrowed_after, union);
}
