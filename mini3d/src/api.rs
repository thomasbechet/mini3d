use core::fmt::{Arguments, Display};

use alloc::boxed::Box;
use mini3d_db::{
    database::{ComponentHandle, Database, GetComponentHandle},
    entity::Entity,
    field::{ComponentField, Field, FieldType},
    query::Query,
};
use mini3d_input::{action::InputActionHandle, axis::InputAxisHandle, InputManager};
use mini3d_logger::{level::LogLevel, LoggerManager};
use mini3d_math::mat::M4I32F16;
use mini3d_renderer::{
    camera::CameraId, font::FontId, mesh::MeshHandle, renderpass::RenderPassId,
    rendertarget::RenderTargetId, texture::TextureHandle, transform::RenderTransformId,
    RendererManager,
};
use mini3d_scheduler::{Scheduler, StageId, SystemHandle, SystemOrder};
use mini3d_utils::{handle::Handle, slotmap::DefaultKey};

use crate::{
    event::{Event, UserEventHandle},
    execute_stage,
    system::IntoSystem,
    Invocation, RuntimeState,
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

    pub fn register_component(&mut self, name: &str, fields: &[ComponentField]) -> ComponentHandle {
        let id = self.database.register(name, fields).unwrap();
        self.register_component_callbacks(id);
        id
    }

    pub fn register_component_tag(&mut self, name: &str) -> ComponentHandle {
        let id = self.database.register_tag(name).unwrap();
        self.register_component_callbacks(id);
        id
    }

    pub fn register_component_handle(&mut self, name: &str) -> ComponentHandle {
        let id = self.database.register_handle(name).unwrap();
        self.register_component_callbacks(id);
        id
    }

    fn register_component_callbacks(&mut self, c: ComponentHandle) {
        self.state
            .events
            .register_component_callbacks(self.scheduler, c);
        self.state.rebuild_scheduler = true;
    }

    pub fn unregister_component(&mut self, c: ComponentHandle) {
        self.database.unregister_component(c);
        self.state
            .events
            .unregister_component_callbacks(self.scheduler, c);
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
        execute_stage(self.state.events.component[c.handle()].added.unwrap(), self);
    }

    pub fn remove(&mut self, e: Entity, c: impl GetComponentHandle) {
        execute_stage(
            self.state.events.component[c.handle()].removed.unwrap(),
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

    pub(crate) fn write_handle(
        &mut self,
        e: Entity,
        c: impl GetComponentHandle,
        v: impl Into<Handle>,
    ) {
        self.database.write_handle(e, c.handle(), v.into())
    }

    pub(crate) fn read_handle(&self, e: Entity, c: impl GetComponentHandle) -> Handle {
        self.database.read_handle(e, c.handle())
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

    pub fn find_event(&self, name: &str) -> Entity {
        self.state.events.find_user_event(name).unwrap()
    }

    pub fn find_system(&self, name: &str) -> Option<SystemHandle> {
        self.scheduler.find_system(name)
    }

    pub fn register_system<Params>(
        &mut self,
        name: &str,
        event: Event,
        order: SystemOrder,
        callback: impl IntoSystem<Params>,
    ) -> SystemHandle {
        let stage = self.state.events.get_stage_from_event(event).unwrap();
        let id = self.scheduler.add_system(name, stage, order).unwrap();
        self.state
            .created_native_systems
            .push((id, Box::new(callback.into_system())));
        self.state.rebuild_scheduler = true;
        id
    }

    pub fn invoke(&mut self, event: UserEventHandle, invocation: Invocation) {
        let stage = self.state.events.user[event].stage.unwrap();
        match invocation {
            Invocation::Immediate => {
                // Recursive call
                execute_stage(stage, self);
            }
            Invocation::NextStage => {
                // Called after the current stage
                self.state.next_stages.push_back(stage);
            }
            Invocation::NextTick => {
                // Called on the next tick
                self.state.next_tick_stages.push_back(stage);
            }
        }
    }

    pub fn debug_sched(&mut self) {
        self.state.events.debug(self.logger);
    }

    /// LOGGER API

    pub fn log(&self, args: Arguments<'_>, level: LogLevel, source: Option<(&'static str, u32)>) {
        self.logger.log(args, level, source)
    }

    pub fn set_max_log_level(&mut self, level: LogLevel) {
        self.logger.set_max_level(level)
    }

    /// INPUT API

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

    pub fn bind_texture(&mut self, pass: RenderPassId, texture: TextureHandle) {}

    pub fn bind_font(&mut self, font: FontId) {}

    pub fn draw_mesh(&mut self, pass: RenderPassId, mesh: MeshHandle) {}

    pub fn draw_mesh_skinned(&mut self, pass: RenderPassId, mesh: MeshHandle) {}

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
