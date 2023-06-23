use std::num::{ParseFloatError, ParseIntError};

use super::source::{
    symbol::SourceSymbolId,
    token::{Span, TokenKind},
};

#[derive(Debug)]
pub enum LexicalError {
    UnterminatedString { span: Span },
    MalformedNumber { span: Span },
    IllegalCharacter { span: Span, c: char },
    IntegerParseError { span: Span, error: ParseIntError },
    FloatParseError { span: Span, error: ParseFloatError },
}

#[derive(Debug)]
pub enum SyntaxError {
    UnexpectedToken { span: Span, got: TokenKind },
    NonStatementToken { span: Span, got: TokenKind },
    FunctionDeclarationOutsideOfGlobalScope { span: Span },
    ExportedConstantOutsideOfGlobalScope { span: Span },
    MissingConstantType { span: Span },
    InvalidAtomExpression { span: Span, got: TokenKind },
    UnexpectedBinaryOperator { span: Span },
    IdentifierAsStatement { span: Span },
    DuplicatedArgument { span: Span },
    SymbolAlreadyDefined { span: Span },
    ModuleNotFound { span: Span },
}

#[derive(Debug)]
pub enum SemanticError {
    TypeMistmatch,
    UndefinedSymbol(SourceSymbolId),
    MultipleDefinitions,
}

#[derive(Debug)]
pub enum CompileError {
    Lexical(LexicalError),
    Syntax(SyntaxError),
    Semantic(SemanticError),
    ScriptNotFound,
    ModuleNotFound,
}

impl From<LexicalError> for CompileError {
    fn from(e: LexicalError) -> Self {
        CompileError::Lexical(e)
    }
}

impl From<SyntaxError> for CompileError {
    fn from(e: SyntaxError) -> Self {
        CompileError::Syntax(e)
    }
}

impl From<SemanticError> for CompileError {
    fn from(e: SemanticError) -> Self {
        CompileError::Semantic(e)
    }
}

impl CompileError {
    // pub(crate) fn message(&self, source: &str) -> String {
    //     match self {
    //         CompilationError::Lexical(e) => {}
    //         CompilationError::Syntax(e) => {}
    //         CompilationError::Semantic(e) => {}
    //     }
    // }
}
