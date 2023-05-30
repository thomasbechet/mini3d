use self::{
    error::{CompilationError, SyntaxError},
    lexical::Lexer,
    semantic::SemanticAnalysis,
    syntax::SyntaxAnalysis,
};

use super::interpreter::program::Program;

pub mod ast;
pub mod error;
pub mod lexical;
pub mod semantic;
pub mod symbol;
pub mod syntax;
pub mod token;

pub struct Compiler {}

impl Compiler {
    pub fn compile(source: &str, parse_comments: bool) -> Result<Program, CompilationError> {
        // Lexical analysis
        let lexer = Lexer::new(source);
        // Syntax analysis
        let (ast, symtab) = SyntaxAnalysis::evaluate(lexer, parse_comments)?;
        ast.print();
        symtab.print(source);
        // Semantic analysis
        SemanticAnalysis::check_undefined_symbols(&symtab)?;
        // Code generation
        // Optimization
        Ok(Program::empty())
    }
}
