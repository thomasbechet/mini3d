use crate::uid::UID;

use super::{
    mir::primitive::{PrimitiveType, PrimitiveValue},
    module::ModuleId,
};

#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash)]
pub(crate) struct ExportId(u32);

pub(crate) enum Export {
    Function {
        return_type: Option<PrimitiveType>,
        first_arg: Option<ExportId>,
    },
    FunctionArgument {
        arg_type: PrimitiveType,
        next_arg: Option<ExportId>,
    },
    Constant {
        value: PrimitiveValue,
    },
}

pub(crate) struct ExportEntry {
    pub(crate) ident: UID,
    pub(crate) export: Export,
    pub(crate) module: ModuleId,
}

#[derive(Default)]
pub(crate) struct ExportTable {
    exports: Vec<ExportEntry>,
}
