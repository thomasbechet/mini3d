use anyhow::Result;

use crate::{context::SystemContext, feature::component::ui::UIComponent};

pub fn update(ctx: &SystemContext) -> Result<()> {
    let world = ctx.world().active();
    let uis = world.view::<UIComponent>(UIComponent::UID)?;
    for e in &world.query(&[UIComponent::UID]) {
        if uis[e].active {
            uis[e].ui.update(&ctx.input(), ctx.time().global())?;
        }
    }
    Ok(())
}

pub fn render(ctx: &SystemContext) -> Result<()> {
    // for (_, ui) in world.query::<&UIComponent>() {
    //     if ui.visible {
    //         for render_target in &ui.render_targets {
    //             match render_target {
    //                 UIRenderTarget::Screen { offset } => {
    //                     ui.ui.render(ctx.renderer.graphics(), *offset, ctx.time);
    //                 },
    //                 UIRenderTarget::Canvas { offset, canvas } => {
    //                     let canvas = world.query_one::<&mut Canvas>(*canvas).with_context(|| "Canvas entity not found")?;
    //                     ui.ui.render(&mut canvas.graphics, *offset, ctx.time);
    //                 },
    //                 UIRenderTarget::Texture { offset: _, texture: _ } => {},
    //             }
    //         }
    //     }   
    // }
    Ok(())
}