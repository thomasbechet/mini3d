use mini3d::{program::{ProgramId, ProgramBuilder, Program, ProgramContext}, asset::{AssetGroupId, font::Font, texture::Texture, mesh::Mesh, material::Material}, hecs::{World, PreparedQuery}, ecs::{component::{transform::TransformComponent, model::ModelComponent, rotator::RotatorComponent, free_fly::FreeFlyComponent, camera::CameraComponent}, system::{transform::system_transfer_model_transforms, rotator::system_rotator, free_fly::system_free_fly, camera::system_update_camera}}, graphics::{CommandBuffer, SCREEN_WIDTH, SCREEN_HEIGHT, SCREEN_CENTER}, anyhow::{Result, Context}, backend::renderer::RendererModelDescriptor, glam::{Vec3, Quat}, input::{InputGroupId, control_layout::{ControlLayout, ControlProfileId, ControlInputs}, axis::{AxisKind, AxisDescriptor}, action::ActionDescriptor}, slotmap::Key, math::rect::IRect};

use crate::input::{OSAxis, OSAction, OSGroup};

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
        }
    }
}

impl Program for OSProgram {
    
    fn start(&mut self, ctx: &mut ProgramContext) -> Result<()> {

        // Register os asset group
        self.asset_group = ctx.asset.register_group(OSGroup::ASSET, self.id)
            .context("Failed to register os asset group")?;
        // Register os input group
        self.input_group = ctx.input.register_group(OSGroup::INPUT, self.id)
            .context("Failed to register os input group")?;

        // Register os inputs
        ctx.input.register_axis(self.input_group, AxisDescriptor {
            name: OSAxis::CURSOR_X.to_string(),
            display_name: "Cursor X".to_string(),
            description: "Horizontal position of the mouse cursor relative to the screen.".to_string(),
            kind: AxisKind::Clamped { min: 0.0, max: SCREEN_WIDTH as f32 },
        })?;
        ctx.input.register_axis(self.input_group, AxisDescriptor {
            name: OSAxis::CURSOR_Y.to_string(),
            display_name: "Cursor Y".to_string(),
            description: "Vertical position of the mouse cursor relative to the screen.".to_string(),
            kind: AxisKind::Clamped { min: 0.0, max: SCREEN_HEIGHT as f32 },
        })?;
        ctx.input.register_axis(self.input_group, AxisDescriptor {
            name: OSAxis::MOTION_X.to_string(),
            display_name: "Motion X".to_string(),
            description: "Delta mouvement of the mouse on the horizontal axis.".to_string(),
            kind: AxisKind::Infinite,
        })?;
        ctx.input.register_axis(self.input_group, AxisDescriptor {
            name: OSAxis::MOTION_Y.to_string(),
            display_name: "Motion Y".to_string(),
            description: "Delta mouvement of the mouse on the vertical axis.".to_string(),
            kind: AxisKind::Infinite,
        })?;
        ctx.input.register_action(self.input_group, ActionDescriptor {
            name: OSAction::UP.to_string(),
            display_name: "Up".to_string(),
            description: "Layout navigation control (go up).".to_string(),
        })?;
        ctx.input.register_action(self.input_group, ActionDescriptor {
            name: OSAction::LEFT.to_string(),
            display_name: "Left".to_string(),
            description: "Layout navigation control (go left).".to_string(),
        })?;
        ctx.input.register_action(self.input_group, ActionDescriptor {
            name: OSAction::DOWN.to_string(),
            display_name: "Down".to_string(),
            description: "Layout navigation control (go down).".to_string(),
        })?;
        ctx.input.register_action(self.input_group, ActionDescriptor {
            name: OSAction::RIGHT.to_string(),
            display_name: "Right".to_string(),
            description: "Layout navigation control (go right).".to_string(),
        })?;
                
        // Register default font
        let id = ctx.asset.register("default", self.asset_group, Font::default())
            .context("Failed to register default core font")?;
        ctx.asset.set_default::<Font>(id)
            .context("Failed to set default font asset")?;

        // Register default inuts
        let test_group = ctx.input.register_group("test", self.id).context("Failed to register test group")?;
        // let click = ctx.input.register_action("click", self.input_group)?;
        ctx.input.register_axis(test_group, AxisDescriptor { 
            name: "move_forward".to_string(), 
            display_name: "Move Forward".to_string(), 
            description: "".to_string(), 
            kind: AxisKind::Clamped { min: 0.0, max: 1.0 }, 
        }).unwrap();
        ctx.input.register_axis(test_group, AxisDescriptor { 
            name: "move_backward".to_string(), 
            display_name: "Move Backward".to_string(), 
            description: "".to_string(), 
            kind: AxisKind::Clamped { min: 0.0, max: 1.0 }, 
        }).unwrap();
        ctx.input.register_axis(test_group, AxisDescriptor { 
            name: "move_left".to_string(), 
            display_name: "Move Left".to_string(), 
            description: "".to_string(), 
            kind: AxisKind::Clamped { min: 0.0, max: 1.0 }, 
        }).unwrap();
        ctx.input.register_axis(test_group, AxisDescriptor { 
            name: "move_right".to_string(), 
            display_name: "Move Right".to_string(), 
            description: "".to_string(), 
            kind: AxisKind::Clamped { min: 0.0, max: 1.0 }, 
        }).unwrap();
        ctx.input.register_axis(test_group, AxisDescriptor { 
            name: "move_up".to_string(), 
            display_name: "Move Up".to_string(), 
            description: "".to_string(), 
            kind: AxisKind::Clamped { min: 0.0, max: 1.0 }, 
        }).unwrap();
        ctx.input.register_axis(test_group, AxisDescriptor { 
            name: "move_down".to_string(), 
            display_name: "Move Down".to_string(), 
            description: "".to_string(), 
            kind: AxisKind::Clamped { min: 0.0, max: 1.0 }, 
        }).unwrap();
        ctx.input.register_action(test_group, ActionDescriptor { 
            name: "switch_mode".to_string(), 
            display_name: "Switch Mode".to_string(), 
            description: "".to_string(), 
        }).unwrap();
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
            name: "toggle_layout".to_string(), 
            display_name: "Toggle Layout".to_string(), 
            description: "".to_string(), 
        }).unwrap();

        // Add initial control profile
        self.control_profile = self.control_layout.add_profile(ControlInputs {
            up: ctx.input.find_action(self.input_group, OSAction::UP).unwrap().id,
            down: ctx.input.find_action(self.input_group, OSAction::DOWN).unwrap().id,
            left: ctx.input.find_action(self.input_group, OSAction::LEFT).unwrap().id,
            right: ctx.input.find_action(self.input_group, OSAction::RIGHT).unwrap().id,
            cursor_x: ctx.input.find_axis(self.input_group, OSAxis::CURSOR_X).unwrap().id, 
            cursor_y: ctx.input.find_axis(self.input_group, OSAxis::CURSOR_Y).unwrap().id,
            motion_x: ctx.input.find_axis(self.input_group, OSAxis::MOTION_X).unwrap().id,
            motion_y: ctx.input.find_axis(self.input_group, OSAxis::MOTION_Y).unwrap().id,
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
        self.world.spawn((
            TransformComponent {
                translation: Vec3::new(0.0, -7.0, 0.0),    
                rotation: Quat::IDENTITY,    
                scale: Vec3::new(0.5, 0.5, 0.5),    
            },
            RotatorComponent {},
            ModelComponent::new(ctx.renderer, &RendererModelDescriptor {
                mesh: alfred_mesh,
                materials: &[alfred_material, alfred_material, alfred_material],
                dynamic_materials: &[],
            })
        ));
        self.world.spawn((
            TransformComponent::from_translation(Vec3::new(0.0, -7.0, 9.0)),
            ModelComponent::new(ctx.renderer, &RendererModelDescriptor {
                mesh: alfred_mesh,
                materials: &[alfred_material, alfred_material, alfred_material],
                dynamic_materials: &[],
            })
        ));
        for i in 0..100 {
            self.world.spawn((
                TransformComponent::from_translation(
                    Vec3::new(((i / 10) * 5) as f32, 0.0,  -((i % 10) * 8) as f32
                )),
                ModelComponent::new(ctx.renderer, &RendererModelDescriptor { 
                    mesh: car_mesh, 
                    materials: &[car_material], 
                    dynamic_materials: &[] 
                })
            ));
        }
        self.world.spawn((
            TransformComponent::from_translation(Vec3::new(0.0, 0.0, 4.0)),
            ModelComponent::new(ctx.renderer, &RendererModelDescriptor { 
                mesh: car_mesh, 
                materials: &[car_material], 
                dynamic_materials: &[] 
            }),
            RotatorComponent {}
        ));
        self.world.spawn((
            TransformComponent::from_translation(Vec3::new(0.0, 0.0, -10.0)),
            FreeFlyComponent {
                switch_mode: ctx.input.find_action(test_group, "switch_mode").unwrap().id,
                roll_left: ctx.input.find_action(test_group, "roll_left").unwrap().id,
                roll_right: ctx.input.find_action(test_group, "roll_right").unwrap().id,
                look_x: ctx.input.find_axis(self.input_group, OSAxis::MOTION_X).unwrap().id,
                look_y: ctx.input.find_axis(self.input_group, OSAxis::MOTION_Y).unwrap().id,
                move_forward: ctx.input.find_axis(test_group, "move_forward").unwrap().id,
                move_backward: ctx.input.find_axis(test_group, "move_backward").unwrap().id,
                move_up: ctx.input.find_axis(test_group, "move_up").unwrap().id,
                move_down: ctx.input.find_axis(test_group, "move_down").unwrap().id,
                move_left: ctx.input.find_axis(test_group, "move_left").unwrap().id,
                move_right: ctx.input.find_axis(test_group, "move_right").unwrap().id,
                free_mode: false,
                yaw: 0.0,
                pitch: 0.0,
            },
            CameraComponent::new(ctx.renderer),
        ));

        Ok(())
    }

    fn update(&mut self, ctx: &mut ProgramContext) -> Result<()> {

        // Call ECS systems
        let mut query = PreparedQuery::<(&mut TransformComponent, &ModelComponent)>::new();
        system_rotator(&mut self.world, ctx.delta_time as f32);
        system_transfer_model_transforms(&mut self.world, &mut query, ctx.renderer);
        system_update_camera(&mut self.world, ctx.renderer);

        // Custom code
        {
            // Compute fps
            self.dt_record.push(ctx.delta_time);
            if self.dt_record.len() > 30 {
                self.dt_record.sort_by(|a, b| a.partial_cmp(b).unwrap());
                self.last_dt = self.dt_record[14];
                self.dt_record.clear();
            }

            // if ctx.input.find_action("toggle_layout").unwrap().is_just_pressed() {
            let test_group = ctx.input.find_group("test").unwrap().id;
            let toggle_layout_id = ctx.input.find_action(test_group, "toggle_layout").unwrap().id;
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
        }

        Ok(())
    }

    fn stop(&mut self, _ctx: &mut ProgramContext) -> Result<()> { 
        Ok(()) 
    }
}