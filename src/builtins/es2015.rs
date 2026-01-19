//! ES2015 (ES6) built-in type declarations
//!
//! This module provides ES2015 built-in types:
//! - Promise
//! - Symbol
//! - Map, Set, WeakMap, WeakSet
//! - Generator, Iterable, Iterator
//! - Proxy, Reflect
//! - Array/Object/Number/String extensions

use super::types::*;

/// Get all ES2015 built-in declarations
pub fn get_es2015_declarations() -> LibDeclarations {
    let mut lib = LibDeclarations::new();

    // Core ES2015 interfaces
    lib.interfaces.push(promise_interface());
    lib.interfaces.push(promise_constructor_interface());
    lib.interfaces.push(promise_like_interface());
    lib.interfaces.push(symbol_interface());
    lib.interfaces.push(symbol_constructor_interface());

    // Collections
    lib.interfaces.push(map_interface());
    lib.interfaces.push(map_constructor_interface());
    lib.interfaces.push(readonly_map_interface());
    lib.interfaces.push(set_interface());
    lib.interfaces.push(set_constructor_interface());
    lib.interfaces.push(readonly_set_interface());
    lib.interfaces.push(weak_map_interface());
    lib.interfaces.push(weak_map_constructor_interface());
    lib.interfaces.push(weak_set_interface());
    lib.interfaces.push(weak_set_constructor_interface());

    // Generators
    lib.interfaces.push(generator_interface());
    lib.interfaces.push(generator_function_interface());
    lib.interfaces.push(generator_function_constructor_interface());
    lib.interfaces.push(iterable_iterator_interface());

    // Proxy and Reflect
    lib.interfaces.push(proxy_handler_interface());
    lib.interfaces.push(proxy_constructor_interface());
    lib.interfaces.push(reflect_namespace());

    // Array extensions
    lib.interfaces.push(array_es2015_interface());

    // Object extensions
    lib.interfaces.push(object_es2015_constructor());

    // String extensions
    lib.interfaces.push(string_es2015_interface());
    lib.interfaces.push(string_es2015_constructor());

    // Number extensions
    lib.interfaces.push(number_es2015_constructor());

    // Math extensions
    lib.interfaces.push(math_es2015_interface());

    // Global variables
    lib.variables.push(VariableDeclaration {
        name: "Promise",
        type_: Type::reference("PromiseConstructor"),
        readonly: true,
    });
    lib.variables.push(VariableDeclaration {
        name: "Symbol",
        type_: Type::reference("SymbolConstructor"),
        readonly: true,
    });
    lib.variables.push(VariableDeclaration {
        name: "Map",
        type_: Type::reference("MapConstructor"),
        readonly: true,
    });
    lib.variables.push(VariableDeclaration {
        name: "Set",
        type_: Type::reference("SetConstructor"),
        readonly: true,
    });
    lib.variables.push(VariableDeclaration {
        name: "WeakMap",
        type_: Type::reference("WeakMapConstructor"),
        readonly: true,
    });
    lib.variables.push(VariableDeclaration {
        name: "WeakSet",
        type_: Type::reference("WeakSetConstructor"),
        readonly: true,
    });
    lib.variables.push(VariableDeclaration {
        name: "Proxy",
        type_: Type::reference("ProxyConstructor"),
        readonly: true,
    });
    lib.variables.push(VariableDeclaration {
        name: "Reflect",
        type_: Type::reference("Reflect"),
        readonly: true,
    });

    lib
}

fn promise_interface() -> InterfaceDeclaration {
    InterfaceDeclaration {
        name: "Promise",
        type_parameters: vec![TypeParameter::new("T")],
        extends: vec![],
        properties: vec![],
        methods: vec![
            MethodSignature::generic(
                "then",
                vec![
                    TypeParameter::with_default("TResult1", Type::type_param("T")),
                    TypeParameter::with_default("TResult2", Type::Never),
                ],
                vec![
                    ParameterDeclaration::optional(
                        "onfulfilled",
                        Type::nullable(Type::func(
                            vec![ParameterDeclaration::new("value", Type::type_param("T"))],
                            Type::union(vec![
                                Type::type_param("TResult1"),
                                Type::reference1("PromiseLike", Type::type_param("TResult1")),
                            ]),
                        )),
                    ),
                    ParameterDeclaration::optional(
                        "onrejected",
                        Type::nullable(Type::func(
                            vec![ParameterDeclaration::new("reason", Type::Any)],
                            Type::union(vec![
                                Type::type_param("TResult2"),
                                Type::reference1("PromiseLike", Type::type_param("TResult2")),
                            ]),
                        )),
                    ),
                ],
                Type::reference1(
                    "Promise",
                    Type::union(vec![Type::type_param("TResult1"), Type::type_param("TResult2")]),
                ),
            ),
            MethodSignature::generic(
                "catch",
                vec![TypeParameter::with_default("TResult", Type::Never)],
                vec![ParameterDeclaration::optional(
                    "onrejected",
                    Type::nullable(Type::func(
                        vec![ParameterDeclaration::new("reason", Type::Any)],
                        Type::union(vec![
                            Type::type_param("TResult"),
                            Type::reference1("PromiseLike", Type::type_param("TResult")),
                        ]),
                    )),
                )],
                Type::reference1(
                    "Promise",
                    Type::union(vec![Type::type_param("T"), Type::type_param("TResult")]),
                ),
            ),
            MethodSignature::new(
                "finally",
                vec![ParameterDeclaration::optional(
                    "onfinally",
                    Type::nullable(Type::func(vec![], Type::Void)),
                )],
                Type::reference1("Promise", Type::type_param("T")),
            ),
        ],
        call_signatures: vec![],
        construct_signatures: vec![],
        index_signatures: vec![],
    }
}

fn promise_constructor_interface() -> InterfaceDeclaration {
    InterfaceDeclaration {
        name: "PromiseConstructor",
        type_parameters: vec![],
        extends: vec![],
        properties: vec![
            PropertySignature::readonly("prototype", Type::reference1("Promise", Type::Any)),
        ],
        methods: vec![
            MethodSignature::generic(
                "all",
                vec![TypeParameter::new("T")],
                vec![ParameterDeclaration::new(
                    "values",
                    Type::reference1("Iterable", Type::union(vec![
                        Type::type_param("T"),
                        Type::reference1("PromiseLike", Type::type_param("T")),
                    ])),
                )],
                Type::reference1("Promise", Type::array(Type::type_param("T"))),
            ),
            MethodSignature::generic(
                "race",
                vec![TypeParameter::new("T")],
                vec![ParameterDeclaration::new(
                    "values",
                    Type::reference1("Iterable", Type::union(vec![
                        Type::type_param("T"),
                        Type::reference1("PromiseLike", Type::type_param("T")),
                    ])),
                )],
                Type::reference1("Promise", Type::type_param("T")),
            ),
            MethodSignature::new(
                "reject",
                vec![ParameterDeclaration::optional("reason", Type::Any)],
                Type::reference1("Promise", Type::Never),
            ),
            MethodSignature::generic(
                "resolve",
                vec![TypeParameter::new("T")],
                vec![ParameterDeclaration::new(
                    "value",
                    Type::union(vec![
                        Type::type_param("T"),
                        Type::reference1("PromiseLike", Type::type_param("T")),
                    ]),
                )],
                Type::reference1("Promise", Type::type_param("T")),
            ),
            MethodSignature::new(
                "resolve",
                vec![],
                Type::reference1("Promise", Type::Void),
            ),
            // ES2020+ additions
            MethodSignature::generic(
                "allSettled",
                vec![TypeParameter::new("T")],
                vec![ParameterDeclaration::new(
                    "values",
                    Type::reference1("Iterable", Type::union(vec![
                        Type::type_param("T"),
                        Type::reference1("PromiseLike", Type::type_param("T")),
                    ])),
                )],
                Type::reference1("Promise", Type::array(Type::reference1("PromiseSettledResult", Type::type_param("T")))),
            ),
            MethodSignature::generic(
                "any",
                vec![TypeParameter::new("T")],
                vec![ParameterDeclaration::new(
                    "values",
                    Type::reference1("Iterable", Type::union(vec![
                        Type::type_param("T"),
                        Type::reference1("PromiseLike", Type::type_param("T")),
                    ])),
                )],
                Type::reference1("Promise", Type::type_param("T")),
            ),
        ],
        call_signatures: vec![],
        construct_signatures: vec![ConstructSignature {
            type_parameters: vec![TypeParameter::new("T")],
            parameters: vec![ParameterDeclaration::new(
                "executor",
                Type::func(
                    vec![
                        ParameterDeclaration::new(
                            "resolve",
                            Type::func(
                                vec![ParameterDeclaration::new(
                                    "value",
                                    Type::union(vec![
                                        Type::type_param("T"),
                                        Type::reference1("PromiseLike", Type::type_param("T")),
                                    ]),
                                )],
                                Type::Void,
                            ),
                        ),
                        ParameterDeclaration::new(
                            "reject",
                            Type::func(
                                vec![ParameterDeclaration::optional("reason", Type::Any)],
                                Type::Void,
                            ),
                        ),
                    ],
                    Type::Void,
                ),
            )],
            return_type: Type::reference1("Promise", Type::type_param("T")),
        }],
        index_signatures: vec![],
    }
}

fn promise_like_interface() -> InterfaceDeclaration {
    InterfaceDeclaration {
        name: "PromiseLike",
        type_parameters: vec![TypeParameter::new("T")],
        extends: vec![],
        properties: vec![],
        methods: vec![MethodSignature::generic(
            "then",
            vec![
                TypeParameter::with_default("TResult1", Type::type_param("T")),
                TypeParameter::with_default("TResult2", Type::Never),
            ],
            vec![
                ParameterDeclaration::optional(
                    "onfulfilled",
                    Type::nullable(Type::func(
                        vec![ParameterDeclaration::new("value", Type::type_param("T"))],
                        Type::union(vec![
                            Type::type_param("TResult1"),
                            Type::reference1("PromiseLike", Type::type_param("TResult1")),
                        ]),
                    )),
                ),
                ParameterDeclaration::optional(
                    "onrejected",
                    Type::nullable(Type::func(
                        vec![ParameterDeclaration::new("reason", Type::Any)],
                        Type::union(vec![
                            Type::type_param("TResult2"),
                            Type::reference1("PromiseLike", Type::type_param("TResult2")),
                        ]),
                    )),
                ),
            ],
            Type::reference1(
                "PromiseLike",
                Type::union(vec![Type::type_param("TResult1"), Type::type_param("TResult2")]),
            ),
        )],
        call_signatures: vec![],
        construct_signatures: vec![],
        index_signatures: vec![],
    }
}

fn symbol_interface() -> InterfaceDeclaration {
    InterfaceDeclaration {
        name: "Symbol",
        type_parameters: vec![],
        extends: vec![],
        properties: vec![
            PropertySignature::readonly("description", Type::optional(Type::String)),
        ],
        methods: vec![
            MethodSignature::new("toString", vec![], Type::String),
            MethodSignature::new("valueOf", vec![], Type::Symbol),
        ],
        call_signatures: vec![],
        construct_signatures: vec![],
        index_signatures: vec![],
    }
}

fn symbol_constructor_interface() -> InterfaceDeclaration {
    InterfaceDeclaration {
        name: "SymbolConstructor",
        type_parameters: vec![],
        extends: vec![],
        properties: vec![
            PropertySignature::readonly("prototype", Type::Symbol),
            PropertySignature::readonly("asyncIterator", Type::Symbol),
            PropertySignature::readonly("hasInstance", Type::Symbol),
            PropertySignature::readonly("isConcatSpreadable", Type::Symbol),
            PropertySignature::readonly("iterator", Type::Symbol),
            PropertySignature::readonly("match", Type::Symbol),
            PropertySignature::readonly("matchAll", Type::Symbol),
            PropertySignature::readonly("replace", Type::Symbol),
            PropertySignature::readonly("search", Type::Symbol),
            PropertySignature::readonly("species", Type::Symbol),
            PropertySignature::readonly("split", Type::Symbol),
            PropertySignature::readonly("toPrimitive", Type::Symbol),
            PropertySignature::readonly("toStringTag", Type::Symbol),
            PropertySignature::readonly("unscopables", Type::Symbol),
        ],
        methods: vec![
            MethodSignature::new(
                "for",
                vec![ParameterDeclaration::new("key", Type::String)],
                Type::Symbol,
            ),
            MethodSignature::new(
                "keyFor",
                vec![ParameterDeclaration::new("sym", Type::Symbol)],
                Type::optional(Type::String),
            ),
        ],
        call_signatures: vec![CallSignature {
            type_parameters: vec![],
            parameters: vec![ParameterDeclaration::optional("description", Type::union(vec![Type::String, Type::Number]))],
            return_type: Type::Symbol,
        }],
        construct_signatures: vec![],
        index_signatures: vec![],
    }
}

fn map_interface() -> InterfaceDeclaration {
    InterfaceDeclaration {
        name: "Map",
        type_parameters: vec![TypeParameter::new("K"), TypeParameter::new("V")],
        extends: vec![],
        properties: vec![
            PropertySignature::readonly("size", Type::Number),
        ],
        methods: vec![
            MethodSignature::new("clear", vec![], Type::Void),
            MethodSignature::new(
                "delete",
                vec![ParameterDeclaration::new("key", Type::type_param("K"))],
                Type::Boolean,
            ),
            MethodSignature::new(
                "forEach",
                vec![
                    ParameterDeclaration::new(
                        "callbackfn",
                        Type::func(
                            vec![
                                ParameterDeclaration::new("value", Type::type_param("V")),
                                ParameterDeclaration::new("key", Type::type_param("K")),
                                ParameterDeclaration::new("map", Type::reference2("Map", Type::type_param("K"), Type::type_param("V"))),
                            ],
                            Type::Void,
                        ),
                    ),
                    ParameterDeclaration::optional("thisArg", Type::Any),
                ],
                Type::Void,
            ),
            MethodSignature::new(
                "get",
                vec![ParameterDeclaration::new("key", Type::type_param("K"))],
                Type::optional(Type::type_param("V")),
            ),
            MethodSignature::new(
                "has",
                vec![ParameterDeclaration::new("key", Type::type_param("K"))],
                Type::Boolean,
            ),
            MethodSignature::new(
                "set",
                vec![
                    ParameterDeclaration::new("key", Type::type_param("K")),
                    ParameterDeclaration::new("value", Type::type_param("V")),
                ],
                Type::reference("this"),
            ),
            MethodSignature::new(
                "entries",
                vec![],
                Type::reference1("IterableIterator", Type::Tuple(vec![Type::type_param("K"), Type::type_param("V")])),
            ),
            MethodSignature::new(
                "keys",
                vec![],
                Type::reference1("IterableIterator", Type::type_param("K")),
            ),
            MethodSignature::new(
                "values",
                vec![],
                Type::reference1("IterableIterator", Type::type_param("V")),
            ),
        ],
        call_signatures: vec![],
        construct_signatures: vec![],
        index_signatures: vec![],
    }
}

fn map_constructor_interface() -> InterfaceDeclaration {
    InterfaceDeclaration {
        name: "MapConstructor",
        type_parameters: vec![],
        extends: vec![],
        properties: vec![
            PropertySignature::readonly("prototype", Type::reference2("Map", Type::Any, Type::Any)),
        ],
        methods: vec![],
        call_signatures: vec![],
        construct_signatures: vec![
            ConstructSignature {
                type_parameters: vec![TypeParameter::new("K"), TypeParameter::new("V")],
                parameters: vec![],
                return_type: Type::reference2("Map", Type::type_param("K"), Type::type_param("V")),
            },
            ConstructSignature {
                type_parameters: vec![TypeParameter::new("K"), TypeParameter::new("V")],
                parameters: vec![ParameterDeclaration::optional(
                    "entries",
                    Type::nullable(Type::reference1("ReadonlyArray", Type::Tuple(vec![Type::type_param("K"), Type::type_param("V")]))),
                )],
                return_type: Type::reference2("Map", Type::type_param("K"), Type::type_param("V")),
            },
        ],
        index_signatures: vec![],
    }
}

fn readonly_map_interface() -> InterfaceDeclaration {
    InterfaceDeclaration {
        name: "ReadonlyMap",
        type_parameters: vec![TypeParameter::new("K"), TypeParameter::new("V")],
        extends: vec![],
        properties: vec![
            PropertySignature::readonly("size", Type::Number),
        ],
        methods: vec![
            MethodSignature::new(
                "forEach",
                vec![
                    ParameterDeclaration::new(
                        "callbackfn",
                        Type::func(
                            vec![
                                ParameterDeclaration::new("value", Type::type_param("V")),
                                ParameterDeclaration::new("key", Type::type_param("K")),
                                ParameterDeclaration::new("map", Type::reference2("ReadonlyMap", Type::type_param("K"), Type::type_param("V"))),
                            ],
                            Type::Void,
                        ),
                    ),
                    ParameterDeclaration::optional("thisArg", Type::Any),
                ],
                Type::Void,
            ),
            MethodSignature::new(
                "get",
                vec![ParameterDeclaration::new("key", Type::type_param("K"))],
                Type::optional(Type::type_param("V")),
            ),
            MethodSignature::new(
                "has",
                vec![ParameterDeclaration::new("key", Type::type_param("K"))],
                Type::Boolean,
            ),
            MethodSignature::new(
                "entries",
                vec![],
                Type::reference1("IterableIterator", Type::Tuple(vec![Type::type_param("K"), Type::type_param("V")])),
            ),
            MethodSignature::new(
                "keys",
                vec![],
                Type::reference1("IterableIterator", Type::type_param("K")),
            ),
            MethodSignature::new(
                "values",
                vec![],
                Type::reference1("IterableIterator", Type::type_param("V")),
            ),
        ],
        call_signatures: vec![],
        construct_signatures: vec![],
        index_signatures: vec![],
    }
}

fn set_interface() -> InterfaceDeclaration {
    InterfaceDeclaration {
        name: "Set",
        type_parameters: vec![TypeParameter::new("T")],
        extends: vec![],
        properties: vec![
            PropertySignature::readonly("size", Type::Number),
        ],
        methods: vec![
            MethodSignature::new(
                "add",
                vec![ParameterDeclaration::new("value", Type::type_param("T"))],
                Type::reference("this"),
            ),
            MethodSignature::new("clear", vec![], Type::Void),
            MethodSignature::new(
                "delete",
                vec![ParameterDeclaration::new("value", Type::type_param("T"))],
                Type::Boolean,
            ),
            MethodSignature::new(
                "forEach",
                vec![
                    ParameterDeclaration::new(
                        "callbackfn",
                        Type::func(
                            vec![
                                ParameterDeclaration::new("value", Type::type_param("T")),
                                ParameterDeclaration::new("value2", Type::type_param("T")),
                                ParameterDeclaration::new("set", Type::reference1("Set", Type::type_param("T"))),
                            ],
                            Type::Void,
                        ),
                    ),
                    ParameterDeclaration::optional("thisArg", Type::Any),
                ],
                Type::Void,
            ),
            MethodSignature::new(
                "has",
                vec![ParameterDeclaration::new("value", Type::type_param("T"))],
                Type::Boolean,
            ),
            MethodSignature::new(
                "entries",
                vec![],
                Type::reference1("IterableIterator", Type::Tuple(vec![Type::type_param("T"), Type::type_param("T")])),
            ),
            MethodSignature::new(
                "keys",
                vec![],
                Type::reference1("IterableIterator", Type::type_param("T")),
            ),
            MethodSignature::new(
                "values",
                vec![],
                Type::reference1("IterableIterator", Type::type_param("T")),
            ),
        ],
        call_signatures: vec![],
        construct_signatures: vec![],
        index_signatures: vec![],
    }
}

fn set_constructor_interface() -> InterfaceDeclaration {
    InterfaceDeclaration {
        name: "SetConstructor",
        type_parameters: vec![],
        extends: vec![],
        properties: vec![
            PropertySignature::readonly("prototype", Type::reference1("Set", Type::Any)),
        ],
        methods: vec![],
        call_signatures: vec![],
        construct_signatures: vec![
            ConstructSignature {
                type_parameters: vec![TypeParameter::new("T")],
                parameters: vec![],
                return_type: Type::reference1("Set", Type::type_param("T")),
            },
            ConstructSignature {
                type_parameters: vec![TypeParameter::new("T")],
                parameters: vec![ParameterDeclaration::optional(
                    "values",
                    Type::nullable(Type::reference1("ReadonlyArray", Type::type_param("T"))),
                )],
                return_type: Type::reference1("Set", Type::type_param("T")),
            },
        ],
        index_signatures: vec![],
    }
}

fn readonly_set_interface() -> InterfaceDeclaration {
    InterfaceDeclaration {
        name: "ReadonlySet",
        type_parameters: vec![TypeParameter::new("T")],
        extends: vec![],
        properties: vec![
            PropertySignature::readonly("size", Type::Number),
        ],
        methods: vec![
            MethodSignature::new(
                "forEach",
                vec![
                    ParameterDeclaration::new(
                        "callbackfn",
                        Type::func(
                            vec![
                                ParameterDeclaration::new("value", Type::type_param("T")),
                                ParameterDeclaration::new("value2", Type::type_param("T")),
                                ParameterDeclaration::new("set", Type::reference1("ReadonlySet", Type::type_param("T"))),
                            ],
                            Type::Void,
                        ),
                    ),
                    ParameterDeclaration::optional("thisArg", Type::Any),
                ],
                Type::Void,
            ),
            MethodSignature::new(
                "has",
                vec![ParameterDeclaration::new("value", Type::type_param("T"))],
                Type::Boolean,
            ),
            MethodSignature::new(
                "entries",
                vec![],
                Type::reference1("IterableIterator", Type::Tuple(vec![Type::type_param("T"), Type::type_param("T")])),
            ),
            MethodSignature::new(
                "keys",
                vec![],
                Type::reference1("IterableIterator", Type::type_param("T")),
            ),
            MethodSignature::new(
                "values",
                vec![],
                Type::reference1("IterableIterator", Type::type_param("T")),
            ),
        ],
        call_signatures: vec![],
        construct_signatures: vec![],
        index_signatures: vec![],
    }
}

fn weak_map_interface() -> InterfaceDeclaration {
    InterfaceDeclaration {
        name: "WeakMap",
        type_parameters: vec![
            TypeParameter::with_constraint("K", Type::reference("WeakKey")),
            TypeParameter::new("V"),
        ],
        extends: vec![],
        properties: vec![],
        methods: vec![
            MethodSignature::new(
                "delete",
                vec![ParameterDeclaration::new("key", Type::type_param("K"))],
                Type::Boolean,
            ),
            MethodSignature::new(
                "get",
                vec![ParameterDeclaration::new("key", Type::type_param("K"))],
                Type::optional(Type::type_param("V")),
            ),
            MethodSignature::new(
                "has",
                vec![ParameterDeclaration::new("key", Type::type_param("K"))],
                Type::Boolean,
            ),
            MethodSignature::new(
                "set",
                vec![
                    ParameterDeclaration::new("key", Type::type_param("K")),
                    ParameterDeclaration::new("value", Type::type_param("V")),
                ],
                Type::reference("this"),
            ),
        ],
        call_signatures: vec![],
        construct_signatures: vec![],
        index_signatures: vec![],
    }
}

fn weak_map_constructor_interface() -> InterfaceDeclaration {
    InterfaceDeclaration {
        name: "WeakMapConstructor",
        type_parameters: vec![],
        extends: vec![],
        properties: vec![
            PropertySignature::readonly("prototype", Type::reference2("WeakMap", Type::reference("WeakKey"), Type::Any)),
        ],
        methods: vec![],
        call_signatures: vec![],
        construct_signatures: vec![
            ConstructSignature {
                type_parameters: vec![
                    TypeParameter::with_constraint("K", Type::reference("WeakKey")),
                    TypeParameter::new("V"),
                ],
                parameters: vec![],
                return_type: Type::reference2("WeakMap", Type::type_param("K"), Type::type_param("V")),
            },
            ConstructSignature {
                type_parameters: vec![
                    TypeParameter::with_constraint("K", Type::reference("WeakKey")),
                    TypeParameter::new("V"),
                ],
                parameters: vec![ParameterDeclaration::optional(
                    "entries",
                    Type::nullable(Type::reference1("ReadonlyArray", Type::Tuple(vec![Type::type_param("K"), Type::type_param("V")]))),
                )],
                return_type: Type::reference2("WeakMap", Type::type_param("K"), Type::type_param("V")),
            },
        ],
        index_signatures: vec![],
    }
}

fn weak_set_interface() -> InterfaceDeclaration {
    InterfaceDeclaration {
        name: "WeakSet",
        type_parameters: vec![TypeParameter::with_constraint("T", Type::reference("WeakKey"))],
        extends: vec![],
        properties: vec![],
        methods: vec![
            MethodSignature::new(
                "add",
                vec![ParameterDeclaration::new("value", Type::type_param("T"))],
                Type::reference("this"),
            ),
            MethodSignature::new(
                "delete",
                vec![ParameterDeclaration::new("value", Type::type_param("T"))],
                Type::Boolean,
            ),
            MethodSignature::new(
                "has",
                vec![ParameterDeclaration::new("value", Type::type_param("T"))],
                Type::Boolean,
            ),
        ],
        call_signatures: vec![],
        construct_signatures: vec![],
        index_signatures: vec![],
    }
}

fn weak_set_constructor_interface() -> InterfaceDeclaration {
    InterfaceDeclaration {
        name: "WeakSetConstructor",
        type_parameters: vec![],
        extends: vec![],
        properties: vec![
            PropertySignature::readonly("prototype", Type::reference1("WeakSet", Type::reference("WeakKey"))),
        ],
        methods: vec![],
        call_signatures: vec![],
        construct_signatures: vec![
            ConstructSignature {
                type_parameters: vec![TypeParameter::with_constraint("T", Type::reference("WeakKey"))],
                parameters: vec![],
                return_type: Type::reference1("WeakSet", Type::type_param("T")),
            },
            ConstructSignature {
                type_parameters: vec![TypeParameter::with_constraint("T", Type::reference("WeakKey"))],
                parameters: vec![ParameterDeclaration::optional(
                    "values",
                    Type::nullable(Type::reference1("ReadonlyArray", Type::type_param("T"))),
                )],
                return_type: Type::reference1("WeakSet", Type::type_param("T")),
            },
        ],
        index_signatures: vec![],
    }
}

fn generator_interface() -> InterfaceDeclaration {
    InterfaceDeclaration {
        name: "Generator",
        type_parameters: vec![
            TypeParameter::with_default("T", Type::Unknown),
            TypeParameter::with_default("TReturn", Type::Any),
            TypeParameter::with_default("TNext", Type::Unknown),
        ],
        extends: vec![Type::reference1("Iterator", Type::type_param("T"))],
        properties: vec![],
        methods: vec![
            MethodSignature::new(
                "next",
                vec![ParameterDeclaration::rest("args", Type::union(vec![
                    Type::Tuple(vec![]),
                    Type::Tuple(vec![Type::type_param("TNext")]),
                ]))],
                Type::reference2("IteratorResult", Type::type_param("T"), Type::type_param("TReturn")),
            ),
            MethodSignature::new(
                "return",
                vec![ParameterDeclaration::new("value", Type::type_param("TReturn"))],
                Type::reference2("IteratorResult", Type::type_param("T"), Type::type_param("TReturn")),
            ),
            MethodSignature::new(
                "throw",
                vec![ParameterDeclaration::new("e", Type::Any)],
                Type::reference2("IteratorResult", Type::type_param("T"), Type::type_param("TReturn")),
            ),
        ],
        call_signatures: vec![],
        construct_signatures: vec![],
        index_signatures: vec![],
    }
}

fn generator_function_interface() -> InterfaceDeclaration {
    InterfaceDeclaration {
        name: "GeneratorFunction",
        type_parameters: vec![],
        extends: vec![],
        properties: vec![
            PropertySignature::readonly("length", Type::Number),
            PropertySignature::readonly("name", Type::String),
            PropertySignature::readonly("prototype", Type::reference1("Generator", Type::Unknown)),
        ],
        methods: vec![],
        call_signatures: vec![CallSignature {
            type_parameters: vec![],
            parameters: vec![ParameterDeclaration::rest("args", Type::array(Type::Any))],
            return_type: Type::reference1("Generator", Type::Unknown),
        }],
        construct_signatures: vec![],
        index_signatures: vec![],
    }
}

fn generator_function_constructor_interface() -> InterfaceDeclaration {
    InterfaceDeclaration {
        name: "GeneratorFunctionConstructor",
        type_parameters: vec![],
        extends: vec![],
        properties: vec![
            PropertySignature::readonly("prototype", Type::reference("GeneratorFunction")),
        ],
        methods: vec![],
        call_signatures: vec![CallSignature {
            type_parameters: vec![],
            parameters: vec![ParameterDeclaration::rest("args", Type::array(Type::String))],
            return_type: Type::reference("GeneratorFunction"),
        }],
        construct_signatures: vec![ConstructSignature {
            type_parameters: vec![],
            parameters: vec![ParameterDeclaration::rest("args", Type::array(Type::String))],
            return_type: Type::reference("GeneratorFunction"),
        }],
        index_signatures: vec![],
    }
}

fn iterable_iterator_interface() -> InterfaceDeclaration {
    InterfaceDeclaration {
        name: "IterableIterator",
        type_parameters: vec![TypeParameter::new("T")],
        extends: vec![
            Type::reference1("Iterator", Type::type_param("T")),
            Type::reference1("Iterable", Type::type_param("T")),
        ],
        properties: vec![],
        methods: vec![],
        call_signatures: vec![],
        construct_signatures: vec![],
        index_signatures: vec![],
    }
}

fn proxy_handler_interface() -> InterfaceDeclaration {
    InterfaceDeclaration {
        name: "ProxyHandler",
        type_parameters: vec![TypeParameter::new("T")],
        extends: vec![],
        properties: vec![],
        methods: vec![
            MethodSignature {
                name: "apply",
                type_parameters: vec![],
                parameters: vec![
                    ParameterDeclaration::new("target", Type::type_param("T")),
                    ParameterDeclaration::new("thisArg", Type::Any),
                    ParameterDeclaration::new("argArray", Type::array(Type::Any)),
                ],
                return_type: Type::Any,
                optional: true,
            },
            MethodSignature {
                name: "construct",
                type_parameters: vec![],
                parameters: vec![
                    ParameterDeclaration::new("target", Type::type_param("T")),
                    ParameterDeclaration::new("argArray", Type::array(Type::Any)),
                    ParameterDeclaration::new("newTarget", Type::reference("Function")),
                ],
                return_type: Type::Object,
                optional: true,
            },
            MethodSignature {
                name: "defineProperty",
                type_parameters: vec![],
                parameters: vec![
                    ParameterDeclaration::new("target", Type::type_param("T")),
                    ParameterDeclaration::new("property", Type::union(vec![Type::String, Type::Symbol])),
                    ParameterDeclaration::new("attributes", Type::reference("PropertyDescriptor")),
                ],
                return_type: Type::Boolean,
                optional: true,
            },
            MethodSignature {
                name: "deleteProperty",
                type_parameters: vec![],
                parameters: vec![
                    ParameterDeclaration::new("target", Type::type_param("T")),
                    ParameterDeclaration::new("p", Type::union(vec![Type::String, Type::Symbol])),
                ],
                return_type: Type::Boolean,
                optional: true,
            },
            MethodSignature {
                name: "get",
                type_parameters: vec![],
                parameters: vec![
                    ParameterDeclaration::new("target", Type::type_param("T")),
                    ParameterDeclaration::new("p", Type::union(vec![Type::String, Type::Symbol])),
                    ParameterDeclaration::new("receiver", Type::Any),
                ],
                return_type: Type::Any,
                optional: true,
            },
            MethodSignature {
                name: "getOwnPropertyDescriptor",
                type_parameters: vec![],
                parameters: vec![
                    ParameterDeclaration::new("target", Type::type_param("T")),
                    ParameterDeclaration::new("p", Type::union(vec![Type::String, Type::Symbol])),
                ],
                return_type: Type::optional(Type::reference("PropertyDescriptor")),
                optional: true,
            },
            MethodSignature {
                name: "getPrototypeOf",
                type_parameters: vec![],
                parameters: vec![
                    ParameterDeclaration::new("target", Type::type_param("T")),
                ],
                return_type: Type::nullable(Type::Object),
                optional: true,
            },
            MethodSignature {
                name: "has",
                type_parameters: vec![],
                parameters: vec![
                    ParameterDeclaration::new("target", Type::type_param("T")),
                    ParameterDeclaration::new("p", Type::union(vec![Type::String, Type::Symbol])),
                ],
                return_type: Type::Boolean,
                optional: true,
            },
            MethodSignature {
                name: "isExtensible",
                type_parameters: vec![],
                parameters: vec![
                    ParameterDeclaration::new("target", Type::type_param("T")),
                ],
                return_type: Type::Boolean,
                optional: true,
            },
            MethodSignature {
                name: "ownKeys",
                type_parameters: vec![],
                parameters: vec![
                    ParameterDeclaration::new("target", Type::type_param("T")),
                ],
                return_type: Type::reference1("ArrayLike", Type::union(vec![Type::String, Type::Symbol])),
                optional: true,
            },
            MethodSignature {
                name: "preventExtensions",
                type_parameters: vec![],
                parameters: vec![
                    ParameterDeclaration::new("target", Type::type_param("T")),
                ],
                return_type: Type::Boolean,
                optional: true,
            },
            MethodSignature {
                name: "set",
                type_parameters: vec![],
                parameters: vec![
                    ParameterDeclaration::new("target", Type::type_param("T")),
                    ParameterDeclaration::new("p", Type::union(vec![Type::String, Type::Symbol])),
                    ParameterDeclaration::new("newValue", Type::Any),
                    ParameterDeclaration::new("receiver", Type::Any),
                ],
                return_type: Type::Boolean,
                optional: true,
            },
            MethodSignature {
                name: "setPrototypeOf",
                type_parameters: vec![],
                parameters: vec![
                    ParameterDeclaration::new("target", Type::type_param("T")),
                    ParameterDeclaration::new("v", Type::nullable(Type::Object)),
                ],
                return_type: Type::Boolean,
                optional: true,
            },
        ],
        call_signatures: vec![],
        construct_signatures: vec![],
        index_signatures: vec![],
    }
}

fn proxy_constructor_interface() -> InterfaceDeclaration {
    InterfaceDeclaration {
        name: "ProxyConstructor",
        type_parameters: vec![],
        extends: vec![],
        properties: vec![],
        methods: vec![
            MethodSignature::generic(
                "revocable",
                vec![TypeParameter::new("T")],
                vec![
                    ParameterDeclaration::new("target", Type::type_param("T")),
                    ParameterDeclaration::new("handler", Type::reference1("ProxyHandler", Type::type_param("T"))),
                ],
                Type::ObjectLiteral(vec![
                    PropertySignature::new("proxy", Type::type_param("T")),
                    PropertySignature::new("revoke", Type::func(vec![], Type::Void)),
                ]),
            ),
        ],
        call_signatures: vec![],
        construct_signatures: vec![ConstructSignature {
            type_parameters: vec![TypeParameter::new("T")],
            parameters: vec![
                ParameterDeclaration::new("target", Type::type_param("T")),
                ParameterDeclaration::new("handler", Type::reference1("ProxyHandler", Type::type_param("T"))),
            ],
            return_type: Type::type_param("T"),
        }],
        index_signatures: vec![],
    }
}

fn reflect_namespace() -> InterfaceDeclaration {
    InterfaceDeclaration {
        name: "Reflect",
        type_parameters: vec![],
        extends: vec![],
        properties: vec![],
        methods: vec![
            MethodSignature::new(
                "apply",
                vec![
                    ParameterDeclaration::new("target", Type::reference("Function")),
                    ParameterDeclaration::new("thisArgument", Type::Any),
                    ParameterDeclaration::new("argumentsList", Type::reference1("ArrayLike", Type::Any)),
                ],
                Type::Any,
            ),
            MethodSignature::new(
                "construct",
                vec![
                    ParameterDeclaration::new("target", Type::reference("Function")),
                    ParameterDeclaration::new("argumentsList", Type::reference1("ArrayLike", Type::Any)),
                    ParameterDeclaration::optional("newTarget", Type::reference("Function")),
                ],
                Type::Any,
            ),
            MethodSignature::new(
                "defineProperty",
                vec![
                    ParameterDeclaration::new("target", Type::Object),
                    ParameterDeclaration::new("propertyKey", Type::union(vec![Type::String, Type::Symbol])),
                    ParameterDeclaration::new("attributes", Type::reference("PropertyDescriptor")),
                ],
                Type::Boolean,
            ),
            MethodSignature::new(
                "deleteProperty",
                vec![
                    ParameterDeclaration::new("target", Type::Object),
                    ParameterDeclaration::new("propertyKey", Type::union(vec![Type::String, Type::Symbol])),
                ],
                Type::Boolean,
            ),
            MethodSignature::new(
                "get",
                vec![
                    ParameterDeclaration::new("target", Type::Object),
                    ParameterDeclaration::new("propertyKey", Type::union(vec![Type::String, Type::Symbol])),
                    ParameterDeclaration::optional("receiver", Type::Any),
                ],
                Type::Any,
            ),
            MethodSignature::new(
                "getOwnPropertyDescriptor",
                vec![
                    ParameterDeclaration::new("target", Type::Object),
                    ParameterDeclaration::new("propertyKey", Type::union(vec![Type::String, Type::Symbol])),
                ],
                Type::optional(Type::reference("PropertyDescriptor")),
            ),
            MethodSignature::new(
                "getPrototypeOf",
                vec![
                    ParameterDeclaration::new("target", Type::Object),
                ],
                Type::nullable(Type::Object),
            ),
            MethodSignature::new(
                "has",
                vec![
                    ParameterDeclaration::new("target", Type::Object),
                    ParameterDeclaration::new("propertyKey", Type::union(vec![Type::String, Type::Symbol])),
                ],
                Type::Boolean,
            ),
            MethodSignature::new(
                "isExtensible",
                vec![
                    ParameterDeclaration::new("target", Type::Object),
                ],
                Type::Boolean,
            ),
            MethodSignature::new(
                "ownKeys",
                vec![
                    ParameterDeclaration::new("target", Type::Object),
                ],
                Type::array(Type::union(vec![Type::String, Type::Symbol])),
            ),
            MethodSignature::new(
                "preventExtensions",
                vec![
                    ParameterDeclaration::new("target", Type::Object),
                ],
                Type::Boolean,
            ),
            MethodSignature::new(
                "set",
                vec![
                    ParameterDeclaration::new("target", Type::Object),
                    ParameterDeclaration::new("propertyKey", Type::union(vec![Type::String, Type::Symbol])),
                    ParameterDeclaration::new("value", Type::Any),
                    ParameterDeclaration::optional("receiver", Type::Any),
                ],
                Type::Boolean,
            ),
            MethodSignature::new(
                "setPrototypeOf",
                vec![
                    ParameterDeclaration::new("target", Type::Object),
                    ParameterDeclaration::new("proto", Type::nullable(Type::Object)),
                ],
                Type::Boolean,
            ),
        ],
        call_signatures: vec![],
        construct_signatures: vec![],
        index_signatures: vec![],
    }
}

fn array_es2015_interface() -> InterfaceDeclaration {
    InterfaceDeclaration {
        name: "Array",
        type_parameters: vec![TypeParameter::new("T")],
        extends: vec![],
        properties: vec![],
        methods: vec![
            MethodSignature::new(
                "find",
                vec![
                    ParameterDeclaration::new("predicate", Type::func(
                        vec![
                            ParameterDeclaration::new("value", Type::type_param("T")),
                            ParameterDeclaration::new("index", Type::Number),
                            ParameterDeclaration::new("obj", Type::array(Type::type_param("T"))),
                        ],
                        Type::Unknown,
                    )),
                    ParameterDeclaration::optional("thisArg", Type::Any),
                ],
                Type::optional(Type::type_param("T")),
            ),
            MethodSignature::new(
                "findIndex",
                vec![
                    ParameterDeclaration::new("predicate", Type::func(
                        vec![
                            ParameterDeclaration::new("value", Type::type_param("T")),
                            ParameterDeclaration::new("index", Type::Number),
                            ParameterDeclaration::new("obj", Type::array(Type::type_param("T"))),
                        ],
                        Type::Unknown,
                    )),
                    ParameterDeclaration::optional("thisArg", Type::Any),
                ],
                Type::Number,
            ),
            MethodSignature::new(
                "fill",
                vec![
                    ParameterDeclaration::new("value", Type::type_param("T")),
                    ParameterDeclaration::optional("start", Type::Number),
                    ParameterDeclaration::optional("end", Type::Number),
                ],
                Type::reference("this"),
            ),
            MethodSignature::new(
                "copyWithin",
                vec![
                    ParameterDeclaration::new("target", Type::Number),
                    ParameterDeclaration::new("start", Type::Number),
                    ParameterDeclaration::optional("end", Type::Number),
                ],
                Type::reference("this"),
            ),
            MethodSignature::new(
                "entries",
                vec![],
                Type::reference1("IterableIterator", Type::Tuple(vec![Type::Number, Type::type_param("T")])),
            ),
            MethodSignature::new(
                "keys",
                vec![],
                Type::reference1("IterableIterator", Type::Number),
            ),
            MethodSignature::new(
                "values",
                vec![],
                Type::reference1("IterableIterator", Type::type_param("T")),
            ),
            MethodSignature::new(
                "includes",
                vec![
                    ParameterDeclaration::new("searchElement", Type::type_param("T")),
                    ParameterDeclaration::optional("fromIndex", Type::Number),
                ],
                Type::Boolean,
            ),
            MethodSignature::generic(
                "flatMap",
                vec![TypeParameter::new("U")],
                vec![
                    ParameterDeclaration::new("callback", Type::func(
                        vec![
                            ParameterDeclaration::new("value", Type::type_param("T")),
                            ParameterDeclaration::new("index", Type::Number),
                            ParameterDeclaration::new("array", Type::array(Type::type_param("T"))),
                        ],
                        Type::union(vec![Type::type_param("U"), Type::reference1("ReadonlyArray", Type::type_param("U"))]),
                    )),
                    ParameterDeclaration::optional("thisArg", Type::Any),
                ],
                Type::array(Type::type_param("U")),
            ),
            MethodSignature::new(
                "flat",
                vec![ParameterDeclaration::optional("depth", Type::Number)],
                Type::array(Type::Any),
            ),
            MethodSignature::new(
                "at",
                vec![ParameterDeclaration::new("index", Type::Number)],
                Type::optional(Type::type_param("T")),
            ),
        ],
        call_signatures: vec![],
        construct_signatures: vec![],
        index_signatures: vec![],
    }
}

fn object_es2015_constructor() -> InterfaceDeclaration {
    InterfaceDeclaration {
        name: "ObjectConstructor",
        type_parameters: vec![],
        extends: vec![],
        properties: vec![],
        methods: vec![
            MethodSignature::generic(
                "assign",
                vec![TypeParameter::new("T"), TypeParameter::new("U")],
                vec![
                    ParameterDeclaration::new("target", Type::type_param("T")),
                    ParameterDeclaration::new("source", Type::type_param("U")),
                ],
                Type::intersection(vec![Type::type_param("T"), Type::type_param("U")]),
            ),
            MethodSignature::new(
                "getOwnPropertySymbols",
                vec![ParameterDeclaration::new("o", Type::Any)],
                Type::array(Type::Symbol),
            ),
            MethodSignature::new(
                "is",
                vec![
                    ParameterDeclaration::new("value1", Type::Any),
                    ParameterDeclaration::new("value2", Type::Any),
                ],
                Type::Boolean,
            ),
            MethodSignature::new(
                "setPrototypeOf",
                vec![
                    ParameterDeclaration::new("o", Type::Any),
                    ParameterDeclaration::new("proto", Type::nullable(Type::Object)),
                ],
                Type::Any,
            ),
            MethodSignature::generic(
                "values",
                vec![TypeParameter::new("T")],
                vec![ParameterDeclaration::new("o", Type::union(vec![
                    Type::ObjectLiteral(vec![IndexSignature {
                        key_name: "s",
                        key_type: Type::String,
                        value_type: Type::type_param("T"),
                        readonly: false,
                    }.into()]),
                    Type::reference1("ArrayLike", Type::type_param("T")),
                ]))],
                Type::array(Type::type_param("T")),
            ),
            MethodSignature::generic(
                "entries",
                vec![TypeParameter::new("T")],
                vec![ParameterDeclaration::new("o", Type::union(vec![
                    Type::ObjectLiteral(vec![IndexSignature {
                        key_name: "s",
                        key_type: Type::String,
                        value_type: Type::type_param("T"),
                        readonly: false,
                    }.into()]),
                    Type::reference1("ArrayLike", Type::type_param("T")),
                ]))],
                Type::array(Type::Tuple(vec![Type::String, Type::type_param("T")])),
            ),
            MethodSignature::generic(
                "fromEntries",
                vec![TypeParameter::new("T")],
                vec![ParameterDeclaration::new(
                    "entries",
                    Type::reference1("Iterable", Type::Tuple(vec![
                        Type::union(vec![Type::String, Type::Symbol]),
                        Type::type_param("T"),
                    ])),
                )],
                Type::ObjectLiteral(vec![IndexSignature {
                    key_name: "k",
                    key_type: Type::String,
                    value_type: Type::type_param("T"),
                    readonly: false,
                }.into()]),
            ),
        ],
        call_signatures: vec![],
        construct_signatures: vec![],
        index_signatures: vec![],
    }
}

fn string_es2015_interface() -> InterfaceDeclaration {
    InterfaceDeclaration {
        name: "String",
        type_parameters: vec![],
        extends: vec![],
        properties: vec![],
        methods: vec![
            MethodSignature::new(
                "codePointAt",
                vec![ParameterDeclaration::new("pos", Type::Number)],
                Type::optional(Type::Number),
            ),
            MethodSignature::new(
                "includes",
                vec![
                    ParameterDeclaration::new("searchString", Type::String),
                    ParameterDeclaration::optional("position", Type::Number),
                ],
                Type::Boolean,
            ),
            MethodSignature::new(
                "endsWith",
                vec![
                    ParameterDeclaration::new("searchString", Type::String),
                    ParameterDeclaration::optional("endPosition", Type::Number),
                ],
                Type::Boolean,
            ),
            MethodSignature::new(
                "normalize",
                vec![ParameterDeclaration::optional("form", Type::union(vec![
                    Type::StringLiteral("NFC"),
                    Type::StringLiteral("NFD"),
                    Type::StringLiteral("NFKC"),
                    Type::StringLiteral("NFKD"),
                ]))],
                Type::String,
            ),
            MethodSignature::new(
                "repeat",
                vec![ParameterDeclaration::new("count", Type::Number)],
                Type::String,
            ),
            MethodSignature::new(
                "startsWith",
                vec![
                    ParameterDeclaration::new("searchString", Type::String),
                    ParameterDeclaration::optional("position", Type::Number),
                ],
                Type::Boolean,
            ),
            MethodSignature::new(
                "anchor",
                vec![ParameterDeclaration::new("name", Type::String)],
                Type::String,
            ),
            MethodSignature::new("big", vec![], Type::String),
            MethodSignature::new("blink", vec![], Type::String),
            MethodSignature::new("bold", vec![], Type::String),
            MethodSignature::new("fixed", vec![], Type::String),
            MethodSignature::new(
                "fontcolor",
                vec![ParameterDeclaration::new("color", Type::String)],
                Type::String,
            ),
            MethodSignature::new(
                "fontsize",
                vec![ParameterDeclaration::new("size", Type::union(vec![Type::Number, Type::String]))],
                Type::String,
            ),
            MethodSignature::new("italics", vec![], Type::String),
            MethodSignature::new(
                "link",
                vec![ParameterDeclaration::new("url", Type::String)],
                Type::String,
            ),
            MethodSignature::new("small", vec![], Type::String),
            MethodSignature::new("strike", vec![], Type::String),
            MethodSignature::new("sub", vec![], Type::String),
            MethodSignature::new("sup", vec![], Type::String),
            MethodSignature::new("padStart", vec![
                ParameterDeclaration::new("maxLength", Type::Number),
                ParameterDeclaration::optional("fillString", Type::String),
            ], Type::String),
            MethodSignature::new("padEnd", vec![
                ParameterDeclaration::new("maxLength", Type::Number),
                ParameterDeclaration::optional("fillString", Type::String),
            ], Type::String),
            MethodSignature::new("trimStart", vec![], Type::String),
            MethodSignature::new("trimEnd", vec![], Type::String),
            MethodSignature::new("trimLeft", vec![], Type::String),
            MethodSignature::new("trimRight", vec![], Type::String),
            MethodSignature::new(
                "at",
                vec![ParameterDeclaration::new("index", Type::Number)],
                Type::optional(Type::String),
            ),
        ],
        call_signatures: vec![],
        construct_signatures: vec![],
        index_signatures: vec![],
    }
}

fn string_es2015_constructor() -> InterfaceDeclaration {
    InterfaceDeclaration {
        name: "StringConstructor",
        type_parameters: vec![],
        extends: vec![],
        properties: vec![],
        methods: vec![
            MethodSignature::new(
                "fromCodePoint",
                vec![ParameterDeclaration::rest("codePoints", Type::array(Type::Number))],
                Type::String,
            ),
            MethodSignature::new(
                "raw",
                vec![
                    ParameterDeclaration::new("template", Type::ObjectLiteral(vec![
                        PropertySignature::readonly("raw", Type::union(vec![
                            Type::reference1("ReadonlyArray", Type::String),
                            Type::reference1("ArrayLike", Type::String),
                        ])),
                    ])),
                    ParameterDeclaration::rest("substitutions", Type::array(Type::Any)),
                ],
                Type::String,
            ),
        ],
        call_signatures: vec![],
        construct_signatures: vec![],
        index_signatures: vec![],
    }
}

fn number_es2015_constructor() -> InterfaceDeclaration {
    InterfaceDeclaration {
        name: "NumberConstructor",
        type_parameters: vec![],
        extends: vec![],
        properties: vec![
            PropertySignature::readonly("EPSILON", Type::Number),
            PropertySignature::readonly("MAX_SAFE_INTEGER", Type::Number),
            PropertySignature::readonly("MIN_SAFE_INTEGER", Type::Number),
        ],
        methods: vec![
            MethodSignature::new(
                "isFinite",
                vec![ParameterDeclaration::new("number", Type::Unknown)],
                Type::Boolean,
            ),
            MethodSignature::new(
                "isInteger",
                vec![ParameterDeclaration::new("number", Type::Unknown)],
                Type::Boolean,
            ),
            MethodSignature::new(
                "isNaN",
                vec![ParameterDeclaration::new("number", Type::Unknown)],
                Type::Boolean,
            ),
            MethodSignature::new(
                "isSafeInteger",
                vec![ParameterDeclaration::new("number", Type::Unknown)],
                Type::Boolean,
            ),
            MethodSignature::new(
                "parseFloat",
                vec![ParameterDeclaration::new("string", Type::String)],
                Type::Number,
            ),
            MethodSignature::new(
                "parseInt",
                vec![
                    ParameterDeclaration::new("string", Type::String),
                    ParameterDeclaration::optional("radix", Type::Number),
                ],
                Type::Number,
            ),
        ],
        call_signatures: vec![],
        construct_signatures: vec![],
        index_signatures: vec![],
    }
}

fn math_es2015_interface() -> InterfaceDeclaration {
    InterfaceDeclaration {
        name: "Math",
        type_parameters: vec![],
        extends: vec![],
        properties: vec![],
        methods: vec![
            MethodSignature::new("clz32", vec![ParameterDeclaration::new("x", Type::Number)], Type::Number),
            MethodSignature::new("imul", vec![ParameterDeclaration::new("x", Type::Number), ParameterDeclaration::new("y", Type::Number)], Type::Number),
            MethodSignature::new("sign", vec![ParameterDeclaration::new("x", Type::Number)], Type::Number),
            MethodSignature::new("log10", vec![ParameterDeclaration::new("x", Type::Number)], Type::Number),
            MethodSignature::new("log2", vec![ParameterDeclaration::new("x", Type::Number)], Type::Number),
            MethodSignature::new("log1p", vec![ParameterDeclaration::new("x", Type::Number)], Type::Number),
            MethodSignature::new("expm1", vec![ParameterDeclaration::new("x", Type::Number)], Type::Number),
            MethodSignature::new("cosh", vec![ParameterDeclaration::new("x", Type::Number)], Type::Number),
            MethodSignature::new("sinh", vec![ParameterDeclaration::new("x", Type::Number)], Type::Number),
            MethodSignature::new("tanh", vec![ParameterDeclaration::new("x", Type::Number)], Type::Number),
            MethodSignature::new("acosh", vec![ParameterDeclaration::new("x", Type::Number)], Type::Number),
            MethodSignature::new("asinh", vec![ParameterDeclaration::new("x", Type::Number)], Type::Number),
            MethodSignature::new("atanh", vec![ParameterDeclaration::new("x", Type::Number)], Type::Number),
            MethodSignature::new("hypot", vec![ParameterDeclaration::rest("values", Type::array(Type::Number))], Type::Number),
            MethodSignature::new("trunc", vec![ParameterDeclaration::new("x", Type::Number)], Type::Number),
            MethodSignature::new("fround", vec![ParameterDeclaration::new("x", Type::Number)], Type::Number),
            MethodSignature::new("cbrt", vec![ParameterDeclaration::new("x", Type::Number)], Type::Number),
        ],
        call_signatures: vec![],
        construct_signatures: vec![],
        index_signatures: vec![],
    }
}

// Helper to convert IndexSignature to PropertySignature for ObjectLiteral
impl From<IndexSignature> for PropertySignature {
    fn from(idx: IndexSignature) -> Self {
        PropertySignature {
            name: idx.key_name,
            type_: idx.value_type,
            optional: false,
            readonly: idx.readonly,
        }
    }
}
