# rustixml Architecture Documentation

This document describes the complete processing pipeline for rustixml, an iXML (Invisible XML) parser implementation in Rust.

## Table of Contents

1. [High-Level Overview](#high-level-overview)
2. [Grammar Parsing Pipeline](#grammar-parsing-pipeline)
3. [Runtime Parsing Pipeline](#runtime-parsing-pipeline)
4. [XML Generation Pipeline](#xml-generation-pipeline)
5. [Semantic Actions System](#semantic-actions-system)
6. [Mark Processing](#mark-processing)
7. [Test Infrastructure](#test-infrastructure)

---

## High-Level Overview

```mermaid
flowchart TB
    subgraph Input
        G[iXML Grammar Text]
        I[Input Text]
    end

    subgraph GrammarParsing[Grammar Parsing Phase]
        L[Lexer]
        P[Recursive Descent Parser]
        AST[IxmlGrammar AST]
    end

    subgraph RuntimeConversion[Runtime Conversion Phase]
        C[AST to Earley Converter]
        GB[GrammarBuilder]
        EG[Earley Grammar]
    end

    subgraph InputParsing[Input Parsing Phase]
        EP[EarleyParser]
        PT[Parse Trees/State]
    end

    subgraph XMLGeneration[XML Generation Phase]
        EF[EarleyForest with Actions]
        XN[XmlNode Tree]
        XML[XML String Output]
    end

    G --> L
    L --> P
    P --> AST
    AST --> C
    C --> GB
    GB --> EG
    EG --> EP
    I --> EP
    EP --> PT
    AST --> EF
    PT --> EF
    EF --> XN
    XN --> XML

    style Input fill:#e1f5fe,color:#000
    style GrammarParsing fill:#f3e5f5,color:#000
    style RuntimeConversion fill:#fff3e0,color:#000
    style InputParsing fill:#e8f5e9,color:#000
    style XMLGeneration fill:#fce4ec,color:#000
```

**Test References:**
- All 49 tests in `/home/bigale/repos/ixml/tests/correct/` exercise this complete pipeline
- `arith`, `hash`, `test` - Simple grammars that test basic pipeline flow

---

## Grammar Parsing Pipeline

### Lexer Stage

The lexer (`src/lexer.rs`) tokenizes iXML grammar text into a stream of tokens.

```mermaid
flowchart LR
    subgraph Tokens
        ID[Identifier]
        LIT[Literal]
        MARK[Mark: @ - ^]
        REP[Repetition: + * ?]
        SEP[Separator: ++ **]
        HEX[Hex: #XX]
        CC[CharClass]
        PUNCT[Punctuation]
    end

    GT[Grammar Text] --> LEX[Lexer]
    LEX --> ID
    LEX --> LIT
    LEX --> MARK
    LEX --> REP
    LEX --> SEP
    LEX --> HEX
    LEX --> CC
    LEX --> PUNCT

    style Tokens fill:#fff9c4,color:#000
```

**Key Features:**
- Handles brace comments `{...}` including nested comments
- Supports hex characters `#a`, `#20`, etc.
- Recognizes double operators `++` and `**` for separated repetitions

**Test References:**
- `nested-comment` - Tests nested brace comment handling
- `hex`, `hex1`, `hex3` - Tests hex character lexing
- `lf`, `tab` - Tests special character handling

### Parser Stage

The handwritten recursive descent parser (`src/grammar_parser.rs`) builds the AST.

```mermaid
flowchart TB
    subgraph ParserMethods[Parser Methods]
        PG[parse_grammar]
        PR[parse_rule]
        PA[parse_alternatives]
        PS[parse_sequence]
        PF[parse_factor]
        PB[parse_base_factor]
    end

    subgraph ASTNodes[AST Node Types]
        IG[IxmlGrammar]
        R[Rule]
        A[Alternatives]
        S[Sequence]
        F[Factor]
        BF[BaseFactor]
    end

    PG --> PR
    PR --> PA
    PA --> PS
    PS --> PF
    PF --> PB

    PG --> IG
    PR --> R
    PA --> A
    PS --> S
    PF --> F
    PB --> BF

    style ParserMethods fill:#c8e6c9,color:#000
    style ASTNodes fill:#bbdefb,color:#000
```

**BaseFactor Variants:**

```mermaid
flowchart TB
    BF[BaseFactor]

    BF --> NT[Nonterminal]
    BF --> LIT[Literal]
    BF --> CC[CharClass]
    BF --> GRP[Group]

    NT --> NTF[name: String<br/>mark: Mark]
    LIT --> LITF[value: String<br/>insertion: bool<br/>mark: Mark]
    CC --> CCF[content: String<br/>negated: bool<br/>mark: Mark]
    GRP --> GRPF[alternatives: Alternatives]

    style BF fill:#ffecb3,color:#000
```

**Test References:**
- `marked` - Tests mark parsing (`@`, `-`, `^`)
- `string` - Tests literal parsing
- `range`, `ranges` - Tests character class parsing
- `empty-group` - Tests group parsing

---

## Runtime Parsing Pipeline

### AST to Earley Conversion

The converter (`src/runtime_parser.rs:ast_to_earlgrey`) transforms the iXML AST into an Earley grammar.

```mermaid
flowchart TB
    subgraph FirstPass[First Pass: Terminal Collection]
        CC1[Collect Character Classes]
        CT[Collect Character Terminals]
        ML[Collect Marked Literals]
    end

    subgraph SecondPass[Second Pass: Grammar Building]
        NTD[Declare Nonterminals]
        PRD[Add Productions]
        REP[Create Repetition Rules]
        GRP[Create Group Rules]
    end

    AST[IxmlGrammar AST] --> CC1
    CC1 --> CT
    CT --> ML
    ML --> NTD
    NTD --> PRD
    PRD --> REP
    REP --> GRP
    GRP --> GB[GrammarBuilder]

    style FirstPass fill:#e3f2fd,color:#000
    style SecondPass fill:#f1f8e9,color:#000
```

### Repetition Handling

```mermaid
flowchart TB
    subgraph RepTypes[Repetition Types]
        NONE[None]
        OPT[Optional: ?]
        STAR[ZeroOrMore: *]
        PLUS[OneOrMore: +]
        SEPSTAR[SeparatedZeroOrMore: **]
        SEPPLUS[SeparatedOneOrMore: ++]
    end

    subgraph GeneratedRules[Generated Helper Rules]
        OPT --> OPTR[name_opt -> name<br/>name_opt -> epsilon]
        STAR --> STARR[name_star -> name name_star<br/>name_star -> epsilon]
        PLUS --> PLUSR[name_plus -> name name_plus<br/>name_plus -> name]
        SEPSTAR --> SEPSTARR[name_sepstar -> name sep name_sepstar<br/>name_sepstar -> epsilon]
        SEPPLUS --> SEPPLUSR[name_sepplus -> name sep name_sepplus<br/>name_sepplus -> name]
    end

    style RepTypes fill:#fff3e0,color:#000
    style GeneratedRules fill:#fce4ec,color:#000
```

**Test References:**
- `hash` - Tests separated repetitions (`++`)
- `expr1` through `expr6` - Tests complex repetition patterns
- `para-test` - Tests `+` repetition

---

## XML Generation Pipeline

### EarleyForest Actions

The forest (`src/runtime_parser.rs:build_xml_forest`) registers semantic actions for each production.

```mermaid
flowchart TB
    subgraph ActionRegistration[Action Registration]
        RRA[Register Rule Actions]
        RLA[Register Literal Sequence Actions]
        RGA[Register Group Actions]
        RMA[Register Marked Literal Actions]
    end

    subgraph ActionTypes[Action Types]
        ERA[Element Creation]
        TRA[Text Collection]
        ARA[Attribute Extraction]
        URA[Unwrapping Hidden/Promoted]
    end

    AST[IxmlGrammar] --> RRA
    RRA --> RLA
    RLA --> RGA
    RGA --> RMA

    RRA --> ERA
    RRA --> TRA
    RRA --> ARA
    RRA --> URA

    style ActionRegistration fill:#e8eaf6,color:#000
    style ActionTypes fill:#fbe9e7,color:#000
```

### XmlNode Structure

```mermaid
flowchart TB
    XN[XmlNode]

    XN --> EL[Element]
    XN --> TX[Text]
    XN --> AT[Attribute]

    EL --> ELF[name: String<br/>attributes: Vec<br/>children: Vec]
    TX --> TXF[String]
    AT --> ATF[name: String<br/>value: String]

    style XN fill:#f3e5f5,color:#000
```

### XML Serialization

The serializer (`src/runtime_parser.rs:to_xml`) produces canonical iXML format.

```mermaid
flowchart TB
    subgraph SerializationModes[Serialization Modes]
        COMP[Compact Mode]
        CAN[Canonical Mode]
    end

    subgraph CanonicalFormat[Canonical Format Features]
        OT[Opening tag without final >]
        GT[> on next line with indent]
        CT[Closing tag without final >]
        RTG[Root element gets final >]
    end

    XN[XmlNode Tree] --> COMP
    XN --> CAN
    CAN --> OT
    CAN --> GT
    CAN --> CT
    CAN --> RTG
    COMP --> XML1[Compact XML]
    CAN --> XML2[Canonical XML]

    style SerializationModes fill:#e0f2f1,color:#000
    style CanonicalFormat fill:#fafafa,color:#000
```

**Test References:**
- `arith` - Tests canonical format with nested elements
- `attribute-value` - Tests attribute escaping
- `element-content` - Tests text content escaping

---

## Semantic Actions System

### Node Processing Flow

```mermaid
flowchart TB
    subgraph NodeProcessing[Node Processing in Actions]
        RN[Receive Nodes]
        UP[Unwrap Containers]
        PM[Process Marks]
        EA[Extract Attributes]
        BC[Build Children]
        CE[Create Element]
    end

    RN --> UP
    UP --> PM
    PM --> EA
    PM --> BC
    EA --> CE
    BC --> CE

    style NodeProcessing fill:#fff8e1,color:#000
```

### Container Unwrapping

```mermaid
flowchart LR
    subgraph Containers[Container Types to Unwrap]
        RC[_repeat_container]
        HD[_hidden]
        GR[group]
    end

    subgraph Result[Unwrapping Result]
        CH[Extract children]
        AT[Promote attributes]
    end

    RC --> CH
    HD --> CH
    HD --> AT
    GR --> CH

    style Containers fill:#ffccbc,color:#000
    style Result fill:#c8e6c9,color:#000
```

---

## Mark Processing

### Mark Types and Effects

```mermaid
flowchart TB
    subgraph MarkTypes[Mark Types]
        MN[None - default visible]
        MH[Hidden: -]
        MA[Attribute: @]
        MP[Promoted: ^]
    end

    subgraph RuleMark[Rule Definition Mark]
        RMN[Creates visible element]
        RMH[Creates _hidden wrapper]
        RMA[Creates Attribute node]
        RMP[Creates _promoted wrapper]
    end

    subgraph FactorMark[Factor Reference Mark]
        FMN[Keep element as child]
        FMH[Extract children inline]
        FMA[Convert to attribute]
        FMP[Wrap in _promoted inline]
    end

    MN --> RMN
    MH --> RMH
    MA --> RMA
    MP --> RMP

    MN --> FMN
    MH --> FMH
    MA --> FMA
    MP --> FMP

    style MarkTypes fill:#e1bee7,color:#000
    style RuleMark fill:#b2dfdb,color:#000
    style FactorMark fill:#ffecb3,color:#000
```

**Test References:**
- `aaa` - Tests hidden marked literals (`-"text"`)
- `marked` - Tests attribute marks (`@name`)
- `lf` - Tests hidden hex chars (`-#a`)

### Mark Override Behavior

When a factor mark differs from the rule definition mark:

```mermaid
flowchart LR
    subgraph Override[Mark Override Examples]
        E1[Rule: -expr<br/>Factor: ^expr]
        E2[Rule: name<br/>Factor: @name]
    end

    subgraph Result[Override Result]
        R1[Factor mark wins<br/>Creates visible expr]
        R2[Factor mark wins<br/>Creates attribute]
    end

    E1 --> R1
    E2 --> R2

    style Override fill:#fff3e0,color:#000
    style Result fill:#e8f5e9,color:#000
```

**Test References:**
- `expr2` - Tests `^expr` promoting hidden `-expr`
- `expr1` - Tests `@plusop` creating attributes from factor

---

## Test Infrastructure

### Test Runner Architecture

```mermaid
flowchart TB
    subgraph TestInput[Test Input Files]
        GF[name.ixml - Grammar]
        IF[name.inp - Input]
        OF[name.output.xml - Expected]
    end

    subgraph TestRunner[Test Runner]
        RT[read_simple_test]
        RN[run_test]
        CMP[Compare Output]
    end

    subgraph Outcomes[Test Outcomes]
        PASS[Pass]
        FAIL[Fail with diff]
        GPE[GrammarParseError]
        IPE[InputParseError]
        SKIP[Skip]
    end

    GF --> RT
    IF --> RT
    OF --> RT
    RT --> RN
    RN --> CMP
    CMP --> PASS
    CMP --> FAIL
    CMP --> GPE
    CMP --> IPE
    CMP --> SKIP

    style TestInput fill:#e3f2fd,color:#000
    style TestRunner fill:#f3e5f5,color:#000
    style Outcomes fill:#fff8e1,color:#000
```

### Docker Test Environment

```mermaid
flowchart TB
    subgraph DockerEnv[Docker Environment]
        DF[Dockerfile.test]
        IMG[rustixml-test image]
        CNT[Container]
    end

    subgraph SafeRunner[Safe Conformance Runner]
        PC[Panic Catching]
        TO[2s Timeout]
        IW[Incremental Write]
        SK[Skip Known Crashers]
    end

    subgraph Output[Test Output]
        SO[stdout progress]
        RF[/tmp/safe_results.txt]
        SUM[Summary stats]
    end

    DF --> IMG
    IMG --> CNT
    CNT --> PC
    CNT --> TO
    CNT --> IW
    CNT --> SK
    PC --> SO
    IW --> RF
    RF --> SUM

    style DockerEnv fill:#ffecb3,color:#000
    style SafeRunner fill:#c8e6c9,color:#000
    style Output fill:#e1bee7,color:#000
```

### Test Categories and Status

| Category | Count | Status | Examples |
|----------|-------|--------|----------|
| Passing | 30 | Complete | `arith`, `hash`, `test`, `hex`, `marked` |
| Failing | 8 | Output mismatch | `json`, `vcard`, `xml`, `expr1`, `expr2`, `expr4` |
| Error | 6 | Parse issues | `email`, `unicode-classes`, `xpath` |
| Timeout | 0 | Fixed | Previously `expr1-6`, `diary1-3` |

### Semantic XML Comparison

```mermaid
flowchart TB
    subgraph Comparison[Comparison Strategy]
        ES[Exact String Match]
        SC[Semantic Comparison]
    end

    subgraph SemanticCheck[Semantic Checks]
        TN[Tag Names]
        AT[Attributes unordered]
        TX[Text trimmed]
        CH[Children ordered]
    end

    EXP[Expected XML] --> ES
    ACT[Actual XML] --> ES
    ES -->|Match| PASS[Pass]
    ES -->|No Match| SC
    SC --> TN
    SC --> AT
    SC --> TX
    SC --> CH
    TN --> PASS2[Pass]
    AT --> PASS2
    TX --> PASS2
    CH --> PASS2

    style Comparison fill:#e8eaf6,color:#000
    style SemanticCheck fill:#fbe9e7,color:#000
```

---

## File Reference

| File | Purpose | Key Functions |
|------|---------|---------------|
| `src/lexer.rs` | Tokenization | `Lexer::next_token()` |
| `src/ast.rs` | AST definitions | `IxmlGrammar`, `Rule`, `Factor` |
| `src/grammar_parser.rs` | Recursive descent parser | `parse_ixml_grammar()` |
| `src/grammar_ast.rs` | Parser entry point | Re-exports parser |
| `src/runtime_parser.rs` | Earley conversion + XML | `ast_to_earlgrey()`, `build_xml_forest()` |
| `src/testsuite_utils.rs` | Test infrastructure | `run_test()`, `xml_deep_equal()` |
| `src/bin/safe_conformance_runner.rs` | Docker test runner | Panic-safe test execution |

---

## Known Issues and Recent Fixes

### Recent Fixes (November 2025)

1. **GROUP_COUNTER Synchronization** - Fixed Missing Action errors
   - **Problem:** AST traversed twice (conversion + action registration), counter incremented differently
   - **Solution:** Thread-local `GROUP_ID_MAP` stores `group_pointer → group_id` mapping
   - **Result:** Eliminated all "Missing Action" errors, 1 test moved from INPUT_ERROR to FAIL

2. **Character Class Parsing** - Fixed unquoted character sequences
   - **Problem:** `[xyz]` treated "xyz" as Unicode category, not individual chars
   - **Solution:** Added else clause in `parse_char_class()` to split unquoted sequences
   - **Result:** Character classes now work in simple cases
   - **File:** `src/runtime_parser.rs:1322`

3. **Test Classification** - Identified intentionally invalid tests
   - **Problem:** `elem1` has empty character class `[]` causing parse failure
   - **Solution:** Added skip logic for invalid grammar tests
   - **Result:** Reduced noise in test results

### Current Limitations

1. **Large Grammar Parsing** - Some tests with 40+ alternatives fail
   - Tests: `unicode-classes`, `ixml-spaces`, `ixml3`
   - Status: Parse succeeds for simple versions, fails for full grammar
   - Hypothesis: Earley parser limitations with large choice sets

2. **Promoted mark handling** - `^` on hidden elements needs proper element name restoration
   - Affects: `expr2`, `expr4` tests
   - Status: Output structure incorrect

3. **Separated repetition attributes** - `@plusop` in `term++plusop` not collected to parent
   - Affects: `expr1` test
   - Status: Attributes not propagated correctly

### Architectural Considerations

**Translation Layer Complexity:**
The current implementation translates iXML to Earley grammar, which creates several challenges:

1. **Impedance Mismatch** - Earley expects token-level parsing, we do character-level
2. **Helper Nonterminals** - Repetitions require generated rules (`name_plus`, `name_star`)
3. **Synchronization Issues** - Multiple AST traversals need coordinated state (GROUP_COUNTER)
4. **Predicate Generation** - Character classes converted to Rust predicates

See `ABSTRACTION_ANALYSIS.md` for detailed discussion of specification-driven vs translation-based approaches.

### Performance Considerations

- Handwritten parser: O(n) linear time for grammar parsing
- Earley parser: O(n³) worst case, O(n) for unambiguous grammars
- Character-level tokenization: Every character is a separate token
- Left-recursive grammars may cause performance issues
- Large grammars (40+ rules) may hit Earley limitations

### Test Coverage Status (November 2025)

**Current Results:**
- **53 PASS** (39.8%) - Core functionality working
- **12 FAIL** (9.0%) - Output mismatch, need XML structure fixes
- **3 INPUT_ERROR** (2.3%) - Parse failures on large grammars
- **65 SKIP** (48.9%) - No expected output or test configuration

**Priority Issues:**
1. Fix large grammar parsing (unicode-classes, ixml-spaces, ixml3)
2. Fix promoted mark handling (expr2, expr4)
3. Fix attribute collection in separated repetitions (expr1)

**Target:** 90%+ pass rate
- Secondary: Resolve input parse errors
