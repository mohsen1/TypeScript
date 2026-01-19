//! ES5 built-in type declarations
//!
//! This module provides the core ES5 built-in types:
//! - Object, Function, Array, String, Number, Boolean
//! - Math, Date, RegExp, Error, JSON
//! - Global functions and values

use super::types::*;

/// Get all ES5 built-in declarations
pub fn get_es5_declarations() -> LibDeclarations {
    let mut lib = LibDeclarations::new();

    // Core interfaces
    lib.interfaces.push(object_interface());
    lib.interfaces.push(object_constructor_interface());
    lib.interfaces.push(function_interface());
    lib.interfaces.push(function_constructor_interface());
    lib.interfaces.push(array_interface());
    lib.interfaces.push(array_constructor_interface());
    lib.interfaces.push(readonly_array_interface());
    lib.interfaces.push(string_interface());
    lib.interfaces.push(string_constructor_interface());
    lib.interfaces.push(number_interface());
    lib.interfaces.push(number_constructor_interface());
    lib.interfaces.push(boolean_interface());
    lib.interfaces.push(boolean_constructor_interface());

    // Additional ES5 interfaces
    lib.interfaces.push(math_interface());
    lib.interfaces.push(date_interface());
    lib.interfaces.push(date_constructor_interface());
    lib.interfaces.push(regexp_interface());
    lib.interfaces.push(regexp_constructor_interface());
    lib.interfaces.push(error_interface());
    lib.interfaces.push(error_constructor_interface());
    lib.interfaces.push(eval_error_interface());
    lib.interfaces.push(range_error_interface());
    lib.interfaces.push(reference_error_interface());
    lib.interfaces.push(syntax_error_interface());
    lib.interfaces.push(type_error_interface());
    lib.interfaces.push(uri_error_interface());
    lib.interfaces.push(json_interface());

    // Iterable protocol interfaces
    lib.interfaces.push(iterable_interface());
    lib.interfaces.push(iterator_interface());
    lib.interfaces.push(iterator_result_interface());

    // Array-like interfaces
    lib.interfaces.push(array_like_interface());
    lib.interfaces.push(concat_array_interface());

    // Typed array interfaces
    lib.interfaces.push(array_buffer_interface());
    lib.interfaces.push(array_buffer_constructor_interface());
    lib.interfaces.push(data_view_interface());

    // Global variables
    lib.variables.push(VariableDeclaration {
        name: "NaN",
        type_: Type::Number,
        readonly: true,
    });
    lib.variables.push(VariableDeclaration {
        name: "Infinity",
        type_: Type::Number,
        readonly: true,
    });
    lib.variables.push(VariableDeclaration {
        name: "undefined",
        type_: Type::Undefined,
        readonly: true,
    });
    lib.variables.push(VariableDeclaration {
        name: "Object",
        type_: Type::reference("ObjectConstructor"),
        readonly: true,
    });
    lib.variables.push(VariableDeclaration {
        name: "Function",
        type_: Type::reference("FunctionConstructor"),
        readonly: true,
    });
    lib.variables.push(VariableDeclaration {
        name: "Array",
        type_: Type::reference("ArrayConstructor"),
        readonly: true,
    });
    lib.variables.push(VariableDeclaration {
        name: "String",
        type_: Type::reference("StringConstructor"),
        readonly: true,
    });
    lib.variables.push(VariableDeclaration {
        name: "Number",
        type_: Type::reference("NumberConstructor"),
        readonly: true,
    });
    lib.variables.push(VariableDeclaration {
        name: "Boolean",
        type_: Type::reference("BooleanConstructor"),
        readonly: true,
    });
    lib.variables.push(VariableDeclaration {
        name: "Math",
        type_: Type::reference("Math"),
        readonly: true,
    });
    lib.variables.push(VariableDeclaration {
        name: "Date",
        type_: Type::reference("DateConstructor"),
        readonly: true,
    });
    lib.variables.push(VariableDeclaration {
        name: "RegExp",
        type_: Type::reference("RegExpConstructor"),
        readonly: true,
    });
    lib.variables.push(VariableDeclaration {
        name: "Error",
        type_: Type::reference("ErrorConstructor"),
        readonly: true,
    });
    lib.variables.push(VariableDeclaration {
        name: "JSON",
        type_: Type::reference("JSON"),
        readonly: true,
    });

    // Global functions
    lib.functions.push(FunctionDeclaration {
        name: "eval",
        type_parameters: vec![],
        parameters: vec![ParameterDeclaration::new("x", Type::String)],
        return_type: Type::Any,
    });
    lib.functions.push(FunctionDeclaration {
        name: "parseInt",
        type_parameters: vec![],
        parameters: vec![
            ParameterDeclaration::new("string", Type::String),
            ParameterDeclaration::optional("radix", Type::Number),
        ],
        return_type: Type::Number,
    });
    lib.functions.push(FunctionDeclaration {
        name: "parseFloat",
        type_parameters: vec![],
        parameters: vec![ParameterDeclaration::new("string", Type::String)],
        return_type: Type::Number,
    });
    lib.functions.push(FunctionDeclaration {
        name: "isNaN",
        type_parameters: vec![],
        parameters: vec![ParameterDeclaration::new("number", Type::Number)],
        return_type: Type::Boolean,
    });
    lib.functions.push(FunctionDeclaration {
        name: "isFinite",
        type_parameters: vec![],
        parameters: vec![ParameterDeclaration::new("number", Type::Number)],
        return_type: Type::Boolean,
    });
    lib.functions.push(FunctionDeclaration {
        name: "decodeURI",
        type_parameters: vec![],
        parameters: vec![ParameterDeclaration::new("encodedURI", Type::String)],
        return_type: Type::String,
    });
    lib.functions.push(FunctionDeclaration {
        name: "decodeURIComponent",
        type_parameters: vec![],
        parameters: vec![ParameterDeclaration::new("encodedURIComponent", Type::String)],
        return_type: Type::String,
    });
    lib.functions.push(FunctionDeclaration {
        name: "encodeURI",
        type_parameters: vec![],
        parameters: vec![ParameterDeclaration::new("uri", Type::String)],
        return_type: Type::String,
    });
    lib.functions.push(FunctionDeclaration {
        name: "encodeURIComponent",
        type_parameters: vec![],
        parameters: vec![ParameterDeclaration::new("uriComponent", Type::union(vec![
            Type::String,
            Type::Number,
            Type::Boolean,
        ]))],
        return_type: Type::String,
    });

    lib
}

fn object_interface() -> InterfaceDeclaration {
    InterfaceDeclaration {
        name: "Object",
        type_parameters: vec![],
        extends: vec![],
        properties: vec![
            PropertySignature::readonly("constructor", Type::reference("Function")),
        ],
        methods: vec![
            MethodSignature::new("toString", vec![], Type::String),
            MethodSignature::new("toLocaleString", vec![], Type::String),
            MethodSignature::new("valueOf", vec![], Type::reference("Object")),
            MethodSignature::new(
                "hasOwnProperty",
                vec![ParameterDeclaration::new("v", Type::union(vec![Type::String, Type::Number, Type::Symbol]))],
                Type::Boolean,
            ),
            MethodSignature::new(
                "isPrototypeOf",
                vec![ParameterDeclaration::new("v", Type::reference("Object"))],
                Type::Boolean,
            ),
            MethodSignature::new(
                "propertyIsEnumerable",
                vec![ParameterDeclaration::new("v", Type::union(vec![Type::String, Type::Number, Type::Symbol]))],
                Type::Boolean,
            ),
        ],
        call_signatures: vec![],
        construct_signatures: vec![],
        index_signatures: vec![],
    }
}

fn object_constructor_interface() -> InterfaceDeclaration {
    InterfaceDeclaration {
        name: "ObjectConstructor",
        type_parameters: vec![],
        extends: vec![],
        properties: vec![
            PropertySignature::readonly("prototype", Type::reference("Object")),
        ],
        methods: vec![
            MethodSignature::new(
                "getPrototypeOf",
                vec![ParameterDeclaration::new("o", Type::Any)],
                Type::Any,
            ),
            MethodSignature::new(
                "getOwnPropertyDescriptor",
                vec![
                    ParameterDeclaration::new("o", Type::Any),
                    ParameterDeclaration::new("p", Type::union(vec![Type::String, Type::Number, Type::Symbol])),
                ],
                Type::optional(Type::reference("PropertyDescriptor")),
            ),
            MethodSignature::new(
                "getOwnPropertyNames",
                vec![ParameterDeclaration::new("o", Type::Any)],
                Type::array(Type::String),
            ),
            MethodSignature::generic(
                "create",
                vec![TypeParameter::new("T")],
                vec![ParameterDeclaration::new("o", Type::nullable(Type::type_param("T")))],
                Type::type_param("T"),
            ),
            MethodSignature::generic(
                "defineProperty",
                vec![TypeParameter::new("T")],
                vec![
                    ParameterDeclaration::new("o", Type::type_param("T")),
                    ParameterDeclaration::new("p", Type::union(vec![Type::String, Type::Number, Type::Symbol])),
                    ParameterDeclaration::new("attributes", Type::reference("PropertyDescriptor")),
                ],
                Type::type_param("T"),
            ),
            MethodSignature::new(
                "keys",
                vec![ParameterDeclaration::new("o", Type::Object)],
                Type::array(Type::String),
            ),
            MethodSignature::generic(
                "freeze",
                vec![TypeParameter::new("T")],
                vec![ParameterDeclaration::new("o", Type::type_param("T"))],
                Type::reference1("Readonly", Type::type_param("T")),
            ),
            MethodSignature::new(
                "isFrozen",
                vec![ParameterDeclaration::new("o", Type::Any)],
                Type::Boolean,
            ),
            MethodSignature::generic(
                "seal",
                vec![TypeParameter::new("T")],
                vec![ParameterDeclaration::new("o", Type::type_param("T"))],
                Type::type_param("T"),
            ),
            MethodSignature::new(
                "isSealed",
                vec![ParameterDeclaration::new("o", Type::Any)],
                Type::Boolean,
            ),
            MethodSignature::new(
                "isExtensible",
                vec![ParameterDeclaration::new("o", Type::Any)],
                Type::Boolean,
            ),
            MethodSignature::generic(
                "preventExtensions",
                vec![TypeParameter::new("T")],
                vec![ParameterDeclaration::new("o", Type::type_param("T"))],
                Type::type_param("T"),
            ),
        ],
        call_signatures: vec![
            CallSignature {
                type_parameters: vec![],
                parameters: vec![],
                return_type: Type::Any,
            },
            CallSignature {
                type_parameters: vec![],
                parameters: vec![ParameterDeclaration::new("value", Type::Any)],
                return_type: Type::Any,
            },
        ],
        construct_signatures: vec![
            ConstructSignature {
                type_parameters: vec![],
                parameters: vec![],
                return_type: Type::reference("Object"),
            },
            ConstructSignature {
                type_parameters: vec![],
                parameters: vec![ParameterDeclaration::new("value", Type::Any)],
                return_type: Type::Any,
            },
        ],
        index_signatures: vec![],
    }
}

fn function_interface() -> InterfaceDeclaration {
    InterfaceDeclaration {
        name: "Function",
        type_parameters: vec![],
        extends: vec![],
        properties: vec![
            PropertySignature::readonly("prototype", Type::Any),
            PropertySignature::readonly("length", Type::Number),
            PropertySignature::new("name", Type::String),
        ],
        methods: vec![
            MethodSignature::new(
                "apply",
                vec![
                    ParameterDeclaration::new("thisArg", Type::Any),
                    ParameterDeclaration::optional("argArray", Type::Any),
                ],
                Type::Any,
            ),
            MethodSignature::new(
                "call",
                vec![
                    ParameterDeclaration::new("thisArg", Type::Any),
                    ParameterDeclaration::rest("argArray", Type::array(Type::Any)),
                ],
                Type::Any,
            ),
            MethodSignature::new(
                "bind",
                vec![
                    ParameterDeclaration::new("thisArg", Type::Any),
                    ParameterDeclaration::rest("argArray", Type::array(Type::Any)),
                ],
                Type::Any,
            ),
            MethodSignature::new("toString", vec![], Type::String),
        ],
        call_signatures: vec![CallSignature {
            type_parameters: vec![],
            parameters: vec![ParameterDeclaration::rest("args", Type::array(Type::Any))],
            return_type: Type::Any,
        }],
        construct_signatures: vec![],
        index_signatures: vec![],
    }
}

fn function_constructor_interface() -> InterfaceDeclaration {
    InterfaceDeclaration {
        name: "FunctionConstructor",
        type_parameters: vec![],
        extends: vec![],
        properties: vec![
            PropertySignature::readonly("prototype", Type::reference("Function")),
        ],
        methods: vec![],
        call_signatures: vec![CallSignature {
            type_parameters: vec![],
            parameters: vec![ParameterDeclaration::rest("args", Type::array(Type::String))],
            return_type: Type::reference("Function"),
        }],
        construct_signatures: vec![ConstructSignature {
            type_parameters: vec![],
            parameters: vec![ParameterDeclaration::rest("args", Type::array(Type::String))],
            return_type: Type::reference("Function"),
        }],
        index_signatures: vec![],
    }
}

fn array_interface() -> InterfaceDeclaration {
    InterfaceDeclaration {
        name: "Array",
        type_parameters: vec![TypeParameter::new("T")],
        extends: vec![],
        properties: vec![
            PropertySignature::new("length", Type::Number),
        ],
        methods: vec![
            MethodSignature::new("toString", vec![], Type::String),
            MethodSignature::new("toLocaleString", vec![], Type::String),
            MethodSignature::new("pop", vec![], Type::optional(Type::type_param("T"))),
            MethodSignature::new(
                "push",
                vec![ParameterDeclaration::rest("items", Type::array(Type::type_param("T")))],
                Type::Number,
            ),
            MethodSignature::new(
                "concat",
                vec![ParameterDeclaration::rest("items", Type::array(
                    Type::union(vec![
                        Type::type_param("T"),
                        Type::reference1("ConcatArray", Type::type_param("T")),
                    ])
                ))],
                Type::array(Type::type_param("T")),
            ),
            MethodSignature::new(
                "join",
                vec![ParameterDeclaration::optional("separator", Type::String)],
                Type::String,
            ),
            MethodSignature::new("reverse", vec![], Type::array(Type::type_param("T"))),
            MethodSignature::new("shift", vec![], Type::optional(Type::type_param("T"))),
            MethodSignature::new(
                "slice",
                vec![
                    ParameterDeclaration::optional("start", Type::Number),
                    ParameterDeclaration::optional("end", Type::Number),
                ],
                Type::array(Type::type_param("T")),
            ),
            MethodSignature::new(
                "sort",
                vec![ParameterDeclaration::optional("compareFn", Type::func(
                    vec![
                        ParameterDeclaration::new("a", Type::type_param("T")),
                        ParameterDeclaration::new("b", Type::type_param("T")),
                    ],
                    Type::Number,
                ))],
                Type::reference("this"),
            ),
            MethodSignature::new(
                "splice",
                vec![
                    ParameterDeclaration::new("start", Type::Number),
                    ParameterDeclaration::optional("deleteCount", Type::Number),
                ],
                Type::array(Type::type_param("T")),
            ),
            MethodSignature::new(
                "unshift",
                vec![ParameterDeclaration::rest("items", Type::array(Type::type_param("T")))],
                Type::Number,
            ),
            MethodSignature::new(
                "indexOf",
                vec![
                    ParameterDeclaration::new("searchElement", Type::type_param("T")),
                    ParameterDeclaration::optional("fromIndex", Type::Number),
                ],
                Type::Number,
            ),
            MethodSignature::new(
                "lastIndexOf",
                vec![
                    ParameterDeclaration::new("searchElement", Type::type_param("T")),
                    ParameterDeclaration::optional("fromIndex", Type::Number),
                ],
                Type::Number,
            ),
            MethodSignature::new(
                "every",
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
                Type::Boolean,
            ),
            MethodSignature::new(
                "some",
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
                Type::Boolean,
            ),
            MethodSignature::new(
                "forEach",
                vec![
                    ParameterDeclaration::new("callbackfn", Type::func(
                        vec![
                            ParameterDeclaration::new("value", Type::type_param("T")),
                            ParameterDeclaration::new("index", Type::Number),
                            ParameterDeclaration::new("array", Type::array(Type::type_param("T"))),
                        ],
                        Type::Void,
                    )),
                    ParameterDeclaration::optional("thisArg", Type::Any),
                ],
                Type::Void,
            ),
            MethodSignature::generic(
                "map",
                vec![TypeParameter::new("U")],
                vec![
                    ParameterDeclaration::new("callbackfn", Type::func(
                        vec![
                            ParameterDeclaration::new("value", Type::type_param("T")),
                            ParameterDeclaration::new("index", Type::Number),
                            ParameterDeclaration::new("array", Type::array(Type::type_param("T"))),
                        ],
                        Type::type_param("U"),
                    )),
                    ParameterDeclaration::optional("thisArg", Type::Any),
                ],
                Type::array(Type::type_param("U")),
            ),
            MethodSignature::new(
                "filter",
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
                Type::array(Type::type_param("T")),
            ),
            MethodSignature::generic(
                "reduce",
                vec![TypeParameter::new("U")],
                vec![
                    ParameterDeclaration::new("callbackfn", Type::func(
                        vec![
                            ParameterDeclaration::new("previousValue", Type::type_param("U")),
                            ParameterDeclaration::new("currentValue", Type::type_param("T")),
                            ParameterDeclaration::new("currentIndex", Type::Number),
                            ParameterDeclaration::new("array", Type::array(Type::type_param("T"))),
                        ],
                        Type::type_param("U"),
                    )),
                    ParameterDeclaration::new("initialValue", Type::type_param("U")),
                ],
                Type::type_param("U"),
            ),
            MethodSignature::generic(
                "reduceRight",
                vec![TypeParameter::new("U")],
                vec![
                    ParameterDeclaration::new("callbackfn", Type::func(
                        vec![
                            ParameterDeclaration::new("previousValue", Type::type_param("U")),
                            ParameterDeclaration::new("currentValue", Type::type_param("T")),
                            ParameterDeclaration::new("currentIndex", Type::Number),
                            ParameterDeclaration::new("array", Type::array(Type::type_param("T"))),
                        ],
                        Type::type_param("U"),
                    )),
                    ParameterDeclaration::new("initialValue", Type::type_param("U")),
                ],
                Type::type_param("U"),
            ),
        ],
        call_signatures: vec![],
        construct_signatures: vec![],
        index_signatures: vec![IndexSignature {
            key_name: "n",
            key_type: Type::Number,
            value_type: Type::type_param("T"),
            readonly: false,
        }],
    }
}

fn array_constructor_interface() -> InterfaceDeclaration {
    InterfaceDeclaration {
        name: "ArrayConstructor",
        type_parameters: vec![],
        extends: vec![],
        properties: vec![
            PropertySignature::readonly("prototype", Type::array(Type::Any)),
        ],
        methods: vec![
            MethodSignature::new(
                "isArray",
                vec![ParameterDeclaration::new("arg", Type::Any)],
                Type::Boolean,
            ),
        ],
        call_signatures: vec![
            CallSignature {
                type_parameters: vec![TypeParameter::new("T")],
                parameters: vec![ParameterDeclaration::rest("items", Type::array(Type::type_param("T")))],
                return_type: Type::array(Type::type_param("T")),
            },
        ],
        construct_signatures: vec![
            ConstructSignature {
                type_parameters: vec![TypeParameter::new("T")],
                parameters: vec![ParameterDeclaration::optional("arrayLength", Type::Number)],
                return_type: Type::array(Type::type_param("T")),
            },
            ConstructSignature {
                type_parameters: vec![TypeParameter::new("T")],
                parameters: vec![ParameterDeclaration::rest("items", Type::array(Type::type_param("T")))],
                return_type: Type::array(Type::type_param("T")),
            },
        ],
        index_signatures: vec![],
    }
}

fn readonly_array_interface() -> InterfaceDeclaration {
    InterfaceDeclaration {
        name: "ReadonlyArray",
        type_parameters: vec![TypeParameter::new("T")],
        extends: vec![],
        properties: vec![
            PropertySignature::readonly("length", Type::Number),
        ],
        methods: vec![
            MethodSignature::new("toString", vec![], Type::String),
            MethodSignature::new("toLocaleString", vec![], Type::String),
            MethodSignature::new(
                "concat",
                vec![ParameterDeclaration::rest("items", Type::array(
                    Type::union(vec![
                        Type::type_param("T"),
                        Type::reference1("ConcatArray", Type::type_param("T")),
                    ])
                ))],
                Type::array(Type::type_param("T")),
            ),
            MethodSignature::new(
                "join",
                vec![ParameterDeclaration::optional("separator", Type::String)],
                Type::String,
            ),
            MethodSignature::new(
                "slice",
                vec![
                    ParameterDeclaration::optional("start", Type::Number),
                    ParameterDeclaration::optional("end", Type::Number),
                ],
                Type::array(Type::type_param("T")),
            ),
            MethodSignature::new(
                "indexOf",
                vec![
                    ParameterDeclaration::new("searchElement", Type::type_param("T")),
                    ParameterDeclaration::optional("fromIndex", Type::Number),
                ],
                Type::Number,
            ),
            MethodSignature::new(
                "lastIndexOf",
                vec![
                    ParameterDeclaration::new("searchElement", Type::type_param("T")),
                    ParameterDeclaration::optional("fromIndex", Type::Number),
                ],
                Type::Number,
            ),
            MethodSignature::new(
                "every",
                vec![
                    ParameterDeclaration::new("predicate", Type::func(
                        vec![
                            ParameterDeclaration::new("value", Type::type_param("T")),
                            ParameterDeclaration::new("index", Type::Number),
                            ParameterDeclaration::new("array", Type::reference1("ReadonlyArray", Type::type_param("T"))),
                        ],
                        Type::Unknown,
                    )),
                    ParameterDeclaration::optional("thisArg", Type::Any),
                ],
                Type::Boolean,
            ),
            MethodSignature::new(
                "some",
                vec![
                    ParameterDeclaration::new("predicate", Type::func(
                        vec![
                            ParameterDeclaration::new("value", Type::type_param("T")),
                            ParameterDeclaration::new("index", Type::Number),
                            ParameterDeclaration::new("array", Type::reference1("ReadonlyArray", Type::type_param("T"))),
                        ],
                        Type::Unknown,
                    )),
                    ParameterDeclaration::optional("thisArg", Type::Any),
                ],
                Type::Boolean,
            ),
            MethodSignature::new(
                "forEach",
                vec![
                    ParameterDeclaration::new("callbackfn", Type::func(
                        vec![
                            ParameterDeclaration::new("value", Type::type_param("T")),
                            ParameterDeclaration::new("index", Type::Number),
                            ParameterDeclaration::new("array", Type::reference1("ReadonlyArray", Type::type_param("T"))),
                        ],
                        Type::Void,
                    )),
                    ParameterDeclaration::optional("thisArg", Type::Any),
                ],
                Type::Void,
            ),
            MethodSignature::generic(
                "map",
                vec![TypeParameter::new("U")],
                vec![
                    ParameterDeclaration::new("callbackfn", Type::func(
                        vec![
                            ParameterDeclaration::new("value", Type::type_param("T")),
                            ParameterDeclaration::new("index", Type::Number),
                            ParameterDeclaration::new("array", Type::reference1("ReadonlyArray", Type::type_param("T"))),
                        ],
                        Type::type_param("U"),
                    )),
                    ParameterDeclaration::optional("thisArg", Type::Any),
                ],
                Type::array(Type::type_param("U")),
            ),
            MethodSignature::new(
                "filter",
                vec![
                    ParameterDeclaration::new("predicate", Type::func(
                        vec![
                            ParameterDeclaration::new("value", Type::type_param("T")),
                            ParameterDeclaration::new("index", Type::Number),
                            ParameterDeclaration::new("array", Type::reference1("ReadonlyArray", Type::type_param("T"))),
                        ],
                        Type::Unknown,
                    )),
                    ParameterDeclaration::optional("thisArg", Type::Any),
                ],
                Type::array(Type::type_param("T")),
            ),
            MethodSignature::generic(
                "reduce",
                vec![TypeParameter::new("U")],
                vec![
                    ParameterDeclaration::new("callbackfn", Type::func(
                        vec![
                            ParameterDeclaration::new("previousValue", Type::type_param("U")),
                            ParameterDeclaration::new("currentValue", Type::type_param("T")),
                            ParameterDeclaration::new("currentIndex", Type::Number),
                            ParameterDeclaration::new("array", Type::reference1("ReadonlyArray", Type::type_param("T"))),
                        ],
                        Type::type_param("U"),
                    )),
                    ParameterDeclaration::new("initialValue", Type::type_param("U")),
                ],
                Type::type_param("U"),
            ),
            MethodSignature::generic(
                "reduceRight",
                vec![TypeParameter::new("U")],
                vec![
                    ParameterDeclaration::new("callbackfn", Type::func(
                        vec![
                            ParameterDeclaration::new("previousValue", Type::type_param("U")),
                            ParameterDeclaration::new("currentValue", Type::type_param("T")),
                            ParameterDeclaration::new("currentIndex", Type::Number),
                            ParameterDeclaration::new("array", Type::reference1("ReadonlyArray", Type::type_param("T"))),
                        ],
                        Type::type_param("U"),
                    )),
                    ParameterDeclaration::new("initialValue", Type::type_param("U")),
                ],
                Type::type_param("U"),
            ),
        ],
        call_signatures: vec![],
        construct_signatures: vec![],
        index_signatures: vec![IndexSignature {
            key_name: "n",
            key_type: Type::Number,
            value_type: Type::type_param("T"),
            readonly: true,
        }],
    }
}

fn string_interface() -> InterfaceDeclaration {
    InterfaceDeclaration {
        name: "String",
        type_parameters: vec![],
        extends: vec![],
        properties: vec![
            PropertySignature::readonly("length", Type::Number),
        ],
        methods: vec![
            MethodSignature::new("toString", vec![], Type::String),
            MethodSignature::new("charAt", vec![ParameterDeclaration::new("pos", Type::Number)], Type::String),
            MethodSignature::new("charCodeAt", vec![ParameterDeclaration::new("index", Type::Number)], Type::Number),
            MethodSignature::new(
                "concat",
                vec![ParameterDeclaration::rest("strings", Type::array(Type::String))],
                Type::String,
            ),
            MethodSignature::new(
                "indexOf",
                vec![
                    ParameterDeclaration::new("searchString", Type::String),
                    ParameterDeclaration::optional("position", Type::Number),
                ],
                Type::Number,
            ),
            MethodSignature::new(
                "lastIndexOf",
                vec![
                    ParameterDeclaration::new("searchString", Type::String),
                    ParameterDeclaration::optional("position", Type::Number),
                ],
                Type::Number,
            ),
            MethodSignature::new(
                "localeCompare",
                vec![ParameterDeclaration::new("that", Type::String)],
                Type::Number,
            ),
            MethodSignature::new(
                "match",
                vec![ParameterDeclaration::new("regexp", Type::union(vec![Type::String, Type::reference("RegExp")]))],
                Type::nullable(Type::reference("RegExpMatchArray")),
            ),
            MethodSignature::new(
                "replace",
                vec![
                    ParameterDeclaration::new("searchValue", Type::union(vec![Type::String, Type::reference("RegExp")])),
                    ParameterDeclaration::new("replaceValue", Type::String),
                ],
                Type::String,
            ),
            MethodSignature::new(
                "search",
                vec![ParameterDeclaration::new("regexp", Type::union(vec![Type::String, Type::reference("RegExp")]))],
                Type::Number,
            ),
            MethodSignature::new(
                "slice",
                vec![
                    ParameterDeclaration::optional("start", Type::Number),
                    ParameterDeclaration::optional("end", Type::Number),
                ],
                Type::String,
            ),
            MethodSignature::new(
                "split",
                vec![
                    ParameterDeclaration::new("separator", Type::union(vec![Type::String, Type::reference("RegExp")])),
                    ParameterDeclaration::optional("limit", Type::Number),
                ],
                Type::array(Type::String),
            ),
            MethodSignature::new(
                "substring",
                vec![
                    ParameterDeclaration::new("start", Type::Number),
                    ParameterDeclaration::optional("end", Type::Number),
                ],
                Type::String,
            ),
            MethodSignature::new("toLowerCase", vec![], Type::String),
            MethodSignature::new("toLocaleLowerCase", vec![ParameterDeclaration::optional("locales", Type::union(vec![Type::String, Type::array(Type::String)]))], Type::String),
            MethodSignature::new("toUpperCase", vec![], Type::String),
            MethodSignature::new("toLocaleUpperCase", vec![ParameterDeclaration::optional("locales", Type::union(vec![Type::String, Type::array(Type::String)]))], Type::String),
            MethodSignature::new("trim", vec![], Type::String),
            MethodSignature::new("valueOf", vec![], Type::String),
        ],
        call_signatures: vec![],
        construct_signatures: vec![],
        index_signatures: vec![IndexSignature {
            key_name: "index",
            key_type: Type::Number,
            value_type: Type::String,
            readonly: true,
        }],
    }
}

fn string_constructor_interface() -> InterfaceDeclaration {
    InterfaceDeclaration {
        name: "StringConstructor",
        type_parameters: vec![],
        extends: vec![],
        properties: vec![
            PropertySignature::readonly("prototype", Type::reference("String")),
        ],
        methods: vec![
            MethodSignature::new(
                "fromCharCode",
                vec![ParameterDeclaration::rest("codes", Type::array(Type::Number))],
                Type::String,
            ),
        ],
        call_signatures: vec![CallSignature {
            type_parameters: vec![],
            parameters: vec![ParameterDeclaration::optional("value", Type::Any)],
            return_type: Type::String,
        }],
        construct_signatures: vec![ConstructSignature {
            type_parameters: vec![],
            parameters: vec![ParameterDeclaration::optional("value", Type::Any)],
            return_type: Type::reference("String"),
        }],
        index_signatures: vec![],
    }
}

fn number_interface() -> InterfaceDeclaration {
    InterfaceDeclaration {
        name: "Number",
        type_parameters: vec![],
        extends: vec![],
        properties: vec![],
        methods: vec![
            MethodSignature::new("toString", vec![ParameterDeclaration::optional("radix", Type::Number)], Type::String),
            MethodSignature::new("toFixed", vec![ParameterDeclaration::optional("fractionDigits", Type::Number)], Type::String),
            MethodSignature::new("toExponential", vec![ParameterDeclaration::optional("fractionDigits", Type::Number)], Type::String),
            MethodSignature::new("toPrecision", vec![ParameterDeclaration::optional("precision", Type::Number)], Type::String),
            MethodSignature::new("valueOf", vec![], Type::Number),
        ],
        call_signatures: vec![],
        construct_signatures: vec![],
        index_signatures: vec![],
    }
}

fn number_constructor_interface() -> InterfaceDeclaration {
    InterfaceDeclaration {
        name: "NumberConstructor",
        type_parameters: vec![],
        extends: vec![],
        properties: vec![
            PropertySignature::readonly("prototype", Type::reference("Number")),
            PropertySignature::readonly("MAX_VALUE", Type::Number),
            PropertySignature::readonly("MIN_VALUE", Type::Number),
            PropertySignature::readonly("NaN", Type::Number),
            PropertySignature::readonly("NEGATIVE_INFINITY", Type::Number),
            PropertySignature::readonly("POSITIVE_INFINITY", Type::Number),
        ],
        methods: vec![],
        call_signatures: vec![CallSignature {
            type_parameters: vec![],
            parameters: vec![ParameterDeclaration::optional("value", Type::Any)],
            return_type: Type::Number,
        }],
        construct_signatures: vec![ConstructSignature {
            type_parameters: vec![],
            parameters: vec![ParameterDeclaration::optional("value", Type::Any)],
            return_type: Type::reference("Number"),
        }],
        index_signatures: vec![],
    }
}

fn boolean_interface() -> InterfaceDeclaration {
    InterfaceDeclaration {
        name: "Boolean",
        type_parameters: vec![],
        extends: vec![],
        properties: vec![],
        methods: vec![
            MethodSignature::new("valueOf", vec![], Type::Boolean),
        ],
        call_signatures: vec![],
        construct_signatures: vec![],
        index_signatures: vec![],
    }
}

fn boolean_constructor_interface() -> InterfaceDeclaration {
    InterfaceDeclaration {
        name: "BooleanConstructor",
        type_parameters: vec![],
        extends: vec![],
        properties: vec![
            PropertySignature::readonly("prototype", Type::reference("Boolean")),
        ],
        methods: vec![],
        call_signatures: vec![CallSignature {
            type_parameters: vec![],
            parameters: vec![ParameterDeclaration::optional("value", Type::Any)],
            return_type: Type::Boolean,
        }],
        construct_signatures: vec![ConstructSignature {
            type_parameters: vec![],
            parameters: vec![ParameterDeclaration::optional("value", Type::Any)],
            return_type: Type::reference("Boolean"),
        }],
        index_signatures: vec![],
    }
}

fn math_interface() -> InterfaceDeclaration {
    InterfaceDeclaration {
        name: "Math",
        type_parameters: vec![],
        extends: vec![],
        properties: vec![
            PropertySignature::readonly("E", Type::Number),
            PropertySignature::readonly("LN10", Type::Number),
            PropertySignature::readonly("LN2", Type::Number),
            PropertySignature::readonly("LOG2E", Type::Number),
            PropertySignature::readonly("LOG10E", Type::Number),
            PropertySignature::readonly("PI", Type::Number),
            PropertySignature::readonly("SQRT1_2", Type::Number),
            PropertySignature::readonly("SQRT2", Type::Number),
        ],
        methods: vec![
            MethodSignature::new("abs", vec![ParameterDeclaration::new("x", Type::Number)], Type::Number),
            MethodSignature::new("acos", vec![ParameterDeclaration::new("x", Type::Number)], Type::Number),
            MethodSignature::new("asin", vec![ParameterDeclaration::new("x", Type::Number)], Type::Number),
            MethodSignature::new("atan", vec![ParameterDeclaration::new("x", Type::Number)], Type::Number),
            MethodSignature::new("atan2", vec![ParameterDeclaration::new("y", Type::Number), ParameterDeclaration::new("x", Type::Number)], Type::Number),
            MethodSignature::new("ceil", vec![ParameterDeclaration::new("x", Type::Number)], Type::Number),
            MethodSignature::new("cos", vec![ParameterDeclaration::new("x", Type::Number)], Type::Number),
            MethodSignature::new("exp", vec![ParameterDeclaration::new("x", Type::Number)], Type::Number),
            MethodSignature::new("floor", vec![ParameterDeclaration::new("x", Type::Number)], Type::Number),
            MethodSignature::new("log", vec![ParameterDeclaration::new("x", Type::Number)], Type::Number),
            MethodSignature::new("max", vec![ParameterDeclaration::rest("values", Type::array(Type::Number))], Type::Number),
            MethodSignature::new("min", vec![ParameterDeclaration::rest("values", Type::array(Type::Number))], Type::Number),
            MethodSignature::new("pow", vec![ParameterDeclaration::new("x", Type::Number), ParameterDeclaration::new("y", Type::Number)], Type::Number),
            MethodSignature::new("random", vec![], Type::Number),
            MethodSignature::new("round", vec![ParameterDeclaration::new("x", Type::Number)], Type::Number),
            MethodSignature::new("sin", vec![ParameterDeclaration::new("x", Type::Number)], Type::Number),
            MethodSignature::new("sqrt", vec![ParameterDeclaration::new("x", Type::Number)], Type::Number),
            MethodSignature::new("tan", vec![ParameterDeclaration::new("x", Type::Number)], Type::Number),
        ],
        call_signatures: vec![],
        construct_signatures: vec![],
        index_signatures: vec![],
    }
}

fn date_interface() -> InterfaceDeclaration {
    InterfaceDeclaration {
        name: "Date",
        type_parameters: vec![],
        extends: vec![],
        properties: vec![],
        methods: vec![
            MethodSignature::new("toString", vec![], Type::String),
            MethodSignature::new("toDateString", vec![], Type::String),
            MethodSignature::new("toTimeString", vec![], Type::String),
            MethodSignature::new("toLocaleString", vec![], Type::String),
            MethodSignature::new("toLocaleDateString", vec![], Type::String),
            MethodSignature::new("toLocaleTimeString", vec![], Type::String),
            MethodSignature::new("valueOf", vec![], Type::Number),
            MethodSignature::new("getTime", vec![], Type::Number),
            MethodSignature::new("getFullYear", vec![], Type::Number),
            MethodSignature::new("getUTCFullYear", vec![], Type::Number),
            MethodSignature::new("getMonth", vec![], Type::Number),
            MethodSignature::new("getUTCMonth", vec![], Type::Number),
            MethodSignature::new("getDate", vec![], Type::Number),
            MethodSignature::new("getUTCDate", vec![], Type::Number),
            MethodSignature::new("getDay", vec![], Type::Number),
            MethodSignature::new("getUTCDay", vec![], Type::Number),
            MethodSignature::new("getHours", vec![], Type::Number),
            MethodSignature::new("getUTCHours", vec![], Type::Number),
            MethodSignature::new("getMinutes", vec![], Type::Number),
            MethodSignature::new("getUTCMinutes", vec![], Type::Number),
            MethodSignature::new("getSeconds", vec![], Type::Number),
            MethodSignature::new("getUTCSeconds", vec![], Type::Number),
            MethodSignature::new("getMilliseconds", vec![], Type::Number),
            MethodSignature::new("getUTCMilliseconds", vec![], Type::Number),
            MethodSignature::new("getTimezoneOffset", vec![], Type::Number),
            MethodSignature::new("setTime", vec![ParameterDeclaration::new("time", Type::Number)], Type::Number),
            MethodSignature::new("setMilliseconds", vec![ParameterDeclaration::new("ms", Type::Number)], Type::Number),
            MethodSignature::new("setUTCMilliseconds", vec![ParameterDeclaration::new("ms", Type::Number)], Type::Number),
            MethodSignature::new("setSeconds", vec![ParameterDeclaration::new("sec", Type::Number), ParameterDeclaration::optional("ms", Type::Number)], Type::Number),
            MethodSignature::new("setUTCSeconds", vec![ParameterDeclaration::new("sec", Type::Number), ParameterDeclaration::optional("ms", Type::Number)], Type::Number),
            MethodSignature::new("setMinutes", vec![ParameterDeclaration::new("min", Type::Number), ParameterDeclaration::optional("sec", Type::Number), ParameterDeclaration::optional("ms", Type::Number)], Type::Number),
            MethodSignature::new("setUTCMinutes", vec![ParameterDeclaration::new("min", Type::Number), ParameterDeclaration::optional("sec", Type::Number), ParameterDeclaration::optional("ms", Type::Number)], Type::Number),
            MethodSignature::new("setHours", vec![ParameterDeclaration::new("hours", Type::Number), ParameterDeclaration::optional("min", Type::Number), ParameterDeclaration::optional("sec", Type::Number), ParameterDeclaration::optional("ms", Type::Number)], Type::Number),
            MethodSignature::new("setUTCHours", vec![ParameterDeclaration::new("hours", Type::Number), ParameterDeclaration::optional("min", Type::Number), ParameterDeclaration::optional("sec", Type::Number), ParameterDeclaration::optional("ms", Type::Number)], Type::Number),
            MethodSignature::new("setDate", vec![ParameterDeclaration::new("date", Type::Number)], Type::Number),
            MethodSignature::new("setUTCDate", vec![ParameterDeclaration::new("date", Type::Number)], Type::Number),
            MethodSignature::new("setMonth", vec![ParameterDeclaration::new("month", Type::Number), ParameterDeclaration::optional("date", Type::Number)], Type::Number),
            MethodSignature::new("setUTCMonth", vec![ParameterDeclaration::new("month", Type::Number), ParameterDeclaration::optional("date", Type::Number)], Type::Number),
            MethodSignature::new("setFullYear", vec![ParameterDeclaration::new("year", Type::Number), ParameterDeclaration::optional("month", Type::Number), ParameterDeclaration::optional("date", Type::Number)], Type::Number),
            MethodSignature::new("setUTCFullYear", vec![ParameterDeclaration::new("year", Type::Number), ParameterDeclaration::optional("month", Type::Number), ParameterDeclaration::optional("date", Type::Number)], Type::Number),
            MethodSignature::new("toUTCString", vec![], Type::String),
            MethodSignature::new("toISOString", vec![], Type::String),
            MethodSignature::new("toJSON", vec![ParameterDeclaration::optional("key", Type::Any)], Type::String),
        ],
        call_signatures: vec![],
        construct_signatures: vec![],
        index_signatures: vec![],
    }
}

fn date_constructor_interface() -> InterfaceDeclaration {
    InterfaceDeclaration {
        name: "DateConstructor",
        type_parameters: vec![],
        extends: vec![],
        properties: vec![
            PropertySignature::readonly("prototype", Type::reference("Date")),
        ],
        methods: vec![
            MethodSignature::new("parse", vec![ParameterDeclaration::new("s", Type::String)], Type::Number),
            MethodSignature::new("UTC", vec![
                ParameterDeclaration::new("year", Type::Number),
                ParameterDeclaration::optional("month", Type::Number),
                ParameterDeclaration::optional("date", Type::Number),
                ParameterDeclaration::optional("hours", Type::Number),
                ParameterDeclaration::optional("minutes", Type::Number),
                ParameterDeclaration::optional("seconds", Type::Number),
                ParameterDeclaration::optional("ms", Type::Number),
            ], Type::Number),
            MethodSignature::new("now", vec![], Type::Number),
        ],
        call_signatures: vec![CallSignature {
            type_parameters: vec![],
            parameters: vec![],
            return_type: Type::String,
        }],
        construct_signatures: vec![
            ConstructSignature {
                type_parameters: vec![],
                parameters: vec![],
                return_type: Type::reference("Date"),
            },
            ConstructSignature {
                type_parameters: vec![],
                parameters: vec![ParameterDeclaration::new("value", Type::union(vec![Type::Number, Type::String]))],
                return_type: Type::reference("Date"),
            },
            ConstructSignature {
                type_parameters: vec![],
                parameters: vec![
                    ParameterDeclaration::new("year", Type::Number),
                    ParameterDeclaration::new("month", Type::Number),
                    ParameterDeclaration::optional("date", Type::Number),
                    ParameterDeclaration::optional("hours", Type::Number),
                    ParameterDeclaration::optional("minutes", Type::Number),
                    ParameterDeclaration::optional("seconds", Type::Number),
                    ParameterDeclaration::optional("ms", Type::Number),
                ],
                return_type: Type::reference("Date"),
            },
        ],
        index_signatures: vec![],
    }
}

fn regexp_interface() -> InterfaceDeclaration {
    InterfaceDeclaration {
        name: "RegExp",
        type_parameters: vec![],
        extends: vec![],
        properties: vec![
            PropertySignature::readonly("source", Type::String),
            PropertySignature::readonly("global", Type::Boolean),
            PropertySignature::readonly("ignoreCase", Type::Boolean),
            PropertySignature::readonly("multiline", Type::Boolean),
            PropertySignature::new("lastIndex", Type::Number),
        ],
        methods: vec![
            MethodSignature::new("exec", vec![ParameterDeclaration::new("string", Type::String)], Type::nullable(Type::reference("RegExpExecArray"))),
            MethodSignature::new("test", vec![ParameterDeclaration::new("string", Type::String)], Type::Boolean),
            MethodSignature::new("compile", vec![], Type::reference("this")),
        ],
        call_signatures: vec![],
        construct_signatures: vec![],
        index_signatures: vec![],
    }
}

fn regexp_constructor_interface() -> InterfaceDeclaration {
    InterfaceDeclaration {
        name: "RegExpConstructor",
        type_parameters: vec![],
        extends: vec![],
        properties: vec![
            PropertySignature::readonly("prototype", Type::reference("RegExp")),
        ],
        methods: vec![],
        call_signatures: vec![
            CallSignature {
                type_parameters: vec![],
                parameters: vec![ParameterDeclaration::new("pattern", Type::union(vec![Type::String, Type::reference("RegExp")])), ParameterDeclaration::optional("flags", Type::String)],
                return_type: Type::reference("RegExp"),
            },
        ],
        construct_signatures: vec![
            ConstructSignature {
                type_parameters: vec![],
                parameters: vec![ParameterDeclaration::new("pattern", Type::union(vec![Type::String, Type::reference("RegExp")])), ParameterDeclaration::optional("flags", Type::String)],
                return_type: Type::reference("RegExp"),
            },
        ],
        index_signatures: vec![],
    }
}

fn error_interface() -> InterfaceDeclaration {
    InterfaceDeclaration {
        name: "Error",
        type_parameters: vec![],
        extends: vec![],
        properties: vec![
            PropertySignature::new("name", Type::String),
            PropertySignature::new("message", Type::String),
            PropertySignature::optional("stack", Type::String),
        ],
        methods: vec![],
        call_signatures: vec![],
        construct_signatures: vec![],
        index_signatures: vec![],
    }
}

fn error_constructor_interface() -> InterfaceDeclaration {
    InterfaceDeclaration {
        name: "ErrorConstructor",
        type_parameters: vec![],
        extends: vec![],
        properties: vec![
            PropertySignature::readonly("prototype", Type::reference("Error")),
        ],
        methods: vec![],
        call_signatures: vec![CallSignature {
            type_parameters: vec![],
            parameters: vec![ParameterDeclaration::optional("message", Type::String)],
            return_type: Type::reference("Error"),
        }],
        construct_signatures: vec![ConstructSignature {
            type_parameters: vec![],
            parameters: vec![ParameterDeclaration::optional("message", Type::String)],
            return_type: Type::reference("Error"),
        }],
        index_signatures: vec![],
    }
}

fn eval_error_interface() -> InterfaceDeclaration {
    InterfaceDeclaration {
        name: "EvalError",
        type_parameters: vec![],
        extends: vec![Type::reference("Error")],
        properties: vec![],
        methods: vec![],
        call_signatures: vec![],
        construct_signatures: vec![],
        index_signatures: vec![],
    }
}

fn range_error_interface() -> InterfaceDeclaration {
    InterfaceDeclaration {
        name: "RangeError",
        type_parameters: vec![],
        extends: vec![Type::reference("Error")],
        properties: vec![],
        methods: vec![],
        call_signatures: vec![],
        construct_signatures: vec![],
        index_signatures: vec![],
    }
}

fn reference_error_interface() -> InterfaceDeclaration {
    InterfaceDeclaration {
        name: "ReferenceError",
        type_parameters: vec![],
        extends: vec![Type::reference("Error")],
        properties: vec![],
        methods: vec![],
        call_signatures: vec![],
        construct_signatures: vec![],
        index_signatures: vec![],
    }
}

fn syntax_error_interface() -> InterfaceDeclaration {
    InterfaceDeclaration {
        name: "SyntaxError",
        type_parameters: vec![],
        extends: vec![Type::reference("Error")],
        properties: vec![],
        methods: vec![],
        call_signatures: vec![],
        construct_signatures: vec![],
        index_signatures: vec![],
    }
}

fn type_error_interface() -> InterfaceDeclaration {
    InterfaceDeclaration {
        name: "TypeError",
        type_parameters: vec![],
        extends: vec![Type::reference("Error")],
        properties: vec![],
        methods: vec![],
        call_signatures: vec![],
        construct_signatures: vec![],
        index_signatures: vec![],
    }
}

fn uri_error_interface() -> InterfaceDeclaration {
    InterfaceDeclaration {
        name: "URIError",
        type_parameters: vec![],
        extends: vec![Type::reference("Error")],
        properties: vec![],
        methods: vec![],
        call_signatures: vec![],
        construct_signatures: vec![],
        index_signatures: vec![],
    }
}

fn json_interface() -> InterfaceDeclaration {
    InterfaceDeclaration {
        name: "JSON",
        type_parameters: vec![],
        extends: vec![],
        properties: vec![],
        methods: vec![
            MethodSignature::new("parse", vec![
                ParameterDeclaration::new("text", Type::String),
                ParameterDeclaration::optional("reviver", Type::func(
                    vec![
                        ParameterDeclaration::new("this", Type::Any),
                        ParameterDeclaration::new("key", Type::String),
                        ParameterDeclaration::new("value", Type::Any),
                    ],
                    Type::Any,
                )),
            ], Type::Any),
            MethodSignature::new("stringify", vec![
                ParameterDeclaration::new("value", Type::Any),
                ParameterDeclaration::optional("replacer", Type::union(vec![
                    Type::func(
                        vec![
                            ParameterDeclaration::new("this", Type::Any),
                            ParameterDeclaration::new("key", Type::String),
                            ParameterDeclaration::new("value", Type::Any),
                        ],
                        Type::Any,
                    ),
                    Type::array(Type::union(vec![Type::Number, Type::String])),
                ])),
                ParameterDeclaration::optional("space", Type::union(vec![Type::String, Type::Number])),
            ], Type::optional(Type::String)),
        ],
        call_signatures: vec![],
        construct_signatures: vec![],
        index_signatures: vec![],
    }
}

fn iterable_interface() -> InterfaceDeclaration {
    InterfaceDeclaration {
        name: "Iterable",
        type_parameters: vec![TypeParameter::new("T")],
        extends: vec![],
        properties: vec![],
        methods: vec![],
        call_signatures: vec![],
        construct_signatures: vec![],
        index_signatures: vec![],
    }
}

fn iterator_interface() -> InterfaceDeclaration {
    InterfaceDeclaration {
        name: "Iterator",
        type_parameters: vec![
            TypeParameter::new("T"),
            TypeParameter::with_default("TReturn", Type::Any),
            TypeParameter::with_default("TNext", Type::Undefined),
        ],
        extends: vec![],
        properties: vec![],
        methods: vec![
            MethodSignature::new(
                "next",
                vec![ParameterDeclaration::rest("args", Type::union(vec![
                    Type::Tuple(vec![]),
                    Type::Tuple(vec![Type::type_param("TNext")]),
                ]))],
                Type::reference1("IteratorResult", Type::type_param("T")),
            ),
            MethodSignature {
                name: "return",
                type_parameters: vec![],
                parameters: vec![ParameterDeclaration::optional("value", Type::type_param("TReturn"))],
                return_type: Type::reference2("IteratorResult", Type::type_param("T"), Type::type_param("TReturn")),
                optional: true,
            },
            MethodSignature {
                name: "throw",
                type_parameters: vec![],
                parameters: vec![ParameterDeclaration::optional("e", Type::Any)],
                return_type: Type::reference2("IteratorResult", Type::type_param("T"), Type::type_param("TReturn")),
                optional: true,
            },
        ],
        call_signatures: vec![],
        construct_signatures: vec![],
        index_signatures: vec![],
    }
}

fn iterator_result_interface() -> InterfaceDeclaration {
    InterfaceDeclaration {
        name: "IteratorResult",
        type_parameters: vec![
            TypeParameter::new("T"),
            TypeParameter::with_default("TReturn", Type::Any),
        ],
        extends: vec![],
        properties: vec![],
        methods: vec![],
        call_signatures: vec![],
        construct_signatures: vec![],
        index_signatures: vec![],
    }
}

fn array_like_interface() -> InterfaceDeclaration {
    InterfaceDeclaration {
        name: "ArrayLike",
        type_parameters: vec![TypeParameter::new("T")],
        extends: vec![],
        properties: vec![
            PropertySignature::readonly("length", Type::Number),
        ],
        methods: vec![],
        call_signatures: vec![],
        construct_signatures: vec![],
        index_signatures: vec![IndexSignature {
            key_name: "n",
            key_type: Type::Number,
            value_type: Type::type_param("T"),
            readonly: true,
        }],
    }
}

fn concat_array_interface() -> InterfaceDeclaration {
    InterfaceDeclaration {
        name: "ConcatArray",
        type_parameters: vec![TypeParameter::new("T")],
        extends: vec![],
        properties: vec![
            PropertySignature::readonly("length", Type::Number),
        ],
        methods: vec![
            MethodSignature::new(
                "join",
                vec![ParameterDeclaration::optional("separator", Type::String)],
                Type::String,
            ),
            MethodSignature::new(
                "slice",
                vec![
                    ParameterDeclaration::optional("start", Type::Number),
                    ParameterDeclaration::optional("end", Type::Number),
                ],
                Type::array(Type::type_param("T")),
            ),
        ],
        call_signatures: vec![],
        construct_signatures: vec![],
        index_signatures: vec![IndexSignature {
            key_name: "n",
            key_type: Type::Number,
            value_type: Type::type_param("T"),
            readonly: true,
        }],
    }
}

fn array_buffer_interface() -> InterfaceDeclaration {
    InterfaceDeclaration {
        name: "ArrayBuffer",
        type_parameters: vec![],
        extends: vec![],
        properties: vec![
            PropertySignature::readonly("byteLength", Type::Number),
        ],
        methods: vec![
            MethodSignature::new(
                "slice",
                vec![
                    ParameterDeclaration::new("begin", Type::Number),
                    ParameterDeclaration::optional("end", Type::Number),
                ],
                Type::reference("ArrayBuffer"),
            ),
        ],
        call_signatures: vec![],
        construct_signatures: vec![],
        index_signatures: vec![],
    }
}

fn array_buffer_constructor_interface() -> InterfaceDeclaration {
    InterfaceDeclaration {
        name: "ArrayBufferConstructor",
        type_parameters: vec![],
        extends: vec![],
        properties: vec![
            PropertySignature::readonly("prototype", Type::reference("ArrayBuffer")),
        ],
        methods: vec![
            MethodSignature::new(
                "isView",
                vec![ParameterDeclaration::new("arg", Type::Any)],
                Type::Boolean,
            ),
        ],
        call_signatures: vec![],
        construct_signatures: vec![ConstructSignature {
            type_parameters: vec![],
            parameters: vec![ParameterDeclaration::new("byteLength", Type::Number)],
            return_type: Type::reference("ArrayBuffer"),
        }],
        index_signatures: vec![],
    }
}

fn data_view_interface() -> InterfaceDeclaration {
    InterfaceDeclaration {
        name: "DataView",
        type_parameters: vec![],
        extends: vec![],
        properties: vec![
            PropertySignature::readonly("buffer", Type::reference("ArrayBuffer")),
            PropertySignature::readonly("byteLength", Type::Number),
            PropertySignature::readonly("byteOffset", Type::Number),
        ],
        methods: vec![
            MethodSignature::new("getFloat32", vec![ParameterDeclaration::new("byteOffset", Type::Number), ParameterDeclaration::optional("littleEndian", Type::Boolean)], Type::Number),
            MethodSignature::new("getFloat64", vec![ParameterDeclaration::new("byteOffset", Type::Number), ParameterDeclaration::optional("littleEndian", Type::Boolean)], Type::Number),
            MethodSignature::new("getInt8", vec![ParameterDeclaration::new("byteOffset", Type::Number)], Type::Number),
            MethodSignature::new("getInt16", vec![ParameterDeclaration::new("byteOffset", Type::Number), ParameterDeclaration::optional("littleEndian", Type::Boolean)], Type::Number),
            MethodSignature::new("getInt32", vec![ParameterDeclaration::new("byteOffset", Type::Number), ParameterDeclaration::optional("littleEndian", Type::Boolean)], Type::Number),
            MethodSignature::new("getUint8", vec![ParameterDeclaration::new("byteOffset", Type::Number)], Type::Number),
            MethodSignature::new("getUint16", vec![ParameterDeclaration::new("byteOffset", Type::Number), ParameterDeclaration::optional("littleEndian", Type::Boolean)], Type::Number),
            MethodSignature::new("getUint32", vec![ParameterDeclaration::new("byteOffset", Type::Number), ParameterDeclaration::optional("littleEndian", Type::Boolean)], Type::Number),
            MethodSignature::new("setFloat32", vec![ParameterDeclaration::new("byteOffset", Type::Number), ParameterDeclaration::new("value", Type::Number), ParameterDeclaration::optional("littleEndian", Type::Boolean)], Type::Void),
            MethodSignature::new("setFloat64", vec![ParameterDeclaration::new("byteOffset", Type::Number), ParameterDeclaration::new("value", Type::Number), ParameterDeclaration::optional("littleEndian", Type::Boolean)], Type::Void),
            MethodSignature::new("setInt8", vec![ParameterDeclaration::new("byteOffset", Type::Number), ParameterDeclaration::new("value", Type::Number)], Type::Void),
            MethodSignature::new("setInt16", vec![ParameterDeclaration::new("byteOffset", Type::Number), ParameterDeclaration::new("value", Type::Number), ParameterDeclaration::optional("littleEndian", Type::Boolean)], Type::Void),
            MethodSignature::new("setInt32", vec![ParameterDeclaration::new("byteOffset", Type::Number), ParameterDeclaration::new("value", Type::Number), ParameterDeclaration::optional("littleEndian", Type::Boolean)], Type::Void),
            MethodSignature::new("setUint8", vec![ParameterDeclaration::new("byteOffset", Type::Number), ParameterDeclaration::new("value", Type::Number)], Type::Void),
            MethodSignature::new("setUint16", vec![ParameterDeclaration::new("byteOffset", Type::Number), ParameterDeclaration::new("value", Type::Number), ParameterDeclaration::optional("littleEndian", Type::Boolean)], Type::Void),
            MethodSignature::new("setUint32", vec![ParameterDeclaration::new("byteOffset", Type::Number), ParameterDeclaration::new("value", Type::Number), ParameterDeclaration::optional("littleEndian", Type::Boolean)], Type::Void),
        ],
        call_signatures: vec![],
        construct_signatures: vec![],
        index_signatures: vec![],
    }
}
