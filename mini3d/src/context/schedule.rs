use std::collections::VecDeque;

use anyhow::Result;

use crate::uid::UID;

pub struct ScheduleContext<'a> {
    signal_queue: &'a mut VecDeque<UID>
}

impl<'a> ScheduleContext<'a> {

    pub(crate) fn new(signal_queue: &'a mut VecDeque<UID>) -> Self {
        Self { signal_queue }
    }

    pub fn signal(&mut self, signal: UID) -> Result<()> {
        self.signal_queue.push_back(signal);
        Ok(())
    }
}