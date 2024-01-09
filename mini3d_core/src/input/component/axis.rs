use mini3d_derive::{Reflect, Serialize};

use crate::{
    ecs::{
        component::{Component, ComponentError, ComponentStorage},
        context::Context,
        entity::Entity,
    },
    input::provider::InputProviderHandle,
    math::fixed::I32F16,
    utils::string::AsciiArray,
};

#[derive(Default, Clone, Copy, Serialize)]
pub enum InputAxisRange {
    Clamped {
        min: I32F16,
        max: I32F16,
    },
    Normalized {
        norm: I32F16,
    },
    ClampedNormalized {
        min: I32F16,
        max: I32F16,
        norm: I32F16,
    },
    #[default]
    Infinite,
}

#[derive(Clone, Reflect, Default, Serialize)]
pub struct InputAxisState {
    pub(crate) value: I32F16,
}

#[derive(Clone, Reflect, Default, Serialize)]
pub struct InputAxis {
    pub(crate) name: AsciiArray<64>,
    pub(crate) range: InputAxisRange,
    pub(crate) state: InputAxisState,
    #[serialize(skip)]
    pub(crate) handle: InputProviderHandle,
}

impl InputAxis {
    pub const NAME: &'static str = "RTY_InputAxis";

    pub fn name(&self) -> &str {
        self.name.as_str()
    }

    pub fn set_value(&mut self, value: I32F16) {
        self.state.value = match &self.range {
            InputAxisRange::Clamped { min, max } => value.max(*min).min(*max),
            InputAxisRange::Normalized { norm } => value / norm,
            InputAxisRange::ClampedNormalized { min, max, norm } => {
                value.max(*min).min(*max) / norm
            }
            InputAxisRange::Infinite => value,
        }
    }

    pub fn range(&self) -> InputAxisRange {
        self.range
    }

    pub fn value(&self) -> I32F16 {
        self.state.value
    }
}

impl Component for InputAxis {
    const STORAGE: ComponentStorage = ComponentStorage::Single;

    fn on_added(&mut self, entity: Entity, ctx: &mut Context) -> Result<(), ComponentError> {
        self.handle = ctx
            .input
            .add_axis(self.name.as_str(), entity, &self.range)?;
        Ok(())
    }

    fn on_removed(&mut self, entity: Entity, ctx: &mut Context) -> Result<(), ComponentError> {
        ctx.input.remove_axis(&self.name, self.handle)?;
        Ok(())
    }
}
