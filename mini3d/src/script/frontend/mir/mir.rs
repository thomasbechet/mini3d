use crate::{
    script::constant::{ConstantId, ConstantTable},
    uid::UID,
};

use super::primitive::PrimitiveType;

pub(crate) type LocalId = u16;
pub(crate) type BasicBlockId = u16;
pub(crate) type FunctionId = u16;

#[derive(Clone, Copy)]
pub(crate) enum Operand {
    Constant(ConstantId),
    Local(LocalId),
}

struct OperandFormatter<'a> {
    operand: Operand,
    constants: &'a ConstantTable,
}

impl<'a> OperandFormatter<'a> {
    fn new(operand: Operand, constants: &'a ConstantTable) -> Self {
        Self { operand, constants }
    }
}

impl<'a> core::fmt::Display for OperandFormatter<'a> {
    fn fmt(&self, f: &mut core::fmt::Formatter<'_>) -> core::fmt::Result {
        match self.operand {
            Operand::Constant(id) => {
                write!(f, "{}", self.constants.format(id))
            }
            Operand::Local(id) => write!(f, "%{}", id),
        }
    }
}

pub(crate) enum Branch {
    Equal,
    NotEqual,
    LessEqual,
    GreaterEqual,
    Less,
    Greater,
}

pub(crate) enum Terminator {
    Branch {
        branch: Branch,
        lhs: Operand,
        rhs: Operand,
        true_block: BasicBlockId,
        false_block: BasicBlockId,
    },
    Jump {
        block: BasicBlockId,
    },
    Return {
        value: Option<Operand>,
    },
}

pub(crate) enum Instruction {
    Call {
        function: u16,
    },
    CallParameter {
        value: Operand,
    },
    Phi {
        dst: LocalId,
    },
    PhiParameter {
        block: BasicBlockId,
        value: Operand,
    },
    Add {
        dst: LocalId,
        lhs: Operand,
        rhs: Operand,
    },
    Sub {
        dst: LocalId,
        lhs: Operand,
        rhs: Operand,
    },
    Mul {
        dst: LocalId,
        lhs: Operand,
        rhs: Operand,
    },
    Div {
        dst: LocalId,
        lhs: Operand,
        rhs: Operand,
    },
    ReadComponent,
    WriteComponent,
}

#[derive(Default)]
pub(crate) struct BasicBlock {
    predecessors: Vec<BasicBlockId>,
    instructions: Vec<Instruction>,
    terminator: Option<Terminator>,
    sealed: bool,
}

pub(crate) struct Local {
    ty: PrimitiveType,
    id: LocalId,
}

pub(crate) struct Function {
    pub(crate) name: String,
    pub(crate) module: UID,
    pub(crate) entry: BasicBlockId,
}

#[derive(Default)]
#[warn(clippy::upper_case_acronyms)]
pub(crate) struct MIR {
    pub(crate) functions: Vec<Function>,
    pub(crate) blocks: Vec<BasicBlock>,
    pub(crate) constants: ConstantTable,
    pub(crate) locals: Vec<Local>,
}

impl MIR {
    pub(crate) fn add_local(&mut self, ty: PrimitiveType) -> LocalId {
        let id = self.locals.len() as LocalId;
        self.locals.push(Local { ty, id });
        id
    }

    fn add_basic_block(&mut self) -> BasicBlockId {
        let id = self.blocks.len() as BasicBlockId;
        self.blocks.push(BasicBlock::default());
        id
    }

    pub(crate) fn add_function(&mut self, name: String, module: UID) -> BasicBlockId {
        let entry = self.add_basic_block();
        self.functions.len() as FunctionId;
        self.functions.push(Function {
            name,
            module,
            entry,
        });
        entry
    }

    pub(crate) fn add_branch(
        &mut self,
        block: BasicBlockId,
        branch: Branch,
        lhs: Operand,
        rhs: Operand,
    ) -> (BasicBlockId, BasicBlockId) {
        let true_block = self.add_basic_block();
        let false_block = self.add_basic_block();
        let terminator = Terminator::Branch {
            branch,
            lhs,
            rhs,
            true_block,
            false_block,
        };
        self.blocks[block as usize].terminator = Some(terminator);
        (true_block, false_block)
    }

    pub(crate) fn add_instruction(&mut self, block: BasicBlockId, instruction: Instruction) {
        self.blocks[block as usize].instructions.push(instruction);
    }

    fn print_instruction(&self, instruction: &Instruction) {
        match instruction {
            Instruction::Call { function } => todo!(),
            Instruction::CallParameter { value } => todo!(),
            Instruction::Phi { dst } => todo!(),
            Instruction::PhiParameter { block, value } => todo!(),
            Instruction::Add { dst, lhs, rhs } => {
                println!(
                    "    add {} {} {}",
                    OperandFormatter::new(Operand::Local(*dst), &self.constants),
                    OperandFormatter::new(*lhs, &self.constants),
                    OperandFormatter::new(*rhs, &self.constants),
                );
            }
            Instruction::Sub { dst, lhs, rhs } => {
                println!(
                    "    sub {} {} {}",
                    OperandFormatter::new(Operand::Local(*dst), &self.constants),
                    OperandFormatter::new(*lhs, &self.constants),
                    OperandFormatter::new(*rhs, &self.constants),
                );
            }
            Instruction::Mul { dst, lhs, rhs } => {
                println!(
                    "    mul {} {} {}",
                    OperandFormatter::new(Operand::Local(*dst), &self.constants),
                    OperandFormatter::new(*lhs, &self.constants),
                    OperandFormatter::new(*rhs, &self.constants),
                );
            }
            Instruction::Div { dst, lhs, rhs } => {
                println!(
                    "    div {} {} {}",
                    OperandFormatter::new(Operand::Local(*dst), &self.constants),
                    OperandFormatter::new(*lhs, &self.constants),
                    OperandFormatter::new(*rhs, &self.constants),
                );
            }
            Instruction::ReadComponent => todo!(),
            Instruction::WriteComponent => todo!(),
        }
    }

    fn print_terminator(&self, terminator: &Terminator) {
        match terminator {
            Terminator::Branch {
                branch,
                lhs,
                rhs,
                true_block,
                false_block,
            } => {
                match branch {
                    Branch::Equal => print!("    beq"),
                    Branch::NotEqual => print!("    bne"),
                    Branch::Less => print!("    blt"),
                    Branch::LessEqual => print!("    ble"),
                    Branch::Greater => print!("    bgt"),
                    Branch::GreaterEqual => print!("    bge"),
                }
                println!(
                    "{} {} .L{} .L{}",
                    OperandFormatter::new(*lhs, &self.constants),
                    OperandFormatter::new(*rhs, &self.constants),
                    true_block,
                    false_block,
                );
            }
            Terminator::Jump { block } => {
                println!("    jmp .L{}", block);
            }
            Terminator::Return { value } => {
                if let Some(value) = value {
                    println!("    ret {}", OperandFormatter::new(*value, &self.constants),);
                } else {
                    println!("    ret");
                }
            }
        }
    }

    fn print_block(&self, block: BasicBlockId) {
        println!(".L{}:", block);
        let block = &self.blocks[block as usize];
        for instruction in &block.instructions {
            self.print_instruction(instruction);
        }
        if let Some(terminator) = &block.terminator {
            self.print_terminator(terminator);
        }
    }

    pub(crate) fn print(&self) {
        for function in &self.functions {
            println!("{}:", function.name);
            self.print_block(function.entry);
        }
    }
}

#[cfg(test)]
mod test {
    use super::*;
    #[test]
    fn test_print() {
        let mut mir = MIR::default();
        let constants = ConstantTable::default();
        let entry = mir.add_function("__main".to_string(), 0.into());
        let dst = mir.add_local(PrimitiveType::Float);
        let lhs = mir.add_constant(&PrimitiveValue::Float(1.0), &strings);
        let rhs = mir.add_constant(&PrimitiveValue::Float(1.0), &strings);
        mir.add_instruction(
            entry,
            Instruction::Add {
                dst,
                lhs: Operand::Constant(lhs),
                rhs: Operand::Constant(rhs),
            },
        );
        mir.print();
    }
}
