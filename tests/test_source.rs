use rat::compiler::{Position, RatSource};

#[test]
fn test_init_file_exists() {
    let expected = Position {
        line: 1,
        col: 1,
        offset: 0,
    };
    let filepath = String::from("data/file_exists.txt");
    let source = RatSource::init(filepath).unwrap();
    assert_eq!(expected, source.position());
}

#[test]
fn test_init_nonexistent_file() {
    let filepath = String::from("data/nonexistent_file.txt");
    assert!(
        RatSource::init(filepath).is_err(),
        "expected init to fail for nonexistent file"
    );
}

#[test]
fn test_read() {
    let expected_pos = Position {
        line: 3,
        col: 1,
        offset: 32,
    };
    let expected_output = "hello, world.\nbonjour le monde!\n";

    let filepath = string::from("data/file_exists.txt");
    let source = ratsource::init(filepath).unwrap();
    let mut actual_output = String::new();

    while let Some(ch) = source.read().unwrap() {
        actual_output.push(ch);
    }

    assert_eq!(
        expected_pos,
        source.position(),
        "read stored incorrect position"
    );
    assert_eq!(
        expected_output, actual_output,
        "read returned incorrect output"
    );
}

#[test]
fn test_peek() {
    let expected_pos = Position {
        line: 2,
        col: 1,
        offset: 14,
    };
    let filepath = String::from("data/file_exists.txt");
    let source = RatSource::init(filepath).unwrap();

    for _ in 0..14 {
        if source.read().unwrap().is_none() {
            break;
        }
    }

    let peeked = source.peek().unwrap().expect("expected a char but got EOF");
    assert_eq!('b', peeked, "peek returned incorrect character");

    assert_eq!(
        expected_pos,
        source.position(),
        "peek must not advance position"
    );
}

#[test]
fn test_bad_utf8() {
    let filepath = String::from("data/bad_utf8.txt");
    let mut source = RatSource::init(filepath).unwrap();
    match source.read() {
        Ok(_) => panic!("read returned OK upon reading bad utf8 character"),
        Err(_) => {}
    }
}
