# Contributing to rustixml

Thank you for your interest in contributing to rustixml! This document provides guidelines and instructions for contributing.

## 🚀 Quick Start

### Prerequisites

- **Rust**: Install from [rustup.rs](https://rustup.rs/)
- **wasm-pack**: For WebAssembly builds
  ```bash
  cargo install wasm-pack
  ```
- **Python 3**: For running the local demo server

### Development Setup

1. **Clone the repository**:
   ```bash
   git clone https://github.com/bigale/rustixml.git
   cd rustixml
   ```

2. **Build the project**:
   ```bash
   cargo build
   ```

3. **Run tests**:
   ```bash
   cargo test
   ```

4. **Build WebAssembly**:
   ```bash
   wasm-pack build --target web --out-dir pkg
   ```

5. **Run local demo**:
   ```bash
   python3 -m http.server 8080
   # Visit: http://localhost:8080/www/
   ```

## 📋 Development Workflow

### Before Making Changes

1. **Create a branch**:
   ```bash
   git checkout -b feature/your-feature-name
   ```

2. **Check existing issues**: See if your change is already being discussed

3. **Open an issue first** for major changes to discuss the approach

### Making Changes

1. **Write tests** for new functionality
2. **Update documentation** if changing APIs
3. **Follow code style** (see below)
4. **Keep commits focused** - one logical change per commit

### Testing Your Changes

```bash
# Run all tests
cargo test

# Run specific test
cargo test test_name

# Run with verbose output
cargo test -- --nocapture

# Run conformance tests
cargo run --bin conformance_test

# Check formatting
cargo fmt -- --check

# Run linter
cargo clippy -- -D warnings

# Build WASM
wasm-pack build --target web --out-dir pkg

# Test WASM in browser
python3 -m http.server 8080
# Visit: http://localhost:8080/docs/wasmz.html
```

### Submitting Changes

1. **Commit with clear messages**:
   ```bash
   git commit -m "Add feature: brief description
   
   Detailed explanation of what changed and why.
   Fixes #issue-number"
   ```

2. **Push to your fork**:
   ```bash
   git push origin feature/your-feature-name
   ```

3. **Create a Pull Request**:
   - Use a clear title
   - Reference related issues
   - Describe what changed and why
   - Include test results if applicable

## 🎨 Code Style

### Rust Code

- **Follow Rustfmt**: Run `cargo fmt` before committing
- **Pass Clippy**: Run `cargo clippy` and fix warnings
- **Document public APIs**: Use `///` doc comments
- **Use descriptive names**: Prefer clarity over brevity
- **Keep functions focused**: One responsibility per function

Example:
```rust
/// Parse an iXML grammar into an AST
///
/// # Arguments
/// * `input` - The iXML grammar source code
///
/// # Returns
/// * `Ok(IxmlGrammar)` - Parsed grammar AST
/// * `Err(String)` - Parse error with message
///
/// # Example
/// ```
/// use rustixml::parse_ixml_grammar;
/// let ast = parse_ixml_grammar("rule: 'literal'.")?;
/// ```
pub fn parse_ixml_grammar(input: &str) -> Result<IxmlGrammar, String> {
    // Implementation
}
```

### Commit Messages

Follow [Conventional Commits](https://www.conventionalcommits.org/):

```
<type>(<scope>): <description>

[optional body]

[optional footer]
```

Types:
- `feat`: New feature
- `fix`: Bug fix
- `docs`: Documentation changes
- `style`: Code style changes (formatting, no logic change)
- `refactor`: Code refactoring
- `test`: Adding or updating tests
- `chore`: Maintenance tasks

Examples:
```
feat(parser): Add support for hex character codes

Implements #123. Adds parsing for hex character codes like #xFF.

fix(wasm): Handle empty input gracefully

Previously crashed on empty string. Now returns proper error.

docs(readme): Update WASM installation instructions
```

## 🐛 Reporting Bugs

### Before Reporting

1. **Search existing issues** - Your bug may already be reported
2. **Try latest version** - May already be fixed
3. **Minimal reproduction** - Simplify to smallest failing case

### Bug Report Template

```markdown
**Description**
Clear description of the bug

**To Reproduce**
1. Grammar: `rule: "literal".`
2. Input: `test`
3. Run: `cargo run -- parse grammar.ixml input.txt`
4. See error: ...

**Expected Behavior**
What should happen

**Actual Behavior**
What actually happens

**Environment**
- OS: Ubuntu 22.04
- Rust version: 1.75.0
- rustixml version: 0.2.0

**Additional Context**
Any other relevant information
```

## 💡 Feature Requests

### Before Requesting

1. **Check existing issues** - May already be planned
2. **Check iXML spec** - Ensure feature is in spec
3. **Consider scope** - Should it be in core or extension?

### Feature Request Template

```markdown
**Feature Description**
Clear description of the feature

**Use Case**
Why is this needed? What problem does it solve?

**Proposed Solution**
How might this be implemented?

**Alternatives Considered**
What other approaches could work?

**iXML Spec Reference**
Link to relevant spec section if applicable
```

## 📚 Areas for Contribution

### High Priority

- **Test coverage**: Add tests for uncovered code paths
- **iXML conformance**: Fix failing conformance tests
- **Documentation**: Improve examples, tutorials, API docs
- **Performance**: Profile and optimize hot paths

### Medium Priority

- **Error messages**: Make error messages more helpful
- **WASM features**: Add more WASMZ demo examples
- **Tooling**: Editor support, syntax highlighting
- **Examples**: Real-world grammar examples

### Good First Issues

Look for issues labeled `good-first-issue`:
- Simple bug fixes
- Documentation improvements
- Adding test cases
- Code cleanup

## 🔍 Code Review Process

### What We Look For

- **Correctness**: Does it work as intended?
- **Tests**: Are there tests? Do they pass?
- **Documentation**: Is it documented?
- **Style**: Does it follow project conventions?
- **Performance**: Are there obvious inefficiencies?
- **Backwards compatibility**: Does it break existing code?

### Review Timeline

- **Initial response**: Within 1-3 days
- **Full review**: Within 1 week
- **Merge decision**: After approval from maintainer

### Review Feedback

- Be patient - reviews take time
- Respond to feedback promptly
- Don't take criticism personally
- Ask questions if unclear

## 🏗️ Architecture Overview

### Project Structure

```
rustixml/
├── src/
│   ├── lib.rs              # Main library entry
│   ├── grammar_parser.rs   # iXML grammar parser
│   ├── grammar_ast.rs      # Grammar AST types
│   ├── native_parser.rs    # Runtime parser
│   ├── lexer.rs           # Lexical analyzer
│   ├── charclass.rs       # Character class handling
│   ├── wasm.rs            # WebAssembly bindings
│   └── bin/               # Binary executables
├── www/                   # Web demos
│   ├── index.html         # Standard demo
│   ├── htmz-standalone.html  # HTMZ pattern demo
│   └── wasmz.html         # WASMZ pattern demo
├── tests/                 # Integration tests
├── ixml_tests/           # iXML conformance tests
└── docs/                 # Documentation
```

### Key Concepts

1. **Two-Phase Parsing**:
   - Phase 1: Parse iXML grammar → AST
   - Phase 2: Use AST to parse input → XML

2. **Native Parser**:
   - Recursive descent implementation
   - Direct interpretation of grammar AST
   - No intermediate PEG/CFG conversion

3. **WASM Integration**:
   - Template-returning functions
   - wasm:// URL routing
   - Zero-copy where possible

## 📖 Additional Resources

- **iXML Specification**: [invisiblexml.org](https://invisiblexml.org)
- **Rust Book**: [doc.rust-lang.org/book](https://doc.rust-lang.org/book/)
- **WebAssembly**: [webassembly.org](https://webassembly.org)
- **WASMZ Pattern**: See `docs/WASMZ-PATTERN.md`

## 🤝 Code of Conduct

### Our Pledge

We are committed to providing a welcoming and inclusive environment for all contributors.

### Expected Behavior

- **Be respectful** of differing viewpoints
- **Be constructive** in criticism
- **Be professional** in communication
- **Be inclusive** and welcoming

### Unacceptable Behavior

- Harassment, discrimination, or trolling
- Personal attacks or insults
- Publishing private information
- Any conduct that creates an unwelcoming environment

### Enforcement

Violations can be reported to bigale@netzero.net. All reports will be reviewed and investigated.

## ❓ Getting Help

- **Questions**: Open a GitHub Discussion
- **Bugs**: Open a GitHub Issue
- **Chat**: (Add Discord/Slack link if available)
- **Email**: bigale@netzero.net (for sensitive matters)

## 📝 License

By contributing, you agree that your contributions will be licensed under the same dual MIT/Apache-2.0 license as the project.

---

**Thank you for contributing to rustixml!** 🎉
