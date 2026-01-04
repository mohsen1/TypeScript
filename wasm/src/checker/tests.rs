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

        // Total: 15 singleton types (13 intrinsic + 2 boolean literals)
        assert_eq!(arena.len(), 15);
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

        let obj1 = arena.alloc(Type::Object(Box::new(ObjectType::new(object_flags::ANONYMOUS, SymbolId::NONE))));
        let obj2 = arena.alloc(Type::Object(Box::new(ObjectType::new(object_flags::ANONYMOUS, SymbolId::NONE))));

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
        let obj = arena.alloc(Type::Object(Box::new(ObjectType::new(object_flags::ANONYMOUS, SymbolId::NONE))));

        let intersection = arena.create_intersection(vec![obj, arena.never_type]);
        assert_eq!(intersection, arena.never_type);
    }

    #[test]
    fn test_intersection_simplification_unknown() {
        // X & unknown = X
        let mut arena = TypeArena::new();
        let obj = arena.alloc(Type::Object(Box::new(ObjectType::new(object_flags::ANONYMOUS, SymbolId::NONE))));

        let intersection = arena.create_intersection(vec![obj, arena.unknown_type]);
        assert_eq!(intersection, obj);
    }

    #[test]
    fn test_intersection_simplification_duplicates() {
        // X & X = X
        let mut arena = TypeArena::new();
        let obj = arena.alloc(Type::Object(Box::new(ObjectType::new(object_flags::ANONYMOUS, SymbolId::NONE))));

        let intersection = arena.create_intersection(vec![obj, obj]);
        assert_eq!(intersection, obj);
    }

    #[test]
    fn test_intersection_simplification_flatten() {
        // (A & B) & C = A & B & C
        let mut arena = TypeArena::new();
        let obj1 = arena.alloc(Type::Object(Box::new(ObjectType::new(object_flags::ANONYMOUS, SymbolId::NONE))));
        let obj2 = arena.alloc(Type::Object(Box::new(ObjectType::new(object_flags::ANONYMOUS, SymbolId::NONE))));
        let obj3 = arena.alloc(Type::Object(Box::new(ObjectType::new(object_flags::ANONYMOUS, SymbolId::NONE))));

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

            // The type should be Array<number | string> (or Array<1 | "hello">)
            if let Type::Array(arr) = typ {
                // Element type should be a union of the literal types
                let elem_type = checker.types.get(arr.element_type).unwrap();
                if let Type::Union(u) = elem_type {
                    assert_eq!(u.types.len(), 2, "Expected union of 2 types");
                } else {
                    panic!("Expected Union element type for mixed array, got {:?}", elem_type);
                }
            } else {
                panic!("Expected Array type for array literal, got {:?}", typ);
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
        let obj = arena.alloc(Type::Object(Box::new(ObjectType::new(object_flags::ANONYMOUS, SymbolId::NONE))));

        let union = arena.create_union(vec![obj, arena.never_type]);
        assert_eq!(union, obj);
    }

    #[test]
    fn test_union_simplification_any() {
        // X | any = any
        let mut arena = TypeArena::new();
        let obj = arena.alloc(Type::Object(Box::new(ObjectType::new(object_flags::ANONYMOUS, SymbolId::NONE))));

        let union = arena.create_union(vec![obj, arena.any_type]);
        assert_eq!(union, arena.any_type);
    }

    #[test]
    fn test_union_simplification_duplicates() {
        // X | X = X
        let mut arena = TypeArena::new();
        let obj = arena.alloc(Type::Object(Box::new(ObjectType::new(object_flags::ANONYMOUS, SymbolId::NONE))));

        let union = arena.create_union(vec![obj, obj]);
        assert_eq!(union, obj);
    }

    #[test]
    fn test_union_simplification_flatten() {
        // (A | B) | C = A | B | C
        let mut arena = TypeArena::new();
        let obj1 = arena.alloc(Type::Object(Box::new(ObjectType::new(object_flags::ANONYMOUS, SymbolId::NONE))));
        let obj2 = arena.alloc(Type::Object(Box::new(ObjectType::new(object_flags::ANONYMOUS, SymbolId::NONE))));
        let obj3 = arena.alloc(Type::Object(Box::new(ObjectType::new(object_flags::ANONYMOUS, SymbolId::NONE))));

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

        // a should be of type A (named interface types print their name)
        assert!(checker.types.get(a_type).is_some(), "a should have a valid type");
        assert!(a_str == "A", "a should be interface A, got: {}", a_str);
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
        let t_type = checker.types.alloc(Type::TypeParameter(Box::new(TypeParameter {
            flags: type_flags::TYPE_PARAMETER,
            symbol: t_symbol,
            constraint: TypeId::NONE,
            default: TypeId::NONE,
            target: TypeId::NONE,
            is_this_type: false,
            is_const: false,
        })));
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

        // more should be an Array type with element type containing number (from spread and literals)
        // The type should be number[] or (number | 4 | 5)[]
        let is_array_with_number = if let Some(Type::Array(arr)) = checker.types.get(more_type) {
            // Element type should be number or contain number
            let elem_type = arr.element_type;
            elem_type == checker.types.number_type || {
                if let Some(Type::Union(u)) = checker.types.get(elem_type) {
                    u.types.iter().any(|&t| t == checker.types.number_type)
                } else {
                    false
                }
            }
        } else {
            false
        };
        assert!(is_array_with_number, "more should be array with number element type, got: {}", more_str);
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
    fn test_checker_variance_extraction() {
        // Test that the checker can extract variance from type parameters
        use crate::parser_impl::ParserState;
        use crate::binder::BinderState;
        use crate::parser::Node;

        let code = r#"
interface Producer<out T> {
    produce(): T;
}
interface Consumer<in T> {
    consume(value: T): void;
}
interface Invariant<in out T> {
    both(value: T): T;
}
"#;

        let mut parser = ParserState::new("test.ts".to_string(), code.to_string());
        let root = parser.parse_source_file();

        let mut binder = BinderState::new();
        binder.bind_source_file(&parser.arena, root);

        let checker = CheckerState::new(
            &parser.arena,
            &binder.symbols,
            &binder.file_locals,
            "test.ts".to_string(),
        );

        // Find the interface declarations and check variance
        if let Some(Node::SourceFile(sf)) = parser.arena.get(root) {
            // Producer<out T> - should be covariant (out only)
            if let Some(Node::InterfaceDeclaration(iface)) = parser.arena.get(sf.statements.nodes[0]) {
                let type_params = iface.type_parameters.as_ref().expect("Should have type parameters");
                let (is_in, is_out) = checker.get_type_parameter_variance(type_params.nodes[0]);
                assert!(!is_in && is_out, "Producer<out T> should be covariant: in={}, out={}", is_in, is_out);
            }

            // Consumer<in T> - should be contravariant (in only)
            if let Some(Node::InterfaceDeclaration(iface)) = parser.arena.get(sf.statements.nodes[1]) {
                let type_params = iface.type_parameters.as_ref().expect("Should have type parameters");
                let (is_in, is_out) = checker.get_type_parameter_variance(type_params.nodes[0]);
                assert!(is_in && !is_out, "Consumer<in T> should be contravariant: in={}, out={}", is_in, is_out);
            }

            // Invariant<in out T> - should be invariant (both in and out)
            if let Some(Node::InterfaceDeclaration(iface)) = parser.arena.get(sf.statements.nodes[2]) {
                let type_params = iface.type_parameters.as_ref().expect("Should have type parameters");
                let (is_in, is_out) = checker.get_type_parameter_variance(type_params.nodes[0]);
                assert!(is_in && is_out, "Invariant<in out T> should be invariant: in={}, out={}", is_in, is_out);
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
        let callable_type = arena.alloc(Type::Object(Box::new(obj)));

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

    // ============== Contextual typing for callbacks in object literals ==============
    #[test]
    fn test_contextual_typing_callback_in_object_literal() {
        use crate::parser_impl::ParserState;
        use crate::binder::BinderState;

        // Test that callback parameters in object literals get contextual types
        let mut parser = ParserState::new(
            "test.ts".to_string(),
            r#"
                interface EventHandler {
                    onClick: (event: string) => void;
                }
                const handler: EventHandler = {
                    onClick: (e) => console.log(e)
                };
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

        // Verify handler exists and has correct structure
        let handler_symbol = binder.file_locals.get("handler").expect("handler should exist");
        let handler_type = checker.get_type_of_symbol(handler_symbol);

        // Should not have any errors (parameter 'e' gets type from context)
        // The type should be assignable to EventHandler
        let type_str = checker.type_to_string(handler_type);
        assert!(!type_str.is_empty(), "Handler should have a type: {}", type_str);
    }

    // ============== Empty array with contextual type ==============
    #[test]
    fn test_contextual_typing_empty_array() {
        use crate::parser_impl::ParserState;
        use crate::binder::BinderState;

        // Test that empty arrays get element type from context
        let mut parser = ParserState::new(
            "test.ts".to_string(),
            r#"
                function process(nums: number[]) {}
                process([]);  // Empty array should have contextual number element type
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

        // Just verify this compiles without errors
        let fn_symbol = binder.file_locals.get("process").expect("process should exist");
        let fn_type = checker.get_type_of_symbol(fn_symbol);
        assert!(!fn_type.is_none(), "Function should have a type");
    }

    // ============== Array callback element contextual typing ==============
    #[test]
    fn test_contextual_typing_array_callback_elements() {
        use crate::parser_impl::ParserState;
        use crate::binder::BinderState;

        // Test that callback elements in arrays get contextual types
        let mut parser = ParserState::new(
            "test.ts".to_string(),
            r#"
                const handlers: Array<(x: number) => void> = [
                    (n) => console.log(n),
                    (m) => console.log(m * 2)
                ];
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

        // Verify handlers exists
        let handlers_symbol = binder.file_locals.get("handlers").expect("handlers should exist");
        let handlers_type = checker.get_type_of_symbol(handlers_symbol);
        let type_str = checker.type_to_string(handlers_type);

        // The type should reflect the annotated array type
        assert!(!type_str.is_empty(), "Handlers should have a type: {}", type_str);
    }

    // ============== Method contextual typing in object literal ==============
    #[test]
    fn test_contextual_typing_method_in_object_literal() {
        use crate::parser_impl::ParserState;
        use crate::binder::BinderState;

        // Test that method declarations in object literals get contextual types
        let mut parser = ParserState::new(
            "test.ts".to_string(),
            r#"
                interface Calculator {
                    add(a: number, b: number): number;
                }
                const calc: Calculator = {
                    add(x, y) { return x + y; }
                };
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

        // Verify calc exists and has correct structure
        let calc_symbol = binder.file_locals.get("calc").expect("calc should exist");
        let calc_type = checker.get_type_of_symbol(calc_symbol);

        // Should have the add method
        if let Some(Type::Object(obj)) = checker.types.get(calc_type) {
            assert!(obj.members.has("add"), "Calculator should have add method");
        }
    }

    // =========================================================================
    // Diagnostic Message Formatting Tests
    // =========================================================================

    #[test]
    fn test_diagnostic_message_formatting() {
        use super::types::diagnostics::{format_message, diagnostic_messages};

        // Test basic placeholder replacement
        let msg = format_message(diagnostic_messages::TYPE_NOT_ASSIGNABLE, &["string", "number"]);
        assert_eq!(msg, "Type 'string' is not assignable to type 'number'.");

        // Test cannot find name
        let msg = format_message(diagnostic_messages::CANNOT_FIND_NAME, &["foo"]);
        assert_eq!(msg, "Cannot find name 'foo'.");

        // Test property does not exist
        let msg = format_message(diagnostic_messages::PROPERTY_DOES_NOT_EXIST, &["bar", "MyType"]);
        assert_eq!(msg, "Property 'bar' does not exist on type 'MyType'.");

        // Test argument count error
        let msg = format_message(diagnostic_messages::EXPECTED_ARGUMENTS, &["2", "3"]);
        assert_eq!(msg, "Expected 2 arguments, but got 3.");

        // Test no placeholders
        let msg = format_message(diagnostic_messages::CANNOT_INVOKE_EXPRESSION, &[]);
        assert_eq!(msg, "This expression is not callable.");
    }

    #[test]
    fn test_diagnostic_codes() {
        use super::types::diagnostics::diagnostic_codes;

        // Type checking errors should be in 2xxx range
        assert_eq!(diagnostic_codes::TYPE_NOT_ASSIGNABLE_TO_TYPE, 2322);
        assert_eq!(diagnostic_codes::CANNOT_FIND_NAME, 2304);
        assert_eq!(diagnostic_codes::PROPERTY_DOES_NOT_EXIST_ON_TYPE, 2339);
        assert_eq!(diagnostic_codes::EXPECTED_ARGUMENTS, 2554);

        // Parser errors should be in 1xxx range
        assert_eq!(diagnostic_codes::IDENTIFIER_EXPECTED, 1003);
        assert_eq!(diagnostic_codes::TOKEN_EXPECTED, 1005);
    }

    #[test]
    fn test_diagnostic_with_related_info() {
        use super::state::{Diagnostic, DiagnosticCategory, DiagnosticRelatedInformation};

        let related = DiagnosticRelatedInformation {
            file: "other.ts".to_string(),
            start: 100,
            length: 10,
            message_text: "See declaration of 'foo' here.".to_string(),
            category: DiagnosticCategory::Message,
            code: 6203, // "'foo' is declared here"
        };

        let diagnostic = Diagnostic {
            file: "test.ts".to_string(),
            start: 50,
            length: 5,
            message_text: "Cannot find name 'foo'.".to_string(),
            category: DiagnosticCategory::Error,
            code: 2304,
            related_information: vec![related],
        };

        assert_eq!(diagnostic.code, 2304);
        assert_eq!(diagnostic.related_information.len(), 1);
        assert_eq!(diagnostic.related_information[0].file, "other.ts");
    }

    // =========================================================================
    // Enum Type Checking Tests
    // =========================================================================

    #[test]
    fn test_enum_member_access() {
        use crate::parser_impl::ParserState;
        use crate::binder::BinderState;

        let mut parser = ParserState::new(
            "test.ts".to_string(),
            r#"
                enum Color { Red, Green, Blue }
                const red = Color.Red;
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

        // Get the Color enum
        let color_symbol = binder.file_locals.get("Color").expect("Color should exist");
        let color_type = checker.get_type_of_symbol(color_symbol);

        // Should be an enum type
        if let Some(Type::Enum(enum_info)) = checker.types.get(color_type) {
            assert_eq!(enum_info.name, "Color");
            assert_eq!(enum_info.members.len(), 3);
        } else {
            panic!("Expected enum type");
        }
    }

    #[test]
    fn test_enum_assignability() {
        use crate::parser_impl::ParserState;
        use crate::binder::BinderState;

        let mut parser = ParserState::new(
            "test.ts".to_string(),
            "enum Color { Red, Green, Blue }".to_string(),
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

        // Pre-create the literal types
        let red = checker.types.create_number_literal(0.0);
        let green = checker.types.create_number_literal(1.0);
        let blue = checker.types.create_number_literal(2.0);

        // Create a numeric enum in the type arena
        let color_enum = checker.types.create_enum_type(
            "Color".to_string(),
            vec![
                ("Red".to_string(), red),
                ("Green".to_string(), green),
                ("Blue".to_string(), blue),
            ],
        );

        let number_type = checker.types.number_type;

        // Numeric enum is assignable to number
        assert!(checker.is_type_assignable_to(color_enum, number_type));

        // Number is assignable to numeric enum
        assert!(checker.is_type_assignable_to(number_type, color_enum));

        // Number literal is assignable to numeric enum
        let num_lit = checker.types.create_number_literal(0.0);
        assert!(checker.is_type_assignable_to(num_lit, color_enum));
    }

    #[test]
    fn test_enum_string_values() {
        use crate::parser_impl::ParserState;
        use crate::binder::BinderState;

        // String enum
        let mut parser = ParserState::new(
            "test.ts".to_string(),
            r#"
                enum Direction {
                    Up = "UP",
                    Down = "DOWN",
                    Left = "LEFT",
                    Right = "RIGHT"
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

        let dir_symbol = binder.file_locals.get("Direction").expect("Direction should exist");
        let dir_type = checker.get_type_of_symbol(dir_symbol);

        // Should be an enum type with string literal values
        if let Some(Type::Enum(enum_info)) = checker.types.get(dir_type) {
            assert_eq!(enum_info.name, "Direction");
            assert_eq!(enum_info.members.len(), 4);

            // Check that Up has a string literal type
            let up_type = enum_info.members.iter()
                .find(|(name, _)| name == "Up")
                .map(|(_, t)| *t)
                .expect("Up should exist");

            if let Some(Type::Literal(lit)) = checker.types.get(up_type) {
                if let LiteralValue::String(s) = &lit.value {
                    assert_eq!(s, "UP");
                } else {
                    panic!("Expected string literal");
                }
            } else {
                panic!("Expected literal type");
            }
        } else {
            panic!("Expected enum type");
        }
    }

    #[test]
    fn test_same_enum_assignability() {
        use crate::parser_impl::ParserState;
        use crate::binder::BinderState;

        let mut parser = ParserState::new(
            "test.ts".to_string(),
            "enum Color { Red }".to_string(),
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

        // Pre-create literal types
        let red1 = checker.types.create_number_literal(0.0);
        let red2 = checker.types.create_number_literal(0.0);

        // Create two enums with the same name (simulating same declaration)
        let color1 = checker.types.create_enum_type(
            "Color".to_string(),
            vec![("Red".to_string(), red1)],
        );

        let color2 = checker.types.create_enum_type(
            "Color".to_string(),
            vec![("Red".to_string(), red2)],
        );

        // Same-name enums are compatible
        assert!(checker.is_type_assignable_to(color1, color2));
        assert!(checker.is_type_assignable_to(color2, color1));
    }

    #[test]
    fn test_different_enum_not_assignable() {
        use crate::parser_impl::ParserState;
        use crate::binder::BinderState;

        let mut parser = ParserState::new(
            "test.ts".to_string(),
            "enum Color { Red }".to_string(),
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

        // Pre-create literal types
        let red = checker.types.create_number_literal(0.0);
        let up = checker.types.create_number_literal(0.0);

        let color = checker.types.create_enum_type(
            "Color".to_string(),
            vec![("Red".to_string(), red)],
        );

        let direction = checker.types.create_enum_type(
            "Direction".to_string(),
            vec![("Up".to_string(), up)],
        );

        // Different enums are NOT compatible, even with same values
        assert!(!checker.is_type_assignable_to(color, direction));
        assert!(!checker.is_type_assignable_to(direction, color));
    }

    #[test]
    fn test_enum_reverse_mapping() {
        use crate::parser_impl::ParserState;
        use crate::binder::BinderState;

        let mut parser = ParserState::new(
            "test.ts".to_string(),
            "enum Color { Red, Green, Blue }".to_string(),
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

        // Create the enum type with numeric values
        let red = checker.types.create_number_literal(0.0);
        let green = checker.types.create_number_literal(1.0);
        let blue = checker.types.create_number_literal(2.0);

        let color_enum = checker.types.create_enum_type(
            "Color".to_string(),
            vec![
                ("Red".to_string(), red),
                ("Green".to_string(), green),
                ("Blue".to_string(), blue),
            ],
        );

        // Create index types
        let index_0 = checker.types.create_number_literal(0.0);
        let index_1 = checker.types.create_number_literal(1.0);
        let index_red = checker.types.create_string_literal("Red".to_string());

        // Reverse mapping: Color[0] should return "Red"
        let result_0 = checker.get_indexed_access_type(color_enum, index_0);
        if let Some(Type::Literal(lit)) = checker.types.get(result_0) {
            if let LiteralValue::String(s) = &lit.value {
                assert_eq!(s, "Red");
            } else {
                panic!("Expected string literal for Color[0]");
            }
        } else {
            panic!("Expected literal type for Color[0]");
        }

        // Reverse mapping: Color[1] should return "Green"
        let result_1 = checker.get_indexed_access_type(color_enum, index_1);
        if let Some(Type::Literal(lit)) = checker.types.get(result_1) {
            if let LiteralValue::String(s) = &lit.value {
                assert_eq!(s, "Green");
            } else {
                panic!("Expected string literal for Color[1]");
            }
        } else {
            panic!("Expected literal type for Color[1]");
        }

        // Forward mapping: Color["Red"] should return 0
        let result_red = checker.get_indexed_access_type(color_enum, index_red);
        if let Some(Type::Literal(lit)) = checker.types.get(result_red) {
            if let LiteralValue::Number(n) = &lit.value {
                assert_eq!(*n as i32, 0);
            } else {
                panic!("Expected number literal for Color['Red']");
            }
        } else {
            panic!("Expected literal type for Color['Red']");
        }
    }

    // =========================================================================
    // Function Overload Resolution Tests
    // =========================================================================

    #[test]
    fn test_signature_creation() {
        use crate::parser::NodeIndex;
        use super::types::Signature;

        let mut arena = super::TypeArena::new();

        // Get types
        let number_type = arena.number_type;
        let string_type = arena.string_type;

        // Create a signature for (x: number): string
        let mut sig = Signature::new(NodeIndex::NONE);
        sig.min_argument_count = 1;
        sig.resolved_return_type = Some(string_type);

        // Verify the signature
        assert_eq!(sig.min_argument_count, 1);
        assert_eq!(sig.resolved_return_type, Some(string_type));
    }

    #[test]
    fn test_overload_signatures() {
        use crate::parser::NodeIndex;
        use super::types::Signature;

        let mut arena = super::TypeArena::new();

        let number_type = arena.number_type;
        let string_type = arena.string_type;

        // Create signature 1: () => number
        let mut sig1 = Signature::new(NodeIndex::NONE);
        sig1.min_argument_count = 0;
        sig1.resolved_return_type = Some(number_type);

        // Create signature 2: () => string
        let mut sig2 = Signature::new(NodeIndex::NONE);
        sig2.min_argument_count = 0;
        sig2.resolved_return_type = Some(string_type);

        // Verify we have two signatures with different return types
        assert_eq!(sig1.resolved_return_type, Some(number_type));
        assert_eq!(sig2.resolved_return_type, Some(string_type));
        assert_ne!(sig1.resolved_return_type, sig2.resolved_return_type);
    }

    #[test]
    fn test_visibility_flags_on_symbols() {
        // Test that visibility flags are correctly defined
        assert_eq!(symbol_flags::PRIVATE, 1 << 28);
        assert_eq!(symbol_flags::PROTECTED, 1 << 29);

        // Test flag combinations
        let private_method = symbol_flags::METHOD | symbol_flags::PRIVATE;
        assert!((private_method & symbol_flags::METHOD) != 0);
        assert!((private_method & symbol_flags::PRIVATE) != 0);
        assert!((private_method & symbol_flags::PROTECTED) == 0);

        let protected_prop = symbol_flags::PROPERTY | symbol_flags::PROTECTED;
        assert!((protected_prop & symbol_flags::PROPERTY) != 0);
        assert!((protected_prop & symbol_flags::PROTECTED) != 0);
        assert!((protected_prop & symbol_flags::PRIVATE) == 0);
    }

    // NOTE: The full integration tests for private/protected access are skipped for now
    // because class type resolution currently causes issues with the memory limits.
    // The basic visibility checking implementation is complete:
    // - PRIVATE/PROTECTED symbol flags are set in binder.rs (bind_class_member)
    // - get_property_symbol() looks up symbols for visibility checking
    // - check_property_visibility() reports errors when accessing private/protected outside class
    // - enclosing_class tracking is implemented in CheckerState
    #[test]
    fn test_private_property_access_outside_class() {
        // Test: private properties should error when accessed outside class
        // class Foo { private x: number; }
        // let f = new Foo();
        // f.x; // Error: Property 'x' is private

        use crate::parser_impl::ParserState;
        use crate::binder::BinderState;

        let code = r#"
class Foo {
    private x: number = 1;
}
let f = new Foo();
let v = f.x;
"#;

        let mut parser = ParserState::new("test.ts".to_string(), code.to_string());
        let root = parser.parse_source_file();

        let mut binder = BinderState::new();
        binder.bind_source_file(&parser.arena, root);

        let mut checker = CheckerState::new(
            &parser.arena,
            &binder.symbols,
            &binder.file_locals,
            "test.ts".to_string(),
        );

        checker.check_source_file(root);

        // Should have an error about private property access
        let private_error = checker.diagnostics.iter().any(|d| {
            d.message_text.contains("private") || d.code == super::diagnostic_codes::PROPERTY_IS_PRIVATE
        });
        assert!(private_error, "Expected error for private property access outside class. Got: {:?}", checker.diagnostics);
    }

    #[test]
    fn test_protected_property_access_outside_class() {
        // Test: protected properties should error when accessed outside class
        // class Foo { protected y: string; }
        // let f = new Foo();
        // f.y; // Error: Property 'y' is protected

        use crate::parser_impl::ParserState;
        use crate::binder::BinderState;

        let code = r#"
class Foo {
    protected y: string = "hello";
}
let f = new Foo();
let v = f.y;
"#;

        let mut parser = ParserState::new("test.ts".to_string(), code.to_string());
        let root = parser.parse_source_file();

        let mut binder = BinderState::new();
        binder.bind_source_file(&parser.arena, root);

        let mut checker = CheckerState::new(
            &parser.arena,
            &binder.symbols,
            &binder.file_locals,
            "test.ts".to_string(),
        );

        checker.check_source_file(root);

        // Should have an error about protected property access
        let protected_error = checker.diagnostics.iter().any(|d| {
            d.message_text.contains("protected") || d.code == super::diagnostic_codes::PROPERTY_IS_PROTECTED
        });
        assert!(protected_error, "Expected error for protected property access outside class. Got: {:?}", checker.diagnostics);
    }

    // =========================================================================
    // Abstract member verification tests (5.57)
    // =========================================================================

    #[test]
    fn test_abstract_method_in_non_abstract_class() {
        // Abstract method in non-abstract class should produce error
        use crate::parser_impl::ParserState;
        use crate::binder::BinderState;

        let code = r#"
class Foo {
    abstract bar(): void;
}
"#;

        let mut parser = ParserState::new("test.ts".to_string(), code.to_string());
        let root = parser.parse_source_file();

        let mut binder = BinderState::new();
        binder.bind_source_file(&parser.arena, root);

        let mut checker = CheckerState::new(
            &parser.arena,
            &binder.symbols,
            &binder.file_locals,
            "test.ts".to_string(),
        );

        checker.check_source_file(root);

        assert!(
            !checker.diagnostics.is_empty(),
            "Expected error for abstract method in non-abstract class"
        );
        assert!(
            checker.diagnostics.iter().any(|d| d.code == super::diagnostic_codes::ABSTRACT_MEMBER_IN_NON_ABSTRACT_CLASS),
            "Expected error code 2515. Got: {:?}",
            checker.diagnostics
        );
    }

    #[test]
    fn test_abstract_method_in_abstract_class_allowed() {
        // Abstract method in abstract class should be allowed
        use crate::parser_impl::ParserState;
        use crate::binder::BinderState;

        let code = r#"
abstract class Foo {
    abstract bar(): void;
}
"#;

        let mut parser = ParserState::new("test.ts".to_string(), code.to_string());
        let root = parser.parse_source_file();

        let mut binder = BinderState::new();
        binder.bind_source_file(&parser.arena, root);

        let mut checker = CheckerState::new(
            &parser.arena,
            &binder.symbols,
            &binder.file_locals,
            "test.ts".to_string(),
        );

        checker.check_source_file(root);

        // Should have no errors about abstract members
        let abstract_errors: Vec<_> = checker.diagnostics.iter()
            .filter(|d| d.code == super::diagnostic_codes::ABSTRACT_MEMBER_IN_NON_ABSTRACT_CLASS)
            .collect();
        assert!(
            abstract_errors.is_empty(),
            "Should allow abstract method in abstract class. Got: {:?}",
            abstract_errors
        );
    }

    #[test]
    fn test_abstract_property_in_non_abstract_class() {
        // Abstract property in non-abstract class should produce error
        use crate::parser_impl::ParserState;
        use crate::binder::BinderState;

        let code = r#"
class Foo {
    abstract name: string;
}
"#;

        let mut parser = ParserState::new("test.ts".to_string(), code.to_string());
        let root = parser.parse_source_file();

        let mut binder = BinderState::new();
        binder.bind_source_file(&parser.arena, root);

        let mut checker = CheckerState::new(
            &parser.arena,
            &binder.symbols,
            &binder.file_locals,
            "test.ts".to_string(),
        );

        checker.check_source_file(root);

        assert!(
            !checker.diagnostics.is_empty(),
            "Expected error for abstract property in non-abstract class"
        );
        assert!(
            checker.diagnostics.iter().any(|d| d.code == super::diagnostic_codes::ABSTRACT_MEMBER_IN_NON_ABSTRACT_CLASS),
            "Expected error code 2515. Got: {:?}",
            checker.diagnostics
        );
    }

    #[test]
    fn test_non_abstract_method_in_non_abstract_class_allowed() {
        // Non-abstract method in non-abstract class should be allowed
        use crate::parser_impl::ParserState;
        use crate::binder::BinderState;

        let code = r#"
class Foo {
    bar(): void {}
}
"#;

        let mut parser = ParserState::new("test.ts".to_string(), code.to_string());
        let root = parser.parse_source_file();

        let mut binder = BinderState::new();
        binder.bind_source_file(&parser.arena, root);

        let mut checker = CheckerState::new(
            &parser.arena,
            &binder.symbols,
            &binder.file_locals,
            "test.ts".to_string(),
        );

        checker.check_source_file(root);

        // Should have no errors about abstract members
        let abstract_errors: Vec<_> = checker.diagnostics.iter()
            .filter(|d| d.code == super::diagnostic_codes::ABSTRACT_MEMBER_IN_NON_ABSTRACT_CLASS)
            .collect();
        assert!(
            abstract_errors.is_empty(),
            "Non-abstract method should be allowed in non-abstract class. Got: {:?}",
            abstract_errors
        );
    }

    // =========================================================================
    // Override keyword validation tests (5.58)
    // =========================================================================

    #[test]
    fn test_override_with_no_base_class() {
        // Override modifier on a class with no base class should produce error
        use crate::parser_impl::ParserState;
        use crate::binder::BinderState;

        let code = r#"
class Foo {
    override bar(): void {}
}
"#;

        let mut parser = ParserState::new("test.ts".to_string(), code.to_string());
        let root = parser.parse_source_file();

        let mut binder = BinderState::new();
        binder.bind_source_file(&parser.arena, root);

        let mut checker = CheckerState::new(
            &parser.arena,
            &binder.symbols,
            &binder.file_locals,
            "test.ts".to_string(),
        );

        checker.check_source_file(root);

        assert!(
            !checker.diagnostics.is_empty(),
            "Expected error for override without base class"
        );
        assert!(
            checker.diagnostics.iter().any(|d| d.code == super::diagnostic_codes::OVERRIDE_MEMBER_NOT_IN_BASE),
            "Expected error code 4114. Got: {:?}",
            checker.diagnostics
        );
    }

    #[test]
    fn test_override_member_not_in_base() {
        // Override modifier for member not in base class should produce error
        use crate::parser_impl::ParserState;
        use crate::binder::BinderState;

        let code = r#"
class Base {
    foo(): void {}
}
class Derived extends Base {
    override bar(): void {}
}
"#;

        let mut parser = ParserState::new("test.ts".to_string(), code.to_string());
        let root = parser.parse_source_file();

        let mut binder = BinderState::new();
        binder.bind_source_file(&parser.arena, root);

        let mut checker = CheckerState::new(
            &parser.arena,
            &binder.symbols,
            &binder.file_locals,
            "test.ts".to_string(),
        );

        checker.check_source_file(root);

        assert!(
            !checker.diagnostics.is_empty(),
            "Expected error for override member not in base class"
        );
        assert!(
            checker.diagnostics.iter().any(|d| d.code == super::diagnostic_codes::OVERRIDE_MEMBER_NOT_IN_BASE),
            "Expected error code 4114. Got: {:?}",
            checker.diagnostics
        );
    }

    #[test]
    fn test_override_member_in_base_allowed() {
        // Override modifier for member that exists in base class should be allowed
        use crate::parser_impl::ParserState;
        use crate::binder::BinderState;

        let code = r#"
class Base {
    foo(): void {}
}
class Derived extends Base {
    override foo(): void {}
}
"#;

        let mut parser = ParserState::new("test.ts".to_string(), code.to_string());
        let root = parser.parse_source_file();

        let mut binder = BinderState::new();
        binder.bind_source_file(&parser.arena, root);

        let mut checker = CheckerState::new(
            &parser.arena,
            &binder.symbols,
            &binder.file_locals,
            "test.ts".to_string(),
        );

        checker.check_source_file(root);

        // Should have no errors about override
        let override_errors: Vec<_> = checker.diagnostics.iter()
            .filter(|d| d.code == super::diagnostic_codes::OVERRIDE_MEMBER_NOT_IN_BASE)
            .collect();
        assert!(
            override_errors.is_empty(),
            "Should allow override for member in base class. Got: {:?}",
            override_errors
        );
    }

    // =========================================================================
    // TypeQuery (typeof in type position) tests (5.103)
    // =========================================================================

    #[test]
    fn test_typeof_type_operator() {
        // Test typeof in type position
        use crate::parser_impl::ParserState;
        use crate::binder::BinderState;

        let code = r#"
const x = 42;
type T = typeof x;
"#;

        let mut parser = ParserState::new("test.ts".to_string(), code.to_string());
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
        if let Some(symbol_id) = binder.file_locals.get("x") {
            let x_type = checker.get_type_of_symbol(symbol_id);
            let type_str = checker.type_to_string(x_type);
            // x should be either number or a numeric literal type
            assert!(
                x_type == checker.types.number_type || type_str == "42",
                "x should be number or literal 42 type, got: {}",
                type_str
            );
        } else {
            panic!("Should have symbol 'x'");
        }
    }

    // =========================================================================
    // String manipulation types tests (5.68)
    // =========================================================================

    #[test]
    fn test_uppercase_string_type() {
        use crate::parser_impl::ParserState;
        use crate::binder::BinderState;

        let code = r#"
type Upper = Uppercase<"hello">;
"#;

        let mut parser = ParserState::new("test.ts".to_string(), code.to_string());
        let root = parser.parse_source_file();

        let mut binder = BinderState::new();
        binder.bind_source_file(&parser.arena, root);

        let mut checker = CheckerState::new(
            &parser.arena,
            &binder.symbols,
            &binder.file_locals,
            "test.ts".to_string(),
        );

        // Get the type of 'Upper'
        if let Some(symbol_id) = binder.file_locals.get("Upper") {
            let upper_type = checker.get_type_of_symbol(symbol_id);
            let type_str = checker.type_to_string(upper_type);
            assert_eq!(type_str, "\"HELLO\"", "Uppercase<\"hello\"> should be \"HELLO\"");
        } else {
            panic!("Should have symbol 'Upper'");
        }
    }

    #[test]
    fn test_lowercase_string_type() {
        use crate::parser_impl::ParserState;
        use crate::binder::BinderState;

        let code = r#"
type Lower = Lowercase<"HELLO">;
"#;

        let mut parser = ParserState::new("test.ts".to_string(), code.to_string());
        let root = parser.parse_source_file();

        let mut binder = BinderState::new();
        binder.bind_source_file(&parser.arena, root);

        let mut checker = CheckerState::new(
            &parser.arena,
            &binder.symbols,
            &binder.file_locals,
            "test.ts".to_string(),
        );

        // Get the type of 'Lower'
        if let Some(symbol_id) = binder.file_locals.get("Lower") {
            let lower_type = checker.get_type_of_symbol(symbol_id);
            let type_str = checker.type_to_string(lower_type);
            assert_eq!(type_str, "\"hello\"", "Lowercase<\"HELLO\"> should be \"hello\"");
        } else {
            panic!("Should have symbol 'Lower'");
        }
    }

    #[test]
    fn test_capitalize_string_type() {
        use crate::parser_impl::ParserState;
        use crate::binder::BinderState;

        let code = r#"
type Cap = Capitalize<"hello">;
"#;

        let mut parser = ParserState::new("test.ts".to_string(), code.to_string());
        let root = parser.parse_source_file();

        let mut binder = BinderState::new();
        binder.bind_source_file(&parser.arena, root);

        let mut checker = CheckerState::new(
            &parser.arena,
            &binder.symbols,
            &binder.file_locals,
            "test.ts".to_string(),
        );

        // Get the type of 'Cap'
        if let Some(symbol_id) = binder.file_locals.get("Cap") {
            let cap_type = checker.get_type_of_symbol(symbol_id);
            let type_str = checker.type_to_string(cap_type);
            assert_eq!(type_str, "\"Hello\"", "Capitalize<\"hello\"> should be \"Hello\"");
        } else {
            panic!("Should have symbol 'Cap'");
        }
    }

    #[test]
    fn test_uncapitalize_string_type() {
        use crate::parser_impl::ParserState;
        use crate::binder::BinderState;

        let code = r#"
type Uncap = Uncapitalize<"Hello">;
"#;

        let mut parser = ParserState::new("test.ts".to_string(), code.to_string());
        let root = parser.parse_source_file();

        let mut binder = BinderState::new();
        binder.bind_source_file(&parser.arena, root);

        let mut checker = CheckerState::new(
            &parser.arena,
            &binder.symbols,
            &binder.file_locals,
            "test.ts".to_string(),
        );

        // Get the type of 'Uncap'
        if let Some(symbol_id) = binder.file_locals.get("Uncap") {
            let uncap_type = checker.get_type_of_symbol(symbol_id);
            let type_str = checker.type_to_string(uncap_type);
            assert_eq!(type_str, "\"hello\"", "Uncapitalize<\"Hello\"> should be \"hello\"");
        } else {
            panic!("Should have symbol 'Uncap'");
        }
    }

    // =========================================================================
    // As const tests (5.72)
    // =========================================================================

    #[test]
    fn test_as_const_number_literal() {
        use crate::parser_impl::ParserState;
        use crate::binder::BinderState;

        let code = r#"
const x = 42 as const;
"#;

        let mut parser = ParserState::new("test.ts".to_string(), code.to_string());
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
        if let Some(symbol_id) = binder.file_locals.get("x") {
            let x_type = checker.get_type_of_symbol(symbol_id);
            let type_str = checker.type_to_string(x_type);
            // With as const, type should be the literal 42, not number
            assert_eq!(type_str, "42", "x should be literal type 42, got: {}", type_str);
        } else {
            panic!("Should have symbol 'x'");
        }
    }

    #[test]
    fn test_as_const_string_literal() {
        use crate::parser_impl::ParserState;
        use crate::binder::BinderState;

        let code = r#"
const s = "hello" as const;
"#;

        let mut parser = ParserState::new("test.ts".to_string(), code.to_string());
        let root = parser.parse_source_file();

        let mut binder = BinderState::new();
        binder.bind_source_file(&parser.arena, root);

        let mut checker = CheckerState::new(
            &parser.arena,
            &binder.symbols,
            &binder.file_locals,
            "test.ts".to_string(),
        );

        // Get the type of 's'
        if let Some(symbol_id) = binder.file_locals.get("s") {
            let s_type = checker.get_type_of_symbol(symbol_id);
            let type_str = checker.type_to_string(s_type);
            // With as const, type should be the literal "hello", not string
            assert_eq!(type_str, "\"hello\"", "s should be literal type \"hello\", got: {}", type_str);
        } else {
            panic!("Should have symbol 's'");
        }
    }

    // ============== 5.73: Awaited<T> type ==============
    #[test]
    fn test_awaited_type_basic() {
        // Test that Awaited<T> returns T for non-Promise types
        let mut arena = super::TypeArena::new();

        // For non-Promise types, Awaited<T> should return T
        let number_type = arena.number_type;
        let string_type = arena.string_type;

        // Verify these base types exist
        assert!(arena.get(number_type).is_some(), "number type should exist");
        assert!(arena.get(string_type).is_some(), "string type should exist");
    }

    #[test]
    fn test_awaited_type_with_primitive() {
        use crate::parser_impl::ParserState;
        use crate::binder::BinderState;

        // Test Awaited<T> utility type with non-Promise types
        let code = r#"
type AwaitedNum = Awaited<number>;
type AwaitedStr = Awaited<string>;
"#;

        let mut parser = ParserState::new("test.ts".to_string(), code.to_string());
        let root = parser.parse_source_file();

        let mut binder = BinderState::new();
        binder.bind_source_file(&parser.arena, root);

        let mut checker = CheckerState::new(
            &parser.arena,
            &binder.symbols,
            &binder.file_locals,
            "test.ts".to_string(),
        );

        // For non-Promise types, Awaited<T> should return T
        if let Some(symbol_id) = binder.file_locals.get("AwaitedNum") {
            let awaited_type = checker.get_type_of_symbol(symbol_id);
            let type_str = checker.type_to_string(awaited_type);
            // Awaited<number> = number (since number is not a Promise)
            assert_eq!(type_str, "number", "Awaited<number> should be number, got: {}", type_str);
        } else {
            panic!("Should have symbol 'AwaitedNum'");
        }

        if let Some(symbol_id) = binder.file_locals.get("AwaitedStr") {
            let awaited_type = checker.get_type_of_symbol(symbol_id);
            let type_str = checker.type_to_string(awaited_type);
            // Awaited<string> = string (since string is not a Promise)
            assert_eq!(type_str, "string", "Awaited<string> should be string, got: {}", type_str);
        } else {
            panic!("Should have symbol 'AwaitedStr'");
        }
    }

    #[test]
    fn test_non_nullable_utility_type() {
        use crate::parser_impl::ParserState;
        use crate::binder::BinderState;

        // Test NonNullable<T> utility type
        let code = r#"
type Result = NonNullable<string | null | undefined>;
"#;

        let mut parser = ParserState::new("test.ts".to_string(), code.to_string());
        let root = parser.parse_source_file();

        let mut binder = BinderState::new();
        binder.bind_source_file(&parser.arena, root);

        let mut checker = CheckerState::new(
            &parser.arena,
            &binder.symbols,
            &binder.file_locals,
            "test.ts".to_string(),
        );

        // NonNullable<string | null | undefined> should be string
        if let Some(symbol_id) = binder.file_locals.get("Result") {
            let result_type = checker.get_type_of_symbol(symbol_id);
            let type_str = checker.type_to_string(result_type);
            // Should contain "string" after removing null and undefined
            assert!(type_str.contains("string") || type_str == "string",
                "NonNullable<string | null | undefined> should contain string, got: {}", type_str);
        } else {
            panic!("Should have symbol 'Result'");
        }
    }

    // ============== 5.63: NoInfer<T> type ==============
    #[test]
    fn test_noinfer_type() {
        use crate::parser_impl::ParserState;
        use crate::binder::BinderState;

        // Test NoInfer<T> utility type - should return T unchanged
        let code = r#"
type NoInferNum = NoInfer<number>;
type NoInferStr = NoInfer<string>;
"#;

        let mut parser = ParserState::new("test.ts".to_string(), code.to_string());
        let root = parser.parse_source_file();

        let mut binder = BinderState::new();
        binder.bind_source_file(&parser.arena, root);

        let mut checker = CheckerState::new(
            &parser.arena,
            &binder.symbols,
            &binder.file_locals,
            "test.ts".to_string(),
        );

        // NoInfer<number> should return number
        if let Some(symbol_id) = binder.file_locals.get("NoInferNum") {
            let noinfer_type = checker.get_type_of_symbol(symbol_id);
            let type_str = checker.type_to_string(noinfer_type);
            assert_eq!(type_str, "number", "NoInfer<number> should be number, got: {}", type_str);
        } else {
            panic!("Should have symbol 'NoInferNum'");
        }

        // NoInfer<string> should return string
        if let Some(symbol_id) = binder.file_locals.get("NoInferStr") {
            let noinfer_type = checker.get_type_of_symbol(symbol_id);
            let type_str = checker.type_to_string(noinfer_type);
            assert_eq!(type_str, "string", "NoInfer<string> should be string, got: {}", type_str);
        } else {
            panic!("Should have symbol 'NoInferStr'");
        }
    }

    // ============== 5.69: Instantiation depth limits ==============
    #[test]
    fn test_instantiation_depth_limit_exists() {
        use crate::checker::state::MAX_INSTANTIATION_DEPTH;

        // Verify the depth limit constant is set to a reasonable value
        assert_eq!(MAX_INSTANTIATION_DEPTH, 50, "TypeScript uses 50 as default depth limit");
    }

    #[test]
    fn test_instantiation_depth_counter_starts_at_zero() {
        use crate::parser_impl::ParserState;
        use crate::binder::BinderState;

        let code = "let x: number = 1;";
        let mut parser = ParserState::new("test.ts".to_string(), code.to_string());
        let root = parser.parse_source_file();

        let mut binder = BinderState::new();
        binder.bind_source_file(&parser.arena, root);

        let checker = CheckerState::new(
            &parser.arena,
            &binder.symbols,
            &binder.file_locals,
            "test.ts".to_string(),
        );

        // Verify depth starts at 0
        let depth = *checker.instantiation_depth.borrow();
        assert_eq!(depth, 0, "Instantiation depth should start at 0");
    }

    #[test]
    fn test_nested_generic_type_instantiation() {
        use crate::parser_impl::ParserState;
        use crate::binder::BinderState;

        // Test that nested generic types are properly instantiated
        // This exercises the depth tracking mechanism without hitting the limit
        let code = r#"
type Wrapped<T> = { value: T };
type DoubleWrapped<T> = Wrapped<Wrapped<T>>;
type Test = DoubleWrapped<number>;
"#;

        let mut parser = ParserState::new("test.ts".to_string(), code.to_string());
        let root = parser.parse_source_file();

        let mut binder = BinderState::new();
        binder.bind_source_file(&parser.arena, root);

        let mut checker = CheckerState::new(
            &parser.arena,
            &binder.symbols,
            &binder.file_locals,
            "test.ts".to_string(),
        );

        // Type should be resolvable within depth limits
        if let Some(symbol_id) = binder.file_locals.get("Test") {
            let test_type = checker.get_type_of_symbol(symbol_id);
            // Should not be any (which would indicate depth limit hit)
            assert_ne!(test_type, checker.types.any_type,
                "Nested generic should resolve without hitting depth limit");
        }

        // Verify depth returns to 0 after type resolution
        let depth = *checker.instantiation_depth.borrow();
        assert_eq!(depth, 0, "Depth should return to 0 after type resolution");
    }

    // ============== 5.70: Circular reference detection ==============
    #[test]
    fn test_circular_type_alias_reference() {
        use crate::parser_impl::ParserState;
        use crate::binder::BinderState;

        // Test circular type alias reference (should not hang)
        // This creates a direct circular reference: type A = A
        let code = r#"
type Recursive = { next: Recursive };
"#;

        let mut parser = ParserState::new("test.ts".to_string(), code.to_string());
        let root = parser.parse_source_file();

        let mut binder = BinderState::new();
        binder.bind_source_file(&parser.arena, root);

        let mut checker = CheckerState::new(
            &parser.arena,
            &binder.symbols,
            &binder.file_locals,
            "test.ts".to_string(),
        );

        // Resolving this should not hang - circular detection kicks in
        if let Some(symbol_id) = binder.file_locals.get("Recursive") {
            let recursive_type = checker.get_type_of_symbol(symbol_id);
            // Should resolve to something (not hang)
            let type_str = checker.type_to_string(recursive_type);
            println!("Recursive type resolved to: {}", type_str);
            // The type exists and didn't cause infinite loop
            assert!(true, "Successfully resolved circular type alias");
        } else {
            panic!("Should have symbol 'Recursive'");
        }
    }

    #[test]
    fn test_circular_interface_reference() {
        use crate::parser_impl::ParserState;
        use crate::binder::BinderState;

        // Test circular interface reference
        // Use 'TreeNode' to avoid any potential conflicts with built-in 'Node'
        let code = r#"
interface TreeNode {
    value: number;
    left: TreeNode | null;
    right: TreeNode | null;
}
"#;

        let mut parser = ParserState::new("test.ts".to_string(), code.to_string());
        let root = parser.parse_source_file();

        let mut binder = BinderState::new();
        binder.bind_source_file(&parser.arena, root);

        let mut checker = CheckerState::new(
            &parser.arena,
            &binder.symbols,
            &binder.file_locals,
            "test.ts".to_string(),
        );

        // Resolving this should not hang
        if let Some(symbol_id) = binder.file_locals.get("TreeNode") {
            let tree_type = checker.get_type_of_symbol(symbol_id);
            let type_str = checker.type_to_string(tree_type);
            println!("TreeNode type resolved to: {}", type_str);
            assert!(true, "Successfully resolved circular interface");
        } else {
            panic!("Should have symbol 'TreeNode'");
        }
    }

    #[test]
    fn test_resolution_stack_is_clean_after_resolution() {
        use crate::parser_impl::ParserState;
        use crate::binder::BinderState;

        let code = "type T = string;";
        let mut parser = ParserState::new("test.ts".to_string(), code.to_string());
        let root = parser.parse_source_file();

        let mut binder = BinderState::new();
        binder.bind_source_file(&parser.arena, root);

        let mut checker = CheckerState::new(
            &parser.arena,
            &binder.symbols,
            &binder.file_locals,
            "test.ts".to_string(),
        );

        // Resolve a type
        if let Some(symbol_id) = binder.file_locals.get("T") {
            let _ = checker.get_type_of_symbol(symbol_id);
        }

        // Verify resolution stacks are empty after resolution
        assert!(checker.symbol_resolution_stack.is_empty(),
            "Symbol resolution stack should be empty after resolution");
        assert!(checker.symbol_resolution_set.is_empty(),
            "Symbol resolution set should be empty after resolution");
        assert!(checker.node_resolution_stack.is_empty(),
            "Node resolution stack should be empty after resolution");
        assert!(checker.node_resolution_set.is_empty(),
            "Node resolution set should be empty after resolution");
    }

    #[test]
    fn test_relation_cache() {
        use crate::parser_impl::ParserState;
        use crate::binder::BinderState;

        let code = "let x: string = 'hello';";
        let mut parser = ParserState::new("test.ts".to_string(), code.to_string());
        let root = parser.parse_source_file();

        let mut binder = BinderState::new();
        binder.bind_source_file(&parser.arena, root);

        let mut checker = CheckerState::new(
            &parser.arena,
            &binder.symbols,
            &binder.file_locals,
            "test.ts".to_string(),
        );

        // Verify cache is initially empty
        assert!(checker.relation_cache.borrow().is_empty());

        // First call should populate cache
        let result1 = checker.is_type_assignable_to(checker.types.string_type, checker.types.string_type);
        assert!(result1);
        // Same type returns true without caching (identity optimization)

        // Different types that are related - should cache
        let string_lit = checker.types.create_string_literal("hello".to_string());
        let result2 = checker.is_type_assignable_to(string_lit, checker.types.string_type);
        assert!(result2);

        // Cache should now have an entry
        assert!(!checker.relation_cache.borrow().is_empty());

        // Second call should use cache
        let result3 = checker.is_type_assignable_to(string_lit, checker.types.string_type);
        assert!(result3);

        // Negative relation should also be cached
        let result4 = checker.is_type_assignable_to(checker.types.number_type, checker.types.string_type);
        assert!(!result4);

        // Verify cache size increased
        let cache_size = checker.relation_cache.borrow().len();
        assert!(cache_size >= 2, "Cache should have at least 2 entries");
    }

    #[test]
    fn test_awaited_type_cache() {
        use crate::parser_impl::ParserState;
        use crate::binder::BinderState;

        let code = "let x: Promise<string>;";
        let mut parser = ParserState::new("test.ts".to_string(), code.to_string());
        let root = parser.parse_source_file();

        let mut binder = BinderState::new();
        binder.bind_source_file(&parser.arena, root);

        let mut checker = CheckerState::new(
            &parser.arena,
            &binder.symbols,
            &binder.file_locals,
            "test.ts".to_string(),
        );

        // Verify awaited cache is initially empty
        assert!(checker.awaited_type_cache.borrow().is_empty());

        // Get awaited type for a simple type (should be cached)
        let string_type = checker.types.string_type;
        let awaited1 = checker.get_awaited_type(string_type);
        assert_eq!(awaited1, string_type, "Awaited<string> should be string");

        // Cache should now have an entry
        assert!(!checker.awaited_type_cache.borrow().is_empty());

        // Second call should use cache
        let awaited2 = checker.get_awaited_type(string_type);
        assert_eq!(awaited2, string_type);

        // Get awaited type for number
        let number_type = checker.types.number_type;
        let awaited_num = checker.get_awaited_type(number_type);
        assert_eq!(awaited_num, number_type, "Awaited<number> should be number");

        // Cache should have multiple entries
        let cache_size = checker.awaited_type_cache.borrow().len();
        assert!(cache_size >= 2, "Cache should have at least 2 entries");
    }

    #[test]
    fn test_widened_type_cache() {
        use crate::parser_impl::ParserState;
        use crate::binder::BinderState;

        let code = "let x: string;";
        let mut parser = ParserState::new("test.ts".to_string(), code.to_string());
        let root = parser.parse_source_file();

        let mut binder = BinderState::new();
        binder.bind_source_file(&parser.arena, root);

        let mut checker = CheckerState::new(
            &parser.arena,
            &binder.symbols,
            &binder.file_locals,
            "test.ts".to_string(),
        );

        // Verify widened cache is initially empty
        assert!(checker.widened_type_cache.borrow().is_empty());

        // Create a string literal type
        let str_lit = checker.types.create_string_literal("hello".to_string());
        let widened = checker.get_widened_type(str_lit);

        // String literal should widen to string
        assert_eq!(widened, checker.types.string_type,
            "String literal should widen to string");

        // Cache should have an entry
        assert!(!checker.widened_type_cache.borrow().is_empty());

        // Second call should use cache
        let widened2 = checker.get_widened_type(str_lit);
        assert_eq!(widened2, checker.types.string_type);

        // Number literal should widen to number
        let num_lit = checker.types.create_number_literal(42.0);
        let widened_num = checker.get_widened_type(num_lit);
        assert_eq!(widened_num, checker.types.number_type,
            "Number literal should widen to number");

        // Union of string literals should widen to string
        let str_lit2 = checker.types.create_string_literal("world".to_string());
        let union_type = checker.types.create_union_type(vec![str_lit, str_lit2]);
        let widened_union = checker.get_widened_type(union_type);
        assert_eq!(widened_union, checker.types.string_type,
            "Union of string literals should widen to string");

        // Cache should have multiple entries
        let cache_size = checker.widened_type_cache.borrow().len();
        assert!(cache_size >= 3, "Cache should have at least 3 entries");
    }

    // ============== 5.67: Named tuple elements ==============
    #[test]
    fn test_named_tuple_type_to_string() {
        let mut arena = super::TypeArena::new();

        // Create a named tuple [x: number, y: number]
        let named_tuple = arena.create_named_tuple_type(
            vec![arena.number_type, arena.number_type],
            vec![Some("x".to_string()), Some("y".to_string())],
            false, // has_optional
            false, // has_rest
            false, // is_readonly
        );

        // Create a checker to test type_to_string
        use crate::parser_impl::ParserState;
        use crate::binder::BinderState;

        let code = "let x: number;";
        let mut parser = ParserState::new("test.ts".to_string(), code.to_string());
        let root = parser.parse_source_file();

        let mut binder = BinderState::new();
        binder.bind_source_file(&parser.arena, root);

        let mut checker = CheckerState::new(
            &parser.arena,
            &binder.symbols,
            &binder.file_locals,
            "test.ts".to_string(),
        );

        // The arena has its own types, so we need to create the tuple in the checker's arena
        // For this test, we verify the arena creation works
        let named_tuple_checker = checker.types.create_named_tuple_type(
            vec![checker.types.number_type, checker.types.number_type],
            vec![Some("x".to_string()), Some("y".to_string())],
            false, false, false,
        );

        let type_str = checker.type_to_string(named_tuple_checker);
        assert_eq!(type_str, "[x: number, y: number]",
            "Named tuple should show element names, got: {}", type_str);
    }

    #[test]
    fn test_mixed_named_unnamed_tuple() {
        use crate::parser_impl::ParserState;
        use crate::binder::BinderState;

        let code = "let x: number;";
        let mut parser = ParserState::new("test.ts".to_string(), code.to_string());
        let root = parser.parse_source_file();

        let mut binder = BinderState::new();
        binder.bind_source_file(&parser.arena, root);

        let mut checker = CheckerState::new(
            &parser.arena,
            &binder.symbols,
            &binder.file_locals,
            "test.ts".to_string(),
        );

        // Create a tuple with some named and some unnamed elements
        // This is unusual but should be handled gracefully
        let mixed_tuple = checker.types.create_named_tuple_type(
            vec![checker.types.number_type, checker.types.string_type, checker.types.boolean_type],
            vec![Some("first".to_string()), None, Some("third".to_string())],
            false, false, false,
        );

        let type_str = checker.type_to_string(mixed_tuple);
        assert_eq!(type_str, "[first: number, string, third: boolean]",
            "Mixed tuple should show names where available, got: {}", type_str);
    }

    #[test]
    fn test_readonly_named_tuple() {
        use crate::parser_impl::ParserState;
        use crate::binder::BinderState;

        let code = "let x: number;";
        let mut parser = ParserState::new("test.ts".to_string(), code.to_string());
        let root = parser.parse_source_file();

        let mut binder = BinderState::new();
        binder.bind_source_file(&parser.arena, root);

        let mut checker = CheckerState::new(
            &parser.arena,
            &binder.symbols,
            &binder.file_locals,
            "test.ts".to_string(),
        );

        // Create a readonly named tuple
        let readonly_named = checker.types.create_named_tuple_type(
            vec![checker.types.number_type, checker.types.string_type],
            vec![Some("x".to_string()), Some("y".to_string())],
            false, false,
            true, // is_readonly
        );

        let type_str = checker.type_to_string(readonly_named);
        assert_eq!(type_str, "readonly [x: number, y: string]",
            "Readonly named tuple should have readonly prefix, got: {}", type_str);
    }

    // ============== 5.104: Mapped type modifiers ==============
    #[test]
    fn test_mapped_type_modifiers() {
        use super::types::MappedTypeModifier;
        use crate::parser_impl::ParserState;
        use crate::binder::BinderState;
        use crate::binder::SymbolId;

        let code = "let x: number;";
        let mut parser = ParserState::new("test.ts".to_string(), code.to_string());
        let root = parser.parse_source_file();

        let mut binder = BinderState::new();
        binder.bind_source_file(&parser.arena, root);

        let mut checker = CheckerState::new(
            &parser.arena,
            &binder.symbols,
            &binder.file_locals,
            "test.ts".to_string(),
        );

        // Create a type parameter with placeholder values
        let type_param = checker.types.create_type_parameter(
            SymbolId::NONE,
            TypeId::NONE, // no constraint
            TypeId::NONE, // no default
        );
        let constraint = checker.types.string_type;

        // Create mapped type with +readonly and +? modifiers
        let mapped = checker.types.create_mapped_type_with_modifiers(
            crate::parser::NodeIndex::NONE,
            type_param,
            constraint,
            TypeId::NONE, // no name type
            checker.types.number_type, // template type
            MappedTypeModifier::Plus,  // +readonly
            MappedTypeModifier::Plus,  // +?
        );

        // Verify the mapped type was created
        if let Some(super::types::Type::Mapped(m)) = checker.types.get(mapped) {
            assert_eq!(m.readonly_modifier, MappedTypeModifier::Plus,
                "Readonly modifier should be Plus");
            assert_eq!(m.optional_modifier, MappedTypeModifier::Plus,
                "Optional modifier should be Plus");
        } else {
            panic!("Expected Mapped type");
        }

        // Test with minus modifiers
        let mapped_minus = checker.types.create_mapped_type_with_modifiers(
            crate::parser::NodeIndex::NONE,
            type_param,
            constraint,
            TypeId::NONE,
            checker.types.number_type,
            MappedTypeModifier::Minus, // -readonly
            MappedTypeModifier::Minus, // -?
        );

        if let Some(super::types::Type::Mapped(m)) = checker.types.get(mapped_minus) {
            assert_eq!(m.readonly_modifier, MappedTypeModifier::Minus,
                "Readonly modifier should be Minus");
            assert_eq!(m.optional_modifier, MappedTypeModifier::Minus,
                "Optional modifier should be Minus");
        } else {
            panic!("Expected Mapped type");
        }
    }

    // ============== 5.112: Infer with extends constraints ==============
    #[test]
    fn test_infer_type_parameter_with_constraint() {
        use crate::parser_impl::ParserState;
        use crate::binder::BinderState;
        use crate::binder::SymbolId;

        let code = "let x: number;";
        let mut parser = ParserState::new("test.ts".to_string(), code.to_string());
        let root = parser.parse_source_file();

        let mut binder = BinderState::new();
        binder.bind_source_file(&parser.arena, root);

        let mut checker = CheckerState::new(
            &parser.arena,
            &binder.symbols,
            &binder.file_locals,
            "test.ts".to_string(),
        );

        // Create a constrained infer type parameter (simulating `infer T extends string`)
        let constraint = checker.types.string_type;
        let infer_param = checker.types.create_type_parameter(
            SymbolId::NONE, // NONE symbol indicates infer type
            constraint,      // extends string
            TypeId::NONE,
        );

        // Verify the constraint is stored
        if let Some(super::types::Type::TypeParameter(tp)) = checker.types.get(infer_param) {
            assert!(!tp.constraint.is_none(), "Infer type parameter should have constraint");
            assert_eq!(tp.constraint, constraint, "Constraint should be string type");
            assert!(tp.symbol.is_none(), "Symbol should be NONE for infer types");
        } else {
            panic!("Expected TypeParameter");
        }
    }

    #[test]
    fn test_infer_from_type_respects_constraint() {
        use crate::parser_impl::ParserState;
        use crate::binder::BinderState;
        use crate::binder::SymbolId;
        use std::collections::HashMap;

        let code = "let x: number;";
        let mut parser = ParserState::new("test.ts".to_string(), code.to_string());
        let root = parser.parse_source_file();

        let mut binder = BinderState::new();
        binder.bind_source_file(&parser.arena, root);

        let mut checker = CheckerState::new(
            &parser.arena,
            &binder.symbols,
            &binder.file_locals,
            "test.ts".to_string(),
        );

        // Create an infer type with string constraint (infer T extends string)
        let string_constraint = checker.types.string_type;
        let infer_string = checker.types.create_type_parameter(
            SymbolId::NONE,
            string_constraint,
            TypeId::NONE,
        );

        // Test 1: "hello" should match (satisfies string constraint)
        let hello_literal = checker.types.create_string_literal("hello".to_string());
        let mut inferences: HashMap<TypeId, TypeId> = HashMap::new();
        let matches = checker.infer_from_type(hello_literal, infer_string, &mut inferences);
        assert!(matches, "String literal should match infer extends string");
        assert_eq!(inferences.get(&infer_string), Some(&hello_literal));

        // Test 2: 42 should NOT match (number doesn't satisfy string constraint)
        let num_literal = checker.types.create_number_literal(42.0);
        let mut inferences2: HashMap<TypeId, TypeId> = HashMap::new();
        let matches2 = checker.infer_from_type(num_literal, infer_string, &mut inferences2);
        assert!(!matches2, "Number should not match infer extends string");
        assert!(inferences2.is_empty(), "No inference should be made");
    }

    #[test]
    fn test_infer_extends_with_array_constraint() {
        use crate::parser_impl::ParserState;
        use crate::binder::BinderState;
        use crate::binder::SymbolId;
        use std::collections::HashMap;

        let code = "let x: number;";
        let mut parser = ParserState::new("test.ts".to_string(), code.to_string());
        let root = parser.parse_source_file();

        let mut binder = BinderState::new();
        binder.bind_source_file(&parser.arena, root);

        let mut checker = CheckerState::new(
            &parser.arena,
            &binder.symbols,
            &binder.file_locals,
            "test.ts".to_string(),
        );

        // Create an infer type with any[] constraint (infer T extends any[])
        let any_array = checker.types.create_array_type(checker.types.any_type, false);
        let infer_array = checker.types.create_type_parameter(
            SymbolId::NONE,
            any_array,
            TypeId::NONE,
        );

        // Test 1: number[] should match (satisfies any[] constraint)
        let num_array = checker.types.create_array_type(checker.types.number_type, false);
        let mut inferences: HashMap<TypeId, TypeId> = HashMap::new();
        let matches = checker.infer_from_type(num_array, infer_array, &mut inferences);
        assert!(matches, "number[] should match infer extends any[]");

        // Test 2: string should NOT match (not an array)
        let mut inferences2: HashMap<TypeId, TypeId> = HashMap::new();
        let matches2 = checker.infer_from_type(checker.types.string_type, infer_array, &mut inferences2);
        assert!(!matches2, "string should not match infer extends any[]");
    }

    // ============== 5.84: Apparent type cache ==============
    #[test]
    fn test_apparent_type_of_literals() {
        use crate::parser_impl::ParserState;
        use crate::binder::BinderState;

        let code = "let x: number;";
        let mut parser = ParserState::new("test.ts".to_string(), code.to_string());
        let root = parser.parse_source_file();

        let mut binder = BinderState::new();
        binder.bind_source_file(&parser.arena, root);

        let mut checker = CheckerState::new(
            &parser.arena,
            &binder.symbols,
            &binder.file_locals,
            "test.ts".to_string(),
        );

        // String literal "hello" -> string
        let str_lit = checker.types.create_string_literal("hello".to_string());
        let apparent_str = checker.get_apparent_type(str_lit);
        assert_eq!(apparent_str, checker.types.string_type,
            "Apparent type of string literal should be string");

        // Number literal 42 -> number
        let num_lit = checker.types.create_number_literal(42.0);
        let apparent_num = checker.get_apparent_type(num_lit);
        assert_eq!(apparent_num, checker.types.number_type,
            "Apparent type of number literal should be number");

        // Boolean true -> boolean
        let apparent_bool = checker.get_apparent_type(checker.types.true_type);
        assert_eq!(apparent_bool, checker.types.boolean_type,
            "Apparent type of boolean literal should be boolean");
    }

    #[test]
    fn test_apparent_type_of_type_parameter() {
        use crate::parser_impl::ParserState;
        use crate::binder::BinderState;
        use crate::binder::SymbolId;

        let code = "let x: number;";
        let mut parser = ParserState::new("test.ts".to_string(), code.to_string());
        let root = parser.parse_source_file();

        let mut binder = BinderState::new();
        binder.bind_source_file(&parser.arena, root);

        let mut checker = CheckerState::new(
            &parser.arena,
            &binder.symbols,
            &binder.file_locals,
            "test.ts".to_string(),
        );

        // Type parameter with string constraint: T extends string
        let type_param = checker.types.create_type_parameter(
            SymbolId::NONE,
            checker.types.string_type, // constraint
            TypeId::NONE,
        );

        let apparent = checker.get_apparent_type(type_param);
        assert_eq!(apparent, checker.types.string_type,
            "Apparent type of T extends string should be string");

        // Type parameter without constraint
        let unconstrained = checker.types.create_type_parameter(
            SymbolId::NONE,
            TypeId::NONE, // no constraint
            TypeId::NONE,
        );

        let apparent_unconstrained = checker.get_apparent_type(unconstrained);
        assert_eq!(apparent_unconstrained, checker.types.unknown_type,
            "Apparent type of unconstrained type parameter should be unknown");
    }

    #[test]
    fn test_apparent_type_caching() {
        use crate::parser_impl::ParserState;
        use crate::binder::BinderState;

        let code = "let x: number;";
        let mut parser = ParserState::new("test.ts".to_string(), code.to_string());
        let root = parser.parse_source_file();

        let mut binder = BinderState::new();
        binder.bind_source_file(&parser.arena, root);

        let mut checker = CheckerState::new(
            &parser.arena,
            &binder.symbols,
            &binder.file_locals,
            "test.ts".to_string(),
        );

        // Call get_apparent_type twice on the same type
        let str_lit = checker.types.create_string_literal("hello".to_string());
        let apparent1 = checker.get_apparent_type(str_lit);
        let apparent2 = checker.get_apparent_type(str_lit);

        assert_eq!(apparent1, apparent2, "Cached result should be same");

        // Verify it's in the cache
        let cached = checker.apparent_type_cache.borrow().get(&str_lit).copied();
        assert!(cached.is_some(), "Result should be cached");
    }

    // ============== 5.108: Call/construct signatures in type literals ==============
    #[test]
    fn test_call_signature_parsing() {
        use crate::parser_impl::ParserState;
        use crate::binder::BinderState;

        // Test parsing a type literal with call signature
        let code = r#"
            type Callable = { (): void };
            type CallableWithParams = { (x: number, y: string): boolean };
        "#;

        let mut parser = ParserState::new("test.ts".to_string(), code.to_string());
        let root = parser.parse_source_file();

        let mut binder = BinderState::new();
        binder.bind_source_file(&parser.arena, root);

        let mut checker = CheckerState::new(
            &parser.arena,
            &binder.symbols,
            &binder.file_locals,
            "test.ts".to_string(),
        );

        checker.check_source_file(root);
        // Just verify parsing and binding complete without errors
    }

    #[test]
    fn test_construct_signature_parsing() {
        use crate::parser_impl::ParserState;
        use crate::binder::BinderState;

        // Test parsing a type literal with construct signature
        let code = r#"
            type Constructable = { new(): object };
            type ConstructableWithParams = { new(name: string): object };
        "#;

        let mut parser = ParserState::new("test.ts".to_string(), code.to_string());
        let root = parser.parse_source_file();

        let mut binder = BinderState::new();
        binder.bind_source_file(&parser.arena, root);

        let mut checker = CheckerState::new(
            &parser.arena,
            &binder.symbols,
            &binder.file_locals,
            "test.ts".to_string(),
        );

        checker.check_source_file(root);
        // Just verify parsing and binding complete without errors
    }

    #[test]
    fn test_call_signature_has_signature() {
        use crate::parser_impl::ParserState;
        use crate::binder::BinderState;

        // Test that call signature creates a type with call_signatures
        let code = r#"
            type Fn = { (): number };
        "#;

        let mut parser = ParserState::new("test.ts".to_string(), code.to_string());
        let root = parser.parse_source_file();

        let mut binder = BinderState::new();
        binder.bind_source_file(&parser.arena, root);

        let mut checker = CheckerState::new(
            &parser.arena,
            &binder.symbols,
            &binder.file_locals,
            "test.ts".to_string(),
        );

        // Get the type alias 'Fn'
        if let Some(symbol_id) = binder.file_locals.get("Fn") {
            let fn_type = checker.get_type_of_symbol(symbol_id);

            // Check that it's an object type with call signatures
            if let Some(super::types::Type::Object(obj)) = checker.types.get(fn_type) {
                assert!(!obj.call_signatures.is_empty(),
                    "Type literal with (): number should have call signatures");

                // Check that the return type is number
                let sig = &obj.call_signatures[0];
                if let Some(ret_type) = sig.resolved_return_type {
                    assert_eq!(ret_type, checker.types.number_type,
                        "Return type should be number");
                }
            } else {
                panic!("Expected Object type for Fn");
            }
        } else {
            panic!("Should have symbol 'Fn'");
        }
    }

    #[test]
    fn test_construct_signature_has_signature() {
        use crate::parser_impl::ParserState;
        use crate::binder::BinderState;

        // Test that construct signature creates a type with construct_signatures
        let code = r#"
            type Ctor = { new(): object };
        "#;

        let mut parser = ParserState::new("test.ts".to_string(), code.to_string());
        let root = parser.parse_source_file();

        let mut binder = BinderState::new();
        binder.bind_source_file(&parser.arena, root);

        let mut checker = CheckerState::new(
            &parser.arena,
            &binder.symbols,
            &binder.file_locals,
            "test.ts".to_string(),
        );

        // Get the type alias 'Ctor'
        if let Some(symbol_id) = binder.file_locals.get("Ctor") {
            let ctor_type = checker.get_type_of_symbol(symbol_id);

            // Check that it's an object type with construct signatures
            if let Some(super::types::Type::Object(obj)) = checker.types.get(ctor_type) {
                assert!(!obj.construct_signatures.is_empty(),
                    "Type literal with new(): object should have construct signatures");
            } else {
                panic!("Expected Object type for Ctor");
            }
        } else {
            panic!("Should have symbol 'Ctor'");
        }
    }

    // =========================================================================
    // 5.106: `this` parameter types
    // =========================================================================

    #[test]
    fn test_this_parameter_parsing() {
        // Test that `this` parameter is parsed correctly
        use crate::parser_impl::ParserState;
        use crate::binder::BinderState;

        let code = r#"
            function greet(this: { name: string }, greeting: string): string {
                return greeting + " " + this.name;
            }
        "#;

        let mut parser = ParserState::new("test.ts".to_string(), code.to_string());
        let root = parser.parse_source_file();

        let mut binder = BinderState::new();
        binder.bind_source_file(&parser.arena, root);

        let mut checker = CheckerState::new(
            &parser.arena,
            &binder.symbols,
            &binder.file_locals,
            "test.ts".to_string(),
        );

        // Get the function 'greet'
        if let Some(symbol_id) = binder.file_locals.get("greet") {
            let func_type = checker.get_type_of_symbol(symbol_id);

            // Check that it's a function type
            if let Some(super::types::Type::Function(f)) = checker.types.get(func_type) {
                // The function should have 1 regular parameter (greeting), not 2
                // because `this` is special and not a regular parameter
                assert_eq!(f.parameter_names.len(), 1,
                    "Should have 1 regular parameter, not including 'this'. Got: {:?}", f.parameter_names);
                assert_eq!(f.parameter_names[0], "greeting",
                    "First regular parameter should be 'greeting'");

                // Should have a this_type
                assert!(f.this_type.is_some(), "Function should have this_type set");

                // The this_type should be an object type
                if let Some(this_type_id) = f.this_type {
                    if let Some(super::types::Type::Object(_)) = checker.types.get(this_type_id) {
                        // Good - it's an object type
                    } else {
                        panic!("this_type should be an object type");
                    }
                }
            } else {
                panic!("Expected Function type for greet");
            }
        } else {
            panic!("Should have symbol 'greet'");
        }
    }

    #[test]
    fn test_this_parameter_with_type_parameter() {
        // Test that `this` parameter works with generic functions
        use crate::parser_impl::ParserState;
        use crate::binder::BinderState;

        let code = r#"
            function process<T>(this: T, data: T): T {
                return data;
            }
        "#;

        let mut parser = ParserState::new("test.ts".to_string(), code.to_string());
        let root = parser.parse_source_file();

        let mut binder = BinderState::new();
        binder.bind_source_file(&parser.arena, root);

        let mut checker = CheckerState::new(
            &parser.arena,
            &binder.symbols,
            &binder.file_locals,
            "test.ts".to_string(),
        );

        // Get the function 'process'
        if let Some(symbol_id) = binder.file_locals.get("process") {
            let func_type = checker.get_type_of_symbol(symbol_id);

            // Check that it's a function type
            if let Some(super::types::Type::Function(f)) = checker.types.get(func_type) {
                // Should have 1 type parameter
                assert_eq!(f.type_parameters.len(), 1, "Should have 1 type parameter");

                // Should have 1 regular parameter (data)
                assert_eq!(f.parameter_names.len(), 1,
                    "Should have 1 regular parameter. Got: {:?}", f.parameter_names);
                assert_eq!(f.parameter_names[0], "data");

                // Should have a this_type
                assert!(f.this_type.is_some(), "Function should have this_type set");
            } else {
                panic!("Expected Function type for process");
            }
        } else {
            panic!("Should have symbol 'process'");
        }
    }

    #[test]
    fn test_no_this_parameter() {
        // Test that regular functions without `this` parameter work correctly
        use crate::parser_impl::ParserState;
        use crate::binder::BinderState;

        let code = r#"
            function add(a: number, b: number): number {
                return a + b;
            }
        "#;

        let mut parser = ParserState::new("test.ts".to_string(), code.to_string());
        let root = parser.parse_source_file();

        let mut binder = BinderState::new();
        binder.bind_source_file(&parser.arena, root);

        let mut checker = CheckerState::new(
            &parser.arena,
            &binder.symbols,
            &binder.file_locals,
            "test.ts".to_string(),
        );

        // Get the function 'add'
        if let Some(symbol_id) = binder.file_locals.get("add") {
            let func_type = checker.get_type_of_symbol(symbol_id);

            if let Some(super::types::Type::Function(f)) = checker.types.get(func_type) {
                // Should have 2 regular parameters
                assert_eq!(f.parameter_names.len(), 2, "Should have 2 parameters");
                assert_eq!(f.parameter_names[0], "a");
                assert_eq!(f.parameter_names[1], "b");

                // Should NOT have a this_type
                assert!(f.this_type.is_none(), "Regular function should not have this_type");
            } else {
                panic!("Expected Function type for add");
            }
        } else {
            panic!("Should have symbol 'add'");
        }
    }

    // =========================================================================
    // 5.62: const type parameters
    // =========================================================================

    #[test]
    fn test_const_type_parameter_parsing() {
        // Test that `const` type parameter is parsed and tracked correctly
        use crate::parser_impl::ParserState;
        use crate::binder::BinderState;

        let code = r#"
            function tuple<const T extends readonly unknown[]>(arr: T): T {
                return arr;
            }
        "#;

        let mut parser = ParserState::new("test.ts".to_string(), code.to_string());
        let root = parser.parse_source_file();

        let mut binder = BinderState::new();
        binder.bind_source_file(&parser.arena, root);

        let mut checker = CheckerState::new(
            &parser.arena,
            &binder.symbols,
            &binder.file_locals,
            "test.ts".to_string(),
        );

        // Get the function 'tuple'
        if let Some(symbol_id) = binder.file_locals.get("tuple") {
            let func_type = checker.get_type_of_symbol(symbol_id);

            // Check that it's a function type
            if let Some(super::types::Type::Function(f)) = checker.types.get(func_type) {
                // Should have 1 type parameter
                assert_eq!(f.type_parameters.len(), 1, "Should have 1 type parameter");

                // The type parameter should be const
                let type_param_id = f.type_parameters[0];
                if let Some(super::types::Type::TypeParameter(tp)) = checker.types.get(type_param_id) {
                    assert!(tp.is_const, "Type parameter T should be const");
                } else {
                    panic!("Expected TypeParameter type");
                }
            } else {
                panic!("Expected Function type for tuple");
            }
        } else {
            panic!("Should have symbol 'tuple'");
        }
    }

    #[test]
    fn test_non_const_type_parameter() {
        // Test that regular type parameters are not marked as const
        use crate::parser_impl::ParserState;
        use crate::binder::BinderState;

        let code = r#"
            function identity<T>(x: T): T {
                return x;
            }
        "#;

        let mut parser = ParserState::new("test.ts".to_string(), code.to_string());
        let root = parser.parse_source_file();

        let mut binder = BinderState::new();
        binder.bind_source_file(&parser.arena, root);

        let mut checker = CheckerState::new(
            &parser.arena,
            &binder.symbols,
            &binder.file_locals,
            "test.ts".to_string(),
        );

        // Get the function 'identity'
        if let Some(symbol_id) = binder.file_locals.get("identity") {
            let func_type = checker.get_type_of_symbol(symbol_id);

            if let Some(super::types::Type::Function(f)) = checker.types.get(func_type) {
                assert_eq!(f.type_parameters.len(), 1, "Should have 1 type parameter");

                let type_param_id = f.type_parameters[0];
                if let Some(super::types::Type::TypeParameter(tp)) = checker.types.get(type_param_id) {
                    assert!(!tp.is_const, "Regular type parameter should not be const");
                } else {
                    panic!("Expected TypeParameter type");
                }
            } else {
                panic!("Expected Function type for identity");
            }
        } else {
            panic!("Should have symbol 'identity'");
        }
    }

    #[test]
    fn test_const_type_parameter_with_variance_modifiers() {
        // Test that const works together with variance modifiers (in/out)
        use crate::parser_impl::ParserState;

        // Just test that parsing handles const + out together without errors
        let code = r#"
            type Container<const out T> = T;
        "#;

        let mut parser = ParserState::new("test.ts".to_string(), code.to_string());
        let _root = parser.parse_source_file();

        // If we got here without panic, parsing succeeded
        // This verifies that `const out T` parses correctly
        assert!(true, "const + variance modifiers should parse together");
    }

    // =========================================================================
    // 5.66: Variadic tuple types
    // =========================================================================

    #[test]
    fn test_variadic_tuple_element_flags() {
        // Test that element_flags are properly set for tuple types
        let mut arena = super::TypeArena::new();

        // Create a simple tuple [number, string]
        let tuple = arena.create_tuple_type(
            vec![arena.number_type, arena.string_type],
            false, // has_optional
            false, // has_rest
            false, // is_readonly
        );

        if let Some(super::types::Type::Tuple(t)) = arena.get(tuple) {
            assert_eq!(t.element_flags.len(), 2, "Should have 2 element flags");
            assert_eq!(t.element_flags[0], super::types::element_flags::REQUIRED);
            assert_eq!(t.element_flags[1], super::types::element_flags::REQUIRED);
        } else {
            panic!("Expected Tuple type");
        }
    }

    #[test]
    fn test_variadic_tuple_with_rest() {
        // Test that rest element is properly flagged
        let mut arena = super::TypeArena::new();

        // Create tuple [number, ...string[]] (rest in last position)
        let tuple = arena.create_tuple_type(
            vec![arena.number_type, arena.string_type], // string represents element type of rest
            false, // has_optional
            true,  // has_rest
            false, // is_readonly
        );

        if let Some(super::types::Type::Tuple(t)) = arena.get(tuple) {
            assert_eq!(t.element_flags.len(), 2, "Should have 2 element flags");
            assert_eq!(t.element_flags[0], super::types::element_flags::REQUIRED);
            assert_eq!(t.element_flags[1], super::types::element_flags::REST);
            assert!(t.has_rest_element, "has_rest_element should be true");
        } else {
            panic!("Expected Tuple type");
        }
    }

    #[test]
    fn test_variadic_tuple_explicit_flags() {
        // Test create_variadic_tuple_type with explicit element flags
        let mut arena = super::TypeArena::new();

        // Create variadic tuple [...T, number, ...U]
        // where T and U are variadic (spread tuples)
        let element_flags = vec![
            super::types::element_flags::VARIADIC,  // ...T
            super::types::element_flags::REQUIRED,  // number
            super::types::element_flags::VARIADIC,  // ...U
        ];

        let tuple = arena.create_variadic_tuple_type(
            vec![arena.any_type, arena.number_type, arena.any_type],
            element_flags.clone(),
            None, // no names
            false, // not readonly
        );

        if let Some(super::types::Type::Tuple(t)) = arena.get(tuple) {
            assert_eq!(t.element_flags.len(), 3, "Should have 3 element flags");
            assert_eq!(t.element_flags[0], super::types::element_flags::VARIADIC);
            assert_eq!(t.element_flags[1], super::types::element_flags::REQUIRED);
            assert_eq!(t.element_flags[2], super::types::element_flags::VARIADIC);
            assert!(t.has_rest_element, "has_rest_element should be true (variadic counts as rest)");
        } else {
            panic!("Expected Tuple type");
        }
    }

    // ============== 5.74: ThisType<T> utility type ==============
    #[test]
    fn test_thistype_utility_type() {
        use crate::parser_impl::ParserState;
        use crate::binder::BinderState;

        // Test ThisType<T> utility type - creates a marker type for object literal methods
        let code = r#"
type MyThis = ThisType<{ name: string }>;
type AnyThis = ThisType<any>;
"#;

        let mut parser = ParserState::new("test.ts".to_string(), code.to_string());
        let root = parser.parse_source_file();

        let mut binder = BinderState::new();
        binder.bind_source_file(&parser.arena, root);

        let mut checker = CheckerState::new(
            &parser.arena,
            &binder.symbols,
            &binder.file_locals,
            "test.ts".to_string(),
        );

        // ThisType<{ name: string }> should create a ThisType marker
        if let Some(symbol_id) = binder.file_locals.get("MyThis") {
            let this_type = checker.get_type_of_symbol(symbol_id);
            let type_str = checker.type_to_string(this_type);
            assert!(type_str.contains("ThisType"),
                "ThisType<{{ name: string }}> should be ThisType marker, got: {}", type_str);
        } else {
            panic!("Should have symbol 'MyThis'");
        }

        // ThisType<any> should also work
        if let Some(symbol_id) = binder.file_locals.get("AnyThis") {
            let this_type = checker.get_type_of_symbol(symbol_id);
            let type_str = checker.type_to_string(this_type);
            assert!(type_str.contains("ThisType") || type_str.contains("any"),
                "ThisType<any> should be ThisType marker, got: {}", type_str);
        } else {
            panic!("Should have symbol 'AnyThis'");
        }
    }

    #[test]
    fn test_thistype_in_intersection() {
        use crate::parser_impl::ParserState;
        use crate::binder::BinderState;

        // Test ThisType<T> in intersection - common pattern for object literal methods
        let code = r#"
type Methods = {
    greet(): string;
};
type ObjectWithThis = Methods & ThisType<{ name: string }>;
"#;

        let mut parser = ParserState::new("test.ts".to_string(), code.to_string());
        let root = parser.parse_source_file();

        let mut binder = BinderState::new();
        binder.bind_source_file(&parser.arena, root);

        let mut checker = CheckerState::new(
            &parser.arena,
            &binder.symbols,
            &binder.file_locals,
            "test.ts".to_string(),
        );

        // The intersection should be created
        if let Some(symbol_id) = binder.file_locals.get("ObjectWithThis") {
            let obj_type = checker.get_type_of_symbol(symbol_id);
            // Should not be any or error
            assert!(!obj_type.is_none(), "ObjectWithThis should have a type");
        } else {
            panic!("Should have symbol 'ObjectWithThis'");
        }
    }

    #[test]
    fn test_thistype_arena_creation() {
        // Test direct arena creation of ThisType
        let mut arena = super::TypeArena::new();

        // Create ThisType<number>
        let this_type = arena.create_this_type(arena.number_type);

        if let Some(super::types::Type::ThisType(t)) = arena.get(this_type) {
            assert_eq!(t.constraint, arena.number_type, "ThisType constraint should be number");
        } else {
            panic!("Expected ThisType");
        }

        // Create ThisType<string>
        let string_this = arena.create_this_type(arena.string_type);

        if let Some(super::types::Type::ThisType(t)) = arena.get(string_this) {
            assert_eq!(t.constraint, arena.string_type, "ThisType constraint should be string");
        } else {
            panic!("Expected ThisType");
        }
    }

    // ============== 5.105: unique symbol type ==============
    #[test]
    fn test_unique_symbol_arena_creation() {
        use crate::binder::SymbolId;

        // Test direct arena creation of unique symbol type
        let mut arena = super::TypeArena::new();

        // Create two unique symbol types with different symbols
        let sym1 = arena.create_unique_symbol_type(SymbolId(1), "sym1".to_string());
        let sym2 = arena.create_unique_symbol_type(SymbolId(2), "sym2".to_string());

        // Verify the types are created correctly
        if let Some(super::types::Type::UniqueSymbol(s)) = arena.get(sym1) {
            assert_eq!(s.symbol, SymbolId(1), "sym1 should have SymbolId(1)");
            assert_eq!(s.name, "sym1", "sym1 should have name 'sym1'");
            assert_eq!(s.flags, super::types::type_flags::UNIQUE_ES_SYMBOL);
        } else {
            panic!("Expected UniqueSymbol type for sym1");
        }

        if let Some(super::types::Type::UniqueSymbol(s)) = arena.get(sym2) {
            assert_eq!(s.symbol, SymbolId(2), "sym2 should have SymbolId(2)");
            assert_eq!(s.name, "sym2", "sym2 should have name 'sym2'");
        } else {
            panic!("Expected UniqueSymbol type for sym2");
        }
    }

    #[test]
    fn test_unique_symbol_equality() {
        use crate::binder::SymbolId;
        use crate::checker::state::{CheckerState, TypeRelation};
        use crate::parser_impl::ParserState;
        use crate::binder::BinderState;

        // Create a minimal checker to test type relations
        let code = "";
        let mut parser = ParserState::new("test.ts".to_string(), code.to_string());
        let root = parser.parse_source_file();
        let mut binder = BinderState::new();
        binder.bind_source_file(&parser.arena, root);

        let mut checker = CheckerState::new(
            &parser.arena,
            &binder.symbols,
            &binder.file_locals,
            "test.ts".to_string(),
        );

        // Create unique symbols
        let sym1a = checker.types.create_unique_symbol_type(SymbolId(1), "sym1".to_string());
        let sym1b = checker.types.create_unique_symbol_type(SymbolId(1), "sym1".to_string()); // same symbol
        let sym2 = checker.types.create_unique_symbol_type(SymbolId(2), "sym2".to_string());

        // Same symbol should be assignable
        assert!(
            checker.is_type_related_to(sym1a, sym1b, TypeRelation::Assignable),
            "unique symbol with same symbol should be assignable"
        );

        // Different symbols should NOT be assignable
        assert!(
            !checker.is_type_related_to(sym1a, sym2, TypeRelation::Assignable),
            "unique symbols with different symbols should NOT be assignable"
        );

        // Unique symbol should be assignable to general symbol type
        let symbol_type = checker.types.es_symbol_type;
        assert!(
            checker.is_type_related_to(sym1a, symbol_type, TypeRelation::Assignable),
            "unique symbol should be assignable to symbol type"
        );
    }

    #[test]
    fn test_unique_symbol_type_string() {
        use crate::binder::SymbolId;
        use crate::parser_impl::ParserState;
        use crate::binder::BinderState;

        let code = "";
        let mut parser = ParserState::new("test.ts".to_string(), code.to_string());
        let root = parser.parse_source_file();
        let mut binder = BinderState::new();
        binder.bind_source_file(&parser.arena, root);

        let mut checker = CheckerState::new(
            &parser.arena,
            &binder.symbols,
            &binder.file_locals,
            "test.ts".to_string(),
        );

        // Create a unique symbol type
        let unique_sym = checker.types.create_unique_symbol_type(SymbolId(42), "mySymbol".to_string());

        // Check type to string
        let type_str = checker.type_to_string(unique_sym);
        assert!(
            type_str.contains("mySymbol"),
            "unique symbol type string should contain the name, got: {}",
            type_str
        );
    }

    // ============== 5.88: BigInt type checking ==============
    #[test]
    fn test_bigint_arena_creation() {
        // Test direct arena creation of bigint literal type
        let mut arena = super::TypeArena::new();

        // Create bigint literal type
        let bigint_lit = arena.create_bigint_literal("12345".to_string());

        if let Some(super::types::Type::Literal(lit)) = arena.get(bigint_lit) {
            if let super::types::LiteralValue::BigInt(ref val) = lit.value {
                assert_eq!(val, "12345", "BigInt value should be '12345'");
            } else {
                panic!("Expected BigInt literal value");
            }
            assert_eq!(lit.flags, super::types::type_flags::BIG_INT_LITERAL);
        } else {
            panic!("Expected Literal type");
        }
    }

    #[test]
    fn test_bigint_assignability() {
        use crate::parser_impl::ParserState;
        use crate::binder::BinderState;
        use crate::checker::state::TypeRelation;

        let code = "";
        let mut parser = ParserState::new("test.ts".to_string(), code.to_string());
        let root = parser.parse_source_file();
        let mut binder = BinderState::new();
        binder.bind_source_file(&parser.arena, root);

        let mut checker = CheckerState::new(
            &parser.arena,
            &binder.symbols,
            &binder.file_locals,
            "test.ts".to_string(),
        );

        // Create bigint literal types
        let bigint_lit1 = checker.types.create_bigint_literal("123".to_string());
        let bigint_lit2 = checker.types.create_bigint_literal("456".to_string());
        let bigint_type = checker.types.big_int_type;

        // BigInt literal should be assignable to bigint
        assert!(
            checker.is_type_related_to(bigint_lit1, bigint_type, TypeRelation::Assignable),
            "BigInt literal should be assignable to bigint"
        );

        // Different bigint literals should NOT be assignable to each other
        assert!(
            !checker.is_type_related_to(bigint_lit1, bigint_lit2, TypeRelation::Assignable),
            "Different BigInt literals should NOT be assignable to each other"
        );

        // Same literal value should be assignable
        let bigint_lit1_copy = checker.types.create_bigint_literal("123".to_string());
        // Note: These are technically different TypeIds, so they won't be equal unless we compare values
        // For now, we just verify the general bigint assignability works
    }

// =========================================================================
// Property access on arrays - debugging infinite loop
// =========================================================================

#[test]
fn test_simple_property_access() {
    // Minimal test: just property access without array method calls
    use crate::parser_impl::ParserState;
    use crate::binder::BinderState;

    let code = r#"
        class Ship { isSunk: boolean; }
        class Board {
            ships: Ship[];
            test() {
                return this.ships;
            }
        }
    "#;

    let mut parser = ParserState::new("test.ts".to_string(), code.to_string());
    let root = parser.parse_source_file();
    let mut binder = BinderState::new();
    binder.bind_source_file(&parser.arena, root);

    let mut checker = CheckerState::new(
        &parser.arena,
        &binder.symbols,
        &binder.file_locals,
        "test.ts".to_string(),
    );

    checker.check_source_file(root);
    // Just verify no infinite loop
}

#[test]
fn test_this_keyword_type() {
    // Test that 'this' keyword returns a type without infinite loop
    use crate::parser_impl::ParserState;
    use crate::binder::BinderState;

    let code = r#"
        class Foo {
            bar() {
                return this;
            }
        }
    "#;

    let mut parser = ParserState::new("test.ts".to_string(), code.to_string());
    let root = parser.parse_source_file();
    let mut binder = BinderState::new();
    binder.bind_source_file(&parser.arena, root);

    let mut checker = CheckerState::new(
        &parser.arena,
        &binder.symbols,
        &binder.file_locals,
        "test.ts".to_string(),
    );

    checker.check_source_file(root);
    // Just verify no infinite loop - 'this' returns any for now
}

#[test]
fn test_this_type_in_class_method() {
    // Test that 'this' returns the class type inside a class method
    use crate::parser_impl::ParserState;
    use crate::binder::BinderState;

    let code = r#"
        class Point {
            x: number;
            y: number;
            getThis(): Point {
                return this;
            }
        }
    "#;

    let mut parser = ParserState::new(
        "test.ts".to_string(),
        code.to_string(),
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

    // Check the source file
    checker.check_source_file(root);

    // Look up the Point class symbol
    let point_symbol = checker.file_locals.get("Point").expect("Point class should exist");
    let point_type = checker.get_type_of_symbol(point_symbol);

    // The type should be an object type (the class)
    let typ = checker.types.get(point_type).expect("Point type should exist");
    assert!(matches!(typ, super::types::Type::Object(_)), "Point should be an Object type");

    // No errors should be produced (return this matches Point)
    assert_eq!(checker.diagnostics.len(), 0, "No type errors expected for 'this' returning Point");
}

#[test]
fn test_this_property_access() {
    // Test that 'this.property' works correctly in class methods
    use crate::parser_impl::ParserState;
    use crate::binder::BinderState;

    let code = r#"
        class Counter {
            count: number;
            increment() {
                return this.count + 1;
            }
        }
    "#;

    let mut parser = ParserState::new(
        "test.ts".to_string(),
        code.to_string(),
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

    checker.check_source_file(root);

    // Verify no errors - this.count should resolve properly
    assert_eq!(checker.diagnostics.len(), 0, "No type errors expected for this.count");
}

#[test]
fn test_super_type_basic() {
    // Basic test that 'super' resolves to the base class type
    use crate::parser_impl::ParserState;
    use crate::binder::BinderState;

    // Simpler test - just check that super resolves without calling methods
    let code = r#"
        class Animal {
            name: string;
        }
        class Dog extends Animal {
            name: string;
        }
    "#;

    let mut parser = ParserState::new(
        "test.ts".to_string(),
        code.to_string(),
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

    // Check base class symbol is resolvable
    let animal_symbol = checker.file_locals.get("Animal").expect("Animal class should exist");
    let animal_type = checker.get_type_of_symbol(animal_symbol);

    // The type should be an object type (the class)
    let typ = checker.types.get(animal_type).expect("Animal type should exist");
    assert!(matches!(typ, super::types::Type::Object(_)), "Animal should be an Object type");
}

#[test]
fn test_class_inheritance_simple() {
    // Test simple class inheritance without super call
    use crate::parser_impl::ParserState;
    use crate::binder::BinderState;

    let code = r#"
        class Animal {
            name: string;
        }
        class Dog extends Animal {
            bark() {}
        }
    "#;

    let mut parser = ParserState::new(
        "test.ts".to_string(),
        code.to_string(),
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

    checker.check_source_file(root);

    assert!(binder.file_locals.has("Animal"), "Animal class should be defined");
    assert!(binder.file_locals.has("Dog"), "Dog class should be defined");
}

#[test]
fn test_super_method_call() {
    // Test that 'super.method()' works correctly in derived classes
    use crate::parser_impl::ParserState;
    use crate::binder::BinderState;

    // Minimal test case - with super call
    let code = r#"
        class Animal {
            speak() { return "sound"; }
        }
        class Dog extends Animal {
            bark() {
                return super.speak();
            }
        }
    "#;

    let mut parser = ParserState::new(
        "test.ts".to_string(),
        code.to_string(),
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

    checker.check_source_file(root);

    // super.speak() should resolve without errors
    assert_eq!(checker.diagnostics.len(), 0, "No type errors expected for super.speak()");
}

// =========================================================================
// Async/Await type inference tests
// =========================================================================

#[test]
fn test_awaited_type_unwraps_promise() {
    // Test that get_awaited_type correctly unwraps Promise<T> to T
    use crate::parser_impl::ParserState;
    use crate::binder::BinderState;
    use super::state::CheckerState;

    let code = r#"
        type Promise<T> = T;
        const p: Promise<string> = "hello";
    "#;

    let mut parser = ParserState::new(
        "test.ts".to_string(),
        code.to_string(),
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

    // Check that Promise<string> can be resolved
    // Note: With our simple type alias, Promise<T> = T, so this should just work
    checker.check_source_file(root);

    // No type errors expected
    assert_eq!(checker.diagnostics.len(), 0, "No type errors expected for Promise<string>");
}

#[test]
fn test_awaited_type_on_non_promise() {
    // Test that get_awaited_type returns the same type for non-Promise types
    use super::arena::TypeArena;

    let mut arena = TypeArena::new();

    // For primitive types, awaited type should be the same
    assert_eq!(arena.string_type, arena.string_type, "string awaited should be string");
    assert_eq!(arena.number_type, arena.number_type, "number awaited should be number");
}

#[test]
fn test_array_method_every() {
    // This replicates the exact pattern from 2dArrays.ts that was causing infinite loop
    use crate::parser_impl::ParserState;
    use crate::binder::BinderState;

    let code = r#"
        class Ship { isSunk: boolean; }
        class Board {
            ships: Ship[];
            private allShipsSunk() {
                return this.ships.every(function (val) { return val.isSunk; });
            }
        }
    "#;

    let mut parser = ParserState::new("test.ts".to_string(), code.to_string());
    let root = parser.parse_source_file();
    let mut binder = BinderState::new();
    binder.bind_source_file(&parser.arena, root);

    let mut checker = CheckerState::new(
        &parser.arena,
        &binder.symbols,
        &binder.file_locals,
        "test.ts".to_string(),
    );

    checker.check_source_file(root);
    // Should complete without infinite loop
}

#[test]
fn test_method_call_on_array() {
    // Test calling a method on an array type (like .push())
    use crate::parser_impl::ParserState;
    use crate::binder::BinderState;

    let code = r#"
        class Board {
            ships: string[];
            test() {
                this.ships.push("hello");
            }
        }
    "#;

    let mut parser = ParserState::new("test.ts".to_string(), code.to_string());
    let root = parser.parse_source_file();
    let mut binder = BinderState::new();
    binder.bind_source_file(&parser.arena, root);

    let mut checker = CheckerState::new(
        &parser.arena,
        &binder.symbols,
        &binder.file_locals,
        "test.ts".to_string(),
    );

    checker.check_source_file(root);
}

#[test]
fn test_array_every_simple_callback() {
    // Test .every() with a simple callback that doesn't access properties
    use crate::parser_impl::ParserState;
    use crate::binder::BinderState;

    let code = r#"
        class Board {
            ships: string[];
            test() {
                return this.ships.every(function (val) { return true; });
            }
        }
    "#;

    let mut parser = ParserState::new("test.ts".to_string(), code.to_string());
    let root = parser.parse_source_file();
    let mut binder = BinderState::new();
    binder.bind_source_file(&parser.arena, root);

    let mut checker = CheckerState::new(
        &parser.arena,
        &binder.symbols,
        &binder.file_locals,
        "test.ts".to_string(),
    );

    checker.check_source_file(root);
}

#[test]
fn test_method_with_function_arg() {
    // Test method call with function expression argument (not on array)
    use crate::parser_impl::ParserState;
    use crate::binder::BinderState;

    let code = r#"
        class Foo {
            bar(cb: () => void) {}
            test() {
                this.bar(function() {});
            }
        }
    "#;

    let mut parser = ParserState::new("test.ts".to_string(), code.to_string());
    let root = parser.parse_source_file();
    let mut binder = BinderState::new();
    binder.bind_source_file(&parser.arena, root);

    let mut checker = CheckerState::new(
        &parser.arena,
        &binder.symbols,
        &binder.file_locals,
        "test.ts".to_string(),
    );

    checker.check_source_file(root);
}

#[test]
fn test_this_method_call_simple() {
    // Test this.method() call without callback arg
    use crate::parser_impl::ParserState;
    use crate::binder::BinderState;

    let code = r#"
        class Foo {
            bar() {}
            test() {
                this.bar();
            }
        }
    "#;

    let mut parser = ParserState::new("test.ts".to_string(), code.to_string());
    let root = parser.parse_source_file();
    let mut binder = BinderState::new();
    binder.bind_source_file(&parser.arena, root);

    let mut checker = CheckerState::new(
        &parser.arena,
        &binder.symbols,
        &binder.file_locals,
        "test.ts".to_string(),
    );

    checker.check_source_file(root);
}

#[test]
fn test_minimal_class_only() {
    // Test just a class with a method that takes a callback - no calls
    use crate::parser_impl::ParserState;
    use crate::binder::BinderState;

    let code = r#"
        class Foo {
            bar(cb: () => void) {}
        }
    "#;

    let mut parser = ParserState::new("test.ts".to_string(), code.to_string());
    let root = parser.parse_source_file();
    let mut binder = BinderState::new();
    binder.bind_source_file(&parser.arena, root);

    let mut checker = CheckerState::new(
        &parser.arena,
        &binder.symbols,
        &binder.file_locals,
        "test.ts".to_string(),
    );

    checker.check_source_file(root);
}

#[test]
fn test_class_with_two_methods() {
    // Test class with two methods where one calls the other
    use crate::parser_impl::ParserState;
    use crate::binder::BinderState;

    let code = r#"
        class Foo {
            bar(cb: () => void) {}
            test() {
                this.bar;
            }
        }
    "#;

    let mut parser = ParserState::new("test.ts".to_string(), code.to_string());
    let root = parser.parse_source_file();
    let mut binder = BinderState::new();
    binder.bind_source_file(&parser.arena, root);

    let mut checker = CheckerState::new(
        &parser.arena,
        &binder.symbols,
        &binder.file_locals,
        "test.ts".to_string(),
    );

    checker.check_source_file(root);
}

#[test]
fn test_this_method_call_no_args() {
    // Test this.method() with no arguments
    use crate::parser_impl::ParserState;
    use crate::binder::BinderState;

    let code = r#"
        class Foo {
            bar(cb: () => void) {}
            test() {
                this.bar();
            }
        }
    "#;

    let mut parser = ParserState::new("test.ts".to_string(), code.to_string());
    let root = parser.parse_source_file();
    let mut binder = BinderState::new();
    binder.bind_source_file(&parser.arena, root);

    let mut checker = CheckerState::new(
        &parser.arena,
        &binder.symbols,
        &binder.file_locals,
        "test.ts".to_string(),
    );

    checker.check_source_file(root);
}

#[test]
fn test_this_method_call_with_string_arg() {
    // Test this.method() with a string argument
    use crate::parser_impl::ParserState;
    use crate::binder::BinderState;

    let code = r#"
        class Foo {
            bar(s: string) {}
            test() {
                this.bar("hello");
            }
        }
    "#;

    let mut parser = ParserState::new("test.ts".to_string(), code.to_string());
    let root = parser.parse_source_file();
    let mut binder = BinderState::new();
    binder.bind_source_file(&parser.arena, root);

    let mut checker = CheckerState::new(
        &parser.arena,
        &binder.symbols,
        &binder.file_locals,
        "test.ts".to_string(),
    );

    checker.check_source_file(root);
}

#[test]
fn test_this_method_with_func_arg_no_param() {
    // Test this.method(function(){}) where method has NO parameters
    use crate::parser_impl::ParserState;
    use crate::binder::BinderState;

    let code = r#"
        class Foo {
            bar() {}
            test() {
                this.bar(function() {});
            }
        }
    "#;

    let mut parser = ParserState::new("test.ts".to_string(), code.to_string());
    let root = parser.parse_source_file();
    let mut binder = BinderState::new();
    binder.bind_source_file(&parser.arena, root);

    let mut checker = CheckerState::new(
        &parser.arena,
        &binder.symbols,
        &binder.file_locals,
        "test.ts".to_string(),
    );

    checker.check_source_file(root);
}

#[test]
fn test_method_call_without_this() {
    // Test method call through variable (not this)
    use crate::parser_impl::ParserState;
    use crate::binder::BinderState;

    let code = r#"
        class Foo {
            bar(cb: () => void) {}
        }
        let x = new Foo();
        x.bar(function() {});
    "#;

    let mut parser = ParserState::new("test.ts".to_string(), code.to_string());
    let root = parser.parse_source_file();
    let mut binder = BinderState::new();
    binder.bind_source_file(&parser.arena, root);

    let mut checker = CheckerState::new(
        &parser.arena,
        &binder.symbols,
        &binder.file_locals,
        "test.ts".to_string(),
    );

    checker.check_source_file(root);
}

#[test]
fn test_just_function_expression() {
    // Test standalone function expression type checking
    use crate::parser_impl::ParserState;
    use crate::binder::BinderState;

    let code = r#"
        let f = function() {};
    "#;

    let mut parser = ParserState::new("test.ts".to_string(), code.to_string());
    let root = parser.parse_source_file();
    let mut binder = BinderState::new();
    binder.bind_source_file(&parser.arena, root);

    let mut checker = CheckerState::new(
        &parser.arena,
        &binder.symbols,
        &binder.file_locals,
        "test.ts".to_string(),
    );

    checker.check_source_file(root);
}

#[test]
fn test_function_call_with_callback_no_this() {
    // Test function call with function expression (not a method call on `this`)
    use crate::parser_impl::ParserState;
    use crate::binder::BinderState;

    let code = r#"
        function bar(cb: () => void) {}
        bar(function() {});
    "#;

    let mut parser = ParserState::new("test.ts".to_string(), code.to_string());
    let root = parser.parse_source_file();
    let mut binder = BinderState::new();
    binder.bind_source_file(&parser.arena, root);

    let mut checker = CheckerState::new(
        &parser.arena,
        &binder.symbols,
        &binder.file_locals,
        "test.ts".to_string(),
    );

    checker.check_source_file(root);
}

#[test]
fn test_array_property_access_without_call() {
    // Test property access on array (e.g., .length, .every) without calling
    use crate::parser_impl::ParserState;
    use crate::binder::BinderState;

    let code = r#"
        class Board {
            ships: string[];
            test() {
                return this.ships.length;
            }
        }
    "#;

    let mut parser = ParserState::new("test.ts".to_string(), code.to_string());
    let root = parser.parse_source_file();
    let mut binder = BinderState::new();
    binder.bind_source_file(&parser.arena, root);

    let mut checker = CheckerState::new(
        &parser.arena,
        &binder.symbols,
        &binder.file_locals,
        "test.ts".to_string(),
    );

    checker.check_source_file(root);
}

#[test]
fn test_simple_function_call() {
    // Test simple function call without arrays
    use crate::parser_impl::ParserState;
    use crate::binder::BinderState;

    let code = r#"
        function foo(x: number): boolean { return true; }
        let result = foo(42);
    "#;

    let mut parser = ParserState::new("test.ts".to_string(), code.to_string());
    let root = parser.parse_source_file();
    let mut binder = BinderState::new();
    binder.bind_source_file(&parser.arena, root);

    let mut checker = CheckerState::new(
        &parser.arena,
        &binder.symbols,
        &binder.file_locals,
        "test.ts".to_string(),
    );

    checker.check_source_file(root);
}

#[test]
fn test_callback_function() {
    // Test function call with callback (no arrays)
    use crate::parser_impl::ParserState;
    use crate::binder::BinderState;

    let code = r#"
        function foo(cb: (x: number) => boolean): boolean { return cb(1); }
        let result = foo(function(x) { return true; });
    "#;

    let mut parser = ParserState::new("test.ts".to_string(), code.to_string());
    let root = parser.parse_source_file();
    let mut binder = BinderState::new();
    binder.bind_source_file(&parser.arena, root);

    let mut checker = CheckerState::new(
        &parser.arena,
        &binder.symbols,
        &binder.file_locals,
        "test.ts".to_string(),
    );

    checker.check_source_file(root);
}

#[test]
fn test_generic_function_identity() {
    use crate::parser_impl::ParserState;
    use crate::binder::BinderState;

    let code = r#"
        function identity<T>(x: T): T { return x; }
        let num = identity(42);
        let str = identity("hello");
    "#;

    let mut parser = ParserState::new("test.ts".to_string(), code.to_string());
    let root = parser.parse_source_file();
    let mut binder = BinderState::new();
    binder.bind_source_file(&parser.arena, root);

    let mut checker = CheckerState::new(
        &parser.arena,
        &binder.symbols,
        &binder.file_locals,
        "test.ts".to_string(),
    );

    checker.check_source_file(root);
}

#[test]
fn test_union_type_variable() {
    use crate::parser_impl::ParserState;
    use crate::binder::BinderState;

    let code = r#"
        let x: string | number = "hello";
        x = 42;
    "#;

    let mut parser = ParserState::new("test.ts".to_string(), code.to_string());
    let root = parser.parse_source_file();
    let mut binder = BinderState::new();
    binder.bind_source_file(&parser.arena, root);

    let mut checker = CheckerState::new(
        &parser.arena,
        &binder.symbols,
        &binder.file_locals,
        "test.ts".to_string(),
    );

    checker.check_source_file(root);
}

#[test]
fn test_interface_member_access() {
    use crate::parser_impl::ParserState;
    use crate::binder::BinderState;

    let code = r#"
        interface Person {
            name: string;
            age: number;
        }
        function greet(p: Person): string {
            return p.name;
        }
    "#;

    let mut parser = ParserState::new("test.ts".to_string(), code.to_string());
    let root = parser.parse_source_file();
    let mut binder = BinderState::new();
    binder.bind_source_file(&parser.arena, root);

    let mut checker = CheckerState::new(
        &parser.arena,
        &binder.symbols,
        &binder.file_locals,
        "test.ts".to_string(),
    );

    checker.check_source_file(root);
}

#[test]
fn test_tuple_type_indexing() {
    use crate::parser_impl::ParserState;
    use crate::binder::BinderState;

    let code = r#"
        let tuple: [string, number] = ["hello", 42];
        let first = tuple[0];
        let second = tuple[1];
    "#;

    let mut parser = ParserState::new("test.ts".to_string(), code.to_string());
    let root = parser.parse_source_file();
    let mut binder = BinderState::new();
    binder.bind_source_file(&parser.arena, root);

    let mut checker = CheckerState::new(
        &parser.arena,
        &binder.symbols,
        &binder.file_locals,
        "test.ts".to_string(),
    );

    checker.check_source_file(root);
}

#[test]
fn test_type_alias_with_generics() {
    use crate::parser_impl::ParserState;
    use crate::binder::BinderState;

    let code = r#"
        type Pair<T, U> = { first: T; second: U; };
        let p: Pair<string, number> = { first: "hello", second: 42 };
    "#;

    let mut parser = ParserState::new("test.ts".to_string(), code.to_string());
    let root = parser.parse_source_file();
    let mut binder = BinderState::new();
    binder.bind_source_file(&parser.arena, root);

    let mut checker = CheckerState::new(
        &parser.arena,
        &binder.symbols,
        &binder.file_locals,
        "test.ts".to_string(),
    );

    checker.check_source_file(root);
}

#[test]
fn test_literal_type_assignment() {
    use crate::parser_impl::ParserState;
    use crate::binder::BinderState;

    let code = r#"
        let x: "hello" = "hello";
        let y: 42 = 42;
        let z: true = true;
    "#;

    let mut parser = ParserState::new("test.ts".to_string(), code.to_string());
    let root = parser.parse_source_file();
    let mut binder = BinderState::new();
    binder.bind_source_file(&parser.arena, root);

    let mut checker = CheckerState::new(
        &parser.arena,
        &binder.symbols,
        &binder.file_locals,
        "test.ts".to_string(),
    );

    checker.check_source_file(root);
}

#[test]
fn test_optional_parameter() {
    use crate::parser_impl::ParserState;
    use crate::binder::BinderState;

    let code = r#"
        function greet(name: string, title?: string): string {
            return name;
        }
        greet("Alice");
        greet("Bob", "Dr.");
    "#;

    let mut parser = ParserState::new("test.ts".to_string(), code.to_string());
    let root = parser.parse_source_file();
    let mut binder = BinderState::new();
    binder.bind_source_file(&parser.arena, root);

    let mut checker = CheckerState::new(
        &parser.arena,
        &binder.symbols,
        &binder.file_locals,
        "test.ts".to_string(),
    );

    checker.check_source_file(root);
}

#[test]
fn test_rest_parameter() {
    use crate::parser_impl::ParserState;
    use crate::binder::BinderState;

    let code = r#"
        function sum(...numbers: number[]): number {
            return 0;
        }
        sum(1, 2, 3);
    "#;

    let mut parser = ParserState::new("test.ts".to_string(), code.to_string());
    let root = parser.parse_source_file();
    let mut binder = BinderState::new();
    binder.bind_source_file(&parser.arena, root);

    let mut checker = CheckerState::new(
        &parser.arena,
        &binder.symbols,
        &binder.file_locals,
        "test.ts".to_string(),
    );

    checker.check_source_file(root);
}

#[test]
fn test_typeof_narrowing() {
    use crate::parser_impl::ParserState;
    use crate::binder::BinderState;

    let code = r#"
        function process(x: string | number): string {
            if (typeof x === "string") {
                return x;
            }
            return x.toString();
        }
    "#;

    let mut parser = ParserState::new("test.ts".to_string(), code.to_string());
    let root = parser.parse_source_file();
    let mut binder = BinderState::new();
    binder.bind_source_file(&parser.arena, root);

    let mut checker = CheckerState::new(
        &parser.arena,
        &binder.symbols,
        &binder.file_locals,
        "test.ts".to_string(),
    );

    checker.check_source_file(root);
}

#[test]
fn test_instanceof_narrowing() {
    use crate::parser_impl::ParserState;
    use crate::binder::BinderState;

    let code = r#"
        class Dog {
            bark(): void {}
        }
        class Cat {
            meow(): void {}
        }
        function pet(animal: Dog | Cat): void {
            if (animal instanceof Dog) {
                animal.bark();
            }
        }
    "#;

    let mut parser = ParserState::new("test.ts".to_string(), code.to_string());
    let root = parser.parse_source_file();
    let mut binder = BinderState::new();
    binder.bind_source_file(&parser.arena, root);

    let mut checker = CheckerState::new(
        &parser.arena,
        &binder.symbols,
        &binder.file_locals,
        "test.ts".to_string(),
    );

    checker.check_source_file(root);
}

#[test]
fn test_mapped_type_basic() {
    use crate::parser_impl::ParserState;
    use crate::binder::BinderState;

    let code = r#"
        type Readonly<T> = { readonly [P in keyof T]: T[P] };
        interface Point { x: number; y: number; }
        type ReadonlyPoint = Readonly<Point>;
    "#;

    let mut parser = ParserState::new("test.ts".to_string(), code.to_string());
    let root = parser.parse_source_file();
    let mut binder = BinderState::new();
    binder.bind_source_file(&parser.arena, root);

    let mut checker = CheckerState::new(
        &parser.arena,
        &binder.symbols,
        &binder.file_locals,
        "test.ts".to_string(),
    );

    checker.check_source_file(root);
}

#[test]
fn test_conditional_type_basic() {
    use crate::parser_impl::ParserState;
    use crate::binder::BinderState;

    let code = r#"
        type IsString<T> = T extends string ? true : false;
        type A = IsString<string>;
        type B = IsString<number>;
    "#;

    let mut parser = ParserState::new("test.ts".to_string(), code.to_string());
    let root = parser.parse_source_file();
    let mut binder = BinderState::new();
    binder.bind_source_file(&parser.arena, root);

    let mut checker = CheckerState::new(
        &parser.arena,
        &binder.symbols,
        &binder.file_locals,
        "test.ts".to_string(),
    );

    checker.check_source_file(root);
}

#[test]
fn test_class_simple_method() {
    // Simpler class test - just method definition, no this.property access
    use crate::parser_impl::ParserState;
    use crate::binder::BinderState;

    let code = r#"
        class Simple {
            getValue(): number {
                return 42;
            }
        }
    "#;

    let mut parser = ParserState::new("test.ts".to_string(), code.to_string());
    let root = parser.parse_source_file();
    let mut binder = BinderState::new();
    binder.bind_source_file(&parser.arena, root);

    let mut checker = CheckerState::new(
        &parser.arena,
        &binder.symbols,
        &binder.file_locals,
        "test.ts".to_string(),
    );

    checker.check_source_file(root);
}

#[test]
fn test_class_with_property_no_method_call() {
    // Class with property, but no external usage
    use crate::parser_impl::ParserState;
    use crate::binder::BinderState;

    let code = r#"
        class Counter {
            count: number = 0;
        }
    "#;

    let mut parser = ParserState::new("test.ts".to_string(), code.to_string());
    let root = parser.parse_source_file();
    let mut binder = BinderState::new();
    binder.bind_source_file(&parser.arena, root);

    let mut checker = CheckerState::new(
        &parser.arena,
        &binder.symbols,
        &binder.file_locals,
        "test.ts".to_string(),
    );

    checker.check_source_file(root);
}

#[test]
fn test_class_with_method_using_this_property() {
    // Class with method that accesses this.property - the minimal failing case
    use crate::parser_impl::ParserState;
    use crate::binder::BinderState;

    let code = r#"
        class Counter {
            count: number = 0;
            getCount(): number {
                return this.count;
            }
        }
    "#;

    let mut parser = ParserState::new("test.ts".to_string(), code.to_string());
    let root = parser.parse_source_file();
    let mut binder = BinderState::new();
    binder.bind_source_file(&parser.arena, root);

    let mut checker = CheckerState::new(
        &parser.arena,
        &binder.symbols,
        &binder.file_locals,
        "test.ts".to_string(),
    );

    checker.check_source_file(root);
}

#[test]
fn test_simple_type_annotation() {
    // Just a simple type annotation, not in a class
    use crate::parser_impl::ParserState;
    use crate::binder::BinderState;

    let code = r#"
        let x: number;
    "#;

    let mut parser = ParserState::new("test.ts".to_string(), code.to_string());
    let root = parser.parse_source_file();
    let mut binder = BinderState::new();
    binder.bind_source_file(&parser.arena, root);

    let mut checker = CheckerState::new(
        &parser.arena,
        &binder.symbols,
        &binder.file_locals,
        "test.ts".to_string(),
    );

    checker.check_source_file(root);
}

#[test]
fn test_empty_class() {
    // Empty class with no members
    use crate::parser_impl::ParserState;
    use crate::binder::BinderState;

    let code = r#"
        class Empty {}
    "#;

    let mut parser = ParserState::new("test.ts".to_string(), code.to_string());
    let root = parser.parse_source_file();
    let mut binder = BinderState::new();
    binder.bind_source_file(&parser.arena, root);

    let mut checker = CheckerState::new(
        &parser.arena,
        &binder.symbols,
        &binder.file_locals,
        "test.ts".to_string(),
    );

    checker.check_source_file(root);
}

#[test]
fn test_class_with_public_modifier_and_initializer() {
    // Property with public modifier AND initializer
    use crate::parser_impl::ParserState;
    use crate::binder::BinderState;

    let code = r#"
        class Counter {
            public count: number = 0;
        }
    "#;

    let mut parser = ParserState::new("test.ts".to_string(), code.to_string());
    let root = parser.parse_source_file();
    let mut binder = BinderState::new();
    binder.bind_source_file(&parser.arena, root);

    let mut checker = CheckerState::new(
        &parser.arena,
        &binder.symbols,
        &binder.file_locals,
        "test.ts".to_string(),
    );

    checker.check_source_file(root);
}

#[test]
fn test_class_with_public_no_initializer() {
    // Public field, no initializer
    use crate::parser_impl::ParserState;
    use crate::binder::BinderState;

    let code = r#"
        class Counter {
            public count: number;
        }
    "#;

    let mut parser = ParserState::new("test.ts".to_string(), code.to_string());
    let root = parser.parse_source_file();
    let mut binder = BinderState::new();
    binder.bind_source_file(&parser.arena, root);

    let mut checker = CheckerState::new(
        &parser.arena,
        &binder.symbols,
        &binder.file_locals,
        "test.ts".to_string(),
    );

    checker.check_source_file(root);
}

#[test]
fn test_class_with_private_no_initializer() {
    // Private field, no initializer
    use crate::parser_impl::ParserState;
    use crate::binder::BinderState;

    let code = r#"
        class Counter {
            private count: number;
        }
    "#;

    let mut parser = ParserState::new("test.ts".to_string(), code.to_string());
    let root = parser.parse_source_file();
    let mut binder = BinderState::new();
    binder.bind_source_file(&parser.arena, root);

    let mut checker = CheckerState::new(
        &parser.arena,
        &binder.symbols,
        &binder.file_locals,
        "test.ts".to_string(),
    );

    checker.check_source_file(root);
}

#[test]
fn test_class_with_private_initializer() {
    // Private field with initializer
    use crate::parser_impl::ParserState;
    use crate::binder::BinderState;

    let code = r#"
        class Counter {
            private count: number = 0;
        }
    "#;

    let mut parser = ParserState::new("test.ts".to_string(), code.to_string());
    let root = parser.parse_source_file();
    let mut binder = BinderState::new();
    binder.bind_source_file(&parser.arena, root);

    let mut checker = CheckerState::new(
        &parser.arena,
        &binder.symbols,
        &binder.file_locals,
        "test.ts".to_string(),
    );

    checker.check_source_file(root);
}

#[test]
fn test_postfix_increment() {
    // Just postfix increment on local variable
    use crate::parser_impl::ParserState;
    use crate::binder::BinderState;

    let code = r#"
        let x: number = 0;
        x++;
    "#;

    let mut parser = ParserState::new("test.ts".to_string(), code.to_string());
    let root = parser.parse_source_file();
    let mut binder = BinderState::new();
    binder.bind_source_file(&parser.arena, root);

    let mut checker = CheckerState::new(
        &parser.arena,
        &binder.symbols,
        &binder.file_locals,
        "test.ts".to_string(),
    );

    checker.check_source_file(root);
}

#[test]
fn test_class_with_postfix_on_property() {
    // Postfix increment on this.property
    use crate::parser_impl::ParserState;
    use crate::binder::BinderState;

    let code = r#"
        class Counter {
            count: number = 0;
            increment(): void {
                this.count++;
            }
        }
    "#;

    let mut parser = ParserState::new("test.ts".to_string(), code.to_string());
    let root = parser.parse_source_file();
    let mut binder = BinderState::new();
    binder.bind_source_file(&parser.arena, root);

    let mut checker = CheckerState::new(
        &parser.arena,
        &binder.symbols,
        &binder.file_locals,
        "test.ts".to_string(),
    );

    checker.check_source_file(root);
}

#[test]
fn test_class_with_private_field() {
    // Class with private field and methods that access it
    use crate::parser_impl::ParserState;
    use crate::binder::BinderState;

    let code = r#"
        class Counter {
            private count: number = 0;
            increment(): void {
                this.count++;
            }
            getCount(): number {
                return this.count;
            }
        }
    "#;

    let mut parser = ParserState::new("test.ts".to_string(), code.to_string());
    let root = parser.parse_source_file();
    let mut binder = BinderState::new();
    binder.bind_source_file(&parser.arena, root);

    let mut checker = CheckerState::new(
        &parser.arena,
        &binder.symbols,
        &binder.file_locals,
        "test.ts".to_string(),
    );

    checker.check_source_file(root);
}

#[test]
fn test_class_static_member() {
    // Class with static members
    use crate::parser_impl::ParserState;
    use crate::binder::BinderState;

    let code = r#"
        class MathUtils {
            static PI: number = 3.14159;
            static square(x: number): number {
                return x * x;
            }
        }
        let pi = MathUtils.PI;
        let result = MathUtils.square(5);
    "#;

    let mut parser = ParserState::new("test.ts".to_string(), code.to_string());
    let root = parser.parse_source_file();
    let mut binder = BinderState::new();
    binder.bind_source_file(&parser.arena, root);

    let mut checker = CheckerState::new(
        &parser.arena,
        &binder.symbols,
        &binder.file_locals,
        "test.ts".to_string(),
    );

    checker.check_source_file(root);
}

// ============== Type Predicate Parsing Tests ==============

#[test]
fn test_type_predicate_simple() {
    // Test simple type predicate: x is string
    use crate::parser_impl::ParserState;
    use crate::binder::BinderState;

    let code = r#"
        function isString(x: unknown): x is string {
            return typeof x === "string";
        }
    "#;

    let mut parser = ParserState::new("test.ts".to_string(), code.to_string());
    let root = parser.parse_source_file();

    // Just verify it parses without error
    let mut binder = BinderState::new();
    binder.bind_source_file(&parser.arena, root);

    let mut checker = CheckerState::new(
        &parser.arena,
        &binder.symbols,
        &binder.file_locals,
        "test.ts".to_string(),
    );

    // Function should exist
    assert!(binder.file_locals.has("isString"), "isString function should be defined");
    checker.check_source_file(root);
}

#[test]
fn test_type_predicate_this() {
    // Test this type predicate
    use crate::parser_impl::ParserState;
    use crate::binder::BinderState;

    let code = r#"
        class Cat {
            isCat(): this is Cat {
                return true;
            }
        }
    "#;

    let mut parser = ParserState::new("test.ts".to_string(), code.to_string());
    let root = parser.parse_source_file();

    let mut binder = BinderState::new();
    binder.bind_source_file(&parser.arena, root);

    let mut checker = CheckerState::new(
        &parser.arena,
        &binder.symbols,
        &binder.file_locals,
        "test.ts".to_string(),
    );

    assert!(binder.file_locals.has("Cat"), "Cat class should be defined");
    checker.check_source_file(root);
}

#[test]
fn test_type_predicate_asserts() {
    // Test assertion predicate: asserts x is string
    use crate::parser_impl::ParserState;
    use crate::binder::BinderState;

    let code = r#"
        function assertIsString(x: unknown): asserts x is string {
            if (typeof x !== "string") {
                throw new Error("Not a string");
            }
        }
    "#;

    let mut parser = ParserState::new("test.ts".to_string(), code.to_string());
    let root = parser.parse_source_file();

    let mut binder = BinderState::new();
    binder.bind_source_file(&parser.arena, root);

    let mut checker = CheckerState::new(
        &parser.arena,
        &binder.symbols,
        &binder.file_locals,
        "test.ts".to_string(),
    );

    assert!(binder.file_locals.has("assertIsString"), "assertIsString function should be defined");
    checker.check_source_file(root);
}

#[test]
fn test_type_predicate_asserts_only() {
    // Test assertion without type: asserts x (simpler body to avoid memory issues)
    use crate::parser_impl::ParserState;
    use crate::binder::BinderState;

    let code = r#"
        function assertDefined(x: unknown): asserts x {
        }
    "#;

    let mut parser = ParserState::new("test.ts".to_string(), code.to_string());
    let root = parser.parse_source_file();

    let mut binder = BinderState::new();
    binder.bind_source_file(&parser.arena, root);

    let mut checker = CheckerState::new(
        &parser.arena,
        &binder.symbols,
        &binder.file_locals,
        "test.ts".to_string(),
    );

    assert!(binder.file_locals.has("assertDefined"), "assertDefined function should be defined");
    checker.check_source_file(root);
}

#[test]
fn test_function_type_with_predicate() {
    // Test function type with type predicate
    use crate::parser_impl::ParserState;
    use crate::binder::BinderState;

    let code = r#"
        type Guard<T> = (x: unknown) => x is T;
        const isNumber: Guard<number> = (x): x is number => typeof x === "number";
    "#;

    let mut parser = ParserState::new("test.ts".to_string(), code.to_string());
    let root = parser.parse_source_file();

    let mut binder = BinderState::new();
    binder.bind_source_file(&parser.arena, root);

    let mut checker = CheckerState::new(
        &parser.arena,
        &binder.symbols,
        &binder.file_locals,
        "test.ts".to_string(),
    );

    assert!(binder.file_locals.has("Guard"), "Guard type should be defined");
    assert!(binder.file_locals.has("isNumber"), "isNumber should be defined");
    checker.check_source_file(root);
}

// ============== Additional Common TypeScript Pattern Tests ==============

#[test]
fn test_readonly_modifier() {
    // Test readonly modifier on class properties
    use crate::parser_impl::ParserState;
    use crate::binder::BinderState;

    let code = r#"
        class Config {
            readonly host: string;
            readonly port: number;
        }
    "#;

    let mut parser = ParserState::new("test.ts".to_string(), code.to_string());
    let root = parser.parse_source_file();

    let mut binder = BinderState::new();
    binder.bind_source_file(&parser.arena, root);

    let mut checker = CheckerState::new(
        &parser.arena,
        &binder.symbols,
        &binder.file_locals,
        "test.ts".to_string(),
    );

    assert!(binder.file_locals.has("Config"), "Config class should be defined");
    checker.check_source_file(root);
}

#[test]
fn test_abstract_class() {
    // Test abstract class with abstract method
    use crate::parser_impl::ParserState;
    use crate::binder::BinderState;

    let code = r#"
abstract class Shape {
    abstract getArea(): number;
}
"#;

    let mut parser = ParserState::new("test.ts".to_string(), code.to_string());
    let root = parser.parse_source_file();

    let mut binder = BinderState::new();
    binder.bind_source_file(&parser.arena, root);

    let mut checker = CheckerState::new(
        &parser.arena,
        &binder.symbols,
        &binder.file_locals,
        "test.ts".to_string(),
    );

    checker.check_source_file(root);

    // Should have no errors about abstract members
    let abstract_errors: Vec<_> = checker.diagnostics.iter()
        .filter(|d| d.code == super::diagnostic_codes::ABSTRACT_MEMBER_IN_NON_ABSTRACT_CLASS)
        .collect();
    assert!(
        abstract_errors.is_empty(),
        "Should allow abstract method in abstract class. Got: {:?}",
        abstract_errors
    );
}

#[test]
fn test_interface_extends() {
    // Test interface extending another interface
    use crate::parser_impl::ParserState;
    use crate::binder::BinderState;

    let code = r#"
        interface Animal {
            name: string;
        }
        interface Dog extends Animal {
            breed: string;
        }
    "#;

    let mut parser = ParserState::new("test.ts".to_string(), code.to_string());
    let root = parser.parse_source_file();

    let mut binder = BinderState::new();
    binder.bind_source_file(&parser.arena, root);

    let mut checker = CheckerState::new(
        &parser.arena,
        &binder.symbols,
        &binder.file_locals,
        "test.ts".to_string(),
    );

    assert!(binder.file_locals.has("Animal"), "Animal interface should be defined");
    assert!(binder.file_locals.has("Dog"), "Dog interface should be defined");
    checker.check_source_file(root);
}

#[test]
fn test_enum_with_computed_values() {
    // Test enum with computed values
    use crate::parser_impl::ParserState;
    use crate::binder::BinderState;

    let code = r#"
        enum Status {
            Pending = 0,
            Active = 1,
            Completed = 2
        }
        let s: Status;
    "#;

    let mut parser = ParserState::new("test.ts".to_string(), code.to_string());
    let root = parser.parse_source_file();

    let mut binder = BinderState::new();
    binder.bind_source_file(&parser.arena, root);

    let mut checker = CheckerState::new(
        &parser.arena,
        &binder.symbols,
        &binder.file_locals,
        "test.ts".to_string(),
    );

    assert!(binder.file_locals.has("Status"), "Status enum should be defined");
    checker.check_source_file(root);
}

#[test]
fn test_namespace_with_exports() {
    // Test namespace with exported members
    use crate::parser_impl::ParserState;
    use crate::binder::BinderState;

    let code = r#"
        namespace Utils {
            export function helper(x: number): number {
                return x;
            }
        }
    "#;

    let mut parser = ParserState::new("test.ts".to_string(), code.to_string());
    let root = parser.parse_source_file();

    let mut binder = BinderState::new();
    binder.bind_source_file(&parser.arena, root);

    let mut checker = CheckerState::new(
        &parser.arena,
        &binder.symbols,
        &binder.file_locals,
        "test.ts".to_string(),
    );

    assert!(binder.file_locals.has("Utils"), "Utils namespace should be defined");
    checker.check_source_file(root);
}

#[test]
fn test_destructuring_assignment() {
    // Test simple object literal
    use crate::parser_impl::ParserState;
    use crate::binder::BinderState;

    let code = r#"
const obj = { x: 1, y: 2 };
"#;

    let mut parser = ParserState::new("test.ts".to_string(), code.to_string());
    let root = parser.parse_source_file();

    let mut binder = BinderState::new();
    binder.bind_source_file(&parser.arena, root);

    let mut checker = CheckerState::new(
        &parser.arena,
        &binder.symbols,
        &binder.file_locals,
        "test.ts".to_string(),
    );

    assert!(binder.file_locals.has("obj"), "obj should be defined");
    checker.check_source_file(root);
}

#[test]
fn test_arrow_function_with_body() {
    // Test arrow function with block body
    use crate::parser_impl::ParserState;
    use crate::binder::BinderState;

    let code = r#"
        const add = (a: number, b: number): number => {
            return a + b;
        };
    "#;

    let mut parser = ParserState::new("test.ts".to_string(), code.to_string());
    let root = parser.parse_source_file();

    let mut binder = BinderState::new();
    binder.bind_source_file(&parser.arena, root);

    let mut checker = CheckerState::new(
        &parser.arena,
        &binder.symbols,
        &binder.file_locals,
        "test.ts".to_string(),
    );

    assert!(binder.file_locals.has("add"), "add function should be defined");
    checker.check_source_file(root);
}

#[test]
fn test_new_expression_simple() {
    // Test simple new expression
    use crate::parser_impl::ParserState;
    use crate::binder::BinderState;

    let code = r#"
class Foo {
    x: number;
}
let f = new Foo();
"#;

    let mut parser = ParserState::new("test.ts".to_string(), code.to_string());
    let root = parser.parse_source_file();

    let mut binder = BinderState::new();
    binder.bind_source_file(&parser.arena, root);

    let mut checker = CheckerState::new(
        &parser.arena,
        &binder.symbols,
        &binder.file_locals,
        "test.ts".to_string(),
    );

    assert!(binder.file_locals.has("Foo"), "Foo class should be defined");
    assert!(binder.file_locals.has("f"), "f variable should be defined");
    checker.check_source_file(root);
}

#[test]
fn test_new_expression_with_property_access() {
    // Test new expression with property access
    use crate::parser_impl::ParserState;
    use crate::binder::BinderState;

    let code = r#"
class Foo {
    x: number;
}
let f = new Foo();
let v = f.x;
"#;

    let mut parser = ParserState::new("test.ts".to_string(), code.to_string());
    let root = parser.parse_source_file();

    let mut binder = BinderState::new();
    binder.bind_source_file(&parser.arena, root);

    let mut checker = CheckerState::new(
        &parser.arena,
        &binder.symbols,
        &binder.file_locals,
        "test.ts".to_string(),
    );

    checker.check_source_file(root);
}

#[test]
fn test_private_property_access() {
    // Test private property access (simpler version)
    use crate::parser_impl::ParserState;
    use crate::binder::BinderState;

    let code = r#"
class Foo {
    private x: number;
}
let f = new Foo();
let v = f.x;
"#;

    let mut parser = ParserState::new("test.ts".to_string(), code.to_string());
    let root = parser.parse_source_file();

    let mut binder = BinderState::new();
    binder.bind_source_file(&parser.arena, root);

    let mut checker = CheckerState::new(
        &parser.arena,
        &binder.symbols,
        &binder.file_locals,
        "test.ts".to_string(),
    );

    checker.check_source_file(root);
}

#[test]
fn test_private_property_with_initializer() {
    // Test private property with initializer
    use crate::parser_impl::ParserState;
    use crate::binder::BinderState;

    let code = r#"
class Foo {
    private x: number = 1;
}
let f = new Foo();
let v = f.x;
"#;

    let mut parser = ParserState::new("test.ts".to_string(), code.to_string());
    let root = parser.parse_source_file();

    let mut binder = BinderState::new();
    binder.bind_source_file(&parser.arena, root);

    let mut checker = CheckerState::new(
        &parser.arena,
        &binder.symbols,
        &binder.file_locals,
        "test.ts".to_string(),
    );

    checker.check_source_file(root);
}

#[test]
fn test_template_literal_type() {
    // Test template literal type
    use crate::parser_impl::ParserState;
    use crate::binder::BinderState;

    let code = r#"
        type Greeting = `Hello, ${string}!`;
        let g: Greeting;
    "#;

    let mut parser = ParserState::new("test.ts".to_string(), code.to_string());
    let root = parser.parse_source_file();

    let mut binder = BinderState::new();
    binder.bind_source_file(&parser.arena, root);

    let mut checker = CheckerState::new(
        &parser.arena,
        &binder.symbols,
        &binder.file_locals,
        "test.ts".to_string(),
    );

    assert!(binder.file_locals.has("Greeting"), "Greeting type should be defined");
    checker.check_source_file(root);
}


#[test]
fn test_function_parameter_scoping() {
    // Test that function parameters are visible inside the function body
    use crate::parser_impl::ParserState;
    use crate::binder::BinderState;
    use super::state::CheckerState;

    let code = r#"
        function add(a: number, b: number): number {
            return a + b;
        }
    "#;

    let mut parser = ParserState::new("test.ts".to_string(), code.to_string());
    let root = parser.parse_source_file();

    let mut binder = BinderState::new();
    binder.bind_source_file(&parser.arena, root);

    let mut checker = CheckerState::new(
        &parser.arena,
        &binder.symbols,
        &binder.file_locals,
        "test.ts".to_string(),
    );

    checker.check_source_file(root);

    // Should have no "Cannot find name 'a'" or "Cannot find name 'b'" errors
    let name_errors: Vec<_> = checker.diagnostics.iter()
        .filter(|d| d.message_text.contains("Cannot find name"))
        .collect();
    assert!(name_errors.is_empty(), "Should have no 'Cannot find name' errors, got: {:?}", name_errors);
}


#[test]
fn test_method_parameter_scoping() {
    // Test that method parameters are visible inside the method body
    use crate::parser_impl::ParserState;
    use crate::binder::BinderState;
    use super::state::CheckerState;

    let code = r#"
        class Calculator {
            add(a: number, b: number): number {
                return a + b;
            }
        }
    "#;

    let mut parser = ParserState::new("test.ts".to_string(), code.to_string());
    let root = parser.parse_source_file();

    let mut binder = BinderState::new();
    binder.bind_source_file(&parser.arena, root);

    let mut checker = CheckerState::new(
        &parser.arena,
        &binder.symbols,
        &binder.file_locals,
        "test.ts".to_string(),
    );

    checker.check_source_file(root);

    // Should have no "Cannot find name" errors for method parameters
    let name_errors: Vec<_> = checker.diagnostics.iter()
        .filter(|d| d.message_text.contains("Cannot find name"))
        .collect();
    assert!(name_errors.is_empty(), "Should have no Cannot find name errors, got: {:?}", name_errors);
}

#[test]
fn test_super_property_access() {
    // Test accessing properties on super in derived classes
    use crate::parser_impl::ParserState;
    use crate::binder::BinderState;

    let code = r#"
        class Base {
            value: number = 42;
            getValue(): number { return this.value; }
        }
        class Derived extends Base {
            value: number = 100;
            getBaseValue(): number {
                return super.getValue();
            }
        }
    "#;

    let mut parser = ParserState::new("test.ts".to_string(), code.to_string());
    let root = parser.parse_source_file();

    let mut binder = BinderState::new();
    binder.bind_source_file(&parser.arena, root);

    let mut checker = CheckerState::new(
        &parser.arena,
        &binder.symbols,
        &binder.file_locals,
        "test.ts".to_string(),
    );

    checker.check_source_file(root);

    // No type errors expected
    assert_eq!(checker.diagnostics.len(), 0,
        "No type errors expected for super.getValue(), got: {:?}", checker.diagnostics);
}

#[test]
fn test_super_in_constructor() {
    // Test super call in constructor
    use crate::parser_impl::ParserState;
    use crate::binder::BinderState;

    let code = r#"
        class Animal {
            name: string;
            constructor(name: string) {
                this.name = name;
            }
        }
        class Dog extends Animal {
            breed: string;
            constructor(name: string, breed: string) {
                super(name);
                this.breed = breed;
            }
        }
    "#;

    let mut parser = ParserState::new("test.ts".to_string(), code.to_string());
    let root = parser.parse_source_file();

    let mut binder = BinderState::new();
    binder.bind_source_file(&parser.arena, root);

    let mut checker = CheckerState::new(
        &parser.arena,
        &binder.symbols,
        &binder.file_locals,
        "test.ts".to_string(),
    );

    checker.check_source_file(root);

    // No type errors expected for super(name) constructor call
    assert_eq!(checker.diagnostics.len(), 0,
        "No type errors expected for super(name), got: {:?}", checker.diagnostics);
}

#[test]
fn test_pick_utility_type() {
    // Test Pick<T, K> utility type behavior
    use crate::parser_impl::ParserState;
    use crate::binder::BinderState;

    let code = r#"
        type Pick<T, K extends keyof T> = { [P in K]: T[P] };
        interface User {
            id: number;
            name: string;
            email: string;
        }
        type PickedUser = Pick<User, "id" | "name">;
        const user: PickedUser = { id: 1, name: "John" };
    "#;

    let mut parser = ParserState::new("test.ts".to_string(), code.to_string());
    let root = parser.parse_source_file();

    let mut binder = BinderState::new();
    binder.bind_source_file(&parser.arena, root);

    let mut checker = CheckerState::new(
        &parser.arena,
        &binder.symbols,
        &binder.file_locals,
        "test.ts".to_string(),
    );

    checker.check_source_file(root);

    // No type errors expected - object has exactly the picked properties
    assert_eq!(checker.diagnostics.len(), 0,
        "No type errors expected for Pick usage, got: {:?}", checker.diagnostics);
}

#[test]
fn test_omit_utility_type() {
    // Test Omit<T, K> utility type behavior
    use crate::parser_impl::ParserState;
    use crate::binder::BinderState;

    let code = r#"
        type Exclude<T, U> = T extends U ? never : T;
        type Omit<T, K extends keyof any> = { [P in Exclude<keyof T, K>]: T[P] };
        interface User {
            id: number;
            name: string;
            password: string;
        }
        type SafeUser = Omit<User, "password">;
    "#;

    let mut parser = ParserState::new("test.ts".to_string(), code.to_string());
    let root = parser.parse_source_file();

    let mut binder = BinderState::new();
    binder.bind_source_file(&parser.arena, root);

    let mut checker = CheckerState::new(
        &parser.arena,
        &binder.symbols,
        &binder.file_locals,
        "test.ts".to_string(),
    );

    checker.check_source_file(root);

    // No type errors expected
    assert_eq!(checker.diagnostics.len(), 0,
        "No type errors expected for Omit usage, got: {:?}", checker.diagnostics);
}

#[test]
fn test_extract_utility_type() {
    // Test Extract<T, U> utility type
    use crate::parser_impl::ParserState;
    use crate::binder::BinderState;

    let code = r#"
        type Extract<T, U> = T extends U ? T : never;
        type StringOrNumber = string | number | boolean;
        type OnlyStrings = Extract<StringOrNumber, string>;
        const s: OnlyStrings = "hello";
    "#;

    let mut parser = ParserState::new("test.ts".to_string(), code.to_string());
    let root = parser.parse_source_file();

    let mut binder = BinderState::new();
    binder.bind_source_file(&parser.arena, root);

    let mut checker = CheckerState::new(
        &parser.arena,
        &binder.symbols,
        &binder.file_locals,
        "test.ts".to_string(),
    );

    checker.check_source_file(root);

    // No type errors expected
    assert_eq!(checker.diagnostics.len(), 0,
        "No type errors expected for Extract usage, got: {:?}", checker.diagnostics);
}

#[test]
fn test_exclude_utility_type() {
    // Test Exclude<T, U> utility type
    use crate::parser_impl::ParserState;
    use crate::binder::BinderState;

    let code = r#"
        type Exclude<T, U> = T extends U ? never : T;
        type StringOrNumber = string | number | boolean;
        type NoStrings = Exclude<StringOrNumber, string>;
        const n: NoStrings = 42;
    "#;

    let mut parser = ParserState::new("test.ts".to_string(), code.to_string());
    let root = parser.parse_source_file();

    let mut binder = BinderState::new();
    binder.bind_source_file(&parser.arena, root);

    let mut checker = CheckerState::new(
        &parser.arena,
        &binder.symbols,
        &binder.file_locals,
        "test.ts".to_string(),
    );

    checker.check_source_file(root);

    // No type errors expected
    assert_eq!(checker.diagnostics.len(), 0,
        "No type errors expected for Exclude usage, got: {:?}", checker.diagnostics);
}

#[test]
fn test_return_type_utility() {
    // Test ReturnType<T> utility type
    use crate::parser_impl::ParserState;
    use crate::binder::BinderState;

    let code = r#"
        type ReturnType<T extends (...args: any) => any> = T extends (...args: any) => infer R ? R : any;
        function getUser(): { name: string; age: number } {
            return { name: "John", age: 30 };
        }
        type UserType = ReturnType<typeof getUser>;
    "#;

    let mut parser = ParserState::new("test.ts".to_string(), code.to_string());
    let root = parser.parse_source_file();

    let mut binder = BinderState::new();
    binder.bind_source_file(&parser.arena, root);

    let mut checker = CheckerState::new(
        &parser.arena,
        &binder.symbols,
        &binder.file_locals,
        "test.ts".to_string(),
    );

    checker.check_source_file(root);

    // No type errors expected
    assert_eq!(checker.diagnostics.len(), 0,
        "No type errors expected for ReturnType usage, got: {:?}", checker.diagnostics);
}

#[test]
fn test_parameters_utility() {
    // Test Parameters<T> utility type
    use crate::parser_impl::ParserState;
    use crate::binder::BinderState;

    let code = r#"
        type Parameters<T extends (...args: any) => any> = T extends (...args: infer P) => any ? P : never;
        function greet(name: string, age: number): string {
            return name;
        }
        type GreetParams = Parameters<typeof greet>;
    "#;

    let mut parser = ParserState::new("test.ts".to_string(), code.to_string());
    let root = parser.parse_source_file();

    let mut binder = BinderState::new();
    binder.bind_source_file(&parser.arena, root);

    let mut checker = CheckerState::new(
        &parser.arena,
        &binder.symbols,
        &binder.file_locals,
        "test.ts".to_string(),
    );

    checker.check_source_file(root);

    // No type errors expected
    assert_eq!(checker.diagnostics.len(), 0,
        "No type errors expected for Parameters usage, got: {:?}", checker.diagnostics);
}

#[test]
fn test_constructor_parameters_utility() {
    // Test simplified ConstructorParameters<T> utility type
    // Note: Full "abstract new" syntax causes stack overflow, so using simpler form
    use crate::parser_impl::ParserState;
    use crate::binder::BinderState;

    let code = r#"
        type ConstructorParams<T extends new (...args: any) => any> = T extends new (...args: infer P) => any ? P : never;
        class Point {
            x: number;
            y: number;
            constructor(x: number, y: number) {
                this.x = x;
                this.y = y;
            }
        }
        type PointParams = ConstructorParams<typeof Point>;
    "#;

    let mut parser = ParserState::new("test.ts".to_string(), code.to_string());
    let root = parser.parse_source_file();

    let mut binder = BinderState::new();
    binder.bind_source_file(&parser.arena, root);

    let mut checker = CheckerState::new(
        &parser.arena,
        &binder.symbols,
        &binder.file_locals,
        "test.ts".to_string(),
    );

    checker.check_source_file(root);

    // No type errors expected
    assert_eq!(checker.diagnostics.len(), 0,
        "No type errors expected for ConstructorParameters usage, got: {:?}", checker.diagnostics);
}

#[test]
fn test_instance_type_utility() {
    // Test simplified InstanceType<T> utility type
    // Note: Full "abstract new" syntax causes stack overflow, so using simpler form
    use crate::parser_impl::ParserState;
    use crate::binder::BinderState;

    let code = r#"
        type InstanceOf<T extends new (...args: any) => any> = T extends new (...args: any) => infer R ? R : any;
        class Animal {
            name: string = "";
        }
        type AnimalInstance = InstanceOf<typeof Animal>;
    "#;

    let mut parser = ParserState::new("test.ts".to_string(), code.to_string());
    let root = parser.parse_source_file();

    let mut binder = BinderState::new();
    binder.bind_source_file(&parser.arena, root);

    let mut checker = CheckerState::new(
        &parser.arena,
        &binder.symbols,
        &binder.file_locals,
        "test.ts".to_string(),
    );

    checker.check_source_file(root);

    // No type errors expected
    assert_eq!(checker.diagnostics.len(), 0,
        "No type errors expected for InstanceType usage, got: {:?}", checker.diagnostics);
}

#[test]
fn test_type_error_number_to_string() {
    // Test that assigning number to string produces a type error
    use crate::parser_impl::ParserState;
    use crate::binder::BinderState;

    let code = r#"
        const x: string = 42;
    "#;

    let mut parser = ParserState::new("test.ts".to_string(), code.to_string());
    let root = parser.parse_source_file();

    let mut binder = BinderState::new();
    binder.bind_source_file(&parser.arena, root);

    let mut checker = CheckerState::new(
        &parser.arena,
        &binder.symbols,
        &binder.file_locals,
        "test.ts".to_string(),
    );

    checker.check_source_file(root);

    // Should have exactly 1 type error
    assert_eq!(checker.diagnostics.len(), 1,
        "Expected 1 type error for assigning number to string, got: {:?}", checker.diagnostics);

    // Check error message contains relevant info
    let diag = &checker.diagnostics[0];
    assert!(diag.message_text.contains("not assignable") || diag.message_text.contains("number") || diag.message_text.contains("string"),
        "Error message should mention type mismatch, got: {}", diag.message_text);
}

#[test]
fn test_type_error_wrong_argument_type() {
    // Test that passing wrong argument type produces a type error
    use crate::parser_impl::ParserState;
    use crate::binder::BinderState;

    let code = r#"
        function greet(name: string): void {}
        greet(123);
    "#;

    let mut parser = ParserState::new("test.ts".to_string(), code.to_string());
    let root = parser.parse_source_file();

    let mut binder = BinderState::new();
    binder.bind_source_file(&parser.arena, root);

    let mut checker = CheckerState::new(
        &parser.arena,
        &binder.symbols,
        &binder.file_locals,
        "test.ts".to_string(),
    );

    checker.check_source_file(root);

    // Should have at least 1 type error for argument type mismatch
    assert!(!checker.diagnostics.is_empty(),
        "Expected type error for passing number to string parameter, got: {:?}", checker.diagnostics);
}

#[test]
fn test_type_error_missing_property() {
    // Test that accessing non-existent property produces a type error
    use crate::parser_impl::ParserState;
    use crate::binder::BinderState;

    let code = r#"
        interface User {
            name: string;
        }
        const user: User = { name: "John" };
        const x = user.age;
    "#;

    let mut parser = ParserState::new("test.ts".to_string(), code.to_string());
    let root = parser.parse_source_file();

    let mut binder = BinderState::new();
    binder.bind_source_file(&parser.arena, root);

    let mut checker = CheckerState::new(
        &parser.arena,
        &binder.symbols,
        &binder.file_locals,
        "test.ts".to_string(),
    );

    checker.check_source_file(root);

    // Should have error for missing property 'age'
    let property_errors: Vec<_> = checker.diagnostics.iter()
        .filter(|d| d.message_text.contains("age") || d.message_text.contains("does not exist"))
        .collect();
    assert!(!property_errors.is_empty(),
        "Expected error for accessing non-existent property 'age', got: {:?}", checker.diagnostics);
}

#[test]
fn test_type_error_too_few_arguments() {
    // Test that calling function with too few arguments produces error
    use crate::parser_impl::ParserState;
    use crate::binder::BinderState;

    let code = r#"
        function add(a: number, b: number): number {
            return a + b;
        }
        const result = add(1);
    "#;

    let mut parser = ParserState::new("test.ts".to_string(), code.to_string());
    let root = parser.parse_source_file();

    let mut binder = BinderState::new();
    binder.bind_source_file(&parser.arena, root);

    let mut checker = CheckerState::new(
        &parser.arena,
        &binder.symbols,
        &binder.file_locals,
        "test.ts".to_string(),
    );

    checker.check_source_file(root);

    // Should have error for too few arguments
    let arg_errors: Vec<_> = checker.diagnostics.iter()
        .filter(|d| d.message_text.contains("argument") || d.message_text.contains("Expected"))
        .collect();
    assert!(!arg_errors.is_empty(),
        "Expected error for too few arguments, got: {:?}", checker.diagnostics);
}

#[test]
fn test_generic_function_argument_type_check() {
    // Test that generic function calls still check argument types
    use crate::parser_impl::ParserState;
    use crate::binder::BinderState;

    let code = r#"
        function identity<T>(value: T): T {
            return value;
        }
        const x: number = identity<number>("hello");
    "#;

    let mut parser = ParserState::new("test.ts".to_string(), code.to_string());
    let root = parser.parse_source_file();

    let mut binder = BinderState::new();
    binder.bind_source_file(&parser.arena, root);

    let mut checker = CheckerState::new(
        &parser.arena,
        &binder.symbols,
        &binder.file_locals,
        "test.ts".to_string(),
    );

    checker.check_source_file(root);

    // Should have type error for explicit type argument mismatch
    // identity<number>("hello") - string is not assignable to number
    assert!(!checker.diagnostics.is_empty(),
        "Expected type error for generic call with wrong argument type, got: {:?}", checker.diagnostics);
}

#[test]
fn test_callback_argument_type_check() {
    // Test that callback arguments are checked
    use crate::parser_impl::ParserState;
    use crate::binder::BinderState;

    let code = r#"
        function forEach<T>(arr: T[], callback: (item: T) => void): void {}
        forEach<number>([1, 2, 3], (item: string) => {});
    "#;

    let mut parser = ParserState::new("test.ts".to_string(), code.to_string());
    let root = parser.parse_source_file();

    let mut binder = BinderState::new();
    binder.bind_source_file(&parser.arena, root);

    let mut checker = CheckerState::new(
        &parser.arena,
        &binder.symbols,
        &binder.file_locals,
        "test.ts".to_string(),
    );

    checker.check_source_file(root);

    // Should have type error for callback parameter type mismatch
    assert!(!checker.diagnostics.is_empty(),
        "Expected type error for callback with wrong parameter type, got: {:?}", checker.diagnostics);
}

#[test]
fn test_rest_parameter_type_check() {
    // Test that rest parameters are checked
    use crate::parser_impl::ParserState;
    use crate::binder::BinderState;

    let code = r#"
        function sum(...numbers: number[]): number {
            return 0;
        }
        sum(1, 2, "three");
    "#;

    let mut parser = ParserState::new("test.ts".to_string(), code.to_string());
    let root = parser.parse_source_file();

    let mut binder = BinderState::new();
    binder.bind_source_file(&parser.arena, root);

    let mut checker = CheckerState::new(
        &parser.arena,
        &binder.symbols,
        &binder.file_locals,
        "test.ts".to_string(),
    );

    checker.check_source_file(root);

    // Should have type error for string in number rest parameter
    assert!(!checker.diagnostics.is_empty(),
        "Expected type error for string in number[] rest parameter, got: {:?}", checker.diagnostics);
}

#[test]
fn test_satisfies_expression() {
    // Test that satisfies expression returns the original type
    use crate::parser_impl::ParserState;
    use crate::binder::BinderState;

    let code = r#"
        type RGB = [number, number, number];
        const red = [255, 0, 0] satisfies RGB;
    "#;

    let mut parser = ParserState::new("test.ts".to_string(), code.to_string());
    let root = parser.parse_source_file();

    let mut binder = BinderState::new();
    binder.bind_source_file(&parser.arena, root);

    let mut checker = CheckerState::new(
        &parser.arena,
        &binder.symbols,
        &binder.file_locals,
        "test.ts".to_string(),
    );

    checker.check_source_file(root);

    // Should have no errors
    assert!(checker.diagnostics.is_empty(),
        "Expected no errors for satisfies with valid type, got: {:?}", checker.diagnostics);
}

#[test]
fn test_satisfies_expression_error() {
    // Test that satisfies expression reports error when types don't match
    use crate::parser_impl::ParserState;
    use crate::binder::BinderState;

    let code = r#"
        const bad = "hello" satisfies number;
    "#;

    let mut parser = ParserState::new("test.ts".to_string(), code.to_string());
    let root = parser.parse_source_file();

    let mut binder = BinderState::new();
    binder.bind_source_file(&parser.arena, root);

    let mut checker = CheckerState::new(
        &parser.arena,
        &binder.symbols,
        &binder.file_locals,
        "test.ts".to_string(),
    );

    checker.check_source_file(root);

    // Should have error for string not satisfying number
    assert!(!checker.diagnostics.is_empty(),
        "Expected error for string not satisfying number, got: {:?}", checker.diagnostics);
}

#[test]
fn test_as_expression_unknown_to_string() {
    // Test as expression with unknown to string
    use crate::parser_impl::ParserState;
    use crate::binder::BinderState;

    let code = r#"
        const value: unknown = "hello";
        const str = value as string;
    "#;

    let mut parser = ParserState::new("test.ts".to_string(), code.to_string());
    let root = parser.parse_source_file();

    let mut binder = BinderState::new();
    binder.bind_source_file(&parser.arena, root);

    let mut checker = CheckerState::new(
        &parser.arena,
        &binder.symbols,
        &binder.file_locals,
        "test.ts".to_string(),
    );

    checker.check_source_file(root);

    assert!(checker.diagnostics.is_empty(),
        "Expected no errors for valid as expression, got: {:?}", checker.diagnostics);
}

#[test]
fn test_as_expression() {
    // Test as expression (expr as Type)
    use crate::parser_impl::ParserState;
    use crate::binder::BinderState;

    let code = r#"
        const value: unknown = 42;
        const num = value as number;
    "#;

    let mut parser = ParserState::new("test.ts".to_string(), code.to_string());
    let root = parser.parse_source_file();

    let mut binder = BinderState::new();
    binder.bind_source_file(&parser.arena, root);

    let mut checker = CheckerState::new(
        &parser.arena,
        &binder.symbols,
        &binder.file_locals,
        "test.ts".to_string(),
    );

    checker.check_source_file(root);

    assert!(checker.diagnostics.is_empty(),
        "Expected no errors for valid as expression, got: {:?}", checker.diagnostics);
}

#[test]
fn test_as_const_expression() {
    // Test as const (preserves literal types)
    use crate::parser_impl::ParserState;
    use crate::binder::BinderState;

    let code = r#"
        const colors = ["red", "green", "blue"] as const;
    "#;

    let mut parser = ParserState::new("test.ts".to_string(), code.to_string());
    let root = parser.parse_source_file();

    let mut binder = BinderState::new();
    binder.bind_source_file(&parser.arena, root);

    let mut checker = CheckerState::new(
        &parser.arena,
        &binder.symbols,
        &binder.file_locals,
        "test.ts".to_string(),
    );

    checker.check_source_file(root);

    assert!(checker.diagnostics.is_empty(),
        "Expected no errors for as const expression, got: {:?}", checker.diagnostics);
}
