use super::*;

#[test]
fn test_emit_extends_helper() {
    let mut helpers = HelpersNeeded::default();
    helpers.extends = true;
    let output = emit_helpers(&helpers);
    assert!(output.contains("__extends"));
    assert!(output.contains("extendStatics"));
}

#[test]
fn test_emit_awaiter_helper() {
    let mut helpers = HelpersNeeded::default();
    helpers.awaiter = true;
    let output = emit_helpers(&helpers);
    assert!(output.contains("__awaiter"));
    assert!(output.contains("adopt"));
}

#[test]
fn test_emit_multiple_helpers() {
    let mut helpers = HelpersNeeded::default();
    helpers.extends = true;
    helpers.assign = true;
    helpers.rest = true;
    let output = emit_helpers(&helpers);
    assert!(output.contains("__extends"));
    assert!(output.contains("__assign"));
    assert!(output.contains("__rest"));
}

#[test]
fn test_emit_decorate_helper() {
    let mut helpers = HelpersNeeded::default();
    helpers.decorate = true;
    let output = emit_helpers(&helpers);
    assert!(output.contains("__decorate"));
    assert!(output.contains("Reflect.decorate"));
}

#[test]
fn test_emit_param_helper() {
    let mut helpers = HelpersNeeded::default();
    helpers.param = true;
    let output = emit_helpers(&helpers);
    assert!(output.contains("__param"));
    assert!(output.contains("paramIndex"));
}

#[test]
fn test_emit_metadata_helper() {
    let mut helpers = HelpersNeeded::default();
    helpers.metadata = true;
    let output = emit_helpers(&helpers);
    assert!(output.contains("__metadata"));
    assert!(output.contains("Reflect.metadata"));
}

#[test]
fn test_emit_generator_helper() {
    let mut helpers = HelpersNeeded::default();
    helpers.generator = true;
    let output = emit_helpers(&helpers);
    assert!(output.contains("__generator"));
    assert!(output.contains("thisArg"));
    assert!(output.contains("body"));
}

#[test]
fn test_emit_values_helper() {
    let mut helpers = HelpersNeeded::default();
    helpers.values = true;
    let output = emit_helpers(&helpers);
    assert!(output.contains("__values"));
    assert!(output.contains("Symbol.iterator"));
}

#[test]
fn test_emit_read_helper() {
    let mut helpers = HelpersNeeded::default();
    helpers.read = true;
    let output = emit_helpers(&helpers);
    assert!(output.contains("__read"));
    assert!(output.contains("Symbol.iterator"));
}

#[test]
fn test_emit_spread_array_helper() {
    let mut helpers = HelpersNeeded::default();
    helpers.spread_array = true;
    let output = emit_helpers(&helpers);
    assert!(output.contains("__spreadArray"));
    assert!(output.contains("Array.prototype.slice"));
}

#[test]
fn test_emit_import_default_helper() {
    let mut helpers = HelpersNeeded::default();
    helpers.import_default = true;
    let output = emit_helpers(&helpers);
    assert!(output.contains("__importDefault"));
    assert!(output.contains("__esModule"));
}

#[test]
fn test_emit_import_star_helper() {
    let mut helpers = HelpersNeeded::default();
    helpers.import_star = true;
    let output = emit_helpers(&helpers);
    assert!(output.contains("__importStar"));
    assert!(output.contains("__setModuleDefault"));
    assert!(output.contains("__esModule"));
}

#[test]
fn test_emit_export_star_helper() {
    let mut helpers = HelpersNeeded::default();
    helpers.export_star = true;
    let output = emit_helpers(&helpers);
    assert!(output.contains("__exportStar"));
    assert!(output.contains("__createBinding"));
}

#[test]
fn test_emit_make_template_object_helper() {
    let mut helpers = HelpersNeeded::default();
    helpers.make_template_object = true;
    let output = emit_helpers(&helpers);
    assert!(output.contains("__makeTemplateObject"));
    assert!(output.contains("cooked"));
    assert!(output.contains("raw"));
}

#[test]
fn test_emit_class_private_field_get_helper() {
    let mut helpers = HelpersNeeded::default();
    helpers.class_private_field_get = true;
    let output = emit_helpers(&helpers);
    assert!(output.contains("__classPrivateFieldGet"));
    assert!(output.contains("Private accessor"));
}

#[test]
fn test_emit_class_private_field_set_helper() {
    let mut helpers = HelpersNeeded::default();
    helpers.class_private_field_set = true;
    let output = emit_helpers(&helpers);
    assert!(output.contains("__classPrivateFieldSet"));
    assert!(output.contains("Private method is not writable"));
}

#[test]
fn test_emit_class_private_field_in_helper() {
    let mut helpers = HelpersNeeded::default();
    helpers.class_private_field_in = true;
    let output = emit_helpers(&helpers);
    assert!(output.contains("__classPrivateFieldIn"));
    assert!(output.contains("Cannot use 'in' operator"));
}

#[test]
fn test_emit_create_binding_helper() {
    let mut helpers = HelpersNeeded::default();
    helpers.create_binding = true;
    let output = emit_helpers(&helpers);
    assert!(output.contains("__createBinding"));
    assert!(output.contains("Object.getOwnPropertyDescriptor"));
}

#[test]
fn test_helper_ordering_create_binding_before_import_star() {
    // __importStar depends on __createBinding and __setModuleDefault
    let mut helpers = HelpersNeeded::default();
    helpers.create_binding = true;
    helpers.import_star = true;
    let output = emit_helpers(&helpers);
    let create_binding_pos = output.find("__createBinding").unwrap();
    let set_module_default_pos = output.find("__setModuleDefault").unwrap();
    let import_star_pos = output.find("__importStar").unwrap();
    // __createBinding should come before __setModuleDefault and __importStar
    assert!(create_binding_pos < set_module_default_pos);
    assert!(set_module_default_pos < import_star_pos);
}

#[test]
fn test_all_helpers_combined() {
    let mut helpers = HelpersNeeded::default();
    helpers.extends = true;
    helpers.assign = true;
    helpers.rest = true;
    helpers.decorate = true;
    helpers.param = true;
    helpers.metadata = true;
    helpers.awaiter = true;
    helpers.generator = true;
    helpers.values = true;
    helpers.read = true;
    helpers.spread_array = true;
    helpers.import_default = true;
    helpers.export_star = true;
    helpers.make_template_object = true;
    helpers.class_private_field_get = true;
    helpers.class_private_field_set = true;
    helpers.class_private_field_in = true;
    helpers.create_binding = true;
    let output = emit_helpers(&helpers);
    // Verify all helpers are present
    assert!(output.contains("__extends"));
    assert!(output.contains("__assign"));
    assert!(output.contains("__rest"));
    assert!(output.contains("__decorate"));
    assert!(output.contains("__param"));
    assert!(output.contains("__metadata"));
    assert!(output.contains("__awaiter"));
    assert!(output.contains("__generator"));
    assert!(output.contains("__values"));
    assert!(output.contains("__read"));
    assert!(output.contains("__spreadArray"));
    assert!(output.contains("__importDefault"));
    assert!(output.contains("__exportStar"));
    assert!(output.contains("__makeTemplateObject"));
    assert!(output.contains("__classPrivateFieldGet"));
    assert!(output.contains("__classPrivateFieldSet"));
    assert!(output.contains("__classPrivateFieldIn"));
    assert!(output.contains("__createBinding"));
}
