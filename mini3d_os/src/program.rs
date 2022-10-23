use mini3d::{program::{ProgramId, ProgramBuilder, Program, ProgramContext}, asset::{AssetGroupId, font::Font, texture::Texture, mesh::Mesh, material::Material, script::RhaiScript, model::Model}, hecs::World, ecs::{component::{transform::TransformComponent, model::ModelComponent, rotator::RotatorComponent, free_fly::FreeFlyComponent, camera::CameraComponent, rhai_scripts::RhaiScriptsComponent, script_storage::ScriptStorageComponent, lifecycle::LifecycleComponent}, system::{rotator::system_rotator, free_fly::system_free_fly, rhai::system_rhai_update_scripts, despawn::system_despawn_entities, renderer::{system_renderer_update_camera, system_renderer_transfer_transforms, system_renderer_check_lifecycle}}}, graphics::{CommandBuffer, SCREEN_WIDTH, SCREEN_HEIGHT, SCREEN_CENTER}, anyhow::{Result, Context}, glam::{Vec3, Quat}, input::{InputGroupId, control_layout::{ControlLayout, ControlProfileId, ControlInputs}, axis::{AxisKind, AxisDescriptor}, action::ActionDescriptor}, slotmap::Key, math::rect::IRect, rhai::RhaiContext, rand::{random, self}};

use crate::{input::{CommonAxis, CommonAction, CommonInput}, asset::DefaultAsset};

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
    asset_group: AssetGroupId,
    input_group: InputGroupId,
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
            asset_group: AssetGroupId::null(), 
            input_group: InputGroupId::null(), 
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

        // Register default asset group
        self.asset_group = ctx.asset.register_group(DefaultAsset::GROUP, self.id)
            .context("Failed to register default asset group")?;
        // Register common input group
        self.input_group = ctx.input.register_group(CommonInput::GROUP, self.id)
            .context("Failed to register common input group")?;

        // Register common inputs
        ctx.input.register_action(self.input_group, ActionDescriptor {
            name: CommonAction::UP.to_string(),
            display_name: "Up".to_string(),
            description: "Layout navigation control (go up).".to_string(),
        })?;
        ctx.input.register_action(self.input_group, ActionDescriptor {
            name: CommonAction::LEFT.to_string(),
            display_name: "Left".to_string(),
            description: "Layout navigation control (go left).".to_string(),
        })?;
        ctx.input.register_action(self.input_group, ActionDescriptor {
            name: CommonAction::DOWN.to_string(),
            display_name: "Down".to_string(),
            description: "Layout navigation control (go down).".to_string(),
        })?;
        ctx.input.register_action(self.input_group, ActionDescriptor {
            name: CommonAction::RIGHT.to_string(),
            display_name: "Right".to_string(),
            description: "Layout navigation control (go right).".to_string(),
        })?;
        ctx.input.register_action(self.input_group, ActionDescriptor {
            name: CommonAction::CHANGE_CONTROL_MODE.to_string(),
            display_name: "Change Control Mode".to_string(),
            description: "Switch between selection and cursor control mode.".to_string(),
        })?;
        ctx.input.register_axis(self.input_group, AxisDescriptor {
            name: CommonAxis::CURSOR_X.to_string(),
            display_name: "Cursor X".to_string(),
            description: "Horizontal position of the mouse cursor relative to the screen.".to_string(),
            kind: AxisKind::Clamped { min: 0.0, max: SCREEN_WIDTH as f32 },
        })?;
        ctx.input.register_axis(self.input_group, AxisDescriptor {
            name: CommonAxis::CURSOR_Y.to_string(),
            display_name: "Cursor Y".to_string(),
            description: "Vertical position of the mouse cursor relative to the screen.".to_string(),
            kind: AxisKind::Clamped { min: 0.0, max: SCREEN_HEIGHT as f32 },
        })?;
        ctx.input.register_axis(self.input_group, AxisDescriptor {
            name: CommonAxis::CURSOR_MOTION_X.to_string(),
            display_name: "Cursor Motion X".to_string(),
            description: "Delta mouvement of the mouse on the horizontal axis.".to_string(),
            kind: AxisKind::Infinite,
        })?;
        ctx.input.register_axis(self.input_group, AxisDescriptor {
            name: CommonAxis::CURSOR_MOTION_Y.to_string(),
            display_name: "Cursor Motion Y".to_string(),
            description: "Delta mouvement of the mouse on the vertical axis.".to_string(),
            kind: AxisKind::Infinite,
        })?;
        ctx.input.register_axis(self.input_group, AxisDescriptor {
            name: CommonAxis::VIEW_X.to_string(),
            display_name: "View X".to_string(),
            description: "View horizontal delta movement.".to_string(),
            kind: AxisKind::Infinite,
        })?;
        ctx.input.register_axis(self.input_group, AxisDescriptor {
            name: CommonAxis::VIEW_Y.to_string(),
            display_name: "View Y".to_string(),
            description: "View vertical delta movement.".to_string(),
            kind: AxisKind::Infinite,
        })?;
        ctx.input.register_axis(self.input_group, AxisDescriptor { 
            name: CommonAxis::MOVE_FORWARD.to_string(), 
            display_name: "Move Forward".to_string(), 
            description: "".to_string(), 
            kind: AxisKind::Clamped { min: 0.0, max: 1.0 }, 
        })?;
        ctx.input.register_axis(self.input_group, AxisDescriptor { 
            name: CommonAxis::MOVE_BACKWARD.to_string(), 
            display_name: "Move Backward".to_string(), 
            description: "".to_string(), 
            kind: AxisKind::Clamped { min: 0.0, max: 1.0 }, 
        })?;
        ctx.input.register_axis(self.input_group, AxisDescriptor { 
            name: CommonAxis::MOVE_LEFT.to_string(), 
            display_name: "Move Left".to_string(), 
            description: "".to_string(), 
            kind: AxisKind::Clamped { min: 0.0, max: 1.0 }, 
        })?;
        ctx.input.register_axis(self.input_group, AxisDescriptor { 
            name: CommonAxis::MOVE_RIGHT.to_string(), 
            display_name: "Move Right".to_string(), 
            description: "".to_string(), 
            kind: AxisKind::Clamped { min: 0.0, max: 1.0 }, 
        })?;
        ctx.input.register_axis(self.input_group, AxisDescriptor { 
            name: CommonAxis::MOVE_UP.to_string(), 
            display_name: "Move Up".to_string(), 
            description: "".to_string(), 
            kind: AxisKind::Clamped { min: 0.0, max: 1.0 }, 
        })?;
        ctx.input.register_axis(self.input_group, AxisDescriptor { 
            name: CommonAxis::MOVE_DOWN.to_string(), 
            display_name: "Move Down".to_string(), 
            description: "".to_string(), 
            kind: AxisKind::Clamped { min: 0.0, max: 1.0 }, 
        })?;
                
        // Register default font
        let id = ctx.asset.register("default", self.asset_group, Font::default())
            .context("Failed to register default core font")?;
        ctx.asset.set_default::<Font>(id)
            .context("Failed to set default font asset")?;

        // Register default inuts
        let test_group = ctx.input.register_group("test", self.id).context("Failed to register test group")?;
        // let click = ctx.input.register_action("click", self.input_group)?;
        
        ctx.input.register_action(test_group, ActionDescriptor { 
            name: "roll_left".to_string(), 
            display_name: "Roll Left".to_string(), 
            description: "".to_string(), 
        }).unwrap();
        ctx.input.register_action(test_group, ActionDescriptor { 
            name: "roll_right".to_string(), 
            display_name: "Roll Right".to_string(), 
            description: "".to_string(), 
        }).unwrap();
        ctx.input.register_action(test_group, ActionDescriptor { 
            name: "switch_mode".to_string(), 
            display_name: "Switch Mode".to_string(), 
            description: "".to_string(), 
        }).unwrap();

        // Add initial control profile
        self.control_profile = self.control_layout.add_profile(ControlInputs {
            up: ctx.input.find_action(self.input_group, CommonAction::UP).unwrap().id,
            down: ctx.input.find_action(self.input_group, CommonAction::DOWN).unwrap().id,
            left: ctx.input.find_action(self.input_group, CommonAction::LEFT).unwrap().id,
            right: ctx.input.find_action(self.input_group, CommonAction::RIGHT).unwrap().id,
            cursor_x: ctx.input.find_axis(self.input_group, CommonAxis::CURSOR_X).unwrap().id, 
            cursor_y: ctx.input.find_axis(self.input_group, CommonAxis::CURSOR_Y).unwrap().id,
            cursor_motion_x: ctx.input.find_axis(self.input_group, CommonAxis::CURSOR_MOTION_X).unwrap().id,
            cursor_motion_y: ctx.input.find_axis(self.input_group, CommonAxis::CURSOR_MOTION_Y).unwrap().id,
        });

        self.control_layout.add_control(IRect::new(5, 5, 100, 50));
        self.control_layout.add_control(IRect::new(5, 200, 100, 50));

        // Import initial assets
        ctx.asset.iter_import::<Texture>().map(|e| e.id).collect::<Vec<_>>()
            .iter().for_each(|id| { ctx.asset.transfer::<Texture>(*id, self.asset_group)
                .expect("Failed to transfer asset"); });
        ctx.asset.iter_import::<Mesh>().map(|e| e.id).collect::<Vec<_>>()
            .iter().for_each(|id| { ctx.asset.transfer::<Mesh>(*id, self.asset_group)
                .expect("Failed to transfer asset"); });

        // Initialize world
        let texture = ctx.asset.find::<Texture>("alfred", self.asset_group)
            .context("Failed to get alfred texture")?.id;
        let alfred_material = ctx.asset.register("alfred", self.asset_group, Material {
            diffuse: texture,
        }).context("Failed to create alfred material")?;
        let texture = ctx.asset.find::<Texture>("car", self.asset_group).unwrap().id;
        let car_material = ctx.asset.register("car", self.asset_group, Material {
            diffuse: texture,
        }).context("Failed to create car material")?;
        let alfred_mesh = ctx.asset.find::<Mesh>("alfred", self.asset_group)
            .context("Failed to find alfred mesh")?.id;
        let car_mesh = ctx.asset.find::<Mesh>("car", self.asset_group)
            .context("Failed to find car mesh")?.id;
        let car_model = ctx.asset.register::<Model>("car", self.asset_group, Model { mesh: car_mesh, materials: Vec::from([car_material]) })
            .context("Failed to create car model")?;
        let alfred_model = ctx.asset.register::<Model>("alfred", self.asset_group, Model { mesh: alfred_mesh, materials: Vec::from([alfred_material, alfred_material, alfred_material]) })
            .context("Failed to create alfred model")?;
        
        self.world.spawn((
            LifecycleComponent::default(),
            TransformComponent {
                translation: Vec3::new(0.0, -7.0, 0.0),    
                rotation: Quat::IDENTITY,    
                scale: Vec3::new(0.5, 0.5, 0.5),    
            },
            RotatorComponent { speed: 90.0 },
            ModelComponent::from(alfred_model),
        ));
        self.world.spawn((
            LifecycleComponent::default(),
            TransformComponent::from_translation(Vec3::new(0.0, -7.0, 9.0)),
            ModelComponent::from(alfred_model),
        ));
        for i in 0..100 {
            self.world.spawn((
                LifecycleComponent::default(),
                TransformComponent::from_translation(
                    Vec3::new(((i / 10) * 5) as f32, 0.0,  -((i % 10) * 8) as f32
                )),
                ModelComponent::from(car_model),
                RotatorComponent { speed: -90.0 + rand::random::<f32>() * 90.0 * 2.0 }
            ));
        }
        self.world.spawn((
            LifecycleComponent::default(),
            TransformComponent::from_translation(Vec3::new(0.0, 0.0, 4.0)),
            ModelComponent::from(car_model),
            RotatorComponent { speed: 30.0 }
        ));
        let e = self.world.spawn((
            LifecycleComponent::default(),
            TransformComponent::from_translation(Vec3::new(0.0, 0.0, -10.0)),
            FreeFlyComponent {
                switch_mode: ctx.input.find_action(test_group, "switch_mode").unwrap().id,
                roll_left: ctx.input.find_action(test_group, "roll_left").unwrap().id,
                roll_right: ctx.input.find_action(test_group, "roll_right").unwrap().id,
                view_x: ctx.input.find_axis(self.input_group, CommonAxis::VIEW_X).unwrap().id,
                view_y: ctx.input.find_axis(self.input_group, CommonAxis::VIEW_Y).unwrap().id,
                move_forward: ctx.input.find_axis(self.input_group, CommonAxis::MOVE_FORWARD).unwrap().id,
                move_backward: ctx.input.find_axis(self.input_group, CommonAxis::MOVE_BACKWARD).unwrap().id,
                move_up: ctx.input.find_axis(self.input_group, CommonAxis::MOVE_UP).unwrap().id,
                move_down: ctx.input.find_axis(self.input_group, CommonAxis::MOVE_DOWN).unwrap().id,
                move_left: ctx.input.find_axis(self.input_group, CommonAxis::MOVE_LEFT).unwrap().id,
                move_right: ctx.input.find_axis(self.input_group, CommonAxis::MOVE_RIGHT).unwrap().id,
                free_mode: false,
                yaw: 0.0,
                pitch: 0.0,
            },
            CameraComponent::default(),
            ScriptStorageComponent::default(),
            RhaiScriptsComponent::default(),
        ));
        let group = ctx.asset.find_group("import").unwrap();
        let script = ctx.asset.find::<RhaiScript>("inventory", group.id).unwrap();
        self.rhai.compile(script.id, &script.data.source).unwrap();
        self.world.get::<&mut RhaiScriptsComponent>(e).unwrap().add(script.id).unwrap();

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

            // if ctx.input.find_action("toggle_layout").unwrap().is_just_pressed() {
            let group = ctx.input.find_group(CommonInput::GROUP).unwrap().id;
            let toggle_layout_id = ctx.input.find_action(group, CommonAction::CHANGE_CONTROL_MODE).unwrap().id;
            if ctx.input.action(toggle_layout_id).unwrap().is_just_pressed() {
                self.layout_active = !self.layout_active;
            }

            if self.layout_active {
                self.control_layout.update(ctx.input);
                let cb0 = self.control_layout.render();
                ctx.renderer.push_command_buffer(cb0);
            } else {
                system_free_fly(&mut self.world, ctx.input, ctx.delta_time as f32);
            }

            let id = ctx.asset.default::<Font>()
                .expect("Failed to find default font.").id;
            let cb1 = CommandBuffer::build_with(|builder| {
                builder
                .print((8, 8).into(), format!("dt : {:.2} ({:.1})", self.last_dt * 1000.0, 1.0 / self.last_dt).as_str(), id)
                .print((8, 17).into(), format!("dc : {}", ctx.renderer.statistics().draw_count).as_str(), id)
                .print((8, 26).into(), format!("tc : {}", ctx.renderer.statistics().triangle_count).as_str(), id)
                .print((8, 35).into(), format!("vp : {}x{}", ctx.renderer.statistics().viewport.0, ctx.renderer.statistics().viewport.1).as_str(), id)
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