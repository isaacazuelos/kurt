use syntax::lexer::*;

#[test]
fn lexer_empty() {
    let mut lexer = Lexer::new("");
    assert!(lexer.is_empty());
    assert!(matches!(lexer.token(), Err(Error::UnexpectedEOF(_))));
}

#[test]
fn lexer_whitespace() {
    let mut lexer = Lexer::new("  \t\r\n  a    \t\r\n ");

    assert!(!lexer.is_empty());
    let _ = lexer.token();
    assert!(lexer.is_empty());
    assert!(matches!(lexer.token(), Err(Error::UnexpectedEOF(_))));
}

#[test]
fn lexer_identifier_simple() {
    let mut lexer = Lexer::new("input");
    assert_eq!(lexer.token().unwrap().kind(), Kind::Identifier);
}

#[test]
fn lexer_identifier_start_underscore() {
    let mut lexer = Lexer::new("_input");
    assert_eq!(lexer.token().unwrap().kind(), Kind::Identifier);
}

#[test]
fn lexer_punctuation() {
    let mut lexer = Lexer::new("( @; [], . => =>>");

    assert_eq!(
        lexer.token().unwrap().kind(),
        Kind::Open(Delimiter::Parenthesis)
    );
    assert_eq!(lexer.token().unwrap().kind(), Kind::At);
    assert_eq!(lexer.token().unwrap().kind(), Kind::Semicolon);
    assert_eq!(
        lexer.token().unwrap().kind(),
        Kind::Open(Delimiter::Bracket)
    );
    assert_eq!(
        lexer.token().unwrap().kind(),
        Kind::Close(Delimiter::Bracket)
    );
    assert_eq!(lexer.token().unwrap().kind(), Kind::Comma);
    assert_eq!(lexer.token().unwrap().kind(), Kind::Dot);
    assert_eq!(lexer.token().unwrap().kind(), Kind::DoubleArrow);
    assert_eq!(lexer.token().unwrap().kind(), Kind::Operator);
}

#[test]
fn lexer_operator_intuition() {
    let mut lexer = Lexer::new("a<$>b");
    assert_eq!(lexer.token().unwrap().kind(), Kind::Identifier);
    assert_eq!(lexer.token().unwrap().kind(), Kind::Operator);
    assert_eq!(lexer.token().unwrap().kind(), Kind::Identifier);

    let mut lexer = Lexer::new("a<$> =<<b");
    assert_eq!(lexer.token().unwrap().kind(), Kind::Identifier);
    assert_eq!(lexer.token().unwrap().kind(), Kind::Operator);
    assert_eq!(lexer.token().unwrap().kind(), Kind::Operator);
    assert_eq!(lexer.token().unwrap().kind(), Kind::Identifier);

    let mut lexer = Lexer::new("List<T> > Bar.Foo");
    assert_eq!(lexer.token().unwrap().kind(), Kind::Identifier); // List
    assert_eq!(lexer.token().unwrap().kind(), Kind::Operator); // <
    assert_eq!(lexer.token().unwrap().kind(), Kind::Identifier); // T
    assert_eq!(lexer.token().unwrap().kind(), Kind::Operator); // >
    assert_eq!(lexer.token().unwrap().kind(), Kind::Operator); // >
    assert_eq!(lexer.token().unwrap().kind(), Kind::Identifier); // Bar
    assert_eq!(lexer.token().unwrap().kind(), Kind::Dot); // .
    assert_eq!(lexer.token().unwrap().kind(), Kind::Identifier); // Foo
}

#[test]
fn comments_no_newline() {
    let mut lexer = Lexer::new("// comment");
    assert!(matches!(
        lexer.token().unwrap().kind(),
        Kind::Comment(Comment::Line)
    ));
}

#[test]
fn comments_doc() {
    let mut lexer = Lexer::new("/// doc comment");
    assert!(matches!(
        lexer.token().unwrap().kind(),
        Kind::Comment(Comment::Doc)
    ));
}

#[test]
fn reserved() {
    let mut lexer = Lexer::new("#test");
    assert!(matches!(lexer.token(), Err(Error::Reserved(_, '#'))));
}

#[test]
fn dots() {
    let mut lexer = Lexer::new(". .. ... ....");
    assert_eq!(lexer.token().unwrap().kind(), Kind::Dot);
    assert_eq!(lexer.token().unwrap().kind(), Kind::Range);
    assert_eq!(lexer.token().unwrap().kind(), Kind::Spread);
    assert_eq!(lexer.token().unwrap().kind(), Kind::Operator);
}

#[test]
fn unicode_identifier() {
    // Google translate tells me this is 'identifier'.
    let mut lexer = Lexer::new("标识符");
    assert_eq!(lexer.token().unwrap().kind(), Kind::Identifier);
    assert!(lexer.is_empty());

    // Google translate tells me this is 'word'.
    let mut lexer = Lexer::new("let لفظ = word;");
    assert_eq!(lexer.token().unwrap().kind(), Kind::Reserved(Reserved::Let));
    assert_eq!(lexer.token().unwrap().kind(), Kind::Identifier);
    assert_eq!(lexer.token().unwrap().kind(), Kind::Operator);
    assert_eq!(lexer.token().unwrap().kind(), Kind::Identifier);
    assert_eq!(lexer.token().unwrap().kind(), Kind::Semicolon);
    assert!(lexer.is_empty());
}
