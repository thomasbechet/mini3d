use mini3d_core::simulation::{Simulation, SimulationConfig};

fn main() {
    let mut simulation = Simulation::new(SimulationConfig::default());
    for _ in 0..10 {
        simulation.tick().expect("Instance error");
    }
}
