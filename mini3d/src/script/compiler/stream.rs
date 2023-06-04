use std::str::Chars;

use super::token::Location;

pub(crate) struct SourceStream<'a> {
    chars: Chars<'a>,
    next_location: Location,
}

impl<'a> SourceStream<'a> {
    pub(crate) fn new(source: &'a str) -> Self {
        Self {
            chars: source.chars(),
            next_location: Location::new(1, 1),
        }
    }
}

impl<'a> Iterator for SourceStream<'a> {
    type Item = (char, Location);

    fn next(&mut self) -> Option<Self::Item> {
        let mut c = self.chars.next()?;
        while c == '\n' {
            c = self.chars.next()?;
            self.next_location.line += 1;
            self.next_location.column = 1;
        }
        let location = self.next_location;
        self.next_location.column += 1;
        Some((c, location))
    }
}
