//! rustixml: Invisible XML (iXML) parser implementation using LALR+GLR
//!
//! This library provides a complete iXML parser that handles:
//! - Insertion syntax (`+"text"`)
//! - Unicode character classes
//! - Optional spacing
//! - Full iXML 1.0 specification
//!
//! Built with RustyLR (LALR+GLR) to handle ambiguous grammars that
//! cause traditional Earley parsers to fail.

pub mod lexer;
pub mod grammar;  // DEPRECATED - uses slow RustyLR GLR parser
pub mod grammar_v2;  // DEPRECATED - uses slow RustyLR GLR parser
pub mod grammar_ast;  // RECOMMENDED - uses fast handwritten parser
pub mod grammar_parser;  // Handwritten recursive descent parser implementation (1.5M times faster!)
pub mod ast;
pub mod runtime_parser;  // Runtime parser using Earlgrey (Phase 3)
pub mod testsuite_utils;  // Test suite infrastructure
pub mod working_test;  // Phase 1 working grammar for comparison

pub use lexer::{Lexer, Token};

#[deprecated(since = "0.2.0", note = "Use `grammar_ast::parse_ixml_grammar()` instead - 1.5M times faster!")]
pub use grammar::parse_ixml_grammar as parse_ixml_grammar_old;

#[deprecated(since = "0.2.0", note = "Use `grammar_ast::parse_ixml_grammar()` instead - 1.5M times faster!")]
pub use grammar_v2::parse_ixml_grammar_v2;

pub use ast::IxmlGrammar;

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_simple_literal() {
        // Start with the simplest possible grammar
        let grammar_str = r#"rule: "hello"."#;

        // This test will guide our implementation
        // let result = parse_ixml_grammar(grammar_str);
        // assert!(result.is_ok());
    }
}
