use syntax::{ast, parser::Parser};

#[test]
fn literal() {
    let mut parser = Parser::new("123").unwrap();
    let literal: Result<ast::Literal, _> = parser.parse();
    assert!(literal.is_ok());
}

#[test]
fn empty_module() {
    let mut parser = Parser::new("").unwrap();
    let literal: Result<ast::Module, _> = parser.parse();
    assert!(literal.is_ok());
}

#[test]
fn empty_statements() {
    let mut parser = Parser::new(";;;;").unwrap();
    let literal: Result<ast::Module, _> = parser.parse();
    assert!(literal.is_ok());
}

#[test]
fn module_trailing_semicolon_optional() {
    let mut parser = Parser::new("0;").unwrap();
    let literal: Result<ast::Module, _> = parser.parse();
    assert!(literal.is_ok());

    let mut parser = Parser::new("0").unwrap();
    let literal: Result<ast::Module, _> = parser.parse();
    assert!(literal.is_ok());
}
