use crate::{script::string::StringTable, uid::UID};

use super::primitive::{PrimitiveType, PrimitiveValue};

pub(crate) type LocalId = u16;
pub(crate) type TemporaryId = u16;
pub(crate) type ConstantId = u16;
pub(crate) type BasicBlockId = u16;
pub(crate) type FunctionId = u16;

pub(crate) enum Value {
    Constant(ConstantId),
    Local(LocalId),
    Temporary(TemporaryId),
}

pub(crate) enum Destination {
    Local(LocalId),
    Temporary(TemporaryId),
}

pub(crate) enum Terminator {
    Branch {
        condition: Value,
        true_block: BasicBlockId,
        false_block: BasicBlockId,
    },
    Jump {
        block: BasicBlockId,
    },
    Return,
}

pub(crate) enum Instruction {
    Allocate {
        local: u16,
        ty: PrimitiveType,
    },
    Call {
        function: u16,
    },
    CallParameter {
        value: Value,
    },
    Phi {
        dst: Destination,
    },
    PhiParameter {
        block: BasicBlockId,
        value: Value,
    },
    Add {
        dst: Destination,
        lhs: Value,
        rhs: Value,
    },
    ReadComponent,
    WriteComponent,
}

pub(crate) struct BasicBlock {
    predecessors: Vec<BasicBlockId>,
    instructions: Vec<Instruction>,
    terminator: Option<Terminator>,
    sealed: bool,
}

pub(crate) struct Constant {
    pub(crate) ty: PrimitiveType,
    pub(crate) data: (u32, u32),
}

pub(crate) struct Function {
    pub(crate) uid: UID,
    pub(crate) name: (u32, u32),
    pub(crate) module: UID,
    pub(crate) entry: BasicBlockId,
}

#[derive(Default)]
#[warn(clippy::upper_case_acronyms)]
pub(crate) struct MIR {
    pub(crate) functions: Vec<Function>,
    pub(crate) blocks: Vec<BasicBlock>,
    pub(crate) constants: Vec<Constant>,
    pub(crate) data: Vec<u8>,
}

impl MIR {
    pub(crate) fn add_constant(
        &mut self,
        value: &PrimitiveValue,
        strings: &StringTable,
    ) -> ConstantId {
        match value {
            PrimitiveValue::Boolean(_) => todo!(),
            PrimitiveValue::Integer(_) => todo!(),
            PrimitiveValue::Float(_) => todo!(),
            PrimitiveValue::Vec2(_) => todo!(),
            PrimitiveValue::IVec2(_) => todo!(),
            PrimitiveValue::Vec3(_) => todo!(),
            PrimitiveValue::IVec3(_) => todo!(),
            PrimitiveValue::Vec4(_) => todo!(),
            PrimitiveValue::IVec4(_) => todo!(),
            PrimitiveValue::Mat4(_) => todo!(),
            PrimitiveValue::Quat(_) => todo!(),
            PrimitiveValue::String(_) => todo!(),
            PrimitiveValue::Entity(_) => todo!(),
            PrimitiveValue::Object(_) => todo!(),
        }
        0
    }
}
