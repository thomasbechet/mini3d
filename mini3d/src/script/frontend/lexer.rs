use std::{iter::Peekable, str::CharIndices};

use super::error::LexerError;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum TokenKind {
    Import,
    As,
    Comment,
    Space,
    LeftParen,
    RightParen,
    LeftBracket,
    RightBracket,
    LeftBrace,
    RightBrace,
    Comma,
    Colon,
    Dot,
    Identifier,
    Integer,
    Float,
    String,
    Plus,
    Minus,
    Multiply,
    Divide,
    Assign,
    Equal,
    NotEqual,
    LessEqual,
    GreaterEqual,
    Less,
    Greater,
    And,
    Or,
    Not,
    Let,
    If,
    Then,
    Else,
    Elif,
    End,
    Do,
    For,
    In,
    While,
    Function,
    True,
    False,
    Nil,
    Break,
    Continue,
    Return,
    EOF,
}

impl TokenKind {

    pub(crate) fn is_binop(&self) -> bool {
        matches!(self, TokenKind::Plus | TokenKind::Minus | TokenKind::Multiply | TokenKind::Divide | TokenKind::Equal | TokenKind::NotEqual | TokenKind::LessEqual | TokenKind::GreaterEqual | TokenKind::Less | TokenKind::Greater | TokenKind::And | TokenKind::Or)
    }

    pub(crate) fn is_unaop(&self) -> bool {
        matches!(self, TokenKind::Minus | TokenKind::Not)
    }
}

#[derive(Debug, Clone, Copy)]
pub struct TokenSpan {
    start: usize,
    end: usize,
}

impl TokenSpan {

    pub fn new(start: usize, end: usize) -> Self {
        Self { 
            start, 
            end, 
        }
    }

    pub fn slice<'a>(&self, source: &'a str) -> &'a str {
        &source[self.start..self.end]
    }

    pub fn string_content_slice<'a>(&self, source: &'a str) -> &'a str {
        &source[self.start + 1..self.end - 1]
    }

    pub fn comment_content_slice<'a>(&self, source: &'a str) -> &'a str {
        &source[self.start + 2..self.end]
    }
}

#[derive(Debug, Clone, Copy)]
pub struct Token {
    pub kind: TokenKind,
    pub span: TokenSpan,
}

impl Token {

    pub fn new(kind: TokenKind, span: TokenSpan) -> Self {
        Self {
            kind,
            span,
        }
    }

    pub fn eof() -> Self {
        Self {
            kind: TokenKind::EOF,
            span: TokenSpan::new(0, 0),
        }
    }
}

pub struct Lexer<'a> {
    chars: Peekable<CharIndices<'a>>,
    source: &'a str,
}

fn is_identifier_character(c: char) -> bool {
    c.is_alphanumeric() || c == '_'
}

impl<'a> Lexer<'a> {

    pub fn new(source: &'a str) -> Self {
        Self {
            chars: source.char_indices().peekable(),
            source,
        }
    }

    fn consume_comment(&mut self, start: usize) -> Token {
        for (offset, c) in self.chars.by_ref() {
            if c == '\n' { // Check for new line
                return Token::new(TokenKind::Comment, TokenSpan::new(start, offset));
            }
        }
        Token::new(TokenKind::Comment, TokenSpan::new(start, self.source.len() - 1))
    }

    fn consume_string(&mut self, start: usize) -> Result<Token, LexerError> {
        for (offset, c) in self.chars.by_ref() {
            if c == '\'' { // Check for end of string
                return Ok(Token::new(TokenKind::String, TokenSpan::new(start, offset + 1)));
            }
        }
        Err(LexerError::UnterminatedString)
    }

    fn consume_identifier(&mut self, start: usize) -> Token {
        
        // Find end of identifier
        let mut end = start + 1;
        while let Some((_, peek)) = self.chars.peek() {
            if !is_identifier_character(*peek) { // Check for end of identifier
                break;
            }
            self.chars.next();
            end += 1;
        }

        // Check if identifier is a keyword
        match &self.source[start..end] {
            "import" => Token::new(TokenKind::Import, TokenSpan::new(start, end)),
            "true" => Token::new(TokenKind::True, TokenSpan::new(start, end)),
            "false" => Token::new(TokenKind::False, TokenSpan::new(start, end)),
            "nil" => Token::new(TokenKind::Nil, TokenSpan::new(start, end)),
            "let" => Token::new(TokenKind::Let, TokenSpan::new(start, end)),
            "if" => Token::new(TokenKind::If, TokenSpan::new(start, end)),
            "then" => Token::new(TokenKind::Then, TokenSpan::new(start, end)),
            "else" => Token::new(TokenKind::Else, TokenSpan::new(start, end)),
            "elif" => Token::new(TokenKind::Elif, TokenSpan::new(start, end)),
            "end" => Token::new(TokenKind::End, TokenSpan::new(start, end)),
            "do" => Token::new(TokenKind::Do, TokenSpan::new(start, end)),
            "for" => Token::new(TokenKind::For, TokenSpan::new(start, end)),
            "in" => Token::new(TokenKind::In, TokenSpan::new(start, end)),
            "while" => Token::new(TokenKind::While, TokenSpan::new(start, end)),
            "function" => Token::new(TokenKind::Function, TokenSpan::new(start, end)),
            "break" => Token::new(TokenKind::Break, TokenSpan::new(start, end)),
            "continue" => Token::new(TokenKind::Continue, TokenSpan::new(start, end)),
            "and" => Token::new(TokenKind::And, TokenSpan::new(start, end)),
            "or" => Token::new(TokenKind::Or, TokenSpan::new(start, end)),
            "return" => Token::new(TokenKind::Return, TokenSpan::new(start, end)),
            "not" => Token::new(TokenKind::Not, TokenSpan::new(start, end)),
            "as" => Token::new(TokenKind::As, TokenSpan::new(start, end)),
            _ => Token::new(TokenKind::Identifier, TokenSpan::new(start, end)),
        }
    }

    fn consume_number(&mut self, start: usize) -> Result<Token, LexerError> {
        let mut has_dot = false;
        let mut end = start + 1;
        while let Some((_, c)) = self.chars.peek().copied() {
            if c == '.' || c.is_ascii_digit() {
                // Check for float dot
                if c == '.' {
                    if has_dot {
                        return Err(LexerError::MalformedNumber);
                    } else {
                        has_dot = true;
                    }
                }
                // Consume character
                end += 1;
                self.chars.next().unwrap();
            } else if is_identifier_character(c) {
                return Err(LexerError::MalformedNumber);
            } else {
                break;
            }
        }
        if has_dot {
            Ok(Token::new(TokenKind::Float, TokenSpan::new(start, end)))
        } else {
            Ok(Token::new(TokenKind::Integer, TokenSpan::new(start, end)))
        }
    }

    fn consume_spaces(&mut self) {
        while let Some((_, c)) = self.chars.peek() {
            if !c.is_whitespace() {
                break;
            }
            self.chars.next();
        }
    }

    pub fn next_token(&mut self) -> Result<Token, LexerError> {
        while let Some((offset, c)) = self.chars.next() {
            match c {
                '+' => return Ok(Token::new(TokenKind::Plus, TokenSpan::new(offset, offset + 1))),
                '-' => {
                    if let Some((offset, next)) = self.chars.peek().copied() {
                        if next == '-' { // Comment detected
                            self.chars.next(); // Skip second '-' character
                            return Ok(self.consume_comment(offset - 1));
                        } else {
                            return Ok(Token::new(TokenKind::Minus, TokenSpan::new(offset, offset + 1)));
                        }
                    } else {
                        return Ok(Token::new(TokenKind::Minus, TokenSpan::new(offset, offset + 1)));
                    }
                },
                '*' => return Ok(Token::new(TokenKind::Multiply, TokenSpan::new(offset, offset + 1))),
                '/' => return Ok(Token::new(TokenKind::Divide, TokenSpan::new(offset, offset + 1))),
                '(' => return Ok(Token::new(TokenKind::LeftParen, TokenSpan::new(offset, offset + 1))),
                ')' => return Ok(Token::new(TokenKind::RightParen, TokenSpan::new(offset, offset + 1))),
                '[' => return Ok(Token::new(TokenKind::LeftBracket, TokenSpan::new(offset, offset + 1))),
                ']' => return Ok(Token::new(TokenKind::RightBracket, TokenSpan::new(offset, offset + 1))),
                '{' => return Ok(Token::new(TokenKind::LeftBrace, TokenSpan::new(offset, offset + 1))),
                '}' => return Ok(Token::new(TokenKind::RightBrace, TokenSpan::new(offset, offset + 1))),
                ',' => return Ok(Token::new(TokenKind::Comma, TokenSpan::new(offset, offset + 1))),
                ':' => return Ok(Token::new(TokenKind::Colon, TokenSpan::new(offset, offset + 1))),
                '.' => return Ok(Token::new(TokenKind::Dot, TokenSpan::new(offset, offset + 1))),
                '=' => {
                    if let Some((_, next)) = self.chars.peek().copied() {
                        if next == '=' { // Double equal detected
                            self.chars.next(); // Skip second '=' character
                            return Ok(Token::new(TokenKind::Equal, TokenSpan::new(offset, offset + 2)));
                        }
                    }
                    return Ok(Token::new(TokenKind::Assign, TokenSpan::new(offset, offset + 1)));
                },
                '<' => {
                    if let Some((_, next)) = self.chars.peek().copied() {
                        if next == '=' {
                            self.chars.next();
                            return Ok(Token::new(TokenKind::LessEqual, TokenSpan::new(offset, offset + 2)));
                        }
                    }
                    return Ok(Token::new(TokenKind::Less, TokenSpan::new(offset, offset + 1)));
                },
                '>' => {
                    if let Some((_, next)) = self.chars.peek().copied() {
                        if next == '=' {
                            self.chars.next();
                            return Ok(Token::new(TokenKind::GreaterEqual, TokenSpan::new(offset, offset + 2)));
                        }
                    }
                    return Ok(Token::new(TokenKind::Greater, TokenSpan::new(offset, offset + 1)));
                },
                '!' => {
                    if let Some((_, next)) = self.chars.next() {
                        if next == '=' {
                            return Ok(Token::new(TokenKind::NotEqual, TokenSpan::new(offset, offset + 2)));
                        }
                    }
                    return Err(LexerError::InvalidCharacter { c });
                },
                '\'' => return self.consume_string(offset),
                ' ' => {
                    self.consume_spaces(); // Ignore spaces
                    // return Ok(Some(Token::Space));
                },
                _ => {}
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