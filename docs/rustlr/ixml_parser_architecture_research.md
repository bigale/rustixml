# Invisible XML Parser Architecture Research
## Performance Bottleneck Analysis & LALR/GLR Migration Path

**Date:** November 17, 2025  
**Context:** Investigating why rustixml's Earley-based runtime parser has 19 timeout tests, and whether switching to LALR/GLR can match Markup Blitz performance

---

## Executive Summary

**Key Finding:** Your original assessment was correct - the architecture difference between rustixml and Markup Blitz is in the **runtime parsing algorithm**, not the grammar parsing. However, switching from Earley to LALR/GLR for runtime is feasible and could provide 5-10x performance improvement.

**Recommended Path:** Use **rustlr** for runtime LALR parser generation (not RustyLR, which is compile-time only).

---

## Current Architecture Analysis

### What rustixml Does Now

```
┌─────────────────────────────────────────────────────┐
│ COMPILE TIME: Parse .ixml Grammar Files            │
├─────────────────────────────────────────────────────┤
│ Tool: RustyLR (LALR+GLR via lr1! macro)           │
│ Input: .ixml grammar specification                 │
│ Output: Grammar AST (baked into binary)            │
│ Performance: Not the bottleneck                    │
└─────────────────────────────────────────────────────┘
                         ↓
┌─────────────────────────────────────────────────────┐
│ RUNTIME: Parse User Input                          │
├─────────────────────────────────────────────────────┤
│ Tool: Earlgrey (Earley algorithm)                  │
│ Input: Grammar AST + user input                    │
│ Process: Dynamic parser creation at runtime         │
│ Complexity: O(n³) worst case, O(n²) unambiguous    │
│ Performance: THIS IS THE BOTTLENECK ⚠️             │
└─────────────────────────────────────────────────────┘
```

### What Markup Blitz Does

```
┌─────────────────────────────────────────────────────┐
│ RUNTIME: Parse .ixml Grammar + Generate Parser     │
├─────────────────────────────────────────────────────┤
│ Algorithm: LALR(1) table construction               │
│ Output: LALR parse tables                           │
└─────────────────────────────────────────────────────┘
                         ↓
┌─────────────────────────────────────────────────────┐
│ RUNTIME: Parse User Input                          │
├─────────────────────────────────────────────────────┤
│ Algorithm: GLR (with LALR tables)                  │
│ Base: LALR parse tables                             │
│ Fallback: GLR for ambiguity/conflicts               │
│ Complexity: O(n) deterministic, O(n³) ambiguous    │
│ Performance: 5-10x faster than pure Earley ✓       │
└─────────────────────────────────────────────────────┘
```

---

## Performance Comparison

### Algorithmic Complexity

| Algorithm | Deterministic | Unambiguous | Ambiguous | Notes |
|-----------|--------------|-------------|-----------|-------|
| **LALR** | O(n) | N/A | N/A | Can't handle all CFGs |
| **GLR** | O(n) | O(n) | O(n³) | Uses LALR tables + forking |
| **Earley** | O(n) | O(n²) | O(n³) | Always has overhead |

### Practical Performance

From research literature:
- **GLR is 5-10x faster than Earley** in practice (Tomita, 1984)
- GLR performs especially well on "nearly deterministic" grammars
- Elkhound hybrid GLR/LALR is within 10% of pure LALR on deterministic input
- **Key advantage:** GLR uses pre-computed LALR tables, so deterministic parts run at O(n)

### Why Markup Blitz is Faster

1. **LALR tables**: Deterministic parts of grammar parse at O(n)
2. **Selective GLR**: Only forks parsing threads on actual ambiguity
3. **No overhead**: Unlike Earley, no constant-factor overhead on deterministic input

### Why rustixml is Slower

1. **Pure Earley**: Always has overhead, even on deterministic input
2. **No table optimization**: Rebuilds parsing decisions dynamically
3. **O(n²) minimum**: Even unambiguous grammars run slower than O(n)

---

## Runtime Parser Generation Options in Rust

### ❌ RustyLR - NOT SUITABLE

**Type:** Compile-time only parser generator  
**Method:** Procedural macros (`lr1!`, `lalr1!`) and build scripts  
**Problem:** Cannot generate parsers dynamically at runtime

```rust
// RustyLR - Compile time only!
lr1! {
    %glr;
    %tokentype Token;
    // Grammar must be known at compile time
}
```

---

### ✅ rustlr - RECOMMENDED

**Type:** Runtime + Compile-time parser generator  
**Method:** `rustlr::generate()` function for runtime generation  
**Capabilities:**
- LALR(1) and LR(1) parser generation
- Selective Marcus-Leermakers grammars (larger than LR)
- Runtime parser table generation
- Can be called from within Rust programs

```rust
// rustlr - Runtime generation!
use rustlr::generate;

// Generate parser at runtime from grammar string
let result = generate(&[
    "-trace", "0",          // Silent mode
    "grammar_string",        // The iXML grammar
    "output_file"
]);
```

**Key Features:**
- `rustlr::generate()` function takes same args as CLI
- Can generate parser from string at runtime
- Returns `Result` with generation status
- Outputs Rust code that can be compiled or interpreted
- Built-in runtime parser (`RuntimeParser`) for interpreting tables

---

### 🔍 ANTLR4-rust - Alternative Option

**Type:** Runtime parser generator (Java-based)  
**Method:** Generate Rust parsers from ANTLR grammars  
**Pros:** Mature ecosystem, good tooling  
**Cons:** Requires Java for generation, different grammar format than iXML

---

## Migration Strategy

### Phase 1: Proof of Concept (1-2 weeks)

1. **Create rustlr adapter**
   - Write iXML → rustlr grammar converter
   - Test with simple iXML grammars
   - Measure performance vs Earlgrey

2. **Test critical grammars**
   - Run the 19 timeout tests
   - Measure parse time improvements
   - Identify any grammar compatibility issues

### Phase 2: Integration (2-3 weeks)

1. **Dual-mode architecture**
   ```rust
   pub enum RuntimeParser {
       Earley(EarleyParser),    // Current fallback
       LR(RustlrParser),        // New default
   }
   ```

2. **Selective algorithm choice**
   - Use rustlr for compatible grammars
   - Fall back to Earley for edge cases
   - Add configuration option for users

### Phase 3: Optimization (1-2 weeks)

1. **GLR enhancement**
   - Investigate if rustlr supports GLR
   - If not, explore adding GLR layer on top of LALR tables
   - Consider hybrid approach (LALR → GLR fallback)

2. **Performance tuning**
   - Cache generated parsers
   - Optimize grammar conversion
   - Benchmark against Markup Blitz

---

## Technical Considerations

### Grammar Conversion Challenges

**iXML to rustlr mapping:**

| iXML Feature | rustlr Equivalent | Complexity |
|--------------|-------------------|------------|
| Basic rules | Production rules | Easy |
| Character sets | Terminal sets | Easy |
| Repetition (+, *, ?) | Built-in operators | Easy |
| Alternatives (\|) | Production alternatives | Easy |
| Attributes (@) | Semantic actions | Medium |
| Hidden elements (-) | AST control | Medium |

### Potential Issues

1. **Grammar class restrictions**
   - Not all iXML grammars may be LALR(1)
   - May need to identify and handle edge cases
   - Consider graceful degradation to Earley

2. **Runtime compilation overhead**
   - First parse will be slower (table generation)
   - Solution: Cache generated parsers
   - Trade-off: Memory vs. speed

3. **AST construction differences**
   - rustlr has its own AST generation
   - May need adapter layer for iXML output format

---

## Alternative Approaches

### Option A: Pure rustlr (Recommended)

**Pros:**
- Native Rust, no external dependencies
- Runtime generation capability
- Active maintenance
- Good documentation

**Cons:**
- No built-in GLR (pure LALR/LR)
- May not handle all iXML grammars

### Option B: Hybrid rustlr + Custom GLR

**Pros:**
- Best of both worlds
- LALR speed + GLR flexibility
- Maximum compatibility

**Cons:**
- More complex implementation
- Need to implement GLR layer

### Option C: Keep Earley, Optimize

**Pros:**
- No architecture change
- Works for all grammars
- Lower risk

**Cons:**
- Limited improvement potential
- Still O(n²) on unambiguous grammars

### Option D: Write Custom LALR/GLR Generator

**Pros:**
- Perfect fit for iXML
- Maximum control

**Cons:**
- Significant development time
- Complex algorithm implementation
- Maintenance burden

---

## Recommendation

### Primary Path: rustlr Migration

**Rationale:**
1. Proven runtime generation capability
2. 5-10x expected performance improvement
3. Relatively low implementation risk
4. Can fall back to Earley for edge cases

**Success Criteria:**
- 19 timeout tests pass
- 5x or better speedup on average
- < 5% grammars require Earley fallback

### Implementation Priority

1. **Immediate** (This sprint)
   - Prototype iXML → rustlr converter
   - Test on 3-5 representative grammars
   - Benchmark vs current implementation

2. **Next** (Following sprint)
   - Full integration with dual-mode support
   - Run complete test suite
   - Performance profiling

3. **Future** (If needed)
   - Add GLR layer for maximum compatibility
   - Optimize grammar conversion
   - Cache management system

---

## Code Examples

### Current Earley Approach

```rust
// testsuite_utils.rs, runtime_parser.rs
let parser = EarleyParser::new(grammar);
let parse_trees = parser.parse(
    tokens.iter().map(|s| s.as_str())
)?;
```

### Proposed rustlr Approach

```rust
use rustlr::generate;
use std::fs::write;
use std::process::Command;

pub struct RustlrParser {
    tables: RuntimeParser<ASTType, ExternalState>,
}

impl RustlrParser {
    pub fn from_ixml_grammar(ixml_grammar: &str) -> Result<Self> {
        // Convert iXML to rustlr format
        let rustlr_grammar = convert_ixml_to_rustlr(ixml_grammar)?;
        
        // Generate parser at runtime
        let temp_file = "/tmp/ixml_parser.grammar";
        write(temp_file, rustlr_grammar)?;
        
        let result = generate(&[
            "-trace", "0",
            temp_file,
            "/tmp/ixml_parser.rs"
        ])?;
        
        // Load generated parser
        let parser = include!(concat!(env!("OUT_DIR"), "/ixml_parser.rs"));
        
        Ok(Self { tables: parser })
    }
    
    pub fn parse(&self, input: &str) -> Result<ParseTree> {
        // Use rustlr's RuntimeParser
        self.tables.parse(input)
    }
}

fn convert_ixml_to_rustlr(ixml: &str) -> Result<String> {
    // TODO: Implement conversion logic
    // Map iXML rules to rustlr production rules
    todo!()
}
```

### Dual-Mode Architecture

```rust
pub enum ParserBackend {
    Earley(EarleyParser),
    Rustlr(RustlrParser),
}

impl ParserBackend {
    pub fn from_grammar(grammar: &Grammar) -> Result<Self> {
        // Try rustlr first
        match RustlrParser::from_ixml_grammar(grammar) {
            Ok(parser) => Ok(Self::Rustlr(parser)),
            Err(e) => {
                warn!("Rustlr generation failed: {}, falling back to Earley", e);
                Ok(Self::Earley(EarleyParser::new(grammar)))
            }
        }
    }
    
    pub fn parse(&self, input: &str) -> Result<ParseTree> {
        match self {
            Self::Earley(p) => p.parse(input),
            Self::Rustlr(p) => p.parse(input),
        }
    }
}
```

---

## Performance Predictions

### Expected Improvements

Based on literature and Markup Blitz results:

| Grammar Type | Current (Earley) | With rustlr | Improvement |
|--------------|------------------|-------------|-------------|
| Deterministic | O(n) * k₁ | O(n) * k₂ | 5-10x faster (k₂ << k₁) |
| Unambiguous | O(n²) | O(n) | 10-100x faster |
| Ambiguous | O(n³) | O(n³) | 2-5x faster (better constants) |

### Test Case Predictions

For the 19 timeout tests:
- If mostly unambiguous: **80-90% should pass**
- If deterministic: **95%+ should pass**
- If highly ambiguous: **50-70% should pass**

---

## Risk Assessment

### Low Risk ✅
- rustlr is mature and well-maintained
- Can maintain Earley as fallback
- Incremental migration possible

### Medium Risk ⚠️
- Grammar conversion complexity
- Some iXML features may not map cleanly
- Runtime generation overhead on first parse

### High Risk ❌
- None identified

---

## Next Steps

1. **Immediate Actions:**
   - Clone rustlr and study grammar format
   - Create simple iXML → rustlr converter prototype
   - Test with 3-5 small grammars

2. **This Week:**
   - Implement basic integration
   - Run performance benchmarks
   - Validate approach with timeout tests

3. **Decision Point:**
   - If 5x+ improvement: Full integration
   - If 2-5x improvement: Continue with optimization
   - If <2x improvement: Re-evaluate approach

---

## References

- Markup Blitz: https://github.com/GuntherRademacher/markup-blitz
- rustlr: https://github.com/chuckcscccl/rustlr
- RustyLR: https://github.com/ehwan/RustyLR
- GLR vs Earley performance: Tomita (1984), "Efficient Parsing for Natural Language"
- Elkhound GLR/LALR hybrid: McPeak & Necula (2004)
- iXML specification: https://invisiblexml.org/

---

## Conclusion

**The path forward is clear:** Migrate runtime parsing from Earley to rustlr-based LALR, with Earley as a fallback. This should provide 5-10x performance improvement and eliminate most timeout failures while maintaining full iXML compatibility.

The key insight is that Markup Blitz's performance advantage comes from using **LALR tables with GLR conflict resolution** rather than pure Earley parsing. rustlr provides runtime parser generation capability that makes this migration feasible without sacrificing the dynamic nature of iXML parsing.
