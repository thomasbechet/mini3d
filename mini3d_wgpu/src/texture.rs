use mini3d::feature::asset;
use wgpu::util::DeviceExt;

use crate::context::WGPUContext;

pub(crate) struct Texture {
    pub(crate) texture: wgpu::Texture,
    pub(crate) view: wgpu::TextureView,
}

impl Texture {
    pub(crate) fn from_asset(
        context: &WGPUContext,
        texture: &asset::texture::Texture,
        usage: wgpu::TextureUsages,
        label: Option<&str>,
    ) -> Self {
        let texture = context.device.create_texture_with_data(&context.queue, &wgpu::TextureDescriptor {
            label,
            size: wgpu::Extent3d {
                width: texture.width,
                height: texture.height,
                depth_or_array_layers: 1,
            },
            mip_level_count: 1,
            sample_count: 1,
            dimension: wgpu::TextureDimension::D2,
            format: wgpu::TextureFormat::Rgba8Unorm,
            usage,
        }, texture.data.as_slice());
        let view = texture.create_view(&wgpu::TextureViewDescriptor::default());
        Self { texture, view }
    }
}

impl Drop for Texture {
    fn drop(&mut self) {
        self.texture.destroy();
    }
}