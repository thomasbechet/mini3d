use super::strings::StringId;

#[derive(Debug, Clone, Copy)]
pub(crate) enum Literal {
    Nil,
    Boolean(bool),
    Integer(u32),
    Real(f32),
    String(StringId),
}
