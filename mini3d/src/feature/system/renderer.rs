use crate::{
    ecs::{
        api::{ecs::ExclusiveECS, ExclusiveAPI},
        instance::{ExclusiveResolver, SystemResult},
        query::{FilterQuery, Query},
    },
    feature::component::{
        renderer::{camera::Camera, static_mesh::StaticMesh, viewport::Viewport},
        scene::local_to_world::LocalToWorld,
        ui::canvas::Canvas,
    },
    registry::{
        component::{ComponentData, StaticComponent},
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
    added_viewport: FilterQuery,
    removed_viewport: FilterQuery,
    model_query: Query,
    added_camera: FilterQuery,
    removed_camera: FilterQuery,
    camera_query: Query,
    added_model: FilterQuery,
    removed_model: FilterQuery,
    added_canvas: FilterQuery,
    removed_canvas: FilterQuery,
    scene_canvas_query: Query,
}

impl ExclusiveSystem for DespawnRendererEntities {
    const NAME: &'static str = "despawn_renderer_entities";

    fn setup(&mut self, resolver: &mut ExclusiveResolver) -> Result<(), RegistryError> {
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

    fn run(&self, ecs: &mut ExclusiveECS, api: &mut ExclusiveAPI) -> SystemResult {
        let mut viewports = ecs.view_mut(self.viewport)?;
        let mut cameras = ecs.view_mut(self.camera)?;
        let mut static_meshes = ecs.view_mut(self.static_mesh)?;
        let mut canvases = ecs.view_mut(self.canvas)?;
        let mut local_to_worlds = ecs.view_mut(self.local_to_world)?;

        // Camera
        for e in ecs.filter_query(self.removed_camera) {
            api.renderer.server.scene_camera_remove(cameras[e].handle)?;
        }
        for e in ecs.filter_query(self.added_camera) {
            let camera = &mut cameras[e];
            camera.handle = api.renderer.server.scene_camera_add()?;
            let local_to_world = &local_to_worlds[e];
            api.renderer.server.scene_camera_update(
                camera.handle,
                local_to_world.translation(),
                local_to_world.forward(),
                local_to_world.up(),
                camera.fov,
            )?;
        }
        for e in ecs.query(self.camera_query) {
            let camera = &mut cameras[e];
            camera.handle = api.renderer.server.scene_camera_add()?;
            let local_to_world = &local_to_worlds[e];
            api.renderer.server.scene_camera_update(
                camera.handle,
                local_to_world.translation(),
                local_to_world.forward(),
                local_to_world.up(),
                camera.fov,
            )?;
        }
        // Model
        for e in ecs.filter_query(self.removed_model) {
            api.renderer
                .server
                .scene_model_remove(static_meshes[e].handle)?;
        }
        for e in ecs.filter_query(self.added_model) {
            let s = &mut static_meshes[e];
            let t = &mut local_to_worlds[e];
            let model = api.asset.read(s.model)?;
            // Load mesh
            let mesh_handle = api
                .renderer
                .manager
                .resources
                .request_mesh(model.mesh, api.renderer.server, api.asset.manager)?
                .handle;
            let handle = api.renderer.server.scene_model_add(mesh_handle)?;
            // Load material
            for (index, material) in model.materials.iter().enumerate() {
                let material_handle = api
                    .renderer
                    .manager
                    .resources
                    .request_material(*material, api.renderer.server, api.asset.manager)?
                    .handle;
                api.renderer
                    .server
                    .scene_model_set_material(handle, index, material_handle)?;
            }
            s.handle = handle;
        }
        for e in ecs.query(self.model_query) {
            let s = &static_meshes[e];
            let t = &local_to_worlds[e];
            api.renderer
                .server
                .scene_model_transfer_matrix(s.handle, t.matrix)?;
        }
        // Canvas
        for e in ecs.filter_query(self.removed_canvas) {
            api.renderer
                .server
                .scene_canvas_remove(canvases[e].handle)?;
        }
        for e in ecs.filter_query(self.added_canvas) {
            let c = &mut canvases[e];
            let t = &local_to_worlds[e];
            api.renderer.server.scene_canvas_add(c.resolution)?;
        }
        for e in ecs.query(self.scene_canvas_query) {
            let c = &canvases[e];
            let t = &local_to_worlds[e];
            api.renderer
                .server
                .scene_canvas_transfer_matrix(c.handle, t.matrix)?;
        }
        // Viewport
        for e in ecs.filter_query(self.removed_viewport) {
            api.renderer.server.viewport_remove(viewports[e].handle)?;
        }
        for e in ecs.filter_query(self.added_viewport) {
            let v = &mut viewports[e];
            v.handle = api.renderer.server.viewport_add(v.resolution)?;
            let camera = v.camera.map(|e| cameras[e].handle);
            api.renderer.server.viewport_set_camera(v.handle, camera)?;
            api.renderer
                .server
                .viewport_set_resolution(v.handle, v.resolution)?;
        }

        Ok(())
    }
}
