//! # Tokens
//!
//! Each token is an individual lexeme in our language -- the smallest unit of
//! meaning.
//!
//! Tokens provide both the semantic information in the form of their `Kind`,
//! and the general context they were found in.

use diagnostic::Span;

/// An individual lexeme in our language.
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct Token<'a> {
    /// The semantic kind thing the token is. See `Kind` for more.
    pub(crate) kind: Kind,

    /// This is the `Span` of this token's body, not including any surrounding
    /// whitespace.
    pub(crate) span: Span,

    /// The body of the token as it was represented in the original input.
    /// This does _not_ do any unicode normalization.
    pub(crate) body: &'a str,
}

impl<'a> Token<'a> {
    /// The kind of token this is.
    pub fn kind(&self) -> Kind {
        self.kind
    }
    /// The span of the body of this token, not including surrounding
    /// whitespace.
    pub fn span(&self) -> Span {
        self.span
    }

    /// The way the token was represented in the source.
    pub fn body(&self) -> &'a str {
        self.body
    }
}

impl<'a> ::std::fmt::Display for Token<'a> {
    fn fmt(&self, f: &mut ::std::fmt::Formatter) -> ::std::fmt::Result {
        write!(f, "{}", self.body)
    }
}

/// A [`Token`]'s kind is the semantically-relevant part of the token, removed
/// from the source context.
#[derive(Debug, Clone, Copy, PartialEq)]
pub enum Kind {
    /// Comments which begin with `//`.
    Comment(Comment),

    /// Words which are used by the language and can't be identifiers.
    Reserved(Reserved),

    /// The boolean values `true` or `false`.
    Bool,
    /// A Unicode scalar like `'\u{40}'`
    Char,
    /// An integer like `5`.
    Int,
    /// A floating point number like `12.34e-56.
    Float,
    /// A hexadecimal value like `0xCAFE`.
    Hex,
    /// A binary value like `0b0101`
    Bin,
    /// An octal value like `0o777`
    Oct,

    /// A String literal like `"Hello World!\n"`.
    String,

    /// Things like `foo` are identifies, names for things.
    Identifier,
    /// Things like `+`, `==` or `>>=` are operators. Notably, a single `=` is
    /// not.
    Operator,

    /// `->` or `→`
    Arrow,
    /// `@`
    At,
    /// A single backtick, i.e. a _Grave Accent_
    Backtick,
    /// `:`
    Colon,
    /// `,`
    Comma,
    /// `=>` or `⇒`
    DoubleArrow,
    ///`=`
    Equals,
    /// `;`
    Semicolon,

    /// `.`
    Dot,
    /// `..`
    Range,
    /// `...`
    Spread,

    /// Open a paired delimiter.
    Open(Delimiter),

    /// Close a paired delimiter
    Close(Delimiter),
}

impl Kind {
    /// The user-facing name of this kind of token.
    pub fn name(&self) -> &'static str {
        use self::Comment::*;
        use Delimiter::*;
        use Kind::*;
        match self {
            Comment(Line) => "line comment",
            Comment(Doc) => "documentation comment",
            Comment(Header) => "header comment",
            Comment(Markup) => "markup comment",
            Reserved(r) => r.as_str(),
            Bool => "boolean",
            Char => "character",
            Int => "number",
            Float => "number",
            Hex => "hex number",
            Bin => "binary number",
            Oct => "octal number",
            String => "string",
            Identifier => "identifier",
            Operator => "operator",
            Arrow => "arrow (->)",
            At => "at sign (@)",
            Backtick => "backtick (`)",
            Colon => "colon (:)",
            Comma => "comma (,)",
            DoubleArrow => "double arrow (=>)",
            Equals => "equals sign (=)",
            Semicolon => "semicolon (;)",
            Dot => "period (.)",
            Range => "range (..)",
            Spread => "spread (...)",
            Open(Parenthesis) => "open parenthesis",
            Close(Parenthesis) => "close parenthesis",
            Open(Bracket) => "open bracket",
            Close(Bracket) => "close bracket",
            Open(Brace) => "open brace",
            Close(Brace) => "close brace",
        }
    }

    /// Is this token kind always a literal?
    ///
    /// Note that a keyword literal like `:foo` is not covered here because it's
    /// more than one token.
    pub fn is_literal(&self) -> bool {
        use Kind::*;
        matches!(self, Bin | Bool | Char | Float | Hex | Int | Oct | String)
    }
}

/// Delimiters are the different sorts of characters with a distinct opening and
/// closing characters.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Delimiter {
    /// `{}`
    Brace,

    /// `[]`
    Bracket,

    /// `()`
    Parenthesis,
}

/// Reserved words are words which can't be used by programmers, but instead
/// are reserved for use by the language. Not all of these will be used, but
/// reserving them ahead of time is a good call.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Reserved {
    Async,
    Atomic,
    Await,
    Break,
    Case,
    Catch,
    Cond,
    Const,
    Continue,
    Do,
    Else,
    Enum,
    Export,
    Extern,
    Extend,
    For,
    Goto,
    If,
    Implement,
    Import,
    In,
    Interface,
    Let,
    Loop,
    Module,
    Panic,
    Protocol,
    Raise,
    Rec,
    Receive,
    Return,
    Send,
    Throw,
    Trait,
    Type,
    Var,
    While,
    With,
    Yield,
}

impl Reserved {
    /// The in-code representation of a reserved word.
    pub fn as_str(self) -> &'static str {
        use self::Reserved::*;
        match self {
            Async => "async",
            Atomic => "atomic",
            Await => "await",
            Break => "break",
            Case => "case",
            Catch => "catch",
            Cond => "cond",
            Const => "cons",
            Continue => "continue",
            Do => "do",
            Else => "else",
            Enum => "enum",
            Export => "export",
            Extern => "extern",
            Extend => "extend",
            For => "for",
            Goto => "goto",
            If => "if",
            Implement => "implement",
            Import => "import",
            In => "in",
            Interface => "interface",
            Let => "let",
            Loop => "loop",
            Module => "module",
            Panic => "panic",
            Protocol => "protocol",
            Raise => "raise",
            Rec => "rec",
            Receive => "receive",
            Return => "return",
            Send => "send",
            Throw => "throw",
            Trait => "trait",
            Type => "type",
            Var => "var",
            While => "while",
            With => "with",
            Yield => "yield",
        }
    }

    pub(crate) fn try_from_bytes(b: &str) -> Option<Reserved> {
        use self::Reserved::*;
        Some(match b {
            "async" => Async,
            "atomic" => Atomic,
            "await" => Await,
            "break" => Break,
            "case" => Case,
            "catch" => Catch,
            "cond" => Cond,
            "cons" => Const,
            "continue" => Continue,
            "do" => Do,
            "else" => Else,
            "enum" => Enum,
            "export" => Export,
            "extern" => Extern,
            "extend" => Extend,
            "for" => For,
            "goto" => Goto,
            "if" => If,
            "implement" => Implement,
            "import" => Import,
            "in" => In,
            "interface" => Interface,
            "let" => Let,
            "loop" => Loop,
            "module" => Module,
            "panic" => Panic,
            "protocol" => Protocol,
            "raise" => Raise,
            "rec" => Rec,
            "receive" => Receive,
            "return" => Return,
            "send" => Send,
            "throw" => Throw,
            "trait" => Trait,
            "type" => Type,
            "var" => Var,
            "while" => While,
            "with" => With,
            "yield" => Yield,
            _ => return None,
        })
    }
}

impl ::std::fmt::Display for Reserved {
    fn fmt(&self, f: &mut ::std::fmt::Formatter) -> ::std::fmt::Result {
        write!(f, "{}", self.as_str())
    }
}

/// There different kinds of comments, which will be useful to differentiate for
/// documentation, pretty printing, etc..
//
// We can include ways to identify item documentation, header documentation,
// license headers, compiler directives, etc.
#[derive(Debug, Clone, Copy, PartialEq)]
pub enum Comment {
    /// Line comments start with `//` and go to the end of the line. Comments
    /// don't include the end of line character -- that belongs to a following
    /// Whitespace fill. This is to handle a comment ending in EOF.
    Line,
    /// Doc comments are comments that start with `///` which are used to
    /// provide documentation.
    Doc,
    /// Head comments go at the start of the document and begin with `//!`,
    /// they're for module metadata like copyright info, dates, authors, etc..
    Header,
    /// Markup comments being with `//:` and are used for documentation that
    /// isn't bound to an item. This is useful for a sort of poor-man's
    /// literal programming.
    Markup,
}
