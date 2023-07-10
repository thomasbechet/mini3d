use crate::{
    context::SystemContext,
    ecs::system::SystemResult,
    feature::component::{
        canvas::Canvas,
        ui::{UIRenderTarget, UI},
    },
    registry::component::Component,
};

pub fn update(ctx: &mut SystemContext) -> SystemResult {
    let world = ctx.world.active();
    let mut uis = world.static_view_mut::<UI>(UI::UID)?;
    for ui in uis.iter() {
        ui.update(ctx.time.global())?;
        for event in ui.events() {
            println!("{:?}", event);
        }
    }
    Ok(())
}

pub fn render(ctx: &mut SystemContext) -> SystemResult {
    let world = ctx.world.active();
    let mut canvases = world.static_view_mut::<Canvas>(Canvas::UID)?;

    for e in &world.query(&[UI::UID, UIRenderTarget::UID]) {
        let ui = world.get_static_component::<UI>(e, UI::UID)?.unwrap();
        let target = world
            .get_static_component::<UIRenderTarget>(e, UIRenderTarget::UID)?
            .unwrap();
        match *target {
            UIRenderTarget::Screen { offset } => {
                ui.render(ctx.renderer.graphics(), offset, ctx.time.global())?;
            }
            UIRenderTarget::Canvas { offset, canvas } => {
                let canvas = canvases.get_mut(canvas).ok_or("Canvas entity not found")?;
                ui.render(&mut canvas.graphics, offset, ctx.time.global())?;
            }
            UIRenderTarget::Texture {
                offset: _,
                texture: _,
            } => {}
        }
    }

    Ok(())
}
