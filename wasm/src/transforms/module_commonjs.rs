//! CommonJS Module Transform
//!
//! Transforms ES modules to CommonJS format:
//!
//! ```typescript
//! import { foo } from "./module";
//! export const bar = 42;
//! export default myFunc;
//! ```
//!
//! Becomes:
//!
//! ```javascript
//! "use strict";
//! Object.defineProperty(exports, "__esModule", { value: true });
//! exports.bar = void 0;
//! var module_1 = require("./module");
//! exports.bar = 42;
//! exports.default = myFunc;
//! ```

use crate::parser::thin_node::{ThinNode, ThinNodeArena};
use crate::parser::NodeIndex;
use crate::parser::syntax_kind_ext;
use crate::scanner::SyntaxKind;

/// Emit the CommonJS module preamble
///
/// Outputs:
/// ```javascript
/// "use strict";
/// Object.defineProperty(exports, "__esModule", { value: true });
/// ```
pub fn emit_commonjs_preamble(writer: &mut impl std::fmt::Write) -> std::fmt::Result {
    writeln!(writer, "\"use strict\";")?;
    writeln!(writer, "Object.defineProperty(exports, \"__esModule\", {{ value: true }});")?;
    Ok(())
}

/// Helper function to collect export name from a single declaration node
fn collect_export_name_from_declaration(arena: &ThinNodeArena, decl_node: &ThinNode, exports: &mut Vec<String>) {
    match decl_node.kind {
        k if k == syntax_kind_ext::CLASS_DECLARATION => {
            if let Some(class) = arena.get_class(decl_node) {
                if let Some(name) = get_identifier_text(arena, class.name) {
                    exports.push(name);
                }
            }
        }
        k if k == syntax_kind_ext::FUNCTION_DECLARATION => {
            if let Some(func) = arena.get_function(decl_node) {
                if let Some(name) = get_identifier_text(arena, func.name) {
                    exports.push(name);
                }
            }
        }
        k if k == syntax_kind_ext::VARIABLE_STATEMENT => {
            if let Some(var_stmt) = arena.get_variable(decl_node) {
                for &decl_idx in &var_stmt.declarations.nodes {
                    collect_declaration_names(arena, decl_idx, exports);
                }
            }
        }
        k if k == syntax_kind_ext::ENUM_DECLARATION => {
            if let Some(enum_decl) = arena.get_enum(decl_node) {
                if let Some(name) = get_identifier_text(arena, enum_decl.name) {
                    exports.push(name);
                }
            }
        }
        k if k == syntax_kind_ext::MODULE_DECLARATION => {
            if let Some(module) = arena.get_module(decl_node) {
                if let Some(name) = get_identifier_text(arena, module.name) {
                    exports.push(name);
                }
            }
        }
        _ => {
            // Interface, Type Alias, etc. don't need runtime exports
        }
    }
}

/// Collect all export names from a source file for the exports initialization
///
/// Returns a list of exported names (e.g., ["foo", "bar"])
pub fn collect_export_names(arena: &ThinNodeArena, statements: &[NodeIndex]) -> Vec<String> {
    let mut exports = Vec::new();

    for &stmt_idx in statements {
        let Some(node) = arena.get(stmt_idx) else { continue };

        match node.kind {
            // export class C {} / export function f() {} / export { x } / export default ...
            // These are wrapped in EXPORT_DECLARATION nodes
            k if k == syntax_kind_ext::EXPORT_DECLARATION => {
                if let Some(export_decl) = arena.get_export_decl(node) {
                    if export_decl.is_default_export {
                        exports.push("default".to_string());
                        continue;
                    }

                    // Only pre-initialize local exports (no module specifier)
                    if export_decl.module_specifier.is_none() {
                        if let Some(clause_node) = arena.get(export_decl.export_clause) {
                            if let Some(named_exports) = arena.get_named_imports(clause_node) {
                                for &spec_idx in &named_exports.elements.nodes {
                                    if let Some(spec) = arena.get(spec_idx).and_then(|n| arena.get_specifier(n)) {
                                        // Use the exported name (name), not the local name (property_name)
                                        if let Some(name) = get_identifier_text(arena, spec.name) {
                                            exports.push(name);
                                        }
                                    }
                                }
                            } else {
                                collect_export_name_from_declaration(arena, clause_node, &mut exports);
                            }
                        }
                    }
                }
            }
            // export const foo = ...
            // export let bar = ...
            // export var baz = ...
            k if k == syntax_kind_ext::VARIABLE_STATEMENT => {
                if let Some(var_stmt) = arena.get_variable(node) {
                    if has_export_modifier_from_list(arena, &var_stmt.modifiers) {
                        for &decl_idx in &var_stmt.declarations.nodes {
                            collect_declaration_names(arena, decl_idx, &mut exports);
                        }
                    }
                }
            }
            // export function foo() {}
            k if k == syntax_kind_ext::FUNCTION_DECLARATION => {
                if let Some(func) = arena.get_function(node) {
                    if has_export_modifier_from_list(arena, &func.modifiers) {
                        if let Some(name) = get_identifier_text(arena, func.name) {
                            exports.push(name);
                        }
                    }
                }
            }
            // export class Foo {}
            k if k == syntax_kind_ext::CLASS_DECLARATION => {
                if let Some(class) = arena.get_class(node) {
                    if has_export_modifier_from_list(arena, &class.modifiers) {
                        if let Some(name) = get_identifier_text(arena, class.name) {
                            exports.push(name);
                        }
                    }
                }
            }
            // export enum E {}
            k if k == syntax_kind_ext::ENUM_DECLARATION => {
                if let Some(enum_decl) = arena.get_enum(node) {
                    if has_export_modifier_from_list(arena, &enum_decl.modifiers) {
                        if let Some(name) = get_identifier_text(arena, enum_decl.name) {
                            exports.push(name);
                        }
                    }
                }
            }
            // export namespace N {}
            k if k == syntax_kind_ext::MODULE_DECLARATION => {
                if let Some(module) = arena.get_module(node) {
                    if has_export_modifier_from_list(arena, &module.modifiers) {
                        if let Some(name) = get_identifier_text(arena, module.name) {
                            exports.push(name);
                        }
                    }
                }
            }
            _ => {}
        }
    }

    exports
}

/// Emit the exports initialization line
///
/// ```javascript
/// exports.foo = exports.bar = void 0;
/// ```
pub fn emit_exports_init(writer: &mut impl std::fmt::Write, exports: &[String]) -> std::fmt::Result {
    if exports.is_empty() {
        return Ok(());
    }

    // Build: exports.a = exports.b = ... = void 0;
    for (i, name) in exports.iter().enumerate() {
        if i > 0 {
            write!(writer, " = ")?;
        }
        write!(writer, "exports.{}", name)?;
    }
    writeln!(writer, " = void 0;")?;

    Ok(())
}

/// Transform an import declaration to CommonJS require
///
/// ```typescript
/// import { foo, bar } from "./module";
/// ```
/// Becomes:
/// ```javascript
/// var module_1 = require("./module");
/// ```
pub fn transform_import_to_require(
    arena: &ThinNodeArena,
    node: &ThinNode,
    module_counter: &mut u32,
) -> Option<(String, String)> {
    let import = arena.get_import_decl(node)?;

    // Get module specifier text
    let module_spec = get_string_literal_text(arena, import.module_specifier)?;

    // Generate module variable name (e.g., module_1, module_2)
    *module_counter += 1;
    let var_name = format!("{}_1", sanitize_module_name(&module_spec));

    // Return (var_name, require_statement)
    let require_stmt = format!("var {} = require(\"{}\");", var_name, module_spec);

    Some((var_name, require_stmt))
}

/// Transform import bindings to variable declarations
///
/// For:
/// ```typescript
/// import { foo, bar as baz } from "./module";
/// ```
///
/// After `var module_1 = require("./module");`:
/// We don't need separate var declarations - just use module_1.foo directly
///
/// For default imports:
/// ```typescript
/// import myDefault from "./module";
/// ```
/// Becomes:
/// ```javascript
/// var myDefault = module_1.default;
/// ```
pub fn get_import_bindings(
    arena: &ThinNodeArena,
    node: &ThinNode,
    module_var: &str,
) -> Vec<String> {
    let mut bindings = Vec::new();

    let Some(import) = arena.get_import_decl(node) else {
        return bindings;
    };

    let Some(clause_node) = arena.get(import.import_clause) else {
        return bindings;
    };

    let Some(clause) = arena.get_import_clause(clause_node) else {
        return bindings;
    };

    // Default import: import foo from "..."
    if !clause.name.is_none() {
        if let Some(name) = get_identifier_text(arena, clause.name) {
            bindings.push(format!("var {} = {}.default;", name, module_var));
        }
    }

    // Named bindings: import { a, b as c } from "..." or import * as ns from "..."
    if !clause.named_bindings.is_none() {
        if let Some(named_node) = arena.get(clause.named_bindings) {
            // NamedImportsData handles both namespace and named imports
            if let Some(named_imports) = arena.get_named_imports(named_node) {
                // Check if it's a namespace import: import * as ns from "..."
                // Namespace imports have a name but no elements
                if !named_imports.name.is_none() && named_imports.elements.nodes.is_empty() {
                    if let Some(name) = get_identifier_text(arena, named_imports.name) {
                        // Use __importStar helper for namespace imports
                        bindings.push(format!("var {} = __importStar({});", name, module_var));
                    }
                } else {
                    // Named imports: import { a, b } from "..."
                    for &spec_idx in &named_imports.elements.nodes {
                        if let Some(spec_node) = arena.get(spec_idx) {
                            if let Some(spec) = arena.get_specifier(spec_node) {
                                let local_name = get_identifier_text(arena, spec.name).unwrap_or_default();
                                let import_name = if !spec.property_name.is_none() {
                                    get_identifier_text(arena, spec.property_name).unwrap_or(local_name.clone())
                                } else {
                                    local_name.clone()
                                };
                                bindings.push(format!("var {} = {}.{};", local_name, module_var, import_name));
                            }
                        }
                    }
                }
            }
        }
    }

    bindings
}

/// Generate export assignment for a name
///
/// ```javascript
/// exports.foo = foo;
/// ```
pub fn emit_export_assignment(name: &str) -> String {
    format!("exports.{} = {};", name, name)
}

/// Generate Object.defineProperty for re-exports
///
/// For:
/// ```typescript
/// export { foo } from "./module";
/// ```
/// Becomes:
/// ```javascript
/// Object.defineProperty(exports, "foo", { enumerable: true, get: function () { return module_1.foo; } });
/// ```
pub fn emit_reexport_property(export_name: &str, module_var: &str, import_name: &str) -> String {
    format!(
        "Object.defineProperty(exports, \"{}\", {{ enumerable: true, get: function () {{ return {}.{}; }} }});",
        export_name, module_var, import_name
    )
}

// =============================================================================
// Helper Functions
// =============================================================================

/// Check if modifiers contain a specific modifier kind
fn has_modifier(arena: &ThinNodeArena, modifiers: &Option<crate::parser::NodeList>, kind: u16) -> bool {
    if let Some(mods) = modifiers {
        for &mod_idx in &mods.nodes {
            if let Some(mod_node) = arena.get(mod_idx) {
                if mod_node.kind == kind {
                    return true;
                }
            }
        }
    }
    false
}

/// Check if a node has the `export` modifier
fn has_export_modifier_from_list(arena: &ThinNodeArena, modifiers: &Option<crate::parser::NodeList>) -> bool {
    has_modifier(arena, modifiers, SyntaxKind::ExportKeyword as u16)
}

/// Check if a node has the `default` modifier
pub fn has_default_modifier_from_list(arena: &ThinNodeArena, modifiers: &Option<crate::parser::NodeList>) -> bool {
    has_modifier(arena, modifiers, SyntaxKind::DefaultKeyword as u16)
}

/// Collect exported names from a variable declaration (identifier or binding pattern).
fn collect_declaration_names(arena: &ThinNodeArena, decl_idx: NodeIndex, exports: &mut Vec<String>) {
    let Some(decl_node) = arena.get(decl_idx) else {
        return;
    };

    if decl_node.kind == syntax_kind_ext::VARIABLE_DECLARATION_LIST {
        if let Some(decl_list) = arena.get_variable(decl_node) {
            for &inner_decl_idx in &decl_list.declarations.nodes {
                collect_declaration_names(arena, inner_decl_idx, exports);
            }
        }
        return;
    }

    if let Some(decl) = arena.get_variable_declaration(decl_node) {
        collect_binding_names(arena, decl.name, exports);
    }
}

fn collect_binding_names(arena: &ThinNodeArena, name_idx: NodeIndex, exports: &mut Vec<String>) {
    if name_idx.is_none() {
        return;
    }

    let Some(node) = arena.get(name_idx) else {
        return;
    };

    if node.kind == SyntaxKind::Identifier as u16 {
        if let Some(id) = arena.get_identifier(node) {
            exports.push(id.escaped_text.clone());
        }
        return;
    }

    match node.kind {
        k if k == syntax_kind_ext::OBJECT_BINDING_PATTERN
            || k == syntax_kind_ext::ARRAY_BINDING_PATTERN =>
        {
            if let Some(pattern) = arena.get_binding_pattern(node) {
                for &elem_idx in &pattern.elements.nodes {
                    collect_binding_names_from_element(arena, elem_idx, exports);
                }
            }
        }
        k if k == syntax_kind_ext::BINDING_ELEMENT => {
            if let Some(elem) = arena.get_binding_element(node) {
                collect_binding_names(arena, elem.name, exports);
            }
        }
        _ => {}
    }
}

fn collect_binding_names_from_element(
    arena: &ThinNodeArena,
    elem_idx: NodeIndex,
    exports: &mut Vec<String>,
) {
    if elem_idx.is_none() {
        return;
    }

    let Some(elem_node) = arena.get(elem_idx) else {
        return;
    };

    if let Some(elem) = arena.get_binding_element(elem_node) {
        collect_binding_names(arena, elem.name, exports);
    }
}

/// Get identifier text from a node index
fn get_identifier_text(arena: &ThinNodeArena, idx: NodeIndex) -> Option<String> {
    let node = arena.get(idx)?;
    if node.kind == SyntaxKind::Identifier as u16 {
        arena.get_identifier(node).map(|id| id.escaped_text.clone())
    } else {
        None
    }
}

/// Get string literal text from a node index
fn get_string_literal_text(arena: &ThinNodeArena, idx: NodeIndex) -> Option<String> {
    let node = arena.get(idx)?;
    if node.kind == SyntaxKind::StringLiteral as u16 {
        arena.get_literal(node).map(|s| s.text.clone())
    } else {
        None
    }
}

/// Sanitize module specifier for use as variable name
/// "./foo/bar" -> "foo_bar"
pub fn sanitize_module_name(module_spec: &str) -> String {
    module_spec
        .trim_start_matches("./")
        .trim_start_matches("../")
        .replace(['/', '-', '.', '@'], "_")
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_sanitize_module_name() {
        assert_eq!(sanitize_module_name("./foo"), "foo");
        assert_eq!(sanitize_module_name("./foo/bar"), "foo_bar");
        assert_eq!(sanitize_module_name("../utils"), "utils");
        assert_eq!(sanitize_module_name("@scope/pkg"), "_scope_pkg");
    }

    #[test]
    fn test_emit_commonjs_preamble() {
        let mut output = String::new();
        emit_commonjs_preamble(&mut output).unwrap();
        assert!(output.contains("\"use strict\";"));
        assert!(output.contains("Object.defineProperty(exports, \"__esModule\""));
    }

    #[test]
    fn test_emit_exports_init() {
        let mut output = String::new();
        emit_exports_init(&mut output, &["foo".to_string(), "bar".to_string()]).unwrap();
        assert_eq!(output, "exports.foo = exports.bar = void 0;\n");
    }

    #[test]
    fn test_emit_export_assignment() {
        assert_eq!(emit_export_assignment("foo"), "exports.foo = foo;");
    }

    #[test]
    fn test_emit_reexport_property() {
        let result = emit_reexport_property("foo", "module_1", "foo");
        assert!(result.contains("Object.defineProperty"));
        assert!(result.contains("\"foo\""));
        assert!(result.contains("module_1.foo"));
    }

    #[test]
    fn test_collect_export_names_with_parsed_ast() {
        use crate::thin_parser::ThinParserState;
        use crate::parser::syntax_kind_ext;

        let source = "export class C {}";
        let mut parser = ThinParserState::new("test.ts".to_string(), source.to_string());
        let root = parser.parse_source_file();

        let Some(source_file) = parser.arena.get_source_file(parser.arena.get(root).unwrap()) else {
            panic!("Failed to get source file");
        };

        // Debug: print the statements
        eprintln!("Source file has {} statements", source_file.statements.nodes.len());
        for (i, &stmt_idx) in source_file.statements.nodes.iter().enumerate() {
            if let Some(node) = parser.arena.get(stmt_idx) {
                eprintln!("Statement {}: kind = {} (ClassDecl = {})",
                    i, node.kind, syntax_kind_ext::CLASS_DECLARATION);

                if node.kind == syntax_kind_ext::CLASS_DECLARATION {
                    if let Some(class) = parser.arena.get_class(node) {
                        eprintln!("  Found class, modifiers: {:?}", class.modifiers);
                        if let Some(modifiers) = &class.modifiers {
                            eprintln!("  Modifiers count: {}", modifiers.nodes.len());
                            for &mod_idx in &modifiers.nodes {
                                if let Some(mod_node) = parser.arena.get(mod_idx) {
                                    eprintln!("    Modifier kind: {} (Export = {})",
                                        mod_node.kind, SyntaxKind::ExportKeyword as u16);
                                }
                            }
                        }
                        if let Some(name_node) = parser.arena.get(class.name) {
                            if let Some(ident) = parser.arena.get_identifier(name_node) {
                                eprintln!("  Class name: {}", ident.escaped_text);
                            }
                        }
                    }
                }
            }
        }

        let export_names = collect_export_names(&parser.arena, &source_file.statements.nodes);

        eprintln!("Collected export names: {:?}", export_names);

        assert!(!export_names.is_empty(), "Expected to find exported class name");
        assert_eq!(export_names, vec!["C"], "Expected to find class name 'C' in exports");
    }

    #[test]
    fn test_collect_export_names_with_destructuring() {
        use crate::thin_parser::ThinParserState;

        let source = "export const { a, b: c } = obj;";
        let mut parser = ThinParserState::new("test.ts".to_string(), source.to_string());
        let root = parser.parse_source_file();

        let Some(source_file) = parser.arena.get_source_file(parser.arena.get(root).unwrap()) else {
            panic!("Failed to get source file");
        };

        let export_names = collect_export_names(&parser.arena, &source_file.statements.nodes);

        assert_eq!(
            export_names,
            vec!["a", "c"],
            "Expected destructured export names"
        );
    }

    #[test]
    fn test_collect_export_names_with_default_export() {
        use crate::thin_parser::ThinParserState;

        let source = "export default function () {}";
        let mut parser = ThinParserState::new("test.ts".to_string(), source.to_string());
        let root = parser.parse_source_file();

        let Some(source_file) = parser.arena.get_source_file(parser.arena.get(root).unwrap()) else {
            panic!("Failed to get source file");
        };

        let export_names = collect_export_names(&parser.arena, &source_file.statements.nodes);

        assert_eq!(
            export_names,
            vec!["default"],
            "Expected default export name"
        );
    }

    #[test]
    fn test_collect_export_names_with_named_exports() {
        use crate::thin_parser::ThinParserState;

        let source = "const foo = 1; export { foo as bar };";
        let mut parser = ThinParserState::new("test.ts".to_string(), source.to_string());
        let root = parser.parse_source_file();

        let Some(source_file) = parser.arena.get_source_file(parser.arena.get(root).unwrap()) else {
            panic!("Failed to get source file");
        };

        let export_names = collect_export_names(&parser.arena, &source_file.statements.nodes);

        assert_eq!(
            export_names,
            vec!["bar"],
            "Expected exported name from named export"
        );
    }
}
