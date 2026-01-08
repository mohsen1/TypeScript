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
