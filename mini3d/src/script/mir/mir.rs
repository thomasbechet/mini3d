use alloc::vec::Vec;

use crate::{
    slot_map_key,
    utils::slotmap::{Key, SlotMap},
};

use super::{
    data::{DataId, DataTable},
    instruction::{Instruction, InstructionKind, Operand},
    primitive::PrimitiveType,
};

slot_map_key!(BasicBlockKey);
slot_map_key!(FunctionKey);
slot_map_key!(ConstantKey);
slot_map_key!(LocalKey);
slot_map_key!(InstructionKey);

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
        true_block: BasicBlockKey,
        false_block: BasicBlockKey,
    },
    Jump {
        block: BasicBlockKey,
    },
    Return {
        value: Option<Operand>,
    },
}

#[derive(Default)]
pub(crate) struct BasicBlock {
    first: InstructionKey,
    last: InstructionKey,
    predecessors: Vec<BasicBlockKey>,
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
        first_arg: FunctionKey,
        entry_block: BasicBlockKey,
        export: bool,
    },
    External {
        path: DataId,
        name: DataId,
        return_ty: Option<PrimitiveType>,
    },
    Argument {
        ty: PrimitiveType,
        next_arg: FunctionKey,
    },
}

pub(crate) enum Constant {
    Internal {
        name: DataId,
        ty: PrimitiveType,
        value: Option<DataId>,
        expression_block: BasicBlockKey,
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
    pub(crate) instructions: SlotMap<InstructionKey, Instruction>,
    pub(crate) basic_blocks: SlotMap<BasicBlockKey, BasicBlock>,
    pub(crate) functions: SlotMap<FunctionKey, Function>,
    pub(crate) constants: SlotMap<ConstantKey, Constant>,
    pub(crate) data: DataTable,
}

impl MIR {
    // pub(crate) fn add_local(&mut self, ty: PrimitiveType) -> LocalId {
    //     let id = self.locals.len() as LocalId;
    //     self.locals.push(Local { ty, id });
    //     id
    // }

    fn add_basic_block(&mut self) -> BasicBlockKey {
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
        block: BasicBlockKey,
        branch: Branch,
        lhs: Operand,
        rhs: Operand,
    ) -> (BasicBlockKey, BasicBlockKey) {
        let true_block = self.add_basic_block();
        let false_block = self.add_basic_block();
        let terminator = Terminator::Branch {
            branch,
            lhs,
            rhs,
            true_block,
            false_block,
        };
        self.basic_blocks.get_mut(block).unwrap().terminator = Some(terminator);
        (true_block, false_block)
    }

    pub(crate) fn add_instruction(
        &mut self,
        block: BasicBlockKey,
        kind: InstructionKind,
        op0: Operand,
        op1: Operand,
        op2: Operand,
    ) -> InstructionKey {
        let id = self.instructions.add(Instruction {
            kind,
            op0,
            op1,
            op2,
            next: InstructionKey::null(),
            prev: InstructionKey::null(),
        });
        if self.basic_blocks.get(block).unwrap().last.is_null() {
            let block = self.basic_blocks.get_mut(block).unwrap();
            block.first = id;
            block.last = id;
        } else {
            let block = self.basic_blocks.get_mut(block).unwrap();
            self.instructions.get_mut(block.last).unwrap().next = id;
            self.instructions.get_mut(id).unwrap().prev = block.last;
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
