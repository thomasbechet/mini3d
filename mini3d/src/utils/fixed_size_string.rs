pub struct FixedSizeString<const SIZE: usize> {
    data: [u8; SIZE],
}

impl<const SIZE: usize> FixedSizeString<SIZE> {
    pub fn set(&mut self, value: &str) {
        self.data.copy_from_slice(src) value.as_bytes()
    }
}
