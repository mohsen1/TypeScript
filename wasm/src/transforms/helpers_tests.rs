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
fn test_emit_helpers_empty() {
    let helpers = HelpersNeeded::default();
    let output = emit_helpers(&helpers);
    assert!(output.is_empty(), "Expected no helpers when none requested");
}

#[test]
fn test_emit_import_star_helpers() {
    let mut helpers = HelpersNeeded::default();
    helpers.import_star = true;
    let output = emit_helpers(&helpers);
    assert!(output.contains("__setModuleDefault"));
    assert!(output.contains("__importStar"));
}

#[test]
fn test_emit_create_binding_helper() {
    let mut helpers = HelpersNeeded::default();
    helpers.create_binding = true;
    let output = emit_helpers(&helpers);
    assert!(output.contains("__createBinding"));
}

#[test]
fn test_emit_export_star_orders_create_binding() {
    let mut helpers = HelpersNeeded::default();
    helpers.create_binding = true;
    helpers.export_star = true;
    let output = emit_helpers(&helpers);
    let create_binding_pos = output.find("__createBinding").expect("Expected __createBinding helper");
    let export_star_pos = output.find("__exportStar").expect("Expected __exportStar helper");
    assert!(create_binding_pos < export_star_pos, "__createBinding should precede __exportStar");
}

#[test]
fn test_emit_import_star_orders_set_module_default() {
    let mut helpers = HelpersNeeded::default();
    helpers.import_star = true;
    let output = emit_helpers(&helpers);
    let set_module_default_pos = output.find("__setModuleDefault").expect("Expected __setModuleDefault helper");
    let import_star_pos = output.find("__importStar").expect("Expected __importStar helper");
    assert!(set_module_default_pos < import_star_pos, "__setModuleDefault should precede __importStar");
}

#[test]
fn test_emit_create_binding_before_import_star_helpers() {
    let mut helpers = HelpersNeeded::default();
    helpers.create_binding = true;
    helpers.import_star = true;
    let output = emit_helpers(&helpers);
    let create_binding_pos = output.find("__createBinding").expect("Expected __createBinding helper");
    let set_module_default_pos = output.find("__setModuleDefault").expect("Expected __setModuleDefault helper");
    let import_star_pos = output.find("__importStar").expect("Expected __importStar helper");
    assert!(create_binding_pos < set_module_default_pos, "__createBinding should precede __setModuleDefault");
    assert!(create_binding_pos < import_star_pos, "__createBinding should precede __importStar");
}

#[test]
fn test_emit_class_private_helpers_ordering() {
    let mut helpers = HelpersNeeded::default();
    helpers.class_private_field_get = true;
    helpers.class_private_field_set = true;
    helpers.class_private_field_in = true;
    let output = emit_helpers(&helpers);
    let get_pos = output.find("__classPrivateFieldGet").expect("Expected __classPrivateFieldGet helper");
    let set_pos = output.find("__classPrivateFieldSet").expect("Expected __classPrivateFieldSet helper");
    let in_pos = output.find("__classPrivateFieldIn").expect("Expected __classPrivateFieldIn helper");
    assert!(get_pos < set_pos, "__classPrivateFieldGet should precede __classPrivateFieldSet");
    assert!(set_pos < in_pos, "__classPrivateFieldSet should precede __classPrivateFieldIn");
}

#[test]
fn test_emit_values_before_read_helpers() {
    let mut helpers = HelpersNeeded::default();
    helpers.values = true;
    helpers.read = true;
    let output = emit_helpers(&helpers);
    let values_pos = output.find("__values").expect("Expected __values helper");
    let read_pos = output.find("__read").expect("Expected __read helper");
    assert!(values_pos < read_pos, "__values should precede __read");
}

#[test]
fn test_emit_awaiter_before_generator_helpers() {
    let mut helpers = HelpersNeeded::default();
    helpers.awaiter = true;
    helpers.generator = true;
    let output = emit_helpers(&helpers);
    let awaiter_pos = output.find("__awaiter").expect("Expected __awaiter helper");
    let generator_pos = output.find("__generator").expect("Expected __generator helper");
    assert!(awaiter_pos < generator_pos, "__awaiter should precede __generator");
}
