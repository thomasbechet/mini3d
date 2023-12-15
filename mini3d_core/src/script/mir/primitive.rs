use mini3d_derive::Serialize;

use crate::script::frontend::{
    error::CompileError,
    source::operator::{BinaryOperator, UnaryOperator},
};

pub(crate) enum ReferenceType {
    Function,
    Component,
    Resource,
    Query,
}

#[derive(Debug, PartialEq, Clone, Copy, Serialize)]
pub(crate) enum PrimitiveType {
    Nil,
    Bool,
    I32,
    I32F16,
    V2I32F16,
    V2I32,
    V3I32F16,
    V3I32,
    V4I32F16,
    V4I32,
    M4I32F16,
    QI32F16,
    String,
    Entity,
    Object,
    UID,
    ResourceRef,
}

impl PrimitiveType {
    fn binary(
        left: PrimitiveType,
        right: PrimitiveType,
        op: BinaryOperator,
    ) -> Result<PrimitiveType, CompileError> {
        match op {
            BinaryOperator::Addition => todo!(),
            BinaryOperator::Subtraction => todo!(),
            BinaryOperator::Multiplication => todo!(),
            BinaryOperator::Division => todo!(),
            BinaryOperator::LessEqual
            | BinaryOperator::GreaterEqual
            | BinaryOperator::Less
            | BinaryOperator::Greater
            | BinaryOperator::NotEqual
            | BinaryOperator::Equal => todo!(),
            BinaryOperator::And | BinaryOperator::Or => {}
        }
        Ok(PrimitiveType::Bool)
    }

    fn unary(primitive: PrimitiveType, op: UnaryOperator) -> Result<PrimitiveType, ()> {
        match op {
            UnaryOperator::Minus => match primitive {
                PrimitiveType::I32 => Ok(PrimitiveType::I32),
                PrimitiveType::I32F16 => Ok(PrimitiveType::I32F16),
                _ => Err(()),
            },
            UnaryOperator::Not => match primitive {
                PrimitiveType::Bool => Ok(PrimitiveType::Bool),
                _ => Err(()),
            },
        }
    }
}
