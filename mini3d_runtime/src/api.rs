use mini3d_db::{database::Database, container::Component, error::ComponentError, field::{ComponentField, FieldType, Field}, entity::Entity, query::{Query, EntityQuery}};
use mini3d_input::InputManager;

pub struct API<'a> {
    pub(crate) db: &'a mut Database,
    pub(crate) input: &'a mut InputManager,
}

impl<'a> API<'a>  {
    pub fn register_tag(&mut self, name: &str) -> Result<Component, ComponentError> {
        self.db.register_tag(name)
    }

    pub fn register(
        &mut self,
        name: &str,
        fields: &[ComponentField],
    ) -> Result<Component, ComponentError> {
        self.db.register(name, fields)
    }

    pub fn unregister(&mut self, c: Component) {
        self.db.unregister(c)
    }

    pub fn create(&mut self) -> Entity {
        self.db.create()
    }

    pub fn destroy(&mut self, e: Entity) {
        self.db.destroy(e)
    }

    pub fn add_default(&mut self, e: Entity, c: Component) {
        self.db.add_default(e, c)        
    }

    pub fn remove(&mut self, e: Entity, c: Component) {
        self.db.remove(e, c)
    }

    pub fn has(&self, e: Entity, c: Component) -> bool {
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

    pub fn find_component(&self, name: &str) -> Option<Component> {
        self.db.find_component(name)
    }

    pub fn find_field<T: FieldType>(&self, c: Component, name: &str) -> Option<Field<T>> {
        self.db.find_field(c, name)
    }

    pub fn query_entities<'b>(&self, query: &'b Query) -> EntityQuery<'b> {
        self.db.query_entities(query)
    }
}
