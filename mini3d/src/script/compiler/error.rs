use std::num::{ParseFloatError, ParseIntError};

use super::{
    symbol::SymbolId,
    token::{Span, TokenKind},
};

pub enum LexicalError {
    UnterminatedString { span: Span },
    MalformedNumber { span: Span },
    IllegalCharacter { span: Span, c: char },
}

pub enum SyntaxError {
    UnexpectedToken {
        span: Span,
        expected: TokenKind,
        got: TokenKind,
    },
    InvalidAtomExpression {
        span: Span,
        got: TokenKind,
    },
    IntegerParseError {
        span: Span,
        error: ParseIntError,
    },
    FloatParseError {
        span: Span,
        error: ParseFloatError,
    },
    UnexpectedBinaryOperator {
        span: Span,
    },
    UnexpectedImportStatement {
        span: Span,
    },
    IdentifierAsStatement {
        span: Span,
    },
    DuplicatedArgument {
        span: Span,
    },
}

pub enum SemanticError {
    TypeMistmatch,
    UndefinedSymbol(SymbolId),
    MultipleDefinitions,
}

pub enum CompilationError {
    Lexical(LexicalError),
    Syntax(SyntaxError),
    Semantic(SemanticError),
}

impl From<LexicalError> for CompilationError {
    fn from(e: LexicalError) -> Self {
        CompilationError::Lexical(e)
    }
}

impl From<SyntaxError> for CompilationError {
    fn from(e: SyntaxError) -> Self {
        CompilationError::Syntax(e)
    }
}

impl From<SemanticError> for CompilationError {
    fn from(e: SemanticError) -> Self {
        CompilationError::Semantic(e)
    }
}

impl CompilationError {
    // pub(crate) fn message(&self, source: &str) -> String {
    //     match self {
    //         CompilationError::Lexical(e) => {}
    //         CompilationError::Syntax(e) => {}
    //         CompilationError::Semantic(e) => {}
    //     }
    // }
}
