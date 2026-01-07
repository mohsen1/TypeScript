//! ThinEmitter - Emitter using ThinNodeArena
//!
//! This emitter uses the ThinNode architecture for cache-optimized AST access.
//! It works directly with ThinNodeArena instead of the old Node enum.
//!
//! # Architecture
//!
//! - Uses ThinNodeArena for AST access (16-byte nodes, 13x cache improvement)
//! - Dispatches based on ThinNode.kind (u16)
//! - Uses accessor methods to get typed node data
//!
//! # Module Organization
//!
//! The emitter is organized as a directory module:
//! - `mod.rs` - Core ThinPrinter struct, dispatch logic, and emit methods
//! - Future: emit methods can be split into expressions.rs, statements.rs, declarations.rs
//!
//! Note: pub(super) fields and methods allow future submodules to access ThinPrinter internals.

// Allow dead code for emitter infrastructure methods that will be used in future phases
#![allow(dead_code)]

use crate::emit_context::EmitContext;
use crate::parser::{NodeIndex, NodeList};
use crate::parser::thin_node::{ThinNode, ThinNodeArena};
use crate::parser::syntax_kind_ext;
use crate::scanner::SyntaxKind;
use crate::source_writer::SourceWriter;
use crate::transform_context::TransformDirective;
use crate::transform_context::TransformContext;
use crate::transforms::class_es5::ClassES5Emitter;
use crate::transforms::arrow_es5::contains_this_reference;

// =============================================================================
// Comment Utilities
// =============================================================================

/// Represents a comment range in the source text.
#[derive(Debug, Clone, Copy)]
pub struct CommentRange {
    pub pos: u32,
    pub end: u32,
    pub kind: CommentKind,
    pub has_trailing_newline: bool,
}

/// Kind of comment.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum CommentKind {
    SingleLine,  // // comment
    MultiLine,   // /* comment */
}

/// Check if a character is a line break.
fn is_line_break(ch: char) -> bool {
    ch == '\n' || ch == '\r' || ch == '\u{2028}' || ch == '\u{2029}'
}

/// Check if a character is whitespace (but not a line break).
fn is_whitespace_single_line(ch: char) -> bool {
    ch == ' ' || ch == '\t' || ch == '\u{000B}' || ch == '\u{000C}'
}

/// UTF-8 safe helper to get the character at a byte position.
/// Returns None if pos is out of bounds or not on a char boundary.
fn char_at(text: &str, pos: usize) -> Option<char> {
    if pos >= text.len() {
        return None;
    }
    text[pos..].chars().next()
}

/// Get trailing comments starting at a position in the source text.
/// Trailing comments are comments that appear on the same line after a token,
/// before a newline.
pub fn get_trailing_comment_ranges(text: &str, pos: usize) -> Vec<CommentRange> {
    let mut comments = Vec::new();
    let len = text.len();
    let mut i = pos;

    // Scan for trailing comments (on the same line, before newline)
    while i < len {
        let ch = char_at(text, i).unwrap_or('\0');
        let char_len = ch.len_utf8();

        // Skip whitespace (but not newlines)
        if is_whitespace_single_line(ch) {
            i += char_len;
            continue;
        }

        // Stop at newline - trailing comments end here
        if is_line_break(ch) {
            break;
        }

        // Check for comment start (/ is ASCII, safe to check byte directly)
        if ch == '/' && i + 1 < len {
            let next_byte = text.as_bytes()[i + 1];

            if next_byte == b'/' {
                // Single-line comment: // ...
                let start = i;
                i += 2;
                while i < len {
                    let c = char_at(text, i).unwrap_or('\0');
                    if is_line_break(c) {
                        break;
                    }
                    i += c.len_utf8();
                }
                comments.push(CommentRange {
                    pos: start as u32,
                    end: i as u32,
                    kind: CommentKind::SingleLine,
                    has_trailing_newline: i < len && is_line_break(char_at(text, i).unwrap_or('\0')),
                });
                continue;
            } else if next_byte == b'*' {
                // Multi-line comment: /* ... */
                let start = i;
                i += 2;
                let mut has_newline = false;
                while i + 1 < len {
                    let c = char_at(text, i).unwrap_or('\0');
                    if c == '*' && text.as_bytes()[i + 1] == b'/' {
                        i += 2;
                        break;
                    }
                    if is_line_break(c) {
                        has_newline = true;
                    }
                    i += c.len_utf8();
                }
                // For trailing comments, we stop after the first multi-line comment
                // if it spans multiple lines
                comments.push(CommentRange {
                    pos: start as u32,
                    end: i as u32,
                    kind: CommentKind::MultiLine,
                    has_trailing_newline: has_newline,
                });
                if has_newline {
                    break;
                }
                continue;
            }
        }

        // Non-whitespace, non-comment character - stop scanning
        break;
    }

    comments
}

/// Get leading comments before a position in the source text.
/// Leading comments are comments that appear before a token,
/// potentially on preceding lines.
pub fn get_leading_comment_ranges(text: &str, pos: usize) -> Vec<CommentRange> {
    let mut comments = Vec::new();
    let len = text.len();
    let mut i = pos;

    // Skip shebang at the start of file
    if i == 0 && len >= 2 && text.as_bytes()[0] == b'#' && text.as_bytes()[1] == b'!' {
        while i < len {
            let c = char_at(text, i).unwrap_or('\0');
            if is_line_break(c) {
                break;
            }
            i += c.len_utf8();
        }
    }

    // Scan for leading comments
    let mut pending: Option<CommentRange> = None;

    while i < len {
        let ch = char_at(text, i).unwrap_or('\0');
        let char_len = ch.len_utf8();

        // Skip whitespace
        if is_whitespace_single_line(ch) {
            i += char_len;
            continue;
        }

        // Handle newlines - they mark comment boundaries
        if is_line_break(ch) {
            i += char_len;
            // Skip \r\n as a single newline
            if ch == '\r' && i < len && text.as_bytes()[i] == b'\n' {
                i += 1;
            }
            if let Some(mut p) = pending.take() {
                p.has_trailing_newline = true;
                comments.push(p);
            }
            continue;
        }

        // Check for comment start (/ is ASCII, safe to check byte directly)
        if ch == '/' && i + 1 < len {
            let next_byte = text.as_bytes()[i + 1];

            if next_byte == b'/' {
                // Emit any pending comment first
                if let Some(p) = pending.take() {
                    comments.push(p);
                }
                // Single-line comment
                let start = i;
                i += 2;
                while i < len {
                    let c = char_at(text, i).unwrap_or('\0');
                    if is_line_break(c) {
                        break;
                    }
                    i += c.len_utf8();
                }
                pending = Some(CommentRange {
                    pos: start as u32,
                    end: i as u32,
                    kind: CommentKind::SingleLine,
                    has_trailing_newline: false,
                });
                continue;
            } else if next_byte == b'*' {
                // Emit any pending comment first
                if let Some(p) = pending.take() {
                    comments.push(p);
                }
                // Multi-line comment
                let start = i;
                i += 2;
                while i + 1 < len {
                    if text.as_bytes()[i] == b'*' && text.as_bytes()[i + 1] == b'/' {
                        i += 2;
                        break;
                    }
                    let c = char_at(text, i).unwrap_or('\0');
                    i += c.len_utf8();
                }
                pending = Some(CommentRange {
                    pos: start as u32,
                    end: i as u32,
                    kind: CommentKind::MultiLine,
                    has_trailing_newline: false,
                });
                continue;
            }
        }

        // Non-whitespace, non-comment - we're done
        break;
    }

    // Emit final pending comment
    if let Some(p) = pending {
        comments.push(p);
    }

    comments
}

// =============================================================================
// Emitter Options
// =============================================================================

/// ECMAScript target version.
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
pub enum ScriptTarget {
    ES3 = 0,
    ES5 = 1,
    ES2015 = 2,
    ES2016 = 3,
    ES2017 = 4,
    ES2018 = 5,
    ES2019 = 6,
    ES2020 = 7,
    ES2021 = 8,
    ES2022 = 9,
    #[default]
    ESNext = 99,
}

/// Module system kind.
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
pub enum ModuleKind {
    #[default]
    None = 0,
    CommonJS = 1,
    AMD = 2,
    UMD = 3,
    System = 4,
    ES2015 = 5,
    ES2020 = 6,
    ES2022 = 7,
    ESNext = 99,
    Node16 = 100,
    NodeNext = 199,
}

/// New line kind.
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
pub enum NewLineKind {
    #[default]
    LineFeed = 0,
    CarriageReturnLineFeed = 1,
}

/// Printer configuration options.
#[derive(Clone, Debug)]
pub struct PrinterOptions {
    /// Remove comments from output
    pub remove_comments: bool,
    /// Target ECMAScript version
    pub target: ScriptTarget,
    /// Use single quotes for strings
    pub single_quote: bool,
    /// Omit trailing semicolons
    pub omit_trailing_semicolon: bool,
    /// Don't emit helpers
    pub no_emit_helpers: bool,
    /// Module kind
    pub module: ModuleKind,
    /// New line character
    pub new_line: NewLineKind,
}

impl Default for PrinterOptions {
    fn default() -> Self {
        PrinterOptions {
            remove_comments: false,
            target: ScriptTarget::ESNext,
            single_quote: false,
            omit_trailing_semicolon: false,
            no_emit_helpers: false,
            module: ModuleKind::None,
            new_line: NewLineKind::LineFeed,
        }
    }
}

#[derive(Default)]
struct ParamTransformPlan {
    params: Vec<ParamTransform>,
    rest: Option<RestParamTransform>,
}

impl ParamTransformPlan {
    fn has_transforms(&self) -> bool {
        !self.params.is_empty() || self.rest.is_some()
    }
}

struct ParamTransform {
    name: String,
    pattern: Option<NodeIndex>,
    initializer: Option<NodeIndex>,
}

struct RestParamTransform {
    name: String,
    pattern: Option<NodeIndex>,
    index: usize,
}

// =============================================================================
// ThinPrinter
// =============================================================================

/// Printer that works with ThinNodeArena.
///
/// Uses SourceWriter for output generation (enables source map support).
/// Uses EmitContext for transform-specific state management.
/// Uses TransformContext for directive-based transforms (Phase 2 architecture).
pub struct ThinPrinter<'a> {
    /// The ThinNodeArena containing the AST.
    pub(super) arena: &'a ThinNodeArena,

    /// Source writer for output generation and source map tracking
    pub(super) writer: SourceWriter,

    /// Emit context holding options and transform state
    pub(super) ctx: EmitContext,

    /// Transform directives from lowering pass (optional, defaults to empty)
    pub(super) transforms: TransformContext,

    /// Source text for detecting single-line constructs
    pub(super) source_text: Option<&'a str>,

    /// Last processed position in source text for comment gap detection
    pub(super) last_processed_pos: u32,
}

impl<'a> ThinPrinter<'a> {
    /// Create a new ThinPrinter.
    pub fn new(arena: &'a ThinNodeArena) -> Self {
        Self::with_options(arena, PrinterOptions::default())
    }

    /// Create a new ThinPrinter with pre-allocated output capacity
    /// This reduces allocations when the expected output size is known (e.g., ~1.5x source size)
    pub fn with_capacity(arena: &'a ThinNodeArena, capacity: usize) -> Self {
        Self::with_capacity_and_options(arena, capacity, PrinterOptions::default())
    }

    /// Create a new ThinPrinter with options.
    pub fn with_options(arena: &'a ThinNodeArena, options: PrinterOptions) -> Self {
        Self::with_capacity_and_options(arena, 1024, options)
    }

    /// Create a new ThinPrinter with pre-allocated capacity and options.
    pub fn with_capacity_and_options(arena: &'a ThinNodeArena, capacity: usize, options: PrinterOptions) -> Self {
        let mut writer = SourceWriter::with_capacity(capacity);
        writer.set_new_line_kind(options.new_line);

        // Create EmitContext with ES5 targeting by default for baseline compatibility
        let mut ctx = EmitContext::with_options(options);
        ctx.target_es5 = true;

        ThinPrinter {
            arena,
            writer,
            ctx,
            transforms: TransformContext::new(), // Empty by default, can be set later
            source_text: None,
            last_processed_pos: 0,
        }
    }

    /// Create a new ThinPrinter with transform directives.
    /// This is the Phase 2 constructor that accepts pre-computed transforms.
    pub fn with_transforms(arena: &'a ThinNodeArena, transforms: TransformContext) -> Self {
        let mut printer = Self::new(arena);
        printer.transforms = transforms;
        printer
    }

    /// Create a new ThinPrinter with transforms and options.
    pub fn with_transforms_and_options(
        arena: &'a ThinNodeArena,
        transforms: TransformContext,
        options: PrinterOptions,
    ) -> Self {
        let mut printer = Self::with_options(arena, options);
        printer.transforms = transforms;
        printer
    }

    /// Create a new ThinPrinter targeting ES5.
    pub fn new_es5(arena: &'a ThinNodeArena) -> Self {
        let mut printer = Self::new(arena);
        printer.ctx.target_es5 = true;
        printer
    }

    /// Create a new ThinPrinter targeting ES6+.
    pub fn new_es6(arena: &'a ThinNodeArena) -> Self {
        let mut printer = Self::new(arena);
        printer.ctx.target_es5 = false;
        printer
    }

    /// Set whether to target ES5 (classes→IIFEs, arrows→functions).
    pub fn set_target_es5(&mut self, es5: bool) {
        self.ctx.target_es5 = es5;
    }

    /// Set the module kind (CommonJS, ESM, etc.).
    pub fn set_module_kind(&mut self, kind: ModuleKind) {
        self.ctx.options.module = kind;
    }

    /// Set auto-detect module mode. When enabled, the emitter will detect if
    /// the source file contains import/export statements and apply CommonJS
    /// transforms automatically.
    pub fn set_auto_detect_module(&mut self, enabled: bool) {
        self.ctx.auto_detect_module = enabled;
    }

    /// Set the source text (for detecting single-line constructs).
    pub fn set_source_text(&mut self, text: &'a str) {
        self.source_text = Some(text);
    }

    /// Check if a node spans a single line in the source.
    /// For blocks like `{ }`, we look for the closing `}` and check if there's a newline
    /// between the opening `{` and the first `}`.
    fn is_single_line(&self, node: &ThinNode) -> bool {
        if let Some(text) = self.source_text {
            let start = node.pos as usize;
            if start < text.len() {
                // Find the first closing brace after the opening
                // For a block, the source starts with `{` and we want to find the matching `}`
                let slice = &text[start..];
                if let Some(close_idx) = slice.find('}') {
                    // Check if there's a newline between `{` and `}`
                    let inner = &slice[..close_idx + 1];
                    return !inner.contains('\n');
                }
            }
        }
        // Default to multi-line if we can't determine
        false
    }

    /// Get the output.
    pub fn get_output(&self) -> &str {
        self.writer.get_output()
    }

    /// Take the output.
    pub fn take_output(self) -> String {
        self.writer.take_output()
    }

    // =========================================================================
    // Comment Emission Helpers
    // =========================================================================

    /// Emit trailing comments after a node's end position.
    /// Note: In TypeScript's AST, node.end often includes trailing trivia (including comments).
    /// We clamp the position to valid bounds and scan from there.
    fn emit_trailing_comments(&mut self, end_pos: u32) {
        if self.ctx.options.remove_comments {
            return;
        }

        let Some(text) = self.source_text else {
            return;
        };

        // Clamp position to valid range
        let pos = std::cmp::min(end_pos as usize, text.len());
        let comments = get_trailing_comment_ranges(text, pos);
        for comment in comments {
            // Add space before trailing comment
            self.write_space();
            // Emit the comment text
            let comment_text = &text[comment.pos as usize..comment.end as usize];
            self.write(comment_text);
        }
    }

    /// Emit leading comments before a node's start position.
    fn emit_leading_comments(&mut self, pos: u32) {
        if self.ctx.options.remove_comments {
            return;
        }

        let Some(text) = self.source_text else {
            return;
        };

        let comments = get_leading_comment_ranges(text, pos as usize);
        for comment in comments {
            let comment_text = &text[comment.pos as usize..comment.end as usize];
            self.write(comment_text);
            if comment.has_trailing_newline {
                self.write_line();
            } else if comment.kind == CommentKind::MultiLine {
                self.write_space();
            }
        }
    }

    /// Emit comments in the gap between last_processed_pos and the given position.
    /// This handles comments that appear between AST nodes.
    fn emit_comments_in_gap(&mut self, up_to_pos: u32) {
        if self.ctx.options.remove_comments {
            return;
        }

        let Some(text) = self.source_text else {
            return;
        };

        // Scan for comments between last_processed_pos and up_to_pos
        let start = self.last_processed_pos as usize;
        let end = std::cmp::min(up_to_pos as usize, text.len());

        if start >= end {
            return;
        }

        // Scan the gap for comments
        let gap_text = &text[start..end];
        let bytes = gap_text.as_bytes();
        let len = bytes.len();
        let mut pos = 0;

        while pos < len {
            let ch = bytes[pos];

            // Skip whitespace
            if ch == b' ' || ch == b'\t' || ch == b'\r' || ch == b'\n' {
                pos += 1;
                continue;
            }

            // Check for comment start
            if ch == b'/' && pos + 1 < len {
                let next = bytes[pos + 1];

                if next == b'/' {
                    // Single-line comment
                    let comment_start = start + pos;
                    let mut comment_end = pos + 2;
                    while comment_end < len && bytes[comment_end] != b'\n' && bytes[comment_end] != b'\r' {
                        comment_end += 1;
                    }
                    let comment_text = &text[comment_start..start + comment_end];
                    self.write(comment_text);
                    self.write_line();

                    // Skip past the comment and newline
                    pos = comment_end;
                    if pos < len && bytes[pos] == b'\r' {
                        pos += 1;
                    }
                    if pos < len && bytes[pos] == b'\n' {
                        pos += 1;
                    }
                    continue;
                } else if next == b'*' {
                    // Multi-line comment
                    let comment_start = start + pos;
                    let mut comment_end = pos + 2;
                    while comment_end + 1 < len {
                        if bytes[comment_end] == b'*' && bytes[comment_end + 1] == b'/' {
                            comment_end += 2;
                            break;
                        }
                        comment_end += 1;
                    }
                    let comment_text = &text[comment_start..start + comment_end];
                    self.write(comment_text);
                    self.write_line();

                    pos = comment_end;
                    continue;
                }
            }

            // Hit non-whitespace, non-comment content - stop scanning
            break;
        }
    }

    // =========================================================================
    // Output Helpers (delegate to SourceWriter)
    // pub(super) for access from submodules (expressions, statements, declarations)
    // =========================================================================

    /// Write text to output.
    pub(super) fn write(&mut self, text: &str) {
        self.writer.write(text);
    }

    /// Write a single character.
    pub(super) fn write_char(&mut self, ch: char) {
        self.writer.write_char(ch);
    }

    /// Write a newline.
    pub(super) fn write_line(&mut self) {
        self.writer.write_line();
    }

    /// Write a space.
    pub(super) fn write_space(&mut self) {
        self.writer.write_space();
    }

    /// Write a semicolon (respecting options).
    pub(super) fn write_semicolon(&mut self) {
        if !self.ctx.options.omit_trailing_semicolon {
            self.write(";");
        }
    }

    /// Increase indentation.
    pub(super) fn increase_indent(&mut self) {
        self.writer.increase_indent();
    }

    /// Decrease indentation.
    pub(super) fn decrease_indent(&mut self) {
        self.writer.decrease_indent();
    }

    // =========================================================================
    // Transform Application (Phase 2 Architecture)
    // =========================================================================

    /// Apply a transform directive to a node.
    /// This is called when a node has an entry in the TransformContext.
    fn apply_transform(&mut self, node: &ThinNode, idx: NodeIndex) {
        use crate::transform_context::TransformDirective;

        // Clone the directive to avoid borrow checker issues
        let Some(directive) = self.transforms.get(idx).cloned() else {
            // No transform, emit normally (should not happen if has_transform returned true)
            self.emit_node_default(node, idx);
            return;
        };

        match directive {
            TransformDirective::Identity => {
                // No transformation needed, emit as-is
                self.emit_node_default(node, idx);
            }

            TransformDirective::ES5Class { class_node, .. } => {
                // Delegate to existing ClassES5Emitter
                let mut es5_emitter = ClassES5Emitter::new(self.arena);
                es5_emitter.set_indent_level(self.writer.indent_level());
                if let Some(source_text) = self.source_text {
                    es5_emitter.set_source_text(source_text);
                }
                let es5_output = es5_emitter.emit_class(class_node);
                self.write(&es5_output);
            }

            TransformDirective::CommonJSExport {
                names,
                is_default,
                inner,
            } => {
                let export_name = names.first().map(|name| name.as_str());
                self.emit_commonjs_export(names.as_slice(), is_default, |this| {
                    this.emit_commonjs_inner(node, idx, &inner, export_name);
                });
            }

            TransformDirective::ES5ArrowFunction { arrow_node, .. } => {
                if let Some(arrow_node) = self.arena.get(arrow_node) {
                    if let Some(func) = self.arena.get_function(arrow_node) {
                        self.emit_arrow_function_es5(arrow_node, func);
                        return;
                    }
                }

                self.emit_node_default(node, idx);
            }

            TransformDirective::ES5AsyncFunction { function_node } => {
                if let Some(func_node) = self.arena.get(function_node) {
                    if let Some(func) = self.arena.get_function(func_node) {
                        let is_exported = self.ctx.is_commonjs()
                            && self.has_export_modifier(&func.modifiers)
                            && !self.ctx.module_state.has_export_assignment;
                        let is_default = self.has_default_modifier(&func.modifiers);

                        let func_name = if !func.name.is_none() {
                            self.get_identifier_text_idx(func.name)
                        } else {
                            String::new()
                        };

                        self.emit_async_function_es5(func, &func_name, is_exported, is_default);
                        return;
                    }
                }

                self.emit_node_default(node, idx);
            }

            TransformDirective::ES5ForOf { for_of_node } => {
                if let Some(for_of_node) = self.arena.get(for_of_node) {
                    if let Some(for_in_of) = self.arena.get_for_in_of(for_of_node) {
                        if !for_in_of.await_modifier {
                            self.emit_for_of_statement_es5(for_in_of);
                            return;
                        }
                    }
                }

                self.emit_node_default(node, idx);
            }

            TransformDirective::ModuleWrapper {
                format,
                dependencies,
                ..
            } => {
                if let Some(source) = self.arena.get_source_file(node) {
                    self.emit_module_wrapper(&format, &dependencies, node, source);
                    return;
                }

                self.emit_node_default(node, idx);
            }

            TransformDirective::Chain(directives) => {
                self.emit_chained_directives(node, idx, directives);
            }
        }
    }

    fn emit_commonjs_export<F>(&mut self, names: &[String], is_default: bool, mut emit_inner: F)
    where
        F: FnMut(&mut Self),
    {
        if names.is_empty() {
            emit_inner(self);
            return;
        }

        let prev_module = self.ctx.options.module;
        self.ctx.options.module = ModuleKind::None;

        emit_inner(self);

        self.ctx.options.module = prev_module;

        self.write_line();
        if is_default {
            self.write("exports.default = ");
            self.write(&names[0]);
            self.write(";");
        } else {
            for (i, name) in names.iter().enumerate() {
                if i > 0 {
                    self.write_line();
                }
                self.write("exports.");
                self.write(name);
                self.write(" = ");
                self.write(name);
                self.write(";");
            }
        }
    }

    fn emit_commonjs_inner(
        &mut self,
        node: &ThinNode,
        idx: NodeIndex,
        inner: &TransformDirective,
        export_name: Option<&str>,
    ) {
        match inner {
            TransformDirective::ES5Class { class_node, .. } => {
                let mut es5_emitter = ClassES5Emitter::new(self.arena);
                es5_emitter.set_indent_level(self.writer.indent_level());
                if let Some(source_text) = self.source_text {
                    es5_emitter.set_source_text(source_text);
                }
                let es5_output = es5_emitter.emit_class(*class_node);
                self.write(&es5_output);
            }
            TransformDirective::ES5AsyncFunction { function_node } => {
                if let Some(func_node) = self.arena.get(*function_node) {
                    if let Some(func) = self.arena.get_function(func_node) {
                        if !func.name.is_none() {
                            let func_name = self.get_identifier_text_idx(func.name);
                            self.emit_async_function_es5(func, &func_name, false, false);
                        } else {
                            self.emit_async_function_es5(
                                func,
                                export_name.unwrap_or(""),
                                false,
                                false,
                            );
                        }
                    }
                }
            }
            TransformDirective::ES5ArrowFunction { arrow_node, .. } => {
                if let Some(arrow_node) = self.arena.get(*arrow_node) {
                    if let Some(func) = self.arena.get_function(arrow_node) {
                        self.emit_arrow_function_es5(arrow_node, func);
                    }
                }
            }
            TransformDirective::Identity => {
                self.emit_node_default(node, idx);
            }
            TransformDirective::Chain(directives) => {
                self.emit_chained_directives(node, idx, directives.clone());
            }
            _ => {
                self.emit_node_default(node, idx);
            }
        }
    }

    fn emit_chained_directives(
        &mut self,
        node: &ThinNode,
        idx: NodeIndex,
        directives: Vec<TransformDirective>,
    ) {
        let mut flattened = Vec::new();
        Self::flatten_chain(directives, &mut flattened);

        if flattened.is_empty() {
            self.emit_node_default(node, idx);
            return;
        }

        let last = flattened.len() - 1;
        self.emit_chained_directive(node, idx, &flattened, last);
    }

    fn emit_chained_directive(
        &mut self,
        node: &ThinNode,
        idx: NodeIndex,
        directives: &[TransformDirective],
        index: usize,
    ) {
        let directive = &directives[index];
        match directive {
            TransformDirective::Identity => {
                self.emit_chained_previous(node, idx, directives, index);
            }
            TransformDirective::ES5Class { class_node, .. } => {
                let mut es5_emitter = ClassES5Emitter::new(self.arena);
                es5_emitter.set_indent_level(self.writer.indent_level());
                if let Some(source_text) = self.source_text {
                    es5_emitter.set_source_text(source_text);
                }
                let es5_output = es5_emitter.emit_class(*class_node);
                self.write(&es5_output);
            }
            TransformDirective::CommonJSExport {
                names,
                is_default,
                inner,
            } => {
                let export_name = names.first().map(|name| name.as_str());
                self.emit_commonjs_export(names.as_slice(), *is_default, |this| {
                    if index == 0 {
                        this.emit_commonjs_inner(node, idx, inner, export_name);
                    } else {
                        this.emit_chained_directive(node, idx, directives, index - 1);
                    }
                });
            }
            TransformDirective::ES5ArrowFunction { arrow_node, .. } => {
                if let Some(arrow_node) = self.arena.get(*arrow_node) {
                    if let Some(func) = self.arena.get_function(arrow_node) {
                        self.emit_arrow_function_es5(arrow_node, func);
                        return;
                    }
                }

                self.emit_chained_previous(node, idx, directives, index);
            }
            TransformDirective::ES5AsyncFunction { function_node } => {
                if let Some(func_node) = self.arena.get(*function_node) {
                    if let Some(func) = self.arena.get_function(func_node) {
                        let is_exported = self.ctx.is_commonjs()
                            && self.has_export_modifier(&func.modifiers)
                            && !self.ctx.module_state.has_export_assignment;
                        let is_default = self.has_default_modifier(&func.modifiers);

                        let func_name = if !func.name.is_none() {
                            self.get_identifier_text_idx(func.name)
                        } else {
                            String::new()
                        };

                        self.emit_async_function_es5(func, &func_name, is_exported, is_default);
                        return;
                    }
                }

                self.emit_chained_previous(node, idx, directives, index);
            }
            TransformDirective::ES5ForOf { for_of_node } => {
                if let Some(for_of_node) = self.arena.get(*for_of_node) {
                    if let Some(for_in_of) = self.arena.get_for_in_of(for_of_node) {
                        if !for_in_of.await_modifier {
                            self.emit_for_of_statement_es5(for_in_of);
                            return;
                        }
                    }
                }

                self.emit_chained_previous(node, idx, directives, index);
            }
            TransformDirective::ModuleWrapper {
                format,
                dependencies,
                ..
            } => {
                if let Some(source) = self.arena.get_source_file(node) {
                    self.emit_module_wrapper(&format, dependencies.as_slice(), node, source);
                    return;
                }

                self.emit_chained_previous(node, idx, directives, index);
            }
            TransformDirective::Chain(nested) => {
                self.emit_chained_directives(node, idx, nested.clone());
            }
        }
    }

    fn emit_chained_previous(
        &mut self,
        node: &ThinNode,
        idx: NodeIndex,
        directives: &[TransformDirective],
        index: usize,
    ) {
        if index == 0 {
            self.emit_node_default(node, idx);
        } else {
            self.emit_chained_directive(node, idx, directives, index - 1);
        }
    }

    fn flatten_chain(
        directives: Vec<TransformDirective>,
        out: &mut Vec<TransformDirective>,
    ) {
        for directive in directives {
            match directive {
                TransformDirective::Chain(inner) => {
                    Self::flatten_chain(inner, out);
                }
                other => out.push(other),
            }
        }
    }

    /// Emit a node using default logic (no transforms).
    /// This is the old emit_node logic extracted for reuse.
    fn emit_node_default(&mut self, node: &ThinNode, idx: NodeIndex) {
        // This will be populated by moving the match statement from emit_node
        // For now, just recursively call emit_node which will use the match
        // We'll refactor this properly in the next step
        let kind = node.kind;
        self.emit_node_by_kind(node, idx, kind);
    }

    fn emit_module_wrapper(
        &mut self,
        format: &crate::transform_context::ModuleFormat,
        dependencies: &[String],
        source_node: &ThinNode,
        source: &crate::parser::thin_node::SourceFileData,
    ) {
        match format {
            crate::transform_context::ModuleFormat::AMD => {
                self.emit_amd_wrapper(dependencies, source_node);
            }
            crate::transform_context::ModuleFormat::UMD => {
                self.emit_umd_wrapper(source_node);
            }
            crate::transform_context::ModuleFormat::System => {
                self.emit_system_wrapper(dependencies, source_node);
            }
            _ => {
                for &stmt_idx in &source.statements.nodes {
                    self.emit(stmt_idx);
                    self.write_line();
                }
            }
        }
    }

    fn emit_amd_wrapper(
        &mut self,
        dependencies: &[String],
        source_node: &ThinNode,
    ) {
        use crate::transforms::module_commonjs;

        self.write("define([\"require\", \"exports\"");
        for dep in dependencies {
            self.write(", \"");
            self.write(dep);
            self.write("\"");
        }
        self.write("], function (require, exports");
        for dep in dependencies {
            let name = module_commonjs::sanitize_module_name(dep);
            self.write(", ");
            self.write(&name);
        }
        self.write(") {");
        self.write_line();
        self.increase_indent();

        self.emit_module_wrapper_body(source_node);

        self.decrease_indent();
        self.write("});");
    }

    fn emit_umd_wrapper(
        &mut self,
        source_node: &ThinNode,
    ) {
        self.write("(function (factory) {");
        self.write_line();
        self.increase_indent();
        self.write("if (typeof module === \"object\" && typeof module.exports === \"object\") {");
        self.write_line();
        self.increase_indent();
        self.write("var v = factory(require, exports);");
        self.write_line();
        self.write("if (v !== undefined) module.exports = v;");
        self.write_line();
        self.decrease_indent();
        self.write("}");
        self.write_line();
        self.write("else if (typeof define === \"function\" && define.amd) {");
        self.write_line();
        self.increase_indent();
        self.write("define([\"require\", \"exports\"], factory);");
        self.write_line();
        self.decrease_indent();
        self.write("}");
        self.write_line();
        self.decrease_indent();
        self.write("})(function (require, exports) {");
        self.write_line();
        self.increase_indent();

        self.emit_module_wrapper_body(source_node);

        self.decrease_indent();
        self.write("});");
    }

    fn emit_system_wrapper(
        &mut self,
        dependencies: &[String],
        source_node: &ThinNode,
    ) {
        self.write("System.register([");
        for (i, dep) in dependencies.iter().enumerate() {
            if i > 0 {
                self.write(", ");
            }
            self.write("\"");
            self.write(dep);
            self.write("\"");
        }
        self.write("], function (exports_1, context_1) {");
        self.write_line();
        self.increase_indent();
        self.write("return {");
        self.write_line();
        self.increase_indent();
        self.write("setters: [],");
        self.write_line();
        self.write("execute: function () {");
        self.write_line();
        self.increase_indent();

        self.emit_module_wrapper_body(source_node);

        self.decrease_indent();
        self.write("}");
        self.write_line();
        self.decrease_indent();
        self.write("};");
        self.write_line();
        self.decrease_indent();
        self.write("});");
    }

    fn emit_module_wrapper_body(&mut self, source_node: &ThinNode) {
        let prev_module = self.ctx.options.module;
        let prev_auto_detect = self.ctx.auto_detect_module;

        self.ctx.options.module = ModuleKind::CommonJS;
        self.ctx.auto_detect_module = false;

        self.emit_source_file(source_node);

        self.ctx.options.module = prev_module;
        self.ctx.auto_detect_module = prev_auto_detect;
    }

    // =========================================================================
    // Main Emit Method
    // =========================================================================

    /// Emit a node by index.
    pub fn emit(&mut self, idx: NodeIndex) {
        if idx.is_none() {
            return;
        }

        let Some(node) = self.arena.get(idx) else {
            return;
        };

        self.emit_node(node, idx);
    }

    /// Emit a node in an expression context.
    /// If the node is an error/unknown node, emits `void 0` for parse error tolerance.
    pub fn emit_expression(&mut self, idx: NodeIndex) {
        if idx.is_none() {
            self.write("void 0");
            return;
        }

        let Some(node) = self.arena.get(idx) else {
            self.write("void 0");
            return;
        };

        // Check if this is an error/unknown node
        use crate::scanner::SyntaxKind;
        if node.kind == SyntaxKind::Unknown as u16 {
            self.write("void 0");
            return;
        }

        // Otherwise, emit normally
        self.emit_node(node, idx);
    }

    /// Emit a node.
    fn emit_node(&mut self, node: &ThinNode, idx: NodeIndex) {
        // Phase 2 Architecture: Check transform directives first
        if self.transforms.has_transform(idx) {
            self.apply_transform(node, idx);
            return;
        }

        // No transform, emit using default logic
        let kind = node.kind;
        self.emit_node_by_kind(node, idx, kind);
    }

    /// Emit a node by kind using default logic (no transforms).
    /// This is the main dispatch method for emission.
    fn emit_node_by_kind(&mut self, node: &ThinNode, idx: NodeIndex, kind: u16) {
        match kind {
            // Identifiers
            k if k == SyntaxKind::Identifier as u16 => {
                self.emit_identifier(node);
            }

            // Literals
            k if k == SyntaxKind::NumericLiteral as u16 => {
                self.emit_numeric_literal(node);
            }
            k if k == SyntaxKind::StringLiteral as u16 => {
                self.emit_string_literal(node);
            }
            k if k == SyntaxKind::TrueKeyword as u16 => {
                self.write("true");
            }
            k if k == SyntaxKind::FalseKeyword as u16 => {
                self.write("false");
            }
            k if k == SyntaxKind::NullKeyword as u16 => {
                self.write("null");
            }

            // Binary expression
            k if k == syntax_kind_ext::BINARY_EXPRESSION => {
                self.emit_binary_expression(node);
            }

            // Unary expressions
            k if k == syntax_kind_ext::PREFIX_UNARY_EXPRESSION => {
                self.emit_prefix_unary(node);
            }
            k if k == syntax_kind_ext::POSTFIX_UNARY_EXPRESSION => {
                self.emit_postfix_unary(node);
            }

            // Call expression
            k if k == syntax_kind_ext::CALL_EXPRESSION => {
                self.emit_call_expression(node);
            }

            // New expression
            k if k == syntax_kind_ext::NEW_EXPRESSION => {
                self.emit_new_expression(node);
            }

            // Property access
            k if k == syntax_kind_ext::PROPERTY_ACCESS_EXPRESSION => {
                self.emit_property_access(node);
            }

            // Element access
            k if k == syntax_kind_ext::ELEMENT_ACCESS_EXPRESSION => {
                self.emit_element_access(node);
            }

            // Parenthesized expression
            k if k == syntax_kind_ext::PARENTHESIZED_EXPRESSION => {
                self.emit_parenthesized(node);
            }

            // Conditional expression
            k if k == syntax_kind_ext::CONDITIONAL_EXPRESSION => {
                self.emit_conditional(node);
            }

            // Array literal
            k if k == syntax_kind_ext::ARRAY_LITERAL_EXPRESSION => {
                self.emit_array_literal(node);
            }

            // Object literal
            k if k == syntax_kind_ext::OBJECT_LITERAL_EXPRESSION => {
                self.emit_object_literal(node);
            }

            // Arrow function
            k if k == syntax_kind_ext::ARROW_FUNCTION => {
                self.emit_arrow_function(node, idx);
            }

            // Function expression
            k if k == syntax_kind_ext::FUNCTION_EXPRESSION => {
                self.emit_function_expression(node, idx);
            }

            // Function declaration
            k if k == syntax_kind_ext::FUNCTION_DECLARATION => {
                self.emit_function_declaration(node, idx);
            }

            // Variable declaration
            k if k == syntax_kind_ext::VARIABLE_DECLARATION => {
                self.emit_variable_declaration(node);
            }

            // Variable declaration list
            k if k == syntax_kind_ext::VARIABLE_DECLARATION_LIST => {
                self.emit_variable_declaration_list(node);
            }

            // Variable statement
            k if k == syntax_kind_ext::VARIABLE_STATEMENT => {
                self.emit_variable_statement(node);
            }

            // Expression statement
            k if k == syntax_kind_ext::EXPRESSION_STATEMENT => {
                self.emit_expression_statement(node);
            }

            // Block
            k if k == syntax_kind_ext::BLOCK => {
                self.emit_block(node);
            }

            // If statement
            k if k == syntax_kind_ext::IF_STATEMENT => {
                self.emit_if_statement(node);
            }

            // While statement
            k if k == syntax_kind_ext::WHILE_STATEMENT => {
                self.emit_while_statement(node);
            }

            // For statement
            k if k == syntax_kind_ext::FOR_STATEMENT => {
                self.emit_for_statement(node);
            }

            // For-in statement
            k if k == syntax_kind_ext::FOR_IN_STATEMENT => {
                self.emit_for_in_statement(node);
            }

            // For-of statement
            k if k == syntax_kind_ext::FOR_OF_STATEMENT => {
                self.emit_for_of_statement(node);
            }

            // Return statement
            k if k == syntax_kind_ext::RETURN_STATEMENT => {
                self.emit_return_statement(node);
            }

            // Class declaration
            k if k == syntax_kind_ext::CLASS_DECLARATION => {
                self.emit_class_declaration(node, idx);
            }

            // Property assignment
            k if k == syntax_kind_ext::PROPERTY_ASSIGNMENT => {
                self.emit_property_assignment(node);
            }

            // Shorthand property assignment
            k if k == syntax_kind_ext::SHORTHAND_PROPERTY_ASSIGNMENT => {
                self.emit_shorthand_property(node);
            }

            // Parameter declaration
            k if k == syntax_kind_ext::PARAMETER => {
                self.emit_parameter(node);
            }

            // Type keywords (for type annotations)
            k if k == SyntaxKind::NumberKeyword as u16 => self.write("number"),
            k if k == SyntaxKind::StringKeyword as u16 => self.write("string"),
            k if k == SyntaxKind::BooleanKeyword as u16 => self.write("boolean"),
            k if k == SyntaxKind::VoidKeyword as u16 => self.write("void"),
            k if k == SyntaxKind::AnyKeyword as u16 => self.write("any"),
            k if k == SyntaxKind::NeverKeyword as u16 => self.write("never"),
            k if k == SyntaxKind::UnknownKeyword as u16 => self.write("unknown"),
            k if k == SyntaxKind::UndefinedKeyword as u16 => self.write("undefined"),
            k if k == SyntaxKind::ObjectKeyword as u16 => self.write("object"),
            k if k == SyntaxKind::SymbolKeyword as u16 => self.write("symbol"),
            k if k == SyntaxKind::BigIntKeyword as u16 => self.write("bigint"),

            // Type reference
            k if k == syntax_kind_ext::TYPE_REFERENCE => {
                self.emit_type_reference(node);
            }

            // Array type
            k if k == syntax_kind_ext::ARRAY_TYPE => {
                self.emit_array_type(node);
            }

            // Union type
            k if k == syntax_kind_ext::UNION_TYPE => {
                self.emit_union_type(node);
            }

            // Intersection type
            k if k == syntax_kind_ext::INTERSECTION_TYPE => {
                self.emit_intersection_type(node);
            }

            // Tuple type
            k if k == syntax_kind_ext::TUPLE_TYPE => {
                self.emit_tuple_type(node);
            }

            // Function type
            k if k == syntax_kind_ext::FUNCTION_TYPE => {
                self.emit_function_type(node);
            }

            // Type literal
            k if k == syntax_kind_ext::TYPE_LITERAL => {
                self.emit_type_literal(node);
            }

            // Parenthesized type
            k if k == syntax_kind_ext::PARENTHESIZED_TYPE => {
                self.emit_parenthesized_type(node);
            }

            // Empty statement
            k if k == syntax_kind_ext::EMPTY_STATEMENT => {
                self.write_semicolon();
            }

            // JSX
            k if k == syntax_kind_ext::JSX_ELEMENT => {
                self.emit_jsx_element(node);
            }
            k if k == syntax_kind_ext::JSX_SELF_CLOSING_ELEMENT => {
                self.emit_jsx_self_closing_element(node);
            }
            k if k == syntax_kind_ext::JSX_OPENING_ELEMENT => {
                self.emit_jsx_opening_element(node);
            }
            k if k == syntax_kind_ext::JSX_CLOSING_ELEMENT => {
                self.emit_jsx_closing_element(node);
            }
            k if k == syntax_kind_ext::JSX_FRAGMENT => {
                self.emit_jsx_fragment(node);
            }
            k if k == syntax_kind_ext::JSX_OPENING_FRAGMENT => {
                self.write("<>");
            }
            k if k == syntax_kind_ext::JSX_CLOSING_FRAGMENT => {
                self.write("</>");
            }
            k if k == syntax_kind_ext::JSX_ATTRIBUTES => {
                self.emit_jsx_attributes(node);
            }
            k if k == syntax_kind_ext::JSX_ATTRIBUTE => {
                self.emit_jsx_attribute(node);
            }
            k if k == syntax_kind_ext::JSX_SPREAD_ATTRIBUTE => {
                self.emit_jsx_spread_attribute(node);
            }
            k if k == syntax_kind_ext::JSX_EXPRESSION => {
                self.emit_jsx_expression(node);
            }
            k if k == SyntaxKind::JsxText as u16 => {
                self.emit_jsx_text(node);
            }
            k if k == syntax_kind_ext::JSX_NAMESPACED_NAME => {
                self.emit_jsx_namespaced_name(node);
            }

            // Imports/Exports
            k if k == syntax_kind_ext::IMPORT_DECLARATION => {
                self.emit_import_declaration(node);
            }
            k if k == syntax_kind_ext::IMPORT_CLAUSE => {
                self.emit_import_clause(node);
            }
            k if k == syntax_kind_ext::NAMED_IMPORTS => {
                self.emit_named_imports(node);
            }
            k if k == syntax_kind_ext::IMPORT_SPECIFIER => {
                self.emit_import_specifier(node);
            }
            k if k == syntax_kind_ext::EXPORT_DECLARATION => {
                self.emit_export_declaration(node);
            }
            k if k == syntax_kind_ext::NAMED_EXPORTS => {
                self.emit_named_exports(node);
            }
            k if k == syntax_kind_ext::EXPORT_SPECIFIER => {
                self.emit_export_specifier(node);
            }
            k if k == syntax_kind_ext::EXPORT_ASSIGNMENT => {
                self.emit_export_assignment(node);
            }

            // Additional statements
            k if k == syntax_kind_ext::THROW_STATEMENT => {
                self.emit_throw_statement(node);
            }
            k if k == syntax_kind_ext::TRY_STATEMENT => {
                self.emit_try_statement(node);
            }
            k if k == syntax_kind_ext::CATCH_CLAUSE => {
                self.emit_catch_clause(node);
            }
            k if k == syntax_kind_ext::SWITCH_STATEMENT => {
                self.emit_switch_statement(node);
            }
            k if k == syntax_kind_ext::CASE_CLAUSE => {
                self.emit_case_clause(node);
            }
            k if k == syntax_kind_ext::DEFAULT_CLAUSE => {
                self.emit_default_clause(node);
            }
            k if k == syntax_kind_ext::CASE_BLOCK => {
                self.emit_case_block(node);
            }
            k if k == syntax_kind_ext::BREAK_STATEMENT => {
                self.emit_break_statement();
            }
            k if k == syntax_kind_ext::CONTINUE_STATEMENT => {
                self.emit_continue_statement();
            }
            k if k == syntax_kind_ext::DO_STATEMENT => {
                self.emit_do_statement(node);
            }
            k if k == syntax_kind_ext::DEBUGGER_STATEMENT => {
                self.emit_debugger_statement();
            }

            // Declarations
            k if k == syntax_kind_ext::ENUM_DECLARATION => {
                self.emit_enum_declaration(node, idx);
            }
            k if k == syntax_kind_ext::ENUM_MEMBER => {
                self.emit_enum_member(node);
            }
            k if k == syntax_kind_ext::INTERFACE_DECLARATION => {
                // Interface declarations are TypeScript-only - skip for JavaScript
                // self.emit_interface_declaration(node);
            }
            k if k == syntax_kind_ext::TYPE_ALIAS_DECLARATION => {
                // Type alias declarations are TypeScript-only - skip for JavaScript
                // self.emit_type_alias_declaration(node);
            }
            k if k == syntax_kind_ext::MODULE_DECLARATION => {
                self.emit_module_declaration(node, idx);
            }

            // Class members
            k if k == syntax_kind_ext::METHOD_DECLARATION => {
                self.emit_method_declaration(node);
            }
            k if k == syntax_kind_ext::PROPERTY_DECLARATION => {
                self.emit_property_declaration(node);
            }
            k if k == syntax_kind_ext::CONSTRUCTOR => {
                self.emit_constructor_declaration(node);
            }
            k if k == syntax_kind_ext::GET_ACCESSOR => {
                self.emit_get_accessor(node);
            }
            k if k == syntax_kind_ext::SET_ACCESSOR => {
                self.emit_set_accessor(node);
            }
            k if k == syntax_kind_ext::DECORATOR => {
                self.emit_decorator(node);
            }

            // Interface/type members (signatures)
            k if k == syntax_kind_ext::PROPERTY_SIGNATURE => {
                self.emit_property_signature(node);
            }
            k if k == syntax_kind_ext::METHOD_SIGNATURE => {
                self.emit_method_signature(node);
            }
            k if k == syntax_kind_ext::CALL_SIGNATURE => {
                self.emit_call_signature(node);
            }
            k if k == syntax_kind_ext::CONSTRUCT_SIGNATURE => {
                self.emit_construct_signature(node);
            }
            k if k == syntax_kind_ext::INDEX_SIGNATURE => {
                self.emit_index_signature(node);
            }

            // Template literals
            k if k == syntax_kind_ext::TEMPLATE_EXPRESSION => {
                self.emit_template_expression(node);
            }
            k if k == SyntaxKind::NoSubstitutionTemplateLiteral as u16 => {
                self.emit_no_substitution_template(node);
            }
            k if k == syntax_kind_ext::TEMPLATE_SPAN => {
                self.emit_template_span(node);
            }
            k if k == SyntaxKind::TemplateHead as u16 => {
                self.emit_template_head(node);
            }
            k if k == SyntaxKind::TemplateMiddle as u16 => {
                self.emit_template_middle(node);
            }
            k if k == SyntaxKind::TemplateTail as u16 => {
                self.emit_template_tail(node);
            }

            // Yield/Await/Spread
            k if k == syntax_kind_ext::YIELD_EXPRESSION => {
                self.emit_yield_expression(node);
            }
            k if k == syntax_kind_ext::AWAIT_EXPRESSION => {
                self.emit_await_expression(node);
            }
            k if k == syntax_kind_ext::SPREAD_ELEMENT => {
                self.emit_spread_element(node);
            }

            // Source file
            k if k == syntax_kind_ext::SOURCE_FILE => {
                self.emit_source_file(node);
            }

            // Other tokens and keywords - emit their text
            k if k == SyntaxKind::ThisKeyword as u16 => {
                // In ES5 mode inside an arrow function body, use _this instead of this
                if self.ctx.target_es5 && self.ctx.arrow_state.this_capture_depth > 0 {
                    self.write("_this")
                } else {
                    self.write("this")
                }
            }
            k if k == SyntaxKind::SuperKeyword as u16 => self.write("super"),

            // Binding patterns (for destructuring)
            k if k == syntax_kind_ext::OBJECT_BINDING_PATTERN => {
                // When emitting as-is (non-ES5 or for parameters), just emit the pattern
                self.emit_object_binding_pattern(node);
            }
            k if k == syntax_kind_ext::ARRAY_BINDING_PATTERN => {
                self.emit_array_binding_pattern(node);
            }
            k if k == syntax_kind_ext::BINDING_ELEMENT => {
                self.emit_binding_element(node);
            }

            // Default: do nothing (or handle other cases as needed)
            _ => {}
        }
    }

    // =========================================================================
    // Literals
    // =========================================================================

    fn emit_identifier(&mut self, node: &ThinNode) {
        if let Some(ident) = self.arena.get_identifier(node) {
            self.write(&ident.escaped_text);
        }
    }

    fn emit_numeric_literal(&mut self, node: &ThinNode) {
        if let Some(lit) = self.arena.get_literal(node) {
            self.write(&lit.text);
        }
    }

    fn emit_string_literal(&mut self, node: &ThinNode) {
        if let Some(lit) = self.arena.get_literal(node) {
            let quote = if self.ctx.options.single_quote { '\'' } else { '"' };
            self.write_char(quote);
            self.emit_escaped_string(&lit.text, quote);
            self.write_char(quote);
        }
    }

    fn emit_escaped_string(&mut self, s: &str, quote_char: char) {
        for ch in s.chars() {
            match ch {
                '\n' => self.write("\\n"),
                '\r' => self.write("\\r"),
                '\t' => self.write("\\t"),
                '\\' => self.write("\\\\"),
                c if c == quote_char => {
                    self.write_char('\\');
                    self.write_char(c);
                }
                c => self.write_char(c),
            }
        }
    }

    // =========================================================================
    // Expressions
    // =========================================================================

    fn emit_binary_expression(&mut self, node: &ThinNode) {
        let Some(binary) = self.arena.get_binary_expr(node) else {
            return;
        };

        self.emit(binary.left);
        self.write_space();
        self.write(get_operator_text(binary.operator_token));
        self.write_space();
        self.emit(binary.right);
    }

    fn emit_prefix_unary(&mut self, node: &ThinNode) {
        let Some(unary) = self.arena.get_unary_expr(node) else {
            return;
        };

        self.write(get_operator_text(unary.operator));
        self.emit(unary.operand);
    }

    fn emit_postfix_unary(&mut self, node: &ThinNode) {
        let Some(unary) = self.arena.get_unary_expr(node) else {
            return;
        };

        self.emit(unary.operand);
        self.write(get_operator_text(unary.operator));
    }

    fn emit_call_expression(&mut self, node: &ThinNode) {
        let Some(call) = self.arena.get_call_expr(node) else {
            return;
        };

        self.emit(call.expression);
        self.write("(");
        if let Some(ref args) = call.arguments {
            self.emit_comma_separated(&args.nodes);
        }
        self.write(")");
    }

    fn emit_new_expression(&mut self, node: &ThinNode) {
        let Some(call) = self.arena.get_call_expr(node) else {
            return;
        };

        self.write("new ");
        self.emit(call.expression);
        self.write("(");
        if let Some(ref args) = call.arguments {
            self.emit_comma_separated(&args.nodes);
        }
        self.write(")");
    }

    fn emit_property_access(&mut self, node: &ThinNode) {
        let Some(access) = self.arena.get_access_expr(node) else {
            return;
        };

        self.emit(access.expression);
        self.write(".");
        self.emit(access.name_or_argument);
    }

    fn emit_element_access(&mut self, node: &ThinNode) {
        let Some(access) = self.arena.get_access_expr(node) else {
            return;
        };

        self.emit(access.expression);
        self.write("[");
        self.emit(access.name_or_argument);
        self.write("]");
    }

    fn emit_parenthesized(&mut self, node: &ThinNode) {
        let Some(paren) = self.arena.get_parenthesized(node) else {
            return;
        };

        self.write("(");
        self.emit(paren.expression);
        self.write(")");
    }

    fn emit_conditional(&mut self, node: &ThinNode) {
        let Some(cond) = self.arena.get_conditional_expr(node) else {
            return;
        };

        self.emit(cond.condition);
        self.write(" ? ");
        self.emit(cond.when_true);
        self.write(" : ");
        self.emit(cond.when_false);
    }

    fn emit_array_literal(&mut self, node: &ThinNode) {
        let Some(array) = self.arena.get_literal_expr(node) else {
            return;
        };

        self.write("[");
        self.emit_comma_separated(&array.elements.nodes);
        self.write("]");
    }

    fn emit_object_literal(&mut self, node: &ThinNode) {
        let Some(obj) = self.arena.get_literal_expr(node) else {
            return;
        };

        if obj.elements.nodes.is_empty() {
            self.write("{}");
            return;
        }

        // Check if we need ES5 computed property transform
        if self.ctx.target_es5 && self.has_computed_property_in_object(&obj.elements.nodes) {
            self.emit_object_literal_es5(&obj.elements.nodes);
            return;
        }

        // Multi-line format for object literals with multiple properties
        if obj.elements.nodes.len() > 1 {
            self.write("{");
            self.write_line();
            self.increase_indent();
            for (i, &prop) in obj.elements.nodes.iter().enumerate() {
                self.emit(prop);
                if i < obj.elements.nodes.len() - 1 {
                    self.write(",");
                }
                self.write_line();
            }
            self.decrease_indent();
            self.write("}");
        } else {
            // Single property: { key: value }
            self.write("{ ");
            self.emit(obj.elements.nodes[0]);
            self.write(" }");
        }
    }

    /// Check if any property in the object literal has a computed property name
    fn has_computed_property_in_object(&self, elements: &[NodeIndex]) -> bool {
        for &idx in elements {
            if self.is_computed_property_member(idx) {
                return true;
            }
            // Also check for spread elements which need ES5 transform
            if let Some(node) = self.arena.get(idx) {
                if node.kind == syntax_kind_ext::SPREAD_ASSIGNMENT
                    || node.kind == syntax_kind_ext::SPREAD_ELEMENT
                {
                    return true;
                }
            }
        }
        false
    }

    /// Check if a property member has a computed property name
    fn is_computed_property_member(&self, idx: NodeIndex) -> bool {
        let Some(node) = self.arena.get(idx) else { return false };

        let name_idx = match node.kind {
            k if k == syntax_kind_ext::PROPERTY_ASSIGNMENT => {
                self.arena.get_property_assignment(node).map(|p| p.name)
            }
            k if k == syntax_kind_ext::METHOD_DECLARATION => {
                self.arena.get_method_decl(node).map(|m| m.name)
            }
            k if k == syntax_kind_ext::GET_ACCESSOR || k == syntax_kind_ext::SET_ACCESSOR => {
                self.arena.get_accessor(node).map(|a| a.name)
            }
            _ => None
        };

        if let Some(name_idx) = name_idx {
            if let Some(name_node) = self.arena.get(name_idx) {
                return name_node.kind == syntax_kind_ext::COMPUTED_PROPERTY_NAME;
            }
        }
        false
    }

    /// Emit ES5-compatible object literal with computed properties
    /// Pattern: { [k]: v } → (_a = {}, _a[k] = v, _a)
    /// Pattern: { a: 1, [k]: v, b: 2 } → (_a = { a: 1 }, _a[k] = v, _a.b = 2, _a)
    fn emit_object_literal_es5(&mut self, elements: &[NodeIndex]) {
        // Get temp variable name
        let temp_var = self.ctx.destructuring_state.next_temp_var();

        // Find the index of the first computed property
        let first_computed_idx = elements.iter()
            .position(|&idx| self.is_computed_property_member(idx) || {
                self.arena.get(idx).map(|n| {
                    n.kind == syntax_kind_ext::SPREAD_ASSIGNMENT
                        || n.kind == syntax_kind_ext::SPREAD_ELEMENT
                }).unwrap_or(false)
            })
            .unwrap_or(elements.len());

        self.write("(");
        self.write(&temp_var);
        self.write(" = ");

        // Emit initial non-computed properties as the object literal
        if first_computed_idx > 0 {
            self.write("{");
            if first_computed_idx > 1 {
                self.write_line();
                self.increase_indent();
            } else {
                self.write(" ");
            }
            for i in 0..first_computed_idx {
                let prop_idx = elements[i];
                self.emit(prop_idx);
                if i < first_computed_idx - 1 {
                    self.write(",");
                    self.write_line();
                }
            }
            if first_computed_idx > 1 {
                self.write_line();
                self.decrease_indent();
            } else {
                self.write(" ");
            }
            self.write("}");
        } else {
            self.write("{}");
        }

        // Emit remaining properties as assignments
        for i in first_computed_idx..elements.len() {
            let prop_idx = elements[i];
            self.write(", ");
            self.emit_property_assignment_es5(prop_idx, &temp_var);
        }

        // Return the temp variable
        self.write(", ");
        self.write(&temp_var);
        self.write(")");
    }

    /// Emit a property assignment in ES5 computed property transform
    fn emit_property_assignment_es5(&mut self, prop_idx: NodeIndex, temp_var: &str) {
        let Some(node) = self.arena.get(prop_idx) else { return };

        match node.kind {
            k if k == syntax_kind_ext::PROPERTY_ASSIGNMENT => {
                if let Some(prop) = self.arena.get_property_assignment(node) {
                    self.emit_assignment_target_es5(prop.name, temp_var);
                    self.write(" = ");
                    self.emit(prop.initializer);
                }
            }
            k if k == syntax_kind_ext::SHORTHAND_PROPERTY_ASSIGNMENT => {
                if let Some(shorthand) = self.arena.get_shorthand_property(node) {
                    let name = self.get_identifier_text(shorthand.name);
                    self.write(temp_var);
                    self.write(".");
                    self.write(&name);
                    self.write(" = ");
                    self.write(&name);
                }
            }
            k if k == syntax_kind_ext::METHOD_DECLARATION => {
                if let Some(method) = self.arena.get_method_decl(node) {
                    self.emit_assignment_target_es5(method.name, temp_var);
                    self.write(" = function");
                    if method.asterisk_token {
                        self.write("*");
                    }
                    self.write(" (");
                    self.emit_function_parameters_js(&method.parameters.nodes);
                    self.write(") ");
                    self.emit(method.body);
                }
            }
            k if k == syntax_kind_ext::GET_ACCESSOR => {
                if let Some(accessor) = self.arena.get_accessor(node) {
                    self.write("Object.defineProperty(");
                    self.write(temp_var);
                    self.write(", ");
                    self.emit_property_key_string(accessor.name);
                    self.write(", { get: function () ");
                    self.emit(accessor.body);
                    self.write(", enumerable: true, configurable: true })");
                }
            }
            k if k == syntax_kind_ext::SET_ACCESSOR => {
                if let Some(accessor) = self.arena.get_accessor(node) {
                    self.write("Object.defineProperty(");
                    self.write(temp_var);
                    self.write(", ");
                    self.emit_property_key_string(accessor.name);
                    self.write(", { set: function (");
                    self.emit_function_parameters_js(&accessor.parameters.nodes);
                    self.write(") ");
                    self.emit(accessor.body);
                    self.write(", enumerable: true, configurable: true })");
                }
            }
            k if k == syntax_kind_ext::SPREAD_ASSIGNMENT => {
                // Spread: { ...x } → Object.assign(_a, x)
                if let Some(spread) = self.arena.get_spread(node) {
                    self.write("Object.assign(");
                    self.write(temp_var);
                    self.write(", ");
                    self.emit(spread.expression);
                    self.write(")");
                }
            }
            k if k == syntax_kind_ext::SPREAD_ELEMENT => {
                // Spread: { ...x } → Object.assign(_a, x)
                if let Some(spread) = self.arena.unary_exprs_ex.get(node.data_index as usize) {
                    self.write("Object.assign(");
                    self.write(temp_var);
                    self.write(", ");
                    self.emit_expression(spread.expression);
                    self.write(")");
                }
            }
            _ => {}
        }
    }

    /// Emit assignment target for ES5 computed property transform
    /// For computed: _a[expr]
    /// For regular: _a.name
    fn emit_assignment_target_es5(&mut self, name_idx: NodeIndex, temp_var: &str) {
        self.write(temp_var);

        let Some(name_node) = self.arena.get(name_idx) else { return };

        if name_node.kind == syntax_kind_ext::COMPUTED_PROPERTY_NAME {
            // Computed property: _a[expr]
            if let Some(computed) = self.arena.get_computed_property(name_node) {
                self.write("[");
                self.emit(computed.expression);
                self.write("]");
            }
        } else if name_node.kind == SyntaxKind::Identifier as u16 {
            // Regular identifier: _a.name
            let name = self.get_identifier_text(name_idx);
            self.write(".");
            self.write(&name);
        } else if name_node.kind == SyntaxKind::StringLiteral as u16 {
            // String literal: _a["name"]
            if let Some(lit) = self.arena.get_literal(name_node) {
                self.write("[\"");
                self.write(&lit.text);
                self.write("\"]");
            }
        } else if name_node.kind == SyntaxKind::NumericLiteral as u16 {
            // Numeric literal: _a[123]
            if let Some(lit) = self.arena.get_literal(name_node) {
                self.write("[");
                self.write(&lit.text);
                self.write("]");
            }
        }
    }

    /// Emit property key as a string for Object.defineProperty
    fn emit_property_key_string(&mut self, name_idx: NodeIndex) {
        let Some(name_node) = self.arena.get(name_idx) else { return };

        if name_node.kind == syntax_kind_ext::COMPUTED_PROPERTY_NAME {
            // Computed property: emit the expression directly
            if let Some(computed) = self.arena.get_computed_property(name_node) {
                self.emit(computed.expression);
            }
        } else if name_node.kind == SyntaxKind::Identifier as u16 {
            let name = self.get_identifier_text(name_idx);
            self.write("\"");
            self.write(&name);
            self.write("\"");
        } else if name_node.kind == SyntaxKind::StringLiteral as u16 {
            if let Some(lit) = self.arena.get_literal(name_node) {
                self.write("\"");
                self.write(&lit.text);
                self.write("\"");
            }
        } else if name_node.kind == SyntaxKind::NumericLiteral as u16 {
            if let Some(lit) = self.arena.get_literal(name_node) {
                self.write(&lit.text);
            }
        }
    }

    fn emit_property_assignment(&mut self, node: &ThinNode) {
        let Some(prop) = self.arena.get_property_assignment(node) else {
            return;
        };

        self.emit(prop.name);
        self.write(": ");
        self.emit_expression(prop.initializer);
    }

    fn emit_shorthand_property(&mut self, node: &ThinNode) {
        let Some(shorthand) = self.arena.get_shorthand_property(node) else {
            // Fallback: try to get identifier data directly
            if let Some(ident) = self.arena.get_identifier(node) {
                self.write(&ident.escaped_text);
            }
            return;
        };

        self.emit(shorthand.name);
        if shorthand.equals_token {
            self.write(" = ");
            // Object assignment pattern default value would go here
        }
    }

    // =========================================================================
    // Functions
    // =========================================================================

    fn emit_arrow_function(&mut self, node: &ThinNode, _idx: NodeIndex) {
        let Some(func) = self.arena.get_function(node) else {
            return;
        };

        // Transform arrow function to regular function for ES5
        if self.ctx.target_es5 {
            self.emit_arrow_function_es5(node, func);
        } else {
            self.emit_arrow_function_native(func);
        }
    }

    /// Emit ES5-compatible function expression for arrow function
    /// Arrow: (x) => x + 1  →  function (x) { return x + 1; }
    fn emit_arrow_function_es5(&mut self, _node: &ThinNode, func: &crate::parser::thin_node::FunctionData) {
        // Check if arrow body uses `this` - if so, we need _this capture
        let body_uses_this = !func.body.is_none()
            && contains_this_reference(self.arena, func.body);

        // Track that we're inside an arrow function body with `this`
        if body_uses_this {
            self.ctx.arrow_state.this_capture_depth += 1;
        }

        if func.is_async {
            self.write("async ");
        }

        self.write("function (");
        let param_transforms = self.emit_function_parameters_es5(&func.parameters.nodes);
        self.write(") ");

        // If body is not a block (concise arrow), wrap with return
        let body_node = self.arena.get(func.body);
        let is_block = body_node.map(|n| n.kind == syntax_kind_ext::BLOCK).unwrap_or(false);
        let needs_param_prologue = param_transforms.has_transforms();

        if is_block {
            // Check if it's a simple single-return block
            if let Some(block_node) = self.arena.get(func.body) {
                if let Some(block) = self.arena.get_block(block_node) {
                    if !needs_param_prologue
                        && block.statements.nodes.len() == 1
                        && self.is_simple_return_statement(block.statements.nodes[0]) {
                        self.emit_single_line_block(func.body);
                    } else if needs_param_prologue {
                        self.emit_block_with_param_prologue(func.body, &param_transforms);
                    } else {
                        self.emit(func.body);
                    }
                } else {
                    if needs_param_prologue {
                        self.emit_block_with_param_prologue(func.body, &param_transforms);
                    } else {
                        self.emit(func.body);
                    }
                }
            } else {
                if needs_param_prologue {
                    self.emit_block_with_param_prologue(func.body, &param_transforms);
                } else {
                    self.emit(func.body);
                }
            }
        } else {
            if needs_param_prologue {
                self.write("{");
                self.write_line();
                self.increase_indent();
                self.emit_param_prologue(&param_transforms);
                self.write("return ");
                self.emit(func.body);
                self.write(";");
                self.write_line();
                self.decrease_indent();
                self.write("}");
            } else {
                // Concise body: (x) => x + 1  →  function (x) { return x + 1; }
                self.write("{ return ");
                self.emit(func.body);
                self.write("; }");
            }
        }

        // Restore this capture depth
        if body_uses_this {
            self.ctx.arrow_state.this_capture_depth -= 1;
        }
    }

    /// Emit native ES6+ arrow function syntax
    fn emit_arrow_function_native(&mut self, func: &crate::parser::thin_node::FunctionData) {
        if func.is_async {
            self.write("async ");
        }

        // Parameters (without types for JavaScript)
        self.write("(");
        self.emit_function_parameters_js(&func.parameters.nodes);
        self.write(")");

        // Skip return type for JavaScript

        self.write(" => ");

        // Body
        self.emit(func.body);
    }

    fn emit_function_expression(&mut self, node: &ThinNode, _idx: NodeIndex) {
        let Some(func) = self.arena.get_function(node) else {
            return;
        };

        if func.is_async {
            self.write("async ");
        }

        self.write("function");

        if func.asterisk_token {
            self.write("*");
        }

        // Name (if any) - add space before open paren whether or not there's a name
        if !func.name.is_none() {
            self.write_space();
            self.emit(func.name);
        }

        // Space before ( for TypeScript compatibility: function (x) vs function(x)
        self.write(" ");

        // Parameters (without types for JavaScript)
        self.write("(");
        let param_transforms = if self.ctx.target_es5 {
            self.emit_function_parameters_es5(&func.parameters.nodes)
        } else {
            self.emit_function_parameters_js(&func.parameters.nodes);
            ParamTransformPlan::default()
        };
        self.write(") ");

        // Emit body - check if it's a simple single-statement body
        let body_node = self.arena.get(func.body);
        let is_simple_body = if let Some(body) = body_node {
            if let Some(block) = self.arena.get_block(body) {
                // Single return statement = simple body
                block.statements.nodes.len() == 1
                    && self.is_simple_return_statement(block.statements.nodes[0])
            } else {
                false
            }
        } else {
            false
        };
        
        if self.ctx.target_es5 && param_transforms.has_transforms() {
            self.emit_block_with_param_prologue(func.body, &param_transforms);
        } else if is_simple_body {
            self.emit_single_line_block(func.body);
        } else {
            self.emit(func.body);
        }
    }
    
    /// Check if a statement is a simple return statement (for single-line emission)
    fn is_simple_return_statement(&self, stmt_idx: NodeIndex) -> bool {
        let Some(node) = self.arena.get(stmt_idx) else { return false };
        if node.kind != syntax_kind_ext::RETURN_STATEMENT {
            return false;
        }
        // Consider it simple if it has an expression (not just "return;")
        if let Some(ret) = self.arena.get_return_statement(node) {
            return !ret.expression.is_none();
        }
        false
    }
    
    /// Emit a block on a single line: { return expr; }
    fn emit_single_line_block(&mut self, block_idx: NodeIndex) {
        let Some(block_node) = self.arena.get(block_idx) else { return };
        let Some(block) = self.arena.get_block(block_node) else { return };
        
        self.write("{ ");
        for (i, &stmt_idx) in block.statements.nodes.iter().enumerate() {
            if i > 0 {
                self.write(" ");
            }
            self.emit(stmt_idx);
        }
        self.write(" }");
    }

    fn emit_block_with_param_prologue(&mut self, block_idx: NodeIndex, transforms: &ParamTransformPlan) {
        let Some(block_node) = self.arena.get(block_idx) else { return };
        let Some(block) = self.arena.get_block(block_node) else { return };

        self.write("{");
        self.write_line();
        self.increase_indent();
        self.emit_param_prologue(transforms);

        for &stmt_idx in &block.statements.nodes {
            let before_len = self.writer.len();
            self.emit(stmt_idx);
            if self.writer.len() > before_len {
                self.write_line();
            }
        }

        self.decrease_indent();
        self.write("}");
        self.emit_trailing_comments(block_node.end);
    }

    fn emit_function_declaration(&mut self, node: &ThinNode, _idx: NodeIndex) {
        let Some(func) = self.arena.get_function(node) else {
            return;
        };

        // Skip ambient declarations (declare function)
        if self.has_declare_modifier(&func.modifiers) {
            return;
        }

        // For JavaScript emit: skip declaration-only functions (no body)
        // These are just type information in TypeScript
        if func.body.is_none() {
            return;
        }

        let is_exported = self.ctx.is_commonjs()
            && self.has_export_modifier(&func.modifiers)
            && !self.ctx.module_state.has_export_assignment;
        let is_default = self.has_default_modifier(&func.modifiers);

        // Get function name for export
        let func_name = if !func.name.is_none() {
            self.get_identifier_text_idx(func.name)
        } else {
            String::new()
        };

        // ES5 async transform: wrap in __awaiter/__generator
        if self.ctx.target_es5 && func.is_async {
            self.emit_async_function_es5(func, &func_name, is_exported, is_default);
            return;
        }

        if func.is_async {
            self.write("async ");
        }

        self.write("function");

        if func.asterisk_token {
            self.write("*");
        }

        // Name
        if !func.name.is_none() {
            self.write_space();
            self.emit(func.name);
        }

        // Parameters - only emit names, not types for JavaScript
        self.write("(");
        let param_transforms = if self.ctx.target_es5 {
            self.emit_function_parameters_es5(&func.parameters.nodes)
        } else {
            self.emit_function_parameters_js(&func.parameters.nodes);
            ParamTransformPlan::default()
        };
        self.write(")");

        // No return type for JavaScript

        self.write_space();
        if self.ctx.target_es5 && param_transforms.has_transforms() {
            self.emit_block_with_param_prologue(func.body, &param_transforms);
        } else {
            self.emit(func.body);
        }

        // CommonJS: emit exports.funcName = funcName; after the function
        if is_exported && !func_name.is_empty() {
            self.write_line();
            if is_default {
                self.write("exports.default = ");
            } else {
                self.write("exports.");
                self.write(&func_name);
                self.write(" = ");
            }
            self.write(&func_name);
            self.write(";");
        }
    }

    /// Emit an async function transformed to ES5 __awaiter/__generator pattern
    fn emit_async_function_es5(
        &mut self,
        func: &crate::parser::thin_node::FunctionData,
        func_name: &str,
        is_exported: bool,
        is_default: bool,
    ) {
        // function name(params) {
        self.write("function");
        if !func_name.is_empty() {
            self.write_space();
            self.write(func_name);
        }
        self.write("(");
        let param_transforms = self.emit_function_parameters_es5(&func.parameters.nodes);
        self.write(") {");
        self.write_line();
        self.increase_indent();

        self.emit_param_prologue(&param_transforms);

        // Emit indented __awaiter body
        //     return __awaiter(this, void 0, void 0, function () {
        //         return __generator(this, function (_a) { ... });
        //     });
        let mut async_emitter = crate::transforms::async_es5::AsyncES5Emitter::new(self.arena);
        // Transform emitter handles its own indentation inside __awaiter
        async_emitter.set_indent_level(self.writer.indent_level() + 1);

        let generator_body = if async_emitter.body_contains_await(func.body) {
            async_emitter.emit_generator_body_with_await(func.body)
        } else {
            async_emitter.emit_simple_generator_body(func.body)
        };

        // Write with surrounding __awaiter wrapper
        self.write("return __awaiter(this, void 0, void 0, function () {");
        self.write_line();
        self.increase_indent();
        self.write(&generator_body);
        self.decrease_indent();
        self.write_line();
        self.write("});");
        self.write_line();
        self.decrease_indent();
        self.write("}");

        // CommonJS: emit exports.funcName = funcName; after the function
        if is_exported && !func_name.is_empty() {
            self.write_line();
            if is_default {
                self.write("exports.default = ");
            } else {
                self.write("exports.");
                self.write(func_name);
                self.write(" = ");
            }
            self.write(func_name);
            self.write(";");
        }
    }

    fn emit_function_parameters_es5(&mut self, params: &[NodeIndex]) -> ParamTransformPlan {
        let mut plan = ParamTransformPlan::default();
        let mut first = true;

        for (index, &param_idx) in params.iter().enumerate() {
            let Some(param_node) = self.arena.get(param_idx) else { continue };
            let Some(param) = self.arena.get_parameter(param_node) else { continue };

            if param.dot_dot_dot_token {
                let rest_target = param.name;
                let rest_is_pattern = self.is_binding_pattern(rest_target);
                let rest_name = if rest_is_pattern {
                    self.get_temp_var_name()
                } else {
                    self.get_identifier_text(rest_target)
                };

                if !rest_name.is_empty() {
                    plan.rest = Some(RestParamTransform {
                        name: rest_name,
                        pattern: if rest_is_pattern { Some(rest_target) } else { None },
                        index,
                    });
                }
                break;
            }

            if !first {
                self.write(", ");
            }
            first = false;

            if self.is_binding_pattern(param.name) {
                let temp_name = self.get_temp_var_name();
                self.write(&temp_name);
                plan.params.push(ParamTransform {
                    name: temp_name,
                    pattern: Some(param.name),
                    initializer: if param.initializer.is_none() {
                        None
                    } else {
                        Some(param.initializer)
                    },
                });
            } else {
                self.emit(param.name);
                if !param.initializer.is_none() {
                    let name = self.get_identifier_text(param.name);
                    if !name.is_empty() {
                        plan.params.push(ParamTransform {
                            name,
                            pattern: None,
                            initializer: Some(param.initializer),
                        });
                    }
                }
            }
        }

        plan
    }

    /// Emit function parameters for JavaScript (no types)
    fn emit_function_parameters_js(&mut self, params: &[NodeIndex]) {
        let mut first = true;
        for &param_idx in params {
            if !first {
                self.write(", ");
            }
            first = false;

            if let Some(param_node) = self.arena.get(param_idx) {
                if let Some(param) = self.arena.get_parameter(param_node) {
                    if param.dot_dot_dot_token {
                        self.write("...");
                    }
                    self.emit(param.name);
                    // Skip type annotations and defaults for JS emit
                    if !param.initializer.is_none() {
                        self.write(" = ");
                        self.emit(param.initializer);
                    }
                }
            }
        }
    }

    fn emit_parameter(&mut self, node: &ThinNode) {
        let Some(param) = self.arena.get_parameter(node) else {
            return;
        };

        if param.dot_dot_dot_token {
            self.write("...");
        }

        self.emit(param.name);

        if param.question_token {
            self.write("?");
        }

        if !param.type_annotation.is_none() {
            self.write(": ");
            self.emit(param.type_annotation);
        }

        if !param.initializer.is_none() {
            self.write(" = ");
            self.emit_expression(param.initializer);
        }
    }

    // =========================================================================
    // Statements
    // =========================================================================

    fn emit_block(&mut self, node: &ThinNode) {
        let Some(block) = self.arena.get_block(node) else {
            return;
        };

        // Empty blocks: preserve original format (single-line vs multi-line)
        if block.statements.nodes.is_empty() {
            if self.is_single_line(node) {
                // Single-line empty block: { }
                self.write("{ }");
            } else {
                // Multi-line empty block: {\n}
                self.write("{");
                self.write_line();
                self.write("}");
            }
            // Emit trailing comments after the block's closing brace
            self.emit_trailing_comments(node.end);
            return;
        }

        self.write("{");
        self.write_line();
        self.increase_indent();

        for &stmt_idx in &block.statements.nodes {
            let before_len = self.writer.len();
            self.emit(stmt_idx);
            // Only add newline if something was actually emitted
            if self.writer.len() > before_len {
                self.write_line();
            }
        }

        self.decrease_indent();
        self.write("}");
        // Emit trailing comments after the block's closing brace
        self.emit_trailing_comments(node.end);
    }

    fn emit_variable_statement(&mut self, node: &ThinNode) {
        let Some(var_stmt) = self.arena.get_variable(node) else {
            return;
        };

        // Skip ambient declarations (declare var/let/const)
        if self.has_declare_modifier(&var_stmt.modifiers) {
            return;
        }

        let is_exported = self.ctx.is_commonjs()
            && self.has_export_modifier(&var_stmt.modifiers)
            && !self.ctx.module_state.has_export_assignment;
        let is_default = self.has_default_modifier(&var_stmt.modifiers);

        // Collect declaration names for export assignment
        let export_names: Vec<String> = if is_exported {
            self.collect_variable_names(&var_stmt.declarations)
        } else {
            Vec::new()
        };

        // VariableStatement.declarations contains a VARIABLE_DECLARATION_LIST
        // Emit the declaration list (which handles the let/const/var keyword)
        for &decl_list_idx in &var_stmt.declarations.nodes {
            self.emit(decl_list_idx);
        }
        self.write_semicolon();

        // CommonJS: emit exports.X = X; after the declaration
        if is_exported && !export_names.is_empty() {
            self.write_line();
            if is_default && export_names.len() == 1 {
                // export default const x = ... -> exports.default = x;
                self.write("exports.default = ");
                self.write(&export_names[0]);
                self.write(";");
            } else {
                // export const x = ..., y = ...; -> exports.x = x; exports.y = y;
                for name in &export_names {
                    self.write("exports.");
                    self.write(name);
                    self.write(" = ");
                    self.write(name);
                    self.write(";");
                    self.write_line();
                }
            }
        }
    }

    /// Collect variable names from a declaration list for CommonJS export
    fn collect_variable_names(&self, declarations: &NodeList) -> Vec<String> {
        let mut names = Vec::new();
        for &decl_list_idx in &declarations.nodes {
            let Some(decl_list_node) = self.arena.get(decl_list_idx) else { continue };
            let Some(decl_list) = self.arena.get_variable(decl_list_node) else { continue };

            for &decl_idx in &decl_list.declarations.nodes {
                let Some(decl_node) = self.arena.get(decl_idx) else { continue };
                let Some(decl) = self.arena.get_variable_declaration(decl_node) else { continue };
                self.collect_binding_names(decl.name, &mut names);
            }
        }
        names
    }

    fn collect_binding_names(&self, name_idx: NodeIndex, names: &mut Vec<String>) {
        if name_idx.is_none() {
            return;
        }

        let Some(node) = self.arena.get(name_idx) else {
            return;
        };

        if node.kind == SyntaxKind::Identifier as u16 {
            if let Some(id) = self.arena.get_identifier(node) {
                names.push(id.escaped_text.clone());
            }
            return;
        }

        match node.kind {
            k if k == syntax_kind_ext::OBJECT_BINDING_PATTERN
                || k == syntax_kind_ext::ARRAY_BINDING_PATTERN =>
            {
                if let Some(pattern) = self.arena.get_binding_pattern(node) {
                    for &elem_idx in &pattern.elements.nodes {
                        self.collect_binding_names_from_element(elem_idx, names);
                    }
                }
            }
            k if k == syntax_kind_ext::BINDING_ELEMENT => {
                if let Some(elem) = self.arena.get_binding_element(node) {
                    self.collect_binding_names(elem.name, names);
                }
            }
            _ => {}
        }
    }

    fn collect_binding_names_from_element(&self, elem_idx: NodeIndex, names: &mut Vec<String>) {
        if elem_idx.is_none() {
            return;
        }

        let Some(elem_node) = self.arena.get(elem_idx) else {
            return;
        };

        if let Some(elem) = self.arena.get_binding_element(elem_node) {
            self.collect_binding_names(elem.name, names);
        }
    }

    fn emit_variable_declaration_list(&mut self, node: &ThinNode) {
        // Variable declaration list is stored as VariableData
        let Some(decl_list) = self.arena.get_variable(node) else {
            return;
        };

        // Emit keyword based on node flags - for ES5, always use "var"
        let flags = node.flags as u32;
        let keyword = if self.ctx.target_es5 {
            "var"
        } else if flags & crate::parser::node_flags::CONST != 0 {
            "const"
        } else if flags & crate::parser::node_flags::LET != 0 {
            "let"
        } else {
            "var"
        };
        self.write(keyword);
        self.write(" ");

        // For ES5, check if any declaration uses destructuring
        if self.ctx.target_es5 {
            let mut first = true;
            for &decl_idx in &decl_list.declarations.nodes {
                let Some(decl_node) = self.arena.get(decl_idx) else { continue };
                let Some(decl) = self.arena.get_variable_declaration(decl_node) else { continue };

                if self.is_binding_pattern(decl.name) && !decl.initializer.is_none() {
                    // ES5 destructuring transform
                    self.emit_es5_destructuring(decl_idx, &mut first);
                } else {
                    // Normal variable declaration
                    if !first {
                        self.write(", ");
                    }
                    first = false;
                    self.emit(decl_idx);
                }
            }
        } else {
            self.emit_comma_separated(&decl_list.declarations.nodes);
        }
    }

    fn emit_variable_declaration(&mut self, node: &ThinNode) {
        let Some(decl) = self.arena.get_variable_declaration(node) else {
            return;
        };

        self.emit(decl.name);

        // Skip type annotation for JavaScript emit

        if !decl.initializer.is_none() {
            self.write(" = ");
            self.emit_expression(decl.initializer);
        }
    }

    /// Emit ES5 destructuring: { x, y } = obj → _a = obj, x = _a.x, y = _a.y
    fn emit_es5_destructuring(&mut self, decl_idx: NodeIndex, first: &mut bool) {
        let Some(decl_node) = self.arena.get(decl_idx) else { return };
        let Some(decl) = self.arena.get_variable_declaration(decl_node) else { return };
        let Some(pattern_node) = self.arena.get(decl.name) else { return };

        // Get temp variable name
        let temp_name = self.get_temp_var_name();

        // Emit temp variable assignment: _a = initializer
        if !*first {
            self.write(", ");
        }
        *first = false;
        self.write(&temp_name);
        self.write(" = ");
        self.emit(decl.initializer);

        self.emit_es5_destructuring_pattern(pattern_node, &temp_name);
    }

    fn emit_es5_destructuring_from_value(&mut self, pattern_idx: NodeIndex, result_name: &str, first: &mut bool) {
        let Some(pattern_node) = self.arena.get(pattern_idx) else { return };

        let temp_name = self.get_temp_var_name();

        if !*first {
            self.write(", ");
        }
        *first = false;
        self.write(&temp_name);
        self.write(" = ");
        self.write(result_name);
        self.write(".value");

        self.emit_es5_destructuring_pattern(pattern_node, &temp_name);
    }

    fn get_binding_element_property_key(
        &self,
        elem: &crate::parser::thin_node::BindingElementData,
    ) -> Option<NodeIndex> {
        let key_idx = if !elem.property_name.is_none() {
            elem.property_name
        } else {
            elem.name
        };
        let Some(key_node) = self.arena.get(key_idx) else { return None };
        match key_node.kind {
            k if k == syntax_kind_ext::COMPUTED_PROPERTY_NAME
                || k == SyntaxKind::Identifier as u16
                || k == SyntaxKind::StringLiteral as u16
                || k == SyntaxKind::NumericLiteral as u16 =>
            {
                Some(key_idx)
            }
            _ => None,
        }
    }

    /// Emit a single binding element for ES5 object destructuring
    fn emit_es5_binding_element(&mut self, elem_idx: NodeIndex, temp_name: &str) {
        let Some(elem_node) = self.arena.get(elem_idx) else { return };
        let Some(elem) = self.arena.get_binding_element(elem_node) else { return };
        if elem.dot_dot_dot_token {
            return;
        }

        let Some(key_idx) = self.get_binding_element_property_key(elem) else {
            return;
        };

        if self.is_binding_pattern(elem.name) {
            let value_name = self.get_temp_var_name();
            self.write(", ");
            self.write(&value_name);
            self.write(" = ");
            self.emit_assignment_target_es5(key_idx, temp_name);

            if !elem.initializer.is_none() {
                self.write(", ");
                self.write(&value_name);
                self.write(" = ");
                self.write(&value_name);
                self.write(" === void 0 ? ");
                self.emit_expression(elem.initializer);
                self.write(" : ");
                self.write(&value_name);
            }

            self.emit_es5_destructuring_pattern_idx(elem.name, &value_name);
            return;
        }

        let binding_name = self.get_identifier_text(elem.name);
        if binding_name.is_empty() {
            return;
        }

        if elem.initializer.is_none() {
            // Emit: , bindingName = temp.propName
            self.write(", ");
            self.write(&binding_name);
            self.write(" = ");
            self.emit_assignment_target_es5(key_idx, temp_name);
        } else {
            let value_name = self.get_temp_var_name();
            self.write(", ");
            self.write(&value_name);
            self.write(" = ");
            self.emit_assignment_target_es5(key_idx, temp_name);
            self.write(", ");
            self.write(&binding_name);
            self.write(" = ");
            self.write(&value_name);
            self.write(" === void 0 ? ");
            self.emit_expression(elem.initializer);
            self.write(" : ");
            self.write(&value_name);
        }
    }

    /// Emit a single binding element for ES5 array destructuring
    fn emit_es5_array_binding_element(&mut self, elem_idx: NodeIndex, temp_name: &str, index: usize) {
        let Some(elem_node) = self.arena.get(elem_idx) else { return };
        let Some(elem) = self.arena.get_binding_element(elem_node) else { return };

        if elem.dot_dot_dot_token {
            self.emit_es5_array_rest_element(elem.name, temp_name, index);
            return;
        }

        if self.is_binding_pattern(elem.name) {
            let value_name = self.get_temp_var_name();
            self.write(", ");
            self.write(&value_name);
            self.write(" = ");
            self.write(temp_name);
            self.write("[");
            self.write(&index.to_string());
            self.write("]");

            if !elem.initializer.is_none() {
                self.write(", ");
                self.write(&value_name);
                self.write(" = ");
                self.write(&value_name);
                self.write(" === void 0 ? ");
                self.emit_expression(elem.initializer);
                self.write(" : ");
                self.write(&value_name);
            }

            self.emit_es5_destructuring_pattern_idx(elem.name, &value_name);
            return;
        }

        let binding_name = self.get_identifier_text(elem.name);
        if binding_name.is_empty() {
            return;
        }

        if elem.initializer.is_none() {
            // Emit: , bindingName = temp[index]
            self.write(", ");
            self.write(&binding_name);
            self.write(" = ");
            self.write(temp_name);
            self.write("[");
            self.write(&index.to_string());
            self.write("]");
        } else {
            let value_name = self.get_temp_var_name();
            self.write(", ");
            self.write(&value_name);
            self.write(" = ");
            self.write(temp_name);
            self.write("[");
            self.write(&index.to_string());
            self.write("]");
            self.write(", ");
            self.write(&binding_name);
            self.write(" = ");
            self.write(&value_name);
            self.write(" === void 0 ? ");
            self.emit_expression(elem.initializer);
            self.write(" : ");
            self.write(&value_name);
        }
    }

    fn emit_es5_destructuring_pattern(&mut self, pattern_node: &ThinNode, temp_name: &str) {
        if pattern_node.kind == syntax_kind_ext::OBJECT_BINDING_PATTERN {
            let Some(pattern) = self.arena.get_binding_pattern(pattern_node) else { return };
            let rest_props = self.collect_object_rest_props(pattern);
            for &elem_idx in &pattern.elements.nodes {
                if elem_idx.is_none() {
                    continue;
                }
                let Some(elem_node) = self.arena.get(elem_idx) else { continue };
                let Some(elem) = self.arena.get_binding_element(elem_node) else { continue };
                if elem.dot_dot_dot_token {
                    self.emit_es5_object_rest_element(elem, &rest_props, temp_name);
                } else {
                    self.emit_es5_binding_element(elem_idx, temp_name);
                }
            }
        } else if pattern_node.kind == syntax_kind_ext::ARRAY_BINDING_PATTERN {
            if let Some(pattern) = self.arena.get_binding_pattern(pattern_node) {
                for (i, &elem_idx) in pattern.elements.nodes.iter().enumerate() {
                    self.emit_es5_array_binding_element(elem_idx, temp_name, i);
                }
            }
        }
    }

    fn emit_param_prologue(&mut self, transforms: &ParamTransformPlan) {
        for param in &transforms.params {
            if let Some(initializer) = param.initializer {
                self.emit_param_default_assignment(&param.name, initializer);
            }
            if let Some(pattern) = param.pattern {
                let mut started = false;
                self.emit_param_binding_assignments(pattern, &param.name, &mut started);
                if started {
                    self.write(";");
                    self.write_line();
                }
            }
        }

        if let Some(rest) = &transforms.rest {
            if !rest.name.is_empty() {
                self.write("var ");
                self.write(&rest.name);
                self.write(" = [];");
                self.write_line();

                let iter_name = self.get_temp_var_name();
                self.write("for (var ");
                self.write(&iter_name);
                self.write(" = ");
                self.write(&rest.index.to_string());
                self.write("; ");
                self.write(&iter_name);
                self.write(" < arguments.length; ");
                self.write(&iter_name);
                self.write("++) ");
                self.write(&rest.name);
                self.write("[");
                self.write(&iter_name);
                self.write(" - ");
                self.write(&rest.index.to_string());
                self.write("] = arguments[");
                self.write(&iter_name);
                self.write("];");
                self.write_line();
            }

            if let Some(pattern) = rest.pattern {
                let mut started = false;
                self.emit_param_binding_assignments(pattern, &rest.name, &mut started);
                if started {
                    self.write(";");
                    self.write_line();
                }
            }
        }
    }

    fn emit_param_default_assignment(&mut self, name: &str, initializer: NodeIndex) {
        if name.is_empty() {
            return;
        }
        self.write("if (");
        self.write(name);
        self.write(" === void 0) { ");
        self.write(name);
        self.write(" = ");
        self.emit_expression(initializer);
        self.write("; }");
        self.write_line();
    }

    fn emit_param_binding_assignments(
        &mut self,
        pattern_idx: NodeIndex,
        temp_name: &str,
        started: &mut bool,
    ) {
        let Some(pattern_node) = self.arena.get(pattern_idx) else { return };

        match pattern_node.kind {
            k if k == syntax_kind_ext::OBJECT_BINDING_PATTERN => {
                if let Some(pattern) = self.arena.get_binding_pattern(pattern_node) {
                    let rest_props = self.collect_object_rest_props(pattern);
                    for &elem_idx in &pattern.elements.nodes {
                        if elem_idx.is_none() {
                            continue;
                        }
                        let Some(elem_node) = self.arena.get(elem_idx) else { continue };
                        let Some(elem) = self.arena.get_binding_element(elem_node) else { continue };
                        if elem.dot_dot_dot_token {
                            self.emit_param_object_rest_element(elem, &rest_props, temp_name, started);
                        } else {
                            self.emit_param_object_binding_element(elem_idx, temp_name, started);
                        }
                    }
                }
            }
            k if k == syntax_kind_ext::ARRAY_BINDING_PATTERN => {
                if let Some(pattern) = self.arena.get_binding_pattern(pattern_node) {
                    for (i, &elem_idx) in pattern.elements.nodes.iter().enumerate() {
                        self.emit_param_array_binding_element(elem_idx, temp_name, i, started);
                    }
                }
            }
            _ => {}
        }
    }

    fn emit_param_object_binding_element(
        &mut self,
        elem_idx: NodeIndex,
        temp_name: &str,
        started: &mut bool,
    ) {
        let Some(elem_node) = self.arena.get(elem_idx) else { return };
        let Some(elem) = self.arena.get_binding_element(elem_node) else { return };

        if elem.dot_dot_dot_token {
            return;
        }

        let Some(key_idx) = self.get_binding_element_property_key(elem) else {
            return;
        };

        if self.is_binding_pattern(elem.name) {
            let value_name = self.get_temp_var_name();
            self.emit_param_assignment_prefix(started);
            self.write(&value_name);
            self.write(" = ");
            self.emit_assignment_target_es5(key_idx, temp_name);

            if !elem.initializer.is_none() {
                self.write(", ");
                self.write(&value_name);
                self.write(" = ");
                self.write(&value_name);
                self.write(" === void 0 ? ");
                self.emit_expression(elem.initializer);
                self.write(" : ");
                self.write(&value_name);
            }

            self.emit_param_binding_assignments(elem.name, &value_name, started);
            return;
        }

        let binding_name = self.get_identifier_text(elem.name);
        if binding_name.is_empty() {
            return;
        }

        self.emit_param_assignment_prefix(started);
        if !elem.initializer.is_none() {
            let value_name = self.get_temp_var_name();
            self.write(&value_name);
            self.write(" = ");
            self.emit_assignment_target_es5(key_idx, temp_name);
            self.write(", ");
            self.write(&binding_name);
            self.write(" = ");
            self.write(&value_name);
            self.write(" === void 0 ? ");
            self.emit_expression(elem.initializer);
            self.write(" : ");
            self.write(&value_name);
        } else {
            self.write(&binding_name);
            self.write(" = ");
            self.emit_assignment_target_es5(key_idx, temp_name);
        }
    }

    fn emit_param_array_binding_element(
        &mut self,
        elem_idx: NodeIndex,
        temp_name: &str,
        index: usize,
        started: &mut bool,
    ) {
        if elem_idx.is_none() {
            return;
        }
        let Some(elem_node) = self.arena.get(elem_idx) else { return };
        let Some(elem) = self.arena.get_binding_element(elem_node) else { return };

        if elem.dot_dot_dot_token {
            self.emit_param_array_rest_element(elem.name, temp_name, index, started);
            return;
        }

        if self.is_binding_pattern(elem.name) {
            let value_name = self.get_temp_var_name();
            self.emit_param_assignment_prefix(started);
            self.write(&value_name);
            self.write(" = ");
            self.write(temp_name);
            self.write("[");
            self.write(&index.to_string());
            self.write("]");

            if !elem.initializer.is_none() {
                self.write(", ");
                self.write(&value_name);
                self.write(" = ");
                self.write(&value_name);
                self.write(" === void 0 ? ");
                self.emit_expression(elem.initializer);
                self.write(" : ");
                self.write(&value_name);
            }

            self.emit_param_binding_assignments(elem.name, &value_name, started);
            return;
        }

        let binding_name = self.get_identifier_text(elem.name);
        if binding_name.is_empty() {
            return;
        }

        self.emit_param_assignment_prefix(started);
        if !elem.initializer.is_none() {
            let value_name = self.get_temp_var_name();
            self.write(&value_name);
            self.write(" = ");
            self.write(temp_name);
            self.write("[");
            self.write(&index.to_string());
            self.write("]");
            self.write(", ");
            self.write(&binding_name);
            self.write(" = ");
            self.write(&value_name);
            self.write(" === void 0 ? ");
            self.emit_expression(elem.initializer);
            self.write(" : ");
            self.write(&value_name);
        } else {
            self.write(&binding_name);
            self.write(" = ");
            self.write(temp_name);
            self.write("[");
            self.write(&index.to_string());
            self.write("]");
        }
    }

    fn emit_param_object_rest_element(
        &mut self,
        elem: &crate::parser::thin_node::BindingElementData,
        rest_props: &[NodeIndex],
        temp_name: &str,
        started: &mut bool,
    ) {
        let rest_target = elem.name;
        let is_pattern = self.is_binding_pattern(rest_target);
        let rest_temp = if is_pattern {
            Some(self.get_temp_var_name())
        } else {
            None
        };

        self.emit_param_assignment_prefix(started);
        if let Some(ref name) = rest_temp {
            self.write(name);
        } else {
            self.emit(rest_target);
        }
        self.write(" = __rest(");
        self.write(temp_name);
        self.write(", ");
        self.emit_rest_exclude_list(rest_props);
        self.write(")");

        if let Some(ref name) = rest_temp {
            self.emit_param_binding_assignments(rest_target, name, started);
        }
    }

    fn emit_param_array_rest_element(
        &mut self,
        rest_target: NodeIndex,
        temp_name: &str,
        index: usize,
        started: &mut bool,
    ) {
        let is_pattern = self.is_binding_pattern(rest_target);
        let rest_temp = if is_pattern {
            Some(self.get_temp_var_name())
        } else {
            None
        };

        self.emit_param_assignment_prefix(started);
        if let Some(ref name) = rest_temp {
            self.write(name);
        } else {
            let binding_name = self.get_identifier_text(rest_target);
            if binding_name.is_empty() {
                return;
            }
            self.write(&binding_name);
        }
        self.write(" = ");
        self.write(temp_name);
        self.write(".slice(");
        self.write(&index.to_string());
        self.write(")");

        if let Some(ref name) = rest_temp {
            self.emit_param_binding_assignments(rest_target, name, started);
        }
    }

    fn emit_param_assignment_prefix(&mut self, started: &mut bool) {
        if !*started {
            self.write("var ");
            *started = true;
        } else {
            self.write(", ");
        }
    }

    fn emit_es5_object_rest_element(
        &mut self,
        elem: &crate::parser::thin_node::BindingElementData,
        rest_props: &[NodeIndex],
        temp_name: &str,
    ) {
        let rest_target = elem.name;
        let is_pattern = self.is_binding_pattern(rest_target);
        let rest_temp = if is_pattern {
            Some(self.get_temp_var_name())
        } else {
            None
        };

        self.write(", ");
        if let Some(ref name) = rest_temp {
            self.write(name);
        } else {
            self.emit(rest_target);
        }
        self.write(" = __rest(");
        self.write(temp_name);
        self.write(", ");
        self.emit_rest_exclude_list(rest_props);
        self.write(")");

        if let Some(ref name) = rest_temp {
            self.emit_es5_destructuring_pattern_idx(rest_target, name);
        }
    }

    fn emit_es5_array_rest_element(&mut self, rest_target: NodeIndex, temp_name: &str, index: usize) {
        let is_pattern = self.is_binding_pattern(rest_target);
        let rest_temp = if is_pattern {
            Some(self.get_temp_var_name())
        } else {
            None
        };

        self.write(", ");
        if let Some(ref name) = rest_temp {
            self.write(name);
        } else {
            let binding_name = self.get_identifier_text(rest_target);
            if binding_name.is_empty() {
                return;
            }
            self.write(&binding_name);
        }
        self.write(" = ");
        self.write(temp_name);
        self.write(".slice(");
        self.write(&index.to_string());
        self.write(")");

        if let Some(ref name) = rest_temp {
            self.emit_es5_destructuring_pattern_idx(rest_target, name);
        }
    }

    fn emit_es5_destructuring_pattern_idx(&mut self, pattern_idx: NodeIndex, temp_name: &str) {
        let Some(pattern_node) = self.arena.get(pattern_idx) else { return };
        self.emit_es5_destructuring_pattern(pattern_node, temp_name);
    }

    fn collect_object_rest_props(&self, pattern: &crate::parser::thin_node::BindingPatternData) -> Vec<NodeIndex> {
        let mut props = Vec::new();
        for &elem_idx in &pattern.elements.nodes {
            let Some(elem_node) = self.arena.get(elem_idx) else { continue };
            let Some(elem) = self.arena.get_binding_element(elem_node) else { continue };
            if elem.dot_dot_dot_token {
                continue;
            }
            let key_idx = if !elem.property_name.is_none() {
                elem.property_name
            } else {
                elem.name
            };
            if let Some(key_node) = self.arena.get(key_idx) {
                if key_node.kind == syntax_kind_ext::OBJECT_BINDING_PATTERN
                    || key_node.kind == syntax_kind_ext::ARRAY_BINDING_PATTERN
                {
                    continue;
                }
            }
            props.push(key_idx);
        }
        props
    }

    fn emit_rest_exclude_list(&mut self, props: &[NodeIndex]) {
        self.write("[");
        let mut first = true;
        for &prop_idx in props {
            if !first {
                self.write(", ");
            }
            first = false;
            self.emit_rest_property_key(prop_idx);
        }
        self.write("]");
    }

    fn emit_rest_property_key(&mut self, key_idx: NodeIndex) {
        let Some(key_node) = self.arena.get(key_idx) else { return };

        if key_node.kind == syntax_kind_ext::COMPUTED_PROPERTY_NAME {
            if let Some(computed) = self.arena.get_computed_property(key_node) {
                self.emit_expression(computed.expression);
            }
            return;
        }

        if let Some(ident) = self.arena.get_identifier(key_node) {
            self.write("\"");
            self.write(&ident.escaped_text);
            self.write("\"");
            return;
        }

        if let Some(lit) = self.arena.get_literal(key_node) {
            self.write("\"");
            self.write(&lit.text);
            self.write("\"");
            return;
        }

        self.emit_expression(key_idx);
    }

    /// Get identifier text from a node index
    fn get_identifier_text(&self, idx: NodeIndex) -> String {
        let Some(node) = self.arena.get(idx) else { return String::new() };
        if let Some(ident) = self.arena.get_identifier(node) {
            return ident.escaped_text.clone();
        }
        String::new()
    }

    fn emit_expression_statement(&mut self, node: &ThinNode) {
        let Some(expr_stmt) = self.arena.get_expression_statement(node) else {
            return;
        };

        self.emit(expr_stmt.expression);
        self.write_semicolon();

        // Emit trailing comments: find the position after the semicolon
        // We scan backwards from the expression end to find the semicolon.
        if let Some(text) = self.source_text {
            // Find the semicolon by scanning backwards from the statement end
            let bytes = text.as_bytes();
            let stmt_end = std::cmp::min(node.end as usize, bytes.len());

            // Scan backwards to find the semicolon
            let mut semi_pos = None;
            let mut i = stmt_end;
            while i > 0 {
                i -= 1;
                let ch = bytes[i] as char;
                if ch == ';' {
                    semi_pos = Some(i + 1); // Position after semicolon
                    break;
                } else if ch == '\n' || ch == '\r' {
                    // Stop at newline if no semicolon found (ASI case)
                    break;
                } else if ch == ' ' || ch == '\t' || ch == '/' {
                    // Skip whitespace and potential comment start (scanning backwards)
                    continue;
                } else {
                    // Some other character
                    continue;
                }
            }

            // Emit trailing comments from after the semicolon
            if let Some(pos) = semi_pos {
                let comments = get_trailing_comment_ranges(text, pos);
                for comment in comments {
                    self.write_space();
                    let comment_text = &text[comment.pos as usize..comment.end as usize];
                    self.write(comment_text);
                }
            }
        }
    }

    fn emit_if_statement(&mut self, node: &ThinNode) {
        let Some(if_stmt) = self.arena.get_if_statement(node) else {
            return;
        };

        self.write("if (");
        self.emit(if_stmt.expression);
        self.write(") ");
        self.emit(if_stmt.then_statement);

        if !if_stmt.else_statement.is_none() {
            self.write(" else ");
            self.emit(if_stmt.else_statement);
        }
    }

    fn emit_while_statement(&mut self, node: &ThinNode) {
        let Some(loop_stmt) = self.arena.get_loop(node) else {
            return;
        };

        self.write("while (");
        self.emit(loop_stmt.condition);
        self.write(") ");
        self.emit(loop_stmt.statement);
    }

    fn emit_for_statement(&mut self, node: &ThinNode) {
        let Some(loop_stmt) = self.arena.get_loop(node) else {
            return;
        };

        self.write("for (");
        self.emit(loop_stmt.initializer);
        self.write("; ");
        self.emit(loop_stmt.condition);
        self.write("; ");
        self.emit(loop_stmt.incrementor);
        self.write(") ");
        self.emit(loop_stmt.statement);
    }

    fn emit_for_in_statement(&mut self, node: &ThinNode) {
        let Some(for_in_of) = self.arena.get_for_in_of(node) else {
            return;
        };

        self.write("for (");
        self.emit(for_in_of.initializer);
        self.write(" in ");
        self.emit(for_in_of.expression);
        self.write(") ");
        self.emit(for_in_of.statement);
    }

    fn emit_for_of_statement(&mut self, node: &ThinNode) {
        let Some(for_in_of) = self.arena.get_for_in_of(node) else {
            return;
        };

        if self.ctx.target_es5 && !for_in_of.await_modifier {
            self.emit_for_of_statement_es5(for_in_of);
            return;
        }

        self.write("for ");
        if for_in_of.await_modifier {
            self.write("await ");
        }
        self.write("(");
        self.emit(for_in_of.initializer);
        self.write(" of ");
        self.emit(for_in_of.expression);
        self.write(") ");
        self.emit(for_in_of.statement);
    }

    fn emit_for_of_statement_es5(&mut self, for_in_of: &crate::parser::thin_node::ForInOfData) {
        let error_name = self.get_temp_var_name();
        let return_name = self.get_temp_var_name();
        let iterator_name = self.get_temp_var_name();
        let result_name = self.get_temp_var_name();

        self.write("var ");
        self.write(&error_name);
        self.write(", ");
        self.write(&return_name);
        self.write_semicolon();
        self.write_line();

        self.write("try {");
        self.write_line();
        self.increase_indent();

        self.write("for (var ");
        self.write(&iterator_name);
        self.write(" = __values(");
        self.emit_expression(for_in_of.expression);
        self.write("), ");
        self.write(&result_name);
        self.write(" = ");
        self.write(&iterator_name);
        self.write(".next(); !");
        self.write(&result_name);
        self.write(".done; ");
        self.write(&result_name);
        self.write(" = ");
        self.write(&iterator_name);
        self.write(".next()) ");

        self.write("{");
        self.write_line();
        self.increase_indent();
        self.emit_for_of_value_binding_es5(for_in_of.initializer, &result_name);
        self.write_line();
        self.emit(for_in_of.statement);
        self.write_line();
        self.decrease_indent();
        self.write("}");
        self.write_line();

        self.decrease_indent();
        self.write("}");
        self.write_line();

        self.write("catch (");
        self.write(&error_name);
        self.write("_1) { ");
        self.write(&error_name);
        self.write(" = { error: ");
        self.write(&error_name);
        self.write("_1 }; }");
        self.write_line();

        self.write("finally {");
        self.write_line();
        self.increase_indent();

        self.write("try {");
        self.write_line();
        self.increase_indent();

        self.write("if (");
        self.write(&result_name);
        self.write(" && !");
        self.write(&result_name);
        self.write(".done && (");
        self.write(&return_name);
        self.write(" = ");
        self.write(&iterator_name);
        self.write(".return)) ");
        self.write(&return_name);
        self.write(".call(");
        self.write(&iterator_name);
        self.write(")");
        self.write_semicolon();
        self.write_line();

        self.decrease_indent();
        self.write("} finally {");
        self.write_line();
        self.increase_indent();

        self.write("if (");
        self.write(&error_name);
        self.write(") throw ");
        self.write(&error_name);
        self.write(".error");
        self.write_semicolon();
        self.write_line();

        self.decrease_indent();
        self.write("}");
        self.write_line();

        self.decrease_indent();
        self.write("}");
    }

    fn emit_for_of_value_binding_es5(&mut self, initializer: NodeIndex, result_name: &str) {
        if initializer.is_none() {
            return;
        }

        let Some(init_node) = self.arena.get(initializer) else {
            return;
        };

        if init_node.kind == syntax_kind_ext::VARIABLE_DECLARATION_LIST {
            self.write("var ");
            if let Some(decl_list) = self.arena.get_variable(init_node) {
                let mut first = true;
                for &decl_idx in &decl_list.declarations.nodes {
                    self.emit_for_of_declaration_value_es5(decl_idx, result_name, &mut first);
                }
            }
            self.write_semicolon();
        } else if self.is_binding_pattern(initializer) {
            self.write("var ");
            let mut first = true;
            self.emit_es5_destructuring_from_value(initializer, result_name, &mut first);
            self.write_semicolon();
        } else {
            self.emit_expression(initializer);
            self.write(" = ");
            self.write(result_name);
            self.write(".value");
            self.write_semicolon();
        }
    }

    fn emit_for_of_declaration_value_es5(&mut self, decl_idx: NodeIndex, result_name: &str, first: &mut bool) {
        let Some(decl_node) = self.arena.get(decl_idx) else { return };
        let Some(decl) = self.arena.get_variable_declaration(decl_node) else { return };

        if self.is_binding_pattern(decl.name) {
            self.emit_es5_destructuring_from_value(decl.name, result_name, first);
            return;
        }

        if !*first {
            self.write(", ");
        }
        *first = false;
        self.emit(decl.name);
        self.write(" = ");
        self.write(result_name);
        self.write(".value");
    }

    fn emit_return_statement(&mut self, node: &ThinNode) {
        let Some(ret) = self.arena.get_return_statement(node) else {
            self.write("return");
            self.write_semicolon();
            return;
        };

        self.write("return");
        if !ret.expression.is_none() {
            self.write(" ");
            self.emit_expression(ret.expression);
        }
        self.write_semicolon();
    }

    // =========================================================================
    // Classes
    // =========================================================================

    /// Emit a class declaration.
    ///
    /// **Architecture Note**: This method contains the old inline transform logic for
    /// backward compatibility. When transforms are provided via TransformContext,
    /// the transform system handles ES5/CommonJS transforms and this method is NOT called.
    ///
    /// New code should use: LoweringPass → TransformContext → apply_transform()
    fn emit_class_declaration(&mut self, node: &ThinNode, idx: NodeIndex) {
        let Some(class) = self.arena.get_class(node) else {
            return;
        };

        // Skip ambient declarations (declare class)
        if self.has_declare_modifier(&class.modifiers) {
            return;
        }

        let is_exported = self.ctx.is_commonjs()
            && self.has_export_modifier(&class.modifiers)
            && !self.ctx.module_state.has_export_assignment;
        let is_default = self.has_default_modifier(&class.modifiers);

        // Get class name for export
        let class_name = if !class.name.is_none() {
            self.get_identifier_text_idx(class.name)
        } else {
            String::new()
        };

        // Use ES5 IIFE transform when targeting ES5 (OLD PATH - for backward compatibility)
        // When TransformContext is used, this is handled by TransformDirective::ES5Class
        if self.ctx.target_es5 {
            let mut es5_emitter = ClassES5Emitter::new(self.arena);
            es5_emitter.set_indent_level(self.writer.indent_level());
            if let Some(source_text) = self.source_text {
                es5_emitter.set_source_text(source_text);
            }
            let es5_output = es5_emitter.emit_class(idx);
            self.write(&es5_output);

            // CommonJS: emit exports.ClassName = ClassName; after the ES5 class
            if is_exported && !class_name.is_empty() {
                self.write_line();
                if is_default {
                    self.write("exports.default = ");
                } else {
                    self.write("exports.");
                    self.write(&class_name);
                    self.write(" = ");
                }
                self.write(&class_name);
                self.write(";");
            }
            return;
        }

        // ES6+ path: emit native class syntax
        self.emit_class_es6(node, idx);

        // CommonJS: emit exports.ClassName = ClassName; after the class (OLD PATH)
        // When TransformContext is used, this is handled by TransformDirective::CommonJSExport
        if is_exported && !class_name.is_empty() {
            self.write_line();
            if is_default {
                self.write("exports.default = ");
            } else {
                self.write("exports.");
                self.write(&class_name);
                self.write(" = ");
            }
            self.write(&class_name);
            self.write(";");
        }
    }

    /// Emit a class using ES6 native class syntax (no transforms).
    /// This is the pure emission logic that can be reused by both the old API
    /// and the new transform system.
    fn emit_class_es6(&mut self, node: &ThinNode, _idx: NodeIndex) {
        let Some(class) = self.arena.get_class(node) else {
            return;
        };

        // Emit modifiers (including decorators) - skip export/default for CommonJS
        if let Some(ref modifiers) = class.modifiers {
            for &mod_idx in &modifiers.nodes {
                if let Some(mod_node) = self.arena.get(mod_idx) {
                    // Skip export/default modifiers in CommonJS mode
                    if self.ctx.is_commonjs() {
                        if mod_node.kind == SyntaxKind::ExportKeyword as u16 ||
                           mod_node.kind == SyntaxKind::DefaultKeyword as u16 {
                            continue;
                        }
                    }
                    self.emit(mod_idx);
                    // Add space or newline after decorator
                    if mod_node.kind == syntax_kind_ext::DECORATOR {
                        self.write_line();
                    } else {
                        self.write_space();
                    }
                }
            }
        }

        self.write("class");

        if !class.name.is_none() {
            self.write_space();
            self.emit(class.name);
        }

        if let Some(ref heritage_clauses) = class.heritage_clauses {
            for &clause_idx in &heritage_clauses.nodes {
                let Some(clause_node) = self.arena.get(clause_idx) else {
                    continue;
                };
                let Some(heritage) = self.arena.get_heritage(clause_node) else {
                    continue;
                };
                if heritage.token != SyntaxKind::ExtendsKeyword as u16 {
                    continue;
                }

                if let Some(&extends_type) = heritage.types.nodes.first() {
                    self.write(" extends ");
                    self.emit_heritage_expression(extends_type);
                }
                break;
            }
        }

        self.write(" {");
        self.write_line();
        self.increase_indent();

        for &member_idx in &class.members.nodes {
            self.emit(member_idx);
            self.write_line();
        }

        self.decrease_indent();
        self.write("}");
    }

    // =========================================================================
    // Types
    // =========================================================================

    fn emit_type_reference(&mut self, node: &ThinNode) {
        let Some(type_ref) = self.arena.get_type_ref(node) else {
            return;
        };

        self.emit(type_ref.type_name);

        if let Some(ref type_args) = type_ref.type_arguments {
            if !type_args.nodes.is_empty() {
                self.write("<");
                self.emit_comma_separated(&type_args.nodes);
                self.write(">");
            }
        }
    }

    // =========================================================================
    // Helpers
    // =========================================================================

    fn emit_comma_separated(&mut self, nodes: &[NodeIndex]) {
        let mut first = true;
        for &idx in nodes {
            if !first {
                self.write(", ");
            }
            first = false;
            self.emit(idx);
        }
    }

    fn emit_heritage_expression(&mut self, idx: NodeIndex) {
        if idx.is_none() {
            return;
        }

        let Some(node) = self.arena.get(idx) else {
            return;
        };

        if let Some(expr) = self.arena.get_expr_type_args(node) {
            self.emit(expr.expression);
        } else {
            self.emit(idx);
        }
    }

    // =========================================================================
    // JSX
    // =========================================================================

    fn emit_jsx_element(&mut self, node: &ThinNode) {
        let Some(jsx) = self.arena.get_jsx_element(node) else {
            return;
        };

        self.emit(jsx.opening_element);
        for &child in &jsx.children.nodes {
            self.emit(child);
        }
        self.emit(jsx.closing_element);
    }

    fn emit_jsx_self_closing_element(&mut self, node: &ThinNode) {
        let Some(jsx) = self.arena.get_jsx_opening(node) else {
            return;
        };

        self.write("<");
        self.emit(jsx.tag_name);
        self.emit(jsx.attributes);
        self.write(" />");
    }

    fn emit_jsx_opening_element(&mut self, node: &ThinNode) {
        let Some(jsx) = self.arena.get_jsx_opening(node) else {
            return;
        };

        self.write("<");
        self.emit(jsx.tag_name);
        self.emit(jsx.attributes);
        self.write(">");
    }

    fn emit_jsx_closing_element(&mut self, node: &ThinNode) {
        let Some(jsx) = self.arena.get_jsx_closing(node) else {
            return;
        };

        self.write("</");
        self.emit(jsx.tag_name);
        self.write(">");
    }

    fn emit_jsx_fragment(&mut self, node: &ThinNode) {
        let Some(jsx) = self.arena.get_jsx_fragment(node) else {
            return;
        };

        self.write("<>");
        for &child in &jsx.children.nodes {
            self.emit(child);
        }
        self.write("</>");
    }

    fn emit_jsx_attributes(&mut self, node: &ThinNode) {
        let Some(attrs) = self.arena.get_jsx_attributes(node) else {
            return;
        };

        for &attr in &attrs.properties.nodes {
            self.write_space();
            self.emit(attr);
        }
    }

    fn emit_jsx_attribute(&mut self, node: &ThinNode) {
        let Some(attr) = self.arena.get_jsx_attribute(node) else {
            return;
        };

        self.emit(attr.name);
        if !attr.initializer.is_none() {
            self.write("=");
            self.emit(attr.initializer);
        }
    }

    fn emit_jsx_spread_attribute(&mut self, node: &ThinNode) {
        let Some(spread) = self.arena.get_jsx_spread_attribute(node) else {
            return;
        };

        self.write("{...");
        self.emit(spread.expression);
        self.write("}");
    }

    fn emit_jsx_expression(&mut self, node: &ThinNode) {
        let Some(expr) = self.arena.get_jsx_expression(node) else {
            return;
        };

        self.write("{");
        if expr.dot_dot_dot_token {
            self.write("...");
        }
        self.emit(expr.expression);
        self.write("}");
    }

    fn emit_jsx_text(&mut self, node: &ThinNode) {
        let Some(text) = self.arena.get_jsx_text(node) else {
            return;
        };

        self.write(&text.text);
    }

    fn emit_jsx_namespaced_name(&mut self, node: &ThinNode) {
        let Some(ns) = self.arena.get_jsx_namespaced_name(node) else {
            return;
        };

        self.emit(ns.namespace);
        self.write(":");
        self.emit(ns.name);
    }

    // =========================================================================
    // Imports/Exports
    // =========================================================================

    fn emit_import_declaration(&mut self, node: &ThinNode) {
        if self.ctx.is_commonjs() {
            self.emit_import_declaration_commonjs(node);
        } else {
            self.emit_import_declaration_es6(node);
        }
    }

    fn emit_import_declaration_es6(&mut self, node: &ThinNode) {
        let Some(import) = self.arena.get_import_decl(node) else {
            return;
        };

        self.write("import ");

        if !import.import_clause.is_none() {
            self.emit(import.import_clause);
            self.write(" from ");
        }

        self.emit(import.module_specifier);
        self.write_semicolon();
    }

    fn emit_import_declaration_commonjs(&mut self, node: &ThinNode) {
        use crate::transforms::module_commonjs;

        let Some(import) = self.arena.get_import_decl(node) else {
            return;
        };

        // Skip type-only imports in CommonJS
        if import.import_clause.is_none() {
            return; // Side-effect import: import "module"; -> skip or emit require
        }

        // Get module specifier and generate var name
        let module_spec = if let Some(spec_node) = self.arena.get(import.module_specifier) {
            if let Some(lit) = self.arena.get_literal(spec_node) {
                lit.text.clone()
            } else {
                return;
            }
        } else {
            return;
        };

        // Generate module var name: "./foo" -> "foo_1"
        let module_var = format!("{}_1", module_commonjs::sanitize_module_name(&module_spec));

        // Emit: var module_1 = require("module");
        self.write("var ");
        self.write(&module_var);
        self.write(" = require(\"");
        self.write(&module_spec);
        self.write("\");");
        self.write_line();

        // Emit bindings
        let bindings = module_commonjs::get_import_bindings(self.arena, node, &module_var);
        for binding in bindings {
            self.write(&binding);
            self.write_line();
        }
    }

    fn emit_import_clause(&mut self, node: &ThinNode) {
        let Some(clause) = self.arena.get_import_clause(node) else {
            return;
        };

        let mut has_default = false;

        // Default import
        if !clause.name.is_none() {
            self.emit(clause.name);
            has_default = true;
        }

        // Named bindings
        if !clause.named_bindings.is_none() {
            if has_default {
                self.write(", ");
            }
            self.emit(clause.named_bindings);
        }
    }

    fn emit_named_imports(&mut self, node: &ThinNode) {
        let Some(imports) = self.arena.get_named_imports(node) else {
            return;
        };

        self.write("{ ");
        self.emit_comma_separated(&imports.elements.nodes);
        self.write(" }");
    }

    fn emit_import_specifier(&mut self, node: &ThinNode) {
        let Some(spec) = self.arena.get_specifier(node) else {
            return;
        };

        if !spec.property_name.is_none() {
            self.emit(spec.property_name);
            self.write(" as ");
        }
        self.emit(spec.name);
    }

    fn emit_export_declaration(&mut self, node: &ThinNode) {
        if self.ctx.is_commonjs() {
            self.emit_export_declaration_commonjs(node);
        } else {
            self.emit_export_declaration_es6(node);
        }
    }

    fn emit_export_declaration_es6(&mut self, node: &ThinNode) {
        let Some(export) = self.arena.get_export_decl(node) else {
            return;
        };

        self.write("export ");

        if !export.export_clause.is_none() {
            self.emit(export.export_clause);
        } else {
            self.write("*");
        }

        if !export.module_specifier.is_none() {
            self.write(" from ");
            self.emit(export.module_specifier);
        }

        self.write_semicolon();
    }

    fn emit_export_declaration_commonjs(&mut self, node: &ThinNode) {
        use crate::transforms::module_commonjs;

        let Some(export) = self.arena.get_export_decl(node) else {
            return;
        };

        // Re-export from another module: export { x } from "module";
        if !export.module_specifier.is_none() {
            let module_spec = if let Some(spec_node) = self.arena.get(export.module_specifier) {
                if let Some(lit) = self.arena.get_literal(spec_node) {
                    lit.text.clone()
                } else {
                    return;
                }
            } else {
                return;
            };

            let module_var = format!("{}_1", module_commonjs::sanitize_module_name(&module_spec));

            // First emit the require
            self.write("var ");
            self.write(&module_var);
            self.write(" = require(\"");
            self.write(&module_spec);
            self.write("\");");
            self.write_line();

            if export.export_clause.is_none() {
                self.write("__exportStar(");
                self.write(&module_var);
                self.write(", exports);");
                self.write_line();
                return;
            }

            // Then emit Object.defineProperty for each export
            if let Some(clause_node) = self.arena.get(export.export_clause) {
                if let Some(named_exports) = self.arena.get_named_imports(clause_node) {
                    for &spec_idx in &named_exports.elements.nodes {
                        if let Some(spec_node) = self.arena.get(spec_idx) {
                            if let Some(spec) = self.arena.get_specifier(spec_node) {
                                // Get export name and import name
                                let export_name = self.get_identifier_text_idx(spec.name);
                                let import_name = if !spec.property_name.is_none() {
                                    self.get_identifier_text_idx(spec.property_name)
                                } else {
                                    export_name.clone()
                                };

                                // Object.defineProperty(exports, "name", { enumerable: true, get: function () { return mod.name; } });
                                self.write("Object.defineProperty(exports, \"");
                                self.write(&export_name);
                                self.write("\", { enumerable: true, get: function () { return ");
                                self.write(&module_var);
                                self.write(".");
                                self.write(&import_name);
                                self.write("; } });");
                                self.write_line();
                            }
                        }
                    }
                }
            }
            return;
        }

        // Check if export_clause contains a declaration (export const x, export function f, etc.)
        if let Some(clause_node) = self.arena.get(export.export_clause) {
            match clause_node.kind {
                // export const/let/var x = ...
                k if k == syntax_kind_ext::VARIABLE_STATEMENT => {
                    // Collect export names before emitting
                    let export_names = self.collect_variable_names_from_node(clause_node);

                    // Emit the variable declaration
                    self.emit_variable_statement(clause_node);
                    self.write_line();

                    // Emit exports.x = x; for each name (unless file has export =)
                    if !self.ctx.module_state.has_export_assignment {
                        for name in &export_names {
                            self.write("exports.");
                            self.write(name);
                            self.write(" = ");
                            self.write(name);
                            self.write(";");
                            self.write_line();
                        }
                    }
                }
                // export function f() {} or export default function f() {}
                k if k == syntax_kind_ext::FUNCTION_DECLARATION => {
                    // Emit the function declaration
                    self.emit_function_declaration(clause_node, export.export_clause);
                    self.write_line();

                    // Get function name and emit export (unless file has export =)
                    if !self.ctx.module_state.has_export_assignment {
                        if let Some(func) = self.arena.get_function(clause_node) {
                            if let Some(name) = self.get_identifier_text_opt(func.name) {
                                if export.is_default_export {
                                    self.write("exports.default = ");
                                } else {
                                    self.write("exports.");
                                    self.write(&name);
                                    self.write(" = ");
                                }
                                self.write(&name);
                                self.write(";");
                                self.write_line();
                            }
                        }
                    }
                }
                // export class C {} or export default class C {}
                k if k == syntax_kind_ext::CLASS_DECLARATION => {
                    // Emit the class declaration
                    self.emit_class_declaration(clause_node, export.export_clause);
                    self.write_line();

                    // Get class name and emit export (unless file has export =)
                    if !self.ctx.module_state.has_export_assignment {
                        if let Some(class) = self.arena.get_class(clause_node) {
                            if let Some(name) = self.get_identifier_text_opt(class.name) {
                                if export.is_default_export {
                                    self.write("exports.default = ");
                                } else {
                                    self.write("exports.");
                                    self.write(&name);
                                    self.write(" = ");
                                }
                                self.write(&name);
                                self.write(";");
                                self.write_line();
                            }
                        }
                    }
                }
                // export { x, y } - local re-export without module specifier
                k if k == syntax_kind_ext::NAMED_EXPORTS => {
                    // Emit exports.x = x; for each name
                    if let Some(named_exports) = self.arena.get_named_imports(clause_node) {
                        for &spec_idx in &named_exports.elements.nodes {
                            if let Some(spec_node) = self.arena.get(spec_idx) {
                                if let Some(spec) = self.arena.get_specifier(spec_node) {
                                    let export_name = self.get_identifier_text_idx(spec.name);
                                    let local_name = if !spec.property_name.is_none() {
                                        self.get_identifier_text_idx(spec.property_name)
                                    } else {
                                        export_name.clone()
                                    };

                                    self.write("exports.");
                                    self.write(&export_name);
                                    self.write(" = ");
                                    self.write(&local_name);
                                    self.write(";");
                                    self.write_line();
                                }
                            }
                        }
                    }
                }
                // Type-only declarations (interface, type alias) - skip for CommonJS
                k if k == syntax_kind_ext::INTERFACE_DECLARATION => {}
                k if k == syntax_kind_ext::TYPE_ALIAS_DECLARATION => {}
                // export default <expression> - emit as exports.default = expr;
                _ => {
                    // This is likely an expression-based default export: export default 42;
                    self.write("exports.default = ");
                    self.emit(export.export_clause);
                    self.write_semicolon();
                }
            }
        }
    }

    /// Emit export assignment (export = expr or export default expr)
    fn emit_export_assignment(&mut self, node: &ThinNode) {
        let Some(export_assign) = self.arena.get_export_assignment(node) else {
            return;
        };

        if self.ctx.is_commonjs() {
            // CommonJS: export = expr → module.exports = expr;
            //           export default expr → exports.default = expr;
            if export_assign.is_export_equals {
                self.write("module.exports = ");
            } else {
                self.write("exports.default = ");
            }
            self.emit_expression(export_assign.expression);
            self.write_semicolon();
        } else {
            // ES6: export = expr (not valid ES6, but emit as export default)
            //      export default expr → export default expr;
            self.write("export default ");
            self.emit_expression(export_assign.expression);
            self.write_semicolon();
        }
    }

    /// Collect variable names from a VARIABLE_STATEMENT node
    fn collect_variable_names_from_node(&self, node: &ThinNode) -> Vec<String> {
        let mut names = Vec::new();
        if let Some(var_stmt) = self.arena.get_variable(node) {
            // VARIABLE_STATEMENT has declarations containing VARIABLE_DECLARATION_LIST
            for &decl_list_idx in &var_stmt.declarations.nodes {
                if let Some(decl_list_node) = self.arena.get(decl_list_idx) {
                    // VARIABLE_DECLARATION_LIST has declarations containing VARIABLE_DECLARATION
                    if let Some(decl_list) = self.arena.get_variable(decl_list_node) {
                        for &decl_idx in &decl_list.declarations.nodes {
                            if let Some(decl_node) = self.arena.get(decl_idx) {
                                if let Some(decl) = self.arena.get_variable_declaration(decl_node) {
                                    self.collect_binding_names(decl.name, &mut names);
                                }
                            }
                        }
                    }
                }
            }
        }
        names
    }

    /// Get identifier text from optional node index
    fn get_identifier_text_opt(&self, idx: NodeIndex) -> Option<String> {
        let node = self.arena.get(idx)?;
        if node.kind == SyntaxKind::Identifier as u16 {
            self.arena.get_identifier(node).map(|id| id.escaped_text.clone())
        } else {
            None
        }
    }

    /// Get identifier text from a node index
    fn get_identifier_text_idx(&self, idx: NodeIndex) -> String {
        if let Some(node) = self.arena.get(idx) {
            if node.kind == SyntaxKind::Identifier as u16 {
                if let Some(id) = self.arena.get_identifier(node) {
                    return id.escaped_text.clone();
                }
            }
        }
        String::new()
    }

    fn emit_named_exports(&mut self, node: &ThinNode) {
        // Named exports uses the same data structure as named imports
        let Some(exports) = self.arena.get_named_imports(node) else {
            self.write("{ }");
            return;
        };

        self.write("{ ");
        self.emit_comma_separated(&exports.elements.nodes);
        self.write(" }");
    }

    fn emit_export_specifier(&mut self, node: &ThinNode) {
        let Some(spec) = self.arena.get_specifier(node) else {
            return;
        };

        if !spec.property_name.is_none() {
            self.emit(spec.property_name);
            self.write(" as ");
        }
        self.emit(spec.name);
    }

    // =========================================================================
    // Additional Statements
    // =========================================================================

    fn emit_throw_statement(&mut self, node: &ThinNode) {
        // ThrowStatement uses ReturnData (same structure)
        let Some(throw_data) = self.arena.get_return_statement(node) else {
            self.write("throw");
            self.write_semicolon();
            return;
        };

        self.write("throw ");
        self.emit(throw_data.expression);
        self.write_semicolon();
    }

    fn emit_try_statement(&mut self, node: &ThinNode) {
        let Some(try_stmt) = self.arena.get_try(node) else {
            return;
        };

        self.write("try ");
        self.emit(try_stmt.try_block);

        if !try_stmt.catch_clause.is_none() {
            self.write(" ");
            self.emit(try_stmt.catch_clause);
        }

        if !try_stmt.finally_block.is_none() {
            self.write(" finally ");
            self.emit(try_stmt.finally_block);
        }
    }

    fn emit_catch_clause(&mut self, node: &ThinNode) {
        let Some(catch) = self.arena.get_catch_clause(node) else {
            return;
        };

        self.write("catch");

        if !catch.variable_declaration.is_none() {
            self.write(" (");
            self.emit(catch.variable_declaration);
            self.write(")");
        }

        self.write(" ");
        self.emit(catch.block);
    }

    fn emit_switch_statement(&mut self, node: &ThinNode) {
        let Some(switch) = self.arena.get_switch(node) else {
            return;
        };

        self.write("switch (");
        self.emit(switch.expression);
        self.write(") ");
        // case_block is a NodeIndex pointing to a CaseBlock node
        self.emit(switch.case_block);
    }

    fn emit_case_block(&mut self, node: &ThinNode) {
        if !node.has_data() || node.kind != syntax_kind_ext::CASE_BLOCK {
            return;
        }
        let Some(case_block) = self.arena.blocks.get(node.data_index as usize) else {
            return;
        };

        self.write("{");
        self.write_line();
        self.increase_indent();

        for &clause_idx in &case_block.statements.nodes {
            self.emit(clause_idx);
            self.write_line();
        }

        self.decrease_indent();
        self.write("}");
    }

    fn emit_case_clause(&mut self, node: &ThinNode) {
        let Some(clause) = self.arena.get_case_clause(node) else {
            return;
        };

        self.write("case ");
        self.emit(clause.expression);
        self.write(":");
        self.write_line();
        self.increase_indent();

        for &stmt in &clause.statements.nodes {
            self.emit(stmt);
            self.write_line();
        }

        self.decrease_indent();
    }

    fn emit_default_clause(&mut self, node: &ThinNode) {
        let Some(clause) = self.arena.get_case_clause(node) else {
            return;
        };

        self.write("default:");
        self.write_line();
        self.increase_indent();

        for &stmt in &clause.statements.nodes {
            self.emit(stmt);
            self.write_line();
        }

        self.decrease_indent();
    }

    fn emit_break_statement(&mut self) {
        self.write("break");
        self.write_semicolon();
    }

    fn emit_continue_statement(&mut self) {
        self.write("continue");
        self.write_semicolon();
    }

    fn emit_do_statement(&mut self, node: &ThinNode) {
        let Some(loop_stmt) = self.arena.get_loop(node) else {
            return;
        };

        self.write("do ");
        self.emit(loop_stmt.statement);
        self.write(" while (");
        self.emit(loop_stmt.condition);
        self.write(")");
        self.write_semicolon();
    }

    fn emit_debugger_statement(&mut self) {
        self.write("debugger");
        self.write_semicolon();
    }

    // =========================================================================
    // Type Emit Methods
    // =========================================================================

    fn emit_union_type(&mut self, node: &ThinNode) {
        let Some(union) = self.arena.get_composite_type(node) else {
            return;
        };

        let mut first = true;
        for &type_idx in &union.types.nodes {
            if !first {
                self.write(" | ");
            }
            first = false;
            self.emit(type_idx);
        }
    }

    fn emit_intersection_type(&mut self, node: &ThinNode) {
        let Some(intersection) = self.arena.get_composite_type(node) else {
            return;
        };

        let mut first = true;
        for &type_idx in &intersection.types.nodes {
            if !first {
                self.write(" & ");
            }
            first = false;
            self.emit(type_idx);
        }
    }

    fn emit_array_type(&mut self, node: &ThinNode) {
        let Some(array) = self.arena.get_array_type(node) else {
            return;
        };

        self.emit(array.element_type);
        self.write("[]");
    }

    fn emit_tuple_type(&mut self, node: &ThinNode) {
        let Some(tuple) = self.arena.get_tuple_type(node) else {
            self.write("[]");
            return;
        };

        self.write("[");
        self.emit_comma_separated(&tuple.elements.nodes);
        self.write("]");
    }

    fn emit_function_type(&mut self, node: &ThinNode) {
        let Some(func_type) = self.arena.get_function_type(node) else {
            return;
        };

        // Type parameters
        if let Some(ref type_params) = func_type.type_parameters {
            if !type_params.nodes.is_empty() {
                self.write("<");
                self.emit_comma_separated(&type_params.nodes);
                self.write(">");
            }
        }

        // Parameters
        self.write("(");
        self.emit_comma_separated(&func_type.parameters.nodes);
        self.write(") => ");

        // Return type
        self.emit(func_type.type_annotation);
    }

    fn emit_type_literal(&mut self, node: &ThinNode) {
        let Some(type_lit) = self.arena.get_type_literal(node) else {
            self.write("{}");
            return;
        };

        if type_lit.members.nodes.is_empty() {
            self.write("{}");
            return;
        }

        self.write("{");
        self.write_line();
        self.increase_indent();

        for &member_idx in &type_lit.members.nodes {
            self.emit(member_idx);
            self.write_semicolon();
            self.write_line();
        }

        self.decrease_indent();
        self.write("}");
    }

    fn emit_parenthesized_type(&mut self, node: &ThinNode) {
        let Some(paren_type) = self.arena.get_wrapped_type(node) else {
            return;
        };

        self.write("(");
        self.emit(paren_type.type_node);
        self.write(")");
    }

    // =========================================================================
    // Declarations - Enum, Interface, Type Alias
    // =========================================================================

    fn emit_enum_declaration(&mut self, node: &ThinNode, idx: NodeIndex) {
        let Some(enum_decl) = self.arena.get_enum(node) else {
            return;
        };

        // Skip ambient declarations (declare enum)
        if self.has_declare_modifier(&enum_decl.modifiers) {
            return;
        }

        // For ES5 target: transform to IIFE pattern
        if self.ctx.target_es5 {
            let mut enum_emitter = crate::transforms::enum_es5::EnumES5Emitter::new(self.arena);
            enum_emitter.set_indent_level(self.writer.indent_level());
            let output = enum_emitter.emit_enum(idx);
            self.write(&output);
            return;
        }

        // For modern targets: emit TypeScript-style enum
        self.write("enum ");
        self.emit(enum_decl.name);
        self.write(" {");
        self.write_line();
        self.increase_indent();

        for &member_idx in &enum_decl.members.nodes {
            self.emit(member_idx);
            self.write(",");
            self.write_line();
        }

        self.decrease_indent();
        self.write("}");
    }

    fn emit_enum_member(&mut self, node: &ThinNode) {
        let Some(member) = self.arena.get_enum_member(node) else {
            return;
        };

        self.emit(member.name);

        if !member.initializer.is_none() {
            self.write(" = ");
            self.emit(member.initializer);
        }
    }

    fn emit_interface_declaration(&mut self, node: &ThinNode) {
        let Some(interface) = self.arena.get_interface(node) else {
            return;
        };

        self.write("interface ");
        self.emit(interface.name);

        // Type parameters
        if let Some(ref type_params) = interface.type_parameters {
            if !type_params.nodes.is_empty() {
                self.write("<");
                self.emit_comma_separated(&type_params.nodes);
                self.write(">");
            }
        }

        // Heritage clauses
        if let Some(ref heritage) = interface.heritage_clauses {
            if !heritage.nodes.is_empty() {
                self.write(" extends ");
                self.emit_comma_separated(&heritage.nodes);
            }
        }

        self.write(" {");
        self.write_line();
        self.increase_indent();

        for &member_idx in &interface.members.nodes {
            self.emit(member_idx);
            self.write_semicolon();
            self.write_line();
        }

        self.decrease_indent();
        self.write("}");
    }

    fn emit_type_alias_declaration(&mut self, node: &ThinNode) {
        let Some(type_alias) = self.arena.get_type_alias(node) else {
            return;
        };

        self.write("type ");
        self.emit(type_alias.name);

        // Type parameters
        if let Some(ref type_params) = type_alias.type_parameters {
            if !type_params.nodes.is_empty() {
                self.write("<");
                self.emit_comma_separated(&type_params.nodes);
                self.write(">");
            }
        }

        self.write(" = ");
        self.emit(type_alias.type_node);
        self.write_semicolon();
    }

    fn emit_module_declaration(&mut self, node: &ThinNode, idx: NodeIndex) {
        if self.ctx.target_es5 {
            // Use ES5 namespace transform: namespace → IIFE pattern
            let mut ns_emitter = crate::transforms::namespace_es5::NamespaceES5Emitter::new(self.arena);
            let output = ns_emitter.emit_namespace(idx);
            self.write(&output);
            return;
        }

        let Some(module) = self.arena.get_module(node) else {
            return;
        };

        self.write("namespace ");
        self.emit(module.name);
        self.write(" ");
        self.emit(module.body);
    }

    // =========================================================================
    // Template Literals
    // =========================================================================

    fn emit_template_expression(&mut self, node: &ThinNode) {
        let Some(tpl) = self.arena.get_template_expr(node) else {
            self.write("``");
            return;
        };

        // Emit the template head (opening backtick and initial text)
        self.emit(tpl.head);

        // Emit each template span (expression + middle/tail)
        for &span_idx in &tpl.template_spans.nodes {
            self.emit(span_idx);
        }
    }

    fn emit_no_substitution_template(&mut self, node: &ThinNode) {
        if let Some(lit) = self.arena.get_literal(node) {
            self.write("`");
            self.write(&lit.text);
            self.write("`");
        }
    }

    fn emit_template_span(&mut self, node: &ThinNode) {
        let Some(span) = self.arena.get_template_span(node) else {
            return;
        };

        // Emit ${expression}
        self.write("${");
        self.emit(span.expression);
        self.write("}");
        // Emit the literal part (middle or tail)
        self.emit(span.literal);
    }

    fn emit_template_head(&mut self, node: &ThinNode) {
        if let Some(lit) = self.arena.get_literal(node) {
            // Template head starts with ` and ends with ${
            self.write("`");
            self.write(&lit.text);
        }
    }

    fn emit_template_middle(&mut self, node: &ThinNode) {
        if let Some(lit) = self.arena.get_literal(node) {
            // Template middle is between } and ${
            self.write(&lit.text);
        }
    }

    fn emit_template_tail(&mut self, node: &ThinNode) {
        if let Some(lit) = self.arena.get_literal(node) {
            // Template tail ends with `
            self.write(&lit.text);
            self.write("`");
        }
    }

    // =========================================================================
    // Modifier Helpers
    // =========================================================================

    /// Check if modifiers include the `declare` keyword
    fn has_declare_modifier(&self, modifiers: &Option<NodeList>) -> bool {
        self.has_modifier(modifiers, SyntaxKind::DeclareKeyword as u16)
    }

    /// Check if modifiers include the `export` keyword
    fn has_export_modifier(&self, modifiers: &Option<NodeList>) -> bool {
        self.has_modifier(modifiers, SyntaxKind::ExportKeyword as u16)
    }

    /// Check if modifiers include the `default` keyword
    fn has_default_modifier(&self, modifiers: &Option<NodeList>) -> bool {
        self.has_modifier(modifiers, SyntaxKind::DefaultKeyword as u16)
    }

    /// Check if modifiers include a specific keyword
    fn has_modifier(&self, modifiers: &Option<NodeList>, kind: u16) -> bool {
        if let Some(mods) = modifiers {
            for &mod_idx in &mods.nodes {
                if let Some(mod_node) = self.arena.get(mod_idx) {
                    if mod_node.kind == kind {
                        return true;
                    }
                }
            }
        }
        false
    }

    // =========================================================================
    // Class Members
    // =========================================================================

    /// Emit class member modifiers (static, public, private, etc.)
    fn emit_class_member_modifiers(&mut self, modifiers: &Option<NodeList>) {
        if let Some(mods) = modifiers {
            for &mod_idx in &mods.nodes {
                if let Some(mod_node) = self.arena.get(mod_idx) {
                    // Emit the modifier keyword based on its kind
                    let keyword = match mod_node.kind as u32 {
                        k if k == SyntaxKind::StaticKeyword as u32 => "static",
                        k if k == SyntaxKind::PublicKeyword as u32 => "public",
                        k if k == SyntaxKind::PrivateKeyword as u32 => "private",
                        k if k == SyntaxKind::ProtectedKeyword as u32 => "protected",
                        k if k == SyntaxKind::ReadonlyKeyword as u32 => "readonly",
                        k if k == SyntaxKind::AbstractKeyword as u32 => "abstract",
                        k if k == SyntaxKind::OverrideKeyword as u32 => "override",
                        k if k == SyntaxKind::AsyncKeyword as u32 => "async",
                        k if k == SyntaxKind::DeclareKeyword as u32 => "declare",
                        _ => continue,
                    };
                    self.write(keyword);
                    self.write_space();
                }
            }
        }
    }

    fn emit_method_declaration(&mut self, node: &ThinNode) {
        let Some(method) = self.arena.get_method_decl(node) else {
            return;
        };

        // Skip method declarations without bodies (TypeScript-only overloads)
        if method.body.is_none() {
            return;
        }

        // Emit modifiers (static, async only for JavaScript)
        self.emit_method_modifiers_js(&method.modifiers);

        self.emit(method.name);
        self.write("(");
        self.emit_function_parameters_js(&method.parameters.nodes);
        self.write(")");

        // Skip return type for JavaScript emit

        self.write(" ");
        self.emit(method.body);
    }

    /// Emit method modifiers for JavaScript (static, async only)
    fn emit_method_modifiers_js(&mut self, modifiers: &Option<NodeList>) {
        if let Some(mods) = modifiers {
            for &mod_idx in &mods.nodes {
                if let Some(mod_node) = self.arena.get(mod_idx) {
                    match mod_node.kind {
                        k if k == SyntaxKind::StaticKeyword as u16 => self.write("static "),
                        k if k == SyntaxKind::AsyncKeyword as u16 => self.write("async "),
                        _ => {} // Skip private/protected/public/readonly/abstract
                    }
                }
            }
        }
    }

    fn emit_property_declaration(&mut self, node: &ThinNode) {
        let Some(prop) = self.arena.get_property_decl(node) else {
            return;
        };

        // For JavaScript: Skip property declarations that are TypeScript-only
        // (declarations with type annotation but no initializer)
        if prop.initializer.is_none() && !prop.type_annotation.is_none() {
            return;
        }

        // Emit modifiers (static only for JavaScript)
        self.emit_class_member_modifiers_js(&prop.modifiers);

        self.emit(prop.name);

        // Skip type annotations for JavaScript emit

        if !prop.initializer.is_none() {
            self.write(" = ");
            self.emit(prop.initializer);
        }

        self.write_semicolon();
    }

    /// Emit class member modifiers for JavaScript (only static is valid)
    fn emit_class_member_modifiers_js(&mut self, modifiers: &Option<NodeList>) {
        if let Some(mods) = modifiers {
            for &mod_idx in &mods.nodes {
                if let Some(mod_node) = self.arena.get(mod_idx) {
                    // Only emit 'static' for JavaScript - skip private/readonly/public/protected
                    if mod_node.kind == SyntaxKind::StaticKeyword as u16 {
                        self.write("static ");
                    }
                }
            }
        }
    }

    fn emit_constructor_declaration(&mut self, node: &ThinNode) {
        let Some(ctor) = self.arena.get_constructor(node) else {
            return;
        };

        // Emit modifiers (public, protected, private)
        self.emit_class_member_modifiers(&ctor.modifiers);

        self.write("constructor(");
        self.emit_comma_separated(&ctor.parameters.nodes);
        self.write(")");

        if !ctor.body.is_none() {
            self.write(" ");
            self.emit(ctor.body);
        }
    }

    fn emit_get_accessor(&mut self, node: &ThinNode) {
        let Some(accessor) = self.arena.get_accessor(node) else {
            return;
        };

        // Emit modifiers (static, private, etc.)
        self.emit_class_member_modifiers(&accessor.modifiers);

        self.write("get ");
        self.emit(accessor.name);
        self.write("()");

        // Skip type annotation for JS emit

        if !accessor.body.is_none() {
            self.write(" ");
            self.emit(accessor.body);
        } else {
            // For JS emit, add empty body for accessors without body
            self.write(" { }");
        }
    }

    fn emit_set_accessor(&mut self, node: &ThinNode) {
        let Some(accessor) = self.arena.get_accessor(node) else {
            return;
        };

        // Emit modifiers (static, private, etc.)
        self.emit_class_member_modifiers(&accessor.modifiers);

        self.write("set ");
        self.emit(accessor.name);
        self.write("(");
        self.emit_comma_separated(&accessor.parameters.nodes);
        self.write(")");

        if !accessor.body.is_none() {
            self.write(" ");
            self.emit(accessor.body);
        } else {
            // For JS emit, add empty body for accessors without body
            self.write(" { }");
        }
    }

    // =========================================================================
    // Interface/Type Members (Signatures)
    // =========================================================================

    fn emit_property_signature(&mut self, node: &ThinNode) {
        let Some(sig) = self.arena.get_signature(node) else {
            return;
        };

        // Emit modifiers (readonly)
        self.emit_class_member_modifiers(&sig.modifiers);

        if !sig.name.is_none() {
            self.emit(sig.name);
        }

        if sig.question_token {
            self.write("?");
        }

        if !sig.type_annotation.is_none() {
            self.write(": ");
            self.emit(sig.type_annotation);
        }
    }

    fn emit_method_signature(&mut self, node: &ThinNode) {
        let Some(sig) = self.arena.get_signature(node) else {
            return;
        };

        if !sig.name.is_none() {
            self.emit(sig.name);
        }

        if sig.question_token {
            self.write("?");
        }

        self.write("(");
        if let Some(ref params) = sig.parameters {
            self.emit_comma_separated(&params.nodes);
        }
        self.write(")");

        if !sig.type_annotation.is_none() {
            self.write(": ");
            self.emit(sig.type_annotation);
        }
    }

    fn emit_call_signature(&mut self, node: &ThinNode) {
        let Some(sig) = self.arena.get_signature(node) else {
            return;
        };

        // TODO: type parameters

        self.write("(");
        if let Some(ref params) = sig.parameters {
            self.emit_comma_separated(&params.nodes);
        }
        self.write(")");

        if !sig.type_annotation.is_none() {
            self.write(": ");
            self.emit(sig.type_annotation);
        }
    }

    fn emit_construct_signature(&mut self, node: &ThinNode) {
        let Some(sig) = self.arena.get_signature(node) else {
            return;
        };

        self.write("new ");

        // TODO: type parameters

        self.write("(");
        if let Some(ref params) = sig.parameters {
            self.emit_comma_separated(&params.nodes);
        }
        self.write(")");

        if !sig.type_annotation.is_none() {
            self.write(": ");
            self.emit(sig.type_annotation);
        }
    }

    fn emit_index_signature(&mut self, node: &ThinNode) {
        let Some(sig) = self.arena.get_index_signature(node) else {
            return;
        };

        // Emit modifiers (readonly)
        self.emit_class_member_modifiers(&sig.modifiers);

        self.write("[");
        self.emit_comma_separated(&sig.parameters.nodes);
        self.write("]");

        if !sig.type_annotation.is_none() {
            self.write(": ");
            self.emit(sig.type_annotation);
        }
    }

    // =========================================================================
    // Yield and Await
    // =========================================================================

    fn emit_yield_expression(&mut self, node: &ThinNode) {
        // YieldExpression is stored with UnaryExprData (operand = expression, operator = asterisk flag)
        let Some(unary) = self.arena.get_unary_expr(node) else {
            self.write("yield");
            return;
        };

        self.write("yield");
        // Check if this is yield* (operator stores asterisk flag as SyntaxKind)
        if unary.operator == crate::scanner::SyntaxKind::AsteriskToken as u16 {
            self.write("*");
        }
        if !unary.operand.is_none() {
            self.write(" ");
            self.emit_expression(unary.operand);
        }
    }

    fn emit_await_expression(&mut self, node: &ThinNode) {
        // AwaitExpression is stored with UnaryExprData
        let Some(unary) = self.arena.get_unary_expr(node) else {
            self.write("await");
            return;
        };

        self.write("await ");
        self.emit_expression(unary.operand);
    }

    fn emit_spread_element(&mut self, node: &ThinNode) {
        let Some(spread) = self.arena.get_spread(node) else {
            self.write("...");
            return;
        };

        self.write("...");
        self.emit_expression(spread.expression);
    }

    // =========================================================================
    // Decorators
    // =========================================================================

    fn emit_decorator(&mut self, node: &ThinNode) {
        let Some(decorator) = self.arena.get_decorator(node) else {
            return;
        };

        self.write("@");
        self.emit(decorator.expression);
    }

    // =========================================================================
    // Source File
    // =========================================================================

    fn emit_source_file(&mut self, node: &ThinNode) {
        let Some(source) = self.arena.get_source_file(node) else {
            return;
        };

        // Auto-detect module: if enabled and file has imports/exports, switch to CommonJS
        if self.ctx.auto_detect_module && self.file_is_module(&source.statements) {
            self.ctx.options.module = ModuleKind::CommonJS;
        }

        // Detect export assignment (export =) to suppress other exports
        if self.has_export_assignment(&source.statements) {
            self.ctx.module_state.has_export_assignment = true;
        }

        // Extract and filter comments (strip compiler directives)
        let all_comments = if !self.ctx.options.remove_comments {
            if let Some(text) = self.source_text {
                crate::comments::get_comment_ranges(text)
                    .into_iter()
                    .filter(|c| {
                        // Filter out triple-slash directives (/// <reference ..., /// <amd ...)
                        // TypeScript strips these from JS output
                        let content = c.get_text(text);
                        !content.starts_with("/// <reference") && !content.starts_with("/// <amd")
                    })
                    .collect()
            } else {
                Vec::new()
            }
        } else {
            Vec::new()
        };

        let mut comment_idx = 0;

        // CommonJS: Emit "use strict" FIRST (before comments and helpers)
        if self.ctx.is_commonjs() {
            self.write("\"use strict\";");
            self.write_line();
        }

        // Emit header comments AFTER "use strict" but BEFORE helpers
        let first_stmt_pos = source.statements.nodes.first()
            .and_then(|&idx| self.arena.get(idx))
            .map(|n| n.pos)
            .unwrap_or(node.end);

        if let Some(text) = self.source_text {
            while comment_idx < all_comments.len() {
                let comment = &all_comments[comment_idx];
                if comment.end <= first_stmt_pos {
                    let comment_text = comment.get_text(text);
                    self.write(comment_text);
                    if comment.has_trailing_new_line {
                        self.write_line();
                    }
                    comment_idx += 1;
                } else {
                    break;
                }
            }
        }

        // Emit runtime helpers (must come BEFORE __esModule marker)
        // Order: "use strict" → helpers → __esModule → exports init
        let mut helpers = crate::transforms::helpers::HelpersNeeded::default();

        // Detect CommonJS import/export helpers
        if self.ctx.is_commonjs() {
            self.detect_commonjs_helpers(&source.statements, &mut helpers);
        }

        // Detect ES5 class helpers
        if self.ctx.target_es5 && self.needs_extends_helper(&source.statements) {
            helpers.extends = true;
        }

        if self.ctx.target_es5 && self.needs_values_helper() {
            helpers.values = true;
        }
        if self.ctx.target_es5 && self.needs_rest_helper() {
            helpers.rest = true;
        }

        // Emit all needed helpers
        let helpers_code = crate::transforms::helpers::emit_helpers(&helpers);
        if !helpers_code.is_empty() {
            self.write(&helpers_code);
            // emit_helpers() already adds newlines, no need to add more
        }

        // CommonJS: Emit __esModule and exports initialization (AFTER helpers)
        if self.ctx.is_commonjs() {
            use crate::transforms::module_commonjs;

            // Emit __esModule if this is an ES module
            if self.should_emit_es_module_marker(&source.statements) {
                self.write("Object.defineProperty(exports, \"__esModule\", { value: true });");
                self.write_line();
            }

            // Collect and emit exports initialization
            let export_names = module_commonjs::collect_export_names(self.arena, &source.statements.nodes);
            if !export_names.is_empty() {
                for (i, name) in export_names.iter().enumerate() {
                    if i > 0 {
                        self.write(" = ");
                    }
                    self.write("exports.");
                    self.write(name);
                }
                self.write(" = void 0;");
                self.write_line();
            }
        }

        // Emit statements with their comments
        for &stmt_idx in &source.statements.nodes {
            if let Some(stmt_node) = self.arena.get(stmt_idx) {
                // Emit any comments that appear before this statement
                if let Some(text) = self.source_text {
                    while comment_idx < all_comments.len() {
                        let comment = &all_comments[comment_idx];
                        if comment.end <= stmt_node.pos {
                            // This comment is before the statement, emit it
                            let comment_text = comment.get_text(text);
                            self.write(comment_text);
                            // Only add newline if the comment has a trailing newline
                            if comment.has_trailing_new_line {
                                self.write_line();
                            }
                            comment_idx += 1;
                        } else {
                            // This comment is after the statement start, stop
                            break;
                        }
                    }
                }
            }

            let before_len = self.writer.len();
            self.emit(stmt_idx);
            // Only add newline if something was actually emitted
            if self.writer.len() > before_len {
                self.write_line();
            }
        }

        // Emit remaining trailing comments at the end of file
        if let Some(text) = self.source_text {
            while comment_idx < all_comments.len() {
                let comment = &all_comments[comment_idx];
                let comment_text = comment.get_text(text);
                self.write(comment_text);
                if comment.has_trailing_new_line {
                    self.write_line();
                }
                comment_idx += 1;
            }
        }
    }

    /// Check if the file contains an export assignment (export =)
    fn has_export_assignment(&self, statements: &NodeList) -> bool {
        for &stmt_idx in &statements.nodes {
            if let Some(node) = self.arena.get(stmt_idx) {
                if node.kind == syntax_kind_ext::EXPORT_ASSIGNMENT {
                    return true;
                }
            }
        }
        false
    }

    /// Check if a file is a module (has import/export statements)
    fn file_is_module(&self, statements: &NodeList) -> bool {
        for &stmt_idx in &statements.nodes {
            if let Some(node) = self.arena.get(stmt_idx) {
                match node.kind {
                    k if k == syntax_kind_ext::IMPORT_DECLARATION => return true,
                    k if k == syntax_kind_ext::EXPORT_DECLARATION => return true,
                    k if k == syntax_kind_ext::EXPORT_ASSIGNMENT => return true,
                    // Check for export modifier on declarations
                    k if k == syntax_kind_ext::VARIABLE_STATEMENT => {
                        if let Some(var_stmt) = self.arena.get_variable(node) {
                            if self.has_export_modifier(&var_stmt.modifiers) {
                                return true;
                            }
                        }
                    }
                    k if k == syntax_kind_ext::FUNCTION_DECLARATION => {
                        if let Some(func) = self.arena.get_function(node) {
                            if self.has_export_modifier(&func.modifiers) {
                                return true;
                            }
                        }
                    }
                    k if k == syntax_kind_ext::CLASS_DECLARATION => {
                        if let Some(class) = self.arena.get_class(node) {
                            if self.has_export_modifier(&class.modifiers) {
                                return true;
                            }
                        }
                    }
                    k if k == syntax_kind_ext::ENUM_DECLARATION => {
                        if let Some(enum_decl) = self.arena.get_enum(node) {
                            if self.has_export_modifier(&enum_decl.modifiers) {
                                return true;
                            }
                        }
                    }
                    k if k == syntax_kind_ext::INTERFACE_DECLARATION => {
                        if let Some(iface) = self.arena.get_interface(node) {
                            if self.has_export_modifier(&iface.modifiers) {
                                return true;
                            }
                        }
                    }
                    k if k == syntax_kind_ext::TYPE_ALIAS_DECLARATION => {
                        if let Some(type_alias) = self.arena.get_type_alias(node) {
                            if self.has_export_modifier(&type_alias.modifiers) {
                                return true;
                            }
                        }
                    }
                    k if k == syntax_kind_ext::MODULE_DECLARATION => {
                        if let Some(module) = self.arena.get_module(node) {
                            if self.has_export_modifier(&module.modifiers) {
                                return true;
                            }
                        }
                    }
                    _ => {}
                }
            }
        }
        false
    }

    /// Check if we should emit the __esModule marker.
    /// Returns true if the file contains any ES6 module syntax (import/export),
    /// excluding `export =` which is legacy CommonJS.
    fn should_emit_es_module_marker(&self, statements: &NodeList) -> bool {
        // First check: if file has export =, don't emit __esModule at all
        for &stmt_idx in &statements.nodes {
            if let Some(node) = self.arena.get(stmt_idx) {
                if node.kind == syntax_kind_ext::EXPORT_ASSIGNMENT {
                    return false;
                }
            }
        }

        // Second check: look for ES6 module syntax
        for &stmt_idx in &statements.nodes {
            if let Some(node) = self.arena.get(stmt_idx) {
                match node.kind {
                    k if k == syntax_kind_ext::IMPORT_DECLARATION => return true,
                    k if k == syntax_kind_ext::EXPORT_DECLARATION => return true,
                    // Note: EXPORT_ASSIGNMENT (export =) is excluded - it's CommonJS style
                    // Check for export modifier on declarations
                    k if k == syntax_kind_ext::VARIABLE_STATEMENT => {
                        if let Some(var_stmt) = self.arena.get_variable(node) {
                            if self.has_export_modifier(&var_stmt.modifiers) {
                                return true;
                            }
                        }
                    }
                    k if k == syntax_kind_ext::FUNCTION_DECLARATION => {
                        if let Some(func) = self.arena.get_function(node) {
                            if self.has_export_modifier(&func.modifiers) {
                                return true;
                            }
                        }
                    }
                    k if k == syntax_kind_ext::CLASS_DECLARATION => {
                        if let Some(class) = self.arena.get_class(node) {
                            if self.has_export_modifier(&class.modifiers) {
                                return true;
                            }
                        }
                    }
                    k if k == syntax_kind_ext::ENUM_DECLARATION => {
                        if let Some(enum_decl) = self.arena.get_enum(node) {
                            if self.has_export_modifier(&enum_decl.modifiers) {
                                return true;
                            }
                        }
                    }
                    k if k == syntax_kind_ext::MODULE_DECLARATION => {
                        if let Some(module) = self.arena.get_module(node) {
                            if self.has_export_modifier(&module.modifiers) {
                                return true;
                            }
                        }
                    }
                    k if k == syntax_kind_ext::INTERFACE_DECLARATION => {
                        if let Some(iface) = self.arena.get_interface(node) {
                            if self.has_export_modifier(&iface.modifiers) {
                                return true;
                            }
                        }
                    }
                    k if k == syntax_kind_ext::TYPE_ALIAS_DECLARATION => {
                        if let Some(type_alias) = self.arena.get_type_alias(node) {
                            if self.has_export_modifier(&type_alias.modifiers) {
                                return true;
                            }
                        }
                    }
                    _ => {}
                }
            }
        }
        false
    }

    /// Emit CommonJS module preamble
    fn emit_commonjs_preamble(&mut self, statements: &NodeList) {
        use crate::transforms::module_commonjs;

        // "use strict";
        self.write("\"use strict\";");
        self.write_line();

        // Emit __esModule if this is an ES module (has imports or ES exports)
        // Note: 'export =' is CommonJS style and doesn't get __esModule
        if self.should_emit_es_module_marker(statements) {
            self.write("Object.defineProperty(exports, \"__esModule\", { value: true });");
            self.write_line();
        }

        // Collect and emit exports initialization
        // TypeScript emits: exports.C = void 0; (NOT Object.defineProperty)
        let export_names = module_commonjs::collect_export_names(self.arena, &statements.nodes);
        if !export_names.is_empty() {
            // exports.a = exports.b = void 0;
            for (i, name) in export_names.iter().enumerate() {
                if i > 0 {
                    self.write(" = ");
                }
                self.write("exports.");
                self.write(name);
            }
            self.write(" = void 0;");
            self.write_line();
        }
    }

    /// Detect which CommonJS import/export helpers are needed for the file
    fn detect_commonjs_helpers(&self, statements: &NodeList, helpers: &mut crate::transforms::helpers::HelpersNeeded) {
        use crate::parser::syntax_kind_ext;

        for &stmt_idx in &statements.nodes {
            let Some(node) = self.arena.get(stmt_idx) else { continue };

            match node.kind {
                k if k == syntax_kind_ext::IMPORT_DECLARATION => {
                    if let Some(import) = self.arena.get_import_decl(node) {
                        // Check for: import * as ns from "mod"
                        if let Some(clause_node) = self.arena.get(import.import_clause) {
                            if let Some(clause) = self.arena.get_import_clause(clause_node) {
                                if let Some(bindings_node) = self.arena.get(clause.named_bindings) {
                                    // NAMESPACE_IMPORT = 275
                                    if bindings_node.kind == syntax_kind_ext::NAMESPACE_IMPORT {
                                        helpers.import_star = true;
                                        helpers.create_binding = true; // __importStar depends on __createBinding
                                    }
                                }
                            }
                        }
                    }
                }
                k if k == syntax_kind_ext::EXPORT_DECLARATION => {
                    if let Some(export) = self.arena.get_export_decl(node) {
                        // Check for: export * from "mod" (module_specifier present, no export_clause)
                        if !export.module_specifier.is_none() && export.export_clause.is_none() {
                            helpers.export_star = true;
                            helpers.create_binding = true; // __exportStar depends on __createBinding
                        }
                    }
                }
                _ => {}
            }
        }
    }

    /// Check if any class in the statements (recursively) extends another class
    fn needs_extends_helper(&self, statements: &NodeList) -> bool {
        for &stmt_idx in &statements.nodes {
            if self.statement_needs_extends(stmt_idx) {
                return true;
            }
        }
        false
    }

    fn needs_values_helper(&self) -> bool {
        self.arena.nodes.iter().any(|node| {
            if node.kind != syntax_kind_ext::FOR_OF_STATEMENT {
                return false;
            }

            if let Some(for_in_of) = self.arena.get_for_in_of(node) {
                return !for_in_of.await_modifier;
            }

            false
        })
    }

    fn needs_rest_helper(&self) -> bool {
        self.arena.nodes.iter().any(|node| {
            if node.kind != syntax_kind_ext::OBJECT_BINDING_PATTERN {
                return false;
            }

            let Some(pattern) = self.arena.get_binding_pattern(node) else {
                return false;
            };

            for &elem_idx in &pattern.elements.nodes {
                let Some(elem_node) = self.arena.get(elem_idx) else { continue };
                let Some(elem) = self.arena.get_binding_element(elem_node) else { continue };
                if elem.dot_dot_dot_token {
                    return true;
                }
            }

            false
        })
    }

    /// Check if a statement contains a class that extends another (recursive)
    fn statement_needs_extends(&self, stmt_idx: NodeIndex) -> bool {
        let Some(node) = self.arena.get(stmt_idx) else {
            return false;
        };

        match node.kind {
            // Class declaration
            k if k == syntax_kind_ext::CLASS_DECLARATION => {
                if let Some(class_data) = self.arena.get_class(node) {
                    if self.class_has_extends(&class_data.heritage_clauses) {
                        return true;
                    }
                }
                false
            }
            // Expression statement - might contain IIFE with class
            k if k == syntax_kind_ext::EXPRESSION_STATEMENT => {
                if let Some(expr_stmt) = self.arena.get_expression_statement(node) {
                    self.expression_needs_extends(expr_stmt.expression)
                } else {
                    false
                }
            }
            // Block - recurse into statements
            k if k == syntax_kind_ext::BLOCK => {
                if let Some(block) = self.arena.get_block(node) {
                    self.needs_extends_helper(&block.statements)
                } else {
                    false
                }
            }
            _ => false,
        }
    }

    /// Check if an expression contains a class that extends another (recursive)
    fn expression_needs_extends(&self, expr_idx: NodeIndex) -> bool {
        let Some(node) = self.arena.get(expr_idx) else {
            return false;
        };

        match node.kind {
            // Call expression - check arguments and the called function
            k if k == syntax_kind_ext::CALL_EXPRESSION => {
                if let Some(call) = self.arena.get_call_expr(node) {
                    // Check the function being called
                    if self.expression_needs_extends(call.expression) {
                        return true;
                    }
                    // Check arguments
                    if let Some(ref args) = call.arguments {
                        for &arg_idx in &args.nodes {
                            if self.expression_needs_extends(arg_idx) {
                                return true;
                            }
                        }
                    }
                }
                false
            }
            // Parenthesized expression
            k if k == syntax_kind_ext::PARENTHESIZED_EXPRESSION => {
                if let Some(paren) = self.arena.get_parenthesized(node) {
                    self.expression_needs_extends(paren.expression)
                } else {
                    false
                }
            }
            // Arrow function - check body
            k if k == syntax_kind_ext::ARROW_FUNCTION => {
                if let Some(func) = self.arena.get_function(node) {
                    self.statement_needs_extends(func.body)
                } else {
                    false
                }
            }
            // Function expression - check body
            k if k == syntax_kind_ext::FUNCTION_EXPRESSION => {
                if let Some(func) = self.arena.get_function(node) {
                    self.statement_needs_extends(func.body)
                } else {
                    false
                }
            }
            // Class expression
            k if k == syntax_kind_ext::CLASS_EXPRESSION => {
                if let Some(class_data) = self.arena.get_class(node) {
                    self.class_has_extends(&class_data.heritage_clauses)
                } else {
                    false
                }
            }
            _ => false,
        }
    }

    /// Check if a class has an extends clause
    fn class_has_extends(&self, heritage_clauses: &Option<NodeList>) -> bool {
        let Some(clauses) = heritage_clauses else {
            return false;
        };
        for &clause_idx in &clauses.nodes {
            let Some(clause_node) = self.arena.get(clause_idx) else {
                continue;
            };
            let Some(heritage_data) = self.arena.get_heritage(clause_node) else {
                continue;
            };
            if heritage_data.token == SyntaxKind::ExtendsKeyword as u16 {
                return true;
            }
        }
        false
    }

    /// Emit the __extends helper function
    fn emit_extends_helper(&mut self) {
        // TypeScript's ES5 __extends helper
        self.write("var __extends = (this && this.__extends) || (function () {");
        self.write_line();
        self.increase_indent();

        self.write("var extendStatics = function (d, b) {");
        self.write_line();
        self.increase_indent();

        self.write("extendStatics = Object.setPrototypeOf ||");
        self.write_line();
        self.write("    ({ __proto__: [] } instanceof Array && function (d, b) { d.__proto__ = b; }) ||");
        self.write_line();
        self.write("    function (d, b) { for (var p in b) if (Object.prototype.hasOwnProperty.call(b, p)) d[p] = b[p]; };");
        self.write_line();
        self.write("return extendStatics(d, b);");
        self.write_line();

        self.decrease_indent();
        self.write("};");
        self.write_line();

        self.write("return function (d, b) {");
        self.write_line();
        self.increase_indent();

        self.write("if (typeof b !== \"function\" && b !== null)");
        self.write_line();
        self.write("    throw new TypeError(\"Class extends value \" + String(b) + \" is not a constructor or null\");");
        self.write_line();
        self.write("extendStatics(d, b);");
        self.write_line();
        self.write("function __() { this.constructor = d; }");
        self.write_line();
        self.write("d.prototype = b === null ? Object.create(b) : (__.prototype = b.prototype, new __());");
        self.write_line();

        self.decrease_indent();
        self.write("};");
        self.write_line();

        self.decrease_indent();
        self.write("})();");
        self.write_line();
    }

    // =========================================================================
    // Binding Patterns (Destructuring)
    // =========================================================================

    /// Emit an object binding pattern: { x, y }
    fn emit_object_binding_pattern(&mut self, node: &ThinNode) {
        let Some(pattern) = self.arena.get_binding_pattern(node) else {
            return;
        };

        self.write("{ ");
        self.emit_comma_separated(&pattern.elements.nodes);
        self.write(" }");
    }

    /// Emit an array binding pattern: [x, y]
    fn emit_array_binding_pattern(&mut self, node: &ThinNode) {
        let Some(pattern) = self.arena.get_binding_pattern(node) else {
            return;
        };

        self.write("[");
        self.emit_comma_separated(&pattern.elements.nodes);
        self.write("]");
    }

    /// Emit a binding element: x or x = default or propertyName: x
    fn emit_binding_element(&mut self, node: &ThinNode) {
        let Some(elem) = self.arena.get_binding_element(node) else {
            return;
        };

        // Rest element: ...x
        if elem.dot_dot_dot_token {
            self.write("...");
        }

        // propertyName: name  or just name
        if !elem.property_name.is_none() {
            self.emit(elem.property_name);
            self.write(": ");
        }

        self.emit(elem.name);

        // Default value: = expr
        if !elem.initializer.is_none() {
            self.write(" = ");
            self.emit(elem.initializer);
        }
    }

    /// Get the next temporary variable name (_a, _b, _c, etc.)
    fn get_temp_var_name(&mut self) -> String {
        let name = format!("_{}", (b'a' + (self.ctx.destructuring_state.temp_var_counter % 26) as u8) as char);
        self.ctx.destructuring_state.temp_var_counter += 1;
        name
    }

    /// Check if a node is a binding pattern
    fn is_binding_pattern(&self, idx: NodeIndex) -> bool {
        let Some(node) = self.arena.get(idx) else {
            return false;
        };
        node.kind == syntax_kind_ext::OBJECT_BINDING_PATTERN
            || node.kind == syntax_kind_ext::ARRAY_BINDING_PATTERN
    }
}

// =============================================================================
// Operator Text Helper
// =============================================================================

fn get_operator_text(op: u16) -> &'static str {
    match op {
        k if k == SyntaxKind::PlusToken as u16 => "+",
        k if k == SyntaxKind::MinusToken as u16 => "-",
        k if k == SyntaxKind::AsteriskToken as u16 => "*",
        k if k == SyntaxKind::SlashToken as u16 => "/",
        k if k == SyntaxKind::PercentToken as u16 => "%",
        k if k == SyntaxKind::AsteriskAsteriskToken as u16 => "**",
        k if k == SyntaxKind::PlusPlusToken as u16 => "++",
        k if k == SyntaxKind::MinusMinusToken as u16 => "--",
        k if k == SyntaxKind::LessThanToken as u16 => "<",
        k if k == SyntaxKind::GreaterThanToken as u16 => ">",
        k if k == SyntaxKind::LessThanEqualsToken as u16 => "<=",
        k if k == SyntaxKind::GreaterThanEqualsToken as u16 => ">=",
        k if k == SyntaxKind::EqualsEqualsToken as u16 => "==",
        k if k == SyntaxKind::ExclamationEqualsToken as u16 => "!=",
        k if k == SyntaxKind::EqualsEqualsEqualsToken as u16 => "===",
        k if k == SyntaxKind::ExclamationEqualsEqualsToken as u16 => "!==",
        k if k == SyntaxKind::EqualsToken as u16 => "=",
        k if k == SyntaxKind::PlusEqualsToken as u16 => "+=",
        k if k == SyntaxKind::MinusEqualsToken as u16 => "-=",
        k if k == SyntaxKind::AsteriskEqualsToken as u16 => "*=",
        k if k == SyntaxKind::SlashEqualsToken as u16 => "/=",
        k if k == SyntaxKind::PercentEqualsToken as u16 => "%=",
        k if k == SyntaxKind::AmpersandToken as u16 => "&",
        k if k == SyntaxKind::BarToken as u16 => "|",
        k if k == SyntaxKind::CaretToken as u16 => "^",
        k if k == SyntaxKind::TildeToken as u16 => "~",
        k if k == SyntaxKind::AmpersandAmpersandToken as u16 => "&&",
        k if k == SyntaxKind::BarBarToken as u16 => "||",
        k if k == SyntaxKind::ExclamationToken as u16 => "!",
        k if k == SyntaxKind::QuestionQuestionToken as u16 => "??",
        k if k == SyntaxKind::LessThanLessThanToken as u16 => "<<",
        k if k == SyntaxKind::GreaterThanGreaterThanToken as u16 => ">>",
        k if k == SyntaxKind::GreaterThanGreaterThanGreaterThanToken as u16 => ">>>",
        _ => "",
    }
}


#[cfg(test)]
mod comment_tests {
    use super::*;

    #[test]
    fn test_trailing_comments_parsing() {
        let text = "constructor(public p3:any) {} // OK";
        //                                       ^
        //                                       position 29 (after the closing brace)
        let comments = get_trailing_comment_ranges(text, 29);
        assert_eq!(comments.len(), 1);
        assert_eq!(&text[comments[0].pos as usize..comments[0].end as usize], "// OK");
    }

    #[test]
    fn test_trailing_comments_with_space() {
        let text = "} // OK\n";
        let comments = get_trailing_comment_ranges(text, 1); // after }
        assert_eq!(comments.len(), 1);
        assert_eq!(&text[comments[0].pos as usize..comments[0].end as usize], "// OK");
    }

}
