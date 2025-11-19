/// Test line 3 specifically
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

    //Test just line 3 alone
    let line3 = "Ll aàǆµßαϐюաდᏸℊℹⰰⲁａ𐐨𐓘𐖗𐳀𑣁ðþ\n";
    
    println!("Test 1: Line 3 alone");
    test(grammar, line3);
    
    // Test lines 1+3
    let lines_1_3 = "Lm ʰ\nLl aàǆµßαϐюաდᏸℊℹⰰⲁａ𐐨𐓘𐖗𐳀𑣁ðþ\n";
    println!("\nTest 2: Lines 1+3");
    test(grammar, lines_1_3);
    
    // Test lines 1+2+3
    let lines_1_2_3 = "Lm ʰ\nLo ªאתܐޓߊࠀࡀऄঅਅઅଅஅఅಅഅඅกກༀကᄀሀᐁᚁᚠᜀᜠᝀᝠកᠠᢰᤁᥐᦀᨀᨠᬅᮃᯀᰀᱚᳩℵⴰⶀぁァㄅ智取威虎山\nLl aàǆµßαϐюաდᏸℊℹⰰⲁａ𐐨𐓘𐖗𐳀𑣁ðþ\n";
    println!("\nTest 3: Lines 1+2+3");
    test(grammar, lines_1_2_3);
}

fn test(grammar: &str, input: &str) {
    println!("  Input chars: {}", input.chars().count());
    let tokens: Vec<String> = input.chars().map(|c| c.to_string()).collect();
    println!("  Tokens: {}", tokens.len());
    println!("  First 10 tokens: {:?}", &tokens[..tokens.len().min(10)]);
    
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
