//! CompilerOptions struct matching TypeScript's compiler options
//!
//! This module defines the CompilerOptions struct and all related enums
//! that represent TypeScript's compiler configuration.

use serde::{Deserialize, Serialize};
use std::collections::HashMap;

/// Module code generation target
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize, Default)]
#[serde(rename_all = "lowercase")]
pub enum ModuleKind {
    #[default]
    None,
    #[serde(rename = "commonjs")]
    CommonJS,
    #[serde(rename = "amd")]
    AMD,
    #[serde(rename = "umd")]
    UMD,
    #[serde(rename = "system")]
    System,
    #[serde(rename = "es2015")]
    ES2015,
    #[serde(rename = "es2020")]
    ES2020,
    #[serde(rename = "es2022")]
    ES2022,
    #[serde(rename = "esnext")]
    ESNext,
    #[serde(rename = "node12")]
    Node12,
    #[serde(rename = "nodenext")]
    NodeNext,
}

/// Module resolution strategy
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize, Default)]
#[serde(rename_all = "lowercase")]
pub enum ModuleResolutionKind {
    Classic,
    #[default]
    #[serde(rename = "node")]
    NodeJs,
    #[serde(rename = "node12")]
    Node12,
    #[serde(rename = "nodenext")]
    NodeNext,
    #[serde(rename = "bundler")]
    Bundler,
}

/// JavaScript/TypeScript compilation target
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize, Default)]
#[serde(rename_all = "lowercase")]
pub enum ScriptTarget {
    ES3,
    ES5,
    #[serde(rename = "es2015")]
    ES2015,
    #[serde(rename = "es2016")]
    ES2016,
    #[serde(rename = "es2017")]
    ES2017,
    #[serde(rename = "es2018")]
    ES2018,
    #[serde(rename = "es2019")]
    ES2019,
    #[serde(rename = "es2020")]
    ES2020,
    #[serde(rename = "es2021")]
    ES2021,
    #[serde(rename = "es2022")]
    ES2022,
    #[default]
    #[serde(rename = "esnext")]
    ESNext,
}

/// JSX code generation
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize, Default)]
#[serde(rename_all = "lowercase")]
pub enum JsxEmit {
    #[default]
    None,
    Preserve,
    React,
    #[serde(rename = "react-native")]
    ReactNative,
    #[serde(rename = "react-jsx")]
    ReactJSX,
    #[serde(rename = "react-jsxdev")]
    ReactJSXDev,
}

/// How to handle imports not used as values
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize, Default)]
#[serde(rename_all = "lowercase")]
pub enum ImportsNotUsedAsValues {
    #[default]
    Remove,
    Preserve,
    Error,
}

/// New line kind for output files
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize, Default)]
#[serde(rename_all = "lowercase")]
pub enum NewLineKind {
    #[serde(rename = "crlf")]
    CarriageReturnLineFeed,
    #[default]
    #[serde(rename = "lf")]
    LineFeed,
}

/// TypeScript compiler options
#[derive(Debug, Clone, Default, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct CompilerOptions {
    // Basic Options
    /// Allow JavaScript files to be compiled
    #[serde(skip_serializing_if = "Option::is_none")]
    pub allow_js: Option<bool>,

    /// Check JavaScript files for errors
    #[serde(skip_serializing_if = "Option::is_none")]
    pub check_js: Option<bool>,

    /// Generate .d.ts declaration files
    #[serde(skip_serializing_if = "Option::is_none")]
    pub declaration: Option<bool>,

    /// Generate sourcemaps for .d.ts files
    #[serde(skip_serializing_if = "Option::is_none")]
    pub declaration_map: Option<bool>,

    /// Only emit .d.ts files, no JavaScript
    #[serde(skip_serializing_if = "Option::is_none")]
    pub emit_declaration_only: Option<bool>,

    /// Output directory for declarations
    #[serde(skip_serializing_if = "Option::is_none")]
    pub declaration_dir: Option<String>,

    /// Generate source map files
    #[serde(skip_serializing_if = "Option::is_none")]
    pub source_map: Option<bool>,

    /// Inline source maps in emitted JavaScript
    #[serde(skip_serializing_if = "Option::is_none")]
    pub inline_source_map: Option<bool>,

    /// Include source code in sourcemaps
    #[serde(skip_serializing_if = "Option::is_none")]
    pub inline_sources: Option<bool>,

    /// Output file for bundled output
    #[serde(skip_serializing_if = "Option::is_none")]
    pub out_file: Option<String>,

    /// Output directory for emitted files
    #[serde(skip_serializing_if = "Option::is_none")]
    pub out_dir: Option<String>,

    /// Root directory of input files
    #[serde(skip_serializing_if = "Option::is_none")]
    pub root_dir: Option<String>,

    /// List of root directories
    #[serde(skip_serializing_if = "Option::is_none")]
    pub root_dirs: Option<Vec<String>>,

    /// Composite project configuration
    #[serde(skip_serializing_if = "Option::is_none")]
    pub composite: Option<bool>,

    /// Enable incremental compilation
    #[serde(skip_serializing_if = "Option::is_none")]
    pub incremental: Option<bool>,

    /// File for storing incremental build info
    #[serde(skip_serializing_if = "Option::is_none")]
    pub ts_build_info_file: Option<String>,

    /// Remove comments from output
    #[serde(skip_serializing_if = "Option::is_none")]
    pub remove_comments: Option<bool>,

    /// Don't emit output files
    #[serde(skip_serializing_if = "Option::is_none")]
    pub no_emit: Option<bool>,

    /// Don't emit on error
    #[serde(skip_serializing_if = "Option::is_none")]
    pub no_emit_on_error: Option<bool>,

    /// Import helpers from tslib
    #[serde(skip_serializing_if = "Option::is_none")]
    pub import_helpers: Option<bool>,

    /// Don't emit helper functions
    #[serde(skip_serializing_if = "Option::is_none")]
    pub no_emit_helpers: Option<bool>,

    /// Downlevel iteration support
    #[serde(skip_serializing_if = "Option::is_none")]
    pub downlevel_iteration: Option<bool>,

    /// Isolate modules (each file is a module)
    #[serde(skip_serializing_if = "Option::is_none")]
    pub isolated_modules: Option<bool>,

    // Strict Type-Checking Options
    /// Enable all strict type checking options
    #[serde(skip_serializing_if = "Option::is_none")]
    pub strict: Option<bool>,

    /// Error on implicit any type
    #[serde(skip_serializing_if = "Option::is_none")]
    pub no_implicit_any: Option<bool>,

    /// Strict null checks
    #[serde(skip_serializing_if = "Option::is_none")]
    pub strict_null_checks: Option<bool>,

    /// Strict function type checking
    #[serde(skip_serializing_if = "Option::is_none")]
    pub strict_function_types: Option<bool>,

    /// Strict bind/call/apply checking
    #[serde(skip_serializing_if = "Option::is_none")]
    pub strict_bind_call_apply: Option<bool>,

    /// Strict property initialization
    #[serde(skip_serializing_if = "Option::is_none")]
    pub strict_property_initialization: Option<bool>,

    /// Error on implicit this
    #[serde(skip_serializing_if = "Option::is_none")]
    pub no_implicit_this: Option<bool>,

    /// Use unknown in catch variables
    #[serde(skip_serializing_if = "Option::is_none")]
    pub use_unknown_in_catch_variables: Option<bool>,

    /// Always emit 'use strict'
    #[serde(skip_serializing_if = "Option::is_none")]
    pub always_strict: Option<bool>,

    /// Exact optional property types
    #[serde(skip_serializing_if = "Option::is_none")]
    pub exact_optional_property_types: Option<bool>,

    /// Report unused local variables
    #[serde(skip_serializing_if = "Option::is_none")]
    pub no_unused_locals: Option<bool>,

    /// Report unused parameters
    #[serde(skip_serializing_if = "Option::is_none")]
    pub no_unused_parameters: Option<bool>,

    /// Error on implicit returns
    #[serde(skip_serializing_if = "Option::is_none")]
    pub no_implicit_returns: Option<bool>,

    /// Error on fallthrough in switch
    #[serde(skip_serializing_if = "Option::is_none")]
    pub no_fallthrough_cases_in_switch: Option<bool>,

    /// Error on unchecked indexed access
    #[serde(skip_serializing_if = "Option::is_none")]
    pub no_unchecked_indexed_access: Option<bool>,

    /// Require override keyword
    #[serde(skip_serializing_if = "Option::is_none")]
    pub no_implicit_override: Option<bool>,

    /// Disallow property access from index signatures
    #[serde(skip_serializing_if = "Option::is_none")]
    pub no_property_access_from_index_signature: Option<bool>,

    // Module Resolution Options
    /// Module code generation
    #[serde(skip_serializing_if = "Option::is_none")]
    pub module: Option<ModuleKind>,

    /// Module resolution strategy
    #[serde(skip_serializing_if = "Option::is_none")]
    pub module_resolution: Option<ModuleResolutionKind>,

    /// Base URL for module resolution
    #[serde(skip_serializing_if = "Option::is_none")]
    pub base_url: Option<String>,

    /// Path mappings for module names
    #[serde(skip_serializing_if = "Option::is_none")]
    pub paths: Option<HashMap<String, Vec<String>>>,

    /// Type declaration roots
    #[serde(skip_serializing_if = "Option::is_none")]
    pub type_roots: Option<Vec<String>>,

    /// Type packages to include
    #[serde(skip_serializing_if = "Option::is_none")]
    pub types: Option<Vec<String>>,

    /// Allow default imports from modules without default export
    #[serde(skip_serializing_if = "Option::is_none")]
    pub allow_synthetic_default_imports: Option<bool>,

    /// ES module interop
    #[serde(skip_serializing_if = "Option::is_none")]
    pub es_module_interop: Option<bool>,

    /// Preserve symlinks
    #[serde(skip_serializing_if = "Option::is_none")]
    pub preserve_symlinks: Option<bool>,

    /// Allow UMD global access
    #[serde(skip_serializing_if = "Option::is_none")]
    pub allow_umd_global_access: Option<bool>,

    /// Resolve JSON modules
    #[serde(skip_serializing_if = "Option::is_none")]
    pub resolve_json_module: Option<bool>,

    // Source Map Options
    /// Source root for source maps
    #[serde(skip_serializing_if = "Option::is_none")]
    pub source_root: Option<String>,

    /// Map root for source maps
    #[serde(skip_serializing_if = "Option::is_none")]
    pub map_root: Option<String>,

    // Additional Options
    /// Target ECMAScript version
    #[serde(skip_serializing_if = "Option::is_none")]
    pub target: Option<ScriptTarget>,

    /// Library files to include
    #[serde(skip_serializing_if = "Option::is_none")]
    pub lib: Option<Vec<String>>,

    /// JSX compilation mode
    #[serde(skip_serializing_if = "Option::is_none")]
    pub jsx: Option<JsxEmit>,

    /// JSX factory function
    #[serde(skip_serializing_if = "Option::is_none")]
    pub jsx_factory: Option<String>,

    /// JSX fragment factory
    #[serde(skip_serializing_if = "Option::is_none")]
    pub jsx_fragment_factory: Option<String>,

    /// JSX import source
    #[serde(skip_serializing_if = "Option::is_none")]
    pub jsx_import_source: Option<String>,

    /// React namespace
    #[serde(skip_serializing_if = "Option::is_none")]
    pub react_namespace: Option<String>,

    /// New line kind
    #[serde(skip_serializing_if = "Option::is_none")]
    pub new_line: Option<NewLineKind>,

    /// Skip lib check
    #[serde(skip_serializing_if = "Option::is_none")]
    pub skip_lib_check: Option<bool>,

    /// Skip default lib check
    #[serde(skip_serializing_if = "Option::is_none")]
    pub skip_default_lib_check: Option<bool>,

    /// Emit decorator metadata
    #[serde(skip_serializing_if = "Option::is_none")]
    pub emit_decorator_metadata: Option<bool>,

    /// Enable experimental decorators
    #[serde(skip_serializing_if = "Option::is_none")]
    pub experimental_decorators: Option<bool>,

    /// Allow unreachable code
    #[serde(skip_serializing_if = "Option::is_none")]
    pub allow_unreachable_code: Option<bool>,

    /// Allow unused labels
    #[serde(skip_serializing_if = "Option::is_none")]
    pub allow_unused_labels: Option<bool>,

    /// Force consistent casing in file names
    #[serde(skip_serializing_if = "Option::is_none")]
    pub force_consistent_casing_in_file_names: Option<bool>,

    /// Emit BOM
    #[serde(skip_serializing_if = "Option::is_none")]
    pub emit_bom: Option<bool>,

    /// No resolve
    #[serde(skip_serializing_if = "Option::is_none")]
    pub no_resolve: Option<bool>,

    /// No lib
    #[serde(skip_serializing_if = "Option::is_none")]
    pub no_lib: Option<bool>,

    /// No strict generic checks
    #[serde(skip_serializing_if = "Option::is_none")]
    pub no_strict_generic_checks: Option<bool>,

    /// Preserve const enums
    #[serde(skip_serializing_if = "Option::is_none")]
    pub preserve_const_enums: Option<bool>,

    /// Preserve value imports
    #[serde(skip_serializing_if = "Option::is_none")]
    pub preserve_value_imports: Option<bool>,

    /// Imports not used as values handling
    #[serde(skip_serializing_if = "Option::is_none")]
    pub imports_not_used_as_values: Option<ImportsNotUsedAsValues>,

    /// Use define for class fields
    #[serde(skip_serializing_if = "Option::is_none")]
    pub use_define_for_class_fields: Option<bool>,

    /// Max depth for node_modules
    #[serde(skip_serializing_if = "Option::is_none")]
    pub max_node_module_js_depth: Option<u32>,

    /// Disable size limit
    #[serde(skip_serializing_if = "Option::is_none")]
    pub disable_size_limit: Option<bool>,

    /// Disable source of project reference redirect
    #[serde(skip_serializing_if = "Option::is_none")]
    pub disable_source_of_project_reference_redirect: Option<bool>,

    /// Disable solution searching
    #[serde(skip_serializing_if = "Option::is_none")]
    pub disable_solution_searching: Option<bool>,

    /// Disable referenced project load
    #[serde(skip_serializing_if = "Option::is_none")]
    pub disable_referenced_project_load: Option<bool>,

    /// No implicit use strict
    #[serde(skip_serializing_if = "Option::is_none")]
    pub no_implicit_use_strict: Option<bool>,

    /// Suppress excess property errors
    #[serde(skip_serializing_if = "Option::is_none")]
    pub suppress_excess_property_errors: Option<bool>,

    /// Suppress implicit any index errors
    #[serde(skip_serializing_if = "Option::is_none")]
    pub suppress_implicit_any_index_errors: Option<bool>,

    /// Trace resolution
    #[serde(skip_serializing_if = "Option::is_none")]
    pub trace_resolution: Option<bool>,

    /// Any additional/unknown options
    #[serde(flatten)]
    pub extra: HashMap<String, serde_json::Value>,
}

impl CompilerOptions {
    /// Create a new CompilerOptions with default values
    pub fn new() -> Self {
        Self::default()
    }

    /// Merge another CompilerOptions into this one (other takes precedence)
    pub fn merge(&mut self, other: &CompilerOptions) {
        macro_rules! merge_option {
            ($field:ident) => {
                if other.$field.is_some() {
                    self.$field = other.$field.clone();
                }
            };
        }

        merge_option!(allow_js);
        merge_option!(check_js);
        merge_option!(declaration);
        merge_option!(declaration_map);
        merge_option!(emit_declaration_only);
        merge_option!(declaration_dir);
        merge_option!(source_map);
        merge_option!(inline_source_map);
        merge_option!(inline_sources);
        merge_option!(out_file);
        merge_option!(out_dir);
        merge_option!(root_dir);
        merge_option!(root_dirs);
        merge_option!(composite);
        merge_option!(incremental);
        merge_option!(ts_build_info_file);
        merge_option!(remove_comments);
        merge_option!(no_emit);
        merge_option!(no_emit_on_error);
        merge_option!(import_helpers);
        merge_option!(no_emit_helpers);
        merge_option!(downlevel_iteration);
        merge_option!(isolated_modules);
        merge_option!(strict);
        merge_option!(no_implicit_any);
        merge_option!(strict_null_checks);
        merge_option!(strict_function_types);
        merge_option!(strict_bind_call_apply);
        merge_option!(strict_property_initialization);
        merge_option!(no_implicit_this);
        merge_option!(use_unknown_in_catch_variables);
        merge_option!(always_strict);
        merge_option!(exact_optional_property_types);
        merge_option!(no_unused_locals);
        merge_option!(no_unused_parameters);
        merge_option!(no_implicit_returns);
        merge_option!(no_fallthrough_cases_in_switch);
        merge_option!(no_unchecked_indexed_access);
        merge_option!(no_implicit_override);
        merge_option!(no_property_access_from_index_signature);
        merge_option!(module);
        merge_option!(module_resolution);
        merge_option!(base_url);
        merge_option!(paths);
        merge_option!(type_roots);
        merge_option!(types);
        merge_option!(allow_synthetic_default_imports);
        merge_option!(es_module_interop);
        merge_option!(preserve_symlinks);
        merge_option!(allow_umd_global_access);
        merge_option!(resolve_json_module);
        merge_option!(source_root);
        merge_option!(map_root);
        merge_option!(target);
        merge_option!(lib);
        merge_option!(jsx);
        merge_option!(jsx_factory);
        merge_option!(jsx_fragment_factory);
        merge_option!(jsx_import_source);
        merge_option!(react_namespace);
        merge_option!(new_line);
        merge_option!(skip_lib_check);
        merge_option!(skip_default_lib_check);
        merge_option!(emit_decorator_metadata);
        merge_option!(experimental_decorators);
        merge_option!(allow_unreachable_code);
        merge_option!(allow_unused_labels);
        merge_option!(force_consistent_casing_in_file_names);
        merge_option!(emit_bom);
        merge_option!(no_resolve);
        merge_option!(no_lib);
        merge_option!(no_strict_generic_checks);
        merge_option!(preserve_const_enums);
        merge_option!(preserve_value_imports);
        merge_option!(imports_not_used_as_values);
        merge_option!(use_define_for_class_fields);
        merge_option!(max_node_module_js_depth);
        merge_option!(disable_size_limit);
        merge_option!(disable_source_of_project_reference_redirect);
        merge_option!(disable_solution_searching);
        merge_option!(disable_referenced_project_load);
        merge_option!(no_implicit_use_strict);
        merge_option!(suppress_excess_property_errors);
        merge_option!(suppress_implicit_any_index_errors);
        merge_option!(trace_resolution);

        // Merge extra options
        for (key, value) in &other.extra {
            self.extra.insert(key.clone(), value.clone());
        }
    }

    /// Apply strict mode defaults
    /// When strict is true, it implies several other options
    pub fn apply_strict_defaults(&mut self) {
        if self.strict == Some(true) {
            // Only set these if they haven't been explicitly set
            if self.no_implicit_any.is_none() {
                self.no_implicit_any = Some(true);
            }
            if self.strict_null_checks.is_none() {
                self.strict_null_checks = Some(true);
            }
            if self.strict_function_types.is_none() {
                self.strict_function_types = Some(true);
            }
            if self.strict_bind_call_apply.is_none() {
                self.strict_bind_call_apply = Some(true);
            }
            if self.strict_property_initialization.is_none() {
                self.strict_property_initialization = Some(true);
            }
            if self.no_implicit_this.is_none() {
                self.no_implicit_this = Some(true);
            }
            if self.use_unknown_in_catch_variables.is_none() {
                self.use_unknown_in_catch_variables = Some(true);
            }
            if self.always_strict.is_none() {
                self.always_strict = Some(true);
            }
        }
    }

    /// Check if strict mode is effectively enabled
    pub fn is_strict(&self) -> bool {
        self.strict == Some(true)
    }

    /// Get the effective noImplicitAny setting
    pub fn effective_no_implicit_any(&self) -> bool {
        self.no_implicit_any.unwrap_or_else(|| self.strict == Some(true))
    }

    /// Get the effective strictNullChecks setting
    pub fn effective_strict_null_checks(&self) -> bool {
        self.strict_null_checks.unwrap_or_else(|| self.strict == Some(true))
    }

    /// Get the effective strictFunctionTypes setting
    pub fn effective_strict_function_types(&self) -> bool {
        self.strict_function_types.unwrap_or_else(|| self.strict == Some(true))
    }

    /// Get the effective strictBindCallApply setting
    pub fn effective_strict_bind_call_apply(&self) -> bool {
        self.strict_bind_call_apply.unwrap_or_else(|| self.strict == Some(true))
    }

    /// Get the effective strictPropertyInitialization setting
    pub fn effective_strict_property_initialization(&self) -> bool {
        self.strict_property_initialization.unwrap_or_else(|| self.strict == Some(true))
    }

    /// Get the effective noImplicitThis setting
    pub fn effective_no_implicit_this(&self) -> bool {
        self.no_implicit_this.unwrap_or_else(|| self.strict == Some(true))
    }

    /// Get the effective alwaysStrict setting
    pub fn effective_always_strict(&self) -> bool {
        self.always_strict.unwrap_or_else(|| self.strict == Some(true))
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_default_compiler_options() {
        let opts = CompilerOptions::new();
        assert!(opts.strict.is_none());
        assert!(opts.target.is_none());
        assert!(opts.module.is_none());
    }

    #[test]
    fn test_strict_mode_defaults() {
        let mut opts = CompilerOptions::new();
        opts.strict = Some(true);
        opts.apply_strict_defaults();

        assert_eq!(opts.no_implicit_any, Some(true));
        assert_eq!(opts.strict_null_checks, Some(true));
        assert_eq!(opts.strict_function_types, Some(true));
        assert_eq!(opts.strict_bind_call_apply, Some(true));
        assert_eq!(opts.strict_property_initialization, Some(true));
        assert_eq!(opts.no_implicit_this, Some(true));
        assert_eq!(opts.always_strict, Some(true));
    }

    #[test]
    fn test_strict_mode_explicit_override() {
        let mut opts = CompilerOptions::new();
        opts.strict = Some(true);
        opts.no_implicit_any = Some(false); // Explicitly set to false
        opts.apply_strict_defaults();

        // no_implicit_any should remain false because it was explicitly set
        assert_eq!(opts.no_implicit_any, Some(false));
        // But strict_null_checks should be true from strict
        assert_eq!(opts.strict_null_checks, Some(true));
    }

    #[test]
    fn test_merge_options() {
        let mut base = CompilerOptions::new();
        base.target = Some(ScriptTarget::ES2015);
        base.strict = Some(true);
        base.out_dir = Some("dist".to_string());

        let mut child = CompilerOptions::new();
        child.target = Some(ScriptTarget::ES2020);
        child.source_map = Some(true);

        base.merge(&child);

        // Child value should override base
        assert_eq!(base.target, Some(ScriptTarget::ES2020));
        // Base value should remain
        assert_eq!(base.strict, Some(true));
        assert_eq!(base.out_dir, Some("dist".to_string()));
        // Child value should be added
        assert_eq!(base.source_map, Some(true));
    }

    #[test]
    fn test_effective_strict_settings() {
        let mut opts = CompilerOptions::new();

        // Without strict mode
        assert!(!opts.effective_no_implicit_any());
        assert!(!opts.effective_strict_null_checks());

        // With strict mode
        opts.strict = Some(true);
        assert!(opts.effective_no_implicit_any());
        assert!(opts.effective_strict_null_checks());

        // With explicit override
        opts.no_implicit_any = Some(false);
        assert!(!opts.effective_no_implicit_any());
        // strict_null_checks should still be true from strict
        assert!(opts.effective_strict_null_checks());
    }

    #[test]
    fn test_serialize_deserialize() {
        let mut opts = CompilerOptions::new();
        opts.target = Some(ScriptTarget::ES2020);
        opts.module = Some(ModuleKind::ESNext);
        opts.strict = Some(true);
        opts.out_dir = Some("dist".to_string());

        let json = serde_json::to_string(&opts).unwrap();
        let parsed: CompilerOptions = serde_json::from_str(&json).unwrap();

        assert_eq!(parsed.target, Some(ScriptTarget::ES2020));
        assert_eq!(parsed.module, Some(ModuleKind::ESNext));
        assert_eq!(parsed.strict, Some(true));
        assert_eq!(parsed.out_dir, Some("dist".to_string()));
    }
}
