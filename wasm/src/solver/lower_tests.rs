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

/// Helper to parse a type alias and return the tuple type node index
fn parse_tuple_type(source: &str) -> (ThinNodeArena, crate::parser::base::NodeIndex) {
    let mut parser = ThinParserState::new(
        "test.ts".to_string(),
        source.to_string(),
    );
    let _root = parser.parse_source_file();
    assert!(parser.get_diagnostics().is_empty(), "Parse errors: {:?}", parser.get_diagnostics());

    let arena = std::mem::take(&mut parser.arena);
    for i in 0..arena.len() {
        let idx = crate::parser::base::NodeIndex(i as u32);
        if let Some(node) = arena.get(idx) {
            if node.kind == syntax_kind_ext::TUPLE_TYPE {
                return (arena, idx);
            }
        }
    }

    panic!("Could not find tuple type in parsed AST");
}

/// Helper to parse a type alias and return the template literal type node index
fn parse_template_literal_type(source: &str) -> (ThinNodeArena, crate::parser::base::NodeIndex) {
    let mut parser = ThinParserState::new(
        "test.ts".to_string(),
        source.to_string(),
    );
    let _root = parser.parse_source_file();
    assert!(parser.get_diagnostics().is_empty(), "Parse errors: {:?}", parser.get_diagnostics());

    let arena = std::mem::take(&mut parser.arena);
    for i in 0..arena.len() {
        let idx = crate::parser::base::NodeIndex(i as u32);
        if let Some(node) = arena.get(idx) {
            if node.kind == syntax_kind_ext::TEMPLATE_LITERAL_TYPE {
                return (arena, idx);
            }
        }
    }

    panic!("Could not find template literal type in parsed AST");
}

/// Helper to parse a type alias and return the mapped type node index.
fn parse_mapped_type(source: &str) -> (ThinNodeArena, crate::parser::base::NodeIndex) {
    let mut parser = ThinParserState::new(
        "test.ts".to_string(),
        source.to_string(),
    );
    let _root = parser.parse_source_file();
    assert!(parser.get_diagnostics().is_empty(), "Parse errors: {:?}", parser.get_diagnostics());

    let arena = std::mem::take(&mut parser.arena);
    for i in 0..arena.len() {
        let idx = crate::parser::base::NodeIndex(i as u32);
        if let Some(node) = arena.get(idx) {
            if node.kind == syntax_kind_ext::MAPPED_TYPE {
                return (arena, idx);
            }
        }
    }

    panic!("Could not find mapped type in parsed AST");
}

/// Helper to parse a type alias and return the type reference node index for a name.
fn parse_type_reference(source: &str, name: &str) -> (ThinNodeArena, crate::parser::base::NodeIndex) {
    let mut parser = ThinParserState::new(
        "test.ts".to_string(),
        source.to_string(),
    );
    let _root = parser.parse_source_file();
    assert!(parser.get_diagnostics().is_empty(), "Parse errors: {:?}", parser.get_diagnostics());

    let arena = std::mem::take(&mut parser.arena);
    for i in 0..arena.len() {
        let idx = crate::parser::base::NodeIndex(i as u32);
        if let Some(node) = arena.get(idx) {
            if node.kind == syntax_kind_ext::TYPE_REFERENCE {
                if let Some(data) = arena.get_type_ref(node) {
                    if let Some(type_name_node) = arena.get(data.type_name) {
                        if let Some(ident) = arena.get_identifier(type_name_node) {
                            if ident.escaped_text == name {
                                return (arena, idx);
                            }
                        }
                    }
                }
            }
        }
    }

    panic!("Could not find type reference in parsed AST");
}

/// Helper to parse a type alias and return the type literal node index.
fn parse_type_literal(source: &str) -> (ThinNodeArena, crate::parser::base::NodeIndex) {
    let mut parser = ThinParserState::new(
        "test.ts".to_string(),
        source.to_string(),
    );
    let _root = parser.parse_source_file();
    assert!(parser.get_diagnostics().is_empty(), "Parse errors: {:?}", parser.get_diagnostics());

    let arena = std::mem::take(&mut parser.arena);
    for i in 0..arena.len() {
        let idx = crate::parser::base::NodeIndex(i as u32);
        if let Some(node) = arena.get(idx) {
            if node.kind == syntax_kind_ext::TYPE_LITERAL {
                return (arena, idx);
            }
        }
    }

    panic!("Could not find type literal in parsed AST");
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
fn test_lower_tuple_type_metadata() {
    let (arena, tuple_idx) = parse_tuple_type("type T = [x?: string, string?, ...number[]];");

    let interner = TypeInterner::new();
    let lowering = TypeLowering::new(&arena, &interner);

    let type_id = lowering.lower_type(tuple_idx);
    let key = interner.lookup(type_id).expect("Type should exist");
    match key {
        TypeKey::Tuple(elements) => {
            assert_eq!(elements.len(), 3);

            let first = &elements[0];
            assert_eq!(first.name.map(|a| interner.resolve_atom(a)), Some("x".to_string()));
            assert!(first.optional);
            assert!(!first.rest);
            assert_eq!(first.type_id, TypeId::STRING);

            let second = &elements[1];
            assert!(second.name.is_none());
            assert!(second.optional);
            assert!(!second.rest);
            assert_eq!(second.type_id, TypeId::STRING);

            let third = &elements[2];
            assert!(third.name.is_none());
            assert!(!third.optional);
            assert!(third.rest);
            match interner.lookup(third.type_id) {
                Some(TypeKey::Array(elem)) => assert_eq!(elem, TypeId::NUMBER),
                other => panic!("Expected array type for rest element, got {:?}", other),
            }
        }
        _ => panic!("Expected Tuple type, got {:?}", key),
    }
}

#[test]
fn test_lower_union_type_normalization() {
    let (arena, union_idx) = parse_type_alias_type_node("type T = string | number | string;");
    let interner = TypeInterner::new();
    let lowering = TypeLowering::new(&arena, &interner);

    let type_id = lowering.lower_type(union_idx);
    let key = interner.lookup(type_id).expect("Type should exist");
    match key {
        TypeKey::Union(members) => {
            assert_eq!(members, vec![TypeId::NUMBER, TypeId::STRING]);
        }
        _ => panic!("Expected Union type, got {:?}", key),
    }
}

#[test]
fn test_lower_intersection_type_normalization() {
    let (arena, intersection_idx) = parse_type_alias_type_node("type T = string & number & string;");
    let interner = TypeInterner::new();
    let lowering = TypeLowering::new(&arena, &interner);

    let type_id = lowering.lower_type(intersection_idx);
    let key = interner.lookup(type_id).expect("Type should exist");
    match key {
        TypeKey::Intersection(members) => {
            assert_eq!(members, vec![TypeId::NUMBER, TypeId::STRING]);
        }
        _ => panic!("Expected Intersection type, got {:?}", key),
    }
}

#[test]
fn test_lower_function_parameter_names() {
    let (arena, func_type_idx) = parse_type_alias("type F = (x: string, y?: number) => void;");

    let interner = TypeInterner::new();
    let lowering = TypeLowering::new(&arena, &interner);

    let type_id = lowering.lower_type(func_type_idx);
    let key = interner.lookup(type_id).expect("Type should exist");
    match key {
        TypeKey::Function(shape) => {
            assert_eq!(shape.params.len(), 2);
            assert_eq!(shape.params[0].name.map(|a| interner.resolve_atom(a)), Some("x".to_string()));
            assert_eq!(shape.params[0].type_id, TypeId::STRING);
            assert!(!shape.params[0].optional);

            assert_eq!(shape.params[1].name.map(|a| interner.resolve_atom(a)), Some("y".to_string()));
            assert_eq!(shape.params[1].type_id, TypeId::NUMBER);
            assert!(shape.params[1].optional);

            assert_eq!(shape.return_type, TypeId::VOID);
        }
        _ => panic!("Expected Function type, got {:?}", key),
    }
}

#[test]
fn test_lower_type_reference_with_arguments() {
    let (arena, type_ref_idx) = parse_type_reference("type T = Box<string>;", "Box");
    let interner = TypeInterner::new();

    let resolver = |node_idx: NodeIndex| {
        arena.get(node_idx)
            .and_then(|node| arena.get_identifier(node))
            .and_then(|ident| {
                if ident.escaped_text == "Box" {
                    Some(1)
                } else {
                    None
                }
            })
    };

    let lowering = TypeLowering::with_resolver(&arena, &interner, &resolver);
    let type_id = lowering.lower_type(type_ref_idx);
    let key = interner.lookup(type_id).expect("Type should exist");
    match key {
        TypeKey::Application(app) => {
            assert_eq!(app.args, vec![TypeId::STRING]);
            match interner.lookup(app.base) {
                Some(TypeKey::Ref(SymbolRef(sym_id))) => assert_eq!(sym_id, 1),
                other => panic!("Expected Ref base type, got {:?}", other),
            }
        }
        _ => panic!("Expected Application type, got {:?}", key),
    }
}

#[test]
fn test_lower_template_literal_type_spans() {
    let (arena, template_idx) = parse_template_literal_type("type T = `hello${string}world`;");

    let interner = TypeInterner::new();
    let lowering = TypeLowering::new(&arena, &interner);

    let type_id = lowering.lower_type(template_idx);
    let key = interner.lookup(type_id).expect("Type should exist");
    match key {
        TypeKey::TemplateLiteral(spans) => {
            assert_eq!(spans.len(), 3);
            match spans[0] {
                TemplateSpan::Text(atom) => assert_eq!(interner.resolve_atom(atom), "hello"),
                _ => panic!("Expected head text span"),
            }
            match spans[1] {
                TemplateSpan::Type(t) => assert_eq!(t, TypeId::STRING),
                _ => panic!("Expected type span"),
            }
            match spans[2] {
                TemplateSpan::Text(atom) => assert_eq!(interner.resolve_atom(atom), "world"),
                _ => panic!("Expected tail text span"),
            }
        }
        _ => panic!("Expected TemplateLiteral type, got {:?}", key),
    }
}

#[test]
fn test_lower_mapped_type_modifiers_and_constraint() {
    let (arena, mapped_idx) = parse_mapped_type("type T = { readonly [K in string]?: number };");
    let interner = TypeInterner::new();
    let lowering = TypeLowering::new(&arena, &interner);

    let type_id = lowering.lower_type(mapped_idx);
    let key = interner.lookup(type_id).expect("Type should exist");
    match key {
        TypeKey::Mapped(mapped) => {
            assert_eq!(interner.resolve_atom(mapped.type_param.name), "K");
            assert_eq!(mapped.constraint, TypeId::STRING);
            assert_eq!(mapped.template, TypeId::NUMBER);
            assert_eq!(mapped.readonly_modifier, Some(MappedModifier::Add));
            assert_eq!(mapped.optional_modifier, Some(MappedModifier::Add));
        }
        _ => panic!("Expected Mapped type, got {:?}", key),
    }
}

#[test]
fn test_lower_mapped_type_remove_modifiers() {
    let (arena, mapped_idx) = parse_mapped_type("type T = { -readonly [K in string]-?: number };");
    let interner = TypeInterner::new();
    let lowering = TypeLowering::new(&arena, &interner);

    let type_id = lowering.lower_type(mapped_idx);
    let key = interner.lookup(type_id).expect("Type should exist");
    match key {
        TypeKey::Mapped(mapped) => {
            assert_eq!(mapped.readonly_modifier, Some(MappedModifier::Remove));
            assert_eq!(mapped.optional_modifier, Some(MappedModifier::Remove));
        }
        _ => panic!("Expected Mapped type, got {:?}", key),
    }
}

#[test]
fn test_lower_type_literal_object_properties() {
    let (arena, literal_idx) = parse_type_literal("type T = { readonly foo?: string; bar: number; };");
    let interner = TypeInterner::new();
    let lowering = TypeLowering::new(&arena, &interner);

    let type_id = lowering.lower_type(literal_idx);
    let key = interner.lookup(type_id).expect("Type should exist");
    match key {
        TypeKey::Object(properties) => {
            let foo = properties.iter()
                .find(|prop| interner.resolve_atom(prop.name) == "foo")
                .expect("Expected foo property");
            assert_eq!(foo.type_id, TypeId::STRING);
            assert!(foo.optional);
            assert!(foo.readonly);

            let bar = properties.iter()
                .find(|prop| interner.resolve_atom(prop.name) == "bar")
                .expect("Expected bar property");
            assert_eq!(bar.type_id, TypeId::NUMBER);
            assert!(!bar.optional);
            assert!(!bar.readonly);
        }
        _ => panic!("Expected Object type, got {:?}", key),
    }
}

#[test]
fn test_lower_type_literal_call_signature() {
    let (arena, literal_idx) = parse_type_literal("type T = { (x: string): number; foo: string; };");
    let interner = TypeInterner::new();
    let lowering = TypeLowering::new(&arena, &interner);

    let type_id = lowering.lower_type(literal_idx);
    let key = interner.lookup(type_id).expect("Type should exist");
    match key {
        TypeKey::Callable(callable) => {
            assert_eq!(callable.call_signatures.len(), 1);
            assert_eq!(callable.construct_signatures.len(), 0);
            assert_eq!(callable.properties.len(), 1);
            assert_eq!(interner.resolve_atom(callable.properties[0].name), "foo");
            assert_eq!(callable.properties[0].type_id, TypeId::STRING);
        }
        _ => panic!("Expected Callable type, got {:?}", key),
    }
}

#[test]
fn test_lower_type_literal_construct_signature() {
    let (arena, literal_idx) = parse_type_literal("type T = { new (x: string): number; };");
    let interner = TypeInterner::new();
    let lowering = TypeLowering::new(&arena, &interner);

    let type_id = lowering.lower_type(literal_idx);
    let key = interner.lookup(type_id).expect("Type should exist");
    match key {
        TypeKey::Callable(callable) => {
            assert_eq!(callable.call_signatures.len(), 0);
            assert_eq!(callable.construct_signatures.len(), 1);
        }
        _ => panic!("Expected Callable type, got {:?}", key),
    }
}

#[test]
fn test_lower_type_literal_index_signature() {
    let (arena, literal_idx) = parse_type_literal("type T = { [key: string]: number; foo: string; };");
    let interner = TypeInterner::new();
    let lowering = TypeLowering::new(&arena, &interner);

    let type_id = lowering.lower_type(literal_idx);
    let key = interner.lookup(type_id).expect("Type should exist");
    match key {
        TypeKey::ObjectWithIndex(shape) => {
            assert_eq!(shape.properties.len(), 1);
            assert_eq!(interner.resolve_atom(shape.properties[0].name), "foo");
            let string_index = shape.string_index.expect("Expected string index signature");
            assert_eq!(string_index.key_type, TypeId::STRING);
            assert_eq!(string_index.value_type, TypeId::NUMBER);
        }
        _ => panic!("Expected ObjectWithIndex type, got {:?}", key),
    }
}
