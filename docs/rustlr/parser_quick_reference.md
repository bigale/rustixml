# Parser Algorithm & Library Quick Reference
## For Invisible XML Implementation Decisions

---

## Algorithm Comparison Matrix

| Algorithm | Complexity (Det) | Complexity (Unamb) | Complexity (Amb) | Grammar Class | Runtime Gen | Notes |
|-----------|------------------|--------------------|--------------------|---------------|-------------|-------|
| **LALR(1)** | O(n) | O(n) | ❌ Fails | LALR(1) CFG | Yes | Fast, limited grammars |
| **GLR** | O(n) | O(n) | O(n³) | Any CFG | Yes | Fast + flexible |
| **Earley** | O(n) | O(n²) | O(n³) | Any CFG | Yes | Overhead on det input |
| **GLL** | O(n) | O(n²) | O(n³) | Any CFG | Yes | LL-based alternative |
| **PEG** | O(n) | O(n) | ❌ No ambiguity | PEG (not CFG) | Yes | Ordered choice |

### Key Insights

- **GLR** = LALR tables + forking for ambiguity → best of both worlds
- **Earley** = Always has overhead, even on deterministic input
- **For iXML:** GLR is optimal (handles all CFGs, fast on typical input)

---

## Rust Parser Library Comparison

### For Runtime Parser Generation

| Library | Algorithm | Runtime Gen? | Maturity | Performance | iXML Suitability |
|---------|-----------|--------------|----------|-------------|------------------|
| **rustlr** ✅ | LALR/LR | ✅ Yes | Mature | Fast | **HIGH** - Recommended |
| RustyLR | LALR/GLR | ❌ No | Active | Fast | LOW - Compile only |
| earlgrey | Earley | ✅ Yes | Stable | Slow | Medium - Current impl |
| pest | PEG | ❌ No | Mature | Fast | LOW - Not CFG |
| nom | Combinators | ❌ N/A | Mature | Fast | LOW - Manual impl |
| pom | Combinators | ❌ N/A | Small | Med | LOW - Manual impl |
| lalrpop | LALR | ❌ No | Mature | Fast | LOW - Compile only |
| antlr4-rust | LL(*) | ⚠️ Via Java | Mature | Good | Medium - Java req'd |

### Detailed Analysis

#### ✅ rustlr (RECOMMENDED)

```toml
[dependencies]
rustlr = "0.6"
```

**Pros:**
- ✅ Runtime parser generation via `rustlr::generate()`
- ✅ LALR(1) and LR(1) support
- ✅ Selective Marcus-Leermakers (beyond LR!)
- ✅ Pure Rust, no dependencies
- ✅ Good documentation
- ✅ Active maintenance

**Cons:**
- ❌ No built-in GLR (but can add)
- ⚠️ Generated code needs compilation or interpretation

**Usage:**
```rust
use rustlr::generate;

let result = generate(&[
    "-trace", "0",
    "grammar.grammar",
    "parser.rs"
]);
```

---

#### ❌ RustyLR (NOT SUITABLE)

```toml
[dependencies]
rusty_lr_parser = "0.14"
```

**Pros:**
- ✅ GLR support
- ✅ Excellent error messages
- ✅ LALR/LR(1)/IELR(1)

**Cons:**
- ❌ **Compile-time only** - uses proc macros
- ❌ Cannot generate parsers at runtime
- ❌ Not suitable for iXML

**Usage (compile-time only):**
```rust
lr1! {
    %glr;
    %tokentype Token;
    // Grammar must be hardcoded
}
```

---

#### ⚠️ earlgrey (CURRENT - FALLBACK)

```toml
[dependencies]
earlgrey = "0.3"
```

**Pros:**
- ✅ Runtime generation
- ✅ Handles all CFGs
- ✅ Simple API
- ✅ Currently works

**Cons:**
- ❌ **5-10x slower than GLR**
- ❌ O(n²) on unambiguous grammars
- ❌ Overhead on deterministic input

**Keep as fallback for:**
- Grammars that don't fit LALR
- Edge cases
- Validation/comparison

---

#### ❌ lalrpop (NOT SUITABLE)

```toml
[dependencies]
lalrpop = "0.20"
```

**Pros:**
- ✅ Excellent LALR(1) implementation
- ✅ Great error messages
- ✅ Good docs

**Cons:**
- ❌ **Compile-time only** - build script
- ❌ Cannot generate at runtime

---

#### ⚠️ antlr4-rust (ALTERNATIVE)

```toml
[dependencies]
antlr-rust = "0.3"
```

**Pros:**
- ✅ Mature ecosystem
- ✅ LL(*) handles many grammars
- ✅ Great tooling

**Cons:**
- ⚠️ Requires Java for generation
- ⚠️ Different grammar format
- ⚠️ More complex setup

**Consider if:**
- Already using ANTLR
- Need Java interop
- Want richer tooling

---

## Performance Comparison (Relative)

Based on literature and benchmarks:

```
Parsing "typical" programming language input (mostly deterministic):

LALR:    ████████████████████ (baseline - 100%)
GLR:     ████████████████████ (100% on det, 20% on amb)
Earley:  ████                 (20% - 5x slower)
GLL:     ███                  (15% - 6x slower)
PEG:     ████████████████     (80% - faster but not CFG)
```

**Key Point:** GLR with LALR tables matches LALR speed on deterministic input!

---

## Grammar Feature Support

| Feature | LALR | GLR | Earley | PEG | iXML Needs |
|---------|------|-----|--------|-----|------------|
| All CFGs | ❌ | ✅ | ✅ | ❌ | ✅ Required |
| Ambiguity | ❌ | ✅ | ✅ | ❌ | ✅ Required |
| Left recursion | ⚠️ | ✅ | ✅ | ❌ | ⚠️ Sometimes |
| Lookahead > 1 | ❌ | ✅ | ✅ | ✅ | ⚠️ Sometimes |
| Deterministic | ✅ | ✅ | ✅ | ✅ | ✅ Common |

**Conclusion:** Need GLR or Earley for full iXML support. GLR is faster.

---

## Implementation Strategies

### Strategy A: Pure rustlr (Fastest Path) ⭐

```
┌─────────────┐
│ iXML Grammar│
└──────┬──────┘
       │ convert
       ▼
┌─────────────────┐
│ rustlr Grammar  │
└──────┬──────────┘
       │ rustlr::generate()
       ▼
┌─────────────────┐
│ LALR Tables     │
└──────┬──────────┘
       │ RuntimeParser
       ▼
┌─────────────────┐
│ Parse Input     │
└─────────────────┘
```

**Pros:**
- Simplest implementation
- Fastest performance (on compatible grammars)
- Native Rust

**Cons:**
- Some grammars may not fit LALR
- Need fallback to Earley

---

### Strategy B: rustlr + Custom GLR Layer (Best Performance) ⭐⭐⭐

```
┌─────────────┐
│ iXML Grammar│
└──────┬──────┘
       │
       ▼
┌─────────────────┐
│ LALR Tables     │ ← rustlr::generate()
└──────┬──────────┘
       │
       ▼
┌─────────────────┐
│ GLR Parser      │ ← Custom implementation
│ (uses tables)   │
└──────┬──────────┘
       │
       ▼
┌─────────────────┐
│ Parse Input     │
└─────────────────┘
```

**Pros:**
- Best performance
- Handles all iXML grammars
- Matches Markup Blitz architecture

**Cons:**
- More complex (need to implement GLR)
- More testing required

---

### Strategy C: Dual Backend (Safest) ⭐⭐

```
┌─────────────┐
│ iXML Grammar│
└──────┬──────┘
       │
       ├──────────────┬─────────────┐
       │              │             │
       ▼              ▼             ▼
   ┌────────┐   ┌─────────┐  ┌─────────┐
   │ rustlr │   │ Earley  │  │  GLR    │
   └────┬───┘   └────┬────┘  └────┬────┘
        │            │            │
        └────────────┴────────────┘
                     │
                     ▼
               ┌──────────┐
               │ Pick best│
               └──────────┘
```

**Pros:**
- Maximum compatibility
- Can A/B test
- Gradual migration

**Cons:**
- More code to maintain
- More complex logic

---

## Decision Matrix

### Choose rustlr (LALR) if:
- ✅ Grammar is deterministic or nearly so
- ✅ Need maximum speed
- ✅ Willing to handle edge cases separately
- ✅ 80%+ test cases pass

### Add GLR layer if:
- ✅ Need to handle all grammars
- ✅ Want Markup Blitz-level performance
- ✅ Have time to implement
- ✅ Need production quality

### Keep Earley if:
- ✅ Quick validation needed
- ✅ Fallback for edge cases
- ✅ Not performance-critical paths
- ✅ Research/testing only

---

## Recommended Implementation Path

### Phase 1: rustlr LALR (2-3 weeks)
```
Week 1: Grammar conversion + basic parsing
Week 2: Integration + test suite
Week 3: Optimization + edge cases
```

**Expected Results:**
- 70-80% of tests pass
- 5-10x speedup on passing tests
- 19 timeout tests → ~15 passing

---

### Phase 2: GLR Layer (2-3 weeks)
```
Week 1: GLR algorithm implementation
Week 2: Integration with LALR tables
Week 3: Testing + optimization
```

**Expected Results:**
- 95%+ tests pass
- Maintain 5-10x speedup
- 19 timeout tests → 18-19 passing

---

### Phase 3: Polish (1-2 weeks)
```
Week 1: Performance tuning
Week 2: Documentation + examples
```

**Expected Results:**
- Match or exceed Markup Blitz
- Production ready
- All tests pass

---

## Code Size Estimates

| Component | Lines of Code | Complexity |
|-----------|---------------|------------|
| Grammar converter | 500-800 | Medium |
| rustlr wrapper | 200-300 | Low |
| GLR layer | 800-1200 | High |
| Dual backend | 300-400 | Low |
| Tests | 500+ | Low |
| **Total** | **2300-3200** | **Medium** |

---

## Performance Targets

### Current (Earley)
```
Small grammar (10 rules):     10ms
Medium grammar (100 rules):   100ms  
Large grammar (1000 rules):   1000ms ❌ TIMEOUT
```

### Target (rustlr LALR)
```
Small grammar:   1-2ms     (5-10x faster)
Medium grammar:  10-20ms   (5-10x faster)
Large grammar:   100-200ms (5-10x faster) ✅
```

### Stretch (rustlr + GLR)
```
Small grammar:   1-2ms      (matches LALR)
Medium grammar:  10-20ms    (matches LALR)
Large grammar:   50-100ms   (10-20x faster!) ✅✅
Ambiguous:       200-300ms  (still fast)
```

---

## Key Takeaways

1. **rustlr is the right choice** for runtime LALR generation in Rust
2. **GLR layer** will unlock full performance (like Markup Blitz)
3. **Keep Earley as fallback** for edge cases
4. **Expected 5-10x speedup** with LALR alone
5. **10-20x with GLR** on compatible grammars

---

## Quick Start Command

```bash
# Add dependency
cargo add rustlr

# Create test
cat > test_grammar.grammar << 'EOF'
valuetype i32
nonterminals E T F
terminals + * ( ) num
topsym E

E --> E + T | T
T --> T * F | F
F --> ( E ) | num
EOF

# Test rustlr
cargo test --test rustlr_integration
```

---

## Decision Checklist

Before implementing:
- [ ] Review rustlr documentation
- [ ] Test grammar conversion with 3-5 examples
- [ ] Benchmark against Earley
- [ ] Verify 5x+ speedup
- [ ] Plan fallback strategy
- [ ] Define success criteria

If 5x+ speedup achieved:
- [ ] Full integration
- [ ] Test suite adaptation
- [ ] Consider GLR layer

If <5x speedup:
- [ ] Re-evaluate approach
- [ ] Consider alternative strategies
- [ ] Optimize grammar conversion

---

## Resources Quick Links

- **rustlr**: https://github.com/chuckcscccl/rustlr
- **RustyLR**: https://github.com/ehwan/RustyLR
- **Markup Blitz**: https://github.com/GuntherRademacher/markup-blitz
- **iXML Spec**: https://invisiblexml.org/
- **GLR Paper**: Tomita (1984)
- **Earley Paper**: Earley (1970)
