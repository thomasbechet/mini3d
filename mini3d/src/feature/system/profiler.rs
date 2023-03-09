use anyhow::Result;
use serde::{Serialize, Deserialize};

use crate::{renderer::{SCREEN_HEIGHT, graphics::Graphics, color::Color}, process::{ProcessContext, Process}, uid::UID};

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
    
    pub fn render(&self, gfx: &mut Graphics) {
        let mut current = self.head;
        let base_x = 5;
        let base_y = 5;
        let height = 60;
        gfx.draw_hline(SCREEN_HEIGHT as i32 - base_y, base_x, self.records.len() as i32, Color::WHITE);
        gfx.draw_vline(base_x, SCREEN_HEIGHT as i32 - base_y - height, SCREEN_HEIGHT as i32 - base_y, Color::WHITE);
        loop {
            let vy0 = ((self.records[current] / (2.0 / 60.0)) * height as f64) as u32;
            let x0 = base_x + current as i32;
            let y0 = SCREEN_HEIGHT as i32 - base_y - vy0 as i32;
            gfx.draw_line((x0, y0).into(), (x0, y0).into(), Color::WHITE);
            current = (current + 1) % self.records.len();
            if current == self.head {
                break
            }
        }
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
            self.dt_record.push(ctx.delta_time());
            self.time_graph.add(ctx.delta_time());
            if self.dt_record.len() > 30 {
                self.dt_record.sort_by(|a, b| a.partial_cmp(b).unwrap());
                self.last_dt = self.dt_record[14];
                self.dt_record.clear();
            }

            self.time_graph.render(ctx.renderer.graphics());

            let statistics = ctx.renderer.statistics();
            let gfx = ctx.renderer.graphics();
            let font = UID::new("default");
            gfx.print((8, 8).into(), &format!("dt   : {:.2} ({:.1})", self.last_dt * 1000.0, 1.0 / self.last_dt), font);
            gfx.print((8, 17).into(), &format!("time : {:.2}", ctx.time), font);
            gfx.print((8, 26).into(), &format!("dc   : {}", statistics.draw_count), font);
            gfx.print((8, 35).into(), &format!("tc   : {}", statistics.triangle_count), font);
        }

        Ok(())
    }
}