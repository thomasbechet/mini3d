use super::Word;

#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord)]
pub(crate) struct Opcode(u8);

impl Opcode {
    pub(crate) const PUSH: Self = Self(0); // Push word at location
    pub(crate) const PUSHC: Self = Self(1); // Push constant at location
    pub(crate) const POP: Self = Self(2); // [COUNT:24][CODE:8]
    pub(crate) const POPS: Self = Self(3); // Pop single word
    pub(crate) const ADDF: Self = Self(4); // [f1][f2] -> [f1 + f2]
    pub(crate) const SUBF: Self = Self(5); // [f1][f2] -> [f1 - f2]
    pub(crate) const MULF: Self = Self(6); // [f1][f2] -> [f1 * f2]
    pub(crate) const DIVF: Self = Self(7); // [f1][f2] -> [f1 / f2]
    pub(crate) const ADDI: Self = Self(8); // [i1][i2] -> [i1 + i2]
    pub(crate) const SUBI: Self = Self(9); // [i1][i2] -> [i1 - i2]
    pub(crate) const MULI: Self = Self(10); // [i1][i2] -> [i1 * i2]
    pub(crate) const DIVI: Self = Self(11); // [i1][i2] -> [i1 / i2]
    pub(crate) const HALT: Self = Self(12);
}

pub(crate) type Operand = u32;

pub(crate) struct Instruction;

impl Instruction {

    pub(crate) fn decode(word: Word) -> (Opcode, Operand) {
        (Opcode((word & 0xFF) as u8), word >> 8)
    }

    pub(crate) fn encode(opcode: Opcode, operand: Operand) -> Word {
        (opcode.0 as Word) | (operand << 8)
    }
}

pub struct Program {
    pub(crate) instructions: Vec<Word>,
    pub(crate) constants: Vec<Word>,
}

impl Program {

    pub fn empty() -> Program {
        Program {
            instructions: Vec::new(),
            constants: Vec::new(),
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