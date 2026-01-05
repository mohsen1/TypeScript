use super::*;

#[test]
fn test_type_id_intrinsics() {
    assert!(TypeId::ANY.is_intrinsic());
    assert!(TypeId::STRING.is_intrinsic());
    assert!(!TypeId(100).is_intrinsic());
    assert!(!TypeId(1000).is_intrinsic());
}

#[test]
fn test_type_id_equality() {
    // O(1) equality check
    let a = TypeId(42);
    let b = TypeId(42);
    let c = TypeId(43);

    assert_eq!(a, b);
    assert_ne!(a, c);
}

#[test]
fn test_ordered_float_hash() {
    use std::collections::HashSet;

    let mut set = HashSet::new();
    set.insert(OrderedFloat(1.5));
    set.insert(OrderedFloat(2.5));
    set.insert(OrderedFloat(1.5)); // duplicate

    assert_eq!(set.len(), 2);
}
