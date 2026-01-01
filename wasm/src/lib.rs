use wasm_bindgen::prelude::*;

// Scanner types and token definitions
pub mod scanner;
pub use scanner::*;

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
            let a_upper = a.to_uppercase();
            let b_upper = b.to_uppercase();
            if a_upper < b_upper {
                Comparison::LessThan
            } else if a_upper > b_upper {
                Comparison::GreaterThan
            } else {
                Comparison::EqualTo
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
            let a_lower = a.to_lowercase();
            let b_lower = b.to_lowercase();
            if a_lower < b_lower {
                Comparison::LessThan
            } else if a_lower > b_lower {
                Comparison::GreaterThan
            } else {
                Comparison::EqualTo
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
#[wasm_bindgen(js_name = equateStringsCaseInsensitive)]
pub fn equate_strings_case_insensitive(a: &str, b: &str) -> bool {
    a.to_uppercase() == b.to_uppercase()
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
#[wasm_bindgen(js_name = removeTrailingDirectorySeparator)]
pub fn remove_trailing_directory_separator(path: &str) -> String {
    if has_trailing_directory_separator(path) && path.len() > 1 {
        path[..path.len() - 1].to_string()
    } else {
        path.to_string()
    }
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
#[wasm_bindgen(js_name = getBaseFileName)]
pub fn get_base_file_name(path: &str) -> String {
    let path = normalize_slashes(path);
    // Remove trailing separator
    let path = if has_trailing_directory_separator(&path) && path.len() > 1 {
        &path[..path.len() - 1]
    } else {
        &path
    };
    // Find last separator
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

// Character codes matching TypeScript's CharacterCodes enum
mod char_codes {
    pub const LINE_FEED: u32 = 0x0A;
    pub const CARRIAGE_RETURN: u32 = 0x0D;
    pub const LINE_SEPARATOR: u32 = 0x2028;
    pub const PARAGRAPH_SEPARATOR: u32 = 0x2029;
    pub const NEXT_LINE: u32 = 0x0085;

    pub const SPACE: u32 = 0x0020;
    pub const TAB: u32 = 0x09;
    pub const VERTICAL_TAB: u32 = 0x0B;
    pub const FORM_FEED: u32 = 0x0C;
    pub const NON_BREAKING_SPACE: u32 = 0x00A0;
    pub const OGHAM: u32 = 0x1680;
    pub const EN_QUAD: u32 = 0x2000;
    pub const ZERO_WIDTH_SPACE: u32 = 0x200B;
    pub const NARROW_NO_BREAK_SPACE: u32 = 0x202F;
    pub const MATHEMATICAL_SPACE: u32 = 0x205F;
    pub const IDEOGRAPHIC_SPACE: u32 = 0x3000;
    pub const BYTE_ORDER_MARK: u32 = 0xFEFF;

    pub const _0: u32 = 0x30;
    pub const _9: u32 = 0x39;
    pub const _7: u32 = 0x37;

    pub const UPPER_A: u32 = 0x41;
    pub const UPPER_F: u32 = 0x46;
    pub const UPPER_Z: u32 = 0x5A;
    pub const LOWER_A: u32 = 0x61;
    pub const LOWER_F: u32 = 0x66;
    pub const LOWER_Z: u32 = 0x7A;

    pub const UNDERSCORE: u32 = 0x5F;
}

/// Check if character is a line break (LF, CR, LS, PS).
#[wasm_bindgen(js_name = isLineBreak)]
pub fn is_line_break(ch: u32) -> bool {
    ch == char_codes::LINE_FEED
        || ch == char_codes::CARRIAGE_RETURN
        || ch == char_codes::LINE_SEPARATOR
        || ch == char_codes::PARAGRAPH_SEPARATOR
}

/// Check if character is a single-line whitespace (not including line breaks).
#[wasm_bindgen(js_name = isWhiteSpaceSingleLine)]
pub fn is_white_space_single_line(ch: u32) -> bool {
    ch == char_codes::SPACE
        || ch == char_codes::TAB
        || ch == char_codes::VERTICAL_TAB
        || ch == char_codes::FORM_FEED
        || ch == char_codes::NON_BREAKING_SPACE
        || ch == char_codes::NEXT_LINE
        || ch == char_codes::OGHAM
        || (ch >= char_codes::EN_QUAD && ch <= char_codes::ZERO_WIDTH_SPACE)
        || ch == char_codes::NARROW_NO_BREAK_SPACE
        || ch == char_codes::MATHEMATICAL_SPACE
        || ch == char_codes::IDEOGRAPHIC_SPACE
        || ch == char_codes::BYTE_ORDER_MARK
}

/// Check if character is any whitespace (including line breaks).
#[wasm_bindgen(js_name = isWhiteSpaceLike)]
pub fn is_white_space_like(ch: u32) -> bool {
    is_white_space_single_line(ch) || is_line_break(ch)
}

/// Check if character is a decimal digit (0-9).
#[wasm_bindgen(js_name = isDigit)]
pub fn is_digit(ch: u32) -> bool {
    ch >= char_codes::_0 && ch <= char_codes::_9
}

/// Check if character is an octal digit (0-7).
#[wasm_bindgen(js_name = isOctalDigit)]
pub fn is_octal_digit(ch: u32) -> bool {
    ch >= char_codes::_0 && ch <= char_codes::_7
}

/// Check if character is a hexadecimal digit (0-9, A-F, a-f).
#[wasm_bindgen(js_name = isHexDigit)]
pub fn is_hex_digit(ch: u32) -> bool {
    is_digit(ch)
        || (ch >= char_codes::UPPER_A && ch <= char_codes::UPPER_F)
        || (ch >= char_codes::LOWER_A && ch <= char_codes::LOWER_F)
}

/// Check if character is an ASCII letter (A-Z, a-z).
#[wasm_bindgen(js_name = isASCIILetter)]
pub fn is_ascii_letter(ch: u32) -> bool {
    (ch >= char_codes::UPPER_A && ch <= char_codes::UPPER_Z)
        || (ch >= char_codes::LOWER_A && ch <= char_codes::LOWER_Z)
}

/// Check if character is a word character (A-Z, a-z, 0-9, _).
#[wasm_bindgen(js_name = isWordCharacter)]
pub fn is_word_character(ch: u32) -> bool {
    is_ascii_letter(ch) || is_digit(ch) || ch == char_codes::UNDERSCORE
}

// =============================================================================
// Unit Tests
// =============================================================================

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_add() {
        assert_eq!(add(2, 2), 4);
        assert_eq!(add(-1, 1), 0);
    }

    #[test]
    fn test_compare_strings_case_sensitive() {
        // Equal strings
        assert_eq!(
            compare_strings_case_sensitive(Some("abc".into()), Some("abc".into())),
            Comparison::EqualTo
        );

        // Less than
        assert_eq!(
            compare_strings_case_sensitive(Some("abc".into()), Some("abd".into())),
            Comparison::LessThan
        );

        // Greater than
        assert_eq!(
            compare_strings_case_sensitive(Some("abd".into()), Some("abc".into())),
            Comparison::GreaterThan
        );

        // Case matters: 'B' (66) < 'a' (97) in ASCII
        assert_eq!(
            compare_strings_case_sensitive(Some("B".into()), Some("a".into())),
            Comparison::LessThan
        );

        // None handling
        assert_eq!(
            compare_strings_case_sensitive(None, Some("a".into())),
            Comparison::LessThan
        );
        assert_eq!(
            compare_strings_case_sensitive(Some("a".into()), None),
            Comparison::GreaterThan
        );
        assert_eq!(
            compare_strings_case_sensitive(None, None),
            Comparison::EqualTo
        );
    }

    #[test]
    fn test_compare_strings_case_insensitive() {
        // Case-insensitive equal
        assert_eq!(
            compare_strings_case_insensitive(Some("ABC".into()), Some("abc".into())),
            Comparison::EqualTo
        );

        // Case-insensitive less than
        assert_eq!(
            compare_strings_case_insensitive(Some("abc".into()), Some("ABD".into())),
            Comparison::LessThan
        );

        // Case-insensitive greater than
        assert_eq!(
            compare_strings_case_insensitive(Some("ABD".into()), Some("abc".into())),
            Comparison::GreaterThan
        );
    }

    #[test]
    fn test_equate_strings() {
        assert!(equate_strings_case_sensitive("abc", "abc"));
        assert!(!equate_strings_case_sensitive("abc", "ABC"));

        assert!(equate_strings_case_insensitive("abc", "ABC"));
        assert!(equate_strings_case_insensitive("ABC", "abc"));
        assert!(!equate_strings_case_insensitive("abc", "abd"));
    }

    // Path utility tests

    #[test]
    fn test_is_any_directory_separator() {
        assert!(is_any_directory_separator('/' as u32));
        assert!(is_any_directory_separator('\\' as u32));
        assert!(!is_any_directory_separator('a' as u32));
        assert!(!is_any_directory_separator(':' as u32));
    }

    #[test]
    fn test_normalize_slashes() {
        assert_eq!(normalize_slashes("path/to/file"), "path/to/file");
        assert_eq!(normalize_slashes("path\\to\\file"), "path/to/file");
        assert_eq!(normalize_slashes("path\\to/file"), "path/to/file");
        assert_eq!(normalize_slashes("c:\\windows\\system32"), "c:/windows/system32");
    }

    #[test]
    fn test_has_trailing_directory_separator() {
        assert!(has_trailing_directory_separator("/path/to/dir/"));
        assert!(has_trailing_directory_separator("path\\"));
        assert!(!has_trailing_directory_separator("/path/to/file.ext"));
        assert!(!has_trailing_directory_separator(""));
    }

    #[test]
    fn test_path_is_relative() {
        assert!(path_is_relative("./path"));
        assert!(path_is_relative(".\\path"));
        assert!(path_is_relative("../path"));
        assert!(path_is_relative("..\\path"));
        assert!(path_is_relative("."));
        assert!(path_is_relative(".."));
        assert!(!path_is_relative("/absolute/path"));
        assert!(!path_is_relative("path/to/file"));
        assert!(!path_is_relative("c:/windows"));
    }

    #[test]
    fn test_remove_trailing_directory_separator() {
        assert_eq!(remove_trailing_directory_separator("/path/to/dir/"), "/path/to/dir");
        assert_eq!(remove_trailing_directory_separator("/path/to/file"), "/path/to/file");
        assert_eq!(remove_trailing_directory_separator("/"), "/");
    }

    #[test]
    fn test_ensure_trailing_directory_separator() {
        assert_eq!(ensure_trailing_directory_separator("/path/to/dir"), "/path/to/dir/");
        assert_eq!(ensure_trailing_directory_separator("/path/to/dir/"), "/path/to/dir/");
    }

    #[test]
    fn test_get_base_file_name() {
        assert_eq!(get_base_file_name("/path/to/file.ext"), "file.ext");
        assert_eq!(get_base_file_name("/path/to/"), "to");
        assert_eq!(get_base_file_name("file.ext"), "file.ext");
        assert_eq!(get_base_file_name("/"), "");
    }

    #[test]
    fn test_has_extension() {
        assert!(has_extension("file.ext"));
        assert!(has_extension("/path/to/file.ts"));
        assert!(!has_extension("/path/to/"));
        assert!(!has_extension("noextension"));
    }

    #[test]
    fn test_file_extension_is() {
        assert!(file_extension_is("file.ts", ".ts"));
        assert!(file_extension_is("/path/to/file.d.ts", ".d.ts"));
        assert!(!file_extension_is("file.ts", ".js"));
        assert!(!file_extension_is(".ts", ".ts")); // path must be longer than extension
    }

    // Character classification tests

    #[test]
    fn test_is_line_break() {
        assert!(is_line_break(0x0A)); // LF
        assert!(is_line_break(0x0D)); // CR
        assert!(is_line_break(0x2028)); // Line separator
        assert!(is_line_break(0x2029)); // Paragraph separator
        assert!(!is_line_break(0x20)); // Space
        assert!(!is_line_break(0x09)); // Tab
    }

    #[test]
    fn test_is_white_space_single_line() {
        assert!(is_white_space_single_line(0x20)); // Space
        assert!(is_white_space_single_line(0x09)); // Tab
        assert!(is_white_space_single_line(0x0B)); // Vertical tab
        assert!(is_white_space_single_line(0x0C)); // Form feed
        assert!(is_white_space_single_line(0xA0)); // Non-breaking space
        assert!(!is_white_space_single_line(0x0A)); // LF is not single-line whitespace
        assert!(!is_white_space_single_line(0x61)); // 'a'
    }

    #[test]
    fn test_is_white_space_like() {
        // Includes both single-line and line breaks
        assert!(is_white_space_like(0x20)); // Space
        assert!(is_white_space_like(0x0A)); // LF
        assert!(is_white_space_like(0x0D)); // CR
        assert!(!is_white_space_like(0x61)); // 'a'
    }

    #[test]
    fn test_is_digit() {
        assert!(is_digit('0' as u32));
        assert!(is_digit('5' as u32));
        assert!(is_digit('9' as u32));
        assert!(!is_digit('a' as u32));
        assert!(!is_digit('A' as u32));
    }

    #[test]
    fn test_is_octal_digit() {
        assert!(is_octal_digit('0' as u32));
        assert!(is_octal_digit('7' as u32));
        assert!(!is_octal_digit('8' as u32));
        assert!(!is_octal_digit('9' as u32));
    }

    #[test]
    fn test_is_hex_digit() {
        assert!(is_hex_digit('0' as u32));
        assert!(is_hex_digit('9' as u32));
        assert!(is_hex_digit('a' as u32));
        assert!(is_hex_digit('f' as u32));
        assert!(is_hex_digit('A' as u32));
        assert!(is_hex_digit('F' as u32));
        assert!(!is_hex_digit('g' as u32));
        assert!(!is_hex_digit('G' as u32));
    }

    #[test]
    fn test_is_ascii_letter() {
        assert!(is_ascii_letter('a' as u32));
        assert!(is_ascii_letter('z' as u32));
        assert!(is_ascii_letter('A' as u32));
        assert!(is_ascii_letter('Z' as u32));
        assert!(!is_ascii_letter('0' as u32));
        assert!(!is_ascii_letter('_' as u32));
    }

    #[test]
    fn test_is_word_character() {
        assert!(is_word_character('a' as u32));
        assert!(is_word_character('Z' as u32));
        assert!(is_word_character('0' as u32));
        assert!(is_word_character('_' as u32));
        assert!(!is_word_character('-' as u32));
        assert!(!is_word_character(' ' as u32));
    }
}
