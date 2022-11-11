use anyhow::{Result, Context};
use hecs::World;
use serde::{Serialize, Deserialize};

use crate::{program::ProgramContext, asset::system_schedule::{SystemScheduleType, SystemSchedule}};

use self::system::SystemContext;

pub mod component;
pub mod system;

#[derive(Default, Serialize, Deserialize)]
pub struct SystemScheduler {
    systems: Vec<SystemScheduleType>,
}

impl SystemScheduler {
    pub(crate) fn run(&self, ctx: &mut ProgramContext, world: &mut World) -> Result<()> {
        let mut system_context = SystemContext {
            asset: ctx.asset,
            input: ctx.input,
            script: ctx.script,
            renderer: ctx.renderer,
            delta_time: ctx.delta_time,
        };
        for system in &self.systems {
            match system {
                SystemScheduleType::Builtin(system_uid) => {
                    let entry = ctx.system.systems.get(system_uid)
                        .context(format!("Builtin system with UID '{}' from scheduler was not registered", system_uid))?;           
                    entry.system.run(&mut system_context, world).context(format!("Error raised while executing system '{}'", entry.name))?;
                },
                SystemScheduleType::RhaiScript(_) => {
                    // TODO:
                },
            }
        }
        Ok(())
    }
}

#[derive(Default)]
pub struct ECS {
    pub world: World,
    scheduler: SystemScheduler,
}

impl ECS {

    pub fn new() -> Self {
        Self { ..Default::default() }
    }

    pub fn set_schedule(&mut self, schedule: &SystemSchedule) -> Result<()> {
        self.scheduler.systems = schedule.systems.clone();
        Ok(())
    }

    pub fn progress(&mut self, ctx: &mut ProgramContext) -> Result<()> {
        self.scheduler.run(ctx, &mut self.world)?;
        Ok(())
    }
}