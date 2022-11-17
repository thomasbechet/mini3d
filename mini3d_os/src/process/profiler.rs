use mini3d::{graphics::{CommandBuffer, SCREEN_HEIGHT}, math::rect::IRect, process::{ProcessBuilder, Process, ProcessContext}, uid::UID, anyhow::Result};

use crate::input::CommonAction;

struct TimeGraph {
    records: Vec<f64>,
    head: usize,
}

impl TimeGraph {
    pub fn new(count: usize) -> Self {
        Self {
            records: vec![0.0; count],
            head: 0,
        }
    }
    pub fn add(&mut self, value: f64) {
        self.records[self.head] = value;
        self.head = (self.head + 1) % self.records.len();
    }
    pub fn render(&self) -> CommandBuffer {
        let mut builder = CommandBuffer::builder();
        let mut current = self.head;
        let base_x = 5;
        let base_y = 5;
        let height = 60;
        builder.draw_hline(SCREEN_HEIGHT as i32 - base_y, base_x, self.records.len() as i32);
        builder.draw_vline(base_x, SCREEN_HEIGHT as i32 - base_y - height, SCREEN_HEIGHT as i32 - base_y);
        loop {
            let vy = ((self.records[current] / (2.0 / 60.0)) * height as f64) as u32;
            let x = base_x + current as i32;
            let y = SCREEN_HEIGHT as i32 - base_y - vy as i32;
            builder.fill_rect(IRect::new(x, y, 1, 1));
            // builder.draw_vline(x, y, SCREEN_HEIGHT as i32 - base_y);
            current = (current + 1) % self.records.len();
            if current == self.head {
                break
            }
        }
        builder.build()
    }
}

pub(crate) struct ProfilerProcess {
    active: bool,
    dt_record: Vec<f64>,
    last_dt: f64,
    time_graph: TimeGraph,
}

impl ProcessBuilder for ProfilerProcess {
    
    type BuildData = ();

    fn build(_uid: UID, _data: Self::BuildData) -> Self {
        Self {
            active: false,
            dt_record: Vec::new(),
            last_dt: 0.0,
            time_graph: TimeGraph::new(240),
        }
    }
}

impl Process for ProfilerProcess {

    fn post_update(&mut self, ctx: &mut ProcessContext) -> Result<()> {

        // Toggle active
        if ctx.input.action(UID::new(CommonAction::TOGGLE_PROFILER))?.is_just_pressed() {
            self.active = !self.active;
        }

        // Process
        if self.active {
            self.dt_record.push(ctx.delta_time);
            self.time_graph.add(ctx.delta_time);
            if self.dt_record.len() > 30 {
                self.dt_record.sort_by(|a, b| a.partial_cmp(b).unwrap());
                self.last_dt = self.dt_record[14];
                self.dt_record.clear();
            }

            let cb1 = CommandBuffer::build_with(|builder| {
                let font = UID::new("default");
                builder
                    .print((8, 8).into(), format!("dt : {:.2} ({:.1})", self.last_dt * 1000.0, 1.0 / self.last_dt).as_str(), font)
                    .print((8, 17).into(), format!("dc : {}", ctx.renderer.statistics().draw_count).as_str(), font)
                    .print((8, 26).into(), format!("tc : {}", ctx.renderer.statistics().triangle_count).as_str(), font)
                    .print((8, 35).into(), format!("vp : {}x{}", ctx.renderer.statistics().viewport.0, ctx.renderer.statistics().viewport.1).as_str(), font)
            });
            ctx.renderer.push_command_buffer(cb1);
            ctx.renderer.push_command_buffer(self.time_graph.render());
        }

        Ok(())
    }
}