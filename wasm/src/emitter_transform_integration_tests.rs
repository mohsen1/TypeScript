//! Integration tests for Transform/Print separation architecture
//!
//! These tests demonstrate the complete two-phase emission pipeline:
//! 1. LoweringPass analyzes AST and produces TransformDirective entries
//! 2. ThinPrinter consults TransformContext and applies transforms during emission

use crate::emit_context::EmitContext;
use crate::lowering_pass::LoweringPass;
use crate::thin_emitter::ThinPrinter;
use crate::thin_parser::ThinParserState;

#[test]
fn test_two_phase_emission_es5_class() {
    // Parse source
    let source = "class Point { constructor(x, y) { this.x = x; this.y = y; } }";
    let mut parser = ThinParserState::new("test.ts".to_string(), source.to_string());
    let root = parser.parse_source_file();
    let arena = parser.arena;

    // Phase 1: Lowering Pass (Transform)
    let mut ctx = EmitContext::default();
    ctx.target_es5 = true;

    let lowering = LoweringPass::new(&arena, &ctx);
    let transforms = lowering.run(root);

    // Verify transforms were generated
    assert!(
        !transforms.is_empty(),
        "LoweringPass should generate ES5Class transform"
    );

    // Phase 2: Print Pass (with Transforms)
    let mut printer = ThinPrinter::with_transforms(&arena, transforms);
    printer.set_target_es5(true);
    printer.emit(root);

    let output = printer.get_output();

    // Verify ES5 IIFE pattern was emitted
    assert!(
        output.contains("var Point"),
        "ES5 output should contain 'var Point'"
    );
    assert!(
        output.contains("function ()"),
        "ES5 output should contain IIFE pattern"
    );
    assert!(
        output.contains("return Point"),
        "ES5 output should return constructor"
    );
}

#[test]
fn test_two_phase_emission_es6_class_no_transform() {
    // Parse source
    let source = "class Point { constructor(x, y) { this.x = x; this.y = y; } }";
    let mut parser = ThinParserState::new("test.ts".to_string(), source.to_string());
    let root = parser.parse_source_file();
    let arena = parser.arena;

    // Phase 1: Lowering Pass (Transform) - ES6 target
    let ctx = EmitContext::default(); // ES6 by default

    let lowering = LoweringPass::new(&arena, &ctx);
    let transforms = lowering.run(root);

    // Verify no transforms were generated for ES6
    assert!(
        transforms.is_empty(),
        "LoweringPass should NOT generate transforms for ES6 target"
    );

    // Phase 2: Print Pass (with empty Transforms)
    let mut printer = ThinPrinter::with_transforms(&arena, transforms);
    printer.set_target_es5(false); // ES6 target
    printer.emit(root);

    let output = printer.get_output();

    // Verify native class syntax was emitted
    assert!(
        output.contains("class Point"),
        "ES6 output should contain 'class Point'"
    );
    assert!(
        !output.contains("var Point"),
        "ES6 output should NOT contain 'var Point' (no IIFE)"
    );
}

#[test]
fn test_two_phase_backward_compatibility() {
    // Parse source
    let source = "class Foo {}";
    let mut parser = ThinParserState::new("test.ts".to_string(), source.to_string());
    let root = parser.parse_source_file();
    let arena = parser.arena;

    // Old way: printer without transforms (backward compatibility)
    let mut printer_old = ThinPrinter::new(&arena);
    printer_old.set_target_es5(true);
    printer_old.emit(root);
    let output_old = printer_old.get_output();

    // New way: printer with empty transforms
    let transforms = crate::transform_context::TransformContext::new();
    let mut printer_new = ThinPrinter::with_transforms(&arena, transforms);
    printer_new.set_target_es5(true);
    printer_new.emit(root);
    let output_new = printer_new.get_output();

    // Both should produce the same output (backward compatibility)
    assert_eq!(
        output_old, output_new,
        "Old and new emission paths should produce identical output"
    );
}

#[test]
fn test_transform_directive_composability() {
    // This test verifies that the architecture supports composable transforms
    // For now, we just verify that the TransformContext can be created and passed around
    use crate::transform_context::{TransformContext, TransformDirective};
    use crate::parser::NodeIndex;

    let mut ctx = TransformContext::new();

    // Add a transform directive
    ctx.insert(
        NodeIndex(1),
        TransformDirective::ES5Class {
            class_node: NodeIndex(1),
            class_name: Some("TestClass".to_string()),
            heritage: None,
            members: vec![],
        },
    );

    // Verify it was stored
    assert!(ctx.has_transform(NodeIndex(1)));
    assert!(!ctx.has_transform(NodeIndex(2)));
}
