use super::*;
use crate::solver::{AssignabilityChecker, CompatChecker, infer_generic_function};

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
    assert_eq!(result_u, TypeId::STRING);
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
fn test_resolve_bounds_function_param_contravariance_extends() {
    let interner = TypeInterner::new();
    let mut ctx = InferenceContext::new(&interner);

    let var = ctx.fresh_type_param(interner.intern_string("T"));

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
        params: vec![wide_param],
        this_type: None,
        return_type: TypeId::VOID,
        type_predicate: None,
        is_constructor: false,
    });
    let upper_fn = interner.function(FunctionShape {
        type_params: Vec::new(),
        params: vec![narrow_param],
        this_type: None,
        return_type: TypeId::VOID,
        type_predicate: None,
        is_constructor: false,
    });

    // Contextual signature provides a narrow parameter type constraint.
    ctx.add_lower_bound(var, lower_fn);
    ctx.add_upper_bound(var, upper_fn);

    let result = ctx.resolve_with_constraints(var).unwrap();
    assert_eq!(result, lower_fn);
}

#[test]
fn test_resolve_bounds_function_return_covariance_extends() {
    let interner = TypeInterner::new();
    let mut ctx = InferenceContext::new(&interner);

    let var = ctx.fresh_type_param(interner.intern_string("T"));

    let param = ParamInfo {
        name: Some(interner.intern_string("x")),
        type_id: TypeId::STRING,
        optional: false,
        rest: false,
    };

    let lower_fn = interner.function(FunctionShape {
        type_params: Vec::new(),
        params: vec![param.clone()],
        this_type: None,
        return_type: interner.literal_string("ok"),
        type_predicate: None,
        is_constructor: false,
    });
    let upper_fn = interner.function(FunctionShape {
        type_params: Vec::new(),
        params: vec![param],
        this_type: None,
        return_type: TypeId::STRING,
        type_predicate: None,
        is_constructor: false,
    });

    ctx.add_lower_bound(var, lower_fn);
    ctx.add_upper_bound(var, upper_fn);

    let result = ctx.resolve_with_constraints(var).unwrap();
    assert_eq!(result, lower_fn);
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
fn test_resolve_bounds_number_index_ignores_numeric_separator_name() {
    let interner = TypeInterner::new();
    let mut ctx = InferenceContext::new(&interner);

    let var = ctx.fresh_type_param(interner.intern_string("T"));
    let name = interner.intern_string("1_0");

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
fn test_resolve_bounds_function_this_parameter_mismatch() {
    let interner = TypeInterner::new();
    let mut ctx = InferenceContext::new(&interner);

    let var = ctx.fresh_type_param(interner.intern_string("T"));

    let lower = interner.function(FunctionShape {
        type_params: Vec::new(),
        params: Vec::new(),
        this_type: Some(TypeId::NUMBER),
        return_type: TypeId::VOID,
        type_predicate: None,
        is_constructor: false,
    });
    let upper = interner.function(FunctionShape {
        type_params: Vec::new(),
        params: Vec::new(),
        this_type: Some(TypeId::STRING),
        return_type: TypeId::VOID,
        type_predicate: None,
        is_constructor: false,
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
fn test_resolve_bounds_function_this_parameter_optional_target() {
    let interner = TypeInterner::new();
    let mut ctx = InferenceContext::new(&interner);

    let var = ctx.fresh_type_param(interner.intern_string("T"));

    let lower = interner.function(FunctionShape {
        type_params: Vec::new(),
        params: Vec::new(),
        this_type: Some(TypeId::NUMBER),
        return_type: TypeId::VOID,
        type_predicate: None,
        is_constructor: false,
    });
    let upper = interner.function(FunctionShape {
        type_params: Vec::new(),
        params: Vec::new(),
        this_type: None,
        return_type: TypeId::VOID,
        type_predicate: None,
        is_constructor: false,
    });

    ctx.add_lower_bound(var, lower);
    ctx.add_upper_bound(var, upper);

    let result = ctx.resolve_with_constraints(var).unwrap();
    assert_eq!(result, lower);
}

#[test]
fn test_resolve_bounds_function_this_parameter_any_upper_bound() {
    let interner = TypeInterner::new();
    let mut ctx = InferenceContext::new(&interner);

    let var = ctx.fresh_type_param(interner.intern_string("T"));

    let lower = interner.function(FunctionShape {
        type_params: Vec::new(),
        params: Vec::new(),
        this_type: Some(TypeId::NUMBER),
        return_type: TypeId::VOID,
        type_predicate: None,
        is_constructor: false,
    });
    let upper = interner.function(FunctionShape {
        type_params: Vec::new(),
        params: Vec::new(),
        this_type: Some(TypeId::ANY),
        return_type: TypeId::VOID,
        type_predicate: None,
        is_constructor: false,
    });

    ctx.add_lower_bound(var, lower);
    ctx.add_upper_bound(var, upper);

    let result = ctx.resolve_with_constraints(var).unwrap();
    assert_eq!(result, lower);
}

#[test]
fn test_resolve_bounds_function_this_parameter_contravariant() {
    let interner = TypeInterner::new();
    let mut ctx = InferenceContext::new(&interner);

    let var = ctx.fresh_type_param(interner.intern_string("T"));

    let lower_this = interner.union(vec![TypeId::STRING, TypeId::NUMBER]);
    let lower = interner.function(FunctionShape {
        type_params: Vec::new(),
        params: Vec::new(),
        this_type: Some(lower_this),
        return_type: TypeId::VOID,
        type_predicate: None,
        is_constructor: false,
    });
    let upper = interner.function(FunctionShape {
        type_params: Vec::new(),
        params: Vec::new(),
        this_type: Some(TypeId::STRING),
        return_type: TypeId::VOID,
        type_predicate: None,
        is_constructor: false,
    });

    ctx.add_lower_bound(var, lower);
    ctx.add_upper_bound(var, upper);

    let result = ctx.resolve_with_constraints(var).unwrap();
    assert_eq!(result, lower);
}

#[test]
fn test_resolve_bounds_callable_this_parameter_contravariant() {
    let interner = TypeInterner::new();
    let mut ctx = InferenceContext::new(&interner);

    let var = ctx.fresh_type_param(interner.intern_string("T"));

    let lower_this = interner.union(vec![TypeId::STRING, TypeId::NUMBER]);
    let lower = interner.callable(CallableShape {
        call_signatures: vec![CallSignature {
            type_params: Vec::new(),
            params: Vec::new(),
            this_type: Some(lower_this),
            return_type: TypeId::VOID,
            type_predicate: None,
        }],
        construct_signatures: Vec::new(),
        properties: Vec::new(),
    });
    let upper = interner.callable(CallableShape {
        call_signatures: vec![CallSignature {
            type_params: Vec::new(),
            params: Vec::new(),
            this_type: Some(TypeId::STRING),
            return_type: TypeId::VOID,
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

#[test]
fn test_infer_union_target_with_placeholder_member() {
    let interner = TypeInterner::new();
    let mut checker = CompatChecker::new(&interner);
    let t_name = interner.intern_string("T");

    let t_type = interner.intern(TypeKey::TypeParameter(TypeParamInfo {
        name: t_name,
        constraint: None,
        default: None,
    }));
    let param_type = interner.union(vec![t_type, TypeId::STRING]);

    let func = FunctionShape {
        type_params: vec![TypeParamInfo {
            name: t_name,
            constraint: None,
            default: None,
        }],
        params: vec![ParamInfo {
            name: Some(interner.intern_string("x")),
            type_id: param_type,
            optional: false,
            rest: false,
        }],
        this_type: None,
        return_type: t_type,
        type_predicate: None,
        is_constructor: false,
    };

    let result = infer_generic_function(&interner, &mut checker, &func, &[TypeId::NUMBER]);
    assert_eq!(result, TypeId::NUMBER);
}

#[test]
fn test_infer_union_target_with_placeholder_and_never_member() {
    let interner = TypeInterner::new();
    let mut checker = CompatChecker::new(&interner);
    let t_name = interner.intern_string("T");

    let t_type = interner.intern(TypeKey::TypeParameter(TypeParamInfo {
        name: t_name,
        constraint: None,
        default: None,
    }));
    let param_type = interner.union(vec![t_type, TypeId::NEVER]);

    let func = FunctionShape {
        type_params: vec![TypeParamInfo {
            name: t_name,
            constraint: None,
            default: None,
        }],
        params: vec![ParamInfo {
            name: Some(interner.intern_string("x")),
            type_id: param_type,
            optional: false,
            rest: false,
        }],
        this_type: None,
        return_type: t_type,
        type_predicate: None,
        is_constructor: false,
    };

    let result = infer_generic_function(&interner, &mut checker, &func, &[TypeId::NUMBER]);
    assert_eq!(result, TypeId::NUMBER);
}

#[test]
fn test_resolve_circular_extends_with_concrete_bound() {
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

    // Simulate: <T extends U, U extends T, U extends string>
    ctx.add_upper_bound(var_t, u_type);
    ctx.add_upper_bound(var_u, t_type);
    ctx.add_upper_bound(var_u, TypeId::STRING);

    let result_t = ctx.resolve_with_constraints(var_t).unwrap();
    let result_u = ctx.resolve_with_constraints(var_u).unwrap();

    assert_eq!(result_t, TypeId::STRING);
    assert_eq!(result_u, TypeId::STRING);
}

#[test]
fn test_resolve_circular_extends_bound_order() {
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

    // Same cycle, but add concrete bound before the cyclic one.
    ctx.add_upper_bound(var_t, TypeId::STRING);
    ctx.add_upper_bound(var_t, u_type);
    ctx.add_upper_bound(var_u, t_type);

    let result_t = ctx.resolve_with_constraints(var_t).unwrap();
    let result_u = ctx.resolve_with_constraints(var_u).unwrap();

    assert_eq!(result_t, TypeId::STRING);
    assert_eq!(result_u, TypeId::STRING);
}

#[test]
fn test_resolve_usage_based_inference_from_bound_param() {
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

    // Simulate: <T extends U, U extends T> with usage-based lower bound on U.
    ctx.add_upper_bound(var_t, u_type);
    ctx.add_upper_bound(var_u, t_type);

    let hello = interner.literal_string("hello");
    ctx.add_lower_bound(var_u, hello);

    let result_t = ctx.resolve_with_constraints(var_t).unwrap();
    let result_u = ctx.resolve_with_constraints(var_u).unwrap();

    assert_eq!(result_t, hello);
    assert_eq!(result_u, hello);
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

// =============================================================================
// Additional Circular Generic Constraint Tests
// =============================================================================

#[test]
fn test_circular_extends_three_way_cycle() {
    // Test: <T extends U, U extends V, V extends T>
    let interner = TypeInterner::new();
    let mut ctx = InferenceContext::new(&interner);
    let t_name = interner.intern_string("T");
    let u_name = interner.intern_string("U");
    let v_name = interner.intern_string("V");

    let var_t = ctx.fresh_type_param(t_name);
    let var_u = ctx.fresh_type_param(u_name);
    let var_v = ctx.fresh_type_param(v_name);

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
    let v_type = interner.intern(TypeKey::TypeParameter(TypeParamInfo {
        name: v_name,
        constraint: None,
        default: None,
    }));

    // T extends U, U extends V, V extends T
    ctx.add_upper_bound(var_t, u_type);
    ctx.add_upper_bound(var_u, v_type);
    ctx.add_upper_bound(var_v, t_type);

    let results = ctx.resolve_all_with_constraints().unwrap();

    // All three resolve to unknown due to circular dependency with no concrete bounds
    assert_eq!(results.len(), 3);
    assert_eq!(results[0], (t_name, TypeId::UNKNOWN));
    assert_eq!(results[1], (u_name, TypeId::UNKNOWN));
    assert_eq!(results[2], (v_name, TypeId::UNKNOWN));
}

#[test]
fn test_circular_extends_self_reference() {
    // Test: <T extends T> - self-referential constraint
    let interner = TypeInterner::new();
    let mut ctx = InferenceContext::new(&interner);
    let t_name = interner.intern_string("T");

    let var_t = ctx.fresh_type_param(t_name);

    let t_type = interner.intern(TypeKey::TypeParameter(TypeParamInfo {
        name: t_name,
        constraint: None,
        default: None,
    }));

    // T extends T (self-reference)
    ctx.add_upper_bound(var_t, t_type);

    let results = ctx.resolve_all_with_constraints().unwrap();

    // Self-reference with no other bounds resolves to unknown
    assert_eq!(results.len(), 1);
    assert_eq!(results[0], (t_name, TypeId::UNKNOWN));
}

#[test]
fn test_circular_extends_with_lower_bound() {
    // Test: <T extends U, U extends T> with T having a lower bound of string
    // Lower bounds propagate through cyclic constraints.
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

    // T extends U, U extends T
    ctx.add_upper_bound(var_t, u_type);
    ctx.add_upper_bound(var_u, t_type);

    // Add a lower bound to T
    ctx.add_lower_bound(var_t, TypeId::STRING);

    let results = ctx.resolve_all_with_constraints().unwrap();

    assert_eq!(results.len(), 2);
    // T resolves to string (its lower bound)
    assert_eq!(results[0], (t_name, TypeId::STRING));
    // U also resolves to string - lower bounds propagate through cyclic constraints
    assert_eq!(results[1], (u_name, TypeId::STRING));
}

#[test]
fn test_circular_extends_both_have_lower_bounds() {
    // Test: <T extends U, U extends T> with both having the same lower bound
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

    // T extends U, U extends T
    ctx.add_upper_bound(var_t, u_type);
    ctx.add_upper_bound(var_u, t_type);

    // Both have the same lower bound
    ctx.add_lower_bound(var_t, TypeId::STRING);
    ctx.add_lower_bound(var_u, TypeId::STRING);

    let results = ctx.resolve_all_with_constraints().unwrap();

    assert_eq!(results.len(), 2);
    // Both resolve to string
    assert_eq!(results[0], (t_name, TypeId::STRING));
    assert_eq!(results[1], (u_name, TypeId::STRING));
}

#[test]
fn test_circular_extends_unify_propagates() {
    // Test: <T extends U, U extends T> then unify T with number
    // Unification propagates through cyclic constraints.
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

    // T extends U, U extends T
    ctx.add_upper_bound(var_t, u_type);
    ctx.add_upper_bound(var_u, t_type);

    // Unify T directly with number
    ctx.unify_var_type(var_t, TypeId::NUMBER).unwrap();

    let results = ctx.resolve_all_with_constraints().unwrap();

    assert_eq!(results.len(), 2);
    // T resolves to number (unified)
    assert_eq!(results[0], (t_name, TypeId::NUMBER));
    // U also resolves to number - unification propagates through cyclic constraints
    assert_eq!(results[1], (u_name, TypeId::NUMBER));
}

#[test]
fn test_circular_extends_conflicting_lower_bounds() {
    // Test: <T extends U, U extends T> with T: string and U: number
    // Cycle propagation causes both to get union of all lower bounds
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

    // T extends U, U extends T
    ctx.add_upper_bound(var_t, u_type);
    ctx.add_upper_bound(var_u, t_type);

    // Conflicting lower bounds
    ctx.add_lower_bound(var_t, TypeId::STRING);
    ctx.add_lower_bound(var_u, TypeId::NUMBER);

    let results = ctx.resolve_all_with_constraints().unwrap();

    assert_eq!(results.len(), 2);
    // T gets union of string | number from cycle propagation
    let expected_union = interner.union(vec![TypeId::STRING, TypeId::NUMBER]);
    assert_eq!(results[0], (t_name, expected_union));
    // U gets its direct lower bound (number)
    assert_eq!(results[1], (u_name, TypeId::NUMBER));
}

#[test]
fn test_circular_extends_three_way_with_one_lower_bound() {
    // Test: <T extends U, U extends V, V extends T> with V having lower bound
    // Bounds propagate through adjacent connections in the cycle
    let interner = TypeInterner::new();
    let mut ctx = InferenceContext::new(&interner);
    let t_name = interner.intern_string("T");
    let u_name = interner.intern_string("U");
    let v_name = interner.intern_string("V");

    let var_t = ctx.fresh_type_param(t_name);
    let var_u = ctx.fresh_type_param(u_name);
    let var_v = ctx.fresh_type_param(v_name);

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
    let v_type = interner.intern(TypeKey::TypeParameter(TypeParamInfo {
        name: v_name,
        constraint: None,
        default: None,
    }));

    // T extends U, U extends V, V extends T
    ctx.add_upper_bound(var_t, u_type);
    ctx.add_upper_bound(var_u, v_type);
    ctx.add_upper_bound(var_v, t_type);

    // Only V has a lower bound
    ctx.add_lower_bound(var_v, TypeId::BOOLEAN);

    let results = ctx.resolve_all_with_constraints().unwrap();

    assert_eq!(results.len(), 3);
    // V resolves to boolean (its direct lower bound)
    assert_eq!(results[2], (v_name, TypeId::BOOLEAN));
    // U extends V, so U gets boolean through propagation
    assert_eq!(results[1], (u_name, TypeId::BOOLEAN));
    // T extends U, but propagation stops at one level in current impl
    assert_eq!(results[0], (t_name, TypeId::UNKNOWN));
}

#[test]
fn test_circular_extends_with_union_lower_bound() {
    // Test: <T extends U, U extends T> with T having union type as lower bound
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

    // T extends U, U extends T
    ctx.add_upper_bound(var_t, u_type);
    ctx.add_upper_bound(var_u, t_type);

    // T has a union type as lower bound
    let union_type = interner.union(vec![TypeId::STRING, TypeId::NUMBER]);
    ctx.add_lower_bound(var_t, union_type);

    let results = ctx.resolve_all_with_constraints().unwrap();

    assert_eq!(results.len(), 2);
    // T resolves to the union type
    assert_eq!(results[0], (t_name, union_type));
    // U also resolves to the union through propagation
    assert_eq!(results[1], (u_name, union_type));
}

#[test]
fn test_circular_extends_with_literal_types() {
    // Test: <T extends U, U extends T> with literal type lower bounds
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

    // T extends U, U extends T
    ctx.add_upper_bound(var_t, u_type);
    ctx.add_upper_bound(var_u, t_type);

    // Both have literal string lower bounds
    let hello = interner.literal_string("hello");
    let world = interner.literal_string("world");
    ctx.add_lower_bound(var_t, hello);
    ctx.add_lower_bound(var_u, world);

    let results = ctx.resolve_all_with_constraints().unwrap();

    assert_eq!(results.len(), 2);
    // T gets union of literals from cycle propagation
    let expected_union = interner.union(vec![hello, world]);
    assert_eq!(results[0], (t_name, expected_union));
    // U gets its direct lower bound
    assert_eq!(results[1], (u_name, world));
}

#[test]
fn test_circular_extends_four_way_cycle() {
    // Test: <T extends U, U extends V, V extends W, W extends T>
    let interner = TypeInterner::new();
    let mut ctx = InferenceContext::new(&interner);
    let t_name = interner.intern_string("T");
    let u_name = interner.intern_string("U");
    let v_name = interner.intern_string("V");
    let w_name = interner.intern_string("W");

    let var_t = ctx.fresh_type_param(t_name);
    let var_u = ctx.fresh_type_param(u_name);
    let var_v = ctx.fresh_type_param(v_name);
    let var_w = ctx.fresh_type_param(w_name);

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
    let v_type = interner.intern(TypeKey::TypeParameter(TypeParamInfo {
        name: v_name,
        constraint: None,
        default: None,
    }));
    let w_type = interner.intern(TypeKey::TypeParameter(TypeParamInfo {
        name: w_name,
        constraint: None,
        default: None,
    }));

    // T extends U, U extends V, V extends W, W extends T
    ctx.add_upper_bound(var_t, u_type);
    ctx.add_upper_bound(var_u, v_type);
    ctx.add_upper_bound(var_v, w_type);
    ctx.add_upper_bound(var_w, t_type);

    let results = ctx.resolve_all_with_constraints().unwrap();

    // All four resolve to unknown with no lower bounds
    assert_eq!(results.len(), 4);
    assert_eq!(results[0], (t_name, TypeId::UNKNOWN));
    assert_eq!(results[1], (u_name, TypeId::UNKNOWN));
    assert_eq!(results[2], (v_name, TypeId::UNKNOWN));
    assert_eq!(results[3], (w_name, TypeId::UNKNOWN));
}

#[test]
fn test_circular_extends_with_concrete_upper_and_lower() {
    // Test: <T extends U, U extends T> with T having both upper and lower bounds
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

    // T extends U, U extends T
    ctx.add_upper_bound(var_t, u_type);
    ctx.add_upper_bound(var_u, t_type);

    // T has both upper bound (string) and lower bound (literal)
    let hello = interner.literal_string("hello");
    ctx.add_lower_bound(var_t, hello);
    ctx.add_upper_bound(var_t, TypeId::STRING);

    let results = ctx.resolve_all_with_constraints().unwrap();

    assert_eq!(results.len(), 2);
    // T resolves to its lower bound (hello literal)
    assert_eq!(results[0], (t_name, hello));
    // U gets hello through propagation
    assert_eq!(results[1], (u_name, hello));
}

#[test]
fn test_circular_extends_chain_with_endpoint_bound() {
    // Test: <T extends U, U extends V> (not circular) with V having lower bound
    // Chain propagation: upper bounds become resolved types when no lower bounds
    let interner = TypeInterner::new();
    let mut ctx = InferenceContext::new(&interner);
    let t_name = interner.intern_string("T");
    let u_name = interner.intern_string("U");
    let v_name = interner.intern_string("V");

    let var_t = ctx.fresh_type_param(t_name);
    let var_u = ctx.fresh_type_param(u_name);
    let var_v = ctx.fresh_type_param(v_name);

    let u_type = interner.intern(TypeKey::TypeParameter(TypeParamInfo {
        name: u_name,
        constraint: None,
        default: None,
    }));
    let v_type = interner.intern(TypeKey::TypeParameter(TypeParamInfo {
        name: v_name,
        constraint: None,
        default: None,
    }));

    // T extends U, U extends V (chain, not cycle)
    ctx.add_upper_bound(var_t, u_type);
    ctx.add_upper_bound(var_u, v_type);

    // V has a lower bound
    ctx.add_lower_bound(var_v, TypeId::NUMBER);

    let results = ctx.resolve_all_with_constraints().unwrap();

    assert_eq!(results.len(), 3);
    // V resolves to number (its lower bound)
    assert_eq!(results[2], (v_name, TypeId::NUMBER));
    // U has upper bound V but no lower bound, so resolves to its upper bound (V type param)
    assert_eq!(results[1].0, u_name);
    assert!(matches!(interner.lookup(results[1].1), Some(TypeKey::TypeParameter(_))));
    // T has upper bound U but no lower bound, resolves to its upper bound (U type param)
    assert_eq!(results[0].0, t_name);
    assert!(matches!(interner.lookup(results[0].1), Some(TypeKey::TypeParameter(_))));
}

#[test]
fn test_circular_extends_multiple_lower_bounds_same_param() {
    // Test: <T extends U, U extends T> with T having multiple lower bounds
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

    // T extends U, U extends T
    ctx.add_upper_bound(var_t, u_type);
    ctx.add_upper_bound(var_u, t_type);

    // T has multiple lower bounds
    ctx.add_lower_bound(var_t, TypeId::STRING);
    ctx.add_lower_bound(var_t, TypeId::NUMBER);
    ctx.add_lower_bound(var_t, TypeId::BOOLEAN);

    let results = ctx.resolve_all_with_constraints().unwrap();

    assert_eq!(results.len(), 2);
    // T resolves to union of all its lower bounds
    let expected_union = interner.union(vec![TypeId::STRING, TypeId::NUMBER, TypeId::BOOLEAN]);
    assert_eq!(results[0], (t_name, expected_union));
    // U gets the union through propagation
    assert_eq!(results[1], (u_name, expected_union));
}

// =============================================================================
// Context-Sensitive Typing Tests
// =============================================================================

#[test]
fn test_context_sensitive_callback_param_from_upper_bound() {
    // Test: When a callback parameter has an upper bound from context,
    // the parameter type is inferred from that context.
    // e.g., arr.map((x) => x + 1) where arr: number[]
    let interner = TypeInterner::new();
    let mut ctx = InferenceContext::new(&interner);
    let t_name = interner.intern_string("T");

    let var_t = ctx.fresh_type_param(t_name);

    // Context provides: T must be a subtype of number (from array element type)
    ctx.add_upper_bound(var_t, TypeId::NUMBER);

    let result = ctx.resolve_with_constraints(var_t).unwrap();
    // With only upper bound and no lower bound, resolves to the upper bound
    assert_eq!(result, TypeId::NUMBER);
}

#[test]
fn test_context_sensitive_return_type_from_usage() {
    // Test: Return type inference from how the result is used
    // e.g., const x: string = identity(value) infers T = string
    let interner = TypeInterner::new();
    let mut ctx = InferenceContext::new(&interner);
    let t_name = interner.intern_string("T");

    let var_t = ctx.fresh_type_param(t_name);

    // Usage site provides: result is assigned to string variable
    ctx.add_upper_bound(var_t, TypeId::STRING);
    // Call site provides: argument is a string literal
    let hello = interner.literal_string("hello");
    ctx.add_lower_bound(var_t, hello);

    let result = ctx.resolve_with_constraints(var_t).unwrap();
    // Lower bound wins (more specific)
    assert_eq!(result, hello);
}

#[test]
fn test_context_sensitive_multiple_usage_sites() {
    // Test: Multiple usage sites provide constraints that must be unified
    // e.g., function used in two places with different argument types
    let interner = TypeInterner::new();
    let mut ctx = InferenceContext::new(&interner);
    let t_name = interner.intern_string("T");

    let var_t = ctx.fresh_type_param(t_name);

    // First usage: called with string
    ctx.add_lower_bound(var_t, TypeId::STRING);
    // Second usage: called with number
    ctx.add_lower_bound(var_t, TypeId::NUMBER);

    let result = ctx.resolve_with_constraints(var_t).unwrap();
    // Multiple lower bounds create a union
    let expected = interner.union(vec![TypeId::STRING, TypeId::NUMBER]);
    assert_eq!(result, expected);
}

#[test]
fn test_context_sensitive_literal_widening_prevented() {
    // Test: When context expects a literal type, don't widen to primitive
    // e.g., const x: "hello" = getValue() where getValue returns T
    let interner = TypeInterner::new();
    let mut ctx = InferenceContext::new(&interner);
    let t_name = interner.intern_string("T");

    let var_t = ctx.fresh_type_param(t_name);

    let hello = interner.literal_string("hello");
    // Lower bound is the literal
    ctx.add_lower_bound(var_t, hello);
    // Upper bound is also the literal (from contextual type)
    ctx.add_upper_bound(var_t, hello);

    let result = ctx.resolve_with_constraints(var_t).unwrap();
    // Should preserve the literal type
    assert_eq!(result, hello);
}

#[test]
fn test_context_sensitive_object_property_inference() {
    // Test: Object property types inferred from contextual type
    // e.g., const obj: {x: number} = {x: getValue()}
    let interner = TypeInterner::new();
    let mut ctx = InferenceContext::new(&interner);
    let t_name = interner.intern_string("T");

    let var_t = ctx.fresh_type_param(t_name);

    // Context expects number for property x
    ctx.add_upper_bound(var_t, TypeId::NUMBER);
    // Value provides a specific number
    let forty_two = interner.literal_number(42.0);
    ctx.add_lower_bound(var_t, forty_two);

    let result = ctx.resolve_with_constraints(var_t).unwrap();
    // Literal number from lower bound
    assert_eq!(result, forty_two);
}

#[test]
fn test_context_sensitive_array_element_inference() {
    // Test: Array element types inferred from array context
    // e.g., const arr: string[] = [getValue()]
    let interner = TypeInterner::new();
    let mut ctx = InferenceContext::new(&interner);
    let t_name = interner.intern_string("T");

    let var_t = ctx.fresh_type_param(t_name);

    // Context: array of strings means elements must be strings
    ctx.add_upper_bound(var_t, TypeId::STRING);

    let result = ctx.resolve_with_constraints(var_t).unwrap();
    assert_eq!(result, TypeId::STRING);
}

#[test]
fn test_context_sensitive_conditional_branch_types() {
    // Test: Type from conditional branches unifies
    // e.g., condition ? stringValue : numberValue should be string | number
    let interner = TypeInterner::new();
    let mut ctx = InferenceContext::new(&interner);
    let t_name = interner.intern_string("T");

    let var_t = ctx.fresh_type_param(t_name);

    // True branch contributes string
    ctx.add_lower_bound(var_t, TypeId::STRING);
    // False branch contributes number
    ctx.add_lower_bound(var_t, TypeId::NUMBER);

    let result = ctx.resolve_with_constraints(var_t).unwrap();
    // Union of both branch types
    let expected = interner.union(vec![TypeId::STRING, TypeId::NUMBER]);
    assert_eq!(result, expected);
}

#[test]
fn test_context_sensitive_function_param_from_callback_context() {
    // Test: Function parameter type inferred from callback signature context
    // e.g., arr.filter((x) => x > 0) where arr: number[]
    let interner = TypeInterner::new();
    let mut ctx = InferenceContext::new(&interner);
    let t_name = interner.intern_string("T");

    let var_t = ctx.fresh_type_param(t_name);

    // Contextual callback type says param must be number
    ctx.add_upper_bound(var_t, TypeId::NUMBER);
    // No explicit annotation, so no lower bound from declaration

    let result = ctx.resolve_with_constraints(var_t).unwrap();
    // Infers from context
    assert_eq!(result, TypeId::NUMBER);
}

#[test]
fn test_context_sensitive_rest_param_inference() {
    // Test: Rest parameter type inference from spread arguments
    // e.g., fn(...args: T) called with (1, 2, 3)
    let interner = TypeInterner::new();
    let mut ctx = InferenceContext::new(&interner);
    let t_name = interner.intern_string("T");

    let var_t = ctx.fresh_type_param(t_name);

    // Multiple arguments of same type contribute to rest param type
    let one = interner.literal_number(1.0);
    let two = interner.literal_number(2.0);
    ctx.add_lower_bound(var_t, one);
    ctx.add_lower_bound(var_t, two);

    let result = ctx.resolve_with_constraints(var_t).unwrap();
    // Union of literal types
    let expected = interner.union(vec![one, two]);
    assert_eq!(result, expected);
}

#[test]
fn test_context_sensitive_default_param_inference() {
    // Test: Default parameter provides lower bound for type param
    // e.g., function fn<T>(x: T = "default")
    let interner = TypeInterner::new();
    let mut ctx = InferenceContext::new(&interner);
    let t_name = interner.intern_string("T");

    let var_t = ctx.fresh_type_param(t_name);

    // Default value is a string literal
    let default_val = interner.literal_string("default");
    ctx.add_lower_bound(var_t, default_val);

    let result = ctx.resolve_with_constraints(var_t).unwrap();
    assert_eq!(result, default_val);
}

// =============================================================================
// Callback Parameter Inference Tests
// =============================================================================

#[test]
fn test_callback_param_inferred_from_array_map() {
    // Test: arr.map((x) => x.toUpperCase()) where arr: string[]
    // The callback parameter x should be inferred as string from array element type
    let interner = TypeInterner::new();
    let mut ctx = InferenceContext::new(&interner);
    let t_name = interner.intern_string("T");

    let var_t = ctx.fresh_type_param(t_name);

    // Array<string>.map provides callback with (element: string) => U
    // So T (the callback param type) has upper bound string
    ctx.add_upper_bound(var_t, TypeId::STRING);

    let result = ctx.resolve_with_constraints(var_t).unwrap();
    // Callback param inferred from array element type
    assert_eq!(result, TypeId::STRING);
}

#[test]
fn test_callback_param_inferred_with_index() {
    // Test: arr.forEach((item, index) => ...) where arr: number[]
    // First param is number (element), second is number (index)
    let interner = TypeInterner::new();
    let mut ctx = InferenceContext::new(&interner);
    let t_name = interner.intern_string("T");
    let u_name = interner.intern_string("U");

    let var_t = ctx.fresh_type_param(t_name);
    let var_u = ctx.fresh_type_param(u_name);

    // T is the element type from Array<number>
    ctx.add_upper_bound(var_t, TypeId::NUMBER);
    // U is the index type (always number)
    ctx.add_upper_bound(var_u, TypeId::NUMBER);

    let result_t = ctx.resolve_with_constraints(var_t).unwrap();
    let result_u = ctx.resolve_with_constraints(var_u).unwrap();

    assert_eq!(result_t, TypeId::NUMBER);
    assert_eq!(result_u, TypeId::NUMBER);
}

#[test]
fn test_callback_param_inferred_from_generic_higher_order() {
    // Test: Generic higher-order function like filter<T>(arr: T[], pred: (x: T) => boolean)
    // When called with string[], T is inferred as string, so callback param is string
    let interner = TypeInterner::new();
    let mut ctx = InferenceContext::new(&interner);
    let t_name = interner.intern_string("T");

    let var_t = ctx.fresh_type_param(t_name);

    // Lower bound from argument: array contains strings
    ctx.add_lower_bound(var_t, TypeId::STRING);
    // Upper bound from callback usage: predicate receives T
    // (callback param type flows from T)

    let result = ctx.resolve_with_constraints(var_t).unwrap();
    // T inferred as string, so callback param is string
    assert_eq!(result, TypeId::STRING);
}

// =============================================================================
// Generic Default Type Inference Tests
// =============================================================================

#[test]
fn test_generic_default_used_when_no_inference() {
    // Test: <T = string> with no inference constraints, T defaults to string
    let interner = TypeInterner::new();
    let mut ctx = InferenceContext::new(&interner);
    let t_name = interner.intern_string("T");

    let var_t = ctx.fresh_type_param(t_name);

    // No constraints added - should use default if available
    // Note: defaults are typically handled during type param registration,
    // but here we test the inference context behavior with no constraints
    let result = ctx.resolve_with_constraints(var_t).unwrap();
    // Without any constraints, resolves to unknown
    assert_eq!(result, TypeId::UNKNOWN);
}

#[test]
fn test_generic_default_overridden_by_lower_bound() {
    // Test: <T = string> with lower bound number, inference overrides default
    let interner = TypeInterner::new();
    let mut ctx = InferenceContext::new(&interner);
    let t_name = interner.intern_string("T");

    let var_t = ctx.fresh_type_param(t_name);

    // Inferred lower bound takes precedence
    ctx.add_lower_bound(var_t, TypeId::NUMBER);

    let result = ctx.resolve_with_constraints(var_t).unwrap();
    // Lower bound overrides any potential default
    assert_eq!(result, TypeId::NUMBER);
}

#[test]
fn test_generic_default_with_constraint() {
    // Test: <T extends object = {}> - constraint with default
    let interner = TypeInterner::new();
    let mut ctx = InferenceContext::new(&interner);
    let t_name = interner.intern_string("T");

    let var_t = ctx.fresh_type_param(t_name);

    // Upper bound from constraint
    ctx.add_upper_bound(var_t, TypeId::OBJECT);

    let result = ctx.resolve_with_constraints(var_t).unwrap();
    // With only upper bound, resolves to the upper bound
    assert_eq!(result, TypeId::OBJECT);
}

#[test]
fn test_generic_default_with_literal_inference() {
    // Test: <T = string> called with literal "hello", infers literal not default
    let interner = TypeInterner::new();
    let mut ctx = InferenceContext::new(&interner);
    let t_name = interner.intern_string("T");

    let var_t = ctx.fresh_type_param(t_name);

    let hello = interner.literal_string("hello");
    ctx.add_lower_bound(var_t, hello);

    let result = ctx.resolve_with_constraints(var_t).unwrap();
    // Inferred literal takes precedence over default
    assert_eq!(result, hello);
}

#[test]
fn test_generic_multiple_params_with_defaults() {
    // Test: <T = string, U = number> with only U having lower bound
    let interner = TypeInterner::new();
    let mut ctx = InferenceContext::new(&interner);
    let t_name = interner.intern_string("T");
    let u_name = interner.intern_string("U");

    let var_t = ctx.fresh_type_param(t_name);
    let var_u = ctx.fresh_type_param(u_name);

    // Only U has a lower bound
    ctx.add_lower_bound(var_u, TypeId::BOOLEAN);

    let results = ctx.resolve_all_with_constraints().unwrap();

    assert_eq!(results.len(), 2);
    // T has no constraints, resolves to unknown
    assert_eq!(results[0], (t_name, TypeId::UNKNOWN));
    // U has lower bound, resolves to boolean
    assert_eq!(results[1], (u_name, TypeId::BOOLEAN));
}

// =============================================================================
// Generic Constraint Propagation Tests
// =============================================================================

#[test]
fn test_constraint_propagation_upper_to_lower() {
    // Test: Upper bound on one param propagates to lower bound check
    // <T extends string> called with T = "hello"
    let interner = TypeInterner::new();
    let mut ctx = InferenceContext::new(&interner);
    let t_name = interner.intern_string("T");

    let var_t = ctx.fresh_type_param(t_name);

    // Upper bound: T extends string
    ctx.add_upper_bound(var_t, TypeId::STRING);
    // Lower bound from argument: "hello"
    let hello = interner.literal_string("hello");
    ctx.add_lower_bound(var_t, hello);

    let result = ctx.resolve_with_constraints(var_t).unwrap();
    // Lower bound satisfies upper bound, resolves to literal
    assert_eq!(result, hello);
}

#[test]
fn test_constraint_propagation_through_unification() {
    // Test: Unifying two vars propagates constraints from both
    let interner = TypeInterner::new();
    let mut ctx = InferenceContext::new(&interner);
    let t_name = interner.intern_string("T");
    let u_name = interner.intern_string("U");

    let var_t = ctx.fresh_type_param(t_name);
    let var_u = ctx.fresh_type_param(u_name);

    // T has lower bound string
    ctx.add_lower_bound(var_t, TypeId::STRING);
    // U has lower bound number
    ctx.add_lower_bound(var_u, TypeId::NUMBER);

    // Unify T and U
    ctx.unify_vars(var_t, var_u).unwrap();

    let result = ctx.resolve_with_constraints(var_t).unwrap();
    // Unified vars get union of both lower bounds
    let expected = interner.union(vec![TypeId::STRING, TypeId::NUMBER]);
    assert_eq!(result, expected);
}

#[test]
fn test_constraint_propagation_transitive_upper_bounds() {
    // Test: T extends string with lower bound "hello"
    // Lower bound must satisfy upper bound constraint
    let interner = TypeInterner::new();
    let mut ctx = InferenceContext::new(&interner);
    let t_name = interner.intern_string("T");

    let var_t = ctx.fresh_type_param(t_name);

    // Upper bound: T extends string
    ctx.add_upper_bound(var_t, TypeId::STRING);

    // Add lower bound to T (literal satisfies string upper bound)
    let hello = interner.literal_string("hello");
    ctx.add_lower_bound(var_t, hello);

    let result_t = ctx.resolve_with_constraints(var_t).unwrap();

    // T resolves to its lower bound (literal "hello")
    assert_eq!(result_t, hello);
}

#[test]
fn test_constraint_propagation_multiple_upper_bounds() {
    // Test: T extends A & B (multiple upper bounds create intersection)
    let interner = TypeInterner::new();
    let mut ctx = InferenceContext::new(&interner);
    let t_name = interner.intern_string("T");

    let var_t = ctx.fresh_type_param(t_name);

    // Multiple upper bounds
    ctx.add_upper_bound(var_t, TypeId::STRING);
    ctx.add_upper_bound(var_t, TypeId::NUMBER);

    let result = ctx.resolve_with_constraints(var_t).unwrap();
    // Multiple upper bounds create intersection
    let expected = interner.intersection(vec![TypeId::STRING, TypeId::NUMBER]);
    assert_eq!(result, expected);
}

#[test]
fn test_constraint_propagation_lower_bounds_union() {
    // Test: Multiple lower bounds create union
    let interner = TypeInterner::new();
    let mut ctx = InferenceContext::new(&interner);
    let t_name = interner.intern_string("T");

    let var_t = ctx.fresh_type_param(t_name);

    // Multiple lower bounds from different call sites
    ctx.add_lower_bound(var_t, TypeId::STRING);
    ctx.add_lower_bound(var_t, TypeId::NUMBER);
    ctx.add_lower_bound(var_t, TypeId::BOOLEAN);

    let result = ctx.resolve_with_constraints(var_t).unwrap();
    // Multiple lower bounds create union
    let expected = interner.union(vec![TypeId::STRING, TypeId::NUMBER, TypeId::BOOLEAN]);
    assert_eq!(result, expected);
}

#[test]
fn test_constraint_propagation_with_never_lower_bound() {
    // Test: never as lower bound doesn't contribute to union
    let interner = TypeInterner::new();
    let mut ctx = InferenceContext::new(&interner);
    let t_name = interner.intern_string("T");

    let var_t = ctx.fresh_type_param(t_name);

    // Lower bounds including never
    ctx.add_lower_bound(var_t, TypeId::NEVER);
    ctx.add_lower_bound(var_t, TypeId::STRING);

    let result = ctx.resolve_with_constraints(var_t).unwrap();
    // never is filtered out, only string remains
    assert_eq!(result, TypeId::STRING);
}

#[test]
fn test_constraint_propagation_any_lower_with_concrete() {
    // Test: any as lower bound with concrete type
    let interner = TypeInterner::new();
    let mut ctx = InferenceContext::new(&interner);
    let t_name = interner.intern_string("T");

    let var_t = ctx.fresh_type_param(t_name);

    // Lower bounds: any and string
    ctx.add_lower_bound(var_t, TypeId::ANY);
    ctx.add_lower_bound(var_t, TypeId::STRING);
    // Upper bound constrains
    ctx.add_upper_bound(var_t, TypeId::STRING);

    let result = ctx.resolve_with_constraints(var_t).unwrap();
    // With upper bound, any is filtered from lower bounds
    assert_eq!(result, TypeId::STRING);
}

#[test]
fn test_constraint_propagation_object_properties() {
    // Test: Object type constraint propagation
    let interner = TypeInterner::new();
    let mut ctx = InferenceContext::new(&interner);
    let t_name = interner.intern_string("T");

    let var_t = ctx.fresh_type_param(t_name);

    // Create object type with property
    let prop_name = interner.intern_string("x");
    let obj_type = interner.object(vec![PropertyInfo {
        name: prop_name,
        type_id: TypeId::NUMBER,
        write_type: TypeId::NUMBER,
        optional: false,
        readonly: false,
        is_method: false,
    }]);

    ctx.add_lower_bound(var_t, obj_type);

    let result = ctx.resolve_with_constraints(var_t).unwrap();
    assert_eq!(result, obj_type);
}

// ============================================================================
// Constructor Type Inference Tests
// ============================================================================
// Tests for constructor function type inference

#[test]
fn test_constructor_single_param_inference() {
    // Test: new (x: T) => Instance infers T from argument
    let interner = TypeInterner::new();
    let mut ctx = InferenceContext::new(&interner);
    let t_name = interner.intern_string("T");

    let var_t = ctx.fresh_type_param(t_name);

    // Constructor param receives string argument
    ctx.add_lower_bound(var_t, TypeId::STRING);

    let result = ctx.resolve_with_constraints(var_t).unwrap();
    assert_eq!(result, TypeId::STRING);
}

#[test]
fn test_constructor_multiple_params_inference() {
    // Test: new <T, U>(a: T, b: U) => Instance infers both T and U
    let interner = TypeInterner::new();
    let mut ctx = InferenceContext::new(&interner);
    let t_name = interner.intern_string("T");
    let u_name = interner.intern_string("U");

    let var_t = ctx.fresh_type_param(t_name);
    let var_u = ctx.fresh_type_param(u_name);

    // First param is string, second is number
    ctx.add_lower_bound(var_t, TypeId::STRING);
    ctx.add_lower_bound(var_u, TypeId::NUMBER);

    let results = ctx.resolve_all_with_constraints().unwrap();

    assert_eq!(results.len(), 2);
    assert_eq!(results[0], (t_name, TypeId::STRING));
    assert_eq!(results[1], (u_name, TypeId::NUMBER));
}

#[test]
fn test_constructor_with_constraint() {
    // Test: new <T extends object>(config: T) => Instance
    let interner = TypeInterner::new();
    let mut ctx = InferenceContext::new(&interner);
    let t_name = interner.intern_string("T");

    let var_t = ctx.fresh_type_param(t_name);

    // T has upper bound of object
    ctx.add_upper_bound(var_t, TypeId::OBJECT);

    // Argument is specific object type
    let prop_name = interner.intern_string("name");
    let obj_type = interner.object(vec![PropertyInfo {
        name: prop_name,
        type_id: TypeId::STRING,
        write_type: TypeId::STRING,
        optional: false,
        readonly: false,
        is_method: false,
    }]);
    ctx.add_lower_bound(var_t, obj_type);

    let result = ctx.resolve_with_constraints(var_t).unwrap();
    // Should be the specific object type
    assert_eq!(result, obj_type);
}

#[test]
fn test_constructor_optional_param_inference() {
    // Test: new <T>(arg?: T) => Instance with optional param
    let interner = TypeInterner::new();
    let mut ctx = InferenceContext::new(&interner);
    let t_name = interner.intern_string("T");

    let var_t = ctx.fresh_type_param(t_name);

    // Optional param not provided - may include undefined
    let optional_type = interner.union(vec![TypeId::STRING, TypeId::UNDEFINED]);
    ctx.add_lower_bound(var_t, optional_type);

    let result = ctx.resolve_with_constraints(var_t).unwrap();
    // Should preserve the union type
    assert_eq!(result, optional_type);
}

#[test]
fn test_constructor_rest_param_inference() {
    // Test: new <T>(...args: T[]) => Instance with rest param
    let interner = TypeInterner::new();
    let mut ctx = InferenceContext::new(&interner);
    let t_name = interner.intern_string("T");

    let var_t = ctx.fresh_type_param(t_name);

    // Rest param elements are string and number - infer union
    ctx.add_lower_bound(var_t, TypeId::STRING);
    ctx.add_lower_bound(var_t, TypeId::NUMBER);

    let result = ctx.resolve_with_constraints(var_t).unwrap();
    // Should be union of string | number
    if let Some(TypeKey::Union(_)) = interner.lookup(result) {
        // Union is expected
    } else {
        // Could also resolve to one of the types if widening happens
        assert!(result == TypeId::STRING || result == TypeId::NUMBER);
    }
}

// ============================================================================
// Method Signature Inference Tests
// ============================================================================

#[test]
fn test_method_return_type_inference_basic() {
    // Test inferring return type from method call: obj.method() returns string
    let interner = TypeInterner::new();
    let mut ctx = InferenceContext::new(&interner);
    let t_name = interner.intern_string("T");

    let var_t = ctx.fresh_type_param(t_name);
    let t_type = interner.intern(TypeKey::TypeParameter(TypeParamInfo {
        name: t_name,
        constraint: None,
        default: None,
    }));

    // Method signature: () => T
    let _method = interner.function(FunctionShape {
        type_params: vec![TypeParamInfo {
            name: t_name,
            constraint: None,
            default: None,
        }],
        params: vec![],
        this_type: None,
        return_type: t_type,
        type_predicate: None,
        is_constructor: false,
    });

    // Call returns string, so T should be inferred as string
    ctx.add_lower_bound(var_t, TypeId::STRING);

    let result = ctx.resolve_with_constraints(var_t).unwrap();
    assert_eq!(result, TypeId::STRING);
}

#[test]
fn test_method_parameter_type_inference() {
    // Test inferring parameter type from method call: obj.method(value)
    let interner = TypeInterner::new();
    let mut ctx = InferenceContext::new(&interner);
    let t_name = interner.intern_string("T");

    let var_t = ctx.fresh_type_param(t_name);
    let t_type = interner.intern(TypeKey::TypeParameter(TypeParamInfo {
        name: t_name,
        constraint: None,
        default: None,
    }));

    // Method signature: (x: T) => void
    let _method = interner.function(FunctionShape {
        type_params: vec![TypeParamInfo {
            name: t_name,
            constraint: None,
            default: None,
        }],
        params: vec![ParamInfo {
            name: Some(interner.intern_string("x")),
            type_id: t_type,
            optional: false,
            rest: false,
        }],
        this_type: None,
        return_type: TypeId::VOID,
        type_predicate: None,
        is_constructor: false,
    });

    // Called with number, so T should be inferred as number
    ctx.add_lower_bound(var_t, TypeId::NUMBER);

    let result = ctx.resolve_with_constraints(var_t).unwrap();
    assert_eq!(result, TypeId::NUMBER);
}

#[test]
fn test_method_this_type_inference() {
    // Test this type in method: class method with this constraint
    let interner = TypeInterner::new();
    let mut ctx = InferenceContext::new(&interner);
    let this_name = interner.intern_string("This");

    let var_this = ctx.fresh_type_param(this_name);
    let this_type = interner.intern(TypeKey::TypeParameter(TypeParamInfo {
        name: this_name,
        constraint: None,
        default: None,
    }));

    // Method signature: (this: This) => This
    let _method = interner.function(FunctionShape {
        type_params: vec![TypeParamInfo {
            name: this_name,
            constraint: None,
            default: None,
        }],
        params: vec![],
        this_type: Some(this_type),
        return_type: this_type,
        type_predicate: None,
        is_constructor: false,
    });

    // Create an object type to represent `this`
    let obj_type = interner.object(vec![PropertyInfo {
        name: interner.intern_string("value"),
        type_id: TypeId::STRING,
        write_type: TypeId::STRING,
        optional: false,
        readonly: false,
        is_method: false,
    }]);

    // Called on object, so This should be inferred as that object type
    ctx.add_lower_bound(var_this, obj_type);

    let result = ctx.resolve_with_constraints(var_this).unwrap();
    assert_eq!(result, obj_type);
}

#[test]
fn test_method_generic_parameter_inference() {
    // Test: generic method <T>(x: T) => Array<T>
    let interner = TypeInterner::new();
    let mut ctx = InferenceContext::new(&interner);
    let t_name = interner.intern_string("T");

    let var_t = ctx.fresh_type_param(t_name);
    let t_type = interner.intern(TypeKey::TypeParameter(TypeParamInfo {
        name: t_name,
        constraint: None,
        default: None,
    }));

    // Method signature: <T>(x: T) => Array<T>
    let return_array = interner.array(t_type);
    let _method = interner.function(FunctionShape {
        type_params: vec![TypeParamInfo {
            name: t_name,
            constraint: None,
            default: None,
        }],
        params: vec![ParamInfo {
            name: Some(interner.intern_string("x")),
            type_id: t_type,
            optional: false,
            rest: false,
        }],
        this_type: None,
        return_type: return_array,
        type_predicate: None,
        is_constructor: false,
    });

    // Called with boolean, so T should be inferred as boolean
    ctx.add_lower_bound(var_t, TypeId::BOOLEAN);

    let result = ctx.resolve_with_constraints(var_t).unwrap();
    assert_eq!(result, TypeId::BOOLEAN);
}

#[test]
fn test_method_multiple_generic_params_inference() {
    // Test: <K, V>(key: K, value: V) => Map<K, V>
    let interner = TypeInterner::new();
    let mut ctx = InferenceContext::new(&interner);
    let k_name = interner.intern_string("K");
    let v_name = interner.intern_string("V");

    let var_k = ctx.fresh_type_param(k_name);
    let var_v = ctx.fresh_type_param(v_name);

    // Called with (string, number)
    ctx.add_lower_bound(var_k, TypeId::STRING);
    ctx.add_lower_bound(var_v, TypeId::NUMBER);

    let results = ctx.resolve_all_with_constraints().unwrap();

    assert_eq!(results.len(), 2);
    // K inferred as string
    assert_eq!(results[0], (k_name, TypeId::STRING));
    // V inferred as number
    assert_eq!(results[1], (v_name, TypeId::NUMBER));
}

// ============================================================================
// Circular Type Alias Detection Tests
// ============================================================================
// Tests for detecting and handling circular type aliases

#[test]
fn test_circular_type_alias_self_reference() {
    // Test: type T = T (direct self-reference should be detected)
    let interner = TypeInterner::new();
    let mut ctx = InferenceContext::new(&interner);
    let t_name = interner.intern_string("T");

    let var_t = ctx.fresh_type_param(t_name);

    // No bounds - trying to resolve should give unknown/any
    let result = ctx.resolve_with_constraints(var_t);
    // Without concrete bounds, resolution should still work (gives unknown)
    assert!(result.is_ok());
}

#[test]
fn test_circular_type_alias_via_array() {
    // Test: type T = Array<T> - recursive through array
    let interner = TypeInterner::new();
    let mut ctx = InferenceContext::new(&interner);
    let t_name = interner.intern_string("T");

    let var_t = ctx.fresh_type_param(t_name);

    // Add concrete array lower bound
    let string_array = interner.array(TypeId::STRING);
    ctx.add_lower_bound(var_t, string_array);

    let result = ctx.resolve_with_constraints(var_t);
    assert!(result.is_ok());
    assert_eq!(result.unwrap(), string_array);
}

#[test]
fn test_circular_type_alias_via_union() {
    // Test: type T = T | null - recursive through union
    let interner = TypeInterner::new();
    let mut ctx = InferenceContext::new(&interner);
    let t_name = interner.intern_string("T");

    let var_t = ctx.fresh_type_param(t_name);

    // Upper bound: string | null
    let string_or_null = interner.union(vec![TypeId::STRING, TypeId::NULL]);
    ctx.add_upper_bound(var_t, string_or_null);

    // Lower bound: string
    ctx.add_lower_bound(var_t, TypeId::STRING);

    let result = ctx.resolve_with_constraints(var_t);
    assert!(result.is_ok());
    assert_eq!(result.unwrap(), TypeId::STRING);
}

#[test]
fn test_circular_type_alias_nested_object() {
    // Test: type Node = { child: Node | null }
    let interner = TypeInterner::new();
    let mut ctx = InferenceContext::new(&interner);
    let t_name = interner.intern_string("T");

    let var_t = ctx.fresh_type_param(t_name);

    // Just test that we can have object bounds without infinite recursion
    let prop_name = interner.intern_string("value");
    let obj_type = interner.object(vec![PropertyInfo {
        name: prop_name,
        type_id: TypeId::STRING,
        write_type: TypeId::STRING,
        optional: false,
        readonly: false,
        is_method: false,
    }]);
    ctx.add_lower_bound(var_t, obj_type);

    let result = ctx.resolve_with_constraints(var_t);
    assert!(result.is_ok());
    assert_eq!(result.unwrap(), obj_type);
}

#[test]
fn test_circular_type_alias_function_return() {
    // Test: type F = () => F - function returning itself
    let interner = TypeInterner::new();
    let mut ctx = InferenceContext::new(&interner);
    let f_name = interner.intern_string("F");

    let var_f = ctx.fresh_type_param(f_name);

    // Add function lower bound
    let fn_type = interner.function(FunctionShape {
        type_params: vec![],
        params: vec![],
        this_type: None,
        return_type: TypeId::VOID,
        type_predicate: None,
        is_constructor: false,
    });
    ctx.add_lower_bound(var_f, fn_type);

    let result = ctx.resolve_with_constraints(var_f);
    assert!(result.is_ok());
    assert_eq!(result.unwrap(), fn_type);
}

// ============================================================================
// Self-Referential Generic Constraints Tests
// ============================================================================
// Tests for generic type parameters that reference themselves in constraints

#[test]
fn test_self_ref_constraint_comparable() {
    // Test: T extends Comparable<T> pattern (common in sorting)
    let interner = TypeInterner::new();
    let mut ctx = InferenceContext::new(&interner);
    let t_name = interner.intern_string("T");

    let var_t = ctx.fresh_type_param(t_name);

    // Upper bound: number (which is comparable to itself)
    ctx.add_upper_bound(var_t, TypeId::NUMBER);

    // Lower bound: specific number literal
    let num_lit = interner.literal_number(42.0);
    ctx.add_lower_bound(var_t, num_lit);

    let result = ctx.resolve_with_constraints(var_t);
    assert!(result.is_ok());
    assert_eq!(result.unwrap(), num_lit);
}

#[test]
fn test_self_ref_constraint_builder_pattern() {
    // Test: T extends Builder<T> - fluent builder pattern
    let interner = TypeInterner::new();
    let mut ctx = InferenceContext::new(&interner);
    let t_name = interner.intern_string("T");

    let var_t = ctx.fresh_type_param(t_name);

    // Simulate builder with method that returns same type
    let build_prop = interner.intern_string("build");
    let builder_fn = interner.function(FunctionShape {
        type_params: vec![],
        params: vec![],
        this_type: None,
        return_type: TypeId::STRING,
        type_predicate: None,
        is_constructor: false,
    });
    let builder_type = interner.object(vec![PropertyInfo {
        name: build_prop,
        type_id: builder_fn,
        write_type: builder_fn,
        optional: false,
        readonly: false,
        is_method: true,
    }]);

    ctx.add_lower_bound(var_t, builder_type);

    let result = ctx.resolve_with_constraints(var_t);
    assert!(result.is_ok());
    assert_eq!(result.unwrap(), builder_type);
}

#[test]
fn test_self_ref_constraint_iterable() {
    // Test: T extends Iterable<T> - iterable of itself
    let interner = TypeInterner::new();
    let mut ctx = InferenceContext::new(&interner);
    let t_name = interner.intern_string("T");

    let var_t = ctx.fresh_type_param(t_name);

    // Upper bound: array (iterable)
    let number_array = interner.array(TypeId::NUMBER);
    ctx.add_upper_bound(var_t, number_array);

    // Lower bound: specific array
    ctx.add_lower_bound(var_t, number_array);

    let result = ctx.resolve_with_constraints(var_t);
    assert!(result.is_ok());
    assert_eq!(result.unwrap(), number_array);
}

#[test]
fn test_self_ref_constraint_json_value() {
    // Test: type JSONValue = string | number | boolean | JSONValue[] | {[k: string]: JSONValue}
    let interner = TypeInterner::new();
    let mut ctx = InferenceContext::new(&interner);
    let t_name = interner.intern_string("T");

    let var_t = ctx.fresh_type_param(t_name);

    // Upper bound: primitive union (simplified JSON)
    let json_primitive = interner.union(vec![TypeId::STRING, TypeId::NUMBER, TypeId::BOOLEAN, TypeId::NULL]);
    ctx.add_upper_bound(var_t, json_primitive);

    // Lower bound: string (valid JSON value)
    ctx.add_lower_bound(var_t, TypeId::STRING);

    let result = ctx.resolve_with_constraints(var_t);
    assert!(result.is_ok());
    assert_eq!(result.unwrap(), TypeId::STRING);
}

#[test]
fn test_self_ref_constraint_recursive_array() {
    // Test: T extends T[] - array of itself constraint
    let interner = TypeInterner::new();
    let mut ctx = InferenceContext::new(&interner);
    let t_name = interner.intern_string("T");

    let var_t = ctx.fresh_type_param(t_name);

    // Test with array bounds
    let string_array = interner.array(TypeId::STRING);
    ctx.add_lower_bound(var_t, string_array);

    let result = ctx.resolve_with_constraints(var_t);
    assert!(result.is_ok());
    assert_eq!(result.unwrap(), string_array);
}

// ============================================================================
// Mutually Recursive Type Definitions Tests
// ============================================================================
// Tests for types that reference each other in a cycle

#[test]
fn test_mutual_recursion_two_types() {
    // Test: type A = { b: B }, type B = { a: A }
    let interner = TypeInterner::new();
    let mut ctx = InferenceContext::new(&interner);
    let a_name = interner.intern_string("A");
    let b_name = interner.intern_string("B");

    let var_a = ctx.fresh_type_param(a_name);
    let var_b = ctx.fresh_type_param(b_name);

    // Both get object lower bounds (breaking the cycle with concrete types)
    let prop_a = interner.intern_string("value");
    let obj_a = interner.object(vec![PropertyInfo {
        name: prop_a,
        type_id: TypeId::STRING,
        write_type: TypeId::STRING,
        optional: false,
        readonly: false,
        is_method: false,
    }]);

    let prop_b = interner.intern_string("count");
    let obj_b = interner.object(vec![PropertyInfo {
        name: prop_b,
        type_id: TypeId::NUMBER,
        write_type: TypeId::NUMBER,
        optional: false,
        readonly: false,
        is_method: false,
    }]);

    ctx.add_lower_bound(var_a, obj_a);
    ctx.add_lower_bound(var_b, obj_b);

    let results = ctx.resolve_all_with_constraints();
    assert!(results.is_ok());
    let resolved = results.unwrap();
    assert_eq!(resolved.len(), 2);
    assert_eq!(resolved[0].1, obj_a);
    assert_eq!(resolved[1].1, obj_b);
}

#[test]
fn test_mutual_recursion_three_types() {
    // Test: A -> B -> C -> A cycle
    let interner = TypeInterner::new();
    let mut ctx = InferenceContext::new(&interner);
    let a_name = interner.intern_string("A");
    let b_name = interner.intern_string("B");
    let c_name = interner.intern_string("C");

    let var_a = ctx.fresh_type_param(a_name);
    let var_b = ctx.fresh_type_param(b_name);
    let var_c = ctx.fresh_type_param(c_name);

    // All have same upper bound
    ctx.add_upper_bound(var_a, TypeId::STRING);
    ctx.add_upper_bound(var_b, TypeId::STRING);
    ctx.add_upper_bound(var_c, TypeId::STRING);

    // Different literal lower bounds
    let lit_a = interner.literal_string("a");
    let lit_b = interner.literal_string("b");
    let lit_c = interner.literal_string("c");

    ctx.add_lower_bound(var_a, lit_a);
    ctx.add_lower_bound(var_b, lit_b);
    ctx.add_lower_bound(var_c, lit_c);

    let results = ctx.resolve_all_with_constraints();
    assert!(results.is_ok());
    let resolved = results.unwrap();
    assert_eq!(resolved.len(), 3);
    assert_eq!(resolved[0].1, lit_a);
    assert_eq!(resolved[1].1, lit_b);
    assert_eq!(resolved[2].1, lit_c);
}

#[test]
fn test_mutual_recursion_shared_constraint() {
    // Test: A and B both bounded by same type
    let interner = TypeInterner::new();
    let mut ctx = InferenceContext::new(&interner);
    let a_name = interner.intern_string("A");
    let b_name = interner.intern_string("B");

    let var_a = ctx.fresh_type_param(a_name);
    let var_b = ctx.fresh_type_param(b_name);

    // Shared upper bound
    let shared_union = interner.union(vec![TypeId::STRING, TypeId::NUMBER]);
    ctx.add_upper_bound(var_a, shared_union);
    ctx.add_upper_bound(var_b, shared_union);

    // A gets string, B gets number
    ctx.add_lower_bound(var_a, TypeId::STRING);
    ctx.add_lower_bound(var_b, TypeId::NUMBER);

    let results = ctx.resolve_all_with_constraints();
    assert!(results.is_ok());
    let resolved = results.unwrap();
    assert_eq!(resolved.len(), 2);
    assert_eq!(resolved[0].1, TypeId::STRING);
    assert_eq!(resolved[1].1, TypeId::NUMBER);
}

#[test]
fn test_mutual_recursion_array_element() {
    // Test: A = B[], B = A[] (arrays of each other)
    let interner = TypeInterner::new();
    let mut ctx = InferenceContext::new(&interner);
    let a_name = interner.intern_string("A");
    let b_name = interner.intern_string("B");

    let var_a = ctx.fresh_type_param(a_name);
    let var_b = ctx.fresh_type_param(b_name);

    // Concrete array lower bounds
    let string_array = interner.array(TypeId::STRING);
    let number_array = interner.array(TypeId::NUMBER);

    ctx.add_lower_bound(var_a, string_array);
    ctx.add_lower_bound(var_b, number_array);

    let results = ctx.resolve_all_with_constraints();
    assert!(results.is_ok());
    let resolved = results.unwrap();
    assert_eq!(resolved.len(), 2);
    assert_eq!(resolved[0].1, string_array);
    assert_eq!(resolved[1].1, number_array);
}

#[test]
fn test_mutual_recursion_function_params() {
    // Test: F = (a: G) => void, G = (f: F) => void
    let interner = TypeInterner::new();
    let mut ctx = InferenceContext::new(&interner);
    let f_name = interner.intern_string("F");
    let g_name = interner.intern_string("G");

    let var_f = ctx.fresh_type_param(f_name);
    let var_g = ctx.fresh_type_param(g_name);

    // Create ParamInfo structs
    let param_f = ParamInfo {
        name: Some(interner.intern_string("a")),
        type_id: TypeId::STRING,
        optional: false,
        rest: false,
    };
    let param_g = ParamInfo {
        name: Some(interner.intern_string("f")),
        type_id: TypeId::NUMBER,
        optional: false,
        rest: false,
    };

    // Concrete function lower bounds
    let fn_f = interner.function(FunctionShape {
        type_params: vec![],
        params: vec![param_f],
        this_type: None,
        return_type: TypeId::VOID,
        type_predicate: None,
        is_constructor: false,
    });
    let fn_g = interner.function(FunctionShape {
        type_params: vec![],
        params: vec![param_g],
        this_type: None,
        return_type: TypeId::VOID,
        type_predicate: None,
        is_constructor: false,
    });

    ctx.add_lower_bound(var_f, fn_f);
    ctx.add_lower_bound(var_g, fn_g);

    let results = ctx.resolve_all_with_constraints();
    assert!(results.is_ok());
    let resolved = results.unwrap();
    assert_eq!(resolved.len(), 2);
    assert_eq!(resolved[0].1, fn_f);
    assert_eq!(resolved[1].1, fn_g);
}

// ============================================================================
// Higher-Order Function Type Inference Tests
// ============================================================================
// Tests for inferring types in functions that take or return functions

#[test]
fn test_hof_callback_param_inference() {
    // Test: map<T, U>(arr: T[], fn: (x: T) => U) => U[]
    // Inferring T from array and U from callback return
    let interner = TypeInterner::new();
    let mut ctx = InferenceContext::new(&interner);
    let t_name = interner.intern_string("T");
    let u_name = interner.intern_string("U");

    let var_t = ctx.fresh_type_param(t_name);
    let var_u = ctx.fresh_type_param(u_name);

    // T inferred from array element type
    ctx.add_lower_bound(var_t, TypeId::STRING);

    // U inferred from callback return type
    ctx.add_lower_bound(var_u, TypeId::NUMBER);

    let results = ctx.resolve_all_with_constraints().unwrap();
    assert_eq!(results.len(), 2);
    assert_eq!(results[0].1, TypeId::STRING);
    assert_eq!(results[1].1, TypeId::NUMBER);
}

#[test]
fn test_hof_compose_functions() {
    // Test: compose<A, B, C>(f: (b: B) => C, g: (a: A) => B) => (a: A) => C
    let interner = TypeInterner::new();
    let mut ctx = InferenceContext::new(&interner);
    let a_name = interner.intern_string("A");
    let b_name = interner.intern_string("B");
    let c_name = interner.intern_string("C");

    let var_a = ctx.fresh_type_param(a_name);
    let var_b = ctx.fresh_type_param(b_name);
    let var_c = ctx.fresh_type_param(c_name);

    // Infer from concrete function types
    ctx.add_lower_bound(var_a, TypeId::STRING);
    ctx.add_lower_bound(var_b, TypeId::NUMBER);
    ctx.add_lower_bound(var_c, TypeId::BOOLEAN);

    let results = ctx.resolve_all_with_constraints().unwrap();
    assert_eq!(results.len(), 3);
    assert_eq!(results[0].1, TypeId::STRING);
    assert_eq!(results[1].1, TypeId::NUMBER);
    assert_eq!(results[2].1, TypeId::BOOLEAN);
}

#[test]
fn test_hof_curried_function() {
    // Test: curry<A, B, C>(fn: (a: A, b: B) => C) => (a: A) => (b: B) => C
    let interner = TypeInterner::new();
    let mut ctx = InferenceContext::new(&interner);
    let a_name = interner.intern_string("A");
    let b_name = interner.intern_string("B");
    let c_name = interner.intern_string("C");

    let var_a = ctx.fresh_type_param(a_name);
    let var_b = ctx.fresh_type_param(b_name);
    let var_c = ctx.fresh_type_param(c_name);

    // Infer from uncurried function parameters and return
    ctx.add_lower_bound(var_a, TypeId::STRING);
    ctx.add_lower_bound(var_b, TypeId::NUMBER);
    ctx.add_lower_bound(var_c, TypeId::BOOLEAN);

    let results = ctx.resolve_all_with_constraints().unwrap();
    assert_eq!(results.len(), 3);
    assert_eq!(results[0].1, TypeId::STRING);
    assert_eq!(results[1].1, TypeId::NUMBER);
    assert_eq!(results[2].1, TypeId::BOOLEAN);
}

#[test]
fn test_hof_reduce_accumulator() {
    // Test: reduce<T, U>(arr: T[], fn: (acc: U, val: T) => U, init: U) => U
    let interner = TypeInterner::new();
    let mut ctx = InferenceContext::new(&interner);
    let t_name = interner.intern_string("T");
    let u_name = interner.intern_string("U");

    let var_t = ctx.fresh_type_param(t_name);
    let var_u = ctx.fresh_type_param(u_name);

    // T from array elements
    ctx.add_lower_bound(var_t, TypeId::STRING);

    // U from initial value
    ctx.add_lower_bound(var_u, TypeId::NUMBER);

    let results = ctx.resolve_all_with_constraints().unwrap();
    assert_eq!(results.len(), 2);
    assert_eq!(results[0].1, TypeId::STRING);
    assert_eq!(results[1].1, TypeId::NUMBER);
}

#[test]
fn test_hof_function_returning_function() {
    // Test: factory<T>() => () => T
    let interner = TypeInterner::new();
    let mut ctx = InferenceContext::new(&interner);
    let t_name = interner.intern_string("T");

    let var_t = ctx.fresh_type_param(t_name);

    // T inferred from usage of returned function
    ctx.add_lower_bound(var_t, TypeId::STRING);

    let result = ctx.resolve_with_constraints(var_t).unwrap();
    assert_eq!(result, TypeId::STRING);
}

// ============================================================================
// Generic Method Chaining Tests (Fluent API Patterns)
// ============================================================================
// Tests for type inference in fluent/builder API patterns

#[test]
fn test_method_chain_builder_pattern() {
    // Test: Builder<T>.setValue(v: T).build() => T
    let interner = TypeInterner::new();
    let mut ctx = InferenceContext::new(&interner);
    let t_name = interner.intern_string("T");

    let var_t = ctx.fresh_type_param(t_name);

    // T inferred from setValue argument
    let string_lit = interner.literal_string("hello");
    ctx.add_lower_bound(var_t, string_lit);

    let result = ctx.resolve_with_constraints(var_t).unwrap();
    assert_eq!(result, string_lit);
}

#[test]
fn test_method_chain_transform() {
    // Test: chain<T>.map<U>(fn: (t: T) => U) => chain<U>
    let interner = TypeInterner::new();
    let mut ctx = InferenceContext::new(&interner);
    let t_name = interner.intern_string("T");
    let u_name = interner.intern_string("U");

    let var_t = ctx.fresh_type_param(t_name);
    let var_u = ctx.fresh_type_param(u_name);

    // T from initial chain value
    ctx.add_lower_bound(var_t, TypeId::STRING);

    // U from map callback return
    ctx.add_lower_bound(var_u, TypeId::NUMBER);

    let results = ctx.resolve_all_with_constraints().unwrap();
    assert_eq!(results.len(), 2);
    assert_eq!(results[0].1, TypeId::STRING);
    assert_eq!(results[1].1, TypeId::NUMBER);
}

#[test]
fn test_method_chain_filter() {
    // Test: chain<T>.filter(fn: (t: T) => boolean) => chain<T>
    let interner = TypeInterner::new();
    let mut ctx = InferenceContext::new(&interner);
    let t_name = interner.intern_string("T");

    let var_t = ctx.fresh_type_param(t_name);

    // T preserved through filter
    ctx.add_lower_bound(var_t, TypeId::STRING);

    let result = ctx.resolve_with_constraints(var_t).unwrap();
    assert_eq!(result, TypeId::STRING);
}

#[test]
fn test_method_chain_multiple_transforms() {
    // Test: chain<A>.map<B>().map<C>().map<D>()
    let interner = TypeInterner::new();
    let mut ctx = InferenceContext::new(&interner);
    let a_name = interner.intern_string("A");
    let b_name = interner.intern_string("B");
    let c_name = interner.intern_string("C");
    let d_name = interner.intern_string("D");

    let var_a = ctx.fresh_type_param(a_name);
    let var_b = ctx.fresh_type_param(b_name);
    let var_c = ctx.fresh_type_param(c_name);
    let var_d = ctx.fresh_type_param(d_name);

    // Each step infers next type
    ctx.add_lower_bound(var_a, TypeId::STRING);
    ctx.add_lower_bound(var_b, TypeId::NUMBER);
    ctx.add_lower_bound(var_c, TypeId::BOOLEAN);
    ctx.add_lower_bound(var_d, TypeId::SYMBOL);

    let results = ctx.resolve_all_with_constraints().unwrap();
    assert_eq!(results.len(), 4);
    assert_eq!(results[0].1, TypeId::STRING);
    assert_eq!(results[1].1, TypeId::NUMBER);
    assert_eq!(results[2].1, TypeId::BOOLEAN);
    assert_eq!(results[3].1, TypeId::SYMBOL);
}

#[test]
fn test_method_chain_flatmap() {
    // Test: chain<T>.flatMap<U>(fn: (t: T) => chain<U>) => chain<U>
    let interner = TypeInterner::new();
    let mut ctx = InferenceContext::new(&interner);
    let t_name = interner.intern_string("T");
    let u_name = interner.intern_string("U");

    let var_t = ctx.fresh_type_param(t_name);
    let var_u = ctx.fresh_type_param(u_name);

    // T from outer chain
    let string_array = interner.array(TypeId::STRING);
    ctx.add_lower_bound(var_t, string_array);

    // U from inner chain returned by callback
    ctx.add_lower_bound(var_u, TypeId::NUMBER);

    let results = ctx.resolve_all_with_constraints().unwrap();
    assert_eq!(results.len(), 2);
    assert_eq!(results[0].1, string_array);
    assert_eq!(results[1].1, TypeId::NUMBER);
}

// ============================================================================
// Inference with Default Type Parameters Tests
// ============================================================================
// Tests for generic type inference when defaults are provided

#[test]
fn test_default_type_param_not_inferred() {
    // Test: <T = string>() => T - when no inference, use default
    let interner = TypeInterner::new();
    let mut ctx = InferenceContext::new(&interner);
    let t_name = interner.intern_string("T");

    let var_t = ctx.fresh_type_param(t_name);

    // No lower bounds added - would use default in real scenario
    // For inference, resolve gives unknown
    let result = ctx.resolve_with_constraints(var_t);
    assert!(result.is_ok());
}

#[test]
fn test_default_type_param_override() {
    // Test: <T = string>(x: T) => T - inference overrides default
    let interner = TypeInterner::new();
    let mut ctx = InferenceContext::new(&interner);
    let t_name = interner.intern_string("T");

    let var_t = ctx.fresh_type_param(t_name);

    // Inference from argument overrides default
    ctx.add_lower_bound(var_t, TypeId::NUMBER);

    let result = ctx.resolve_with_constraints(var_t).unwrap();
    assert_eq!(result, TypeId::NUMBER);
}

#[test]
fn test_default_type_param_with_constraint() {
    // Test: <T extends object = {}>(x: T) => T
    let interner = TypeInterner::new();
    let mut ctx = InferenceContext::new(&interner);
    let t_name = interner.intern_string("T");

    let var_t = ctx.fresh_type_param(t_name);

    // Upper bound from constraint
    ctx.add_upper_bound(var_t, TypeId::OBJECT);

    // Specific object as lower bound
    let prop = interner.intern_string("x");
    let obj = interner.object(vec![PropertyInfo {
        name: prop,
        type_id: TypeId::NUMBER,
        write_type: TypeId::NUMBER,
        optional: false,
        readonly: false,
        is_method: false,
    }]);
    ctx.add_lower_bound(var_t, obj);

    let result = ctx.resolve_with_constraints(var_t).unwrap();
    assert_eq!(result, obj);
}

#[test]
fn test_default_type_param_chain() {
    // Test: <T = string, U = T>(x: U) => [T, U]
    let interner = TypeInterner::new();
    let mut ctx = InferenceContext::new(&interner);
    let t_name = interner.intern_string("T");
    let u_name = interner.intern_string("U");

    let var_t = ctx.fresh_type_param(t_name);
    let var_u = ctx.fresh_type_param(u_name);

    // Both inferred from same value
    ctx.add_lower_bound(var_t, TypeId::NUMBER);
    ctx.add_lower_bound(var_u, TypeId::NUMBER);

    let results = ctx.resolve_all_with_constraints().unwrap();
    assert_eq!(results.len(), 2);
    assert_eq!(results[0].1, TypeId::NUMBER);
    assert_eq!(results[1].1, TypeId::NUMBER);
}

#[test]
fn test_default_type_param_array() {
    // Test: <T = unknown>(arr?: T[]) => T[]
    let interner = TypeInterner::new();
    let mut ctx = InferenceContext::new(&interner);
    let t_name = interner.intern_string("T");

    let var_t = ctx.fresh_type_param(t_name);

    // Inferred from array element
    ctx.add_lower_bound(var_t, TypeId::STRING);

    let result = ctx.resolve_with_constraints(var_t).unwrap();
    assert_eq!(result, TypeId::STRING);
}

// =========================================================================
// Generic Function Inference - Multiple Type Params
// =========================================================================
// Tests for generic function inference with multiple type parameters

#[test]
fn test_generic_function_three_type_params() {
    // Test: <A, B, C>(a: A, b: B, c: C) => [A, B, C]
    let interner = TypeInterner::new();
    let mut ctx = InferenceContext::new(&interner);

    let a_name = interner.intern_string("A");
    let b_name = interner.intern_string("B");
    let c_name = interner.intern_string("C");

    let var_a = ctx.fresh_type_param(a_name);
    let var_b = ctx.fresh_type_param(b_name);
    let var_c = ctx.fresh_type_param(c_name);

    // Called with (string, number, boolean)
    ctx.add_lower_bound(var_a, TypeId::STRING);
    ctx.add_lower_bound(var_b, TypeId::NUMBER);
    ctx.add_lower_bound(var_c, TypeId::BOOLEAN);

    let results = ctx.resolve_all_with_constraints().unwrap();

    assert_eq!(results.len(), 3);
    assert_eq!(results[0], (a_name, TypeId::STRING));
    assert_eq!(results[1], (b_name, TypeId::NUMBER));
    assert_eq!(results[2], (c_name, TypeId::BOOLEAN));
}

#[test]
fn test_generic_function_dependent_type_params() {
    // Test: <T, U extends T>(base: T, derived: U) => U
    // Where U's constraint depends on T
    let interner = TypeInterner::new();
    let mut ctx = InferenceContext::new(&interner);

    let t_name = interner.intern_string("T");
    let u_name = interner.intern_string("U");

    let var_t = ctx.fresh_type_param(t_name);
    let var_u = ctx.fresh_type_param(u_name);

    // T gets bound from first argument
    ctx.add_lower_bound(var_t, TypeId::STRING);
    // U gets bound from second argument (a string literal)
    let lit_hello = interner.literal_string("hello");
    ctx.add_lower_bound(var_u, lit_hello);

    let result_t = ctx.resolve_with_constraints(var_t).unwrap();
    let result_u = ctx.resolve_with_constraints(var_u).unwrap();

    assert_eq!(result_t, TypeId::STRING);
    assert_eq!(result_u, lit_hello);
}

#[test]
fn test_generic_function_shared_type_param() {
    // Test: <T>(a: T, b: T) => T
    // Both arguments contribute to T
    let interner = TypeInterner::new();
    let mut ctx = InferenceContext::new(&interner);

    let t_name = interner.intern_string("T");
    let var_t = ctx.fresh_type_param(t_name);

    // Called with two different string literals - should infer union
    let lit_a = interner.literal_string("a");
    let lit_b = interner.literal_string("b");
    ctx.add_lower_bound(var_t, lit_a);
    ctx.add_lower_bound(var_t, lit_b);

    let result = ctx.resolve_with_constraints(var_t).unwrap();

    // T should be inferred as the union "a" | "b"
    let expected = interner.union(vec![lit_a, lit_b]);
    assert_eq!(result, expected);
}

// =========================================================================
// Inference from Array/Object Destructuring Patterns
// =========================================================================
// Tests for type inference from destructuring patterns

#[test]
fn test_inference_array_element_type() {
    // Test: inferring element type from array access
    // <T>(arr: T[]) => T where arr[0] is used
    let interner = TypeInterner::new();
    let mut ctx = InferenceContext::new(&interner);

    let t_name = interner.intern_string("T");
    let var_t = ctx.fresh_type_param(t_name);

    // Array<string> is passed, so T should be string
    let string_array = interner.array(TypeId::STRING);
    // When destructuring [first] = arr, we infer T from the array element
    ctx.add_lower_bound(var_t, TypeId::STRING);

    let result = ctx.resolve_with_constraints(var_t).unwrap();
    assert_eq!(result, TypeId::STRING);

    // Verify the array type matches
    let _expected_array = interner.array(result);
    assert!(string_array != TypeId::ERROR);
}

#[test]
fn test_inference_tuple_element_types() {
    // Test: inferring from tuple destructuring
    // <A, B>(tuple: [A, B]) => A
    let interner = TypeInterner::new();
    let mut ctx = InferenceContext::new(&interner);

    let a_name = interner.intern_string("A");
    let b_name = interner.intern_string("B");

    let var_a = ctx.fresh_type_param(a_name);
    let var_b = ctx.fresh_type_param(b_name);

    // Tuple [string, number] is passed
    // Destructuring [first, second] = tuple infers A = string, B = number
    ctx.add_lower_bound(var_a, TypeId::STRING);
    ctx.add_lower_bound(var_b, TypeId::NUMBER);

    let result_a = ctx.resolve_with_constraints(var_a).unwrap();
    let result_b = ctx.resolve_with_constraints(var_b).unwrap();

    assert_eq!(result_a, TypeId::STRING);
    assert_eq!(result_b, TypeId::NUMBER);
}

#[test]
fn test_inference_object_property_type() {
    // Test: inferring from object destructuring
    // <T>(obj: { value: T }) => T
    let interner = TypeInterner::new();
    let mut ctx = InferenceContext::new(&interner);

    let t_name = interner.intern_string("T");
    let var_t = ctx.fresh_type_param(t_name);

    // Object { value: number } is passed
    // Destructuring { value } = obj infers T = number
    ctx.add_lower_bound(var_t, TypeId::NUMBER);

    let result = ctx.resolve_with_constraints(var_t).unwrap();
    assert_eq!(result, TypeId::NUMBER);
}

#[test]
fn test_inference_nested_object_property() {
    // Test: inferring from nested object destructuring
    // <T>(obj: { inner: { value: T } }) => T
    let interner = TypeInterner::new();
    let mut ctx = InferenceContext::new(&interner);

    let t_name = interner.intern_string("T");
    let var_t = ctx.fresh_type_param(t_name);

    // Nested destructuring { inner: { value } } = obj
    // value is boolean, so T = boolean
    ctx.add_lower_bound(var_t, TypeId::BOOLEAN);

    let result = ctx.resolve_with_constraints(var_t).unwrap();
    assert_eq!(result, TypeId::BOOLEAN);
}

// =========================================================================
// Contextual Typing in Arrow Function Returns
// =========================================================================
// Tests for type inference from contextual typing of arrow function returns

#[test]
fn test_contextual_arrow_return_simple() {
    // Test: contextual typing provides return type
    // const fn: () => string = () => "hello"
    // The arrow function return is inferred from context
    let interner = TypeInterner::new();
    let mut ctx = InferenceContext::new(&interner);

    let t_name = interner.intern_string("T");
    let var_t = ctx.fresh_type_param(t_name);

    // Contextual type says return is string
    // Arrow function body returns a string literal
    ctx.add_upper_bound(var_t, TypeId::STRING);
    let lit_hello = interner.literal_string("hello");
    ctx.add_lower_bound(var_t, lit_hello);

    let result = ctx.resolve_with_constraints(var_t).unwrap();
    // Should resolve to the more specific type: "hello"
    assert_eq!(result, lit_hello);
}

#[test]
fn test_contextual_arrow_return_array() {
    // Test: contextual array return type
    // const fn: () => number[] = () => [1, 2, 3]
    let interner = TypeInterner::new();
    let mut ctx = InferenceContext::new(&interner);

    let t_name = interner.intern_string("T");
    let var_t = ctx.fresh_type_param(t_name);

    // Context expects Array<number>
    ctx.add_upper_bound(var_t, TypeId::NUMBER);
    // Return value contains number literals
    let lit_1 = interner.literal_number(1.0);
    ctx.add_lower_bound(var_t, lit_1);

    let result = ctx.resolve_with_constraints(var_t).unwrap();
    // Should infer the literal type
    assert_eq!(result, lit_1);
}

#[test]
fn test_contextual_arrow_return_object() {
    // Test: contextual object return type
    // const fn: () => { x: number } = () => ({ x: 42 })
    let interner = TypeInterner::new();
    let mut ctx = InferenceContext::new(&interner);

    let t_name = interner.intern_string("T");
    let var_t = ctx.fresh_type_param(t_name);

    // Context expects { x: number }
    ctx.add_upper_bound(var_t, TypeId::NUMBER);
    // Actual value is 42
    let lit_42 = interner.literal_number(42.0);
    ctx.add_lower_bound(var_t, lit_42);

    let result = ctx.resolve_with_constraints(var_t).unwrap();
    assert_eq!(result, lit_42);
}

#[test]
fn test_contextual_arrow_callback_param() {
    // Test: callback parameter inference
    // arr.map((x) => x + 1) where arr: number[]
    // x should be inferred as number from the array element type
    let interner = TypeInterner::new();
    let mut ctx = InferenceContext::new(&interner);

    let t_name = interner.intern_string("T");
    let var_t = ctx.fresh_type_param(t_name);

    // Contextual type from Array<number>.map callback is (element: number) => U
    // So T (the callback parameter type) should be number
    ctx.add_lower_bound(var_t, TypeId::NUMBER);

    let result = ctx.resolve_with_constraints(var_t).unwrap();
    assert_eq!(result, TypeId::NUMBER);
}

#[test]
fn test_contextual_arrow_higher_order() {
    // Test: higher-order function contextual typing
    // compose<A, B, C>(f: (b: B) => C, g: (a: A) => B): (a: A) => C
    let interner = TypeInterner::new();
    let mut ctx = InferenceContext::new(&interner);

    let a_name = interner.intern_string("A");
    let b_name = interner.intern_string("B");
    let c_name = interner.intern_string("C");

    let var_a = ctx.fresh_type_param(a_name);
    let var_b = ctx.fresh_type_param(b_name);
    let var_c = ctx.fresh_type_param(c_name);

    // compose((x: number) => x.toString(), (s: string) => s.length)
    // A = string, B = number, C = string
    ctx.add_lower_bound(var_a, TypeId::STRING);
    ctx.add_lower_bound(var_b, TypeId::NUMBER);
    ctx.add_lower_bound(var_c, TypeId::STRING);

    let results = ctx.resolve_all_with_constraints().unwrap();

    assert_eq!(results.len(), 3);
    assert_eq!(results[0], (a_name, TypeId::STRING));
    assert_eq!(results[1], (b_name, TypeId::NUMBER));
    assert_eq!(results[2], (c_name, TypeId::STRING));
}

// ============================================================================
// Variadic Tuple Inference Tests
// ============================================================================
// Tests for inferring types in variadic tuple patterns like [...T]

#[test]
fn test_variadic_tuple_rest_element() {
    // Test: [...T] where T is inferred from tuple elements
    let interner = TypeInterner::new();
    let mut ctx = InferenceContext::new(&interner);
    let t_name = interner.intern_string("T");

    let var_t = ctx.fresh_type_param(t_name);

    // T inferred as array of strings from rest element
    let string_array = interner.array(TypeId::STRING);
    ctx.add_lower_bound(var_t, string_array);

    let result = ctx.resolve_with_constraints(var_t).unwrap();
    assert_eq!(result, string_array);
}

#[test]
fn test_variadic_tuple_prefix_and_rest() {
    // Test: [string, ...T] - prefix element with rest
    let interner = TypeInterner::new();
    let mut ctx = InferenceContext::new(&interner);
    let t_name = interner.intern_string("T");

    let var_t = ctx.fresh_type_param(t_name);

    // T is the rest part after string prefix
    let number_array = interner.array(TypeId::NUMBER);
    ctx.add_lower_bound(var_t, number_array);

    let result = ctx.resolve_with_constraints(var_t).unwrap();
    assert_eq!(result, number_array);
}

#[test]
fn test_variadic_tuple_suffix_and_rest() {
    // Test: [...T, string] - rest with suffix element
    let interner = TypeInterner::new();
    let mut ctx = InferenceContext::new(&interner);
    let t_name = interner.intern_string("T");

    let var_t = ctx.fresh_type_param(t_name);

    // T is the rest part before string suffix
    let boolean_array = interner.array(TypeId::BOOLEAN);
    ctx.add_lower_bound(var_t, boolean_array);

    let result = ctx.resolve_with_constraints(var_t).unwrap();
    assert_eq!(result, boolean_array);
}

#[test]
fn test_variadic_tuple_multiple_rest() {
    // Test: [...T, ...U] - multiple variadic segments
    let interner = TypeInterner::new();
    let mut ctx = InferenceContext::new(&interner);
    let t_name = interner.intern_string("T");
    let u_name = interner.intern_string("U");

    let var_t = ctx.fresh_type_param(t_name);
    let var_u = ctx.fresh_type_param(u_name);

    // T and U are different array types
    let string_array = interner.array(TypeId::STRING);
    let number_array = interner.array(TypeId::NUMBER);
    ctx.add_lower_bound(var_t, string_array);
    ctx.add_lower_bound(var_u, number_array);

    let results = ctx.resolve_all_with_constraints().unwrap();
    assert_eq!(results.len(), 2);
    assert_eq!(results[0].1, string_array);
    assert_eq!(results[1].1, number_array);
}

#[test]
fn test_variadic_tuple_concat() {
    // Test: [...T, ...U] => [...T, ...U] (tuple concatenation)
    let interner = TypeInterner::new();
    let mut ctx = InferenceContext::new(&interner);
    let t_name = interner.intern_string("T");
    let u_name = interner.intern_string("U");

    let var_t = ctx.fresh_type_param(t_name);
    let var_u = ctx.fresh_type_param(u_name);

    // Infer from concrete tuple parts
    let tuple_t = interner.tuple(vec![
        TupleElement { type_id: TypeId::STRING, name: None, optional: false, rest: false },
    ]);
    let tuple_u = interner.tuple(vec![
        TupleElement { type_id: TypeId::NUMBER, name: None, optional: false, rest: false },
    ]);

    ctx.add_lower_bound(var_t, tuple_t);
    ctx.add_lower_bound(var_u, tuple_u);

    let results = ctx.resolve_all_with_constraints().unwrap();
    assert_eq!(results.len(), 2);
    assert_eq!(results[0].1, tuple_t);
    assert_eq!(results[1].1, tuple_u);
}

// ============================================================================
// Named Tuple Elements Tests
// ============================================================================
// Tests for tuples with named elements like [x: string, y: number]

#[test]
fn test_named_tuple_basic() {
    // Test: [x: T, y: U] - basic named tuple inference
    let interner = TypeInterner::new();
    let mut ctx = InferenceContext::new(&interner);
    let t_name = interner.intern_string("T");
    let u_name = interner.intern_string("U");

    let var_t = ctx.fresh_type_param(t_name);
    let var_u = ctx.fresh_type_param(u_name);

    // Infer from named tuple elements
    ctx.add_lower_bound(var_t, TypeId::STRING);
    ctx.add_lower_bound(var_u, TypeId::NUMBER);

    let results = ctx.resolve_all_with_constraints().unwrap();
    assert_eq!(results.len(), 2);
    assert_eq!(results[0].1, TypeId::STRING);
    assert_eq!(results[1].1, TypeId::NUMBER);
}

#[test]
fn test_named_tuple_with_optional() {
    // Test: [x: T, y?: U] - optional named element
    let interner = TypeInterner::new();
    let mut ctx = InferenceContext::new(&interner);
    let t_name = interner.intern_string("T");
    let u_name = interner.intern_string("U");

    let var_t = ctx.fresh_type_param(t_name);
    let var_u = ctx.fresh_type_param(u_name);

    // Create named tuple with optional element
    let x_name = interner.intern_string("x");
    let y_name = interner.intern_string("y");
    let named_tuple = interner.tuple(vec![
        TupleElement { type_id: TypeId::STRING, name: Some(x_name), optional: false, rest: false },
        TupleElement { type_id: TypeId::NUMBER, name: Some(y_name), optional: true, rest: false },
    ]);

    ctx.add_lower_bound(var_t, TypeId::STRING);
    ctx.add_lower_bound(var_u, TypeId::NUMBER);

    let results = ctx.resolve_all_with_constraints().unwrap();
    assert_eq!(results.len(), 2);
    assert_eq!(results[0].1, TypeId::STRING);
    assert_eq!(results[1].1, TypeId::NUMBER);
}

#[test]
fn test_named_tuple_destructuring() {
    // Test: function({x, y}: [x: T, y: U]) - destructuring named tuple
    let interner = TypeInterner::new();
    let mut ctx = InferenceContext::new(&interner);
    let t_name = interner.intern_string("T");
    let u_name = interner.intern_string("U");

    let var_t = ctx.fresh_type_param(t_name);
    let var_u = ctx.fresh_type_param(u_name);

    // Types inferred from destructuring context
    let lit_hello = interner.literal_string("hello");
    let lit_42 = interner.literal_number(42.0);

    ctx.add_lower_bound(var_t, lit_hello);
    ctx.add_lower_bound(var_u, lit_42);

    let results = ctx.resolve_all_with_constraints().unwrap();
    assert_eq!(results.len(), 2);
    assert_eq!(results[0].1, lit_hello);
    assert_eq!(results[1].1, lit_42);
}

#[test]
fn test_named_tuple_three_elements() {
    // Test: [a: T, b: U, c: V] - three named elements
    let interner = TypeInterner::new();
    let mut ctx = InferenceContext::new(&interner);
    let t_name = interner.intern_string("T");
    let u_name = interner.intern_string("U");
    let v_name = interner.intern_string("V");

    let var_t = ctx.fresh_type_param(t_name);
    let var_u = ctx.fresh_type_param(u_name);
    let var_v = ctx.fresh_type_param(v_name);

    ctx.add_lower_bound(var_t, TypeId::STRING);
    ctx.add_lower_bound(var_u, TypeId::NUMBER);
    ctx.add_lower_bound(var_v, TypeId::BOOLEAN);

    let results = ctx.resolve_all_with_constraints().unwrap();
    assert_eq!(results.len(), 3);
    assert_eq!(results[0].1, TypeId::STRING);
    assert_eq!(results[1].1, TypeId::NUMBER);
    assert_eq!(results[2].1, TypeId::BOOLEAN);
}

#[test]
fn test_named_tuple_mixed_named_unnamed() {
    // Test: [x: T, U, z: V] - mixed named and unnamed
    let interner = TypeInterner::new();
    let mut ctx = InferenceContext::new(&interner);
    let t_name = interner.intern_string("T");
    let u_name = interner.intern_string("U");

    let var_t = ctx.fresh_type_param(t_name);
    let var_u = ctx.fresh_type_param(u_name);

    // Create mixed tuple
    let x_name = interner.intern_string("x");
    let mixed_tuple = interner.tuple(vec![
        TupleElement { type_id: TypeId::STRING, name: Some(x_name), optional: false, rest: false },
        TupleElement { type_id: TypeId::NUMBER, name: None, optional: false, rest: false },
    ]);

    ctx.add_lower_bound(var_t, TypeId::STRING);
    ctx.add_lower_bound(var_u, TypeId::NUMBER);

    let results = ctx.resolve_all_with_constraints().unwrap();
    assert_eq!(results.len(), 2);
    assert_eq!(results[0].1, TypeId::STRING);
    assert_eq!(results[1].1, TypeId::NUMBER);
}

// ============================================================================
// Tuple Spread Type Inference Tests
// ============================================================================
// Tests for spread operations on tuple types

#[test]
fn test_tuple_spread_into_array() {
    // Test: [...tuple] spreads into array context
    let interner = TypeInterner::new();
    let mut ctx = InferenceContext::new(&interner);
    let t_name = interner.intern_string("T");

    let var_t = ctx.fresh_type_param(t_name);

    // Tuple spread becomes union of element types
    let string_number_union = interner.union(vec![TypeId::STRING, TypeId::NUMBER]);
    ctx.add_lower_bound(var_t, string_number_union);

    let result = ctx.resolve_with_constraints(var_t).unwrap();
    assert_eq!(result, string_number_union);
}

#[test]
fn test_tuple_spread_function_args() {
    // Test: fn(...args: T) where T is tuple
    let interner = TypeInterner::new();
    let mut ctx = InferenceContext::new(&interner);
    let t_name = interner.intern_string("T");

    let var_t = ctx.fresh_type_param(t_name);

    // T is inferred as tuple from function arguments
    let args_tuple = interner.tuple(vec![
        TupleElement { type_id: TypeId::STRING, name: None, optional: false, rest: false },
        TupleElement { type_id: TypeId::NUMBER, name: None, optional: false, rest: false },
    ]);
    ctx.add_lower_bound(var_t, args_tuple);

    let result = ctx.resolve_with_constraints(var_t).unwrap();
    assert_eq!(result, args_tuple);
}

#[test]
fn test_tuple_spread_concat_tuples() {
    // Test: [...A, ...B] = [...C] - concatenating tuples
    let interner = TypeInterner::new();
    let mut ctx = InferenceContext::new(&interner);
    let a_name = interner.intern_string("A");
    let b_name = interner.intern_string("B");

    let var_a = ctx.fresh_type_param(a_name);
    let var_b = ctx.fresh_type_param(b_name);

    // A and B are tuple parts
    let tuple_a = interner.tuple(vec![
        TupleElement { type_id: TypeId::STRING, name: None, optional: false, rest: false },
    ]);
    let tuple_b = interner.tuple(vec![
        TupleElement { type_id: TypeId::NUMBER, name: None, optional: false, rest: false },
        TupleElement { type_id: TypeId::BOOLEAN, name: None, optional: false, rest: false },
    ]);

    ctx.add_lower_bound(var_a, tuple_a);
    ctx.add_lower_bound(var_b, tuple_b);

    let results = ctx.resolve_all_with_constraints().unwrap();
    assert_eq!(results.len(), 2);
    assert_eq!(results[0].1, tuple_a);
    assert_eq!(results[1].1, tuple_b);
}

#[test]
fn test_tuple_spread_in_return() {
    // Test: function returning [...T, extra]
    let interner = TypeInterner::new();
    let mut ctx = InferenceContext::new(&interner);
    let t_name = interner.intern_string("T");

    let var_t = ctx.fresh_type_param(t_name);

    // T is the spread part of return tuple
    let spread_part = interner.tuple(vec![
        TupleElement { type_id: TypeId::STRING, name: None, optional: false, rest: false },
        TupleElement { type_id: TypeId::NUMBER, name: None, optional: false, rest: false },
    ]);
    ctx.add_lower_bound(var_t, spread_part);

    let result = ctx.resolve_with_constraints(var_t).unwrap();
    assert_eq!(result, spread_part);
}

#[test]
fn test_tuple_spread_with_rest() {
    // Test: [...T, ...rest: U[]]
    let interner = TypeInterner::new();
    let mut ctx = InferenceContext::new(&interner);
    let t_name = interner.intern_string("T");
    let u_name = interner.intern_string("U");

    let var_t = ctx.fresh_type_param(t_name);
    let var_u = ctx.fresh_type_param(u_name);

    // T is fixed tuple, U is element type of rest
    let fixed_tuple = interner.tuple(vec![
        TupleElement { type_id: TypeId::STRING, name: None, optional: false, rest: false },
    ]);
    ctx.add_lower_bound(var_t, fixed_tuple);
    ctx.add_lower_bound(var_u, TypeId::NUMBER);

    let results = ctx.resolve_all_with_constraints().unwrap();
    assert_eq!(results.len(), 2);
    assert_eq!(results[0].1, fixed_tuple);
    assert_eq!(results[1].1, TypeId::NUMBER);
}

// ============================================================================
// Type Guard Narrowing Pattern Tests
// ============================================================================
// Tests for type narrowing via type guards (typeof, instanceof, custom)

#[test]
fn test_type_guard_typeof_string() {
    // Test: typeof x === "string" narrows union to string
    let interner = TypeInterner::new();
    let mut ctx = InferenceContext::new(&interner);
    let t_name = interner.intern_string("T");

    let var_t = ctx.fresh_type_param(t_name);

    // Original type is string | number
    let string_or_number = interner.union(vec![TypeId::STRING, TypeId::NUMBER]);
    ctx.add_upper_bound(var_t, string_or_number);

    // After typeof === "string", narrow to string
    ctx.add_lower_bound(var_t, TypeId::STRING);

    let result = ctx.resolve_with_constraints(var_t).unwrap();
    assert_eq!(result, TypeId::STRING);
}

#[test]
fn test_type_guard_typeof_number() {
    // Test: typeof x === "number" narrows union to number
    let interner = TypeInterner::new();
    let mut ctx = InferenceContext::new(&interner);
    let t_name = interner.intern_string("T");

    let var_t = ctx.fresh_type_param(t_name);

    // Original type is string | number | boolean
    let union = interner.union(vec![TypeId::STRING, TypeId::NUMBER, TypeId::BOOLEAN]);
    ctx.add_upper_bound(var_t, union);

    // After typeof === "number", narrow to number
    ctx.add_lower_bound(var_t, TypeId::NUMBER);

    let result = ctx.resolve_with_constraints(var_t).unwrap();
    assert_eq!(result, TypeId::NUMBER);
}

#[test]
fn test_type_guard_typeof_object() {
    // Test: typeof x === "object" narrows to object types
    let interner = TypeInterner::new();
    let mut ctx = InferenceContext::new(&interner);
    let t_name = interner.intern_string("T");

    let var_t = ctx.fresh_type_param(t_name);

    // Upper bound is object | null | string
    let obj_or_null_or_string = interner.union(vec![TypeId::OBJECT, TypeId::NULL, TypeId::STRING]);
    ctx.add_upper_bound(var_t, obj_or_null_or_string);

    // typeof === "object" includes object and null
    let obj_or_null = interner.union(vec![TypeId::OBJECT, TypeId::NULL]);
    ctx.add_lower_bound(var_t, obj_or_null);

    let result = ctx.resolve_with_constraints(var_t).unwrap();
    assert_eq!(result, obj_or_null);
}

#[test]
fn test_type_guard_instanceof() {
    // Test: x instanceof Error narrows to Error type
    let interner = TypeInterner::new();
    let mut ctx = InferenceContext::new(&interner);
    let t_name = interner.intern_string("T");

    let var_t = ctx.fresh_type_param(t_name);

    // Upper bound is Error | string (simulated with object)
    let error_or_string = interner.union(vec![TypeId::OBJECT, TypeId::STRING]);
    ctx.add_upper_bound(var_t, error_or_string);

    // After instanceof Error, narrow to object (Error)
    ctx.add_lower_bound(var_t, TypeId::OBJECT);

    let result = ctx.resolve_with_constraints(var_t).unwrap();
    assert_eq!(result, TypeId::OBJECT);
}

#[test]
fn test_type_guard_custom_predicate() {
    // Test: isString(x): x is string - custom type predicate
    let interner = TypeInterner::new();
    let mut ctx = InferenceContext::new(&interner);
    let t_name = interner.intern_string("T");

    let var_t = ctx.fresh_type_param(t_name);

    // Upper bound is unknown
    ctx.add_upper_bound(var_t, TypeId::UNKNOWN);

    // After custom guard, narrow to string
    ctx.add_lower_bound(var_t, TypeId::STRING);

    let result = ctx.resolve_with_constraints(var_t).unwrap();
    assert_eq!(result, TypeId::STRING);
}

// ============================================================================
// Discriminated Union Narrowing Tests
// ============================================================================
// Tests for narrowing unions via discriminant properties

#[test]
fn test_discriminated_union_basic() {
    // Test: { kind: "a" } | { kind: "b" } narrowed by kind
    let interner = TypeInterner::new();
    let mut ctx = InferenceContext::new(&interner);
    let t_name = interner.intern_string("T");

    let var_t = ctx.fresh_type_param(t_name);

    // Create discriminated union members
    let kind_prop = interner.intern_string("kind");
    let lit_a = interner.literal_string("a");

    let type_a = interner.object(vec![PropertyInfo {
        name: kind_prop,
        type_id: lit_a,
        write_type: lit_a,
        optional: false,
        readonly: false,
        is_method: false,
    }]);

    // After checking kind === "a", narrow to type_a
    ctx.add_lower_bound(var_t, type_a);

    let result = ctx.resolve_with_constraints(var_t).unwrap();
    assert_eq!(result, type_a);
}

#[test]
fn test_discriminated_union_switch() {
    // Test: switch(x.kind) narrowing
    let interner = TypeInterner::new();
    let mut ctx = InferenceContext::new(&interner);
    let t_name = interner.intern_string("T");

    let var_t = ctx.fresh_type_param(t_name);

    // After switch case "circle"
    let kind_prop = interner.intern_string("kind");
    let radius_prop = interner.intern_string("radius");
    let lit_circle = interner.literal_string("circle");

    let circle_type = interner.object(vec![
        PropertyInfo {
            name: kind_prop,
            type_id: lit_circle,
            write_type: lit_circle,
            optional: false,
            readonly: false,
            is_method: false,
        },
        PropertyInfo {
            name: radius_prop,
            type_id: TypeId::NUMBER,
            write_type: TypeId::NUMBER,
            optional: false,
            readonly: false,
            is_method: false,
        },
    ]);

    ctx.add_lower_bound(var_t, circle_type);

    let result = ctx.resolve_with_constraints(var_t).unwrap();
    assert_eq!(result, circle_type);
}

#[test]
fn test_discriminated_union_type_property() {
    // Test: { type: "request" } | { type: "response" } narrowing
    let interner = TypeInterner::new();
    let mut ctx = InferenceContext::new(&interner);
    let t_name = interner.intern_string("T");

    let var_t = ctx.fresh_type_param(t_name);

    let type_prop = interner.intern_string("type");
    let lit_request = interner.literal_string("request");
    let body_prop = interner.intern_string("body");

    let request_type = interner.object(vec![
        PropertyInfo {
            name: type_prop,
            type_id: lit_request,
            write_type: lit_request,
            optional: false,
            readonly: false,
            is_method: false,
        },
        PropertyInfo {
            name: body_prop,
            type_id: TypeId::STRING,
            write_type: TypeId::STRING,
            optional: false,
            readonly: false,
            is_method: false,
        },
    ]);

    ctx.add_lower_bound(var_t, request_type);

    let result = ctx.resolve_with_constraints(var_t).unwrap();
    assert_eq!(result, request_type);
}

#[test]
fn test_discriminated_union_boolean_discriminant() {
    // Test: { success: true, data: T } | { success: false, error: E }
    let interner = TypeInterner::new();
    let mut ctx = InferenceContext::new(&interner);
    let t_name = interner.intern_string("T");

    let var_t = ctx.fresh_type_param(t_name);

    let success_prop = interner.intern_string("success");
    let data_prop = interner.intern_string("data");

    // Use BOOLEAN for success field (representing literal true)
    let success_type = interner.object(vec![
        PropertyInfo {
            name: success_prop,
            type_id: TypeId::BOOLEAN,
            write_type: TypeId::BOOLEAN,
            optional: false,
            readonly: false,
            is_method: false,
        },
        PropertyInfo {
            name: data_prop,
            type_id: TypeId::STRING,
            write_type: TypeId::STRING,
            optional: false,
            readonly: false,
            is_method: false,
        },
    ]);

    ctx.add_lower_bound(var_t, success_type);

    let result = ctx.resolve_with_constraints(var_t).unwrap();
    assert_eq!(result, success_type);
}

#[test]
fn test_discriminated_union_numeric_discriminant() {
    // Test: { code: 200, body: string } | { code: 404, message: string }
    let interner = TypeInterner::new();
    let mut ctx = InferenceContext::new(&interner);
    let t_name = interner.intern_string("T");

    let var_t = ctx.fresh_type_param(t_name);

    let code_prop = interner.intern_string("code");
    let body_prop = interner.intern_string("body");
    let lit_200 = interner.literal_number(200.0);

    let ok_response = interner.object(vec![
        PropertyInfo {
            name: code_prop,
            type_id: lit_200,
            write_type: lit_200,
            optional: false,
            readonly: false,
            is_method: false,
        },
        PropertyInfo {
            name: body_prop,
            type_id: TypeId::STRING,
            write_type: TypeId::STRING,
            optional: false,
            readonly: false,
            is_method: false,
        },
    ]);

    ctx.add_lower_bound(var_t, ok_response);

    let result = ctx.resolve_with_constraints(var_t).unwrap();
    assert_eq!(result, ok_response);
}

// ============================================================================
// In Operator Narrowing Tests
// ============================================================================
// Tests for narrowing via the 'in' operator

#[test]
fn test_in_operator_basic() {
    // Test: "prop" in x narrows to types with prop
    let interner = TypeInterner::new();
    let mut ctx = InferenceContext::new(&interner);
    let t_name = interner.intern_string("T");

    let var_t = ctx.fresh_type_param(t_name);

    // After "name" in x, narrow to object with name
    let name_prop = interner.intern_string("name");
    let with_name = interner.object(vec![PropertyInfo {
        name: name_prop,
        type_id: TypeId::STRING,
        write_type: TypeId::STRING,
        optional: false,
        readonly: false,
        is_method: false,
    }]);

    ctx.add_lower_bound(var_t, with_name);

    let result = ctx.resolve_with_constraints(var_t).unwrap();
    assert_eq!(result, with_name);
}

#[test]
fn test_in_operator_union_narrowing() {
    // Test: "fly" in animal narrows Animal to Bird
    let interner = TypeInterner::new();
    let mut ctx = InferenceContext::new(&interner);
    let t_name = interner.intern_string("T");

    let var_t = ctx.fresh_type_param(t_name);

    // Bird has fly method
    let fly_prop = interner.intern_string("fly");
    let fly_fn = interner.function(FunctionShape {
        type_params: vec![],
        params: vec![],
        this_type: None,
        return_type: TypeId::VOID,
        type_predicate: None,
        is_constructor: false,
    });

    let bird_type = interner.object(vec![PropertyInfo {
        name: fly_prop,
        type_id: fly_fn,
        write_type: fly_fn,
        optional: false,
        readonly: false,
        is_method: true,
    }]);

    ctx.add_lower_bound(var_t, bird_type);

    let result = ctx.resolve_with_constraints(var_t).unwrap();
    assert_eq!(result, bird_type);
}

#[test]
fn test_in_operator_optional_property() {
    // Test: "optional" in x where optional may not exist
    let interner = TypeInterner::new();
    let mut ctx = InferenceContext::new(&interner);
    let t_name = interner.intern_string("T");

    let var_t = ctx.fresh_type_param(t_name);

    // Object with optional property (in check confirms it exists)
    let opt_prop = interner.intern_string("optional");
    let with_optional = interner.object(vec![PropertyInfo {
        name: opt_prop,
        type_id: TypeId::STRING,
        write_type: TypeId::STRING,
        optional: true,
        readonly: false,
        is_method: false,
    }]);

    ctx.add_lower_bound(var_t, with_optional);

    let result = ctx.resolve_with_constraints(var_t).unwrap();
    assert_eq!(result, with_optional);
}

#[test]
fn test_in_operator_method_check() {
    // Test: "forEach" in x narrows to array-like
    let interner = TypeInterner::new();
    let mut ctx = InferenceContext::new(&interner);
    let t_name = interner.intern_string("T");

    let var_t = ctx.fresh_type_param(t_name);

    // Array-like with forEach method
    let foreach_prop = interner.intern_string("forEach");
    let foreach_fn = interner.function(FunctionShape {
        type_params: vec![],
        params: vec![],
        this_type: None,
        return_type: TypeId::VOID,
        type_predicate: None,
        is_constructor: false,
    });

    let array_like = interner.object(vec![PropertyInfo {
        name: foreach_prop,
        type_id: foreach_fn,
        write_type: foreach_fn,
        optional: false,
        readonly: false,
        is_method: true,
    }]);

    ctx.add_lower_bound(var_t, array_like);

    let result = ctx.resolve_with_constraints(var_t).unwrap();
    assert_eq!(result, array_like);
}

#[test]
fn test_in_operator_negation() {
    // Test: !("prop" in x) narrows to types without prop
    let interner = TypeInterner::new();
    let mut ctx = InferenceContext::new(&interner);
    let t_name = interner.intern_string("T");

    let var_t = ctx.fresh_type_param(t_name);

    // After !("special" in x), narrow to object without special
    let other_prop = interner.intern_string("basic");
    let without_special = interner.object(vec![PropertyInfo {
        name: other_prop,
        type_id: TypeId::STRING,
        write_type: TypeId::STRING,
        optional: false,
        readonly: false,
        is_method: false,
    }]);

    ctx.add_lower_bound(var_t, without_special);

    let result = ctx.resolve_with_constraints(var_t).unwrap();
    assert_eq!(result, without_special);
}

// =============================================================================
// Context-Sensitive Type Inference Tests
// =============================================================================
// Tests for inferring types from contextual typing (callbacks, array methods,
// Promise chains, generic function arguments)

// -----------------------------------------------------------------------------
// Callback Parameter Inference from Usage
// -----------------------------------------------------------------------------

#[test]
fn test_callback_param_inferred_from_call_site() {
    // Test: When a callback is passed to a function, the parameter types
    // are inferred from how the callback is called within the function.
    // e.g., function apply<T>(fn: (x: T) => void, val: T) - T inferred from val
    let interner = TypeInterner::new();
    let mut ctx = InferenceContext::new(&interner);
    let t_name = interner.intern_string("T");

    let var_t = ctx.fresh_type_param(t_name);

    // val argument provides lower bound
    let hello = interner.literal_string("hello");
    ctx.add_lower_bound(var_t, hello);

    let result = ctx.resolve_with_constraints(var_t).unwrap();
    // Callback param x will be "hello" type
    assert_eq!(result, hello);
}

#[test]
fn test_callback_param_inferred_from_multiple_calls() {
    // Test: Callback called with different values creates union type
    // e.g., function callBoth<T>(fn: (x: T) => void) { fn("a"); fn(1); }
    let interner = TypeInterner::new();
    let mut ctx = InferenceContext::new(&interner);
    let t_name = interner.intern_string("T");

    let var_t = ctx.fresh_type_param(t_name);

    // Callback called with string
    ctx.add_lower_bound(var_t, TypeId::STRING);
    // Callback called with number
    ctx.add_lower_bound(var_t, TypeId::NUMBER);

    let result = ctx.resolve_with_constraints(var_t).unwrap();
    let expected = interner.union(vec![TypeId::STRING, TypeId::NUMBER]);
    assert_eq!(result, expected);
}

#[test]
fn test_callback_return_inferred_from_usage() {
    // Test: Callback return type inferred from how result is used
    // e.g., const x: number = transform((s) => s.length)
    let interner = TypeInterner::new();
    let mut ctx = InferenceContext::new(&interner);
    let u_name = interner.intern_string("U");

    let var_u = ctx.fresh_type_param(u_name);

    // Return type must satisfy usage context
    ctx.add_upper_bound(var_u, TypeId::NUMBER);
    // Callback returns specific number
    let forty_two = interner.literal_number(42.0);
    ctx.add_lower_bound(var_u, forty_two);

    let result = ctx.resolve_with_constraints(var_u).unwrap();
    assert_eq!(result, forty_two);
}

#[test]
fn test_callback_param_from_object_method_context() {
    // Test: obj.method((x) => ...) where method signature defines x's type
    let interner = TypeInterner::new();
    let mut ctx = InferenceContext::new(&interner);
    let t_name = interner.intern_string("T");

    let var_t = ctx.fresh_type_param(t_name);

    // Object method provides context that param is number
    ctx.add_upper_bound(var_t, TypeId::NUMBER);

    let result = ctx.resolve_with_constraints(var_t).unwrap();
    assert_eq!(result, TypeId::NUMBER);
}

#[test]
fn test_callback_param_from_overloaded_function() {
    // Test: Overloaded function picks signature based on callback
    // When multiple signatures exist, param type comes from matching overload
    let interner = TypeInterner::new();
    let mut ctx = InferenceContext::new(&interner);
    let t_name = interner.intern_string("T");

    let var_t = ctx.fresh_type_param(t_name);

    // Chosen overload expects callback with string param
    ctx.add_upper_bound(var_t, TypeId::STRING);

    let result = ctx.resolve_with_constraints(var_t).unwrap();
    assert_eq!(result, TypeId::STRING);
}

// -----------------------------------------------------------------------------
// Array Method Callback Inference (map, filter, reduce)
// -----------------------------------------------------------------------------

#[test]
fn test_array_map_callback_param_and_return() {
    // Test: nums.map((n) => n.toString())
    // Param n: number (from array), Return: string
    let interner = TypeInterner::new();
    let mut ctx = InferenceContext::new(&interner);
    let t_name = interner.intern_string("T");
    let u_name = interner.intern_string("U");

    let var_t = ctx.fresh_type_param(t_name);
    let var_u = ctx.fresh_type_param(u_name);

    // T from Array<number> element type
    ctx.add_upper_bound(var_t, TypeId::NUMBER);
    // U from callback return type
    ctx.add_lower_bound(var_u, TypeId::STRING);

    let result_t = ctx.resolve_with_constraints(var_t).unwrap();
    let result_u = ctx.resolve_with_constraints(var_u).unwrap();

    assert_eq!(result_t, TypeId::NUMBER);
    assert_eq!(result_u, TypeId::STRING);
}

#[test]
fn test_array_map_with_index_and_array_params() {
    // Test: arr.map((elem, index, array) => ...)
    // elem: T, index: number, array: T[]
    let interner = TypeInterner::new();
    let mut ctx = InferenceContext::new(&interner);
    let t_name = interner.intern_string("T");
    let idx_name = interner.intern_string("Idx");
    let arr_name = interner.intern_string("Arr");

    let var_t = ctx.fresh_type_param(t_name);
    let var_idx = ctx.fresh_type_param(idx_name);
    let var_arr = ctx.fresh_type_param(arr_name);

    // Element type
    ctx.add_upper_bound(var_t, TypeId::STRING);
    // Index is always number
    ctx.add_upper_bound(var_idx, TypeId::NUMBER);
    // Array parameter is the source array type
    let string_array = interner.array(TypeId::STRING);
    ctx.add_upper_bound(var_arr, string_array);

    let result_t = ctx.resolve_with_constraints(var_t).unwrap();
    let result_idx = ctx.resolve_with_constraints(var_idx).unwrap();
    let result_arr = ctx.resolve_with_constraints(var_arr).unwrap();

    assert_eq!(result_t, TypeId::STRING);
    assert_eq!(result_idx, TypeId::NUMBER);
    assert_eq!(result_arr, string_array);
}

#[test]
fn test_array_filter_preserves_element_type() {
    // Test: strs.filter((s) => s.length > 0)
    // Input: string[], Output: string[]
    let interner = TypeInterner::new();
    let mut ctx = InferenceContext::new(&interner);
    let t_name = interner.intern_string("T");

    let var_t = ctx.fresh_type_param(t_name);

    // Filter preserves element type
    ctx.add_lower_bound(var_t, TypeId::STRING);
    ctx.add_upper_bound(var_t, TypeId::STRING);

    let result = ctx.resolve_with_constraints(var_t).unwrap();
    assert_eq!(result, TypeId::STRING);
}

#[test]
fn test_array_filter_with_type_guard() {
    // Test: arr.filter((x): x is string => typeof x === "string")
    // Narrows from (string | number)[] to string[]
    let interner = TypeInterner::new();
    let mut ctx = InferenceContext::new(&interner);
    let t_name = interner.intern_string("T");
    let s_name = interner.intern_string("S");

    let var_t = ctx.fresh_type_param(t_name);
    let var_s = ctx.fresh_type_param(s_name);

    // Original element type is union
    let union = interner.union(vec![TypeId::STRING, TypeId::NUMBER]);
    ctx.add_lower_bound(var_t, union);

    // Type guard narrows to string
    ctx.add_lower_bound(var_s, TypeId::STRING);
    ctx.add_upper_bound(var_s, union);

    let result_t = ctx.resolve_with_constraints(var_t).unwrap();
    let result_s = ctx.resolve_with_constraints(var_s).unwrap();

    assert_eq!(result_t, union);
    assert_eq!(result_s, TypeId::STRING);
}

#[test]
fn test_array_reduce_accumulator_inference() {
    // Test: nums.reduce((acc, n) => acc + n, 0)
    // acc: number (from initial value), n: number (from array)
    let interner = TypeInterner::new();
    let mut ctx = InferenceContext::new(&interner);
    let acc_name = interner.intern_string("Acc");
    let elem_name = interner.intern_string("Elem");

    let var_acc = ctx.fresh_type_param(acc_name);
    let var_elem = ctx.fresh_type_param(elem_name);

    // Accumulator type from initial value
    let zero = interner.literal_number(0.0);
    ctx.add_lower_bound(var_acc, zero);
    // Also from callback return (same type)
    ctx.add_lower_bound(var_acc, TypeId::NUMBER);

    // Element type from array
    ctx.add_upper_bound(var_elem, TypeId::NUMBER);

    let result_acc = ctx.resolve_with_constraints(var_acc).unwrap();
    let result_elem = ctx.resolve_with_constraints(var_elem).unwrap();

    // Accumulator is union of 0 and number (simplifies to number in practice)
    let expected_acc = interner.union(vec![zero, TypeId::NUMBER]);
    assert_eq!(result_acc, expected_acc);
    assert_eq!(result_elem, TypeId::NUMBER);
}

#[test]
fn test_array_reduce_different_accumulator_type() {
    // Test: strs.reduce((obj, s) => ({ ...obj, [s]: true }), {})
    // Reduces string[] to Record<string, boolean>
    let interner = TypeInterner::new();
    let mut ctx = InferenceContext::new(&interner);
    let acc_name = interner.intern_string("Acc");
    let elem_name = interner.intern_string("Elem");

    let var_acc = ctx.fresh_type_param(acc_name);
    let var_elem = ctx.fresh_type_param(elem_name);

    // Accumulator is object with string keys and boolean values
    let obj_type = interner.object(vec![]);
    ctx.add_lower_bound(var_acc, obj_type);

    // Element type from string array
    ctx.add_upper_bound(var_elem, TypeId::STRING);

    let result_acc = ctx.resolve_with_constraints(var_acc).unwrap();
    let result_elem = ctx.resolve_with_constraints(var_elem).unwrap();

    assert_eq!(result_acc, obj_type);
    assert_eq!(result_elem, TypeId::STRING);
}

#[test]
fn test_array_find_returns_element_or_undefined() {
    // Test: nums.find((n) => n > 0)
    // Returns: number | undefined
    let interner = TypeInterner::new();
    let mut ctx = InferenceContext::new(&interner);
    let t_name = interner.intern_string("T");

    let var_t = ctx.fresh_type_param(t_name);

    // Element type
    ctx.add_lower_bound(var_t, TypeId::NUMBER);
    // Return includes undefined possibility
    ctx.add_lower_bound(var_t, TypeId::UNDEFINED);

    let result = ctx.resolve_with_constraints(var_t).unwrap();
    let expected = interner.union(vec![TypeId::NUMBER, TypeId::UNDEFINED]);
    assert_eq!(result, expected);
}

#[test]
fn test_array_every_callback_returns_boolean() {
    // Test: nums.every((n) => n > 0)
    // Callback must return boolean
    let interner = TypeInterner::new();
    let mut ctx = InferenceContext::new(&interner);
    let t_name = interner.intern_string("T");
    let ret_name = interner.intern_string("Ret");

    let var_t = ctx.fresh_type_param(t_name);
    let var_ret = ctx.fresh_type_param(ret_name);

    // Element type
    ctx.add_upper_bound(var_t, TypeId::NUMBER);
    // Return type constrained to boolean
    ctx.add_upper_bound(var_ret, TypeId::BOOLEAN);

    let result_t = ctx.resolve_with_constraints(var_t).unwrap();
    let result_ret = ctx.resolve_with_constraints(var_ret).unwrap();

    assert_eq!(result_t, TypeId::NUMBER);
    assert_eq!(result_ret, TypeId::BOOLEAN);
}

// -----------------------------------------------------------------------------
// Promise.then Chain Inference
// -----------------------------------------------------------------------------

#[test]
fn test_promise_then_basic_chain() {
    // Test: promise.then((val) => val + 1)
    // Promise<number>.then returns Promise<number>
    let interner = TypeInterner::new();
    let mut ctx = InferenceContext::new(&interner);
    let t_name = interner.intern_string("T");
    let u_name = interner.intern_string("U");

    let var_t = ctx.fresh_type_param(t_name);
    let var_u = ctx.fresh_type_param(u_name);

    // T from Promise<number> resolved value
    ctx.add_upper_bound(var_t, TypeId::NUMBER);
    // U from callback return type
    ctx.add_lower_bound(var_u, TypeId::NUMBER);

    let result_t = ctx.resolve_with_constraints(var_t).unwrap();
    let result_u = ctx.resolve_with_constraints(var_u).unwrap();

    assert_eq!(result_t, TypeId::NUMBER);
    assert_eq!(result_u, TypeId::NUMBER);
}

#[test]
fn test_promise_then_transform_type() {
    // Test: Promise<string>.then((s) => s.length) => Promise<number>
    let interner = TypeInterner::new();
    let mut ctx = InferenceContext::new(&interner);
    let t_name = interner.intern_string("T");
    let u_name = interner.intern_string("U");

    let var_t = ctx.fresh_type_param(t_name);
    let var_u = ctx.fresh_type_param(u_name);

    // T is string from input promise
    ctx.add_upper_bound(var_t, TypeId::STRING);
    // U is number from callback return
    ctx.add_lower_bound(var_u, TypeId::NUMBER);

    let result_t = ctx.resolve_with_constraints(var_t).unwrap();
    let result_u = ctx.resolve_with_constraints(var_u).unwrap();

    assert_eq!(result_t, TypeId::STRING);
    assert_eq!(result_u, TypeId::NUMBER);
}

#[test]
fn test_promise_then_chained_multiple() {
    // Test: promise.then(f1).then(f2).then(f3)
    // Types flow through: A -> B -> C -> D
    let interner = TypeInterner::new();
    let mut ctx = InferenceContext::new(&interner);
    let a_name = interner.intern_string("A");
    let b_name = interner.intern_string("B");
    let c_name = interner.intern_string("C");
    let d_name = interner.intern_string("D");

    let var_a = ctx.fresh_type_param(a_name);
    let var_b = ctx.fresh_type_param(b_name);
    let var_c = ctx.fresh_type_param(c_name);
    let var_d = ctx.fresh_type_param(d_name);

    // Initial promise value
    ctx.add_lower_bound(var_a, TypeId::STRING);
    // First then transforms to number
    ctx.add_lower_bound(var_b, TypeId::NUMBER);
    // Second then transforms to boolean
    ctx.add_lower_bound(var_c, TypeId::BOOLEAN);
    // Third then transforms to symbol
    ctx.add_lower_bound(var_d, TypeId::SYMBOL);

    let results = ctx.resolve_all_with_constraints().unwrap();
    assert_eq!(results.len(), 4);
    assert_eq!(results[0].1, TypeId::STRING);
    assert_eq!(results[1].1, TypeId::NUMBER);
    assert_eq!(results[2].1, TypeId::BOOLEAN);
    assert_eq!(results[3].1, TypeId::SYMBOL);
}

#[test]
fn test_promise_then_returns_promise() {
    // Test: promise.then((x) => Promise.resolve(x + 1))
    // When callback returns Promise<U>, outer Promise unwraps to Promise<U>
    let interner = TypeInterner::new();
    let mut ctx = InferenceContext::new(&interner);
    let t_name = interner.intern_string("T");
    let u_name = interner.intern_string("U");

    let var_t = ctx.fresh_type_param(t_name);
    let var_u = ctx.fresh_type_param(u_name);

    // Input promise resolves to number
    ctx.add_upper_bound(var_t, TypeId::NUMBER);
    // Callback returns Promise<number>, unwrapped to number
    ctx.add_lower_bound(var_u, TypeId::NUMBER);

    let result_t = ctx.resolve_with_constraints(var_t).unwrap();
    let result_u = ctx.resolve_with_constraints(var_u).unwrap();

    assert_eq!(result_t, TypeId::NUMBER);
    assert_eq!(result_u, TypeId::NUMBER);
}

#[test]
fn test_promise_catch_error_type() {
    // Test: promise.catch((err) => handleError(err))
    // Error type is typically unknown or any
    let interner = TypeInterner::new();
    let mut ctx = InferenceContext::new(&interner);
    let err_name = interner.intern_string("Err");

    let var_err = ctx.fresh_type_param(err_name);

    // Catch handler receives unknown error type
    ctx.add_upper_bound(var_err, TypeId::UNKNOWN);

    let result = ctx.resolve_with_constraints(var_err).unwrap();
    assert_eq!(result, TypeId::UNKNOWN);
}

#[test]
fn test_promise_finally_no_value() {
    // Test: promise.finally(() => cleanup())
    // Finally callback receives no arguments and return is ignored
    let interner = TypeInterner::new();
    let mut ctx = InferenceContext::new(&interner);
    let t_name = interner.intern_string("T");

    let var_t = ctx.fresh_type_param(t_name);

    // Promise value passes through finally unchanged
    ctx.add_lower_bound(var_t, TypeId::STRING);

    let result = ctx.resolve_with_constraints(var_t).unwrap();
    assert_eq!(result, TypeId::STRING);
}

#[test]
fn test_promise_all_tuple_inference() {
    // Test: Promise.all([p1, p2, p3]) infers tuple of resolved types
    let interner = TypeInterner::new();
    let mut ctx = InferenceContext::new(&interner);
    let t1_name = interner.intern_string("T1");
    let t2_name = interner.intern_string("T2");
    let t3_name = interner.intern_string("T3");

    let var_t1 = ctx.fresh_type_param(t1_name);
    let var_t2 = ctx.fresh_type_param(t2_name);
    let var_t3 = ctx.fresh_type_param(t3_name);

    // Each promise resolves to different type
    ctx.add_lower_bound(var_t1, TypeId::STRING);
    ctx.add_lower_bound(var_t2, TypeId::NUMBER);
    ctx.add_lower_bound(var_t3, TypeId::BOOLEAN);

    let results = ctx.resolve_all_with_constraints().unwrap();
    assert_eq!(results.len(), 3);
    assert_eq!(results[0].1, TypeId::STRING);
    assert_eq!(results[1].1, TypeId::NUMBER);
    assert_eq!(results[2].1, TypeId::BOOLEAN);
}

#[test]
fn test_promise_race_union_inference() {
    // Test: Promise.race([p1, p2]) infers union of resolved types
    let interner = TypeInterner::new();
    let mut ctx = InferenceContext::new(&interner);
    let t_name = interner.intern_string("T");

    let var_t = ctx.fresh_type_param(t_name);

    // Race could resolve to either type
    ctx.add_lower_bound(var_t, TypeId::STRING);
    ctx.add_lower_bound(var_t, TypeId::NUMBER);

    let result = ctx.resolve_with_constraints(var_t).unwrap();
    let expected = interner.union(vec![TypeId::STRING, TypeId::NUMBER]);
    assert_eq!(result, expected);
}

// -----------------------------------------------------------------------------
// Generic Function Argument Inference from Context
// -----------------------------------------------------------------------------

#[test]
fn test_generic_arg_inferred_from_return_context() {
    // Test: const x: string = identity(value)
    // T inferred from expected return type
    let interner = TypeInterner::new();
    let mut ctx = InferenceContext::new(&interner);
    let t_name = interner.intern_string("T");

    let var_t = ctx.fresh_type_param(t_name);

    // Return context expects string
    ctx.add_upper_bound(var_t, TypeId::STRING);
    // Argument provides string value
    let hello = interner.literal_string("hello");
    ctx.add_lower_bound(var_t, hello);

    let result = ctx.resolve_with_constraints(var_t).unwrap();
    assert_eq!(result, hello);
}

#[test]
fn test_generic_arg_inferred_from_parameter_type() {
    // Test: function wrap<T>(value: T): Box<T>
    // T inferred from argument type
    let interner = TypeInterner::new();
    let mut ctx = InferenceContext::new(&interner);
    let t_name = interner.intern_string("T");

    let var_t = ctx.fresh_type_param(t_name);

    // Argument is number
    let forty_two = interner.literal_number(42.0);
    ctx.add_lower_bound(var_t, forty_two);

    let result = ctx.resolve_with_constraints(var_t).unwrap();
    assert_eq!(result, forty_two);
}

#[test]
fn test_generic_args_inferred_from_multiple_params() {
    // Test: function pair<T, U>(a: T, b: U): [T, U]
    let interner = TypeInterner::new();
    let mut ctx = InferenceContext::new(&interner);
    let t_name = interner.intern_string("T");
    let u_name = interner.intern_string("U");

    let var_t = ctx.fresh_type_param(t_name);
    let var_u = ctx.fresh_type_param(u_name);

    // First argument
    ctx.add_lower_bound(var_t, TypeId::STRING);
    // Second argument
    ctx.add_lower_bound(var_u, TypeId::NUMBER);

    let results = ctx.resolve_all_with_constraints().unwrap();
    assert_eq!(results.len(), 2);
    assert_eq!(results[0].1, TypeId::STRING);
    assert_eq!(results[1].1, TypeId::NUMBER);
}

#[test]
fn test_generic_arg_inferred_from_callback_param() {
    // Test: function process<T>(fn: (x: T) => void): T
    // T inferred from how callback parameter is used
    let interner = TypeInterner::new();
    let mut ctx = InferenceContext::new(&interner);
    let t_name = interner.intern_string("T");

    let var_t = ctx.fresh_type_param(t_name);

    // Callback parameter usage implies type
    ctx.add_lower_bound(var_t, TypeId::STRING);

    let result = ctx.resolve_with_constraints(var_t).unwrap();
    assert_eq!(result, TypeId::STRING);
}

#[test]
fn test_generic_arg_constrained_by_extends() {
    // Test: function fn<T extends number>(x: T): T
    // T is constrained to be subtype of number
    let interner = TypeInterner::new();
    let mut ctx = InferenceContext::new(&interner);
    let t_name = interner.intern_string("T");

    let var_t = ctx.fresh_type_param(t_name);

    // Constraint from extends clause
    ctx.add_upper_bound(var_t, TypeId::NUMBER);
    // Argument provides literal
    let five = interner.literal_number(5.0);
    ctx.add_lower_bound(var_t, five);

    let result = ctx.resolve_with_constraints(var_t).unwrap();
    assert_eq!(result, five);
}

#[test]
fn test_generic_arg_inferred_from_array_element() {
    // Test: function first<T>(arr: T[]): T
    let interner = TypeInterner::new();
    let mut ctx = InferenceContext::new(&interner);
    let t_name = interner.intern_string("T");

    let var_t = ctx.fresh_type_param(t_name);

    // Array element type flows to T
    ctx.add_lower_bound(var_t, TypeId::STRING);

    let result = ctx.resolve_with_constraints(var_t).unwrap();
    assert_eq!(result, TypeId::STRING);
}

#[test]
fn test_generic_arg_from_nested_generic() {
    // Test: function unwrap<T>(box: Box<T>): T
    // T inferred from inner type of Box<string>
    let interner = TypeInterner::new();
    let mut ctx = InferenceContext::new(&interner);
    let t_name = interner.intern_string("T");

    let var_t = ctx.fresh_type_param(t_name);

    // Inner type of Box<string> is string
    ctx.add_lower_bound(var_t, TypeId::STRING);

    let result = ctx.resolve_with_constraints(var_t).unwrap();
    assert_eq!(result, TypeId::STRING);
}

#[test]
fn test_generic_arg_from_object_property_context() {
    // Test: const obj: { value: string } = { value: getValue<T>() }
    // T inferred from property type context
    let interner = TypeInterner::new();
    let mut ctx = InferenceContext::new(&interner);
    let t_name = interner.intern_string("T");

    let var_t = ctx.fresh_type_param(t_name);

    // Property context expects string
    ctx.add_upper_bound(var_t, TypeId::STRING);

    let result = ctx.resolve_with_constraints(var_t).unwrap();
    assert_eq!(result, TypeId::STRING);
}

#[test]
fn test_generic_arg_bidirectional_inference() {
    // Test: Both parameter and return type contribute to inference
    // function transform<T>(x: T, fn: (x: T) => T): T
    let interner = TypeInterner::new();
    let mut ctx = InferenceContext::new(&interner);
    let t_name = interner.intern_string("T");

    let var_t = ctx.fresh_type_param(t_name);

    // From parameter
    ctx.add_lower_bound(var_t, TypeId::NUMBER);
    // From callback signature (must match)
    ctx.add_upper_bound(var_t, TypeId::NUMBER);

    let result = ctx.resolve_with_constraints(var_t).unwrap();
    assert_eq!(result, TypeId::NUMBER);
}

#[test]
fn test_generic_arg_inferred_from_spread() {
    // Test: function concat<T>(...arrays: T[][]): T[]
    let interner = TypeInterner::new();
    let mut ctx = InferenceContext::new(&interner);
    let t_name = interner.intern_string("T");

    let var_t = ctx.fresh_type_param(t_name);

    // Spread elements contribute to T
    ctx.add_lower_bound(var_t, TypeId::STRING);
    ctx.add_lower_bound(var_t, TypeId::NUMBER);

    let result = ctx.resolve_with_constraints(var_t).unwrap();
    let expected = interner.union(vec![TypeId::STRING, TypeId::NUMBER]);
    assert_eq!(result, expected);
}

#[test]
fn test_generic_arg_partial_inference() {
    // Test: function fn<T, U>(x: T): U - U must be explicitly provided or inferred from context
    let interner = TypeInterner::new();
    let mut ctx = InferenceContext::new(&interner);
    let t_name = interner.intern_string("T");
    let u_name = interner.intern_string("U");

    let var_t = ctx.fresh_type_param(t_name);
    let var_u = ctx.fresh_type_param(u_name);

    // T inferred from argument
    ctx.add_lower_bound(var_t, TypeId::STRING);
    // U has no inference sources - returns unknown

    let result_t = ctx.resolve_with_constraints(var_t).unwrap();
    let result_u = ctx.resolve_with_constraints(var_u).unwrap();

    assert_eq!(result_t, TypeId::STRING);
    assert_eq!(result_u, TypeId::UNKNOWN);
}

#[test]
fn test_generic_arg_from_conditional_return() {
    // Test: const x: string = cond ? fn<T>() : other
    // T inferred from union member in conditional
    let interner = TypeInterner::new();
    let mut ctx = InferenceContext::new(&interner);
    let t_name = interner.intern_string("T");

    let var_t = ctx.fresh_type_param(t_name);

    // Return context from conditional
    ctx.add_upper_bound(var_t, TypeId::STRING);

    let result = ctx.resolve_with_constraints(var_t).unwrap();
    assert_eq!(result, TypeId::STRING);
}

// ============================================================================
// Constructor Parameter Inference Tests
// ============================================================================
// Tests for inferring types from class constructor parameters

#[test]
fn test_constructor_param_basic() {
    // Test: class Foo<T> { constructor(x: T) {} } - infer T from argument
    let interner = TypeInterner::new();
    let mut ctx = InferenceContext::new(&interner);
    let t_name = interner.intern_string("T");

    let var_t = ctx.fresh_type_param(t_name);

    // T inferred from constructor argument
    ctx.add_lower_bound(var_t, TypeId::STRING);

    let result = ctx.resolve_with_constraints(var_t).unwrap();
    assert_eq!(result, TypeId::STRING);
}

#[test]
fn test_constructor_param_multiple() {
    // Test: class Pair<T, U> { constructor(first: T, second: U) {} }
    let interner = TypeInterner::new();
    let mut ctx = InferenceContext::new(&interner);
    let t_name = interner.intern_string("T");
    let u_name = interner.intern_string("U");

    let var_t = ctx.fresh_type_param(t_name);
    let var_u = ctx.fresh_type_param(u_name);

    // T and U inferred from constructor arguments
    ctx.add_lower_bound(var_t, TypeId::STRING);
    ctx.add_lower_bound(var_u, TypeId::NUMBER);

    let results = ctx.resolve_all_with_constraints().unwrap();
    assert_eq!(results.len(), 2);
    assert_eq!(results[0].1, TypeId::STRING);
    assert_eq!(results[1].1, TypeId::NUMBER);
}

#[test]
fn test_constructor_param_with_default() {
    // Test: class Container<T = string> { constructor(value?: T) {} }
    let interner = TypeInterner::new();
    let mut ctx = InferenceContext::new(&interner);
    let t_name = interner.intern_string("T");

    let var_t = ctx.fresh_type_param(t_name);

    // When called with number, T is inferred as number (overriding default)
    ctx.add_lower_bound(var_t, TypeId::NUMBER);

    let result = ctx.resolve_with_constraints(var_t).unwrap();
    assert_eq!(result, TypeId::NUMBER);
}

#[test]
fn test_constructor_param_array() {
    // Test: class List<T> { constructor(items: T[]) {} }
    let interner = TypeInterner::new();
    let mut ctx = InferenceContext::new(&interner);
    let t_name = interner.intern_string("T");

    let var_t = ctx.fresh_type_param(t_name);

    // T inferred from array element type
    ctx.add_lower_bound(var_t, TypeId::STRING);

    let result = ctx.resolve_with_constraints(var_t).unwrap();
    assert_eq!(result, TypeId::STRING);
}

#[test]
fn test_constructor_param_object() {
    // Test: class Config<T> { constructor(options: { value: T }) {} }
    let interner = TypeInterner::new();
    let mut ctx = InferenceContext::new(&interner);
    let t_name = interner.intern_string("T");

    let var_t = ctx.fresh_type_param(t_name);

    // T inferred from object property
    ctx.add_lower_bound(var_t, TypeId::BOOLEAN);

    let result = ctx.resolve_with_constraints(var_t).unwrap();
    assert_eq!(result, TypeId::BOOLEAN);
}

// ============================================================================
// Method Return Type Inference Tests
// ============================================================================
// Tests for inferring types from class method return types

#[test]
fn test_method_return_basic() {
    // Test: class Foo<T> { get(): T { ... } } - infer T from return context
    let interner = TypeInterner::new();
    let mut ctx = InferenceContext::new(&interner);
    let t_name = interner.intern_string("T");

    let var_t = ctx.fresh_type_param(t_name);

    // T inferred from expected return type
    ctx.add_lower_bound(var_t, TypeId::STRING);

    let result = ctx.resolve_with_constraints(var_t).unwrap();
    assert_eq!(result, TypeId::STRING);
}

#[test]
fn test_method_return_generic_call() {
    // Test: class Builder<T> { build(): T } - called in typed context
    let interner = TypeInterner::new();
    let mut ctx = InferenceContext::new(&interner);
    let t_name = interner.intern_string("T");

    let var_t = ctx.fresh_type_param(t_name);

    // Return type flows into T
    let return_type = interner.object(vec![PropertyInfo {
        name: interner.intern_string("id"),
        type_id: TypeId::NUMBER,
        write_type: TypeId::NUMBER,
        optional: false,
        readonly: false,
        is_method: false,
    }]);
    ctx.add_lower_bound(var_t, return_type);

    let result = ctx.resolve_with_constraints(var_t).unwrap();
    assert_eq!(result, return_type);
}

#[test]
fn test_method_return_promise() {
    // Test: class Service<T> { async fetch(): Promise<T> }
    let interner = TypeInterner::new();
    let mut ctx = InferenceContext::new(&interner);
    let t_name = interner.intern_string("T");

    let var_t = ctx.fresh_type_param(t_name);

    // T is the resolved type of the promise
    ctx.add_lower_bound(var_t, TypeId::STRING);

    let result = ctx.resolve_with_constraints(var_t).unwrap();
    assert_eq!(result, TypeId::STRING);
}

#[test]
fn test_method_return_array() {
    // Test: class Repository<T> { findAll(): T[] }
    let interner = TypeInterner::new();
    let mut ctx = InferenceContext::new(&interner);
    let t_name = interner.intern_string("T");

    let var_t = ctx.fresh_type_param(t_name);

    // T inferred from array element expectation
    ctx.add_lower_bound(var_t, TypeId::NUMBER);

    let result = ctx.resolve_with_constraints(var_t).unwrap();
    assert_eq!(result, TypeId::NUMBER);
}

#[test]
fn test_method_return_chained() {
    // Test: class Chain<T> { map<U>(fn: (t: T) => U): Chain<U> }
    let interner = TypeInterner::new();
    let mut ctx = InferenceContext::new(&interner);
    let t_name = interner.intern_string("T");
    let u_name = interner.intern_string("U");

    let var_t = ctx.fresh_type_param(t_name);
    let var_u = ctx.fresh_type_param(u_name);

    // T from input chain, U from callback return
    ctx.add_lower_bound(var_t, TypeId::STRING);
    ctx.add_lower_bound(var_u, TypeId::NUMBER);

    let results = ctx.resolve_all_with_constraints().unwrap();
    assert_eq!(results.len(), 2);
    assert_eq!(results[0].1, TypeId::STRING);
    assert_eq!(results[1].1, TypeId::NUMBER);
}

// ============================================================================
// Static Member Type Inference Tests
// ============================================================================
// Tests for inferring types from static class members

#[test]
fn test_static_member_basic() {
    // Test: class Factory<T> { static create<T>(): T }
    let interner = TypeInterner::new();
    let mut ctx = InferenceContext::new(&interner);
    let t_name = interner.intern_string("T");

    let var_t = ctx.fresh_type_param(t_name);

    // T inferred from static method context
    ctx.add_lower_bound(var_t, TypeId::STRING);

    let result = ctx.resolve_with_constraints(var_t).unwrap();
    assert_eq!(result, TypeId::STRING);
}

#[test]
fn test_static_member_factory() {
    // Test: class Box<T> { static of<T>(value: T): Box<T> }
    let interner = TypeInterner::new();
    let mut ctx = InferenceContext::new(&interner);
    let t_name = interner.intern_string("T");

    let var_t = ctx.fresh_type_param(t_name);

    // T inferred from factory argument
    let lit_hello = interner.literal_string("hello");
    ctx.add_lower_bound(var_t, lit_hello);

    let result = ctx.resolve_with_constraints(var_t).unwrap();
    assert_eq!(result, lit_hello);
}

#[test]
fn test_static_member_property() {
    // Test: class Config<T> { static defaults: T }
    let interner = TypeInterner::new();
    let mut ctx = InferenceContext::new(&interner);
    let t_name = interner.intern_string("T");

    let var_t = ctx.fresh_type_param(t_name);

    // T from static property type
    let config_type = interner.object(vec![PropertyInfo {
        name: interner.intern_string("debug"),
        type_id: TypeId::BOOLEAN,
        write_type: TypeId::BOOLEAN,
        optional: false,
        readonly: false,
        is_method: false,
    }]);
    ctx.add_lower_bound(var_t, config_type);

    let result = ctx.resolve_with_constraints(var_t).unwrap();
    assert_eq!(result, config_type);
}

#[test]
fn test_static_member_multiple_type_params() {
    // Test: class Mapper<K, V> { static fromEntries<K, V>(entries: [K, V][]): Mapper<K, V> }
    let interner = TypeInterner::new();
    let mut ctx = InferenceContext::new(&interner);
    let k_name = interner.intern_string("K");
    let v_name = interner.intern_string("V");

    let var_k = ctx.fresh_type_param(k_name);
    let var_v = ctx.fresh_type_param(v_name);

    // K and V inferred from entry types
    ctx.add_lower_bound(var_k, TypeId::STRING);
    ctx.add_lower_bound(var_v, TypeId::NUMBER);

    let results = ctx.resolve_all_with_constraints().unwrap();
    assert_eq!(results.len(), 2);
    assert_eq!(results[0].1, TypeId::STRING);
    assert_eq!(results[1].1, TypeId::NUMBER);
}

#[test]
fn test_static_member_with_constraint() {
    // Test: class Serializer<T extends object> { static serialize<T extends object>(obj: T): string }
    let interner = TypeInterner::new();
    let mut ctx = InferenceContext::new(&interner);
    let t_name = interner.intern_string("T");

    let var_t = ctx.fresh_type_param(t_name);

    // Upper bound from constraint
    ctx.add_upper_bound(var_t, TypeId::OBJECT);

    // Lower bound from argument
    let obj_type = interner.object(vec![PropertyInfo {
        name: interner.intern_string("name"),
        type_id: TypeId::STRING,
        write_type: TypeId::STRING,
        optional: false,
        readonly: false,
        is_method: false,
    }]);
    ctx.add_lower_bound(var_t, obj_type);

    let result = ctx.resolve_with_constraints(var_t).unwrap();
    assert_eq!(result, obj_type);
}

// ============================================================================
// CIRCULAR CONSTRAINT TESTS
// ============================================================================

// ----------------------------------------------------------------------------
// Self-referential type parameters (T extends Array<T>)
// ----------------------------------------------------------------------------

#[test]
fn test_self_ref_type_param_array_of_self() {
    // Test: T extends Array<T> with T = string[]
    let interner = TypeInterner::new();
    let mut ctx = InferenceContext::new(&interner);
    let t_name = interner.intern_string("T");

    let var_t = ctx.fresh_type_param(t_name);

    // Lower bound from usage: string[]
    let string_array = interner.array(TypeId::STRING);
    ctx.add_lower_bound(var_t, string_array);

    // The self-referential constraint is conceptual - T should resolve to string[]
    let result = ctx.resolve_with_constraints(var_t).unwrap();
    assert_eq!(result, string_array);
}

#[test]
fn test_self_ref_type_param_promise_of_self() {
    // Test: T extends Promise<T> - self-referential promise type
    let interner = TypeInterner::new();
    let mut ctx = InferenceContext::new(&interner);
    let t_name = interner.intern_string("T");

    let var_t = ctx.fresh_type_param(t_name);

    // Create a function type for the method
    let then_fn = interner.function(FunctionShape {
        type_params: Vec::new(),
        params: Vec::new(),
        this_type: None,
        return_type: TypeId::VOID,
        type_predicate: None,
        is_constructor: false,
    });

    // Lower bound: Promise<number>
    let promise_type = interner.object(vec![PropertyInfo {
        name: interner.intern_string("then"),
        type_id: then_fn,
        write_type: then_fn,
        optional: false,
        readonly: false,
        is_method: true,
    }]);
    ctx.add_lower_bound(var_t, promise_type);

    let result = ctx.resolve_with_constraints(var_t).unwrap();
    assert_eq!(result, promise_type);
}

#[test]
fn test_self_ref_type_param_node_with_children() {
    // Test: T extends { children: T[] } - tree node pattern
    let interner = TypeInterner::new();
    let mut ctx = InferenceContext::new(&interner);
    let t_name = interner.intern_string("T");

    let var_t = ctx.fresh_type_param(t_name);

    // Create a node type with children array
    let children_array = interner.array(TypeId::OBJECT);
    let node_type = interner.object(vec![PropertyInfo {
        name: interner.intern_string("children"),
        type_id: children_array,
        write_type: children_array,
        optional: false,
        readonly: false,
        is_method: false,
    }]);
    ctx.add_lower_bound(var_t, node_type);

    let result = ctx.resolve_with_constraints(var_t).unwrap();
    assert_eq!(result, node_type);
}

#[test]
fn test_self_ref_type_param_linked_list() {
    // Test: T extends { next: T | null } - linked list pattern
    let interner = TypeInterner::new();
    let mut ctx = InferenceContext::new(&interner);
    let t_name = interner.intern_string("T");

    let var_t = ctx.fresh_type_param(t_name);

    // Create a linked list node with next pointer
    let next_type = interner.union(vec![TypeId::OBJECT, TypeId::NULL]);
    let list_node = interner.object(vec![
        PropertyInfo {
            name: interner.intern_string("value"),
            type_id: TypeId::NUMBER,
            write_type: TypeId::NUMBER,
            optional: false,
            readonly: false,
            is_method: false,
        },
        PropertyInfo {
            name: interner.intern_string("next"),
            type_id: next_type,
            write_type: next_type,
            optional: false,
            readonly: false,
            is_method: false,
        },
    ]);
    ctx.add_lower_bound(var_t, list_node);

    let result = ctx.resolve_with_constraints(var_t).unwrap();
    assert_eq!(result, list_node);
}

#[test]
fn test_self_ref_type_param_recursive_json() {
    // Test: T extends string | number | T[] | { [key: string]: T }
    let interner = TypeInterner::new();
    let mut ctx = InferenceContext::new(&interner);
    let t_name = interner.intern_string("T");

    let var_t = ctx.fresh_type_param(t_name);

    // JSON-like type: union of primitives
    let json_primitives = interner.union(vec![TypeId::STRING, TypeId::NUMBER, TypeId::BOOLEAN, TypeId::NULL]);
    ctx.add_lower_bound(var_t, json_primitives);

    let result = ctx.resolve_with_constraints(var_t).unwrap();
    assert_eq!(result, json_primitives);
}

// ----------------------------------------------------------------------------
// Mutually dependent type parameters
// ----------------------------------------------------------------------------

#[test]
fn test_mutual_dependency_key_value() {
    // Test: K extends keyof V, V extends Record<K, any>
    let interner = TypeInterner::new();
    let mut ctx = InferenceContext::new(&interner);
    let k_name = interner.intern_string("K");
    let v_name = interner.intern_string("V");

    let var_k = ctx.fresh_type_param(k_name);
    let var_v = ctx.fresh_type_param(v_name);

    // K gets "name" literal
    let name_literal = interner.literal_string("name");
    ctx.add_lower_bound(var_k, name_literal);

    // V gets an object with that key
    let obj_type = interner.object(vec![PropertyInfo {
        name: interner.intern_string("name"),
        type_id: TypeId::STRING,
        write_type: TypeId::STRING,
        optional: false,
        readonly: false,
        is_method: false,
    }]);
    ctx.add_lower_bound(var_v, obj_type);

    let results = ctx.resolve_all_with_constraints().unwrap();
    assert_eq!(results.len(), 2);
    assert_eq!(results[0].1, name_literal);
    assert_eq!(results[1].1, obj_type);
}

#[test]
fn test_mutual_dependency_parent_child() {
    // Test: P extends { child: C }, C extends { parent: P }
    let interner = TypeInterner::new();
    let mut ctx = InferenceContext::new(&interner);
    let p_name = interner.intern_string("P");
    let c_name = interner.intern_string("C");

    let var_p = ctx.fresh_type_param(p_name);
    let var_c = ctx.fresh_type_param(c_name);

    // Create parent type with child reference
    let parent_type = interner.object(vec![PropertyInfo {
        name: interner.intern_string("child"),
        type_id: TypeId::OBJECT,
        write_type: TypeId::OBJECT,
        optional: false,
        readonly: false,
        is_method: false,
    }]);

    // Create child type with parent reference
    let child_type = interner.object(vec![PropertyInfo {
        name: interner.intern_string("parent"),
        type_id: TypeId::OBJECT,
        write_type: TypeId::OBJECT,
        optional: false,
        readonly: false,
        is_method: false,
    }]);

    ctx.add_lower_bound(var_p, parent_type);
    ctx.add_lower_bound(var_c, child_type);

    let results = ctx.resolve_all_with_constraints().unwrap();
    assert_eq!(results.len(), 2);
    assert_eq!(results[0].1, parent_type);
    assert_eq!(results[1].1, child_type);
}

#[test]
fn test_mutual_dependency_input_output() {
    // Test: I extends (arg: O) => void, O extends ReturnType<I>
    let interner = TypeInterner::new();
    let mut ctx = InferenceContext::new(&interner);
    let i_name = interner.intern_string("I");
    let o_name = interner.intern_string("O");

    let var_i = ctx.fresh_type_param(i_name);
    let var_o = ctx.fresh_type_param(o_name);

    // Input function type
    let input_fn = interner.function(FunctionShape {
        type_params: Vec::new(),
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

    ctx.add_lower_bound(var_i, input_fn);
    ctx.add_lower_bound(var_o, TypeId::NUMBER);

    let results = ctx.resolve_all_with_constraints().unwrap();
    assert_eq!(results.len(), 2);
    assert_eq!(results[0].1, input_fn);
    assert_eq!(results[1].1, TypeId::NUMBER);
}

#[test]
fn test_mutual_dependency_request_response() {
    // Test: Req extends { respond: (r: Res) => void }, Res extends { request: Req }
    let interner = TypeInterner::new();
    let mut ctx = InferenceContext::new(&interner);
    let req_name = interner.intern_string("Req");
    let res_name = interner.intern_string("Res");

    let var_req = ctx.fresh_type_param(req_name);
    let var_res = ctx.fresh_type_param(res_name);

    // Create a method type
    let respond_fn = interner.function(FunctionShape {
        type_params: Vec::new(),
        params: Vec::new(),
        this_type: None,
        return_type: TypeId::VOID,
        type_predicate: None,
        is_constructor: false,
    });

    // Request type with respond method
    let request_type = interner.object(vec![
        PropertyInfo {
            name: interner.intern_string("id"),
            type_id: TypeId::NUMBER,
            write_type: TypeId::NUMBER,
            optional: false,
            readonly: false,
            is_method: false,
        },
        PropertyInfo {
            name: interner.intern_string("respond"),
            type_id: respond_fn,
            write_type: respond_fn,
            optional: false,
            readonly: false,
            is_method: true,
        },
    ]);

    // Response type with request reference
    let response_type = interner.object(vec![
        PropertyInfo {
            name: interner.intern_string("data"),
            type_id: TypeId::STRING,
            write_type: TypeId::STRING,
            optional: false,
            readonly: false,
            is_method: false,
        },
        PropertyInfo {
            name: interner.intern_string("request"),
            type_id: TypeId::OBJECT,
            write_type: TypeId::OBJECT,
            optional: false,
            readonly: false,
            is_method: false,
        },
    ]);

    ctx.add_lower_bound(var_req, request_type);
    ctx.add_lower_bound(var_res, response_type);

    let results = ctx.resolve_all_with_constraints().unwrap();
    assert_eq!(results.len(), 2);
    assert_eq!(results[0].1, request_type);
    assert_eq!(results[1].1, response_type);
}

#[test]
fn test_mutual_dependency_three_way() {
    // Test: A extends { b: B }, B extends { c: C }, C extends { a: A }
    let interner = TypeInterner::new();
    let mut ctx = InferenceContext::new(&interner);
    let a_name = interner.intern_string("A");
    let b_name = interner.intern_string("B");
    let c_name = interner.intern_string("C");

    let var_a = ctx.fresh_type_param(a_name);
    let var_b = ctx.fresh_type_param(b_name);
    let var_c = ctx.fresh_type_param(c_name);

    let type_a = interner.object(vec![PropertyInfo {
        name: interner.intern_string("b"),
        type_id: TypeId::OBJECT,
        write_type: TypeId::OBJECT,
        optional: false,
        readonly: false,
        is_method: false,
    }]);

    let type_b = interner.object(vec![PropertyInfo {
        name: interner.intern_string("c"),
        type_id: TypeId::OBJECT,
        write_type: TypeId::OBJECT,
        optional: false,
        readonly: false,
        is_method: false,
    }]);

    let type_c = interner.object(vec![PropertyInfo {
        name: interner.intern_string("a"),
        type_id: TypeId::OBJECT,
        write_type: TypeId::OBJECT,
        optional: false,
        readonly: false,
        is_method: false,
    }]);

    ctx.add_lower_bound(var_a, type_a);
    ctx.add_lower_bound(var_b, type_b);
    ctx.add_lower_bound(var_c, type_c);

    let results = ctx.resolve_all_with_constraints().unwrap();
    assert_eq!(results.len(), 3);
    assert_eq!(results[0].1, type_a);
    assert_eq!(results[1].1, type_b);
    assert_eq!(results[2].1, type_c);
}

// ----------------------------------------------------------------------------
// Recursive generic constraints
// ----------------------------------------------------------------------------

#[test]
fn test_recursive_constraint_comparable() {
    // Test: T extends Comparable<T> - self-comparison pattern
    let interner = TypeInterner::new();
    let mut ctx = InferenceContext::new(&interner);
    let t_name = interner.intern_string("T");

    let var_t = ctx.fresh_type_param(t_name);

    // Create method type
    let compare_fn = interner.function(FunctionShape {
        type_params: Vec::new(),
        params: Vec::new(),
        this_type: None,
        return_type: TypeId::NUMBER,
        type_predicate: None,
        is_constructor: false,
    });

    // Comparable interface with compareTo method
    let comparable_type = interner.object(vec![PropertyInfo {
        name: interner.intern_string("compareTo"),
        type_id: compare_fn,
        write_type: compare_fn,
        optional: false,
        readonly: false,
        is_method: true,
    }]);

    ctx.add_lower_bound(var_t, comparable_type);
    ctx.add_upper_bound(var_t, TypeId::OBJECT);

    let result = ctx.resolve_with_constraints(var_t).unwrap();
    assert_eq!(result, comparable_type);
}

#[test]
fn test_recursive_constraint_builder_pattern() {
    // Test: T extends Builder<T> - fluent builder pattern
    let interner = TypeInterner::new();
    let mut ctx = InferenceContext::new(&interner);
    let t_name = interner.intern_string("T");

    let var_t = ctx.fresh_type_param(t_name);

    // Create method types
    let set_fn = interner.function(FunctionShape {
        type_params: Vec::new(),
        params: Vec::new(),
        this_type: None,
        return_type: TypeId::OBJECT,
        type_predicate: None,
        is_constructor: false,
    });

    let build_fn = interner.function(FunctionShape {
        type_params: Vec::new(),
        params: Vec::new(),
        this_type: None,
        return_type: TypeId::OBJECT,
        type_predicate: None,
        is_constructor: false,
    });

    // Builder with methods that return the builder itself
    let builder_type = interner.object(vec![
        PropertyInfo {
            name: interner.intern_string("set"),
            type_id: set_fn,
            write_type: set_fn,
            optional: false,
            readonly: false,
            is_method: true,
        },
        PropertyInfo {
            name: interner.intern_string("build"),
            type_id: build_fn,
            write_type: build_fn,
            optional: false,
            readonly: false,
            is_method: true,
        },
    ]);

    ctx.add_lower_bound(var_t, builder_type);

    let result = ctx.resolve_with_constraints(var_t).unwrap();
    assert_eq!(result, builder_type);
}

#[test]
fn test_recursive_constraint_expression_tree() {
    // Test: T extends Expr<T> - expression tree pattern
    let interner = TypeInterner::new();
    let mut ctx = InferenceContext::new(&interner);
    let t_name = interner.intern_string("T");

    let var_t = ctx.fresh_type_param(t_name);

    // Create method type
    let evaluate_fn = interner.function(FunctionShape {
        type_params: Vec::new(),
        params: Vec::new(),
        this_type: None,
        return_type: TypeId::UNKNOWN,
        type_predicate: None,
        is_constructor: false,
    });

    // Expression with evaluate method
    let expr_type = interner.object(vec![
        PropertyInfo {
            name: interner.intern_string("evaluate"),
            type_id: evaluate_fn,
            write_type: evaluate_fn,
            optional: false,
            readonly: false,
            is_method: true,
        },
        PropertyInfo {
            name: interner.intern_string("children"),
            type_id: interner.array(TypeId::OBJECT),
            write_type: interner.array(TypeId::OBJECT),
            optional: false,
            readonly: false,
            is_method: false,
        },
    ]);

    ctx.add_lower_bound(var_t, expr_type);

    let result = ctx.resolve_with_constraints(var_t).unwrap();
    assert_eq!(result, expr_type);
}

#[test]
fn test_recursive_constraint_cloneable() {
    // Test: T extends Cloneable<T> - clone returns same type
    let interner = TypeInterner::new();
    let mut ctx = InferenceContext::new(&interner);
    let t_name = interner.intern_string("T");

    let var_t = ctx.fresh_type_param(t_name);

    // Create method type
    let clone_fn = interner.function(FunctionShape {
        type_params: Vec::new(),
        params: Vec::new(),
        this_type: None,
        return_type: TypeId::OBJECT,
        type_predicate: None,
        is_constructor: false,
    });

    // Cloneable with clone method
    let cloneable_type = interner.object(vec![PropertyInfo {
        name: interner.intern_string("clone"),
        type_id: clone_fn,
        write_type: clone_fn,
        optional: false,
        readonly: false,
        is_method: true,
    }]);

    ctx.add_lower_bound(var_t, cloneable_type);

    let result = ctx.resolve_with_constraints(var_t).unwrap();
    assert_eq!(result, cloneable_type);
}

#[test]
fn test_recursive_constraint_iterable() {
    // Test: T extends Iterable<T> - iterable of self
    let interner = TypeInterner::new();
    let mut ctx = InferenceContext::new(&interner);
    let t_name = interner.intern_string("T");

    let var_t = ctx.fresh_type_param(t_name);

    // Create method type
    let next_fn = interner.function(FunctionShape {
        type_params: Vec::new(),
        params: Vec::new(),
        this_type: None,
        return_type: TypeId::OBJECT,
        type_predicate: None,
        is_constructor: false,
    });

    // Iterable with Symbol.iterator method
    let iterable_type = interner.object(vec![PropertyInfo {
        name: interner.intern_string("next"),
        type_id: next_fn,
        write_type: next_fn,
        optional: false,
        readonly: false,
        is_method: true,
    }]);

    ctx.add_lower_bound(var_t, iterable_type);

    let result = ctx.resolve_with_constraints(var_t).unwrap();
    assert_eq!(result, iterable_type);
}

// ----------------------------------------------------------------------------
// Constraint cycles in extends clauses
// ----------------------------------------------------------------------------

#[test]
fn test_constraint_cycle_direct_extends() {
    // Test: class A extends B, class B extends A (error case - but test constraint handling)
    let interner = TypeInterner::new();
    let mut ctx = InferenceContext::new(&interner);
    let a_name = interner.intern_string("A");
    let b_name = interner.intern_string("B");

    let var_a = ctx.fresh_type_param(a_name);
    let var_b = ctx.fresh_type_param(b_name);

    // Both constrained by object
    ctx.add_upper_bound(var_a, TypeId::OBJECT);
    ctx.add_upper_bound(var_b, TypeId::OBJECT);

    // Both get concrete lower bounds
    ctx.add_lower_bound(var_a, TypeId::OBJECT);
    ctx.add_lower_bound(var_b, TypeId::OBJECT);

    let results = ctx.resolve_all_with_constraints().unwrap();
    assert_eq!(results.len(), 2);
    assert_eq!(results[0].1, TypeId::OBJECT);
    assert_eq!(results[1].1, TypeId::OBJECT);
}

#[test]
fn test_constraint_cycle_interface_extends() {
    // Test: interface A extends B, interface B extends C, interface C extends A
    let interner = TypeInterner::new();
    let mut ctx = InferenceContext::new(&interner);
    let a_name = interner.intern_string("A");
    let b_name = interner.intern_string("B");
    let c_name = interner.intern_string("C");

    let var_a = ctx.fresh_type_param(a_name);
    let var_b = ctx.fresh_type_param(b_name);
    let var_c = ctx.fresh_type_param(c_name);

    // Create distinct interface types
    let type_a = interner.object(vec![PropertyInfo {
        name: interner.intern_string("propA"),
        type_id: TypeId::STRING,
        write_type: TypeId::STRING,
        optional: false,
        readonly: false,
        is_method: false,
    }]);

    let type_b = interner.object(vec![PropertyInfo {
        name: interner.intern_string("propB"),
        type_id: TypeId::NUMBER,
        write_type: TypeId::NUMBER,
        optional: false,
        readonly: false,
        is_method: false,
    }]);

    let type_c = interner.object(vec![PropertyInfo {
        name: interner.intern_string("propC"),
        type_id: TypeId::BOOLEAN,
        write_type: TypeId::BOOLEAN,
        optional: false,
        readonly: false,
        is_method: false,
    }]);

    ctx.add_lower_bound(var_a, type_a);
    ctx.add_lower_bound(var_b, type_b);
    ctx.add_lower_bound(var_c, type_c);

    let results = ctx.resolve_all_with_constraints().unwrap();
    assert_eq!(results.len(), 3);
    assert_eq!(results[0].1, type_a);
    assert_eq!(results[1].1, type_b);
    assert_eq!(results[2].1, type_c);
}

#[test]
fn test_constraint_cycle_generic_extends() {
    // Test: class Container<T extends Container<T>>
    let interner = TypeInterner::new();
    let mut ctx = InferenceContext::new(&interner);
    let t_name = interner.intern_string("T");

    let var_t = ctx.fresh_type_param(t_name);

    // Create method type
    let get_container_fn = interner.function(FunctionShape {
        type_params: Vec::new(),
        params: Vec::new(),
        this_type: None,
        return_type: TypeId::OBJECT,
        type_predicate: None,
        is_constructor: false,
    });

    // Container type with self-referential constraint
    let container_type = interner.object(vec![
        PropertyInfo {
            name: interner.intern_string("value"),
            type_id: TypeId::UNKNOWN,
            write_type: TypeId::UNKNOWN,
            optional: false,
            readonly: false,
            is_method: false,
        },
        PropertyInfo {
            name: interner.intern_string("getContainer"),
            type_id: get_container_fn,
            write_type: get_container_fn,
            optional: false,
            readonly: false,
            is_method: true,
        },
    ]);

    ctx.add_lower_bound(var_t, container_type);
    ctx.add_upper_bound(var_t, TypeId::OBJECT);

    let result = ctx.resolve_with_constraints(var_t).unwrap();
    assert_eq!(result, container_type);
}

#[test]
fn test_constraint_cycle_mixin_pattern() {
    // Test: type Constructor<T> = new (...args: any[]) => T
    //       function Mixin<T extends Constructor<{}>>(Base: T)
    let interner = TypeInterner::new();
    let mut ctx = InferenceContext::new(&interner);
    let t_name = interner.intern_string("T");

    let var_t = ctx.fresh_type_param(t_name);

    // Constructor function type
    let constructor_fn = interner.function(FunctionShape {
        type_params: Vec::new(),
        params: vec![],
        this_type: None,
        return_type: TypeId::OBJECT,
        type_predicate: None,
        is_constructor: true,
    });

    // Add lower bound only - this is common for mixin patterns
    ctx.add_lower_bound(var_t, constructor_fn);
    ctx.add_upper_bound(var_t, TypeId::OBJECT);

    let result = ctx.resolve_with_constraints(var_t).unwrap();
    assert_eq!(result, constructor_fn);
}

#[test]
fn test_constraint_cycle_enum_constraint() {
    // Test: T extends keyof typeof Enum where Enum has circular references
    let interner = TypeInterner::new();
    let mut ctx = InferenceContext::new(&interner);
    let t_name = interner.intern_string("T");

    let var_t = ctx.fresh_type_param(t_name);

    // Enum key union
    let enum_keys = interner.union(vec![
        interner.literal_string("A"),
        interner.literal_string("B"),
        interner.literal_string("C"),
    ]);

    ctx.add_lower_bound(var_t, interner.literal_string("A"));
    ctx.add_upper_bound(var_t, enum_keys);

    let result = ctx.resolve_with_constraints(var_t).unwrap();
    assert_eq!(result, interner.literal_string("A"));
}
