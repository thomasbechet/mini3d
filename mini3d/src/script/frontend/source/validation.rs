use crate::script::{
    export::ExportTable,
    frontend::error::{CompileError, SemanticError},
    mir::primitive::PrimitiveType,
};

use super::{
    ast::{ASTNode, ASTNodeId, ASTVisitor, AST},
    literal::Literal,
    symbol::{Symbol, SymbolTable},
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

// fn evaluate_member_lookup(loopkup: ASTNodeId, ast: &AST)

fn evaluate_expression(
    expr: ASTNodeId,
    ast: &AST,
    symbols: &SymbolTable,
) -> Result<PrimitiveType, CompileError> {
    match ast.get(expr).unwrap() {
        ASTNode::Literal(lit) => match lit {
            Literal::Nil => Ok(PrimitiveType::Nil),
            Literal::Boolean(_) => Ok(PrimitiveType::Boolean),
            Literal::Integer(_) => Ok(PrimitiveType::Integer),
            Literal::Float(_) => Ok(PrimitiveType::Float),
            Literal::String(_) => Ok(PrimitiveType::String),
        },
        ASTNode::Identifier { symbol, span } => match symbols.get(*symbol).unwrap() {
            Symbol::Function { return_type, .. } => Ok(*return_type),
            Symbol::FunctionArgument { arg_type, .. } => Ok(*arg_type),
            Symbol::Constant { const_type, .. } => Ok(*const_type),
            Symbol::Variable { var_type } => {
                if let Some(var_type) = var_type {
                    Ok(*var_type)
                } else {
                    Err(SemanticError::UnresolvedSymbolType(*symbol).into())
                }
            }
            Symbol::Module { .. } => Err(SemanticError::TypeMistmatch { span: *span }.into()),
        },
        ASTNode::MemberLookup { .. } => {
            // TODO
            Ok(PrimitiveType::Integer)
        }
        ASTNode::Call => {
            // TODO
            Ok(PrimitiveType::Integer)
        }
        ASTNode::BinaryOperator(_) => {
            // TODO
            Ok(PrimitiveType::Integer)
        }
        ASTNode::UnaryOperator(_) => {
            // TODO
            Ok(PrimitiveType::Integer)
        }
        _ => unreachable!(),
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
