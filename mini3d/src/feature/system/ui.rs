use anyhow::{Result, Context};

use crate::{context::SystemContext, feature::component::{ui::{UIComponent, UIRenderTarget}, canvas::Canvas}};

pub fn update(ctx: &mut SystemContext) -> Result<()> {
    let world = ctx.world.active();
    let mut uis = world.view_mut::<UIComponent>(UIComponent::UID)?;
    for e in &world.query(&[UIComponent::UID]) {
        if uis[e].active {
            uis[e].ui.update(&ctx.input, ctx.time.global())?;
        }
    }
    Ok(())
}

pub fn render(ctx: &mut SystemContext) -> Result<()> {

    let world = ctx.world.active();
    let mut canvases = world.view_mut::<Canvas>(Canvas::UID)?;
    for ui in world.view_mut::<UIComponent>(UIComponent::UID)?.iter() {
        if ui.visible {
            for render_target in &ui.render_targets {
                match render_target {
                    UIRenderTarget::Screen { offset } => {
                        ui.ui.render(ctx.renderer.graphics(), *offset, ctx.time.global());
                    },
                    UIRenderTarget::Canvas { offset, canvas } => {
                        let canvas = canvases.get_mut(*canvas).with_context(|| "Canvas entity not found")?;
                        ui.ui.render(&mut canvas.graphics, *offset, ctx.time.global());
                    },
                    UIRenderTarget::Texture { offset: _, texture: _ } => {},
                }
            }
        }
    }

    Ok(())
}