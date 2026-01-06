//! Project container for multi-file LSP operations.
//!
//! This provides a lightweight home for parsed files, binders, and line maps so
//! LSP features can be extended across multiple files.

use rustc_hash::FxHashMap;

use crate::parser::{NodeIndex, thin_node::ThinNodeArena};
use crate::thin_binder::ThinBinderState;
use crate::thin_parser::ThinParserState;
use crate::lsp::definition::GoToDefinition;
use crate::lsp::references::FindReferences;
use crate::lsp::position::{LineMap, Position, Location};

/// Parsed file state used by LSP features.
pub struct ProjectFile {
    file_name: String,
    root: NodeIndex,
    parser: ThinParserState,
    binder: ThinBinderState,
    line_map: LineMap,
}

impl ProjectFile {
    /// Parse and bind a single source file for LSP queries.
    pub fn new(file_name: String, source_text: String) -> Self {
        let mut parser = ThinParserState::new(file_name.clone(), source_text);
        let root = parser.parse_source_file();
        let arena = parser.get_arena();

        let mut binder = ThinBinderState::new();
        binder.bind_source_file(arena, root);

        let line_map = LineMap::build(parser.get_source_text());

        Self {
            file_name,
            root,
            parser,
            binder,
            line_map,
        }
    }

    /// File name used for LSP locations.
    pub fn file_name(&self) -> &str {
        &self.file_name
    }

    /// Root node of the parsed source file.
    pub fn root(&self) -> NodeIndex {
        self.root
    }

    /// Arena containing parsed ThinNodes.
    pub fn arena(&self) -> &ThinNodeArena {
        self.parser.get_arena()
    }

    /// Binder state for symbol lookup.
    pub fn binder(&self) -> &ThinBinderState {
        &self.binder
    }

    /// Line map for offset <-> position conversions.
    pub fn line_map(&self) -> &LineMap {
        &self.line_map
    }

    /// Original source text for this file.
    pub fn source_text(&self) -> &str {
        self.parser.get_source_text()
    }
}

/// Multi-file container for LSP operations.
pub struct Project {
    files: FxHashMap<String, ProjectFile>,
}

impl Project {
    /// Create a new empty project.
    pub fn new() -> Self {
        Self {
            files: FxHashMap::default(),
        }
    }

    /// Total number of files tracked by the project.
    pub fn file_count(&self) -> usize {
        self.files.len()
    }

    /// Add or replace a file, re-parsing and re-binding its contents.
    pub fn set_file(&mut self, file_name: String, source_text: String) {
        let file = ProjectFile::new(file_name.clone(), source_text);
        self.files.insert(file_name, file);
    }

    /// Remove a file from the project.
    pub fn remove_file(&mut self, file_name: &str) -> Option<ProjectFile> {
        self.files.remove(file_name)
    }

    /// Fetch a file by name.
    pub fn file(&self, file_name: &str) -> Option<&ProjectFile> {
        self.files.get(file_name)
    }

    /// Go to definition within a single file.
    pub fn get_definition(&self, file_name: &str, position: Position) -> Option<Vec<Location>> {
        let file = self.files.get(file_name)?;
        let goto_def = GoToDefinition::new(
            file.arena(),
            file.binder(),
            file.line_map(),
            file.file_name().to_string(),
            file.source_text(),
        );
        goto_def.get_definition(file.root(), position)
    }

    /// Find references within a single file.
    pub fn find_references(&self, file_name: &str, position: Position) -> Option<Vec<Location>> {
        let file = self.files.get(file_name)?;
        let find_refs = FindReferences::new(
            file.arena(),
            file.binder(),
            file.line_map(),
            file.file_name().to_string(),
            file.source_text(),
        );
        find_refs.find_references(file.root(), position)
    }
}
