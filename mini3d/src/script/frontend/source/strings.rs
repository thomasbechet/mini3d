#[derive(Debug, Clone, Copy)]
pub(crate) struct StringId(u32, u32);

#[derive(Default)]
pub(crate) struct StringTable {
    strings: String,
}

impl StringTable {
    pub(crate) fn add(&mut self, string: &str) -> StringId {
        let start = self.strings.len() as u32;
        self.strings.push_str(string);
        let end = self.strings.len() as u32;
        StringId(start, end)
    }

    pub(crate) fn get(&self, slice: StringId) -> &str {
        &self.strings[slice.0 as usize..slice.1 as usize]
    }
}
