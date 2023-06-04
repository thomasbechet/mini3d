use crate::uid::UID;

use super::{
    primitive::Primitive,
    string::{StringId, StringTable},
};

pub(crate) type SymbolId = u16;
pub(crate) type BlockId = u16;

#[derive(Debug, PartialEq)]
pub(crate) enum SymbolKind {
    Function { return_type: Option<Primitive> },
    Variable { var_type: Option<Primitive> },
    Constant { const_type: Option<Primitive> },
    Import,
}

#[derive(Debug)]
pub(crate) struct BlockEntry {
    parent: Option<BlockId>,
    last: Option<SymbolId>,
    previous_scope_symbol: Option<SymbolId>,
}

#[derive(Debug)]
pub(crate) struct Symbol {
    pub(crate) hash: UID,
    pub(crate) kind: Option<SymbolKind>,
    pub(crate) ident: StringId,
    pub(crate) block: BlockId,
}

impl Symbol {
    pub(crate) fn is_defined(&self) -> bool {
        self.kind.is_some()
    }
}

#[derive(Debug)]
pub(crate) struct SymbolEntry {
    pub(crate) symbol: Symbol,
    pub(crate) previous_in_block: Option<SymbolId>,
    pub(crate) previous_in_scope: Option<SymbolId>,
}

#[derive(Default, Debug)]
pub(crate) struct SymbolTable {
    pub(crate) symbols: Vec<SymbolEntry>,
    pub(crate) blocks: Vec<BlockEntry>,
}

impl SymbolTable {
    pub(crate) const GLOBAL_BLOCK: BlockId = 0;

    pub(crate) fn clear(&mut self) {
        self.symbols.clear();
        self.blocks.clear();
    }

    pub(crate) fn add_symbol(
        &mut self,
        strings: &StringTable,
        ident: StringId,
        kind: Option<SymbolKind>,
        block: BlockId,
    ) -> SymbolId {
        // Prepare entry
        let value = strings.slice(ident);
        let mut entry = SymbolEntry {
            symbol: Symbol {
                hash: value.into(),
                kind,
                ident,
                block,
            },
            previous_in_block: None,
            previous_in_scope: None,
        };
        let id = self.symbols.len() as u16;

        // Update previous in scope
        if let Some(last) = self.blocks[block as usize].last {
            entry.previous_in_scope = Some(last);
        } else {
            entry.previous_in_scope = self.blocks[block as usize].previous_scope_symbol;
        }

        // Update block list
        let block_entry = self.blocks.get_mut(block as usize).unwrap();
        entry.previous_in_block = block_entry.last;
        block_entry.last = Some(id);

        // Add symbol
        self.symbols.push(entry);
        self.symbols.len() as u16 - 1
    }

    pub(crate) fn add_block(&mut self, parent: Option<BlockId>) -> BlockId {
        // Compute previous_scope_symbol
        let previous_scope_symbol = parent.and_then(|p| {
            let parent = self.blocks.get(p as usize).unwrap();
            parent.last.or(parent.previous_scope_symbol)
        });

        // Add scope
        self.blocks.push(BlockEntry {
            parent,
            last: None,
            previous_scope_symbol,
        });
        self.blocks.len() as u16 - 1
    }

    pub(crate) fn find_in_block(
        &self,
        strings: &StringTable,
        ident: StringId,
        block: BlockId,
    ) -> Option<SymbolId> {
        let mut entry = self.blocks.get(block as usize).unwrap().last;
        loop {
            if let Some(id) = entry {
                let symbol = &self.symbols[id as usize].symbol;
                if symbol.hash == strings.slice(ident).into() {
                    return Some(id);
                }
                entry = self.symbols[id as usize].previous_in_block;
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
        let mut entry = self.blocks.get(block as usize).unwrap().last;
        loop {
            if let Some(id) = entry {
                let symbol = &self.symbols[id as usize].symbol;
                if symbol.hash == strings.slice(ident).into() {
                    return Some(id);
                }
                entry = self.symbols[id as usize].previous_in_scope;
            } else {
                return None;
            }
        }
    }

    pub(crate) fn get(&self, id: SymbolId) -> &Symbol {
        &self.symbols.get(id as usize).unwrap().symbol
    }

    pub(crate) fn get_mut(&mut self, id: SymbolId) -> &mut Symbol {
        &mut self.symbols.get_mut(id as usize).unwrap().symbol
    }

    pub(crate) fn print(&self, strings: &StringTable) {
        println!("SYMBOLS:");
        for (i, entry) in self.symbols.iter().enumerate() {
            println!(
                "- [{}] '{}' {:?}",
                i,
                strings.slice(entry.symbol.ident),
                entry.symbol
            );
        }
        println!("BLOCKS:");
        for (i, block) in self.blocks.iter().enumerate() {
            println!("- {} {:?}", i, block);
        }
    }
}
