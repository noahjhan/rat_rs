use crate::compiler::{RatError, RatSource};

// #[derive(Eq)]
pub struct Render<'a> {
    pub source: &'a RatSource,
}

impl<'a> Render<'a> {
    pub fn init(source: &'a RatSource) -> Self {
        Render { source }
    }

    pub fn print(&mut self, err: RatError) {
        self.print_header(&err);
        self.print_position(&err);
        self.print_message(&err);
        println!();
        self.print_source_line(&err);
        self.print_span(&err);
        println!();
    }

    fn print_header(&self, err: &RatError) {
        if err.is_fatal() {
            eprintln!("error:");
        } else {
            eprintln!("warning:");
        }
    }

    fn print_message(&self, err: &RatError) {
        eprintln!("{}", err);
    }

    fn print_position(&self, err: &RatError) {
        let start_pos = err.span.start_pos;
        eprint!(
            "from line: {}, column: {} to ",
            start_pos.line, start_pos.col
        );
        let end_pos = err.span.end_pos;
        eprintln!("line: {}, column: {}", end_pos.line, end_pos.col);
    }

    pub fn print_source_line(&self, err: &RatError) {
        match self.source.from_span(err.span) {
            Some(string) => {
                print!("{}", string);
            }
            None => {
                panic!("No line found in print_source_line()");
            }
        };
    }

    fn print_span(&self, err: &RatError) {
        let start_col = err.span.start_pos.col.saturating_sub(1);
        let end_col = err.span.end_pos.col.saturating_sub(1);
        let caret_col = end_col.saturating_sub(1);
        print!("{}", " ".repeat(start_col));
        let line_diff = err
            .span
            .end_pos
            .line
            .saturating_sub(err.span.start_pos.line);
        if line_diff == 0 {
            let width = caret_col.saturating_sub(start_col);
            if width == 0 {
                print!("‾");
            } else {
                print!("‾");
                if width > 1 {
                    print!("{}", "‾".repeat(width.saturating_sub(1)));
                }
                print!("^");
            }
        } else if line_diff == 1 && err.span.end_pos.col == 1 {
            print!("‾^");
        } else {
            print!("‾");
            if start_col != caret_col {
                println!();
                print!("{}", " ".repeat(caret_col));
                print!("^");
            }
        }
    }
}
