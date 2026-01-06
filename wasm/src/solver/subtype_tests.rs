use super::*;

#[test]
fn test_intrinsic_subtyping() {
    let interner = TypeInterner::new();
    let mut checker = SubtypeChecker::new(&interner);

    // Same type
    assert!(checker.is_subtype_of(TypeId::STRING, TypeId::STRING));
    assert!(checker.is_subtype_of(TypeId::NUMBER, TypeId::NUMBER));

    // Different intrinsics
    assert!(!checker.is_subtype_of(TypeId::STRING, TypeId::NUMBER));

    // Any relations
    assert!(checker.is_subtype_of(TypeId::ANY, TypeId::STRING));
    assert!(checker.is_subtype_of(TypeId::STRING, TypeId::ANY));

    // Unknown relations
    assert!(checker.is_subtype_of(TypeId::STRING, TypeId::UNKNOWN));
    assert!(!checker.is_subtype_of(TypeId::UNKNOWN, TypeId::STRING));

    // Never relations
    assert!(checker.is_subtype_of(TypeId::NEVER, TypeId::STRING));
    assert!(!checker.is_subtype_of(TypeId::STRING, TypeId::NEVER));
}

#[test]
fn test_literal_subtyping() {
    let interner = TypeInterner::new();
    let mut checker = SubtypeChecker::new(&interner);

    let hello = interner.literal_string("hello");
    let world = interner.literal_string("world");

    // Literal to same literal
    assert!(checker.is_subtype_of(hello, hello));

    // Literal to different literal
    assert!(!checker.is_subtype_of(hello, world));

    // Literal to intrinsic
    assert!(checker.is_subtype_of(hello, TypeId::STRING));
    assert!(!checker.is_subtype_of(hello, TypeId::NUMBER));
}

#[test]
fn test_union_subtyping() {
    let interner = TypeInterner::new();
    let mut checker = SubtypeChecker::new(&interner);

    let string_or_number = interner.union(vec![TypeId::STRING, TypeId::NUMBER]);

    // Union member is subtype of union
    assert!(checker.is_subtype_of(TypeId::STRING, string_or_number));
    assert!(checker.is_subtype_of(TypeId::NUMBER, string_or_number));

    // Non-member is not subtype
    assert!(!checker.is_subtype_of(TypeId::BOOLEAN, string_or_number));

    // Union is subtype if all members are subtypes
    let just_string = interner.union(vec![TypeId::STRING]);
    assert!(checker.is_subtype_of(just_string, string_or_number));
}

#[test]
fn test_object_subtyping() {
    use std::sync::Arc;

    let interner = TypeInterner::new();
    let mut checker = SubtypeChecker::new(&interner);

    // { x: number }
    let obj_x = interner.object(vec![
        PropertyInfo { name: Arc::from("x"), type_id: TypeId::NUMBER, optional: false, readonly: false },
    ]);

    // { x: number, y: string }
    let obj_xy = interner.object(vec![
        PropertyInfo { name: Arc::from("x"), type_id: TypeId::NUMBER, optional: false, readonly: false },
        PropertyInfo { name: Arc::from("y"), type_id: TypeId::STRING, optional: false, readonly: false },
    ]);

    // Object with more properties is subtype
    assert!(checker.is_subtype_of(obj_xy, obj_x));

    // Object with fewer properties is not subtype
    assert!(!checker.is_subtype_of(obj_x, obj_xy));
}

#[test]
fn test_array_subtyping() {
    let interner = TypeInterner::new();
    let mut checker = SubtypeChecker::new(&interner);

    let string_array = interner.array(TypeId::STRING);
    let number_array = interner.array(TypeId::NUMBER);
    let any_array = interner.array(TypeId::ANY);

    // Same element type
    assert!(checker.is_subtype_of(string_array, string_array));

    // Different element type
    assert!(!checker.is_subtype_of(string_array, number_array));

    // Covariance with any
    assert!(checker.is_subtype_of(string_array, any_array));
}

#[test]
fn test_type_environment() {
    let interner = TypeInterner::new();
    let mut env = TypeEnvironment::new();

    // Initially empty
    assert!(env.is_empty());
    assert_eq!(env.len(), 0);

    // Register some types
    let sym1 = SymbolRef(1);
    let sym2 = SymbolRef(2);
    env.insert(sym1, TypeId::STRING);
    env.insert(sym2, TypeId::NUMBER);

    // Check retrieval
    assert_eq!(env.get(sym1), Some(TypeId::STRING));
    assert_eq!(env.get(sym2), Some(TypeId::NUMBER));
    assert_eq!(env.get(SymbolRef(999)), None);

    // Check contains
    assert!(env.contains(sym1));
    assert!(!env.contains(SymbolRef(999)));

    // Check len
    assert_eq!(env.len(), 2);
}

#[test]
fn test_ref_resolution_with_environment() {
    use std::sync::Arc;

    let interner = TypeInterner::new();
    let mut env = TypeEnvironment::new();

    // Create a Ref type for symbol 1
    let ref_type = interner.reference(SymbolRef(1));

    // Without resolution, Ref to anything should fail (no noop resolution)
    let mut checker = SubtypeChecker::new(&interner);
    // Ref to intrinsic - can't resolve, so falls back to false
    assert!(!checker.is_subtype_of(ref_type, TypeId::STRING));

    // Add resolution: symbol 1 = string
    env.insert(SymbolRef(1), TypeId::STRING);

    // With environment, Ref(1) resolves to string
    let mut checker_with_env = SubtypeChecker::with_resolver(&interner, &env);
    assert!(checker_with_env.is_subtype_of(ref_type, TypeId::STRING));
    assert!(!checker_with_env.is_subtype_of(ref_type, TypeId::NUMBER));
}

#[test]
fn test_ref_to_ref_resolution() {
    use std::sync::Arc;

    let interner = TypeInterner::new();
    let mut env = TypeEnvironment::new();

    // Two refs that should be equal when resolved
    let ref1 = interner.reference(SymbolRef(1));
    let ref2 = interner.reference(SymbolRef(2));

    // Both resolve to string
    env.insert(SymbolRef(1), TypeId::STRING);
    env.insert(SymbolRef(2), TypeId::STRING);

    let mut checker = SubtypeChecker::with_resolver(&interner, &env);
    assert!(checker.is_subtype_of(ref1, ref2));
    assert!(checker.is_subtype_of(ref2, ref1));
}

#[test]
fn test_ref_to_object_resolution() {
    use std::sync::Arc;

    let interner = TypeInterner::new();
    let mut env = TypeEnvironment::new();

    // Create an object type: { x: number }
    let obj_x = interner.object(vec![
        PropertyInfo { name: Arc::from("x"), type_id: TypeId::NUMBER, optional: false, readonly: false },
    ]);

    // Create a Ref that resolves to { x: number, y: string }
    let obj_xy = interner.object(vec![
        PropertyInfo { name: Arc::from("x"), type_id: TypeId::NUMBER, optional: false, readonly: false },
        PropertyInfo { name: Arc::from("y"), type_id: TypeId::STRING, optional: false, readonly: false },
    ]);

    let ref_type = interner.reference(SymbolRef(100));
    env.insert(SymbolRef(100), obj_xy);

    let mut checker = SubtypeChecker::with_resolver(&interner, &env);

    // Ref resolves to { x: number, y: string } which is subtype of { x: number }
    assert!(checker.is_subtype_of(ref_type, obj_x));
}

#[test]
fn test_unresolved_ref_behavior() {
    let interner = TypeInterner::new();
    let env = TypeEnvironment::new(); // Empty environment

    let ref_type = interner.reference(SymbolRef(999));

    let mut checker = SubtypeChecker::with_resolver(&interner, &env);

    // Unresolved ref to itself should be true (same TypeId)
    assert!(checker.is_subtype_of(ref_type, ref_type));

    // Unresolved ref to something else should be false
    assert!(!checker.is_subtype_of(ref_type, TypeId::STRING));
}

#[test]
fn test_function_rest_parameter_subtyping() {
    use std::sync::Arc;

    let interner = TypeInterner::new();
    let mut checker = SubtypeChecker::new(&interner);

    // Create any[] type for rest parameter
    let any_array = interner.array(TypeId::ANY);

    // (a: string, b: any, c: any) => any - 3 fixed params
    let fixed_params = FunctionShape {
        type_params: vec![],
        params: vec![
            ParamInfo { name: Some(Arc::from("a")), type_id: TypeId::STRING, optional: false, rest: false },
            ParamInfo { name: Some(Arc::from("b")), type_id: TypeId::ANY, optional: false, rest: false },
            ParamInfo { name: Some(Arc::from("c")), type_id: TypeId::ANY, optional: false, rest: false },
        ],
        return_type: TypeId::ANY,
        is_constructor: false,
    };
    let fixed_fn = interner.function(fixed_params);

    // (a: string, b: any, ...args: any[]) => any - 2 fixed + rest
    let rest_params = FunctionShape {
        type_params: vec![],
        params: vec![
            ParamInfo { name: Some(Arc::from("a")), type_id: TypeId::STRING, optional: false, rest: false },
            ParamInfo { name: Some(Arc::from("b")), type_id: TypeId::ANY, optional: false, rest: false },
            ParamInfo { name: Some(Arc::from("args")), type_id: any_array, optional: false, rest: true },
        ],
        return_type: TypeId::ANY,
        is_constructor: false,
    };
    let rest_fn = interner.function(rest_params);

    // Function with 3 fixed params IS assignable to function with 2 fixed + rest
    // Because (a, b, c) can be called as (a, b, ...args) where args = [c]
    assert!(checker.is_subtype_of(fixed_fn, rest_fn));

    // Function with rest is NOT assignable to function with fixed params
    // (because rest can accept 0 or more args, but fixed expects exactly 3)
    // This depends on semantics - TypeScript actually allows this in some cases
    // For now, test the basic case
}
