use super::vm::Word;

pub struct Program {
    pub(crate) bytecodes: Vec<u8>,
    pub(crate) constants: Vec<Word>,
}

impl Program {

    pub fn empty() -> Program {
        Program {
            bytecodes: Vec::new(),
            constants: Vec::new(),
        }
    }

    pub fn put(mut self, byte: u8) -> Self {
        self.bytecodes.push(byte);
        self
    }

    pub fn put_int(mut self, value: u32) -> Self {
        self.bytecodes.extend_from_slice(&value.to_be_bytes());
        self
    }

    pub fn put_float(mut self, value: f32) -> Self {
        self.bytecodes.extend_from_slice(&value.to_be_bytes());
        self
    }
}