use mini3d::{program::{ProgramId, ProgramBuilder, Program, ProgramContext}, asset::{AssetGroupId, font::Font, texture::Texture, mesh::Mesh, material::Material}, hecs::{World, PreparedQuery}, ecs::{component::{transform::TransformComponent, model::ModelComponent, rotator::RotatorComponent}, system::{transform::system_transfer_model_transforms, rotator::system_rotator}}, graphics::CommandBuffer, anyhow::{Result, Context}, backend::renderer::RendererModelDescriptor, glam::Vec3, input::{InputGroupId, axis::AxisKind, control_layout::{ControlLayout, ControlProfileId}}, slotmap::Key};

use crate::input::{Button, Axis};

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
        let up = ctx.input.register_button(Button::UP, self.input_group)?;
        let down = ctx.input.register_button(Button::DOWN, self.input_group)?;
        let left = ctx.input.register_button(Button::LEFT, self.input_group)?;
        let right = ctx.input.register_button(Button::RIGHT, self.input_group)?;
        let click = ctx.input.register_button(Button::CLICK, self.input_group)?;
        let switch_control_mode = ctx.input.register_button(Button::SWITCH_CONTROL_MODE, self.input_group)?;

        let motion_x = ctx.input.register_axis(Axis::CURSOR_X, self.input_group, AxisKind::Infinite)?;
        let motion_y = ctx.input.register_axis(Axis::CURSOR_Y, self.input_group, AxisKind::Infinite)?;
        let switch_mode2 = ctx.input.register_button("switch2", self.input_group)?;

        // Add initial control profile
        self.control_profile = self.control_layout.add_profile(switch_control_mode, up, down, left, right, motion_x, motion_y, true);
        self.control_profile = self.control_layout.add_profile(switch_mode2, up, down, left, right, motion_x, motion_y, false);

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
            TransformComponent::from_translation(Vec3::new(0.0, -7.0, 0.0)),
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

        Ok(())
    }

    fn update(&mut self, ctx: &mut ProgramContext) -> Result<()> {

        // Call ECS systems
        let mut query = PreparedQuery::<(&TransformComponent, &ModelComponent)>::new();
        system_transfer_model_transforms(&mut self.world, &mut query, ctx.renderer);
        system_rotator(&mut self.world);

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