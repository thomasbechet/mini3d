#![no_std]

#[cfg(test)]
extern crate std;

extern crate alloc;

pub mod scheduler;

#[derive(Default)]
pub struct Runner {}

impl Runner {
    pub fn update(&mut self, context: &mut Context) {
        // Prepare frame stages
        self.scheduler.prepare_next_frame_stages();

        // Run stages
        // TODO: protect against infinite loops
        loop {
            // Acquire next node
            let node = self.scheduler.next_node(&self.containers);
            if node.is_none() {
                break;
            }
            let node = node.unwrap();

            // Execute node
            if node.count == 1 {
                // Find callback
                let callback = self.scheduler.callbacks[node.first as usize];

                // Run the callback
                callback(&mut ECS {
                    containers: &mut self.containers,
                    registry: &mut self.registry,
                    scheduler: &mut self.scheduler,
                });
            } else {
                // TODO: use thread pool
            }
        }
    }
}
