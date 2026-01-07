use super::*;

#[test]
fn test_interner_intrinsics() {
    let interner = TypeInterner::new();

    // Intrinsics should be pre-registered
    assert!(interner.lookup(TypeId::STRING).is_some());
    assert!(interner.lookup(TypeId::NUMBER).is_some());
    assert!(interner.lookup(TypeId::ANY).is_some());
}

#[test]
fn test_interner_deduplication() {
    let interner = TypeInterner::new();

    // Same structure should get same TypeId
    let id1 = interner.literal_string("hello");
    let id2 = interner.literal_string("hello");
    let id3 = interner.literal_string("world");

    assert_eq!(id1, id2);
    assert_ne!(id1, id3);
}

#[test]
fn test_interner_bigint_literal() {
    let interner = TypeInterner::new();

    let id = interner.literal_bigint("123");
    let key = interner.lookup(id).expect("bigint literal should be interned");

    match key {
        TypeKey::Literal(LiteralValue::BigInt(atom)) => {
            assert_eq!(interner.resolve_atom(atom), "123");
        }
        _ => panic!("Expected bigint literal, got {:?}", key),
    }
}

#[test]
fn test_interner_union_normalization() {
    let interner = TypeInterner::new();

    // Union with single member should return that member
    let single = interner.union(vec![TypeId::STRING]);
    assert_eq!(single, TypeId::STRING);

    // Union with `any` should be `any`
    let with_any = interner.union(vec![TypeId::STRING, TypeId::ANY]);
    assert_eq!(with_any, TypeId::ANY);

    // Union with `never` should exclude `never`
    let with_never = interner.union(vec![TypeId::STRING, TypeId::NEVER]);
    assert_eq!(with_never, TypeId::STRING);

    // Empty union is `never`
    let empty = interner.union(vec![]);
    assert_eq!(empty, TypeId::NEVER);

    // Union with `error` should be `error`
    let with_error = interner.union(vec![TypeId::STRING, TypeId::ERROR]);
    assert_eq!(with_error, TypeId::ERROR);
}

#[test]
fn test_interner_intersection_normalization() {
    let interner = TypeInterner::new();

    // Intersection with single member should return that member
    let single = interner.intersection(vec![TypeId::STRING]);
    assert_eq!(single, TypeId::STRING);

    // Intersection with `never` should be `never`
    let with_never = interner.intersection(vec![TypeId::STRING, TypeId::NEVER]);
    assert_eq!(with_never, TypeId::NEVER);

    // Empty intersection is `unknown`
    let empty = interner.intersection(vec![]);
    assert_eq!(empty, TypeId::UNKNOWN);

    // Intersection with `any` should be `any`
    let with_any = interner.intersection(vec![TypeId::STRING, TypeId::ANY]);
    assert_eq!(with_any, TypeId::ANY);

    // Intersection with `error` should be `error`
    let with_error = interner.intersection(vec![TypeId::STRING, TypeId::ERROR]);
    assert_eq!(with_error, TypeId::ERROR);
}

#[test]
fn test_interner_intersection_disjoint_primitives() {
    let interner = TypeInterner::new();

    let disjoint = interner.intersection(vec![TypeId::STRING, TypeId::NUMBER]);
    assert_eq!(disjoint, TypeId::NEVER);

    let literal = interner.literal_string("a");
    let disjoint_literal = interner.intersection(vec![literal, TypeId::BOOLEAN]);
    assert_eq!(disjoint_literal, TypeId::NEVER);
}

#[test]
fn test_interner_object_sorting() {
    let interner = TypeInterner::new();
    use std::sync::Arc;

    // Properties in different order should produce same TypeId
    let props1 = vec![
        PropertyInfo { name: interner.intern_string("a"), type_id: TypeId::STRING, optional: false, readonly: false, is_method: false },
        PropertyInfo { name: interner.intern_string("b"), type_id: TypeId::NUMBER, optional: false, readonly: false, is_method: false },
    ];
    let props2 = vec![
        PropertyInfo { name: interner.intern_string("b"), type_id: TypeId::NUMBER, optional: false, readonly: false, is_method: false },
        PropertyInfo { name: interner.intern_string("a"), type_id: TypeId::STRING, optional: false, readonly: false, is_method: false },
    ];

    let id1 = interner.object(props1);
    let id2 = interner.object(props2);

    assert_eq!(id1, id2);
}

#[test]
fn test_interner_application_deduplication() {
    let interner = TypeInterner::new();

    let base = interner.reference(SymbolRef(1));
    let app1 = interner.application(base, vec![TypeId::STRING]);
    let app2 = interner.application(base, vec![TypeId::STRING]);
    let app3 = interner.application(base, vec![TypeId::NUMBER]);

    assert_eq!(app1, app2);
    assert_ne!(app1, app3);
}
