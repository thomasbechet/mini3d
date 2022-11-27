use anyhow::Result;
use serde::{Serialize, Deserialize};

use crate::{graphics::{SCREEN_HEIGHT, command_buffer::{CommandBuffer, Command}}, math::rect::IRect, process::{ProcessContext, Process}, uid::UID};

#[derive(Serialize, Deserialize)]
struct TimeGraph {
    records: Vec<f64>,
    head: usize,
}

impl Default for TimeGraph {
    fn default() -> Self {
        Self {
            records: vec![0.0; 240],
            head: 0,
        }
    }
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
        let mut cb = CommandBuffer::empty();
        let mut current = self.head;
        let base_x = 5;
        let base_y = 5;
        let height = 60;
        cb.push(Command::DrawHLine { y: SCREEN_HEIGHT as i32 - base_y, x0: base_x, x1: self.records.len() as i32 });
        cb.push(Command::DrawVLine { x: base_x, y0: SCREEN_HEIGHT as i32 - base_y - height, y1: SCREEN_HEIGHT as i32 - base_y });
        loop {
            let vy = ((self.records[current] / (2.0 / 60.0)) * height as f64) as u32;
            let x = base_x + current as i32;
            let y = SCREEN_HEIGHT as i32 - base_y - vy as i32;
            cb.push(Command::FillRect { rect: IRect::new(x, y, 1, 1) });
            // cb.push(Command::DrawVLine { x, y0: y, y1: SCREEN_HEIGHT as i32 - base_y });
            current = (current + 1) % self.records.len();
            if current == self.head {
                break
            }
        }
        cb
    }
}

#[derive(Serialize, Deserialize)]
pub struct ProfilerProcess {
    toggle_action: UID,
    active: bool,
    dt_record: Vec<f64>,
    last_dt: f64,
    time_graph: TimeGraph,
}

impl ProfilerProcess {
    pub fn new(toggle_action: UID) -> Self {
        Self {
            toggle_action,
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
        if ctx.input.action(self.toggle_action)?.is_just_pressed() {
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

            let mut cb1 = CommandBuffer::empty();
            let font = UID::new("default");
            cb1.push(Command::Print { p: (8, 8).into(), text: format!("dt   : {:.2} ({:.1})", self.last_dt * 1000.0, 1.0 / self.last_dt), font });
            cb1.push(Command::Print { p: (8, 17).into(), text: format!("time : {:.2}", ctx.time), font });
            cb1.push(Command::Print { p: (8, 26).into(), text: format!("dc   : {}", ctx.renderer.statistics().draw_count), font });
            cb1.push(Command::Print { p: (8, 35).into(), text: format!("tc   : {}", ctx.renderer.statistics().triangle_count), font });
            cb1.push(Command::Print { p: (8, 44).into(), text: format!("vp   : {}x{}", ctx.renderer.statistics().viewport.0, ctx.renderer.statistics().viewport.1), font });
            ctx.renderer.push_command_buffer(cb1);
            ctx.renderer.push_command_buffer(self.time_graph.render());
        }

        Ok(())
    }
}