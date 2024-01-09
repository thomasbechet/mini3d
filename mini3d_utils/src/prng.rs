pub struct PCG32 {
    state: u64,
    increment: u64,
}

impl PCG32 {

    pub fn new(seed: u64) -> Self {
        Self { state: seed, increment: seed }
    }
    
    #[allow(clippy::should_implement_trait)]
    pub fn next_u32(&mut self) -> u32 {
        let old_state = self.state;
        self.state = old_state.wrapping_mul(6364136223846793005).wrapping_add(self.increment | 1);
        let xorshifted = ((old_state >> 18) ^ old_state) >> 27;
        let rot = old_state >> 59;
        ((xorshifted >> rot) | (xorshifted << ((-(rot as i64)) & 31))) as u32
    }

    pub fn next_f32(&mut self) -> f32 {
        // Generate uniformly distributed single precision number in [1,2)
        let next = (self.next_u32() >> 9) | 0x3f800000;
        f32::from_bits(next) - 1.0
    }
}