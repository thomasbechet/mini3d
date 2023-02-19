use anyhow::Result;

use crate::{rhai::{input::InputManagerHandle, script_storage::ScriptStorageHandle}, feature::component::{rhai_scripts::{RhaiScripts, RhaiScriptStatus}, script_storage::ScriptStorage}, context::SystemContext};

pub fn update_scripts(ctx: &SystemContext) -> Result<()> {
    let world = ctx.world().active();
    let scripts = world.view::<RhaiScripts>(RhaiScripts::UID)?;
    let storages = world.view::<ScriptStorage>(ScriptStorage::UID)?;

    // for e in &world.query(&[RhaiScripts::UID]) {
    //     let mut scope = rhai::Scope::new();
    //     scope.push_constant("INPUT", <InputManagerHandle>::from(&mut *ctx));
    //     if let Some(storage) = storages.get(e) {
    //         scope.push_constant("STORAGE", <ScriptStorageHandle>::from(storage));
    //     }
    //     for instance in &mut scripts.instances {
    //         match instance {
    //             Some(instance) => {
    //                 if instance.status == RhaiScriptStatus::Starting {
    //                     ctx.script.rhai.call(instance.uid, ctx.asset, &mut scope, "start")?;
    //                     instance.status = RhaiScriptStatus::Updating;
    //                 }
    //                 ctx.script.rhai.call(instance.uid, ctx.asset, &mut scope, "update")?;
    //             },
    //             None => {}
    //         }
    //     }
    // }

    Ok(())
}