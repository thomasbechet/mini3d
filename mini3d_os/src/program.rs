use mini3d::{program::{ProgramId, ProgramBuilder, Program, ProgramContext}, asset::{AssetGroupId, font::Font, texture::Texture, mesh::Mesh, material::Material}, hecs::{World, PreparedQuery}, ecs::{component::{transform::TransformComponent, model::ModelComponent, rotator::RotatorComponent, free_fly::FreeFlyComponent, camera::CameraComponent}, system::{transform::system_transfer_model_transforms, rotator::system_rotator, free_fly::system_free_fly, camera::system_update_camera}}, graphics::{CommandBuffer, SCREEN_WIDTH, SCREEN_HEIGHT}, anyhow::{Result, Context}, backend::renderer::RendererModelDescriptor, glam::{Vec3, Quat}, input::{InputGroupId, control_layout::{ControlLayout, ControlProfileId, ControlInputs}, axis::AxisKind}, slotmap::Key, math::rect::IRect};

use crate::input::{OSAxis, OSAction, OSGroup};

pub struct OSProgram {
    id: ProgramId,
    asset_group: AssetGroupId,
    input_group: InputGroupId,
    world: World,
    control_layout: ControlLayout,
    control_profile: ControlProfileId,
    layout_active: bool,
    fps_record: Vec<f32>,
    last_fps: f32,
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
            fps_record: Vec::new(),
            last_fps: 0.0,
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
        ctx.input.register_axis(OSAxis::CURSOR_X, self.input_group, AxisKind::Clamped { min: 0.0, max: SCREEN_WIDTH as f32 }).unwrap();
        ctx.input.register_axis(OSAxis::CURSOR_Y, self.input_group, AxisKind::Clamped { min: 0.0, max: SCREEN_HEIGHT as f32 }).unwrap();
        ctx.input.register_axis(OSAxis::MOTION_X, self.input_group, AxisKind::Infinite).unwrap();
        ctx.input.register_axis(OSAxis::MOTION_Y, self.input_group, AxisKind::Infinite).unwrap();
        ctx.input.register_action(OSAction::UP, self.input_group).unwrap();
        ctx.input.register_action(OSAction::DOWN, self.input_group).unwrap();
        ctx.input.register_action(OSAction::LEFT, self.input_group).unwrap();
        ctx.input.register_action(OSAction::RIGHT, self.input_group).unwrap();
        
        // Register default font
        let id = ctx.asset.register("default", self.asset_group, Font::default())
            .context("Failed to register default core font")?;
        ctx.asset.set_default::<Font>(id)
            .context("Failed to set default font asset")?;

        // Register default inuts
        // let click = ctx.input.register_action("click", self.input_group)?;
        ctx.input.register_axis("move_forward", self.input_group, AxisKind::Clamped { min: 0.0, max: 1.0 }).unwrap();
        ctx.input.register_axis("move_backward", self.input_group, AxisKind::Clamped { min: 0.0, max: 1.0 }).unwrap();
        ctx.input.register_axis("move_left", self.input_group, AxisKind::Clamped { min: 0.0, max: 1.0 }).unwrap();
        ctx.input.register_axis("move_right", self.input_group, AxisKind::Clamped { min: 0.0, max: 1.0 }).unwrap();
        ctx.input.register_axis("move_up", self.input_group, AxisKind::Clamped { min: 0.0, max: 1.0 }).unwrap();
        ctx.input.register_axis("move_down", self.input_group, AxisKind::Clamped { min: 0.0, max: 1.0 }).unwrap();
        ctx.input.register_action("switch_mode", self.input_group).unwrap();
        ctx.input.register_action("roll_left", self.input_group).unwrap();
        ctx.input.register_action("roll_right", self.input_group).unwrap();
        ctx.input.register_action("toggle_layout", self.input_group).unwrap();

        // Add initial control profile
        self.control_profile = self.control_layout.add_profile(ControlInputs {
            up: ctx.input.find_action(OSAction::UP, self.input_group).unwrap().id,
            down: ctx.input.find_action(OSAction::DOWN, self.input_group).unwrap().id,
            left: ctx.input.find_action(OSAction::LEFT, self.input_group).unwrap().id,
            right: ctx.input.find_action(OSAction::RIGHT, self.input_group).unwrap().id,
            cursor_x: ctx.input.find_axis(OSAxis::CURSOR_X, self.input_group).unwrap().id, 
            cursor_y: ctx.input.find_axis(OSAxis::CURSOR_Y, self.input_group).unwrap().id,
            motion_x: ctx.input.find_axis(OSAxis::MOTION_X, self.input_group).unwrap().id,
            motion_y: ctx.input.find_axis(OSAxis::MOTION_Y, self.input_group).unwrap().id,
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
                switch_mode: ctx.input.find_action("switch_mode", self.input_group).unwrap().id,
                roll_left: ctx.input.find_action("roll_left", self.input_group).unwrap().id,
                roll_right: ctx.input.find_action("roll_right", self.input_group).unwrap().id,
                look_x: ctx.input.find_axis(OSAxis::MOTION_X, self.input_group).unwrap().id,
                look_y: ctx.input.find_axis(OSAxis::MOTION_Y, self.input_group).unwrap().id,
                move_forward: ctx.input.find_axis("move_forward", self.input_group).unwrap().id,
                move_backward: ctx.input.find_axis("move_backward", self.input_group).unwrap().id,
                move_up: ctx.input.find_axis("move_up", self.input_group).unwrap().id,
                move_down: ctx.input.find_axis("move_down", self.input_group).unwrap().id,
                move_left: ctx.input.find_axis("move_left", self.input_group).unwrap().id,
                move_right: ctx.input.find_axis("move_right", self.input_group).unwrap().id,
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
            self.fps_record.push(1.0 / ctx.delta_time as f32);
            if self.fps_record.len() > 30 {
                self.fps_record.sort_by(|a, b| a.partial_cmp(b).unwrap());
                self.last_fps = self.fps_record[14];
                self.fps_record.clear();
            }

            // if ctx.input.find_action("toggle_layout").unwrap().is_just_pressed() {
            let toggle_layout_id = ctx.input.find_action("toggle_layout", self.input_group).unwrap().id;
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
                .print((8, 8).into(), format!("fps : {:.1}", self.last_fps).as_str(), id)
                .print((8, 17).into(), format!("dc  : {}", ctx.renderer.statistics().draw_count).as_str(), id)
                .print((8, 26).into(), format!("tc  : {}", ctx.renderer.statistics().triangle_count).as_str(), id)
                .print((8, 35).into(), format!("vp  : {}x{}", ctx.renderer.statistics().viewport.0, ctx.renderer.statistics().viewport.1).as_str(), id)
            });
            ctx.renderer.push_command_buffer(cb1);
        }

        Ok(())
    }

    fn stop(&mut self, _ctx: &mut ProgramContext) -> Result<()> { 
        Ok(()) 
    }
}