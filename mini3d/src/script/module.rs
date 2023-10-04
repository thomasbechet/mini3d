use crate::{
    resource::handle::ResourceHandle,
    utils::uid::{ToUID, UID},
};

use super::{interface::InterfaceId, mir::primitive::PrimitiveType};

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct ModuleId(u32);

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Module {
    Source { resource: ResourceHandle },
    Node { resource: ResourceHandle },
    Interface { id: InterfaceId },
    Builtin,
}

#[derive(Debug)]
struct ModuleEntry {
    uid: UID,
    module: Module,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct ModuleSymbolId(u32);

#[derive(Debug)]
pub(crate) enum ModuleSymbol {
    Function {
        ident: UID,
        ty: PrimitiveType,
        first_arg: Option<ModuleSymbolId>,
    },
    Argument {
        ident: UID,
        ty: PrimitiveType,
        next_arg: Option<ModuleSymbolId>,
    },
    // BuiltinFunction {
    //     ident: UID,
    // },
    Constant {
        ident: UID,
        ty: PrimitiveType,
    },
}

struct ModuleSymbolEntry {
    module: ModuleId,
    symbol: ModuleSymbol,
}

#[derive(Default)]
pub(crate) struct ModuleTable {
    modules: Vec<ModuleEntry>,
    symbols: Vec<ModuleSymbolEntry>,
}

impl ModuleTable {
    pub(crate) fn add(&mut self, name: impl ToUID, module: Module) -> ModuleId {
        let uid = name.to_uid();
        if let Some(id) = self.find(uid) {
            return id;
        }
        let id = ModuleId(self.modules.len() as u32);
        self.modules.push(ModuleEntry { uid, module });
        id
    }

    pub(crate) fn find(&self, name: impl ToUID) -> Option<ModuleId> {
        let uid = name.to_uid();
        for (i, entry) in self.modules.iter().enumerate() {
            if entry.uid == uid {
                return Some(ModuleId(i as u32));
            }
        }
        None
    }

    pub(crate) fn get(&self, id: ModuleId) -> Option<&Module> {
        self.modules.get(id.0 as usize).map(|e| &e.module)
    }

    pub(crate) fn add_symbol(&mut self, module: ModuleId, symbol: ModuleSymbol) -> ModuleSymbolId {
        let id = self.symbols.len();
        self.symbols.push(ModuleSymbolEntry { module, symbol });
        ModuleSymbolId(id as u32)
    }

    pub(crate) fn get_symbol(&self, id: ModuleSymbolId) -> Option<&ModuleSymbol> {
        self.symbols.get(id.0 as usize).map(|entry| &entry.symbol)
    }

    pub(crate) fn get_symbol_mut(&mut self, id: ModuleSymbolId) -> Option<&mut ModuleSymbol> {
        self.symbols
            .get_mut(id.0 as usize)
            .map(|entry| &mut entry.symbol)
    }

    pub(crate) fn iter_symbols(
        &self,
        module: ModuleId,
    ) -> impl Iterator<Item = ModuleSymbolId> + '_ {
        self.symbols
            .iter()
            .enumerate()
            .filter(move |(_, entry)| {
                entry.module == module
                    && matches!(
                        entry.symbol,
                        ModuleSymbol::Function { .. } | ModuleSymbol::Constant { .. }
                    )
            })
            .map(|(id, _)| ModuleSymbolId(id as u32))
    }

    pub(crate) fn find_symbol(&self, module: ModuleId, name: UID) -> Option<ModuleSymbolId> {
        self.symbols
            .iter()
            .enumerate()
            .find(|(_, entry)| {
                entry.module == module
                    && match entry.symbol {
                        ModuleSymbol::Function { ident: n, .. }
                        | ModuleSymbol::Constant { ident: n, .. } => n == name,
                        _ => false,
                    }
            })
            .map(|(id, _)| ModuleSymbolId(id as u32))
    }

    pub(crate) fn reset(&mut self) {
        self.modules.clear();
        self.symbols.clear();
    }

    pub(crate) fn print(&self) {
        println!("MODULES:");
        for (id, entry) in self
            .modules
            .iter()
            .enumerate()
            .map(|(id, entry)| (ModuleId(id as u32), entry))
        {
            println!("- [{}]: {:?}", id.0, entry.module);
            for (id, symbol) in self
                .symbols
                .iter()
                .enumerate()
                .filter(|(_, symbol)| symbol.module == id)
                .map(|(id, symbol)| (ModuleSymbolId(id as u32), symbol))
            {
                println!("  - [{}]: {:?}", id.0, symbol.symbol);
            }
        }
    }
}
