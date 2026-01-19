//! TSConfig structures and types
//!
//! This module defines the TSConfig JSON structure and related types
//! for representing tsconfig.json files.

use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::path::PathBuf;

use super::compiler_options::CompilerOptions;

/// Watch options for file watching configuration
#[derive(Debug, Clone, Default, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct WatchOptions {
    /// Watch file strategy
    #[serde(skip_serializing_if = "Option::is_none")]
    pub watch_file: Option<String>,

    /// Watch directory strategy
    #[serde(skip_serializing_if = "Option::is_none")]
    pub watch_directory: Option<String>,

    /// Fallback polling strategy
    #[serde(skip_serializing_if = "Option::is_none")]
    pub fallback_polling: Option<String>,

    /// Synchronous watch directory
    #[serde(skip_serializing_if = "Option::is_none")]
    pub synchronous_watch_directory: Option<bool>,

    /// Directories to exclude from watching
    #[serde(skip_serializing_if = "Option::is_none")]
    pub exclude_directories: Option<Vec<String>>,

    /// Files to exclude from watching
    #[serde(skip_serializing_if = "Option::is_none")]
    pub exclude_files: Option<Vec<String>>,

    /// Additional options
    #[serde(flatten)]
    pub extra: HashMap<String, serde_json::Value>,
}

/// Type acquisition configuration for JavaScript projects
#[derive(Debug, Clone, Default, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct TypeAcquisition {
    /// Enable automatic type acquisition
    #[serde(skip_serializing_if = "Option::is_none")]
    pub enable: Option<bool>,

    /// Types to include
    #[serde(skip_serializing_if = "Option::is_none")]
    pub include: Option<Vec<String>>,

    /// Types to exclude
    #[serde(skip_serializing_if = "Option::is_none")]
    pub exclude: Option<Vec<String>>,

    /// Disable filename-based type acquisition
    #[serde(skip_serializing_if = "Option::is_none")]
    pub disable_filename_based_type_acquisition: Option<bool>,
}

/// Project reference for composite projects
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ProjectReference {
    /// Path to the referenced project's tsconfig.json
    pub path: String,

    /// Original path before normalization
    #[serde(skip_serializing_if = "Option::is_none")]
    pub original_path: Option<String>,

    /// Whether to prepend output
    #[serde(skip_serializing_if = "Option::is_none")]
    pub prepend: Option<bool>,

    /// Whether this reference is circular
    #[serde(skip_serializing_if = "Option::is_none")]
    pub circular: Option<bool>,
}

/// Raw TSConfig JSON structure as read from file
#[derive(Debug, Clone, Default, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct RawTsConfig {
    /// Extends directive - path to parent config
    #[serde(skip_serializing_if = "Option::is_none")]
    pub extends: Option<String>,

    /// Compiler options
    #[serde(skip_serializing_if = "Option::is_none")]
    pub compiler_options: Option<CompilerOptions>,

    /// Watch options
    #[serde(skip_serializing_if = "Option::is_none")]
    pub watch_options: Option<WatchOptions>,

    /// Type acquisition options
    #[serde(skip_serializing_if = "Option::is_none")]
    pub type_acquisition: Option<TypeAcquisition>,

    /// Explicit files to include
    #[serde(skip_serializing_if = "Option::is_none")]
    pub files: Option<Vec<String>>,

    /// Include glob patterns
    #[serde(skip_serializing_if = "Option::is_none")]
    pub include: Option<Vec<String>>,

    /// Exclude glob patterns
    #[serde(skip_serializing_if = "Option::is_none")]
    pub exclude: Option<Vec<String>>,

    /// Project references
    #[serde(skip_serializing_if = "Option::is_none")]
    pub references: Option<Vec<ProjectReference>>,

    /// Compile on save
    #[serde(skip_serializing_if = "Option::is_none")]
    pub compile_on_save: Option<bool>,

    /// Additional/unknown options
    #[serde(flatten)]
    pub extra: HashMap<String, serde_json::Value>,
}

impl RawTsConfig {
    /// Parse a RawTsConfig from JSON string
    pub fn from_json(json: &str) -> Result<Self, serde_json::Error> {
        serde_json::from_str(json)
    }

    /// Serialize to JSON string
    pub fn to_json(&self) -> Result<String, serde_json::Error> {
        serde_json::to_string_pretty(self)
    }
}

/// Resolved TSConfig after processing extends and inheritance
#[derive(Debug, Clone, Default)]
pub struct TsConfig {
    /// The resolved compiler options
    pub compiler_options: CompilerOptions,

    /// Watch options
    pub watch_options: Option<WatchOptions>,

    /// Type acquisition options
    pub type_acquisition: Option<TypeAcquisition>,

    /// Explicit files to include
    pub files: Option<Vec<String>>,

    /// Include glob patterns
    pub include: Option<Vec<String>>,

    /// Exclude glob patterns
    pub exclude: Option<Vec<String>>,

    /// Project references
    pub references: Option<Vec<ProjectReference>>,

    /// Compile on save
    pub compile_on_save: bool,

    /// The base directory for this config
    pub base_path: PathBuf,

    /// The config file path
    pub config_file_path: Option<PathBuf>,

    /// Configs that were extended (for debugging/tracing)
    pub extended_configs: Vec<PathBuf>,
}

impl TsConfig {
    /// Create a new TsConfig with default values
    pub fn new() -> Self {
        Self::default()
    }

    /// Create a TsConfig with a base path
    pub fn with_base_path(base_path: PathBuf) -> Self {
        TsConfig {
            base_path,
            ..Default::default()
        }
    }

    /// Get effective include patterns
    /// Returns ["**/*"] if neither files nor include is specified
    pub fn effective_include(&self) -> Vec<String> {
        if let Some(ref include) = self.include {
            include.clone()
        } else if self.files.is_none() {
            vec!["**/*".to_string()]
        } else {
            vec![]
        }
    }

    /// Get effective exclude patterns
    /// Automatically excludes node_modules, bower_components, jspm_packages,
    /// and outDir/declarationDir if specified
    pub fn effective_exclude(&self) -> Vec<String> {
        let mut exclude = self.exclude.clone().unwrap_or_default();

        // Add default excludes if not explicitly included
        let defaults = [
            "node_modules",
            "bower_components",
            "jspm_packages",
        ];

        for default in &defaults {
            let default_str = (*default).to_string();
            if !exclude.iter().any(|e| e == default || e.starts_with(&format!("{}/**", default))) {
                exclude.push(default_str);
            }
        }

        // Exclude outDir and declarationDir if specified
        if let Some(ref out_dir) = self.compiler_options.out_dir {
            if !exclude.contains(out_dir) {
                exclude.push(out_dir.clone());
            }
        }

        if let Some(ref decl_dir) = self.compiler_options.declaration_dir {
            if !exclude.contains(decl_dir) {
                exclude.push(decl_dir.clone());
            }
        }

        exclude
    }

    /// Check if this is a composite project
    pub fn is_composite(&self) -> bool {
        self.compiler_options.composite == Some(true)
    }

    /// Check if this config has project references
    pub fn has_references(&self) -> bool {
        self.references.as_ref().map_or(false, |r| !r.is_empty())
    }

    /// Get the root directory for source files
    pub fn get_root_dir(&self) -> PathBuf {
        if let Some(ref root_dir) = self.compiler_options.root_dir {
            self.base_path.join(root_dir)
        } else {
            self.base_path.clone()
        }
    }

    /// Get the output directory
    pub fn get_out_dir(&self) -> Option<PathBuf> {
        self.compiler_options.out_dir.as_ref().map(|d| self.base_path.join(d))
    }

    /// Get the declaration output directory
    pub fn get_declaration_dir(&self) -> Option<PathBuf> {
        self.compiler_options.declaration_dir
            .as_ref()
            .map(|d| self.base_path.join(d))
            .or_else(|| self.get_out_dir())
    }
}

/// Configuration file specifications for matching files
#[derive(Debug, Clone, Default)]
pub struct ConfigFileSpecs {
    /// Files explicitly listed
    pub files_specs: Option<Vec<String>>,

    /// Include patterns
    pub include_specs: Option<Vec<String>>,

    /// Exclude patterns
    pub exclude_specs: Option<Vec<String>>,

    /// Validated file specs (only string values)
    pub validated_files_spec: Vec<String>,

    /// Validated include specs
    pub validated_include_specs: Vec<String>,

    /// Validated exclude specs
    pub validated_exclude_specs: Vec<String>,
}

/// Result of parsing a config file
#[derive(Debug, Clone)]
pub struct ParsedCommandLine {
    /// Resolved compiler options
    pub options: CompilerOptions,

    /// Watch options
    pub watch_options: Option<WatchOptions>,

    /// List of file names to compile
    pub file_names: Vec<String>,

    /// Project references
    pub project_references: Option<Vec<ProjectReference>>,

    /// Type acquisition settings
    pub type_acquisition: TypeAcquisition,

    /// The raw config object
    pub raw: RawTsConfig,

    /// Any parsing errors
    pub errors: Vec<ConfigError>,

    /// Wildcard directories for file watching
    pub wildcard_directories: HashMap<String, WatchDirectoryFlags>,

    /// Whether to compile on save
    pub compile_on_save: bool,
}

/// Error during config parsing
#[derive(Debug, Clone)]
pub struct ConfigError {
    /// Error message
    pub message: String,

    /// Error code
    pub code: u32,

    /// File path where error occurred
    pub file: Option<String>,

    /// Line number (1-based)
    pub line: Option<u32>,

    /// Column number (1-based)
    pub column: Option<u32>,
}

impl ConfigError {
    pub fn new(message: impl Into<String>, code: u32) -> Self {
        ConfigError {
            message: message.into(),
            code,
            file: None,
            line: None,
            column: None,
        }
    }

    pub fn with_location(mut self, file: impl Into<String>, line: u32, column: u32) -> Self {
        self.file = Some(file.into());
        self.line = Some(line);
        self.column = Some(column);
        self
    }
}

/// Flags for wildcard directory watching
#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
pub struct WatchDirectoryFlags {
    /// Whether this directory should be watched recursively
    pub recursive: bool,
}

/// Config diagnostic error codes
pub mod error_codes {
    pub const CIRCULAR_EXTENDS: u32 = 18000;
    pub const EXTENDS_NOT_FOUND: u32 = 18001;
    pub const INVALID_CONFIG_FILE: u32 = 18002;
    pub const NO_INPUT_FILES: u32 = 18003;
    pub const INVALID_COMPILER_OPTION: u32 = 18004;
    pub const DUPLICATE_COMPILER_OPTION: u32 = 18005;
    pub const CONFLICTING_OPTIONS: u32 = 18006;
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_raw_tsconfig_parse() {
        let json = r#"{
            "compilerOptions": {
                "target": "es2020",
                "module": "esnext",
                "strict": true,
                "outDir": "dist"
            },
            "include": ["src/**/*"],
            "exclude": ["node_modules"]
        }"#;

        let config = RawTsConfig::from_json(json).unwrap();
        assert!(config.compiler_options.is_some());

        let opts = config.compiler_options.unwrap();
        assert_eq!(opts.strict, Some(true));
        assert_eq!(opts.out_dir, Some("dist".to_string()));

        assert_eq!(config.include, Some(vec!["src/**/*".to_string()]));
        assert_eq!(config.exclude, Some(vec!["node_modules".to_string()]));
    }

    #[test]
    fn test_raw_tsconfig_with_extends() {
        let json = r#"{
            "extends": "./base.json",
            "compilerOptions": {
                "outDir": "dist"
            }
        }"#;

        let config = RawTsConfig::from_json(json).unwrap();
        assert_eq!(config.extends, Some("./base.json".to_string()));
    }

    #[test]
    fn test_raw_tsconfig_with_references() {
        let json = r#"{
            "compilerOptions": {
                "composite": true
            },
            "references": [
                { "path": "../lib" },
                { "path": "../utils", "prepend": true }
            ]
        }"#;

        let config = RawTsConfig::from_json(json).unwrap();
        let refs = config.references.unwrap();
        assert_eq!(refs.len(), 2);
        assert_eq!(refs[0].path, "../lib");
        assert_eq!(refs[1].path, "../utils");
        assert_eq!(refs[1].prepend, Some(true));
    }

    #[test]
    fn test_effective_include_default() {
        let config = TsConfig::new();
        let include = config.effective_include();
        assert_eq!(include, vec!["**/*"]);
    }

    #[test]
    fn test_effective_include_with_files() {
        let mut config = TsConfig::new();
        config.files = Some(vec!["index.ts".to_string()]);
        let include = config.effective_include();
        assert!(include.is_empty());
    }

    #[test]
    fn test_effective_include_explicit() {
        let mut config = TsConfig::new();
        config.include = Some(vec!["src/**/*".to_string()]);
        let include = config.effective_include();
        assert_eq!(include, vec!["src/**/*"]);
    }

    #[test]
    fn test_effective_exclude_defaults() {
        let config = TsConfig::new();
        let exclude = config.effective_exclude();
        assert!(exclude.contains(&"node_modules".to_string()));
        assert!(exclude.contains(&"bower_components".to_string()));
        assert!(exclude.contains(&"jspm_packages".to_string()));
    }

    #[test]
    fn test_effective_exclude_with_out_dir() {
        let mut config = TsConfig::new();
        config.compiler_options.out_dir = Some("dist".to_string());
        let exclude = config.effective_exclude();
        assert!(exclude.contains(&"dist".to_string()));
    }

    #[test]
    fn test_is_composite() {
        let mut config = TsConfig::new();
        assert!(!config.is_composite());

        config.compiler_options.composite = Some(true);
        assert!(config.is_composite());
    }

    #[test]
    fn test_has_references() {
        let mut config = TsConfig::new();
        assert!(!config.has_references());

        config.references = Some(vec![ProjectReference {
            path: "../lib".to_string(),
            original_path: None,
            prepend: None,
            circular: None,
        }]);
        assert!(config.has_references());
    }

    #[test]
    fn test_config_error() {
        let error = ConfigError::new("Test error", 18000)
            .with_location("tsconfig.json", 5, 10);

        assert_eq!(error.message, "Test error");
        assert_eq!(error.code, 18000);
        assert_eq!(error.file, Some("tsconfig.json".to_string()));
        assert_eq!(error.line, Some(5));
        assert_eq!(error.column, Some(10));
    }
}
