use rat::compiler::{Category, Lexer, Position, RatSource, Span, Token};

#[test]
fn test_advance_string_literal() {
    let source = RatSource::init("data/string_literal.txt").unwrap();
    let mut lexer = Lexer::init(source);

    let expected_first = Token::new(
        Category::Literal,
        String::from("\"bonjour le monde!\""),
        Span::new(
            Position {
                line: 1,
                col: 1,
                offset: 0,
            },
            Position {
                line: 1,
                col: 20,
                offset: 19,
            },
        ),
    );

    let actual_first = lexer
        .advance_token()
        .expect("advance returned an error")
        .expect("advance returned None before first token");

    assert_eq!(
        expected_first.kind, actual_first.kind,
        "wrong kind for first token"
    );
    assert_eq!(
        expected_first.value, actual_first.value,
        "wrong value for first token"
    );
    assert_eq!(
        expected_first.span, actual_first.span,
        "wrong span for first token"
    );

    let expected_second = Token::new(
        Category::Punctuator,
        String::from("\n"),
        Span::new(
            Position {
                line: 1,
                col: 20,
                offset: 19,
            },
            Position {
                line: 2,
                col: 1,
                offset: 20,
            },
        ),
    );

    let actual_second = lexer
        .advance_token()
        .expect("advance returned an error")
        .expect("advance returned None before newline token");

    assert_eq!(
        expected_second.kind, actual_second.kind,
        "wrong kind for second token"
    );
    assert_eq!(
        expected_second.value, actual_second.value,
        "wrong value for second token"
    );
    assert_eq!(
        expected_second.span, actual_second.span,
        "wrong span for second token"
    );

    match lexer.advance_token() {
        Ok(None) => {}
        Ok(Some(tok)) => panic!("expected EOF but got token {:?}", tok),
        Err(err) => panic!("expected EOF but got error {:?}", err),
    }
}
