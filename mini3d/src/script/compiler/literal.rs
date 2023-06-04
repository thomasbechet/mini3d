use super::string::StringId;

#[derive(Debug, Clone, Copy)]
pub enum Literal {
    Nil,
    Boolean(bool),
    Integer(u32),
    Float(f32),
    String(StringId),
}
