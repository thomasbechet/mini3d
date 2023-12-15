use mini3d_core::{
    ecs::{
        api::{context::Context, ecs::ECS, input::Input, renderer::Renderer},
        instance::ExclusiveResolver,
        query::Query,
    },
    expect,
    feature::common::free_fly::FreeFly,
    math::rect::IRect,
    registry::{component::StaticComponentType, error::RegistryError, system::ExclusiveSystem},
    renderer::{color::Color, SCREEN_CENTER},
};

use crate::{component::os::OS, input::CommonAction};

#[derive(Default)]
pub struct OSUpdate {
    os: StaticComponentType<OS>,
    free_fly: StaticComponentType<FreeFly>,
    // ui: StaticComponent<UI>,
    query: Query,
}

impl OSUpdate {
    pub const NAME: &'static str = "update_os";
}

impl ExclusiveSystem for OSUpdate {
    fn setup(&mut self, resolver: &mut ExclusiveResolver) -> Result<(), RegistryError> {
        self.os = resolver.find(OS::NAME)?;
        self.free_fly = resolver.find(FreeFly::NAME)?;
        // self.ui = resolver.find(UI::NAME)?;
        self.query = resolver.query().all(&[FreeFly::NAME])?.build();
        Ok(())
    }

    fn run(&self, ecs: &mut ECS, ctx: &mut Context) {
        let mut os = expect!(ctx, ecs.view_mut(self.os).singleton());

        // Toggle control mode
        if expect!(
            ctx,
            Input::action(
                ctx,
                Input::find_action(ctx, CommonAction::CHANGE_CONTROL_MODE).unwrap()
            )
        )
        .is_just_pressed()
        {
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

        // let mut uis = ecs.view_mut(self.ui);
        // let ui = uis.iter_mut().next().unwrap();
        // let user = ui.user("main".into()).unwrap();

        // expect!(ctx, os.controller.update(ctx.input, user));

        // Render center cross
        Renderer::graphics(ctx).fill_rect(
            IRect::new(SCREEN_CENTER.x as i32, SCREEN_CENTER.y as i32, 2, 2),
            Color::WHITE,
        );
    }
}
