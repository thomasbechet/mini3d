use core::num::{ParseFloatError, ParseIntError};

use crate::script::module::ModuleId;

use super::source::{
    symbol::SymbolId,
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
    UnexpectedToken {
        span: Span,
        got: TokenKind,
        expect: TokenKind,
    },
    UnexpectedExportToken {
        span: Span,
        got: TokenKind,
    },
    NonStatementToken {
        span: Span,
        got: TokenKind,
    },
    FunctionDeclarationOutsideOfGlobalScope {
        span: Span,
    },
    ExportOutsideOfGlobalScope {
        span: Span,
    },
    ImportOutsideOfGlobalScope {
        span: Span,
    },
    MissingConstantType {
        span: Span,
    },
    MissingArgumentType {
        span: Span,
    },
    InvalidAtomExpression {
        span: Span,
        got: TokenKind,
    },
    UnexpectedBinaryOperator {
        span: Span,
    },
    IdentifierAsStatement {
        span: Span,
    },
    DuplicatedArgument {
        span: Span,
    },
    SymbolAlreadyDefined {
        span: Span,
    },
    BreakOutsideLoop {
        span: Span,
    },
    ContinueOutsideLoop {
        span: Span,
    },
    ReturnOutsideFunction {
        span: Span,
    },
}

#[derive(Debug)]
pub enum SemanticError {
    TypeMistmatch { span: Span }, // Incompatible types in operands
    TypeViolation,                // Assigning value to a constant
    UndefinedSymbol(SymbolId),
    UnresolvedSymbolType(SymbolId),
    MultipleDefinitions,
    ModuleNotFound { span: Span },
    ModuleImportNotFound { module: ModuleId, span: Span },
    ImportSelf { span: Span },
    MissingImportSymbols { span: Span },
}

#[derive(Debug)]
pub enum CompileError {
    Lexical(LexicalError),
    Syntax(SyntaxError),
    Semantic(SemanticError),
    ScriptNotFound,
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
