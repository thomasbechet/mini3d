use mini3d::{program::{ProgramId, ProgramBuilder, Program, ProgramContext}, asset::{GroupId, font::Font, texture::Texture, mesh::Mesh}, hecs::{World, PreparedQuery}, ecs::{component::{transform::TransformComponent, model::ModelComponent}, system::transform::system_transfer_model_transforms}, graphics::CommandBuffer, anyhow::{Result, Context}};

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
        let alfred_mesh = ctx.asset.find::<Mesh>("alfred", self.core_group)
            .context("Failed to find alfred mesh")?.id;
        let car_mesh = ctx.asset.find::<Mesh>("car", self.core_group)
            .context("Failed to find car mesh")?.id;
        self.world.spawn((
            TransformComponent::default(),
            ModelComponent::new(ctx.renderer, alfred_mesh)
        ));
        self.world.spawn((
            TransformComponent::default(),
            ModelComponent::new(ctx.renderer, car_mesh)
        ));

        Ok(())
    }

    fn update(&mut self, ctx: &mut ProgramContext) -> Result<()> {

        // Call ECS systems
        let mut query = PreparedQuery::<(&TransformComponent, &ModelComponent)>::new();
        system_transfer_model_transforms(&mut self.world, &mut query, ctx.renderer);

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