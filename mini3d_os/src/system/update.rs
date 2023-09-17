use mini3d::{
    ecs::{
        api::{ecs::ExclusiveECS, ExclusiveAPI},
        instance::ExclusiveResolver,
        query::Query,
    },
    expect,
    feature::component::{common::free_fly::FreeFly, ui::ui::UI},
    math::rect::IRect,
    registry::{component::StaticComponent, error::RegistryError, system::ExclusiveSystem},
    renderer::{color::Color, SCREEN_CENTER},
};

use crate::{component::os::OS, input::CommonAction};

#[derive(Default)]
pub struct UpdateOS {
    os: StaticComponent<OS>,
    free_fly: StaticComponent<FreeFly>,
    ui: StaticComponent<UI>,
    query: Query,
}

impl UpdateOS {
    pub const NAME: &'static str = "update_os";
}

impl ExclusiveSystem for UpdateOS {
    fn setup(&mut self, resolver: &mut ExclusiveResolver) -> Result<(), RegistryError> {
        self.os = resolver.find(OS::NAME)?;
        self.free_fly = resolver.find(FreeFly::NAME)?;
        self.ui = resolver.find(UI::NAME)?;
        self.query = resolver.query().all(&[FreeFly::NAME])?.build();
        Ok(())
    }

    fn run(&self, ecs: &mut ExclusiveECS, api: &mut ExclusiveAPI) {
        let mut os = expect!(api, ecs.view_mut(self.os).singleton());

        // Toggle control mode
        if expect!(api, api.input.action(CommonAction::CHANGE_CONTROL_MODE)).is_just_pressed() {
            os.layout_active = !os.layout_active;
            let mut view = ecs.view_mut(self.free_fly);
            for e in ecs.query(self.query) {
                let free_fly = &mut view[e];
                free_fly.active = !os.layout_active;
            }
        }

        // let ui = scene.get::<UI>(UI::UID)?.unwrap();
        // for event in ui.events() {
        //     if let UIEvent::Action { profile, id } = event {
        //         println!("{:?}", id);
        //     }
        // }

        let mut uis = ecs.view_mut(self.ui);
        let ui = uis.iter_mut().next().unwrap();
        let user = ui.user("main".into()).unwrap();

        expect!(api, os.controller.update(&api.input, user));

        // Render center cross
        api.renderer.graphics().fill_rect(
            IRect::new(SCREEN_CENTER.x as i32, SCREEN_CENTER.y as i32, 2, 2),
            Color::WHITE,
        );
    }
}
