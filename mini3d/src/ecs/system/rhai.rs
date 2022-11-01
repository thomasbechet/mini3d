use hecs::World;

use crate::{ecs::component::{rhai_scripts::{RhaiScriptsComponent, RhaiScriptState}, script_storage::ScriptStorageComponent}, rhai::{RhaiContext, script_storage::ScriptStorageHandle, input::InputManagerHandle}, program::ProgramContext};

pub fn system_rhai_update_scripts(
    world: &mut World,
    rhai: &mut RhaiContext,
    ctx: &mut ProgramContext,
) {
    for (_, (scripts, storage)) in world.query_mut::<(&mut RhaiScriptsComponent, Option<&mut ScriptStorageComponent>)>() {
        let mut scope = rhai::Scope::new();
        scope.push_constant("INPUT", <InputManagerHandle>::from(&mut *ctx));
        if let Some(storage) = storage {
            scope.push_constant("STORAGE", <ScriptStorageHandle>::from(storage));
        }
        for instance in &mut scripts.instances {
            match instance {
                Some(instance) => {
                    if let Some(entry) = instance.script.get_or_resolve(&ctx.asset) {
                        if instance.state == RhaiScriptState::Init {
                            rhai.call(entry, &mut scope, "init").expect("Failed to call init");
                            instance.state = RhaiScriptState::Update;
                        }
                        rhai.call(entry, &mut scope, "update").expect("Failed to call update");
                    }
                },
                None => {}
            }
        }
    }
}