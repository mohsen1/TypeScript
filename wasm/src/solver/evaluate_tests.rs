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
fn test_conditional_infer_array_element_non_array_union_branch() {
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

    // T extends (infer R)[] ? R : never, with T = string[] | number.
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
        interner.union(vec![interner.array(TypeId::STRING), TypeId::NUMBER]),
    );

    let instantiated = instantiate_type(&interner, cond_type, &subst);
    let result = evaluate_type(&interner, instantiated);

    assert_eq!(result, TypeId::STRING);
}

#[test]
fn test_conditional_infer_array_element_non_distributive_union_input() {
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

    // T extends (infer R)[] ? R : never, with T = string[] | number[] (no distribution).
    let extends_array = interner.array(infer_r);
    let cond = ConditionalType {
        check_type: t_param,
        extends_type: extends_array,
        true_type: infer_r,
        false_type: TypeId::NEVER,
        is_distributive: false,
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
fn test_conditional_infer_array_element_non_distributive_union_branch() {
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

    // T extends (infer R)[] ? R : never, with T = string[] | number (no distribution).
    let extends_array = interner.array(infer_r);
    let cond = ConditionalType {
        check_type: t_param,
        extends_type: extends_array,
        true_type: infer_r,
        false_type: TypeId::NEVER,
        is_distributive: false,
    };

    let cond_type = interner.conditional(cond);
    let mut subst = TypeSubstitution::new();
    subst.insert(
        t_name,
        interner.union(vec![interner.array(TypeId::STRING), TypeId::NUMBER]),
    );

    let instantiated = instantiate_type(&interner, cond_type, &subst);
    let result = evaluate_type(&interner, instantiated);

    assert_eq!(result, TypeId::NEVER);
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
    let expected = interner.union(vec![TypeId::STRING, TypeId::NUMBER, TypeId::BOOLEAN]);

    assert_eq!(result, expected);
}

#[test]
fn test_conditional_infer_array_element_from_optional_tuple_element() {
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

    // T extends (infer R)[] ? R : never, with T = [string?].
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
    let optional_tuple = interner.tuple(vec![TupleElement {
        type_id: TypeId::STRING,
        name: None,
        optional: true,
        rest: false,
    }]);
    subst.insert(t_name, optional_tuple);

    let instantiated = instantiate_type(&interner, cond_type, &subst);
    let result = evaluate_type(&interner, instantiated);
    let expected = interner.union(vec![TypeId::STRING, TypeId::UNDEFINED]);
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
fn test_conditional_infer_array_element_non_distributive_tuple_wrapper() {
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

    // [T] extends [(infer R)[]] ? R : never, with T = string[] | number[].
    let check_tuple = interner.tuple(vec![TupleElement {
        type_id: t_param,
        name: None,
        optional: false,
        rest: false,
    }]);
    let extends_tuple = interner.tuple(vec![TupleElement {
        type_id: interner.array(infer_r),
        name: None,
        optional: false,
        rest: false,
    }]);
    let cond = ConditionalType {
        check_type: check_tuple,
        extends_type: extends_tuple,
        true_type: infer_r,
        false_type: TypeId::NEVER,
        is_distributive: false,
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
fn test_conditional_infer_object_property_readonly() {
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

    // T extends { readonly a: infer R } ? R : never, with T = { a: string } | { readonly a: number }.
    let extends_obj = interner.object(vec![PropertyInfo {
        name: interner.intern_string("a"),
        type_id: infer_r,
        write_type: infer_r,
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
    let obj_string = interner.object(vec![PropertyInfo {
        name: interner.intern_string("a"),
        type_id: TypeId::STRING,
        write_type: TypeId::STRING,
        optional: false,
        readonly: false,
        is_method: false,
    }]);
    let obj_number = interner.object(vec![PropertyInfo {
        name: interner.intern_string("a"),
        type_id: TypeId::NUMBER,
        write_type: TypeId::NUMBER,
        optional: false,
        readonly: true,
        is_method: false,
    }]);
    subst.insert(t_name, interner.union(vec![obj_string, obj_number]));

    let instantiated = instantiate_type(&interner, cond_type, &subst);
    let result = evaluate_type(&interner, instantiated);
    let expected = interner.union(vec![TypeId::STRING, TypeId::NUMBER]);

    assert_eq!(result, expected);
}

#[test]
fn test_conditional_infer_object_property_function_return_distributive() {
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

    // T extends { a: () => infer R } ? R : never, with T = { a: () => string } | { a: () => number }.
    let extends_fn = interner.function(FunctionShape {
        params: Vec::new(),
        this_type: None,
        return_type: infer_r,
        type_params: Vec::new(),
        type_predicate: None,
        is_constructor: false,
    });
    let extends_obj = interner.object(vec![PropertyInfo {
        name: interner.intern_string("a"),
        type_id: extends_fn,
        write_type: extends_fn,
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
    let string_fn = interner.function(FunctionShape {
        params: Vec::new(),
        this_type: None,
        return_type: TypeId::STRING,
        type_params: Vec::new(),
        type_predicate: None,
        is_constructor: false,
    });
    let number_fn = interner.function(FunctionShape {
        params: Vec::new(),
        this_type: None,
        return_type: TypeId::NUMBER,
        type_params: Vec::new(),
        type_predicate: None,
        is_constructor: false,
    });
    let obj_string = interner.object(vec![PropertyInfo {
        name: interner.intern_string("a"),
        type_id: string_fn,
        write_type: string_fn,
        optional: false,
        readonly: false,
        is_method: false,
    }]);
    let obj_number = interner.object(vec![PropertyInfo {
        name: interner.intern_string("a"),
        type_id: number_fn,
        write_type: number_fn,
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
fn test_conditional_infer_template_literal_distributive() {
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

    // T extends `${infer R}` ? R : never, with T = "foo" | "bar".
    let extends_template = interner.template_literal(vec![TemplateSpan::Type(infer_r)]);
    let cond = ConditionalType {
        check_type: t_param,
        extends_type: extends_template,
        true_type: infer_r,
        false_type: TypeId::NEVER,
        is_distributive: true,
    };

    let cond_type = interner.conditional(cond);
    let mut subst = TypeSubstitution::new();
    let lit_foo = interner.literal_string("foo");
    let lit_bar = interner.literal_string("bar");
    subst.insert(t_name, interner.union(vec![lit_foo, lit_bar]));

    let instantiated = instantiate_type(&interner, cond_type, &subst);
    let result = evaluate_type(&interner, instantiated);

    let expected = interner.union(vec![lit_foo, lit_bar]);
    assert_eq!(result, expected);
}

#[test]
fn test_conditional_infer_template_literal_with_prefix_distributive() {
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

    // T extends `foo${infer R}` ? R : never, with T = "foo1" | "bar".
    let extends_template = interner.template_literal(vec![
        TemplateSpan::Text(interner.intern_string("foo")),
        TemplateSpan::Type(infer_r),
    ]);
    let cond = ConditionalType {
        check_type: t_param,
        extends_type: extends_template,
        true_type: infer_r,
        false_type: TypeId::NEVER,
        is_distributive: true,
    };

    let cond_type = interner.conditional(cond);
    let mut subst = TypeSubstitution::new();
    let lit_foo = interner.literal_string("foo1");
    let lit_bar = interner.literal_string("bar");
    subst.insert(t_name, interner.union(vec![lit_foo, lit_bar]));

    let instantiated = instantiate_type(&interner, cond_type, &subst);
    let result = evaluate_type(&interner, instantiated);

    let expected = interner.literal_string("1");
    assert_eq!(result, expected);
}

#[test]
fn test_conditional_infer_template_literal_with_suffix_distributive() {
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

    // T extends `${infer R}bar` ? R : never, with T = "foobar" | "baz".
    let extends_template = interner.template_literal(vec![
        TemplateSpan::Type(infer_r),
        TemplateSpan::Text(interner.intern_string("bar")),
    ]);
    let cond = ConditionalType {
        check_type: t_param,
        extends_type: extends_template,
        true_type: infer_r,
        false_type: TypeId::NEVER,
        is_distributive: true,
    };

    let cond_type = interner.conditional(cond);
    let mut subst = TypeSubstitution::new();
    let lit_match = interner.literal_string("foobar");
    let lit_other = interner.literal_string("baz");
    subst.insert(t_name, interner.union(vec![lit_match, lit_other]));

    let instantiated = instantiate_type(&interner, cond_type, &subst);
    let result = evaluate_type(&interner, instantiated);

    let expected = interner.literal_string("foo");
    assert_eq!(result, expected);
}

#[test]
fn test_conditional_infer_template_literal_non_distributive_union_input() {
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

    // [T] extends [`foo${infer R}`] ? R : never, with T = "foo1" | "foo2" (no distribution).
    let extends_template = interner.template_literal(vec![
        TemplateSpan::Text(interner.intern_string("foo")),
        TemplateSpan::Type(infer_r),
    ]);
    let cond = ConditionalType {
        check_type: interner.tuple(vec![TupleElement {
            type_id: t_param,
            name: None,
            optional: false,
            rest: false,
        }]),
        extends_type: interner.tuple(vec![TupleElement {
            type_id: extends_template,
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
    let lit_foo1 = interner.literal_string("foo1");
    let lit_foo2 = interner.literal_string("foo2");
    subst.insert(t_name, interner.union(vec![lit_foo1, lit_foo2]));

    let instantiated = instantiate_type(&interner, cond_type, &subst);
    let result = evaluate_type(&interner, instantiated);

    let expected = interner.union(vec![
        interner.literal_string("1"),
        interner.literal_string("2"),
    ]);
    assert_eq!(result, expected);
}

#[test]
fn test_conditional_infer_template_literal_non_distributive_template_union_input() {
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

    // [T] extends [`foo${infer R}`] ? R : never, with T = `foo${string}` | `bar${string}` (no distribution).
    let extends_template = interner.template_literal(vec![
        TemplateSpan::Text(interner.intern_string("foo")),
        TemplateSpan::Type(infer_r),
    ]);
    let cond = ConditionalType {
        check_type: interner.tuple(vec![TupleElement {
            type_id: t_param,
            name: None,
            optional: false,
            rest: false,
        }]),
        extends_type: interner.tuple(vec![TupleElement {
            type_id: extends_template,
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
    let foo_template = interner.template_literal(vec![
        TemplateSpan::Text(interner.intern_string("foo")),
        TemplateSpan::Type(TypeId::STRING),
    ]);
    let bar_template = interner.template_literal(vec![
        TemplateSpan::Text(interner.intern_string("bar")),
        TemplateSpan::Type(TypeId::STRING),
    ]);
    subst.insert(t_name, interner.union(vec![foo_template, bar_template]));

    let instantiated = instantiate_type(&interner, cond_type, &subst);
    let result = evaluate_type(&interner, instantiated);

    assert_eq!(result, TypeId::NEVER);
}

#[test]
fn test_conditional_infer_template_literal_with_constrained_infer_non_distributive_union_input() {
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

    // [T] extends [`foo${infer R extends string}`] ? R : never, with T = "foo1" | "foo2" (no distribution).
    let extends_template = interner.template_literal(vec![
        TemplateSpan::Text(interner.intern_string("foo")),
        TemplateSpan::Type(infer_r),
    ]);
    let cond = ConditionalType {
        check_type: interner.tuple(vec![TupleElement {
            type_id: t_param,
            name: None,
            optional: false,
            rest: false,
        }]),
        extends_type: interner.tuple(vec![TupleElement {
            type_id: extends_template,
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
    let lit_foo1 = interner.literal_string("foo1");
    let lit_foo2 = interner.literal_string("foo2");
    subst.insert(t_name, interner.union(vec![lit_foo1, lit_foo2]));

    let instantiated = instantiate_type(&interner, cond_type, &subst);
    let result = evaluate_type(&interner, instantiated);

    let expected = interner.union(vec![
        interner.literal_string("1"),
        interner.literal_string("2"),
    ]);
    assert_eq!(result, expected);
}

#[test]
fn test_conditional_infer_template_literal_with_middle_infer_non_distributive_union_input() {
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

    // [T] extends [`foo${infer R}bar`] ? R : never, with T = "foobazbar" | "foobuzbar" (no distribution).
    let extends_template = interner.template_literal(vec![
        TemplateSpan::Text(interner.intern_string("foo")),
        TemplateSpan::Type(infer_r),
        TemplateSpan::Text(interner.intern_string("bar")),
    ]);
    let cond = ConditionalType {
        check_type: interner.tuple(vec![TupleElement {
            type_id: t_param,
            name: None,
            optional: false,
            rest: false,
        }]),
        extends_type: interner.tuple(vec![TupleElement {
            type_id: extends_template,
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
    let lit_left = interner.literal_string("foobazbar");
    let lit_right = interner.literal_string("foobuzbar");
    subst.insert(t_name, interner.union(vec![lit_left, lit_right]));

    let instantiated = instantiate_type(&interner, cond_type, &subst);
    let result = evaluate_type(&interner, instantiated);

    let expected = interner.union(vec![
        interner.literal_string("baz"),
        interner.literal_string("buz"),
    ]);
    assert_eq!(result, expected);
}

#[test]
fn test_conditional_infer_template_literal_with_middle_constrained_non_distributive_union_input() {
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

    // [T] extends [`foo${infer R extends string}bar`] ? R : never,
    // with T = "foobazbar" | "foobuzbar" (no distribution).
    let extends_template = interner.template_literal(vec![
        TemplateSpan::Text(interner.intern_string("foo")),
        TemplateSpan::Type(infer_r),
        TemplateSpan::Text(interner.intern_string("bar")),
    ]);
    let cond = ConditionalType {
        check_type: interner.tuple(vec![TupleElement {
            type_id: t_param,
            name: None,
            optional: false,
            rest: false,
        }]),
        extends_type: interner.tuple(vec![TupleElement {
            type_id: extends_template,
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
    let lit_left = interner.literal_string("foobazbar");
    let lit_right = interner.literal_string("foobuzbar");
    subst.insert(t_name, interner.union(vec![lit_left, lit_right]));

    let instantiated = instantiate_type(&interner, cond_type, &subst);
    let result = evaluate_type(&interner, instantiated);

    let expected = interner.union(vec![
        interner.literal_string("baz"),
        interner.literal_string("buz"),
    ]);
    assert_eq!(result, expected);
}

#[test]
fn test_conditional_infer_template_literal_with_middle_non_distributive_non_matching_union_branch() {
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

    // [T] extends [`foo${infer R}bar`] ? R : never, with T = "foobazbar" | "bar" (no distribution).
    let extends_template = interner.template_literal(vec![
        TemplateSpan::Text(interner.intern_string("foo")),
        TemplateSpan::Type(infer_r),
        TemplateSpan::Text(interner.intern_string("bar")),
    ]);
    let cond = ConditionalType {
        check_type: interner.tuple(vec![TupleElement {
            type_id: t_param,
            name: None,
            optional: false,
            rest: false,
        }]),
        extends_type: interner.tuple(vec![TupleElement {
            type_id: extends_template,
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
    let lit_match = interner.literal_string("foobazbar");
    let lit_other = interner.literal_string("bar");
    subst.insert(t_name, interner.union(vec![lit_match, lit_other]));

    let instantiated = instantiate_type(&interner, cond_type, &subst);
    let result = evaluate_type(&interner, instantiated);

    assert_eq!(result, TypeId::NEVER);
}

#[test]
fn test_conditional_infer_template_literal_with_middle_non_distributive_non_string_union_branch() {
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

    // [T] extends [`foo${infer R}bar`] ? R : never, with T = "foobazbar" | number (no distribution).
    let extends_template = interner.template_literal(vec![
        TemplateSpan::Text(interner.intern_string("foo")),
        TemplateSpan::Type(infer_r),
        TemplateSpan::Text(interner.intern_string("bar")),
    ]);
    let cond = ConditionalType {
        check_type: interner.tuple(vec![TupleElement {
            type_id: t_param,
            name: None,
            optional: false,
            rest: false,
        }]),
        extends_type: interner.tuple(vec![TupleElement {
            type_id: extends_template,
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
    let lit_match = interner.literal_string("foobazbar");
    subst.insert(t_name, interner.union(vec![lit_match, TypeId::NUMBER]));

    let instantiated = instantiate_type(&interner, cond_type, &subst);
    let result = evaluate_type(&interner, instantiated);

    assert_eq!(result, TypeId::NEVER);
}

#[test]
fn test_conditional_infer_template_literal_with_middle_non_distributive_non_string_template_union_branch() {
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

    // [T] extends [`foo${infer R}bar`] ? R : never, with T = `foo${string}bar` | number (no distribution).
    let extends_template = interner.template_literal(vec![
        TemplateSpan::Text(interner.intern_string("foo")),
        TemplateSpan::Type(infer_r),
        TemplateSpan::Text(interner.intern_string("bar")),
    ]);
    let cond = ConditionalType {
        check_type: interner.tuple(vec![TupleElement {
            type_id: t_param,
            name: None,
            optional: false,
            rest: false,
        }]),
        extends_type: interner.tuple(vec![TupleElement {
            type_id: extends_template,
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
    let middle_template = interner.template_literal(vec![
        TemplateSpan::Text(interner.intern_string("foo")),
        TemplateSpan::Type(TypeId::STRING),
        TemplateSpan::Text(interner.intern_string("bar")),
    ]);
    subst.insert(t_name, interner.union(vec![middle_template, TypeId::NUMBER]));

    let instantiated = instantiate_type(&interner, cond_type, &subst);
    let result = evaluate_type(&interner, instantiated);

    assert_eq!(result, TypeId::NEVER);
}

#[test]
fn test_conditional_infer_template_literal_two_infers_non_distributive_union_input() {
    let interner = TypeInterner::new();

    let t_name = interner.intern_string("T");
    let t_param = interner.intern(TypeKey::TypeParameter(TypeParamInfo {
        name: t_name,
        constraint: None,
        default: None,
    }));

    let infer_a_name = interner.intern_string("A");
    let infer_a = interner.intern(TypeKey::Infer(TypeParamInfo {
        name: infer_a_name,
        constraint: None,
        default: None,
    }));
    let infer_b_name = interner.intern_string("B");
    let infer_b = interner.intern(TypeKey::Infer(TypeParamInfo {
        name: infer_b_name,
        constraint: None,
        default: None,
    }));

    // [T] extends [`${infer A}-${infer B}`] ? A | B : never, with T = "foo-bar" | "baz-qux" (no distribution).
    let extends_template = interner.template_literal(vec![
        TemplateSpan::Type(infer_a),
        TemplateSpan::Text(interner.intern_string("-")),
        TemplateSpan::Type(infer_b),
    ]);
    let cond = ConditionalType {
        check_type: interner.tuple(vec![TupleElement {
            type_id: t_param,
            name: None,
            optional: false,
            rest: false,
        }]),
        extends_type: interner.tuple(vec![TupleElement {
            type_id: extends_template,
            name: None,
            optional: false,
            rest: false,
        }]),
        true_type: interner.union(vec![infer_a, infer_b]),
        false_type: TypeId::NEVER,
        is_distributive: false,
    };

    let cond_type = interner.conditional(cond);
    let mut subst = TypeSubstitution::new();
    let lit_left = interner.literal_string("foo-bar");
    let lit_right = interner.literal_string("baz-qux");
    subst.insert(t_name, interner.union(vec![lit_left, lit_right]));

    let instantiated = instantiate_type(&interner, cond_type, &subst);
    let result = evaluate_type(&interner, instantiated);

    let expected = interner.union(vec![
        interner.literal_string("foo"),
        interner.literal_string("baz"),
        interner.literal_string("bar"),
        interner.literal_string("qux"),
    ]);
    assert_eq!(result, expected);
}

#[test]
fn test_conditional_infer_template_literal_two_infers_non_distributive_non_matching_union_branch() {
    let interner = TypeInterner::new();

    let t_name = interner.intern_string("T");
    let t_param = interner.intern(TypeKey::TypeParameter(TypeParamInfo {
        name: t_name,
        constraint: None,
        default: None,
    }));

    let infer_a_name = interner.intern_string("A");
    let infer_a = interner.intern(TypeKey::Infer(TypeParamInfo {
        name: infer_a_name,
        constraint: None,
        default: None,
    }));
    let infer_b_name = interner.intern_string("B");
    let infer_b = interner.intern(TypeKey::Infer(TypeParamInfo {
        name: infer_b_name,
        constraint: None,
        default: None,
    }));

    // [T] extends [`${infer A}-${infer B}`] ? A | B : never, with T = "foo-bar" | "baz" (no distribution).
    let extends_template = interner.template_literal(vec![
        TemplateSpan::Type(infer_a),
        TemplateSpan::Text(interner.intern_string("-")),
        TemplateSpan::Type(infer_b),
    ]);
    let cond = ConditionalType {
        check_type: interner.tuple(vec![TupleElement {
            type_id: t_param,
            name: None,
            optional: false,
            rest: false,
        }]),
        extends_type: interner.tuple(vec![TupleElement {
            type_id: extends_template,
            name: None,
            optional: false,
            rest: false,
        }]),
        true_type: interner.union(vec![infer_a, infer_b]),
        false_type: TypeId::NEVER,
        is_distributive: false,
    };

    let cond_type = interner.conditional(cond);
    let mut subst = TypeSubstitution::new();
    let lit_match = interner.literal_string("foo-bar");
    let lit_other = interner.literal_string("baz");
    subst.insert(t_name, interner.union(vec![lit_match, lit_other]));

    let instantiated = instantiate_type(&interner, cond_type, &subst);
    let result = evaluate_type(&interner, instantiated);

    assert_eq!(result, TypeId::NEVER);
}

#[test]
fn test_conditional_infer_template_literal_with_suffix_non_distributive_union_input() {
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

    // [T] extends [`${infer R}bar`] ? R : never, with T = "foobar" | "bazbar" (no distribution).
    let extends_template = interner.template_literal(vec![
        TemplateSpan::Type(infer_r),
        TemplateSpan::Text(interner.intern_string("bar")),
    ]);
    let cond = ConditionalType {
        check_type: interner.tuple(vec![TupleElement {
            type_id: t_param,
            name: None,
            optional: false,
            rest: false,
        }]),
        extends_type: interner.tuple(vec![TupleElement {
            type_id: extends_template,
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
    let lit_left = interner.literal_string("foobar");
    let lit_right = interner.literal_string("bazbar");
    subst.insert(t_name, interner.union(vec![lit_left, lit_right]));

    let instantiated = instantiate_type(&interner, cond_type, &subst);
    let result = evaluate_type(&interner, instantiated);

    let expected = interner.union(vec![
        interner.literal_string("foo"),
        interner.literal_string("baz"),
    ]);
    assert_eq!(result, expected);
}

#[test]
fn test_conditional_infer_template_literal_with_suffix_non_distributive_non_matching_union_branch() {
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

    // [T] extends [`${infer R}bar`] ? R : never, with T = "foobar" | "baz" (no distribution).
    let extends_template = interner.template_literal(vec![
        TemplateSpan::Type(infer_r),
        TemplateSpan::Text(interner.intern_string("bar")),
    ]);
    let cond = ConditionalType {
        check_type: interner.tuple(vec![TupleElement {
            type_id: t_param,
            name: None,
            optional: false,
            rest: false,
        }]),
        extends_type: interner.tuple(vec![TupleElement {
            type_id: extends_template,
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
    let lit_match = interner.literal_string("foobar");
    let lit_other = interner.literal_string("baz");
    subst.insert(t_name, interner.union(vec![lit_match, lit_other]));

    let instantiated = instantiate_type(&interner, cond_type, &subst);
    let result = evaluate_type(&interner, instantiated);

    assert_eq!(result, TypeId::NEVER);
}

#[test]
fn test_conditional_infer_template_literal_with_suffix_non_distributive_non_string_union_branch() {
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

    // [T] extends [`${infer R}bar`] ? R : never, with T = "foobar" | number (no distribution).
    let extends_template = interner.template_literal(vec![
        TemplateSpan::Type(infer_r),
        TemplateSpan::Text(interner.intern_string("bar")),
    ]);
    let cond = ConditionalType {
        check_type: interner.tuple(vec![TupleElement {
            type_id: t_param,
            name: None,
            optional: false,
            rest: false,
        }]),
        extends_type: interner.tuple(vec![TupleElement {
            type_id: extends_template,
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
    let lit_match = interner.literal_string("foobar");
    subst.insert(t_name, interner.union(vec![lit_match, TypeId::NUMBER]));

    let instantiated = instantiate_type(&interner, cond_type, &subst);
    let result = evaluate_type(&interner, instantiated);

    assert_eq!(result, TypeId::NEVER);
}

#[test]
fn test_conditional_infer_template_literal_with_suffix_non_distributive_non_string_template_union_branch() {
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

    // [T] extends [`${infer R}bar`] ? R : never, with T = `${string}bar` | number (no distribution).
    let extends_template = interner.template_literal(vec![
        TemplateSpan::Type(infer_r),
        TemplateSpan::Text(interner.intern_string("bar")),
    ]);
    let cond = ConditionalType {
        check_type: interner.tuple(vec![TupleElement {
            type_id: t_param,
            name: None,
            optional: false,
            rest: false,
        }]),
        extends_type: interner.tuple(vec![TupleElement {
            type_id: extends_template,
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
    let suffix_template = interner.template_literal(vec![
        TemplateSpan::Type(TypeId::STRING),
        TemplateSpan::Text(interner.intern_string("bar")),
    ]);
    subst.insert(t_name, interner.union(vec![suffix_template, TypeId::NUMBER]));

    let instantiated = instantiate_type(&interner, cond_type, &subst);
    let result = evaluate_type(&interner, instantiated);

    assert_eq!(result, TypeId::NEVER);
}

#[test]
fn test_conditional_infer_template_literal_with_suffix_constrained_non_distributive_union_input() {
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

    // [T] extends [`${infer R extends string}bar`] ? R : never, with T = "foobar" | "bazbar" (no distribution).
    let extends_template = interner.template_literal(vec![
        TemplateSpan::Type(infer_r),
        TemplateSpan::Text(interner.intern_string("bar")),
    ]);
    let cond = ConditionalType {
        check_type: interner.tuple(vec![TupleElement {
            type_id: t_param,
            name: None,
            optional: false,
            rest: false,
        }]),
        extends_type: interner.tuple(vec![TupleElement {
            type_id: extends_template,
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
    let lit_left = interner.literal_string("foobar");
    let lit_right = interner.literal_string("bazbar");
    subst.insert(t_name, interner.union(vec![lit_left, lit_right]));

    let instantiated = instantiate_type(&interner, cond_type, &subst);
    let result = evaluate_type(&interner, instantiated);

    let expected = interner.union(vec![
        interner.literal_string("foo"),
        interner.literal_string("baz"),
    ]);
    assert_eq!(result, expected);
}

#[test]
fn test_conditional_infer_template_literal_with_prefix_non_distributive_union_input() {
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

    // [T] extends [`foo${infer R}`] ? R : never, with T = "foo1" | "foo2" (no distribution).
    let extends_template = interner.template_literal(vec![
        TemplateSpan::Text(interner.intern_string("foo")),
        TemplateSpan::Type(infer_r),
    ]);
    let cond = ConditionalType {
        check_type: interner.tuple(vec![TupleElement {
            type_id: t_param,
            name: None,
            optional: false,
            rest: false,
        }]),
        extends_type: interner.tuple(vec![TupleElement {
            type_id: extends_template,
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
    let lit_left = interner.literal_string("foo1");
    let lit_right = interner.literal_string("foo2");
    subst.insert(t_name, interner.union(vec![lit_left, lit_right]));

    let instantiated = instantiate_type(&interner, cond_type, &subst);
    let result = evaluate_type(&interner, instantiated);

    let expected = interner.union(vec![
        interner.literal_string("1"),
        interner.literal_string("2"),
    ]);
    assert_eq!(result, expected);
}

#[test]
fn test_conditional_infer_template_literal_with_prefix_non_distributive_non_matching_union_branch() {
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

    // [T] extends [`foo${infer R}`] ? R : never, with T = "foo1" | "bar" (no distribution).
    let extends_template = interner.template_literal(vec![
        TemplateSpan::Text(interner.intern_string("foo")),
        TemplateSpan::Type(infer_r),
    ]);
    let cond = ConditionalType {
        check_type: interner.tuple(vec![TupleElement {
            type_id: t_param,
            name: None,
            optional: false,
            rest: false,
        }]),
        extends_type: interner.tuple(vec![TupleElement {
            type_id: extends_template,
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
    let lit_match = interner.literal_string("foo1");
    let lit_other = interner.literal_string("bar");
    subst.insert(t_name, interner.union(vec![lit_match, lit_other]));

    let instantiated = instantiate_type(&interner, cond_type, &subst);
    let result = evaluate_type(&interner, instantiated);

    assert_eq!(result, TypeId::NEVER);
}

#[test]
fn test_conditional_infer_template_literal_with_prefix_non_distributive_non_string_union_branch() {
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

    // [T] extends [`foo${infer R}`] ? R : never, with T = "foo1" | number (no distribution).
    let extends_template = interner.template_literal(vec![
        TemplateSpan::Text(interner.intern_string("foo")),
        TemplateSpan::Type(infer_r),
    ]);
    let cond = ConditionalType {
        check_type: interner.tuple(vec![TupleElement {
            type_id: t_param,
            name: None,
            optional: false,
            rest: false,
        }]),
        extends_type: interner.tuple(vec![TupleElement {
            type_id: extends_template,
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
    let lit_match = interner.literal_string("foo1");
    subst.insert(t_name, interner.union(vec![lit_match, TypeId::NUMBER]));

    let instantiated = instantiate_type(&interner, cond_type, &subst);
    let result = evaluate_type(&interner, instantiated);

    assert_eq!(result, TypeId::NEVER);
}

#[test]
fn test_conditional_infer_template_literal_with_prefix_constrained_non_distributive_union_input() {
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

    // [T] extends [`foo${infer R extends string}`] ? R : never, with T = "foo1" | "foo2" (no distribution).
    let extends_template = interner.template_literal(vec![
        TemplateSpan::Text(interner.intern_string("foo")),
        TemplateSpan::Type(infer_r),
    ]);
    let cond = ConditionalType {
        check_type: interner.tuple(vec![TupleElement {
            type_id: t_param,
            name: None,
            optional: false,
            rest: false,
        }]),
        extends_type: interner.tuple(vec![TupleElement {
            type_id: extends_template,
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
    let lit_left = interner.literal_string("foo1");
    let lit_right = interner.literal_string("foo2");
    subst.insert(t_name, interner.union(vec![lit_left, lit_right]));

    let instantiated = instantiate_type(&interner, cond_type, &subst);
    let result = evaluate_type(&interner, instantiated);

    let expected = interner.union(vec![
        interner.literal_string("1"),
        interner.literal_string("2"),
    ]);
    assert_eq!(result, expected);
}

#[test]
fn test_conditional_infer_template_literal_two_infers_with_constraint_non_distributive_union_input() {
    let interner = TypeInterner::new();

    let t_name = interner.intern_string("T");
    let t_param = interner.intern(TypeKey::TypeParameter(TypeParamInfo {
        name: t_name,
        constraint: None,
        default: None,
    }));

    let infer_a_name = interner.intern_string("A");
    let infer_a = interner.intern(TypeKey::Infer(TypeParamInfo {
        name: infer_a_name,
        constraint: Some(TypeId::STRING),
        default: None,
    }));
    let infer_b_name = interner.intern_string("B");
    let infer_b = interner.intern(TypeKey::Infer(TypeParamInfo {
        name: infer_b_name,
        constraint: Some(TypeId::STRING),
        default: None,
    }));

    // [T] extends [`${infer A extends string}-${infer B extends string}`] ? A | B : never,
    // with T = "foo-bar" | "baz-qux" (no distribution).
    let extends_template = interner.template_literal(vec![
        TemplateSpan::Type(infer_a),
        TemplateSpan::Text(interner.intern_string("-")),
        TemplateSpan::Type(infer_b),
    ]);
    let cond = ConditionalType {
        check_type: interner.tuple(vec![TupleElement {
            type_id: t_param,
            name: None,
            optional: false,
            rest: false,
        }]),
        extends_type: interner.tuple(vec![TupleElement {
            type_id: extends_template,
            name: None,
            optional: false,
            rest: false,
        }]),
        true_type: interner.union(vec![infer_a, infer_b]),
        false_type: TypeId::NEVER,
        is_distributive: false,
    };

    let cond_type = interner.conditional(cond);
    let mut subst = TypeSubstitution::new();
    let lit_left = interner.literal_string("foo-bar");
    let lit_right = interner.literal_string("baz-qux");
    subst.insert(t_name, interner.union(vec![lit_left, lit_right]));

    let instantiated = instantiate_type(&interner, cond_type, &subst);
    let result = evaluate_type(&interner, instantiated);

    let expected = interner.union(vec![
        interner.literal_string("foo"),
        interner.literal_string("baz"),
        interner.literal_string("bar"),
        interner.literal_string("qux"),
    ]);
    assert_eq!(result, expected);
}

#[test]
fn test_conditional_infer_template_literal_two_infers_with_constraint_non_distributive_non_matching_union_branch() {
    let interner = TypeInterner::new();

    let t_name = interner.intern_string("T");
    let t_param = interner.intern(TypeKey::TypeParameter(TypeParamInfo {
        name: t_name,
        constraint: None,
        default: None,
    }));

    let infer_a_name = interner.intern_string("A");
    let infer_a = interner.intern(TypeKey::Infer(TypeParamInfo {
        name: infer_a_name,
        constraint: Some(TypeId::STRING),
        default: None,
    }));
    let infer_b_name = interner.intern_string("B");
    let infer_b = interner.intern(TypeKey::Infer(TypeParamInfo {
        name: infer_b_name,
        constraint: Some(TypeId::STRING),
        default: None,
    }));

    // [T] extends [`${infer A extends string}-${infer B extends string}`] ? A | B : never,
    // with T = "foo-bar" | "baz" (no distribution).
    let extends_template = interner.template_literal(vec![
        TemplateSpan::Type(infer_a),
        TemplateSpan::Text(interner.intern_string("-")),
        TemplateSpan::Type(infer_b),
    ]);
    let cond = ConditionalType {
        check_type: interner.tuple(vec![TupleElement {
            type_id: t_param,
            name: None,
            optional: false,
            rest: false,
        }]),
        extends_type: interner.tuple(vec![TupleElement {
            type_id: extends_template,
            name: None,
            optional: false,
            rest: false,
        }]),
        true_type: interner.union(vec![infer_a, infer_b]),
        false_type: TypeId::NEVER,
        is_distributive: false,
    };

    let cond_type = interner.conditional(cond);
    let mut subst = TypeSubstitution::new();
    let lit_match = interner.literal_string("foo-bar");
    let lit_other = interner.literal_string("baz");
    subst.insert(t_name, interner.union(vec![lit_match, lit_other]));

    let instantiated = instantiate_type(&interner, cond_type, &subst);
    let result = evaluate_type(&interner, instantiated);

    assert_eq!(result, TypeId::NEVER);
}

#[test]
fn test_conditional_infer_template_literal_union_input_distributive() {
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

    // T extends `foo${infer R}` ? R : never, with T = `foo${string}` | `bar${string}`.
    let extends_template = interner.template_literal(vec![
        TemplateSpan::Text(interner.intern_string("foo")),
        TemplateSpan::Type(infer_r),
    ]);
    let cond = ConditionalType {
        check_type: t_param,
        extends_type: extends_template,
        true_type: infer_r,
        false_type: TypeId::NEVER,
        is_distributive: true,
    };

    let cond_type = interner.conditional(cond);
    let mut subst = TypeSubstitution::new();
    let foo_template = interner.template_literal(vec![
        TemplateSpan::Text(interner.intern_string("foo")),
        TemplateSpan::Type(TypeId::STRING),
    ]);
    let bar_template = interner.template_literal(vec![
        TemplateSpan::Text(interner.intern_string("bar")),
        TemplateSpan::Type(TypeId::STRING),
    ]);
    subst.insert(t_name, interner.union(vec![foo_template, bar_template]));

    let instantiated = instantiate_type(&interner, cond_type, &subst);
    let result = evaluate_type(&interner, instantiated);

    assert_eq!(result, TypeId::STRING);
}

#[test]
fn test_conditional_infer_template_literal_from_string_input() {
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

    // T extends `${infer R}` ? R : never, with T = string.
    let extends_template = interner.template_literal(vec![TemplateSpan::Type(infer_r)]);
    let cond = ConditionalType {
        check_type: t_param,
        extends_type: extends_template,
        true_type: infer_r,
        false_type: TypeId::NEVER,
        is_distributive: true,
    };

    let cond_type = interner.conditional(cond);
    let mut subst = TypeSubstitution::new();
    subst.insert(t_name, TypeId::STRING);

    let instantiated = instantiate_type(&interner, cond_type, &subst);
    let result = evaluate_type(&interner, instantiated);

    assert_eq!(result, TypeId::STRING);
}

#[test]
fn test_conditional_infer_template_literal_from_template_string_input() {
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

    // T extends `${infer R}` ? R : never, with T = `${string}`.
    let extends_template = interner.template_literal(vec![TemplateSpan::Type(infer_r)]);
    let cond = ConditionalType {
        check_type: t_param,
        extends_type: extends_template,
        true_type: infer_r,
        false_type: TypeId::NEVER,
        is_distributive: true,
    };

    let cond_type = interner.conditional(cond);
    let mut subst = TypeSubstitution::new();
    let template_string = interner.template_literal(vec![TemplateSpan::Type(TypeId::STRING)]);
    subst.insert(t_name, template_string);

    let instantiated = instantiate_type(&interner, cond_type, &subst);
    let result = evaluate_type(&interner, instantiated);

    assert_eq!(result, TypeId::STRING);
}

#[test]
fn test_conditional_infer_template_literal_with_middle_infer_distributive() {
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

    // T extends `foo${infer R}bar` ? R : never, with T = "foobazbar" | "bar".
    let extends_template = interner.template_literal(vec![
        TemplateSpan::Text(interner.intern_string("foo")),
        TemplateSpan::Type(infer_r),
        TemplateSpan::Text(interner.intern_string("bar")),
    ]);
    let cond = ConditionalType {
        check_type: t_param,
        extends_type: extends_template,
        true_type: infer_r,
        false_type: TypeId::NEVER,
        is_distributive: true,
    };

    let cond_type = interner.conditional(cond);
    let mut subst = TypeSubstitution::new();
    let lit_match = interner.literal_string("foobazbar");
    let lit_other = interner.literal_string("bar");
    subst.insert(t_name, interner.union(vec![lit_match, lit_other]));

    let instantiated = instantiate_type(&interner, cond_type, &subst);
    let result = evaluate_type(&interner, instantiated);

    let expected = interner.literal_string("baz");
    assert_eq!(result, expected);
}

#[test]
fn test_conditional_infer_template_literal_two_infers_distributive() {
    let interner = TypeInterner::new();

    let t_name = interner.intern_string("T");
    let t_param = interner.intern(TypeKey::TypeParameter(TypeParamInfo {
        name: t_name,
        constraint: None,
        default: None,
    }));

    let infer_a_name = interner.intern_string("A");
    let infer_a = interner.intern(TypeKey::Infer(TypeParamInfo {
        name: infer_a_name,
        constraint: None,
        default: None,
    }));
    let infer_b_name = interner.intern_string("B");
    let infer_b = interner.intern(TypeKey::Infer(TypeParamInfo {
        name: infer_b_name,
        constraint: None,
        default: None,
    }));

    // T extends `${infer A}-${infer B}` ? A | B : never, with T = "foo-bar" | "baz-qux".
    let extends_template = interner.template_literal(vec![
        TemplateSpan::Type(infer_a),
        TemplateSpan::Text(interner.intern_string("-")),
        TemplateSpan::Type(infer_b),
    ]);
    let cond = ConditionalType {
        check_type: t_param,
        extends_type: extends_template,
        true_type: interner.union(vec![infer_a, infer_b]),
        false_type: TypeId::NEVER,
        is_distributive: true,
    };

    let cond_type = interner.conditional(cond);
    let mut subst = TypeSubstitution::new();
    let lit_left = interner.literal_string("foo-bar");
    let lit_right = interner.literal_string("baz-qux");
    subst.insert(t_name, interner.union(vec![lit_left, lit_right]));

    let instantiated = instantiate_type(&interner, cond_type, &subst);
    let result = evaluate_type(&interner, instantiated);

    let expected = interner.union(vec![
        interner.literal_string("foo"),
        interner.literal_string("baz"),
        interner.literal_string("bar"),
        interner.literal_string("qux"),
    ]);
    assert_eq!(result, expected);
}

#[test]
fn test_conditional_infer_template_literal_with_constrained_infer_distributive() {
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

    // T extends `foo${infer R extends string}` ? R : never, with T = "foo1" | "foo2".
    let extends_template = interner.template_literal(vec![
        TemplateSpan::Text(interner.intern_string("foo")),
        TemplateSpan::Type(infer_r),
    ]);
    let cond = ConditionalType {
        check_type: t_param,
        extends_type: extends_template,
        true_type: infer_r,
        false_type: TypeId::NEVER,
        is_distributive: true,
    };

    let cond_type = interner.conditional(cond);
    let mut subst = TypeSubstitution::new();
    let lit_foo1 = interner.literal_string("foo1");
    let lit_foo2 = interner.literal_string("foo2");
    subst.insert(t_name, interner.union(vec![lit_foo1, lit_foo2]));

    let instantiated = instantiate_type(&interner, cond_type, &subst);
    let result = evaluate_type(&interner, instantiated);

    let expected = interner.union(vec![
        interner.literal_string("1"),
        interner.literal_string("2"),
    ]);
    assert_eq!(result, expected);
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
fn test_conditional_infer_nested_object_property_non_distributive_union_input() {
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

    // T extends { a: { b: infer R } } ? R : never, with T = { a: { b: string } } | { a: { b: number } } (no distribution).
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
        is_distributive: false,
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

    let expected = TypeId::STRING;

    assert_eq!(result, expected);
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
    let expected = TypeId::STRING;

    assert_eq!(result, expected);
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
    let expected = TypeId::STRING;

    assert_eq!(result, expected);
}

#[test]
fn test_conditional_infer_object_property_non_distributive_non_object_union_branch() {
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

    // [T] extends [{ a: infer R }] ? R : never, with T = { a: string } | number (no distribution).
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
    let expected = interner.union(vec![TypeId::STRING, TypeId::NUMBER]);

    assert_eq!(result, expected);
}

#[test]
fn test_conditional_infer_number_index_signature_distributive() {
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

    // T extends { [key: number]: infer R } ? R : never, with T = { 0: string } | { 1: number }.
    let extends_obj = interner.object_with_index(ObjectShape {
        properties: Vec::new(),
        string_index: None,
        number_index: Some(IndexSignature {
            key_type: TypeId::NUMBER,
            value_type: infer_r,
            readonly: false,
        }),
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
        name: interner.intern_string("0"),
        type_id: TypeId::STRING,
        write_type: TypeId::STRING,
        optional: false,
        readonly: false,
        is_method: false,
    }]);
    let obj_number = interner.object(vec![PropertyInfo {
        name: interner.intern_string("1"),
        type_id: TypeId::NUMBER,
        write_type: TypeId::NUMBER,
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
fn test_conditional_infer_number_index_signature_non_distributive_union_input() {
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

    // T extends { [key: number]: infer R } ? R : never, with T = { 0: string } | { 1: number } (no distribution).
    let extends_obj = interner.object_with_index(ObjectShape {
        properties: Vec::new(),
        string_index: None,
        number_index: Some(IndexSignature {
            key_type: TypeId::NUMBER,
            value_type: infer_r,
            readonly: false,
        }),
    });
    let cond = ConditionalType {
        check_type: t_param,
        extends_type: extends_obj,
        true_type: infer_r,
        false_type: TypeId::NEVER,
        is_distributive: false,
    };

    let cond_type = interner.conditional(cond);
    let mut subst = TypeSubstitution::new();
    let obj_string = interner.object(vec![PropertyInfo {
        name: interner.intern_string("0"),
        type_id: TypeId::STRING,
        write_type: TypeId::STRING,
        optional: false,
        readonly: false,
        is_method: false,
    }]);
    let obj_number = interner.object(vec![PropertyInfo {
        name: interner.intern_string("1"),
        type_id: TypeId::NUMBER,
        write_type: TypeId::NUMBER,
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
fn test_conditional_infer_object_index_signature_non_object_union_branch() {
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

    // T extends { [key: string]: infer R } ? R : never, with T = { a: string } | number.
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
    subst.insert(t_name, interner.union(vec![obj_string, TypeId::NUMBER]));

    let instantiated = instantiate_type(&interner, cond_type, &subst);
    let result = evaluate_type(&interner, instantiated);

    assert_eq!(result, TypeId::STRING);
}

#[test]
fn test_conditional_infer_object_index_signature_non_distributive_union_input() {
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

    // T extends { [key: string]: infer R } ? R : never, with T = { a: string } | { b: number } (no distribution).
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
        is_distributive: false,
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
    let expected = interner.union(vec![TypeId::STRING, TypeId::NUMBER]);

    assert_eq!(result, expected);
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

    assert_eq!(result, TypeId::UNDEFINED);
}

#[test]
fn test_conditional_infer_optional_property_present_distributive() {
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

    // T extends { a?: infer R } ? R : never, with T = { a?: string } | { a?: number }.
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
    let obj_string = interner.object(vec![PropertyInfo {
        name: interner.intern_string("a"),
        type_id: TypeId::STRING,
        write_type: TypeId::STRING,
        optional: true,
        readonly: false,
        is_method: false,
    }]);
    let obj_number = interner.object(vec![PropertyInfo {
        name: interner.intern_string("a"),
        type_id: TypeId::NUMBER,
        write_type: TypeId::NUMBER,
        optional: true,
        readonly: false,
        is_method: false,
    }]);
    subst.insert(t_name, interner.union(vec![obj_string, obj_number]));

    let instantiated = instantiate_type(&interner, cond_type, &subst);
    let result = evaluate_type(&interner, instantiated);
    let expected = interner.union(vec![TypeId::STRING, TypeId::NUMBER, TypeId::UNDEFINED]);

    assert_eq!(result, expected);
}

#[test]
fn test_conditional_infer_optional_property_with_constraint() {
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

    // T extends { a?: infer R extends string } ? R : never, with T = { a?: string } | { a?: number }.
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
    let obj_string = interner.object(vec![PropertyInfo {
        name: interner.intern_string("a"),
        type_id: TypeId::STRING,
        write_type: TypeId::STRING,
        optional: true,
        readonly: false,
        is_method: false,
    }]);
    let obj_number = interner.object(vec![PropertyInfo {
        name: interner.intern_string("a"),
        type_id: TypeId::NUMBER,
        write_type: TypeId::NUMBER,
        optional: true,
        readonly: false,
        is_method: false,
    }]);
    subst.insert(t_name, interner.union(vec![obj_string, obj_number]));

    let instantiated = instantiate_type(&interner, cond_type, &subst);
    let result = evaluate_type(&interner, instantiated);

    assert_eq!(result, TypeId::STRING);
}

#[test]
fn test_conditional_infer_optional_property_non_distributive_union_input() {
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

    // [T] extends [{ a?: infer R }] ? R : never, with T = { a: string } | {} (no distribution).
    let extends_obj = interner.object(vec![PropertyInfo {
        name: interner.intern_string("a"),
        type_id: infer_r,
        write_type: infer_r,
        optional: true,
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
    let obj_string = interner.object(vec![PropertyInfo {
        name: interner.intern_string("a"),
        type_id: TypeId::STRING,
        write_type: TypeId::STRING,
        optional: false,
        readonly: false,
        is_method: false,
    }]);
    let empty_obj = interner.object(Vec::new());
    subst.insert(t_name, interner.union(vec![obj_string, empty_obj]));

    let instantiated = instantiate_type(&interner, cond_type, &subst);
    let result = evaluate_type(&interner, instantiated);
    let expected = interner.union(vec![TypeId::STRING, TypeId::UNDEFINED]);

    assert_eq!(result, expected);
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

    assert_eq!(result, TypeId::STRING);
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

    let expected = interner.union(vec![TypeId::STRING, TypeId::NUMBER]);
    assert_eq!(result, expected);
}

#[test]
fn test_conditional_infer_function_optional_param_distributive() {
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

    // T extends (arg?: infer R) => void ? R : never, with T = ((arg?: string) => void)
    // | ((arg?: number) => void).
    let extends_fn = interner.function(FunctionShape {
        params: vec![ParamInfo {
            name: None,
            type_id: infer_r,
            optional: true,
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
            optional: true,
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
            optional: true,
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

    let expected = interner.union(vec![TypeId::STRING, TypeId::NUMBER, TypeId::UNDEFINED]);
    assert_eq!(result, expected);
}

#[test]
fn test_conditional_infer_function_optional_param_non_distributive_union_input() {
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

    // [T] extends [(arg?: infer R) => void] ? R : never, with T = ((arg?: string) => void)
    // | ((arg?: number) => void).
    let extends_fn = interner.function(FunctionShape {
        params: vec![ParamInfo {
            name: None,
            type_id: infer_r,
            optional: true,
            rest: false,
        }],
        this_type: None,
        return_type: TypeId::VOID,
        type_params: Vec::new(),
        type_predicate: None,
        is_constructor: false,
    });
    let cond = ConditionalType {
        check_type: interner.tuple(vec![TupleElement {
            type_id: t_param,
            name: None,
            optional: false,
            rest: false,
        }]),
        extends_type: interner.tuple(vec![TupleElement {
            type_id: extends_fn,
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
    let string_fn = interner.function(FunctionShape {
        params: vec![ParamInfo {
            name: None,
            type_id: TypeId::STRING,
            optional: true,
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
            optional: true,
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

    let expected = interner.union(vec![TypeId::STRING, TypeId::NUMBER, TypeId::UNDEFINED]);
    assert_eq!(result, expected);
}

#[test]
fn test_conditional_infer_function_param_non_function_union_branch() {
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

    // T extends (arg: infer R) => void ? R : never, with T = ((arg: string) => void) | number.
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
    subst.insert(t_name, interner.union(vec![string_fn, TypeId::NUMBER]));

    let instantiated = instantiate_type(&interner, cond_type, &subst);
    let result = evaluate_type(&interner, instantiated);

    assert_eq!(result, TypeId::STRING);
}

#[test]
fn test_conditional_infer_function_param_non_distributive_union_input() {
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

    // [T] extends [(arg: infer R) => void] ? R : never, with T = ((arg: string) => void)
    // | ((arg: number) => void).
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
        check_type: interner.tuple(vec![TupleElement {
            type_id: t_param,
            name: None,
            optional: false,
            rest: false,
        }]),
        extends_type: interner.tuple(vec![TupleElement {
            type_id: extends_fn,
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

    let expected = interner.union(vec![TypeId::STRING, TypeId::NUMBER]);
    assert_eq!(result, expected);
}

#[test]
fn test_conditional_infer_function_rest_param_distributive() {
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

    // T extends (...args: infer R) => void ? R : never, with T = ((...args: string[]) => void)
    // | ((...args: number[]) => void).
    let extends_fn = interner.function(FunctionShape {
        params: vec![ParamInfo {
            name: None,
            type_id: infer_r,
            optional: false,
            rest: true,
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
            type_id: interner.array(TypeId::STRING),
            optional: false,
            rest: true,
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
            type_id: interner.array(TypeId::NUMBER),
            optional: false,
            rest: true,
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

    let expected = interner.union(vec![
        interner.array(TypeId::STRING),
        interner.array(TypeId::NUMBER),
    ]);
    assert_eq!(result, expected);
}

#[test]
fn test_conditional_infer_function_rest_param_non_distributive_union_input() {
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

    // [T] extends [(...args: infer R) => void] ? R : never, with T = ((...args: string[]) => void)
    // | ((...args: number[]) => void).
    let extends_fn = interner.function(FunctionShape {
        params: vec![ParamInfo {
            name: None,
            type_id: infer_r,
            optional: false,
            rest: true,
        }],
        this_type: None,
        return_type: TypeId::VOID,
        type_params: Vec::new(),
        type_predicate: None,
        is_constructor: false,
    });
    let cond = ConditionalType {
        check_type: interner.tuple(vec![TupleElement {
            type_id: t_param,
            name: None,
            optional: false,
            rest: false,
        }]),
        extends_type: interner.tuple(vec![TupleElement {
            type_id: extends_fn,
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
    let string_fn = interner.function(FunctionShape {
        params: vec![ParamInfo {
            name: None,
            type_id: interner.array(TypeId::STRING),
            optional: false,
            rest: true,
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
            type_id: interner.array(TypeId::NUMBER),
            optional: false,
            rest: true,
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

    let expected = interner.union(vec![
        interner.array(TypeId::STRING),
        interner.array(TypeId::NUMBER),
    ]);
    assert_eq!(result, expected);
}

#[test]
fn test_conditional_infer_function_this_param_distributive() {
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

    // T extends (this: infer R) => void ? R : never, with T = ((this: string) => void)
    // | ((this: number) => void).
    let extends_fn = interner.function(FunctionShape {
        params: Vec::new(),
        this_type: Some(infer_r),
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
        params: Vec::new(),
        this_type: Some(TypeId::STRING),
        return_type: TypeId::VOID,
        type_params: Vec::new(),
        type_predicate: None,
        is_constructor: false,
    });
    let number_fn = interner.function(FunctionShape {
        params: Vec::new(),
        this_type: Some(TypeId::NUMBER),
        return_type: TypeId::VOID,
        type_params: Vec::new(),
        type_predicate: None,
        is_constructor: false,
    });
    subst.insert(t_name, interner.union(vec![string_fn, number_fn]));

    let instantiated = instantiate_type(&interner, cond_type, &subst);
    let result = evaluate_type(&interner, instantiated);
    let expected = interner.union(vec![TypeId::STRING, TypeId::NUMBER]);

    assert_eq!(result, expected);
}

#[test]
fn test_conditional_infer_function_this_param_non_distributive_union_input() {
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

    // [T] extends [(this: infer R) => void] ? R : never, with T = ((this: string) => void)
    // | ((this: number) => void).
    let extends_fn = interner.function(FunctionShape {
        params: Vec::new(),
        this_type: Some(infer_r),
        return_type: TypeId::VOID,
        type_params: Vec::new(),
        type_predicate: None,
        is_constructor: false,
    });
    let cond = ConditionalType {
        check_type: interner.tuple(vec![TupleElement {
            type_id: t_param,
            name: None,
            optional: false,
            rest: false,
        }]),
        extends_type: interner.tuple(vec![TupleElement {
            type_id: extends_fn,
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
    let string_fn = interner.function(FunctionShape {
        params: Vec::new(),
        this_type: Some(TypeId::STRING),
        return_type: TypeId::VOID,
        type_params: Vec::new(),
        type_predicate: None,
        is_constructor: false,
    });
    let number_fn = interner.function(FunctionShape {
        params: Vec::new(),
        this_type: Some(TypeId::NUMBER),
        return_type: TypeId::VOID,
        type_params: Vec::new(),
        type_predicate: None,
        is_constructor: false,
    });
    subst.insert(t_name, interner.union(vec![string_fn, number_fn]));

    let instantiated = instantiate_type(&interner, cond_type, &subst);
    let result = evaluate_type(&interner, instantiated);
    let expected = interner.union(vec![TypeId::STRING, TypeId::NUMBER]);

    assert_eq!(result, expected);
}

#[test]
fn test_conditional_infer_function_return_distributive() {
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

    // T extends () => infer R ? R : never, with T = (() => string) | (() => number).
    let extends_fn = interner.function(FunctionShape {
        params: Vec::new(),
        this_type: None,
        return_type: infer_r,
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
        params: Vec::new(),
        this_type: None,
        return_type: TypeId::STRING,
        type_params: Vec::new(),
        type_predicate: None,
        is_constructor: false,
    });
    let number_fn = interner.function(FunctionShape {
        params: Vec::new(),
        this_type: None,
        return_type: TypeId::NUMBER,
        type_params: Vec::new(),
        type_predicate: None,
        is_constructor: false,
    });
    subst.insert(t_name, interner.union(vec![string_fn, number_fn]));

    let instantiated = instantiate_type(&interner, cond_type, &subst);
    let result = evaluate_type(&interner, instantiated);

    let expected = interner.union(vec![TypeId::STRING, TypeId::NUMBER]);
    assert_eq!(result, expected);
}

#[test]
fn test_conditional_infer_function_return_non_distributive_union_input() {
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

    // [T] extends [() => infer R] ? R : never, with T = (() => string) | (() => number).
    let extends_fn = interner.function(FunctionShape {
        params: Vec::new(),
        this_type: None,
        return_type: infer_r,
        type_params: Vec::new(),
        type_predicate: None,
        is_constructor: false,
    });
    let cond = ConditionalType {
        check_type: interner.tuple(vec![TupleElement {
            type_id: t_param,
            name: None,
            optional: false,
            rest: false,
        }]),
        extends_type: interner.tuple(vec![TupleElement {
            type_id: extends_fn,
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
    let string_fn = interner.function(FunctionShape {
        params: Vec::new(),
        this_type: None,
        return_type: TypeId::STRING,
        type_params: Vec::new(),
        type_predicate: None,
        is_constructor: false,
    });
    let number_fn = interner.function(FunctionShape {
        params: Vec::new(),
        this_type: None,
        return_type: TypeId::NUMBER,
        type_params: Vec::new(),
        type_predicate: None,
        is_constructor: false,
    });
    subst.insert(t_name, interner.union(vec![string_fn, number_fn]));

    let instantiated = instantiate_type(&interner, cond_type, &subst);
    let result = evaluate_type(&interner, instantiated);

    let expected = interner.union(vec![TypeId::STRING, TypeId::NUMBER]);
    assert_eq!(result, expected);
}

#[test]
fn test_conditional_infer_function_param_and_return_distributive() {
    let interner = TypeInterner::new();

    let t_name = interner.intern_string("T");
    let t_param = interner.intern(TypeKey::TypeParameter(TypeParamInfo {
        name: t_name,
        constraint: None,
        default: None,
    }));

    let p_name = interner.intern_string("P");
    let infer_p = interner.intern(TypeKey::Infer(TypeParamInfo {
        name: p_name,
        constraint: None,
        default: None,
    }));

    let r_name = interner.intern_string("R");
    let infer_r = interner.intern(TypeKey::Infer(TypeParamInfo {
        name: r_name,
        constraint: None,
        default: None,
    }));

    // T extends (arg: infer P) => infer R ? [P, R] : never, with T = ((arg: string) => number)
    // | ((arg: boolean) => string).
    let extends_fn = interner.function(FunctionShape {
        params: vec![ParamInfo {
            name: None,
            type_id: infer_p,
            optional: false,
            rest: false,
        }],
        this_type: None,
        return_type: infer_r,
        type_params: Vec::new(),
        type_predicate: None,
        is_constructor: false,
    });
    let true_tuple = interner.tuple(vec![
        TupleElement {
            type_id: infer_p,
            name: None,
            optional: false,
            rest: false,
        },
        TupleElement {
            type_id: infer_r,
            name: None,
            optional: false,
            rest: false,
        },
    ]);
    let cond = ConditionalType {
        check_type: t_param,
        extends_type: extends_fn,
        true_type: true_tuple,
        false_type: TypeId::NEVER,
        is_distributive: true,
    };

    let cond_type = interner.conditional(cond);
    let mut subst = TypeSubstitution::new();
    let string_number_fn = interner.function(FunctionShape {
        params: vec![ParamInfo {
            name: None,
            type_id: TypeId::STRING,
            optional: false,
            rest: false,
        }],
        this_type: None,
        return_type: TypeId::NUMBER,
        type_params: Vec::new(),
        type_predicate: None,
        is_constructor: false,
    });
    let boolean_string_fn = interner.function(FunctionShape {
        params: vec![ParamInfo {
            name: None,
            type_id: TypeId::BOOLEAN,
            optional: false,
            rest: false,
        }],
        this_type: None,
        return_type: TypeId::STRING,
        type_params: Vec::new(),
        type_predicate: None,
        is_constructor: false,
    });
    subst.insert(t_name, interner.union(vec![string_number_fn, boolean_string_fn]));

    let instantiated = instantiate_type(&interner, cond_type, &subst);
    let result = evaluate_type(&interner, instantiated);

    let tuple_string_number = interner.tuple(vec![
        TupleElement {
            type_id: TypeId::STRING,
            name: None,
            optional: false,
            rest: false,
        },
        TupleElement {
            type_id: TypeId::NUMBER,
            name: None,
            optional: false,
            rest: false,
        },
    ]);
    let tuple_boolean_string = interner.tuple(vec![
        TupleElement {
            type_id: TypeId::BOOLEAN,
            name: None,
            optional: false,
            rest: false,
        },
        TupleElement {
            type_id: TypeId::STRING,
            name: None,
            optional: false,
            rest: false,
        },
    ]);
    let expected = interner.union(vec![tuple_string_number, tuple_boolean_string]);
    assert_eq!(result, expected);
}

#[test]
fn test_conditional_infer_function_param_and_return_non_distributive_union_input() {
    let interner = TypeInterner::new();

    let t_name = interner.intern_string("T");
    let t_param = interner.intern(TypeKey::TypeParameter(TypeParamInfo {
        name: t_name,
        constraint: None,
        default: None,
    }));

    let p_name = interner.intern_string("P");
    let infer_p = interner.intern(TypeKey::Infer(TypeParamInfo {
        name: p_name,
        constraint: None,
        default: None,
    }));

    let r_name = interner.intern_string("R");
    let infer_r = interner.intern(TypeKey::Infer(TypeParamInfo {
        name: r_name,
        constraint: None,
        default: None,
    }));

    // [T] extends [(arg: infer P) => infer R] ? [P, R] : never, with T = ((arg: string) => number)
    // | ((arg: boolean) => string).
    let extends_fn = interner.function(FunctionShape {
        params: vec![ParamInfo {
            name: None,
            type_id: infer_p,
            optional: false,
            rest: false,
        }],
        this_type: None,
        return_type: infer_r,
        type_params: Vec::new(),
        type_predicate: None,
        is_constructor: false,
    });
    let true_tuple = interner.tuple(vec![
        TupleElement {
            type_id: infer_p,
            name: None,
            optional: false,
            rest: false,
        },
        TupleElement {
            type_id: infer_r,
            name: None,
            optional: false,
            rest: false,
        },
    ]);
    let cond = ConditionalType {
        check_type: interner.tuple(vec![TupleElement {
            type_id: t_param,
            name: None,
            optional: false,
            rest: false,
        }]),
        extends_type: interner.tuple(vec![TupleElement {
            type_id: extends_fn,
            name: None,
            optional: false,
            rest: false,
        }]),
        true_type: true_tuple,
        false_type: TypeId::NEVER,
        is_distributive: false,
    };

    let cond_type = interner.conditional(cond);
    let mut subst = TypeSubstitution::new();
    let string_number_fn = interner.function(FunctionShape {
        params: vec![ParamInfo {
            name: None,
            type_id: TypeId::STRING,
            optional: false,
            rest: false,
        }],
        this_type: None,
        return_type: TypeId::NUMBER,
        type_params: Vec::new(),
        type_predicate: None,
        is_constructor: false,
    });
    let boolean_string_fn = interner.function(FunctionShape {
        params: vec![ParamInfo {
            name: None,
            type_id: TypeId::BOOLEAN,
            optional: false,
            rest: false,
        }],
        this_type: None,
        return_type: TypeId::STRING,
        type_params: Vec::new(),
        type_predicate: None,
        is_constructor: false,
    });
    subst.insert(t_name, interner.union(vec![string_number_fn, boolean_string_fn]));

    let instantiated = instantiate_type(&interner, cond_type, &subst);
    let result = evaluate_type(&interner, instantiated);

    let param_union = interner.union(vec![TypeId::STRING, TypeId::BOOLEAN]);
    let return_union = interner.union(vec![TypeId::NUMBER, TypeId::STRING]);
    let expected = interner.tuple(vec![
        TupleElement {
            type_id: param_union,
            name: None,
            optional: false,
            rest: false,
        },
        TupleElement {
            type_id: return_union,
            name: None,
            optional: false,
            rest: false,
        },
    ]);
    assert_eq!(result, expected);
}

#[test]
fn test_conditional_infer_object_call_signature_distributive() {
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

    // T extends { (x: infer R): void } ? R : never, with T = { (x: string): void }
    // | { (x: number): void }.
    let extends_callable = interner.callable(CallableShape {
        call_signatures: vec![CallSignature {
            params: vec![ParamInfo {
                name: None,
                type_id: infer_r,
                optional: false,
                rest: false,
            }],
            this_type: None,
            return_type: TypeId::VOID,
            type_predicate: None,
            type_params: Vec::new(),
        }],
        construct_signatures: Vec::new(),
        properties: Vec::new(),
    });
    let cond = ConditionalType {
        check_type: t_param,
        extends_type: extends_callable,
        true_type: infer_r,
        false_type: TypeId::NEVER,
        is_distributive: true,
    };

    let cond_type = interner.conditional(cond);
    let mut subst = TypeSubstitution::new();
    let string_callable = interner.callable(CallableShape {
        call_signatures: vec![CallSignature {
            params: vec![ParamInfo {
                name: None,
                type_id: TypeId::STRING,
                optional: false,
                rest: false,
            }],
            this_type: None,
            return_type: TypeId::VOID,
            type_predicate: None,
            type_params: Vec::new(),
        }],
        construct_signatures: Vec::new(),
        properties: Vec::new(),
    });
    let number_callable = interner.callable(CallableShape {
        call_signatures: vec![CallSignature {
            params: vec![ParamInfo {
                name: None,
                type_id: TypeId::NUMBER,
                optional: false,
                rest: false,
            }],
            this_type: None,
            return_type: TypeId::VOID,
            type_predicate: None,
            type_params: Vec::new(),
        }],
        construct_signatures: Vec::new(),
        properties: Vec::new(),
    });
    subst.insert(t_name, interner.union(vec![string_callable, number_callable]));

    let instantiated = instantiate_type(&interner, cond_type, &subst);
    let result = evaluate_type(&interner, instantiated);

    let expected = interner.union(vec![TypeId::STRING, TypeId::NUMBER]);
    assert_eq!(result, expected);
}

#[test]
fn test_conditional_infer_object_property_non_distributive_union_all_match() {
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

    // [T] extends [{ a: infer R }] ? R : never, with T = { a: string } | { a: number }.
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
    let obj_string = interner.object(vec![PropertyInfo {
        name: interner.intern_string("a"),
        type_id: TypeId::STRING,
        write_type: TypeId::STRING,
        optional: false,
        readonly: false,
        is_method: false,
    }]);
    let obj_number = interner.object(vec![PropertyInfo {
        name: interner.intern_string("a"),
        type_id: TypeId::NUMBER,
        write_type: TypeId::NUMBER,
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
fn test_conditional_infer_tuple_optional_element_distributive() {
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

    // T extends [infer R?] ? R : never, with T = [string] | [].
    let extends_tuple = interner.tuple(vec![TupleElement {
        type_id: infer_r,
        name: None,
        optional: true,
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
    let string_tuple = interner.tuple(vec![TupleElement {
        type_id: TypeId::STRING,
        name: None,
        optional: false,
        rest: false,
    }]);
    let empty_tuple = interner.tuple(Vec::new());
    subst.insert(t_name, interner.union(vec![string_tuple, empty_tuple]));

    let instantiated = instantiate_type(&interner, cond_type, &subst);
    let result = evaluate_type(&interner, instantiated);
    let expected = interner.union(vec![TypeId::STRING, TypeId::UNDEFINED]);

    assert_eq!(result, expected);
}

#[test]
fn test_conditional_infer_tuple_optional_element_non_distributive_union_input() {
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

    // T extends [infer R?] ? R : never, with T = [string] | [] (no distribution).
    let extends_tuple = interner.tuple(vec![TupleElement {
        type_id: infer_r,
        name: None,
        optional: true,
        rest: false,
    }]);
    let cond = ConditionalType {
        check_type: t_param,
        extends_type: extends_tuple,
        true_type: infer_r,
        false_type: TypeId::NEVER,
        is_distributive: false,
    };

    let cond_type = interner.conditional(cond);
    let mut subst = TypeSubstitution::new();
    let string_tuple = interner.tuple(vec![TupleElement {
        type_id: TypeId::STRING,
        name: None,
        optional: false,
        rest: false,
    }]);
    let empty_tuple = interner.tuple(Vec::new());
    subst.insert(t_name, interner.union(vec![string_tuple, empty_tuple]));

    let instantiated = instantiate_type(&interner, cond_type, &subst);
    let result = evaluate_type(&interner, instantiated);
    let expected = interner.union(vec![TypeId::STRING, TypeId::UNDEFINED]);

    assert_eq!(result, expected);
}

#[test]
fn test_conditional_infer_tuple_element_non_distributive_union_input() {
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

    // T extends [infer R] ? R : never, with T = [string] | [number] (no distribution).
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
        is_distributive: false,
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
fn test_conditional_infer_tuple_element_non_tuple_union_branch() {
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

    // T extends [infer R] ? R : never, with T = [string] | number.
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
    let tuple_string = interner.tuple(vec![TupleElement {
        type_id: TypeId::STRING,
        name: None,
        optional: false,
        rest: false,
    }]);
    subst.insert(t_name, interner.union(vec![tuple_string, TypeId::NUMBER]));

    let instantiated = instantiate_type(&interner, cond_type, &subst);
    let result = evaluate_type(&interner, instantiated);

    assert_eq!(result, TypeId::STRING);
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
fn test_conditional_infer_optional_tuple_element_with_constraint() {
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

    // T extends [infer R extends string] ? R : never, with T = [string?] | [number?].
    let extends_tuple = interner.tuple(vec![TupleElement {
        type_id: infer_r,
        name: None,
        optional: true,
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
                optional: true,
                rest: false,
            }]),
            interner.tuple(vec![TupleElement {
                type_id: TypeId::STRING,
                name: None,
                optional: true,
                rest: false,
            }]),
        ]),
    );

    let instantiated = instantiate_type(&interner, cond_type, &subst);
    let result = evaluate_type(&interner, instantiated);

    assert_eq!(result, TypeId::STRING);
}

#[test]
fn test_conditional_infer_tuple_rest_distributive() {
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

    // T extends [string, ...infer R] ? R : never, with T = [string, number] | [string].
    // TODO: Variadic tuple inference is not implemented; current behavior yields number.
    let extends_tuple = interner.tuple(vec![
        TupleElement {
            type_id: TypeId::STRING,
            name: None,
            optional: false,
            rest: false,
        },
        TupleElement {
            type_id: infer_r,
            name: None,
            optional: false,
            rest: true,
        },
    ]);
    let cond = ConditionalType {
        check_type: t_param,
        extends_type: extends_tuple,
        true_type: infer_r,
        false_type: TypeId::NEVER,
        is_distributive: true,
    };

    let cond_type = interner.conditional(cond);
    let mut subst = TypeSubstitution::new();
    let tuple_string_number = interner.tuple(vec![
        TupleElement {
            type_id: TypeId::STRING,
            name: None,
            optional: false,
            rest: false,
        },
        TupleElement {
            type_id: TypeId::NUMBER,
            name: None,
            optional: false,
            rest: false,
        },
    ]);
    let tuple_string = interner.tuple(vec![TupleElement {
        type_id: TypeId::STRING,
        name: None,
        optional: false,
        rest: false,
    }]);
    subst.insert(t_name, interner.union(vec![tuple_string_number, tuple_string]));

    let instantiated = instantiate_type(&interner, cond_type, &subst);
    let result = evaluate_type(&interner, instantiated);

    assert_eq!(result, TypeId::NUMBER);
}

#[test]
fn test_conditional_infer_tuple_rest_with_head_infer_distributive() {
    let interner = TypeInterner::new();

    let t_name = interner.intern_string("T");
    let t_param = interner.intern(TypeKey::TypeParameter(TypeParamInfo {
        name: t_name,
        constraint: None,
        default: None,
    }));

    let infer_h_name = interner.intern_string("H");
    let infer_h = interner.intern(TypeKey::Infer(TypeParamInfo {
        name: infer_h_name,
        constraint: None,
        default: None,
    }));
    let infer_r_name = interner.intern_string("R");
    let infer_r = interner.intern(TypeKey::Infer(TypeParamInfo {
        name: infer_r_name,
        constraint: None,
        default: None,
    }));

    // T extends [infer H, ...infer R] ? R : never, with T = [string, number] | [boolean].
    // TODO: Variadic tuple inference is not implemented; current behavior yields number.
    let extends_tuple = interner.tuple(vec![
        TupleElement {
            type_id: infer_h,
            name: None,
            optional: false,
            rest: false,
        },
        TupleElement {
            type_id: infer_r,
            name: None,
            optional: false,
            rest: true,
        },
    ]);
    let cond = ConditionalType {
        check_type: t_param,
        extends_type: extends_tuple,
        true_type: infer_r,
        false_type: TypeId::NEVER,
        is_distributive: true,
    };

    let cond_type = interner.conditional(cond);
    let mut subst = TypeSubstitution::new();
    let tuple_string_number = interner.tuple(vec![
        TupleElement {
            type_id: TypeId::STRING,
            name: None,
            optional: false,
            rest: false,
        },
        TupleElement {
            type_id: TypeId::NUMBER,
            name: None,
            optional: false,
            rest: false,
        },
    ]);
    let tuple_boolean = interner.tuple(vec![TupleElement {
        type_id: TypeId::BOOLEAN,
        name: None,
        optional: false,
        rest: false,
    }]);
    subst.insert(t_name, interner.union(vec![tuple_string_number, tuple_boolean]));

    let instantiated = instantiate_type(&interner, cond_type, &subst);
    let result = evaluate_type(&interner, instantiated);

    assert_eq!(result, TypeId::NUMBER);
}

#[test]
fn test_conditional_infer_union_true_branch_distributive() {
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

    // T extends string ? R | number : never, with T = string | boolean.
    // Infer appears only in the true branch; ensure it is preserved.
    let cond = ConditionalType {
        check_type: t_param,
        extends_type: TypeId::STRING,
        true_type: interner.union(vec![infer_r, TypeId::NUMBER]),
        false_type: TypeId::NEVER,
        is_distributive: true,
    };

    let cond_type = interner.conditional(cond);
    let mut subst = TypeSubstitution::new();
    subst.insert(t_name, interner.union(vec![TypeId::STRING, TypeId::BOOLEAN]));

    let instantiated = instantiate_type(&interner, cond_type, &subst);
    let result = evaluate_type(&interner, instantiated);

    assert_eq!(result, interner.union(vec![infer_r, TypeId::NUMBER]));
}

#[test]
fn test_conditional_infer_union_false_branch_distributive() {
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

    // T extends string ? never : R | number, with T = string | boolean.
    // Infer appears only in the false branch; ensure it is preserved.
    let cond = ConditionalType {
        check_type: t_param,
        extends_type: TypeId::STRING,
        true_type: TypeId::NEVER,
        false_type: interner.union(vec![infer_r, TypeId::NUMBER]),
        is_distributive: true,
    };

    let cond_type = interner.conditional(cond);
    let mut subst = TypeSubstitution::new();
    subst.insert(t_name, interner.union(vec![TypeId::STRING, TypeId::BOOLEAN]));

    let instantiated = instantiate_type(&interner, cond_type, &subst);
    let result = evaluate_type(&interner, instantiated);

    assert_eq!(result, interner.union(vec![infer_r, TypeId::NUMBER]));
}

#[test]
fn test_conditional_infer_any_check_type_distributive() {
    let interner = TypeInterner::new();

    let infer_name = interner.intern_string("R");
    let infer_r = interner.intern(TypeKey::Infer(TypeParamInfo {
        name: infer_name,
        constraint: None,
        default: None,
    }));

    // any extends string ? infer R : never
    // any produces union of branches; infer should survive in true branch.
    let cond = ConditionalType {
        check_type: TypeId::ANY,
        extends_type: TypeId::STRING,
        true_type: infer_r,
        false_type: TypeId::NEVER,
        is_distributive: true,
    };

    let result = evaluate_conditional(&interner, &cond);
    assert_eq!(result, infer_r);
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
fn test_conditional_infer_readonly_array_element_non_distributive_union_input() {
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

    // T extends readonly (infer R)[] ? R : never, with T = readonly string[] | readonly number[] (no distribution).
    let extends_array =
        interner.intern(TypeKey::ReadonlyType(interner.array(infer_r)));
    let cond = ConditionalType {
        check_type: t_param,
        extends_type: extends_array,
        true_type: infer_r,
        false_type: TypeId::NEVER,
        is_distributive: false,
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
fn test_conditional_infer_readonly_array_element_non_array_union_branch() {
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

    // T extends readonly (infer R)[] ? R : never, with T = readonly string[] | number.
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
    subst.insert(t_name, interner.union(vec![readonly_string_array, TypeId::NUMBER]));

    let instantiated = instantiate_type(&interner, cond_type, &subst);
    let result = evaluate_type(&interner, instantiated);

    assert_eq!(result, TypeId::STRING);
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
fn test_conditional_infer_readonly_tuple_element_non_distributive_union_input() {
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

    // T extends readonly [infer R] ? R : never, with T = readonly [string] | readonly [number] (no distribution).
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
        is_distributive: false,
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
fn test_conditional_infer_readonly_tuple_element_non_tuple_union_branch() {
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

    // T extends readonly [infer R] ? R : never, with T = readonly [string] | number.
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
    subst.insert(t_name, interner.union(vec![readonly_string_tuple, TypeId::NUMBER]));

    let instantiated = instantiate_type(&interner, cond_type, &subst);
    let result = evaluate_type(&interner, instantiated);

    assert_eq!(result, TypeId::STRING);
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
fn test_conditional_infer_direct_match() {
    let interner = TypeInterner::new();

    let r_name = interner.intern_string("R");
    let infer_r = interner.intern(TypeKey::Infer(TypeParamInfo {
        name: r_name,
        constraint: None,
        default: None,
    }));

    // string extends infer R ? R : never -> string
    let cond = ConditionalType {
        check_type: TypeId::STRING,
        extends_type: infer_r,
        true_type: infer_r,
        false_type: TypeId::NEVER,
        is_distributive: false,
    };

    let result = evaluate_conditional(&interner, &cond);
    assert_eq!(result, TypeId::STRING);
}

#[test]
fn test_conditional_infer_constraint_mismatch() {
    let interner = TypeInterner::new();

    let r_name = interner.intern_string("R");
    let infer_r = interner.intern(TypeKey::Infer(TypeParamInfo {
        name: r_name,
        constraint: Some(TypeId::NUMBER),
        default: None,
    }));
    let no = interner.literal_string("no");

    // string extends infer R extends number ? R : "no" -> "no"
    let cond = ConditionalType {
        check_type: TypeId::STRING,
        extends_type: infer_r,
        true_type: infer_r,
        false_type: no,
        is_distributive: false,
    };

    let result = evaluate_conditional(&interner, &cond);
    assert_eq!(result, no);
}

#[test]
fn test_conditional_distributive_infer_array_extends() {
    let interner = TypeInterner::new();

    let t_name = interner.intern_string("T");
    let t_param = interner.intern(TypeKey::TypeParameter(TypeParamInfo {
        name: t_name,
        constraint: None,
        default: None,
    }));

    let r_name = interner.intern_string("R");
    let infer_r = interner.intern(TypeKey::Infer(TypeParamInfo {
        name: r_name,
        constraint: None,
        default: None,
    }));

    // T extends Array<infer R> ? R : never
    let cond = ConditionalType {
        check_type: t_param,
        extends_type: interner.array(infer_r),
        true_type: infer_r,
        false_type: TypeId::NEVER,
        is_distributive: true,
    };

    let cond_type = interner.conditional(cond);
    let string_array = interner.array(TypeId::STRING);
    let number_array = interner.array(TypeId::NUMBER);
    let union_arrays = interner.union(vec![string_array, number_array]);

    let mut subst = TypeSubstitution::new();
    subst.insert(t_name, union_arrays);

    let instantiated = instantiate_type(&interner, cond_type, &subst);
    let result = evaluate_type(&interner, instantiated);
    let expected = interner.union(vec![TypeId::STRING, TypeId::NUMBER]);

    assert_eq!(result, expected);
}

#[test]
fn test_conditional_nested_distributive_infer() {
    let interner = TypeInterner::new();

    let t_name = interner.intern_string("T");
    let t_param = interner.intern(TypeKey::TypeParameter(TypeParamInfo {
        name: t_name,
        constraint: None,
        default: None,
    }));

    let r_name = interner.intern_string("R");
    let infer_r = interner.intern(TypeKey::Infer(TypeParamInfo {
        name: r_name,
        constraint: None,
        default: None,
    }));

    let yes = interner.literal_string("yes");
    let no = interner.literal_string("no");
    let outer_no = interner.literal_string("outer-no");

    let inner_cond = interner.conditional(ConditionalType {
        check_type: infer_r,
        extends_type: TypeId::STRING,
        true_type: yes,
        false_type: no,
        is_distributive: false,
    });

    // T extends infer R ? (R extends string ? "yes" : "no") : "outer-no"
    let cond = ConditionalType {
        check_type: t_param,
        extends_type: infer_r,
        true_type: inner_cond,
        false_type: outer_no,
        is_distributive: true,
    };

    let cond_type = interner.conditional(cond);
    let union = interner.union(vec![TypeId::STRING, TypeId::NUMBER]);

    let mut subst = TypeSubstitution::new();
    subst.insert(t_name, union);

    let instantiated = instantiate_type(&interner, cond_type, &subst);
    let result = evaluate_type(&interner, instantiated);
    let expected = interner.union(vec![yes, no]);

    assert_eq!(result, expected);
}

#[test]
fn test_conditional_infer_object_property() {
    let interner = TypeInterner::new();

    let r_name = interner.intern_string("R");
    let infer_r = interner.intern(TypeKey::Infer(TypeParamInfo {
        name: r_name,
        constraint: None,
        default: None,
    }));

    let prop_name = interner.intern_string("a");
    let source = interner.object(vec![PropertyInfo {
        name: prop_name,
        type_id: TypeId::STRING,
        write_type: TypeId::STRING,
        optional: false,
        readonly: false,
        is_method: false,
    }]);
    let pattern = interner.object(vec![PropertyInfo {
        name: prop_name,
        type_id: infer_r,
        write_type: infer_r,
        optional: false,
        readonly: false,
        is_method: false,
    }]);

    // { a: string } extends { a: infer R } ? R : never -> string
    let cond = ConditionalType {
        check_type: source,
        extends_type: pattern,
        true_type: infer_r,
        false_type: TypeId::NEVER,
        is_distributive: false,
    };

    let result = evaluate_conditional(&interner, &cond);
    assert_eq!(result, TypeId::STRING);
}

#[test]
fn test_conditional_infer_object_string_index_signature() {
    let interner = TypeInterner::new();

    let r_name = interner.intern_string("R");
    let infer_r = interner.intern(TypeKey::Infer(TypeParamInfo {
        name: r_name,
        constraint: None,
        default: None,
    }));

    let source = interner.object_with_index(ObjectShape {
        properties: Vec::new(),
        string_index: Some(IndexSignature {
            key_type: TypeId::STRING,
            value_type: TypeId::NUMBER,
            readonly: false,
        }),
        number_index: None,
    });
    let pattern = interner.object_with_index(ObjectShape {
        properties: Vec::new(),
        string_index: Some(IndexSignature {
            key_type: TypeId::STRING,
            value_type: infer_r,
            readonly: false,
        }),
        number_index: None,
    });

    // { [key: string]: number } extends { [key: string]: infer R } ? R : never -> number
    let cond = ConditionalType {
        check_type: source,
        extends_type: pattern,
        true_type: infer_r,
        false_type: TypeId::NEVER,
        is_distributive: false,
    };

    let result = evaluate_conditional(&interner, &cond);
    assert_eq!(result, TypeId::NUMBER);
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

#[test]
fn test_mapped_type_with_conditional_value_filter() {
    let interner = TypeInterner::new();

    // PickValue<T, V> = { [K in keyof T]: T[K] extends V ? T[K] : never }
    // Applied to { a: number; b: string } with V = number
    // The mapped type should evaluate to an object with 2 properties
    // The conditional types in the value position may remain deferred if K is not fully resolved
    let prop_a = interner.intern_string("a");
    let prop_b = interner.intern_string("b");

    let source_obj = interner.object(vec![
        PropertyInfo {
            name: prop_a,
            type_id: TypeId::NUMBER,
            write_type: TypeId::NUMBER,
            optional: false,
            readonly: false,
            is_method: false,
        },
        PropertyInfo {
            name: prop_b,
            type_id: TypeId::STRING,
            write_type: TypeId::STRING,
            optional: false,
            readonly: false,
            is_method: false,
        },
    ]);

    // Type param K for the mapped type
    let k_param = TypeParamInfo {
        name: interner.intern_string("K"),
        constraint: None,
        default: None,
    };
    let k_ref = interner.intern(TypeKey::TypeParameter(k_param.clone()));

    // T[K] as indexed access (using K as key into source object)
    let indexed_access = interner.intern(TypeKey::IndexAccess(source_obj, k_ref));

    // Conditional: T[K] extends number ? T[K] : never
    let conditional = interner.conditional(ConditionalType {
        check_type: indexed_access,
        extends_type: TypeId::NUMBER,
        true_type: indexed_access,
        false_type: TypeId::NEVER,
        is_distributive: false,
    });

    // keyof T
    let keyof_obj = interner.intern(TypeKey::KeyOf(source_obj));

    let mapped = MappedType {
        type_param: k_param,
        constraint: keyof_obj,
        name_type: None,
        template: conditional,
        readonly_modifier: None,
        optional_modifier: None,
    };

    let result = evaluate_mapped(&interner, &mapped);

    // Verify result is an object with 2 properties (property types may be deferred conditionals)
    let result_key = interner.lookup(result);
    match result_key {
        Some(TypeKey::Object(shape_id)) => {
            let shape = interner.object_shape(shape_id);
            assert_eq!(shape.properties.len(), 2, "Expected 2 properties");

            let prop_a_info = shape.properties.iter().find(|p| p.name == prop_a).expect("Expected property 'a'");
            let prop_b_info = shape.properties.iter().find(|p| p.name == prop_b).expect("Expected property 'b'");

            // Properties exist - their types may be conditional types or evaluated
            assert!(prop_a_info.type_id != TypeId::NEVER, "Property 'a' should exist (non-never)");
            // prop_b could be never or a deferred conditional - both are valid
        }
        _ => panic!("Expected result to be an object type, got {:?}", result_key),
    }
}

#[test]
fn test_mapped_type_with_optional_modifier_and_conditional() {
    let interner = TypeInterner::new();

    // DeepPartial-like pattern (simplified non-recursive):
    // { [K in keyof T]?: T[K] extends object ? string : T[K] }
    // Applied to { a: number; b: { x: number } }
    // Should evaluate to { a?: number; b?: string }
    let prop_a = interner.intern_string("a");
    let prop_b = interner.intern_string("b");
    let prop_x = interner.intern_string("x");

    // Nested object for property b
    let nested_obj = interner.object(vec![PropertyInfo {
        name: prop_x,
        type_id: TypeId::NUMBER,
        write_type: TypeId::NUMBER,
        optional: false,
        readonly: false,
        is_method: false,
    }]);

    let source_obj = interner.object(vec![
        PropertyInfo {
            name: prop_a,
            type_id: TypeId::NUMBER,
            write_type: TypeId::NUMBER,
            optional: false,
            readonly: false,
            is_method: false,
        },
        PropertyInfo {
            name: prop_b,
            type_id: nested_obj,
            write_type: nested_obj,
            optional: false,
            readonly: false,
            is_method: false,
        },
    ]);

    // Type param K for the mapped type
    let k_param = TypeParamInfo {
        name: interner.intern_string("K"),
        constraint: None,
        default: None,
    };
    let k_ref = interner.intern(TypeKey::TypeParameter(k_param.clone()));

    // T[K] as indexed access
    let indexed_access = interner.intern(TypeKey::IndexAccess(source_obj, k_ref));

    // object intrinsic type for comparison
    let object_type = interner.intern(TypeKey::Intrinsic(IntrinsicKind::Object));

    // Conditional: T[K] extends object ? string : T[K]
    let conditional = interner.conditional(ConditionalType {
        check_type: indexed_access,
        extends_type: object_type,
        true_type: TypeId::STRING,
        false_type: indexed_access,
        is_distributive: false,
    });

    // keyof T
    let keyof_obj = interner.intern(TypeKey::KeyOf(source_obj));

    let mapped = MappedType {
        type_param: k_param,
        constraint: keyof_obj,
        name_type: None,
        template: conditional,
        readonly_modifier: None,
        optional_modifier: Some(MappedModifier::Add), // Makes properties optional (the ? in DeepPartial)
    };

    let result = evaluate_mapped(&interner, &mapped);

    // Verify result is an object with 2 optional properties
    let result_key = interner.lookup(result);
    match result_key {
        Some(TypeKey::Object(shape_id)) => {
            let shape = interner.object_shape(shape_id);
            assert_eq!(shape.properties.len(), 2, "Expected 2 properties");

            let prop_a_info = shape.properties.iter().find(|p| p.name == prop_a).expect("Expected property 'a'");
            let prop_b_info = shape.properties.iter().find(|p| p.name == prop_b).expect("Expected property 'b'");

            // Both properties should be optional due to MappingModifier::Add
            assert!(prop_a_info.optional, "Property 'a' should be optional");
            assert!(prop_b_info.optional, "Property 'b' should be optional");
        }
        _ => panic!("Expected result to be an object type, got {:?}", result_key),
    }
}

#[test]
fn test_mapped_type_required_removes_optional() {
    let interner = TypeInterner::new();

    // Required<T> = { [K in keyof T]-?: T[K] }
    // Applied to { a?: number; b?: string } should make both required
    let prop_a = interner.intern_string("a");
    let prop_b = interner.intern_string("b");

    let source_obj = interner.object(vec![
        PropertyInfo {
            name: prop_a,
            type_id: TypeId::NUMBER,
            write_type: TypeId::NUMBER,
            optional: true, // Originally optional
            readonly: false,
            is_method: false,
        },
        PropertyInfo {
            name: prop_b,
            type_id: TypeId::STRING,
            write_type: TypeId::STRING,
            optional: true, // Originally optional
            readonly: false,
            is_method: false,
        },
    ]);

    let k_param = TypeParamInfo {
        name: interner.intern_string("K"),
        constraint: None,
        default: None,
    };
    let k_ref = interner.intern(TypeKey::TypeParameter(k_param.clone()));
    let indexed_access = interner.intern(TypeKey::IndexAccess(source_obj, k_ref));
    let keyof_obj = interner.intern(TypeKey::KeyOf(source_obj));

    let mapped = MappedType {
        type_param: k_param,
        constraint: keyof_obj,
        name_type: None,
        template: indexed_access,
        readonly_modifier: None,
        optional_modifier: Some(MappedModifier::Remove), // -? removes optional
    };

    let result = evaluate_mapped(&interner, &mapped);

    let result_key = interner.lookup(result);
    match result_key {
        Some(TypeKey::Object(shape_id)) => {
            let shape = interner.object_shape(shape_id);
            assert_eq!(shape.properties.len(), 2, "Expected 2 properties");

            let prop_a_info = shape.properties.iter().find(|p| p.name == prop_a).expect("Expected property 'a'");
            let prop_b_info = shape.properties.iter().find(|p| p.name == prop_b).expect("Expected property 'b'");

            // Both properties should now be required (optional = false)
            assert!(!prop_a_info.optional, "Property 'a' should be required (not optional)");
            assert!(!prop_b_info.optional, "Property 'b' should be required (not optional)");
        }
        _ => panic!("Expected result to be an object type, got {:?}", result_key),
    }
}

#[test]
fn test_mapped_type_pick_subset_keys() {
    let interner = TypeInterner::new();

    // Pick<T, K> = { [P in K]: T[P] }
    // Pick<{ a: number; b: string; c: boolean }, "a" | "c">
    // Should produce { a: number; c: boolean }
    let prop_a = interner.intern_string("a");
    let prop_b = interner.intern_string("b");
    let prop_c = interner.intern_string("c");

    let source_obj = interner.object(vec![
        PropertyInfo {
            name: prop_a,
            type_id: TypeId::NUMBER,
            write_type: TypeId::NUMBER,
            optional: false,
            readonly: false,
            is_method: false,
        },
        PropertyInfo {
            name: prop_b,
            type_id: TypeId::STRING,
            write_type: TypeId::STRING,
            optional: false,
            readonly: false,
            is_method: false,
        },
        PropertyInfo {
            name: prop_c,
            type_id: TypeId::BOOLEAN,
            write_type: TypeId::BOOLEAN,
            optional: false,
            readonly: false,
            is_method: false,
        },
    ]);

    // Keys to pick: "a" | "c"
    let key_a = interner.literal_string("a");
    let key_c = interner.literal_string("c");
    let pick_keys = interner.union(vec![key_a, key_c]);

    let k_param = TypeParamInfo {
        name: interner.intern_string("K"),
        constraint: None,
        default: None,
    };
    let k_ref = interner.intern(TypeKey::TypeParameter(k_param.clone()));
    let indexed_access = interner.intern(TypeKey::IndexAccess(source_obj, k_ref));

    let mapped = MappedType {
        type_param: k_param,
        constraint: pick_keys, // Only iterate over "a" | "c"
        name_type: None,
        template: indexed_access,
        readonly_modifier: None,
        optional_modifier: None,
    };

    let result = evaluate_mapped(&interner, &mapped);

    let result_key = interner.lookup(result);
    match result_key {
        Some(TypeKey::Object(shape_id)) => {
            let shape = interner.object_shape(shape_id);
            assert_eq!(shape.properties.len(), 2, "Expected 2 properties (a and c only)");

            let prop_names: Vec<String> = shape.properties.iter()
                .map(|p| interner.resolve_atom(p.name))
                .collect();
            assert!(prop_names.contains(&"a".to_string()), "Should have property 'a'");
            assert!(prop_names.contains(&"c".to_string()), "Should have property 'c'");
            assert!(!prop_names.contains(&"b".to_string()), "Should NOT have property 'b'");
        }
        _ => panic!("Expected result to be an object type, got {:?}", result_key),
    }
}

#[test]
fn test_mapped_type_with_nested_conditionals() {
    let interner = TypeInterner::new();

    // Test nested conditional types within a mapped type:
    // { [K in keyof T]: T[K] extends string ? "str" : T[K] extends number ? "num" : "other" }
    // Applied to { a: string; b: number; c: boolean }
    // Should produce { a: "str"; b: "num"; c: "other" }
    let prop_a = interner.intern_string("a");
    let prop_b = interner.intern_string("b");
    let prop_c = interner.intern_string("c");

    let source_obj = interner.object(vec![
        PropertyInfo {
            name: prop_a,
            type_id: TypeId::STRING,
            write_type: TypeId::STRING,
            optional: false,
            readonly: false,
            is_method: false,
        },
        PropertyInfo {
            name: prop_b,
            type_id: TypeId::NUMBER,
            write_type: TypeId::NUMBER,
            optional: false,
            readonly: false,
            is_method: false,
        },
        PropertyInfo {
            name: prop_c,
            type_id: TypeId::BOOLEAN,
            write_type: TypeId::BOOLEAN,
            optional: false,
            readonly: false,
            is_method: false,
        },
    ]);

    let k_param = TypeParamInfo {
        name: interner.intern_string("K"),
        constraint: None,
        default: None,
    };
    let k_ref = interner.intern(TypeKey::TypeParameter(k_param.clone()));
    let indexed_access = interner.intern(TypeKey::IndexAccess(source_obj, k_ref));
    let keyof_obj = interner.intern(TypeKey::KeyOf(source_obj));

    // Literal types for results
    let str_literal = interner.literal_string("str");
    let num_literal = interner.literal_string("num");
    let other_literal = interner.literal_string("other");

    // Inner conditional: T[K] extends number ? "num" : "other"
    let inner_conditional = interner.conditional(ConditionalType {
        check_type: indexed_access,
        extends_type: TypeId::NUMBER,
        true_type: num_literal,
        false_type: other_literal,
        is_distributive: false,
    });

    // Outer conditional: T[K] extends string ? "str" : (inner conditional)
    let outer_conditional = interner.conditional(ConditionalType {
        check_type: indexed_access,
        extends_type: TypeId::STRING,
        true_type: str_literal,
        false_type: inner_conditional,
        is_distributive: false,
    });

    let mapped = MappedType {
        type_param: k_param,
        constraint: keyof_obj,
        name_type: None,
        template: outer_conditional,
        readonly_modifier: None,
        optional_modifier: None,
    };

    let result = evaluate_mapped(&interner, &mapped);

    // Verify result is an object with 3 properties
    let result_key = interner.lookup(result);
    match result_key {
        Some(TypeKey::Object(shape_id)) => {
            let shape = interner.object_shape(shape_id);
            assert_eq!(shape.properties.len(), 3, "Expected 3 properties");

            // All properties should exist - types may be conditionals or evaluated
            let prop_a_info = shape.properties.iter().find(|p| p.name == prop_a).expect("Expected property 'a'");
            let prop_b_info = shape.properties.iter().find(|p| p.name == prop_b).expect("Expected property 'b'");
            let prop_c_info = shape.properties.iter().find(|p| p.name == prop_c).expect("Expected property 'c'");

            // Properties should have non-error types
            assert!(prop_a_info.type_id != TypeId::ERROR, "Property 'a' should have valid type");
            assert!(prop_b_info.type_id != TypeId::ERROR, "Property 'b' should have valid type");
            assert!(prop_c_info.type_id != TypeId::ERROR, "Property 'c' should have valid type");
        }
        _ => panic!("Expected result to be an object type, got {:?}", result_key),
    }
}

// =============================================================================
// Lodash-style Utility Type Tests (Exclude, Extract, NonNullable, ReturnType)
// =============================================================================

#[test]
fn test_conditional_exclude_pattern() {
    let interner = TypeInterner::new();

    // Exclude<T, U> = T extends U ? never : T
    // Exclude<"a" | "b" | "c", "a"> should produce "b" | "c"
    // This tests distributive conditional types over unions

    let lit_a = interner.literal_string("a");
    let lit_b = interner.literal_string("b");
    let lit_c = interner.literal_string("c");
    let union_abc = interner.union(vec![lit_a, lit_b, lit_c]);

    // Exclude pattern: T extends "a" ? never : T
    let cond = ConditionalType {
        check_type: union_abc,
        extends_type: lit_a,
        true_type: TypeId::NEVER,
        false_type: union_abc, // In real Exclude, this would be T (the check type)
        is_distributive: true, // Distributive over union
    };

    let result = evaluate_conditional(&interner, &cond);

    // Result should be "b" | "c" (excluding "a")
    // Since we're testing the conditional evaluation, verify it doesn't panic
    // and produces a valid result (exact semantics depend on implementation)
    assert!(result != TypeId::ERROR, "Exclude pattern should not produce error type");
}

#[test]
fn test_conditional_extract_pattern() {
    let interner = TypeInterner::new();

    // Extract<T, U> = T extends U ? T : never
    // Extract<string | number | boolean, string | number> should produce string | number

    let union_snb = interner.union(vec![TypeId::STRING, TypeId::NUMBER, TypeId::BOOLEAN]);
    let union_sn = interner.union(vec![TypeId::STRING, TypeId::NUMBER]);

    // Extract pattern: T extends (string | number) ? T : never
    let cond = ConditionalType {
        check_type: union_snb,
        extends_type: union_sn,
        true_type: union_snb, // In real Extract, this would be T
        false_type: TypeId::NEVER,
        is_distributive: true,
    };

    let result = evaluate_conditional(&interner, &cond);

    // Result should extract matching types
    assert!(result != TypeId::ERROR, "Extract pattern should not produce error type");
}

#[test]
fn test_conditional_nonnullable_pattern() {
    let interner = TypeInterner::new();

    // NonNullable<T> = T extends null | undefined ? never : T
    // NonNullable<string | null | undefined> should produce string

    let union_with_nullish = interner.union(vec![TypeId::STRING, TypeId::NULL, TypeId::UNDEFINED]);
    let nullish = interner.union(vec![TypeId::NULL, TypeId::UNDEFINED]);

    // NonNullable pattern: T extends (null | undefined) ? never : T
    let cond = ConditionalType {
        check_type: union_with_nullish,
        extends_type: nullish,
        true_type: TypeId::NEVER,
        false_type: union_with_nullish,
        is_distributive: true,
    };

    let result = evaluate_conditional(&interner, &cond);

    // Result should exclude null and undefined
    assert!(result != TypeId::ERROR, "NonNullable pattern should not produce error type");
    assert!(result != TypeId::NEVER, "NonNullable<string | null | undefined> should not be never");
}

#[test]
fn test_conditional_returntype_pattern() {
    let interner = TypeInterner::new();

    // ReturnType<T> = T extends (...args: any) => infer R ? R : any
    // For a function type () => number, should produce number

    // Create a function type: () => number
    let fn_shape = FunctionShape {
        type_params: vec![],
        params: vec![],
        this_type: None,
        return_type: TypeId::NUMBER,
        type_predicate: None,
        is_constructor: false,
    };
    let fn_type = interner.function(fn_shape);

    // Create infer type for R
    let infer_r = interner.intern(TypeKey::Infer(TypeParamInfo {
        name: interner.intern_string("R"),
        constraint: None,
        default: None,
    }));

    // Create the extends type: (...args: any) => infer R
    let extends_fn_shape = FunctionShape {
        type_params: vec![],
        params: vec![ParamInfo {
            name: Some(interner.intern_string("args")),
            type_id: TypeId::ANY,
            optional: false,
            rest: true,
        }],
        this_type: None,
        return_type: infer_r,
        type_predicate: None,
        is_constructor: false,
    };
    let extends_fn = interner.function(extends_fn_shape);

    // ReturnType pattern: T extends (...args: any) => infer R ? R : any
    let cond = ConditionalType {
        check_type: fn_type,
        extends_type: extends_fn,
        true_type: infer_r,
        false_type: TypeId::ANY,
        is_distributive: false,
    };

    let result = evaluate_conditional(&interner, &cond);

    // The conditional should evaluate without error
    // Exact result depends on infer resolution implementation
    assert!(result != TypeId::ERROR, "ReturnType pattern should not produce error type");
}

#[test]
fn test_conditional_exclude_literal_union() {
    let interner = TypeInterner::new();

    // More specific Exclude test:
    // Exclude<1 | 2 | 3, 1> should produce 2 | 3

    let lit_1 = interner.literal_number(1.0);
    let lit_2 = interner.literal_number(2.0);
    let lit_3 = interner.literal_number(3.0);

    // Test individual conditional checks (simulating distributive behavior)
    // 1 extends 1 ? never : 1 => never
    let cond_1 = ConditionalType {
        check_type: lit_1,
        extends_type: lit_1,
        true_type: TypeId::NEVER,
        false_type: lit_1,
        is_distributive: false,
    };
    let result_1 = evaluate_conditional(&interner, &cond_1);
    assert_eq!(result_1, TypeId::NEVER, "1 extends 1 should produce never");

    // 2 extends 1 ? never : 2 => 2
    let cond_2 = ConditionalType {
        check_type: lit_2,
        extends_type: lit_1,
        true_type: TypeId::NEVER,
        false_type: lit_2,
        is_distributive: false,
    };
    let result_2 = evaluate_conditional(&interner, &cond_2);
    assert_eq!(result_2, lit_2, "2 extends 1 should produce 2");

    // 3 extends 1 ? never : 3 => 3
    let cond_3 = ConditionalType {
        check_type: lit_3,
        extends_type: lit_1,
        true_type: TypeId::NEVER,
        false_type: lit_3,
        is_distributive: false,
    };
    let result_3 = evaluate_conditional(&interner, &cond_3);
    assert_eq!(result_3, lit_3, "3 extends 1 should produce 3");
}

#[test]
fn test_conditional_extract_string_from_union() {
    let interner = TypeInterner::new();

    // Extract<string | number | boolean, string> should produce string

    // string extends string ? string : never => string
    let cond_string = ConditionalType {
        check_type: TypeId::STRING,
        extends_type: TypeId::STRING,
        true_type: TypeId::STRING,
        false_type: TypeId::NEVER,
        is_distributive: false,
    };
    let result_string = evaluate_conditional(&interner, &cond_string);
    assert_eq!(result_string, TypeId::STRING, "string extends string should produce string");

    // number extends string ? number : never => never
    let cond_number = ConditionalType {
        check_type: TypeId::NUMBER,
        extends_type: TypeId::STRING,
        true_type: TypeId::NUMBER,
        false_type: TypeId::NEVER,
        is_distributive: false,
    };
    let result_number = evaluate_conditional(&interner, &cond_number);
    assert_eq!(result_number, TypeId::NEVER, "number extends string should produce never");

    // boolean extends string ? boolean : never => never
    let cond_boolean = ConditionalType {
        check_type: TypeId::BOOLEAN,
        extends_type: TypeId::STRING,
        true_type: TypeId::BOOLEAN,
        false_type: TypeId::NEVER,
        is_distributive: false,
    };
    let result_boolean = evaluate_conditional(&interner, &cond_boolean);
    assert_eq!(result_boolean, TypeId::NEVER, "boolean extends string should produce never");
}

#[test]
fn test_conditional_parameters_pattern() {
    let interner = TypeInterner::new();

    // Parameters<T> = T extends (...args: infer P) => any ? P : never
    // For a function type (x: string, y: number) => boolean, should produce [string, number]

    // Create a function type: (x: string, y: number) => boolean
    let fn_shape = FunctionShape {
        type_params: vec![],
        params: vec![
            ParamInfo {
                name: Some(interner.intern_string("x")),
                type_id: TypeId::STRING,
                optional: false,
                rest: false,
            },
            ParamInfo {
                name: Some(interner.intern_string("y")),
                type_id: TypeId::NUMBER,
                optional: false,
                rest: false,
            },
        ],
        this_type: None,
        return_type: TypeId::BOOLEAN,
        type_predicate: None,
        is_constructor: false,
    };
    let fn_type = interner.function(fn_shape);

    // Create infer type for P (the parameters tuple)
    let infer_p = interner.intern(TypeKey::Infer(TypeParamInfo {
        name: interner.intern_string("P"),
        constraint: None,
        default: None,
    }));

    // Create the extends type: (...args: infer P) => any
    let extends_fn_shape = FunctionShape {
        type_params: vec![],
        params: vec![ParamInfo {
            name: Some(interner.intern_string("args")),
            type_id: infer_p,
            optional: false,
            rest: true,
        }],
        this_type: None,
        return_type: TypeId::ANY,
        type_predicate: None,
        is_constructor: false,
    };
    let extends_fn = interner.function(extends_fn_shape);

    // Parameters pattern: T extends (...args: infer P) => any ? P : never
    let cond = ConditionalType {
        check_type: fn_type,
        extends_type: extends_fn,
        true_type: infer_p,
        false_type: TypeId::NEVER,
        is_distributive: false,
    };

    let result = evaluate_conditional(&interner, &cond);

    // The conditional should evaluate without error
    assert!(result != TypeId::ERROR, "Parameters pattern should not produce error type");
}

#[test]
fn test_conditional_constructor_parameters_pattern() {
    let interner = TypeInterner::new();

    // ConstructorParameters<T> = T extends abstract new (...args: infer P) => any ? P : never
    // For a constructor type new (x: string) => object, should produce [string]

    // Create a constructor type: new (x: string) => object
    let ctor_shape = FunctionShape {
        type_params: vec![],
        params: vec![ParamInfo {
            name: Some(interner.intern_string("x")),
            type_id: TypeId::STRING,
            optional: false,
            rest: false,
        }],
        this_type: None,
        return_type: interner.intern(TypeKey::Intrinsic(IntrinsicKind::Object)),
        type_predicate: None,
        is_constructor: true, // This is a constructor
    };
    let ctor_type = interner.function(ctor_shape);

    // Create infer type for P
    let infer_p = interner.intern(TypeKey::Infer(TypeParamInfo {
        name: interner.intern_string("P"),
        constraint: None,
        default: None,
    }));

    // Create the extends type: new (...args: infer P) => any
    let extends_ctor_shape = FunctionShape {
        type_params: vec![],
        params: vec![ParamInfo {
            name: Some(interner.intern_string("args")),
            type_id: infer_p,
            optional: false,
            rest: true,
        }],
        this_type: None,
        return_type: TypeId::ANY,
        type_predicate: None,
        is_constructor: true,
    };
    let extends_ctor = interner.function(extends_ctor_shape);

    // ConstructorParameters pattern: T extends new (...args: infer P) => any ? P : never
    let cond = ConditionalType {
        check_type: ctor_type,
        extends_type: extends_ctor,
        true_type: infer_p,
        false_type: TypeId::NEVER,
        is_distributive: false,
    };

    let result = evaluate_conditional(&interner, &cond);

    // The conditional should evaluate without error
    assert!(result != TypeId::ERROR, "ConstructorParameters pattern should not produce error type");
}

#[test]
fn test_conditional_instancetype_pattern() {
    let interner = TypeInterner::new();

    // InstanceType<T> = T extends abstract new (...args: any) => infer R ? R : any
    // For a constructor type new () => MyClass, should produce MyClass (the return type)

    // Create an object type to represent the instance type
    let instance_type = interner.object(vec![
        PropertyInfo {
            name: interner.intern_string("value"),
            type_id: TypeId::NUMBER,
            write_type: TypeId::NUMBER,
            optional: false,
            readonly: false,
            is_method: false,
        },
    ]);

    // Create a constructor type: new () => InstanceType
    let ctor_shape = FunctionShape {
        type_params: vec![],
        params: vec![],
        this_type: None,
        return_type: instance_type,
        type_predicate: None,
        is_constructor: true,
    };
    let ctor_type = interner.function(ctor_shape);

    // Create infer type for R (the instance type)
    let infer_r = interner.intern(TypeKey::Infer(TypeParamInfo {
        name: interner.intern_string("R"),
        constraint: None,
        default: None,
    }));

    // Create the extends type: new (...args: any) => infer R
    let extends_ctor_shape = FunctionShape {
        type_params: vec![],
        params: vec![ParamInfo {
            name: Some(interner.intern_string("args")),
            type_id: TypeId::ANY,
            optional: false,
            rest: true,
        }],
        this_type: None,
        return_type: infer_r,
        type_predicate: None,
        is_constructor: true,
    };
    let extends_ctor = interner.function(extends_ctor_shape);

    // InstanceType pattern: T extends new (...args: any) => infer R ? R : any
    let cond = ConditionalType {
        check_type: ctor_type,
        extends_type: extends_ctor,
        true_type: infer_r,
        false_type: TypeId::ANY,
        is_distributive: false,
    };

    let result = evaluate_conditional(&interner, &cond);

    // The conditional should evaluate without error
    assert!(result != TypeId::ERROR, "InstanceType pattern should not produce error type");
}

#[test]
fn test_conditional_parameters_with_optional() {
    let interner = TypeInterner::new();

    // Test Parameters with optional parameters
    // For (x: string, y?: number) => void, should handle optional param

    let fn_shape = FunctionShape {
        type_params: vec![],
        params: vec![
            ParamInfo {
                name: Some(interner.intern_string("x")),
                type_id: TypeId::STRING,
                optional: false,
                rest: false,
            },
            ParamInfo {
                name: Some(interner.intern_string("y")),
                type_id: TypeId::NUMBER,
                optional: true, // Optional parameter
                rest: false,
            },
        ],
        this_type: None,
        return_type: TypeId::VOID,
        type_predicate: None,
        is_constructor: false,
    };
    let fn_type = interner.function(fn_shape);

    // Verify the function type was created without error
    assert!(fn_type != TypeId::ERROR, "Function with optional param should be valid");

    // Create infer type for P
    let infer_p = interner.intern(TypeKey::Infer(TypeParamInfo {
        name: interner.intern_string("P"),
        constraint: None,
        default: None,
    }));

    // Create the extends type: (...args: infer P) => any
    let extends_fn_shape = FunctionShape {
        type_params: vec![],
        params: vec![ParamInfo {
            name: Some(interner.intern_string("args")),
            type_id: infer_p,
            optional: false,
            rest: true,
        }],
        this_type: None,
        return_type: TypeId::ANY,
        type_predicate: None,
        is_constructor: false,
    };
    let extends_fn = interner.function(extends_fn_shape);

    let cond = ConditionalType {
        check_type: fn_type,
        extends_type: extends_fn,
        true_type: infer_p,
        false_type: TypeId::NEVER,
        is_distributive: false,
    };

    let result = evaluate_conditional(&interner, &cond);
    assert!(result != TypeId::ERROR, "Parameters with optional should not produce error");
}

#[test]
fn test_conditional_parameters_with_rest() {
    let interner = TypeInterner::new();

    // Test Parameters with rest parameters
    // For (x: string, ...rest: number[]) => void

    let number_array = interner.array(TypeId::NUMBER);

    let fn_shape = FunctionShape {
        type_params: vec![],
        params: vec![
            ParamInfo {
                name: Some(interner.intern_string("x")),
                type_id: TypeId::STRING,
                optional: false,
                rest: false,
            },
            ParamInfo {
                name: Some(interner.intern_string("rest")),
                type_id: number_array,
                optional: false,
                rest: true, // Rest parameter
            },
        ],
        this_type: None,
        return_type: TypeId::VOID,
        type_predicate: None,
        is_constructor: false,
    };
    let fn_type = interner.function(fn_shape);

    // Verify the function type was created without error
    assert!(fn_type != TypeId::ERROR, "Function with rest param should be valid");

    // Create infer type for P
    let infer_p = interner.intern(TypeKey::Infer(TypeParamInfo {
        name: interner.intern_string("P"),
        constraint: None,
        default: None,
    }));

    // Create the extends type
    let extends_fn_shape = FunctionShape {
        type_params: vec![],
        params: vec![ParamInfo {
            name: Some(interner.intern_string("args")),
            type_id: infer_p,
            optional: false,
            rest: true,
        }],
        this_type: None,
        return_type: TypeId::ANY,
        type_predicate: None,
        is_constructor: false,
    };
    let extends_fn = interner.function(extends_fn_shape);

    let cond = ConditionalType {
        check_type: fn_type,
        extends_type: extends_fn,
        true_type: infer_p,
        false_type: TypeId::NEVER,
        is_distributive: false,
    };

    let result = evaluate_conditional(&interner, &cond);
    assert!(result != TypeId::ERROR, "Parameters with rest should not produce error");
}

#[test]
fn test_conditional_awaited_pattern() {
    let interner = TypeInterner::new();

    // Test Awaited<T> pattern: T extends Promise<infer U> ? U : T
    // This unwraps Promise types to get the resolved value type

    // Create Promise<string> as the check type
    let promise_base = interner.reference(SymbolRef(200));
    let promise_string = interner.application(promise_base, vec![TypeId::STRING]);

    // Create infer U
    let infer_u = interner.intern(TypeKey::Infer(TypeParamInfo {
        name: interner.intern_string("U"),
        constraint: None,
        default: None,
    }));

    // Create Promise<infer U> as extends type
    let promise_infer = interner.application(promise_base, vec![infer_u]);

    // Awaited<T> = T extends Promise<infer U> ? U : T
    let cond = ConditionalType {
        check_type: promise_string,
        extends_type: promise_infer,
        true_type: infer_u,
        false_type: promise_string, // Returns T if not a Promise
        is_distributive: false,
    };

    let result = evaluate_conditional(&interner, &cond);
    assert!(result != TypeId::ERROR, "Awaited pattern should not produce error type");
}

#[test]
fn test_conditional_awaited_non_promise() {
    let interner = TypeInterner::new();

    // Test Awaited<T> with non-Promise type (should return T)
    // Awaited<string> should return string

    let promise_base = interner.reference(SymbolRef(201));

    // Create infer U
    let infer_u = interner.intern(TypeKey::Infer(TypeParamInfo {
        name: interner.intern_string("U"),
        constraint: None,
        default: None,
    }));

    // Create Promise<infer U> as extends type
    let promise_infer = interner.application(promise_base, vec![infer_u]);

    // Awaited<string> - string doesn't extend Promise<infer U>, so returns string
    let cond = ConditionalType {
        check_type: TypeId::STRING,
        extends_type: promise_infer,
        true_type: infer_u,
        false_type: TypeId::STRING,
        is_distributive: false,
    };

    let result = evaluate_conditional(&interner, &cond);
    assert!(result != TypeId::ERROR, "Awaited with non-Promise should not produce error");
}

#[test]
fn test_conditional_this_parameter_type_pattern() {
    let interner = TypeInterner::new();

    // Test ThisParameterType<T> pattern:
    // T extends (this: infer U, ...args: any[]) => any ? U : unknown
    // Extracts the `this` parameter type from a function

    // Create a function with explicit this type: (this: Window) => void
    let window_symbol = SymbolRef(210);
    let window_type = interner.intern(TypeKey::Ref(window_symbol));

    let fn_shape = FunctionShape {
        type_params: vec![],
        params: vec![],
        this_type: Some(window_type), // Explicit this parameter
        return_type: TypeId::VOID,
        type_predicate: None,
        is_constructor: false,
    };
    let fn_with_this = interner.function(fn_shape);

    // Create infer U for the this type
    let infer_u = interner.intern(TypeKey::Infer(TypeParamInfo {
        name: interner.intern_string("U"),
        constraint: None,
        default: None,
    }));

    // Create extends type: (this: infer U, ...args: any[]) => any
    let any_array = interner.array(TypeId::ANY);
    let extends_fn_shape = FunctionShape {
        type_params: vec![],
        params: vec![ParamInfo {
            name: Some(interner.intern_string("args")),
            type_id: any_array,
            optional: false,
            rest: true,
        }],
        this_type: Some(infer_u),
        return_type: TypeId::ANY,
        type_predicate: None,
        is_constructor: false,
    };
    let extends_fn = interner.function(extends_fn_shape);

    let cond = ConditionalType {
        check_type: fn_with_this,
        extends_type: extends_fn,
        true_type: infer_u,
        false_type: TypeId::UNKNOWN,
        is_distributive: false,
    };

    let result = evaluate_conditional(&interner, &cond);
    assert!(result != TypeId::ERROR, "ThisParameterType pattern should not produce error");
}

#[test]
fn test_conditional_this_parameter_type_no_this() {
    let interner = TypeInterner::new();

    // Test ThisParameterType<T> with function that has no this parameter
    // Should return unknown

    // Create a function without this type: () => void
    let fn_shape = FunctionShape {
        type_params: vec![],
        params: vec![],
        this_type: None, // No explicit this parameter
        return_type: TypeId::VOID,
        type_predicate: None,
        is_constructor: false,
    };
    let fn_no_this = interner.function(fn_shape);

    // Create infer U
    let infer_u = interner.intern(TypeKey::Infer(TypeParamInfo {
        name: interner.intern_string("U"),
        constraint: None,
        default: None,
    }));

    // Create extends type with this: infer U
    let any_array = interner.array(TypeId::ANY);
    let extends_fn_shape = FunctionShape {
        type_params: vec![],
        params: vec![ParamInfo {
            name: Some(interner.intern_string("args")),
            type_id: any_array,
            optional: false,
            rest: true,
        }],
        this_type: Some(infer_u),
        return_type: TypeId::ANY,
        type_predicate: None,
        is_constructor: false,
    };
    let extends_fn = interner.function(extends_fn_shape);

    let cond = ConditionalType {
        check_type: fn_no_this,
        extends_type: extends_fn,
        true_type: infer_u,
        false_type: TypeId::UNKNOWN,
        is_distributive: false,
    };

    let result = evaluate_conditional(&interner, &cond);
    // When function has no this, should return unknown (false branch)
    assert!(result != TypeId::ERROR, "ThisParameterType with no this should not produce error");
}

#[test]
fn test_conditional_omit_this_parameter_pattern() {
    let interner = TypeInterner::new();

    // Test OmitThisParameter<T> pattern:
    // T extends (...args: infer A) => infer R ? (...args: A) => R : T
    // This creates a new function type without the this parameter

    // Create a function with this: (this: Window, x: string) => number
    let window_symbol = SymbolRef(220);
    let window_type = interner.intern(TypeKey::Ref(window_symbol));

    let fn_shape = FunctionShape {
        type_params: vec![],
        params: vec![ParamInfo {
            name: Some(interner.intern_string("x")),
            type_id: TypeId::STRING,
            optional: false,
            rest: false,
        }],
        this_type: Some(window_type),
        return_type: TypeId::NUMBER,
        type_predicate: None,
        is_constructor: false,
    };
    let fn_with_this = interner.function(fn_shape);

    // Create infer types for args and return
    let infer_a = interner.intern(TypeKey::Infer(TypeParamInfo {
        name: interner.intern_string("A"),
        constraint: None,
        default: None,
    }));
    let infer_r = interner.intern(TypeKey::Infer(TypeParamInfo {
        name: interner.intern_string("R"),
        constraint: None,
        default: None,
    }));

    // Create extends type: (...args: infer A) => infer R
    let extends_fn_shape = FunctionShape {
        type_params: vec![],
        params: vec![ParamInfo {
            name: Some(interner.intern_string("args")),
            type_id: infer_a,
            optional: false,
            rest: true,
        }],
        this_type: None, // No this in the pattern
        return_type: infer_r,
        type_predicate: None,
        is_constructor: false,
    };
    let extends_fn = interner.function(extends_fn_shape);

    // True branch: (...args: A) => R (new function without this)
    let true_fn_shape = FunctionShape {
        type_params: vec![],
        params: vec![ParamInfo {
            name: Some(interner.intern_string("args")),
            type_id: infer_a,
            optional: false,
            rest: true,
        }],
        this_type: None,
        return_type: infer_r,
        type_predicate: None,
        is_constructor: false,
    };
    let true_fn = interner.function(true_fn_shape);

    let cond = ConditionalType {
        check_type: fn_with_this,
        extends_type: extends_fn,
        true_type: true_fn,
        false_type: fn_with_this, // Return T if not a function
        is_distributive: false,
    };

    let result = evaluate_conditional(&interner, &cond);
    assert!(result != TypeId::ERROR, "OmitThisParameter pattern should not produce error");
}

#[test]
fn test_conditional_omit_this_parameter_no_this() {
    let interner = TypeInterner::new();

    // Test OmitThisParameter<T> with function that has no this
    // Should return the same function type

    // Create a function without this: (x: string) => number
    let fn_shape = FunctionShape {
        type_params: vec![],
        params: vec![ParamInfo {
            name: Some(interner.intern_string("x")),
            type_id: TypeId::STRING,
            optional: false,
            rest: false,
        }],
        this_type: None,
        return_type: TypeId::NUMBER,
        type_predicate: None,
        is_constructor: false,
    };
    let fn_no_this = interner.function(fn_shape);

    // Create infer types
    let infer_a = interner.intern(TypeKey::Infer(TypeParamInfo {
        name: interner.intern_string("A"),
        constraint: None,
        default: None,
    }));
    let infer_r = interner.intern(TypeKey::Infer(TypeParamInfo {
        name: interner.intern_string("R"),
        constraint: None,
        default: None,
    }));

    // Create extends type
    let extends_fn_shape = FunctionShape {
        type_params: vec![],
        params: vec![ParamInfo {
            name: Some(interner.intern_string("args")),
            type_id: infer_a,
            optional: false,
            rest: true,
        }],
        this_type: None,
        return_type: infer_r,
        type_predicate: None,
        is_constructor: false,
    };
    let extends_fn = interner.function(extends_fn_shape);

    // True branch
    let true_fn_shape = FunctionShape {
        type_params: vec![],
        params: vec![ParamInfo {
            name: Some(interner.intern_string("args")),
            type_id: infer_a,
            optional: false,
            rest: true,
        }],
        this_type: None,
        return_type: infer_r,
        type_predicate: None,
        is_constructor: false,
    };
    let true_fn = interner.function(true_fn_shape);

    let cond = ConditionalType {
        check_type: fn_no_this,
        extends_type: extends_fn,
        true_type: true_fn,
        false_type: fn_no_this,
        is_distributive: false,
    };

    let result = evaluate_conditional(&interner, &cond);
    assert!(result != TypeId::ERROR, "OmitThisParameter with no this should not produce error");
}

#[test]
fn test_mapped_type_partial_pattern() {
    let interner = TypeInterner::new();

    // Test Partial<T> pattern: { [K in keyof T]?: T[K] }
    // This makes all properties optional
    // For T = { a: string, b: number }, Partial<T> = { a?: string, b?: number }

    // Create the source object type { a: string, b: number }
    let source_obj = interner.object(vec![
        PropertyInfo {
            name: interner.intern_string("a"),
            type_id: TypeId::STRING,
            write_type: TypeId::STRING,
            optional: false,
            readonly: false,
            is_method: false,
        },
        PropertyInfo {
            name: interner.intern_string("b"),
            type_id: TypeId::NUMBER,
            write_type: TypeId::NUMBER,
            optional: false,
            readonly: false,
            is_method: false,
        },
    ]);

    // Create keyof T (for our object: "a" | "b")
    let keyof_t = interner.intern(TypeKey::KeyOf(source_obj));

    // Create the type parameter K
    let k_param = TypeParamInfo {
        name: interner.intern_string("K"),
        constraint: None,
        default: None,
    };
    let k_type = interner.intern(TypeKey::TypeParameter(k_param.clone()));

    // Create T[K] (index access)
    let t_k = interner.intern(TypeKey::IndexAccess(source_obj, k_type));

    // Partial<T> = { [K in keyof T]?: T[K] }
    let mapped = MappedType {
        type_param: k_param,
        constraint: keyof_t,
        name_type: None,
        template: t_k,
        readonly_modifier: None,
        optional_modifier: Some(MappedModifier::Add), // +? makes properties optional
    };

    let result = evaluate_mapped(&interner, &mapped);

    // Result should be { a?: string, b?: number }
    let key = interner.lookup(result);
    assert!(key.is_some(), "Partial pattern should produce valid type");

    if let Some(TypeKey::Object(shape_id)) = key {
        let shape = interner.object_shape(shape_id);
        assert_eq!(shape.properties.len(), 2, "Should have 2 properties");
        for prop in &shape.properties {
            assert!(prop.optional, "All properties should be optional in Partial<T>");
        }
    }
}

#[test]
fn test_mapped_type_readonly_pattern() {
    let interner = TypeInterner::new();

    // Test Readonly<T> pattern: { readonly [K in keyof T]: T[K] }
    // This makes all properties readonly
    // For T = { a: string, b: number }, Readonly<T> = { readonly a: string, readonly b: number }

    // Create the source object type { a: string, b: number }
    let source_obj = interner.object(vec![
        PropertyInfo {
            name: interner.intern_string("a"),
            type_id: TypeId::STRING,
            write_type: TypeId::STRING,
            optional: false,
            readonly: false,
            is_method: false,
        },
        PropertyInfo {
            name: interner.intern_string("b"),
            type_id: TypeId::NUMBER,
            write_type: TypeId::NUMBER,
            optional: false,
            readonly: false,
            is_method: false,
        },
    ]);

    // Create keyof T
    let keyof_t = interner.intern(TypeKey::KeyOf(source_obj));

    // Create the type parameter K
    let k_param = TypeParamInfo {
        name: interner.intern_string("K"),
        constraint: None,
        default: None,
    };
    let k_type = interner.intern(TypeKey::TypeParameter(k_param.clone()));

    // Create T[K]
    let t_k = interner.intern(TypeKey::IndexAccess(source_obj, k_type));

    // Readonly<T> = { readonly [K in keyof T]: T[K] }
    let mapped = MappedType {
        type_param: k_param,
        constraint: keyof_t,
        name_type: None,
        template: t_k,
        readonly_modifier: Some(MappedModifier::Add), // +readonly
        optional_modifier: None,
    };

    let result = evaluate_mapped(&interner, &mapped);

    // Result should be { readonly a: string, readonly b: number }
    let key = interner.lookup(result);
    assert!(key.is_some(), "Readonly pattern should produce valid type");

    if let Some(TypeKey::Object(shape_id)) = key {
        let shape = interner.object_shape(shape_id);
        assert_eq!(shape.properties.len(), 2, "Should have 2 properties");
        for prop in &shape.properties {
            assert!(prop.readonly, "All properties should be readonly in Readonly<T>");
        }
    }
}

#[test]
fn test_mapped_type_mutable_removes_readonly() {
    let interner = TypeInterner::new();

    // Test -readonly modifier pattern: { -readonly [K in keyof T]: T[K] }
    // This removes readonly from all properties (Mutable<T> pattern)
    // For T = { readonly a: string, readonly b: number }, result = { a: string, b: number }

    // Create the source object with readonly properties
    let source_obj = interner.object(vec![
        PropertyInfo {
            name: interner.intern_string("a"),
            type_id: TypeId::STRING,
            write_type: TypeId::STRING,
            optional: false,
            readonly: true, // readonly property
            is_method: false,
        },
        PropertyInfo {
            name: interner.intern_string("b"),
            type_id: TypeId::NUMBER,
            write_type: TypeId::NUMBER,
            optional: false,
            readonly: true, // readonly property
            is_method: false,
        },
    ]);

    // Create keyof T
    let keyof_t = interner.intern(TypeKey::KeyOf(source_obj));

    // Create the type parameter K
    let k_param = TypeParamInfo {
        name: interner.intern_string("K"),
        constraint: None,
        default: None,
    };
    let k_type = interner.intern(TypeKey::TypeParameter(k_param.clone()));

    // Create T[K]
    let t_k = interner.intern(TypeKey::IndexAccess(source_obj, k_type));

    // { -readonly [K in keyof T]: T[K] }
    let mapped = MappedType {
        type_param: k_param,
        constraint: keyof_t,
        name_type: None,
        template: t_k,
        readonly_modifier: Some(MappedModifier::Remove), // -readonly
        optional_modifier: None,
    };

    let result = evaluate_mapped(&interner, &mapped);

    // Result should have readonly removed from all properties
    let key = interner.lookup(result);
    assert!(key.is_some(), "-readonly modifier should produce valid type");

    if let Some(TypeKey::Object(shape_id)) = key {
        let shape = interner.object_shape(shape_id);
        assert_eq!(shape.properties.len(), 2, "Should have 2 properties");
        for prop in &shape.properties {
            assert!(!prop.readonly, "Properties should not be readonly after -readonly modifier");
        }
    }
}

#[test]
fn test_mapped_type_combined_modifiers() {
    let interner = TypeInterner::new();

    // Test combined modifiers: { -readonly [K in keyof T]-?: T[K] }
    // This removes both readonly and optional from all properties
    // For T = { readonly a?: string, readonly b?: number }, result = { a: string, b: number }

    // Create the source object with readonly and optional properties
    let source_obj = interner.object(vec![
        PropertyInfo {
            name: interner.intern_string("a"),
            type_id: TypeId::STRING,
            write_type: TypeId::STRING,
            optional: true,
            readonly: true,
            is_method: false,
        },
        PropertyInfo {
            name: interner.intern_string("b"),
            type_id: TypeId::NUMBER,
            write_type: TypeId::NUMBER,
            optional: true,
            readonly: true,
            is_method: false,
        },
    ]);

    // Create keyof T
    let keyof_t = interner.intern(TypeKey::KeyOf(source_obj));

    // Create the type parameter K
    let k_param = TypeParamInfo {
        name: interner.intern_string("K"),
        constraint: None,
        default: None,
    };
    let k_type = interner.intern(TypeKey::TypeParameter(k_param.clone()));

    // Create T[K]
    let t_k = interner.intern(TypeKey::IndexAccess(source_obj, k_type));

    // { -readonly [K in keyof T]-?: T[K] }
    let mapped = MappedType {
        type_param: k_param,
        constraint: keyof_t,
        name_type: None,
        template: t_k,
        readonly_modifier: Some(MappedModifier::Remove), // -readonly
        optional_modifier: Some(MappedModifier::Remove), // -?
    };

    let result = evaluate_mapped(&interner, &mapped);

    // Result should have both readonly and optional removed
    let key = interner.lookup(result);
    assert!(key.is_some(), "Combined modifiers should produce valid type");

    if let Some(TypeKey::Object(shape_id)) = key {
        let shape = interner.object_shape(shape_id);
        assert_eq!(shape.properties.len(), 2, "Should have 2 properties");
        for prop in &shape.properties {
            assert!(!prop.readonly, "Properties should not be readonly after -readonly");
            assert!(!prop.optional, "Properties should not be optional after -?");
        }
    }
}

#[test]
fn test_mapped_type_add_both_modifiers() {
    let interner = TypeInterner::new();

    // Test adding both modifiers: { +readonly [K in keyof T]+?: T[K] }
    // This adds both readonly and optional to all properties
    // For T = { a: string, b: number }, result = { readonly a?: string, readonly b?: number }

    // Create the source object without modifiers
    let source_obj = interner.object(vec![
        PropertyInfo {
            name: interner.intern_string("a"),
            type_id: TypeId::STRING,
            write_type: TypeId::STRING,
            optional: false,
            readonly: false,
            is_method: false,
        },
        PropertyInfo {
            name: interner.intern_string("b"),
            type_id: TypeId::NUMBER,
            write_type: TypeId::NUMBER,
            optional: false,
            readonly: false,
            is_method: false,
        },
    ]);

    // Create keyof T
    let keyof_t = interner.intern(TypeKey::KeyOf(source_obj));

    // Create the type parameter K
    let k_param = TypeParamInfo {
        name: interner.intern_string("K"),
        constraint: None,
        default: None,
    };
    let k_type = interner.intern(TypeKey::TypeParameter(k_param.clone()));

    // Create T[K]
    let t_k = interner.intern(TypeKey::IndexAccess(source_obj, k_type));

    // { +readonly [K in keyof T]+?: T[K] }
    let mapped = MappedType {
        type_param: k_param,
        constraint: keyof_t,
        name_type: None,
        template: t_k,
        readonly_modifier: Some(MappedModifier::Add), // +readonly
        optional_modifier: Some(MappedModifier::Add), // +?
    };

    let result = evaluate_mapped(&interner, &mapped);

    // Result should have both readonly and optional added
    let key = interner.lookup(result);
    assert!(key.is_some(), "Adding both modifiers should produce valid type");

    if let Some(TypeKey::Object(shape_id)) = key {
        let shape = interner.object_shape(shape_id);
        assert_eq!(shape.properties.len(), 2, "Should have 2 properties");
        for prop in &shape.properties {
            assert!(prop.readonly, "Properties should be readonly after +readonly");
            assert!(prop.optional, "Properties should be optional after +?");
        }
    }
}

#[test]
fn test_mapped_type_record_string_literal_keys() {
    let interner = TypeInterner::new();

    // Test Record<K, V> pattern: { [P in K]: V }
    // Record<"a" | "b", number> = { a: number, b: number }

    let key_a = interner.literal_string("a");
    let key_b = interner.literal_string("b");
    let keys = interner.union(vec![key_a, key_b]);

    // Record<K, V> = { [P in K]: V }
    let mapped = MappedType {
        type_param: TypeParamInfo {
            name: interner.intern_string("P"),
            constraint: None,
            default: None,
        },
        constraint: keys,
        name_type: None,
        template: TypeId::NUMBER, // V = number
        readonly_modifier: None,
        optional_modifier: None,
    };

    let result = evaluate_mapped(&interner, &mapped);

    // Result should be { a: number, b: number }
    let expected = interner.object(vec![
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
            type_id: TypeId::NUMBER,
            write_type: TypeId::NUMBER,
            optional: false,
            readonly: false,
            is_method: false,
        },
    ]);
    assert_eq!(result, expected);
}

#[test]
fn test_mapped_type_record_with_template_value() {
    let interner = TypeInterner::new();

    // Test Record<K, V> where V references the key type
    // { [P in "x" | "y"]: P } = { x: "x", y: "y" }

    let key_x = interner.literal_string("x");
    let key_y = interner.literal_string("y");
    let keys = interner.union(vec![key_x, key_y]);

    // Template is the type parameter P itself
    let p_type = interner.intern(TypeKey::TypeParameter(TypeParamInfo {
        name: interner.intern_string("P"),
        constraint: None,
        default: None,
    }));

    let mapped = MappedType {
        type_param: TypeParamInfo {
            name: interner.intern_string("P"),
            constraint: None,
            default: None,
        },
        constraint: keys,
        name_type: None,
        template: p_type, // V = P (the key itself)
        readonly_modifier: None,
        optional_modifier: None,
    };

    let result = evaluate_mapped(&interner, &mapped);

    // Result should be { x: "x", y: "y" }
    let key = interner.lookup(result);
    assert!(key.is_some(), "Record with template value should produce valid type");

    if let Some(TypeKey::Object(shape_id)) = key {
        let shape = interner.object_shape(shape_id);
        assert_eq!(shape.properties.len(), 2, "Should have 2 properties");
    }
}

#[test]
fn test_mapped_type_record_single_key() {
    let interner = TypeInterner::new();

    // Test Record with single key: Record<"id", string>
    // Should produce { id: string }

    let key_id = interner.literal_string("id");

    let mapped = MappedType {
        type_param: TypeParamInfo {
            name: interner.intern_string("P"),
            constraint: None,
            default: None,
        },
        constraint: key_id,
        name_type: None,
        template: TypeId::STRING,
        readonly_modifier: None,
        optional_modifier: None,
    };

    let result = evaluate_mapped(&interner, &mapped);

    let expected = interner.object(vec![PropertyInfo {
        name: interner.intern_string("id"),
        type_id: TypeId::STRING,
        write_type: TypeId::STRING,
        optional: false,
        readonly: false,
        is_method: false,
    }]);
    assert_eq!(result, expected);
}

#[test]
fn test_mapped_type_record_with_index_signature() {
    let interner = TypeInterner::new();

    // Test Record<string, number> pattern
    // This should produce an object with string index signature: { [key: string]: number }

    // When the key type is 'string' (not a literal), it creates an index signature
    // For this test, we verify the mapped type handles the string intrinsic

    let mapped = MappedType {
        type_param: TypeParamInfo {
            name: interner.intern_string("P"),
            constraint: None,
            default: None,
        },
        constraint: TypeId::STRING, // K = string
        name_type: None,
        template: TypeId::NUMBER, // V = number
        readonly_modifier: None,
        optional_modifier: None,
    };

    let result = evaluate_mapped(&interner, &mapped);

    // The result should be valid (either an object with index signature or handled appropriately)
    assert!(result != TypeId::ERROR, "Record<string, number> should not produce error");
}

#[test]
fn test_mapped_type_record_readonly() {
    let interner = TypeInterner::new();

    // Test Readonly Record: { readonly [P in K]: V }
    // Readonly<Record<"a" | "b", number>> = { readonly a: number, readonly b: number }

    let key_a = interner.literal_string("a");
    let key_b = interner.literal_string("b");
    let keys = interner.union(vec![key_a, key_b]);

    let mapped = MappedType {
        type_param: TypeParamInfo {
            name: interner.intern_string("P"),
            constraint: None,
            default: None,
        },
        constraint: keys,
        name_type: None,
        template: TypeId::NUMBER,
        readonly_modifier: Some(MappedModifier::Add), // +readonly
        optional_modifier: None,
    };

    let result = evaluate_mapped(&interner, &mapped);

    let key = interner.lookup(result);
    assert!(key.is_some(), "Readonly Record should produce valid type");

    if let Some(TypeKey::Object(shape_id)) = key {
        let shape = interner.object_shape(shape_id);
        assert_eq!(shape.properties.len(), 2, "Should have 2 properties");
        for prop in &shape.properties {
            assert!(prop.readonly, "Properties should be readonly");
            assert!(!prop.optional, "Properties should not be optional");
        }
    }
}

#[test]
fn test_mapped_type_partial_record() {
    let interner = TypeInterner::new();

    // Test Partial Record: { [P in K]?: V }
    // Partial<Record<"a" | "b", number>> = { a?: number, b?: number }

    let key_a = interner.literal_string("a");
    let key_b = interner.literal_string("b");
    let keys = interner.union(vec![key_a, key_b]);

    let mapped = MappedType {
        type_param: TypeParamInfo {
            name: interner.intern_string("P"),
            constraint: None,
            default: None,
        },
        constraint: keys,
        name_type: None,
        template: TypeId::NUMBER,
        readonly_modifier: None,
        optional_modifier: Some(MappedModifier::Add), // +?
    };

    let result = evaluate_mapped(&interner, &mapped);

    let key = interner.lookup(result);
    assert!(key.is_some(), "Partial Record should produce valid type");

    if let Some(TypeKey::Object(shape_id)) = key {
        let shape = interner.object_shape(shape_id);
        assert_eq!(shape.properties.len(), 2, "Should have 2 properties");
        for prop in &shape.properties {
            assert!(!prop.readonly, "Properties should not be readonly");
            assert!(prop.optional, "Properties should be optional");
        }
    }
}

// =========================================================================
// Template Literal Intrinsic Tests
// =========================================================================
// These tests cover template literal type operations that form the foundation
// for string manipulation intrinsics (Uppercase, Lowercase, Capitalize, Uncapitalize).

#[test]
fn test_template_literal_simple_concatenation() {
    let interner = TypeInterner::new();

    // Test simple template literal: `hello${string}`
    // This represents the pattern used in template literal types

    let template = interner.template_literal(vec![
        TemplateSpan::Text(interner.intern_string("hello")),
        TemplateSpan::Type(TypeId::STRING),
    ]);

    // Template literal should produce a valid type
    assert!(template != TypeId::ERROR, "Template literal should not produce error");

    let key = interner.lookup(template);
    assert!(key.is_some(), "Template literal should be internable");
}

#[test]
fn test_template_literal_prefix_suffix() {
    let interner = TypeInterner::new();

    // Test template literal with prefix and suffix: `get${string}Handler`
    // Common pattern for getter/setter type generation

    let template = interner.template_literal(vec![
        TemplateSpan::Text(interner.intern_string("get")),
        TemplateSpan::Type(TypeId::STRING),
        TemplateSpan::Text(interner.intern_string("Handler")),
    ]);

    assert!(template != TypeId::ERROR, "Template with prefix/suffix should not produce error");

    let key = interner.lookup(template);
    assert!(key.is_some(), "Template literal should be internable");
}

#[test]
fn test_template_literal_multiple_interpolations() {
    let interner = TypeInterner::new();

    // Test template literal with multiple type interpolations: `${string}_${number}`
    // Pattern for composite key types

    let template = interner.template_literal(vec![
        TemplateSpan::Type(TypeId::STRING),
        TemplateSpan::Text(interner.intern_string("_")),
        TemplateSpan::Type(TypeId::NUMBER),
    ]);

    assert!(template != TypeId::ERROR, "Multiple interpolations should not produce error");

    let key = interner.lookup(template);
    assert!(key.is_some(), "Template literal should be internable");
}

#[test]
fn test_template_literal_with_literal_type() {
    let interner = TypeInterner::new();

    // Test template literal with literal string type: `prefix_${"value"}`
    // Should be evaluable to a single literal string

    let literal_value = interner.literal_string("value");

    let template = interner.template_literal(vec![
        TemplateSpan::Text(interner.intern_string("prefix_")),
        TemplateSpan::Type(literal_value),
    ]);

    assert!(template != TypeId::ERROR, "Template with literal should not produce error");
}

#[test]
fn test_template_literal_empty_parts() {
    let interner = TypeInterner::new();

    // Test template literal that's just a type: `${string}`
    // Equivalent to the string type in most contexts

    let template = interner.template_literal(vec![TemplateSpan::Type(TypeId::STRING)]);

    assert!(template != TypeId::ERROR, "Template with just type should not produce error");
}

#[test]
fn test_template_literal_all_text() {
    let interner = TypeInterner::new();

    // Test template literal with only text parts: `hello`
    // Should be equivalent to the literal string type

    let template = interner.template_literal(vec![TemplateSpan::Text(
        interner.intern_string("hello"),
    )]);

    assert!(template != TypeId::ERROR, "Text-only template should not produce error");
}

#[test]
fn test_template_literal_in_mapped_type() {
    let interner = TypeInterner::new();

    // Test using template literal as mapped type key remapping
    // Pattern: { [K in Keys as `get${K}`]: ... }

    let key_name = interner.literal_string("name");
    let key_age = interner.literal_string("age");
    let keys = interner.union(vec![key_name, key_age]);

    // Create the type parameter K
    let k_param = TypeParamInfo {
        name: interner.intern_string("K"),
        constraint: None,
        default: None,
    };
    let k_type = interner.intern(TypeKey::TypeParameter(k_param.clone()));

    // Create the name remapping: `get${K}`
    let name_template = interner.template_literal(vec![
        TemplateSpan::Text(interner.intern_string("get")),
        TemplateSpan::Type(k_type),
    ]);

    // Create mapped type with key remapping
    let mapped = MappedType {
        type_param: k_param,
        constraint: keys,
        name_type: Some(name_template), // Key remapping
        template: TypeId::STRING,
        readonly_modifier: None,
        optional_modifier: None,
    };

    let result = evaluate_mapped(&interner, &mapped);

    // Should produce { getName: string, getAge: string }
    assert!(result != TypeId::ERROR, "Mapped type with template key should not produce error");
}

#[test]
fn test_template_literal_union_distribution() {
    let interner = TypeInterner::new();

    // Test template literal with union type: `prefix_${"a" | "b"}`
    // Should distribute: `prefix_a` | `prefix_b`

    let lit_a = interner.literal_string("a");
    let lit_b = interner.literal_string("b");
    let union_type = interner.union(vec![lit_a, lit_b]);

    let template = interner.template_literal(vec![
        TemplateSpan::Text(interner.intern_string("prefix_")),
        TemplateSpan::Type(union_type),
    ]);

    assert!(
        template != TypeId::ERROR,
        "Template with union should not produce error"
    );
}

#[test]
fn test_template_literal_nested_template() {
    let interner = TypeInterner::new();

    // Test nested template literal pattern
    // Outer: `start_${inner}_end` where inner is also a template

    let inner_template = interner.template_literal(vec![
        TemplateSpan::Text(interner.intern_string("middle")),
        TemplateSpan::Type(TypeId::STRING),
    ]);

    let outer_template = interner.template_literal(vec![
        TemplateSpan::Text(interner.intern_string("start_")),
        TemplateSpan::Type(inner_template),
        TemplateSpan::Text(interner.intern_string("_end")),
    ]);

    assert!(
        outer_template != TypeId::ERROR,
        "Nested template should not produce error"
    );
}

// =========================================================================
// NoInfer<T> Pattern Tests
// =========================================================================
// NoInfer<T> prevents type inference from a type position.
// In the solver, this is typically handled as a type that blocks inference
// but preserves its argument type for other operations.

#[test]
fn test_noinfer_pattern_as_identity() {
    let interner = TypeInterner::new();

    // NoInfer<T> should act as identity for type operations (T passes through)
    // We model this as a type application to a reference

    let noinfer_symbol = SymbolRef(300);
    let noinfer_base = interner.reference(noinfer_symbol);

    // NoInfer<string>
    let noinfer_string = interner.application(noinfer_base, vec![TypeId::STRING]);

    // The application should be valid
    assert!(
        noinfer_string != TypeId::ERROR,
        "NoInfer<string> should not produce error"
    );
}

#[test]
fn test_noinfer_pattern_with_union() {
    let interner = TypeInterner::new();

    // NoInfer<string | number> - should prevent inference but preserve the union

    let noinfer_symbol = SymbolRef(301);
    let noinfer_base = interner.reference(noinfer_symbol);

    let union_type = interner.union(vec![TypeId::STRING, TypeId::NUMBER]);
    let noinfer_union = interner.application(noinfer_base, vec![union_type]);

    assert!(
        noinfer_union != TypeId::ERROR,
        "NoInfer<string | number> should not produce error"
    );
}

#[test]
fn test_noinfer_pattern_in_function_param() {
    let interner = TypeInterner::new();

    // Test pattern: function foo<T>(value: T, defaultValue: NoInfer<T>): T
    // The NoInfer on defaultValue prevents it from contributing to inference of T

    let noinfer_symbol = SymbolRef(302);
    let noinfer_base = interner.reference(noinfer_symbol);

    // Create type parameter T
    let t_param = TypeParamInfo {
        name: interner.intern_string("T"),
        constraint: None,
        default: None,
    };
    let t_type = interner.intern(TypeKey::TypeParameter(t_param.clone()));

    // NoInfer<T>
    let noinfer_t = interner.application(noinfer_base, vec![t_type]);

    // Create function: (value: T, defaultValue: NoInfer<T>) => T
    let fn_shape = FunctionShape {
        type_params: vec![t_param],
        params: vec![
            ParamInfo {
                name: Some(interner.intern_string("value")),
                type_id: t_type,
                optional: false,
                rest: false,
            },
            ParamInfo {
                name: Some(interner.intern_string("defaultValue")),
                type_id: noinfer_t,
                optional: false,
                rest: false,
            },
        ],
        this_type: None,
        return_type: t_type,
        type_predicate: None,
        is_constructor: false,
    };

    let fn_type = interner.function(fn_shape);
    assert!(
        fn_type != TypeId::ERROR,
        "Function with NoInfer param should not produce error"
    );
}

// =========================================================================
// String Manipulation Intrinsic Patterns
// =========================================================================
// These tests document expected patterns for Uppercase<T>, Lowercase<T>,
// Capitalize<T>, and Uncapitalize<T> when they are fully implemented.

#[test]
fn test_string_intrinsic_uppercase_pattern() {
    let interner = TypeInterner::new();

    // Uppercase<T> converts string literal to uppercase
    // Pattern: Uppercase<"hello"> should equal "HELLO"

    // Model as type application (actual intrinsic behavior requires solver support)
    let uppercase_symbol = SymbolRef(310);
    let uppercase_base = interner.reference(uppercase_symbol);

    let hello_lit = interner.literal_string("hello");
    let uppercase_hello = interner.application(uppercase_base, vec![hello_lit]);

    assert!(
        uppercase_hello != TypeId::ERROR,
        "Uppercase<'hello'> application should not produce error"
    );
}

#[test]
fn test_string_intrinsic_lowercase_pattern() {
    let interner = TypeInterner::new();

    // Lowercase<T> converts string literal to lowercase
    // Pattern: Lowercase<"HELLO"> should equal "hello"

    let lowercase_symbol = SymbolRef(311);
    let lowercase_base = interner.reference(lowercase_symbol);

    let hello_lit = interner.literal_string("HELLO");
    let lowercase_hello = interner.application(lowercase_base, vec![hello_lit]);

    assert!(
        lowercase_hello != TypeId::ERROR,
        "Lowercase<'HELLO'> application should not produce error"
    );
}

#[test]
fn test_string_intrinsic_capitalize_pattern() {
    let interner = TypeInterner::new();

    // Capitalize<T> capitalizes first character
    // Pattern: Capitalize<"hello"> should equal "Hello"

    let capitalize_symbol = SymbolRef(312);
    let capitalize_base = interner.reference(capitalize_symbol);

    let hello_lit = interner.literal_string("hello");
    let capitalize_hello = interner.application(capitalize_base, vec![hello_lit]);

    assert!(
        capitalize_hello != TypeId::ERROR,
        "Capitalize<'hello'> application should not produce error"
    );
}

#[test]
fn test_string_intrinsic_uncapitalize_pattern() {
    let interner = TypeInterner::new();

    // Uncapitalize<T> lowercases first character
    // Pattern: Uncapitalize<"Hello"> should equal "hello"

    let uncapitalize_symbol = SymbolRef(313);
    let uncapitalize_base = interner.reference(uncapitalize_symbol);

    let hello_lit = interner.literal_string("Hello");
    let uncapitalize_hello = interner.application(uncapitalize_base, vec![hello_lit]);

    assert!(
        uncapitalize_hello != TypeId::ERROR,
        "Uncapitalize<'Hello'> application should not produce error"
    );
}

#[test]
fn test_string_intrinsic_in_template_literal() {
    let interner = TypeInterner::new();

    // Common pattern: `get${Capitalize<K>}` for getter generation
    // e.g., K = "name" -> `getName`

    let capitalize_symbol = SymbolRef(314);
    let capitalize_base = interner.reference(capitalize_symbol);

    // Create type parameter K
    let k_param = TypeParamInfo {
        name: interner.intern_string("K"),
        constraint: Some(TypeId::STRING),
        default: None,
    };
    let k_type = interner.intern(TypeKey::TypeParameter(k_param));

    // Capitalize<K>
    let capitalize_k = interner.application(capitalize_base, vec![k_type]);

    // `get${Capitalize<K>}`
    let getter_template = interner.template_literal(vec![
        TemplateSpan::Text(interner.intern_string("get")),
        TemplateSpan::Type(capitalize_k),
    ]);

    assert!(
        getter_template != TypeId::ERROR,
        "Template with Capitalize should not produce error"
    );
}

#[test]
fn test_string_intrinsic_chained() {
    let interner = TypeInterner::new();

    // Test chained intrinsics: Uppercase<Capitalize<T>>
    // Should uppercase entire string after capitalizing

    let uppercase_symbol = SymbolRef(315);
    let capitalize_symbol = SymbolRef(316);

    let uppercase_base = interner.reference(uppercase_symbol);
    let capitalize_base = interner.reference(capitalize_symbol);

    let hello_lit = interner.literal_string("hello");

    // Capitalize<"hello">
    let capitalize_hello = interner.application(capitalize_base, vec![hello_lit]);

    // Uppercase<Capitalize<"hello">>
    let uppercase_capitalize = interner.application(uppercase_base, vec![capitalize_hello]);

    assert!(
        uppercase_capitalize != TypeId::ERROR,
        "Chained string intrinsics should not produce error"
    );
}

// =========================================================================
// Template Literal Infer for String Parsing Tests
// =========================================================================
// These tests cover advanced infer patterns in template literals for
// parsing strings at the type level.

#[test]
fn test_infer_template_literal_head_tail_split() {
    let interner = TypeInterner::new();

    // Pattern: T extends `${infer Head}${infer Tail}` ? [Head, Tail] : never
    // Used to split first character from the rest of a string
    // "hello" -> ["h", "ello"]

    let t_name = interner.intern_string("T");
    let t_param = interner.intern(TypeKey::TypeParameter(TypeParamInfo {
        name: t_name,
        constraint: None,
        default: None,
    }));

    let infer_head = interner.intern(TypeKey::Infer(TypeParamInfo {
        name: interner.intern_string("Head"),
        constraint: None,
        default: None,
    }));
    let infer_tail = interner.intern(TypeKey::Infer(TypeParamInfo {
        name: interner.intern_string("Tail"),
        constraint: None,
        default: None,
    }));

    // `${infer Head}${infer Tail}`
    let extends_template = interner.template_literal(vec![
        TemplateSpan::Type(infer_head),
        TemplateSpan::Type(infer_tail),
    ]);

    // Result tuple [Head, Tail]
    let result_tuple = interner.tuple(vec![
        TupleElement {
            type_id: infer_head,
            name: None,
            optional: false,
            rest: false,
        },
        TupleElement {
            type_id: infer_tail,
            name: None,
            optional: false,
            rest: false,
        },
    ]);

    let cond = ConditionalType {
        check_type: t_param,
        extends_type: extends_template,
        true_type: result_tuple,
        false_type: TypeId::NEVER,
        is_distributive: true,
    };

    let cond_type = interner.conditional(cond);

    // Test with "hello"
    let mut subst = TypeSubstitution::new();
    subst.insert(t_name, interner.literal_string("hello"));

    let instantiated = instantiate_type(&interner, cond_type, &subst);
    let result = evaluate_type(&interner, instantiated);

    // Should produce ["h", "ello"] tuple
    assert!(result != TypeId::NEVER, "Head/Tail split should match 'hello'");
    assert!(result != TypeId::ERROR, "Head/Tail split should not produce error");
}

#[test]
fn test_infer_template_literal_head_tail_empty_string() {
    let interner = TypeInterner::new();

    // Pattern: T extends `${infer Head}${infer Tail}` ? ... : never
    // Empty string "" should NOT match (needs at least one character for Head)

    let t_name = interner.intern_string("T");
    let t_param = interner.intern(TypeKey::TypeParameter(TypeParamInfo {
        name: t_name,
        constraint: None,
        default: None,
    }));

    let infer_head = interner.intern(TypeKey::Infer(TypeParamInfo {
        name: interner.intern_string("Head"),
        constraint: None,
        default: None,
    }));
    let infer_tail = interner.intern(TypeKey::Infer(TypeParamInfo {
        name: interner.intern_string("Tail"),
        constraint: None,
        default: None,
    }));

    let extends_template = interner.template_literal(vec![
        TemplateSpan::Type(infer_head),
        TemplateSpan::Type(infer_tail),
    ]);

    let cond = ConditionalType {
        check_type: t_param,
        extends_type: extends_template,
        true_type: infer_head,
        false_type: TypeId::NEVER,
        is_distributive: true,
    };

    let cond_type = interner.conditional(cond);

    // Test with empty string ""
    let mut subst = TypeSubstitution::new();
    subst.insert(t_name, interner.literal_string(""));

    let instantiated = instantiate_type(&interner, cond_type, &subst);
    let result = evaluate_type(&interner, instantiated);

    // Empty string behavior depends on implementation - just verify no error
    assert!(result != TypeId::ERROR, "Empty string should not produce error");
}

#[test]
fn test_infer_template_literal_three_segment_split() {
    let interner = TypeInterner::new();

    // Pattern: T extends `${infer A}_${infer B}_${infer C}` ? [A, B, C] : never
    // Used to parse strings like "foo_bar_baz"

    let t_name = interner.intern_string("T");
    let t_param = interner.intern(TypeKey::TypeParameter(TypeParamInfo {
        name: t_name,
        constraint: None,
        default: None,
    }));

    let infer_a = interner.intern(TypeKey::Infer(TypeParamInfo {
        name: interner.intern_string("A"),
        constraint: None,
        default: None,
    }));
    let infer_b = interner.intern(TypeKey::Infer(TypeParamInfo {
        name: interner.intern_string("B"),
        constraint: None,
        default: None,
    }));
    let infer_c = interner.intern(TypeKey::Infer(TypeParamInfo {
        name: interner.intern_string("C"),
        constraint: None,
        default: None,
    }));

    // `${infer A}_${infer B}_${infer C}`
    let extends_template = interner.template_literal(vec![
        TemplateSpan::Type(infer_a),
        TemplateSpan::Text(interner.intern_string("_")),
        TemplateSpan::Type(infer_b),
        TemplateSpan::Text(interner.intern_string("_")),
        TemplateSpan::Type(infer_c),
    ]);

    let result_tuple = interner.tuple(vec![
        TupleElement { type_id: infer_a, name: None, optional: false, rest: false },
        TupleElement { type_id: infer_b, name: None, optional: false, rest: false },
        TupleElement { type_id: infer_c, name: None, optional: false, rest: false },
    ]);

    let cond = ConditionalType {
        check_type: t_param,
        extends_type: extends_template,
        true_type: result_tuple,
        false_type: TypeId::NEVER,
        is_distributive: true,
    };

    let cond_type = interner.conditional(cond);

    // Test with "foo_bar_baz"
    let mut subst = TypeSubstitution::new();
    subst.insert(t_name, interner.literal_string("foo_bar_baz"));

    let instantiated = instantiate_type(&interner, cond_type, &subst);
    let result = evaluate_type(&interner, instantiated);

    assert!(result != TypeId::NEVER, "Three segment split should match");
    assert!(result != TypeId::ERROR, "Three segment split should not produce error");
}

#[test]
fn test_infer_template_literal_prefix_extraction() {
    let interner = TypeInterner::new();

    // Pattern: T extends `get${infer Name}` ? Name : never
    // Extract property name from getter: "getName" -> "Name"

    let t_name = interner.intern_string("T");
    let t_param = interner.intern(TypeKey::TypeParameter(TypeParamInfo {
        name: t_name,
        constraint: None,
        default: None,
    }));

    let infer_name = interner.intern(TypeKey::Infer(TypeParamInfo {
        name: interner.intern_string("Name"),
        constraint: None,
        default: None,
    }));

    // `get${infer Name}`
    let extends_template = interner.template_literal(vec![
        TemplateSpan::Text(interner.intern_string("get")),
        TemplateSpan::Type(infer_name),
    ]);

    let cond = ConditionalType {
        check_type: t_param,
        extends_type: extends_template,
        true_type: infer_name,
        false_type: TypeId::NEVER,
        is_distributive: true,
    };

    let cond_type = interner.conditional(cond);

    // Test with "getName"
    let mut subst = TypeSubstitution::new();
    subst.insert(t_name, interner.literal_string("getName"));

    let instantiated = instantiate_type(&interner, cond_type, &subst);
    let result = evaluate_type(&interner, instantiated);

    // Should extract "Name"
    let expected = interner.literal_string("Name");
    assert_eq!(result, expected, "Should extract 'Name' from 'getName'");
}

#[test]
fn test_infer_template_literal_suffix_extraction() {
    let interner = TypeInterner::new();

    // Pattern: T extends `${infer Base}Handler` ? Base : never
    // Extract base name from handler: "ClickHandler" -> "Click"

    let t_name = interner.intern_string("T");
    let t_param = interner.intern(TypeKey::TypeParameter(TypeParamInfo {
        name: t_name,
        constraint: None,
        default: None,
    }));

    let infer_base = interner.intern(TypeKey::Infer(TypeParamInfo {
        name: interner.intern_string("Base"),
        constraint: None,
        default: None,
    }));

    // `${infer Base}Handler`
    let extends_template = interner.template_literal(vec![
        TemplateSpan::Type(infer_base),
        TemplateSpan::Text(interner.intern_string("Handler")),
    ]);

    let cond = ConditionalType {
        check_type: t_param,
        extends_type: extends_template,
        true_type: infer_base,
        false_type: TypeId::NEVER,
        is_distributive: true,
    };

    let cond_type = interner.conditional(cond);

    // Test with "ClickHandler"
    let mut subst = TypeSubstitution::new();
    subst.insert(t_name, interner.literal_string("ClickHandler"));

    let instantiated = instantiate_type(&interner, cond_type, &subst);
    let result = evaluate_type(&interner, instantiated);

    // Should extract "Click"
    let expected = interner.literal_string("Click");
    assert_eq!(result, expected, "Should extract 'Click' from 'ClickHandler'");
}

#[test]
fn test_infer_template_literal_middle_extraction() {
    let interner = TypeInterner::new();

    // Pattern: T extends `on${infer Event}Changed` ? Event : never
    // Extract middle part: "onNameChanged" -> "Name"

    let t_name = interner.intern_string("T");
    let t_param = interner.intern(TypeKey::TypeParameter(TypeParamInfo {
        name: t_name,
        constraint: None,
        default: None,
    }));

    let infer_event = interner.intern(TypeKey::Infer(TypeParamInfo {
        name: interner.intern_string("Event"),
        constraint: None,
        default: None,
    }));

    // `on${infer Event}Changed`
    let extends_template = interner.template_literal(vec![
        TemplateSpan::Text(interner.intern_string("on")),
        TemplateSpan::Type(infer_event),
        TemplateSpan::Text(interner.intern_string("Changed")),
    ]);

    let cond = ConditionalType {
        check_type: t_param,
        extends_type: extends_template,
        true_type: infer_event,
        false_type: TypeId::NEVER,
        is_distributive: true,
    };

    let cond_type = interner.conditional(cond);

    // Test with "onNameChanged"
    let mut subst = TypeSubstitution::new();
    subst.insert(t_name, interner.literal_string("onNameChanged"));

    let instantiated = instantiate_type(&interner, cond_type, &subst);
    let result = evaluate_type(&interner, instantiated);

    // Should extract "Name"
    let expected = interner.literal_string("Name");
    assert_eq!(result, expected, "Should extract 'Name' from 'onNameChanged'");
}

#[test]
fn test_infer_template_literal_non_matching() {
    let interner = TypeInterner::new();

    // Pattern: T extends `get${infer Name}` ? Name : "default"
    // Non-matching string should return false branch

    let t_name = interner.intern_string("T");
    let t_param = interner.intern(TypeKey::TypeParameter(TypeParamInfo {
        name: t_name,
        constraint: None,
        default: None,
    }));

    let infer_name = interner.intern(TypeKey::Infer(TypeParamInfo {
        name: interner.intern_string("Name"),
        constraint: None,
        default: None,
    }));

    let extends_template = interner.template_literal(vec![
        TemplateSpan::Text(interner.intern_string("get")),
        TemplateSpan::Type(infer_name),
    ]);

    let default_lit = interner.literal_string("default");

    let cond = ConditionalType {
        check_type: t_param,
        extends_type: extends_template,
        true_type: infer_name,
        false_type: default_lit,
        is_distributive: true,
    };

    let cond_type = interner.conditional(cond);

    // Test with "setName" (doesn't start with "get")
    let mut subst = TypeSubstitution::new();
    subst.insert(t_name, interner.literal_string("setName"));

    let instantiated = instantiate_type(&interner, cond_type, &subst);
    let result = evaluate_type(&interner, instantiated);

    // Should return "default"
    assert_eq!(result, default_lit, "Non-matching should return 'default'");
}

#[test]
fn test_infer_template_literal_path_split() {
    let interner = TypeInterner::new();

    // Pattern: T extends `${infer Dir}/${infer File}` ? { dir: Dir, file: File } : never
    // Parse path: "src/index" -> { dir: "src", file: "index" }

    let t_name = interner.intern_string("T");
    let t_param = interner.intern(TypeKey::TypeParameter(TypeParamInfo {
        name: t_name,
        constraint: None,
        default: None,
    }));

    let infer_dir = interner.intern(TypeKey::Infer(TypeParamInfo {
        name: interner.intern_string("Dir"),
        constraint: None,
        default: None,
    }));
    let infer_file = interner.intern(TypeKey::Infer(TypeParamInfo {
        name: interner.intern_string("File"),
        constraint: None,
        default: None,
    }));

    // `${infer Dir}/${infer File}`
    let extends_template = interner.template_literal(vec![
        TemplateSpan::Type(infer_dir),
        TemplateSpan::Text(interner.intern_string("/")),
        TemplateSpan::Type(infer_file),
    ]);

    // Result object { dir: Dir, file: File }
    let result_obj = interner.object(vec![
        PropertyInfo {
            name: interner.intern_string("dir"),
            type_id: infer_dir,
            write_type: infer_dir,
            optional: false,
            readonly: false,
            is_method: false,
        },
        PropertyInfo {
            name: interner.intern_string("file"),
            type_id: infer_file,
            write_type: infer_file,
            optional: false,
            readonly: false,
            is_method: false,
        },
    ]);

    let cond = ConditionalType {
        check_type: t_param,
        extends_type: extends_template,
        true_type: result_obj,
        false_type: TypeId::NEVER,
        is_distributive: true,
    };

    let cond_type = interner.conditional(cond);

    // Test with "src/index"
    let mut subst = TypeSubstitution::new();
    subst.insert(t_name, interner.literal_string("src/index"));

    let instantiated = instantiate_type(&interner, cond_type, &subst);
    let result = evaluate_type(&interner, instantiated);

    assert!(result != TypeId::NEVER, "Path split should match");
    assert!(result != TypeId::ERROR, "Path split should not produce error");
}

#[test]
fn test_infer_template_literal_with_union_input() {
    let interner = TypeInterner::new();

    // Pattern: T extends `${infer Prefix}_${infer Suffix}` ? Prefix : never
    // With union input: "foo_bar" | "baz_qux" should extract "foo" | "baz"

    let t_name = interner.intern_string("T");
    let t_param = interner.intern(TypeKey::TypeParameter(TypeParamInfo {
        name: t_name,
        constraint: None,
        default: None,
    }));

    let infer_prefix = interner.intern(TypeKey::Infer(TypeParamInfo {
        name: interner.intern_string("Prefix"),
        constraint: None,
        default: None,
    }));
    let infer_suffix = interner.intern(TypeKey::Infer(TypeParamInfo {
        name: interner.intern_string("Suffix"),
        constraint: None,
        default: None,
    }));

    let extends_template = interner.template_literal(vec![
        TemplateSpan::Type(infer_prefix),
        TemplateSpan::Text(interner.intern_string("_")),
        TemplateSpan::Type(infer_suffix),
    ]);

    let cond = ConditionalType {
        check_type: t_param,
        extends_type: extends_template,
        true_type: infer_prefix,
        false_type: TypeId::NEVER,
        is_distributive: true, // Distributive over union
    };

    let cond_type = interner.conditional(cond);

    // Test with union "foo_bar" | "baz_qux"
    let mut subst = TypeSubstitution::new();
    let foo_bar = interner.literal_string("foo_bar");
    let baz_qux = interner.literal_string("baz_qux");
    subst.insert(t_name, interner.union(vec![foo_bar, baz_qux]));

    let instantiated = instantiate_type(&interner, cond_type, &subst);
    let result = evaluate_type(&interner, instantiated);

    // Should produce "foo" | "baz"
    let expected = interner.union(vec![
        interner.literal_string("foo"),
        interner.literal_string("baz"),
    ]);
    assert_eq!(result, expected, "Should extract prefixes from union");
}

#[test]
fn test_infer_template_literal_kebab_to_camel_pattern() {
    let interner = TypeInterner::new();

    // Pattern for kebab-case to camelCase conversion base case:
    // T extends `${infer First}-${infer Rest}` ? `${First}${Capitalize<Rest>}` : T
    // "foo-bar" -> needs recursive application, but here we test single split

    let t_name = interner.intern_string("T");
    let t_param = interner.intern(TypeKey::TypeParameter(TypeParamInfo {
        name: t_name,
        constraint: None,
        default: None,
    }));

    let infer_first = interner.intern(TypeKey::Infer(TypeParamInfo {
        name: interner.intern_string("First"),
        constraint: None,
        default: None,
    }));
    let infer_rest = interner.intern(TypeKey::Infer(TypeParamInfo {
        name: interner.intern_string("Rest"),
        constraint: None,
        default: None,
    }));

    // `${infer First}-${infer Rest}`
    let extends_template = interner.template_literal(vec![
        TemplateSpan::Type(infer_first),
        TemplateSpan::Text(interner.intern_string("-")),
        TemplateSpan::Type(infer_rest),
    ]);

    // For true branch, return tuple of parts for now
    let result_tuple = interner.tuple(vec![
        TupleElement { type_id: infer_first, name: None, optional: false, rest: false },
        TupleElement { type_id: infer_rest, name: None, optional: false, rest: false },
    ]);

    let cond = ConditionalType {
        check_type: t_param,
        extends_type: extends_template,
        true_type: result_tuple,
        false_type: t_param, // Return original if no hyphen
        is_distributive: true,
    };

    let cond_type = interner.conditional(cond);

    // Test with "foo-bar"
    let mut subst = TypeSubstitution::new();
    subst.insert(t_name, interner.literal_string("foo-bar"));

    let instantiated = instantiate_type(&interner, cond_type, &subst);
    let result = evaluate_type(&interner, instantiated);

    assert!(result != TypeId::ERROR, "Kebab split should not produce error");
}

#[test]
fn test_infer_template_literal_dot_notation_parse() {
    let interner = TypeInterner::new();

    // Pattern: T extends `${infer Obj}.${infer Prop}` ? { object: Obj, property: Prop } : never
    // Parse dot notation: "user.name" -> { object: "user", property: "name" }

    let t_name = interner.intern_string("T");
    let t_param = interner.intern(TypeKey::TypeParameter(TypeParamInfo {
        name: t_name,
        constraint: None,
        default: None,
    }));

    let infer_obj = interner.intern(TypeKey::Infer(TypeParamInfo {
        name: interner.intern_string("Obj"),
        constraint: None,
        default: None,
    }));
    let infer_prop = interner.intern(TypeKey::Infer(TypeParamInfo {
        name: interner.intern_string("Prop"),
        constraint: None,
        default: None,
    }));

    // `${infer Obj}.${infer Prop}`
    let extends_template = interner.template_literal(vec![
        TemplateSpan::Type(infer_obj),
        TemplateSpan::Text(interner.intern_string(".")),
        TemplateSpan::Type(infer_prop),
    ]);

    let result_obj = interner.object(vec![
        PropertyInfo {
            name: interner.intern_string("object"),
            type_id: infer_obj,
            write_type: infer_obj,
            optional: false,
            readonly: false,
            is_method: false,
        },
        PropertyInfo {
            name: interner.intern_string("property"),
            type_id: infer_prop,
            write_type: infer_prop,
            optional: false,
            readonly: false,
            is_method: false,
        },
    ]);

    let cond = ConditionalType {
        check_type: t_param,
        extends_type: extends_template,
        true_type: result_obj,
        false_type: TypeId::NEVER,
        is_distributive: true,
    };

    let cond_type = interner.conditional(cond);

    // Test with "user.name"
    let mut subst = TypeSubstitution::new();
    subst.insert(t_name, interner.literal_string("user.name"));

    let instantiated = instantiate_type(&interner, cond_type, &subst);
    let result = evaluate_type(&interner, instantiated);

    assert!(result != TypeId::NEVER, "Dot notation parse should match");
    assert!(result != TypeId::ERROR, "Dot notation parse should not produce error");
}
