//! Binding statements like `let x = 10`

use parser::{
    lexer::{Reserved, Token, TokenKind as Kind},
    Parse,
};

use super::*;

/// Binding statements.
///
/// # Grammar
///
/// [`Binding`] := `let`[Identifier] `=` [`Expression`]  
/// [`Binding`] := `let`[`Identifier`] `=` [`Expression`]  
#[derive(Debug)]
pub struct Binding<'a> {
    keyword: Token<'a>,
    name: Identifier,
    equals: Span,
    body: Expression<'a>,
}

impl Binding<'_> {
    ///
    pub fn keyword(&self) -> &Token {
        &self.keyword
    }

    /// Is this a `var` binding?
    pub fn is_var(&self) -> bool {
        self.keyword.kind() == Kind::Reserved(Reserved::Var)
    }

    /// Is this a `let` binding?
    pub fn is_let(&self) -> bool {
        self.keyword.kind() == Kind::Reserved(Reserved::Let)
    }

    /// The expression on the right of the `=` which is evaluated and bound.
    pub fn body(&self) -> &Expression {
        &self.body
    }

    /// The name the value is being bound to.
    pub fn name(&self) -> &Identifier {
        &self.name
    }

    /// The span of the `=` used in this statement.
    pub fn equals(&self) -> Span {
        self.equals
    }
}

impl Syntax for Binding<'_> {
    fn span(&self) -> Span {
        self.keyword.span() + self.body.span()
    }
}

impl<'a> Parse<'a> for Binding<'a> {
    type SyntaxError = SyntaxError;

    fn parse_with(parser: &mut Parser<'a>) -> SyntaxResult<Binding<'a>> {
        let keyword = parser
            .consume_if(|t| {
                matches!(
                    t.kind(),
                    Kind::Reserved(Reserved::Let | Reserved::Var),
                )
            })
            .ok_or_else(|| {
                SyntaxError::BindingNoReserved(parser.peek_span())
            })?;

        let name = parser.parse()?;

        let equals = parser
            .consume_if(|token| token.body() == "=")
            .ok_or_else(|| {
                SyntaxError::BindingNoEquals(
                    keyword.span(),
                    keyword.body() == "let",
                    parser.peek_span(),
                )
            })?
            .span();

        let body = parser.parse()?;

        Ok(Binding {
            keyword,
            name,
            equals,
            body,
        })
    }
}

#[cfg(test)]
mod parser_tests {
    use super::*;

    #[test]
    fn test_let() {
        let mut parser = Parser::new("let x = x").unwrap();
        let binding = parser.parse::<Binding>();
        assert!(binding.is_ok(), "binding expected, but got {:?}", binding);
        assert!(parser.is_empty());
    }

    #[test]
    fn test_var() {
        let mut parser = Parser::new("let x = x").unwrap();
        let binding = parser.parse::<Binding>();
        assert!(binding.is_ok(), "binding expected, but got {:?}", binding);
        assert!(parser.is_empty());
    }
}
