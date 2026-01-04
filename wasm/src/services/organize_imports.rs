//! Organize Imports implementation.
//!
//! This module provides functionality for organizing imports including:
//! - Sorting imports alphabetically
//! - Removing unused imports
//! - Grouping imports by type
//! - Combining imports from the same module

use std::collections::{HashMap, HashSet};
use crate::binder::{Symbol, SymbolFlags};
use crate::checker::CheckerState;
use crate::parser::ast::{Node, NodeId};
use crate::parser::arena::NodeArena;
use crate::scanner::SyntaxKind;

use super::code_fix_provider::FileTextChanges;
use super::text_span::TextSpan;
use super::text_changes::TextChange;

// =============================================================================
// Organize Imports Options
// =============================================================================

/// Mode for organizing imports.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
pub enum OrganizeImportsMode {
    /// Remove unused imports and sort.
    #[default]
    All,
    /// Only sort imports.
    SortOnly,
    /// Only remove unused imports.
    RemoveUnused,
}

/// Options for organizing imports.
#[derive(Debug, Clone, Default)]
pub struct OrganizeImportsOptions {
    /// The mode for organizing.
    pub mode: OrganizeImportsMode,
    /// Whether to skip destructive actions.
    pub skip_destructive_code_actions: bool,
    /// Whether to use type-only imports where possible.
    pub prefer_type_only_imports: bool,
}

/// Grouping preferences for import organization.
#[derive(Debug, Clone, Default)]
pub struct ImportGroupPreferences {
    /// The order of import groups.
    pub group_order: Vec<ImportGroupKind>,
    /// Whether to separate groups with blank lines.
    pub separate_groups: bool,
}

/// The kind of import group.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ImportGroupKind {
    /// Built-in modules (e.g., 'fs', 'path').
    Builtin,
    /// External modules (from node_modules).
    External,
    /// Internal modules (from the project).
    Internal,
    /// Parent imports (../).
    Parent,
    /// Sibling imports (./).
    Sibling,
    /// Index imports (./index).
    Index,
    /// Type-only imports.
    Type,
}

// =============================================================================
// Organize Imports Context
// =============================================================================

/// Context for organize imports operations.
pub struct OrganizeImportsContext<'a> {
    pub arena: &'a NodeArena,
    pub checker: &'a CheckerState,
    pub source_text: &'a str,
    pub file_name: &'a str,
    pub options: OrganizeImportsOptions,
    pub preferences: ImportGroupPreferences,
}

// =============================================================================
// Organize Imports
// =============================================================================

/// Organize imports in a file.
pub fn organize_imports(
    ctx: &OrganizeImportsContext,
) -> Vec<FileTextChanges> {
    let mut changes = Vec::new();

    // Get all import declarations
    let imports = collect_import_declarations(ctx);

    if imports.is_empty() {
        return changes;
    }

    // Parse imports into structured format
    let mut parsed_imports = parse_imports(ctx, &imports);

    // Remove unused if requested
    if ctx.options.mode != OrganizeImportsMode::SortOnly {
        remove_unused_imports(ctx, &mut parsed_imports);
    }

    // Sort if requested
    if ctx.options.mode != OrganizeImportsMode::RemoveUnused {
        sort_imports(&mut parsed_imports, &ctx.preferences);
    }

    // Group imports
    let grouped = group_imports(parsed_imports, &ctx.preferences);

    // Generate new import text
    let new_import_text = generate_import_text(&grouped, ctx);

    // Calculate the span to replace
    let import_span = get_imports_span(ctx, &imports);

    // Create the change
    if !new_import_text.is_empty() || !imports.is_empty() {
        changes.push(FileTextChanges::new(
            ctx.file_name.to_string(),
            vec![TextChange::new(import_span, new_import_text)],
        ));
    }

    changes
}

/// Get code actions for organizing imports.
pub fn get_organize_imports_actions(
    ctx: &OrganizeImportsContext,
) -> Vec<OrganizeImportsAction> {
    let mut actions = Vec::new();

    // Check if there are any imports to organize
    let imports = collect_import_declarations(ctx);

    if imports.is_empty() {
        return actions;
    }

    // Add "Organize Imports" action
    actions.push(OrganizeImportsAction {
        title: "Organize Imports".to_string(),
        kind: OrganizeImportsActionKind::OrganizeImports,
    });

    // Check for unused imports
    let unused = find_unused_imports(ctx, &imports);
    if !unused.is_empty() {
        actions.push(OrganizeImportsAction {
            title: format!("Remove {} unused import(s)", unused.len()),
            kind: OrganizeImportsActionKind::RemoveUnused,
        });
    }

    // Add "Sort Imports" action
    actions.push(OrganizeImportsAction {
        title: "Sort Imports".to_string(),
        kind: OrganizeImportsActionKind::SortOnly,
    });

    actions
}

/// An organize imports action.
#[derive(Debug, Clone)]
pub struct OrganizeImportsAction {
    pub title: String,
    pub kind: OrganizeImportsActionKind,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum OrganizeImportsActionKind {
    OrganizeImports,
    RemoveUnused,
    SortOnly,
}

// =============================================================================
// Import Parsing
// =============================================================================

/// A parsed import declaration.
#[derive(Debug, Clone)]
struct ParsedImport {
    /// The node ID of the import declaration.
    node_id: NodeId,
    /// The module specifier.
    module_specifier: String,
    /// Default import name.
    default_import: Option<String>,
    /// Namespace import name.
    namespace_import: Option<String>,
    /// Named imports.
    named_imports: Vec<NamedImport>,
    /// Whether this is a type-only import.
    is_type_only: bool,
    /// The original text.
    original_text: String,
    /// Whether any part is used.
    is_used: bool,
}

#[derive(Debug, Clone)]
struct NamedImport {
    name: String,
    alias: Option<String>,
    is_type_only: bool,
    is_used: bool,
}

fn collect_import_declarations(ctx: &OrganizeImportsContext) -> Vec<NodeId> {
    let mut imports = Vec::new();

    let root = match ctx.arena.root() {
        Some(r) => r,
        None => return imports,
    };

    let source_file = match ctx.arena.get(root) {
        Some(sf) => sf,
        None => return imports,
    };

    for child_id in source_file.children() {
        let child = match ctx.arena.get(child_id) {
            Some(c) => c,
            None => continue,
        };

        if child.kind() == SyntaxKind::ImportDeclaration {
            imports.push(child_id);
        }
    }

    imports
}

fn parse_imports(ctx: &OrganizeImportsContext, imports: &[NodeId]) -> Vec<ParsedImport> {
    let mut parsed = Vec::new();

    for &import_id in imports {
        if let Some(import) = parse_single_import(ctx, import_id) {
            parsed.push(import);
        }
    }

    parsed
}

fn parse_single_import(
    ctx: &OrganizeImportsContext,
    import_id: NodeId,
) -> Option<ParsedImport> {
    let import_node = ctx.arena.get(import_id)?;
    let span = TextSpan::from_bounds(import_node.pos(), import_node.end());
    let start = span.start as usize;
    let end = start + span.length as usize;

    let original_text = if end <= ctx.source_text.len() {
        ctx.source_text[start..end].to_string()
    } else {
        String::new()
    };

    // In a full implementation, we would parse the import structure
    // from the AST nodes. For now, create a simplified version.

    // Extract module specifier from original text
    let module_specifier = extract_module_specifier(&original_text)?;

    Some(ParsedImport {
        node_id: import_id,
        module_specifier,
        default_import: None,
        namespace_import: None,
        named_imports: Vec::new(),
        is_type_only: original_text.contains("import type"),
        original_text,
        is_used: true, // Assume used by default
    })
}

fn extract_module_specifier(import_text: &str) -> Option<String> {
    // Find the string literal
    let start = import_text.find(|c| c == '"' || c == '\'')?;
    let quote = import_text.chars().nth(start)?;
    let rest = &import_text[start + 1..];
    let end = rest.find(quote)?;

    Some(rest[..end].to_string())
}

// =============================================================================
// Unused Import Detection
// =============================================================================

fn find_unused_imports(
    ctx: &OrganizeImportsContext,
    imports: &[NodeId],
) -> Vec<NodeId> {
    let mut unused = Vec::new();

    for &import_id in imports {
        if !is_import_used(ctx, import_id) {
            unused.push(import_id);
        }
    }

    unused
}

fn is_import_used(ctx: &OrganizeImportsContext, import_id: NodeId) -> bool {
    // In a full implementation, we would:
    // 1. Get all symbols imported by this declaration
    // 2. Check if each symbol is referenced elsewhere in the file
    // 3. Return true if any symbol is used

    // For now, assume all imports are used
    true
}

fn remove_unused_imports(
    ctx: &OrganizeImportsContext,
    imports: &mut Vec<ParsedImport>,
) {
    imports.retain(|import| import.is_used);

    // Also filter named imports within each import
    for import in imports.iter_mut() {
        import.named_imports.retain(|named| named.is_used);
    }
}

// =============================================================================
// Import Sorting
// =============================================================================

fn sort_imports(
    imports: &mut Vec<ParsedImport>,
    preferences: &ImportGroupPreferences,
) {
    // Sort by module specifier
    imports.sort_by(|a, b| {
        compare_module_specifiers(&a.module_specifier, &b.module_specifier)
    });

    // Sort named imports within each import
    for import in imports.iter_mut() {
        import.named_imports.sort_by(|a, b| a.name.cmp(&b.name));
    }
}

fn compare_module_specifiers(a: &str, b: &str) -> std::cmp::Ordering {
    // Handle special prefixes
    let a_kind = get_specifier_kind(a);
    let b_kind = get_specifier_kind(b);

    if a_kind != b_kind {
        return a_kind.cmp(&b_kind);
    }

    // Then sort alphabetically (case-insensitive)
    a.to_lowercase().cmp(&b.to_lowercase())
}

fn get_specifier_kind(specifier: &str) -> u8 {
    if specifier.starts_with("./") {
        3 // Sibling
    } else if specifier.starts_with("../") {
        2 // Parent
    } else if specifier.starts_with('@') {
        1 // Scoped external
    } else {
        0 // External or builtin
    }
}

// =============================================================================
// Import Grouping
// =============================================================================

fn group_imports(
    imports: Vec<ParsedImport>,
    preferences: &ImportGroupPreferences,
) -> Vec<Vec<ParsedImport>> {
    if preferences.group_order.is_empty() {
        return vec![imports];
    }

    let mut groups: HashMap<ImportGroupKind, Vec<ParsedImport>> = HashMap::new();

    for import in imports {
        let kind = categorize_import(&import.module_specifier);
        groups.entry(kind).or_default().push(import);
    }

    // Build result in order
    let mut result = Vec::new();

    for group_kind in &preferences.group_order {
        if let Some(group) = groups.remove(group_kind) {
            if !group.is_empty() {
                result.push(group);
            }
        }
    }

    // Add any remaining imports
    for (_, group) in groups {
        if !group.is_empty() {
            result.push(group);
        }
    }

    result
}

fn categorize_import(specifier: &str) -> ImportGroupKind {
    if specifier.starts_with("./index") || specifier == "." {
        ImportGroupKind::Index
    } else if specifier.starts_with("./") {
        ImportGroupKind::Sibling
    } else if specifier.starts_with("../") {
        ImportGroupKind::Parent
    } else if is_builtin_module(specifier) {
        ImportGroupKind::Builtin
    } else {
        ImportGroupKind::External
    }
}

fn is_builtin_module(specifier: &str) -> bool {
    const BUILTINS: &[&str] = &[
        "assert", "buffer", "child_process", "cluster", "console",
        "constants", "crypto", "dgram", "dns", "domain", "events",
        "fs", "http", "https", "module", "net", "os", "path",
        "process", "punycode", "querystring", "readline", "repl",
        "stream", "string_decoder", "timers", "tls", "tty", "url",
        "util", "v8", "vm", "zlib",
    ];

    // Check for node: prefix
    if specifier.starts_with("node:") {
        return true;
    }

    BUILTINS.contains(&specifier)
}

// =============================================================================
// Import Text Generation
// =============================================================================

fn generate_import_text(
    groups: &[Vec<ParsedImport>],
    ctx: &OrganizeImportsContext,
) -> String {
    let mut result = String::new();

    for (i, group) in groups.iter().enumerate() {
        if i > 0 && ctx.preferences.separate_groups {
            result.push('\n');
        }

        for import in group {
            result.push_str(&import.original_text);
            result.push('\n');
        }
    }

    // Remove trailing newline if present
    if result.ends_with('\n') {
        result.pop();
    }

    result
}

fn get_imports_span(ctx: &OrganizeImportsContext, imports: &[NodeId]) -> TextSpan {
    if imports.is_empty() {
        return TextSpan::new(0, 0);
    }

    let first = ctx.arena.get(imports[0]);
    let last = ctx.arena.get(*imports.last().unwrap());

    match (first, last) {
        (Some(f), Some(l)) => TextSpan::from_bounds(f.pos(), l.end()),
        _ => TextSpan::new(0, 0),
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_extract_module_specifier() {
        assert_eq!(
            extract_module_specifier("import { foo } from 'bar';"),
            Some("bar".to_string())
        );
        assert_eq!(
            extract_module_specifier("import foo from \"bar\";"),
            Some("bar".to_string())
        );
    }

    #[test]
    fn test_get_specifier_kind() {
        assert_eq!(get_specifier_kind("react"), 0);
        assert_eq!(get_specifier_kind("@angular/core"), 1);
        assert_eq!(get_specifier_kind("../utils"), 2);
        assert_eq!(get_specifier_kind("./helper"), 3);
    }

    #[test]
    fn test_categorize_import() {
        assert_eq!(categorize_import("fs"), ImportGroupKind::Builtin);
        assert_eq!(categorize_import("react"), ImportGroupKind::External);
        assert_eq!(categorize_import("../utils"), ImportGroupKind::Parent);
        assert_eq!(categorize_import("./helper"), ImportGroupKind::Sibling);
        assert_eq!(categorize_import("./index"), ImportGroupKind::Index);
    }

    #[test]
    fn test_is_builtin_module() {
        assert!(is_builtin_module("fs"));
        assert!(is_builtin_module("path"));
        assert!(is_builtin_module("node:fs"));
        assert!(!is_builtin_module("react"));
        assert!(!is_builtin_module("lodash"));
    }

    #[test]
    fn test_compare_module_specifiers() {
        use std::cmp::Ordering;

        assert_eq!(
            compare_module_specifiers("react", "lodash"),
            Ordering::Greater
        );
        assert_eq!(
            compare_module_specifiers("./a", "./b"),
            Ordering::Less
        );
        assert_eq!(
            compare_module_specifiers("react", "./local"),
            Ordering::Less
        );
    }
}
