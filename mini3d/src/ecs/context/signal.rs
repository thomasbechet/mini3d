use std::collections::VecDeque;

use crate::{ecs::scheduler::Invocation, utils::uid::UID};

pub struct ExclusiveSignalContext<'a> {
    pub(crate) active_signal: UID,
    pub(crate) frame_signals: &'a mut VecDeque<UID>,
    pub(crate) next_frame_signals: &'a mut VecDeque<UID>,
}

impl<'a> ExclusiveSignalContext<'a> {
    pub fn invoke(&mut self, signal: UID, invocation: Invocation) {
        match invocation {
            Invocation::Immediate => {
                self.frame_signals.push_front(signal);
            }
            Invocation::EndFrame => {
                self.frame_signals.push_back(signal);
            }
            Invocation::NextFrame => {
                self.next_frame_signals.push_back(signal);
            }
        }
    }

    pub fn uid(&self) -> UID {
        self.active_signal
    }
}

pub struct ParallelSignalContext {
    pub(crate) active_signal: UID,
}

impl ParallelSignalContext {
    pub fn uid(&self) -> UID {
        self.active_signal
    }
}
