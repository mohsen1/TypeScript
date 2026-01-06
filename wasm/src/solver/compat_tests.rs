use super::*;

#[test]
fn test_any_assignability() {
    let interner = TypeInterner::new();
    let mut checker = CompatChecker::new(&interner);

    assert!(checker.is_assignable(TypeId::ANY, TypeId::STRING));
    assert!(checker.is_assignable(TypeId::STRING, TypeId::ANY));
}

#[test]
fn test_unknown_assignability() {
    let interner = TypeInterner::new();
    let mut checker = CompatChecker::new(&interner);

    assert!(checker.is_assignable(TypeId::STRING, TypeId::UNKNOWN));
    assert!(checker.is_assignable(TypeId::UNKNOWN, TypeId::ANY));
    assert!(checker.is_assignable(TypeId::UNKNOWN, TypeId::UNKNOWN));
    assert!(!checker.is_assignable(TypeId::UNKNOWN, TypeId::STRING));
}
