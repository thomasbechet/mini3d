use crate::{
    ecs::{
        context::ExclusiveContext,
        query::QueryId,
        system::{ExclusiveResolver, SystemResult},
    },
    feature::component::{
        common::lifecycle::Lifecycle,
        renderer::{camera::Camera, static_mesh::StaticMesh},
        ui::{canvas::Canvas, viewport::Viewport},
    },
    registry::{
        component::{Component, ComponentId},
        error::RegistryError,
        system::ExclusiveSystem,
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
    viewport_query: QueryId,
    camera_query: QueryId,
    model_query: QueryId,
    canvas_query: QueryId,
}

impl ExclusiveSystem for DespawnRendererEntities {
    const NAME: &'static str = "despawn_renderer_entities";

    fn resolve(&mut self, resolver: &mut ExclusiveResolver) -> Result<(), RegistryError> {
        self.life_cycle = resolver.find(Lifecycle::UID)?;
        self.viewport = resolver.find(Viewport::UID)?;
        self.camera = resolver.find(Camera::UID)?;
        self.model = resolver.find(StaticMesh::UID)?;
        self.canvas = resolver.find(Canvas::UID)?;
        self.static_mesh = resolver.find(StaticMesh::UID)?;
        self.viewport_query = resolver
            .query()
            .all(&[self.life_cycle, self.viewport])
            .build();
        self.camera_query = resolver
            .query()
            .all(&[self.life_cycle, self.camera])
            .build();
        self.model_query = resolver.query().all(&[self.life_cycle, self.model]).build();
        self.canvas_query = resolver
            .query()
            .all(&[self.life_cycle, self.canvas])
            .build();
        Ok(())
    }

    fn run(&self, ctx: &mut ExclusiveContext) -> SystemResult {
        let lifecycles = ctx.scene.view(self.life_cycle)?.as_static::<Lifecycle>()?;
        let viewports = ctx.scene.view(self.viewport)?.as_static::<Viewport>()?;
        let cameras = ctx.scene.view(self.camera)?.as_static::<Camera>()?;
        let models = ctx.scene.view(self.model)?.as_static::<StaticMesh>()?;
        let canvases = ctx.scene.view(self.canvas)?.as_static::<Canvas>()?;

        for e in ctx.scene.query(self.viewport_query) {
            if !lifecycles[e].alive {
                if let Some(handle) = viewports[e].handle {
                    ctx.renderer.manager.viewports_removed.insert(handle);
                }
            }
        }

        for e in ctx.scene.query(self.camera_query) {
            if !lifecycles[e].alive {
                if let Some(handle) = cameras[e].handle {
                    ctx.renderer.manager.scene_cameras_removed.insert(handle);
                }
            }
        }

        for e in ctx.scene.query(self.model_query) {
            if !lifecycles[e].alive {
                if let Some(handle) = models[e].handle {
                    ctx.renderer.manager.scene_models_removed.insert(handle);
                }
            }
        }

        for e in ctx.scene.query(self.canvas_query) {
            if !lifecycles[e].alive {
                if let Some(handle) = canvases[e].handle {
                    ctx.renderer.manager.scene_canvases_removed.insert(handle);
                }
            }
        }

        Ok(())
    }
}
