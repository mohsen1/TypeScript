//! Zang WASM Bindings
//!
//! WebAssembly bindings for the Zang TypeScript compiler.
//! Provides a JavaScript-friendly API for parsing and type checking TypeScript.

use wasm_bindgen::prelude::*;
use zang_core::{Arena, StringInterner};
use zang_parser::Parser;

/// Initialize the WASM module
#[wasm_bindgen(start)]
pub fn init() {
    // Set up panic hook for better error messages
    #[cfg(feature = "console_error_panic_hook")]
    console_error_panic_hook::set_once();
}

/// Parse TypeScript source code and return a JSON representation of the AST
#[wasm_bindgen]
pub fn parse(source: &str, file_name: &str) -> Result<String, JsValue> {
    let arena = Arena::new();
    let interner = StringInterner::new();

    let parser = Parser::new(source, &arena, &interner);
    let result = parser.parse(file_name);

    // Convert errors to JSON
    let errors: Vec<_> = result.errors.iter().map(|e| {
        serde_json::json!({
            "message": e.message,
            "start": e.span.start,
            "end": e.span.end,
        })
    }).collect();

    let response = serde_json::json!({
        "success": result.errors.is_empty(),
        "errors": errors,
        "file_name": file_name,
    });

    serde_json::to_string(&response)
        .map_err(|e| JsValue::from_str(&e.to_string()))
}

/// Check if the source has syntax errors
#[wasm_bindgen]
pub fn has_syntax_errors(source: &str) -> bool {
    let arena = Arena::new();
    let interner = StringInterner::new();

    let parser = Parser::new(source, &arena, &interner);
    let result = parser.parse("input.ts");

    !result.errors.is_empty()
}

/// Get syntax errors as JSON
#[wasm_bindgen]
pub fn get_syntax_errors(source: &str) -> String {
    let arena = Arena::new();
    let interner = StringInterner::new();

    let parser = Parser::new(source, &arena, &interner);
    let result = parser.parse("input.ts");

    let errors: Vec<_> = result.errors.iter().map(|e| {
        serde_json::json!({
            "message": e.message,
            "start": e.span.start,
            "end": e.span.end,
            "code": format!("{:?}", e.code),
        })
    }).collect();

    serde_json::to_string(&errors).unwrap_or_else(|_| "[]".to_string())
}

/// Get the version of the Zang compiler
#[wasm_bindgen]
pub fn version() -> String {
    env!("CARGO_PKG_VERSION").to_string()
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_parse_simple() {
        let result = parse("let x = 1;", "test.ts");
        assert!(result.is_ok());
    }

    #[test]
    fn test_has_syntax_errors() {
        assert!(!has_syntax_errors("let x = 1;"));
        assert!(has_syntax_errors("let x = ;")); // Missing value
    }

    #[test]
    fn test_version() {
        let v = version();
        assert!(!v.is_empty());
    }
}
