use crate::{feature::common::program::Program, script::interpreter::opcode::Opcode};

pub(crate) type Word = u32;

pub struct VirtualMachine {
    stack: Vec<Word>,
    sp: i32, // Stack pointer : index of the top of the stack
    ip: i32, // Instruction pointer : index of the current instruction to execute
}

impl VirtualMachine {
    const INITIAL_STACK_SIZE: usize = 1024;

    pub fn new() -> VirtualMachine {
        Self {
            stack: vec![0; Self::INITIAL_STACK_SIZE],
            sp: -1, // Will be incremented to 0 when the first value is pushed
            ip: -1, // Will be incremented to 0 when the first instruction is executed
        }
    }

    fn print_stack(&self, program: &Program) {
        println!("bytecodes: {:?}", program.bytecodes);
        println!("{:#08X}", self.stack[self.sp as usize]);
    }

    #[inline]
    fn next_byte(&mut self, program: &Program) -> u8 {
        self.ip += 1;
        program.bytecodes[self.ip as usize]
    }

    #[inline]
    fn next_half(&mut self, program: &Program) -> u16 {
        let b0 = self.next_byte(program);
        let b1 = self.next_byte(program);
        u16::from_be_bytes([b0, b1])
    }

    #[inline]
    fn next_word(&mut self, program: &Program) -> Word {
        let b0 = self.next_byte(program);
        let b1 = self.next_byte(program);
        let b2 = self.next_byte(program);
        let b3 = self.next_byte(program);
        u32::from_be_bytes([b0, b1, b2, b3])
    }

    #[inline]
    fn push(&mut self, word: Word) {
        self.sp += 1;
        self.stack[self.sp as usize] = word;
    }

    #[inline]
    fn pop(&mut self) -> Word {
        let value = self.stack[self.sp as usize];
        self.sp -= 1;
        value
    }

    pub fn run(&mut self, program: &Program) {
        loop {
            let opcode = self.next_byte(program);
            println!("opcode: {}", opcode);
            match opcode {
                Opcode::LOAD => {
                    let addr = self.pop();
                    self.push(self.stack[addr as usize]);
                }
                Opcode::STORE => {
                    let addr = self.pop();
                    let value = self.pop();
                    self.stack[addr as usize] = value;
                }
                Opcode::PUSHC => {
                    unimplemented!();
                }
                Opcode::PUSHLB => {
                    let byte = self.next_byte(program);
                    self.push(byte as Word);
                }
                Opcode::PUSHLH => {
                    let half = self.next_half(program);
                    self.push(half as Word);
                }
                Opcode::PUSHLW => {
                    let word = self.next_word(program);
                    self.push(word);
                }
                Opcode::POP => {
                    self.pop();
                }
                Opcode::ADDF => {
                    let a = f32::from_bits(self.pop());
                    let b = f32::from_bits(self.pop());
                    self.push((a + b).to_bits());
                }
                Opcode::SUBF => {
                    let a = f32::from_bits(self.pop());
                    let b = f32::from_bits(self.pop());
                    self.push((a - b).to_bits());
                }
                Opcode::MULF => {
                    let a = f32::from_bits(self.pop());
                    let b = f32::from_bits(self.pop());
                    self.push((a * b).to_bits());
                }
                Opcode::DIVF => {
                    let a = f32::from_bits(self.pop());
                    let b = f32::from_bits(self.pop());
                    self.push((a / b).to_bits());
                }
                Opcode::ADDI => {
                    let a = self.pop();
                    let b = self.pop();
                    self.push(a + b);
                }
                Opcode::SUBI => {
                    let a = self.pop();
                    let b = self.pop();
                    self.push(a - b);
                }
                Opcode::MULI => {
                    let a = self.pop();
                    let b = self.pop();
                    self.push(a * b);
                }
                Opcode::DIVI => {
                    let a = self.pop();
                    let b = self.pop();
                    self.push(a / b);
                }
                Opcode::INT => {
                    break;
                }
                _ => {}
            }
        }
        self.print_stack(program);
    }
}
