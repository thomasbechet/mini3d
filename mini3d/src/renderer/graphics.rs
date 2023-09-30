use glam::IVec2;
use mini3d_derive::Serialize;

use crate::{
    asset::{handle::AssetHandle, AssetManager},
    ecs::{
        entity::Entity,
        view::single::{StaticSingleView, StaticSingleViewRef},
    },
    feature::renderer::viewport::Viewport,
    math::rect::IRect,
};

use super::{
    color::Color,
    provider::{RendererProvider, RendererProviderError, SceneCanvasHandle},
    RendererResourceManager,
};

#[derive(Clone, Copy, Serialize)]
pub enum TextureWrapMode {
    Clamp,
    Repeat,
    Mirror,
}

#[derive(Serialize)]
enum Command {
    Print {
        position: IVec2,
        start: usize,
        stop: usize,
        font: AssetHandle,
    },
    BlitTexture {
        texture: AssetHandle,
        extent: IRect,
        texture_extent: IRect,
        filtering: Color,
        wrap_mode: TextureWrapMode,
        alpha_threshold: u8,
    },
    BlitViewport {
        position: IVec2,
        viewport: Entity, // TODO: use scene ?
    },
    DrawLine {
        x0: IVec2,
        x1: IVec2,
        color: Color,
    },
    DrawVLine {
        x: i32,
        y0: i32,
        y1: i32,
        color: Color,
    },
    DrawHLine {
        y: i32,
        x0: i32,
        x1: i32,
        color: Color,
    },
    DrawRect {
        extent: IRect,
        color: Color,
    },
    FillRect {
        extent: IRect,
        color: Color,
    },
    Scissor {
        extent: Option<IRect>,
    },
}

#[derive(Default, Serialize)]
pub struct Graphics {
    commands: Vec<Command>,
    text_buffer: String,
}

impl Graphics {
    pub(crate) fn clear(&mut self) {
        self.commands.clear();
        self.text_buffer.clear();
    }

    pub(crate) fn submit_provider(
        &self,
        canvas: Option<SceneCanvasHandle>,
        clear_color: Color,
        resources: &mut RendererResourceManager,
        asset: &mut AssetManager,
        viewports: &StaticSingleViewRef<Viewport>,
        provider: &mut dyn RendererProvider,
    ) -> Result<(), RendererProviderError> {
        if let Some(canvas) = canvas {
            provider.scene_canvas_begin(canvas, clear_color)?;
        } else {
            provider.screen_canvas_begin(clear_color)?;
        }
        for command in self.commands.iter() {
            match command {
                Command::Print {
                    position,
                    start,
                    stop,
                    font,
                } => {
                    let font = resources.request_font(*font, provider, asset)?;
                    let mut position = *position;
                    for c in self.text_buffer[*start..*stop].chars() {
                        let char_extent = font
                            .atlas
                            .extents
                            .get(&c)
                            .expect("Character extent not found");
                        let extent = IRect::new(
                            position.x,
                            position.y,
                            char_extent.width(),
                            char_extent.height(),
                        );
                        provider.canvas_blit_texture(
                            font.handle,
                            extent,
                            *char_extent,
                            Color::WHITE,
                            TextureWrapMode::Clamp,
                            1,
                        )?;
                        position.x += char_extent.width() as i32;
                    }
                }
                Command::BlitTexture {
                    texture,
                    extent,
                    texture_extent,
                    filtering,
                    wrap_mode,
                    alpha_threshold,
                } => {
                    let texture = resources.request_texture(*texture, provider, asset)?;
                    provider.canvas_blit_texture(
                        texture.handle,
                        *extent,
                        *texture_extent,
                        *filtering,
                        *wrap_mode,
                        *alpha_threshold,
                    )?;
                }
                Command::BlitViewport { position, viewport } => {
                    let viewport = viewports.get(*viewport).unwrap();
                    provider.canvas_blit_viewport(viewport.handle, *position)?;
                }
                Command::DrawLine { x0, x1, color } => {
                    provider.canvas_draw_line(*x0, *x1, *color)?;
                }
                Command::DrawVLine { x, y0, y1, color } => {
                    provider.canvas_draw_vline(*x, *y0, *y1, *color)?;
                }
                Command::DrawHLine { y, x0, x1, color } => {
                    provider.canvas_draw_hline(*y, *x0, *x1, *color)?;
                }
                Command::DrawRect { extent, color } => {
                    provider.canvas_draw_rect(*extent, *color)?;
                }
                Command::FillRect { extent, color } => {
                    provider.canvas_fill_rect(*extent, *color)?;
                }
                Command::Scissor { extent } => {
                    provider.canvas_scissor(*extent)?;
                }
            }
        }
        provider.canvas_end()?;
        Ok(())
    }

    pub fn print(&mut self, position: IVec2, text: &str, font: AssetHandle) {
        let start = self.text_buffer.len();
        self.text_buffer.push_str(text);
        let stop = self.text_buffer.len();
        self.commands.push(Command::Print {
            position,
            start,
            stop,
            font,
        });
    }

    pub fn blit_texture(
        &mut self,
        texture: AssetHandle,
        extent: IRect,
        texture_extent: IRect,
        filtering: Color,
        wrap_mode: TextureWrapMode,
        alpha_threshold: u8,
    ) {
        self.commands.push(Command::BlitTexture {
            texture,
            extent,
            texture_extent,
            filtering,
            wrap_mode,
            alpha_threshold,
        });
    }

    pub fn blit_viewport(&mut self, viewport: Entity, position: IVec2) {
        self.commands
            .push(Command::BlitViewport { position, viewport });
    }

    pub fn fill_rect(&mut self, extent: IRect, color: Color) {
        self.commands.push(Command::FillRect { extent, color });
    }

    pub fn draw_rect(&mut self, extent: IRect, color: Color) {
        self.commands.push(Command::DrawRect { extent, color });
    }

    pub fn draw_line(&mut self, x0: IVec2, x1: IVec2, color: Color) {
        self.commands.push(Command::DrawLine { x0, x1, color });
    }

    pub fn draw_vline(&mut self, x: i32, y0: i32, y1: i32, color: Color) {
        self.commands.push(Command::DrawVLine { x, y0, y1, color });
    }

    pub fn draw_hline(&mut self, y: i32, x0: i32, x1: i32, color: Color) {
        self.commands.push(Command::DrawHLine { y, x0, x1, color });
    }

    pub fn scissor(&mut self, extent: Option<IRect>) {
        self.commands.push(Command::Scissor { extent });
    }
}
