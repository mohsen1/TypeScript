use super::*;
use crate::solver::types::*;

fn make_animal_dog(interner: &TypeInterner) -> (TypeId, TypeId) {
    let name = interner.intern_string("name");
    let breed = interner.intern_string("breed");

    let animal = interner.object(vec![PropertyInfo {
        name,
        type_id: TypeId::STRING,
        optional: false,
        readonly: false,
    }]);

    let dog = interner.object(vec![
        PropertyInfo {
            name,
            type_id: TypeId::STRING,
            optional: false,
            readonly: false,
        },
        PropertyInfo {
            name: breed,
            type_id: TypeId::STRING,
            optional: false,
            readonly: false,
        },
    ]);

    (animal, dog)
}

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

#[test]
fn test_function_bivariance_default() {
    let interner = TypeInterner::new();
    let mut checker = CompatChecker::new(&interner);

    let (animal, dog) = make_animal_dog(&interner);

    let fn_dog = interner.function(FunctionShape {
        params: vec![ParamInfo {
            name: None,
            type_id: dog,
            optional: false,
            rest: false,
        }],
        return_type: TypeId::VOID,
        type_params: Vec::new(),
        is_constructor: false,
    });

    let fn_animal = interner.function(FunctionShape {
        params: vec![ParamInfo {
            name: None,
            type_id: animal,
            optional: false,
            rest: false,
        }],
        return_type: TypeId::VOID,
        type_params: Vec::new(),
        is_constructor: false,
    });

    assert!(checker.is_assignable(fn_dog, fn_animal));
}

#[test]
fn test_function_variance_strict() {
    let interner = TypeInterner::new();
    let mut checker = CompatChecker::new(&interner);
    checker.set_strict_function_types(true);

    let (animal, dog) = make_animal_dog(&interner);

    let fn_dog = interner.function(FunctionShape {
        params: vec![ParamInfo {
            name: None,
            type_id: dog,
            optional: false,
            rest: false,
        }],
        return_type: TypeId::VOID,
        type_params: Vec::new(),
        is_constructor: false,
    });

    let fn_animal = interner.function(FunctionShape {
        params: vec![ParamInfo {
            name: None,
            type_id: animal,
            optional: false,
            rest: false,
        }],
        return_type: TypeId::VOID,
        type_params: Vec::new(),
        is_constructor: false,
    });

    assert!(!checker.is_assignable(fn_dog, fn_animal));
}

#[test]
fn test_function_return_covariance() {
    let interner = TypeInterner::new();
    let mut checker = CompatChecker::new(&interner);

    let (animal, dog) = make_animal_dog(&interner);

    let returns_dog = interner.function(FunctionShape {
        params: Vec::new(),
        return_type: dog,
        type_params: Vec::new(),
        is_constructor: false,
    });

    let returns_animal = interner.function(FunctionShape {
        params: Vec::new(),
        return_type: animal,
        type_params: Vec::new(),
        is_constructor: false,
    });

    assert!(checker.is_assignable(returns_dog, returns_animal));
    assert!(!checker.is_assignable(returns_animal, returns_dog));
}

#[test]
fn test_void_return_assignability() {
    let interner = TypeInterner::new();
    let mut checker = CompatChecker::new(&interner);

    let returns_number = interner.function(FunctionShape {
        params: Vec::new(),
        return_type: TypeId::NUMBER,
        type_params: Vec::new(),
        is_constructor: false,
    });

    let returns_void = interner.function(FunctionShape {
        params: Vec::new(),
        return_type: TypeId::VOID,
        type_params: Vec::new(),
        is_constructor: false,
    });

    assert!(checker.is_assignable(returns_number, returns_void));
    assert!(!checker.is_assignable(returns_void, returns_number));
}
