use super::*;
use crate::emitter::ScriptTarget;
use crate::parser_impl::ParserState;
use crate::transforms::transform_source_file;

fn parse_and_transform(source: &str, _target: ScriptTarget) -> HelpersNeeded {
    let mut parser = ParserState::new("test.ts".to_string(), source.to_string());
    let source_file = parser.parse_source_file();
    let mut arena = parser.arena;

    // For module transforms, we need to run the transformer directly
    // since the main transform_source_file checks for ES5 target
    let mut ctx = super::super::TransformContext::new(_target, &mut arena);
    let mut transformer = ModuleTransformer::new(ModuleKind::CommonJS);
    transformer.visit_node(source_file, &mut ctx);
    ctx.helpers_needed
}

#[test]
fn test_import_needs_import_default() {
    let helpers = parse_and_transform(
        "import foo from './bar';",
        ScriptTarget::ES5,
    );
    assert!(helpers.import_default);
}

#[test]
fn test_export_star_needs_export_star() {
    let helpers = parse_and_transform(
        "export * from './bar';",
        ScriptTarget::ES5,
    );
    assert!(helpers.export_star);
}

#[test]
fn test_es_module_marker() {
    let transformer = ModuleTransformer::new(ModuleKind::CommonJS);
    let marker = transformer.emit_es_module_marker();
    assert!(marker.contains("__esModule"));
}

#[test]
fn test_amd_wrapper() {
    let transformer = ModuleTransformer::new(ModuleKind::AMD);
    let wrapper = transformer.emit_amd_wrapper(&[], "");
    assert!(wrapper.contains("define"));
}

#[test]
fn test_umd_wrapper() {
    let transformer = ModuleTransformer::new(ModuleKind::UMD);
    let wrapper = transformer.emit_umd_wrapper(&[], "");
    assert!(wrapper.contains("function"));
}

#[test]
fn test_import_transform_creates_require_call() {
    let mut parser = ParserState::new("test.ts".to_string(), "import foo from './bar';".to_string());
    let source_file = parser.parse_source_file();
    let mut arena = parser.arena;

    let mut ctx = super::super::TransformContext::new(ScriptTarget::ES5, &mut arena);
    let mut transformer = ModuleTransformer::new(ModuleKind::CommonJS);

    // Get the import declaration (first statement)
    if let Some(Node::SourceFile(sf)) = ctx.arena.get(source_file) {
        if let Some(&import_idx) = sf.statements.nodes.first() {
            // Transform the import
            let result = transformer.transform_import_declaration(import_idx, &mut ctx);

            // Verify transformation produced a result
            assert!(result.is_some(), "Import transform should produce a result");

            // Verify the result is a variable statement
            let result_idx = result.unwrap();
            assert!(matches!(ctx.arena.get(result_idx), Some(Node::VariableStatement(_))),
                "Transformed import should be a variable statement");
        }
    }
}

#[test]
fn test_namespace_import_needs_import_star() {
    let helpers = parse_and_transform(
        "import * as foo from './bar';",
        ScriptTarget::ES5,
    );
    assert!(helpers.import_star);
}

#[test]
fn test_export_star_transform() {
    let mut parser = ParserState::new("test.ts".to_string(), "export * from './bar';".to_string());
    let source_file = parser.parse_source_file();
    let mut arena = parser.arena;

    let mut ctx = super::super::TransformContext::new(ScriptTarget::ES5, &mut arena);
    let mut transformer = ModuleTransformer::new(ModuleKind::CommonJS);

    // Get the export declaration (first statement)
    if let Some(Node::SourceFile(sf)) = ctx.arena.get(source_file) {
        if let Some(&export_idx) = sf.statements.nodes.first() {
            // Transform the export
            let result = transformer.transform_export_declaration(export_idx, &mut ctx);

            // Verify transformation produced a result
            assert!(result.is_some(), "Export * transform should produce a result");

            // Verify the result is an expression statement (for __exportStar call)
            let result_idx = result.unwrap();
            assert!(matches!(ctx.arena.get(result_idx), Some(Node::ExpressionStatement(_))),
                "Transformed export * should be an expression statement");
        }
    }
}

#[test]
fn test_named_export_transform() {
    let mut parser = ParserState::new("test.ts".to_string(), "const foo = 1; export { foo };".to_string());
    let source_file = parser.parse_source_file();
    let mut arena = parser.arena;

    let mut ctx = super::super::TransformContext::new(ScriptTarget::ES5, &mut arena);
    let mut transformer = ModuleTransformer::new(ModuleKind::CommonJS);

    // Get the export declaration (second statement)
    if let Some(Node::SourceFile(sf)) = ctx.arena.get(source_file) {
        if let Some(&export_idx) = sf.statements.nodes.get(1) {
            // Transform the export
            let result = transformer.transform_export_declaration(export_idx, &mut ctx);

            // Verify transformation produced a result
            assert!(result.is_some(), "Named export transform should produce a result");

            // Verify the result is an expression statement (exports.foo = foo)
            let result_idx = result.unwrap();
            assert!(matches!(ctx.arena.get(result_idx), Some(Node::ExpressionStatement(_))),
                "Transformed named export should be an expression statement");
        }
    }
}
