use serde_json::{json, Value};
use wasm_bindgen::JsValue;

use crate::checker::types::diagnostics::diagnostic_codes::CANNOT_FIND_NAME;
use crate::lsp::{DiagnosticSeverity, LspDiagnostic, Position, Range};

use super::ThinParser;

#[test]
fn test_get_code_actions_with_context_missing_import() {
    let mut parser = ThinParser::new("b.ts".to_string(), "foo();\n".to_string());
    parser.parse_source_file();

    let diag = LspDiagnostic {
        range: Range::new(Position::new(0, 0), Position::new(0, 3)),
        severity: Some(DiagnosticSeverity::Error),
        code: Some(CANNOT_FIND_NAME),
        source: None,
        message: "Cannot find name 'foo'.".to_string(),
        related_information: None,
    };

    let diagnostics = serde_wasm_bindgen::to_value(&vec![diag]).unwrap();
    let only = JsValue::UNDEFINED;

    let import_candidates = vec![json!({
        "kind": "named",
        "moduleSpecifier": "./a",
        "localName": "foo",
        "exportName": "foo"
    })];
    let import_candidates = serde_wasm_bindgen::to_value(&import_candidates).unwrap();

    let actions_value = parser
        .get_code_actions_with_context(0, 0, 0, 0, diagnostics, only, import_candidates)
        .unwrap();
    let actions: Vec<Value> = serde_wasm_bindgen::from_value(actions_value).unwrap();

    let action = actions
        .iter()
        .find(|action| action.get("title").and_then(Value::as_str)
            == Some("Import 'foo' from './a'"))
        .expect("Expected missing import code action");

    let edit = action.get("edit").expect("Expected workspace edit");
    let changes = edit.get("changes").expect("Expected changes");
    let edits = changes
        .get("b.ts")
        .and_then(Value::as_array)
        .expect("Expected edits for b.ts");

    let new_text = edits[0]
        .get("newText")
        .and_then(Value::as_str)
        .expect("Expected newText");

    assert_eq!(new_text, "import { foo } from \"./a\";\n");
}
