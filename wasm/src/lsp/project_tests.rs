//! Project-level LSP tests.

use super::*;

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
