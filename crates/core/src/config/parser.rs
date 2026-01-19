//! TSConfig parsing and resolution.
//!
//! This module handles reading, parsing, and resolving tsconfig.json files,
//! including extends inheritance and glob pattern resolution.

use std::collections::HashSet;
use std::fs;
use std::io;
use std::path::{Path, PathBuf};

use glob::glob;

use super::options::CompilerOptions;
use super::tsconfig::{RawTsConfig, ResolvedTsConfig};

/// Error type for TSConfig parsing
#[derive(Debug)]
pub enum TsConfigError {
    /// Failed to read the config file
    IoError(io::Error),
    /// Failed to parse JSON
    ParseError(serde_json::Error),
    /// Circular extends reference detected
    CircularExtends(Vec<PathBuf>),
    /// Extended config file not found
    ExtendedConfigNotFound(String, PathBuf),
    /// Invalid glob pattern
    InvalidGlob(String),
    /// Config file not found
    ConfigNotFound(PathBuf),
}

impl std::fmt::Display for TsConfigError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            TsConfigError::IoError(e) => write!(f, "IO error: {}", e),
            TsConfigError::ParseError(e) => write!(f, "JSON parse error: {}", e),
            TsConfigError::CircularExtends(chain) => {
                write!(f, "Circular extends reference: {:?}", chain)
            }
            TsConfigError::ExtendedConfigNotFound(name, base) => {
                write!(f, "Extended config '{}' not found from '{}'", name, base.display())
            }
            TsConfigError::InvalidGlob(pattern) => write!(f, "Invalid glob pattern: {}", pattern),
            TsConfigError::ConfigNotFound(path) => {
                write!(f, "Config file not found: {}", path.display())
            }
        }
    }
}

impl std::error::Error for TsConfigError {}

impl From<io::Error> for TsConfigError {
    fn from(e: io::Error) -> Self {
        TsConfigError::IoError(e)
    }
}

impl From<serde_json::Error> for TsConfigError {
    fn from(e: serde_json::Error) -> Self {
        TsConfigError::ParseError(e)
    }
}

/// TSConfig parser
pub struct TsConfigParser {
    /// Already visited configs (for circular reference detection)
    visited: HashSet<PathBuf>,
    /// Stack of configs being processed
    stack: Vec<PathBuf>,
}

impl TsConfigParser {
    /// Create a new parser
    pub fn new() -> Self {
        Self {
            visited: HashSet::new(),
            stack: Vec::new(),
        }
    }

    /// Parse a tsconfig.json file and resolve all inheritance
    pub fn parse(&mut self, config_path: &Path) -> Result<ResolvedTsConfig, TsConfigError> {
        let canonical_path = config_path
            .canonicalize()
            .unwrap_or_else(|_| config_path.to_path_buf());

        // Check for circular reference
        if self.visited.contains(&canonical_path) {
            return Err(TsConfigError::CircularExtends(self.stack.clone()));
        }

        // Check if file exists
        if !config_path.exists() {
            return Err(TsConfigError::ConfigNotFound(config_path.to_path_buf()));
        }

        self.visited.insert(canonical_path.clone());
        self.stack.push(canonical_path.clone());

        // Read and parse the config file
        let content = fs::read_to_string(config_path)?;
        let raw: RawTsConfig = parse_json_with_comments(&content)?;

        let base_dir = config_path.parent().unwrap_or(Path::new("."));

        // Start with empty options
        let mut resolved_options = CompilerOptions::default();

        // Process extends if present
        if let Some(ref extends) = raw.extends {
            for extend_path in extends.to_vec() {
                let extended_config = self.resolve_extends(extend_path, base_dir)?;
                resolved_options.merge(&extended_config.compiler_options);
            }
        }

        // Merge this config's options on top
        if let Some(ref opts) = raw.compiler_options {
            resolved_options.merge(opts);
        }

        // Apply strict mode if enabled
        resolved_options.apply_strict_mode();

        // Resolve paths relative to config directory
        self.resolve_paths(&mut resolved_options, base_dir);

        // Resolve include/exclude patterns
        let include = raw.include.clone().unwrap_or_else(|| {
            if raw.files.is_some() {
                vec![] // If files is specified, don't include by default
            } else {
                vec!["**/*".to_string()] // Default include pattern
            }
        });

        let exclude = raw.exclude.clone().unwrap_or_else(|| {
            let mut default_exclude = vec![
                "node_modules".to_string(),
                "bower_components".to_string(),
                "jspm_packages".to_string(),
            ];

            // Add outDir to default excludes
            if let Some(ref out_dir) = resolved_options.out_dir {
                if let Some(s) = out_dir.to_str() {
                    default_exclude.push(s.to_string());
                }
            }

            // Add declarationDir to default excludes
            if let Some(ref decl_dir) = resolved_options.declaration_dir {
                if let Some(s) = decl_dir.to_str() {
                    default_exclude.push(s.to_string());
                }
            }

            default_exclude
        });

        // Resolve files
        let files = self.resolve_files(&raw, base_dir, &include, &exclude)?;

        self.stack.pop();

        Ok(ResolvedTsConfig {
            config_path: config_path.to_path_buf(),
            compiler_options: resolved_options,
            files,
            include,
            exclude,
            references: raw.references.clone().unwrap_or_default(),
            watch_options: raw.watch_options.clone().unwrap_or_default(),
            type_acquisition: raw.type_acquisition.clone().unwrap_or_default(),
            raw,
        })
    }

    /// Resolve an extends reference
    fn resolve_extends(
        &mut self,
        extend_path: &str,
        base_dir: &Path,
    ) -> Result<ResolvedTsConfig, TsConfigError> {
        let resolved_path = self.resolve_extends_path(extend_path, base_dir)?;
        self.parse(&resolved_path)
    }

    /// Resolve the path for an extends value
    fn resolve_extends_path(
        &self,
        extend_path: &str,
        base_dir: &Path,
    ) -> Result<PathBuf, TsConfigError> {
        // Handle relative paths
        if extend_path.starts_with("./") || extend_path.starts_with("../") {
            let mut resolved = base_dir.join(extend_path);

            // Try with .json extension if needed
            if !resolved.exists() && !extend_path.ends_with(".json") {
                resolved = base_dir.join(format!("{}.json", extend_path));
            }

            if resolved.exists() {
                return Ok(resolved);
            }

            return Err(TsConfigError::ExtendedConfigNotFound(
                extend_path.to_string(),
                base_dir.to_path_buf(),
            ));
        }

        // Handle node_modules packages
        // Try to find in node_modules
        let mut search_dir = base_dir.to_path_buf();
        loop {
            let node_modules = search_dir.join("node_modules").join(extend_path);

            // Try direct path
            if node_modules.exists() {
                return Ok(node_modules);
            }

            // Try with tsconfig.json
            let with_tsconfig = node_modules.join("tsconfig.json");
            if with_tsconfig.exists() {
                return Ok(with_tsconfig);
            }

            // Try with .json extension
            let with_json = search_dir
                .join("node_modules")
                .join(format!("{}.json", extend_path));
            if with_json.exists() {
                return Ok(with_json);
            }

            // Move up one directory
            if let Some(parent) = search_dir.parent() {
                search_dir = parent.to_path_buf();
            } else {
                break;
            }
        }

        Err(TsConfigError::ExtendedConfigNotFound(
            extend_path.to_string(),
            base_dir.to_path_buf(),
        ))
    }

    /// Resolve relative paths in compiler options
    fn resolve_paths(&self, options: &mut CompilerOptions, base_dir: &Path) {
        // Helper to resolve a path
        let resolve = |p: &Option<PathBuf>| -> Option<PathBuf> {
            p.as_ref().map(|path| {
                if path.is_relative() {
                    base_dir.join(path)
                } else {
                    path.clone()
                }
            })
        };

        options.out_dir = resolve(&options.out_dir);
        options.out_file = resolve(&options.out_file);
        options.root_dir = resolve(&options.root_dir);
        options.base_url = resolve(&options.base_url);
        options.declaration_dir = resolve(&options.declaration_dir);
        options.ts_build_info_file = resolve(&options.ts_build_info_file);

        // Resolve root_dirs
        if let Some(ref dirs) = options.root_dirs {
            options.root_dirs = Some(
                dirs.iter()
                    .map(|d| {
                        if d.is_relative() {
                            base_dir.join(d)
                        } else {
                            d.clone()
                        }
                    })
                    .collect(),
            );
        }

        // Resolve type_roots
        if let Some(ref roots) = options.type_roots {
            options.type_roots = Some(
                roots
                    .iter()
                    .map(|r| {
                        if r.is_relative() {
                            base_dir.join(r)
                        } else {
                            r.clone()
                        }
                    })
                    .collect(),
            );
        }
    }

    /// Resolve files from include/exclude patterns
    fn resolve_files(
        &self,
        raw: &RawTsConfig,
        base_dir: &Path,
        include: &[String],
        exclude: &[String],
    ) -> Result<Vec<PathBuf>, TsConfigError> {
        let mut files = Vec::new();
        let mut seen = HashSet::new();

        // Add explicit files first
        if let Some(ref explicit_files) = raw.files {
            for f in explicit_files {
                let path = base_dir.join(f);
                if !seen.contains(&path) {
                    seen.insert(path.clone());
                    files.push(path);
                }
            }
        }

        // Build exclude patterns
        let exclude_patterns: Vec<glob::Pattern> = exclude
            .iter()
            .filter_map(|pattern| {
                let full_pattern = if pattern.starts_with('/') {
                    base_dir.join(&pattern[1..]).to_string_lossy().into_owned()
                } else {
                    base_dir.join(pattern).to_string_lossy().into_owned()
                };
                glob::Pattern::new(&full_pattern).ok()
            })
            .collect();

        // Process include patterns
        for pattern in include {
            let full_pattern = base_dir.join(pattern).to_string_lossy().into_owned();

            match glob(&full_pattern) {
                Ok(paths) => {
                    for entry in paths.flatten() {
                        // Check if excluded
                        let is_excluded = exclude_patterns
                            .iter()
                            .any(|p| p.matches_path(&entry));

                        if !is_excluded && !seen.contains(&entry) {
                            // Only include TypeScript files
                            if is_typescript_file(&entry) {
                                seen.insert(entry.clone());
                                files.push(entry);
                            }
                        }
                    }
                }
                Err(_) => {
                    return Err(TsConfigError::InvalidGlob(pattern.clone()));
                }
            }
        }

        Ok(files)
    }
}

impl Default for TsConfigParser {
    fn default() -> Self {
        Self::new()
    }
}

/// Check if a file is a TypeScript file
fn is_typescript_file(path: &Path) -> bool {
    match path.extension().and_then(|e| e.to_str()) {
        Some("ts") | Some("tsx") | Some("mts") | Some("cts") => true,
        Some("js") | Some("jsx") | Some("mjs") | Some("cjs") => true, // If allowJs
        _ => false,
    }
}

/// Parse JSON with comments (JSONC)
fn parse_json_with_comments(content: &str) -> Result<RawTsConfig, serde_json::Error> {
    // Strip comments from JSON
    let stripped = strip_json_comments(content);
    serde_json::from_str(&stripped)
}

/// Strip comments from JSON content
fn strip_json_comments(content: &str) -> String {
    let mut result = String::with_capacity(content.len());
    let mut chars = content.chars().peekable();
    let mut in_string = false;
    let mut escape_next = false;

    while let Some(c) = chars.next() {
        if escape_next {
            result.push(c);
            escape_next = false;
            continue;
        }

        if c == '\\' && in_string {
            result.push(c);
            escape_next = true;
            continue;
        }

        if c == '"' && !escape_next {
            in_string = !in_string;
            result.push(c);
            continue;
        }

        if !in_string {
            if c == '/' {
                if let Some(&next) = chars.peek() {
                    if next == '/' {
                        // Single-line comment
                        chars.next();
                        while let Some(&nc) = chars.peek() {
                            if nc == '\n' {
                                break;
                            }
                            chars.next();
                        }
                        continue;
                    } else if next == '*' {
                        // Multi-line comment
                        chars.next();
                        while let Some(nc) = chars.next() {
                            if nc == '*' {
                                if let Some(&'/') = chars.peek() {
                                    chars.next();
                                    break;
                                }
                            }
                        }
                        result.push(' '); // Replace comment with space
                        continue;
                    }
                }
            }

            // Handle trailing commas
            if c == ',' {
                // Look ahead to see if this is a trailing comma
                let mut temp_chars = chars.clone();
                let mut found_value = false;
                while let Some(&nc) = temp_chars.peek() {
                    temp_chars.next();
                    if nc.is_whitespace() {
                        continue;
                    }
                    if nc == '}' || nc == ']' {
                        // This is a trailing comma, skip it
                        continue;
                    }
                    found_value = true;
                    break;
                }
                if found_value {
                    result.push(c);
                }
                continue;
            }
        }

        result.push(c);
    }

    result
}

/// Find a tsconfig.json file by searching up from the given path
pub fn find_config_file(start_path: &Path) -> Option<PathBuf> {
    let mut current = if start_path.is_file() {
        start_path.parent()?.to_path_buf()
    } else {
        start_path.to_path_buf()
    };

    loop {
        let config_path = current.join("tsconfig.json");
        if config_path.exists() {
            return Some(config_path);
        }

        // Also check for jsconfig.json
        let jsconfig_path = current.join("jsconfig.json");
        if jsconfig_path.exists() {
            return Some(jsconfig_path);
        }

        if let Some(parent) = current.parent() {
            current = parent.to_path_buf();
        } else {
            break;
        }
    }

    None
}

/// Parse a tsconfig.json file (convenience function)
pub fn parse_config_file(config_path: &Path) -> Result<ResolvedTsConfig, TsConfigError> {
    let mut parser = TsConfigParser::new();
    parser.parse(config_path)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_strip_single_line_comments() {
        let input = r#"{
            // This is a comment
            "key": "value"
        }"#;

        let stripped = strip_json_comments(input);
        assert!(!stripped.contains("//"));
        assert!(stripped.contains("\"key\""));
    }

    #[test]
    fn test_strip_multi_line_comments() {
        let input = r#"{
            /* Multi
               line
               comment */
            "key": "value"
        }"#;

        let stripped = strip_json_comments(input);
        assert!(!stripped.contains("/*"));
        assert!(!stripped.contains("*/"));
        assert!(stripped.contains("\"key\""));
    }

    #[test]
    fn test_preserve_strings_with_slashes() {
        let input = r#"{"url": "https://example.com"}"#;

        let stripped = strip_json_comments(input);
        assert!(stripped.contains("https://example.com"));
    }

    #[test]
    fn test_is_typescript_file() {
        assert!(is_typescript_file(Path::new("file.ts")));
        assert!(is_typescript_file(Path::new("file.tsx")));
        assert!(is_typescript_file(Path::new("file.mts")));
        assert!(is_typescript_file(Path::new("file.cts")));
        assert!(is_typescript_file(Path::new("file.js")));
        assert!(!is_typescript_file(Path::new("file.txt")));
        assert!(!is_typescript_file(Path::new("file.json")));
    }

    #[test]
    fn test_parse_json_with_comments() {
        let json = r#"{
            // Single line comment
            "compilerOptions": {
                "target": "es2020", // inline comment
                /* block comment */
                "strict": true
            }
        }"#;

        let result: Result<RawTsConfig, _> = parse_json_with_comments(json);
        assert!(result.is_ok());
    }
}
