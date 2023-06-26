use super::strings::StringId;

#[derive(Debug, Clone, Copy)]
pub(crate) enum Literal {
    Nil,
    Boolean(bool),
    Integer(u32),
    Float(f32),
    String(StringId),
}
