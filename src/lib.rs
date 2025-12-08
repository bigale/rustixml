//! rustixml - Native iXML Parser
//!
//! A pure Rust implementation of the Invisible XML (iXML) specification.
//! Works natively in Rust and compiles to WebAssembly for browser use.
//!
//! # Quick Start
//!
//! ```rust
//! use rustixml::{parse_ixml_grammar, NativeParser};
//!
//! let grammar = r#"
//!     greeting: "Hello, ", name, "!".
//!     name: letter+.
//!     letter: ["A"-"Z"; "a"-"z"].
//! "#;
//!
//! let ast = parse_ixml_grammar(grammar).expect("Invalid grammar");
//! let parser = NativeParser::new(ast);
//!
//! let xml = parser.parse("Hello, World!").expect("Parse failed");
//! println!("{}", xml);
//! ```
//!
//! # Features
//!
//! - 🚀 Fast native recursive descent parser
//! - ✅ 75.4% conformance with iXML specification (49/65 tests)
//! - 🌐 WebAssembly support for browser use
//! - 📦 Single dependency (unicode-general-category)
//! - 🔒 Pure safe Rust

pub mod ast;
pub mod charclass;
pub mod grammar_analysis;
pub mod grammar_ast;
pub mod grammar_parser;
pub mod input_stream;
pub mod lexer;
pub mod native_parser;
pub mod normalize;
pub mod parse_context;
pub mod xml_node;

// WASM bindings (only when compiling for wasm32 browser/Node.js, not IC canisters)
#[cfg(all(target_arch = "wasm32", not(feature = "ic-canister")))]
pub mod wasm;

// Re-export main API
pub use ast::IxmlGrammar;
pub use grammar_ast::parse_ixml_grammar;
pub use native_parser::NativeParser;
pub use parse_context::{ParseContext, ParseError, ParseResult};

// Re-export WASM API for convenience (only for browser/Node.js WASM, not IC canisters)
#[cfg(all(target_arch = "wasm32", not(feature = "ic-canister")))]
pub use wasm::*;
