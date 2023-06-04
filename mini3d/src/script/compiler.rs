use self::{
    ast::AST, error::CompileError, lexical::Lexer, semantic::SemanticAnalysis, string::StringTable,
    symbol::SymbolTable, syntax::SyntaxAnalysis,
};

use super::interpreter::program::Program;

pub mod ast;
pub mod error;
pub mod lexical;
pub mod literal;
pub mod operator;
pub mod primitive;
pub mod semantic;
pub mod stream;
pub mod string;
pub mod symbol;
pub mod syntax;
pub mod token;

pub struct Compiler {
    lexer: Lexer,
    ast: AST,
    symtab: SymbolTable,
    strings: StringTable,
}

impl Compiler {
    pub fn new(parse_comments: bool) -> Self {
        Self {
            lexer: Lexer::new(parse_comments),
            ast: AST::new(),
            symtab: Default::default(),
            strings: Default::default(),
        }
    }

    pub fn compile(&mut self, source: &str) -> Result<Program, CompileError> {
        // Prepare resources
        self.lexer.clear();
        self.ast.clear();
        self.symtab.clear();
        self.strings.clear();
        // Lexical analysis
        // Syntax analysis
        SyntaxAnalysis::evaluate(
            &mut self.ast,
            &mut self.symtab,
            &mut self.strings,
            &mut self.lexer,
            source,
        )?;
        self.ast.print();
        self.symtab.print(&self.strings);
        self.strings.print();
        // Semantic analysis
        SemanticAnalysis::check_undefined_symbols(&self.symtab)?;
        // Code generation
        // Optimization
        Ok(Program::empty())
    }
}
