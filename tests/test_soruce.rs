use rat::compiler::source::Position;
use rat::compiler::source::Source;

#[test]
fn test_init_file_exists() {
    let expected = Position {
        line_num: 0,
        col_num: 0,
        offset: 0,
    };

    let result = Source::init("data/file_exists.txt");
    assert!(result.is_ok());

    let source = result.unwrap();
    let actual = source.position();

    assert_eq!(
        actual, expected,
        "Expected init position {:?}, got {:?}",
        expected, actual
    );
}

#[test]
fn test_init_nonexistent_file() {
    let result = Source::init("data/nonexistent_file.txt");

    assert!(
        result.is_err(),
        "Expected init to fail for nonexistent file"
    );
}

#[test]
fn test_read_file_exists() {
    let expected_pos = Position {
        line_num: 2,
        col_num: 0,
        offset: 32,
    };

    let expected_output = "hello, world.\nbonjour le monde!\n";

    let mut source = Source::init("data/file_exists.txt").unwrap();
    let mut actual_output = String::new();

    while let Some(b) = source.read().unwrap() {
        actual_output.push(b as char);
    }

    let actual_pos = source.position();

    assert_eq!(actual_pos, expected_pos);
    assert_eq!(actual_output, expected_output);
}
