//! Compiler options definitions matching TypeScript's CompilerOptions.
//!
//! This module defines all compiler options supported by TSC.

use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::path::PathBuf;

/// ECMAScript target version
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Default, Serialize, Deserialize)]
#[serde(rename_all = "lowercase")]
pub enum ScriptTarget {
    ES3,
    ES5,
    ES2015,
    ES2016,
    ES2017,
    ES2018,
    ES2019,
    ES2020,
    ES2021,
    ES2022,
    ES2023,
    ES2024,
    #[default]
    ESNext,
}

impl ScriptTarget {
    pub fn from_str(s: &str) -> Option<Self> {
        match s.to_lowercase().as_str() {
            "es3" => Some(Self::ES3),
            "es5" => Some(Self::ES5),
            "es6" | "es2015" => Some(Self::ES2015),
            "es2016" => Some(Self::ES2016),
            "es2017" => Some(Self::ES2017),
            "es2018" => Some(Self::ES2018),
            "es2019" => Some(Self::ES2019),
            "es2020" => Some(Self::ES2020),
            "es2021" => Some(Self::ES2021),
            "es2022" => Some(Self::ES2022),
            "es2023" => Some(Self::ES2023),
            "es2024" => Some(Self::ES2024),
            "esnext" => Some(Self::ESNext),
            _ => None,
        }
    }
}

/// Module system
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Default, Serialize, Deserialize)]
#[serde(rename_all = "lowercase")]
pub enum ModuleKind {
    None,
    CommonJS,
    AMD,
    UMD,
    System,
    ES2015,
    ES2020,
    ES2022,
    #[default]
    ESNext,
    Node16,
    NodeNext,
    Preserve,
}

impl ModuleKind {
    pub fn from_str(s: &str) -> Option<Self> {
        match s.to_lowercase().as_str() {
            "none" => Some(Self::None),
            "commonjs" => Some(Self::CommonJS),
            "amd" => Some(Self::AMD),
            "umd" => Some(Self::UMD),
            "system" => Some(Self::System),
            "es6" | "es2015" => Some(Self::ES2015),
            "es2020" => Some(Self::ES2020),
            "es2022" => Some(Self::ES2022),
            "esnext" => Some(Self::ESNext),
            "node16" => Some(Self::Node16),
            "nodenext" => Some(Self::NodeNext),
            "preserve" => Some(Self::Preserve),
            _ => None,
        }
    }
}

/// Module resolution strategy
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Default, Serialize, Deserialize)]
#[serde(rename_all = "lowercase")]
pub enum ModuleResolutionKind {
    Classic,
    #[default]
    Node,
    Node16,
    NodeNext,
    Bundler,
}

impl ModuleResolutionKind {
    pub fn from_str(s: &str) -> Option<Self> {
        match s.to_lowercase().as_str() {
            "classic" => Some(Self::Classic),
            "node" | "node10" => Some(Self::Node),
            "node16" => Some(Self::Node16),
            "nodenext" => Some(Self::NodeNext),
            "bundler" => Some(Self::Bundler),
            _ => None,
        }
    }
}

/// JSX emit mode
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Default, Serialize, Deserialize)]
#[serde(rename_all = "kebab-case")]
pub enum JsxEmit {
    #[default]
    None,
    Preserve,
    React,
    ReactNative,
    ReactJsx,
    ReactJsxdev,
}

impl JsxEmit {
    pub fn from_str(s: &str) -> Option<Self> {
        match s.to_lowercase().as_str() {
            "none" => Some(Self::None),
            "preserve" => Some(Self::Preserve),
            "react" => Some(Self::React),
            "react-native" => Some(Self::ReactNative),
            "react-jsx" => Some(Self::ReactJsx),
            "react-jsxdev" => Some(Self::ReactJsxdev),
            _ => None,
        }
    }
}

/// Newline kind for emit
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Default, Serialize, Deserialize)]
#[serde(rename_all = "lowercase")]
pub enum NewLineKind {
    #[default]
    CarriageReturnLineFeed,
    LineFeed,
}

/// Import/export mode for modules
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Default, Serialize, Deserialize)]
#[serde(rename_all = "lowercase")]
pub enum ImportsNotUsedAsValues {
    #[default]
    Remove,
    Preserve,
    Error,
}

/// Module detection mode
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Default, Serialize, Deserialize)]
#[serde(rename_all = "lowercase")]
pub enum ModuleDetectionKind {
    Legacy,
    #[default]
    Auto,
    Force,
}

/// Compiler options struct matching TSC's CompilerOptions interface
#[derive(Debug, Clone, Default, Serialize, Deserialize)]
#[serde(rename_all = "camelCase", default)]
pub struct CompilerOptions {
    // Basic Options
    #[serde(skip_serializing_if = "Option::is_none")]
    pub target: Option<ScriptTarget>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub module: Option<ModuleKind>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub lib: Option<Vec<String>>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub allow_js: Option<bool>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub check_js: Option<bool>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub jsx: Option<JsxEmit>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub declaration: Option<bool>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub declaration_map: Option<bool>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub source_map: Option<bool>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub out_file: Option<PathBuf>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub out_dir: Option<PathBuf>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub root_dir: Option<PathBuf>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub composite: Option<bool>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub ts_build_info_file: Option<PathBuf>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub remove_comments: Option<bool>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub no_emit: Option<bool>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub import_helpers: Option<bool>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub downlevel_iteration: Option<bool>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub isolated_modules: Option<bool>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub verbatim_module_syntax: Option<bool>,

    // Strict Type-Checking Options
    #[serde(skip_serializing_if = "Option::is_none")]
    pub strict: Option<bool>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub no_implicit_any: Option<bool>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub strict_null_checks: Option<bool>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub strict_function_types: Option<bool>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub strict_bind_call_apply: Option<bool>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub strict_property_initialization: Option<bool>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub no_implicit_this: Option<bool>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub use_unknown_in_catch_variables: Option<bool>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub always_strict: Option<bool>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub exact_optional_property_types: Option<bool>,

    // Additional Checks
    #[serde(skip_serializing_if = "Option::is_none")]
    pub no_unused_locals: Option<bool>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub no_unused_parameters: Option<bool>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub no_implicit_returns: Option<bool>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub no_fallthrough_cases_in_switch: Option<bool>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub no_unchecked_indexed_access: Option<bool>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub no_implicit_override: Option<bool>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub no_property_access_from_index_signature: Option<bool>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub allow_unreachable_code: Option<bool>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub allow_unused_labels: Option<bool>,

    // Module Resolution Options
    #[serde(skip_serializing_if = "Option::is_none")]
    pub module_resolution: Option<ModuleResolutionKind>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub base_url: Option<PathBuf>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub paths: Option<HashMap<String, Vec<String>>>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub root_dirs: Option<Vec<PathBuf>>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub type_roots: Option<Vec<PathBuf>>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub types: Option<Vec<String>>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub allow_synthetic_default_imports: Option<bool>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub es_module_interop: Option<bool>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub preserve_symlinks: Option<bool>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub allow_importing_ts_extensions: Option<bool>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub resolve_package_json_exports: Option<bool>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub resolve_package_json_imports: Option<bool>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub resolve_json_module: Option<bool>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub custom_conditions: Option<Vec<String>>,

    // Source Map Options
    #[serde(skip_serializing_if = "Option::is_none")]
    pub source_root: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub map_root: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub inline_source_map: Option<bool>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub inline_sources: Option<bool>,

    // Experimental Options
    #[serde(skip_serializing_if = "Option::is_none")]
    pub experimental_decorators: Option<bool>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub emit_decorator_metadata: Option<bool>,

    // Advanced Options
    #[serde(skip_serializing_if = "Option::is_none")]
    pub declaration_dir: Option<PathBuf>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub skip_lib_check: Option<bool>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub skip_default_lib_check: Option<bool>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub force_consistent_casing_in_file_names: Option<bool>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub charset: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub emit_bom: Option<bool>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub new_line: Option<NewLineKind>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub no_error_truncation: Option<bool>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub no_lib: Option<bool>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub no_resolve: Option<bool>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub no_strict_generic_checks: Option<bool>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub preserve_const_enums: Option<bool>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub strip_internal: Option<bool>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub disable_size_limit: Option<bool>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub disable_source_of_project_reference_redirect: Option<bool>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub disable_solution_searching: Option<bool>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub disable_referenced_project_load: Option<bool>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub no_emit_helpers: Option<bool>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub no_emit_on_error: Option<bool>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub preserve_value_imports: Option<bool>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub imports_not_used_as_values: Option<ImportsNotUsedAsValues>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub module_detection: Option<ModuleDetectionKind>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub use_define_for_class_fields: Option<bool>,

    // JSX options
    #[serde(skip_serializing_if = "Option::is_none")]
    pub jsx_factory: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub jsx_fragment_factory: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub jsx_import_source: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub react_namespace: Option<String>,

    // Emit options
    #[serde(skip_serializing_if = "Option::is_none")]
    pub emit_declaration_only: Option<bool>,

    // Project references
    #[serde(skip_serializing_if = "Option::is_none")]
    pub incremental: Option<bool>,

    // Plugin support
    #[serde(skip_serializing_if = "Option::is_none")]
    pub plugins: Option<Vec<PluginImport>>,
}

/// Plugin import configuration
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PluginImport {
    pub name: String,
    #[serde(flatten)]
    pub options: Option<serde_json::Value>,
}

impl CompilerOptions {
    /// Create new compiler options with defaults
    pub fn new() -> Self {
        Self::default()
    }

    /// Apply strict mode settings
    pub fn apply_strict_mode(&mut self) {
        if self.strict == Some(true) {
            // Only set if not explicitly specified
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

    /// Merge another set of options into this one (other takes precedence)
    pub fn merge(&mut self, other: &CompilerOptions) {
        macro_rules! merge_field {
            ($field:ident) => {
                if other.$field.is_some() {
                    self.$field = other.$field.clone();
                }
            };
        }

        // Basic options
        merge_field!(target);
        merge_field!(module);
        merge_field!(lib);
        merge_field!(allow_js);
        merge_field!(check_js);
        merge_field!(jsx);
        merge_field!(declaration);
        merge_field!(declaration_map);
        merge_field!(source_map);
        merge_field!(out_file);
        merge_field!(out_dir);
        merge_field!(root_dir);
        merge_field!(composite);
        merge_field!(ts_build_info_file);
        merge_field!(remove_comments);
        merge_field!(no_emit);
        merge_field!(import_helpers);
        merge_field!(downlevel_iteration);
        merge_field!(isolated_modules);
        merge_field!(verbatim_module_syntax);

        // Strict options
        merge_field!(strict);
        merge_field!(no_implicit_any);
        merge_field!(strict_null_checks);
        merge_field!(strict_function_types);
        merge_field!(strict_bind_call_apply);
        merge_field!(strict_property_initialization);
        merge_field!(no_implicit_this);
        merge_field!(use_unknown_in_catch_variables);
        merge_field!(always_strict);
        merge_field!(exact_optional_property_types);

        // Additional checks
        merge_field!(no_unused_locals);
        merge_field!(no_unused_parameters);
        merge_field!(no_implicit_returns);
        merge_field!(no_fallthrough_cases_in_switch);
        merge_field!(no_unchecked_indexed_access);
        merge_field!(no_implicit_override);
        merge_field!(no_property_access_from_index_signature);
        merge_field!(allow_unreachable_code);
        merge_field!(allow_unused_labels);

        // Module resolution
        merge_field!(module_resolution);
        merge_field!(base_url);
        merge_field!(paths);
        merge_field!(root_dirs);
        merge_field!(type_roots);
        merge_field!(types);
        merge_field!(allow_synthetic_default_imports);
        merge_field!(es_module_interop);
        merge_field!(preserve_symlinks);
        merge_field!(allow_importing_ts_extensions);
        merge_field!(resolve_package_json_exports);
        merge_field!(resolve_package_json_imports);
        merge_field!(resolve_json_module);
        merge_field!(custom_conditions);

        // Source map options
        merge_field!(source_root);
        merge_field!(map_root);
        merge_field!(inline_source_map);
        merge_field!(inline_sources);

        // Experimental
        merge_field!(experimental_decorators);
        merge_field!(emit_decorator_metadata);

        // Advanced
        merge_field!(declaration_dir);
        merge_field!(skip_lib_check);
        merge_field!(skip_default_lib_check);
        merge_field!(force_consistent_casing_in_file_names);
        merge_field!(charset);
        merge_field!(emit_bom);
        merge_field!(new_line);
        merge_field!(no_error_truncation);
        merge_field!(no_lib);
        merge_field!(no_resolve);
        merge_field!(no_strict_generic_checks);
        merge_field!(preserve_const_enums);
        merge_field!(strip_internal);
        merge_field!(disable_size_limit);
        merge_field!(disable_source_of_project_reference_redirect);
        merge_field!(disable_solution_searching);
        merge_field!(disable_referenced_project_load);
        merge_field!(no_emit_helpers);
        merge_field!(no_emit_on_error);
        merge_field!(preserve_value_imports);
        merge_field!(imports_not_used_as_values);
        merge_field!(module_detection);
        merge_field!(use_define_for_class_fields);

        // JSX
        merge_field!(jsx_factory);
        merge_field!(jsx_fragment_factory);
        merge_field!(jsx_import_source);
        merge_field!(react_namespace);

        // Emit
        merge_field!(emit_declaration_only);

        // Project
        merge_field!(incremental);
        merge_field!(plugins);
    }

    /// Check if this is effectively strict mode
    pub fn is_strict(&self) -> bool {
        self.strict == Some(true)
    }

    /// Get the effective target
    pub fn effective_target(&self) -> ScriptTarget {
        self.target.unwrap_or(ScriptTarget::ES5)
    }

    /// Get the effective module
    pub fn effective_module(&self) -> ModuleKind {
        self.module.unwrap_or_else(|| {
            match self.effective_target() {
                ScriptTarget::ES3 | ScriptTarget::ES5 => ModuleKind::CommonJS,
                _ => ModuleKind::ES2015,
            }
        })
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_script_target_from_str() {
        assert_eq!(ScriptTarget::from_str("es5"), Some(ScriptTarget::ES5));
        assert_eq!(ScriptTarget::from_str("ES2020"), Some(ScriptTarget::ES2020));
        assert_eq!(ScriptTarget::from_str("es6"), Some(ScriptTarget::ES2015));
        assert_eq!(ScriptTarget::from_str("invalid"), None);
    }

    #[test]
    fn test_module_kind_from_str() {
        assert_eq!(ModuleKind::from_str("commonjs"), Some(ModuleKind::CommonJS));
        assert_eq!(ModuleKind::from_str("NodeNext"), Some(ModuleKind::NodeNext));
        assert_eq!(ModuleKind::from_str("es6"), Some(ModuleKind::ES2015));
    }

    #[test]
    fn test_strict_mode_application() {
        let mut opts = CompilerOptions::new();
        opts.strict = Some(true);
        opts.apply_strict_mode();

        assert_eq!(opts.no_implicit_any, Some(true));
        assert_eq!(opts.strict_null_checks, Some(true));
        assert_eq!(opts.strict_function_types, Some(true));
    }

    #[test]
    fn test_merge_options() {
        let mut base = CompilerOptions::new();
        base.target = Some(ScriptTarget::ES5);
        base.strict = Some(true);

        let mut override_opts = CompilerOptions::new();
        override_opts.target = Some(ScriptTarget::ES2020);

        base.merge(&override_opts);

        assert_eq!(base.target, Some(ScriptTarget::ES2020));
        assert_eq!(base.strict, Some(true)); // Not overridden
    }

    #[test]
    fn test_effective_module() {
        let mut opts = CompilerOptions::new();
        opts.target = Some(ScriptTarget::ES5);
        assert_eq!(opts.effective_module(), ModuleKind::CommonJS);

        opts.target = Some(ScriptTarget::ES2020);
        assert_eq!(opts.effective_module(), ModuleKind::ES2015);

        opts.module = Some(ModuleKind::NodeNext);
        assert_eq!(opts.effective_module(), ModuleKind::NodeNext);
    }
}
