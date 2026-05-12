use rat::compiler::Lexer;
use rat::compiler::RatSource;

#[test]
fn test_print_full_file() {
    let source = RatSource::init("data/file_exists.txt").unwrap();
    let _ = Lexer::init(source).unwrap();
}
