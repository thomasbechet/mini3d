use crate::{
    script::{
        frontend::error::{CompileError, SyntaxError},
        mir::{primitive::PrimitiveType, value::ValueId},
    },
    uid::UID,
};

use super::{
    strings::{StringId, StringTable},
    token::Span,
};

#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash)]
pub struct SymbolId(u32);

impl From<usize> for SymbolId {
    fn from(value: usize) -> Self {
        Self(value as u32)
    }
}

impl SymbolId {
    fn index(self) -> usize {
        self.0 as usize
    }
}

#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash)]
pub(crate) struct BlockId(u32);

impl BlockId {
    fn index(self) -> usize {
        self.0 as usize
    }
}

#[derive(Debug)]
pub(crate) enum Symbol {
    Function {
        return_type: Option<PrimitiveType>,
        first_arg: Option<SymbolId>,
    },
    FunctionArgument {
        arg_type: PrimitiveType,
        next_arg: Option<SymbolId>,
    },
    Variable {
        var_type: Option<PrimitiveType>,
    },
    Constant {
        value: Option<ValueId>,
    },
    Import {
        module: UID,
        id: ExportId,
    },
    Module {
        path: StringId,
    },
}

#[derive(Debug)]
pub(crate) struct SourceBlockEntry {
    parent: Option<BlockId>,
    last: Option<SymbolId>,
    previous_scope_symbol: Option<SymbolId>,
}

#[derive(Debug)]
pub(crate) struct SymbolEntry {
    pub(crate) ident: StringId,
    pub(crate) hash: UID,
    pub(crate) block: BlockId,
    pub(crate) symbol: Option<Symbol>,
    pub(crate) previous_in_block: Option<SymbolId>,
    pub(crate) previous_in_scope: Option<SymbolId>,
}

#[derive(Default, Debug)]
pub(crate) struct SymbolTable {
    pub(crate) symbols: Vec<SymbolEntry>,
    pub(crate) blocks: Vec<SourceBlockEntry>,
    pub(crate) symbol_state: usize,
    pub(crate) block_state: usize,
}

impl SymbolTable {
    pub(crate) const GLOBAL_BLOCK: BlockId = BlockId(0);

    pub(crate) fn clear(&mut self) {
        self.symbols.clear();
        self.blocks.clear();
        self.symbol_state = 0;
        self.block_state = 0;
    }

    pub(crate) fn push_state(&mut self) {}

    pub(crate) fn pop_state(&mut self) {}

    fn add_symbol(
        &mut self,
        strings: &StringTable,
        ident: StringId,
        block: BlockId,
        symbol: Option<Symbol>,
    ) -> SymbolId {
        // Prepare entry
        let value = strings.get(ident);
        let mut entry = SymbolEntry {
            symbol,
            hash: value.into(),
            ident,
            block,
            previous_in_block: None,
            previous_in_scope: None,
        };
        let id = self.symbols.len();

        // Update previous in scope
        if let Some(last) = self.blocks[block.index()].last {
            entry.previous_in_scope = Some(last);
        } else {
            entry.previous_in_scope = self.blocks[block.index()].previous_scope_symbol;
        }

        // Update block list
        let block_entry = self.blocks.get_mut(block.index()).unwrap();
        entry.previous_in_block = block_entry.last;
        block_entry.last = Some(id.into());

        // Add symbol
        self.symbols.push(entry);
        SymbolId(self.symbols.len() as u32 - 1)
    }

    pub(crate) fn define_symbol(
        &mut self,
        strings: &StringTable,
        ident: StringId,
        block: BlockId,
        span: Span,
        symbol: Symbol,
        allow_shadowing: bool,
    ) -> Result<SymbolId, CompileError> {
        let exist = self.find_in_scope(strings, ident, block).is_some();
        if exist && !allow_shadowing {
            return Err(SyntaxError::SymbolAlreadyDefined { span }.into());
        }
        Ok(self.add_symbol(strings, ident, block, Some(symbol)))
    }

    pub(crate) fn get_mut(&mut self, id: SymbolId) -> Option<&mut Symbol> {
        self.symbols
            .get_mut(id.0 as usize)
            .and_then(|s| s.symbol.as_mut())
    }

    pub(crate) fn lookup_symbol(
        &mut self,
        strings: &StringTable,
        ident: StringId,
        block: BlockId,
    ) -> SymbolId {
        if let Some(id) = self.find_in_scope(strings, ident, block) {
            id
        } else {
            self.add_symbol(strings, ident, block, None)
        }
    }

    pub(crate) fn add_block(&mut self, parent: Option<BlockId>) -> BlockId {
        // Compute previous_scope_symbol
        let previous_scope_symbol = parent.and_then(|p| {
            let parent = self.blocks.get(p.index()).unwrap();
            parent.last.or(parent.previous_scope_symbol)
        });

        // Add scope
        self.blocks.push(SourceBlockEntry {
            parent,
            last: None,
            previous_scope_symbol,
        });
        BlockId(self.blocks.len() as u32 - 1)
    }

    pub(crate) fn find_in_block(
        &self,
        strings: &StringTable,
        ident: StringId,
        block: BlockId,
    ) -> Option<SymbolId> {
        let mut entry = self.blocks.get(block.index()).unwrap().last;
        loop {
            if let Some(id) = entry {
                let symbol = self.symbols.get(id.index()).unwrap();
                if symbol.hash == strings.get(ident).into() {
                    return Some(id);
                }
                entry = self.symbols[id.index()].previous_in_block;
            } else {
                return None;
            }
        }
    }

    pub(crate) fn find_in_scope(
        &self,
        strings: &StringTable,
        ident: StringId,
        block: BlockId,
    ) -> Option<SymbolId> {
        let mut entry = self.blocks.get(block.index()).unwrap().last;
        loop {
            if let Some(id) = entry {
                let symbol = &self.symbols.get(id.index()).unwrap();
                if symbol.hash == strings.get(ident).into() {
                    return Some(id);
                }
                entry = self.symbols[id.index()].previous_in_scope;
            } else {
                return None;
            }
        }
    }

    pub(crate) fn print(&self, strings: &StringTable) {
        println!("SYMBOLS:");
        for (i, entry) in self.symbols.iter().enumerate() {
            println!(
                "- [{}] '{}' {:?}",
                i,
                strings.get(entry.ident),
                entry.symbol
            );
        }
        println!("BLOCKS:");
        for (i, block) in self.blocks.iter().enumerate() {
            println!("- {} {:?}", i, block);
        }
    }
}
