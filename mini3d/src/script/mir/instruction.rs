use crate::utils::slotmap::SlotMap;

use super::{
    data::{DataId, DataTable},
    mir::{BasicBlockId, Constant, ConstantId, FunctionId, InstructionId, LocalId},
};

struct OperandFormatter<'a> {
    operand: Operand,
    constants: &'a SlotMap<Constant>,
    data: &'a DataTable,
}

// impl<'a> OperandFormatter<'a> {
//     fn new(operand: Operand, constants: &'a Vec<Constant>, values: &'a ValueTable) -> Self {
//         Self {
//             operand,
//             constants,
//             values,
//         }
//     }
// }

// impl<'a> core::fmt::Display for OperandFormatter<'a> {
//     fn fmt(&self, f: &mut core::fmt::Formatter<'_>) -> core::fmt::Result {
//         match self.operand {
//             Operand::Value(id) => {
//                 write!(f, "{}", self.values.format(id))
//             }
//             Operand::Local(id) => write!(f, "%{}", id),
//             Operand::Constant(id) => write!(f, ""),
//         }
//     }
// }

pub(crate) enum Operand {
    Local(LocalId),
    Data(DataId),
    Function(FunctionId),
    Constant(ConstantId),
    BasicBlock(BasicBlockId),
}

pub(crate) enum InstructionKind {
    Add,
    Sub,
    Div,
    Mul,
    Call,
    CallArgument,
    Phi,
    PhiArgument,
    ReadComponent,
    WriteComponent,
}

pub(crate) struct Instruction {
    pub(crate) kind: InstructionKind,
    pub(crate) op0: Operand,
    pub(crate) op1: Operand,
    pub(crate) op2: Operand,
    pub(crate) next: InstructionId,
    pub(crate) prev: InstructionId,
}
