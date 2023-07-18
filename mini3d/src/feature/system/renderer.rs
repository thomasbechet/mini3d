use crate::{
    ecs::{context::ExclusiveContext, system::SystemResult},
    feature::component::{
        common::lifecycle::Lifecycle,
        renderer::{camera::Camera, static_mesh::StaticMesh},
        ui::{canvas::Canvas, viewport::Viewport},
    },
    registry::{
        component::{Component, ComponentId},
        error::RegistryError,
        system::{ExclusiveResolver, ExclusiveSystem},
    },
};

#[derive(Default)]
pub struct DespawnRendererEntities {
    life_cycle: ComponentId,
    viewport: ComponentId,
    camera: ComponentId,
    model: ComponentId,
    canvas: ComponentId,
    static_mesh: ComponentId,
}

impl ExclusiveSystem for DespawnRendererEntities {
    const NAME: &'static str = "despawn_renderer_entities";

    fn resolve(&mut self, resolver: &ExclusiveResolver) -> Result<(), RegistryError> {
        self.life_cycle = resolver.find(Lifecycle::UID)?;
        self.viewport = resolver.find(Viewport::UID)?;
        self.camera = resolver.find(Camera::UID)?;
        self.model = resolver.find(StaticMesh::UID)?;
        self.canvas = resolver.find(Canvas::UID)?;
        self.static_mesh = resolver.find(StaticMesh::UID)?;
        Ok(())
    }

    fn run(&self, ctx: &mut ExclusiveContext) -> SystemResult {
        let lifecycles = ctx.scene.view(self.life_cycle)?.as_static::<Lifecycle>()?;
        let viewports = ctx.scene.view(self.viewport)?.as_static::<Viewport>()?;
        let cameras = ctx.scene.view(self.camera)?.as_static::<Camera>()?;
        let models = ctx.scene.view(self.model)?.as_static::<StaticMesh>()?;
        let canvases = ctx.scene.view(self.canvas)?.as_static::<Canvas>()?;

        for e in &ctx.scene.query(&[self.life_cycle, self.viewport]) {
            if !lifecycles[e].alive {
                if let Some(handle) = viewports[e].handle {
                    ctx.renderer.manager.viewports_removed.insert(handle);
                }
            }
        }

        for e in &ctx.scene.query(&[self.life_cycle, self.camera]) {
            if !lifecycles[e].alive {
                if let Some(handle) = cameras[e].handle {
                    ctx.renderer.manager.scene_cameras_removed.insert(handle);
                }
            }
        }

        for e in &ctx.scene.query(&[self.life_cycle, self.static_mesh]) {
            if !lifecycles[e].alive {
                if let Some(handle) = models[e].handle {
                    ctx.renderer.manager.scene_models_removed.insert(handle);
                }
            }
        }

        for e in &ctx.scene.query(&[self.life_cycle, self.canvas]) {
            if !lifecycles[e].alive {
                if let Some(handle) = canvases[e].handle {
                    ctx.renderer.manager.scene_canvases_removed.insert(handle);
                }
            }
        }

        Ok(())
    }
}
