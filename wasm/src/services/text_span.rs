//! Text span and range utilities for position tracking.
//!
//! These types represent positions and ranges within source text,
//! matching TypeScript's TextSpan, TextRange, and TextChange types.

use serde::{Deserialize, Serialize};

/// A span of text in a source file, represented as start position and length.
/// Matches TypeScript's `TextSpan` interface.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize, Default)]
pub struct TextSpan {
    pub start: u32,
    pub length: u32,
}

impl TextSpan {
    /// Create a new TextSpan from start position and length.
    #[inline]
    pub fn new(start: u32, length: u32) -> Self {
        Self { start, length }
    }

    /// Create a TextSpan from start and end positions (exclusive end).
    #[inline]
    pub fn from_bounds(start: u32, end: u32) -> Self {
        debug_assert!(end >= start, "end must be >= start");
        Self {
            start,
            length: end - start,
        }
    }

    /// Get the end position (exclusive).
    #[inline]
    pub fn end(&self) -> u32 {
        self.start + self.length
    }

    /// Check if this span is empty (zero length).
    #[inline]
    pub fn is_empty(&self) -> bool {
        self.length == 0
    }

    /// Check if this span contains a position.
    #[inline]
    pub fn contains_position(&self, position: u32) -> bool {
        position >= self.start && position < self.end()
    }

    /// Check if this span contains another span entirely.
    #[inline]
    pub fn contains_span(&self, other: &TextSpan) -> bool {
        other.start >= self.start && other.end() <= self.end()
    }

    /// Check if this span overlaps with another span.
    #[inline]
    pub fn overlaps_with(&self, other: &TextSpan) -> bool {
        self.start < other.end() && other.start < self.end()
    }

    /// Check if this span intersects with another span (touching counts).
    #[inline]
    pub fn intersects_with(&self, other: &TextSpan) -> bool {
        self.start <= other.end() && other.start <= self.end()
    }

    /// Get the intersection of two spans, if any.
    pub fn intersection(&self, other: &TextSpan) -> Option<TextSpan> {
        let start = self.start.max(other.start);
        let end = self.end().min(other.end());
        if start < end {
            Some(TextSpan::from_bounds(start, end))
        } else {
            None
        }
    }

    /// Create a span that covers both this span and another.
    pub fn union(&self, other: &TextSpan) -> TextSpan {
        let start = self.start.min(other.start);
        let end = self.end().max(other.end());
        TextSpan::from_bounds(start, end)
    }
}

/// A range in a source file with pos and end.
/// Matches TypeScript's `TextRange` interface.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize, Default)]
pub struct TextRange {
    pub pos: u32,
    pub end: u32,
}

impl TextRange {
    /// Create a new TextRange.
    #[inline]
    pub fn new(pos: u32, end: u32) -> Self {
        Self { pos, end }
    }

    /// Get the length of this range.
    #[inline]
    pub fn len(&self) -> u32 {
        self.end.saturating_sub(self.pos)
    }

    /// Check if this range is empty.
    #[inline]
    pub fn is_empty(&self) -> bool {
        self.end <= self.pos
    }

    /// Convert to a TextSpan.
    #[inline]
    pub fn to_span(&self) -> TextSpan {
        TextSpan::from_bounds(self.pos, self.end)
    }

    /// Check if this range contains a position.
    #[inline]
    pub fn contains(&self, position: u32) -> bool {
        position >= self.pos && position < self.end
    }

    /// Check if this range contains another range.
    #[inline]
    pub fn contains_range(&self, other: &TextRange) -> bool {
        other.pos >= self.pos && other.end <= self.end
    }
}

impl From<TextSpan> for TextRange {
    fn from(span: TextSpan) -> Self {
        TextRange {
            pos: span.start,
            end: span.end(),
        }
    }
}

impl From<TextRange> for TextSpan {
    fn from(range: TextRange) -> Self {
        TextSpan::from_bounds(range.pos, range.end)
    }
}

/// A text change with span and replacement text.
/// Matches TypeScript's `TextChange` interface.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct TextChange {
    pub span: TextSpan,
    pub new_text: String,
}

impl TextChange {
    /// Create a new text change.
    pub fn new(span: TextSpan, new_text: String) -> Self {
        Self { span, new_text }
    }

    /// Create a text change for an insertion at a position.
    pub fn insert(position: u32, text: String) -> Self {
        Self {
            span: TextSpan::new(position, 0),
            new_text: text,
        }
    }

    /// Create a text change for a deletion of a span.
    pub fn delete(span: TextSpan) -> Self {
        Self {
            span,
            new_text: String::new(),
        }
    }

    /// Create a text change for replacing a span with new text.
    pub fn replace(span: TextSpan, new_text: String) -> Self {
        Self { span, new_text }
    }

    /// Check if this is an insertion (empty span).
    pub fn is_insertion(&self) -> bool {
        self.span.is_empty()
    }

    /// Check if this is a deletion (empty new text).
    pub fn is_deletion(&self) -> bool {
        self.new_text.is_empty() && !self.span.is_empty()
    }

    /// Get the change in length that this change would cause.
    pub fn length_delta(&self) -> i32 {
        self.new_text.len() as i32 - self.span.length as i32
    }
}

/// A text change range for incremental parsing.
/// Matches TypeScript's `TextChangeRange` interface.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub struct TextChangeRange {
    pub span: TextSpan,
    pub new_length: u32,
}

impl TextChangeRange {
    /// Create a new text change range.
    pub fn new(span: TextSpan, new_length: u32) -> Self {
        Self { span, new_length }
    }

    /// Create an unchanged range (no edits).
    pub fn unchanged() -> Self {
        Self {
            span: TextSpan::new(0, 0),
            new_length: 0,
        }
    }

    /// Check if this represents no change.
    pub fn is_unchanged(&self) -> bool {
        self.span.is_empty() && self.new_length == 0
    }

    /// Collapse multiple text change ranges into one.
    pub fn collapse_changes(changes: &[TextChangeRange]) -> TextChangeRange {
        if changes.is_empty() {
            return TextChangeRange::unchanged();
        }

        if changes.len() == 1 {
            return changes[0];
        }

        // Find the total span covered by all changes
        let mut start = u32::MAX;
        let mut old_end = 0u32;
        let mut new_end = 0u32;

        for change in changes {
            start = start.min(change.span.start);
            old_end = old_end.max(change.span.end());
            new_end = new_end.max(change.span.start + change.new_length);
        }

        TextChangeRange {
            span: TextSpan::from_bounds(start, old_end),
            new_length: new_end - start,
        }
    }
}

/// Line and character position (0-indexed).
/// Matches TypeScript's `LineAndCharacter` interface.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize, Default)]
pub struct LineAndCharacter {
    pub line: u32,
    pub character: u32,
}

impl LineAndCharacter {
    pub fn new(line: u32, character: u32) -> Self {
        Self { line, character }
    }
}

/// Compute line starts for a source text.
/// Returns an array where line_starts[i] is the position of the first character of line i.
pub fn compute_line_starts(text: &str) -> Vec<u32> {
    let mut result = vec![0];
    let mut pos = 0u32;

    for ch in text.chars() {
        pos += ch.len_utf8() as u32;
        if ch == '\n' {
            result.push(pos);
        } else if ch == '\r' {
            // Handle \r\n as a single line break
            if text.chars().nth(pos as usize) != Some('\n') {
                result.push(pos);
            }
        }
    }

    result
}

/// Get line and character from a position given line starts.
pub fn get_line_and_character_of_position(line_starts: &[u32], position: u32) -> LineAndCharacter {
    let line = line_starts
        .binary_search(&position)
        .unwrap_or_else(|i| i.saturating_sub(1));

    let line_start = line_starts.get(line).copied().unwrap_or(0);
    let character = position.saturating_sub(line_start);

    LineAndCharacter {
        line: line as u32,
        character,
    }
}

/// Get position from line and character given line starts.
pub fn get_position_of_line_and_character(
    line_starts: &[u32],
    line: u32,
    character: u32,
) -> Option<u32> {
    if line as usize >= line_starts.len() {
        return None;
    }
    Some(line_starts[line as usize] + character)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_text_span_creation() {
        let span = TextSpan::new(10, 5);
        assert_eq!(span.start, 10);
        assert_eq!(span.length, 5);
        assert_eq!(span.end(), 15);
    }

    #[test]
    fn test_text_span_from_bounds() {
        let span = TextSpan::from_bounds(10, 20);
        assert_eq!(span.start, 10);
        assert_eq!(span.length, 10);
        assert_eq!(span.end(), 20);
    }

    #[test]
    fn test_text_span_contains() {
        let span = TextSpan::new(10, 10);
        assert!(span.contains_position(10));
        assert!(span.contains_position(15));
        assert!(span.contains_position(19));
        assert!(!span.contains_position(9));
        assert!(!span.contains_position(20));
    }

    #[test]
    fn test_text_span_overlap() {
        let span1 = TextSpan::new(10, 10); // 10-20
        let span2 = TextSpan::new(15, 10); // 15-25

        assert!(span1.overlaps_with(&span2));
        assert!(span2.overlaps_with(&span1));

        let span3 = TextSpan::new(25, 10); // 25-35
        assert!(!span1.overlaps_with(&span3));
    }

    #[test]
    fn test_text_change() {
        let insert = TextChange::insert(10, "hello".to_string());
        assert!(insert.is_insertion());
        assert_eq!(insert.length_delta(), 5);

        let delete = TextChange::delete(TextSpan::new(10, 5));
        assert!(delete.is_deletion());
        assert_eq!(delete.length_delta(), -5);
    }

    #[test]
    fn test_line_starts() {
        let text = "line1\nline2\nline3";
        let line_starts = compute_line_starts(text);
        assert_eq!(line_starts, vec![0, 6, 12]);
    }

    #[test]
    fn test_line_and_character() {
        let line_starts = vec![0, 6, 12];

        let lc = get_line_and_character_of_position(&line_starts, 0);
        assert_eq!(lc.line, 0);
        assert_eq!(lc.character, 0);

        let lc = get_line_and_character_of_position(&line_starts, 8);
        assert_eq!(lc.line, 1);
        assert_eq!(lc.character, 2);
    }
}
