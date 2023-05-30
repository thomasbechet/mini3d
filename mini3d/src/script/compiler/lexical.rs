use std::{iter::Peekable, str::CharIndices};

use super::{
    error::{CompilationError, LexicalError},
    token::{Span, Token, TokenKind},
};

pub(crate) struct Lexer<'a> {
    chars: Peekable<CharIndices<'a>>,
    pub(crate) source: &'a str,
    current_line: usize,
    current_column: usize,
}

fn is_identifier_character(c: char) -> bool {
    c.is_alphanumeric() || c == '_'
}

impl<'a> Lexer<'a> {
    pub(crate) fn new(source: &'a str) -> Self {
        Self {
            chars: source.char_indices().peekable(),
            source,
            current_line: 1,
            current_column: 1,
        }
    }

    fn new_line(&mut self) -> (usize, usize) {
        let lc = (self.current_line, self.current_column);
        self.current_line += 1;
        self.current_column = 0;
        lc
    }

    fn advance_span(&mut self, start: usize, len: usize) -> Span {
        self.current_column += len;
        Span::new(start, len, self.current_line, self.current_column)
    }

    fn consume_comment(&mut self, start: usize) -> Token {
        for (offset, c) in self.chars.by_ref() {
            if c == '\n' {
                let (l, c) = self.new_line();
                // Check for new line
                return Token::new(TokenKind::Comment, Span::new(start, offset, l, c));
            }
        }
        let len = self.source.len() - start;
        Token::new(
            TokenKind::Comment,
            Span::new(start, len, self.current_line, self.current_column),
        )
    }

    fn consume_string(&mut self, start: usize) -> Result<Token, CompilationError> {
        for (offset, c) in self.chars.by_ref() {
            if c == '\'' {
                // Check for end of string
                let span = self.advance_span(start, offset + 1);
                return Ok(Token::new(TokenKind::String, span));
            }
        }
        Err(LexicalError::UnterminatedString {
            span: Span::new(
                start,
                self.source.len() - start,
                self.current_line,
                self.current_column,
            ),
        }
        .into())
    }

    fn consume_identifier(&mut self, start: usize) -> Token {
        // Find end of identifier
        let mut end = start + 1;
        while let Some((_, peek)) = self.chars.peek() {
            if !is_identifier_character(*peek) {
                // Check for end of identifier
                break;
            }
            self.chars.next();
            end += 1;
        }

        // Check if identifier is a keyword
        match &self.source[start..end] {
            "import" => Token::new(TokenKind::Import, self.advance_span(start, end - start)),
            "true" => Token::new(TokenKind::True, self.advance_span(start, end - start)),
            "false" => Token::new(TokenKind::False, self.advance_span(start, end - start)),
            "nil" => Token::new(TokenKind::Nil, self.advance_span(start, end - start)),
            "let" => Token::new(TokenKind::Let, self.advance_span(start, end - start)),
            "const" => Token::new(TokenKind::Const, self.advance_span(start, end - start)),
            "if" => Token::new(TokenKind::If, self.advance_span(start, end - start)),
            "then" => Token::new(TokenKind::Then, self.advance_span(start, end - start)),
            "else" => Token::new(TokenKind::Else, self.advance_span(start, end - start)),
            "elif" => Token::new(TokenKind::Elif, self.advance_span(start, end - start)),
            "end" => Token::new(TokenKind::End, self.advance_span(start, end - start)),
            "do" => Token::new(TokenKind::Do, self.advance_span(start, end - start)),
            "for" => Token::new(TokenKind::For, self.advance_span(start, end - start)),
            "in" => Token::new(TokenKind::In, self.advance_span(start, end - start)),
            "while" => Token::new(TokenKind::While, self.advance_span(start, end - start)),
            "function" => Token::new(TokenKind::Function, self.advance_span(start, end - start)),
            "break" => Token::new(TokenKind::Break, self.advance_span(start, end - start)),
            "continue" => Token::new(TokenKind::Continue, self.advance_span(start, end - start)),
            "and" => Token::new(TokenKind::And, self.advance_span(start, end - start)),
            "or" => Token::new(TokenKind::Or, self.advance_span(start, end - start)),
            "return" => Token::new(TokenKind::Return, self.advance_span(start, end - start)),
            "not" => Token::new(TokenKind::Not, self.advance_span(start, end - start)),
            "as" => Token::new(TokenKind::As, self.advance_span(start, end - start)),
            _ => Token::new(TokenKind::Identifier, self.advance_span(start, end - start)),
        }
    }

    fn consume_number(&mut self, start: usize) -> Result<Token, CompilationError> {
        let mut has_dot = false;
        let mut end = start + 1;
        while let Some((_, c)) = self.chars.peek().copied() {
            if c == '.' || c.is_ascii_digit() {
                // Check for float dot
                if c == '.' {
                    if has_dot {
                        return Err(LexicalError::MalformedNumber {
                            span: self.advance_span(start, end - start),
                        }
                        .into());
                    } else {
                        has_dot = true;
                    }
                }
                // Consume character
                end += 1;
                self.chars.next().unwrap();
            } else if is_identifier_character(c) {
                return Err(LexicalError::MalformedNumber {
                    span: self.advance_span(start, end - start),
                }
                .into());
            } else {
                break;
            }
        }
        let span = self.advance_span(start, end - start);
        if has_dot {
            Ok(Token::new(TokenKind::Float, span))
        } else {
            Ok(Token::new(TokenKind::Integer, span))
        }
    }

    fn consume_spaces(&mut self) {
        while let Some((_, c)) = self.chars.peek() {
            if !c.is_whitespace() {
                break;
            }
            self.current_column += 1;
            self.chars.next();
        }
    }

    pub(crate) fn next_token(&mut self) -> Result<Token, CompilationError> {
        while let Some((offset, c)) = self.chars.next() {
            match c {
                '+' => return Ok(Token::new(TokenKind::Plus, self.advance_span(offset, 1))),
                '-' => {
                    if let Some((offset, next)) = self.chars.peek().copied() {
                        if next == '-' {
                            // Comment detected
                            self.chars.next(); // Skip second '-' character
                            return Ok(self.consume_comment(offset - 1));
                        } else {
                            return Ok(Token::new(TokenKind::Minus, self.advance_span(offset, 1)));
                        }
                    } else {
                        return Ok(Token::new(TokenKind::Minus, self.advance_span(offset, 1)));
                    }
                }
                '*' => {
                    return Ok(Token::new(
                        TokenKind::Multiply,
                        self.advance_span(offset, 1),
                    ))
                }
                '/' => return Ok(Token::new(TokenKind::Divide, self.advance_span(offset, 1))),
                '(' => {
                    return Ok(Token::new(
                        TokenKind::LeftParen,
                        self.advance_span(offset, 1),
                    ))
                }
                ')' => {
                    return Ok(Token::new(
                        TokenKind::RightParen,
                        self.advance_span(offset, 1),
                    ))
                }
                '[' => {
                    return Ok(Token::new(
                        TokenKind::LeftBracket,
                        self.advance_span(offset, 1),
                    ))
                }
                ']' => {
                    return Ok(Token::new(
                        TokenKind::RightBracket,
                        self.advance_span(offset, 1),
                    ))
                }
                '{' => {
                    return Ok(Token::new(
                        TokenKind::LeftBrace,
                        self.advance_span(offset, 1),
                    ))
                }
                '}' => {
                    return Ok(Token::new(
                        TokenKind::RightBrace,
                        self.advance_span(offset, 1),
                    ))
                }
                ',' => return Ok(Token::new(TokenKind::Comma, self.advance_span(offset, 1))),
                ':' => return Ok(Token::new(TokenKind::Colon, self.advance_span(offset, 1))),
                '.' => return Ok(Token::new(TokenKind::Dot, self.advance_span(offset, 1))),
                '=' => {
                    if let Some((_, next)) = self.chars.peek().copied() {
                        if next == '=' {
                            // Double equal detected
                            self.chars.next(); // Skip second '=' character
                            return Ok(Token::new(TokenKind::Equal, self.advance_span(offset, 2)));
                        }
                    }
                    return Ok(Token::new(TokenKind::Assign, self.advance_span(offset, 1)));
                }
                '<' => {
                    if let Some((_, next)) = self.chars.peek().copied() {
                        if next == '=' {
                            self.chars.next();
                            return Ok(Token::new(
                                TokenKind::LessEqual,
                                self.advance_span(offset, 2),
                            ));
                        }
                    }
                    return Ok(Token::new(TokenKind::Less, self.advance_span(offset, 1)));
                }
                '>' => {
                    if let Some((_, next)) = self.chars.peek().copied() {
                        if next == '=' {
                            self.chars.next();
                            return Ok(Token::new(
                                TokenKind::GreaterEqual,
                                self.advance_span(offset, 2),
                            ));
                        }
                    }
                    return Ok(Token::new(TokenKind::Greater, self.advance_span(offset, 1)));
                }
                '!' => {
                    if let Some((_, next)) = self.chars.next() {
                        if next == '=' {
                            return Ok(Token::new(
                                TokenKind::NotEqual,
                                self.advance_span(offset, 2),
                            ));
                        }
                    }
                    return Err(LexicalError::IllegalCharacter {
                        span: self.advance_span(offset, 1),
                        c,
                    }
                    .into());
                }
                '\'' => return self.consume_string(offset),
                ' ' => {
                    self.consume_spaces(); // Ignore spaces
                                           // return Ok(Some(Token::Space));
                }
                '\n' => {
                    self.new_line();
                }
                _ => {
                    self.current_column += 1;
                }
            }

            if c.is_numeric() {
                // Try to parse a number
                return self.consume_number(offset);
            } else if is_identifier_character(c) {
                // Try to parse an identifier
                return Ok(self.consume_identifier(offset));
            }
        }
        Ok(Token::eof())
    }
}
