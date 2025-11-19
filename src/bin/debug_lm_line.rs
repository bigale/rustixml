/// Minimal test for the Lm line from unicode-classes
use rustixml::grammar_ast::parse_ixml_grammar;
use rustixml::runtime_parser::ast_to_earlgrey;
use earlgrey::EarleyParser;

fn main() {
    // Test 1: Just the Lm rule with a simple input
    let grammar1 = r#"
        Lm: -"Lm ", [Lm]*.
    "#;
    
    println!("=== Test 1: Just Lm rule ===");
    test_parse(grammar1, "Lm ʰ");
    
    // Test 2: Add the line wrapper (simplified)
    let grammar2 = r#"
        line: Lm, -#a.
        Lm: -"Lm ", [Lm]*.
    "#;
    
    println!("\n=== Test 2: With line wrapper ===");
    test_parse(grammar2, "Lm ʰ\n");
    
    // Test 3: Full alternation structure (simplified to just 3 options)
    let grammar3 = r#"
        classes: line+.
        -line: (C; L; Lm), -#a.
        C: -"C ", [C]*.
        L: -"L ", [L]*.
        Lm: -"Lm ", [Lm]*.
    "#;
    
    println!("\n=== Test 3: Multiple line alternatives ===");
    test_parse(grammar3, "Lm ʰ\n");
    
    // Test 4: Full 41-option alternation like actual unicode-classes
    let grammar4 = r#"
        classes: line+.
        -line: ( C; Cc; Cf; Cn; Co; Cs; L; LC; Ll; Lm; Lo; Lt; Lu; M; Mc; Me; Mn; N; Nd; Nl; No; P; Pc; Pd; Pe; Pf; Pi; Po; Ps; S; Sc; Sk; Sm; So; Z; Zl; Zp; Zs), -#a.
        C: -"C ", [C]*.
        Cc: -"Cc ", [Cc]*.
        Cf: -"Cf ", [Cf]*.
        Cn: -"Cn ", [Cn]*.
        Co: -"Co ", [Co]*.
        Cs: -"Cs ", [Cs]*.
        L: -"L ", [L]*.
        LC: -"LC ", [LC]*.
        Ll: -"Ll ", [Ll]*.
        Lm: -"Lm ", [Lm]*.
        Lo: -"Lo ", [Lo]*.
        Lt: -"Lt ", [Lt]*.
        Lu: -"Lu ", [Lu]*.
        M: -"M ", [M]*.
        Mc: -"Mc ", [Mc]*.
        Me: -"Me ", [Me]*.
        Mn: -"Mn ", [Mn]*.
        N: -"N ", [N]*.
        Nd: -"Nd ", [Nd]*.
        Nl: -"Nl ", [Nl]*.
        No: -"No ", [No]*.
        P: -"P ", [P]*.
        Pc: -"Pc ", [Pc]*.
        Pd: -"Pd ", [Pd]*.
        Pe: -"Pe ", [Pe]*.
        Pf: -"Pf ", [Pf]*.
        Pi: -"Pi ", [Pi]*.
        Po: -"Po ", [Po]*.
        Ps: -"Ps ", [Ps]*.
        S: -"S ", [S]*.
        Sc: -"Sc ", [Sc]*.
        Sk: -"Sk ", [Sk]*.
        Sm: -"Sm ", [Sm]*.
        So: -"So ", [So]*.
        Z: -"Z ", [Z]*.
        Zl: -"Zl ", [Zl]*.
        Zp: -"Zp ", [Zp]*.
        Zs: -"Zs ", [Zs]*.
    "#;
    
    println!("\n=== Test 4: Full 41-option alternation ===");
    test_parse(grammar4, "Lm ʰ\n");
}

fn test_parse(grammar_text: &str, input: &str) {
    println!("Grammar:\n{}", grammar_text.lines().take(3).collect::<Vec<_>>().join("\n"));
    println!("... ({} chars total)", grammar_text.len());
    println!("Input: {:?} ({} chars)", &input[..input.len().min(20)], input.len());
    
    let ast = match parse_ixml_grammar(grammar_text) {
        Ok(ast) => {
            println!("✓ Grammar parsed ({} rules)", ast.rules.len());
            ast
        }
        Err(e) => {
            println!("✗ Grammar parse failed: {}", e);
            return;
        }
    };
    
    let (builder, _transformed) = match ast_to_earlgrey(&ast) {
        Ok(result) => {
            println!("✓ Converted to Earley");
            result
        }
        Err(e) => {
            println!("✗ Conversion failed: {}", e);
            return;
        }
    };
    
    let start = &ast.rules[0].name;
    let grammar = match builder.into_grammar(start) {
        Ok(g) => {
            println!("✓ Grammar built");
            g
        }
        Err(e) => {
            println!("✗ Grammar build failed: {:?}", e);
            return;
        }
    };
    
    let parser = EarleyParser::new(grammar);
    let tokens: Vec<String> = input.chars().map(|c| c.to_string()).collect();
    println!("Tokens: {} ({:?})", tokens.len(), &tokens[..tokens.len().min(10)]);
    
    match parser.parse(tokens.iter().map(|s| s.as_str())) {
        Ok(_) => {
            println!("✓ PARSE SUCCESS!");
        }
        Err(e) => {
            println!("✗ Parse error: {:?}", e);
        }
    }
}
