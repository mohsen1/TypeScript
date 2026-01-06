use super::*;
use crate::thin_parser::ThinParserState;
use crate::parser::syntax_kind_ext;

#[test]
fn test_intrinsic_type_ids() {
    assert_eq!(TypeId::ANY.0, 4);
    assert_eq!(TypeId::STRING.0, 10);
    assert_eq!(TypeId::NUMBER.0, 9);
}

#[test]
fn test_lowering_new() {
    let arena = ThinNodeArena::new();
    let interner = TypeInterner::new();
    let _lowering = TypeLowering::new(&arena, &interner);
}

// =============================================================================
// Type Parameter Lowering Tests
// =============================================================================

/// Helper to parse a type alias and return the type node index
fn parse_type_alias(source: &str) -> (ThinNodeArena, crate::parser::base::NodeIndex) {
    let mut parser = ThinParserState::new(
        "test.ts".to_string(),
        source.to_string(),
    );
    let _root = parser.parse_source_file();
    assert!(parser.get_diagnostics().is_empty(), "Parse errors: {:?}", parser.get_diagnostics());

    // Find the type alias declaration and extract its type_node
    let arena = std::mem::take(&mut parser.arena);

    // The type alias is typically node 1 (after source file at 0)
    // We need to find the FunctionType node
    for i in 0..arena.len() {
        let idx = crate::parser::base::NodeIndex(i as u32);
        if let Some(node) = arena.get(idx) {
            if node.kind == syntax_kind_ext::FUNCTION_TYPE ||
               node.kind == syntax_kind_ext::CONSTRUCTOR_TYPE {
                return (arena, idx);
            }
        }
    }

    panic!("Could not find function type in parsed AST");
}

/// Helper to parse a type alias and return its type node index
fn parse_type_alias_type_node(source: &str) -> (ThinNodeArena, crate::parser::base::NodeIndex) {
    let mut parser = ThinParserState::new(
        "test.ts".to_string(),
        source.to_string(),
    );
    let _root = parser.parse_source_file();
    assert!(parser.get_diagnostics().is_empty(), "Parse errors: {:?}", parser.get_diagnostics());

    let arena = std::mem::take(&mut parser.arena);

    let mut type_node = crate::parser::base::NodeIndex::NONE;
    for i in 0..arena.len() {
        let idx = crate::parser::base::NodeIndex(i as u32);
        if let Some(node) = arena.get(idx) {
            if node.kind == syntax_kind_ext::TYPE_ALIAS_DECLARATION {
                if let Some(alias) = arena.get_type_alias(node) {
                    type_node = alias.type_node;
                    break;
                }
            }
        }
    }

    if type_node == crate::parser::base::NodeIndex::NONE {
        panic!("Could not find type alias in parsed AST");
    }

    (arena, type_node)
}

#[test]
fn test_lower_function_type_with_type_parameter() {
    // Parse: type F = <T>(x: T) => T
    let (arena, func_type_idx) = parse_type_alias("type F = <T>(x: T) => T;");

    let interner = TypeInterner::new();
    let lowering = TypeLowering::new(&arena, &interner);

    let type_id = lowering.lower_type(func_type_idx);

    // Verify it's a function type
    let key = interner.lookup(type_id).expect("Type should exist");
    match key {
        TypeKey::Function(shape) => {
            // Should have 1 type parameter named "T"
            assert_eq!(shape.type_params.len(), 1, "Expected 1 type parameter");
            assert_eq!(interner.resolve_atom(shape.type_params[0].name).as_str(), "T");
            assert!(shape.type_params[0].constraint.is_none(), "T should have no constraint");
            assert!(shape.type_params[0].default.is_none(), "T should have no default");
        }
        _ => panic!("Expected Function type, got {:?}", key),
    }
}

#[test]
fn test_lower_function_type_with_constrained_type_parameter() {
    // Parse: type F = <T extends string>(x: T) => T
    let (arena, func_type_idx) = parse_type_alias("type F = <T extends string>(x: T) => T;");

    let interner = TypeInterner::new();
    let lowering = TypeLowering::new(&arena, &interner);

    let type_id = lowering.lower_type(func_type_idx);

    let key = interner.lookup(type_id).expect("Type should exist");
    match key {
        TypeKey::Function(shape) => {
            assert_eq!(shape.type_params.len(), 1);
            assert_eq!(interner.resolve_atom(shape.type_params[0].name).as_str(), "T");
            // Should have a constraint
            assert!(shape.type_params[0].constraint.is_some(), "T should have constraint");
            let constraint = shape.type_params[0].constraint.unwrap();
            assert_eq!(constraint, TypeId::STRING, "Constraint should be string");
        }
        _ => panic!("Expected Function type, got {:?}", key),
    }
}

#[test]
fn test_lower_function_type_with_default_type_parameter() {
    // Parse: type F = <T = string>() => T
    let (arena, func_type_idx) = parse_type_alias("type F = <T = string>() => T;");

    let interner = TypeInterner::new();
    let lowering = TypeLowering::new(&arena, &interner);

    let type_id = lowering.lower_type(func_type_idx);

    let key = interner.lookup(type_id).expect("Type should exist");
    match key {
        TypeKey::Function(shape) => {
            assert_eq!(shape.type_params.len(), 1);
            assert_eq!(interner.resolve_atom(shape.type_params[0].name).as_str(), "T");
            assert!(shape.type_params[0].constraint.is_none());
            // Should have a default
            assert!(shape.type_params[0].default.is_some(), "T should have default");
            let default = shape.type_params[0].default.unwrap();
            assert_eq!(default, TypeId::STRING, "Default should be string");
        }
        _ => panic!("Expected Function type, got {:?}", key),
    }
}

#[test]
fn test_lower_function_type_with_multiple_type_parameters() {
    // Parse: type F = <T, U, V>(x: T, y: U) => V
    let (arena, func_type_idx) = parse_type_alias("type F = <T, U, V>(x: T, y: U) => V;");

    let interner = TypeInterner::new();
    let lowering = TypeLowering::new(&arena, &interner);

    let type_id = lowering.lower_type(func_type_idx);

    let key = interner.lookup(type_id).expect("Type should exist");
    match key {
        TypeKey::Function(shape) => {
            assert_eq!(shape.type_params.len(), 3, "Expected 3 type parameters");
            assert_eq!(interner.resolve_atom(shape.type_params[0].name).as_str(), "T");
            assert_eq!(interner.resolve_atom(shape.type_params[1].name).as_str(), "U");
            assert_eq!(interner.resolve_atom(shape.type_params[2].name).as_str(), "V");
        }
        _ => panic!("Expected Function type, got {:?}", key),
    }
}

#[test]
fn test_lower_function_type_with_constraint_and_default() {
    // Parse: type F = <T extends object = {}>(x: T) => T
    let (arena, func_type_idx) = parse_type_alias("type F = <T extends object = {}>(x: T) => T;");

    let interner = TypeInterner::new();
    let lowering = TypeLowering::new(&arena, &interner);

    let type_id = lowering.lower_type(func_type_idx);

    let key = interner.lookup(type_id).expect("Type should exist");
    match key {
        TypeKey::Function(shape) => {
            assert_eq!(shape.type_params.len(), 1);
            assert_eq!(interner.resolve_atom(shape.type_params[0].name).as_str(), "T");
            // Should have both constraint and default
            assert!(shape.type_params[0].constraint.is_some(), "T should have constraint");
            assert!(shape.type_params[0].default.is_some(), "T should have default");
        }
        _ => panic!("Expected Function type, got {:?}", key),
    }
}

#[test]
fn test_lower_function_type_no_type_parameters() {
    // Parse: type F = (x: string) => number
    let (arena, func_type_idx) = parse_type_alias("type F = (x: string) => number;");

    let interner = TypeInterner::new();
    let lowering = TypeLowering::new(&arena, &interner);

    let type_id = lowering.lower_type(func_type_idx);

    let key = interner.lookup(type_id).expect("Type should exist");
    match key {
        TypeKey::Function(shape) => {
            assert_eq!(shape.type_params.len(), 0, "Expected no type parameters");
        }
        _ => panic!("Expected Function type, got {:?}", key),
    }
}

#[test]
fn test_lower_tuple_optional_and_rest_elements() {
    let (arena, tuple_type_idx) = parse_type_alias_type_node("type T = [number, string?, ...boolean[]];");

    let interner = TypeInterner::new();
    let lowering = TypeLowering::new(&arena, &interner);

    let type_id = lowering.lower_type(tuple_type_idx);

    let key = interner.lookup(type_id).expect("Type should exist");
    match key {
        TypeKey::Tuple(elements) => {
            assert_eq!(elements.len(), 3);

            assert_eq!(elements[0].type_id, TypeId::NUMBER);
            assert!(!elements[0].optional);
            assert!(!elements[0].rest);

            assert_eq!(elements[1].type_id, TypeId::STRING);
            assert!(elements[1].optional);
            assert!(!elements[1].rest);

            assert!(elements[2].rest);
            assert!(!elements[2].optional);
            match interner.lookup(elements[2].type_id) {
                Some(TypeKey::Array(elem)) => assert_eq!(elem, TypeId::BOOLEAN),
                other => panic!("Expected rest element to be array<boolean>, got {:?}", other),
            }
        }
        _ => panic!("Expected Tuple type, got {:?}", key),
    }
}

#[test]
fn test_lower_named_tuple_elements() {
    let (arena, tuple_type_idx) = parse_type_alias_type_node("type T = [name: string, age?: number, ...flags: boolean[]];");

    let interner = TypeInterner::new();
    let lowering = TypeLowering::new(&arena, &interner);

    let type_id = lowering.lower_type(tuple_type_idx);

    let key = interner.lookup(type_id).expect("Type should exist");
    match key {
        TypeKey::Tuple(elements) => {
            assert_eq!(elements.len(), 3);

            let name = elements[0].name.expect("Expected name for first element");
            assert_eq!(interner.resolve_atom(name).as_str(), "name");
            assert_eq!(elements[0].type_id, TypeId::STRING);
            assert!(!elements[0].optional);
            assert!(!elements[0].rest);

            let age = elements[1].name.expect("Expected name for second element");
            assert_eq!(interner.resolve_atom(age).as_str(), "age");
            assert_eq!(elements[1].type_id, TypeId::NUMBER);
            assert!(elements[1].optional);
            assert!(!elements[1].rest);

            let flags = elements[2].name.expect("Expected name for third element");
            assert_eq!(interner.resolve_atom(flags).as_str(), "flags");
            assert!(elements[2].rest);
            assert!(!elements[2].optional);
        }
        _ => panic!("Expected Tuple type, got {:?}", key),
    }
}
