use crate::script::{export::ExportTable, frontend::error::CompileError};

use super::{
    ast::{ASTNode, ASTNodeId, ASTVisitor, AST},
    symbol::SymbolTable,
};

struct ValidateStatement {}

impl ASTVisitor for ValidateStatement {
    fn accept(&self, node: &ASTNode) -> bool {
        node.is_statement()
    }

    fn visit(&mut self, node: ASTNodeId, ast: &mut AST) -> bool {
        match ast.get(node).unwrap() {
            ASTNode::Assignment { .. } => {}
            ASTNode::FunctionDeclaration {
                span,
                symbol,
                function_block,
            } => {}
            ASTNode::If => {}
            ASTNode::For { .. } => {}
            ASTNode::While { .. } => {}
            ASTNode::Loop { .. } => {}
            ASTNode::Break => {}
            ASTNode::Continue => {}
            ASTNode::Return => {}
            ASTNode::Call => {}
            ASTNode::VariableDeclaration { span, symbol } => {}
            _ => {}
        }
        false
    }
}

pub(crate) fn validate_ast(
    ast: &mut AST,
    symbols: &mut SymbolTable,
    exports: &ExportTable,
) -> Result<(), CompileError> {
    // ast.vi
    // Resolve symbols
    // Check typing
    Ok(())
}
