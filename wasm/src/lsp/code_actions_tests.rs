use super::*;
use crate::checker::types::diagnostics::diagnostic_codes::UNUSED_IMPORT;
use crate::lsp::position::LineMap;
use crate::thin_binder::ThinBinderState;
use crate::thin_parser::ThinParserState;

fn range_for_substring(source: &str, line_map: &LineMap, needle: &str) -> Range {
    let start = source.find(needle).expect("substring not found") as u32;
    let end = start + needle.len() as u32;
    let start_pos = line_map.offset_to_position(start, source);
    let end_pos = line_map.offset_to_position(end, source);
    Range::new(start_pos, end_pos)
}

#[test]
fn test_extract_variable_property_access() {
    let source = "const x = foo.bar.baz + 1;";
    let mut parser = ThinParserState::new("test.ts".to_string(), source.to_string());
    let root = parser.parse_source_file();
    let arena = parser.get_arena();

    let mut binder = ThinBinderState::new();
    binder.bind_source_file(arena, root);

    let line_map = LineMap::build(source);
    let provider = CodeActionProvider::new(
        arena,
        &binder,
        &line_map,
        "test.ts".to_string(),
        source,
    );

    let range = Range {
        start: Position::new(0, 10),
        end: Position::new(0, 21),
    };

    let actions = provider.provide_code_actions(
        root,
        range,
        CodeActionContext {
            diagnostics: Vec::new(),
            only: None,
        },
    );

    assert_eq!(actions.len(), 1);
    assert_eq!(actions[0].title, "Extract to constant 'extracted'");
    assert_eq!(actions[0].kind, CodeActionKind::RefactorExtract);
    assert!(actions[0].is_preferred);

    let edit = actions[0].edit.as_ref().unwrap();
    let edits = edit.changes.get("test.ts").unwrap();
    assert_eq!(edits.len(), 2);

    assert!(edits[0].new_text.contains("const extracted = foo.bar.baz;"));
    assert_eq!(edits[1].new_text, "extracted");
}

#[test]
fn test_extract_variable_no_action_for_simple_literal() {
    let source = "const x = 42;";
    let mut parser = ThinParserState::new("test.ts".to_string(), source.to_string());
    let root = parser.parse_source_file();
    let arena = parser.get_arena();

    let mut binder = ThinBinderState::new();
    binder.bind_source_file(arena, root);

    let line_map = LineMap::build(source);
    let provider = CodeActionProvider::new(
        arena,
        &binder,
        &line_map,
        "test.ts".to_string(),
        source,
    );

    let range = Range {
        start: Position::new(0, 10),
        end: Position::new(0, 12),
    };

    let actions = provider.provide_code_actions(
        root,
        range,
        CodeActionContext {
            diagnostics: Vec::new(),
            only: None,
        },
    );

    assert_eq!(actions.len(), 0);
}

#[test]
fn test_extract_variable_empty_range() {
    let source = "const x = foo.bar.baz;";
    let mut parser = ThinParserState::new("test.ts".to_string(), source.to_string());
    let root = parser.parse_source_file();
    let arena = parser.get_arena();

    let mut binder = ThinBinderState::new();
    binder.bind_source_file(arena, root);

    let line_map = LineMap::build(source);
    let provider = CodeActionProvider::new(
        arena,
        &binder,
        &line_map,
        "test.ts".to_string(),
        source,
    );

    let range = Range {
        start: Position::new(0, 10),
        end: Position::new(0, 10),
    };

    let actions = provider.provide_code_actions(
        root,
        range,
        CodeActionContext {
            diagnostics: Vec::new(),
            only: None,
        },
    );

    assert_eq!(actions.len(), 0);
}

#[test]
fn test_organize_imports_sort_only() {
    let source = "import { b } from \"b\";\nimport { a } from \"a\";\nconst x = 1;\n";
    let mut parser = ThinParserState::new("test.ts".to_string(), source.to_string());
    let root = parser.parse_source_file();
    let arena = parser.get_arena();

    let mut binder = ThinBinderState::new();
    binder.bind_source_file(arena, root);

    let line_map = LineMap::build(source);
    let provider = CodeActionProvider::new(
        arena,
        &binder,
        &line_map,
        "test.ts".to_string(),
        source,
    );

    let range = Range {
        start: Position::new(0, 0),
        end: Position::new(0, 0),
    };

    let actions = provider.provide_code_actions(
        root,
        range,
        CodeActionContext {
            diagnostics: Vec::new(),
            only: Some(vec![CodeActionKind::SourceOrganizeImports]),
        },
    );

    assert_eq!(actions.len(), 1);
    let edit = actions[0].edit.as_ref().unwrap();
    let edits = edit.changes.get("test.ts").unwrap();
    assert_eq!(edits.len(), 1);

    let new_text = &edits[0].new_text;
    let pos_a = new_text.find("import { a } from \"a\";").unwrap();
    let pos_b = new_text.find("import { b } from \"b\";").unwrap();
    assert!(pos_a < pos_b, "Imports should be sorted by module specifier");
}

#[test]
fn test_quickfix_remove_unused_named_import() {
    let source = "import { foo, bar } from \"mod\";\nbar;\n";
    let mut parser = ThinParserState::new("test.ts".to_string(), source.to_string());
    let root = parser.parse_source_file();
    let arena = parser.get_arena();

    let mut binder = ThinBinderState::new();
    binder.bind_source_file(arena, root);

    let line_map = LineMap::build(source);
    let range = range_for_substring(source, &line_map, "foo");
    let diag = LspDiagnostic {
        range,
        severity: Some(DiagnosticSeverity::Warning),
        code: Some(UNUSED_IMPORT),
        source: None,
        message: "unused import".to_string(),
        related_information: None,
    };

    let provider = CodeActionProvider::new(
        arena,
        &binder,
        &line_map,
        "test.ts".to_string(),
        source,
    );

    let empty_range = Range::new(Position::new(0, 0), Position::new(0, 0));
    let actions = provider.provide_code_actions(
        root,
        empty_range,
        CodeActionContext {
            diagnostics: vec![diag],
            only: Some(vec![CodeActionKind::QuickFix]),
        },
    );

    assert_eq!(actions.len(), 1);
    let edit = actions[0].edit.as_ref().unwrap();
    let edits = edit.changes.get("test.ts").unwrap();
    assert_eq!(edits.len(), 1);
    assert_eq!(edits[0].new_text, "import { bar } from \"mod\";\n");
}

#[test]
fn test_quickfix_remove_unused_named_import_entire_decl() {
    let source = "import { foo } from \"mod\";\nconst x = 1;\n";
    let mut parser = ThinParserState::new("test.ts".to_string(), source.to_string());
    let root = parser.parse_source_file();
    let arena = parser.get_arena();

    let mut binder = ThinBinderState::new();
    binder.bind_source_file(arena, root);

    let line_map = LineMap::build(source);
    let range = range_for_substring(source, &line_map, "foo");
    let diag = LspDiagnostic {
        range,
        severity: Some(DiagnosticSeverity::Warning),
        code: Some(UNUSED_IMPORT),
        source: None,
        message: "unused import".to_string(),
        related_information: None,
    };

    let provider = CodeActionProvider::new(
        arena,
        &binder,
        &line_map,
        "test.ts".to_string(),
        source,
    );

    let empty_range = Range::new(Position::new(0, 0), Position::new(0, 0));
    let actions = provider.provide_code_actions(
        root,
        empty_range,
        CodeActionContext {
            diagnostics: vec![diag],
            only: Some(vec![CodeActionKind::QuickFix]),
        },
    );

    assert_eq!(actions.len(), 1);
    let edit = actions[0].edit.as_ref().unwrap();
    let edits = edit.changes.get("test.ts").unwrap();
    assert_eq!(edits.len(), 1);
    assert_eq!(edits[0].new_text, "");
}

#[test]
fn test_quickfix_remove_unused_default_import() {
    let source = "import foo, { bar } from \"mod\";\nbar;\n";
    let mut parser = ThinParserState::new("test.ts".to_string(), source.to_string());
    let root = parser.parse_source_file();
    let arena = parser.get_arena();

    let mut binder = ThinBinderState::new();
    binder.bind_source_file(arena, root);

    let line_map = LineMap::build(source);
    let range = range_for_substring(source, &line_map, "foo");
    let diag = LspDiagnostic {
        range,
        severity: Some(DiagnosticSeverity::Warning),
        code: Some(UNUSED_IMPORT),
        source: None,
        message: "unused import".to_string(),
        related_information: None,
    };

    let provider = CodeActionProvider::new(
        arena,
        &binder,
        &line_map,
        "test.ts".to_string(),
        source,
    );

    let empty_range = Range::new(Position::new(0, 0), Position::new(0, 0));
    let actions = provider.provide_code_actions(
        root,
        empty_range,
        CodeActionContext {
            diagnostics: vec![diag],
            only: Some(vec![CodeActionKind::QuickFix]),
        },
    );

    assert_eq!(actions.len(), 1);
    let edit = actions[0].edit.as_ref().unwrap();
    let edits = edit.changes.get("test.ts").unwrap();
    assert_eq!(edits.len(), 1);
    assert_eq!(edits[0].new_text, "import { bar } from \"mod\";\n");
}

#[test]
fn test_quickfix_preserves_type_only_named_import() {
    let source = "import { type Foo, Bar } from \"mod\";\nlet x: Foo;\n";
    let mut parser = ThinParserState::new("test.ts".to_string(), source.to_string());
    let root = parser.parse_source_file();
    let arena = parser.get_arena();

    let mut binder = ThinBinderState::new();
    binder.bind_source_file(arena, root);

    let line_map = LineMap::build(source);
    let range = range_for_substring(source, &line_map, "Bar");
    let diag = LspDiagnostic {
        range,
        severity: Some(DiagnosticSeverity::Warning),
        code: Some(UNUSED_IMPORT),
        source: None,
        message: "unused import".to_string(),
        related_information: None,
    };

    let provider = CodeActionProvider::new(
        arena,
        &binder,
        &line_map,
        "test.ts".to_string(),
        source,
    );

    let empty_range = Range::new(Position::new(0, 0), Position::new(0, 0));
    let actions = provider.provide_code_actions(
        root,
        empty_range,
        CodeActionContext {
            diagnostics: vec![diag],
            only: Some(vec![CodeActionKind::QuickFix]),
        },
    );

    assert_eq!(actions.len(), 1);
    let edit = actions[0].edit.as_ref().unwrap();
    let edits = edit.changes.get("test.ts").unwrap();
    assert_eq!(edits.len(), 1);
    assert_eq!(edits[0].new_text, "import { type Foo } from \"mod\";\n");
}
