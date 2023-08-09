use crate::{
    ecs::{
        component::StaticComponent,
        context::ExclusiveContext,
        query::QueryId,
        system::{ExclusiveResolver, SystemResult},
    },
    feature::component::{
        common::lifecycle::Lifecycle,
        renderer::{camera::Camera, model::Model, static_mesh::StaticMesh},
        ui::{canvas::Canvas, viewport::Viewport},
    },
    registry::{component::Component, error::RegistryError, system::ExclusiveSystem},
};

#[derive(Default)]
pub struct DespawnRendererEntities {
    life_cycle: StaticComponent<Lifecycle>,
    viewport: StaticComponent<Viewport>,
    camera: StaticComponent<Camera>,
    model: StaticComponent<Model>,
    canvas: StaticComponent<Canvas>,
    static_mesh: StaticComponent<StaticMesh>,
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
            .all(&[Lifecycle::UID, Viewport::UID])?
            .build();
        self.camera_query = resolver
            .query()
            .all(&[Lifecycle::UID, Camera::UID])?
            .build();
        self.model_query = resolver
            .query()
            .all(&[Lifecycle::UID, StaticMesh::UID])?
            .build();
        self.canvas_query = resolver
            .query()
            .all(&[Lifecycle::UID, Canvas::UID])?
            .build();
        Ok(())
    }

    fn run(&self, ctx: &mut ExclusiveContext) -> SystemResult {
        let lifecycles = ctx.scene.view(self.life_cycle)?;
        let viewports = ctx.scene.view(self.viewport)?;
        let cameras = ctx.scene.view(self.camera)?;
        let static_meshes = ctx.scene.view(self.static_mesh)?;
        let canvases = ctx.scene.view(self.canvas)?;

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
                if let Some(handle) = static_meshes[e].handle {
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
