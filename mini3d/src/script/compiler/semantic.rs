use super::{
    error::SemanticError,
    symbol::{SymbolId, SymbolTable},
};

pub(crate) struct SemanticAnalysis;

// // TODO:
// // Type Checking
// // Score Checking
// //

impl SemanticAnalysis {
    pub(crate) fn check_undefined_symbols(table: &SymbolTable) -> Result<(), SemanticError> {
        for (id, symbol) in table.symbols.iter().enumerate() {
            if !symbol.symbol.is_defined() {
                return Err(SemanticError::UndefinedSymbol(id as SymbolId));
            }
        }
        Ok(())
    }
}
