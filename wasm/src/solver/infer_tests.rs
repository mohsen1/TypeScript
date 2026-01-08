use super::*;
use crate::solver::{AssignabilityChecker, CompatChecker};

#[test]
fn test_inference_basic() {
    let interner = TypeInterner::new();
    let mut ctx = InferenceContext::new(&interner);

    // Create inference variable
    let var = ctx.fresh_var();

    // Should start unresolved
    assert!(ctx.probe(var).is_none());

    // Unify with number
    ctx.unify_var_type(var, TypeId::NUMBER).unwrap();

    // Should now be number
    assert_eq!(ctx.probe(var), Some(TypeId::NUMBER));
}

#[test]
fn test_inference_type_param() {
    let interner = TypeInterner::new();
    let mut ctx = InferenceContext::new(&interner);
    let t_name = interner.intern_string("T");
    let u_name = interner.intern_string("U");

    // Create type parameter T
    let var_t = ctx.fresh_type_param(t_name);

    // Look it up
    let found = ctx.find_type_param(t_name);
    assert_eq!(found, Some(var_t));

    // Not found
    let not_found = ctx.find_type_param(u_name);
    assert!(not_found.is_none());
}

#[test]
fn test_inference_conflict() {
    let interner = TypeInterner::new();
    let mut ctx = InferenceContext::new(&interner);

    let var = ctx.fresh_var();

    // Unify with string
    ctx.unify_var_type(var, TypeId::STRING).unwrap();

    // Try to unify with number - should fail
    let result = ctx.unify_var_type(var, TypeId::NUMBER);
    assert!(result.is_err());
}

#[test]
fn test_inference_unify_vars() {
    let interner = TypeInterner::new();
    let mut ctx = InferenceContext::new(&interner);
    let t_name = interner.intern_string("T");
    let u_name = interner.intern_string("U");

    let var_t = ctx.fresh_type_param(t_name);
    let var_u = ctx.fresh_type_param(u_name);

    ctx.unify_vars(var_t, var_u).unwrap();
    ctx.unify_var_type(var_u, TypeId::STRING).unwrap();

    assert_eq!(ctx.probe(var_t), Some(TypeId::STRING));
    assert_eq!(ctx.probe(var_u), Some(TypeId::STRING));
}

#[test]
fn test_inference_unify_vars_conflict() {
    let interner = TypeInterner::new();
    let mut ctx = InferenceContext::new(&interner);

    let var_a = ctx.fresh_var();
    let var_b = ctx.fresh_var();

    ctx.unify_var_type(var_a, TypeId::STRING).unwrap();
    ctx.unify_var_type(var_b, TypeId::NUMBER).unwrap();

    let result = ctx.unify_vars(var_a, var_b);
    assert!(matches!(
        result,
        Err(InferenceError::Conflict(a, b))
            if (a == TypeId::STRING && b == TypeId::NUMBER)
            || (a == TypeId::NUMBER && b == TypeId::STRING)
    ));
}

#[test]
fn test_inference_occurs_check() {
    let interner = TypeInterner::new();
    let mut ctx = InferenceContext::new(&interner);
    let t_name = interner.intern_string("T");

    let var_t = ctx.fresh_type_param(t_name);
    let t_type = interner.intern(TypeKey::TypeParameter(TypeParamInfo {
        name: t_name,
        constraint: None,
        default: None,
    }));
    let array_t = interner.array(t_type);

    let result = ctx.unify_var_type(var_t, array_t);
    assert!(matches!(result, Err(InferenceError::OccursCheck { .. })));
}

#[test]
fn test_inference_occurs_check_function_this_type() {
    let interner = TypeInterner::new();
    let mut ctx = InferenceContext::new(&interner);
    let t_name = interner.intern_string("T");

    let var_t = ctx.fresh_type_param(t_name);
    let t_type = interner.intern(TypeKey::TypeParameter(TypeParamInfo {
        name: t_name,
        constraint: None,
        default: None,
    }));
    let func = interner.function(FunctionShape {
        type_params: Vec::new(),
        params: Vec::new(),
        this_type: Some(t_type),
        return_type: TypeId::VOID,
        type_predicate: None,
        is_constructor: false,
    });

    let result = ctx.unify_var_type(var_t, func);
    assert!(matches!(result, Err(InferenceError::OccursCheck { .. })));
}

// =============================================================================
// Constraint Collection Tests
// =============================================================================

#[test]
fn test_constraint_lower_bound() {
    let interner = TypeInterner::new();
    let mut ctx = InferenceContext::new(&interner);

    let var = ctx.fresh_var();

    // Add lower bound: string <: var
    ctx.add_lower_bound(var, TypeId::STRING);

    let constraints = ctx.get_constraints(var).unwrap();
    assert_eq!(constraints.lower_bounds.len(), 1);
    assert!(constraints.lower_bounds.contains(&TypeId::STRING));
}

#[test]
fn test_constraint_upper_bound() {
    let interner = TypeInterner::new();
    let mut ctx = InferenceContext::new(&interner);

    let var = ctx.fresh_var();

    // Add upper bound: var <: string
    ctx.add_upper_bound(var, TypeId::STRING);

    let constraints = ctx.get_constraints(var).unwrap();
    assert_eq!(constraints.upper_bounds.len(), 1);
    assert!(constraints.upper_bounds.contains(&TypeId::STRING));
}

#[test]
fn test_constraint_multiple_lower_bounds() {
    let interner = TypeInterner::new();
    let mut ctx = InferenceContext::new(&interner);

    let var = ctx.fresh_var();

    // foo<T>(a: T, b: T) called with foo("hello", 42)
    // Lower bounds: "hello" <: T, 42 <: T
    let hello = interner.literal_string("hello");
    let forty_two = interner.literal_number(42.0);

    ctx.add_lower_bound(var, hello);
    ctx.add_lower_bound(var, forty_two);

    let constraints = ctx.get_constraints(var).unwrap();
    assert_eq!(constraints.lower_bounds.len(), 2);
}

#[test]
fn test_constraint_merge_on_unify() {
    let interner = TypeInterner::new();
    let mut ctx = InferenceContext::new(&interner);

    let var_a = ctx.fresh_var();
    let var_b = ctx.fresh_var();

    ctx.add_lower_bound(var_a, TypeId::STRING);
    ctx.add_upper_bound(var_b, TypeId::NUMBER);

    ctx.unify_vars(var_a, var_b).unwrap();

    let constraints = ctx.get_constraints(var_a).unwrap();
    assert!(constraints.lower_bounds.contains(&TypeId::STRING));
    assert!(constraints.upper_bounds.contains(&TypeId::NUMBER));
}

// =============================================================================
// Bounds Resolution Tests
// =============================================================================

#[test]
fn test_resolve_unified_vars_merged_constraints() {
    let interner = TypeInterner::new();
    let mut ctx = InferenceContext::new(&interner);

    let var_a = ctx.fresh_var();
    let var_b = ctx.fresh_var();
    let hello = interner.literal_string("hello");

    ctx.add_lower_bound(var_a, hello);
    ctx.add_upper_bound(var_b, TypeId::STRING);
    ctx.unify_vars(var_a, var_b).unwrap();

    let result = ctx.resolve_with_constraints(var_a).unwrap();
    assert_eq!(result, hello);
}

#[test]
fn test_resolve_single_lower_bound() {
    let interner = TypeInterner::new();
    let mut ctx = InferenceContext::new(&interner);
    let t_name = interner.intern_string("T");

    let var = ctx.fresh_type_param(t_name);

    // Add lower bound: string <: T
    ctx.add_lower_bound(var, TypeId::STRING);

    // Resolve should return string
    let result = ctx.resolve_with_constraints(var).unwrap();
    assert_eq!(result, TypeId::STRING);
}

#[test]
fn test_resolve_multiple_lower_bounds_union() {
    let interner = TypeInterner::new();
    let mut ctx = InferenceContext::new(&interner);
    let t_name = interner.intern_string("T");

    let var = ctx.fresh_type_param(t_name);

    // foo<T>(a: T, b: T) called with foo("hello", 42)
    let hello = interner.literal_string("hello");
    let forty_two = interner.literal_number(42.0);

    ctx.add_lower_bound(var, hello);
    ctx.add_lower_bound(var, forty_two);

    // Resolve should return "hello" | 42
    let result = ctx.resolve_with_constraints(var).unwrap();
    let expected = interner.union(vec![hello, forty_two]);
    assert_eq!(result, expected);
}

#[test]
fn test_resolve_lower_bounds_ignores_never() {
    let interner = TypeInterner::new();
    let mut ctx = InferenceContext::new(&interner);
    let t_name = interner.intern_string("T");

    let var = ctx.fresh_type_param(t_name);
    let hello = interner.literal_string("hello");

    ctx.add_lower_bound(var, TypeId::NEVER);
    ctx.add_lower_bound(var, hello);

    let result = ctx.resolve_with_constraints(var).unwrap();
    assert_eq!(result, hello);
}

#[test]
fn test_resolve_upper_bound_only() {
    let interner = TypeInterner::new();
    let mut ctx = InferenceContext::new(&interner);

    let var = ctx.fresh_type_param(interner.intern_string("T"));

    // function f<T extends string>() - upper bound only
    ctx.add_upper_bound(var, TypeId::STRING);

    // No lower bounds - should default to upper bound
    let result = ctx.resolve_with_constraints(var).unwrap();
    assert_eq!(result, TypeId::STRING);
}

#[test]
fn test_resolve_any_lower_prefers_upper_bound() {
    let interner = TypeInterner::new();
    let mut ctx = InferenceContext::new(&interner);

    let var = ctx.fresh_type_param(interner.intern_string("T"));

    ctx.add_lower_bound(var, TypeId::ANY);
    ctx.add_upper_bound(var, TypeId::STRING);

    let result = ctx.resolve_with_constraints(var).unwrap();
    assert_eq!(result, TypeId::STRING);
}

#[test]
fn test_resolve_unknown_lower_prefers_upper_bound() {
    let interner = TypeInterner::new();
    let mut ctx = InferenceContext::new(&interner);

    let var = ctx.fresh_type_param(interner.intern_string("T"));

    ctx.add_lower_bound(var, TypeId::UNKNOWN);
    ctx.add_upper_bound(var, TypeId::STRING);

    let result = ctx.resolve_with_constraints(var).unwrap();
    assert_eq!(result, TypeId::STRING);
}

#[test]
fn test_resolve_error_lower_prefers_upper_bound() {
    let interner = TypeInterner::new();
    let mut ctx = InferenceContext::new(&interner);

    let var = ctx.fresh_type_param(interner.intern_string("T"));

    ctx.add_lower_bound(var, TypeId::ERROR);
    ctx.add_upper_bound(var, TypeId::STRING);

    let result = ctx.resolve_with_constraints(var).unwrap();
    assert_eq!(result, TypeId::STRING);
}

#[test]
fn test_resolve_error_lower_with_literal_prefers_literal() {
    let interner = TypeInterner::new();
    let mut ctx = InferenceContext::new(&interner);

    let var = ctx.fresh_type_param(interner.intern_string("T"));
    let hello = interner.literal_string("hello");

    ctx.add_lower_bound(var, TypeId::ERROR);
    ctx.add_lower_bound(var, hello);
    ctx.add_upper_bound(var, TypeId::STRING);

    let result = ctx.resolve_with_constraints(var).unwrap();
    assert_eq!(result, hello);
}

#[test]
fn test_resolve_contextual_ignores_any_lower_with_literal() {
    let interner = TypeInterner::new();
    let mut ctx = InferenceContext::new(&interner);

    let var = ctx.fresh_type_param(interner.intern_string("T"));
    let hello = interner.literal_string("hello");

    ctx.add_lower_bound(var, TypeId::ANY);
    ctx.add_lower_bound(var, hello);
    ctx.add_upper_bound(var, TypeId::STRING);

    let result = ctx.resolve_with_constraints(var).unwrap();
    assert_eq!(result, hello);
}

#[test]
fn test_resolve_circular_upper_bound_defaults_unknown() {
    let interner = TypeInterner::new();
    let mut ctx = InferenceContext::new(&interner);
    let t_name = interner.intern_string("T");

    let var = ctx.fresh_type_param(t_name);
    let t_type = interner.intern(TypeKey::TypeParameter(TypeParamInfo {
        name: t_name,
        constraint: None,
        default: None,
    }));
    let name_next = interner.intern_string("next");
    let upper = interner.object(vec![PropertyInfo {
        name: name_next,
        type_id: t_type,
        write_type: t_type,
        optional: false,
        readonly: false,
        is_method: false,
    }]);

    ctx.add_upper_bound(var, upper);

    let result = ctx.resolve_with_constraints(var).unwrap();
    assert_eq!(result, TypeId::UNKNOWN);
}

#[test]
fn test_resolve_self_upper_bound_with_concrete() {
    let interner = TypeInterner::new();
    let mut ctx = InferenceContext::new(&interner);
    let t_name = interner.intern_string("T");

    let var = ctx.fresh_type_param(t_name);
    let t_type = interner.intern(TypeKey::TypeParameter(TypeParamInfo {
        name: t_name,
        constraint: None,
        default: None,
    }));

    ctx.add_upper_bound(var, t_type);
    ctx.add_upper_bound(var, TypeId::STRING);

    let result = ctx.resolve_with_constraints(var).unwrap();
    assert_eq!(result, TypeId::STRING);
}

#[test]
fn test_resolve_mutual_circular_upper_bounds_unknown() {
    let interner = TypeInterner::new();
    let mut ctx = InferenceContext::new(&interner);
    let t_name = interner.intern_string("T");
    let u_name = interner.intern_string("U");

    let var_t = ctx.fresh_type_param(t_name);
    let var_u = ctx.fresh_type_param(u_name);

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

    ctx.add_upper_bound(var_t, u_type);
    ctx.add_upper_bound(var_u, t_type);

    let result_t = ctx.resolve_with_constraints(var_t).unwrap();
    let result_u = ctx.resolve_with_constraints(var_u).unwrap();

    assert_eq!(result_t, TypeId::UNKNOWN);
    assert_eq!(result_u, TypeId::UNKNOWN);
}

#[test]
fn test_resolve_mutual_circular_upper_bounds_with_concrete() {
    let interner = TypeInterner::new();
    let mut ctx = InferenceContext::new(&interner);
    let t_name = interner.intern_string("T");
    let u_name = interner.intern_string("U");

    let var_t = ctx.fresh_type_param(t_name);
    let var_u = ctx.fresh_type_param(u_name);

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

    ctx.add_upper_bound(var_t, u_type);
    ctx.add_upper_bound(var_u, t_type);
    ctx.add_upper_bound(var_t, TypeId::STRING);

    let result_t = ctx.resolve_with_constraints(var_t).unwrap();
    let result_u = ctx.resolve_with_constraints(var_u).unwrap();

    assert_eq!(result_t, TypeId::STRING);
    assert_eq!(result_u, TypeId::UNKNOWN);
}

#[test]
fn test_resolve_self_recursive_object_bounds_two_params_unknown() {
    let interner = TypeInterner::new();
    let mut ctx = InferenceContext::new(&interner);
    let t_name = interner.intern_string("T");
    let u_name = interner.intern_string("U");

    let var_t = ctx.fresh_type_param(t_name);
    let var_u = ctx.fresh_type_param(u_name);

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
    let name_next = interner.intern_string("next");

    let upper_t = interner.object(vec![PropertyInfo {
        name: name_next,
        type_id: t_type,
        write_type: t_type,
        optional: false,
        readonly: false,
        is_method: false,
    }]);
    let upper_u = interner.object(vec![PropertyInfo {
        name: name_next,
        type_id: u_type,
        write_type: u_type,
        optional: false,
        readonly: false,
        is_method: false,
    }]);

    ctx.add_upper_bound(var_t, upper_t);
    ctx.add_upper_bound(var_u, upper_u);

    let result_t = ctx.resolve_with_constraints(var_t).unwrap();
    let result_u = ctx.resolve_with_constraints(var_u).unwrap();

    assert_eq!(result_t, TypeId::UNKNOWN);
    assert_eq!(result_u, TypeId::UNKNOWN);
}

#[test]
fn test_resolve_mutual_recursive_object_bounds_unknown() {
    let interner = TypeInterner::new();
    let mut ctx = InferenceContext::new(&interner);
    let t_name = interner.intern_string("T");
    let u_name = interner.intern_string("U");

    let var_t = ctx.fresh_type_param(t_name);
    let var_u = ctx.fresh_type_param(u_name);

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
    let name_next = interner.intern_string("next");

    let upper_t = interner.object(vec![PropertyInfo {
        name: name_next,
        type_id: u_type,
        write_type: u_type,
        optional: false,
        readonly: false,
        is_method: false,
    }]);
    let upper_u = interner.object(vec![PropertyInfo {
        name: name_next,
        type_id: t_type,
        write_type: t_type,
        optional: false,
        readonly: false,
        is_method: false,
    }]);

    ctx.add_upper_bound(var_t, upper_t);
    ctx.add_upper_bound(var_u, upper_u);

    let result_t = ctx.resolve_with_constraints(var_t).unwrap();
    let result_u = ctx.resolve_with_constraints(var_u).unwrap();

    assert_eq!(result_t, TypeId::UNKNOWN);
    assert_eq!(result_u, TypeId::UNKNOWN);
}

#[test]
fn test_resolve_multiple_upper_bounds_intersection() {
    let interner = TypeInterner::new();
    let mut ctx = InferenceContext::new(&interner);
    let t_name = interner.intern_string("T");

    let var = ctx.fresh_type_param(t_name);

    ctx.add_upper_bound(var, TypeId::STRING);
    ctx.add_upper_bound(var, TypeId::NUMBER);

    let result = ctx.resolve_with_constraints(var).unwrap();
    let expected = interner.intersection(vec![TypeId::STRING, TypeId::NUMBER]);
    assert_eq!(result, expected);
}

#[test]
fn test_resolve_bounds_valid() {
    let interner = TypeInterner::new();
    let mut ctx = InferenceContext::new(&interner);
    let t_name = interner.intern_string("T");

    let var = ctx.fresh_type_param(t_name);

    // function f<T extends string>(x: T) called with f("hello")
    // Lower: "hello" <: T, Upper: T <: string
    let hello = interner.literal_string("hello");
    ctx.add_lower_bound(var, hello);
    ctx.add_upper_bound(var, TypeId::STRING);

    // Resolve should work: "hello" is subtype of string
    let result = ctx.resolve_with_constraints(var).unwrap();
    assert_eq!(result, hello);
}

#[test]
fn test_resolve_bounds_tuple_lower_array_upper() {
    let interner = TypeInterner::new();
    let mut ctx = InferenceContext::new(&interner);
    let t_name = interner.intern_string("T");

    let var = ctx.fresh_type_param(t_name);
    let string_array = interner.array(TypeId::STRING);
    let tuple = interner.tuple(vec![
        TupleElement { type_id: TypeId::STRING, name: None, optional: false, rest: false },
        TupleElement { type_id: string_array, name: None, optional: false, rest: true },
    ]);

    ctx.add_lower_bound(var, tuple);
    ctx.add_upper_bound(var, string_array);

    let result = ctx.resolve_with_constraints(var).unwrap();
    assert_eq!(result, tuple);
}

#[test]
fn test_resolve_bounds_union_upper_allows_literal_lower() {
    let interner = TypeInterner::new();
    let mut ctx = InferenceContext::new(&interner);

    let var = ctx.fresh_type_param(interner.intern_string("T"));
    let hello = interner.literal_string("hello");
    let upper = interner.union(vec![TypeId::STRING, TypeId::NUMBER]);

    ctx.add_lower_bound(var, hello);
    ctx.add_upper_bound(var, upper);

    let result = ctx.resolve_with_constraints(var).unwrap();
    assert_eq!(result, hello);
}

#[test]
fn test_resolve_bounds_object_subtype() {
    let interner = TypeInterner::new();
    let mut ctx = InferenceContext::new(&interner);

    let var = ctx.fresh_type_param(interner.intern_string("T"));
    let name_a = interner.intern_string("a");
    let name_b = interner.intern_string("b");

    let upper = interner.object(vec![PropertyInfo {
        name: name_a,
        type_id: TypeId::STRING,
        write_type: TypeId::STRING,
        optional: false,
        readonly: false,
        is_method: false,
    }]);
    let lower = interner.object(vec![
        PropertyInfo {
            name: name_a,
            type_id: TypeId::STRING,
            write_type: TypeId::STRING,
            optional: false,
            readonly: false,
            is_method: false,
        },
        PropertyInfo {
            name: name_b,
            type_id: TypeId::NUMBER,
            write_type: TypeId::NUMBER,
            optional: false,
            readonly: false,
            is_method: false,
        },
    ]);

    ctx.add_lower_bound(var, lower);
    ctx.add_upper_bound(var, upper);

    let result = ctx.resolve_with_constraints(var).unwrap();
    assert_eq!(result, lower);
}

#[test]
fn test_resolve_bounds_union_lower_vs_string_upper() {
    let interner = TypeInterner::new();
    let mut ctx = InferenceContext::new(&interner);

    let var = ctx.fresh_type_param(interner.intern_string("T"));
    let lower = interner.union(vec![TypeId::STRING, TypeId::NUMBER]);

    ctx.add_lower_bound(var, lower);
    ctx.add_upper_bound(var, TypeId::STRING);

    let result = ctx.resolve_with_constraints(var);
    assert!(matches!(
        result,
        Err(InferenceError::BoundsViolation {
            lower: actual_lower,
            upper: actual_upper,
            ..
        }) if actual_lower == lower && actual_upper == TypeId::STRING
    ));
}

#[test]
fn test_resolve_bounds_object_readonly_property_mismatch() {
    let interner = TypeInterner::new();
    let mut ctx = InferenceContext::new(&interner);

    let var = ctx.fresh_type_param(interner.intern_string("T"));
    let name_a = interner.intern_string("a");

    let upper = interner.object(vec![PropertyInfo {
        name: name_a,
        type_id: TypeId::STRING,
        write_type: TypeId::STRING,
        optional: false,
        readonly: false,
        is_method: false,
    }]);

    let lower = interner.object(vec![PropertyInfo {
        name: name_a,
        type_id: TypeId::STRING,
        write_type: TypeId::STRING,
        optional: false,
        readonly: true,
        is_method: false,
    }]);

    ctx.add_lower_bound(var, lower);
    ctx.add_upper_bound(var, upper);

    let result = ctx.resolve_with_constraints(var);
    assert!(matches!(
        result,
        Err(InferenceError::BoundsViolation {
            lower: actual_lower,
            upper: actual_upper,
            ..
        }) if actual_lower == lower && actual_upper == upper
    ));
}

#[test]
fn test_resolve_bounds_object_readonly_property_ok() {
    let interner = TypeInterner::new();
    let mut ctx = InferenceContext::new(&interner);

    let var = ctx.fresh_type_param(interner.intern_string("T"));
    let name_a = interner.intern_string("a");

    let upper = interner.object(vec![PropertyInfo {
        name: name_a,
        type_id: TypeId::STRING,
        write_type: TypeId::STRING,
        optional: false,
        readonly: true,
        is_method: false,
    }]);

    let lower = interner.object(vec![PropertyInfo {
        name: name_a,
        type_id: TypeId::STRING,
        write_type: TypeId::STRING,
        optional: false,
        readonly: false,
        is_method: false,
    }]);

    ctx.add_lower_bound(var, lower);
    ctx.add_upper_bound(var, upper);

    let result = ctx.resolve_with_constraints(var).unwrap();
    assert_eq!(result, lower);
}

#[test]
fn test_resolve_bounds_object_readonly_property_missing_ok() {
    let interner = TypeInterner::new();
    let mut ctx = InferenceContext::new(&interner);

    let var = ctx.fresh_type_param(interner.intern_string("T"));
    let name_a = interner.intern_string("a");

    let upper = interner.object(vec![PropertyInfo {
        name: name_a,
        type_id: TypeId::STRING,
        write_type: TypeId::STRING,
        optional: true,
        readonly: true,
        is_method: false,
    }]);
    let lower = interner.object(Vec::new());

    ctx.add_lower_bound(var, lower);
    ctx.add_upper_bound(var, upper);

    let result = ctx.resolve_with_constraints(var).unwrap();
    assert_eq!(result, lower);
}

#[test]
fn test_resolve_bounds_method_property_bivariant_params() {
    let interner = TypeInterner::new();
    let mut ctx = InferenceContext::new(&interner);

    let var = ctx.fresh_type_param(interner.intern_string("T"));
    let name_m = interner.intern_string("m");

    let narrow_param = ParamInfo {
        name: Some(interner.intern_string("x")),
        type_id: TypeId::STRING,
        optional: false,
        rest: false,
    };
    let wide_param = ParamInfo {
        name: Some(interner.intern_string("x")),
        type_id: interner.union(vec![TypeId::STRING, TypeId::NUMBER]),
        optional: false,
        rest: false,
    };

    let lower_fn = interner.function(FunctionShape {
        type_params: Vec::new(),
        params: vec![narrow_param],
        this_type: None,
        return_type: TypeId::NUMBER,
        type_predicate: None,
        is_constructor: false,
    });
    let upper_fn = interner.function(FunctionShape {
        type_params: Vec::new(),
        params: vec![wide_param],
        this_type: None,
        return_type: TypeId::NUMBER,
        type_predicate: None,
        is_constructor: false,
    });

    let lower = interner.object(vec![PropertyInfo {
        name: name_m,
        type_id: lower_fn,
        write_type: lower_fn,
        optional: false,
        readonly: false,
        is_method: true,
    }]);
    let upper = interner.object(vec![PropertyInfo {
        name: name_m,
        type_id: upper_fn,
        write_type: upper_fn,
        optional: false,
        readonly: false,
        is_method: true,
    }]);

    ctx.add_lower_bound(var, lower);
    ctx.add_upper_bound(var, upper);

    let result = ctx.resolve_with_constraints(var).unwrap();
    assert_eq!(result, lower);
}

#[test]
fn test_resolve_bounds_function_property_contravariant_params() {
    let interner = TypeInterner::new();
    let mut ctx = InferenceContext::new(&interner);

    let var = ctx.fresh_type_param(interner.intern_string("T"));
    let name_f = interner.intern_string("f");

    let narrow_param = ParamInfo {
        name: Some(interner.intern_string("x")),
        type_id: TypeId::STRING,
        optional: false,
        rest: false,
    };
    let wide_param = ParamInfo {
        name: Some(interner.intern_string("x")),
        type_id: interner.union(vec![TypeId::STRING, TypeId::NUMBER]),
        optional: false,
        rest: false,
    };

    let lower_fn = interner.function(FunctionShape {
        type_params: Vec::new(),
        params: vec![narrow_param],
        this_type: None,
        return_type: TypeId::NUMBER,
        type_predicate: None,
        is_constructor: false,
    });
    let upper_fn = interner.function(FunctionShape {
        type_params: Vec::new(),
        params: vec![wide_param],
        this_type: None,
        return_type: TypeId::NUMBER,
        type_predicate: None,
        is_constructor: false,
    });

    let lower = interner.object(vec![PropertyInfo {
        name: name_f,
        type_id: lower_fn,
        write_type: lower_fn,
        optional: false,
        readonly: false,
        is_method: false,
    }]);
    let upper = interner.object(vec![PropertyInfo {
        name: name_f,
        type_id: upper_fn,
        write_type: upper_fn,
        optional: false,
        readonly: false,
        is_method: false,
    }]);

    ctx.add_lower_bound(var, lower);
    ctx.add_upper_bound(var, upper);

    let result = ctx.resolve_with_constraints(var);
    assert!(matches!(
        result,
        Err(InferenceError::BoundsViolation {
            lower: actual_lower,
            upper: actual_upper,
            ..
        }) if actual_lower == lower && actual_upper == upper
    ));
}

#[test]
fn test_resolve_bounds_with_assignability_bivariant_function_property() {
    let interner = TypeInterner::new();
    let mut ctx = InferenceContext::new(&interner);
    let mut checker = CompatChecker::new(&interner);

    let var = ctx.fresh_type_param(interner.intern_string("T"));
    let name_f = interner.intern_string("f");

    let narrow_param = ParamInfo {
        name: Some(interner.intern_string("x")),
        type_id: TypeId::STRING,
        optional: false,
        rest: false,
    };
    let wide_param = ParamInfo {
        name: Some(interner.intern_string("x")),
        type_id: interner.union(vec![TypeId::STRING, TypeId::NUMBER]),
        optional: false,
        rest: false,
    };

    let lower_fn = interner.function(FunctionShape {
        type_params: Vec::new(),
        params: vec![narrow_param],
        this_type: None,
        return_type: TypeId::NUMBER,
        type_predicate: None,
        is_constructor: false,
    });
    let upper_fn = interner.function(FunctionShape {
        type_params: Vec::new(),
        params: vec![wide_param],
        this_type: None,
        return_type: TypeId::NUMBER,
        type_predicate: None,
        is_constructor: false,
    });

    let lower = interner.object(vec![PropertyInfo {
        name: name_f,
        type_id: lower_fn,
        write_type: lower_fn,
        optional: false,
        readonly: false,
        is_method: false,
    }]);
    let upper = interner.object(vec![PropertyInfo {
        name: name_f,
        type_id: upper_fn,
        write_type: upper_fn,
        optional: false,
        readonly: false,
        is_method: false,
    }]);

    ctx.add_lower_bound(var, lower);
    ctx.add_upper_bound(var, upper);

    let result = ctx
        .resolve_with_constraints_by(var, |source, target| {
            checker.is_assignable_to(source, target)
        })
        .unwrap();
    assert_eq!(result, lower);
}

#[test]
fn test_resolve_bounds_object_keyword_upper_allows_array() {
    let interner = TypeInterner::new();
    let mut ctx = InferenceContext::new(&interner);

    let var = ctx.fresh_type_param(interner.intern_string("T"));
    let lower = interner.array(TypeId::STRING);
    let upper = TypeId::OBJECT;

    ctx.add_lower_bound(var, lower);
    ctx.add_upper_bound(var, upper);

    let result = ctx.resolve_with_constraints(var).unwrap();
    assert_eq!(result, lower);
}

#[test]
fn test_resolve_bounds_object_keyword_rejects_string() {
    let interner = TypeInterner::new();
    let mut ctx = InferenceContext::new(&interner);

    let var = ctx.fresh_type_param(interner.intern_string("T"));
    let lower = TypeId::STRING;
    let upper = TypeId::OBJECT;

    ctx.add_lower_bound(var, lower);
    ctx.add_upper_bound(var, upper);

    let result = ctx.resolve_with_constraints(var);
    assert!(matches!(
        result,
        Err(InferenceError::BoundsViolation {
            lower: actual_lower,
            upper: actual_upper,
            ..
        }) if actual_lower == lower && actual_upper == upper
    ));
}

#[test]
fn test_resolve_bounds_object_with_index_subtype() {
    let interner = TypeInterner::new();
    let mut ctx = InferenceContext::new(&interner);

    let var = ctx.fresh_type_param(interner.intern_string("T"));
    let name_a = interner.intern_string("a");

    let upper = interner.object_with_index(ObjectShape {
        properties: Vec::new(),
        string_index: Some(IndexSignature {
            key_type: TypeId::STRING,
            value_type: TypeId::STRING,
            readonly: false,
        }),
        number_index: None,
    });

    let lower = interner.object_with_index(ObjectShape {
        properties: vec![PropertyInfo {
            name: name_a,
            type_id: TypeId::STRING,
            write_type: TypeId::STRING,
            optional: false,
            readonly: false,
            is_method: false,
        }],
        string_index: Some(IndexSignature {
            key_type: TypeId::STRING,
            value_type: TypeId::STRING,
            readonly: false,
        }),
        number_index: None,
    });

    ctx.add_lower_bound(var, lower);
    ctx.add_upper_bound(var, upper);

    let result = ctx.resolve_with_constraints(var).unwrap();
    assert_eq!(result, lower);
}

#[test]
fn test_resolve_bounds_string_index_property_mismatch() {
    let interner = TypeInterner::new();
    let mut ctx = InferenceContext::new(&interner);

    let var = ctx.fresh_type_param(interner.intern_string("T"));
    let name_a = interner.intern_string("a");

    let upper = interner.object_with_index(ObjectShape {
        properties: Vec::new(),
        string_index: Some(IndexSignature {
            key_type: TypeId::STRING,
            value_type: TypeId::STRING,
            readonly: false,
        }),
        number_index: None,
    });

    let lower = interner.object(vec![PropertyInfo {
        name: name_a,
        type_id: TypeId::NUMBER,
        write_type: TypeId::NUMBER,
        optional: false,
        readonly: false,
        is_method: false,
    }]);

    ctx.add_lower_bound(var, lower);
    ctx.add_upper_bound(var, upper);

    let result = ctx.resolve_with_constraints(var);
    assert!(matches!(
        result,
        Err(InferenceError::BoundsViolation {
            lower: actual_lower,
            upper: actual_upper,
            ..
        }) if actual_lower == lower && actual_upper == upper
    ));
}

#[test]
fn test_resolve_bounds_index_readonly_property_mismatch() {
    let interner = TypeInterner::new();
    let mut ctx = InferenceContext::new(&interner);

    let var = ctx.fresh_type_param(interner.intern_string("T"));
    let name_a = interner.intern_string("a");

    let upper = interner.object_with_index(ObjectShape {
        properties: Vec::new(),
        string_index: Some(IndexSignature {
            key_type: TypeId::STRING,
            value_type: TypeId::NUMBER,
            readonly: false,
        }),
        number_index: None,
    });

    let lower = interner.object(vec![PropertyInfo {
        name: name_a,
        type_id: TypeId::NUMBER,
        write_type: TypeId::NUMBER,
        optional: false,
        readonly: true,
        is_method: false,
    }]);

    ctx.add_lower_bound(var, lower);
    ctx.add_upper_bound(var, upper);

    let result = ctx.resolve_with_constraints(var);
    assert!(matches!(
        result,
        Err(InferenceError::BoundsViolation {
            lower: actual_lower,
            upper: actual_upper,
            ..
        }) if actual_lower == lower && actual_upper == upper
    ));
}

#[test]
fn test_resolve_bounds_index_readonly_signature_mismatch() {
    let interner = TypeInterner::new();
    let mut ctx = InferenceContext::new(&interner);

    let var = ctx.fresh_type_param(interner.intern_string("T"));

    let upper = interner.object_with_index(ObjectShape {
        properties: Vec::new(),
        string_index: Some(IndexSignature {
            key_type: TypeId::STRING,
            value_type: TypeId::NUMBER,
            readonly: false,
        }),
        number_index: None,
    });

    let lower = interner.object_with_index(ObjectShape {
        properties: Vec::new(),
        string_index: Some(IndexSignature {
            key_type: TypeId::STRING,
            value_type: TypeId::NUMBER,
            readonly: true,
        }),
        number_index: None,
    });

    ctx.add_lower_bound(var, lower);
    ctx.add_upper_bound(var, upper);

    let result = ctx.resolve_with_constraints(var);
    assert!(matches!(
        result,
        Err(InferenceError::BoundsViolation {
            lower: actual_lower,
            upper: actual_upper,
            ..
        }) if actual_lower == lower && actual_upper == upper
    ));
}

#[test]
fn test_resolve_bounds_index_readonly_signature_allows_mutable_source() {
    let interner = TypeInterner::new();
    let mut ctx = InferenceContext::new(&interner);

    let var = ctx.fresh_type_param(interner.intern_string("T"));

    let upper = interner.object_with_index(ObjectShape {
        properties: Vec::new(),
        string_index: Some(IndexSignature {
            key_type: TypeId::STRING,
            value_type: TypeId::NUMBER,
            readonly: true,
        }),
        number_index: None,
    });

    let lower = interner.object_with_index(ObjectShape {
        properties: Vec::new(),
        string_index: Some(IndexSignature {
            key_type: TypeId::STRING,
            value_type: TypeId::NUMBER,
            readonly: false,
        }),
        number_index: None,
    });

    ctx.add_lower_bound(var, lower);
    ctx.add_upper_bound(var, upper);

    let result = ctx.resolve_with_constraints(var).unwrap();
    assert_eq!(result, lower);
}

#[test]
fn test_resolve_bounds_number_index_allows_non_numeric_property() {
    let interner = TypeInterner::new();
    let mut ctx = InferenceContext::new(&interner);

    let var = ctx.fresh_type_param(interner.intern_string("T"));
    let name_a = interner.intern_string("a");

    let upper = interner.object_with_index(ObjectShape {
        properties: Vec::new(),
        string_index: None,
        number_index: Some(IndexSignature {
            key_type: TypeId::NUMBER,
            value_type: TypeId::NUMBER,
            readonly: false,
        }),
    });

    let lower = interner.object_with_index(ObjectShape {
        properties: vec![PropertyInfo {
            name: name_a,
            type_id: TypeId::STRING,
            write_type: TypeId::STRING,
            optional: false,
            readonly: false,
            is_method: false,
        }],
        string_index: None,
        number_index: None,
    });

    ctx.add_lower_bound(var, lower);
    ctx.add_upper_bound(var, upper);

    let result = ctx.resolve_with_constraints(var).unwrap();
    assert_eq!(result, lower);
}

#[test]
fn test_resolve_bounds_number_index_numeric_property_mismatch() {
    let interner = TypeInterner::new();
    let mut ctx = InferenceContext::new(&interner);

    let var = ctx.fresh_type_param(interner.intern_string("T"));
    let name_zero = interner.intern_string("0");

    let upper_type = interner.object_with_index(ObjectShape {
        properties: Vec::new(),
        string_index: None,
        number_index: Some(IndexSignature {
            key_type: TypeId::NUMBER,
            value_type: TypeId::NUMBER,
            readonly: false,
        }),
    });

    let lower_type = interner.object_with_index(ObjectShape {
        properties: vec![PropertyInfo {
            name: name_zero,
            type_id: TypeId::STRING,
            write_type: TypeId::STRING,
            optional: false,
            readonly: false,
            is_method: false,
        }],
        string_index: None,
        number_index: None,
    });

    ctx.add_lower_bound(var, lower_type);
    ctx.add_upper_bound(var, upper_type);

    let result = ctx.resolve_with_constraints(var);
    assert!(matches!(
        result,
        Err(InferenceError::BoundsViolation {
            lower: actual_lower,
            upper: actual_upper,
            ..
        }) if actual_lower == lower_type && actual_upper == upper_type
    ));
}

#[test]
fn test_resolve_bounds_number_index_readonly_property_mismatch() {
    let interner = TypeInterner::new();
    let mut ctx = InferenceContext::new(&interner);

    let var = ctx.fresh_type_param(interner.intern_string("T"));
    let name_zero = interner.intern_string("0");

    let upper_type = interner.object_with_index(ObjectShape {
        properties: Vec::new(),
        string_index: None,
        number_index: Some(IndexSignature {
            key_type: TypeId::NUMBER,
            value_type: TypeId::NUMBER,
            readonly: false,
        }),
    });

    let lower_type = interner.object(vec![PropertyInfo {
        name: name_zero,
        type_id: TypeId::NUMBER,
        write_type: TypeId::NUMBER,
        optional: false,
        readonly: true,
        is_method: false,
    }]);

    ctx.add_lower_bound(var, lower_type);
    ctx.add_upper_bound(var, upper_type);

    let result = ctx.resolve_with_constraints(var);
    assert!(matches!(
        result,
        Err(InferenceError::BoundsViolation {
            lower: actual_lower,
            upper: actual_upper,
            ..
        }) if actual_lower == lower_type && actual_upper == upper_type
    ));
}

#[test]
fn test_resolve_bounds_number_index_readonly_signature_mismatch() {
    let interner = TypeInterner::new();
    let mut ctx = InferenceContext::new(&interner);

    let var = ctx.fresh_type_param(interner.intern_string("T"));

    let upper_type = interner.object_with_index(ObjectShape {
        properties: Vec::new(),
        string_index: None,
        number_index: Some(IndexSignature {
            key_type: TypeId::NUMBER,
            value_type: TypeId::NUMBER,
            readonly: false,
        }),
    });

    let lower_type = interner.object_with_index(ObjectShape {
        properties: Vec::new(),
        string_index: None,
        number_index: Some(IndexSignature {
            key_type: TypeId::NUMBER,
            value_type: TypeId::NUMBER,
            readonly: true,
        }),
    });

    ctx.add_lower_bound(var, lower_type);
    ctx.add_upper_bound(var, upper_type);

    let result = ctx.resolve_with_constraints(var);
    assert!(matches!(
        result,
        Err(InferenceError::BoundsViolation {
            lower: actual_lower,
            upper: actual_upper,
            ..
        }) if actual_lower == lower_type && actual_upper == upper_type
    ));
}

#[test]
fn test_resolve_bounds_number_index_readonly_signature_allows_mutable_source() {
    let interner = TypeInterner::new();
    let mut ctx = InferenceContext::new(&interner);

    let var = ctx.fresh_type_param(interner.intern_string("T"));

    let upper_type = interner.object_with_index(ObjectShape {
        properties: Vec::new(),
        string_index: None,
        number_index: Some(IndexSignature {
            key_type: TypeId::NUMBER,
            value_type: TypeId::NUMBER,
            readonly: true,
        }),
    });

    let lower_type = interner.object_with_index(ObjectShape {
        properties: Vec::new(),
        string_index: None,
        number_index: Some(IndexSignature {
            key_type: TypeId::NUMBER,
            value_type: TypeId::NUMBER,
            readonly: false,
        }),
    });

    ctx.add_lower_bound(var, lower_type);
    ctx.add_upper_bound(var, upper_type);

    let result = ctx.resolve_with_constraints(var).unwrap();
    assert_eq!(result, lower_type);
}

#[test]
fn test_resolve_bounds_number_index_ignores_non_canonical_numeric_name() {
    let interner = TypeInterner::new();
    let mut ctx = InferenceContext::new(&interner);

    let var = ctx.fresh_type_param(interner.intern_string("T"));
    let name = interner.intern_string("01");

    let upper_type = interner.object_with_index(ObjectShape {
        properties: Vec::new(),
        string_index: None,
        number_index: Some(IndexSignature {
            key_type: TypeId::NUMBER,
            value_type: TypeId::NUMBER,
            readonly: false,
        }),
    });

    let lower_type = interner.object_with_index(ObjectShape {
        properties: vec![PropertyInfo {
            name,
            type_id: TypeId::STRING,
            write_type: TypeId::STRING,
            optional: false,
            readonly: false,
            is_method: false,
        }],
        string_index: None,
        number_index: None,
    });

    ctx.add_lower_bound(var, lower_type);
    ctx.add_upper_bound(var, upper_type);

    let result = ctx.resolve_with_constraints(var).unwrap();
    assert_eq!(result, lower_type);
}

#[test]
fn test_resolve_bounds_number_index_accepts_exponent_name() {
    let interner = TypeInterner::new();
    let mut ctx = InferenceContext::new(&interner);

    let var = ctx.fresh_type_param(interner.intern_string("T"));
    let name = interner.intern_string("1e-7");

    let upper_type = interner.object_with_index(ObjectShape {
        properties: Vec::new(),
        string_index: None,
        number_index: Some(IndexSignature {
            key_type: TypeId::NUMBER,
            value_type: TypeId::NUMBER,
            readonly: false,
        }),
    });

    let lower_type = interner.object_with_index(ObjectShape {
        properties: vec![PropertyInfo {
            name,
            type_id: TypeId::STRING,
            write_type: TypeId::STRING,
            optional: false,
            readonly: false,
            is_method: false,
        }],
        string_index: None,
        number_index: None,
    });

    ctx.add_lower_bound(var, lower_type);
    ctx.add_upper_bound(var, upper_type);

    let result = ctx.resolve_with_constraints(var);
    assert!(matches!(
        result,
        Err(InferenceError::BoundsViolation {
            lower: actual_lower,
            upper: actual_upper,
            ..
        }) if actual_lower == lower_type && actual_upper == upper_type
    ));
}

#[test]
fn test_resolve_bounds_number_index_accepts_infinity_name() {
    let interner = TypeInterner::new();
    let mut ctx = InferenceContext::new(&interner);

    let var = ctx.fresh_type_param(interner.intern_string("T"));
    let name = interner.intern_string("Infinity");

    let upper_type = interner.object_with_index(ObjectShape {
        properties: Vec::new(),
        string_index: None,
        number_index: Some(IndexSignature {
            key_type: TypeId::NUMBER,
            value_type: TypeId::NUMBER,
            readonly: false,
        }),
    });

    let lower_type = interner.object_with_index(ObjectShape {
        properties: vec![PropertyInfo {
            name,
            type_id: TypeId::STRING,
            write_type: TypeId::STRING,
            optional: false,
            readonly: false,
            is_method: false,
        }],
        string_index: None,
        number_index: None,
    });

    ctx.add_lower_bound(var, lower_type);
    ctx.add_upper_bound(var, upper_type);

    let result = ctx.resolve_with_constraints(var);
    assert!(matches!(
        result,
        Err(InferenceError::BoundsViolation {
            lower: actual_lower,
            upper: actual_upper,
            ..
        }) if actual_lower == lower_type && actual_upper == upper_type
    ));
}

#[test]
fn test_resolve_bounds_number_index_accepts_nan_name() {
    let interner = TypeInterner::new();
    let mut ctx = InferenceContext::new(&interner);

    let var = ctx.fresh_type_param(interner.intern_string("T"));
    let name = interner.intern_string("NaN");

    let upper_type = interner.object_with_index(ObjectShape {
        properties: Vec::new(),
        string_index: None,
        number_index: Some(IndexSignature {
            key_type: TypeId::NUMBER,
            value_type: TypeId::NUMBER,
            readonly: false,
        }),
    });

    let lower_type = interner.object_with_index(ObjectShape {
        properties: vec![PropertyInfo {
            name,
            type_id: TypeId::STRING,
            write_type: TypeId::STRING,
            optional: false,
            readonly: false,
            is_method: false,
        }],
        string_index: None,
        number_index: None,
    });

    ctx.add_lower_bound(var, lower_type);
    ctx.add_upper_bound(var, upper_type);

    let result = ctx.resolve_with_constraints(var);
    assert!(matches!(
        result,
        Err(InferenceError::BoundsViolation {
            lower: actual_lower,
            upper: actual_upper,
            ..
        }) if actual_lower == lower_type && actual_upper == upper_type
    ));
}

#[test]
fn test_resolve_bounds_number_index_accepts_negative_infinity_name() {
    let interner = TypeInterner::new();
    let mut ctx = InferenceContext::new(&interner);

    let var = ctx.fresh_type_param(interner.intern_string("T"));
    let name = interner.intern_string("-Infinity");

    let upper_type = interner.object_with_index(ObjectShape {
        properties: Vec::new(),
        string_index: None,
        number_index: Some(IndexSignature {
            key_type: TypeId::NUMBER,
            value_type: TypeId::NUMBER,
            readonly: false,
        }),
    });

    let lower_type = interner.object_with_index(ObjectShape {
        properties: vec![PropertyInfo {
            name,
            type_id: TypeId::STRING,
            write_type: TypeId::STRING,
            optional: false,
            readonly: false,
            is_method: false,
        }],
        string_index: None,
        number_index: None,
    });

    ctx.add_lower_bound(var, lower_type);
    ctx.add_upper_bound(var, upper_type);

    let result = ctx.resolve_with_constraints(var);
    assert!(matches!(
        result,
        Err(InferenceError::BoundsViolation {
            lower: actual_lower,
            upper: actual_upper,
            ..
        }) if actual_lower == lower_type && actual_upper == upper_type
    ));
}

#[test]
fn test_resolve_bounds_number_index_ignores_negative_zero_name() {
    let interner = TypeInterner::new();
    let mut ctx = InferenceContext::new(&interner);

    let var = ctx.fresh_type_param(interner.intern_string("T"));
    let name = interner.intern_string("-0");

    let upper_type = interner.object_with_index(ObjectShape {
        properties: Vec::new(),
        string_index: None,
        number_index: Some(IndexSignature {
            key_type: TypeId::NUMBER,
            value_type: TypeId::NUMBER,
            readonly: false,
        }),
    });

    let lower_type = interner.object_with_index(ObjectShape {
        properties: vec![PropertyInfo {
            name,
            type_id: TypeId::STRING,
            write_type: TypeId::STRING,
            optional: false,
            readonly: false,
            is_method: false,
        }],
        string_index: None,
        number_index: None,
    });

    ctx.add_lower_bound(var, lower_type);
    ctx.add_upper_bound(var, upper_type);

    let result = ctx.resolve_with_constraints(var).unwrap();
    assert_eq!(result, lower_type);
}

#[test]
fn test_resolve_bounds_number_index_ignores_negative_zero_property() {
    let interner = TypeInterner::new();
    let mut ctx = InferenceContext::new(&interner);

    let var = ctx.fresh_type_param(interner.intern_string("T"));
    let name = interner.intern_string("-0");

    let upper_type = interner.object_with_index(ObjectShape {
        properties: Vec::new(),
        string_index: None,
        number_index: Some(IndexSignature {
            key_type: TypeId::NUMBER,
            value_type: TypeId::NUMBER,
            readonly: false,
        }),
    });

    let lower_type = interner.object(vec![PropertyInfo {
        name,
        type_id: TypeId::STRING,
        write_type: TypeId::STRING,
        optional: false,
        readonly: false,
        is_method: false,
    }]);

    ctx.add_lower_bound(var, lower_type);
    ctx.add_upper_bound(var, upper_type);

    let result = ctx.resolve_with_constraints(var).unwrap();
    assert_eq!(result, lower_type);
}

#[test]
fn test_resolve_bounds_number_index_accepts_decimal_boundary_name() {
    let interner = TypeInterner::new();
    let mut ctx = InferenceContext::new(&interner);

    let var = ctx.fresh_type_param(interner.intern_string("T"));
    let name = interner.intern_string("0.000001");

    let upper_type = interner.object_with_index(ObjectShape {
        properties: Vec::new(),
        string_index: None,
        number_index: Some(IndexSignature {
            key_type: TypeId::NUMBER,
            value_type: TypeId::NUMBER,
            readonly: false,
        }),
    });

    let lower_type = interner.object_with_index(ObjectShape {
        properties: vec![PropertyInfo {
            name,
            type_id: TypeId::STRING,
            write_type: TypeId::STRING,
            optional: false,
            readonly: false,
            is_method: false,
        }],
        string_index: None,
        number_index: None,
    });

    ctx.add_lower_bound(var, lower_type);
    ctx.add_upper_bound(var, upper_type);

    let result = ctx.resolve_with_constraints(var);
    assert!(matches!(
        result,
        Err(InferenceError::BoundsViolation {
            lower: actual_lower,
            upper: actual_upper,
            ..
        }) if actual_lower == lower_type && actual_upper == upper_type
    ));
}

#[test]
fn test_resolve_bounds_number_index_accepts_exponent_boundary_name() {
    let interner = TypeInterner::new();
    let mut ctx = InferenceContext::new(&interner);

    let var = ctx.fresh_type_param(interner.intern_string("T"));
    let name = interner.intern_string("1e+21");

    let upper_type = interner.object_with_index(ObjectShape {
        properties: Vec::new(),
        string_index: None,
        number_index: Some(IndexSignature {
            key_type: TypeId::NUMBER,
            value_type: TypeId::NUMBER,
            readonly: false,
        }),
    });

    let lower_type = interner.object_with_index(ObjectShape {
        properties: vec![PropertyInfo {
            name,
            type_id: TypeId::STRING,
            write_type: TypeId::STRING,
            optional: false,
            readonly: false,
            is_method: false,
        }],
        string_index: None,
        number_index: None,
    });

    ctx.add_lower_bound(var, lower_type);
    ctx.add_upper_bound(var, upper_type);

    let result = ctx.resolve_with_constraints(var);
    assert!(matches!(
        result,
        Err(InferenceError::BoundsViolation {
            lower: actual_lower,
            upper: actual_upper,
            ..
        }) if actual_lower == lower_type && actual_upper == upper_type
    ));
}

#[test]
fn test_resolve_bounds_number_index_ignores_non_canonical_exponent_name() {
    let interner = TypeInterner::new();
    let mut ctx = InferenceContext::new(&interner);

    let var = ctx.fresh_type_param(interner.intern_string("T"));
    let name = interner.intern_string("1e+021");

    let upper_type = interner.object_with_index(ObjectShape {
        properties: Vec::new(),
        string_index: None,
        number_index: Some(IndexSignature {
            key_type: TypeId::NUMBER,
            value_type: TypeId::NUMBER,
            readonly: false,
        }),
    });

    let lower_type = interner.object_with_index(ObjectShape {
        properties: vec![PropertyInfo {
            name,
            type_id: TypeId::STRING,
            write_type: TypeId::STRING,
            optional: false,
            readonly: false,
            is_method: false,
        }],
        string_index: None,
        number_index: None,
    });

    ctx.add_lower_bound(var, lower_type);
    ctx.add_upper_bound(var, upper_type);

    let result = ctx.resolve_with_constraints(var).unwrap();
    assert_eq!(result, lower_type);
}

#[test]
fn test_resolve_bounds_number_index_ignores_uppercase_exponent_name() {
    let interner = TypeInterner::new();
    let mut ctx = InferenceContext::new(&interner);

    let var = ctx.fresh_type_param(interner.intern_string("T"));
    let name = interner.intern_string("1E+21");

    let upper_type = interner.object_with_index(ObjectShape {
        properties: Vec::new(),
        string_index: None,
        number_index: Some(IndexSignature {
            key_type: TypeId::NUMBER,
            value_type: TypeId::NUMBER,
            readonly: false,
        }),
    });

    let lower_type = interner.object_with_index(ObjectShape {
        properties: vec![PropertyInfo {
            name,
            type_id: TypeId::STRING,
            write_type: TypeId::STRING,
            optional: false,
            readonly: false,
            is_method: false,
        }],
        string_index: None,
        number_index: None,
    });

    ctx.add_lower_bound(var, lower_type);
    ctx.add_upper_bound(var, upper_type);

    let result = ctx.resolve_with_constraints(var).unwrap();
    assert_eq!(result, lower_type);
}

#[test]
fn test_resolve_bounds_number_index_ignores_uppercase_exponent_missing_sign() {
    let interner = TypeInterner::new();
    let mut ctx = InferenceContext::new(&interner);

    let var = ctx.fresh_type_param(interner.intern_string("T"));
    let name = interner.intern_string("1E21");

    let upper_type = interner.object_with_index(ObjectShape {
        properties: Vec::new(),
        string_index: None,
        number_index: Some(IndexSignature {
            key_type: TypeId::NUMBER,
            value_type: TypeId::NUMBER,
            readonly: false,
        }),
    });

    let lower_type = interner.object_with_index(ObjectShape {
        properties: vec![PropertyInfo {
            name,
            type_id: TypeId::STRING,
            write_type: TypeId::STRING,
            optional: false,
            readonly: false,
            is_method: false,
        }],
        string_index: None,
        number_index: None,
    });

    ctx.add_lower_bound(var, lower_type);
    ctx.add_upper_bound(var, upper_type);

    let result = ctx.resolve_with_constraints(var).unwrap();
    assert_eq!(result, lower_type);
}

#[test]
fn test_resolve_bounds_number_index_ignores_uppercase_exponent_leading_zeros() {
    let interner = TypeInterner::new();
    let mut ctx = InferenceContext::new(&interner);

    let var = ctx.fresh_type_param(interner.intern_string("T"));
    let name = interner.intern_string("1E+0001");

    let upper_type = interner.object_with_index(ObjectShape {
        properties: Vec::new(),
        string_index: None,
        number_index: Some(IndexSignature {
            key_type: TypeId::NUMBER,
            value_type: TypeId::NUMBER,
            readonly: false,
        }),
    });

    let lower_type = interner.object_with_index(ObjectShape {
        properties: vec![PropertyInfo {
            name,
            type_id: TypeId::STRING,
            write_type: TypeId::STRING,
            optional: false,
            readonly: false,
            is_method: false,
        }],
        string_index: None,
        number_index: None,
    });

    ctx.add_lower_bound(var, lower_type);
    ctx.add_upper_bound(var, upper_type);

    let result = ctx.resolve_with_constraints(var).unwrap();
    assert_eq!(result, lower_type);
}

#[test]
fn test_resolve_bounds_number_index_ignores_uppercase_exponent_leading_zeros_zero() {
    let interner = TypeInterner::new();
    let mut ctx = InferenceContext::new(&interner);

    let var = ctx.fresh_type_param(interner.intern_string("T"));
    let name = interner.intern_string("1E+00");

    let upper_type = interner.object_with_index(ObjectShape {
        properties: Vec::new(),
        string_index: None,
        number_index: Some(IndexSignature {
            key_type: TypeId::NUMBER,
            value_type: TypeId::NUMBER,
            readonly: false,
        }),
    });

    let lower_type = interner.object_with_index(ObjectShape {
        properties: vec![PropertyInfo {
            name,
            type_id: TypeId::STRING,
            write_type: TypeId::STRING,
            optional: false,
            readonly: false,
            is_method: false,
        }],
        string_index: None,
        number_index: None,
    });

    ctx.add_lower_bound(var, lower_type);
    ctx.add_upper_bound(var, upper_type);

    let result = ctx.resolve_with_constraints(var).unwrap();
    assert_eq!(result, lower_type);
}

#[test]
fn test_resolve_bounds_number_index_ignores_uppercase_exponent_leading_zeros_without_sign() {
    let interner = TypeInterner::new();
    let mut ctx = InferenceContext::new(&interner);

    let var = ctx.fresh_type_param(interner.intern_string("T"));
    let name = interner.intern_string("1E0001");

    let upper_type = interner.object_with_index(ObjectShape {
        properties: Vec::new(),
        string_index: None,
        number_index: Some(IndexSignature {
            key_type: TypeId::NUMBER,
            value_type: TypeId::NUMBER,
            readonly: false,
        }),
    });

    let lower_type = interner.object_with_index(ObjectShape {
        properties: vec![PropertyInfo {
            name,
            type_id: TypeId::STRING,
            write_type: TypeId::STRING,
            optional: false,
            readonly: false,
            is_method: false,
        }],
        string_index: None,
        number_index: None,
    });

    ctx.add_lower_bound(var, lower_type);
    ctx.add_upper_bound(var, upper_type);

    let result = ctx.resolve_with_constraints(var).unwrap();
    assert_eq!(result, lower_type);
}

#[test]
fn test_resolve_bounds_number_index_ignores_uppercase_exponent_negative_leading_zeros() {
    let interner = TypeInterner::new();
    let mut ctx = InferenceContext::new(&interner);

    let var = ctx.fresh_type_param(interner.intern_string("T"));
    let name = interner.intern_string("1E-0001");

    let upper_type = interner.object_with_index(ObjectShape {
        properties: Vec::new(),
        string_index: None,
        number_index: Some(IndexSignature {
            key_type: TypeId::NUMBER,
            value_type: TypeId::NUMBER,
            readonly: false,
        }),
    });

    let lower_type = interner.object_with_index(ObjectShape {
        properties: vec![PropertyInfo {
            name,
            type_id: TypeId::STRING,
            write_type: TypeId::STRING,
            optional: false,
            readonly: false,
            is_method: false,
        }],
        string_index: None,
        number_index: None,
    });

    ctx.add_lower_bound(var, lower_type);
    ctx.add_upper_bound(var, upper_type);

    let result = ctx.resolve_with_constraints(var).unwrap();
    assert_eq!(result, lower_type);
}

#[test]
fn test_resolve_bounds_number_index_ignores_mixed_case_exponent() {
    let interner = TypeInterner::new();
    let mut ctx = InferenceContext::new(&interner);

    let var = ctx.fresh_type_param(interner.intern_string("T"));
    let name = interner.intern_string("1eE1");

    let upper_type = interner.object_with_index(ObjectShape {
        properties: Vec::new(),
        string_index: None,
        number_index: Some(IndexSignature {
            key_type: TypeId::NUMBER,
            value_type: TypeId::NUMBER,
            readonly: false,
        }),
    });

    let lower_type = interner.object_with_index(ObjectShape {
        properties: vec![PropertyInfo {
            name,
            type_id: TypeId::STRING,
            write_type: TypeId::STRING,
            optional: false,
            readonly: false,
            is_method: false,
        }],
        string_index: None,
        number_index: None,
    });

    ctx.add_lower_bound(var, lower_type);
    ctx.add_upper_bound(var, upper_type);

    let result = ctx.resolve_with_constraints(var).unwrap();
    assert_eq!(result, lower_type);
}

#[test]
fn test_resolve_bounds_number_index_ignores_mixed_case_exponent_with_sign() {
    let interner = TypeInterner::new();
    let mut ctx = InferenceContext::new(&interner);

    let var = ctx.fresh_type_param(interner.intern_string("T"));
    let name = interner.intern_string("1Ee+1");

    let upper_type = interner.object_with_index(ObjectShape {
        properties: Vec::new(),
        string_index: None,
        number_index: Some(IndexSignature {
            key_type: TypeId::NUMBER,
            value_type: TypeId::NUMBER,
            readonly: false,
        }),
    });

    let lower_type = interner.object_with_index(ObjectShape {
        properties: vec![PropertyInfo {
            name,
            type_id: TypeId::STRING,
            write_type: TypeId::STRING,
            optional: false,
            readonly: false,
            is_method: false,
        }],
        string_index: None,
        number_index: None,
    });

    ctx.add_lower_bound(var, lower_type);
    ctx.add_upper_bound(var, upper_type);

    let result = ctx.resolve_with_constraints(var).unwrap();
    assert_eq!(result, lower_type);
}

#[test]
fn test_resolve_bounds_number_index_ignores_mixed_case_exponent_missing_digits() {
    let interner = TypeInterner::new();
    let mut ctx = InferenceContext::new(&interner);

    let var = ctx.fresh_type_param(interner.intern_string("T"));
    let name = interner.intern_string("1eE");

    let upper_type = interner.object_with_index(ObjectShape {
        properties: Vec::new(),
        string_index: None,
        number_index: Some(IndexSignature {
            key_type: TypeId::NUMBER,
            value_type: TypeId::NUMBER,
            readonly: false,
        }),
    });

    let lower_type = interner.object_with_index(ObjectShape {
        properties: vec![PropertyInfo {
            name,
            type_id: TypeId::STRING,
            write_type: TypeId::STRING,
            optional: false,
            readonly: false,
            is_method: false,
        }],
        string_index: None,
        number_index: None,
    });

    ctx.add_lower_bound(var, lower_type);
    ctx.add_upper_bound(var, upper_type);

    let result = ctx.resolve_with_constraints(var).unwrap();
    assert_eq!(result, lower_type);
}

#[test]
fn test_resolve_bounds_number_index_ignores_uppercase_exponent_missing_sign_with_leading_zero() {
    let interner = TypeInterner::new();
    let mut ctx = InferenceContext::new(&interner);

    let var = ctx.fresh_type_param(interner.intern_string("T"));
    let name = interner.intern_string("1E01");

    let upper_type = interner.object_with_index(ObjectShape {
        properties: Vec::new(),
        string_index: None,
        number_index: Some(IndexSignature {
            key_type: TypeId::NUMBER,
            value_type: TypeId::NUMBER,
            readonly: false,
        }),
    });

    let lower_type = interner.object_with_index(ObjectShape {
        properties: vec![PropertyInfo {
            name,
            type_id: TypeId::STRING,
            write_type: TypeId::STRING,
            optional: false,
            readonly: false,
            is_method: false,
        }],
        string_index: None,
        number_index: None,
    });

    ctx.add_lower_bound(var, lower_type);
    ctx.add_upper_bound(var, upper_type);

    let result = ctx.resolve_with_constraints(var).unwrap();
    assert_eq!(result, lower_type);
}

#[test]
fn test_resolve_bounds_number_index_ignores_mixed_case_exponent_double_sign() {
    let interner = TypeInterner::new();
    let mut ctx = InferenceContext::new(&interner);

    let var = ctx.fresh_type_param(interner.intern_string("T"));
    let name = interner.intern_string("1eE++1");

    let upper_type = interner.object_with_index(ObjectShape {
        properties: Vec::new(),
        string_index: None,
        number_index: Some(IndexSignature {
            key_type: TypeId::NUMBER,
            value_type: TypeId::NUMBER,
            readonly: false,
        }),
    });

    let lower_type = interner.object_with_index(ObjectShape {
        properties: vec![PropertyInfo {
            name,
            type_id: TypeId::STRING,
            write_type: TypeId::STRING,
            optional: false,
            readonly: false,
            is_method: false,
        }],
        string_index: None,
        number_index: None,
    });

    ctx.add_lower_bound(var, lower_type);
    ctx.add_upper_bound(var, upper_type);

    let result = ctx.resolve_with_constraints(var).unwrap();
    assert_eq!(result, lower_type);
}

#[test]
fn test_resolve_bounds_number_index_ignores_uppercase_exponent_with_lowercase_e() {
    let interner = TypeInterner::new();
    let mut ctx = InferenceContext::new(&interner);

    let var = ctx.fresh_type_param(interner.intern_string("T"));
    let name = interner.intern_string("1eE+1");

    let upper_type = interner.object_with_index(ObjectShape {
        properties: Vec::new(),
        string_index: None,
        number_index: Some(IndexSignature {
            key_type: TypeId::NUMBER,
            value_type: TypeId::NUMBER,
            readonly: false,
        }),
    });

    let lower_type = interner.object_with_index(ObjectShape {
        properties: vec![PropertyInfo {
            name,
            type_id: TypeId::STRING,
            write_type: TypeId::STRING,
            optional: false,
            readonly: false,
            is_method: false,
        }],
        string_index: None,
        number_index: None,
    });

    ctx.add_lower_bound(var, lower_type);
    ctx.add_upper_bound(var, upper_type);

    let result = ctx.resolve_with_constraints(var).unwrap();
    assert_eq!(result, lower_type);
}

#[test]
fn test_resolve_bounds_number_index_ignores_mixed_case_exponent_double_minus() {
    let interner = TypeInterner::new();
    let mut ctx = InferenceContext::new(&interner);

    let var = ctx.fresh_type_param(interner.intern_string("T"));
    let name = interner.intern_string("1Ee--1");

    let upper_type = interner.object_with_index(ObjectShape {
        properties: Vec::new(),
        string_index: None,
        number_index: Some(IndexSignature {
            key_type: TypeId::NUMBER,
            value_type: TypeId::NUMBER,
            readonly: false,
        }),
    });

    let lower_type = interner.object_with_index(ObjectShape {
        properties: vec![PropertyInfo {
            name,
            type_id: TypeId::STRING,
            write_type: TypeId::STRING,
            optional: false,
            readonly: false,
            is_method: false,
        }],
        string_index: None,
        number_index: None,
    });

    ctx.add_lower_bound(var, lower_type);
    ctx.add_upper_bound(var, upper_type);

    let result = ctx.resolve_with_constraints(var).unwrap();
    assert_eq!(result, lower_type);
}

#[test]
fn test_resolve_bounds_number_index_ignores_mixed_case_exponent_plus_minus() {
    let interner = TypeInterner::new();
    let mut ctx = InferenceContext::new(&interner);

    let var = ctx.fresh_type_param(interner.intern_string("T"));
    let name = interner.intern_string("1Ee+-1");

    let upper_type = interner.object_with_index(ObjectShape {
        properties: Vec::new(),
        string_index: None,
        number_index: Some(IndexSignature {
            key_type: TypeId::NUMBER,
            value_type: TypeId::NUMBER,
            readonly: false,
        }),
    });

    let lower_type = interner.object_with_index(ObjectShape {
        properties: vec![PropertyInfo {
            name,
            type_id: TypeId::STRING,
            write_type: TypeId::STRING,
            optional: false,
            readonly: false,
            is_method: false,
        }],
        string_index: None,
        number_index: None,
    });

    ctx.add_lower_bound(var, lower_type);
    ctx.add_upper_bound(var, upper_type);

    let result = ctx.resolve_with_constraints(var).unwrap();
    assert_eq!(result, lower_type);
}

#[test]
fn test_resolve_bounds_number_index_ignores_mixed_case_exponent_minus_plus() {
    let interner = TypeInterner::new();
    let mut ctx = InferenceContext::new(&interner);

    let var = ctx.fresh_type_param(interner.intern_string("T"));
    let name = interner.intern_string("1Ee-+1");

    let upper_type = interner.object_with_index(ObjectShape {
        properties: Vec::new(),
        string_index: None,
        number_index: Some(IndexSignature {
            key_type: TypeId::NUMBER,
            value_type: TypeId::NUMBER,
            readonly: false,
        }),
    });

    let lower_type = interner.object_with_index(ObjectShape {
        properties: vec![PropertyInfo {
            name,
            type_id: TypeId::STRING,
            write_type: TypeId::STRING,
            optional: false,
            readonly: false,
            is_method: false,
        }],
        string_index: None,
        number_index: None,
    });

    ctx.add_lower_bound(var, lower_type);
    ctx.add_upper_bound(var, upper_type);

    let result = ctx.resolve_with_constraints(var).unwrap();
    assert_eq!(result, lower_type);
}

#[test]
fn test_resolve_bounds_number_index_ignores_mixed_case_exponent_trailing_sign() {
    let interner = TypeInterner::new();
    let mut ctx = InferenceContext::new(&interner);

    let var = ctx.fresh_type_param(interner.intern_string("T"));
    let name = interner.intern_string("1Ee+");

    let upper_type = interner.object_with_index(ObjectShape {
        properties: Vec::new(),
        string_index: None,
        number_index: Some(IndexSignature {
            key_type: TypeId::NUMBER,
            value_type: TypeId::NUMBER,
            readonly: false,
        }),
    });

    let lower_type = interner.object_with_index(ObjectShape {
        properties: vec![PropertyInfo {
            name,
            type_id: TypeId::STRING,
            write_type: TypeId::STRING,
            optional: false,
            readonly: false,
            is_method: false,
        }],
        string_index: None,
        number_index: None,
    });

    ctx.add_lower_bound(var, lower_type);
    ctx.add_upper_bound(var, upper_type);

    let result = ctx.resolve_with_constraints(var).unwrap();
    assert_eq!(result, lower_type);
}

#[test]
fn test_resolve_bounds_number_index_ignores_mixed_case_exponent_trailing_minus() {
    let interner = TypeInterner::new();
    let mut ctx = InferenceContext::new(&interner);

    let var = ctx.fresh_type_param(interner.intern_string("T"));
    let name = interner.intern_string("1Ee-");

    let upper_type = interner.object_with_index(ObjectShape {
        properties: Vec::new(),
        string_index: None,
        number_index: Some(IndexSignature {
            key_type: TypeId::NUMBER,
            value_type: TypeId::NUMBER,
            readonly: false,
        }),
    });

    let lower_type = interner.object_with_index(ObjectShape {
        properties: vec![PropertyInfo {
            name,
            type_id: TypeId::STRING,
            write_type: TypeId::STRING,
            optional: false,
            readonly: false,
            is_method: false,
        }],
        string_index: None,
        number_index: None,
    });

    ctx.add_lower_bound(var, lower_type);
    ctx.add_upper_bound(var, upper_type);

    let result = ctx.resolve_with_constraints(var).unwrap();
    assert_eq!(result, lower_type);
}

#[test]
fn test_resolve_bounds_number_index_ignores_mixed_case_exponent_trailing_double_minus() {
    let interner = TypeInterner::new();
    let mut ctx = InferenceContext::new(&interner);

    let var = ctx.fresh_type_param(interner.intern_string("T"));
    let name = interner.intern_string("1Ee--");

    let upper_type = interner.object_with_index(ObjectShape {
        properties: Vec::new(),
        string_index: None,
        number_index: Some(IndexSignature {
            key_type: TypeId::NUMBER,
            value_type: TypeId::NUMBER,
            readonly: false,
        }),
    });

    let lower_type = interner.object_with_index(ObjectShape {
        properties: vec![PropertyInfo {
            name,
            type_id: TypeId::STRING,
            write_type: TypeId::STRING,
            optional: false,
            readonly: false,
            is_method: false,
        }],
        string_index: None,
        number_index: None,
    });

    ctx.add_lower_bound(var, lower_type);
    ctx.add_upper_bound(var, upper_type);

    let result = ctx.resolve_with_constraints(var).unwrap();
    assert_eq!(result, lower_type);
}

#[test]
fn test_resolve_bounds_number_index_ignores_mixed_case_exponent_leading_zeros() {
    let interner = TypeInterner::new();
    let mut ctx = InferenceContext::new(&interner);

    let var = ctx.fresh_type_param(interner.intern_string("T"));
    let name = interner.intern_string("1Ee+0001");

    let upper_type = interner.object_with_index(ObjectShape {
        properties: Vec::new(),
        string_index: None,
        number_index: Some(IndexSignature {
            key_type: TypeId::NUMBER,
            value_type: TypeId::NUMBER,
            readonly: false,
        }),
    });

    let lower_type = interner.object_with_index(ObjectShape {
        properties: vec![PropertyInfo {
            name,
            type_id: TypeId::STRING,
            write_type: TypeId::STRING,
            optional: false,
            readonly: false,
            is_method: false,
        }],
        string_index: None,
        number_index: None,
    });

    ctx.add_lower_bound(var, lower_type);
    ctx.add_upper_bound(var, upper_type);

    let result = ctx.resolve_with_constraints(var).unwrap();
    assert_eq!(result, lower_type);
}

#[test]
fn test_resolve_bounds_number_index_ignores_mixed_case_exponent_negative_leading_zeros() {
    let interner = TypeInterner::new();
    let mut ctx = InferenceContext::new(&interner);

    let var = ctx.fresh_type_param(interner.intern_string("T"));
    let name = interner.intern_string("1Ee-0001");

    let upper_type = interner.object_with_index(ObjectShape {
        properties: Vec::new(),
        string_index: None,
        number_index: Some(IndexSignature {
            key_type: TypeId::NUMBER,
            value_type: TypeId::NUMBER,
            readonly: false,
        }),
    });

    let lower_type = interner.object_with_index(ObjectShape {
        properties: vec![PropertyInfo {
            name,
            type_id: TypeId::STRING,
            write_type: TypeId::STRING,
            optional: false,
            readonly: false,
            is_method: false,
        }],
        string_index: None,
        number_index: None,
    });

    ctx.add_lower_bound(var, lower_type);
    ctx.add_upper_bound(var, upper_type);

    let result = ctx.resolve_with_constraints(var).unwrap();
    assert_eq!(result, lower_type);
}

#[test]
fn test_resolve_bounds_number_index_ignores_mixed_case_exponent_leading_zeros_without_sign() {
    let interner = TypeInterner::new();
    let mut ctx = InferenceContext::new(&interner);

    let var = ctx.fresh_type_param(interner.intern_string("T"));
    let name = interner.intern_string("1Ee0001");

    let upper_type = interner.object_with_index(ObjectShape {
        properties: Vec::new(),
        string_index: None,
        number_index: Some(IndexSignature {
            key_type: TypeId::NUMBER,
            value_type: TypeId::NUMBER,
            readonly: false,
        }),
    });

    let lower_type = interner.object_with_index(ObjectShape {
        properties: vec![PropertyInfo {
            name,
            type_id: TypeId::STRING,
            write_type: TypeId::STRING,
            optional: false,
            readonly: false,
            is_method: false,
        }],
        string_index: None,
        number_index: None,
    });

    ctx.add_lower_bound(var, lower_type);
    ctx.add_upper_bound(var, upper_type);

    let result = ctx.resolve_with_constraints(var).unwrap();
    assert_eq!(result, lower_type);
}

#[test]
fn test_resolve_bounds_number_index_ignores_mixed_case_exponent_missing_sign_with_leading_zeros() {
    let interner = TypeInterner::new();
    let mut ctx = InferenceContext::new(&interner);

    let var = ctx.fresh_type_param(interner.intern_string("T"));
    let name = interner.intern_string("1Ee01");

    let upper_type = interner.object_with_index(ObjectShape {
        properties: Vec::new(),
        string_index: None,
        number_index: Some(IndexSignature {
            key_type: TypeId::NUMBER,
            value_type: TypeId::NUMBER,
            readonly: false,
        }),
    });

    let lower_type = interner.object_with_index(ObjectShape {
        properties: vec![PropertyInfo {
            name,
            type_id: TypeId::STRING,
            write_type: TypeId::STRING,
            optional: false,
            readonly: false,
            is_method: false,
        }],
        string_index: None,
        number_index: None,
    });

    ctx.add_lower_bound(var, lower_type);
    ctx.add_upper_bound(var, upper_type);

    let result = ctx.resolve_with_constraints(var).unwrap();
    assert_eq!(result, lower_type);
}

#[test]
fn test_resolve_bounds_number_index_ignores_mixed_case_negative_exponent_zero() {
    let interner = TypeInterner::new();
    let mut ctx = InferenceContext::new(&interner);

    let var = ctx.fresh_type_param(interner.intern_string("T"));
    let name = interner.intern_string("1Ee-0");

    let upper_type = interner.object_with_index(ObjectShape {
        properties: Vec::new(),
        string_index: None,
        number_index: Some(IndexSignature {
            key_type: TypeId::NUMBER,
            value_type: TypeId::NUMBER,
            readonly: false,
        }),
    });

    let lower_type = interner.object_with_index(ObjectShape {
        properties: vec![PropertyInfo {
            name,
            type_id: TypeId::STRING,
            write_type: TypeId::STRING,
            optional: false,
            readonly: false,
            is_method: false,
        }],
        string_index: None,
        number_index: None,
    });

    ctx.add_lower_bound(var, lower_type);
    ctx.add_upper_bound(var, upper_type);

    let result = ctx.resolve_with_constraints(var).unwrap();
    assert_eq!(result, lower_type);
}

#[test]
fn test_resolve_bounds_number_index_ignores_mixed_case_exponent_positive_zero() {
    let interner = TypeInterner::new();
    let mut ctx = InferenceContext::new(&interner);

    let var = ctx.fresh_type_param(interner.intern_string("T"));
    let name = interner.intern_string("1Ee+0");

    let upper_type = interner.object_with_index(ObjectShape {
        properties: Vec::new(),
        string_index: None,
        number_index: Some(IndexSignature {
            key_type: TypeId::NUMBER,
            value_type: TypeId::NUMBER,
            readonly: false,
        }),
    });

    let lower_type = interner.object_with_index(ObjectShape {
        properties: vec![PropertyInfo {
            name,
            type_id: TypeId::STRING,
            write_type: TypeId::STRING,
            optional: false,
            readonly: false,
            is_method: false,
        }],
        string_index: None,
        number_index: None,
    });

    ctx.add_lower_bound(var, lower_type);
    ctx.add_upper_bound(var, upper_type);

    let result = ctx.resolve_with_constraints(var).unwrap();
    assert_eq!(result, lower_type);
}

#[test]
fn test_resolve_bounds_number_index_ignores_mixed_case_exponent_zero_without_sign() {
    let interner = TypeInterner::new();
    let mut ctx = InferenceContext::new(&interner);

    let var = ctx.fresh_type_param(interner.intern_string("T"));
    let name = interner.intern_string("1Ee0");

    let upper_type = interner.object_with_index(ObjectShape {
        properties: Vec::new(),
        string_index: None,
        number_index: Some(IndexSignature {
            key_type: TypeId::NUMBER,
            value_type: TypeId::NUMBER,
            readonly: false,
        }),
    });

    let lower_type = interner.object_with_index(ObjectShape {
        properties: vec![PropertyInfo {
            name,
            type_id: TypeId::STRING,
            write_type: TypeId::STRING,
            optional: false,
            readonly: false,
            is_method: false,
        }],
        string_index: None,
        number_index: None,
    });

    ctx.add_lower_bound(var, lower_type);
    ctx.add_upper_bound(var, upper_type);

    let result = ctx.resolve_with_constraints(var).unwrap();
    assert_eq!(result, lower_type);
}

#[test]
fn test_resolve_bounds_number_index_ignores_mixed_case_exponent_double_sign_trailing() {
    let interner = TypeInterner::new();
    let mut ctx = InferenceContext::new(&interner);

    let var = ctx.fresh_type_param(interner.intern_string("T"));
    let name = interner.intern_string("1Ee++");

    let upper_type = interner.object_with_index(ObjectShape {
        properties: Vec::new(),
        string_index: None,
        number_index: Some(IndexSignature {
            key_type: TypeId::NUMBER,
            value_type: TypeId::NUMBER,
            readonly: false,
        }),
    });

    let lower_type = interner.object_with_index(ObjectShape {
        properties: vec![PropertyInfo {
            name,
            type_id: TypeId::STRING,
            write_type: TypeId::STRING,
            optional: false,
            readonly: false,
            is_method: false,
        }],
        string_index: None,
        number_index: None,
    });

    ctx.add_lower_bound(var, lower_type);
    ctx.add_upper_bound(var, upper_type);

    let result = ctx.resolve_with_constraints(var).unwrap();
    assert_eq!(result, lower_type);
}

#[test]
fn test_resolve_bounds_number_index_ignores_uppercase_exponent_missing_digits() {
    let interner = TypeInterner::new();
    let mut ctx = InferenceContext::new(&interner);

    let var = ctx.fresh_type_param(interner.intern_string("T"));
    let name = interner.intern_string("1E+");

    let upper_type = interner.object_with_index(ObjectShape {
        properties: Vec::new(),
        string_index: None,
        number_index: Some(IndexSignature {
            key_type: TypeId::NUMBER,
            value_type: TypeId::NUMBER,
            readonly: false,
        }),
    });

    let lower_type = interner.object_with_index(ObjectShape {
        properties: vec![PropertyInfo {
            name,
            type_id: TypeId::STRING,
            write_type: TypeId::STRING,
            optional: false,
            readonly: false,
            is_method: false,
        }],
        string_index: None,
        number_index: None,
    });

    ctx.add_lower_bound(var, lower_type);
    ctx.add_upper_bound(var, upper_type);

    let result = ctx.resolve_with_constraints(var).unwrap();
    assert_eq!(result, lower_type);
}

#[test]
fn test_resolve_bounds_number_index_ignores_uppercase_exponent_minus_missing_digits() {
    let interner = TypeInterner::new();
    let mut ctx = InferenceContext::new(&interner);

    let var = ctx.fresh_type_param(interner.intern_string("T"));
    let name = interner.intern_string("1E-");

    let upper_type = interner.object_with_index(ObjectShape {
        properties: Vec::new(),
        string_index: None,
        number_index: Some(IndexSignature {
            key_type: TypeId::NUMBER,
            value_type: TypeId::NUMBER,
            readonly: false,
        }),
    });

    let lower_type = interner.object_with_index(ObjectShape {
        properties: vec![PropertyInfo {
            name,
            type_id: TypeId::STRING,
            write_type: TypeId::STRING,
            optional: false,
            readonly: false,
            is_method: false,
        }],
        string_index: None,
        number_index: None,
    });

    ctx.add_lower_bound(var, lower_type);
    ctx.add_upper_bound(var, upper_type);

    let result = ctx.resolve_with_constraints(var).unwrap();
    assert_eq!(result, lower_type);
}

#[test]
fn test_resolve_bounds_number_index_ignores_uppercase_exponent_double_sign() {
    let interner = TypeInterner::new();
    let mut ctx = InferenceContext::new(&interner);

    let var = ctx.fresh_type_param(interner.intern_string("T"));
    let name = interner.intern_string("1E++1");

    let upper_type = interner.object_with_index(ObjectShape {
        properties: Vec::new(),
        string_index: None,
        number_index: Some(IndexSignature {
            key_type: TypeId::NUMBER,
            value_type: TypeId::NUMBER,
            readonly: false,
        }),
    });

    let lower_type = interner.object_with_index(ObjectShape {
        properties: vec![PropertyInfo {
            name,
            type_id: TypeId::STRING,
            write_type: TypeId::STRING,
            optional: false,
            readonly: false,
            is_method: false,
        }],
        string_index: None,
        number_index: None,
    });

    ctx.add_lower_bound(var, lower_type);
    ctx.add_upper_bound(var, upper_type);

    let result = ctx.resolve_with_constraints(var).unwrap();
    assert_eq!(result, lower_type);
}

#[test]
fn test_resolve_bounds_number_index_ignores_uppercase_exponent_double_minus() {
    let interner = TypeInterner::new();
    let mut ctx = InferenceContext::new(&interner);

    let var = ctx.fresh_type_param(interner.intern_string("T"));
    let name = interner.intern_string("1E--1");

    let upper_type = interner.object_with_index(ObjectShape {
        properties: Vec::new(),
        string_index: None,
        number_index: Some(IndexSignature {
            key_type: TypeId::NUMBER,
            value_type: TypeId::NUMBER,
            readonly: false,
        }),
    });

    let lower_type = interner.object_with_index(ObjectShape {
        properties: vec![PropertyInfo {
            name,
            type_id: TypeId::STRING,
            write_type: TypeId::STRING,
            optional: false,
            readonly: false,
            is_method: false,
        }],
        string_index: None,
        number_index: None,
    });

    ctx.add_lower_bound(var, lower_type);
    ctx.add_upper_bound(var, upper_type);

    let result = ctx.resolve_with_constraints(var).unwrap();
    assert_eq!(result, lower_type);
}

#[test]
fn test_resolve_bounds_number_index_ignores_exponent_leading_zeros_negative() {
    let interner = TypeInterner::new();
    let mut ctx = InferenceContext::new(&interner);

    let var = ctx.fresh_type_param(interner.intern_string("T"));
    let name = interner.intern_string("1e-0001");

    let upper_type = interner.object_with_index(ObjectShape {
        properties: Vec::new(),
        string_index: None,
        number_index: Some(IndexSignature {
            key_type: TypeId::NUMBER,
            value_type: TypeId::NUMBER,
            readonly: false,
        }),
    });

    let lower_type = interner.object_with_index(ObjectShape {
        properties: vec![PropertyInfo {
            name,
            type_id: TypeId::STRING,
            write_type: TypeId::STRING,
            optional: false,
            readonly: false,
            is_method: false,
        }],
        string_index: None,
        number_index: None,
    });

    ctx.add_lower_bound(var, lower_type);
    ctx.add_upper_bound(var, upper_type);

    let result = ctx.resolve_with_constraints(var).unwrap();
    assert_eq!(result, lower_type);
}

#[test]
fn test_resolve_bounds_number_index_ignores_exponent_leading_zeros_positive() {
    let interner = TypeInterner::new();
    let mut ctx = InferenceContext::new(&interner);

    let var = ctx.fresh_type_param(interner.intern_string("T"));
    let name = interner.intern_string("1e+0001");

    let upper_type = interner.object_with_index(ObjectShape {
        properties: Vec::new(),
        string_index: None,
        number_index: Some(IndexSignature {
            key_type: TypeId::NUMBER,
            value_type: TypeId::NUMBER,
            readonly: false,
        }),
    });

    let lower_type = interner.object_with_index(ObjectShape {
        properties: vec![PropertyInfo {
            name,
            type_id: TypeId::STRING,
            write_type: TypeId::STRING,
            optional: false,
            readonly: false,
            is_method: false,
        }],
        string_index: None,
        number_index: None,
    });

    ctx.add_lower_bound(var, lower_type);
    ctx.add_upper_bound(var, upper_type);

    let result = ctx.resolve_with_constraints(var).unwrap();
    assert_eq!(result, lower_type);
}

#[test]
fn test_resolve_bounds_number_index_ignores_exponent_leading_zeros_without_sign() {
    let interner = TypeInterner::new();
    let mut ctx = InferenceContext::new(&interner);

    let var = ctx.fresh_type_param(interner.intern_string("T"));
    let name = interner.intern_string("1e0001");

    let upper_type = interner.object_with_index(ObjectShape {
        properties: Vec::new(),
        string_index: None,
        number_index: Some(IndexSignature {
            key_type: TypeId::NUMBER,
            value_type: TypeId::NUMBER,
            readonly: false,
        }),
    });

    let lower_type = interner.object_with_index(ObjectShape {
        properties: vec![PropertyInfo {
            name,
            type_id: TypeId::STRING,
            write_type: TypeId::STRING,
            optional: false,
            readonly: false,
            is_method: false,
        }],
        string_index: None,
        number_index: None,
    });

    ctx.add_lower_bound(var, lower_type);
    ctx.add_upper_bound(var, upper_type);

    let result = ctx.resolve_with_constraints(var).unwrap();
    assert_eq!(result, lower_type);
}

#[test]
fn test_resolve_bounds_number_index_ignores_missing_exponent_sign() {
    let interner = TypeInterner::new();
    let mut ctx = InferenceContext::new(&interner);

    let var = ctx.fresh_type_param(interner.intern_string("T"));
    let name = interner.intern_string("1e21");

    let upper_type = interner.object_with_index(ObjectShape {
        properties: Vec::new(),
        string_index: None,
        number_index: Some(IndexSignature {
            key_type: TypeId::NUMBER,
            value_type: TypeId::NUMBER,
            readonly: false,
        }),
    });

    let lower_type = interner.object_with_index(ObjectShape {
        properties: vec![PropertyInfo {
            name,
            type_id: TypeId::STRING,
            write_type: TypeId::STRING,
            optional: false,
            readonly: false,
            is_method: false,
        }],
        string_index: None,
        number_index: None,
    });

    ctx.add_lower_bound(var, lower_type);
    ctx.add_upper_bound(var, upper_type);

    let result = ctx.resolve_with_constraints(var).unwrap();
    assert_eq!(result, lower_type);
}

#[test]
fn test_resolve_bounds_number_index_ignores_leading_zero_decimal_name() {
    let interner = TypeInterner::new();
    let mut ctx = InferenceContext::new(&interner);

    let var = ctx.fresh_type_param(interner.intern_string("T"));
    let name = interner.intern_string("01.0");

    let upper_type = interner.object_with_index(ObjectShape {
        properties: Vec::new(),
        string_index: None,
        number_index: Some(IndexSignature {
            key_type: TypeId::NUMBER,
            value_type: TypeId::NUMBER,
            readonly: false,
        }),
    });

    let lower_type = interner.object_with_index(ObjectShape {
        properties: vec![PropertyInfo {
            name,
            type_id: TypeId::STRING,
            write_type: TypeId::STRING,
            optional: false,
            readonly: false,
            is_method: false,
        }],
        string_index: None,
        number_index: None,
    });

    ctx.add_lower_bound(var, lower_type);
    ctx.add_upper_bound(var, upper_type);

    let result = ctx.resolve_with_constraints(var).unwrap();
    assert_eq!(result, lower_type);
}

#[test]
fn test_resolve_bounds_number_index_ignores_hex_name() {
    let interner = TypeInterner::new();
    let mut ctx = InferenceContext::new(&interner);

    let var = ctx.fresh_type_param(interner.intern_string("T"));
    let name = interner.intern_string("0x1");

    let upper_type = interner.object_with_index(ObjectShape {
        properties: Vec::new(),
        string_index: None,
        number_index: Some(IndexSignature {
            key_type: TypeId::NUMBER,
            value_type: TypeId::NUMBER,
            readonly: false,
        }),
    });

    let lower_type = interner.object_with_index(ObjectShape {
        properties: vec![PropertyInfo {
            name,
            type_id: TypeId::STRING,
            write_type: TypeId::STRING,
            optional: false,
            readonly: false,
            is_method: false,
        }],
        string_index: None,
        number_index: None,
    });

    ctx.add_lower_bound(var, lower_type);
    ctx.add_upper_bound(var, upper_type);

    let result = ctx.resolve_with_constraints(var).unwrap();
    assert_eq!(result, lower_type);
}

#[test]
fn test_resolve_bounds_number_index_ignores_binary_name() {
    let interner = TypeInterner::new();
    let mut ctx = InferenceContext::new(&interner);

    let var = ctx.fresh_type_param(interner.intern_string("T"));
    let name = interner.intern_string("0b1");

    let upper_type = interner.object_with_index(ObjectShape {
        properties: Vec::new(),
        string_index: None,
        number_index: Some(IndexSignature {
            key_type: TypeId::NUMBER,
            value_type: TypeId::NUMBER,
            readonly: false,
        }),
    });

    let lower_type = interner.object_with_index(ObjectShape {
        properties: vec![PropertyInfo {
            name,
            type_id: TypeId::STRING,
            write_type: TypeId::STRING,
            optional: false,
            readonly: false,
            is_method: false,
        }],
        string_index: None,
        number_index: None,
    });

    ctx.add_lower_bound(var, lower_type);
    ctx.add_upper_bound(var, upper_type);

    let result = ctx.resolve_with_constraints(var).unwrap();
    assert_eq!(result, lower_type);
}

#[test]
fn test_resolve_bounds_number_index_ignores_octal_name() {
    let interner = TypeInterner::new();
    let mut ctx = InferenceContext::new(&interner);

    let var = ctx.fresh_type_param(interner.intern_string("T"));
    let name = interner.intern_string("0o7");

    let upper_type = interner.object_with_index(ObjectShape {
        properties: Vec::new(),
        string_index: None,
        number_index: Some(IndexSignature {
            key_type: TypeId::NUMBER,
            value_type: TypeId::NUMBER,
            readonly: false,
        }),
    });

    let lower_type = interner.object_with_index(ObjectShape {
        properties: vec![PropertyInfo {
            name,
            type_id: TypeId::STRING,
            write_type: TypeId::STRING,
            optional: false,
            readonly: false,
            is_method: false,
        }],
        string_index: None,
        number_index: None,
    });

    ctx.add_lower_bound(var, lower_type);
    ctx.add_upper_bound(var, upper_type);

    let result = ctx.resolve_with_constraints(var).unwrap();
    assert_eq!(result, lower_type);
}

#[test]
fn test_resolve_bounds_number_index_ignores_exponent_leading_zero_mantissa() {
    let interner = TypeInterner::new();
    let mut ctx = InferenceContext::new(&interner);

    let var = ctx.fresh_type_param(interner.intern_string("T"));
    let name = interner.intern_string("01e+1");

    let upper_type = interner.object_with_index(ObjectShape {
        properties: Vec::new(),
        string_index: None,
        number_index: Some(IndexSignature {
            key_type: TypeId::NUMBER,
            value_type: TypeId::NUMBER,
            readonly: false,
        }),
    });

    let lower_type = interner.object_with_index(ObjectShape {
        properties: vec![PropertyInfo {
            name,
            type_id: TypeId::STRING,
            write_type: TypeId::STRING,
            optional: false,
            readonly: false,
            is_method: false,
        }],
        string_index: None,
        number_index: None,
    });

    ctx.add_lower_bound(var, lower_type);
    ctx.add_upper_bound(var, upper_type);

    let result = ctx.resolve_with_constraints(var).unwrap();
    assert_eq!(result, lower_type);
}

#[test]
fn test_resolve_bounds_number_index_ignores_leading_dot_decimal_name() {
    let interner = TypeInterner::new();
    let mut ctx = InferenceContext::new(&interner);

    let var = ctx.fresh_type_param(interner.intern_string("T"));
    let name = interner.intern_string(".5");

    let upper_type = interner.object_with_index(ObjectShape {
        properties: Vec::new(),
        string_index: None,
        number_index: Some(IndexSignature {
            key_type: TypeId::NUMBER,
            value_type: TypeId::NUMBER,
            readonly: false,
        }),
    });

    let lower_type = interner.object_with_index(ObjectShape {
        properties: vec![PropertyInfo {
            name,
            type_id: TypeId::STRING,
            write_type: TypeId::STRING,
            optional: false,
            readonly: false,
            is_method: false,
        }],
        string_index: None,
        number_index: None,
    });

    ctx.add_lower_bound(var, lower_type);
    ctx.add_upper_bound(var, upper_type);

    let result = ctx.resolve_with_constraints(var).unwrap();
    assert_eq!(result, lower_type);
}

#[test]
fn test_resolve_bounds_number_index_ignores_multiple_leading_zeros() {
    let interner = TypeInterner::new();
    let mut ctx = InferenceContext::new(&interner);

    let var = ctx.fresh_type_param(interner.intern_string("T"));
    let name = interner.intern_string("00");

    let upper_type = interner.object_with_index(ObjectShape {
        properties: Vec::new(),
        string_index: None,
        number_index: Some(IndexSignature {
            key_type: TypeId::NUMBER,
            value_type: TypeId::NUMBER,
            readonly: false,
        }),
    });

    let lower_type = interner.object_with_index(ObjectShape {
        properties: vec![PropertyInfo {
            name,
            type_id: TypeId::STRING,
            write_type: TypeId::STRING,
            optional: false,
            readonly: false,
            is_method: false,
        }],
        string_index: None,
        number_index: None,
    });

    ctx.add_lower_bound(var, lower_type);
    ctx.add_upper_bound(var, upper_type);

    let result = ctx.resolve_with_constraints(var).unwrap();
    assert_eq!(result, lower_type);
}

#[test]
fn test_resolve_bounds_number_index_ignores_negative_hex_name() {
    let interner = TypeInterner::new();
    let mut ctx = InferenceContext::new(&interner);

    let var = ctx.fresh_type_param(interner.intern_string("T"));
    let name = interner.intern_string("-0x1");

    let upper_type = interner.object_with_index(ObjectShape {
        properties: Vec::new(),
        string_index: None,
        number_index: Some(IndexSignature {
            key_type: TypeId::NUMBER,
            value_type: TypeId::NUMBER,
            readonly: false,
        }),
    });

    let lower_type = interner.object_with_index(ObjectShape {
        properties: vec![PropertyInfo {
            name,
            type_id: TypeId::STRING,
            write_type: TypeId::STRING,
            optional: false,
            readonly: false,
            is_method: false,
        }],
        string_index: None,
        number_index: None,
    });

    ctx.add_lower_bound(var, lower_type);
    ctx.add_upper_bound(var, upper_type);

    let result = ctx.resolve_with_constraints(var).unwrap();
    assert_eq!(result, lower_type);
}

#[test]
fn test_resolve_bounds_number_index_ignores_negative_binary_name() {
    let interner = TypeInterner::new();
    let mut ctx = InferenceContext::new(&interner);

    let var = ctx.fresh_type_param(interner.intern_string("T"));
    let name = interner.intern_string("-0b1");

    let upper_type = interner.object_with_index(ObjectShape {
        properties: Vec::new(),
        string_index: None,
        number_index: Some(IndexSignature {
            key_type: TypeId::NUMBER,
            value_type: TypeId::NUMBER,
            readonly: false,
        }),
    });

    let lower_type = interner.object_with_index(ObjectShape {
        properties: vec![PropertyInfo {
            name,
            type_id: TypeId::STRING,
            write_type: TypeId::STRING,
            optional: false,
            readonly: false,
            is_method: false,
        }],
        string_index: None,
        number_index: None,
    });

    ctx.add_lower_bound(var, lower_type);
    ctx.add_upper_bound(var, upper_type);

    let result = ctx.resolve_with_constraints(var).unwrap();
    assert_eq!(result, lower_type);
}

#[test]
fn test_resolve_bounds_number_index_ignores_negative_octal_name() {
    let interner = TypeInterner::new();
    let mut ctx = InferenceContext::new(&interner);

    let var = ctx.fresh_type_param(interner.intern_string("T"));
    let name = interner.intern_string("-0o7");

    let upper_type = interner.object_with_index(ObjectShape {
        properties: Vec::new(),
        string_index: None,
        number_index: Some(IndexSignature {
            key_type: TypeId::NUMBER,
            value_type: TypeId::NUMBER,
            readonly: false,
        }),
    });

    let lower_type = interner.object_with_index(ObjectShape {
        properties: vec![PropertyInfo {
            name,
            type_id: TypeId::STRING,
            write_type: TypeId::STRING,
            optional: false,
            readonly: false,
            is_method: false,
        }],
        string_index: None,
        number_index: None,
    });

    ctx.add_lower_bound(var, lower_type);
    ctx.add_upper_bound(var, upper_type);

    let result = ctx.resolve_with_constraints(var).unwrap();
    assert_eq!(result, lower_type);
}

#[test]
fn test_resolve_bounds_number_index_ignores_exponent_double_sign() {
    let interner = TypeInterner::new();
    let mut ctx = InferenceContext::new(&interner);

    let var = ctx.fresh_type_param(interner.intern_string("T"));
    let name = interner.intern_string("1e++1");

    let upper_type = interner.object_with_index(ObjectShape {
        properties: Vec::new(),
        string_index: None,
        number_index: Some(IndexSignature {
            key_type: TypeId::NUMBER,
            value_type: TypeId::NUMBER,
            readonly: false,
        }),
    });

    let lower_type = interner.object_with_index(ObjectShape {
        properties: vec![PropertyInfo {
            name,
            type_id: TypeId::STRING,
            write_type: TypeId::STRING,
            optional: false,
            readonly: false,
            is_method: false,
        }],
        string_index: None,
        number_index: None,
    });

    ctx.add_lower_bound(var, lower_type);
    ctx.add_upper_bound(var, upper_type);

    let result = ctx.resolve_with_constraints(var).unwrap();
    assert_eq!(result, lower_type);
}

#[test]
fn test_resolve_bounds_number_index_ignores_exponent_double_minus() {
    let interner = TypeInterner::new();
    let mut ctx = InferenceContext::new(&interner);

    let var = ctx.fresh_type_param(interner.intern_string("T"));
    let name = interner.intern_string("1e--1");

    let upper_type = interner.object_with_index(ObjectShape {
        properties: Vec::new(),
        string_index: None,
        number_index: Some(IndexSignature {
            key_type: TypeId::NUMBER,
            value_type: TypeId::NUMBER,
            readonly: false,
        }),
    });

    let lower_type = interner.object_with_index(ObjectShape {
        properties: vec![PropertyInfo {
            name,
            type_id: TypeId::STRING,
            write_type: TypeId::STRING,
            optional: false,
            readonly: false,
            is_method: false,
        }],
        string_index: None,
        number_index: None,
    });

    ctx.add_lower_bound(var, lower_type);
    ctx.add_upper_bound(var, upper_type);

    let result = ctx.resolve_with_constraints(var).unwrap();
    assert_eq!(result, lower_type);
}

#[test]
fn test_resolve_bounds_number_index_ignores_exponent_missing_digits() {
    let interner = TypeInterner::new();
    let mut ctx = InferenceContext::new(&interner);

    let var = ctx.fresh_type_param(interner.intern_string("T"));
    let name = interner.intern_string("1e+");

    let upper_type = interner.object_with_index(ObjectShape {
        properties: Vec::new(),
        string_index: None,
        number_index: Some(IndexSignature {
            key_type: TypeId::NUMBER,
            value_type: TypeId::NUMBER,
            readonly: false,
        }),
    });

    let lower_type = interner.object_with_index(ObjectShape {
        properties: vec![PropertyInfo {
            name,
            type_id: TypeId::STRING,
            write_type: TypeId::STRING,
            optional: false,
            readonly: false,
            is_method: false,
        }],
        string_index: None,
        number_index: None,
    });

    ctx.add_lower_bound(var, lower_type);
    ctx.add_upper_bound(var, upper_type);

    let result = ctx.resolve_with_constraints(var).unwrap();
    assert_eq!(result, lower_type);
}

#[test]
fn test_resolve_bounds_number_index_ignores_exponent_minus_missing_digits() {
    let interner = TypeInterner::new();
    let mut ctx = InferenceContext::new(&interner);

    let var = ctx.fresh_type_param(interner.intern_string("T"));
    let name = interner.intern_string("1e-");

    let upper_type = interner.object_with_index(ObjectShape {
        properties: Vec::new(),
        string_index: None,
        number_index: Some(IndexSignature {
            key_type: TypeId::NUMBER,
            value_type: TypeId::NUMBER,
            readonly: false,
        }),
    });

    let lower_type = interner.object_with_index(ObjectShape {
        properties: vec![PropertyInfo {
            name,
            type_id: TypeId::STRING,
            write_type: TypeId::STRING,
            optional: false,
            readonly: false,
            is_method: false,
        }],
        string_index: None,
        number_index: None,
    });

    ctx.add_lower_bound(var, lower_type);
    ctx.add_upper_bound(var, upper_type);

    let result = ctx.resolve_with_constraints(var).unwrap();
    assert_eq!(result, lower_type);
}

#[test]
fn test_resolve_bounds_number_index_ignores_negative_exponent_zero() {
    let interner = TypeInterner::new();
    let mut ctx = InferenceContext::new(&interner);

    let var = ctx.fresh_type_param(interner.intern_string("T"));
    let name = interner.intern_string("-0e+0");

    let upper_type = interner.object_with_index(ObjectShape {
        properties: Vec::new(),
        string_index: None,
        number_index: Some(IndexSignature {
            key_type: TypeId::NUMBER,
            value_type: TypeId::NUMBER,
            readonly: false,
        }),
    });

    let lower_type = interner.object_with_index(ObjectShape {
        properties: vec![PropertyInfo {
            name,
            type_id: TypeId::STRING,
            write_type: TypeId::STRING,
            optional: false,
            readonly: false,
            is_method: false,
        }],
        string_index: None,
        number_index: None,
    });

    ctx.add_lower_bound(var, lower_type);
    ctx.add_upper_bound(var, upper_type);

    let result = ctx.resolve_with_constraints(var).unwrap();
    assert_eq!(result, lower_type);
}

#[test]
fn test_resolve_bounds_number_index_ignores_positive_exponent_zero() {
    let interner = TypeInterner::new();
    let mut ctx = InferenceContext::new(&interner);

    let var = ctx.fresh_type_param(interner.intern_string("T"));
    let name = interner.intern_string("1e+0");

    let upper_type = interner.object_with_index(ObjectShape {
        properties: Vec::new(),
        string_index: None,
        number_index: Some(IndexSignature {
            key_type: TypeId::NUMBER,
            value_type: TypeId::NUMBER,
            readonly: false,
        }),
    });

    let lower_type = interner.object_with_index(ObjectShape {
        properties: vec![PropertyInfo {
            name,
            type_id: TypeId::STRING,
            write_type: TypeId::STRING,
            optional: false,
            readonly: false,
            is_method: false,
        }],
        string_index: None,
        number_index: None,
    });

    ctx.add_lower_bound(var, lower_type);
    ctx.add_upper_bound(var, upper_type);

    let result = ctx.resolve_with_constraints(var).unwrap();
    assert_eq!(result, lower_type);
}

#[test]
fn test_resolve_bounds_number_index_accepts_negative_decimal_boundary_name() {
    let interner = TypeInterner::new();
    let mut ctx = InferenceContext::new(&interner);

    let var = ctx.fresh_type_param(interner.intern_string("T"));
    let name = interner.intern_string("-0.000001");

    let upper_type = interner.object_with_index(ObjectShape {
        properties: Vec::new(),
        string_index: None,
        number_index: Some(IndexSignature {
            key_type: TypeId::NUMBER,
            value_type: TypeId::NUMBER,
            readonly: false,
        }),
    });

    let lower_type = interner.object_with_index(ObjectShape {
        properties: vec![PropertyInfo {
            name,
            type_id: TypeId::STRING,
            write_type: TypeId::STRING,
            optional: false,
            readonly: false,
            is_method: false,
        }],
        string_index: None,
        number_index: None,
    });

    ctx.add_lower_bound(var, lower_type);
    ctx.add_upper_bound(var, upper_type);

    let result = ctx.resolve_with_constraints(var);
    assert!(matches!(
        result,
        Err(InferenceError::BoundsViolation {
            lower: actual_lower,
            upper: actual_upper,
            ..
        }) if actual_lower == lower_type && actual_upper == upper_type
    ));
}

#[test]
fn test_resolve_bounds_number_index_ignores_trailing_decimal_name() {
    let interner = TypeInterner::new();
    let mut ctx = InferenceContext::new(&interner);

    let var = ctx.fresh_type_param(interner.intern_string("T"));
    let name = interner.intern_string("1.");

    let upper_type = interner.object_with_index(ObjectShape {
        properties: Vec::new(),
        string_index: None,
        number_index: Some(IndexSignature {
            key_type: TypeId::NUMBER,
            value_type: TypeId::NUMBER,
            readonly: false,
        }),
    });

    let lower_type = interner.object_with_index(ObjectShape {
        properties: vec![PropertyInfo {
            name,
            type_id: TypeId::STRING,
            write_type: TypeId::STRING,
            optional: false,
            readonly: false,
            is_method: false,
        }],
        string_index: None,
        number_index: None,
    });

    ctx.add_lower_bound(var, lower_type);
    ctx.add_upper_bound(var, upper_type);

    let result = ctx.resolve_with_constraints(var).unwrap();
    assert_eq!(result, lower_type);
}

#[test]
fn test_resolve_bounds_number_index_ignores_leading_plus_name() {
    let interner = TypeInterner::new();
    let mut ctx = InferenceContext::new(&interner);

    let var = ctx.fresh_type_param(interner.intern_string("T"));
    let name = interner.intern_string("+1");

    let upper_type = interner.object_with_index(ObjectShape {
        properties: Vec::new(),
        string_index: None,
        number_index: Some(IndexSignature {
            key_type: TypeId::NUMBER,
            value_type: TypeId::NUMBER,
            readonly: false,
        }),
    });

    let lower_type = interner.object_with_index(ObjectShape {
        properties: vec![PropertyInfo {
            name,
            type_id: TypeId::STRING,
            write_type: TypeId::STRING,
            optional: false,
            readonly: false,
            is_method: false,
        }],
        string_index: None,
        number_index: None,
    });

    ctx.add_lower_bound(var, lower_type);
    ctx.add_upper_bound(var, upper_type);

    let result = ctx.resolve_with_constraints(var).unwrap();
    assert_eq!(result, lower_type);
}

#[test]
fn test_resolve_bounds_inconsistent_index_signatures() {
    let interner = TypeInterner::new();
    let mut ctx = InferenceContext::new(&interner);

    let var = ctx.fresh_type_param(interner.intern_string("T"));

    let upper_type = interner.object_with_index(ObjectShape {
        properties: Vec::new(),
        string_index: Some(IndexSignature {
            key_type: TypeId::STRING,
            value_type: TypeId::NUMBER,
            readonly: false,
        }),
        number_index: None,
    });

    let lower_type = interner.object_with_index(ObjectShape {
        properties: Vec::new(),
        string_index: Some(IndexSignature {
            key_type: TypeId::STRING,
            value_type: TypeId::NUMBER,
            readonly: false,
        }),
        number_index: Some(IndexSignature {
            key_type: TypeId::NUMBER,
            value_type: TypeId::STRING,
            readonly: false,
        }),
    });

    ctx.add_lower_bound(var, lower_type);
    ctx.add_upper_bound(var, upper_type);

    let result = ctx.resolve_with_constraints(var);
    assert!(matches!(
        result,
        Err(InferenceError::BoundsViolation {
            lower: actual_lower,
            upper: actual_upper,
            ..
        }) if actual_lower == lower_type && actual_upper == upper_type
    ));
}

#[test]
fn test_resolve_bounds_object_with_index_mismatch() {
    let interner = TypeInterner::new();
    let mut ctx = InferenceContext::new(&interner);

    let var = ctx.fresh_type_param(interner.intern_string("T"));

    let upper_type = interner.object_with_index(ObjectShape {
        properties: Vec::new(),
        string_index: Some(IndexSignature {
            key_type: TypeId::STRING,
            value_type: TypeId::STRING,
            readonly: false,
        }),
        number_index: None,
    });

    let lower_type = interner.object_with_index(ObjectShape {
        properties: Vec::new(),
        string_index: Some(IndexSignature {
            key_type: TypeId::STRING,
            value_type: TypeId::NUMBER,
            readonly: false,
        }),
        number_index: None,
    });

    ctx.add_lower_bound(var, lower_type);
    ctx.add_upper_bound(var, upper_type);

    let result = ctx.resolve_with_constraints(var);
    assert!(matches!(
        result,
        Err(InferenceError::BoundsViolation {
            lower: actual_lower,
            upper: actual_upper,
            ..
        }) if actual_lower == lower_type && actual_upper == upper_type
    ));
}

#[test]
fn test_resolve_bounds_function_subtype() {
    let interner = TypeInterner::new();
    let mut ctx = InferenceContext::new(&interner);

    let var = ctx.fresh_type_param(interner.intern_string("T"));

    let source_param = ParamInfo {
        name: Some(interner.intern_string("x")),
        type_id: interner.union(vec![TypeId::STRING, TypeId::NUMBER]),
        optional: false,
        rest: false,
    };
    let target_param = ParamInfo {
        name: Some(interner.intern_string("y")),
        type_id: TypeId::STRING,
        optional: false,
        rest: false,
    };

    let lower = interner.function(FunctionShape {
        type_params: Vec::new(),
        params: vec![source_param],
        this_type: None,
        return_type: TypeId::NUMBER,
        type_predicate: None,
        is_constructor: false,
    });
    let upper = interner.function(FunctionShape {
        type_params: Vec::new(),
        params: vec![target_param],
        this_type: None,
        return_type: TypeId::NUMBER,
        type_predicate: None,
        is_constructor: false,
    });

    ctx.add_lower_bound(var, lower);
    ctx.add_upper_bound(var, upper);

    let result = ctx.resolve_with_constraints(var).unwrap();
    assert_eq!(result, lower);
}

#[test]
fn test_resolve_bounds_optional_property_compatible() {
    let interner = TypeInterner::new();
    let mut ctx = InferenceContext::new(&interner);

    let var = ctx.fresh_type_param(interner.intern_string("T"));
    let name_a = interner.intern_string("a");

    let upper = interner.object(vec![PropertyInfo {
        name: name_a,
        type_id: TypeId::STRING,
        write_type: TypeId::STRING,
        optional: true,
        readonly: false,
        is_method: false,
    }]);
    let lower = interner.object(vec![PropertyInfo {
        name: name_a,
        type_id: TypeId::STRING,
        write_type: TypeId::STRING,
        optional: false,
        readonly: false,
        is_method: false,
    }]);

    ctx.add_lower_bound(var, lower);
    ctx.add_upper_bound(var, upper);

    let result = ctx.resolve_with_constraints(var).unwrap();
    assert_eq!(result, lower);
}

#[test]
fn test_resolve_bounds_optional_property_mismatch() {
    let interner = TypeInterner::new();
    let mut ctx = InferenceContext::new(&interner);

    let var = ctx.fresh_type_param(interner.intern_string("T"));
    let name_a = interner.intern_string("a");

    let upper = interner.object(vec![PropertyInfo {
        name: name_a,
        type_id: TypeId::STRING,
        write_type: TypeId::STRING,
        optional: false,
        readonly: false,
        is_method: false,
    }]);
    let lower = interner.object(vec![PropertyInfo {
        name: name_a,
        type_id: TypeId::STRING,
        write_type: TypeId::STRING,
        optional: true,
        readonly: false,
        is_method: false,
    }]);

    ctx.add_lower_bound(var, lower);
    ctx.add_upper_bound(var, upper);

    let result = ctx.resolve_with_constraints(var);
    assert!(matches!(
        result,
        Err(InferenceError::BoundsViolation {
            lower: actual_lower,
            upper: actual_upper,
            ..
        }) if actual_lower == lower && actual_upper == upper
    ));
}

#[test]
fn test_resolve_bounds_optional_property_missing_ok() {
    let interner = TypeInterner::new();
    let mut ctx = InferenceContext::new(&interner);

    let var = ctx.fresh_type_param(interner.intern_string("T"));
    let name_a = interner.intern_string("a");

    let upper = interner.object(vec![PropertyInfo {
        name: name_a,
        type_id: TypeId::STRING,
        write_type: TypeId::STRING,
        optional: true,
        readonly: false,
        is_method: false,
    }]);
    let lower = interner.object(Vec::new());

    ctx.add_lower_bound(var, lower);
    ctx.add_upper_bound(var, upper);

    let result = ctx.resolve_with_constraints(var).unwrap();
    assert_eq!(result, lower);
}

#[test]
fn test_resolve_bounds_callable_subtype() {
    let interner = TypeInterner::new();
    let mut ctx = InferenceContext::new(&interner);

    let var = ctx.fresh_type_param(interner.intern_string("T"));

    let source_param = ParamInfo {
        name: Some(interner.intern_string("x")),
        type_id: interner.union(vec![TypeId::STRING, TypeId::NUMBER]),
        optional: false,
        rest: false,
    };
    let target_param = ParamInfo {
        name: Some(interner.intern_string("y")),
        type_id: TypeId::STRING,
        optional: false,
        rest: false,
    };

    let lower = interner.callable(CallableShape {
        call_signatures: vec![CallSignature {
            type_params: Vec::new(),
            params: vec![source_param],
            this_type: None,
            return_type: TypeId::NUMBER,
            type_predicate: None,
        }],
        construct_signatures: Vec::new(),
        properties: Vec::new(),
    });
    let upper = interner.callable(CallableShape {
        call_signatures: vec![CallSignature {
            type_params: Vec::new(),
            params: vec![target_param],
            this_type: None,
            return_type: TypeId::NUMBER,
            type_predicate: None,
        }],
        construct_signatures: Vec::new(),
        properties: Vec::new(),
    });

    ctx.add_lower_bound(var, lower);
    ctx.add_upper_bound(var, upper);

    let result = ctx.resolve_with_constraints(var).unwrap();
    assert_eq!(result, lower);
}

#[test]
fn test_resolve_bounds_function_to_callable() {
    let interner = TypeInterner::new();
    let mut ctx = InferenceContext::new(&interner);

    let var = ctx.fresh_type_param(interner.intern_string("T"));

    let source_param = ParamInfo {
        name: Some(interner.intern_string("x")),
        type_id: interner.union(vec![TypeId::STRING, TypeId::NUMBER]),
        optional: false,
        rest: false,
    };
    let target_param = ParamInfo {
        name: Some(interner.intern_string("y")),
        type_id: TypeId::STRING,
        optional: false,
        rest: false,
    };

    let lower = interner.function(FunctionShape {
        type_params: Vec::new(),
        params: vec![source_param],
        this_type: None,
        return_type: TypeId::NUMBER,
        type_predicate: None,
        is_constructor: false,
    });
    let upper = interner.callable(CallableShape {
        call_signatures: vec![CallSignature {
            type_params: Vec::new(),
            params: vec![target_param],
            this_type: None,
            return_type: TypeId::NUMBER,
            type_predicate: None,
        }],
        construct_signatures: Vec::new(),
        properties: Vec::new(),
    });

    ctx.add_lower_bound(var, lower);
    ctx.add_upper_bound(var, upper);

    let result = ctx.resolve_with_constraints(var).unwrap();
    assert_eq!(result, lower);
}

#[test]
fn test_resolve_bounds_callable_to_function() {
    let interner = TypeInterner::new();
    let mut ctx = InferenceContext::new(&interner);

    let var = ctx.fresh_type_param(interner.intern_string("T"));

    let source_param = ParamInfo {
        name: Some(interner.intern_string("x")),
        type_id: interner.union(vec![TypeId::STRING, TypeId::NUMBER]),
        optional: false,
        rest: false,
    };
    let target_param = ParamInfo {
        name: Some(interner.intern_string("y")),
        type_id: TypeId::STRING,
        optional: false,
        rest: false,
    };

    let lower = interner.callable(CallableShape {
        call_signatures: vec![CallSignature {
            type_params: Vec::new(),
            params: vec![source_param],
            this_type: None,
            return_type: TypeId::NUMBER,
            type_predicate: None,
        }],
        construct_signatures: Vec::new(),
        properties: Vec::new(),
    });
    let upper = interner.function(FunctionShape {
        type_params: Vec::new(),
        params: vec![target_param],
        this_type: None,
        return_type: TypeId::NUMBER,
        type_predicate: None,
        is_constructor: false,
    });

    ctx.add_lower_bound(var, lower);
    ctx.add_upper_bound(var, upper);

    let result = ctx.resolve_with_constraints(var).unwrap();
    assert_eq!(result, lower);
}

#[test]
fn test_resolve_bounds_application_subtype() {
    let interner = TypeInterner::new();
    let mut ctx = InferenceContext::new(&interner);

    let var = ctx.fresh_type_param(interner.intern_string("T"));
    let base = interner.reference(SymbolRef(1));
    let upper = interner.application(base, vec![TypeId::STRING]);
    let lower = interner.application(base, vec![interner.literal_string("hello")]);

    ctx.add_lower_bound(var, lower);
    ctx.add_upper_bound(var, upper);

    let result = ctx.resolve_with_constraints(var).unwrap();
    assert_eq!(result, lower);
}

#[test]
fn test_resolve_bounds_conflict() {
    let interner = TypeInterner::new();
    let mut ctx = InferenceContext::new(&interner);
    let t_name = interner.intern_string("T");

    let var = ctx.fresh_type_param(t_name);

    ctx.add_lower_bound(var, TypeId::STRING);
    ctx.add_upper_bound(var, TypeId::NUMBER);

    let result = ctx.resolve_with_constraints(var);
    assert!(matches!(
        result,
        Err(InferenceError::BoundsViolation {
            lower,
            upper,
            ..
        }) if lower == TypeId::STRING && upper == TypeId::NUMBER
    ));
}

#[test]
fn test_resolve_bounds_duplicate_upper_bounds_no_intersection() {
    let interner = TypeInterner::new();
    let mut ctx = InferenceContext::new(&interner);
    let t_name = interner.intern_string("T");

    let var = ctx.fresh_type_param(t_name);

    ctx.add_upper_bound(var, TypeId::STRING);
    ctx.add_upper_bound(var, TypeId::STRING);

    let result = ctx.resolve_with_constraints(var).unwrap();
    assert_eq!(result, TypeId::STRING);
}

#[test]
fn test_resolve_no_constraints() {
    let interner = TypeInterner::new();
    let mut ctx = InferenceContext::new(&interner);

    let var = ctx.fresh_type_param(interner.intern_string("T"));

    // No constraints at all
    let result = ctx.resolve_with_constraints(var).unwrap();
    assert_eq!(result, TypeId::UNKNOWN);
}

// =============================================================================
// Best Common Type Tests
// =============================================================================

#[test]
fn test_best_common_type_single() {
    let interner = TypeInterner::new();
    let ctx = InferenceContext::new(&interner);

    let result = ctx.best_common_type(&[TypeId::STRING]);
    assert_eq!(result, TypeId::STRING);
}

#[test]
fn test_best_common_type_union() {
    let interner = TypeInterner::new();
    let ctx = InferenceContext::new(&interner);

    let result = ctx.best_common_type(&[TypeId::STRING, TypeId::NUMBER]);
    let expected = interner.union(vec![TypeId::STRING, TypeId::NUMBER]);
    assert_eq!(result, expected);
}

#[test]
fn test_best_common_type_dedup() {
    let interner = TypeInterner::new();
    let ctx = InferenceContext::new(&interner);

    // Duplicate types should be deduped
    let result = ctx.best_common_type(&[TypeId::STRING, TypeId::STRING, TypeId::NUMBER]);
    let expected = interner.union(vec![TypeId::STRING, TypeId::NUMBER]);
    assert_eq!(result, expected);
}

#[test]
fn test_best_common_type_empty() {
    let interner = TypeInterner::new();
    let ctx = InferenceContext::new(&interner);

    let result = ctx.best_common_type(&[]);
    assert_eq!(result, TypeId::UNKNOWN);
}

#[test]
fn test_best_common_type_never_ignored() {
    let interner = TypeInterner::new();
    let ctx = InferenceContext::new(&interner);

    // never doesn't contribute to union
    let result = ctx.best_common_type(&[TypeId::STRING, TypeId::NEVER]);
    assert_eq!(result, TypeId::STRING);
}

#[test]
fn test_best_common_type_all_never() {
    let interner = TypeInterner::new();
    let ctx = InferenceContext::new(&interner);

    let result = ctx.best_common_type(&[TypeId::NEVER, TypeId::NEVER]);
    assert_eq!(result, TypeId::NEVER);
}

// =============================================================================
// Full Inference Scenario Tests
// =============================================================================

#[test]
fn test_resolve_all_with_constraints() {
    let interner = TypeInterner::new();
    let mut ctx = InferenceContext::new(&interner);
    let t_name = interner.intern_string("T");
    let u_name = interner.intern_string("U");

    // Simulate: function foo<T, U>(a: T, b: U) called with foo("hello", 42)
    let var_t = ctx.fresh_type_param(t_name);
    let var_u = ctx.fresh_type_param(u_name);

    let hello = interner.literal_string("hello");
    let forty_two = interner.literal_number(42.0);

    ctx.add_lower_bound(var_t, hello);
    ctx.add_lower_bound(var_u, forty_two);

    let results = ctx.resolve_all_with_constraints().unwrap();

    assert_eq!(results.len(), 2);
    assert_eq!(results[0], (t_name, hello));
    assert_eq!(results[1], (u_name, forty_two));
}

#[test]
fn test_resolve_all_with_circular_extends_unknown() {
    let interner = TypeInterner::new();
    let mut ctx = InferenceContext::new(&interner);
    let t_name = interner.intern_string("T");
    let u_name = interner.intern_string("U");

    let var_t = ctx.fresh_type_param(t_name);
    let var_u = ctx.fresh_type_param(u_name);

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

    // Simulate: <T extends U, U extends T>
    ctx.add_upper_bound(var_t, u_type);
    ctx.add_upper_bound(var_u, t_type);

    let results = ctx.resolve_all_with_constraints().unwrap();

    assert_eq!(results.len(), 2);
    assert_eq!(results[0], (t_name, TypeId::UNKNOWN));
    assert_eq!(results[1], (u_name, TypeId::UNKNOWN));
}
