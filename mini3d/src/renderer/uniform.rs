use glam::{Vec2, Vec4};

pub(crate) enum UniformType {
    Buffer,
    Texture,
    TextureCube,
    Primitive,
}

enum Uniform {
    Buffer { id: u16, offset: u16 },
    Texture { id: u16 },
    TextureCube { id: u16 },
    Float(f32),
    Int(i32),
    Vec2(Vec2),
    Vec4(Vec4),
    // Mat4(Mat4),
}
