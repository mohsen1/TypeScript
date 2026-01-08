//! Tests for source_map.rs

use crate::source_map::*;

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
