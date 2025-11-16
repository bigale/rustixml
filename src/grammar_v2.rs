//! Token-based iXML grammar parser using RustyLR

use rusty_lr::lr1;
use crate::lexer::Token;

lr1! {
    %err String;
    %glr;
    %tokentype Token;
    %start Grammar;

    // Map token patterns to terminal symbols
    %token ident Token::Ident(_);
    %token string Token::String(_);
    %token charclass Token::CharClass(_);
    %token colon Token::Colon;
    %token period Token::Period;
    %token semicolon Token::Semicolon;
    %token pipe Token::Pipe;
    %token plus Token::Plus;
    %token star Token::Star;
    %token question Token::Question;
    %token at Token::At;
    %token minus Token::Minus;
    %token caret Token::Caret;
    %token tilde Token::Tilde;
    %token lparen Token::LParen;
    %token rparen Token::RParen;

    // Forward declare Alternatives for recursion

    // Base factor (without repetition operator)
    BaseFactor(String): tok=string {
        match tok {
            Token::String(s) => format!("LIT:{}", s),
            _ => unreachable!(),
        }
    }
    | plus tok=string {
        match tok {
            Token::String(s) => format!("+LIT:{}", s),
            _ => unreachable!(),
        }
    }
    | tok=ident {
        match tok {
            Token::Ident(name) => format!("NT:{}", name),
            _ => unreachable!(),
        }
    }
    | at tok=ident {
        match tok {
            Token::Ident(name) => format!("@NT:{}", name),
            _ => unreachable!(),
        }
    }
    | minus tok=ident {
        match tok {
            Token::Ident(name) => format!("-NT:{}", name),
            _ => unreachable!(),
        }
    }
    | caret tok=ident {
        match tok {
            Token::Ident(name) => format!("^NT:{}", name),
            _ => unreachable!(),
        }
    }
    | lparen alts=Alternatives rparen {
        format!("({})", alts)
    }
    | tok=charclass {
        match tok {
            Token::CharClass(content) => format!("CHARCLASS:[{}]", content),
            _ => unreachable!(),
        }
    }
    | tilde tok=charclass {
        match tok {
            Token::CharClass(content) => format!("~CHARCLASS:[{}]", content),
            _ => unreachable!(),
        }
    };

    // Factor with optional repetition operator
    Factor(String): base=BaseFactor plus {
        format!("{}+", base)
    }
    | base=BaseFactor star {
        format!("{}*", base)
    }
    | base=BaseFactor question {
        format!("{}?", base)
    }
    | BaseFactor  // No repetition operator
    ;

    // Sequence: one or more factors
    Sequence(String): factors=Factor+ {
        factors.join(" ")
    };

    // Alternatives: one or more sequences separated by pipe
    Alternatives(String): alts=$sep(Sequence, pipe, +) {
        let formatted_alts: Vec<String> = alts.iter()
            .map(|alt| format!("ALT[{}]", alt))
            .collect();
        format!("ALTS({})", formatted_alts.join(" | "))
    };

    // Rule: name: alternatives.
    Rule(String): name_tok=ident colon body=Alternatives period {
        match name_tok {
            Token::Ident(name) => format!("RULE:{}={}", name, body),
            _ => unreachable!(),
        }
    };

    // Grammar: one or more rules
    Grammar(String): rules=Rule+ {
        format!("GRAMMAR[\n  {}\n]", rules.join("\n  "))
    };
}

pub fn parse_tokens(tokens: Vec<Token>) -> Result<String, String> {
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

pub fn parse_ixml_grammar_v2(input: &str) -> Result<String, String> {
    use crate::lexer::Lexer;

    let mut lexer = Lexer::new(input);
    let tokens = lexer.tokenize()?;
    parse_tokens(tokens)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_rule_with_literal() {
        let input = r#"rule: "hello"."#;
        let result = parse_ixml_grammar_v2(input);
        println!("Parse result: {:?}", result);
        assert!(result.is_ok());
        let output = result.unwrap();
        assert!(output.contains("RULE:rule"));
        assert!(output.contains("LIT:hello"));
    }

    #[test]
    fn test_rule_with_nonterminal() {
        let input = "rule: body.";
        let result = parse_ixml_grammar_v2(input);
        println!("Parse result: {:?}", result);
        assert!(result.is_ok());
        let output = result.unwrap();
        assert!(output.contains("RULE:rule"));
        assert!(output.contains("NT:body"));
    }

    #[test]
    fn test_rule_with_whitespace() {
        let input = r#"rule  :  "hello"  ."#;
        let result = parse_ixml_grammar_v2(input);
        println!("Parse result: {:?}", result);
        assert!(result.is_ok());
        let output = result.unwrap();
        assert!(output.contains("RULE:rule"));
        assert!(output.contains("LIT:hello"));
    }

    #[test]
    fn test_rule_with_multiple_factors() {
        let input = "rule: foo bar.";
        let result = parse_ixml_grammar_v2(input);
        println!("Parse result: {:?}", result);
        assert!(result.is_ok());
        let output = result.unwrap();
        assert!(output.contains("RULE:rule"));
        assert!(output.contains("NT:foo"));
        assert!(output.contains("NT:bar"));
    }

    #[test]
    fn test_rule_with_mixed_factors() {
        let input = r#"rule: "hello" world "there"."#;
        let result = parse_ixml_grammar_v2(input);
        println!("Parse result: {:?}", result);
        assert!(result.is_ok());
        let output = result.unwrap();
        assert!(output.contains("RULE:rule"));
        assert!(output.contains("LIT:hello"));
        assert!(output.contains("NT:world"));
        assert!(output.contains("LIT:there"));
    }

    #[test]
    fn test_rule_with_alternatives() {
        let input = r#"rule: "hello" | "world"."#;
        let result = parse_ixml_grammar_v2(input);
        println!("Parse result: {:?}", result);
        assert!(result.is_ok());
        let output = result.unwrap();
        assert!(output.contains("RULE:rule"));
        assert!(output.contains("LIT:hello"));
        assert!(output.contains("LIT:world"));
        assert!(output.contains("|"));
    }

    #[test]
    fn test_rule_with_multiple_alternatives() {
        let input = "rule: foo | bar | baz.";
        let result = parse_ixml_grammar_v2(input);
        println!("Parse result: {:?}", result);
        assert!(result.is_ok());
        let output = result.unwrap();
        assert!(output.contains("RULE:rule"));
        assert!(output.contains("NT:foo"));
        assert!(output.contains("NT:bar"));
        assert!(output.contains("NT:baz"));
    }

    #[test]
    fn test_multiple_rules() {
        let input = r#"
            rule1: "hello".
            rule2: "world".
        "#;
        let result = parse_ixml_grammar_v2(input);
        println!("Parse result: {:?}", result);
        assert!(result.is_ok());
        let output = result.unwrap();
        assert!(output.contains("GRAMMAR"));
        assert!(output.contains("RULE:rule1"));
        assert!(output.contains("RULE:rule2"));
        assert!(output.contains("LIT:hello"));
        assert!(output.contains("LIT:world"));
    }

    #[test]
    fn test_grammar_with_references() {
        let input = r#"
            expr: term.
            term: "number" | "variable".
        "#;
        let result = parse_ixml_grammar_v2(input);
        println!("Parse result: {:?}", result);
        assert!(result.is_ok());
        let output = result.unwrap();
        assert!(output.contains("RULE:expr"));
        assert!(output.contains("RULE:term"));
        assert!(output.contains("NT:term"));
        assert!(output.contains("LIT:number"));
        assert!(output.contains("LIT:variable"));
    }

    #[test]
    fn test_repetition_plus() {
        let input = r#"list: item+."#;
        let result = parse_ixml_grammar_v2(input);
        println!("Parse result: {:?}", result);
        assert!(result.is_ok());
        let output = result.unwrap();
        assert!(output.contains("RULE:list"));
        assert!(output.contains("NT:item"));
        assert!(output.contains("+"));
    }

    #[test]
    fn test_repetition_star() {
        let input = r#"list: item*."#;
        let result = parse_ixml_grammar_v2(input);
        println!("Parse result: {:?}", result);
        assert!(result.is_ok());
        let output = result.unwrap();
        assert!(output.contains("RULE:list"));
        assert!(output.contains("NT:item"));
        assert!(output.contains("*"));
    }

    #[test]
    fn test_repetition_question() {
        let input = r#"optional: item?."#;
        let result = parse_ixml_grammar_v2(input);
        println!("Parse result: {:?}", result);
        assert!(result.is_ok());
        let output = result.unwrap();
        assert!(output.contains("RULE:optional"));
        assert!(output.contains("NT:item"));
        assert!(output.contains("?"));
    }

    #[test]
    fn test_mixed_repetitions() {
        let input = r#"expr: term+ op? value*."#;
        let result = parse_ixml_grammar_v2(input);
        println!("Parse result: {:?}", result);
        assert!(result.is_ok());
        let output = result.unwrap();
        assert!(output.contains("RULE:expr"));
        assert!(output.contains("NT:term+"));
        assert!(output.contains("NT:op?"));
        assert!(output.contains("NT:value*"));
    }

    #[test]
    fn test_parentheses_simple() {
        let input = r#"rule: ("a" | "b")."#;
        let result = parse_ixml_grammar_v2(input);
        println!("Parse result: {:?}", result);
        assert!(result.is_ok());
        let output = result.unwrap();
        assert!(output.contains("RULE:rule"));
        assert!(output.contains("("));
        assert!(output.contains(")"));
        assert!(output.contains("LIT:a"));
        assert!(output.contains("LIT:b"));
    }

    #[test]
    fn test_parentheses_with_repetition() {
        let input = r#"list: ("a" | "b")+."#;
        let result = parse_ixml_grammar_v2(input);
        println!("Parse result: {:?}", result);
        assert!(result.is_ok());
        let output = result.unwrap();
        assert!(output.contains("RULE:list"));
        assert!(output.contains("("));
        assert!(output.contains(")+"));
    }

    #[test]
    fn test_nested_parentheses() {
        let input = r#"expr: (a (b | c))."#;
        let result = parse_ixml_grammar_v2(input);
        println!("Parse result: {:?}", result);
        assert!(result.is_ok());
        let output = result.unwrap();
        assert!(output.contains("RULE:expr"));
        assert!(output.contains("NT:a"));
        assert!(output.contains("NT:b"));
        assert!(output.contains("NT:c"));
    }

    #[test]
    fn test_complex_expression() {
        let input = r#"expr: term (("+" | "-") term)*."#;
        let result = parse_ixml_grammar_v2(input);
        println!("Parse result: {:?}", result);
        assert!(result.is_ok());
        let output = result.unwrap();
        assert!(output.contains("RULE:expr"));
        assert!(output.contains("NT:term"));
        assert!(output.contains("LIT:+"));
        assert!(output.contains("LIT:-"));
    }

    #[test]
    fn test_simple_character_class() {
        let input = r#"digit: ['0'-'9']."#;
        let result = parse_ixml_grammar_v2(input);
        println!("Parse result: {:?}", result);
        assert!(result.is_ok());
        let output = result.unwrap();
        assert!(output.contains("RULE:digit"));
        assert!(output.contains("CHARCLASS:['0'-'9']"));
    }

    #[test]
    fn test_unicode_category_class() {
        let input = "letter: [L].";
        let result = parse_ixml_grammar_v2(input);
        println!("Parse result: {:?}", result);
        assert!(result.is_ok());
        let output = result.unwrap();
        assert!(output.contains("RULE:letter"));
        assert!(output.contains("CHARCLASS:[L]"));
    }

    #[test]
    fn test_negated_character_class() {
        let input = r#"nondigit: ~['0'-'9']."#;
        let result = parse_ixml_grammar_v2(input);
        println!("Parse result: {:?}", result);
        assert!(result.is_ok());
        let output = result.unwrap();
        assert!(output.contains("RULE:nondigit"));
        assert!(output.contains("~CHARCLASS:['0'-'9']"));
    }

    #[test]
    fn test_character_class_with_repetition() {
        let input = "word: [a-z]+.";
        let result = parse_ixml_grammar_v2(input);
        println!("Parse result: {:?}", result);
        assert!(result.is_ok());
        let output = result.unwrap();
        assert!(output.contains("RULE:word"));
        assert!(output.contains("CHARCLASS:[a-z]"));
        assert!(output.contains("+"));
    }

    #[test]
    fn test_mixed_factors_with_charclass() {
        let input = r#"identifier: [a-z] [a-z0-9]* "suffix"."#;
        let result = parse_ixml_grammar_v2(input);
        println!("Parse result: {:?}", result);
        assert!(result.is_ok());
        let output = result.unwrap();
        assert!(output.contains("RULE:identifier"));
        assert!(output.contains("CHARCLASS:[a-z]"));
        assert!(output.contains("CHARCLASS:[a-z0-9]"));
        assert!(output.contains("LIT:suffix"));
    }

    #[test]
    fn test_attribute_mark() {
        let input = "element: @id value.";
        let result = parse_ixml_grammar_v2(input);
        println!("Parse result: {:?}", result);
        assert!(result.is_ok());
        let output = result.unwrap();
        assert!(output.contains("RULE:element"));
        assert!(output.contains("@NT:id"));
        assert!(output.contains("NT:value"));
    }

    #[test]
    fn test_hidden_mark() {
        let input = "expr: term -sep term.";
        let result = parse_ixml_grammar_v2(input);
        println!("Parse result: {:?}", result);
        assert!(result.is_ok());
        let output = result.unwrap();
        assert!(output.contains("RULE:expr"));
        assert!(output.contains("NT:term"));
        assert!(output.contains("-NT:sep"));
    }

    #[test]
    fn test_promoted_mark() {
        let input = "wrapper: ^content.";
        let result = parse_ixml_grammar_v2(input);
        println!("Parse result: {:?}", result);
        assert!(result.is_ok());
        let output = result.unwrap();
        assert!(output.contains("RULE:wrapper"));
        assert!(output.contains("^NT:content"));
    }

    #[test]
    fn test_mixed_marks() {
        let input = "record: @id ^name -separator.";
        let result = parse_ixml_grammar_v2(input);
        println!("Parse result: {:?}", result);
        assert!(result.is_ok());
        let output = result.unwrap();
        assert!(output.contains("RULE:record"));
        assert!(output.contains("@NT:id"));
        assert!(output.contains("^NT:name"));
        assert!(output.contains("-NT:separator"));
    }

    #[test]
    fn test_marks_with_repetition() {
        let input = "list: @item+.";
        let result = parse_ixml_grammar_v2(input);
        println!("Parse result: {:?}", result);
        assert!(result.is_ok());
        let output = result.unwrap();
        assert!(output.contains("RULE:list"));
        assert!(output.contains("@NT:item"));
        assert!(output.contains("+"));
    }

    #[test]
    fn test_insertion_syntax() {
        let input = r#"tag: +"<"."#;
        let result = parse_ixml_grammar_v2(input);
        println!("Parse result: {:?}", result);
        assert!(result.is_ok());
        let output = result.unwrap();
        assert!(output.contains("RULE:tag"));
        assert!(output.contains("+LIT:<"));
    }

    #[test]
    fn test_insertion_with_regular_literal() {
        let input = r#"element: +"<" name +">"."#;
        let result = parse_ixml_grammar_v2(input);
        println!("Parse result: {:?}", result);
        assert!(result.is_ok());
        let output = result.unwrap();
        assert!(output.contains("RULE:element"));
        assert!(output.contains("+LIT:<"));
        assert!(output.contains("NT:name"));
        assert!(output.contains("+LIT:>"));
    }

    #[test]
    fn test_insertion_vs_repetition() {
        let input = r#"test: item+ +"inserted"."#;
        let result = parse_ixml_grammar_v2(input);
        println!("Parse result: {:?}", result);
        assert!(result.is_ok());
        let output = result.unwrap();
        assert!(output.contains("RULE:test"));
        assert!(output.contains("NT:item+"));  // item with repetition
        assert!(output.contains("+LIT:inserted"));  // inserted literal
    }

    #[test]
    fn test_complex_with_all_features() {
        let input = r#"xml: +"<" @name +">" ^content +"</" -endtag +">"."#;
        let result = parse_ixml_grammar_v2(input);
        println!("Parse result: {:?}", result);
        assert!(result.is_ok());
        let output = result.unwrap();
        assert!(output.contains("RULE:xml"));
        assert!(output.contains("+LIT:<"));
        assert!(output.contains("@NT:name"));
        assert!(output.contains("+LIT:>"));
        assert!(output.contains("^NT:content"));
        assert!(output.contains("+LIT:</"));
        assert!(output.contains("-NT:endtag"));
    }
}
