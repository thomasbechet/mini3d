use mini3d_db::{
    container::ComponentId,
    database::Database,
    entity::Entity,
    error::ComponentError,
    field::{ComponentField, Field, FieldType},
    query::{EntityQuery, Query},
};
use mini3d_input::{
    action::InputActionId,
    axis::{InputAxisId, InputAxisRange},
    InputError, InputManager,
};
use mini3d_scheduler::{Scheduler, SchedulerError, StageId, SystemId, SystemOrder};

use crate::{CallbackList, RuntimeState};

pub struct API<'a> {
    pub(crate) db: &'a mut Database,
    pub(crate) scheduler: &'a mut Scheduler,
    pub(crate) input: &'a mut InputManager,
    pub(crate) state: &'a mut RuntimeState,
    pub(crate) callbacks: &'a mut CallbackList,
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
        self.db.register(name, fields)
    }

    pub fn unregister(&mut self, c: ComponentId) {
        self.db.unregister(c)
    }

    pub fn create(&mut self) -> Entity {
        self.db.create()
    }

    pub fn destroy(&mut self, e: Entity) {
        self.db.destroy(e)
    }

    pub fn add_default(&mut self, e: Entity, c: ComponentId) {
        self.db.add_default(e, c)
    }

    pub fn remove(&mut self, e: Entity, c: ComponentId) {
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

    pub fn find_component(&self, name: &str) -> Option<ComponentId> {
        self.db.find_component(name)
    }

    pub fn find_field<T: FieldType>(&self, c: ComponentId, name: &str) -> Option<Field<T>> {
        self.db.find_field(c, name)
    }

    pub fn query_entities<'b>(&self, query: &'b Query) -> EntityQuery<'b> {
        self.db.query_entities(query)
    }

    /// SCHEDULER API

    pub fn find_stage(&self, name: &str) -> Option<StageId> {
        self.scheduler.find_stage(name)
    }

    pub fn find_system(&self, name: &str) -> Option<SystemId> {
        self.scheduler.find_system(name)
    }

    pub fn add_stage(&mut self, name: &str) -> Result<StageId, SchedulerError> {
        let stage = self.scheduler.add_stage(name)?;
        self.scheduler.rebuild();
        Ok(stage)
    }

    pub fn add_system(
        &mut self,
        name: &str,
        stage: StageId,
        order: SystemOrder,
        callback: fn(&mut API),
    ) -> Result<SystemId, SchedulerError> {
        let system = self.scheduler.add_system(name, stage, order)?;
        self.callbacks.insert(system, Some(callback));
        self.scheduler.rebuild();
        Ok(system)
    }

    /// SCHEDULER API

    pub fn add_action(&mut self, name: &str) -> Result<InputActionId, InputError> {
        self.input.add_action(name)
    }

    pub fn remove_action(&mut self, id: InputActionId) -> Result<(), InputError> {
        self.input.remove_action(id)
    }

    pub fn add_axis(
        &mut self,
        name: &str,
        range: InputAxisRange,
    ) -> Result<InputAxisId, InputError> {
        self.input.add_axis(name, range)
    }

    pub fn remove_axis(&mut self, id: InputAxisId) -> Result<(), InputError> {
        self.input.remove_axis(id)
    }

    /// INPUT API
    
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
