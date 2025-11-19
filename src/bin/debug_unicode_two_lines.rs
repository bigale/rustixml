/// Test with just first 2 lines
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

    // First two lines only
    let input = "Lm ʰ\nLo ªאתܐޓߊࠀࡀऄঅਅઅଅஅఅಅഅඅกກༀကᄀሀᐁᚁᚠᜀᜠᝀᝠកᠠᢰᤁᥐᦀᨀᨠᬅᮃᯀᰀᱚᳩℵⴰⶀぁァㄅ智取威虎山\n";
    
    println!("Testing first 2 lines");
    println!("Input length: {} chars", input.chars().count());
    
    let ast = match parse_ixml_grammar(grammar) {
        Ok(ast) => {
            println!("✓ Grammar parsed ({} rules)", ast.rules.len());
            ast
        }
        Err(e) => {
            println!("✗ Grammar parse failed: {:?}", e);
            std::process::exit(1);
        }
    };
    
    let (builder, _transformed) = match ast_to_earlgrey(&ast) {
        Ok(result) => {
            println!("✓ Converted to Earley");
            result
        }
        Err(e) => {
            println!("✗ Conversion failed: {}", e);
            std::process::exit(1);
        }
    };
    
    let start = "classes";
    let grammar = match builder.into_grammar(start) {
        Ok(g) => {
            println!("✓ Grammar built");
            g
        }
        Err(e) => {
            println!("✗ Grammar build failed: {:?}", e);
            std::process::exit(1);
        }
    };
    
    let parser = EarleyParser::new(grammar);
    let tokens: Vec<String> = input.chars().map(|c| c.to_string()).collect();
    println!("Tokens: {} total", tokens.len());
    println!("First 10 tokens: {:?}", &tokens[..10]);
    println!("Tokens 5-15: {:?}", &tokens[5..15]);
    
    println!("\nAttempting parse...");
    match parser.parse(tokens.iter().map(|s| s.as_str())) {
        Ok(_) => {
            println!("✓ PARSE SUCCESS!");
        }
        Err(e) => {
            println!("✗ Parse failed: {:?}", e);
            std::process::exit(1);
        }
    }
}
