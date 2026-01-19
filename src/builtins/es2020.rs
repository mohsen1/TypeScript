//! ES2020+ built-in type declarations
//!
//! This module provides ES2020 and later built-in types:
//! - BigInt and BigIntConstructor
//! - globalThis
//! - Promise.allSettled, Promise.any
//! - String.matchAll
//! - Optional chaining / Nullish coalescing types

use super::types::*;

/// Get all ES2020+ built-in declarations
pub fn get_es2020_declarations() -> LibDeclarations {
    let mut lib = LibDeclarations::new();

    // BigInt
    lib.interfaces.push(bigint_interface());
    lib.interfaces.push(bigint_constructor_interface());

    // Promise extensions
    lib.interfaces.push(promise_fulfilled_result_interface());
    lib.interfaces.push(promise_rejected_result_interface());

    // Type aliases
    lib.type_aliases.push(TypeAliasDeclaration {
        name: "PromiseSettledResult",
        type_parameters: vec![TypeParameter::new("T")],
        type_: Type::union(vec![
            Type::reference1("PromiseFulfilledResult", Type::type_param("T")),
            Type::reference("PromiseRejectedResult"),
        ]),
    });

    // Awaited type (recursive promise unwrapping)
    lib.type_aliases.push(TypeAliasDeclaration {
        name: "Awaited",
        type_parameters: vec![TypeParameter::new("T")],
        type_: Type::Conditional {
            check_type: Box::new(Type::type_param("T")),
            extends_type: Box::new(Type::nullable(Type::Undefined)),
            true_type: Box::new(Type::type_param("T")),
            false_type: Box::new(Type::Conditional {
                check_type: Box::new(Type::type_param("T")),
                extends_type: Box::new(Type::reference1("PromiseLike", Type::Intrinsic("infer U"))),
                true_type: Box::new(Type::reference1("Awaited", Type::type_param("U"))),
                false_type: Box::new(Type::type_param("T")),
            }),
        },
    });

    // String extensions
    lib.interfaces.push(string_es2020_interface());
    lib.interfaces.push(regexp_string_iterator_interface());

    // WeakRef and FinalizationRegistry
    lib.interfaces.push(weak_ref_interface());
    lib.interfaces.push(weak_ref_constructor_interface());
    lib.interfaces.push(finalization_registry_interface());
    lib.interfaces.push(finalization_registry_constructor_interface());

    // Global variables
    lib.variables.push(VariableDeclaration {
        name: "globalThis",
        type_: Type::reference("typeof globalThis"),
        readonly: true,
    });
    lib.variables.push(VariableDeclaration {
        name: "BigInt",
        type_: Type::reference("BigIntConstructor"),
        readonly: true,
    });
    lib.variables.push(VariableDeclaration {
        name: "WeakRef",
        type_: Type::reference("WeakRefConstructor"),
        readonly: true,
    });
    lib.variables.push(VariableDeclaration {
        name: "FinalizationRegistry",
        type_: Type::reference("FinalizationRegistryConstructor"),
        readonly: true,
    });

    lib
}

fn bigint_interface() -> InterfaceDeclaration {
    InterfaceDeclaration {
        name: "BigInt",
        type_parameters: vec![],
        extends: vec![],
        properties: vec![],
        methods: vec![
            MethodSignature::new(
                "toString",
                vec![ParameterDeclaration::optional("radix", Type::Number)],
                Type::String,
            ),
            MethodSignature::new("toLocaleString", vec![
                ParameterDeclaration::optional("locales", Type::union(vec![Type::String, Type::array(Type::String)])),
                ParameterDeclaration::optional("options", Type::reference("BigIntToLocaleStringOptions")),
            ], Type::String),
            MethodSignature::new("valueOf", vec![], Type::BigInt),
        ],
        call_signatures: vec![],
        construct_signatures: vec![],
        index_signatures: vec![],
    }
}

fn bigint_constructor_interface() -> InterfaceDeclaration {
    InterfaceDeclaration {
        name: "BigIntConstructor",
        type_parameters: vec![],
        extends: vec![],
        properties: vec![
            PropertySignature::readonly("prototype", Type::BigInt),
        ],
        methods: vec![
            MethodSignature::new(
                "asIntN",
                vec![
                    ParameterDeclaration::new("bits", Type::Number),
                    ParameterDeclaration::new("int", Type::BigInt),
                ],
                Type::BigInt,
            ),
            MethodSignature::new(
                "asUintN",
                vec![
                    ParameterDeclaration::new("bits", Type::Number),
                    ParameterDeclaration::new("int", Type::BigInt),
                ],
                Type::BigInt,
            ),
        ],
        call_signatures: vec![CallSignature {
            type_parameters: vec![],
            parameters: vec![ParameterDeclaration::new("value", Type::union(vec![
                Type::BigInt,
                Type::Boolean,
                Type::Number,
                Type::String,
            ]))],
            return_type: Type::BigInt,
        }],
        construct_signatures: vec![], // BigInt is not constructable with new
        index_signatures: vec![],
    }
}

fn promise_fulfilled_result_interface() -> InterfaceDeclaration {
    InterfaceDeclaration {
        name: "PromiseFulfilledResult",
        type_parameters: vec![TypeParameter::new("T")],
        extends: vec![],
        properties: vec![
            PropertySignature::new("status", Type::StringLiteral("fulfilled")),
            PropertySignature::new("value", Type::type_param("T")),
        ],
        methods: vec![],
        call_signatures: vec![],
        construct_signatures: vec![],
        index_signatures: vec![],
    }
}

fn promise_rejected_result_interface() -> InterfaceDeclaration {
    InterfaceDeclaration {
        name: "PromiseRejectedResult",
        type_parameters: vec![],
        extends: vec![],
        properties: vec![
            PropertySignature::new("status", Type::StringLiteral("rejected")),
            PropertySignature::new("reason", Type::Any),
        ],
        methods: vec![],
        call_signatures: vec![],
        construct_signatures: vec![],
        index_signatures: vec![],
    }
}

fn string_es2020_interface() -> InterfaceDeclaration {
    InterfaceDeclaration {
        name: "String",
        type_parameters: vec![],
        extends: vec![],
        properties: vec![],
        methods: vec![
            MethodSignature::new(
                "matchAll",
                vec![ParameterDeclaration::new("regexp", Type::reference("RegExp"))],
                Type::reference1("IterableIterator", Type::reference("RegExpMatchArray")),
            ),
            MethodSignature::new(
                "replaceAll",
                vec![
                    ParameterDeclaration::new("searchValue", Type::union(vec![Type::String, Type::reference("RegExp")])),
                    ParameterDeclaration::new("replaceValue", Type::String),
                ],
                Type::String,
            ),
        ],
        call_signatures: vec![],
        construct_signatures: vec![],
        index_signatures: vec![],
    }
}

fn regexp_string_iterator_interface() -> InterfaceDeclaration {
    InterfaceDeclaration {
        name: "RegExpStringIterator",
        type_parameters: vec![TypeParameter::new("T")],
        extends: vec![Type::reference1("IterableIterator", Type::type_param("T"))],
        properties: vec![],
        methods: vec![],
        call_signatures: vec![],
        construct_signatures: vec![],
        index_signatures: vec![],
    }
}

fn weak_ref_interface() -> InterfaceDeclaration {
    InterfaceDeclaration {
        name: "WeakRef",
        type_parameters: vec![TypeParameter::with_constraint("T", Type::reference("WeakKey"))],
        extends: vec![],
        properties: vec![],
        methods: vec![
            MethodSignature::new(
                "deref",
                vec![],
                Type::optional(Type::type_param("T")),
            ),
        ],
        call_signatures: vec![],
        construct_signatures: vec![],
        index_signatures: vec![],
    }
}

fn weak_ref_constructor_interface() -> InterfaceDeclaration {
    InterfaceDeclaration {
        name: "WeakRefConstructor",
        type_parameters: vec![],
        extends: vec![],
        properties: vec![
            PropertySignature::readonly("prototype", Type::reference1("WeakRef", Type::Any)),
        ],
        methods: vec![],
        call_signatures: vec![],
        construct_signatures: vec![ConstructSignature {
            type_parameters: vec![TypeParameter::with_constraint("T", Type::reference("WeakKey"))],
            parameters: vec![ParameterDeclaration::new("target", Type::type_param("T"))],
            return_type: Type::reference1("WeakRef", Type::type_param("T")),
        }],
        index_signatures: vec![],
    }
}

fn finalization_registry_interface() -> InterfaceDeclaration {
    InterfaceDeclaration {
        name: "FinalizationRegistry",
        type_parameters: vec![TypeParameter::new("T")],
        extends: vec![],
        properties: vec![],
        methods: vec![
            MethodSignature::new(
                "register",
                vec![
                    ParameterDeclaration::new("target", Type::reference("WeakKey")),
                    ParameterDeclaration::new("heldValue", Type::type_param("T")),
                    ParameterDeclaration::optional("unregisterToken", Type::reference("WeakKey")),
                ],
                Type::Void,
            ),
            MethodSignature::new(
                "unregister",
                vec![ParameterDeclaration::new("unregisterToken", Type::reference("WeakKey"))],
                Type::Boolean,
            ),
        ],
        call_signatures: vec![],
        construct_signatures: vec![],
        index_signatures: vec![],
    }
}

fn finalization_registry_constructor_interface() -> InterfaceDeclaration {
    InterfaceDeclaration {
        name: "FinalizationRegistryConstructor",
        type_parameters: vec![],
        extends: vec![],
        properties: vec![
            PropertySignature::readonly("prototype", Type::reference1("FinalizationRegistry", Type::Any)),
        ],
        methods: vec![],
        call_signatures: vec![],
        construct_signatures: vec![ConstructSignature {
            type_parameters: vec![TypeParameter::new("T")],
            parameters: vec![ParameterDeclaration::new(
                "cleanupCallback",
                Type::func(
                    vec![ParameterDeclaration::new("heldValue", Type::type_param("T"))],
                    Type::Void,
                ),
            )],
            return_type: Type::reference1("FinalizationRegistry", Type::type_param("T")),
        }],
        index_signatures: vec![],
    }
}

/// ES2021+ utility types
pub fn get_es2021_declarations() -> LibDeclarations {
    let mut lib = LibDeclarations::new();

    // String extensions
    lib.interfaces.push(InterfaceDeclaration {
        name: "String",
        type_parameters: vec![],
        extends: vec![],
        properties: vec![],
        methods: vec![
            // replaceAll already in ES2020
        ],
        call_signatures: vec![],
        construct_signatures: vec![],
        index_signatures: vec![],
    });

    // Promise.any (already in ES2015 Promise constructor)
    // AggregateError
    lib.interfaces.push(aggregate_error_interface());
    lib.interfaces.push(aggregate_error_constructor_interface());

    lib.variables.push(VariableDeclaration {
        name: "AggregateError",
        type_: Type::reference("AggregateErrorConstructor"),
        readonly: true,
    });

    lib
}

fn aggregate_error_interface() -> InterfaceDeclaration {
    InterfaceDeclaration {
        name: "AggregateError",
        type_parameters: vec![],
        extends: vec![Type::reference("Error")],
        properties: vec![
            PropertySignature::new("errors", Type::array(Type::Any)),
        ],
        methods: vec![],
        call_signatures: vec![],
        construct_signatures: vec![],
        index_signatures: vec![],
    }
}

fn aggregate_error_constructor_interface() -> InterfaceDeclaration {
    InterfaceDeclaration {
        name: "AggregateErrorConstructor",
        type_parameters: vec![],
        extends: vec![],
        properties: vec![
            PropertySignature::readonly("prototype", Type::reference("AggregateError")),
        ],
        methods: vec![],
        call_signatures: vec![CallSignature {
            type_parameters: vec![],
            parameters: vec![
                ParameterDeclaration::new("errors", Type::reference1("Iterable", Type::Any)),
                ParameterDeclaration::optional("message", Type::String),
            ],
            return_type: Type::reference("AggregateError"),
        }],
        construct_signatures: vec![ConstructSignature {
            type_parameters: vec![],
            parameters: vec![
                ParameterDeclaration::new("errors", Type::reference1("Iterable", Type::Any)),
                ParameterDeclaration::optional("message", Type::String),
            ],
            return_type: Type::reference("AggregateError"),
        }],
        index_signatures: vec![],
    }
}

/// ES2022+ utility types
pub fn get_es2022_declarations() -> LibDeclarations {
    let mut lib = LibDeclarations::new();

    // Array.at, String.at (already defined in ES2015)
    // Object.hasOwn
    lib.interfaces.push(InterfaceDeclaration {
        name: "ObjectConstructor",
        type_parameters: vec![],
        extends: vec![],
        properties: vec![],
        methods: vec![
            MethodSignature::new(
                "hasOwn",
                vec![
                    ParameterDeclaration::new("o", Type::Object),
                    ParameterDeclaration::new("v", Type::union(vec![Type::String, Type::Number, Type::Symbol])),
                ],
                Type::Boolean,
            ),
        ],
        call_signatures: vec![],
        construct_signatures: vec![],
        index_signatures: vec![],
    });

    // Error.cause
    lib.interfaces.push(InterfaceDeclaration {
        name: "Error",
        type_parameters: vec![],
        extends: vec![],
        properties: vec![
            PropertySignature::optional("cause", Type::Unknown),
        ],
        methods: vec![],
        call_signatures: vec![],
        construct_signatures: vec![],
        index_signatures: vec![],
    });

    // ErrorOptions
    lib.interfaces.push(InterfaceDeclaration {
        name: "ErrorOptions",
        type_parameters: vec![],
        extends: vec![],
        properties: vec![
            PropertySignature::optional("cause", Type::Unknown),
        ],
        methods: vec![],
        call_signatures: vec![],
        construct_signatures: vec![],
        index_signatures: vec![],
    });

    lib
}

/// ES2023+ utility types
pub fn get_es2023_declarations() -> LibDeclarations {
    let mut lib = LibDeclarations::new();

    // Array findLast, findLastIndex
    lib.interfaces.push(InterfaceDeclaration {
        name: "Array",
        type_parameters: vec![TypeParameter::new("T")],
        extends: vec![],
        properties: vec![],
        methods: vec![
            MethodSignature::new(
                "findLast",
                vec![
                    ParameterDeclaration::new("predicate", Type::func(
                        vec![
                            ParameterDeclaration::new("value", Type::type_param("T")),
                            ParameterDeclaration::new("index", Type::Number),
                            ParameterDeclaration::new("array", Type::array(Type::type_param("T"))),
                        ],
                        Type::Unknown,
                    )),
                    ParameterDeclaration::optional("thisArg", Type::Any),
                ],
                Type::optional(Type::type_param("T")),
            ),
            MethodSignature::new(
                "findLastIndex",
                vec![
                    ParameterDeclaration::new("predicate", Type::func(
                        vec![
                            ParameterDeclaration::new("value", Type::type_param("T")),
                            ParameterDeclaration::new("index", Type::Number),
                            ParameterDeclaration::new("array", Type::array(Type::type_param("T"))),
                        ],
                        Type::Unknown,
                    )),
                    ParameterDeclaration::optional("thisArg", Type::Any),
                ],
                Type::Number,
            ),
            MethodSignature::new("toReversed", vec![], Type::array(Type::type_param("T"))),
            MethodSignature::new(
                "toSorted",
                vec![ParameterDeclaration::optional("compareFn", Type::func(
                    vec![
                        ParameterDeclaration::new("a", Type::type_param("T")),
                        ParameterDeclaration::new("b", Type::type_param("T")),
                    ],
                    Type::Number,
                ))],
                Type::array(Type::type_param("T")),
            ),
            MethodSignature::new(
                "toSpliced",
                vec![
                    ParameterDeclaration::new("start", Type::Number),
                    ParameterDeclaration::new("deleteCount", Type::Number),
                    ParameterDeclaration::rest("items", Type::array(Type::type_param("T"))),
                ],
                Type::array(Type::type_param("T")),
            ),
            MethodSignature::new(
                "with",
                vec![
                    ParameterDeclaration::new("index", Type::Number),
                    ParameterDeclaration::new("value", Type::type_param("T")),
                ],
                Type::array(Type::type_param("T")),
            ),
        ],
        call_signatures: vec![],
        construct_signatures: vec![],
        index_signatures: vec![],
    });

    lib
}
