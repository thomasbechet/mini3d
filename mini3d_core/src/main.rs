use mini3d_core::engine::{Engine, EngineConfig};

fn main() {
    let mut engine = Engine::new(EngineConfig::default());
    for _ in 0..10 {
        engine.tick().expect("Instance error");
    }
}
