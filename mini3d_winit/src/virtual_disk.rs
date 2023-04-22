use mini3d::disk::backend::DiskBackend;

pub struct VirtualDisk {
    data: Vec<u8>,
}

impl VirtualDisk {
    pub fn new() -> Self {
        Self { data: Vec::new() }
    }
}

impl DiskBackend for VirtualDisk {
    
}