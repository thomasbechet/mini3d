use mini3d::{program::{ProgramId, ProgramBuilder, Program, ProgramContext}, asset::{material::Material, model::Model, AssetRef, input_action::InputAction, font::Font, input_axis::{InputAxis, InputAxisKind}}, hecs::World, ecs::{component::{transform::TransformComponent, model::ModelComponent, rotator::RotatorComponent, free_fly::FreeFlyComponent, camera::CameraComponent, rhai_scripts::RhaiScriptsComponent, script_storage::ScriptStorageComponent, lifecycle::LifecycleComponent}, system::{rotator::system_rotator, free_fly::system_free_fly, rhai::system_rhai_update_scripts, despawn::system_despawn_entities, renderer::{system_renderer_update_camera, system_renderer_transfer_transforms, system_renderer_check_lifecycle}}}, renderer::{CommandBuffer, SCREEN_WIDTH, SCREEN_HEIGHT, SCREEN_CENTER}, anyhow::Result, glam::{Vec3, Quat}, input::{control_layout::{ControlLayout, ControlProfileId, ControlInputs}}, slotmap::Key, math::rect::IRect, rhai::RhaiContext, rand};

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
    world: World,
    control_layout: ControlLayout,
    control_profile: ControlProfileId,
    layout_active: bool,
    dt_record: Vec<f64>,
    last_dt: f64,
    time_graph: TimeGraph,
    rhai: RhaiContext,
}

impl ProgramBuilder for OSProgram {
    
    type BuildData = ();

    fn build(id: ProgramId, _data: Self::BuildData) -> Self {
        Self { 
            id, 
            world: Default::default(),
            control_layout: ControlLayout::new(),
            control_profile: ControlProfileId::null(),
            layout_active: false,
            dt_record: Vec::new(),
            last_dt: 0.0,
            time_graph: TimeGraph::new(240),
            rhai: RhaiContext::new(),
        }
    }
}

impl Program for OSProgram {
    
    fn start(&mut self, ctx: &mut ProgramContext) -> Result<()> {

        // Register common inputs
        ctx.asset.register(CommonAction::UP, InputAction {
            display_name: "Up".to_string(),
            description: "Layout navigation control (go up).".to_string(),
            default_pressed: false,
        })?;
        ctx.asset.register(CommonAction::LEFT, InputAction {
            display_name: "Left".to_string(),
            description: "Layout navigation control (go left).".to_string(),
            default_pressed: false,
        })?;
        ctx.asset.register(CommonAction::DOWN, InputAction {
            display_name: "Down".to_string(),
            description: "Layout navigation control (go down).".to_string(),
            default_pressed: false,
        })?;
        ctx.asset.register(CommonAction::RIGHT, InputAction {
            display_name: "Right".to_string(),
            description: "Layout navigation control (go right).".to_string(),
            default_pressed: false,
        })?;
        ctx.asset.register(CommonAction::CHANGE_CONTROL_MODE, InputAction {
            display_name: "Change Control Mode".to_string(),
            description: "Switch between selection and cursor control mode.".to_string(),
            default_pressed: false,
        })?;
        ctx.asset.register(CommonAxis::CURSOR_X, InputAxis {
            display_name: "Cursor X".to_string(),
            description: "Horizontal position of the mouse cursor relative to the screen.".to_string(),
            kind: InputAxisKind::Clamped { min: 0.0, max: SCREEN_WIDTH as f32 },
            default_value: 0.0,
        })?;
        ctx.asset.register(CommonAxis::CURSOR_Y, InputAxis {
            display_name: "Cursor Y".to_string(),
            description: "Vertical position of the mouse cursor relative to the screen.".to_string(),
            kind: InputAxisKind::Clamped { min: 0.0, max: SCREEN_HEIGHT as f32 },
            default_value: 0.0,
        })?;
        ctx.asset.register(CommonAxis::CURSOR_MOTION_X, InputAxis {
            display_name: "Cursor Motion X".to_string(),
            description: "Delta mouvement of the mouse on the horizontal axis.".to_string(),
            kind: InputAxisKind::Infinite,
            default_value: 0.0,
        })?;
        ctx.asset.register(CommonAxis::CURSOR_MOTION_Y, InputAxis {
            display_name: "Cursor Motion Y".to_string(),
            description: "Delta mouvement of the mouse on the vertical axis.".to_string(),
            kind: InputAxisKind::Infinite,
            default_value: 0.0,
        })?;
        ctx.asset.register(CommonAxis::VIEW_X, InputAxis {
            display_name: "View X".to_string(),
            description: "View horizontal delta movement.".to_string(),
            kind: InputAxisKind::Infinite,
            default_value: 0.0,
        })?;
        ctx.asset.register(CommonAxis::VIEW_Y, InputAxis {
            display_name: "View Y".to_string(),
            description: "View vertical delta movement.".to_string(),
            kind: InputAxisKind::Infinite,
            default_value: 0.0,
        })?;
        ctx.asset.register(CommonAxis::MOVE_FORWARD, InputAxis { 
            display_name: "Move Forward".to_string(), 
            description: "".to_string(), 
            kind: InputAxisKind::Clamped { min: 0.0, max: 1.0 }, 
            default_value: 0.0,
        })?;
        ctx.asset.register(CommonAxis::MOVE_BACKWARD, InputAxis { 
            display_name: "Move Backward".to_string(), 
            description: "".to_string(), 
            kind: InputAxisKind::Clamped { min: 0.0, max: 1.0 }, 
            default_value: 0.0,
        })?;
        ctx.asset.register(CommonAxis::MOVE_LEFT, InputAxis { 
            display_name: "Move Left".to_string(), 
            description: "".to_string(), 
            kind: InputAxisKind::Clamped { min: 0.0, max: 1.0 }, 
            default_value: 0.0,
        })?;
        ctx.asset.register(CommonAxis::MOVE_RIGHT, InputAxis { 
            display_name: "Move Right".to_string(), 
            description: "".to_string(), 
            kind: InputAxisKind::Clamped { min: 0.0, max: 1.0 }, 
            default_value: 0.0,
        })?;
        ctx.asset.register(CommonAxis::MOVE_UP, InputAxis { 
            display_name: "Move Up".to_string(), 
            description: "".to_string(), 
            kind: InputAxisKind::Clamped { min: 0.0, max: 1.0 }, 
            default_value: 0.0,
        })?;
        ctx.asset.register(CommonAxis::MOVE_DOWN, InputAxis { 
            display_name: "Move Down".to_string(), 
            description: "".to_string(), 
            kind: InputAxisKind::Clamped { min: 0.0, max: 1.0 },
            default_value: 0.0,
        })?;
                
        // Register default inuts
        ctx.asset.register("roll_left", InputAction { 
            display_name: "Roll Left".to_string(), 
            description: "".to_string(),
            default_pressed: false,
        }).unwrap();
        ctx.asset.register("roll_right", InputAction { 
            display_name: "Roll Right".to_string(), 
            description: "".to_string(), 
            default_pressed: false,
        }).unwrap();
        ctx.asset.register("switch_mode", InputAction { 
            display_name: "Switch Mode".to_string(), 
            description: "".to_string(), 
            default_pressed: false,
        }).unwrap();

        // Add initial control profile
        self.control_profile = self.control_layout.add_profile(ControlInputs {
            up: AssetRef::new(CommonAction::UP),
            down: AssetRef::new(CommonAction::DOWN),
            left: AssetRef::new(CommonAction::LEFT),
            right: AssetRef::new(CommonAction::RIGHT),
            cursor_x: AssetRef::new(CommonAxis::CURSOR_X), 
            cursor_y: AssetRef::new(CommonAxis::CURSOR_Y),
            cursor_motion_x: AssetRef::new(CommonAxis::CURSOR_MOTION_X),
            cursor_motion_y: AssetRef::new(CommonAxis::CURSOR_MOTION_Y),
        });

        self.control_layout.add_control(IRect::new(5, 5, 100, 50));
        self.control_layout.add_control(IRect::new(5, 200, 100, 50));

        // Initialize world
        ctx.asset.register::<Material>("alfred", Material { 
            diffuse: "alfred".into(),
        })?;
        ctx.asset.register::<Material>("car", Material {
            diffuse: "car".into(),
        })?;
        ctx.asset.register::<Model>("car", Model { 
            mesh: "car".into(),
            materials: Vec::from(["car".into()])
        })?;
        ctx.asset.register::<Model>("alfred", Model { 
            mesh: "alfred".into(), 
            materials: Vec::from([
                "car".into(),
                "car".into(),
                "car".into(),
            ])
        })?;
        
        self.world.spawn((
            LifecycleComponent::default(),
            TransformComponent {
                translation: Vec3::new(0.0, -7.0, 0.0),    
                rotation: Quat::IDENTITY,    
                scale: Vec3::new(0.5, 0.5, 0.5),    
            },
            RotatorComponent { speed: 90.0 },
            ModelComponent::from(AssetRef::new("alfred")),
        ));
        self.world.spawn((
            LifecycleComponent::default(),
            TransformComponent::from_translation(Vec3::new(0.0, -7.0, 9.0)),
            ModelComponent::from(AssetRef::new("alfred")),
        ));
        for i in 0..100 {
            self.world.spawn((
                LifecycleComponent::default(),
                TransformComponent::from_translation(
                    Vec3::new(((i / 10) * 5) as f32, 0.0,  -((i % 10) * 8) as f32
                )),
                ModelComponent::from(AssetRef::new("car")),
                RotatorComponent { speed: -90.0 + rand::random::<f32>() * 90.0 * 2.0 }
            ));
        }
        self.world.spawn((
            LifecycleComponent::default(),
            TransformComponent::from_translation(Vec3::new(0.0, 0.0, 4.0)),
            ModelComponent::from(AssetRef::new("car")),
            RotatorComponent { speed: 30.0 }
        ));
        let e = self.world.spawn((
            LifecycleComponent::default(),
            TransformComponent::from_translation(Vec3::new(0.0, 0.0, -10.0)),
            FreeFlyComponent {
                switch_mode: AssetRef::new("switch_mode"),
                roll_left: AssetRef::new("roll_left"),
                roll_right: AssetRef::new("roll_right"),
                view_x: AssetRef::new(CommonAxis::VIEW_X), 
                view_y: AssetRef::new(CommonAxis::VIEW_Y),
                move_forward: AssetRef::new(CommonAxis::MOVE_FORWARD),
                move_backward: AssetRef::new(CommonAxis::MOVE_BACKWARD),
                move_up: AssetRef::new(CommonAxis::MOVE_UP),
                move_down: AssetRef::new(CommonAxis::MOVE_DOWN),
                move_left: AssetRef::new(CommonAxis::MOVE_LEFT),
                move_right: AssetRef::new(CommonAxis::MOVE_RIGHT),
                free_mode: false,
                yaw: 0.0,
                pitch: 0.0,
            },
            CameraComponent::default(),
            ScriptStorageComponent::default(),
            RhaiScriptsComponent::default(),
        ));
                
        self.world.get::<&mut RhaiScriptsComponent>(e).unwrap().add(AssetRef::new("inventory")).unwrap();

        // let mut file = File::create("assets.bin").unwrap();
        // let bytes = bincode::serialize(&ctx.asset.bundle(self.asset_bundle))?;
        // let bytes = miniz_oxide::deflate::compress_to_vec_zlib(bytes.as_slice(), 10);
        // file.write(bytes.as_slice()).unwrap();

        Ok(())
    }

    fn update(&mut self, ctx: &mut ProgramContext) -> Result<()> {

        // Call ECS systems
        system_rotator(&mut self.world, ctx.delta_time as f32);
        system_rhai_update_scripts(&mut self.world, &mut self.rhai, ctx);
        system_renderer_transfer_transforms(&mut self.world, ctx.renderer);
        system_renderer_update_camera(&mut self.world, ctx.renderer);
        system_renderer_check_lifecycle(&mut self.world, ctx.renderer, &ctx.asset);
        system_despawn_entities(&mut self.world);

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

            if ctx.input.find_action(CommonAction::CHANGE_CONTROL_MODE.into(), ctx.asset, false).is_just_pressed() {
                self.layout_active = !self.layout_active;
            }

            if self.layout_active {
                self.control_layout.update(ctx.asset, ctx.input);
                let cb0 = self.control_layout.render();
                ctx.renderer.push_command_buffer(cb0);
            } else {
                system_free_fly(&mut self.world, ctx.input, ctx.asset, ctx.delta_time as f32);
            }

            let font = ctx.asset.find::<Font>("default".into()).unwrap().id;
            let cb1 = CommandBuffer::build_with(|builder| {
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