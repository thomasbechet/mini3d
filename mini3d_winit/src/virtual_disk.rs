pub struct VirtualDisk {
    data: Vec<u8>,
}

impl VirtualDisk {
    pub fn new() -> Self {
        Self { data: Vec::new() }
    }
}
