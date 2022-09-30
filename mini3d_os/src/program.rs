use mini3d::{program::{ProgramId, ProgramBuilder, Program, ProgramContext}, asset::{AssetGroupId, font::Font, texture::Texture, mesh::Mesh, material::Material}, hecs::{World, PreparedQuery}, ecs::{component::{transform::TransformComponent, model::ModelComponent, rotator::RotatorComponent, free_fly::FreeFlyComponent, camera::CameraComponent}, system::{transform::system_transfer_model_transforms, rotator::system_rotator, free_fly::system_free_fly, camera::system_update_camera}}, graphics::CommandBuffer, anyhow::{Result, Context}, backend::renderer::RendererModelDescriptor, glam::{Vec3, Quat}, input::{InputGroupId, control_layout::{ControlLayout, ControlProfileId, ControlBindings}, button::{ButtonInput}, axis::{AxisInput, AxisKind}}, slotmap::Key, math::rect::IRect};

pub struct OSProgram {
    id: ProgramId,
    asset_group: AssetGroupId,
    input_group: InputGroupId,
    world: World,
    control_layout: ControlLayout,
    control_profile: ControlProfileId,
    count: u32,
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
            count: 0
        }
    }
}

impl Program for OSProgram {
    
    fn start(&mut self, ctx: &mut ProgramContext) -> Result<()> {

        // Register core asset group
        self.asset_group = ctx.asset.register_group("core", self.id)
            .context("Failed to register core asset group")?;
        
        // Register core input group
        self.input_group = ctx.input.register_group("core", self.id)
            .context("Failed to register core input group")?;
        
        // Register default font
        let id = ctx.asset.register("default", self.asset_group, Font::default())
            .context("Failed to register default core font")?;
        ctx.asset.set_default::<Font>(id)
            .context("Failed to set default font asset")?;

        // Register default inuts
        // let click = ctx.input.register_button("click", self.input_group)?;
        ctx.input.register_axis("move_forward", self.input_group, AxisKind::Clamped { min: 0.0, max: 1.0 }).unwrap();
        ctx.input.register_axis("move_backward", self.input_group, AxisKind::Clamped { min: 0.0, max: 1.0 }).unwrap();
        ctx.input.register_axis("move_left", self.input_group, AxisKind::Clamped { min: 0.0, max: 1.0 }).unwrap();
        ctx.input.register_axis("move_right", self.input_group, AxisKind::Clamped { min: 0.0, max: 1.0 }).unwrap();
        ctx.input.register_axis("move_up", self.input_group, AxisKind::Clamped { min: 0.0, max: 1.0 }).unwrap();
        ctx.input.register_axis("move_down", self.input_group, AxisKind::Clamped { min: 0.0, max: 1.0 }).unwrap();
        ctx.input.register_button("switch_mode", self.input_group).unwrap();
        ctx.input.register_button("roll_left", self.input_group).unwrap();
        ctx.input.register_button("roll_right", self.input_group).unwrap();

        // Add initial control profile
        self.control_profile = self.control_layout.add_profile(ControlBindings {
            up: ctx.input.find_button(ButtonInput::UP).unwrap().id,
            down: ctx.input.find_button(ButtonInput::DOWN).unwrap().id,
            left: ctx.input.find_button(ButtonInput::LEFT).unwrap().id,
            right: ctx.input.find_button(ButtonInput::RIGHT).unwrap().id,
            cursor_x: ctx.input.find_axis(AxisInput::CURSOR_X).unwrap().id, 
            cursor_y: ctx.input.find_axis(AxisInput::CURSOR_Y).unwrap().id,
            motion_x: ctx.input.find_axis(AxisInput::MOTION_X).unwrap().id,
            motion_y: ctx.input.find_axis(AxisInput::MOTION_Y).unwrap().id,
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
        let texture = ctx.asset.find::<Texture>("alfred")
            .context("Failed to get alfred texture")?.id;
        let alfred_material = ctx.asset.register("alfred", self.asset_group, Material {
            diffuse: texture,
        }).context("Failed to create alfred material")?;
        let texture = ctx.asset.find::<Texture>("car").unwrap().id;
        let car_material = ctx.asset.register("car", self.asset_group, Material {
            diffuse: texture,
        }).context("Failed to create car material")?;
        let alfred_mesh = ctx.asset.find::<Mesh>("alfred")
            .context("Failed to find alfred mesh")?.id;
        let car_mesh = ctx.asset.find::<Mesh>("car")
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
        self.world.spawn((
            TransformComponent::from_translation(Vec3::new(4.0, 0.0, 0.0)),
            ModelComponent::new(ctx.renderer, &RendererModelDescriptor { 
                mesh: car_mesh, 
                materials: &[car_material], 
                dynamic_materials: &[] 
            })
        ));
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
                switch_mode: ctx.input.find_button("switch_mode").unwrap().id,
                roll_left: ctx.input.find_button("roll_left").unwrap().id,
                roll_right: ctx.input.find_button("roll_right").unwrap().id,
                look_x: ctx.input.find_axis(AxisInput::MOTION_X).unwrap().id,
                look_y: ctx.input.find_axis(AxisInput::MOTION_Y).unwrap().id,
                move_forward: ctx.input.find_axis("move_forward").unwrap().id,
                move_backward: ctx.input.find_axis("move_backward").unwrap().id,
                move_up: ctx.input.find_axis("move_up").unwrap().id,
                move_down: ctx.input.find_axis("move_down").unwrap().id,
                move_left: ctx.input.find_axis("move_left").unwrap().id,
                move_right: ctx.input.find_axis("move_right").unwrap().id,
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
        system_rotator(&mut self.world, ctx.delta_time);
        system_free_fly(&mut self.world, ctx.input, ctx.delta_time);
        system_transfer_model_transforms(&mut self.world, &mut query, ctx.renderer);
        system_update_camera(&mut self.world, ctx.renderer);

        // Custom code
        {
            self.control_layout.update(ctx.input);
            let cb0 = self.control_layout.render();
            let id = ctx.asset.default::<Font>()
                .expect("Failed to find default font.").id;
            let cb1 = CommandBuffer::build_with(|builder| {
                builder
                .print((8, 8).into(), format!("{} zefiozefjzoefijzeofijzoeifjâzpkeazêpfzeojfzoeijf", self.count).as_str(), id)
                .print((8, 32).into(), format!("{} zefiozefjzoefijzeofijzoeifjâzpkeazêpfzeojfzoeijf", self.count).as_str(), id)
                .print((8, 52).into(), format!("{} zefiozefjzoefijzeofijzoeifjâzpkeazêpfzeojfzoeijf", self.count).as_str(), id)
                .print((8, 70).into(), format!("{} zefiozefjzoefijzeofijzoeifjâzpkeazêpfzeojfzoeijf", self.count).as_str(), id)
                .print((8, 100).into(), format!("{} zefiozefjzoefijzeofijzoeifjâzpkeazêpfzeojfzoeijf", self.count).as_str(), id)
                .print((8, 150).into(), format!("{} {{|}}~éèê!\"#$%&\'()*+,-./:;<=>?[]^_`", self.count).as_str(), id)
                .print((8, 170).into(), format!("{} if self.is_defined() [], '''", self.count).as_str(), id)
            });
            ctx.renderer.push_command_buffer(cb0);
            ctx.renderer.push_command_buffer(cb1);
            self.count += 1;
        }

        Ok(())
    }

    fn stop(&mut self, _ctx: &mut ProgramContext) -> Result<()> { 
        Ok(()) 
    }
}