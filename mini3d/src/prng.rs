pub struct PCG32 {
    state: u64,
    increment: u64,
}

impl PCG32 {

    pub fn new(seed: u64) -> Self {
        Self { state: seed, increment: seed }
    }
    
    #[allow(clippy::should_implement_trait)]
    pub fn next(&mut self) -> u32 {
        let old_state = self.state;
        self.state = old_state.wrapping_mul(6364136223846793005).wrapping_add(self.increment | 1);
        let xorshifted = ((old_state >> 18) ^ old_state) >> 27;
        let rot = old_state >> 59;
        ((xorshifted >> rot) | (xorshifted << ((-(rot as i64)) & 31))) as u32
    }
}