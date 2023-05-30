use std::fmt::Display;

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
    Const,
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

impl Display for TokenKind {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{:?}", self)
    }
}

impl TokenKind {
    pub(crate) fn is_binop(&self) -> bool {
        matches!(
            self,
            TokenKind::Plus
                | TokenKind::Minus
                | TokenKind::Multiply
                | TokenKind::Divide
                | TokenKind::Equal
                | TokenKind::NotEqual
                | TokenKind::LessEqual
                | TokenKind::GreaterEqual
                | TokenKind::Less
                | TokenKind::Greater
                | TokenKind::And
                | TokenKind::Or
        )
    }

    pub(crate) fn is_unaop(&self) -> bool {
        matches!(self, TokenKind::Minus | TokenKind::Not)
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct Span {
    pub(crate) offset: u32,
    pub(crate) len: u32,
    pub(crate) line: u32,
    pub(crate) column: u32,
}

impl Span {
    pub(crate) fn new(offset: usize, len: usize, line: usize, column: usize) -> Self {
        Self {
            offset: offset as u32,
            len: len as u32,
            line: line as u32,
            column: column as u32,
        }
    }

    pub(crate) fn slice<'a>(&self, source: &'a str) -> &'a str {
        &source[self.offset as usize..(self.offset + self.len) as usize]
    }

    pub(crate) fn string_content_slice<'a>(&self, source: &'a str) -> &'a str {
        &source[self.offset as usize + 1..(self.offset + self.len) as usize - 1]
    }

    pub(crate) fn comment_content_slice<'a>(&self, source: &'a str) -> &'a str {
        &source[self.offset as usize + 2..(self.offset + self.len) as usize]
    }

    pub(crate) fn end(&self) -> u32 {
        self.offset + self.len
    }

    pub(crate) fn join(&self, other: &Self) -> Self {
        let offset = self.offset.min(other.offset);
        let end = self.end().max(other.end());
        let (line, column) = if self.offset < other.offset {
            (self.line, self.column)
        } else {
            (other.line, other.column)
        };
        Self {
            offset,
            len: end - offset,
            line,
            column,
        }
    }
}

#[derive(Debug, Clone, Copy)]
pub(crate) struct Token {
    pub(crate) kind: TokenKind,
    pub(crate) span: Span,
}

impl Token {
    pub(crate) fn new(kind: TokenKind, span: Span) -> Self {
        Self { kind, span }
    }

    pub(crate) fn eof() -> Self {
        Self {
            kind: TokenKind::EOF,
            span: Span::new(0, 0, 0, 0),
        }
    }
}
