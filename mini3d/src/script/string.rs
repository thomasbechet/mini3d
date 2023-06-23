#[derive(Debug, Clone, Copy)]
pub struct StringId(u32, u32);

#[derive(Default)]
pub(crate) struct StringTable {
    buffer: String,
}

impl StringTable {
    pub(crate) fn add(&mut self, value: &str) -> StringId {
        let start = self.buffer.len();
        self.buffer.push_str(value);
        let end = self.buffer.len();
        StringId(start as u32, end as u32)
    }

    pub(crate) fn clear(&mut self) {
        self.buffer.clear();
    }

    pub(crate) fn slice(&self, id: StringId) -> &str {
        &self.buffer[id.0 as usize..id.1 as usize]
    }

    pub(crate) fn print(&self) {
        println!("STRINGS:");
        println!("{}", self.buffer);
    }
}
