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
        PropertyInfo { name: type_name, type_id: type_add, optional: false, readonly: false, is_method: false },
    ]);
    let member2 = interner.object(vec![
        PropertyInfo { name: type_name, type_id: type_remove, optional: false, readonly: false, is_method: false },
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
        PropertyInfo { name: kind_name, type_id: kind_a, optional: false, readonly: false, is_method: false },
        PropertyInfo { name: type_name, type_id: type_1, optional: false, readonly: false, is_method: false },
    ]);
    let member2 = interner.object(vec![
        PropertyInfo { name: kind_name, type_id: kind_b, optional: false, readonly: false, is_method: false },
        PropertyInfo { name: type_name, type_id: type_2, optional: false, readonly: false, is_method: false },
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
        PropertyInfo { name: type_name, type_id: TypeId::STRING, optional: false, readonly: false, is_method: false },
    ]);
    let member2 = interner.object(vec![
        PropertyInfo { name: type_name, type_id: TypeId::STRING, optional: false, readonly: false, is_method: false },
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
        PropertyInfo { name: type_name, type_id: type_a, optional: false, readonly: false, is_method: false },
    ]);
    let member2 = interner.object(vec![
        PropertyInfo { name: kind_name, type_id: kind_b, optional: false, readonly: false, is_method: false },
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
        PropertyInfo { name: type_name, type_id: type_add, optional: false, readonly: false, is_method: false },
        PropertyInfo { name: interner.intern_string("value"), type_id: TypeId::NUMBER, optional: false, readonly: false, is_method: false },
    ]);
    let member_remove = interner.object(vec![
        PropertyInfo { name: type_name, type_id: type_remove, optional: false, readonly: false, is_method: false },
        PropertyInfo { name: interner.intern_string("id"), type_id: TypeId::STRING, optional: false, readonly: false, is_method: false },
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
        PropertyInfo { name: type_name, type_id: type_add, optional: false, readonly: false, is_method: false },
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
        PropertyInfo { name: type_name, type_id: type_a, optional: false, readonly: false, is_method: false },
    ]);
    let member_b = interner.object(vec![
        PropertyInfo { name: type_name, type_id: type_b, optional: false, readonly: false, is_method: false },
    ]);
    let member_c = interner.object(vec![
        PropertyInfo { name: type_name, type_id: type_c, optional: false, readonly: false, is_method: false },
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

    let template = interner.intern(TypeKey::TemplateLiteral(vec![
        TemplateSpan::Text(interner.intern_string("prefix")),
        TemplateSpan::Type(TypeId::STRING),
        TemplateSpan::Text(interner.intern_string("suffix")),
    ]));
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
fn test_narrow_by_typeof_branded_string_intersection() {
    let interner = TypeInterner::new();

    let brand = interner.object(vec![PropertyInfo {
        name: interner.intern_string("__brand"),
        type_id: interner.literal_string("UserId"),
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
