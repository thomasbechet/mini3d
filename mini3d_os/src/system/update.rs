use mini3d::{
    ecs::{
        api::ExclusiveAPI,
        system::{ExclusiveResolver, SystemResult},
    },
    feature::component::{common::free_fly::FreeFly, ui::ui::UI},
    math::rect::IRect,
    registry::{
        component::{Component, StaticComponent},
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
}

impl ExclusiveSystem for UpdateOS {
    const NAME: &'static str = "update_os";

    fn setup(&mut self, resolver: &mut ExclusiveResolver) -> Result<(), RegistryError> {
        self.os = resolver.find(OS::UID)?;
        self.free_fly = resolver.find(FreeFly::UID)?;
        self.ui = resolver.find(UI::UID)?;
        Ok(())
    }

    fn run(&self, ctx: &mut ExclusiveAPI) -> SystemResult {
        let scene = ctx.ecs.active();
        let mut os = scene.get_singleton_mut::<OS>(OS::UID)?.unwrap();

        // Toggle control mode
        if ctx
            .input
            .action(CommonAction::CHANGE_CONTROL_MODE.into())?
            .is_just_pressed()
        {
            os.layout_active = !os.layout_active;
            let mut view = scene.static_view_mut::<FreeFly>(FreeFly::UID)?;
            for e in &scene.query(&[FreeFly::UID]) {
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

        let mut view = scene.static_view_mut::<UI>(UI::UID)?;
        let ui = view.iter().next().unwrap();
        let user = ui.user("main".into())?;

        os.controller.update(&ctx.input, user)?;

        // Render center cross
        ctx.renderer.graphics().fill_rect(
            IRect::new(SCREEN_CENTER.x as i32, SCREEN_CENTER.y as i32, 2, 2),
            Color::WHITE,
        );

        Ok(())
    }
}
