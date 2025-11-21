//! Handwritten recursive descent parser for iXML grammars
//!
//! This replaces the RustyLR GLR parser which had exponential performance issues
//! with complex grammars containing circular references and repetitions.

use crate::ast::{Alternatives, BaseFactor, Factor, IxmlGrammar, Mark, Repetition, Rule, Sequence};
use crate::lexer::Token;

pub struct Parser {
    tokens: Vec<Token>,
    pos: usize,
}

impl Parser {
    pub fn new(tokens: Vec<Token>) -> Self {
        Parser { tokens, pos: 0 }
    }

    fn peek(&self) -> Option<&Token> {
        self.tokens.get(self.pos)
    }

    #[allow(dead_code)]
    fn peek_ahead(&self, offset: usize) -> Option<&Token> {
        self.tokens.get(self.pos + offset)
    }

    fn consume(&mut self) -> Option<Token> {
        if self.pos < self.tokens.len() {
            let token = self.tokens[self.pos].clone();
            self.pos += 1;
            Some(token)
        } else {
            None
        }
    }

    fn expect(&mut self, description: &str) -> Result<Token, String> {
        self.consume()
            .ok_or_else(|| format!("Expected {} but reached end of input", description))
    }

    fn matches(&self, expected: &Token) -> bool {
        self.peek()
            .is_some_and(|t| std::mem::discriminant(t) == std::mem::discriminant(expected))
    }

    fn at_end(&self) -> bool {
        self.pos >= self.tokens.len()
    }

    // Helper to convert hex character code to actual character
    fn hex_to_char(hex: &str) -> Result<char, String> {
        let code_point =
            u32::from_str_radix(hex, 16).map_err(|e| format!("Invalid hex value: {}", e))?;
        char::from_u32(code_point).ok_or_else(|| format!("Invalid Unicode code point: #{}", hex))
    }

    // Grammar: Rule+
    pub fn parse_grammar(&mut self) -> Result<IxmlGrammar, String> {
        let mut rules = Vec::new();

        while !self.at_end() {
            rules.push(self.parse_rule()?);
        }

        if rules.is_empty() {
            return Err("Grammar must contain at least one rule".to_string());
        }

        Ok(IxmlGrammar::new(rules))
    }

    // Rule: [Mark] Ident ":" Alternatives "."
    fn parse_rule(&mut self) -> Result<Rule, String> {
        // Check for mark prefix
        let mark = if self.matches(&Token::At) {
            self.consume();
            Mark::Attribute
        } else if self.matches(&Token::Minus) {
            self.consume();
            Mark::Hidden
        } else if self.matches(&Token::Caret) {
            self.consume();
            Mark::Promoted
        } else {
            Mark::None
        };

        // Expect identifier
        let name = match self.expect("identifier")? {
            Token::Ident(s) => s,
            other => return Err(format!("Expected identifier, got {:?}", other)),
        };

        // Expect colon
        if !self.matches(&Token::Colon) {
            return Err(format!("Expected ':' after rule name '{}'", name));
        }
        self.consume();

        // Parse alternatives
        let alternatives = self.parse_alternatives()?;

        // Expect period
        if !self.matches(&Token::Period) {
            return Err(format!("Expected '.' at end of rule '{}'", name));
        }
        self.consume();

        Ok(Rule::new(name, mark, alternatives))
    }

    // Alternatives: Sequence ("|" | ";") Sequence*
    fn parse_alternatives(&mut self) -> Result<Alternatives, String> {
        let mut alts = vec![self.parse_sequence()?];

        // Check which separator is used (pipe or semicolon)
        while self.matches(&Token::Pipe) || self.matches(&Token::Semicolon) {
            self.consume();
            alts.push(self.parse_sequence()?);
        }

        Ok(Alternatives::new(alts))
    }

    // Sequence: Factor ("," Factor)* | Factor+ | ε (empty)
    fn parse_sequence(&mut self) -> Result<Sequence, String> {
        // Handle empty sequences (e.g., "c: ." or "statement: ...; .")
        if self.matches(&Token::Period)
            || self.matches(&Token::Pipe)
            || self.matches(&Token::Semicolon)
            || self.matches(&Token::RParen)
        {
            return Ok(Sequence::new(vec![]));
        }

        let mut factors = vec![self.parse_factor()?];

        // Check if comma-separated or whitespace-separated
        let comma_separated = self.matches(&Token::Comma);

        if comma_separated {
            while self.matches(&Token::Comma) {
                self.consume();
                factors.push(self.parse_factor()?);
            }
        } else {
            // Whitespace-separated: keep parsing factors until we hit a separator
            while !self.at_end()
                && !self.matches(&Token::Period)
                && !self.matches(&Token::Pipe)
                && !self.matches(&Token::Semicolon)
                && !self.matches(&Token::RParen)
            {
                factors.push(self.parse_factor()?);
            }
        }

        Ok(Sequence::new(factors))
    }

    // Factor: BaseFactor [Repetition]
    fn parse_factor(&mut self) -> Result<Factor, String> {
        let base = self.parse_base_factor()?;

        // Check for repetition operators
        let repetition = if self.matches(&Token::Question) {
            self.consume();
            Some(Repetition::Optional)
        } else if self.matches(&Token::DoubleStar) {
            self.consume();
            // Separator can be: **(sep) or **sep
            let sep = if self.matches(&Token::LParen) {
                self.consume();
                let s = self.parse_sequence()?;
                if !self.matches(&Token::RParen) {
                    return Err("Expected ')' after separator".to_string());
                }
                self.consume();
                s
            } else {
                // Parse a single factor and wrap in a sequence
                let factor = self.parse_base_factor()?;
                Sequence::new(vec![Factor::simple(factor)])
            };
            Some(Repetition::SeparatedZeroOrMore(Box::new(sep)))
        } else if self.matches(&Token::DoublePlus) {
            self.consume();
            // Separator can be: ++(sep) or ++sep
            let sep = if self.matches(&Token::LParen) {
                self.consume();
                let s = self.parse_sequence()?;
                if !self.matches(&Token::RParen) {
                    return Err("Expected ')' after separator".to_string());
                }
                self.consume();
                s
            } else {
                // Parse a single factor and wrap in a sequence
                let factor = self.parse_base_factor()?;
                Sequence::new(vec![Factor::simple(factor)])
            };
            Some(Repetition::SeparatedOneOrMore(Box::new(sep)))
        } else if self.matches(&Token::Star) {
            self.consume();
            Some(Repetition::ZeroOrMore)
        } else if self.matches(&Token::Plus) {
            self.consume();
            Some(Repetition::OneOrMore)
        } else {
            None
        };

        Ok(if let Some(rep) = repetition {
            Factor::new(base, rep)
        } else {
            Factor::simple(base)
        })
    }

    // BaseFactor: [Mark] (Ident | String | CharClass | HexChar | "(" Alternatives ")")
    fn parse_base_factor(&mut self) -> Result<BaseFactor, String> {
        // Check for mark prefix on literals
        if self.matches(&Token::At) || self.matches(&Token::Minus) || self.matches(&Token::Caret) {
            let mark = if self.matches(&Token::At) {
                self.consume();
                Mark::Attribute
            } else if self.matches(&Token::Minus) {
                self.consume();
                Mark::Hidden
            } else {
                self.consume(); // Caret
                Mark::Promoted
            };

            // After mark, expect string, hexchar, charclass, or identifier
            match self.peek() {
                Some(Token::String(s)) => {
                    let s = s.clone();
                    self.consume();
                    Ok(BaseFactor::marked_literal(s, mark))
                }
                Some(Token::HexChar(h)) => {
                    let hex_str = h.clone();
                    self.consume();
                    let ch = Self::hex_to_char(&hex_str)?;
                    Ok(BaseFactor::marked_literal(ch.to_string(), mark))
                }
                Some(Token::CharClass(s)) => {
                    let s = s.clone();
                    self.consume();
                    Ok(BaseFactor::marked_charclass(s, false, mark))
                }
                Some(Token::Ident(s)) => {
                    let s = s.clone();
                    self.consume();
                    Ok(BaseFactor::marked_nonterminal(s, mark))
                }
                other => Err(format!("Expected string, hex char, character class, or identifier after mark, got {:?}", other)),
            }
        } else if self.matches(&Token::Plus) {
            // Insertion: +string
            self.consume();
            match self.expect("string after '+'")? {
                Token::String(s) => Ok(BaseFactor::insertion(s)),
                other => Err(format!("Expected string after '+', got {:?}", other)),
            }
        } else if self.matches(&Token::Tilde) {
            // Exclusion: ~[charclass]
            self.consume();
            match self.expect("character class after '~'")? {
                Token::CharClass(s) => Ok(BaseFactor::negated_charclass(s)),
                other => Err(format!(
                    "Expected character class after '~', got {:?}",
                    other
                )),
            }
        } else {
            // No mark prefix
            match self.peek() {
                Some(Token::Ident(s)) => {
                    let s = s.clone();
                    self.consume();
                    Ok(BaseFactor::nonterminal(s))
                }
                Some(Token::String(s)) => {
                    let s = s.clone();
                    self.consume();
                    Ok(BaseFactor::literal(s))
                }
                Some(Token::CharClass(s)) => {
                    let s = s.clone();
                    self.consume();
                    Ok(BaseFactor::charclass(s))
                }
                Some(Token::HexChar(h)) => {
                    let hex_str = h.clone();
                    self.consume();
                    let ch = Self::hex_to_char(&hex_str)?;
                    Ok(BaseFactor::literal(ch.to_string()))
                }
                Some(Token::LParen) => {
                    self.consume();
                    let alts = self.parse_alternatives()?;
                    if !self.matches(&Token::RParen) {
                        return Err("Expected ')' after grouped alternatives".to_string());
                    }
                    self.consume();
                    Ok(BaseFactor::group(alts))
                }
                other => Err(format!("Expected factor, got {:?}", other)),
            }
        }
    }
}

/// Parse an iXML grammar from a string
pub fn parse_ixml_grammar(input: &str) -> Result<IxmlGrammar, String> {
    use crate::lexer::Lexer;

    // Tokenize
    let mut lexer = Lexer::new(input);
    let tokens = lexer
        .tokenize()
        .map_err(|e| format!("Lexer error: {}", e))?;

    // Filter out EOF token
    let tokens: Vec<Token> = tokens
        .into_iter()
        .filter(|t| !matches!(t, Token::Eof))
        .collect();

    // Parse
    let mut parser = Parser::new(tokens);
    parser.parse_grammar()
}
