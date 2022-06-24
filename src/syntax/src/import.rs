//! `import` statements allow modules to refer to other modules.

use diagnostic::Span;

use parser::{lexer::Reserved, Parse};

use super::*;

#[derive(Debug)]
pub struct Import {
    import: Span,
    name: Identifier,
}

impl Import {
    pub fn name(&self) -> &Identifier {
        &self.name
    }
}

impl Syntax for Import {
    fn span(&self) -> Span {
        self.import + self.name.span()
    }
}

impl<'a> Parse<'a> for Import {
    type SyntaxError = SyntaxError;

    fn parse_with(
        parser: &mut Parser<'a>,
    ) -> Result<Self, Error<Self::SyntaxError>> {
        let import = parser
            .consume(TokenKind::Reserved(Reserved::Import))
            .ok_or_else(|| SyntaxError::ImportNoKeyword(parser.next_span()))?
            .span();

        let name = parser.parse()?;

        Ok(Import { import, name })
    }
}

#[cfg(test)]
mod parser_tests {
    use super::*;

    #[test]
    fn test_import() {
        let mut parser = Parser::new("import foo").unwrap();
        let syntax = parser.parse::<Import>();
        assert!(syntax.is_ok(), "expected an import but got {:?}", syntax);
        assert!(parser.is_empty());
    }
}
