//! Core type definitions for the TypeScript type system.

/// Represents a position in source code.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub struct TextSpan {
    pub start: u32,
    pub length: u32,
}

impl TextSpan {
    pub fn new(start: u32, length: u32) -> Self {
        Self { start, length }
    }

    pub fn end(&self) -> u32 {
        self.start + self.length
    }
}

/// A source file identifier.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub struct SourceFileId(pub u32);

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_text_span() {
        let span = TextSpan::new(10, 5);
        assert_eq!(span.start, 10);
        assert_eq!(span.length, 5);
        assert_eq!(span.end(), 15);
    }
}
