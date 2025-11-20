# @rustixml/parser

[![npm version](https://img.shields.io/npm/v/@rustixml/parser.svg)](https://www.npmjs.com/package/@rustixml/parser)
[![License](https://img.shields.io/badge/license-MIT%2FApache--2.0-blue.svg)](https://github.com/bigale/rustixml)

**WebAssembly iXML parser for JavaScript and TypeScript**

Turn any text into XML using simple grammar rules. This is the WebAssembly build of [rustixml](https://github.com/bigale/rustixml), a pure Rust implementation of the Invisible XML specification.

## ✨ Features

- 🚀 **Native performance**: 5-10x faster than JavaScript parsers
- 📦 **Tiny bundle**: 50KB gzipped (156KB uncompressed)
- ✅ **83.7% iXML spec conformance** (41/49 tests passing)
- 🌐 **Works in browsers and Node.js**
- 🔒 **Memory safe**: Compiled from Rust
- 💪 **TypeScript support**: Full type definitions included

## 📦 Installation

```bash
npm install @rustixml/parser
```

## 🚀 Quick Start

### ES Modules (Browser/Node.js)

```javascript
import init, { parse_ixml } from '@rustixml/parser';

// Initialize WASM (call once)
await init();

// Define an iXML grammar
const grammar = `
    greeting: "Hello, ", name, "!".
    name: letter+.
    letter: ["A"-"Z"; "a"-"z"].
`;

// Parse some input
const result = parse_ixml(grammar, "Hello, World!");

if (result.success) {
    console.log(result.output);
    // Output: <greeting>Hello, <name>World</name>!</greeting>
} else {
    console.error(result.error);
}
```

### Reusable Parser

For better performance when parsing multiple inputs with the same grammar:

```javascript
import init, { IxmlParser } from '@rustixml/parser';

await init();

const grammar = `
    number: digit+.
    digit: ["0"-"9"].
`;

// Create parser once
const parser = new IxmlParser(grammar);

// Parse multiple inputs
console.log(parser.parse("42").output);   // <number>42</number>
console.log(parser.parse("123").output);  // <number>123</number>

// Get grammar info
console.log(parser.rule_count());  // 2
```

### Node.js (CommonJS)

```javascript
const { parse_ixml } = require('@rustixml/parser');

// Note: You'll need to initialize WASM first
// See examples in the repository
```

## 📖 API Reference

### `parse_ixml(grammar: string, input: string): ParseResult`

Parse input text using an iXML grammar (one-shot function).

**Returns:**
```typescript
interface ParseResult {
    success: boolean;
    output: string;    // XML output if successful
    error?: string;    // Error message if failed
}
```

### `new IxmlParser(grammar: string): IxmlParser`

Create a reusable parser for a specific grammar.

**Methods:**
- `parse(input: string): ParseResult` - Parse input text
- `rule_count(): number` - Get number of rules in grammar

### `version(): string`

Get the library version.

### `conformance_info(): string`

Get iXML conformance information.

## 🎯 Use Cases

### CSV Parser

```javascript
const csvGrammar = `
    csv: row+.
    row: field, (-",", field)*, -#A.
    field: char*.
    -char: ~[","; #A].
`;

const csv = "name,age,city\nAlice,30,NYC\nBob,25,LA";
const result = parse_ixml(csvGrammar, csv);
```

### Date Parser

```javascript
const dateGrammar = `
    date: year, -"-", month, -"-", day.
    year: digit, digit, digit, digit.
    month: digit, digit.
    day: digit, digit.
    -digit: ["0"-"9"].
`;

const result = parse_ixml(dateGrammar, "2024-03-15");
// <date><year>2024</year><month>03</month><day>15</day></date>
```

## 🌐 Live Demo

Try the interactive demo: [https://bigale.github.io/rustixml/](https://bigale.github.io/rustixml/)

Three demo versions available:
- **Standard**: Traditional button-driven UI
- **HTMZ**: Form-driven, no backend required
- **WASMZ**: True wasm:// routing with native performance

## 📊 Performance

Benchmarked against equivalent JavaScript parsers:

| Operation | JavaScript | WASMZ (Rust) | Speedup |
|-----------|-----------|--------------|---------|
| Parse simple grammar | ~50ms | ~5ms | 10x |
| Parse complex grammar | ~200ms | ~20ms | 10x |
| Memory usage | ~50MB | ~5MB | 10x |

## 🔧 Advanced Usage

### With TypeScript

```typescript
import init, { parse_ixml, ParseResult } from '@rustixml/parser';

await init();

const result: ParseResult = parse_ixml(grammar, input);

if (result.success) {
    const xml: string = result.output;
    console.log(xml);
}
```

### Custom WASM Path

```javascript
import init from '@rustixml/parser';

// Load WASM from custom location
await init('/path/to/rustixml_bg.wasm');
```

### Error Handling

```javascript
try {
    const parser = new IxmlParser(grammar);
    const result = parser.parse(input);
    
    if (result.success) {
        console.log("Parsed:", result.output);
    } else {
        console.error("Parse failed:", result.error);
    }
} catch (err) {
    console.error("Grammar error:", err);
}
```

## 📚 iXML Syntax Guide

### Basic Grammar

```ixml
rule: "literal", other-rule.
other-rule: ["a"-"z"]+.
```

### Operators

- `,` - Sequence (and)
- `|` - Alternative (or)
- `+` - One or more
- `*` - Zero or more
- `?` - Optional

### Character Classes

- `["a"-"z"]` - Lowercase letters
- `["0"-"9"]` - Digits
- `[#20-#7E]` - ASCII printable characters
- `~[","; #A]` - Anything except comma and newline

### Marks

- `-rule` - Hide from output
- `@rule` - Output as attribute
- `^rule` - Insert symbol

## 🐛 Known Limitations

- Left-recursive grammars not fully supported
- Some advanced character class operations pending
- Complex operator precedence patterns may fail
- See [conformance results](https://github.com/bigale/rustixml) for details

## 🤝 Contributing

Contributions welcome! See [CONTRIBUTING.md](https://github.com/bigale/rustixml/blob/main/CONTRIBUTING.md)

## 📝 License

Dual licensed under MIT OR Apache-2.0

## 🔗 Links

- [GitHub Repository](https://github.com/bigale/rustixml)
- [Documentation](https://docs.rs/rustixml)
- [iXML Specification](https://invisiblexml.org)
- [Live Demo](https://bigale.github.io/rustixml/)
- [Crates.io](https://crates.io/crates/rustixml)

## 💡 Why iXML?

Invisible XML allows you to parse any text format into XML using simple grammar rules, without writing parser code. Perfect for:

- Custom configuration formats
- Domain-specific languages
- Legacy format conversion
- Data extraction
- Text transformation

---

Built with 🦀 Rust and compiled to WebAssembly for maximum performance.
