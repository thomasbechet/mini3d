use alloc::vec::Vec;
use mini3d_db::slot_map_key_handle;
use mini3d_derive::Serialize;

slot_map_key_handle!(TextureHandle);

#[derive(Clone, Serialize, Default)]
pub enum TextureFormat {
    #[default]
    Color,
    ColorAlpha,
    Depth,
    DepthStencil,
    CubeMap,
}

#[derive(Clone, Copy, Serialize)]
pub enum TextureWrapMode {
    Clamp,
    Repeat,
    Mirror,
}

#[derive(Default, Clone, Copy, Serialize)]
pub enum TextureUsage {
    #[default]
    Sample,
    RenderTarget,
    Present,
}

#[derive(Clone, Serialize, Default)]
pub struct Texture {
    pub(crate) format: TextureFormat,
    pub(crate) usage: TextureUsage,
    pub(crate) bytes: Vec<u8>,
    pub(crate) width: u16,
    pub(crate) height: u16,
}

impl Texture {
    pub fn new(format: TextureFormat, bytes: Vec<u8>, width: u16, height: u16) -> Self {
        Self {
            format,
            bytes,
            width,
            height,
            ..Default::default()
        }
    }
}
