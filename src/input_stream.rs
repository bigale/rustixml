//! Input stream with position tracking and backtracking support
//!
//! Manages input text as a sequence of Unicode characters, providing efficient
//! random access and position management for recursive descent parsing.

use std::fmt;

/// Input stream that tracks position in text for parsing with backtracking
#[derive(Clone)]
pub struct InputStream {
    chars: Vec<char>,
    position: usize,
}

impl InputStream {
    /// Create a new input stream from a string
    pub fn new(input: &str) -> Self {
        InputStream {
            chars: input.chars().collect(),
            position: 0,
        }
    }

    /// Get the current character without advancing
    pub fn current(&self) -> Option<char> {
        self.chars.get(self.position).copied()
    }

    /// Get the current character and advance position
    pub fn advance(&mut self) -> Option<char> {
        let ch = self.current();
        if ch.is_some() {
            self.position += 1;
        }
        ch
    }

    /// Look ahead at a character at offset from current position
    pub fn peek(&self, offset: usize) -> Option<char> {
        self.chars.get(self.position + offset).copied()
    }

    /// Get current position (character index, not byte offset)
    pub fn position(&self) -> usize {
        self.position
    }

    /// Set position (for backtracking)
    pub fn set_position(&mut self, pos: usize) {
        self.position = pos.min(self.chars.len());
    }

    /// Get remaining input as a string slice (for debugging)
    pub fn remaining(&self) -> String {
        self.chars[self.position..].iter().collect()
    }

    /// Check if at end of input
    pub fn is_eof(&self) -> bool {
        self.position >= self.chars.len()
    }

    /// Get total length in characters
    pub fn len(&self) -> usize {
        self.chars.len()
    }

    /// Check if input is empty
    pub fn is_empty(&self) -> bool {
        self.chars.is_empty()
    }

    /// Get a substring from start to end positions
    pub fn substring(&self, start: usize, end: usize) -> String {
        self.chars[start.min(self.chars.len())..end.min(self.chars.len())]
            .iter()
            .collect()
    }

    /// Get line and column for a position (for error messages)
    pub fn line_col(&self, pos: usize) -> (usize, usize) {
        let mut line = 1;
        let mut col = 1;

        for (i, ch) in self.chars.iter().enumerate() {
            if i >= pos {
                break;
            }
            if *ch == '\n' {
                line += 1;
                col = 1;
            } else {
                col += 1;
            }
        }

        (line, col)
    }
}

impl fmt::Debug for InputStream {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(
            f,
            "InputStream(pos={}, remaining={:?})",
            self.position,
            self.remaining().chars().take(20).collect::<String>()
        )
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_new() {
        let stream = InputStream::new("hello");
        assert_eq!(stream.len(), 5);
        assert_eq!(stream.position(), 0);
        assert!(!stream.is_eof());
    }

    #[test]
    fn test_current_and_advance() {
        let mut stream = InputStream::new("abc");
        assert_eq!(stream.current(), Some('a'));
        assert_eq!(stream.position(), 0);

        assert_eq!(stream.advance(), Some('a'));
        assert_eq!(stream.position(), 1);
        assert_eq!(stream.current(), Some('b'));

        assert_eq!(stream.advance(), Some('b'));
        assert_eq!(stream.advance(), Some('c'));
        assert_eq!(stream.advance(), None);
        assert!(stream.is_eof());
    }

    #[test]
    fn test_peek() {
        let stream = InputStream::new("hello");
        assert_eq!(stream.peek(0), Some('h'));
        assert_eq!(stream.peek(1), Some('e'));
        assert_eq!(stream.peek(4), Some('o'));
        assert_eq!(stream.peek(5), None);
    }

    #[test]
    fn test_backtracking() {
        let mut stream = InputStream::new("test");
        stream.advance();
        stream.advance();
        assert_eq!(stream.position(), 2);
        assert_eq!(stream.current(), Some('s'));

        stream.set_position(0);
        assert_eq!(stream.position(), 0);
        assert_eq!(stream.current(), Some('t'));
    }

    #[test]
    fn test_unicode() {
        let mut stream = InputStream::new("Hello 世界");
        assert_eq!(stream.len(), 8); // 6 ASCII + 2 Unicode chars

        for _ in 0..6 {
            stream.advance();
        }
        assert_eq!(stream.current(), Some('世'));
        stream.advance();
        assert_eq!(stream.current(), Some('界'));
    }

    #[test]
    fn test_remaining() {
        let mut stream = InputStream::new("hello");
        assert_eq!(stream.remaining(), "hello");

        stream.advance();
        stream.advance();
        assert_eq!(stream.remaining(), "llo");

        while stream.advance().is_some() {}
        assert_eq!(stream.remaining(), "");
    }

    #[test]
    fn test_substring() {
        let stream = InputStream::new("hello world");
        assert_eq!(stream.substring(0, 5), "hello");
        assert_eq!(stream.substring(6, 11), "world");
        assert_eq!(stream.substring(0, 100), "hello world");
    }

    #[test]
    fn test_line_col() {
        let stream = InputStream::new("line1\nline2\nline3");
        assert_eq!(stream.line_col(0), (1, 1));
        assert_eq!(stream.line_col(4), (1, 5));
        assert_eq!(stream.line_col(6), (2, 1));
        assert_eq!(stream.line_col(12), (3, 1));
    }

    #[test]
    fn test_empty() {
        let stream = InputStream::new("");
        assert!(stream.is_empty());
        assert!(stream.is_eof());
        assert_eq!(stream.current(), None);
    }
}
