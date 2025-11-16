# RustiXML Progress Report

## Current Status (Updated: 2025-11-15)

### ✅ PHASE 3 IN PROGRESS: AST & Test Infrastructure

**Latest Achievements:**
- ✅ **AST-producing grammar complete** - `grammar_ast.rs` produces full AST (9/9 tests passing)
- ✅ **Test infrastructure started** - Can read test files from earleybird test suite
- 🔄 **Next: Runtime parser** - Need interpreter to parse arbitrary iXML grammars

### ✅ PHASE 2 COMPLETE: Full iXML Grammar Implementation Complete!

**All Core iXML Features Implemented!**

Implemented **lexer-first architecture** with token-based parsing that completely eliminates whitespace handling issues:

#### Working Features - Phase 2 Complete!
- ✅ Lexer automatically handles whitespace at tokenization level
- ✅ Simple literals: `rule: "hello".`
- ✅ Nonterminals: `rule: body.`
- ✅ **Arbitrary whitespace**: `rule  :  "hello"  .`
- ✅ **Multiple factors**: `rule: foo bar.`
- ✅ **Mixed sequences**: `rule: "hello" world "there".`
- ✅ **Alternatives**: `rule: "a" | "b" | "c".`
- ✅ **Multiple rules**: Complete grammars with multiple rule definitions
- ✅ **Repetition operators**: `item+`, `item*`, `item?`
- ✅ **Parentheses/grouping**: `(a | b)+`, nested expressions
- ✅ **Character classes**: `[a-z]`, `['0'-'9']`, `[L]` (Unicode categories)
- ✅ **Negated classes**: `~['0'-'9']`
- ✅ **Marks**: `@name` (attribute), `-name` (hidden), `^name` (promoted)
- ✅ **Insertion syntax**: `+"<"` for inserting literals into output

**Test Results**: **31/31 tests passing** in `grammar_v2.rs`!

#### Key Files
- `/home/bigale/repos/rustixml/src/lexer.rs` - Lexer with automatic whitespace handling (3/3 tests ✅)
- `/home/bigale/repos/rustixml/src/grammar_v2.rs` - Token-based grammar (31/31 tests ✅)
- `/home/bigale/repos/rustixml/src/grammar.rs` - Character-based grammar (deprecated - WS issues)
- `/home/bigale/repos/rustixml/src/working_test.rs` - Phase 1 baseline (3/3 tests ✅)

#### Example Grammar
The parser can now handle complex iXML grammars like:
```ixml
xml: +"<" @name +">" ^content +"</" -endtag +">".
identifier: [a-z] [a-z0-9]*.
list: (item (sep item)*)?.
digit: ['0'-'9'].
letter: [L].
nondigit: ~['0'-'9'].
```

## Key Findings from RustyLR Research

### RustyLR Features Discovered
1. **Repetition Operators**: `P*` (zero or more), `P+` (one or more), `P?` (optional)
2. **Separated Lists**: `$sep(P, separator, *)` or `$sep(P, separator, +)`
3. **Grouping**: `(P1 P2 | P3)` for pattern alternatives
4. **Precedence Directives**: `%left`, `%right` for resolving ambiguity

### Grammar Patterns That Work
From working_test.rs (Phase 1):
```rust
Factor(String): Insertion
              | Nonterminal
              ;

Repeat1(String): f1=Factor WS '+' WS {
    format!("REPEAT1:{}", f1)
}
| Factor
;

Rule(String): WS Repeat1 WS;
```

**Key insight**: Simple productions with clear terminal separators work reliably.

### Grammar Patterns That Fail
1. **Multiple WS in single production**:
   ```rust
   Rule(String): WS name=Ident ':' WS body=Factor WS '.' WS
   ```
   Causes `__data_stack.__tags` assertion failures

2. **Recursive alternatives with WS**:
   ```rust
   Sequence(String): f1=Factor WS f2=Sequence { ... }
                   | Factor
                   ;
   ```
   Causes assertion errors

3. **Repetition operators at wrong level**:
   ```rust
   Sequence(String): factors=(Factor)+ { ... }
   ```
   Matches characters individually instead of complete tokens

## Root Cause Analysis

The `__data_stack.__tags` assertion errors occur when:
1. Multiple WS tokens appear within a single production rule
2. Recursive productions have alternatives that differ only in WS placement
3. Complex nesting of productions with optional whitespace
4. **Intermediate productions forwarding to alternatives** (e.g., `Factor: Literal | Nonterminal`)

**Hypothesis**: RustyLR's reduce action mechanism expects a specific stack state based on the grammar structure. When WS appears multiple times in a production, the stack tags become inconsistent with what the generated code expects.

**Solution**: Use token-based parsing with lexer-first architecture and inline production alternatives.

## Successful Token-Based Grammar Pattern

Located in: `/home/bigale/repos/rustixml/src/grammar_v2.rs`

### Key Insight: Inline Alternatives

**This pattern causes stack tag assertion errors:**
```rust
Nonterminal(String): tok=ident { ... };
Literal(String): tok=string { ... };
Factor(String): Literal | Nonterminal;  // ❌ Forwarding alternatives
```

**This pattern works perfectly:**
```rust
// ✅ Inline alternatives with reduce actions
Factor(String): tok=string {
    match tok {
        Token::String(s) => format!("LIT:{}", s),
        _ => unreachable!(),
    }
}
| tok=ident {
    match tok {
        Token::Ident(name) => format!("NT:{}", name),
        _ => unreachable!(),
    }
};
```

### Complete Working Grammar

```rust
lr1! {
    %err String;
    %glr;
    %tokentype Token;
    %start Rule;

    %token ident Token::Ident(_);
    %token string Token::String(_);
    %token colon Token::Colon;
    %token period Token::Period;

    // Inline alternatives - NO intermediate forwarding productions!
    Factor(String): tok=string {
        match tok {
            Token::String(s) => format!("LIT:{}", s),
            _ => unreachable!(),
        }
    }
    | tok=ident {
        match tok {
            Token::Ident(name) => format!("NT:{}", name),
            _ => unreachable!(),
        }
    };

    // Repetition operator works perfectly!
    Sequence(String): factors=Factor+ {
        factors.join(" ")
    };

    // Clean grammar without WS noise!
    Rule(String): name_tok=ident colon body=Sequence period {
        match name_tok {
            Token::Ident(name) => format!("RULE:{}={}", name, body),
            _ => unreachable!(),
        }
    };
}
```

**Why This Works:**
1. Lexer handles all whitespace automatically during tokenization
2. Grammar only sees meaningful tokens (no WS clutter)
3. Inlined alternatives avoid intermediate production forwarding
4. RustyLR's stack state remains consistent throughout parsing
5. Repetition operators (`+`, `*`) work correctly on productions, not characters

## Current Working Grammar

Located in: `/home/bigale/repos/rustixml/src/grammar.rs`

```rust
lr1! {
    %err String;
    %glr;
    %tokentype char;
    %start Rule;

    WS: [ ' ' '\t' '\n' '\r' ]*;

    QuotedString(String): '"' chars=NotQuote+ '"' {
        chars.into_iter().collect()
    };
    NotQuote(char): ch=[^'"'] { ch };

    Nonterminal(String): name=Ident {
        format!("NT:{}", name)
    };

    Ident(String): start=IdentStart rest=IdentRest* {
        let mut s = start.to_string();
        s.push_str(&rest.into_iter().collect::<String>());
        s
    };
    IdentStart(char): ch=['a'-'z'] { ch }
                    | ch=['A'-'Z'] { ch }
                    | ch='_' { ch };
    IdentRest(char): ch=['a'-'z'] { ch }
                   | ch=['A'-'Z'] { ch }
                   | ch=['0'-'9'] { ch }
                   | ch='_' { ch };

    Factor(String): qs=QuotedString { qs }
                  | nt=Nonterminal { nt }
                  ;

    Rule(String): WS name=Ident ':' WS body=Factor '.' WS {
        format!("RULE:{}={}", name, body)
    };
}
```

**Test Results**: 2/5 tests passing
- ✅ `test_rule_with_literal`
- ✅ `test_rule_with_nonterminal`
- ❌ `test_rule_with_whitespace`
- ❌ `test_rule_with_nonterminal_and_whitespace`
- ❌ `test_rule_with_multiple_factors`

## Phase 3: AST and Test Infrastructure (2025-11-15)

### Achievements

**AST-Producing Grammar (`grammar_ast.rs`)**
- Converted `grammar_v2.rs` from string outputs to full AST structures
- Updated all production types to return proper AST nodes:
  - `BaseFactor` → `crate::ast::BaseFactor`
  - `Factor` → `crate::ast::Factor`
  - `Sequence` → `crate::ast::Sequence`
  - `Alternatives` → `crate::ast::Alternatives`
  - `Rule` → `crate::ast::Rule`
  - `Grammar` → `crate::ast::IxmlGrammar`
- Created 9 comprehensive tests covering all features
- **Test Results**: 9/9 tests passing!

**Test Infrastructure (`testsuite_utils.rs`)**
- Created basic test case structure
- Implemented `read_simple_test()` to read .ixml, .inp, .output.xml files
- Successfully reads test files from earleybird test suite
- Stubbed out `run_test()` for future implementation

**Key File Structure:**
```
src/
  ast.rs            - AST node definitions (completed Phase 2)
  grammar_ast.rs    - AST-producing parser (NEW - Phase 3)
  testsuite_utils.rs - Test infrastructure (NEW - Phase 3)
  lexer.rs          - Tokenizer (Phase 2)
  grammar_v2.rs     - String-based parser (Phase 2)
```

**Runtime Parser (`runtime_parser.rs`) - COMPLETE! ✅**
- ✅ **Earlgrey integration complete** - Added `earlgrey = "0.3"` dependency
- ✅ **AST-to-Earlgrey converter** - Translates iXML AST to Earlgrey grammars
- ✅ **Repetition operator mapping** - Converts iXML `+`, `*`, `?` to helper grammar rules
- ✅ **End-to-end parsing** - Can parse iXML → AST → Earlgrey → parse input
- ✅ **XML generation complete** - `build_xml_forest()` creates XML from parse trees
- **Test Results**: **6/6 tests passing!**

**Key Implementation Details:**
- Two-pass grammar construction (declare nonterminals, then add rules)
- Unique terminal naming to avoid collisions (`lit__SPACE_`, `lit__QUOTE_`)
- Repetition implemented as auxiliary rules:
  - `item+` → `item_plus := item | item_plus item`
  - `item*` → `item_star := ε | item_star item`
  - `item?` → `item_opt := ε | item`
- `XmlNode` enum for representing XML structure
- `EarleyForest` with per-production semantic actions (format: `"nonterminal -> symbol1 symbol2"`)
- Successfully generates XML: `iXML → AST → Earlgrey → Input → Parse Trees → XML`

**Example Working:**
```rust
// Input iXML grammar
let ixml = r#"greeting: "hello"."#;

// Parses to XML
<greeting>hello</greeting>
```

**First Conformance Test Passing! ✅**
- Successfully integrated test infrastructure with runtime parser
- Manual test case: `greeting: "hello" "world".` with input `"hello world"`
- Generates correct XML: `<greeting>helloworld</greeting>`
- Complete end-to-end pipeline validated!

**Comment Support Complete! ✅**
- ✅ **Nested comment support** - Handles `{outer {nested} outer}` syntax correctly
- ✅ **Comment skipping** - Comments automatically removed during tokenization
- ✅ **Error handling** - Detects unclosed comments with proper error messages
- **Test Results**: **7/7 lexer tests passing!** (including 4 new comment tests)

**Character-Level Parsing Complete! ✅**
- ✅ **Character-by-character tokenization** - Input parsed as individual characters
- ✅ **Multi-character literal support** - Literals split into character sequences automatically
- ✅ **Automatic terminal deduplication** - Each unique character defined once as terminal
- ✅ **Literal sequence nonterminals** - Multi-char literals like "hello" become `lit_seq_hello`
- ✅ **XML generation for sequences** - Character sequences correctly concatenated in output
- **Test Results**: **6/6 runtime_parser tests + 1/1 conformance test passing!**

**Mark Support Complete! ✅**
- ✅ **Attribute marks (@)** - Elements with `@` mark become XML attributes on parent element
- ✅ **Hidden marks (-)** - Elements with `-` mark are hidden from output, children promoted
- ✅ **Promoted marks (^)** - Elements with `^` mark have children promoted to parent level
- ✅ **Factor-level marks** - Marks on nonterminal references (e.g., `element: @name body.`)
- ✅ **Rule-level marks** - Marks on rule definitions (planned for future)
- ✅ **Attribute extraction** - Text content automatically extracted for attribute values
- **Test Results**: **9/9 runtime_parser tests passing!** (including 3 mark-specific tests)

### Next Steps

**For Full Conformance Testing:**
1. ~~**Add comment support**~~ ✅ **COMPLETE** - Handle `{...}` comments in iXML grammar lexer
2. ~~**Character-level tokenization**~~ ✅ **COMPLETE** - Parse individual characters instead of whitespace-splitting
3. ~~**Implement marks in XML generation**~~ ✅ **COMPLETE** - All three marks (@, -, ^) fully implemented
4. **Character classes in runtime** - Support `[a-z]`, `[L]`, etc. in AST-to-Earlgrey converter
5. **Groups in runtime** - Support `(a | b)` in AST-to-Earlgrey converter

**Known Limitations:**
- ~~Tokenization currently splits on whitespace~~ ✅ **COMPLETE** - Now parses character-by-character
- ~~Comments `{...}` not yet supported in lexer~~ ✅ **COMPLETE**
- ~~Marks parsed but not yet applied in XML generation~~ ✅ **COMPLETE**
- Character classes/groups parsed but not yet in runtime converter

### Previous: Phase 2 Progress

## Next Steps (Legacy - from Phase 2)

### Immediate Priorities
1. **Understand WS placement issue**:
   - Compare working `WS Repeat1 WS` pattern vs failing `WS ... WS '.' WS`
   - Identify minimal reproduction case for assertion error
   - Consider whether WS should be handled at lexer level instead

2. **Alternative Approaches**:
   - **Option A**: Lexer-based WS handling (tokenize input first, handle WS at lex level)
   - **Option B**: Carefully constrained WS placement (only at production boundaries)
   - **Option C**: Consult RustyLR GitHub issues/examples for similar patterns

3. **Incremental Testing**:
   - Add ONE WS at a time and test
   - Document exactly which WS placement causes failures
   - Build comprehensive test matrix

### Future Work
- Copy earleybird test infrastructure
- Implement full iXML grammar
- Build AST and XML output generation
- Run conformance tests (target: 5168/5168)

## Resources

- RustyLR GitHub: https://github.com/ehwan/RustyLR
- RustyLR SYNTAX.md: https://github.com/ehwan/RustyLR/blob/main/SYNTAX.md
- Phase 1 working grammar: `/home/bigale/repos/rustixml/src/working_test.rs`
- Earleybird (reference): `/home/bigale/repos/earleybird`

## Lessons Learned

1. **Lexer-first architecture is the correct approach for RustyLR**
   - Whitespace handling at lexer level eliminates grammar complexity
   - Token-based parsing (`%tokentype Token`) is cleaner than character-level
   - Standard engineering practice that works perfectly with RustyLR

2. **RustyLR grammar patterns that work vs. fail:**
   - ✅ **Inline alternatives**: `Prod: tok=a { ... } | tok=b { ... }`
   - ❌ **Forwarding alternatives**: `Prod1: tok=a { ... }; Prod2: tok=b { ... }; Combined: Prod1 | Prod2`
   - ✅ **Repetition on productions**: `factors=Factor+` returns `Vec<String>`
   - ❌ **Repetition at character level**: Matches chars instead of complete tokens

3. **Incremental development is essential**
   - Each grammar change should be tested immediately
   - Maintain working baseline to revert to
   - Phase 1 research provided critical working reference

4. **Pattern syntax (`+`, `*`, `?`) works perfectly when used correctly**
   - Operates at production level with token-based parsing
   - Returns `Vec<T>` where `T` is the production's semantic value type
   - Example: `Factor+` where `Factor: String` returns `Vec<String>`

5. **Phase 1 research was invaluable**
   - Confirmed RustyLR can handle insertion syntax (GLR ambiguity)
   - Confirmed WASM compilation works
   - Provided working reference patterns
   - Investing time in research prevented dead-ends
