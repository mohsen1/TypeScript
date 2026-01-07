//! Tests for type operations.

use super::*;
use crate::solver::intern::TypeInterner;
use crate::solver::subtype::SubtypeChecker;

#[test]
fn test_call_simple_function() {
    let interner = TypeInterner::new();
    let mut subtype = SubtypeChecker::new(&interner);
    let mut evaluator = CallEvaluator::new(&interner, &mut subtype);

    // function(x: number): string
    let func = interner.function(FunctionShape {
        params: vec![ParamInfo {
            name: Some(interner.intern_string("x")),
            type_id: TypeId::NUMBER,
            optional: false,
            rest: false,
        }],
        return_type: TypeId::STRING,
        type_params: Vec::new(),
        type_predicate: None,
        is_constructor: false,
    });

    // Call with correct args
    let result = evaluator.resolve_call(func, &[TypeId::NUMBER]);
    match result {
        CallResult::Success(ret) => assert_eq!(ret, TypeId::STRING),
        _ => panic!("Expected success, got {:?}", result),
    }
}

#[test]
fn test_call_argument_count_mismatch() {
    let interner = TypeInterner::new();
    let mut subtype = SubtypeChecker::new(&interner);
    let mut evaluator = CallEvaluator::new(&interner, &mut subtype);

    // function(x: number): string
    let func = interner.function(FunctionShape {
        params: vec![ParamInfo {
            name: Some(interner.intern_string("x")),
            type_id: TypeId::NUMBER,
            optional: false,
            rest: false,
        }],
        return_type: TypeId::STRING,
        type_params: Vec::new(),
        type_predicate: None,
        is_constructor: false,
    });

    // Call with no args
    let result = evaluator.resolve_call(func, &[]);
    match result {
        CallResult::ArgumentCountMismatch { expected_min, actual, .. } => {
            assert_eq!(expected_min, 1);
            assert_eq!(actual, 0);
        }
        _ => panic!("Expected ArgumentCountMismatch, got {:?}", result),
    }
}

#[test]
fn test_call_argument_type_mismatch() {
    let interner = TypeInterner::new();
    let mut subtype = SubtypeChecker::new(&interner);
    let mut evaluator = CallEvaluator::new(&interner, &mut subtype);

    // function(x: number): string
    let func = interner.function(FunctionShape {
        params: vec![ParamInfo {
            name: Some(interner.intern_string("x")),
            type_id: TypeId::NUMBER,
            optional: false,
            rest: false,
        }],
        return_type: TypeId::STRING,
        type_params: Vec::new(),
        type_predicate: None,
        is_constructor: false,
    });

    // Call with wrong type
    let result = evaluator.resolve_call(func, &[TypeId::STRING]);
    match result {
        CallResult::ArgumentTypeMismatch { index, expected, actual } => {
            assert_eq!(index, 0);
            assert_eq!(expected, TypeId::NUMBER);
            assert_eq!(actual, TypeId::STRING);
        }
        _ => panic!("Expected ArgumentTypeMismatch, got {:?}", result),
    }
}

#[test]
fn test_call_rest_parameter_allows_zero_args() {
    let interner = TypeInterner::new();
    let mut subtype = SubtypeChecker::new(&interner);
    let mut evaluator = CallEvaluator::new(&interner, &mut subtype);

    // function(...args: number[]): string
    let rest_array = interner.array(TypeId::NUMBER);
    let func = interner.function(FunctionShape {
        params: vec![ParamInfo {
            name: Some(interner.intern_string("args")),
            type_id: rest_array,
            optional: false,
            rest: true,
        }],
        return_type: TypeId::STRING,
        type_params: Vec::new(),
        type_predicate: None,
        is_constructor: false,
    });

    let result = evaluator.resolve_call(func, &[]);
    match result {
        CallResult::Success(ret) => assert_eq!(ret, TypeId::STRING),
        _ => panic!("Expected success, got {:?}", result),
    }
}

#[test]
fn test_call_rest_parameter_min_args_with_required() {
    let interner = TypeInterner::new();
    let mut subtype = SubtypeChecker::new(&interner);
    let mut evaluator = CallEvaluator::new(&interner, &mut subtype);

    // function(x: string, ...args: number[]): string
    let rest_array = interner.array(TypeId::NUMBER);
    let func = interner.function(FunctionShape {
        params: vec![
            ParamInfo {
                name: Some(interner.intern_string("x")),
                type_id: TypeId::STRING,
                optional: false,
                rest: false,
            },
            ParamInfo {
                name: Some(interner.intern_string("args")),
                type_id: rest_array,
                optional: false,
                rest: true,
            },
        ],
        return_type: TypeId::STRING,
        type_params: Vec::new(),
        type_predicate: None,
        is_constructor: false,
    });

    let result = evaluator.resolve_call(func, &[]);
    match result {
        CallResult::ArgumentCountMismatch { expected_min, actual, .. } => {
            assert_eq!(expected_min, 1);
            assert_eq!(actual, 0);
        }
        _ => panic!("Expected ArgumentCountMismatch, got {:?}", result),
    }
}

#[test]
fn test_call_rest_parameter_type_match() {
    let interner = TypeInterner::new();
    let mut subtype = SubtypeChecker::new(&interner);
    let mut evaluator = CallEvaluator::new(&interner, &mut subtype);

    // function(...args: number[]): string
    let rest_array = interner.array(TypeId::NUMBER);
    let func = interner.function(FunctionShape {
        params: vec![ParamInfo {
            name: Some(interner.intern_string("args")),
            type_id: rest_array,
            optional: false,
            rest: true,
        }],
        return_type: TypeId::STRING,
        type_params: Vec::new(),
        type_predicate: None,
        is_constructor: false,
    });

    let result = evaluator.resolve_call(func, &[TypeId::NUMBER, TypeId::NUMBER]);
    match result {
        CallResult::Success(ret) => assert_eq!(ret, TypeId::STRING),
        _ => panic!("Expected success, got {:?}", result),
    }
}

#[test]
fn test_call_rest_parameter_type_mismatch() {
    let interner = TypeInterner::new();
    let mut subtype = SubtypeChecker::new(&interner);
    let mut evaluator = CallEvaluator::new(&interner, &mut subtype);

    // function(...args: number[]): string
    let rest_array = interner.array(TypeId::NUMBER);
    let func = interner.function(FunctionShape {
        params: vec![ParamInfo {
            name: Some(interner.intern_string("args")),
            type_id: rest_array,
            optional: false,
            rest: true,
        }],
        return_type: TypeId::STRING,
        type_params: Vec::new(),
        type_predicate: None,
        is_constructor: false,
    });

    let result = evaluator.resolve_call(func, &[TypeId::NUMBER, TypeId::STRING]);
    match result {
        CallResult::ArgumentTypeMismatch { index, expected, actual } => {
            assert_eq!(index, 1);
            assert_eq!(expected, TypeId::NUMBER);
            assert_eq!(actual, TypeId::STRING);
        }
        _ => panic!("Expected ArgumentTypeMismatch, got {:?}", result),
    }
}

#[test]
fn test_call_tuple_rest_argument_count_mismatch() {
    let interner = TypeInterner::new();
    let mut subtype = SubtypeChecker::new(&interner);
    let mut evaluator = CallEvaluator::new(&interner, &mut subtype);

    let tuple_rest = interner.tuple(vec![
        TupleElement { type_id: TypeId::NUMBER, name: None, optional: false, rest: false },
        TupleElement { type_id: TypeId::STRING, name: None, optional: false, rest: false },
    ]);

    let func = interner.function(FunctionShape {
        params: vec![ParamInfo {
            name: Some(interner.intern_string("args")),
            type_id: tuple_rest,
            optional: false,
            rest: true,
        }],
        return_type: TypeId::STRING,
        type_params: Vec::new(),
        type_predicate: None,
        is_constructor: false,
    });

    let result = evaluator.resolve_call(func, &[TypeId::NUMBER]);
    match result {
        CallResult::ArgumentCountMismatch { expected_min, actual, .. } => {
            assert_eq!(expected_min, 2);
            assert_eq!(actual, 1);
        }
        _ => panic!("Expected ArgumentCountMismatch, got {:?}", result),
    }
}

#[test]
fn test_call_tuple_rest_argument_type_mismatch() {
    let interner = TypeInterner::new();
    let mut subtype = SubtypeChecker::new(&interner);
    let mut evaluator = CallEvaluator::new(&interner, &mut subtype);

    let tuple_rest = interner.tuple(vec![
        TupleElement { type_id: TypeId::NUMBER, name: None, optional: false, rest: false },
        TupleElement { type_id: TypeId::STRING, name: None, optional: false, rest: false },
    ]);

    let func = interner.function(FunctionShape {
        params: vec![ParamInfo {
            name: Some(interner.intern_string("args")),
            type_id: tuple_rest,
            optional: false,
            rest: true,
        }],
        return_type: TypeId::STRING,
        type_params: Vec::new(),
        type_predicate: None,
        is_constructor: false,
    });

    let result = evaluator.resolve_call(func, &[TypeId::NUMBER, TypeId::BOOLEAN]);
    match result {
        CallResult::ArgumentTypeMismatch { index, expected, actual } => {
            assert_eq!(index, 1);
            assert_eq!(expected, TypeId::STRING);
            assert_eq!(actual, TypeId::BOOLEAN);
        }
        _ => panic!("Expected ArgumentTypeMismatch, got {:?}", result),
    }
}

#[test]
fn test_call_tuple_rest_argument_success() {
    let interner = TypeInterner::new();
    let mut subtype = SubtypeChecker::new(&interner);
    let mut evaluator = CallEvaluator::new(&interner, &mut subtype);

    let tuple_rest = interner.tuple(vec![
        TupleElement { type_id: TypeId::NUMBER, name: None, optional: false, rest: false },
        TupleElement { type_id: TypeId::STRING, name: None, optional: false, rest: false },
    ]);

    let func = interner.function(FunctionShape {
        params: vec![ParamInfo {
            name: Some(interner.intern_string("args")),
            type_id: tuple_rest,
            optional: false,
            rest: true,
        }],
        return_type: TypeId::STRING,
        type_params: Vec::new(),
        type_predicate: None,
        is_constructor: false,
    });

    let result = evaluator.resolve_call(func, &[TypeId::NUMBER, TypeId::STRING]);
    match result {
        CallResult::Success(ret) => assert_eq!(ret, TypeId::STRING),
        _ => panic!("Expected success, got {:?}", result),
    }
}

#[test]
fn test_property_access_object() {
    let interner = TypeInterner::new();
    let evaluator = PropertyAccessEvaluator::new(&interner);

    // { x: number, y: string }
    let obj = interner.object(vec![
        PropertyInfo {
            name: interner.intern_string("x"),
            type_id: TypeId::NUMBER,
            optional: false,
            readonly: false,
            is_method: false,
        },
        PropertyInfo {
            name: interner.intern_string("y"),
            type_id: TypeId::STRING,
            optional: false,
            readonly: false,
            is_method: false,
        },
    ]);

    // Access existing property
    let result = evaluator.resolve_property_access(obj, "x");
    match result {
        PropertyAccessResult::Success { type_id: t, .. } => assert_eq!(t, TypeId::NUMBER),
        _ => panic!("Expected success, got {:?}", result),
    }

    // Access non-existent property
    let result = evaluator.resolve_property_access(obj, "z");
    match result {
        PropertyAccessResult::PropertyNotFound { .. } => {}
        _ => panic!("Expected PropertyNotFound, got {:?}", result),
    }
}

#[test]
fn test_property_access_void() {
    let interner = TypeInterner::new();
    let evaluator = PropertyAccessEvaluator::new(&interner);

    let result = evaluator.resolve_property_access(TypeId::VOID, "x");
    match result {
        PropertyAccessResult::PossiblyNullOrUndefined { property_type, cause } => {
            assert!(property_type.is_none());
            assert_eq!(cause, TypeId::UNDEFINED);
        }
        _ => panic!("Expected PossiblyNullOrUndefined, got {:?}", result),
    }
}

#[test]
fn test_property_access_index_signature_no_unchecked() {
    let interner = TypeInterner::new();
    let mut evaluator = PropertyAccessEvaluator::new(&interner);

    let obj = interner.object_with_index(ObjectShape {
        properties: Vec::new(),
        string_index: Some(IndexSignature {
            key_type: TypeId::STRING,
            value_type: TypeId::NUMBER,
            readonly: false,
        }),
        number_index: None,
    });

    let result = evaluator.resolve_property_access(obj, "anything");
    match result {
        PropertyAccessResult::Success { type_id, from_index_signature } => {
            assert_eq!(type_id, TypeId::NUMBER);
            assert!(from_index_signature);
        }
        _ => panic!("Expected success, got {:?}", result),
    }

    evaluator.set_no_unchecked_indexed_access(true);

    let result = evaluator.resolve_property_access(obj, "anything");
    match result {
        PropertyAccessResult::Success { type_id, from_index_signature } => {
            let expected = interner.union(vec![TypeId::NUMBER, TypeId::UNDEFINED]);
            assert_eq!(type_id, expected);
            assert!(from_index_signature);
        }
        _ => panic!("Expected success, got {:?}", result),
    }
}

#[test]
fn test_property_access_string() {
    let interner = TypeInterner::new();
    let evaluator = PropertyAccessEvaluator::new(&interner);

    let result = evaluator.resolve_property_access(TypeId::STRING, "length");
    match result {
        PropertyAccessResult::Success { type_id: t, .. } => assert_eq!(t, TypeId::NUMBER),
        _ => panic!("Expected success, got {:?}", result),
    }
}

#[test]
fn test_property_access_number_method() {
    let interner = TypeInterner::new();
    let evaluator = PropertyAccessEvaluator::new(&interner);

    let result = evaluator.resolve_property_access(TypeId::NUMBER, "toFixed");
    match result {
        PropertyAccessResult::Success { type_id, .. } => match interner.lookup(type_id) {
            Some(TypeKey::Function(func)) => assert_eq!(func.return_type, TypeId::STRING),
            other => panic!("Expected function, got {:?}", other),
        },
        _ => panic!("Expected success, got {:?}", result),
    }
}

#[test]
fn test_property_access_boolean_method() {
    let interner = TypeInterner::new();
    let evaluator = PropertyAccessEvaluator::new(&interner);

    let result = evaluator.resolve_property_access(TypeId::BOOLEAN, "valueOf");
    match result {
        PropertyAccessResult::Success { type_id, .. } => match interner.lookup(type_id) {
            Some(TypeKey::Function(func)) => assert_eq!(func.return_type, TypeId::BOOLEAN),
            other => panic!("Expected function, got {:?}", other),
        },
        _ => panic!("Expected success, got {:?}", result),
    }
}

#[test]
fn test_property_access_bigint_method() {
    let interner = TypeInterner::new();
    let evaluator = PropertyAccessEvaluator::new(&interner);

    let result = evaluator.resolve_property_access(TypeId::BIGINT, "toString");
    match result {
        PropertyAccessResult::Success { type_id, .. } => match interner.lookup(type_id) {
            Some(TypeKey::Function(func)) => assert_eq!(func.return_type, TypeId::STRING),
            other => panic!("Expected function, got {:?}", other),
        },
        _ => panic!("Expected success, got {:?}", result),
    }
}

#[test]
fn test_property_access_literal_string_length() {
    let interner = TypeInterner::new();
    let evaluator = PropertyAccessEvaluator::new(&interner);

    let literal = interner.literal_string("hello");
    let result = evaluator.resolve_property_access(literal, "length");
    match result {
        PropertyAccessResult::Success { type_id: t, .. } => assert_eq!(t, TypeId::NUMBER),
        _ => panic!("Expected success, got {:?}", result),
    }
}

#[test]
fn test_binary_op_addition() {
    let interner = TypeInterner::new();
    let evaluator = BinaryOpEvaluator::new(&interner);

    // number + number = number
    let result = evaluator.evaluate(TypeId::NUMBER, TypeId::NUMBER, "+");
    match result {
        BinaryOpResult::Success(t) => assert_eq!(t, TypeId::NUMBER),
        _ => panic!("Expected success, got {:?}", result),
    }

    // string + number = string
    let result = evaluator.evaluate(TypeId::STRING, TypeId::NUMBER, "+");
    match result {
        BinaryOpResult::Success(t) => assert_eq!(t, TypeId::STRING),
        _ => panic!("Expected success, got {:?}", result),
    }
}

#[test]
fn test_binary_op_logical() {
    let interner = TypeInterner::new();
    let evaluator = BinaryOpEvaluator::new(&interner);

    // number && string = number | string
    let result = evaluator.evaluate(TypeId::NUMBER, TypeId::STRING, "&&");
    match result {
        BinaryOpResult::Success(t) => {
            // Should be a union type
            let key = interner.lookup(t).unwrap();
            match key {
                TypeKey::Union(members) => {
                    assert!(members.contains(&TypeId::NUMBER));
                    assert!(members.contains(&TypeId::STRING));
                }
                _ => panic!("Expected union, got {:?}", key),
            }
        }
        _ => panic!("Expected success, got {:?}", result),
    }
}

#[test]
fn test_call_generic_function_identity() {
    let interner = TypeInterner::new();
    let mut subtype = SubtypeChecker::new(&interner);
    let mut evaluator = CallEvaluator::new(&interner, &mut subtype);

    // Create type parameter T
    let t_param = TypeParamInfo {
        name: interner.intern_string("T"),
        constraint: None,
        default: None,
    };
    let t_type = interner.intern(TypeKey::TypeParameter(t_param.clone()));

    // function identity<T>(x: T): T
    let func = interner.function(FunctionShape {
        type_params: vec![t_param],
        params: vec![ParamInfo {
            name: Some(interner.intern_string("x")),
            type_id: t_type,
            optional: false,
            rest: false,
        }],
        return_type: t_type,
        type_predicate: None,
        is_constructor: false,
    });

    // Call identity(42) -> should infer T = number
    let result = evaluator.resolve_call(func, &[TypeId::NUMBER]);
    match result {
        CallResult::Success(ret) => assert_eq!(ret, TypeId::NUMBER),
        _ => panic!("Expected success, got {:?}", result),
    }
}

#[test]
fn test_call_generic_function_with_string() {
    let interner = TypeInterner::new();
    let mut subtype = SubtypeChecker::new(&interner);
    let mut evaluator = CallEvaluator::new(&interner, &mut subtype);

    // Create type parameter T
    let t_param = TypeParamInfo {
        name: interner.intern_string("T"),
        constraint: None,
        default: None,
    };
    let t_type = interner.intern(TypeKey::TypeParameter(t_param.clone()));

    // function identity<T>(x: T): T
    let func = interner.function(FunctionShape {
        type_params: vec![t_param],
        params: vec![ParamInfo {
            name: Some(interner.intern_string("x")),
            type_id: t_type,
            optional: false,
            rest: false,
        }],
        return_type: t_type,
        type_predicate: None,
        is_constructor: false,
    });

    // Call identity("hello") -> should infer T = string
    let result = evaluator.resolve_call(func, &[TypeId::STRING]);
    match result {
        CallResult::Success(ret) => assert_eq!(ret, TypeId::STRING),
        _ => panic!("Expected success, got {:?}", result),
    }
}

#[test]
fn test_call_generic_argument_type_mismatch_with_default() {
    let interner = TypeInterner::new();
    let mut subtype = SubtypeChecker::new(&interner);
    let mut evaluator = CallEvaluator::new(&interner, &mut subtype);

    let t_param = TypeParamInfo {
        name: interner.intern_string("T"),
        constraint: None,
        default: Some(TypeId::NUMBER),
    };
    let t_type = interner.intern(TypeKey::TypeParameter(t_param.clone()));
    let param_type = interner.union(vec![t_type, TypeId::NUMBER]);

    let func = interner.function(FunctionShape {
        type_params: vec![t_param],
        params: vec![ParamInfo {
            name: Some(interner.intern_string("x")),
            type_id: param_type,
            optional: false,
            rest: false,
        }],
        return_type: t_type,
        type_predicate: None,
        is_constructor: false,
    });

    let result = evaluator.resolve_call(func, &[TypeId::STRING]);
    match result {
        CallResult::ArgumentTypeMismatch { index, expected, actual } => {
            assert_eq!(index, 0);
            assert_eq!(expected, TypeId::NUMBER);
            assert_eq!(actual, TypeId::STRING);
        }
        _ => panic!("Expected ArgumentTypeMismatch, got {:?}", result),
    }
}

#[test]
fn test_call_generic_argument_count_mismatch() {
    let interner = TypeInterner::new();
    let mut subtype = SubtypeChecker::new(&interner);
    let mut evaluator = CallEvaluator::new(&interner, &mut subtype);

    let t_param = TypeParamInfo {
        name: interner.intern_string("T"),
        constraint: None,
        default: None,
    };
    let t_type = interner.intern(TypeKey::TypeParameter(t_param.clone()));

    let func = interner.function(FunctionShape {
        type_params: vec![t_param],
        params: vec![ParamInfo {
            name: Some(interner.intern_string("x")),
            type_id: t_type,
            optional: false,
            rest: false,
        }],
        return_type: t_type,
        type_predicate: None,
        is_constructor: false,
    });

    let result = evaluator.resolve_call(func, &[]);
    match result {
        CallResult::ArgumentCountMismatch { expected_min, actual, .. } => {
            assert_eq!(expected_min, 1);
            assert_eq!(actual, 0);
        }
        _ => panic!("Expected ArgumentCountMismatch, got {:?}", result),
    }
}

#[test]
fn test_call_generic_rest_tuple_constraint_count_mismatch() {
    let interner = TypeInterner::new();
    let mut subtype = SubtypeChecker::new(&interner);
    let mut evaluator = CallEvaluator::new(&interner, &mut subtype);

    let tuple_constraint = interner.tuple(vec![
        TupleElement { type_id: TypeId::NUMBER, name: None, optional: false, rest: false },
        TupleElement { type_id: TypeId::STRING, name: None, optional: false, rest: false },
    ]);
    let t_param = TypeParamInfo {
        name: interner.intern_string("T"),
        constraint: Some(tuple_constraint),
        default: None,
    };
    let t_type = interner.intern(TypeKey::TypeParameter(t_param.clone()));

    let func = interner.function(FunctionShape {
        type_params: vec![t_param],
        params: vec![ParamInfo {
            name: Some(interner.intern_string("args")),
            type_id: t_type,
            optional: false,
            rest: true,
        }],
        return_type: TypeId::VOID,
        type_predicate: None,
        is_constructor: false,
    });

    let result = evaluator.resolve_call(func, &[]);
    match result {
        CallResult::ArgumentCountMismatch { expected_min, expected_max, actual } => {
            assert_eq!(expected_min, 2);
            assert_eq!(expected_max, Some(2));
            assert_eq!(actual, 0);
        }
        _ => panic!("Expected ArgumentCountMismatch, got {:?}", result),
    }
}

#[test]
fn test_call_generic_default_rest_tuple_count_mismatch() {
    let interner = TypeInterner::new();
    let mut subtype = SubtypeChecker::new(&interner);
    let mut evaluator = CallEvaluator::new(&interner, &mut subtype);

    let tuple_default = interner.tuple(vec![
        TupleElement { type_id: TypeId::NUMBER, name: None, optional: false, rest: false },
        TupleElement { type_id: TypeId::STRING, name: None, optional: false, rest: false },
    ]);
    let t_param = TypeParamInfo {
        name: interner.intern_string("T"),
        constraint: None,
        default: Some(tuple_default),
    };
    let t_type = interner.intern(TypeKey::TypeParameter(t_param.clone()));

    let func = interner.function(FunctionShape {
        type_params: vec![t_param],
        params: vec![ParamInfo {
            name: Some(interner.intern_string("args")),
            type_id: t_type,
            optional: false,
            rest: true,
        }],
        return_type: TypeId::VOID,
        type_predicate: None,
        is_constructor: false,
    });

    let result = evaluator.resolve_call(func, &[]);
    match result {
        CallResult::ArgumentCountMismatch { expected_min, expected_max, actual } => {
            assert_eq!(expected_min, 2);
            assert_eq!(expected_max, Some(2));
            assert_eq!(actual, 0);
        }
        _ => panic!("Expected ArgumentCountMismatch, got {:?}", result),
    }
}

#[test]
fn test_call_generic_default_rest_tuple_optional_allows_empty() {
    let interner = TypeInterner::new();
    let mut subtype = SubtypeChecker::new(&interner);
    let mut evaluator = CallEvaluator::new(&interner, &mut subtype);

    let tuple_default = interner.tuple(vec![
        TupleElement { type_id: TypeId::NUMBER, name: None, optional: true, rest: false },
    ]);
    let t_param = TypeParamInfo {
        name: interner.intern_string("T"),
        constraint: None,
        default: Some(tuple_default),
    };
    let t_type = interner.intern(TypeKey::TypeParameter(t_param.clone()));

    let func = interner.function(FunctionShape {
        type_params: vec![t_param],
        params: vec![ParamInfo {
            name: Some(interner.intern_string("args")),
            type_id: t_type,
            optional: false,
            rest: true,
        }],
        return_type: t_type,
        type_predicate: None,
        is_constructor: false,
    });

    let result = evaluator.resolve_call(func, &[]);
    match result {
        CallResult::Success(ret) => assert_eq!(ret, tuple_default),
        _ => panic!("Expected success, got {:?}", result),
    }
}

#[test]
fn test_call_generic_argument_type_mismatch_non_generic_param() {
    let interner = TypeInterner::new();
    let mut subtype = SubtypeChecker::new(&interner);
    let mut evaluator = CallEvaluator::new(&interner, &mut subtype);

    let t_param = TypeParamInfo {
        name: interner.intern_string("T"),
        constraint: None,
        default: None,
    };
    let t_type = interner.intern(TypeKey::TypeParameter(t_param.clone()));

    // function foo<T>(x: number, y: T): T
    let func = interner.function(FunctionShape {
        type_params: vec![t_param],
        params: vec![
            ParamInfo {
                name: Some(interner.intern_string("x")),
                type_id: TypeId::NUMBER,
                optional: false,
                rest: false,
            },
            ParamInfo {
                name: Some(interner.intern_string("y")),
                type_id: t_type,
                optional: false,
                rest: false,
            },
        ],
        return_type: t_type,
        type_predicate: None,
        is_constructor: false,
    });

    let result = evaluator.resolve_call(func, &[TypeId::STRING, TypeId::NUMBER]);
    match result {
        CallResult::ArgumentTypeMismatch { index, expected, actual } => {
            assert_eq!(index, 0);
            assert_eq!(expected, TypeId::NUMBER);
            assert_eq!(actual, TypeId::STRING);
        }
        _ => panic!("Expected ArgumentTypeMismatch, got {:?}", result),
    }
}

#[test]
fn test_call_generic_callable_signature() {
    let interner = TypeInterner::new();
    let mut subtype = SubtypeChecker::new(&interner);
    let mut evaluator = CallEvaluator::new(&interner, &mut subtype);

    let t_param = TypeParamInfo {
        name: interner.intern_string("T"),
        constraint: None,
        default: None,
    };
    let t_type = interner.intern(TypeKey::TypeParameter(t_param.clone()));

    let callable = interner.callable(CallableShape {
        call_signatures: vec![CallSignature {
            type_params: vec![t_param],
            params: vec![ParamInfo {
                name: Some(interner.intern_string("x")),
                type_id: t_type,
                optional: false,
                rest: false,
            }],
            return_type: t_type,
            type_predicate: None,
        }],
        construct_signatures: Vec::new(),
        properties: Vec::new(),
    });

    let result = evaluator.resolve_call(callable, &[TypeId::NUMBER]);
    match result {
        CallResult::Success(ret) => assert_eq!(ret, TypeId::NUMBER),
        _ => panic!("Expected success, got {:?}", result),
    }
}

#[test]
fn test_call_generic_array_function() {
    let interner = TypeInterner::new();
    let mut subtype = SubtypeChecker::new(&interner);
    let mut evaluator = CallEvaluator::new(&interner, &mut subtype);

    // Create type parameter T
    let t_param = TypeParamInfo {
        name: interner.intern_string("T"),
        constraint: None,
        default: None,
    };
    let t_type = interner.intern(TypeKey::TypeParameter(t_param.clone()));
    let array_t = interner.array(t_type);

    // function first<T>(arr: T[]): T
    let func = interner.function(FunctionShape {
        type_params: vec![t_param],
        params: vec![ParamInfo {
            name: Some(interner.intern_string("arr")),
            type_id: array_t,
            optional: false,
            rest: false,
        }],
        return_type: t_type,
        type_predicate: None,
        is_constructor: false,
    });

    // Call first(number[]) -> should infer T = number
    let number_array = interner.array(TypeId::NUMBER);
    let result = evaluator.resolve_call(func, &[number_array]);
    match result {
        CallResult::Success(ret) => assert_eq!(ret, TypeId::NUMBER),
        _ => panic!("Expected success, got {:?}", result),
    }
}

#[test]
fn test_infer_call_signature_identity() {
    let interner = TypeInterner::new();
    let mut subtype = SubtypeChecker::new(&interner);

    let t_param = TypeParamInfo {
        name: interner.intern_string("T"),
        constraint: None,
        default: None,
    };
    let t_type = interner.intern(TypeKey::TypeParameter(t_param.clone()));

    let sig = CallSignature {
        type_params: vec![t_param],
        params: vec![ParamInfo {
            name: Some(interner.intern_string("x")),
            type_id: t_type,
            optional: false,
            rest: false,
        }],
        return_type: t_type,
        type_predicate: None,
    };

    let result = infer_call_signature(&interner, &mut subtype, &sig, &[TypeId::NUMBER]);
    assert_eq!(result, TypeId::NUMBER);
}

#[test]
fn test_infer_generic_function_identity() {
    let interner = TypeInterner::new();
    let mut subtype = SubtypeChecker::new(&interner);

    let t_param = TypeParamInfo {
        name: interner.intern_string("T"),
        constraint: None,
        default: None,
    };
    let t_type = interner.intern(TypeKey::TypeParameter(t_param.clone()));

    let func = FunctionShape {
        type_params: vec![t_param],
        params: vec![ParamInfo {
            name: Some(interner.intern_string("x")),
            type_id: t_type,
            optional: false,
            rest: false,
        }],
        return_type: t_type,
        type_predicate: None,
        is_constructor: false,
    };

    let result = infer_generic_function(&interner, &mut subtype, &func, &[TypeId::STRING]);
    assert_eq!(result, TypeId::STRING);
}

#[test]
fn test_infer_generic_array_map() {
    let interner = TypeInterner::new();
    let mut subtype = SubtypeChecker::new(&interner);

    let t_param = TypeParamInfo {
        name: interner.intern_string("T"),
        constraint: None,
        default: None,
    };
    let u_param = TypeParamInfo {
        name: interner.intern_string("U"),
        constraint: None,
        default: None,
    };
    let t_type = interner.intern(TypeKey::TypeParameter(t_param.clone()));
    let u_type = interner.intern(TypeKey::TypeParameter(u_param.clone()));
    let array_t = interner.array(t_type);
    let array_u = interner.array(u_type);

    let callback_param = interner.function(FunctionShape {
        type_params: Vec::new(),
        params: vec![ParamInfo {
            name: Some(interner.intern_string("value")),
            type_id: t_type,
            optional: false,
            rest: false,
        }],
        return_type: u_type,
        type_predicate: None,
        is_constructor: false,
    });

    let map_func = FunctionShape {
        type_params: vec![t_param, u_param],
        params: vec![
            ParamInfo {
                name: Some(interner.intern_string("arr")),
                type_id: array_t,
                optional: false,
                rest: false,
            },
            ParamInfo {
                name: Some(interner.intern_string("callback")),
                type_id: callback_param,
                optional: false,
                rest: false,
            },
        ],
        return_type: array_u,
        type_predicate: None,
        is_constructor: false,
    };

    let number_array = interner.array(TypeId::NUMBER);
    let callback_arg = interner.function(FunctionShape {
        type_params: Vec::new(),
        params: vec![ParamInfo {
            name: Some(interner.intern_string("x")),
            type_id: TypeId::NUMBER,
            optional: false,
            rest: false,
        }],
        return_type: TypeId::STRING,
        type_predicate: None,
        is_constructor: false,
    });

    let result = infer_generic_function(
        &interner,
        &mut subtype,
        &map_func,
        &[number_array, callback_arg],
    );
    let expected = interner.array(TypeId::STRING);
    assert_eq!(result, expected);
}

#[test]
fn test_infer_generic_constructor_instantiation() {
    let interner = TypeInterner::new();
    let mut subtype = SubtypeChecker::new(&interner);

    let t_param = TypeParamInfo {
        name: interner.intern_string("T"),
        constraint: None,
        default: None,
    };
    let t_type = interner.intern(TypeKey::TypeParameter(t_param.clone()));

    let box_base = interner.reference(SymbolRef(42));
    let box_t = interner.application(box_base, vec![t_type]);

    let ctor = FunctionShape {
        type_params: vec![t_param],
        params: vec![ParamInfo {
            name: Some(interner.intern_string("value")),
            type_id: t_type,
            optional: false,
            rest: false,
        }],
        return_type: box_t,
        type_predicate: None,
        is_constructor: true,
    };

    let result = infer_generic_function(&interner, &mut subtype, &ctor, &[TypeId::NUMBER]);
    let expected = interner.application(box_base, vec![TypeId::NUMBER]);
    assert_eq!(result, expected);
}

#[test]
fn test_infer_generic_application_param() {
    let interner = TypeInterner::new();
    let mut subtype = SubtypeChecker::new(&interner);

    let t_param = TypeParamInfo {
        name: interner.intern_string("T"),
        constraint: None,
        default: None,
    };
    let t_type = interner.intern(TypeKey::TypeParameter(t_param.clone()));

    let promise_base = interner.reference(SymbolRef(77));
    let promise_t = interner.application(promise_base, vec![t_type]);

    let func = FunctionShape {
        type_params: vec![t_param],
        params: vec![ParamInfo {
            name: Some(interner.intern_string("value")),
            type_id: promise_t,
            optional: false,
            rest: false,
        }],
        return_type: t_type,
        type_predicate: None,
        is_constructor: false,
    };

    let arg = interner.application(promise_base, vec![TypeId::NUMBER]);
    let result = infer_generic_function(&interner, &mut subtype, &func, &[arg]);
    assert_eq!(result, TypeId::NUMBER);
}

#[test]
fn test_infer_generic_object_property() {
    let interner = TypeInterner::new();
    let mut subtype = SubtypeChecker::new(&interner);

    let t_param = TypeParamInfo {
        name: interner.intern_string("T"),
        constraint: None,
        default: None,
    };
    let t_type = interner.intern(TypeKey::TypeParameter(t_param.clone()));

    let boxed_t = interner.object(vec![PropertyInfo {
        name: interner.intern_string("value"),
        type_id: t_type,
        optional: false,
        readonly: false,
        is_method: false,
    }]);

    let func = FunctionShape {
        type_params: vec![t_param],
        params: vec![ParamInfo {
            name: Some(interner.intern_string("boxed")),
            type_id: boxed_t,
            optional: false,
            rest: false,
        }],
        return_type: t_type,
        type_predicate: None,
        is_constructor: false,
    };

    let arg = interner.object(vec![PropertyInfo {
        name: interner.intern_string("value"),
        type_id: TypeId::STRING,
        optional: false,
        readonly: false,
        is_method: false,
    }]);
    let result = infer_generic_function(&interner, &mut subtype, &func, &[arg]);
    assert_eq!(result, TypeId::STRING);
}

#[test]
fn test_infer_generic_tuple_element() {
    let interner = TypeInterner::new();
    let mut subtype = SubtypeChecker::new(&interner);

    let t_param = TypeParamInfo {
        name: interner.intern_string("T"),
        constraint: None,
        default: None,
    };
    let t_type = interner.intern(TypeKey::TypeParameter(t_param.clone()));

    let tuple_t = interner.tuple(vec![
        TupleElement { type_id: t_type, name: None, optional: false, rest: false },
        TupleElement { type_id: t_type, name: None, optional: false, rest: false },
    ]);

    let func = FunctionShape {
        type_params: vec![t_param],
        params: vec![ParamInfo {
            name: Some(interner.intern_string("pair")),
            type_id: tuple_t,
            optional: false,
            rest: false,
        }],
        return_type: t_type,
        type_predicate: None,
        is_constructor: false,
    };

    let tuple_arg = interner.tuple(vec![
        TupleElement { type_id: TypeId::NUMBER, name: None, optional: false, rest: false },
        TupleElement { type_id: TypeId::NUMBER, name: None, optional: false, rest: false },
    ]);
    let result = infer_generic_function(&interner, &mut subtype, &func, &[tuple_arg]);
    assert_eq!(result, TypeId::NUMBER);
}

#[test]
fn test_infer_generic_tuple_rest_elements() {
    let interner = TypeInterner::new();
    let mut subtype = SubtypeChecker::new(&interner);

    let t_param = TypeParamInfo {
        name: interner.intern_string("T"),
        constraint: None,
        default: None,
    };
    let t_type = interner.intern(TypeKey::TypeParameter(t_param.clone()));
    let t_array = interner.array(t_type);

    let tuple_t = interner.tuple(vec![
        TupleElement { type_id: t_type, name: None, optional: false, rest: false },
        TupleElement { type_id: t_array, name: None, optional: false, rest: true },
    ]);

    let func = FunctionShape {
        type_params: vec![t_param],
        params: vec![ParamInfo {
            name: Some(interner.intern_string("items")),
            type_id: tuple_t,
            optional: false,
            rest: false,
        }],
        return_type: t_type,
        type_predicate: None,
        is_constructor: false,
    };

    let tuple_arg = interner.tuple(vec![
        TupleElement { type_id: TypeId::NUMBER, name: None, optional: false, rest: false },
        TupleElement { type_id: TypeId::STRING, name: None, optional: false, rest: false },
    ]);
    let result = infer_generic_function(&interner, &mut subtype, &func, &[tuple_arg]);
    let expected = interner.union(vec![TypeId::NUMBER, TypeId::STRING]);
    assert_eq!(result, expected);
}

#[test]
fn test_infer_generic_tuple_rest_parameter() {
    let interner = TypeInterner::new();
    let mut subtype = SubtypeChecker::new(&interner);

    let t_param = TypeParamInfo {
        name: interner.intern_string("T"),
        constraint: None,
        default: None,
    };
    let t_type = interner.intern(TypeKey::TypeParameter(t_param.clone()));

    let tuple_t = interner.tuple(vec![
        TupleElement { type_id: t_type, name: None, optional: false, rest: false },
        TupleElement { type_id: t_type, name: None, optional: false, rest: false },
    ]);

    let func = FunctionShape {
        type_params: vec![t_param],
        params: vec![ParamInfo {
            name: Some(interner.intern_string("items")),
            type_id: tuple_t,
            optional: false,
            rest: true,
        }],
        return_type: t_type,
        type_predicate: None,
        is_constructor: false,
    };

    let result = infer_generic_function(
        &interner,
        &mut subtype,
        &func,
        &[TypeId::NUMBER, TypeId::STRING],
    );
    let expected = interner.union(vec![TypeId::NUMBER, TypeId::STRING]);
    assert_eq!(result, expected);
}

#[test]
fn test_infer_generic_tuple_rest_from_rest_argument() {
    let interner = TypeInterner::new();
    let mut subtype = SubtypeChecker::new(&interner);

    let t_param = TypeParamInfo {
        name: interner.intern_string("T"),
        constraint: None,
        default: None,
    };
    let t_type = interner.intern(TypeKey::TypeParameter(t_param.clone()));
    let t_array = interner.array(t_type);

    let tuple_t = interner.tuple(vec![
        TupleElement { type_id: t_type, name: None, optional: false, rest: false },
        TupleElement { type_id: t_array, name: None, optional: false, rest: true },
    ]);

    let func = FunctionShape {
        type_params: vec![t_param],
        params: vec![ParamInfo {
            name: Some(interner.intern_string("items")),
            type_id: tuple_t,
            optional: false,
            rest: false,
        }],
        return_type: t_type,
        type_predicate: None,
        is_constructor: false,
    };

    let string_array = interner.array(TypeId::STRING);
    let tuple_arg = interner.tuple(vec![
        TupleElement { type_id: TypeId::NUMBER, name: None, optional: false, rest: false },
        TupleElement { type_id: string_array, name: None, optional: false, rest: true },
    ]);

    let result = infer_generic_function(&interner, &mut subtype, &func, &[tuple_arg]);
    let expected = interner.union(vec![TypeId::NUMBER, TypeId::STRING]);
    assert_eq!(result, expected);
}

#[test]
fn test_infer_generic_index_signature() {
    let interner = TypeInterner::new();
    let mut subtype = SubtypeChecker::new(&interner);

    let t_param = TypeParamInfo {
        name: interner.intern_string("T"),
        constraint: None,
        default: None,
    };
    let t_type = interner.intern(TypeKey::TypeParameter(t_param.clone()));

    let indexed_t = interner.object_with_index(ObjectShape {
        properties: Vec::new(),
        string_index: Some(IndexSignature {
            key_type: TypeId::STRING,
            value_type: t_type,
            readonly: false,
        }),
        number_index: None,
    });

    let func = FunctionShape {
        type_params: vec![t_param],
        params: vec![ParamInfo {
            name: Some(interner.intern_string("bag")),
            type_id: indexed_t,
            optional: false,
            rest: false,
        }],
        return_type: t_type,
        type_predicate: None,
        is_constructor: false,
    };

    let indexed_number = interner.object_with_index(ObjectShape {
        properties: Vec::new(),
        string_index: Some(IndexSignature {
            key_type: TypeId::STRING,
            value_type: TypeId::NUMBER,
            readonly: false,
        }),
        number_index: None,
    });

    let result = infer_generic_function(&interner, &mut subtype, &func, &[indexed_number]);
    assert_eq!(result, TypeId::NUMBER);
}

#[test]
fn test_infer_generic_union_source() {
    let interner = TypeInterner::new();
    let mut subtype = SubtypeChecker::new(&interner);

    let t_param = TypeParamInfo {
        name: interner.intern_string("T"),
        constraint: None,
        default: None,
    };
    let t_type = interner.intern(TypeKey::TypeParameter(t_param.clone()));

    let boxed_t = interner.object(vec![PropertyInfo {
        name: interner.intern_string("value"),
        type_id: t_type,
        optional: false,
        readonly: false,
        is_method: false,
    }]);

    let func = FunctionShape {
        type_params: vec![t_param],
        params: vec![ParamInfo {
            name: Some(interner.intern_string("boxed")),
            type_id: boxed_t,
            optional: false,
            rest: false,
        }],
        return_type: t_type,
        type_predicate: None,
        is_constructor: false,
    };

    let boxed_number = interner.object(vec![PropertyInfo {
        name: interner.intern_string("value"),
        type_id: TypeId::NUMBER,
        optional: false,
        readonly: false,
        is_method: false,
    }]);
    let boxed_string = interner.object(vec![PropertyInfo {
        name: interner.intern_string("value"),
        type_id: TypeId::STRING,
        optional: false,
        readonly: false,
        is_method: false,
    }]);

    let union_arg = interner.union(vec![boxed_number, boxed_string]);
    let result = infer_generic_function(&interner, &mut subtype, &func, &[union_arg]);
    let expected = interner.union(vec![TypeId::NUMBER, TypeId::STRING]);
    assert_eq!(result, expected);
}

#[test]
fn test_infer_generic_optional_union_target() {
    let interner = TypeInterner::new();
    let mut subtype = SubtypeChecker::new(&interner);

    let t_param = TypeParamInfo {
        name: interner.intern_string("T"),
        constraint: None,
        default: None,
    };
    let t_type = interner.intern(TypeKey::TypeParameter(t_param.clone()));

    let optional_t = interner.union(vec![t_type, TypeId::UNDEFINED]);
    let func = FunctionShape {
        type_params: vec![t_param],
        params: vec![ParamInfo {
            name: Some(interner.intern_string("value")),
            type_id: optional_t,
            optional: false,
            rest: false,
        }],
        return_type: t_type,
        type_predicate: None,
        is_constructor: false,
    };

    let result = infer_generic_function(&interner, &mut subtype, &func, &[TypeId::NUMBER]);
    assert_eq!(result, TypeId::NUMBER);
}

#[test]
fn test_infer_generic_optional_union_target_with_null() {
    let interner = TypeInterner::new();
    let mut subtype = SubtypeChecker::new(&interner);

    let t_param = TypeParamInfo {
        name: interner.intern_string("T"),
        constraint: None,
        default: None,
    };
    let t_type = interner.intern(TypeKey::TypeParameter(t_param.clone()));

    let optional_t = interner.union(vec![t_type, TypeId::UNDEFINED, TypeId::NULL]);
    let func = FunctionShape {
        type_params: vec![t_param],
        params: vec![ParamInfo {
            name: Some(interner.intern_string("value")),
            type_id: optional_t,
            optional: false,
            rest: false,
        }],
        return_type: t_type,
        type_predicate: None,
        is_constructor: false,
    };

    let result = infer_generic_function(&interner, &mut subtype, &func, &[TypeId::NUMBER]);
    assert_eq!(result, TypeId::NUMBER);
}

#[test]
fn test_infer_generic_rest_parameters() {
    let interner = TypeInterner::new();
    let mut subtype = SubtypeChecker::new(&interner);

    let t_param = TypeParamInfo {
        name: interner.intern_string("T"),
        constraint: None,
        default: None,
    };
    let t_type = interner.intern(TypeKey::TypeParameter(t_param.clone()));
    let array_t = interner.array(t_type);

    let func = FunctionShape {
        type_params: vec![t_param],
        params: vec![ParamInfo {
            name: Some(interner.intern_string("items")),
            type_id: array_t,
            optional: false,
            rest: true,
        }],
        return_type: t_type,
        type_predicate: None,
        is_constructor: false,
    };

    let result = infer_generic_function(
        &interner,
        &mut subtype,
        &func,
        &[TypeId::NUMBER, TypeId::STRING],
    );
    let expected = interner.union(vec![TypeId::NUMBER, TypeId::STRING]);
    assert_eq!(result, expected);
}

#[test]
fn test_infer_generic_rest_tuple_type_param() {
    let interner = TypeInterner::new();
    let mut subtype = SubtypeChecker::new(&interner);

    let t_param = TypeParamInfo {
        name: interner.intern_string("T"),
        constraint: Some(interner.array(TypeId::ANY)),
        default: None,
    };
    let t_type = interner.intern(TypeKey::TypeParameter(t_param.clone()));

    let func = FunctionShape {
        type_params: vec![t_param],
        params: vec![ParamInfo {
            name: Some(interner.intern_string("args")),
            type_id: t_type,
            optional: false,
            rest: true,
        }],
        return_type: t_type,
        type_predicate: None,
        is_constructor: false,
    };

    let result = infer_generic_function(
        &interner,
        &mut subtype,
        &func,
        &[TypeId::NUMBER, TypeId::STRING],
    );
    let expected = interner.tuple(vec![
        TupleElement { type_id: TypeId::NUMBER, name: None, optional: false, rest: false },
        TupleElement { type_id: TypeId::STRING, name: None, optional: false, rest: false },
    ]);
    assert_eq!(result, expected);
}

#[test]
fn test_infer_generic_tuple_rest_type_param() {
    let interner = TypeInterner::new();
    let mut subtype = SubtypeChecker::new(&interner);

    let t_param = TypeParamInfo {
        name: interner.intern_string("T"),
        constraint: Some(interner.array(TypeId::ANY)),
        default: None,
    };
    let t_type = interner.intern(TypeKey::TypeParameter(t_param.clone()));

    let tuple_t = interner.tuple(vec![
        TupleElement { type_id: TypeId::NUMBER, name: None, optional: false, rest: false },
        TupleElement { type_id: t_type, name: None, optional: false, rest: true },
    ]);

    let func = FunctionShape {
        type_params: vec![t_param],
        params: vec![ParamInfo {
            name: Some(interner.intern_string("args")),
            type_id: tuple_t,
            optional: false,
            rest: true,
        }],
        return_type: t_type,
        type_predicate: None,
        is_constructor: false,
    };

    let result = infer_generic_function(
        &interner,
        &mut subtype,
        &func,
        &[TypeId::NUMBER, TypeId::STRING, TypeId::BOOLEAN],
    );
    let expected = interner.tuple(vec![
        TupleElement { type_id: TypeId::STRING, name: None, optional: false, rest: false },
        TupleElement { type_id: TypeId::BOOLEAN, name: None, optional: false, rest: false },
    ]);
    assert_eq!(result, expected);
}

#[test]
fn test_infer_generic_tuple_rest_in_tuple_param() {
    let interner = TypeInterner::new();
    let mut subtype = SubtypeChecker::new(&interner);

    let t_param = TypeParamInfo {
        name: interner.intern_string("T"),
        constraint: Some(interner.array(TypeId::ANY)),
        default: None,
    };
    let t_type = interner.intern(TypeKey::TypeParameter(t_param.clone()));

    let tuple_t = interner.tuple(vec![
        TupleElement { type_id: TypeId::NUMBER, name: None, optional: false, rest: false },
        TupleElement { type_id: t_type, name: None, optional: false, rest: true },
    ]);

    let func = FunctionShape {
        type_params: vec![t_param],
        params: vec![ParamInfo {
            name: Some(interner.intern_string("args")),
            type_id: tuple_t,
            optional: false,
            rest: false,
        }],
        return_type: t_type,
        type_predicate: None,
        is_constructor: false,
    };

    let tuple_arg = interner.tuple(vec![
        TupleElement { type_id: TypeId::NUMBER, name: None, optional: false, rest: false },
        TupleElement { type_id: TypeId::STRING, name: None, optional: false, rest: false },
        TupleElement { type_id: TypeId::BOOLEAN, name: None, optional: false, rest: false },
    ]);

    let result = infer_generic_function(&interner, &mut subtype, &func, &[tuple_arg]);
    let expected = interner.tuple(vec![
        TupleElement { type_id: TypeId::STRING, name: None, optional: false, rest: false },
        TupleElement { type_id: TypeId::BOOLEAN, name: None, optional: false, rest: false },
    ]);
    assert_eq!(result, expected);
}

#[test]
fn test_infer_generic_tuple_rest_in_tuple_param_from_rest_argument() {
    let interner = TypeInterner::new();
    let mut subtype = SubtypeChecker::new(&interner);

    let t_param = TypeParamInfo {
        name: interner.intern_string("T"),
        constraint: Some(interner.array(TypeId::ANY)),
        default: None,
    };
    let t_type = interner.intern(TypeKey::TypeParameter(t_param.clone()));

    let tuple_t = interner.tuple(vec![
        TupleElement { type_id: TypeId::NUMBER, name: None, optional: false, rest: false },
        TupleElement { type_id: t_type, name: None, optional: false, rest: true },
    ]);

    let func = FunctionShape {
        type_params: vec![t_param],
        params: vec![ParamInfo {
            name: Some(interner.intern_string("args")),
            type_id: tuple_t,
            optional: false,
            rest: false,
        }],
        return_type: t_type,
        type_predicate: None,
        is_constructor: false,
    };

    let string_array = interner.array(TypeId::STRING);
    let tuple_arg = interner.tuple(vec![
        TupleElement { type_id: TypeId::NUMBER, name: None, optional: false, rest: false },
        TupleElement { type_id: string_array, name: None, optional: false, rest: true },
    ]);

    let result = infer_generic_function(&interner, &mut subtype, &func, &[tuple_arg]);
    let expected = interner.tuple(vec![TupleElement {
        type_id: string_array,
        name: None,
        optional: false,
        rest: true,
    }]);
    assert_eq!(result, expected);
}

#[test]
fn test_infer_generic_tuple_rest_in_tuple_param_from_rest_argument_with_fixed_tail() {
    let interner = TypeInterner::new();
    let mut subtype = SubtypeChecker::new(&interner);

    let t_param = TypeParamInfo {
        name: interner.intern_string("T"),
        constraint: Some(interner.array(TypeId::ANY)),
        default: None,
    };
    let t_type = interner.intern(TypeKey::TypeParameter(t_param.clone()));

    let tuple_t = interner.tuple(vec![
        TupleElement { type_id: TypeId::NUMBER, name: None, optional: false, rest: false },
        TupleElement { type_id: t_type, name: None, optional: false, rest: true },
    ]);

    let func = FunctionShape {
        type_params: vec![t_param],
        params: vec![ParamInfo {
            name: Some(interner.intern_string("args")),
            type_id: tuple_t,
            optional: false,
            rest: false,
        }],
        return_type: t_type,
        type_predicate: None,
        is_constructor: false,
    };

    let boolean_array = interner.array(TypeId::BOOLEAN);
    let tuple_arg = interner.tuple(vec![
        TupleElement { type_id: TypeId::NUMBER, name: None, optional: false, rest: false },
        TupleElement { type_id: TypeId::STRING, name: None, optional: false, rest: false },
        TupleElement { type_id: boolean_array, name: None, optional: false, rest: true },
    ]);

    let result = infer_generic_function(&interner, &mut subtype, &func, &[tuple_arg]);
    let expected = interner.tuple(vec![
        TupleElement { type_id: TypeId::STRING, name: None, optional: false, rest: false },
        TupleElement { type_id: boolean_array, name: None, optional: false, rest: true },
    ]);
    assert_eq!(result, expected);
}

#[test]
fn test_infer_generic_tuple_rest_in_tuple_param_empty_tail() {
    let interner = TypeInterner::new();
    let mut subtype = SubtypeChecker::new(&interner);

    let t_param = TypeParamInfo {
        name: interner.intern_string("T"),
        constraint: Some(interner.array(TypeId::ANY)),
        default: None,
    };
    let t_type = interner.intern(TypeKey::TypeParameter(t_param.clone()));

    let tuple_t = interner.tuple(vec![
        TupleElement { type_id: TypeId::NUMBER, name: None, optional: false, rest: false },
        TupleElement { type_id: t_type, name: None, optional: false, rest: true },
    ]);

    let func = FunctionShape {
        type_params: vec![t_param],
        params: vec![ParamInfo {
            name: Some(interner.intern_string("args")),
            type_id: tuple_t,
            optional: false,
            rest: false,
        }],
        return_type: t_type,
        type_predicate: None,
        is_constructor: false,
    };

    let tuple_arg = interner.tuple(vec![TupleElement {
        type_id: TypeId::NUMBER,
        name: None,
        optional: false,
        rest: false,
    }]);

    let result = infer_generic_function(&interner, &mut subtype, &func, &[tuple_arg]);
    let expected = interner.tuple(vec![]);
    assert_eq!(result, expected);
}

#[test]
fn test_infer_generic_default_type_param() {
    let interner = TypeInterner::new();
    let mut subtype = SubtypeChecker::new(&interner);

    let t_param = TypeParamInfo {
        name: interner.intern_string("T"),
        constraint: None,
        default: Some(TypeId::STRING),
    };
    let t_type = interner.intern(TypeKey::TypeParameter(t_param.clone()));

    let func = FunctionShape {
        type_params: vec![t_param],
        params: vec![ParamInfo {
            name: Some(interner.intern_string("value")),
            type_id: t_type,
            optional: true,
            rest: false,
        }],
        return_type: t_type,
        type_predicate: None,
        is_constructor: false,
    };

    let result = infer_generic_function(&interner, &mut subtype, &func, &[]);
    assert_eq!(result, TypeId::STRING);
}

#[test]
fn test_infer_generic_default_depends_on_prior_param() {
    let interner = TypeInterner::new();
    let mut subtype = SubtypeChecker::new(&interner);

    let t_param = TypeParamInfo {
        name: interner.intern_string("T"),
        constraint: None,
        default: None,
    };
    let t_type = interner.intern(TypeKey::TypeParameter(t_param.clone()));

    let u_param = TypeParamInfo {
        name: interner.intern_string("U"),
        constraint: None,
        default: Some(t_type),
    };
    let u_type = interner.intern(TypeKey::TypeParameter(u_param.clone()));

    let func = FunctionShape {
        type_params: vec![t_param, u_param],
        params: vec![ParamInfo {
            name: Some(interner.intern_string("value")),
            type_id: t_type,
            optional: false,
            rest: false,
        }],
        return_type: u_type,
        type_predicate: None,
        is_constructor: false,
    };

    let result = infer_generic_function(&interner, &mut subtype, &func, &[TypeId::NUMBER]);
    assert_eq!(result, TypeId::NUMBER);
}

#[test]
fn test_infer_generic_constraint_fallback() {
    let interner = TypeInterner::new();
    let mut subtype = SubtypeChecker::new(&interner);

    let t_param = TypeParamInfo {
        name: interner.intern_string("T"),
        constraint: Some(TypeId::NUMBER),
        default: None,
    };
    let t_type = interner.intern(TypeKey::TypeParameter(t_param.clone()));

    let func = FunctionShape {
        type_params: vec![t_param],
        params: vec![ParamInfo {
            name: Some(interner.intern_string("value")),
            type_id: t_type,
            optional: true,
            rest: false,
        }],
        return_type: t_type,
        type_predicate: None,
        is_constructor: false,
    };

    let result = infer_generic_function(&interner, &mut subtype, &func, &[]);
    assert_eq!(result, TypeId::NUMBER);
}

#[test]
fn test_infer_generic_constraint_violation() {
    let interner = TypeInterner::new();
    let mut subtype = SubtypeChecker::new(&interner);

    let t_param = TypeParamInfo {
        name: interner.intern_string("T"),
        constraint: Some(TypeId::STRING),
        default: None,
    };
    let t_type = interner.intern(TypeKey::TypeParameter(t_param.clone()));

    let func = FunctionShape {
        type_params: vec![t_param],
        params: vec![ParamInfo {
            name: Some(interner.intern_string("value")),
            type_id: t_type,
            optional: false,
            rest: false,
        }],
        return_type: t_type,
        type_predicate: None,
        is_constructor: false,
    };

    let result = infer_generic_function(&interner, &mut subtype, &func, &[TypeId::NUMBER]);
    assert_eq!(result, TypeId::ANY);
}

#[test]
fn test_infer_generic_constraint_depends_on_prior_param() {
    let interner = TypeInterner::new();
    let mut subtype = SubtypeChecker::new(&interner);

    let t_param = TypeParamInfo {
        name: interner.intern_string("T"),
        constraint: None,
        default: None,
    };
    let t_type = interner.intern(TypeKey::TypeParameter(t_param.clone()));

    let u_param = TypeParamInfo {
        name: interner.intern_string("U"),
        constraint: Some(t_type),
        default: None,
    };
    let u_type = interner.intern(TypeKey::TypeParameter(u_param.clone()));

    let func = FunctionShape {
        type_params: vec![t_param, u_param],
        params: vec![
            ParamInfo {
                name: Some(interner.intern_string("first")),
                type_id: t_type,
                optional: false,
                rest: false,
            },
            ParamInfo {
                name: Some(interner.intern_string("second")),
                type_id: u_type,
                optional: false,
                rest: false,
            },
        ],
        return_type: u_type,
        type_predicate: None,
        is_constructor: false,
    };

    let result = infer_generic_function(
        &interner,
        &mut subtype,
        &func,
        &[TypeId::STRING, TypeId::STRING],
    );
    assert_eq!(result, TypeId::STRING);
}
