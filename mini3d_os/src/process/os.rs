use mini3d::{uid::UID, process::{ProcessContext, Process}, feature::{asset::{font::FontAsset, input_action::InputActionAsset, input_axis::{InputAxisAsset, InputAxisRange}, input_table::InputTableAsset, material::MaterialAsset, model::ModelAsset, mesh::MeshAsset, rhai_script::RhaiScriptAsset, system_schedule::{SystemScheduleAsset, SystemScheduleType}, texture::TextureAsset}, component::{lifecycle::LifecycleComponent, transform::TransformComponent, rotator::RotatorComponent, model::ModelComponent, free_fly::FreeFlyComponent, camera::CameraComponent, script_storage::ScriptStorageComponent, rhai_scripts::RhaiScriptsComponent, ui::{UIComponent, UIRenderTarget}, viewport::ViewportComponent}, process::profiler::ProfilerProcess}, renderer::{SCREEN_WIDTH, SCREEN_HEIGHT, SCREEN_CENTER, SCREEN_RESOLUTION, color::Color}, anyhow::{Result, Context}, glam::{Vec3, Quat, IVec2}, rand, math::rect::IRect, scene::Scene, ui::{interaction_layout::{InteractionLayout, InteractionInputs}, UI, viewport::Viewport, checkbox::Checkbox, label::Label}};
use serde::{Serialize, Deserialize};

use crate::{input::{CommonAxis, CommonAction}};

#[derive(Default, Serialize, Deserialize)]
pub struct OSProcess {
    scene: UID,
    navigation_layout: InteractionLayout,
    // ui: UI,
    control_profile: UID,
    layout_active: bool,
}

impl OSProcess {

    fn setup_assets(&mut self, ctx: &mut ProcessContext) -> Result<()> {
        ctx.asset.add_bundle("default").unwrap();
        let default_bundle = UID::new("default");

        // Register default font
        ctx.asset.add("default", default_bundle, FontAsset::default())?;

        // Register common inputs
        ctx.asset.add(CommonAction::CLICK, default_bundle, InputActionAsset {
            display_name: "Click".to_string(),
            description: "UI interaction layout (click).".to_string(),
            default_pressed: false,
        })?;
        ctx.asset.add(CommonAction::UP, default_bundle, InputActionAsset {
            display_name: "Up".to_string(),
            description: "UI interaction layout (go up).".to_string(),
            default_pressed: false,
        })?;
        ctx.asset.add(CommonAction::LEFT, default_bundle, InputActionAsset {
            display_name: "Left".to_string(),
            description: "UI interaction layout (go left).".to_string(),
            default_pressed: false,
        })?;
        ctx.asset.add(CommonAction::DOWN, default_bundle, InputActionAsset {
            display_name: "Down".to_string(),
            description: "UI interaction layout (go down).".to_string(),
            default_pressed: false,
        })?;
        ctx.asset.add(CommonAction::RIGHT, default_bundle, InputActionAsset {
            display_name: "Right".to_string(),
            description: "UI interaction layout (go right).".to_string(),
            default_pressed: false,
        })?;
        ctx.asset.add(CommonAction::CHANGE_CONTROL_MODE, default_bundle, InputActionAsset {
            display_name: "Change Control Mode".to_string(),
            description: "Switch between selection and cursor control mode.".to_string(),
            default_pressed: false,
        })?;
        ctx.asset.add(CommonAction::TOGGLE_PROFILER, default_bundle, InputActionAsset {
            display_name: "Toggle Profiler".to_string(),
            description: "Show or hide the profiler.".to_string(),
            default_pressed: false,
        })?;
        ctx.asset.add(CommonAxis::CURSOR_X, default_bundle, InputAxisAsset {
            display_name: "Cursor X".to_string(),
            description: "Horizontal position of the mouse cursor relative to the screen.".to_string(),
            range: InputAxisRange::Clamped { min: 0.0, max: SCREEN_WIDTH as f32 },
            default_value: 0.0,
        })?;
        ctx.asset.add(CommonAxis::CURSOR_Y, default_bundle, InputAxisAsset {
            display_name: "Cursor Y".to_string(),
            description: "Vertical position of the mouse cursor relative to the screen.".to_string(),
            range: InputAxisRange::Clamped { min: 0.0, max: SCREEN_HEIGHT as f32 },
            default_value: 0.0,
        })?;
        ctx.asset.add(CommonAxis::SCROLL_MOTION, default_bundle, InputAxisAsset {
            display_name: "Scroll Motion".to_string(),
            description: "Delta scrolling value.".to_string(),
            range: InputAxisRange::Infinite,
            default_value: 0.0,
        })?;
        ctx.asset.add(CommonAxis::CURSOR_MOTION_X, default_bundle, InputAxisAsset {
            display_name: "Cursor Motion X".to_string(),
            description: "Delta mouvement of the mouse on the horizontal axis.".to_string(),
            range: InputAxisRange::Infinite,
            default_value: 0.0,
        })?;
        ctx.asset.add(CommonAxis::CURSOR_MOTION_Y, default_bundle, InputAxisAsset {
            display_name: "Cursor Motion Y".to_string(),
            description: "Delta mouvement of the mouse on the vertical axis.".to_string(),
            range: InputAxisRange::Infinite,
            default_value: 0.0,
        })?;
        ctx.asset.add(CommonAxis::VIEW_X, default_bundle, InputAxisAsset {
            display_name: "View X".to_string(),
            description: "View horizontal delta movement.".to_string(),
            range: InputAxisRange::Infinite,
            default_value: 0.0,
        })?;
        ctx.asset.add(CommonAxis::VIEW_Y, default_bundle, InputAxisAsset {
            display_name: "View Y".to_string(),
            description: "View vertical delta movement.".to_string(),
            range: InputAxisRange::Infinite,
            default_value: 0.0,
        })?;
        ctx.asset.add(CommonAxis::MOVE_FORWARD, default_bundle, InputAxisAsset { 
            display_name: "Move Forward".to_string(), 
            description: "".to_string(), 
            range: InputAxisRange::Clamped { min: 0.0, max: 1.0 }, 
            default_value: 0.0,
        })?;
        ctx.asset.add(CommonAxis::MOVE_BACKWARD, default_bundle, InputAxisAsset { 
            display_name: "Move Backward".to_string(), 
            description: "".to_string(), 
            range: InputAxisRange::Clamped { min: 0.0, max: 1.0 }, 
            default_value: 0.0,
        })?;
        ctx.asset.add(CommonAxis::MOVE_LEFT, default_bundle, InputAxisAsset { 
            display_name: "Move Left".to_string(), 
            description: "".to_string(), 
            range: InputAxisRange::Clamped { min: 0.0, max: 1.0 }, 
            default_value: 0.0,
        })?;
        ctx.asset.add(CommonAxis::MOVE_RIGHT, default_bundle, InputAxisAsset { 
            display_name: "Move Right".to_string(), 
            description: "".to_string(), 
            range: InputAxisRange::Clamped { min: 0.0, max: 1.0 }, 
            default_value: 0.0,
        })?;
        ctx.asset.add(CommonAxis::MOVE_UP, default_bundle, InputAxisAsset { 
            display_name: "Move Up".to_string(), 
            description: "".to_string(), 
            range: InputAxisRange::Clamped { min: 0.0, max: 1.0 }, 
            default_value: 0.0,
        })?;
        ctx.asset.add(CommonAxis::MOVE_DOWN, default_bundle, InputAxisAsset { 
            display_name: "Move Down".to_string(), 
            description: "".to_string(), 
            range: InputAxisRange::Clamped { min: 0.0, max: 1.0 },
            default_value: 0.0,
        })?;
                
        // Register default inputs
        ctx.asset.add("roll_left", default_bundle, InputActionAsset { 
            display_name: "Roll Left".to_string(), 
            description: "".to_string(),
            default_pressed: false,
        })?;
        ctx.asset.add("roll_right", default_bundle, InputActionAsset { 
            display_name: "Roll Right".to_string(), 
            description: "".to_string(), 
            default_pressed: false,
        })?;
        ctx.asset.add("switch_mode", default_bundle, InputActionAsset { 
            display_name: "Switch Mode".to_string(), 
            description: "".to_string(), 
            default_pressed: false,
        })?;
        ctx.asset.add("move_fast", default_bundle, InputActionAsset { 
            display_name: "Move Fast".to_string(), 
            description: "".to_string(), 
            default_pressed: false,
        })?;
        ctx.asset.add("move_slow", default_bundle, InputActionAsset { 
            display_name: "Move Slow".to_string(), 
            description: "".to_string(), 
            default_pressed: false,
        })?;

        // Register input tables
        ctx.asset.add::<InputTableAsset>("common", default_bundle, InputTableAsset { 
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
        ctx.asset.add::<InputTableAsset>("default", default_bundle, InputTableAsset {
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
        ctx.asset.add::<MaterialAsset>("alfred", default_bundle, MaterialAsset { 
            diffuse: "alfred".into(),
        })?;
        ctx.asset.add::<MaterialAsset>("car", default_bundle, MaterialAsset {
            diffuse: "car".into(),
        })?;
        ctx.asset.add::<ModelAsset>("car", default_bundle, ModelAsset { 
            mesh: "car".into(),
            materials: Vec::from(["car".into()])
        })?;
        ctx.asset.add::<ModelAsset>("alfred", default_bundle, ModelAsset { 
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
                mini3d::event::asset::ImportAssetEvent::Material(material) => {
                    ctx.asset.add::<MaterialAsset>(&material.name, default_bundle, material.data.clone())?;
                },
                mini3d::event::asset::ImportAssetEvent::Mesh(mesh) => {
                    ctx.asset.add::<MeshAsset>(&mesh.name, default_bundle, mesh.data.clone())?;
                },
                mini3d::event::asset::ImportAssetEvent::Model(model) => {
                    ctx.asset.add::<ModelAsset>(&model.name, default_bundle, model.data.clone())?;
                },
                mini3d::event::asset::ImportAssetEvent::RhaiScript(rhai_script) => {
                    ctx.asset.add::<RhaiScriptAsset>(&rhai_script.name, default_bundle, rhai_script.data.clone())?;
                },
                mini3d::event::asset::ImportAssetEvent::Texture(texture) => {
                    ctx.asset.add::<TextureAsset>(&texture.name, default_bundle, texture.data.clone())?;
                },
                _ => {},
            }
        }

        // Scheduler
        ctx.asset.add("test_scheduler", default_bundle, SystemScheduleAsset {
            systems: Vec::from([
                SystemScheduleType::Builtin("rotator".into()),
                SystemScheduleType::Builtin("rhai_update_scripts".into()),
                SystemScheduleType::Builtin("ui_update_and_render".into()),
                SystemScheduleType::Builtin("renderer".into()),
                SystemScheduleType::Builtin("despawn_entities".into()),
                SystemScheduleType::Builtin("free_fly".into()),
            ]),
        })?;

        Ok(())
    }

    fn setup_world(&mut self, ctx: &mut ProcessContext) -> Result<()> {
        self.scene = ctx.scene.add("main").with_context(|| "Failed to create ECS")?;
        let world = ctx.scene.world(self.scene)?;
        world.spawn((
            LifecycleComponent::alive(),
            TransformComponent {
                translation: Vec3::new(0.0, -7.0, 0.0),    
                rotation: Quat::IDENTITY,    
                scale: Vec3::new(0.5, 0.5, 0.5),
                dirty: true, 
            },
            RotatorComponent { speed: 90.0 },
            ModelComponent::new("alfred".into()),
        ));
        world.spawn((
            LifecycleComponent::alive(),
            TransformComponent::from_translation(Vec3::new(0.0, -7.0, 9.0)),
            ModelComponent::new("alfred".into()),
        ));
        for i in 0..100 {
            world.spawn((
                LifecycleComponent::alive(),
                TransformComponent::from_translation(
                    Vec3::new(((i / 10) * 5) as f32, 0.0,  -((i % 10) * 8) as f32
                )),
                ModelComponent::new("car".into()),
                RotatorComponent { speed: -90.0 + rand::random::<f32>() * 90.0 * 2.0 }
            ));
            world.spawn((
                LifecycleComponent::alive(),
                TransformComponent::from_translation(
                    Vec3::new(((i / 10) * 5) as f32, 10.0,  -((i % 10) * 8) as f32
                )),
                ModelComponent::new("alfred".into()),
                RotatorComponent { speed: -90.0 + rand::random::<f32>() * 90.0 * 2.0 }
            ));
        }
        world.spawn((
            LifecycleComponent::alive(),
            TransformComponent::from_translation(Vec3::new(0.0, 0.0, 4.0)),
            ModelComponent::new("car".into()),
            RotatorComponent { speed: 30.0 }
        ));
        let e = world.spawn((
            LifecycleComponent::alive(),
            TransformComponent::from_translation(Vec3::new(0.0, 0.0, -10.0)),
            FreeFlyComponent {
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
            ModelComponent::new("car".into()),
            CameraComponent::default(),
            ScriptStorageComponent::default(),
            RhaiScriptsComponent::default(),
        ));
        let cam2 = world.spawn((
            LifecycleComponent::alive(),
            TransformComponent::from_translation(Vec3::new(10.0, 30.0, -10.0)),
            CameraComponent::default(),
        ));
                
        world.get::<&mut RhaiScriptsComponent>(e).unwrap().add("inventory".into()).unwrap();
        
        let viewport = world.spawn((
            ViewportComponent::new(SCREEN_RESOLUTION, Some(e)),
        ));
        let viewport2 = world.spawn((
            ViewportComponent::new((200, 50).into(), Some(cam2)),
        ));

        {
            let mut ui = UI::default();
            for i in 0..30 {
                // ui.add_label(&format!("test{}", i), 30, UID::null(), Label::new((5, i * 10).into(), "0123456789012345678901234567890123456789", "default".into()))?;
            }
            ui.add_checkbox("checkbox", 50, UID::null(), Checkbox::new((50, 100).into(), true))?;
            ui.add_viewport("main_viewport", 0, UID::null(), Viewport::new(IVec2::ZERO, self.scene, viewport))?;
            ui.add_viewport("second_viewport", 50, UID::null(), Viewport::new((440, 200).into(), UID::null(), viewport2))?;
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
                LifecycleComponent::alive(),
                UIComponent::new(ui, UIRenderTarget::Screen { offset: IVec2::ZERO }),
            ));
        }
        
        Ok(())
    }
}

impl Process for OSProcess {
    
    fn start(&mut self, ctx: &mut ProcessContext) -> Result<()> {

        // Register default bundle
        {
            self.setup_assets(ctx)?;
        
            // let file = File::create("assets/dump.json").unwrap();
            // let mut json_serializer = serde_json::Serializer::new(file);
            // ctx.asset.serialize_bundle("default".into(), &mut json_serializer)?;

            // let mut file = File::create("assets/rom.bin").unwrap();
            // let mut bytes: Vec<u8> = Default::default();
            // let mut bincode_serializer = bincode::Serializer::new(&mut bytes, bincode::options());
            // ctx.asset.serialize_bundle("default".into(), &mut bincode_serializer)?;
            // bytes = miniz_oxide::deflate::compress_to_vec_zlib(bytes.as_slice(), 10);
            // use std::io::Write;
            // file.write_all(&bytes).unwrap();


            // let mut file = File::open("assets/rom.bin").with_context(|| "Failed to open file")?;
            // let mut bytes: Vec<u8> = Default::default();
            // file.read_to_end(&mut bytes).with_context(|| "Failed to read to end")?;
            // let bytes = miniz_oxide::inflate::decompress_to_vec_zlib(&bytes).expect("Failed to decompress");
            // let mut deserializer = bincode::Deserializer::from_slice(&bytes, bincode::options());
            // let import = ctx.asset.deserialize_bundle(&mut deserializer)?;
            // ctx.asset.import_bundle(import)?;

            // let file = File::open("assets/dump.json").unwrap();
            // let mut json_deserializer = serde_json::Deserializer::from_reader(file);
            // let import = ctx.asset.deserialize_bundle(&mut json_deserializer)?;
            // ctx.asset.import_bundle(import)?;
        }

        ctx.input.reload_input_tables(ctx.asset)?;

        {
            // Initialize world
            self.setup_world(ctx)?;

            // let file = File::create("assets/world.json")?;
            // let mut serializer = serde_json::Serializer::new(file);
            // self.scene.serialize(ctx.scene, &mut serializer)?;

            // let file = File::create("assets/world.bin")?;
            // let mut serializer = bincode::Serializer::new(file, bincode::options());
            // self.scene.serialize(ctx.scene, &mut serializer)?;

            // let file = File::open("assets/world.json")?;
            // let mut deserializer = serde_json::Deserializer::from_reader(file);
            // self.scene.deserialize(ctx.scene, &mut deserializer)?;

            // let mut file = File::open("assets/world.bin")?;
            // let mut bytes: Vec<u8> = Default::default();
            // file.read_to_end(&mut bytes).unwrap();
            // let mut deserializer = bincode::Deserializer::from_slice(&bytes, bincode::options());
            // self.scene.deserialize(ctx.scene, &mut deserializer)?;
        }

        // Configure schedule
        let schedule = ctx.asset.get::<SystemScheduleAsset>("test_scheduler".into()).unwrap();
        ctx.scene.schedule(self.scene, schedule)?;

        // Run profiler
        ctx.process.start("profiler", ProfilerProcess::new(UID::new(CommonAction::TOGGLE_PROFILER)))?;

        Ok(())
    }

    fn update(&mut self, ctx: &mut ProcessContext) -> Result<()> {

        // Progress ECS
        Scene::progress(self.scene, ctx)?;

        // // Toggle control mode
        if ctx.input.action(CommonAction::CHANGE_CONTROL_MODE.into())?.is_just_pressed() {
            self.layout_active = !self.layout_active;
            for (_, free_fly) in ctx.scene.world(self.scene)?.query_mut::<&mut FreeFlyComponent>() {
                free_fly.active = !self.layout_active;
            }
        }

        // Toggle control layout
        if self.layout_active {

        }

        // Render center cross
        ctx.renderer.graphics().fill_rect(IRect::new(SCREEN_CENTER.x as i32, SCREEN_CENTER.y as i32, 4, 4), Color::WHITE);

        Ok(())
    }
}