use mini3d::{
    ecs::{
        api::{ecs::ExclusiveECS, ExclusiveAPI},
        instance::{ExclusiveResolver, SystemResult},
        query::Query,
    },
    feature::component::{common::free_fly::FreeFly, ui::ui::UI},
    math::rect::IRect,
    registry::{
        component::{ComponentData, StaticComponent},
        error::RegistryError,
        system::ExclusiveSystem,
    },
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

impl ExclusiveSystem for UpdateOS {
    const NAME: &'static str = "update_os";

    fn setup(&mut self, resolver: &mut ExclusiveResolver) -> Result<(), RegistryError> {
        self.os = resolver.find(OS::UID)?;
        self.free_fly = resolver.find(FreeFly::UID)?;
        self.ui = resolver.find(UI::UID)?;
        self.query = resolver.query().all(&[FreeFly::UID])?.build();
        Ok(())
    }

    fn run(&self, ecs: &mut ExclusiveECS, api: &mut ExclusiveAPI) -> SystemResult {
        let mut os = ecs.view_mut(self.os)?.singleton().unwrap();

        // Toggle control mode
        if api
            .input
            .action(CommonAction::CHANGE_CONTROL_MODE.into())?
            .is_just_pressed()
        {
            os.layout_active = !os.layout_active;
            let mut view = ecs.view_mut(self.free_fly)?;
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

        let mut uis = ecs.view_mut(self.ui)?;
        let ui = uis.iter_mut().next().unwrap();
        let user = ui.user("main".into())?;

        os.controller.update(&api.input, user)?;

        // Render center cross
        api.renderer.graphics().fill_rect(
            IRect::new(SCREEN_CENTER.x as i32, SCREEN_CENTER.y as i32, 2, 2),
            Color::WHITE,
        );

        Ok(())
    }
}
