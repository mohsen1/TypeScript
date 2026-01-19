//! TypeScript Parser CLI
//!
//! A simple CLI for testing the TypeScript parser.

use ts_parser::{NodeArena, NodeId, NodeKind, ThinNode};

fn main() {
    println!("TypeScript Parser v{}", ts_parser::version());
    println!();

    // Demo: Create a simple AST structure
    let mut arena = NodeArena::with_capacity(1000, 2000);

    // Simulate parsing: const greeting = "Hello, World!";
    let source_file = arena.alloc_node(NodeKind::SourceFile, 0, 34);
    let var_stmt = arena.alloc_node(NodeKind::VariableStatement, 0, 34);
    let var_decl_list = arena.alloc_node(NodeKind::VariableDeclarationList, 0, 33);
    let var_decl = arena.alloc_node(NodeKind::VariableDeclaration, 6, 33);
    let identifier = arena.alloc_node(NodeKind::Identifier, 6, 14);
    let string_lit = arena.alloc_node(NodeKind::StringLiteral, 17, 33);

    // Intern strings
    let greeting_str = arena.intern_string("greeting");
    arena.set_string(identifier, greeting_str);

    let hello_str = arena.intern_string("Hello, World!");
    arena.set_string(string_lit, hello_str);

    // Build tree
    arena.add_children(var_decl, &[identifier, string_lit]);
    arena.add_children(var_decl_list, &[var_decl]);
    arena.add_children(var_stmt, &[var_decl_list]);
    arena.add_children(source_file, &[var_stmt]);

    println!("AST Demo: const greeting = \"Hello, World!\";");
    println!();
    println!("Arena statistics:");
    println!("  Nodes: {}", arena.len());
    println!(
        "  ThinNode size: {} bytes",
        std::mem::size_of::<ThinNode>()
    );
    println!("  NodeKind size: {} bytes", std::mem::size_of::<NodeKind>());
    println!("  NodeId size: {} bytes", std::mem::size_of::<NodeId>());
    println!();

    // Walk and print the AST
    println!("AST structure:");
    print_ast(&arena, source_file, 0);
}

fn print_ast(arena: &NodeArena, node_id: NodeId, indent: usize) {
    if let Some(node) = arena.get(node_id) {
        let prefix = "  ".repeat(indent);
        let name = if !node.string_id.is_empty() {
            if let Some(s) = arena.get_string(node.string_id) {
                format!(" \"{}\"", s)
            } else {
                String::new()
            }
        } else {
            String::new()
        };

        println!(
            "{}{}{} ({}..{})",
            prefix,
            node.kind.name(),
            name,
            node.pos,
            node.end
        );

        for &child_id in arena.get_children(node_id) {
            print_ast(arena, child_id, indent + 1);
        }
    }
}
