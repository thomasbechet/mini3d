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
        let location = self.next_location;
        if c == '\n' {
            c = ' ';
            self.next_location.line += 1;
            self.next_location.column = 1;
        } else {
            self.next_location.column += 1;
        }
        Some((c, location))
    }
}

#[test]
fn test_multiple_lines() {
    let script = "a\nb";
    let mut stream = SourceStream::new(script);
    assert_eq!(stream.next().unwrap(), ('a', Location::new(1, 1)));
    assert_eq!(stream.next().unwrap(), (' ', Location::new(1, 2)));
    assert_eq!(stream.next().unwrap(), ('b', Location::new(2, 1)));
}
