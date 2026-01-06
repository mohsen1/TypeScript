//! Tests for type operations.

use super::*;
use crate::solver::types::*;
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
            name: Some("x".into()),
            type_id: TypeId::NUMBER,
            optional: false,
            rest: false,
        }],
        return_type: TypeId::STRING,
        type_params: Vec::new(),
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
            name: Some("x".into()),
            type_id: TypeId::NUMBER,
            optional: false,
            rest: false,
        }],
        return_type: TypeId::STRING,
        type_params: Vec::new(),
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
            name: Some("x".into()),
            type_id: TypeId::NUMBER,
            optional: false,
            rest: false,
        }],
        return_type: TypeId::STRING,
        type_params: Vec::new(),
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
fn test_property_access_object() {
    let interner = TypeInterner::new();
    let evaluator = PropertyAccessEvaluator::new(&interner);

    // { x: number, y: string }
    let obj = interner.object(vec![
        PropertyInfo {
            name: "x".into(),
            type_id: TypeId::NUMBER,
            optional: false,
            readonly: false,
        },
        PropertyInfo {
            name: "y".into(),
            type_id: TypeId::STRING,
            optional: false,
            readonly: false,
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
    use std::sync::Arc;

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
            name: Some("x".into()),
            type_id: t_type,
            optional: false,
            rest: false,
        }],
        return_type: t_type,
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
    use std::sync::Arc;

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
            name: Some("x".into()),
            type_id: t_type,
            optional: false,
            rest: false,
        }],
        return_type: t_type,
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
fn test_call_generic_array_function() {
    use std::sync::Arc;

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
            name: Some("arr".into()),
            type_id: array_t,
            optional: false,
            rest: false,
        }],
        return_type: t_type,
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
