use mini3d::{context::SystemContext, anyhow::Result, feature::{asset::{font::Font, input_action::InputAction, input_axis::{InputAxis, InputAxisRange}, input_table::InputTable, material::Material, model::Model, mesh::Mesh, rhai_script::RhaiScript, texture::Texture, system_group::{SystemGroup, SystemPipeline}}, component::{lifecycle::Lifecycle, transform::Transform, local_to_world::LocalToWorld, rotator::Rotator}}, renderer::{SCREEN_WIDTH, SCREEN_HEIGHT}, ecs::{procedure::Procedure, world::World}, glam::{Vec3, Quat}, event::asset::ImportAssetEvent};

use crate::{input::{CommonAction, CommonAxis}, asset::DefaultAsset};

fn setup_assets(ctx: &mut SystemContext) -> Result<()> {
    ctx.asset.add_bundle(DefaultAsset::BUNDLE).unwrap();
    let default_bundle = DefaultAsset::BUNDLE.into();

    // Register default font
    ctx.asset.add(Font::UID, "default", default_bundle, Font::default())?;

    // Register common inputs
    ctx.asset.add(InputAction::UID, CommonAction::CLICK, default_bundle, InputAction {
        display_name: "Click".to_string(),
        description: "UI interaction layout (click).".to_string(),
        default_pressed: false,
    })?;
    ctx.asset.add(InputAction::UID, CommonAction::UP, default_bundle, InputAction {
        display_name: "Up".to_string(),
        description: "UI interaction layout (go up).".to_string(),
        default_pressed: false,
    })?;
    ctx.asset.add(InputAction::UID, CommonAction::LEFT, default_bundle, InputAction {
        display_name: "Left".to_string(),
        description: "UI interaction layout (go left).".to_string(),
        default_pressed: false,
    })?;
    ctx.asset.add(InputAction::UID, CommonAction::DOWN, default_bundle, InputAction {
        display_name: "Down".to_string(),
        description: "UI interaction layout (go down).".to_string(),
        default_pressed: false,
    })?;
    ctx.asset.add(InputAction::UID, CommonAction::RIGHT, default_bundle, InputAction {
        display_name: "Right".to_string(),
        description: "UI interaction layout (go right).".to_string(),
        default_pressed: false,
    })?;
    ctx.asset.add(InputAction::UID, CommonAction::CHANGE_CONTROL_MODE, default_bundle, InputAction {
        display_name: "Change Control Mode".to_string(),
        description: "Switch between selection and cursor control mode.".to_string(),
        default_pressed: false,
    })?;
    ctx.asset.add(InputAction::UID, CommonAction::TOGGLE_PROFILER, default_bundle, InputAction {
        display_name: "Toggle Profiler".to_string(),
        description: "Show or hide the profiler.".to_string(),
        default_pressed: false,
    })?;
    ctx.asset.add(InputAxis::UID, CommonAxis::CURSOR_X, default_bundle, InputAxis {
        display_name: "Cursor X".to_string(),
        description: "Horizontal position of the mouse cursor relative to the screen.".to_string(),
        range: InputAxisRange::Clamped { min: 0.0, max: SCREEN_WIDTH as f32 },
        default_value: 0.0,
    })?;
    ctx.asset.add(InputAxis::UID, CommonAxis::CURSOR_Y, default_bundle, InputAxis {
        display_name: "Cursor Y".to_string(),
        description: "Vertical position of the mouse cursor relative to the screen.".to_string(),
        range: InputAxisRange::Clamped { min: 0.0, max: SCREEN_HEIGHT as f32 },
        default_value: 0.0,
    })?;
    ctx.asset.add(InputAxis::UID, CommonAxis::SCROLL_MOTION, default_bundle, InputAxis {
        display_name: "Scroll Motion".to_string(),
        description: "Delta scrolling value.".to_string(),
        range: InputAxisRange::Infinite,
        default_value: 0.0,
    })?;
    ctx.asset.add(InputAxis::UID, CommonAxis::CURSOR_MOTION_X, default_bundle, InputAxis {
        display_name: "Cursor Motion X".to_string(),
        description: "Delta mouvement of the mouse on the horizontal axis.".to_string(),
        range: InputAxisRange::Infinite,
        default_value: 0.0,
    })?;
    ctx.asset.add(InputAxis::UID, CommonAxis::CURSOR_MOTION_Y, default_bundle, InputAxis {
        display_name: "Cursor Motion Y".to_string(),
        description: "Delta mouvement of the mouse on the vertical axis.".to_string(),
        range: InputAxisRange::Infinite,
        default_value: 0.0,
    })?;
    ctx.asset.add(InputAxis::UID, CommonAxis::VIEW_X, default_bundle, InputAxis {
        display_name: "View X".to_string(),
        description: "View horizontal delta movement.".to_string(),
        range: InputAxisRange::Infinite,
        default_value: 0.0,
    })?;
    ctx.asset.add(InputAxis::UID, CommonAxis::VIEW_Y, default_bundle, InputAxis {
        display_name: "View Y".to_string(),
        description: "View vertical delta movement.".to_string(),
        range: InputAxisRange::Infinite,
        default_value: 0.0,
    })?;
    ctx.asset.add(InputAxis::UID, CommonAxis::MOVE_FORWARD, default_bundle, InputAxis { 
        display_name: "Move Forward".to_string(), 
        description: "".to_string(), 
        range: InputAxisRange::Clamped { min: 0.0, max: 1.0 }, 
        default_value: 0.0,
    })?;
    ctx.asset.add(InputAxis::UID, CommonAxis::MOVE_BACKWARD, default_bundle, InputAxis { 
        display_name: "Move Backward".to_string(), 
        description: "".to_string(), 
        range: InputAxisRange::Clamped { min: 0.0, max: 1.0 }, 
        default_value: 0.0,
    })?;
    ctx.asset.add(InputAxis::UID, CommonAxis::MOVE_LEFT, default_bundle, InputAxis { 
        display_name: "Move Left".to_string(), 
        description: "".to_string(), 
        range: InputAxisRange::Clamped { min: 0.0, max: 1.0 }, 
        default_value: 0.0,
    })?;
    ctx.asset.add(InputAxis::UID, CommonAxis::MOVE_RIGHT, default_bundle, InputAxis { 
        display_name: "Move Right".to_string(), 
        description: "".to_string(), 
        range: InputAxisRange::Clamped { min: 0.0, max: 1.0 }, 
        default_value: 0.0,
    })?;
    ctx.asset.add(InputAxis::UID, CommonAxis::MOVE_UP, default_bundle, InputAxis { 
        display_name: "Move Up".to_string(), 
        description: "".to_string(), 
        range: InputAxisRange::Clamped { min: 0.0, max: 1.0 }, 
        default_value: 0.0,
    })?;
    ctx.asset.add(InputAxis::UID, CommonAxis::MOVE_DOWN, default_bundle, InputAxis { 
        display_name: "Move Down".to_string(), 
        description: "".to_string(), 
        range: InputAxisRange::Clamped { min: 0.0, max: 1.0 },
        default_value: 0.0,
    })?;
            
    // Register default inputs
    ctx.asset.add(InputAction::UID, "roll_left", default_bundle, InputAction { 
        display_name: "Roll Left".to_string(), 
        description: "".to_string(),
        default_pressed: false,
    })?;
    ctx.asset.add(InputAction::UID, "roll_right", default_bundle, InputAction { 
        display_name: "Roll Right".to_string(), 
        description: "".to_string(), 
        default_pressed: false,
    })?;
    ctx.asset.add(InputAction::UID, "switch_mode", default_bundle, InputAction { 
        display_name: "Switch Mode".to_string(), 
        description: "".to_string(), 
        default_pressed: false,
    })?;
    ctx.asset.add(InputAction::UID, "move_fast", default_bundle, InputAction { 
        display_name: "Move Fast".to_string(), 
        description: "".to_string(), 
        default_pressed: false,
    })?;
    ctx.asset.add(InputAction::UID, "move_slow", default_bundle, InputAction { 
        display_name: "Move Slow".to_string(), 
        description: "".to_string(), 
        default_pressed: false,
    })?;

    // Register input tables
    ctx.asset.add(InputTable::UID, "common", default_bundle, InputTable { 
        display_name: "Common Inputs".to_string(),
        description: "".to_string(), 
        actions: Vec::from([
            CommonAction::CLICK.into(),
            CommonAction::UP.into(),
            CommonAction::LEFT.into(),
            CommonAction::DOWN.into(),
            CommonAction::RIGHT.into(),
            CommonAction::CHANGE_CONTROL_MODE.into(),
            CommonAction::TOGGLE_PROFILER.into(),
        ]),
        axis: Vec::from([
            CommonAxis::CURSOR_X.into(),
            CommonAxis::CURSOR_Y.into(),
            CommonAxis::SCROLL_MOTION.into(),
            CommonAxis::CURSOR_MOTION_X.into(),
            CommonAxis::CURSOR_MOTION_Y.into(),
            CommonAxis::VIEW_X.into(),
            CommonAxis::VIEW_Y.into(),
            CommonAxis::MOVE_FORWARD.into(),
            CommonAxis::MOVE_BACKWARD.into(),
            CommonAxis::MOVE_LEFT.into(),
            CommonAxis::MOVE_RIGHT.into(),
            CommonAxis::MOVE_UP.into(),
            CommonAxis::MOVE_DOWN.into(),
        ]),
    })?;
    ctx.asset.add(InputTable::UID, "default", default_bundle, InputTable {
        display_name: "Default Inputs".to_string(),
        description: "".to_string(),
        actions: Vec::from([
            "roll_left".into(),
            "roll_right".into(),
            "switch_mode".into(),
            "move_fast".into(),
            "move_slow".into(),
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
    for import in &ctx.events.asset {
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

fn setup_scheduler(ctx: &mut SystemContext) -> Result<()> {
    let pipeline = SystemPipeline::from([
        "rotator",
        "rhai_update_scripts",
        "transform_propagate",
        "ui_update",
        "ui_render",
        "renderer",
        "despawn_entities",
        "free_fly",
    ]);
    let mut group = SystemGroup::empty();
    group.insert(Procedure::UPDATE, pipeline, 0);
    ctx.scheduler.add_group("os", group)?;
    Ok(())
}

fn setup_world(ctx: &mut SystemContext) -> Result<()> {

    let world = ctx.world.add("main")?;
    ctx.world.change(world)?;
    let world = ctx.world.get(world)?;

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
        world.add(e, , Rotator { speed: 90.0 })?;
    }
    

    world.spawn((
        Lifecycle::alive(),
        Transform {
            translation: Vec3::new(0.0, -7.0, 0.0),    
            rotation: Quat::IDENTITY,    
            scale: Vec3::new(0.5, 0.5, 0.5),
        },
        LocalToWorld::default(),
        Rotator { speed: 90.0 },
        Model::new("alfred".into()),
    ));
    world.spawn((
        Lifecycle::alive(),
        Transform::from_translation(Vec3::new(0.0, -7.0, 9.0)),
        LocalToWorld::default(),
        Model::new("alfred".into()),
    ));
    for i in 0..100 {
        world.spawn((
            Lifecycle::alive(),
            Transform::from_translation(
                Vec3::new(((i / 10) * 5) as f32, 0.0,  -((i % 10) * 8) as f32
            )),
            LocalToWorld::default(),
            Model::new("car".into()),
            Rotator { speed: -90.0 + rand::random::<f32>() * 90.0 * 2.0 }
        ));
        world.spawn((
            Lifecycle::alive(),
            Transform::from_translation(
                Vec3::new(((i / 10) * 5) as f32, 10.0,  -((i % 10) * 8) as f32
            )),
            LocalToWorld::default(),
            Model::new("alfred".into()),
            Rotator { speed: -90.0 + rand::random::<f32>() * 90.0 * 2.0 }
        ));
    }
    world.spawn((
        Lifecycle::alive(),
        Transform::from_translation(Vec3::new(0.0, 0.0, 4.0)),
        LocalToWorld::default(),
        Model::new("car".into()),
        Rotator { speed: 30.0 }
    ));
    let e = world.spawn((
        Lifecycle::alive(),
        Transform::from_translation(Vec3::new(0.0, 0.0, -10.0)),
        LocalToWorld::default(),
        FreeFly {
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
        },
        Model::new("car".into()),
        ScriptStorage::default(),
        RhaiScripts::default(),
        Hierarchy::default(),
    ));
    let cam = world.spawn((
        Lifecycle::alive(),
        Transform::from_translation(Vec3::new(4.0, -1.0, 0.0)),
        LocalToWorld::default(),
        Camera::default(),
        Hierarchy::default(),
    ));
    Hierarchy::attach(e, cam, world)?;
    // let cam2 = world.spawn((
    //     LifecycleComponent::alive(),
    //     TransformComponent::from_translation(Vec3::new(10.0, 30.0, -10.0)),
    //     LocalToWorldComponent::default(),
    //     CameraComponent::default(),
    // ));
            
    world.get::<&mut RhaiScripts>(e).unwrap().add("inventory".into()).unwrap();

    let viewport = world.spawn((
        Viewport::new(SCREEN_RESOLUTION, Some(cam)),
    ));
    // let viewport2 = world.spawn((
    //     ViewportComponent::new((200, 50).into(), Some(cam2)),
    // ));

    {
        let mut ui = UI::default();
        for i in 0..30 {
            // ui.add_label(&format!("test{}", i), 30, UID::null(), Label::new((5, i * 10).into(), "0123456789012345678901234567890123456789", "default".into()))?;
        }
        ui.add_checkbox("checkbox", 50, UID::null(), Checkbox::new((50, 100).into(), true))?;
        ui.add_viewport("main_viewport", 0, UID::null(), Viewport::new(IVec2::ZERO, self.scene, viewport))?;
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
        world.spawn((
            Lifecycle::alive(),
            UIComponent::new(ui, UIRenderTarget::Screen { offset: IVec2::ZERO }),
        ));
    }
    
    Ok(())
}

pub fn init(ctx: &mut SystemContext) -> Result<()> {
    setup_assets(ctx)?;
    setup_scheduler(ctx)?;
    setup_world(ctx)?;
    Ok(())
}