# Quick Start Implementation Guide
## Migrating rustixml from Earley to rustlr LALR Parsing

This guide provides concrete steps and code to begin the migration.

---

## Step 1: Add rustlr Dependency

### Cargo.toml

```toml
[dependencies]
# Keep existing Earlgrey as fallback
earlgrey = "0.3"

# Add rustlr with runtime generation features
rustlr = { version = "0.6", default-features = true }

# Add for grammar conversion
pest = "2.7"  # If needed for parsing iXML
```

---

## Step 2: Understand rustlr Grammar Format

### iXML Example

```ixml
date = year, -'-', month, -'-', day .
year = d, d, d, d .
month = '0', d | '1', ['0'|'1'|'2'] .
day = ['0'|'1'|'2'], d | '3', ['0'|'1'] .
-d = ['0'-'9'] .
```

### Equivalent rustlr Grammar

```rust
// In rustlr format (.grammar file or string)
valuetype DateAST
nonterminals Date Year Month Day
terminals DASH DIGIT
topsym Date

Date --> Year:y DASH Month:m DASH Day:d {
    DateAST::Date(y.value, m.value, d.value)
}

Year --> DIGIT:d1 DIGIT:d2 DIGIT:d3 DIGIT:d4 {
    format!("{}{}{}{}", d1.value, d2.value, d3.value, d4.value).parse().unwrap()
}

Month --> DIGIT:d {
    if d.value == '0' {
        // Month starting with 0
    }
}
// ... etc
```

---

## Step 3: Create Grammar Converter Module

### src/grammar_converter.rs

```rust
use anyhow::{Result, bail};
use std::collections::HashMap;

pub struct IxmlToRustlr {
    nonterminals: Vec<String>,
    terminals: HashMap<String, String>,
}

impl IxmlToRustlr {
    pub fn new() -> Self {
        Self {
            nonterminals: Vec::new(),
            terminals: HashMap::new(),
        }
    }
    
    pub fn convert(&mut self, ixml_grammar: &str) -> Result<String> {
        let mut output = String::new();
        
        // Header
        output.push_str("valuetype IxmlASTNode\n");
        output.push_str("nonterminals ");
        
        // Parse iXML and extract nonterminals
        let rules = self.parse_ixml_rules(ixml_grammar)?;
        
        // Collect nonterminals
        for rule in &rules {
            if !self.nonterminals.contains(&rule.name) {
                self.nonterminals.push(rule.name.clone());
            }
        }
        
        output.push_str(&self.nonterminals.join(" "));
        output.push_str("\n");
        
        // Terminals (character sets)
        output.push_str("terminals ");
        output.push_str(&self.collect_terminals(&rules));
        output.push_str("\n");
        
        // Top symbol (first rule)
        if let Some(first) = rules.first() {
            output.push_str(&format!("topsym {}\n\n", first.name));
        }
        
        // Convert each rule
        for rule in rules {
            output.push_str(&self.convert_rule(&rule)?);
            output.push_str("\n");
        }
        
        Ok(output)
    }
    
    fn parse_ixml_rules(&self, ixml: &str) -> Result<Vec<IxmlRule>> {
        // TODO: Implement iXML parser
        // For now, use your existing RustyLR-based iXML parser!
        todo!("Use existing grammar_ast.rs parser here")
    }
    
    fn convert_rule(&self, rule: &IxmlRule) -> Result<String> {
        let mut output = String::new();
        
        // Handle rule name and visibility
        let name = if rule.hidden {
            format!("-{}", rule.name)  // Hidden in iXML
        } else {
            rule.name.clone()
        };
        
        output.push_str(&format!("{} --> ", name));
        
        // Convert alternatives
        for (i, alt) in rule.alternatives.iter().enumerate() {
            if i > 0 {
                output.push_str(" | ");
            }
            output.push_str(&self.convert_alternative(alt)?);
        }
        
        output.push_str(" { /* semantic action */ }\n");
        
        Ok(output)
    }
    
    fn convert_alternative(&self, alt: &Alternative) -> Result<String> {
        let mut parts = Vec::new();
        
        for term in &alt.terms {
            match term {
                Term::NonTerminal(name) => {
                    parts.push(format!("{}:_{}", name, parts.len()));
                }
                Term::Terminal(s) => {
                    parts.push(self.terminal_to_rustlr(s));
                }
                Term::CharacterSet(chars) => {
                    // Convert [a-z] to rustlr terminal
                    parts.push(self.charset_to_terminal(chars));
                }
                Term::Repetition(inner, kind) => {
                    let inner_str = self.convert_term(inner)?;
                    match kind {
                        RepKind::ZeroOrMore => parts.push(format!("{}*", inner_str)),
                        RepKind::OneOrMore => parts.push(format!("{}+", inner_str)),
                        RepKind::Optional => parts.push(format!("{}?", inner_str)),
                    }
                }
            }
        }
        
        Ok(parts.join(" "))
    }
    
    fn charset_to_terminal(&self, chars: &str) -> String {
        // Map iXML character sets to rustlr terminal sets
        // e.g., ['0'-'9'] -> [DIGIT]
        format!("[{}]", chars)
    }
    
    fn collect_terminals(&mut self, rules: &[IxmlRule]) -> String {
        // Extract all terminals used in grammar
        let mut terminals = std::collections::HashSet::new();
        
        // Scan rules for terminals
        for rule in rules {
            // Extract terminals from alternatives
            // ...
        }
        
        terminals.iter().cloned().collect::<Vec<_>>().join(" ")
    }
}

// Data structures for parsed iXML
#[derive(Debug, Clone)]
pub struct IxmlRule {
    pub name: String,
    pub hidden: bool,
    pub alternatives: Vec<Alternative>,
}

#[derive(Debug, Clone)]
pub struct Alternative {
    pub terms: Vec<Term>,
}

#[derive(Debug, Clone)]
pub enum Term {
    NonTerminal(String),
    Terminal(String),
    CharacterSet(String),
    Repetition(Box<Term>, RepKind),
}

#[derive(Debug, Clone)]
pub enum RepKind {
    ZeroOrMore,
    OneOrMore,
    Optional,
}
```

---

## Step 4: Create rustlr Parser Wrapper

### src/rustlr_parser.rs

```rust
use anyhow::{Result, Context};
use rustlr::{generate, RuntimeParser};
use std::fs;
use std::path::PathBuf;
use tempfile::TempDir;

pub struct RustlrBackend {
    // Store generated parser module path
    parser_path: PathBuf,
    temp_dir: TempDir,
}

impl RustlrBackend {
    pub fn from_ixml(ixml_grammar: &str) -> Result<Self> {
        // Create temporary directory for generated files
        let temp_dir = TempDir::new()
            .context("Failed to create temp directory")?;
        
        // Convert iXML to rustlr format
        let mut converter = crate::grammar_converter::IxmlToRustlr::new();
        let rustlr_grammar = converter.convert(ixml_grammar)
            .context("Failed to convert iXML to rustlr format")?;
        
        // Write grammar file
        let grammar_path = temp_dir.path().join("grammar.grammar");
        fs::write(&grammar_path, rustlr_grammar)
            .context("Failed to write grammar file")?;
        
        // Generate parser
        let parser_path = temp_dir.path().join("parser.rs");
        
        let args = vec![
            "-trace", "0",  // Silent mode
            grammar_path.to_str().unwrap(),
            parser_path.to_str().unwrap(),
        ];
        
        generate(&args)
            .context("rustlr parser generation failed")?;
        
        Ok(Self {
            parser_path,
            temp_dir,
        })
    }
    
    pub fn parse(&self, input: &str) -> Result<ParseTree> {
        // Load and use the generated parser
        // This is tricky because we need to dynamically load Rust code
        
        // Option 1: Use rustlr's RuntimeParser directly
        // (requires keeping the state machine in memory)
        
        // Option 2: Compile the generated parser as a dynamic library
        // (more complex but cleaner)
        
        // For now, we'll sketch Option 1
        self.parse_with_runtime(input)
    }
    
    fn parse_with_runtime(&self, input: &str) -> Result<ParseTree> {
        // Read generated parser file
        let parser_code = fs::read_to_string(&self.parser_path)?;
        
        // The generated parser contains a make_parser() function
        // We need to eval this or use a different approach
        
        // CHALLENGE: Rust doesn't have eval()
        // SOLUTION: Use rustlr's internal APIs directly
        
        todo!("Implement runtime parser execution")
    }
}
```

---

## Step 5: Alternative Approach - Use rustlr Grammar Directly

### Better Strategy: Generate Parser from Grammar String

```rust
use rustlr::{Grammar, LR1};

pub struct RustlrBackend {
    grammar: Grammar,
    parser: LR1Parser,
}

impl RustlrBackend {
    pub fn from_ixml_ast(ast: &GrammarAst) -> Result<Self> {
        // Convert AST to rustlr Grammar object directly
        let mut grammar = Grammar::new();
        
        // Add productions from AST
        for rule in &ast.rules {
            self.add_rule_to_grammar(&mut grammar, rule)?;
        }
        
        // Generate parser tables
        let parser = LR1Parser::from_grammar(&grammar)?;
        
        Ok(Self { grammar, parser })
    }
    
    fn add_rule_to_grammar(
        &self, 
        grammar: &mut Grammar, 
        rule: &Rule
    ) -> Result<()> {
        // Map iXML rule to rustlr production
        let lhs = grammar.add_nonterminal(&rule.name);
        
        for alternative in &rule.alternatives {
            let mut rhs = Vec::new();
            
            for term in &alternative.terms {
                match term {
                    Term::NonTerminal(name) => {
                        let nt = grammar.add_nonterminal(name);
                        rhs.push(Symbol::NonTerminal(nt));
                    }
                    Term::Terminal(s) => {
                        let t = grammar.add_terminal(s);
                        rhs.push(Symbol::Terminal(t));
                    }
                    // ... handle other cases
                }
            }
            
            grammar.add_production(lhs, rhs)?;
        }
        
        Ok(())
    }
    
    pub fn parse(&self, input: &str) -> Result<ParseTree> {
        // Tokenize input
        let tokens = self.tokenize(input)?;
        
        // Parse with LR parser
        self.parser.parse(&tokens)
    }
    
    fn tokenize(&self, input: &str) -> Result<Vec<Token>> {
        // Implement tokenization based on grammar terminals
        todo!()
    }
}
```

---

## Step 6: Integration with Existing Code

### src/runtime_parser.rs (Modified)

```rust
pub enum ParserBackend {
    Earley(EarleyParser),
    Rustlr(RustlrBackend),
}

pub struct Parser {
    backend: ParserBackend,
    grammar: Arc<Grammar>,
}

impl Parser {
    pub fn new(grammar: Grammar) -> Result<Self> {
        // Try rustlr first
        let backend = match RustlrBackend::from_ixml_ast(&grammar.ast) {
            Ok(rustlr) => {
                log::info!("Using rustlr LALR backend");
                ParserBackend::Rustlr(rustlr)
            }
            Err(e) => {
                log::warn!("rustlr failed ({}), falling back to Earley", e);
                ParserBackend::Earley(EarleyParser::new(&grammar))
            }
        };
        
        Ok(Self {
            backend,
            grammar: Arc::new(grammar),
        })
    }
    
    pub fn parse(&self, input: &str) -> Result<ParseTree> {
        match &self.backend {
            ParserBackend::Earley(p) => p.parse(input),
            ParserBackend::Rustlr(p) => p.parse(input),
        }
    }
}
```

---

## Step 7: Testing Strategy

### tests/rustlr_integration_test.rs

```rust
#[cfg(test)]
mod tests {
    use super::*;
    
    #[test]
    fn test_simple_grammar() {
        let ixml = r#"
            S = A, B .
            A = 'a' .
            B = 'b' .
        "#;
        
        let parser = Parser::new_from_ixml(ixml).unwrap();
        let result = parser.parse("ab").unwrap();
        
        assert!(result.is_valid());
    }
    
    #[test]
    fn test_timeout_grammar_001() {
        // Test one of the 19 timeout cases
        let ixml = load_test_grammar("timeout_001.ixml");
        let input = load_test_input("timeout_001.txt");
        
        let parser = Parser::new_from_ixml(&ixml).unwrap();
        
        // Should complete in reasonable time now
        let start = Instant::now();
        let result = parser.parse(&input);
        let duration = start.elapsed();
        
        assert!(duration < Duration::from_secs(5), 
                "Parse took too long: {:?}", duration);
        assert!(result.is_ok());
    }
    
    #[test]
    fn test_all_timeout_cases() {
        for i in 1..=19 {
            let test_name = format!("timeout_{:03}", i);
            // Test each case
        }
    }
}
```

---

## Step 8: Benchmarking

### benches/parser_comparison.rs

```rust
use criterion::{black_box, criterion_group, criterion_main, Criterion};

fn bench_earley(c: &mut Criterion) {
    let grammar = load_test_grammar();
    let input = generate_test_input(1000);
    
    c.bench_function("earley_1k", |b| {
        let parser = EarleyParser::new(&grammar);
        b.iter(|| {
            parser.parse(black_box(&input))
        });
    });
}

fn bench_rustlr(c: &mut Criterion) {
    let grammar = load_test_grammar();
    let input = generate_test_input(1000);
    
    c.bench_function("rustlr_1k", |b| {
        let parser = RustlrBackend::from_ixml_ast(&grammar.ast).unwrap();
        b.iter(|| {
            parser.parse(black_box(&input))
        });
    });
}

criterion_group!(benches, bench_earley, bench_rustlr);
criterion_main!(benches);
```

---

## Step 9: Fallback Strategy

### Determining When to Use Each Backend

```rust
impl Parser {
    fn select_backend(grammar: &Grammar) -> ParserBackend {
        // Analyze grammar to decide which backend to use
        
        if grammar.is_highly_ambiguous() {
            // Earley might be better for very ambiguous grammars
            return Self::create_earley(grammar);
        }
        
        if grammar.has_unsupported_features() {
            // Some iXML features might not map to LALR
            return Self::create_earley(grammar);
        }
        
        // Try rustlr, fall back on error
        match Self::create_rustlr(grammar) {
            Ok(backend) => backend,
            Err(e) => {
                log::warn!("rustlr creation failed: {}", e);
                Self::create_earley(grammar)
            }
        }
    }
}
```

---

## Step 10: Performance Monitoring

### Add Metrics

```rust
pub struct ParserMetrics {
    pub backend_type: String,
    pub parse_time: Duration,
    pub grammar_size: usize,
    pub input_size: usize,
}

impl Parser {
    pub fn parse_with_metrics(
        &self, 
        input: &str
    ) -> Result<(ParseTree, ParserMetrics)> {
        let start = Instant::now();
        let result = self.parse(input)?;
        let duration = start.elapsed();
        
        let metrics = ParserMetrics {
            backend_type: match &self.backend {
                ParserBackend::Earley(_) => "earley",
                ParserBackend::Rustlr(_) => "rustlr",
            }.to_string(),
            parse_time: duration,
            grammar_size: self.grammar.rules.len(),
            input_size: input.len(),
        };
        
        Ok((result, metrics))
    }
}
```

---

## Common Pitfalls & Solutions

### Pitfall 1: Grammar Conversion Errors

**Problem:** iXML features that don't map cleanly to LALR

**Solution:**
```rust
// Add validation before conversion
fn validate_for_lalr(grammar: &Grammar) -> Result<()> {
    for rule in &grammar.rules {
        if rule.has_left_recursion() {
            bail!("Left recursion not supported in LALR");
        }
        // More checks...
    }
    Ok(())
}
```

### Pitfall 2: Runtime Code Generation

**Problem:** Rust doesn't allow dynamic code execution

**Solution:** Use rustlr's internal APIs directly instead of generating and compiling code:

```rust
// Don't do this:
// let code = generate_parser_code(grammar);
// eval(code);  // Not possible in Rust!

// Do this instead:
let state_machine = build_lalr_automaton(grammar);
let parser = RuntimeParser::new(state_machine);
```

### Pitfall 3: Performance Regression

**Problem:** Some grammars might be slower with LALR

**Solution:** A/B test and maintain both backends

```rust
#[cfg(feature = "dual-parse-validation")]
fn validate_parsers(&self, input: &str) -> Result<()> {
    let earley_result = self.earley_parse(input)?;
    let rustlr_result = self.rustlr_parse(input)?;
    
    assert_eq!(earley_result, rustlr_result, 
               "Parser results diverged!");
    
    Ok(())
}
```

---

## Next Actions

1. **Week 1: Prototype**
   - [ ] Implement basic iXML → rustlr converter
   - [ ] Test with 3 simple grammars
   - [ ] Benchmark against Earley

2. **Week 2: Integration**
   - [ ] Add dual-backend support
   - [ ] Run test suite
   - [ ] Fix compatibility issues

3. **Week 3: Optimization**
   - [ ] Cache generated parsers
   - [ ] Tune grammar conversion
   - [ ] Performance profiling

4. **Week 4: Validation**
   - [ ] Run all 19 timeout tests
   - [ ] Compare against Markup Blitz
   - [ ] Document findings

---

## Resources

- rustlr tutorial: https://cs.hofstra.edu/~cscccl/rustlr_project/
- rustlr docs: https://docs.rs/rustlr/latest/rustlr/
- rustlr examples: https://github.com/chuckcscccl/rustlr/tree/master/examples
- iXML spec: https://invisiblexml.org/

---

## Questions to Resolve

1. **How to best access rustlr's internal Grammar/StateM
 APIs?**
   - Study rustlr source code
   - Look at RuntimeParser implementation

2. **Which iXML features are LALR-incompatible?**
   - Create compatibility matrix
   - Test edge cases

3. **How to handle parser caching?**
   - Use persistent cache directory
   - Cache key = hash(grammar)

4. **What's the optimal fallback strategy?**
   - Profile both backends on each grammar
   - Learn which patterns work best where
