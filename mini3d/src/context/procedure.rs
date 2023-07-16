use crate::{ecs::scheduler::Invocation, utils::uid::UID};
use std::collections::VecDeque;

pub struct ProcedureContext<'a> {
    pub(crate) active_procedure: UID,
    pub(crate) frame_procedures: &'a mut VecDeque<UID>,
    pub(crate) next_frame_procedures: &'a mut VecDeque<UID>,
}

impl<'a> ProcedureContext<'a> {
    pub fn invoke(&mut self, procedure: UID, invocation: Invocation) {
        match invocation {
            Invocation::Immediate => {
                self.frame_procedures.push_front(procedure);
            }
            Invocation::EndFrame => {
                self.frame_procedures.push_back(procedure);
            }
            Invocation::NextFrame => {
                self.next_frame_procedures.push_back(procedure);
            }
        }
    }

    pub fn uid(&self) -> UID {
        self.active_procedure
    }
}
