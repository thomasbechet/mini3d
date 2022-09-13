use glam::{UVec2, uvec2, IVec2, Mat4};
use slotmap::{SlotMap, new_key_type};

use crate::{math::rect::IRect, asset::{AssetManager, MeshId, MaterialId, FontId}};

use self::immediate_command::ImmediateCommand;

pub mod rasterizer;
pub mod immediate_command;

// 3:2 aspect ratio
// pub const SCREEN_WIDTH: u32 = 480;
// pub const SCREEN_HEIGHT: u32 = 320;
// // 4:3 aspect ratio
// pub const SCREEN_WIDTH: u32 = 512;
// pub const SCREEN_HEIGHT: u32 = 384;
// // 16:10 aspect ratio
pub const SCREEN_WIDTH: u32 = 432;
pub const SCREEN_HEIGHT: u32 = 240;
// // 16:9 aspect ratio
// pub const SCREEN_WIDTH: u32 = 384;
// pub const SCREEN_HEIGHT: u32 = 216;

pub const SCREEN_PIXEL_COUNT: usize = SCREEN_WIDTH as usize * SCREEN_HEIGHT as usize;
pub const SCREEN_RESOLUTION: UVec2 = uvec2(SCREEN_WIDTH, SCREEN_HEIGHT);
pub const SCREEN_VIEWPORT: IRect = IRect::new(0, 0, SCREEN_WIDTH, SCREEN_HEIGHT);
pub const SCREEN_ASPECT_RATIO: f32 = SCREEN_WIDTH as f32 / SCREEN_HEIGHT as f32;

new_key_type! { 
    pub struct ModelId;
    pub struct StaticModelId;
}

pub struct Model {
    pub transform: Mat4,
    pub transform_changed: bool,
    pub mesh: MeshId,
    pub materials: Vec<MaterialId>,
}

impl Model {
    pub(crate) fn set_transform(&mut self, transform: Mat4) {
        self.transform = transform;
        self.transform_changed = true;
    }
    pub(crate) fn set_mesh(&mut self, mesh: MeshId) {
        self.mesh = mesh;
    }
}

#[derive(Default)]
pub struct Graphics {
    immediate_commands: Vec<ImmediateCommand>,
    models: SlotMap<ModelId, Model>,
    added_models: Vec<ModelId>,
    removed_models: Vec<ModelId>,
}

impl Graphics {

    pub(crate) fn prepare(&mut self) {
        self.immediate_commands.clear();
        self.models.values_mut().for_each(|m| {
            m.transform_changed = false;
        });
        self.added_models.clear();
        self.removed_models.clear();
    }
    
    pub(crate) fn print(&mut self, p: IVec2, text: &str, font_id: FontId) {
        self.immediate_commands.push(ImmediateCommand::Print { p, text: String::from(text), font_id })
    }
    // pub(crate) fn draw_line(&mut self, p0: IVec2, p1: IVec2) {
    //     self.commands.push(ImmediateCommand::DrawLine { p0, p1 });
    // }
    // pub(crate) fn draw_vline(&mut self, x: i32, y0: i32, y1: i32) {
    //     self.commands.push(ImmediateCommand::DrawVLine { x, y0, y1 });
    // }
    // pub(crate) fn draw_hline(&mut self, y: i32, x0: i32, x1: i32) {
    //     self.commands.push(ImmediateCommand::DrawHLine { y, x0, x1 });
    // }
    pub(crate) fn draw_rect(&mut self, rect: IRect) {
        self.immediate_commands.push(ImmediateCommand::DrawRect { rect });
    }
    pub(crate) fn fill_rect(&mut self, rect: IRect) {
        self.immediate_commands.push(ImmediateCommand::FillRect { rect });
    }

    pub(crate) fn add_model(&mut self, mesh: MeshId, material: MaterialId) -> ModelId {
        let id = self.models.insert(Model { 
            transform: Mat4::IDENTITY,
            transform_changed: true,
            mesh,
            materials: vec![material],
        });
        self.added_models.push(id);
        id
    }
    pub(crate) fn remove_model(&mut self, id: ModelId) {
        if self.models.remove(id).is_some() {
            self.removed_models.push(id);
        }
    }

    pub fn immediate_commands(&self) -> &[ImmediateCommand] {
        &self.immediate_commands
    }
    pub fn model(&self, id: ModelId) -> Option<&Model> {
        self.models.get(id)
    }
    pub fn models(&self) -> impl Iterator<Item = (ModelId, &Model)> {
        self.models.iter()
    }
    pub fn added_models(&self) -> &[ModelId] {
        &self.added_models
    }
    pub fn removed_models(&self) -> &[ModelId] {
        &self.removed_models
    }
}