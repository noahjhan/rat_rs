use rat::compiler::Lexer;
use rat::compiler::{Category, Span, Token};
use rat::compiler::{Position, RatSource};

// #[test]
// fn test_print_full_file() {
//     let source = RatSource::init("data/file_exists.txt").unwrap();
//     let _ = Lexer::init(source).unwrap();
// }

#[test]
fn test_advance_string_literal() {
    let source = RatSource::init("data/string_literal.txt").unwrap();
    let mut lexer = Lexer::init(source);

    let expected_first = Token::new(
        Category::Literal,
        String::from("\"bonjour le monde!\""),
        Span {
            start: Position {
                line: 1,
                col: 1,
                offset: 0,
            },
            end: Position {
                line: 1,
                col: 20,
                offset: 19,
            },
        },
    );

    let actual_first = match lexer.next() {
        Ok(Some(token)) => token,
        Ok(None) => panic!("advance_token returned None before EOF"),
        Err(err) => panic!(
            "advance_token returned error for valid token. Got {:?}",
            err
        ),
    };

    assert_eq!(
        expected_first.kind, actual_first.kind,
        "advance_token returned incorrect kind. Expected {:?}, Got {:?}",
        expected_first.kind, actual_first.kind,
    );

    assert_eq!(
        expected_first.value, actual_first.value,
        "advance_token returned incorrect value. Expected {:?}, Got {:?}",
        expected_first.value, actual_first.value,
    );

    assert_eq!(
        expected_first.span, actual_first.span,
        "advance_token returned incorrect span. Expected {:?}, Got {:?}",
        expected_first.span, actual_first.span,
    );

    let actual_second = match lexer.next() {
        Ok(Some(token)) => token,
        Ok(None) => panic!("advance_token returned None before newline token"),
        Err(err) => panic!(
            "advance_token returned error while reading second token. Got {:?}",
            err
        ),
    };

    let expected_second = Token::new(
        Category::Punctuator,
        String::from("\n"),
        Span {
            start: Position {
                line: 1,
                col: 20,
                offset: 19,
            },
            end: Position {
                line: 2,
                col: 1,
                offset: 20,
            },
        },
    );

    assert_eq!(
        expected_second.kind, actual_second.kind,
        "advance_token returned incorrect second token kind. Expected {:?}, Got {:?}",
        expected_second.kind, actual_second.kind,
    );

    assert_eq!(
        expected_second.value, actual_second.value,
        "advance_token returned incorrect second token value. Expected {:?}, Got {:?}",
        expected_second.value, actual_second.value,
    );

    assert_eq!(
        expected_second.span, actual_second.span,
        "advance_token returned incorrect second token span. Expected {:?}, Got {:?}",
        expected_second.span, actual_second.span,
    );

    match lexer.next() {
        Ok(Some(token)) => panic!(
            "advance_token returned a valid token while reading EOF. Got {:?}",
            token
        ),
        Ok(None) => {}
        Err(err) => panic!(
            "advance_token returned error while reading EOF. Got {:?}",
            err
        ),
    };
}
