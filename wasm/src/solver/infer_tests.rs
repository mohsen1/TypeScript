use super::*;

#[test]
fn test_inference_basic() {
    let interner = TypeInterner::new();
    let mut ctx = InferenceContext::new(&interner);

    // Create inference variable
    let var = ctx.fresh_var();

    // Should start unresolved
    assert!(ctx.probe(var).is_none());

    // Unify with string
    ctx.unify_var_type(var, TypeId::STRING).unwrap();

    // Should now be string
    assert_eq!(ctx.probe(var), Some(TypeId::STRING));
}

#[test]
fn test_inference_type_param() {
    let interner = TypeInterner::new();
    let mut ctx = InferenceContext::new(&interner);

    // Create type parameter T
    let var_t = ctx.fresh_type_param(Arc::from("T"));

    // Look it up
    let found = ctx.find_type_param("T");
    assert_eq!(found, Some(var_t));

    // Not found
    let not_found = ctx.find_type_param("U");
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
