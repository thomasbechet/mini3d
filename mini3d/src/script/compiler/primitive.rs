use super::{
    error::CompileError,
    operator::{BinaryOperator, UnaryOperator},
};

#[derive(Debug, PartialEq, Clone, Copy)]
pub(crate) enum Primitive {
    Boolean,
    Integer,
    Float,
    String,
    Entity,
    Object,
}

impl Primitive {
    fn binary(
        left: Primitive,
        right: Primitive,
        op: BinaryOperator,
    ) -> Result<Primitive, CompileError> {
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
        Ok(Primitive::Boolean)
    }

    fn unary(primitive: Primitive, op: UnaryOperator) -> Result<Primitive, ()> {
        match op {
            UnaryOperator::Minus => match primitive {
                Primitive::Integer => Ok(Primitive::Integer),
                Primitive::Float => Ok(Primitive::Float),
                _ => Err(()),
            },
            UnaryOperator::Not => match primitive {
                Primitive::Boolean => Ok(Primitive::Boolean),
                _ => Err(()),
            },
        }
    }
}
