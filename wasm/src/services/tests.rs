//! Language Service test utilities and integration tests.
//!
//! This module provides test utilities for the language service
//! and contains integration tests.

use std::collections::HashMap;
use std::sync::Arc;

use super::language_service::{
    LanguageService, LanguageServiceHost, LanguageServiceMode,
    CompilerOptions, ResolvedModule, ScriptTarget, ModuleKind,
};
use super::document_registry::ScriptSnapshot;
use super::text_span::TextSpan;

// =============================================================================
// Mock Language Service Host
// =============================================================================

/// A simple in-memory implementation of LanguageServiceHost for testing.
pub struct MockLanguageServiceHost {
    /// Files in the virtual file system.
    files: HashMap<String, MockFile>,
    /// Compiler options.
    options: CompilerOptions,
    /// Current directory.
    current_directory: String,
}

struct MockFile {
    content: String,
    version: u32,
}

impl MockLanguageServiceHost {
    /// Create a new mock host.
    pub fn new() -> Self {
        Self {
            files: HashMap::new(),
            options: CompilerOptions::default(),
            current_directory: "/".to_string(),
        }
    }

    /// Add a file to the virtual file system.
    pub fn add_file(&mut self, name: &str, content: &str) {
        self.files.insert(
            name.to_string(),
            MockFile {
                content: content.to_string(),
                version: 1,
            },
        );
    }

    /// Update a file's content.
    pub fn update_file(&mut self, name: &str, content: &str) {
        if let Some(file) = self.files.get_mut(name) {
            file.content = content.to_string();
            file.version += 1;
        }
    }

    /// Remove a file.
    pub fn remove_file(&mut self, name: &str) {
        self.files.remove(name);
    }

    /// Set compiler options.
    pub fn set_options(&mut self, options: CompilerOptions) {
        self.options = options;
    }
}

impl Default for MockLanguageServiceHost {
    fn default() -> Self {
        Self::new()
    }
}

/// A simple string-based script snapshot.
struct StringSnapshot(String);

impl ScriptSnapshot for StringSnapshot {
    fn get_text(&self) -> String {
        self.0.clone()
    }

    fn get_length(&self) -> u32 {
        self.0.len() as u32
    }

    fn get_change_range(&self, _old_snapshot: &dyn ScriptSnapshot) -> Option<TextChangeRange> {
        None
    }
}

/// Text change range for incremental updates.
pub struct TextChangeRange {
    pub span: TextSpan,
    pub new_length: u32,
}

impl LanguageServiceHost for MockLanguageServiceHost {
    fn get_compilation_settings(&self) -> CompilerOptions {
        self.options.clone()
    }

    fn get_script_file_names(&self) -> Vec<String> {
        self.files.keys().cloned().collect()
    }

    fn get_script_version(&self, file_name: &str) -> String {
        self.files
            .get(file_name)
            .map(|f| f.version.to_string())
            .unwrap_or_else(|| "0".to_string())
    }

    fn get_script_snapshot(&self, file_name: &str) -> Option<Box<dyn ScriptSnapshot>> {
        self.files
            .get(file_name)
            .map(|f| Box::new(StringSnapshot(f.content.clone())) as Box<dyn ScriptSnapshot>)
    }

    fn get_current_directory(&self) -> String {
        self.current_directory.clone()
    }

    fn get_default_lib_file_name(&self, _options: &CompilerOptions) -> String {
        "lib.d.ts".to_string()
    }

    fn read_file(&self, file_name: &str) -> Option<String> {
        self.files.get(file_name).map(|f| f.content.clone())
    }

    fn file_exists(&self, file_name: &str) -> bool {
        self.files.contains_key(file_name)
    }

    fn get_directories(&self, _path: &str) -> Vec<String> {
        Vec::new()
    }

    fn resolve_module_names(
        &self,
        module_names: &[String],
        _containing_file: &str,
    ) -> Vec<Option<ResolvedModule>> {
        module_names
            .iter()
            .map(|name| {
                // Simple resolution: check if file exists
                let file_name = format!("{}.ts", name);
                if self.files.contains_key(&file_name) {
                    Some(ResolvedModule {
                        resolved_file_name: file_name,
                        is_external_library_import: false,
                    })
                } else {
                    None
                }
            })
            .collect()
    }
}

// Send + Sync implementation (required for Arc)
unsafe impl Send for MockLanguageServiceHost {}
unsafe impl Sync for MockLanguageServiceHost {}

// =============================================================================
// Test Helpers
// =============================================================================

/// Create a language service with a single file.
pub fn create_test_service(content: &str) -> (LanguageService, Arc<MockLanguageServiceHost>) {
    let mut host = MockLanguageServiceHost::new();
    host.add_file("/test.ts", content);

    let host = Arc::new(host);
    let service = LanguageService::new(
        host.clone() as Arc<dyn LanguageServiceHost>,
        LanguageServiceMode::Semantic,
    );

    (service, host)
}

/// Create a language service with multiple files.
pub fn create_test_service_with_files(
    files: &[(&str, &str)],
) -> (LanguageService, Arc<MockLanguageServiceHost>) {
    let mut host = MockLanguageServiceHost::new();

    for (name, content) in files {
        host.add_file(name, content);
    }

    let host = Arc::new(host);
    let service = LanguageService::new(
        host.clone() as Arc<dyn LanguageServiceHost>,
        LanguageServiceMode::Semantic,
    );

    (service, host)
}

/// Find the position of a marker in source code.
/// Markers are written as `/*|*/` in the source.
pub fn find_marker_position(source: &str) -> Option<u32> {
    source.find("/*|*/").map(|pos| pos as u32)
}

/// Find all marker positions in source code.
pub fn find_all_marker_positions(source: &str) -> Vec<u32> {
    let mut positions = Vec::new();
    let mut search_start = 0;

    while let Some(pos) = source[search_start..].find("/*|*/") {
        positions.push((search_start + pos) as u32);
        search_start = search_start + pos + 5;
    }

    positions
}

/// Remove markers from source code.
pub fn remove_markers(source: &str) -> String {
    source.replace("/*|*/", "")
}

/// Create test source with a cursor at a position.
pub fn source_with_cursor(source: &str, position: u32) -> (String, u32) {
    let mut result = source.to_string();
    let cursor = "/*|*/";
    result.insert_str(position as usize, cursor);
    (result, position)
}

// =============================================================================
// Integration Tests
// =============================================================================

#[cfg(test)]
mod integration_tests {
    use super::*;

    #[test]
    fn test_mock_host_add_file() {
        let mut host = MockLanguageServiceHost::new();
        host.add_file("/test.ts", "const x = 1;");

        assert!(host.file_exists("/test.ts"));
        assert_eq!(host.get_script_version("/test.ts"), "1");
    }

    #[test]
    fn test_mock_host_update_file() {
        let mut host = MockLanguageServiceHost::new();
        host.add_file("/test.ts", "const x = 1;");
        host.update_file("/test.ts", "const x = 2;");

        assert_eq!(host.get_script_version("/test.ts"), "2");
        assert_eq!(host.read_file("/test.ts"), Some("const x = 2;".to_string()));
    }

    #[test]
    fn test_mock_host_remove_file() {
        let mut host = MockLanguageServiceHost::new();
        host.add_file("/test.ts", "const x = 1;");
        host.remove_file("/test.ts");

        assert!(!host.file_exists("/test.ts"));
    }

    #[test]
    fn test_find_marker_position() {
        let source = "const x = /*|*/1;";
        assert_eq!(find_marker_position(source), Some(10));

        let source_no_marker = "const x = 1;";
        assert_eq!(find_marker_position(source_no_marker), None);
    }

    #[test]
    fn test_find_all_marker_positions() {
        let source = "const x = /*|*/1; const y = /*|*/2;";
        let positions = find_all_marker_positions(source);
        assert_eq!(positions.len(), 2);
        assert_eq!(positions[0], 10);
        assert_eq!(positions[1], 32);
    }

    #[test]
    fn test_remove_markers() {
        let source = "const x = /*|*/1;";
        let cleaned = remove_markers(source);
        assert_eq!(cleaned, "const x = 1;");
    }

    #[test]
    fn test_create_test_service() {
        let (mut service, _host) = create_test_service("const x: number = 1;");

        // The service should be created successfully
        // In a full implementation, we could test various methods
        service.synchronize();
    }

    #[test]
    fn test_create_test_service_with_files() {
        let files = [
            ("/main.ts", "import { foo } from './utils';"),
            ("/utils.ts", "export function foo() {}"),
        ];

        let (mut service, host) = create_test_service_with_files(&files);

        assert_eq!(host.get_script_file_names().len(), 2);
        service.synchronize();
    }
}

// =============================================================================
// Feature Tests
// =============================================================================

#[cfg(test)]
mod feature_tests {
    use super::*;

    // These tests would be more comprehensive with a real parser/checker

    #[test]
    fn test_navigation_bar_items() {
        let (mut service, _) = create_test_service(r#"
            function foo() {}
            class Bar {
                method() {}
            }
        "#);

        // In a full implementation, this would return navigation items
        let items = service.get_navigation_bar_items("/test.ts");
        // Items would be populated if the parser were integrated
    }

    #[test]
    fn test_outlining_spans() {
        let (mut service, _) = create_test_service(r#"
            function foo() {
                // body
            }

            class Bar {
                method() {
                    // method body
                }
            }
        "#);

        let spans = service.get_outlining_spans("/test.ts");
        // Spans would be populated with folding regions
    }

    #[test]
    fn test_formatting() {
        let (mut service, _) = create_test_service("const x=1;");

        let changes = service.get_formatting_edits_for_document(
            "/test.ts",
            Default::default(),
        );
        // Changes would contain formatting edits
    }
}

// =============================================================================
// Benchmarks (if enabled)
// =============================================================================

#[cfg(all(test, feature = "bench"))]
mod benchmarks {
    use super::*;
    use std::time::Instant;

    #[test]
    fn bench_synchronize() {
        let mut host = MockLanguageServiceHost::new();

        // Add many files
        for i in 0..100 {
            host.add_file(
                &format!("/file{}.ts", i),
                &format!("const x{} = {};", i, i),
            );
        }

        let host = Arc::new(host);
        let mut service = LanguageService::new(
            host as Arc<dyn LanguageServiceHost>,
            LanguageServiceMode::Syntactic,
        );

        let start = Instant::now();
        service.synchronize();
        let duration = start.elapsed();

        println!("Synchronize 100 files: {:?}", duration);
    }
}
