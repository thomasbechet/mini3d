use crate::{
    ecs::{
        api::{context::Context, ecs::ECS},
        instance::ExclusiveResolver,
        query::{FilterQuery, Query},
    },
    expect,
    feature::common::local_to_world::LocalToWorld,
    registry::{component::StaticComponentType, error::RegistryError, system::ExclusiveSystem},
};

use super::{
    camera::Camera, canvas::Canvas, model::Model, static_mesh::StaticMesh, viewport::Viewport,
};

#[derive(Default)]
pub struct SynchronizeRendererResources {
    // Components
    viewport: StaticComponentType<Viewport>,
    camera: StaticComponentType<Camera>,
    canvas: StaticComponentType<Canvas>,
    static_mesh: StaticComponentType<StaticMesh>,
    local_to_world: StaticComponentType<LocalToWorld>,
    // Queries
    added_viewport: FilterQuery,
    removed_viewport: FilterQuery,
    model_query: Query,
    added_camera: FilterQuery,
    removed_camera: FilterQuery,
    camera_query: Query,
    added_static_mesh: FilterQuery,
    removed_static_mesh: FilterQuery,
    added_canvas: FilterQuery,
    removed_canvas: FilterQuery,
    scene_canvas_query: Query,
}

impl SynchronizeRendererResources {
    pub const NAME: &'static str = "synchronize_renderer_resources";
}

impl ExclusiveSystem for SynchronizeRendererResources {
    fn setup(&mut self, resolver: &mut ExclusiveResolver) -> Result<(), RegistryError> {
        self.viewport = resolver.find(Viewport::NAME)?;
        self.camera = resolver.find(Camera::NAME)?;
        self.canvas = resolver.find(Canvas::NAME)?;
        self.static_mesh = resolver.find(StaticMesh::NAME)?;
        self.local_to_world = resolver.find(LocalToWorld::NAME)?;
        self.added_viewport = resolver.query().all(&[Viewport::NAME])?.added();
        self.removed_viewport = resolver.query().all(&[Viewport::NAME])?.removed();
        self.added_camera = resolver
            .query()
            .all(&[LocalToWorld::NAME, Camera::NAME])?
            .added();
        self.removed_camera = resolver
            .query()
            .all(&[LocalToWorld::NAME, Camera::NAME])?
            .removed();
        self.camera_query = resolver
            .query()
            .all(&[LocalToWorld::NAME, Camera::NAME])?
            .build();
        self.added_static_mesh = resolver
            .query()
            .all(&[LocalToWorld::NAME, StaticMesh::NAME])?
            .added();
        self.removed_static_mesh = resolver
            .query()
            .all(&[LocalToWorld::NAME, StaticMesh::NAME])?
            .removed();
        self.model_query = resolver
            .query()
            .all(&[LocalToWorld::NAME, StaticMesh::NAME])?
            .build();
        self.added_canvas = resolver.query().all(&[Canvas::NAME])?.added();
        self.removed_canvas = resolver.query().all(&[Canvas::NAME])?.removed();
        self.scene_canvas_query = resolver
            .query()
            .all(&[LocalToWorld::NAME, Canvas::NAME])?
            .build();
        Ok(())
    }

    fn run(&self, ecs: &mut ECS, ctx: &mut Context) {
        let mut viewports = ecs.view_mut(self.viewport);
        let mut cameras = ecs.view_mut(self.camera);
        let mut static_meshes = ecs.view_mut(self.static_mesh);
        let mut canvases = ecs.view_mut(self.canvas);
        let local_to_worlds = ecs.view_mut(self.local_to_world);

        // Camera
        for e in ecs.query_filter(self.removed_camera) {
            expect!(
                ctx,
                ctx.renderer.provider.scene_camera_remove(cameras[e].handle)
            );
        }
        for e in ecs.query_filter(self.added_camera) {
            let camera = &mut cameras[e];
            camera.handle = expect!(ctx, ctx.renderer.provider.scene_camera_add());
        }
        for e in ecs.query(self.camera_query) {
            let camera = &mut cameras[e];
            let local_to_world = &local_to_worlds[e];
            expect!(
                ctx,
                ctx.renderer.provider.scene_camera_update(
                    camera.handle,
                    local_to_world.translation(),
                    local_to_world.forward(),
                    local_to_world.up(),
                    camera.fov,
                )
            );
        }
        // StaticMesh
        for e in ecs.query_filter(self.removed_static_mesh) {
            expect!(
                ctx,
                ctx.renderer
                    .provider
                    .scene_model_remove(static_meshes[e].handle)
            );
        }
        for e in ecs.query_filter(self.added_static_mesh) {
            let s = &mut static_meshes[e];
            let model = expect!(ctx, ctx.resource.read::<Model>(s.model));
            // Load mesh
            let mesh_handle = expect!(
                ctx,
                ctx.renderer.resources.request_mesh(
                    model.mesh,
                    ctx.renderer.provider.as_mut(),
                    ctx.resource
                )
            )
            .handle;
            let handle = expect!(ctx, ctx.renderer.provider.scene_model_add(mesh_handle));
            // Load material
            for (index, material) in model.materials.iter().enumerate() {
                let material_handle = expect!(
                    ctx,
                    ctx.renderer.resources.request_material(
                        *material,
                        ctx.renderer.provider.as_mut(),
                        ctx.resource
                    )
                )
                .handle;
                expect!(
                    ctx,
                    ctx.renderer
                        .provider
                        .scene_model_set_material(handle, index, material_handle)
                );
            }
            s.handle = handle;
        }
        for e in ecs.query(self.model_query) {
            let s = &static_meshes[e];
            let t = &local_to_worlds[e];
            expect!(
                ctx,
                ctx.renderer
                    .provider
                    .scene_model_transfer_matrix(s.handle, t.matrix)
            );
        }
        // Canvas
        for e in ecs.query_filter(self.removed_canvas) {
            expect!(
                ctx,
                ctx.renderer
                    .provider
                    .scene_canvas_remove(canvases[e].handle)
            );
        }
        for e in ecs.query_filter(self.added_canvas) {
            let c = &mut canvases[e];
            let t = &local_to_worlds[e];
            expect!(ctx, ctx.renderer.provider.scene_canvas_add(c.resolution));
        }
        for e in ecs.query(self.scene_canvas_query) {
            let c = &canvases[e];
            let t = &local_to_worlds[e];
            expect!(
                ctx,
                ctx.renderer
                    .provider
                    .scene_canvas_transfer_matrix(c.handle, t.matrix)
            );
        }
        // Viewport
        for e in ecs.query_filter(self.removed_viewport) {
            expect!(
                ctx,
                ctx.renderer.provider.viewport_remove(viewports[e].handle)
            );
        }
        for e in ecs.query_filter(self.added_viewport) {
            let v = &mut viewports[e];
            v.handle = expect!(ctx, ctx.renderer.provider.viewport_add(v.resolution));
            let camera = v.camera.map(|e| cameras[e].handle);
            expect!(
                ctx,
                ctx.renderer.provider.viewport_set_camera(v.handle, camera)
            );
            expect!(
                ctx,
                ctx.renderer
                    .provider
                    .viewport_set_resolution(v.handle, v.resolution)
            );
        }
    }
}
