use mini3d::simulation::Simulation;
use mini3d_utils::stdout::StdoutLogger;

fn main() {
    let mut sim = Simulation::new(true);
    sim.set_logger_provider(StdoutLogger);
    loop {
        sim.progress(1.0 / 60.0).expect("Simulation error");
    }
}
