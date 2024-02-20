use mini3d_runtime::{Runtime, RuntimeConfig};

fn main() {
    let mut runtime = Runtime::new(RuntimeConfig::default());
    for _ in 0..10 {
        runtime.tick().expect("Runtime error");
    }
}
