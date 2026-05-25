use crate::compiler::{Category, ErrorKind, LexicalError, RatError};

pub enum Recovery {
    StopAtNewline,
    StopAtDelimiterOrNewline(char),
    StopAtAnyDelimiter,
    StopImmediately,
    StopAtWordBoundary,
}

impl Recovery {
    pub fn from_error(err: &RatError) -> Self {
        let opener = err
            .value
            .chars()
            .next()
            .filter(|ch| matches!(ch, '\'' | '"'));

        match &err.kind {
            ErrorKind::Lexical(LexicalError::UnterminatedString)
            | ErrorKind::Lexical(LexicalError::UnterminatedCharLiteral)
            | ErrorKind::Lexical(LexicalError::UnterminatedMultiLineComment) => Self::StopAtNewline,

            ErrorKind::Lexical(LexicalError::UnterminatedEscapeSequence)
            | ErrorKind::Lexical(LexicalError::UnterminatedUnicodeEscape)
            | ErrorKind::Lexical(LexicalError::InvalidEscapeSequence(_))
            | ErrorKind::Lexical(LexicalError::InvalidUnicodeEscapeOpener(_))
            | ErrorKind::Lexical(LexicalError::InvalidUnicodeEscapeDigit(_))
            | ErrorKind::Lexical(LexicalError::InvalidUnicodeDigitCount(_))
            | ErrorKind::Lexical(LexicalError::InvalidUnicodeCodepoint(_))
            | ErrorKind::Lexical(LexicalError::SurrogateCodepoint(_)) => match opener {
                Some(delim) => Self::StopAtDelimiterOrNewline(delim),
                None => Self::StopAtAnyDelimiter,
            },

            ErrorKind::Lexical(LexicalError::EmptyCharLiteral)
            | ErrorKind::Lexical(LexicalError::MultipleCharsInLiteral)
            | ErrorKind::Lexical(LexicalError::UnexpectedChar(_)) => Self::StopImmediately,

            ErrorKind::Lexical(LexicalError::UnexpectedCharAfterNumeric(_))
            | ErrorKind::Lexical(LexicalError::NoDigitsInNumericLiteral) => {
                Self::StopAtWordBoundary
            }

            _ => panic!("unexpected ErrorKind in Recovery::from_error()"),
        }
    }

    pub fn is_stop(&self, ch: char) -> bool {
        match self {
            Self::StopAtNewline => ch == '\n',
            Self::StopAtDelimiterOrNewline(delim) => ch == '\n' || ch == *delim,
            Self::StopAtAnyDelimiter => matches!(ch, '\'' | '"' | '\n'),
            Self::StopImmediately => true,
            Self::StopAtWordBoundary => Category::is_delimiter(ch),
        }
    }

    pub fn consume_stop(&self) -> bool {
        matches!(
            self,
            Self::StopAtDelimiterOrNewline(_) | Self::StopAtAnyDelimiter
        )
    }

    pub fn skips_escapes(&self) -> bool {
        matches!(
            self,
            Self::StopAtDelimiterOrNewline(_) | Self::StopAtAnyDelimiter
        )
    }
}
