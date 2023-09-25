use crate::{
    ecs::{
        api::{ecs::ExclusiveECS, ExclusiveAPI},
        instance::ExclusiveResolver,
        query::Query,
    },
    expect,
    registry::{component::StaticComponent, error::RegistryError, system::ExclusiveSystem},
};

use super::{
    canvas::Canvas,
    ui::{UIRenderTarget, UI},
};

#[derive(Default)]
pub struct RenderUI {
    canvas: StaticComponent<Canvas>,
    ui: StaticComponent<UI>,
    target: StaticComponent<UIRenderTarget>,
    query: Query,
}

impl RenderUI {
    pub const NAME: &'static str = "render_ui";
}

impl ExclusiveSystem for RenderUI {
    fn setup(&mut self, resolver: &mut ExclusiveResolver) -> Result<(), RegistryError> {
        self.canvas = resolver.find(Canvas::NAME)?;
        self.ui = resolver.find(UI::NAME)?;
        self.target = resolver.find(UIRenderTarget::NAME)?;
        self.query = resolver
            .query()
            .all(&[UI::NAME, UIRenderTarget::NAME])?
            .build();
        Ok(())
    }

    fn run(&self, ecs: &mut ExclusiveECS, api: &mut ExclusiveAPI) {
        let mut canvases = ecs.view_mut(self.canvas);
        let uis = ecs.view(self.ui);
        let targets = ecs.view(self.target);

        for e in ecs.query(self.query) {
            let ui = &uis[e];
            let target = &targets[e];
            match *target {
                UIRenderTarget::Screen { offset } => {
                    expect!(
                        api,
                        ui.render(api.renderer.graphics(), offset, api.time.global())
                    );
                }
                UIRenderTarget::Canvas { offset, canvas } => {
                    let canvas = expect!(api, canvases.get_mut(canvas), "Canvas entity not found");
                    expect!(
                        api,
                        ui.render(&mut canvas.graphics, offset, api.time.global())
                    );
                }
                UIRenderTarget::Texture {
                    offset: _,
                    texture: _,
                } => {}
            }
        }
    }
}
