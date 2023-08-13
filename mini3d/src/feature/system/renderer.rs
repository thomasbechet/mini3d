use crate::{
    ecs::{
        context::ExclusiveContext,
        query::{FilterQueryId, QueryId},
        system::{ExclusiveResolver, SystemResult},
    },
    feature::component::{
        renderer::{camera::Camera, static_mesh::StaticMesh},
        scene::local_to_world::LocalToWorld,
        ui::{canvas::Canvas, viewport::Viewport},
    },
    registry::{
        component::{Component, StaticComponent},
        error::RegistryError,
        system::ExclusiveSystem,
    },
};

#[derive(Default)]
pub struct DespawnRendererEntities {
    // Components
    viewport: StaticComponent<Viewport>,
    camera: StaticComponent<Camera>,
    canvas: StaticComponent<Canvas>,
    static_mesh: StaticComponent<StaticMesh>,
    local_to_world: StaticComponent<LocalToWorld>,
    // Queries
    added_viewport: FilterQueryId,
    removed_viewport: FilterQueryId,
    model_query: QueryId,
    added_camera: FilterQueryId,
    removed_camera: FilterQueryId,
    camera_query: QueryId,
    added_model: FilterQueryId,
    removed_model: FilterQueryId,
    added_canvas: FilterQueryId,
    removed_canvas: FilterQueryId,
    scene_canvas_query: QueryId,
}

impl ExclusiveSystem for DespawnRendererEntities {
    const NAME: &'static str = "despawn_renderer_entities";

    fn resolve(&mut self, resolver: &mut ExclusiveResolver) -> Result<(), RegistryError> {
        self.viewport = resolver.find(Viewport::UID)?;
        self.camera = resolver.find(Camera::UID)?;
        self.canvas = resolver.find(Canvas::UID)?;
        self.static_mesh = resolver.find(StaticMesh::UID)?;
        self.local_to_world = resolver.find(LocalToWorld::UID)?;
        self.added_viewport = resolver.query().all(&[Viewport::UID])?.added();
        self.removed_viewport = resolver.query().all(&[Viewport::UID])?.removed();
        self.added_camera = resolver
            .query()
            .all(&[LocalToWorld::UID, Camera::UID])?
            .added();
        self.removed_camera = resolver
            .query()
            .all(&[LocalToWorld::UID, Camera::UID])?
            .removed();
        self.camera_query = resolver
            .query()
            .all(&[LocalToWorld::UID, Camera::UID])?
            .build();
        self.added_model = resolver
            .query()
            .all(&[LocalToWorld::UID, StaticMesh::UID])?
            .added();
        self.removed_model = resolver
            .query()
            .all(&[LocalToWorld::UID, StaticMesh::UID])?
            .removed();
        self.model_query = resolver
            .query()
            .all(&[LocalToWorld::UID, StaticMesh::UID])?
            .build();
        self.added_canvas = resolver.query().all(&[Canvas::UID])?.added();
        self.removed_canvas = resolver.query().all(&[Canvas::UID])?.removed();
        self.scene_canvas_query = resolver
            .query()
            .all(&[LocalToWorld::UID, Canvas::UID])?
            .build();
        Ok(())
    }

    fn run(&self, ctx: &mut ExclusiveContext) -> SystemResult {
        let mut viewports = ctx.scene.view_mut(self.viewport)?;
        let mut cameras = ctx.scene.view_mut(self.camera)?;
        let mut static_meshes = ctx.scene.view_mut(self.static_mesh)?;
        let mut canvases = ctx.scene.view_mut(self.canvas)?;
        let mut local_to_worlds = ctx.scene.view_mut(self.local_to_world)?;

        // Camera
        for e in ctx.scene.filter_query(self.removed_camera) {
            ctx.renderer
                .backend
                .scene_camera_remove(cameras[e].handle)?;
        }
        for e in ctx.scene.filter_query(self.added_camera) {
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
        for e in ctx.scene.query(self.camera_query) {
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
        for e in ctx.scene.filter_query(self.removed_model) {
            ctx.renderer
                .backend
                .scene_model_remove(static_meshes[e].handle)?;
        }
        for e in ctx.scene.filter_query(self.added_model) {
            let s = &mut static_meshes[e];
            let t = &mut local_to_worlds[e];
            let model = ctx.asset.read(s.model)?;
            // Load mesh
            let mesh_handle = ctx
                .renderer
                .manager
                .resources
                .request_mesh(model.mesh, ctx.renderer.backend, &ctx.asset.manager)?
                .handle;
            let handle = ctx.renderer.backend.scene_model_add(mesh_handle)?;
            // Load material
            for (index, material) in model.materials.iter().enumerate() {
                let material_handle = ctx
                    .renderer
                    .manager
                    .resources
                    .request_material(*material, ctx.renderer.backend, &ctx.asset.manager)?
                    .handle;
                ctx.renderer
                    .backend
                    .scene_model_set_material(handle, index, material_handle)?;
            }
            s.handle = handle;
        }
        for e in ctx.scene.query(self.model_query) {
            let s = &static_meshes[e];
            let t = &local_to_worlds[e];
            ctx.renderer
                .backend
                .scene_model_transfer_matrix(s.handle, t.matrix)?;
        }
        // Canvas
        for e in ctx.scene.filter_query(self.removed_canvas) {
            ctx.renderer
                .backend
                .scene_canvas_remove(canvases[e].handle)?;
        }
        for e in ctx.scene.filter_query(self.added_canvas) {
            let c = &mut canvases[e];
            let t = &local_to_worlds[e];
            ctx.renderer.backend.scene_canvas_add(c.resolution)?;
        }
        for e in ctx.scene.query(self.scene_canvas_query) {
            let c = &canvases[e];
            let t = &local_to_worlds[e];
            ctx.renderer
                .backend
                .scene_canvas_transfer_matrix(c.handle, t.matrix)?;
        }
        // Viewport
        for e in ctx.scene.filter_query(self.removed_viewport) {
            ctx.renderer.backend.viewport_remove(viewports[e].handle)?;
        }
        for e in ctx.scene.filter_query(self.added_viewport) {
            let v = &mut viewports[e];
            v.handle = ctx.renderer.backend.viewport_add(v.resolution)?;
            let camera = v.camera.map(|e| cameras[e].handle);
            ctx.renderer.backend.viewport_set_camera(v.handle, camera)?;
            ctx.renderer
                .backend
                .viewport_set_resolution(v.handle, v.resolution)?;
        }

        Ok(())
    }
}
