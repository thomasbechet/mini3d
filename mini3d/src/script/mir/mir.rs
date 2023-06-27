use crate::uid::UID;

use super::{
    primitive::PrimitiveType,
    value::{ValueId, ValueTable},
};

pub(crate) type LocalId = u16;
pub(crate) type InstructionId = u16;
pub(crate) type ConstantId = u16;
pub(crate) type BasicBlockId = u16;
pub(crate) type FunctionId = u16;

#[derive(Clone, Copy)]
pub(crate) enum Operand {
    Value(ValueId),
    Local(LocalId),
    Constant(ConstantId),
}

struct OperandFormatter<'a> {
    operand: Operand,
    constants: &'a Vec<Constant>,
    values: &'a ValueTable,
}

impl<'a> OperandFormatter<'a> {
    fn new(operand: Operand, constants: &'a Vec<Constant>, values: &'a ValueTable) -> Self {
        Self {
            operand,
            constants,
            values,
        }
    }
}

impl<'a> core::fmt::Display for OperandFormatter<'a> {
    fn fmt(&self, f: &mut core::fmt::Formatter<'_>) -> core::fmt::Result {
        match self.operand {
            Operand::Value(id) => {
                write!(f, "{}", self.values.format(id))
            }
            Operand::Local(id) => write!(f, "%{}", id),
            Operand::Constant(id) => write!(f, ""),
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
        function: FunctionId,
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

pub(crate) enum Function {
    Internal {
        return_ty: Option<PrimitiveType>,
        first_arg: Option<FunctionId>,
        entry: Option<BasicBlockId>,
        export: bool,
    },
    Argument {
        ty: PrimitiveType,
        next_arg: Option<FunctionId>,
    },
    External {
        module: UID,
        uid: UID,
    },
}

pub(crate) enum Constant {
    Internal {
        ty: PrimitiveType,
        value: Option<ValueId>,
        expression: Option<InstructionId>,
        export: bool,
    },
    External {
        module: UID,
        uid: UID,
    },
}

#[derive(Default)]
#[warn(clippy::upper_case_acronyms)]
pub(crate) struct MIR {
    pub(crate) values: ValueTable,
    pub(crate) blocks: Vec<BasicBlock>,
    pub(crate) locals: Vec<Local>,
    pub(crate) functions: Vec<Function>,
    pub(crate) constants: Vec<Constant>,
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
                    OperandFormatter::new(Operand::Local(*dst), &self.constants, &self.values),
                    OperandFormatter::new(*lhs, &self.constants, &self.values),
                    OperandFormatter::new(*rhs, &self.constants, &self.values),
                );
            }
            Instruction::Sub { dst, lhs, rhs } => {
                println!(
                    "    sub {} {} {}",
                    OperandFormatter::new(Operand::Local(*dst), &self.constants, &self.values),
                    OperandFormatter::new(*lhs, &self.constants, &self.values),
                    OperandFormatter::new(*rhs, &self.constants, &self.values),
                );
            }
            Instruction::Mul { dst, lhs, rhs } => {
                println!(
                    "    mul {} {} {}",
                    OperandFormatter::new(Operand::Local(*dst), &self.constants, &self.values),
                    OperandFormatter::new(*lhs, &self.constants, &self.values),
                    OperandFormatter::new(*rhs, &self.constants, &self.values),
                );
            }
            Instruction::Div { dst, lhs, rhs } => {
                println!(
                    "    div {} {} {}",
                    OperandFormatter::new(Operand::Local(*dst), &self.constants, &self.values),
                    OperandFormatter::new(*lhs, &self.constants, &self.values),
                    OperandFormatter::new(*rhs, &self.constants, &self.values),
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
                    " {} {} .L{} .L{}",
                    OperandFormatter::new(*lhs, &self.constants, &self.values),
                    OperandFormatter::new(*rhs, &self.constants, &self.values),
                    true_block,
                    false_block,
                );
            }
            Terminator::Jump { block } => {
                println!("    jmp .L{}", block);
            }
            Terminator::Return { value } => {
                if let Some(value) = value {
                    println!(
                        "    ret {}",
                        OperandFormatter::new(*value, &self.constants, &self.values),
                    );
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
        let lhs = mir.values.add_f32(1.0);
        let rhs = mir.values.add_f32(1.0);
        let values = ValueTable::default();
        let entry = mir.add_function("__main".to_string(), 0.into());
        let dst = mir.add_local(PrimitiveType::Float);
        mir.add_instruction(
            entry,
            Instruction::Add {
                dst,
                lhs: Operand::Value(lhs),
                rhs: Operand::Value(rhs),
            },
        );
        mir.print();
    }
}
