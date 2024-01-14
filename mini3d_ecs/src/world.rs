use crate::{
    container::ContainerTable, entity::EntityTable, runner::Runner, scheduler::Scheduler,
    system::SystemTable,
};

pub struct World {
    entities: EntityTable,
    containers: ContainerTable,
    scheduler: Scheduler,
    systems: SystemTable,
    runner: Runner,
}

impl World {
    pub fn new() -> Self {
        let mut world = Self {
            entities: EntityTable::default(),
            containers: ContainerTable::default(),
            scheduler: Scheduler::default(),
            systems: SystemTable::default(),
            runner: Runner::default(),
        };
        world
    }

    pub fn update(&mut self) {}
}

#[cfg(test)]
mod test {
    use super::*;

    #[test]
    fn test() {
        let mut world = World::new();
        world.update();
    }
}
