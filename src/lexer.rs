//! Lexer for iXML grammar syntax
//!
//! Converts input text into a stream of tokens, handling whitespace automatically.

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum Token {
    Ident(String),
    String(String),
    CharClass(String),  // Content between [ and ], e.g., "'a'-'z'" or "Ll"
    Colon,
    Period,
    Semicolon,
    Pipe,
    Plus,
    Star,
    Question,
    At,
    Minus,
    Caret,  // For promoted mark ^
    Tilde,  // For negated character classes ~[...]
    LParen,
    RParen,
    LBracket,
    RBracket,
    Comma,
    Equals,
    Eof,
}

pub struct Lexer {
    input: Vec<char>,
    pos: usize,
}

impl Lexer {
    pub fn new(input: &str) -> Self {
        Self {
            input: input.chars().collect(),
            pos: 0,
        }
    }

    pub fn tokenize(&mut self) -> Result<Vec<Token>, String> {
        let mut tokens = Vec::new();

        while self.pos < self.input.len() {
            self.skip_whitespace_and_comments()?;

            if self.pos >= self.input.len() {
                break;
            }

            let token = self.next_token()?;
            tokens.push(token);
        }

        tokens.push(Token::Eof);
        Ok(tokens)
    }

    fn skip_whitespace_and_comments(&mut self) -> Result<(), String> {
        loop {
            // Skip whitespace
            while self.pos < self.input.len() && self.input[self.pos].is_whitespace() {
                self.pos += 1;
            }

            // Check for comment start
            if self.peek() == Some('{') {
                self.skip_comment()?;
            } else {
                break;
            }
        }
        Ok(())
    }

    fn skip_comment(&mut self) -> Result<(), String> {
        // iXML comments are {like this} and can be nested
        if self.peek() != Some('{') {
            return Ok(());
        }

        self.advance(); // consume '{'
        let mut depth = 1;

        while depth > 0 && self.pos < self.input.len() {
            match self.peek() {
                Some('{') => {
                    depth += 1;
                    self.advance();
                }
                Some('}') => {
                    depth -= 1;
                    self.advance();
                }
                Some(_) => {
                    self.advance();
                }
                None => {
                    return Err("Unclosed comment".to_string());
                }
            }
        }

        if depth > 0 {
            return Err("Unclosed comment".to_string());
        }

        Ok(())
    }

    fn skip_whitespace(&mut self) {
        while self.pos < self.input.len() && self.input[self.pos].is_whitespace() {
            self.pos += 1;
        }
    }

    fn peek(&self) -> Option<char> {
        if self.pos < self.input.len() {
            Some(self.input[self.pos])
        } else {
            None
        }
    }

    fn advance(&mut self) -> Option<char> {
        if self.pos < self.input.len() {
            let ch = self.input[self.pos];
            self.pos += 1;
            Some(ch)
        } else {
            None
        }
    }

    fn next_token(&mut self) -> Result<Token, String> {
        match self.peek() {
            Some('"') => self.read_string(),
            Some('\'') => self.read_char_literal(),
            Some(':') => {
                self.advance();
                Ok(Token::Colon)
            }
            Some('.') => {
                self.advance();
                Ok(Token::Period)
            }
            Some(';') => {
                self.advance();
                Ok(Token::Semicolon)
            }
            Some('|') => {
                self.advance();
                Ok(Token::Pipe)
            }
            Some('+') => {
                self.advance();
                Ok(Token::Plus)
            }
            Some('*') => {
                self.advance();
                Ok(Token::Star)
            }
            Some('?') => {
                self.advance();
                Ok(Token::Question)
            }
            Some('@') => {
                self.advance();
                Ok(Token::At)
            }
            Some('-') => {
                self.advance();
                Ok(Token::Minus)
            }
            Some('~') => {
                self.advance();
                Ok(Token::Tilde)
            }
            Some('^') => {
                self.advance();
                Ok(Token::Caret)
            }
            Some('(') => {
                self.advance();
                Ok(Token::LParen)
            }
            Some(')') => {
                self.advance();
                Ok(Token::RParen)
            }
            Some('[') => self.read_char_class(),
            Some(']') => {
                // Standalone ] is an error (should only appear inside char class)
                Err("Unexpected ] outside character class".to_string())
            }
            Some(',') => {
                self.advance();
                Ok(Token::Comma)
            }
            Some('=') => {
                self.advance();
                Ok(Token::Equals)
            }
            Some(ch) if ch.is_alphabetic() || ch == '_' => self.read_ident(),
            Some(ch) => Err(format!("Unexpected character: {}", ch)),
            None => Err("Unexpected end of input".to_string()),
        }
    }

    fn read_string(&mut self) -> Result<Token, String> {
        self.advance(); // skip opening quote
        let mut s = String::new();

        while let Some(ch) = self.peek() {
            if ch == '"' {
                self.advance();
                // Check for escaped quote (doubled quote)
                if self.peek() == Some('"') {
                    // It's an escaped quote, add one quote to string and continue
                    s.push('"');
                    self.advance();
                } else {
                    // It's the closing quote
                    return Ok(Token::String(s));
                }
            } else {
                s.push(ch);
                self.advance();
            }
        }

        Err("Unterminated string".to_string())
    }

    fn read_char_literal(&mut self) -> Result<Token, String> {
        self.advance(); // skip opening quote
        let mut s = String::new();

        while let Some(ch) = self.peek() {
            if ch == '\'' {
                self.advance();
                // Check for escaped quote (doubled quote)
                if self.peek() == Some('\'') {
                    // It's an escaped quote, add one quote to string and continue
                    s.push('\'');
                    self.advance();
                } else {
                    // It's the closing quote
                    return Ok(Token::String(s));
                }
            } else {
                s.push(ch);
                self.advance();
            }
        }

        Err("Unterminated character literal".to_string())
    }

    fn read_ident(&mut self) -> Result<Token, String> {
        let mut ident = String::new();

        while let Some(ch) = self.peek() {
            if ch.is_alphanumeric() || ch == '_' || ch == '-' {
                ident.push(ch);
                self.advance();
            } else {
                break;
            }
        }

        Ok(Token::Ident(ident))
    }

    fn read_char_class(&mut self) -> Result<Token, String> {
        self.advance(); // skip opening bracket
        let mut content = String::new();

        while let Some(ch) = self.peek() {
            if ch == ']' {
                self.advance(); // skip closing bracket
                return Ok(Token::CharClass(content));
            }
            content.push(ch);
            self.advance();
        }

        Err("Unterminated character class".to_string())
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_simple_rule() {
        let mut lexer = Lexer::new(r#"rule: "hello"."#);
        let tokens = lexer.tokenize().unwrap();

        assert_eq!(tokens[0], Token::Ident("rule".to_string()));
        assert_eq!(tokens[1], Token::Colon);
        assert_eq!(tokens[2], Token::String("hello".to_string()));
        assert_eq!(tokens[3], Token::Period);
        assert_eq!(tokens[4], Token::Eof);
    }

    #[test]
    fn test_whitespace_handling() {
        let mut lexer = Lexer::new("rule  :   \"hello\"  .");
        let tokens = lexer.tokenize().unwrap();

        assert_eq!(tokens.len(), 5); // ident, colon, string, period, eof
    }

    #[test]
    fn test_nonterminal() {
        let mut lexer = Lexer::new("rule: body.");
        let tokens = lexer.tokenize().unwrap();

        assert_eq!(tokens[0], Token::Ident("rule".to_string()));
        assert_eq!(tokens[1], Token::Colon);
        assert_eq!(tokens[2], Token::Ident("body".to_string()));
        assert_eq!(tokens[3], Token::Period);
    }

    #[test]
    fn test_simple_comment() {
        let mut lexer = Lexer::new(r#"{This is a comment} rule: "hello"."#);
        let tokens = lexer.tokenize().unwrap();

        // Comment should be skipped
        assert_eq!(tokens[0], Token::Ident("rule".to_string()));
        assert_eq!(tokens[1], Token::Colon);
        assert_eq!(tokens[2], Token::String("hello".to_string()));
        assert_eq!(tokens[3], Token::Period);
    }

    #[test]
    fn test_nested_comments() {
        let mut lexer = Lexer::new(r#"{Outer {nested} comment} rule: "hello"."#);
        let tokens = lexer.tokenize().unwrap();

        // Nested comment should be skipped
        assert_eq!(tokens[0], Token::Ident("rule".to_string()));
        assert_eq!(tokens[1], Token::Colon);
        assert_eq!(tokens[2], Token::String("hello".to_string()));
        assert_eq!(tokens[3], Token::Period);
    }

    #[test]
    fn test_comment_between_tokens() {
        let mut lexer = Lexer::new(r#"rule {comment here} : "hello"."#);
        let tokens = lexer.tokenize().unwrap();

        assert_eq!(tokens[0], Token::Ident("rule".to_string()));
        assert_eq!(tokens[1], Token::Colon);
        assert_eq!(tokens[2], Token::String("hello".to_string()));
        assert_eq!(tokens[3], Token::Period);
    }

    #[test]
    fn test_unclosed_comment_error() {
        let mut lexer = Lexer::new(r#"{Unclosed comment rule: "hello"."#);
        let result = lexer.tokenize();

        assert!(result.is_err());
        assert_eq!(result.unwrap_err(), "Unclosed comment");
    }
}
