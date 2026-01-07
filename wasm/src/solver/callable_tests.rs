//! Tests for callable type (overloaded signatures) subtype checking.

use super::*;
use std::sync::Arc;

// =============================================================================
// Callable Subtype Tests
// =============================================================================

#[test]
fn test_callable_same_signature() {
    let interner = TypeInterner::new();

    // { (x: string): number } <: { (x: string): number }
    let sig = CallSignature {
        type_params: vec![],
        params: vec![ParamInfo {
            name: Some(interner.intern_string("x")),
            type_id: TypeId::STRING,
            optional: false,
            rest: false,
        }],
        this_type: None,
        return_type: TypeId::NUMBER,
        type_predicate: None,
    };

    let source = interner.callable(CallableShape {
        call_signatures: vec![sig.clone()],
        construct_signatures: vec![],
        properties: vec![],
    });

    let target = interner.callable(CallableShape {
        call_signatures: vec![sig],
        construct_signatures: vec![],
        properties: vec![],
    });

    assert!(is_subtype_of(&interner, source, target));
}

#[test]
fn test_callable_more_overloads() {
    let interner = TypeInterner::new();

    // { (x: string): number; (x: number): string } <: { (x: string): number }
    let sig1 = CallSignature {
        type_params: vec![],
        params: vec![ParamInfo {
            name: Some(interner.intern_string("x")),
            type_id: TypeId::STRING,
            optional: false,
            rest: false,
        }],
        this_type: None,
        return_type: TypeId::NUMBER,
        type_predicate: None,
    };

    let sig2 = CallSignature {
        type_params: vec![],
        params: vec![ParamInfo {
            name: Some(interner.intern_string("x")),
            type_id: TypeId::NUMBER,
            optional: false,
            rest: false,
        }],
        this_type: None,
        return_type: TypeId::STRING,
        type_predicate: None,
    };

    let source = interner.callable(CallableShape {
        call_signatures: vec![sig1.clone(), sig2],
        construct_signatures: vec![],
        properties: vec![],
    });

    let target = interner.callable(CallableShape {
        call_signatures: vec![sig1],
        construct_signatures: vec![],
        properties: vec![],
    });

    assert!(is_subtype_of(&interner, source, target));
}

#[test]
fn test_callable_missing_overload() {
    let interner = TypeInterner::new();

    // { (x: string): number } NOT <: { (x: string): number; (x: number): string }
    let sig1 = CallSignature {
        type_params: vec![],
        params: vec![ParamInfo {
            name: Some(interner.intern_string("x")),
            type_id: TypeId::STRING,
            optional: false,
            rest: false,
        }],
        this_type: None,
        return_type: TypeId::NUMBER,
        type_predicate: None,
    };

    let sig2 = CallSignature {
        type_params: vec![],
        params: vec![ParamInfo {
            name: Some(interner.intern_string("x")),
            type_id: TypeId::NUMBER,
            optional: false,
            rest: false,
        }],
        this_type: None,
        return_type: TypeId::STRING,
        type_predicate: None,
    };

    let source = interner.callable(CallableShape {
        call_signatures: vec![sig1.clone()],
        construct_signatures: vec![],
        properties: vec![],
    });

    let target = interner.callable(CallableShape {
        call_signatures: vec![sig1, sig2],
        construct_signatures: vec![],
        properties: vec![],
    });

    assert!(!is_subtype_of(&interner, source, target));
}

#[test]
fn test_callable_with_construct() {
    let interner = TypeInterner::new();

    // { new(): Foo } <: { new(): Foo }
    let obj_type = interner.object(vec![
        PropertyInfo { name: interner.intern_string("x"), type_id: TypeId::NUMBER,
 write_type: TypeId::NUMBER, optional: false, readonly: false, is_method: false },
    ]);

    let sig = CallSignature {
        type_params: vec![],
        params: vec![],
        this_type: None,
        return_type: obj_type,
        type_predicate: None,
    };

    let source = interner.callable(CallableShape {
        call_signatures: vec![],
        construct_signatures: vec![sig.clone()],
        properties: vec![],
    });

    let target = interner.callable(CallableShape {
        call_signatures: vec![],
        construct_signatures: vec![sig],
        properties: vec![],
    });

    assert!(is_subtype_of(&interner, source, target));
}

#[test]
fn test_callable_covariant_return() {
    let interner = TypeInterner::new();

    // { (): "hello" } <: { (): string }
    let hello = interner.literal_string("hello");

    let source_sig = CallSignature {
        type_params: vec![],
        params: vec![],
        this_type: None,
        return_type: hello,
        type_predicate: None,
    };

    let target_sig = CallSignature {
        type_params: vec![],
        params: vec![],
        this_type: None,
        return_type: TypeId::STRING,
        type_predicate: None,
    };

    let source = interner.callable(CallableShape {
        call_signatures: vec![source_sig],
        construct_signatures: vec![],
        properties: vec![],
    });

    let target = interner.callable(CallableShape {
        call_signatures: vec![target_sig],
        construct_signatures: vec![],
        properties: vec![],
    });

    assert!(is_subtype_of(&interner, source, target));
}

#[test]
fn test_function_to_callable() {
    let interner = TypeInterner::new();

    // (x: string) => number <: { (x: string): number }
    let fn_type = interner.function(FunctionShape {
        type_params: vec![],
        params: vec![ParamInfo {
            name: Some(interner.intern_string("x")),
            type_id: TypeId::STRING,
            optional: false,
            rest: false,
        }],
        this_type: None,
        return_type: TypeId::NUMBER,
        type_predicate: None,
        is_constructor: false,
    });

    let callable = interner.callable(CallableShape {
        call_signatures: vec![CallSignature {
            type_params: vec![],
            params: vec![ParamInfo {
                name: Some(interner.intern_string("x")),
                type_id: TypeId::STRING,
                optional: false,
                rest: false,
            }],
            this_type: None,
            return_type: TypeId::NUMBER,
            type_predicate: None,
        }],
        construct_signatures: vec![],
        properties: vec![],
    });

    assert!(is_subtype_of(&interner, fn_type, callable));
}

#[test]
fn test_callable_to_function() {
    let interner = TypeInterner::new();

    // { (x: string): number } <: (x: string) => number
    // At least one signature must match
    let callable = interner.callable(CallableShape {
        call_signatures: vec![CallSignature {
            type_params: vec![],
            params: vec![ParamInfo {
                name: Some(interner.intern_string("x")),
                type_id: TypeId::STRING,
                optional: false,
                rest: false,
            }],
            this_type: None,
            return_type: TypeId::NUMBER,
            type_predicate: None,
        }],
        construct_signatures: vec![],
        properties: vec![],
    });

    let fn_type = interner.function(FunctionShape {
        type_params: vec![],
        params: vec![ParamInfo {
            name: Some(interner.intern_string("x")),
            type_id: TypeId::STRING,
            optional: false,
            rest: false,
        }],
        this_type: None,
        return_type: TypeId::NUMBER,
        type_predicate: None,
        is_constructor: false,
    });

    assert!(is_subtype_of(&interner, callable, fn_type));
}

#[test]
fn test_callable_with_properties() {
    let interner = TypeInterner::new();

    // { (): void; length: number } <: { (): void; length: number }
    let source = interner.callable(CallableShape {
        call_signatures: vec![CallSignature {
            type_params: vec![],
            params: vec![],
            this_type: None,
            return_type: TypeId::VOID,
            type_predicate: None,
        }],
        construct_signatures: vec![],
        properties: vec![
            PropertyInfo { name: interner.intern_string("length"), type_id: TypeId::NUMBER,
 write_type: TypeId::NUMBER, optional: false, readonly: false, is_method: false },
        ],
    });

    let target = interner.callable(CallableShape {
        call_signatures: vec![CallSignature {
            type_params: vec![],
            params: vec![],
            this_type: None,
            return_type: TypeId::VOID,
            type_predicate: None,
        }],
        construct_signatures: vec![],
        properties: vec![
            PropertyInfo { name: interner.intern_string("length"), type_id: TypeId::NUMBER,
 write_type: TypeId::NUMBER, optional: false, readonly: false, is_method: false },
        ],
    });

    assert!(is_subtype_of(&interner, source, target));
}

#[test]
fn test_callable_missing_property() {
    let interner = TypeInterner::new();

    // { (): void } NOT <: { (): void; length: number }
    let source = interner.callable(CallableShape {
        call_signatures: vec![CallSignature {
            type_params: vec![],
            params: vec![],
            this_type: None,
            return_type: TypeId::VOID,
            type_predicate: None,
        }],
        construct_signatures: vec![],
        properties: vec![],
    });

    let target = interner.callable(CallableShape {
        call_signatures: vec![CallSignature {
            type_params: vec![],
            params: vec![],
            this_type: None,
            return_type: TypeId::VOID,
            type_predicate: None,
        }],
        construct_signatures: vec![],
        properties: vec![
            PropertyInfo { name: interner.intern_string("length"), type_id: TypeId::NUMBER,
 write_type: TypeId::NUMBER, optional: false, readonly: false, is_method: false },
        ],
    });

    assert!(!is_subtype_of(&interner, source, target));
}
