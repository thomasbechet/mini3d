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
        scene::local_to_world::LocalToWorld,
        ui::{canvas::Canvas, viewport::Viewport},
    },
    registry::{component::Component, error::RegistryError, system::ExclusiveSystem},
};

#[derive(Default)]
pub struct DespawnRendererEntities {
    viewport: StaticComponent<Viewport>,
    camera: StaticComponent<Camera>,
    canvas: StaticComponent<Canvas>,
    static_mesh: StaticComponent<StaticMesh>,
    local_to_world: StaticComponent<LocalToWorld>,
    added_viewport: QueryId,
    removed_viewport: QueryId,
    added_camera: QueryId,
    removed_camera: QueryId,
    added_model: QueryId,
    removed_model: QueryId,
    added_canvas: QueryId,
    removed_canvas: QueryId,
}

impl ExclusiveSystem for DespawnRendererEntities {
    const NAME: &'static str = "despawn_renderer_entities";

    fn resolve(&mut self, resolver: &mut ExclusiveResolver) -> Result<(), RegistryError> {
        self.viewport = resolver.find(Viewport::UID)?;
        self.camera = resolver.find(Camera::UID)?;
        self.model = resolver.find(Model::UID)?;
        self.canvas = resolver.find(Canvas::UID)?;
        self.static_mesh = resolver.find(StaticMesh::UID)?;
        self.viewport_query = resolver.query().all(&[Viewport::UID])?.build();
        self.camera_query = resolver
            .query()
            .all(&[LocalToWorld::UID, Camera::UID])?
            .build();
        self.model_query = resolver
            .query()
            .all(&[LocalToWorld::UID, StaticMesh::UID])?
            .build();
        self.canvas_query = resolver.query().all(&[Canvas::UID])?.build();
        Ok(())
    }

    fn run(&self, ctx: &mut ExclusiveContext) -> SystemResult {
        let viewports = ctx.scene.view_mut(self.viewport)?;
        let cameras = ctx.scene.view_mut(self.camera)?;
        let static_meshes = ctx.scene.view_mut(self.static_mesh)?;
        let canvases = ctx.scene.view_mut(self.canvas)?;
        let local_to_worlds = ctx.scene.view_mut(self.local_to_world)?;

        // Camera
        for e in ctx.scene.query(self.removed_camera) {
            ctx.renderer
                .backend
                .scene_camera_remove(cameras[e].handle)?;
        }
        for e in ctx.scene.query(self.added_camera) {
            let camera = &mut cameras[e];
            camera.handle = ctx.renderer.backend.scene_camera_add()?;
            let local_to_world = &local_to_worlds[e];
            ctx.renderer.backend.scene_camera_update(
                camera.handle,
                local_to_world.translation(),
                local_to_world.forward(),
                local_to_world.up(),
                camera.fov,
            )?;
        }
        // Model
        for e in ctx.scene.query(self.removed_model) {
            ctx.renderer
                .backend
                .model_camera_remove(static_meshes[e].handle)?;
        }
        // Canvas
        for e in ctx.scene.query(self.removed_canvas) {
            ctx.renderer
                .backend
                .scene_canvas_remove(canvases[e].handle)?;
        }
        // Viewport
        for e in ctx.scene.query(self.removed_viewport) {
            ctx.renderer.backend.viewport_remove(viewports[e].handle)?;
        }

        Ok(())
    }
}
