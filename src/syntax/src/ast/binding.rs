//! Binding statements, things like `let` and `var`.

use crate::lexer::{Reserved, Token, TokenKind as Kind};

use super::*;

#[derive(Debug)]
pub struct Binding<'a> {
    keyword: Token<'a>,
    name: Identifier<'a>,
    equals: Token<'a>,
    body: Expression<'a>,
}

impl Binding<'_> {
    /// Is this a `var` binding?
    pub fn is_var(&self) -> bool {
        self.keyword.kind() == Kind::Reserved(Reserved::Var)
    }

    /// Is this a `let` binding?
    pub fn is_let(&self) -> bool {
        self.keyword.kind() == Kind::Reserved(Reserved::Let)
    }

    /// A reference to the expression which is evaluated to be bound to the
    /// name.
    pub fn body(&self) -> &Expression {
        &self.body
    }

    /// The identifier the value is being bound to.
    pub fn name(&self) -> &Identifier {
        &self.name
    }

    /// The token used for the `=` in the binding site.
    pub fn equals(&self) -> &Token {
        &self.equals
    }
}

impl Syntax for Binding<'_> {
    const NAME: &'static str = "a `let` or `var` binding";

    fn span(&self) -> Span {
        self.keyword.span() + self.body.span()
    }
}

impl<'a> Parse<'a> for Binding<'a> {
    fn parse_with(parser: &mut Parser<'a>) -> Result<Binding<'a>, Error> {
        let keyword = match parser.peek() {
            Some(Kind::Reserved(Reserved::Let | Reserved::Var)) => {
                let token = parser.advance().unwrap();
                Ok(token)
            }
            Some(found) => Err(Error::Unexpected {
                wanted: Binding::NAME,
                found,
            }),
            None => Err(Error::EOFExpecting(Binding::NAME)),
        }?;

        let name = parser.parse()?;

        let equals = match parser.peek() {
            None => Err(Error::EOFExpecting(Kind::Equals.name())),
            Some(Kind::Equals) => Ok(parser.advance().unwrap()),
            Some(found) => Err(Error::Unexpected {
                wanted: Kind::Equals.name(),
                found,
            }),
        }?;

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
    fn test_binding() {
        let mut parser = Parser::new("let x = x").unwrap();
        let binding = parser.parse::<Binding>();
        assert!(binding.is_ok(), "binding expected, but got {:?}", binding);
        assert!(parser.is_empty());
    }
}
