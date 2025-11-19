/// Test the ACTUAL unicode-classes grammar
use rustixml::grammar_ast::parse_ixml_grammar;
use rustixml::runtime_parser::ast_to_earlgrey;
use earlgrey::EarleyParser;

fn main() {
    // First let's test just the newline rule
    println!("=== Test 1: Just the newline rule ===");
    let grammar1 = r#"
        newline: (-#a; -#d)+.
    "#;
    test_parse(grammar1, "\n");
    
    // Test with single line ending
    println!("\n=== Test 2: Single Lm line with simple newline ===");
    let grammar2 = r#"
        classes: line+.
        -line: Lm, -#a.
        Lm: -"Lm ", [Lm]*.
    "#;
    test_parse(grammar2, "Lm ʰ\n");
    
    // Test with the actual newline rule
    println!("\n=== Test 3: Single Lm line with complex newline ===");
    let grammar3 = r#"
        classes: line+.
        -line: Lm, newline.
        -newline: (-#a; -#d)+.
        Lm: -"Lm ", [Lm]*.
    "#;
    test_parse(grammar3, "Lm ʰ\n");
    
    // Test with actual full structure but just Lm
    println!("\n=== Test 4: Full structure with just Lm ===");
    let grammar4 = r#"
        classes: line+.
        -line: Lm, newline.
        -newline: (-#a; -#d)+.
        Lm: -"Lm ", [Lm]*.
    "#;
    test_parse(grammar4, "Lm ʰ\n");
    
    // Test with a few alternatives
    println!("\n=== Test 5: With 3 alternatives and complex newline ===");
    let grammar5 = r#"
        classes: line+.
        -line: (C; L; Lm), newline.
        -newline: (-#a; -#d)+.
        C: -"C ", [C]*.
        L: -"L ", [L]*.
        Lm: -"Lm ", [Lm]*.
    "#;
    test_parse(grammar5, "Lm ʰ\n");
    
    // Test with ALL 41 alternatives and complex newline
    println!("\n=== Test 6: Full 41 alternatives with complex newline ===");
    let grammar6 = r#"
        classes: line+.
        -line: ( C; Cc; Cf; Cn; Co; Cs; L; LC; Ll; Lm; Lo; Lt; Lu; M; Mc; Me; Mn; N; Nd; Nl; No; P; Pc; Pd; Pe; Pf; Pi; Po; Ps; S; Sc; Sk; Sm; So; Z; Zl; Zp; Zs), newline.
        -newline: (-#a; -#d)+.
        
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
    test_parse(grammar6, "Lm ʰ\n");
}

fn test_parse(grammar_text: &str, input: &str) {
    println!("Grammar: {} chars", grammar_text.len());
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
