//! WebAssembly bindings for rustixml
//!
//! This module provides JavaScript-friendly bindings for the iXML parser.
//! It's only compiled when targeting wasm32.

#![cfg(target_arch = "wasm32")]

use crate::{parse_ixml_grammar, NativeParser};
use wasm_bindgen::prelude::*;

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

    /// Get the number of rules in the grammar (for debugging)
    pub fn rule_count(&self) -> usize {
        self.parser.rule_count()
    }
}

/// Convenience function: parse grammar and input in one step
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

/// Get conformance information
#[wasm_bindgen]
pub fn conformance_info() -> String {
    "83.7% conformance (41/49 tests passing)".to_string()
}

// ============================================================================
// WASMZ Pattern: Functions that return HTML templates for htmz routing
// ============================================================================

/// Parse iXML and return HTML template (for WASMZ wasm:// routing)
#[wasm_bindgen]
pub fn parse_ixml_template(grammar: &str, input: &str) -> String {
    match IxmlParser::new(grammar) {
        Ok(parser) => match parser.parse(input) {
            ParseResult {
                success: true,
                output,
                ..
            } => {
                // Escape HTML for display
                let escaped = output
                    .replace('&', "&amp;")
                    .replace('<', "&lt;")
                    .replace('>', "&gt;");

                format!(
                    r#"
                    <div class="result success">
                        <h3>✅ Parse Successful</h3>
                        <div class="output-section">
                            <h4>XML Output:</h4>
                            <pre class="xml-output">{}</pre>
                        </div>
                        <div class="stats">
                            <span class="badge">Rules: {}</span>
                            <span class="badge">Length: {} chars</span>
                        </div>
                    </div>
                "#,
                    escaped,
                    parser.rule_count(),
                    output.len()
                )
            }
            ParseResult {
                error: Some(err), ..
            } => {
                format!(
                    r#"
                    <div class="result error">
                        <h3>❌ Parse Failed</h3>
                        <div class="error-section">
                            <pre class="error-message">{}</pre>
                        </div>
                    </div>
                "#,
                    err
                )
            }
            _ => String::from(r#"<div class="result error"><h3>Unknown error</h3></div>"#),
        },
        Err(e) => {
            format!(
                r#"
                <div class="result error">
                    <h3>❌ Grammar Error</h3>
                    <div class="error-section">
                        <pre class="error-message">{:?}</pre>
                    </div>
                </div>
            "#,
                e
            )
        }
    }
}

/// Load example grammar and input (returns HTML template)
#[wasm_bindgen]
pub fn load_example_template(example_name: &str) -> String {
    let (grammar, input, description) = match example_name {
        "simple" => (
            r#"sentence: word+.
word: letter+, -" "?.
letter: ["a"-"z"; "A"-"Z"]."#,
            "hello world",
            "Simple whitespace-separated words",
        ),
        "numbers" => (
            r#"number: digit+.
digit: ["0"-"9"]."#,
            "42",
            "Simple number parser",
        ),
        "date" => (
            r#"date: year, -"-", month, -"-", day.
year: digit, digit, digit, digit.
month: digit, digit.
day: digit, digit.
-digit: ["0"-"9"]."#,
            "2024-03-15",
            "ISO date format parser (YYYY-MM-DD)",
        ),
        _ => (
            r#"greeting: "Hello, ", name, "!".
name: letter+.
letter: ["A"-"Z"; "a"-"z"]."#,
            "Hello, World!",
            "Default greeting example",
        ),
    };

    // Simple JSON-like escaping for JavaScript strings
    let grammar_escaped = grammar
        .replace('\\', "\\\\")
        .replace('"', "\\\"")
        .replace('\n', "\\n")
        .replace('\r', "\\r");
    let input_escaped = input
        .replace('\\', "\\\\")
        .replace('"', "\\\"")
        .replace('\n', "\\n")
        .replace('\r', "\\r");

    format!(
        r#"
        <div class="example-loaded">
            <h4>📚 Loaded: {}</h4>
            <p>{}</p>
            <div class="example-content">
                <div class="grammar-preview">
                    <strong>Grammar:</strong>
                    <pre>{}</pre>
                </div>
                <div class="input-preview">
                    <strong>Input:</strong>
                    <pre>{}</pre>
                </div>
            </div>
            <script>
                // Update form fields
                document.getElementById('grammar').value = "{}";
                document.getElementById('input').value = "{}";
            </script>
        </div>
    "#,
        example_name, description, grammar, input, grammar_escaped, input_escaped
    )
}
