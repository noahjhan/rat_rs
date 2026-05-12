use std::io;

#[derive(Debug)]
pub enum RatError {
    IoError(io::Error),
    LexicalError(String),
    InternalError(String),
    // ParseError(String),
}

impl From<io::Error> for RatError {
    fn from(e: io::Error) -> Self {
        RatError::IoError(e)
    }
}
