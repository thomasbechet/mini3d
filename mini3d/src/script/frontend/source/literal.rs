use crate::script::constant::ConstantId;

#[derive(Debug, Clone, Copy)]
pub enum Literal {
    Nil,
    Boolean(bool),
    Integer(u32),
    Float(f32),
    String(ConstantId),
}
