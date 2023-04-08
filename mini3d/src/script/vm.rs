use self::{program::{Program, Opcode, Instruction}, stack::Stack};

pub mod program;
mod stack;

pub(crate) type Word = u32;

pub struct VirtualMachine {
    program: Program,
    stack: Stack,
    ip: usize, // Instruction pointer : index of the current instruction executed
}

impl VirtualMachine {

    pub fn new(program: Program) -> VirtualMachine {
        VirtualMachine {
            program,
            stack: Stack::new(1024),
            ip: 0,
        }
    }

    pub fn execute(&mut self) {
        loop {
            let opcode = Instruction::decode_opcode(self.program.instructions[self.ip]);
            match opcode {
                Opcode::PUSH => {
                    let word_count = Instruction::decode_word_count(self.program.instructions[self.ip]);
                    let words = &self.program.instructions[self.ip + 1..self.ip + 1 + word_count];
                    self.stack.push_words(words);
                    self.ip += word_count;
                },
                Opcode::PUSHS => {
                    self.ip += 1;
                    self.stack.push_word(self.program.instructions[self.ip]);
                }
                Opcode::POP => {
                    let word_count = Instruction::decode_word_count(self.program.instructions[self.ip]);
                    self.stack.pop_words(word_count);
                },
                Opcode::ADDF => {
                    let a = self.stack.pop_float();
                    let b = self.stack.pop_float();
                    self.stack.push_float(a + b);
                },
                Opcode::ADDI => {
                    let a = self.stack.pop_int();
                    let b = self.stack.pop_int();
                    self.stack.push_int(a + b);
                },
                Opcode::MULI => {
                    let a = self.stack.pop_int();
                    let b = self.stack.pop_int();
                    self.stack.push_int(a * b);
                },
                Opcode::HALT => {
                    break;
                }
                _ => {}
            }
            // Next instruction
            self.ip += 1;
        }
        self.stack.print();
    }

    fn next_float(&mut self) -> f32 {
        self.ip += 1;
        Instruction::decode_float(self.program.instructions[self.ip])
    }

    fn next_int(&mut self) -> u32 {
        self.ip += 1;
        Instruction::decode_int(self.program.instructions[self.ip])
    }
}