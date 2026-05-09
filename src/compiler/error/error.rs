use std::io;

#[derive(Debug)]
pub enum RatError {
    Io(io::Error),
    LexicalError(String),
    // ParseError(String),
}

impl From<io::Error> for RatError {
    fn from(e: io::Error) -> Self {
        RatError::Io(e)
    }
}
