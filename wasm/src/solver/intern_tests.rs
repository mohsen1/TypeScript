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
}

#[test]
fn test_interner_object_sorting() {
    let interner = TypeInterner::new();
    use std::sync::Arc;

    // Properties in different order should produce same TypeId
    let props1 = vec![
        PropertyInfo { name: interner.intern_string("a"), type_id: TypeId::STRING, optional: false, readonly: false },
        PropertyInfo { name: interner.intern_string("b"), type_id: TypeId::NUMBER, optional: false, readonly: false },
    ];
    let props2 = vec![
        PropertyInfo { name: interner.intern_string("b"), type_id: TypeId::NUMBER, optional: false, readonly: false },
        PropertyInfo { name: interner.intern_string("a"), type_id: TypeId::STRING, optional: false, readonly: false },
    ];

    let id1 = interner.object(props1);
    let id2 = interner.object(props2);

    assert_eq!(id1, id2);
}
