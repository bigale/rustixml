//! Parse context and result types for native interpreter
//!
//! Tracks parsing state during recursive descent, including rule stack
//! for left-recursion detection and parse results with consumed counts.

use crate::runtime_parser::XmlNode;
use std::collections::HashSet;

/// Context maintained during parsing for tracking and error reporting
#[derive(Debug, Clone)]
pub struct ParseContext {
    /// Current rule being parsed (for error messages)
    pub rule_name: String,
    
    /// Recursion depth (for debugging and loop detection)
    pub depth: usize,
    
    /// Rules currently on the call stack (for left-recursion detection)
    pub left_recursion: HashSet<String>,
}

impl ParseContext {
    /// Create a new parse context
    pub fn new() -> Self {
        ParseContext {
            rule_name: String::new(),
            depth: 0,
            left_recursion: HashSet::new(),
        }
    }

    /// Enter a rule (push onto recursion stack)
    pub fn enter_rule(&mut self, rule_name: &str) -> bool {
        self.depth += 1;
        self.rule_name = rule_name.to_string();
        
        // Check if we're already parsing this rule (direct left-recursion)
        if self.left_recursion.contains(rule_name) {
            return false; // Left recursion detected
        }
        
        self.left_recursion.insert(rule_name.to_string());
        true
    }

    /// Exit a rule (pop from recursion stack)
    pub fn exit_rule(&mut self, rule_name: &str) {
        self.depth = self.depth.saturating_sub(1);
        self.left_recursion.remove(rule_name);
    }
}

impl Default for ParseContext {
    fn default() -> Self {
        Self::new()
    }
}

/// Result of parsing operation
#[derive(Debug, Clone)]
pub struct ParseResult {
    /// The parsed XML node (None if suppressed with - mark)
    pub node: Option<XmlNode>,
    
    /// Number of characters consumed from input
    pub consumed: usize,
}

impl ParseResult {
    /// Create a new parse result
    pub fn new(node: Option<XmlNode>, consumed: usize) -> Self {
        ParseResult { node, consumed }
    }

    /// Create a result with no node (suppressed) but characters consumed
    pub fn suppressed(consumed: usize) -> Self {
        ParseResult {
            node: None,
            consumed,
        }
    }

    /// Create a result with a node
    pub fn with_node(node: XmlNode, consumed: usize) -> Self {
        ParseResult {
            node: Some(node),
            consumed,
        }
    }

    /// Create a result for insertion (node but no consumption)
    pub fn insertion(node: XmlNode) -> Self {
        ParseResult {
            node: Some(node),
            consumed: 0,
        }
    }
}

/// Error type for parsing failures
#[derive(Debug, Clone)]
pub enum ParseError {
    /// Unexpected end of input
    UnexpectedEof {
        position: usize,
        expected: String,
    },

    /// Terminal literal didn't match
    TerminalMismatch {
        expected: String,
        actual: String,
        position: usize,
    },

    /// Character class didn't match
    CharClassMismatch {
        charclass: String,
        negated: bool,
        actual: char,
        position: usize,
    },

    /// No alternative in choice succeeded
    NoAlternativeMatched {
        position: usize,
        rule: String,
        attempts: usize,
    },

    /// Rule not found in grammar
    UndefinedRule {
        rule: String,
        position: usize,
    },

    /// Direct left recursion detected
    LeftRecursion {
        rule: String,
        position: usize,
    },

    /// Custom error message
    Custom {
        message: String,
        position: usize,
    },
}

impl ParseError {
    /// Get the position where the error occurred
    pub fn position(&self) -> usize {
        match self {
            ParseError::UnexpectedEof { position, .. } => *position,
            ParseError::TerminalMismatch { position, .. } => *position,
            ParseError::CharClassMismatch { position, .. } => *position,
            ParseError::NoAlternativeMatched { position, .. } => *position,
            ParseError::UndefinedRule { position, .. } => *position,
            ParseError::LeftRecursion { position, .. } => *position,
            ParseError::Custom { position, .. } => *position,
        }
    }

    /// Format error with context from input
    pub fn format_with_context(&self, input: &str) -> String {
        let stream = crate::input_stream::InputStream::new(input);
        let (line, col) = stream.line_col(self.position());
        let context = stream.substring(
            self.position().saturating_sub(20),
            (self.position() + 20).min(stream.len()),
        );

        match self {
            ParseError::UnexpectedEof { expected, .. } => {
                format!(
                    "Parse error at line {}, column {}: Unexpected end of input, expected {}",
                    line, col, expected
                )
            }
            ParseError::TerminalMismatch {
                expected, actual, ..
            } => {
                format!(
                    "Parse error at line {}, column {}: Expected '{}' but found '{}'\nContext: ...{}...",
                    line, col, expected, actual, context
                )
            }
            ParseError::CharClassMismatch {
                charclass,
                negated,
                actual,
                ..
            } => {
                let neg_str = if *negated { "not " } else { "" };
                format!(
                    "Parse error at line {}, column {}: Expected {}[{}] but found '{}'\nContext: ...{}...",
                    line, col, neg_str, charclass, actual, context
                )
            }
            ParseError::NoAlternativeMatched { rule, attempts, .. } => {
                format!(
                    "Parse error at line {}, column {}: No alternative matched in rule '{}' ({} alternatives tried)\nContext: ...{}...",
                    line, col, rule, attempts, context
                )
            }
            ParseError::UndefinedRule { rule, .. } => {
                format!(
                    "Parse error at line {}, column {}: Undefined rule '{}'",
                    line, col, rule
                )
            }
            ParseError::LeftRecursion { rule, .. } => {
                format!(
                    "Parse error at line {}, column {}: Left recursion detected in rule '{}'",
                    line, col, rule
                )
            }
            ParseError::Custom { message, .. } => {
                format!(
                    "Parse error at line {}, column {}: {}\nContext: ...{}...",
                    line, col, message, context
                )
            }
        }
    }
}

impl std::fmt::Display for ParseError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            ParseError::UnexpectedEof { expected, .. } => {
                write!(f, "Unexpected EOF, expected {}", expected)
            }
            ParseError::TerminalMismatch { expected, actual, .. } => {
                write!(f, "Expected '{}' but found '{}'", expected, actual)
            }
            ParseError::CharClassMismatch {
                charclass,
                negated,
                actual,
                ..
            } => {
                let neg = if *negated { "not " } else { "" };
                write!(f, "Expected {}[{}] but found '{}'", neg, charclass, actual)
            }
            ParseError::NoAlternativeMatched { rule, attempts, .. } => {
                write!(f, "No alternative matched in '{}' ({} tried)", rule, attempts)
            }
            ParseError::UndefinedRule { rule, .. } => {
                write!(f, "Undefined rule '{}'", rule)
            }
            ParseError::LeftRecursion { rule, .. } => {
                write!(f, "Left recursion in rule '{}'", rule)
            }
            ParseError::Custom { message, .. } => write!(f, "{}", message),
        }
    }
}

impl std::error::Error for ParseError {}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_context_new() {
        let ctx = ParseContext::new();
        assert_eq!(ctx.depth, 0);
        assert!(ctx.left_recursion.is_empty());
    }

    #[test]
    fn test_enter_exit_rule() {
        let mut ctx = ParseContext::new();

        assert!(ctx.enter_rule("test"));
        assert_eq!(ctx.depth, 1);
        assert_eq!(ctx.rule_name, "test");
        assert!(ctx.left_recursion.contains("test"));

        // Direct left recursion should fail (but depth still increments)
        assert!(!ctx.enter_rule("test"));
        assert_eq!(ctx.depth, 2); // Depth incremented even though recursion detected

        ctx.exit_rule("test");
        assert_eq!(ctx.depth, 1); // Back to level 1
        
        ctx.exit_rule("test");
        assert_eq!(ctx.depth, 0);
        assert!(!ctx.left_recursion.contains("test"));
    }

    #[test]
    fn test_nested_rules() {
        let mut ctx = ParseContext::new();

        assert!(ctx.enter_rule("rule1"));
        assert!(ctx.enter_rule("rule2"));
        assert_eq!(ctx.depth, 2);
        assert!(ctx.left_recursion.contains("rule1"));
        assert!(ctx.left_recursion.contains("rule2"));

        ctx.exit_rule("rule2");
        assert_eq!(ctx.depth, 1);
        assert!(!ctx.left_recursion.contains("rule2"));

        ctx.exit_rule("rule1");
        assert_eq!(ctx.depth, 0);
    }

    #[test]
    fn test_parse_result_constructors() {
        let node = XmlNode::Text("test".to_string());

        let result1 = ParseResult::new(Some(node.clone()), 4);
        assert!(result1.node.is_some());
        assert_eq!(result1.consumed, 4);

        let result2 = ParseResult::suppressed(5);
        assert!(result2.node.is_none());
        assert_eq!(result2.consumed, 5);

        let result3 = ParseResult::with_node(node.clone(), 3);
        assert!(result3.node.is_some());
        assert_eq!(result3.consumed, 3);

        let result4 = ParseResult::insertion(node);
        assert!(result4.node.is_some());
        assert_eq!(result4.consumed, 0);
    }
}
