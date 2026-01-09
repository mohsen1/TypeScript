use super::*;

// =============================================================================
// Discriminant Detection Tests
// =============================================================================

#[test]
fn test_find_discriminants_basic() {
    let interner = TypeInterner::new();
    let type_name = interner.intern_string("type");

    // type Action = { type: "add" } | { type: "remove" }
    let type_add = interner.literal_string("add");
    let type_remove = interner.literal_string("remove");

    let member1 = interner.object(vec![
        PropertyInfo { name: type_name, type_id: type_add,
 write_type: type_add, optional: false, readonly: false, is_method: false },
    ]);
    let member2 = interner.object(vec![
        PropertyInfo { name: type_name, type_id: type_remove,
 write_type: type_remove, optional: false, readonly: false, is_method: false },
    ]);

    let union = interner.union(vec![member1, member2]);

    let discriminants = find_discriminants(&interner, union);

    assert_eq!(discriminants.len(), 1);
    assert_eq!(discriminants[0].property_name, type_name);
    assert_eq!(discriminants[0].variants.len(), 2);
}

#[test]
fn test_find_discriminants_multiple_props() {
    let interner = TypeInterner::new();
    let kind_name = interner.intern_string("kind");
    let type_name = interner.intern_string("type");

    // type Action = { kind: "a", type: 1 } | { kind: "b", type: 2 }
    let kind_a = interner.literal_string("a");
    let kind_b = interner.literal_string("b");
    let type_1 = interner.literal_number(1.0);
    let type_2 = interner.literal_number(2.0);

    let member1 = interner.object(vec![
        PropertyInfo { name: kind_name, type_id: kind_a,
 write_type: kind_a, optional: false, readonly: false, is_method: false },
        PropertyInfo { name: type_name, type_id: type_1,
 write_type: type_1, optional: false, readonly: false, is_method: false },
    ]);
    let member2 = interner.object(vec![
        PropertyInfo { name: kind_name, type_id: kind_b,
 write_type: kind_b, optional: false, readonly: false, is_method: false },
        PropertyInfo { name: type_name, type_id: type_2,
 write_type: type_2, optional: false, readonly: false, is_method: false },
    ]);

    let union = interner.union(vec![member1, member2]);

    let discriminants = find_discriminants(&interner, union);

    // Both "kind" and "type" are discriminants
    assert_eq!(discriminants.len(), 2);
}

#[test]
fn test_find_discriminants_non_literal() {
    let interner = TypeInterner::new();
    let type_name = interner.intern_string("type");

    // type T = { type: string } | { type: string }
    // Not a discriminated union - type is not literal
    let member1 = interner.object(vec![
        PropertyInfo { name: type_name, type_id: TypeId::STRING,
 write_type: TypeId::STRING, optional: false, readonly: false, is_method: false },
    ]);
    let member2 = interner.object(vec![
        PropertyInfo { name: type_name, type_id: TypeId::STRING,
 write_type: TypeId::STRING, optional: false, readonly: false, is_method: false },
    ]);

    let union = interner.union(vec![member1, member2]);

    let discriminants = find_discriminants(&interner, union);

    // No discriminants - not literal types
    assert_eq!(discriminants.len(), 0);
}

#[test]
fn test_find_discriminants_missing_property() {
    let interner = TypeInterner::new();
    let type_name = interner.intern_string("type");
    let kind_name = interner.intern_string("kind");

    // type T = { type: "a" } | { kind: "b" }
    // Not a discriminated union - no common property
    let type_a = interner.literal_string("a");
    let kind_b = interner.literal_string("b");

    let member1 = interner.object(vec![
        PropertyInfo { name: type_name, type_id: type_a,
 write_type: type_a, optional: false, readonly: false, is_method: false },
    ]);
    let member2 = interner.object(vec![
        PropertyInfo { name: kind_name, type_id: kind_b,
 write_type: kind_b, optional: false, readonly: false, is_method: false },
    ]);

    let union = interner.union(vec![member1, member2]);

    let discriminants = find_discriminants(&interner, union);

    assert_eq!(discriminants.len(), 0);
}

// =============================================================================
// Narrowing by Discriminant Tests
// =============================================================================

#[test]
fn test_narrow_by_discriminant() {
    let interner = TypeInterner::new();
    let type_name = interner.intern_string("type");

    // type Action = { type: "add", value: number } | { type: "remove", id: string }
    let type_add = interner.literal_string("add");
    let type_remove = interner.literal_string("remove");

    let member_add = interner.object(vec![
        PropertyInfo { name: type_name, type_id: type_add,
 write_type: type_add, optional: false, readonly: false, is_method: false },
        PropertyInfo { name: interner.intern_string("value"), type_id: TypeId::NUMBER,
 write_type: TypeId::NUMBER, optional: false, readonly: false, is_method: false },
    ]);
    let member_remove = interner.object(vec![
        PropertyInfo { name: type_name, type_id: type_remove,
 write_type: type_remove, optional: false, readonly: false, is_method: false },
        PropertyInfo { name: interner.intern_string("id"), type_id: TypeId::STRING,
 write_type: TypeId::STRING, optional: false, readonly: false, is_method: false },
    ]);

    let union = interner.union(vec![member_add, member_remove]);

    // Narrow to "add" variant
    let narrowed = narrow_by_discriminant(&interner, union, type_name, type_add);
    assert_eq!(narrowed, member_add);

    // Narrow to "remove" variant
    let narrowed = narrow_by_discriminant(&interner, union, type_name, type_remove);
    assert_eq!(narrowed, member_remove);
}

#[test]
fn test_narrow_by_discriminant_no_match() {
    let interner = TypeInterner::new();
    let type_name = interner.intern_string("type");

    let type_add = interner.literal_string("add");
    let type_unknown = interner.literal_string("unknown");

    let member = interner.object(vec![
        PropertyInfo { name: type_name, type_id: type_add,
 write_type: type_add, optional: false, readonly: false, is_method: false },
    ]);

    let union = interner.union(vec![member]);

    // Narrow to non-existent variant - returns original
    let narrowed = narrow_by_discriminant(&interner, union, type_name, type_unknown);
    assert_eq!(narrowed, union);
}

#[test]
fn test_narrow_excluding_discriminant() {
    let interner = TypeInterner::new();
    let type_name = interner.intern_string("type");

    // type Action = { type: "a" } | { type: "b" } | { type: "c" }
    let type_a = interner.literal_string("a");
    let type_b = interner.literal_string("b");
    let type_c = interner.literal_string("c");

    let member_a = interner.object(vec![
        PropertyInfo { name: type_name, type_id: type_a,
 write_type: type_a, optional: false, readonly: false, is_method: false },
    ]);
    let member_b = interner.object(vec![
        PropertyInfo { name: type_name, type_id: type_b,
 write_type: type_b, optional: false, readonly: false, is_method: false },
    ]);
    let member_c = interner.object(vec![
        PropertyInfo { name: type_name, type_id: type_c,
 write_type: type_c, optional: false, readonly: false, is_method: false },
    ]);

    let union = interner.union(vec![member_a, member_b, member_c]);

    let ctx = NarrowingContext::new(&interner);

    // Exclude "a" - should get "b" | "c"
    let narrowed = ctx.narrow_by_excluding_discriminant(union, type_name, type_a);
    let expected = interner.union(vec![member_b, member_c]);
    assert_eq!(narrowed, expected);
}

// =============================================================================
// Typeof Narrowing Tests
// =============================================================================

#[test]
fn test_narrow_by_typeof_string() {
    let interner = TypeInterner::new();

    // string | number narrowed by typeof "string" -> string
    let union = interner.union(vec![TypeId::STRING, TypeId::NUMBER]);

    let narrowed = narrow_by_typeof(&interner, union, "string");
    assert_eq!(narrowed, TypeId::STRING);
}

#[test]
fn test_narrow_by_typeof_number() {
    let interner = TypeInterner::new();

    // string | number narrowed by typeof "number" -> number
    let union = interner.union(vec![TypeId::STRING, TypeId::NUMBER]);

    let narrowed = narrow_by_typeof(&interner, union, "number");
    assert_eq!(narrowed, TypeId::NUMBER);
}

#[test]
fn test_narrow_by_typeof_no_match() {
    let interner = TypeInterner::new();

    // string narrowed by typeof "number" -> never
    let narrowed = narrow_by_typeof(&interner, TypeId::STRING, "number");
    assert_eq!(narrowed, TypeId::NEVER);
}

#[test]
fn test_narrow_by_typeof_literal() {
    let interner = TypeInterner::new();

    // "hello" | 42 narrowed by typeof "string" -> "hello"
    let hello = interner.literal_string("hello");
    let forty_two = interner.literal_number(42.0);
    let union = interner.union(vec![hello, forty_two]);

    let narrowed = narrow_by_typeof(&interner, union, "string");
    assert_eq!(narrowed, hello);
}

#[test]
fn test_narrow_by_typeof_template_literal() {
    let interner = TypeInterner::new();

    let template = interner.template_literal(vec![
        TemplateSpan::Text(interner.intern_string("prefix")),
        TemplateSpan::Type(TypeId::STRING),
        TemplateSpan::Text(interner.intern_string("suffix")),
    ]);
    let union = interner.union(vec![template, TypeId::NUMBER]);

    let narrowed = narrow_by_typeof(&interner, union, "string");
    assert_eq!(narrowed, template);
}

#[test]
fn test_narrow_by_typeof_any() {
    let interner = TypeInterner::new();

    let narrowed = narrow_by_typeof(&interner, TypeId::ANY, "string");
    assert_eq!(narrowed, TypeId::ANY);
}

#[test]
fn test_narrow_by_typeof_unknown_string() {
    let interner = TypeInterner::new();

    let narrowed = narrow_by_typeof(&interner, TypeId::UNKNOWN, "string");
    assert_eq!(narrowed, TypeId::STRING);
}

#[test]
fn test_narrow_by_typeof_unknown_object() {
    let interner = TypeInterner::new();

    let narrowed = narrow_by_typeof(&interner, TypeId::UNKNOWN, "object");
    let expected = interner.union(vec![TypeId::OBJECT, TypeId::NULL]);
    assert_eq!(narrowed, expected);
}

#[test]
fn test_narrow_by_typeof_unknown_function() {
    let interner = TypeInterner::new();
    let ctx = NarrowingContext::new(&interner);

    let narrowed = narrow_by_typeof(&interner, TypeId::UNKNOWN, "function");
    assert_eq!(narrowed, ctx.function_type());
}

#[test]
fn test_narrow_by_typeof_object_function() {
    let interner = TypeInterner::new();
    let ctx = NarrowingContext::new(&interner);

    let narrowed = narrow_by_typeof(&interner, TypeId::OBJECT, "function");
    assert_eq!(narrowed, ctx.function_type());
}

#[test]
fn test_narrow_by_typeof_empty_object_function() {
    let interner = TypeInterner::new();
    let ctx = NarrowingContext::new(&interner);

    let empty_object = interner.object(vec![]);
    let narrowed = narrow_by_typeof(&interner, empty_object, "function");
    assert_eq!(narrowed, ctx.function_type());
}

#[test]
fn test_narrow_by_typeof_negation_function() {
    let interner = TypeInterner::new();
    let ctx = NarrowingContext::new(&interner);

    let func = interner.function(FunctionShape {
        params: vec![ParamInfo {
            name: Some(interner.intern_string("x")),
            type_id: TypeId::NUMBER,
            optional: false,
            rest: false,
        }],
        this_type: None,
        return_type: TypeId::VOID,
        type_params: Vec::new(),
        type_predicate: None,
        is_constructor: false,
    });
    let obj = interner.object(vec![PropertyInfo {
        name: interner.intern_string("value"),
        type_id: TypeId::NUMBER,
        write_type: TypeId::NUMBER,
        optional: false,
        readonly: false,
        is_method: false,
    }]);
    let union = interner.union(vec![func, obj]);

    let narrowed = ctx.narrow_excluding_function(union);
    assert_eq!(narrowed, obj);
}

#[test]
fn test_narrow_by_typeof_negation_function_branded_intersection() {
    let interner = TypeInterner::new();
    let ctx = NarrowingContext::new(&interner);

    let brand = interner.object(vec![PropertyInfo {
        name: interner.intern_string("__brand"),
        type_id: interner.literal_string("Tagged"),
        write_type: interner.literal_string("Tagged"),
        optional: false,
        readonly: false,
        is_method: false,
    }]);
    let func = interner.function(FunctionShape {
        params: vec![ParamInfo {
            name: Some(interner.intern_string("x")),
            type_id: TypeId::NUMBER,
            optional: false,
            rest: false,
        }],
        this_type: None,
        return_type: TypeId::VOID,
        type_params: Vec::new(),
        type_predicate: None,
        is_constructor: false,
    });
    let branded = interner.intersection(vec![func, brand]);
    let union = interner.union(vec![branded, TypeId::NUMBER]);

    let narrowed = ctx.narrow_excluding_function(union);
    assert_eq!(narrowed, TypeId::NUMBER);
}

#[test]
fn test_narrow_by_typeof_negation_function_type_param_with_union_constraint() {
    let interner = TypeInterner::new();
    let ctx = NarrowingContext::new(&interner);

    let func = interner.function(FunctionShape {
        params: vec![ParamInfo {
            name: Some(interner.intern_string("x")),
            type_id: TypeId::NUMBER,
            optional: false,
            rest: false,
        }],
        this_type: None,
        return_type: TypeId::VOID,
        type_params: Vec::new(),
        type_predicate: None,
        is_constructor: false,
    });
    let constraint = interner.union(vec![func, TypeId::STRING]);
    let param = interner.intern(TypeKey::TypeParameter(TypeParamInfo {
        name: interner.intern_string("T"),
        constraint: Some(constraint),
        default: None,
    }));
    let union = interner.union(vec![param, TypeId::BOOLEAN]);

    let narrowed = ctx.narrow_excluding_function(union);
    let expected_param = interner.intersection(vec![param, TypeId::STRING]);
    let expected = interner.union(vec![expected_param, TypeId::BOOLEAN]);
    assert_eq!(narrowed, expected);
}

#[test]
fn test_narrow_by_typeof_negation_function_type_param_to_never() {
    let interner = TypeInterner::new();
    let ctx = NarrowingContext::new(&interner);

    let func = interner.function(FunctionShape {
        params: vec![ParamInfo {
            name: Some(interner.intern_string("x")),
            type_id: TypeId::NUMBER,
            optional: false,
            rest: false,
        }],
        this_type: None,
        return_type: TypeId::VOID,
        type_params: Vec::new(),
        type_predicate: None,
        is_constructor: false,
    });
    let param = interner.intern(TypeKey::TypeParameter(TypeParamInfo {
        name: interner.intern_string("T"),
        constraint: Some(func),
        default: None,
    }));

    let narrowed = ctx.narrow_excluding_function(param);
    assert_eq!(narrowed, TypeId::NEVER);
}

#[test]
fn test_narrow_by_typeof_type_param_with_union_constraint() {
    let interner = TypeInterner::new();
    let constraint = interner.union(vec![TypeId::STRING, TypeId::NUMBER]);
    let param = interner.intern(TypeKey::TypeParameter(TypeParamInfo {
        name: interner.intern_string("T"),
        constraint: Some(constraint),
        default: None,
    }));
    let union = interner.union(vec![param, TypeId::BOOLEAN]);

    let narrowed = narrow_by_typeof(&interner, union, "string");
    let expected = interner.intersection(vec![param, TypeId::STRING]);
    assert_eq!(narrowed, expected);
}

#[test]
fn test_narrow_by_typeof_function_type_param_with_union_constraint() {
    let interner = TypeInterner::new();

    let func = interner.function(FunctionShape {
        params: vec![ParamInfo {
            name: Some(interner.intern_string("x")),
            type_id: TypeId::NUMBER,
            optional: false,
            rest: false,
        }],
        this_type: None,
        return_type: TypeId::VOID,
        type_params: Vec::new(),
        type_predicate: None,
        is_constructor: false,
    });
    let constraint = interner.union(vec![func, TypeId::STRING]);
    let param = interner.intern(TypeKey::TypeParameter(TypeParamInfo {
        name: interner.intern_string("T"),
        constraint: Some(constraint),
        default: None,
    }));
    let union = interner.union(vec![param, TypeId::BOOLEAN]);

    let narrowed = narrow_by_typeof(&interner, union, "function");
    let expected = interner.intersection(vec![param, func]);
    assert_eq!(narrowed, expected);
}

#[test]
fn test_narrow_by_typeof_function_type_param_with_non_function_constraint() {
    let interner = TypeInterner::new();
    let param = interner.intern(TypeKey::TypeParameter(TypeParamInfo {
        name: interner.intern_string("T"),
        constraint: Some(TypeId::NUMBER),
        default: None,
    }));

    let narrowed = narrow_by_typeof(&interner, param, "function");
    assert_eq!(narrowed, TypeId::NEVER);
}

#[test]
fn test_narrow_by_typeof_function_unconstrained_type_param() {
    let interner = TypeInterner::new();
    let ctx = NarrowingContext::new(&interner);
    let param = interner.intern(TypeKey::TypeParameter(TypeParamInfo {
        name: interner.intern_string("T"),
        constraint: None,
        default: None,
    }));

    let narrowed = narrow_by_typeof(&interner, param, "function");
    let expected = interner.intersection(vec![param, ctx.function_type()]);
    assert_eq!(narrowed, expected);
}

#[test]
fn test_narrow_by_typeof_type_param_with_non_overlapping_constraint() {
    let interner = TypeInterner::new();
    let param = interner.intern(TypeKey::TypeParameter(TypeParamInfo {
        name: interner.intern_string("T"),
        constraint: Some(TypeId::NUMBER),
        default: None,
    }));

    let narrowed = narrow_by_typeof(&interner, param, "string");
    assert_eq!(narrowed, TypeId::NEVER);
}

#[test]
fn test_narrow_by_typeof_unconstrained_type_param() {
    let interner = TypeInterner::new();
    let param = interner.intern(TypeKey::TypeParameter(TypeParamInfo {
        name: interner.intern_string("T"),
        constraint: None,
        default: None,
    }));

    let narrowed = narrow_by_typeof(&interner, param, "string");
    let expected = interner.intersection(vec![param, TypeId::STRING]);
    assert_eq!(narrowed, expected);
}

#[test]
fn test_narrow_by_typeof_branded_string_intersection() {
    let interner = TypeInterner::new();

    let brand = interner.object(vec![PropertyInfo {
        name: interner.intern_string("__brand"),
        type_id: interner.literal_string("UserId"),
        write_type: interner.literal_string("UserId"),
        optional: false,
        readonly: false,
        is_method: false,
    }]);
    let branded = interner.intersection(vec![TypeId::STRING, brand]);
    let union = interner.union(vec![branded, TypeId::NUMBER]);

    let narrowed = narrow_by_typeof(&interner, union, "string");
    assert_eq!(narrowed, branded);
}

#[test]
fn test_narrow_by_typeof_branded_function_intersection() {
    let interner = TypeInterner::new();

    let brand = interner.object(vec![PropertyInfo {
        name: interner.intern_string("__brand"),
        type_id: interner.literal_string("Tagged"),
        write_type: interner.literal_string("Tagged"),
        optional: false,
        readonly: false,
        is_method: false,
    }]);
    let func = interner.function(FunctionShape {
        params: vec![ParamInfo {
            name: Some(interner.intern_string("x")),
            type_id: TypeId::NUMBER,
            optional: false,
            rest: false,
        }],
        this_type: None,
        return_type: TypeId::VOID,
        type_params: Vec::new(),
        type_predicate: None,
        is_constructor: false,
    });
    let branded = interner.intersection(vec![func, brand]);
    let union = interner.union(vec![branded, TypeId::NUMBER]);

    let narrowed = narrow_by_typeof(&interner, union, "function");
    assert_eq!(narrowed, branded);
}

#[test]
fn test_narrow_by_typeof_object_excludes_branded_function_intersection() {
    let interner = TypeInterner::new();

    let brand = interner.object(vec![PropertyInfo {
        name: interner.intern_string("__brand"),
        type_id: interner.literal_string("Tagged"),
        write_type: interner.literal_string("Tagged"),
        optional: false,
        readonly: false,
        is_method: false,
    }]);
    let func = interner.function(FunctionShape {
        params: vec![ParamInfo {
            name: Some(interner.intern_string("x")),
            type_id: TypeId::NUMBER,
            optional: false,
            rest: false,
        }],
        this_type: None,
        return_type: TypeId::VOID,
        type_params: Vec::new(),
        type_predicate: None,
        is_constructor: false,
    });
    let branded = interner.intersection(vec![func, brand]);
    let obj = interner.object(vec![PropertyInfo {
        name: interner.intern_string("value"),
        type_id: TypeId::NUMBER,
        write_type: TypeId::NUMBER,
        optional: false,
        readonly: false,
        is_method: false,
    }]);
    let union = interner.union(vec![branded, obj]);

    let narrowed = narrow_by_typeof(&interner, union, "object");
    assert_eq!(narrowed, obj);
}

#[test]
fn test_narrow_by_typeof_object_with_object_literal() {
    let interner = TypeInterner::new();

    let obj = interner.object(vec![PropertyInfo {
        name: interner.intern_string("value"),
        type_id: TypeId::NUMBER,
        write_type: TypeId::NUMBER,
        optional: false,
        readonly: false,
        is_method: false,
    }]);
    let union = interner.union(vec![obj, TypeId::NUMBER]);

    let narrowed = narrow_by_typeof(&interner, union, "object");
    assert_eq!(narrowed, obj);
}

#[test]
fn test_narrow_by_typeof_object_excludes_function() {
    let interner = TypeInterner::new();

    let obj = interner.object(vec![PropertyInfo {
        name: interner.intern_string("value"),
        type_id: TypeId::NUMBER,
        write_type: TypeId::NUMBER,
        optional: false,
        readonly: false,
        is_method: false,
    }]);
    let func = interner.function(FunctionShape {
        params: vec![ParamInfo {
            name: Some(interner.intern_string("x")),
            type_id: TypeId::NUMBER,
            optional: false,
            rest: false,
        }],
        this_type: None,
        return_type: TypeId::VOID,
        type_params: Vec::new(),
        type_predicate: None,
        is_constructor: false,
    });
    let union = interner.union(vec![obj, func]);

    let narrowed = narrow_by_typeof(&interner, union, "object");
    assert_eq!(narrowed, obj);
}

#[test]
fn test_narrow_by_typeof_function_includes_callable() {
    let interner = TypeInterner::new();

    let sig = CallSignature {
        type_params: vec![],
        params: vec![ParamInfo {
            name: Some(interner.intern_string("x")),
            type_id: TypeId::NUMBER,
            optional: false,
            rest: false,
        }],
        this_type: None,
        return_type: TypeId::STRING,
        type_predicate: None,
    };
    let callable = interner.callable(CallableShape {
        call_signatures: vec![sig],
        construct_signatures: vec![],
        properties: vec![],
    });
    let union = interner.union(vec![callable, TypeId::NUMBER]);

    let narrowed = narrow_by_typeof(&interner, union, "function");
    assert_eq!(narrowed, callable);
}

// =============================================================================
// General Narrowing Tests
// =============================================================================

#[test]
fn test_narrow_to_type() {
    let interner = TypeInterner::new();
    let ctx = NarrowingContext::new(&interner);

    // string | number | boolean narrowed to string -> string
    let union = interner.union(vec![TypeId::STRING, TypeId::NUMBER, TypeId::BOOLEAN]);

    let narrowed = ctx.narrow_to_type(union, TypeId::STRING);
    assert_eq!(narrowed, TypeId::STRING);
}

#[test]
fn test_narrow_excluding_type() {
    let interner = TypeInterner::new();
    let ctx = NarrowingContext::new(&interner);

    // string | number | boolean excluding string -> number | boolean
    let union = interner.union(vec![TypeId::STRING, TypeId::NUMBER, TypeId::BOOLEAN]);

    let narrowed = ctx.narrow_excluding_type(union, TypeId::STRING);
    let expected = interner.union(vec![TypeId::NUMBER, TypeId::BOOLEAN]);
    assert_eq!(narrowed, expected);
}

#[test]
fn test_narrow_excluding_type_param_with_union_constraint() {
    let interner = TypeInterner::new();
    let ctx = NarrowingContext::new(&interner);

    let constraint = interner.union(vec![TypeId::STRING, TypeId::NUMBER]);
    let param = interner.intern(TypeKey::TypeParameter(TypeParamInfo {
        name: interner.intern_string("T"),
        constraint: Some(constraint),
        default: None,
    }));
    let union = interner.union(vec![param, TypeId::BOOLEAN]);

    let narrowed = ctx.narrow_excluding_type(union, TypeId::STRING);
    let expected_param = interner.intersection(vec![param, TypeId::NUMBER]);
    let expected = interner.union(vec![expected_param, TypeId::BOOLEAN]);
    assert_eq!(narrowed, expected);
}

#[test]
fn test_narrow_excluding_type_param_with_non_overlapping_constraint() {
    let interner = TypeInterner::new();
    let ctx = NarrowingContext::new(&interner);

    let param = interner.intern(TypeKey::TypeParameter(TypeParamInfo {
        name: interner.intern_string("T"),
        constraint: Some(TypeId::NUMBER),
        default: None,
    }));

    let narrowed = ctx.narrow_excluding_type(param, TypeId::STRING);
    assert_eq!(narrowed, param);
}

#[test]
fn test_narrow_excluding_type_param_to_never() {
    let interner = TypeInterner::new();
    let ctx = NarrowingContext::new(&interner);

    let param = interner.intern(TypeKey::TypeParameter(TypeParamInfo {
        name: interner.intern_string("T"),
        constraint: Some(TypeId::STRING),
        default: None,
    }));

    let narrowed = ctx.narrow_excluding_type(param, TypeId::STRING);
    assert_eq!(narrowed, TypeId::NEVER);
}

#[test]
fn test_narrow_to_never() {
    let interner = TypeInterner::new();
    let ctx = NarrowingContext::new(&interner);

    // string excluding string -> never
    let narrowed = ctx.narrow_excluding_type(TypeId::STRING, TypeId::STRING);
    assert_eq!(narrowed, TypeId::NEVER);
}

#[test]
fn test_narrow_single_member_union() {
    let interner = TypeInterner::new();
    let ctx = NarrowingContext::new(&interner);

    // string | number excluding string -> number (not a union)
    let union = interner.union(vec![TypeId::STRING, TypeId::NUMBER]);

    let narrowed = ctx.narrow_excluding_type(union, TypeId::STRING);
    assert_eq!(narrowed, TypeId::NUMBER);
}

// =============================================================================
// Type Predicate Structure Tests
// =============================================================================
// These tests verify TypePredicate structures are correctly created.
// Actual narrowing with type predicates happens at the checker level.

#[test]
fn test_type_predicate_basic_structure() {
    use super::TypePredicate;
    use super::TypePredicateTarget;

    let interner = TypeInterner::new();
    let x_name = interner.intern_string("x");

    // x is string
    let predicate = TypePredicate {
        asserts: false,
        target: TypePredicateTarget::Identifier(x_name),
        type_id: Some(TypeId::STRING),
    };

    assert!(!predicate.asserts);
    assert_eq!(predicate.target, TypePredicateTarget::Identifier(x_name));
    assert_eq!(predicate.type_id, Some(TypeId::STRING));
}

#[test]
fn test_type_predicate_asserts_structure() {
    use super::TypePredicate;
    use super::TypePredicateTarget;

    let interner = TypeInterner::new();
    let x_name = interner.intern_string("x");

    // asserts x is string
    let predicate = TypePredicate {
        asserts: true,
        target: TypePredicateTarget::Identifier(x_name),
        type_id: Some(TypeId::STRING),
    };

    assert!(predicate.asserts);
    assert_eq!(predicate.target, TypePredicateTarget::Identifier(x_name));
    assert_eq!(predicate.type_id, Some(TypeId::STRING));
}

#[test]
fn test_type_predicate_this_target() {
    use super::TypePredicate;
    use super::TypePredicateTarget;

    let interner = TypeInterner::new();

    // Create an object type for the predicate
    let foo_name = interner.intern_string("foo");
    let foo_type = interner.object(vec![
        PropertyInfo {
            name: foo_name,
            type_id: TypeId::STRING,
            write_type: TypeId::STRING,
            optional: false,
            readonly: false,
            is_method: false,
        },
    ]);

    // this is Foo
    let predicate = TypePredicate {
        asserts: false,
        target: TypePredicateTarget::This,
        type_id: Some(foo_type),
    };

    assert!(!predicate.asserts);
    assert_eq!(predicate.target, TypePredicateTarget::This);
    assert_eq!(predicate.type_id, Some(foo_type));
}

#[test]
fn test_type_predicate_asserts_without_type() {
    use super::TypePredicate;
    use super::TypePredicateTarget;

    let interner = TypeInterner::new();
    let x_name = interner.intern_string("x");

    // asserts x (no type - just assertion that x is truthy)
    let predicate = TypePredicate {
        asserts: true,
        target: TypePredicateTarget::Identifier(x_name),
        type_id: None,
    };

    assert!(predicate.asserts);
    assert_eq!(predicate.target, TypePredicateTarget::Identifier(x_name));
    assert_eq!(predicate.type_id, None);
}

#[test]
fn test_function_shape_with_type_predicate() {
    use super::{FunctionShape, ParamInfo, TypePredicate, TypePredicateTarget};

    let interner = TypeInterner::new();
    let x_name = interner.intern_string("x");

    // function isString(x: any): x is string
    let shape = FunctionShape {
        type_params: vec![],
        params: vec![ParamInfo {
            name: Some(x_name),
            type_id: TypeId::ANY,
            optional: false,
            rest: false,
        }],
        this_type: None,
        return_type: TypeId::BOOLEAN,
        type_predicate: Some(TypePredicate {
            asserts: false,
            target: TypePredicateTarget::Identifier(x_name),
            type_id: Some(TypeId::STRING),
        }),
        is_constructor: false,
    };

    assert!(shape.type_predicate.is_some());
    let pred = shape.type_predicate.unwrap();
    assert!(!pred.asserts);
    assert_eq!(pred.type_id, Some(TypeId::STRING));
}

#[test]
fn test_call_signature_with_type_predicate() {
    use super::{CallSignature, ParamInfo, TypePredicate, TypePredicateTarget};

    let interner = TypeInterner::new();
    let x_name = interner.intern_string("x");

    // Overload: (x: any): x is number
    let sig = CallSignature {
        type_params: vec![],
        params: vec![ParamInfo {
            name: Some(x_name),
            type_id: TypeId::ANY,
            optional: false,
            rest: false,
        }],
        this_type: None,
        return_type: TypeId::BOOLEAN,
        type_predicate: Some(TypePredicate {
            asserts: false,
            target: TypePredicateTarget::Identifier(x_name),
            type_id: Some(TypeId::NUMBER),
        }),
    };

    assert!(sig.type_predicate.is_some());
    let pred = sig.type_predicate.unwrap();
    assert_eq!(pred.type_id, Some(TypeId::NUMBER));
}

#[test]
fn test_narrow_to_type_simulates_type_predicate_narrowing() {
    let interner = TypeInterner::new();
    let ctx = NarrowingContext::new(&interner);

    // Simulating what happens after a type predicate check:
    // if (isString(x)) { /* x is narrowed to string here */ }

    // Start with x: string | number
    let union = interner.union(vec![TypeId::STRING, TypeId::NUMBER]);

    // After type predicate `x is string` returns true:
    // Narrow to string (the predicate type)
    let narrowed = ctx.narrow_to_type(union, TypeId::STRING);

    // Should be narrowed to string
    assert_eq!(narrowed, TypeId::STRING);
}

#[test]
fn test_narrow_excluding_type_simulates_type_predicate_false_branch() {
    let interner = TypeInterner::new();
    let ctx = NarrowingContext::new(&interner);

    // Simulating the else branch after a type predicate check:
    // if (isString(x)) { ... } else { /* x is NOT string here */ }

    // Start with x: string | number
    let union = interner.union(vec![TypeId::STRING, TypeId::NUMBER]);

    // After type predicate `x is string` returns false:
    // Narrow by excluding string
    let narrowed = ctx.narrow_excluding_type(union, TypeId::STRING);

    // Should be narrowed to number
    assert_eq!(narrowed, TypeId::NUMBER);
}

#[test]
fn test_narrow_to_interface_type() {
    let interner = TypeInterner::new();
    let ctx = NarrowingContext::new(&interner);

    // Simulating interface narrowing:
    // interface Cat { meow(): void }
    // interface Dog { bark(): void }
    // function isCat(x: Cat | Dog): x is Cat

    let meow_name = interner.intern_string("meow");
    let bark_name = interner.intern_string("bark");

    let cat_type = interner.object(vec![PropertyInfo {
        name: meow_name,
        type_id: TypeId::VOID,
        write_type: TypeId::VOID,
        optional: false,
        readonly: false,
        is_method: true,
    }]);

    let dog_type = interner.object(vec![PropertyInfo {
        name: bark_name,
        type_id: TypeId::VOID,
        write_type: TypeId::VOID,
        optional: false,
        readonly: false,
        is_method: true,
    }]);

    let union = interner.union(vec![cat_type, dog_type]);

    // After type predicate `x is Cat` returns true:
    let narrowed = ctx.narrow_to_type(union, cat_type);

    // Should be narrowed to Cat
    assert_eq!(narrowed, cat_type);
}

// =============================================================================
// Unknown Type Tests (Type Guards, Narrowing from Unknown)
// =============================================================================

#[test]
fn test_narrow_unknown_by_typeof_boolean() {
    let interner = TypeInterner::new();

    let narrowed = narrow_by_typeof(&interner, TypeId::UNKNOWN, "boolean");
    assert_eq!(narrowed, TypeId::BOOLEAN);
}

#[test]
fn test_narrow_unknown_by_typeof_bigint() {
    let interner = TypeInterner::new();

    let narrowed = narrow_by_typeof(&interner, TypeId::UNKNOWN, "bigint");
    assert_eq!(narrowed, TypeId::BIGINT);
}

#[test]
fn test_narrow_unknown_by_typeof_symbol() {
    let interner = TypeInterner::new();

    let narrowed = narrow_by_typeof(&interner, TypeId::UNKNOWN, "symbol");
    assert_eq!(narrowed, TypeId::SYMBOL);
}

#[test]
fn test_narrow_unknown_by_typeof_undefined() {
    let interner = TypeInterner::new();

    let narrowed = narrow_by_typeof(&interner, TypeId::UNKNOWN, "undefined");
    assert_eq!(narrowed, TypeId::UNDEFINED);
}

#[test]
fn test_narrow_unknown_by_typeof_number() {
    let interner = TypeInterner::new();

    let narrowed = narrow_by_typeof(&interner, TypeId::UNKNOWN, "number");
    assert_eq!(narrowed, TypeId::NUMBER);
}

#[test]
fn test_narrow_unknown_by_equality_to_null() {
    let interner = TypeInterner::new();
    let ctx = NarrowingContext::new(&interner);

    // x === null should narrow unknown to null
    let narrowed = ctx.narrow_to_type(TypeId::UNKNOWN, TypeId::NULL);
    assert_eq!(narrowed, TypeId::NULL);
}

#[test]
fn test_narrow_unknown_by_equality_to_undefined() {
    let interner = TypeInterner::new();
    let ctx = NarrowingContext::new(&interner);

    // x === undefined should narrow unknown to undefined
    let narrowed = ctx.narrow_to_type(TypeId::UNKNOWN, TypeId::UNDEFINED);
    assert_eq!(narrowed, TypeId::UNDEFINED);
}

#[test]
fn test_narrow_unknown_by_equality_to_literal() {
    let interner = TypeInterner::new();
    let ctx = NarrowingContext::new(&interner);

    // x === "hello" should narrow unknown to "hello"
    let hello = interner.literal_string("hello");
    let narrowed = ctx.narrow_to_type(TypeId::UNKNOWN, hello);
    assert_eq!(narrowed, hello);
}

#[test]
fn test_narrow_unknown_by_truthiness() {
    let interner = TypeInterner::new();
    let ctx = NarrowingContext::new(&interner);

    // if (x) where x: unknown should exclude nullish values
    let narrowed = ctx.narrow_excluding_type(TypeId::UNKNOWN, TypeId::NULL);
    // Should still include other values
    assert!(narrowed != TypeId::NEVER);
}

#[test]
fn test_narrow_unknown_by_in_operator() {
    let interner = TypeInterner::new();
    let ctx = NarrowingContext::new(&interner);

    // "prop" in x where x: unknown should narrow to object with that property
    let prop_name = interner.intern_string("prop");
    let obj_with_prop = interner.object(vec![PropertyInfo {
        name: prop_name,
        type_id: TypeId::UNKNOWN,
        write_type: TypeId::UNKNOWN,
        optional: false,
        readonly: false,
        is_method: false,
    }]);

    let narrowed = ctx.narrow_to_type(TypeId::UNKNOWN, obj_with_prop);
    assert_eq!(narrowed, obj_with_prop);
}

#[test]
fn test_narrow_unknown_by_type_predicate() {
    let interner = TypeInterner::new();
    let ctx = NarrowingContext::new(&interner);

    // isString(x) where x is string predicate
    let narrowed = ctx.narrow_to_type(TypeId::UNKNOWN, TypeId::STRING);
    assert_eq!(narrowed, TypeId::STRING);
}

#[test]
fn test_narrow_unknown_by_type_predicate_to_object() {
    let interner = TypeInterner::new();
    let ctx = NarrowingContext::new(&interner);

    // isUser(x) where x is User predicate
    let name_prop = interner.intern_string("name");
    let user_type = interner.object(vec![PropertyInfo {
        name: name_prop,
        type_id: TypeId::STRING,
        write_type: TypeId::STRING,
        optional: false,
        readonly: false,
        is_method: false,
    }]);

    let narrowed = ctx.narrow_to_type(TypeId::UNKNOWN, user_type);
    assert_eq!(narrowed, user_type);
}

#[test]
fn test_narrow_unknown_excludes_null() {
    let interner = TypeInterner::new();
    let ctx = NarrowingContext::new(&interner);

    // x !== null
    let narrowed = ctx.narrow_excluding_type(TypeId::UNKNOWN, TypeId::NULL);
    // Should not be unknown anymore, but also not never
    assert!(narrowed != TypeId::NEVER);
}

#[test]
fn test_narrow_unknown_excludes_undefined() {
    let interner = TypeInterner::new();
    let ctx = NarrowingContext::new(&interner);

    // x !== undefined
    let narrowed = ctx.narrow_excluding_type(TypeId::UNKNOWN, TypeId::UNDEFINED);
    assert!(narrowed != TypeId::NEVER);
}

#[test]
fn test_narrow_unknown_by_typeof_negation_object() {
    let interner = TypeInterner::new();

    // typeof x !== "object"
    let narrowed = narrow_by_typeof_negation(&interner, TypeId::UNKNOWN, "object");
    // Should exclude object and null, keeping primitives
    assert!(narrowed != TypeId::NEVER);
    assert!(narrowed != TypeId::UNKNOWN);
}

#[test]
fn test_narrow_unknown_by_typeof_negation_function() {
    let interner = TypeInterner::new();

    // typeof x !== "function"
    let narrowed = narrow_by_typeof_negation(&interner, TypeId::UNKNOWN, "function");
    // Should exclude functions
    assert!(narrowed != TypeId::NEVER);
}

#[test]
fn test_narrow_unknown_by_typeof_negation_string() {
    let interner = TypeInterner::new();

    // typeof x !== "string"
    let narrowed = narrow_by_typeof_negation(&interner, TypeId::UNKNOWN, "string");
    // Should exclude string
    assert!(narrowed != TypeId::NEVER);
}

#[test]
fn test_narrow_unknown_by_typeof_negation_number() {
    let interner = TypeInterner::new();

    // typeof x !== "number"
    let narrowed = narrow_by_typeof_negation(&interner, TypeId::UNKNOWN, "number");
    assert!(narrowed != TypeId::NEVER);
}

#[test]
fn test_narrow_unknown_sequential_typeof() {
    let interner = TypeInterner::new();

    // First: typeof x === "object"
    let after_object = narrow_by_typeof(&interner, TypeId::UNKNOWN, "object");
    // Result should be object | null
    let expected = interner.union(vec![TypeId::OBJECT, TypeId::NULL]);
    assert_eq!(after_object, expected);

    // Second: x !== null (on the narrowed type)
    let ctx = NarrowingContext::new(&interner);
    let final_narrowed = ctx.narrow_excluding_type(after_object, TypeId::NULL);
    // Should be just object
    assert_eq!(final_narrowed, TypeId::OBJECT);
}

#[test]
fn test_narrow_unknown_to_array() {
    let interner = TypeInterner::new();
    let ctx = NarrowingContext::new(&interner);

    // Array.isArray(x)
    let array_type = interner.array(TypeId::UNKNOWN);
    let narrowed = ctx.narrow_to_type(TypeId::UNKNOWN, array_type);
    assert_eq!(narrowed, array_type);
}

#[test]
fn test_narrow_unknown_to_tuple() {
    let interner = TypeInterner::new();
    let ctx = NarrowingContext::new(&interner);

    // x is [string, number]
    let tuple_type = interner.tuple_with_info(vec![
        TupleElementInfo {
            type_id: TypeId::STRING,
            optional: false,
            label: None,
        },
        TupleElementInfo {
            type_id: TypeId::NUMBER,
            optional: false,
            label: None,
        },
    ]);

    let narrowed = ctx.narrow_to_type(TypeId::UNKNOWN, tuple_type);
    assert_eq!(narrowed, tuple_type);
}

#[test]
fn test_narrow_unknown_to_function() {
    let interner = TypeInterner::new();
    let ctx = NarrowingContext::new(&interner);

    // x is (a: number) => string
    let fn_type = interner.function(FunctionShape {
        type_params: vec![],
        params: vec![ParamInfo {
            name: Some(interner.intern_string("a")),
            type_id: TypeId::NUMBER,
            optional: false,
            rest: false,
        }],
        this_type: None,
        return_type: TypeId::STRING,
        type_predicate: None,
        is_constructor: false,
    });

    let narrowed = ctx.narrow_to_type(TypeId::UNKNOWN, fn_type);
    assert_eq!(narrowed, fn_type);
}

#[test]
fn test_narrow_unknown_to_union() {
    let interner = TypeInterner::new();
    let ctx = NarrowingContext::new(&interner);

    // x is string | number
    let union = interner.union(vec![TypeId::STRING, TypeId::NUMBER]);
    let narrowed = ctx.narrow_to_type(TypeId::UNKNOWN, union);
    assert_eq!(narrowed, union);
}

#[test]
fn test_narrow_unknown_to_intersection() {
    let interner = TypeInterner::new();
    let ctx = NarrowingContext::new(&interner);

    // x is A & B
    let a_name = interner.intern_string("a");
    let b_name = interner.intern_string("b");

    let type_a = interner.object(vec![PropertyInfo {
        name: a_name,
        type_id: TypeId::STRING,
        write_type: TypeId::STRING,
        optional: false,
        readonly: false,
        is_method: false,
    }]);

    let type_b = interner.object(vec![PropertyInfo {
        name: b_name,
        type_id: TypeId::NUMBER,
        write_type: TypeId::NUMBER,
        optional: false,
        readonly: false,
        is_method: false,
    }]);

    let intersection = interner.intersection(vec![type_a, type_b]);
    let narrowed = ctx.narrow_to_type(TypeId::UNKNOWN, intersection);
    assert_eq!(narrowed, intersection);
}

#[test]
fn test_narrow_unknown_with_discriminated_union() {
    let interner = TypeInterner::new();
    let ctx = NarrowingContext::new(&interner);

    // x.kind === "success"
    let kind_name = interner.intern_string("kind");
    let success_lit = interner.literal_string("success");
    let error_lit = interner.literal_string("error");

    let success_type = interner.object(vec![PropertyInfo {
        name: kind_name,
        type_id: success_lit,
        write_type: success_lit,
        optional: false,
        readonly: false,
        is_method: false,
    }]);

    let error_type = interner.object(vec![PropertyInfo {
        name: kind_name,
        type_id: error_lit,
        write_type: error_lit,
        optional: false,
        readonly: false,
        is_method: false,
    }]);

    let union = interner.union(vec![success_type, error_type]);

    // Narrow unknown to the union first
    let narrowed_to_union = ctx.narrow_to_type(TypeId::UNKNOWN, union);
    assert_eq!(narrowed_to_union, union);

    // Then narrow by discriminant
    let narrowed_to_success = ctx.narrow_by_discriminant(narrowed_to_union, kind_name, success_lit);
    assert_eq!(narrowed_to_success, success_type);
}

#[test]
fn test_narrow_unknown_excludes_nullish() {
    let interner = TypeInterner::new();
    let ctx = NarrowingContext::new(&interner);

    // x != null (excludes both null and undefined)
    let nullish = interner.union(vec![TypeId::NULL, TypeId::UNDEFINED]);
    let narrowed = ctx.narrow_excluding_type(TypeId::UNKNOWN, nullish);
    assert!(narrowed != TypeId::NEVER);
}

#[test]
fn test_narrow_unknown_intersection_preserves_unknown() {
    // unknown & T = T
    let interner = TypeInterner::new();

    let intersection = interner.intersection(vec![TypeId::UNKNOWN, TypeId::STRING]);
    // unknown & string should simplify to string
    assert!(intersection != TypeId::ERROR);
}

#[test]
fn test_narrow_unknown_union_with_unknown() {
    // T | unknown = unknown
    let interner = TypeInterner::new();

    let union = interner.union(vec![TypeId::STRING, TypeId::UNKNOWN]);
    // string | unknown should be unknown (unknown absorbs all)
    assert_eq!(union, TypeId::UNKNOWN);
}
