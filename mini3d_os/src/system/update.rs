use mini3d::{context::SystemContext, anyhow::Result, math::rect::IRect, renderer::{SCREEN_CENTER, color::Color}, feature::component::{free_fly::FreeFly, ui::UI}, ui::event::{UIEvent, Direction}, glam::{Vec2, IVec2}};

use crate::{input::CommonAction, component::os::OS};

pub fn update(ctx: &mut SystemContext) -> Result<()> {
    
    let world = ctx.world.active();
    let mut os = world.get_singleton_mut::<OS>(OS::UID)?.unwrap(); 

    // Toggle control mode
    if ctx.input.action(CommonAction::CHANGE_CONTROL_MODE.into())?.is_just_pressed() {
        os.layout_active = !os.layout_active;
        let mut view = world.view_mut::<FreeFly>(FreeFly::UID)?;
        for e in &world.query(&[FreeFly::UID]) {
            let free_fly = view.get_mut(e).unwrap();
            free_fly.active = !os.layout_active;
        }
    }

    // let ui = world.get::<UI>(UI::UID)?.unwrap();
    // for event in ui.events() {
    //     if let UIEvent::Action { profile, id } = event {
    //         println!("{:?}", id);
    //     }
    // }

    let mut view = world.view_mut::<UI>(UI::UID)?;
    let ui = view.iter().next().unwrap();
    let user = ui.user("main".into())?;

    os.controller.update(&ctx.input, user)?;

    // Render center cross
    ctx.renderer.graphics().fill_rect(IRect::new(SCREEN_CENTER.x as i32, SCREEN_CENTER.y as i32, 2, 2), Color::WHITE);

    Ok(())
}