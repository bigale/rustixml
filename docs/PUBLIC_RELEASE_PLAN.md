# Repository Organization Plan for Public Release

## Current Situation Analysis

**Repository**: rustixml - iXML Parser Implementation in Rust  
**Current Status**: Private, 58 commits, ~1100 LOC (native) + ~2500 LOC (Earley)  
**State**: Two complete implementations with rich experimental history

### What We Have

1. **Two Complete Implementations**:
   - ✅ Earley-based parser (39.8% pass rate, 19/49 tests)
   - ✅ Native recursive descent parser (83.7% pass rate, 41/49 tests)

2. **Extensive Documentation** (14 markdown files):
   - Architecture docs, design docs, progress reports
   - Parser comparison, abstraction analysis
   - Implementation checklists, transition guides

3. **Rich Experimental History**:
   - 58 commits documenting the full journey
   - Multiple debug binaries (80+ debug_*.rs files)
   - Test files and experimentation artifacts

4. **Test Infrastructure**:
   - 133 iXML conformance tests
   - Multiple test runners
   - Performance benchmarks

## Public Release Options

### Option 1: Clean Slate (Recommended for Maximum Impact) 🎯

**Approach**: Start fresh with only the winning implementation

**Structure**:
```
rustixml/
├── README.md                    # Clean, professional introduction
├── LICENSE                      # MIT/Apache-2.0
├── Cargo.toml                   # Streamlined dependencies
├── CHANGELOG.md                 # Release history
├── CONTRIBUTING.md              # Contribution guidelines
├── .github/
│   └── workflows/
│       └── ci.yml              # CI/CD
├── src/
│   ├── lib.rs
│   ├── ast.rs                  # Grammar AST
│   ├── lexer.rs                # iXML lexer
│   ├── grammar_parser.rs       # Grammar parser
│   ├── input_stream.rs         # Unicode input handling
│   ├── native_parser.rs        # ⭐ THE PARSER
│   └── parse_context.rs        # Parsing context
├── examples/
│   ├── basic_usage.rs          # Simple examples
│   ├── json_parser.rs          # JSON grammar
│   └── csv_parser.rs           # CSV grammar
├── benches/
│   └── parsing_benchmarks.rs   # Performance tests
├── tests/
│   └── conformance_tests.rs    # Integration tests
├── docs/
│   ├── architecture.md         # System design
│   ├── ixml_reference.md       # iXML spec summary
│   └── api.md                  # API documentation
└── ixml_tests/                 # Conformance test suite
```

**Actions**:
1. Archive experimental history to separate branch `archive/experimental`
2. Create clean `main` branch with only native parser
3. Consolidate docs to 3-4 essential files
4. Remove all debug binaries
5. Add examples and proper API

**Pros**:
- ✅ Clean, professional impression
- ✅ Easy to understand and contribute to
- ✅ Small surface area (~1100 LOC core)
- ✅ Clear value proposition (83.7% conformance)
- ✅ Experimental history preserved in archive branch

**Cons**:
- ❌ Loses visible journey/learning story
- ❌ Earley implementation not accessible

---

### Option 2: Two-Track Implementation 🔄

**Approach**: Keep both implementations, clearly separated

**Structure**:
```
rustixml/
├── README.md                    # Explains both implementations
├── LICENSE
├── Cargo.toml                   # Features: native (default), earley
├── src/
│   ├── lib.rs                  # Feature-gated exports
│   ├── common/                 # Shared code
│   │   ├── ast.rs
│   │   ├── lexer.rs
│   │   └── grammar_parser.rs
│   ├── native/                 # ⭐ Native parser (default)
│   │   ├── mod.rs
│   │   ├── parser.rs
│   │   └── context.rs
│   └── earley/                 # Earley parser (feature-gated)
│       ├── mod.rs
│       ├── translator.rs
│       └── runtime.rs
├── examples/
│   ├── native_examples.rs
│   └── earley_examples.rs
├── docs/
│   ├── COMPARISON.md           # Native vs Earley
│   ├── architecture.md
│   └── migration.md            # Earley → Native guide
└── tests/
```

**Cargo.toml**:
```toml
[features]
default = ["native"]
native = []
earley = ["earlgrey", "rusty_lr"]
all = ["native", "earley"]
```

**Pros**:
- ✅ Academic/research value (two approaches)
- ✅ Users can choose implementation
- ✅ Shows evolution and comparison
- ✅ Educational value

**Cons**:
- ❌ More complex to maintain
- ❌ Larger codebase (~3600 LOC)
- ❌ May confuse users about which to use
- ❌ Both need maintenance

---

### Option 3: History Preservation with Clean Defaults 📚

**Approach**: Clean main branch + comprehensive history documentation

**Structure**:
```
rustixml/
├── README.md                    # Clean, native parser focused
├── HISTORY.md                   # The full journey
├── LICENSE
├── Cargo.toml
├── src/                         # ONLY native parser
│   ├── lib.rs
│   ├── ast.rs
│   ├── lexer.rs
│   ├── grammar_parser.rs
│   ├── native_parser.rs
│   └── parse_context.rs
├── examples/
├── docs/
│   ├── architecture.md
│   ├── design_decisions.md     # Why native > Earley
│   └── PARSER_COMPARISON.md    # Your existing comparison
└── archive/                     # Git subdirectory or submodule
    └── earley-implementation/   # Frozen Earley code
```

**HISTORY.md**:
```markdown
# Development History

## The Journey to 83.7% Conformance

This project went through two major implementation phases:

1. **Phase 1: Earley-based Parser** (Commits: d2785be → 3e0033d)
   - 39.8% conformance (19/49 tests)
   - Translation-based approach
   - Hit fundamental abstraction mismatches

2. **Phase 2: Native Interpreter** (Commits: 69bea14f → present)
   - 83.7% conformance (41/49 tests)
   - Direct AST interpretation
   - +115% improvement over Earley

See [docs/PARSER_COMPARISON.md](docs/PARSER_COMPARISON.md) for detailed analysis.

For the complete experimental history, see branch: `archive/experimental`
```

**Pros**:
- ✅ Clean main codebase (native only)
- ✅ History is documented and accessible
- ✅ Shows evolution without clutter
- ✅ Easy to navigate for new users
- ✅ Preserves learning journey

**Cons**:
- ❌ Earley code not runnable (only in archive)
- ❌ Needs extra documentation work

---

## Recommendations by Use Case

### For Maximum GitHub Impact & Stars ⭐
→ **Option 1: Clean Slate**

Why:
- First impression matters - clean code attracts contributors
- Small, focused codebase is more approachable
- 83.7% conformance is impressive standalone
- Archive branch keeps history for interested parties

### For Academic/Research Value 📖
→ **Option 3: History Preservation**

Why:
- Shows the full problem-solving process
- Comparison between approaches is valuable
- Educational for parser implementation learners
- Maintains experimental rigor

### For Production Library Use 🚀
→ **Option 1: Clean Slate**

Why:
- Clear which implementation to use (only one!)
- Minimal dependencies (no earlgrey/rusty_lr)
- Smaller compile times
- Focused maintenance

## Detailed Reorganization Plan (Option 1 - Recommended)

### Step 1: Create Archive Branch
```bash
git checkout -b archive/experimental
git push origin archive/experimental
```

### Step 2: Create Clean Main Branch
```bash
git checkout --orphan main-clean
git rm -rf .
```

### Step 3: Cherry-pick Essential Commits
```bash
# Core infrastructure
git cherry-pick 69bea14f  # Phase 1: Native interpreter core
git cherry-pick 723dceba  # Phase 6: Longest-match
git cherry-pick 646cd3b0  # Phase 6: Flatten sequences
git cherry-pick 364821df  # Phase 6: Separator collection
git cherry-pick 88f58d3f  # Factor-level hidden
git cherry-pick 763aeba   # Promoted mark
git cherry-pick 29f97ff   # Status document
git cherry-pick 9c50c23   # Parser comparison
```

### Step 4: File Organization

**Keep**:
- `src/lib.rs`
- `src/ast.rs`
- `src/lexer.rs`
- `src/grammar_parser.rs`
- `src/grammar_ast.rs`
- `src/input_stream.rs`
- `src/native_parser.rs`
- `src/parse_context.rs`
- `ixml_tests/` (conformance tests)

**Remove**:
- `src/runtime_parser.rs` (Earley)
- `src/grammar.rs` (Earley translator)
- `src/grammar_v2.rs` (Earley)
- `src/testsuite_utils.rs` (Earley-specific)
- All 80+ `debug_*.rs` files
- Root-level test files (`test_*.rs`)
- `src/bin/` (except main.rs, rename to examples/)

**Reorganize docs/**:
- Keep: `PARSER_COMPARISON.md`, `NATIVE_PARSER_STATUS.md`
- Merge: `ARCHITECTURE.md` + `NATIVE_INTERPRETER_DESIGN.md` → `docs/architecture.md`
- Remove: Earley-specific docs, rustlr docs, experimental docs
- Create: `docs/api.md`, `docs/ixml_reference.md`

### Step 5: Update Cargo.toml
```toml
[package]
name = "rustixml"
version = "0.2.0"  # Breaking change from Earley to Native
edition = "2021"
authors = ["Alex Everitt <bigale@netzero.net>"]
description = "Native iXML (Invisible XML) parser implementation in Rust - 83.7% conformance"
license = "MIT OR Apache-2.0"
repository = "https://github.com/bigale/rustixml"
keywords = ["parser", "xml", "ixml", "invisible-xml"]
categories = ["parsing", "text-processing"]
readme = "README.md"

[dependencies]
unicode-general-category = "1.0"  # Only real dependency!

[dev-dependencies]
criterion = "0.5"  # For benchmarks
```

### Step 6: Create Professional README.md
```markdown
# rustixml

A native Rust implementation of the iXML (Invisible XML) specification.

## Features

- 🚀 **Fast**: Native recursive descent parser, no translation overhead
- ✅ **Conformant**: 83.7% pass rate on official iXML test suite (41/49 tests)
- 🦀 **Pure Rust**: No external parser dependencies
- 📝 **Well-documented**: Comprehensive API docs and examples
- 🔧 **Small**: ~1100 LOC core implementation

## Quick Start

\`\`\`rust
use rustixml::parse_ixml_grammar;

let grammar = r#"
    greeting: "Hello, ", name, "!".
    name: ["A"-"Z"], ["a"-"z"]*.
"#;

let result = parse_ixml_grammar(grammar)?;
// ... parse input with grammar
\`\`\`

## Installation

\`\`\`toml
[dependencies]
rustixml = "0.2"
\`\`\`

## Status

**Production Ready** for most iXML grammars.

- ✅ All core iXML features (marks, repetitions, character classes)
- ✅ Unicode support (character ranges, hex literals)
- ✅ Separator operators (++, **)
- ⚠️ Left-recursive grammars not supported (3 tests)
- ⚠️ Ambiguous grammars return first parse only

See [docs/NATIVE_PARSER_STATUS.md](docs/NATIVE_PARSER_STATUS.md) for details.

## Why This Implementation?

This is a **native** iXML interpreter that directly implements the specification,
rather than translating to an intermediate parser format. This approach provides:

- **Better performance**: 0 timeouts vs 19 timeouts in translation-based approach
- **Clearer errors**: Direct position-based error messages
- **Simpler code**: ~1100 LOC vs ~2500 LOC for translation approach

See [docs/PARSER_COMPARISON.md](docs/PARSER_COMPARISON.md) for detailed comparison.

## Documentation

- [Architecture Overview](docs/architecture.md)
- [iXML Reference](docs/ixml_reference.md)
- [API Documentation](docs/api.md)
- [Parser Comparison](docs/PARSER_COMPARISON.md)

## Contributing

Contributions welcome! See [CONTRIBUTING.md](CONTRIBUTING.md).

## License

Licensed under either of Apache License 2.0 or MIT license at your option.
```

### Step 7: Add CI/CD (.github/workflows/ci.yml)
```yaml
name: CI

on: [push, pull_request]

jobs:
  test:
    runs-on: ubuntu-latest
    steps:
      - uses: actions/checkout@v3
      - uses: actions-rs/toolchain@v1
        with:
          toolchain: stable
      - run: cargo test --all-features
      - run: cargo test --release  # Run conformance tests

  lint:
    runs-on: ubuntu-latest
    steps:
      - uses: actions/checkout@v3
      - uses: actions-rs/toolchain@v1
        with:
          toolchain: stable
          components: clippy, rustfmt
      - run: cargo clippy -- -D warnings
      - run: cargo fmt -- --check
```

### Step 8: Add Examples

**examples/basic_usage.rs**:
```rust
//! Basic iXML parsing example

use rustixml::{parse_ixml_grammar, NativeParser};

fn main() {
    let grammar = r#"
        greeting: "Hello, ", name, "!".
        name: letter+.
        letter: ["A"-"Z"; "a"-"z"].
    "#;
    
    let parsed = parse_ixml_grammar(grammar).expect("Invalid grammar");
    let parser = NativeParser::new(parsed);
    
    let input = "Hello, World!";
    match parser.parse(input) {
        Ok(xml) => println!("Parsed XML:\n{}", xml),
        Err(e) => eprintln!("Parse error: {}", e),
    }
}
```

## Timeline & Action Items

### Week 1: Archive & Cleanup
- [ ] Create `archive/experimental` branch with full history
- [ ] Create clean `main` branch from orphan
- [ ] Remove all Earley code and debug binaries
- [ ] Clean up Cargo.toml dependencies
- [ ] Update .gitignore

### Week 2: Documentation
- [ ] Write professional README.md
- [ ] Create CONTRIBUTING.md
- [ ] Consolidate docs/ to 4-5 essential files
- [ ] Write API documentation
- [ ] Create CHANGELOG.md

### Week 3: Polish
- [ ] Add 3-5 examples/
- [ ] Add benchmarks/
- [ ] Set up CI/CD
- [ ] Add badges to README
- [ ] Create GitHub repository description

### Week 4: Public Release
- [ ] Make repository public
- [ ] Post on /r/rust
- [ ] Post on Hacker News
- [ ] Tweet about it
- [ ] Submit to This Week in Rust

## Decision Matrix

| Criterion | Option 1 (Clean) | Option 2 (Two-Track) | Option 3 (History) |
|-----------|------------------|----------------------|-------------------|
| First Impression | ⭐⭐⭐⭐⭐ | ⭐⭐⭐ | ⭐⭐⭐⭐ |
| Maintainability | ⭐⭐⭐⭐⭐ | ⭐⭐ | ⭐⭐⭐⭐ |
| Educational Value | ⭐⭐⭐ | ⭐⭐⭐⭐ | ⭐⭐⭐⭐⭐ |
| Code Size | ⭐⭐⭐⭐⭐ | ⭐⭐ | ⭐⭐⭐⭐ |
| Clarity | ⭐⭐⭐⭐⭐ | ⭐⭐⭐ | ⭐⭐⭐⭐ |
| **Total** | **23** | **14** | **21** |

## Final Recommendation

**Go with Option 1: Clean Slate** 🎯

**Reasoning**:
1. You want GitHub impact → clean code attracts stars
2. You want contributors → approachable codebase matters
3. History is preserved → archive branch keeps everything
4. Native parser is the winner → no reason to maintain Earley
5. Educational value → your PARSER_COMPARISON.md tells the story

**Next Steps**:
1. Review this plan
2. Decide on timeline
3. Start with Week 1 tasks
4. I'll help with documentation and examples

The experimental history isn't lost - it's just archived. Anyone interested can explore `archive/experimental` branch to see the full journey. But new users see a clean, professional implementation that solves their problem.

**Make it public with confidence!** 🚀
