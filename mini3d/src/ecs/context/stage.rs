use crate::{
    ecs::{scheduler::Invocation},
    utils::uid::UID,
};

pub struct ExclusiveStageContext<'a> {
    pub(crate) stages: 
    pub(crate) frame_stages: &'a mut VecDeque<UID>,
    pub(crate) next_frame_stages: &'a mut VecDeque<UID>,
}

impl<'a> ExclusiveStageContext<'a> {
    pub fn invoke(&mut self, stage: UID, invocation: Invocation) {
        match invocation {
            Invocation::Immediate => {
                self.frame_stages.push_front(stage);
            }
            Invocation::EndFrame => {
                self.frame_stages.push_back(stage);
            }
            Invocation::NextFrame => {
                self.next_frame_stages.push_back(stage);
            }
        }
    }
}

pub struct ParallelStageContext {
    pub(crate) active_stage: UID,
}

impl ParallelStageContext {
    pub fn uid(&self) -> UID {
        self.active_stage
    }
}
