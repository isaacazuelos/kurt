//! Tuple types, with optional keyword tags.

use crate::lexer::{Delimiter, TokenKind};

use super::*;

#[derive(Debug)]
pub struct Tuple<'a> {
    tag: Option<(Span, Identifier)>,
    open: Span,
    elements: Vec<Expression<'a>>,
    commas: Vec<Span>,
    close: Span,
}

impl<'a> Tuple<'a> {
    pub fn is_tagged(&self) -> bool {
        self.tag.is_some()
    }

    pub fn tag(&self) -> Option<&Identifier> {
        if let Some((_, id)) = &self.tag {
            Some(id)
        } else {
            None
        }
    }

    pub fn tag_span(&self) -> Option<Span> {
        if let Some((span, _)) = self.tag {
            Some(span)
        } else {
            None
        }
    }

    /// The span for the opening delimiter token.
    pub fn open(&self) -> Span {
        self.open
    }

    /// The span fo the closing delimiter token.
    pub fn close(&self) -> Span {
        self.close
    }
}

impl<'a> Sequence for Tuple<'a> {
    type Element = Expression<'a>;

    const SEPARATOR: TokenKind = TokenKind::Comma;

    fn elements(&self) -> &[Self::Element] {
        &self.elements
    }

    fn separators(&self) -> &[Span] {
        &self.commas
    }
}

impl<'a> Syntax for Tuple<'a> {
    fn span(&self) -> Span {
        self.open + self.close()
    }
}

impl<'a> Parse<'a> for Tuple<'a> {
    type SyntaxError = SyntaxError;

    fn parse_with(parser: &mut Parser<'a>) -> SyntaxResult<Self> {
        let tag = if parser.peek_kind() == Some(TokenKind::Colon)
            && parser.peek_kind_nth(1) == Some(TokenKind::Identifier)
        {
            let colon = parser.consume(TokenKind::Colon).unwrap();
            let id: Identifier = parser.parse()?;
            let span = id.span() + colon.span();
            Some((span, id))
        } else {
            None
        };

        let open = parser
            .consume(TokenKind::Open(Delimiter::Parenthesis))
            .ok_or_else(|| SyntaxError::ListNoOpen(parser.next_span()))?
            .span();

        let (elements, commas) = parser.sep_by_trailing(TokenKind::Comma)?;

        let close = parser
            .consume(TokenKind::Close(Delimiter::Parenthesis))
            .ok_or_else(|| SyntaxError::ListNoClose(open, parser.next_span()))?
            .span();

        Ok(Tuple {
            tag,
            open,
            elements,
            commas,
            close,
        })
    }
}

#[cfg(test)]
mod tuples {
    use super::*;

    #[test]
    fn empty() {
        let mut parser = Parser::new(" () ").unwrap();
        let result = parser.parse::<Tuple>();
        assert!(result.is_ok());
        assert!(parser.is_empty());
    }

    #[test]
    fn empty_tagged() {
        let mut parser = Parser::new(" :ok() ").unwrap();
        let result = parser.parse::<Tuple>();
        assert!(result.is_ok());
        assert!(parser.is_empty());
    }

    #[test]
    fn empty_with_comma() {
        let mut parser = Parser::new(" (,) ").unwrap();
        let result = parser.parse::<Tuple>();
        assert!(result.is_ok());
        assert!(parser.is_empty());
    }

    #[test]
    fn simple() {
        let mut parser = Parser::new(" (1,2,3,) ").unwrap();
        let result = parser.parse::<Tuple>();
        assert!(result.is_ok());
        assert!(parser.is_empty());
    }
}
