//! Project-level LSP tests.

use super::*;
use crate::lsp::position::LineMap;

fn apply_text_edits(source: &str, line_map: &LineMap, edits: &[TextEdit]) -> String {
    let mut result = source.to_string();
    let mut edits_with_offsets: Vec<(usize, usize, &TextEdit)> = edits
        .iter()
        .map(|edit| {
            let start = line_map
                .position_to_offset(edit.range.start, source)
                .unwrap_or(0) as usize;
            let end = line_map
                .position_to_offset(edit.range.end, source)
                .unwrap_or(0) as usize;
            (start, end, edit)
        })
        .collect();

    edits_with_offsets.sort_by(|a, b| b.0.cmp(&a.0));
    for (start, end, edit) in edits_with_offsets {
        result.replace_range(start..end, &edit.new_text);
    }
    result
}

fn range_for_substring(source: &str, line_map: &LineMap, needle: &str) -> Range {
    let start = source.find(needle).expect("substring not found") as u32;
    let end = start + needle.len() as u32;
    let start_pos = line_map.offset_to_position(start, source);
    let end_pos = line_map.offset_to_position(end, source);
    Range::new(start_pos, end_pos)
}

#[test]
fn test_project_cross_file_references_named_import() {
    let mut project = Project::new();

    project.set_file("a.ts".to_string(), "export const foo = 1;\nfoo;\n".to_string());
    project.set_file("b.ts".to_string(), "import { foo } from \"./a\";\nfoo;\n".to_string());

    let refs = project.find_references("b.ts", Position::new(1, 0));
    assert!(refs.is_some(), "Should find references for imported foo");

    let refs = refs.unwrap();
    assert!(refs.iter().any(|loc| loc.file_path == "a.ts"), "Should include references from a.ts");
    assert!(refs.iter().any(|loc| loc.file_path == "b.ts"), "Should include references from b.ts");
}

#[test]
fn test_project_cross_file_references_default_import() {
    let mut project = Project::new();

    project.set_file("a.ts".to_string(), "export default function foo() {}\nfoo();".to_string());
    project.set_file("b.ts".to_string(), "import foo from \"./a\";\nfoo();".to_string());

    let refs = project.find_references("b.ts", Position::new(1, 0));
    assert!(refs.is_some(), "Should find references for default import");

    let refs = refs.unwrap();
    assert!(refs.iter().any(|loc| loc.file_path == "a.ts"), "Should include references from a.ts");
    assert!(refs.iter().any(|loc| loc.file_path == "b.ts"), "Should include references from b.ts");
}

#[test]
fn test_project_cross_file_references_namespace_import() {
    let mut project = Project::new();

    project.set_file("a.ts".to_string(), "export const foo = 1;\n".to_string());
    project.set_file("b.ts".to_string(), "import * as ns from \"./a\";\nns.foo;\n".to_string());

    let refs = project.find_references("a.ts", Position::new(0, 13));
    assert!(refs.is_some(), "Should find references for namespace import");

    let refs = refs.unwrap();
    assert!(refs.iter().any(|loc| loc.file_path == "b.ts"), "Should include references from b.ts");
}

#[test]
fn test_project_cross_file_references_tsx_import() {
    let mut project = Project::new();

    project.set_file("a.tsx".to_string(), "export const foo = 1;\n".to_string());
    project.set_file("b.ts".to_string(), "import { foo } from \"./a\";\nfoo;\n".to_string());

    let refs = project.find_references("b.ts", Position::new(1, 0));
    assert!(refs.is_some(), "Should find references for tsx import");

    let refs = refs.unwrap();
    assert!(refs.iter().any(|loc| loc.file_path == "a.tsx"), "Should include references from a.tsx");
    assert!(refs.iter().any(|loc| loc.file_path == "b.ts"), "Should include references from b.ts");
}

#[test]
fn test_project_update_file_applies_edits() {
    let mut project = Project::new();
    project.set_file("a.ts".to_string(), "const value = 1;\n".to_string());

    let file = project.file("a.ts").unwrap();
    let range = range_for_substring(file.source_text(), file.line_map(), "1");
    let edit = TextEdit::new(range, "2".to_string());

    project.update_file("a.ts", &[edit]).expect("Expected update to succeed");

    let updated = project.file("a.ts").unwrap().source_text();
    assert_eq!(updated, "const value = 2;\n");
}

#[test]
fn test_project_update_file_refreshes_cross_file_references() {
    let mut project = Project::new();

    project.set_file("a.ts".to_string(), "export const foo = 1;\n".to_string());
    project.set_file("b.ts".to_string(), "import { foo } from \"./a\";\nfoo;\n".to_string());

    let before_refs = project
        .find_references("b.ts", Position::new(1, 0))
        .expect("Expected references for foo");
    assert!(before_refs.iter().any(|loc| loc.file_path == "a.ts"));

    let rename_edit = {
        let file = project.file("a.ts").unwrap();
        let range = range_for_substring(file.source_text(), file.line_map(), "foo");
        TextEdit::new(range, "bar".to_string())
    };
    project
        .update_file("a.ts", &[rename_edit])
        .expect("Expected update to succeed");

    let after_refs = project
        .find_references("b.ts", Position::new(1, 0))
        .expect("Expected references for foo");
    assert!(after_refs.iter().all(|loc| loc.file_path != "a.ts"));
}

#[test]
fn test_project_hover_includes_jsdoc() {
    let mut project = Project::new();
    let source = "/** The answer */\nconst x = 42;\nx;";
    project.set_file("a.ts".to_string(), source.to_string());

    let info = project
        .get_hover("a.ts", Position::new(2, 0))
        .expect("Expected hover info");

    assert!(info.contents.iter().any(|content| content.contains("The answer")));
}

#[test]
fn test_project_signature_help_includes_jsdoc() {
    let mut project = Project::new();
    let source = "/** Adds two numbers. */\nfunction add(a: number, b: number): number { return a + b; }\nadd(1, 2);";
    project.set_file("a.ts".to_string(), source.to_string());

    let pos = {
        let file = project.file("a.ts").unwrap();
        range_for_substring(file.source_text(), file.line_map(), "1").start
    };

    let help = project
        .get_signature_help("a.ts", pos)
        .expect("Expected signature help");

    let doc = help.signatures[help.active_signature as usize]
        .documentation
        .clone()
        .unwrap_or_default();
    assert_eq!(doc, "Adds two numbers.");
}

#[test]
fn test_project_completions_auto_import_named() {
    let mut project = Project::new();

    project.set_file("a.ts".to_string(), "export const foo = 1;\n".to_string());
    project.set_file("b.ts".to_string(), "foo;\n".to_string());

    let items = project
        .get_completions("b.ts", Position::new(0, 1))
        .expect("Expected completions");

    let has_auto_import = items.iter().any(|item| {
        if item.label != "foo" {
            return false;
        }
        let detail = item.detail.as_deref().unwrap_or("");
        let doc = item.documentation.as_deref().unwrap_or("");
        detail.contains("auto-import") && detail.contains("./a") && doc.contains("import { foo } from \"./a\";")
    });

    assert!(has_auto_import, "Should include auto-import completion for foo");
}

#[test]
fn test_project_cross_file_references_reexport_named() {
    let mut project = Project::new();

    project.set_file("a.ts".to_string(), "export const foo = 1;\n".to_string());
    project.set_file("b.ts".to_string(), "export { foo as bar } from \"./a\";\n".to_string());
    project.set_file("c.ts".to_string(), "import { bar } from \"./b\";\nbar;\n".to_string());

    let refs = project.find_references("a.ts", Position::new(0, 13));
    assert!(refs.is_some(), "Should find references across re-exports");

    let refs = refs.unwrap();
    assert!(refs.iter().any(|loc| loc.file_path == "b.ts"), "Should include re-export reference in b.ts");
    assert!(refs.iter().any(|loc| loc.file_path == "c.ts"), "Should include references from c.ts");
}

#[test]
fn test_project_cross_file_references_namespace_reexport() {
    let mut project = Project::new();

    project.set_file("a.ts".to_string(), "export const foo = 1;\n".to_string());
    project.set_file("b.ts".to_string(), "export * as ns from \"./a\";\n".to_string());
    project.set_file("c.ts".to_string(), "import { ns } from \"./b\";\nns.foo;\n".to_string());

    let refs = project.find_references("a.ts", Position::new(0, 13));
    assert!(refs.is_some(), "Should find references through namespace re-export");

    let refs = refs.unwrap();
    assert!(refs.iter().any(|loc| loc.file_path == "c.ts"), "Should include namespace member reference in c.ts");
}

#[test]
fn test_project_code_actions_missing_import_named() {
    let mut project = Project::new();

    project.set_file("a.ts".to_string(), "export const foo = 1;\n".to_string());
    project.set_file("b.ts".to_string(), "foo();\n".to_string());

    let file = project.file("b.ts").unwrap();
    let source = file.source_text();
    let line_map = file.line_map();
    let start = source.find("foo").unwrap();
    let range = Range::new(
        line_map.offset_to_position(start as u32, source),
        line_map.offset_to_position((start + 3) as u32, source),
    );

    let diag = LspDiagnostic {
        range,
        severity: Some(DiagnosticSeverity::Error),
        code: Some(crate::checker::types::diagnostics::diagnostic_codes::CANNOT_FIND_NAME),
        source: None,
        message: "Cannot find name 'foo'.".to_string(),
        related_information: None,
    };

    let actions = project
        .get_code_actions(
            "b.ts",
            Range::new(Position::new(0, 0), Position::new(0, 0)),
            vec![diag],
            Some(vec![CodeActionKind::QuickFix]),
        )
        .expect("Expected missing import quick fix");

    let edit = actions[0].edit.as_ref().unwrap();
    let edits = edit.changes.get("b.ts").unwrap();
    let updated = apply_text_edits(source, line_map, edits);
    assert_eq!(updated, "import { foo } from \"./a\";\nfoo();\n");
}

#[test]
fn test_project_code_actions_missing_import_default_export() {
    let mut project = Project::new();

    project.set_file("a.ts".to_string(), "export default function bar() {}\n".to_string());
    project.set_file("b.ts".to_string(), "foo();\n".to_string());

    let file = project.file("b.ts").unwrap();
    let source = file.source_text();
    let line_map = file.line_map();
    let start = source.find("foo").unwrap();
    let range = Range::new(
        line_map.offset_to_position(start as u32, source),
        line_map.offset_to_position((start + 3) as u32, source),
    );

    let diag = LspDiagnostic {
        range,
        severity: Some(DiagnosticSeverity::Error),
        code: Some(crate::checker::types::diagnostics::diagnostic_codes::CANNOT_FIND_NAME),
        source: None,
        message: "Cannot find name 'foo'.".to_string(),
        related_information: None,
    };

    let actions = project
        .get_code_actions(
            "b.ts",
            Range::new(Position::new(0, 0), Position::new(0, 0)),
            vec![diag],
            Some(vec![CodeActionKind::QuickFix]),
        )
        .expect("Expected missing import quick fix");

    let edit = actions[0].edit.as_ref().unwrap();
    let edits = edit.changes.get("b.ts").unwrap();
    let updated = apply_text_edits(source, line_map, edits);
    assert_eq!(updated, "import foo from \"./a\";\nfoo();\n");
}

#[test]
fn test_project_code_actions_missing_import_tsx() {
    let mut project = Project::new();

    project.set_file("a.tsx".to_string(), "export const foo = 1;\n".to_string());
    project.set_file("b.ts".to_string(), "foo();\n".to_string());

    let file = project.file("b.ts").unwrap();
    let source = file.source_text();
    let line_map = file.line_map();
    let start = source.find("foo").unwrap();
    let range = Range::new(
        line_map.offset_to_position(start as u32, source),
        line_map.offset_to_position((start + 3) as u32, source),
    );

    let diag = LspDiagnostic {
        range,
        severity: Some(DiagnosticSeverity::Error),
        code: Some(crate::checker::types::diagnostics::diagnostic_codes::CANNOT_FIND_NAME),
        source: None,
        message: "Cannot find name 'foo'.".to_string(),
        related_information: None,
    };

    let actions = project
        .get_code_actions(
            "b.ts",
            Range::new(Position::new(0, 0), Position::new(0, 0)),
            vec![diag],
            Some(vec![CodeActionKind::QuickFix]),
        )
        .expect("Expected missing import quick fix");

    let edit = actions[0].edit.as_ref().unwrap();
    let edits = edit.changes.get("b.ts").unwrap();
    let updated = apply_text_edits(source, line_map, edits);
    assert_eq!(updated, "import { foo } from \"./a\";\nfoo();\n");
}

#[test]
fn test_project_code_actions_missing_import_default_reexport() {
    let mut project = Project::new();

    project.set_file("a.ts".to_string(), "export default function bar() {}\n".to_string());
    project.set_file("index.ts".to_string(), "export { default } from \"./a\";\n".to_string());
    project.set_file("b.ts".to_string(), "foo();\n".to_string());

    let file = project.file("b.ts").unwrap();
    let source = file.source_text();
    let line_map = file.line_map();
    let start = source.find("foo").unwrap();
    let range = Range::new(
        line_map.offset_to_position(start as u32, source),
        line_map.offset_to_position((start + 3) as u32, source),
    );

    let diag = LspDiagnostic {
        range,
        severity: Some(DiagnosticSeverity::Error),
        code: Some(crate::checker::types::diagnostics::diagnostic_codes::CANNOT_FIND_NAME),
        source: None,
        message: "Cannot find name 'foo'.".to_string(),
        related_information: None,
    };

    let actions = project
        .get_code_actions(
            "b.ts",
            Range::new(Position::new(0, 0), Position::new(0, 0)),
            vec![diag],
            Some(vec![CodeActionKind::QuickFix]),
        )
        .expect("Expected missing import quick fix");

    let edit = actions[0].edit.as_ref().unwrap();
    let edits = edit.changes.get("b.ts").unwrap();
    let updated = apply_text_edits(source, line_map, edits);
    assert_eq!(updated, "import foo from \"./index\";\nfoo();\n");
}

#[test]
fn test_project_code_actions_missing_import_reexport() {
    let mut project = Project::new();

    project.set_file("a.ts".to_string(), "export const foo = 1;\n".to_string());
    project.set_file("index.ts".to_string(), "export { foo as bar } from \"./a\";\n".to_string());
    project.set_file("b.ts".to_string(), "bar();\n".to_string());

    let file = project.file("b.ts").unwrap();
    let source = file.source_text();
    let line_map = file.line_map();
    let start = source.find("bar").unwrap();
    let range = Range::new(
        line_map.offset_to_position(start as u32, source),
        line_map.offset_to_position((start + 3) as u32, source),
    );

    let diag = LspDiagnostic {
        range,
        severity: Some(DiagnosticSeverity::Error),
        code: Some(crate::checker::types::diagnostics::diagnostic_codes::CANNOT_FIND_NAME),
        source: None,
        message: "Cannot find name 'bar'.".to_string(),
        related_information: None,
    };

    let actions = project
        .get_code_actions(
            "b.ts",
            Range::new(Position::new(0, 0), Position::new(0, 0)),
            vec![diag],
            Some(vec![CodeActionKind::QuickFix]),
        )
        .expect("Expected missing import quick fix");

    let edit = actions[0].edit.as_ref().unwrap();
    let edits = edit.changes.get("b.ts").unwrap();
    let updated = apply_text_edits(source, line_map, edits);
    assert_eq!(updated, "import { bar } from \"./index\";\nbar();\n");
}
