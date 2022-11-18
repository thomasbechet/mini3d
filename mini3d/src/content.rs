use anyhow::Result;

use crate::app::App;

pub mod asset;
pub mod component;
pub mod signal;
pub mod system;

pub(crate) fn register_core_content(app: &mut App) -> Result<()> {

    // Assets
    app.asset_manager.register::<asset::font::Font>("font")?;
    app.asset_manager.register::<asset::input_action::InputAction>("input_action")?;
    app.asset_manager.register::<asset::input_axis::InputAxis>("input_axis")?;
    app.asset_manager.register::<asset::input_table::InputTable>("input_table")?;
    app.asset_manager.register::<asset::material::Material>("material")?;
    app.asset_manager.register::<asset::mesh::Mesh>("mesh")?;
    app.asset_manager.register::<asset::model::Model>("model")?;
    app.asset_manager.register::<asset::rhai_script::RhaiScript>("rhai_script")?;
    app.asset_manager.register::<asset::system_schedule::SystemSchedule>("system_schedule")?;
    app.asset_manager.register::<asset::texture::Texture>("texture")?;

    // Components
    app.ecs_manager.register_component::<component::camera::CameraComponent>("camera")?;
    app.ecs_manager.register_component::<component::free_fly::FreeFlyComponent>("free_fly")?;
    app.ecs_manager.register_component::<component::lifecycle::LifecycleComponent>("lifecycle")?;
    app.ecs_manager.register_component::<component::model::ModelComponent>("model")?;
    app.ecs_manager.register_component::<component::rhai_scripts::RhaiScriptsComponent>("rhai_scripts")?;
    app.ecs_manager.register_component::<component::rotator::RotatorComponent>("rotator")?;
    app.ecs_manager.register_component::<component::script_storage::ScriptStorageComponent>("script_storage")?;
    app.ecs_manager.register_component::<component::transform::TransformComponent>("transform")?;

    // Systems
    app.ecs_manager.register_system("despawn_entities", system::despawn::run)?;
    app.ecs_manager.register_system("free_fly", system::free_fly::run)?;
    app.ecs_manager.register_system("renderer_check_lifecycle", system::renderer::check_lifecycle)?;
    app.ecs_manager.register_system("renderer_transfer_transforms", system::renderer::transfer_transforms)?;
    app.ecs_manager.register_system("renderer_update_camera", system::renderer::update_camera)?;
    app.ecs_manager.register_system("rhai_update_scripts", system::rhai::update_scripts)?;
    app.ecs_manager.register_system("rotator", system::rotator::run)?;

    // Signals
    app.signal_manager.register::<signal::command::CommandSignal>("command")?;

    Ok(())
}