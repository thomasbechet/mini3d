use crate::{
    ecs::{
        api::{ecs::ParallelECS, ParallelAPI},
        instance::ParallelResolver,
        query::Query,
    },
    expect,
    registry::{component::StaticComponent, error::RegistryError, system::ParallelSystem},
};

use super::ui::UI;

#[derive(Default)]
pub struct UpdateUI {
    ui: StaticComponent<UI>,
    query: Query,
}

impl UpdateUI {
    pub const NAME: &'static str = "update_ui";
}

impl ParallelSystem for UpdateUI {
    fn setup(&mut self, resolver: &mut ParallelResolver) -> Result<(), RegistryError> {
        self.ui = resolver.write(UI::NAME)?;
        self.query = resolver.query().all(&[UI::NAME])?.build();
        Ok(())
    }

    fn run(&self, ecs: &mut ParallelECS, api: &mut ParallelAPI) {
        let mut uis = ecs.view_mut(self.ui);
        for e in ecs.query(self.query) {
            let ui = &mut uis[e];
            expect!(api, ui.update(api.time.global()));
            for event in ui.events() {
                println!("{:?}", event);
            }
        }
    }
}
