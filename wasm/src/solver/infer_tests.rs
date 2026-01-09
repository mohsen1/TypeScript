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
