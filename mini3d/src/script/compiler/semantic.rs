use super::{
    ast::{ASTNode, AST},
    error::SemanticError,
    symbol::{SymbolId, SymbolKind, SymbolTable},
};

pub(crate) struct SemanticAnalysis;

// // TODO:
// // Type Checking
// // Score Checking
// //

impl SemanticAnalysis {
    pub(crate) fn check_undefined_symbols(table: &SymbolTable) -> Result<(), SemanticError> {
        for (id, symbol) in table.symbols.iter().enumerate() {
            if symbol.symbol.kind == SymbolKind::Undefined {
                return Err(SemanticError::UndefinedSymbol(id as SymbolId));
            }
        }
        Ok(())
    }
}
