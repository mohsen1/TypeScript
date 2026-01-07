use super::*;
use crate::solver::subtype::SubtypeFailureReason;
use crate::solver::types::*;

fn make_animal_dog(interner: &TypeInterner) -> (TypeId, TypeId) {
    let name = interner.intern_string("name");
    let breed = interner.intern_string("breed");

    let animal = interner.object(vec![PropertyInfo {
        name,
        type_id: TypeId::STRING,
        optional: false,
        readonly: false,
        is_method: false,
    }]);

    let dog = interner.object(vec![
        PropertyInfo {
            name,
            type_id: TypeId::STRING,
            optional: false,
            readonly: false,
            is_method: false,
        },
        PropertyInfo {
            name: breed,
            type_id: TypeId::STRING,
            optional: false,
            readonly: false,
            is_method: false,
        },
    ]);

    (animal, dog)
}

#[test]
fn test_any_assignability() {
    let interner = TypeInterner::new();
    let mut checker = CompatChecker::new(&interner);

    assert!(checker.is_assignable(TypeId::ANY, TypeId::STRING));
    assert!(checker.is_assignable(TypeId::STRING, TypeId::ANY));
}

#[test]
fn test_unknown_assignability() {
    let interner = TypeInterner::new();
    let mut checker = CompatChecker::new(&interner);

    assert!(checker.is_assignable(TypeId::STRING, TypeId::UNKNOWN));
    assert!(checker.is_assignable(TypeId::UNKNOWN, TypeId::ANY));
    assert!(checker.is_assignable(TypeId::UNKNOWN, TypeId::UNKNOWN));
    assert!(!checker.is_assignable(TypeId::UNKNOWN, TypeId::STRING));
}

#[test]
fn test_function_bivariance_default() {
    let interner = TypeInterner::new();
    let mut checker = CompatChecker::new(&interner);

    let (animal, dog) = make_animal_dog(&interner);

    let fn_dog = interner.function(FunctionShape {
        params: vec![ParamInfo {
            name: None,
            type_id: dog,
            optional: false,
            rest: false,
        }],
        return_type: TypeId::VOID,
        type_params: Vec::new(),
        type_predicate: None,
        is_constructor: false,
    });

    let fn_animal = interner.function(FunctionShape {
        params: vec![ParamInfo {
            name: None,
            type_id: animal,
            optional: false,
            rest: false,
        }],
        return_type: TypeId::VOID,
        type_params: Vec::new(),
        type_predicate: None,
        is_constructor: false,
    });

    assert!(checker.is_assignable(fn_dog, fn_animal));
}

#[test]
fn test_function_variance_strict() {
    let interner = TypeInterner::new();
    let mut checker = CompatChecker::new(&interner);
    checker.set_strict_function_types(true);

    let (animal, dog) = make_animal_dog(&interner);

    let fn_dog = interner.function(FunctionShape {
        params: vec![ParamInfo {
            name: None,
            type_id: dog,
            optional: false,
            rest: false,
        }],
        return_type: TypeId::VOID,
        type_params: Vec::new(),
        type_predicate: None,
        is_constructor: false,
    });

    let fn_animal = interner.function(FunctionShape {
        params: vec![ParamInfo {
            name: None,
            type_id: animal,
            optional: false,
            rest: false,
        }],
        return_type: TypeId::VOID,
        type_params: Vec::new(),
        type_predicate: None,
        is_constructor: false,
    });

    assert!(!checker.is_assignable(fn_dog, fn_animal));
}

#[test]
fn test_method_bivariance_even_strict() {
    let interner = TypeInterner::new();
    let mut checker = CompatChecker::new(&interner);
    checker.set_strict_function_types(true);

    let name = interner.intern_string("fn");
    let string_or_number = interner.union(vec![TypeId::STRING, TypeId::NUMBER]);

    let source_fn = interner.function(FunctionShape {
        params: vec![ParamInfo {
            name: None,
            type_id: TypeId::STRING,
            optional: false,
            rest: false,
        }],
        return_type: TypeId::VOID,
        type_params: Vec::new(),
        type_predicate: None,
        is_constructor: false,
    });

    let target_fn = interner.function(FunctionShape {
        params: vec![ParamInfo {
            name: None,
            type_id: string_or_number,
            optional: false,
            rest: false,
        }],
        return_type: TypeId::VOID,
        type_params: Vec::new(),
        type_predicate: None,
        is_constructor: false,
    });

    let source = interner.object(vec![PropertyInfo {
        name,
        type_id: source_fn,
        optional: false,
        readonly: false,
        is_method: true,
    }]);

    let target = interner.object(vec![PropertyInfo {
        name,
        type_id: target_fn,
        optional: false,
        readonly: false,
        is_method: true,
    }]);

    assert!(checker.is_assignable(source, target));
}

#[test]
fn test_function_property_stays_strict() {
    let interner = TypeInterner::new();
    let mut checker = CompatChecker::new(&interner);
    checker.set_strict_function_types(true);

    let name = interner.intern_string("fn");
    let string_or_number = interner.union(vec![TypeId::STRING, TypeId::NUMBER]);

    let source_fn = interner.function(FunctionShape {
        params: vec![ParamInfo {
            name: None,
            type_id: TypeId::STRING,
            optional: false,
            rest: false,
        }],
        return_type: TypeId::VOID,
        type_params: Vec::new(),
        type_predicate: None,
        is_constructor: false,
    });

    let target_fn = interner.function(FunctionShape {
        params: vec![ParamInfo {
            name: None,
            type_id: string_or_number,
            optional: false,
            rest: false,
        }],
        return_type: TypeId::VOID,
        type_params: Vec::new(),
        type_predicate: None,
        is_constructor: false,
    });

    let source = interner.object(vec![PropertyInfo {
        name,
        type_id: source_fn,
        optional: false,
        readonly: false,
        is_method: false,
    }]);

    let target = interner.object(vec![PropertyInfo {
        name,
        type_id: target_fn,
        optional: false,
        readonly: false,
        is_method: false,
    }]);

    assert!(!checker.is_assignable(source, target));
}

#[test]
fn test_function_return_covariance() {
    let interner = TypeInterner::new();
    let mut checker = CompatChecker::new(&interner);

    let (animal, dog) = make_animal_dog(&interner);

    let returns_dog = interner.function(FunctionShape {
        params: Vec::new(),
        return_type: dog,
        type_params: Vec::new(),
        type_predicate: None,
        is_constructor: false,
    });

    let returns_animal = interner.function(FunctionShape {
        params: Vec::new(),
        return_type: animal,
        type_params: Vec::new(),
        type_predicate: None,
        is_constructor: false,
    });

    assert!(checker.is_assignable(returns_dog, returns_animal));
    assert!(!checker.is_assignable(returns_animal, returns_dog));
}

#[test]
fn test_void_return_assignability() {
    let interner = TypeInterner::new();
    let mut checker = CompatChecker::new(&interner);

    let returns_number = interner.function(FunctionShape {
        params: Vec::new(),
        return_type: TypeId::NUMBER,
        type_params: Vec::new(),
        type_predicate: None,
        is_constructor: false,
    });

    let returns_void = interner.function(FunctionShape {
        params: Vec::new(),
        return_type: TypeId::VOID,
        type_params: Vec::new(),
        type_predicate: None,
        is_constructor: false,
    });

    assert!(checker.is_assignable(returns_number, returns_void));
    assert!(!checker.is_assignable(returns_void, returns_number));
}

#[test]
fn test_explain_failure_missing_property() {
    let interner = TypeInterner::new();
    let mut checker = CompatChecker::new(&interner);

    let (animal, dog) = make_animal_dog(&interner);
    let breed_name = interner.intern_string("breed");

    let reason = checker.explain_failure(animal, dog);
    assert!(
        matches!(reason, Some(SubtypeFailureReason::MissingProperty { property_name, .. })
            if property_name == breed_name)
    );
}

#[test]
fn test_explain_failure_parameter_mismatch_strict() {
    let interner = TypeInterner::new();
    let mut checker = CompatChecker::new(&interner);
    checker.set_strict_function_types(true);

    let (animal, dog) = make_animal_dog(&interner);

    let fn_dog = interner.function(FunctionShape {
        params: vec![ParamInfo {
            name: None,
            type_id: dog,
            optional: false,
            rest: false,
        }],
        return_type: TypeId::VOID,
        type_params: Vec::new(),
        type_predicate: None,
        is_constructor: false,
    });

    let fn_animal = interner.function(FunctionShape {
        params: vec![ParamInfo {
            name: None,
            type_id: animal,
            optional: false,
            rest: false,
        }],
        return_type: TypeId::VOID,
        type_params: Vec::new(),
        type_predicate: None,
        is_constructor: false,
    });

    let reason = checker.explain_failure(fn_dog, fn_animal);
    assert!(matches!(
        reason,
        Some(SubtypeFailureReason::ParameterTypeMismatch { param_index: 0, .. })
    ));
}

#[test]
fn test_weak_type_rejects_no_common_properties() {
    let interner = TypeInterner::new();
    let mut checker = CompatChecker::new(&interner);

    let a = interner.intern_string("a");
    let b = interner.intern_string("b");

    let weak_target = interner.object(vec![PropertyInfo {
        name: a,
        type_id: TypeId::NUMBER,
        optional: true,
        readonly: false,
        is_method: false,
    }]);

    let source = interner.object(vec![PropertyInfo {
        name: b,
        type_id: TypeId::NUMBER,
        optional: false,
        readonly: false,
        is_method: false,
    }]);

    assert!(!checker.is_assignable(source, weak_target));
    assert!(matches!(
        checker.explain_failure(source, weak_target),
        Some(SubtypeFailureReason::NoCommonProperties { .. })
    ));
}

#[test]
fn test_weak_type_allows_overlap() {
    let interner = TypeInterner::new();
    let mut checker = CompatChecker::new(&interner);

    let a = interner.intern_string("a");

    let weak_target = interner.object(vec![PropertyInfo {
        name: a,
        type_id: TypeId::NUMBER,
        optional: true,
        readonly: false,
        is_method: false,
    }]);

    let source = interner.object(vec![PropertyInfo {
        name: a,
        type_id: TypeId::NUMBER,
        optional: false,
        readonly: false,
        is_method: false,
    }]);

    assert!(checker.is_assignable(source, weak_target));
}

#[test]
fn test_weak_type_skips_empty_target() {
    let interner = TypeInterner::new();
    let mut checker = CompatChecker::new(&interner);

    let a = interner.intern_string("a");

    let empty_target = interner.object(Vec::new());
    let source = interner.object(vec![PropertyInfo {
        name: a,
        type_id: TypeId::NUMBER,
        optional: false,
        readonly: false,
        is_method: false,
    }]);

    assert!(checker.is_assignable(source, empty_target));
}

#[test]
fn test_rest_any_bivariant_even_strict() {
    let interner = TypeInterner::new();
    let mut checker = CompatChecker::new(&interner);
    checker.set_strict_function_types(true);

    let rest_any = interner.array(TypeId::ANY);
    let target = interner.function(FunctionShape {
        params: vec![ParamInfo {
            name: None,
            type_id: rest_any,
            optional: false,
            rest: true,
        }],
        return_type: TypeId::VOID,
        type_params: Vec::new(),
        type_predicate: None,
        is_constructor: false,
    });

    let source = interner.function(FunctionShape {
        params: vec![ParamInfo {
            name: None,
            type_id: TypeId::NUMBER,
            optional: false,
            rest: false,
        }],
        return_type: TypeId::VOID,
        type_params: Vec::new(),
        type_predicate: None,
        is_constructor: false,
    });

    assert!(checker.is_assignable(source, target));
}

#[test]
fn test_rest_unknown_bivariant_even_strict() {
    let interner = TypeInterner::new();
    let mut checker = CompatChecker::new(&interner);
    checker.set_strict_function_types(true);

    let rest_unknown = interner.array(TypeId::UNKNOWN);
    let target = interner.function(FunctionShape {
        params: vec![ParamInfo {
            name: None,
            type_id: rest_unknown,
            optional: false,
            rest: true,
        }],
        return_type: TypeId::VOID,
        type_params: Vec::new(),
        type_predicate: None,
        is_constructor: false,
    });

    let source = interner.function(FunctionShape {
        params: vec![ParamInfo {
            name: None,
            type_id: TypeId::STRING,
            optional: false,
            rest: false,
        }],
        return_type: TypeId::VOID,
        type_params: Vec::new(),
        type_predicate: None,
        is_constructor: false,
    });

    assert!(checker.is_assignable(source, target));
}

#[test]
fn test_rest_any_still_checks_return_type() {
    let interner = TypeInterner::new();
    let mut checker = CompatChecker::new(&interner);

    let rest_any = interner.array(TypeId::ANY);
    let target = interner.function(FunctionShape {
        params: vec![ParamInfo {
            name: None,
            type_id: rest_any,
            optional: false,
            rest: true,
        }],
        return_type: TypeId::NUMBER,
        type_params: Vec::new(),
        type_predicate: None,
        is_constructor: false,
    });

    let source = interner.function(FunctionShape {
        params: vec![ParamInfo {
            name: None,
            type_id: TypeId::STRING,
            optional: false,
            rest: false,
        }],
        return_type: TypeId::STRING,
        type_params: Vec::new(),
        type_predicate: None,
        is_constructor: false,
    });

    assert!(!checker.is_assignable(source, target));
}

#[test]
fn test_explain_failure_skips_rest_unknown() {
    let interner = TypeInterner::new();
    let mut checker = CompatChecker::new(&interner);
    checker.set_strict_function_types(true);

    let rest_unknown = interner.array(TypeId::UNKNOWN);
    let target = interner.function(FunctionShape {
        params: vec![ParamInfo {
            name: None,
            type_id: rest_unknown,
            optional: false,
            rest: true,
        }],
        return_type: TypeId::VOID,
        type_params: Vec::new(),
        type_predicate: None,
        is_constructor: false,
    });

    let source = interner.function(FunctionShape {
        params: vec![
            ParamInfo {
                name: None,
                type_id: TypeId::NUMBER,
                optional: false,
                rest: false,
            },
            ParamInfo {
                name: None,
                type_id: TypeId::STRING,
                optional: false,
                rest: false,
            },
        ],
        return_type: TypeId::VOID,
        type_params: Vec::new(),
        type_predicate: None,
        is_constructor: false,
    });

    assert!(checker.explain_failure(source, target).is_none());
}

#[test]
fn test_explain_failure_reports_rest_mismatch() {
    let interner = TypeInterner::new();
    let mut checker = CompatChecker::new(&interner);

    let rest_number = interner.array(TypeId::NUMBER);
    let target = interner.function(FunctionShape {
        params: vec![ParamInfo {
            name: None,
            type_id: rest_number,
            optional: false,
            rest: true,
        }],
        return_type: TypeId::VOID,
        type_params: Vec::new(),
        type_predicate: None,
        is_constructor: false,
    });

    let source = interner.function(FunctionShape {
        params: vec![
            ParamInfo {
                name: None,
                type_id: TypeId::NUMBER,
                optional: false,
                rest: false,
            },
            ParamInfo {
                name: None,
                type_id: TypeId::STRING,
                optional: false,
                rest: false,
            },
        ],
        return_type: TypeId::VOID,
        type_params: Vec::new(),
        type_predicate: None,
        is_constructor: false,
    });

    assert!(matches!(
        checker.explain_failure(source, target),
        Some(SubtypeFailureReason::ParameterTypeMismatch { .. })
    ));
}

#[test]
fn test_empty_object_accepts_non_nullish() {
    let interner = TypeInterner::new();
    let mut checker = CompatChecker::new(&interner);

    let empty_object = interner.object(Vec::new());

    assert!(checker.is_assignable(TypeId::STRING, empty_object));
    assert!(checker.is_assignable(TypeId::NUMBER, empty_object));

    let array = interner.array(TypeId::NUMBER);
    assert!(checker.is_assignable(array, empty_object));

    let func = interner.function(FunctionShape {
        params: Vec::new(),
        return_type: TypeId::VOID,
        type_params: Vec::new(),
        type_predicate: None,
        is_constructor: false,
    });
    assert!(checker.is_assignable(func, empty_object));
}

#[test]
fn test_empty_object_rejects_nullish_and_unknown() {
    let interner = TypeInterner::new();
    let mut checker = CompatChecker::new(&interner);

    let empty_object = interner.object(Vec::new());

    assert!(!checker.is_assignable(TypeId::NULL, empty_object));
    assert!(!checker.is_assignable(TypeId::UNDEFINED, empty_object));
    assert!(!checker.is_assignable(TypeId::VOID, empty_object));
    assert!(!checker.is_assignable(TypeId::UNKNOWN, empty_object));
}

#[test]
fn test_strict_null_checks_toggle() {
    let interner = TypeInterner::new();
    let mut checker = CompatChecker::new(&interner);

    let empty_object = interner.object(Vec::new());
    let nullable_string = interner.union(vec![TypeId::STRING, TypeId::NULL]);

    assert!(!checker.is_assignable(TypeId::NULL, TypeId::STRING));
    assert!(!checker.is_assignable(nullable_string, TypeId::STRING));
    assert!(!checker.is_assignable(nullable_string, empty_object));

    checker.set_strict_null_checks(false);

    assert!(checker.is_assignable(TypeId::NULL, TypeId::STRING));
    assert!(checker.is_assignable(TypeId::UNDEFINED, TypeId::NUMBER));
    assert!(checker.is_assignable(nullable_string, TypeId::STRING));
    assert!(checker.is_assignable(TypeId::UNDEFINED, empty_object));
    assert!(checker.is_assignable(nullable_string, empty_object));
}

#[test]
fn test_no_unchecked_indexed_access_toggle() {
    let interner = TypeInterner::new();
    let mut checker = CompatChecker::new(&interner);

    let indexed = interner.object_with_index(ObjectShape {
        properties: Vec::new(),
        string_index: Some(IndexSignature {
            key_type: TypeId::STRING,
            value_type: TypeId::NUMBER,
            readonly: false,
        }),
        number_index: None,
    });

    let index_access = interner.intern(TypeKey::IndexAccess(indexed, TypeId::STRING));
    let number_or_undefined = interner.union(vec![TypeId::NUMBER, TypeId::UNDEFINED]);

    assert!(checker.is_assignable(index_access, TypeId::NUMBER));

    checker.set_no_unchecked_indexed_access(true);

    assert!(!checker.is_assignable(index_access, TypeId::NUMBER));
    assert!(checker.is_assignable(index_access, number_or_undefined));
}

#[test]
fn test_object_keyword_accepts_non_primitives() {
    let interner = TypeInterner::new();
    let mut checker = CompatChecker::new(&interner);

    let name = interner.intern_string("name");
    let obj = interner.object(vec![PropertyInfo {
        name,
        type_id: TypeId::STRING,
        optional: false,
        readonly: false,
        is_method: false,
    }]);
    assert!(checker.is_assignable(obj, TypeId::OBJECT));

    let array = interner.array(TypeId::NUMBER);
    assert!(checker.is_assignable(array, TypeId::OBJECT));

    let tuple = interner.tuple(vec![TupleElement {
        type_id: TypeId::NUMBER,
        name: None,
        optional: false,
        rest: false,
    }]);
    assert!(checker.is_assignable(tuple, TypeId::OBJECT));

    let func = interner.function(FunctionShape {
        params: Vec::new(),
        return_type: TypeId::VOID,
        type_params: Vec::new(),
        type_predicate: None,
        is_constructor: false,
    });
    assert!(checker.is_assignable(func, TypeId::OBJECT));
}

#[test]
fn test_object_keyword_rejects_primitives() {
    let interner = TypeInterner::new();
    let mut checker = CompatChecker::new(&interner);

    assert!(!checker.is_assignable(TypeId::STRING, TypeId::OBJECT));
    assert!(!checker.is_assignable(TypeId::NUMBER, TypeId::OBJECT));
}

#[test]
fn test_apparent_string_members_assignable() {
    let interner = TypeInterner::new();
    let mut checker = CompatChecker::new(&interner);

    let length = interner.intern_string("length");
    let to_upper = interner.intern_string("toUpperCase");
    let to_upper_type = interner.function(FunctionShape {
        params: Vec::new(),
        return_type: TypeId::STRING,
        type_params: Vec::new(),
        type_predicate: None,
        is_constructor: false,
    });

    let target = interner.object(vec![
        PropertyInfo {
            name: length,
            type_id: TypeId::NUMBER,
            optional: false,
            readonly: false,
            is_method: false,
        },
        PropertyInfo {
            name: to_upper,
            type_id: to_upper_type,
            optional: false,
            readonly: false,
            is_method: true,
        },
    ]);

    assert!(checker.is_assignable(TypeId::STRING, target));

    let literal = interner.literal_string("hello");
    assert!(checker.is_assignable(literal, target));
}

#[test]
fn test_apparent_string_members_reject_mismatch() {
    let interner = TypeInterner::new();
    let mut checker = CompatChecker::new(&interner);

    let length = interner.intern_string("length");
    let target = interner.object(vec![PropertyInfo {
        name: length,
        type_id: TypeId::STRING,
        optional: false,
        readonly: false,
        is_method: false,
    }]);

    assert!(!checker.is_assignable(TypeId::STRING, target));
}

#[test]
fn test_apparent_number_method_assignable() {
    let interner = TypeInterner::new();
    let mut checker = CompatChecker::new(&interner);

    let to_fixed = interner.intern_string("toFixed");
    let to_fixed_type = interner.function(FunctionShape {
        params: Vec::new(),
        return_type: TypeId::STRING,
        type_params: Vec::new(),
        type_predicate: None,
        is_constructor: false,
    });

    let target = interner.object(vec![PropertyInfo {
        name: to_fixed,
        type_id: to_fixed_type,
        optional: false,
        readonly: false,
        is_method: true,
    }]);

    assert!(checker.is_assignable(TypeId::NUMBER, target));
}

#[test]
fn test_apparent_boolean_members_assignable() {
    let interner = TypeInterner::new();
    let mut checker = CompatChecker::new(&interner);

    let to_string = interner.intern_string("toString");
    let to_string_type = interner.function(FunctionShape {
        params: Vec::new(),
        return_type: TypeId::STRING,
        type_params: Vec::new(),
        type_predicate: None,
        is_constructor: false,
    });

    let target = interner.object(vec![PropertyInfo {
        name: to_string,
        type_id: to_string_type,
        optional: false,
        readonly: false,
        is_method: true,
    }]);

    assert!(checker.is_assignable(TypeId::BOOLEAN, target));
}

#[test]
fn test_apparent_bigint_members_assignable() {
    let interner = TypeInterner::new();
    let mut checker = CompatChecker::new(&interner);

    let value_of = interner.intern_string("valueOf");
    let value_of_type = interner.function(FunctionShape {
        params: Vec::new(),
        return_type: TypeId::BIGINT,
        type_params: Vec::new(),
        type_predicate: None,
        is_constructor: false,
    });

    let target = interner.object(vec![PropertyInfo {
        name: value_of,
        type_id: value_of_type,
        optional: false,
        readonly: false,
        is_method: true,
    }]);

    assert!(checker.is_assignable(TypeId::BIGINT, target));
}

#[test]
fn test_apparent_symbol_members_assignable() {
    let interner = TypeInterner::new();
    let mut checker = CompatChecker::new(&interner);

    let description = interner.intern_string("description");
    let to_string = interner.intern_string("toString");
    let description_type = interner.union(vec![TypeId::STRING, TypeId::UNDEFINED]);
    let to_string_type = interner.function(FunctionShape {
        params: Vec::new(),
        return_type: TypeId::STRING,
        type_params: Vec::new(),
        type_predicate: None,
        is_constructor: false,
    });

    let target = interner.object(vec![
        PropertyInfo {
            name: description,
            type_id: description_type,
            optional: false,
            readonly: false,
            is_method: false,
        },
        PropertyInfo {
            name: to_string,
            type_id: to_string_type,
            optional: false,
            readonly: false,
            is_method: true,
        },
    ]);

    assert!(checker.is_assignable(TypeId::SYMBOL, target));
}

#[test]
fn test_apparent_string_number_index_assignable() {
    let interner = TypeInterner::new();
    let mut checker = CompatChecker::new(&interner);

    let target = interner.object_with_index(ObjectShape {
        properties: Vec::new(),
        string_index: None,
        number_index: Some(IndexSignature {
            key_type: TypeId::NUMBER,
            value_type: TypeId::STRING,
            readonly: false,
        }),
    });

    assert!(checker.is_assignable(TypeId::STRING, target));
}

#[test]
fn test_apparent_string_rejects_string_index_signature() {
    let interner = TypeInterner::new();
    let mut checker = CompatChecker::new(&interner);

    let target = interner.object_with_index(ObjectShape {
        properties: Vec::new(),
        string_index: Some(IndexSignature {
            key_type: TypeId::STRING,
            value_type: TypeId::STRING,
            readonly: false,
        }),
        number_index: None,
    });

    assert!(!checker.is_assignable(TypeId::STRING, target));
}

#[test]
fn test_optional_property_allows_undefined() {
    let interner = TypeInterner::new();
    let mut checker = CompatChecker::new(&interner);

    let name = interner.intern_string("x");
    let source = interner.object(vec![PropertyInfo {
        name,
        type_id: TypeId::UNDEFINED,
        optional: true,
        readonly: false,
        is_method: false,
    }]);
    let target = interner.object(vec![PropertyInfo {
        name,
        type_id: TypeId::NUMBER,
        optional: true,
        readonly: false,
        is_method: false,
    }]);

    assert!(checker.is_assignable(source, target));
}

#[test]
fn test_optional_property_rejects_required_target() {
    let interner = TypeInterner::new();
    let mut checker = CompatChecker::new(&interner);

    let name = interner.intern_string("x");
    let source = interner.object(vec![PropertyInfo {
        name,
        type_id: TypeId::NUMBER,
        optional: true,
        readonly: false,
        is_method: false,
    }]);
    let target = interner.object(vec![PropertyInfo {
        name,
        type_id: TypeId::NUMBER,
        optional: false,
        readonly: false,
        is_method: false,
    }]);

    assert!(!checker.is_assignable(source, target));
}

#[test]
fn test_optional_property_rejects_string_index_signature() {
    let interner = TypeInterner::new();
    let mut checker = CompatChecker::new(&interner);

    let name = interner.intern_string("x");
    let source = interner.object(vec![PropertyInfo {
        name,
        type_id: TypeId::NUMBER,
        optional: true,
        readonly: false,
        is_method: false,
    }]);

    let target = interner.object_with_index(ObjectShape {
        properties: Vec::new(),
        string_index: Some(IndexSignature {
            key_type: TypeId::STRING,
            value_type: TypeId::NUMBER,
            readonly: false,
        }),
        number_index: None,
    });

    assert!(!checker.is_assignable(source, target));
}

#[test]
fn test_exact_optional_property_rejects_undefined() {
    let interner = TypeInterner::new();
    let mut checker = CompatChecker::new(&interner);
    checker.set_exact_optional_property_types(true);

    let name = interner.intern_string("x");
    let source = interner.object(vec![PropertyInfo {
        name,
        type_id: TypeId::UNDEFINED,
        optional: true,
        readonly: false,
        is_method: false,
    }]);
    let target = interner.object(vec![PropertyInfo {
        name,
        type_id: TypeId::NUMBER,
        optional: true,
        readonly: false,
        is_method: false,
    }]);

    assert!(!checker.is_assignable(source, target));
}

#[test]
fn test_exact_optional_property_allows_string_index_signature() {
    let interner = TypeInterner::new();
    let mut checker = CompatChecker::new(&interner);
    checker.set_exact_optional_property_types(true);

    let name = interner.intern_string("x");
    let source = interner.object(vec![PropertyInfo {
        name,
        type_id: TypeId::NUMBER,
        optional: true,
        readonly: false,
        is_method: false,
    }]);

    let target = interner.object_with_index(ObjectShape {
        properties: Vec::new(),
        string_index: Some(IndexSignature {
            key_type: TypeId::STRING,
            value_type: TypeId::NUMBER,
            readonly: false,
        }),
        number_index: None,
    });

    assert!(checker.is_assignable(source, target));
}

#[test]
fn test_rest_any_callable_target_from_function() {
    let interner = TypeInterner::new();
    let mut checker = CompatChecker::new(&interner);
    checker.set_strict_function_types(true);

    let rest_any = interner.array(TypeId::ANY);
    let target = interner.callable(CallableShape {
        call_signatures: vec![CallSignature {
            params: vec![ParamInfo {
                name: None,
                type_id: rest_any,
                optional: false,
                rest: true,
            }],
            return_type: TypeId::VOID,
            type_predicate: None,
            type_params: Vec::new(),
        }],
        construct_signatures: Vec::new(),
        properties: Vec::new(),
    });

    let source = interner.function(FunctionShape {
        params: vec![ParamInfo {
            name: None,
            type_id: TypeId::NUMBER,
            optional: false,
            rest: false,
        }],
        return_type: TypeId::VOID,
        type_params: Vec::new(),
        type_predicate: None,
        is_constructor: false,
    });

    assert!(checker.is_assignable(source, target));
}

#[test]
fn test_rest_unknown_callable_target_from_callable() {
    let interner = TypeInterner::new();
    let mut checker = CompatChecker::new(&interner);
    checker.set_strict_function_types(true);

    let rest_unknown = interner.array(TypeId::UNKNOWN);
    let target = interner.callable(CallableShape {
        call_signatures: vec![CallSignature {
            params: vec![ParamInfo {
                name: None,
                type_id: rest_unknown,
                optional: false,
                rest: true,
            }],
            return_type: TypeId::VOID,
            type_predicate: None,
            type_params: Vec::new(),
        }],
        construct_signatures: Vec::new(),
        properties: Vec::new(),
    });

    let source = interner.callable(CallableShape {
        call_signatures: vec![CallSignature {
            params: vec![ParamInfo {
                name: None,
                type_id: TypeId::STRING,
                optional: false,
                rest: false,
            }],
            return_type: TypeId::VOID,
            type_predicate: None,
            type_params: Vec::new(),
        }],
        construct_signatures: Vec::new(),
        properties: Vec::new(),
    });

    assert!(checker.is_assignable(source, target));
}
