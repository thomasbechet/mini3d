use crate::uid::UID;

use super::{constant::ConstantId, primitive::PrimitiveType};

#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash)]
pub(crate) struct ExportId(u32);

pub(crate) enum Export {
    Function {
        ty: Option<PrimitiveType>,
        first_arg: Option<ExportId>,
    },
    FunctionArgument {
        ty: PrimitiveType,
        next_arg: Option<ExportId>,
    },
    Constant {
        ty: PrimitiveType,
        value: Option<ConstantId>,
    },
}

pub(crate) struct ExportEntry {
    pub(crate) ident: UID,
    pub(crate) export: Export,
}

#[derive(Default)]
pub(crate) struct ExportTable {
    exports: Vec<ExportEntry>,
}

impl ExportTable {
    pub(crate) fn is_complete(&self) -> bool {
        for entry in &self.exports {
            match &entry.export {
                Export::Function { ty, .. } => {
                    if ty.is_none() {
                        return false;
                    }
                }
                Export::Constant { ty, value } => {
                    if value.is_none() {
                        return false;
                    }
                }
                _ => {}
            }
        }
        return true;
    }
}
