#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub struct Position {
    pub line: usize,
    pub col: usize,
    pub offset: usize,
}

impl Position {
    pub fn new() -> Self {
        Position {
            line: 0,
            col: 0,
            offset: 0,
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub struct Span {
    pub start_pos: Position,
    pub end_pos: Position,
}

impl Span {
    pub fn set(start_pos: Position, end_pos: Position) -> Self {
        Span { start_pos, end_pos }
    }

    pub fn new() -> Self {
        Span::set(Position::new(), Position::new())
    }
}
