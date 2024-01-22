use core::{any::Any, cell::UnsafeCell};

use alloc::boxed::Box;
use mini3d_utils::{
    slot_map_key,
    slotmap::{Key, SlotMap},
};

use crate::{
    bitset::Bitset,
    component::{
        component_type::{ComponentKind, ComponentType},
        identifier::Identifier,
        system::System,
        system_stage::SystemStage,
        Component,
    },
    context::Context,
    entity::Entity,
    error::ComponentError,
};

use self::native::NativeSingleContainer;

pub mod native;

pub(crate) trait Container {
    fn as_any(&self) -> &dyn Any;
    fn as_any_mut(&mut self) -> &mut dyn Any;
    fn bitset(&self) -> &Bitset;
    fn remove(&mut self, ctx: &mut Context, entity: Entity) -> Result<(), ComponentError>;
}

pub(crate) type ContainerWrapper = Box<UnsafeCell<dyn Container>>;

slot_map_key!(ContainerKey);

pub(crate) struct ContainerEntry {
    pub(crate) container: ContainerWrapper,
    pub(crate) entity: Entity,
}

pub(crate) struct ContainerTable {
    pub(crate) entries: SlotMap<ContainerKey, ContainerEntry>,
    pub(crate) component_type_key: ContainerKey,
    pub(crate) system_key: ContainerKey,
    pub(crate) system_stage_key: ContainerKey,
    pub(crate) identifier_key: ContainerKey,
}

impl ContainerTable {
    pub(crate) fn component_type(&self, entity: Entity) -> Option<&ComponentType> {
        self.get(self.component_type_key)
            .unwrap()
            .as_any()
            .downcast_ref::<NativeSingleContainer<ComponentType>>()
            .unwrap()
            .get(entity)
    }

    pub(crate) fn component_type_mut(&mut self, entity: Entity) -> Option<&mut ComponentType> {
        self.get_mut(self.component_type_key)
            .unwrap()
            .as_any_mut()
            .downcast_mut::<NativeSingleContainer<ComponentType>>()
            .unwrap()
            .get_mut(entity)
    }

    pub(crate) fn find_container_key(&self, ty: Entity) -> Option<ContainerKey> {
        self.component_types().get(ty).map(|ty| ty.key)
    }

    pub(crate) fn find_component_type_by_name(&self, name: &str) -> Option<Entity> {
        for (entity, ty) in self.component_types().iter() {
            println!("{} == {}", ty.name, name);
        }
        self.component_types()
            .iter()
            .find(|(_, cty)| cty.name == name)
            .map(|(e, _)| e)
    }

    pub(crate) fn component_types_mut(&mut self) -> &mut NativeSingleContainer<ComponentType> {
        self.get_mut(self.component_type_key)
            .unwrap()
            .as_any_mut()
            .downcast_mut::<NativeSingleContainer<ComponentType>>()
            .unwrap()
    }

    pub(crate) fn component_types(&self) -> &NativeSingleContainer<ComponentType> {
        self.get(self.component_type_key)
            .unwrap()
            .as_any()
            .downcast_ref::<NativeSingleContainer<ComponentType>>()
            .unwrap()
    }

    pub(crate) fn systems(&mut self) -> &mut NativeSingleContainer<System> {
        self.get_mut(self.system_key)
            .unwrap()
            .as_any_mut()
            .downcast_mut::<NativeSingleContainer<System>>()
            .unwrap()
    }

    pub(crate) fn system_stages(&mut self) -> &mut NativeSingleContainer<SystemStage> {
        self.get_mut(self.system_stage_key)
            .unwrap()
            .as_any_mut()
            .downcast_mut::<NativeSingleContainer<SystemStage>>()
            .unwrap()
    }

    pub(crate) fn identifiers_mut(&mut self) -> &mut NativeSingleContainer<Identifier> {
        self.get_mut(self.identifier_key)
            .unwrap()
            .as_any_mut()
            .downcast_mut::<NativeSingleContainer<Identifier>>()
            .unwrap()
    }

    pub(crate) fn identifiers(&self) -> &NativeSingleContainer<Identifier> {
        self.get(self.identifier_key)
            .unwrap()
            .as_any()
            .downcast_ref::<NativeSingleContainer<Identifier>>()
            .unwrap()
    }

    pub(crate) fn get(&self, key: ContainerKey) -> Option<&dyn Container> {
        self.entries
            .get(key)
            .map(|entry| unsafe { &*entry.container.get() })
    }

    pub(crate) fn get_mut(&mut self, key: ContainerKey) -> Option<&mut dyn Container> {
        self.entries
            .get_mut(key)
            .map(|entry| unsafe { &mut *entry.container.get() })
    }

    pub(crate) fn new(component_type_entity: Entity) -> Self {
        let mut table = Self {
            entries: SlotMap::with_key(),
            component_type_key: ContainerKey::null(),
            system_key: Default::default(),
            system_stage_key: Default::default(),
            identifier_key: Default::default(),
        };

        // Register component type container
        table.component_type_key = table.entries.add(ContainerEntry {
            container: Box::new(UnsafeCell::new(
                NativeSingleContainer::<ComponentType>::with_capacity(128),
            )),
            entity: Entity::null(),
        });

        // Add component type entity
        let component_type_key = table.component_type_key;
        table.component_types_mut().add(
            component_type_entity,
            ComponentType {
                name: ComponentType::NAME.into(),
                kind: ComponentKind::Tag, // TODO: find better type...
                auto_enable: false,
                key: component_type_key,
            },
        );

        table
    }
}
