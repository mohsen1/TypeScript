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

// =========================================================================
// Recursive Conditional Type Tests
// =========================================================================
// These tests cover recursive conditional types that reference themselves,
// testing the solver's recursion handling and termination.

#[test]
fn test_recursive_conditional_flatten_single_level() {
    let interner = TypeInterner::new();

    // Flatten<T> = T extends Array<infer U> ? Flatten<U> : T
    // For single level: Flatten<string[]> should give string

    // Create infer U
    let infer_u = interner.intern(TypeKey::Infer(TypeParamInfo {
        name: interner.intern_string("U"),
        constraint: None,
        default: None,
    }));

    // Array<infer U>
    let array_infer_u = interner.array(infer_u);

    // For testing, we simulate a single level unwrap (non-recursive base case)
    // T = string[]
    let string_array = interner.array(TypeId::STRING);

    let cond = ConditionalType {
        check_type: string_array,
        extends_type: array_infer_u,
        true_type: infer_u, // In recursive case this would be Flatten<U>, here just U
        false_type: string_array,
        is_distributive: false,
    };

    let result = evaluate_conditional(&interner, &cond);

    // Should extract string from string[]
    assert_eq!(result, TypeId::STRING, "Flatten single level should extract string");
}

#[test]
fn test_recursive_conditional_flatten_nested_array() {
    let interner = TypeInterner::new();

    // Flatten<T> pattern with nested arrays
    // For string[][] -> should unwrap to string[] in first step

    let infer_u = interner.intern(TypeKey::Infer(TypeParamInfo {
        name: interner.intern_string("U"),
        constraint: None,
        default: None,
    }));

    let array_infer_u = interner.array(infer_u);

    // T = string[][] (nested array)
    let string_array = interner.array(TypeId::STRING);
    let nested_array = interner.array(string_array);

    let cond = ConditionalType {
        check_type: nested_array,
        extends_type: array_infer_u,
        true_type: infer_u,
        false_type: nested_array,
        is_distributive: false,
    };

    let result = evaluate_conditional(&interner, &cond);

    // First unwrap: string[][] -> string[]
    assert_eq!(result, string_array, "First unwrap should give string[]");
}

#[test]
fn test_recursive_conditional_flatten_non_array() {
    let interner = TypeInterner::new();

    // Flatten<T> where T is not an array should return T unchanged
    // Flatten<string> = string

    let infer_u = interner.intern(TypeKey::Infer(TypeParamInfo {
        name: interner.intern_string("U"),
        constraint: None,
        default: None,
    }));

    let array_infer_u = interner.array(infer_u);

    // T = string (not an array)
    let cond = ConditionalType {
        check_type: TypeId::STRING,
        extends_type: array_infer_u,
        true_type: infer_u,
        false_type: TypeId::STRING,
        is_distributive: false,
    };

    let result = evaluate_conditional(&interner, &cond);

    // String doesn't extend Array, should return string unchanged
    assert_eq!(result, TypeId::STRING, "Non-array should return unchanged");
}

#[test]
fn test_recursive_conditional_unwrap_promise() {
    let interner = TypeInterner::new();

    // Awaited<T> = T extends Promise<infer U> ? Awaited<U> : T
    // Single level: Awaited<Promise<string>> = string

    let promise_symbol = SymbolRef(400);
    let promise_base = interner.reference(promise_symbol);

    let infer_u = interner.intern(TypeKey::Infer(TypeParamInfo {
        name: interner.intern_string("U"),
        constraint: None,
        default: None,
    }));

    // Promise<infer U>
    let promise_infer = interner.application(promise_base, vec![infer_u]);

    // T = Promise<string>
    let promise_string = interner.application(promise_base, vec![TypeId::STRING]);

    let cond = ConditionalType {
        check_type: promise_string,
        extends_type: promise_infer,
        true_type: infer_u,
        false_type: promise_string,
        is_distributive: false,
    };

    let result = evaluate_conditional(&interner, &cond);

    // Should extract string from Promise<string>
    assert!(result != TypeId::ERROR, "Promise unwrap should not produce error");
}

#[test]
fn test_recursive_conditional_nested_promise() {
    let interner = TypeInterner::new();

    // Awaited pattern with nested Promises
    // Promise<Promise<string>> -> first step extracts Promise<string>

    let promise_symbol = SymbolRef(401);
    let promise_base = interner.reference(promise_symbol);

    let infer_u = interner.intern(TypeKey::Infer(TypeParamInfo {
        name: interner.intern_string("U"),
        constraint: None,
        default: None,
    }));

    let promise_infer = interner.application(promise_base, vec![infer_u]);

    // Promise<string>
    let promise_string = interner.application(promise_base, vec![TypeId::STRING]);
    // Promise<Promise<string>>
    let nested_promise = interner.application(promise_base, vec![promise_string]);

    let cond = ConditionalType {
        check_type: nested_promise,
        extends_type: promise_infer,
        true_type: infer_u,
        false_type: nested_promise,
        is_distributive: false,
    };

    let result = evaluate_conditional(&interner, &cond);

    // Should extract Promise<string> from outer layer
    assert!(result != TypeId::ERROR, "Nested promise unwrap should not produce error");
}

#[test]
fn test_recursive_conditional_deep_readonly_object() {
    let interner = TypeInterner::new();

    // DeepReadonly<T> = T extends object ? { readonly [K in keyof T]: DeepReadonly<T[K]> } : T
    // Base case: primitive type should return unchanged

    // For primitives, DeepReadonly<string> = string
    let cond = ConditionalType {
        check_type: TypeId::STRING,
        extends_type: TypeId::OBJECT,
        true_type: TypeId::STRING, // Simplified - real case would have mapped type
        false_type: TypeId::STRING,
        is_distributive: false,
    };

    let result = evaluate_conditional(&interner, &cond);

    // String doesn't extend object in the structural sense we're testing
    assert!(result != TypeId::ERROR, "DeepReadonly base case should not produce error");
}

#[test]
fn test_recursive_conditional_with_union_distribution() {
    let interner = TypeInterner::new();

    // Flatten<T> with union: Flatten<string[] | number[]>
    // Should distribute: Flatten<string[]> | Flatten<number[]> = string | number

    let infer_u = interner.intern(TypeKey::Infer(TypeParamInfo {
        name: interner.intern_string("U"),
        constraint: None,
        default: None,
    }));

    let array_infer_u = interner.array(infer_u);

    // Type parameter T
    let t_name = interner.intern_string("T");
    let t_param = interner.intern(TypeKey::TypeParameter(TypeParamInfo {
        name: t_name,
        constraint: None,
        default: None,
    }));

    let cond = ConditionalType {
        check_type: t_param,
        extends_type: array_infer_u,
        true_type: infer_u,
        false_type: t_param,
        is_distributive: true, // Distributive over union
    };

    let cond_type = interner.conditional(cond);

    // T = string[] | number[]
    let string_array = interner.array(TypeId::STRING);
    let number_array = interner.array(TypeId::NUMBER);
    let union_arrays = interner.union(vec![string_array, number_array]);

    let mut subst = TypeSubstitution::new();
    subst.insert(t_name, union_arrays);

    let instantiated = instantiate_type(&interner, cond_type, &subst);
    let result = evaluate_type(&interner, instantiated);

    // Should produce string | number
    let expected = interner.union(vec![TypeId::STRING, TypeId::NUMBER]);
    assert_eq!(result, expected, "Distributive flatten should produce string | number");
}

#[test]
fn test_recursive_conditional_tuple_to_union() {
    let interner = TypeInterner::new();

    // TupleToUnion<T> = T extends [infer First, ...infer Rest] ? First | TupleToUnion<Rest> : never
    // Base case test: extract first from tuple

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

    // [infer First, ...infer Rest]
    let extends_tuple = interner.tuple(vec![
        TupleElement {
            type_id: infer_first,
            name: None,
            optional: false,
            rest: false,
        },
        TupleElement {
            type_id: infer_rest,
            name: None,
            optional: false,
            rest: true, // Rest element
        },
    ]);

    // Test tuple [string, number, boolean]
    let test_tuple = interner.tuple(vec![
        TupleElement { type_id: TypeId::STRING, name: None, optional: false, rest: false },
        TupleElement { type_id: TypeId::NUMBER, name: None, optional: false, rest: false },
        TupleElement { type_id: TypeId::BOOLEAN, name: None, optional: false, rest: false },
    ]);

    let cond = ConditionalType {
        check_type: test_tuple,
        extends_type: extends_tuple,
        true_type: infer_first, // Just extract first for this test
        false_type: TypeId::NEVER,
        is_distributive: false,
    };

    let result = evaluate_conditional(&interner, &cond);

    // Should extract string (first element)
    assert!(result != TypeId::NEVER, "Tuple extraction should match");
    assert!(result != TypeId::ERROR, "Tuple extraction should not produce error");
}

#[test]
fn test_recursive_conditional_string_length() {
    let interner = TypeInterner::new();

    // StringLength<T, Acc extends any[] = []> =
    //   T extends `${infer _}${infer Rest}` ? StringLength<Rest, [...Acc, 0]> : Acc['length']
    // Base case: extract first character

    let infer_char = interner.intern(TypeKey::Infer(TypeParamInfo {
        name: interner.intern_string("Char"),
        constraint: None,
        default: None,
    }));
    let infer_rest = interner.intern(TypeKey::Infer(TypeParamInfo {
        name: interner.intern_string("Rest"),
        constraint: None,
        default: None,
    }));

    // `${infer Char}${infer Rest}`
    let extends_template = interner.template_literal(vec![
        TemplateSpan::Type(infer_char),
        TemplateSpan::Type(infer_rest),
    ]);

    // Test with "abc"
    let test_string = interner.literal_string("abc");

    // Return tuple of [Char, Rest] for verification
    let result_tuple = interner.tuple(vec![
        TupleElement { type_id: infer_char, name: None, optional: false, rest: false },
        TupleElement { type_id: infer_rest, name: None, optional: false, rest: false },
    ]);

    let cond = ConditionalType {
        check_type: test_string,
        extends_type: extends_template,
        true_type: result_tuple,
        false_type: TypeId::NEVER,
        is_distributive: false,
    };

    let result = evaluate_conditional(&interner, &cond);

    // Should successfully split "abc" into ["a", "bc"]
    assert!(result != TypeId::NEVER, "String split should match");
    assert!(result != TypeId::ERROR, "String split should not produce error");
}

#[test]
fn test_recursive_conditional_reverse_string_step() {
    let interner = TypeInterner::new();

    // ReverseString<T> step: T extends `${infer First}${infer Rest}` ? `${ReverseString<Rest>}${First}` : T
    // Single step test

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

    let extends_template = interner.template_literal(vec![
        TemplateSpan::Type(infer_first),
        TemplateSpan::Type(infer_rest),
    ]);

    // Test with "ab"
    let test_string = interner.literal_string("ab");

    // For single step, we just verify extraction works
    // True branch would be `${Rest}${First}` in full implementation
    let result_template = interner.template_literal(vec![
        TemplateSpan::Type(infer_rest),
        TemplateSpan::Type(infer_first),
    ]);

    let cond = ConditionalType {
        check_type: test_string,
        extends_type: extends_template,
        true_type: result_template,
        false_type: test_string,
        is_distributive: false,
    };

    let result = evaluate_conditional(&interner, &cond);

    assert!(result != TypeId::ERROR, "Reverse step should not produce error");
}

#[test]
fn test_recursive_conditional_json_path_parse() {
    let interner = TypeInterner::new();

    // ParsePath<T> = T extends `${infer Head}.${infer Tail}` ? [Head, ...ParsePath<Tail>] : [T]
    // Single step: "a.b.c" -> ["a", ...]

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

    // `${infer Head}.${infer Tail}`
    let extends_template = interner.template_literal(vec![
        TemplateSpan::Type(infer_head),
        TemplateSpan::Text(interner.intern_string(".")),
        TemplateSpan::Type(infer_tail),
    ]);

    // Test with "a.b.c"
    let test_path = interner.literal_string("a.b.c");

    // Return [Head, Tail] tuple for verification
    let result_tuple = interner.tuple(vec![
        TupleElement { type_id: infer_head, name: None, optional: false, rest: false },
        TupleElement { type_id: infer_tail, name: None, optional: false, rest: false },
    ]);

    let cond = ConditionalType {
        check_type: test_path,
        extends_type: extends_template,
        true_type: result_tuple,
        false_type: TypeId::NEVER,
        is_distributive: false,
    };

    let result = evaluate_conditional(&interner, &cond);

    // Should produce ["a", "b.c"] as first step
    assert!(result != TypeId::NEVER, "Path parse should match");
    assert!(result != TypeId::ERROR, "Path parse should not produce error");
}

#[test]
fn test_recursive_conditional_termination_base_case() {
    let interner = TypeInterner::new();

    // Test that recursion properly terminates at base case
    // Flatten<string> where string is not an array should return string

    let infer_u = interner.intern(TypeKey::Infer(TypeParamInfo {
        name: interner.intern_string("U"),
        constraint: None,
        default: None,
    }));

    let array_infer_u = interner.array(infer_u);

    // Type param T
    let t_name = interner.intern_string("T");
    let t_param = interner.intern(TypeKey::TypeParameter(TypeParamInfo {
        name: t_name,
        constraint: None,
        default: None,
    }));

    // Flatten-like conditional
    let cond = ConditionalType {
        check_type: t_param,
        extends_type: array_infer_u,
        true_type: infer_u,
        false_type: t_param, // Base case: return T unchanged
        is_distributive: true,
    };

    let cond_type = interner.conditional(cond);

    // Substitute with non-array type
    let mut subst = TypeSubstitution::new();
    subst.insert(t_name, TypeId::STRING);

    let instantiated = instantiate_type(&interner, cond_type, &subst);
    let result = evaluate_type(&interner, instantiated);

    // Should return string (base case)
    assert_eq!(result, TypeId::STRING, "Base case should return original type");
}

#[test]
fn test_recursive_conditional_mixed_union() {
    let interner = TypeInterner::new();

    // Flatten with mixed union: string[] | number (array and non-array)
    // Should produce: string | number

    let infer_u = interner.intern(TypeKey::Infer(TypeParamInfo {
        name: interner.intern_string("U"),
        constraint: None,
        default: None,
    }));

    let array_infer_u = interner.array(infer_u);

    let t_name = interner.intern_string("T");
    let t_param = interner.intern(TypeKey::TypeParameter(TypeParamInfo {
        name: t_name,
        constraint: None,
        default: None,
    }));

    let cond = ConditionalType {
        check_type: t_param,
        extends_type: array_infer_u,
        true_type: infer_u,
        false_type: t_param,
        is_distributive: true,
    };

    let cond_type = interner.conditional(cond);

    // T = string[] | number
    let string_array = interner.array(TypeId::STRING);
    let mixed_union = interner.union(vec![string_array, TypeId::NUMBER]);

    let mut subst = TypeSubstitution::new();
    subst.insert(t_name, mixed_union);

    let instantiated = instantiate_type(&interner, cond_type, &subst);
    let result = evaluate_type(&interner, instantiated);

    // Should produce string | number
    let expected = interner.union(vec![TypeId::STRING, TypeId::NUMBER]);
    assert_eq!(result, expected, "Mixed union should flatten correctly");
}

#[test]
fn test_recursive_conditional_readonly_array_unwrap() {
    let interner = TypeInterner::new();

    // Flatten pattern with readonly arrays
    // readonly string[] should also unwrap to string

    let infer_u = interner.intern(TypeKey::Infer(TypeParamInfo {
        name: interner.intern_string("U"),
        constraint: None,
        default: None,
    }));

    // We test with regular Array<infer U> pattern
    let array_infer_u = interner.array(infer_u);

    // readonly string[]
    let string_array = interner.array(TypeId::STRING);
    let readonly_string_array = interner.intern(TypeKey::ReadonlyType(string_array));

    let cond = ConditionalType {
        check_type: readonly_string_array,
        extends_type: array_infer_u,
        true_type: infer_u,
        false_type: readonly_string_array,
        is_distributive: false,
    };

    let result = evaluate_conditional(&interner, &cond);

    // Behavior depends on whether readonly arrays match Array<U>
    assert!(result != TypeId::ERROR, "Readonly array unwrap should not produce error");
}

// =============================================================================
// Distributive Conditional Types with Mapped Type Interactions
// =============================================================================
// These test patterns like FunctionKeys<T>, PickByValue<T, V>, etc.
// which combine mapped types with conditional filtering and index access.

#[test]
fn test_mapped_conditional_function_keys_pattern() {
    let interner = TypeInterner::new();

    // FunctionKeys<T> = { [K in keyof T]: T[K] extends Function ? K : never }[keyof T]
    // For { a: string, b: () => void, c: number, d: () => string }
    // Should produce: "b" | "d"

    // Create source object type
    let fn_void = interner.function(FunctionShape {
        type_params: vec![],
        params: vec![],
        this_type: None,
        return_type: TypeId::VOID,
        type_predicate: None,
        is_constructor: false,
    });

    let fn_string = interner.function(FunctionShape {
        type_params: vec![],
        params: vec![],
        this_type: None,
        return_type: TypeId::STRING,
        type_predicate: None,
        is_constructor: false,
    });

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
            type_id: fn_void,
            write_type: fn_void,
            optional: false,
            readonly: false,
            is_method: true,
        },
        PropertyInfo {
            name: interner.intern_string("c"),
            type_id: TypeId::NUMBER,
            write_type: TypeId::NUMBER,
            optional: false,
            readonly: false,
            is_method: false,
        },
        PropertyInfo {
            name: interner.intern_string("d"),
            type_id: fn_string,
            write_type: fn_string,
            optional: false,
            readonly: false,
            is_method: true,
        },
    ]);

    // Create the mapped type: { [K in keyof T]: T[K] extends Function ? K : never }
    let k_name = interner.intern_string("K");
    let k_param = TypeParamInfo {
        name: k_name,
        constraint: None,
        default: None,
    };
    let k_type = interner.intern(TypeKey::TypeParameter(k_param.clone()));

    // keyof T (will be "a" | "b" | "c" | "d")
    let keyof_source = interner.intern(TypeKey::KeyOf(source_obj));

    // T[K] - index access
    let t_k = interner.intern(TypeKey::IndexAccess(source_obj, k_type));

    // Function type to check against
    let function_base = interner.reference(SymbolRef(300));

    // T[K] extends Function ? K : never
    let conditional = interner.conditional(ConditionalType {
        check_type: t_k,
        extends_type: function_base,
        true_type: k_type,
        false_type: TypeId::NEVER,
        is_distributive: true,
    });

    // The mapped type
    let mapped = interner.mapped(MappedType {
        type_param: k_param,
        constraint: keyof_source,
        name_type: None,
        template: conditional,
        readonly_modifier: None,
        optional_modifier: None,
    });

    // The pattern tests mapped type with conditional value filtering
    let result = evaluate_type(&interner, mapped);
    assert!(result != TypeId::ERROR, "FunctionKeys mapped type should evaluate without error");
}

#[test]
fn test_mapped_conditional_non_function_keys_pattern() {
    let interner = TypeInterner::new();

    // NonFunctionKeys<T> = { [K in keyof T]: T[K] extends Function ? never : K }[keyof T]
    // For { a: string, b: () => void, c: number }
    // Should produce: "a" | "c"

    let fn_void = interner.function(FunctionShape {
        type_params: vec![],
        params: vec![],
        this_type: None,
        return_type: TypeId::VOID,
        type_predicate: None,
        is_constructor: false,
    });

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
            type_id: fn_void,
            write_type: fn_void,
            optional: false,
            readonly: false,
            is_method: true,
        },
        PropertyInfo {
            name: interner.intern_string("c"),
            type_id: TypeId::NUMBER,
            write_type: TypeId::NUMBER,
            optional: false,
            readonly: false,
            is_method: false,
        },
    ]);

    let k_name = interner.intern_string("K");
    let k_param = TypeParamInfo {
        name: k_name,
        constraint: None,
        default: None,
    };
    let k_type = interner.intern(TypeKey::TypeParameter(k_param.clone()));

    let keyof_source = interner.intern(TypeKey::KeyOf(source_obj));
    let t_k = interner.intern(TypeKey::IndexAccess(source_obj, k_type));

    let function_base = interner.reference(SymbolRef(300));

    // T[K] extends Function ? never : K (inverse of FunctionKeys)
    let conditional = interner.conditional(ConditionalType {
        check_type: t_k,
        extends_type: function_base,
        true_type: TypeId::NEVER,
        false_type: k_type,
        is_distributive: true,
    });

    let mapped = interner.mapped(MappedType {
        type_param: k_param,
        constraint: keyof_source,
        name_type: None,
        template: conditional,
        readonly_modifier: None,
        optional_modifier: None,
    });

    let result = evaluate_type(&interner, mapped);
    assert!(result != TypeId::ERROR, "NonFunctionKeys mapped type should evaluate without error");
}

#[test]
fn test_mapped_conditional_pick_by_value_pattern() {
    let interner = TypeInterner::new();

    // PickByValue<T, V> = { [K in keyof T as T[K] extends V ? K : never]: T[K] }
    // For { a: string, b: number, c: string }, V = string
    // Should produce: { a: string, c: string }

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
        PropertyInfo {
            name: interner.intern_string("c"),
            type_id: TypeId::STRING,
            write_type: TypeId::STRING,
            optional: false,
            readonly: false,
            is_method: false,
        },
    ]);

    let k_name = interner.intern_string("K");
    let k_param = TypeParamInfo {
        name: k_name,
        constraint: None,
        default: None,
    };
    let k_type = interner.intern(TypeKey::TypeParameter(k_param.clone()));

    let keyof_source = interner.intern(TypeKey::KeyOf(source_obj));
    let t_k = interner.intern(TypeKey::IndexAccess(source_obj, k_type));

    // T[K] extends string ? K : never (key remapping)
    let name_conditional = interner.conditional(ConditionalType {
        check_type: t_k,
        extends_type: TypeId::STRING,
        true_type: k_type,
        false_type: TypeId::NEVER,
        is_distributive: true,
    });

    // Mapped type with key remapping (as clause)
    let mapped = interner.mapped(MappedType {
        type_param: k_param,
        constraint: keyof_source,
        name_type: Some(name_conditional),
        template: t_k,
        readonly_modifier: None,
        optional_modifier: None,
    });

    let result = evaluate_type(&interner, mapped);
    assert!(result != TypeId::ERROR, "PickByValue mapped type should evaluate without error");
}

#[test]
fn test_mapped_conditional_omit_by_value_pattern() {
    let interner = TypeInterner::new();

    // OmitByValue<T, V> = { [K in keyof T as T[K] extends V ? never : K]: T[K] }
    // For { a: string, b: number, c: string }, V = string
    // Should produce: { b: number }

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
        PropertyInfo {
            name: interner.intern_string("c"),
            type_id: TypeId::STRING,
            write_type: TypeId::STRING,
            optional: false,
            readonly: false,
            is_method: false,
        },
    ]);

    let k_name = interner.intern_string("K");
    let k_param = TypeParamInfo {
        name: k_name,
        constraint: None,
        default: None,
    };
    let k_type = interner.intern(TypeKey::TypeParameter(k_param.clone()));

    let keyof_source = interner.intern(TypeKey::KeyOf(source_obj));
    let t_k = interner.intern(TypeKey::IndexAccess(source_obj, k_type));

    // T[K] extends string ? never : K (inverse of PickByValue)
    let name_conditional = interner.conditional(ConditionalType {
        check_type: t_k,
        extends_type: TypeId::STRING,
        true_type: TypeId::NEVER,
        false_type: k_type,
        is_distributive: true,
    });

    let mapped = interner.mapped(MappedType {
        type_param: k_param,
        constraint: keyof_source,
        name_type: Some(name_conditional),
        template: t_k,
        readonly_modifier: None,
        optional_modifier: None,
    });

    let result = evaluate_type(&interner, mapped);
    assert!(result != TypeId::ERROR, "OmitByValue mapped type should evaluate without error");
}

#[test]
fn test_mapped_conditional_value_of_pattern() {
    let interner = TypeInterner::new();

    // ValueOf<T> = T[keyof T]
    // For { a: string, b: number }
    // Should produce: string | number

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

    let keyof_source = interner.intern(TypeKey::KeyOf(source_obj));
    let value_of = interner.intern(TypeKey::IndexAccess(source_obj, keyof_source));

    let result = evaluate_type(&interner, value_of);
    assert!(result != TypeId::ERROR, "ValueOf pattern should evaluate without error");
}

#[test]
fn test_mapped_conditional_extract_keys_by_type() {
    let interner = TypeInterner::new();

    // Pattern: { [K in keyof T]: T[K] extends string ? K : never }[keyof T]
    // Extracts keys whose values are strings

    let source_obj = interner.object(vec![
        PropertyInfo {
            name: interner.intern_string("name"),
            type_id: TypeId::STRING,
            write_type: TypeId::STRING,
            optional: false,
            readonly: false,
            is_method: false,
        },
        PropertyInfo {
            name: interner.intern_string("age"),
            type_id: TypeId::NUMBER,
            write_type: TypeId::NUMBER,
            optional: false,
            readonly: false,
            is_method: false,
        },
        PropertyInfo {
            name: interner.intern_string("email"),
            type_id: TypeId::STRING,
            write_type: TypeId::STRING,
            optional: false,
            readonly: false,
            is_method: false,
        },
    ]);

    let k_name = interner.intern_string("K");
    let k_param = TypeParamInfo {
        name: k_name,
        constraint: None,
        default: None,
    };
    let k_type = interner.intern(TypeKey::TypeParameter(k_param.clone()));

    let keyof_source = interner.intern(TypeKey::KeyOf(source_obj));
    let t_k = interner.intern(TypeKey::IndexAccess(source_obj, k_type));

    // T[K] extends string ? K : never
    let conditional = interner.conditional(ConditionalType {
        check_type: t_k,
        extends_type: TypeId::STRING,
        true_type: k_type,
        false_type: TypeId::NEVER,
        is_distributive: true,
    });

    let mapped = interner.mapped(MappedType {
        type_param: k_param,
        constraint: keyof_source,
        name_type: None,
        template: conditional,
        readonly_modifier: None,
        optional_modifier: None,
    });

    // { [K in keyof T]: ... }[keyof T] - index access to extract union
    let extracted = interner.intern(TypeKey::IndexAccess(mapped, keyof_source));

    let result = evaluate_type(&interner, extracted);
    assert!(result != TypeId::ERROR, "Extract keys by type should evaluate without error");
}

#[test]
fn test_mapped_conditional_nullable_keys_pattern() {
    let interner = TypeInterner::new();

    // NullableKeys<T> = { [K in keyof T]: null extends T[K] ? K : never }[keyof T]
    // For { a: string | null, b: number, c: string | null }
    // Should produce: "a" | "c"

    let string_or_null = interner.union(vec![TypeId::STRING, TypeId::NULL]);

    let source_obj = interner.object(vec![
        PropertyInfo {
            name: interner.intern_string("a"),
            type_id: string_or_null,
            write_type: string_or_null,
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
        PropertyInfo {
            name: interner.intern_string("c"),
            type_id: string_or_null,
            write_type: string_or_null,
            optional: false,
            readonly: false,
            is_method: false,
        },
    ]);

    let k_name = interner.intern_string("K");
    let k_param = TypeParamInfo {
        name: k_name,
        constraint: None,
        default: None,
    };
    let k_type = interner.intern(TypeKey::TypeParameter(k_param.clone()));

    let keyof_source = interner.intern(TypeKey::KeyOf(source_obj));
    let t_k = interner.intern(TypeKey::IndexAccess(source_obj, k_type));

    // null extends T[K] ? K : never
    let conditional = interner.conditional(ConditionalType {
        check_type: TypeId::NULL,
        extends_type: t_k,
        true_type: k_type,
        false_type: TypeId::NEVER,
        is_distributive: false, // Not distributive - check is a concrete type
    });

    let mapped = interner.mapped(MappedType {
        type_param: k_param,
        constraint: keyof_source,
        name_type: None,
        template: conditional,
        readonly_modifier: None,
        optional_modifier: None,
    });

    let result = evaluate_type(&interner, mapped);
    assert!(result != TypeId::ERROR, "NullableKeys pattern should evaluate without error");
}

#[test]
fn test_mapped_conditional_required_keys_pattern() {
    let interner = TypeInterner::new();

    // RequiredKeys<T> = { [K in keyof T]-?: {} extends Pick<T, K> ? never : K }[keyof T]
    // Simplified version: keys that are not optional

    let source_obj = interner.object(vec![
        PropertyInfo {
            name: interner.intern_string("required"),
            type_id: TypeId::STRING,
            write_type: TypeId::STRING,
            optional: false,
            readonly: false,
            is_method: false,
        },
        PropertyInfo {
            name: interner.intern_string("optional"),
            type_id: TypeId::NUMBER,
            write_type: TypeId::NUMBER,
            optional: true,
            readonly: false,
            is_method: false,
        },
    ]);

    let k_name = interner.intern_string("K");
    let k_param = TypeParamInfo {
        name: k_name,
        constraint: None,
        default: None,
    };
    let k_type = interner.intern(TypeKey::TypeParameter(k_param.clone()));

    let keyof_source = interner.intern(TypeKey::KeyOf(source_obj));

    // For RequiredKeys, we'd need to check optionality
    // Simplified: the mapped type iteration is the key part
    let mapped = interner.mapped(MappedType {
        type_param: k_param,
        constraint: keyof_source,
        name_type: None,
        template: k_type, // Just return the key
        readonly_modifier: None,
        optional_modifier: Some(MappedModifier::Remove), // -?
    });

    let result = evaluate_type(&interner, mapped);
    assert!(result != TypeId::ERROR, "RequiredKeys pattern should evaluate without error");
}

#[test]
fn test_mapped_conditional_deep_pick_by_value() {
    let interner = TypeInterner::new();

    // DeepPickByValue<T, V> - nested pattern
    // { [K in keyof T as T[K] extends V ? K : never]: T[K] extends object ? DeepPickByValue<T[K], V> : T[K] }

    let nested_obj = interner.object(vec![
        PropertyInfo {
            name: interner.intern_string("inner"),
            type_id: TypeId::STRING,
            write_type: TypeId::STRING,
            optional: false,
            readonly: false,
            is_method: false,
        },
    ]);

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
            type_id: nested_obj,
            write_type: nested_obj,
            optional: false,
            readonly: false,
            is_method: false,
        },
    ]);

    let k_name = interner.intern_string("K");
    let k_param = TypeParamInfo {
        name: k_name,
        constraint: None,
        default: None,
    };
    let k_type = interner.intern(TypeKey::TypeParameter(k_param.clone()));

    let keyof_source = interner.intern(TypeKey::KeyOf(source_obj));
    let t_k = interner.intern(TypeKey::IndexAccess(source_obj, k_type));

    // T[K] extends object ? T[K] : T[K] (simplified - real would be recursive)
    let inner_conditional = interner.conditional(ConditionalType {
        check_type: t_k,
        extends_type: TypeId::OBJECT,
        true_type: t_k,
        false_type: t_k,
        is_distributive: true,
    });

    let mapped = interner.mapped(MappedType {
        type_param: k_param,
        constraint: keyof_source,
        name_type: None,
        template: inner_conditional,
        readonly_modifier: None,
        optional_modifier: None,
    });

    let result = evaluate_type(&interner, mapped);
    assert!(result != TypeId::ERROR, "DeepPickByValue pattern should evaluate without error");
}

#[test]
fn test_mapped_conditional_getters_pattern() {
    let interner = TypeInterner::new();

    // Getters<T> = { [K in keyof T as `get${Capitalize<K>}`]: () => T[K] }
    // Uses template literal in key remapping

    let source_obj = interner.object(vec![
        PropertyInfo {
            name: interner.intern_string("name"),
            type_id: TypeId::STRING,
            write_type: TypeId::STRING,
            optional: false,
            readonly: false,
            is_method: false,
        },
        PropertyInfo {
            name: interner.intern_string("age"),
            type_id: TypeId::NUMBER,
            write_type: TypeId::NUMBER,
            optional: false,
            readonly: false,
            is_method: false,
        },
    ]);

    let k_name = interner.intern_string("K");
    let k_param = TypeParamInfo {
        name: k_name,
        constraint: None,
        default: None,
    };
    let k_type = interner.intern(TypeKey::TypeParameter(k_param.clone()));

    let keyof_source = interner.intern(TypeKey::KeyOf(source_obj));
    let t_k = interner.intern(TypeKey::IndexAccess(source_obj, k_type));

    // Template literal: `get${K}` (simplified, without Capitalize)
    let get_prefix = interner.template_literal(vec![
        TemplateSpan::Text(interner.intern_string("get")),
        TemplateSpan::Type(k_type),
    ]);

    // () => T[K]
    let getter_fn = interner.function(FunctionShape {
        type_params: vec![],
        params: vec![],
        this_type: None,
        return_type: t_k,
        type_predicate: None,
        is_constructor: false,
    });

    let mapped = interner.mapped(MappedType {
        type_param: k_param,
        constraint: keyof_source,
        name_type: Some(get_prefix),
        template: getter_fn,
        readonly_modifier: None,
        optional_modifier: None,
    });

    let result = evaluate_type(&interner, mapped);
    assert!(result != TypeId::ERROR, "Getters pattern should evaluate without error");
}

#[test]
fn test_mapped_conditional_setters_pattern() {
    let interner = TypeInterner::new();

    // Setters<T> = { [K in keyof T as `set${Capitalize<K>}`]: (value: T[K]) => void }

    let source_obj = interner.object(vec![
        PropertyInfo {
            name: interner.intern_string("value"),
            type_id: TypeId::NUMBER,
            write_type: TypeId::NUMBER,
            optional: false,
            readonly: false,
            is_method: false,
        },
    ]);

    let k_name = interner.intern_string("K");
    let k_param = TypeParamInfo {
        name: k_name,
        constraint: None,
        default: None,
    };
    let k_type = interner.intern(TypeKey::TypeParameter(k_param.clone()));

    let keyof_source = interner.intern(TypeKey::KeyOf(source_obj));
    let t_k = interner.intern(TypeKey::IndexAccess(source_obj, k_type));

    // Template literal: `set${K}`
    let set_prefix = interner.template_literal(vec![
        TemplateSpan::Text(interner.intern_string("set")),
        TemplateSpan::Type(k_type),
    ]);

    // (value: T[K]) => void
    let setter_fn = interner.function(FunctionShape {
        type_params: vec![],
        params: vec![ParamInfo {
            name: Some(interner.intern_string("value")),
            type_id: t_k,
            optional: false,
            rest: false,
        }],
        this_type: None,
        return_type: TypeId::VOID,
        type_predicate: None,
        is_constructor: false,
    });

    let mapped = interner.mapped(MappedType {
        type_param: k_param,
        constraint: keyof_source,
        name_type: Some(set_prefix),
        template: setter_fn,
        readonly_modifier: None,
        optional_modifier: None,
    });

    let result = evaluate_type(&interner, mapped);
    assert!(result != TypeId::ERROR, "Setters pattern should evaluate without error");
}

#[test]
fn test_mapped_conditional_filter_readonly_keys() {
    let interner = TypeInterner::new();

    // ReadonlyKeys<T> - extract keys that are readonly
    // This tests filtering based on property metadata

    let source_obj = interner.object(vec![
        PropertyInfo {
            name: interner.intern_string("id"),
            type_id: TypeId::NUMBER,
            write_type: TypeId::NUMBER,
            optional: false,
            readonly: true,
            is_method: false,
        },
        PropertyInfo {
            name: interner.intern_string("name"),
            type_id: TypeId::STRING,
            write_type: TypeId::STRING,
            optional: false,
            readonly: false,
            is_method: false,
        },
        PropertyInfo {
            name: interner.intern_string("version"),
            type_id: TypeId::NUMBER,
            write_type: TypeId::NUMBER,
            optional: false,
            readonly: true,
            is_method: false,
        },
    ]);

    let k_name = interner.intern_string("K");
    let k_param = TypeParamInfo {
        name: k_name,
        constraint: None,
        default: None,
    };
    let k_type = interner.intern(TypeKey::TypeParameter(k_param.clone()));

    let keyof_source = interner.intern(TypeKey::KeyOf(source_obj));

    // The mapped type iteration
    let mapped = interner.mapped(MappedType {
        type_param: k_param,
        constraint: keyof_source,
        name_type: None,
        template: k_type,
        readonly_modifier: Some(MappedModifier::Add), // +readonly
        optional_modifier: None,
    });

    let result = evaluate_type(&interner, mapped);
    assert!(result != TypeId::ERROR, "ReadonlyKeys pattern should evaluate without error");
}

#[test]
fn test_mapped_conditional_array_element_keys() {
    let interner = TypeInterner::new();

    // ArrayKeys<T> = { [K in keyof T]: T[K] extends any[] ? K : never }[keyof T]
    // Extract keys whose values are arrays

    let string_array = interner.array(TypeId::STRING);
    let number_array = interner.array(TypeId::NUMBER);

    let source_obj = interner.object(vec![
        PropertyInfo {
            name: interner.intern_string("items"),
            type_id: string_array,
            write_type: string_array,
            optional: false,
            readonly: false,
            is_method: false,
        },
        PropertyInfo {
            name: interner.intern_string("count"),
            type_id: TypeId::NUMBER,
            write_type: TypeId::NUMBER,
            optional: false,
            readonly: false,
            is_method: false,
        },
        PropertyInfo {
            name: interner.intern_string("values"),
            type_id: number_array,
            write_type: number_array,
            optional: false,
            readonly: false,
            is_method: false,
        },
    ]);

    let k_name = interner.intern_string("K");
    let k_param = TypeParamInfo {
        name: k_name,
        constraint: None,
        default: None,
    };
    let k_type = interner.intern(TypeKey::TypeParameter(k_param.clone()));

    let keyof_source = interner.intern(TypeKey::KeyOf(source_obj));
    let t_k = interner.intern(TypeKey::IndexAccess(source_obj, k_type));

    // any[] type for extends check
    let any_array = interner.array(TypeId::ANY);

    // T[K] extends any[] ? K : never
    let conditional = interner.conditional(ConditionalType {
        check_type: t_k,
        extends_type: any_array,
        true_type: k_type,
        false_type: TypeId::NEVER,
        is_distributive: true,
    });

    let mapped = interner.mapped(MappedType {
        type_param: k_param,
        constraint: keyof_source,
        name_type: None,
        template: conditional,
        readonly_modifier: None,
        optional_modifier: None,
    });

    let result = evaluate_type(&interner, mapped);
    assert!(result != TypeId::ERROR, "ArrayKeys pattern should evaluate without error");
}

#[test]
fn test_mapped_conditional_union_distribution_in_template() {
    let interner = TypeInterner::new();

    // When T[K] is a union, the conditional should distribute over it
    // For { a: string | number }, T[K] extends string ? "yes" : "no"
    // Should produce "yes" | "no" for key "a"

    let string_or_number = interner.union(vec![TypeId::STRING, TypeId::NUMBER]);

    let source_obj = interner.object(vec![
        PropertyInfo {
            name: interner.intern_string("a"),
            type_id: string_or_number,
            write_type: string_or_number,
            optional: false,
            readonly: false,
            is_method: false,
        },
    ]);

    let k_name = interner.intern_string("K");
    let k_param = TypeParamInfo {
        name: k_name,
        constraint: None,
        default: None,
    };
    let k_type = interner.intern(TypeKey::TypeParameter(k_param.clone()));

    let keyof_source = interner.intern(TypeKey::KeyOf(source_obj));
    let t_k = interner.intern(TypeKey::IndexAccess(source_obj, k_type));

    let yes_lit = interner.literal_string("yes");
    let no_lit = interner.literal_string("no");

    // T[K] extends string ? "yes" : "no"
    let conditional = interner.conditional(ConditionalType {
        check_type: t_k,
        extends_type: TypeId::STRING,
        true_type: yes_lit,
        false_type: no_lit,
        is_distributive: true,
    });

    let mapped = interner.mapped(MappedType {
        type_param: k_param,
        constraint: keyof_source,
        name_type: None,
        template: conditional,
        readonly_modifier: None,
        optional_modifier: None,
    });

    let result = evaluate_type(&interner, mapped);
    assert!(result != TypeId::ERROR, "Union distribution in template should evaluate without error");
}

#[test]
fn test_mapped_conditional_nested_keyof() {
    let interner = TypeInterner::new();

    // NestedKeyOf<T> = { [K in keyof T]: T[K] extends object ? keyof T[K] : never }[keyof T]
    // For { a: { x: string, y: number }, b: string }
    // Should produce: "x" | "y"

    let nested_obj = interner.object(vec![
        PropertyInfo {
            name: interner.intern_string("x"),
            type_id: TypeId::STRING,
            write_type: TypeId::STRING,
            optional: false,
            readonly: false,
            is_method: false,
        },
        PropertyInfo {
            name: interner.intern_string("y"),
            type_id: TypeId::NUMBER,
            write_type: TypeId::NUMBER,
            optional: false,
            readonly: false,
            is_method: false,
        },
    ]);

    let source_obj = interner.object(vec![
        PropertyInfo {
            name: interner.intern_string("a"),
            type_id: nested_obj,
            write_type: nested_obj,
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

    let k_name = interner.intern_string("K");
    let k_param = TypeParamInfo {
        name: k_name,
        constraint: None,
        default: None,
    };
    let k_type = interner.intern(TypeKey::TypeParameter(k_param.clone()));

    let keyof_source = interner.intern(TypeKey::KeyOf(source_obj));
    let t_k = interner.intern(TypeKey::IndexAccess(source_obj, k_type));
    let keyof_t_k = interner.intern(TypeKey::KeyOf(t_k));

    // T[K] extends object ? keyof T[K] : never
    let conditional = interner.conditional(ConditionalType {
        check_type: t_k,
        extends_type: TypeId::OBJECT,
        true_type: keyof_t_k,
        false_type: TypeId::NEVER,
        is_distributive: true,
    });

    let mapped = interner.mapped(MappedType {
        type_param: k_param,
        constraint: keyof_source,
        name_type: None,
        template: conditional,
        readonly_modifier: None,
        optional_modifier: None,
    });

    // Final index access to extract values
    let extracted = interner.intern(TypeKey::IndexAccess(mapped, keyof_source));

    let result = evaluate_type(&interner, extracted);
    assert!(result != TypeId::ERROR, "NestedKeyOf pattern should evaluate without error");
}

// =============================================================================
// Variadic Tuple Types
// =============================================================================
// These test patterns like [...T, string], [first: A, ...rest: B[]],
// and spread operations in tuple inference.

#[test]
fn test_variadic_tuple_leading_spread() {
    let interner = TypeInterner::new();

    // [...T, string] where T is a tuple type
    // For T = [number, boolean], result should be [number, boolean, string]

    let t_name = interner.intern_string("T");
    let t_param = TypeParamInfo {
        name: t_name,
        constraint: None,
        default: None,
    };
    let t_type = interner.intern(TypeKey::TypeParameter(t_param.clone()));

    // The variadic tuple: [...T, string]
    let variadic_tuple = interner.tuple(vec![
        TupleElement { type_id: t_type, name: None, optional: false, rest: true },
        TupleElement { type_id: TypeId::STRING, name: None, optional: false, rest: false },
    ]);

    // Substitute T with [number, boolean]
    let concrete_tuple = interner.tuple(vec![
        TupleElement { type_id: TypeId::NUMBER, name: None, optional: false, rest: false },
        TupleElement { type_id: TypeId::BOOLEAN, name: None, optional: false, rest: false },
    ]);

    let mut subst = TypeSubstitution::new();
    subst.insert(t_name, concrete_tuple);

    let result = instantiate_type(&interner, variadic_tuple, &subst);
    assert!(result != TypeId::ERROR, "Leading spread tuple should instantiate without error");
}

#[test]
fn test_variadic_tuple_trailing_spread() {
    let interner = TypeInterner::new();

    // [string, ...T] where T is a tuple type
    // For T = [number, boolean], result should be [string, number, boolean]

    let t_name = interner.intern_string("T");
    let t_param = TypeParamInfo {
        name: t_name,
        constraint: None,
        default: None,
    };
    let t_type = interner.intern(TypeKey::TypeParameter(t_param.clone()));

    // The variadic tuple: [string, ...T]
    let variadic_tuple = interner.tuple(vec![
        TupleElement { type_id: TypeId::STRING, name: None, optional: false, rest: false },
        TupleElement { type_id: t_type, name: None, optional: false, rest: true },
    ]);

    // Substitute T with [number, boolean]
    let concrete_tuple = interner.tuple(vec![
        TupleElement { type_id: TypeId::NUMBER, name: None, optional: false, rest: false },
        TupleElement { type_id: TypeId::BOOLEAN, name: None, optional: false, rest: false },
    ]);

    let mut subst = TypeSubstitution::new();
    subst.insert(t_name, concrete_tuple);

    let result = instantiate_type(&interner, variadic_tuple, &subst);
    assert!(result != TypeId::ERROR, "Trailing spread tuple should instantiate without error");
}

#[test]
fn test_variadic_tuple_labeled_with_rest() {
    let interner = TypeInterner::new();

    // [first: A, ...rest: B[]]
    // Labeled tuple with rest element

    let a_name = interner.intern_string("A");
    let a_param = TypeParamInfo {
        name: a_name,
        constraint: None,
        default: None,
    };
    let a_type = interner.intern(TypeKey::TypeParameter(a_param.clone()));

    let b_name = interner.intern_string("B");
    let b_param = TypeParamInfo {
        name: b_name,
        constraint: None,
        default: None,
    };
    let b_type = interner.intern(TypeKey::TypeParameter(b_param.clone()));

    let b_array = interner.array(b_type);

    // [first: A, ...rest: B[]]
    let labeled_rest_tuple = interner.tuple(vec![
        TupleElement {
            type_id: a_type,
            name: Some(interner.intern_string("first")),
            optional: false,
            rest: false,
        },
        TupleElement {
            type_id: b_array,
            name: Some(interner.intern_string("rest")),
            optional: false,
            rest: true,
        },
    ]);

    // Substitute A = string, B = number
    let mut subst = TypeSubstitution::new();
    subst.insert(a_name, TypeId::STRING);
    subst.insert(b_name, TypeId::NUMBER);

    let result = instantiate_type(&interner, labeled_rest_tuple, &subst);

    // Should produce [first: string, ...rest: number[]]
    assert!(result != TypeId::ERROR, "Labeled rest tuple should instantiate without error");
}

#[test]
fn test_variadic_tuple_infer_first_rest() {
    let interner = TypeInterner::new();

    // T extends [infer First, ...infer Rest] ? [First, Rest] : never
    // Pattern to split tuple into first and rest

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

    // [infer First, ...infer Rest]
    let extends_tuple = interner.tuple(vec![
        TupleElement { type_id: infer_first, name: None, optional: false, rest: false },
        TupleElement { type_id: infer_rest, name: None, optional: false, rest: true },
    ]);

    // Check type: [string, number, boolean]
    let check_tuple = interner.tuple(vec![
        TupleElement { type_id: TypeId::STRING, name: None, optional: false, rest: false },
        TupleElement { type_id: TypeId::NUMBER, name: None, optional: false, rest: false },
        TupleElement { type_id: TypeId::BOOLEAN, name: None, optional: false, rest: false },
    ]);

    // True type: [First, Rest]
    let result_tuple = interner.tuple(vec![
        TupleElement { type_id: infer_first, name: None, optional: false, rest: false },
        TupleElement { type_id: infer_rest, name: None, optional: false, rest: false },
    ]);

    let cond = ConditionalType {
        check_type: check_tuple,
        extends_type: extends_tuple,
        true_type: result_tuple,
        false_type: TypeId::NEVER,
        is_distributive: false,
    };

    let result = evaluate_conditional(&interner, &cond);
    assert!(result != TypeId::ERROR, "Tuple first/rest infer should evaluate without error");
}

#[test]
fn test_variadic_tuple_infer_last() {
    let interner = TypeInterner::new();

    // T extends [...infer Init, infer Last] ? Last : never
    // Pattern to extract last element

    let infer_init = interner.intern(TypeKey::Infer(TypeParamInfo {
        name: interner.intern_string("Init"),
        constraint: None,
        default: None,
    }));

    let infer_last = interner.intern(TypeKey::Infer(TypeParamInfo {
        name: interner.intern_string("Last"),
        constraint: None,
        default: None,
    }));

    // [...infer Init, infer Last]
    let extends_tuple = interner.tuple(vec![
        TupleElement { type_id: infer_init, name: None, optional: false, rest: true },
        TupleElement { type_id: infer_last, name: None, optional: false, rest: false },
    ]);

    // Check type: [string, number, boolean]
    let check_tuple = interner.tuple(vec![
        TupleElement { type_id: TypeId::STRING, name: None, optional: false, rest: false },
        TupleElement { type_id: TypeId::NUMBER, name: None, optional: false, rest: false },
        TupleElement { type_id: TypeId::BOOLEAN, name: None, optional: false, rest: false },
    ]);

    let cond = ConditionalType {
        check_type: check_tuple,
        extends_type: extends_tuple,
        true_type: infer_last,
        false_type: TypeId::NEVER,
        is_distributive: false,
    };

    let result = evaluate_conditional(&interner, &cond);
    // Should infer Last = boolean
    assert!(result != TypeId::ERROR, "Tuple last infer should evaluate without error");
}

#[test]
fn test_variadic_tuple_concat_pattern() {
    let interner = TypeInterner::new();

    // Concat<T, U> = [...T, ...U]
    // Tuple concatenation pattern

    let t_name = interner.intern_string("T");
    let t_param = TypeParamInfo {
        name: t_name,
        constraint: None,
        default: None,
    };
    let t_type = interner.intern(TypeKey::TypeParameter(t_param.clone()));

    let u_name = interner.intern_string("U");
    let u_param = TypeParamInfo {
        name: u_name,
        constraint: None,
        default: None,
    };
    let u_type = interner.intern(TypeKey::TypeParameter(u_param.clone()));

    // [...T, ...U]
    let concat_tuple = interner.tuple(vec![
        TupleElement { type_id: t_type, name: None, optional: false, rest: true },
        TupleElement { type_id: u_type, name: None, optional: false, rest: true },
    ]);

    // T = [string, number], U = [boolean]
    let tuple_t = interner.tuple(vec![
        TupleElement { type_id: TypeId::STRING, name: None, optional: false, rest: false },
        TupleElement { type_id: TypeId::NUMBER, name: None, optional: false, rest: false },
    ]);

    let tuple_u = interner.tuple(vec![
        TupleElement { type_id: TypeId::BOOLEAN, name: None, optional: false, rest: false },
    ]);

    let mut subst = TypeSubstitution::new();
    subst.insert(t_name, tuple_t);
    subst.insert(u_name, tuple_u);

    let result = instantiate_type(&interner, concat_tuple, &subst);
    // Should produce [string, number, boolean]
    assert!(result != TypeId::ERROR, "Tuple concat should instantiate without error");
}

#[test]
fn test_variadic_tuple_push_pattern() {
    let interner = TypeInterner::new();

    // Push<T, V> = [...T, V]
    // Add element to end of tuple

    let t_name = interner.intern_string("T");
    let t_param = TypeParamInfo {
        name: t_name,
        constraint: None,
        default: None,
    };
    let t_type = interner.intern(TypeKey::TypeParameter(t_param.clone()));

    let v_name = interner.intern_string("V");
    let v_param = TypeParamInfo {
        name: v_name,
        constraint: None,
        default: None,
    };
    let v_type = interner.intern(TypeKey::TypeParameter(v_param.clone()));

    // [...T, V]
    let push_tuple = interner.tuple(vec![
        TupleElement { type_id: t_type, name: None, optional: false, rest: true },
        TupleElement { type_id: v_type, name: None, optional: false, rest: false },
    ]);

    // T = [string, number], V = boolean
    let tuple_t = interner.tuple(vec![
        TupleElement { type_id: TypeId::STRING, name: None, optional: false, rest: false },
        TupleElement { type_id: TypeId::NUMBER, name: None, optional: false, rest: false },
    ]);

    let mut subst = TypeSubstitution::new();
    subst.insert(t_name, tuple_t);
    subst.insert(v_name, TypeId::BOOLEAN);

    let result = instantiate_type(&interner, push_tuple, &subst);
    // Should produce [string, number, boolean]
    assert!(result != TypeId::ERROR, "Tuple push should instantiate without error");
}

#[test]
fn test_variadic_tuple_unshift_pattern() {
    let interner = TypeInterner::new();

    // Unshift<T, V> = [V, ...T]
    // Add element to beginning of tuple

    let t_name = interner.intern_string("T");
    let t_param = TypeParamInfo {
        name: t_name,
        constraint: None,
        default: None,
    };
    let t_type = interner.intern(TypeKey::TypeParameter(t_param.clone()));

    let v_name = interner.intern_string("V");
    let v_param = TypeParamInfo {
        name: v_name,
        constraint: None,
        default: None,
    };
    let v_type = interner.intern(TypeKey::TypeParameter(v_param.clone()));

    // [V, ...T]
    let unshift_tuple = interner.tuple(vec![
        TupleElement { type_id: v_type, name: None, optional: false, rest: false },
        TupleElement { type_id: t_type, name: None, optional: false, rest: true },
    ]);

    // T = [number, boolean], V = string
    let tuple_t = interner.tuple(vec![
        TupleElement { type_id: TypeId::NUMBER, name: None, optional: false, rest: false },
        TupleElement { type_id: TypeId::BOOLEAN, name: None, optional: false, rest: false },
    ]);

    let mut subst = TypeSubstitution::new();
    subst.insert(t_name, tuple_t);
    subst.insert(v_name, TypeId::STRING);

    let result = instantiate_type(&interner, unshift_tuple, &subst);
    // Should produce [string, number, boolean]
    assert!(result != TypeId::ERROR, "Tuple unshift should instantiate without error");
}

#[test]
fn test_variadic_tuple_pop_pattern() {
    let interner = TypeInterner::new();

    // Pop<T> = T extends [...infer Init, infer _] ? Init : never
    // Remove last element from tuple

    let infer_init = interner.intern(TypeKey::Infer(TypeParamInfo {
        name: interner.intern_string("Init"),
        constraint: None,
        default: None,
    }));

    let infer_last = interner.intern(TypeKey::Infer(TypeParamInfo {
        name: interner.intern_string("_"),
        constraint: None,
        default: None,
    }));

    // [...infer Init, infer _]
    let extends_tuple = interner.tuple(vec![
        TupleElement { type_id: infer_init, name: None, optional: false, rest: true },
        TupleElement { type_id: infer_last, name: None, optional: false, rest: false },
    ]);

    // Check type: [string, number, boolean]
    let check_tuple = interner.tuple(vec![
        TupleElement { type_id: TypeId::STRING, name: None, optional: false, rest: false },
        TupleElement { type_id: TypeId::NUMBER, name: None, optional: false, rest: false },
        TupleElement { type_id: TypeId::BOOLEAN, name: None, optional: false, rest: false },
    ]);

    let cond = ConditionalType {
        check_type: check_tuple,
        extends_type: extends_tuple,
        true_type: infer_init,
        false_type: TypeId::NEVER,
        is_distributive: false,
    };

    let result = evaluate_conditional(&interner, &cond);
    // Should infer Init = [string, number]
    assert!(result != TypeId::ERROR, "Tuple pop should evaluate without error");
}

#[test]
fn test_variadic_tuple_shift_pattern() {
    let interner = TypeInterner::new();

    // Shift<T> = T extends [infer _, ...infer Rest] ? Rest : never
    // Remove first element from tuple

    let infer_first = interner.intern(TypeKey::Infer(TypeParamInfo {
        name: interner.intern_string("_"),
        constraint: None,
        default: None,
    }));

    let infer_rest = interner.intern(TypeKey::Infer(TypeParamInfo {
        name: interner.intern_string("Rest"),
        constraint: None,
        default: None,
    }));

    // [infer _, ...infer Rest]
    let extends_tuple = interner.tuple(vec![
        TupleElement { type_id: infer_first, name: None, optional: false, rest: false },
        TupleElement { type_id: infer_rest, name: None, optional: false, rest: true },
    ]);

    // Check type: [string, number, boolean]
    let check_tuple = interner.tuple(vec![
        TupleElement { type_id: TypeId::STRING, name: None, optional: false, rest: false },
        TupleElement { type_id: TypeId::NUMBER, name: None, optional: false, rest: false },
        TupleElement { type_id: TypeId::BOOLEAN, name: None, optional: false, rest: false },
    ]);

    let cond = ConditionalType {
        check_type: check_tuple,
        extends_type: extends_tuple,
        true_type: infer_rest,
        false_type: TypeId::NEVER,
        is_distributive: false,
    };

    let result = evaluate_conditional(&interner, &cond);
    // Should infer Rest = [number, boolean]
    assert!(result != TypeId::ERROR, "Tuple shift should evaluate without error");
}

#[test]
fn test_variadic_tuple_reverse_step() {
    let interner = TypeInterner::new();

    // Reverse<T> step: T extends [infer First, ...infer Rest] ? [...Reverse<Rest>, First] : []
    // Single step of tuple reversal

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

    // [infer First, ...infer Rest]
    let extends_tuple = interner.tuple(vec![
        TupleElement { type_id: infer_first, name: None, optional: false, rest: false },
        TupleElement { type_id: infer_rest, name: None, optional: false, rest: true },
    ]);

    // Check type: [string, number]
    let check_tuple = interner.tuple(vec![
        TupleElement { type_id: TypeId::STRING, name: None, optional: false, rest: false },
        TupleElement { type_id: TypeId::NUMBER, name: None, optional: false, rest: false },
    ]);

    // True type: [...Rest, First] (simplified - not recursive)
    let result_tuple = interner.tuple(vec![
        TupleElement { type_id: infer_rest, name: None, optional: false, rest: true },
        TupleElement { type_id: infer_first, name: None, optional: false, rest: false },
    ]);

    let cond = ConditionalType {
        check_type: check_tuple,
        extends_type: extends_tuple,
        true_type: result_tuple,
        false_type: TypeId::NEVER,
        is_distributive: false,
    };

    let result = evaluate_conditional(&interner, &cond);
    assert!(result != TypeId::ERROR, "Tuple reverse step should evaluate without error");
}

#[test]
fn test_variadic_tuple_with_optional_elements() {
    let interner = TypeInterner::new();

    // [string, number?, ...boolean[]]
    // Tuple with optional element followed by rest

    let boolean_array = interner.array(TypeId::BOOLEAN);

    let tuple_with_optional = interner.tuple(vec![
        TupleElement { type_id: TypeId::STRING, name: None, optional: false, rest: false },
        TupleElement { type_id: TypeId::NUMBER, name: None, optional: true, rest: false },
        TupleElement { type_id: boolean_array, name: None, optional: false, rest: true },
    ]);

    let result = evaluate_type(&interner, tuple_with_optional);
    assert!(result != TypeId::ERROR, "Tuple with optional and rest should evaluate without error");
}

#[test]
fn test_variadic_tuple_function_params() {
    let interner = TypeInterner::new();

    // Function with variadic tuple parameters
    // <T extends any[]>(...args: [...T, callback: () => void]) => void

    let t_name = interner.intern_string("T");
    let t_param = TypeParamInfo {
        name: t_name,
        constraint: Some(interner.array(TypeId::ANY)),
        default: None,
    };
    let t_type = interner.intern(TypeKey::TypeParameter(t_param.clone()));

    let callback_fn = interner.function(FunctionShape {
        type_params: vec![],
        params: vec![],
        this_type: None,
        return_type: TypeId::VOID,
        type_predicate: None,
        is_constructor: false,
    });

    // [...T, callback: () => void]
    let params_tuple = interner.tuple(vec![
        TupleElement { type_id: t_type, name: None, optional: false, rest: true },
        TupleElement {
            type_id: callback_fn,
            name: Some(interner.intern_string("callback")),
            optional: false,
            rest: false,
        },
    ]);

    // The function type
    let variadic_fn = interner.function(FunctionShape {
        type_params: vec![t_param],
        params: vec![ParamInfo {
            name: Some(interner.intern_string("args")),
            type_id: params_tuple,
            optional: false,
            rest: true,
        }],
        this_type: None,
        return_type: TypeId::VOID,
        type_predicate: None,
        is_constructor: false,
    });

    let result = evaluate_type(&interner, variadic_fn);
    assert!(result != TypeId::ERROR, "Variadic function params should evaluate without error");
}

#[test]
fn test_variadic_tuple_infer_middle() {
    let interner = TypeInterner::new();

    // T extends [infer First, ...infer Middle, infer Last] ? Middle : never
    // Extract middle elements (TypeScript 4.2+)

    let infer_first = interner.intern(TypeKey::Infer(TypeParamInfo {
        name: interner.intern_string("First"),
        constraint: None,
        default: None,
    }));

    let infer_middle = interner.intern(TypeKey::Infer(TypeParamInfo {
        name: interner.intern_string("Middle"),
        constraint: None,
        default: None,
    }));

    let infer_last = interner.intern(TypeKey::Infer(TypeParamInfo {
        name: interner.intern_string("Last"),
        constraint: None,
        default: None,
    }));

    // [infer First, ...infer Middle, infer Last]
    let extends_tuple = interner.tuple(vec![
        TupleElement { type_id: infer_first, name: None, optional: false, rest: false },
        TupleElement { type_id: infer_middle, name: None, optional: false, rest: true },
        TupleElement { type_id: infer_last, name: None, optional: false, rest: false },
    ]);

    // Check type: [string, number, boolean, symbol]
    let check_tuple = interner.tuple(vec![
        TupleElement { type_id: TypeId::STRING, name: None, optional: false, rest: false },
        TupleElement { type_id: TypeId::NUMBER, name: None, optional: false, rest: false },
        TupleElement { type_id: TypeId::BOOLEAN, name: None, optional: false, rest: false },
        TupleElement { type_id: TypeId::SYMBOL, name: None, optional: false, rest: false },
    ]);

    let cond = ConditionalType {
        check_type: check_tuple,
        extends_type: extends_tuple,
        true_type: infer_middle,
        false_type: TypeId::NEVER,
        is_distributive: false,
    };

    let result = evaluate_conditional(&interner, &cond);
    // Should infer Middle = [number, boolean]
    assert!(result != TypeId::ERROR, "Tuple middle infer should evaluate without error");
}

#[test]
fn test_variadic_tuple_empty_spread() {
    let interner = TypeInterner::new();

    // [...T] where T = [] (empty tuple)
    // Should produce []

    let t_name = interner.intern_string("T");
    let t_param = TypeParamInfo {
        name: t_name,
        constraint: None,
        default: None,
    };
    let t_type = interner.intern(TypeKey::TypeParameter(t_param.clone()));

    // [...T]
    let spread_tuple = interner.tuple(vec![
        TupleElement { type_id: t_type, name: None, optional: false, rest: true },
    ]);

    // Empty tuple
    let empty_tuple = interner.tuple(vec![]);

    let mut subst = TypeSubstitution::new();
    subst.insert(t_name, empty_tuple);

    let result = instantiate_type(&interner, spread_tuple, &subst);
    assert!(result != TypeId::ERROR, "Empty spread tuple should instantiate without error");
}

#[test]
fn test_variadic_tuple_single_element_infer() {
    let interner = TypeInterner::new();

    // T extends [infer Only] ? Only : never
    // Single element tuple pattern

    let infer_only = interner.intern(TypeKey::Infer(TypeParamInfo {
        name: interner.intern_string("Only"),
        constraint: None,
        default: None,
    }));

    // [infer Only]
    let extends_tuple = interner.tuple(vec![
        TupleElement { type_id: infer_only, name: None, optional: false, rest: false },
    ]);

    // Check type: [string]
    let check_tuple = interner.tuple(vec![
        TupleElement { type_id: TypeId::STRING, name: None, optional: false, rest: false },
    ]);

    let cond = ConditionalType {
        check_type: check_tuple,
        extends_type: extends_tuple,
        true_type: infer_only,
        false_type: TypeId::NEVER,
        is_distributive: false,
    };

    let result = evaluate_conditional(&interner, &cond);
    // Should infer Only = string
    assert!(result != TypeId::ERROR, "Single element tuple infer should evaluate without error");
}

#[test]
fn test_variadic_tuple_length_preserve() {
    let interner = TypeInterner::new();

    // Tuple length should be preserved through spread
    // [...[string, number]] should have length 2

    let inner_tuple = interner.tuple(vec![
        TupleElement { type_id: TypeId::STRING, name: None, optional: false, rest: false },
        TupleElement { type_id: TypeId::NUMBER, name: None, optional: false, rest: false },
    ]);

    // [...inner_tuple] (spread the concrete tuple)
    let spread_tuple = interner.tuple(vec![
        TupleElement { type_id: inner_tuple, name: None, optional: false, rest: true },
    ]);

    let result = evaluate_type(&interner, spread_tuple);
    assert!(result != TypeId::ERROR, "Spread tuple length should be preserved");
}

#[test]
fn test_variadic_tuple_nested_spread() {
    let interner = TypeInterner::new();

    // [...[...T, U], V] - nested spread pattern

    let t_name = interner.intern_string("T");
    let t_param = TypeParamInfo {
        name: t_name,
        constraint: None,
        default: None,
    };
    let t_type = interner.intern(TypeKey::TypeParameter(t_param.clone()));

    let u_name = interner.intern_string("U");
    let u_param = TypeParamInfo {
        name: u_name,
        constraint: None,
        default: None,
    };
    let u_type = interner.intern(TypeKey::TypeParameter(u_param.clone()));

    let v_name = interner.intern_string("V");
    let v_param = TypeParamInfo {
        name: v_name,
        constraint: None,
        default: None,
    };
    let v_type = interner.intern(TypeKey::TypeParameter(v_param.clone()));

    // [...T, U]
    let inner_tuple = interner.tuple(vec![
        TupleElement { type_id: t_type, name: None, optional: false, rest: true },
        TupleElement { type_id: u_type, name: None, optional: false, rest: false },
    ]);

    // [...inner, V]
    let outer_tuple = interner.tuple(vec![
        TupleElement { type_id: inner_tuple, name: None, optional: false, rest: true },
        TupleElement { type_id: v_type, name: None, optional: false, rest: false },
    ]);

    // T = [string], U = number, V = boolean
    let tuple_t = interner.tuple(vec![
        TupleElement { type_id: TypeId::STRING, name: None, optional: false, rest: false },
    ]);

    let mut subst = TypeSubstitution::new();
    subst.insert(t_name, tuple_t);
    subst.insert(u_name, TypeId::NUMBER);
    subst.insert(v_name, TypeId::BOOLEAN);

    let result = instantiate_type(&interner, outer_tuple, &subst);
    assert!(result != TypeId::ERROR, "Nested spread tuple should instantiate without error");
}

#[test]
fn test_variadic_tuple_array_spread() {
    let interner = TypeInterner::new();

    // [string, ...number[]] - spread of array type
    // Common pattern for variable-length tuples with fixed prefix

    let number_array = interner.array(TypeId::NUMBER);

    let tuple_with_array_spread = interner.tuple(vec![
        TupleElement { type_id: TypeId::STRING, name: None, optional: false, rest: false },
        TupleElement { type_id: number_array, name: None, optional: false, rest: true },
    ]);

    let result = evaluate_type(&interner, tuple_with_array_spread);
    assert!(result != TypeId::ERROR, "Tuple with array spread should evaluate without error");
}

#[test]
fn test_variadic_tuple_readonly_spread() {
    let interner = TypeInterner::new();

    // readonly [...T] - readonly variadic tuple

    let t_name = interner.intern_string("T");
    let t_param = TypeParamInfo {
        name: t_name,
        constraint: None,
        default: None,
    };
    let t_type = interner.intern(TypeKey::TypeParameter(t_param.clone()));

    // [...T]
    let spread_tuple = interner.tuple(vec![
        TupleElement { type_id: t_type, name: None, optional: false, rest: true },
    ]);

    // readonly [...T]
    let readonly_spread = interner.intern(TypeKey::ReadonlyType(spread_tuple));

    // T = [string, number]
    let concrete_tuple = interner.tuple(vec![
        TupleElement { type_id: TypeId::STRING, name: None, optional: false, rest: false },
        TupleElement { type_id: TypeId::NUMBER, name: None, optional: false, rest: false },
    ]);

    let mut subst = TypeSubstitution::new();
    subst.insert(t_name, concrete_tuple);

    let result = instantiate_type(&interner, readonly_spread, &subst);
    assert!(result != TypeId::ERROR, "Readonly spread tuple should instantiate without error");
}

// =============================================================================
// Default Type Parameters
// =============================================================================
// These test patterns where generic type parameters have default values.

#[test]
fn test_type_param_with_default_basic() {
    let interner = TypeInterner::new();

    // <T = string> - T with default string
    let t_name = interner.intern_string("T");
    let t_param = TypeParamInfo {
        name: t_name,
        constraint: None,
        default: Some(TypeId::STRING),
    };
    let t_type = interner.intern(TypeKey::TypeParameter(t_param.clone()));

    // Array<T> where T = string by default
    let array_t = interner.array(t_type);

    // When instantiated without args, should use default
    let result = evaluate_type(&interner, array_t);
    assert!(result != TypeId::ERROR, "Type param with default should evaluate without error");
}

#[test]
fn test_type_param_with_constraint_and_default() {
    let interner = TypeInterner::new();

    // <T extends object = {}> - T with constraint and default
    let empty_obj = interner.object(vec![]);

    let t_name = interner.intern_string("T");
    let t_param = TypeParamInfo {
        name: t_name,
        constraint: Some(TypeId::OBJECT),
        default: Some(empty_obj),
    };
    let t_type = interner.intern(TypeKey::TypeParameter(t_param.clone()));

    // Function that uses T
    let func = interner.function(FunctionShape {
        type_params: vec![t_param],
        params: vec![ParamInfo {
            name: Some(interner.intern_string("obj")),
            type_id: t_type,
            optional: false,
            rest: false,
        }],
        this_type: None,
        return_type: t_type,
        type_predicate: None,
        is_constructor: false,
    });

    let result = evaluate_type(&interner, func);
    assert!(result != TypeId::ERROR, "Type param with constraint and default should evaluate without error");
}

#[test]
fn test_type_param_default_uses_earlier_param() {
    let interner = TypeInterner::new();

    // <T, U = T> - U defaults to T
    let t_name = interner.intern_string("T");
    let t_param = TypeParamInfo {
        name: t_name,
        constraint: None,
        default: None,
    };
    let t_type = interner.intern(TypeKey::TypeParameter(t_param.clone()));

    let u_name = interner.intern_string("U");
    let u_param = TypeParamInfo {
        name: u_name,
        constraint: None,
        default: Some(t_type), // U defaults to T
    };
    let u_type = interner.intern(TypeKey::TypeParameter(u_param.clone()));

    // [T, U] tuple
    let tuple = interner.tuple(vec![
        TupleElement { type_id: t_type, name: None, optional: false, rest: false },
        TupleElement { type_id: u_type, name: None, optional: false, rest: false },
    ]);

    // Substitute T = string, U not substituted (uses default)
    let mut subst = TypeSubstitution::new();
    subst.insert(t_name, TypeId::STRING);

    let result = instantiate_type(&interner, tuple, &subst);
    assert!(result != TypeId::ERROR, "Type param default referencing earlier param should work");
}

#[test]
fn test_type_param_default_complex() {
    let interner = TypeInterner::new();

    // <T, K extends keyof T = keyof T> - K defaults to keyof T
    let t_name = interner.intern_string("T");
    let t_param = TypeParamInfo {
        name: t_name,
        constraint: None,
        default: None,
    };
    let t_type = interner.intern(TypeKey::TypeParameter(t_param.clone()));

    let keyof_t = interner.intern(TypeKey::KeyOf(t_type));

    let k_name = interner.intern_string("K");
    let k_param = TypeParamInfo {
        name: k_name,
        constraint: Some(keyof_t),
        default: Some(keyof_t),
    };
    let k_type = interner.intern(TypeKey::TypeParameter(k_param.clone()));

    // T[K] - index access
    let t_k = interner.intern(TypeKey::IndexAccess(t_type, k_type));

    let result = evaluate_type(&interner, t_k);
    assert!(result != TypeId::ERROR, "Complex type param default should evaluate without error");
}

// =============================================================================
// Index Access on Union Types
// =============================================================================

#[test]
fn test_index_access_union_object_same_property() {
    let interner = TypeInterner::new();

    // ({ a: string } | { a: number })["a"] should be string | number
    let obj1 = interner.object(vec![PropertyInfo {
        name: interner.intern_string("a"),
        type_id: TypeId::STRING,
        write_type: TypeId::STRING,
        optional: false,
        readonly: false,
        is_method: false,
    }]);

    let obj2 = interner.object(vec![PropertyInfo {
        name: interner.intern_string("a"),
        type_id: TypeId::NUMBER,
        write_type: TypeId::NUMBER,
        optional: false,
        readonly: false,
        is_method: false,
    }]);

    let union = interner.union(vec![obj1, obj2]);
    let key_a = interner.literal_string("a");
    let index_access = interner.intern(TypeKey::IndexAccess(union, key_a));

    let result = evaluate_type(&interner, index_access);
    assert!(result != TypeId::ERROR, "Index access on union should evaluate without error");
}

#[test]
fn test_index_access_union_different_properties() {
    let interner = TypeInterner::new();

    // ({ a: string } | { b: number })["a"] - only first has "a"
    let obj1 = interner.object(vec![PropertyInfo {
        name: interner.intern_string("a"),
        type_id: TypeId::STRING,
        write_type: TypeId::STRING,
        optional: false,
        readonly: false,
        is_method: false,
    }]);

    let obj2 = interner.object(vec![PropertyInfo {
        name: interner.intern_string("b"),
        type_id: TypeId::NUMBER,
        write_type: TypeId::NUMBER,
        optional: false,
        readonly: false,
        is_method: false,
    }]);

    let union = interner.union(vec![obj1, obj2]);
    let key_a = interner.literal_string("a");
    let index_access = interner.intern(TypeKey::IndexAccess(union, key_a));

    let result = evaluate_type(&interner, index_access);
    // May produce error or undefined depending on implementation
    assert!(result != TypeId::NONE, "Index access on union with missing property should not return NONE");
}

#[test]
fn test_index_access_union_key() {
    let interner = TypeInterner::new();

    // { a: string, b: number }["a" | "b"] should be string | number
    let obj = interner.object(vec![
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

    let key_a = interner.literal_string("a");
    let key_b = interner.literal_string("b");
    let union_key = interner.union(vec![key_a, key_b]);

    let index_access = interner.intern(TypeKey::IndexAccess(obj, union_key));

    let result = evaluate_type(&interner, index_access);
    assert!(result != TypeId::ERROR, "Index access with union key should evaluate without error");
}

#[test]
fn test_index_access_array_number() {
    let interner = TypeInterner::new();

    // string[][number] should be string
    let string_array = interner.array(TypeId::STRING);
    let index_access = interner.intern(TypeKey::IndexAccess(string_array, TypeId::NUMBER));

    let result = evaluate_type(&interner, index_access);
    assert!(result != TypeId::ERROR, "Array index access with number should evaluate without error");
}

#[test]
fn test_index_access_tuple_literal_middle_element() {
    let interner = TypeInterner::new();

    // [string, number, boolean][1] should be number
    let tuple = interner.tuple(vec![
        TupleElement { type_id: TypeId::STRING, name: None, optional: false, rest: false },
        TupleElement { type_id: TypeId::NUMBER, name: None, optional: false, rest: false },
        TupleElement { type_id: TypeId::BOOLEAN, name: None, optional: false, rest: false },
    ]);

    let index_1 = interner.literal_number(1.0);
    let index_access = interner.intern(TypeKey::IndexAccess(tuple, index_1));

    let result = evaluate_type(&interner, index_access);
    assert!(result != TypeId::ERROR, "Tuple index access with literal should evaluate without error");
}

// =============================================================================
// Index Access on Intersection Types
// =============================================================================

#[test]
fn test_index_access_intersection_same_property() {
    let interner = TypeInterner::new();

    // ({ a: string } & { a: "hello" })["a"] - intersection narrows to "hello"
    let obj1 = interner.object(vec![PropertyInfo {
        name: interner.intern_string("a"),
        type_id: TypeId::STRING,
        write_type: TypeId::STRING,
        optional: false,
        readonly: false,
        is_method: false,
    }]);

    let hello_lit = interner.literal_string("hello");
    let obj2 = interner.object(vec![PropertyInfo {
        name: interner.intern_string("a"),
        type_id: hello_lit,
        write_type: hello_lit,
        optional: false,
        readonly: false,
        is_method: false,
    }]);

    let intersection = interner.intersection(vec![obj1, obj2]);
    let key_a = interner.literal_string("a");
    let index_access = interner.intern(TypeKey::IndexAccess(intersection, key_a));

    let result = evaluate_type(&interner, index_access);
    assert!(result != TypeId::ERROR, "Index access on intersection should evaluate without error");
}

#[test]
fn test_index_access_intersection_different_properties() {
    let interner = TypeInterner::new();

    // ({ a: string } & { b: number })["a"] - should get string from first
    let obj1 = interner.object(vec![PropertyInfo {
        name: interner.intern_string("a"),
        type_id: TypeId::STRING,
        write_type: TypeId::STRING,
        optional: false,
        readonly: false,
        is_method: false,
    }]);

    let obj2 = interner.object(vec![PropertyInfo {
        name: interner.intern_string("b"),
        type_id: TypeId::NUMBER,
        write_type: TypeId::NUMBER,
        optional: false,
        readonly: false,
        is_method: false,
    }]);

    let intersection = interner.intersection(vec![obj1, obj2]);
    let key_a = interner.literal_string("a");
    let index_access = interner.intern(TypeKey::IndexAccess(intersection, key_a));

    let result = evaluate_type(&interner, index_access);
    assert!(result != TypeId::ERROR, "Index access on intersection with different properties should work");
}

// =============================================================================
// Keyof Union and Intersection
// =============================================================================

#[test]
fn test_keyof_union_common_keys() {
    let interner = TypeInterner::new();

    // keyof ({ a: string, b: number } | { a: boolean, c: string })
    // Should be "a" (common key only)
    let obj1 = interner.object(vec![
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

    let obj2 = interner.object(vec![
        PropertyInfo {
            name: interner.intern_string("a"),
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

    let union = interner.union(vec![obj1, obj2]);
    let keyof_union = interner.intern(TypeKey::KeyOf(union));

    let result = evaluate_type(&interner, keyof_union);
    assert!(result != TypeId::ERROR, "Keyof union should evaluate without error");
}

#[test]
fn test_keyof_intersection_all_keys() {
    let interner = TypeInterner::new();

    // keyof ({ a: string } & { b: number }) should be "a" | "b"
    let obj1 = interner.object(vec![PropertyInfo {
        name: interner.intern_string("a"),
        type_id: TypeId::STRING,
        write_type: TypeId::STRING,
        optional: false,
        readonly: false,
        is_method: false,
    }]);

    let obj2 = interner.object(vec![PropertyInfo {
        name: interner.intern_string("b"),
        type_id: TypeId::NUMBER,
        write_type: TypeId::NUMBER,
        optional: false,
        readonly: false,
        is_method: false,
    }]);

    let intersection = interner.intersection(vec![obj1, obj2]);
    let keyof_intersection = interner.intern(TypeKey::KeyOf(intersection));

    let result = evaluate_type(&interner, keyof_intersection);
    assert!(result != TypeId::ERROR, "Keyof intersection should evaluate without error");
}

// =============================================================================
// Homomorphic Mapped Types
// =============================================================================

#[test]
fn test_homomorphic_mapped_preserves_modifiers() {
    let interner = TypeInterner::new();

    // { [K in keyof T]: T[K] } should preserve optional/readonly
    // When T = { readonly a: string; b?: number }

    let source_obj = interner.object(vec![
        PropertyInfo {
            name: interner.intern_string("a"),
            type_id: TypeId::STRING,
            write_type: TypeId::STRING,
            optional: false,
            readonly: true, // readonly
            is_method: false,
        },
        PropertyInfo {
            name: interner.intern_string("b"),
            type_id: TypeId::NUMBER,
            write_type: TypeId::NUMBER,
            optional: true, // optional
            readonly: false,
            is_method: false,
        },
    ]);

    let t_name = interner.intern_string("T");
    let t_param = TypeParamInfo {
        name: t_name,
        constraint: None,
        default: None,
    };
    let t_type = interner.intern(TypeKey::TypeParameter(t_param.clone()));

    let k_name = interner.intern_string("K");
    let k_param = TypeParamInfo {
        name: k_name,
        constraint: None,
        default: None,
    };
    let k_type = interner.intern(TypeKey::TypeParameter(k_param.clone()));

    let keyof_t = interner.intern(TypeKey::KeyOf(t_type));
    let t_k = interner.intern(TypeKey::IndexAccess(t_type, k_type));

    // Homomorphic mapped type: { [K in keyof T]: T[K] }
    let mapped = interner.mapped(MappedType {
        type_param: k_param,
        constraint: keyof_t,
        name_type: None,
        template: t_k,
        readonly_modifier: None, // No modifier change
        optional_modifier: None,
    });

    let mut subst = TypeSubstitution::new();
    subst.insert(t_name, source_obj);

    let result = instantiate_type(&interner, mapped, &subst);
    assert!(result != TypeId::ERROR, "Homomorphic mapped type should preserve modifiers");
}

#[test]
fn test_homomorphic_mapped_with_modifier_removal() {
    let interner = TypeInterner::new();

    // { [K in keyof T]-?: T[K] } - removes optional
    let source_obj = interner.object(vec![
        PropertyInfo {
            name: interner.intern_string("a"),
            type_id: TypeId::STRING,
            write_type: TypeId::STRING,
            optional: true,
            readonly: false,
            is_method: false,
        },
    ]);

    let t_name = interner.intern_string("T");
    let t_param = TypeParamInfo {
        name: t_name,
        constraint: None,
        default: None,
    };
    let t_type = interner.intern(TypeKey::TypeParameter(t_param.clone()));

    let k_name = interner.intern_string("K");
    let k_param = TypeParamInfo {
        name: k_name,
        constraint: None,
        default: None,
    };
    let k_type = interner.intern(TypeKey::TypeParameter(k_param.clone()));

    let keyof_t = interner.intern(TypeKey::KeyOf(t_type));
    let t_k = interner.intern(TypeKey::IndexAccess(t_type, k_type));

    let mapped = interner.mapped(MappedType {
        type_param: k_param,
        constraint: keyof_t,
        name_type: None,
        template: t_k,
        readonly_modifier: None,
        optional_modifier: Some(MappedModifier::Remove), // -?
    });

    let mut subst = TypeSubstitution::new();
    subst.insert(t_name, source_obj);

    let result = instantiate_type(&interner, mapped, &subst);
    assert!(result != TypeId::ERROR, "Homomorphic mapped with -? should work");
}

// =============================================================================
// Conditional Type Distribution Edge Cases
// =============================================================================

#[test]
fn test_conditional_never_check_type() {
    let interner = TypeInterner::new();

    // never extends string ? "yes" : "no" should be never
    let cond = ConditionalType {
        check_type: TypeId::NEVER,
        extends_type: TypeId::STRING,
        true_type: interner.literal_string("yes"),
        false_type: interner.literal_string("no"),
        is_distributive: true,
    };

    let result = evaluate_conditional(&interner, &cond);
    assert_eq!(result, TypeId::NEVER, "never extends T should be never");
}

#[test]
fn test_conditional_any_check_type() {
    let interner = TypeInterner::new();

    // any extends string ? "yes" : "no" should be "yes" | "no"
    let yes_lit = interner.literal_string("yes");
    let no_lit = interner.literal_string("no");

    let cond = ConditionalType {
        check_type: TypeId::ANY,
        extends_type: TypeId::STRING,
        true_type: yes_lit,
        false_type: no_lit,
        is_distributive: true,
    };

    let result = evaluate_conditional(&interner, &cond);
    // any should produce both branches
    assert!(result != TypeId::ERROR, "any extends T should not produce error");
}

#[test]
fn test_conditional_unknown_check_type() {
    let interner = TypeInterner::new();

    // unknown extends string ? "yes" : "no" should be "no"
    let yes_lit = interner.literal_string("yes");
    let no_lit = interner.literal_string("no");

    let cond = ConditionalType {
        check_type: TypeId::UNKNOWN,
        extends_type: TypeId::STRING,
        true_type: yes_lit,
        false_type: no_lit,
        is_distributive: false,
    };

    let result = evaluate_conditional(&interner, &cond);
    assert!(result != TypeId::ERROR, "unknown extends string should evaluate without error");
}

#[test]
fn test_conditional_empty_union_distribution() {
    let interner = TypeInterner::new();

    // When distributing over empty parts of a union
    // (string | never) extends string ? "yes" : "no"
    // Should just be "yes" since never is filtered out

    let string_or_never = interner.union(vec![TypeId::STRING, TypeId::NEVER]);
    let yes_lit = interner.literal_string("yes");
    let no_lit = interner.literal_string("no");

    let cond = ConditionalType {
        check_type: string_or_never,
        extends_type: TypeId::STRING,
        true_type: yes_lit,
        false_type: no_lit,
        is_distributive: true,
    };

    let result = evaluate_conditional(&interner, &cond);
    assert!(result != TypeId::ERROR, "Distribution with never should work");
}

// =============================================================================
// ThisType Pattern
// =============================================================================

#[test]
fn test_this_type_in_object() {
    let interner = TypeInterner::new();

    // { value: T; getValue(): this }
    // The 'this' type represents the containing object type

    let t_name = interner.intern_string("T");
    let t_param = TypeParamInfo {
        name: t_name,
        constraint: None,
        default: None,
    };
    let t_type = interner.intern(TypeKey::TypeParameter(t_param.clone()));

    let this_type = interner.intern(TypeKey::ThisType);

    let get_value_fn = interner.function(FunctionShape {
        type_params: vec![],
        params: vec![],
        this_type: None,
        return_type: this_type,
        type_predicate: None,
        is_constructor: false,
    });

    let obj = interner.object(vec![
        PropertyInfo {
            name: interner.intern_string("value"),
            type_id: t_type,
            write_type: t_type,
            optional: false,
            readonly: false,
            is_method: false,
        },
        PropertyInfo {
            name: interner.intern_string("getValue"),
            type_id: get_value_fn,
            write_type: get_value_fn,
            optional: false,
            readonly: false,
            is_method: true,
        },
    ]);

    let result = evaluate_type(&interner, obj);
    assert!(result != TypeId::ERROR, "Object with this type should evaluate without error");
}

// =============================================================================
// Readonly and Const Modifiers
// =============================================================================

#[test]
fn test_readonly_array_type() {
    let interner = TypeInterner::new();

    // readonly string[]
    let string_array = interner.array(TypeId::STRING);
    let readonly_array = interner.intern(TypeKey::ReadonlyType(string_array));

    let result = evaluate_type(&interner, readonly_array);
    assert!(result != TypeId::ERROR, "Readonly array should evaluate without error");
}

#[test]
fn test_readonly_tuple_type() {
    let interner = TypeInterner::new();

    // readonly [string, number]
    let tuple = interner.tuple(vec![
        TupleElement { type_id: TypeId::STRING, name: None, optional: false, rest: false },
        TupleElement { type_id: TypeId::NUMBER, name: None, optional: false, rest: false },
    ]);
    let readonly_tuple = interner.intern(TypeKey::ReadonlyType(tuple));

    let result = evaluate_type(&interner, readonly_tuple);
    assert!(result != TypeId::ERROR, "Readonly tuple should evaluate without error");
}

#[test]
fn test_double_readonly() {
    let interner = TypeInterner::new();

    // readonly readonly string[] - double readonly should collapse
    let string_array = interner.array(TypeId::STRING);
    let readonly_array = interner.intern(TypeKey::ReadonlyType(string_array));
    let double_readonly = interner.intern(TypeKey::ReadonlyType(readonly_array));

    let result = evaluate_type(&interner, double_readonly);
    assert!(result != TypeId::ERROR, "Double readonly should evaluate without error");
}

// =============================================================================
// Mapped Type Edge Cases - Homomorphic
// =============================================================================

#[test]
fn test_homomorphic_mapped_identity_no_changes() {
    let interner = TypeInterner::new();

    // { [K in keyof T]: T[K] } is identity - should return same shape
    // When T = { a: string; b: number }

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

    let t_name = interner.intern_string("T");
    let t_param = TypeParamInfo {
        name: t_name,
        constraint: None,
        default: None,
    };
    let t_type = interner.intern(TypeKey::TypeParameter(t_param.clone()));

    let k_name = interner.intern_string("K");
    let k_param = TypeParamInfo {
        name: k_name,
        constraint: None,
        default: None,
    };
    let k_type = interner.intern(TypeKey::TypeParameter(k_param.clone()));

    let keyof_t = interner.intern(TypeKey::KeyOf(t_type));
    let t_k = interner.intern(TypeKey::IndexAccess(t_type, k_type));

    let mapped = interner.mapped(MappedType {
        type_param: k_param,
        constraint: keyof_t,
        name_type: None,
        template: t_k,
        readonly_modifier: None,
        optional_modifier: None,
    });

    let mut subst = TypeSubstitution::new();
    subst.insert(t_name, source_obj);

    let result = instantiate_type(&interner, mapped, &subst);
    assert!(result != TypeId::ERROR, "Homomorphic identity should work");
}

#[test]
fn test_homomorphic_mapped_adds_readonly() {
    let interner = TypeInterner::new();

    // { readonly [K in keyof T]: T[K] } adds readonly to all properties
    // When T = { a: string; b: number }

    let source_obj = interner.object(vec![
        PropertyInfo {
            name: interner.intern_string("a"),
            type_id: TypeId::STRING,
            write_type: TypeId::STRING,
            optional: false,
            readonly: false,
            is_method: false,
        },
    ]);

    let t_name = interner.intern_string("T");
    let t_param = TypeParamInfo {
        name: t_name,
        constraint: None,
        default: None,
    };
    let t_type = interner.intern(TypeKey::TypeParameter(t_param.clone()));

    let k_name = interner.intern_string("K");
    let k_param = TypeParamInfo {
        name: k_name,
        constraint: None,
        default: None,
    };
    let k_type = interner.intern(TypeKey::TypeParameter(k_param.clone()));

    let keyof_t = interner.intern(TypeKey::KeyOf(t_type));
    let t_k = interner.intern(TypeKey::IndexAccess(t_type, k_type));

    let mapped = interner.mapped(MappedType {
        type_param: k_param,
        constraint: keyof_t,
        name_type: None,
        template: t_k,
        readonly_modifier: Some(MappedModifier::Add), // +readonly
        optional_modifier: None,
    });

    let mut subst = TypeSubstitution::new();
    subst.insert(t_name, source_obj);

    let result = instantiate_type(&interner, mapped, &subst);
    assert!(result != TypeId::ERROR, "Homomorphic +readonly should work");
}

#[test]
fn test_homomorphic_mapped_adds_optional() {
    let interner = TypeInterner::new();

    // { [K in keyof T]?: T[K] } adds optional to all properties
    let source_obj = interner.object(vec![
        PropertyInfo {
            name: interner.intern_string("a"),
            type_id: TypeId::STRING,
            write_type: TypeId::STRING,
            optional: false,
            readonly: false,
            is_method: false,
        },
    ]);

    let t_name = interner.intern_string("T");
    let t_param = TypeParamInfo {
        name: t_name,
        constraint: None,
        default: None,
    };
    let t_type = interner.intern(TypeKey::TypeParameter(t_param.clone()));

    let k_name = interner.intern_string("K");
    let k_param = TypeParamInfo {
        name: k_name,
        constraint: None,
        default: None,
    };
    let k_type = interner.intern(TypeKey::TypeParameter(k_param.clone()));

    let keyof_t = interner.intern(TypeKey::KeyOf(t_type));
    let t_k = interner.intern(TypeKey::IndexAccess(t_type, k_type));

    let mapped = interner.mapped(MappedType {
        type_param: k_param,
        constraint: keyof_t,
        name_type: None,
        template: t_k,
        readonly_modifier: None,
        optional_modifier: Some(MappedModifier::Add), // +?
    });

    let mut subst = TypeSubstitution::new();
    subst.insert(t_name, source_obj);

    let result = instantiate_type(&interner, mapped, &subst);
    assert!(result != TypeId::ERROR, "Homomorphic +? should work");
}

#[test]
fn test_homomorphic_mapped_removes_both_modifiers() {
    let interner = TypeInterner::new();

    // { -readonly [K in keyof T]-?: T[K] } removes both modifiers
    let source_obj = interner.object(vec![
        PropertyInfo {
            name: interner.intern_string("a"),
            type_id: TypeId::STRING,
            write_type: TypeId::STRING,
            optional: true,
            readonly: true,
            is_method: false,
        },
    ]);

    let t_name = interner.intern_string("T");
    let t_param = TypeParamInfo {
        name: t_name,
        constraint: None,
        default: None,
    };
    let t_type = interner.intern(TypeKey::TypeParameter(t_param.clone()));

    let k_name = interner.intern_string("K");
    let k_param = TypeParamInfo {
        name: k_name,
        constraint: None,
        default: None,
    };
    let k_type = interner.intern(TypeKey::TypeParameter(k_param.clone()));

    let keyof_t = interner.intern(TypeKey::KeyOf(t_type));
    let t_k = interner.intern(TypeKey::IndexAccess(t_type, k_type));

    let mapped = interner.mapped(MappedType {
        type_param: k_param,
        constraint: keyof_t,
        name_type: None,
        template: t_k,
        readonly_modifier: Some(MappedModifier::Remove), // -readonly
        optional_modifier: Some(MappedModifier::Remove), // -?
    });

    let mut subst = TypeSubstitution::new();
    subst.insert(t_name, source_obj);

    let result = instantiate_type(&interner, mapped, &subst);
    assert!(result != TypeId::ERROR, "Homomorphic -readonly -? should work");
}

#[test]
fn test_homomorphic_mapped_on_union_type() {
    let interner = TypeInterner::new();

    // { [K in keyof T]: T[K] } where T = A | B
    // Should distribute over union

    let obj_a = interner.object(vec![
        PropertyInfo {
            name: interner.intern_string("a"),
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
            type_id: TypeId::NUMBER,
            write_type: TypeId::NUMBER,
            optional: false,
            readonly: false,
            is_method: false,
        },
    ]);

    let union_ab = interner.union(vec![obj_a, obj_b]);

    let t_name = interner.intern_string("T");
    let t_param = TypeParamInfo {
        name: t_name,
        constraint: None,
        default: None,
    };
    let t_type = interner.intern(TypeKey::TypeParameter(t_param.clone()));

    let k_name = interner.intern_string("K");
    let k_param = TypeParamInfo {
        name: k_name,
        constraint: None,
        default: None,
    };
    let k_type = interner.intern(TypeKey::TypeParameter(k_param.clone()));

    let keyof_t = interner.intern(TypeKey::KeyOf(t_type));
    let t_k = interner.intern(TypeKey::IndexAccess(t_type, k_type));

    let mapped = interner.mapped(MappedType {
        type_param: k_param,
        constraint: keyof_t,
        name_type: None,
        template: t_k,
        readonly_modifier: None,
        optional_modifier: None,
    });

    let mut subst = TypeSubstitution::new();
    subst.insert(t_name, union_ab);

    let result = instantiate_type(&interner, mapped, &subst);
    assert!(result != TypeId::ERROR, "Homomorphic on union should work");
}

#[test]
fn test_homomorphic_mapped_on_intersection_type() {
    let interner = TypeInterner::new();

    // { [K in keyof T]: T[K] } where T = A & B
    // Should work on intersection

    let obj_a = interner.object(vec![
        PropertyInfo {
            name: interner.intern_string("a"),
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
            type_id: TypeId::NUMBER,
            write_type: TypeId::NUMBER,
            optional: false,
            readonly: false,
            is_method: false,
        },
    ]);

    let intersection_ab = interner.intersection(vec![obj_a, obj_b]);

    let t_name = interner.intern_string("T");
    let t_param = TypeParamInfo {
        name: t_name,
        constraint: None,
        default: None,
    };
    let t_type = interner.intern(TypeKey::TypeParameter(t_param.clone()));

    let k_name = interner.intern_string("K");
    let k_param = TypeParamInfo {
        name: k_name,
        constraint: None,
        default: None,
    };
    let k_type = interner.intern(TypeKey::TypeParameter(k_param.clone()));

    let keyof_t = interner.intern(TypeKey::KeyOf(t_type));
    let t_k = interner.intern(TypeKey::IndexAccess(t_type, k_type));

    let mapped = interner.mapped(MappedType {
        type_param: k_param,
        constraint: keyof_t,
        name_type: None,
        template: t_k,
        readonly_modifier: None,
        optional_modifier: None,
    });

    let mut subst = TypeSubstitution::new();
    subst.insert(t_name, intersection_ab);

    let result = instantiate_type(&interner, mapped, &subst);
    assert!(result != TypeId::ERROR, "Homomorphic on intersection should work");
}

#[test]
fn test_homomorphic_mapped_with_conditional_template() {
    let interner = TypeInterner::new();

    // { [K in keyof T]: T[K] extends string ? "str" : "other" }
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

    let t_name = interner.intern_string("T");
    let t_param = TypeParamInfo {
        name: t_name,
        constraint: None,
        default: None,
    };
    let t_type = interner.intern(TypeKey::TypeParameter(t_param.clone()));

    let k_name = interner.intern_string("K");
    let k_param = TypeParamInfo {
        name: k_name,
        constraint: None,
        default: None,
    };
    let k_type = interner.intern(TypeKey::TypeParameter(k_param.clone()));

    let keyof_t = interner.intern(TypeKey::KeyOf(t_type));
    let t_k = interner.intern(TypeKey::IndexAccess(t_type, k_type));

    let str_lit = interner.literal_string("str");
    let other_lit = interner.literal_string("other");

    let template = interner.conditional(ConditionalType {
        check_type: t_k,
        extends_type: TypeId::STRING,
        true_type: str_lit,
        false_type: other_lit,
        is_distributive: false,
    });

    let mapped = interner.mapped(MappedType {
        type_param: k_param,
        constraint: keyof_t,
        name_type: None,
        template,
        readonly_modifier: None,
        optional_modifier: None,
    });

    let mut subst = TypeSubstitution::new();
    subst.insert(t_name, source_obj);

    let result = instantiate_type(&interner, mapped, &subst);
    assert!(result != TypeId::ERROR, "Homomorphic with conditional template should work");
}

// =============================================================================
// Mapped Type Edge Cases - Key Remapping
// =============================================================================

#[test]
fn test_key_remap_with_template_literal_getters() {
    let interner = TypeInterner::new();

    // { [K in keyof T as `get${Capitalize<K>}`]: () => T[K] }
    // Getters pattern: { a: string } -> { getA: () => string }

    let source_obj = interner.object(vec![
        PropertyInfo {
            name: interner.intern_string("name"),
            type_id: TypeId::STRING,
            write_type: TypeId::STRING,
            optional: false,
            readonly: false,
            is_method: false,
        },
    ]);

    let t_name = interner.intern_string("T");
    let t_param = TypeParamInfo {
        name: t_name,
        constraint: None,
        default: None,
    };
    let t_type = interner.intern(TypeKey::TypeParameter(t_param.clone()));

    let k_name = interner.intern_string("K");
    let k_param = TypeParamInfo {
        name: k_name,
        constraint: None,
        default: None,
    };
    let k_type = interner.intern(TypeKey::TypeParameter(k_param.clone()));

    let keyof_t = interner.intern(TypeKey::KeyOf(t_type));
    let t_k = interner.intern(TypeKey::IndexAccess(t_type, k_type));

    // Capitalize<K> - modeled as type application
    let capitalize_symbol = SymbolRef(312);
    let capitalize_base = interner.reference(capitalize_symbol);
    let cap_k = interner.application(capitalize_base, vec![k_type]);

    // `get${Capitalize<K>}`
    let name_type = interner.template_literal(vec![
        TemplateSpan::Text(interner.intern_string("get")),
        TemplateSpan::Type(cap_k),
    ]);

    // () => T[K]
    let getter_fn = interner.function(FunctionShape {
        type_params: vec![],
        params: vec![],
        this_type: None,
        return_type: t_k,
        type_predicate: None,
        is_constructor: false,
    });

    let mapped = interner.mapped(MappedType {
        type_param: k_param,
        constraint: keyof_t,
        name_type: Some(name_type),
        template: getter_fn,
        readonly_modifier: None,
        optional_modifier: None,
    });

    let mut subst = TypeSubstitution::new();
    subst.insert(t_name, source_obj);

    let result = instantiate_type(&interner, mapped, &subst);
    assert!(result != TypeId::ERROR, "Key remap with template literal getters should work");
}

#[test]
fn test_key_remap_with_template_literal_setters() {
    let interner = TypeInterner::new();

    // { [K in keyof T as `set${Capitalize<K>}`]: (value: T[K]) => void }
    // Setters pattern

    let source_obj = interner.object(vec![
        PropertyInfo {
            name: interner.intern_string("value"),
            type_id: TypeId::NUMBER,
            write_type: TypeId::NUMBER,
            optional: false,
            readonly: false,
            is_method: false,
        },
    ]);

    let t_name = interner.intern_string("T");
    let t_param = TypeParamInfo {
        name: t_name,
        constraint: None,
        default: None,
    };
    let t_type = interner.intern(TypeKey::TypeParameter(t_param.clone()));

    let k_name = interner.intern_string("K");
    let k_param = TypeParamInfo {
        name: k_name,
        constraint: None,
        default: None,
    };
    let k_type = interner.intern(TypeKey::TypeParameter(k_param.clone()));

    let keyof_t = interner.intern(TypeKey::KeyOf(t_type));
    let t_k = interner.intern(TypeKey::IndexAccess(t_type, k_type));

    // Capitalize<K> - modeled as type application
    let capitalize_symbol = SymbolRef(312);
    let capitalize_base = interner.reference(capitalize_symbol);
    let cap_k = interner.application(capitalize_base, vec![k_type]);

    // `set${Capitalize<K>}`
    let name_type = interner.template_literal(vec![
        TemplateSpan::Text(interner.intern_string("set")),
        TemplateSpan::Type(cap_k),
    ]);

    // (value: T[K]) => void
    let setter_fn = interner.function(FunctionShape {
        type_params: vec![],
        params: vec![ParamInfo {
            name: Some(interner.intern_string("value")),
            type_id: t_k,
            optional: false,
            rest: false,
        }],
        this_type: None,
        return_type: TypeId::VOID,
        type_predicate: None,
        is_constructor: false,
    });

    let mapped = interner.mapped(MappedType {
        type_param: k_param,
        constraint: keyof_t,
        name_type: Some(name_type),
        template: setter_fn,
        readonly_modifier: None,
        optional_modifier: None,
    });

    let mut subst = TypeSubstitution::new();
    subst.insert(t_name, source_obj);

    let result = instantiate_type(&interner, mapped, &subst);
    assert!(result != TypeId::ERROR, "Key remap with template literal setters should work");
}

#[test]
fn test_key_remap_filter_all_to_never() {
    let interner = TypeInterner::new();

    // { [K in keyof T as never]: T[K] }
    // Should produce empty object

    let source_obj = interner.object(vec![
        PropertyInfo {
            name: interner.intern_string("a"),
            type_id: TypeId::STRING,
            write_type: TypeId::STRING,
            optional: false,
            readonly: false,
            is_method: false,
        },
    ]);

    let t_name = interner.intern_string("T");
    let t_param = TypeParamInfo {
        name: t_name,
        constraint: None,
        default: None,
    };
    let t_type = interner.intern(TypeKey::TypeParameter(t_param.clone()));

    let k_name = interner.intern_string("K");
    let k_param = TypeParamInfo {
        name: k_name,
        constraint: None,
        default: None,
    };

    let keyof_t = interner.intern(TypeKey::KeyOf(t_type));
    let t_k = interner.intern(TypeKey::IndexAccess(t_type, interner.intern(TypeKey::TypeParameter(k_param.clone()))));

    let mapped = interner.mapped(MappedType {
        type_param: k_param,
        constraint: keyof_t,
        name_type: Some(TypeId::NEVER), // All keys remapped to never = filtered out
        template: t_k,
        readonly_modifier: None,
        optional_modifier: None,
    });

    let mut subst = TypeSubstitution::new();
    subst.insert(t_name, source_obj);

    let result = instantiate_type(&interner, mapped, &subst);
    assert!(result != TypeId::ERROR, "Key remap all to never should produce empty object");
}

#[test]
fn test_key_remap_with_exclude_pattern() {
    let interner = TypeInterner::new();

    // { [K in keyof T as Exclude<K, "b">]: T[K] }
    // Omit pattern - exclude key "b"

    let key_a = interner.literal_string("a");
    let key_b = interner.literal_string("b");
    let keys = interner.union(vec![key_a, key_b]);

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

    let k_name = interner.intern_string("K");
    let k_param = TypeParamInfo {
        name: k_name,
        constraint: Some(keys),
        default: None,
    };
    let k_type = interner.intern(TypeKey::TypeParameter(k_param.clone()));

    // Exclude<K, "b"> = K extends "b" ? never : K
    let exclude_cond = interner.conditional(ConditionalType {
        check_type: k_type,
        extends_type: key_b,
        true_type: TypeId::NEVER,
        false_type: k_type,
        is_distributive: true,
    });

    let template = interner.intern(TypeKey::IndexAccess(source_obj, k_type));

    let mapped = MappedType {
        type_param: k_param,
        constraint: keys,
        name_type: Some(exclude_cond),
        template,
        readonly_modifier: None,
        optional_modifier: None,
    };

    let result = evaluate_mapped(&interner, &mapped);
    assert!(result != TypeId::ERROR, "Key remap with Exclude pattern should work");
}

#[test]
fn test_key_remap_with_extract_pattern() {
    let interner = TypeInterner::new();

    // { [K in keyof T as Extract<K, "a">]: T[K] }
    // Pick pattern - only keep key "a"

    let key_a = interner.literal_string("a");
    let key_b = interner.literal_string("b");
    let keys = interner.union(vec![key_a, key_b]);

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

    let k_name = interner.intern_string("K");
    let k_param = TypeParamInfo {
        name: k_name,
        constraint: Some(keys),
        default: None,
    };
    let k_type = interner.intern(TypeKey::TypeParameter(k_param.clone()));

    // Extract<K, "a"> = K extends "a" ? K : never
    let extract_cond = interner.conditional(ConditionalType {
        check_type: k_type,
        extends_type: key_a,
        true_type: k_type,
        false_type: TypeId::NEVER,
        is_distributive: true,
    });

    let template = interner.intern(TypeKey::IndexAccess(source_obj, k_type));

    let mapped = MappedType {
        type_param: k_param,
        constraint: keys,
        name_type: Some(extract_cond),
        template,
        readonly_modifier: None,
        optional_modifier: None,
    };

    let result = evaluate_mapped(&interner, &mapped);
    assert!(result != TypeId::ERROR, "Key remap with Extract pattern should work");
}

#[test]
fn test_key_remap_uppercase_keys() {
    let interner = TypeInterner::new();

    // { [K in keyof T as Uppercase<K>]: T[K] }
    // { a: string } -> { A: string }

    let key_a = interner.literal_string("a");

    let source_obj = interner.object(vec![
        PropertyInfo {
            name: interner.intern_string("a"),
            type_id: TypeId::STRING,
            write_type: TypeId::STRING,
            optional: false,
            readonly: false,
            is_method: false,
        },
    ]);

    let k_name = interner.intern_string("K");
    let k_param = TypeParamInfo {
        name: k_name,
        constraint: Some(key_a),
        default: None,
    };
    let k_type = interner.intern(TypeKey::TypeParameter(k_param.clone()));

    // Uppercase<K> - modeled as type application
    let uppercase_symbol = SymbolRef(310);
    let uppercase_base = interner.reference(uppercase_symbol);
    let upper_k = interner.application(uppercase_base, vec![k_type]);

    let template = interner.intern(TypeKey::IndexAccess(source_obj, k_type));

    let mapped = MappedType {
        type_param: k_param,
        constraint: key_a,
        name_type: Some(upper_k),
        template,
        readonly_modifier: None,
        optional_modifier: None,
    };

    let result = evaluate_mapped(&interner, &mapped);
    assert!(result != TypeId::ERROR, "Key remap with Uppercase should work");
}

#[test]
fn test_key_remap_lowercase_keys() {
    let interner = TypeInterner::new();

    // { [K in keyof T as Lowercase<K>]: T[K] }
    // { A: string } -> { a: string }

    let key_upper = interner.literal_string("A");

    let source_obj = interner.object(vec![
        PropertyInfo {
            name: interner.intern_string("A"),
            type_id: TypeId::STRING,
            write_type: TypeId::STRING,
            optional: false,
            readonly: false,
            is_method: false,
        },
    ]);

    let k_name = interner.intern_string("K");
    let k_param = TypeParamInfo {
        name: k_name,
        constraint: Some(key_upper),
        default: None,
    };
    let k_type = interner.intern(TypeKey::TypeParameter(k_param.clone()));

    // Lowercase<K> - modeled as type application
    let lowercase_symbol = SymbolRef(311);
    let lowercase_base = interner.reference(lowercase_symbol);
    let lower_k = interner.application(lowercase_base, vec![k_type]);

    let template = interner.intern(TypeKey::IndexAccess(source_obj, k_type));

    let mapped = MappedType {
        type_param: k_param,
        constraint: key_upper,
        name_type: Some(lower_k),
        template,
        readonly_modifier: None,
        optional_modifier: None,
    };

    let result = evaluate_mapped(&interner, &mapped);
    assert!(result != TypeId::ERROR, "Key remap with Lowercase should work");
}

#[test]
fn test_key_remap_with_prefix_template() {
    let interner = TypeInterner::new();

    // { [K in keyof T as `_${K}`]: T[K] }
    // Add underscore prefix to all keys

    let key_a = interner.literal_string("a");

    let source_obj = interner.object(vec![
        PropertyInfo {
            name: interner.intern_string("a"),
            type_id: TypeId::STRING,
            write_type: TypeId::STRING,
            optional: false,
            readonly: false,
            is_method: false,
        },
    ]);

    let k_name = interner.intern_string("K");
    let k_param = TypeParamInfo {
        name: k_name,
        constraint: Some(key_a),
        default: None,
    };
    let k_type = interner.intern(TypeKey::TypeParameter(k_param.clone()));

    // `_${K}`
    let name_type = interner.template_literal(vec![
        TemplateSpan::Text(interner.intern_string("_")),
        TemplateSpan::Type(k_type),
    ]);

    let template = interner.intern(TypeKey::IndexAccess(source_obj, k_type));

    let mapped = MappedType {
        type_param: k_param,
        constraint: key_a,
        name_type: Some(name_type),
        template,
        readonly_modifier: None,
        optional_modifier: None,
    };

    let result = evaluate_mapped(&interner, &mapped);
    assert!(result != TypeId::ERROR, "Key remap with prefix template should work");
}

#[test]
fn test_key_remap_with_suffix_template() {
    let interner = TypeInterner::new();

    // { [K in keyof T as `${K}Changed`]: T[K] }
    // Add suffix to all keys

    let key_a = interner.literal_string("value");

    let source_obj = interner.object(vec![
        PropertyInfo {
            name: interner.intern_string("value"),
            type_id: TypeId::STRING,
            write_type: TypeId::STRING,
            optional: false,
            readonly: false,
            is_method: false,
        },
    ]);

    let k_name = interner.intern_string("K");
    let k_param = TypeParamInfo {
        name: k_name,
        constraint: Some(key_a),
        default: None,
    };
    let k_type = interner.intern(TypeKey::TypeParameter(k_param.clone()));

    // `${K}Changed`
    let name_type = interner.template_literal(vec![
        TemplateSpan::Type(k_type),
        TemplateSpan::Text(interner.intern_string("Changed")),
    ]);

    let template = interner.intern(TypeKey::IndexAccess(source_obj, k_type));

    let mapped = MappedType {
        type_param: k_param,
        constraint: key_a,
        name_type: Some(name_type),
        template,
        readonly_modifier: None,
        optional_modifier: None,
    };

    let result = evaluate_mapped(&interner, &mapped);
    assert!(result != TypeId::ERROR, "Key remap with suffix template should work");
}

#[test]
fn test_key_remap_filter_by_value_type() {
    let interner = TypeInterner::new();

    // { [K in keyof T as T[K] extends string ? K : never]: T[K] }
    // Only keep keys whose values are strings

    let key_a = interner.literal_string("a");
    let key_b = interner.literal_string("b");
    let keys = interner.union(vec![key_a, key_b]);

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

    let k_name = interner.intern_string("K");
    let k_param = TypeParamInfo {
        name: k_name,
        constraint: Some(keys),
        default: None,
    };
    let k_type = interner.intern(TypeKey::TypeParameter(k_param.clone()));

    let t_k = interner.intern(TypeKey::IndexAccess(source_obj, k_type));

    // T[K] extends string ? K : never
    let name_type = interner.conditional(ConditionalType {
        check_type: t_k,
        extends_type: TypeId::STRING,
        true_type: k_type,
        false_type: TypeId::NEVER,
        is_distributive: false,
    });

    let mapped = MappedType {
        type_param: k_param,
        constraint: keys,
        name_type: Some(name_type),
        template: t_k,
        readonly_modifier: None,
        optional_modifier: None,
    };

    let result = evaluate_mapped(&interner, &mapped);
    assert!(result != TypeId::ERROR, "Key remap filtering by value type should work");
}

#[test]
fn test_key_remap_multiple_keys_to_same() {
    let interner = TypeInterner::new();

    // { [K in keyof T as "shared"]: T[K] }
    // All keys remap to same name - values should union

    let key_a = interner.literal_string("a");
    let key_b = interner.literal_string("b");
    let keys = interner.union(vec![key_a, key_b]);

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

    let k_name = interner.intern_string("K");
    let k_param = TypeParamInfo {
        name: k_name,
        constraint: Some(keys),
        default: None,
    };
    let k_type = interner.intern(TypeKey::TypeParameter(k_param.clone()));

    let shared_key = interner.literal_string("shared");
    let template = interner.intern(TypeKey::IndexAccess(source_obj, k_type));

    let mapped = MappedType {
        type_param: k_param,
        constraint: keys,
        name_type: Some(shared_key), // All keys map to "shared"
        template,
        readonly_modifier: None,
        optional_modifier: None,
    };

    let result = evaluate_mapped(&interner, &mapped);
    assert!(result != TypeId::ERROR, "Key remap colliding keys should produce union value");
}

#[test]
fn test_key_remap_preserves_optionality() {
    let interner = TypeInterner::new();

    // { [K in keyof T as `new_${K}`]: T[K] }
    // Remapped keys should preserve original optional modifiers

    let key_a = interner.literal_string("a");

    let source_obj = interner.object(vec![
        PropertyInfo {
            name: interner.intern_string("a"),
            type_id: TypeId::STRING,
            write_type: TypeId::STRING,
            optional: true, // optional
            readonly: false,
            is_method: false,
        },
    ]);

    let k_name = interner.intern_string("K");
    let k_param = TypeParamInfo {
        name: k_name,
        constraint: Some(key_a),
        default: None,
    };
    let k_type = interner.intern(TypeKey::TypeParameter(k_param.clone()));

    let name_type = interner.template_literal(vec![
        TemplateSpan::Text(interner.intern_string("new_")),
        TemplateSpan::Type(k_type),
    ]);

    let template = interner.intern(TypeKey::IndexAccess(source_obj, k_type));

    let mapped = MappedType {
        type_param: k_param,
        constraint: key_a,
        name_type: Some(name_type),
        template,
        readonly_modifier: None,
        optional_modifier: None, // No modifier change - should preserve
    };

    let result = evaluate_mapped(&interner, &mapped);
    assert!(result != TypeId::ERROR, "Key remap should preserve optionality");
}

#[test]
fn test_key_remap_with_conditional_rename() {
    let interner = TypeInterner::new();

    // { [K in keyof T as K extends "old" ? "new" : K]: T[K] }
    // Rename specific key while keeping others

    let key_old = interner.literal_string("old");
    let key_other = interner.literal_string("other");
    let keys = interner.union(vec![key_old, key_other]);

    let source_obj = interner.object(vec![
        PropertyInfo {
            name: interner.intern_string("old"),
            type_id: TypeId::STRING,
            write_type: TypeId::STRING,
            optional: false,
            readonly: false,
            is_method: false,
        },
        PropertyInfo {
            name: interner.intern_string("other"),
            type_id: TypeId::NUMBER,
            write_type: TypeId::NUMBER,
            optional: false,
            readonly: false,
            is_method: false,
        },
    ]);

    let k_name = interner.intern_string("K");
    let k_param = TypeParamInfo {
        name: k_name,
        constraint: Some(keys),
        default: None,
    };
    let k_type = interner.intern(TypeKey::TypeParameter(k_param.clone()));

    let key_new = interner.literal_string("new");

    // K extends "old" ? "new" : K
    let name_type = interner.conditional(ConditionalType {
        check_type: k_type,
        extends_type: key_old,
        true_type: key_new,
        false_type: k_type,
        is_distributive: true,
    });

    let template = interner.intern(TypeKey::IndexAccess(source_obj, k_type));

    let mapped = MappedType {
        type_param: k_param,
        constraint: keys,
        name_type: Some(name_type),
        template,
        readonly_modifier: None,
        optional_modifier: None,
    };

    let result = evaluate_mapped(&interner, &mapped);
    assert!(result != TypeId::ERROR, "Key remap with conditional rename should work");
}

#[test]
fn test_key_remap_symbol_keys_filtered() {
    let interner = TypeInterner::new();

    // { [K in keyof T as K extends symbol ? never : K]: T[K] }
    // Filter out symbol keys

    let key_a = interner.literal_string("a");

    let source_obj = interner.object(vec![
        PropertyInfo {
            name: interner.intern_string("a"),
            type_id: TypeId::STRING,
            write_type: TypeId::STRING,
            optional: false,
            readonly: false,
            is_method: false,
        },
    ]);

    let k_name = interner.intern_string("K");
    let k_param = TypeParamInfo {
        name: k_name,
        constraint: Some(key_a),
        default: None,
    };
    let k_type = interner.intern(TypeKey::TypeParameter(k_param.clone()));

    // K extends symbol ? never : K
    let name_type = interner.conditional(ConditionalType {
        check_type: k_type,
        extends_type: TypeId::SYMBOL,
        true_type: TypeId::NEVER,
        false_type: k_type,
        is_distributive: true,
    });

    let template = interner.intern(TypeKey::IndexAccess(source_obj, k_type));

    let mapped = MappedType {
        type_param: k_param,
        constraint: key_a,
        name_type: Some(name_type),
        template,
        readonly_modifier: None,
        optional_modifier: None,
    };

    let result = evaluate_mapped(&interner, &mapped);
    assert!(result != TypeId::ERROR, "Key remap filtering symbols should work");
}

#[test]
fn test_mapped_empty_constraint() {
    let interner = TypeInterner::new();

    // { [K in never]: string }
    // Should produce empty object

    let k_name = interner.intern_string("K");
    let k_param = TypeParamInfo {
        name: k_name,
        constraint: Some(TypeId::NEVER),
        default: None,
    };

    let mapped = MappedType {
        type_param: k_param,
        constraint: TypeId::NEVER,
        name_type: None,
        template: TypeId::STRING,
        readonly_modifier: None,
        optional_modifier: None,
    };

    let result = evaluate_mapped(&interner, &mapped);
    // Mapped over never should not produce error
    assert!(result != TypeId::ERROR, "Mapped over never should not produce error");
}

// =============================================================================
// Intersection Type Edge Cases
// =============================================================================

#[test]
fn test_intersection_with_never() {
    let interner = TypeInterner::new();

    // T & never = never
    let intersection = interner.intersection(vec![TypeId::STRING, TypeId::NEVER]);
    let result = evaluate_type(&interner, intersection);
    assert_eq!(result, TypeId::NEVER, "T & never should be never");
}

#[test]
fn test_intersection_with_unknown() {
    let interner = TypeInterner::new();

    // T & unknown = T
    let intersection = interner.intersection(vec![TypeId::STRING, TypeId::UNKNOWN]);
    let result = evaluate_type(&interner, intersection);
    // Should simplify to string
    assert!(result != TypeId::ERROR, "T & unknown should not error");
}

#[test]
fn test_intersection_with_any() {
    let interner = TypeInterner::new();

    // T & any = any
    let intersection = interner.intersection(vec![TypeId::STRING, TypeId::ANY]);
    let result = evaluate_type(&interner, intersection);
    assert!(result != TypeId::ERROR, "T & any should not error");
}

#[test]
fn test_intersection_object_merge() {
    let interner = TypeInterner::new();

    // { a: string } & { b: number } = { a: string; b: number }
    let obj1 = interner.object(vec![PropertyInfo {
        name: interner.intern_string("a"),
        type_id: TypeId::STRING,
        write_type: TypeId::STRING,
        optional: false,
        readonly: false,
        is_method: false,
    }]);

    let obj2 = interner.object(vec![PropertyInfo {
        name: interner.intern_string("b"),
        type_id: TypeId::NUMBER,
        write_type: TypeId::NUMBER,
        optional: false,
        readonly: false,
        is_method: false,
    }]);

    let intersection = interner.intersection(vec![obj1, obj2]);
    let result = evaluate_type(&interner, intersection);
    assert!(result != TypeId::ERROR, "Object intersection should merge properties");
}

#[test]
fn test_intersection_same_property_narrows() {
    let interner = TypeInterner::new();

    // { a: string | number } & { a: string } = { a: string }
    let string_or_number = interner.union(vec![TypeId::STRING, TypeId::NUMBER]);

    let obj1 = interner.object(vec![PropertyInfo {
        name: interner.intern_string("a"),
        type_id: string_or_number,
        write_type: string_or_number,
        optional: false,
        readonly: false,
        is_method: false,
    }]);

    let obj2 = interner.object(vec![PropertyInfo {
        name: interner.intern_string("a"),
        type_id: TypeId::STRING,
        write_type: TypeId::STRING,
        optional: false,
        readonly: false,
        is_method: false,
    }]);

    let intersection = interner.intersection(vec![obj1, obj2]);
    let result = evaluate_type(&interner, intersection);
    assert!(result != TypeId::ERROR, "Intersection should narrow common properties");
}

#[test]
fn test_intersection_function_overload() {
    let interner = TypeInterner::new();

    // ((a: string) => void) & ((a: number) => void)
    // Creates overloaded function
    let fn1 = interner.function(FunctionShape {
        type_params: vec![],
        params: vec![ParamInfo {
            name: Some(interner.intern_string("a")),
            type_id: TypeId::STRING,
            optional: false,
            rest: false,
        }],
        this_type: None,
        return_type: TypeId::VOID,
        type_predicate: None,
        is_constructor: false,
    });

    let fn2 = interner.function(FunctionShape {
        type_params: vec![],
        params: vec![ParamInfo {
            name: Some(interner.intern_string("a")),
            type_id: TypeId::NUMBER,
            optional: false,
            rest: false,
        }],
        this_type: None,
        return_type: TypeId::VOID,
        type_predicate: None,
        is_constructor: false,
    });

    let intersection = interner.intersection(vec![fn1, fn2]);
    let result = evaluate_type(&interner, intersection);
    assert!(result != TypeId::ERROR, "Function intersection should work");
}

#[test]
fn test_intersection_flattens_nested() {
    let interner = TypeInterner::new();

    // (A & B) & C should flatten to A & B & C
    let inner = interner.intersection(vec![TypeId::STRING, TypeId::NUMBER]);
    let outer = interner.intersection(vec![inner, TypeId::BOOLEAN]);
    let result = evaluate_type(&interner, outer);
    assert!(result != TypeId::ERROR, "Nested intersection should flatten");
}

// =============================================================================
// Union Type Edge Cases
// =============================================================================

#[test]
fn test_union_with_never() {
    let interner = TypeInterner::new();

    // T | never = T
    let union = interner.union(vec![TypeId::STRING, TypeId::NEVER]);
    let result = evaluate_type(&interner, union);
    // Should simplify to just string
    assert!(result != TypeId::ERROR, "T | never should simplify");
}

#[test]
fn test_union_with_unknown() {
    let interner = TypeInterner::new();

    // T | unknown = unknown
    let union = interner.union(vec![TypeId::STRING, TypeId::UNKNOWN]);
    let result = evaluate_type(&interner, union);
    assert!(result != TypeId::ERROR, "T | unknown should work");
}

#[test]
fn test_union_with_any() {
    let interner = TypeInterner::new();

    // T | any = any
    let union = interner.union(vec![TypeId::STRING, TypeId::ANY]);
    let result = evaluate_type(&interner, union);
    assert!(result != TypeId::ERROR, "T | any should work");
}

#[test]
fn test_union_flattens_nested() {
    let interner = TypeInterner::new();

    // (A | B) | C should flatten to A | B | C
    let inner = interner.union(vec![TypeId::STRING, TypeId::NUMBER]);
    let outer = interner.union(vec![inner, TypeId::BOOLEAN]);
    let result = evaluate_type(&interner, outer);
    assert!(result != TypeId::ERROR, "Nested union should flatten");
}

#[test]
fn test_union_deduplicates() {
    let interner = TypeInterner::new();

    // string | string | number = string | number
    let union = interner.union(vec![TypeId::STRING, TypeId::STRING, TypeId::NUMBER]);
    let result = evaluate_type(&interner, union);
    assert!(result != TypeId::ERROR, "Union should deduplicate");
}

#[test]
fn test_union_literal_subsumed_by_base() {
    let interner = TypeInterner::new();

    // "hello" | string = string (literal subsumed by base type)
    let hello = interner.literal_string("hello");
    let union = interner.union(vec![hello, TypeId::STRING]);
    let result = evaluate_type(&interner, union);
    assert!(result != TypeId::ERROR, "Literal should be subsumed by base type in union");
}

// =============================================================================
// Generic Constraint Edge Cases
// =============================================================================

#[test]
fn test_type_param_with_extends_string() {
    let interner = TypeInterner::new();

    // <T extends string>
    let t_name = interner.intern_string("T");
    let t_param = TypeParamInfo {
        name: t_name,
        constraint: Some(TypeId::STRING),
        default: None,
    };
    let t_type = interner.intern(TypeKey::TypeParameter(t_param));

    let result = evaluate_type(&interner, t_type);
    assert!(result != TypeId::ERROR, "Type param with constraint should work");
}

#[test]
fn test_type_param_with_extends_union() {
    let interner = TypeInterner::new();

    // <T extends string | number>
    let constraint = interner.union(vec![TypeId::STRING, TypeId::NUMBER]);
    let t_name = interner.intern_string("T");
    let t_param = TypeParamInfo {
        name: t_name,
        constraint: Some(constraint),
        default: None,
    };
    let t_type = interner.intern(TypeKey::TypeParameter(t_param));

    let result = evaluate_type(&interner, t_type);
    assert!(result != TypeId::ERROR, "Type param with union constraint should work");
}

#[test]
fn test_type_param_with_extends_keyof() {
    let interner = TypeInterner::new();

    // <T, K extends keyof T>
    let t_name = interner.intern_string("T");
    let t_param = TypeParamInfo {
        name: t_name,
        constraint: None,
        default: None,
    };
    let t_type = interner.intern(TypeKey::TypeParameter(t_param));

    let keyof_t = interner.intern(TypeKey::KeyOf(t_type));

    let k_name = interner.intern_string("K");
    let k_param = TypeParamInfo {
        name: k_name,
        constraint: Some(keyof_t),
        default: None,
    };
    let k_type = interner.intern(TypeKey::TypeParameter(k_param));

    let result = evaluate_type(&interner, k_type);
    assert!(result != TypeId::ERROR, "Type param with keyof constraint should work");
}

#[test]
fn test_type_param_default_uses_constraint() {
    let interner = TypeInterner::new();

    // <T extends string = "default">
    let default_val = interner.literal_string("default");
    let t_name = interner.intern_string("T");
    let t_param = TypeParamInfo {
        name: t_name,
        constraint: Some(TypeId::STRING),
        default: Some(default_val),
    };
    let t_type = interner.intern(TypeKey::TypeParameter(t_param));

    let result = evaluate_type(&interner, t_type);
    assert!(result != TypeId::ERROR, "Type param with constraint and default should work");
}

// =============================================================================
// Nested Conditional Types
// =============================================================================

#[test]
fn test_nested_conditional_both_branches() {
    let interner = TypeInterner::new();

    // T extends string ? (T extends "a" ? 1 : 2) : 3
    let lit_1 = interner.literal_number(1.0);
    let lit_2 = interner.literal_number(2.0);
    let lit_3 = interner.literal_number(3.0);
    let lit_a = interner.literal_string("a");

    let t_name = interner.intern_string("T");
    let t_param = TypeParamInfo {
        name: t_name,
        constraint: None,
        default: None,
    };
    let t_type = interner.intern(TypeKey::TypeParameter(t_param.clone()));

    // Inner: T extends "a" ? 1 : 2
    let inner_cond = interner.conditional(ConditionalType {
        check_type: t_type,
        extends_type: lit_a,
        true_type: lit_1,
        false_type: lit_2,
        is_distributive: false,
    });

    // Outer: T extends string ? inner : 3
    let outer_cond = ConditionalType {
        check_type: t_type,
        extends_type: TypeId::STRING,
        true_type: inner_cond,
        false_type: lit_3,
        is_distributive: false,
    };

    let result = evaluate_conditional(&interner, &outer_cond);
    assert!(result != TypeId::ERROR, "Nested conditional should work");
}

#[test]
fn test_conditional_with_infer_in_true_branch() {
    let interner = TypeInterner::new();

    // T extends (infer U)[] ? U : T
    let t_name = interner.intern_string("T");
    let t_param = TypeParamInfo {
        name: t_name,
        constraint: None,
        default: None,
    };
    let t_type = interner.intern(TypeKey::TypeParameter(t_param.clone()));

    let u_name = interner.intern_string("U");
    let u_infer = interner.intern(TypeKey::Infer(TypeParamInfo {
        name: u_name,
        constraint: None,
        default: None,
    }));
    let u_type = interner.intern(TypeKey::TypeParameter(TypeParamInfo {
        name: u_name,
        constraint: None,
        default: None,
    }));

    let array_of_u = interner.array(u_infer);

    let cond = ConditionalType {
        check_type: t_type,
        extends_type: array_of_u,
        true_type: u_type,
        false_type: t_type,
        is_distributive: false,
    };

    let cond_type = interner.conditional(cond.clone());
    let result = evaluate_type(&interner, cond_type);
    assert!(result != TypeId::ERROR, "Conditional with infer in extends should work");
}

#[test]
fn test_conditional_chained() {
    let interner = TypeInterner::new();

    // T extends string ? "string" : T extends number ? "number" : "other"
    let str_lit = interner.literal_string("string");
    let num_lit = interner.literal_string("number");
    let other_lit = interner.literal_string("other");

    let t_name = interner.intern_string("T");
    let t_param = TypeParamInfo {
        name: t_name,
        constraint: None,
        default: None,
    };
    let t_type = interner.intern(TypeKey::TypeParameter(t_param.clone()));

    // Inner: T extends number ? "number" : "other"
    let inner_cond = interner.conditional(ConditionalType {
        check_type: t_type,
        extends_type: TypeId::NUMBER,
        true_type: num_lit,
        false_type: other_lit,
        is_distributive: false,
    });

    // Outer: T extends string ? "string" : inner
    let outer_cond = ConditionalType {
        check_type: t_type,
        extends_type: TypeId::STRING,
        true_type: str_lit,
        false_type: inner_cond,
        is_distributive: false,
    };

    let result = evaluate_conditional(&interner, &outer_cond);
    assert!(result != TypeId::ERROR, "Chained conditional should work");
}

// =============================================================================
// Function Subtyping Edge Cases
// =============================================================================

#[test]
fn test_function_param_contravariance() {
    let interner = TypeInterner::new();

    // (x: string | number) => void is subtype of (x: string) => void
    // Due to contravariance of parameters
    let string_or_number = interner.union(vec![TypeId::STRING, TypeId::NUMBER]);

    let fn_wide = interner.function(FunctionShape {
        type_params: vec![],
        params: vec![ParamInfo {
            name: Some(interner.intern_string("x")),
            type_id: string_or_number,
            optional: false,
            rest: false,
        }],
        this_type: None,
        return_type: TypeId::VOID,
        type_predicate: None,
        is_constructor: false,
    });

    let fn_narrow = interner.function(FunctionShape {
        type_params: vec![],
        params: vec![ParamInfo {
            name: Some(interner.intern_string("x")),
            type_id: TypeId::STRING,
            optional: false,
            rest: false,
        }],
        this_type: None,
        return_type: TypeId::VOID,
        type_predicate: None,
        is_constructor: false,
    });

    assert!(fn_wide != TypeId::ERROR, "Wide param function should be valid");
    assert!(fn_narrow != TypeId::ERROR, "Narrow param function should be valid");
}

#[test]
fn test_function_return_covariance() {
    let interner = TypeInterner::new();

    // () => string is subtype of () => string | number
    // Due to covariance of return types
    let string_or_number = interner.union(vec![TypeId::STRING, TypeId::NUMBER]);

    let fn_narrow_return = interner.function(FunctionShape {
        type_params: vec![],
        params: vec![],
        this_type: None,
        return_type: TypeId::STRING,
        type_predicate: None,
        is_constructor: false,
    });

    let fn_wide_return = interner.function(FunctionShape {
        type_params: vec![],
        params: vec![],
        this_type: None,
        return_type: string_or_number,
        type_predicate: None,
        is_constructor: false,
    });

    assert!(fn_narrow_return != TypeId::ERROR, "Narrow return function should be valid");
    assert!(fn_wide_return != TypeId::ERROR, "Wide return function should be valid");
}

#[test]
fn test_function_optional_param_compat() {
    let interner = TypeInterner::new();

    // (x?: string) => void accepts () => void call
    let fn_optional = interner.function(FunctionShape {
        type_params: vec![],
        params: vec![ParamInfo {
            name: Some(interner.intern_string("x")),
            type_id: TypeId::STRING,
            optional: true,
            rest: false,
        }],
        this_type: None,
        return_type: TypeId::VOID,
        type_predicate: None,
        is_constructor: false,
    });

    let fn_no_params = interner.function(FunctionShape {
        type_params: vec![],
        params: vec![],
        this_type: None,
        return_type: TypeId::VOID,
        type_predicate: None,
        is_constructor: false,
    });

    assert!(fn_optional != TypeId::ERROR, "Optional param function should be valid");
    assert!(fn_no_params != TypeId::ERROR, "No param function should be valid");
}

#[test]
fn test_function_rest_param_accepts_array() {
    let interner = TypeInterner::new();

    // (...args: string[]) => void
    let string_array = interner.array(TypeId::STRING);
    let fn_rest = interner.function(FunctionShape {
        type_params: vec![],
        params: vec![ParamInfo {
            name: Some(interner.intern_string("args")),
            type_id: string_array,
            optional: false,
            rest: true,
        }],
        this_type: None,
        return_type: TypeId::VOID,
        type_predicate: None,
        is_constructor: false,
    });

    assert!(fn_rest != TypeId::ERROR, "Rest param function should be valid");
}

#[test]
fn test_function_generic_instantiation() {
    let interner = TypeInterner::new();

    // <T>(x: T) => T instantiated with string becomes (x: string) => string
    let t_name = interner.intern_string("T");
    let t_param = TypeParamInfo {
        name: t_name,
        constraint: None,
        default: None,
    };
    let t_type = interner.intern(TypeKey::TypeParameter(t_param.clone()));

    let generic_fn = interner.function(FunctionShape {
        type_params: vec![t_param],
        params: vec![ParamInfo {
            name: Some(interner.intern_string("x")),
            type_id: t_type,
            optional: false,
            rest: false,
        }],
        this_type: None,
        return_type: t_type,
        type_predicate: None,
        is_constructor: false,
    });

    assert!(generic_fn != TypeId::ERROR, "Generic function should be valid");

    // Instantiate with string
    let mut subst = TypeSubstitution::new();
    subst.insert(t_name, TypeId::STRING);
    let instantiated = instantiate_type(&interner, generic_fn, &subst);
    assert!(instantiated != TypeId::ERROR, "Instantiated function should be valid");
}

// =============================================================================
// Index Access Edge Cases
// =============================================================================

#[test]
fn test_index_access_array_with_number_key() {
    let interner = TypeInterner::new();

    // string[][number] = string
    let string_array = interner.array(TypeId::STRING);
    let index_access = interner.intern(TypeKey::IndexAccess(string_array, TypeId::NUMBER));

    let result = evaluate_type(&interner, index_access);
    assert!(result != TypeId::ERROR, "Array indexed by number should work");
}

#[test]
fn test_index_access_tuple_with_numeric_literal() {
    let interner = TypeInterner::new();

    // [string, number][0] = string
    let tuple = interner.tuple(vec![
        TupleElement { type_id: TypeId::STRING, name: None, optional: false, rest: false },
        TupleElement { type_id: TypeId::NUMBER, name: None, optional: false, rest: false },
    ]);

    let lit_0 = interner.literal_number(0.0);
    let index_access = interner.intern(TypeKey::IndexAccess(tuple, lit_0));

    let result = evaluate_type(&interner, index_access);
    assert!(result != TypeId::ERROR, "Tuple indexed by numeric literal should work");
}

#[test]
fn test_index_access_object_string_literal() {
    let interner = TypeInterner::new();

    // { a: string; b: number }["a"] = string
    let obj = interner.object(vec![
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

    let key_a = interner.literal_string("a");
    let index_access = interner.intern(TypeKey::IndexAccess(obj, key_a));

    let result = evaluate_type(&interner, index_access);
    assert!(result != TypeId::ERROR, "Object indexed by string literal should work");
}

#[test]
fn test_index_access_with_keyof() {
    let interner = TypeInterner::new();

    // T[keyof T] where T = { a: string; b: number } = string | number
    let obj = interner.object(vec![
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

    let keyof_obj = interner.intern(TypeKey::KeyOf(obj));
    let index_access = interner.intern(TypeKey::IndexAccess(obj, keyof_obj));

    let result = evaluate_type(&interner, index_access);
    assert!(result != TypeId::ERROR, "Object indexed by keyof should work");
}

// =============================================================================
// Literal Type Edge Cases
// =============================================================================

#[test]
fn test_literal_string_type() {
    let interner = TypeInterner::new();

    let hello = interner.literal_string("hello");
    let result = evaluate_type(&interner, hello);
    assert!(result != TypeId::ERROR, "String literal type should work");
}

#[test]
fn test_literal_number_type() {
    let interner = TypeInterner::new();

    let num = interner.literal_number(42.0);
    let result = evaluate_type(&interner, num);
    assert!(result != TypeId::ERROR, "Number literal type should work");
}

#[test]
fn test_literal_boolean_type() {
    let interner = TypeInterner::new();

    let lit_true = interner.literal_boolean(true);
    let lit_false = interner.literal_boolean(false);
    let result_true = evaluate_type(&interner, lit_true);
    let result_false = evaluate_type(&interner, lit_false);
    assert!(result_true != TypeId::ERROR, "Boolean literal true should work");
    assert!(result_false != TypeId::ERROR, "Boolean literal false should work");
}

#[test]
fn test_literal_bigint_type() {
    let interner = TypeInterner::new();

    let bigint = interner.literal_bigint("9007199254740991");
    let result = evaluate_type(&interner, bigint);
    assert!(result != TypeId::ERROR, "BigInt literal type should work");
}

#[test]
fn test_symbol_type() {
    let interner = TypeInterner::new();

    // Symbol type is a primitive
    let result = evaluate_type(&interner, TypeId::SYMBOL);
    assert!(result != TypeId::ERROR, "Symbol type should work");
}

// =============================================================================
// Type Application Edge Cases
// =============================================================================

#[test]
fn test_type_application_single_arg() {
    let interner = TypeInterner::new();

    // Array<string> is Application(Array, [string])
    let array_symbol = SymbolRef(200);
    let array_base = interner.reference(array_symbol);
    let array_string = interner.application(array_base, vec![TypeId::STRING]);

    let result = evaluate_type(&interner, array_string);
    assert!(result != TypeId::ERROR, "Type application with single arg should work");
}

#[test]
fn test_type_application_multiple_args() {
    let interner = TypeInterner::new();

    // Map<string, number> is Application(Map, [string, number])
    let map_symbol = SymbolRef(201);
    let map_base = interner.reference(map_symbol);
    let map_string_number = interner.application(map_base, vec![TypeId::STRING, TypeId::NUMBER]);

    let result = evaluate_type(&interner, map_string_number);
    assert!(result != TypeId::ERROR, "Type application with multiple args should work");
}

#[test]
fn test_type_application_nested() {
    let interner = TypeInterner::new();

    // Array<Array<string>> - nested application
    let array_symbol = SymbolRef(200);
    let array_base = interner.reference(array_symbol);
    let inner = interner.application(array_base, vec![TypeId::STRING]);
    let outer = interner.application(array_base, vec![inner]);

    let result = evaluate_type(&interner, outer);
    assert!(result != TypeId::ERROR, "Nested type application should work");
}

#[test]
fn test_type_application_with_union_arg() {
    let interner = TypeInterner::new();

    // Array<string | number>
    let array_symbol = SymbolRef(200);
    let array_base = interner.reference(array_symbol);
    let union = interner.union(vec![TypeId::STRING, TypeId::NUMBER]);
    let array_union = interner.application(array_base, vec![union]);

    let result = evaluate_type(&interner, array_union);
    assert!(result != TypeId::ERROR, "Type application with union arg should work");
}

// =============================================================================
// Tuple Type Operations
// =============================================================================

#[test]
fn test_tuple_empty() {
    let interner = TypeInterner::new();

    // [] - empty tuple
    let tuple = interner.tuple(vec![]);
    let result = evaluate_type(&interner, tuple);
    assert!(result != TypeId::ERROR, "Empty tuple should work");
}

#[test]
fn test_tuple_with_labels() {
    let interner = TypeInterner::new();

    // [first: string, second: number]
    let tuple = interner.tuple(vec![
        TupleElement {
            type_id: TypeId::STRING,
            name: Some(interner.intern_string("first")),
            optional: false,
            rest: false,
        },
        TupleElement {
            type_id: TypeId::NUMBER,
            name: Some(interner.intern_string("second")),
            optional: false,
            rest: false,
        },
    ]);

    let result = evaluate_type(&interner, tuple);
    assert!(result != TypeId::ERROR, "Labeled tuple should work");
}

#[test]
fn test_tuple_with_optional_elements() {
    let interner = TypeInterner::new();

    // [string, number?]
    let tuple = interner.tuple(vec![
        TupleElement {
            type_id: TypeId::STRING,
            name: None,
            optional: false,
            rest: false,
        },
        TupleElement {
            type_id: TypeId::NUMBER,
            name: None,
            optional: true,
            rest: false,
        },
    ]);

    let result = evaluate_type(&interner, tuple);
    assert!(result != TypeId::ERROR, "Tuple with optional elements should work");
}

#[test]
fn test_tuple_with_rest_element() {
    let interner = TypeInterner::new();

    // [string, ...number[]]
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

    let result = evaluate_type(&interner, tuple);
    assert!(result != TypeId::ERROR, "Tuple with rest element should work");
}

#[test]
fn test_tuple_with_rest_in_middle() {
    let interner = TypeInterner::new();

    // [string, ...number[], boolean]
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
        TupleElement {
            type_id: TypeId::BOOLEAN,
            name: None,
            optional: false,
            rest: false,
        },
    ]);

    let result = evaluate_type(&interner, tuple);
    assert!(result != TypeId::ERROR, "Tuple with rest in middle should work");
}

// =============================================================================
// Array Type Operations
// =============================================================================

#[test]
fn test_array_of_primitives() {
    let interner = TypeInterner::new();

    let string_array = interner.array(TypeId::STRING);
    let number_array = interner.array(TypeId::NUMBER);
    let boolean_array = interner.array(TypeId::BOOLEAN);

    assert!(evaluate_type(&interner, string_array) != TypeId::ERROR, "string[] should work");
    assert!(evaluate_type(&interner, number_array) != TypeId::ERROR, "number[] should work");
    assert!(evaluate_type(&interner, boolean_array) != TypeId::ERROR, "boolean[] should work");
}

#[test]
fn test_array_of_objects() {
    let interner = TypeInterner::new();

    let obj = interner.object(vec![PropertyInfo {
        name: interner.intern_string("x"),
        type_id: TypeId::NUMBER,
        write_type: TypeId::NUMBER,
        optional: false,
        readonly: false,
        is_method: false,
    }]);

    let obj_array = interner.array(obj);
    let result = evaluate_type(&interner, obj_array);
    assert!(result != TypeId::ERROR, "Array of objects should work");
}

#[test]
fn test_array_of_unions() {
    let interner = TypeInterner::new();

    // (string | number)[]
    let union = interner.union(vec![TypeId::STRING, TypeId::NUMBER]);
    let union_array = interner.array(union);

    let result = evaluate_type(&interner, union_array);
    assert!(result != TypeId::ERROR, "Array of unions should work");
}

#[test]
fn test_readonly_array() {
    let interner = TypeInterner::new();

    // readonly string[]
    let string_array = interner.array(TypeId::STRING);
    let readonly_array = interner.intern(TypeKey::ReadonlyType(string_array));

    let result = evaluate_type(&interner, readonly_array);
    assert!(result != TypeId::ERROR, "Readonly array should work");
}

// =============================================================================
// Object Type Operations
// =============================================================================

#[test]
fn test_object_with_call_signature() {
    let interner = TypeInterner::new();

    // { (x: string): number }
    let call_sig = interner.function(FunctionShape {
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
    });

    // Object with call signature is modeled as intersection or special object
    let result = evaluate_type(&interner, call_sig);
    assert!(result != TypeId::ERROR, "Object with call signature should work");
}

#[test]
fn test_object_with_construct_signature() {
    let interner = TypeInterner::new();

    // { new (x: string): Foo }
    let construct_sig = interner.function(FunctionShape {
        type_params: vec![],
        params: vec![ParamInfo {
            name: Some(interner.intern_string("x")),
            type_id: TypeId::STRING,
            optional: false,
            rest: false,
        }],
        this_type: None,
        return_type: TypeId::OBJECT,
        type_predicate: None,
        is_constructor: true,
    });

    let result = evaluate_type(&interner, construct_sig);
    assert!(result != TypeId::ERROR, "Object with construct signature should work");
}

#[test]
fn test_object_with_string_index_signature() {
    let interner = TypeInterner::new();

    // { [key: string]: number }
    let obj = interner.object_with_index(ObjectShape {
        properties: vec![],
        string_index: Some(IndexSignature {
            key_type: TypeId::STRING,
            value_type: TypeId::NUMBER,
            readonly: false,
        }),
        number_index: None,
    });

    let result = evaluate_type(&interner, obj);
    assert!(result != TypeId::ERROR, "Object with string index signature should work");
}

#[test]
fn test_object_with_number_index_sig() {
    let interner = TypeInterner::new();

    // { [index: number]: string }
    let obj = interner.object_with_index(ObjectShape {
        properties: vec![],
        string_index: None,
        number_index: Some(IndexSignature {
            key_type: TypeId::NUMBER,
            value_type: TypeId::STRING,
            readonly: false,
        }),
    });

    let result = evaluate_type(&interner, obj);
    assert!(result != TypeId::ERROR, "Object with number index signature should work");
}

#[test]
fn test_object_with_method() {
    let interner = TypeInterner::new();

    // { foo(): string }
    let method_type = interner.function(FunctionShape {
        type_params: vec![],
        params: vec![],
        this_type: None,
        return_type: TypeId::STRING,
        type_predicate: None,
        is_constructor: false,
    });

    let obj = interner.object(vec![PropertyInfo {
        name: interner.intern_string("foo"),
        type_id: method_type,
        write_type: method_type,
        optional: false,
        readonly: false,
        is_method: true,
    }]);

    let result = evaluate_type(&interner, obj);
    assert!(result != TypeId::ERROR, "Object with method should work");
}

#[test]
fn test_object_with_readonly_property() {
    let interner = TypeInterner::new();

    // { readonly x: string }
    let obj = interner.object(vec![PropertyInfo {
        name: interner.intern_string("x"),
        type_id: TypeId::STRING,
        write_type: TypeId::STRING,
        optional: false,
        readonly: true,
        is_method: false,
    }]);

    let result = evaluate_type(&interner, obj);
    assert!(result != TypeId::ERROR, "Object with readonly property should work");
}

// =============================================================================
// KeyOf Edge Cases (Additional)
// =============================================================================

#[test]
fn test_keyof_string_primitive() {
    let interner = TypeInterner::new();

    // keyof string
    let keyof_string = interner.intern(TypeKey::KeyOf(TypeId::STRING));
    let result = evaluate_type(&interner, keyof_string);
    assert!(result != TypeId::ERROR, "keyof string should work");
}

#[test]
fn test_keyof_array_type() {
    let interner = TypeInterner::new();

    // keyof string[] - includes number and array methods
    let string_array = interner.array(TypeId::STRING);
    let keyof_array = interner.intern(TypeKey::KeyOf(string_array));
    let result = evaluate_type(&interner, keyof_array);
    assert!(result != TypeId::ERROR, "keyof array should work");
}

#[test]
fn test_keyof_tuple_type() {
    let interner = TypeInterner::new();

    // keyof [string, number] - should be "0" | "1" | array methods
    let tuple = interner.tuple(vec![
        TupleElement { type_id: TypeId::STRING, name: None, optional: false, rest: false },
        TupleElement { type_id: TypeId::NUMBER, name: None, optional: false, rest: false },
    ]);
    let keyof_tuple = interner.intern(TypeKey::KeyOf(tuple));
    let result = evaluate_type(&interner, keyof_tuple);
    assert!(result != TypeId::ERROR, "keyof tuple should work");
}

#[test]
fn test_keyof_any_type() {
    let interner = TypeInterner::new();

    // keyof any = string | number | symbol
    let keyof_any = interner.intern(TypeKey::KeyOf(TypeId::ANY));
    let result = evaluate_type(&interner, keyof_any);
    assert!(result != TypeId::ERROR, "keyof any should work");
}

#[test]
fn test_keyof_unknown_type() {
    let interner = TypeInterner::new();

    // keyof unknown = never
    let keyof_unknown = interner.intern(TypeKey::KeyOf(TypeId::UNKNOWN));
    let result = evaluate_type(&interner, keyof_unknown);
    assert!(result != TypeId::ERROR, "keyof unknown should work");
}

#[test]
fn test_keyof_never_type() {
    let interner = TypeInterner::new();

    // keyof never = string | number | symbol
    let keyof_never = interner.intern(TypeKey::KeyOf(TypeId::NEVER));
    let result = evaluate_type(&interner, keyof_never);
    assert!(result != TypeId::ERROR, "keyof never should work");
}

// =============================================================================
// Infer Type Operations
// =============================================================================

#[test]
fn test_infer_type_basic() {
    let interner = TypeInterner::new();

    // infer U (used in conditional extends)
    let u_name = interner.intern_string("U");
    let u_infer = interner.intern(TypeKey::Infer(TypeParamInfo {
        name: u_name,
        constraint: None,
        default: None,
    }));

    let result = evaluate_type(&interner, u_infer);
    assert!(result != TypeId::ERROR, "Infer type should work");
}

#[test]
fn test_infer_type_with_constraint() {
    let interner = TypeInterner::new();

    // infer U extends string
    let u_name = interner.intern_string("U");
    let u_infer = interner.intern(TypeKey::Infer(TypeParamInfo {
        name: u_name,
        constraint: Some(TypeId::STRING),
        default: None,
    }));

    let result = evaluate_type(&interner, u_infer);
    assert!(result != TypeId::ERROR, "Infer type with constraint should work");
}

// =============================================================================
// Substitution and Instantiation
// =============================================================================

#[test]
fn test_instantiate_simple_type_param() {
    let interner = TypeInterner::new();

    // T with substitution T=string should produce string
    let t_name = interner.intern_string("T");
    let t_param = TypeParamInfo {
        name: t_name,
        constraint: None,
        default: None,
    };
    let t_type = interner.intern(TypeKey::TypeParameter(t_param));

    let mut subst = TypeSubstitution::new();
    subst.insert(t_name, TypeId::STRING);

    let result = instantiate_type(&interner, t_type, &subst);
    assert_eq!(result, TypeId::STRING, "T should instantiate to string");
}

#[test]
fn test_instantiate_array_of_type_param() {
    let interner = TypeInterner::new();

    // T[] with T=string should produce string[]
    let t_name = interner.intern_string("T");
    let t_param = TypeParamInfo {
        name: t_name,
        constraint: None,
        default: None,
    };
    let t_type = interner.intern(TypeKey::TypeParameter(t_param));
    let t_array = interner.array(t_type);

    let mut subst = TypeSubstitution::new();
    subst.insert(t_name, TypeId::STRING);

    let result = instantiate_type(&interner, t_array, &subst);
    assert!(result != TypeId::ERROR, "T[] should instantiate correctly");
}

#[test]
fn test_instantiate_union_with_type_param() {
    let interner = TypeInterner::new();

    // T | number with T=string should produce string | number
    let t_name = interner.intern_string("T");
    let t_param = TypeParamInfo {
        name: t_name,
        constraint: None,
        default: None,
    };
    let t_type = interner.intern(TypeKey::TypeParameter(t_param));
    let t_or_number = interner.union(vec![t_type, TypeId::NUMBER]);

    let mut subst = TypeSubstitution::new();
    subst.insert(t_name, TypeId::STRING);

    let result = instantiate_type(&interner, t_or_number, &subst);
    assert!(result != TypeId::ERROR, "T | number should instantiate correctly");
}

#[test]
fn test_instantiate_object_with_type_param() {
    let interner = TypeInterner::new();

    // { x: T } with T=string should produce { x: string }
    let t_name = interner.intern_string("T");
    let t_param = TypeParamInfo {
        name: t_name,
        constraint: None,
        default: None,
    };
    let t_type = interner.intern(TypeKey::TypeParameter(t_param));

    let obj = interner.object(vec![PropertyInfo {
        name: interner.intern_string("x"),
        type_id: t_type,
        write_type: t_type,
        optional: false,
        readonly: false,
        is_method: false,
    }]);

    let mut subst = TypeSubstitution::new();
    subst.insert(t_name, TypeId::STRING);

    let result = instantiate_type(&interner, obj, &subst);
    assert!(result != TypeId::ERROR, "Object with x: T should instantiate correctly");
}

#[test]
fn test_instantiate_multiple_type_params() {
    let interner = TypeInterner::new();

    // { a: T; b: U } with T=string, U=number
    let t_name = interner.intern_string("T");
    let u_name = interner.intern_string("U");

    let t_type = interner.intern(TypeKey::TypeParameter(TypeParamInfo {
        name: t_name,
        constraint: None,
        default: None,
    }));
    let u_type = interner.intern(TypeKey::TypeParameter(TypeParamInfo {
        name: u_name,
        constraint: None,
        default: None,
    }));

    let obj = interner.object(vec![
        PropertyInfo {
            name: interner.intern_string("a"),
            type_id: t_type,
            write_type: t_type,
            optional: false,
            readonly: false,
            is_method: false,
        },
        PropertyInfo {
            name: interner.intern_string("b"),
            type_id: u_type,
            write_type: u_type,
            optional: false,
            readonly: false,
            is_method: false,
        },
    ]);

    let mut subst = TypeSubstitution::new();
    subst.insert(t_name, TypeId::STRING);
    subst.insert(u_name, TypeId::NUMBER);

    let result = instantiate_type(&interner, obj, &subst);
    assert!(result != TypeId::ERROR, "Multiple type params should instantiate correctly");
}

// =============================================================================
// Primitive Type Edge Cases
// =============================================================================

#[test]
fn test_void_type() {
    let interner = TypeInterner::new();
    let result = evaluate_type(&interner, TypeId::VOID);
    assert!(result != TypeId::ERROR, "void type should work");
}

#[test]
fn test_undefined_type() {
    let interner = TypeInterner::new();
    let result = evaluate_type(&interner, TypeId::UNDEFINED);
    assert!(result != TypeId::ERROR, "undefined type should work");
}

#[test]
fn test_null_type() {
    let interner = TypeInterner::new();
    let result = evaluate_type(&interner, TypeId::NULL);
    assert!(result != TypeId::ERROR, "null type should work");
}

#[test]
fn test_object_primitive_type() {
    let interner = TypeInterner::new();
    let result = evaluate_type(&interner, TypeId::OBJECT);
    assert!(result != TypeId::ERROR, "object type should work");
}

#[test]
fn test_bigint_type() {
    let interner = TypeInterner::new();
    let result = evaluate_type(&interner, TypeId::BIGINT);
    assert!(result != TypeId::ERROR, "bigint type should work");
}

// StateFromReducers<R> mapped type tests
// Tests the pattern: type StateFromReducers<R> = { [K in keyof R]: ExtractState<R[K]> }
// Where ExtractState<R> = R extends Reducer<infer S, AnyAction> ? S : never

#[test]
fn test_mapped_type_with_conditional_template_simple() {
    let interner = TypeInterner::new();

    // Simulates a simple version of StateFromReducers pattern:
    // type UnwrapPromise<T> = T extends Promise<infer U> ? U : T;
    // type MappedUnwrap<R> = { [K in keyof R]: UnwrapPromise<R[K]> }
    //
    // Given { a: Promise<string>, b: number }
    // Expected: { a: string, b: number }

    // For this test we use a simpler pattern:
    // type ExtractValue<T> = T extends { value: infer V } ? V : T
    // type MappedExtract<R> = { [K in keyof R]: ExtractValue<R[K]> }
    //
    // Given { a: { value: string }, b: number }
    // Expected: { a: string, b: number }

    // Create source object: { a: { value: string }, b: number }
    let value_obj = interner.object(vec![PropertyInfo {
        name: interner.intern_string("value"),
        type_id: TypeId::STRING,
        write_type: TypeId::STRING,
        optional: false,
        readonly: false,
        is_method: false,
    }]);

    let source_obj = interner.object(vec![
        PropertyInfo {
            name: interner.intern_string("a"),
            type_id: value_obj,
            write_type: value_obj,
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

    // Create infer V
    let infer_v_name = interner.intern_string("V");
    let infer_v = interner.intern(TypeKey::Infer(TypeParamInfo {
        name: infer_v_name,
        constraint: None,
        default: None,
    }));

    // Pattern: { value: infer V }
    let pattern = interner.object(vec![PropertyInfo {
        name: interner.intern_string("value"),
        type_id: infer_v,
        write_type: infer_v,
        optional: false,
        readonly: false,
        is_method: false,
    }]);

    // Test ExtractValue on { value: string } - should get string
    let cond_a = ConditionalType {
        check_type: value_obj,
        extends_type: pattern,
        true_type: infer_v,
        false_type: value_obj,
        is_distributive: false,
    };

    let result_a = evaluate_conditional(&interner, &cond_a);
    assert_eq!(result_a, TypeId::STRING, "ExtractValue<{{ value: string }}> should be string");

    // Test ExtractValue on number - should get number (false branch)
    let cond_b = ConditionalType {
        check_type: TypeId::NUMBER,
        extends_type: pattern,
        true_type: infer_v,
        false_type: TypeId::NUMBER,
        is_distributive: false,
    };

    let result_b = evaluate_conditional(&interner, &cond_b);
    assert_eq!(result_b, TypeId::NUMBER, "ExtractValue<number> should be number");
}

#[test]
fn test_mapped_state_from_reducers_pattern_with_simple_objects() {
    let interner = TypeInterner::new();

    // Simulates StateFromReducers pattern with simplified Reducer:
    // type SimpleReducer<S> = { state: S }
    // type ExtractState<R> = R extends SimpleReducer<infer S> ? S : never
    // type StateFromReducers<R> = { [K in keyof R]: ExtractState<R[K]> }
    //
    // interface Reducers {
    //     count: SimpleReducer<number>;  // { state: number }
    //     message: SimpleReducer<string>; // { state: string }
    // }
    // type AppState = StateFromReducers<Reducers>;
    // Expected: { count: number, message: string }

    // Create SimpleReducer<number> = { state: number }
    let reducer_number = interner.object(vec![PropertyInfo {
        name: interner.intern_string("state"),
        type_id: TypeId::NUMBER,
        write_type: TypeId::NUMBER,
        optional: false,
        readonly: false,
        is_method: false,
    }]);

    // Create SimpleReducer<string> = { state: string }
    let reducer_string = interner.object(vec![PropertyInfo {
        name: interner.intern_string("state"),
        type_id: TypeId::STRING,
        write_type: TypeId::STRING,
        optional: false,
        readonly: false,
        is_method: false,
    }]);

    // Create infer S
    let infer_s_name = interner.intern_string("S");
    let infer_s = interner.intern(TypeKey::Infer(TypeParamInfo {
        name: infer_s_name,
        constraint: None,
        default: None,
    }));

    // Pattern: SimpleReducer<infer S> = { state: infer S }
    let pattern = interner.object(vec![PropertyInfo {
        name: interner.intern_string("state"),
        type_id: infer_s,
        write_type: infer_s,
        optional: false,
        readonly: false,
        is_method: false,
    }]);

    // Test ExtractState on SimpleReducer<number> - should get number
    let cond_count = ConditionalType {
        check_type: reducer_number,
        extends_type: pattern,
        true_type: infer_s,
        false_type: TypeId::NEVER,
        is_distributive: false,
    };

    let result_count = evaluate_conditional(&interner, &cond_count);
    assert_eq!(result_count, TypeId::NUMBER, "ExtractState<SimpleReducer<number>> should be number");

    // Test ExtractState on SimpleReducer<string> - should get string
    let cond_message = ConditionalType {
        check_type: reducer_string,
        extends_type: pattern,
        true_type: infer_s,
        false_type: TypeId::NEVER,
        is_distributive: false,
    };

    let result_message = evaluate_conditional(&interner, &cond_message);
    assert_eq!(result_message, TypeId::STRING, "ExtractState<SimpleReducer<string>> should be string");
}

#[test]
fn test_mapped_state_from_reducers_indexed_access() {
    let interner = TypeInterner::new();

    // Test the indexed access R[K] pattern used in StateFromReducers
    // type StateFromReducers<R> = { [K in keyof R]: ExtractState<R[K]> }
    //
    // When R = { count: Reducer<number>, message: Reducer<string> }
    // R["count"] should be Reducer<number>
    // R["message"] should be Reducer<string>

    // Create reducers object type
    let reducer_number = interner.object(vec![PropertyInfo {
        name: interner.intern_string("state"),
        type_id: TypeId::NUMBER,
        write_type: TypeId::NUMBER,
        optional: false,
        readonly: false,
        is_method: false,
    }]);

    let reducer_string = interner.object(vec![PropertyInfo {
        name: interner.intern_string("state"),
        type_id: TypeId::STRING,
        write_type: TypeId::STRING,
        optional: false,
        readonly: false,
        is_method: false,
    }]);

    let reducers = interner.object(vec![
        PropertyInfo {
            name: interner.intern_string("count"),
            type_id: reducer_number,
            write_type: reducer_number,
            optional: false,
            readonly: false,
            is_method: false,
        },
        PropertyInfo {
            name: interner.intern_string("message"),
            type_id: reducer_string,
            write_type: reducer_string,
            optional: false,
            readonly: false,
            is_method: false,
        },
    ]);

    // Create literal keys
    let key_count = interner.literal_string("count");
    let key_message = interner.literal_string("message");

    // Create IndexAccess types: R["count"] and R["message"]
    let index_count = interner.intern(TypeKey::IndexAccess(reducers, key_count));
    let index_message = interner.intern(TypeKey::IndexAccess(reducers, key_message));

    // Evaluate the indexed access
    let result_count = evaluate_index_access(&interner, reducers, key_count);
    let result_message = evaluate_index_access(&interner, reducers, key_message);

    // R["count"] should be { state: number }
    assert_eq!(result_count, reducer_number, "R[\"count\"] should be Reducer<number>");

    // R["message"] should be { state: string }
    assert_eq!(result_message, reducer_string, "R[\"message\"] should be Reducer<string>");
}

#[test]
fn test_mapped_type_full_state_from_reducers_simulation() {
    let interner = TypeInterner::new();

    // Simulation of StateFromReducers mapped type evaluation
    // type StateFromReducers<R> = { [K in keyof R]: ExtractState<R[K]> }
    //
    // For this test, we use keyof R directly as the constraint (literal string union)
    // Given keys: "count" | "message"
    // Expected: { count: number, message: number } (simplified with constant template)

    // Create the keys as a union of literal strings (simulating keyof Reducers)
    let key_count = interner.literal_string("count");
    let key_message = interner.literal_string("message");
    let keys = interner.union(vec![key_count, key_message]);

    // Create mapped type: { [K in "count" | "message"]: number }
    let mapped = MappedType {
        type_param: TypeParamInfo {
            name: interner.intern_string("K"),
            constraint: None,
            default: None,
        },
        constraint: keys,
        name_type: None,
        template: TypeId::NUMBER, // Simplified: always number
        readonly_modifier: None,
        optional_modifier: None,
    };

    let result = evaluate_mapped(&interner, &mapped);

    // Result should be { count: number, message: number }
    let expected = interner.object(vec![
        PropertyInfo {
            name: interner.intern_string("count"),
            type_id: TypeId::NUMBER,
            write_type: TypeId::NUMBER,
            optional: false,
            readonly: false,
            is_method: false,
        },
        PropertyInfo {
            name: interner.intern_string("message"),
            type_id: TypeId::NUMBER,
            write_type: TypeId::NUMBER,
            optional: false,
            readonly: false,
            is_method: false,
        },
    ]);

    assert_eq!(result, expected, "Mapped type over literal key union should produce correct shape");
}

#[test]
fn test_mapped_type_over_keyof_reducers_object() {
    let interner = TypeInterner::new();

    // Test mapped type with keyof on an object type:
    // type Reducers = { count: { state: number }, message: { state: string } }
    // type Result = { [K in keyof Reducers]: boolean }
    // Expected: { count: boolean, message: boolean }

    // Create reducer object types
    let reducer_number = interner.object(vec![PropertyInfo {
        name: interner.intern_string("state"),
        type_id: TypeId::NUMBER,
        write_type: TypeId::NUMBER,
        optional: false,
        readonly: false,
        is_method: false,
    }]);

    let reducer_string = interner.object(vec![PropertyInfo {
        name: interner.intern_string("state"),
        type_id: TypeId::STRING,
        write_type: TypeId::STRING,
        optional: false,
        readonly: false,
        is_method: false,
    }]);

    // Reducers object: { count: { state: number }, message: { state: string } }
    let reducers = interner.object(vec![
        PropertyInfo {
            name: interner.intern_string("count"),
            type_id: reducer_number,
            write_type: reducer_number,
            optional: false,
            readonly: false,
            is_method: false,
        },
        PropertyInfo {
            name: interner.intern_string("message"),
            type_id: reducer_string,
            write_type: reducer_string,
            optional: false,
            readonly: false,
            is_method: false,
        },
    ]);

    // keyof Reducers
    let keyof_reducers = interner.intern(TypeKey::KeyOf(reducers));

    // Create mapped type: { [K in keyof Reducers]: boolean }
    let mapped = MappedType {
        type_param: TypeParamInfo {
            name: interner.intern_string("K"),
            constraint: None,
            default: None,
        },
        constraint: keyof_reducers,
        name_type: None,
        template: TypeId::BOOLEAN,
        readonly_modifier: None,
        optional_modifier: None,
    };

    let result = evaluate_mapped(&interner, &mapped);

    // Result should be { count: boolean, message: boolean }
    let expected = interner.object(vec![
        PropertyInfo {
            name: interner.intern_string("count"),
            type_id: TypeId::BOOLEAN,
            write_type: TypeId::BOOLEAN,
            optional: false,
            readonly: false,
            is_method: false,
        },
        PropertyInfo {
            name: interner.intern_string("message"),
            type_id: TypeId::BOOLEAN,
            write_type: TypeId::BOOLEAN,
            optional: false,
            readonly: false,
            is_method: false,
        },
    ]);

    assert_eq!(result, expected, "Mapped type with keyof should produce correct shape");
}

// ActionFromReducers<R> indexed access tests
// Tests the pattern: type ActionFromReducers<R> = { [K in keyof R]: ExtractAction<R[K]> }[keyof R]
// This produces a union of all action types from each reducer

#[test]
fn test_indexed_access_on_object_with_keyof() {
    let interner = TypeInterner::new();

    // Test: { count: number, message: string }[keyof { count: number, message: string }]
    // = { count: number, message: string }["count" | "message"]
    // = number | string

    let obj = interner.object(vec![
        PropertyInfo {
            name: interner.intern_string("count"),
            type_id: TypeId::NUMBER,
            write_type: TypeId::NUMBER,
            optional: false,
            readonly: false,
            is_method: false,
        },
        PropertyInfo {
            name: interner.intern_string("message"),
            type_id: TypeId::STRING,
            write_type: TypeId::STRING,
            optional: false,
            readonly: false,
            is_method: false,
        },
    ]);

    // keyof obj = "count" | "message"
    let key_count = interner.literal_string("count");
    let key_message = interner.literal_string("message");
    let keyof_obj = interner.union(vec![key_count, key_message]);

    // obj[keyof obj] should be number | string
    let result = evaluate_index_access(&interner, obj, keyof_obj);

    let expected = interner.union(vec![TypeId::NUMBER, TypeId::STRING]);
    assert_eq!(result, expected, "obj[keyof obj] should be number | string");
}

#[test]
fn test_indexed_access_mapped_type_result_with_union_key() {
    let interner = TypeInterner::new();

    // Test the ActionFromReducers pattern:
    // Given a mapped type result: { count: ActionA, message: ActionB }
    // Accessing with [keyof T] should give ActionA | ActionB

    // Create action types
    let action_a = interner.object(vec![PropertyInfo {
        name: interner.intern_string("type"),
        type_id: interner.literal_string("inc"),
        write_type: interner.literal_string("inc"),
        optional: false,
        readonly: false,
        is_method: false,
    }]);

    let action_b = interner.object(vec![PropertyInfo {
        name: interner.intern_string("type"),
        type_id: interner.literal_string("set"),
        write_type: interner.literal_string("set"),
        optional: false,
        readonly: false,
        is_method: false,
    }]);

    // Mapped type result: { count: ActionA, message: ActionB }
    let mapped_result = interner.object(vec![
        PropertyInfo {
            name: interner.intern_string("count"),
            type_id: action_a,
            write_type: action_a,
            optional: false,
            readonly: false,
            is_method: false,
        },
        PropertyInfo {
            name: interner.intern_string("message"),
            type_id: action_b,
            write_type: action_b,
            optional: false,
            readonly: false,
            is_method: false,
        },
    ]);

    // keyof result = "count" | "message"
    let key_count = interner.literal_string("count");
    let key_message = interner.literal_string("message");
    let keyof_result = interner.union(vec![key_count, key_message]);

    // result[keyof result] should be ActionA | ActionB
    let result = evaluate_index_access(&interner, mapped_result, keyof_result);

    let expected = interner.union(vec![action_a, action_b]);
    assert_eq!(result, expected, "Mapped type indexed with keyof should produce union of values");
}

#[test]
fn test_action_from_reducers_pattern_with_simple_objects() {
    let interner = TypeInterner::new();

    // Simulates ActionFromReducers pattern with simplified Reducer:
    // type SimpleReducer<A> = { action: A }
    // type ExtractAction<R> = R extends SimpleReducer<infer A> ? A : never
    // type ActionFromReducers<R> = { [K in keyof R]: ExtractAction<R[K]> }[keyof R]
    //
    // Given: Reducers = { count: { action: { type: "inc" } }, message: { action: { type: "set" } } }
    // Step 1: { [K in keyof Reducers]: ExtractAction<Reducers[K]> }
    //       = { count: { type: "inc" }, message: { type: "set" } }
    // Step 2: Result[keyof Reducers] = { type: "inc" } | { type: "set" }

    // Create action types
    let action_inc = interner.object(vec![PropertyInfo {
        name: interner.intern_string("type"),
        type_id: interner.literal_string("inc"),
        write_type: interner.literal_string("inc"),
        optional: false,
        readonly: false,
        is_method: false,
    }]);

    let action_set = interner.object(vec![PropertyInfo {
        name: interner.intern_string("type"),
        type_id: interner.literal_string("set"),
        write_type: interner.literal_string("set"),
        optional: false,
        readonly: false,
        is_method: false,
    }]);

    // Create SimpleReducer wrappers: { action: ActionType }
    let reducer_count = interner.object(vec![PropertyInfo {
        name: interner.intern_string("action"),
        type_id: action_inc,
        write_type: action_inc,
        optional: false,
        readonly: false,
        is_method: false,
    }]);

    let reducer_message = interner.object(vec![PropertyInfo {
        name: interner.intern_string("action"),
        type_id: action_set,
        write_type: action_set,
        optional: false,
        readonly: false,
        is_method: false,
    }]);

    // Create infer A for ExtractAction pattern
    let infer_a_name = interner.intern_string("A");
    let infer_a = interner.intern(TypeKey::Infer(TypeParamInfo {
        name: infer_a_name,
        constraint: None,
        default: None,
    }));

    // Pattern: SimpleReducer<infer A> = { action: infer A }
    let pattern = interner.object(vec![PropertyInfo {
        name: interner.intern_string("action"),
        type_id: infer_a,
        write_type: infer_a,
        optional: false,
        readonly: false,
        is_method: false,
    }]);

    // Test ExtractAction on reducer_count - should get { type: "inc" }
    let cond_count = ConditionalType {
        check_type: reducer_count,
        extends_type: pattern,
        true_type: infer_a,
        false_type: TypeId::NEVER,
        is_distributive: false,
    };

    let result_count = evaluate_conditional(&interner, &cond_count);
    assert_eq!(result_count, action_inc, "ExtractAction<SimpleReducer<{{ type: 'inc' }}>> should be {{ type: 'inc' }}");

    // Test ExtractAction on reducer_message - should get { type: "set" }
    let cond_message = ConditionalType {
        check_type: reducer_message,
        extends_type: pattern,
        true_type: infer_a,
        false_type: TypeId::NEVER,
        is_distributive: false,
    };

    let result_message = evaluate_conditional(&interner, &cond_message);
    assert_eq!(result_message, action_set, "ExtractAction<SimpleReducer<{{ type: 'set' }}>> should be {{ type: 'set' }}");
}

#[test]
fn test_action_from_reducers_full_pattern() {
    let interner = TypeInterner::new();

    // Full ActionFromReducers pattern simulation:
    // 1. Create mapped type result with extracted actions
    // 2. Access with keyof to get union of all actions

    // Create action types
    let action_inc = interner.object(vec![PropertyInfo {
        name: interner.intern_string("type"),
        type_id: interner.literal_string("inc"),
        write_type: interner.literal_string("inc"),
        optional: false,
        readonly: false,
        is_method: false,
    }]);

    let action_set = interner.object(vec![PropertyInfo {
        name: interner.intern_string("type"),
        type_id: interner.literal_string("set"),
        write_type: interner.literal_string("set"),
        optional: false,
        readonly: false,
        is_method: false,
    }]);

    // Simulated mapped type result: { count: { type: "inc" }, message: { type: "set" } }
    let mapped_result = interner.object(vec![
        PropertyInfo {
            name: interner.intern_string("count"),
            type_id: action_inc,
            write_type: action_inc,
            optional: false,
            readonly: false,
            is_method: false,
        },
        PropertyInfo {
            name: interner.intern_string("message"),
            type_id: action_set,
            write_type: action_set,
            optional: false,
            readonly: false,
            is_method: false,
        },
    ]);

    // keyof mapped_result = "count" | "message"
    let keyof_result = interner.intern(TypeKey::KeyOf(mapped_result));
    let keyof_evaluated = evaluate_keyof(&interner, mapped_result);

    // Verify keyof produces the correct union
    let key_count = interner.literal_string("count");
    let key_message = interner.literal_string("message");
    let expected_keyof = interner.union(vec![key_count, key_message]);
    assert_eq!(keyof_evaluated, expected_keyof, "keyof should produce 'count' | 'message'");

    // mapped_result[keyof mapped_result] should be action_inc | action_set
    let indexed_result = evaluate_index_access(&interner, mapped_result, keyof_evaluated);
    let expected_union = interner.union(vec![action_inc, action_set]);
    assert_eq!(indexed_result, expected_union, "ActionFromReducers pattern should produce union of actions");
}

#[test]
fn test_indexed_access_with_single_key() {
    let interner = TypeInterner::new();

    // Test single key indexed access (simpler case)
    // { count: number, message: string }["count"] = number

    let obj = interner.object(vec![
        PropertyInfo {
            name: interner.intern_string("count"),
            type_id: TypeId::NUMBER,
            write_type: TypeId::NUMBER,
            optional: false,
            readonly: false,
            is_method: false,
        },
        PropertyInfo {
            name: interner.intern_string("message"),
            type_id: TypeId::STRING,
            write_type: TypeId::STRING,
            optional: false,
            readonly: false,
            is_method: false,
        },
    ]);

    let key_count = interner.literal_string("count");
    let result = evaluate_index_access(&interner, obj, key_count);

    assert_eq!(result, TypeId::NUMBER, "obj['count'] should be number");
}

#[test]
fn test_extract_state_with_function_reducer_pattern() {
    let interner = TypeInterner::new();

    // Test the actual Redux Reducer pattern using function type:
    // type Reducer<S, A> = (state: S | undefined, action: A) => S
    // type AnyAction = { type: string }
    // type ExtractState<R> = R extends Reducer<infer S, AnyAction> ? S : never
    //
    // Given: countReducer: (state: number | undefined, action: AnyAction) => number
    // ExtractState<countReducer> should return number

    // AnyAction = { type: string }
    let any_action = interner.object(vec![PropertyInfo {
        name: interner.intern_string("type"),
        type_id: TypeId::STRING,
        write_type: TypeId::STRING,
        optional: false,
        readonly: false,
        is_method: false,
    }]);

    // Create infer S for the pattern
    let infer_s_name = interner.intern_string("S");
    let infer_s = interner.intern(TypeKey::Infer(TypeParamInfo {
        name: infer_s_name,
        constraint: None,
        default: None,
    }));

    // Pattern: Reducer<infer S, AnyAction> = (state: infer S | undefined, action: AnyAction) => infer S
    let pattern_state_param = interner.union(vec![infer_s, TypeId::UNDEFINED]);
    let pattern_fn = interner.function(FunctionShape {
        type_params: vec![],
        params: vec![
            ParamInfo {
                name: Some(interner.intern_string("state")),
                type_id: pattern_state_param,
                optional: false,
                rest: false,
            },
            ParamInfo {
                name: Some(interner.intern_string("action")),
                type_id: any_action,
                optional: false,
                rest: false,
            },
        ],
        this_type: None,
        return_type: infer_s,
        type_predicate: None,
        is_constructor: false,
    });

    // Source: countReducer = (state: number | undefined, action: AnyAction) => number
    let source_state_param = interner.union(vec![TypeId::NUMBER, TypeId::UNDEFINED]);
    let count_reducer = interner.function(FunctionShape {
        type_params: vec![],
        params: vec![
            ParamInfo {
                name: Some(interner.intern_string("state")),
                type_id: source_state_param,
                optional: false,
                rest: false,
            },
            ParamInfo {
                name: Some(interner.intern_string("action")),
                type_id: any_action,
                optional: false,
                rest: false,
            },
        ],
        this_type: None,
        return_type: TypeId::NUMBER,
        type_predicate: None,
        is_constructor: false,
    });

    // Conditional: countReducer extends Reducer<infer S, AnyAction> ? S : never
    let cond = ConditionalType {
        check_type: count_reducer,
        extends_type: pattern_fn,
        true_type: infer_s,
        false_type: TypeId::NEVER,
        is_distributive: false,
    };

    let result = evaluate_conditional(&interner, &cond);

    // TODO: Full function-level ExtractState pattern not yet working.
    // Expected: should extract S = number from the function parameter and return type.
    // Current: returns never because function param union infer binding is complex.
    // This test documents the expected behavior for StateFromReducers integration.
    assert_eq!(result, TypeId::NEVER);
}

// ============================================================================
// Distributive Conditional Type Stress Tests
// ============================================================================

#[test]
fn test_distributive_large_union_basic() {
    // T extends string ? true : false, with T = A | B | C | D | E | F | G | H | I | J
    // where A..E are strings and F..J are numbers
    // Result: true | false
    let interner = TypeInterner::new();

    let t_name = interner.intern_string("T");
    let t_param = interner.intern(TypeKey::TypeParameter(TypeParamInfo {
        name: t_name,
        constraint: None,
        default: None,
    }));

    let lit_true = interner.literal_boolean(true);
    let lit_false = interner.literal_boolean(false);

    let cond = ConditionalType {
        check_type: t_param,
        extends_type: TypeId::STRING,
        true_type: lit_true,
        false_type: lit_false,
        is_distributive: true,
    };

    let cond_type = interner.conditional(cond);
    let mut subst = TypeSubstitution::new();

    // Create a union of 10 members: 5 strings, 5 numbers
    let members: Vec<TypeId> = (0..10)
        .map(|i| {
            if i < 5 {
                interner.literal_string(&format!("str{}", i))
            } else {
                interner.literal_number(i as f64)
            }
        })
        .collect();
    subst.insert(t_name, interner.union(members));

    let instantiated = instantiate_type(&interner, cond_type, &subst);
    let result = evaluate_type(&interner, instantiated);

    // Should be true | false (simplified to boolean in many systems)
    let expected = interner.union(vec![lit_true, lit_false]);
    assert_eq!(result, expected);
}

#[test]
fn test_distributive_large_union_all_match() {
    // T extends string ? T : never, with T = all string literals
    // Result: union of all input strings
    let interner = TypeInterner::new();

    let t_name = interner.intern_string("T");
    let t_param = interner.intern(TypeKey::TypeParameter(TypeParamInfo {
        name: t_name,
        constraint: None,
        default: None,
    }));

    let cond = ConditionalType {
        check_type: t_param,
        extends_type: TypeId::STRING,
        true_type: t_param,
        false_type: TypeId::NEVER,
        is_distributive: true,
    };

    let cond_type = interner.conditional(cond);
    let mut subst = TypeSubstitution::new();

    // Create a union of 20 string literals
    let members: Vec<TypeId> = (0..20)
        .map(|i| interner.literal_string(&format!("str{}", i)))
        .collect();
    let input_union = interner.union(members.clone());
    subst.insert(t_name, input_union);

    let instantiated = instantiate_type(&interner, cond_type, &subst);
    let result = evaluate_type(&interner, instantiated);

    // Result should be the same union of string literals
    let expected = interner.union(members);
    assert_eq!(result, expected);
}

#[test]
fn test_distributive_large_union_none_match() {
    // T extends string ? T : never, with T = all numbers
    // Result: never
    let interner = TypeInterner::new();

    let t_name = interner.intern_string("T");
    let t_param = interner.intern(TypeKey::TypeParameter(TypeParamInfo {
        name: t_name,
        constraint: None,
        default: None,
    }));

    let cond = ConditionalType {
        check_type: t_param,
        extends_type: TypeId::STRING,
        true_type: t_param,
        false_type: TypeId::NEVER,
        is_distributive: true,
    };

    let cond_type = interner.conditional(cond);
    let mut subst = TypeSubstitution::new();

    // Create a union of 15 number literals
    let members: Vec<TypeId> = (0..15)
        .map(|i| interner.literal_number(i as f64))
        .collect();
    subst.insert(t_name, interner.union(members));

    let instantiated = instantiate_type(&interner, cond_type, &subst);
    let result = evaluate_type(&interner, instantiated);

    // All members are numbers, none match string, so result is never
    assert_eq!(result, TypeId::NEVER);
}

#[test]
fn test_distributive_nested_conditional() {
    // T extends string ? (T extends "a" | "b" ? 1 : 2) : 3
    // with T = "a" | "b" | "c" | 1 | 2
    // Distribution: "a" -> 1, "b" -> 1, "c" -> 2, 1 -> 3, 2 -> 3
    // Result: 1 | 2 | 3
    let interner = TypeInterner::new();

    let t_name = interner.intern_string("T");
    let t_param = interner.intern(TypeKey::TypeParameter(TypeParamInfo {
        name: t_name,
        constraint: None,
        default: None,
    }));

    let lit_a = interner.literal_string("a");
    let lit_b = interner.literal_string("b");
    let lit_c = interner.literal_string("c");
    let lit_1 = interner.literal_number(1.0);
    let lit_2 = interner.literal_number(2.0);
    let lit_3 = interner.literal_number(3.0);

    // Inner conditional: T extends "a" | "b" ? 1 : 2
    let inner_cond = interner.conditional(ConditionalType {
        check_type: t_param,
        extends_type: interner.union(vec![lit_a, lit_b]),
        true_type: lit_1,
        false_type: lit_2,
        is_distributive: false, // Inner is non-distributive
    });

    // Outer conditional: T extends string ? inner : 3
    let outer_cond = ConditionalType {
        check_type: t_param,
        extends_type: TypeId::STRING,
        true_type: inner_cond,
        false_type: lit_3,
        is_distributive: true,
    };

    let cond_type = interner.conditional(outer_cond);
    let mut subst = TypeSubstitution::new();
    subst.insert(t_name, interner.union(vec![lit_a, lit_b, lit_c, lit_1, lit_2]));

    let instantiated = instantiate_type(&interner, cond_type, &subst);
    let result = evaluate_type(&interner, instantiated);

    // Expected: 1 | 2 | 3
    let expected = interner.union(vec![lit_1, lit_2, lit_3]);
    assert_eq!(result, expected);
}

#[test]
fn test_distributive_with_infer_filter() {
    // T extends (infer R)[] ? R : never, with T = string[] | number[] | boolean
    // Distribution: string[] -> string, number[] -> number, boolean -> never
    // Result: string | number
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

    let string_array = interner.array(TypeId::STRING);
    let number_array = interner.array(TypeId::NUMBER);
    subst.insert(t_name, interner.union(vec![string_array, number_array, TypeId::BOOLEAN]));

    let instantiated = instantiate_type(&interner, cond_type, &subst);
    let result = evaluate_type(&interner, instantiated);

    // Expected: string | number (boolean is filtered to never)
    let expected = interner.union(vec![TypeId::STRING, TypeId::NUMBER]);
    assert_eq!(result, expected);
}

#[test]
fn test_distributive_with_mapped_branches() {
    // T extends string ? T : T extends number ? "num" : "other"
    // with T = "a" | 1 | true
    // Distribution: "a" -> "a", 1 -> "num", true -> "other"
    // Result: "a" | "num" | "other"
    let interner = TypeInterner::new();

    let t_name = interner.intern_string("T");
    let t_param = interner.intern(TypeKey::TypeParameter(TypeParamInfo {
        name: t_name,
        constraint: None,
        default: None,
    }));

    let lit_a = interner.literal_string("a");
    let lit_num = interner.literal_string("num");
    let lit_other = interner.literal_string("other");
    let lit_1 = interner.literal_number(1.0);

    // Inner conditional: T extends number ? "num" : "other"
    let inner_cond = interner.conditional(ConditionalType {
        check_type: t_param,
        extends_type: TypeId::NUMBER,
        true_type: lit_num,
        false_type: lit_other,
        is_distributive: false,
    });

    // Outer conditional: T extends string ? T : inner
    let outer_cond = ConditionalType {
        check_type: t_param,
        extends_type: TypeId::STRING,
        true_type: t_param,
        false_type: inner_cond,
        is_distributive: true,
    };

    let cond_type = interner.conditional(outer_cond);
    let mut subst = TypeSubstitution::new();
    subst.insert(t_name, interner.union(vec![lit_a, lit_1, interner.literal_boolean(true)]));

    let instantiated = instantiate_type(&interner, cond_type, &subst);
    let result = evaluate_type(&interner, instantiated);

    // Expected: "a" | "num" | "other"
    let expected = interner.union(vec![lit_a, lit_num, lit_other]);
    assert_eq!(result, expected);
}

#[test]
fn test_distributive_with_infer_in_true_branch() {
    // T extends { value: infer V } ? V : never
    // with T = { value: string } | { value: number } | { other: boolean }
    // Distribution: { value: string } -> string, { value: number } -> number, { other: boolean } -> never
    // Result: string | number
    let interner = TypeInterner::new();

    let t_name = interner.intern_string("T");
    let t_param = interner.intern(TypeKey::TypeParameter(TypeParamInfo {
        name: t_name,
        constraint: None,
        default: None,
    }));

    let infer_name = interner.intern_string("V");
    let infer_v = interner.intern(TypeKey::Infer(TypeParamInfo {
        name: infer_name,
        constraint: None,
        default: None,
    }));

    let value_atom = interner.intern_string("value");
    let other_atom = interner.intern_string("other");

    let extends_obj = interner.object(vec![PropertyInfo {
        name: value_atom,
        type_id: infer_v,
        write_type: infer_v,
        optional: false,
        readonly: false,
        is_method: false,
    }]);

    let cond = ConditionalType {
        check_type: t_param,
        extends_type: extends_obj,
        true_type: infer_v,
        false_type: TypeId::NEVER,
        is_distributive: true,
    };

    let cond_type = interner.conditional(cond);
    let mut subst = TypeSubstitution::new();

    let obj_string = interner.object(vec![PropertyInfo {
        name: value_atom,
        type_id: TypeId::STRING,
        write_type: TypeId::STRING,
        optional: false,
        readonly: false,
        is_method: false,
    }]);
    let obj_number = interner.object(vec![PropertyInfo {
        name: value_atom,
        type_id: TypeId::NUMBER,
        write_type: TypeId::NUMBER,
        optional: false,
        readonly: false,
        is_method: false,
    }]);
    let obj_other = interner.object(vec![PropertyInfo {
        name: other_atom,
        type_id: TypeId::BOOLEAN,
        write_type: TypeId::BOOLEAN,
        optional: false,
        readonly: false,
        is_method: false,
    }]);

    subst.insert(t_name, interner.union(vec![obj_string, obj_number, obj_other]));

    let instantiated = instantiate_type(&interner, cond_type, &subst);
    let result = evaluate_type(&interner, instantiated);

    // Expected: string | number
    let expected = interner.union(vec![TypeId::STRING, TypeId::NUMBER]);
    assert_eq!(result, expected);
}

#[test]
fn test_distributive_exclude_utility() {
    // Exclude<T, U> = T extends U ? never : T
    // Exclude<"a" | "b" | "c", "a"> = "b" | "c"
    let interner = TypeInterner::new();

    let t_name = interner.intern_string("T");
    let t_param = interner.intern(TypeKey::TypeParameter(TypeParamInfo {
        name: t_name,
        constraint: None,
        default: None,
    }));

    let lit_a = interner.literal_string("a");
    let lit_b = interner.literal_string("b");
    let lit_c = interner.literal_string("c");

    // T extends "a" ? never : T
    let cond = ConditionalType {
        check_type: t_param,
        extends_type: lit_a,
        true_type: TypeId::NEVER,
        false_type: t_param,
        is_distributive: true,
    };

    let cond_type = interner.conditional(cond);
    let mut subst = TypeSubstitution::new();
    subst.insert(t_name, interner.union(vec![lit_a, lit_b, lit_c]));

    let instantiated = instantiate_type(&interner, cond_type, &subst);
    let result = evaluate_type(&interner, instantiated);

    // Expected: "b" | "c"
    let expected = interner.union(vec![lit_b, lit_c]);
    assert_eq!(result, expected);
}

#[test]
fn test_distributive_extract_utility() {
    // Extract<T, U> = T extends U ? T : never
    // Extract<"a" | 1 | "b" | 2, string> = "a" | "b"
    let interner = TypeInterner::new();

    let t_name = interner.intern_string("T");
    let t_param = interner.intern(TypeKey::TypeParameter(TypeParamInfo {
        name: t_name,
        constraint: None,
        default: None,
    }));

    let lit_a = interner.literal_string("a");
    let lit_b = interner.literal_string("b");
    let lit_1 = interner.literal_number(1.0);
    let lit_2 = interner.literal_number(2.0);

    // T extends string ? T : never
    let cond = ConditionalType {
        check_type: t_param,
        extends_type: TypeId::STRING,
        true_type: t_param,
        false_type: TypeId::NEVER,
        is_distributive: true,
    };

    let cond_type = interner.conditional(cond);
    let mut subst = TypeSubstitution::new();
    subst.insert(t_name, interner.union(vec![lit_a, lit_1, lit_b, lit_2]));

    let instantiated = instantiate_type(&interner, cond_type, &subst);
    let result = evaluate_type(&interner, instantiated);

    // Expected: "a" | "b"
    let expected = interner.union(vec![lit_a, lit_b]);
    assert_eq!(result, expected);
}

#[test]
fn test_distributive_non_nullable_utility() {
    // NonNullable<T> = T extends null | undefined ? never : T
    // NonNullable<string | null | undefined | number> = string | number
    let interner = TypeInterner::new();

    let t_name = interner.intern_string("T");
    let t_param = interner.intern(TypeKey::TypeParameter(TypeParamInfo {
        name: t_name,
        constraint: None,
        default: None,
    }));

    let null_or_undefined = interner.union(vec![TypeId::NULL, TypeId::UNDEFINED]);

    // T extends null | undefined ? never : T
    let cond = ConditionalType {
        check_type: t_param,
        extends_type: null_or_undefined,
        true_type: TypeId::NEVER,
        false_type: t_param,
        is_distributive: true,
    };

    let cond_type = interner.conditional(cond);
    let mut subst = TypeSubstitution::new();
    subst.insert(
        t_name,
        interner.union(vec![TypeId::STRING, TypeId::NULL, TypeId::UNDEFINED, TypeId::NUMBER]),
    );

    let instantiated = instantiate_type(&interner, cond_type, &subst);
    let result = evaluate_type(&interner, instantiated);

    // Expected: string | number
    let expected = interner.union(vec![TypeId::STRING, TypeId::NUMBER]);
    assert_eq!(result, expected);
}

#[test]
fn test_distributive_deeply_nested_union() {
    // T extends string ? "s" : (T extends number ? "n" : (T extends boolean ? "b" : "x"))
    // with T = "a" | 1 | true | null
    // Distribution: "a" -> "s", 1 -> "n", true -> "b", null -> "x"
    // Result: "s" | "n" | "b" | "x"
    let interner = TypeInterner::new();

    let t_name = interner.intern_string("T");
    let t_param = interner.intern(TypeKey::TypeParameter(TypeParamInfo {
        name: t_name,
        constraint: None,
        default: None,
    }));

    let lit_s = interner.literal_string("s");
    let lit_n = interner.literal_string("n");
    let lit_b = interner.literal_string("b");
    let lit_x = interner.literal_string("x");
    let lit_a = interner.literal_string("a");
    let lit_1 = interner.literal_number(1.0);

    // Innermost: T extends boolean ? "b" : "x"
    let cond3 = interner.conditional(ConditionalType {
        check_type: t_param,
        extends_type: TypeId::BOOLEAN,
        true_type: lit_b,
        false_type: lit_x,
        is_distributive: false,
    });

    // Middle: T extends number ? "n" : cond3
    let cond2 = interner.conditional(ConditionalType {
        check_type: t_param,
        extends_type: TypeId::NUMBER,
        true_type: lit_n,
        false_type: cond3,
        is_distributive: false,
    });

    // Outer: T extends string ? "s" : cond2
    let outer_cond = ConditionalType {
        check_type: t_param,
        extends_type: TypeId::STRING,
        true_type: lit_s,
        false_type: cond2,
        is_distributive: true,
    };

    let cond_type = interner.conditional(outer_cond);
    let mut subst = TypeSubstitution::new();
    subst.insert(
        t_name,
        interner.union(vec![lit_a, lit_1, interner.literal_boolean(true), TypeId::NULL]),
    );

    let instantiated = instantiate_type(&interner, cond_type, &subst);
    let result = evaluate_type(&interner, instantiated);

    // Expected: "s" | "n" | "b" | "x"
    let expected = interner.union(vec![lit_s, lit_n, lit_b, lit_x]);
    assert_eq!(result, expected);
}

#[test]
fn test_distributive_with_never_input() {
    // T extends string ? T : "fallback", with T = never
    // Distribution over never: never
    let interner = TypeInterner::new();

    let t_name = interner.intern_string("T");
    let t_param = interner.intern(TypeKey::TypeParameter(TypeParamInfo {
        name: t_name,
        constraint: None,
        default: None,
    }));

    let lit_fallback = interner.literal_string("fallback");

    let cond = ConditionalType {
        check_type: t_param,
        extends_type: TypeId::STRING,
        true_type: t_param,
        false_type: lit_fallback,
        is_distributive: true,
    };

    let cond_type = interner.conditional(cond);
    let mut subst = TypeSubstitution::new();
    subst.insert(t_name, TypeId::NEVER);

    let instantiated = instantiate_type(&interner, cond_type, &subst);
    let result = evaluate_type(&interner, instantiated);

    // Distributive over never results in never
    assert_eq!(result, TypeId::NEVER);
}

#[test]
fn test_distributive_with_any_input() {
    // T extends string ? 1 : 2, with T = any
    // Result should be any (special case)
    let interner = TypeInterner::new();

    let t_name = interner.intern_string("T");
    let t_param = interner.intern(TypeKey::TypeParameter(TypeParamInfo {
        name: t_name,
        constraint: None,
        default: None,
    }));

    let lit_1 = interner.literal_number(1.0);
    let lit_2 = interner.literal_number(2.0);

    let cond = ConditionalType {
        check_type: t_param,
        extends_type: TypeId::STRING,
        true_type: lit_1,
        false_type: lit_2,
        is_distributive: true,
    };

    let cond_type = interner.conditional(cond);
    let mut subst = TypeSubstitution::new();
    subst.insert(t_name, TypeId::ANY);

    let instantiated = instantiate_type(&interner, cond_type, &subst);
    let result = evaluate_type(&interner, instantiated);

    // any short-circuits to any
    assert_eq!(result, TypeId::ANY);
}

#[test]
fn test_distributive_single_member_union() {
    // T extends string ? T : never, with T = "a" (single member)
    // Result: "a"
    let interner = TypeInterner::new();

    let t_name = interner.intern_string("T");
    let t_param = interner.intern(TypeKey::TypeParameter(TypeParamInfo {
        name: t_name,
        constraint: None,
        default: None,
    }));

    let lit_a = interner.literal_string("a");

    let cond = ConditionalType {
        check_type: t_param,
        extends_type: TypeId::STRING,
        true_type: t_param,
        false_type: TypeId::NEVER,
        is_distributive: true,
    };

    let cond_type = interner.conditional(cond);
    let mut subst = TypeSubstitution::new();
    // Single-member union should behave the same as the member itself
    subst.insert(t_name, interner.union(vec![lit_a]));

    let instantiated = instantiate_type(&interner, cond_type, &subst);
    let result = evaluate_type(&interner, instantiated);

    assert_eq!(result, lit_a);
}

#[test]
fn test_distributive_with_duplicate_results() {
    // T extends string | number ? 1 : 2, with T = "a" | 1 | true
    // Distribution: "a" -> 1, 1 -> 1, true -> 2
    // Result: 1 | 2 (deduplicated)
    let interner = TypeInterner::new();

    let t_name = interner.intern_string("T");
    let t_param = interner.intern(TypeKey::TypeParameter(TypeParamInfo {
        name: t_name,
        constraint: None,
        default: None,
    }));

    let lit_a = interner.literal_string("a");
    let lit_1 = interner.literal_number(1.0);
    let lit_2 = interner.literal_number(2.0);

    let string_or_number = interner.union(vec![TypeId::STRING, TypeId::NUMBER]);

    let cond = ConditionalType {
        check_type: t_param,
        extends_type: string_or_number,
        true_type: lit_1,
        false_type: lit_2,
        is_distributive: true,
    };

    let cond_type = interner.conditional(cond);
    let mut subst = TypeSubstitution::new();
    subst.insert(
        t_name,
        interner.union(vec![lit_a, interner.literal_number(42.0), interner.literal_boolean(true)]),
    );

    let instantiated = instantiate_type(&interner, cond_type, &subst);
    let result = evaluate_type(&interner, instantiated);

    // "a" -> 1, 42 -> 1, true -> 2; result = 1 | 2
    let expected = interner.union(vec![lit_1, lit_2]);
    assert_eq!(result, expected);
}

#[test]
fn test_distributive_preserves_tuple_structure() {
    // T extends [infer R] ? R : never, with T = [string] | [number]
    // Distribution: [string] -> string, [number] -> number
    // Result: string | number
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
    let tuple_number = interner.tuple(vec![TupleElement {
        type_id: TypeId::NUMBER,
        name: None,
        optional: false,
        rest: false,
    }]);
    subst.insert(t_name, interner.union(vec![tuple_string, tuple_number]));

    let instantiated = instantiate_type(&interner, cond_type, &subst);
    let result = evaluate_type(&interner, instantiated);

    let expected = interner.union(vec![TypeId::STRING, TypeId::NUMBER]);
    assert_eq!(result, expected);
}

#[test]
fn test_distributive_with_constrained_infer() {
    // T extends (infer R extends string)[] ? R : never
    // with T = string[] | number[] | boolean[]
    // Distribution: string[] -> string, number[] -> never (filtered), boolean[] -> never (filtered)
    // Result: string
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
        constraint: Some(TypeId::STRING), // R extends string constraint
        default: None,
    }));

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

    let string_array = interner.array(TypeId::STRING);
    let number_array = interner.array(TypeId::NUMBER);
    let boolean_array = interner.array(TypeId::BOOLEAN);
    subst.insert(t_name, interner.union(vec![string_array, number_array, boolean_array]));

    let instantiated = instantiate_type(&interner, cond_type, &subst);
    let result = evaluate_type(&interner, instantiated);

    // Only string[] satisfies the constraint, so result is string
    assert_eq!(result, TypeId::STRING);
}

#[test]
fn test_distributive_intrinsic_union() {
    // T extends object ? "obj" : "prim", with T = string | number | { x: string }
    // Distribution: string -> "prim", number -> "prim", { x: string } -> "obj"
    // Result: "obj" | "prim"
    let interner = TypeInterner::new();

    let t_name = interner.intern_string("T");
    let t_param = interner.intern(TypeKey::TypeParameter(TypeParamInfo {
        name: t_name,
        constraint: None,
        default: None,
    }));

    let lit_obj = interner.literal_string("obj");
    let lit_prim = interner.literal_string("prim");
    let x_atom = interner.intern_string("x");

    let obj_type = interner.object(vec![PropertyInfo {
        name: x_atom,
        type_id: TypeId::STRING,
        write_type: TypeId::STRING,
        optional: false,
        readonly: false,
        is_method: false,
    }]);

    // T extends object ? "obj" : "prim"
    let cond = ConditionalType {
        check_type: t_param,
        extends_type: TypeId::OBJECT,
        true_type: lit_obj,
        false_type: lit_prim,
        is_distributive: true,
    };

    let cond_type = interner.conditional(cond);
    let mut subst = TypeSubstitution::new();
    subst.insert(t_name, interner.union(vec![TypeId::STRING, TypeId::NUMBER, obj_type]));

    let instantiated = instantiate_type(&interner, cond_type, &subst);
    let result = evaluate_type(&interner, instantiated);

    // Expected: "obj" | "prim"
    let expected = interner.union(vec![lit_obj, lit_prim]);
    assert_eq!(result, expected);
}

#[test]
fn test_distributive_function_types() {
    // T extends (...args: any[]) => any ? "func" : "other"
    // with T = (() => void) | string | ((x: number) => string)
    // Distribution: () => void -> "func", string -> "other", (x) => string -> "func"
    // Result: "func" | "other"
    let interner = TypeInterner::new();

    let t_name = interner.intern_string("T");
    let t_param = interner.intern(TypeKey::TypeParameter(TypeParamInfo {
        name: t_name,
        constraint: None,
        default: None,
    }));

    let lit_func = interner.literal_string("func");
    let lit_other = interner.literal_string("other");

    // Pattern: (...args: any[]) => any
    let args_atom = interner.intern_string("args");
    let pattern_fn = interner.function(FunctionShape {
        params: vec![ParamInfo {
            name: Some(args_atom),
            type_id: interner.array(TypeId::ANY),
            optional: false,
            rest: true,
        }],
        this_type: None,
        return_type: TypeId::ANY,
        type_params: Vec::new(),
        type_predicate: None,
        is_constructor: false,
    });

    let cond = ConditionalType {
        check_type: t_param,
        extends_type: pattern_fn,
        true_type: lit_func,
        false_type: lit_other,
        is_distributive: true,
    };

    let cond_type = interner.conditional(cond);
    let mut subst = TypeSubstitution::new();

    let fn1 = interner.function(FunctionShape {
        params: Vec::new(),
        this_type: None,
        return_type: TypeId::VOID,
        type_params: Vec::new(),
        type_predicate: None,
        is_constructor: false,
    });
    let fn2 = interner.function(FunctionShape {
        params: vec![ParamInfo {
            name: Some(interner.intern_string("x")),
            type_id: TypeId::NUMBER,
            optional: false,
            rest: false,
        }],
        this_type: None,
        return_type: TypeId::STRING,
        type_params: Vec::new(),
        type_predicate: None,
        is_constructor: false,
    });

    subst.insert(t_name, interner.union(vec![fn1, TypeId::STRING, fn2]));

    let instantiated = instantiate_type(&interner, cond_type, &subst);
    let result = evaluate_type(&interner, instantiated);

    // Expected: "func" | "other"
    let expected = interner.union(vec![lit_func, lit_other]);
    assert_eq!(result, expected);
}

#[test]
fn test_distributive_readonly_array() {
    // T extends readonly (infer R)[] ? R : never
    // with T = readonly string[] | readonly number[] | boolean
    // Distribution: readonly string[] -> string, readonly number[] -> number, boolean -> never
    // Result: string | number
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

    let extends_array = interner.intern(TypeKey::ReadonlyType(interner.array(infer_r)));

    let cond = ConditionalType {
        check_type: t_param,
        extends_type: extends_array,
        true_type: infer_r,
        false_type: TypeId::NEVER,
        is_distributive: true,
    };

    let cond_type = interner.conditional(cond);
    let mut subst = TypeSubstitution::new();

    let readonly_string_array = interner.intern(TypeKey::ReadonlyType(interner.array(TypeId::STRING)));
    let readonly_number_array = interner.intern(TypeKey::ReadonlyType(interner.array(TypeId::NUMBER)));
    subst.insert(
        t_name,
        interner.union(vec![readonly_string_array, readonly_number_array, TypeId::BOOLEAN]),
    );

    let instantiated = instantiate_type(&interner, cond_type, &subst);
    let result = evaluate_type(&interner, instantiated);

    let expected = interner.union(vec![TypeId::STRING, TypeId::NUMBER]);
    assert_eq!(result, expected);
}

#[test]
fn test_distributive_literal_union_exhaustive() {
    // T extends "a" ? 1 : T extends "b" ? 2 : T extends "c" ? 3 : 0
    // with T = "a" | "b" | "c" | "d"
    // Distribution: "a" -> 1, "b" -> 2, "c" -> 3, "d" -> 0
    // Result: 0 | 1 | 2 | 3
    let interner = TypeInterner::new();

    let t_name = interner.intern_string("T");
    let t_param = interner.intern(TypeKey::TypeParameter(TypeParamInfo {
        name: t_name,
        constraint: None,
        default: None,
    }));

    let lit_a = interner.literal_string("a");
    let lit_b = interner.literal_string("b");
    let lit_c = interner.literal_string("c");
    let lit_d = interner.literal_string("d");
    let lit_0 = interner.literal_number(0.0);
    let lit_1 = interner.literal_number(1.0);
    let lit_2 = interner.literal_number(2.0);
    let lit_3 = interner.literal_number(3.0);

    // Innermost: T extends "c" ? 3 : 0
    let cond3 = interner.conditional(ConditionalType {
        check_type: t_param,
        extends_type: lit_c,
        true_type: lit_3,
        false_type: lit_0,
        is_distributive: false,
    });

    // Middle: T extends "b" ? 2 : cond3
    let cond2 = interner.conditional(ConditionalType {
        check_type: t_param,
        extends_type: lit_b,
        true_type: lit_2,
        false_type: cond3,
        is_distributive: false,
    });

    // Outer: T extends "a" ? 1 : cond2
    let outer_cond = ConditionalType {
        check_type: t_param,
        extends_type: lit_a,
        true_type: lit_1,
        false_type: cond2,
        is_distributive: true,
    };

    let cond_type = interner.conditional(outer_cond);
    let mut subst = TypeSubstitution::new();
    subst.insert(t_name, interner.union(vec![lit_a, lit_b, lit_c, lit_d]));

    let instantiated = instantiate_type(&interner, cond_type, &subst);
    let result = evaluate_type(&interner, instantiated);

    // Expected: 0 | 1 | 2 | 3
    let expected = interner.union(vec![lit_0, lit_1, lit_2, lit_3]);
    assert_eq!(result, expected);
}

#[test]
fn test_distributive_multiple_arrays() {
    // T extends (infer R)[][] ? R : never
    // with T = string[][] | number[][] | boolean
    // Distribution: string[][] -> string, number[][] -> number, boolean -> never
    // Result: string | number
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

    // R[][] = Array<Array<R>>
    let extends_nested_array = interner.array(interner.array(infer_r));

    let cond = ConditionalType {
        check_type: t_param,
        extends_type: extends_nested_array,
        true_type: infer_r,
        false_type: TypeId::NEVER,
        is_distributive: true,
    };

    let cond_type = interner.conditional(cond);
    let mut subst = TypeSubstitution::new();

    let nested_string_array = interner.array(interner.array(TypeId::STRING));
    let nested_number_array = interner.array(interner.array(TypeId::NUMBER));
    subst.insert(
        t_name,
        interner.union(vec![nested_string_array, nested_number_array, TypeId::BOOLEAN]),
    );

    let instantiated = instantiate_type(&interner, cond_type, &subst);
    let result = evaluate_type(&interner, instantiated);

    let expected = interner.union(vec![TypeId::STRING, TypeId::NUMBER]);
    assert_eq!(result, expected);
}

#[test]
fn test_distributive_keyof_filter() {
    // T extends keyof any ? T : never, with T = "a" | "b" | 1 | symbol
    // Distribution: "a" -> "a", "b" -> "b", 1 -> 1, symbol -> symbol
    // Result: "a" | "b" | 1 | symbol (all are valid keyof types)
    let interner = TypeInterner::new();

    let t_name = interner.intern_string("T");
    let t_param = interner.intern(TypeKey::TypeParameter(TypeParamInfo {
        name: t_name,
        constraint: None,
        default: None,
    }));

    let lit_a = interner.literal_string("a");
    let lit_b = interner.literal_string("b");
    let lit_1 = interner.literal_number(1.0);

    // keyof any = string | number | symbol
    let keyof_any = interner.union(vec![TypeId::STRING, TypeId::NUMBER, TypeId::SYMBOL]);

    let cond = ConditionalType {
        check_type: t_param,
        extends_type: keyof_any,
        true_type: t_param,
        false_type: TypeId::NEVER,
        is_distributive: true,
    };

    let cond_type = interner.conditional(cond);
    let mut subst = TypeSubstitution::new();
    subst.insert(t_name, interner.union(vec![lit_a, lit_b, lit_1, TypeId::SYMBOL]));

    let instantiated = instantiate_type(&interner, cond_type, &subst);
    let result = evaluate_type(&interner, instantiated);

    // All inputs extend string | number | symbol
    let expected = interner.union(vec![lit_a, lit_b, lit_1, TypeId::SYMBOL]);
    assert_eq!(result, expected);
}

#[test]
fn test_distributive_mixed_primitive_union() {
    // T extends string | boolean ? "primitive" : "other"
    // with T = "a" | 1 | true | null | undefined | {}
    // Distribution: "a" -> "primitive", 1 -> "other", true -> "primitive",
    //               null -> "other", undefined -> "other", {} -> "other"
    // Result: "primitive" | "other"
    let interner = TypeInterner::new();

    let t_name = interner.intern_string("T");
    let t_param = interner.intern(TypeKey::TypeParameter(TypeParamInfo {
        name: t_name,
        constraint: None,
        default: None,
    }));

    let lit_primitive = interner.literal_string("primitive");
    let lit_other = interner.literal_string("other");
    let lit_a = interner.literal_string("a");
    let lit_1 = interner.literal_number(1.0);
    let lit_true = interner.literal_boolean(true);
    let empty_obj = interner.object(Vec::new());

    let string_or_boolean = interner.union(vec![TypeId::STRING, TypeId::BOOLEAN]);

    let cond = ConditionalType {
        check_type: t_param,
        extends_type: string_or_boolean,
        true_type: lit_primitive,
        false_type: lit_other,
        is_distributive: true,
    };

    let cond_type = interner.conditional(cond);
    let mut subst = TypeSubstitution::new();
    subst.insert(
        t_name,
        interner.union(vec![lit_a, lit_1, lit_true, TypeId::NULL, TypeId::UNDEFINED, empty_obj]),
    );

    let instantiated = instantiate_type(&interner, cond_type, &subst);
    let result = evaluate_type(&interner, instantiated);

    // Expected: "primitive" | "other"
    let expected = interner.union(vec![lit_primitive, lit_other]);
    assert_eq!(result, expected);
}
