use wasm_bindgen::prelude::*;

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
}
