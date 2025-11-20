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
//! - ✅ 83.7% conformance with iXML specification (41/49 tests)
//! - 🌐 WebAssembly support for browser use
//! - 📦 Single dependency (unicode-general-category)
//! - 🔒 Pure safe Rust

pub mod ast;
pub mod lexer;
pub mod grammar_parser;
pub mod grammar_ast;
pub mod input_stream;
pub mod native_parser;
pub mod parse_context;
pub mod xml_node;
pub mod charclass;

// WASM bindings (only when compiling for wasm32)
#[cfg(target_arch = "wasm32")]
pub mod wasm;

// Re-export main API
pub use grammar_ast::parse_ixml_grammar;
pub use native_parser::NativeParser;
pub use parse_context::{ParseContext, ParseError, ParseResult};
pub use ast::IxmlGrammar;

// Re-export WASM API for convenience
#[cfg(target_arch = "wasm32")]
pub use wasm::*;
