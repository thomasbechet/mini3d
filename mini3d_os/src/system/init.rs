use mini3d::{context::SystemContext, anyhow::Result, feature::{asset::{font::Font, input_table::{InputTable, InputAction, InputAxis, InputAxisRange}, material::Material, model::Model, mesh::Mesh, rhai_script::RhaiScript, texture::Texture, system_group::{SystemGroup, SystemPipeline}}, component::{lifecycle::Lifecycle, transform::Transform, local_to_world::LocalToWorld, rotator::Rotator, static_mesh::StaticMesh, free_fly::FreeFly, script_storage::ScriptStorage, rhai_scripts::RhaiScripts, hierarchy::Hierarchy, camera::Camera, viewport::Viewport, ui::{UIComponent, UIRenderTarget}}}, renderer::{SCREEN_WIDTH, SCREEN_HEIGHT, SCREEN_RESOLUTION}, ecs::procedure::Procedure, glam::{Vec3, Quat, IVec2}, event::asset::ImportAssetEvent, rand, ui::{UI, checkbox::Checkbox, interaction_layout::InteractionInputs, self}, uid::UID};

use crate::{input::{CommonAction, CommonAxis}, asset::DefaultAsset, component::os::OS};

fn define_features(ctx: &mut SystemContext) -> Result<()> {
    ctx.registry.define_static_component::<OS>(OS::NAME)?;
    ctx.registry.define_static_system("update", crate::system::update::update)?;
    Ok(())
}

fn setup_assets(ctx: &mut SystemContext) -> Result<()> {
    ctx.asset.add_bundle(DefaultAsset::BUNDLE).unwrap();
    let default_bundle = DefaultAsset::BUNDLE.into();

    // Register default font
    ctx.asset.add(Font::UID, "default", default_bundle, Font::default())?;

    // Register input tables
    ctx.input.add_table(&InputTable {
        name: "common".to_string(),
        display_name: "Common Inputs".to_string(),
        description: "".to_string(), 
        actions: Vec::from([
            InputAction {
                name: CommonAction::CLICK.to_string(),
                display_name: "Click".to_string(),
                description: "UI interaction layout (click).".to_string(),
                default_pressed: false,
            },
            InputAction {
                name: CommonAction::UP.to_string(),
                display_name: "Up".to_string(),
                description: "UI interaction layout (go up).".to_string(),
                default_pressed: false,
            },
            InputAction {
                name: CommonAction::LEFT.to_string(),
                display_name: "Left".to_string(),
                description: "UI interaction layout (go left).".to_string(),
                default_pressed: false,
            },
            InputAction {
                name: CommonAction::DOWN.to_string(),
                display_name: "Down".to_string(),
                description: "UI interaction layout (go down).".to_string(),
                default_pressed: false,
            },
            InputAction {
                name: CommonAction::RIGHT.to_string(),
                display_name: "Right".to_string(),
                description: "UI interaction layout (go right).".to_string(),
                default_pressed: false,
            },
            InputAction {
                name: CommonAction::CHANGE_CONTROL_MODE.to_string(),
                display_name: "Change Control Mode".to_string(),
                description: "Switch between selection and cursor control mode.".to_string(),
                default_pressed: false,
            },
            InputAction {
                name: CommonAction::TOGGLE_PROFILER.to_string(),
                display_name: "Toggle Profiler".to_string(),
                description: "Show or hide the profiler.".to_string(),
                default_pressed: false,
            },
        ]),
        axis: Vec::from([
            InputAxis {
                name: CommonAxis::CURSOR_X.to_string(),
                display_name: "Cursor X".to_string(),
                description: "Horizontal position of the mouse cursor relative to the screen.".to_string(),
                range: InputAxisRange::Clamped { min: 0.0, max: SCREEN_WIDTH as f32 },
                default_value: 0.0,
            },
            InputAxis {
                name: CommonAxis::CURSOR_Y.to_string(),
                display_name: "Cursor Y".to_string(),
                description: "Vertical position of the mouse cursor relative to the screen.".to_string(),
                range: InputAxisRange::Clamped { min: 0.0, max: SCREEN_HEIGHT as f32 },
                default_value: 0.0,
            },
            InputAxis {
                name: CommonAxis::SCROLL_MOTION.to_string(),
                display_name: "Scroll Motion".to_string(),
                description: "Delta scrolling value.".to_string(),
                range: InputAxisRange::Infinite,
                default_value: 0.0,
            },
            InputAxis {
                name: CommonAxis::CURSOR_MOTION_X.to_string(),
                display_name: "Cursor Motion X".to_string(),
                description: "Delta mouvement of the mouse on the horizontal axis.".to_string(),
                range: InputAxisRange::Infinite,
                default_value: 0.0,
            },
            InputAxis {
                name: CommonAxis::CURSOR_MOTION_Y.to_string(),
                display_name: "Cursor Motion Y".to_string(),
                description: "Delta mouvement of the mouse on the vertical axis.".to_string(),
                range: InputAxisRange::Infinite,
                default_value: 0.0,
            },
            InputAxis {
                name: CommonAxis::VIEW_X.to_string(),
                display_name: "View X".to_string(),
                description: "View horizontal delta movement.".to_string(),
                range: InputAxisRange::Infinite,
                default_value: 0.0,
            },
            InputAxis {
                name: CommonAxis::VIEW_Y.to_string(),
                display_name: "View Y".to_string(),
                description: "View vertical delta movement.".to_string(),
                range: InputAxisRange::Infinite,
                default_value: 0.0,
            },
            InputAxis {
                name: CommonAxis::MOVE_FORWARD.to_string(),
                display_name: "Move Forward".to_string(), 
                description: "".to_string(), 
                range: InputAxisRange::Clamped { min: 0.0, max: 1.0 }, 
                default_value: 0.0,
            },
            InputAxis {
                name: CommonAxis::MOVE_BACKWARD.to_string(),
                display_name: "Move Backward".to_string(), 
                description: "".to_string(), 
                range: InputAxisRange::Clamped { min: 0.0, max: 1.0 }, 
                default_value: 0.0,
            },
            InputAxis {
                name: CommonAxis::MOVE_LEFT.to_string(),
                display_name: "Move Left".to_string(), 
                description: "".to_string(), 
                range: InputAxisRange::Clamped { min: 0.0, max: 1.0 }, 
                default_value: 0.0,
            },
            InputAxis {
                name: CommonAxis::MOVE_RIGHT.to_string(),
                display_name: "Move Right".to_string(), 
                description: "".to_string(), 
                range: InputAxisRange::Clamped { min: 0.0, max: 1.0 }, 
                default_value: 0.0,
            },
            InputAxis {
                name: CommonAxis::MOVE_UP.to_string(),
                display_name: "Move Up".to_string(), 
                description: "".to_string(), 
                range: InputAxisRange::Clamped { min: 0.0, max: 1.0 }, 
                default_value: 0.0,
            },
            InputAxis {
                name: CommonAxis::MOVE_DOWN.to_string(),
                display_name: "Move Down".to_string(), 
                description: "".to_string(), 
                range: InputAxisRange::Clamped { min: 0.0, max: 1.0 },
                default_value: 0.0,
            },
        ]),
    })?;
    ctx.input.add_table(&InputTable {
        name: "default".to_string(),
        display_name: "Default Inputs".to_string(),
        description: "".to_string(),
        actions: Vec::from([
            InputAction {
                name: "roll_left".to_string(),
                display_name: "Roll Left".to_string(), 
                description: "".to_string(),
                default_pressed: false,
            },
            InputAction { 
                name: "roll_right".to_string(),
                display_name: "Roll Right".to_string(), 
                description: "".to_string(), 
                default_pressed: false,
            },
            InputAction {
                name: "switch_mode".to_string(),
                display_name: "Switch Mode".to_string(), 
                description: "".to_string(), 
                default_pressed: false,
            },
            InputAction {
                name: "move_fast".to_string(),
                display_name: "Move Fast".to_string(), 
                description: "".to_string(), 
                default_pressed: false,
            },
            InputAction {
                name: "move_slow".to_string(),
                display_name: "Move Slow".to_string(), 
                description: "".to_string(), 
                default_pressed: false,
            },
        ]),
        axis: Vec::from([]),
    })?;

    // Non default assets
    ctx.asset.add(Material::UID, "alfred", default_bundle, Material { 
        diffuse: "alfred".into(),
    })?;
    ctx.asset.add(Material::UID, "car", default_bundle, Material {
        diffuse: "car".into(),
    })?;
    ctx.asset.add(Model::UID, "car", default_bundle, Model { 
        mesh: "car".into(),
        materials: Vec::from(["car".into()])
    })?;
    ctx.asset.add(Model::UID, "alfred", default_bundle, Model { 
        mesh: "alfred".into(), 
        materials: Vec::from([
            "alfred".into(),
            "alfred".into(),
            "alfred".into(),
        ])
    })?;

    // Import assets
    for import in ctx.event.import_asset() {
        match import {
            ImportAssetEvent::Material(material) => {
                ctx.asset.add(Material::UID, &material.name, default_bundle, material.data.clone())?;
            },
            ImportAssetEvent::Mesh(mesh) => {
                ctx.asset.add(Mesh::UID, &mesh.name, default_bundle, mesh.data.clone())?;
            },
            ImportAssetEvent::Model(model) => {
                ctx.asset.add(Model::UID, &model.name, default_bundle, model.data.clone())?;
            },
            ImportAssetEvent::RhaiScript(rhai_script) => {
                ctx.asset.add(RhaiScript::UID, &rhai_script.name, default_bundle, rhai_script.data.clone())?;
            },
            ImportAssetEvent::Texture(texture) => {
                ctx.asset.add(Texture::UID, &texture.name, default_bundle, texture.data.clone())?;
            },
            _ => {},
        }
    }

    Ok(())
}

fn setup_world(ctx: &mut SystemContext) -> Result<()> {

    let world = ctx.world.add("main")?;
    ctx.world.change(world)?;
    let mut world = ctx.world.get(world)?;

    {
        let e = world.create();
        world.add(e, Lifecycle::UID, Lifecycle::alive())?;
        world.add(e, Transform::UID, Transform {
            translation: Vec3::new(0.0, -7.0, 0.0),    
            rotation: Quat::IDENTITY,    
            scale: Vec3::new(0.5, 0.5, 0.5),
        })?;
        world.add(e, LocalToWorld::UID, LocalToWorld::default())?;
        world.add(e, Rotator::UID, Rotator { speed: 90.0 })?;
        world.add(e, StaticMesh::UID, StaticMesh::new("alfred".into()))?;
    }
    {
        let e = world.create();
        world.add(e, Lifecycle::UID, Lifecycle::alive())?;
        world.add(e, Transform::UID, Transform::from_translation(Vec3::new(0.0, -7.0, 9.0)))?;
        world.add(e, LocalToWorld::UID, LocalToWorld::default())?;
        world.add(e, StaticMesh::UID, StaticMesh::new("alfred".into()))?;
    }
    {
        for i in 0..100 {
            let e = world.create();
            world.add(e, Lifecycle::UID, Lifecycle::alive())?;
            world.add(e, Transform::UID, Transform::from_translation(
                Vec3::new(((i / 10) * 5) as f32, 0.0,  -((i % 10) * 8) as f32
            )))?;
            world.add(e, LocalToWorld::UID, LocalToWorld::default())?;
            world.add(e, StaticMesh::UID, StaticMesh::new("car".into()))?;
            world.add(e, Rotator::UID, Rotator { speed: -90.0 + rand::random::<f32>() * 90.0 * 2.0 })?;
            let e = world.create();
            world.add(e, Lifecycle::UID, Lifecycle::alive())?;
            world.add(e, Transform::UID, Transform::from_translation(
                Vec3::new(((i / 10) * 5) as f32, 10.0, -((i % 10) * 8) as f32
            )))?;
            world.add(e, LocalToWorld::UID, LocalToWorld::default())?;
            world.add(e, StaticMesh::UID, StaticMesh::new("alfred".into()))?;
            world.add(e, Rotator::UID, Rotator { speed: -90.0 + rand::random::<f32>() * 90.0 * 2.0 })?;
        }
    }
    {
        let e = world.create();
        world.add(e, Lifecycle::UID, Lifecycle::alive())?;
        world.add(e, Transform::UID, Transform::from_translation(Vec3::new(0.0, 0.0, 4.0)))?;
        world.add(e, LocalToWorld::UID, LocalToWorld::default())?;
        world.add(e, StaticMesh::UID, StaticMesh::new("car".into()))?;
        world.add(e, Rotator::UID, Rotator { speed: 30.0 })?;
    }
    {
        let e = world.create();
        world.add(e, Lifecycle::UID, Lifecycle::alive())?;
        world.add(e, Transform::UID, Transform::from_translation(Vec3::new(0.0, 0.0, -10.0)))?;
        world.add(e, LocalToWorld::UID, LocalToWorld::default())?;
        world.add(e, FreeFly::UID, FreeFly {
            active: true,
            switch_mode: "switch_mode".into(),
            roll_left: "roll_left".into(),
            roll_right: "roll_right".into(),
            view_x: CommonAxis::VIEW_X.into(), 
            view_y: CommonAxis::VIEW_Y.into(),
            move_forward: CommonAxis::MOVE_FORWARD.into(),
            move_backward: CommonAxis::MOVE_BACKWARD.into(),
            move_up: CommonAxis::MOVE_UP.into(),
            move_down: CommonAxis::MOVE_DOWN.into(),
            move_left: CommonAxis::MOVE_LEFT.into(),
            move_right: CommonAxis::MOVE_RIGHT.into(),
            move_fast: "move_fast".into(),
            move_slow: "move_slow".into(),
            free_mode: false,
            yaw: 0.0,
            pitch: 0.0,
        })?;
        world.add(e, StaticMesh::UID, StaticMesh::new("car".into()))?;
        world.add(e, ScriptStorage::UID, ScriptStorage::default())?;
        world.add(e, RhaiScripts::UID, RhaiScripts::default())?;
        world.add(e, Hierarchy::UID, Hierarchy::default())?;

        let cam = world.create();
        world.add(cam, Lifecycle::UID, Lifecycle::alive())?;
        world.add(cam, Transform::UID, Transform::from_translation(Vec3::new(4.0, -1.0, 0.0)))?;
        world.add(cam, LocalToWorld::UID, LocalToWorld::default())?;
        world.add(cam, Camera::UID, Camera::default())?;
        world.add(cam, Hierarchy::UID, Hierarchy::default())?;
        
        Hierarchy::attach(e, cam, &mut world.view_mut::<Hierarchy>(Hierarchy::UID)?)?;

        world.get_mut::<RhaiScripts>(e, RhaiScripts::UID)?.unwrap()
            .add("inventory".into()).unwrap();

        let viewport = world.create();
        world.add(viewport, Viewport::UID, Viewport::new(SCREEN_RESOLUTION, Some(cam)))?;
    
        let mut ui = UI::default();
        for _ in 0..30 {
            // ui.add_label(&format!("test{}", i), 30, UID::null(), Label::new((5, i * 10).into(), "0123456789012345678901234567890123456789", "default".into()))?;
        }
        ui.add_checkbox("checkbox", 50, UID::null(), Checkbox::new((50, 100).into(), true))?;
        ui.add_viewport("main_viewport", 0, UID::null(), ui::viewport::Viewport::new(IVec2::ZERO, world.uid(), viewport))?;
        // ui.add_viewport("second_viewport", 50, UID::null(), Viewport::new((440, 200).into(), UID::null(), viewport2))?;
        ui.add_profile("main", InteractionInputs {
            click: CommonAction::CLICK.into(),
            up: CommonAction::UP.into(),
            down: CommonAction::DOWN.into(),
            left: CommonAction::LEFT.into(),
            right: CommonAction::RIGHT.into(),
            cursor_x: CommonAxis::CURSOR_X.into(), 
            cursor_y: CommonAxis::CURSOR_Y.into(),
            cursor_motion_x: CommonAxis::CURSOR_MOTION_X.into(),
            cursor_motion_y: CommonAxis::CURSOR_MOTION_Y.into(),
            scroll: CommonAxis::SCROLL_MOTION.into(),
        })?;
        let uie = world.create();
        world.add(uie, Lifecycle::UID, Lifecycle::alive())?;
        world.add(uie, UIComponent::UID, UIComponent::new(ui, UIRenderTarget::Screen { offset: IVec2::ZERO }))?;
    }

    // Setup singleton
    {
        world.add_singleton(OS::UID, OS { layout_active: true })?;
    }
    
    Ok(())
}

fn setup_scheduler(ctx: &mut SystemContext) -> Result<()> {
    let pipeline = SystemPipeline::new(&[
        UID::new("rotator"),
        UID::new("rhai_update_scripts"),
        UID::new("transform_propagate"),
        UID::new("ui_update"),
        UID::new("ui_render"),
        UID::new("renderer"),
        UID::new("despawn_entities"),
        UID::new("free_fly"),
        UID::new("update"),
    ]);
    let mut group = SystemGroup::empty();
    group.insert(Procedure::UPDATE, pipeline, 0);
    ctx.scheduler.add_group("os", group)?;
    Ok(())
}

pub fn init(ctx: &mut SystemContext) -> Result<()> {
    define_features(ctx)?;
    setup_assets(ctx)?;
    setup_world(ctx)?;
    setup_scheduler(ctx)?;
    Ok(())
}