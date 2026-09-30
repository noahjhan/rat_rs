use crate::compiler::{Category, LexicalError};

pub enum Recovery {
    StopAtNewline,
    StopAtDelimiterOrNewline(char),
    StopAtAnyDelimiter,
    StopImmediately,
    StopAtWordBoundary,
}

impl Recovery {
    pub fn from_error(kind: &LexicalError, opener: Option<char>) -> Self {
        use LexicalError::*;
        match kind {
            UnterminatedString | UnterminatedCharLiteral | UnterminatedMultiLineComment => {
                Self::StopAtNewline
            }
            UnterminatedEscapeSequence
            | UnterminatedUnicodeEscape
            | InvalidEscapeSequence(_)
            | InvalidUnicodeEscapeOpener(_)
            | InvalidUnicodeEscapeDigit(_)
            | InvalidUnicodeDigitCount(_)
            | InvalidUnicodeCodepoint(_)
            | SurrogateCodepoint(_) => match opener {
                Some(delim) => Self::StopAtDelimiterOrNewline(delim),
                None => Self::StopAtAnyDelimiter,
            },
            EmptyCharLiteral | MultipleCharsInLiteral | UnexpectedChar(_) => Self::StopImmediately,
            UnexpectedCharAfterNumeric(_) | NoDigitsInNumericLiteral => Self::StopAtWordBoundary,
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
