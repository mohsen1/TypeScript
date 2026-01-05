use super::*;

#[test]
fn test_intrinsic_type_ids() {
    assert_eq!(TypeId::ANY.0, 4);
    assert_eq!(TypeId::STRING.0, 10);
    assert_eq!(TypeId::NUMBER.0, 9);
}

#[test]
fn test_lowering_new() {
    let arena = ThinNodeArena::new();
    let interner = TypeInterner::new();
    let _lowering = TypeLowering::new(&arena, &interner);
}
