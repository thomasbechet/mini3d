use crate::{
    context::ExclusiveSystemContext,
    ecs::system::SystemResult,
    feature::component::ui::{
        canvas::Canvas,
        ui::{UIRenderTarget, UI},
    },
    registry::component::Component,
};

pub fn update(ctx: &mut ExclusiveSystemContext) -> SystemResult {
    let scene = ctx.scene.active();
    let mut uis = scene.static_view_mut::<UI>(UI::UID)?;
    for ui in uis.iter() {
        ui.update(ctx.time.global())?;
        for event in ui.events() {
            println!("{:?}", event);
        }
    }
    Ok(())
}

pub fn render(ctx: &mut ExclusiveSystemContext) -> SystemResult {
    let scene = ctx.scene.active();
    let mut canvases = scene.static_view_mut::<Canvas>(Canvas::UID)?;
    let uis = scene.static_view::<UI>(UI::UID)?;
    let targets = scene.static_view::<UIRenderTarget>(UIRenderTarget::UID)?;

    for e in &scene.query(&[UI::UID, UIRenderTarget::UID]) {
        let ui = &uis[e];
        let target = &targets[e];
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
