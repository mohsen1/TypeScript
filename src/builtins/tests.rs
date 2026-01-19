//! Tests for the builtins module

use super::*;

#[test]
fn test_es5_array_interface() {
    let lib = es5::get_es5_declarations();
    let array = lib.find_interface("Array").expect("Array interface should exist");

    // Should have type parameter T
    assert_eq!(array.type_parameters.len(), 1);
    assert_eq!(array.type_parameters[0].name, "T");

    // Should have length property
    assert!(array.properties.iter().any(|p| p.name == "length"));

    // Should have common methods
    let method_names: Vec<&str> = array.methods.iter().map(|m| m.name).collect();
    assert!(method_names.contains(&"push"));
    assert!(method_names.contains(&"pop"));
    assert!(method_names.contains(&"map"));
    assert!(method_names.contains(&"filter"));
    assert!(method_names.contains(&"reduce"));
    assert!(method_names.contains(&"forEach"));
    assert!(method_names.contains(&"indexOf"));
    assert!(method_names.contains(&"slice"));
    assert!(method_names.contains(&"splice"));
}

#[test]
fn test_es5_object_interface() {
    let lib = es5::get_es5_declarations();
    let object = lib.find_interface("Object").expect("Object interface should exist");

    // Should have constructor property
    assert!(object.properties.iter().any(|p| p.name == "constructor"));

    // Should have common methods
    let method_names: Vec<&str> = object.methods.iter().map(|m| m.name).collect();
    assert!(method_names.contains(&"toString"));
    assert!(method_names.contains(&"valueOf"));
    assert!(method_names.contains(&"hasOwnProperty"));
}

#[test]
fn test_es5_function_interface() {
    let lib = es5::get_es5_declarations();
    let func = lib.find_interface("Function").expect("Function interface should exist");

    // Should have common properties
    let prop_names: Vec<&str> = func.properties.iter().map(|p| p.name).collect();
    assert!(prop_names.contains(&"prototype"));
    assert!(prop_names.contains(&"length"));
    assert!(prop_names.contains(&"name"));

    // Should have common methods
    let method_names: Vec<&str> = func.methods.iter().map(|m| m.name).collect();
    assert!(method_names.contains(&"apply"));
    assert!(method_names.contains(&"call"));
    assert!(method_names.contains(&"bind"));
}

#[test]
fn test_es5_string_interface() {
    let lib = es5::get_es5_declarations();
    let string = lib.find_interface("String").expect("String interface should exist");

    // Should have length property
    assert!(string.properties.iter().any(|p| p.name == "length"));

    // Should have common methods
    let method_names: Vec<&str> = string.methods.iter().map(|m| m.name).collect();
    assert!(method_names.contains(&"charAt"));
    assert!(method_names.contains(&"charCodeAt"));
    assert!(method_names.contains(&"concat"));
    assert!(method_names.contains(&"indexOf"));
    assert!(method_names.contains(&"slice"));
    assert!(method_names.contains(&"substring"));
    assert!(method_names.contains(&"toLowerCase"));
    assert!(method_names.contains(&"toUpperCase"));
    assert!(method_names.contains(&"trim"));
}

#[test]
fn test_es5_global_variables() {
    let lib = es5::get_es5_declarations();

    // Should have global variables
    assert!(lib.find_variable("NaN").is_some());
    assert!(lib.find_variable("Infinity").is_some());
    assert!(lib.find_variable("undefined").is_some());
    assert!(lib.find_variable("Object").is_some());
    assert!(lib.find_variable("Array").is_some());
    assert!(lib.find_variable("String").is_some());
    assert!(lib.find_variable("Number").is_some());
    assert!(lib.find_variable("Boolean").is_some());
    assert!(lib.find_variable("Math").is_some());
    assert!(lib.find_variable("JSON").is_some());
}

#[test]
fn test_es5_global_functions() {
    let lib = es5::get_es5_declarations();

    // Should have global functions
    let func_names: Vec<&str> = lib.functions.iter().map(|f| f.name).collect();
    assert!(func_names.contains(&"eval"));
    assert!(func_names.contains(&"parseInt"));
    assert!(func_names.contains(&"parseFloat"));
    assert!(func_names.contains(&"isNaN"));
    assert!(func_names.contains(&"isFinite"));
    assert!(func_names.contains(&"encodeURI"));
    assert!(func_names.contains(&"decodeURI"));
    assert!(func_names.contains(&"encodeURIComponent"));
    assert!(func_names.contains(&"decodeURIComponent"));
}

#[test]
fn test_es2015_promise_interface() {
    let lib = es2015::get_es2015_declarations();
    let promise = lib.find_interface("Promise").expect("Promise interface should exist");

    // Should have type parameter T
    assert_eq!(promise.type_parameters.len(), 1);
    assert_eq!(promise.type_parameters[0].name, "T");

    // Should have then, catch, finally methods
    let method_names: Vec<&str> = promise.methods.iter().map(|m| m.name).collect();
    assert!(method_names.contains(&"then"));
    assert!(method_names.contains(&"catch"));
    assert!(method_names.contains(&"finally"));
}

#[test]
fn test_es2015_symbol_interface() {
    let lib = es2015::get_es2015_declarations();
    let symbol = lib.find_interface("Symbol").expect("Symbol interface should exist");

    // Should have common methods
    let method_names: Vec<&str> = symbol.methods.iter().map(|m| m.name).collect();
    assert!(method_names.contains(&"toString"));
    assert!(method_names.contains(&"valueOf"));
}

#[test]
fn test_es2015_map_interface() {
    let lib = es2015::get_es2015_declarations();
    let map = lib.find_interface("Map").expect("Map interface should exist");

    // Should have type parameters K, V
    assert_eq!(map.type_parameters.len(), 2);
    assert_eq!(map.type_parameters[0].name, "K");
    assert_eq!(map.type_parameters[1].name, "V");

    // Should have size property
    assert!(map.properties.iter().any(|p| p.name == "size"));

    // Should have common methods
    let method_names: Vec<&str> = map.methods.iter().map(|m| m.name).collect();
    assert!(method_names.contains(&"get"));
    assert!(method_names.contains(&"set"));
    assert!(method_names.contains(&"has"));
    assert!(method_names.contains(&"delete"));
    assert!(method_names.contains(&"clear"));
    assert!(method_names.contains(&"forEach"));
    assert!(method_names.contains(&"entries"));
    assert!(method_names.contains(&"keys"));
    assert!(method_names.contains(&"values"));
}

#[test]
fn test_es2015_set_interface() {
    let lib = es2015::get_es2015_declarations();
    let set = lib.find_interface("Set").expect("Set interface should exist");

    // Should have type parameter T
    assert_eq!(set.type_parameters.len(), 1);
    assert_eq!(set.type_parameters[0].name, "T");

    // Should have common methods
    let method_names: Vec<&str> = set.methods.iter().map(|m| m.name).collect();
    assert!(method_names.contains(&"add"));
    assert!(method_names.contains(&"has"));
    assert!(method_names.contains(&"delete"));
    assert!(method_names.contains(&"clear"));
}

#[test]
fn test_es2020_bigint_interface() {
    let lib = es2020::get_es2020_declarations();
    let bigint = lib.find_interface("BigInt").expect("BigInt interface should exist");

    // Should have common methods
    let method_names: Vec<&str> = bigint.methods.iter().map(|m| m.name).collect();
    assert!(method_names.contains(&"toString"));
    assert!(method_names.contains(&"valueOf"));
}

#[test]
fn test_dom_document_interface() {
    let lib = dom::get_dom_declarations();
    let document = lib.find_interface("Document").expect("Document interface should exist");

    // Should extend Node
    assert!(document.extends.iter().any(|e| matches!(e, Type::TypeReference { name: "Node", .. })));

    // Should have common properties
    let prop_names: Vec<&str> = document.properties.iter().map(|p| p.name).collect();
    assert!(prop_names.contains(&"body"));
    assert!(prop_names.contains(&"documentElement"));
    assert!(prop_names.contains(&"title"));

    // Should have common methods
    let method_names: Vec<&str> = document.methods.iter().map(|m| m.name).collect();
    assert!(method_names.contains(&"getElementById"));
    assert!(method_names.contains(&"createElement"));
    assert!(method_names.contains(&"querySelector"));
    assert!(method_names.contains(&"querySelectorAll"));
}

#[test]
fn test_dom_element_interface() {
    let lib = dom::get_dom_declarations();
    let element = lib.find_interface("Element").expect("Element interface should exist");

    // Should extend Node
    assert!(element.extends.iter().any(|e| matches!(e, Type::TypeReference { name: "Node", .. })));

    // Should have common properties
    let prop_names: Vec<&str> = element.properties.iter().map(|p| p.name).collect();
    assert!(prop_names.contains(&"id"));
    assert!(prop_names.contains(&"className"));
    assert!(prop_names.contains(&"innerHTML"));
    assert!(prop_names.contains(&"tagName"));

    // Should have common methods
    let method_names: Vec<&str> = element.methods.iter().map(|m| m.name).collect();
    assert!(method_names.contains(&"getAttribute"));
    assert!(method_names.contains(&"setAttribute"));
    assert!(method_names.contains(&"querySelector"));
}

#[test]
fn test_dom_event_interface() {
    let lib = dom::get_dom_declarations();
    let event = lib.find_interface("Event").expect("Event interface should exist");

    // Should have common properties
    let prop_names: Vec<&str> = event.properties.iter().map(|p| p.name).collect();
    assert!(prop_names.contains(&"type"));
    assert!(prop_names.contains(&"target"));
    assert!(prop_names.contains(&"currentTarget"));
    assert!(prop_names.contains(&"bubbles"));
    assert!(prop_names.contains(&"cancelable"));

    // Should have common methods
    let method_names: Vec<&str> = event.methods.iter().map(|m| m.name).collect();
    assert!(method_names.contains(&"preventDefault"));
    assert!(method_names.contains(&"stopPropagation"));
}

#[test]
fn test_dom_window_interface() {
    let lib = dom::get_dom_declarations();
    let window = lib.find_interface("Window").expect("Window interface should exist");

    // Should have common properties
    let prop_names: Vec<&str> = window.properties.iter().map(|p| p.name).collect();
    assert!(prop_names.contains(&"document"));
    assert!(prop_names.contains(&"location"));
    assert!(prop_names.contains(&"history"));
    assert!(prop_names.contains(&"navigator"));

    // Should have common methods
    let method_names: Vec<&str> = window.methods.iter().map(|m| m.name).collect();
    assert!(method_names.contains(&"alert"));
    assert!(method_names.contains(&"confirm"));
    assert!(method_names.contains(&"open"));
}

#[test]
fn test_dom_console_interface() {
    let lib = dom::get_dom_declarations();
    let console = lib.find_interface("Console").expect("Console interface should exist");

    // Should have common methods
    let method_names: Vec<&str> = console.methods.iter().map(|m| m.name).collect();
    assert!(method_names.contains(&"log"));
    assert!(method_names.contains(&"error"));
    assert!(method_names.contains(&"warn"));
    assert!(method_names.contains(&"info"));
    assert!(method_names.contains(&"debug"));
    assert!(method_names.contains(&"trace"));
    assert!(method_names.contains(&"time"));
    assert!(method_names.contains(&"timeEnd"));
}

#[test]
fn test_dom_global_variables() {
    let lib = dom::get_dom_declarations();

    // Should have global DOM variables
    assert!(lib.find_variable("window").is_some());
    assert!(lib.find_variable("document").is_some());
    assert!(lib.find_variable("console").is_some());
    assert!(lib.find_variable("localStorage").is_some());
    assert!(lib.find_variable("sessionStorage").is_some());
}

#[test]
fn test_dom_global_functions() {
    let lib = dom::get_dom_declarations();

    // Should have global DOM functions
    let func_names: Vec<&str> = lib.functions.iter().map(|f| f.name).collect();
    assert!(func_names.contains(&"fetch"));
    assert!(func_names.contains(&"setTimeout"));
    assert!(func_names.contains(&"clearTimeout"));
    assert!(func_names.contains(&"setInterval"));
    assert!(func_names.contains(&"clearInterval"));
    assert!(func_names.contains(&"alert"));
    assert!(func_names.contains(&"confirm"));
    assert!(func_names.contains(&"prompt"));
}

#[test]
fn test_type_helpers() {
    // Test Type::reference
    let ref_type = Type::reference("Foo");
    assert!(matches!(ref_type, Type::TypeReference { name: "Foo", type_arguments } if type_arguments.is_empty()));

    // Test Type::reference1
    let ref_type = Type::reference1("Array", Type::String);
    assert!(matches!(ref_type, Type::TypeReference { name: "Array", type_arguments } if type_arguments.len() == 1));

    // Test Type::array
    let arr_type = Type::array(Type::Number);
    assert!(matches!(arr_type, Type::Array(_)));

    // Test Type::union
    let union_type = Type::union(vec![Type::String, Type::Number]);
    assert!(matches!(union_type, Type::Union(types) if types.len() == 2));

    // Test Type::nullable
    let nullable_type = Type::nullable(Type::String);
    assert!(matches!(nullable_type, Type::Union(types) if types.len() == 2));

    // Test Type::optional
    let optional_type = Type::optional(Type::String);
    assert!(matches!(optional_type, Type::Union(types) if types.len() == 2));
}

#[test]
fn test_lib_declarations_merge() {
    let lib1 = es5::get_es5_declarations();
    let lib2 = es2015::get_es2015_declarations();

    let mut merged = LibDeclarations::new();
    merged.merge(lib1);
    merged.merge(lib2);

    // Should have both ES5 and ES2015 types
    assert!(merged.find_interface("Array").is_some());
    assert!(merged.find_interface("Promise").is_some());
    assert!(merged.find_interface("Map").is_some());
}
