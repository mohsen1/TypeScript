//! TSConfig file representation and types.
//!
//! This module defines the structure of tsconfig.json files.

use serde::{Deserialize, Serialize};
use std::path::PathBuf;

use super::options::CompilerOptions;

/// Project reference configuration
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct ProjectReference {
    /// Path to the referenced project's tsconfig.json
    pub path: PathBuf,
    /// Whether to prepend the referenced project's output
    #[serde(skip_serializing_if = "Option::is_none")]
    pub prepend: Option<bool>,
    /// Whether this reference should be circular
    #[serde(skip_serializing_if = "Option::is_none")]
    pub circular: Option<bool>,
}

/// Watch options for incremental compilation
#[derive(Debug, Clone, Default, Serialize, Deserialize)]
#[serde(rename_all = "camelCase", default)]
pub struct WatchOptions {
    #[serde(skip_serializing_if = "Option::is_none")]
    pub watch_file: Option<WatchFileKind>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub watch_directory: Option<WatchDirectoryKind>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub fallback_polling: Option<PollingWatchKind>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub synchronous_watch_directory: Option<bool>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub exclude_directories: Option<Vec<String>>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub exclude_files: Option<Vec<String>>,
}

/// Watch file strategy
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub enum WatchFileKind {
    FixedPollingInterval,
    PriorityPollingInterval,
    DynamicPriorityPolling,
    FixedChunkSizePolling,
    UseFsEvents,
    UseFsEventsOnParentDirectory,
}

/// Watch directory strategy
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub enum WatchDirectoryKind {
    UseFsEvents,
    FixedPollingInterval,
    DynamicPriorityPolling,
    FixedChunkSizePolling,
}

/// Polling watch strategy
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub enum PollingWatchKind {
    FixedInterval,
    PriorityInterval,
    DynamicPriority,
    FixedChunkSize,
}

/// Type acquisition options for JavaScript projects
#[derive(Debug, Clone, Default, Serialize, Deserialize)]
#[serde(rename_all = "camelCase", default)]
pub struct TypeAcquisition {
    #[serde(skip_serializing_if = "Option::is_none")]
    pub enable: Option<bool>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub include: Option<Vec<String>>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub exclude: Option<Vec<String>>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub disable_filename_based_type_acquisition: Option<bool>,
}

/// Build options for project building
#[derive(Debug, Clone, Default, Serialize, Deserialize)]
#[serde(rename_all = "camelCase", default)]
pub struct BuildOptions {
    #[serde(skip_serializing_if = "Option::is_none")]
    pub dry: Option<bool>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub force: Option<bool>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub verbose: Option<bool>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub incremental: Option<bool>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub assume_changes_only_affect_direct_dependencies: Option<bool>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub trace_resolution: Option<bool>,
}

/// Raw TSConfig file structure (as read from JSON)
#[derive(Debug, Clone, Default, Serialize, Deserialize)]
#[serde(rename_all = "camelCase", default)]
pub struct RawTsConfig {
    /// Path to a base configuration file to extend
    #[serde(skip_serializing_if = "Option::is_none")]
    pub extends: Option<ExtendsValue>,

    /// Compiler options
    #[serde(skip_serializing_if = "Option::is_none")]
    pub compiler_options: Option<CompilerOptions>,

    /// Files to include in the compilation
    #[serde(skip_serializing_if = "Option::is_none")]
    pub files: Option<Vec<String>>,

    /// Glob patterns for files to include
    #[serde(skip_serializing_if = "Option::is_none")]
    pub include: Option<Vec<String>>,

    /// Glob patterns for files to exclude
    #[serde(skip_serializing_if = "Option::is_none")]
    pub exclude: Option<Vec<String>>,

    /// Project references
    #[serde(skip_serializing_if = "Option::is_none")]
    pub references: Option<Vec<ProjectReference>>,

    /// Watch options
    #[serde(skip_serializing_if = "Option::is_none")]
    pub watch_options: Option<WatchOptions>,

    /// Type acquisition options
    #[serde(skip_serializing_if = "Option::is_none")]
    pub type_acquisition: Option<TypeAcquisition>,

    /// Build options (for tsc -b)
    #[serde(skip_serializing_if = "Option::is_none")]
    pub build_options: Option<BuildOptions>,

    /// Whether this is a solution-style config
    #[serde(skip_serializing_if = "Option::is_none", rename = "compileOnSave")]
    pub compile_on_save: Option<bool>,
}

/// Extends can be a string or array of strings
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(untagged)]
pub enum ExtendsValue {
    Single(String),
    Multiple(Vec<String>),
}

impl ExtendsValue {
    /// Get all extends values as a vector
    pub fn to_vec(&self) -> Vec<&str> {
        match self {
            ExtendsValue::Single(s) => vec![s.as_str()],
            ExtendsValue::Multiple(v) => v.iter().map(|s| s.as_str()).collect(),
        }
    }
}

/// Resolved TSConfig with all inheritance applied
#[derive(Debug, Clone)]
pub struct ResolvedTsConfig {
    /// The config file path
    pub config_path: PathBuf,

    /// Resolved compiler options (with inheritance applied)
    pub compiler_options: CompilerOptions,

    /// Resolved files list
    pub files: Vec<PathBuf>,

    /// Resolved include patterns
    pub include: Vec<String>,

    /// Resolved exclude patterns
    pub exclude: Vec<String>,

    /// Project references
    pub references: Vec<ProjectReference>,

    /// Watch options
    pub watch_options: WatchOptions,

    /// Type acquisition options
    pub type_acquisition: TypeAcquisition,

    /// Raw config for additional fields
    pub raw: RawTsConfig,
}

impl ResolvedTsConfig {
    /// Create a new resolved config
    pub fn new(config_path: PathBuf) -> Self {
        Self {
            config_path,
            compiler_options: CompilerOptions::default(),
            files: Vec::new(),
            include: Vec::new(),
            exclude: Vec::new(),
            references: Vec::new(),
            watch_options: WatchOptions::default(),
            type_acquisition: TypeAcquisition::default(),
            raw: RawTsConfig::default(),
        }
    }

    /// Get the base directory for this config
    pub fn base_dir(&self) -> PathBuf {
        self.config_path.parent()
            .map(|p| p.to_path_buf())
            .unwrap_or_else(|| PathBuf::from("."))
    }

    /// Check if this is a composite project
    pub fn is_composite(&self) -> bool {
        self.compiler_options.composite == Some(true)
    }

    /// Check if this has project references
    pub fn has_references(&self) -> bool {
        !self.references.is_empty()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_parse_simple_tsconfig() {
        let json = r#"{
            "compilerOptions": {
                "target": "es2020",
                "module": "esnext",
                "strict": true
            },
            "include": ["src/**/*"],
            "exclude": ["node_modules"]
        }"#;

        let config: RawTsConfig = serde_json::from_str(json).unwrap();

        assert!(config.compiler_options.is_some());
        let opts = config.compiler_options.unwrap();
        assert_eq!(opts.strict, Some(true));

        assert_eq!(config.include, Some(vec!["src/**/*".to_string()]));
        assert_eq!(config.exclude, Some(vec!["node_modules".to_string()]));
    }

    #[test]
    fn test_parse_extends_single() {
        let json = r#"{
            "extends": "./base.json"
        }"#;

        let config: RawTsConfig = serde_json::from_str(json).unwrap();

        assert!(config.extends.is_some());
        if let Some(ExtendsValue::Single(s)) = config.extends {
            assert_eq!(s, "./base.json");
        } else {
            panic!("Expected single extends");
        }
    }

    #[test]
    fn test_parse_extends_multiple() {
        let json = r#"{
            "extends": ["./base.json", "./strict.json"]
        }"#;

        let config: RawTsConfig = serde_json::from_str(json).unwrap();

        assert!(config.extends.is_some());
        if let Some(ExtendsValue::Multiple(v)) = config.extends {
            assert_eq!(v.len(), 2);
        } else {
            panic!("Expected multiple extends");
        }
    }

    #[test]
    fn test_parse_project_references() {
        let json = r#"{
            "references": [
                { "path": "./core" },
                { "path": "./utils", "prepend": true }
            ]
        }"#;

        let config: RawTsConfig = serde_json::from_str(json).unwrap();

        assert!(config.references.is_some());
        let refs = config.references.unwrap();
        assert_eq!(refs.len(), 2);
        assert_eq!(refs[0].path, PathBuf::from("./core"));
        assert_eq!(refs[1].prepend, Some(true));
    }

    #[test]
    fn test_resolved_config_base_dir() {
        let config = ResolvedTsConfig::new(PathBuf::from("/project/tsconfig.json"));
        assert_eq!(config.base_dir(), PathBuf::from("/project"));
    }
}
