pub struct Opcode;

impl Opcode {
    // - [ADDR] + [STACK[ADDR]]
    pub const LOAD: u8 = 0;
    // - [ADDR][VALUE] + [] | STACK[ADDR] = VALUE
    pub const STORE: u8 = 1;
    // - [ADDR] + [CONSTANT[ADDR]]
    pub const PUSHC: u8 = 2;
    // - [] + [VALUE]
    pub const PUSHLB: u8 = 3;
    // - [] + [VALUE]
    pub const PUSHLH: u8 = 4;
    // - [] + [VALUE]
    pub const PUSHLW: u8 = 5;
    // - [VALUE] + []
    pub const POP: u8 = 6;
    // - [F1][F2] + [F1+F2]
    pub const ADDF: u8 = 7;
    // - [F1][F2] + [F1-F2]
    pub const SUBF: u8 = 8;
    // - [F1][F2] + [F1*F2]
    pub const MULF: u8 = 9;
    // - [F1][F2] + [F1/F2]
    pub const DIVF: u8 = 10;
    // - [I1][I2] + [I1+I2]
    pub const ADDI: u8 = 11;
    // - [I1][I2] + [I1-I2]
    pub const SUBI: u8 = 12;
    // - [I1][I2] + [I1*I2]
    pub const MULI: u8 = 13;
    // - [I1][I2] + [I1/I2]
    pub const DIVI: u8 = 14;
    // Interruption
    pub const INT: u8 = 15;
}