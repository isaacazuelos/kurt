use parser::lexer::*;

#[test]
fn lexer_empty() {
    let mut lexer = Lexer::new("");
    assert!(lexer.is_empty());
    assert!(matches!(lexer.token(), Err(Error::UnexpectedEOF(_))));
}

#[test]
fn lexer_empty_whitespace_only() {
    let lexer = Lexer::new("  ");
    assert!(lexer.is_empty());
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
    assert_eq!(lexer.token().unwrap().kind(), TokenKind::Identifier);
}

#[test]
fn lexer_identifier_start_underscore() {
    let mut lexer = Lexer::new("_input");
    assert_eq!(lexer.token().unwrap().kind(), TokenKind::Identifier);
}

#[test]
fn lexer_punctuation() {
    let mut lexer = Lexer::new("( @; [], . = => =>>");

    assert_eq!(
        lexer.token().unwrap().kind(),
        TokenKind::Open(Delimiter::Parenthesis)
    );
    assert_eq!(lexer.token().unwrap().kind(), TokenKind::At);
    assert_eq!(lexer.token().unwrap().kind(), TokenKind::Semicolon);
    assert_eq!(
        lexer.token().unwrap().kind(),
        TokenKind::Open(Delimiter::Bracket)
    );
    assert_eq!(
        lexer.token().unwrap().kind(),
        TokenKind::Close(Delimiter::Bracket)
    );
    assert_eq!(lexer.token().unwrap().kind(), TokenKind::Comma);
    assert_eq!(lexer.token().unwrap().kind(), TokenKind::Dot);
    assert_eq!(lexer.token().unwrap().kind(), TokenKind::Equals);
    assert_eq!(lexer.token().unwrap().kind(), TokenKind::DoubleArrow);
    assert_eq!(lexer.token().unwrap().kind(), TokenKind::Operator);
}

#[test]
fn lexer_operator_intuition() {
    let mut lexer = Lexer::new("a<$>b");
    assert_eq!(lexer.token().unwrap().kind(), TokenKind::Identifier);
    assert_eq!(lexer.token().unwrap().kind(), TokenKind::Operator);
    assert_eq!(lexer.token().unwrap().kind(), TokenKind::Identifier);

    let mut lexer = Lexer::new("a<$> =<<b");
    assert_eq!(lexer.token().unwrap().kind(), TokenKind::Identifier);
    assert_eq!(lexer.token().unwrap().kind(), TokenKind::Operator);
    assert_eq!(lexer.token().unwrap().kind(), TokenKind::Operator);
    assert_eq!(lexer.token().unwrap().kind(), TokenKind::Identifier);

    let mut lexer = Lexer::new("List<T> > Bar.Foo");
    assert_eq!(lexer.token().unwrap().kind(), TokenKind::Identifier); // List
    assert_eq!(lexer.token().unwrap().kind(), TokenKind::Operator); // <
    assert_eq!(lexer.token().unwrap().kind(), TokenKind::Identifier); // T
    assert_eq!(lexer.token().unwrap().kind(), TokenKind::Operator); // >
    assert_eq!(lexer.token().unwrap().kind(), TokenKind::Operator); // >
    assert_eq!(lexer.token().unwrap().kind(), TokenKind::Identifier); // Bar
    assert_eq!(lexer.token().unwrap().kind(), TokenKind::Dot); // .
    assert_eq!(lexer.token().unwrap().kind(), TokenKind::Identifier); // Foo
}

#[test]
fn comments_no_newline() {
    let mut lexer = Lexer::new("// comment");
    assert!(matches!(
        lexer.token().unwrap().kind(),
        TokenKind::Comment(Comment::Line)
    ));
}

#[test]
fn comments_doc() {
    let mut lexer = Lexer::new("/// doc comment");
    assert!(matches!(
        lexer.token().unwrap().kind(),
        TokenKind::Comment(Comment::Doc)
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
    assert_eq!(lexer.token().unwrap().kind(), TokenKind::Dot);
    assert_eq!(lexer.token().unwrap().kind(), TokenKind::Range);
    assert_eq!(lexer.token().unwrap().kind(), TokenKind::Spread);
    assert_eq!(lexer.token().unwrap().kind(), TokenKind::Operator);
}

#[test]
fn unicode_identifier() {
    // Google translate tells me this is 'identifier'.
    let mut lexer = Lexer::new("标识符");
    assert_eq!(lexer.token().unwrap().kind(), TokenKind::Identifier);
    assert!(lexer.is_empty());

    // Google translate tells me this is 'word'.
    let mut lexer = Lexer::new("let لفظ = word;");
    assert_eq!(
        lexer.token().unwrap().kind(),
        TokenKind::Reserved(Reserved::Let)
    );
    assert_eq!(lexer.token().unwrap().kind(), TokenKind::Identifier);
    assert_eq!(lexer.token().unwrap().kind(), TokenKind::Equals);
    assert_eq!(lexer.token().unwrap().kind(), TokenKind::Identifier);
    assert_eq!(lexer.token().unwrap().kind(), TokenKind::Semicolon);
    assert!(lexer.is_empty());
}

#[test]
fn space_cadet() {
    // Here are the symbols on the face of a [Space Cadet Keyboard][link], save
    // for the floor and ceiling opening characters on Z and X. These should all
    // be lexed as operators.
    //
    // [link]: https://en.wikipedia.org/wiki/Space-cadet_keyboard
    let mut lexer =
        Lexer::new("∧ ∨ ∪ ∩ ⊂ ⊃ ∀ ∞ ∃ ∂ ⊥ ⊤ ⊢ ⊣ ↑ ↓ ← → ↔ ≠ ≃ ≡ ≤ ≥");

    let mut count = 0;

    while let Ok(token) = lexer.token() {
        count += 1;
        assert_eq!(
            token.kind(),
            TokenKind::Operator,
            "expected {} to be an operator.",
            token.body()
        );
    }

    assert_eq!(count, 24);
}
