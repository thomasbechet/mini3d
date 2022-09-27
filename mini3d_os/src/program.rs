use mini3d::{program::{ProgramId, ProgramBuilder, Program, ProgramContext}, asset::{GroupId, font::Font, texture::Texture, mesh::Mesh, material::Material}, hecs::{World, PreparedQuery}, ecs::{component::{transform::TransformComponent, model::ModelComponent, rotator::RotatorComponent}, system::{transform::system_transfer_model_transforms, rotator::system_rotator}}, graphics::CommandBuffer, anyhow::{Result, Context}, backend::renderer::RendererModelDescriptor, glam::Vec3};

pub struct OSProgram {
    id: ProgramId,
    core_group: GroupId, 
    world: World,
    count: u32,
}

impl ProgramBuilder for OSProgram {
    
    type BuildData = ();

    fn build(id: ProgramId, _data: Self::BuildData) -> Self {
        Self { id, core_group: GroupId::default(), world: Default::default(), count: 0 }
    }
}

impl Program for OSProgram {
    
    fn start(&mut self, ctx: &mut ProgramContext) -> Result<()> {

        // Register core asset group
        self.core_group = ctx.asset.register_group("core")
            .context("Failed to register core group")?;
        // Register default font
        let id = ctx.asset.register("default", self.core_group, Font::default())
            .context("Failed to register default core font")?;
        ctx.asset.set_default::<Font>(id)
            .context("Failed to set default font asset")?;

        // Import initial assets
        ctx.asset.iter_import::<Texture>().map(|e| e.id).collect::<Vec<_>>()
            .iter().for_each(|id| { ctx.asset.transfer::<Texture>(*id, self.core_group); });
        ctx.asset.iter_import::<Mesh>().map(|e| e.id).collect::<Vec<_>>()
            .iter().for_each(|id| { ctx.asset.transfer::<Mesh>(*id, self.core_group); });

        // Initialize world
        let texture = ctx.asset.find::<Texture>("alfred", self.core_group)
            .context("Failed to get alfred texture")?.id;
        let alfred_material = ctx.asset.register("alfred", self.core_group, Material {
            diffuse: texture,
        }).context("Failed to create alfred material")?;
        let texture = ctx.asset.find::<Texture>("car", self.core_group).unwrap().id;
        let car_material = ctx.asset.register("car", self.core_group, Material {
            diffuse: texture,
        }).context("Failed to create car material")?;
        let alfred_mesh = ctx.asset.find::<Mesh>("alfred", self.core_group)
            .context("Failed to find alfred mesh")?.id;
        let car_mesh = ctx.asset.find::<Mesh>("car", self.core_group)
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
            ctx.input.update();
            let cb0 = ctx.input.render();
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