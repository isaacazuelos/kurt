use syntax::{ast, Parse};

#[test]
fn literal() {
    let syntax = ast::Literal::parse("123");
    assert!(syntax.is_ok());
}

#[test]
fn binding() {
    let syntax = ast::Binding::parse("let x = 0");
    assert!(syntax.is_ok());

    let syntax = ast::Binding::parse("var x = 0");
    assert!(syntax.is_ok());
}

#[test]
fn empty_module() {
    let syntax = ast::Module::parse("");
    assert!(syntax.is_ok());
}

#[test]
fn empty_statements() {
    let syntax = ast::Module::parse(";;;;");
    assert!(syntax.is_ok());
}

#[test]
fn module_trailing_semicolon_optional() {
    let syntax = ast::Module::parse("0;");
    assert!(syntax.is_ok());

    let syntax = ast::Module::parse("0");
    assert!(syntax.is_ok());
}
