use std::fmt::Display;

use crate::{
    script::{frontend::mir::primitive::PrimitiveType, string::StringId},
    uid::UID,
};

use super::literal::Literal;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum TokenKind {
    Import,
    Export,
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
    Primitive,
    Literal,
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

#[derive(Debug, Clone, Copy)]
pub struct Location {
    pub(crate) line: u32,
    pub(crate) column: u32,
}

impl Location {
    pub(crate) fn new(line: u32, column: u32) -> Self {
        Self { line, column }
    }

    fn min(&self, other: &Self) -> Self {
        if self.line < other.line {
            *self
        } else if self.line > other.line {
            *other
        } else if self.column < other.column {
            *self
        } else {
            *other
        }
    }

    fn max(&self, other: &Self) -> Self {
        if self.line > other.line {
            *self
        } else if self.line < other.line {
            *other
        } else if self.column > other.column {
            *self
        } else {
            *other
        }
    }
}

#[derive(Debug, Clone, Copy)]
pub struct Span {
    pub(crate) start: Location,
    pub(crate) stop: Location,
}

impl Span {
    pub(crate) fn join(&self, other: &Self) -> Self {
        Self {
            start: self.start.min(&other.start),
            stop: self.stop.max(&other.stop),
        }
    }
}

#[derive(Debug, Clone, Copy)]
pub(crate) enum TokenValue {
    Literal(Literal),
    Identifier(StringId),
    Primitive(PrimitiveType),
    Comment(StringId),
    None,
}

impl From<TokenValue> for bool {
    fn from(value: TokenValue) -> Self {
        match value {
            TokenValue::Literal(Literal::Boolean(value)) => value,
            _ => panic!("TokenValue is not a boolean"),
        }
    }
}

impl From<TokenValue> for u32 {
    fn from(value: TokenValue) -> Self {
        match value {
            TokenValue::Literal(Literal::Integer(value)) => value,
            _ => panic!("TokenValue is not an integer"),
        }
    }
}

impl From<TokenValue> for f32 {
    fn from(value: TokenValue) -> Self {
        match value {
            TokenValue::Literal(Literal::Float(value)) => value,
            _ => panic!("TokenValue is not a float"),
        }
    }
}

impl From<TokenValue> for StringId {
    fn from(value: TokenValue) -> Self {
        match value {
            TokenValue::Literal(Literal::String(value)) => value,
            TokenValue::Identifier(value) => value,
            TokenValue::Comment(value) => value,
            _ => panic!("TokenValue is not a string, identifier or comment"),
        }
    }
}

impl From<TokenValue> for Literal {
    fn from(value: TokenValue) -> Self {
        match value {
            TokenValue::Literal(value) => value,
            _ => panic!("TokenValue is not a literal type"),
        }
    }
}

impl From<TokenValue> for PrimitiveType {
    fn from(value: TokenValue) -> Self {
        match value {
            TokenValue::Primitive(value) => value,
            _ => panic!("TokenValue is not a primitive type"),
        }
    }
}

#[derive(Debug, Clone, Copy)]
pub(crate) struct Token {
    pub(crate) kind: TokenKind,
    pub(crate) span: Span,
    pub(crate) value: TokenValue,
}

impl Token {
    pub(crate) fn eof() -> Self {
        Self {
            kind: TokenKind::EOF,
            span: Span {
                start: Location::new(0, 0),
                stop: Location::new(0, 0),
            },
            value: TokenValue::None,
        }
    }

    pub(crate) fn single(kind: TokenKind, location: Location) -> Self {
        Self {
            kind,
            span: Span {
                start: location,
                stop: location,
            },
            value: TokenValue::None,
        }
    }

    pub(crate) fn double(kind: TokenKind, location: Location) -> Self {
        Self {
            kind,
            span: Span {
                start: location,
                stop: Location {
                    line: location.line,
                    column: location.column + 1,
                },
            },
            value: TokenValue::None,
        }
    }
}
