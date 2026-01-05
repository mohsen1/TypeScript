use super::*;
use std::sync::Arc;

#[test]
fn test_conditional_true_branch() {
    let interner = TypeInterner::new();

    // string extends string ? number : boolean
    // Should resolve to number
    let cond = ConditionalType {
        check_type: TypeId::STRING,
        extends_type: TypeId::STRING,
        true_type: TypeId::NUMBER,
        false_type: TypeId::BOOLEAN,
    };

    let result = evaluate_conditional(&interner, &cond);
    assert_eq!(result, TypeId::NUMBER);
}

#[test]
fn test_conditional_false_branch() {
    let interner = TypeInterner::new();

    // number extends string ? number : boolean
    // Should resolve to boolean (number is not subtype of string)
    let cond = ConditionalType {
        check_type: TypeId::NUMBER,
        extends_type: TypeId::STRING,
        true_type: TypeId::NUMBER,
        false_type: TypeId::BOOLEAN,
    };

    let result = evaluate_conditional(&interner, &cond);
    assert_eq!(result, TypeId::BOOLEAN);
}

#[test]
fn test_conditional_literal_extends_base() {
    let interner = TypeInterner::new();

    // "hello" extends string ? true : false
    // Should resolve to true (literal is subtype of base)
    let hello = interner.literal_string("hello");
    let lit_true = interner.literal_boolean(true);
    let lit_false = interner.literal_boolean(false);

    let cond = ConditionalType {
        check_type: hello,
        extends_type: TypeId::STRING,
        true_type: lit_true,
        false_type: lit_false,
    };

    let result = evaluate_conditional(&interner, &cond);
    assert_eq!(result, lit_true);
}

#[test]
fn test_conditional_distributive() {
    let interner = TypeInterner::new();

    // (string | number) extends string ? true : false
    // Distributes to: (string extends string ? true : false) | (number extends string ? true : false)
    // = true | false
    let string_or_number = interner.union(vec![TypeId::STRING, TypeId::NUMBER]);
    let lit_true = interner.literal_boolean(true);
    let lit_false = interner.literal_boolean(false);

    let cond = ConditionalType {
        check_type: string_or_number,
        extends_type: TypeId::STRING,
        true_type: lit_true,
        false_type: lit_false,
    };

    let result = evaluate_conditional(&interner, &cond);

    // Result should be true | false (i.e., boolean union of literals)
    let expected = interner.union(vec![lit_true, lit_false]);
    assert_eq!(result, expected);
}

#[test]
fn test_conditional_deferred_type_parameter() {
    let interner = TypeInterner::new();

    // T extends string ? number : boolean
    // Should remain deferred when T is an unsubstituted type parameter
    let type_param_t = interner.intern(TypeKey::TypeParameter(TypeParamInfo {
        name: Arc::from("T"),
        constraint: None,
        default: None,
    }));

    let cond = ConditionalType {
        check_type: type_param_t,
        extends_type: TypeId::STRING,
        true_type: TypeId::NUMBER,
        false_type: TypeId::BOOLEAN,
    };

    let cond_type = interner.intern(TypeKey::Conditional(Box::new(cond.clone())));
    let result = evaluate_conditional(&interner, &cond);

    // Should return the same conditional (deferred)
    assert_eq!(result, cond_type);
}

#[test]
fn test_index_access_object_literal() {
    let interner = TypeInterner::new();

    // { x: number, y: string }["x"] -> number
    let obj = interner.object(vec![
        PropertyInfo { name: Arc::from("x"), type_id: TypeId::NUMBER, optional: false, readonly: false },
        PropertyInfo { name: Arc::from("y"), type_id: TypeId::STRING, optional: false, readonly: false },
    ]);
    let key_x = interner.literal_string("x");

    let result = evaluate_index_access(&interner, obj, key_x);
    assert_eq!(result, TypeId::NUMBER);
}

#[test]
fn test_index_access_object_string_key() {
    let interner = TypeInterner::new();

    // { x: number, y: string }["y"] -> string
    let obj = interner.object(vec![
        PropertyInfo { name: Arc::from("x"), type_id: TypeId::NUMBER, optional: false, readonly: false },
        PropertyInfo { name: Arc::from("y"), type_id: TypeId::STRING, optional: false, readonly: false },
    ]);
    let key_y = interner.literal_string("y");

    let result = evaluate_index_access(&interner, obj, key_y);
    assert_eq!(result, TypeId::STRING);
}

#[test]
fn test_index_access_object_missing_key() {
    let interner = TypeInterner::new();

    // { x: number }["z"] -> undefined
    let obj = interner.object(vec![
        PropertyInfo { name: Arc::from("x"), type_id: TypeId::NUMBER, optional: false, readonly: false },
    ]);
    let key_z = interner.literal_string("z");

    let result = evaluate_index_access(&interner, obj, key_z);
    assert_eq!(result, TypeId::UNDEFINED);
}

#[test]
fn test_index_access_object_union_key() {
    let interner = TypeInterner::new();

    // { x: number, y: string }["x" | "y"] -> number | string
    let obj = interner.object(vec![
        PropertyInfo { name: Arc::from("x"), type_id: TypeId::NUMBER, optional: false, readonly: false },
        PropertyInfo { name: Arc::from("y"), type_id: TypeId::STRING, optional: false, readonly: false },
    ]);
    let key_x = interner.literal_string("x");
    let key_y = interner.literal_string("y");
    let key_union = interner.union(vec![key_x, key_y]);

    let result = evaluate_index_access(&interner, obj, key_union);

    // Should be number | string
    let expected = interner.union(vec![TypeId::NUMBER, TypeId::STRING]);
    assert_eq!(result, expected);
}

#[test]
fn test_index_access_array() {
    let interner = TypeInterner::new();

    // string[][number] -> string
    let string_array = interner.array(TypeId::STRING);

    let result = evaluate_index_access(&interner, string_array, TypeId::NUMBER);
    assert_eq!(result, TypeId::STRING);
}

#[test]
fn test_index_access_tuple_literal() {
    let interner = TypeInterner::new();

    // [string, number][0] -> string
    let tuple = interner.tuple(vec![
        TupleElement { type_id: TypeId::STRING, name: None, optional: false, rest: false },
        TupleElement { type_id: TypeId::NUMBER, name: None, optional: false, rest: false },
    ]);
    let zero = interner.literal_number(0.0);

    let result = evaluate_index_access(&interner, tuple, zero);
    assert_eq!(result, TypeId::STRING);
}

#[test]
fn test_index_access_tuple_second() {
    let interner = TypeInterner::new();

    // [string, number][1] -> number
    let tuple = interner.tuple(vec![
        TupleElement { type_id: TypeId::STRING, name: None, optional: false, rest: false },
        TupleElement { type_id: TypeId::NUMBER, name: None, optional: false, rest: false },
    ]);
    let one = interner.literal_number(1.0);

    let result = evaluate_index_access(&interner, tuple, one);
    assert_eq!(result, TypeId::NUMBER);
}

#[test]
fn test_index_access_tuple_number() {
    let interner = TypeInterner::new();

    // [string, number][number] -> string | number
    let tuple = interner.tuple(vec![
        TupleElement { type_id: TypeId::STRING, name: None, optional: false, rest: false },
        TupleElement { type_id: TypeId::NUMBER, name: None, optional: false, rest: false },
    ]);

    let result = evaluate_index_access(&interner, tuple, TypeId::NUMBER);

    // Should be string | number
    let expected = interner.union(vec![TypeId::STRING, TypeId::NUMBER]);
    assert_eq!(result, expected);
}

#[test]
fn test_nested_conditional() {
    let interner = TypeInterner::new();

    // string extends string ? (number extends number ? "yes" : "no") : "outer-no"
    // Inner should resolve to "yes", so result is "yes"
    let yes = interner.literal_string("yes");
    let no = interner.literal_string("no");
    let outer_no = interner.literal_string("outer-no");

    let inner_cond = interner.intern(TypeKey::Conditional(Box::new(ConditionalType {
        check_type: TypeId::NUMBER,
        extends_type: TypeId::NUMBER,
        true_type: yes,
        false_type: no,
    })));

    let cond = ConditionalType {
        check_type: TypeId::STRING,
        extends_type: TypeId::STRING,
        true_type: inner_cond,
        false_type: outer_no,
    };

    let result = evaluate_conditional(&interner, &cond);

    // Inner conditional should also be evaluated -> "yes"
    assert_eq!(result, yes);
}

#[test]
fn test_evaluate_type_non_meta() {
    let interner = TypeInterner::new();

    // Non-meta types should pass through unchanged
    assert_eq!(evaluate_type(&interner, TypeId::STRING), TypeId::STRING);
    assert_eq!(evaluate_type(&interner, TypeId::NUMBER), TypeId::NUMBER);

    let obj = interner.object(vec![
        PropertyInfo { name: Arc::from("x"), type_id: TypeId::NUMBER, optional: false, readonly: false },
    ]);
    assert_eq!(evaluate_type(&interner, obj), obj);
}

// =============================================================================
// Keyof Tests
// =============================================================================

#[test]
fn test_keyof_object() {
    let interner = TypeInterner::new();

    // keyof { x: number, y: string } = "x" | "y"
    let obj = interner.object(vec![
        PropertyInfo { name: Arc::from("x"), type_id: TypeId::NUMBER, optional: false, readonly: false },
        PropertyInfo { name: Arc::from("y"), type_id: TypeId::STRING, optional: false, readonly: false },
    ]);

    let result = evaluate_keyof(&interner, obj);

    let key_x = interner.literal_string("x");
    let key_y = interner.literal_string("y");
    let expected = interner.union(vec![key_x, key_y]);
    assert_eq!(result, expected);
}

#[test]
fn test_keyof_empty_object() {
    let interner = TypeInterner::new();

    // keyof {} = never
    let obj = interner.object(vec![]);

    let result = evaluate_keyof(&interner, obj);
    assert_eq!(result, TypeId::NEVER);
}

#[test]
fn test_keyof_array() {
    let interner = TypeInterner::new();

    // keyof string[] = number (simplified)
    let arr = interner.array(TypeId::STRING);

    let result = evaluate_keyof(&interner, arr);
    assert_eq!(result, TypeId::NUMBER);
}

#[test]
fn test_keyof_tuple() {
    let interner = TypeInterner::new();

    // keyof [string, number] = "0" | "1"
    let tuple = interner.tuple(vec![
        TupleElement { type_id: TypeId::STRING, name: None, optional: false, rest: false },
        TupleElement { type_id: TypeId::NUMBER, name: None, optional: false, rest: false },
    ]);

    let result = evaluate_keyof(&interner, tuple);

    let key_0 = interner.literal_string("0");
    let key_1 = interner.literal_string("1");
    let expected = interner.union(vec![key_0, key_1]);
    assert_eq!(result, expected);
}

#[test]
fn test_keyof_any() {
    let interner = TypeInterner::new();

    // keyof any = string | number | symbol
    let result = evaluate_keyof(&interner, TypeId::ANY);

    let expected = interner.union(vec![TypeId::STRING, TypeId::NUMBER, TypeId::SYMBOL]);
    assert_eq!(result, expected);
}

#[test]
fn test_keyof_unknown() {
    let interner = TypeInterner::new();

    // keyof unknown = never
    let result = evaluate_keyof(&interner, TypeId::UNKNOWN);
    assert_eq!(result, TypeId::NEVER);
}

// =============================================================================
// Mapped Type Tests
// =============================================================================

#[test]
fn test_mapped_type_basic() {
    let interner = TypeInterner::new();

    // { [K in "x" | "y"]: number }
    // Should produce { x: number, y: number }
    let key_x = interner.literal_string("x");
    let key_y = interner.literal_string("y");
    let keys = interner.union(vec![key_x, key_y]);

    let mapped = MappedType {
        type_param: TypeParamInfo {
            name: Arc::from("K"),
            constraint: None,
            default: None,
        },
        constraint: keys,
        template: TypeId::NUMBER,
        readonly_modifier: None,
        optional_modifier: None,
    };

    let result = evaluate_mapped(&interner, &mapped);

    // Result should be { x: number, y: number }
    let expected = interner.object(vec![
        PropertyInfo { name: Arc::from("x"), type_id: TypeId::NUMBER, optional: false, readonly: false },
        PropertyInfo { name: Arc::from("y"), type_id: TypeId::NUMBER, optional: false, readonly: false },
    ]);
    assert_eq!(result, expected);
}

#[test]
fn test_mapped_type_single_key() {
    let interner = TypeInterner::new();

    // { [K in "foo"]: string }
    // Should produce { foo: string }
    let key_foo = interner.literal_string("foo");

    let mapped = MappedType {
        type_param: TypeParamInfo {
            name: Arc::from("K"),
            constraint: None,
            default: None,
        },
        constraint: key_foo,
        template: TypeId::STRING,
        readonly_modifier: None,
        optional_modifier: None,
    };

    let result = evaluate_mapped(&interner, &mapped);

    let expected = interner.object(vec![
        PropertyInfo { name: Arc::from("foo"), type_id: TypeId::STRING, optional: false, readonly: false },
    ]);
    assert_eq!(result, expected);
}

#[test]
fn test_mapped_type_with_optional_modifier() {
    let interner = TypeInterner::new();

    // { [K in "x" | "y"]?: number }
    // Should produce { x?: number, y?: number }
    let key_x = interner.literal_string("x");
    let key_y = interner.literal_string("y");
    let keys = interner.union(vec![key_x, key_y]);

    let mapped = MappedType {
        type_param: TypeParamInfo {
            name: Arc::from("K"),
            constraint: None,
            default: None,
        },
        constraint: keys,
        template: TypeId::NUMBER,
        readonly_modifier: None,
        optional_modifier: Some(MappedModifier::Add),
    };

    let result = evaluate_mapped(&interner, &mapped);

    // Result should be { x?: number, y?: number }
    let expected = interner.object(vec![
        PropertyInfo { name: Arc::from("x"), type_id: TypeId::NUMBER, optional: true, readonly: false },
        PropertyInfo { name: Arc::from("y"), type_id: TypeId::NUMBER, optional: true, readonly: false },
    ]);
    assert_eq!(result, expected);
}

#[test]
fn test_mapped_type_with_readonly_modifier() {
    let interner = TypeInterner::new();

    // { readonly [K in "x"]: number }
    // Should produce { readonly x: number }
    let key_x = interner.literal_string("x");

    let mapped = MappedType {
        type_param: TypeParamInfo {
            name: Arc::from("K"),
            constraint: None,
            default: None,
        },
        constraint: key_x,
        template: TypeId::NUMBER,
        readonly_modifier: Some(MappedModifier::Add),
        optional_modifier: None,
    };

    let result = evaluate_mapped(&interner, &mapped);

    let expected = interner.object(vec![
        PropertyInfo { name: Arc::from("x"), type_id: TypeId::NUMBER, optional: false, readonly: true },
    ]);
    assert_eq!(result, expected);
}

#[test]
fn test_mapped_type_with_template_substitution() {
    let interner = TypeInterner::new();

    // { [K in "x" | "y"]: K }
    // Should produce { x: "x", y: "y" }
    let key_x = interner.literal_string("x");
    let key_y = interner.literal_string("y");
    let keys = interner.union(vec![key_x, key_y]);

    // Template is the type parameter K itself
    let type_param_k = interner.intern(TypeKey::TypeParameter(TypeParamInfo {
        name: Arc::from("K"),
        constraint: None,
        default: None,
    }));

    let mapped = MappedType {
        type_param: TypeParamInfo {
            name: Arc::from("K"),
            constraint: None,
            default: None,
        },
        constraint: keys,
        template: type_param_k,
        readonly_modifier: None,
        optional_modifier: None,
    };

    let result = evaluate_mapped(&interner, &mapped);

    // Result should be { x: "x", y: "y" }
    let expected = interner.object(vec![
        PropertyInfo { name: Arc::from("x"), type_id: key_x, optional: false, readonly: false },
        PropertyInfo { name: Arc::from("y"), type_id: key_y, optional: false, readonly: false },
    ]);
    assert_eq!(result, expected);
}

#[test]
fn test_mapped_type_deferred() {
    let interner = TypeInterner::new();

    // { [K in T]: number } where T is a type parameter
    // Should remain as mapped type (deferred)
    let type_param_t = interner.intern(TypeKey::TypeParameter(TypeParamInfo {
        name: Arc::from("T"),
        constraint: None,
        default: None,
    }));

    let mapped = MappedType {
        type_param: TypeParamInfo {
            name: Arc::from("K"),
            constraint: None,
            default: None,
        },
        constraint: type_param_t,
        template: TypeId::NUMBER,
        readonly_modifier: None,
        optional_modifier: None,
    };

    let mapped_type = interner.intern(TypeKey::Mapped(Box::new(mapped.clone())));
    let result = evaluate_mapped(&interner, &mapped);

    // Should return the same mapped type (deferred)
    assert_eq!(result, mapped_type);
}
