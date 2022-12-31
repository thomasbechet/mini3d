use anyhow::{Result, Context};
use hecs::World;

use crate::{scene::SystemContext, feature::component::{ui::{UIComponent, UIRenderTarget}, canvas::CanvasComponent}};

pub fn update_and_render(ctx: &mut SystemContext, world: &mut World) -> Result<()> {
    
    // Update UI
    for (_, ui) in world.query_mut::<&mut UIComponent>() {
        if ui.active {
            ui.ui.update(ctx.input, ctx.time)?;
        }
        
    }

    // Render UI
    for (_, ui) in world.query::<&UIComponent>().iter() {
        if ui.visible {
            for render_target in &ui.render_targets {
                match render_target {
                    UIRenderTarget::Screen { offset } => {
                        ui.ui.render(ctx.renderer.graphics(), *offset);
                    },
                    UIRenderTarget::Canvas { offset, canvas } => {
                        let mut entity = world.query_one::<&mut CanvasComponent>(*canvas).with_context(|| "Canvas entity not found")?;
                        let canvas = entity.get().unwrap();
                        ui.ui.render(&mut canvas.graphics, *offset);
                    },
                    UIRenderTarget::Texture { offset: _, texture: _ } => {},
                }
            }
        }   
    }

    Ok(())
}