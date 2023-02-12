fn run(ctx: &SystemContext, world: &mut World) -> Result<()> {

    let positions = world.view_mut::<Position>(Position::UID);
    let velocities = world.view::<Velocity>(Velocity::UID);
    for e in world.query(&[Position::UID, Velocity::UID]).iter() {
        positions[e] += velocities[e] * ctx.delta_time;
    }

    Ok(())
}