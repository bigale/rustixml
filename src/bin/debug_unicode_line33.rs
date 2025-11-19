/// Test line 33 - the Co (Private Use) category
use rustixml::grammar_ast::parse_ixml_grammar;
use rustixml::runtime_parser::ast_to_earlgrey;
use earlgrey::EarleyParser;

fn main() {
    let grammar = r#"
classes: line+.
-line: ( C; Cc; Cf; Cn; Co; Cs; L; LC; Ll; Lm; Lo; Lt; Lu; M; Mc; Me; Mn; N; Nd; Nl; No; P; Pc; Pd; Pe; Pf; Pi; Po; Ps; S; Sc; Sk; Sm; So; Z; Zl; Zp; Zs), newline.
-newline: (-#a; -#d)+.

  C: -"C ", (-[C], +".")*.
  L: -"L ", [L]*.
  M: -"M ", [M]*.
  N: -"N ", [N]*.
  P: -"P ", [P]*.
  S: -"S ", [S]*.
  Z: -"Z ", [Z]*.
  
  Cc: -"Cc ", (-[Cc], +".")*.
  Cf: -"Cf ", (-[Cf], +".")*.
  Cn: -"Cn ", (-[Cn], +".")*.
  Co: -"Co ", (-[Co], +".")*.
  Cs: -"Cs ", (-[Cs], +".")*.
  LC: -"LC ", [LC]*.
  Ll: -"Ll ", [Ll]*.
  Lm: -"Lm ", [Lm]*.
  Lo: -"Lo ", [Lo]*.
  Lt: -"Lt ", [Lt]*.
  Lu: -"Lu ", [Lu]*.
  Mc: -"Mc ", [Mc]*.
  Me: -"Me ", [Me]*.
  Mn: -"Mn ", [Mn]*.
  Nd: -"Nd ", [Nd]*.
  Nl: -"Nl ", [Nl]*.
  No: -"No ", [No]*.
  Pc: -"Pc ", [Pc]*.
  Pd: -"Pd ", [Pd]*.
  Pe: -"Pe ", [Pe]*.
  Pf: -"Pf ", [Pf]*.
  Pi: -"Pi ", [Pi]*.
  Po: -"Po ", [Po]*.
  Ps: -"Ps ", [Ps]*.
  Sc: -"Sc ", [Sc]*.
  Sk: -"Sk ", [Sk]*.
  Sm: -"Sm ", [Sm]*.
  So: -"So ", [So]*.
  Zl: -"Zl ", [Zl]*.
  Zp: -"Zp ", [Zp]*.
  Zs: -"Zs ", [Zs]*.
"#;

    //Test just line 33 alone - private use character U+E000
    let private_use_char = '\u{E000}';  // Co (Other, Private Use)
    let line33 = format!("Co {}\n", private_use_char);
    
    println!("Test 1: Line 33 alone (Co category with private use char U+E000)");
    test(grammar, &line33);
    
    // Test lines 32 + 33
    let line32 = "C ­͸\n";  // This appears to work from previous tests
    let lines_32_33 = format!("{}{}", line32, line33);
    println!("\nTest 2: Lines 32+33");
    test(grammar, &lines_32_33);
}

fn test(grammar: &str, input: &str) {
    println!("  Input: {} chars", input.chars().count());
    let tokens: Vec<String> = input.chars().map(|c| c.to_string()).collect();
    println!("  Tokens: {} ({:?})", tokens.len(), &tokens[..tokens.len().min(10)]);
    
    let ast = match parse_ixml_grammar(grammar) {
        Ok(ast) => ast,
        Err(e) => {
            println!("  ✗ Grammar parse failed: {:?}", e);
            return;
        }
    };
    
    let (builder, _transformed) = match ast_to_earlgrey(&ast) {
        Ok(result) => result,
        Err(e) => {
            println!("  ✗ Conversion failed: {}", e);
            return;
        }
    };
    
    let grammar = match builder.into_grammar("classes") {
        Ok(g) => g,
        Err(e) => {
            println!("  ✗ Grammar build failed: {:?}", e);
            return;
        }
    };
    
    let parser = EarleyParser::new(grammar);
    
    match parser.parse(tokens.iter().map(|s| s.as_str())) {
        Ok(_) => {
            println!("  ✓ PARSE SUCCESS!");
        }
        Err(e) => {
            println!("  ✗ Parse failed: {:?}", e);
        }
    }
}
