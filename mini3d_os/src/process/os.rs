use std::{fs::File, io::Read};

use mini3d::{uid::UID, process::{ProcessContext, Process}, feature::{asset::{font::Font, input_action::InputAction, input_axis::{InputAxis, InputAxisRange}, input_table::InputTable, material::Material, model::Model, mesh::Mesh, rhai_script::RhaiScript, system_schedule::{SystemSchedule, SystemScheduleType}, texture::Texture}, component::{lifecycle::LifecycleComponent, transform::TransformComponent, rotator::RotatorComponent, model::ModelComponent, free_fly::FreeFlyComponent, camera::CameraComponent, script_storage::ScriptStorageComponent, rhai_scripts::RhaiScriptsComponent}, process::profiler::ProfilerProcess}, graphics::{SCREEN_WIDTH, SCREEN_HEIGHT, SCREEN_CENTER, command_buffer::{Command, CommandBuffer}}, anyhow::{Result, Context}, glam::{Vec3, Quat}, rand, math::rect::IRect, ecs::ECS, gui::navigation_layout::{NavigationLayout, NavigationLayoutInputs}};
use serde::{Serialize, Deserialize};

use crate::{input::{CommonAxis, CommonAction}};

#[derive(Default, Serialize, Deserialize)]
pub struct OSProcess {
    ecs: UID,
    navigation_layout: NavigationLayout,
    control_profile: UID,
    layout_active: bool,
}

impl OSProcess {

    fn setup_assets(&mut self, ctx: &mut ProcessContext) -> Result<()> {
        ctx.asset.add_bundle("default").unwrap();
        let default_bundle = UID::new("default");

        // Register default font
        ctx.asset.add("default", default_bundle, Font::default())?;

        // Register common inputs
        ctx.asset.add(CommonAction::UP, default_bundle, InputAction {
            display_name: "Up".to_string(),
            description: "Layout navigation control (go up).".to_string(),
            default_pressed: false,
        })?;
        ctx.asset.add(CommonAction::LEFT, default_bundle, InputAction {
            display_name: "Left".to_string(),
            description: "Layout navigation control (go left).".to_string(),
            default_pressed: false,
        })?;
        ctx.asset.add(CommonAction::DOWN, default_bundle, InputAction {
            display_name: "Down".to_string(),
            description: "Layout navigation control (go down).".to_string(),
            default_pressed: false,
        })?;
        ctx.asset.add(CommonAction::RIGHT, default_bundle, InputAction {
            display_name: "Right".to_string(),
            description: "Layout navigation control (go right).".to_string(),
            default_pressed: false,
        })?;
        ctx.asset.add(CommonAction::CHANGE_CONTROL_MODE, default_bundle, InputAction {
            display_name: "Change Control Mode".to_string(),
            description: "Switch between selection and cursor control mode.".to_string(),
            default_pressed: false,
        })?;
        ctx.asset.add(CommonAction::TOGGLE_PROFILER, default_bundle, InputAction {
            display_name: "Toggle Profiler".to_string(),
            description: "Show or hide the profiler.".to_string(),
            default_pressed: false,
        })?;
        ctx.asset.add(CommonAxis::CURSOR_X, default_bundle, InputAxis {
            display_name: "Cursor X".to_string(),
            description: "Horizontal position of the mouse cursor relative to the screen.".to_string(),
            range: InputAxisRange::Clamped { min: 0.0, max: SCREEN_WIDTH as f32 },
            default_value: 0.0,
        })?;
        ctx.asset.add(CommonAxis::CURSOR_Y, default_bundle, InputAxis {
            display_name: "Cursor Y".to_string(),
            description: "Vertical position of the mouse cursor relative to the screen.".to_string(),
            range: InputAxisRange::Clamped { min: 0.0, max: SCREEN_HEIGHT as f32 },
            default_value: 0.0,
        })?;
        ctx.asset.add(CommonAxis::CURSOR_MOTION_X, default_bundle, InputAxis {
            display_name: "Cursor Motion X".to_string(),
            description: "Delta mouvement of the mouse on the horizontal axis.".to_string(),
            range: InputAxisRange::Infinite,
            default_value: 0.0,
        })?;
        ctx.asset.add(CommonAxis::CURSOR_MOTION_Y, default_bundle, InputAxis {
            display_name: "Cursor Motion Y".to_string(),
            description: "Delta mouvement of the mouse on the vertical axis.".to_string(),
            range: InputAxisRange::Infinite,
            default_value: 0.0,
        })?;
        ctx.asset.add(CommonAxis::VIEW_X, default_bundle, InputAxis {
            display_name: "View X".to_string(),
            description: "View horizontal delta movement.".to_string(),
            range: InputAxisRange::Infinite,
            default_value: 0.0,
        })?;
        ctx.asset.add(CommonAxis::VIEW_Y, default_bundle, InputAxis {
            display_name: "View Y".to_string(),
            description: "View vertical delta movement.".to_string(),
            range: InputAxisRange::Infinite,
            default_value: 0.0,
        })?;
        ctx.asset.add(CommonAxis::MOVE_FORWARD, default_bundle, InputAxis { 
            display_name: "Move Forward".to_string(), 
            description: "".to_string(), 
            range: InputAxisRange::Clamped { min: 0.0, max: 1.0 }, 
            default_value: 0.0,
        })?;
        ctx.asset.add(CommonAxis::MOVE_BACKWARD, default_bundle, InputAxis { 
            display_name: "Move Backward".to_string(), 
            description: "".to_string(), 
            range: InputAxisRange::Clamped { min: 0.0, max: 1.0 }, 
            default_value: 0.0,
        })?;
        ctx.asset.add(CommonAxis::MOVE_LEFT, default_bundle, InputAxis { 
            display_name: "Move Left".to_string(), 
            description: "".to_string(), 
            range: InputAxisRange::Clamped { min: 0.0, max: 1.0 }, 
            default_value: 0.0,
        })?;
        ctx.asset.add(CommonAxis::MOVE_RIGHT, default_bundle, InputAxis { 
            display_name: "Move Right".to_string(), 
            description: "".to_string(), 
            range: InputAxisRange::Clamped { min: 0.0, max: 1.0 }, 
            default_value: 0.0,
        })?;
        ctx.asset.add(CommonAxis::MOVE_UP, default_bundle, InputAxis { 
            display_name: "Move Up".to_string(), 
            description: "".to_string(), 
            range: InputAxisRange::Clamped { min: 0.0, max: 1.0 }, 
            default_value: 0.0,
        })?;
        ctx.asset.add(CommonAxis::MOVE_DOWN, default_bundle, InputAxis { 
            display_name: "Move Down".to_string(), 
            description: "".to_string(), 
            range: InputAxisRange::Clamped { min: 0.0, max: 1.0 },
            default_value: 0.0,
        })?;
                
        // Register default inputs
        ctx.asset.add("roll_left", default_bundle, InputAction { 
            display_name: "Roll Left".to_string(), 
            description: "".to_string(),
            default_pressed: false,
        })?;
        ctx.asset.add("roll_right", default_bundle, InputAction { 
            display_name: "Roll Right".to_string(), 
            description: "".to_string(), 
            default_pressed: false,
        })?;
        ctx.asset.add("switch_mode", default_bundle, InputAction { 
            display_name: "Switch Mode".to_string(), 
            description: "".to_string(), 
            default_pressed: false,
        })?;

        // Register input tables
        ctx.asset.add::<InputTable>("common", default_bundle, InputTable { 
            display_name: "Common Inputs".to_string(),
            description: "".to_string(), 
            actions: Vec::from([
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
        ctx.asset.add::<InputTable>("default", default_bundle, InputTable {
            display_name: "Default Inputs".to_string(),
            description: "".to_string(),
            actions: Vec::from([
                "roll_left".into(),
                "roll_right".into(),
                "switch_mode".into(),
            ]),
            axis: Vec::from([]),
        })?;

        // Non default assets
        ctx.asset.add::<Material>("alfred", default_bundle, Material { 
            diffuse: "alfred".into(),
        })?;
        ctx.asset.add::<Material>("car", default_bundle, Material {
            diffuse: "car".into(),
        })?;
        ctx.asset.add::<Model>("car", default_bundle, Model { 
            mesh: "car".into(),
            materials: Vec::from(["car".into()])
        })?;
        ctx.asset.add::<Model>("alfred", default_bundle, Model { 
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
                    ctx.asset.add::<Material>(&material.name, default_bundle, material.data.clone())?;
                },
                mini3d::event::asset::ImportAssetEvent::Mesh(mesh) => {
                    ctx.asset.add::<Mesh>(&mesh.name, default_bundle, mesh.data.clone())?;
                },
                mini3d::event::asset::ImportAssetEvent::Model(model) => {
                    ctx.asset.add::<Model>(&model.name, default_bundle, model.data.clone())?;
                },
                mini3d::event::asset::ImportAssetEvent::RhaiScript(rhai_script) => {
                    ctx.asset.add::<RhaiScript>(&rhai_script.name, default_bundle, rhai_script.data.clone())?;
                },
                mini3d::event::asset::ImportAssetEvent::Texture(texture) => {
                    ctx.asset.add::<Texture>(&texture.name, default_bundle, texture.data.clone())?;
                },
                _ => {},
            }
        }

        // Scheduler
        ctx.asset.add("test_scheduler", default_bundle, SystemSchedule {
            systems: Vec::from([
                SystemScheduleType::Builtin("rotator".into()),
                SystemScheduleType::Builtin("rhai_update_scripts".into()),
                SystemScheduleType::Builtin("renderer_check_lifecycle".into()),
                SystemScheduleType::Builtin("renderer_transfer_transforms".into()),
                SystemScheduleType::Builtin("renderer_update_camera".into()),
                SystemScheduleType::Builtin("despawn_entities".into()),
                SystemScheduleType::Builtin("free_fly".into()),
            ]),
        })?;

        Ok(())
    }

    fn setup_world(&mut self, ctx: &mut ProcessContext) -> Result<()> {
        self.ecs = ctx.ecs.add("main").with_context(|| "Failed to create ECS")?;
        let world = ctx.ecs.world(self.ecs)?;
        world.spawn((
            LifecycleComponent::default(),
            TransformComponent {
                translation: Vec3::new(0.0, -7.0, 0.0),    
                rotation: Quat::IDENTITY,    
                scale: Vec3::new(0.5, 0.5, 0.5),    
            },
            RotatorComponent { speed: 90.0 },
            ModelComponent::new("alfred".into()),
        ));
        world.spawn((
            LifecycleComponent::default(),
            TransformComponent::from_translation(Vec3::new(0.0, -7.0, 9.0)),
            ModelComponent::new("alfred".into()),
        ));
        for i in 0..100 {
            world.spawn((
                LifecycleComponent::default(),
                TransformComponent::from_translation(
                    Vec3::new(((i / 10) * 5) as f32, 0.0,  -((i % 10) * 8) as f32
                )),
                ModelComponent::new("car".into()),
                RotatorComponent { speed: -90.0 + rand::random::<f32>() * 90.0 * 2.0 }
            ));
        }
        world.spawn((
            LifecycleComponent::default(),
            TransformComponent::from_translation(Vec3::new(0.0, 0.0, 4.0)),
            ModelComponent::new("car".into()),
            RotatorComponent { speed: 30.0 }
        ));
        let e = world.spawn((
            LifecycleComponent::default(),
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
                free_mode: false,
                yaw: 0.0,
                pitch: 0.0,
            },
            CameraComponent::default(),
            ScriptStorageComponent::default(),
            RhaiScriptsComponent::default(),
        ));
                
        world.get::<&mut RhaiScriptsComponent>(e).unwrap().add("inventory".into()).unwrap();

        Ok(())
    }
}

impl Process for OSProcess {
    
    fn start(&mut self, ctx: &mut ProcessContext) -> Result<()> {

        // Register default bundle
        {
            // self.setup_assets(ctx)?;
        
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


            let mut file = File::open("assets/rom.bin").with_context(|| "Failed to open file")?;
            let mut bytes: Vec<u8> = Default::default();
            file.read_to_end(&mut bytes).with_context(|| "Failed to read to end")?;
            let bytes = miniz_oxide::inflate::decompress_to_vec_zlib(&bytes).expect("Failed to decompress");
            let mut deserializer = bincode::Deserializer::from_slice(&bytes, bincode::options());
            let import = ctx.asset.deserialize_bundle(&mut deserializer)?;
            ctx.asset.import_bundle(import)?;

            // let file = File::open("assets/dump.json").unwrap();
            // let mut json_deserializer = serde_json::Deserializer::from_reader(file);
            // let import = ctx.asset.deserialize_bundle(&mut json_deserializer)?;
            // ctx.asset.import_bundle(import)?;
        }

        ctx.input.reload_input_tables(ctx.asset)?;

        // Add initial control profile
        self.control_profile = self.navigation_layout.add_profile("main", NavigationLayoutInputs {
            up: CommonAction::UP.into(),
            down: CommonAction::DOWN.into(),
            left: CommonAction::LEFT.into(),
            right: CommonAction::RIGHT.into(),
            cursor_x: CommonAxis::CURSOR_X.into(), 
            cursor_y: CommonAxis::CURSOR_Y.into(),
            cursor_motion_x: CommonAxis::CURSOR_MOTION_X.into(),
            cursor_motion_y: CommonAxis::CURSOR_MOTION_Y.into(),
        })?;

        self.navigation_layout.add_area("area1", IRect::new(5, 5, 100, 50))?;
        self.navigation_layout.add_area("area2", IRect::new(5, 200, 100, 50))?;
        self.navigation_layout.add_area("area3", IRect::new(150, 5, 100, 50))?;
        self.navigation_layout.add_area("area4", IRect::new(150, 200, 50, 50))?;
        self.navigation_layout.add_area("area5", IRect::new(400, 50, 100, 200))?;

        {
            // Initialize world
            self.setup_world(ctx)?;

            // let file = File::create("assets/world.json")?;
            // let mut serializer = serde_json::Serializer::new(file);
            // self.ecs.serialize(ctx.ecs, &mut serializer)?;

            // let file = File::create("assets/world.bin")?;
            // let mut serializer = bincode::Serializer::new(file, bincode::options());
            // self.ecs.serialize(ctx.ecs, &mut serializer)?;

            // let file = File::open("assets/world.json")?;
            // let mut deserializer = serde_json::Deserializer::from_reader(file);
            // self.ecs.deserialize(ctx.ecs, &mut deserializer)?;

            // let mut file = File::open("assets/world.bin")?;
            // let mut bytes: Vec<u8> = Default::default();
            // file.read_to_end(&mut bytes).unwrap();
            // let mut deserializer = bincode::Deserializer::from_slice(&bytes, bincode::options());
            // self.ecs.deserialize(ctx.ecs, &mut deserializer)?;
        }

        // Configure schedule
        let schedule = ctx.asset.get::<SystemSchedule>("test_scheduler".into()).unwrap();
        ctx.ecs.set_schedule(self.ecs, schedule)?;

        // Run profiler
        ctx.process.start("profiler", ProfilerProcess::new(UID::new(CommonAction::TOGGLE_PROFILER)))?;
        Ok(())
    }

    fn update(&mut self, ctx: &mut ProcessContext) -> Result<()> {

        // Progress ECS
        ECS::progress(self.ecs, ctx)?;

        // // Toggle control mode
        if ctx.input.action(CommonAction::CHANGE_CONTROL_MODE.into())?.is_just_pressed() {
            self.layout_active = !self.layout_active;
            for (_, free_fly) in ctx.ecs.world(self.ecs)?.query_mut::<&mut FreeFlyComponent>() {
                free_fly.active = !self.layout_active;
            } 
        }

        // Toggle control layout
        if self.layout_active {
            self.navigation_layout.update(ctx.input, ctx.time)?;
            let cb0 = self.navigation_layout.render(ctx.time);
            ctx.renderer.push_command_buffer(cb0);
        }

        // Render center cross
        let mut cb = CommandBuffer::empty();
        cb.push(Command::FillRect { rect: IRect::new(SCREEN_CENTER.x as i32, SCREEN_CENTER.y as i32, 2, 2) });
        ctx.renderer.push_command_buffer(cb);

        Ok(())
    }
}