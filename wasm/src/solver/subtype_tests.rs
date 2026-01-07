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
fn test_error_poisoning_subtyping() {
    let interner = TypeInterner::new();
    let mut checker = SubtypeChecker::new(&interner);

    assert!(checker.is_subtype_of(TypeId::ERROR, TypeId::STRING));
    assert!(checker.is_subtype_of(TypeId::STRING, TypeId::ERROR));
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
fn test_unique_symbol_subtyping() {
    let interner = TypeInterner::new();
    let mut checker = SubtypeChecker::new(&interner);

    let sym_a = interner.intern(TypeKey::UniqueSymbol(SymbolRef(1)));
    let sym_b = interner.intern(TypeKey::UniqueSymbol(SymbolRef(2)));

    assert!(checker.is_subtype_of(sym_a, sym_a));
    assert!(!checker.is_subtype_of(sym_a, sym_b));
    assert!(checker.is_subtype_of(sym_a, TypeId::SYMBOL));
    assert!(!checker.is_subtype_of(TypeId::SYMBOL, sym_a));
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
fn test_no_unchecked_indexed_access_array_subtyping() {
    let interner = TypeInterner::new();
    let mut checker = SubtypeChecker::new(&interner);

    let string_array = interner.array(TypeId::STRING);
    let index_access = interner.intern(TypeKey::IndexAccess(string_array, TypeId::NUMBER));

    assert!(checker.is_subtype_of(index_access, TypeId::STRING));

    checker.no_unchecked_indexed_access = true;
    assert!(!checker.is_subtype_of(index_access, TypeId::STRING));

    let string_or_undefined = interner.union(vec![TypeId::STRING, TypeId::UNDEFINED]);
    assert!(checker.is_subtype_of(index_access, string_or_undefined));
}

#[test]
fn test_correlated_union_index_access_subtyping() {
    let interner = TypeInterner::new();
    let mut checker = SubtypeChecker::new(&interner);

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
    let index_access = interner.intern(TypeKey::IndexAccess(union_obj, key_union));
    let expected = interner.union(vec![TypeId::NUMBER, TypeId::STRING]);

    assert!(checker.is_subtype_of(index_access, expected));
    assert!(!checker.is_subtype_of(index_access, TypeId::NUMBER));
}

#[test]
fn test_object_subtyping() {
    use std::sync::Arc;

    let interner = TypeInterner::new();
    let mut checker = SubtypeChecker::new(&interner);

    // { x: number }
    let obj_x = interner.object(vec![
        PropertyInfo { name: interner.intern_string("x"), type_id: TypeId::NUMBER,
 write_type: TypeId::NUMBER, optional: false, readonly: false, is_method: false },
    ]);

    // { x: number, y: string }
    let obj_xy = interner.object(vec![
        PropertyInfo { name: interner.intern_string("x"), type_id: TypeId::NUMBER,
 write_type: TypeId::NUMBER, optional: false, readonly: false, is_method: false },
        PropertyInfo { name: interner.intern_string("y"), type_id: TypeId::STRING,
 write_type: TypeId::STRING, optional: false, readonly: false, is_method: false },
    ]);

    // Object with more properties is subtype
    assert!(checker.is_subtype_of(obj_xy, obj_x));

    // Object with fewer properties is not subtype
    assert!(!checker.is_subtype_of(obj_x, obj_xy));
}

#[test]
fn test_readonly_property_subtyping() {
    let interner = TypeInterner::new();
    let mut checker = SubtypeChecker::new(&interner);

    let name = interner.intern_string("x");
    let readonly_obj = interner.object(vec![
        PropertyInfo { name, type_id: TypeId::NUMBER,
 write_type: TypeId::NUMBER, optional: false, readonly: true, is_method: false },
    ]);
    let mutable_obj = interner.object(vec![
        PropertyInfo { name, type_id: TypeId::NUMBER,
 write_type: TypeId::NUMBER, optional: false, readonly: false, is_method: false },
    ]);

    assert!(!checker.is_subtype_of(readonly_obj, mutable_obj));
    assert!(checker.is_subtype_of(mutable_obj, readonly_obj));
}

#[test]
fn test_readonly_array_subtyping() {
    let interner = TypeInterner::new();
    let mut checker = SubtypeChecker::new(&interner);

    let mutable_array = interner.array(TypeId::STRING);
    let readonly_array = interner.intern(TypeKey::ReadonlyType(mutable_array));

    assert!(checker.is_subtype_of(mutable_array, readonly_array));
    assert!(!checker.is_subtype_of(readonly_array, mutable_array));
}

#[test]
fn test_readonly_tuple_subtyping() {
    let interner = TypeInterner::new();
    let mut checker = SubtypeChecker::new(&interner);

    let tuple = interner.tuple(vec![
        TupleElement { type_id: TypeId::NUMBER, name: None, optional: false, rest: false },
        TupleElement { type_id: TypeId::STRING, name: None, optional: false, rest: false },
    ]);
    let readonly_tuple = interner.intern(TypeKey::ReadonlyType(tuple));

    assert!(checker.is_subtype_of(tuple, readonly_tuple));
    assert!(!checker.is_subtype_of(readonly_tuple, tuple));
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
        PropertyInfo { name: interner.intern_string("x"), type_id: TypeId::NUMBER,
 write_type: TypeId::NUMBER, optional: false, readonly: false, is_method: false },
    ]);

    // Create a Ref that resolves to { x: number, y: string }
    let obj_xy = interner.object(vec![
        PropertyInfo { name: interner.intern_string("x"), type_id: TypeId::NUMBER,
 write_type: TypeId::NUMBER, optional: false, readonly: false, is_method: false },
        PropertyInfo { name: interner.intern_string("y"), type_id: TypeId::STRING,
 write_type: TypeId::STRING, optional: false, readonly: false, is_method: false },
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
            ParamInfo { name: Some(interner.intern_string("a")), type_id: TypeId::STRING, optional: false, rest: false },
            ParamInfo { name: Some(interner.intern_string("b")), type_id: TypeId::ANY, optional: false, rest: false },
            ParamInfo { name: Some(interner.intern_string("c")), type_id: TypeId::ANY, optional: false, rest: false },
        ],
        this_type: None,
        return_type: TypeId::ANY,
        type_predicate: None,
        is_constructor: false,
    };
    let fixed_fn = interner.function(fixed_params);

    // (a: string, b: any, ...args: any[]) => any - 2 fixed + rest
    let rest_params = FunctionShape {
        type_params: vec![],
        params: vec![
            ParamInfo { name: Some(interner.intern_string("a")), type_id: TypeId::STRING, optional: false, rest: false },
            ParamInfo { name: Some(interner.intern_string("b")), type_id: TypeId::ANY, optional: false, rest: false },
            ParamInfo { name: Some(interner.intern_string("args")), type_id: any_array, optional: false, rest: true },
        ],
        this_type: None,
        return_type: TypeId::ANY,
        type_predicate: None,
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

#[test]
fn test_tuple_subtyping_extra_elements() {
    // CRITICAL: [number, string] is NOT assignable to [number]
    let interner = TypeInterner::new();
    let mut checker = SubtypeChecker::new(&interner);

    // [number, string]
    let source = interner.tuple(vec![
        TupleElement { type_id: TypeId::NUMBER, name: None, optional: false, rest: false },
        TupleElement { type_id: TypeId::STRING, name: None, optional: false, rest: false },
    ]);

    // [number]
    let target = interner.tuple(vec![
        TupleElement { type_id: TypeId::NUMBER, name: None, optional: false, rest: false },
    ]);

    // Source has extra elements, target is closed -> should FAIL
    assert!(!checker.is_subtype_of(source, target));
}

#[test]
fn test_tuple_subtyping_with_rest_target() {
    // [number, string] IS assignable to [number, ...string[]]
    let interner = TypeInterner::new();
    let mut checker = SubtypeChecker::new(&interner);

    let string_array = interner.array(TypeId::STRING);

    // [number, string]
    let source = interner.tuple(vec![
        TupleElement { type_id: TypeId::NUMBER, name: None, optional: false, rest: false },
        TupleElement { type_id: TypeId::STRING, name: None, optional: false, rest: false },
    ]);

    // [number, ...string[]]
    let target = interner.tuple(vec![
        TupleElement { type_id: TypeId::NUMBER, name: None, optional: false, rest: false },
        TupleElement { type_id: string_array, name: None, optional: false, rest: true },
    ]);

    // Target has rest -> should accept extra elements
    assert!(checker.is_subtype_of(source, target));
}

#[test]
fn test_tuple_subtyping_rest_tuple_expansion() {
    // [number, string, boolean] IS assignable to [number, ...[string, boolean]]
    let interner = TypeInterner::new();
    let mut checker = SubtypeChecker::new(&interner);

    let rest_tuple = interner.tuple(vec![
        TupleElement { type_id: TypeId::STRING, name: None, optional: false, rest: false },
        TupleElement { type_id: TypeId::BOOLEAN, name: None, optional: false, rest: false },
    ]);

    let source = interner.tuple(vec![
        TupleElement { type_id: TypeId::NUMBER, name: None, optional: false, rest: false },
        TupleElement { type_id: TypeId::STRING, name: None, optional: false, rest: false },
        TupleElement { type_id: TypeId::BOOLEAN, name: None, optional: false, rest: false },
    ]);

    let target = interner.tuple(vec![
        TupleElement { type_id: TypeId::NUMBER, name: None, optional: false, rest: false },
        TupleElement { type_id: rest_tuple, name: None, optional: false, rest: true },
    ]);

    assert!(checker.is_subtype_of(source, target));
}

#[test]
fn test_tuple_subtyping_rest_tuple_missing_element() {
    // [number, string] is NOT assignable to [number, ...[string, boolean]]
    let interner = TypeInterner::new();
    let mut checker = SubtypeChecker::new(&interner);

    let rest_tuple = interner.tuple(vec![
        TupleElement { type_id: TypeId::STRING, name: None, optional: false, rest: false },
        TupleElement { type_id: TypeId::BOOLEAN, name: None, optional: false, rest: false },
    ]);

    let source = interner.tuple(vec![
        TupleElement { type_id: TypeId::NUMBER, name: None, optional: false, rest: false },
        TupleElement { type_id: TypeId::STRING, name: None, optional: false, rest: false },
    ]);

    let target = interner.tuple(vec![
        TupleElement { type_id: TypeId::NUMBER, name: None, optional: false, rest: false },
        TupleElement { type_id: rest_tuple, name: None, optional: false, rest: true },
    ]);

    assert!(!checker.is_subtype_of(source, target));
}

#[test]
fn test_tuple_subtyping_rest_tuple_extra_element() {
    // [number, string, boolean, boolean] is NOT assignable to [number, ...[string, boolean]]
    let interner = TypeInterner::new();
    let mut checker = SubtypeChecker::new(&interner);

    let rest_tuple = interner.tuple(vec![
        TupleElement { type_id: TypeId::STRING, name: None, optional: false, rest: false },
        TupleElement { type_id: TypeId::BOOLEAN, name: None, optional: false, rest: false },
    ]);

    let source = interner.tuple(vec![
        TupleElement { type_id: TypeId::NUMBER, name: None, optional: false, rest: false },
        TupleElement { type_id: TypeId::STRING, name: None, optional: false, rest: false },
        TupleElement { type_id: TypeId::BOOLEAN, name: None, optional: false, rest: false },
        TupleElement { type_id: TypeId::BOOLEAN, name: None, optional: false, rest: false },
    ]);

    let target = interner.tuple(vec![
        TupleElement { type_id: TypeId::NUMBER, name: None, optional: false, rest: false },
        TupleElement { type_id: rest_tuple, name: None, optional: false, rest: true },
    ]);

    assert!(!checker.is_subtype_of(source, target));
}

#[test]
fn test_tuple_subtyping_rest_tuple_variadic_tail() {
    // [number, string, boolean, boolean] IS assignable to [number, ...[string, ...boolean[]]]
    let interner = TypeInterner::new();
    let mut checker = SubtypeChecker::new(&interner);

    let boolean_array = interner.array(TypeId::BOOLEAN);
    let rest_tuple = interner.tuple(vec![
        TupleElement { type_id: TypeId::STRING, name: None, optional: false, rest: false },
        TupleElement { type_id: boolean_array, name: None, optional: false, rest: true },
    ]);

    let source = interner.tuple(vec![
        TupleElement { type_id: TypeId::NUMBER, name: None, optional: false, rest: false },
        TupleElement { type_id: TypeId::STRING, name: None, optional: false, rest: false },
        TupleElement { type_id: TypeId::BOOLEAN, name: None, optional: false, rest: false },
        TupleElement { type_id: TypeId::BOOLEAN, name: None, optional: false, rest: false },
    ]);

    let target = interner.tuple(vec![
        TupleElement { type_id: TypeId::NUMBER, name: None, optional: false, rest: false },
        TupleElement { type_id: rest_tuple, name: None, optional: false, rest: true },
    ]);

    assert!(checker.is_subtype_of(source, target));
}

#[test]
fn test_tuple_subtyping_source_rest_closed_target() {
    // [number, ...string[]] is NOT assignable to [number, string]
    let interner = TypeInterner::new();
    let mut checker = SubtypeChecker::new(&interner);

    let string_array = interner.array(TypeId::STRING);

    // [number, ...string[]]
    let source = interner.tuple(vec![
        TupleElement { type_id: TypeId::NUMBER, name: None, optional: false, rest: false },
        TupleElement { type_id: string_array, name: None, optional: false, rest: true },
    ]);

    // [number, string]
    let target = interner.tuple(vec![
        TupleElement { type_id: TypeId::NUMBER, name: None, optional: false, rest: false },
        TupleElement { type_id: TypeId::STRING, name: None, optional: false, rest: false },
    ]);

    // Source has rest but target is closed -> should FAIL
    assert!(!checker.is_subtype_of(source, target));
}

#[test]
fn test_tuple_subtyping_optional_elements() {
    // [number, string?] IS assignable to [number]
    let interner = TypeInterner::new();
    let mut checker = SubtypeChecker::new(&interner);

    // [number, string?]
    let source = interner.tuple(vec![
        TupleElement { type_id: TypeId::NUMBER, name: None, optional: false, rest: false },
        TupleElement { type_id: TypeId::STRING, name: None, optional: true, rest: false },
    ]);

    // [number]
    let target = interner.tuple(vec![
        TupleElement { type_id: TypeId::NUMBER, name: None, optional: false, rest: false },
    ]);

    // Optional elements don't count as "extra" if they're beyond target length
    // This is actually a borderline case - TypeScript may reject this
    // For strictness, we reject tuples with more elements even if optional
    assert!(!checker.is_subtype_of(source, target));
}

#[test]
fn test_tuple_subtyping_rest_to_rest() {
    // [number, ...string[]] IS assignable to [number, ...string[]]
    let interner = TypeInterner::new();
    let mut checker = SubtypeChecker::new(&interner);

    let string_array = interner.array(TypeId::STRING);

    // [number, ...string[]]
    let source = interner.tuple(vec![
        TupleElement { type_id: TypeId::NUMBER, name: None, optional: false, rest: false },
        TupleElement { type_id: string_array, name: None, optional: false, rest: true },
    ]);

    // [number, ...string[]]
    let target = interner.tuple(vec![
        TupleElement { type_id: TypeId::NUMBER, name: None, optional: false, rest: false },
        TupleElement { type_id: string_array, name: None, optional: false, rest: true },
    ]);

    // Both have rest, same types -> should succeed
    assert!(checker.is_subtype_of(source, target));
}

#[test]
fn test_tuple_to_array_with_rest() {
    // BLOCKER fix: [number, ...string[]] IS assignable to string[]
    let interner = TypeInterner::new();
    let mut checker = SubtypeChecker::new(&interner);

    let string_array = interner.array(TypeId::STRING);

    // [number, ...string[]]
    let source = interner.tuple(vec![
        TupleElement { type_id: TypeId::NUMBER, name: None, optional: false, rest: false },
        TupleElement { type_id: string_array, name: None, optional: false, rest: true },
    ]);

    // string[]
    let target = string_array;

    // This should FAIL because first element is number, not string
    assert!(!checker.is_subtype_of(source, target));
}

#[test]
fn test_tuple_to_array_with_rest_tuple() {
    // [string, ...[string, string]] IS assignable to string[]
    let interner = TypeInterner::new();
    let mut checker = SubtypeChecker::new(&interner);

    let string_array = interner.array(TypeId::STRING);
    let rest_tuple = interner.tuple(vec![
        TupleElement { type_id: TypeId::STRING, name: None, optional: false, rest: false },
        TupleElement { type_id: TypeId::STRING, name: None, optional: false, rest: false },
    ]);

    let source = interner.tuple(vec![
        TupleElement { type_id: TypeId::STRING, name: None, optional: false, rest: false },
        TupleElement { type_id: rest_tuple, name: None, optional: false, rest: true },
    ]);

    assert!(checker.is_subtype_of(source, string_array));
}

#[test]
fn test_tuple_to_array_with_rest_tuple_mismatch() {
    // [string, ...[string, number]] is NOT assignable to string[]
    let interner = TypeInterner::new();
    let mut checker = SubtypeChecker::new(&interner);

    let string_array = interner.array(TypeId::STRING);
    let rest_tuple = interner.tuple(vec![
        TupleElement { type_id: TypeId::STRING, name: None, optional: false, rest: false },
        TupleElement { type_id: TypeId::NUMBER, name: None, optional: false, rest: false },
    ]);

    let source = interner.tuple(vec![
        TupleElement { type_id: TypeId::STRING, name: None, optional: false, rest: false },
        TupleElement { type_id: rest_tuple, name: None, optional: false, rest: true },
    ]);

    assert!(!checker.is_subtype_of(source, string_array));
}

#[test]
fn test_tuple_to_array_with_rest_tuple_variadic() {
    // [string, ...[string, ...string[]]] IS assignable to string[]
    let interner = TypeInterner::new();
    let mut checker = SubtypeChecker::new(&interner);

    let string_array = interner.array(TypeId::STRING);
    let rest_tuple = interner.tuple(vec![
        TupleElement { type_id: TypeId::STRING, name: None, optional: false, rest: false },
        TupleElement { type_id: string_array, name: None, optional: false, rest: true },
    ]);

    let source = interner.tuple(vec![
        TupleElement { type_id: TypeId::STRING, name: None, optional: false, rest: false },
        TupleElement { type_id: rest_tuple, name: None, optional: false, rest: true },
    ]);

    assert!(checker.is_subtype_of(source, string_array));
}

#[test]
fn test_tuple_to_array_all_matching_with_rest() {
    // [string, ...string[]] IS assignable to string[]
    let interner = TypeInterner::new();
    let mut checker = SubtypeChecker::new(&interner);

    let string_array = interner.array(TypeId::STRING);

    // [string, ...string[]]
    let source = interner.tuple(vec![
        TupleElement { type_id: TypeId::STRING, name: None, optional: false, rest: false },
        TupleElement { type_id: string_array, name: None, optional: false, rest: true },
    ]);

    // string[]
    let target = string_array;

    // This should SUCCEED - all elements (including rest) are strings
    assert!(checker.is_subtype_of(source, target));
}

#[test]
fn test_tuple_to_array_no_rest() {
    // [string, string] IS assignable to string[]
    let interner = TypeInterner::new();
    let mut checker = SubtypeChecker::new(&interner);

    let string_array = interner.array(TypeId::STRING);

    // [string, string]
    let source = interner.tuple(vec![
        TupleElement { type_id: TypeId::STRING, name: None, optional: false, rest: false },
        TupleElement { type_id: TypeId::STRING, name: None, optional: false, rest: false },
    ]);

    // string[]
    let target = string_array;

    // This should SUCCEED - all fixed elements are strings
    assert!(checker.is_subtype_of(source, target));
}

#[test]
fn test_tuple_to_array_mixed_types() {
    // [number, string] is NOT assignable to string[]
    let interner = TypeInterner::new();
    let mut checker = SubtypeChecker::new(&interner);

    let string_array = interner.array(TypeId::STRING);

    // [number, string]
    let source = interner.tuple(vec![
        TupleElement { type_id: TypeId::NUMBER, name: None, optional: false, rest: false },
        TupleElement { type_id: TypeId::STRING, name: None, optional: false, rest: false },
    ]);

    // string[]
    let target = string_array;

    // This should FAIL - first element is number, not string
    assert!(!checker.is_subtype_of(source, target));
}

#[test]
fn test_array_to_variadic_tuple() {
    // string[] is NOT assignable to [...string[]]
    let interner = TypeInterner::new();
    let mut checker = SubtypeChecker::new(&interner);

    let string_array = interner.array(TypeId::STRING);
    let target = interner.tuple(vec![
        TupleElement { type_id: string_array, name: None, optional: false, rest: true },
    ]);

    assert!(!checker.is_subtype_of(string_array, target));
}

#[test]
fn test_array_to_variadic_tuple_with_required_prefix() {
    // string[] is NOT assignable to [string, ...string[]]
    let interner = TypeInterner::new();
    let mut checker = SubtypeChecker::new(&interner);

    let string_array = interner.array(TypeId::STRING);
    let target = interner.tuple(vec![
        TupleElement { type_id: TypeId::STRING, name: None, optional: false, rest: false },
        TupleElement { type_id: string_array, name: None, optional: false, rest: true },
    ]);

    assert!(!checker.is_subtype_of(string_array, target));
}

#[test]
fn test_array_to_variadic_tuple_with_optional_prefix() {
    // string[] is NOT assignable to [string?, ...string[]]
    let interner = TypeInterner::new();
    let mut checker = SubtypeChecker::new(&interner);

    let string_array = interner.array(TypeId::STRING);
    let target = interner.tuple(vec![
        TupleElement { type_id: TypeId::STRING, name: None, optional: true, rest: false },
        TupleElement { type_id: string_array, name: None, optional: false, rest: true },
    ]);

    assert!(!checker.is_subtype_of(string_array, target));
}

#[test]
fn test_array_to_fixed_optional_tuple() {
    // string[] is NOT assignable to [string?]
    let interner = TypeInterner::new();
    let mut checker = SubtypeChecker::new(&interner);

    let string_array = interner.array(TypeId::STRING);
    let target = interner.tuple(vec![
        TupleElement { type_id: TypeId::STRING, name: None, optional: true, rest: false },
    ]);

    assert!(!checker.is_subtype_of(string_array, target));
}

#[test]
fn test_never_array_to_optional_tuple() {
    // never[] IS assignable to [] and [string?]
    let interner = TypeInterner::new();
    let mut checker = SubtypeChecker::new(&interner);

    let never_array = interner.array(TypeId::NEVER);
    let empty_tuple = interner.tuple(Vec::new());
    let optional_tuple = interner.tuple(vec![
        TupleElement { type_id: TypeId::STRING, name: None, optional: true, rest: false },
    ]);
    let required_tuple = interner.tuple(vec![
        TupleElement { type_id: TypeId::STRING, name: None, optional: false, rest: false },
    ]);

    assert!(checker.is_subtype_of(never_array, empty_tuple));
    assert!(checker.is_subtype_of(never_array, optional_tuple));
    assert!(!checker.is_subtype_of(never_array, required_tuple));
}

#[test]
fn test_never_array_to_variadic_tuple() {
    // never[] IS assignable to [...string[]]
    let interner = TypeInterner::new();
    let mut checker = SubtypeChecker::new(&interner);

    let never_array = interner.array(TypeId::NEVER);
    let string_array = interner.array(TypeId::STRING);
    let target = interner.tuple(vec![
        TupleElement { type_id: string_array, name: None, optional: false, rest: true },
    ]);

    assert!(checker.is_subtype_of(never_array, target));
}

#[test]
fn test_number_index_signature_numeric_property() {
    // CRITICAL: { 0: string } should match { [x: number]: string }
    use std::sync::Arc;
    let interner = TypeInterner::new();
    let mut checker = SubtypeChecker::new(&interner);

    // { 0: string }
    let source = interner.object(vec![
        PropertyInfo {
            name: interner.intern_string("0"),
            type_id: TypeId::STRING,
            write_type: TypeId::STRING,
            optional: false,
            readonly: false,
            is_method: false,
        },
    ]);

    // { [x: number]: string }
    let target_shape = ObjectShape {
        properties: vec![],
        number_index: Some(IndexSignature {
            key_type: TypeId::NUMBER,
            value_type: TypeId::STRING,
            readonly: false,
        }),
        string_index: None,
    };
    let target = interner.object_with_index(target_shape);

    // This should SUCCEED - numeric property "0" matches number index signature
    assert!(checker.is_subtype_of(source, target));
}

#[test]
fn test_number_index_signature_type_mismatch() {
    // { 0: number } should NOT match { [x: number]: string }
    use std::sync::Arc;
    let interner = TypeInterner::new();
    let mut checker = SubtypeChecker::new(&interner);

    // { 0: number }
    let source = interner.object(vec![
        PropertyInfo {
            name: interner.intern_string("0"),
            type_id: TypeId::NUMBER,
            write_type: TypeId::NUMBER,
            optional: false,
            readonly: false,
            is_method: false,
        },
    ]);

    // { [x: number]: string }
    let target_shape = ObjectShape {
        properties: vec![],
        number_index: Some(IndexSignature {
            key_type: TypeId::NUMBER,
            value_type: TypeId::STRING,
            readonly: false,
        }),
        string_index: None,
    };
    let target = interner.object_with_index(target_shape);

    // This should FAIL - numeric property has wrong type
    assert!(!checker.is_subtype_of(source, target));
}

#[test]
fn test_number_index_signature_multiple_numeric_props() {
    // { 0: string, 1: string, 2: string } should match { [x: number]: string }
    use std::sync::Arc;
    let interner = TypeInterner::new();
    let mut checker = SubtypeChecker::new(&interner);

    // { 0: string, 1: string, 2: string }
    let source = interner.object(vec![
        PropertyInfo {
            name: interner.intern_string("0"),
            type_id: TypeId::STRING,
            write_type: TypeId::STRING,
            optional: false,
            readonly: false,
            is_method: false,
        },
        PropertyInfo {
            name: interner.intern_string("1"),
            type_id: TypeId::STRING,
            write_type: TypeId::STRING,
            optional: false,
            readonly: false,
            is_method: false,
        },
        PropertyInfo {
            name: interner.intern_string("2"),
            type_id: TypeId::STRING,
            write_type: TypeId::STRING,
            optional: false,
            readonly: false,
            is_method: false,
        },
    ]);

    // { [x: number]: string }
    let target_shape = ObjectShape {
        properties: vec![],
        number_index: Some(IndexSignature {
            key_type: TypeId::NUMBER,
            value_type: TypeId::STRING,
            readonly: false,
        }),
        string_index: None,
    };
    let target = interner.object_with_index(target_shape);

    // This should SUCCEED - all numeric properties match
    assert!(checker.is_subtype_of(source, target));
}

#[test]
fn test_number_and_string_index_signatures() {
    // { 0: string, foo: string } should match { [x: number]: string; [y: string]: string }
    use std::sync::Arc;
    let interner = TypeInterner::new();
    let mut checker = SubtypeChecker::new(&interner);

    // { 0: string, foo: string }
    let source = interner.object(vec![
        PropertyInfo {
            name: interner.intern_string("0"),
            type_id: TypeId::STRING,
            write_type: TypeId::STRING,
            optional: false,
            readonly: false,
            is_method: false,
        },
        PropertyInfo {
            name: interner.intern_string("foo"),
            type_id: TypeId::STRING,
            write_type: TypeId::STRING,
            optional: false,
            readonly: false,
            is_method: false,
        },
    ]);

    // { [x: number]: string; [y: string]: string }
    let target_shape = ObjectShape {
        properties: vec![],
        number_index: Some(IndexSignature {
            key_type: TypeId::NUMBER,
            value_type: TypeId::STRING,
            readonly: false,
        }),
        string_index: Some(IndexSignature {
            key_type: TypeId::STRING,
            value_type: TypeId::STRING,
            readonly: false,
        }),
    };
    let target = interner.object_with_index(target_shape);

    // This should SUCCEED - "0" satisfies number index, both satisfy string index
    assert!(checker.is_subtype_of(source, target));
}

#[test]
fn test_readonly_index_signature_subtyping() {
    let interner = TypeInterner::new();
    let mut checker = SubtypeChecker::new(&interner);

    let readonly_source = interner.object_with_index(ObjectShape {
        properties: vec![],
        number_index: None,
        string_index: Some(IndexSignature {
            key_type: TypeId::STRING,
            value_type: TypeId::NUMBER,
            readonly: true,
        }),
    });

    let mutable_target = interner.object_with_index(ObjectShape {
        properties: vec![],
        number_index: None,
        string_index: Some(IndexSignature {
            key_type: TypeId::STRING,
            value_type: TypeId::NUMBER,
            readonly: false,
        }),
    });

    let readonly_target = interner.object_with_index(ObjectShape {
        properties: vec![],
        number_index: None,
        string_index: Some(IndexSignature {
            key_type: TypeId::STRING,
            value_type: TypeId::NUMBER,
            readonly: true,
        }),
    });

    assert!(!checker.is_subtype_of(readonly_source, mutable_target));
    assert!(checker.is_subtype_of(mutable_target, readonly_target));
}

#[test]
fn test_readonly_property_with_mutable_index_signature() {
    let interner = TypeInterner::new();
    let mut checker = SubtypeChecker::new(&interner);

    let source = interner.object(vec![PropertyInfo {
        name: interner.intern_string("x"),
        type_id: TypeId::NUMBER,
        write_type: TypeId::NUMBER,
        optional: false,
        readonly: true,
        is_method: false,
    }]);

    let mutable_index = interner.object_with_index(ObjectShape {
        properties: vec![],
        number_index: None,
        string_index: Some(IndexSignature {
            key_type: TypeId::STRING,
            value_type: TypeId::NUMBER,
            readonly: false,
        }),
    });

    let readonly_index = interner.object_with_index(ObjectShape {
        properties: vec![],
        number_index: None,
        string_index: Some(IndexSignature {
            key_type: TypeId::STRING,
            value_type: TypeId::NUMBER,
            readonly: true,
        }),
    });

    assert!(!checker.is_subtype_of(source, mutable_index));
    assert!(checker.is_subtype_of(source, readonly_index));
}

#[test]
fn test_object_with_index_properties_match_target_index() {
    let interner = TypeInterner::new();
    let mut checker = SubtypeChecker::new(&interner);

    let source = interner.object_with_index(ObjectShape {
        properties: vec![
            PropertyInfo {
                name: interner.intern_string("0"),
                type_id: TypeId::STRING,
                write_type: TypeId::STRING,
                optional: false,
                readonly: false,
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
        ],
        number_index: Some(IndexSignature {
            key_type: TypeId::NUMBER,
            value_type: TypeId::STRING,
            readonly: false,
        }),
        string_index: Some(IndexSignature {
            key_type: TypeId::STRING,
            value_type: TypeId::STRING,
            readonly: false,
        }),
    });

    let target = interner.object_with_index(ObjectShape {
        properties: vec![],
        number_index: Some(IndexSignature {
            key_type: TypeId::NUMBER,
            value_type: TypeId::STRING,
            readonly: false,
        }),
        string_index: Some(IndexSignature {
            key_type: TypeId::STRING,
            value_type: TypeId::STRING,
            readonly: false,
        }),
    });

    assert!(checker.is_subtype_of(source, target));
}

#[test]
fn test_object_with_index_property_mismatch_string_index() {
    let interner = TypeInterner::new();
    let mut checker = SubtypeChecker::new(&interner);

    let source = interner.object_with_index(ObjectShape {
        properties: vec![PropertyInfo {
            name: interner.intern_string("name"),
            type_id: TypeId::STRING,
            write_type: TypeId::STRING,
            optional: false,
            readonly: false,
            is_method: false,
        }],
        number_index: None,
        string_index: Some(IndexSignature {
            key_type: TypeId::STRING,
            value_type: TypeId::NUMBER,
            readonly: false,
        }),
    });

    let target = interner.object_with_index(ObjectShape {
        properties: vec![],
        number_index: None,
        string_index: Some(IndexSignature {
            key_type: TypeId::STRING,
            value_type: TypeId::NUMBER,
            readonly: false,
        }),
    });

    assert!(!checker.is_subtype_of(source, target));
}

#[test]
fn test_object_with_index_property_mismatch_number_index() {
    let interner = TypeInterner::new();
    let mut checker = SubtypeChecker::new(&interner);

    let source = interner.object_with_index(ObjectShape {
        properties: vec![PropertyInfo {
            name: interner.intern_string("0"),
            type_id: TypeId::STRING,
            write_type: TypeId::STRING,
            optional: false,
            readonly: false,
            is_method: false,
        }],
        number_index: Some(IndexSignature {
            key_type: TypeId::NUMBER,
            value_type: TypeId::NUMBER,
            readonly: false,
        }),
        string_index: None,
    });

    let target = interner.object_with_index(ObjectShape {
        properties: vec![],
        number_index: Some(IndexSignature {
            key_type: TypeId::NUMBER,
            value_type: TypeId::NUMBER,
            readonly: false,
        }),
        string_index: None,
    });

    assert!(!checker.is_subtype_of(source, target));
}

#[test]
fn test_object_with_index_satisfies_named_property_string_index() {
    let interner = TypeInterner::new();
    let mut checker = SubtypeChecker::new(&interner);

    let source = interner.object_with_index(ObjectShape {
        properties: vec![],
        number_index: None,
        string_index: Some(IndexSignature {
            key_type: TypeId::STRING,
            value_type: TypeId::NUMBER,
            readonly: false,
        }),
    });

    let target = interner.object(vec![PropertyInfo {
        name: interner.intern_string("a"),
        type_id: TypeId::NUMBER,
        write_type: TypeId::NUMBER,
        optional: false,
        readonly: false,
        is_method: false,
    }]);

    assert!(checker.is_subtype_of(source, target));
}

#[test]
fn test_object_with_index_named_property_mismatch_string_index() {
    let interner = TypeInterner::new();
    let mut checker = SubtypeChecker::new(&interner);

    let source = interner.object_with_index(ObjectShape {
        properties: vec![],
        number_index: None,
        string_index: Some(IndexSignature {
            key_type: TypeId::STRING,
            value_type: TypeId::NUMBER,
            readonly: false,
        }),
    });

    let target = interner.object(vec![PropertyInfo {
        name: interner.intern_string("a"),
        type_id: TypeId::STRING,
        write_type: TypeId::STRING,
        optional: false,
        readonly: false,
        is_method: false,
    }]);

    assert!(!checker.is_subtype_of(source, target));
}

#[test]
fn test_object_with_index_satisfies_numeric_property_number_index() {
    let interner = TypeInterner::new();
    let mut checker = SubtypeChecker::new(&interner);

    let source = interner.object_with_index(ObjectShape {
        properties: vec![],
        number_index: Some(IndexSignature {
            key_type: TypeId::NUMBER,
            value_type: TypeId::STRING,
            readonly: false,
        }),
        string_index: None,
    });

    let target = interner.object(vec![PropertyInfo {
        name: interner.intern_string("0"),
        type_id: TypeId::STRING,
        write_type: TypeId::STRING,
        optional: false,
        readonly: false,
        is_method: false,
    }]);

    assert!(checker.is_subtype_of(source, target));
}

#[test]
fn test_object_with_index_noncanonical_numeric_property_fails() {
    let interner = TypeInterner::new();
    let mut checker = SubtypeChecker::new(&interner);

    let source = interner.object_with_index(ObjectShape {
        properties: vec![],
        number_index: Some(IndexSignature {
            key_type: TypeId::NUMBER,
            value_type: TypeId::STRING,
            readonly: false,
        }),
        string_index: None,
    });

    let target = interner.object(vec![PropertyInfo {
        name: interner.intern_string("01"),
        type_id: TypeId::STRING,
        write_type: TypeId::STRING,
        optional: false,
        readonly: false,
        is_method: false,
    }]);

    assert!(!checker.is_subtype_of(source, target));
}

#[test]
fn test_object_with_index_readonly_index_to_mutable_property_fails() {
    let interner = TypeInterner::new();
    let mut checker = SubtypeChecker::new(&interner);

    let source = interner.object_with_index(ObjectShape {
        properties: vec![],
        number_index: None,
        string_index: Some(IndexSignature {
            key_type: TypeId::STRING,
            value_type: TypeId::NUMBER,
            readonly: true,
        }),
    });

    let target = interner.object(vec![PropertyInfo {
        name: interner.intern_string("a"),
        type_id: TypeId::NUMBER,
        write_type: TypeId::NUMBER,
        optional: false,
        readonly: false,
        is_method: false,
    }]);

    assert!(!checker.is_subtype_of(source, target));
}

#[test]
fn test_type_parameter_constraint_assignability() {
    let interner = TypeInterner::new();
    let mut checker = SubtypeChecker::new(&interner);

    let t_param = interner.intern(TypeKey::TypeParameter(TypeParamInfo {
        name: interner.intern_string("T"),
        constraint: Some(TypeId::STRING),
        default: None,
    }));

    assert!(checker.is_subtype_of(t_param, TypeId::STRING));
    assert!(!checker.is_subtype_of(t_param, TypeId::NUMBER));

    let unconstrained = interner.intern(TypeKey::TypeParameter(TypeParamInfo {
        name: interner.intern_string("U"),
        constraint: None,
        default: None,
    }));
    assert!(!checker.is_subtype_of(unconstrained, TypeId::STRING));
}

#[test]
fn test_type_parameter_identity_only() {
    let interner = TypeInterner::new();
    let mut checker = SubtypeChecker::new(&interner);

    let t_param = interner.intern(TypeKey::TypeParameter(TypeParamInfo {
        name: interner.intern_string("T"),
        constraint: Some(TypeId::STRING),
        default: None,
    }));
    let u_param = interner.intern(TypeKey::TypeParameter(TypeParamInfo {
        name: interner.intern_string("U"),
        constraint: Some(TypeId::STRING),
        default: None,
    }));

    assert!(!checker.is_subtype_of(t_param, u_param));
}

#[test]
fn test_deferred_conditional_source_subtyping() {
    let interner = TypeInterner::new();
    let mut checker = SubtypeChecker::new(&interner);

    let t_param = interner.intern(TypeKey::TypeParameter(TypeParamInfo {
        name: interner.intern_string("T"),
        constraint: None,
        default: None,
    }));

    let conditional = interner.conditional(ConditionalType {
        check_type: t_param,
        extends_type: TypeId::STRING,
        true_type: TypeId::NUMBER,
        false_type: TypeId::BOOLEAN,
        is_distributive: true,
    });

    let target_union = interner.union(vec![TypeId::NUMBER, TypeId::BOOLEAN]);

    assert!(checker.is_subtype_of(conditional, target_union));
    assert!(!checker.is_subtype_of(conditional, TypeId::NUMBER));
}

#[test]
fn test_deferred_conditional_target_subtyping() {
    let interner = TypeInterner::new();
    let mut checker = SubtypeChecker::new(&interner);

    let t_param = interner.intern(TypeKey::TypeParameter(TypeParamInfo {
        name: interner.intern_string("T"),
        constraint: None,
        default: None,
    }));

    let conditional = interner.conditional(ConditionalType {
        check_type: t_param,
        extends_type: TypeId::STRING,
        true_type: TypeId::NUMBER,
        false_type: TypeId::BOOLEAN,
        is_distributive: true,
    });

    assert!(!checker.is_subtype_of(TypeId::NUMBER, conditional));
}

#[test]
fn test_deferred_conditional_structural_subtyping() {
    let interner = TypeInterner::new();
    let mut checker = SubtypeChecker::new(&interner);

    let t_param = interner.intern(TypeKey::TypeParameter(TypeParamInfo {
        name: interner.intern_string("T"),
        constraint: None,
        default: None,
    }));

    let source = interner.conditional(ConditionalType {
        check_type: t_param,
        extends_type: TypeId::STRING,
        true_type: TypeId::NUMBER,
        false_type: TypeId::BOOLEAN,
        is_distributive: true,
    });

    let union_nb = interner.union(vec![TypeId::NUMBER, TypeId::BOOLEAN]);
    let target = interner.conditional(ConditionalType {
        check_type: t_param,
        extends_type: TypeId::STRING,
        true_type: union_nb,
        false_type: union_nb,
        is_distributive: true,
    });

    let mismatch = interner.conditional(ConditionalType {
        check_type: t_param,
        extends_type: TypeId::NUMBER,
        true_type: union_nb,
        false_type: union_nb,
        is_distributive: true,
    });

    assert!(checker.is_subtype_of(source, target));
    assert!(!checker.is_subtype_of(source, mismatch));
}

#[test]
fn test_strict_function_variance() {
    use std::sync::Arc;
    let interner = TypeInterner::new();
    let mut checker = SubtypeChecker::new(&interner);
    // Ensure strict mode is on (default)
    assert_eq!(checker.strict_function_types, true);

    // (x: string | number) => void
    let union_arg_fn = interner.function(FunctionShape {
        type_params: vec![],
        params: vec![ParamInfo {
            name: Some(interner.intern_string("x")),
            type_id: interner.union(vec![TypeId::STRING, TypeId::NUMBER]),
            optional: false,
            rest: false,
        }],
        this_type: None,
        return_type: TypeId::VOID,
        type_predicate: None,
        is_constructor: false,
    });

    // (x: string) => void
    let string_arg_fn = interner.function(FunctionShape {
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

    // 1. Safe assignment: (string | number) => void  <:  (string) => void
    // Target param (string) <: Source param (string | number) -> OK (contravariant)
    assert!(checker.is_subtype_of(union_arg_fn, string_arg_fn));

    // 2. Unsafe assignment: (string) => void  <:  (string | number) => void
    // Target param (string | number) <: Source param (string) -> FAIL (would be unsound)
    assert!(!checker.is_subtype_of(string_arg_fn, union_arg_fn));

    // 3. Disable strict mode (Bivariant)
    checker.strict_function_types = false;
    // Now unsafe assignment should pass (legacy behavior)
    assert!(checker.is_subtype_of(string_arg_fn, union_arg_fn));
}

#[test]
fn test_this_parameter_variance() {
    let interner = TypeInterner::new();
    let mut checker = SubtypeChecker::new(&interner);

    let union_this = interner.union(vec![TypeId::STRING, TypeId::NUMBER]);
    let union_this_fn = interner.function(FunctionShape {
        type_params: vec![],
        params: vec![],
        this_type: Some(union_this),
        return_type: TypeId::VOID,
        type_predicate: None,
        is_constructor: false,
    });

    let string_this_fn = interner.function(FunctionShape {
        type_params: vec![],
        params: vec![],
        this_type: Some(TypeId::STRING),
        return_type: TypeId::VOID,
        type_predicate: None,
        is_constructor: false,
    });

    // this parameter is contravariant like regular parameters
    assert!(checker.is_subtype_of(union_this_fn, string_this_fn));
    assert!(!checker.is_subtype_of(string_this_fn, union_this_fn));
}
#[test]
fn test_function_fixed_to_rest_subtyping() {
    use std::sync::Arc;

    let interner = TypeInterner::new();
    let mut checker = SubtypeChecker::new(&interner);

    // Source: (name: string, mixed: any, arg: any) => any
    let source = interner.function(FunctionShape {
        type_params: vec![],
        params: vec![
            ParamInfo { name: Some(interner.intern_string("name")), type_id: TypeId::STRING, optional: false, rest: false },
            ParamInfo { name: Some(interner.intern_string("mixed")), type_id: TypeId::ANY, optional: false, rest: false },
            ParamInfo { name: Some(interner.intern_string("arg")), type_id: TypeId::ANY, optional: false, rest: false },
        ],
        this_type: None,
        return_type: TypeId::ANY,
        type_predicate: None,
        is_constructor: false,
    });

    // Target: (name: string, mixed: any, ...args: any[]) => any
    let any_array = interner.array(TypeId::ANY);
    let target = interner.function(FunctionShape {
        type_params: vec![],
        params: vec![
            ParamInfo { name: Some(interner.intern_string("name")), type_id: TypeId::STRING, optional: false, rest: false },
            ParamInfo { name: Some(interner.intern_string("mixed")), type_id: TypeId::ANY, optional: false, rest: false },
            ParamInfo { name: Some(interner.intern_string("args")), type_id: any_array, optional: false, rest: true },
        ],
        this_type: None,
        return_type: TypeId::ANY,
        type_predicate: None,
        is_constructor: false,
    });

    // Function with fixed params should be subtype of function with rest params
    // This matches TypeScript behavior
    assert!(checker.is_subtype_of(source, target), "Function with 3 fixed params should be subtype of function with 2 fixed + rest params");
}

#[test]
fn test_function_rest_tuple_to_rest_array_subtyping() {
    use std::sync::Arc;

    let interner = TypeInterner::new();
    let mut checker = SubtypeChecker::new(&interner);

    // Source: (name: string, mixed: any, ...args: [any]) => any
    let tuple_one_any = interner.tuple(vec![TupleElement {
        type_id: TypeId::ANY,
        name: None,
        optional: false,
        rest: false,
    }]);
    let source = interner.function(FunctionShape {
        type_params: vec![],
        params: vec![
            ParamInfo { name: Some(interner.intern_string("name")), type_id: TypeId::STRING, optional: false, rest: false },
            ParamInfo { name: Some(interner.intern_string("mixed")), type_id: TypeId::ANY, optional: false, rest: false },
            ParamInfo { name: Some(interner.intern_string("args")), type_id: tuple_one_any, optional: false, rest: true },
        ],
        this_type: None,
        return_type: TypeId::ANY,
        type_predicate: None,
        is_constructor: false,
    });

    // Target: (name: string, mixed: any, ...args: any[]) => any
    let any_array = interner.array(TypeId::ANY);
    let target = interner.function(FunctionShape {
        type_params: vec![],
        params: vec![
            ParamInfo { name: Some(interner.intern_string("name")), type_id: TypeId::STRING, optional: false, rest: false },
            ParamInfo { name: Some(interner.intern_string("mixed")), type_id: TypeId::ANY, optional: false, rest: false },
            ParamInfo { name: Some(interner.intern_string("args")), type_id: any_array, optional: false, rest: true },
        ],
        this_type: None,
        return_type: TypeId::ANY,
        type_predicate: None,
        is_constructor: false,
    });

    // Function with rest tuple should be subtype of function with rest array
    // (name, mixed, ...args: [any]) should be assignable to (name, mixed, ...args: any[])
    assert!(checker.is_subtype_of(source, target), "Function with rest tuple [any] should be subtype of function with rest array any[]");
}

#[test]
fn test_keyof_intersection_contravariant() {
    let interner = TypeInterner::new();
    let mut checker = SubtypeChecker::new(&interner);

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
    let keyof_a = interner.intern(TypeKey::KeyOf(obj_a));
    let keyof_intersection = interner.intern(TypeKey::KeyOf(intersection));

    assert!(checker.is_subtype_of(keyof_a, keyof_intersection));
    assert!(!checker.is_subtype_of(keyof_intersection, keyof_a));
}

#[test]
fn test_keyof_union_index_signature_contravariant() {
    let interner = TypeInterner::new();
    let mut checker = SubtypeChecker::new(&interner);

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
    let keyof_union = interner.intern(TypeKey::KeyOf(union));

    assert!(checker.is_subtype_of(keyof_union, TypeId::NUMBER));
    assert!(!checker.is_subtype_of(keyof_union, TypeId::STRING));
}

#[test]
fn test_keyof_deferred_not_subtype_of_string() {
    let interner = TypeInterner::new();
    let mut checker = SubtypeChecker::new(&interner);

    let type_param = interner.intern(TypeKey::TypeParameter(TypeParamInfo {
        name: interner.intern_string("T"),
        constraint: None,
        default: None,
    }));
    let keyof_param = interner.intern(TypeKey::KeyOf(type_param));

    assert!(!checker.is_subtype_of(keyof_param, TypeId::STRING));
}

#[test]
fn test_keyof_deferred_subtype_of_string_number_symbol_union() {
    let interner = TypeInterner::new();
    let mut checker = SubtypeChecker::new(&interner);

    let type_param = interner.intern(TypeKey::TypeParameter(TypeParamInfo {
        name: interner.intern_string("T"),
        constraint: None,
        default: None,
    }));
    let keyof_param = interner.intern(TypeKey::KeyOf(type_param));

    let key_union = interner.union(vec![TypeId::STRING, TypeId::NUMBER, TypeId::SYMBOL]);
    assert!(checker.is_subtype_of(keyof_param, key_union));
}

#[test]
fn test_keyof_deferred_not_subtype_of_string_number_union() {
    let interner = TypeInterner::new();
    let mut checker = SubtypeChecker::new(&interner);

    let type_param = interner.intern(TypeKey::TypeParameter(TypeParamInfo {
        name: interner.intern_string("T"),
        constraint: None,
        default: None,
    }));
    let keyof_param = interner.intern(TypeKey::KeyOf(type_param));

    let key_union = interner.union(vec![TypeId::STRING, TypeId::NUMBER]);
    assert!(!checker.is_subtype_of(keyof_param, key_union));
}
