use std::{collections::HashSet, fs::File, io::{Read, Write}};

use bincode::Options;
use mini3d::{program::{ProgramId, ProgramBuilder, Program, ProgramContext}, asset::{material::Material, model::Model, input_action::InputAction, input_axis::{InputAxis, InputAxisRange}, input_table::InputTable, font::Font, mesh::Mesh, rhai_script::RhaiScript, texture::Texture, AssetBundle, system_schedule::{SystemSchedule, SystemScheduleType}}, ecs::{component::{transform::TransformComponent, model::ModelComponent, rotator::RotatorComponent, free_fly::FreeFlyComponent, camera::CameraComponent, rhai_scripts::RhaiScriptsComponent, script_storage::ScriptStorageComponent, lifecycle::LifecycleComponent}, ECS}, graphics::{CommandBuffer, SCREEN_WIDTH, SCREEN_HEIGHT, SCREEN_CENTER}, anyhow::{Result, Context}, glam::{Vec3, Quat}, input::{control_layout::{ControlLayout, ControlProfileId, ControlInputs}}, slotmap::Key, math::rect::IRect, rand, uid::UID};

use crate::{input::{CommonAxis, CommonAction}};

struct TimeGraph {
    records: Vec<f64>,
    head: usize,
}

impl TimeGraph {
    pub fn new(count: usize) -> Self {
        Self {
            records: vec![0.0; count],
            head: 0,
        }
    }
    pub fn add(&mut self, value: f64) {
        self.records[self.head] = value;
        self.head = (self.head + 1) % self.records.len();
    }
    pub fn render(&self) -> CommandBuffer {
        let mut builder = CommandBuffer::builder();
        let mut current = self.head;
        let base_x = 5;
        let base_y = 5;
        let height = 60;
        builder.draw_hline(SCREEN_HEIGHT as i32 - base_y, base_x, self.records.len() as i32);
        builder.draw_vline(base_x, SCREEN_HEIGHT as i32 - base_y - height, SCREEN_HEIGHT as i32 - base_y);
        loop {
            let vy = ((self.records[current] / (2.0 / 60.0)) * height as f64) as u32;
            let x = base_x + current as i32;
            let y = SCREEN_HEIGHT as i32 - base_y - vy as i32;
            builder.fill_rect(IRect::new(x, y, 1, 1));
            // builder.draw_vline(x, y, SCREEN_HEIGHT as i32 - base_y);
            current = (current + 1) % self.records.len();
            if current == self.head {
                break
            }
        }
        builder.build()
    }
}

pub struct OSProgram {
    id: ProgramId,
    ecs: ECS,
    control_layout: ControlLayout,
    control_profile: ControlProfileId,
    layout_active: bool,
    dt_record: Vec<f64>,
    last_dt: f64,
    time_graph: TimeGraph,
}

impl ProgramBuilder for OSProgram {
    
    type BuildData = ();

    fn build(id: ProgramId, _data: Self::BuildData) -> Self {
        Self { 
            id,
            ecs: ECS::new(),
            control_layout: ControlLayout::default(),
            control_profile: ControlProfileId::null(),
            layout_active: false,
            dt_record: Vec::new(),
            last_dt: 0.0,
            time_graph: TimeGraph::new(240),
        }
    }
}

impl OSProgram {
    fn load_assets(&mut self, ctx: &mut ProgramContext) -> Result<()> {
        let default_bundle = UID::new("default");
        // Register default font
        ctx.asset.register("default", default_bundle, Font::default())?;

        // Register common inputs
        ctx.asset.register(CommonAction::UP, default_bundle, InputAction {
            display_name: "Up".to_string(),
            description: "Layout navigation control (go up).".to_string(),
            default_pressed: false,
        })?;
        ctx.asset.register(CommonAction::LEFT, default_bundle, InputAction {
            display_name: "Left".to_string(),
            description: "Layout navigation control (go left).".to_string(),
            default_pressed: false,
        })?;
        ctx.asset.register(CommonAction::DOWN, default_bundle, InputAction {
            display_name: "Down".to_string(),
            description: "Layout navigation control (go down).".to_string(),
            default_pressed: false,
        })?;
        ctx.asset.register(CommonAction::RIGHT, default_bundle, InputAction {
            display_name: "Right".to_string(),
            description: "Layout navigation control (go right).".to_string(),
            default_pressed: false,
        })?;
        ctx.asset.register(CommonAction::CHANGE_CONTROL_MODE, default_bundle, InputAction {
            display_name: "Change Control Mode".to_string(),
            description: "Switch between selection and cursor control mode.".to_string(),
            default_pressed: false,
        })?;
        ctx.asset.register(CommonAxis::CURSOR_X, default_bundle, InputAxis {
            display_name: "Cursor X".to_string(),
            description: "Horizontal position of the mouse cursor relative to the screen.".to_string(),
            range: InputAxisRange::Clamped { min: 0.0, max: SCREEN_WIDTH as f32 },
            default_value: 0.0,
        })?;
        ctx.asset.register(CommonAxis::CURSOR_Y, default_bundle, InputAxis {
            display_name: "Cursor Y".to_string(),
            description: "Vertical position of the mouse cursor relative to the screen.".to_string(),
            range: InputAxisRange::Clamped { min: 0.0, max: SCREEN_HEIGHT as f32 },
            default_value: 0.0,
        })?;
        ctx.asset.register(CommonAxis::CURSOR_MOTION_X, default_bundle, InputAxis {
            display_name: "Cursor Motion X".to_string(),
            description: "Delta mouvement of the mouse on the horizontal axis.".to_string(),
            range: InputAxisRange::Infinite,
            default_value: 0.0,
        })?;
        ctx.asset.register(CommonAxis::CURSOR_MOTION_Y, default_bundle, InputAxis {
            display_name: "Cursor Motion Y".to_string(),
            description: "Delta mouvement of the mouse on the vertical axis.".to_string(),
            range: InputAxisRange::Infinite,
            default_value: 0.0,
        })?;
        ctx.asset.register(CommonAxis::VIEW_X, default_bundle, InputAxis {
            display_name: "View X".to_string(),
            description: "View horizontal delta movement.".to_string(),
            range: InputAxisRange::Infinite,
            default_value: 0.0,
        })?;
        ctx.asset.register(CommonAxis::VIEW_Y, default_bundle, InputAxis {
            display_name: "View Y".to_string(),
            description: "View vertical delta movement.".to_string(),
            range: InputAxisRange::Infinite,
            default_value: 0.0,
        })?;
        ctx.asset.register(CommonAxis::MOVE_FORWARD, default_bundle, InputAxis { 
            display_name: "Move Forward".to_string(), 
            description: "".to_string(), 
            range: InputAxisRange::Clamped { min: 0.0, max: 1.0 }, 
            default_value: 0.0,
        })?;
        ctx.asset.register(CommonAxis::MOVE_BACKWARD, default_bundle, InputAxis { 
            display_name: "Move Backward".to_string(), 
            description: "".to_string(), 
            range: InputAxisRange::Clamped { min: 0.0, max: 1.0 }, 
            default_value: 0.0,
        })?;
        ctx.asset.register(CommonAxis::MOVE_LEFT, default_bundle, InputAxis { 
            display_name: "Move Left".to_string(), 
            description: "".to_string(), 
            range: InputAxisRange::Clamped { min: 0.0, max: 1.0 }, 
            default_value: 0.0,
        })?;
        ctx.asset.register(CommonAxis::MOVE_RIGHT, default_bundle, InputAxis { 
            display_name: "Move Right".to_string(), 
            description: "".to_string(), 
            range: InputAxisRange::Clamped { min: 0.0, max: 1.0 }, 
            default_value: 0.0,
        })?;
        ctx.asset.register(CommonAxis::MOVE_UP, default_bundle, InputAxis { 
            display_name: "Move Up".to_string(), 
            description: "".to_string(), 
            range: InputAxisRange::Clamped { min: 0.0, max: 1.0 }, 
            default_value: 0.0,
        })?;
        ctx.asset.register(CommonAxis::MOVE_DOWN, default_bundle, InputAxis { 
            display_name: "Move Down".to_string(), 
            description: "".to_string(), 
            range: InputAxisRange::Clamped { min: 0.0, max: 1.0 },
            default_value: 0.0,
        })?;
                
        // Register default inputs
        ctx.asset.register("roll_left", default_bundle, InputAction { 
            display_name: "Roll Left".to_string(), 
            description: "".to_string(),
            default_pressed: false,
        })?;
        ctx.asset.register("roll_right", default_bundle, InputAction { 
            display_name: "Roll Right".to_string(), 
            description: "".to_string(), 
            default_pressed: false,
        })?;
        ctx.asset.register("switch_mode", default_bundle, InputAction { 
            display_name: "Switch Mode".to_string(), 
            description: "".to_string(), 
            default_pressed: false,
        })?;

        // Register input tables
        ctx.asset.register::<InputTable>("common", default_bundle, InputTable { 
            display_name: "Common Inputs".to_string(),
            description: "".to_string(), 
            actions: HashSet::from([
                CommonAction::UP.into(),
                CommonAction::LEFT.into(),
                CommonAction::DOWN.into(),
                CommonAction::RIGHT.into(),
                CommonAction::CHANGE_CONTROL_MODE.into(),
            ]),
            axis: HashSet::from([
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
        ctx.asset.register::<InputTable>("default", default_bundle, InputTable {
            display_name: "Default Inputs".to_string(),
            description: "".to_string(),
            actions: HashSet::from([
                "roll_left".into(),
                "roll_right".into(),
                "switch_mode".into(),
            ]),
            axis: HashSet::from([]),
        })?;

        // Non default assets
        ctx.asset.register::<Material>("alfred", default_bundle, Material { 
            diffuse: "alfred".into(),
        })?;
        ctx.asset.register::<Material>("car", default_bundle, Material {
            diffuse: "car".into(),
        })?;
        ctx.asset.register::<Model>("car", default_bundle, Model { 
            mesh: "car".into(),
            materials: Vec::from(["car".into()])
        })?;
        ctx.asset.register::<Model>("alfred", default_bundle, Model { 
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
                    ctx.asset.register::<Material>(&material.name, default_bundle, material.data.clone())?;
                },
                mini3d::event::asset::ImportAssetEvent::Mesh(mesh) => {
                    ctx.asset.register::<Mesh>(&mesh.name, default_bundle, mesh.data.clone())?;
                },
                mini3d::event::asset::ImportAssetEvent::Model(model) => {
                    ctx.asset.register::<Model>(&model.name, default_bundle, model.data.clone())?;
                },
                mini3d::event::asset::ImportAssetEvent::RhaiScript(rhai_script) => {
                    ctx.asset.register::<RhaiScript>(&rhai_script.name, default_bundle, rhai_script.data.clone())?;
                },
                mini3d::event::asset::ImportAssetEvent::Texture(texture) => {
                    ctx.asset.register::<Texture>(&texture.name, default_bundle, texture.data.clone())?;
                },
                _ => {},
            }
        }

        // Scheduler
        ctx.asset.register("test_scheduler", default_bundle, SystemSchedule {
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
}

impl Program for OSProgram {
    
    fn start(&mut self, ctx: &mut ProgramContext) -> Result<()> {

        // Register default bundle
        {
            ctx.asset.register_bundle("default", self.id).unwrap();
            self.load_assets(ctx)?;
            let export = ctx.asset.export_bundle("default".into())?;
            let mut file = File::create("assets/rom.bin").unwrap();
            let bytes = bincode::serialize(&export)?;
            let bytes = miniz_oxide::deflate::compress_to_vec_zlib(bytes.as_slice(), 10);
            file.write_all(&bytes).unwrap();
        
            // let file = File::create("assets/dump.json").unwrap();
            // serde_json::to_writer(file, &export).expect("Failed to serialize json");

            // let mut file = File::open("assets/rom.bin").context("Failed to open file")?;
            // let mut bytes: Vec<u8> = Default::default();
            // file.read_to_end(&mut bytes).context("Failed to read to end")?;
            // let bytes = miniz_oxide::inflate::decompress_to_vec_zlib(&bytes)
            //     .expect("Failed to decompress");
            // let import: AssetBundle = bincode::deserialize(&bytes).context("Failed to deserialize")?;
            // ctx.asset.import_bundle(import, self.id).context("Failed to import")?;
        }

        ctx.input.reload_input_tables(ctx.asset);

        // Add initial control profile
        self.control_profile = self.control_layout.add_profile(ControlInputs {
            up: ctx.input.find_action(CommonAction::UP.into())?,
            down: ctx.input.find_action(CommonAction::DOWN.into())?,
            left: ctx.input.find_action(CommonAction::LEFT.into())?,
            right: ctx.input.find_action(CommonAction::RIGHT.into())?,
            cursor_x: ctx.input.find_axis(CommonAxis::CURSOR_X.into())?, 
            cursor_y: ctx.input.find_axis(CommonAxis::CURSOR_Y.into())?,
            cursor_motion_x: ctx.input.find_axis(CommonAxis::CURSOR_MOTION_X.into())?,
            cursor_motion_y: ctx.input.find_axis(CommonAxis::CURSOR_MOTION_Y.into())?,
        });

        self.control_layout.add_control(IRect::new(5, 5, 100, 50));
        self.control_layout.add_control(IRect::new(5, 200, 100, 50));

        // Initialize world
        self.ecs.world.spawn((
            LifecycleComponent::default(),
            TransformComponent {
                translation: Vec3::new(0.0, -7.0, 0.0),    
                rotation: Quat::IDENTITY,    
                scale: Vec3::new(0.5, 0.5, 0.5),    
            },
            RotatorComponent { speed: 90.0 },
            ModelComponent::new("alfred".into()),
        ));
        self.ecs.world.spawn((
            LifecycleComponent::default(),
            TransformComponent::from_translation(Vec3::new(0.0, -7.0, 9.0)),
            ModelComponent::new("alfred".into()),
        ));
        for i in 0..100 {
            self.ecs.world.spawn((
                LifecycleComponent::default(),
                TransformComponent::from_translation(
                    Vec3::new(((i / 10) * 5) as f32, 0.0,  -((i % 10) * 8) as f32
                )),
                ModelComponent::new("car".into()),
                RotatorComponent { speed: -90.0 + rand::random::<f32>() * 90.0 * 2.0 }
            ));
        }
        self.ecs.world.spawn((
            LifecycleComponent::default(),
            TransformComponent::from_translation(Vec3::new(0.0, 0.0, 4.0)),
            ModelComponent::new("car".into()),
            RotatorComponent { speed: 30.0 }
        ));
        let e = self.ecs.world.spawn((
            LifecycleComponent::default(),
            TransformComponent::from_translation(Vec3::new(0.0, 0.0, -10.0)),
            FreeFlyComponent {
                active: true,
                switch_mode: ctx.input.find_action("switch_mode".into())?,
                roll_left: ctx.input.find_action("roll_left".into())?,
                roll_right: ctx.input.find_action("roll_right".into())?,
                view_x: ctx.input.find_axis(CommonAxis::VIEW_X.into())?, 
                view_y: ctx.input.find_axis(CommonAxis::VIEW_Y.into())?,
                move_forward: ctx.input.find_axis(CommonAxis::MOVE_FORWARD.into())?,
                move_backward: ctx.input.find_axis(CommonAxis::MOVE_BACKWARD.into())?,
                move_up: ctx.input.find_axis(CommonAxis::MOVE_UP.into())?,
                move_down: ctx.input.find_axis(CommonAxis::MOVE_DOWN.into())?,
                move_left: ctx.input.find_axis(CommonAxis::MOVE_LEFT.into())?,
                move_right: ctx.input.find_axis(CommonAxis::MOVE_RIGHT.into())?,
                free_mode: false,
                yaw: 0.0,
                pitch: 0.0,
            },
            CameraComponent::default(),
            ScriptStorageComponent::default(),
            RhaiScriptsComponent::default(),
        ));
                
        self.ecs.world.get::<&mut RhaiScriptsComponent>(e).unwrap().add("inventory".into()).unwrap();
        
        // Configure schedule
        let schedule = ctx.asset.get::<SystemSchedule>("test_scheduler".into()).unwrap();
        self.ecs.set_schedule(schedule).unwrap();

        Ok(())
    }

    fn update(&mut self, ctx: &mut ProgramContext) -> Result<()> {

        // Progress ECS scheduler
        self.ecs.progress(ctx)?;

        // Custom code
        {
            // Compute fps
            self.dt_record.push(ctx.delta_time);
            self.time_graph.add(ctx.delta_time);
            if self.dt_record.len() > 30 {
                self.dt_record.sort_by(|a, b| a.partial_cmp(b).unwrap());
                self.last_dt = self.dt_record[14];
                self.dt_record.clear();
            }

            {
                let id = ctx.input.find_action(CommonAction::CHANGE_CONTROL_MODE.into())?;
                if ctx.input.action(id)?.is_just_pressed() {
                    self.layout_active = !self.layout_active;
                    for (_, free_fly) in self.ecs.world.query_mut::<&mut FreeFlyComponent>() {
                        free_fly.active = !self.layout_active;
                    } 
                }

                if self.layout_active {
                    self.control_layout.update(ctx.input)?;
                    let cb0 = self.control_layout.render();
                    ctx.renderer.push_command_buffer(cb0);
                }
            }

            let cb1 = CommandBuffer::build_with(|builder| {
                let font = UID::new("default");
                builder
                .print((8, 8).into(), format!("dt : {:.2} ({:.1})", self.last_dt * 1000.0, 1.0 / self.last_dt).as_str(), font)
                .print((8, 17).into(), format!("dc : {}", ctx.renderer.statistics().draw_count).as_str(), font)
                .print((8, 26).into(), format!("tc : {}", ctx.renderer.statistics().triangle_count).as_str(), font)
                .print((8, 35).into(), format!("vp : {}x{}", ctx.renderer.statistics().viewport.0, ctx.renderer.statistics().viewport.1).as_str(), font)
                .fill_rect(IRect::new(SCREEN_CENTER.x as i32, SCREEN_CENTER.y as i32, 2, 2))
            });
            ctx.renderer.push_command_buffer(cb1);
            ctx.renderer.push_command_buffer(self.time_graph.render());
        }

        Ok(())
    }

    fn stop(&mut self, _ctx: &mut ProgramContext) -> Result<()> { 
        Ok(()) 
    }
}