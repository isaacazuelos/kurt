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

use crate::{
    error::Error,
    lexer::{Delimiter, Token, TokenKind},
    operator::{Associativity, DefinedOperators, Precedence},
    parser::Parser,
};

// These methods are long, but not too complicated. To disambiguate operators, a
// lot of things have to be checked. To the control flow flatter we use early
// returns as each condition is violated.
impl Parser<'_> {
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
    /// 1. There's another token after it. The input cannot end with a prefix
    ///    operator, since it needs to bind to something.
    ///
    /// 2. The next token is an [`Operator`][TokenKind::Operator].
    ///
    /// 3. The operator is defined for use as prefix. See [`DefinedOperators`].
    ///
    /// 4. No whitespace occurs on the right of the operator.
    ///
    /// 5. If there's a token before the operator, there's either whitespace
    ///    between them, or the token is in [`Parser::ALLOWED_BEFORE_PREFIX`].
    pub fn consume_prefix(&mut self) -> Result<Token, Error> {
        const WANTED: &str = "a prefix operator";

        // 1
        if self.cursor + 1 >= self.tokens.len() {
            return Err(Error::EOFExpecting(
                "something following a prefix operator",
            ));
        }

        // 2
        let token = self.tokens[self.cursor];
        if token.kind() != TokenKind::Operator {
            return Err(Error::Unexpected {
                wanted: WANTED,
                found: token.kind(),
            });
        }

        // 3
        if !self.operators.is_prefix(token.body()) {
            return Err(Error::OperatorNotDefinedAsPrefix);
        }

        // 4
        let after_tok = self.tokens[self.cursor + 1];
        let space_after = token.span().end() != after_tok.span().start();

        if space_after {
            return Err(Error::PrefixSpaceAfter);
        }

        // 5
        if self.cursor > 0 {
            let before_token = self.tokens[self.cursor - 1];
            let space_before =
                before_token.span().end() != token.span().start();
            let is_allowed =
                Parser::ALLOWED_BEFORE_PREFIX.contains(&before_token.kind());

            if !(space_before || is_allowed) {
                return Err(Error::PrefixNoSpaceBefore);
            }
        }

        self.consume(token.kind(), "a prefix operator")
    }

    /// Consume the next token if it's a postfix operator.
    ///
    /// It's a postfix operator when:
    ///
    /// 1. There's at least one token before it. It can't bind to nothing
    ///
    /// 2. The next token is an [`Operator`][TokenKind::Operator].
    ///
    /// 3. The operator is defined for postfix use. See [`DefinedOperators`].
    ///
    /// 4. No whitespace occurs on the left, otherwise it would be infix.
    ///
    /// 5. If there's a token after, there must be whitespace between them, or
    ///    the token must be in [`Parser::ALLOWED_AFTER_POSTFIX`].
    pub fn consume_postfix(&mut self) -> Result<Token, Error> {
        const WANTED: &str = "a postfix operator";

        // 1
        if self.cursor == 0 {
            return Err(Error::PostfixOperatorAtStartOfInput);
        }

        // 2
        let token = self.tokens[self.cursor];
        if token.kind() != TokenKind::Operator {
            return Err(Error::Unexpected {
                wanted: WANTED,
                found: token.kind(),
            });
        }

        // 3
        if !self.operators.is_postfix(token.body()) {
            return Err(Error::OperatorNotDefinedAsPostfix);
        }

        // 4
        let before_token = self.tokens[self.cursor - 1];
        let space_before = before_token.span().end() != token.span().start();
        if space_before {
            return Err(Error::PostfixSpaceBefore);
        }

        // 5
        if self.cursor + 1 < self.tokens.len() {
            let after_token = self.tokens[self.cursor + 1];
            let space_after = token.span().end() != after_token.span().start();
            let is_allowed =
                Parser::ALLOWED_AFTER_POSTFIX.contains(&after_token.kind());

            if !(space_after || is_allowed) {
                return Err(Error::PostfixNoSpaceAfter);
            }
        }

        self.consume(token.kind(), WANTED)
    }

    /// Consume the next token iff it looks to be an infix operator, i.e. if the
    /// following conditions are met:
    ///
    /// 1. We need at least 1 token before for the lhs, and 2 after for the
    ///    operator we'll return and the rhs.
    ///
    /// 2. The next token is an `Kind::Operator`.
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
    ) -> Result<(Token, Associativity), Error> {
        const WANTED: &str = "a prefix operator";

        // 1
        // if self.cursor == 0 || (self.cursor + 2 >= self.tokens.len()) {
        if !(self.cursor >= 1 && self.cursor + 2 <= self.tokens.len()) {
            return Err(Error::InfixAtStartOrEnd);
        }

        // 2
        let token = self.tokens[self.cursor];
        if token.kind() != TokenKind::Operator {
            return Err(Error::Unexpected {
                wanted: WANTED,
                found: token.kind(),
            });
        }

        // 3
        let (associativity, found) = self
            .operators
            .get_infix(token.body())
            .ok_or(Error::OperatorNotDefinedAsInfix)?;

        // 4
        if found != wanted {
            return Err(Error::InfixWrongPrecedence);
        }

        // 5
        let before_tok = self.tokens[self.cursor - 1];
        let after_tok = self.tokens[self.cursor + 1];

        let space_before = before_tok.span().end() != token.span().start();
        let space_after = token.span().end() != after_tok.span().start();

        if space_before != space_after {
            return Err(Error::InfixUnbalancedWhitespace);
        }

        self.consume(token.kind(), WANTED)
            .map(|t| (t, associativity))
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
    fn prefix_1_at_eof() {
        let mut parser = Parser::new("-").unwrap();
        assert!(parser.consume_prefix().is_err());
    }

    #[test]
    fn prefix_2_non_operator() {
        let mut parser = Parser::new("= a").unwrap();
        assert!(parser.consume_prefix().is_err());
    }

    #[test]
    fn prefix_3_undefined() {
        let mut parser = Parser::new("<$> a").unwrap();
        assert!(!parser.operators.is_prefix("<$>"));
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
        assert!(parser.consume(TokenKind::Identifier, "a").is_ok());
        assert!(parser.consume_prefix().is_err());
    }

    #[test]
    fn prefix_5_missing_space_but_allowed() {
        let mut parser = Parser::new("(-a)").unwrap();
        assert!(parser
            .consume(TokenKind::Open(Delimiter::Parenthesis), "(")
            .is_ok());
        assert!(parser.consume_prefix().is_ok());
    }

    #[test]
    fn prefix_5_disallowed_but_space() {
        let mut parser = Parser::new("a -a").unwrap();
        assert!(parser.consume(TokenKind::Identifier, "a").is_ok());
        assert!(parser.consume_prefix().is_ok());
    }

    #[test]
    fn prefix_5_allowed_and_space() {
        let mut parser = Parser::new("( -a)").unwrap();
        assert!(parser
            .consume(TokenKind::Open(Delimiter::Parenthesis), "(")
            .is_ok());
        assert!(parser.consume_prefix().is_ok());
    }

    #[test]
    fn postfix() {
        let mut parser = Parser::new("a?").unwrap();
        assert!(parser.consume(TokenKind::Identifier, "a").is_ok());
        assert!(parser.consume_postfix().is_ok());
    }

    #[test]
    fn postfix_1_at_start() {
        let mut parser = Parser::new("?a").unwrap();
        assert!(parser.consume_postfix().is_err());
    }

    #[test]
    fn postfix_2_need_operator() {
        let mut parser = Parser::new("a a").unwrap();
        assert!(parser.consume(TokenKind::Identifier, "a").is_ok());
        assert!(parser.consume_postfix().is_err());
    }

    #[test]
    fn postfix_3_undefined() {
        let mut parser = Parser::new("a-").unwrap();
        assert!(!parser.operators.is_postfix("-"));
        assert!(parser.consume(TokenKind::Identifier, "a").is_ok());
        assert!(parser.consume_postfix().is_err());
    }

    #[test]
    fn postfix_4_no_left_whitespace() {
        let mut parser = Parser::new("a ?").unwrap();
        assert!(parser.consume(TokenKind::Identifier, "a").is_ok());
        assert!(parser.consume_postfix().is_err());
    }

    #[test]
    fn postfix_5_no_right_whitespace_and_disallowed() {
        let mut parser = Parser::new("a?a").unwrap();
        assert!(parser.consume(TokenKind::Identifier, "a").is_ok());
        assert!(parser.consume_postfix().is_err());
    }

    #[test]
    fn postfix_5_no_right_whitespace_but_allowed() {
        let mut parser = Parser::new("a?[").unwrap();
        assert!(parser.consume(TokenKind::Identifier, "a").is_ok());
        let result = parser.consume_postfix();
        assert!(result.is_ok(), "got {:?}", result);
    }

    #[test]
    fn postfix_5_whitespace_after() {
        let mut parser = Parser::new("a? a").unwrap();
        assert!(parser.consume(TokenKind::Identifier, "a").is_ok());
        assert!(parser.consume_postfix().is_ok());
    }

    #[test]
    fn postfix_with_dot() {
        let mut parser = Parser::new("a?.b").unwrap();
        assert!(parser.consume(TokenKind::Identifier, "a").is_ok());
        let result = parser.consume_postfix();
        assert!(result.is_ok(), "got {:?}", result);
        assert!(parser.consume(TokenKind::Dot, ".").is_ok());
    }

    #[test]
    fn postfix_with_other() {
        let mut parser = Parser::new("a?[b]").unwrap();
        assert!(parser.consume(TokenKind::Identifier, "a").is_ok());
        let result = parser.consume_postfix();
        assert!(result.is_ok(), "got {:?}", result);
    }

    #[test]
    fn infix() {
        let mut parser = Parser::new("a + a").unwrap();
        assert!(parser.consume(TokenKind::Identifier, "a").is_ok());
        assert!(parser.consume_infix(prec_of("+")).is_ok());
    }

    #[test]
    fn infix_wrong_precedence() {
        let mut parser = Parser::new("a + a").unwrap();
        assert!(parser.consume(TokenKind::Identifier, "a").is_ok());
        assert!(parser.consume_infix(prec_of("*")).is_err());
    }

    #[test]
    fn infix_no_spaces() {
        let mut parser = Parser::new("a+a").unwrap();
        assert!(parser.consume(TokenKind::Identifier, "a").is_ok());
        assert!(parser.consume_infix(prec_of("+")).is_ok());
    }

    #[test]
    fn infix_unbalanced_left() {
        let mut parser = Parser::new("a+ a").unwrap();
        assert!(parser.consume(TokenKind::Identifier, "a").is_ok());
        assert!(parser.consume_infix(prec_of("+")).is_err());
    }

    #[test]
    fn infix_unbalanced_right() {
        let mut parser = Parser::new("a +a").unwrap();
        assert!(parser.consume(TokenKind::Identifier, "a").is_ok());
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

        assert!(parser.consume(TokenKind::Identifier, "a").is_ok());

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

        assert!(parser.consume(TokenKind::Identifier, "a").is_ok());
        assert!(parser.is_empty());
    }
}
