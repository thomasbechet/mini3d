use crate::script::{export::ExportTable, frontend::error::CompileError};

use super::{
    ast::{ASTNode, ASTNodeId, ASTVisitor, AST},
    symbol::SymbolTable,
};

struct ValidateExpression {}

impl ASTVisitor for ValidateExpression {
    fn accept(&self, node: &ASTNode) -> bool {
        false
    }

    fn visit(&mut self, node: ASTNodeId, ast: &mut AST) -> bool {
        false
    }
}

struct ValidateStatement {}

impl ASTVisitor for ValidateStatement {
    fn accept(&self, node: &ASTNode) -> bool {
        node.is_statement()
    }

    fn visit(&mut self, node: ASTNodeId, ast: &mut AST) -> bool {
        match ast.get(node).unwrap() {
            ASTNode::Assignment { .. } => {
                // Check typing if explicit or resolve expression
                // Check symbol type
            }
            ASTNode::FunctionDeclaration { .. } => {
                // Check return statement
            }
            ASTNode::VariableDeclaration { .. } => {
                // Check expression
            }
            ASTNode::If => {
                // Check condition expression
                // Check statements
            }
            ASTNode::For { .. } => {
                // TODO
            }
            ASTNode::While { .. } => {
                // Check condition expression
                // Check statements
            }
            ASTNode::Loop { .. } => {
                // Check statements
            }
            ASTNode::Return => {
                // Check expression
            }
            ASTNode::Call => {
                // Check argument count
                // Check argument expressions
                // Check expression
            }
            _ => return true, // visit children
        }
        false // don't visit children
    }
}

pub(crate) fn validate_ast(
    ast: &mut AST,
    symbols: &mut SymbolTable,
    exports: &ExportTable,
) -> Result<(), CompileError> {
    // Resolve symbols
    // Check typing
    Ok(())
}
