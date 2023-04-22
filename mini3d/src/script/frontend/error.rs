use std::{error::Error, fmt::Display, num::{ParseIntError, ParseFloatError}};

use super::lexer::TokenKind;

#[derive(Debug)]
pub enum LexerError {
    UnterminatedString,
    MalformedNumber,
    InvalidCharacter { c: char },
}

impl Error for LexerError {}

impl Display for LexerError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::UnterminatedString => write!(f, "Unterminated string"),
            Self::MalformedNumber => write!(f, "Malformed number"),
            Self::InvalidCharacter { c } => write!(f, "Invalid character: {}", c),
        }
    }
}

#[derive(Debug)]
pub enum ParserError {
    Lexer(LexerError),
    UnexpectedToken { expected: TokenKind, got: TokenKind },
    InvalidAtomExpression { got: TokenKind },
    IntegerParseError(ParseIntError),
    FloatParseError(ParseFloatError),
    UnexpectedBinaryOperator,
    UnexpectedImportStatement,
    IdentifierAsStatement,
}

impl Error for ParserError {}

impl Display for ParserError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::Lexer(error) => write!(f, "Lexer error: {}", error),
            Self::UnexpectedToken { expected, got } => write!(f, "Unexpected token: expected {:?}, got {:?}", expected, got),
            Self::InvalidAtomExpression { got } => write!(f, "Invalid atom expression: got {:?}", got),
            Self::IntegerParseError(error) => write!(f, "Integer parse error: {}", error),
            Self::FloatParseError(error) => write!(f, "Float parse error: {}", error),
            Self::UnexpectedBinaryOperator => write!(f, "Unexpected binary operator"),
            Self::UnexpectedImportStatement => write!(f, "Unexpected import statement"),
            Self::IdentifierAsStatement => write!(f, "Identifier as statement"),
        }
    }
}