use crate::{
    ecs::{
        context::{ExclusiveContext, ParallelContext},
        system::SystemResult,
    },
    feature::component::ui::{
        canvas::Canvas,
        ui::{UIRenderTarget, UI},
    },
    registry::{
        component::{Component, ComponentId},
        error::RegistryError,
        system::{ExclusiveResolver, ExclusiveSystem, ParallelResolver, ParallelSystem},
    },
};

#[derive(Default)]
pub struct UpdateUI {
    ui: ComponentId,
}

impl ParallelSystem for UpdateUI {
    const NAME: &'static str = "update_ui";

    fn resolve(&mut self, resolver: &mut ParallelResolver) -> Result<(), RegistryError> {
        self.ui = resolver.read(UI::UID)?;
        Ok(())
    }

    fn run(&self, ctx: &mut ParallelContext) -> SystemResult {
        let mut uis = ctx.scene.view_mut(self.ui)?.as_static::<UI>()?;
        for ui in uis.iter() {
            ui.update(ctx.time.global())?;
            for event in ui.events() {
                println!("{:?}", event);
            }
        }
        Ok(())
    }
}

#[derive(Default)]
pub struct RenderUI {
    canvas: ComponentId,
    ui: ComponentId,
    target: ComponentId,
}

impl ExclusiveSystem for RenderUI {
    const NAME: &'static str = "render_ui";

    fn resolve(&mut self, resolver: &ExclusiveResolver) -> Result<(), RegistryError> {
        self.canvas = resolver.find(Canvas::UID)?;
        self.ui = resolver.find(UI::UID)?;
        self.target = resolver.find(UIRenderTarget::UID)?;
        Ok(())
    }

    fn run(&self, ctx: &mut ExclusiveContext) -> SystemResult {
        let mut canvases = ctx.scene.view_mut(self.canvas)?.as_static::<Canvas>()?;
        let uis = ctx.scene.view(self.ui)?.as_static::<UI>()?;
        let targets = ctx.scene.view(self.target)?.as_static::<UIRenderTarget>()?;

        for e in &ctx.scene.query(&[self.ui, self.target]) {
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
}
