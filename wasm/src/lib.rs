use wasm_bindgen::prelude::*;

// String interning for identifier deduplication (Performance optimization)
pub mod interner;
pub use interner::{Atom, Interner};
#[cfg(test)]
mod interner_tests;

// Character code constants
pub mod char_codes;

// Scanner types and token definitions
pub mod scanner;
pub use scanner::*;
#[cfg(test)]
mod scanner_tests;

// Scanner implementation
pub mod scanner_impl;
pub use scanner_impl::*;
#[cfg(test)]
mod scanner_impl_tests;

// Parser AST types (Phase 3)
pub mod parser;

// ThinParser - Cache-optimized parser using ThinNodeArena (Phase 0.1)
pub mod thin_parser;
#[cfg(test)]
mod thin_parser_tests;

// Binder types and implementation (Phase 4)
pub mod binder;

// ThinBinder - Binder using ThinNodeArena (Phase 0.1)
pub mod thin_binder;
#[cfg(test)]
mod thin_binder_tests;

// Checker types and implementation (Phase 5)
pub mod checker;

// ThinChecker - Type checker using ThinNodeArena (Phase 0.1)
pub mod thin_checker;
#[cfg(test)]
mod thin_checker_tests;

// ThinEmitter - Emitter using ThinNodeArena (Phase 0.1)
pub mod thin_emitter;
#[cfg(test)]
mod thin_emitter_tests;


// Parallel processing with Rayon (Phase 0.4)
pub mod parallel;

// Comment preservation (Phase 6.3)
pub mod comments;
#[cfg(test)]
mod comments_tests;

// Source Map generation (Phase 6.2)
pub mod source_map;
#[cfg(test)]
mod source_map_tests;

// Declaration file emitter (Phase 6.4)
pub mod declaration_emitter;
#[cfg(test)]
mod declaration_emitter_tests;

// JavaScript transforms (Phase 6.5+)
pub mod transforms;

// Query-based Structural Solver (Phase 7.5)
pub mod solver;

// =============================================================================
// Scanner Factory Function
// =============================================================================

/// Create a new scanner for the given source text.
/// This is the wasm-bindgen entry point for creating scanners from JavaScript.
#[wasm_bindgen(js_name = createScanner)]
pub fn create_scanner(text: String, skip_trivia: bool) -> ScannerState {
    ScannerState::new(text, skip_trivia)
}

// =============================================================================
// Binder Factory Function
// =============================================================================

/// Create a new binder for binding AST nodes to symbols.
/// This is the wasm-bindgen entry point for creating binders from JavaScript.
#[wasm_bindgen(js_name = createBinder)]
pub fn create_binder() -> binder::BinderState {
    binder::BinderState::new()
}

// =============================================================================
// ThinParser WASM Interface (High-Performance Parser)
// =============================================================================

use crate::thin_parser::ThinParserState;
use crate::thin_binder::ThinBinderState;
use crate::thin_checker::ThinCheckerState;
use crate::thin_emitter::ThinPrinter;
use crate::solver::TypeInterner;

/// High-performance parser using ThinNode architecture (16 bytes/node).
/// This is the optimized path for Phase 8 test suite evaluation.
#[wasm_bindgen]
pub struct ThinParser {
    parser: ThinParserState,
    source_file_idx: Option<parser::NodeIndex>,
    binder: Option<ThinBinderState>,
    /// Local type interner for single-file checking.
    /// For multi-file compilation, use MergedProgram.type_interner instead.
    type_interner: TypeInterner,
}

#[wasm_bindgen]
impl ThinParser {
    /// Create a new ThinParser for the given source file.
    #[wasm_bindgen(constructor)]
    pub fn new(file_name: String, source_text: String) -> ThinParser {
        ThinParser {
            parser: ThinParserState::new(file_name, source_text),
            source_file_idx: None,
            binder: None,
            type_interner: TypeInterner::new(),
        }
    }

    /// Parse the source file and return the root node index.
    #[wasm_bindgen(js_name = parseSourceFile)]
    pub fn parse_source_file(&mut self) -> u32 {
        let idx = self.parser.parse_source_file();
        self.source_file_idx = Some(idx);
        idx.0
    }

    /// Get the number of nodes in the AST.
    #[wasm_bindgen(js_name = getNodeCount)]
    pub fn get_node_count(&self) -> usize {
        self.parser.get_node_count()
    }

    /// Get parse diagnostics as JSON.
    #[wasm_bindgen(js_name = getDiagnosticsJson)]
    pub fn get_diagnostics_json(&self) -> String {
        let diags: Vec<_> = self.parser.get_diagnostics().iter().map(|d| {
            serde_json::json!({
                "message": d.message,
                "start": d.start,
                "length": d.length,
                "code": d.code,
            })
        }).collect();
        serde_json::to_string(&diags).unwrap_or_else(|_| "[]".to_string())
    }

    /// Bind the source file and return symbol count.
    #[wasm_bindgen(js_name = bindSourceFile)]
    pub fn bind_source_file(&mut self) -> String {
        if let Some(root_idx) = self.source_file_idx {
            let mut binder = ThinBinderState::new();
            binder.bind_source_file(self.parser.get_arena(), root_idx);

            // Collect symbol names for the result
            let symbols: std::collections::HashMap<String, u32> = binder.file_locals
                .iter()
                .map(|(name, id)| (name.clone(), id.0))
                .collect();

            let result = serde_json::json!({
                "symbols": symbols,
                "symbolCount": binder.symbols.len(),
            });

            self.binder = Some(binder);
            serde_json::to_string(&result).unwrap_or_else(|_| "{}".to_string())
        } else {
            r#"{"error": "Source file not parsed"}"#.to_string()
        }
    }

    /// Type check the source file and return diagnostics.
    #[wasm_bindgen(js_name = checkSourceFile)]
    pub fn check_source_file(&mut self) -> String {
        if self.binder.is_none() {
            // Auto-bind if not done yet
            if self.source_file_idx.is_some() {
                self.bind_source_file();
            }
        }

        if let (Some(root_idx), Some(binder)) = (self.source_file_idx, &self.binder) {
            let file_name = self.parser.get_file_name().to_string();
            let mut checker = ThinCheckerState::new(
                self.parser.get_arena(),
                binder,
                &self.type_interner,
                file_name,
            );

            // Full source file type checking - traverse all statements
            checker.check_source_file(root_idx);

            let result = serde_json::json!({
                "typeCount": self.type_interner.len(),
                "diagnostics": checker.ctx.diagnostics.iter().map(|d| {
                    serde_json::json!({
                        "message_text": d.message_text.clone(),
                        "code": d.code,
                        "start": d.start,
                        "length": d.length,
                        "category": format!("{:?}", d.category),
                    })
                }).collect::<Vec<_>>(),
            });

            serde_json::to_string(&result).unwrap_or_else(|_| "{}".to_string())
        } else {
            r#"{"error": "Source file not parsed or bound"}"#.to_string()
        }
    }

    /// Get the type of a node as a string.
    #[wasm_bindgen(js_name = getTypeOfNode)]
    pub fn get_type_of_node(&mut self, node_idx: u32) -> String {
        if let (Some(_), Some(binder)) = (self.source_file_idx, &self.binder) {
            let file_name = self.parser.get_file_name().to_string();
            let mut checker = ThinCheckerState::new(
                self.parser.get_arena(),
                binder,
                &self.type_interner,
                file_name,
            );

            let type_id = checker.get_type_of_node(parser::NodeIndex(node_idx));
            // Use basic type display since ThinCheckerState doesn't have type_to_string
            format!("TypeId({})", type_id.0)
        } else {
            "unknown".to_string()
        }
    }

    /// Emit the source file as JavaScript (ES5 target to match TypeScript baselines).
    #[wasm_bindgen(js_name = emit)]
    pub fn emit(&self) -> String {
        if let Some(root_idx) = self.source_file_idx {
            let mut printer = ThinPrinter::new(self.parser.get_arena());
            printer.set_target_es5(true); // Match TypeScript baselines
            printer.set_source_text(self.parser.get_source_text());
            printer.emit(root_idx);
            printer.get_output().to_string()
        } else {
            String::new()
        }
    }
    
    /// Emit the source file as JavaScript (ES6+ modern output).
    #[wasm_bindgen(js_name = emitModern)]
    pub fn emit_modern(&self) -> String {
        if let Some(root_idx) = self.source_file_idx {
            let mut printer = ThinPrinter::new(self.parser.get_arena());
            printer.emit(root_idx);
            printer.get_output().to_string()
        } else {
            String::new()
        }
    }

    /// Get the AST as JSON (for debugging).
    #[wasm_bindgen(js_name = getAstJson)]
    pub fn get_ast_json(&self) -> String {
        if let Some(root_idx) = self.source_file_idx {
            let arena = self.parser.get_arena();
            format!("{{\"nodeCount\": {}, \"rootIdx\": {}}}", arena.len(), root_idx.0)
        } else {
            "{}".to_string()
        }
    }
}

/// Create a new ThinParser for the given source text.
/// This is the recommended parser for production use.
#[wasm_bindgen(js_name = createThinParser)]
pub fn create_thin_parser(file_name: String, source_text: String) -> ThinParser {
    ThinParser::new(file_name, source_text)
}

// =============================================================================
// Comparison enum - matches TypeScript's Comparison const enum
// =============================================================================

/// Comparison result for ordering operations.
/// Matches TypeScript's `Comparison` const enum in src/compiler/corePublic.ts
#[wasm_bindgen]
#[repr(i32)]
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Comparison {
    LessThan = -1,
    EqualTo = 0,
    GreaterThan = 1,
}

// =============================================================================
// POC function (keep for verification)
// =============================================================================

#[wasm_bindgen]
pub fn add(a: i32, b: i32) -> i32 {
    a + b
}

// =============================================================================
// String Comparison Utilities (Phase 1.1)
// =============================================================================

/// Compare two strings using a case-sensitive ordinal comparison.
///
/// Ordinal comparisons are based on the difference between the unicode code points
/// of both strings. Characters with multiple unicode representations are considered
/// unequal. Ordinal comparisons provide predictable ordering, but place "a" after "B".
#[wasm_bindgen(js_name = compareStringsCaseSensitive)]
pub fn compare_strings_case_sensitive(a: Option<String>, b: Option<String>) -> Comparison {
    match (a, b) {
        (None, None) => Comparison::EqualTo,
        (None, Some(_)) => Comparison::LessThan,
        (Some(_), None) => Comparison::GreaterThan,
        (Some(a), Some(b)) => {
            if a == b {
                Comparison::EqualTo
            } else if a < b {
                Comparison::LessThan
            } else {
                Comparison::GreaterThan
            }
        }
    }
}

/// Compare two strings using a case-insensitive ordinal comparison.
///
/// Case-insensitive comparisons compare both strings one code-point at a time using
/// the integer value of each code-point after applying `to_uppercase` to each string.
/// We always map both strings to their upper-case form as some unicode characters do
/// not properly round-trip to lowercase (such as `ẞ` German sharp capital s).
#[wasm_bindgen(js_name = compareStringsCaseInsensitive)]
pub fn compare_strings_case_insensitive(a: Option<String>, b: Option<String>) -> Comparison {
    match (a, b) {
        (None, None) => Comparison::EqualTo,
        (None, Some(_)) => Comparison::LessThan,
        (Some(_), None) => Comparison::GreaterThan,
        (Some(a), Some(b)) => {
            if a == b {
                return Comparison::EqualTo;
            }
            // Use iterator-based comparison to avoid allocating new strings
            compare_strings_case_insensitive_iter(&a, &b)
        }
    }
}

/// Iterator-based case-insensitive comparison (no allocation).
/// Maps characters to uppercase on-the-fly without creating new strings.
#[inline]
fn compare_strings_case_insensitive_iter(a: &str, b: &str) -> Comparison {
    use std::cmp::Ordering;

    let mut a_chars = a.chars().flat_map(char::to_uppercase);
    let mut b_chars = b.chars().flat_map(char::to_uppercase);

    loop {
        match (a_chars.next(), b_chars.next()) {
            (None, None) => return Comparison::EqualTo,
            (None, Some(_)) => return Comparison::LessThan,
            (Some(_), None) => return Comparison::GreaterThan,
            (Some(a_char), Some(b_char)) => {
                match a_char.cmp(&b_char) {
                    Ordering::Less => return Comparison::LessThan,
                    Ordering::Greater => return Comparison::GreaterThan,
                    Ordering::Equal => continue,
                }
            }
        }
    }
}

/// Compare two strings using a case-insensitive ordinal comparison (eslint-compatible).
///
/// This uses `to_lowercase` instead of `to_uppercase` to match eslint's `sort-imports`
/// rule behavior. The difference affects the relative order of letters and ASCII
/// characters 91-96, of which `_` is a valid identifier character.
#[wasm_bindgen(js_name = compareStringsCaseInsensitiveEslintCompatible)]
pub fn compare_strings_case_insensitive_eslint_compatible(
    a: Option<String>,
    b: Option<String>,
) -> Comparison {
    match (a, b) {
        (None, None) => Comparison::EqualTo,
        (None, Some(_)) => Comparison::LessThan,
        (Some(_), None) => Comparison::GreaterThan,
        (Some(a), Some(b)) => {
            if a == b {
                return Comparison::EqualTo;
            }
            // Use iterator-based comparison to avoid allocating new strings
            compare_strings_case_insensitive_lower_iter(&a, &b)
        }
    }
}

/// Iterator-based case-insensitive comparison using lowercase (no allocation).
/// Used for eslint compatibility.
#[inline]
fn compare_strings_case_insensitive_lower_iter(a: &str, b: &str) -> Comparison {
    use std::cmp::Ordering;

    let mut a_chars = a.chars().flat_map(char::to_lowercase);
    let mut b_chars = b.chars().flat_map(char::to_lowercase);

    loop {
        match (a_chars.next(), b_chars.next()) {
            (None, None) => return Comparison::EqualTo,
            (None, Some(_)) => return Comparison::LessThan,
            (Some(_), None) => return Comparison::GreaterThan,
            (Some(a_char), Some(b_char)) => {
                match a_char.cmp(&b_char) {
                    Ordering::Less => return Comparison::LessThan,
                    Ordering::Greater => return Comparison::GreaterThan,
                    Ordering::Equal => continue,
                }
            }
        }
    }
}

/// Check if two strings are equal (case-sensitive).
#[wasm_bindgen(js_name = equateStringsCaseSensitive)]
pub fn equate_strings_case_sensitive(a: &str, b: &str) -> bool {
    a == b
}

/// Check if two strings are equal (case-insensitive).
/// Uses iterator-based comparison to avoid allocating new strings.
#[wasm_bindgen(js_name = equateStringsCaseInsensitive)]
pub fn equate_strings_case_insensitive(a: &str, b: &str) -> bool {
    if a.len() != b.len() {
        // Quick length check - but note that uppercase/lowercase might change length
        // for some unicode characters, so we need the full comparison
    }
    a.chars()
        .flat_map(char::to_uppercase)
        .eq(b.chars().flat_map(char::to_uppercase))
}

// =============================================================================
// Path Utilities (Phase 1.2)
// =============================================================================

/// Directory separator used internally (forward slash).
pub const DIRECTORY_SEPARATOR: char = '/';

/// Alternative directory separator (backslash, used on Windows).
pub const ALT_DIRECTORY_SEPARATOR: char = '\\';

/// Determines whether a charCode corresponds to `/` or `\`.
#[wasm_bindgen(js_name = isAnyDirectorySeparator)]
pub fn is_any_directory_separator(char_code: u32) -> bool {
    char_code == DIRECTORY_SEPARATOR as u32 || char_code == ALT_DIRECTORY_SEPARATOR as u32
}

/// Normalize path separators, converting `\` into `/`.
#[wasm_bindgen(js_name = normalizeSlashes)]
pub fn normalize_slashes(path: &str) -> String {
    if path.contains('\\') {
        path.replace('\\', "/")
    } else {
        path.to_string()
    }
}

/// Determines whether a path has a trailing separator (`/` or `\\`).
#[wasm_bindgen(js_name = hasTrailingDirectorySeparator)]
pub fn has_trailing_directory_separator(path: &str) -> bool {
    if path.is_empty() {
        return false;
    }
    let last_char = path.chars().last().unwrap();
    last_char == DIRECTORY_SEPARATOR || last_char == ALT_DIRECTORY_SEPARATOR
}

/// Determines whether a path starts with a relative path component (i.e. `.` or `..`).
#[wasm_bindgen(js_name = pathIsRelative)]
pub fn path_is_relative(path: &str) -> bool {
    // Matches /^\.\.?(?:$|[\\/])/
    if path.starts_with("./") || path.starts_with(".\\") || path == "." {
        return true;
    }
    if path.starts_with("../") || path.starts_with("..\\") || path == ".." {
        return true;
    }
    false
}

/// Removes a trailing directory separator from a path, if it has one.
/// Uses char-based operations for UTF-8 safety.
#[wasm_bindgen(js_name = removeTrailingDirectorySeparator)]
pub fn remove_trailing_directory_separator(path: &str) -> String {
    if !has_trailing_directory_separator(path) || path.len() <= 1 {
        return path.to_string();
    }
    // Use strip_suffix for UTF-8 safe character removal
    path.strip_suffix(DIRECTORY_SEPARATOR)
        .or_else(|| path.strip_suffix(ALT_DIRECTORY_SEPARATOR))
        .unwrap_or(path)
        .to_string()
}

/// Ensures a path has a trailing directory separator.
#[wasm_bindgen(js_name = ensureTrailingDirectorySeparator)]
pub fn ensure_trailing_directory_separator(path: &str) -> String {
    if has_trailing_directory_separator(path) {
        path.to_string()
    } else {
        format!("{}/", path)
    }
}

/// Determines whether a path has an extension.
#[wasm_bindgen(js_name = hasExtension)]
pub fn has_extension(file_name: &str) -> bool {
    get_base_file_name(file_name).contains('.')
}

/// Returns the path except for its containing directory name (basename).
/// Uses char-based operations for UTF-8 safety.
#[wasm_bindgen(js_name = getBaseFileName)]
pub fn get_base_file_name(path: &str) -> String {
    let path = normalize_slashes(path);
    // Remove trailing separator using UTF-8 safe operations
    let path = if has_trailing_directory_separator(&path) && path.len() > 1 {
        path.strip_suffix(DIRECTORY_SEPARATOR)
            .or_else(|| path.strip_suffix(ALT_DIRECTORY_SEPARATOR))
            .unwrap_or(&path)
    } else {
        &path
    };
    // Find last separator - safe because '/' is ASCII and rfind returns valid char boundary
    match path.rfind('/') {
        Some(idx) => path[idx + 1..].to_string(),
        None => path.to_string(),
    }
}

/// Check if path ends with a specific extension.
#[wasm_bindgen(js_name = fileExtensionIs)]
pub fn file_extension_is(path: &str, extension: &str) -> bool {
    path.len() > extension.len() && path.ends_with(extension)
}

// =============================================================================
// Character Classification (Phase 1.3 - Scanner Prep)
// =============================================================================

use crate::char_codes::CharacterCodes;

/// Check if character is a line break (LF, CR, LS, PS).
#[wasm_bindgen(js_name = isLineBreak)]
pub fn is_line_break(ch: u32) -> bool {
    ch == CharacterCodes::LINE_FEED
        || ch == CharacterCodes::CARRIAGE_RETURN
        || ch == CharacterCodes::LINE_SEPARATOR
        || ch == CharacterCodes::PARAGRAPH_SEPARATOR
}

/// Check if character is a single-line whitespace (not including line breaks).
#[wasm_bindgen(js_name = isWhiteSpaceSingleLine)]
pub fn is_white_space_single_line(ch: u32) -> bool {
    ch == CharacterCodes::SPACE
        || ch == CharacterCodes::TAB
        || ch == CharacterCodes::VERTICAL_TAB
        || ch == CharacterCodes::FORM_FEED
        || ch == CharacterCodes::NON_BREAKING_SPACE
        || ch == CharacterCodes::NEXT_LINE
        || ch == CharacterCodes::OGHAM
        || (ch >= CharacterCodes::EN_QUAD && ch <= CharacterCodes::ZERO_WIDTH_SPACE)
        || ch == CharacterCodes::NARROW_NO_BREAK_SPACE
        || ch == CharacterCodes::MATHEMATICAL_SPACE
        || ch == CharacterCodes::IDEOGRAPHIC_SPACE
        || ch == CharacterCodes::BYTE_ORDER_MARK
}

/// Check if character is any whitespace (including line breaks).
#[wasm_bindgen(js_name = isWhiteSpaceLike)]
pub fn is_white_space_like(ch: u32) -> bool {
    is_white_space_single_line(ch) || is_line_break(ch)
}

/// Check if character is a decimal digit (0-9).
#[wasm_bindgen(js_name = isDigit)]
pub fn is_digit(ch: u32) -> bool {
    ch >= CharacterCodes::_0 && ch <= CharacterCodes::_9
}

/// Check if character is an octal digit (0-7).
#[wasm_bindgen(js_name = isOctalDigit)]
pub fn is_octal_digit(ch: u32) -> bool {
    ch >= CharacterCodes::_0 && ch <= CharacterCodes::_7
}

/// Check if character is a hexadecimal digit (0-9, A-F, a-f).
#[wasm_bindgen(js_name = isHexDigit)]
pub fn is_hex_digit(ch: u32) -> bool {
    is_digit(ch)
        || (ch >= CharacterCodes::UPPER_A && ch <= CharacterCodes::UPPER_F)
        || (ch >= CharacterCodes::LOWER_A && ch <= CharacterCodes::LOWER_F)
}

/// Check if character is an ASCII letter (A-Z, a-z).
#[wasm_bindgen(js_name = isASCIILetter)]
pub fn is_ascii_letter(ch: u32) -> bool {
    (ch >= CharacterCodes::UPPER_A && ch <= CharacterCodes::UPPER_Z)
        || (ch >= CharacterCodes::LOWER_A && ch <= CharacterCodes::LOWER_Z)
}

/// Check if character is a word character (A-Z, a-z, 0-9, _).
#[wasm_bindgen(js_name = isWordCharacter)]
pub fn is_word_character(ch: u32) -> bool {
    is_ascii_letter(ch) || is_digit(ch) || ch == CharacterCodes::UNDERSCORE
}

// =============================================================================
// Unit Tests
// =============================================================================

#[cfg(test)]
mod lib_tests;
