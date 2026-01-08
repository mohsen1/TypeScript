use super::*;
use crate::solver::{instantiate_type, TypeSubstitution};

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
        is_distributive: false,
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
        is_distributive: false,
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
        is_distributive: false,
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
        is_distributive: true,
    };

    let result = evaluate_conditional(&interner, &cond);

    // Result should be true | false (i.e., boolean union of literals)
    let expected = interner.union(vec![lit_true, lit_false]);
    assert_eq!(result, expected);
}

#[test]
fn test_conditional_non_distributive_union() {
    let interner = TypeInterner::new();

    // (string | number) extends string ? true : false
    // Non-distributive: union is not a subtype of string, so false
    let string_or_number = interner.union(vec![TypeId::STRING, TypeId::NUMBER]);
    let lit_true = interner.literal_boolean(true);
    let lit_false = interner.literal_boolean(false);

    let cond = ConditionalType {
        check_type: string_or_number,
        extends_type: TypeId::STRING,
        true_type: lit_true,
        false_type: lit_false,
        is_distributive: false,
    };

    let result = evaluate_conditional(&interner, &cond);
    assert_eq!(result, lit_false);
}

#[test]
fn test_rest_unknown_bivariant_conditional_evaluate_strict() {
    let interner = TypeInterner::new();

    let rest_unknown = interner.array(TypeId::UNKNOWN);
    let target = interner.function(FunctionShape {
        params: vec![ParamInfo {
            name: None,
            type_id: rest_unknown,
            optional: false,
            rest: true,
        }],
        this_type: None,
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
        this_type: None,
        return_type: TypeId::VOID,
        type_params: Vec::new(),
        type_predicate: None,
        is_constructor: false,
    });

    let lit_true = interner.literal_boolean(true);
    let lit_false = interner.literal_boolean(false);
    let cond = ConditionalType {
        check_type: source,
        extends_type: target,
        true_type: lit_true,
        false_type: lit_false,
        is_distributive: false,
    };

    let result = evaluate_conditional(&interner, &cond);
    assert_eq!(result, lit_false);
}

#[test]
fn test_conditional_instantiated_param_distributes() {
    let interner = TypeInterner::new();

    let t_name = interner.intern_string("T");
    let t_param = interner.intern(TypeKey::TypeParameter(TypeParamInfo {
        name: t_name,
        constraint: None,
        default: None,
    }));

    let string_or_number = interner.union(vec![TypeId::STRING, TypeId::NUMBER]);
    let lit_true = interner.literal_boolean(true);
    let lit_false = interner.literal_boolean(false);

    // T extends string ? true : false, with T = string | number (distributive).
    let cond = ConditionalType {
        check_type: t_param,
        extends_type: TypeId::STRING,
        true_type: lit_true,
        false_type: lit_false,
        is_distributive: true,
    };

    let cond_type = interner.conditional(cond);
    let mut subst = TypeSubstitution::new();
    subst.insert(t_name, string_or_number);

    let instantiated = instantiate_type(&interner, cond_type, &subst);
    let result = evaluate_type(&interner, instantiated);
    let expected = interner.union(vec![lit_true, lit_false]);

    assert_eq!(result, expected);
}

#[test]
fn test_conditional_instantiated_param_distributes_branch_substitution() {
    let interner = TypeInterner::new();

    let t_name = interner.intern_string("T");
    let t_param = interner.intern(TypeKey::TypeParameter(TypeParamInfo {
        name: t_name,
        constraint: None,
        default: None,
    }));

    // T extends string ? T : never, with T = string | number
    // Distributes to: (string extends string ? string : never) |
    //                 (number extends string ? number : never)
    // Result should be string.
    let cond = ConditionalType {
        check_type: t_param,
        extends_type: TypeId::STRING,
        true_type: t_param,
        false_type: TypeId::NEVER,
        is_distributive: true,
    };

    let cond_type = interner.conditional(cond);
    let string_or_number = interner.union(vec![TypeId::STRING, TypeId::NUMBER]);
    let mut subst = TypeSubstitution::new();
    subst.insert(t_name, string_or_number);

    let instantiated = instantiate_type(&interner, cond_type, &subst);
    let result = evaluate_type(&interner, instantiated);

    assert_eq!(result, TypeId::STRING);
}

#[test]
fn test_conditional_distributive_nested_extends() {
    let interner = TypeInterner::new();

    let t_name = interner.intern_string("T");
    let t_param = interner.intern(TypeKey::TypeParameter(TypeParamInfo {
        name: t_name,
        constraint: None,
        default: None,
    }));

    // T extends string ? (T extends "a" ? 1 : 2) : 3, with T = "a" | "b"
    // Distributes to 1 | 2.
    let lit_a = interner.literal_string("a");
    let lit_b = interner.literal_string("b");
    let lit_one = interner.literal_number(1.0);
    let lit_two = interner.literal_number(2.0);
    let lit_three = interner.literal_number(3.0);

    let inner_cond = interner.conditional(ConditionalType {
        check_type: t_param,
        extends_type: lit_a,
        true_type: lit_one,
        false_type: lit_two,
        is_distributive: false,
    });

    let outer_cond = ConditionalType {
        check_type: t_param,
        extends_type: TypeId::STRING,
        true_type: inner_cond,
        false_type: lit_three,
        is_distributive: true,
    };

    let cond_type = interner.conditional(outer_cond);
    let mut subst = TypeSubstitution::new();
    subst.insert(t_name, interner.union(vec![lit_a, lit_b]));

    let instantiated = instantiate_type(&interner, cond_type, &subst);
    let result = evaluate_type(&interner, instantiated);
    let expected = interner.union(vec![lit_one, lit_two]);

    assert_eq!(result, expected);
}

#[test]
fn test_conditional_distributive_infer_extends_nested() {
    let interner = TypeInterner::new();

    let t_name = interner.intern_string("T");
    let t_param = interner.intern(TypeKey::TypeParameter(TypeParamInfo {
        name: t_name,
        constraint: None,
        default: None,
    }));

    let infer_name = interner.intern_string("R");
    let infer_r = interner.intern(TypeKey::Infer(TypeParamInfo {
        name: infer_name,
        constraint: Some(TypeId::STRING),
        default: None,
    }));

    // T extends infer R extends string ? (R extends "a" ? "yes" : "no") : "fallback"
    // with T = "a" | "b" | number.
    let lit_a = interner.literal_string("a");
    let lit_yes = interner.literal_string("yes");
    let lit_no = interner.literal_string("no");
    let lit_fallback = interner.literal_string("fallback");

    let inner_cond = interner.conditional(ConditionalType {
        check_type: infer_r,
        extends_type: lit_a,
        true_type: lit_yes,
        false_type: lit_no,
        is_distributive: false,
    });

    let outer_cond = ConditionalType {
        check_type: t_param,
        extends_type: infer_r,
        true_type: inner_cond,
        false_type: lit_fallback,
        is_distributive: true,
    };

    let cond_type = interner.conditional(outer_cond);
    let mut subst = TypeSubstitution::new();
    subst.insert(
        t_name,
        interner.union(vec![lit_a, interner.literal_string("b"), TypeId::NUMBER]),
    );

    let instantiated = instantiate_type(&interner, cond_type, &subst);
    let result = evaluate_type(&interner, instantiated);
    let expected = interner.union(vec![lit_yes, lit_no, lit_fallback]);

    assert_eq!(result, expected);
}

#[test]
fn test_conditional_infer_true_branch_substitution() {
    let interner = TypeInterner::new();

    let infer_name = interner.intern_string("R");
    let infer_r = interner.intern(TypeKey::Infer(TypeParamInfo {
        name: infer_name,
        constraint: Some(TypeId::STRING),
        default: None,
    }));

    // "a" extends infer R extends string ? R : never
    let lit_a = interner.literal_string("a");
    let cond = ConditionalType {
        check_type: lit_a,
        extends_type: infer_r,
        true_type: infer_r,
        false_type: TypeId::NEVER,
        is_distributive: false,
    };

    let result = evaluate_conditional(&interner, &cond);
    assert_eq!(result, lit_a);
}

#[test]
fn test_conditional_infer_false_branch_substitution() {
    let interner = TypeInterner::new();

    let infer_name = interner.intern_string("R");
    let infer_r = interner.intern(TypeKey::Infer(TypeParamInfo {
        name: infer_name,
        constraint: Some(TypeId::STRING),
        default: None,
    }));

    // number extends infer R extends string ? string : R
    let cond = ConditionalType {
        check_type: TypeId::NUMBER,
        extends_type: infer_r,
        true_type: TypeId::STRING,
        false_type: infer_r,
        is_distributive: false,
    };

    let result = evaluate_conditional(&interner, &cond);
    assert_eq!(result, TypeId::NUMBER);
}

#[test]
fn test_conditional_infer_array_element_extraction() {
    let interner = TypeInterner::new();

    let t_name = interner.intern_string("T");
    let t_param = interner.intern(TypeKey::TypeParameter(TypeParamInfo {
        name: t_name,
        constraint: None,
        default: None,
    }));

    let infer_name = interner.intern_string("R");
    let infer_r = interner.intern(TypeKey::Infer(TypeParamInfo {
        name: infer_name,
        constraint: None,
        default: None,
    }));

    // T extends (infer R)[] ? R : never, with T = string[] | number[].
    let extends_array = interner.array(infer_r);
    let cond = ConditionalType {
        check_type: t_param,
        extends_type: extends_array,
        true_type: infer_r,
        false_type: TypeId::NEVER,
        is_distributive: true,
    };

    let cond_type = interner.conditional(cond);
    let mut subst = TypeSubstitution::new();
    subst.insert(
        t_name,
        interner.union(vec![
            interner.array(TypeId::STRING),
            interner.array(TypeId::NUMBER),
        ]),
    );

    let instantiated = instantiate_type(&interner, cond_type, &subst);
    let result = evaluate_type(&interner, instantiated);
    let expected = interner.union(vec![TypeId::STRING, TypeId::NUMBER]);

    assert_eq!(result, expected);
}

#[test]
fn test_conditional_infer_array_element_from_tuple_rest() {
    let interner = TypeInterner::new();

    let t_name = interner.intern_string("T");
    let t_param = interner.intern(TypeKey::TypeParameter(TypeParamInfo {
        name: t_name,
        constraint: None,
        default: None,
    }));

    let infer_name = interner.intern_string("R");
    let infer_r = interner.intern(TypeKey::Infer(TypeParamInfo {
        name: infer_name,
        constraint: None,
        default: None,
    }));

    // T extends (infer R)[] ? R : never, with T = [string, ...number[]].
    let extends_array = interner.array(infer_r);
    let cond = ConditionalType {
        check_type: t_param,
        extends_type: extends_array,
        true_type: infer_r,
        false_type: TypeId::NEVER,
        is_distributive: true,
    };

    let cond_type = interner.conditional(cond);
    let number_array = interner.array(TypeId::NUMBER);
    let tuple = interner.tuple(vec![
        TupleElement {
            type_id: TypeId::STRING,
            name: None,
            optional: false,
            rest: false,
        },
        TupleElement {
            type_id: number_array,
            name: None,
            optional: false,
            rest: true,
        },
    ]);
    let mut subst = TypeSubstitution::new();
    subst.insert(t_name, tuple);

    let instantiated = instantiate_type(&interner, cond_type, &subst);
    let result = evaluate_type(&interner, instantiated);
    let expected = interner.union(vec![TypeId::STRING, TypeId::NUMBER]);

    assert_eq!(result, expected);
}

#[test]
fn test_conditional_infer_array_element_from_tuple_rest_tuple() {
    let interner = TypeInterner::new();

    let t_name = interner.intern_string("T");
    let t_param = interner.intern(TypeKey::TypeParameter(TypeParamInfo {
        name: t_name,
        constraint: None,
        default: None,
    }));

    let infer_name = interner.intern_string("R");
    let infer_r = interner.intern(TypeKey::Infer(TypeParamInfo {
        name: infer_name,
        constraint: None,
        default: None,
    }));

    // T extends (infer R)[] ? R : never, with T = [string, ...[number, boolean]].
    // TODO: Current inference keeps the rest tuple as a single element (string | [number, boolean]).
    let extends_array = interner.array(infer_r);
    let cond = ConditionalType {
        check_type: t_param,
        extends_type: extends_array,
        true_type: infer_r,
        false_type: TypeId::NEVER,
        is_distributive: true,
    };

    let cond_type = interner.conditional(cond);
    let rest_tuple = interner.tuple(vec![
        TupleElement {
            type_id: TypeId::NUMBER,
            name: None,
            optional: false,
            rest: false,
        },
        TupleElement {
            type_id: TypeId::BOOLEAN,
            name: None,
            optional: false,
            rest: false,
        },
    ]);
    let tuple = interner.tuple(vec![
        TupleElement {
            type_id: TypeId::STRING,
            name: None,
            optional: false,
            rest: false,
        },
        TupleElement {
            type_id: rest_tuple,
            name: None,
            optional: false,
            rest: true,
        },
    ]);
    let mut subst = TypeSubstitution::new();
    subst.insert(t_name, tuple);

    let instantiated = instantiate_type(&interner, cond_type, &subst);
    let result = evaluate_type(&interner, instantiated);
    let expected = interner.union(vec![TypeId::STRING, rest_tuple]);

    assert_eq!(result, expected);
}

#[test]
fn test_conditional_infer_array_element_with_constraint() {
    let interner = TypeInterner::new();

    let t_name = interner.intern_string("T");
    let t_param = interner.intern(TypeKey::TypeParameter(TypeParamInfo {
        name: t_name,
        constraint: None,
        default: None,
    }));

    let infer_name = interner.intern_string("R");
    let infer_r = interner.intern(TypeKey::Infer(TypeParamInfo {
        name: infer_name,
        constraint: Some(TypeId::STRING),
        default: None,
    }));

    // T extends (infer R extends string)[] ? R : never, with T = number[] | string[].
    let extends_array = interner.array(infer_r);
    let cond = ConditionalType {
        check_type: t_param,
        extends_type: extends_array,
        true_type: infer_r,
        false_type: TypeId::NEVER,
        is_distributive: true,
    };

    let cond_type = interner.conditional(cond);
    let mut subst = TypeSubstitution::new();
    subst.insert(
        t_name,
        interner.union(vec![
            interner.array(TypeId::NUMBER),
            interner.array(TypeId::STRING),
        ]),
    );

    let instantiated = instantiate_type(&interner, cond_type, &subst);
    let result = evaluate_type(&interner, instantiated);

    assert_eq!(result, TypeId::STRING);
}

#[test]
fn test_conditional_infer_array_element_non_distributive() {
    let interner = TypeInterner::new();

    let t_name = interner.intern_string("T");
    let t_param = interner.intern(TypeKey::TypeParameter(TypeParamInfo {
        name: t_name,
        constraint: None,
        default: None,
    }));

    let infer_name = interner.intern_string("R");
    let infer_r = interner.intern(TypeKey::Infer(TypeParamInfo {
        name: infer_name,
        constraint: None,
        default: None,
    }));

    // (T[]) extends (infer R)[] ? R : never, with T = string | number (no distribution).
    let check_array = interner.array(t_param);
    let extends_array = interner.array(infer_r);
    let cond = ConditionalType {
        check_type: check_array,
        extends_type: extends_array,
        true_type: infer_r,
        false_type: TypeId::NEVER,
        is_distributive: false,
    };

    let cond_type = interner.conditional(cond);
    let mut subst = TypeSubstitution::new();
    subst.insert(t_name, interner.union(vec![TypeId::STRING, TypeId::NUMBER]));

    let instantiated = instantiate_type(&interner, cond_type, &subst);
    let result = evaluate_type(&interner, instantiated);
    let expected = interner.union(vec![TypeId::STRING, TypeId::NUMBER]);

    assert_eq!(result, expected);
}

#[test]
fn test_conditional_infer_object_property_distributive() {
    let interner = TypeInterner::new();

    let t_name = interner.intern_string("T");
    let t_param = interner.intern(TypeKey::TypeParameter(TypeParamInfo {
        name: t_name,
        constraint: None,
        default: None,
    }));

    let infer_name = interner.intern_string("R");
    let infer_r = interner.intern(TypeKey::Infer(TypeParamInfo {
        name: infer_name,
        constraint: None,
        default: None,
    }));

    // T extends { a: infer R } ? R : never, with T = { a: string } | { a: number } | { b: boolean }.
    let extends_obj = interner.object(vec![PropertyInfo {
        name: interner.intern_string("a"),
        type_id: infer_r,
        write_type: infer_r,
        optional: false,
        readonly: false,
        is_method: false,
    }]);
    let cond = ConditionalType {
        check_type: t_param,
        extends_type: extends_obj,
        true_type: infer_r,
        false_type: TypeId::NEVER,
        is_distributive: true,
    };

    let cond_type = interner.conditional(cond);
    let mut subst = TypeSubstitution::new();
    let obj_a = interner.object(vec![PropertyInfo {
        name: interner.intern_string("a"),
        type_id: TypeId::STRING,
        write_type: TypeId::STRING,
        optional: false,
        readonly: false,
        is_method: false,
    }]);
    let obj_b = interner.object(vec![PropertyInfo {
        name: interner.intern_string("a"),
        type_id: TypeId::NUMBER,
        write_type: TypeId::NUMBER,
        optional: false,
        readonly: false,
        is_method: false,
    }]);
    let obj_c = interner.object(vec![PropertyInfo {
        name: interner.intern_string("b"),
        type_id: TypeId::BOOLEAN,
        write_type: TypeId::BOOLEAN,
        optional: false,
        readonly: false,
        is_method: false,
    }]);
    subst.insert(t_name, interner.union(vec![obj_a, obj_b, obj_c]));

    let instantiated = instantiate_type(&interner, cond_type, &subst);
    let result = evaluate_type(&interner, instantiated);
    let expected = interner.union(vec![TypeId::STRING, TypeId::NUMBER]);

    assert_eq!(result, expected);
}

#[test]
fn test_conditional_infer_object_property_with_constraint() {
    let interner = TypeInterner::new();

    let t_name = interner.intern_string("T");
    let t_param = interner.intern(TypeKey::TypeParameter(TypeParamInfo {
        name: t_name,
        constraint: None,
        default: None,
    }));

    let infer_name = interner.intern_string("R");
    let infer_r = interner.intern(TypeKey::Infer(TypeParamInfo {
        name: infer_name,
        constraint: Some(TypeId::STRING),
        default: None,
    }));

    // T extends { a: infer R extends string } ? R : never, with T = { a: string } | { a: number }.
    let extends_obj = interner.object(vec![PropertyInfo {
        name: interner.intern_string("a"),
        type_id: infer_r,
        write_type: infer_r,
        optional: false,
        readonly: false,
        is_method: false,
    }]);
    let cond = ConditionalType {
        check_type: t_param,
        extends_type: extends_obj,
        true_type: infer_r,
        false_type: TypeId::NEVER,
        is_distributive: true,
    };

    let cond_type = interner.conditional(cond);
    let mut subst = TypeSubstitution::new();
    let obj_a = interner.object(vec![PropertyInfo {
        name: interner.intern_string("a"),
        type_id: TypeId::STRING,
        write_type: TypeId::STRING,
        optional: false,
        readonly: false,
        is_method: false,
    }]);
    let obj_b = interner.object(vec![PropertyInfo {
        name: interner.intern_string("a"),
        type_id: TypeId::NUMBER,
        write_type: TypeId::NUMBER,
        optional: false,
        readonly: false,
        is_method: false,
    }]);
    subst.insert(t_name, interner.union(vec![obj_a, obj_b]));

    let instantiated = instantiate_type(&interner, cond_type, &subst);
    let result = evaluate_type(&interner, instantiated);

    assert_eq!(result, TypeId::STRING);
}

#[test]
fn test_conditional_infer_nested_object_property_distributive() {
    let interner = TypeInterner::new();

    let t_name = interner.intern_string("T");
    let t_param = interner.intern(TypeKey::TypeParameter(TypeParamInfo {
        name: t_name,
        constraint: None,
        default: None,
    }));

    let infer_name = interner.intern_string("R");
    let infer_r = interner.intern(TypeKey::Infer(TypeParamInfo {
        name: infer_name,
        constraint: None,
        default: None,
    }));

    // T extends { a: { b: infer R } } ? R : never, with T = { a: { b: string } } | { a: { b: number } }.
    let extends_inner = interner.object(vec![PropertyInfo {
        name: interner.intern_string("b"),
        type_id: infer_r,
        write_type: infer_r,
        optional: false,
        readonly: false,
        is_method: false,
    }]);
    let extends_obj = interner.object(vec![PropertyInfo {
        name: interner.intern_string("a"),
        type_id: extends_inner,
        write_type: extends_inner,
        optional: false,
        readonly: false,
        is_method: false,
    }]);
    let cond = ConditionalType {
        check_type: t_param,
        extends_type: extends_obj,
        true_type: infer_r,
        false_type: TypeId::NEVER,
        is_distributive: true,
    };

    let cond_type = interner.conditional(cond);
    let mut subst = TypeSubstitution::new();
    let obj_a_string = interner.object(vec![PropertyInfo {
        name: interner.intern_string("b"),
        type_id: TypeId::STRING,
        write_type: TypeId::STRING,
        optional: false,
        readonly: false,
        is_method: false,
    }]);
    let obj_a_number = interner.object(vec![PropertyInfo {
        name: interner.intern_string("b"),
        type_id: TypeId::NUMBER,
        write_type: TypeId::NUMBER,
        optional: false,
        readonly: false,
        is_method: false,
    }]);
    let obj_string = interner.object(vec![PropertyInfo {
        name: interner.intern_string("a"),
        type_id: obj_a_string,
        write_type: obj_a_string,
        optional: false,
        readonly: false,
        is_method: false,
    }]);
    let obj_number = interner.object(vec![PropertyInfo {
        name: interner.intern_string("a"),
        type_id: obj_a_number,
        write_type: obj_a_number,
        optional: false,
        readonly: false,
        is_method: false,
    }]);
    subst.insert(t_name, interner.union(vec![obj_string, obj_number]));

    let instantiated = instantiate_type(&interner, cond_type, &subst);
    let result = evaluate_type(&interner, instantiated);
    let expected = interner.union(vec![TypeId::STRING, TypeId::NUMBER]);

    assert_eq!(result, expected);
}

#[test]
fn test_conditional_infer_nested_object_property_with_constraint() {
    let interner = TypeInterner::new();

    let t_name = interner.intern_string("T");
    let t_param = interner.intern(TypeKey::TypeParameter(TypeParamInfo {
        name: t_name,
        constraint: None,
        default: None,
    }));

    let infer_name = interner.intern_string("R");
    let infer_r = interner.intern(TypeKey::Infer(TypeParamInfo {
        name: infer_name,
        constraint: Some(TypeId::STRING),
        default: None,
    }));

    // T extends { a: { b: infer R extends string } } ? R : never, with T = { a: { b: string } } | { a: { b: number } }.
    let extends_inner = interner.object(vec![PropertyInfo {
        name: interner.intern_string("b"),
        type_id: infer_r,
        write_type: infer_r,
        optional: false,
        readonly: false,
        is_method: false,
    }]);
    let extends_obj = interner.object(vec![PropertyInfo {
        name: interner.intern_string("a"),
        type_id: extends_inner,
        write_type: extends_inner,
        optional: false,
        readonly: false,
        is_method: false,
    }]);
    let cond = ConditionalType {
        check_type: t_param,
        extends_type: extends_obj,
        true_type: infer_r,
        false_type: TypeId::NEVER,
        is_distributive: true,
    };

    let cond_type = interner.conditional(cond);
    let mut subst = TypeSubstitution::new();
    let obj_a_string = interner.object(vec![PropertyInfo {
        name: interner.intern_string("b"),
        type_id: TypeId::STRING,
        write_type: TypeId::STRING,
        optional: false,
        readonly: false,
        is_method: false,
    }]);
    let obj_a_number = interner.object(vec![PropertyInfo {
        name: interner.intern_string("b"),
        type_id: TypeId::NUMBER,
        write_type: TypeId::NUMBER,
        optional: false,
        readonly: false,
        is_method: false,
    }]);
    let obj_string = interner.object(vec![PropertyInfo {
        name: interner.intern_string("a"),
        type_id: obj_a_string,
        write_type: obj_a_string,
        optional: false,
        readonly: false,
        is_method: false,
    }]);
    let obj_number = interner.object(vec![PropertyInfo {
        name: interner.intern_string("a"),
        type_id: obj_a_number,
        write_type: obj_a_number,
        optional: false,
        readonly: false,
        is_method: false,
    }]);
    subst.insert(t_name, interner.union(vec![obj_string, obj_number]));

    let instantiated = instantiate_type(&interner, cond_type, &subst);
    let result = evaluate_type(&interner, instantiated);

    assert_eq!(result, TypeId::STRING);
}

#[test]
fn test_conditional_infer_nested_object_property_readonly() {
    let interner = TypeInterner::new();

    let t_name = interner.intern_string("T");
    let t_param = interner.intern(TypeKey::TypeParameter(TypeParamInfo {
        name: t_name,
        constraint: None,
        default: None,
    }));

    let infer_name = interner.intern_string("R");
    let infer_r = interner.intern(TypeKey::Infer(TypeParamInfo {
        name: infer_name,
        constraint: None,
        default: None,
    }));

    // T extends { readonly a: { b: infer R } } ? R : never, with T = { readonly a: { b: string } } | { a: { b: number } }.
    let extends_inner = interner.object(vec![PropertyInfo {
        name: interner.intern_string("b"),
        type_id: infer_r,
        write_type: infer_r,
        optional: false,
        readonly: false,
        is_method: false,
    }]);
    let extends_obj = interner.object(vec![PropertyInfo {
        name: interner.intern_string("a"),
        type_id: extends_inner,
        write_type: extends_inner,
        optional: false,
        readonly: true,
        is_method: false,
    }]);
    let cond = ConditionalType {
        check_type: t_param,
        extends_type: extends_obj,
        true_type: infer_r,
        false_type: TypeId::NEVER,
        is_distributive: true,
    };

    let cond_type = interner.conditional(cond);
    let mut subst = TypeSubstitution::new();
    let obj_a_string = interner.object(vec![PropertyInfo {
        name: interner.intern_string("b"),
        type_id: TypeId::STRING,
        write_type: TypeId::STRING,
        optional: false,
        readonly: false,
        is_method: false,
    }]);
    let obj_a_number = interner.object(vec![PropertyInfo {
        name: interner.intern_string("b"),
        type_id: TypeId::NUMBER,
        write_type: TypeId::NUMBER,
        optional: false,
        readonly: false,
        is_method: false,
    }]);
    let obj_string = interner.object(vec![PropertyInfo {
        name: interner.intern_string("a"),
        type_id: obj_a_string,
        write_type: obj_a_string,
        optional: false,
        readonly: true,
        is_method: false,
    }]);
    let obj_number = interner.object(vec![PropertyInfo {
        name: interner.intern_string("a"),
        type_id: obj_a_number,
        write_type: obj_a_number,
        optional: false,
        readonly: false,
        is_method: false,
    }]);
    subst.insert(t_name, interner.union(vec![obj_string, obj_number]));

    let instantiated = instantiate_type(&interner, cond_type, &subst);
    let result = evaluate_type(&interner, instantiated);
    let expected = interner.union(vec![TypeId::STRING, TypeId::NUMBER]);

    assert_eq!(result, expected);
}

#[test]
fn test_conditional_infer_nested_object_property_readonly_wrapper() {
    let interner = TypeInterner::new();

    let t_name = interner.intern_string("T");
    let t_param = interner.intern(TypeKey::TypeParameter(TypeParamInfo {
        name: t_name,
        constraint: None,
        default: None,
    }));

    let infer_name = interner.intern_string("R");
    let infer_r = interner.intern(TypeKey::Infer(TypeParamInfo {
        name: infer_name,
        constraint: None,
        default: None,
    }));

    // T extends { a: Readonly<{ b: infer R }> } ? R : never,
    // with T = { a: Readonly<{ b: string }> } | { a: { b: number } }.
    let extends_inner_obj = interner.object(vec![PropertyInfo {
        name: interner.intern_string("b"),
        type_id: infer_r,
        write_type: infer_r,
        optional: false,
        readonly: false,
        is_method: false,
    }]);
    let extends_inner =
        interner.intern(TypeKey::ReadonlyType(extends_inner_obj));
    let extends_obj = interner.object(vec![PropertyInfo {
        name: interner.intern_string("a"),
        type_id: extends_inner,
        write_type: extends_inner,
        optional: false,
        readonly: false,
        is_method: false,
    }]);
    let cond = ConditionalType {
        check_type: t_param,
        extends_type: extends_obj,
        true_type: infer_r,
        false_type: TypeId::NEVER,
        is_distributive: true,
    };

    let cond_type = interner.conditional(cond);
    let mut subst = TypeSubstitution::new();
    let obj_a_string_inner = interner.object(vec![PropertyInfo {
        name: interner.intern_string("b"),
        type_id: TypeId::STRING,
        write_type: TypeId::STRING,
        optional: false,
        readonly: false,
        is_method: false,
    }]);
    let obj_a_string =
        interner.intern(TypeKey::ReadonlyType(obj_a_string_inner));
    let obj_a_number = interner.object(vec![PropertyInfo {
        name: interner.intern_string("b"),
        type_id: TypeId::NUMBER,
        write_type: TypeId::NUMBER,
        optional: false,
        readonly: false,
        is_method: false,
    }]);
    let obj_string = interner.object(vec![PropertyInfo {
        name: interner.intern_string("a"),
        type_id: obj_a_string,
        write_type: obj_a_string,
        optional: false,
        readonly: false,
        is_method: false,
    }]);
    let obj_number = interner.object(vec![PropertyInfo {
        name: interner.intern_string("a"),
        type_id: obj_a_number,
        write_type: obj_a_number,
        optional: false,
        readonly: false,
        is_method: false,
    }]);
    subst.insert(t_name, interner.union(vec![obj_string, obj_number]));

    let instantiated = instantiate_type(&interner, cond_type, &subst);
    let result = evaluate_type(&interner, instantiated);
    let expected = interner.union(vec![TypeId::STRING, TypeId::NUMBER]);

    assert_eq!(result, expected);
}

#[test]
fn test_conditional_infer_nested_object_property_non_matching_branch() {
    let interner = TypeInterner::new();

    let t_name = interner.intern_string("T");
    let t_param = interner.intern(TypeKey::TypeParameter(TypeParamInfo {
        name: t_name,
        constraint: None,
        default: None,
    }));

    let infer_name = interner.intern_string("R");
    let infer_r = interner.intern(TypeKey::Infer(TypeParamInfo {
        name: infer_name,
        constraint: None,
        default: None,
    }));

    // T extends { a: { b: infer R } } ? R : never, with T = { a: { b: string } } | { a: { c: number } }.
    let extends_inner = interner.object(vec![PropertyInfo {
        name: interner.intern_string("b"),
        type_id: infer_r,
        write_type: infer_r,
        optional: false,
        readonly: false,
        is_method: false,
    }]);
    let extends_obj = interner.object(vec![PropertyInfo {
        name: interner.intern_string("a"),
        type_id: extends_inner,
        write_type: extends_inner,
        optional: false,
        readonly: false,
        is_method: false,
    }]);
    let cond = ConditionalType {
        check_type: t_param,
        extends_type: extends_obj,
        true_type: infer_r,
        false_type: TypeId::NEVER,
        is_distributive: true,
    };

    let cond_type = interner.conditional(cond);
    let mut subst = TypeSubstitution::new();
    let obj_a_string = interner.object(vec![PropertyInfo {
        name: interner.intern_string("b"),
        type_id: TypeId::STRING,
        write_type: TypeId::STRING,
        optional: false,
        readonly: false,
        is_method: false,
    }]);
    let obj_a_number = interner.object(vec![PropertyInfo {
        name: interner.intern_string("c"),
        type_id: TypeId::NUMBER,
        write_type: TypeId::NUMBER,
        optional: false,
        readonly: false,
        is_method: false,
    }]);
    let obj_match = interner.object(vec![PropertyInfo {
        name: interner.intern_string("a"),
        type_id: obj_a_string,
        write_type: obj_a_string,
        optional: false,
        readonly: false,
        is_method: false,
    }]);
    let obj_non_match = interner.object(vec![PropertyInfo {
        name: interner.intern_string("a"),
        type_id: obj_a_number,
        write_type: obj_a_number,
        optional: false,
        readonly: false,
        is_method: false,
    }]);
    subst.insert(t_name, interner.union(vec![obj_match, obj_non_match]));

    let instantiated = instantiate_type(&interner, cond_type, &subst);
    let result = evaluate_type(&interner, instantiated);

    assert_eq!(result, TypeId::STRING);
}

#[test]
fn test_conditional_infer_nested_object_property_union_value() {
    let interner = TypeInterner::new();

    let t_name = interner.intern_string("T");
    let t_param = interner.intern(TypeKey::TypeParameter(TypeParamInfo {
        name: t_name,
        constraint: None,
        default: None,
    }));

    let infer_name = interner.intern_string("R");
    let infer_r = interner.intern(TypeKey::Infer(TypeParamInfo {
        name: infer_name,
        constraint: None,
        default: None,
    }));

    // T extends { a: { b: infer R } } ? R : never, with T = { a: { b: string | number } }.
    let extends_inner = interner.object(vec![PropertyInfo {
        name: interner.intern_string("b"),
        type_id: infer_r,
        write_type: infer_r,
        optional: false,
        readonly: false,
        is_method: false,
    }]);
    let extends_obj = interner.object(vec![PropertyInfo {
        name: interner.intern_string("a"),
        type_id: extends_inner,
        write_type: extends_inner,
        optional: false,
        readonly: false,
        is_method: false,
    }]);
    let cond = ConditionalType {
        check_type: t_param,
        extends_type: extends_obj,
        true_type: infer_r,
        false_type: TypeId::NEVER,
        is_distributive: true,
    };

    let cond_type = interner.conditional(cond);
    let mut subst = TypeSubstitution::new();
    let b_union = interner.union(vec![TypeId::STRING, TypeId::NUMBER]);
    let obj_a = interner.object(vec![PropertyInfo {
        name: interner.intern_string("b"),
        type_id: b_union,
        write_type: b_union,
        optional: false,
        readonly: false,
        is_method: false,
    }]);
    let obj = interner.object(vec![PropertyInfo {
        name: interner.intern_string("a"),
        type_id: obj_a,
        write_type: obj_a,
        optional: false,
        readonly: false,
        is_method: false,
    }]);
    subst.insert(t_name, obj);

    let instantiated = instantiate_type(&interner, cond_type, &subst);
    let result = evaluate_type(&interner, instantiated);

    assert_eq!(result, b_union);
}

#[test]
fn test_conditional_infer_object_property_non_object_union_branch() {
    let interner = TypeInterner::new();

    let t_name = interner.intern_string("T");
    let t_param = interner.intern(TypeKey::TypeParameter(TypeParamInfo {
        name: t_name,
        constraint: None,
        default: None,
    }));

    let infer_name = interner.intern_string("R");
    let infer_r = interner.intern(TypeKey::Infer(TypeParamInfo {
        name: infer_name,
        constraint: None,
        default: None,
    }));

    // T extends { a: infer R } ? R : never, with T = { a: string } | number.
    let extends_obj = interner.object(vec![PropertyInfo {
        name: interner.intern_string("a"),
        type_id: infer_r,
        write_type: infer_r,
        optional: false,
        readonly: false,
        is_method: false,
    }]);
    let cond = ConditionalType {
        check_type: t_param,
        extends_type: extends_obj,
        true_type: infer_r,
        false_type: TypeId::NEVER,
        is_distributive: true,
    };

    let cond_type = interner.conditional(cond);
    let mut subst = TypeSubstitution::new();
    let obj_match = interner.object(vec![PropertyInfo {
        name: interner.intern_string("a"),
        type_id: TypeId::STRING,
        write_type: TypeId::STRING,
        optional: false,
        readonly: false,
        is_method: false,
    }]);
    subst.insert(t_name, interner.union(vec![obj_match, TypeId::NUMBER]));

    let instantiated = instantiate_type(&interner, cond_type, &subst);
    let result = evaluate_type(&interner, instantiated);

    assert_eq!(result, TypeId::STRING);
}

#[test]
fn test_conditional_infer_object_index_signature_distributive() {
    let interner = TypeInterner::new();

    let t_name = interner.intern_string("T");
    let t_param = interner.intern(TypeKey::TypeParameter(TypeParamInfo {
        name: t_name,
        constraint: None,
        default: None,
    }));

    let infer_name = interner.intern_string("R");
    let infer_r = interner.intern(TypeKey::Infer(TypeParamInfo {
        name: infer_name,
        constraint: None,
        default: None,
    }));

    // T extends { [key: string]: infer R } ? R : never, with T = { a: string } | { b: number }.
    // TODO: TypeScript infers R as string | number; solver currently yields never (no index signature inference).
    let extends_obj = interner.object_with_index(ObjectShape {
        properties: Vec::new(),
        string_index: Some(IndexSignature {
            key_type: TypeId::STRING,
            value_type: infer_r,
            readonly: false,
        }),
        number_index: None,
    });
    let cond = ConditionalType {
        check_type: t_param,
        extends_type: extends_obj,
        true_type: infer_r,
        false_type: TypeId::NEVER,
        is_distributive: true,
    };

    let cond_type = interner.conditional(cond);
    let mut subst = TypeSubstitution::new();
    let obj_string = interner.object(vec![PropertyInfo {
        name: interner.intern_string("a"),
        type_id: TypeId::STRING,
        write_type: TypeId::STRING,
        optional: false,
        readonly: false,
        is_method: false,
    }]);
    let obj_number = interner.object(vec![PropertyInfo {
        name: interner.intern_string("b"),
        type_id: TypeId::NUMBER,
        write_type: TypeId::NUMBER,
        optional: false,
        readonly: false,
        is_method: false,
    }]);
    subst.insert(t_name, interner.union(vec![obj_string, obj_number]));

    let instantiated = instantiate_type(&interner, cond_type, &subst);
    let result = evaluate_type(&interner, instantiated);

    assert_eq!(result, TypeId::NEVER);
}

#[test]
fn test_conditional_infer_optional_property_missing_object() {
    let interner = TypeInterner::new();

    let t_name = interner.intern_string("T");
    let t_param = interner.intern(TypeKey::TypeParameter(TypeParamInfo {
        name: t_name,
        constraint: None,
        default: None,
    }));

    let infer_name = interner.intern_string("R");
    let infer_r = interner.intern(TypeKey::Infer(TypeParamInfo {
        name: infer_name,
        constraint: None,
        default: None,
    }));

    // T extends { a?: infer R } ? R : never, with T = {}.
    // TODO: Optional property inference currently treats missing properties as non-matches.
    let extends_obj = interner.object(vec![PropertyInfo {
        name: interner.intern_string("a"),
        type_id: infer_r,
        write_type: infer_r,
        optional: true,
        readonly: false,
        is_method: false,
    }]);
    let cond = ConditionalType {
        check_type: t_param,
        extends_type: extends_obj,
        true_type: infer_r,
        false_type: TypeId::NEVER,
        is_distributive: true,
    };

    let cond_type = interner.conditional(cond);
    let mut subst = TypeSubstitution::new();
    let empty_obj = interner.object(Vec::new());
    subst.insert(t_name, empty_obj);

    let instantiated = instantiate_type(&interner, cond_type, &subst);
    let result = evaluate_type(&interner, instantiated);

    assert_eq!(result, TypeId::NEVER);
}

#[test]
fn test_conditional_infer_object_property_intersection_check() {
    let interner = TypeInterner::new();

    let t_name = interner.intern_string("T");
    let t_param = interner.intern(TypeKey::TypeParameter(TypeParamInfo {
        name: t_name,
        constraint: None,
        default: None,
    }));

    let infer_name = interner.intern_string("R");
    let infer_r = interner.intern(TypeKey::Infer(TypeParamInfo {
        name: infer_name,
        constraint: None,
        default: None,
    }));

    // T extends { a: infer R } ? R : never, with T = { a: string } & { b: number }.
    // TODO: Intersection object inference currently yields never; TypeScript infers string.
    let extends_obj = interner.object(vec![PropertyInfo {
        name: interner.intern_string("a"),
        type_id: infer_r,
        write_type: infer_r,
        optional: false,
        readonly: false,
        is_method: false,
    }]);
    let cond = ConditionalType {
        check_type: t_param,
        extends_type: extends_obj,
        true_type: infer_r,
        false_type: TypeId::NEVER,
        is_distributive: true,
    };

    let cond_type = interner.conditional(cond);
    let mut subst = TypeSubstitution::new();
    let obj_a = interner.object(vec![PropertyInfo {
        name: interner.intern_string("a"),
        type_id: TypeId::STRING,
        write_type: TypeId::STRING,
        optional: false,
        readonly: false,
        is_method: false,
    }]);
    let obj_b = interner.object(vec![PropertyInfo {
        name: interner.intern_string("b"),
        type_id: TypeId::NUMBER,
        write_type: TypeId::NUMBER,
        optional: false,
        readonly: false,
        is_method: false,
    }]);
    let intersection = interner.intersection(vec![obj_a, obj_b]);
    subst.insert(t_name, intersection);

    let instantiated = instantiate_type(&interner, cond_type, &subst);
    let result = evaluate_type(&interner, instantiated);

    assert_eq!(result, TypeId::NEVER);
}

#[test]
fn test_conditional_infer_function_param_distributive() {
    let interner = TypeInterner::new();

    let t_name = interner.intern_string("T");
    let t_param = interner.intern(TypeKey::TypeParameter(TypeParamInfo {
        name: t_name,
        constraint: None,
        default: None,
    }));

    let infer_name = interner.intern_string("R");
    let infer_r = interner.intern(TypeKey::Infer(TypeParamInfo {
        name: infer_name,
        constraint: None,
        default: None,
    }));

    // T extends (arg: infer R) => void ? R : never, with T = ((arg: string) => void)
    // | ((arg: number) => void).
    // TODO: Function parameter inference is not implemented; current behavior yields never.
    let extends_fn = interner.function(FunctionShape {
        params: vec![ParamInfo {
            name: None,
            type_id: infer_r,
            optional: false,
            rest: false,
        }],
        this_type: None,
        return_type: TypeId::VOID,
        type_params: Vec::new(),
        type_predicate: None,
        is_constructor: false,
    });
    let cond = ConditionalType {
        check_type: t_param,
        extends_type: extends_fn,
        true_type: infer_r,
        false_type: TypeId::NEVER,
        is_distributive: true,
    };

    let cond_type = interner.conditional(cond);
    let mut subst = TypeSubstitution::new();
    let string_fn = interner.function(FunctionShape {
        params: vec![ParamInfo {
            name: None,
            type_id: TypeId::STRING,
            optional: false,
            rest: false,
        }],
        this_type: None,
        return_type: TypeId::VOID,
        type_params: Vec::new(),
        type_predicate: None,
        is_constructor: false,
    });
    let number_fn = interner.function(FunctionShape {
        params: vec![ParamInfo {
            name: None,
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
    subst.insert(t_name, interner.union(vec![string_fn, number_fn]));

    let instantiated = instantiate_type(&interner, cond_type, &subst);
    let result = evaluate_type(&interner, instantiated);

    assert_eq!(result, TypeId::NEVER);
}

#[test]
fn test_conditional_infer_object_property_non_distributive_union_branch() {
    let interner = TypeInterner::new();

    let t_name = interner.intern_string("T");
    let t_param = interner.intern(TypeKey::TypeParameter(TypeParamInfo {
        name: t_name,
        constraint: None,
        default: None,
    }));

    let infer_name = interner.intern_string("R");
    let infer_r = interner.intern(TypeKey::Infer(TypeParamInfo {
        name: infer_name,
        constraint: None,
        default: None,
    }));

    // [T] extends [{ a: infer R }] ? R : never, with T = { a: string } | number.
    let extends_obj = interner.object(vec![PropertyInfo {
        name: interner.intern_string("a"),
        type_id: infer_r,
        write_type: infer_r,
        optional: false,
        readonly: false,
        is_method: false,
    }]);
    let cond = ConditionalType {
        check_type: interner.tuple(vec![TupleElement {
            type_id: t_param,
            name: None,
            optional: false,
            rest: false,
        }]),
        extends_type: interner.tuple(vec![TupleElement {
            type_id: extends_obj,
            name: None,
            optional: false,
            rest: false,
        }]),
        true_type: infer_r,
        false_type: TypeId::NEVER,
        is_distributive: false,
    };

    let cond_type = interner.conditional(cond);
    let mut subst = TypeSubstitution::new();
    let obj_match = interner.object(vec![PropertyInfo {
        name: interner.intern_string("a"),
        type_id: TypeId::STRING,
        write_type: TypeId::STRING,
        optional: false,
        readonly: false,
        is_method: false,
    }]);
    subst.insert(t_name, interner.union(vec![obj_match, TypeId::NUMBER]));

    let instantiated = instantiate_type(&interner, cond_type, &subst);
    let result = evaluate_type(&interner, instantiated);

    assert_eq!(result, TypeId::NEVER);
}

#[test]
fn test_conditional_infer_tuple_element_extraction() {
    let interner = TypeInterner::new();

    let t_name = interner.intern_string("T");
    let t_param = interner.intern(TypeKey::TypeParameter(TypeParamInfo {
        name: t_name,
        constraint: None,
        default: None,
    }));

    let infer_name = interner.intern_string("R");
    let infer_r = interner.intern(TypeKey::Infer(TypeParamInfo {
        name: infer_name,
        constraint: None,
        default: None,
    }));

    // T extends [infer R] ? R : never, with T = [string] | [number].
    let extends_tuple = interner.tuple(vec![TupleElement {
        type_id: infer_r,
        name: None,
        optional: false,
        rest: false,
    }]);
    let cond = ConditionalType {
        check_type: t_param,
        extends_type: extends_tuple,
        true_type: infer_r,
        false_type: TypeId::NEVER,
        is_distributive: true,
    };

    let cond_type = interner.conditional(cond);
    let mut subst = TypeSubstitution::new();
    subst.insert(
        t_name,
        interner.union(vec![
            interner.tuple(vec![TupleElement {
                type_id: TypeId::STRING,
                name: None,
                optional: false,
                rest: false,
            }]),
            interner.tuple(vec![TupleElement {
                type_id: TypeId::NUMBER,
                name: None,
                optional: false,
                rest: false,
            }]),
        ]),
    );

    let instantiated = instantiate_type(&interner, cond_type, &subst);
    let result = evaluate_type(&interner, instantiated);
    let expected = interner.union(vec![TypeId::STRING, TypeId::NUMBER]);

    assert_eq!(result, expected);
}

#[test]
fn test_conditional_infer_tuple_element_with_constraint() {
    let interner = TypeInterner::new();

    let t_name = interner.intern_string("T");
    let t_param = interner.intern(TypeKey::TypeParameter(TypeParamInfo {
        name: t_name,
        constraint: None,
        default: None,
    }));

    let infer_name = interner.intern_string("R");
    let infer_r = interner.intern(TypeKey::Infer(TypeParamInfo {
        name: infer_name,
        constraint: Some(TypeId::STRING),
        default: None,
    }));

    // T extends [infer R extends string] ? R : never, with T = [number] | [string].
    let extends_tuple = interner.tuple(vec![TupleElement {
        type_id: infer_r,
        name: None,
        optional: false,
        rest: false,
    }]);
    let cond = ConditionalType {
        check_type: t_param,
        extends_type: extends_tuple,
        true_type: infer_r,
        false_type: TypeId::NEVER,
        is_distributive: true,
    };

    let cond_type = interner.conditional(cond);
    let mut subst = TypeSubstitution::new();
    subst.insert(
        t_name,
        interner.union(vec![
            interner.tuple(vec![TupleElement {
                type_id: TypeId::NUMBER,
                name: None,
                optional: false,
                rest: false,
            }]),
            interner.tuple(vec![TupleElement {
                type_id: TypeId::STRING,
                name: None,
                optional: false,
                rest: false,
            }]),
        ]),
    );

    let instantiated = instantiate_type(&interner, cond_type, &subst);
    let result = evaluate_type(&interner, instantiated);

    assert_eq!(result, TypeId::STRING);
}

#[test]
fn test_conditional_infer_readonly_array_element_extraction() {
    let interner = TypeInterner::new();

    let t_name = interner.intern_string("T");
    let t_param = interner.intern(TypeKey::TypeParameter(TypeParamInfo {
        name: t_name,
        constraint: None,
        default: None,
    }));

    let infer_name = interner.intern_string("R");
    let infer_r = interner.intern(TypeKey::Infer(TypeParamInfo {
        name: infer_name,
        constraint: None,
        default: None,
    }));

    // T extends readonly (infer R)[] ? R : never, with T = readonly string[] | readonly number[].
    let extends_array =
        interner.intern(TypeKey::ReadonlyType(interner.array(infer_r)));
    let cond = ConditionalType {
        check_type: t_param,
        extends_type: extends_array,
        true_type: infer_r,
        false_type: TypeId::NEVER,
        is_distributive: true,
    };

    let cond_type = interner.conditional(cond);
    let mut subst = TypeSubstitution::new();
    let readonly_string_array =
        interner.intern(TypeKey::ReadonlyType(interner.array(TypeId::STRING)));
    let readonly_number_array =
        interner.intern(TypeKey::ReadonlyType(interner.array(TypeId::NUMBER)));
    subst.insert(t_name, interner.union(vec![readonly_string_array, readonly_number_array]));

    let instantiated = instantiate_type(&interner, cond_type, &subst);
    let result = evaluate_type(&interner, instantiated);
    let expected = interner.union(vec![TypeId::STRING, TypeId::NUMBER]);

    assert_eq!(result, expected);
}

#[test]
fn test_conditional_infer_readonly_tuple_element_extraction() {
    let interner = TypeInterner::new();

    let t_name = interner.intern_string("T");
    let t_param = interner.intern(TypeKey::TypeParameter(TypeParamInfo {
        name: t_name,
        constraint: None,
        default: None,
    }));

    let infer_name = interner.intern_string("R");
    let infer_r = interner.intern(TypeKey::Infer(TypeParamInfo {
        name: infer_name,
        constraint: None,
        default: None,
    }));

    // T extends readonly [infer R] ? R : never, with T = readonly [string] | readonly [number].
    let extends_tuple = interner.intern(TypeKey::ReadonlyType(interner.tuple(vec![
        TupleElement {
            type_id: infer_r,
            name: None,
            optional: false,
            rest: false,
        },
    ])));
    let cond = ConditionalType {
        check_type: t_param,
        extends_type: extends_tuple,
        true_type: infer_r,
        false_type: TypeId::NEVER,
        is_distributive: true,
    };

    let cond_type = interner.conditional(cond);
    let mut subst = TypeSubstitution::new();
    let readonly_string_tuple = interner.intern(TypeKey::ReadonlyType(interner.tuple(vec![
        TupleElement {
            type_id: TypeId::STRING,
            name: None,
            optional: false,
            rest: false,
        },
    ])));
    let readonly_number_tuple = interner.intern(TypeKey::ReadonlyType(interner.tuple(vec![
        TupleElement {
            type_id: TypeId::NUMBER,
            name: None,
            optional: false,
            rest: false,
        },
    ])));
    subst.insert(t_name, interner.union(vec![readonly_string_tuple, readonly_number_tuple]));

    let instantiated = instantiate_type(&interner, cond_type, &subst);
    let result = evaluate_type(&interner, instantiated);
    let expected = interner.union(vec![TypeId::STRING, TypeId::NUMBER]);

    assert_eq!(result, expected);
}

#[test]
fn test_conditional_infer_readonly_array_mixed_input() {
    let interner = TypeInterner::new();

    let t_name = interner.intern_string("T");
    let t_param = interner.intern(TypeKey::TypeParameter(TypeParamInfo {
        name: t_name,
        constraint: None,
        default: None,
    }));

    let infer_name = interner.intern_string("R");
    let infer_r = interner.intern(TypeKey::Infer(TypeParamInfo {
        name: infer_name,
        constraint: None,
        default: None,
    }));

    // T extends readonly (infer R)[] ? R : never, with T = readonly string[] | number[].
    let extends_array =
        interner.intern(TypeKey::ReadonlyType(interner.array(infer_r)));
    let cond = ConditionalType {
        check_type: t_param,
        extends_type: extends_array,
        true_type: infer_r,
        false_type: TypeId::NEVER,
        is_distributive: true,
    };

    let cond_type = interner.conditional(cond);
    let mut subst = TypeSubstitution::new();
    let readonly_string_array =
        interner.intern(TypeKey::ReadonlyType(interner.array(TypeId::STRING)));
    let number_array = interner.array(TypeId::NUMBER);
    subst.insert(t_name, interner.union(vec![readonly_string_array, number_array]));

    let instantiated = instantiate_type(&interner, cond_type, &subst);
    let result = evaluate_type(&interner, instantiated);
    let expected = interner.union(vec![TypeId::STRING, TypeId::NUMBER]);

    assert_eq!(result, expected);
}

#[test]
fn test_conditional_instantiated_param_tuple_wrapper_no_distribution() {
    let interner = TypeInterner::new();

    let t_name = interner.intern_string("T");
    let t_param = interner.intern(TypeKey::TypeParameter(TypeParamInfo {
        name: t_name,
        constraint: None,
        default: None,
    }));

    let string_or_number = interner.union(vec![TypeId::STRING, TypeId::NUMBER]);
    let lit_true = interner.literal_boolean(true);
    let lit_false = interner.literal_boolean(false);

    let tuple_check = interner.tuple(vec![TupleElement {
        type_id: t_param,
        name: None,
        optional: false,
        rest: false,
    }]);
    let tuple_extends = interner.tuple(vec![TupleElement {
        type_id: TypeId::STRING,
        name: None,
        optional: false,
        rest: false,
    }]);

    // [T] extends [string] ? true : false, with T = string | number (no distribution).
    let cond = ConditionalType {
        check_type: tuple_check,
        extends_type: tuple_extends,
        true_type: lit_true,
        false_type: lit_false,
        is_distributive: false,
    };

    let cond_type = interner.conditional(cond);
    let mut subst = TypeSubstitution::new();
    subst.insert(t_name, string_or_number);

    let instantiated = instantiate_type(&interner, cond_type, &subst);
    let result = evaluate_type(&interner, instantiated);

    assert_eq!(result, lit_false);
}

#[test]
fn test_conditional_any_produces_union() {
    let interner = TypeInterner::new();

    // any extends string ? number : boolean
    // any produces union of branches
    let cond = ConditionalType {
        check_type: TypeId::ANY,
        extends_type: TypeId::STRING,
        true_type: TypeId::NUMBER,
        false_type: TypeId::BOOLEAN,
        is_distributive: false,
    };

    let result = evaluate_conditional(&interner, &cond);
    let expected = interner.union(vec![TypeId::NUMBER, TypeId::BOOLEAN]);
    assert_eq!(result, expected);
}

#[test]
fn test_conditional_any_error_poisoning() {
    let interner = TypeInterner::new();

    // any extends string ? error : number
    // any produces union of branches, which should poison to error.
    let cond = ConditionalType {
        check_type: TypeId::ANY,
        extends_type: TypeId::STRING,
        true_type: TypeId::ERROR,
        false_type: TypeId::NUMBER,
        is_distributive: false,
    };

    let result = evaluate_conditional(&interner, &cond);
    assert_eq!(result, TypeId::ERROR);
}

#[test]
fn test_conditional_distributive_never() {
    let interner = TypeInterner::new();

    // T extends string ? number : boolean, with T = never (distributive)
    // Distributes over empty union -> never
    let cond = ConditionalType {
        check_type: TypeId::NEVER,
        extends_type: TypeId::STRING,
        true_type: TypeId::NUMBER,
        false_type: TypeId::BOOLEAN,
        is_distributive: true,
    };

    let result = evaluate_conditional(&interner, &cond);
    assert_eq!(result, TypeId::NEVER);
}

#[test]
fn test_conditional_deferred_type_parameter() {
    let interner = TypeInterner::new();

    // T extends string ? number : boolean
    // Should remain deferred when T is an unsubstituted type parameter
    let type_param_t = interner.intern(TypeKey::TypeParameter(TypeParamInfo {
        name: interner.intern_string("T"),
        constraint: None,
        default: None,
    }));

    let cond = ConditionalType {
        check_type: type_param_t,
        extends_type: TypeId::STRING,
        true_type: TypeId::NUMBER,
        false_type: TypeId::BOOLEAN,
        is_distributive: true,
    };

    let cond_type = interner.conditional(cond.clone());
    let result = evaluate_conditional(&interner, &cond);

    // Should return the same conditional (deferred)
    assert_eq!(result, cond_type);
}

#[test]
fn test_index_access_object_literal() {
    let interner = TypeInterner::new();

    // { x: number, y: string }["x"] -> number
    let obj = interner.object(vec![
        PropertyInfo { name: interner.intern_string("x"), type_id: TypeId::NUMBER,
 write_type: TypeId::NUMBER, optional: false, readonly: false, is_method: false },
        PropertyInfo { name: interner.intern_string("y"), type_id: TypeId::STRING,
 write_type: TypeId::STRING, optional: false, readonly: false, is_method: false },
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
        PropertyInfo { name: interner.intern_string("x"), type_id: TypeId::NUMBER,
 write_type: TypeId::NUMBER, optional: false, readonly: false, is_method: false },
        PropertyInfo { name: interner.intern_string("y"), type_id: TypeId::STRING,
 write_type: TypeId::STRING, optional: false, readonly: false, is_method: false },
    ]);
    let key_y = interner.literal_string("y");

    let result = evaluate_index_access(&interner, obj, key_y);
    assert_eq!(result, TypeId::STRING);
}

#[test]
fn test_index_access_object_string_index_optional_properties() {
    let interner = TypeInterner::new();

    let obj = interner.object(vec![
        PropertyInfo { name: interner.intern_string("x"), type_id: TypeId::NUMBER,
 write_type: TypeId::NUMBER, optional: true, readonly: false, is_method: false },
        PropertyInfo { name: interner.intern_string("y"), type_id: TypeId::STRING,
 write_type: TypeId::STRING, optional: false, readonly: false, is_method: false },
    ]);

    let result = evaluate_index_access(&interner, obj, TypeId::STRING);
    let expected = interner.union(vec![TypeId::NUMBER, TypeId::STRING, TypeId::UNDEFINED]);
    assert_eq!(result, expected);
}

#[test]
fn test_index_access_object_missing_key() {
    let interner = TypeInterner::new();

    // { x: number }["z"] -> undefined
    let obj = interner.object(vec![
        PropertyInfo { name: interner.intern_string("x"), type_id: TypeId::NUMBER,
 write_type: TypeId::NUMBER, optional: false, readonly: false, is_method: false },
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
        PropertyInfo { name: interner.intern_string("x"), type_id: TypeId::NUMBER,
 write_type: TypeId::NUMBER, optional: false, readonly: false, is_method: false },
        PropertyInfo { name: interner.intern_string("y"), type_id: TypeId::STRING,
 write_type: TypeId::STRING, optional: false, readonly: false, is_method: false },
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
fn test_index_access_union_object_literal_key() {
    let interner = TypeInterner::new();

    let obj_a = interner.object(vec![PropertyInfo {
        name: interner.intern_string("x"),
        type_id: TypeId::NUMBER,
        write_type: TypeId::NUMBER,
        optional: false,
        readonly: false,
        is_method: false,
    }]);
    let obj_b = interner.object(vec![PropertyInfo {
        name: interner.intern_string("y"),
        type_id: TypeId::STRING,
        write_type: TypeId::STRING,
        optional: false,
        readonly: false,
        is_method: false,
    }]);
    let union_obj = interner.union(vec![obj_a, obj_b]);
    let key_x = interner.literal_string("x");

    let result = evaluate_index_access(&interner, union_obj, key_x);
    assert_eq!(result, TypeId::NUMBER);
}

#[test]
fn test_index_access_union_object_union_key() {
    let interner = TypeInterner::new();

    let obj_a = interner.object(vec![PropertyInfo {
        name: interner.intern_string("x"),
        type_id: TypeId::NUMBER,
        write_type: TypeId::NUMBER,
        optional: false,
        readonly: false,
        is_method: false,
    }]);
    let obj_b = interner.object(vec![PropertyInfo {
        name: interner.intern_string("y"),
        type_id: TypeId::STRING,
        write_type: TypeId::STRING,
        optional: false,
        readonly: false,
        is_method: false,
    }]);
    let union_obj = interner.union(vec![obj_a, obj_b]);
    let key_x = interner.literal_string("x");
    let key_y = interner.literal_string("y");
    let key_union = interner.union(vec![key_x, key_y]);

    let result = evaluate_index_access(&interner, union_obj, key_union);
    let expected = interner.union(vec![TypeId::NUMBER, TypeId::STRING]);
    assert_eq!(result, expected);
}

#[test]
fn test_correlated_union_index_access_cross_product() {
    let interner = TypeInterner::new();

    let kind = interner.intern_string("kind");
    let key_a = interner.intern_string("a");
    let key_b = interner.intern_string("b");

    let obj_a = interner.object(vec![
        PropertyInfo {
            name: kind,
            type_id: interner.literal_string("a"),
            write_type: interner.literal_string("a"),
            optional: false,
            readonly: false,
            is_method: false,
        },
        PropertyInfo {
            name: key_a,
            type_id: TypeId::NUMBER,
            write_type: TypeId::NUMBER,
            optional: false,
            readonly: false,
            is_method: false,
        },
    ]);
    let obj_b = interner.object(vec![
        PropertyInfo {
            name: kind,
            type_id: interner.literal_string("b"),
            write_type: interner.literal_string("b"),
            optional: false,
            readonly: false,
            is_method: false,
        },
        PropertyInfo {
            name: key_b,
            type_id: TypeId::STRING,
            write_type: TypeId::STRING,
            optional: false,
            readonly: false,
            is_method: false,
        },
    ]);

    let union_obj = interner.union(vec![obj_a, obj_b]);
    let key_union = interner.union(vec![
        interner.literal_string("a"),
        interner.literal_string("b"),
    ]);

    let result = evaluate_index_access(&interner, union_obj, key_union);
    let expected = interner.union(vec![TypeId::NUMBER, TypeId::STRING]);
    assert_eq!(result, expected);
}

#[test]
fn test_index_access_union_object_union_key_no_unchecked() {
    let interner = TypeInterner::new();

    let obj_a = interner.object(vec![PropertyInfo {
        name: interner.intern_string("x"),
        type_id: TypeId::NUMBER,
        write_type: TypeId::NUMBER,
        optional: false,
        readonly: false,
        is_method: false,
    }]);
    let obj_b = interner.object(vec![PropertyInfo {
        name: interner.intern_string("y"),
        type_id: TypeId::STRING,
        write_type: TypeId::STRING,
        optional: false,
        readonly: false,
        is_method: false,
    }]);
    let union_obj = interner.union(vec![obj_a, obj_b]);
    let key_x = interner.literal_string("x");
    let key_y = interner.literal_string("y");
    let key_union = interner.union(vec![key_x, key_y]);

    let mut evaluator = TypeEvaluator::new(&interner);
    evaluator.set_no_unchecked_indexed_access(true);
    let result = evaluator.evaluate_index_access(union_obj, key_union);
    let expected = interner.union(vec![TypeId::NUMBER, TypeId::STRING, TypeId::UNDEFINED]);
    assert_eq!(result, expected);
}

#[test]
fn test_index_access_union_object_literal_key_no_unchecked() {
    let interner = TypeInterner::new();

    let obj_a = interner.object(vec![PropertyInfo {
        name: interner.intern_string("x"),
        type_id: TypeId::NUMBER,
        write_type: TypeId::NUMBER,
        optional: false,
        readonly: false,
        is_method: false,
    }]);
    let obj_b = interner.object(vec![PropertyInfo {
        name: interner.intern_string("y"),
        type_id: TypeId::STRING,
        write_type: TypeId::STRING,
        optional: false,
        readonly: false,
        is_method: false,
    }]);
    let union_obj = interner.union(vec![obj_a, obj_b]);
    let key_x = interner.literal_string("x");

    let mut evaluator = TypeEvaluator::new(&interner);
    evaluator.set_no_unchecked_indexed_access(true);

    let result = evaluator.evaluate_index_access(union_obj, key_x);
    let expected = interner.union(vec![TypeId::NUMBER, TypeId::UNDEFINED]);
    assert_eq!(result, expected);
}

#[test]
fn test_index_access_object_with_string_index_signature() {
    let interner = TypeInterner::new();

    let key_x = interner.intern_string("x");
    let key_y = interner.literal_string("y");

    let obj = interner.object_with_index(ObjectShape {
        properties: vec![PropertyInfo {
            name: key_x,
            type_id: TypeId::STRING,
            write_type: TypeId::STRING,
            optional: false,
            readonly: false,
            is_method: false,
        }],
        string_index: Some(IndexSignature {
            key_type: TypeId::STRING,
            value_type: TypeId::NUMBER,
            readonly: false,
        }),
        number_index: None,
    });

    let key_x_literal = interner.literal_string("x");
    let result = evaluate_index_access(&interner, obj, key_x_literal);
    assert_eq!(result, TypeId::STRING);

    let result = evaluate_index_access(&interner, obj, key_y);
    assert_eq!(result, TypeId::NUMBER);

    let result = evaluate_index_access(&interner, obj, TypeId::STRING);
    assert_eq!(result, TypeId::NUMBER);

    let key_union = interner.union(vec![key_x_literal, key_y]);
    let result = evaluate_index_access(&interner, obj, key_union);
    let expected = interner.union(vec![TypeId::STRING, TypeId::NUMBER]);
    assert_eq!(result, expected);
}

#[test]
fn test_index_access_object_with_string_index_signature_optional_property() {
    let interner = TypeInterner::new();

    let key_x = interner.intern_string("x");
    let key_y = interner.literal_string("y");

    let obj = interner.object_with_index(ObjectShape {
        properties: vec![PropertyInfo {
            name: key_x,
            type_id: TypeId::NUMBER,
            write_type: TypeId::NUMBER,
            optional: true,
            readonly: false,
            is_method: false,
        }],
        string_index: Some(IndexSignature {
            key_type: TypeId::STRING,
            value_type: TypeId::BOOLEAN,
            readonly: false,
        }),
        number_index: None,
    });

    let key_x_literal = interner.literal_string("x");
    let result = evaluate_index_access(&interner, obj, key_x_literal);
    let expected = interner.union(vec![TypeId::NUMBER, TypeId::UNDEFINED]);
    assert_eq!(result, expected);

    let result = evaluate_index_access(&interner, obj, key_y);
    assert_eq!(result, TypeId::BOOLEAN);

    let key_union = interner.union(vec![key_x_literal, key_y]);
    let result = evaluate_index_access(&interner, obj, key_union);
    let expected = interner.union(vec![TypeId::NUMBER, TypeId::UNDEFINED, TypeId::BOOLEAN]);
    assert_eq!(result, expected);
}

#[test]
fn test_index_access_object_with_string_index_signature_optional_property_no_unchecked() {
    let interner = TypeInterner::new();

    let obj = interner.object_with_index(ObjectShape {
        properties: vec![PropertyInfo {
            name: interner.intern_string("x"),
            type_id: TypeId::NUMBER,
            write_type: TypeId::NUMBER,
            optional: true,
            readonly: false,
            is_method: false,
        }],
        string_index: Some(IndexSignature {
            key_type: TypeId::STRING,
            value_type: TypeId::BOOLEAN,
            readonly: false,
        }),
        number_index: None,
    });

    let mut evaluator = TypeEvaluator::new(&interner);
    evaluator.set_no_unchecked_indexed_access(true);

    let key_x = interner.literal_string("x");
    let result = evaluator.evaluate_index_access(obj, key_x);
    let expected = interner.union(vec![TypeId::NUMBER, TypeId::UNDEFINED]);
    assert_eq!(result, expected);

    let key_y = interner.literal_string("y");
    let result = evaluator.evaluate_index_access(obj, key_y);
    let expected = interner.union(vec![TypeId::BOOLEAN, TypeId::UNDEFINED]);
    assert_eq!(result, expected);

    let result = evaluator.evaluate_index_access(obj, TypeId::STRING);
    let expected = interner.union(vec![TypeId::BOOLEAN, TypeId::UNDEFINED]);
    assert_eq!(result, expected);

    let key_union = interner.union(vec![key_x, key_y]);
    let result = evaluator.evaluate_index_access(obj, key_union);
    let expected = interner.union(vec![TypeId::NUMBER, TypeId::BOOLEAN, TypeId::UNDEFINED]);
    assert_eq!(result, expected);
}

#[test]
fn test_no_unchecked_object_index_signature_evaluate() {
    let interner = TypeInterner::new();

    let obj = interner.object_with_index(ObjectShape {
        properties: Vec::new(),
        string_index: Some(IndexSignature {
            key_type: TypeId::STRING,
            value_type: TypeId::NUMBER,
            readonly: false,
        }),
        number_index: None,
    });

    let mut evaluator = TypeEvaluator::new(&interner);
    evaluator.set_no_unchecked_indexed_access(true);

    let result = evaluator.evaluate_index_access(obj, TypeId::NUMBER);
    let expected = interner.union(vec![TypeId::NUMBER, TypeId::UNDEFINED]);
    assert_eq!(result, expected);
}

#[test]
fn test_index_access_object_with_number_index_signature() {
    let interner = TypeInterner::new();

    let obj = interner.object_with_index(ObjectShape {
        properties: Vec::new(),
        string_index: None,
        number_index: Some(IndexSignature {
            key_type: TypeId::NUMBER,
            value_type: TypeId::BOOLEAN,
            readonly: false,
        }),
    });

    let result = evaluate_index_access(&interner, obj, TypeId::NUMBER);
    assert_eq!(result, TypeId::BOOLEAN);

    let one = interner.literal_number(1.0);
    let result = evaluate_index_access(&interner, obj, one);
    assert_eq!(result, TypeId::BOOLEAN);
}

#[test]
fn test_index_access_object_with_number_index_signature_no_unchecked() {
    let interner = TypeInterner::new();

    let obj = interner.object_with_index(ObjectShape {
        properties: Vec::new(),
        string_index: Some(IndexSignature {
            key_type: TypeId::STRING,
            value_type: TypeId::BOOLEAN,
            readonly: false,
        }),
        number_index: Some(IndexSignature {
            key_type: TypeId::NUMBER,
            value_type: TypeId::NUMBER,
            readonly: false,
        }),
    });

    let mut evaluator = TypeEvaluator::new(&interner);
    evaluator.set_no_unchecked_indexed_access(true);

    let result = evaluator.evaluate_index_access(obj, TypeId::NUMBER);
    let expected = interner.union(vec![TypeId::NUMBER, TypeId::UNDEFINED]);
    assert_eq!(result, expected);

    let zero = interner.literal_number(0.0);
    let result = evaluator.evaluate_index_access(obj, zero);
    let expected = interner.union(vec![TypeId::NUMBER, TypeId::UNDEFINED]);
    assert_eq!(result, expected);

    let zero_str = interner.literal_string("0");
    let result = evaluator.evaluate_index_access(obj, zero_str);
    let expected = interner.union(vec![TypeId::NUMBER, TypeId::UNDEFINED]);
    assert_eq!(result, expected);
}

#[test]
fn test_index_access_resolves_ref() {
    use crate::solver::{TypeEnvironment, SymbolRef};

    let interner = TypeInterner::new();
    let mut env = TypeEnvironment::new();

    let obj = interner.object(vec![
        PropertyInfo { name: interner.intern_string("x"), type_id: TypeId::NUMBER,
 write_type: TypeId::NUMBER, optional: false, readonly: false, is_method: false },
    ]);

    let sym = SymbolRef(1);
    env.insert(sym, obj);

    let ref_type = interner.reference(sym);
    let key_x = interner.literal_string("x");

    let evaluator = TypeEvaluator::with_resolver(&interner, &env);
    let result = evaluator.evaluate_index_access(ref_type, key_x);
    assert_eq!(result, TypeId::NUMBER);
}

#[test]
fn test_index_access_type_param_constraint() {
    let interner = TypeInterner::new();

    let constraint = interner.object(vec![
        PropertyInfo { name: interner.intern_string("x"), type_id: TypeId::NUMBER,
 write_type: TypeId::NUMBER, optional: false, readonly: false, is_method: false },
    ]);

    let type_param = interner.intern(TypeKey::TypeParameter(TypeParamInfo {
        name: interner.intern_string("T"),
        constraint: Some(constraint),
        default: None,
    }));

    let key_x = interner.literal_string("x");
    let result = evaluate_index_access(&interner, type_param, key_x);
    assert_eq!(result, TypeId::NUMBER);
}

#[test]
fn test_index_access_type_param_no_constraint_deferred() {
    let interner = TypeInterner::new();

    let type_param = interner.intern(TypeKey::TypeParameter(TypeParamInfo {
        name: interner.intern_string("T"),
        constraint: None,
        default: None,
    }));

    let key_x = interner.literal_string("x");
    let result = evaluate_index_access(&interner, type_param, key_x);

    match interner.lookup(result) {
        Some(TypeKey::IndexAccess(obj, idx)) => {
            assert_eq!(obj, type_param);
            assert_eq!(idx, key_x);
        }
        other => panic!("Expected deferred IndexAccess, got {:?}", other),
    }
}

#[test]
fn test_index_access_optional_property() {
    let interner = TypeInterner::new();

    let obj = interner.object(vec![
        PropertyInfo { name: interner.intern_string("x"), type_id: TypeId::NUMBER,
 write_type: TypeId::NUMBER, optional: true, readonly: false, is_method: false },
    ]);

    let key_x = interner.literal_string("x");
    let result = evaluate_index_access(&interner, obj, key_x);
    let expected = interner.union(vec![TypeId::NUMBER, TypeId::UNDEFINED]);
    assert_eq!(result, expected);
}

#[test]
fn test_index_access_with_no_unchecked_indexed_access() {
    let interner = TypeInterner::new();

    let indexed = interner.object_with_index(ObjectShape {
        properties: Vec::new(),
        string_index: Some(IndexSignature {
            key_type: TypeId::STRING,
            value_type: TypeId::NUMBER,
            readonly: false,
        }),
        number_index: None,
    });

    let array = interner.array(TypeId::STRING);

    let mut evaluator = TypeEvaluator::new(&interner);
    evaluator.set_no_unchecked_indexed_access(true);

    let result = evaluator.evaluate_index_access(indexed, TypeId::STRING);
    let expected = interner.union(vec![TypeId::NUMBER, TypeId::UNDEFINED]);
    assert_eq!(result, expected);

    let result = evaluator.evaluate_index_access(array, TypeId::NUMBER);
    let expected = interner.union(vec![TypeId::STRING, TypeId::UNDEFINED]);
    assert_eq!(result, expected);
}

#[test]
fn test_index_access_array_literal_with_no_unchecked_indexed_access() {
    let interner = TypeInterner::new();

    let array = interner.array(TypeId::STRING);
    let zero = interner.literal_number(0.0);

    let mut evaluator = TypeEvaluator::new(&interner);
    evaluator.set_no_unchecked_indexed_access(true);

    let result = evaluator.evaluate_index_access(array, zero);
    let expected = interner.union(vec![TypeId::STRING, TypeId::UNDEFINED]);
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
fn test_no_unchecked_indexed_access_array_union_key() {
    let interner = TypeInterner::new();

    let string_array = interner.array(TypeId::STRING);
    let length_key = interner.literal_string("length");
    let key_union = interner.union(vec![TypeId::NUMBER, length_key]);

    let mut evaluator = TypeEvaluator::new(&interner);
    let result = evaluator.evaluate_index_access(string_array, key_union);
    let expected = interner.union(vec![TypeId::STRING, TypeId::NUMBER]);
    assert_eq!(result, expected);

    evaluator.set_no_unchecked_indexed_access(true);
    let result = evaluator.evaluate_index_access(string_array, key_union);
    let expected = interner.union(vec![TypeId::STRING, TypeId::NUMBER, TypeId::UNDEFINED]);
    assert_eq!(result, expected);
}

#[test]
fn test_index_access_array_string_index() {
    let interner = TypeInterner::new();

    let string_array = interner.array(TypeId::STRING);
    let includes_key = interner.literal_string("includes");
    let includes_type = evaluate_index_access(&interner, string_array, includes_key);

    let result = evaluate_index_access(&interner, string_array, TypeId::STRING);
    let key = interner.lookup(result).expect("expected union for array[string]");

    match key {
        TypeKey::Union(members) => {
            let members = interner.type_list(members);
            assert!(members.contains(&TypeId::NUMBER));
            assert!(members.contains(&includes_type));
            assert!(!members.contains(&TypeId::STRING));
        }
        other => panic!("Expected union, got {:?}", other),
    }
}

#[test]
fn test_index_access_array_string_index_with_no_unchecked_indexed_access() {
    let interner = TypeInterner::new();

    let string_array = interner.array(TypeId::STRING);
    let mut evaluator = TypeEvaluator::new(&interner);
    evaluator.set_no_unchecked_indexed_access(true);

    let result = evaluator.evaluate_index_access(string_array, TypeId::STRING);
    let key = interner.lookup(result).expect("expected union for array[string]");

    match key {
        TypeKey::Union(members) => {
            let members = interner.type_list(members);
            assert!(members.contains(&TypeId::UNDEFINED));
        }
        other => panic!("Expected union, got {:?}", other),
    }
}

#[test]
fn test_index_access_array_string_literal_length() {
    let interner = TypeInterner::new();

    let string_array = interner.array(TypeId::STRING);
    let length_key = interner.literal_string("length");

    let result = evaluate_index_access(&interner, string_array, length_key);
    assert_eq!(result, TypeId::NUMBER);
}

#[test]
fn test_index_access_array_string_literal_method() {
    let interner = TypeInterner::new();

    let string_array = interner.array(TypeId::STRING);
    let includes_key = interner.literal_string("includes");

    let result = evaluate_index_access(&interner, string_array, includes_key);
    match interner.lookup(result) {
        Some(TypeKey::Function(func_id)) => {
            let func = interner.function_shape(func_id);
            assert_eq!(func.return_type, TypeId::BOOLEAN);
            assert_eq!(func.params.len(), 1);
            assert!(func.params[0].rest);
        }
        other => panic!("Expected function type, got {:?}", other),
    }
}

#[test]
fn test_index_access_array_string_literal_numeric_key_with_no_unchecked_indexed_access() {
    let interner = TypeInterner::new();

    let string_array = interner.array(TypeId::STRING);
    let zero = interner.literal_string("0");

    let mut evaluator = TypeEvaluator::new(&interner);
    evaluator.set_no_unchecked_indexed_access(true);

    let result = evaluator.evaluate_index_access(string_array, zero);
    let expected = interner.union(vec![TypeId::STRING, TypeId::UNDEFINED]);
    assert_eq!(result, expected);
}

#[test]
fn test_index_access_readonly_array() {
    let interner = TypeInterner::new();

    let array = interner.array(TypeId::STRING);
    let readonly_array = interner.intern(TypeKey::ReadonlyType(array));

    let result = evaluate_index_access(&interner, readonly_array, TypeId::NUMBER);
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
fn test_index_access_tuple_rest_array_literal() {
    let interner = TypeInterner::new();

    // [string, ...number[]][1] -> number
    let number_array = interner.array(TypeId::NUMBER);
    let tuple = interner.tuple(vec![
        TupleElement { type_id: TypeId::STRING, name: None, optional: false, rest: false },
        TupleElement { type_id: number_array, name: None, optional: false, rest: true },
    ]);
    let one = interner.literal_number(1.0);
    let two = interner.literal_number(2.0);

    assert_eq!(evaluate_index_access(&interner, tuple, one), TypeId::NUMBER);
    assert_eq!(evaluate_index_access(&interner, tuple, two), TypeId::NUMBER);
}

#[test]
fn test_index_access_tuple_rest_tuple_literal() {
    let interner = TypeInterner::new();

    // [string, ...[number, boolean]][1] -> number
    let rest_tuple = interner.tuple(vec![
        TupleElement { type_id: TypeId::NUMBER, name: None, optional: false, rest: false },
        TupleElement { type_id: TypeId::BOOLEAN, name: None, optional: false, rest: false },
    ]);
    let tuple = interner.tuple(vec![
        TupleElement { type_id: TypeId::STRING, name: None, optional: false, rest: false },
        TupleElement { type_id: rest_tuple, name: None, optional: false, rest: true },
    ]);

    let one = interner.literal_number(1.0);
    let two = interner.literal_number(2.0);
    let three = interner.literal_number(3.0);

    assert_eq!(evaluate_index_access(&interner, tuple, one), TypeId::NUMBER);
    assert_eq!(evaluate_index_access(&interner, tuple, two), TypeId::BOOLEAN);
    assert_eq!(evaluate_index_access(&interner, tuple, three), TypeId::UNDEFINED);
}

#[test]
fn test_index_access_tuple_optional_literal() {
    let interner = TypeInterner::new();

    // [string, number?][1] -> number | undefined
    let tuple = interner.tuple(vec![
        TupleElement { type_id: TypeId::STRING, name: None, optional: false, rest: false },
        TupleElement { type_id: TypeId::NUMBER, name: None, optional: true, rest: false },
    ]);
    let one = interner.literal_number(1.0);

    let result = evaluate_index_access(&interner, tuple, one);
    let expected = interner.union(vec![TypeId::NUMBER, TypeId::UNDEFINED]);
    assert_eq!(result, expected);
}

#[test]
fn test_index_access_tuple_negative_literal() {
    let interner = TypeInterner::new();

    let number_array = interner.array(TypeId::NUMBER);
    let tuple = interner.tuple(vec![
        TupleElement { type_id: TypeId::STRING, name: None, optional: false, rest: false },
        TupleElement { type_id: number_array, name: None, optional: false, rest: true },
    ]);
    let negative = interner.literal_number(-1.0);

    let result = evaluate_index_access(&interner, tuple, negative);
    assert_eq!(result, TypeId::UNDEFINED);
}

#[test]
fn test_index_access_tuple_fractional_literal() {
    let interner = TypeInterner::new();

    let tuple = interner.tuple(vec![
        TupleElement { type_id: TypeId::STRING, name: None, optional: false, rest: false },
        TupleElement { type_id: TypeId::NUMBER, name: None, optional: false, rest: false },
    ]);
    let fractional = interner.literal_number(1.5);

    let result = evaluate_index_access(&interner, tuple, fractional);
    assert_eq!(result, TypeId::UNDEFINED);
}

#[test]
fn test_index_access_tuple_negative_string_literal() {
    let interner = TypeInterner::new();

    let number_array = interner.array(TypeId::NUMBER);
    let tuple = interner.tuple(vec![
        TupleElement { type_id: TypeId::STRING, name: None, optional: false, rest: false },
        TupleElement { type_id: number_array, name: None, optional: false, rest: true },
    ]);
    let negative = interner.literal_string("-1");

    let result = evaluate_index_access(&interner, tuple, negative);
    assert_eq!(result, TypeId::UNDEFINED);
}

#[test]
fn test_index_access_tuple_fractional_string_literal() {
    let interner = TypeInterner::new();

    let tuple = interner.tuple(vec![
        TupleElement { type_id: TypeId::STRING, name: None, optional: false, rest: false },
        TupleElement { type_id: TypeId::NUMBER, name: None, optional: false, rest: false },
    ]);
    let fractional = interner.literal_string("1.5");

    let result = evaluate_index_access(&interner, tuple, fractional);
    assert_eq!(result, TypeId::UNDEFINED);
}

#[test]
fn test_index_access_tuple_string_index() {
    let interner = TypeInterner::new();

    let tuple = interner.tuple(vec![
        TupleElement { type_id: TypeId::STRING, name: None, optional: false, rest: false },
        TupleElement { type_id: TypeId::NUMBER, name: None, optional: false, rest: false },
    ]);
    let map_key = interner.literal_string("map");
    let map_type = evaluate_index_access(&interner, tuple, map_key);

    let result = evaluate_index_access(&interner, tuple, TypeId::STRING);
    let key = interner.lookup(result).expect("expected union for tuple[string]");

    match key {
        TypeKey::Union(members) => {
            let members = interner.type_list(members);
            assert!(members.contains(&TypeId::STRING));
            assert!(members.contains(&TypeId::NUMBER));
            assert!(members.contains(&map_type));
        }
        other => panic!("Expected union, got {:?}", other),
    }
}

#[test]
fn test_index_access_tuple_string_index_with_no_unchecked_indexed_access() {
    let interner = TypeInterner::new();

    let tuple = interner.tuple(vec![
        TupleElement { type_id: TypeId::STRING, name: None, optional: false, rest: false },
        TupleElement { type_id: TypeId::NUMBER, name: None, optional: false, rest: false },
    ]);
    let mut evaluator = TypeEvaluator::new(&interner);
    evaluator.set_no_unchecked_indexed_access(true);

    let result = evaluator.evaluate_index_access(tuple, TypeId::STRING);
    let key = interner.lookup(result).expect("expected union for tuple[string]");

    match key {
        TypeKey::Union(members) => {
            let members = interner.type_list(members);
            assert!(members.contains(&TypeId::UNDEFINED));
        }
        other => panic!("Expected union, got {:?}", other),
    }
}

#[test]
fn test_index_access_tuple_string_literal_length() {
    let interner = TypeInterner::new();

    let tuple = interner.tuple(vec![
        TupleElement { type_id: TypeId::STRING, name: None, optional: false, rest: false },
        TupleElement { type_id: TypeId::NUMBER, name: None, optional: false, rest: false },
    ]);
    let length_key = interner.literal_string("length");

    let result = evaluate_index_access(&interner, tuple, length_key);
    assert_eq!(result, TypeId::NUMBER);
}

#[test]
fn test_index_access_tuple_string_literal_numeric_key() {
    let interner = TypeInterner::new();

    let tuple = interner.tuple(vec![
        TupleElement { type_id: TypeId::STRING, name: None, optional: false, rest: false },
        TupleElement { type_id: TypeId::NUMBER, name: None, optional: false, rest: false },
    ]);
    let zero = interner.literal_string("0");

    let result = evaluate_index_access(&interner, tuple, zero);
    assert_eq!(result, TypeId::STRING);
}

#[test]
fn test_index_access_readonly_tuple_literal() {
    let interner = TypeInterner::new();

    let tuple = interner.tuple(vec![
        TupleElement { type_id: TypeId::STRING, name: None, optional: false, rest: false },
        TupleElement { type_id: TypeId::NUMBER, name: None, optional: false, rest: false },
    ]);
    let readonly_tuple = interner.intern(TypeKey::ReadonlyType(tuple));
    let one = interner.literal_number(1.0);

    let result = evaluate_index_access(&interner, readonly_tuple, one);
    assert_eq!(result, TypeId::NUMBER);
}

#[test]
fn test_index_access_string_number() {
    let interner = TypeInterner::new();

    let result = evaluate_index_access(&interner, TypeId::STRING, TypeId::NUMBER);
    assert_eq!(result, TypeId::STRING);
}

#[test]
fn test_index_access_string_literal_numeric_key() {
    let interner = TypeInterner::new();

    let zero = interner.literal_string("0");
    let result = evaluate_index_access(&interner, TypeId::STRING, zero);
    assert_eq!(result, TypeId::STRING);
}

#[test]
fn test_index_access_string_number_with_no_unchecked_indexed_access() {
    let interner = TypeInterner::new();

    let mut evaluator = TypeEvaluator::new(&interner);
    evaluator.set_no_unchecked_indexed_access(true);

    let result = evaluator.evaluate_index_access(TypeId::STRING, TypeId::NUMBER);
    let expected = interner.union(vec![TypeId::STRING, TypeId::UNDEFINED]);
    assert_eq!(result, expected);
}

#[test]
fn test_index_access_string_literal_numeric_key_with_no_unchecked_indexed_access() {
    let interner = TypeInterner::new();

    let mut evaluator = TypeEvaluator::new(&interner);
    evaluator.set_no_unchecked_indexed_access(true);

    let zero = interner.literal_string("0");
    let result = evaluator.evaluate_index_access(TypeId::STRING, zero);
    let expected = interner.union(vec![TypeId::STRING, TypeId::UNDEFINED]);
    assert_eq!(result, expected);
}

#[test]
fn test_index_access_string_literal_member() {
    let interner = TypeInterner::new();

    let length_key = interner.literal_string("length");
    let length_type = evaluate_index_access(&interner, TypeId::STRING, length_key);
    assert_eq!(length_type, TypeId::NUMBER);

    let to_string_key = interner.literal_string("toString");
    let to_string_type = evaluate_index_access(&interner, TypeId::STRING, to_string_key);
    match interner.lookup(to_string_type) {
        Some(TypeKey::Function(func_id)) => {
            let func = interner.function_shape(func_id);
            assert_eq!(func.return_type, TypeId::STRING);
            assert_eq!(func.params.len(), 1);
            assert!(func.params[0].rest);
        }
        other => panic!("Expected function type, got {:?}", other),
    }
}

#[test]
fn test_index_access_template_literal_members() {
    let interner = TypeInterner::new();

    let template = interner.template_literal(vec![
        TemplateSpan::Text(interner.intern_string("prefix")),
        TemplateSpan::Type(TypeId::STRING),
        TemplateSpan::Text(interner.intern_string("suffix")),
    ]);

    let length_key = interner.literal_string("length");
    let length_type = evaluate_index_access(&interner, template, length_key);
    assert_eq!(length_type, TypeId::NUMBER);

    let number_index = evaluate_index_access(&interner, template, TypeId::NUMBER);
    assert_eq!(number_index, TypeId::STRING);
}

#[test]
fn test_keyof_readonly_array() {
    let interner = TypeInterner::new();

    let array = interner.array(TypeId::STRING);
    let readonly_array = interner.intern(TypeKey::ReadonlyType(array));

    let result = evaluate_keyof(&interner, readonly_array);
    let key = interner.lookup(result).expect("expected union for keyof readonly array");

    match key {
        TypeKey::Union(members) => {
            let members = interner.type_list(members);
            let length = interner.literal_string("length");
            let map = interner.literal_string("map");
            assert!(members.contains(&TypeId::NUMBER));
            assert!(members.contains(&length));
            assert!(members.contains(&map));
        }
        other => panic!("Expected union, got {:?}", other),
    }
}

#[test]
fn test_keyof_readonly_tuple() {
    let interner = TypeInterner::new();

    let tuple = interner.tuple(vec![
        TupleElement { type_id: TypeId::STRING, name: None, optional: false, rest: false },
        TupleElement { type_id: TypeId::NUMBER, name: None, optional: false, rest: false },
    ]);
    let readonly_tuple = interner.intern(TypeKey::ReadonlyType(tuple));

    let result = evaluate_keyof(&interner, readonly_tuple);
    let key = interner.lookup(result).expect("expected union for keyof readonly tuple");

    match key {
        TypeKey::Union(members) => {
            let members = interner.type_list(members);
            let key_0 = interner.literal_string("0");
            let key_1 = interner.literal_string("1");
            let length = interner.literal_string("length");
            let map = interner.literal_string("map");
            assert!(members.contains(&key_0));
            assert!(members.contains(&key_1));
            assert!(members.contains(&TypeId::NUMBER));
            assert!(members.contains(&length));
            assert!(members.contains(&map));
        }
        other => panic!("Expected union, got {:?}", other),
    }
}

#[test]
fn test_keyof_type_param_constraint() {
    let interner = TypeInterner::new();

    let constraint = interner.object(vec![
        PropertyInfo { name: interner.intern_string("x"), type_id: TypeId::NUMBER,
 write_type: TypeId::NUMBER, optional: false, readonly: false, is_method: false },
        PropertyInfo { name: interner.intern_string("y"), type_id: TypeId::STRING,
 write_type: TypeId::STRING, optional: false, readonly: false, is_method: false },
    ]);

    let type_param = interner.intern(TypeKey::TypeParameter(TypeParamInfo {
        name: interner.intern_string("T"),
        constraint: Some(constraint),
        default: None,
    }));

    let result = evaluate_keyof(&interner, type_param);
    let expected = interner.union(vec![
        interner.literal_string("x"),
        interner.literal_string("y"),
    ]);
    assert_eq!(result, expected);
}

#[test]
fn test_base_constraint_assignability_evaluate_keyof() {
    let interner = TypeInterner::new();

    let constraint = interner.object(vec![
        PropertyInfo { name: interner.intern_string("x"), type_id: TypeId::NUMBER,
 write_type: TypeId::NUMBER, optional: false, readonly: false, is_method: false },
        PropertyInfo { name: interner.intern_string("y"), type_id: TypeId::STRING,
 write_type: TypeId::STRING, optional: false, readonly: false, is_method: false },
    ]);

    let type_param = interner.intern(TypeKey::TypeParameter(TypeParamInfo {
        name: interner.intern_string("T"),
        constraint: Some(constraint),
        default: None,
    }));

    let key_of = interner.intern(TypeKey::KeyOf(type_param));
    let result = evaluate_type(&interner, key_of);
    let expected = interner.union(vec![
        interner.literal_string("x"),
        interner.literal_string("y"),
    ]);
    assert_eq!(result, expected);
}

#[test]
fn test_keyof_type_param_no_constraint_deferred() {
    let interner = TypeInterner::new();

    let type_param = interner.intern(TypeKey::TypeParameter(TypeParamInfo {
        name: interner.intern_string("T"),
        constraint: None,
        default: None,
    }));

    let result = evaluate_keyof(&interner, type_param);
    match interner.lookup(result) {
        Some(TypeKey::KeyOf(inner)) => assert_eq!(inner, type_param),
        other => panic!("Expected deferred KeyOf, got {:?}", other),
    }
}

#[test]
fn test_keyof_resolves_ref() {
    use crate::solver::{TypeEnvironment, SymbolRef};

    let interner = TypeInterner::new();
    let mut env = TypeEnvironment::new();

    let obj = interner.object(vec![
        PropertyInfo { name: interner.intern_string("x"), type_id: TypeId::NUMBER,
 write_type: TypeId::NUMBER, optional: false, readonly: false, is_method: false },
        PropertyInfo { name: interner.intern_string("y"), type_id: TypeId::STRING,
 write_type: TypeId::STRING, optional: false, readonly: false, is_method: false },
    ]);

    let sym = SymbolRef(2);
    env.insert(sym, obj);

    let ref_type = interner.reference(sym);
    let evaluator = TypeEvaluator::with_resolver(&interner, &env);
    let result = evaluator.evaluate_keyof(ref_type);

    let expected = interner.union(vec![
        interner.literal_string("x"),
        interner.literal_string("y"),
    ]);
    assert_eq!(result, expected);
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
fn test_index_access_tuple_optional_number() {
    let interner = TypeInterner::new();

    // [string, number?][number] -> string | number | undefined
    let tuple = interner.tuple(vec![
        TupleElement { type_id: TypeId::STRING, name: None, optional: false, rest: false },
        TupleElement { type_id: TypeId::NUMBER, name: None, optional: true, rest: false },
    ]);

    let result = evaluate_index_access(&interner, tuple, TypeId::NUMBER);
    let expected = interner.union(vec![TypeId::STRING, TypeId::NUMBER, TypeId::UNDEFINED]);
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

    let inner_cond = interner.conditional(ConditionalType {
        check_type: TypeId::NUMBER,
        extends_type: TypeId::NUMBER,
        true_type: yes,
        false_type: no,
        is_distributive: false,
    });

    let cond = ConditionalType {
        check_type: TypeId::STRING,
        extends_type: TypeId::STRING,
        true_type: inner_cond,
        false_type: outer_no,
        is_distributive: false,
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
        PropertyInfo { name: interner.intern_string("x"), type_id: TypeId::NUMBER,
 write_type: TypeId::NUMBER, optional: false, readonly: false, is_method: false },
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
        PropertyInfo { name: interner.intern_string("x"), type_id: TypeId::NUMBER,
 write_type: TypeId::NUMBER, optional: false, readonly: false, is_method: false },
        PropertyInfo { name: interner.intern_string("y"), type_id: TypeId::STRING,
 write_type: TypeId::STRING, optional: false, readonly: false, is_method: false },
    ]);

    let result = evaluate_keyof(&interner, obj);

    let key_x = interner.literal_string("x");
    let key_y = interner.literal_string("y");
    let expected = interner.union(vec![key_x, key_y]);
    assert_eq!(result, expected);
}

#[test]
fn test_keyof_object_with_string_index_signature() {
    let interner = TypeInterner::new();

    let key_x = interner.intern_string("x");
    let obj = interner.object_with_index(ObjectShape {
        properties: vec![PropertyInfo {
            name: key_x,
            type_id: TypeId::NUMBER,
            write_type: TypeId::NUMBER,
            optional: false,
            readonly: false,
            is_method: false,
        }],
        string_index: Some(IndexSignature {
            key_type: TypeId::STRING,
            value_type: TypeId::BOOLEAN,
            readonly: false,
        }),
        number_index: None,
    });

    let result = evaluate_keyof(&interner, obj);
    let expected = interner.union(vec![interner.literal_string("x"), TypeId::STRING, TypeId::NUMBER]);
    assert_eq!(result, expected);
}

#[test]
fn test_keyof_object_with_number_index_signature() {
    let interner = TypeInterner::new();

    let key_x = interner.intern_string("x");
    let obj = interner.object_with_index(ObjectShape {
        properties: vec![PropertyInfo {
            name: key_x,
            type_id: TypeId::NUMBER,
            write_type: TypeId::NUMBER,
            optional: false,
            readonly: false,
            is_method: false,
        }],
        string_index: None,
        number_index: Some(IndexSignature {
            key_type: TypeId::NUMBER,
            value_type: TypeId::STRING,
            readonly: false,
        }),
    });

    let result = evaluate_keyof(&interner, obj);
    let expected = interner.union(vec![interner.literal_string("x"), TypeId::NUMBER]);
    assert_eq!(result, expected);
}

#[test]
fn test_keyof_union_disjoint_objects() {
    let interner = TypeInterner::new();

    let obj_a = interner.object(vec![PropertyInfo {
        name: interner.intern_string("a"),
        type_id: TypeId::NUMBER,
        write_type: TypeId::NUMBER,
        optional: false,
        readonly: false,
        is_method: false,
    }]);
    let obj_b = interner.object(vec![PropertyInfo {
        name: interner.intern_string("b"),
        type_id: TypeId::STRING,
        write_type: TypeId::STRING,
        optional: false,
        readonly: false,
        is_method: false,
    }]);

    let union = interner.union(vec![obj_a, obj_b]);
    let result = evaluate_keyof(&interner, union);
    assert_eq!(result, TypeId::NEVER);
}

#[test]
fn test_keyof_union_overlap_objects() {
    let interner = TypeInterner::new();

    let obj_a = interner.object(vec![
        PropertyInfo {
            name: interner.intern_string("a"),
            type_id: TypeId::NUMBER,
            write_type: TypeId::NUMBER,
            optional: false,
            readonly: false,
            is_method: false,
        },
        PropertyInfo {
            name: interner.intern_string("b"),
            type_id: TypeId::STRING,
            write_type: TypeId::STRING,
            optional: false,
            readonly: false,
            is_method: false,
        },
    ]);
    let obj_b = interner.object(vec![
        PropertyInfo {
            name: interner.intern_string("b"),
            type_id: TypeId::BOOLEAN,
            write_type: TypeId::BOOLEAN,
            optional: false,
            readonly: false,
            is_method: false,
        },
        PropertyInfo {
            name: interner.intern_string("c"),
            type_id: TypeId::STRING,
            write_type: TypeId::STRING,
            optional: false,
            readonly: false,
            is_method: false,
        },
    ]);

    let union = interner.union(vec![obj_a, obj_b]);
    let result = evaluate_keyof(&interner, union);
    let expected = interner.literal_string("b");
    assert_eq!(result, expected);
}

#[test]
fn test_keyof_intersection_unions_keys() {
    let interner = TypeInterner::new();

    let obj_a = interner.object(vec![PropertyInfo {
        name: interner.intern_string("a"),
        type_id: TypeId::NUMBER,
        write_type: TypeId::NUMBER,
        optional: false,
        readonly: false,
        is_method: false,
    }]);
    let obj_b = interner.object(vec![PropertyInfo {
        name: interner.intern_string("b"),
        type_id: TypeId::STRING,
        write_type: TypeId::STRING,
        optional: false,
        readonly: false,
        is_method: false,
    }]);

    let intersection = interner.intersection(vec![obj_a, obj_b]);
    let result = evaluate_keyof(&interner, intersection);
    let expected = interner.union(vec![interner.literal_string("a"), interner.literal_string("b")]);
    assert_eq!(result, expected);
}

#[test]
fn test_keyof_union_string_index_overlap_literal() {
    let interner = TypeInterner::new();

    let obj_index = interner.object_with_index(ObjectShape {
        properties: Vec::new(),
        string_index: Some(IndexSignature {
            key_type: TypeId::STRING,
            value_type: TypeId::NUMBER,
            readonly: false,
        }),
        number_index: None,
    });
    let obj_literal = interner.object(vec![PropertyInfo {
        name: interner.intern_string("a"),
        type_id: TypeId::BOOLEAN,
        write_type: TypeId::BOOLEAN,
        optional: false,
        readonly: false,
        is_method: false,
    }]);

    let union = interner.union(vec![obj_index, obj_literal]);
    let result = evaluate_keyof(&interner, union);
    let expected = interner.literal_string("a");
    assert_eq!(result, expected);
}

#[test]
fn test_keyof_union_index_signature_intersection() {
    let interner = TypeInterner::new();

    let string_index = interner.object_with_index(ObjectShape {
        properties: Vec::new(),
        string_index: Some(IndexSignature {
            key_type: TypeId::STRING,
            value_type: TypeId::NUMBER,
            readonly: false,
        }),
        number_index: None,
    });
    let number_index = interner.object_with_index(ObjectShape {
        properties: Vec::new(),
        string_index: None,
        number_index: Some(IndexSignature {
            key_type: TypeId::NUMBER,
            value_type: TypeId::NUMBER,
            readonly: false,
        }),
    });

    let union = interner.union(vec![string_index, number_index]);
    let result = evaluate_keyof(&interner, union);

    assert_eq!(result, TypeId::NUMBER);
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

    // keyof string[] includes number and array members
    let arr = interner.array(TypeId::STRING);

    let result = evaluate_keyof(&interner, arr);
    let key = interner.lookup(result).expect("expected union for keyof array");

    match key {
        TypeKey::Union(members) => {
            let members = interner.type_list(members);
            let length = interner.literal_string("length");
            let map = interner.literal_string("map");
            assert!(members.contains(&TypeId::NUMBER));
            assert!(members.contains(&length));
            assert!(members.contains(&map));
        }
        other => panic!("Expected union, got {:?}", other),
    }
}

#[test]
fn test_keyof_tuple() {
    let interner = TypeInterner::new();

    // keyof [string, number] includes tuple indices and array members
    let tuple = interner.tuple(vec![
        TupleElement { type_id: TypeId::STRING, name: None, optional: false, rest: false },
        TupleElement { type_id: TypeId::NUMBER, name: None, optional: false, rest: false },
    ]);

    let result = evaluate_keyof(&interner, tuple);

    let key = interner.lookup(result).expect("expected union for keyof tuple");

    match key {
        TypeKey::Union(members) => {
            let members = interner.type_list(members);
            let key_0 = interner.literal_string("0");
            let key_1 = interner.literal_string("1");
            let length = interner.literal_string("length");
            let map = interner.literal_string("map");
            assert!(members.contains(&key_0));
            assert!(members.contains(&key_1));
            assert!(members.contains(&TypeId::NUMBER));
            assert!(members.contains(&length));
            assert!(members.contains(&map));
        }
        other => panic!("Expected union, got {:?}", other),
    }
}

#[test]
fn test_keyof_tuple_with_rest_tuple() {
    let interner = TypeInterner::new();

    // keyof [string, ...[number, boolean]] includes expanded indices
    let rest_tuple = interner.tuple(vec![
        TupleElement { type_id: TypeId::NUMBER, name: None, optional: false, rest: false },
        TupleElement { type_id: TypeId::BOOLEAN, name: None, optional: false, rest: false },
    ]);
    let tuple = interner.tuple(vec![
        TupleElement { type_id: TypeId::STRING, name: None, optional: false, rest: false },
        TupleElement { type_id: rest_tuple, name: None, optional: false, rest: true },
    ]);

    let result = evaluate_keyof(&interner, tuple);
    let key = interner.lookup(result).expect("expected union for keyof tuple with rest");

    match key {
        TypeKey::Union(members) => {
            let members = interner.type_list(members);
            let key_0 = interner.literal_string("0");
            let key_1 = interner.literal_string("1");
            let key_2 = interner.literal_string("2");
            let length = interner.literal_string("length");
            assert!(members.contains(&key_0));
            assert!(members.contains(&key_1));
            assert!(members.contains(&key_2));
            assert!(members.contains(&TypeId::NUMBER));
            assert!(members.contains(&length));
        }
        other => panic!("Expected union, got {:?}", other),
    }
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

#[test]
fn test_keyof_object_keyword() {
    let interner = TypeInterner::new();

    // keyof object = never
    let result = evaluate_keyof(&interner, TypeId::OBJECT);
    assert_eq!(result, TypeId::NEVER);
}

#[test]
fn test_object_trifecta_keyof_object_interface() {
    use crate::solver::{TypeEnvironment, SymbolRef};

    let interner = TypeInterner::new();
    let mut env = TypeEnvironment::new();

    let object_interface = interner.object(vec![
        PropertyInfo { name: interner.intern_string("toString"), type_id: TypeId::STRING,
 write_type: TypeId::STRING, optional: false, readonly: false, is_method: false },
        PropertyInfo { name: interner.intern_string("valueOf"), type_id: TypeId::NUMBER,
 write_type: TypeId::NUMBER, optional: false, readonly: false, is_method: false },
    ]);

    let sym = SymbolRef(1);
    env.insert(sym, object_interface);

    let ref_type = interner.reference(sym);
    let evaluator = TypeEvaluator::with_resolver(&interner, &env);
    let result = evaluator.evaluate_keyof(ref_type);
    let key = interner.lookup(result).expect("expected union for keyof Object interface");

    match key {
        TypeKey::Union(members) => {
            let members = interner.type_list(members);
            let to_string = interner.literal_string("toString");
            let value_of = interner.literal_string("valueOf");
            assert!(members.contains(&to_string));
            assert!(members.contains(&value_of));
        }
        other => panic!("Expected union, got {:?}", other),
    }
}

#[test]
fn test_keyof_never() {
    let interner = TypeInterner::new();

    // keyof never = never
    let result = evaluate_keyof(&interner, TypeId::NEVER);
    assert_eq!(result, TypeId::NEVER);
}

#[test]
fn test_keyof_nullish() {
    let interner = TypeInterner::new();

    // keyof null/undefined/void = never
    assert_eq!(evaluate_keyof(&interner, TypeId::NULL), TypeId::NEVER);
    assert_eq!(evaluate_keyof(&interner, TypeId::UNDEFINED), TypeId::NEVER);
    assert_eq!(evaluate_keyof(&interner, TypeId::VOID), TypeId::NEVER);
}

#[test]
fn test_keyof_string_apparent_members() {
    let interner = TypeInterner::new();

    let result = evaluate_keyof(&interner, TypeId::STRING);
    let key = interner.lookup(result).expect("expected union for keyof string");

    match key {
        TypeKey::Union(members) => {
            let members = interner.type_list(members);
            let length = interner.literal_string("length");
            let to_string = interner.literal_string("toString");
            assert!(members.contains(&length));
            assert!(members.contains(&to_string));
            assert!(members.contains(&TypeId::NUMBER));
        }
        other => panic!("Expected union, got {:?}", other),
    }
}

#[test]
fn test_apparent_number_keyof_members() {
    let interner = TypeInterner::new();

    let result = evaluate_keyof(&interner, TypeId::NUMBER);
    let key = interner.lookup(result).expect("expected union for keyof number");

    match key {
        TypeKey::Union(members) => {
            let members = interner.type_list(members);
            let to_fixed = interner.literal_string("toFixed");
            let value_of = interner.literal_string("valueOf");
            assert!(members.contains(&to_fixed));
            assert!(members.contains(&value_of));
        }
        other => panic!("Expected union, got {:?}", other),
    }
}

#[test]
fn test_keyof_template_literal_matches_string() {
    let interner = TypeInterner::new();

    let template = interner.template_literal(vec![
        TemplateSpan::Text(interner.intern_string("prefix")),
        TemplateSpan::Type(TypeId::STRING),
        TemplateSpan::Text(interner.intern_string("suffix")),
    ]);

    let result = evaluate_keyof(&interner, template);
    let expected = evaluate_keyof(&interner, TypeId::STRING);
    assert_eq!(result, expected);
}

#[test]
fn test_intersection_reduction_disjoint_discriminant_evaluates_never() {
    let interner = TypeInterner::new();

    let kind = interner.intern_string("kind");
    let obj_a = interner.object(vec![PropertyInfo {
        name: kind,
        type_id: interner.literal_string("a"),
        write_type: interner.literal_string("a"),
        optional: false,
        readonly: false,
        is_method: false,
    }]);
    let obj_b = interner.object(vec![PropertyInfo {
        name: kind,
        type_id: interner.literal_string("b"),
        write_type: interner.literal_string("b"),
        optional: false,
        readonly: false,
        is_method: false,
    }]);

    let intersection = interner.intersection(vec![obj_a, obj_b]);
    let result = evaluate_type(&interner, intersection);

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
            name: interner.intern_string("K"),
            constraint: None,
            default: None,
        },
        constraint: keys,
        name_type: None,
        template: TypeId::NUMBER,
        readonly_modifier: None,
        optional_modifier: None,
    };

    let result = evaluate_mapped(&interner, &mapped);

    // Result should be { x: number, y: number }
    let expected = interner.object(vec![
        PropertyInfo { name: interner.intern_string("x"), type_id: TypeId::NUMBER,
 write_type: TypeId::NUMBER, optional: false, readonly: false, is_method: false },
        PropertyInfo { name: interner.intern_string("y"), type_id: TypeId::NUMBER,
 write_type: TypeId::NUMBER, optional: false, readonly: false, is_method: false },
    ]);
    assert_eq!(result, expected);
}

#[test]
fn test_mapped_type_over_string_keys() {
    let interner = TypeInterner::new();

    let constraint = interner.intern(TypeKey::KeyOf(TypeId::STRING));
    let mapped = MappedType {
        type_param: TypeParamInfo {
            name: interner.intern_string("K"),
            constraint: None,
            default: None,
        },
        constraint,
        name_type: None,
        template: TypeId::BOOLEAN,
        readonly_modifier: None,
        optional_modifier: None,
    };

    let result = evaluate_mapped(&interner, &mapped);
    let key = interner.lookup(result).expect("Expected object type");

    match key {
        TypeKey::ObjectWithIndex(shape_id) => {
            let shape = interner.object_shape(shape_id);
            let length = interner.intern_string("length");
            let to_string = interner.intern_string("toString");
            let mut saw_length = false;
            let mut saw_to_string = false;

            for prop in &shape.properties {
                if prop.name == length {
                    assert_eq!(prop.type_id, TypeId::BOOLEAN);
                    saw_length = true;
                }
                if prop.name == to_string {
                    assert_eq!(prop.type_id, TypeId::BOOLEAN);
                    saw_to_string = true;
                }
            }

            assert!(saw_length, "missing length property");
            assert!(saw_to_string, "missing toString property");
            let number_index = shape.number_index.as_ref().expect("expected number index signature");
            assert_eq!(number_index.key_type, TypeId::NUMBER);
            assert_eq!(number_index.value_type, TypeId::BOOLEAN);
        }
        other => panic!("Expected object type, got {:?}", other),
    }
}

#[test]
fn test_mapped_type_over_number_keys() {
    let interner = TypeInterner::new();

    let constraint = interner.intern(TypeKey::KeyOf(TypeId::NUMBER));
    let mapped = MappedType {
        type_param: TypeParamInfo {
            name: interner.intern_string("K"),
            constraint: None,
            default: None,
        },
        constraint,
        name_type: None,
        template: TypeId::BOOLEAN,
        readonly_modifier: None,
        optional_modifier: None,
    };

    let result = evaluate_mapped(&interner, &mapped);
    let key = interner.lookup(result).expect("Expected object type");

    match key {
        TypeKey::Object(shape_id) => {
            let shape = interner.object_shape(shape_id);
            let to_fixed = interner.intern_string("toFixed");
            let value_of = interner.intern_string("valueOf");
            let has_own = interner.intern_string("hasOwnProperty");
            let mut saw_to_fixed = false;
            let mut saw_value_of = false;
            let mut saw_has_own = false;

            for prop in &shape.properties {
                if prop.name == to_fixed {
                    assert_eq!(prop.type_id, TypeId::BOOLEAN);
                    saw_to_fixed = true;
                }
                if prop.name == value_of {
                    assert_eq!(prop.type_id, TypeId::BOOLEAN);
                    saw_value_of = true;
                }
                if prop.name == has_own {
                    assert_eq!(prop.type_id, TypeId::BOOLEAN);
                    saw_has_own = true;
                }
            }

            assert!(saw_to_fixed, "missing toFixed property");
            assert!(saw_value_of, "missing valueOf property");
            assert!(saw_has_own, "missing hasOwnProperty property");
        }
        other => panic!("Expected object type, got {:?}", other),
    }
}

#[test]
fn test_mapped_type_over_number_keys_evaluate_type() {
    let interner = TypeInterner::new();

    let constraint = interner.intern(TypeKey::KeyOf(TypeId::NUMBER));
    let mapped = interner.mapped(MappedType {
        type_param: TypeParamInfo {
            name: interner.intern_string("K"),
            constraint: None,
            default: None,
        },
        constraint,
        name_type: None,
        template: TypeId::BOOLEAN,
        readonly_modifier: None,
        optional_modifier: None,
    });

    let result = evaluate_type(&interner, mapped);
    let key = interner.lookup(result).expect("Expected object type");

    match key {
        TypeKey::Object(shape_id) => {
            let shape = interner.object_shape(shape_id);
            let to_fixed = interner.intern_string("toFixed");
            let mut saw_to_fixed = false;

            for prop in &shape.properties {
                if prop.name == to_fixed {
                    assert_eq!(prop.type_id, TypeId::BOOLEAN);
                    saw_to_fixed = true;
                }
            }

            assert!(saw_to_fixed, "missing toFixed property");
        }
        other => panic!("Expected object type, got {:?}", other),
    }
}

#[test]
fn test_mapped_type_over_boolean_keys() {
    let interner = TypeInterner::new();

    let constraint = interner.intern(TypeKey::KeyOf(TypeId::BOOLEAN));
    let mapped = MappedType {
        type_param: TypeParamInfo {
            name: interner.intern_string("K"),
            constraint: None,
            default: None,
        },
        constraint,
        name_type: None,
        template: TypeId::NUMBER,
        readonly_modifier: None,
        optional_modifier: None,
    };

    let result = evaluate_mapped(&interner, &mapped);
    let key = interner.lookup(result).expect("Expected object type");

    match key {
        TypeKey::Object(shape_id) => {
            let shape = interner.object_shape(shape_id);
            let to_string = interner.intern_string("toString");
            let value_of = interner.intern_string("valueOf");
            let has_own = interner.intern_string("hasOwnProperty");
            let mut saw_to_string = false;
            let mut saw_value_of = false;
            let mut saw_has_own = false;

            for prop in &shape.properties {
                if prop.name == to_string {
                    assert_eq!(prop.type_id, TypeId::NUMBER);
                    saw_to_string = true;
                }
                if prop.name == value_of {
                    assert_eq!(prop.type_id, TypeId::NUMBER);
                    saw_value_of = true;
                }
                if prop.name == has_own {
                    assert_eq!(prop.type_id, TypeId::NUMBER);
                    saw_has_own = true;
                }
            }

            assert!(saw_to_string, "missing toString property");
            assert!(saw_value_of, "missing valueOf property");
            assert!(saw_has_own, "missing hasOwnProperty property");
        }
        other => panic!("Expected object type, got {:?}", other),
    }
}

#[test]
fn test_mapped_type_over_symbol_keys() {
    let interner = TypeInterner::new();

    let constraint = interner.intern(TypeKey::KeyOf(TypeId::SYMBOL));
    let mapped = MappedType {
        type_param: TypeParamInfo {
            name: interner.intern_string("K"),
            constraint: None,
            default: None,
        },
        constraint,
        name_type: None,
        template: TypeId::BOOLEAN,
        readonly_modifier: None,
        optional_modifier: None,
    };

    let result = evaluate_mapped(&interner, &mapped);
    let key = interner.lookup(result).expect("Expected object type");

    match key {
        TypeKey::Object(shape_id) => {
            let shape = interner.object_shape(shape_id);
            let description = interner.intern_string("description");
            let to_string = interner.intern_string("toString");
            let value_of = interner.intern_string("valueOf");
            let mut saw_description = false;
            let mut saw_to_string = false;
            let mut saw_value_of = false;

            for prop in &shape.properties {
                if prop.name == description {
                    assert_eq!(prop.type_id, TypeId::BOOLEAN);
                    saw_description = true;
                }
                if prop.name == to_string {
                    assert_eq!(prop.type_id, TypeId::BOOLEAN);
                    saw_to_string = true;
                }
                if prop.name == value_of {
                    assert_eq!(prop.type_id, TypeId::BOOLEAN);
                    saw_value_of = true;
                }
            }

            assert!(saw_description, "missing description property");
            assert!(saw_to_string, "missing toString property");
            assert!(saw_value_of, "missing valueOf property");
        }
        other => panic!("Expected object type, got {:?}", other),
    }
}

#[test]
fn test_mapped_type_over_bigint_keys() {
    let interner = TypeInterner::new();

    let constraint = interner.intern(TypeKey::KeyOf(TypeId::BIGINT));
    let mapped = MappedType {
        type_param: TypeParamInfo {
            name: interner.intern_string("K"),
            constraint: None,
            default: None,
        },
        constraint,
        name_type: None,
        template: TypeId::STRING,
        readonly_modifier: None,
        optional_modifier: None,
    };

    let result = evaluate_mapped(&interner, &mapped);
    let key = interner.lookup(result).expect("Expected object type");

    match key {
        TypeKey::Object(shape_id) => {
            let shape = interner.object_shape(shape_id);
            let to_string = interner.intern_string("toString");
            let value_of = interner.intern_string("valueOf");
            let has_own = interner.intern_string("hasOwnProperty");
            let mut saw_to_string = false;
            let mut saw_value_of = false;
            let mut saw_has_own = false;

            for prop in &shape.properties {
                if prop.name == to_string {
                    assert_eq!(prop.type_id, TypeId::STRING);
                    saw_to_string = true;
                }
                if prop.name == value_of {
                    assert_eq!(prop.type_id, TypeId::STRING);
                    saw_value_of = true;
                }
                if prop.name == has_own {
                    assert_eq!(prop.type_id, TypeId::STRING);
                    saw_has_own = true;
                }
            }

            assert!(saw_to_string, "missing toString property");
            assert!(saw_value_of, "missing valueOf property");
            assert!(saw_has_own, "missing hasOwnProperty property");
        }
        other => panic!("Expected object type, got {:?}", other),
    }
}

#[test]
fn test_mapped_type_string_index_signature() {
    let interner = TypeInterner::new();

    let mapped = MappedType {
        type_param: TypeParamInfo {
            name: interner.intern_string("K"),
            constraint: None,
            default: None,
        },
        constraint: TypeId::STRING,
        name_type: None,
        template: TypeId::NUMBER,
        readonly_modifier: Some(MappedModifier::Add),
        optional_modifier: Some(MappedModifier::Add),
    };

    let result = evaluate_mapped(&interner, &mapped);
    let key = interner.lookup(result).expect("Expected object type");

    match key {
        TypeKey::ObjectWithIndex(shape_id) => {
            let shape = interner.object_shape(shape_id);
            assert!(shape.properties.is_empty());
            assert!(shape.number_index.is_none());

            let string_index = shape.string_index.as_ref().expect("expected string index signature");
            assert_eq!(string_index.key_type, TypeId::STRING);
            let expected_value = interner.union(vec![TypeId::NUMBER, TypeId::UNDEFINED]);
            assert_eq!(string_index.value_type, expected_value);
            assert!(string_index.readonly);
        }
        other => panic!("Expected object type, got {:?}", other),
    }
}

#[test]
fn test_mapped_type_number_index_signature() {
    let interner = TypeInterner::new();

    let mapped = MappedType {
        type_param: TypeParamInfo {
            name: interner.intern_string("K"),
            constraint: None,
            default: None,
        },
        constraint: TypeId::NUMBER,
        name_type: None,
        template: TypeId::STRING,
        readonly_modifier: None,
        optional_modifier: None,
    };

    let result = evaluate_mapped(&interner, &mapped);
    let key = interner.lookup(result).expect("Expected object type");

    match key {
        TypeKey::ObjectWithIndex(shape_id) => {
            let shape = interner.object_shape(shape_id);
            assert!(shape.properties.is_empty());
            assert!(shape.string_index.is_none());

            let number_index = shape.number_index.as_ref().expect("expected number index signature");
            assert_eq!(number_index.key_type, TypeId::NUMBER);
            assert_eq!(number_index.value_type, TypeId::STRING);
            assert!(!number_index.readonly);
        }
        other => panic!("Expected object type, got {:?}", other),
    }
}

#[test]
fn test_mapped_type_single_key() {
    let interner = TypeInterner::new();

    // { [K in "foo"]: string }
    // Should produce { foo: string }
    let key_foo = interner.literal_string("foo");

    let mapped = MappedType {
        type_param: TypeParamInfo {
            name: interner.intern_string("K"),
            constraint: None,
            default: None,
        },
        constraint: key_foo,
        name_type: None,
        template: TypeId::STRING,
        readonly_modifier: None,
        optional_modifier: None,
    };

    let result = evaluate_mapped(&interner, &mapped);

    let expected = interner.object(vec![
        PropertyInfo { name: interner.intern_string("foo"), type_id: TypeId::STRING,
 write_type: TypeId::STRING, optional: false, readonly: false, is_method: false },
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
            name: interner.intern_string("K"),
            constraint: None,
            default: None,
        },
        constraint: keys,
        name_type: None,
        template: TypeId::NUMBER,
        readonly_modifier: None,
        optional_modifier: Some(MappedModifier::Add),
    };

    let result = evaluate_mapped(&interner, &mapped);

    // Result should be { x?: number, y?: number }
    let expected = interner.object(vec![
        PropertyInfo { name: interner.intern_string("x"), type_id: TypeId::NUMBER,
 write_type: TypeId::NUMBER, optional: true, readonly: false, is_method: false },
        PropertyInfo { name: interner.intern_string("y"), type_id: TypeId::NUMBER,
 write_type: TypeId::NUMBER, optional: true, readonly: false, is_method: false },
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
            name: interner.intern_string("K"),
            constraint: None,
            default: None,
        },
        constraint: key_x,
        name_type: None,
        template: TypeId::NUMBER,
        readonly_modifier: Some(MappedModifier::Add),
        optional_modifier: None,
    };

    let result = evaluate_mapped(&interner, &mapped);

    let expected = interner.object(vec![
        PropertyInfo { name: interner.intern_string("x"), type_id: TypeId::NUMBER,
 write_type: TypeId::NUMBER, optional: false, readonly: true, is_method: false },
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
        name: interner.intern_string("K"),
        constraint: None,
        default: None,
    }));

    let mapped = MappedType {
        type_param: TypeParamInfo {
            name: interner.intern_string("K"),
            constraint: None,
            default: None,
        },
        constraint: keys,
        name_type: None,
        template: type_param_k,
        readonly_modifier: None,
        optional_modifier: None,
    };

    let result = evaluate_mapped(&interner, &mapped);

    // Result should be { x: "x", y: "y" }
    let expected = interner.object(vec![
        PropertyInfo { name: interner.intern_string("x"), type_id: key_x,
 write_type: key_x, optional: false, readonly: false, is_method: false },
        PropertyInfo { name: interner.intern_string("y"), type_id: key_y,
 write_type: key_y, optional: false, readonly: false, is_method: false },
    ]);
    assert_eq!(result, expected);
}

#[test]
fn test_mapped_type_key_remap_filters_keys() {
    let interner = TypeInterner::new();

    let prop_a = PropertyInfo {
        name: interner.intern_string("a"),
        type_id: TypeId::STRING,
        write_type: TypeId::STRING,
        optional: false,
        readonly: false,
        is_method: false,
    };
    let prop_b = PropertyInfo {
        name: interner.intern_string("b"),
        type_id: TypeId::NUMBER,
        write_type: TypeId::NUMBER,
        optional: false,
        readonly: false,
        is_method: false,
    };
    let obj = interner.object(vec![prop_a.clone(), prop_b.clone()]);

    let key_a = interner.literal_string("a");
    let key_b = interner.literal_string("b");
    let keys = interner.union(vec![key_a, key_b]);

    let key_param = TypeParamInfo {
        name: interner.intern_string("K"),
        constraint: Some(keys),
        default: None,
    };
    let key_param_id = interner.intern(TypeKey::TypeParameter(key_param.clone()));

    let name_type = interner.conditional(ConditionalType {
        check_type: key_param_id,
        extends_type: key_a,
        true_type: TypeId::NEVER,
        false_type: key_param_id,
        is_distributive: true,
    });
    let template = interner.intern(TypeKey::IndexAccess(obj, key_param_id));

    let mapped = MappedType {
        type_param: key_param,
        constraint: keys,
        name_type: Some(name_type),
        template,
        readonly_modifier: None,
        optional_modifier: None,
    };

    let result = evaluate_mapped(&interner, &mapped);
    let prop_b_index = interner.intern(TypeKey::IndexAccess(obj, key_b));
    let expected = interner.object(vec![PropertyInfo {
        name: prop_b.name,
        type_id: prop_b_index,
        write_type: prop_b_index,
        optional: false,
        readonly: false,
        is_method: false,
    }]);

    assert_eq!(result, expected);
}

#[test]
fn test_mapped_type_deferred() {
    let interner = TypeInterner::new();

    // { [K in T]: number } where T is a type parameter
    // Should remain as mapped type (deferred)
    let type_param_t = interner.intern(TypeKey::TypeParameter(TypeParamInfo {
        name: interner.intern_string("T"),
        constraint: None,
        default: None,
    }));

    let mapped = MappedType {
        type_param: TypeParamInfo {
            name: interner.intern_string("K"),
            constraint: None,
            default: None,
        },
        constraint: type_param_t,
        name_type: None,
        template: TypeId::NUMBER,
        readonly_modifier: None,
        optional_modifier: None,
    };

    let mapped_type = interner.mapped(mapped.clone());
    let result = evaluate_mapped(&interner, &mapped);

    // Should return the same mapped type (deferred)
    assert_eq!(result, mapped_type);
}
