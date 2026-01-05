use super::*;
use std::sync::Arc;

#[test]
fn test_substitution_basic() {
    let mut subst = TypeSubstitution::new();

    // Initially empty
    assert!(subst.is_empty());
    assert_eq!(subst.len(), 0);

    // Add a substitution
    subst.insert(Arc::from("T"), TypeId::STRING);
    assert_eq!(subst.get("T"), Some(TypeId::STRING));
    assert_eq!(subst.get("U"), None);
    assert_eq!(subst.len(), 1);
}

#[test]
fn test_substitution_from_args() {
    let type_params = vec![
        TypeParamInfo { name: Arc::from("T"), constraint: None, default: None },
        TypeParamInfo { name: Arc::from("U"), constraint: None, default: None },
    ];
    let type_args = vec![TypeId::STRING, TypeId::NUMBER];

    let subst = TypeSubstitution::from_args(&type_params, &type_args);

    assert_eq!(subst.get("T"), Some(TypeId::STRING));
    assert_eq!(subst.get("U"), Some(TypeId::NUMBER));
    assert_eq!(subst.get("V"), None);
}

#[test]
fn test_instantiate_type_parameter() {
    let interner = TypeInterner::new();

    // Create a type parameter T
    let type_param = interner.intern(TypeKey::TypeParameter(TypeParamInfo {
        name: Arc::from("T"),
        constraint: None,
        default: None,
    }));

    // No substitution - should stay as is
    let empty_subst = TypeSubstitution::new();
    let result = instantiate_type(&interner, type_param, &empty_subst);
    assert_eq!(result, type_param);

    // With substitution T = string
    let mut subst = TypeSubstitution::new();
    subst.insert(Arc::from("T"), TypeId::STRING);
    let result = instantiate_type(&interner, type_param, &subst);
    assert_eq!(result, TypeId::STRING);
}

#[test]
fn test_instantiate_array() {
    let interner = TypeInterner::new();

    // Create Array<T>
    let type_param_t = interner.intern(TypeKey::TypeParameter(TypeParamInfo {
        name: Arc::from("T"),
        constraint: None,
        default: None,
    }));
    let array_t = interner.array(type_param_t);

    // Substitute T = number -> Array<number>
    let mut subst = TypeSubstitution::new();
    subst.insert(Arc::from("T"), TypeId::NUMBER);
    let result = instantiate_type(&interner, array_t, &subst);

    // Result should be Array<number>
    let expected = interner.array(TypeId::NUMBER);
    assert_eq!(result, expected);
}

#[test]
fn test_instantiate_union() {
    let interner = TypeInterner::new();

    // Create T | null
    let type_param_t = interner.intern(TypeKey::TypeParameter(TypeParamInfo {
        name: Arc::from("T"),
        constraint: None,
        default: None,
    }));
    let union = interner.union(vec![type_param_t, TypeId::NULL]);

    // Substitute T = string -> string | null
    let mut subst = TypeSubstitution::new();
    subst.insert(Arc::from("T"), TypeId::STRING);
    let result = instantiate_type(&interner, union, &subst);

    // Result should be string | null
    let expected = interner.union(vec![TypeId::STRING, TypeId::NULL]);
    assert_eq!(result, expected);
}

#[test]
fn test_instantiate_object() {
    let interner = TypeInterner::new();

    // Create { value: T }
    let type_param_t = interner.intern(TypeKey::TypeParameter(TypeParamInfo {
        name: Arc::from("T"),
        constraint: None,
        default: None,
    }));
    let obj = interner.object(vec![
        PropertyInfo {
            name: Arc::from("value"),
            type_id: type_param_t,
            optional: false,
            readonly: false,
        },
    ]);

    // Substitute T = number -> { value: number }
    let mut subst = TypeSubstitution::new();
    subst.insert(Arc::from("T"), TypeId::NUMBER);
    let result = instantiate_type(&interner, obj, &subst);

    // Result should be { value: number }
    let expected = interner.object(vec![
        PropertyInfo {
            name: Arc::from("value"),
            type_id: TypeId::NUMBER,
            optional: false,
            readonly: false,
        },
    ]);
    assert_eq!(result, expected);
}

#[test]
fn test_instantiate_function() {
    let interner = TypeInterner::new();

    // Create (x: T) => T
    let type_param_t = interner.intern(TypeKey::TypeParameter(TypeParamInfo {
        name: Arc::from("T"),
        constraint: None,
        default: None,
    }));
    let func = interner.function(FunctionShape {
        type_params: vec![],
        params: vec![ParamInfo {
            name: Some(Arc::from("x")),
            type_id: type_param_t,
            optional: false,
            rest: false,
        }],
        return_type: type_param_t,
        is_constructor: false,
    });

    // Substitute T = string -> (x: string) => string
    let mut subst = TypeSubstitution::new();
    subst.insert(Arc::from("T"), TypeId::STRING);
    let result = instantiate_type(&interner, func, &subst);

    // Result should be (x: string) => string
    let expected = interner.function(FunctionShape {
        type_params: vec![],
        params: vec![ParamInfo {
            name: Some(Arc::from("x")),
            type_id: TypeId::STRING,
            optional: false,
            rest: false,
        }],
        return_type: TypeId::STRING,
        is_constructor: false,
    });
    assert_eq!(result, expected);
}

#[test]
fn test_instantiate_tuple() {
    let interner = TypeInterner::new();

    // Create [T, U]
    let type_param_t = interner.intern(TypeKey::TypeParameter(TypeParamInfo {
        name: Arc::from("T"),
        constraint: None,
        default: None,
    }));
    let type_param_u = interner.intern(TypeKey::TypeParameter(TypeParamInfo {
        name: Arc::from("U"),
        constraint: None,
        default: None,
    }));
    let tuple = interner.tuple(vec![
        TupleElement { type_id: type_param_t, name: None, optional: false, rest: false },
        TupleElement { type_id: type_param_u, name: None, optional: false, rest: false },
    ]);

    // Substitute T = string, U = number -> [string, number]
    let mut subst = TypeSubstitution::new();
    subst.insert(Arc::from("T"), TypeId::STRING);
    subst.insert(Arc::from("U"), TypeId::NUMBER);
    let result = instantiate_type(&interner, tuple, &subst);

    // Result should be [string, number]
    let expected = interner.tuple(vec![
        TupleElement { type_id: TypeId::STRING, name: None, optional: false, rest: false },
        TupleElement { type_id: TypeId::NUMBER, name: None, optional: false, rest: false },
    ]);
    assert_eq!(result, expected);
}

#[test]
fn test_instantiate_generic_convenience() {
    let interner = TypeInterner::new();

    // Create Array<T>
    let type_param_t = interner.intern(TypeKey::TypeParameter(TypeParamInfo {
        name: Arc::from("T"),
        constraint: None,
        default: None,
    }));
    let array_t = interner.array(type_param_t);

    // Use convenience function
    let type_params = vec![
        TypeParamInfo { name: Arc::from("T"), constraint: None, default: None },
    ];
    let type_args = vec![TypeId::STRING];

    let result = instantiate_generic(&interner, array_t, &type_params, &type_args);

    // Result should be Array<string>
    let expected = interner.array(TypeId::STRING);
    assert_eq!(result, expected);
}

#[test]
fn test_instantiate_nested() {
    let interner = TypeInterner::new();

    // Create Array<Array<T>>
    let type_param_t = interner.intern(TypeKey::TypeParameter(TypeParamInfo {
        name: Arc::from("T"),
        constraint: None,
        default: None,
    }));
    let inner_array = interner.array(type_param_t);
    let outer_array = interner.array(inner_array);

    // Substitute T = number -> Array<Array<number>>
    let mut subst = TypeSubstitution::new();
    subst.insert(Arc::from("T"), TypeId::NUMBER);
    let result = instantiate_type(&interner, outer_array, &subst);

    // Result should be Array<Array<number>>
    let inner_expected = interner.array(TypeId::NUMBER);
    let expected = interner.array(inner_expected);
    assert_eq!(result, expected);
}

#[test]
fn test_instantiate_intrinsics_unchanged() {
    let interner = TypeInterner::new();

    // Intrinsics should not be affected by substitution
    let mut subst = TypeSubstitution::new();
    subst.insert(Arc::from("T"), TypeId::NUMBER);

    assert_eq!(instantiate_type(&interner, TypeId::STRING, &subst), TypeId::STRING);
    assert_eq!(instantiate_type(&interner, TypeId::NUMBER, &subst), TypeId::NUMBER);
    assert_eq!(instantiate_type(&interner, TypeId::BOOLEAN, &subst), TypeId::BOOLEAN);
    assert_eq!(instantiate_type(&interner, TypeId::NULL, &subst), TypeId::NULL);
    assert_eq!(instantiate_type(&interner, TypeId::UNDEFINED, &subst), TypeId::UNDEFINED);
}

#[test]
fn test_instantiate_conditional() {
    let interner = TypeInterner::new();

    // Create T extends string ? T : never
    let type_param_t = interner.intern(TypeKey::TypeParameter(TypeParamInfo {
        name: Arc::from("T"),
        constraint: None,
        default: None,
    }));
    let cond = interner.intern(TypeKey::Conditional(Box::new(ConditionalType {
        check_type: type_param_t,
        extends_type: TypeId::STRING,
        true_type: type_param_t,
        false_type: TypeId::NEVER,
    })));

    // Substitute T = "hello" (a string literal)
    let hello_lit = interner.literal_string("hello");
    let mut subst = TypeSubstitution::new();
    subst.insert(Arc::from("T"), hello_lit);
    let result = instantiate_type(&interner, cond, &subst);

    // Result should be "hello" extends string ? "hello" : never
    let expected = interner.intern(TypeKey::Conditional(Box::new(ConditionalType {
        check_type: hello_lit,
        extends_type: TypeId::STRING,
        true_type: hello_lit,
        false_type: TypeId::NEVER,
    })));
    assert_eq!(result, expected);
}
