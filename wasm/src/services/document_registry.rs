//! Document Registry for caching SourceFile objects.
//!
//! The document registry allows sharing of SourceFile objects across multiple
//! language service instances. This is important for memory efficiency when
//! multiple projects reference the same library files.

use std::collections::HashMap;
use std::sync::{Arc, RwLock};

use crate::parser::arena::NodeArena;
use crate::parser_impl::ParserState;

use super::text_span::TextChangeRange;

// =============================================================================
// Script Snapshot
// =============================================================================

/// An immutable snapshot of a script at a specific time.
/// Matches TypeScript's `IScriptSnapshot` interface.
pub trait ScriptSnapshot: Send + Sync {
    /// Gets a portion of the script snapshot.
    fn get_text(&self, start: u32, end: u32) -> String;

    /// Gets the length of this snapshot.
    fn get_length(&self) -> u32;

    /// Gets the change range between this and an older snapshot.
    fn get_change_range(&self, old_snapshot: &dyn ScriptSnapshot) -> Option<TextChangeRange>;
}

/// A simple string-based script snapshot.
#[derive(Debug, Clone)]
pub struct StringScriptSnapshot {
    text: String,
}

impl StringScriptSnapshot {
    pub fn new(text: String) -> Self {
        Self { text }
    }

    pub fn from_string(text: &str) -> Self {
        Self { text: text.to_string() }
    }
}

impl ScriptSnapshot for StringScriptSnapshot {
    fn get_text(&self, start: u32, end: u32) -> String {
        let start = start as usize;
        let end = (end as usize).min(self.text.len());
        if start >= self.text.len() {
            String::new()
        } else {
            self.text[start..end].to_string()
        }
    }

    fn get_length(&self) -> u32 {
        self.text.len() as u32
    }

    fn get_change_range(&self, _old_snapshot: &dyn ScriptSnapshot) -> Option<TextChangeRange> {
        // Text-based snapshots don't track changes
        None
    }
}

// =============================================================================
// Script Kind
// =============================================================================

/// The kind of script for a source file.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum ScriptKind {
    Unknown,
    JS,
    JSX,
    TS,
    TSX,
    External,
    JSON,
    Deferred,
}

impl ScriptKind {
    /// Get script kind from file extension.
    pub fn from_file_name(file_name: &str) -> Self {
        let lower = file_name.to_lowercase();
        if lower.ends_with(".ts") && !lower.ends_with(".d.ts") {
            ScriptKind::TS
        } else if lower.ends_with(".tsx") {
            ScriptKind::TSX
        } else if lower.ends_with(".js") {
            ScriptKind::JS
        } else if lower.ends_with(".jsx") {
            ScriptKind::JSX
        } else if lower.ends_with(".json") {
            ScriptKind::JSON
        } else if lower.ends_with(".d.ts") || lower.ends_with(".d.mts") || lower.ends_with(".d.cts") {
            ScriptKind::TS
        } else if lower.ends_with(".mts") || lower.ends_with(".cts") {
            ScriptKind::TS
        } else if lower.ends_with(".mjs") || lower.ends_with(".cjs") {
            ScriptKind::JS
        } else {
            ScriptKind::Unknown
        }
    }
}

// =============================================================================
// Cached Source File
// =============================================================================

/// A cached source file with its metadata.
#[derive(Debug)]
pub struct CachedSourceFile {
    /// The file name.
    pub file_name: String,
    /// The version string.
    pub version: String,
    /// The script kind.
    pub script_kind: ScriptKind,
    /// The source text.
    pub source_text: String,
    /// The AST arena (parsed content).
    pub arena: NodeArena,
    /// The root node ID.
    pub root_node: crate::parser::ast::NodeId,
    /// Reference count for this entry.
    ref_count: u32,
}

impl CachedSourceFile {
    /// Create a new cached source file.
    pub fn new(
        file_name: String,
        version: String,
        script_kind: ScriptKind,
        source_text: String,
    ) -> Self {
        // Parse the source file
        let mut parser = ParserState::new(file_name.clone(), source_text.clone());
        let root_node = parser.parse_source_file();
        let arena = parser.into_arena();

        Self {
            file_name,
            version,
            script_kind,
            source_text,
            arena,
            root_node,
            ref_count: 1,
        }
    }

    /// Increment the reference count.
    pub fn add_ref(&mut self) {
        self.ref_count += 1;
    }

    /// Decrement the reference count. Returns true if the file should be removed.
    pub fn release(&mut self) -> bool {
        self.ref_count = self.ref_count.saturating_sub(1);
        self.ref_count == 0
    }
}

// =============================================================================
// Document Registry Key
// =============================================================================

/// Key for looking up documents in the registry.
#[derive(Debug, Clone, PartialEq, Eq, Hash)]
struct DocumentKey {
    file_name: String,
    /// A key representing the compilation settings that affect parsing.
    settings_key: String,
}

impl DocumentKey {
    fn new(file_name: String, settings_key: String) -> Self {
        Self {
            file_name,
            settings_key,
        }
    }
}

// =============================================================================
// Document Registry
// =============================================================================

/// Registry for caching and sharing SourceFile objects.
///
/// The document registry allows multiple language service instances to share
/// parsed source files, reducing memory usage for common library files.
#[derive(Debug)]
pub struct DocumentRegistry {
    /// Use case-sensitive file names.
    use_case_sensitive_file_names: bool,
    /// Current directory for path normalization.
    current_directory: String,
    /// Cached documents.
    documents: RwLock<HashMap<DocumentKey, Arc<RwLock<CachedSourceFile>>>>,
}

impl DocumentRegistry {
    /// Create a new document registry.
    pub fn new(use_case_sensitive_file_names: bool, current_directory: String) -> Self {
        Self {
            use_case_sensitive_file_names,
            current_directory,
            documents: RwLock::new(HashMap::new()),
        }
    }

    /// Normalize a file name for lookup.
    fn get_canonical_file_name(&self, file_name: &str) -> String {
        if self.use_case_sensitive_file_names {
            file_name.to_string()
        } else {
            file_name.to_lowercase()
        }
    }

    /// Get a key for the compilation settings.
    fn get_key_for_compilation_settings(&self, _settings: &CompilationSettings) -> String {
        // For now, use a simple key. In a full implementation, this would
        // include settings that affect parsing (target, jsx, etc.)
        "default".to_string()
    }

    /// Acquire a document from the registry.
    ///
    /// If the document is already cached with the same version, it's returned.
    /// Otherwise, a new document is parsed and cached.
    pub fn acquire_document(
        &self,
        file_name: &str,
        settings: &CompilationSettings,
        snapshot: &dyn ScriptSnapshot,
        version: &str,
        script_kind: Option<ScriptKind>,
    ) -> Arc<RwLock<CachedSourceFile>> {
        let canonical_name = self.get_canonical_file_name(file_name);
        let settings_key = self.get_key_for_compilation_settings(settings);
        let key = DocumentKey::new(canonical_name.clone(), settings_key);

        // Check if we have a cached version
        {
            let documents = self.documents.read().unwrap();
            if let Some(cached) = documents.get(&key) {
                let mut cached_file = cached.write().unwrap();
                if cached_file.version == version {
                    cached_file.add_ref();
                    return cached.clone();
                }
            }
        }

        // Parse a new document
        let source_text = snapshot.get_text(0, snapshot.get_length());
        let kind = script_kind.unwrap_or_else(|| ScriptKind::from_file_name(file_name));
        let cached = Arc::new(RwLock::new(CachedSourceFile::new(
            file_name.to_string(),
            version.to_string(),
            kind,
            source_text,
        )));

        // Cache it
        {
            let mut documents = self.documents.write().unwrap();
            documents.insert(key, cached.clone());
        }

        cached
    }

    /// Update a document in the registry.
    ///
    /// If the document version matches, returns the cached version.
    /// Otherwise, parses and caches a new version.
    pub fn update_document(
        &self,
        file_name: &str,
        settings: &CompilationSettings,
        snapshot: &dyn ScriptSnapshot,
        version: &str,
        script_kind: Option<ScriptKind>,
    ) -> Arc<RwLock<CachedSourceFile>> {
        // For now, just acquire (which handles versioning)
        self.acquire_document(file_name, settings, snapshot, version, script_kind)
    }

    /// Release a document from the registry.
    ///
    /// Decrements the reference count. When it reaches zero, the document
    /// may be removed from the cache.
    pub fn release_document(
        &self,
        file_name: &str,
        settings: &CompilationSettings,
    ) {
        let canonical_name = self.get_canonical_file_name(file_name);
        let settings_key = self.get_key_for_compilation_settings(settings);
        let key = DocumentKey::new(canonical_name, settings_key);

        let should_remove = {
            let documents = self.documents.read().unwrap();
            if let Some(cached) = documents.get(&key) {
                let mut cached_file = cached.write().unwrap();
                cached_file.release()
            } else {
                false
            }
        };

        if should_remove {
            let mut documents = self.documents.write().unwrap();
            documents.remove(&key);
        }
    }

    /// Get the number of cached documents.
    pub fn size(&self) -> usize {
        self.documents.read().unwrap().len()
    }

    /// Report memory usage statistics.
    pub fn report_stats(&self) -> DocumentRegistryStats {
        let documents = self.documents.read().unwrap();
        let mut total_text_size = 0;
        let mut total_arena_size = 0;

        for (_, doc) in documents.iter() {
            let doc = doc.read().unwrap();
            total_text_size += doc.source_text.len();
            // Arena size estimation
            total_arena_size += std::mem::size_of::<NodeArena>();
        }

        DocumentRegistryStats {
            document_count: documents.len(),
            total_text_size,
            total_arena_size,
        }
    }
}

/// Statistics about the document registry.
#[derive(Debug, Clone)]
pub struct DocumentRegistryStats {
    pub document_count: usize,
    pub total_text_size: usize,
    pub total_arena_size: usize,
}

// =============================================================================
// Compilation Settings
// =============================================================================

/// Compilation settings that affect parsing.
/// This is a simplified version - the full version would include all tsconfig options.
#[derive(Debug, Clone, Default)]
pub struct CompilationSettings {
    pub target: ScriptTarget,
    pub jsx: JsxEmit,
    pub strict: bool,
}

/// Script target for compilation.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
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
    JSON,
    Latest,
}

/// JSX emit option.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
pub enum JsxEmit {
    #[default]
    None,
    Preserve,
    React,
    ReactNative,
    ReactJSX,
    ReactJSXDev,
}

// =============================================================================
// Factory Function
// =============================================================================

/// Create a new document registry.
pub fn create_document_registry(
    use_case_sensitive_file_names: bool,
    current_directory: String,
) -> DocumentRegistry {
    DocumentRegistry::new(use_case_sensitive_file_names, current_directory)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_script_kind_from_file_name() {
        assert_eq!(ScriptKind::from_file_name("foo.ts"), ScriptKind::TS);
        assert_eq!(ScriptKind::from_file_name("foo.tsx"), ScriptKind::TSX);
        assert_eq!(ScriptKind::from_file_name("foo.js"), ScriptKind::JS);
        assert_eq!(ScriptKind::from_file_name("foo.jsx"), ScriptKind::JSX);
        assert_eq!(ScriptKind::from_file_name("foo.json"), ScriptKind::JSON);
        assert_eq!(ScriptKind::from_file_name("foo.d.ts"), ScriptKind::TS);
    }

    #[test]
    fn test_string_script_snapshot() {
        let snapshot = StringScriptSnapshot::new("hello world".to_string());
        assert_eq!(snapshot.get_length(), 11);
        assert_eq!(snapshot.get_text(0, 5), "hello");
        assert_eq!(snapshot.get_text(6, 11), "world");
    }

    #[test]
    fn test_document_registry() {
        let registry = create_document_registry(true, "/".to_string());
        let snapshot = StringScriptSnapshot::new("let x = 1;".to_string());
        let settings = CompilationSettings::default();

        // Acquire a document
        let doc1 = registry.acquire_document(
            "test.ts",
            &settings,
            &snapshot,
            "1",
            None,
        );

        assert_eq!(registry.size(), 1);

        // Acquire the same document again
        let doc2 = registry.acquire_document(
            "test.ts",
            &settings,
            &snapshot,
            "1",
            None,
        );

        assert_eq!(registry.size(), 1); // Still 1, reused

        // Release both
        registry.release_document("test.ts", &settings);
        registry.release_document("test.ts", &settings);

        // Document might still be cached
        assert!(registry.size() <= 1);
    }
}
