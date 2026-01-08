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
