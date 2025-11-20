# WASM Integration Plan for rustixml

## Overview

The native parser is **perfect for WASM** because:
- ✅ Only 1 dependency: `unicode-general-category` (WASM-compatible)
- ✅ No I/O operations (pure computation)
- ✅ ~1100 LOC - compiles to small WASM binary
- ✅ No threads/async (WASM-friendly)
- ✅ Already has `cdylib` configured

## Strategy: Multi-Target Build with Feature Gates

### Recommended Approach: Feature-Based WASM Support

**Why this works well:**
1. WASM is a **deployment target**, not a different implementation
2. Same parser code works native and in browser
3. Only need different bindings layer (wasm-bindgen)
4. Can optimize separately for each target

### Updated Cargo.toml

```toml
[package]
name = "rustixml"
version = "0.2.0"
edition = "2021"
authors = ["Alex Everitt <bigale@netzero.net>"]
description = "Native iXML parser - works in Rust and WebAssembly"
license = "MIT OR Apache-2.0"
repository = "https://github.com/bigale/rustixml"
keywords = ["parser", "xml", "ixml", "invisible-xml", "wasm"]
categories = ["parsing", "text-processing", "wasm"]
readme = "README.md"

[lib]
crate-type = ["lib", "cdylib", "rlib"]

[dependencies]
unicode-general-category = "1.0"

# WASM-specific dependencies (only when building for wasm)
[target.'cfg(target_arch = "wasm32")'.dependencies]
wasm-bindgen = "0.2"
serde = { version = "1.0", features = ["derive"] }
serde-wasm-bindgen = "0.6"

[target.'cfg(target_arch = "wasm32")'.dependencies.console_error_panic_hook]
version = "0.1"
optional = true

[target.'cfg(target_arch = "wasm32")'.dependencies.wee_alloc]
version = "0.4"
optional = true

[dev-dependencies]
criterion = "0.5"
wasm-bindgen-test = "0.3"

[features]
default = []
# Enable better panic messages in WASM (slightly larger binary)
console_error_panic_hook = ["dep:console_error_panic_hook"]
# Use smaller allocator in WASM (saves ~10KB)
wee_alloc = ["dep:wee_alloc"]

[profile.release]
opt-level = 3
lto = true
codegen-units = 1

# Optimize for small WASM binary size
[profile.release.package."*"]
opt-level = "z"  # Optimize for size
```

## Project Structure

```
rustixml/
├── src/
│   ├── lib.rs                  # Main library (works everywhere)
│   ├── ast.rs
│   ├── lexer.rs
│   ├── grammar_parser.rs
│   ├── native_parser.rs
│   ├── parse_context.rs
│   ├── input_stream.rs
│   └── wasm.rs                 # WASM bindings (feature-gated)
├── examples/
│   ├── basic_usage.rs          # Rust example
│   └── wasm_usage.html         # Browser example
├── www/                         # WASM demo website
│   ├── package.json
│   ├── webpack.config.js
│   ├── index.html
│   ├── index.js
│   └── styles.css
├── benches/
│   └── parsing.rs
└── tests/
    ├── conformance_tests.rs
    └── wasm_tests.rs           # WASM-specific tests
```

## Implementation

### 1. WASM Bindings (src/wasm.rs)

```rust
//! WebAssembly bindings for rustixml
//!
//! This module provides JavaScript-friendly bindings for the parser.

#![cfg(target_arch = "wasm32")]

use wasm_bindgen::prelude::*;
use crate::{parse_ixml_grammar, NativeParser};

// Set panic hook for better error messages in browser
#[cfg(feature = "console_error_panic_hook")]
pub fn set_panic_hook() {
    console_error_panic_hook::set_once();
}

// Use smaller allocator for WASM
#[cfg(feature = "wee_alloc")]
#[global_allocator]
static ALLOC: wee_alloc::WeeAlloc = wee_alloc::WeeAlloc::INIT;

/// Result type for JavaScript interop
#[wasm_bindgen]
#[derive(Debug)]
pub struct ParseResult {
    success: bool,
    output: String,
    error: Option<String>,
}

#[wasm_bindgen]
impl ParseResult {
    #[wasm_bindgen(getter)]
    pub fn success(&self) -> bool {
        self.success
    }

    #[wasm_bindgen(getter)]
    pub fn output(&self) -> String {
        self.output.clone()
    }

    #[wasm_bindgen(getter)]
    pub fn error(&self) -> Option<String> {
        self.error.clone()
    }
}

/// WASM-friendly iXML parser
#[wasm_bindgen]
pub struct IxmlParser {
    parser: NativeParser,
}

#[wasm_bindgen]
impl IxmlParser {
    /// Create a new parser from an iXML grammar
    #[wasm_bindgen(constructor)]
    pub fn new(grammar: &str) -> Result<IxmlParser, JsValue> {
        #[cfg(feature = "console_error_panic_hook")]
        set_panic_hook();

        let ast = parse_ixml_grammar(grammar)
            .map_err(|e| JsValue::from_str(&format!("Grammar parse error: {}", e)))?;
        
        Ok(IxmlParser {
            parser: NativeParser::new(ast),
        })
    }

    /// Parse input text according to the grammar
    pub fn parse(&self, input: &str) -> ParseResult {
        match self.parser.parse(input) {
            Ok(xml) => ParseResult {
                success: true,
                output: xml,
                error: None,
            },
            Err(e) => ParseResult {
                success: false,
                output: String::new(),
                error: Some(e.to_string()),
            },
        }
    }

    /// Get parser statistics (for debugging)
    pub fn stats(&self) -> String {
        format!("Rules: {}", self.parser.rule_count())
    }
}

/// Convenience function: parse in one step
#[wasm_bindgen]
pub fn parse_ixml(grammar: &str, input: &str) -> ParseResult {
    match IxmlParser::new(grammar) {
        Ok(parser) => parser.parse(input),
        Err(e) => ParseResult {
            success: false,
            output: String::new(),
            error: Some(format!("{:?}", e)),
        },
    }
}

/// Get version information
#[wasm_bindgen]
pub fn version() -> String {
    env!("CARGO_PKG_VERSION").to_string()
}
```

### 2. Update src/lib.rs

```rust
//! rustixml - Native iXML Parser
//!
//! A pure Rust implementation of the Invisible XML (iXML) specification.
//! Works natively in Rust and compiles to WebAssembly for browser use.

pub mod ast;
pub mod lexer;
pub mod grammar_parser;
pub mod grammar_ast;
pub mod input_stream;
pub mod native_parser;
pub mod parse_context;

// WASM bindings (only when compiling for wasm32)
#[cfg(target_arch = "wasm32")]
pub mod wasm;

// Re-export main API
pub use grammar_ast::parse_ixml_grammar;
pub use native_parser::NativeParser;
pub use parse_context::ParseContext;

// Re-export WASM API for convenience
#[cfg(target_arch = "wasm32")]
pub use wasm::*;
```

### 3. WASM Demo Website (www/package.json)

```json
{
  "name": "rustixml-demo",
  "version": "0.2.0",
  "description": "Interactive iXML parser demo",
  "scripts": {
    "build": "webpack --mode production",
    "dev": "webpack serve --mode development",
    "test": "cargo test --target wasm32-unknown-unknown"
  },
  "devDependencies": {
    "@wasm-tool/wasm-pack-plugin": "^1.7.0",
    "webpack": "^5.88.0",
    "webpack-cli": "^5.1.0",
    "webpack-dev-server": "^4.15.0",
    "html-webpack-plugin": "^5.5.0",
    "css-loader": "^6.8.0",
    "style-loader": "^3.3.0"
  }
}
```

### 4. WASM Demo Website (www/index.html)

```html
<!DOCTYPE html>
<html>
<head>
    <meta charset="utf-8">
    <title>rustixml - Interactive iXML Parser</title>
    <meta name="viewport" content="width=device-width, initial-scale=1">
    <style>
        body {
            font-family: -apple-system, BlinkMacSystemFont, "Segoe UI", Roboto, sans-serif;
            max-width: 1400px;
            margin: 0 auto;
            padding: 20px;
            background: #f5f5f5;
        }
        .container {
            display: grid;
            grid-template-columns: 1fr 1fr;
            gap: 20px;
            margin-top: 20px;
        }
        .panel {
            background: white;
            border-radius: 8px;
            padding: 20px;
            box-shadow: 0 2px 4px rgba(0,0,0,0.1);
        }
        textarea {
            width: 100%;
            height: 300px;
            font-family: 'Monaco', 'Courier New', monospace;
            font-size: 14px;
            border: 1px solid #ddd;
            border-radius: 4px;
            padding: 10px;
            resize: vertical;
        }
        button {
            background: #0066cc;
            color: white;
            border: none;
            padding: 10px 20px;
            border-radius: 4px;
            cursor: pointer;
            font-size: 16px;
            margin-top: 10px;
        }
        button:hover {
            background: #0052a3;
        }
        .output {
            background: #f8f9fa;
            border: 1px solid #ddd;
            border-radius: 4px;
            padding: 10px;
            min-height: 300px;
            font-family: 'Monaco', 'Courier New', monospace;
            font-size: 14px;
            white-space: pre-wrap;
            overflow-x: auto;
        }
        .error {
            color: #d32f2f;
            background: #ffebee;
            padding: 10px;
            border-radius: 4px;
            margin-top: 10px;
        }
        .success {
            color: #2e7d32;
            background: #e8f5e9;
            padding: 10px;
            border-radius: 4px;
            margin-top: 10px;
        }
        h1 {
            color: #333;
        }
        h2 {
            color: #666;
            font-size: 18px;
            margin-top: 0;
        }
        .examples {
            margin-top: 10px;
        }
        .example-btn {
            background: #6c757d;
            padding: 5px 10px;
            font-size: 14px;
            margin-right: 5px;
        }
        .stats {
            font-size: 12px;
            color: #666;
            margin-top: 10px;
        }
    </style>
</head>
<body>
    <h1>🦀 rustixml - Interactive iXML Parser</h1>
    <p>A pure Rust iXML parser running entirely in your browser via WebAssembly. <strong>83.7% conformance</strong> with the iXML specification.</p>

    <div class="container">
        <div class="panel">
            <h2>iXML Grammar</h2>
            <textarea id="grammar" placeholder="Enter your iXML grammar here...">greeting: "Hello, ", name, "!".
name: letter+.
letter: ["A"-"Z"; "a"-"z"].</textarea>
            
            <div class="examples">
                <button class="example-btn" onclick="loadExample('greeting')">Greeting</button>
                <button class="example-btn" onclick="loadExample('json')">JSON</button>
                <button class="example-btn" onclick="loadExample('csv')">CSV</button>
                <button class="example-btn" onclick="loadExample('arithmetic')">Arithmetic</button>
            </div>
        </div>

        <div class="panel">
            <h2>Input Text</h2>
            <textarea id="input" placeholder="Enter text to parse...">Hello, World!</textarea>
            <button onclick="parseInput()">Parse ▶</button>
        </div>
    </div>

    <div class="panel" style="margin-top: 20px;">
        <h2>XML Output</h2>
        <div id="output" class="output">Click "Parse" to see XML output...</div>
        <div id="status"></div>
        <div id="stats" class="stats"></div>
    </div>

    <script src="./index.js"></script>
</body>
</html>
```

### 5. WASM Demo JavaScript (www/index.js)

```javascript
import * as wasm from '../pkg/rustixml';

// Initialize when WASM loads
console.log('rustixml version:', wasm.version());

// Example grammars
const examples = {
    greeting: {
        grammar: `greeting: "Hello, ", name, "!".
name: letter+.
letter: ["A"-"Z"; "a"-"z"].`,
        input: "Hello, World!"
    },
    json: {
        grammar: `value: object | array | string | number | "true" | "false" | "null".
object: "{", (pair, (",", pair)*)?, "}".
pair: string, ":", value.
array: "[", (value, (",", value)*)?, "]".
string: '"', char*, '"'.
char: ~['"'].
number: digit+.
digit: ["0"-"9"].`,
        input: '{"name": "Alice", "age": 30}'
    },
    csv: {
        grammar: `csv: row+.
row: field, (",", field)*, -#a.
field: quoted | unquoted.
quoted: '"', qchar*, '"'.
qchar: ~['"'].
unquoted: uchar*.
uchar: ~[","; #a].`,
        input: 'Name,Age,City\nAlice,30,"New York"\nBob,25,Boston'
    },
    arithmetic: {
        grammar: `expr: term, (addop, term)*.
term: factor, (mulop, factor)*.
factor: number | "(", expr, ")".
number: digit+.
digit: ["0"-"9"].
addop: "+" | "-".
mulop: "*" | "/".`,
        input: "3 + 4 * (2 + 5)"
    }
};

window.loadExample = function(name) {
    const example = examples[name];
    document.getElementById('grammar').value = example.grammar;
    document.getElementById('input').value = example.input;
};

window.parseInput = function() {
    const grammar = document.getElementById('grammar').value;
    const input = document.getElementById('input').value;
    const outputDiv = document.getElementById('output');
    const statusDiv = document.getElementById('status');
    const statsDiv = document.getElementById('stats');

    const startTime = performance.now();

    try {
        const result = wasm.parse_ixml(grammar, input);
        const endTime = performance.now();
        const duration = (endTime - startTime).toFixed(2);

        if (result.success) {
            outputDiv.textContent = result.output;
            statusDiv.innerHTML = `<div class="success">✓ Parsed successfully in ${duration}ms</div>`;
        } else {
            outputDiv.textContent = '';
            statusDiv.innerHTML = `<div class="error">✗ Parse error: ${result.error}</div>`;
        }

        statsDiv.textContent = `Parse time: ${duration}ms | WASM binary running in browser`;
    } catch (e) {
        outputDiv.textContent = '';
        statusDiv.innerHTML = `<div class="error">✗ Error: ${e.message}</div>`;
        statsDiv.textContent = '';
    }
};
```

### 6. Build Scripts

**Makefile** (or just scripts in README):
```makefile
# Build WASM package
.PHONY: wasm
wasm:
	wasm-pack build --target web --out-dir www/pkg

# Build and run WASM demo
.PHONY: demo
demo: wasm
	cd www && npm install && npm run dev

# Build optimized WASM for production
.PHONY: wasm-release
wasm-release:
	wasm-pack build --target web --release --out-dir www/pkg
	cd www && npm run build

# Test WASM build
.PHONY: test-wasm
test-wasm:
	wasm-pack test --headless --firefox

# Build everything
.PHONY: all
all: test wasm demo
```

## Build & Deploy Instructions

### For Rust Developers

```bash
# Normal Rust usage (no WASM needed)
cargo add rustixml
```

```rust
use rustixml::{parse_ixml_grammar, NativeParser};

let grammar = r#"greeting: "Hello, ", name, "!"."#;
let parser = NativeParser::new(parse_ixml_grammar(grammar)?);
let xml = parser.parse("Hello, World!")?;
```

### For JavaScript/TypeScript Developers

```bash
# Install from npm (you'll publish to npm)
npm install rustixml
```

```javascript
import * as rustixml from 'rustixml';

const result = rustixml.parse_ixml(grammar, input);
if (result.success) {
    console.log(result.output);
} else {
    console.error(result.error);
}
```

### Building WASM Locally

```bash
# 1. Install wasm-pack
curl https://rustwasm.github.io/wasm-pack/installer/init.sh -sSf | sh

# 2. Build WASM package
wasm-pack build --target web

# 3. Run demo
cd www
npm install
npm run dev
# Open http://localhost:8080
```

### Deploy to GitHub Pages

```bash
# Build optimized WASM
wasm-pack build --release --target web --out-dir www/pkg
cd www
npm run build

# Deploy dist/ folder to GitHub Pages
# (GitHub Actions can automate this)
```

## Updated README Sections

Add to main README.md:

```markdown
## Installation

### Rust
\`\`\`toml
[dependencies]
rustixml = "0.2"
\`\`\`

### JavaScript/WebAssembly
\`\`\`bash
npm install rustixml
\`\`\`

\`\`\`javascript
import * as rustixml from 'rustixml';

const result = rustixml.parse_ixml(grammar, input);
if (result.success) {
    console.log(result.output);
}
\`\`\`

### Try it Online
🌐 **[Live Demo](https://bigale.github.io/rustixml/)** - Run the parser entirely in your browser!

## Features

- 🚀 **Fast**: Native Rust performance
- 🌐 **Cross-platform**: Works in Rust, WebAssembly, and Node.js
- ✅ **83.7% Conformant**: Passes 41/49 official iXML tests
- 📦 **Tiny**: ~100KB WASM binary (gzipped: ~30KB)
- 🔒 **Safe**: Pure Rust with no unsafe code
```

## CI/CD for WASM

**`.github/workflows/wasm.yml`**:
```yaml
name: WASM Build and Deploy

on:
  push:
    branches: [main]
  pull_request:
    branches: [main]

jobs:
  build-wasm:
    runs-on: ubuntu-latest
    steps:
      - uses: actions/checkout@v3
      
      - name: Install Rust
        uses: actions-rs/toolchain@v1
        with:
          toolchain: stable
          target: wasm32-unknown-unknown
      
      - name: Install wasm-pack
        run: curl https://rustwasm.github.io/wasm-pack/installer/init.sh -sSf | sh
      
      - name: Build WASM
        run: wasm-pack build --target web --release
      
      - name: Test WASM
        run: wasm-pack test --headless --firefox
      
      - name: Build demo site
        run: |
          cd www
          npm install
          npm run build
      
      - name: Deploy to GitHub Pages
        if: github.ref == 'refs/heads/main'
        uses: peaceiris/actions-gh-pages@v3
        with:
          github_token: ${{ secrets.GITHUB_TOKEN }}
          publish_dir: ./www/dist
```

## Binary Size Optimization

The WASM binary will be approximately:
- **Unoptimized**: ~200KB
- **Optimized (opt-level="z")**: ~100KB
- **Gzipped**: ~30KB

This is excellent for a parser! For comparison:
- PEG.js: ~80KB (but less capable)
- nearley: ~50KB (but requires grammar precompilation)
- Your parser: ~30KB gzipped, **works with runtime grammars**, 83.7% conformance

## Publishing

### To crates.io
```bash
cargo publish
```

### To npm
```bash
wasm-pack build --target web --release
wasm-pack publish
```

## Summary

**WASM Integration Strategy**:
1. ✅ **Same codebase** - no duplication
2. ✅ **Feature-gated** - WASM deps only when building for wasm32
3. ✅ **Thin bindings** - JavaScript-friendly API in src/wasm.rs
4. ✅ **Live demo** - Interactive website on GitHub Pages
5. ✅ **Dual publishing** - crates.io (Rust) + npm (JavaScript)

**Benefits**:
- Write once, run everywhere (Rust + Browser + Node.js)
- Single source of truth (no separate implementations)
- Automatic browser support attracts more users
- Live demo is great for marketing
- npm package opens JavaScript ecosystem

**Next Steps**:
1. Implement src/wasm.rs (WASM bindings)
2. Create www/ demo site
3. Set up GitHub Actions for auto-deploy
4. Test WASM build thoroughly
5. Publish to both crates.io and npm

Would you like me to start implementing the WASM bindings? We can have a working browser demo in about 30 minutes! 🚀
