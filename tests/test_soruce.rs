use rat::compiler::Position;
use rat::compiler::RatSource;

#[test]
fn test_init_file_exists() {
    let expected = Position {
        line_num: 1,
        col_num: 1,
        offset: 0,
    };

    let result = RatSource::init("data/file_exists.txt");
    assert!(result.is_ok());

    let source = result.unwrap();
    let actual = source.position();

    assert_eq!(
        expected, actual,
        "Expected init position {:?}, got {:?}",
        expected, actual
    );
}

#[test]
fn test_init_nonexistent_file() {
    let result = RatSource::init("data/nonexistent_file.txt");

    assert!(
        result.is_err(),
        "Expected init to fail for nonexistent file"
    );
}

#[test]
fn test_read_file_exists() {
    let expected_pos = Position {
        line_num: 3,
        col_num: 1,
        offset: 32,
    };

    let expected_output = "hello, world.\nbonjour le monde!\n";

    let mut source = RatSource::init("data/file_exists.txt").unwrap();
    let mut actual_output = String::new();

    while let Some(b) = source.read().unwrap() {
        actual_output.push(b as char);
    }

    let actual_pos = source.position();

    assert_eq!(
        expected_pos, actual_pos,
        "Read stored incorrect position. Expected {:?} but got {:?}",
        expected_pos, actual_pos,
    );

    assert_eq!(
        expected_output, actual_output,
        "Read returned incorrect output. Expected {:?} but got {:?}",
        expected_output, actual_output,
    );
}

#[test]
fn test_peek_file_exists() {
    let expected_pos = Position {
        line_num: 2,
        col_num: 1,
        offset: 14,
    };
    let expected_peek_output = b'b';
    let mut source = RatSource::init("data/file_exists.txt").unwrap();

    for _ in 0..14 {
        match source.read().unwrap() {
            Some(_) => continue,
            None => break,
        }
    }

    let actual_peek_output = source.peek().unwrap().expect("Expected a byte but got EOF");

    assert_eq!(
        expected_peek_output, actual_peek_output,
        "Peek returned incorrect output. Expected {:?} but got {:?}",
        expected_peek_output, actual_peek_output,
    );

    let actual_pos = source.position();
    assert_eq!(
        expected_pos, actual_pos,
        "Peek stored incorrect position. Expected {:?} but got {:?}",
        expected_pos, actual_pos,
    );
}
