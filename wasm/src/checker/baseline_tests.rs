//! Baseline tests runner for TypeScript compiler test cases
//!
//! This module provides utilities to run the Rust type checker against
//! TypeScript's compiler test suite and compare results against baselines.

use crate::parser_impl::ParserState;
use crate::binder::BinderState;
use super::state::CheckerState;

/// Parse expected error codes from a .errors.txt baseline file content
pub fn parse_expected_errors(content: &str) -> Vec<u32> {
    let mut codes = Vec::new();

    // Parse lines like: "file.ts(1,13): error TS1110: Type expected."
    for line in content.lines() {
        if let Some(idx) = line.find(": error TS") {
            let rest = &line[idx + 10..];
            if let Some(colon_idx) = rest.find(':') {
                if let Ok(code) = rest[..colon_idx].parse::<u32>() {
                    codes.push(code);
                }
            }
        }
    }

    codes.sort();
    codes
}

/// Run the checker on source code and return error codes
pub fn check_source(file_name: &str, source: &str) -> Vec<u32> {
    // Run parser
    let mut parser = ParserState::new(file_name.to_string(), source.to_string());
    let root = parser.parse_source_file();

    // Run binder
    let mut binder = BinderState::new();
    binder.bind_source_file(&parser.arena, root);

    // Run checker
    let mut checker = CheckerState::new(
        &parser.arena,
        &binder.symbols,
        &binder.file_locals,
        &binder.node_symbols,
        file_name.to_string(),
    );
    checker.check_source_file(root);

    // Collect actual error codes
    let mut codes: Vec<u32> = checker.diagnostics
        .iter()
        .map(|d| d.code)
        .collect();
    codes.sort();
    codes
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_parse_expected_errors() {
        let content = r#"
ArrowFunction1.ts(1,13): error TS1110: Type expected.


==== ArrowFunction1.ts (1 errors) ====
    var v = (a: ) => {
                ~
!!! error TS1110: Type expected.

    };
"#;
        let codes = parse_expected_errors(content);
        assert_eq!(codes, vec![1110]);
    }

    #[test]
    fn test_parse_multiple_errors() {
        let content = r#"
test.ts(1,1): error TS2304: Cannot find name 'x'.
test.ts(2,1): error TS2322: Type 'string' is not assignable to type 'number'.
test.ts(3,5): error TS2304: Cannot find name 'y'.
"#;
        let codes = parse_expected_errors(content);
        assert_eq!(codes, vec![2304, 2304, 2322]);
    }

    /// Test a simple case with no expected errors
    #[test]
    fn test_simple_no_errors() {
        let code = "const x: number = 42;";

        let mut parser = ParserState::new("test.ts".to_string(), code.to_string());
        let root = parser.parse_source_file();

        let mut binder = BinderState::new();
        binder.bind_source_file(&parser.arena, root);

        let mut checker = CheckerState::new(
            &parser.arena,
            &binder.symbols,
            &binder.file_locals,
            &binder.node_symbols,
            "test.ts".to_string(),
        );
        checker.check_source_file(root);

        assert!(checker.diagnostics.is_empty(),
            "Expected no errors, got: {:?}", checker.diagnostics);
    }

    /// Test a case that should produce TS2322
    #[test]
    fn test_type_mismatch_error() {
        let code = r#"const x: number = "hello";"#;

        let mut parser = ParserState::new("test.ts".to_string(), code.to_string());
        let root = parser.parse_source_file();

        let mut binder = BinderState::new();
        binder.bind_source_file(&parser.arena, root);

        let mut checker = CheckerState::new(
            &parser.arena,
            &binder.symbols,
            &binder.file_locals,
            &binder.node_symbols,
            "test.ts".to_string(),
        );
        checker.check_source_file(root);

        let codes: Vec<u32> = checker.diagnostics.iter().map(|d| d.code).collect();
        assert!(codes.contains(&2322),
            "Expected TS2322 error, got: {:?}", codes);
    }
}
