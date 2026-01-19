//! DOM built-in type declarations
//!
//! This module provides DOM and Web API types:
//! - Document, Element, Event, Node, etc.
//! - Window, Console
//! - Fetch API (Request, Response, Headers)
//! - Storage (localStorage, sessionStorage)
//! - Web APIs

use super::types::*;

/// Get all DOM built-in declarations
pub fn get_dom_declarations() -> LibDeclarations {
    let mut lib = LibDeclarations::new();

    // Core DOM interfaces
    lib.interfaces.push(event_target_interface());
    lib.interfaces.push(event_interface());
    lib.interfaces.push(node_interface());
    lib.interfaces.push(element_interface());
    lib.interfaces.push(html_element_interface());
    lib.interfaces.push(document_interface());

    // Event types
    lib.interfaces.push(mouse_event_interface());
    lib.interfaces.push(keyboard_event_interface());
    lib.interfaces.push(custom_event_interface());

    // Collections
    lib.interfaces.push(node_list_interface());
    lib.interfaces.push(html_collection_interface());
    lib.interfaces.push(dom_token_list_interface());

    // Window and global
    lib.interfaces.push(window_interface());
    lib.interfaces.push(console_interface());
    lib.interfaces.push(location_interface());
    lib.interfaces.push(history_interface());
    lib.interfaces.push(navigator_interface());

    // Storage
    lib.interfaces.push(storage_interface());

    // Fetch API
    lib.interfaces.push(request_interface());
    lib.interfaces.push(response_interface());
    lib.interfaces.push(headers_interface());

    // URL APIs
    lib.interfaces.push(url_interface());
    lib.interfaces.push(url_search_params_interface());

    // Form elements
    lib.interfaces.push(html_form_element_interface());
    lib.interfaces.push(html_input_element_interface());
    lib.interfaces.push(form_data_interface());

    // Global variables
    lib.variables.push(VariableDeclaration {
        name: "window",
        type_: Type::reference("Window"),
        readonly: true,
    });
    lib.variables.push(VariableDeclaration {
        name: "document",
        type_: Type::reference("Document"),
        readonly: true,
    });
    lib.variables.push(VariableDeclaration {
        name: "console",
        type_: Type::reference("Console"),
        readonly: true,
    });
    lib.variables.push(VariableDeclaration {
        name: "localStorage",
        type_: Type::reference("Storage"),
        readonly: true,
    });
    lib.variables.push(VariableDeclaration {
        name: "sessionStorage",
        type_: Type::reference("Storage"),
        readonly: true,
    });
    lib.variables.push(VariableDeclaration {
        name: "location",
        type_: Type::reference("Location"),
        readonly: true,
    });
    lib.variables.push(VariableDeclaration {
        name: "history",
        type_: Type::reference("History"),
        readonly: true,
    });
    lib.variables.push(VariableDeclaration {
        name: "navigator",
        type_: Type::reference("Navigator"),
        readonly: true,
    });

    // Global functions
    lib.functions.push(FunctionDeclaration {
        name: "fetch",
        type_parameters: vec![],
        parameters: vec![
            ParameterDeclaration::new("input", Type::union(vec![Type::String, Type::reference("URL"), Type::reference("Request")])),
            ParameterDeclaration::optional("init", Type::reference("RequestInit")),
        ],
        return_type: Type::reference1("Promise", Type::reference("Response")),
    });
    lib.functions.push(FunctionDeclaration {
        name: "setTimeout",
        type_parameters: vec![],
        parameters: vec![
            ParameterDeclaration::new("handler", Type::union(vec![Type::String, Type::reference("Function")])),
            ParameterDeclaration::optional("timeout", Type::Number),
            ParameterDeclaration::rest("arguments", Type::array(Type::Any)),
        ],
        return_type: Type::Number,
    });
    lib.functions.push(FunctionDeclaration {
        name: "clearTimeout",
        type_parameters: vec![],
        parameters: vec![ParameterDeclaration::optional("id", Type::Number)],
        return_type: Type::Void,
    });
    lib.functions.push(FunctionDeclaration {
        name: "setInterval",
        type_parameters: vec![],
        parameters: vec![
            ParameterDeclaration::new("handler", Type::union(vec![Type::String, Type::reference("Function")])),
            ParameterDeclaration::optional("timeout", Type::Number),
            ParameterDeclaration::rest("arguments", Type::array(Type::Any)),
        ],
        return_type: Type::Number,
    });
    lib.functions.push(FunctionDeclaration {
        name: "clearInterval",
        type_parameters: vec![],
        parameters: vec![ParameterDeclaration::optional("id", Type::Number)],
        return_type: Type::Void,
    });
    lib.functions.push(FunctionDeclaration {
        name: "alert",
        type_parameters: vec![],
        parameters: vec![ParameterDeclaration::optional("message", Type::Any)],
        return_type: Type::Void,
    });
    lib.functions.push(FunctionDeclaration {
        name: "confirm",
        type_parameters: vec![],
        parameters: vec![ParameterDeclaration::optional("message", Type::String)],
        return_type: Type::Boolean,
    });
    lib.functions.push(FunctionDeclaration {
        name: "prompt",
        type_parameters: vec![],
        parameters: vec![
            ParameterDeclaration::optional("message", Type::String),
            ParameterDeclaration::optional("default", Type::String),
        ],
        return_type: Type::nullable(Type::String),
    });

    lib
}

fn event_target_interface() -> InterfaceDeclaration {
    InterfaceDeclaration {
        name: "EventTarget",
        type_parameters: vec![],
        extends: vec![],
        properties: vec![],
        methods: vec![
            MethodSignature::new(
                "addEventListener",
                vec![
                    ParameterDeclaration::new("type", Type::String),
                    ParameterDeclaration::new("listener", Type::nullable(Type::reference("EventListenerOrEventListenerObject"))),
                    ParameterDeclaration::optional("options", Type::union(vec![Type::Boolean, Type::reference("AddEventListenerOptions")])),
                ],
                Type::Void,
            ),
            MethodSignature::new(
                "removeEventListener",
                vec![
                    ParameterDeclaration::new("type", Type::String),
                    ParameterDeclaration::new("listener", Type::nullable(Type::reference("EventListenerOrEventListenerObject"))),
                    ParameterDeclaration::optional("options", Type::union(vec![Type::Boolean, Type::reference("EventListenerOptions")])),
                ],
                Type::Void,
            ),
            MethodSignature::new(
                "dispatchEvent",
                vec![ParameterDeclaration::new("event", Type::reference("Event"))],
                Type::Boolean,
            ),
        ],
        call_signatures: vec![],
        construct_signatures: vec![],
        index_signatures: vec![],
    }
}

fn event_interface() -> InterfaceDeclaration {
    InterfaceDeclaration {
        name: "Event",
        type_parameters: vec![],
        extends: vec![],
        properties: vec![
            PropertySignature::readonly("bubbles", Type::Boolean),
            PropertySignature::new("cancelBubble", Type::Boolean),
            PropertySignature::readonly("cancelable", Type::Boolean),
            PropertySignature::readonly("composed", Type::Boolean),
            PropertySignature::readonly("currentTarget", Type::nullable(Type::reference("EventTarget"))),
            PropertySignature::readonly("defaultPrevented", Type::Boolean),
            PropertySignature::readonly("eventPhase", Type::Number),
            PropertySignature::readonly("isTrusted", Type::Boolean),
            PropertySignature::readonly("target", Type::nullable(Type::reference("EventTarget"))),
            PropertySignature::readonly("timeStamp", Type::Number),
            PropertySignature::readonly("type", Type::String),
        ],
        methods: vec![
            MethodSignature::new("composedPath", vec![], Type::array(Type::reference("EventTarget"))),
            MethodSignature::new("preventDefault", vec![], Type::Void),
            MethodSignature::new("stopImmediatePropagation", vec![], Type::Void),
            MethodSignature::new("stopPropagation", vec![], Type::Void),
        ],
        call_signatures: vec![],
        construct_signatures: vec![],
        index_signatures: vec![],
    }
}

fn node_interface() -> InterfaceDeclaration {
    InterfaceDeclaration {
        name: "Node",
        type_parameters: vec![],
        extends: vec![Type::reference("EventTarget")],
        properties: vec![
            PropertySignature::readonly("baseURI", Type::String),
            PropertySignature::readonly("childNodes", Type::reference("NodeListOf")),
            PropertySignature::readonly("firstChild", Type::nullable(Type::reference("ChildNode"))),
            PropertySignature::readonly("isConnected", Type::Boolean),
            PropertySignature::readonly("lastChild", Type::nullable(Type::reference("ChildNode"))),
            PropertySignature::readonly("nextSibling", Type::nullable(Type::reference("ChildNode"))),
            PropertySignature::new("nodeValue", Type::nullable(Type::String)),
            PropertySignature::readonly("nodeName", Type::String),
            PropertySignature::readonly("nodeType", Type::Number),
            PropertySignature::readonly("ownerDocument", Type::nullable(Type::reference("Document"))),
            PropertySignature::readonly("parentElement", Type::nullable(Type::reference("HTMLElement"))),
            PropertySignature::readonly("parentNode", Type::nullable(Type::reference("ParentNode"))),
            PropertySignature::readonly("previousSibling", Type::nullable(Type::reference("ChildNode"))),
            PropertySignature::new("textContent", Type::nullable(Type::String)),
        ],
        methods: vec![
            MethodSignature::generic(
                "appendChild",
                vec![TypeParameter::with_constraint("T", Type::reference("Node"))],
                vec![ParameterDeclaration::new("node", Type::type_param("T"))],
                Type::type_param("T"),
            ),
            MethodSignature::new(
                "cloneNode",
                vec![ParameterDeclaration::optional("deep", Type::Boolean)],
                Type::reference("Node"),
            ),
            MethodSignature::new(
                "contains",
                vec![ParameterDeclaration::new("other", Type::nullable(Type::reference("Node")))],
                Type::Boolean,
            ),
            MethodSignature::new("hasChildNodes", vec![], Type::Boolean),
            MethodSignature::generic(
                "insertBefore",
                vec![TypeParameter::with_constraint("T", Type::reference("Node"))],
                vec![
                    ParameterDeclaration::new("node", Type::type_param("T")),
                    ParameterDeclaration::new("child", Type::nullable(Type::reference("Node"))),
                ],
                Type::type_param("T"),
            ),
            MethodSignature::new("normalize", vec![], Type::Void),
            MethodSignature::generic(
                "removeChild",
                vec![TypeParameter::with_constraint("T", Type::reference("Node"))],
                vec![ParameterDeclaration::new("child", Type::type_param("T"))],
                Type::type_param("T"),
            ),
            MethodSignature::generic(
                "replaceChild",
                vec![TypeParameter::with_constraint("T", Type::reference("Node"))],
                vec![
                    ParameterDeclaration::new("node", Type::reference("Node")),
                    ParameterDeclaration::new("child", Type::type_param("T")),
                ],
                Type::type_param("T"),
            ),
        ],
        call_signatures: vec![],
        construct_signatures: vec![],
        index_signatures: vec![],
    }
}

fn element_interface() -> InterfaceDeclaration {
    InterfaceDeclaration {
        name: "Element",
        type_parameters: vec![],
        extends: vec![Type::reference("Node")],
        properties: vec![
            PropertySignature::readonly("attributes", Type::reference("NamedNodeMap")),
            PropertySignature::readonly("classList", Type::reference("DOMTokenList")),
            PropertySignature::new("className", Type::String),
            PropertySignature::readonly("clientHeight", Type::Number),
            PropertySignature::readonly("clientLeft", Type::Number),
            PropertySignature::readonly("clientTop", Type::Number),
            PropertySignature::readonly("clientWidth", Type::Number),
            PropertySignature::new("id", Type::String),
            PropertySignature::new("innerHTML", Type::String),
            PropertySignature::readonly("localName", Type::String),
            PropertySignature::new("outerHTML", Type::String),
            PropertySignature::readonly("scrollHeight", Type::Number),
            PropertySignature::new("scrollLeft", Type::Number),
            PropertySignature::new("scrollTop", Type::Number),
            PropertySignature::readonly("scrollWidth", Type::Number),
            PropertySignature::readonly("tagName", Type::String),
        ],
        methods: vec![
            MethodSignature::new("closest", vec![ParameterDeclaration::new("selectors", Type::String)], Type::nullable(Type::reference("Element"))),
            MethodSignature::new("getAttribute", vec![ParameterDeclaration::new("qualifiedName", Type::String)], Type::nullable(Type::String)),
            MethodSignature::new("getBoundingClientRect", vec![], Type::reference("DOMRect")),
            MethodSignature::new("getElementsByClassName", vec![ParameterDeclaration::new("classNames", Type::String)], Type::reference("HTMLCollectionOf")),
            MethodSignature::new("getElementsByTagName", vec![ParameterDeclaration::new("qualifiedName", Type::String)], Type::reference("HTMLCollectionOf")),
            MethodSignature::new("hasAttribute", vec![ParameterDeclaration::new("qualifiedName", Type::String)], Type::Boolean),
            MethodSignature::new("matches", vec![ParameterDeclaration::new("selectors", Type::String)], Type::Boolean),
            MethodSignature::new("querySelector", vec![ParameterDeclaration::new("selectors", Type::String)], Type::nullable(Type::reference("Element"))),
            MethodSignature::new("querySelectorAll", vec![ParameterDeclaration::new("selectors", Type::String)], Type::reference("NodeListOf")),
            MethodSignature::new("removeAttribute", vec![ParameterDeclaration::new("qualifiedName", Type::String)], Type::Void),
            MethodSignature::new("scrollIntoView", vec![ParameterDeclaration::optional("arg", Type::union(vec![Type::Boolean, Type::reference("ScrollIntoViewOptions")]))], Type::Void),
            MethodSignature::new("setAttribute", vec![
                ParameterDeclaration::new("qualifiedName", Type::String),
                ParameterDeclaration::new("value", Type::String),
            ], Type::Void),
            MethodSignature::new("toggleAttribute", vec![
                ParameterDeclaration::new("qualifiedName", Type::String),
                ParameterDeclaration::optional("force", Type::Boolean),
            ], Type::Boolean),
        ],
        call_signatures: vec![],
        construct_signatures: vec![],
        index_signatures: vec![],
    }
}

fn html_element_interface() -> InterfaceDeclaration {
    InterfaceDeclaration {
        name: "HTMLElement",
        type_parameters: vec![],
        extends: vec![Type::reference("Element")],
        properties: vec![
            PropertySignature::new("accessKey", Type::String),
            PropertySignature::new("contentEditable", Type::String),
            PropertySignature::new("dir", Type::String),
            PropertySignature::new("draggable", Type::Boolean),
            PropertySignature::new("hidden", Type::Boolean),
            PropertySignature::new("innerText", Type::String),
            PropertySignature::readonly("isContentEditable", Type::Boolean),
            PropertySignature::new("lang", Type::String),
            PropertySignature::readonly("offsetHeight", Type::Number),
            PropertySignature::readonly("offsetLeft", Type::Number),
            PropertySignature::readonly("offsetParent", Type::nullable(Type::reference("Element"))),
            PropertySignature::readonly("offsetTop", Type::Number),
            PropertySignature::readonly("offsetWidth", Type::Number),
            PropertySignature::new("outerText", Type::String),
            PropertySignature::new("spellcheck", Type::Boolean),
            PropertySignature::readonly("style", Type::reference("CSSStyleDeclaration")),
            PropertySignature::new("tabIndex", Type::Number),
            PropertySignature::new("title", Type::String),
        ],
        methods: vec![
            MethodSignature::new("blur", vec![], Type::Void),
            MethodSignature::new("click", vec![], Type::Void),
            MethodSignature::new("focus", vec![ParameterDeclaration::optional("options", Type::reference("FocusOptions"))], Type::Void),
        ],
        call_signatures: vec![],
        construct_signatures: vec![],
        index_signatures: vec![],
    }
}

fn document_interface() -> InterfaceDeclaration {
    InterfaceDeclaration {
        name: "Document",
        type_parameters: vec![],
        extends: vec![Type::reference("Node")],
        properties: vec![
            PropertySignature::readonly("URL", Type::String),
            PropertySignature::readonly("activeElement", Type::nullable(Type::reference("Element"))),
            PropertySignature::new("body", Type::reference("HTMLElement")),
            PropertySignature::readonly("characterSet", Type::String),
            PropertySignature::new("cookie", Type::String),
            PropertySignature::readonly("defaultView", Type::nullable(Type::reference("Window"))),
            PropertySignature::readonly("doctype", Type::nullable(Type::reference("DocumentType"))),
            PropertySignature::readonly("documentElement", Type::reference("HTMLElement")),
            PropertySignature::readonly("documentURI", Type::String),
            PropertySignature::readonly("head", Type::reference("HTMLHeadElement")),
            PropertySignature::readonly("hidden", Type::Boolean),
            PropertySignature::readonly("readyState", Type::reference("DocumentReadyState")),
            PropertySignature::readonly("referrer", Type::String),
            PropertySignature::new("title", Type::String),
        ],
        methods: vec![
            MethodSignature::new("adoptNode", vec![ParameterDeclaration::new("node", Type::reference("Node"))], Type::reference("Node")),
            MethodSignature::new("createComment", vec![ParameterDeclaration::new("data", Type::String)], Type::reference("Comment")),
            MethodSignature::new("createDocumentFragment", vec![], Type::reference("DocumentFragment")),
            MethodSignature::new("createElement", vec![
                ParameterDeclaration::new("tagName", Type::String),
                ParameterDeclaration::optional("options", Type::reference("ElementCreationOptions")),
            ], Type::reference("HTMLElement")),
            MethodSignature::new("createEvent", vec![ParameterDeclaration::new("eventInterface", Type::String)], Type::reference("Event")),
            MethodSignature::new("createRange", vec![], Type::reference("Range")),
            MethodSignature::new("createTextNode", vec![ParameterDeclaration::new("data", Type::String)], Type::reference("Text")),
            MethodSignature::new("execCommand", vec![
                ParameterDeclaration::new("commandId", Type::String),
                ParameterDeclaration::optional("showUI", Type::Boolean),
                ParameterDeclaration::optional("value", Type::String),
            ], Type::Boolean),
            MethodSignature::new("getElementById", vec![ParameterDeclaration::new("elementId", Type::String)], Type::nullable(Type::reference("HTMLElement"))),
            MethodSignature::new("getElementsByClassName", vec![ParameterDeclaration::new("classNames", Type::String)], Type::reference("HTMLCollectionOf")),
            MethodSignature::new("getElementsByName", vec![ParameterDeclaration::new("elementName", Type::String)], Type::reference("NodeListOf")),
            MethodSignature::new("getElementsByTagName", vec![ParameterDeclaration::new("qualifiedName", Type::String)], Type::reference("HTMLCollectionOf")),
            MethodSignature::new("hasFocus", vec![], Type::Boolean),
            MethodSignature::new("open", vec![], Type::reference("Document")),
            MethodSignature::new("close", vec![], Type::Void),
            MethodSignature::new("write", vec![ParameterDeclaration::rest("text", Type::array(Type::String))], Type::Void),
            MethodSignature::new("writeln", vec![ParameterDeclaration::rest("text", Type::array(Type::String))], Type::Void),
            MethodSignature::new("querySelector", vec![ParameterDeclaration::new("selectors", Type::String)], Type::nullable(Type::reference("Element"))),
            MethodSignature::new("querySelectorAll", vec![ParameterDeclaration::new("selectors", Type::String)], Type::reference("NodeListOf")),
        ],
        call_signatures: vec![],
        construct_signatures: vec![],
        index_signatures: vec![],
    }
}

fn mouse_event_interface() -> InterfaceDeclaration {
    InterfaceDeclaration {
        name: "MouseEvent",
        type_parameters: vec![],
        extends: vec![Type::reference("UIEvent")],
        properties: vec![
            PropertySignature::readonly("altKey", Type::Boolean),
            PropertySignature::readonly("button", Type::Number),
            PropertySignature::readonly("buttons", Type::Number),
            PropertySignature::readonly("clientX", Type::Number),
            PropertySignature::readonly("clientY", Type::Number),
            PropertySignature::readonly("ctrlKey", Type::Boolean),
            PropertySignature::readonly("metaKey", Type::Boolean),
            PropertySignature::readonly("offsetX", Type::Number),
            PropertySignature::readonly("offsetY", Type::Number),
            PropertySignature::readonly("pageX", Type::Number),
            PropertySignature::readonly("pageY", Type::Number),
            PropertySignature::readonly("relatedTarget", Type::nullable(Type::reference("EventTarget"))),
            PropertySignature::readonly("screenX", Type::Number),
            PropertySignature::readonly("screenY", Type::Number),
            PropertySignature::readonly("shiftKey", Type::Boolean),
            PropertySignature::readonly("x", Type::Number),
            PropertySignature::readonly("y", Type::Number),
        ],
        methods: vec![
            MethodSignature::new("getModifierState", vec![ParameterDeclaration::new("keyArg", Type::String)], Type::Boolean),
        ],
        call_signatures: vec![],
        construct_signatures: vec![],
        index_signatures: vec![],
    }
}

fn keyboard_event_interface() -> InterfaceDeclaration {
    InterfaceDeclaration {
        name: "KeyboardEvent",
        type_parameters: vec![],
        extends: vec![Type::reference("UIEvent")],
        properties: vec![
            PropertySignature::readonly("altKey", Type::Boolean),
            PropertySignature::readonly("code", Type::String),
            PropertySignature::readonly("ctrlKey", Type::Boolean),
            PropertySignature::readonly("isComposing", Type::Boolean),
            PropertySignature::readonly("key", Type::String),
            PropertySignature::readonly("location", Type::Number),
            PropertySignature::readonly("metaKey", Type::Boolean),
            PropertySignature::readonly("repeat", Type::Boolean),
            PropertySignature::readonly("shiftKey", Type::Boolean),
        ],
        methods: vec![
            MethodSignature::new("getModifierState", vec![ParameterDeclaration::new("keyArg", Type::String)], Type::Boolean),
        ],
        call_signatures: vec![],
        construct_signatures: vec![],
        index_signatures: vec![],
    }
}

fn custom_event_interface() -> InterfaceDeclaration {
    InterfaceDeclaration {
        name: "CustomEvent",
        type_parameters: vec![TypeParameter::with_default("T", Type::Any)],
        extends: vec![Type::reference("Event")],
        properties: vec![
            PropertySignature::readonly("detail", Type::type_param("T")),
        ],
        methods: vec![
            MethodSignature::new("initCustomEvent", vec![
                ParameterDeclaration::new("type", Type::String),
                ParameterDeclaration::optional("bubbles", Type::Boolean),
                ParameterDeclaration::optional("cancelable", Type::Boolean),
                ParameterDeclaration::optional("detail", Type::type_param("T")),
            ], Type::Void),
        ],
        call_signatures: vec![],
        construct_signatures: vec![],
        index_signatures: vec![],
    }
}

fn node_list_interface() -> InterfaceDeclaration {
    InterfaceDeclaration {
        name: "NodeList",
        type_parameters: vec![],
        extends: vec![],
        properties: vec![
            PropertySignature::readonly("length", Type::Number),
        ],
        methods: vec![
            MethodSignature::new("item", vec![ParameterDeclaration::new("index", Type::Number)], Type::nullable(Type::reference("Node"))),
            MethodSignature::new("forEach", vec![
                ParameterDeclaration::new("callbackfn", Type::func(
                    vec![
                        ParameterDeclaration::new("value", Type::reference("Node")),
                        ParameterDeclaration::new("key", Type::Number),
                        ParameterDeclaration::new("parent", Type::reference("NodeList")),
                    ],
                    Type::Void,
                )),
                ParameterDeclaration::optional("thisArg", Type::Any),
            ], Type::Void),
            MethodSignature::new("entries", vec![], Type::reference1("IterableIterator", Type::Tuple(vec![Type::Number, Type::reference("Node")]))),
            MethodSignature::new("keys", vec![], Type::reference1("IterableIterator", Type::Number)),
            MethodSignature::new("values", vec![], Type::reference1("IterableIterator", Type::reference("Node"))),
        ],
        call_signatures: vec![],
        construct_signatures: vec![],
        index_signatures: vec![IndexSignature {
            key_name: "index",
            key_type: Type::Number,
            value_type: Type::reference("Node"),
            readonly: true,
        }],
    }
}

fn html_collection_interface() -> InterfaceDeclaration {
    InterfaceDeclaration {
        name: "HTMLCollection",
        type_parameters: vec![],
        extends: vec![],
        properties: vec![
            PropertySignature::readonly("length", Type::Number),
        ],
        methods: vec![
            MethodSignature::new("item", vec![ParameterDeclaration::new("index", Type::Number)], Type::nullable(Type::reference("Element"))),
            MethodSignature::new("namedItem", vec![ParameterDeclaration::new("name", Type::String)], Type::nullable(Type::reference("Element"))),
        ],
        call_signatures: vec![],
        construct_signatures: vec![],
        index_signatures: vec![IndexSignature {
            key_name: "index",
            key_type: Type::Number,
            value_type: Type::reference("Element"),
            readonly: true,
        }],
    }
}

fn dom_token_list_interface() -> InterfaceDeclaration {
    InterfaceDeclaration {
        name: "DOMTokenList",
        type_parameters: vec![],
        extends: vec![],
        properties: vec![
            PropertySignature::readonly("length", Type::Number),
            PropertySignature::new("value", Type::String),
        ],
        methods: vec![
            MethodSignature::new("add", vec![ParameterDeclaration::rest("tokens", Type::array(Type::String))], Type::Void),
            MethodSignature::new("contains", vec![ParameterDeclaration::new("token", Type::String)], Type::Boolean),
            MethodSignature::new("item", vec![ParameterDeclaration::new("index", Type::Number)], Type::nullable(Type::String)),
            MethodSignature::new("remove", vec![ParameterDeclaration::rest("tokens", Type::array(Type::String))], Type::Void),
            MethodSignature::new("replace", vec![
                ParameterDeclaration::new("token", Type::String),
                ParameterDeclaration::new("newToken", Type::String),
            ], Type::Boolean),
            MethodSignature::new("toggle", vec![
                ParameterDeclaration::new("token", Type::String),
                ParameterDeclaration::optional("force", Type::Boolean),
            ], Type::Boolean),
            MethodSignature::new("forEach", vec![
                ParameterDeclaration::new("callbackfn", Type::func(
                    vec![
                        ParameterDeclaration::new("value", Type::String),
                        ParameterDeclaration::new("key", Type::Number),
                        ParameterDeclaration::new("parent", Type::reference("DOMTokenList")),
                    ],
                    Type::Void,
                )),
                ParameterDeclaration::optional("thisArg", Type::Any),
            ], Type::Void),
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

fn window_interface() -> InterfaceDeclaration {
    InterfaceDeclaration {
        name: "Window",
        type_parameters: vec![],
        extends: vec![Type::reference("EventTarget")],
        properties: vec![
            PropertySignature::readonly("closed", Type::Boolean),
            PropertySignature::readonly("document", Type::reference("Document")),
            PropertySignature::readonly("history", Type::reference("History")),
            PropertySignature::readonly("innerHeight", Type::Number),
            PropertySignature::readonly("innerWidth", Type::Number),
            PropertySignature::readonly("length", Type::Number),
            PropertySignature::readonly("location", Type::reference("Location")),
            PropertySignature::new("name", Type::String),
            PropertySignature::readonly("navigator", Type::reference("Navigator")),
            PropertySignature::new("opener", Type::Any),
            PropertySignature::readonly("outerHeight", Type::Number),
            PropertySignature::readonly("outerWidth", Type::Number),
            PropertySignature::readonly("parent", Type::nullable(Type::reference("WindowProxy"))),
            PropertySignature::readonly("screen", Type::reference("Screen")),
            PropertySignature::readonly("scrollX", Type::Number),
            PropertySignature::readonly("scrollY", Type::Number),
            PropertySignature::readonly("self", Type::reference("Window")),
            PropertySignature::new("status", Type::String),
            PropertySignature::readonly("top", Type::nullable(Type::reference("WindowProxy"))),
            PropertySignature::readonly("window", Type::reference("Window")),
        ],
        methods: vec![
            MethodSignature::new("alert", vec![ParameterDeclaration::optional("message", Type::Any)], Type::Void),
            MethodSignature::new("blur", vec![], Type::Void),
            MethodSignature::new("close", vec![], Type::Void),
            MethodSignature::new("confirm", vec![ParameterDeclaration::optional("message", Type::String)], Type::Boolean),
            MethodSignature::new("focus", vec![], Type::Void),
            MethodSignature::new("getComputedStyle", vec![
                ParameterDeclaration::new("elt", Type::reference("Element")),
                ParameterDeclaration::optional("pseudoElt", Type::nullable(Type::String)),
            ], Type::reference("CSSStyleDeclaration")),
            MethodSignature::new("open", vec![
                ParameterDeclaration::optional("url", Type::union(vec![Type::String, Type::reference("URL")])),
                ParameterDeclaration::optional("target", Type::String),
                ParameterDeclaration::optional("features", Type::String),
            ], Type::nullable(Type::reference("WindowProxy"))),
            MethodSignature::new("print", vec![], Type::Void),
            MethodSignature::new("prompt", vec![
                ParameterDeclaration::optional("message", Type::String),
                ParameterDeclaration::optional("default", Type::String),
            ], Type::nullable(Type::String)),
            MethodSignature::new("scroll", vec![
                ParameterDeclaration::optional("x", Type::Number),
                ParameterDeclaration::optional("y", Type::Number),
            ], Type::Void),
            MethodSignature::new("scrollBy", vec![
                ParameterDeclaration::optional("x", Type::Number),
                ParameterDeclaration::optional("y", Type::Number),
            ], Type::Void),
            MethodSignature::new("scrollTo", vec![
                ParameterDeclaration::optional("x", Type::Number),
                ParameterDeclaration::optional("y", Type::Number),
            ], Type::Void),
        ],
        call_signatures: vec![],
        construct_signatures: vec![],
        index_signatures: vec![],
    }
}

fn console_interface() -> InterfaceDeclaration {
    InterfaceDeclaration {
        name: "Console",
        type_parameters: vec![],
        extends: vec![],
        properties: vec![],
        methods: vec![
            MethodSignature::new("assert", vec![
                ParameterDeclaration::optional("condition", Type::Boolean),
                ParameterDeclaration::rest("data", Type::array(Type::Any)),
            ], Type::Void),
            MethodSignature::new("clear", vec![], Type::Void),
            MethodSignature::new("count", vec![ParameterDeclaration::optional("label", Type::String)], Type::Void),
            MethodSignature::new("countReset", vec![ParameterDeclaration::optional("label", Type::String)], Type::Void),
            MethodSignature::new("debug", vec![ParameterDeclaration::rest("data", Type::array(Type::Any))], Type::Void),
            MethodSignature::new("dir", vec![ParameterDeclaration::optional("item", Type::Any)], Type::Void),
            MethodSignature::new("error", vec![ParameterDeclaration::rest("data", Type::array(Type::Any))], Type::Void),
            MethodSignature::new("group", vec![ParameterDeclaration::rest("data", Type::array(Type::Any))], Type::Void),
            MethodSignature::new("groupCollapsed", vec![ParameterDeclaration::rest("data", Type::array(Type::Any))], Type::Void),
            MethodSignature::new("groupEnd", vec![], Type::Void),
            MethodSignature::new("info", vec![ParameterDeclaration::rest("data", Type::array(Type::Any))], Type::Void),
            MethodSignature::new("log", vec![ParameterDeclaration::rest("data", Type::array(Type::Any))], Type::Void),
            MethodSignature::new("table", vec![ParameterDeclaration::optional("tabularData", Type::Any)], Type::Void),
            MethodSignature::new("time", vec![ParameterDeclaration::optional("label", Type::String)], Type::Void),
            MethodSignature::new("timeEnd", vec![ParameterDeclaration::optional("label", Type::String)], Type::Void),
            MethodSignature::new("timeLog", vec![ParameterDeclaration::optional("label", Type::String)], Type::Void),
            MethodSignature::new("trace", vec![ParameterDeclaration::rest("data", Type::array(Type::Any))], Type::Void),
            MethodSignature::new("warn", vec![ParameterDeclaration::rest("data", Type::array(Type::Any))], Type::Void),
        ],
        call_signatures: vec![],
        construct_signatures: vec![],
        index_signatures: vec![],
    }
}

fn location_interface() -> InterfaceDeclaration {
    InterfaceDeclaration {
        name: "Location",
        type_parameters: vec![],
        extends: vec![],
        properties: vec![
            PropertySignature::new("hash", Type::String),
            PropertySignature::new("host", Type::String),
            PropertySignature::new("hostname", Type::String),
            PropertySignature::new("href", Type::String),
            PropertySignature::readonly("origin", Type::String),
            PropertySignature::new("pathname", Type::String),
            PropertySignature::new("port", Type::String),
            PropertySignature::new("protocol", Type::String),
            PropertySignature::new("search", Type::String),
        ],
        methods: vec![
            MethodSignature::new("assign", vec![ParameterDeclaration::new("url", Type::union(vec![Type::String, Type::reference("URL")]))], Type::Void),
            MethodSignature::new("reload", vec![], Type::Void),
            MethodSignature::new("replace", vec![ParameterDeclaration::new("url", Type::union(vec![Type::String, Type::reference("URL")]))], Type::Void),
        ],
        call_signatures: vec![],
        construct_signatures: vec![],
        index_signatures: vec![],
    }
}

fn history_interface() -> InterfaceDeclaration {
    InterfaceDeclaration {
        name: "History",
        type_parameters: vec![],
        extends: vec![],
        properties: vec![
            PropertySignature::readonly("length", Type::Number),
            PropertySignature::new("scrollRestoration", Type::reference("ScrollRestoration")),
            PropertySignature::readonly("state", Type::Any),
        ],
        methods: vec![
            MethodSignature::new("back", vec![], Type::Void),
            MethodSignature::new("forward", vec![], Type::Void),
            MethodSignature::new("go", vec![ParameterDeclaration::optional("delta", Type::Number)], Type::Void),
            MethodSignature::new("pushState", vec![
                ParameterDeclaration::new("data", Type::Any),
                ParameterDeclaration::new("unused", Type::String),
                ParameterDeclaration::optional("url", Type::nullable(Type::union(vec![Type::String, Type::reference("URL")]))),
            ], Type::Void),
            MethodSignature::new("replaceState", vec![
                ParameterDeclaration::new("data", Type::Any),
                ParameterDeclaration::new("unused", Type::String),
                ParameterDeclaration::optional("url", Type::nullable(Type::union(vec![Type::String, Type::reference("URL")]))),
            ], Type::Void),
        ],
        call_signatures: vec![],
        construct_signatures: vec![],
        index_signatures: vec![],
    }
}

fn navigator_interface() -> InterfaceDeclaration {
    InterfaceDeclaration {
        name: "Navigator",
        type_parameters: vec![],
        extends: vec![],
        properties: vec![
            PropertySignature::readonly("cookieEnabled", Type::Boolean),
            PropertySignature::readonly("language", Type::String),
            PropertySignature::readonly("languages", Type::reference1("ReadonlyArray", Type::String)),
            PropertySignature::readonly("maxTouchPoints", Type::Number),
            PropertySignature::readonly("onLine", Type::Boolean),
            PropertySignature::readonly("platform", Type::String),
            PropertySignature::readonly("userAgent", Type::String),
        ],
        methods: vec![],
        call_signatures: vec![],
        construct_signatures: vec![],
        index_signatures: vec![],
    }
}

fn storage_interface() -> InterfaceDeclaration {
    InterfaceDeclaration {
        name: "Storage",
        type_parameters: vec![],
        extends: vec![],
        properties: vec![
            PropertySignature::readonly("length", Type::Number),
        ],
        methods: vec![
            MethodSignature::new("clear", vec![], Type::Void),
            MethodSignature::new("getItem", vec![ParameterDeclaration::new("key", Type::String)], Type::nullable(Type::String)),
            MethodSignature::new("key", vec![ParameterDeclaration::new("index", Type::Number)], Type::nullable(Type::String)),
            MethodSignature::new("removeItem", vec![ParameterDeclaration::new("key", Type::String)], Type::Void),
            MethodSignature::new("setItem", vec![
                ParameterDeclaration::new("key", Type::String),
                ParameterDeclaration::new("value", Type::String),
            ], Type::Void),
        ],
        call_signatures: vec![],
        construct_signatures: vec![],
        index_signatures: vec![IndexSignature {
            key_name: "name",
            key_type: Type::String,
            value_type: Type::Any,
            readonly: false,
        }],
    }
}

fn request_interface() -> InterfaceDeclaration {
    InterfaceDeclaration {
        name: "Request",
        type_parameters: vec![],
        extends: vec![],
        properties: vec![
            PropertySignature::readonly("cache", Type::reference("RequestCache")),
            PropertySignature::readonly("credentials", Type::reference("RequestCredentials")),
            PropertySignature::readonly("destination", Type::reference("RequestDestination")),
            PropertySignature::readonly("headers", Type::reference("Headers")),
            PropertySignature::readonly("integrity", Type::String),
            PropertySignature::readonly("keepalive", Type::Boolean),
            PropertySignature::readonly("method", Type::String),
            PropertySignature::readonly("mode", Type::reference("RequestMode")),
            PropertySignature::readonly("redirect", Type::reference("RequestRedirect")),
            PropertySignature::readonly("referrer", Type::String),
            PropertySignature::readonly("signal", Type::reference("AbortSignal")),
            PropertySignature::readonly("url", Type::String),
        ],
        methods: vec![
            MethodSignature::new("clone", vec![], Type::reference("Request")),
            MethodSignature::new("arrayBuffer", vec![], Type::reference1("Promise", Type::reference("ArrayBuffer"))),
            MethodSignature::new("blob", vec![], Type::reference1("Promise", Type::reference("Blob"))),
            MethodSignature::new("formData", vec![], Type::reference1("Promise", Type::reference("FormData"))),
            MethodSignature::new("json", vec![], Type::reference1("Promise", Type::Any)),
            MethodSignature::new("text", vec![], Type::reference1("Promise", Type::String)),
        ],
        call_signatures: vec![],
        construct_signatures: vec![],
        index_signatures: vec![],
    }
}

fn response_interface() -> InterfaceDeclaration {
    InterfaceDeclaration {
        name: "Response",
        type_parameters: vec![],
        extends: vec![],
        properties: vec![
            PropertySignature::readonly("headers", Type::reference("Headers")),
            PropertySignature::readonly("ok", Type::Boolean),
            PropertySignature::readonly("redirected", Type::Boolean),
            PropertySignature::readonly("status", Type::Number),
            PropertySignature::readonly("statusText", Type::String),
            PropertySignature::readonly("type", Type::reference("ResponseType")),
            PropertySignature::readonly("url", Type::String),
        ],
        methods: vec![
            MethodSignature::new("clone", vec![], Type::reference("Response")),
            MethodSignature::new("arrayBuffer", vec![], Type::reference1("Promise", Type::reference("ArrayBuffer"))),
            MethodSignature::new("blob", vec![], Type::reference1("Promise", Type::reference("Blob"))),
            MethodSignature::new("formData", vec![], Type::reference1("Promise", Type::reference("FormData"))),
            MethodSignature::new("json", vec![], Type::reference1("Promise", Type::Any)),
            MethodSignature::new("text", vec![], Type::reference1("Promise", Type::String)),
        ],
        call_signatures: vec![],
        construct_signatures: vec![],
        index_signatures: vec![],
    }
}

fn headers_interface() -> InterfaceDeclaration {
    InterfaceDeclaration {
        name: "Headers",
        type_parameters: vec![],
        extends: vec![],
        properties: vec![],
        methods: vec![
            MethodSignature::new("append", vec![
                ParameterDeclaration::new("name", Type::String),
                ParameterDeclaration::new("value", Type::String),
            ], Type::Void),
            MethodSignature::new("delete", vec![ParameterDeclaration::new("name", Type::String)], Type::Void),
            MethodSignature::new("get", vec![ParameterDeclaration::new("name", Type::String)], Type::nullable(Type::String)),
            MethodSignature::new("has", vec![ParameterDeclaration::new("name", Type::String)], Type::Boolean),
            MethodSignature::new("set", vec![
                ParameterDeclaration::new("name", Type::String),
                ParameterDeclaration::new("value", Type::String),
            ], Type::Void),
            MethodSignature::new("forEach", vec![
                ParameterDeclaration::new("callbackfn", Type::func(
                    vec![
                        ParameterDeclaration::new("value", Type::String),
                        ParameterDeclaration::new("key", Type::String),
                        ParameterDeclaration::new("parent", Type::reference("Headers")),
                    ],
                    Type::Void,
                )),
                ParameterDeclaration::optional("thisArg", Type::Any),
            ], Type::Void),
            MethodSignature::new("entries", vec![], Type::reference1("IterableIterator", Type::Tuple(vec![Type::String, Type::String]))),
            MethodSignature::new("keys", vec![], Type::reference1("IterableIterator", Type::String)),
            MethodSignature::new("values", vec![], Type::reference1("IterableIterator", Type::String)),
        ],
        call_signatures: vec![],
        construct_signatures: vec![],
        index_signatures: vec![],
    }
}

fn url_interface() -> InterfaceDeclaration {
    InterfaceDeclaration {
        name: "URL",
        type_parameters: vec![],
        extends: vec![],
        properties: vec![
            PropertySignature::new("hash", Type::String),
            PropertySignature::new("host", Type::String),
            PropertySignature::new("hostname", Type::String),
            PropertySignature::new("href", Type::String),
            PropertySignature::readonly("origin", Type::String),
            PropertySignature::new("password", Type::String),
            PropertySignature::new("pathname", Type::String),
            PropertySignature::new("port", Type::String),
            PropertySignature::new("protocol", Type::String),
            PropertySignature::new("search", Type::String),
            PropertySignature::readonly("searchParams", Type::reference("URLSearchParams")),
            PropertySignature::new("username", Type::String),
        ],
        methods: vec![
            MethodSignature::new("toJSON", vec![], Type::String),
            MethodSignature::new("toString", vec![], Type::String),
        ],
        call_signatures: vec![],
        construct_signatures: vec![],
        index_signatures: vec![],
    }
}

fn url_search_params_interface() -> InterfaceDeclaration {
    InterfaceDeclaration {
        name: "URLSearchParams",
        type_parameters: vec![],
        extends: vec![],
        properties: vec![
            PropertySignature::readonly("size", Type::Number),
        ],
        methods: vec![
            MethodSignature::new("append", vec![
                ParameterDeclaration::new("name", Type::String),
                ParameterDeclaration::new("value", Type::String),
            ], Type::Void),
            MethodSignature::new("delete", vec![ParameterDeclaration::new("name", Type::String)], Type::Void),
            MethodSignature::new("get", vec![ParameterDeclaration::new("name", Type::String)], Type::nullable(Type::String)),
            MethodSignature::new("getAll", vec![ParameterDeclaration::new("name", Type::String)], Type::array(Type::String)),
            MethodSignature::new("has", vec![ParameterDeclaration::new("name", Type::String)], Type::Boolean),
            MethodSignature::new("set", vec![
                ParameterDeclaration::new("name", Type::String),
                ParameterDeclaration::new("value", Type::String),
            ], Type::Void),
            MethodSignature::new("sort", vec![], Type::Void),
            MethodSignature::new("forEach", vec![
                ParameterDeclaration::new("callbackfn", Type::func(
                    vec![
                        ParameterDeclaration::new("value", Type::String),
                        ParameterDeclaration::new("key", Type::String),
                        ParameterDeclaration::new("parent", Type::reference("URLSearchParams")),
                    ],
                    Type::Void,
                )),
                ParameterDeclaration::optional("thisArg", Type::Any),
            ], Type::Void),
            MethodSignature::new("entries", vec![], Type::reference1("IterableIterator", Type::Tuple(vec![Type::String, Type::String]))),
            MethodSignature::new("keys", vec![], Type::reference1("IterableIterator", Type::String)),
            MethodSignature::new("values", vec![], Type::reference1("IterableIterator", Type::String)),
            MethodSignature::new("toString", vec![], Type::String),
        ],
        call_signatures: vec![],
        construct_signatures: vec![],
        index_signatures: vec![],
    }
}

fn html_form_element_interface() -> InterfaceDeclaration {
    InterfaceDeclaration {
        name: "HTMLFormElement",
        type_parameters: vec![],
        extends: vec![Type::reference("HTMLElement")],
        properties: vec![
            PropertySignature::new("action", Type::String),
            PropertySignature::new("autocomplete", Type::String),
            PropertySignature::readonly("elements", Type::reference("HTMLFormControlsCollection")),
            PropertySignature::new("encoding", Type::String),
            PropertySignature::new("enctype", Type::String),
            PropertySignature::readonly("length", Type::Number),
            PropertySignature::new("method", Type::String),
            PropertySignature::new("name", Type::String),
            PropertySignature::new("noValidate", Type::Boolean),
            PropertySignature::new("target", Type::String),
        ],
        methods: vec![
            MethodSignature::new("checkValidity", vec![], Type::Boolean),
            MethodSignature::new("reportValidity", vec![], Type::Boolean),
            MethodSignature::new("reset", vec![], Type::Void),
            MethodSignature::new("submit", vec![], Type::Void),
        ],
        call_signatures: vec![],
        construct_signatures: vec![],
        index_signatures: vec![],
    }
}

fn html_input_element_interface() -> InterfaceDeclaration {
    InterfaceDeclaration {
        name: "HTMLInputElement",
        type_parameters: vec![],
        extends: vec![Type::reference("HTMLElement")],
        properties: vec![
            PropertySignature::new("accept", Type::String),
            PropertySignature::new("alt", Type::String),
            PropertySignature::new("autocomplete", Type::String),
            PropertySignature::new("checked", Type::Boolean),
            PropertySignature::new("defaultChecked", Type::Boolean),
            PropertySignature::new("defaultValue", Type::String),
            PropertySignature::new("disabled", Type::Boolean),
            PropertySignature::readonly("files", Type::nullable(Type::reference("FileList"))),
            PropertySignature::readonly("form", Type::nullable(Type::reference("HTMLFormElement"))),
            PropertySignature::new("max", Type::String),
            PropertySignature::new("maxLength", Type::Number),
            PropertySignature::new("min", Type::String),
            PropertySignature::new("minLength", Type::Number),
            PropertySignature::new("multiple", Type::Boolean),
            PropertySignature::new("name", Type::String),
            PropertySignature::new("pattern", Type::String),
            PropertySignature::new("placeholder", Type::String),
            PropertySignature::new("readOnly", Type::Boolean),
            PropertySignature::new("required", Type::Boolean),
            PropertySignature::new("selectionEnd", Type::nullable(Type::Number)),
            PropertySignature::new("selectionStart", Type::nullable(Type::Number)),
            PropertySignature::new("size", Type::Number),
            PropertySignature::new("src", Type::String),
            PropertySignature::new("step", Type::String),
            PropertySignature::new("type", Type::String),
            PropertySignature::readonly("validationMessage", Type::String),
            PropertySignature::readonly("validity", Type::reference("ValidityState")),
            PropertySignature::new("value", Type::String),
            PropertySignature::new("valueAsDate", Type::nullable(Type::reference("Date"))),
            PropertySignature::new("valueAsNumber", Type::Number),
            PropertySignature::readonly("willValidate", Type::Boolean),
        ],
        methods: vec![
            MethodSignature::new("checkValidity", vec![], Type::Boolean),
            MethodSignature::new("reportValidity", vec![], Type::Boolean),
            MethodSignature::new("select", vec![], Type::Void),
            MethodSignature::new("setCustomValidity", vec![ParameterDeclaration::new("error", Type::String)], Type::Void),
            MethodSignature::new("setSelectionRange", vec![
                ParameterDeclaration::new("start", Type::nullable(Type::Number)),
                ParameterDeclaration::new("end", Type::nullable(Type::Number)),
            ], Type::Void),
            MethodSignature::new("stepDown", vec![ParameterDeclaration::optional("n", Type::Number)], Type::Void),
            MethodSignature::new("stepUp", vec![ParameterDeclaration::optional("n", Type::Number)], Type::Void),
        ],
        call_signatures: vec![],
        construct_signatures: vec![],
        index_signatures: vec![],
    }
}

fn form_data_interface() -> InterfaceDeclaration {
    InterfaceDeclaration {
        name: "FormData",
        type_parameters: vec![],
        extends: vec![],
        properties: vec![],
        methods: vec![
            MethodSignature::new("append", vec![
                ParameterDeclaration::new("name", Type::String),
                ParameterDeclaration::new("value", Type::union(vec![Type::String, Type::reference("Blob")])),
            ], Type::Void),
            MethodSignature::new("delete", vec![ParameterDeclaration::new("name", Type::String)], Type::Void),
            MethodSignature::new("get", vec![ParameterDeclaration::new("name", Type::String)], Type::nullable(Type::reference("FormDataEntryValue"))),
            MethodSignature::new("getAll", vec![ParameterDeclaration::new("name", Type::String)], Type::array(Type::reference("FormDataEntryValue"))),
            MethodSignature::new("has", vec![ParameterDeclaration::new("name", Type::String)], Type::Boolean),
            MethodSignature::new("set", vec![
                ParameterDeclaration::new("name", Type::String),
                ParameterDeclaration::new("value", Type::union(vec![Type::String, Type::reference("Blob")])),
            ], Type::Void),
            MethodSignature::new("forEach", vec![
                ParameterDeclaration::new("callbackfn", Type::func(
                    vec![
                        ParameterDeclaration::new("value", Type::reference("FormDataEntryValue")),
                        ParameterDeclaration::new("key", Type::String),
                        ParameterDeclaration::new("parent", Type::reference("FormData")),
                    ],
                    Type::Void,
                )),
                ParameterDeclaration::optional("thisArg", Type::Any),
            ], Type::Void),
            MethodSignature::new("entries", vec![], Type::reference1("IterableIterator", Type::Tuple(vec![Type::String, Type::reference("FormDataEntryValue")]))),
            MethodSignature::new("keys", vec![], Type::reference1("IterableIterator", Type::String)),
            MethodSignature::new("values", vec![], Type::reference1("IterableIterator", Type::reference("FormDataEntryValue"))),
        ],
        call_signatures: vec![],
        construct_signatures: vec![],
        index_signatures: vec![],
    }
}
