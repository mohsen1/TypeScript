//! Export Info Map for auto-import caching.
//!
//! This module provides caching of export information for auto-import
//! functionality in code completions.

use std::collections::HashMap;
use crate::binder::{Symbol, SymbolId, SymbolFlags};

// =============================================================================
// Import/Export Kinds
// =============================================================================

/// The kind of import.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum ImportKind {
    /// Named import: `import { foo } from "mod"`
    Named,
    /// Default import: `import foo from "mod"`
    Default,
    /// Namespace import: `import * as foo from "mod"`
    Namespace,
    /// CommonJS require: `const foo = require("mod")`
    CommonJS,
}

/// The kind of export.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum ExportKind {
    /// Named export: `export { foo }`
    Named,
    /// Default export: `export default foo`
    Default,
    /// Export equals: `export = foo`
    ExportEquals,
    /// UMD export
    UMD,
    /// Module (namespace) export
    Module,
}

// =============================================================================
// Symbol Export Info
// =============================================================================

/// Information about an exported symbol.
#[derive(Debug, Clone)]
pub struct SymbolExportInfo {
    /// The symbol being exported.
    pub symbol_id: SymbolId,
    /// The module symbol.
    pub module_symbol_id: SymbolId,
    /// The module file name (if from a file).
    pub module_file_name: Option<String>,
    /// The kind of export.
    pub export_kind: ExportKind,
    /// Flags from the target symbol.
    pub target_flags: SymbolFlags,
    /// Whether this export is from package.json.
    pub is_from_package_json: bool,
}

impl SymbolExportInfo {
    pub fn new(
        symbol_id: SymbolId,
        module_symbol_id: SymbolId,
        module_file_name: Option<String>,
        export_kind: ExportKind,
        target_flags: SymbolFlags,
    ) -> Self {
        Self {
            symbol_id,
            module_symbol_id,
            module_file_name,
            export_kind,
            target_flags,
            is_from_package_json: false,
        }
    }
}

// =============================================================================
// Export Map Info Key
// =============================================================================

/// Key for looking up exports in the map.
#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub struct ExportMapInfoKey {
    /// The name of the export.
    pub name: String,
    /// The symbol ID (for disambiguation).
    pub symbol_id: SymbolId,
}

impl ExportMapInfoKey {
    pub fn new(name: String, symbol_id: SymbolId) -> Self {
        Self { name, symbol_id }
    }
}

// =============================================================================
// Export Info Map
// =============================================================================

/// Cache of export information for auto-import.
///
/// This maps symbol names to their export information, allowing quick lookup
/// of all possible imports for a given identifier.
#[derive(Debug, Default)]
pub struct ExportInfoMap {
    /// Map from export name to list of export infos.
    exports_by_name: HashMap<String, Vec<SymbolExportInfo>>,
    /// All module file names.
    module_files: Vec<String>,
    /// Whether the cache is valid.
    is_valid: bool,
    /// Version for cache invalidation.
    version: u64,
}

impl ExportInfoMap {
    /// Create a new empty export info map.
    pub fn new() -> Self {
        Self::default()
    }

    /// Clear the map and mark as invalid.
    pub fn clear(&mut self) {
        self.exports_by_name.clear();
        self.module_files.clear();
        self.is_valid = false;
    }

    /// Check if the map is valid.
    pub fn is_valid(&self) -> bool {
        self.is_valid
    }

    /// Mark the map as valid.
    pub fn set_valid(&mut self) {
        self.is_valid = true;
        self.version += 1;
    }

    /// Get the version.
    pub fn version(&self) -> u64 {
        self.version
    }

    /// Add an export to the map.
    pub fn add_export(&mut self, name: &str, info: SymbolExportInfo) {
        self.exports_by_name
            .entry(name.to_string())
            .or_default()
            .push(info);
    }

    /// Add a module file.
    pub fn add_module_file(&mut self, file_name: String) {
        if !self.module_files.contains(&file_name) {
            self.module_files.push(file_name);
        }
    }

    /// Get all exports for a name.
    pub fn get_exports_for_name(&self, name: &str) -> Option<&Vec<SymbolExportInfo>> {
        self.exports_by_name.get(name)
    }

    /// Get all export names.
    pub fn get_all_names(&self) -> impl Iterator<Item = &String> {
        self.exports_by_name.keys()
    }

    /// Iterate over all exports.
    pub fn for_each<F>(&self, mut f: F)
    where
        F: FnMut(&str, &SymbolExportInfo),
    {
        for (name, exports) in &self.exports_by_name {
            for export in exports {
                f(name, export);
            }
        }
    }

    /// Get the number of unique export names.
    pub fn len(&self) -> usize {
        self.exports_by_name.len()
    }

    /// Check if the map is empty.
    pub fn is_empty(&self) -> bool {
        self.exports_by_name.is_empty()
    }

    /// Check if a file is usable by another file for imports.
    pub fn is_usable_by_file(&self, exporting_file: &str, importing_file: &str) -> bool {
        // In a full implementation, this would check:
        // - Module resolution (can the importing file resolve the exporting file?)
        // - Package.json exports restrictions
        // - TypeScript path mappings
        // For now, always return true
        !exporting_file.is_empty() && !importing_file.is_empty()
    }

    /// Get exports that match a filter.
    pub fn get_matching_exports<F>(&self, filter: F) -> Vec<(&str, &SymbolExportInfo)>
    where
        F: Fn(&str, &SymbolExportInfo) -> bool,
    {
        let mut results = Vec::new();
        for (name, exports) in &self.exports_by_name {
            for export in exports {
                if filter(name, export) {
                    results.push((name.as_str(), export));
                }
            }
        }
        results
    }
}

// =============================================================================
// Module Specifier Resolution
// =============================================================================

/// Preference for module specifier style.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
pub enum ModuleSpecifierPreference {
    /// Use the shortest specifier.
    #[default]
    Shortest,
    /// Use relative paths when possible.
    Relative,
    /// Use non-relative paths when possible.
    NonRelative,
    /// Use project-relative paths.
    ProjectRelative,
}

/// Ending preference for module specifiers.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
pub enum ModuleSpecifierEnding {
    /// Minimal (no extension).
    #[default]
    Minimal,
    /// Include index.
    Index,
    /// Include .js extension.
    JsExtension,
}

/// Options for generating module specifiers.
#[derive(Debug, Clone, Default)]
pub struct ModuleSpecifierOptions {
    pub preference: ModuleSpecifierPreference,
    pub ending: ModuleSpecifierEnding,
}

/// Get a module specifier for importing a symbol.
pub fn get_module_specifier(
    exporting_file: &str,
    importing_file: &str,
    _options: &ModuleSpecifierOptions,
) -> String {
    // Simplified implementation - in a full version this would:
    // - Consider baseUrl and paths from tsconfig
    // - Consider node_modules resolution
    // - Consider package.json exports
    // - Apply the preference settings

    // For now, just compute a relative path
    let exporting = std::path::Path::new(exporting_file);
    let importing = std::path::Path::new(importing_file);

    if let Some(importing_dir) = importing.parent() {
        if let Ok(relative) = exporting.strip_prefix(importing_dir) {
            let mut specifier = relative.to_string_lossy().to_string();
            // Remove extension
            if let Some(pos) = specifier.rfind('.') {
                specifier = specifier[..pos].to_string();
            }
            // Ensure it starts with ./
            if !specifier.starts_with('.') && !specifier.starts_with('/') {
                specifier = format!("./{}", specifier);
            }
            return specifier;
        }
    }

    // Fallback to the original path
    exporting_file.to_string()
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_export_info_map() {
        let mut map = ExportInfoMap::new();

        let info = SymbolExportInfo::new(
            SymbolId(1),
            SymbolId(2),
            Some("module.ts".to_string()),
            ExportKind::Named,
            SymbolFlags::Function,
        );

        map.add_export("foo", info);
        map.set_valid();

        assert!(map.is_valid());
        assert_eq!(map.len(), 1);

        let exports = map.get_exports_for_name("foo");
        assert!(exports.is_some());
        assert_eq!(exports.unwrap().len(), 1);
    }

    #[test]
    fn test_module_specifier() {
        let options = ModuleSpecifierOptions::default();

        // This is a simplified test - the actual behavior would be more complex
        let specifier = get_module_specifier(
            "/src/utils/helpers.ts",
            "/src/components/App.ts",
            &options,
        );

        // The specifier should be a relative path
        assert!(!specifier.is_empty());
    }
}
