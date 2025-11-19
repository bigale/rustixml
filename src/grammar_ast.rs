//! Token-based iXML grammar parser that produces AST
//!
//! Now uses a handwritten recursive descent parser for fast, linear-time parsing
//! (previously used RustyLR GLR which had exponential performance issues)

// Re-export the handwritten parser's parse function
pub use crate::grammar_parser::parse_ixml_grammar;

// Keep the old RustyLR implementation commented out for reference
/*
use rusty_lr::lr1;
use crate::lexer::Token;
use crate::ast::{IxmlGrammar, Rule, Alternatives, Sequence, Factor, BaseFactor, Mark, Repetition};

lr1! {
    %err String;
    %glr;
    %tokentype Token;
    %start Grammar;

    // Map token patterns to terminal symbols
    %token ident Token::Ident(_);
    %token string Token::String(_);
    %token charclass Token::CharClass(_);
    %token hexchar Token::HexChar(_);
    %token colon Token::Colon;
    %token period Token::Period;
    %token semicolon Token::Semicolon;
    %token pipe Token::Pipe;
    %token plus Token::Plus;
    %token star Token::Star;
    %token double_star Token::DoubleStar;
    %token double_plus Token::DoublePlus;
    %token question Token::Question;
    %token at Token::At;
    %token minus Token::Minus;
    %token caret Token::Caret;
    %token tilde Token::Tilde;
    %token lparen Token::LParen;
    %token rparen Token::RParen;
    %token comma Token::Comma;

    // Base factor (without repetition operator)
    BaseFactor(BaseFactor): tok=string {
        match tok {
            Token::String(s) => BaseFactor::literal(s),
            _ => unreachable!(),
        }
    }
    | plus tok=string {
        match tok {
            Token::String(s) => BaseFactor::insertion(s),
            _ => unreachable!(),
        }
    }
    | at tok=string {
        match tok {
            Token::String(s) => BaseFactor::marked_literal(s, Mark::Attribute),
            _ => unreachable!(),
        }
    }
    | minus tok=string {
        match tok {
            Token::String(s) => BaseFactor::marked_literal(s, Mark::Hidden),
            _ => unreachable!(),
        }
    }
    | caret tok=string {
        match tok {
            Token::String(s) => BaseFactor::marked_literal(s, Mark::Promoted),
            _ => unreachable!(),
        }
    }
    | tok=hexchar {
        match tok {
            Token::HexChar(hex) => {
                // Convert hex string to character
                let code_point = u32::from_str_radix(&hex, 16)
                    .expect("Hex validation should have happened in lexer");
                let ch = char::from_u32(code_point)
                    .expect("Invalid Unicode code point should have been caught in lexer");
                BaseFactor::literal(ch.to_string())
            },
            _ => unreachable!(),
        }
    }
    | plus tok=hexchar {
        match tok {
            Token::HexChar(hex) => {
                // Convert hex string to character
                let code_point = u32::from_str_radix(&hex, 16)
                    .expect("Hex validation should have happened in lexer");
                let ch = char::from_u32(code_point)
                    .expect("Invalid Unicode code point should have been caught in lexer");
                BaseFactor::insertion(ch.to_string())
            },
            _ => unreachable!(),
        }
    }
    | at tok=hexchar {
        match tok {
            Token::HexChar(hex) => {
                let code_point = u32::from_str_radix(&hex, 16)
                    .expect("Hex validation should have happened in lexer");
                let ch = char::from_u32(code_point)
                    .expect("Invalid Unicode code point should have been caught in lexer");
                BaseFactor::marked_literal(ch.to_string(), Mark::Attribute)
            },
            _ => unreachable!(),
        }
    }
    | minus tok=hexchar {
        match tok {
            Token::HexChar(hex) => {
                let code_point = u32::from_str_radix(&hex, 16)
                    .expect("Hex validation should have happened in lexer");
                let ch = char::from_u32(code_point)
                    .expect("Invalid Unicode code point should have been caught in lexer");
                BaseFactor::marked_literal(ch.to_string(), Mark::Hidden)
            },
            _ => unreachable!(),
        }
    }
    | caret tok=hexchar {
        match tok {
            Token::HexChar(hex) => {
                let code_point = u32::from_str_radix(&hex, 16)
                    .expect("Hex validation should have happened in lexer");
                let ch = char::from_u32(code_point)
                    .expect("Invalid Unicode code point should have been caught in lexer");
                BaseFactor::marked_literal(ch.to_string(), Mark::Promoted)
            },
            _ => unreachable!(),
        }
    }
    | tok=ident {
        match tok {
            Token::Ident(name) => BaseFactor::nonterminal(name),
            _ => unreachable!(),
        }
    }
    | at tok=ident {
        match tok {
            Token::Ident(name) => BaseFactor::marked_nonterminal(name, Mark::Attribute),
            _ => unreachable!(),
        }
    }
    | minus tok=ident {
        match tok {
            Token::Ident(name) => BaseFactor::marked_nonterminal(name, Mark::Hidden),
            _ => unreachable!(),
        }
    }
    | caret tok=ident {
        match tok {
            Token::Ident(name) => BaseFactor::marked_nonterminal(name, Mark::Promoted),
            _ => unreachable!(),
        }
    }
    | lparen alts=Alternatives rparen {
        BaseFactor::group(alts)
    }
    | lparen rparen {
        BaseFactor::group(Alternatives::single(Sequence::empty()))
    }
    | tok=charclass {
        match tok {
            Token::CharClass(content) => BaseFactor::charclass(content),
            _ => unreachable!(),
        }
    }
    | tilde tok=charclass {
        match tok {
            Token::CharClass(content) => BaseFactor::negated_charclass(content),
            _ => unreachable!(),
        }
    }
    | at tok=charclass {
        match tok {
            Token::CharClass(content) => BaseFactor::marked_charclass(content, false, Mark::Attribute),
            _ => unreachable!(),
        }
    }
    | minus tok=charclass {
        match tok {
            Token::CharClass(content) => BaseFactor::marked_charclass(content, false, Mark::Hidden),
            _ => unreachable!(),
        }
    }
    | caret tok=charclass {
        match tok {
            Token::CharClass(content) => BaseFactor::marked_charclass(content, false, Mark::Promoted),
            _ => unreachable!(),
        }
    }
    | at tilde tok=charclass {
        match tok {
            Token::CharClass(content) => BaseFactor::marked_charclass(content, true, Mark::Attribute),
            _ => unreachable!(),
        }
    }
    | minus tilde tok=charclass {
        match tok {
            Token::CharClass(content) => BaseFactor::marked_charclass(content, true, Mark::Hidden),
            _ => unreachable!(),
        }
    }
    | caret tilde tok=charclass {
        match tok {
            Token::CharClass(content) => BaseFactor::marked_charclass(content, true, Mark::Promoted),
            _ => unreachable!(),
        }
    };

    // Factor with optional repetition operator
    Factor(Factor): base=BaseFactor plus {
        Factor::new(base, Repetition::OneOrMore)
    }
    | base=BaseFactor star {
        Factor::new(base, Repetition::ZeroOrMore)
    }
    | base=BaseFactor double_star lparen sep=Sequence rparen {
        Factor::new(base, Repetition::SeparatedZeroOrMore(Box::new(sep)))
    }
    | base=BaseFactor double_plus lparen sep=Sequence rparen {
        Factor::new(base, Repetition::SeparatedOneOrMore(Box::new(sep)))
    }
    | base=BaseFactor double_star sep=BaseFactor {
        // Bare separator: hash**S is equivalent to hash**(S)
        let sep_factor = Factor::simple(sep);
        let sep_seq = Sequence::new(vec![sep_factor]);
        Factor::new(base, Repetition::SeparatedZeroOrMore(Box::new(sep_seq)))
    }
    | base=BaseFactor double_plus sep=BaseFactor {
        // Bare separator: atom++dot is equivalent to atom++(dot)
        let sep_factor = Factor::simple(sep);
        let sep_seq = Sequence::new(vec![sep_factor]);
        Factor::new(base, Repetition::SeparatedOneOrMore(Box::new(sep_seq)))
    }
    | base=BaseFactor question {
        Factor::new(base, Repetition::Optional)
    }
    | base=BaseFactor {
        Factor::simple(base)
    }
    ;

    // Sequence: one or more factors (comma-separated or whitespace-separated)
    Sequence(Sequence): factors=$sep(Factor, comma, +) {
        Sequence::new(factors)
    }
    | factors=Factor+ {
        Sequence::new(factors)
    };

    // Alternatives: one or more sequences separated by pipe or semicolon
    Alternatives(Alternatives): alts=$sep(Sequence, pipe, +) {
        Alternatives::new(alts)
    }
    | alts=$sep(Sequence, semicolon, +) {
        Alternatives::new(alts)
    };

    // Rule: name: alternatives.
    Rule(Rule): name_tok=ident colon body=Alternatives period {
        match name_tok {
            Token::Ident(name) => Rule::new(name, Mark::None, body),
            _ => unreachable!(),
        }
    }
    | at name_tok=ident colon body=Alternatives period {
        match name_tok {
            Token::Ident(name) => Rule::new(name, Mark::Attribute, body),
            _ => unreachable!(),
        }
    }
    | minus name_tok=ident colon body=Alternatives period {
        match name_tok {
            Token::Ident(name) => Rule::new(name, Mark::Hidden, body),
            _ => unreachable!(),
        }
    }
    | caret name_tok=ident colon body=Alternatives period {
        match name_tok {
            Token::Ident(name) => Rule::new(name, Mark::Promoted, body),
            _ => unreachable!(),
        }
    };

    // Grammar: one or more rules
    Grammar(IxmlGrammar): rules=Rule+ {
        IxmlGrammar::new(rules)
    };
}

pub fn parse_tokens(tokens: Vec<Token>) -> Result<IxmlGrammar, String> {
    let parser = GrammarParser::new();
    let mut ctx = GrammarContext::new();

    // Feed all tokens except EOF
    for token in tokens {
        if token == Token::Eof {
            break;
        }
        ctx.feed(&parser, token, &mut ()).map_err(|e| format!("Parse error: {:?}", e))?;
    }

    // Signal end of input
    let results: Vec<_> = ctx.accept(&parser, &mut ())
        .map_err(|e| format!("Accept error: {:?}", e))?
        .collect();

    if results.is_empty() {
        Err("No parse results".to_string())
    } else {
        Ok(results[0].clone())
    }
}

pub fn parse_ixml_grammar_old(input: &str) -> Result<IxmlGrammar, String> {
    use crate::lexer::Lexer;

    let mut lexer = Lexer::new(input);
    let tokens = lexer.tokenize()?;
    parse_tokens(tokens)
}
*/

// Tests now use the new handwritten parser
#[cfg(test)]
mod tests {
    use super::*;
    use crate::ast::{BaseFactor, Mark, Repetition};

    #[test]
    fn test_rule_with_literal() {
        let input = r#"rule: "hello"."#;
        let result = parse_ixml_grammar(input);
        println!("Parse result: {:?}", result);
        assert!(result.is_ok());
        let grammar = result.unwrap();
        assert_eq!(grammar.rules.len(), 1);
        assert_eq!(grammar.rules[0].name, "rule");
    }

    #[test]
    fn test_rule_with_nonterminal() {
        let input = "rule: body.";
        let result = parse_ixml_grammar(input);
        assert!(result.is_ok());
    }

    #[test]
    fn test_rule_with_alternatives() {
        let input = r#"rule: "hello" | "world"."#;
        let result = parse_ixml_grammar(input);
        assert!(result.is_ok());
        let grammar = result.unwrap();
        assert_eq!(grammar.rules[0].alternatives.alts.len(), 2);
    }

    #[test]
    fn test_multiple_rules() {
        let input = r#"
            rule1: "hello".
            rule2: "world".
        "#;
        let result = parse_ixml_grammar(input);
        assert!(result.is_ok());
        let grammar = result.unwrap();
        assert_eq!(grammar.rules.len(), 2);
    }

    #[test]
    fn test_repetition() {
        let input = r#"list: item+."#;
        let result = parse_ixml_grammar(input);
        assert!(result.is_ok());
        let grammar = result.unwrap();
        let first_factor = &grammar.rules[0].alternatives.alts[0].factors[0];
        assert_eq!(first_factor.repetition, Repetition::OneOrMore);
    }

    #[test]
    fn test_marks() {
        let input = "element: @id -sep ^content.";
        let result = parse_ixml_grammar(input);
        assert!(result.is_ok());
        let grammar = result.unwrap();
        let factors = &grammar.rules[0].alternatives.alts[0].factors;
        match &factors[0].base {
            BaseFactor::Nonterminal { mark, .. } => assert_eq!(*mark, Mark::Attribute),
            _ => panic!("Expected nonterminal"),
        }
    }

    #[test]
    fn test_insertion() {
        let input = r#"tag: +"<"."#;
        let result = parse_ixml_grammar(input);
        assert!(result.is_ok());
        let grammar = result.unwrap();
        let factor = &grammar.rules[0].alternatives.alts[0].factors[0];
        match &factor.base {
            BaseFactor::Literal { insertion, .. } => assert!(*insertion),
            _ => panic!("Expected literal"),
        }
    }

    #[test]
    fn test_character_class() {
        let input = r#"digit: ['0'-'9']."#;
        let result = parse_ixml_grammar(input);
        assert!(result.is_ok());
    }

    #[test]
    fn test_grouping() {
        let input = r#"rule: ("a" | "b")+."#;
        let result = parse_ixml_grammar(input);
        assert!(result.is_ok());
        let grammar = result.unwrap();
        let factor = &grammar.rules[0].alternatives.alts[0].factors[0];
        assert_eq!(factor.repetition, Repetition::OneOrMore);
        match &factor.base {
            BaseFactor::Group { alternatives } => assert_eq!(alternatives.alts.len(), 2),
            _ => panic!("Expected group"),
        }
    }
}
