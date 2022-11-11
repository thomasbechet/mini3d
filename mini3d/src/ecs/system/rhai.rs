use anyhow::Result;
use hecs::World;

use crate::{ecs::component::{rhai_scripts::{RhaiScriptsComponent, RhaiScriptState}, script_storage::ScriptStorageComponent}, rhai::{script_storage::ScriptStorageHandle, input::InputManagerHandle}};

use super::{System, SystemContext};

pub struct RhaiUpdateScriptsSystem;

impl System for RhaiUpdateScriptsSystem {
    fn run(&self, ctx: &mut SystemContext, world: &mut World) -> Result<()> {
        for (_, (scripts, storage)) in world.query_mut::<(&mut RhaiScriptsComponent, Option<&mut ScriptStorageComponent>)>() {
            let mut scope = rhai::Scope::new();
            scope.push_constant("INPUT", <InputManagerHandle>::from(&mut *ctx));
            if let Some(storage) = storage {
                scope.push_constant("STORAGE", <ScriptStorageHandle>::from(storage));
            }
            for instance in &mut scripts.instances {
                match instance {
                    Some(instance) => {
                        if instance.state == RhaiScriptState::Init {
                            ctx.script.rhai.call(instance.uid, ctx.asset, &mut scope, "init")?;
                            instance.state = RhaiScriptState::Update;
                        }
                        ctx.script.rhai.call(instance.uid, ctx.asset, &mut scope, "update")?;
                    },
                    None => {}
                }
            }
        }
        Ok(())
    }
}