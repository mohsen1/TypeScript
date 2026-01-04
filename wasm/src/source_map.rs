//! Source Map Generation (Phase 6.2)
//!
//! Implements source map generation for the emitter, following the Source Map V3 spec.
//! https://sourcemaps.info/spec.html

/// VLQ (Variable-Length Quantity) encoding for source maps.
/// Used to encode position data in a compact format.
pub mod vlq {
    /// Base64 encoding table for VLQ
    const BASE64_CHARS: &[u8] = b"ABCDEFGHIJKLMNOPQRSTUVWXYZabcdefghijklmnopqrstuvwxyz0123456789+/";

    /// Encode a signed integer as a VLQ string.
    /// The format is: continuation bits (5 bits of data) with sign in the LSB.
    pub fn encode(value: i32) -> String {
        let mut result = String::new();

        // Convert to unsigned with sign in LSB
        let mut unsigned: u32 = if value < 0 {
            (((-value) << 1) | 1) as u32
        } else {
            (value << 1) as u32
        };

        loop {
            // Extract 5 bits
            let mut digit = (unsigned & 0x1F) as u8;
            unsigned >>= 5;

            // Set continuation bit if there are more digits
            if unsigned > 0 {
                digit |= 0x20;
            }

            result.push(BASE64_CHARS[digit as usize] as char);

            if unsigned == 0 {
                break;
            }
        }

        result
    }

    /// Decode a VLQ string to a signed integer.
    /// Returns the decoded value and the number of characters consumed.
    pub fn decode(input: &str) -> Option<(i32, usize)> {
        let bytes = input.as_bytes();
        let mut result: u32 = 0;
        let mut shift = 0;
        let mut index = 0;

        loop {
            if index >= bytes.len() {
                return None;
            }

            let byte = bytes[index];
            let digit = match byte {
                b'A'..=b'Z' => byte - b'A',
                b'a'..=b'z' => byte - b'a' + 26,
                b'0'..=b'9' => byte - b'0' + 52,
                b'+' => 62,
                b'/' => 63,
                _ => return None,
            };

            let continuation = (digit & 0x20) != 0;
            let value = (digit & 0x1F) as u32;
            result |= value << shift;
            shift += 5;
            index += 1;

            if !continuation {
                break;
            }
        }

        // Extract sign from LSB
        let is_negative = (result & 1) == 1;
        let value = (result >> 1) as i32;
        let signed = if is_negative { -value } else { value };

        Some((signed, index))
    }
}

/// A position in the source file (line and column, both 0-indexed).
#[derive(Debug, Clone, Copy, Default, PartialEq, Eq)]
pub struct SourcePosition {
    pub line: u32,
    pub column: u32,
}

/// A single mapping entry from generated to source position.
#[derive(Debug, Clone, Default)]
pub struct Mapping {
    /// Generated position (in the output file)
    pub generated: SourcePosition,
    /// Source file index (into the sources array)
    pub source_index: Option<u32>,
    /// Original position (in the source file)
    pub original: Option<SourcePosition>,
    /// Names index (into the names array)
    pub name_index: Option<u32>,
}

/// Source map builder that accumulates mappings and produces a source map.
#[derive(Debug, Default)]
pub struct SourceMapGenerator {
    /// The generated file name
    pub file: String,
    /// The source root (prefix for source paths)
    pub source_root: String,
    /// List of source file paths
    pub sources: Vec<String>,
    /// List of source file contents (optional)
    pub sources_content: Vec<Option<String>>,
    /// List of symbol names used in mappings
    pub names: Vec<String>,
    /// All mappings, sorted by generated position
    mappings: Vec<Mapping>,

    /// State for incremental mapping encoding
    prev_generated_line: u32,
    prev_generated_column: u32,
    prev_source_index: u32,
    prev_source_line: u32,
    prev_source_column: u32,
    prev_name_index: u32,
}

impl SourceMapGenerator {
    /// Create a new source map generator for the given output file.
    pub fn new(file: String) -> Self {
        SourceMapGenerator {
            file,
            ..Default::default()
        }
    }

    /// Add a source file to the sources list, returning its index.
    pub fn add_source(&mut self, path: String) -> u32 {
        // Check if source already exists
        if let Some(index) = self.sources.iter().position(|s| s == &path) {
            return index as u32;
        }

        let index = self.sources.len() as u32;
        self.sources.push(path);
        self.sources_content.push(None);
        index
    }

    /// Add a source file with its content.
    pub fn add_source_with_content(&mut self, path: String, content: String) -> u32 {
        let index = self.add_source(path);
        if (index as usize) < self.sources_content.len() {
            self.sources_content[index as usize] = Some(content);
        }
        index
    }

    /// Add a name to the names list, returning its index.
    pub fn add_name(&mut self, name: String) -> u32 {
        // Check if name already exists
        if let Some(index) = self.names.iter().position(|n| n == &name) {
            return index as u32;
        }

        let index = self.names.len() as u32;
        self.names.push(name);
        index
    }

    /// Add a mapping from generated position to source position.
    pub fn add_mapping(&mut self, mapping: Mapping) {
        self.mappings.push(mapping);
    }

    /// Add a simple mapping (generated to source, no name).
    pub fn add_simple_mapping(
        &mut self,
        generated_line: u32,
        generated_column: u32,
        source_index: u32,
        original_line: u32,
        original_column: u32,
    ) {
        self.mappings.push(Mapping {
            generated: SourcePosition {
                line: generated_line,
                column: generated_column,
            },
            source_index: Some(source_index),
            original: Some(SourcePosition {
                line: original_line,
                column: original_column,
            }),
            name_index: None,
        });
    }

    /// Add a mapping with a name.
    pub fn add_named_mapping(
        &mut self,
        generated_line: u32,
        generated_column: u32,
        source_index: u32,
        original_line: u32,
        original_column: u32,
        name_index: u32,
    ) {
        self.mappings.push(Mapping {
            generated: SourcePosition {
                line: generated_line,
                column: generated_column,
            },
            source_index: Some(source_index),
            original: Some(SourcePosition {
                line: original_line,
                column: original_column,
            }),
            name_index: Some(name_index),
        });
    }

    /// Encode all mappings as a VLQ-encoded string.
    fn encode_mappings(&mut self) -> String {
        // Sort mappings by generated position
        self.mappings.sort_by(|a, b| {
            a.generated.line.cmp(&b.generated.line)
                .then_with(|| a.generated.column.cmp(&b.generated.column))
        });

        let mut result = String::new();

        // Reset state
        self.prev_generated_line = 0;
        self.prev_generated_column = 0;
        self.prev_source_index = 0;
        self.prev_source_line = 0;
        self.prev_source_column = 0;
        self.prev_name_index = 0;

        for mapping in &self.mappings {
            // Add semicolons for skipped lines
            while self.prev_generated_line < mapping.generated.line {
                result.push(';');
                self.prev_generated_line += 1;
                self.prev_generated_column = 0;
            }

            // Add comma separator between mappings on the same line
            if !result.is_empty() && !result.ends_with(';') {
                result.push(',');
            }

            // Encode generated column (always present, relative to previous)
            let gen_col_delta = mapping.generated.column as i32 - self.prev_generated_column as i32;
            result.push_str(&vlq::encode(gen_col_delta));
            self.prev_generated_column = mapping.generated.column;

            // If we have source information, encode it
            if let Some(source_index) = mapping.source_index {
                // Source index (relative)
                let source_delta = source_index as i32 - self.prev_source_index as i32;
                result.push_str(&vlq::encode(source_delta));
                self.prev_source_index = source_index;

                if let Some(original) = mapping.original {
                    // Original line (relative)
                    let line_delta = original.line as i32 - self.prev_source_line as i32;
                    result.push_str(&vlq::encode(line_delta));
                    self.prev_source_line = original.line;

                    // Original column (relative)
                    let col_delta = original.column as i32 - self.prev_source_column as i32;
                    result.push_str(&vlq::encode(col_delta));
                    self.prev_source_column = original.column;

                    // Name index (relative, optional)
                    if let Some(name_index) = mapping.name_index {
                        let name_delta = name_index as i32 - self.prev_name_index as i32;
                        result.push_str(&vlq::encode(name_delta));
                        self.prev_name_index = name_index;
                    }
                }
            }
        }

        result
    }

    /// Generate the source map as a JSON string.
    pub fn to_json(&mut self) -> String {
        let mappings = self.encode_mappings();

        let mut json = String::from("{\n");
        json.push_str("  \"version\": 3,\n");

        if !self.file.is_empty() {
            json.push_str(&format!("  \"file\": \"{}\",\n", escape_json(&self.file)));
        }

        if !self.source_root.is_empty() {
            json.push_str(&format!("  \"sourceRoot\": \"{}\",\n", escape_json(&self.source_root)));
        }

        // Sources array
        json.push_str("  \"sources\": [");
        for (i, source) in self.sources.iter().enumerate() {
            if i > 0 {
                json.push_str(", ");
            }
            json.push('"');
            json.push_str(&escape_json(source));
            json.push('"');
        }
        json.push_str("],\n");

        // Sources content (optional)
        if self.sources_content.iter().any(|c| c.is_some()) {
            json.push_str("  \"sourcesContent\": [");
            for (i, content) in self.sources_content.iter().enumerate() {
                if i > 0 {
                    json.push_str(", ");
                }
                match content {
                    Some(c) => {
                        json.push('"');
                        json.push_str(&escape_json(c));
                        json.push('"');
                    }
                    None => json.push_str("null"),
                }
            }
            json.push_str("],\n");
        }

        // Names array
        json.push_str("  \"names\": [");
        for (i, name) in self.names.iter().enumerate() {
            if i > 0 {
                json.push_str(", ");
            }
            json.push('"');
            json.push_str(&escape_json(name));
            json.push('"');
        }
        json.push_str("],\n");

        // Mappings
        json.push_str(&format!("  \"mappings\": \"{}\"\n", mappings));

        json.push('}');
        json
    }

    /// Generate inline source map as a data URL.
    pub fn to_inline_comment(&mut self) -> String {
        let json = self.to_json();
        let base64 = base64_encode(json.as_bytes());
        format!("//# sourceMappingURL=data:application/json;base64,{}", base64)
    }
}

/// Escape a string for JSON encoding.
fn escape_json(s: &str) -> String {
    let mut result = String::with_capacity(s.len());
    for c in s.chars() {
        match c {
            '"' => result.push_str("\\\""),
            '\\' => result.push_str("\\\\"),
            '\n' => result.push_str("\\n"),
            '\r' => result.push_str("\\r"),
            '\t' => result.push_str("\\t"),
            c if c.is_control() => {
                result.push_str(&format!("\\u{:04x}", c as u32));
            }
            c => result.push(c),
        }
    }
    result
}

/// Simple base64 encoding for inline source maps.
fn base64_encode(data: &[u8]) -> String {
    const CHARS: &[u8] = b"ABCDEFGHIJKLMNOPQRSTUVWXYZabcdefghijklmnopqrstuvwxyz0123456789+/";

    let mut result = String::with_capacity((data.len() + 2) / 3 * 4);

    for chunk in data.chunks(3) {
        let b0 = chunk[0] as u32;
        let b1 = chunk.get(1).map(|&b| b as u32).unwrap_or(0);
        let b2 = chunk.get(2).map(|&b| b as u32).unwrap_or(0);

        let triple = (b0 << 16) | (b1 << 8) | b2;

        result.push(CHARS[((triple >> 18) & 0x3F) as usize] as char);
        result.push(CHARS[((triple >> 12) & 0x3F) as usize] as char);

        if chunk.len() > 1 {
            result.push(CHARS[((triple >> 6) & 0x3F) as usize] as char);
        } else {
            result.push('=');
        }

        if chunk.len() > 2 {
            result.push(CHARS[(triple & 0x3F) as usize] as char);
        } else {
            result.push('=');
        }
    }

    result
}

#[cfg(test)]
mod tests {
    use super::*;

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
        assert!(json.contains("\"version\": 3"));
        assert!(json.contains("\"file\": \"output.js\""));
        assert!(json.contains("\"sources\": [\"input.ts\"]"));
        assert!(json.contains("\"mappings\""));
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
        assert!(json.contains("\"names\": [\"myVariable\"]"));
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
}
