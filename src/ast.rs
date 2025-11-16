//! AST (Abstract Syntax Tree) for iXML grammars
//!
//! This module defines the data structures representing parsed iXML grammars.

#[derive(Debug, Clone, PartialEq)]
pub struct IxmlGrammar {
    pub rules: Vec<Rule>,
}

#[derive(Debug, Clone, PartialEq)]
pub struct Rule {
    pub name: String,
    pub mark: Mark,
    pub alternatives: Alternatives,
}

#[derive(Debug, Clone, PartialEq)]
pub struct Alternatives {
    pub alts: Vec<Sequence>,
}

#[derive(Debug, Clone, PartialEq)]
pub struct Sequence {
    pub factors: Vec<Factor>,
}

#[derive(Debug, Clone, PartialEq)]
pub struct Factor {
    pub base: BaseFactor,
    pub repetition: Repetition,
}

#[derive(Debug, Clone, PartialEq)]
pub enum BaseFactor {
    Literal {
        value: String,
        insertion: bool,  // true if this is insertion syntax +"text"
    },
    Nonterminal {
        name: String,
        mark: Mark,
    },
    CharClass {
        content: String,
        negated: bool,  // true if ~[...]
    },
    Group {
        alternatives: Box<Alternatives>,
    },
}

#[derive(Debug, Clone, PartialEq, Copy)]
pub enum Mark {
    None,       // no mark
    Attribute,  // @name - becomes XML attribute
    Hidden,     // -name - hidden from output
    Promoted,   // ^name - promoted (replaces parent)
}

#[derive(Debug, Clone, PartialEq, Copy)]
pub enum Repetition {
    None,           // no repetition
    ZeroOrMore,     // *
    OneOrMore,      // +
    Optional,       // ?
}

impl IxmlGrammar {
    pub fn new(rules: Vec<Rule>) -> Self {
        IxmlGrammar { rules }
    }
}

impl Rule {
    pub fn new(name: String, mark: Mark, alternatives: Alternatives) -> Self {
        Rule { name, mark, alternatives }
    }
}

impl Alternatives {
    pub fn new(alts: Vec<Sequence>) -> Self {
        Alternatives { alts }
    }

    pub fn single(seq: Sequence) -> Self {
        Alternatives { alts: vec![seq] }
    }
}

impl Sequence {
    pub fn new(factors: Vec<Factor>) -> Self {
        Sequence { factors }
    }

    pub fn empty() -> Self {
        Sequence { factors: vec![] }
    }
}

impl Factor {
    pub fn new(base: BaseFactor, repetition: Repetition) -> Self {
        Factor { base, repetition }
    }

    pub fn simple(base: BaseFactor) -> Self {
        Factor { base, repetition: Repetition::None }
    }
}

impl BaseFactor {
    pub fn literal(value: String) -> Self {
        BaseFactor::Literal { value, insertion: false }
    }

    pub fn insertion(value: String) -> Self {
        BaseFactor::Literal { value, insertion: true }
    }

    pub fn nonterminal(name: String) -> Self {
        BaseFactor::Nonterminal { name, mark: Mark::None }
    }

    pub fn marked_nonterminal(name: String, mark: Mark) -> Self {
        BaseFactor::Nonterminal { name, mark }
    }

    pub fn charclass(content: String) -> Self {
        BaseFactor::CharClass { content, negated: false }
    }

    pub fn negated_charclass(content: String) -> Self {
        BaseFactor::CharClass { content, negated: true }
    }

    pub fn group(alternatives: Alternatives) -> Self {
        BaseFactor::Group { alternatives: Box::new(alternatives) }
    }
}
