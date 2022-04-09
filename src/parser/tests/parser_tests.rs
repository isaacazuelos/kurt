//! Parser tests
//!
//! These tests use the parser to generate a different kind of AST than our
//! usual syntax.
//!
//! This serves as an example of how to use the parser, and a (user) test for if
//! the APIs make sense, without us needing to keep the parser fixed on our
//! specific grammar.

use diagnostic::Span;
use parser::{
    lexer::{Delimiter, TokenKind},
    Error, Parse, Parser,
};

// test grammar is like a dumb lisp, we have tuples and identifiers.
//
// S -> P | <identifier>
// P -> '(' sep_by_trailing<S>(',') ')'

#[derive(Debug)]
enum S<'a> {
    Identifier(&'a str),
    P(Box<P<'a>>),
}

impl<'a> Parse<'a> for S<'a> {
    fn parse_with(parser: &mut Parser<'a>) -> Result<S<'a>, parser::Error> {
        parser.depth_track(|parser| match parser.peek() {
            Some(TokenKind::Identifier) => {
                let token = parser
                    .consume(TokenKind::Identifier, "identifier")
                    .unwrap();
                Ok(S::Identifier(token.body()))
            }
            Some(TokenKind::Open(Delimiter::Parenthesis)) => {
                P::parse_with(parser).map(|p| S::P(Box::new(p)))
            }

            None => Err(Error::EOFExpecting("an identifier or a tuple")),
            Some(found) => Err(Error::Unexpected {
                wanted: "an identifier or a tuple",
                found,
            }),
        })
    }
}

#[derive(Debug)]
struct P<'a> {
    _open: Span,
    _before: Vec<S<'a>>,
    _close: Span,
}

impl<'a> Parse<'a> for P<'a> {
    fn parse_with(parser: &mut Parser<'a>) -> Result<P<'a>, parser::Error> {
        Ok(P {
            _open: parser
                .consume(
                    TokenKind::Open(Delimiter::Parenthesis),
                    "an open paren",
                )?
                .span(),

            _before: parser.sep_by_trailing(TokenKind::Comma)?.0,

            _close: parser
                .consume(
                    TokenKind::Close(Delimiter::Parenthesis),
                    "a close paren",
                )?
                .span(),
        })
    }
}

fn nested_parens(depth: usize) -> String {
    let mut buf = String::with_capacity(depth);
    for _ in 0..depth {
        buf.push('(');
    }
    buf.push('a');
    for _ in 0..depth {
        buf.push(')');
    }
    buf
}

#[test]
fn empty() {
    let s = S::parse("");
    assert!(s.is_err());
}

#[test]
fn identifier() {
    let syntax = S::parse("a");
    assert!(syntax.is_ok());
}

#[test]
fn parens() {
    let syntax = P::parse("(a)");
    assert!(syntax.is_ok(), "got {:?}", syntax);
}

#[test]
fn parens_nesting() {
    let syntax = P::parse("(((a)))");
    assert!(syntax.is_ok(), "got {:?}", syntax);
}

#[test]
fn sep_by_empty() {
    let syntax = P::parse("( )");
    assert!(syntax.is_ok(), "got {:?}", syntax);
}

#[test]
fn sep_by_simple() {
    let syntax = P::parse("( a, b )");
    assert!(syntax.is_ok(), "got {:?}", syntax);
}

#[test]
fn sep_by_trailing() {
    let syntax = P::parse("( a, b, )");
    assert!(syntax.is_ok(), "got {:?}", syntax);
}

#[test]
fn sep_by_trailing_empty() {
    let syntax = P::parse("( , )");
    assert!(syntax.is_ok(), "got {:?}", syntax);
}

#[test]
fn at_depth_limit() {
    let limit = nested_parens(Parser::MAX_DEPTH - 1);
    let at_limit = S::parse(&limit);
    assert!(at_limit.is_ok(), "failed with {:?}", at_limit);
}

#[test]
fn over_depth_limit() {
    let over_limit = nested_parens(Parser::MAX_DEPTH);
    let over_limit = S::parse(&over_limit);
    assert!(
        matches!(over_limit, Err(Error::ParserDepthExceeded)),
        "should be ParserDepthExceeded limit, but got {:?}",
        over_limit
    );
}
