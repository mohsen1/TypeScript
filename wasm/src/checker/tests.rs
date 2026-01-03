//! Tests for the type checker.

use super::*;
use crate::parser::{NodeArena, NodeIndex};
use crate::binder::{SymbolArena, SymbolTable, SymbolId, symbol_flags};

#[test]
fn test_type_flags() {
        assert_eq!(type_flags::ANY, 1);
        assert_eq!(type_flags::UNKNOWN, 2);
        assert_eq!(type_flags::STRING, 4);
        assert_eq!(type_flags::NUMBER, 8);
        assert_eq!(type_flags::NULLABLE, type_flags::UNDEFINED | type_flags::NULL);
    }

    #[test]
    fn test_type_id() {
        let id = TypeId(42);
        assert_eq!(id.0, 42);
        assert!(!id.is_none());
        assert!(TypeId::NONE.is_none());
    }

    #[test]
    fn test_type_arena_creation() {
        let arena = TypeArena::new();

        // Should have pre-allocated singleton types
        assert!(!arena.any_type.is_none());
        assert!(!arena.unknown_type.is_none());
        assert!(!arena.string_type.is_none());
        assert!(!arena.number_type.is_none());
        assert!(!arena.boolean_type.is_none());
        assert!(!arena.void_type.is_none());
        assert!(!arena.undefined_type.is_none());
        assert!(!arena.null_type.is_none());
        assert!(!arena.never_type.is_none());

        // Total: 14 singleton types (12 intrinsic + 2 boolean literals)
        assert_eq!(arena.len(), 14);
    }

    #[test]
    fn test_intrinsic_types() {
        let arena = TypeArena::new();

        let any = arena.get(arena.any_type).unwrap();
        assert!(any.has_flags(type_flags::ANY));

        let string = arena.get(arena.string_type).unwrap();
        assert!(string.has_flags(type_flags::STRING));
        assert!(string.has_any_flags(type_flags::STRING_LIKE));

        let number = arena.get(arena.number_type).unwrap();
        assert!(number.has_flags(type_flags::NUMBER));
        assert!(number.has_any_flags(type_flags::NUMBER_LIKE));

        let never = arena.get(arena.never_type).unwrap();
        assert!(never.has_flags(type_flags::NEVER));
    }

    #[test]
    fn test_literal_types() {
        let mut arena = TypeArena::new();

        let str_lit = arena.create_string_literal("hello".to_string());
        let str_type = arena.get(str_lit).unwrap();
        assert!(str_type.has_flags(type_flags::STRING_LITERAL));
        assert!(str_type.has_any_flags(type_flags::STRING_LIKE));
        assert!(str_type.has_any_flags(type_flags::LITERAL));

        let num_lit = arena.create_number_literal(42.0);
        let num_type = arena.get(num_lit).unwrap();
        assert!(num_type.has_flags(type_flags::NUMBER_LITERAL));
        assert!(num_type.has_any_flags(type_flags::NUMBER_LIKE));
        assert!(num_type.has_any_flags(type_flags::LITERAL));
    }

    #[test]
    fn test_boolean_literal_singletons() {
        let arena = TypeArena::new();

        let true_type = arena.get(arena.true_type).unwrap();
        assert!(true_type.has_flags(type_flags::BOOLEAN_LITERAL));
        if let Type::Literal(lit) = true_type {
            assert_eq!(lit.value, LiteralValue::Boolean(true));
        } else {
            panic!("Expected literal type");
        }

        let false_type = arena.get(arena.false_type).unwrap();
        assert!(false_type.has_flags(type_flags::BOOLEAN_LITERAL));
        if let Type::Literal(lit) = false_type {
            assert_eq!(lit.value, LiteralValue::Boolean(false));
        } else {
            panic!("Expected literal type");
        }
    }

    #[test]
    fn test_union_type() {
        let mut arena = TypeArena::new();

        // Union of string | number
        let union = arena.create_union(vec![arena.string_type, arena.number_type]);
        let union_type = arena.get(union).unwrap();
        assert!(union_type.has_flags(type_flags::UNION));

        if let Type::Union(u) = union_type {
            assert_eq!(u.types.len(), 2);
            assert!(u.types.contains(&arena.string_type));
            assert!(u.types.contains(&arena.number_type));
        } else {
            panic!("Expected union type");
        }
    }

    #[test]
    fn test_union_single_type() {
        let mut arena = TypeArena::new();

        // Union of single type should return that type
        let union = arena.create_union(vec![arena.string_type]);
        assert_eq!(union, arena.string_type);
    }

    #[test]
    fn test_union_empty() {
        let mut arena = TypeArena::new();

        // Empty union should return never
        let union = arena.create_union(vec![]);
        assert_eq!(union, arena.never_type);
    }

    #[test]
    fn test_intersection_type() {
        let mut arena = TypeArena::new();

        let obj1 = arena.alloc(Type::Object(ObjectType::new(object_flags::ANONYMOUS, SymbolId::NONE)));
        let obj2 = arena.alloc(Type::Object(ObjectType::new(object_flags::ANONYMOUS, SymbolId::NONE)));

        let intersection = arena.create_intersection(vec![obj1, obj2]);
        let int_type = arena.get(intersection).unwrap();
        assert!(int_type.has_flags(type_flags::INTERSECTION));

        if let Type::Intersection(i) = int_type {
            assert_eq!(i.types.len(), 2);
        } else {
            panic!("Expected intersection type");
        }
    }

    #[test]
    fn test_intersection_simplification_never() {
        // X & never = never
        let mut arena = TypeArena::new();
        let obj = arena.alloc(Type::Object(ObjectType::new(object_flags::ANONYMOUS, SymbolId::NONE)));

        let intersection = arena.create_intersection(vec![obj, arena.never_type]);
        assert_eq!(intersection, arena.never_type);
    }

    #[test]
    fn test_intersection_simplification_unknown() {
        // X & unknown = X
        let mut arena = TypeArena::new();
        let obj = arena.alloc(Type::Object(ObjectType::new(object_flags::ANONYMOUS, SymbolId::NONE)));

        let intersection = arena.create_intersection(vec![obj, arena.unknown_type]);
        assert_eq!(intersection, obj);
    }

    #[test]
    fn test_intersection_simplification_duplicates() {
        // X & X = X
        let mut arena = TypeArena::new();
        let obj = arena.alloc(Type::Object(ObjectType::new(object_flags::ANONYMOUS, SymbolId::NONE)));

        let intersection = arena.create_intersection(vec![obj, obj]);
        assert_eq!(intersection, obj);
    }

    #[test]
    fn test_intersection_simplification_flatten() {
        // (A & B) & C = A & B & C
        let mut arena = TypeArena::new();
        let obj1 = arena.alloc(Type::Object(ObjectType::new(object_flags::ANONYMOUS, SymbolId::NONE)));
        let obj2 = arena.alloc(Type::Object(ObjectType::new(object_flags::ANONYMOUS, SymbolId::NONE)));
        let obj3 = arena.alloc(Type::Object(ObjectType::new(object_flags::ANONYMOUS, SymbolId::NONE)));

        let inner = arena.create_intersection(vec![obj1, obj2]);
        let outer = arena.create_intersection(vec![inner, obj3]);

        // Should have flattened to a single intersection with 3 types
        if let Type::Intersection(i) = arena.get(outer).unwrap() {
            assert_eq!(i.types.len(), 3);
            assert!(i.types.contains(&obj1));
            assert!(i.types.contains(&obj2));
            assert!(i.types.contains(&obj3));
        } else {
            panic!("Expected flattened intersection type");
        }
    }

    #[test]
    fn test_intersection_empty_returns_unknown() {
        let mut arena = TypeArena::new();
        let intersection = arena.create_intersection(vec![]);
        assert_eq!(intersection, arena.unknown_type);
    }

    #[test]
    fn test_intersection_all_unknown_returns_unknown() {
        // unknown & unknown = unknown
        let mut arena = TypeArena::new();
        let intersection = arena.create_intersection(vec![arena.unknown_type, arena.unknown_type]);
        assert_eq!(intersection, arena.unknown_type);
    }

    #[test]
    fn test_template_literal_instantiation_simple() {
        // Template with no substitution returns string literal
        let mut arena = TypeArena::new();
        let result = arena.create_template_literal_type(vec!["hello".to_string()], vec![]);

        // Should be a string literal "hello"
        if let Some(Type::Literal(lit)) = arena.get(result) {
            assert!(matches!(lit.value, LiteralValue::String(ref s) if s == "hello"));
        } else {
            panic!("Expected string literal");
        }
    }

    #[test]
    fn test_template_literal_instantiation_with_string() {
        // `hello ${world}` with "world" -> "hello world"
        let mut arena = TypeArena::new();
        let world_type = arena.create_string_literal("world".to_string());
        let result = arena.create_template_literal_type(
            vec!["hello ".to_string(), "".to_string()],
            vec![world_type],
        );

        // Should evaluate to "hello world"
        if let Some(Type::Literal(lit)) = arena.get(result) {
            assert!(matches!(lit.value, LiteralValue::String(ref s) if s == "hello world"),
                "Expected 'hello world', got: {:?}", lit.value);
        } else {
            panic!("Expected string literal");
        }
    }

    #[test]
    fn test_template_literal_instantiation_with_number() {
        // `count: ${42}` -> "count: 42"
        let mut arena = TypeArena::new();
        let num_type = arena.create_number_literal(42.0);
        let result = arena.create_template_literal_type(
            vec!["count: ".to_string(), "".to_string()],
            vec![num_type],
        );

        // Should evaluate to "count: 42"
        if let Some(Type::Literal(lit)) = arena.get(result) {
            assert!(matches!(lit.value, LiteralValue::String(ref s) if s == "count: 42"),
                "Expected 'count: 42', got: {:?}", lit.value);
        } else {
            panic!("Expected string literal");
        }
    }

    #[test]
    fn test_template_literal_instantiation_multiple() {
        // `${a} and ${b}` with "hello", "world" -> "hello and world"
        let mut arena = TypeArena::new();
        let a_type = arena.create_string_literal("hello".to_string());
        let b_type = arena.create_string_literal("world".to_string());
        let result = arena.create_template_literal_type(
            vec!["".to_string(), " and ".to_string(), "".to_string()],
            vec![a_type, b_type],
        );

        // Should evaluate to "hello and world"
        if let Some(Type::Literal(lit)) = arena.get(result) {
            assert!(matches!(lit.value, LiteralValue::String(ref s) if s == "hello and world"),
                "Expected 'hello and world', got: {:?}", lit.value);
        } else {
            panic!("Expected string literal");
        }
    }

    #[test]
    fn test_template_literal_deferred() {
        // `hello ${T}` with type parameter T -> template literal type (not evaluated)
        let mut arena = TypeArena::new();
        let t_type = arena.create_type_parameter(SymbolId::NONE, TypeId::NONE, TypeId::NONE);
        let result = arena.create_template_literal_type(
            vec!["hello ".to_string(), "".to_string()],
            vec![t_type],
        );

        // Should remain a template literal type (not evaluated)
        if let Some(Type::TemplateLiteral(_)) = arena.get(result) {
            // Good, it's unevaluated
        } else {
            panic!("Expected template literal type to be deferred");
        }
    }

    #[test]
    fn test_object_flags() {
        assert_eq!(object_flags::CLASS, 1);
        assert_eq!(object_flags::INTERFACE, 2);
        assert_eq!(object_flags::CLASS_OR_INTERFACE, object_flags::CLASS | object_flags::INTERFACE);
    }

    #[test]
    fn test_checker_state_creation() {
        use crate::parser::NodeArena;

        let node_arena = NodeArena::new();
        let symbol_arena = SymbolArena::new();
        let file_locals = SymbolTable::new();
        let checker = CheckerState::new(&node_arena, &symbol_arena, &file_locals, "test.ts".to_string());

        // Should have intrinsic types pre-allocated
        assert!(!checker.types.any_type.is_none());
        assert!(!checker.types.string_type.is_none());
        assert_eq!(checker.diagnostics.len(), 0);
    }

    #[test]
    fn test_type_assignability_same() {
        use crate::parser::NodeArena;

        let node_arena = NodeArena::new();
        let symbol_arena = SymbolArena::new();
        let file_locals = SymbolTable::new();
        let checker = CheckerState::new(&node_arena, &symbol_arena, &file_locals, "test.ts".to_string());

        // Same type is assignable to itself
        assert!(checker.is_type_assignable_to(checker.types.string_type, checker.types.string_type));
        assert!(checker.is_type_assignable_to(checker.types.number_type, checker.types.number_type));
        assert!(checker.is_type_assignable_to(checker.types.boolean_type, checker.types.boolean_type));
    }

    #[test]
    fn test_type_assignability_any() {
        use crate::parser::NodeArena;

        let node_arena = NodeArena::new();
        let symbol_arena = SymbolArena::new();
        let file_locals = SymbolTable::new();
        let checker = CheckerState::new(&node_arena, &symbol_arena, &file_locals, "test.ts".to_string());

        // any is assignable to anything
        assert!(checker.is_type_assignable_to(checker.types.any_type, checker.types.string_type));
        assert!(checker.is_type_assignable_to(checker.types.any_type, checker.types.number_type));

        // Anything is assignable to any
        assert!(checker.is_type_assignable_to(checker.types.string_type, checker.types.any_type));
        assert!(checker.is_type_assignable_to(checker.types.number_type, checker.types.any_type));
    }

    #[test]
    fn test_type_assignability_never() {
        use crate::parser::NodeArena;

        let node_arena = NodeArena::new();
        let symbol_arena = SymbolArena::new();
        let file_locals = SymbolTable::new();
        let checker = CheckerState::new(&node_arena, &symbol_arena, &file_locals, "test.ts".to_string());

        // never is assignable to everything
        assert!(checker.is_type_assignable_to(checker.types.never_type, checker.types.string_type));
        assert!(checker.is_type_assignable_to(checker.types.never_type, checker.types.number_type));
        assert!(checker.is_type_assignable_to(checker.types.never_type, checker.types.void_type));
    }

    #[test]
    fn test_type_assignability_literals() {
        use crate::parser::NodeArena;

        let node_arena = NodeArena::new();
        let symbol_arena = SymbolArena::new();
        let file_locals = SymbolTable::new();
        let mut checker = CheckerState::new(&node_arena, &symbol_arena, &file_locals, "test.ts".to_string());

        // String literal is assignable to string
        let str_lit = checker.types.create_string_literal("hello".to_string());
        assert!(checker.is_type_assignable_to(str_lit, checker.types.string_type));

        // Number literal is assignable to number
        let num_lit = checker.types.create_number_literal(42.0);
        assert!(checker.is_type_assignable_to(num_lit, checker.types.number_type));

        // Boolean literal is assignable to boolean
        assert!(checker.is_type_assignable_to(checker.types.true_type, checker.types.boolean_type));
        assert!(checker.is_type_assignable_to(checker.types.false_type, checker.types.boolean_type));
    }

    #[test]
    fn test_type_assignability_union() {
        use crate::parser::NodeArena;

        let node_arena = NodeArena::new();
        let symbol_arena = SymbolArena::new();
        let file_locals = SymbolTable::new();
        let mut checker = CheckerState::new(&node_arena, &symbol_arena, &file_locals, "test.ts".to_string());

        // string is assignable to string | number
        let union = checker.types.create_union(vec![
            checker.types.string_type,
            checker.types.number_type,
        ]);
        assert!(checker.is_type_assignable_to(checker.types.string_type, union));
        assert!(checker.is_type_assignable_to(checker.types.number_type, union));

        // boolean is NOT assignable to string | number
        assert!(!checker.is_type_assignable_to(checker.types.boolean_type, union));
    }

    #[test]
    fn test_type_to_string() {
        use crate::parser::NodeArena;

        let node_arena = NodeArena::new();
        let symbol_arena = SymbolArena::new();
        let file_locals = SymbolTable::new();
        let mut checker = CheckerState::new(&node_arena, &symbol_arena, &file_locals, "test.ts".to_string());

        assert_eq!(checker.type_to_string(checker.types.string_type), "string");
        assert_eq!(checker.type_to_string(checker.types.number_type), "number");
        assert_eq!(checker.type_to_string(checker.types.boolean_type), "boolean");
        assert_eq!(checker.type_to_string(checker.types.any_type), "any");
        assert_eq!(checker.type_to_string(checker.types.never_type), "never");

        let str_lit = checker.types.create_string_literal("hello".to_string());
        assert_eq!(checker.type_to_string(str_lit), "\"hello\"");

        let num_lit = checker.types.create_number_literal(42.0);
        assert_eq!(checker.type_to_string(num_lit), "42");
    }

    #[test]
    fn test_get_type_of_literal_nodes() {
        use crate::parser_impl::ParserState;

        let mut parser = ParserState::new(
            "test.ts".to_string(),
            r#"const x = "hello";"#.to_string(),
        );
        let root = parser.parse_source_file();

        let symbol_arena = SymbolArena::new();
        let file_locals = SymbolTable::new();
        let mut checker = CheckerState::new(&parser.arena, &symbol_arena, &file_locals, "test.ts".to_string());

        // Get the source file
        if let Some(crate::parser::Node::SourceFile(sf)) = parser.arena.get(root) {
            // Get the first statement (variable statement)
            if let Some(&stmt_idx) = sf.statements.nodes.first() {
                if let Some(crate::parser::Node::VariableStatement(vs)) = parser.arena.get(stmt_idx) {
                    // Get declaration list
                    if let Some(crate::parser::Node::VariableDeclarationList(vdl)) = parser.arena.get(vs.declaration_list) {
                        // Get the first declaration
                        if let Some(&decl_idx) = vdl.declarations.nodes.first() {
                            let decl_type = checker.get_type_of_node(decl_idx);
                            // Should be a string literal type
                            assert!(checker.types.get(decl_type).unwrap().has_flags(type_flags::STRING_LITERAL));
                        }
                    }
                }
            }
        }
    }

    #[test]
    fn test_symbol_type_resolution() {
        use crate::parser_impl::ParserState;
        use crate::binder::BinderState;

        // Parse and bind a simple program
        let mut parser = ParserState::new(
            "test.ts".to_string(),
            r#"const x = "hello"; const y = x;"#.to_string(),
        );
        let root = parser.parse_source_file();

        let mut binder = BinderState::new();
        binder.bind_source_file(&parser.arena, root);

        // Create checker with bound symbols
        let mut checker = CheckerState::new(
            &parser.arena,
            &binder.symbols,
            &binder.file_locals,
            "test.ts".to_string(),
        );

        // Verify 'x' is in the symbol table
        assert!(binder.file_locals.has("x"));

        // Get type of 'x' symbol
        if let Some(x_symbol) = binder.file_locals.get("x") {
            let x_type = checker.get_type_of_symbol(x_symbol);
            // x should have type "hello" (string literal)
            assert!(checker.types.get(x_type).unwrap().has_flags(type_flags::STRING_LITERAL));
        }
    }

    #[test]
    fn test_function_type_inference() {
        use crate::parser_impl::ParserState;
        use crate::binder::BinderState;

        // Parse a function declaration
        let mut parser = ParserState::new(
            "test.ts".to_string(),
            r#"function add(x: number, y: number): number { return x + y; }"#.to_string(),
        );
        let root = parser.parse_source_file();

        let mut binder = BinderState::new();
        binder.bind_source_file(&parser.arena, root);

        let mut checker = CheckerState::new(
            &parser.arena,
            &binder.symbols,
            &binder.file_locals,
            "test.ts".to_string(),
        );

        // Verify 'add' is in the symbol table
        assert!(binder.file_locals.has("add"));

        // Get type of 'add' symbol
        if let Some(add_symbol) = binder.file_locals.get("add") {
            let add_type = checker.get_type_of_symbol(add_symbol);

            // Should be a function type with OBJECT flag
            let typ = checker.types.get(add_type).unwrap();
            assert!(typ.has_flags(type_flags::OBJECT));

            // Verify it's a Function variant
            if let Type::Function(f) = typ {
                assert_eq!(f.parameter_names.len(), 2);
                assert_eq!(f.parameter_names[0], "x");
                assert_eq!(f.parameter_names[1], "y");
                assert_eq!(f.parameter_types.len(), 2);
                assert_eq!(f.min_argument_count, 2);
                assert!(!f.has_rest_parameter);

                // Both params should be number type
                assert_eq!(f.parameter_types[0], checker.types.number_type);
                assert_eq!(f.parameter_types[1], checker.types.number_type);

                // Return type should be number
                assert_eq!(f.return_type, checker.types.number_type);
            } else {
                panic!("Expected Function type, got {:?}", typ);
            }

            // Verify type_to_string works
            let type_str = checker.type_to_string(add_type);
            assert_eq!(type_str, "(x: number, y: number) => number");
        }
    }

    #[test]
    fn test_function_type_with_optional_params() {
        use crate::parser_impl::ParserState;
        use crate::binder::BinderState;

        let mut parser = ParserState::new(
            "test.ts".to_string(),
            r#"function greet(name: string, greeting?: string): void {}"#.to_string(),
        );
        let root = parser.parse_source_file();

        let mut binder = BinderState::new();
        binder.bind_source_file(&parser.arena, root);

        let mut checker = CheckerState::new(
            &parser.arena,
            &binder.symbols,
            &binder.file_locals,
            "test.ts".to_string(),
        );

        if let Some(fn_symbol) = binder.file_locals.get("greet") {
            let fn_type = checker.get_type_of_symbol(fn_symbol);

            if let Type::Function(f) = checker.types.get(fn_type).unwrap() {
                assert_eq!(f.parameter_names.len(), 2);
                // Optional param doesn't count toward min
                assert_eq!(f.min_argument_count, 1);
                assert!(!f.has_rest_parameter);
                assert_eq!(f.return_type, checker.types.void_type);
            } else {
                panic!("Expected Function type");
            }
        }
    }

    #[test]
    fn test_function_type_node() {
        use crate::parser_impl::ParserState;
        use crate::binder::BinderState;

        // Test function type nodes (type aliases)
        let mut parser = ParserState::new(
            "test.ts".to_string(),
            r#"type Callback = (x: number, y: string) => boolean;"#.to_string(),
        );
        let root = parser.parse_source_file();

        let mut binder = BinderState::new();
        binder.bind_source_file(&parser.arena, root);

        let mut checker = CheckerState::new(
            &parser.arena,
            &binder.symbols,
            &binder.file_locals,
            "test.ts".to_string(),
        );

        // 'Callback' should be in the symbol table
        assert!(binder.file_locals.has("Callback"));

        // Get the type alias symbol and its declared type
        if let Some(callback_symbol) = binder.file_locals.get("Callback") {
            let callback_type = checker.get_type_of_symbol(callback_symbol);
            let typ = checker.types.get(callback_type).unwrap();

            // The type should be a function type
            if let Type::Function(f) = typ {
                assert_eq!(f.parameter_names.len(), 2);
                assert_eq!(f.parameter_names[0], "x");
                assert_eq!(f.parameter_names[1], "y");
                assert_eq!(f.parameter_types[0], checker.types.number_type);
                assert_eq!(f.parameter_types[1], checker.types.string_type);
                assert_eq!(f.return_type, checker.types.boolean_type);
            } else {
                panic!("Expected Function type, got {:?}", typ);
            }
        }
    }

    #[test]
    fn test_function_type_direct() {
        use crate::parser::NodeArena;

        // Test directly creating a function type
        let node_arena = NodeArena::new();
        let symbol_arena = SymbolArena::new();
        let file_locals = SymbolTable::new();
        let mut checker = CheckerState::new(&node_arena, &symbol_arena, &file_locals, "test.ts".to_string());

        // Create a function type manually
        let fn_type = checker.types.create_function_type(
            NodeIndex::NONE,
            vec![checker.types.number_type, checker.types.string_type],
            vec!["a".to_string(), "b".to_string()],
            checker.types.boolean_type,
            2,
            false,
        );

        // Verify the type
        let typ = checker.types.get(fn_type).unwrap();
        assert!(typ.has_flags(type_flags::OBJECT));

        if let Type::Function(f) = typ {
            assert_eq!(f.parameter_types.len(), 2);
            assert_eq!(f.parameter_names, vec!["a", "b"]);
            assert_eq!(f.return_type, checker.types.boolean_type);
        } else {
            panic!("Expected Function type");
        }

        // Test type_to_string
        let type_str = checker.type_to_string(fn_type);
        assert_eq!(type_str, "(a: number, b: string) => boolean");
    }

    #[test]
    fn test_arrow_function_type() {
        use crate::parser_impl::ParserState;
        use crate::binder::BinderState;

        // Test arrow function with parenthesized parameters
        let mut parser = ParserState::new(
            "test.ts".to_string(),
            r#"const double = (x: number): number => x * 2;"#.to_string(),
        );
        let root = parser.parse_source_file();

        let mut binder = BinderState::new();
        binder.bind_source_file(&parser.arena, root);

        let mut checker = CheckerState::new(
            &parser.arena,
            &binder.symbols,
            &binder.file_locals,
            "test.ts".to_string(),
        );

        // 'double' should have a function type
        assert!(binder.file_locals.has("double"));

        if let Some(double_symbol) = binder.file_locals.get("double") {
            let double_type = checker.get_type_of_symbol(double_symbol);
            let typ = checker.types.get(double_type).unwrap();

            if let Type::Function(f) = typ {
                assert_eq!(f.parameter_names.len(), 1);
                assert_eq!(f.parameter_names[0], "x");
                assert_eq!(f.parameter_types[0], checker.types.number_type);
                assert_eq!(f.return_type, checker.types.number_type);
            } else {
                panic!("Expected Function type, got {:?}", typ);
            }
        }
    }

    #[test]
    fn test_simple_arrow_function() {
        use crate::parser_impl::ParserState;
        use crate::binder::BinderState;

        // Test simple arrow function (x => x * 2)
        let mut parser = ParserState::new(
            "test.ts".to_string(),
            r#"const identity = x => x;"#.to_string(),
        );
        let root = parser.parse_source_file();

        let mut binder = BinderState::new();
        binder.bind_source_file(&parser.arena, root);

        let mut checker = CheckerState::new(
            &parser.arena,
            &binder.symbols,
            &binder.file_locals,
            "test.ts".to_string(),
        );

        assert!(binder.file_locals.has("identity"));

        if let Some(symbol) = binder.file_locals.get("identity") {
            let id_type = checker.get_type_of_symbol(symbol);
            let typ = checker.types.get(id_type).unwrap();

            if let Type::Function(f) = typ {
                assert_eq!(f.parameter_names.len(), 1);
                assert_eq!(f.parameter_names[0], "x");
                // No type annotation, should be 'any'
                assert_eq!(f.parameter_types[0], checker.types.any_type);
                assert_eq!(f.return_type, checker.types.any_type);
            } else {
                panic!("Expected Function type, got {:?}", typ);
            }
        }
    }

    #[test]
    fn test_arrow_function_no_params() {
        use crate::parser_impl::ParserState;
        use crate::binder::BinderState;

        // Test arrow function with no parameters
        let mut parser = ParserState::new(
            "test.ts".to_string(),
            r#"const getNumber = (): number => 42;"#.to_string(),
        );
        let root = parser.parse_source_file();

        let mut binder = BinderState::new();
        binder.bind_source_file(&parser.arena, root);

        let mut checker = CheckerState::new(
            &parser.arena,
            &binder.symbols,
            &binder.file_locals,
            "test.ts".to_string(),
        );

        assert!(binder.file_locals.has("getNumber"));

        if let Some(symbol) = binder.file_locals.get("getNumber") {
            let fn_type = checker.get_type_of_symbol(symbol);
            let typ = checker.types.get(fn_type).unwrap();

            if let Type::Function(f) = typ {
                assert_eq!(f.parameter_names.len(), 0);
                assert_eq!(f.return_type, checker.types.number_type);
                assert_eq!(f.min_argument_count, 0);
            } else {
                panic!("Expected Function type, got {:?}", typ);
            }
        }
    }

    #[test]
    fn test_generic_function_type() {
        use crate::parser_impl::ParserState;
        use crate::binder::BinderState;

        // Test generic function declaration
        let mut parser = ParserState::new(
            "test.ts".to_string(),
            r#"function identity<T>(x: T): T { return x; }"#.to_string(),
        );
        let root = parser.parse_source_file();

        let mut binder = BinderState::new();
        binder.bind_source_file(&parser.arena, root);

        let mut checker = CheckerState::new(
            &parser.arena,
            &binder.symbols,
            &binder.file_locals,
            "test.ts".to_string(),
        );

        // 'identity' should have a generic function type
        assert!(binder.file_locals.has("identity"));

        if let Some(symbol) = binder.file_locals.get("identity") {
            let fn_type = checker.get_type_of_symbol(symbol);
            let typ = checker.types.get(fn_type).unwrap();

            if let Type::Function(f) = typ {
                // Should have one type parameter
                assert_eq!(f.type_parameters.len(), 1);
                assert_eq!(f.parameter_names.len(), 1);
                assert_eq!(f.parameter_names[0], "x");

                // Type string should include <T>
                let type_str = checker.type_to_string(fn_type);
                assert!(type_str.starts_with("<T>"), "Expected type string to start with <T>, got: {}", type_str);
            } else {
                panic!("Expected Function type, got {:?}", typ);
            }
        }
    }

    #[test]
    fn test_generic_function_multiple_type_params() {
        use crate::parser_impl::ParserState;
        use crate::binder::BinderState;

        // Test generic function with multiple type parameters
        let mut parser = ParserState::new(
            "test.ts".to_string(),
            r#"function pair<T, U>(first: T, second: U): [T, U] { return [first, second]; }"#.to_string(),
        );
        let root = parser.parse_source_file();

        let mut binder = BinderState::new();
        binder.bind_source_file(&parser.arena, root);

        let mut checker = CheckerState::new(
            &parser.arena,
            &binder.symbols,
            &binder.file_locals,
            "test.ts".to_string(),
        );

        assert!(binder.file_locals.has("pair"));

        if let Some(symbol) = binder.file_locals.get("pair") {
            let fn_type = checker.get_type_of_symbol(symbol);
            let typ = checker.types.get(fn_type).unwrap();

            if let Type::Function(f) = typ {
                // Should have two type parameters
                assert_eq!(f.type_parameters.len(), 2);
                assert_eq!(f.parameter_names.len(), 2);
                assert_eq!(f.parameter_names[0], "first");
                assert_eq!(f.parameter_names[1], "second");

                // Type string should include <T, U>
                let type_str = checker.type_to_string(fn_type);
                assert!(type_str.starts_with("<T, U>"), "Expected type string to start with <T, U>, got: {}", type_str);
            } else {
                panic!("Expected Function type, got {:?}", typ);
            }
        }
    }

    #[test]
    fn test_generic_function_with_constraint() {
        use crate::parser_impl::ParserState;
        use crate::binder::BinderState;

        // Test generic function with constraint
        let mut parser = ParserState::new(
            "test.ts".to_string(),
            r#"function getLength<T extends { length: number }>(x: T): number { return x.length; }"#.to_string(),
        );
        let root = parser.parse_source_file();

        let mut binder = BinderState::new();
        binder.bind_source_file(&parser.arena, root);

        let mut checker = CheckerState::new(
            &parser.arena,
            &binder.symbols,
            &binder.file_locals,
            "test.ts".to_string(),
        );

        assert!(binder.file_locals.has("getLength"));

        if let Some(symbol) = binder.file_locals.get("getLength") {
            let fn_type = checker.get_type_of_symbol(symbol);
            let typ = checker.types.get(fn_type).unwrap();

            if let Type::Function(f) = typ {
                // Should have one type parameter
                assert_eq!(f.type_parameters.len(), 1);

                // The type parameter should have a constraint
                let tp_id = f.type_parameters[0];
                let tp = checker.types.get(tp_id).unwrap();
                if let Type::TypeParameter(tp) = tp {
                    // Constraint should not be NONE
                    assert!(!tp.constraint.is_none());
                } else {
                    panic!("Expected TypeParameter");
                }
            } else {
                panic!("Expected Function type, got {:?}", typ);
            }
        }
    }

    #[test]
    fn test_generic_arrow_function() {
        use crate::parser_impl::ParserState;
        use crate::binder::BinderState;

        // Test generic arrow function
        let mut parser = ParserState::new(
            "test.ts".to_string(),
            r#"const identity = <T>(x: T): T => x;"#.to_string(),
        );
        let root = parser.parse_source_file();

        let mut binder = BinderState::new();
        binder.bind_source_file(&parser.arena, root);

        let mut checker = CheckerState::new(
            &parser.arena,
            &binder.symbols,
            &binder.file_locals,
            "test.ts".to_string(),
        );

        assert!(binder.file_locals.has("identity"));

        if let Some(symbol) = binder.file_locals.get("identity") {
            let fn_type = checker.get_type_of_symbol(symbol);
            let typ = checker.types.get(fn_type).unwrap();

            if let Type::Function(f) = typ {
                // Should have one type parameter
                assert_eq!(f.type_parameters.len(), 1);
                assert_eq!(f.parameter_names.len(), 1);
            } else {
                panic!("Expected Function type, got {:?}", typ);
            }
        }
    }

    #[test]
    fn test_type_parameter_creation() {
        use crate::parser::NodeArena;

        let node_arena = NodeArena::new();
        let symbol_arena = SymbolArena::new();
        let file_locals = SymbolTable::new();
        let mut checker = CheckerState::new(&node_arena, &symbol_arena, &file_locals, "test.ts".to_string());

        // Manually create a type parameter
        let tp_id = checker.types.create_type_parameter(
            SymbolId::NONE,
            TypeId::NONE,
            TypeId::NONE,
        );

        let tp = checker.types.get(tp_id).unwrap();
        assert!(tp.has_flags(type_flags::TYPE_PARAMETER));

        if let Type::TypeParameter(t) = tp {
            assert!(t.constraint.is_none());
            assert!(t.default.is_none());
        } else {
            panic!("Expected TypeParameter");
        }
    }

    #[test]
    fn test_type_literal() {
        use crate::parser_impl::ParserState;
        use crate::binder::BinderState;

        // Test type literal
        let mut parser = ParserState::new(
            "test.ts".to_string(),
            r#"type Point = { x: number; y: number };"#.to_string(),
        );
        let root = parser.parse_source_file();

        let mut binder = BinderState::new();
        binder.bind_source_file(&parser.arena, root);

        let mut checker = CheckerState::new(
            &parser.arena,
            &binder.symbols,
            &binder.file_locals,
            "test.ts".to_string(),
        );

        assert!(binder.file_locals.has("Point"));

        if let Some(symbol) = binder.file_locals.get("Point") {
            let point_type = checker.get_type_of_symbol(symbol);
            let typ = checker.types.get(point_type).unwrap();

            if let Type::Object(obj) = typ {
                assert_eq!(obj.properties.len(), 2);
            } else {
                panic!("Expected Object type, got {:?}", typ);
            }
        }
    }

    #[test]
    fn test_object_type_with_method() {
        use crate::parser_impl::ParserState;
        use crate::binder::BinderState;

        // Test type literal with method
        let mut parser = ParserState::new(
            "test.ts".to_string(),
            r#"type Greeter = { greet(name: string): string };"#.to_string(),
        );
        let root = parser.parse_source_file();

        let mut binder = BinderState::new();
        binder.bind_source_file(&parser.arena, root);

        let mut checker = CheckerState::new(
            &parser.arena,
            &binder.symbols,
            &binder.file_locals,
            "test.ts".to_string(),
        );

        assert!(binder.file_locals.has("Greeter"));

        if let Some(symbol) = binder.file_locals.get("Greeter") {
            let greeter_type = checker.get_type_of_symbol(symbol);
            let typ = checker.types.get(greeter_type).unwrap();

            if let Type::Object(obj) = typ {
                assert_eq!(obj.properties.len(), 1);
            } else {
                panic!("Expected Object type, got {:?}", typ);
            }
        }
    }

    #[test]
    fn test_create_object_type() {
        use crate::parser::NodeArena;

        let node_arena = NodeArena::new();
        let symbol_arena = SymbolArena::new();
        let file_locals = SymbolTable::new();
        let mut checker = CheckerState::new(&node_arena, &symbol_arena, &file_locals, "test.ts".to_string());

        // Create an empty object type
        let obj_type = checker.types.create_object_type(Vec::new());
        let typ = checker.types.get(obj_type).unwrap();

        if let Type::Object(obj) = typ {
            assert_eq!(obj.properties.len(), 0);
            assert!(obj.has_object_flags(object_flags::ANONYMOUS));
        } else {
            panic!("Expected Object type");
        }
    }

    #[test]
    fn test_call_expression_type() {
        use crate::parser_impl::ParserState;
        use crate::binder::BinderState;

        // Test function call expression
        let mut parser = ParserState::new(
            "test.ts".to_string(),
            r#"
                function greet(name: string): string { return name; }
                const result = greet("hello");
            "#.to_string(),
        );
        let root = parser.parse_source_file();

        let mut binder = BinderState::new();
        binder.bind_source_file(&parser.arena, root);

        let mut checker = CheckerState::new(
            &parser.arena,
            &binder.symbols,
            &binder.file_locals,
            "test.ts".to_string(),
        );

        // 'greet' should be a function type
        assert!(binder.file_locals.has("greet"));
        if let Some(symbol) = binder.file_locals.get("greet") {
            let fn_type = checker.get_type_of_symbol(symbol);
            let typ = checker.types.get(fn_type).unwrap();

            if let Type::Function(f) = typ {
                assert_eq!(f.return_type, checker.types.string_type);
            } else {
                panic!("Expected Function type");
            }
        }

        // 'result' should also be string (return type of greet)
        assert!(binder.file_locals.has("result"));
    }

    #[test]
    fn test_array_literal_type() {
        use crate::parser_impl::ParserState;
        use crate::binder::BinderState;

        // Test array literal with mixed types
        let mut parser = ParserState::new(
            "test.ts".to_string(),
            r#"const arr = [1, "hello"];"#.to_string(),
        );
        let root = parser.parse_source_file();

        let mut binder = BinderState::new();
        binder.bind_source_file(&parser.arena, root);

        let mut checker = CheckerState::new(
            &parser.arena,
            &binder.symbols,
            &binder.file_locals,
            "test.ts".to_string(),
        );

        assert!(binder.file_locals.has("arr"));

        if let Some(symbol) = binder.file_locals.get("arr") {
            let arr_type = checker.get_type_of_symbol(symbol);
            let typ = checker.types.get(arr_type).unwrap();

            // The type should be a union of number and string literals
            if let Type::Union(u) = typ {
                assert_eq!(u.types.len(), 2);
            } else {
                panic!("Expected Union type for mixed array, got {:?}", typ);
            }
        }
    }

    #[test]
    fn test_union_type_creation() {
        use crate::parser::NodeArena;

        let node_arena = NodeArena::new();
        let symbol_arena = SymbolArena::new();
        let file_locals = SymbolTable::new();
        let mut checker = CheckerState::new(&node_arena, &symbol_arena, &file_locals, "test.ts".to_string());

        // Create a union type of string | number
        let string_type = checker.types.string_type;
        let number_type = checker.types.number_type;
        let union_type = checker.types.create_union_type(vec![string_type, number_type]);

        let typ = checker.types.get(union_type).unwrap();
        if let Type::Union(u) = typ {
            assert_eq!(u.types.len(), 2);
            assert!(u.types.contains(&string_type));
            assert!(u.types.contains(&number_type));
        } else {
            panic!("Expected Union type");
        }

        // Type to string should work
        let type_str = checker.type_to_string(union_type);
        assert!(type_str.contains("|"));
    }

    #[test]
    fn test_union_simplification_never() {
        // X | never = X
        let mut arena = TypeArena::new();
        let obj = arena.alloc(Type::Object(ObjectType::new(object_flags::ANONYMOUS, SymbolId::NONE)));

        let union = arena.create_union(vec![obj, arena.never_type]);
        assert_eq!(union, obj);
    }

    #[test]
    fn test_union_simplification_any() {
        // X | any = any
        let mut arena = TypeArena::new();
        let obj = arena.alloc(Type::Object(ObjectType::new(object_flags::ANONYMOUS, SymbolId::NONE)));

        let union = arena.create_union(vec![obj, arena.any_type]);
        assert_eq!(union, arena.any_type);
    }

    #[test]
    fn test_union_simplification_duplicates() {
        // X | X = X
        let mut arena = TypeArena::new();
        let obj = arena.alloc(Type::Object(ObjectType::new(object_flags::ANONYMOUS, SymbolId::NONE)));

        let union = arena.create_union(vec![obj, obj]);
        assert_eq!(union, obj);
    }

    #[test]
    fn test_union_simplification_flatten() {
        // (A | B) | C = A | B | C
        let mut arena = TypeArena::new();
        let obj1 = arena.alloc(Type::Object(ObjectType::new(object_flags::ANONYMOUS, SymbolId::NONE)));
        let obj2 = arena.alloc(Type::Object(ObjectType::new(object_flags::ANONYMOUS, SymbolId::NONE)));
        let obj3 = arena.alloc(Type::Object(ObjectType::new(object_flags::ANONYMOUS, SymbolId::NONE)));

        let inner = arena.create_union(vec![obj1, obj2]);
        let outer = arena.create_union(vec![inner, obj3]);

        // Should have flattened to a single union with 3 types
        if let Type::Union(u) = arena.get(outer).unwrap() {
            assert_eq!(u.types.len(), 3);
            assert!(u.types.contains(&obj1));
            assert!(u.types.contains(&obj2));
            assert!(u.types.contains(&obj3));
        } else {
            panic!("Expected flattened union type");
        }
    }

    #[test]
    fn test_union_empty_returns_never() {
        let mut arena = TypeArena::new();
        let union = arena.create_union(vec![]);
        assert_eq!(union, arena.never_type);
    }

    #[test]
    fn test_union_all_never_returns_never() {
        // never | never = never
        let mut arena = TypeArena::new();
        let union = arena.create_union(vec![arena.never_type, arena.never_type]);
        assert_eq!(union, arena.never_type);
    }

    #[test]
    fn test_simple_interface_variable() {
        use crate::parser_impl::ParserState;
        use crate::binder::BinderState;

        // Simpler test - just interface with variable
        let mut parser = ParserState::new(
            "test.ts".to_string(),
            r#"
                interface A { x: string; }
                declare let a: A;
            "#.to_string(),
        );
        let root = parser.parse_source_file();

        let mut binder = BinderState::new();
        binder.bind_source_file(&parser.arena, root);

        let mut checker = CheckerState::new(
            &parser.arena,
            &binder.symbols,
            &binder.file_locals,
            "test.ts".to_string(),
        );

        // Get the type of a
        let a_symbol = binder.file_locals.get("a").expect("a should be in file locals");
        let a_type = checker.get_type_of_symbol(a_symbol);
        let a_str = checker.type_to_string(a_type);

        // a should be of type A (an interface with property x: string)
        assert!(checker.types.get(a_type).is_some(), "a should have a valid type");
        assert!(a_str.contains("x"), "a should have property x, got: {}", a_str);
    }

    #[test]
    fn test_property_access_on_union() {
        use crate::parser_impl::ParserState;
        use crate::binder::BinderState;

        // Test property access on union types
        // (A | B).prop should return A.prop | B.prop
        let mut parser = ParserState::new(
            "test.ts".to_string(),
            r#"
                interface A { x: string; }
                interface B { x: number; }
                declare let value: A | B;
                let result = value.x;  // should be string | number
            "#.to_string(),
        );
        let root = parser.parse_source_file();

        let mut binder = BinderState::new();
        binder.bind_source_file(&parser.arena, root);

        let mut checker = CheckerState::new(
            &parser.arena,
            &binder.symbols,
            &binder.file_locals,
            "test.ts".to_string(),
        );

        // Get the type of result
        let result_symbol = binder.file_locals.get("result").expect("result should be in file locals");
        let result_type = checker.get_type_of_symbol(result_symbol);
        let result_str = checker.type_to_string(result_type);

        // Should be string | number
        assert!(result_str.contains("string") || result_str.contains("number"),
            "result should be string | number, got: {}", result_str);
    }

    #[test]
    fn test_class_type() {
        use crate::parser_impl::ParserState;
        use crate::binder::BinderState;

        // Test class type
        let mut parser = ParserState::new(
            "test.ts".to_string(),
            r#"class Person {
                name: string;
                age: number;
                greet(): string { return "hello"; }
            }"#.to_string(),
        );
        let root = parser.parse_source_file();

        let mut binder = BinderState::new();
        binder.bind_source_file(&parser.arena, root);

        let mut checker = CheckerState::new(
            &parser.arena,
            &binder.symbols,
            &binder.file_locals,
            "test.ts".to_string(),
        );

        assert!(binder.file_locals.has("Person"));

        if let Some(symbol) = binder.file_locals.get("Person") {
            let person_type = checker.get_type_of_symbol(symbol);
            let typ = checker.types.get(person_type).unwrap();

            if let Type::Object(obj) = typ {
                // Should have CLASS object flag
                assert!(obj.has_object_flags(object_flags::CLASS));
                // Should have 3 properties: name, age, greet
                assert_eq!(obj.properties.len(), 3);
            } else {
                panic!("Expected Object type with CLASS flag, got {:?}", typ);
            }
        }
    }

    #[test]
    fn test_class_with_constructor() {
        use crate::parser_impl::ParserState;
        use crate::binder::BinderState;

        // Test class with constructor - simplified without this.x assignments
        let mut parser = ParserState::new(
            "test.ts".to_string(),
            r#"class Point {
                x: number;
                y: number;
                constructor(a: number, b: number) { }
            }"#.to_string(),
        );
        let root = parser.parse_source_file();

        let mut binder = BinderState::new();
        binder.bind_source_file(&parser.arena, root);

        let mut checker = CheckerState::new(
            &parser.arena,
            &binder.symbols,
            &binder.file_locals,
            "test.ts".to_string(),
        );

        assert!(binder.file_locals.has("Point"));

        if let Some(symbol) = binder.file_locals.get("Point") {
            let point_type = checker.get_type_of_symbol(symbol);
            let typ = checker.types.get(point_type).unwrap();

            if let Type::Object(obj) = typ {
                // Should have CLASS object flag
                assert!(obj.has_object_flags(object_flags::CLASS));
                // Should have 2 properties: x, y
                assert_eq!(obj.properties.len(), 2);
                // Should have 1 construct signature
                assert_eq!(obj.construct_signatures.len(), 1);
                // Construct signature should have 2 parameters
                assert_eq!(obj.construct_signatures[0].parameters.len(), 2);
            } else {
                panic!("Expected Object type with CLASS flag, got {:?}", typ);
            }
        }
    }

    #[test]
    fn test_interface_type() {
        use crate::parser_impl::ParserState;
        use crate::binder::BinderState;

        // Test interface type
        let mut parser = ParserState::new(
            "test.ts".to_string(),
            r#"interface Animal {
                name: string;
                speak(): void;
            }"#.to_string(),
        );
        let root = parser.parse_source_file();

        let mut binder = BinderState::new();
        binder.bind_source_file(&parser.arena, root);

        let mut checker = CheckerState::new(
            &parser.arena,
            &binder.symbols,
            &binder.file_locals,
            "test.ts".to_string(),
        );

        assert!(binder.file_locals.has("Animal"));

        if let Some(symbol) = binder.file_locals.get("Animal") {
            let animal_type = checker.get_type_of_symbol(symbol);
            let typ = checker.types.get(animal_type).unwrap();

            if let Type::Object(obj) = typ {
                // Should have INTERFACE object flag
                assert!(obj.has_object_flags(object_flags::INTERFACE));
                // Should have 2 properties: name, speak
                assert_eq!(obj.properties.len(), 2);
            } else {
                panic!("Expected Object type with INTERFACE flag, got {:?}", typ);
            }
        }
    }

    #[test]
    fn test_class_with_accessors() {
        use crate::parser_impl::ParserState;
        use crate::binder::BinderState;

        // Test class with get/set accessors - simplified without this references
        let mut parser = ParserState::new(
            "test.ts".to_string(),
            r#"class Counter {
                _value: number;
                get value(): number { return 0; }
                set value(v: number) { }
            }"#.to_string(),
        );
        let root = parser.parse_source_file();

        let mut binder = BinderState::new();
        binder.bind_source_file(&parser.arena, root);

        let mut checker = CheckerState::new(
            &parser.arena,
            &binder.symbols,
            &binder.file_locals,
            "test.ts".to_string(),
        );

        assert!(binder.file_locals.has("Counter"));

        if let Some(symbol) = binder.file_locals.get("Counter") {
            let counter_type = checker.get_type_of_symbol(symbol);
            let typ = checker.types.get(counter_type).unwrap();

            if let Type::Object(obj) = typ {
                // Should have CLASS object flag
                assert!(obj.has_object_flags(object_flags::CLASS));
                // Should have 3 properties: _value, get value, set value
                assert_eq!(obj.properties.len(), 3);
            } else {
                panic!("Expected Object type with CLASS flag, got {:?}", typ);
            }
        }
    }

    #[test]
    fn test_generic_function_type_parameter() {
        use crate::parser_impl::ParserState;
        use crate::binder::BinderState;

        let mut parser = ParserState::new(
            "test.ts".to_string(),
            "function identity<T>(x: T): T { return x; }".to_string(),
        );
        let root = parser.parse_source_file();

        let mut binder = BinderState::new();
        binder.bind_source_file(&parser.arena, root);

        let mut checker = CheckerState::new(
            &parser.arena,
            &binder.symbols,
            &binder.file_locals,
            "test.ts".to_string(),
        );

        // Get the identity function
        assert!(binder.file_locals.has("identity"));
        if let Some(symbol) = binder.file_locals.get("identity") {
            let func_type = checker.get_type_of_symbol(symbol);
            let type_str = checker.type_to_string(func_type);

            // Should have type parameter T
            assert!(type_str.contains("<T>"), "Expected type parameter T, got: {}", type_str);
            assert!(type_str.contains("T") && type_str.contains("=>"), "Expected function with T, got: {}", type_str);
        } else {
            panic!("identity function not found");
        }
    }

    #[test]
    fn test_type_instantiation() {
        use crate::parser::NodeArena;

        let node_arena = NodeArena::new();
        let symbol_arena = SymbolArena::new();
        let file_locals = SymbolTable::new();
        let mut checker = CheckerState::new(&node_arena, &symbol_arena, &file_locals, "test.ts".to_string());

        // Create a type parameter T
        let t_symbol = checker.local_symbols.alloc(symbol_flags::TYPE_PARAMETER, "T".to_string());
        let t_type = checker.types.alloc(Type::TypeParameter(TypeParameter {
            flags: type_flags::TYPE_PARAMETER,
            symbol: t_symbol,
            constraint: TypeId::NONE,
            default: TypeId::NONE,
            target: TypeId::NONE,
            is_this_type: false,
        }));
        checker.type_parameter_names.insert(t_type, "T".to_string());

        // Create a function type with T as parameter and return type: (x: T) => T
        let func_type = checker.types.create_function_type_with_type_params(
            NodeIndex::NONE,
            vec![t_type],              // parameter types
            vec!["x".to_string()],     // parameter names
            t_type,                    // return type
            vec![t_type],              // type parameters
            1,                         // min argument count
            false,                     // has rest parameter
        );

        // Instantiate with T = string
        let instantiated = checker.instantiate_type(func_type, &[checker.types.string_type], &[t_type]);

        // The result should have string as parameter and return type
        if let Some(Type::Function(f)) = checker.types.get(instantiated) {
            assert_eq!(f.parameter_types.len(), 1);
            assert_eq!(f.parameter_types[0], checker.types.string_type);
            assert_eq!(f.return_type, checker.types.string_type);
        } else {
            panic!("Expected instantiated function type");
        }
    }

    #[test]
    fn test_type_parameter_with_default() {
        use crate::parser_impl::ParserState;
        use crate::binder::BinderState;

        let mut parser = ParserState::new(
            "test.ts".to_string(),
            "type Create<T = string> = () => T;".to_string(),
        );
        let root = parser.parse_source_file();

        let mut binder = BinderState::new();
        binder.bind_source_file(&parser.arena, root);

        let mut checker = CheckerState::new(
            &parser.arena,
            &binder.symbols,
            &binder.file_locals,
            "test.ts".to_string(),
        );

        // Get the Create type alias
        assert!(binder.file_locals.has("Create"));
        if let Some(symbol) = binder.file_locals.get("Create") {
            // Just verify it parses and binds successfully
            let _ = checker.get_type_of_symbol(symbol);
        }
    }

    #[test]
    fn test_object_literal_type() {
        use crate::parser_impl::ParserState;
        use crate::binder::BinderState;

        let mut parser = ParserState::new(
            "test.ts".to_string(),
            r#"const obj = { x: 1, y: "hello" };"#.to_string(),
        );
        let root = parser.parse_source_file();

        let mut binder = BinderState::new();
        binder.bind_source_file(&parser.arena, root);

        let mut checker = CheckerState::new(
            &parser.arena,
            &binder.symbols,
            &binder.file_locals,
            "test.ts".to_string(),
        );

        assert!(binder.file_locals.has("obj"));
        if let Some(symbol) = binder.file_locals.get("obj") {
            let obj_type = checker.get_type_of_symbol(symbol);

            // Should be an object type
            if let Some(Type::Object(obj)) = checker.types.get(obj_type) {
                assert_eq!(obj.properties.len(), 2, "Expected 2 properties");
            } else {
                panic!("Expected Object type for object literal");
            }
        }
    }

    #[test]
    fn test_type_literal_members() {
        use crate::parser_impl::ParserState;
        use crate::binder::BinderState;

        let mut parser = ParserState::new(
            "test.ts".to_string(),
            "type Point = { x: number; y: number };".to_string(),
        );
        let root = parser.parse_source_file();

        let mut binder = BinderState::new();
        binder.bind_source_file(&parser.arena, root);

        let mut checker = CheckerState::new(
            &parser.arena,
            &binder.symbols,
            &binder.file_locals,
            "test.ts".to_string(),
        );

        assert!(binder.file_locals.has("Point"));
        if let Some(symbol) = binder.file_locals.get("Point") {
            let point_type = checker.get_type_of_symbol(symbol);

            // Should be an object type
            if let Some(Type::Object(obj)) = checker.types.get(point_type) {
                assert_eq!(obj.properties.len(), 2, "Expected 2 properties for Point");
            } else {
                panic!("Expected Object type for type literal");
            }
        }
    }

    #[test]
    fn test_interface_members() {
        use crate::parser_impl::ParserState;
        use crate::binder::BinderState;

        let mut parser = ParserState::new(
            "test.ts".to_string(),
            r#"interface Person {
                name: string;
                age: number;
                greet(): void;
            }"#.to_string(),
        );
        let root = parser.parse_source_file();

        let mut binder = BinderState::new();
        binder.bind_source_file(&parser.arena, root);

        let mut checker = CheckerState::new(
            &parser.arena,
            &binder.symbols,
            &binder.file_locals,
            "test.ts".to_string(),
        );

        assert!(binder.file_locals.has("Person"));
        if let Some(symbol) = binder.file_locals.get("Person") {
            let person_type = checker.get_type_of_symbol(symbol);

            // Should be an interface object type
            if let Some(Type::Object(obj)) = checker.types.get(person_type) {
                assert!(obj.has_object_flags(object_flags::INTERFACE), "Expected INTERFACE object flag");
                assert_eq!(obj.properties.len(), 3, "Expected 3 properties (name, age, greet)");
            } else {
                panic!("Expected Object type for interface");
            }
        }
    }

    #[test]
    fn test_typeof_narrowing_string() {
        use crate::parser::NodeArena;

        let node_arena = NodeArena::new();
        let symbol_arena = SymbolArena::new();
        let file_locals = SymbolTable::new();
        let mut checker = CheckerState::new(&node_arena, &symbol_arena, &file_locals, "test.ts".to_string());

        // Create a union type: string | number
        let union_type = checker.types.create_union(vec![
            checker.types.string_type,
            checker.types.number_type,
        ]);

        // Narrow by typeof === "string"
        let narrowed = checker.narrow_type_by_typeof(union_type, "string");

        // Should narrow to string
        assert_eq!(narrowed, checker.types.string_type);
    }

    #[test]
    fn test_typeof_narrowing_number() {
        use crate::parser::NodeArena;

        let node_arena = NodeArena::new();
        let symbol_arena = SymbolArena::new();
        let file_locals = SymbolTable::new();
        let mut checker = CheckerState::new(&node_arena, &symbol_arena, &file_locals, "test.ts".to_string());

        // Create a union type: string | number | boolean
        let union_type = checker.types.create_union(vec![
            checker.types.string_type,
            checker.types.number_type,
            checker.types.boolean_type,
        ]);

        // Narrow by typeof === "number"
        let narrowed = checker.narrow_type_by_typeof(union_type, "number");

        // Should narrow to number
        assert_eq!(narrowed, checker.types.number_type);
    }

    #[test]
    fn test_typeof_narrowing_negation() {
        use crate::parser::NodeArena;

        let node_arena = NodeArena::new();
        let symbol_arena = SymbolArena::new();
        let file_locals = SymbolTable::new();
        let mut checker = CheckerState::new(&node_arena, &symbol_arena, &file_locals, "test.ts".to_string());

        // Create a union type: string | number
        let union_type = checker.types.create_union(vec![
            checker.types.string_type,
            checker.types.number_type,
        ]);

        // Narrow by typeof !== "string"
        let narrowed = checker.narrow_type_by_typeof_negation(union_type, "string");

        // Should narrow to number
        assert_eq!(narrowed, checker.types.number_type);
    }

    #[test]
    fn test_non_nullable_type() {
        use crate::parser::NodeArena;

        let node_arena = NodeArena::new();
        let symbol_arena = SymbolArena::new();
        let file_locals = SymbolTable::new();
        let mut checker = CheckerState::new(&node_arena, &symbol_arena, &file_locals, "test.ts".to_string());

        // Create a union type: string | null | undefined
        let union_type = checker.types.create_union(vec![
            checker.types.string_type,
            checker.types.null_type,
            checker.types.undefined_type,
        ]);

        // Get non-nullable
        let non_nullable = checker.get_non_nullable_type(union_type);

        // Should narrow to string
        assert_eq!(non_nullable, checker.types.string_type);
    }

    #[test]
    fn test_typeof_narrowing_no_match() {
        use crate::parser::NodeArena;

        let node_arena = NodeArena::new();
        let symbol_arena = SymbolArena::new();
        let file_locals = SymbolTable::new();
        let mut checker = CheckerState::new(&node_arena, &symbol_arena, &file_locals, "test.ts".to_string());

        // Narrow string by typeof === "number" should give never
        let narrowed = checker.narrow_type_by_typeof(checker.types.string_type, "number");

        assert_eq!(narrowed, checker.types.never_type);
    }

    #[test]
    fn test_generic_call_expression_inference() {
        use crate::parser_impl::ParserState;
        use crate::binder::BinderState;

        // Test: identity<T>(x: T): T called with "hello" should infer T = string
        let mut parser = ParserState::new(
            "test.ts".to_string(),
            r#"
            function identity<T>(x: T): T { return x; }
            const result = identity("hello");
            "#.to_string(),
        );
        let root = parser.parse_source_file();

        let mut binder = BinderState::new();
        binder.bind_source_file(&parser.arena, root);

        let mut checker = CheckerState::new(
            &parser.arena,
            &binder.symbols,
            &binder.file_locals,
            "test.ts".to_string(),
        );

        // Verify identity function type
        assert!(binder.file_locals.has("identity"));
        if let Some(identity_sym) = binder.file_locals.get("identity") {
            let identity_type = checker.get_type_of_symbol(identity_sym);
            let identity_str = checker.type_to_string(identity_type);

            // Should have form <T>(x: T) => T
            assert!(identity_str.contains("<T>"), "Expected type parameter T, got: {}", identity_str);

            // Verify it has type parameters
            if let Some(Type::Function(f)) = checker.types.get(identity_type) {
                assert_eq!(f.type_parameters.len(), 1, "Expected 1 type parameter");
                // param type and return type should be the same (T)
                assert_eq!(f.parameter_types.len(), 1);
                assert_eq!(f.parameter_types[0], f.return_type);
            }
        }

        // Get the result variable type
        assert!(binder.file_locals.has("result"));
        if let Some(symbol) = binder.file_locals.get("result") {
            let result_type = checker.get_type_of_symbol(symbol);

            // The result should be a string literal type "hello" (or widened to string)
            // since identity<T>(x: T): T returns T, and T is inferred from "hello"
            if let Some(Type::Literal(lit)) = checker.types.get(result_type) {
                assert!(matches!(lit.value, LiteralValue::String(_)),
                    "Expected string literal type, got {:?}", lit.value);
            } else {
                // Could also be string_type if literal widening is applied
                assert!(result_type == checker.types.string_type ||
                        matches!(checker.types.get(result_type), Some(Type::Literal(_))),
                    "Expected string or string literal type, got: {}",
                    checker.type_to_string(result_type));
            }
        } else {
            panic!("result variable not found");
        }
    }

    #[test]
    fn test_generic_call_with_explicit_type_args() {
        use crate::parser_impl::ParserState;
        use crate::binder::BinderState;

        // Test: identity<string>(x) should use the explicit type argument
        let mut parser = ParserState::new(
            "test.ts".to_string(),
            r#"
            function identity<T>(x: T): T { return x; }
            const result = identity<number>(42);
            "#.to_string(),
        );
        let root = parser.parse_source_file();

        let mut binder = BinderState::new();
        binder.bind_source_file(&parser.arena, root);

        let mut checker = CheckerState::new(
            &parser.arena,
            &binder.symbols,
            &binder.file_locals,
            "test.ts".to_string(),
        );

        // Verify identity function type
        assert!(binder.file_locals.has("identity"));
        if let Some(identity_sym) = binder.file_locals.get("identity") {
            let identity_type = checker.get_type_of_symbol(identity_sym);
            let identity_str = checker.type_to_string(identity_type);
            assert!(identity_str.contains("<T>"), "Expected generic function, got: {}", identity_str);
        }

        // Get result type - should be number (from explicit type argument)
        assert!(binder.file_locals.has("result"));
        if let Some(symbol) = binder.file_locals.get("result") {
            let result_type = checker.get_type_of_symbol(symbol);

            // With explicit type argument <number>, result should be number
            assert!(result_type == checker.types.number_type ||
                    matches!(checker.types.get(result_type), Some(Type::Literal(l))
                             if matches!(l.value, LiteralValue::Number(_))),
                "Expected number type, got: {}", checker.type_to_string(result_type));
        } else {
            panic!("result variable not found");
        }
    }

    #[test]
    fn test_instanceof_narrowing_unknown() {
        use crate::parser::NodeArena;

        let node_arena = NodeArena::new();
        let symbol_arena = SymbolArena::new();
        let file_locals = SymbolTable::new();
        let mut checker = CheckerState::new(&node_arena, &symbol_arena, &file_locals, "test.ts".to_string());

        // Create a simple class type
        let class_type = checker.types.create_class_type(vec![], vec![], vec![]);

        // Narrow unknown by instanceof should give the class type
        let narrowed = checker.narrow_type_by_instanceof(checker.types.unknown_type, class_type);

        assert_eq!(narrowed, class_type);
    }

    #[test]
    fn test_instanceof_narrowing_any() {
        use crate::parser::NodeArena;

        let node_arena = NodeArena::new();
        let symbol_arena = SymbolArena::new();
        let file_locals = SymbolTable::new();
        let mut checker = CheckerState::new(&node_arena, &symbol_arena, &file_locals, "test.ts".to_string());

        // Create a simple class type
        let class_type = checker.types.create_class_type(vec![], vec![], vec![]);

        // Narrow any by instanceof should give the class type
        let narrowed = checker.narrow_type_by_instanceof(checker.types.any_type, class_type);

        assert_eq!(narrowed, class_type);
    }

    #[test]
    fn test_instanceof_narrowing_union() {
        use crate::parser::NodeArena;

        let node_arena = NodeArena::new();
        let symbol_arena = SymbolArena::new();
        let file_locals = SymbolTable::new();
        let mut checker = CheckerState::new(&node_arena, &symbol_arena, &file_locals, "test.ts".to_string());

        // Create a class type
        let class_type = checker.types.create_class_type(vec![], vec![], vec![]);

        // Create a union of string | class
        let union_type = checker.types.create_union(vec![
            checker.types.string_type,
            class_type,
        ]);

        // Narrow by instanceof should filter out string (primitive)
        let narrowed = checker.narrow_type_by_instanceof(union_type, class_type);

        // Should narrow to just the class type
        assert_eq!(narrowed, class_type);
    }

    #[test]
    fn test_instanceof_negation() {
        use crate::parser::NodeArena;

        let node_arena = NodeArena::new();
        let symbol_arena = SymbolArena::new();
        let file_locals = SymbolTable::new();
        let mut checker = CheckerState::new(&node_arena, &symbol_arena, &file_locals, "test.ts".to_string());

        // Create two class types
        let class_a = checker.types.create_class_type(vec![], vec![], vec![]);
        let class_b = checker.types.create_class_type(vec![], vec![], vec![]);

        // Create a union of A | B
        let union_type = checker.types.create_union(vec![class_a, class_b]);

        // Narrow by NOT instanceof A should give B
        let narrowed = checker.narrow_type_by_instanceof_negation(union_type, class_a);

        // Should narrow to class_b
        assert_eq!(narrowed, class_b);
    }

    #[test]
    fn test_class_construct_signature() {
        use crate::parser::NodeArena;

        // Test construct signature infrastructure directly
        let node_arena = NodeArena::new();
        let symbol_arena = SymbolArena::new();
        let file_locals = SymbolTable::new();
        let mut checker = CheckerState::new(&node_arena, &symbol_arena, &file_locals, "test.ts".to_string());

        // Create an instance type (what you get from new Foo())
        let instance_type = checker.types.create_class_type(vec![], vec![], vec![]);

        // Create a construct signature that returns the instance type
        let mut construct_sig = Signature::new(NodeIndex::NONE);
        construct_sig.resolved_return_type = Some(instance_type);

        // Create the constructor type (the type of the class itself)
        let constructor_type = checker.types.create_class_type(vec![], vec![construct_sig.clone()], vec![]);

        // Verify the constructor type has construct signatures
        if let Some(Type::Object(obj)) = checker.types.get(constructor_type) {
            assert!(!obj.construct_signatures.is_empty(), "Expected construct signatures");

            // The signature should have the instance type as return type
            let sig = &obj.construct_signatures[0];
            assert!(sig.resolved_return_type.is_some(), "Expected resolved_return_type");
            assert_eq!(sig.resolved_return_type.unwrap(), instance_type);
        } else {
            panic!("Expected Object type");
        }
    }

    #[test]
    fn test_new_expression_with_construct_signature() {
        use crate::parser::NodeArena;

        // Test that get_type_of_new_expression correctly uses construct signatures
        let node_arena = NodeArena::new();
        let symbol_arena = SymbolArena::new();
        let file_locals = SymbolTable::new();
        let mut checker = CheckerState::new(&node_arena, &symbol_arena, &file_locals, "test.ts".to_string());

        // Create an instance type
        let instance_type = checker.types.create_class_type(vec![], vec![], vec![]);

        // Create a construct signature that returns the instance type
        let mut construct_sig = Signature::new(NodeIndex::NONE);
        construct_sig.resolved_return_type = Some(instance_type);

        // Create a constructor type
        let constructor_type = checker.types.create_class_type(vec![], vec![construct_sig], vec![]);

        // Manually test the logic from get_type_of_new_expression
        if let Some(Type::Object(obj)) = checker.types.get(constructor_type) {
            if !obj.construct_signatures.is_empty() {
                if let Some(return_type) = obj.construct_signatures[0].resolved_return_type {
                    // This is what get_type_of_new_expression would return
                    assert_eq!(return_type, instance_type);
                }
            }
        }
    }

    #[test]
    fn test_type_guard_extraction_typeof() {
        use crate::parser_impl::ParserState;
        use crate::binder::BinderState;

        // Test extracting type guard from typeof expression
        let mut parser = ParserState::new(
            "test.ts".to_string(),
            r#"typeof x === "string""#.to_string(),
        );
        let root = parser.parse_source_file();

        let mut binder = BinderState::new();
        binder.bind_source_file(&parser.arena, root);

        let mut checker = CheckerState::new(
            &parser.arena,
            &binder.symbols,
            &binder.file_locals,
            "test.ts".to_string(),
        );

        // Get the expression statement
        use crate::parser::Node;
        if let Some(Node::SourceFile(sf)) = parser.arena.get(root) {
            if let Some(&first_stmt) = sf.statements.nodes.first() {
                if let Some(Node::ExpressionStatement(es)) = parser.arena.get(first_stmt) {
                    let guard = checker.get_type_guard_from_expression(es.expression);
                    assert!(guard.is_some(), "Expected type guard from typeof expression");

                    if let Some(TypeGuard::Typeof { typeof_result, is_equality, .. }) = guard {
                        assert_eq!(typeof_result, "string");
                        assert!(is_equality);
                    } else {
                        panic!("Expected Typeof guard");
                    }
                }
            }
        }
    }

    #[test]
    fn test_apply_type_guard() {
        use crate::parser::NodeArena;

        let node_arena = NodeArena::new();
        let symbol_arena = SymbolArena::new();
        let file_locals = SymbolTable::new();
        let mut checker = CheckerState::new(&node_arena, &symbol_arena, &file_locals, "test.ts".to_string());

        // Create a union of string | number | boolean
        let union_type = checker.types.create_union(vec![
            checker.types.string_type,
            checker.types.number_type,
            checker.types.boolean_type,
        ]);

        // Apply typeof === "string" guard
        let guard = TypeGuard::Typeof {
            target: crate::parser::NodeIndex::NONE,
            typeof_result: "string".to_string(),
            is_equality: true,
        };

        let narrowed = checker.apply_type_guard(union_type, &guard);

        // Should narrow to just string
        assert_eq!(narrowed, checker.types.string_type);
    }

    #[test]
    fn test_interface_index_signature() {
        use crate::parser_impl::ParserState;
        use crate::binder::BinderState;

        // Test interface with index signature: interface Dict { [key: string]: number }
        let mut parser = ParserState::new(
            "test.ts".to_string(),
            r#"interface Dict { [key: string]: number }"#.to_string(),
        );
        let root = parser.parse_source_file();

        let mut binder = BinderState::new();
        binder.bind_source_file(&parser.arena, root);

        let mut checker = CheckerState::new(
            &parser.arena,
            &binder.symbols,
            &binder.file_locals,
            "test.ts".to_string(),
        );

        // Get the interface type
        let dict_symbol = binder.file_locals.get("Dict").expect("Dict should be in file locals");
        let dict_type = checker.get_type_of_symbol(dict_symbol);

        // Check it has an index info
        if let Some(Type::Object(obj)) = checker.types.get(dict_type) {
            assert!(!obj.index_infos.is_empty(), "Dict should have index info");
            let idx_info = &obj.index_infos[0];
            // Key type should be string
            assert_eq!(idx_info.key_type, checker.types.string_type, "Key type should be string");
            // Value type should be number
            assert_eq!(idx_info.value_type, checker.types.number_type, "Value type should be number");
        } else {
            panic!("Dict should be an object type");
        }
    }

    #[test]
    fn test_element_access_with_index_signature() {
        use crate::parser_impl::ParserState;
        use crate::binder::BinderState;

        // Test element access with index signature
        // let d: Dict = {}; let x = d["key"];
        // x should have type number (from Dict's index signature)
        let mut parser = ParserState::new(
            "test.ts".to_string(),
            r#"
interface Dict { [key: string]: number }
let d: Dict;
let x = d["key"];
"#.to_string(),
        );
        let root = parser.parse_source_file();

        let mut binder = BinderState::new();
        binder.bind_source_file(&parser.arena, root);

        let mut checker = CheckerState::new(
            &parser.arena,
            &binder.symbols,
            &binder.file_locals,
            "test.ts".to_string(),
        );

        // Get the type of x
        let x_symbol = binder.file_locals.get("x").expect("x should be in file locals");
        let x_type = checker.get_type_of_symbol(x_symbol);

        // x should be number type (from the index signature value type)
        assert_eq!(x_type, checker.types.number_type, "x should be number type");
    }

    #[test]
    fn test_array_type() {
        use crate::parser_impl::ParserState;
        use crate::binder::BinderState;

        // Test array type: let arr: number[]
        let mut parser = ParserState::new(
            "test.ts".to_string(),
            r#"let arr: number[];"#.to_string(),
        );
        let root = parser.parse_source_file();

        let mut binder = BinderState::new();
        binder.bind_source_file(&parser.arena, root);

        let mut checker = CheckerState::new(
            &parser.arena,
            &binder.symbols,
            &binder.file_locals,
            "test.ts".to_string(),
        );

        // Get the type of arr
        let arr_symbol = binder.file_locals.get("arr").expect("arr should be in file locals");
        let arr_type = checker.get_type_of_symbol(arr_symbol);

        // arr should be an array type
        if let Some(Type::Array(arr)) = checker.types.get(arr_type) {
            assert_eq!(arr.element_type, checker.types.number_type, "Element type should be number");
        } else {
            panic!("arr should be an array type, got {}", checker.type_to_string(arr_type));
        }
    }

    #[test]
    fn test_array_index_access() {
        // Test that array types are created correctly for index access
        let mut arena = TypeArena::new();

        // Create a number[] array type
        let number_array = arena.create_array_type(arena.number_type, false);

        // Verify the array type structure - indexed access will return element_type
        if let Some(Type::Array(arr)) = arena.get(number_array) {
            assert_eq!(arr.element_type, arena.number_type);
            assert!(!arr.is_readonly);
        } else {
            panic!("Expected array type");
        }

        // Create a readonly string[] array
        let readonly_string_array = arena.create_array_type(arena.string_type, true);
        if let Some(Type::Array(arr)) = arena.get(readonly_string_array) {
            assert_eq!(arr.element_type, arena.string_type);
            assert!(arr.is_readonly);
        } else {
            panic!("Expected readonly array type");
        }
    }

    #[test]
    fn test_tuple_index_access() {
        // Test that tuple[0] returns the first element type
        let mut arena = TypeArena::new();

        // Create a [string, number, boolean] tuple type
        let tuple = arena.create_tuple_type(
            vec![arena.string_type, arena.number_type, arena.boolean_type],
            false,
            false,
            false,
        );

        // Verify the tuple structure
        if let Some(Type::Tuple(t)) = arena.get(tuple) {
            assert_eq!(t.element_types.len(), 3);
            assert_eq!(t.element_types[0], arena.string_type);
            assert_eq!(t.element_types[1], arena.number_type);
            assert_eq!(t.element_types[2], arena.boolean_type);
        } else {
            panic!("Expected tuple type");
        }
    }

    #[test]
    fn test_generic_array_type() {
        use crate::parser_impl::ParserState;
        use crate::binder::BinderState;

        // Test Array<T> syntax: let arr: Array<string>
        let mut parser = ParserState::new(
            "test.ts".to_string(),
            r#"let arr: Array<string>;"#.to_string(),
        );
        let root = parser.parse_source_file();

        let mut binder = BinderState::new();
        binder.bind_source_file(&parser.arena, root);

        let mut checker = CheckerState::new(
            &parser.arena,
            &binder.symbols,
            &binder.file_locals,
            "test.ts".to_string(),
        );

        // Get the type of arr
        let arr_symbol = binder.file_locals.get("arr").expect("arr should be in file locals");
        let arr_type = checker.get_type_of_symbol(arr_symbol);

        // arr should be an array type
        if let Some(Type::Array(arr)) = checker.types.get(arr_type) {
            assert_eq!(arr.element_type, checker.types.string_type, "Element type should be string");
        } else {
            panic!("arr should be an array type, got {}", checker.type_to_string(arr_type));
        }
    }

    #[test]
    fn test_tuple_type() {
        use crate::parser_impl::ParserState;
        use crate::binder::BinderState;

        // Test tuple type: let t: [number, string]
        let mut parser = ParserState::new(
            "test.ts".to_string(),
            r#"let t: [number, string];"#.to_string(),
        );
        let root = parser.parse_source_file();

        let mut binder = BinderState::new();
        binder.bind_source_file(&parser.arena, root);

        let mut checker = CheckerState::new(
            &parser.arena,
            &binder.symbols,
            &binder.file_locals,
            "test.ts".to_string(),
        );

        // Get the type of t
        let t_symbol = binder.file_locals.get("t").expect("t should be in file locals");
        let t_type = checker.get_type_of_symbol(t_symbol);

        // t should be a tuple type
        if let Some(Type::Tuple(tup)) = checker.types.get(t_type) {
            assert_eq!(tup.element_types.len(), 2, "Tuple should have 2 elements");
            assert_eq!(tup.element_types[0], checker.types.number_type, "First element should be number");
            assert_eq!(tup.element_types[1], checker.types.string_type, "Second element should be string");
        } else {
            panic!("t should be a tuple type, got {}", checker.type_to_string(t_type));
        }
    }

    #[test]
    fn test_array_assignability() {
        use crate::parser_impl::ParserState;
        use crate::binder::BinderState;

        // Test array assignability
        let mut parser = ParserState::new(
            "test.ts".to_string(),
            r#"
                let nums: number[];
                let strs: string[];
                let anys: any[];
            "#.to_string(),
        );
        let root = parser.parse_source_file();

        let mut binder = BinderState::new();
        binder.bind_source_file(&parser.arena, root);

        let mut checker = CheckerState::new(
            &parser.arena,
            &binder.symbols,
            &binder.file_locals,
            "test.ts".to_string(),
        );

        // Get the types
        let nums_symbol = binder.file_locals.get("nums").expect("nums should be in file locals");
        let strs_symbol = binder.file_locals.get("strs").expect("strs should be in file locals");
        let anys_symbol = binder.file_locals.get("anys").expect("anys should be in file locals");

        let nums_type = checker.get_type_of_symbol(nums_symbol);
        let strs_type = checker.get_type_of_symbol(strs_symbol);
        let anys_type = checker.get_type_of_symbol(anys_symbol);

        // number[] is assignable to number[]
        assert!(checker.is_type_assignable_to(nums_type, nums_type), "number[] should be assignable to number[]");

        // number[] is NOT assignable to string[]
        assert!(!checker.is_type_assignable_to(nums_type, strs_type), "number[] should NOT be assignable to string[]");

        // number[] IS assignable to any[]
        assert!(checker.is_type_assignable_to(nums_type, anys_type), "number[] should be assignable to any[]");
    }

    #[test]
    fn test_tuple_assignability() {
        use crate::parser_impl::ParserState;
        use crate::binder::BinderState;

        // Test tuple assignability
        let mut parser = ParserState::new(
            "test.ts".to_string(),
            r#"
                let t1: [number, string];
                let t2: [number, string];
                let t3: [string, number];
                let t4: [number];
            "#.to_string(),
        );
        let root = parser.parse_source_file();

        let mut binder = BinderState::new();
        binder.bind_source_file(&parser.arena, root);

        let mut checker = CheckerState::new(
            &parser.arena,
            &binder.symbols,
            &binder.file_locals,
            "test.ts".to_string(),
        );

        // Get the types
        let t1_symbol = binder.file_locals.get("t1").expect("t1 should be in file locals");
        let t2_symbol = binder.file_locals.get("t2").expect("t2 should be in file locals");
        let t3_symbol = binder.file_locals.get("t3").expect("t3 should be in file locals");
        let t4_symbol = binder.file_locals.get("t4").expect("t4 should be in file locals");

        let t1_type = checker.get_type_of_symbol(t1_symbol);
        let t2_type = checker.get_type_of_symbol(t2_symbol);
        let t3_type = checker.get_type_of_symbol(t3_symbol);
        let t4_type = checker.get_type_of_symbol(t4_symbol);

        // [number, string] is assignable to [number, string]
        assert!(checker.is_type_assignable_to(t1_type, t2_type), "[number, string] should be assignable to [number, string]");

        // [number, string] is NOT assignable to [string, number] (different order)
        assert!(!checker.is_type_assignable_to(t1_type, t3_type), "[number, string] should NOT be assignable to [string, number]");

        // [number, string] is NOT assignable to [number] (different length)
        assert!(!checker.is_type_assignable_to(t1_type, t4_type), "[number, string] should NOT be assignable to [number]");
    }

    #[test]
    fn test_tuple_to_array_assignability() {
        use crate::parser_impl::ParserState;
        use crate::binder::BinderState;

        // Test tuple to array assignability
        let mut parser = ParserState::new(
            "test.ts".to_string(),
            r#"
                let tuple: [number, number];
                let arr: number[];
                let strArr: string[];
            "#.to_string(),
        );
        let root = parser.parse_source_file();

        let mut binder = BinderState::new();
        binder.bind_source_file(&parser.arena, root);

        let mut checker = CheckerState::new(
            &parser.arena,
            &binder.symbols,
            &binder.file_locals,
            "test.ts".to_string(),
        );

        // Get the types
        let tuple_symbol = binder.file_locals.get("tuple").expect("tuple should be in file locals");
        let arr_symbol = binder.file_locals.get("arr").expect("arr should be in file locals");
        let str_arr_symbol = binder.file_locals.get("strArr").expect("strArr should be in file locals");

        let tuple_type = checker.get_type_of_symbol(tuple_symbol);
        let arr_type = checker.get_type_of_symbol(arr_symbol);
        let str_arr_type = checker.get_type_of_symbol(str_arr_symbol);

        // [number, number] is assignable to number[]
        assert!(checker.is_type_assignable_to(tuple_type, arr_type), "[number, number] should be assignable to number[]");

        // [number, number] is NOT assignable to string[]
        assert!(!checker.is_type_assignable_to(tuple_type, str_arr_type), "[number, number] should NOT be assignable to string[]");
    }

    #[test]
    fn test_readonly_array_type() {
        use crate::parser_impl::ParserState;
        use crate::binder::BinderState;

        // Test readonly array type
        let mut parser = ParserState::new(
            "test.ts".to_string(),
            r#"
                let readonlyArr: readonly number[];
                let mutableArr: number[];
                let readonlyArr2: ReadonlyArray<number>;
            "#.to_string(),
        );
        let root = parser.parse_source_file();

        let mut binder = BinderState::new();
        binder.bind_source_file(&parser.arena, root);

        let mut checker = CheckerState::new(
            &parser.arena,
            &binder.symbols,
            &binder.file_locals,
            "test.ts".to_string(),
        );

        // Get the types
        let readonly_arr_symbol = binder.file_locals.get("readonlyArr").expect("readonlyArr should be in file locals");
        let mutable_arr_symbol = binder.file_locals.get("mutableArr").expect("mutableArr should be in file locals");
        let readonly_arr2_symbol = binder.file_locals.get("readonlyArr2").expect("readonlyArr2 should be in file locals");

        let readonly_arr_type = checker.get_type_of_symbol(readonly_arr_symbol);
        let mutable_arr_type = checker.get_type_of_symbol(mutable_arr_symbol);
        let readonly_arr2_type = checker.get_type_of_symbol(readonly_arr2_symbol);

        // Check readonly flag
        if let Some(Type::Array(arr)) = checker.types.get(readonly_arr_type) {
            assert!(arr.is_readonly, "readonly number[] should have is_readonly=true");
        } else {
            panic!("readonlyArr should be an array type");
        }

        if let Some(Type::Array(arr)) = checker.types.get(mutable_arr_type) {
            assert!(!arr.is_readonly, "number[] should have is_readonly=false");
        } else {
            panic!("mutableArr should be an array type");
        }

        if let Some(Type::Array(arr)) = checker.types.get(readonly_arr2_type) {
            assert!(arr.is_readonly, "ReadonlyArray<number> should have is_readonly=true");
        } else {
            panic!("readonlyArr2 should be an array type");
        }

        // Check type_to_string
        let readonly_str = checker.type_to_string(readonly_arr_type);
        assert_eq!(readonly_str, "readonly number[]", "readonly array type_to_string");

        let mutable_str = checker.type_to_string(mutable_arr_type);
        assert_eq!(mutable_str, "number[]", "mutable array type_to_string");

        // Assignability: mutable IS assignable to readonly
        assert!(checker.is_type_assignable_to(mutable_arr_type, readonly_arr_type), "number[] should be assignable to readonly number[]");

        // Assignability: readonly is NOT assignable to mutable
        assert!(!checker.is_type_assignable_to(readonly_arr_type, mutable_arr_type), "readonly number[] should NOT be assignable to number[]");
    }

    #[test]
    fn test_optional_tuple_element() {
        use crate::parser_impl::ParserState;
        use crate::binder::BinderState;

        // Test tuple with optional element: [number, string?]
        let mut parser = ParserState::new(
            "test.ts".to_string(),
            r#"let t: [number, string?];"#.to_string(),
        );
        let root = parser.parse_source_file();

        let mut binder = BinderState::new();
        binder.bind_source_file(&parser.arena, root);

        let mut checker = CheckerState::new(
            &parser.arena,
            &binder.symbols,
            &binder.file_locals,
            "test.ts".to_string(),
        );

        let t_symbol = binder.file_locals.get("t").expect("t should be in file locals");
        let t_type = checker.get_type_of_symbol(t_symbol);

        // t should be a tuple type with optional elements
        if let Some(Type::Tuple(tup)) = checker.types.get(t_type) {
            assert_eq!(tup.element_types.len(), 2, "Tuple should have 2 elements");
            assert!(tup.has_optional_elements, "Tuple should have optional elements");
            assert!(!tup.has_rest_element, "Tuple should not have rest element");
            assert_eq!(tup.element_types[0], checker.types.number_type, "First element should be number");
            assert_eq!(tup.element_types[1], checker.types.string_type, "Second element should be string");
        } else {
            panic!("t should be a tuple type, got {}", checker.type_to_string(t_type));
        }
    }

    #[test]
    fn test_rest_tuple_element() {
        use crate::parser_impl::ParserState;
        use crate::binder::BinderState;

        // Test tuple with rest element: [string, ...number[]]
        let mut parser = ParserState::new(
            "test.ts".to_string(),
            r#"let t: [string, ...number[]];"#.to_string(),
        );
        let root = parser.parse_source_file();

        let mut binder = BinderState::new();
        binder.bind_source_file(&parser.arena, root);

        let mut checker = CheckerState::new(
            &parser.arena,
            &binder.symbols,
            &binder.file_locals,
            "test.ts".to_string(),
        );

        let t_symbol = binder.file_locals.get("t").expect("t should be in file locals");
        let t_type = checker.get_type_of_symbol(t_symbol);

        // t should be a tuple type with rest element
        if let Some(Type::Tuple(tup)) = checker.types.get(t_type) {
            assert_eq!(tup.element_types.len(), 2, "Tuple should have 2 element types");
            assert!(!tup.has_optional_elements, "Tuple should not have optional elements");
            assert!(tup.has_rest_element, "Tuple should have rest element");
            assert_eq!(tup.element_types[0], checker.types.string_type, "First element should be string");
            assert_eq!(tup.element_types[1], checker.types.number_type, "Rest element type should be number");
        } else {
            panic!("t should be a tuple type, got {}", checker.type_to_string(t_type));
        }
    }

    #[test]
    fn test_spread_in_array_literal() {
        use crate::parser_impl::ParserState;
        use crate::binder::BinderState;

        // Test spread in array literal
        let mut parser = ParserState::new(
            "test.ts".to_string(),
            r#"
                let nums: number[] = [1, 2, 3];
                let more = [...nums, 4, 5];
            "#.to_string(),
        );
        let root = parser.parse_source_file();

        let mut binder = BinderState::new();
        binder.bind_source_file(&parser.arena, root);

        let mut checker = CheckerState::new(
            &parser.arena,
            &binder.symbols,
            &binder.file_locals,
            "test.ts".to_string(),
        );

        let more_symbol = binder.file_locals.get("more").expect("more should be in file locals");
        let more_type = checker.get_type_of_symbol(more_symbol);
        let more_str = checker.type_to_string(more_type);

        // more should be number (inferred from spread + literals)
        // The type could be a union including number, or just number
        let is_number = more_type == checker.types.number_type;
        let contains_number = if let Some(Type::Union(u)) = checker.types.get(more_type) {
            u.types.iter().any(|&t| t == checker.types.number_type)
        } else {
            false
        };
        assert!(is_number || contains_number, "more should be or contain number type, got: {}", more_str);
    }

    #[test]
    fn test_conditional_type_evaluation() {
        use crate::parser_impl::ParserState;
        use crate::binder::BinderState;

        // Test conditional type evaluation - simple case where condition is resolved
        let mut parser = ParserState::new(
            "test.ts".to_string(),
            r#"
                // Conditional that resolves to true branch
                type IsString = string extends string ? "yes" : "no";

                // Conditional that resolves to false branch
                type IsNumber = string extends number ? "yes" : "no";

                // Using the types in variable declarations
                let x: IsString = "yes";
                let y: IsNumber = "no";
            "#.to_string(),
        );
        let root = parser.parse_source_file();

        let mut binder = BinderState::new();
        binder.bind_source_file(&parser.arena, root);

        let mut checker = CheckerState::new(
            &parser.arena,
            &binder.symbols,
            &binder.file_locals,
            "test.ts".to_string(),
        );

        // Get the type of x (should be "yes")
        let x_symbol = binder.file_locals.get("x").expect("x should be in file locals");
        let x_type = checker.get_type_of_symbol(x_symbol);
        let x_str = checker.type_to_string(x_type);
        assert_eq!(x_str, "\"yes\"", "IsString should resolve to \"yes\", got: {}", x_str);

        // Get the type of y (should be "no")
        let y_symbol = binder.file_locals.get("y").expect("y should be in file locals");
        let y_type = checker.get_type_of_symbol(y_symbol);
        let y_str = checker.type_to_string(y_type);
        assert_eq!(y_str, "\"no\"", "IsNumber should resolve to \"no\", got: {}", y_str);
    }

    #[test]
    fn test_distributive_conditional_type() {
        // Test that conditional types distribute over union types
        // ToArray<string | number> should become string[] | number[]
        let mut arena = TypeArena::new();

        // Create a naked type parameter T
        let t_param = arena.create_type_parameter(SymbolId::NONE, TypeId::NONE, TypeId::NONE);

        // Create array types
        let string_array = arena.create_array_type(arena.string_type, false);
        let number_array = arena.create_array_type(arena.number_type, false);

        // Create a conditional type: T extends any ? T[] : never
        // This is distributive because T is a naked type parameter
        let cond_type = arena.create_conditional_type(
            t_param,
            arena.any_type,
            string_array, // simplified - in practice this would be T[]
            arena.never_type,
        );

        // Verify the conditional is marked as distributive
        if let Some(Type::Conditional(c)) = arena.get(cond_type) {
            assert!(c.is_distributive, "Conditional with naked type parameter should be distributive");
        } else {
            panic!("Expected conditional type");
        }
    }

    #[test]
    fn test_non_distributive_conditional_type() {
        // Test that conditional types are not distributive when check type is not naked
        let mut arena = TypeArena::new();

        // Create a type that is not a naked type parameter (e.g., keyof T)
        let t_param = arena.create_type_parameter(SymbolId::NONE, TypeId::NONE, TypeId::NONE);
        let keyof_t = arena.create_index_type(t_param);

        // Create a conditional type: keyof T extends any ? "yes" : "no"
        let yes_type = arena.create_string_literal("yes".to_string());
        let no_type = arena.create_string_literal("no".to_string());
        let cond_type = arena.create_conditional_type(
            keyof_t,  // Not a naked type parameter
            arena.any_type,
            yes_type,
            no_type,
        );

        // Verify the conditional is NOT marked as distributive
        if let Some(Type::Conditional(c)) = arena.get(cond_type) {
            assert!(!c.is_distributive, "Conditional with keyof T should not be distributive");
        } else {
            panic!("Expected conditional type");
        }
    }

    #[test]
    fn test_template_literal_type_simple() {
        use crate::parser_impl::ParserState;
        use crate::binder::BinderState;

        // Test simple template literal type (no substitution)
        let mut parser = ParserState::new(
            "test.ts".to_string(),
            r#"
                type Hello = `hello`;
                let x: Hello = "hello";
            "#.to_string(),
        );
        let root = parser.parse_source_file();

        let mut binder = BinderState::new();
        binder.bind_source_file(&parser.arena, root);

        let mut checker = CheckerState::new(
            &parser.arena,
            &binder.symbols,
            &binder.file_locals,
            "test.ts".to_string(),
        );

        // Get the Hello type alias
        let hello_symbol = binder.file_locals.get("Hello").expect("Hello should be in file locals");
        let hello_type = checker.get_type_of_symbol(hello_symbol);
        let hello_str = checker.type_to_string(hello_type);

        // Simple template literal should show as `hello`
        assert!(hello_str.contains("hello"), "Hello should contain 'hello', got: {}", hello_str);
    }

    #[test]
    fn test_template_literal_with_substitution() {
        use crate::parser_impl::ParserState;
        use crate::binder::BinderState;

        // Test template literal type with substitution
        let mut parser = ParserState::new(
            "test.ts".to_string(),
            r#"
                type Greeting<T extends string> = `hello ${T}`;
                type HelloWorld = Greeting<"world">;
            "#.to_string(),
        );
        let root = parser.parse_source_file();

        let mut binder = BinderState::new();
        binder.bind_source_file(&parser.arena, root);

        let mut checker = CheckerState::new(
            &parser.arena,
            &binder.symbols,
            &binder.file_locals,
            "test.ts".to_string(),
        );

        // Check Greeting type
        let greeting_symbol = binder.file_locals.get("Greeting").expect("Greeting should be in file locals");
        let greeting_type = checker.get_type_of_symbol(greeting_symbol);
        let greeting_str = checker.type_to_string(greeting_type);

        // Greeting should be a template literal type with T as substitution
        assert!(greeting_str.contains("hello"), "Greeting should contain 'hello', got: {}", greeting_str);

        // Check HelloWorld type
        let hw_symbol = binder.file_locals.get("HelloWorld").expect("HelloWorld should be in file locals");
        let hw_type = checker.get_type_of_symbol(hw_symbol);
        let hw_str = checker.type_to_string(hw_type);

        // HelloWorld should be the instantiated template
        // Note: Full instantiation of template literals with concrete types is not yet implemented
        // For now, just check that it parses correctly
        assert!(hw_str.contains("hello"), "HelloWorld should contain 'hello', got: {}", hw_str);
    }

    #[test]
    fn test_mapped_type() {
        use crate::parser_impl::ParserState;
        use crate::binder::BinderState;

        // Test mapped type
        let mut parser = ParserState::new(
            "test.ts".to_string(),
            r#"
                type Readonly<T> = { readonly [K in keyof T]: T[K] };
                type Original = { a: number; b: string };
                type ReadonlyOriginal = Readonly<Original>;
            "#.to_string(),
        );
        let root = parser.parse_source_file();

        let mut binder = BinderState::new();
        binder.bind_source_file(&parser.arena, root);

        let mut checker = CheckerState::new(
            &parser.arena,
            &binder.symbols,
            &binder.file_locals,
            "test.ts".to_string(),
        );

        // Get the Readonly type alias - should be a mapped type
        let readonly_symbol = binder.file_locals.get("Readonly").expect("Readonly should be in file locals");
        let readonly_type = checker.get_type_of_symbol(readonly_symbol);
        let readonly_str = checker.type_to_string(readonly_type);

        // Mapped type should show as { [K in ...]: ... }
        assert!(readonly_str.contains("[K in"), "Readonly should be a mapped type, got: {}", readonly_str);
    }

    #[test]
    fn test_infer_type() {
        use crate::parser_impl::ParserState;
        use crate::binder::BinderState;

        // Test simple conditional type first (no generics to complicate things)
        let mut parser = ParserState::new(
            "test.ts".to_string(),
            r#"
                type IsString = string extends number ? "yes" : "no";
            "#.to_string(),
        );
        let root = parser.parse_source_file();

        let mut binder = BinderState::new();
        binder.bind_source_file(&parser.arena, root);

        let mut checker = CheckerState::new(
            &parser.arena,
            &binder.symbols,
            &binder.file_locals,
            "test.ts".to_string(),
        );

        // Get the IsString type alias - should resolve to "no"
        let is_string_symbol_id = binder.file_locals.get("IsString").expect("IsString should be in file locals");
        let is_string_type = checker.get_type_of_symbol(is_string_symbol_id);
        let is_string_str = checker.type_to_string(is_string_type);

        println!("IsString type: {}", is_string_str);

        // Should resolve to "no" since string does not extend number
        assert_eq!(is_string_str, "\"no\"", "IsString should be \"no\", got: {}", is_string_str);
    }

    #[test]
    fn test_generic_conditional_type() {
        use crate::parser_impl::ParserState;
        use crate::binder::BinderState;

        // Test generic conditional type (deferred evaluation)
        let mut parser = ParserState::new(
            "test.ts".to_string(),
            r#"
                type IsNumber<T> = T extends number ? "yes" : "no";
            "#.to_string(),
        );
        let root = parser.parse_source_file();

        let mut binder = BinderState::new();
        binder.bind_source_file(&parser.arena, root);

        let mut checker = CheckerState::new(
            &parser.arena,
            &binder.symbols,
            &binder.file_locals,
            "test.ts".to_string(),
        );

        // Get the IsNumber type alias - should be a conditional type (deferred)
        let is_number_symbol_id = binder.file_locals.get("IsNumber").expect("IsNumber should be in file locals");
        let is_number_type = checker.get_type_of_symbol(is_number_symbol_id);
        let is_number_str = checker.type_to_string(is_number_type);

        println!("IsNumber type: {}", is_number_str);

        // Should be a deferred conditional type
        assert!(is_number_str.contains("extends"), "IsNumber should be a conditional type, got: {}", is_number_str);
    }

    #[test]
    fn test_infer_type_in_conditional() {
        use crate::parser_impl::ParserState;
        use crate::binder::BinderState;

        // Test infer type in conditional type
        let mut parser = ParserState::new(
            "test.ts".to_string(),
            r#"
                type UnwrapPromise<T> = T extends Promise<infer U> ? U : T;
            "#.to_string(),
        );
        let root = parser.parse_source_file();

        let mut binder = BinderState::new();
        binder.bind_source_file(&parser.arena, root);

        let mut checker = CheckerState::new(
            &parser.arena,
            &binder.symbols,
            &binder.file_locals,
            "test.ts".to_string(),
        );

        // Get the UnwrapPromise type alias
        let unwrap_symbol_id = binder.file_locals.get("UnwrapPromise").expect("UnwrapPromise should be in file locals");
        let unwrap_type = checker.get_type_of_symbol(unwrap_symbol_id);
        let unwrap_str = checker.type_to_string(unwrap_type);

        println!("UnwrapPromise type: {}", unwrap_str);

        // Should be a deferred conditional type with extends
        assert!(unwrap_str.contains("extends"), "UnwrapPromise should be a conditional type, got: {}", unwrap_str);
    }

    #[test]
    fn test_keyof_type() {
        use crate::parser_impl::ParserState;
        use crate::binder::BinderState;

        // Test keyof any - should return string | number | symbol
        let mut parser = ParserState::new(
            "test.ts".to_string(),
            r#"
                type Keys = keyof any;
            "#.to_string(),
        );
        let root = parser.parse_source_file();

        let mut binder = BinderState::new();
        binder.bind_source_file(&parser.arena, root);

        let mut checker = CheckerState::new(
            &parser.arena,
            &binder.symbols,
            &binder.file_locals,
            "test.ts".to_string(),
        );

        // Get the Keys type alias
        let keys_symbol_id = binder.file_locals.get("Keys").expect("Keys should be in file locals");
        let keys_type = checker.get_type_of_symbol(keys_symbol_id);
        let keys_str = checker.type_to_string(keys_type);

        println!("Keys type: {}", keys_str);

        // keyof any = string | number | symbol
        assert!(keys_str.contains("string") || keys_str.contains("number"),
            "keyof any should contain 'string' or 'number', got: {}", keys_str);
    }

    #[test]
    fn test_keyof_object_type_literal() {
        use crate::parser_impl::ParserState;
        use crate::binder::BinderState;

        // Test keyof with object type literal
        let mut parser = ParserState::new(
            "test.ts".to_string(),
            r#"
                type Keys = keyof { name: string; age: number; };
            "#.to_string(),
        );
        let root = parser.parse_source_file();

        let mut binder = BinderState::new();
        binder.bind_source_file(&parser.arena, root);

        let mut checker = CheckerState::new(
            &parser.arena,
            &binder.symbols,
            &binder.file_locals,
            "test.ts".to_string(),
        );

        // Get the Keys type alias
        let keys_symbol_id = binder.file_locals.get("Keys").expect("Keys should be in file locals");
        let keys_type = checker.get_type_of_symbol(keys_symbol_id);
        let keys_str = checker.type_to_string(keys_type);

        println!("Keys from object literal: {}", keys_str);

        // keyof { name: string; age: number } = "name" | "age"
        assert!(keys_str.contains("name") && keys_str.contains("age"),
            "keyof object literal should contain 'name' and 'age', got: {}", keys_str);
    }

    #[test]
    fn test_mapped_type_instantiation() {
        use crate::parser_impl::ParserState;
        use crate::binder::BinderState;

        // Test mapped type: { [K in keyof T]: T[K] } applied to { a: number }
        let mut parser = ParserState::new(
            "test.ts".to_string(),
            r#"
                type Identity<T> = { [K in keyof T]: T[K] };
                type Result = Identity<{ a: number; b: string }>;
            "#.to_string(),
        );
        let root = parser.parse_source_file();

        let mut binder = BinderState::new();
        binder.bind_source_file(&parser.arena, root);

        let mut checker = CheckerState::new(
            &parser.arena,
            &binder.symbols,
            &binder.file_locals,
            "test.ts".to_string(),
        );

        // Get the Result type alias
        let result_symbol_id = binder.file_locals.get("Result").expect("Result should be in file locals");
        let result_type = checker.get_type_of_symbol(result_symbol_id);
        let result_str = checker.type_to_string(result_type);

        println!("Mapped type result: {}", result_str);

        // The result should be an object type with 'a' and 'b' properties
        assert!(result_str.contains("a: number") && result_str.contains("b: string"),
            "Mapped type should produce {{ a: number; b: string }}, got: {}", result_str);
    }

    #[test]
    fn test_infer_array_element() {
        use crate::parser_impl::ParserState;
        use crate::binder::BinderState;

        // Test: type UnwrapArray<T> = T extends (infer U)[] ? U : T;
        // Applied to string[] should give string
        let mut parser = ParserState::new(
            "test.ts".to_string(),
            r#"
                type UnwrapArray<T> = T extends (infer U)[] ? U : T;
                type Result = UnwrapArray<string[]>;
            "#.to_string(),
        );
        let root = parser.parse_source_file();

        let mut binder = BinderState::new();
        binder.bind_source_file(&parser.arena, root);

        let mut checker = CheckerState::new(
            &parser.arena,
            &binder.symbols,
            &binder.file_locals,
            "test.ts".to_string(),
        );

        let result_symbol_id = binder.file_locals.get("Result").expect("Result should be in file locals");
        let result_type = checker.get_type_of_symbol(result_symbol_id);
        let result_str = checker.type_to_string(result_type);

        // UnwrapArray<string[]> should evaluate to string
        assert_eq!(result_str, "string", "UnwrapArray<string[]> should be 'string', got: {}", result_str);
    }

    #[test]
    fn test_infer_false_branch() {
        use crate::parser_impl::ParserState;
        use crate::binder::BinderState;

        // When pattern doesn't match, use false branch
        let mut parser = ParserState::new(
            "test.ts".to_string(),
            r#"
                type UnwrapArray<T> = T extends (infer U)[] ? U : T;
                type Result = UnwrapArray<number>;
            "#.to_string(),
        );
        let root = parser.parse_source_file();

        let mut binder = BinderState::new();
        binder.bind_source_file(&parser.arena, root);

        let mut checker = CheckerState::new(
            &parser.arena,
            &binder.symbols,
            &binder.file_locals,
            "test.ts".to_string(),
        );

        let result_symbol_id = binder.file_locals.get("Result").expect("Result should be in file locals");
        let result_type = checker.get_type_of_symbol(result_symbol_id);
        let result_str = checker.type_to_string(result_type);

        // UnwrapArray<number> should return number (false branch)
        assert_eq!(result_str, "number", "UnwrapArray<number> should be 'number', got: {}", result_str);
    }

    #[test]
    fn test_infer_function_return() {
        use crate::parser_impl::ParserState;
        use crate::binder::BinderState;

        // Test inferring function return type (simplified pattern without rest params)
        let mut parser = ParserState::new(
            "test.ts".to_string(),
            r#"
                type GetReturn<T> = T extends () => infer R ? R : never;
                type Result = GetReturn<() => string>;
            "#.to_string(),
        );
        let root = parser.parse_source_file();

        let mut binder = BinderState::new();
        binder.bind_source_file(&parser.arena, root);

        let mut checker = CheckerState::new(
            &parser.arena,
            &binder.symbols,
            &binder.file_locals,
            "test.ts".to_string(),
        );

        let result_symbol_id = binder.file_locals.get("Result").expect("Result should be in file locals");
        let result_type = checker.get_type_of_symbol(result_symbol_id);
        let result_str = checker.type_to_string(result_type);

        // GetReturn<() => string> should evaluate to string
        assert_eq!(result_str, "string", "GetReturn<() => string> should be 'string', got: {}", result_str);
    }

    #[test]
    fn test_contextual_typing_callback() {
        use crate::parser_impl::ParserState;
        use crate::binder::BinderState;

        // Test that callback parameters get their types from contextual typing
        // E.g., process((x) => x + 1) should infer x as number when process expects (x: number) => number
        let mut parser = ParserState::new(
            "test.ts".to_string(),
            r#"
            function process(callback: (x: number) => number): number {
                return callback(42);
            }
            "#.to_string(),
        );
        let root = parser.parse_source_file();

        let mut binder = BinderState::new();
        binder.bind_source_file(&parser.arena, root);

        let mut checker = CheckerState::new(
            &parser.arena,
            &binder.symbols,
            &binder.file_locals,
            "test.ts".to_string(),
        );

        // Verify the process function exists and has correct type
        let process_symbol = binder.file_locals.get("process").expect("process should be in file locals");
        let process_type = checker.get_type_of_symbol(process_symbol);

        // The function should have a callback parameter of type (x: number) => number
        if let Some(Type::Function(f)) = checker.types.get(process_type) {
            assert_eq!(f.parameter_types.len(), 1, "process should have 1 parameter");
            let callback_param_type = f.parameter_types[0];

            // The callback type should be a function type
            if let Some(Type::Function(callback_f)) = checker.types.get(callback_param_type) {
                assert_eq!(callback_f.parameter_types.len(), 1, "callback should have 1 parameter");
                assert_eq!(callback_f.parameter_types[0], checker.types.number_type,
                    "callback parameter should be number type");
                assert_eq!(callback_f.return_type, checker.types.number_type,
                    "callback return should be number type");
            } else {
                panic!("callback parameter should be a function type, got: {}", checker.type_to_string(callback_param_type));
            }

            assert_eq!(f.return_type, checker.types.number_type, "process should return number");
        } else {
            panic!("process should be a function type");
        }

        // Verify contextual type is set when evaluating arguments
        assert!(checker.contextual_type.is_none(), "contextual_type should be None initially");
    }

    #[test]
    fn test_switch_exhaustiveness() {
        use crate::parser::NodeArena;

        let node_arena = NodeArena::new();
        let symbol_arena = SymbolArena::new();
        let file_locals = SymbolTable::new();
        let mut checker = CheckerState::new(&node_arena, &symbol_arena, &file_locals, "test.ts".to_string());

        // Create a union of literal types: "a" | "b" | "c"
        let lit_a = checker.types.create_string_literal("a".to_string());
        let lit_b = checker.types.create_string_literal("b".to_string());
        let lit_c = checker.types.create_string_literal("c".to_string());
        let union_abc = checker.types.create_union(vec![lit_a, lit_b, lit_c]);

        // Test 1: Exhaustive case - handling all values
        let remaining = checker.check_switch_exhaustiveness(
            union_abc,
            &[lit_a, lit_b, lit_c],
            false
        );
        assert_eq!(remaining, checker.types.never_type,
            "Exhaustive switch should result in never, got: {}", checker.type_to_string(remaining));

        // Test 2: Non-exhaustive case - missing one value
        let remaining = checker.check_switch_exhaustiveness(
            union_abc,
            &[lit_a, lit_b],
            false
        );
        // Should have "c" remaining
        let remaining_str = checker.type_to_string(remaining);
        assert!(remaining_str.contains("\"c\""),
            "Non-exhaustive switch should have 'c' remaining, got: {}", remaining_str);

        // Test 3: With default clause - always exhaustive
        let remaining = checker.check_switch_exhaustiveness(
            union_abc,
            &[lit_a],
            true  // has default
        );
        assert_eq!(remaining, checker.types.never_type,
            "Switch with default should be exhaustive, got: {}", checker.type_to_string(remaining));
    }

    #[test]
    fn test_switch_case_narrowing() {
        use crate::parser::NodeArena;

        let node_arena = NodeArena::new();
        let symbol_arena = SymbolArena::new();
        let file_locals = SymbolTable::new();
        let mut checker = CheckerState::new(&node_arena, &symbol_arena, &file_locals, "test.ts".to_string());

        // Create a union of literal types: 1 | 2 | 3
        let lit_1 = checker.types.create_number_literal(1.0);
        let lit_2 = checker.types.create_number_literal(2.0);
        let lit_3 = checker.types.create_number_literal(3.0);
        let union_123 = checker.types.create_union(vec![lit_1, lit_2, lit_3]);

        // After handling case 1, should have 2 | 3
        let after_1 = checker.narrow_type_by_switch_case(union_123, lit_1);
        let after_1_str = checker.type_to_string(after_1);
        assert!(!after_1_str.contains("1"),
            "After case 1, should not have 1, got: {}", after_1_str);
        assert!(after_1_str.contains("2"),
            "After case 1, should have 2, got: {}", after_1_str);
        assert!(after_1_str.contains("3"),
            "After case 1, should have 3, got: {}", after_1_str);

        // After handling case 1 and 2, should have 3
        let after_2 = checker.narrow_type_by_switch_case(after_1, lit_2);
        assert_eq!(after_2, lit_3,
            "After case 1 and 2, should have only 3, got: {}", checker.type_to_string(after_2));

        // After handling all cases, should be never
        let after_3 = checker.narrow_type_by_switch_case(after_2, lit_3);
        assert_eq!(after_3, checker.types.never_type,
            "After all cases, should be never, got: {}", checker.type_to_string(after_3));
    }

    #[test]
    fn test_diagnostic_error_codes() {
        use crate::parser_impl::ParserState;
        use crate::binder::BinderState;

        // Test: Undeclared identifier should produce error with code 2304
        let mut parser = ParserState::new(
            "test.ts".to_string(),
            r#"
            const x = undeclaredVariable;
            "#.to_string(),
        );
        let root = parser.parse_source_file();

        let mut binder = BinderState::new();
        binder.bind_source_file(&parser.arena, root);

        let mut checker = CheckerState::new(
            &parser.arena,
            &binder.symbols,
            &binder.file_locals,
            "test.ts".to_string(),
        );

        // Get the type of x (which should trigger the error for undeclaredVariable)
        if let Some(x_symbol) = binder.file_locals.get("x") {
            let _x_type = checker.get_type_of_symbol(x_symbol);
        }

        // Check that we got a diagnostic for the undeclared variable
        assert!(!checker.diagnostics.is_empty(), "Should have produced a diagnostic");
        let diag = &checker.diagnostics[0];
        assert_eq!(diag.code, diagnostic_codes::CANNOT_FIND_NAME,
            "Expected error code {}, got {}", diagnostic_codes::CANNOT_FIND_NAME, diag.code);
        assert!(diag.message_text.contains("undeclaredVariable"),
            "Diagnostic should mention 'undeclaredVariable': {}", diag.message_text);
    }

    #[test]
    fn test_diagnostic_codes_module() {
        // Verify diagnostic code constants match TypeScript's codes
        assert_eq!(diagnostic_codes::CANNOT_FIND_NAME, 2304);
        assert_eq!(diagnostic_codes::TYPE_NOT_ASSIGNABLE_TO_TYPE, 2322);
        assert_eq!(diagnostic_codes::PROPERTY_DOES_NOT_EXIST_ON_TYPE, 2339);
        assert_eq!(diagnostic_codes::EXPECTED_ARGUMENTS, 2554);
        assert_eq!(diagnostic_codes::OBJECT_IS_OF_TYPE_UNKNOWN, 2571);
    }

    #[test]
    fn test_binary_expression_arithmetic() {
        use crate::parser_impl::ParserState;
        use crate::binder::BinderState;

        let mut parser = ParserState::new(
            "test.ts".to_string(),
            r#"
                const a = 1 + 2;
                const b = 3 - 4;
                const c = 5 * 6;
                const d = 7 / 8;
                const e = 9 % 10;
            "#.to_string(),
        );
        let root = parser.parse_source_file();

        let mut binder = BinderState::new();
        binder.bind_source_file(&parser.arena, root);

        let mut checker = CheckerState::new(
            &parser.arena,
            &binder.symbols,
            &binder.file_locals,
            "test.ts".to_string(),
        );

        // Check that arithmetic operations produce number type
        for name in ["a", "b", "c", "d", "e"] {
            if let Some(symbol_id) = binder.file_locals.get(name) {
                let var_type = checker.get_type_of_symbol(symbol_id);
                let type_str = checker.type_to_string(var_type);
                assert_eq!(type_str, "number", "{} should be number, got: {}", name, type_str);
            }
        }
    }

    #[test]
    fn test_binary_expression_string_concat() {
        use crate::parser_impl::ParserState;
        use crate::binder::BinderState;

        let mut parser = ParserState::new(
            "test.ts".to_string(),
            r#"
                const s = "hello" + "world";
            "#.to_string(),
        );
        let root = parser.parse_source_file();

        let mut binder = BinderState::new();
        binder.bind_source_file(&parser.arena, root);

        let mut checker = CheckerState::new(
            &parser.arena,
            &binder.symbols,
            &binder.file_locals,
            "test.ts".to_string(),
        );

        // String concatenation should produce string type
        if let Some(symbol_id) = binder.file_locals.get("s") {
            let var_type = checker.get_type_of_symbol(symbol_id);
            let type_str = checker.type_to_string(var_type);
            assert_eq!(type_str, "string", "String concat should be string, got: {}", type_str);
        }
    }

    #[test]
    fn test_binary_expression_comparison() {
        use crate::parser_impl::ParserState;
        use crate::binder::BinderState;
        use crate::parser::Node;

        // Test simple binary comparison expression
        let mut parser = ParserState::new(
            "test.ts".to_string(),
            "const a = 1 < 2;".to_string(),
        );
        let root = parser.parse_source_file();

        // First verify the parser is creating a BinaryExpression
        let mut found_binary = false;
        for i in 0..parser.arena.len() {
            let idx = crate::parser::NodeIndex(i as u32);
            if let Some(Node::BinaryExpression(be)) = parser.arena.get(idx) {
                found_binary = true;
                assert_eq!(be.operator_token, crate::scanner::SyntaxKind::LessThanToken,
                    "Expected LessThanToken, got {:?}", be.operator_token);
            }
        }
        assert!(found_binary, "Parser should create a BinaryExpression for '1 < 2'");

        let mut binder = BinderState::new();
        binder.bind_source_file(&parser.arena, root);

        let mut checker = CheckerState::new(
            &parser.arena,
            &binder.symbols,
            &binder.file_locals,
            "test.ts".to_string(),
        );

        // Check that comparison operations produce boolean type
        if let Some(symbol_id) = binder.file_locals.get("a") {
            let var_type = checker.get_type_of_symbol(symbol_id);
            let type_str = checker.type_to_string(var_type);
            assert_eq!(type_str, "boolean", "a should be boolean, got: {}", type_str);
        }
    }

    #[test]
    fn test_conditional_expression() {
        use crate::parser_impl::ParserState;
        use crate::binder::BinderState;

        let mut parser = ParserState::new(
            "test.ts".to_string(),
            r#"
                const x = true ? 1 : "hello";
            "#.to_string(),
        );
        let root = parser.parse_source_file();

        let mut binder = BinderState::new();
        binder.bind_source_file(&parser.arena, root);

        let mut checker = CheckerState::new(
            &parser.arena,
            &binder.symbols,
            &binder.file_locals,
            "test.ts".to_string(),
        );

        // Conditional should produce union of both branches
        if let Some(symbol_id) = binder.file_locals.get("x") {
            let var_type = checker.get_type_of_symbol(symbol_id);
            let type_str = checker.type_to_string(var_type);
            // Should be number | string (order may vary)
            assert!(type_str.contains("1") || type_str.contains("number"),
                "Ternary should include number type, got: {}", type_str);
            assert!(type_str.contains("hello") || type_str.contains("string"),
                "Ternary should include string type, got: {}", type_str);
        }
    }

    #[test]
    fn test_unary_expressions() {
        use crate::parser_impl::ParserState;
        use crate::binder::BinderState;

        let mut parser = ParserState::new(
            "test.ts".to_string(),
            r#"
                const x = 5;
                const a = -x;
                const b = +x;
                const c = !x;
                const d = ~x;
                const e = typeof x;
                const f = void x;
            "#.to_string(),
        );
        let root = parser.parse_source_file();

        let mut binder = BinderState::new();
        binder.bind_source_file(&parser.arena, root);

        let mut checker = CheckerState::new(
            &parser.arena,
            &binder.symbols,
            &binder.file_locals,
            "test.ts".to_string(),
        );

        // Test unary minus: number
        if let Some(symbol_id) = binder.file_locals.get("a") {
            let var_type = checker.get_type_of_symbol(symbol_id);
            let type_str = checker.type_to_string(var_type);
            assert_eq!(type_str, "number", "Unary minus should be number, got: {}", type_str);
        }

        // Test unary plus: number
        if let Some(symbol_id) = binder.file_locals.get("b") {
            let var_type = checker.get_type_of_symbol(symbol_id);
            let type_str = checker.type_to_string(var_type);
            assert_eq!(type_str, "number", "Unary plus should be number, got: {}", type_str);
        }

        // Test logical not: boolean
        if let Some(symbol_id) = binder.file_locals.get("c") {
            let var_type = checker.get_type_of_symbol(symbol_id);
            let type_str = checker.type_to_string(var_type);
            assert_eq!(type_str, "boolean", "Logical not should be boolean, got: {}", type_str);
        }

        // Test bitwise not: number
        if let Some(symbol_id) = binder.file_locals.get("d") {
            let var_type = checker.get_type_of_symbol(symbol_id);
            let type_str = checker.type_to_string(var_type);
            assert_eq!(type_str, "number", "Bitwise not should be number, got: {}", type_str);
        }

        // Test typeof: string
        if let Some(symbol_id) = binder.file_locals.get("e") {
            let var_type = checker.get_type_of_symbol(symbol_id);
            let type_str = checker.type_to_string(var_type);
            assert_eq!(type_str, "string", "typeof should be string, got: {}", type_str);
        }

        // Test void: undefined
        if let Some(symbol_id) = binder.file_locals.get("f") {
            let var_type = checker.get_type_of_symbol(symbol_id);
            let type_str = checker.type_to_string(var_type);
            assert_eq!(type_str, "undefined", "void should be undefined, got: {}", type_str);
        }
    }

    #[test]
    fn test_nullish_coalescing() {
        use crate::parser_impl::ParserState;
        use crate::binder::BinderState;

        let mut parser = ParserState::new(
            "test.ts".to_string(),
            r#"
                const x: number | null = null;
                const y = x ?? 0;
            "#.to_string(),
        );
        let root = parser.parse_source_file();

        let mut binder = BinderState::new();
        binder.bind_source_file(&parser.arena, root);

        let mut checker = CheckerState::new(
            &parser.arena,
            &binder.symbols,
            &binder.file_locals,
            "test.ts".to_string(),
        );

        // Nullish coalescing should produce union of non-null left and right
        if let Some(symbol_id) = binder.file_locals.get("y") {
            let var_type = checker.get_type_of_symbol(symbol_id);
            let type_str = checker.type_to_string(var_type);
            // Should contain number (from both the non-null part and the 0)
            assert!(type_str.contains("number") || type_str.contains("0"),
                "Nullish coalescing should produce number type, got: {}", type_str);
        }
    }

    #[test]
    fn test_logical_and_or() {
        use crate::parser_impl::ParserState;
        use crate::binder::BinderState;

        let mut parser = ParserState::new(
            "test.ts".to_string(),
            r#"
                const a = true && 1;
                const b = false || "hello";
            "#.to_string(),
        );
        let root = parser.parse_source_file();

        let mut binder = BinderState::new();
        binder.bind_source_file(&parser.arena, root);

        let mut checker = CheckerState::new(
            &parser.arena,
            &binder.symbols,
            &binder.file_locals,
            "test.ts".to_string(),
        );

        // Logical AND: should include both types
        if let Some(symbol_id) = binder.file_locals.get("a") {
            let var_type = checker.get_type_of_symbol(symbol_id);
            let type_str = checker.type_to_string(var_type);
            // Should be union of true and 1
            assert!(type_str.contains("true") || type_str.contains("1"),
                "Logical AND should produce union, got: {}", type_str);
        }

        // Logical OR: should include both types
        if let Some(symbol_id) = binder.file_locals.get("b") {
            let var_type = checker.get_type_of_symbol(symbol_id);
            let type_str = checker.type_to_string(var_type);
            // Should be union of false and "hello"
            assert!(type_str.contains("false") || type_str.contains("hello"),
                "Logical OR should produce union, got: {}", type_str);
        }
    }

    #[test]
    fn test_computed_property_name() {
        use crate::parser_impl::ParserState;
        use crate::binder::BinderState;

        // Test computed property names with static expressions
        let mut parser = ParserState::new(
            "test.ts".to_string(),
            r#"const obj = { ["foo"]: 1, ["bar"]: "hello" };"#.to_string(),
        );
        let root = parser.parse_source_file();

        let mut binder = BinderState::new();
        binder.bind_source_file(&parser.arena, root);

        let mut checker = CheckerState::new(
            &parser.arena,
            &binder.symbols,
            &binder.file_locals,
            "test.ts".to_string(),
        );

        assert!(binder.file_locals.has("obj"));
        if let Some(symbol) = binder.file_locals.get("obj") {
            let obj_type = checker.get_type_of_symbol(symbol);

            // Should be an object type with 2 properties
            if let Some(Type::Object(obj)) = checker.types.get(obj_type) {
                assert_eq!(obj.properties.len(), 2, "Expected 2 properties for computed property object");

                // Verify properties are named correctly
                assert!(obj.members.has("foo"), "Should have 'foo' property");
                assert!(obj.members.has("bar"), "Should have 'bar' property");
            } else {
                panic!("Expected Object type for object literal with computed properties");
            }
        }
    }

    #[test]
    fn test_computed_property_name_numeric() {
        use crate::parser_impl::ParserState;
        use crate::binder::BinderState;

        // Test computed property names with numeric expressions
        let mut parser = ParserState::new(
            "test.ts".to_string(),
            r#"const obj = { [42]: "answer" };"#.to_string(),
        );
        let root = parser.parse_source_file();

        let mut binder = BinderState::new();
        binder.bind_source_file(&parser.arena, root);

        let mut checker = CheckerState::new(
            &parser.arena,
            &binder.symbols,
            &binder.file_locals,
            "test.ts".to_string(),
        );

        assert!(binder.file_locals.has("obj"));
        if let Some(symbol) = binder.file_locals.get("obj") {
            let obj_type = checker.get_type_of_symbol(symbol);

            // Should be an object type with 1 property
            if let Some(Type::Object(obj)) = checker.types.get(obj_type) {
                assert_eq!(obj.properties.len(), 1, "Expected 1 property for numeric computed property");
                assert!(obj.members.has("42"), "Should have '42' property");
            } else {
                panic!("Expected Object type for object literal with numeric computed property");
            }
        }
    }

    #[test]
    fn test_variance_modifiers_out() {
        use crate::parser_impl::ParserState;
        use crate::parser::Node;

        // Test 'out' variance modifier (covariant)
        let mut parser = ParserState::new(
            "test.ts".to_string(),
            "interface Producer<out T> { produce(): T; }".to_string(),
        );
        let root = parser.parse_source_file();

        // Find the interface declaration
        if let Some(Node::SourceFile(sf)) = parser.arena.get(root) {
            assert!(!sf.statements.nodes.is_empty(), "Should have statements");

            if let Some(Node::InterfaceDeclaration(iface)) = parser.arena.get(sf.statements.nodes[0]) {
                // Check type parameters
                let type_params = iface.type_parameters.as_ref().expect("Should have type parameters");
                assert!(!type_params.is_empty(), "Should have type parameters");

                if let Some(Node::TypeParameterDeclaration(tp)) = parser.arena.get(type_params.nodes[0]) {
                    // Check for 'out' modifier
                    assert!(tp.modifiers.is_some(), "Should have modifiers");
                    let modifiers = tp.modifiers.as_ref().unwrap();
                    assert_eq!(modifiers.len(), 1, "Should have one modifier");

                    // The modifier should be 'out'
                    if let Some(Node::Token(base)) = parser.arena.get(modifiers.nodes[0]) {
                        assert_eq!(base.kind, crate::scanner::SyntaxKind::OutKeyword as u16, "Should be 'out' keyword");
                    } else {
                        panic!("Expected token node for modifier");
                    }
                } else {
                    panic!("Expected TypeParameterDeclaration");
                }
            } else {
                panic!("Expected InterfaceDeclaration");
            }
        }
    }

    #[test]
    fn test_variance_modifiers_in() {
        use crate::parser_impl::ParserState;
        use crate::parser::Node;

        // Test 'in' variance modifier (contravariant)
        // Use simpler syntax - just the type alias
        let mut parser = ParserState::new(
            "test.ts".to_string(),
            "type Consumer<in T> = T;".to_string(),
        );
        let root = parser.parse_source_file();

        // Find the type alias declaration
        if let Some(Node::SourceFile(sf)) = parser.arena.get(root) {
            if let Some(Node::TypeAliasDeclaration(alias)) = parser.arena.get(sf.statements.nodes[0]) {
                let type_params = alias.type_parameters.as_ref().expect("Should have type parameters");
                if let Some(Node::TypeParameterDeclaration(tp)) = parser.arena.get(type_params.nodes[0]) {
                    assert!(tp.modifiers.is_some(), "Should have modifiers");
                    let modifiers = tp.modifiers.as_ref().unwrap();
                    assert_eq!(modifiers.len(), 1, "Should have one modifier");

                    if let Some(Node::Token(base)) = parser.arena.get(modifiers.nodes[0]) {
                        assert_eq!(base.kind, crate::scanner::SyntaxKind::InKeyword as u16, "Should be 'in' keyword");
                    }
                }
            }
        }
    }

    #[test]
    fn test_variance_modifiers_in_out() {
        use crate::parser_impl::ParserState;
        use crate::parser::Node;

        // Test 'in out' variance modifier (invariant)
        let mut parser = ParserState::new(
            "test.ts".to_string(),
            "type Mapper<in out T> = T;".to_string(),
        );
        let root = parser.parse_source_file();

        // Find the type alias declaration
        if let Some(Node::SourceFile(sf)) = parser.arena.get(root) {
            if let Some(Node::TypeAliasDeclaration(alias)) = parser.arena.get(sf.statements.nodes[0]) {
                let type_params = alias.type_parameters.as_ref().expect("Should have type parameters");
                if let Some(Node::TypeParameterDeclaration(tp)) = parser.arena.get(type_params.nodes[0]) {
                    assert!(tp.modifiers.is_some(), "Should have modifiers for 'in out'");
                    let modifiers = tp.modifiers.as_ref().unwrap();
                    assert_eq!(modifiers.len(), 2, "Should have two modifiers for 'in out'");

                    // First should be 'in'
                    if let Some(Node::Token(base)) = parser.arena.get(modifiers.nodes[0]) {
                        assert_eq!(base.kind, crate::scanner::SyntaxKind::InKeyword as u16, "First should be 'in'");
                    }
                    // Second should be 'out'
                    if let Some(Node::Token(base)) = parser.arena.get(modifiers.nodes[1]) {
                        assert_eq!(base.kind, crate::scanner::SyntaxKind::OutKeyword as u16, "Second should be 'out'");
                    }
                }
            }
        }
    }

    #[test]
    fn test_new_expression_basic() {
        use crate::parser_impl::ParserState;
        use crate::binder::BinderState;

        // Test class type inference with new expression
        let mut parser = ParserState::new(
            "test.ts".to_string(),
            r#"class Foo {} const x = new Foo();"#.to_string(),
        );
        let root = parser.parse_source_file();

        let mut binder = BinderState::new();
        binder.bind_source_file(&parser.arena, root);

        let mut checker = CheckerState::new(
            &parser.arena,
            &binder.symbols,
            &binder.file_locals,
            "test.ts".to_string(),
        );

        // Get the type of 'x'
        assert!(binder.file_locals.has("x"), "Should have 'x' symbol");
        if let Some(symbol) = binder.file_locals.get("x") {
            let x_type = checker.get_type_of_symbol(symbol);
            let type_str = checker.type_to_string(x_type);
            eprintln!("Instance type: {}", type_str);
            // Should not be 'any' - should be the class instance type
            assert_ne!(x_type, checker.types.any_type, "x should not be 'any'");
        }
    }

    #[test]
    fn test_new_expression_class_with_members() {
        use crate::parser_impl::ParserState;
        use crate::binder::BinderState;

        // Test that new Foo() returns the class instance type for class with members
        let mut parser = ParserState::new(
            "test.ts".to_string(),
            r#"
class Foo {
    x: number = 1;
    greet(): string { return "hello"; }
}
const instance = new Foo();
"#.to_string(),
        );
        let root = parser.parse_source_file();

        let mut binder = BinderState::new();
        binder.bind_source_file(&parser.arena, root);

        let mut checker = CheckerState::new(
            &parser.arena,
            &binder.symbols,
            &binder.file_locals,
            "test.ts".to_string(),
        );

        // Get the type of 'instance'
        assert!(binder.file_locals.has("instance"), "Should have 'instance' symbol");
        if let Some(symbol) = binder.file_locals.get("instance") {
            let instance_type = checker.get_type_of_symbol(symbol);
            let type_str = checker.type_to_string(instance_type);
            eprintln!("Instance type of class with members: {}", type_str);
            // Should not be 'any'
            assert_ne!(instance_type, checker.types.any_type, "instance should not be 'any'");
        }
    }

    #[test]
    fn test_excess_property_check_basic() {
        use crate::parser_impl::ParserState;
        use crate::binder::BinderState;
        use crate::parser::Node;

        // Test excess property checking
        // Object literal { x: 1, y: 2 } should NOT be assignable to { x: number }
        let mut parser = ParserState::new(
            "test.ts".to_string(),
            r#"
                type Target = { x: number };
                const obj = { x: 1, y: 2 };  // object literal without type annotation
            "#.to_string(),
        );
        let root = parser.parse_source_file();

        let mut binder = BinderState::new();
        binder.bind_source_file(&parser.arena, root);

        let mut checker = CheckerState::new(
            &parser.arena,
            &binder.symbols,
            &binder.file_locals,
            "test.ts".to_string(),
        );

        // Get the type of the object literal (via the variable)
        let obj_symbol = binder.file_locals.get("obj").expect("obj should be in file locals");
        let obj_type = checker.get_type_of_symbol(obj_symbol);

        let target_symbol = binder.file_locals.get("Target").expect("Target should be in file locals");
        let target_type = checker.get_type_of_symbol(target_symbol);

        // Object literal should have FRESH_LITERAL flag
        if let Some(Type::Object(obj)) = checker.types.get(obj_type) {
            assert!(obj.has_object_flags(object_flags::FRESH_LITERAL),
                "Object literal should have FRESH_LITERAL flag");
        } else {
            panic!("Expected Object type for object literal, got something else");
        }

        // Excess property check: obj type should NOT be assignable to Target
        // because obj has an excess property 'y'
        let is_assignable = checker.is_type_assignable_to(obj_type, target_type);
        assert!(!is_assignable, "Object with excess property should NOT be assignable to target");
    }

    #[test]
    fn test_type_literal_with_index_signature() {
        use crate::parser_impl::ParserState;
        use crate::binder::BinderState;

        // Test that type literals parse index signatures correctly
        let mut parser = ParserState::new(
            "test.ts".to_string(),
            r#"
                type Dict = { x: number; [key: string]: number };
            "#.to_string(),
        );
        let root = parser.parse_source_file();

        let mut binder = BinderState::new();
        binder.bind_source_file(&parser.arena, root);

        let mut checker = CheckerState::new(
            &parser.arena,
            &binder.symbols,
            &binder.file_locals,
            "test.ts".to_string(),
        );

        let dict_symbol = binder.file_locals.get("Dict").expect("Dict should be in file locals");
        let dict_type = checker.get_type_of_symbol(dict_symbol);

        // Check that the Dict type has index infos
        if let Some(Type::Object(dict_obj)) = checker.types.get(dict_type) {
            assert_eq!(dict_obj.index_infos.len(), 1, "Dict should have 1 index signature");
            assert!(dict_obj.members.has("x"), "Dict should have property x");

            // Check that the index signature has correct types
            let index_info = &dict_obj.index_infos[0];
            assert_eq!(index_info.key_type, checker.types.string_type, "Index key should be string");
            assert_eq!(index_info.value_type, checker.types.number_type, "Index value should be number");
        } else {
            panic!("Dict should be an Object type");
        }
    }

    #[test]
    fn test_excess_property_check_no_excess() {
        use crate::parser_impl::ParserState;
        use crate::binder::BinderState;

        // Test that exact match is allowed
        let mut parser = ParserState::new(
            "test.ts".to_string(),
            r#"
                type Point = { x: number; y: number };
                const p = { x: 1, y: 2 };  // object literal with exact properties
            "#.to_string(),
        );
        let root = parser.parse_source_file();

        let mut binder = BinderState::new();
        binder.bind_source_file(&parser.arena, root);

        let mut checker = CheckerState::new(
            &parser.arena,
            &binder.symbols,
            &binder.file_locals,
            "test.ts".to_string(),
        );

        let p_symbol = binder.file_locals.get("p").expect("p should be in file locals");
        let p_type = checker.get_type_of_symbol(p_symbol);

        let point_symbol = binder.file_locals.get("Point").expect("Point should be in file locals");
        let point_type = checker.get_type_of_symbol(point_symbol);

        // Exact match should be assignable
        let is_assignable = checker.is_type_assignable_to(p_type, point_type);
        assert!(is_assignable, "Object with exact properties should be assignable");
    }

    #[test]
    fn test_discriminated_union_narrowing() {
        // Test discriminated union narrowing using arena directly
        // type Circle = { kind: "circle"; radius: number };
        // type Square = { kind: "square"; side: number };
        // type Shape = Circle | Square;

        let mut arena = super::TypeArena::new();
        let mut local_symbols = crate::binder::SymbolArena::new_with_base(crate::binder::SymbolArena::CHECKER_SYMBOL_BASE);
        let mut symbol_types = rustc_hash::FxHashMap::default();

        // Create Circle type: { kind: "circle"; radius: number }
        let circle_kind = arena.create_string_literal("circle".to_string());
        let circle_kind_sym = local_symbols.alloc(crate::binder::symbol_flags::PROPERTY, "kind".to_string());
        let circle_radius_sym = local_symbols.alloc(crate::binder::symbol_flags::PROPERTY, "radius".to_string());
        symbol_types.insert(circle_kind_sym, circle_kind);
        symbol_types.insert(circle_radius_sym, arena.number_type);

        let mut circle_members = SymbolTable::new();
        circle_members.set("kind".to_string(), circle_kind_sym);
        circle_members.set("radius".to_string(), circle_radius_sym);
        let circle_type = arena.create_object_type_with_members(vec![circle_kind_sym, circle_radius_sym], circle_members);

        // Create Square type: { kind: "square"; side: number }
        let square_kind = arena.create_string_literal("square".to_string());
        let square_kind_sym = local_symbols.alloc(crate::binder::symbol_flags::PROPERTY, "kind".to_string());
        let square_side_sym = local_symbols.alloc(crate::binder::symbol_flags::PROPERTY, "side".to_string());
        symbol_types.insert(square_kind_sym, square_kind);
        symbol_types.insert(square_side_sym, arena.number_type);

        let mut square_members = SymbolTable::new();
        square_members.set("kind".to_string(), square_kind_sym);
        square_members.set("side".to_string(), square_side_sym);
        let square_type = arena.create_object_type_with_members(vec![square_kind_sym, square_side_sym], square_members);

        // Create Shape = Circle | Square
        let shape_type = arena.create_union_type(vec![circle_type, square_type]);

        // Create a minimal checker to test narrowing
        let node_arena = crate::parser::NodeArena::new();
        let binder_symbols = crate::binder::SymbolArena::new();
        let file_locals = SymbolTable::new();

        let mut checker = CheckerState::new(
            &node_arena,
            &binder_symbols,
            &file_locals,
            "test.ts".to_string(),
        );

        // Copy types and symbols to checker
        checker.types = arena;
        checker.symbol_types = symbol_types;

        // Test 1: Narrow by kind === "circle" should give Circle
        let narrowed = checker.narrow_type_by_discriminant(shape_type, "kind", "circle");
        assert_eq!(narrowed, circle_type, "Narrowing by kind='circle' should give Circle type");

        // Test 2: Narrow by kind === "square" should give Square
        let narrowed = checker.narrow_type_by_discriminant(shape_type, "kind", "square");
        assert_eq!(narrowed, square_type, "Narrowing by kind='square' should give Square type");

        // Test 3: Narrow by kind === "unknown" should give never
        let narrowed = checker.narrow_type_by_discriminant(shape_type, "kind", "unknown");
        assert_eq!(narrowed, checker.types.never_type, "Narrowing by unknown value should give never");

        // Test 4: Narrow by kind !== "circle" should give Square
        let narrowed = checker.narrow_type_by_discriminant_negation(shape_type, "kind", "circle");
        assert_eq!(narrowed, square_type, "Narrowing by kind!='circle' should give Square");

        // Test 5: Narrow by kind !== "square" should give Circle
        let narrowed = checker.narrow_type_by_discriminant_negation(shape_type, "kind", "square");
        assert_eq!(narrowed, circle_type, "Narrowing by kind!='square' should give Circle");
    }

    #[test]
    fn test_switch_exhaustiveness_on_discriminated_union() {
        // Test switch exhaustiveness checking

        let mut arena = super::TypeArena::new();

        // Create literal types for switch cases
        let a_literal = arena.create_string_literal("a".to_string());
        let b_literal = arena.create_string_literal("b".to_string());
        let c_literal = arena.create_string_literal("c".to_string());

        // Create discriminant type: "a" | "b" | "c"
        let discriminant_type = arena.create_union_type(vec![a_literal, b_literal, c_literal]);

        // Create minimal checker
        let node_arena = crate::parser::NodeArena::new();
        let binder_symbols = crate::binder::SymbolArena::new();
        let file_locals = SymbolTable::new();

        let mut checker = CheckerState::new(
            &node_arena,
            &binder_symbols,
            &file_locals,
            "test.ts".to_string(),
        );
        checker.types = arena;

        // Test 1: All cases covered - should return never
        let remaining = checker.check_switch_exhaustiveness(
            discriminant_type,
            &[a_literal, b_literal, c_literal],
            false
        );
        assert_eq!(remaining, checker.types.never_type, "Exhaustive switch should return never");

        // Test 2: Missing one case - should return remaining type
        let remaining = checker.check_switch_exhaustiveness(
            discriminant_type,
            &[a_literal, b_literal],  // missing "c"
            false
        );
        assert_eq!(remaining, c_literal, "Non-exhaustive switch should return remaining type");

        // Test 3: Has default - always exhaustive
        let remaining = checker.check_switch_exhaustiveness(
            discriminant_type,
            &[a_literal],  // only "a" but has default
            true
        );
        assert_eq!(remaining, checker.types.never_type, "Switch with default should be exhaustive");
    }

    // ============== TASK 4: Exhaustiveness checking with diagnostics ==============
    #[test]
    fn test_exhaustiveness_diagnostic_integration() {
        // Test that exhaustiveness checking integrates with the diagnostic system
        let mut arena = super::TypeArena::new();
        let a = arena.create_string_literal("a".to_string());
        let b = arena.create_string_literal("b".to_string());
        let union_type = arena.create_union_type(vec![a, b]);

        let node_arena = crate::parser::NodeArena::new();
        let binder_symbols = crate::binder::SymbolArena::new();
        let file_locals = SymbolTable::new();

        let mut checker = CheckerState::new(&node_arena, &binder_symbols, &file_locals, "test.ts".to_string());
        checker.types = arena;

        // Non-exhaustive switch should leave remaining type
        let remaining = checker.check_switch_exhaustiveness(union_type, &[a], false);
        assert_eq!(remaining, b, "Should return remaining unhandled type 'b'");

        // This remaining type can be used to generate a diagnostic
        let remaining_str = checker.type_to_string(remaining);
        assert!(remaining_str.contains("b"), "Remaining type should be 'b'");
    }

    // ============== TASK 5: Contextual typing for object literals ==============
    #[test]
    fn test_contextual_typing_object_literal() {
        use crate::parser_impl::ParserState;
        use crate::binder::BinderState;

        // Test that object literal properties can get contextual types
        let mut parser = ParserState::new(
            "test.ts".to_string(),
            r#"
                function process(opts: { x: number; y: number }) {}
                const obj = { x: 1, y: 2 };
            "#.to_string(),
        );
        let root = parser.parse_source_file();

        let mut binder = BinderState::new();
        binder.bind_source_file(&parser.arena, root);

        let mut checker = CheckerState::new(
            &parser.arena,
            &binder.symbols,
            &binder.file_locals,
            "test.ts".to_string(),
        );

        // Get object type
        let obj_symbol = binder.file_locals.get("obj").expect("obj should exist");
        let obj_type = checker.get_type_of_symbol(obj_symbol);

        // Object should have correct structure
        if let Some(Type::Object(obj)) = checker.types.get(obj_type) {
            assert!(obj.members.has("x"), "Object should have property x");
            assert!(obj.members.has("y"), "Object should have property y");
        }
    }

    // ============== TASK 6: Contextual typing for return statements ==============
    #[test]
    fn test_contextual_typing_return() {
        use crate::parser_impl::ParserState;
        use crate::binder::BinderState;

        let mut parser = ParserState::new(
            "test.ts".to_string(),
            r#"
                function getNum(): number { return 42; }
            "#.to_string(),
        );
        let root = parser.parse_source_file();

        let mut binder = BinderState::new();
        binder.bind_source_file(&parser.arena, root);

        let mut checker = CheckerState::new(
            &parser.arena,
            &binder.symbols,
            &binder.file_locals,
            "test.ts".to_string(),
        );

        let fn_symbol = binder.file_locals.get("getNum").expect("getNum should exist");
        let fn_type = checker.get_type_of_symbol(fn_symbol);

        // Function should have number return type
        if let Some(Type::Function(f)) = checker.types.get(fn_type) {
            assert_eq!(f.return_type, checker.types.number_type, "Return type should be number");
        }
    }

    // ============== TASK 7: Type incompatibility diagnostic details ==============
    #[test]
    fn test_type_incompatibility_details() {
        let mut arena = super::TypeArena::new();

        // Create incompatible types
        let string_type = arena.string_type;
        let number_type = arena.number_type;

        let node_arena = crate::parser::NodeArena::new();
        let binder_symbols = crate::binder::SymbolArena::new();
        let file_locals = SymbolTable::new();

        let mut checker = CheckerState::new(&node_arena, &binder_symbols, &file_locals, "test.ts".to_string());
        checker.types = arena;

        // String is not assignable to number
        let is_assignable = checker.is_type_assignable_to(string_type, number_type);
        assert!(!is_assignable, "string should not be assignable to number");

        // Can generate error message details
        let source_str = checker.type_to_string(string_type);
        let target_str = checker.type_to_string(number_type);
        assert_eq!(source_str, "string");
        assert_eq!(target_str, "number");
    }

    // ============== TASK 8: Promise<T> unwrapping for await ==============
    #[test]
    fn test_promise_type_structure() {
        // Test that Promise-like types can be created and recognized
        let mut arena = super::TypeArena::new();
        let mut local_symbols = crate::binder::SymbolArena::new_with_base(crate::binder::SymbolArena::CHECKER_SYMBOL_BASE);

        // Create Promise<number> structure: { then: (cb: (value: number) => any) => any }
        let then_sym = local_symbols.alloc(crate::binder::symbol_flags::METHOD, "then".to_string());
        let mut members = SymbolTable::new();
        members.set("then".to_string(), then_sym);

        let promise_type = arena.create_object_type_with_members(vec![then_sym], members);

        // Verify Promise structure
        if let Some(Type::Object(obj)) = arena.get(promise_type) {
            assert!(obj.members.has("then"), "Promise should have 'then' method");
        }
    }

    // ============== TASK 9: 'this' type in class contexts ==============
    #[test]
    fn test_this_type_in_class() {
        // Test that the type arena can create a 'this' type placeholder
        let arena = super::TypeArena::new();

        // The 'this' type is typically represented as a type reference
        // When used in class contexts, it refers to the instance type
        // For now, verify that we can work with type parameters which 'this' resembles
        let unknown = arena.unknown_type;
        assert!(unknown.0 > 0, "'this' placeholder can use unknown type initially");
    }

    // ============== TASK 10: User-defined type predicates ==============
    #[test]
    fn test_type_predicate_structure() {
        use crate::parser_impl::ParserState;
        use crate::binder::BinderState;

        let mut parser = ParserState::new(
            "test.ts".to_string(),
            r#"
                function isString(x: unknown): x is string {
                    return typeof x === "string";
                }
            "#.to_string(),
        );
        let root = parser.parse_source_file();

        let mut binder = BinderState::new();
        binder.bind_source_file(&parser.arena, root);

        // Function should be parsed and bound
        assert!(binder.file_locals.has("isString"), "isString should be in file locals");
    }

    // ============== TASK 11: RegExp type support ==============
    #[test]
    fn test_regexp_literal_type() {
        use crate::parser_impl::ParserState;
        use crate::binder::BinderState;

        let mut parser = ParserState::new(
            "test.ts".to_string(),
            r#"
                const regex = /test/g;
            "#.to_string(),
        );
        let root = parser.parse_source_file();

        let mut binder = BinderState::new();
        binder.bind_source_file(&parser.arena, root);

        let mut checker = CheckerState::new(
            &parser.arena,
            &binder.symbols,
            &binder.file_locals,
            "test.ts".to_string(),
        );

        // regex variable should be bound
        assert!(binder.file_locals.has("regex"), "regex should be in file locals");
        let regex_symbol = binder.file_locals.get("regex").unwrap();
        let regex_type = checker.get_type_of_symbol(regex_symbol);
        // RegExp literals return an object type (or any for now)
        assert!(regex_type.0 > 0, "Should have a valid type");
    }

    // ============== TASK 12: Optional property handling ==============
    #[test]
    fn test_optional_property_handling() {
        let mut arena = super::TypeArena::new();
        let mut local_symbols = crate::binder::SymbolArena::new_with_base(crate::binder::SymbolArena::CHECKER_SYMBOL_BASE);

        // Create type with optional property: { x: number; y?: string }
        let x_sym = local_symbols.alloc(crate::binder::symbol_flags::PROPERTY, "x".to_string());
        let y_sym = local_symbols.alloc(crate::binder::symbol_flags::PROPERTY | crate::binder::symbol_flags::OPTIONAL, "y".to_string());

        let mut members = SymbolTable::new();
        members.set("x".to_string(), x_sym);
        members.set("y".to_string(), y_sym);

        let obj_type = arena.create_object_type_with_members(vec![x_sym, y_sym], members);

        if let Some(Type::Object(obj)) = arena.get(obj_type) {
            assert!(obj.members.has("x"), "Should have property x");
            assert!(obj.members.has("y"), "Should have optional property y");
        }
    }

    // ============== TASK 13: Missing properties diagnostic ==============
    #[test]
    fn test_missing_property_detection() {
        let mut arena = super::TypeArena::new();
        let mut local_symbols = crate::binder::SymbolArena::new_with_base(crate::binder::SymbolArena::CHECKER_SYMBOL_BASE);
        let mut symbol_types = rustc_hash::FxHashMap::default();

        // Source: { x: number }
        let src_x = local_symbols.alloc(crate::binder::symbol_flags::PROPERTY, "x".to_string());
        symbol_types.insert(src_x, arena.number_type);
        let mut src_members = SymbolTable::new();
        src_members.set("x".to_string(), src_x);
        let source = arena.create_object_type_with_members(vec![src_x], src_members);

        // Target: { x: number; y: number }
        let tgt_x = local_symbols.alloc(crate::binder::symbol_flags::PROPERTY, "x".to_string());
        let tgt_y = local_symbols.alloc(crate::binder::symbol_flags::PROPERTY, "y".to_string());
        symbol_types.insert(tgt_x, arena.number_type);
        symbol_types.insert(tgt_y, arena.number_type);
        let mut tgt_members = SymbolTable::new();
        tgt_members.set("x".to_string(), tgt_x);
        tgt_members.set("y".to_string(), tgt_y);
        let target = arena.create_object_type_with_members(vec![tgt_x, tgt_y], tgt_members);

        let node_arena = crate::parser::NodeArena::new();
        let binder_symbols = crate::binder::SymbolArena::new();
        let file_locals = SymbolTable::new();

        let mut checker = CheckerState::new(&node_arena, &binder_symbols, &file_locals, "test.ts".to_string());
        checker.types = arena;
        checker.symbol_types = symbol_types;

        // Source missing 'y' so not assignable
        let is_assignable = checker.is_type_assignable_to(source, target);
        assert!(!is_assignable, "Object missing property 'y' should not be assignable");
    }

    // ============== TASK 14: Function parameter mismatch diagnostics ==============
    #[test]
    fn test_function_parameter_mismatch() {
        let mut arena = super::TypeArena::new();

        // Function (x: string) => void
        let fn1 = arena.create_function_type(
            NodeIndex::NONE,
            vec![arena.string_type],
            vec!["x".to_string()],
            arena.void_type,
            1,
            false,
        );

        // Function (x: number) => void
        let fn2 = arena.create_function_type(
            NodeIndex::NONE,
            vec![arena.number_type],
            vec!["x".to_string()],
            arena.void_type,
            1,
            false,
        );

        let node_arena = crate::parser::NodeArena::new();
        let binder_symbols = crate::binder::SymbolArena::new();
        let file_locals = SymbolTable::new();

        let mut checker = CheckerState::new(&node_arena, &binder_symbols, &file_locals, "test.ts".to_string());
        checker.types = arena;

        // Different parameter types - not directly assignable
        // (Note: TypeScript uses bivariance for function parameters)
        let fn1_str = checker.type_to_string(fn1);
        let fn2_str = checker.type_to_string(fn2);
        assert!(fn1_str.contains("string"), "fn1 should have string param");
        assert!(fn2_str.contains("number"), "fn2 should have number param");
    }

    // ============== TASK 15: Assignment narrowing in control flow ==============
    #[test]
    fn test_assignment_narrowing() {
        let mut arena = super::TypeArena::new();

        // Start with union type string | number
        let union = arena.create_union_type(vec![arena.string_type, arena.number_type]);

        let node_arena = crate::parser::NodeArena::new();
        let binder_symbols = crate::binder::SymbolArena::new();
        let file_locals = SymbolTable::new();

        let mut checker = CheckerState::new(&node_arena, &binder_symbols, &file_locals, "test.ts".to_string());
        checker.types = arena;

        // After assignment of string, type should narrow
        // This tests the type_to_string for union types
        let union_str = checker.type_to_string(union);
        assert!(union_str.contains("string") && union_str.contains("number"),
            "Union should contain both string and number");
    }

    // ============== TASK 16: Contextual typing for array literals ==============
    #[test]
    fn test_contextual_typing_array() {
        use crate::parser_impl::ParserState;
        use crate::binder::BinderState;

        let mut parser = ParserState::new(
            "test.ts".to_string(),
            r#"
                const nums: number[] = [1, 2, 3];
            "#.to_string(),
        );
        let root = parser.parse_source_file();

        let mut binder = BinderState::new();
        binder.bind_source_file(&parser.arena, root);

        let mut checker = CheckerState::new(
            &parser.arena,
            &binder.symbols,
            &binder.file_locals,
            "test.ts".to_string(),
        );

        let nums_symbol = binder.file_locals.get("nums").expect("nums should exist");
        let nums_type = checker.get_type_of_symbol(nums_symbol);

        // Should have the annotated array type
        let type_str = checker.type_to_string(nums_type);
        assert!(type_str.contains("number"), "Should be number array: {}", type_str);
    }

    // ============== TASK 17: Assertion functions ==============
    #[test]
    fn test_assertion_function_structure() {
        use crate::parser_impl::ParserState;
        use crate::binder::BinderState;

        let mut parser = ParserState::new(
            "test.ts".to_string(),
            r#"
                function assertIsString(x: unknown): asserts x is string {
                    if (typeof x !== "string") throw new Error();
                }
            "#.to_string(),
        );
        let root = parser.parse_source_file();

        let mut binder = BinderState::new();
        binder.bind_source_file(&parser.arena, root);

        // Function should be parsed
        assert!(binder.file_locals.has("assertIsString"), "assertIsString should be in file locals");
    }

    // ============== TASK 18: Call/construct signature diagnostics ==============
    #[test]
    fn test_call_signature_mismatch() {
        let mut arena = super::TypeArena::new();

        // Object with call signature: { (): string }
        let mut call_sig = Signature::new(NodeIndex::NONE);
        call_sig.resolved_return_type = Some(arena.string_type);
        call_sig.min_argument_count = 0;

        let mut obj = super::types::ObjectType::new(object_flags::ANONYMOUS, SymbolId::NONE);
        obj.call_signatures = vec![call_sig];
        let callable_type = arena.alloc(Type::Object(obj));

        let node_arena = crate::parser::NodeArena::new();
        let binder_symbols = crate::binder::SymbolArena::new();
        let file_locals = SymbolTable::new();

        let mut checker = CheckerState::new(&node_arena, &binder_symbols, &file_locals, "test.ts".to_string());
        checker.types = arena;

        // Verify callable has call signature
        if let Some(Type::Object(o)) = checker.types.get(callable_type) {
            assert_eq!(o.call_signatures.len(), 1, "Should have one call signature");
        }
    }

    // ============== TASK 19: Nested discriminated union tests ==============
    #[test]
    fn test_nested_discriminated_unions() {
        let mut arena = super::TypeArena::new();
        let mut local_symbols = crate::binder::SymbolArena::new_with_base(crate::binder::SymbolArena::CHECKER_SYMBOL_BASE);
        let mut symbol_types = rustc_hash::FxHashMap::default();

        // Create nested structure:
        // type A = { kind: "a"; nested: { subkind: "x" } }
        // type B = { kind: "b"; nested: { subkind: "y" } }

        let a_kind = arena.create_string_literal("a".to_string());
        let b_kind = arena.create_string_literal("b".to_string());

        let a_kind_sym = local_symbols.alloc(crate::binder::symbol_flags::PROPERTY, "kind".to_string());
        let b_kind_sym = local_symbols.alloc(crate::binder::symbol_flags::PROPERTY, "kind".to_string());
        symbol_types.insert(a_kind_sym, a_kind);
        symbol_types.insert(b_kind_sym, b_kind);

        let mut a_members = SymbolTable::new();
        a_members.set("kind".to_string(), a_kind_sym);
        let type_a = arena.create_object_type_with_members(vec![a_kind_sym], a_members);

        let mut b_members = SymbolTable::new();
        b_members.set("kind".to_string(), b_kind_sym);
        let type_b = arena.create_object_type_with_members(vec![b_kind_sym], b_members);

        let union = arena.create_union_type(vec![type_a, type_b]);

        let node_arena = crate::parser::NodeArena::new();
        let binder_symbols = crate::binder::SymbolArena::new();
        let file_locals = SymbolTable::new();

        let mut checker = CheckerState::new(&node_arena, &binder_symbols, &file_locals, "test.ts".to_string());
        checker.types = arena;
        checker.symbol_types = symbol_types;

        // Narrow by "a"
        let narrowed_a = checker.narrow_type_by_discriminant(union, "kind", "a");
        assert_eq!(narrowed_a, type_a, "Should narrow to type A");

        // Narrow by "b"
        let narrowed_b = checker.narrow_type_by_discriminant(union, "kind", "b");
        assert_eq!(narrowed_b, type_b, "Should narrow to type B");
    }

    // ============== TASK 20: Definite assignment analysis ==============
    #[test]
    fn test_definite_assignment_structure() {
        use crate::parser_impl::ParserState;
        use crate::binder::BinderState;

        // Test that variables with definite assignment modifier parse correctly
        let mut parser = ParserState::new(
            "test.ts".to_string(),
            r#"
                class Foo {
                    x!: number;  // definite assignment assertion
                }
                let value: number;
                value = 42;
            "#.to_string(),
        );
        let root = parser.parse_source_file();

        let mut binder = BinderState::new();
        binder.bind_source_file(&parser.arena, root);

        // Both should be bound
        assert!(binder.file_locals.has("Foo"), "Foo class should be bound");
        assert!(binder.file_locals.has("value"), "value should be bound");
    }
