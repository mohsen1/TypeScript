//! Tests for source_map.rs

use crate::emit_context::EmitContext;
use crate::lowering_pass::LoweringPass;
use crate::source_map::*;
use crate::thin_emitter::{PrinterOptions, ScriptTarget, ThinPrinter};
use crate::thin_parser::ThinParserState;
use serde_json::Value;

#[derive(Debug)]
struct DecodedMapping {
    generated_line: u32,
    generated_column: u32,
    source_index: u32,
    original_line: u32,
    original_column: u32,
    #[allow(dead_code)]
    name_index: Option<u32>,
}

fn decode_mappings(mappings: &str) -> Vec<DecodedMapping> {
    let mut decoded = Vec::new();
    let mut generated_line = 0u32;
    let mut prev_generated_column = 0i32;
    let mut prev_source_index = 0i32;
    let mut prev_original_line = 0i32;
    let mut prev_original_column = 0i32;
    let mut prev_name_index = 0i32;

    for line in mappings.split(';') {
        if line.is_empty() {
            generated_line += 1;
            prev_generated_column = 0;
            continue;
        }

        for segment in line.split(',') {
            if segment.is_empty() {
                continue;
            }

            let mut rest = segment;
            let (gen_col_delta, consumed) =
                vlq::decode(rest).expect("decode generated column");
            rest = &rest[consumed..];

            let gen_col = prev_generated_column + gen_col_delta;
            prev_generated_column = gen_col;

            if rest.is_empty() {
                continue;
            }

            let (src_delta, consumed) =
                vlq::decode(rest).expect("decode source index");
            rest = &rest[consumed..];
            let (orig_line_delta, consumed) =
                vlq::decode(rest).expect("decode original line");
            rest = &rest[consumed..];
            let (orig_col_delta, consumed) =
                vlq::decode(rest).expect("decode original column");
            rest = &rest[consumed..];

            let source_index = prev_source_index + src_delta;
            let original_line = prev_original_line + orig_line_delta;
            let original_column = prev_original_column + orig_col_delta;

            prev_source_index = source_index;
            prev_original_line = original_line;
            prev_original_column = original_column;

            let name_index = if !rest.is_empty() {
                let (name_delta, consumed) =
                    vlq::decode(rest).expect("decode name index");
                rest = &rest[consumed..];
                let name_index = prev_name_index + name_delta;
                prev_name_index = name_index;
                Some(name_index as u32)
            } else {
                None
            };

            assert!(
                rest.is_empty(),
                "unexpected trailing data in mappings segment: {segment}"
            );

            decoded.push(DecodedMapping {
                generated_line,
                generated_column: gen_col as u32,
                source_index: source_index as u32,
                original_line: original_line as u32,
                original_column: original_column as u32,
                name_index,
            });
        }

        generated_line += 1;
        prev_generated_column = 0;
    }

    decoded
}

fn find_line_col(text: &str, needle: &str) -> (u32, u32) {
    let idx = text
        .find(needle)
        .unwrap_or_else(|| panic!("expected to find {needle} in {text}"));

    let mut line = 0u32;
    let mut col = 0u32;
    for &b in text.as_bytes().iter().take(idx) {
        if b == b'\n' {
            line += 1;
            col = 0;
        } else {
            col += 1;
        }
    }

    (line, col)
}

#[test]
fn test_vlq_encode_positive() {
    // Simple positive numbers
    assert_eq!(vlq::encode(0), "A");
    assert_eq!(vlq::encode(1), "C");
    assert_eq!(vlq::encode(15), "e");
    assert_eq!(vlq::encode(16), "gB");
}

#[test]
fn test_vlq_encode_negative() {
    // Negative numbers (sign in LSB)
    assert_eq!(vlq::encode(-1), "D");
    assert_eq!(vlq::encode(-15), "f");
}

#[test]
fn test_vlq_decode() {
    // Decode what we encode
    for value in [-100, -1, 0, 1, 100, 1000] {
        let encoded = vlq::encode(value);
        let (decoded, consumed) = vlq::decode(&encoded).unwrap();
        assert_eq!(decoded, value, "Failed for value {}", value);
        assert_eq!(consumed, encoded.len());
    }
}

#[test]
fn test_source_map_simple() {
    let mut generator = SourceMapGenerator::new("output.js".to_string());
    let source_idx = generator.add_source("input.ts".to_string());

    generator.add_simple_mapping(0, 0, source_idx, 0, 0);
    generator.add_simple_mapping(0, 10, source_idx, 0, 5);
    generator.add_simple_mapping(1, 0, source_idx, 1, 0);

    let json = generator.to_json();
    assert!(json.contains("\"version\":3") || json.contains("\"version\": 3"), "Should be v3 source map: {}", json);
    assert!(json.contains("\"file\":\"output.js\"") || json.contains("\"file\": \"output.js\""), "Should have file: {}", json);
    assert!(json.contains("\"sources\":[\"input.ts\"]") || json.contains("\"sources\": [\"input.ts\"]"), "Should have sources: {}", json);
    assert!(json.contains("\"mappings\""), "Should have mappings: {}", json);
}

#[test]
fn test_source_map_with_content() {
    let mut generator = SourceMapGenerator::new("output.js".to_string());
    generator.add_source_with_content("input.ts".to_string(), "const x = 1;".to_string());

    generator.add_simple_mapping(0, 0, 0, 0, 0);

    let json = generator.to_json();
    assert!(json.contains("\"sourcesContent\""));
    assert!(json.contains("const x = 1;"));
}

#[test]
fn test_source_map_with_names() {
    let mut generator = SourceMapGenerator::new("output.js".to_string());
    let source_idx = generator.add_source("input.ts".to_string());
    let name_idx = generator.add_name("myVariable".to_string());

    generator.add_named_mapping(0, 0, source_idx, 0, 0, name_idx);

    let json = generator.to_json();
    assert!(json.contains("\"names\":[\"myVariable\"]") || json.contains("\"names\": [\"myVariable\"]"), "Should have names: {}", json);
}

#[test]
fn test_inline_source_map() {
    let mut generator = SourceMapGenerator::new("output.js".to_string());
    generator.add_source("input.ts".to_string());
    generator.add_simple_mapping(0, 0, 0, 0, 0);

    let inline = generator.to_inline_comment();
    assert!(inline.starts_with("//# sourceMappingURL=data:application/json;base64,"));
}

#[test]
fn test_base64_encode() {
    assert_eq!(base64_encode(b""), "");
    assert_eq!(base64_encode(b"f"), "Zg==");
    assert_eq!(base64_encode(b"fo"), "Zm8=");
    assert_eq!(base64_encode(b"foo"), "Zm9v");
    assert_eq!(base64_encode(b"foobar"), "Zm9vYmFy");
}

#[test]
fn test_escape_json() {
    assert_eq!(escape_json("hello"), "hello");
    assert_eq!(escape_json("hello\"world"), "hello\\\"world");
    assert_eq!(escape_json("path\\to\\file"), "path\\\\to\\\\file");
    assert_eq!(escape_json("line1\nline2"), "line1\\nline2");
}

#[test]
fn test_source_map_es5_transform_async_await_mapping() {
    let source = "async function fetch(payload) { await payload; }";
    let mut parser = ThinParserState::new("test.ts".to_string(), source.to_string());
    let root = parser.parse_source_file();

    let mut options = PrinterOptions::default();
    options.target = ScriptTarget::ES5;
    let ctx = EmitContext::with_options(options.clone());
    let transforms = LoweringPass::new(&parser.arena, &ctx).run(root);

    let mut printer =
        ThinPrinter::with_transforms_and_options(&parser.arena, transforms, options);
    printer.set_target_es5(ctx.target_es5);
    printer.set_source_map_text(parser.get_source_text());
    printer.enable_source_map("test.js", "test.ts");
    printer.emit(root);

    let output = printer.get_output().to_string();
    assert!(
        output.contains("__awaiter(") && output.contains("__generator("),
        "expected async downlevel output, got: {output}"
    );

    let map_json = printer.generate_source_map_json().expect("source map");
    let map_value: Value = serde_json::from_str(&map_json).expect("parse source map");
    let mappings = map_value
        .get("mappings")
        .and_then(|value| value.as_str())
        .unwrap_or("");

    let decoded = decode_mappings(mappings);
    let (source_line, source_col) = find_line_col(source, "payload");
    let (output_line, output_col) = find_line_col(&output, "payload");

    let mapping = decoded
        .iter()
        .find(|entry| {
            entry.original_line == source_line
                && entry.original_column == source_col
        })
        .unwrap_or_else(|| {
            panic!(
                "expected mapping for payload. mappings: {mappings} output: {output}"
            )
        });

    assert_eq!(mapping.source_index, 0);
    assert_eq!(mapping.generated_line, output_line);
    assert_eq!(mapping.generated_column, output_col);
}
