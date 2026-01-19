//! TSConfig parser implementation
//!
//! This module provides functionality for:
//! - Parsing tsconfig.json files
//! - Handling extends directive for config inheritance
//! - Resolving include/exclude glob patterns
//! - Supporting project references

use std::collections::{HashMap, HashSet};
use std::fs;
use std::path::{Path, PathBuf};

use glob::Pattern;

use super::tsconfig::{
    ConfigError, RawTsConfig, TsConfig, WatchDirectoryFlags, error_codes,
};

/// Host interface for file system operations
/// This allows for testing with mock file systems
pub trait ParseConfigHost {
    /// Read a file as a string
    fn read_file(&self, path: &Path) -> Result<String, std::io::Error>;

    /// Check if a file exists
    fn file_exists(&self, path: &Path) -> bool;

    /// Read a directory
    fn read_directory(&self, path: &Path) -> Result<Vec<PathBuf>, std::io::Error>;

    /// Check if path is a directory
    fn is_directory(&self, path: &Path) -> bool;

    /// Get the current working directory
    fn get_current_directory(&self) -> PathBuf;

    /// Whether the file system is case-sensitive
    fn use_case_sensitive_file_names(&self) -> bool;
}

/// Default implementation using the real file system
pub struct RealFileSystem;

impl ParseConfigHost for RealFileSystem {
    fn read_file(&self, path: &Path) -> Result<String, std::io::Error> {
        fs::read_to_string(path)
    }

    fn file_exists(&self, path: &Path) -> bool {
        path.exists() && path.is_file()
    }

    fn read_directory(&self, path: &Path) -> Result<Vec<PathBuf>, std::io::Error> {
        let entries = fs::read_dir(path)?;
        let mut paths = Vec::new();
        for entry in entries {
            paths.push(entry?.path());
        }
        Ok(paths)
    }

    fn is_directory(&self, path: &Path) -> bool {
        path.is_dir()
    }

    fn get_current_directory(&self) -> PathBuf {
        std::env::current_dir().unwrap_or_else(|_| PathBuf::from("."))
    }

    fn use_case_sensitive_file_names(&self) -> bool {
        cfg!(target_os = "linux")
    }
}

/// Parser for tsconfig.json files
pub struct TsConfigParser<H: ParseConfigHost> {
    host: H,
    /// Stack to detect circular extends
    extends_stack: HashSet<PathBuf>,
    /// Cache for already parsed configs (for future use)
    #[allow(dead_code)]
    config_cache: HashMap<PathBuf, RawTsConfig>,
}

impl<H: ParseConfigHost> TsConfigParser<H> {
    /// Create a new parser with the given host
    pub fn new(host: H) -> Self {
        TsConfigParser {
            host,
            extends_stack: HashSet::new(),
            config_cache: HashMap::new(),
        }
    }

    /// Parse a tsconfig.json file
    pub fn parse(&mut self, config_path: &Path) -> Result<TsConfig, Vec<ConfigError>> {
        let config_path = self.normalize_path(config_path);
        let base_path = config_path.parent().unwrap_or(Path::new(".")).to_path_buf();

        self.parse_config_file(&config_path, &base_path)
    }

    /// Parse a config file with a specific base path
    fn parse_config_file(
        &mut self,
        config_path: &PathBuf,
        base_path: &PathBuf,
    ) -> Result<TsConfig, Vec<ConfigError>> {
        let mut errors = Vec::new();

        // Check for circular extends
        if self.extends_stack.contains(config_path) {
            errors.push(ConfigError::new(
                format!("Circular reference in extends: {}", config_path.display()),
                error_codes::CIRCULAR_EXTENDS,
            ));
            return Err(errors);
        }

        self.extends_stack.insert(config_path.clone());

        // Read and parse the config file
        let content = match self.host.read_file(config_path) {
            Ok(c) => c,
            Err(e) => {
                errors.push(ConfigError::new(
                    format!("Cannot read config file '{}': {}", config_path.display(), e),
                    error_codes::INVALID_CONFIG_FILE,
                ));
                self.extends_stack.remove(config_path);
                return Err(errors);
            }
        };

        let raw_config: RawTsConfig = match serde_json::from_str(&content) {
            Ok(c) => c,
            Err(e) => {
                errors.push(ConfigError::new(
                    format!("Invalid JSON in '{}': {}", config_path.display(), e),
                    error_codes::INVALID_CONFIG_FILE,
                ));
                self.extends_stack.remove(config_path);
                return Err(errors);
            }
        };

        // Start with an empty config
        let mut result = TsConfig::new();
        result.base_path = base_path.clone();
        result.config_file_path = Some(config_path.clone());

        // Handle extends directive
        if let Some(ref extends_path) = raw_config.extends {
            match self.resolve_extends(extends_path, base_path) {
                Ok(parent_config_path) => {
                    let parent_base = parent_config_path.parent().unwrap_or(Path::new(".")).to_path_buf();
                    match self.parse_config_file(&parent_config_path, &parent_base) {
                        Ok(parent_config) => {
                            result = parent_config;
                            result.extended_configs.push(parent_config_path);
                            // Reset base path to current config
                            result.base_path = base_path.clone();
                            result.config_file_path = Some(config_path.clone());
                        }
                        Err(mut parent_errors) => {
                            errors.append(&mut parent_errors);
                        }
                    }
                }
                Err(e) => {
                    errors.push(e);
                }
            }
        }

        // Merge compiler options
        if let Some(ref opts) = raw_config.compiler_options {
            result.compiler_options.merge(opts);
        }

        // Apply strict mode defaults after merging
        result.compiler_options.apply_strict_defaults();

        // Override other fields from current config (don't inherit these)
        if raw_config.files.is_some() {
            result.files = raw_config.files;
        }
        if raw_config.include.is_some() {
            result.include = raw_config.include;
        }
        if raw_config.exclude.is_some() {
            result.exclude = raw_config.exclude;
        }
        if raw_config.references.is_some() {
            result.references = raw_config.references;
        }

        // Merge watch options
        if let Some(ref watch_opts) = raw_config.watch_options {
            result.watch_options = Some(watch_opts.clone());
        }

        // Merge type acquisition
        if let Some(ref type_acq) = raw_config.type_acquisition {
            result.type_acquisition = Some(type_acq.clone());
        }

        if let Some(compile_on_save) = raw_config.compile_on_save {
            result.compile_on_save = compile_on_save;
        }

        self.extends_stack.remove(config_path);

        if errors.is_empty() {
            Ok(result)
        } else {
            Err(errors)
        }
    }

    /// Resolve the extends path to an absolute path
    fn resolve_extends(&self, extends_path: &str, base_path: &Path) -> Result<PathBuf, ConfigError> {
        let resolved = if extends_path.starts_with('.') {
            // Relative path
            base_path.join(extends_path)
        } else if extends_path.starts_with('/') || extends_path.contains(':') {
            // Absolute path
            PathBuf::from(extends_path)
        } else {
            // Try to resolve as a package (simplified - in reality would use node resolution)
            // First try node_modules in base path
            let node_modules_path = base_path.join("node_modules").join(extends_path);
            if self.host.file_exists(&node_modules_path) {
                node_modules_path
            } else {
                // Try with .json extension
                let with_json = base_path.join("node_modules").join(format!("{}.json", extends_path));
                if self.host.file_exists(&with_json) {
                    with_json
                } else {
                    // Try as a direct path
                    base_path.join(extends_path)
                }
            }
        };

        // Add .json extension if missing
        let resolved = if resolved.extension().is_none() {
            resolved.with_extension("json")
        } else {
            resolved
        };

        let normalized = self.normalize_path(&resolved);

        if self.host.file_exists(&normalized) {
            Ok(normalized)
        } else {
            Err(ConfigError::new(
                format!("Cannot find config file '{}' specified in extends", extends_path),
                error_codes::EXTENDS_NOT_FOUND,
            ))
        }
    }

    /// Normalize a path (resolve . and ..)
    fn normalize_path(&self, path: &Path) -> PathBuf {
        let mut components = Vec::new();
        for component in path.components() {
            match component {
                std::path::Component::ParentDir => {
                    if !components.is_empty() {
                        components.pop();
                    }
                }
                std::path::Component::CurDir => {}
                _ => {
                    components.push(component);
                }
            }
        }
        components.iter().collect()
    }

    /// Get the file list from the parsed config
    pub fn get_file_names(
        &self,
        config: &TsConfig,
        extra_extensions: &[String],
    ) -> Result<Vec<PathBuf>, Vec<ConfigError>> {
        let mut files = Vec::new();
        let mut errors = Vec::new();

        // Add explicit files
        if let Some(ref explicit_files) = config.files {
            for file in explicit_files {
                let path = config.base_path.join(file);
                if self.host.file_exists(&path) {
                    files.push(path);
                } else {
                    errors.push(ConfigError::new(
                        format!("File '{}' not found", file),
                        error_codes::NO_INPUT_FILES,
                    ));
                }
            }
        }

        // Get include and exclude patterns
        let include = config.effective_include();
        let exclude = config.effective_exclude();

        // Build exclude patterns
        let exclude_patterns: Vec<Pattern> = exclude
            .iter()
            .filter_map(|p| Pattern::new(&self.normalize_glob_pattern(p)).ok())
            .collect();

        // Collect files matching include patterns
        if !include.is_empty() {
            let extensions = self.get_supported_extensions(config, extra_extensions);
            for pattern in &include {
                match self.collect_files_matching_pattern(
                    &config.base_path,
                    pattern,
                    &exclude_patterns,
                    &extensions,
                ) {
                    Ok(matched) => files.extend(matched),
                    Err(e) => errors.push(e),
                }
            }
        }

        // Remove duplicates
        files.sort();
        files.dedup();

        if files.is_empty() && config.files.is_none() {
            errors.push(ConfigError::new(
                format!(
                    "No inputs were found in config file '{}'. Specified include paths were {:?} and exclude paths were {:?}",
                    config.config_file_path.as_ref().map(|p| p.display().to_string()).unwrap_or_default(),
                    include,
                    exclude
                ),
                error_codes::NO_INPUT_FILES,
            ));
        }

        if errors.is_empty() {
            Ok(files)
        } else {
            Err(errors)
        }
    }

    /// Get supported file extensions based on compiler options
    fn get_supported_extensions(&self, config: &TsConfig, extra: &[String]) -> Vec<String> {
        let mut extensions = vec![
            ".ts".to_string(),
            ".tsx".to_string(),
            ".d.ts".to_string(),
        ];

        if config.compiler_options.allow_js == Some(true) {
            extensions.push(".js".to_string());
            extensions.push(".jsx".to_string());
        }

        if config.compiler_options.resolve_json_module == Some(true) {
            extensions.push(".json".to_string());
        }

        extensions.extend(extra.iter().cloned());
        extensions
    }

    /// Normalize a glob pattern for the glob crate
    fn normalize_glob_pattern(&self, pattern: &str) -> String {
        // Convert TypeScript glob patterns to standard glob patterns
        pattern
            .replace("**/*", "**")
            .replace("/**/", "/**/")
    }

    /// Collect files matching a glob pattern
    fn collect_files_matching_pattern(
        &self,
        base_path: &Path,
        pattern: &str,
        exclude_patterns: &[Pattern],
        extensions: &[String],
    ) -> Result<Vec<PathBuf>, ConfigError> {
        let mut files = Vec::new();

        // Simple recursive file collection
        self.collect_files_recursive(base_path, pattern, exclude_patterns, extensions, &mut files)?;

        Ok(files)
    }

    /// Recursively collect files
    fn collect_files_recursive(
        &self,
        dir: &Path,
        pattern: &str,
        exclude_patterns: &[Pattern],
        extensions: &[String],
        files: &mut Vec<PathBuf>,
    ) -> Result<(), ConfigError> {
        if !self.host.is_directory(dir) {
            return Ok(());
        }

        let entries = self.host.read_directory(dir).map_err(|e| {
            ConfigError::new(
                format!("Cannot read directory '{}': {}", dir.display(), e),
                error_codes::INVALID_CONFIG_FILE,
            )
        })?;

        for entry in entries {
            let relative = entry
                .strip_prefix(dir)
                .unwrap_or(&entry)
                .to_string_lossy()
                .to_string();

            // Check if excluded
            let should_exclude = exclude_patterns.iter().any(|p| p.matches(&relative));
            if should_exclude {
                continue;
            }

            if self.host.is_directory(&entry) {
                // Recurse into directories if pattern allows
                if pattern.contains("**") || pattern.contains("**/") {
                    self.collect_files_recursive(&entry, pattern, exclude_patterns, extensions, files)?;
                }
            } else {
                // Check if file matches pattern and has correct extension
                let matches_pattern = self.file_matches_pattern(&entry, pattern);
                let has_extension = extensions.iter().any(|ext| {
                    entry
                        .file_name()
                        .and_then(|n| n.to_str())
                        .map(|n| n.ends_with(ext))
                        .unwrap_or(false)
                });

                if matches_pattern && has_extension {
                    files.push(entry);
                }
            }
        }

        Ok(())
    }

    /// Check if a file matches a pattern (simplified)
    fn file_matches_pattern(&self, file: &Path, pattern: &str) -> bool {
        // Simplified pattern matching
        if pattern == "**/*" || pattern == "**" {
            return true;
        }

        if let Ok(glob_pattern) = Pattern::new(pattern) {
            if let Some(file_str) = file.to_str() {
                return glob_pattern.matches(file_str);
            }
        }

        false
    }

    /// Build wildcard directories for file watching
    pub fn get_wildcard_directories(&self, config: &TsConfig) -> HashMap<String, WatchDirectoryFlags> {
        let mut directories = HashMap::new();

        for pattern in config.effective_include() {
            // Extract directory part from pattern
            let dir = if pattern.contains("**") {
                pattern.split("**").next().unwrap_or(".")
            } else {
                pattern.rsplit('/').skip(1).next().unwrap_or(".")
            };

            let abs_dir = config.base_path.join(dir);
            if self.host.is_directory(&abs_dir) {
                directories.insert(
                    abs_dir.to_string_lossy().to_string(),
                    WatchDirectoryFlags {
                        recursive: pattern.contains("**"),
                    },
                );
            }
        }

        directories
    }
}

/// Convenience function to parse a tsconfig.json file using the real file system
pub fn parse_tsconfig(config_path: &Path) -> Result<TsConfig, Vec<ConfigError>> {
    let mut parser = TsConfigParser::new(RealFileSystem);
    parser.parse(config_path)
}

/// Parse tsconfig.json content from a string
pub fn parse_tsconfig_string(
    content: &str,
    base_path: &Path,
) -> Result<TsConfig, Vec<ConfigError>> {
    let mut errors = Vec::new();

    let raw_config: RawTsConfig = match serde_json::from_str(content) {
        Ok(c) => c,
        Err(e) => {
            errors.push(ConfigError::new(
                format!("Invalid JSON: {}", e),
                error_codes::INVALID_CONFIG_FILE,
            ));
            return Err(errors);
        }
    };

    let mut result = TsConfig::new();
    result.base_path = base_path.to_path_buf();

    if let Some(opts) = raw_config.compiler_options {
        result.compiler_options = opts;
    }

    result.compiler_options.apply_strict_defaults();

    result.files = raw_config.files;
    result.include = raw_config.include;
    result.exclude = raw_config.exclude;
    result.references = raw_config.references;
    result.watch_options = raw_config.watch_options;
    result.type_acquisition = raw_config.type_acquisition;
    result.compile_on_save = raw_config.compile_on_save.unwrap_or(false);

    Ok(result)
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::collections::HashMap;

    /// Mock file system for testing
    struct MockFileSystem {
        files: HashMap<PathBuf, String>,
    }

    impl MockFileSystem {
        fn new() -> Self {
            MockFileSystem {
                files: HashMap::new(),
            }
        }

        fn add_file(&mut self, path: impl Into<PathBuf>, content: impl Into<String>) {
            self.files.insert(path.into(), content.into());
        }
    }

    impl ParseConfigHost for MockFileSystem {
        fn read_file(&self, path: &Path) -> Result<String, std::io::Error> {
            self.files.get(path).cloned().ok_or_else(|| {
                std::io::Error::new(std::io::ErrorKind::NotFound, "File not found")
            })
        }

        fn file_exists(&self, path: &Path) -> bool {
            self.files.contains_key(path)
        }

        fn read_directory(&self, _path: &Path) -> Result<Vec<PathBuf>, std::io::Error> {
            Ok(Vec::new())
        }

        fn is_directory(&self, _path: &Path) -> bool {
            false
        }

        fn get_current_directory(&self) -> PathBuf {
            PathBuf::from("/test")
        }

        fn use_case_sensitive_file_names(&self) -> bool {
            true
        }
    }

    #[test]
    fn test_parse_simple_config() {
        let mut fs = MockFileSystem::new();
        fs.add_file(
            "/test/tsconfig.json",
            r#"{
                "compilerOptions": {
                    "target": "es2020",
                    "module": "esnext",
                    "strict": true
                }
            }"#,
        );

        let mut parser = TsConfigParser::new(fs);
        let config = parser.parse(Path::new("/test/tsconfig.json")).unwrap();

        assert!(config.compiler_options.is_strict());
        // strict should enable noImplicitAny, etc.
        assert!(config.compiler_options.effective_no_implicit_any());
    }

    #[test]
    fn test_parse_with_extends() {
        let mut fs = MockFileSystem::new();
        fs.add_file(
            "/test/base.json",
            r#"{
                "compilerOptions": {
                    "target": "es2015",
                    "strict": true
                }
            }"#,
        );
        fs.add_file(
            "/test/tsconfig.json",
            r#"{
                "extends": "./base.json",
                "compilerOptions": {
                    "outDir": "dist"
                }
            }"#,
        );

        let mut parser = TsConfigParser::new(fs);
        let config = parser.parse(Path::new("/test/tsconfig.json")).unwrap();

        // Should inherit strict from base
        assert!(config.compiler_options.is_strict());
        // Should have outDir from child
        assert_eq!(config.compiler_options.out_dir, Some("dist".to_string()));
    }

    #[test]
    fn test_circular_extends_error() {
        let mut fs = MockFileSystem::new();
        fs.add_file(
            "/test/a.json",
            r#"{ "extends": "./b.json" }"#,
        );
        fs.add_file(
            "/test/b.json",
            r#"{ "extends": "./a.json" }"#,
        );

        let mut parser = TsConfigParser::new(fs);
        let result = parser.parse(Path::new("/test/a.json"));

        assert!(result.is_err());
        let errors = result.unwrap_err();
        assert!(errors.iter().any(|e| e.code == error_codes::CIRCULAR_EXTENDS));
    }

    #[test]
    fn test_extends_not_found_error() {
        let mut fs = MockFileSystem::new();
        fs.add_file(
            "/test/tsconfig.json",
            r#"{ "extends": "./nonexistent.json" }"#,
        );

        let mut parser = TsConfigParser::new(fs);
        let result = parser.parse(Path::new("/test/tsconfig.json"));

        assert!(result.is_err());
        let errors = result.unwrap_err();
        assert!(errors.iter().any(|e| e.code == error_codes::EXTENDS_NOT_FOUND));
    }

    #[test]
    fn test_parse_with_include_exclude() {
        let mut fs = MockFileSystem::new();
        fs.add_file(
            "/test/tsconfig.json",
            r#"{
                "compilerOptions": {
                    "target": "es2020"
                },
                "include": ["src/**/*"],
                "exclude": ["node_modules", "dist"]
            }"#,
        );

        let mut parser = TsConfigParser::new(fs);
        let config = parser.parse(Path::new("/test/tsconfig.json")).unwrap();

        assert_eq!(config.include, Some(vec!["src/**/*".to_string()]));
        assert_eq!(
            config.exclude,
            Some(vec!["node_modules".to_string(), "dist".to_string()])
        );
    }

    #[test]
    fn test_parse_with_references() {
        let mut fs = MockFileSystem::new();
        fs.add_file(
            "/test/tsconfig.json",
            r#"{
                "compilerOptions": {
                    "composite": true
                },
                "references": [
                    { "path": "../lib" },
                    { "path": "../utils" }
                ]
            }"#,
        );

        let mut parser = TsConfigParser::new(fs);
        let config = parser.parse(Path::new("/test/tsconfig.json")).unwrap();

        assert!(config.is_composite());
        assert!(config.has_references());
        let refs = config.references.unwrap();
        assert_eq!(refs.len(), 2);
    }

    #[test]
    fn test_parse_tsconfig_string() {
        let content = r#"{
            "compilerOptions": {
                "target": "es2020",
                "strict": true,
                "outDir": "dist"
            },
            "include": ["src/**/*"]
        }"#;

        let config = parse_tsconfig_string(content, Path::new("/test")).unwrap();

        assert!(config.compiler_options.is_strict());
        assert_eq!(config.compiler_options.out_dir, Some("dist".to_string()));
        assert_eq!(config.include, Some(vec!["src/**/*".to_string()]));
    }

    #[test]
    fn test_multi_level_extends() {
        let mut fs = MockFileSystem::new();
        fs.add_file(
            "/test/base.json",
            r#"{
                "compilerOptions": {
                    "target": "es2015",
                    "strict": true
                }
            }"#,
        );
        fs.add_file(
            "/test/middle.json",
            r#"{
                "extends": "./base.json",
                "compilerOptions": {
                    "module": "esnext"
                }
            }"#,
        );
        fs.add_file(
            "/test/tsconfig.json",
            r#"{
                "extends": "./middle.json",
                "compilerOptions": {
                    "outDir": "dist"
                }
            }"#,
        );

        let mut parser = TsConfigParser::new(fs);
        let config = parser.parse(Path::new("/test/tsconfig.json")).unwrap();

        // Should inherit strict from base
        assert!(config.compiler_options.is_strict());
        // Should have outDir from current config
        assert_eq!(config.compiler_options.out_dir, Some("dist".to_string()));
        // Should track extended configs
        assert_eq!(config.extended_configs.len(), 2);
    }

    #[test]
    fn test_override_strict_option() {
        let mut fs = MockFileSystem::new();
        fs.add_file(
            "/test/base.json",
            r#"{
                "compilerOptions": {
                    "strict": true
                }
            }"#,
        );
        fs.add_file(
            "/test/tsconfig.json",
            r#"{
                "extends": "./base.json",
                "compilerOptions": {
                    "noImplicitAny": false
                }
            }"#,
        );

        let mut parser = TsConfigParser::new(fs);
        let config = parser.parse(Path::new("/test/tsconfig.json")).unwrap();

        // strict is inherited
        assert!(config.compiler_options.is_strict());
        // but noImplicitAny is explicitly set to false
        assert_eq!(config.compiler_options.no_implicit_any, Some(false));
        // effective value should be false
        assert!(!config.compiler_options.effective_no_implicit_any());
    }
}
