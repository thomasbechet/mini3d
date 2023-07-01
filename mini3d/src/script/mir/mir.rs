use super::{
    data::{DataId, DataTable},
    instruction::{Instruction, InstructionKind, Operand},
    primitive::PrimitiveType,
    slotmap::{SlotId, SlotMap},
};

pub(crate) type BasicBlockId = SlotId<BasicBlock>;
pub(crate) type FunctionId = SlotId<Function>;
pub(crate) type ConstantId = SlotId<Constant>;
pub(crate) type LocalId = SlotId<Local>;
pub(crate) type InstructionId = SlotId<Instruction>;

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

#[derive(Default)]
pub(crate) struct BasicBlock {
    first: InstructionId,
    last: InstructionId,
    predecessors: Vec<BasicBlockId>,
    terminator: Option<Terminator>,
    sealed: bool,
}

pub(crate) struct Local {
    ty: PrimitiveType,
}

pub(crate) enum Function {
    Internal {
        name: DataId,
        return_ty: Option<PrimitiveType>,
        first_arg: FunctionId,
        entry_block: BasicBlockId,
        export: bool,
    },
    External {
        path: DataId,
        name: DataId,
        return_ty: Option<PrimitiveType>,
    },
    Argument {
        ty: PrimitiveType,
        next_arg: FunctionId,
    },
}

pub(crate) enum Constant {
    Internal {
        name: DataId,
        ty: PrimitiveType,
        value: Option<DataId>,
        expression_block: BasicBlockId,
        export: bool,
    },
    External {
        path: DataId,
        name: DataId,
        ty: PrimitiveType,
    },
}

#[derive(Default)]
#[warn(clippy::upper_case_acronyms)]
pub(crate) struct MIR {
    pub(crate) instructions: SlotMap<Instruction>,
    pub(crate) basic_blocks: SlotMap<BasicBlock>,
    pub(crate) functions: SlotMap<Function>,
    pub(crate) constants: SlotMap<Constant>,
    pub(crate) data: DataTable,
}

impl MIR {
    // pub(crate) fn add_local(&mut self, ty: PrimitiveType) -> LocalId {
    //     let id = self.locals.len() as LocalId;
    //     self.locals.push(Local { ty, id });
    //     id
    // }

    fn add_basic_block(&mut self) -> BasicBlockId {
        self.basic_blocks.add(BasicBlock::default())
    }

    // pub(crate) fn add_function(&mut self, name: String, module: UID) -> BasicBlockId {
    //     let entry = self.add_basic_block();
    //     self.functions.len() as FunctionId;
    //     self.functions.push(Function {
    //         name,
    //         module,
    //         entry,
    //     });
    //     entry
    // }

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
        self.basic_blocks.get_mut(block).terminator = Some(terminator);
        (true_block, false_block)
    }

    pub(crate) fn add_instruction(
        &mut self,
        block: BasicBlockId,
        kind: InstructionKind,
        op0: Operand,
        op1: Operand,
        op2: Operand,
    ) -> InstructionId {
        let id = self.instructions.add(Instruction {
            kind,
            op0,
            op1,
            op2,
            next: InstructionId::null(),
            prev: InstructionId::null(),
        });
        if self.basic_blocks.get(block).last.is_null() {
            let block = self.basic_blocks.get_mut(block);
            block.first = id;
            block.last = id;
        } else {
            let block = self.basic_blocks.get_mut(block);
            self.instructions.get_mut(block.last).next = id;
            self.instructions.get_mut(id).prev = block.last;
            block.last = id;
        }
        id
    }

    // fn print_instruction(&self, instruction: &Instruction) {
    //     match instruction {
    //         Instruction::Call { function } => todo!(),
    //         Instruction::CallParameter { value } => todo!(),
    //         Instruction::Phi { dst } => todo!(),
    //         Instruction::PhiParameter { block, value } => todo!(),
    //         Instruction::Add { dst, lhs, rhs } => {
    //             println!(
    //                 "    add {} {} {}",
    //                 OperandFormatter::new(Operand::Local(*dst), &self.constants, &self.data),
    //                 OperandFormatter::new(*lhs, &self.constants, &self.data),
    //                 OperandFormatter::new(*rhs, &self.constants, &self.data),
    //             );
    //         }
    //         Instruction::Sub { dst, lhs, rhs } => {
    //             println!(
    //                 "    sub {} {} {}",
    //                 OperandFormatter::new(Operand::Local(*dst), &self.constants, &self.data),
    //                 OperandFormatter::new(*lhs, &self.constants, &self.data),
    //                 OperandFormatter::new(*rhs, &self.constants, &self.data),
    //             );
    //         }
    //         Instruction::Mul { dst, lhs, rhs } => {
    //             println!(
    //                 "    mul {} {} {}",
    //                 OperandFormatter::new(Operand::Local(*dst), &self.constants, &self.data),
    //                 OperandFormatter::new(*lhs, &self.constants, &self.data),
    //                 OperandFormatter::new(*rhs, &self.constants, &self.data),
    //             );
    //         }
    //         Instruction::Div { dst, lhs, rhs } => {
    //             println!(
    //                 "    div {} {} {}",
    //                 OperandFormatter::new(Operand::Local(*dst), &self.constants, &self.data),
    //                 OperandFormatter::new(*lhs, &self.constants, &self.data),
    //                 OperandFormatter::new(*rhs, &self.constants, &self.data),
    //             );
    //         }
    //         Instruction::ReadComponent => todo!(),
    //         Instruction::WriteComponent => todo!(),
    //     }
    // }

    // fn print_terminator(&self, terminator: &Terminator) {
    //     match terminator {
    //         Terminator::Branch {
    //             branch,
    //             lhs,
    //             rhs,
    //             true_block,
    //             false_block,
    //         } => {
    //             match branch {
    //                 Branch::Equal => print!("    beq"),
    //                 Branch::NotEqual => print!("    bne"),
    //                 Branch::Less => print!("    blt"),
    //                 Branch::LessEqual => print!("    ble"),
    //                 Branch::Greater => print!("    bgt"),
    //                 Branch::GreaterEqual => print!("    bge"),
    //             }
    //             println!(
    //                 " {} {} .L{} .L{}",
    //                 OperandFormatter::new(*lhs, &self.constants, &self.data),
    //                 OperandFormatter::new(*rhs, &self.constants, &self.data),
    //                 true_block,
    //                 false_block,
    //             );
    //         }
    //         Terminator::Jump { block } => {
    //             println!("    jmp .L{}", block);
    //         }
    //         Terminator::Return { value } => {
    //             if let Some(value) = value {
    //                 println!(
    //                     "    ret {}",
    //                     OperandFormatter::new(*value, &self.constants, &self.data),
    //                 );
    //             } else {
    //                 println!("    ret");
    //             }
    //         }
    //     }
    // }

    // fn print_block(&self, block: BasicBlockId) {
    //     println!(".L{}:", block);
    //     let block = &self.blocks[block as usize];
    //     for instruction in &block.instructions {
    //         self.print_instruction(instruction);
    //     }
    //     if let Some(terminator) = &block.terminator {
    //         self.print_terminator(terminator);
    //     }
    // }

    // pub(crate) fn print(&self) {
    //     for function in &self.functions {
    //         println!("{}:", function.name);
    //         self.print_block(function.entry);
    //     }
    // }
}

// #[cfg(test)]
// mod test {
//     use super::*;
//     #[test]
//     fn test_print() {
//         let mut mir = MIR::default();
//         let lhs = mir.data.add_f32(1.0);
//         let rhs = mir.data.add_f32(1.0);
//         let values = ValueTable::default();
//         let entry = mir.add_function("__main".to_string(), 0.into());
//         let dst = mir.add_local(PrimitiveType::Float);
//         mir.add_instruction(
//             entry,
//             Instruction::Add {
//                 dst,
//                 lhs: Operand::Value(lhs),
//                 rhs: Operand::Value(rhs),
//             },
//         );
//         mir.print();
//     }
// }
