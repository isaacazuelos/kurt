//! Operator Parsing
//!
//! These methods help us determine if an [`Operator`][TokenKind::Operator] is
//! being used as prefix, postfix or infix.
//!
//! Ultimately, I want to support user-defined operators with user-defined
//! precedence, while using the same operator in any of these positions.
//!
//! To make this work, we use whitespace around operators to tell us how an
//! operator is being used. The full details are in the documentation below.
//!
//! ## Why whitespace?
//!
//! This is simplest way to do this that I can find, and the main side effect on
//! the programmer is requiring nicer code formatting. It's also more or less
//! [what Swift does][swift-lex], and it's surprisingly hard to find people
//! complaining or confused by it.
//!
//! It does mean that an expression like `1+-7` that works in a languages like
//! C, Python or Ruby, is instead parsed as the infix operator `+-` and won't
//! work though. Conceivably we could check for splits like this when generating
//! the full diagnostic messages. It's not something we want to do on the fly
//! though, as the lexer doesn't know which operators are or are not in scope at
//! any given moment. We could work around this by streaming tokens and feeding
//! that information back, but that's [historically][c-lex] not been a great
//! solution. It also would leave us with situation where say `•`, `•◊` and `◊•`
//! are defined, and splitting the expression `a•◊•b` becomes ambiguous again
//! anyway.
//!
//! It was tempting to write something that would allow `a •a` to be infix.
//! However, I felt that order becomes unclear when you have `a• •a` - which of
//! the `•` operators is the infix one? This isn't ambiguous unless `•` is
//! defined for all 3 positions, but it's ambiguous _to the reader_. It's also
//! not clear that the complexity to allow the user to _choose_ which is which
//! would be useful.
//!
//! [swift-lex]: https://docs.swift.org/swift-book/ReferenceManual/LexicalStructure.html#ID418
//! [c-lex]: https://en.wikipedia.org/wiki/Lexer_hack

use diagnostic::{Diagnostic, Span};

use crate::{
    lexer::{Delimiter, Token, TokenKind},
    operator::{Associativity, DefinedOperators, Precedence},
    parser::Parser,
};

// These methods are long, but not too complicated. To disambiguate operators, a
// lot of things have to be checked. To the control flow flatter we use early
// returns as each condition is violated.
impl<'a> Parser<'a> {
    pub const ALLOWED_BEFORE_PREFIX: &'static [TokenKind] = {
        use Delimiter::*;
        use TokenKind::*;
        &[
            Open(Parenthesis),
            Open(Brace),
            Open(Bracket),
            Comma,
            Colon,
            Semicolon,
            Dot,
        ]
    };

    pub const ALLOWED_AFTER_POSTFIX: &'static [TokenKind] = {
        use Delimiter::*;
        use TokenKind::*;
        &[
            Open(Parenthesis),
            Open(Brace),
            Open(Bracket),
            Close(Parenthesis),
            Close(Brace),
            Close(Bracket),
            Comma,
            Colon,
            Semicolon,
            Dot,
        ]
    };

    /// The currently defined operators.
    pub fn defined_operators(&self) -> &DefinedOperators {
        &self.operators
    }

    /// A mutable reference the defined operators, which can be used to add
    /// definitions.
    pub fn defined_operators_mut(&mut self) -> &mut DefinedOperators {
        &mut self.operators
    }

    /// Consume the next token if it's a prefix operator.
    ///
    /// The next token is a prefix operator when:
    ///
    /// 1. The next token is an [`Operator`][TokenKind::Operator].
    ///
    /// 2. The operator is defined for use as prefix. See [`DefinedOperators`].
    ///
    /// 3. There's at least one token after the operator.
    ///
    /// 4. No whitespace occurs on the right of the operator.
    ///
    /// 5. If there's a token before the operator, there's either whitespace
    ///    between them, or the token is in [`Parser::ALLOWED_BEFORE_PREFIX`].
    pub fn consume_prefix(&mut self) -> Result<Token<'a>, Error> {
        // 1
        if self.is_empty() {
            let last_span =
                self.tokens.last().map(Token::span).unwrap_or_default();
            return Err(Error::EOF(last_span));
        }

        let token = self.tokens[self.cursor];
        if token.kind() != TokenKind::Operator {
            return Err(Error::NotOperator(token.span()));
        }

        // 2
        if !self.operators.is_prefix(token.body()) {
            return Err(Error::UndefinedPrefix(token.span()));
        }

        // 3
        if self.cursor + 1 >= self.tokens.len() {
            return Err(Error::PrefixAtEnd(token.span()));
        }

        // 4
        let after_tok = self.tokens[self.cursor + 1];
        let space_after = token.span().end() != after_tok.span().start();

        if space_after {
            return Err(Error::PrefixSpaceAfter(token.span()));
        }

        // 5
        if self.cursor > 0 {
            let before_token = self.tokens[self.cursor - 1];
            let space_before =
                before_token.span().end() != token.span().start();
            let is_allowed =
                Parser::ALLOWED_BEFORE_PREFIX.contains(&before_token.kind());

            if !(space_before || is_allowed) {
                return Err(Error::PrefixNoSpaceBefore(token.span()));
            }
        }

        self.advance();
        Ok(token)
    }

    /// Consume the next token if it's a postfix operator.
    ///
    /// It's a postfix operator when:
    ///
    /// 1. The next token is an [`Operator`][TokenKind::Operator].
    ///
    /// 2. There's at least one token before it. It can't bind to nothing
    ///
    /// 3. The operator is defined for postfix use. See [`DefinedOperators`].
    ///
    /// 4. No whitespace occurs on the left, otherwise it would be infix.
    ///
    /// 5. If there's a token after, there must be whitespace between them, or
    ///    the token must be in [`Parser::ALLOWED_AFTER_POSTFIX`].
    pub fn consume_postfix(&mut self) -> Result<Token<'a>, Error> {
        // 1
        if self.is_empty() {
            let last_span =
                self.tokens.last().map(Token::span).unwrap_or_default();
            return Err(Error::EOF(last_span));
        }

        let token = self.tokens[self.cursor];
        if token.kind() != TokenKind::Operator {
            return Err(Error::NotOperator(token.span()));
        }

        // 2
        if self.cursor == 0 {
            return Err(Error::PostfixAtStart(token.span()));
        }

        // 3
        if !self.operators.is_postfix(token.body()) {
            return Err(Error::UndefinedPostfix(token.span()));
        }

        // 4
        let before_token = self.tokens[self.cursor - 1];
        let space_before = before_token.span().end() != token.span().start();
        if space_before {
            return Err(Error::PostfixSpaceBefore(token.span()));
        }

        // 5
        if self.cursor + 1 < self.tokens.len() {
            let after_token = self.tokens[self.cursor + 1];
            let space_after = token.span().end() != after_token.span().start();
            let is_allowed =
                Parser::ALLOWED_AFTER_POSTFIX.contains(&after_token.kind());

            if !(space_after || is_allowed) {
                return Err(Error::PostfixNoSpaceAfter(token.span()));
            }
        }

        self.advance();
        Ok(token)
    }

    /// Consume the next token iff it looks to be an infix operator, i.e. if the
    /// following conditions are met:
    ///
    /// 1. The next token is an `Kind::Operator`.
    ///
    /// 2. We need at least 1 token before for the lhs, and 2 after for the
    ///    operator we'll return and the rhs.
    ///
    /// 3. The operator is defined for use as an infix operator. This step is
    ///    needed to determine precedence.
    ///
    /// 4. If the infix operator has precedence (i.e. associativity isn't
    ///    explicitly disallowed for it, like with operators like `+=`) this
    ///    precedence is equal to the argument `precedence`.
    ///
    /// 5. Whitespace is balanced -- there's either whitespace on both sides or
    ///    on neither. This is a trick we're pulling from Swift to distinguish
    ///    prefix and postfix from infix.
    pub fn consume_infix(
        &mut self,
        wanted: Precedence,
    ) -> Result<(Token<'a>, Associativity), Error> {
        // 1
        if self.is_empty() {
            let last_span =
                self.tokens.last().map(Token::span).unwrap_or_default();
            return Err(Error::EOF(last_span));
        }

        let token = self.tokens[self.cursor];
        if token.kind() != TokenKind::Operator {
            return Err(Error::NotOperator(token.span()));
        }

        // 2
        if !(self.cursor >= 1 && self.cursor + 2 <= self.tokens.len()) {
            return Err(Error::InfixAtStartOrEnd(token.span()));
        }

        // 3
        let (associativity, found) = self
            .operators
            .get_infix(token.body())
            .ok_or_else(|| Error::UndefinedInfix(token.span()))?;

        // 4
        if found != wanted {
            return Err(Error::InfixWrongPrecedence(token.span()));
        }

        // 5
        let before_tok = self.tokens[self.cursor - 1];
        let after_tok = self.tokens[self.cursor + 1];

        let space_before = before_tok.span().end() != token.span().start();
        let space_after = token.span().end() != after_tok.span().start();

        if space_before != space_after {
            return Err(Error::InfixUnbalanced(token.span()));
        }

        self.advance();
        Ok((token, associativity))
    }
}

/// These are the errors which can come up when trying to work with operators.
#[derive(Debug, Clone, Copy)]
pub enum Error {
    EOF(Span),

    UndefinedPrefix(Span),
    UndefinedPostfix(Span),
    UndefinedInfix(Span),
    MultipleNonAssociative(Span, Span),

    NotOperator(Span),

    PrefixSpaceAfter(Span),
    PrefixNoSpaceBefore(Span),
    PrefixAtEnd(Span),

    PostfixNoSpaceAfter(Span),
    PostfixSpaceBefore(Span),
    PostfixAtStart(Span),

    InfixAtStartOrEnd(Span),
    InfixUnbalanced(Span),
    InfixWrongPrecedence(Span),
}

impl From<Error> for Diagnostic {
    // Many of these really shouldn't appear as-is, and are more useful for
    // creating better syntax-aware diagnostics.
    #[rustfmt::skip]
    fn from(e: Error) -> Diagnostic {
        match e {
            Error::EOF(span) => {
                Diagnostic::new("input ended when expecting operator")
                    .location(span.end())
                    .highlight(span, "an operator was expected after this")
            }

            Error::UndefinedPrefix(span) => {
                Diagnostic::new("a prefix operator not defined")
                    .location(span.start())
                    .highlight(span, "not defined")
            }

            Error::UndefinedPostfix(span) => {
                Diagnostic::new("a postfix operator not defined")
                    .location(span.start())
                    .highlight(span, "not defined")
            }

            Error::UndefinedInfix(span) => {
                Diagnostic::new("a infix operator not defined")
                    .location(span.start())
                    .highlight(span, "not defined")
            }

            Error::MultipleNonAssociative(first, second) => {
                Diagnostic::new("the order of operations is not clear")
                    .location(first.start())
                    .highlight(first, "should this happen first")
                    .highlight(second, "or should this one happen first")
                    .help("use parenthesis to make the ordering clear")
                    .info(
                        "this only happens when we have two non-associative \
                        operators with the same precedence next to each other"
                    )
            }

            Error::NotOperator(span) => {
                Diagnostic::new("an operator was expected")
                    .location(span.start())
                    .highlight(span, "instead we saw this")
            }

            Error::PrefixSpaceAfter(span) => {
                Diagnostic::new(
                    "prefix operators shouldn't have spaces after them"
                )
                    .location(span.start())
                    .highlight(
                        span,
                        "the whitespace after this should be removed"
                    )
                    .info("otherwise it's not clear if it's prefix or infix")
            }

            Error::PrefixNoSpaceBefore(span) => {
                Diagnostic::new("prefix operators needs spaces before them")
                    .location(span.start())
                    .highlight(span, "some whitespace is needed before this")
                    .info("otherwise it's not clear if it's prefix or infix")
            }

            Error::PrefixAtEnd(span) => {
                Diagnostic::new(
                    "prefix operators can't be at the end of the input"
                )
                    .location(span.start())
                    .highlight(span, "this operator has nothing to operate on")
            }

            Error::PostfixNoSpaceAfter(span) => {
                Diagnostic::new(
                    "prefix operators can't have whitespace after them"
                )
                    .location(span.start())
                    .highlight(
                        span,
                        "the whitespace after this should be removed"
                    )
                    .info("otherwise it looks like an infix operator")
            },

            Error::PostfixSpaceBefore(span) => {
                Diagnostic::new(
                    "prefix operators must have whitespace before them"
                )
                    .location(span.start())
                    .highlight(
                        span,
                        "whitespace is needed before this"
                    )
                    .info("otherwise it looks like an infix operator")
            },

            Error::PostfixAtStart(span) => {
                Diagnostic::new(
                    "postfix operators cannot be at the start of the input"
                )
                    .location(span.start())
                    .highlight(span, "this operator has nothing to operate on")
            },

            Error::InfixAtStartOrEnd(span) => {
                Diagnostic::new(
                    "infix operators can't be at the start or end of the input"
                )
                    .location(span.start())
                    .highlight(span, "this operator is missing an operand")
            }

            Error::InfixUnbalanced(span) => {
                Diagnostic::new("infix operators need balanced whitespace")
                    .location(span.start())
                    .highlight(span, "this whitespace isn't balanced")
                    .info(
                        "infix operators either need whitespace on both sides \
                        or neither, otherwise they look prefix or postfix"
                    )
            }

            Error::InfixWrongPrecedence(span) => {
                Diagnostic::new("an infix operator has the wrong precedence")
                    .location(span.start())
                    .highlight(
                        span,
                        "this infix operator doesn't have the precedence \
                        needed by the parser"
                    )
            },
        }
    }
}

#[cfg(test)]
mod tests {
    use crate::operator::DefinedOperators;

    use super::*;

    /// A helper to get the precedence of an operator in [`DefinedOperators`]
    /// [`Default`]s.
    fn prec_of(s: &str) -> Precedence {
        DefinedOperators::default().get_infix(s).unwrap().1
    }

    // This was embarrassingly difficult to get right.

    #[test]
    fn defined_operator_assumptions() {
        // In our tests, we'll be using some operators to test things, and we
        // need to make sure they're defined the way we think they are.
        let parser = Parser::new("").unwrap();

        assert!(parser.operators.is_prefix("-"));
        assert!(parser.operators.is_postfix("?"));
        assert!(parser.operators.get_infix("+").is_some());
        assert_ne!(prec_of("*"), prec_of("+"))
    }

    #[test]
    fn prefix() {
        let mut parser = Parser::new("-a").unwrap();
        assert!(parser.consume_prefix().is_ok());
    }

    #[test]
    fn prefix_1_non_operator() {
        let mut parser = Parser::new("= a").unwrap();
        assert!(parser.consume_prefix().is_err());
    }

    #[test]
    fn prefix_2_undefined() {
        let mut parser = Parser::new("<$> a").unwrap();
        assert!(!parser.operators.is_prefix("<$>"));
        assert!(parser.consume_prefix().is_err());
    }

    #[test]
    fn prefix_3_at_eof() {
        let mut parser = Parser::new("-").unwrap();
        assert!(parser.consume_prefix().is_err());
    }

    #[test]
    fn prefix_4_no_right_whitespace() {
        let mut parser = Parser::new("- a").unwrap();
        assert!(parser.consume_prefix().is_err());
    }

    #[test]
    fn prefix_5_missing_space_and_disallowed() {
        let mut parser = Parser::new("a-a").unwrap();
        assert!(parser.consume(TokenKind::Identifier,).is_some());
        assert!(parser.consume_prefix().is_err());
    }

    #[test]
    fn prefix_5_missing_space_but_allowed() {
        let mut parser = Parser::new("(-a)").unwrap();
        assert!(parser
            .consume(TokenKind::Open(Delimiter::Parenthesis))
            .is_some());
        assert!(parser.consume_prefix().is_ok());
    }

    #[test]
    fn prefix_5_disallowed_but_space() {
        let mut parser = Parser::new("a -a").unwrap();
        assert!(parser.consume(TokenKind::Identifier).is_some());
        assert!(parser.consume_prefix().is_ok());
    }

    #[test]
    fn prefix_5_allowed_and_space() {
        let mut parser = Parser::new("( -a)").unwrap();
        assert!(parser
            .consume(TokenKind::Open(Delimiter::Parenthesis))
            .is_some());
        assert!(parser.consume_prefix().is_ok());
    }

    #[test]
    fn postfix() {
        let mut parser = Parser::new("a?").unwrap();
        assert!(parser.consume(TokenKind::Identifier).is_some());
        assert!(parser.consume_postfix().is_ok());
    }

    #[test]
    fn postfix_1_need_operator() {
        let mut parser = Parser::new("a a").unwrap();
        assert!(parser.consume(TokenKind::Identifier).is_some());
        assert!(parser.consume_postfix().is_err());
    }

    #[test]
    fn postfix_2_at_start() {
        let mut parser = Parser::new("?a").unwrap();
        assert!(parser.consume_postfix().is_err());
    }

    #[test]
    fn postfix_3_undefined() {
        let mut parser = Parser::new("a-").unwrap();
        assert!(!parser.operators.is_postfix("-"));
        assert!(parser.consume(TokenKind::Identifier).is_some());
        assert!(parser.consume_postfix().is_err());
    }

    #[test]
    fn postfix_4_no_left_whitespace() {
        let mut parser = Parser::new("a ?").unwrap();
        assert!(parser.consume(TokenKind::Identifier).is_some());
        assert!(parser.consume_postfix().is_err());
    }

    #[test]
    fn postfix_5_no_right_whitespace_and_disallowed() {
        let mut parser = Parser::new("a?a").unwrap();
        assert!(parser.consume(TokenKind::Identifier).is_some());
        assert!(parser.consume_postfix().is_err());
    }

    #[test]
    fn postfix_5_no_right_whitespace_but_allowed() {
        let mut parser = Parser::new("a?[").unwrap();
        assert!(parser.consume(TokenKind::Identifier).is_some());
        let result = parser.consume_postfix();
        assert!(result.is_ok(), "got {:?}", result);
    }

    #[test]
    fn postfix_5_whitespace_after() {
        let mut parser = Parser::new("a? a").unwrap();
        assert!(parser.consume(TokenKind::Identifier).is_some());
        assert!(parser.consume_postfix().is_ok());
    }

    #[test]
    fn postfix_with_dot() {
        let mut parser = Parser::new("a?.b").unwrap();
        assert!(parser.consume(TokenKind::Identifier).is_some());
        let result = parser.consume_postfix();
        assert!(result.is_ok(), "got {:?}", result);
        assert!(parser.consume(TokenKind::Dot).is_some());
    }

    #[test]
    fn postfix_with_other() {
        let mut parser = Parser::new("a?[b]").unwrap();
        assert!(parser.consume(TokenKind::Identifier).is_some());
        let result = parser.consume_postfix();
        assert!(result.is_ok(), "got {:?}", result);
    }

    #[test]
    fn infix() {
        let mut parser = Parser::new("a + a").unwrap();
        assert!(parser.consume(TokenKind::Identifier).is_some());
        assert!(parser.consume_infix(prec_of("+")).is_ok());
    }

    #[test]
    fn infix_wrong_precedence() {
        let mut parser = Parser::new("a + a").unwrap();
        assert!(parser.consume(TokenKind::Identifier).is_some());
        assert!(parser.consume_infix(prec_of("*")).is_err());
    }

    #[test]
    fn infix_no_spaces() {
        let mut parser = Parser::new("a+a").unwrap();
        assert!(parser.consume(TokenKind::Identifier).is_some());
        assert!(parser.consume_infix(prec_of("+")).is_ok());
    }

    #[test]
    fn infix_unbalanced_left() {
        let mut parser = Parser::new("a+ a").unwrap();
        assert!(parser.consume(TokenKind::Identifier).is_some());
        assert!(parser.consume_infix(prec_of("+")).is_err());
    }

    #[test]
    fn infix_unbalanced_right() {
        let mut parser = Parser::new("a +a").unwrap();
        assert!(parser.consume(TokenKind::Identifier).is_some());
        assert!(parser.consume_infix(prec_of("+")).is_err());
    }

    #[test]
    fn all_three() {
        let dot_prec = prec_of("+");
        let wrong_prec = prec_of("*");

        let mut parser = Parser::new("a• • •a").unwrap();

        parser.defined_operators_mut().define_prefix("•");
        parser.defined_operators_mut().define_postfix("•");
        parser.defined_operators_mut().define_infix(
            "•",
            Associativity::Left,
            dot_prec,
        );

        assert!(parser.consume(TokenKind::Identifier).is_some());

        assert!(parser.consume_prefix().is_err());
        assert!(parser.consume_infix(wrong_prec).is_err());
        assert!(parser.consume_infix(dot_prec).is_err());
        assert!(parser.consume_postfix().is_ok());

        assert!(parser.consume_prefix().is_err());
        assert!(parser.consume_postfix().is_err());
        assert!(parser.consume_infix(wrong_prec).is_err());
        assert!(parser.consume_infix(dot_prec).is_ok());

        assert!(parser.consume_postfix().is_err());
        assert!(parser.consume_infix(wrong_prec).is_err());
        assert!(parser.consume_infix(dot_prec).is_err());
        assert!(parser.consume_prefix().is_ok());

        assert!(parser.consume(TokenKind::Identifier).is_some());
        assert!(parser.is_empty());
    }
}
