use crate::script::frontend::{
    error::CompileError,
    source::operator::{BinaryOperator, UnaryOperator},
};

#[derive(Debug, PartialEq, Clone, Copy)]
pub(crate) enum PrimitiveType {
    Boolean,
    Integer,
    Float,
    Vec2,
    IVec2,
    Vec3,
    IVec3,
    Vec4,
    IVec4,
    Mat4,
    Quat,
    String,
    Entity,
    Object,
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
        Ok(PrimitiveType::Boolean)
    }

    fn unary(primitive: PrimitiveType, op: UnaryOperator) -> Result<PrimitiveType, ()> {
        match op {
            UnaryOperator::Minus => match primitive {
                PrimitiveType::Integer => Ok(PrimitiveType::Integer),
                PrimitiveType::Float => Ok(PrimitiveType::Float),
                _ => Err(()),
            },
            UnaryOperator::Not => match primitive {
                PrimitiveType::Boolean => Ok(PrimitiveType::Boolean),
                _ => Err(()),
            },
        }
    }
}
