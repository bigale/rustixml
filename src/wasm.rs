//! WebAssembly bindings for rustixml
//!
//! This module provides JavaScript-friendly bindings for the iXML parser.
//! It's only compiled when targeting wasm32.

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
