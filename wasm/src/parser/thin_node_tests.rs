use super::*;
use std::mem::size_of;

#[test]
fn test_thin_node_size() {
    // This is the critical test - ThinNode MUST be 16 bytes
    assert_eq!(size_of::<ThinNode>(), 16, "ThinNode must be exactly 16 bytes");

    // 4 nodes per cache line
    let nodes_per_cache_line = 64 / size_of::<ThinNode>();
    assert_eq!(nodes_per_cache_line, 4, "Should fit 4 ThinNodes per 64-byte cache line");
}

#[test]
fn test_thin_node_arena_basic() {
    use crate::scanner::SyntaxKind;

    let mut arena = ThinNodeArena::new();

    // Add a token (no data)
    let token = arena.add_token(SyntaxKind::AsteriskToken as u16, 0, 5);
    assert_eq!(token.0, 0);

    // Add an identifier
    let ident = arena.add_identifier(
        SyntaxKind::Identifier as u16,
        10,
        15,
        IdentifierData {
            escaped_text: "hello".to_string(),
            original_text: None,
            type_arguments: None,
        },
    );
    assert_eq!(ident.0, 1);

    // Verify we can retrieve them
    let node = arena.get(token).unwrap();
    assert_eq!(node.kind, SyntaxKind::AsteriskToken as u16);
    assert_eq!(node.pos, 0);
    assert_eq!(node.end, 5);
    assert!(!node.has_data());

    let node = arena.get(ident).unwrap();
    assert_eq!(node.kind, SyntaxKind::Identifier as u16);
    assert!(node.has_data());

    let data = arena.get_identifier(node).unwrap();
    assert_eq!(data.escaped_text, "hello");
}

#[test]
fn test_data_pool_sizes() {
    // Verify data pool element sizes are reasonable
    assert!(size_of::<IdentifierData>() <= 120, "IdentifierData too large");
    assert!(size_of::<FunctionData>() <= 168, "FunctionData too large");
    assert!(size_of::<ClassData>() <= 200, "ClassData too large");
    assert!(size_of::<SourceFileData>() <= 200, "SourceFileData too large");
}

#[test]
fn test_node_view() {
    use crate::scanner::SyntaxKind;

    let mut arena = ThinNodeArena::new();

    // Add an identifier
    let ident_idx = arena.add_identifier(
        SyntaxKind::Identifier as u16,
        10,
        15,
        IdentifierData {
            escaped_text: "myVar".to_string(),
            original_text: None,
            type_arguments: None,
        },
    );

    // Create a view and access data through it
    let view = NodeView::new(&arena, ident_idx).unwrap();
    assert_eq!(view.kind(), SyntaxKind::Identifier as u16);
    assert_eq!(view.pos(), 10);
    assert_eq!(view.end(), 15);
    assert!(view.has_data());

    let ident = view.as_identifier().unwrap();
    assert_eq!(ident.escaped_text, "myVar");
}

#[test]
fn test_node_kind_utilities() {
    use crate::scanner::SyntaxKind;
    use super::super::syntax_kind_ext::*;

    let ident = ThinNode::new(SyntaxKind::Identifier as u16, 0, 5);
    assert!(ident.is_identifier());
    assert!(!ident.is_string_literal());

    let func = ThinNode::new(FUNCTION_DECLARATION, 0, 100);
    assert!(func.is_function_declaration());
    assert!(func.is_function_like());
    assert!(func.is_declaration());

    let class = ThinNode::new(CLASS_DECLARATION, 0, 200);
    assert!(class.is_class_declaration());
    assert!(class.is_declaration());

    let block = ThinNode::new(BLOCK, 0, 50);
    assert!(block.is_statement());

    let type_ref = ThinNode::new(TYPE_REFERENCE, 0, 10);
    assert!(type_ref.is_type_node());
}

#[test]
fn test_node_access_trait() {
    use crate::scanner::SyntaxKind;

    let mut arena = ThinNodeArena::new();

    // Add an identifier
    let ident_idx = arena.add_identifier(
        SyntaxKind::Identifier as u16,
        10,
        20,
        IdentifierData {
            escaped_text: "testVar".to_string(),
            original_text: None,
            type_arguments: None,
        },
    );

    // Test NodeAccess trait methods
    assert!(arena.exists(ident_idx));
    assert!(!arena.exists(NodeIndex::NONE));

    assert_eq!(arena.kind(ident_idx), Some(SyntaxKind::Identifier as u16));
    assert_eq!(arena.pos_end(ident_idx), Some((10, 20)));
    assert_eq!(arena.get_identifier_text(ident_idx), Some("testVar"));

    // Test NodeInfo
    let info = arena.node_info(ident_idx).unwrap();
    assert_eq!(info.kind, SyntaxKind::Identifier as u16);
    assert_eq!(info.pos, 10);
    assert_eq!(info.end, 20);
}
