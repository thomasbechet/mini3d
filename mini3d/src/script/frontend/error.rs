use std::num::{ParseIntError, ParseFloatError};

use mini3d_derive::Error;

use super::lexer::TokenKind;

#[derive(Debug, Error)]
pub enum LexerError {
    #[error("Unterminated string")]
    UnterminatedString,
    #[error("Malformed number")]
    MalformedNumber,
    #[error("Invalid character: {c}")]
    InvalidCharacter { c: char },
}

#[derive(Debug, Error)]
pub enum ParserError {
    #[error("Lexer error: {0}")]
    Lexer(LexerError),
    #[error("Unexpected token: expected {expected:?}, got {got:?}")]
    UnexpectedToken { expected: TokenKind, got: TokenKind },
    #[error("Invalid atom expression: got {got:?}")]
    InvalidAtomExpression { got: TokenKind },
    #[error("Integer parse error: {0}")]
    IntegerParseError(ParseIntError),
    #[error("Float parse error: {0}")]
    FloatParseError(ParseFloatError),
    #[error("Unexpected binary operator")]
    UnexpectedBinaryOperator,
    #[error("Unexpected import statement")]
    UnexpectedImportStatement,
    #[error("Identifier as statement")]
    IdentifierAsStatement,
}