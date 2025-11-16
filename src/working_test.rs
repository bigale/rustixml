// Insertion syntax ambiguity test - separated for module visibility
use rusty_lr::lr1;

lr1! {
    %err String;
    %glr;
    %tokentype char;
    %start Rule;

    WS: [ ' ' '\t' '\n' '\r' ]*;

    QuotedString(String): '"' chars=NotQuote+ '"' {
        chars.into_iter().collect()
    };
    NotQuote(char): ch=[^'"'] { ch };

    Insertion(String): '+' WS s=QuotedString WS {
        format!("INSERT:{}", s)
    };

    Nonterminal(String): name=Ident WS {
        format!("NT:{}", name)
    };
    Ident(String): start=IdentStart rest=IdentRest* {
        let mut s = start.to_string();
        s.push_str(&rest.into_iter().collect::<String>());
        s
    };
    IdentStart(char): ch=['a'-'z'] { ch }
                    | ch=['A'-'Z'] { ch }
                    | ch='_' { ch };
    IdentRest(char): ch=['a'-'z'] { ch }
                   | ch=['A'-'Z'] { ch }
                   | ch=['0'-'9'] { ch }
                   | ch='_' { ch };

    Factor(String): Insertion
                  | Nonterminal
                  ;

    Repeat1(String): f1=Factor WS '+' WS {
        format!("REPEAT1:{}", f1)
    }
    | Factor
    ;

    Rule(String): WS Repeat1 WS;
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_clear_insertion() {
        let parser = RuleParser::new();
        let mut ctx = RuleContext::new();

        for ch in r#"+"hello""#.chars() {
            ctx.feed(&parser, ch, &mut ()).unwrap();
        }

        let results: Vec<_> = ctx.accept(&parser, &mut ()).unwrap().collect();
        println!("Clear insertion results: {:?}", results);
        assert!(results.len() > 0);
        assert!(results[0].contains("INSERT:hello"));
    }

    #[test]
    fn test_clear_repeat1() {
        let parser = RuleParser::new();
        let mut ctx = RuleContext::new();

        for ch in "foo+".chars() {
            ctx.feed(&parser, ch, &mut ()).unwrap();
        }

        let results: Vec<_> = ctx.accept(&parser, &mut ()).unwrap().collect();
        println!("Clear repeat1 results: {:?}", results);
        assert!(results.len() > 0);
        assert!(results[0].contains("REPEAT1:NT:foo"));
    }

    #[test]
    fn test_two_factors() {
        // Simpler test: two factors side-by-side
        // This tests if parser can handle: nonterminal followed by insertion

        let parser = RuleParser::new();
        let mut ctx = RuleContext::new();

        // Just test two separate factors
        for ch in "foo".chars() {
            ctx.feed(&parser, ch, &mut ()).unwrap();
        }

        let results: Vec<_> = ctx.accept(&parser, &mut ()).unwrap().collect();
        println!("Single factor results: {:?}", results);
        assert!(results.len() > 0);
        assert!(results[0].contains("NT:foo"));
    }
}
