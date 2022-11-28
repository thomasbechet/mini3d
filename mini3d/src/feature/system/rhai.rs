use anyhow::Result;
use hecs::World;

use crate::{scene::SystemContext, rhai::{input::InputManagerHandle, script_storage::ScriptStorageHandle}, feature::component::{rhai_scripts::{RhaiScriptsComponent, RhaiScriptStatus}, script_storage::ScriptStorageComponent}};

pub fn update_scripts(ctx: &mut SystemContext, world: &mut World) -> Result<()> {
    for (_, (scripts, storage)) in world.query_mut::<(&mut RhaiScriptsComponent, Option<&mut ScriptStorageComponent>)>() {
        let mut scope = rhai::Scope::new();
        scope.push_constant("INPUT", <InputManagerHandle>::from(&mut *ctx));
        if let Some(storage) = storage {
            scope.push_constant("STORAGE", <ScriptStorageHandle>::from(storage));
        }
        for instance in &mut scripts.instances {
            match instance {
                Some(instance) => {
                    if instance.status == RhaiScriptStatus::Starting {
                        ctx.script.rhai.call(instance.uid, ctx.asset, &mut scope, "start")?;
                        instance.status = RhaiScriptStatus::Updating;
                    }
                    ctx.script.rhai.call(instance.uid, ctx.asset, &mut scope, "update")?;
                },
                None => {}
            }
        }
    }
    Ok(())
}