use core::fmt::{Arguments, Display};

use alloc::{boxed::Box, format};
use mini3d_db::{
    database::{ComponentHandle, Database, GetComponentHandle},
    entity::Entity,
    field::{ComponentField, Field, FieldType},
    query::Query,
};
use mini3d_input::{
    action::InputActionHandle,
    axis::{InputAxisHandle, InputAxisRange},
    InputError, InputManager,
};
use mini3d_logger::{level::LogLevel, LoggerManager};
use mini3d_math::mat::M4I32F16;
use mini3d_renderer::{
    camera::CameraId, font::FontId, mesh::MeshId, renderpass::RenderPassId,
    rendertarget::RenderTargetId, texture::TextureId, transform::RenderTransformId,
    RendererManager,
};
use mini3d_scheduler::{Scheduler, SchedulerError, StageHandle, SystemHandle, SystemOrder};
use mini3d_utils::slotmap::DefaultKey;

use crate::{
    event::ComponentEventStages, execute_stage, system::IntoSystem, Invocation, RuntimeState,
};

pub struct API<'a> {
    pub(crate) database: &'a mut Database,
    pub(crate) scheduler: &'a mut Scheduler,
    pub(crate) logger: &'a mut LoggerManager,
    pub(crate) input: &'a mut InputManager,
    pub(crate) renderer: &'a mut RendererManager,
    pub(crate) state: &'a mut RuntimeState,
}

impl<'a> API<'a> {
    /// RUNTIME STATE API

    pub fn request_stop(&mut self) {
        self.state.request_stop = true;
    }

    pub fn event_entity(&self) -> Entity {
        self.state.event_entity
    }

    /// DATABASE API

    pub fn register_component_tag(&mut self, name: &str) -> ComponentHandle {
        let id = self.database.register_tag(name).unwrap();
        self.register_component_callbacks(id, name);
        id
    }

    pub fn register_component(&mut self, name: &str, fields: &[ComponentField]) -> ComponentHandle {
        let id = self.database.register(name, fields).unwrap();
        self.register_component_callbacks(id, name);
        id
    }

    pub fn register_component_key(&mut self, name: &str) -> ComponentHandle {
        let id = self.database.register_key(name).unwrap();
        self.register_component_callbacks(id, name);
        id
    }

    fn register_component_callbacks(&mut self, c: ComponentHandle, name: &str) {
        let on_added = self
            .scheduler
            .add_stage(&format!("_on_{}_added", name))
            .unwrap();
        let on_removed = self
            .scheduler
            .add_stage(&format!("_on_{}_removed", name))
            .unwrap();
        self.state.base_stages.components.insert(
            c,
            ComponentEventStages {
                on_added: Some(on_added),
                on_removed: Some(on_removed),
            },
        );
        self.state.rebuild_scheduler = true;
    }

    pub fn unregister_component(&mut self, c: ComponentHandle) {
        self.database.unregister_component(c);
        let stages = &self.state.base_stages.components[c];
        // self.scheduler.remove_stage(stages.on_added);
        // self.scheduler.remove_stage(stages.on_removed);
        self.state.base_stages.components.remove(c);
        self.state.rebuild_scheduler = true;
    }

    pub fn spawn(&mut self) -> Entity {
        self.database.create()
    }

    pub fn despawn(&mut self, e: Entity) {
        let mut c = None;
        while let Some(n) = self.database.find_next_component(e, c) {
            c = Some(n);
            self.remove(e, n);
        }
        // TODO: find proper solution to prevent corrupted database
        self.database.delete(e)
    }

    pub fn add_default(&mut self, e: Entity, c: impl GetComponentHandle) {
        self.database.add_default(e, c.handle());
        execute_stage(
            self.state.base_stages.components[c.handle()]
                .on_added
                .unwrap(),
            self,
        );
    }

    pub fn remove(&mut self, e: Entity, c: impl GetComponentHandle) {
        execute_stage(
            self.state.base_stages.components[c.handle()]
                .on_removed
                .unwrap(),
            self,
        );
        self.database.remove(e, c.handle())
    }

    pub fn has(&self, e: Entity, c: impl GetComponentHandle) -> bool {
        self.database.has(e, c.handle())
    }

    pub fn read<T: FieldType>(&self, e: Entity, f: Field<T>) -> Option<T> {
        self.database.read(e, f)
    }

    pub fn write<T: FieldType>(&mut self, e: Entity, f: Field<T>, v: T) {
        self.database.write(e, f, v)
    }

    pub fn read_key(&self, e: Entity, c: impl GetComponentHandle) -> Option<DefaultKey> {
        self.database.read_key(e, c.handle())
    }

    pub fn write_key(&mut self, e: Entity, c: impl GetComponentHandle, v: DefaultKey) {
        self.database.write_key(e, c.handle(), v)
    }

    pub fn entities(&self) -> impl Iterator<Item = Entity> + '_ {
        self.database.entities()
    }

    pub fn query_entities<'b, 'c: 'b>(
        &'c self,
        query: &'b Query,
    ) -> impl Iterator<Item = Entity> + 'b {
        self.database.query_entities(query).into_iter(self.database)
    }

    pub fn find_component(&self, name: &str) -> Option<ComponentHandle> {
        self.database.find_component(name)
    }

    pub fn find_field<T: FieldType>(
        &self,
        c: impl GetComponentHandle,
        name: &str,
    ) -> Option<Field<T>> {
        self.database.find_field(c.handle(), name)
    }

    pub fn dump(&self, e: Entity) {
        struct EntityFormatter<'a>(Entity, &'a Database);
        impl<'a> Display for EntityFormatter<'a> {
            fn fmt(&self, f: &mut core::fmt::Formatter<'_>) -> core::fmt::Result {
                self.1.display(f, self.0)
            }
        }
        self.log(
            format_args!("{}", EntityFormatter(e, self.database)),
            LogLevel::Info,
            None,
        )
    }

    /// SCHEDULER API

    pub fn find_stage(&self, name: &str) -> Option<StageHandle> {
        self.scheduler.find_stage(name)
    }

    pub fn tick_stage(&self) -> StageHandle {
        self.state.base_stages.tick_stage.unwrap()
    }

    pub fn start_stage(&self) -> StageHandle {
        self.state.base_stages.start_stage.unwrap()
    }

    pub fn on_component_added_stage(&self, c: impl GetComponentHandle) -> StageHandle {
        self.state.base_stages.components[c.handle()]
            .on_added
            .unwrap()
    }

    pub fn on_component_removed_stage(&self, c: impl GetComponentHandle) -> StageHandle {
        self.state.base_stages.components[c.handle()]
            .on_removed
            .unwrap()
    }

    pub fn find_system(&self, name: &str) -> Option<SystemHandle> {
        self.scheduler.find_system(name)
    }

    pub fn register_stage(&mut self, name: &str) -> Result<StageHandle, SchedulerError> {
        let id = self.scheduler.add_stage(name)?;
        self.state.rebuild_scheduler = true;
        Ok(id)
    }

    pub fn register_system<Params>(
        &mut self,
        name: &str,
        stage: StageHandle,
        order: SystemOrder,
        callback: impl IntoSystem<Params>,
    ) -> SystemHandle {
        let id = self.scheduler.add_system(name, stage, order).unwrap();
        self.state
            .created_native_systems
            .push((id, Box::new(callback.into_system())));
        self.state.rebuild_scheduler = true;
        id
    }

    pub fn invoke(&mut self, stage: StageHandle, invocation: Invocation) {
        match invocation {
            Invocation::Immediate => {
                // Recursive call
                execute_stage(stage, self);
            }
            Invocation::NextStage => {
                // Called after the current stage
                self.state.base_stages.next_stages.push_back(stage);
            }
            Invocation::NextTick => {
                // Called on the next tick
                self.state.base_stages.next_tick_stages.push_back(stage);
            }
        }
    }

    pub fn debug_sched(&mut self) {
        for stage in self.scheduler.iter_stages() {
            let stage = self.scheduler.stage(stage).unwrap();
            self.logger
                .log(format_args!("STAGE {}", stage.name), LogLevel::Debug, None);
        }
    }

    /// LOGGER API

    pub fn log(&self, args: Arguments<'_>, level: LogLevel, source: Option<(&'static str, u32)>) {
        self.logger.log(args, level, source)
    }

    pub fn set_max_log_level(&mut self, level: LogLevel) {
        self.logger.set_max_level(level)
    }

    /// INPUT API

    pub fn create_action(&mut self, name: &str) -> Result<InputActionHandle, InputError> {
        self.input.create_action(name)
    }

    pub fn delete_action(&mut self, handle: InputActionHandle) -> Result<(), InputError> {
        self.input.delete_action(handle)
    }

    pub fn create_axis(
        &mut self,
        name: &str,
        range: InputAxisRange,
    ) -> Result<InputAxisHandle, InputError> {
        self.input.create_axis(name, range)
    }

    pub fn delete_axis(&mut self, handle: InputAxisHandle) -> Result<(), InputError> {
        self.input.delete_axis(handle)
    }
    pub fn find_action(&self, name: &str) -> Option<InputActionHandle> {
        self.input.find_action(name).map(|(id, _)| id)
    }

    pub fn find_axis(&self, name: &str) -> Option<InputAxisHandle> {
        self.input.find_axis(name).map(|(id, _)| id)
    }

    pub fn is_action_pressed(&self, handle: InputActionHandle) -> bool {
        self.input.action(handle).unwrap().is_pressed()
    }

    pub fn is_action_released(&self, handle: InputActionHandle) -> bool {
        self.input.action(handle).unwrap().is_released()
    }

    pub fn is_action_just_pressed(&self, handle: InputActionHandle) -> bool {
        self.input.action(handle).unwrap().is_just_pressed()
    }

    pub fn is_action_just_released(&self, handle: InputActionHandle) -> bool {
        self.input.action(handle).unwrap().is_just_released()
    }

    /// RENDERER API

    pub fn delete_camera(&mut self, handle: CameraId) {}

    pub fn update_camera(&mut self, camera: CameraId, view: M4I32F16) {}

    pub fn create_render_transform(&mut self) -> RenderTransformId {
        unimplemented!()
    }

    pub fn delete_render_transform(&mut self, handle: RenderTransformId) {}

    pub fn update_render_transform(&mut self, handle: RenderTransformId, matrix: M4I32F16) {}

    pub fn create_render_target(&mut self) -> RenderTargetId {
        unimplemented!()
    }

    pub fn screen_render_target(&mut self) -> RenderTargetId {
        unimplemented!()
    }

    pub fn delete_render_target(&mut self, handle: RenderTargetId) {}

    pub fn create_unlit_pass(&mut self) -> RenderPassId {
        unimplemented!()
    }

    pub fn create_canvas_pass(&mut self) -> RenderPassId {
        unimplemented!()
    }

    pub fn delete_pass(&mut self, handle: RenderPassId) {}

    pub fn bind_transform(&mut self, pass: RenderPassId, transform: RenderTransformId) {}

    pub fn bind_texture(&mut self, pass: RenderPassId, texture: TextureId) {}

    pub fn bind_font(&mut self, font: FontId) {}

    pub fn draw_mesh(&mut self, pass: RenderPassId, mesh: MeshId) {}

    pub fn draw_mesh_skinned(&mut self, pass: RenderPassId, mesh: MeshId) {}

    pub fn draw_billboard(&mut self, pass: RenderPassId, transform: RenderTransformId) {}

    pub fn submit_unlit_pass(
        &mut self,
        pass: RenderPassId,
        target: RenderTargetId,
        camera: CameraId,
    ) {
    }

    pub fn submut_canvas_pass(&mut self, pass: RenderPassId) {}
}

#[macro_export]
macro_rules! info {
    ($api:ident, $($arg:tt)*) => {{
        $api.log(format_args!($($arg)*), $crate::logger::level::LogLevel::Info, Some((file!(), line!())));
    }}
}

#[macro_export]
macro_rules! debug {
    ($api:ident, $($arg:tt)*) => {{
        $api.log(format_args!($($arg)*), $crate::logger::level::LogLevel::Debug, Some((file!(), line!())));
    }}
}

#[macro_export]
macro_rules! warn {
    ($api:ident, $($arg:tt)*) => {{
        $api.log(format_args!($($arg)*), $crate::logger::level::LogLevel::Warning, Some((file!(), line!())));
    }}
}

#[macro_export]
macro_rules! error {
    ($api:ident, $($arg:tt)*) => {{
        $api.log(format_args!($($arg)*), $crate::logger::level::LogLevel::Error, Some((file!(), line!())));
    }}
}
