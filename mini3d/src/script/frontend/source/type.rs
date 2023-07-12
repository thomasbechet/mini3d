#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash)]
pub(crate) struct TypeId(u32);

impl TypeId {
    fn index(self) -> usize {
        self.0 as usize
    }
}

pub(crate) enum TypeKind {
    Primitive(PrimitiveType),
    ComponentRef(StringId),
    Nil,
}

pub(crate) struct Type {
    nullable: bool,
    kind: TypeKind,
}

pub(crate) struct TypeTable {}
