use crate::{
    ecs::{
        api::{
            ecs::{ExclusiveECS, ParallelECS},
            ExclusiveAPI, ParallelAPI,
        },
        query::QueryId,
        system::{ExclusiveResolver, ParallelResolver, SystemResult},
    },
    feature::component::ui::{
        canvas::Canvas,
        ui::{UIRenderTarget, UI},
    },
    registry::{
        component::{Component, StaticComponent},
        error::RegistryError,
        system::{ExclusiveSystem, ParallelSystem},
    },
};

#[derive(Default)]
pub struct UpdateUI {
    ui: StaticComponent<UI>,
    query: QueryId,
}

impl ParallelSystem for UpdateUI {
    const NAME: &'static str = "update_ui";

    fn setup(&mut self, resolver: &mut ParallelResolver) -> Result<(), RegistryError> {
        self.ui = resolver.write(UI::UID)?;
        self.query = resolver.query().all(&[UI::UID])?.build();
        Ok(())
    }

    fn run(&self, ecs: &mut ParallelECS, api: &mut ParallelAPI) -> SystemResult {
        let mut uis = ecs.view_mut(self.ui)?;
        for e in ecs.query(self.query) {
            let ui = &mut uis[e];
            ui.update(api.time.global())?;
            for event in ui.events() {
                println!("{:?}", event);
            }
        }
        Ok(())
    }
}

#[derive(Default)]
pub struct RenderUI {
    canvas: StaticComponent<Canvas>,
    ui: StaticComponent<UI>,
    target: StaticComponent<UIRenderTarget>,
    query: QueryId,
}

impl ExclusiveSystem for RenderUI {
    const NAME: &'static str = "render_ui";

    fn setup(&mut self, resolver: &mut ExclusiveResolver) -> Result<(), RegistryError> {
        self.canvas = resolver.find(Canvas::UID)?;
        self.ui = resolver.find(UI::UID)?;
        self.target = resolver.find(UIRenderTarget::UID)?;
        self.query = resolver
            .query()
            .all(&[UI::UID, UIRenderTarget::UID])?
            .build();
        Ok(())
    }

    fn run(&self, ecs: &mut ExclusiveECS, api: &mut ExclusiveAPI) -> SystemResult {
        let mut canvases = ecs.view_mut(self.canvas)?;
        let uis = ecs.view(self.ui)?;
        let targets = ecs.view(self.target)?;

        for e in ecs.query(self.query) {
            let ui = &uis[e];
            let target = &targets[e];
            match *target {
                UIRenderTarget::Screen { offset } => {
                    ui.render(api.renderer.graphics(), offset, api.time.global())?;
                }
                UIRenderTarget::Canvas { offset, canvas } => {
                    let canvas = canvases.get_mut(canvas).ok_or("Canvas entity not found")?;
                    ui.render(&mut canvas.graphics, offset, api.time.global())?;
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
