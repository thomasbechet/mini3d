use super::Word;

#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub(crate) struct Opcode(u8);

impl Opcode {
    // [WORD_COUNT:24][OPCODE:8]
    pub(crate) const PUSH: Self = Self(0);
    pub(crate) const PUSHS: Self = Self(1); // Push single word
    // [WORD_COUNT:24][OPCODE:8]
    pub(crate) const POP: Self = Self(2);
    pub(crate) const POPS: Self = Self(3); // Pop single word
    // STACK: [f1][f2] -> [f1 + f2]
    pub(crate) const ADDF: Self = Self(4);
    // STACK: [f1][f2] -> [f1 - f2]
    pub(crate) const SUBF: Self = Self(5);
    pub(crate) const MULF: Self = Self(6);
    pub(crate) const DIVF: Self = Self(7);
    pub(crate) const ADDI: Self = Self(8);
    pub(crate) const SUBI: Self = Self(9);
    pub(crate) const MULI: Self = Self(10);
    pub(crate) const DIVI: Self = Self(11);
    pub(crate) const HALT: Self = Self(12);
}

pub(crate) struct Instruction;

impl Instruction {

    pub(crate) fn encode_put_words(count: u32) -> Word {
        count << 8 | Opcode::PUSH.0 as Word
    }

    pub(crate) fn encode_lit_int(value: u32) -> Word {
        value
    }

    pub(crate) fn encode_lit_float(value: f32) -> Word {
        value.to_bits()
    }

    pub(crate) fn encode_opcode(code: Opcode) -> Word {
        code.0 as Word
    }

    pub(crate) fn decode_opcode(word: Word) -> Opcode {
        Opcode((word & 0xFF) as u8)
    }

    pub(crate) fn decode_word_count(word: Word) -> usize {
        (word >> 8) as usize
    }

    pub(crate) fn decode_float(word: Word) -> f32 {
        f32::from_bits(word)
    }

    pub(crate) fn decode_int(word: Word) -> u32 {
        word
    }
}

pub struct Program {
    pub(crate) instructions: Vec<Word>,
}

impl Program {

    pub fn empty() -> Program {
        Program {
            instructions: Vec::new(),
        }
    }

    pub fn put_int(mut self, value: u32) -> Self {
        self.instructions.push(Instruction::encode_opcode(Opcode::PUSHS));
        self.instructions.push(Instruction::encode_lit_int(value));
        self
    }

    pub fn put_float(mut self, value: f32) -> Self {
        self.instructions.push(Instruction::encode_opcode(Opcode::PUSHS));
        self.instructions.push(Instruction::encode_lit_float(value));
        self
    }

    pub fn add_int(mut self) -> Self {
        self.instructions.push(Instruction::encode_opcode(Opcode::ADDI));
        self
    }

    pub fn mul_int(mut self) -> Self {
        self.instructions.push(Instruction::encode_opcode(Opcode::MULI));
        self
    }

    pub fn halt(mut self) -> Self {
        self.instructions.push(Instruction::encode_opcode(Opcode::HALT));
        self
    }
}