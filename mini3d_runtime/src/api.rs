use core::fmt::Arguments;

use alloc::format;
use mini3d_db::{
    database::{ComponentId, Database},
    entity::Entity,
    error::ComponentError,
    field::{ComponentField, Field, FieldType},
    query::Query,
};
use mini3d_input::{
    action::InputActionId,
    axis::{InputAxisId, InputAxisRange},
    InputError, InputManager,
};
use mini3d_logger::{level::LogLevel, LoggerManager};
use mini3d_scheduler::{Scheduler, SchedulerError, StageId, SystemId, SystemOrder};

use crate::{event::ComponentEventStages, execute_stage, Invocation, RuntimeState};

pub struct API<'a> {
    pub(crate) db: &'a mut Database,
    pub(crate) scheduler: &'a mut Scheduler,
    pub(crate) logger: &'a mut LoggerManager,
    pub(crate) input: &'a mut InputManager,
    pub(crate) state: &'a mut RuntimeState,
}

impl<'a> API<'a> {
    /// RUNTIME STATE API

    pub fn request_stop(&mut self) {
        self.state.request_stop = true;
    }

    /// DATABASE API

    pub fn register_tag(&mut self, name: &str) -> Result<ComponentId, ComponentError> {
        self.db.register_tag(name)
    }

    pub fn register(
        &mut self,
        name: &str,
        fields: &[ComponentField],
    ) -> Result<ComponentId, ComponentError> {
        let id = self.db.register(name, fields)?;
        let on_added = self
            .scheduler
            .add_stage(&format!("_on_{}_added", name))
            .unwrap();
        let on_removed = self
            .scheduler
            .add_stage(&format!("_on_{}_removed", name))
            .unwrap();
        self.state.stages.components.insert(
            id,
            ComponentEventStages {
                on_added: Some(on_added),
                on_removed: Some(on_removed),
            },
        );
        self.scheduler.rebuild();
        Ok(id)
    }

    pub fn unregister(&mut self, c: ComponentId) {
        self.db.unregister(c);
        let stages = &self.state.stages.components[c];
        // self.scheduler.remove_stage(stages.on_added);
        // self.scheduler.remove_stage(stages.on_removed);
        self.state.stages.components.remove(c);
        self.scheduler.rebuild();
    }

    pub fn create(&mut self) -> Entity {
        self.db.create()
    }

    pub fn destroy(&mut self, e: Entity) {
        let mut c = None;
        while let Some(n) = self.db.find_next_component(e, c) {
            c = Some(n);
            self.remove(e, n);
        }
        // TODO: find proper solution to prevent corrupted database
        self.db.destroy(e)
    }

    pub fn add_default(&mut self, e: Entity, c: ComponentId) {
        self.db.add_default(e, c);
        execute_stage(self.state.stages.components[c].on_added.unwrap(), self);
    }

    pub fn remove(&mut self, e: Entity, c: ComponentId) {
        execute_stage(self.state.stages.components[c].on_removed.unwrap(), self);
        self.db.remove(e, c)
    }

    pub fn has(&self, e: Entity, c: ComponentId) -> bool {
        self.db.has(e, c)
    }

    pub fn read<T: FieldType>(&self, e: Entity, f: Field<T>) -> Option<T> {
        self.db.read(e, f)
    }

    pub fn write<T: FieldType>(&mut self, e: Entity, f: Field<T>, v: T) {
        self.db.write(e, f, v)
    }

    pub fn entities(&self) -> impl Iterator<Item = Entity> + '_ {
        self.db.entities()
    }

    pub fn query_entities<'b, 'c: 'b>(
        &'c self,
        query: &'b Query,
    ) -> impl Iterator<Item = Entity> + 'b {
        self.db.query_entities(query).into_iter(self.db)
    }

    pub fn find_component(&self, name: &str) -> Option<ComponentId> {
        self.db.find_component(name)
    }

    pub fn find_field<T: FieldType>(&self, c: ComponentId, name: &str) -> Option<Field<T>> {
        self.db.find_field(c, name)
    }

    /// SCHEDULER API

    pub fn find_stage(&self, name: &str) -> Option<StageId> {
        self.scheduler.find_stage(name)
    }

    pub fn tick_stage(&self) -> StageId {
        self.state.stages.tick_stage.unwrap()
    }

    pub fn start_stage(&self) -> StageId {
        self.state.stages.start_stage.unwrap()
    }

    pub fn on_component_added_stage(&self, c: ComponentId) -> StageId {
        self.state.stages.components[c].on_added.unwrap()
    }

    pub fn on_component_removed_stage(&self, c: ComponentId) -> StageId {
        self.state.stages.components[c].on_removed.unwrap()
    }

    pub fn find_system(&self, name: &str) -> Option<SystemId> {
        self.scheduler.find_system(name)
    }

    pub fn create_stage(&mut self, name: &str) -> Result<StageId, SchedulerError> {
        let stage = self.scheduler.add_stage(name)?;
        self.scheduler.rebuild();
        Ok(stage)
    }

    pub fn create_system(
        &mut self,
        name: &str,
        stage: StageId,
        order: SystemOrder,
        callback: fn(&mut API),
    ) -> Result<SystemId, SchedulerError> {
        let system = self.scheduler.add_system(name, stage, order)?;
        self.state.systems.insert(system, Some(callback));
        self.scheduler.rebuild();
        Ok(system)
    }

    pub fn invoke(&mut self, stage: StageId, invocation: Invocation) {
        match invocation {
            Invocation::Immediate => {
                // Recursive call
                execute_stage(stage, self);
            }
            Invocation::NextStage => {
                // Called after the current stage
                self.state.stages.next_stages.push_back(stage);
            }
            Invocation::NextTick => {
                // Called on the next tick
                self.state.stages.next_tick_stages.push_back(stage);
            }
        }
    }

    pub fn debug_sched(&mut self) {
        for stage in self.scheduler.iter_stages() {
            let stage = self.scheduler.stage(stage).unwrap();
            self.logger.log(format_args!("STAGE {}", stage.name), LogLevel::Debug, None);
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

    pub fn add_action(&mut self, name: &str) -> Result<InputActionId, InputError> {
        self.input.add_action(name)
    }

    pub fn remove_action(&mut self, id: InputActionId) -> Result<(), InputError> {
        self.input.remove_action(id)
    }

    pub fn create_axis(
        &mut self,
        name: &str,
        range: InputAxisRange,
    ) -> Result<InputAxisId, InputError> {
        self.input.add_axis(name, range)
    }

    pub fn delete_axis(&mut self, id: InputAxisId) -> Result<(), InputError> {
        self.input.remove_axis(id)
    }
    pub fn find_action(&self, name: &str) -> Option<InputActionId> {
        self.input.find_action(name).map(|(id, _)| id)
    }

    pub fn find_axis(&self, name: &str) -> Option<InputAxisId> {
        self.input.find_axis(name).map(|(id, _)| id)
    }

    pub fn is_action_pressed(&self, id: InputActionId) -> bool {
        self.input.action(id).unwrap().is_pressed()
    }

    pub fn is_action_released(&self, id: InputActionId) -> bool {
        self.input.action(id).unwrap().is_released()
    }

    pub fn is_action_just_pressed(&self, id: InputActionId) -> bool {
        self.input.action(id).unwrap().is_just_pressed()
    }

    pub fn is_action_just_released(&self, id: InputActionId) -> bool {
        self.input.action(id).unwrap().is_just_released()
    }
}

#[macro_export]
macro_rules! info {
    ($api:ident, $($arg:tt)*) => {{
        $api.log(format_args!($($arg)*), $crate::level::LogLevel::Info, Some((file!(), line!())));
    }}
}

#[macro_export]
macro_rules! debug {
    ($api:ident, $($arg:tt)*) => {{
        $api.log(format_args!($($arg)*), $crate::level::LogLevel::Debug, Some((file!(), line!())));
    }}
}

#[macro_export]
macro_rules! warn {
    ($api:ident, $($arg:tt)*) => {{
        $api.log(format_args!($($arg)*), $crate::level::LogLevel::Warning, Some((file!(), line!())));
    }}
}

#[macro_export]
macro_rules! error {
    ($api:ident, $($arg:tt)*) => {{
        $api.log(format_args!($($arg)*), $crate::level::LogLevel::Error, Some((file!(), line!())));
    }}
}
