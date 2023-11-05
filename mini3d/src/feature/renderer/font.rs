use crate::{
    define_resource_handle,
    feature::core::resource::{Resource, ResourceHookContext},
    math::rect::IRect,
    renderer::provider::RendererProviderHandle,
    resource::handle::ResourceHandle,
};
use glam::{IVec2, UVec2};
use mini3d_derive::{Reflect, Serialize};
use std::collections::HashMap;

use super::texture::{Texture, TextureFormat};

define_resource_handle!(FontHandle);

#[derive(Clone, Reflect, Serialize)]
pub struct Font {
    pub glyph_size: UVec2,
    pub data: Vec<u8>,
    pub glyph_locations: HashMap<char, usize>,
    #[serialize(skip)]
    pub(crate) handle: RendererProviderHandle,
}

impl Default for Font {
    fn default() -> Self {
        let glyph_width = 8;
        let glyph_height = 8;
        let data = include_bytes!("../../../../assets/font.bin").to_vec();
        let glyph_locations: HashMap<_, _> = " !\"#$%&\'()*+,-./0123456789:;<=>?@ABCDEFGHIJKLMNOPQRSTUVWXYZ[]^_`abcdefghijklmnopqrstuvwxyz{|}~éèê"
            .chars().enumerate().map(|(i, x)| (x, i)).collect();
        Font {
            glyph_size: UVec2::new(glyph_width as u32, glyph_height as u32),
            data,
            glyph_locations,
            handle: RendererProviderHandle::null(),
        }
    }
}

impl Font {
    pub const NAME: &'static str = "RTY_Font";
}

impl Resource for Font {
    fn hook_added(handle: ResourceHandle, ctx: ResourceHookContext) {
        let font = ctx.resource.get_mut::<Font>(handle).unwrap();
        ctx.renderer.on_font_added_hook(font, handle.into());
    }

    fn hook_removed(handle: ResourceHandle, ctx: ResourceHookContext) {
        let font = ctx.resource.get_mut::<Font>(handle).unwrap();
        ctx.renderer.on_font_removed_hook(font, handle.into());
    }
}

#[derive(Default)]
pub struct FontAtlas {
    pub texture: Texture,
    pub extents: HashMap<char, IRect>,
}

impl FontAtlas {
    pub fn new(font: &Font) -> FontAtlas {
        let glyph_count = font.glyph_locations.len();
        let width = font.glyph_size.x * glyph_count as u32;
        let height = font.glyph_size.y;
        let mut texture = Texture {
            data: vec![0x0; (width * height * 4) as usize],
            format: TextureFormat::RGBA,
            width,
            height,
            handle: Default::default(),
        };

        let mut extents: HashMap<char, IRect> = Default::default();
        let mut extent = IRect::new(0, 0, font.glyph_size.x, height);
        for (c, location) in &font.glyph_locations {
            // Write pixels to texture
            // TODO: optimize me
            for p in 0..(font.glyph_size.x as usize * font.glyph_size.y as usize) {
                let bit_offset =
                    (*location * (font.glyph_size.x as usize * font.glyph_size.y as usize)) + p;
                let byte = font.data[bit_offset / 8];
                let bit_set = byte & (1 << (7 - (p % 8))) != 0;

                let px = (extent.left() + (p as i32 % font.glyph_size.x as i32)) as usize;
                let py = (extent.top() + (p as i32 / font.glyph_size.x as i32)) as usize;
                let pi = py * texture.width as usize + px;
                let color = if bit_set { 0xFF } else { 0x0 };
                texture.data[pi * 4] = color;
                texture.data[pi * 4 + 1] = color;
                texture.data[pi * 4 + 2] = color;
                texture.data[pi * 4 + 3] = color;
            }

            // Save extent and move to next glyph
            extents.insert(*c, extent);
            extent = extent.translate(IVec2::new(font.glyph_size.x as i32, 0));
        }

        Self { texture, extents }
    }
}
