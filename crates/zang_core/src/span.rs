//! Source code spans for error reporting
//!
//! Provides types for tracking source locations of AST nodes.

/// A span in source code
///
/// Represents a contiguous region of source code, used for error messages
/// and source mapping.
#[derive(Clone, Copy, Debug, Eq, PartialEq, Hash, Default)]
pub struct Span {
    /// Start byte offset (inclusive)
    pub start: u32,
    /// End byte offset (exclusive)
    pub end: u32,
}

impl Span {
    /// Creates a new span
    pub const fn new(start: u32, end: u32) -> Self {
        Self { start, end }
    }

    /// Creates an empty span at a position
    pub const fn empty(pos: u32) -> Self {
        Self { start: pos, end: pos }
    }

    /// Returns the length of this span in bytes
    pub const fn len(&self) -> u32 {
        self.end - self.start
    }

    /// Returns true if this span is empty
    pub const fn is_empty(&self) -> bool {
        self.start == self.end
    }

    /// Extends this span to include another span
    pub fn extend(&self, other: Span) -> Span {
        Span {
            start: self.start.min(other.start),
            end: self.end.max(other.end),
        }
    }

    /// Returns true if this span contains the given position
    pub const fn contains(&self, pos: u32) -> bool {
        pos >= self.start && pos < self.end
    }

    /// Returns true if this span overlaps with another
    pub const fn overlaps(&self, other: &Span) -> bool {
        self.start < other.end && other.start < self.end
    }

    /// Converts to a range for slicing
    pub fn as_range(&self) -> std::ops::Range<usize> {
        self.start as usize..self.end as usize
    }
}

impl From<std::ops::Range<u32>> for Span {
    fn from(range: std::ops::Range<u32>) -> Self {
        Self {
            start: range.start,
            end: range.end,
        }
    }
}

impl From<std::ops::Range<usize>> for Span {
    fn from(range: std::ops::Range<usize>) -> Self {
        Self {
            start: range.start as u32,
            end: range.end as u32,
        }
    }
}

/// A span with an associated file ID
#[derive(Clone, Copy, Debug, Eq, PartialEq, Hash)]
pub struct FileSpan {
    /// The file this span belongs to
    pub file_id: u32,
    /// The span within the file
    pub span: Span,
}

impl FileSpan {
    /// Creates a new file span
    pub const fn new(file_id: u32, span: Span) -> Self {
        Self { file_id, span }
    }
}

/// Line and column information for display
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub struct LineCol {
    /// 1-based line number
    pub line: u32,
    /// 1-based column number (in UTF-16 code units for TS compatibility)
    pub column: u32,
}

impl LineCol {
    /// Creates a new line/column pair
    pub const fn new(line: u32, column: u32) -> Self {
        Self { line, column }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_span_new() {
        let span = Span::new(10, 20);
        assert_eq!(span.start, 10);
        assert_eq!(span.end, 20);
        assert_eq!(span.len(), 10);
    }

    #[test]
    fn test_span_empty() {
        let span = Span::empty(5);
        assert!(span.is_empty());
        assert_eq!(span.len(), 0);
    }

    #[test]
    fn test_span_extend() {
        let span1 = Span::new(10, 20);
        let span2 = Span::new(15, 30);
        let extended = span1.extend(span2);
        assert_eq!(extended.start, 10);
        assert_eq!(extended.end, 30);
    }

    #[test]
    fn test_span_contains() {
        let span = Span::new(10, 20);
        assert!(span.contains(10));
        assert!(span.contains(15));
        assert!(!span.contains(20));
        assert!(!span.contains(5));
    }

    #[test]
    fn test_span_overlaps() {
        let span1 = Span::new(10, 20);
        let span2 = Span::new(15, 25);
        let span3 = Span::new(25, 30);

        assert!(span1.overlaps(&span2));
        assert!(!span1.overlaps(&span3));
    }
}
