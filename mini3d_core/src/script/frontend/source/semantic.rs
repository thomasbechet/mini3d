use crate::script::frontend::{error::SemanticError, source::ast::ASTVisitor};

use super::{
    ast::{ASTNode, ASTNodeId, AST},
    symbol::SymbolTable,
};

pub(crate) struct SemanticAnalysis;

// // TODO:
// // Type Checking
// // Score Checking
// //

impl SemanticAnalysis {
    pub(crate) fn check_undefined_symbols(symtab: &SymbolTable) -> Result<(), SemanticError> {
        for (id, entry) in symtab.symbols.iter().enumerate() {
            if entry.symbol.is_none() {
                return Err(SemanticError::UndefinedSymbol(id.into()));
            }
        }
        Ok(())
    }

    // pub(crate) fn export_symbols(symtab: &SymbolTable) -> ModuleExport {
    //     let symbols = Vec::new();
    //     ModuleExport { symbols }
    // }

    fn eval_expression_type(
        symtab: &SymbolTable,
        ast: &AST,
        node: ASTNodeId,
    ) -> Result<(), SemanticError> {
        let node = ast.get(node).unwrap();
        match node {
            ASTNode::BinaryOperator(op) => {}
            ASTNode::UnaryOperator(op) => {}
            ASTNode::Literal(lit) => {}
            ASTNode::Identifier { symbol, .. } => {}
            ASTNode::MemberLookup { .. } => {}
            ASTNode::Call => {}
            _ => unreachable!(),
        }
        Ok(())
    }

    pub(crate) fn infer_function_return_types(symtab: &mut SymbolTable, ast: &mut AST) {
        struct Visitor;
        impl ASTVisitor for Visitor {
            fn accept(&self, node: &ASTNode) -> bool {
                matches!(node, ASTNode::FunctionDeclaration { .. })
            }

            fn visit(&mut self, node: ASTNodeId, ast: &mut AST) -> bool {
                true
            }
        }
        ast.visit_df(&mut Visitor {});
    }
}
